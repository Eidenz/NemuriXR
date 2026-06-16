// The overlay's link to the desktop-hosted engine. A background thread keeps a
// Unix-socket connection, caches the latest config/state, and ships commands —
// so the 90 Hz render loop never blocks on IPC. When the desktop app isn't
// running it simply reports `connected = false` and retries.
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use nemurixr_core::config::Sensitivity;
use nemurixr_core::ipc::Client;
use nemurixr_core::{Config, SleepPhase, SleepPosition, SleepTrigger, State};

/// The sleep-detection settings the overlay needs each frame.
pub struct Detection {
    pub enabled: bool,
    pub always: bool,
    pub start: String,
    pub end: String,
    pub sensitivity: Sensitivity,
    pub minutes: u32,
    /// Calibrated sleep poses (head-local gravity vectors); empty = uncalibrated.
    pub poses: Vec<[f32; 3]>,
    pub pose_tolerance: u32,
}

enum Cmd {
    SetPhase(SleepPhase, SleepTrigger),
    SetConfig(Config),
    SetSleepingPosition(SleepPosition),
}

struct LinkState {
    connected: bool,
    config: Config,
    state: State,
}

pub struct EngineLink {
    shared: Arc<Mutex<LinkState>>,
    tx: Sender<Cmd>,
}

impl EngineLink {
    pub fn spawn() -> Self {
        let shared = Arc::new(Mutex::new(LinkState { connected: false, config: Config::load(), state: State::default() }));
        let (tx, rx) = channel::<Cmd>();
        let sh = shared.clone();
        std::thread::spawn(move || worker(sh, rx));
        EngineLink { shared, tx }
    }

    pub fn connected(&self) -> bool {
        self.shared.lock().unwrap().connected
    }

    pub fn state(&self) -> State {
        self.shared.lock().unwrap().state.clone()
    }

    pub fn config(&self) -> Config {
        self.shared.lock().unwrap().config.clone()
    }

    pub fn block_game_input(&self) -> bool {
        self.shared.lock().unwrap().config.block_game_input
    }

    /// True when the overlay should report the lying position — either the
    /// continuous sleeping-pose feature is on, or the safety net may pose you.
    pub fn report_position(&self) -> bool {
        let g = self.shared.lock().unwrap();
        g.config.vrchat.sleeping_pose.enabled || (g.config.safety_net.enabled && g.config.safety_net.pose)
    }

    /// Report the current lying position to the engine (it sends the avatar OSC).
    pub fn set_sleeping_position(&self, position: SleepPosition) {
        let _ = self.tx.send(Cmd::SetSleepingPosition(position));
    }

    pub fn detection(&self) -> Detection {
        let g = self.shared.lock().unwrap();
        let s = &g.config.sleep;
        Detection {
            enabled: s.detection_enabled,
            always: s.detection_always,
            start: s.detect_start.clone(),
            end: s.detect_end.clone(),
            sensitivity: s.detection_sensitivity,
            minutes: s.detection_minutes,
            poses: s.detection_poses.iter().map(|p| p.gravity).collect(),
            pose_tolerance: s.detection_pose_tolerance,
        }
    }

    /// Optimistically reflect the change locally, then queue it for the engine.
    pub fn set_phase(&self, phase: SleepPhase, trigger: SleepTrigger) {
        self.shared.lock().unwrap().state.sleep_phase = phase;
        let _ = self.tx.send(Cmd::SetPhase(phase, trigger));
    }

    pub fn set_config(&self, config: Config) {
        self.shared.lock().unwrap().config = config.clone();
        let _ = self.tx.send(Cmd::SetConfig(config));
    }
}

fn worker(shared: Arc<Mutex<LinkState>>, rx: Receiver<Cmd>) {
    loop {
        match Client::connect() {
            Ok(mut c) => {
                if let Ok(cfg) = c.get_config() {
                    shared.lock().unwrap().config = cfg;
                }
                shared.lock().unwrap().connected = true;
                if !pump(&mut c, &shared, &rx) {
                    shared.lock().unwrap().connected = false;
                }
            }
            Err(_) => {
                shared.lock().unwrap().connected = false;
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

/// Poll state + flush commands until the connection drops. Returns false on drop.
fn pump(c: &mut Client, shared: &Arc<Mutex<LinkState>>, rx: &Receiver<Cmd>) -> bool {
    loop {
        while let Ok(cmd) = rx.try_recv() {
            let r = match cmd {
                Cmd::SetPhase(p, t) => c.set_phase(p, t),
                Cmd::SetConfig(cfg) => c.set_config(cfg),
                Cmd::SetSleepingPosition(pos) => c.set_sleeping_position(pos),
            };
            if r.is_err() {
                return false;
            }
        }
        match c.get_state() {
            Ok(st) => shared.lock().unwrap().state = st,
            Err(_) => return false,
        }
        if let Ok(cfg) = c.get_config() {
            shared.lock().unwrap().config = cfg;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}
