// The overlay's link to the desktop-hosted engine. A background thread keeps a
// Unix-socket connection, caches the latest config/state, and ships commands —
// so the 90 Hz render loop never blocks on IPC. When the desktop app isn't
// running it simply reports `connected = false` and retries.
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use nemurixr_core::ipc::Client;
use nemurixr_core::{Config, State};

enum Cmd {
    SetSleep(bool),
    SetConfig(Config),
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

    /// Optimistically reflect the change locally, then queue it for the engine.
    pub fn set_sleep(&self, active: bool) {
        self.shared.lock().unwrap().state.sleep_active = active;
        let _ = self.tx.send(Cmd::SetSleep(active));
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
                Cmd::SetSleep(a) => c.set_sleep(a),
                Cmd::SetConfig(cfg) => c.set_config(cfg),
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
