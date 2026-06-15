// VRChat automation engine: auto-accept invite requests + status automations.
//
// Anti-spam by design: we do NOT poll the VRChat API for state. Invite requests
// arrive in real time over the pipeline websocket; the current world / instance /
// player count come from the log watcher. The only API calls are the actions
// themselves (invite, hide notification, set status), each rate-limited.
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::Value;
use tungstenite::client::IntoClientRequest;
use tungstenite::http::header::USER_AGENT;
use tungstenite::http::HeaderValue;
use tungstenite::Message;

use nemurixr_core::config::{ListMode, VrcStatus};

use crate::vrchat_api::SharedApi;
use crate::Engine;

const UA: &str = "NemuriXR/0.1.0 (https://github.com/eidenz/nemurixr)";

pub fn spawn(engine: Arc<Mutex<Engine>>, api: SharedApi) {
    spawn_pipeline(engine.clone(), api.clone());
    spawn_status_automation(engine, api);
}

// ---- pipeline websocket → auto-accept --------------------------------------

fn spawn_pipeline(engine: Arc<Mutex<Engine>>, api: SharedApi) {
    std::thread::spawn(move || loop {
        let Some(token) = api.lock().unwrap().auth_token() else {
            // Not logged in yet; wait and re-check.
            std::thread::sleep(Duration::from_secs(10));
            continue;
        };
        let url = format!("wss://pipeline.vrchat.cloud/?authToken={token}");
        let req = match url.as_str().into_client_request() {
            Ok(mut r) => {
                r.headers_mut().insert(USER_AGENT, HeaderValue::from_static(UA));
                r
            }
            Err(e) => {
                log::warn!("VRChat pipeline bad request: {e}");
                std::thread::sleep(Duration::from_secs(10));
                continue;
            }
        };
        match tungstenite::connect(req) {
            Ok((mut socket, _)) => {
                log::info!("VRChat pipeline connected");
                loop {
                    match socket.read() {
                        Ok(Message::Text(txt)) => handle_message(txt.as_str(), &engine, &api),
                        Ok(Message::Close(_)) => break,
                        Ok(_) => {} // ping/pong/binary — tungstenite auto-pongs
                        Err(e) => {
                            log::debug!("VRChat pipeline read ended: {e}");
                            break;
                        }
                    }
                }
                log::info!("VRChat pipeline disconnected; will reconnect");
            }
            Err(e) => log::warn!("VRChat pipeline connect failed: {e}"),
        }
        std::thread::sleep(Duration::from_secs(5));
    });
}

fn handle_message(text: &str, engine: &Arc<Mutex<Engine>>, api: &SharedApi) {
    let Ok(msg) = serde_json::from_str::<Value>(text) else { return };
    if msg.get("type").and_then(Value::as_str) != Some("notification") {
        return;
    }
    // `content` is a JSON-encoded string of the notification object.
    let content = msg.get("content").and_then(Value::as_str).unwrap_or("");
    let Ok(notif) = serde_json::from_str::<Value>(content) else { return };
    if notif.get("type").and_then(Value::as_str) != Some("requestInvite") {
        return;
    }
    let (Some(id), Some(sender)) =
        (notif.get("id").and_then(Value::as_str), notif.get("senderUserId").and_then(Value::as_str))
    else {
        return;
    };
    maybe_accept(engine, api, id, sender);
}

fn maybe_accept(engine: &Arc<Mutex<Engine>>, api: &SharedApi, notif_id: &str, sender: &str) {
    // Evaluate conditions against a quick snapshot, then drop the engine lock
    // before any network call.
    let (instance, message_slot) = {
        let g = engine.lock().unwrap();
        let aa = &g.config.vrchat.auto_accept;
        if !aa.enabled {
            return;
        }
        if aa.only_when_sleep && !g.state.sleep_phase.is_active() {
            return;
        }
        if aa.max_players_enabled && g.state.player_count >= aa.max_players {
            log::info!("auto-accept: world at/over player limit, ignoring request from {sender}");
            return;
        }
        let listed = aa.player_ids.iter().any(|id| id == sender);
        let allowed = match aa.list_mode {
            ListMode::Whitelist => listed,
            ListMode::Blacklist => !listed,
        };
        if !allowed {
            return;
        }
        let slot = aa.invite_message_enabled.then_some(aa.invite_message_slot);
        (g.vrchat_instance.clone(), slot)
    };
    let Some(instance) = instance else {
        log::warn!("auto-accept: no current instance known yet; skipping");
        return;
    };
    let mut a = api.lock().unwrap();
    if a.invite_user(sender, &instance, message_slot) {
        a.hide_notification(notif_id);
        log::info!("auto-accepted invite request from {sender}");
    }
}

// ---- status automation (player-count crossing) -----------------------------

enum Eval {
    Inactive,
    Active { above: bool, below: VrcStatus, at_or_above: VrcStatus },
}

fn spawn_status_automation(engine: Arc<Mutex<Engine>>, api: SharedApi) {
    std::thread::spawn(move || {
        let mut prev_above: Option<bool> = None;
        loop {
            std::thread::sleep(Duration::from_secs(3));
            let eval = {
                let g = engine.lock().unwrap();
                let sa = &g.config.vrchat.status_automations;
                if !sa.enabled
                    || g.state.vrchat_world.is_none()
                    || (sa.only_when_sleep && !g.state.sleep_phase.is_active())
                {
                    Eval::Inactive
                } else {
                    Eval::Active {
                        above: g.state.player_count >= sa.player_limit,
                        below: sa.below_status,
                        at_or_above: sa.at_or_above_status,
                    }
                }
            };
            match eval {
                Eval::Inactive => prev_above = None, // re-apply when it becomes active again
                Eval::Active { above, below, at_or_above } => {
                    if prev_above != Some(above) {
                        prev_above = Some(above);
                        let status = vrc_status_str(if above { at_or_above } else { below });
                        if api.lock().unwrap().set_status(status) {
                            log::info!("status automation -> {status}");
                        }
                    }
                }
            }
        }
    });
}

fn vrc_status_str(s: VrcStatus) -> &'static str {
    match s {
        VrcStatus::JoinMe => "join me",
        VrcStatus::Active => "active",
        VrcStatus::AskMe => "ask me",
        VrcStatus::Busy => "busy",
    }
}
