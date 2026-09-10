// Library crate: pure Rust — no Dioxus, no WASM deps.
// `cargo test --lib` targets this crate exclusively:
//   - tests compile and run on any native host (Linux CI, Mac, Windows)
//   - the full Dioxus/WASM dependency tree is never pulled in during testing
//   - the binary (main.rs + views/) imports from here via `sre_audit::`

pub mod config;
pub mod data;
pub mod i18n;
pub mod models;
pub mod orgs;
pub mod services;
