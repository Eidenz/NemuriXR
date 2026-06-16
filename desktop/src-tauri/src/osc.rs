// OSC automations: send a sequence of OSC messages to VRChat on sleep/wake.
//
//  - Encoding/transport: `rosc` over UDP.
//  - Target: discovered via OSCQuery (mDNS `_osc._udp`) when enabled, else the
//    configured host:port (default 127.0.0.1:9000). VRChat is local, so we use
//    the discovered PORT with the loopback address.
//  - Each message carries a `delay_ms` so a list can be sequenced over time.
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use mdns_sd::{ServiceDaemon, ServiceEvent};
use rosc::{OscMessage, OscPacket, OscType};

use nemurixr_core::config::{OscArg, OscConfig, OscMessage as CfgMsg};

use crate::Engine;

const OSC_SERVICE: &str = "_osc._udp.local.";

/// Background mDNS discovery of VRChat's OSC input port. Updates the engine's
/// resolved target as the service appears/disappears.
pub fn spawn_discovery(engine: Arc<Mutex<Engine>>) {
    std::thread::spawn(move || loop {
        discover(&engine);
        // discover() only returns if the mDNS daemon dies; back off and retry.
        std::thread::sleep(Duration::from_secs(5));
    });
}

fn discover(engine: &Arc<Mutex<Engine>>) {
    let mdns = match ServiceDaemon::new() {
        Ok(d) => d,
        Err(e) => {
            log::warn!("OSC mDNS init failed: {e}");
            return;
        }
    };
    let rx = match mdns.browse(OSC_SERVICE) {
        Ok(rx) => rx,
        Err(e) => {
            log::warn!("OSC mDNS browse failed: {e}");
            return;
        }
    };
    log::info!("OSCQuery: browsing for VRChat ({OSC_SERVICE})");
    while let Ok(event) = rx.recv() {
        match event {
            ServiceEvent::ServiceResolved(info) => {
                if info.get_fullname().to_lowercase().contains("vrchat") {
                    // VRChat is local; use the discovered port on loopback.
                    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), info.get_port());
                    log::info!("OSCQuery: found VRChat OSC at {addr}");
                    engine.lock().unwrap().set_osc_target(Some(addr));
                }
            }
            ServiceEvent::ServiceRemoved(_, fullname) => {
                if fullname.to_lowercase().contains("vrchat") {
                    log::info!("OSCQuery: VRChat OSC went away");
                    engine.lock().unwrap().set_osc_target(None);
                }
            }
            _ => {}
        }
    }
}

/// Resolve where to send: discovered target (if OSCQuery on + found), else the
/// configured host:port.
pub fn resolve_target(cfg: &OscConfig, discovered: Option<SocketAddr>) -> Option<SocketAddr> {
    if cfg.use_oscquery {
        if let Some(d) = discovered {
            return Some(d);
        }
    }
    use std::net::ToSocketAddrs;
    (cfg.host.as_str(), cfg.port).to_socket_addrs().ok().and_then(|mut it| it.next())
}

/// Send a message list to `target` on a background thread, honouring per-message
/// delays (so a transition never blocks on the sequence).
pub fn send_sequence(target: SocketAddr, messages: Vec<CfgMsg>) {
    if messages.is_empty() {
        return;
    }
    std::thread::spawn(move || {
        let socket = match UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("OSC: can't open UDP socket: {e}");
                return;
            }
        };
        for m in messages {
            if m.delay_ms > 0 {
                std::thread::sleep(Duration::from_millis(m.delay_ms as u64));
            }
            let args: Vec<OscType> = m.args.iter().map(to_osc).collect();
            let packet = OscPacket::Message(OscMessage { addr: m.address.clone(), args });
            match rosc::encoder::encode(&packet) {
                Ok(buf) => {
                    if let Err(e) = socket.send_to(&buf, target) {
                        log::warn!("OSC send {} -> {target}: {e}", m.address);
                    }
                }
                Err(e) => log::warn!("OSC encode {}: {e}", m.address),
            }
        }
    });
}

/// Press + release VRChat's Voice input — toggles mic mute (in toggle-voice
/// mode). Used by the safety net's in-game mute.
pub fn voice_toggle(target: SocketAddr) {
    send_sequence(
        target,
        vec![
            CfgMsg { address: "/input/Voice".to_string(), args: vec![OscArg::Int(1)], delay_ms: 0 },
            CfgMsg { address: "/input/Voice".to_string(), args: vec![OscArg::Int(0)], delay_ms: 150 },
        ],
    );
}

fn to_osc(a: &OscArg) -> OscType {
    match a {
        OscArg::Bool(b) => OscType::Bool(*b),
        OscArg::Int(i) => OscType::Int(*i),
        OscArg::Float(f) => OscType::Float(*f),
        OscArg::String(s) => OscType::String(s.clone()),
    }
}
