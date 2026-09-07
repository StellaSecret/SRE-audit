use dioxus::prelude::*;
use dioxus_router::Routable;

use crate::i18n;
use crate::theme;
use crate::views::{home::Home, matrix::Matrix, roadmap::Roadmap};
use sre_audit::services::auth;

#[derive(Clone, PartialEq, Routable, Debug)]
pub enum Route {
    #[layout(NavLayout)]
    #[route("/")]
    Home {},
    #[route("/matrix")]
    Matrix {},
    #[route("/roadmap")]
    Roadmap {},
}

#[component]
pub fn NavLayout() -> Element {
    let mut lang: Signal<i18n::Lang> = use_context();
    let auth_sig: Signal<auth::AuthState> = use_context();
    let mut theme_sig: Signal<theme::Theme> = use_context();

    let toggle_lang = move |_: MouseEvent| {
        let mut l = lang();
        l = l.toggle();
        lang.set(l);
    };

    let toggle_theme = move |_: MouseEvent| {
        let t = theme_sig();
        theme_sig.set(t.toggle());
    };

    let email = auth_sig.read().profile.as_ref().map(|p| p.email.clone());
    let sign_out = move |_: MouseEvent| {
        auth::sign_out();
    };

    rsx! {
        div { class: "app",
            header { class: "nav",
                Link { to: Route::Home {}, class: "nav-brand",
                    img { src: asset!("/assets/sre-audit-icon.svg"), class: "nav-brand-icon", alt: "" }
                    "SRE Audit"
                }
                div { class: "nav-links",
                    Link { to: Route::Home {}, class: "nav-link", active_class: "nav-link-active",
                        {i18n::tr("nav_home", lang())}
                    }
                    Link { to: Route::Matrix {}, class: "nav-link", active_class: "nav-link-active",
                        {i18n::tr("nav_matrix", lang())}
                    }
                    Link { to: Route::Roadmap {}, class: "nav-link", active_class: "nav-link-active",
                        {i18n::tr("nav_roadmap", lang())}
                    }
                }
                div { class: "nav-right",
                    if let Some(e) = email {
                        span { class: "nav-email", "{e}" }
                    }
                    button { class: "nav-toggle", onclick: toggle_lang, "{lang().label()}" }
                    button { class: "nav-theme", onclick: toggle_theme,
                        {if theme_sig().is_dark() {
                            i18n::tr("theme_light", lang())
                        } else {
                            i18n::tr("theme_dark", lang())
                        }}
                    }
                    button { class: "nav-signout", onclick: sign_out,
                        {i18n::tr("nav_signout", lang())}
                    }
                }
            }
            main { class: "main",
                Outlet::<Route> {}
            }
        }
    }
}
