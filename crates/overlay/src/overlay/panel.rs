// A single egui surface rendered into an OpenXR swapchain and presented as a
// quad layer floating in space. Ported from monado-frame's PanelGfx, wrapped in
// methods that take the shared `Gpu`.
use anyhow::Result;
use ash::vk;
use ash::vk::Handle as _;
use openxr as xr;

use crate::mathx::{cross, forward, normalize, quat_from_axes, quatf, vec3f};
use crate::theme;

use super::session::Gpu;

const PPP: f32 = 1.5; // egui pixels-per-point (UI scale)

pub struct Panel {
    swapchain: xr::Swapchain<xr::Vulkan>,
    framebuffers: Vec<vk::Framebuffer>,
    ctx: egui::Context,
    renderer: egui_ash_renderer::Renderer,
    pub px: (u32, u32),
    pub size_m: (f32, f32),
    pub pose: xr::Posef,
    prev_pos: Option<egui::Pos2>,
    prev_down: bool,
}

impl Panel {
    /// Create a panel `px` pixels in size, `size_m` metres in the world, at `pose`.
    pub fn new(gpu: &Gpu, session: &xr::Session<xr::Vulkan>, px: (u32, u32), size_m: (f32, f32), pose: xr::Posef) -> Result<Self> {
        let swapchain = session.create_swapchain(&xr::SwapchainCreateInfo {
            create_flags: xr::SwapchainCreateFlags::EMPTY,
            usage_flags: xr::SwapchainUsageFlags::COLOR_ATTACHMENT,
            format: gpu.format.as_raw() as _,
            sample_count: 1,
            width: px.0,
            height: px.1,
            face_count: 1,
            array_size: 1,
            mip_count: 1,
        })?;
        let images: Vec<vk::Image> = swapchain.enumerate_images()?.into_iter().map(vk::Image::from_raw).collect();
        let framebuffers = make_framebuffers(&gpu.device, gpu.render_pass, gpu.format, &images, px)?;

        let ctx = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        ctx.set_fonts(fonts);
        theme::apply_style(&ctx);
        ctx.set_pixels_per_point(PPP);
        ctx.options_mut(|o| {
            o.input_options.max_click_dist = 80.0;
            o.input_options.max_click_duration = 3.0;
        });

        let renderer = egui_ash_renderer::Renderer::with_gpu_allocator(
            gpu.allocator.clone(),
            gpu.device.clone(),
            gpu.render_pass,
            egui_ash_renderer::Options { srgb_framebuffer: gpu.srgb, ..Default::default() },
        )
        .map_err(|e| anyhow::anyhow!("egui renderer init: {e}"))?;

        Ok(Self { swapchain, framebuffers, ctx, renderer, px, size_m, pose, prev_pos: None, prev_down: false })
    }

    /// Upload a texture into this panel's egui context (e.g. an avatar thumbnail).
    pub fn load_texture(&self, name: &str, image: egui::ColorImage) -> egui::TextureHandle {
        self.ctx.load_texture(name, image, egui::TextureOptions::LINEAR)
    }

    /// Run egui for one frame and render it into the swapchain. `pointer` is the
    /// controller hit in normalised (u, v, pressed) coordinates, if any.
    pub fn render(
        &mut self,
        gpu: &Gpu,
        alpha_mode: bool,
        pointer: Option<(f32, f32, bool)>,
        mut build: impl FnMut(&egui::Context),
    ) -> Result<()> {
        let device = &gpu.device;
        let pos = pointer.map(|(u, v, _)| egui::pos2(u * self.px.0 as f32 / PPP, v * self.px.1 as f32 / PPP));
        let down = pointer.is_some_and(|(_, _, d)| d);

        let mut events = Vec::new();
        if let Some(ps) = pos {
            events.push(egui::Event::PointerMoved(ps));
        } else if self.prev_pos.is_some() {
            events.push(egui::Event::PointerGone);
        }
        if down != self.prev_down {
            if let Some(ps) = pos.or(self.prev_pos) {
                events.push(egui::Event::PointerButton {
                    pos: ps,
                    button: egui::PointerButton::Primary,
                    pressed: down,
                    modifiers: egui::Modifiers::default(),
                });
            }
        }
        self.prev_pos = pos;
        self.prev_down = down;

        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(self.px.0 as f32 / PPP, self.px.1 as f32 / PPP),
            )),
            events,
            ..Default::default()
        };
        let out = self.ctx.run(raw, |ctx| {
            build(ctx);
            if let Some(ps) = pos {
                let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("cursor")));
                painter.circle_filled(ps, 5.0, theme::PRIMARY);
                painter.circle_stroke(ps, 5.0, egui::Stroke::new(1.5, egui::Color32::from_black_alpha(150)));
            }
        });

        let prims = self.ctx.tessellate(out.shapes, out.pixels_per_point);
        self.renderer
            .set_textures(gpu.queue, gpu.cmd_pool, &out.textures_delta.set)
            .map_err(|e| anyhow::anyhow!("set_textures: {e}"))?;

        let index = self.swapchain.acquire_image()?;
        self.swapchain.wait_image(xr::Duration::INFINITE)?;
        let clear = if alpha_mode { [0.0, 0.0, 0.0, 0.0] } else { [0.07, 0.07, 0.09, 1.0] };
        let cmd = gpu.cmd;
        unsafe {
            device.reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())?;
            device.begin_command_buffer(cmd, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT))?;
            let clears = [vk::ClearValue { color: vk::ClearColorValue { float32: clear } }];
            let rp = vk::RenderPassBeginInfo::default()
                .render_pass(gpu.render_pass)
                .framebuffer(self.framebuffers[index as usize])
                .render_area(vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: vk::Extent2D { width: self.px.0, height: self.px.1 } })
                .clear_values(&clears);
            device.cmd_begin_render_pass(cmd, &rp, vk::SubpassContents::INLINE);
            self.renderer
                .cmd_draw(cmd, vk::Extent2D { width: self.px.0, height: self.px.1 }, out.pixels_per_point, &prims)
                .map_err(|e| anyhow::anyhow!("cmd_draw: {e}"))?;
            device.cmd_end_render_pass(cmd);
            device.end_command_buffer(cmd)?;
            let cmds = [cmd];
            let submit = vk::SubmitInfo::default().command_buffers(&cmds);
            device.queue_submit(gpu.queue, &[submit], gpu.fence)?;
            device.wait_for_fences(&[gpu.fence], true, u64::MAX)?;
            device.reset_fences(&[gpu.fence])?;
        }
        self.renderer.free_textures(&out.textures_delta.free).map_err(|e| anyhow::anyhow!("free_textures: {e}"))?;
        self.swapchain.release_image()?;
        Ok(())
    }

    /// The composition quad to submit for this panel (call only after `render`).
    pub fn quad<'a>(&'a self, space: &'a xr::Space, alpha_mode: bool) -> xr::CompositionLayerQuad<'a, xr::Vulkan> {
        let sub = xr::SwapchainSubImage::new().swapchain(&self.swapchain).image_array_index(0).image_rect(xr::Rect2Di {
            offset: xr::Offset2Di { x: 0, y: 0 },
            extent: xr::Extent2Di { width: self.px.0 as i32, height: self.px.1 as i32 },
        });
        let mut q = xr::CompositionLayerQuad::new()
            .space(space)
            .eye_visibility(xr::EyeVisibility::BOTH)
            .sub_image(sub)
            .pose(self.pose)
            .size(xr::Extent2Df { width: self.size_m.0, height: self.size_m.1 });
        if alpha_mode {
            q = q.layer_flags(xr::CompositionLayerFlags::BLEND_TEXTURE_SOURCE_ALPHA);
        }
        q
    }
}

fn make_framebuffers(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    format: vk::Format,
    images: &[vk::Image],
    px: (u32, u32),
) -> Result<Vec<vk::Framebuffer>> {
    let range = vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
    };
    let mut fbs = Vec::with_capacity(images.len());
    for &img in images {
        let view = unsafe {
            device.create_image_view(
                &vk::ImageViewCreateInfo::default().image(img).view_type(vk::ImageViewType::TYPE_2D).format(format).subresource_range(range),
                None,
            )?
        };
        let atts = [view];
        let fb = unsafe {
            device.create_framebuffer(
                &vk::FramebufferCreateInfo::default().render_pass(render_pass).attachments(&atts).width(px.0).height(px.1).layers(1),
                None,
            )?
        };
        fbs.push(fb);
    }
    Ok(fbs)
}

/// An identity-orientation pose at position `p`.
pub fn posef(p: [f32; 3]) -> xr::Posef {
    xr::Posef { orientation: xr::Quaternionf { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }, position: xr::Vector3f { x: p[0], y: p[1], z: p[2] } }
}

/// A panel pose `dist` metres ahead of the head (and `lateral` to the side),
/// upright and facing the user.
pub fn front_pose(h: &xr::Posef, dist: f32, lateral: f32) -> xr::Posef {
    let fwd = normalize(forward(h));
    let up = [0.0, 1.0, 0.0];
    let right = normalize(cross(fwd, up));
    let o = [h.position.x, h.position.y, h.position.z];
    let pos = [
        o[0] + fwd[0] * dist + right[0] * lateral,
        o[1] + fwd[1] * dist + right[1] * lateral,
        o[2] + fwd[2] * dist + right[2] * lateral,
    ];
    let z = normalize([o[0] - pos[0], o[1] - pos[1], o[2] - pos[2]]); // face the head
    let x = normalize(cross(up, z));
    let y = cross(z, x);
    xr::Posef { orientation: quatf(quat_from_axes(x, y, z)), position: vec3f(pos) }
}
