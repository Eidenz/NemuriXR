// Read VRChat avatar parameter values over OSCQuery.
//
// VRChat advertises an OSCQuery HTTP service (mDNS `_oscjson._tcp`) that serves
// each address "as well as their values (for readable value endpoints)". We GET
// the node for a parameter and read its current value — used to tell whether
// you've locked yourself in place, and your in-game mute state.
//
// All of this is best-effort: any failure (VRChat closed, OSC off, format quirk)
// returns None and the caller falls back sensibly.
use std::net::{Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};

use mdns_sd::{ServiceDaemon, ServiceEvent};
use serde_json::Value;

const SERVICE: &str = "_oscjson._tcp.local.";

/// Find VRChat's OSCQuery HTTP endpoint (VRChat is local, so loopback + the
/// advertised port). Returns None within ~3s if not found.
pub fn discover() -> Option<SocketAddr> {
    let mdns = ServiceDaemon::new().ok()?;
    let rx = mdns.browse(SERVICE).ok()?;
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return None;
        }
        match rx.recv_timeout(remaining) {
            Ok(ServiceEvent::ServiceResolved(info)) => {
                if info.get_fullname().to_lowercase().contains("vrchat") {
                    return Some(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), info.get_port()));
                }
            }
            Ok(_) => {}
            Err(_) => return None,
        }
    }
}

fn get_node(http: SocketAddr, path: &str) -> Option<Value> {
    let url = format!("http://{http}{path}");
    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(3)).build().ok()?;
    let resp = client.get(&url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json().ok()
}

fn value0(node: &Value) -> Option<&Value> {
    node.get("VALUE").and_then(|v| v.as_array()).and_then(|a| a.first())
}

/// Read a boolean node value, handling both `VALUE:[true]` and OSC's
/// bool-as-type-tag (`TYPE:"T"/"F"`).
fn node_bool(node: &Value) -> Option<bool> {
    if let Some(b) = value0(node).and_then(Value::as_bool) {
        return Some(b);
    }
    match node.get("TYPE").and_then(Value::as_str) {
        Some("T") => Some(true),
        Some("F") => Some(false),
        _ => None,
    }
}

/// Have you locked yourself in place? GoGo Loco's `Go/Stationary` (the
/// feet/station anchor, set by the foot-lock preset) is a HELD boolean — true
/// while you're anchored, e.g. once you've settled into a pose. The safety net
/// reads it to tell "I set myself up, leave me be" from "I dozed off and might
/// spin on a joystick". Unlike the pose emotes it does NOT pulse back to 0, so
/// it's still readable the 15+ minutes later that motion detection fires.
pub fn position_locked(http: SocketAddr) -> Option<bool> {
    let node = get_node(http, "/avatar/parameters/Go/Stationary")?;
    node_bool(&node)
}

/// Your current in-game mic mute state (the `MuteSelf` parameter), if readable.
pub fn mic_muted(http: SocketAddr) -> Option<bool> {
    let node = get_node(http, "/avatar/parameters/MuteSelf")?;
    node_bool(&node)
}
