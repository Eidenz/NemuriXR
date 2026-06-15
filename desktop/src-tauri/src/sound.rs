// Notification sound playback. Plays a file in the background via the first
// available player (pw-play → paplay → ffplay). Bundled default tones are
// embedded and extracted to the cache dir on first use.
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const JOIN_OGG: &[u8] = include_bytes!("../sounds/join.ogg");
const LEAVE_OGG: &[u8] = include_bytes!("../sounds/leave.ogg");

fn cache_dir() -> PathBuf {
    let base = std::env::var("XDG_CACHE_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("{}/.cache", std::env::var("HOME").unwrap_or_default()));
    PathBuf::from(base).join("nemurixr")
}

/// Extract a bundled default sound to the cache dir and return its path.
fn default_path(kind: &str) -> PathBuf {
    let dir = cache_dir();
    let _ = std::fs::create_dir_all(&dir);
    let (name, bytes): (&str, &[u8]) = match kind {
        "leave" => ("leave.ogg", LEAVE_OGG),
        _ => ("join.ogg", JOIN_OGG),
    };
    let path = dir.join(name);
    if !path.exists() {
        let _ = std::fs::write(&path, bytes);
    }
    path
}

/// Play a sound file in the background (non-blocking).
pub fn play_file(path: &Path) {
    let p = path.to_string_lossy().to_string();
    let players: [(&str, &[&str]); 3] = [
        ("pw-play", &[]),
        ("paplay", &[]),
        ("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet"]),
    ];
    for (bin, args) in players {
        let ok = Command::new(bin)
            .args(args)
            .arg(&p)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok();
        if ok {
            return;
        }
    }
    log::warn!("no audio player (pw-play/paplay/ffplay) found to play {p}");
}

/// Play a notification: a custom file if set + present, else the bundled default.
/// `kind` is "join" or "leave".
pub fn play_notification(kind: &str, custom: &str) {
    let path = if !custom.is_empty() && Path::new(custom).exists() {
        PathBuf::from(custom)
    } else {
        default_path(kind)
    };
    play_file(&path);
}
