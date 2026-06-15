// A snapshot of live runtime state the overlay/core publishes to the desktop
// app over IPC (read-only from the desktop's perspective). Distinct from
// `Config`, which is the persisted settings both sides edit.
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct State {
    /// Sleep mode currently active.
    pub sleep_active: bool,
    /// Players in your current VRChat world (includes you), 0 if not in a world.
    pub player_count: u32,
    /// Current VRChat world/instance name, if in one.
    pub vrchat_world: Option<String>,
    /// Current VRChat status string, if known.
    pub vrchat_status: Option<String>,
    /// Which brightness backend is in use ("Bigscreen Beyond", "libmonado", or none).
    pub brightness_backend: Option<String>,
    /// True while the in-headset overlay/core is running and serving IPC.
    pub overlay_running: bool,
}
