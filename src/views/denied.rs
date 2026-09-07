use crate::i18n;
use dioxus::prelude::*;
use sre_audit::services::auth;

#[component]
pub fn Denied() -> Element {
    let lang: Signal<i18n::Lang> = use_context();
    let auth_sig: Signal<auth::AuthState> = use_context();
    let l = *lang.read();
    let a = auth_sig.read().clone();

    let email = a.profile.as_ref().map(|p| p.email.clone());
    let t_title = i18n::tr("denied_title", l);
    let t_body = i18n::tr("denied_body", l);
    let t_email = i18n::tr("denied_email", l);
    let t_back = i18n::tr("denied_back", l);

    let sign_out = move |_: MouseEvent| {
        auth::sign_out();
    };

    rsx! {
        div { class: "gate-screen",
            div { class: "gate-card",
                div { class: "gate-icon", "⛔" }
                h1 { "{t_title}" }
                p { "{t_body}" }
                if let Some(e) = email {
                    div { class: "gate-error",
                        "{t_email} : {e}"
                    }
                }
                button { class: "btn btn-secondary btn-lg", onclick: sign_out,
                    "{t_back}"
                }
            }
        }
    }
}
