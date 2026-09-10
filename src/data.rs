use crate::models::{MatrixData, RoadmapData, RoadmapTemplates};

// Static content extracted from the legacy HTML pages.
// Embedded at compile time; deserialised once at app init.

/// Static SRE maturity matrix content.
pub const MATRIX_JSON: &str = include_str!("data/matrix.json");

/// Static SRE roadmap content.
pub const ROADMAP_JSON: &str = include_str!("data/roadmap.json");

/// Bilingual roadmap generation template bank.
pub const ROADMAP_GEN_JSON: &str = include_str!("data/roadmap_gen.json");

pub fn matrix() -> MatrixData {
    serde_json::from_str(MATRIX_JSON).expect("embedded matrix.json is valid")
}

pub fn roadmap() -> RoadmapData {
    serde_json::from_str(ROADMAP_JSON).expect("embedded roadmap.json is valid")
}

pub fn roadmap_templates() -> RoadmapTemplates {
    serde_json::from_str(ROADMAP_GEN_JSON).expect("embedded roadmap_gen.json is valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_has_7_rows_with_4_levels() {
        let m = matrix();
        assert_eq!(m.rows.len(), 7);
        for row in &m.rows {
            assert_eq!(row.levels.len(), 4, "row {} must have 4 levels", row.id);
            assert!(!row.title.is_empty());
            assert!(!row.process.is_empty());
            for lvl in &row.levels {
                assert!(!lvl.badge.is_empty());
                assert!(!lvl.items.is_empty());
            }
        }
    }

    #[test]
    fn roadmap_has_7_rows_each_side() {
        let r = roadmap();
        assert_eq!(r.st.len(), 7);
        assert_eq!(r.lt.len(), 7);
        assert_eq!(r.st[0].areas.len(), 3, "ST rows have 3 editables");
        assert_eq!(r.lt[0].areas.len(), 2, "LT rows have 2 editables");
        for (st, lt) in r.st.iter().zip(r.lt.iter()) {
            assert_eq!(st.num, lt.num);
            assert!(st.id.starts_with("st_"), "ST id: {}", st.id);
            assert!(lt.id.starts_with("lt_"), "LT id: {}", lt.id);
        }
    }

    #[test]
    fn embedded_json_is_valid() {
        serde_json::from_str::<serde_json::Value>(MATRIX_JSON).unwrap();
        serde_json::from_str::<serde_json::Value>(ROADMAP_JSON).unwrap();
        serde_json::from_str::<serde_json::Value>(ROADMAP_GEN_JSON).unwrap();
    }

    #[test]
    fn roadmap_templates_cover_7_principles_4_levels() {
        let t = roadmap_templates();
        for num in 1..=7 {
            let n = num.to_string();
            for lvl in 1..=4 {
                let l = lvl.to_string();
                assert!(t.constat[&n].contains_key(&l), "constat missing {n}/{l}");
                assert!(
                    t.st_actions[&n].contains_key(&l) && !t.st_actions[&n][&l].fr.is_empty(),
                    "st_actions missing {n}/{l}"
                );
                assert!(
                    t.st_kpi[&n].contains_key(&l) && !t.st_kpi[&n][&l].en.is_empty(),
                    "st_kpi missing {n}/{l}"
                );
                assert!(t.lt_year1[&n].contains_key(&l), "lt_year1 missing {n}/{l}");
                assert!(
                    t.lt_transform[&n].contains_key(&l),
                    "lt_transform missing {n}/{l}"
                );
            }
        }
    }

    #[test]
    fn roadmap_templates_are_bilingual() {
        let t = roadmap_templates();
        for cell in t.constat.values().flat_map(|lv| lv.values()) {
            assert!(!cell.fr.is_empty() && !cell.en.is_empty());
        }
        for lines in t
            .st_actions
            .values()
            .flat_map(|lv| lv.values())
            .chain(t.st_kpi.values().flat_map(|lv| lv.values()))
        {
            assert!(!lines.fr.is_empty() && !lines.en.is_empty());
        }
    }
}
