// ── Google Identity Services (GIS) OAuth2 + verified-email gate ──────────────
//
// Client-side OAuth2 flow (like CVGenerator) requesting the Drive appdata
// scope plus `openid email profile`. On token receipt we call Google's
// oauth2/v3/userinfo endpoint to obtain the SERVER-VERIFIED email, then apply
// the PREMIUM_EMAILS allowlist gate (see gate.rs).
//
// Security note: on a static site there is no backend — any determined user
// could bypass the client gate. The gate exists to keep the tool usable only
// by declared emails (matching the GameTracker model). Session is re-verified
// against Google on every app load (unless built with SRE_AUDIT_E2E=1).

use crate::config::CLIENT_ID;
#[cfg(target_arch = "wasm32")]
use crate::config::{DRIVE_SCOPE, E2E_MODE};
#[cfg(any(target_arch = "wasm32", test))]
use crate::services::gate::is_allowed;
#[cfg(target_arch = "wasm32")]
use crate::services::gate::verify_profile;
use crate::services::gate::VerifiedProfile;
use std::sync::OnceLock;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
const TOKEN_KEY: &str = "sre_audit_google_token";
#[cfg(target_arch = "wasm32")]
const PROFILE_KEY: &str = "sre_audit_profile";

pub const USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";

// ── Session ──────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AuthState {
    pub signed_in: bool,
    pub profile: Option<VerifiedProfile>,
    /// access token for Drive API (empty when not signed in)
    pub token: String,
    /// true only when signed in AND email verified AND in allowlist
    pub allowed: bool,
    pub error: Option<String>,
    pub configured: bool,
}

static AUTH_STATE: OnceLock<std::sync::RwLock<AuthState>> = OnceLock::new();
#[cfg(target_arch = "wasm32")]
static AUTH_INIT: OnceLock<()> = OnceLock::new();

fn state() -> &'static std::sync::RwLock<AuthState> {
    AUTH_STATE.get_or_init(|| {
        std::sync::RwLock::new(AuthState {
            configured: CLIENT_ID.is_some(),
            ..AuthState::default()
        })
    })
}

pub fn get_state() -> AuthState {
    state().read().unwrap().clone()
}

// The auth state lives outside the Dioxus reactive system (it's a plain
// static). The UI subscribes via listener callbacks that bump a Dioxus signal.
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static AUTH_LISTENERS: RefCell<Vec<Box<dyn FnMut()>>> = const { RefCell::new(Vec::new()) };
}

#[cfg(target_arch = "wasm32")]
pub fn on_auth_change(cb: Box<dyn FnMut()>) {
    AUTH_LISTENERS.with(|l| l.borrow_mut().push(cb));
}

#[cfg(not(target_arch = "wasm32"))]
pub fn on_auth_change(_cb: Box<dyn FnMut()>) {}

/// Mutate the shared state, release the lock, THEN notify listeners.
fn update_and_notify(f: impl FnOnce(&mut AuthState)) {
    {
        let mut s = state().write().unwrap();
        f(&mut s);
    }
    #[cfg(target_arch = "wasm32")]
    AUTH_LISTENERS.with(|l| {
        for cb in l.borrow_mut().iter_mut() {
            cb();
        }
    });
}

fn empty_state() -> AuthState {
    AuthState {
        configured: CLIENT_ID.is_some(),
        ..AuthState::default()
    }
}

// ── Session persistence ──────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn persist_state(s: &AuthState) {
    use gloo_storage::{LocalStorage, Storage};
    if s.token.is_empty() {
        LocalStorage::delete(TOKEN_KEY);
        LocalStorage::delete(PROFILE_KEY);
    } else {
        let _ = LocalStorage::set(TOKEN_KEY, &s.token);
        if let Some(p) = &s.profile {
            let _ = LocalStorage::set(PROFILE_KEY, serde_json::to_string(p).unwrap_or_default());
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn erase_storage() {
    use gloo_storage::{LocalStorage, Storage};
    LocalStorage::delete(TOKEN_KEY);
    LocalStorage::delete(PROFILE_KEY);
}

// ── The gate: derive `allowed` from the verified profile ────────────────────

#[cfg(any(target_arch = "wasm32", test))]
fn compute_allowed(s: &mut AuthState) {
    s.allowed = match &s.profile {
        Some(p) => p.email_verified && is_allowed(&p.email),
        None => false,
    };
}

// ── init: inject the GIS script and restore session ─────────────────────────

#[cfg(target_arch = "wasm32")]
pub fn init() {
    use wasm_bindgen::JsCast;

    let doc = web_sys::window()
        .expect("no window in WASM")
        .document()
        .expect("no document in WASM");

    if get_gis_oauth2().is_err() {
        if let Ok(script) = doc.create_element("script") {
            if let Ok(s) = script.dyn_into::<web_sys::HtmlScriptElement>() {
                s.set_src("https://accounts.google.com/gsi/client");
                s.set_defer(true);
                if let Some(body) = doc.body() {
                    let _ = body.append_child(&s);
                }
            }
        }
    }

    // Restore previous session + re-verify with Google, exactly once.
    if AUTH_INIT.get().is_none() {
        AUTH_INIT.set(()).ok();
        restore_session();
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn init() {}

#[cfg(target_arch = "wasm32")]
fn restore_session() {
    use gloo_storage::{LocalStorage, Storage};

    let token: Option<String> = LocalStorage::get(TOKEN_KEY).ok();
    let profile: Option<String> = LocalStorage::get(PROFILE_KEY).ok();

    // E2E builds trust a previously-stored session without a network call.
    if E2E_MODE {
        update_and_notify(|s| {
            *s = AuthState {
                signed_in: token.is_some(),
                token: token.unwrap_or_default(),
                profile: profile
                    .as_deref()
                    .and_then(|p| serde_json::from_str(p).ok()),
                configured: CLIENT_ID.is_some(),
                ..AuthState::default()
            };
            compute_allowed(s);
        });
        return;
    }

    match token {
        Some(tok) => apply_token(tok),
        None => update_and_notify(|s| *s = empty_state()),
    }
}

// ── OAuth flow ───────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn get_gis_oauth2() -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    let google = js_sys::Reflect::get(&js_sys::global(), &"google".into())?;
    let accounts = js_sys::Reflect::get(&google, &"accounts".into())?;
    js_sys::Reflect::get(&accounts, &"oauth2".into())
}

/// Trigger the Google sign-in flow. On success the verified profile is
/// fetched and the allowlist gate applied; listeners are notified either way.
#[cfg(target_arch = "wasm32")]
pub fn start_oauth() {
    use wasm_bindgen::prelude::Closure;

    let cid = match CLIENT_ID {
        Some(c) if !c.is_empty() => c,
        _ => {
            update_and_notify(|s| {
                s.error = Some("GOOGLE_CLIENT_ID not configured at build time".to_string())
            });
            return;
        }
    };

    if get_gis_oauth2().is_err() {
        update_and_notify(|s| {
            s.error = Some("Google Identity Services library not loaded".to_string())
        });
        return;
    }

    let cb = Closure::wrap(Box::new(move |resp: wasm_bindgen::JsValue| {
        let token = js_sys::Reflect::get(&resp, &"access_token".into())
            .ok()
            .and_then(|t| t.as_string());
        match token {
            Some(t) => apply_token(t),
            None => update_and_notify(|s| {
                s.signed_in = false;
                s.error = Some("Google sign-in was cancelled or failed".to_string());
            }),
        }
    }) as Box<dyn FnMut(wasm_bindgen::JsValue)>);

    let config = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&config, &"client_id".into(), &cid.into());
    let _ = js_sys::Reflect::set(&config, &"scope".into(), &DRIVE_SCOPE.into());
    let _ = js_sys::Reflect::set(&config, &"callback".into(), cb.as_ref());

    if let Ok(oauth2) = get_gis_oauth2() {
        if let Some(f) = js_sys::Reflect::get(&oauth2, &"initTokenClient".into())
            .ok()
            .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
        {
            if let Ok(tc) = f.call1(&oauth2, &config) {
                cb.forget();
                if let Some(g) = js_sys::Reflect::get(&tc, &"requestAccessToken".into())
                    .ok()
                    .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
                {
                    let _ = g.call0(&tc);
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn start_oauth() {}

// ── Apply a freshly-obtained access token ────────────────────────────────────
// Fetches the server-verified userinfo then applies the allowlist gate.

#[cfg(target_arch = "wasm32")]
pub fn apply_token(token: String) {
    wasm_bindgen_futures::spawn_local(async move {
        {
            let mut s = state().write().unwrap();
            s.token = token.clone();
            s.signed_in = true;
            s.error = None;
        }
        let profile = fetch_userinfo(&token).await;
        update_and_notify(move |s| {
            match profile {
                Ok(p) => s.profile = Some(p),
                Err(e) => {
                    s.profile = None;
                    s.error = Some(format!("Could not verify identity with Google: {e}"));
                    s.signed_in = false;
                }
            }
            compute_allowed(s);
            if s.allowed {
                persist_state(s);
            } else {
                *s = empty_state();
                erase_storage();
            }
        });
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn apply_token(_token: String) {}

#[cfg(target_arch = "wasm32")]
async fn fetch_userinfo(token: &str) -> Result<VerifiedProfile, String> {
    let resp = reqwest::Client::new()
        .get(USERINFO_URL)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("userinfo request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("userinfo HTTP {}", resp.status()));
    }
    let text = resp
        .text()
        .await
        .map_err(|e| format!("userinfo read: {e}"))?;
    verify_profile(&text)
}

// ── Sign out ─────────────────────────────────────────────────────────────────

pub fn sign_out() {
    update_and_notify(|s| *s = empty_state());
    #[cfg(target_arch = "wasm32")]
    erase_storage();
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_requires_verified_allowlisted_email() {
        // allowlist empty at this test's build time → nobody allowed
        let mut s = AuthState {
            signed_in: true,
            profile: Some(VerifiedProfile {
                email: "nobody@example.com".to_string(),
                email_verified: true,
                sub: "1".to_string(),
                name: String::new(),
            }),
            token: "tok".to_string(),
            ..AuthState::default()
        };
        compute_allowed(&mut s);
        assert!(!s.allowed);
    }

    #[test]
    fn unverified_email_never_allowed() {
        let mut s = AuthState {
            signed_in: true,
            profile: Some(VerifiedProfile {
                email: "x@example.com".to_string(),
                email_verified: false,
                sub: "1".to_string(),
                name: String::new(),
            }),
            token: "tok".to_string(),
            ..AuthState::default()
        };
        compute_allowed(&mut s);
        assert!(!s.allowed);
    }

    #[test]
    fn no_profile_means_not_allowed() {
        let mut s = AuthState {
            signed_in: true,
            token: "tok".to_string(),
            ..AuthState::default()
        };
        compute_allowed(&mut s);
        assert!(!s.allowed);
    }
}
