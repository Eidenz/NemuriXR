//! Shared model + IPC for NemuriXR, used by both the overlay/core and the
//! desktop configuration app.
//!
//! - [`config`] — the persisted settings model (single source of truth).
//! - [`state`] — a live runtime snapshot the overlay publishes.
//! - [`ipc`] — Unix-socket protocol (server `serve` + `Client`).

pub mod config;
pub mod ipc;
pub mod state;

pub use config::Config;
pub use state::{SleepPhase, SleepPosition, State};
