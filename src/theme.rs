// ── Dark mode (persisted, mirrors the lang switch) ──────────────────────────
// Stored as "dark"/"light" in localStorage so the choice survives reloads.

#[cfg(target_arch = "wasm32")]
const THEME_KEY: &str = "sre_audit_theme";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    /// Initial selection: the persisted value, else light.
    pub fn detect() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(s) = web_sys::window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
                .and_then(|s| s.get_item(THEME_KEY).ok())
                .flatten()
            {
                return if s.starts_with("dark") {
                    Theme::Dark
                } else {
                    Theme::Light
                };
            }
            Theme::Light
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Theme::Light
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }

    pub fn is_dark(self) -> bool {
        self == Self::Dark
    }

    pub fn attr(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    pub fn persist(self) {
        let s = self.attr();
        #[cfg(target_arch = "wasm32")]
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .flatten()
        {
            let _ = storage.set_item(THEME_KEY, s);
        }
        let _ = s;
    }
}
