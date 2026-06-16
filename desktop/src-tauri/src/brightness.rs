// Headset brightness (and fan) control with timed fades.
//
//  - Bigscreen Beyond: raw HID feature reports over /dev/hidraw (ported from
//    bsb-control). Works without VR; supports fan speed.
//  - Fallback: libmonado's generic display brightness (any Monado headset; needs
//    monado-service; no fan control).
//
// A `Session` opens the backend once so a fade can write many steps cheaply, and
// `transition()` (run on a thread) lerps from the current level to the target
// over `duration_secs`, cancelable when a newer transition bumps `fade_gen`.
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use libmonado::{DeviceLogic, DeviceRole, Monado};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Beyond,
    Monado,
    None,
}

/// Pick the best available backend right now (Beyond preferred).
pub fn detect() -> Backend {
    if beyond::present() {
        Backend::Beyond
    } else if monado_available() {
        Backend::Monado
    } else {
        Backend::None
    }
}

pub fn name(b: Backend) -> Option<String> {
    match b {
        Backend::Beyond => Some("Bigscreen Beyond".to_string()),
        Backend::Monado => Some("libmonado".to_string()),
        Backend::None => None,
    }
}

/// Whether a Bigscreen Beyond is connected and whether we can talk to its HID:
/// "absent" (none found), "needs_rule" (found but not writable — udev rule
/// missing), or "ready" (found and writable).
pub fn beyond_status() -> &'static str {
    use std::fs::OpenOptions;
    match beyond::find_device() {
        None => "absent",
        Some(path) => {
            if OpenOptions::new().read(true).write(true).open(&path).is_ok() {
                "ready"
            } else {
                "needs_rule"
            }
        }
    }
}

/// An open handle to the backend, reused across the steps of a fade.
pub enum Session {
    Beyond(PathBuf),
    Monado(Monado),
    None,
}

impl Session {
    pub fn open(backend: Backend) -> Self {
        match backend {
            Backend::Beyond => beyond::find_device().map(Session::Beyond).unwrap_or(Session::None),
            Backend::Monado => Monado::auto_connect().ok().map(Session::Monado).unwrap_or(Session::None),
            Backend::None => Session::None,
        }
    }

    pub fn set(&self, brightness_pct: u8, fan_pct: u8) {
        match self {
            Session::Beyond(path) => {
                if !beyond::write(path, brightness_pct, fan_pct) {
                    log::warn!("Beyond HID write failed");
                }
            }
            Session::Monado(m) => {
                if let Ok(dev) = m.device_from_role(DeviceRole::Head) {
                    let _ = dev.set_brightness((brightness_pct.min(100) as f32 / 100.0).clamp(0.0, 1.0), false);
                }
            }
            Session::None => {}
        }
    }
}

/// Set a level immediately (no fade) — used for previews and instant snaps.
pub fn set_now(backend: Backend, brightness_pct: u8, fan_pct: u8) {
    Session::open(backend).set(brightness_pct, fan_pct);
}

/// Fade from the current level (in `current`) to `to` over `duration_secs`,
/// updating `current` as it goes. Aborts if `fade_gen` no longer equals `gen`
/// (a newer transition superseded this one). Intended to be run on a thread.
#[allow(clippy::too_many_arguments)]
pub fn transition(
    backend: Backend,
    current: Arc<Mutex<Option<(u8, u8)>>>,
    fade_gen: Arc<AtomicU64>,
    gen: u64,
    to: (u8, u8),
    duration_secs: u32,
) {
    let session = Session::open(backend);
    let from = current.lock().unwrap().unwrap_or(to);
    if duration_secs == 0 || from == to {
        session.set(to.0, to.1);
        *current.lock().unwrap() = Some(to);
        return;
    }
    let start = Instant::now();
    let total = duration_secs as f32;
    loop {
        if fade_gen.load(Ordering::SeqCst) != gen {
            return; // superseded by a newer transition
        }
        let t = (start.elapsed().as_secs_f32() / total).min(1.0);
        let b = lerp(from.0, to.0, t);
        let f = lerp(from.1, to.1, t);
        session.set(b, f);
        *current.lock().unwrap() = Some((b, f));
        if t >= 1.0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round().clamp(0.0, 255.0) as u8
}

fn monado_available() -> bool {
    let Ok(m) = Monado::auto_connect() else { return false };
    let Ok(dev) = m.device_from_role(DeviceRole::Head) else { return false };
    dev.brightness().is_ok()
}

/// Bigscreen Beyond HID control (VID 0x35BD / PID 0x0101).
mod beyond {
    use std::fs::OpenOptions;
    use std::os::unix::io::AsRawFd;
    use std::path::{Path, PathBuf};

    const VID: &str = "000035BD"; // as it appears in HID_ID (8 hex)
    const PID: &str = "00000101";
    const CMD_BRIGHTNESS: u8 = 0x49; // [0x49, hi, lo] — u16 BE, 0..=1023
    const CMD_FAN: u8 = 0x46; // [0x46, speed] — u8, 0..=100
    const REPORT_LEN: usize = 65;

    // HIDIOCSFEATURE(len) = _IOC(_IOC_WRITE|_IOC_READ, 'H'=0x48, 0x06, len)
    fn hidiocsfeature(len: usize) -> libc::c_ulong {
        ((3u64 << 30) | ((len as u64) << 16) | (0x48u64 << 8) | 0x06) as libc::c_ulong
    }

    pub fn find_device() -> Option<PathBuf> {
        for entry in std::fs::read_dir("/sys/class/hidraw").ok()?.flatten() {
            let uevent = entry.path().join("device/uevent");
            let Ok(txt) = std::fs::read_to_string(&uevent) else { continue };
            let matches = txt
                .lines()
                .any(|l| l.starts_with("HID_ID=") && l.contains(VID) && l.contains(PID));
            if matches {
                return Some(PathBuf::from("/dev").join(entry.file_name()));
            }
        }
        None
    }

    pub fn present() -> bool {
        find_device().is_some()
    }

    fn send_feature(fd: i32, payload: &[u8]) -> bool {
        let mut buf = [0u8; REPORT_LEN];
        for (i, b) in payload.iter().enumerate() {
            if i + 1 < REPORT_LEN {
                buf[i + 1] = *b;
            }
        }
        let r = unsafe { libc::ioctl(fd, hidiocsfeature(REPORT_LEN), buf.as_ptr()) };
        r >= 0
    }

    pub fn write(path: &Path, brightness_pct: u8, fan_pct: u8) -> bool {
        let Ok(file) = OpenOptions::new().read(true).write(true).open(path) else { return false };
        let fd = file.as_raw_fd();
        let val = (brightness_pct.min(100) as u32 * 1023 / 100) as u16;
        let ok_b = send_feature(fd, &[CMD_BRIGHTNESS, (val >> 8) as u8, (val & 0xFF) as u8]);
        let ok_f = send_feature(fd, &[CMD_FAN, fan_pct.min(100)]);
        ok_b && ok_f
    }
}
