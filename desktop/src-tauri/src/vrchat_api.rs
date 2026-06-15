// VRChat API client + authentication.
//
// Security: we never persist the password (it's used once to obtain a session
// cookie, then dropped). The session cookies (`auth`, `twoFactorAuth`) are kept
// in the OS keyring (Secret Service) — encrypted at rest, unlocked with your
// login session — rather than in a plaintext settings file.
//
// Auth flow mirrors VRChat's API: GET /auth/user with HTTP Basic auth; if it
// returns `requiresTwoFactorAuth`, POST the code to /auth/twofactorauth/<m>/verify.
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use base64::Engine as _;
use reqwest::blocking::{Client, Response};
use reqwest::header::SET_COOKIE;
use serde::Serialize;
use serde_json::Value;

const API: &str = "https://api.vrchat.cloud/api/1";
const KEYRING_SERVICE: &str = "NemuriXR";
const KEY_AUTH: &str = "vrchat_auth";
const KEY_2FA: &str = "vrchat_twofactor";

/// Result of a login / 2FA attempt, serialized to the frontend.
#[derive(Serialize)]
#[serde(tag = "kind")]
pub enum LoginOutcome {
    #[serde(rename = "logged_in")]
    LoggedIn { username: String },
    #[serde(rename = "needs_2fa")]
    Needs2fa { methods: Vec<String> },
    #[serde(rename = "failed")]
    Failed { message: String },
}

/// Login status snapshot for the UI.
#[derive(Serialize, Clone)]
pub struct LoginStatus {
    pub logged_in: bool,
    pub username: Option<String>,
}

/// A VRChat friend, for the auto-accept whitelist picker.
#[derive(Serialize, Clone)]
pub struct Friend {
    pub id: String,
    pub display_name: String,
}

/// Sliding-window rate limiter. VRChat permits API use but dislikes spam, so
/// every write is capped per-minute (plus a global cap). Mirrors OyasumiVR's
/// limits. Our triggers are event-driven (websocket + log), so this is mostly a
/// safety net.
#[derive(Default)]
struct RateLimiter {
    hits: HashMap<&'static str, Vec<Instant>>,
}

impl RateLimiter {
    fn under(&mut self, key: &'static str, now: Instant, cap: usize) -> bool {
        let v = self.hits.entry(key).or_default();
        v.retain(|t| now.duration_since(*t) < Duration::from_secs(60));
        v.len() < cap
    }

    /// Allow a call of `key` (per-minute `cap`) if both it and the global cap
    /// (15/min) have room; records the call when allowed.
    fn allow(&mut self, key: &'static str, cap: usize) -> bool {
        let now = Instant::now();
        if !self.under("__global", now, 15) || !self.under(key, now, cap) {
            return false;
        }
        self.hits.get_mut("__global").unwrap().push(now);
        self.hits.get_mut(key).unwrap().push(now);
        true
    }
}

pub struct Api {
    client: Client,
    auth_cookie: Option<String>,
    two_factor_cookie: Option<String>,
    pub user_id: Option<String>,
    pub username: Option<String>,
    limiter: RateLimiter,
}

impl Api {
    pub fn new() -> Self {
        let ua = format!("NemuriXR/{} (https://github.com/eidenz/nemurixr)", env!("CARGO_PKG_VERSION"));
        let client = Client::builder()
            .user_agent(ua)
            .timeout(Duration::from_secs(20))
            .build()
            .unwrap_or_else(|_| Client::new());
        let mut api =
            Self { client, auth_cookie: None, two_factor_cookie: None, user_id: None, username: None, limiter: RateLimiter::default() };
        api.load_cookies();
        api
    }

    /// The websocket auth token (the `auth` session cookie value).
    pub fn auth_token(&self) -> Option<String> {
        self.auth_cookie.clone()
    }

    /// Accept an invite request by inviting the user into our instance.
    pub fn invite_user(&mut self, user_id: &str, instance_id: &str) -> bool {
        if self.user_id.is_none() {
            return false;
        }
        if !self.limiter.allow("invite", 12) {
            log::warn!("VRChat: invite rate-limited, skipping");
            return false;
        }
        match self
            .client
            .post(format!("{API}/invite/{user_id}"))
            .header("Cookie", self.cookie_header())
            .json(&serde_json::json!({ "instanceId": instance_id }))
            .send()
        {
            Ok(r) => r.status().is_success(),
            Err(e) => {
                log::warn!("VRChat invite failed: {e}");
                false
            }
        }
    }

    /// Hide (dismiss) a notification after handling it.
    pub fn hide_notification(&mut self, notification_id: &str) -> bool {
        if !self.limiter.allow("hide_notification", 3) {
            return false;
        }
        match self
            .client
            .put(format!("{API}/auth/user/notifications/{notification_id}/hide"))
            .header("Cookie", self.cookie_header())
            .send()
        {
            Ok(r) => r.status().is_success(),
            Err(e) => {
                log::warn!("VRChat hide notification failed: {e}");
                false
            }
        }
    }

    /// Set the account status (e.g. "join me", "ask me", "busy").
    pub fn set_status(&mut self, status: &str) -> bool {
        let Some(uid) = self.user_id.clone() else { return false };
        if !self.limiter.allow("status", 6) {
            log::warn!("VRChat: status change rate-limited, skipping");
            return false;
        }
        match self
            .client
            .put(format!("{API}/users/{uid}"))
            .header("Cookie", self.cookie_header())
            .json(&serde_json::json!({ "status": status }))
            .send()
        {
            Ok(r) => r.status().is_success(),
            Err(e) => {
                log::warn!("VRChat set status failed: {e}");
                false
            }
        }
    }

    /// Friends list (online + offline, up to 100 each) for the whitelist picker.
    pub fn get_friends(&mut self) -> Vec<Friend> {
        if self.auth_cookie.is_none() {
            return Vec::new();
        }
        let mut out: Vec<Friend> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for offline in [false, true] {
            if !self.limiter.allow("friends", 15) {
                break;
            }
            let url = format!("{API}/auth/user/friends?n=100&offset=0&offline={offline}");
            let Ok(resp) = self.client.get(url).header("Cookie", self.cookie_header()).send() else { continue };
            self.capture_cookies(&resp);
            let Ok(arr) = resp.json::<Vec<Value>>() else { continue };
            for f in arr {
                if let (Some(id), Some(name)) = (f.get("id").and_then(Value::as_str), f.get("displayName").and_then(Value::as_str)) {
                    if seen.insert(id.to_string()) {
                        out.push(Friend { id: id.to_string(), display_name: name.to_string() });
                    }
                }
            }
        }
        out.sort_by(|a, b| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()));
        out
    }

    pub fn login_status(&self) -> LoginStatus {
        LoginStatus { logged_in: self.username.is_some(), username: self.username.clone() }
    }

    /// Validate a restored cookie on startup (populates the username, or clears
    /// the session if the cookie is no longer valid).
    pub fn restore(&mut self) {
        if self.auth_cookie.is_none() {
            return;
        }
        match self.fetch_current_user() {
            Ok(true) => log::info!("VRChat: restored session as {:?}", self.username),
            Ok(false) => {
                log::info!("VRChat: stored session no longer valid; clearing");
                self.clear();
            }
            Err(e) => log::warn!("VRChat: couldn't validate session ({e}); keeping cookie"),
        }
    }

    pub fn login(&mut self, username: &str, password: &str) -> LoginOutcome {
        let creds = format!("{}:{}", urlencoding::encode(username), urlencoding::encode(password));
        let basic = base64::engine::general_purpose::STANDARD.encode(creds);
        let resp = match self.client.get(format!("{API}/auth/user")).header("Authorization", format!("Basic {basic}")).send() {
            Ok(r) => r,
            Err(e) => return LoginOutcome::Failed { message: net_error(e) },
        };
        self.capture_cookies(&resp);
        self.outcome_from_auth_user(resp)
    }

    pub fn verify_2fa(&mut self, method: &str, code: &str) -> LoginOutcome {
        // requiresTwoFactorAuth uses e.g. "totp"/"emailOtp"/"otp"; the verify
        // path is lowercase.
        let m = method.to_lowercase();
        let resp = match self
            .client
            .post(format!("{API}/auth/twofactorauth/{m}/verify"))
            .header("Cookie", self.cookie_header())
            .json(&serde_json::json!({ "code": code }))
            .send()
        {
            Ok(r) => r,
            Err(e) => return LoginOutcome::Failed { message: net_error(e) },
        };
        self.capture_cookies(&resp);
        let verified = resp.json::<Value>().ok().and_then(|b| b.get("verified").and_then(Value::as_bool)).unwrap_or(false);
        if !verified {
            return LoginOutcome::Failed { message: "Invalid 2FA code".to_string() };
        }
        self.persist_cookies();
        match self.fetch_current_user() {
            Ok(true) => LoginOutcome::LoggedIn { username: self.username.clone().unwrap_or_default() },
            _ => LoginOutcome::Failed { message: "Verified, but couldn't load your profile".to_string() },
        }
    }

    pub fn logout(&mut self) {
        self.clear();
    }

    // --- internals ---

    fn outcome_from_auth_user(&mut self, resp: Response) -> LoginOutcome {
        let body: Value = match resp.json() {
            Ok(b) => b,
            Err(e) => return LoginOutcome::Failed { message: net_error(e) },
        };
        if let Some(methods) = body.get("requiresTwoFactorAuth").and_then(Value::as_array) {
            // The temporary auth cookie is set now; keep it for the verify step.
            self.persist_cookies();
            let methods = methods.iter().filter_map(|m| m.as_str().map(str::to_string)).collect();
            return LoginOutcome::Needs2fa { methods };
        }
        if let (Some(id), Some(name)) = (body.get("id").and_then(Value::as_str), body.get("displayName").and_then(Value::as_str)) {
            self.user_id = Some(id.to_string());
            self.username = Some(name.to_string());
            self.persist_cookies();
            return LoginOutcome::LoggedIn { username: name.to_string() };
        }
        let msg = body
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("Login failed — check your username and password")
            .to_string();
        LoginOutcome::Failed { message: msg }
    }

    /// GET /auth/user with the stored cookie. Returns Ok(true) if a valid user
    /// was loaded, Ok(false) if the session is invalid / needs re-login.
    fn fetch_current_user(&mut self) -> Result<bool, String> {
        let resp = self
            .client
            .get(format!("{API}/auth/user"))
            .header("Cookie", self.cookie_header())
            .send()
            .map_err(net_error)?;
        self.capture_cookies(&resp);
        let body: Value = resp.json().map_err(net_error)?;
        if body.get("requiresTwoFactorAuth").is_some() {
            return Ok(false); // cookie expired / 2FA needed again
        }
        match (body.get("id").and_then(Value::as_str), body.get("displayName").and_then(Value::as_str)) {
            (Some(id), Some(name)) => {
                self.user_id = Some(id.to_string());
                self.username = Some(name.to_string());
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn cookie_header(&self) -> String {
        let mut parts = Vec::new();
        if let Some(a) = &self.auth_cookie {
            parts.push(format!("auth={a}"));
        }
        if let Some(t) = &self.two_factor_cookie {
            parts.push(format!("twoFactorAuth={t}"));
        }
        parts.join("; ")
    }

    fn capture_cookies(&mut self, resp: &Response) {
        for hv in resp.headers().get_all(SET_COOKIE) {
            let Ok(s) = hv.to_str() else { continue };
            let Some((name, rest)) = s.split_once('=') else { continue };
            let value = rest.split(';').next().unwrap_or("").to_string();
            match name.trim() {
                "auth" => self.auth_cookie = Some(value),
                "twoFactorAuth" => self.two_factor_cookie = Some(value),
                _ => {}
            }
        }
    }

    // --- keyring-backed cookie storage ---

    fn persist_cookies(&self) {
        store_secret(KEY_AUTH, self.auth_cookie.as_deref());
        store_secret(KEY_2FA, self.two_factor_cookie.as_deref());
    }

    fn load_cookies(&mut self) {
        self.auth_cookie = load_secret(KEY_AUTH);
        self.two_factor_cookie = load_secret(KEY_2FA);
    }

    fn clear(&mut self) {
        self.auth_cookie = None;
        self.two_factor_cookie = None;
        self.user_id = None;
        self.username = None;
        store_secret(KEY_AUTH, None);
        store_secret(KEY_2FA, None);
    }
}

fn net_error(e: reqwest::Error) -> String {
    format!("Network error: {e}")
}

// Wrap keyring so a missing/locked Secret Service degrades gracefully (the
// session just won't persist across restarts).
fn keyring_entry(key: &str) -> Option<keyring::Entry> {
    match keyring::Entry::new(KEYRING_SERVICE, key) {
        Ok(e) => Some(e),
        Err(e) => {
            log::warn!("keyring unavailable ({e}); VRChat session won't persist");
            None
        }
    }
}

fn store_secret(key: &str, value: Option<&str>) {
    let Some(entry) = keyring_entry(key) else { return };
    let r = match value {
        Some(v) => entry.set_password(v),
        None => entry.delete_credential(),
    };
    if let Err(e) = r {
        // delete of a non-existent entry is fine; log others at debug.
        log::debug!("keyring {key}: {e}");
    }
}

fn load_secret(key: &str) -> Option<String> {
    keyring_entry(key)?.get_password().ok()
}

/// Thread-safe handle managed by Tauri.
pub type SharedApi = std::sync::Arc<Mutex<Api>>;
