// VRChat log watcher. Tails VRChat's output log (works whether VRChat runs in VR
// or desktop mode) to track the current world + who's present, drives the live
// player count, and fires join/leave notification sounds. No VRChat login needed.
//
// Notes from real Linux/Proton logs:
//  - VRChat logs the LOCAL player as `OnPlayerJoined` too, so the player set
//    already includes you (count = set size); self must be excluded from sounds.
//  - The local user is identified from `User Authenticated: <name> (usr_…)` and
//    `Initialized PlayerAPI "<name>" is local`.
//  - Joining a world dumps everyone already present as `OnPlayerJoined` (seen ~9s
//    after the room join), so we suppress sounds for a grace period after a room
//    join to avoid ringing for that burst.
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::sound;
use crate::Engine;

const POLL: Duration = Duration::from_millis(1500);
const JOIN_GRACE: Duration = Duration::from_secs(15);

pub fn spawn(engine: Arc<Mutex<Engine>>) {
    std::thread::spawn(move || watch_loop(engine));
}

fn log_dir(engine: &Arc<Mutex<Engine>>) -> Option<PathBuf> {
    let override_dir = engine.lock().unwrap().vrchat_log_dir();
    if !override_dir.is_empty() {
        let p = PathBuf::from(override_dir);
        if p.is_dir() {
            return Some(p);
        }
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let user = std::env::var("USER").unwrap_or_default();
    let candidates = [
        format!("{home}/.steam/steam/steamapps/compatdata/438100/pfx/drive_c/users/steamuser/AppData/LocalLow/VRChat/VRChat"),
        format!("{home}/.local/share/Steam/steamapps/compatdata/438100/pfx/drive_c/users/steamuser/AppData/LocalLow/VRChat/VRChat"),
        format!("{home}/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/compatdata/438100/pfx/drive_c/users/steamuser/AppData/LocalLow/VRChat/VRChat"),
        format!("{home}/.wine/drive_c/users/{user}/AppData/LocalLow/VRChat/VRChat"),
    ];
    candidates.into_iter().map(PathBuf::from).find(|p| p.is_dir())
}

fn newest_log(dir: &Path) -> Option<PathBuf> {
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let p = entry.path();
        let is_log = p
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("output_log_") && n.ends_with(".txt"));
        if !is_log {
            continue;
        }
        if let Ok(mt) = entry.metadata().and_then(|m| m.modified()) {
            if best.as_ref().is_none_or(|(bt, _)| mt > *bt) {
                best = Some((mt, p));
            }
        }
    }
    best.map(|(_, p)| p)
}

/// A live join/leave that should be considered for a notification sound.
struct Notify {
    is_join: bool,
    /// "were you alone before this join" / "are you alone after this leave".
    alone: bool,
}

#[derive(Default)]
struct Watcher {
    world: Option<String>,
    instance: Option<String>, // full location "wrld_…:…" (for inviting people in)
    players: HashSet<String>, // includes the local player (VRChat logs self too)
    local_id: Option<String>,
    local_name: Option<String>,
    suppress_until: Option<Instant>,
    live: bool, // false during the initial catch-up read of a file (no sounds)
}

impl Watcher {
    fn reset_session(&mut self) {
        *self = Watcher::default();
    }

    fn is_self(&self, key: &str) -> bool {
        self.local_id.as_deref() == Some(key) || self.local_name.as_deref() == Some(key)
    }

    /// Other players present (excluding you).
    fn remote_count(&self) -> usize {
        self.players.iter().filter(|k| !self.is_self(k)).count()
    }

    fn player_count(&self) -> u32 {
        if self.world.is_some() {
            self.players.len() as u32
        } else {
            0
        }
    }

    fn suppressed(&self) -> bool {
        self.suppress_until.is_some_and(|t| Instant::now() < t)
    }

    fn set_local(&mut self, s: &str) {
        let key = player_key(s);
        if key.starts_with("usr_") {
            self.local_id = Some(key);
        }
        let name = match s.find(" (") {
            Some(p) => &s[..p],
            None => s,
        };
        self.local_name = Some(name.trim().to_string());
    }

    /// Update state from one log line; return a live notification if warranted.
    fn process_line(&mut self, line: &str) -> Option<Notify> {
        // Local user identification (outside the [Behaviour] block).
        if let Some(i) = line.find("User Authenticated: ") {
            self.set_local(line[i + "User Authenticated: ".len()..].trim());
            return None;
        }
        let idx = line.find("[Behaviour] ")?;
        let rest = &line[idx + "[Behaviour] ".len()..];

        if let Some(after) = rest.strip_prefix("Initialized PlayerAPI \"") {
            if let Some(end) = after.find('"') {
                if after[end..].contains("is local") {
                    self.local_name = Some(after[..end].to_string());
                }
            }
            return None;
        }

        if let Some(arg) = rest.strip_prefix("OnPlayerJoined ") {
            let key = player_key(arg);
            if key.is_empty() {
                return None;
            }
            let was_alone = self.remote_count() == 0;
            let is_new = self.players.insert(key.clone());
            if is_new && self.live && !self.is_self(&key) && !self.suppressed() {
                return Some(Notify { is_join: true, alone: was_alone });
            }
        } else if rest.starts_with("OnPlayerLeft ") && !rest.starts_with("OnPlayerLeftRoom") {
            let key = player_key(&rest["OnPlayerLeft ".len()..]);
            let removed = self.players.remove(&key);
            if removed && self.live && !self.is_self(&key) && !self.suppressed() {
                let now_alone = self.remote_count() == 0;
                return Some(Notify { is_join: false, alone: now_alone });
            }
        } else if let Some(room) = rest.strip_prefix("Joining or Creating Room: ") {
            self.world = Some(room.trim().to_string());
            self.players.clear();
            self.suppress_until = Some(Instant::now() + JOIN_GRACE);
        } else if let Some(loc) = rest.strip_prefix("Joining ") {
            // "Joining wrld_xxx:12345~region(eu)" — the full instance location.
            if loc.starts_with("wrld_") {
                self.instance = loc.split_whitespace().next().map(str::to_string);
            }
        } else if rest.starts_with("OnLeftRoom") {
            self.world = None;
            self.instance = None;
            self.players.clear();
        }
        None
    }
}

fn watch_loop(engine: Arc<Mutex<Engine>>) {
    let mut current: Option<PathBuf> = None;
    let mut offset: u64 = 0;
    let mut w = Watcher::default();

    loop {
        let newest = log_dir(&engine).as_deref().and_then(newest_log);
        if newest != current {
            current = newest;
            offset = 0;
            w.reset_session();
            publish(&engine, &w);
        }
        if let Some(path) = current.clone() {
            read_new(&path, &mut offset, &mut w, &engine);
            publish(&engine, &w);
        }
        std::thread::sleep(POLL);
    }
}

/// Read appended log lines since `offset`, processing complete lines only.
fn read_new(path: &Path, offset: &mut u64, w: &mut Watcher, engine: &Arc<Mutex<Engine>>) {
    let Ok(file) = std::fs::File::open(path) else { return };
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    if len < *offset {
        *offset = 0;
        w.reset_session();
    }
    let mut reader = BufReader::new(file);
    if reader.seek(SeekFrom::Start(*offset)).is_err() {
        return;
    }
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(n) => {
                if !line.ends_with('\n') {
                    break; // partial trailing line — wait for the rest
                }
                *offset += n as u64;
                if let Some(ev) = w.process_line(line.trim_end()) {
                    if let Some(snd) = engine.lock().unwrap().join_notify_sound(ev.is_join, ev.alone) {
                        sound::play_notification(if ev.is_join { "join" } else { "leave" }, &snd);
                    }
                }
            }
            Err(_) => break,
        }
    }
    w.live = true; // caught up; later lines are live
}

/// `DisplayName (usr_xxxx)` → the usr id; otherwise the trimmed name.
fn player_key(s: &str) -> String {
    let s = s.trim();
    if let (Some(a), Some(b)) = (s.rfind('('), s.rfind(')')) {
        if a < b {
            let inside = s[a + 1..b].trim();
            if inside.starts_with("usr_") {
                return inside.to_string();
            }
        }
    }
    s.to_string()
}

fn publish(engine: &Arc<Mutex<Engine>>, w: &Watcher) {
    engine.lock().unwrap().set_vrchat(w.world.clone(), w.instance.clone(), w.player_count());
}

#[cfg(test)]
mod tests {
    use super::*;

    // Lines taken verbatim (timestamps trimmed) from a real Linux/Proton log.
    #[test]
    fn tracks_world_and_self_inclusive_count() {
        let mut w = Watcher::default();
        w.live = true;
        for l in [
            r#"2026.06.15 12:43:51 Debug      -  User Authenticated: Eidenz (usr_f412eaff-59b7-43d5-8637-c538ecb786f3)"#,
            r#"2026.06.15 12:44:08 Debug      -  [Behaviour] Initialized PlayerAPI "Eidenz" is local"#,
            r#"2026.06.15 12:43:56 Debug      -  [Behaviour] Joining or Creating Room: Thad Tiny Library"#,
            r#"2026.06.15 12:44:08 Debug      -  [Behaviour] OnPlayerJoined Eidenz (usr_f412eaff-59b7-43d5-8637-c538ecb786f3)"#,
        ] {
            w.process_line(l);
        }
        assert_eq!(w.world.as_deref(), Some("Thad Tiny Library"));
        assert_eq!(w.player_count(), 1, "you count as one");
        assert_eq!(w.remote_count(), 0, "you are not a remote player");
        assert!(w.is_self("usr_f412eaff-59b7-43d5-8637-c538ecb786f3"));

        // Switch worlds: leave clears, new room + two joins (self + remote).
        for l in [
            r#"... -  [Behaviour] OnLeftRoom"#,
            r#"... -  [Behaviour] OnPlayerLeft Eidenz (usr_f412eaff-59b7-43d5-8637-c538ecb786f3)"#,
            r#"... -  [Behaviour] Joining or Creating Room: Deep Sleep"#,
            r#"... -  [Behaviour] OnPlayerJoined mitsukaki (usr_8c785f7b-5098-482f-a1c0-7cdf060f7dfc)"#,
            r#"... -  [Behaviour] OnPlayerJoined Eidenz (usr_f412eaff-59b7-43d5-8637-c538ecb786f3)"#,
        ] {
            w.process_line(l);
        }
        assert_eq!(w.world.as_deref(), Some("Deep Sleep"));
        assert_eq!(w.player_count(), 2, "you + mitsukaki");
        assert_eq!(w.remote_count(), 1, "just mitsukaki");
    }

    #[test]
    fn live_remote_join_notifies_but_self_does_not() {
        let mut w = Watcher::default();
        w.live = true;
        w.process_line(r#"x User Authenticated: Me (usr_me)"#);
        w.process_line(r#"x [Behaviour] Joining or Creating Room: Home"#);
        // Past the join grace so live events aren't suppressed.
        w.suppress_until = None;
        // Self join: no notification.
        assert!(w.process_line(r#"x [Behaviour] OnPlayerJoined Me (usr_me)"#).is_none());
        // Remote join while alone: notify (join, alone=true).
        let ev = w.process_line(r#"x [Behaviour] OnPlayerJoined Friend (usr_friend)"#).expect("notify");
        assert!(ev.is_join && ev.alone);
        // Remote leave -> alone again: notify (leave, alone=true).
        let ev = w.process_line(r#"x [Behaviour] OnPlayerLeft Friend (usr_friend)"#).expect("notify");
        assert!(!ev.is_join && ev.alone);
        assert_eq!(w.player_count(), 1);
    }
}
