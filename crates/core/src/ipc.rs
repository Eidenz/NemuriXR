// IPC between the always-on overlay/core (server) and the desktop app (client).
//
// Transport: a Unix domain socket carrying newline-delimited JSON. Each request
// gets exactly one response on the same (persistent) connection, so the desktop
// can hold a connection open and poll `GetState` cheaply.
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::state::{SleepPhase, SleepPosition, State};

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Request {
    /// Fetch the persisted config.
    GetConfig,
    /// Replace the persisted config (server saves it and applies live).
    SetConfig { config: Config },
    /// Fetch the live runtime state.
    GetState,
    /// Command the sleep phase (Awake / Prepare / Sleep).
    SetPhase { phase: SleepPhase },
    /// Report the current physical lying position (drives the sleeping-pose OSC).
    SetSleepingPosition { position: SleepPosition },
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Response {
    Config { config: Config },
    State { state: State },
    Ok,
    Error { message: String },
}

/// `$XDG_RUNTIME_DIR/nemurixr.sock`, falling back to `/tmp/nemurixr-<uid>.sock`.
pub fn socket_path() -> String {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        if !dir.is_empty() {
            return format!("{dir}/nemurixr.sock");
        }
    }
    let uid = unsafe { libc_getuid() };
    format!("/tmp/nemurixr-{uid}.sock")
}

// Avoid pulling in the `libc` crate just for getuid().
extern "C" {
    #[link_name = "getuid"]
    fn libc_getuid() -> u32;
}

/// Start the IPC server on a background thread. `handler` answers each request
/// using the server's shared state. Returns once the listener is bound.
pub fn serve<F>(handler: F) -> Result<()>
where
    F: Fn(Request) -> Response + Send + Sync + 'static,
{
    let path = socket_path();
    let _ = std::fs::remove_file(&path); // clear a stale socket from a prior run
    let listener = UnixListener::bind(&path)?;
    log::info!("IPC server listening on {path}");
    let handler = Arc::new(handler);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let h = handler.clone();
                    std::thread::spawn(move || {
                        if let Err(e) = handle_conn(stream, h) {
                            log::debug!("IPC connection closed: {e}");
                        }
                    });
                }
                Err(e) => log::warn!("IPC accept error: {e}"),
            }
        }
    });
    Ok(())
}

fn handle_conn<F>(stream: UnixStream, handler: Arc<F>) -> Result<()>
where
    F: Fn(Request) -> Response,
{
    let mut writer = stream.try_clone()?;
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let resp = match serde_json::from_str::<Request>(&line) {
            Ok(req) => handler(req),
            Err(e) => Response::Error { message: format!("bad request: {e}") },
        };
        let mut out = serde_json::to_string(&resp)?;
        out.push('\n');
        writer.write_all(out.as_bytes())?;
        writer.flush()?;
    }
    Ok(())
}

/// A blocking IPC client (used by the desktop app's Tauri backend).
pub struct Client {
    stream: UnixStream,
    reader: BufReader<UnixStream>,
}

impl Client {
    /// Connect to a running overlay/core. Errors if it isn't running.
    pub fn connect() -> Result<Self> {
        let stream = UnixStream::connect(socket_path())?;
        let reader = BufReader::new(stream.try_clone()?);
        Ok(Self { stream, reader })
    }

    pub fn request(&mut self, req: &Request) -> Result<Response> {
        let mut line = serde_json::to_string(req)?;
        line.push('\n');
        self.stream.write_all(line.as_bytes())?;
        self.stream.flush()?;
        let mut buf = String::new();
        let n = self.reader.read_line(&mut buf)?;
        if n == 0 {
            anyhow::bail!("server closed the connection");
        }
        Ok(serde_json::from_str(&buf)?)
    }

    pub fn get_config(&mut self) -> Result<Config> {
        match self.request(&Request::GetConfig)? {
            Response::Config { config } => Ok(config),
            Response::Error { message } => anyhow::bail!(message),
            _ => anyhow::bail!("unexpected response"),
        }
    }

    pub fn set_config(&mut self, config: Config) -> Result<()> {
        match self.request(&Request::SetConfig { config })? {
            Response::Ok => Ok(()),
            Response::Error { message } => anyhow::bail!(message),
            _ => anyhow::bail!("unexpected response"),
        }
    }

    pub fn get_state(&mut self) -> Result<State> {
        match self.request(&Request::GetState)? {
            Response::State { state } => Ok(state),
            Response::Error { message } => anyhow::bail!(message),
            _ => anyhow::bail!("unexpected response"),
        }
    }

    pub fn set_phase(&mut self, phase: SleepPhase) -> Result<()> {
        match self.request(&Request::SetPhase { phase })? {
            Response::Ok => Ok(()),
            Response::Error { message } => anyhow::bail!(message),
            _ => anyhow::bail!("unexpected response"),
        }
    }

    pub fn set_sleeping_position(&mut self, position: SleepPosition) -> Result<()> {
        match self.request(&Request::SetSleepingPosition { position })? {
            Response::Ok => Ok(()),
            Response::Error { message } => anyhow::bail!(message),
            _ => anyhow::bail!("unexpected response"),
        }
    }
}
