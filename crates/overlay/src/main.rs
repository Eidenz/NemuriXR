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
mod mathx;
mod overlay;
mod theme;
mod ui;

use std::env;

use anyhow::Result;
use openxr as xr;

use blocker::Blocker;
use client::EngineLink;
use mathx::locate_pose;
use overlay::{front_pose, posef, Input, Laser, Panel, TargetId};
use ui::{build_menu, MenuAction, Screen};

const MENU: TargetId = 0;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    if let Err(e) = run() {
        log::error!("NemuriXR exited with error: {e:?}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let (mut xr, gpu) = overlay::session::init("NemuriXR")?;

    let opacity: f32 = env::var("NEMURI_OPACITY").ok().and_then(|s| s.parse().ok()).unwrap_or(0.92);
    let alpha_mode = env::var("NEMURI_NO_ALPHA").is_err();
    let laser_on = env::var("NEMURI_NO_LASER").is_err();
    let panel_alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
    log::info!("alpha_mode={alpha_mode} opacity={opacity} laser={laser_on}");

    let menu_px = (980u32, 620u32);
    let menu_w = 0.52f32;
    let mut menu = Panel::new(&gpu, &xr.session, menu_px, (menu_w, menu_w * menu_px.1 as f32 / menu_px.0 as f32), posef([0.0, 0.0, -1.0]))?;
    let mut laser = Laser::new(&gpu, &xr.session)?;
    let mut input = Input::new(&xr.instance, &xr.session)?;
    let mut blocker = Blocker::new();

    // The link to the desktop-hosted engine (background thread; never blocks us).
    let link = EngineLink::spawn();

    let mut menu_visible = false;
    let mut screen = Screen::Home;

    log::info!("NemuriXR overlay ready. Double-press SYSTEM to open the menu; point to interact, grip to move it.");

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
        let sleeping = engine_state.sleep_active;

        // Input: toggle the menu, then point/grab it.
        let mut ptr: Option<(f32, f32, bool)> = None;
        let mut laser_ray: Option<(xr::Posef, f32)> = None;
        let mut pointing_panel = false;
        if focused {
            input.sync(&xr.session)?;
            if input.system_double_press(&xr.session)? {
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

        if !menu_visible {
            xr.frame_stream.end(time, xr.blend_mode, &[])?;
            continue;
        }

        // Render the menu and forward its actions to the engine.
        let clock = chrono::Local::now().format("%H:%M").to_string();
        let mut action = MenuAction::None;
        let mut cfg = link.config();
        let mut cfg_changed = false;
        menu.render(&gpu, alpha_mode, ptr, |ctx| {
            action = build_menu(ctx, screen, sleeping, connected, &clock, &mut cfg, &mut cfg_changed, panel_alpha);
        })?;
        match action {
            MenuAction::ToggleSleep => link.set_sleep(!sleeping),
            MenuAction::OpenAutomations => screen = Screen::Automations,
            MenuAction::Back => screen = Screen::Home,
            MenuAction::None => {}
        }
        if cfg_changed {
            link.set_config(cfg);
        }

        let laser_ready = laser_on && laser_ray.is_some() && laser.fill(&gpu).is_ok();
        let menu_quad = menu.quad(&xr.space, alpha_mode);
        let laser_quad = match (laser_ready, laser_ray, hmd) {
            (true, Some((aim, t)), Some(h)) => Some(laser.quad(&xr.space, &aim, t, &h)),
            _ => None,
        };
        let mut layers: Vec<&xr::CompositionLayerBase<xr::Vulkan>> = Vec::new();
        layers.push(&menu_quad);
        if let Some(q) = &laser_quad {
            layers.push(q);
        }
        xr.frame_stream.end(time, xr.blend_mode, &layers)?;
    }
}
