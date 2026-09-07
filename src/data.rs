use crate::models::{MatrixData, RoadmapData};

// Static content extracted from the legacy HTML pages.
// Embedded at compile time; deserialised once at app init.

/// Static SRE maturity matrix content.
pub const MATRIX_JSON: &str = include_str!("data/matrix.json");

/// Static SRE roadmap content.
pub const ROADMAP_JSON: &str = include_str!("data/roadmap.json");

pub fn matrix() -> MatrixData {
    serde_json::from_str(MATRIX_JSON).expect("embedded matrix.json is valid")
}

pub fn roadmap() -> RoadmapData {
    serde_json::from_str(ROADMAP_JSON).expect("embedded roadmap.json is valid")
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
    }
}
