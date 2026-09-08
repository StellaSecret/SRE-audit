use crate::i18n;
use crate::router::Route;
use dioxus::prelude::*;
use sre_audit::orgs::{OrgStore, Organization};
use sre_audit::services::auth;

#[component]
pub fn Home() -> Element {
    let lang: Signal<i18n::Lang> = use_context();
    let auth_sig: Signal<auth::AuthState> = use_context();
    let mut orgs_sig: Signal<OrgStore> = use_context();
    let l = *lang.read();
    let a = auth_sig.read().clone();

    let email = a
        .profile
        .as_ref()
        .map(|p| p.email.clone())
        .unwrap_or_default();
    let name = a
        .profile
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_default();

    let t_title = i18n::tr("home_title", l);
    let t_subtitle = i18n::tr("home_subtitle", l);
    let t_welcome = i18n::tr("home_welcome", l);
    let t_orgs = i18n::tr("home_orgs", l);
    let t_add = i18n::tr("org_add", l);
    let t_add_ph = i18n::tr("org_add_ph", l);

    let mut new_name = use_signal(String::new);

    let do_add = move |_: MouseEvent| {
        let name = new_name();
        orgs_sig.write().add(&name);
        new_name.set(String::new());
    };

    rsx! {
        div { class: "page-header",
            h1 { "{t_title}" }
            div { class: "subtitle", "{t_subtitle}" }
        }
        div { class: "welcome",
            "{t_welcome}, "
            b { "{name}" }
            " · {email}"
        }
        div { class: "orgs-section",
            h2 { "{t_orgs}" }
            div { class: "org-list",
                for org in orgs_sig.read().orgs.clone() {
                    OrgCard {
                        org: org.clone(),
                        current: org.id == orgs_sig.read().current,
                    }
                }
            }
            div { class: "org-add",
                input {
                    class: "org-add-input",
                    placeholder: t_add_ph,
                    value: new_name(),
                    oninput: move |e: Event<FormData>| new_name.set(e.value()),
                }
                button { class: "btn btn-primary", onclick: do_add, "{t_add}" }
            }
        }
    }
}

#[component]
fn OrgCard(org: Organization, current: bool) -> Element {
    let lang: Signal<i18n::Lang> = use_context();
    let mut orgs_sig: Signal<OrgStore> = use_context();
    let l = *lang.read();

    let t_current = i18n::tr("org_current", l);
    let t_name = i18n::tr("org_name_label", l);
    let t_matrix = i18n::tr("home_matrix_btn", l);
    let t_roadmap = i18n::tr("home_roadmap_btn", l);
    let t_delete = i18n::tr("org_delete", l);

    let id = org.id.clone();
    let org_name = org.name.clone();

    let switch_a_id = id.clone();
    let switch_a = move |_: MouseEvent| {
        orgs_sig.write().current = switch_a_id.clone();
    };
    let switch_b_id = id.clone();
    let switch_b = move |_: MouseEvent| {
        orgs_sig.write().current = switch_b_id.clone();
    };

    let rename_id = id.clone();
    let rename = move |e: Event<FormData>| {
        let _ = orgs_sig.write().rename_org(&rename_id, &e.value());
    };

    let delete_id = id.clone();
    let delete = move |_: MouseEvent| {
        orgs_sig.write().remove(&delete_id);
    };

    rsx! {
        div { class: "card org-card",
            div { class: "org-card-head",
                b { "{org_name}" }
                if current {
                    span { class: "org-current-badge", "{t_current}" }
                }
            }
            div { class: "org-card-field",
                label { class: "org-name-label", "{t_name}" }
                input {
                    class: "org-name-input",
                    value: "{org_name}",
                    oninput: rename,
                }
            }
            div { class: "org-card-actions",
                Link {
                    to: Route::Matrix {},
                    class: "btn btn-primary",
                    onclick: switch_a,
                    "{t_matrix}"
                }
                Link {
                    to: Route::Roadmap {},
                    class: "btn btn-primary",
                    onclick: switch_b,
                    "{t_roadmap}"
                }
                button { class: "btn btn-danger", onclick: delete, "{t_delete}" }
            }
        }
    }
}
