// ── Roadmap draft generation from the matrix ─────────────────────────────────
// Pure, wasm-free helpers: derive horizon badges from the org's matrix
// selection and draft ST/LT cell content from the matrix bank (current-level
// items = audit finding, next-level items = actions, level-4 items =
// long-term transformation). Only ever fills empty cells, so hand-written
// roadmap content is never overwritten.

use crate::models::{
    LevelItem, MatrixData, MatrixLevel, MatrixRow, MatrixState, RoadmapData, RoadmapState,
};

fn matrix_row(data: &MatrixData, num: u8) -> Option<&MatrixRow> {
    data.rows.iter().find(|r| r.num == num)
}

fn level_at(row: &MatrixRow, level: u8) -> Option<&MatrixLevel> {
    row.levels.get(level.checked_sub(1)? as usize)
}

/// "Niveau {level} · {badge}" for a principle, or None if unknown.
pub fn level_badge(data: &MatrixData, num: u8, level: u8) -> Option<String> {
    let row = matrix_row(data, num)?;
    Some(format!("Niveau {level} · {}", level_at(row, level)?.badge))
}

/// Level-4 badge ("Niveau 4 · …"), the long-term target vision.
pub fn vision_badge(data: &MatrixData, num: u8) -> Option<String> {
    level_badge(data, num, 4)
}

fn bullets(items: &[LevelItem]) -> String {
    items
        .iter()
        .map(|i| format!("• {} : {}", i.label, i.text))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Target wording for the next step: level L+1 (or "maintain" at level 4).
fn next_target(data: &MatrixData, num: u8, level: u8) -> Option<String> {
    let row = matrix_row(data, num)?;
    if level >= 4 {
        Some(format!("Niveau 4 · {}", level_at(row, 4)?.badge))
    } else {
        Some(format!(
            "Niveau {} · {}",
            level + 1,
            level_at(row, level + 1)?.badge
        ))
    }
}

fn fill_cell(
    map: &mut std::collections::BTreeMap<String, Vec<String>>,
    key: &str,
    idx: usize,
    value: &str,
) {
    let entry = map.entry(key.to_string()).or_default();
    if entry.len() <= idx {
        entry.resize(idx + 1, String::new());
    }
    if entry[idx].is_empty() {
        entry[idx] = value.to_string();
    }
}

/// Draft the roadmap for the current matrix selection. Rows without a
/// selected level are left untouched; existing cell content is preserved.
pub fn fill_draft(
    state: &mut RoadmapState,
    roadmap: &RoadmapData,
    matrix: &MatrixData,
    sel: &MatrixState,
) {
    for st_row in &roadmap.st {
        let Some(level) = sel.selected_level(st_row.num) else {
            continue;
        };
        let Some(row) = matrix_row(matrix, st_row.num) else {
            continue;
        };
        let Some(current) = level_at(row, level) else {
            continue;
        };

        let mut constat = String::new();
        if let Some(c) = sel.comments.get(&st_row.num.to_string()) {
            if !c.trim().is_empty() {
                constat.push_str(&format!("Note d'audit : {c}\n"));
            }
        }
        constat.push_str(&bullets(&current.items));

        let actions = if level >= 4 {
            format!("Consolider :\n{}", bullets(&current.items))
        } else if let Some(next) = level_at(row, level + 1) {
            bullets(&next.items)
        } else {
            String::new()
        };

        let target = next_target(matrix, st_row.num, level).unwrap_or_default();

        fill_cell(&mut state.st, &st_row.id, 0, &constat);
        fill_cell(&mut state.st, &st_row.id, 1, &actions);
        fill_cell(
            &mut state.st,
            &st_row.id,
            2,
            &format!("Atteindre {target} à 3 mois."),
        );
    }

    for lt_row in &roadmap.lt {
        let Some(level) = sel.selected_level(lt_row.num) else {
            continue;
        };
        let Some(row) = matrix_row(matrix, lt_row.num) else {
            continue;
        };
        let Some(target) = next_target(matrix, lt_row.num, level) else {
            continue;
        };
        let transformation = level_at(row, 4)
            .map(|l| bullets(&l.items))
            .unwrap_or_default();

        fill_cell(
            &mut state.lt,
            &lt_row.id,
            0,
            &format!("Cible Année 1 : {target}"),
        );
        fill_cell(&mut state.lt, &lt_row.id, 1, &transformation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data;

    fn base() -> (RoadmapData, MatrixData) {
        (data::roadmap(), data::matrix())
    }

    #[test]
    fn level_two_fills_all_st_lt_cells() {
        let (roadmap, matrix) = base();
        let sel = MatrixState {
            selections: [("2".to_string(), "2".to_string())].into_iter().collect(),
            comments: [(
                "2".to_string(),
                "Des SLA purement contractuels.".to_string(),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        let mut state = RoadmapState::default();
        fill_draft(&mut state, &roadmap, &matrix, &sel);

        let st = state.st.get("st_slo").unwrap();
        assert!(st[0].contains("Note d'audit : Des SLA purement contractuels."));
        assert!(st[0].contains("SLA existants mais purement juridiques"));
        assert!(st[1].contains("Alignement tripartite"));
        assert_eq!(st[2], "Atteindre Niveau 3 · SLI/SLO Métier à 3 mois.");

        let lt = state.lt.get("lt_slo").unwrap();
        assert_eq!(lt[0], "Cible Année 1 : Niveau 3 · SLI/SLO Métier");
        assert!(lt[1].contains("Les SLO servent de juge de paix automatisé"));
    }

    #[test]
    fn level_four_uses_consolidate_and_maintain() {
        let (roadmap, matrix) = base();
        let sel = MatrixState {
            selections: [("7".to_string(), "4".to_string())].into_iter().collect(),
            ..Default::default()
        };
        let mut state = RoadmapState::default();
        fill_draft(&mut state, &roadmap, &matrix, &sel);

        let st = state.st.get("st_simp").unwrap();
        assert!(st[0].contains("Audit de configuration continu"));
        assert!(st[1].starts_with("Consolider :"));
        assert_eq!(st[2], "Atteindre Niveau 4 · Native & Sobre à 3 mois.");

        let lt = state.lt.get("lt_simp").unwrap();
        assert_eq!(lt[0], "Cible Année 1 : Niveau 4 · Native & Sobre");
        assert!(lt[1].contains("Bascules inter-régions entièrement automatisées"));
    }

    #[test]
    fn preserves_manual_text_and_skips_unselected() {
        let (roadmap, matrix) = base();
        let sel = MatrixState {
            selections: [("2".to_string(), "2".to_string())].into_iter().collect(),
            ..Default::default()
        };
        let mut state = RoadmapState {
            st: [(
                "st_slo".to_string(),
                vec![
                    "déjà rédigé".to_string(),
                    String::new(),
                    "KPI internes".to_string(),
                ],
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        fill_draft(&mut state, &roadmap, &matrix, &sel);

        let st = state.st.get("st_slo").unwrap();
        assert_eq!(st[0], "déjà rédigé");
        assert_eq!(st[2], "KPI internes");
        assert!(st[1].contains("Alignement tripartite"));

        assert!(state
            .st
            .get("st_obs")
            .is_none_or(|v| v.iter().all(|c| c.is_empty())));
        assert!(state
            .st
            .get("st_toil")
            .is_none_or(|v| v.iter().all(|c| c.is_empty())));
    }

    #[test]
    fn badge_helpers_format_correctly() {
        let (_, matrix) = base();
        assert_eq!(
            level_badge(&matrix, 2, 2).as_deref(),
            Some("Niveau 2 · SLA Contractuels")
        );
        assert_eq!(
            vision_badge(&matrix, 2).as_deref(),
            Some("Niveau 4 · Gouvernance Valeur")
        );
        assert_eq!(
            level_badge(&matrix, 7, 4).as_deref(),
            Some("Niveau 4 · Native & Sobre")
        );
    }
}
