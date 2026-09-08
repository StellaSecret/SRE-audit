use crate::i18n;
use dioxus::prelude::*;
use sre_audit::orgs::OrgStore;
use sre_audit::services::{drive, storage};

#[component]
pub fn Backup() -> Element {
    let lang: Signal<i18n::Lang> = use_context();
    let orgs_sig: Signal<OrgStore> = use_context();
    let l = *lang.read();

    let t_title = i18n::tr("backup_title", l);
    let t_desc = i18n::tr("backup_desc", l);
    let t_drive_up = i18n::tr("drive_upload", l);
    let t_drive_down = i18n::tr("drive_restore", l);
    let t_hint = i18n::tr("drive_hint", l);

    let mut flash = use_signal(|| Option::<String>::None);
    let busy = use_signal(|| false);

    let do_drive_upload = move |_: MouseEvent| {
        let token = sre_audit::services::auth::get_state().token.clone();
        if token.is_empty() {
            flash.set(Some(i18n::tr("error", l)));
            return;
        }
        let store = orgs_sig.read().clone();
        let mut matrices = std::collections::BTreeMap::new();
        let mut roadmaps = std::collections::BTreeMap::new();
        for org in &store.orgs {
            if let Some(m) = storage::load_matrix(&org.id) {
                matrices.insert(org.id.clone(), m);
            }
            if let Some(r) = storage::load_roadmap(&org.id) {
                roadmaps.insert(org.id.clone(), r);
            }
        }
        let json = drive::build_global_backup(&store, &matrices, &roadmaps);
        let mut busy_clone = busy;
        let mut flash_clone = flash;
        busy_clone.set(true);
        spawn(async move {
            match drive::drive_upload(drive::GLOBAL_BACKUP_FILE, &json, &token).await {
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
        let mut busy_clone = busy;
        let mut flash_clone = flash;
        let mut orgs_clone = orgs_sig;
        busy_clone.set(true);
        spawn(async move {
            let result = async {
                let text = drive::drive_download(drive::GLOBAL_BACKUP_FILE, &token).await?;
                let b = drive::restore_global_backup(&text)?;
                for (id, m) in &b.matrices {
                    storage::save_matrix(id, m);
                }
                for (id, r) in &b.roadmaps {
                    storage::save_roadmap(id, r);
                }
                orgs_clone.set(b.orgs.clone());
                Ok::<(), String>(())
            }
            .await;
            match result {
                Ok(()) => flash_clone.set(Some(i18n::tr("flash_drive_down", l))),
                Err(e) => flash_clone.set(Some(format!("{}: {e}", i18n::tr("error", l)))),
            }
            busy_clone.set(false);
        });
    };

    rsx! {
        div { class: "container",
            div { class: "header",
                h1 { "{t_title}" }
                div { class: "meta-info", "{t_desc}" }
            }
            if let Some(f) = flash() {
                div { class: "toolbar-flash", "{f}" }
            }
            div { class: "toolbar",
                button { class: "btn btn-export", onclick: do_drive_upload, disabled: busy(),
                    "{t_drive_up}"
                }
                button { class: "btn btn-export", onclick: do_drive_restore, disabled: busy(),
                    "{t_drive_down}"
                }
            }
            div { class: "drive-hint", "{t_hint}" }
        }
    }
}
