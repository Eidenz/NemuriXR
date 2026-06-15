// NemuriXR in-headset overlay — a thin remote for the desktop-hosted engine.
//
// An OpenXR overlay session (XR_EXTX_overlay) rendering a grabbable egui "quick
// menu" of toggles + Sleep Mode, built on the monado-frame overlay foundation.
// It does not run the automations itself: it connects to the always-on engine in
// the NemuriXR desktop app over a Unix socket (see `client`), shows its live
// state, and sends toggle/sleep commands. Keep the desktop app running (it lives
// in the tray) for this to do anything. See README.md.

// The overlay foundation is a small reusable toolkit consumed incrementally
// across milestones, so some of its API is intentionally unused in v1.
#![allow(dead_code)]

mod blocker;
mod client;
mod detector;
mod mathx;
mod overlay;
mod theme;
mod ui;

use std::env;
use std::time::{Duration, Instant};

use anyhow::Result;
use openxr as xr;

use blocker::Blocker;
use client::{Detection, EngineLink};
use detector::{nearest_pose_deg, Detector, Tick};
use mathx::{gravity_local, locate_pose};
use nemurixr_core::SleepPhase;
use overlay::{front_pose, posef, Input, Laser, Panel, TargetId};
use ui::{build_countdown, build_menu, build_toast, MenuAction, Screen};

const MENU: TargetId = 0;
/// Delay between pressing "Capture a pose" and recording it — time to settle in.
const CALIB_DELAY: Duration = Duration::from_secs(5);
/// How long an event toast stays on screen.
const TOAST_SHOW: Duration = Duration::from_secs(4);
/// Toast fade-out duration at the end.
const TOAST_FADE: f32 = 0.6;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    if let Err(e) = run() {
        log::error!("NemuriXR exited with error: {e:?}");
        std::process::exit(1);
    }
}

/// Single-instance guard (flock): if another overlay holds the lock, exit. The
/// returned file must stay alive for the process lifetime to hold the lock.
fn acquire_singleton() -> Option<std::fs::File> {
    use std::os::unix::io::AsRawFd;
    let dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let path = format!("{dir}/nemurixr-overlay.lock");
    let file = std::fs::OpenOptions::new().create(true).write(true).open(&path).ok()?;
    let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if rc == 0 {
        Some(file)
    } else {
        None
    }
}

fn run() -> Result<()> {
    let _singleton = match acquire_singleton() {
        Some(f) => f,
        None => {
            log::info!("another NemuriXR overlay is already running; exiting");
            return Ok(());
        }
    };

    let (mut xr, gpu) = overlay::session::init("NemuriXR")?;

    let opacity: f32 = env::var("NEMURI_OPACITY").ok().and_then(|s| s.parse().ok()).unwrap_or(0.92);
    let alpha_mode = env::var("NEMURI_NO_ALPHA").is_err();
    let laser_on = env::var("NEMURI_NO_LASER").is_err();
    let panel_alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
    log::info!("alpha_mode={alpha_mode} opacity={opacity} laser={laser_on}");

    let menu_px = (980u32, 880u32);
    let menu_w = 0.52f32;
    let mut menu = Panel::new(&gpu, &xr.session, menu_px, (menu_w, menu_w * menu_px.1 as f32 / menu_px.0 as f32), posef([0.0, 0.0, -1.0]))?;
    let mut laser = Laser::new(&gpu, &xr.session)?;
    let mut input = Input::new(&xr.instance, &xr.session)?;
    let mut blocker = Blocker::new();

    // Motion-based sleep detection + its own (non-interactive) countdown panel.
    let mut detector = Detector::new();
    let cd_px = (640u32, 360u32);
    let cd_w = 0.42f32;
    let mut countdown = Panel::new(&gpu, &xr.session, cd_px, (cd_w, cd_w * cd_px.1 as f32 / cd_px.0 as f32), posef([0.0, 0.0, -1.0]))?;

    // Transient event toasts (auto-accept, status changes…), shown above center.
    let toast_px = (760u32, 150u32);
    let toast_w = 0.46f32;
    let mut toast_panel =
        Panel::new(&gpu, &xr.session, toast_px, (toast_w, toast_w * toast_px.1 as f32 / toast_px.0 as f32), posef([0.0, 0.0, -1.0]))?;

    // The link to the desktop-hosted engine (background thread; never blocks us).
    let link = EngineLink::spawn();

    let mut menu_visible = false;
    let mut screen = Screen::Home;
    // When set, a sleep-pose capture is counting down to this instant.
    let mut capture_at: Option<Instant> = None;
    // Event-toast state: the last notice seq we've shown, and the active toast.
    let mut last_notice_seq: Option<u64> = None;
    let mut toast: Option<(String, Instant)> = None;

    log::info!("NemuriXR overlay ready. Double-tap A (right) to open the menu; point to interact, grip to move it.");

    let mut events = xr::EventDataBuffer::new();
    let mut running = false;
    let mut focused = false;

    loop {
        while let Some(event) = xr.instance.poll_event(&mut events)? {
            use xr::Event::*;
            match event {
                SessionStateChanged(e) => {
                    log::info!("session state -> {:?}", e.state());
                    focused = e.state() == xr::SessionState::FOCUSED;
                    match e.state() {
                        xr::SessionState::READY => {
                            xr.session.begin(xr::ViewConfigurationType::PRIMARY_STEREO)?;
                            running = true;
                        }
                        xr::SessionState::STOPPING => {
                            xr.session.end()?;
                            running = false;
                        }
                        xr::SessionState::EXITING | xr::SessionState::LOSS_PENDING => return Ok(()),
                        _ => {}
                    }
                }
                InstanceLossPending(_) => return Ok(()),
                _ => {}
            }
        }
        if !running {
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        let frame_state = xr.frame_waiter.wait()?;
        xr.frame_stream.begin()?;
        if !frame_state.should_render {
            xr.frame_stream.end(frame_state.predicted_display_time, xr.blend_mode, &[])?;
            continue;
        }
        let time = frame_state.predicted_display_time;
        let hmd = locate_pose(&xr.view_space, &xr.space, time);

        let engine_state = link.state();
        let connected = link.connected();
        let phase = engine_state.sleep_phase;

        // Event toasts: show a new notice once (ignore any that predate launch).
        match last_notice_seq {
            None => last_notice_seq = Some(engine_state.notice_seq),
            Some(prev) if prev != engine_state.notice_seq => {
                last_notice_seq = Some(engine_state.notice_seq);
                if let Some(text) = engine_state.notice.clone() {
                    toast = Some((text, Instant::now() + TOAST_SHOW));
                }
            }
            _ => {}
        }
        if toast.as_ref().is_some_and(|(_, until)| Instant::now() >= *until) {
            toast = None;
        }

        // Input: toggle the menu, then point/grab it.
        let mut ptr: Option<(f32, f32, bool)> = None;
        let mut laser_ray: Option<(xr::Posef, f32)> = None;
        let mut pointing_panel = false;
        let mut controller_active = false;
        if focused {
            input.sync(&xr.session)?;
            controller_active = input.any_input(&xr.session)?;
            if input.a_double_press(&xr.session)? {
                menu_visible = !menu_visible;
                input.clear_grab();
                if menu_visible {
                    screen = Screen::Home;
                    if let Some(h) = hmd {
                        menu.pose = front_pose(&h, 0.8, 0.0);
                    }
                }
                log::info!("menu {}", if menu_visible { "shown" } else { "hidden" });
            }
            if menu_visible {
                let targets = [(MENU, menu.pose, menu.size_m)];
                let it = input.process(&xr.session, &xr.space, time, &targets)?;
                for (tid, pose) in &it.moves {
                    if *tid == MENU {
                        menu.pose = *pose;
                    }
                }
                ptr = it.pointer(MENU);
                laser_ray = it.laser;
                pointing_panel = it.pointing_panel;
            }
        }

        blocker.set(pointing_panel && link.block_game_input());

        // Gravity direction in head-local space — yaw-invariant pose descriptor.
        let g_local = hmd.map(|h| gravity_local(&h.orientation));

        // Motion-based sleep detection. Only while awake, connected, and (if a
        // window is set) inside it; the open menu pauses it. When sleep poses are
        // calibrated the head must also be near one (else stillness alone arms it).
        // Any controller input or head movement cancels the countdown; reaching
        // zero enters Sleep.
        let det = link.detection();
        let pose_ok = det.poses.is_empty()
            || g_local
                .and_then(|g| nearest_pose_deg(g, &det.poses))
                .is_some_and(|a| a <= det.pose_tolerance as f32);
        let det_active = connected
            && det.enabled
            && phase == SleepPhase::Awake
            && !menu_visible
            && hmd.is_some()
            && pose_ok
            && within_window(&det);
        let hmd_ref = hmd.unwrap_or(xr::Posef::IDENTITY);
        let mut countdown_secs: Option<u32> = None;
        match detector.update(det_active, &hmd_ref, controller_active, det.sensitivity, det.minutes) {
            Tick::Sleep => link.set_phase(SleepPhase::Sleep),
            Tick::Counting(s) => countdown_secs = Some(s),
            Tick::Idle => {}
        }

        // An in-progress pose capture only makes sense on the calibrate screen.
        if !menu_visible || screen != Screen::Calibrate {
            capture_at = None;
        }

        // Render the menu (when open) and forward its actions to the engine.
        let mut menu_quad = None;
        let mut laser_quad = None;
        if menu_visible {
            let clock = chrono::Local::now().format("%H:%M").to_string();
            let capture_secs = capture_at.map(|t| {
                let now = Instant::now();
                if now >= t {
                    0
                } else {
                    (t - now).as_secs() as u32 + 1
                }
            });
            // While a capture counts down, keep the panel in view as the user
            // moves into their sleeping position.
            if capture_at.is_some() {
                if let Some(h) = hmd {
                    menu.pose = front_pose(&h, 0.8, 0.0);
                }
            }
            let mut action = MenuAction::None;
            let mut cfg = link.config();
            let mut cfg_changed = false;
            menu.render(&gpu, alpha_mode, ptr, |ctx| {
                action = build_menu(ctx, screen, phase, connected, &clock, &mut cfg, &mut cfg_changed, capture_secs, panel_alpha);
            })?;
            match action {
                MenuAction::SetPhase(p) => link.set_phase(p),
                MenuAction::OpenAutomations => screen = Screen::Automations,
                MenuAction::OpenCalibrate => screen = Screen::Calibrate,
                MenuAction::CapturePose => capture_at = Some(Instant::now() + CALIB_DELAY),
                MenuAction::ClearPoses => {
                    cfg.sleep.detection_poses.clear();
                    cfg_changed = true;
                }
                MenuAction::Back => screen = Screen::Home,
                MenuAction::None => {}
            }
            if cfg_changed {
                link.set_config(cfg);
            }

            // Capture finished? Record the current head-local gravity as a pose.
            if let Some(t) = capture_at {
                if Instant::now() >= t {
                    capture_at = None;
                    if let Some(g) = g_local {
                        let mut c = link.config();
                        c.sleep.detection_poses.push(g);
                        link.set_config(c);
                        input.pulse(&xr.session, false);
                    }
                }
            }

            let laser_ready = laser_on && laser_ray.is_some() && laser.fill(&gpu).is_ok();
            menu_quad = Some(menu.quad(&xr.space, alpha_mode));
            laser_quad = match (laser_ready, laser_ray, hmd) {
                (true, Some((aim, t)), Some(h)) => Some(laser.quad(&xr.space, &aim, t, &h)),
                _ => None,
            };
        }

        // The cancelable sleep countdown, shown centered in front of the user.
        let mut cd_quad = None;
        if let (Some(secs), Some(h)) = (countdown_secs, hmd) {
            countdown.pose = front_pose(&h, 1.0, 0.0);
            countdown.render(&gpu, alpha_mode, None, |ctx| {
                build_countdown(ctx, secs, panel_alpha);
            })?;
            cd_quad = Some(countdown.quad(&xr.space, alpha_mode));
        }

        // Event toast, slightly above center, fading out at the end.
        let mut toast_quad = None;
        if let (Some((text, until)), Some(h)) = (&toast, hmd) {
            let remaining = until.saturating_duration_since(Instant::now()).as_secs_f32();
            let fade = (remaining / TOAST_FADE).clamp(0.0, 1.0);
            let a = (panel_alpha as f32 * fade) as u8;
            let mut pose = front_pose(&h, 1.3, 0.0);
            pose.position.y += 0.30;
            toast_panel.pose = pose;
            let text = text.clone();
            toast_panel.render(&gpu, alpha_mode, None, |ctx| {
                build_toast(ctx, &text, a);
            })?;
            toast_quad = Some(toast_panel.quad(&xr.space, alpha_mode));
        }

        let mut layers: Vec<&xr::CompositionLayerBase<xr::Vulkan>> = Vec::new();
        if let Some(q) = &menu_quad {
            layers.push(q);
        }
        if let Some(q) = &laser_quad {
            layers.push(q);
        }
        if let Some(q) = &cd_quad {
            layers.push(q);
        }
        if let Some(q) = &toast_quad {
            layers.push(q);
        }
        xr.frame_stream.end(time, xr.blend_mode, &layers)?;
    }
}

/// Is "now" inside the detection window? Always-on or unset times => always true.
fn within_window(det: &Detection) -> bool {
    if det.always || det.start.is_empty() || det.end.is_empty() {
        return true;
    }
    let parse = |s: &str| -> Option<chrono::NaiveTime> {
        let mut it = s.split(':');
        let h: u32 = it.next()?.trim().parse().ok()?;
        let m: u32 = it.next()?.trim().parse().ok()?;
        chrono::NaiveTime::from_hms_opt(h, m, 0)
    };
    let (Some(start), Some(end)) = (parse(&det.start), parse(&det.end)) else {
        return true;
    };
    let now = chrono::Local::now().time();
    if start <= end {
        now >= start && now < end
    } else {
        // Window crosses midnight (e.g. 23:00 -> 07:00).
        now >= start || now < end
    }
}
