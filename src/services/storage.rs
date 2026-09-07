// ── Local persistence (localStorage) ────────────────────────────────────────
// Keys match the previously-saved data format so entered data keeps working:
//   - sre_matrix_audit_cache   (matrix: companyName, selections, comments)
//   - sre_audit_roadmap_cache  (roadmap: { st, lt })

use crate::models::{MatrixState, RoadmapState};

#[cfg(target_arch = "wasm32")]
const MATRIX_KEY: &str = "sre_matrix_audit_cache";
#[cfg(target_arch = "wasm32")]
const ROADMAP_KEY: &str = "sre_audit_roadmap_cache";

#[cfg(target_arch = "wasm32")]
pub fn load_matrix() -> Option<MatrixState> {
    use gloo_storage::{LocalStorage, Storage};
    let raw: String = LocalStorage::get(MATRIX_KEY).ok()?;
    serde_json::from_str(&raw).ok()
}

#[cfg(target_arch = "wasm32")]
pub fn save_matrix(s: &MatrixState) {
    use gloo_storage::{LocalStorage, Storage};
    let json = serde_json::to_string(s).expect("matrix state serialization failed");
    let _ = LocalStorage::set(MATRIX_KEY, json);
}

#[cfg(target_arch = "wasm32")]
pub fn load_roadmap() -> Option<RoadmapState> {
    use gloo_storage::{LocalStorage, Storage};
    let raw: String = LocalStorage::get(ROADMAP_KEY).ok()?;
    serde_json::from_str(&raw).ok()
}

#[cfg(target_arch = "wasm32")]
pub fn save_roadmap(s: &RoadmapState) {
    use gloo_storage::{LocalStorage, Storage};
    let json = serde_json::to_string(s).expect("roadmap state serialization failed");
    let _ = LocalStorage::set(ROADMAP_KEY, json);
}

// Native stubs (compile on non-WASM so `cargo test --lib` works).

#[cfg(not(target_arch = "wasm32"))]
pub const MATRIX_KEY: &str = "sre_matrix_audit_cache";
#[cfg(not(target_arch = "wasm32"))]
pub const ROADMAP_KEY: &str = "sre_audit_roadmap_cache";

#[cfg(not(target_arch = "wasm32"))]
pub fn load_matrix() -> Option<MatrixState> {
    None
}
#[cfg(not(target_arch = "wasm32"))]
pub fn save_matrix(_s: &MatrixState) {}
#[cfg(not(target_arch = "wasm32"))]
pub fn load_roadmap() -> Option<RoadmapState> {
    None
}
#[cfg(not(target_arch = "wasm32"))]
pub fn save_roadmap(_s: &RoadmapState) {}
