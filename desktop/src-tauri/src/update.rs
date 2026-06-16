// Update check: ask GitHub for the latest release and compare to our version.
// Best-effort and quiet — any failure (offline, rate-limited) just yields None.
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

const REPO: &str = "Eidenz/NemuriXR";

#[derive(Serialize, Clone)]
pub struct UpdateInfo {
    /// The newer version (without a leading "v").
    pub version: String,
    /// The release page to open.
    pub url: String,
}

/// Returns `Some` when a newer non-prerelease GitHub release exists.
pub fn check() -> Option<UpdateInfo> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("NemuriXR/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?;
    let resp = client
        .get(format!("https://api.github.com/repos/{REPO}/releases/latest"))
        .header("Accept", "application/vnd.github+json")
        .send()
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: Value = resp.json().ok()?;
    if body.get("prerelease").and_then(Value::as_bool).unwrap_or(false)
        || body.get("draft").and_then(Value::as_bool).unwrap_or(false)
    {
        return None;
    }
    let latest = body.get("tag_name").and_then(Value::as_str)?.trim_start_matches('v').to_string();
    if is_newer(&latest, env!("CARGO_PKG_VERSION")) {
        let url = body.get("html_url").and_then(Value::as_str).unwrap_or("").to_string();
        Some(UpdateInfo { version: latest, url })
    } else {
        None
    }
}

/// Numeric x.y.z comparison; pre-release/build suffixes are ignored.
fn is_newer(a: &str, b: &str) -> bool {
    semver(a) > semver(b)
}

fn semver(v: &str) -> (u32, u32, u32) {
    let core = v.split(['-', '+']).next().unwrap_or(v);
    let mut it = core.split('.').map(|p| p.trim().parse::<u32>().unwrap_or(0));
    (it.next().unwrap_or(0), it.next().unwrap_or(0), it.next().unwrap_or(0))
}
