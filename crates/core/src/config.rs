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
    pub vrchat: VrchatConfig,
    pub osc: OscConfig,

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

// ---- VRChat ---------------------------------------------------------------

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct VrchatConfig {
    /// Override the VRChat log directory (empty = auto-detect the Proton prefix).
    pub log_dir: String,
    pub auto_accept: AutoAcceptConfig,
    pub join_notifications: JoinNotifyConfig,
    pub status_automations: StatusConfig,
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
        Self { schedule_enabled: false, sleep_at: String::new(), wake_at: String::new() }
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
            vrchat: VrchatConfig::default(),
            osc: OscConfig::default(),
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
