use crate::i18n;
use dioxus::prelude::*;
use sre_audit::config::E2E_MODE;
use sre_audit::services::auth;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[component]
pub fn Login() -> Element {
    let lang: Signal<i18n::Lang> = use_context();
    let auth_sig: Signal<auth::AuthState> = use_context();
    let l = *lang.read();
    let a = auth_sig.read().clone();

    let t_title = i18n::tr("login_title", l);
    let t_subtitle = i18n::tr("login_subtitle", l);
    let t_button = i18n::tr("login_button", l);

    let do_login = move |_: MouseEvent| {
        auth::start_oauth();
    };

    let build_tag = if E2E_MODE {
        " · e2e-build (SRE_AUDIT_E2E baked)"
    } else {
        ""
    };

    rsx! {
        div { class: "gate-screen",
            div { class: "gate-card",
                div { class: "gate-icon", "🔒" }
                h1 { "{t_title}" }
                p { "{t_subtitle}" }
                if let Some(e) = a.error {
                    div { class: "gate-error", "{e}" }
                }
                button { class: "btn btn-primary btn-lg", onclick: do_login,
                    "{t_button}"
                }
                div { class: "gate-sub", "v{VERSION} · Google OAuth · restricted access{build_tag}" }
            }
        }
    }
}
