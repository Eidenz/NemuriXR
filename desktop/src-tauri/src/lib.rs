// NemuriXR desktop backend — this process hosts the always-on automation engine
// (config + live state, and the automation logic added in later milestones) and
// an IPC server so the in-headset overlay can read state and send commands.
//
// The Svelte frontend talks to this engine in-process via Tauri commands; the VR
// overlay talks to it over the Unix socket. Keep this app running (it can sit in
// the background) for automations to work — in VR or in desktop VRChat.
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};

use nemurixr_core::ipc::{self, Request, Response};
use nemurixr_core::{Config, State};

mod brightness;
mod osc;
mod schedule;
mod sound;
mod vrchat;

/// The in-process engine. Owns the source-of-truth config + live state.
pub(crate) struct Engine {
    config: Config,
    state: State,
    /// Last time the VR overlay talked to us (for the "overlay connected" status).
    last_overlay_seen: Option<Instant>,
    brightness: brightness::Backend,
    /// VRChat OSC target discovered via OSCQuery (mDNS), if any.
    osc_target: Option<SocketAddr>,
}

impl Engine {
    fn new() -> Self {
        let backend = brightness::detect();
        let mut state = State::default();
        state.brightness_backend = brightness::name(backend);
        log::info!("brightness backend: {}", brightness::name(backend).unwrap_or_else(|| "none".into()));
        Engine { config: Config::load(), state, last_overlay_seen: None, brightness: backend, osc_target: None }
    }

    pub(crate) fn set_osc_target(&mut self, target: Option<SocketAddr>) {
        self.osc_target = target;
    }

    /// Fire the OSC message list for a sleep/wake transition (non-blocking).
    fn send_osc(&self, sleeping: bool) {
        let msgs = if sleeping { self.config.osc.on_sleep.clone() } else { self.config.osc.on_wake.clone() };
        if msgs.is_empty() {
            return;
        }
        match osc::resolve_target(&self.config.osc, self.osc_target) {
            Some(t) => osc::send_sequence(t, msgs),
            None => log::warn!("OSC: no target resolved; skipping"),
        }
    }

    /// Re-detect the backend and apply the sleep or wake brightness/fan level.
    fn apply_brightness_level(&mut self, sleeping: bool) {
        self.brightness = brightness::detect();
        self.state.brightness_backend = brightness::name(self.brightness);
        let lvl = if sleeping { self.config.brightness.on_sleep } else { self.config.brightness.on_wake };
        brightness::apply(self.brightness, lvl.brightness, lvl.fan);
    }

    fn apply_config(&mut self, config: Config) {
        self.config = config;
        self.config.save();
        // Later milestones re-apply live settings (brightness, watchers…) here.
    }

    fn set_sleep(&mut self, active: bool) {
        if self.state.sleep_active != active {
            self.state.sleep_active = active;
            log::info!("sleep mode -> {active}");
            if self.config.brightness.enabled {
                self.apply_brightness_level(active);
            }
            self.send_osc(active);
            // Future: VRChat status automations also fire here.
        }
    }

    /// A state snapshot with the derived `overlay_running` flag filled in.
    fn snapshot(&self) -> State {
        let mut s = self.state.clone();
        s.overlay_running = self.last_overlay_seen.is_some_and(|t| t.elapsed().as_secs() < 3);
        s.osc_target = osc::resolve_target(&self.config.osc, self.osc_target).map(|a| a.to_string());
        s
    }

    // --- used by the VRChat watcher (crate-internal) ---

    pub(crate) fn vrchat_log_dir(&self) -> String {
        self.config.vrchat.log_dir.clone()
    }

    pub(crate) fn set_vrchat(&mut self, world: Option<String>, player_count: u32) {
        self.state.vrchat_world = world;
        self.state.player_count = player_count;
    }

    /// Decide whether to play a join/leave sound, returning the configured sound
    /// path ("" = bundled default) when it should play. `alone_condition` is
    /// "were you alone before this join" / "are you alone after this leave".
    pub(crate) fn join_notify_sound(&self, is_join: bool, alone_condition: bool) -> Option<String> {
        let n = &self.config.vrchat.join_notifications;
        if !n.enabled {
            return None;
        }
        if n.only_when_alone && !alone_condition {
            return None;
        }
        if n.only_when_sleep && !self.state.sleep_active {
            return None;
        }
        Some(if is_join { n.join_sound.clone() } else { n.leave_sound.clone() })
    }
}

type Shared = Arc<Mutex<Engine>>;

#[tauri::command]
fn get_config(engine: tauri::State<Shared>) -> Config {
    engine.lock().unwrap().config.clone()
}

#[tauri::command]
fn set_config(engine: tauri::State<Shared>, config: Config) {
    engine.lock().unwrap().apply_config(config);
}

#[tauri::command]
fn get_state(engine: tauri::State<Shared>) -> State {
    engine.lock().unwrap().snapshot()
}

#[tauri::command]
fn set_sleep(engine: tauri::State<Shared>, active: bool) {
    engine.lock().unwrap().set_sleep(active);
}

/// Apply a brightness/fan level now (preview). `which` is "sleep" or "wake".
#[tauri::command]
fn apply_brightness(engine: tauri::State<Shared>, which: String) {
    engine.lock().unwrap().apply_brightness_level(which == "sleep");
}

/// Send an OSC message list now (preview). `which` is "sleep" or "wake".
#[tauri::command]
fn send_osc(engine: tauri::State<Shared>, which: String) {
    engine.lock().unwrap().send_osc(which == "sleep");
}

/// Preview a notification sound. `kind` is "join" or "leave".
#[tauri::command]
fn test_sound(engine: tauri::State<Shared>, kind: String) {
    let custom = {
        let g = engine.lock().unwrap();
        let n = &g.config.vrchat.join_notifications;
        if kind == "leave" {
            n.leave_sound.clone()
        } else {
            n.join_sound.clone()
        }
    };
    sound::play_notification(if kind == "leave" { "leave" } else { "join" }, &custom);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let engine: Shared = Arc::new(Mutex::new(Engine::new()));

    // VRChat log watcher: live player count + join/leave sounds (works without VR).
    vrchat::spawn(engine.clone());
    // OSCQuery discovery of VRChat's OSC port.
    osc::spawn_discovery(engine.clone());
    // Time-based sleep schedule.
    schedule::spawn(engine.clone());

    // IPC server for the VR overlay (reads state, sends sleep/config commands).
    {
        let e = engine.clone();
        if let Err(err) = ipc::serve(move |req| {
            let mut g = e.lock().unwrap();
            g.last_overlay_seen = Some(Instant::now());
            match req {
                Request::GetConfig => Response::Config { config: g.config.clone() },
                Request::GetState => Response::State { state: g.snapshot() },
                Request::SetConfig { config } => {
                    g.apply_config(config);
                    Response::Ok
                }
                Request::SetSleep { active } => {
                    g.set_sleep(active);
                    Response::Ok
                }
            }
        }) {
            log::warn!("IPC server failed to start: {err} (overlay won't be able to connect)");
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(engine)
        .invoke_handler(tauri::generate_handler![
            get_config,
            set_config,
            get_state,
            set_sleep,
            apply_brightness,
            send_osc,
            test_sound
        ])
        .setup(|app| {
            // System tray: left-click opens the window; the menu has Open + Quit.
            let open = MenuItem::with_id(app, "open", "Open NemuriXR", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("NemuriXR")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_main(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        // Closing the window hides it to the tray (the engine keeps running);
        // full quit is via the tray's Quit item.
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_main(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}
