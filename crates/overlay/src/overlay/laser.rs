// The controller laser pointer: a tiny solid-colour swapchain drawn as a thin
// quad from the controller to the panel hit point, billboarded toward the head.
// Ported from monado-frame.
use anyhow::Result;
use ash::vk;
use ash::vk::Handle as _;
use openxr as xr;

use crate::mathx::{cross, forward, normalize, quat_from_axes, quatf, vec3f};

use super::session::Gpu;

pub struct Laser {
    swapchain: xr::Swapchain<xr::Vulkan>,
    images: Vec<vk::Image>,
}

impl Laser {
    pub fn new(gpu: &Gpu, session: &xr::Session<xr::Vulkan>) -> Result<Self> {
        let swapchain = session.create_swapchain(&xr::SwapchainCreateInfo {
            create_flags: xr::SwapchainCreateFlags::EMPTY,
            usage_flags: xr::SwapchainUsageFlags::COLOR_ATTACHMENT | xr::SwapchainUsageFlags::TRANSFER_DST,
            format: gpu.format.as_raw() as _,
            sample_count: 1,
            width: 8,
            height: 8,
            face_count: 1,
            array_size: 1,
            mip_count: 1,
        })?;
        let images = swapchain.enumerate_images()?.into_iter().map(vk::Image::from_raw).collect();
        Ok(Self { swapchain, images })
    }

    /// Fill the laser texture with the accent colour (call per frame it's shown).
    /// Clears the image the runtime handed us, not always images[0], so the
    /// rotating swapchain doesn't flicker uncleared frames.
    pub fn fill(&mut self, gpu: &Gpu) -> Result<()> {
        let device = &gpu.device;
        let cmd = gpu.cmd;
        let index = self.swapchain.acquire_image()? as usize;
        self.swapchain.wait_image(xr::Duration::INFINITE)?;
        let image = self.images[index];
        let range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
        unsafe {
            device.reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())?;
            device.begin_command_buffer(cmd, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT))?;
            let to_dst = vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(range);
            device.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[], &[], &[to_dst]);
            let color = vk::ClearColorValue { float32: [0.29, 0.70, 0.82, 1.0] };
            device.cmd_clear_color_image(cmd, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &color, &[range]);
            let to_src = vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(range);
            device.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT, vk::DependencyFlags::empty(), &[], &[], &[to_src]);
            device.end_command_buffer(cmd)?;
            let cmds = [cmd];
            device.queue_submit(gpu.queue, &[vk::SubmitInfo::default().command_buffers(&cmds)], gpu.fence)?;
            device.wait_for_fences(&[gpu.fence], true, u64::MAX)?;
            device.reset_fences(&[gpu.fence])?;
        }
        self.swapchain.release_image()?;
        Ok(())
    }

    /// A thin quad from the controller (`aim`) to the hit point, billboarded
    /// toward the HMD.
    pub fn quad<'a>(&'a self, space: &'a xr::Space, aim: &xr::Posef, dist: f32, hmd: &xr::Posef) -> xr::CompositionLayerQuad<'a, xr::Vulkan> {
        let o = [aim.position.x, aim.position.y, aim.position.z];
        let dir = normalize(forward(aim));
        let mid = [o[0] + dir[0] * dist * 0.5, o[1] + dir[1] * dist * 0.5, o[2] + dir[2] * dist * 0.5];
        let to_view = normalize([hmd.position.x - mid[0], hmd.position.y - mid[1], hmd.position.z - mid[2]]);
        let x = normalize(cross(dir, to_view));
        let z = cross(x, dir);
        let q = quat_from_axes(x, dir, z);
        let sub = xr::SwapchainSubImage::new().swapchain(&self.swapchain).image_array_index(0).image_rect(xr::Rect2Di {
            offset: xr::Offset2Di { x: 0, y: 0 },
            extent: xr::Extent2Di { width: 8, height: 8 },
        });
        xr::CompositionLayerQuad::new()
            .space(space)
            .eye_visibility(xr::EyeVisibility::BOTH)
            .sub_image(sub)
            .pose(xr::Posef { orientation: quatf(q), position: vec3f(mid) })
            .size(xr::Extent2Df { width: 0.006, height: dist })
    }
}
