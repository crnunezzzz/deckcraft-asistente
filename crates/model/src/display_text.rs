//! Fork addition: the interface's language for the few words the renderer draws itself (the
//! prompts in empty placeholders, [`crate::PhType::prompt`]). The model stays English; the
//! interface installs its translation once ([`set`]) and the renderer asks [`shown`].

use std::sync::OnceLock;

static HOOK: OnceLock<fn(&'static str) -> &'static str> = OnceLock::new();

/// Install the interface's translation (the first call wins).
pub fn set(translate: fn(&'static str) -> &'static str) {
    let _ = HOOK.set(translate);
}

/// `text` as the interface shows it: translated once [`set`] has run, else unchanged.
pub fn shown(text: &'static str) -> &'static str {
    HOOK.get().map_or(text, |translate| translate(text))
}
