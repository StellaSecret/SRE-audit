use dioxus::prelude::*;
use sre_audit::data;
use sre_audit::i18n;
use sre_audit::models::RoadmapState;
use sre_audit::orgs::OrgStore;
use sre_audit::services::roadmap_gen::{self, GenMode};
use sre_audit::services::{drive, storage};

/// Maximum number of undo snapshots kept in-session.
const UNDO_CAP: usize = 10;

/// Record an undo snapshot before a mutating action, capped to `UNDO_CAP`.
fn push_undo(mut stack: Signal<Vec<RoadmapState>>, before: &RoadmapState) {
    let mut s = stack.write();
    s.push(before.clone());
    if s.len() > UNDO_CAP {
        s.remove(0);
    }
}

/// Ask the user before a destructive action. On native (non-wasm) the
/// dialogs don't exist and the action is simply allowed.
fn confirm_action(message: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.confirm_with_message(message).ok())
            .unwrap_or(false)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = message;
        true
    }
}

/// Row ids of the legacy roadmap cache — ST and LT tables have a fixed order
/// mirroring `src/data/roadmap.json`. Kept here (rather than captured through
/// rsx closures) so event handlers only capture Copy indices.
const ST_IDS: [&str; 7] = [
    "st_obs",
    "st_slo",
    "st_risk",
    "st_toil",
    "st_auto",
    "st_release",
    "st_simp",
];
const LT_IDS: [&str; 7] = [
    "lt_obs",
    "lt_slo",
    "lt_risk",
    "lt_toil",
    "lt_auto",
    "lt_release",
    "lt_simp",
];

fn set_field(
    mut state: Signal<RoadmapState>,
    is_st: bool,
    row: usize,
    col: usize,
    e: Event<FormData>,
) {
    let ids: &[&str] = if is_st { &ST_IDS } else { &LT_IDS };
    let Some(key) = ids.get(row) else { return };
    let val = e.value();
    let mut w = state.write();
    let map = if is_st { &mut w.st } else { &mut w.lt };
    let entry = map.entry((*key).to_string()).or_default();
    if entry.len() <= col {
        entry.resize(col + 1, String::new());
    }
    entry[col] = val.clone();
    sre_audit::services::print::mirror(&format!("sre_road_{key}_{col}"), &val);
}

#[component]
pub fn Roadmap() -> Element {
    let lang: Signal<i18n::Lang> = use_context();
    let l = *lang.read();

    let data = data::roadmap();
    let base_matrix = data::matrix();
    let base_templates = data::roadmap_templates();
    let current = use_context::<Signal<OrgStore>>();
    let org_id = current.read().current.clone();
    let current_id = use_memo(move || current.read().current.clone());
    let mut state = use_signal({
        let org_id = org_id.clone();
        move || storage::load_roadmap(&org_id).unwrap_or_default()
    });
    let matrix_sel = use_memo(move || storage::load_matrix(&current_id()).unwrap_or_default());
    use_effect(move || {
        let id = current_id();
        state.set(storage::load_roadmap(&id).unwrap_or_default());
    });
    // Keep the hidden print mirrors in sync with loaded state (they are only
    // set via set_field on input; values restored from storage or another
    // org would otherwise never make it into the PDF).
    let mirror_keys_raw: Vec<(String, String, usize)> = data
        .st
        .iter()
        .chain(data.lt.iter())
        .flat_map(|row| {
            let k = row.id.clone();
            (0..row.areas.len()).map(move |idx| (format!("sre_road_{k}_{idx}"), k.clone(), idx))
        })
        .collect();
    let mirror_keys = use_memo(move || mirror_keys_raw.clone());
    use_effect(move || {
        let s = state.read();
        for (mirror_id, key, idx) in mirror_keys() {
            sre_audit::services::print::mirror(&mirror_id, s.field(&key, idx));
        }
    });
    let mut flash = use_signal(|| Option::<String>::None);
    let mut busy = use_signal(|| false);
    let mut undo_stack = use_signal(Vec::<RoadmapState>::new);

    let t_title = i18n::tr("roadmap_title", l);
    let t_meta = i18n::tr("roadmap_meta", l);
    let t_save = i18n::tr("save", l);
    let t_pdf = i18n::tr("export_pdf", l);
    let t_export_json = i18n::tr("export_json", l);
    let t_import_json = i18n::tr("import_json", l);
    let t_st_title = i18n::tr("roadmap_st_title", l);
    let t_lt_title = i18n::tr("roadmap_lt_title", l);
    let t_generate = i18n::tr("roadmap_generate", l);
    let t_gen_force = i18n::tr("roadmap_gen_overwrite", l);
    let t_clear = i18n::tr("roadmap_clear", l);
    let t_undo = i18n::tr("roadmap_undo", l);

    let st_badges: Vec<(u8, String)> = data
        .st
        .iter()
        .map(|row| {
            let b = matrix_sel()
                .selected_level(row.num)
                .and_then(|lv| {
                    sre_audit::services::roadmap_gen::level_badge(&base_matrix, row.num, lv)
                })
                .or_else(|| row.badges.first().cloned())
                .unwrap_or_default();
            (row.num, b)
        })
        .collect();
    let lt_visions: Vec<(u8, String)> = data
        .lt
        .iter()
        .map(|row| {
            let v = if row.vision.is_empty() {
                sre_audit::services::roadmap_gen::vision_badge(&base_matrix, row.num)
                    .unwrap_or_default()
            } else {
                row.vision.clone()
            };
            (row.num, v)
        })
        .collect();

    let do_save = move |_: MouseEvent| {
        let id = current.read().current.clone();
        let s = state.read().clone();
        storage::save_roadmap(&id, &s);
        flash.set(Some(i18n::tr("flash_saved", l)));
    };

    let do_export_json = move |_: MouseEvent| {
        let s = state.read().clone();
        let fname = if org_id.is_empty() {
            "sre_roadmap.json".to_string()
        } else {
            format!("sre_roadmap_{org_id}.json")
        };
        drive::local_download(&fname, &drive::build_roadmap_backup(&s));
    };

    let do_import_json = move |e: Event<FormData>| {
        if let Some(file) = e.files().first() {
            let file = file.clone();
            let l = l;
            let mut busy_clone = busy;
            let mut state_clone = state;
            let mut flash_clone = flash;
            spawn(async move {
                let text = match file.read_string().await {
                    Ok(t) => t,
                    Err(_) => {
                        flash_clone.set(Some(i18n::tr("flash_import_err", l)));
                        busy_clone.set(false);
                        return;
                    }
                };
                match drive::restore_roadmap_backup(&text) {
                    Ok(s) => {
                        state_clone.set(s.clone());
                        let id = current.read().current.clone();
                        storage::save_roadmap(&id, &s);
                        flash_clone.set(Some(i18n::tr("flash_import_ok", l)));
                    }
                    Err(_) => {
                        flash_clone.set(Some(i18n::tr("flash_import_err", l)));
                    }
                }
                busy_clone.set(false);
            });
            busy.set(true);
        }
    };

    let do_export_pdf = move |_: MouseEvent| {
        #[cfg(target_arch = "wasm32")]
        if let Some(w) = web_sys::window() {
            let _ = w.print();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = ();
        }
    };

    let do_generate = {
        let roadmap_data = data.clone();
        let matrix_data = base_matrix.clone();
        let templates = base_templates.clone();
        move |_: MouseEvent| {
            let id = current.read().current.clone();
            let Some(sel) = storage::load_matrix(&id) else {
                flash.set(Some(i18n::tr("roadmap_gen_noselection", l)));
                return;
            };
            if sel.selections.is_empty() {
                flash.set(Some(i18n::tr("roadmap_gen_noselection", l)));
                return;
            }
            let mut s = state.read().clone();
            let before = s.clone();
            roadmap_gen::fill_draft(
                &mut s,
                &roadmap_data,
                &matrix_data,
                &sel,
                &templates,
                l,
                GenMode::FillEmpty,
            );
            if s == before {
                return;
            }
            push_undo(undo_stack, &before);
            state.set(s.clone());
            storage::save_roadmap(&id, &s);
            flash.set(Some(i18n::tr("roadmap_gen_ok", l)));
        }
    };

    let do_force_generate = {
        let roadmap_data = data.clone();
        let matrix_data = base_matrix.clone();
        let templates = base_templates.clone();
        move |_: MouseEvent| {
            let id = current.read().current.clone();
            let Some(sel) = storage::load_matrix(&id) else {
                flash.set(Some(i18n::tr("roadmap_gen_noselection", l)));
                return;
            };
            if sel.selections.is_empty() {
                flash.set(Some(i18n::tr("roadmap_gen_noselection", l)));
                return;
            }
            if !confirm_action(&i18n::tr("roadmap_confirm_overwrite", l)) {
                return;
            }
            let mut s = state.read().clone();
            let before = s.clone();
            roadmap_gen::fill_draft(
                &mut s,
                &roadmap_data,
                &matrix_data,
                &sel,
                &templates,
                l,
                GenMode::Overwrite,
            );
            push_undo(undo_stack, &before);
            state.set(s.clone());
            storage::save_roadmap(&id, &s);
            flash.set(Some(i18n::tr("roadmap_gen_forced", l)));
        }
    };

    let do_clear = move |_: MouseEvent| {
        if !confirm_action(&i18n::tr("roadmap_confirm_clear", l)) {
            return;
        }
        let id = current.read().current.clone();
        let mut s = state.read().clone();
        let before = s.clone();
        roadmap_gen::clear(&mut s);
        push_undo(undo_stack, &before);
        state.set(s.clone());
        storage::save_roadmap(&id, &s);
        flash.set(Some(i18n::tr("roadmap_clear_ok", l)));
    };

    let do_undo = move |_: MouseEvent| {
        let Some(previous) = undo_stack.write().pop() else {
            return;
        };
        let id = current.read().current.clone();
        state.set(previous.clone());
        storage::save_roadmap(&id, &previous);
        flash.set(Some(i18n::tr("roadmap_undo_ok", l)));
    };

    rsx! {
        div { class: "container",
            div { class: "header",
                h1 { "{t_title}" }
                div { class: "meta-info", "{t_meta}" }
                div { class: "button-group",
                    button { class: "btn btn-save", onclick: do_save,
                        "{t_save}"
                    }
                    button { class: "btn btn-export", onclick: do_export_pdf,
                        "{t_pdf}"
                    }
                }
            }

            if let Some(f) = flash() {
                div { class: "toolbar-flash", "{f}" }
            }

            div { class: "toolbar",
                button { class: "btn btn-export", onclick: do_export_json, disabled: busy(),
                    "{t_export_json}"
                }
                button { class: "btn btn-export", onclick: do_generate, disabled: busy(),
                    "{t_generate}"
                }
                button { class: "btn btn-export", onclick: do_force_generate, disabled: busy(),
                    "{t_gen_force}"
                }
                button { class: "btn btn-export", onclick: do_clear, disabled: busy(),
                    "{t_clear}"
                }
                button { class: "btn btn-export", onclick: do_undo,
                    disabled: busy() || undo_stack().is_empty(),
                    "{t_undo}"
                }
                input { id: "roadmap-import-input", r#type: "file", accept: ".json",
                    style: "display:none", onchange: do_import_json }
                label { class: "btn btn-export", "for": "roadmap-import-input",
                    "{t_import_json}"
                }
            }

            div { class: "section-title", "{t_st_title}" }
            table {
                thead {
                    tr {
                        th { class: "w-principle", {i18n::tr("roadmap_st_h1", l)} }
                        th { class: "w-horizon", {i18n::tr("roadmap_st_h2", l)} }
                        th { class: "w-actions", {i18n::tr("roadmap_st_h3", l)} }
                        th { class: "w-kpi", {i18n::tr("roadmap_st_h4", l)} }
                    }
                }
                tbody {
                    for (i, row) in data.st.iter().enumerate() {
                        tr {
                            td { class: "w-principle",
                                b { "{row.num}. {row.title}" }
                                br {}
                                small { style: "color:#64748b;", "{row.process}" }
                            }
                            for (idx, _area) in row.areas.iter().enumerate() {
                                td {
                                    if idx == 0 {
                                        if let Some(b) = st_badges.iter().find(|(n, _)| *n == row.num).map(|(_, b)| b.as_str()) {
                                            if !b.is_empty() {
                                                div { class: "horizon-badge badge-st", "{b}" }
                                            }
                                        }
                                    }
                                    textarea {
                                        id: "sre_road_{row.id}_{idx}",
                                        class: "editable-area",
                                        value: state().field(&row.id, idx).to_string(),
                                        oninput: move |e| set_field(state, true, i, idx, e),
                                    }
                                    div { class: "print-text", "data-print-for": "sre_road_{row.id}_{idx}" }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "section-title", "{t_lt_title}" }
            table {
                thead {
                    tr {
                        th { class: "w-principle", {i18n::tr("roadmap_lt_h1", l)} }
                        th { class: "w-horizon", {i18n::tr("roadmap_lt_h2", l)} }
                        th { class: "w-actions", {i18n::tr("roadmap_lt_h3", l)} }
                        th { class: "w-horizon", {i18n::tr("roadmap_lt_h4", l)} }
                    }
                }
                tbody {
                    for (i, row) in data.lt.iter().enumerate() {
                        tr {
                            td { class: "w-principle",
                                b { "{row.num}. {row.title}" }
                                br {}
                                small { style: "color:#64748b;", "{row.process}" }
                            }
                            for (idx, _area) in row.areas.iter().enumerate() {
                                td {
                                    if idx == 0 {
                                        if let Some(b) = row.badges.first() {
                                            div { class: "horizon-badge badge-lt", "{b}" }
                                        }
                                    }
                                    textarea {
                                        id: "sre_road_{row.id}_{idx}",
                                        class: "editable-area",
                                        value: state().field(&row.id, idx).to_string(),
                                        oninput: move |e| set_field(state, false, i, idx, e),
                                    }
                                    div { class: "print-text", "data-print-for": "sre_road_{row.id}_{idx}" }
                                }
                            }
                            td {
                                if let Some(v) = lt_visions.iter().find(|(n, _)| *n == row.num).map(|(_, v)| v.as_str()) {
                                    if !v.is_empty() {
                                        div { class: "horizon-badge badge-lt vision",
                                            "{v}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_tables_match_data() {
        let d = data::roadmap();
        let st: Vec<&str> = d.st.iter().map(|r| r.id.as_str()).collect();
        let lt: Vec<&str> = d.lt.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(st, ST_IDS);
        assert_eq!(lt, LT_IDS);
    }
}
