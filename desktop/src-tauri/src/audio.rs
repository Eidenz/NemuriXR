// Audio volume automations (PipeWire/PulseAudio via `pactl`).
//
// We control the DEVICE VRChat is using, not VRChat's own stream: find VRChat's
// playback stream, see which output device (sink) it's routed to, and set that
// device's volume. Same idea for the mic (the input device VRChat captures from).
// When VRChat isn't detected we fall back to the system default device.
use std::process::Command;

use nemurixr_core::config::AudioLevel;
use serde_json::Value;

/// PulseAudio uses u32::MAX to mean "no device" (idle/corked streams).
const NO_DEVICE: i64 = 4294967295;

/// Run `pactl -f json <args>` and parse the output.
fn pactl_json(args: &[&str]) -> Option<Value> {
    let out = Command::new("pactl").arg("-f").arg("json").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    serde_json::from_slice(&out.stdout).ok()
}

fn run_pactl(args: &[&str]) -> bool {
    Command::new("pactl").args(args).status().map(|s| s.success()).unwrap_or(false)
}

fn pactl_available() -> bool {
    Command::new("pactl").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

/// True if a stream's properties look like VRChat (matches under Proton/Wine too).
fn is_vrchat(props: &Value) -> bool {
    const KEYS: [&str; 4] = ["application.name", "application.process.binary", "media.name", "node.name"];
    KEYS.iter().any(|k| props.get(*k).and_then(Value::as_str).is_some_and(|v| v.to_lowercase().contains("vrchat")))
}

fn valid(idx: i64) -> bool {
    idx >= 0 && idx != NO_DEVICE
}

/// The output device (sink) VRChat is currently playing to, if detectable.
fn vrchat_sink() -> Option<i64> {
    let arr = pactl_json(&["list", "sink-inputs"])?;
    arr.as_array()?
        .iter()
        .filter(|si| si.get("properties").is_some_and(is_vrchat))
        .find_map(|si| si.get("sink").and_then(Value::as_i64).filter(|i| valid(*i)))
}

/// The input device (source) VRChat is currently capturing from, if detectable.
fn vrchat_source() -> Option<i64> {
    let arr = pactl_json(&["list", "source-outputs"])?;
    arr.as_array()?
        .iter()
        .filter(|so| so.get("properties").is_some_and(is_vrchat))
        .find_map(|so| so.get("source").and_then(Value::as_i64).filter(|i| valid(*i)))
}

fn sink_description(index: i64) -> Option<String> {
    let arr = pactl_json(&["list", "sinks"])?;
    arr.as_array()?
        .iter()
        .find(|s| s.get("index").and_then(Value::as_i64) == Some(index))
        .and_then(|s| s.get("description").and_then(Value::as_str))
        .map(str::to_string)
}

/// Mute/unmute the microphone device VRChat uses (or the default source). Used
/// by the auto-sleep safety net.
pub fn set_mic_muted(muted: bool) {
    if !pactl_available() {
        return;
    }
    let m = if muted { "1" } else { "0" };
    match vrchat_source() {
        Some(idx) => run_pactl(&["set-source-mute", &idx.to_string(), m]),
        None => run_pactl(&["set-source-mute", "@DEFAULT_SOURCE@", m]),
    };
    log::info!("safety net: mic device {}", if muted { "muted" } else { "unmuted" });
}

/// Apply an audio level. Returns a human description of the output device that
/// was controlled (for the UI), or None if nothing was set / pactl is absent.
pub fn apply(level: &AudioLevel) -> Option<String> {
    if !pactl_available() {
        log::warn!("audio: `pactl` not found; skipping audio automation");
        return None;
    }

    let mut target = None;
    if level.set_volume {
        let vol = level.volume.min(100);
        match vrchat_sink() {
            Some(idx) => {
                run_pactl(&["set-sink-volume", &idx.to_string(), &format!("{vol}%")]);
                let name = sink_description(idx).unwrap_or_else(|| "output device".into());
                target = Some(format!("{name} (VRChat)"));
            }
            None => {
                run_pactl(&["set-sink-volume", "@DEFAULT_SINK@", &format!("{vol}%")]);
                target = Some("Default output".to_string());
            }
        }
        log::info!("audio: output volume -> {vol}% on {}", target.as_deref().unwrap_or("?"));
    }

    if level.set_mic {
        let m = if level.mic_muted { "1" } else { "0" };
        match vrchat_source() {
            Some(idx) => {
                run_pactl(&["set-source-mute", &idx.to_string(), m]);
            }
            None => {
                run_pactl(&["set-source-mute", "@DEFAULT_SOURCE@", m]);
            }
        }
        log::info!("audio: mic {}", if level.mic_muted { "muted" } else { "unmuted" });
    }

    target
}
