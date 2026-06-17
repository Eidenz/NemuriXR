// The shared, persisted configuration model — the single source of truth both
// the overlay (toggles) and the desktop app (full value editing) operate on.
//
// Stored at <XDG config>/nemurixr/config.json. `#[serde(default)]` everywhere
// so older files keep loading as fields are added. Feature `enabled` booleans
// are what the in-headset overlay flips; the desktop edits the detailed values.
use std::{env, fs, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Suppress game controller input while pointing at overlay panels.
    pub block_game_input: bool,
    /// Auto-launch the in-headset overlay when a Monado VR session starts.
    pub auto_launch_overlay: bool,
    pub sleep: SleepConfig,
    pub brightness: BrightnessConfig,
    pub audio: AudioConfig,
    pub vrchat: VrchatConfig,
    pub osc: OscConfig,
    pub commands: CommandsConfig,
    pub safety_net: SafetyNetConfig,

    // Never sent over the wire; defaults to the standard path when a config is
    // deserialized (e.g. received over IPC) so `save()` always has a target.
    #[serde(skip_serializing, default = "config_path")]
    path: String,
}

// ---- Sleep mode -----------------------------------------------------------

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SleepConfig {
    /// Enable the time-based schedule.
    pub schedule_enabled: bool,
    /// "HH:MM" local time to enter sleep mode (empty = unset).
    pub sleep_at: String,
    /// "HH:MM" local time to wake (empty = unset).
    pub wake_at: String,

    // ---- motion-based sleep detection ----
    /// Enter sleep automatically after the headset stays still.
    pub detection_enabled: bool,
    /// Watch always (true), or only inside the detect_start..detect_end window.
    pub detection_always: bool,
    /// "HH:MM" detection-window start (used when not `detection_always`).
    pub detect_start: String,
    /// "HH:MM" detection-window end.
    pub detect_end: String,
    /// How forgiving stillness detection is.
    pub detection_sensitivity: Sensitivity,
    /// Minutes of stillness before the (cancelable) sleep countdown.
    pub detection_minutes: u32,

    // ---- optional sleep-pose calibration ----
    /// Calibrated sleep poses. Empty = uncalibrated => stillness alone triggers;
    /// when non-empty the head must also be near one of these poses.
    #[serde(default, deserialize_with = "de_sleep_poses")]
    pub detection_poses: Vec<SleepPose>,
    /// Max angle (degrees) from a calibrated pose that still counts as "in pose".
    pub detection_pose_tolerance: u32,

    /// Gentle wake-up routine at the scheduled wake time.
    pub wake: WakeConfig,
}

/// A gradual wake-up: ramp brightness back up (a "sunrise") and optionally play
/// an alarm sound at the end. Triggered by the schedule's wake time.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WakeConfig {
    /// Auto-wake at the scheduled wake time. Brightness eases in using the
    /// brightness config's wake fade time (`on_wake.transition_seconds`), same as
    /// a manual wake.
    pub enabled: bool,
    /// Play an alarm sound once you've woken.
    pub alarm_enabled: bool,
    /// Alarm sound file ("" = bundled default chime).
    pub alarm_sound: String,
}

/// One calibrated sleep pose: a user label plus the head-local gravity vector
/// (yaw-invariant) used to recognise the posture.
#[derive(Clone, Serialize, Deserialize)]
pub struct SleepPose {
    pub name: String,
    pub gravity: [f32; 3],
}

/// Accept both the new `{name, gravity}` shape and the old bare `[x,y,z]` shape
/// (pre-naming configs), so existing calibrations keep loading.
fn de_sleep_poses<'de, D>(d: D) -> Result<Vec<SleepPose>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Compat {
        New { name: String, gravity: [f32; 3] },
        Old([f32; 3]),
    }
    let raw = Vec::<Compat>::deserialize(d)?;
    Ok(raw
        .into_iter()
        .enumerate()
        .map(|(i, c)| match c {
            Compat::New { name, gravity } => SleepPose { name, gravity },
            Compat::Old(gravity) => SleepPose { name: format!("Pose {}", i + 1), gravity },
        })
        .collect())
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Sensitivity {
    Low,
    #[default]
    Medium,
    High,
}

// ---- Brightness & fans ----------------------------------------------------

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BrightnessConfig {
    /// Apply brightness/fan changes on phase transitions.
    pub enabled: bool,
    pub on_wake: BrightnessLevel,
    pub on_prepare: BrightnessLevel,
    pub on_sleep: BrightnessLevel,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct BrightnessLevel {
    /// 0–100 % panel brightness.
    pub brightness: u8,
    /// 0–100 % fan speed (Bigscreen Beyond only; ignored otherwise).
    pub fan: u8,
    /// Seconds to fade into this level (0 = instant).
    pub transition_seconds: u32,
}

// ---- Commands (run a script/app per phase) --------------------------------

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CommandsConfig {
    /// Run the per-phase commands on phase transitions.
    pub enabled: bool,
    /// Shell command lines (run via `sh -c`); empty = nothing for that phase.
    pub on_wake: String,
    pub on_prepare: String,
    pub on_sleep: String,
}

// ---- Auto-sleep safety net ------------------------------------------------

/// Actions that run ONLY when sleep was triggered automatically (by motion
/// detection) — for when you doze off without setting yourself up. A manual
/// sleep toggle never triggers these.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SafetyNetConfig {
    pub enabled: bool,
    /// Apply your VRChat sleeping-pose set (from `vrchat.sleeping_pose`).
    pub pose: bool,
    /// Skip the pose if FBT trackers are connected (you're posed already).
    pub pose_skip_if_trackers: bool,
    /// Re-pose even if you're already in a GoGo pose (default off: leave it).
    pub pose_override_existing: bool,
    /// Mute your VRChat mic in-game (OSC).
    pub mute_ingame: bool,
    /// Mute the microphone device (PipeWire/PulseAudio).
    pub mute_device: bool,
}

impl Default for SafetyNetConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            pose: true,
            pose_skip_if_trackers: true,
            pose_override_existing: false,
            mute_ingame: false,
            mute_device: true,
        }
    }
}

// ---- Audio volume ---------------------------------------------------------

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioConfig {
    /// Apply audio changes on phase transitions.
    pub enabled: bool,
    pub on_wake: AudioLevel,
    pub on_prepare: AudioLevel,
    pub on_sleep: AudioLevel,
}

/// Per-phase audio settings. The output device is whichever one VRChat is
/// currently playing to (auto-detected); the default device is used as a
/// fallback. Same for the microphone (the device VRChat captures from).
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioLevel {
    /// Set the output device's volume on this phase.
    pub set_volume: bool,
    /// 0–100 % output volume.
    pub volume: u8,
    /// Change the microphone mute state on this phase.
    pub set_mic: bool,
    /// Mute (true) / unmute (false) the mic on this phase.
    pub mic_muted: bool,
}

// ---- VRChat ---------------------------------------------------------------

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct VrchatConfig {
    /// Override the VRChat log directory (empty = auto-detect the Proton prefix).
    pub log_dir: String,
    pub auto_accept: AutoAcceptConfig,
    pub join_notifications: JoinNotifyConfig,
    pub status_automations: StatusConfig,
    pub sleeping_pose: SleepingPoseConfig,
}

/// While asleep, send OSC to lie your avatar in the direction you're physically
/// lying (back/front/left/right), via a preset (GoGo Loco, …) or custom OSC.
/// Reuses the OSC target (host/port/OSCQuery) from `OscConfig`.
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SleepingPoseConfig {
    pub enabled: bool,
    /// UI hint for which preset is loaded ("gogo_loco" | "gorone" | "custom").
    pub preset: String,
    /// Lock the avatar's feet so it doesn't slide when the pose changes.
    pub lock_feet: bool,
    pub on_back: Vec<OscMessage>,
    pub on_front: Vec<OscMessage>,
    pub on_left: Vec<OscMessage>,
    pub on_right: Vec<OscMessage>,
    pub foot_lock: Vec<OscMessage>,
    pub foot_unlock: Vec<OscMessage>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ListMode {
    #[default]
    Whitelist,
    Blacklist,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoAcceptConfig {
    pub enabled: bool,
    pub list_mode: ListMode,
    /// VRChat user ids on the white/blacklist.
    pub player_ids: Vec<String>,
    pub only_when_sleep: bool,
    pub max_players_enabled: bool,
    /// Only auto-accept when fewer than this many players are in the world
    /// (includes yourself).
    pub max_players: u32,
    /// Send one of your VRChat invite-message templates when accepting.
    pub invite_message_enabled: bool,
    /// Which invite-message slot (0–11) to send.
    pub invite_message_slot: u32,
    /// Send a decline message to requests that are NOT auto-accepted.
    pub decline_message_enabled: bool,
    /// Which request-response slot (0–11) to send when declining.
    pub decline_message_slot: u32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct JoinNotifyConfig {
    pub enabled: bool,
    /// Sound file to play on join (empty = none / default).
    pub join_sound: String,
    pub leave_sound: String,
    /// Only notify when you were alone before the join / are alone after the leave.
    pub only_when_alone: bool,
    pub only_when_sleep: bool,
    /// Only notify for players on your VRChat friends list (needs sign-in; when
    /// signed out it can't filter, so it notifies for everyone).
    pub friends_only: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VrcStatus {
    JoinMe,
    #[default]
    Active,
    AskMe,
    Busy,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StatusConfig {
    pub enabled: bool,
    /// Player count at/above which the "busy" status is used.
    pub player_limit: u32,
    pub below_status: VrcStatus,
    pub at_or_above_status: VrcStatus,
    pub only_when_sleep: bool,
}

// ---- OSC ------------------------------------------------------------------

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OscConfig {
    pub host: String,
    pub port: u16,
    /// Auto-discover VRChat's OSC port via OSCQuery instead of `port`.
    pub use_oscquery: bool,
    pub on_wake: Vec<OscMessage>,
    pub on_prepare: Vec<OscMessage>,
    pub on_sleep: Vec<OscMessage>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct OscMessage {
    pub address: String,
    pub args: Vec<OscArg>,
    /// Wait this many milliseconds before sending this message (sequences a list).
    pub delay_ms: u32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum OscArg {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
}

// ---- Defaults -------------------------------------------------------------

impl Default for SleepConfig {
    fn default() -> Self {
        Self {
            schedule_enabled: false,
            sleep_at: String::new(),
            wake_at: String::new(),
            detection_enabled: false,
            detection_always: true,
            detect_start: String::new(),
            detect_end: String::new(),
            detection_sensitivity: Sensitivity::Medium,
            detection_minutes: 15,
            detection_poses: Vec::new(),
            detection_pose_tolerance: 30,
            wake: WakeConfig::default(),
        }
    }
}

impl Default for WakeConfig {
    fn default() -> Self {
        Self { enabled: false, alarm_enabled: false, alarm_sound: String::new() }
    }
}

impl Default for BrightnessLevel {
    fn default() -> Self {
        Self { brightness: 100, fan: 100, transition_seconds: 0 }
    }
}

impl Default for BrightnessConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            on_wake: BrightnessLevel { brightness: 100, fan: 100, transition_seconds: 2 },
            on_prepare: BrightnessLevel { brightness: 40, fan: 50, transition_seconds: 10 },
            on_sleep: BrightnessLevel { brightness: 10, fan: 30, transition_seconds: 30 },
        }
    }
}

impl Default for AutoAcceptConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            list_mode: ListMode::Whitelist,
            player_ids: Vec::new(),
            only_when_sleep: false,
            max_players_enabled: false,
            max_players: 2,
            invite_message_enabled: false,
            invite_message_slot: 0,
            decline_message_enabled: false,
            decline_message_slot: 0,
        }
    }
}

impl Default for AudioLevel {
    fn default() -> Self {
        Self { set_volume: false, volume: 100, set_mic: false, mic_muted: false }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            on_wake: AudioLevel { set_volume: true, volume: 100, set_mic: true, mic_muted: false },
            on_prepare: AudioLevel { set_volume: true, volume: 60, set_mic: false, mic_muted: false },
            on_sleep: AudioLevel { set_volume: true, volume: 25, set_mic: true, mic_muted: true },
        }
    }
}

impl Default for JoinNotifyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            join_sound: String::new(),
            leave_sound: String::new(),
            only_when_alone: true,
            only_when_sleep: false,
            friends_only: true,
        }
    }
}

impl Default for StatusConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            player_limit: 5,
            below_status: VrcStatus::JoinMe,
            at_or_above_status: VrcStatus::AskMe,
            only_when_sleep: false,
        }
    }
}

impl Default for OscConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9000,
            use_oscquery: true,
            on_wake: Vec::new(),
            on_prepare: Vec::new(),
            on_sleep: Vec::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            block_game_input: true,
            auto_launch_overlay: true,
            sleep: SleepConfig::default(),
            brightness: BrightnessConfig::default(),
            audio: AudioConfig::default(),
            vrchat: VrchatConfig::default(),
            osc: OscConfig::default(),
            commands: CommandsConfig::default(),
            safety_net: SafetyNetConfig::default(),
            path: config_path(),
        }
    }
}

// ---- Load / save ----------------------------------------------------------

pub fn config_path() -> String {
    let base = env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("{}/.config", env::var("HOME").unwrap_or_default()));
    format!("{base}/nemurixr/config.json")
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        let mut cfg = match fs::read_to_string(&path) {
            Ok(txt) => serde_json::from_str::<Config>(&txt).unwrap_or_else(|e| {
                log::warn!("config parse error ({e}); using defaults");
                Config::default()
            }),
            Err(_) => Config::default(),
        };
        cfg.path = path;
        cfg
    }

    pub fn save(&self) {
        if let Some(dir) = Path::new(&self.path).parent() {
            let _ = fs::create_dir_all(dir);
        }
        match serde_json::to_string_pretty(self) {
            Ok(txt) => match fs::write(&self.path, txt) {
                Ok(()) => log::info!("wrote {}", self.path),
                Err(e) => log::warn!("failed to write {}: {e}", self.path),
            },
            Err(e) => log::warn!("serialise config: {e}"),
        }
    }
}
