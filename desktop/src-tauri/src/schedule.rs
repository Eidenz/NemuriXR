// Time-based sleep schedule. A background thread enters sleep mode at `sleep_at`
// and exits at `wake_at` (local time). Edge-triggered at the crossing — it does
// NOT force the state for the whole window, so a manual toggle isn't overridden.
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{Local, Timelike};
use nemurixr_core::SleepPhase;

use crate::Engine;

const TICK: Duration = Duration::from_secs(20);

pub fn spawn(engine: Arc<Mutex<Engine>>) {
    std::thread::spawn(move || run(engine));
}

fn run(engine: Arc<Mutex<Engine>>) {
    let mut prev: Option<u32> = None;
    loop {
        let now = local_minutes();
        let (enabled, sleep_at, wake_at) = {
            let g = engine.lock().unwrap();
            let s = &g.config.sleep;
            (s.schedule_enabled, parse_hhmm(&s.sleep_at), parse_hhmm(&s.wake_at))
        };
        if enabled {
            if let Some(p) = prev {
                if let Some(t) = sleep_at {
                    if crossed(p, now, t) {
                        log::info!("schedule: sleep at {:02}:{:02}", t / 60, t % 60);
                        engine.lock().unwrap().set_phase(SleepPhase::Sleep);
                    }
                }
                if let Some(t) = wake_at {
                    if crossed(p, now, t) {
                        log::info!("schedule: wake at {:02}:{:02}", t / 60, t % 60);
                        engine.lock().unwrap().set_phase(SleepPhase::Awake);
                    }
                }
            }
        }
        prev = Some(now);
        std::thread::sleep(TICK);
    }
}

fn local_minutes() -> u32 {
    let now = Local::now();
    now.hour() * 60 + now.minute()
}

/// "HH:MM" → minutes since midnight (0..1440), or None if empty/invalid.
fn parse_hhmm(s: &str) -> Option<u32> {
    let (h, m) = s.split_once(':')?;
    let h: u32 = h.trim().parse().ok()?;
    let m: u32 = m.trim().parse().ok()?;
    if h < 24 && m < 60 {
        Some(h * 60 + m)
    } else {
        None
    }
}

/// True if `target` lies in (prev, now] on the 24h circle (fires once per
/// crossing, and still catches a target skipped by a missed/long tick).
fn crossed(prev: u32, now: u32, target: u32) -> bool {
    if prev == now {
        return false;
    }
    let span = (now + 1440 - prev) % 1440;
    let off = (target + 1440 - prev) % 1440;
    off != 0 && off <= span
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fires_once_at_crossing() {
        assert!(crossed(1409, 1410, 1410)); // 23:29 -> 23:30 hits 23:30
        assert!(crossed(1380, 1381, 1381)); // 23:00 -> 23:01 hits 23:01
        assert!(!crossed(1381, 1381, 1381)); // same minute, no refire
        assert!(!crossed(1380, 1381, 1382)); // target not yet reached
    }

    #[test]
    fn catches_skipped_minute_and_midnight_wrap() {
        assert!(crossed(1435, 5, 1439)); // 23:55 -> 00:05 across midnight hits 23:59
        assert!(crossed(1435, 5, 0)); // ...and hits 00:00
        assert!(!crossed(1435, 5, 10)); // 00:10 not reached yet
    }

    #[test]
    fn parses_times() {
        assert_eq!(parse_hhmm("23:30"), Some(1410));
        assert_eq!(parse_hhmm("00:00"), Some(0));
        assert_eq!(parse_hhmm(""), None);
        assert_eq!(parse_hhmm("25:00"), None);
    }
}
