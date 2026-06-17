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
    spawn_status_automation(engine.clone(), api.clone());
    spawn_friends_refresh(engine, api);
}

// ---- friends cache (for join notifications' "friends only" filter) ---------

/// Keep the engine's friend set fresh while signed in (re-fetched every ~6 h,
/// cleared on sign-out). Grabs the client+cookie under a brief lock, then fetches
/// lock-free (the list can be large), like the friends picker does.
fn spawn_friends_refresh(engine: Arc<Mutex<Engine>>, api: SharedApi) {
    std::thread::spawn(move || {
        let mut last_fetch: Option<std::time::Instant> = None;
        loop {
            let logged_in = api.lock().unwrap().login_status().logged_in;
            if !logged_in {
                if last_fetch.take().is_some() {
                    engine.lock().unwrap().set_friends(None);
                }
            } else if last_fetch.is_none_or(|t| t.elapsed() > Duration::from_secs(6 * 60 * 60)) {
                let req = api.lock().unwrap().friends_request();
                if let Some((client, cookie)) = req {
                    let list = crate::vrchat_api::fetch_friends(&client, &cookie);
                    engine.lock().unwrap().set_friends(Some(list));
                    last_fetch = Some(std::time::Instant::now());
                }
            }
            std::thread::sleep(Duration::from_secs(60));
        }
    });
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
    // Friendlier than the user id when VRChat includes it.
    let name = notif.get("senderUsername").and_then(Value::as_str).unwrap_or(sender);
    maybe_accept(engine, api, id, sender, name);
}

fn maybe_accept(engine: &Arc<Mutex<Engine>>, api: &SharedApi, notif_id: &str, sender: &str, sender_name: &str) {
    // Evaluate conditions against a quick snapshot, then drop the engine lock
    // before any network call.
    enum Action {
        Accept { instance: String, message_slot: Option<u32> },
        Decline { slot: u32 },
        Ignore,
    }

    let action = {
        let g = engine.lock().unwrap();
        let aa = &g.config.vrchat.auto_accept;
        // The feature is "active" (handling requests) only when enabled and, if
        // gated, while sleeping. When inactive we leave requests to the user.
        if !aa.enabled || (aa.only_when_sleep && !g.state.sleep_phase.is_active()) {
            Action::Ignore
        } else {
            let listed = aa.player_ids.iter().any(|id| id == sender);
            let listed_ok = match aa.list_mode {
                ListMode::Whitelist => listed,
                ListMode::Blacklist => !listed,
            };
            let under_limit = !aa.max_players_enabled || g.state.player_count < aa.max_players;
            if listed_ok && under_limit {
                match g.vrchat_instance.clone() {
                    Some(instance) => {
                        let slot = aa.invite_message_enabled.then_some(aa.invite_message_slot);
                        Action::Accept { instance, message_slot: slot }
                    }
                    None => {
                        log::warn!("auto-accept: no current instance known yet; skipping");
                        Action::Ignore
                    }
                }
            } else if aa.decline_message_enabled {
                Action::Decline { slot: aa.decline_message_slot }
            } else {
                Action::Ignore
            }
        }
    };

    // Do the network action, then drop the api lock before touching the engine.
    let notice = {
        let mut a = api.lock().unwrap();
        match action {
            Action::Accept { instance, message_slot } => {
                if a.invite_user(sender, &instance, message_slot) {
                    a.hide_notification(notif_id);
                    log::info!("auto-accepted invite request from {sender}");
                    Some(format!("Accepted invite from {sender_name}"))
                } else {
                    None
                }
            }
            Action::Decline { slot } => {
                if a.respond_invite(notif_id, slot) {
                    a.hide_notification(notif_id);
                    log::info!("declined invite request from {sender} with a message");
                    Some(format!("Declined invite from {sender_name}"))
                } else {
                    None
                }
            }
            Action::Ignore => None,
        }
    };
    if let Some(msg) = notice {
        engine.lock().unwrap().notify(msg);
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
                        let ok = api.lock().unwrap().set_status(status);
                        if ok {
                            log::info!("status automation -> {status}");
                            engine.lock().unwrap().notify(format!("Status → {status}"));
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
