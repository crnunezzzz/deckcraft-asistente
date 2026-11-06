//! The AI assistant panel shared by DeckCraft's hosts: the panel and the provider protocols
//! compile everywhere; [`desktop`] runs them on Windows, macOS and Linux (the Android host is
//! `deckcraft-android`).
#![deny(unsafe_code, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo, clippy::unimplemented, clippy::unreachable)]

pub mod agent;
pub mod assistant;
pub mod attach;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod desktop;
pub mod protocol;
pub mod update;

/// Where this fork publishes its builds (the workflow's own repository takes precedence). Its
/// former name, `crnunezzzz/deckcraftandroid`, is accepted too (`update::RENAMES`).
pub const RELEASES: &str = "crnunezzzz/deckcraft-asistente";

/// The original project: its newest release tells whether this build is behind (`update::behind`).
pub const UPSTREAM: &str = "storytold/deckcraft";

/// The assistant's texts for this app.
pub const BRANDING: assistant::Branding = assistant::Branding {
    name: "DeckCraft",
    noun: "la presentación",
    welcome: "Pregunta sobre la presentación o pide cambios. El asistente la lee y la edita con las mismas órdenes que la línea de comandos.",
    suggestions: &[
        ("Resumen de la presentación", "Resume esta presentación diapositiva por diapositiva."),
        ("Crea una diapositiva de conclusiones", "Añade al final una diapositiva de conclusiones basada en el contenido de la presentación."),
        ("Revisa ortografía y estilo", "Revisa la ortografía y el estilo de los textos de todas las diapositivas y corrige los errores."),
        ("Sugiere mejoras de diseño", "Sugiere mejoras de diseño y de estructura para esta presentación."),
    ],
};
