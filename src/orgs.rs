// ── Audit organizations ─────────────────────────────────────────────────────
// The app can audit several organizations. Each org owns its own Matrix and
// Roadmap state, scoped in localStorage and Drive by its id.
//
// The *default* org (id == "") maps to the legacy keys/files so data saved
// before multi-org support keeps working with zero migration.

use serde::{Deserialize, Serialize};

pub const ORGS_KEY: &str = "sre_audit_orgs";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
}

impl Organization {
    pub fn is_default(&self) -> bool {
        self.id.is_empty()
    }
}

/// Ordered org list plus the active org id ("" is the default org).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OrgStore {
    #[serde(default)]
    pub orgs: Vec<Organization>,
    #[serde(default)]
    pub current: String,
}

impl OrgStore {
    /// Fall back current to the default org (or first) if it no longer exists.
    pub fn normalize(&mut self) {
        if !self.orgs.iter().any(|o| o.id == self.current) {
            self.current = self.orgs.first().map(|o| o.id.clone()).unwrap_or_default();
        }
    }

    pub fn current_org(&self) -> Option<&Organization> {
        self.orgs.iter().find(|o| o.id == self.current)
    }

    pub fn rename_org(&mut self, id: &str, name: &str) -> bool {
        match self.orgs.iter_mut().find(|o| o.id == id) {
            Some(o) => {
                o.name = name.trim().to_string();
                true
            }
            None => false,
        }
    }

    /// Adds a new org; makes it current.
    pub fn add(&mut self, name: &str) -> String {
        let id = new_id();
        let name = name.trim().to_string();
        let name_ok = if name.is_empty() {
            "Nouvelle organisation".to_string()
        } else {
            name
        };
        self.orgs.push(Organization {
            id: id.clone(),
            name: name_ok,
        });
        self.current = id.clone();
        id
    }

    /// Removes an org and returns its name. The default org ("") can be removed
    /// like any other; current falls back to the first remaining org.
    pub fn remove(&mut self, id: &str) -> Option<String> {
        let pos = self.orgs.iter().position(|o| o.id == id)?;
        let removed = self.orgs.remove(pos).name;
        self.normalize();
        Some(removed)
    }
}

pub fn apply_defaults(mut store: OrgStore, matrix: &crate::models::MatrixState) -> OrgStore {
    let name = if matrix.company_name.trim().is_empty() {
        "Organisation".to_string()
    } else {
        matrix.company_name.trim().to_string()
    };
    store.orgs.insert(
        0,
        Organization {
            id: String::new(),
            name,
        },
    );
    store.normalize();
    store
}

// ── Storage key / Drive file scoping ───────────────────────────────────────
// The default org ("") reuses the pre-multi-org names so existing browsers and
// Drive backups keep working unchanged.

pub fn matrix_key(org_id: &str) -> String {
    suffix("sre_matrix_audit_cache", org_id)
}
pub fn roadmap_key(org_id: &str) -> String {
    suffix("sre_audit_roadmap_cache", org_id)
}
fn suffix(base: &str, org_id: &str) -> String {
    if org_id.is_empty() {
        base.to_string()
    } else {
        format!("{base}__{org_id}")
    }
}

// ── Persistence ─────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub fn load() -> Option<OrgStore> {
    use gloo_storage::{LocalStorage, Storage};
    let raw: String = LocalStorage::get(ORGS_KEY).ok()?;
    serde_json::from_str(&raw).ok()
}

#[cfg(target_arch = "wasm32")]
pub fn save(store: &OrgStore) {
    use gloo_storage::{LocalStorage, Storage};
    let json = serde_json::to_string(store).expect("org store serialization failed");
    let _ = LocalStorage::set(ORGS_KEY, json);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load() -> Option<OrgStore> {
    None
}
#[cfg(not(target_arch = "wasm32"))]
pub fn save(_store: &OrgStore) {}

// ── Org id generation ───────────────────────────────────────────────────────

/// New-org id: millis timestamp (stable across renames — storage/Drive key off
/// id, never name).
pub fn new_id() -> String {
    let now = now_ms();
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&now.to_le_bytes());
    hex(&buf)
}

fn now_ms() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MatrixState;

    #[test]
    fn default_org_uses_legacy_keys() {
        assert_eq!(matrix_key(""), "sre_matrix_audit_cache");
        assert_eq!(roadmap_key(""), "sre_audit_roadmap_cache");
    }

    #[test]
    fn non_default_org_uses_suffixed_keys() {
        assert_eq!(matrix_key("abc"), "sre_matrix_audit_cache__abc");
        assert_eq!(roadmap_key("abc"), "sre_audit_roadmap_cache__abc");
    }

    #[test]
    fn apply_defaults_seeds_from_matrix_company() {
        let m = MatrixState {
            company_name: "  ACME  ".to_string(),
            ..MatrixState::default()
        };
        let store = apply_defaults(OrgStore::default(), &m);
        assert_eq!(store.orgs.len(), 1);
        assert_eq!(store.orgs[0].id, "");
        assert_eq!(store.orgs[0].name, "ACME");
        assert_eq!(store.current, "");
    }

    #[test]
    fn normalize_falls_back_when_current_missing() {
        let mut store = OrgStore {
            orgs: vec![Organization {
                id: "a".into(),
                name: "A".into(),
            }],
            current: "b".into(),
        };
        store.normalize();
        assert_eq!(store.current, "a");
    }

    #[test]
    fn new_id_is_nonempty_hex() {
        let id = new_id();
        assert!(!id.is_empty());
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
