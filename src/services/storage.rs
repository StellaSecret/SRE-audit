// ── Local persistence (localStorage) ────────────────────────────────────────
// Keys match the previously-saved data format so entered data keeps working.
// Storage is scoped per organization via `orgs::matrix_key` / `orgs::roadmap_key`;
// the default org ("") reuses the legacy unprefixed keys.

use crate::models::{MatrixState, RoadmapState};
#[cfg(target_arch = "wasm32")]
use crate::orgs;

#[cfg(target_arch = "wasm32")]
pub fn load_matrix(org_id: &str) -> Option<MatrixState> {
    use gloo_storage::{LocalStorage, Storage};
    let key = orgs::matrix_key(org_id);
    let raw: String = LocalStorage::get(&key).ok()?;
    serde_json::from_str(&raw).ok()
}

#[cfg(target_arch = "wasm32")]
pub fn save_matrix(org_id: &str, s: &MatrixState) {
    use gloo_storage::{LocalStorage, Storage};
    let key = orgs::matrix_key(org_id);
    let json = serde_json::to_string(s).expect("matrix state serialization failed");
    let _ = LocalStorage::set(&key, json);
}

#[cfg(target_arch = "wasm32")]
pub fn load_roadmap(org_id: &str) -> Option<RoadmapState> {
    use gloo_storage::{LocalStorage, Storage};
    let key = orgs::roadmap_key(org_id);
    let raw: String = LocalStorage::get(&key).ok()?;
    serde_json::from_str(&raw).ok()
}

#[cfg(target_arch = "wasm32")]
pub fn save_roadmap(org_id: &str, s: &RoadmapState) {
    use gloo_storage::{LocalStorage, Storage};
    let key = orgs::roadmap_key(org_id);
    let json = serde_json::to_string(s).expect("roadmap state serialization failed");
    let _ = LocalStorage::set(&key, json);
}

// Native stubs (compile on non-WASM so `cargo test --lib` works).

#[cfg(not(target_arch = "wasm32"))]
pub fn load_matrix(_org_id: &str) -> Option<MatrixState> {
    None
}
#[cfg(not(target_arch = "wasm32"))]
pub fn save_matrix(_org_id: &str, _s: &MatrixState) {}
#[cfg(not(target_arch = "wasm32"))]
pub fn load_roadmap(_org_id: &str) -> Option<RoadmapState> {
    None
}
#[cfg(not(target_arch = "wasm32"))]
pub fn save_roadmap(_org_id: &str, _s: &RoadmapState) {}
