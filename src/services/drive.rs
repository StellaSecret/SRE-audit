// ── Google Drive backup (appDataFolder) ──────────────────────────────────────
// Mirrors CVGenerator's drive.rs: search existing file in the private
// per-app folder, create on first run / patch on subsequent, download to
// restore. Works with the access token obtained in auth.rs.

use crate::models::{MatrixState, RoadmapState};
use serde::{Deserialize, Serialize};

const MATRIX_FILE: &str = "sre_matrix_data.json";
const ROADMAP_FILE: &str = "sre_roadmap_data.json";

/// Per-org Drive file names. The default org ("") keeps the legacy names so
/// existing Drive backups keep working unchanged.
pub fn matrix_file(org_id: &str) -> String {
    if org_id.is_empty() {
        MATRIX_FILE.to_string()
    } else {
        format!("{MATRIX_FILE}__{org_id}")
    }
}

pub fn roadmap_file(org_id: &str) -> String {
    if org_id.is_empty() {
        ROADMAP_FILE.to_string()
    } else {
        format!("{ROADMAP_FILE}__{org_id}")
    }
}

#[cfg(target_arch = "wasm32")]
const DRIVE_API: &str = "https://www.googleapis.com/drive/v3/files";
#[cfg(target_arch = "wasm32")]
const UPLOAD_API: &str = "https://www.googleapis.com/upload/drive/v3/files";

// ── Backup payloads ──────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct MatrixBackup {
    version: u8,
    exported_at: i64,
    data: MatrixState,
}

#[derive(Serialize, Deserialize)]
struct RoadmapBackup {
    version: u8,
    exported_at: i64,
    data: RoadmapState,
}

const BACKUP_VERSION: u8 = 1;

pub fn build_matrix_backup(s: &MatrixState) -> String {
    let b = MatrixBackup {
        version: BACKUP_VERSION,
        exported_at: now_ms(),
        data: s.clone(),
    };
    serde_json::to_string_pretty(&b).expect("backup serialization failed")
}

pub fn restore_matrix_backup(json: &str) -> Result<MatrixState, String> {
    let b: MatrixBackup = serde_json::from_str(json).map_err(|e| format!("Invalid backup: {e}"))?;
    Ok(b.data)
}

pub fn build_roadmap_backup(s: &RoadmapState) -> String {
    let b = RoadmapBackup {
        version: BACKUP_VERSION,
        exported_at: now_ms(),
        data: s.clone(),
    };
    serde_json::to_string_pretty(&b).expect("backup serialization failed")
}

pub fn restore_roadmap_backup(json: &str) -> Result<RoadmapState, String> {
    let b: RoadmapBackup =
        serde_json::from_str(json).map_err(|e| format!("Invalid backup: {e}"))?;
    Ok(b.data)
}

// ── HTTP helpers (WASM) ──────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
async fn check(resp: reqwest::Response) -> Result<reqwest::Response, String> {
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }
    Ok(resp)
}

// ── Drive: upload ────────────────────────────────────────────────────────────
// Creates the file in appDataFolder on first run; patches it afterwards.

#[cfg(target_arch = "wasm32")]
pub async fn drive_upload(file_name: &str, json: &str, token: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let q = format!("name='{file_name}' and 'appDataFolder' in parents and trashed=false");

    let search = check(
        client
            .get(DRIVE_API)
            .query(&[
                ("q", q.as_str()),
                ("fields", "files(id)"),
                ("spaces", "appDataFolder"),
            ])
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Drive search: {e}"))?,
    )
    .await?;

    let body: serde_json::Value = search.json().await.map_err(|e| format!("json: {e}"))?;
    let file_id = body["files"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|f| f["id"].as_str().map(String::from));

    let fid = match file_id {
        Some(id) => id,
        None => {
            let meta = serde_json::json!({
                "name": file_name,
                "parents": ["appDataFolder"],
                "mimeType": "application/json"
            });
            let resp = check(
                client
                    .post(DRIVE_API)
                    .bearer_auth(token)
                    .header("Content-Type", "application/json")
                    .body(meta.to_string())
                    .send()
                    .await
                    .map_err(|e| format!("Drive create: {e}"))?,
            )
            .await?;
            let created: serde_json::Value = resp.json().await.map_err(|e| format!("json: {e}"))?;
            created["id"]
                .as_str()
                .map(String::from)
                .ok_or_else(|| format!("No file id returned: {created}"))?
        }
    };

    check(
        client
            .patch(format!("{UPLOAD_API}/{fid}?uploadType=media"))
            .bearer_auth(token)
            .header("Content-Type", "application/json")
            .body(json.to_string())
            .send()
            .await
            .map_err(|e| format!("Drive upload: {e}"))?,
    )
    .await?;

    Ok(fid)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn drive_upload(_file_name: &str, _json: &str, _token: &str) -> Result<String, String> {
    Err("Drive upload is only available on web".to_string())
}

// ── Drive: download ──────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub async fn drive_download(file_name: &str, token: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let q = format!("name='{file_name}' and 'appDataFolder' in parents and trashed=false");

    let search = check(
        client
            .get(DRIVE_API)
            .query(&[
                ("q", q.as_str()),
                ("fields", "files(id)"),
                ("spaces", "appDataFolder"),
            ])
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Drive search: {e}"))?,
    )
    .await?;

    let body: serde_json::Value = search.json().await.map_err(|e| format!("json: {e}"))?;
    let file_id = body["files"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|f| f["id"].as_str().map(String::from))
        .ok_or_else(|| "No backup found in Drive".to_string())?;

    let resp = check(
        client
            .get(format!("{DRIVE_API}/{file_id}?alt=media"))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Drive download: {e}"))?,
    )
    .await?;

    resp.text().await.map_err(|e| format!("Drive read: {e}"))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn drive_download(_file_name: &str, _token: &str) -> Result<String, String> {
    Err("Drive download is only available on web".to_string())
}

// ── Local file export / import (browser) ─────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub fn local_download(filename: &str, content: &str) {
    use js_sys::Array;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use web_sys::{Blob, Url};

    let arr = Array::new();
    arr.push(&JsValue::from_str(content));
    if let Ok(blob) = Blob::new_with_str_sequence(&arr) {
        if let Ok(url) = Url::create_object_url_with_blob(&blob) {
            let window = web_sys::window().expect("no window");
            if let Some(doc) = window.document() {
                if let Ok(a) = doc.create_element("a") {
                    let _ = a.set_attribute("href", &url);
                    let _ = a.set_attribute("download", filename);
                    if let Some(body) = doc.body() {
                        let _ = body.append_child(&a);
                        if let Some(el) = a.dyn_ref::<web_sys::HtmlElement>() {
                            el.click();
                        }
                        let _ = body.remove_child(&a);
                    }
                }
            }
            Url::revoke_object_url(&url).ok();
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn local_download(_filename: &str, _content: &str) {}

// ── Time helper ──────────────────────────────────────────────────────────────

fn now_ms() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as i64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_matrix() -> MatrixState {
        MatrixState {
            company_name: "ACME Corp".to_string(),
            selections: [("1".to_string(), "3".to_string())].into_iter().collect(),
            comments: [("1".to_string(), "test".to_string())]
                .into_iter()
                .collect(),
        }
    }

    #[test]
    fn matrix_backup_roundtrip() {
        let s = sample_matrix();
        let json = build_matrix_backup(&s);
        assert!(json.contains("ACME Corp"));
        let restored = restore_matrix_backup(&json).unwrap();
        assert_eq!(restored, s);
    }

    #[test]
    fn matrix_backup_rejects_invalid() {
        assert!(restore_matrix_backup("garbage").is_err());
        assert!(restore_matrix_backup(r#"{"not":"backup"}"#).is_err());
    }

    #[test]
    fn roadmap_backup_roundtrip() {
        let s = RoadmapState {
            st: [(
                "st_obs".to_string(),
                vec!["a".into(), "b".into(), "c".into()],
            )]
            .into_iter()
            .collect(),
            lt: Default::default(),
        };
        let json = build_roadmap_backup(&s);
        let restored = restore_roadmap_backup(&json).unwrap();
        assert_eq!(restored, s);
        assert_eq!(restored.field("st_obs", 1), "b");
    }

    #[test]
    fn now_ms_is_ms_since_epoch() {
        assert!(now_ms() > 60_000);
    }
}
