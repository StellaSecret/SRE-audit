use crate::i18n;
use crate::router::Route;
use dioxus::prelude::*;
use sre_audit::services::auth;

#[component]
pub fn Home() -> Element {
    let lang: Signal<i18n::Lang> = use_context();
    let auth_sig: Signal<auth::AuthState> = use_context();
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
    let t_matrix_title = i18n::tr("home_matrix_title", l);
    let t_matrix_desc = i18n::tr("home_matrix_desc", l);
    let t_matrix_btn = i18n::tr("home_matrix_btn", l);
    let t_roadmap_title = i18n::tr("home_roadmap_title", l);
    let t_roadmap_desc = i18n::tr("home_roadmap_desc", l);
    let t_roadmap_btn = i18n::tr("home_roadmap_btn", l);

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
        div { class: "cards",
            Link { to: Route::Matrix {}, class: "card",
                div { class: "card-icon", "📊" }
                h2 { "{t_matrix_title}" }
                p { "{t_matrix_desc}" }
                span { class: "btn btn-primary", "{t_matrix_btn}" }
            }
            Link { to: Route::Roadmap {}, class: "card",
                div { class: "card-icon", "🗺️" }
                h2 { "{t_roadmap_title}" }
                p { "{t_roadmap_desc}" }
                span { class: "btn btn-primary", "{t_roadmap_btn}" }
            }
        }
    }
}
