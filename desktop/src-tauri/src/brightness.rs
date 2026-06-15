// Headset brightness (and fan) control for sleep/wake.
//
//  - Bigscreen Beyond: raw HID feature reports over /dev/hidraw (ported from
//    bsb-control). Works without VR; supports fan speed.
//  - Fallback: libmonado's generic display brightness (any Monado headset; needs
//    monado-service running; no fan control).
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

/// Apply brightness (0–100 %) and, on the Beyond, fan speed (0–100 %).
pub fn apply(b: Backend, brightness_pct: u8, fan_pct: u8) {
    match b {
        Backend::Beyond => {
            if !beyond::set(brightness_pct, fan_pct) {
                log::warn!("Beyond HID write failed (brightness {brightness_pct}%, fan {fan_pct}%)");
            }
        }
        Backend::Monado => {
            if !monado_set(brightness_pct) {
                log::warn!("libmonado brightness set failed ({brightness_pct}%)");
            }
        }
        Backend::None => log::warn!("no brightness backend; skipping"),
    }
}

fn monado_available() -> bool {
    let Ok(m) = Monado::auto_connect() else { return false };
    let Ok(dev) = m.device_from_role(DeviceRole::Head) else { return false };
    dev.brightness().is_ok()
}

fn monado_set(brightness_pct: u8) -> bool {
    let Ok(m) = Monado::auto_connect() else { return false };
    let Ok(dev) = m.device_from_role(DeviceRole::Head) else { return false };
    dev.set_brightness((brightness_pct.min(100) as f32 / 100.0).clamp(0.0, 1.0), false).is_ok()
}

/// Bigscreen Beyond HID control (VID 0x35BD / PID 0x0101).
mod beyond {
    use std::fs::OpenOptions;
    use std::os::unix::io::AsRawFd;
    use std::path::PathBuf;

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
            // HID_ID=0003:000035BD:00000101
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
        // buf[0] = report id 0x00; payload follows.
        for (i, b) in payload.iter().enumerate() {
            if i + 1 < REPORT_LEN {
                buf[i + 1] = *b;
            }
        }
        let r = unsafe { libc::ioctl(fd, hidiocsfeature(REPORT_LEN), buf.as_ptr()) };
        r >= 0
    }

    pub fn set(brightness_pct: u8, fan_pct: u8) -> bool {
        let Some(path) = find_device() else { return false };
        let Ok(file) = OpenOptions::new().read(true).write(true).open(&path) else { return false };
        let fd = file.as_raw_fd();
        let val = (brightness_pct.min(100) as u32 * 1023 / 100) as u16;
        let ok_b = send_feature(fd, &[CMD_BRIGHTNESS, (val >> 8) as u8, (val & 0xFF) as u8]);
        let ok_f = send_feature(fd, &[CMD_FAN, fan_pct.min(100)]);
        ok_b && ok_f
    }
}
