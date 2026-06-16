// NemuriXR desktop backend — this process hosts the always-on automation engine
// (config + live state, and the automation logic added in later milestones) and
// an IPC server so the in-headset overlay can read state and send commands.
//
// The Svelte frontend talks to this engine in-process via Tauri commands; the VR
// overlay talks to it over the Unix socket. Keep this app running (it can sit in
// the background) for automations to work — in VR or in desktop VRChat.
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};

use nemurixr_core::config::{AudioLevel, BrightnessLevel};
use nemurixr_core::ipc::{self, Request, Response};
use nemurixr_core::{Config, SleepPhase, State};

mod audio;
mod brightness;
mod osc;
mod overlay_launcher;
mod schedule;
mod sound;
mod udev;
mod update;
mod vrchat;
mod vrchat_api;
mod vrchat_feature;

use overlay_launcher::OverlayChild;

use vrchat_api::{Api, Friend, InviteMessage, LoginOutcome, LoginStatus, SharedApi};

/// The in-process engine. Owns the source-of-truth config + live state.
pub(crate) struct Engine {
    config: Config,
    state: State,
    /// Last time the VR overlay talked to us (for the "overlay connected" status).
    last_overlay_seen: Option<Instant>,
    brightness: brightness::Backend,
    /// VRChat OSC target discovered via OSCQuery (mDNS), if any.
    osc_target: Option<SocketAddr>,
    /// Current VRChat instance location ("wrld_…:…"), for accepting invites.
    vrchat_instance: Option<String>,
    /// Last applied (brightness%, fan%) — the "from" point for the next fade.
    bright_current: Arc<Mutex<Option<(u8, u8)>>>,
    /// Bumped on each brightness transition; an in-flight fade aborts when it changes.
    fade_gen: Arc<AtomicU64>,
    /// Last audio output device an automation controlled (for the UI).
    audio_target: Arc<Mutex<Option<String>>>,
}

impl Engine {
    fn new() -> Self {
        let backend = brightness::detect();
        let mut state = State::default();
        state.brightness_backend = brightness::name(backend);
        log::info!("brightness backend: {}", brightness::name(backend).unwrap_or_else(|| "none".into()));
        Engine {
            config: Config::load(),
            state,
            last_overlay_seen: None,
            brightness: backend,
            osc_target: None,
            vrchat_instance: None,
            bright_current: Arc::new(Mutex::new(None)),
            fade_gen: Arc::new(AtomicU64::new(0)),
            audio_target: Arc::new(Mutex::new(None)),
        }
    }

    fn brightness_level(&self, phase: SleepPhase) -> BrightnessLevel {
        match phase {
            SleepPhase::Awake => self.config.brightness.on_wake,
            SleepPhase::Prepare => self.config.brightness.on_prepare,
            SleepPhase::Sleep => self.config.brightness.on_sleep,
        }
    }

    pub(crate) fn set_osc_target(&mut self, target: Option<SocketAddr>) {
        self.osc_target = target;
    }

    /// Fire the OSC message list for a phase (non-blocking).
    fn send_osc(&self, phase: SleepPhase) {
        let msgs = match phase {
            SleepPhase::Awake => self.config.osc.on_wake.clone(),
            SleepPhase::Prepare => self.config.osc.on_prepare.clone(),
            SleepPhase::Sleep => self.config.osc.on_sleep.clone(),
        };
        if msgs.is_empty() {
            return;
        }
        match osc::resolve_target(&self.config.osc, self.osc_target) {
            Some(t) => osc::send_sequence(t, msgs),
            None => log::warn!("OSC: no target resolved; skipping"),
        }
    }

    /// Fade brightness/fan into `level` over its transition time (cancelable).
    fn fade_brightness(&mut self, level: BrightnessLevel) {
        self.brightness = brightness::detect();
        self.state.brightness_backend = brightness::name(self.brightness);
        let backend = self.brightness;
        let gen = self.fade_gen.fetch_add(1, Ordering::SeqCst) + 1;
        let current = self.bright_current.clone();
        let fade_gen = self.fade_gen.clone();
        let to = (level.brightness, level.fan);
        let dur = level.transition_seconds;
        std::thread::spawn(move || brightness::transition(backend, current, fade_gen, gen, to, dur));
    }

    fn audio_level(&self, phase: SleepPhase) -> AudioLevel {
        match phase {
            SleepPhase::Awake => self.config.audio.on_wake,
            SleepPhase::Prepare => self.config.audio.on_prepare,
            SleepPhase::Sleep => self.config.audio.on_sleep,
        }
    }

    /// Apply a phase's audio level on a background thread (pactl spawns several
    /// short-lived processes; don't hold the engine lock for that).
    fn apply_audio(&self, level: AudioLevel) {
        let target = self.audio_target.clone();
        std::thread::spawn(move || {
            if let Some(desc) = audio::apply(&level) {
                *target.lock().unwrap() = Some(desc);
            }
        });
    }

    /// Apply `level` immediately (no fade) — used for previews.
    fn preview_brightness(&mut self, level: BrightnessLevel) {
        self.brightness = brightness::detect();
        self.state.brightness_backend = brightness::name(self.brightness);
        self.fade_gen.fetch_add(1, Ordering::SeqCst); // cancel any in-flight fade
        brightness::set_now(self.brightness, level.brightness, level.fan);
        *self.bright_current.lock().unwrap() = Some((level.brightness, level.fan));
    }

    fn apply_config(&mut self, config: Config) {
        self.config = config;
        self.config.save();
    }

    fn set_phase(&mut self, phase: SleepPhase) {
        if self.state.sleep_phase == phase {
            return;
        }
        self.state.sleep_phase = phase;
        log::info!("sleep phase -> {phase:?}");
        if self.config.brightness.enabled {
            let level = self.brightness_level(phase);
            self.fade_brightness(level);
        }
        if self.config.audio.enabled {
            let level = self.audio_level(phase);
            self.apply_audio(level);
        }
        self.send_osc(phase);
        self.run_command(phase);
        // Future: VRChat status automations also fire here.
    }

    /// Run the user's command for `phase` (via `sh -c`), if enabled and non-empty.
    /// Spawned on a thread so a slow script never blocks the engine.
    fn run_command(&self, phase: SleepPhase) {
        if !self.config.commands.enabled {
            return;
        }
        let cmd = match phase {
            SleepPhase::Awake => &self.config.commands.on_wake,
            SleepPhase::Prepare => &self.config.commands.on_prepare,
            SleepPhase::Sleep => &self.config.commands.on_sleep,
        }
        .trim()
        .to_string();
        if cmd.is_empty() {
            return;
        }
        log::info!("running {phase:?} command");
        std::thread::spawn(move || match std::process::Command::new("sh").arg("-c").arg(&cmd).spawn() {
            Ok(mut child) => {
                let _ = child.wait();
            }
            Err(e) => log::warn!("command failed to start: {e}"),
        });
    }


    /// Push a transient notice for the in-headset toast (auto-accept, status…).
    pub(crate) fn notify(&mut self, text: impl Into<String>) {
        self.state.notice = Some(text.into());
        self.state.notice_seq = self.state.notice_seq.wrapping_add(1);
    }

    /// A state snapshot with the derived `overlay_running` flag filled in.
    fn snapshot(&self) -> State {
        let mut s = self.state.clone();
        s.overlay_running = self.last_overlay_seen.is_some_and(|t| t.elapsed().as_secs() < 3);
        s.osc_target = osc::resolve_target(&self.config.osc, self.osc_target).map(|a| a.to_string());
        s.audio_target = self.audio_target.lock().unwrap().clone();
        s
    }

    // --- used by the VRChat watcher (crate-internal) ---

    pub(crate) fn vrchat_log_dir(&self) -> String {
        self.config.vrchat.log_dir.clone()
    }

    pub(crate) fn auto_launch_overlay(&self) -> bool {
        self.config.auto_launch_overlay
    }

    pub(crate) fn set_vrchat(&mut self, world: Option<String>, instance: Option<String>, player_count: u32) {
        self.state.vrchat_world = world;
        self.vrchat_instance = instance;
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
        if n.only_when_sleep && !self.state.sleep_phase.is_active() {
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
fn set_phase(engine: tauri::State<Shared>, phase: SleepPhase) {
    engine.lock().unwrap().set_phase(phase);
}

fn phase_from_str(s: &str) -> SleepPhase {
    match s {
        "prepare" => SleepPhase::Prepare,
        "sleep" => SleepPhase::Sleep,
        _ => SleepPhase::Awake,
    }
}

/// Apply a phase's brightness/fan level now (preview). `which` is awake/prepare/sleep.
#[tauri::command]
fn apply_brightness(engine: tauri::State<Shared>, which: String) {
    let mut g = engine.lock().unwrap();
    let level = g.brightness_level(phase_from_str(&which));
    g.preview_brightness(level);
}

/// Send a phase's OSC message list now (preview). `which` is awake/prepare/sleep.
#[tauri::command]
fn send_osc(engine: tauri::State<Shared>, which: String) {
    engine.lock().unwrap().send_osc(phase_from_str(&which));
}

/// Apply a phase's audio level now (preview). `which` is awake/prepare/sleep.
#[tauri::command]
fn apply_audio(engine: tauri::State<Shared>, which: String) {
    let g = engine.lock().unwrap();
    let level = g.audio_level(phase_from_str(&which));
    g.apply_audio(level);
}

/// Run a phase's command now (preview). `which` is awake/prepare/sleep.
#[tauri::command]
fn test_command(engine: tauri::State<Shared>, which: String) {
    engine.lock().unwrap().run_command(phase_from_str(&which));
}

/// Preview the wake-up alarm sound (custom file if set, else the default chime).
#[tauri::command]
fn test_alarm(engine: tauri::State<Shared>) {
    let custom = engine.lock().unwrap().config.sleep.wake.alarm_sound.clone();
    sound::play_notification("alarm", &custom);
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

// --- VRChat account (auth) ---

// All of these touch the network and/or the keyring, so they run async via
// spawn_blocking — a sync command would block the main (UI) thread.
#[tauri::command]
async fn vrchat_login(vrc: tauri::State<'_, SharedApi>, username: String, password: String) -> Result<LoginOutcome, String> {
    let api = vrc.inner().clone();
    tauri::async_runtime::spawn_blocking(move || api.lock().unwrap().login(&username, &password))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn vrchat_verify_2fa(vrc: tauri::State<'_, SharedApi>, method: String, code: String) -> Result<LoginOutcome, String> {
    let api = vrc.inner().clone();
    tauri::async_runtime::spawn_blocking(move || api.lock().unwrap().verify_2fa(&method, &code))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn vrchat_logout(vrc: tauri::State<'_, SharedApi>) -> Result<LoginStatus, String> {
    let api = vrc.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut a = api.lock().unwrap();
        a.logout();
        a.login_status()
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn vrchat_status(vrc: tauri::State<'_, SharedApi>) -> Result<LoginStatus, String> {
    let api = vrc.inner().clone();
    tauri::async_runtime::spawn_blocking(move || api.lock().unwrap().login_status())
        .await
        .map_err(|e| e.to_string())
}

/// Manually launch the in-headset overlay now.
#[tauri::command]
fn launch_overlay(child: tauri::State<OverlayChild>) -> bool {
    overlay_launcher::launch_now(&child)
}

/// The app version (for display in the UI).
#[tauri::command]
fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Check GitHub for a newer release (None if up to date / offline).
#[tauri::command]
async fn check_update() -> Result<Option<update::UpdateInfo>, String> {
    tauri::async_runtime::spawn_blocking(update::check).await.map_err(|e| e.to_string())
}

/// Beyond HID access status: "absent" | "needs_rule" | "ready".
#[tauri::command]
fn beyond_status() -> &'static str {
    brightness::beyond_status()
}

/// The udev rule text (for the manual-install fallback shown in Settings).
#[tauri::command]
fn beyond_rule_text() -> String {
    udev::rule_text()
}

/// Install the Beyond udev rule (prompts for authorization via pkexec).
#[tauri::command]
async fn install_beyond_rule() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(udev::install).await.map_err(|e| e.to_string())?
}

/// Friends list for the auto-accept whitelist picker. Async + spawn_blocking so
/// the multi-page fetch runs off the main thread (no UI freeze); grabs the
/// client + cookie under a brief lock, then fetches all pages without holding it.
#[tauri::command]
async fn vrchat_friends(vrc: tauri::State<'_, SharedApi>) -> Result<Vec<Friend>, String> {
    let api = vrc.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let req = api.lock().unwrap().friends_request();
        match req {
            Some((client, cookie)) => vrchat_api::fetch_friends(&client, &cookie),
            None => Vec::new(),
        }
    })
    .await
    .map_err(|e| e.to_string())
}

/// Message templates for the picker. `kind` is "message" (accept) or
/// "requestResponse" (decline).
#[tauri::command]
async fn vrchat_messages(vrc: tauri::State<'_, SharedApi>, kind: String) -> Result<Vec<InviteMessage>, String> {
    let api = vrc.inner().clone();
    tauri::async_runtime::spawn_blocking(move || api.lock().unwrap().messages(&kind))
        .await
        .map_err(|e| e.to_string())?
}

/// Edit one message slot (rate-limited by VRChat to ~once/hour/slot).
#[tauri::command]
async fn vrchat_update_message(vrc: tauri::State<'_, SharedApi>, kind: String, slot: u32, text: String) -> Result<Vec<InviteMessage>, String> {
    let api = vrc.inner().clone();
    tauri::async_runtime::spawn_blocking(move || api.lock().unwrap().update_message(&kind, slot, &text))
        .await
        .map_err(|e| e.to_string())?
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

    // VRChat account: validate any restored session in the background.
    let vrc: SharedApi = Arc::new(Mutex::new(Api::new()));
    {
        let v = vrc.clone();
        std::thread::spawn(move || v.lock().unwrap().restore());
    }
    // VRChat automation engine: pipeline websocket (auto-accept) + status automations.
    vrchat_feature::spawn(engine.clone(), vrc.clone());

    // Auto-launch the in-headset overlay when a Monado session starts.
    let overlay_child: OverlayChild = Arc::new(Mutex::new(None));
    overlay_launcher::spawn_watcher(engine.clone(), overlay_child.clone());

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
                Request::SetPhase { phase } => {
                    g.set_phase(phase);
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
        .manage(vrc)
        .manage(overlay_child)
        .invoke_handler(tauri::generate_handler![
            get_config,
            set_config,
            get_state,
            set_phase,
            apply_brightness,
            send_osc,
            apply_audio,
            test_sound,
            test_command,
            test_alarm,
            launch_overlay,
            app_version,
            check_update,
            beyond_status,
            beyond_rule_text,
            install_beyond_rule,
            vrchat_login,
            vrchat_verify_2fa,
            vrchat_logout,
            vrchat_status,
            vrchat_friends,
            vrchat_messages,
            vrchat_update_message
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
                    "quit" => {
                        overlay_launcher::kill(&app.state::<OverlayChild>());
                        app.exit(0);
                    }
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
