// Motion-based sleep detection. Samples the head pose; when angular movement
// stays below a sensitivity threshold for `still_minutes`, it starts a cancelable
// countdown and then signals Sleep. Enter-only — it never wakes you.
//
// v1 uses fixed sensitivity presets (deg/s). A calibration step could later feed
// a custom threshold in place of the preset.
use std::time::{Duration, Instant};

use nemurixr_core::config::Sensitivity;
use openxr as xr;

use crate::mathx::qf;

const COUNTDOWN: Duration = Duration::from_secs(30);

pub enum Tick {
    Idle,
    /// Counting down to sleep; seconds remaining (cancelable).
    Counting(u32),
    /// Stillness confirmed — go to sleep now.
    Sleep,
}

pub struct Detector {
    prev: Option<[f32; 4]>,
    ema_deg_s: f32, // smoothed angular speed
    last_frame: Option<Instant>,
    still_since: Option<Instant>,
    countdown_until: Option<Instant>,
}

impl Detector {
    pub fn new() -> Self {
        Self { prev: None, ema_deg_s: 0.0, last_frame: None, still_since: None, countdown_until: None }
    }

    fn reset(&mut self) {
        self.still_since = None;
        self.countdown_until = None;
    }

    /// Feed one frame. `active` gates detection (enabled + Awake + in window).
    /// `controller_active` cancels/holds off (any deliberate input = awake).
    pub fn update(&mut self, active: bool, hmd: &xr::Posef, controller_active: bool, sensitivity: Sensitivity, still_minutes: u32) -> Tick {
        let now = Instant::now();
        let dt = self.last_frame.map(|t| now.duration_since(t).as_secs_f32()).unwrap_or(0.0).clamp(1e-3, 0.1);
        self.last_frame = Some(now);

        let cur = qf(&hmd.orientation);
        let angle = self.prev.map(|p| angle_between_deg(p, cur)).unwrap_or(0.0);
        self.prev = Some(cur);
        self.ema_deg_s = self.ema_deg_s * 0.85 + (angle / dt) * 0.15;

        if !active {
            self.reset();
            return Tick::Idle;
        }

        let threshold = match sensitivity {
            Sensitivity::Low => 1.0,    // strict — needs near-stillness
            Sensitivity::Medium => 2.5,
            Sensitivity::High => 5.0,   // forgiving — sleeps with more movement
        };
        if controller_active || self.ema_deg_s > threshold {
            self.reset();
            return Tick::Idle;
        }

        let started = *self.still_since.get_or_insert(now);
        if self.countdown_until.is_none() {
            if now.duration_since(started) < Duration::from_secs(still_minutes as u64 * 60) {
                return Tick::Idle;
            }
            self.countdown_until = Some(now + COUNTDOWN);
        }

        let until = self.countdown_until.unwrap();
        if now >= until {
            self.reset();
            Tick::Sleep
        } else {
            Tick::Counting((until - now).as_secs() as u32 + 1)
        }
    }
}

/// Angle (degrees) between two unit quaternions.
fn angle_between_deg(a: [f32; 4], b: [f32; 4]) -> f32 {
    let dot = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]).abs().clamp(0.0, 1.0);
    2.0 * dot.acos() * 180.0 / std::f32::consts::PI
}
