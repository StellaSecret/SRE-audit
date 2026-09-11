// ── Roadmap generation from the matrix ───────────────────────────────────────
// Pure, wasm-free helpers: derive horizon badges from the org's matrix
// selection and draft ST/LT cell content from the bilingual template bank
// (`src/data/roadmap_gen.json`, curated to match hand-written roadmap quality).
// The Constat is distilled from the bank, enriched with the tools named in the
// auditor's note, and the full note is appended verbatim for traceability.
// Only ever fills empty cells, so hand-written roadmap content is never
// overwritten.

use std::collections::{BTreeMap, BTreeSet};

use crate::i18n::{self, Lang};
use crate::models::{
    LangLines, LangStrings, MatrixData, MatrixLevel, MatrixRow, MatrixState, RoadmapData,
    RoadmapState, RoadmapTemplates,
};

fn matrix_row(data: &MatrixData, num: u8) -> Option<&MatrixRow> {
    data.rows.iter().find(|r| r.num == num)
}

/// How the generator writes draft cells: only empty ones (default, keeps the
/// auditor's manual edits) or overwriting everything for the selected rows.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GenMode {
    #[default]
    FillEmpty,
    Overwrite,
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

fn fill_cell(
    map: &mut std::collections::BTreeMap<String, Vec<String>>,
    key: &str,
    idx: usize,
    value: &str,
    force: bool,
) {
    let entry = map.entry(key.to_string()).or_default();
    if entry.len() <= idx {
        entry.resize(idx + 1, String::new());
    }
    if force || entry[idx].is_empty() {
        entry[idx] = value.to_string();
    }
}

fn pick(s: &LangStrings, lang: Lang) -> String {
    match lang {
        Lang::Fr => s.fr.clone(),
        Lang::En => s.en.clone(),
    }
}

fn pick_lines(l: &LangLines, lang: Lang) -> Vec<String> {
    match lang {
        Lang::Fr => l.fr.clone(),
        Lang::En => l.en.clone(),
    }
}

fn tpl_string(
    tpl: &BTreeMap<String, BTreeMap<String, LangStrings>>,
    num: u8,
    level: u8,
    lang: Lang,
) -> Option<String> {
    tpl.get(&num.to_string())?
        .get(&level.to_string())
        .map(|e| pick(e, lang))
}

fn tpl_lines(
    tpl: &BTreeMap<String, BTreeMap<String, LangLines>>,
    num: u8,
    level: u8,
    lang: Lang,
) -> Option<Vec<String>> {
    tpl.get(&num.to_string())?
        .get(&level.to_string())
        .map(|e| pick_lines(e, lang))
}

/// Token match with word boundaries, so short needles ("sql", "cft", "elk")
/// never match inside longer words ("MySQL", "postgresql").
fn contains_token(lower: &str, needle: &str) -> bool {
    // Iterate (never hand-rolled pointers) so mutating an operator cannot
    // spin a `start = .. + 1` loop forever under mutation testing.
    lower.match_indices(needle).any(|(i, m)| {
        let before_ok = i == 0 || !lower[..i].chars().last().unwrap().is_alphanumeric();
        let after = i + m.len();
        let after_ok =
            after >= lower.len() || !lower[after..].chars().next().unwrap().is_alphanumeric();
        before_ok && after_ok
    })
}

/// Tool names mentioned in the note, canonicalised and de-duplicated, keeping
/// the reference order of the dictionary. Used to make the generated content
/// organisation-specific.
fn detect_tools(comment: &str) -> Vec<String> {
    const DICT: &[(&str, &str)] = &[
        ("dynatrace", "Dynatrace"),
        ("zabbix", "Zabbix"),
        ("grafana", "Grafana"),
        ("liora", "Liora"),
        ("batch-tracker", "Batch-tracker"),
        ("absana", "Absana"),
        ("elk", "ELK"),
        ("jira", "Jira"),
        ("confluence", "Confluence"),
        ("sharepoint", "SharePoint"),
        ("gitlab-ci", "GitLab CI"),
        ("gitlab", "GitLab CI"),
        ("jenkins", "Jenkins"),
        ("awx", "Ansible AWX"),
        ("ansible", "Ansible"),
        ("terraform", "Terraform"),
        ("kubernetes", "Kubernetes"),
        ("k8s", "Kubernetes"),
        ("pssit", "PSSIT"),
        ("postgres", "Postgres"),
        ("oracle", "Oracle"),
        ("mysql", "MySQL"),
        ("sql", "SQL"),
        ("db2", "DB2"),
        ("cft", "CFT"),
        ("backstage", "Backstage"),
    ];
    let lower = comment.to_lowercase();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut out = Vec::new();
    for (needle, canon) in DICT {
        if contains_token(&lower, needle) && seen.insert(canon) {
            out.push(canon.to_string());
        }
    }
    out
}

/// Distilled Constat: the curated bank sentence for the selected level,
/// organisation-specific tools injected/reported, and the full audit note
/// appended verbatim.
fn build_constat(
    templates: &RoadmapTemplates,
    num: u8,
    level: u8,
    lang: Lang,
    comment: &str,
) -> String {
    let mut s = tpl_string(&templates.constat, num, level, lang).unwrap_or_default();
    let tools = detect_tools(comment);
    if !tools.is_empty() {
        let joined = tools.join(", ");
        if s.contains("{tools}") {
            s = s.replace("{tools}", &joined);
        } else {
            s.push_str(&format!(
                "\n{} : {joined}.",
                i18n::tr("roadmap_tools_label", lang)
            ));
        }
    }
    if !comment.trim().is_empty() {
        s.push_str(&format!(
            "\n{} : {comment}",
            i18n::tr("roadmap_note_label", lang)
        ));
    }
    s
}

/// Draft the roadmap for the current matrix selection. Rows without a
/// selected level are left untouched; with `GenMode::FillEmpty` existing cell
/// content is preserved, with `GenMode::Overwrite` every cell is replaced.
pub fn fill_draft(
    state: &mut RoadmapState,
    roadmap: &RoadmapData,
    matrix: &MatrixData,
    sel: &MatrixState,
    templates: &RoadmapTemplates,
    lang: Lang,
    gen_mode: GenMode,
) {
    let force = gen_mode == GenMode::Overwrite;

    for st_row in &roadmap.st {
        let Some(level) = sel.selected_level(st_row.num) else {
            continue;
        };
        let Some(_) = matrix_row(matrix, st_row.num) else {
            continue;
        };
        let comment = sel
            .comments
            .get(&st_row.num.to_string())
            .map(String::as_str)
            .unwrap_or("");

        fill_cell(
            &mut state.st,
            &st_row.id,
            0,
            &build_constat(templates, st_row.num, level, lang, comment),
            force,
        );
        fill_cell(
            &mut state.st,
            &st_row.id,
            1,
            &tpl_lines(&templates.st_actions, st_row.num, level, lang)
                .unwrap_or_default()
                .join("\n"),
            force,
        );
        fill_cell(
            &mut state.st,
            &st_row.id,
            2,
            &tpl_lines(&templates.st_kpi, st_row.num, level, lang)
                .unwrap_or_default()
                .join("\n"),
            force,
        );
    }

    for lt_row in &roadmap.lt {
        let Some(level) = sel.selected_level(lt_row.num) else {
            continue;
        };
        let Some(_) = matrix_row(matrix, lt_row.num) else {
            continue;
        };

        fill_cell(
            &mut state.lt,
            &lt_row.id,
            0,
            &tpl_string(&templates.lt_year1, lt_row.num, level, lang).unwrap_or_default(),
            force,
        );
        fill_cell(
            &mut state.lt,
            &lt_row.id,
            1,
            &tpl_string(&templates.lt_transform, lt_row.num, level, lang).unwrap_or_default(),
            force,
        );
    }
}

/// Empty every roadmap cell (all rows, both horizons), keeping the layout.
pub fn clear(state: &mut RoadmapState) {
    for v in state.st.values_mut() {
        v.fill(String::new());
    }
    for v in state.lt.values_mut() {
        v.fill(String::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data;

    fn base() -> (RoadmapData, MatrixData, RoadmapTemplates) {
        (data::roadmap(), data::matrix(), data::roadmap_templates())
    }

    #[test]
    fn contains_token_respects_word_boundaries() {
        let lower = "postgresql et mysql, gitlab-ci et elkgraph";
        assert!(!contains_token(lower, "sql"), "inside postgresql");
        assert!(contains_token(lower, "mysql"), "standalone keyword");
        assert!(contains_token(lower, "gitlab-ci"), "hyphenated token");
        assert!(!contains_token(lower, "elk"), "inside elkgraph");
        assert!(contains_token(lower, "et"), "standalone short token");
        assert!(contains_token("jenkins", "jenkins"), "string equals token");
        assert!(contains_token("", ""), "empty needle terminates");
    }

    #[test]
    fn level_two_fills_all_st_lt_cells() {
        let (roadmap, matrix, templates) = base();
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
        fill_draft(
            &mut state,
            &roadmap,
            &matrix,
            &sel,
            &templates,
            Lang::Fr,
            GenMode::FillEmpty,
        );

        let st = state.st.get("st_slo").unwrap();
        assert!(st[0].contains("Note d'audit : Des SLA purement contractuels."));
        assert!(st[0].contains("déconnectés de l'expérience réellement vécue"));
        assert!(st[1].contains("- M1-M2 : Co-concevoir avec le métier"));
        assert!(st[2].contains("3 conventions de SLO métier validées"));

        let lt = state.lt.get("lt_slo").unwrap();
        assert!(lt[0].contains("SLO métier intégrés et visibles en temps réel"));
        assert!(lt[1]
            .to_lowercase()
            .contains("arbitrage algorithmique partagé"));
    }

    #[test]
    fn level_four_uses_consolidate_bank_entries() {
        let (roadmap, matrix, templates) = base();
        let sel = MatrixState {
            selections: [("7".to_string(), "4".to_string())].into_iter().collect(),
            ..Default::default()
        };
        let mut state = RoadmapState::default();
        fill_draft(
            &mut state,
            &roadmap,
            &matrix,
            &sel,
            &templates,
            Lang::Fr,
            GenMode::FillEmpty,
        );

        let st = state.st.get("st_simp").unwrap();
        assert!(st[0].contains("Architecture native et sobre"));
        assert!(!st[0].contains("Note d'audit"));
        assert!(st[1].contains("- M1 : Auditer le taux de redondance restant"));
        assert!(st[2].contains("Taux de redondance résiduelle par domaine"));

        let lt = state.lt.get("lt_simp").unwrap();
        assert!(lt[0].contains("Voie Pavée"));
        assert!(lt[1].contains("bascules inter-régions sans interruption"));
    }

    #[test]
    fn preserves_manual_text_and_skips_unselected() {
        let (roadmap, matrix, templates) = base();
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
        fill_draft(
            &mut state,
            &roadmap,
            &matrix,
            &sel,
            &templates,
            Lang::Fr,
            GenMode::FillEmpty,
        );

        let st = state.st.get("st_slo").unwrap();
        assert_eq!(st[0], "déjà rédigé");
        assert_eq!(st[2], "KPI internes");
        assert!(st[1].contains("- M1-M2 : Co-concevoir avec le métier"));

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
    fn overwrite_mode_replaces_existing_cells() {
        let (roadmap, matrix, templates) = base();
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
        let mut state = RoadmapState {
            st: [(
                "st_slo".to_string(),
                vec![
                    "contenu manuel".to_string(),
                    String::new(),
                    "KPI internes".to_string(),
                ],
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        fill_draft(
            &mut state,
            &roadmap,
            &matrix,
            &sel,
            &templates,
            Lang::Fr,
            GenMode::Overwrite,
        );

        let st = state.st.get("st_slo").unwrap();
        assert!(st[0].contains("Note d'audit : Des SLA purement contractuels."));
        assert!(st[1].contains("- M1-M2 : Co-concevoir avec le métier"));
        assert!(st[2].contains("3 conventions de SLO métier validées"));
        assert!(state
            .st
            .get("st_obs")
            .is_none_or(|v| v.iter().all(|c| c.is_empty())));
    }

    #[test]
    fn clear_empties_every_cell() {
        let (_, _, _) = base();
        let mut state = RoadmapState {
            st: [(
                "st_slo".to_string(),
                vec!["a".to_string(), "b".to_string(), "c".to_string()],
            )]
            .into_iter()
            .collect(),
            lt: [("lt_slo".to_string(), vec!["x".to_string(), "y".to_string()])]
                .into_iter()
                .collect(),
        };
        assert_eq!(state.field("st_slo", 0), "a");
        clear(&mut state);
        assert_eq!(state.field("st_slo", 0), "");
        assert_eq!(state.field("st_slo", 2), "");
        assert_eq!(state.field("lt_slo", 1), "");
        assert!(state.st.iter().all(|(_, v)| v.iter().all(|c| c.is_empty())));
    }

    #[test]
    fn badge_helpers_format_correctly() {
        let (_, matrix, _) = base();
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

    #[test]
    fn detects_tools_in_comment() {
        assert_eq!(
            detect_tools("Alertes Dynatrace, supervision Zabbix et logs ELK"),
            vec!["Dynatrace", "Zabbix", "ELK"]
        );
        assert_eq!(
            detect_tools("Gitlab-CI et Jenkins, CMDB maintenue à la main"),
            vec!["GitLab CI", "Jenkins"]
        );
        assert!(detect_tools("aucun outil cité").is_empty());
    }

    #[test]
    fn injects_tools_into_curated_constat_placeholder() {
        assert_eq!(
            build_constat(
                &data::roadmap_templates(),
                7,
                2,
                Lang::Fr,
                "Prolifération : Postgres, Oracle, MySQL.",
            ),
            "Inventaires semi-automatiques et début de cartographie, mais exceptions locales persistantes (Postgres, Oracle, MySQL) ; failover encore manuel.\nNote d'audit : Prolifération : Postgres, Oracle, MySQL."
        );
    }

    #[test]
    fn golden_fixture_matches_hand_curated_quality() {
        let (roadmap, matrix, templates) = base();
        // CA-GIP audit: selections + the two tool-rich notes from the golden
        // export (MaturityMatrix.json).
        let sel = MatrixState {
            selections: [
                ("1".to_string(), "2".to_string()),
                ("4".to_string(), "1".to_string()),
                ("7".to_string(), "2".to_string()),
            ]
            .into_iter()
            .collect(),
            comments: [
                (
                    "1".to_string(),
                    "Alertes Dynatrace routées automatiquement vers les bonnes \
                     équipes mais multiplication des outils de supervision \
                     (Dynatrace, Zabbix, Grafana, Liora, Batch-tracker) avec \
                     leur lot d'alertes et notifications intempestives. \
                     Tous les logs sont accessibles via ELK."
                        .to_string(),
                ),
                (
                    "4".to_string(),
                    "Patchs postgres PSSIT exécutés manuellement, requêtes SQL \
                     récurrentes, astreintes fréquentes."
                        .to_string(),
                ),
                (
                    "7".to_string(),
                    "La CMDB est maintenue par les équipes via modification \
                     manuelle. Multiplication d'outils répondant au même \
                     besoin (Postgres, Oracle, MySQL, Gitlab-CI, Jenkins)."
                        .to_string(),
                ),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        };

        for lang in [Lang::Fr, Lang::En] {
            let mut state = RoadmapState::default();
            fill_draft(
                &mut state,
                &roadmap,
                &matrix,
                &sel,
                &templates,
                lang,
                GenMode::FillEmpty,
            );

            let note_label = i18n::tr("roadmap_note_label", lang);
            let tools_label = i18n::tr("roadmap_tools_label", lang);

            let obs = state.st.get("st_obs").unwrap();
            assert!(obs[0].contains(note_label.as_str()), "{lang:?}: audit note");
            assert!(obs[0].contains("Dynatrace"), "{lang:?}: tools injected");
            assert!(obs[0].contains("Zabbix"), "{lang:?}: tools injected");
            assert!(obs[1].contains("- M1"), "{lang:?}: phased actions");
            assert!(obs[2].contains("30%"), "{lang:?}: measurable KPI");
            assert!(
                !obs[2].contains("Atteindre Niveau"),
                "{lang:?}: no fake KPI"
            );
            assert!(
                state.lt.get("lt_obs").unwrap()[0]
                    .to_lowercase()
                    .contains(match lang {
                        Lang::Fr => "routage automatique des alertes",
                        Lang::En => "automatic alert routing",
                    }),
                "{lang:?}: consolidated year-1 target"
            );

            let simp = state.st.get("st_simp").unwrap();
            assert!(
                !simp[0].contains("{tools}"),
                "{lang:?}: placeholder resolved"
            );
            assert!(
                simp[0].contains("Postgres") && simp[0].contains("Jenkins"),
                "{lang:?}: org-specific tools injected\n{}",
                simp[0]
            );

            let toil = state.st.get("st_toil").unwrap();
            assert!(
                toil[0].contains(tools_label.as_str()),
                "{lang:?}: tools reported {}",
                toil[0]
            );
            assert!(
                toil[0].contains("PSSIT") && toil[0].contains("SQL"),
                "{lang:?}: toil tools listed"
            );
            assert!(
                state.lt.get("lt_simp").unwrap()[1].contains({
                    match lang {
                        Lang::Fr => "bascules",
                        Lang::En => "failover",
                    }
                }),
                "{lang:?}: major transformation"
            );
        }
    }
}
