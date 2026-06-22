// Notification sound playback. Plays a file in the background via the first
// available player (pw-play → paplay → ffplay). Bundled default tones are
// embedded and extracted to the cache dir on first use.
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const JOIN_OGG: &[u8] = include_bytes!("../sounds/join.ogg");
const LEAVE_OGG: &[u8] = include_bytes!("../sounds/leave.ogg");
const ALARM_OGG: &[u8] = include_bytes!("../sounds/alarm.ogg");

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
        "alarm" => ("alarm.ogg", ALARM_OGG),
        _ => ("join.ogg", JOIN_OGG),
    };
    let path = dir.join(name);
    if !path.exists() {
        let _ = std::fs::write(&path, bytes);
    }
    path
}

/// Spawn a background player for `path`, returning its child handle. Tries the
/// available players in order (pw-play → paplay → ffplay).
fn spawn_player(path: &Path) -> Option<Child> {
    let p = path.to_string_lossy().to_string();
    let players: [(&str, &[&str]); 3] = [
        ("pw-play", &[]),
        ("paplay", &[]),
        ("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet"]),
    ];
    for (bin, args) in players {
        if let Ok(child) = Command::new(bin)
            .args(args)
            .arg(&p)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            return Some(child);
        }
    }
    log::warn!("no audio player (pw-play/paplay/ffplay) found to play {p}");
    None
}

/// Play a sound file once in the background (non-blocking).
pub fn play_file(path: &Path) {
    let _ = spawn_player(path);
}

/// A looping wake-up alarm. A background thread replays the sound on repeat
/// (like a real alarm clock) until `stop()` is called or the handle is dropped.
pub struct Alarm {
    stop: Arc<AtomicBool>,
}

impl Alarm {
    /// Start ringing. `custom` is an optional sound-file path ("" = default chime).
    pub fn start(custom: &str) -> Alarm {
        let path = if !custom.is_empty() && Path::new(custom).exists() {
            PathBuf::from(custom)
        } else {
            default_path("alarm")
        };
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        std::thread::spawn(move || ring_loop(&path, &flag));
        Alarm { stop }
    }

    /// Stop ringing (idempotent).
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

impl Drop for Alarm {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Replay `path` until `stop` is set, cutting the current playthrough short the
/// moment we're asked to stop. A short gap between rings keeps it an alarm, not
/// a continuous drone.
fn ring_loop(path: &Path, stop: &AtomicBool) {
    while !stop.load(Ordering::Relaxed) {
        let mut child = match spawn_player(path) {
            Some(c) => c,
            None => return, // no audio player available — give up quietly
        };
        loop {
            if stop.load(Ordering::Relaxed) {
                let _ = child.kill();
                let _ = child.wait();
                return;
            }
            match child.try_wait() {
                Ok(Some(_)) => break, // finished — loop round to replay
                Ok(None) => std::thread::sleep(Duration::from_millis(100)),
                Err(_) => return,
            }
        }
        // ~0.6s breather, but stay responsive to stop.
        for _ in 0..6 {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
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
