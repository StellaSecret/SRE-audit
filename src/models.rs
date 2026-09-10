use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ── Matrix de Maturité SRE ──────────────────────────────────────────────────
// Statik data (from the legacy HTML) + user-editable audit state.

/// One bullet item inside a maturity level cell.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LevelItem {
    pub label: String,
    pub text: String,
}

/// One maturity level cell (Niveau 1..4) for a given principle.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatrixLevel {
    pub level: u8,
    pub badge: String,
    pub items: Vec<LevelItem>,
}

impl MatrixLevel {
    pub fn badge_class(&self) -> String {
        format!("bg-lvl{}", self.level)
    }
}

/// A single SRE principle row of the matrix.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatrixRow {
    pub id: u8,
    pub num: u8,
    pub title: String,
    pub aspect: String,
    pub process: String,
    pub levels: Vec<MatrixLevel>,
}

/// Static matrix content (embedded from `src/data/matrix.json`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatrixData {
    pub rows: Vec<MatrixRow>,
}

/// User-editable audit state, persisted to localStorage.
///
/// Shape matches the legacy `sre_matrix_audit_cache` snapshot so previously
/// entered data keeps working:
///   { "companyName": String, "selections": { "1": "3", ... }, "comments": { "1": "…" } }
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MatrixState {
    #[serde(default, rename = "companyName")]
    pub company_name: String,
    #[serde(default)]
    pub selections: BTreeMap<String, String>,
    #[serde(default)]
    pub comments: BTreeMap<String, String>,
}

impl MatrixState {
    pub fn selected_level(&self, row_id: u8) -> Option<u8> {
        self.selections
            .get(&row_id.to_string())
            .and_then(|s| s.parse::<u8>().ok())
    }
}

// ── Roadmap SRE ──────────────────────────────────────────────────────────────
// Roadmap: short term (ST) and long term (LT) tables.

/// One SRE principle row, duplicated in ST and LT tables.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoadmapRow {
    /// e.g. "st_obs" or "lt_obs"
    pub id: String,
    pub num: u8,
    pub title: String,
    pub process: String,
    /// badge strings, e.g. ["Niveau 2 : Organisé"] (ST) or ["Année 1", "…"]
    pub badges: Vec<String>,
    /// editable textarea contents (ST: 3, LT: 2 fields)
    pub areas: Vec<String>,
    /// LT-only target vision badge (e.g. "Niveau 4 : Source Unique")
    #[serde(default)]
    pub vision: String,
}

/// Static roadmap content (embedded from `src/data/roadmap.json`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoadmapData {
    pub st: Vec<RoadmapRow>,
    pub lt: Vec<RoadmapRow>,
}

/// User-editable roadmap state, persisted to localStorage.
///
/// Shape matches the persisted `sre_audit_roadmap_cache` snapshot:
///   { "st": { "st_obs": ["..","..",".."], ... }, "lt": { "lt_obs": ["..",".."], ... } }
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RoadmapState {
    #[serde(default)]
    pub st: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub lt: BTreeMap<String, Vec<String>>,
}

impl RoadmapState {
    pub fn field(&self, row_id: &str, index: usize) -> &str {
        for (key, vals) in self.st.iter().chain(self.lt.iter()) {
            if key == row_id {
                return vals.get(index).map(String::as_str).unwrap_or("");
            }
        }
        ""
    }
}

// ── Roadmap generation templates ─────────────────────────────────────────────
// Bilingual (FR/EN) cell templates used to draft roadmap content from the
// matrix selection. Embedded from `src/data/roadmap_gen.json`.

/// A single bilingual string.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct LangStrings {
    pub fr: String,
    pub en: String,
}

/// A bilingual list of lines (e.g. the "M1/M2/M3" actions).
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct LangLines {
    pub fr: Vec<String>,
    pub en: Vec<String>,
}

/// Full template bank, indexed per principle (`"1".."7"`) and per level
/// (`"1".."4"`): audit finding, short-term actions and KPIs, long-term year-1
/// target and major transformation.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct RoadmapTemplates {
    pub constat: BTreeMap<String, BTreeMap<String, LangStrings>>,
    pub st_actions: BTreeMap<String, BTreeMap<String, LangLines>>,
    pub st_kpi: BTreeMap<String, BTreeMap<String, LangLines>>,
    pub lt_year1: BTreeMap<String, BTreeMap<String, LangStrings>>,
    pub lt_transform: BTreeMap<String, BTreeMap<String, LangStrings>>,
}
