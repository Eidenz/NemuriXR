// Detect connected FBT trackers via libmonado (same approach WayVR uses for its
// battery panel). Monado device `name_id` 4..=8 are generic trackers; a tracker
// that's off or has a dead battery simply isn't enumerated, so "no trackers"
// covers both cases.
use libmonado::Monado;

pub fn present() -> bool {
    let Ok(monado) = Monado::auto_connect() else {
        return false;
    };
    let Ok(devices) = monado.devices() else {
        return false;
    };
    // Bind to a local so the borrowing iterator drops before `monado`.
    let present = devices.into_iter().any(|d| (4..=8).contains(&d.name_id));
    present
}
