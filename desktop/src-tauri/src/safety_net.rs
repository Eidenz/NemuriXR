// Auto-sleep safety net: when you doze off (motion detection triggers sleep),
// optionally mute your mic and pose your avatar — but only if you didn't set
// yourself up (no FBT trackers, not already in a GoGo pose). A manual sleep
// never reaches here.
//
// Runs on its own thread because the checks are slow-ish (libmonado +
// OSCQuery HTTP). All of it is best-effort; failures just mean an action is
// skipped. What it did is recorded on the Engine so waking can revert it.
use std::sync::{Arc, Mutex};

use crate::Engine;

pub fn run(engine: Arc<Mutex<Engine>>) {
    std::thread::spawn(move || orchestrate(&engine));
}

fn orchestrate(engine: &Arc<Mutex<Engine>>) {
    let (sn, osc_cfg, osc_target) = {
        let g = engine.lock().unwrap();
        (g.config.safety_net.clone(), g.config.osc.clone(), g.osc_target)
    };
    if !sn.enabled {
        return;
    }
    log::info!("auto-sleep safety net: evaluating");
    let mut acted = false;

    // 1. Mute the mic device.
    if sn.mute_device {
        crate::audio::set_mic_muted(true);
        engine.lock().unwrap().safety_net_muted_device = true;
        acted = true;
    }

    // OSCQuery is only needed to read mute state / current pose.
    let need_query = sn.mute_ingame || (sn.pose && !sn.pose_override_existing);
    let http = if need_query { crate::oscquery::discover() } else { None };

    // 2. Mute in-game (skip if already muted, so we don't toggle you on).
    if sn.mute_ingame {
        let already_muted = http.and_then(crate::oscquery::mic_muted) == Some(true);
        if !already_muted {
            if let Some(t) = crate::osc::resolve_target(&osc_cfg, osc_target) {
                crate::osc::voice_toggle(t);
                engine.lock().unwrap().safety_net_muted_ingame = true;
                acted = true;
            }
        }
    }

    // 3. Fallback pose — unless trackers are on or you're already posed.
    if sn.pose {
        let skip_trackers = sn.pose_skip_if_trackers && crate::trackers::present();
        let already_posed =
            !sn.pose_override_existing && http.and_then(crate::oscquery::in_gogo_pose).unwrap_or(false);
        if skip_trackers {
            log::info!("safety net: FBT trackers connected — leaving the pose to you");
        } else if already_posed {
            log::info!("safety net: already in a GoGo pose — leaving it");
        } else {
            // Turn the pose system on for this sleep and apply the last known
            // direction immediately (the overlay only re-reports on change).
            let mut g = engine.lock().unwrap();
            g.safety_net_pose = true;
            let pos = g.last_position;
            g.apply_sleeping_pose(pos);
            acted = true;
            log::info!("safety net: posing your avatar");
        }
    }

    if acted {
        engine.lock().unwrap().safety_net_acted = true;
    }
}
