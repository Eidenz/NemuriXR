// A snapshot of live runtime state the overlay/core publishes to the desktop
// app over IPC (read-only from the desktop's perspective). Distinct from
// `Config`, which is the persisted settings both sides edit.
use serde::{Deserialize, Serialize};

/// The sleep state machine: Awake → Prepare → Sleep (and back to Awake).
/// Each phase has its own brightness/fan + OSC automations.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SleepPhase {
    #[default]
    Awake,
    Prepare,
    Sleep,
}

impl SleepPhase {
    /// True when sleep mode is "on" (preparing or asleep) — used by the
    /// "only when sleep mode is enabled" automation conditions.
    pub fn is_active(self) -> bool {
        self != SleepPhase::Awake
    }
}

/// How a sleep transition was triggered — lets automations (e.g. the auto-sleep
/// safety net) act only when you fell asleep on your own, not on a manual toggle.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SleepTrigger {
    #[default]
    Manual,
    Schedule,
    Detection,
}

/// Which way you're physically lying, for the VRChat sleeping-pose automation.
/// `Upright` means "not in a lying pose" (no avatar pose sent).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SleepPosition {
    #[default]
    Upright,
    Back,
    Front,
    Left,
    Right,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct State {
    /// Current sleep phase.
    pub sleep_phase: SleepPhase,
    /// Players in your current VRChat world (includes you), 0 if not in a world.
    pub player_count: u32,
    /// Current VRChat world/instance name, if in one.
    pub vrchat_world: Option<String>,
    /// Current VRChat status string, if known.
    pub vrchat_status: Option<String>,
    /// Which brightness backend is in use ("Bigscreen Beyond", "libmonado", or none).
    pub brightness_backend: Option<String>,
    /// Resolved VRChat OSC target (host:port) when discovered, else None.
    pub osc_target: Option<String>,
    /// The audio output device last controlled by an automation (for the UI).
    pub audio_target: Option<String>,
    /// Latest transient notice text (auto-accept, status change, …) for the
    /// in-headset toast. Paired with `notice_seq` so the overlay only shows each
    /// notice once.
    pub notice: Option<String>,
    /// Bumped every time `notice` is set; the overlay toasts when it changes.
    pub notice_seq: u64,
    /// True while the wake-up alarm is ringing — the overlay and desktop show a
    /// big Stop button while this is set.
    pub alarm_active: bool,
    /// True while the in-headset overlay/core is running and serving IPC.
    pub overlay_running: bool,
}
