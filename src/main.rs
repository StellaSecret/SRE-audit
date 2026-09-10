#![allow(non_snake_case)]

mod router;
mod theme;
mod views;

use dioxus::prelude::*;
use router::Route;
use sre_audit::i18n;
use sre_audit::orgs::{self, OrgStore};
use sre_audit::services::auth;
use sre_audit::services::storage;

fn main() {
    #[cfg(target_arch = "wasm32")]
    inject_head_resources();

    dioxus::launch(App);
}

#[cfg(target_arch = "wasm32")]
fn inject_head_resources() {
    let doc = match web_sys::window().and_then(|w| w.document()) {
        Some(d) => d,
        None => return,
    };
    doc.set_title("SRE Audit");
    let head = match doc.head() {
        Some(h) => h,
        None => return,
    };

    if let Ok(el) = doc.create_element("link") {
        let _ = el.set_attribute("rel", "icon");
        let _ = el.set_attribute("type", "image/svg+xml");
        let _ = el.set_attribute("href", &asset!("/assets/sre-audit-icon.svg").to_string());
        let _ = head.append_child(&el);
    }

    if let Ok(el) = doc.create_element("link") {
        let _ = el.set_attribute("rel", "manifest");
        let _ = el.set_attribute("href", &asset!("/assets/manifest.json").to_string());
        let _ = head.append_child(&el);
    }
}

#[component]
fn App() -> Element {
    // Boot auth (inject GIS script + restore/verify session). The lib's init is
    // idempotent — subsequent renders are no-ops.
    auth::init();

    let lang = use_signal(i18n::Lang::detect);
    use_context_provider(|| lang);

    let theme = use_signal(theme::Theme::detect);
    use_context_provider(|| theme);

    // Audited organizations. On first run (no stored store) seed a default org
    // (id == "") whose name comes from the legacy matrix company field — this
    // org maps to the pre-multi-org storage keys / Drive files.
    let orgs = use_signal(|| match orgs::load() {
        Some(store) if !store.orgs.is_empty() => store,
        _ => orgs::apply_defaults(
            OrgStore::default(),
            &storage::load_matrix("").unwrap_or_default(),
        ),
    });
    use_context_provider(|| orgs);
    use_effect(move || {
        orgs::save(&orgs());
    });

    // Reactive mirror of the static auth state. The lib notifies this signal on
    // every auth transition (sign-in, verify, deny, sign-out).
    let auth_sig = use_signal(auth::get_state);
    {
        let mut sig = auth_sig;
        auth::on_auth_change(Box::new(move || {
            sig.set(auth::get_state());
        }));
    }
    use_context_provider(|| auth_sig);
    let a = auth_sig();

    use_effect(move || {
        let l = lang();
        l.persist();
    });

    use_effect(move || {
        let t = theme();
        t.persist();
        #[cfg(target_arch = "wasm32")]
        if let Some(body) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.body())
        {
            let _ = body.set_attribute("data-theme", t.attr());
        }
    });

    #[cfg(target_arch = "wasm32")]
    use_effect(move || {
        let tag = if a.signed_in {
            "signed-in"
        } else if a.allowed {
            "allowed"
        } else {
            "out"
        };
        if let Some(body) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.body())
        {
            let _ = body.set_attribute("data-auth", tag);
            let _ = body.set_attribute(
                "data-e2e",
                if sre_audit::config::E2E_MODE {
                    "1"
                } else {
                    "0"
                },
            );
            let _ = body.set_attribute(
                "data-token",
                if auth::get_state().token.is_empty() {
                    "0"
                } else {
                    "1"
                },
            );
        }
    });

    rsx! {
        style { {include_str!("../assets/main.css")} }
        style { {include_str!("../assets/matrix.css")} }
        style { {include_str!("../assets/roadmap.css")} }

        if !a.signed_in {
            views::login::Login {}
        } else if !a.allowed {
            views::denied::Denied {}
        } else {
            Router::<Route> {}
        }
    }
}
