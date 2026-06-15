// The reusable in-headset overlay foundation, lifted from monado-frame and
// trimmed: an OpenXR overlay session (XR_EXTX_overlay) rendering egui into
// Vulkan quad layers, with a controller laser pointer and grab/click input.
//
//  - `session`  — XR instance + overlay session + Vulkan bootstrap (this file)
//  - `panel`    — a single egui surface: swapchain, render, quad layer
//  - `input`    — action set, raycast, grab, double-press toggle
//  - `laser`    — the 3D pointer beam
pub mod input;
pub mod laser;
pub mod panel;
pub mod session;

#[allow(unused_imports)]
pub use input::{Input, Interaction, TargetId};
pub use laser::Laser;
pub use panel::{front_pose, posef, Panel};
#[allow(unused_imports)]
pub use session::{Gpu, Xr};
