// OpenXR overlay session + Vulkan (vulkan_enable2) bootstrap.
//
// `Xr` owns the session and frame loop handles; `Gpu` owns the Vulkan device,
// queue, command buffer, render pass and allocator that panels render through.
use std::os::raw::c_char;
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{bail, Result};
use ash::vk;
use ash::vk::Handle as _;
use openxr as xr;

// The Vulkan entry must be reachable from the C callback the XR runtime invokes
// to resolve Vulkan symbols, so it lives in a static.
static VK_ENTRY: OnceLock<ash::Entry> = OnceLock::new();

unsafe extern "system" fn get_instance_proc_addr(
    instance: xr::sys::platform::VkInstance,
    name: *const c_char,
) -> Option<unsafe extern "system" fn()> {
    let entry = VK_ENTRY.get().expect("vk entry not initialised");
    let vk_instance = vk::Instance::from_raw(instance as _);
    (entry.static_fn().get_instance_proc_addr)(vk_instance, name)
}

/// XR session + the spaces / frame handles the loop needs.
pub struct Xr {
    pub instance: xr::Instance,
    pub session: xr::Session<xr::Vulkan>,
    pub frame_waiter: xr::FrameWaiter,
    pub frame_stream: xr::FrameStream<xr::Vulkan>,
    pub space: xr::Space,      // LOCAL — panels live here
    pub view_space: xr::Space, // VIEW — the head pose
    pub blend_mode: xr::EnvironmentBlendMode,
}

/// Vulkan device + the bits panels render with.
pub struct Gpu {
    pub device: ash::Device,
    pub queue: vk::Queue,
    pub queue_family_index: u32,
    pub cmd_pool: vk::CommandPool,
    pub cmd: vk::CommandBuffer,
    pub fence: vk::Fence,
    pub allocator: Arc<Mutex<gpu_allocator::vulkan::Allocator>>,
    pub render_pass: vk::RenderPass,
    pub format: vk::Format,
    pub srgb: bool,
}

/// Build the overlay session and Vulkan device. Fails clearly if the runtime
/// lacks the required extensions.
pub fn init(app_name: &str) -> Result<(Xr, Gpu)> {
    let entry = xr::Entry::linked();
    let available = entry.enumerate_extensions()?;
    if !available.khr_vulkan_enable2 {
        bail!("runtime is missing XR_KHR_vulkan_enable2");
    }
    if !available.extx_overlay {
        bail!("runtime is missing XR_EXTX_overlay (is this Monado?)");
    }
    let mut exts = xr::ExtensionSet::default();
    exts.khr_vulkan_enable2 = true;
    exts.extx_overlay = true;
    let xr_instance = entry.create_instance(
        &xr::ApplicationInfo {
            api_version: xr::Version::new(1, 0, 32),
            application_name: app_name,
            application_version: 0,
            engine_name: app_name,
            engine_version: 0,
        },
        &exts,
        &[],
    )?;
    let props = xr_instance.properties()?;
    log::info!("OpenXR runtime: {} {}", props.runtime_name, props.runtime_version);
    let system = xr_instance.system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)?;
    let _reqs = xr_instance.graphics_requirements::<xr::Vulkan>(system)?;
    let blend_mode = xr_instance
        .enumerate_environment_blend_modes(system, xr::ViewConfigurationType::PRIMARY_STEREO)?
        .first()
        .copied()
        .unwrap_or(xr::EnvironmentBlendMode::OPAQUE);

    // Vulkan via vulkan_enable2.
    let vk_entry = unsafe { ash::Entry::load() }?;
    VK_ENTRY.set(vk_entry).ok();
    let vk_entry = VK_ENTRY.get().unwrap();
    let app_info = vk::ApplicationInfo::default().api_version(vk::make_api_version(0, 1, 1, 0));
    let vk_instance_raw = unsafe {
        xr_instance
            .create_vulkan_instance(
                system,
                get_instance_proc_addr,
                std::ptr::from_ref(&vk::InstanceCreateInfo::default().application_info(&app_info)).cast(),
            )?
            .map_err(vk::Result::from_raw)?
    };
    let vk_instance = unsafe { ash::Instance::load(vk_entry.static_fn(), vk::Instance::from_raw(vk_instance_raw as _)) };
    let phys_raw = unsafe { xr_instance.vulkan_graphics_device(system, vk_instance_raw as _)? };
    let physical_device = vk::PhysicalDevice::from_raw(phys_raw as _);
    let queue_family_index = unsafe {
        vk_instance
            .get_physical_device_queue_family_properties(physical_device)
            .iter()
            .enumerate()
            .find(|(_, q)| q.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|(i, _)| i as u32)
            .ok_or_else(|| anyhow::anyhow!("no graphics queue family"))?
    };
    let priorities = [1.0f32];
    let queue_infos = [vk::DeviceQueueCreateInfo::default().queue_family_index(queue_family_index).queue_priorities(&priorities)];
    let device_create_info = vk::DeviceCreateInfo::default().queue_create_infos(&queue_infos);
    let vk_device_raw = unsafe {
        xr_instance
            .create_vulkan_device(system, get_instance_proc_addr, phys_raw as _, std::ptr::from_ref(&device_create_info).cast())?
            .map_err(vk::Result::from_raw)?
    };
    let device = unsafe { ash::Device::load(vk_instance.fp_v1_0(), vk::Device::from_raw(vk_device_raw as _)) };
    let queue = unsafe { device.get_device_queue(queue_family_index, 0) };
    let cmd_pool = unsafe {
        device.create_command_pool(
            &vk::CommandPoolCreateInfo::default().queue_family_index(queue_family_index).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
            None,
        )?
    };
    let cmd = unsafe {
        device.allocate_command_buffers(
            &vk::CommandBufferAllocateInfo::default().command_pool(cmd_pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(1),
        )?[0]
    };
    let fence = unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None)? };

    // Overlay session.
    let (session, frame_waiter, frame_stream) = unsafe {
        let raw = create_overlay_session(
            &xr_instance,
            system,
            &xr::vulkan::SessionCreateInfo {
                instance: vk_instance_raw as _,
                physical_device: phys_raw as _,
                device: vk_device_raw as _,
                queue_family_index,
                queue_index: 0,
            },
        )
        .map_err(|e| anyhow::anyhow!("xrCreateSession (overlay) failed: {:?}", e))?;
        xr::Session::<xr::Vulkan>::from_raw(xr_instance.clone(), raw, Box::new(()))
    };
    let space = session.create_reference_space(xr::ReferenceSpaceType::LOCAL, xr::Posef::IDENTITY)?;
    let view_space = session.create_reference_space(xr::ReferenceSpaceType::VIEW, xr::Posef::IDENTITY)?;

    // Swapchain format + render pass.
    let formats = session.enumerate_swapchain_formats()?;
    let preferred = [vk::Format::B8G8R8A8_SRGB, vk::Format::R8G8B8A8_SRGB, vk::Format::B8G8R8A8_UNORM, vk::Format::R8G8B8A8_UNORM];
    let format = preferred.into_iter().find(|w| formats.iter().any(|f| (*f as i64) == (w.as_raw() as i64))).unwrap_or(vk::Format::B8G8R8A8_SRGB);
    let srgb = matches!(format, vk::Format::B8G8R8A8_SRGB | vk::Format::R8G8B8A8_SRGB);
    log::info!("swapchain format {:?} srgb={}", format, srgb);

    let color_attachment = vk::AttachmentDescription::default()
        .format(format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let color_ref = [vk::AttachmentReference::default().attachment(0).layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
    let subpass = [vk::SubpassDescription::default().pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS).color_attachments(&color_ref)];
    let attachments = [color_attachment];
    let render_pass = unsafe {
        device.create_render_pass(&vk::RenderPassCreateInfo::default().attachments(&attachments).subpasses(&subpass), None)?
    };

    let allocator = Arc::new(Mutex::new(
        gpu_allocator::vulkan::Allocator::new(&gpu_allocator::vulkan::AllocatorCreateDesc {
            instance: vk_instance.clone(),
            device: device.clone(),
            physical_device,
            debug_settings: Default::default(),
            buffer_device_address: false,
            allocation_sizes: Default::default(),
        })
        .map_err(|e| anyhow::anyhow!("gpu-allocator init: {e}"))?,
    ));

    let xr = Xr { instance: xr_instance, session, frame_waiter, frame_stream, space, view_space, blend_mode };
    let gpu = Gpu { device, queue, queue_family_index, cmd_pool, cmd, fence, allocator, render_pass, format, srgb };
    Ok((xr, gpu))
}

unsafe fn create_overlay_session(
    instance: &xr::Instance,
    system: xr::SystemId,
    info: &xr::vulkan::SessionCreateInfo,
) -> std::result::Result<xr::sys::Session, xr::sys::Result> {
    use xr::sys::Handle;
    let overlay = xr::sys::SessionCreateInfoOverlayEXTX {
        ty: xr::sys::SessionCreateInfoOverlayEXTX::TYPE,
        next: std::ptr::null(),
        create_flags: xr::OverlaySessionCreateFlagsEXTX::EMPTY,
        session_layers_placement: 5,
    };
    let binding = xr::sys::GraphicsBindingVulkanKHR {
        ty: xr::sys::GraphicsBindingVulkanKHR::TYPE,
        next: (&raw const overlay).cast(),
        instance: info.instance,
        physical_device: info.physical_device,
        device: info.device,
        queue_family_index: info.queue_family_index,
        queue_index: info.queue_index,
    };
    let create_info = xr::sys::SessionCreateInfo {
        ty: xr::sys::SessionCreateInfo::TYPE,
        next: (&raw const binding).cast(),
        create_flags: xr::SessionCreateFlags::default(),
        system_id: system,
    };
    let mut out = xr::sys::Session::NULL;
    let r = (instance.fp().create_session)(instance.as_raw(), &raw const create_info, &raw mut out);
    if r.into_raw() >= 0 {
        Ok(out)
    } else {
        Err(r)
    }
}
