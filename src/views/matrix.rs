use crate::i18n;
use dioxus::prelude::*;
use sre_audit::data;
use sre_audit::models::MatrixState;
use sre_audit::orgs::OrgStore;
use sre_audit::services::{drive, print, storage};

fn select_level(mut state: Signal<MatrixState>, row_id: u8, level: u8) {
    state
        .write()
        .selections
        .insert(row_id.to_string(), level.to_string());
}

fn set_comment(mut state: Signal<MatrixState>, row_id: u8, e: Event<FormData>) {
    let val = e.value();
    state
        .write()
        .comments
        .insert(row_id.to_string(), val.clone());
    sre_audit::services::print::mirror(&format!("sre_matrix_c_{row_id}"), &val);
}

#[component]
pub fn Matrix() -> Element {
    let lang: Signal<i18n::Lang> = use_context();
    let l = *lang.read();

    let data = data::matrix();
    let rows_ids_raw: Vec<u8> = data.rows.iter().map(|r| r.id).collect();
    let row_ids = use_memo(move || rows_ids_raw.clone());
    let mut current = use_context::<Signal<OrgStore>>();
    let org_id = current.read().current.clone();
    let current_id = use_memo(move || current.read().current.clone());
    let org_name = current
        .read()
        .current_org()
        .map(|o| o.name.clone())
        .unwrap_or_default();
    let mut state = use_signal({
        let org_id = org_id.clone();
        move || storage::load_matrix(&org_id).unwrap_or_default()
    });
    use_effect(move || {
        let id = current_id();
        state.set(storage::load_matrix(&id).unwrap_or_default());
    });
    // Keep the hidden print mirrors in sync with loaded state (they are only
    // set via set_comment on input; values restored from storage or another
    // org would otherwise never make it into the PDF).
    let comment_ids_raw: Vec<(String, String)> = data
        .rows
        .iter()
        .map(|r| (format!("sre_matrix_c_{}", r.id), r.id.to_string()))
        .collect();
    let comment_ids = use_memo(move || comment_ids_raw.clone());
    use_effect(move || {
        let s = state.read();
        for (mirror_id, row_id) in comment_ids() {
            let val = s.comments.get(&row_id).map(String::as_str).unwrap_or("");
            print::mirror(&mirror_id, val);
        }
    });
    let mut flash = use_signal(|| Option::<String>::None);
    let mut busy = use_signal(|| false);

    let t_title = i18n::tr("matrix_title", l);
    let t_company = i18n::tr("matrix_company", l);
    let t_company_ph = i18n::tr("matrix_company_ph", l);
    let t_save = i18n::tr("save", l);
    let t_pdf = i18n::tr("export_pdf", l);
    let t_desc = i18n::tr("matrix_desc", l);
    let t_instructions = i18n::tr("matrix_instructions", l);
    let t_export_json = i18n::tr("export_json", l);
    let t_import_json = i18n::tr("import_json", l);
    let t_drive_up = i18n::tr("drive_upload", l);
    let t_drive_down = i18n::tr("drive_restore", l);
    let t_hint = i18n::tr("drive_hint", l);
    let t_comment_ph = i18n::tr("matrix_comment_ph", l);

    let org_id_rename = org_id.clone();
    let set_company = move |e: Event<FormData>| {
        let val = e.value();
        state.write().company_name = val.clone();
        current.write().rename_org(&org_id_rename, &val);
    };

    let do_save = move |_: MouseEvent| {
        let id = current.read().current.clone();
        {
            let s = state.read().clone();
            storage::save_matrix(&id, &s);
        }
        flash.set(Some(i18n::tr("flash_saved", l)));
    };

    let do_export_json = move |_: MouseEvent| {
        let s = state.read().clone();
        let fname = if org_id.is_empty() {
            "sre_matrix_audit.json".to_string()
        } else {
            format!("sre_matrix_audit_{org_id}.json")
        };
        drive::local_download(&fname, &drive::build_matrix_backup(&s));
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
                match drive::restore_matrix_backup(&text) {
                    Ok(s) => {
                        state_clone.set(s.clone());
                        let id = current.read().current.clone();
                        storage::save_matrix(&id, &s);
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

    let do_drive_upload = move |_: MouseEvent| {
        let token = sre_audit::services::auth::get_state().token.clone();
        if token.is_empty() {
            flash.set(Some(i18n::tr("error", l)));
            return;
        }
        let s = state.read().clone();
        let json = drive::build_matrix_backup(&s);
        let id = current.read().current.clone();
        let fname = drive::matrix_file(&id);
        let mut busy_clone = busy;
        let mut flash_clone = flash;
        busy_clone.set(true);
        spawn(async move {
            match drive::drive_upload(&fname, &json, &token).await {
                Ok(_) => flash_clone.set(Some(i18n::tr("flash_drive_up", l))),
                Err(e) => flash_clone.set(Some(format!("{}: {e}", i18n::tr("error", l)))),
            }
            busy_clone.set(false);
        });
    };

    let do_drive_restore = move |_: MouseEvent| {
        let token = sre_audit::services::auth::get_state().token.clone();
        if token.is_empty() {
            flash.set(Some(i18n::tr("error", l)));
            return;
        }
        let id = current.read().current.clone();
        let fname = drive::matrix_file(&id);
        let mut busy_clone = busy;
        let mut state_clone = state;
        let mut flash_clone = flash;
        busy_clone.set(true);
        spawn(async move {
            match drive::drive_download(&fname, &token).await {
                Ok(text) => match drive::restore_matrix_backup(&text) {
                    Ok(s) => {
                        state_clone.set(s.clone());
                        storage::save_matrix(&id, &s);
                        flash_clone.set(Some(i18n::tr("flash_drive_down", l)));
                    }
                    Err(e) => flash_clone.set(Some(format!("{}: {e}", i18n::tr("error", l)))),
                },
                Err(e) => flash_clone.set(Some(format!("{}: {e}", i18n::tr("error", l)))),
            }
            busy_clone.set(false);
        });
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

    rsx! {
        div { class: "matrix-container",
            div { class: "header-actions",
                div { class: "title-area",
                    h1 { "{t_title}" }
                    div { class: "company-input-wrapper",
                        span { class: "company-label", "{t_company}" }
                        input {
                            class: "company-input", r#type: "text",
                            placeholder: t_company_ph,
                            value: org_name.clone(),
                            oninput: set_company,
                        }
                    }
                }
                div { class: "button-group",
                    button { class: "btn btn-save", onclick: do_save,
                        "{t_save}"
                    }
                    button { class: "btn btn-export", onclick: do_export_pdf,
                        "{t_pdf}"
                    }
                }
            }

            div { class: "description", "{t_desc}" }
            div { class: "instructions",
                span { "{t_instructions}" }
            }

            if let Some(f) = flash() {
                div { class: "toolbar-flash", "{f}" }
            }

            div { class: "toolbar",
                button { class: "btn btn-export", onclick: do_export_json, disabled: busy(),
                    "{t_export_json}"
                }
                input {
                    id: "matrix-import-input", r#type: "file", accept: ".json",
                    style: "display:none",
                    onchange: do_import_json,
                }
                label { class: "btn btn-export", "for": "matrix-import-input",
                    "{t_import_json}"
                }
                button { class: "btn btn-export", onclick: do_drive_upload, disabled: busy(),
                    "{t_drive_up}"
                }
                button { class: "btn btn-export", onclick: do_drive_restore, disabled: busy(),
                    "{t_drive_down}"
                }
            }

            div { class: "drive-hint", "{t_hint}" }

            table {
                thead {
                    tr {
                        th { class: "col-principle", {i18n::tr("matrix_col1", l)} }
                        th { class: "col-lvl1", {i18n::tr("matrix_col2", l)} }
                        th { class: "col-lvl2", {i18n::tr("matrix_col3", l)} }
                        th { class: "col-lvl3", {i18n::tr("matrix_col4", l)} }
                        th { class: "col-lvl4", {i18n::tr("matrix_col5", l)} }
                        th { class: "col-comment", {i18n::tr("matrix_comment", l)} }
                    }
                }
                tbody {
                    for (ri, row) in data.rows.iter().enumerate() {
                        tr {
                            td { class: "col-principle",
                                b { "{row.num}. {row.title}" }
                                br {}
                                br {}
                                span { class: "aspect-tag", "{row.aspect}" }
                                br {}
                                span { class: "process-tag", "{row.process}" }
                            }
                            for (lvl, li) in row.levels.iter().zip(1u8..) {
                                td {
                                    class: if state.read().selected_level(row.id) == Some(li) {
                                        format!("col-lvl{} selected", li)
                                    } else {
                                        format!("col-lvl{}", li)
                                    },
                                    onclick: move |_| select_level(state, row_ids.read()[ri], li),
                                    span { class: "badge {lvl.badge_class()}", "{lvl.badge}" }
                                    ul {
                                        for it in &lvl.items {
                                            li {
                                                b { "{it.label} : " }
                                                "{it.text}"
                                            }
                                        }
                                    }
                                }
                            }
                            td { class: "col-comment",
                                div { class: "comment-container",
                                    textarea {
                                        id: "sre_matrix_c_{row.id}",
                                        class: "comment-box",
                                        placeholder: t_comment_ph.clone(),
                                        value: state.read().comments.get(&row.id.to_string()).cloned().unwrap_or_default(),
                                        oninput: move |e| set_comment(state, row_ids.read()[ri], e),
                                    }
                                    div { class: "comment-print-output", "data-print-for": "sre_matrix_c_{row.id}" }
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
    fn levels_are_one_to_four_in_order() {
        let d = data::matrix();
        for row in &d.rows {
            let got: Vec<u8> = row.levels.iter().map(|l| l.level).collect();
            assert_eq!(got, vec![1, 2, 3, 4], "row {}", row.num);
        }
    }
}
