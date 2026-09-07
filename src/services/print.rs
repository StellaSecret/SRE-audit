// ── Print mirror sync ────────────────────────────────────────────────────────
// The legacy matrices show live-edited fields on paper by mirroring each
// textarea value into a hidden "print" element that is only displayed under
// `@media print`. We replicate that: when a field changes, copy its value into
// the sibling print element (keyed by a stable `data-print-for` attribute).

/// Copy the value of the textarea with `id` into the print element that carries
/// `data-print-for="{id}"`. No-op on native (non-wasm) targets.
#[cfg(target_arch = "wasm32")]
pub fn mirror(id: &str, value: &str) {
    use web_sys::window;
    let Some(w) = window() else { return };
    let Some(doc) = w.document() else { return };

    let target = doc
        .query_selector(&format!("[data-print-for=\"{id}\"]"))
        .ok()
        .flatten();
    if let Some(el) = target {
        el.set_text_content(Some(value));
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn mirror(_id: &str, _value: &str) {}
