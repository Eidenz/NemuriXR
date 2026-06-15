// Auto-launch the in-headset overlay. OpenXR/Monado has no SteamVR-style overlay
// manifest, so the always-on desktop app is the launcher: it watches for a
// running Monado session and spawns the overlay (which self-exits when VR ends).
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::Engine;

const POLL: Duration = Duration::from_secs(10);

pub type OverlayChild = Arc<Mutex<Option<Child>>>;

/// Monado's compositor IPC socket exists iff a Monado VR session is running.
fn monado_present() -> bool {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        if Path::new(&format!("{dir}/monado_comp_ipc")).exists() {
            return true;
        }
    }
    Path::new("/tmp/monado_comp_ipc").exists()
}

/// Locate the overlay binary: env override → sidecar next to us → dev target → PATH.
fn overlay_bin() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("NEMURI_OVERLAY_BIN") {
        let p = PathBuf::from(p);
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sidecar = dir.join("nemurixr-overlay");
            if sidecar.exists() {
                return Some(sidecar);
            }
            // dev: desktop/src-tauri/target/<profile>/ → workspace nemurixr/target/<profile>/
            for rel in ["../../../../target/debug/nemurixr-overlay", "../../../../target/release/nemurixr-overlay"] {
                let c = dir.join(rel);
                if c.exists() {
                    return Some(c);
                }
            }
        }
    }
    if let Ok(path) = std::env::var("PATH") {
        for d in path.split(':') {
            let c = Path::new(d).join("nemurixr-overlay");
            if c.exists() {
                return Some(c);
            }
        }
    }
    None
}

fn spawn_overlay() -> Option<Child> {
    let bin = overlay_bin()?;
    match Command::new(&bin).spawn() {
        Ok(child) => {
            log::info!("launched overlay: {}", bin.display());
            Some(child)
        }
        Err(e) => {
            log::warn!("failed to launch overlay {}: {e}", bin.display());
            None
        }
    }
}

/// Reap an exited child; true if an overlay is currently running.
fn running(slot: &mut Option<Child>) -> bool {
    match slot {
        Some(child) => match child.try_wait() {
            Ok(None) => true,
            _ => {
                *slot = None;
                false
            }
        },
        None => false,
    }
}

/// Watcher: when Monado is up (and auto-launch enabled), ensure the overlay runs.
pub fn spawn_watcher(engine: Arc<Mutex<Engine>>, child: OverlayChild) {
    std::thread::spawn(move || loop {
        let enabled = engine.lock().unwrap().auto_launch_overlay();
        {
            let mut slot = child.lock().unwrap();
            if enabled && monado_present() && !running(&mut slot) {
                *slot = spawn_overlay();
            }
        }
        std::thread::sleep(POLL);
    });
}

/// Launch the overlay now regardless of Monado (it self-exits if there's no VR
/// runtime). Returns true if it's running afterward.
pub fn launch_now(child: &OverlayChild) -> bool {
    let mut slot = child.lock().unwrap();
    if running(&mut slot) {
        return true;
    }
    *slot = spawn_overlay();
    slot.is_some()
}

/// Stop the overlay (on desktop quit).
pub fn kill(child: &OverlayChild) {
    if let Some(mut c) = child.lock().unwrap().take() {
        let _ = c.kill();
    }
}
