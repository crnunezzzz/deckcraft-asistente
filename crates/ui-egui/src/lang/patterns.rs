//! Spanish for interface text with changing parts (fork addition): «Saved report.pptx»,
//! «Slide 3 of 8», «Brightness +20%». The text is written in English where it is built (the
//! original's `format!` calls); [`super::tr`] recognises it here by its pattern, so those calls
//! stay as the original has them and only pass their result through `tr`.
//!
//! In a pattern `{}` stands for any text and `{n}` for a number; the Spanish side uses the same
//! marks, filled in the same order. A part that has a Spanish text of its own («Top» in «Align
//! text to the Top of the cell») is translated too.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Longest text looked at: anything longer is a document's own text, not the interface's.
const MAX_LEN: usize = 400;
/// Most translations kept: each is built once and kept for the life of the program, so the
/// number is bounded (file names make every «Saved …» different).
const MAX_KEPT: usize = 2048;

/// English pattern, Spanish pattern. The first that matches wins: specific before general.
const ES: &[(&str, &str)] = &[
    // Files.
    ("Saved {}", "Se guardó {}"),
    ("Exported {}", "Se exportó {}"),
    ("Couldn't save {}: {}", "No se pudo guardar {}: {}"),
    ("Couldn't save AutoRecover information: {}", "No se pudo guardar la información de Autorrecuperación: {}"),
    ("Couldn't create a private print preview: {}", "No se pudo crear la vista previa de impresión: {}"),
    ("DeckCraft can't export “{}”", "DeckCraft no puede exportar «{}»"),
    ("File › {} needs an open presentation", "Archivo › {} necesita una presentación abierta"),
    ("No recent presentations match “{}”.", "Ninguna presentación reciente coincide con «{}»."),
    ("No themes or templates match “{}”.", "Ningún tema o plantilla coincide con «{}»."),
    ("Do you want to save the changes you made to {n} presentations?", "¿Desea guardar los cambios realizados en {n} presentaciones?"),
    ("Do you want to save the changes you made to “{}”?", "¿Desea guardar los cambios realizados en «{}»?"),
    ("Do you want to save the changes you made to {}?", "¿Desea guardar los cambios realizados en {}?"),
    ("Opening {}…", "Abriendo {}…"),
    ("Still opening {}", "Todavía se está abriendo {}"),
    ("Stopped opening {}", "Se detuvo la apertura de {}"),
    ("Reading the file: {n} s", "Leyendo el archivo: {n} s"),
    ("Version {}", "Versión {}"),
    ("Sample deck: {}", "Presentación de ejemplo: {}"),
    // Slides.
    ("Slide {n} of {n}", "Diapositiva {n} de {n}"),
    ("Slide {n} deleted", "Diapositiva {n} eliminada"),
    ("Slide {n} changed", "Diapositiva {n} modificada"),
    ("Slide inserted after {n}", "Diapositiva insertada después de la {n}"),
    ("Slide {n}: “{}”", "Diapositiva {n}: «{}»"),
    ("Slide {n}: Missing alternative text — {}", "Diapositiva {n}: falta el texto alternativo — {}"),
    ("Slide {n}: {}", "Diapositiva {n}: {}"),
    ("Slide {n}", "Diapositiva {n}"),
    ("Go to slide {n}", "Ir a la diapositiva {n}"),
    ("First {n} slides", "Primeras {n} diapositivas"),
    ("Inserted {n} slides", "Se insertaron {n} diapositivas"),
    ("Insert slides from {} after the current slide:", "Insertar las diapositivas de {} después de la diapositiva actual:"),
    ("Slides Per Page: {n}", "Diapositivas por página: {n}"),
    ("{n} Slides", "{n} diapositivas"),
    ("{n} Slide", "{n} diapositiva"),
    // Find, spelling, review.
    ("Replaced “{}” with “{}”.", "Se reemplazó «{}» por «{}»."),
    ("Replaced {n} occurrence(s).", "Se reemplazaron {n} coincidencias."),
    ("DeckCraft can't find “{}”.", "DeckCraft no encuentra «{}»."),
    ("Match {n} of {n} on slide {}.", "Coincidencia {n} de {n} en la diapositiva {}."),
    ("Dictionary: {}", "Diccionario: {}"),
    (
        "No {} dictionary is installed, so only common misspellings are found. Put a Hunspell .aff/.dic pair in {}.",
        "No hay un diccionario de {} instalado, así que solo se detectan los errores más comunes. Copie un par .aff/.dic de Hunspell en {}.",
    ),
    ("No synonyms for \"{}\".", "No hay sinónimos de «{}»."),
    ("Accept All ({n})", "Aceptar todo ({n})"),
    ("Reject All ({n})", "Rechazar todo ({n})"),
    ("Compared with {}", "Comparado con {}"),
    // Pictures, shapes, tables.
    ("Brightness {}%", "Brillo {}%"),
    ("Contrast {}%", "Contraste {}%"),
    ("Saturation {}%", "Saturación {}%"),
    ("Compressed {n} picture(s): {} KB → {} KB", "Se comprimieron {n} imágenes: {} KB → {} KB"),
    ("Rotation {}°, brightness {}, contrast {}", "Giro {}°, brillo {}, contraste {}"),
    ("Text: {}", "Texto: {}"),
    ("Gradient stop {n}", "Punto de degradado {n}"),
    ("Style {n}", "Estilo {n}"),
    ("WordArt style {n}", "Estilo de WordArt {n}"),
    ("Action Button: {}", "Botón de acción: {}"),
    ("Align text to the {} of the cell", "Alinear el texto en la parte {} de la celda"),
    ("align text to the {} of the cell", "alinear el texto en la parte {} de la celda"),
    ("Light Style {n} - Accent {n}", "Estilo claro {n} - Énfasis {n}"),
    ("Medium Style {n} - Accent {n}", "Estilo medio {n} - Énfasis {n}"),
    ("Dark Style {n} - Accent {n}", "Estilo oscuro {n} - Énfasis {n}"),
    ("Themed Style {n} - Accent {n}", "Estilo temático {n} - Énfasis {n}"),
    ("Light Style {n}", "Estilo claro {n}"),
    ("Medium Style {n}", "Estilo medio {n}"),
    ("Dark Style {n}", "Estilo oscuro {n}"),
    ("Themed Style {n}", "Estilo temático {n}"),
    // Animation, sound.
    ("On Click of {}", "Al hacer clic en {}"),
    ("Start: {}s, End: {}s", "Inicio: {} s, fin: {} s"),
    ("Timing bar {n}", "Barra de intervalos {n}"),
    ("Duration handle {n}", "Control de duración {n}"),
    ("Shape {n}", "Forma {n}"),
    ("No sound: {}", "Sin sonido: {}"),
    // Dialogs.
    ("Increase {}", "Aumentar {}"),
    ("Decrease {}", "Reducir {}"),
    ("Width ({}):", "Ancho ({}):"),
    ("Height ({}):", "Alto ({}):"),
    ("Heading sample  {}", "Muestra de títulos  {}"),
    ("Body sample  {}", "Muestra de cuerpo  {}"),
    ("Clear the tab at {}\"", "Borrar la tabulación en {}\""),
    ("Width: enter {n} to {n} pixels", "Ancho: escriba de {n} a {n} píxeles"),
    ("Seconds per slide: enter {} to {}", "Segundos por diapositiva: escriba de {} a {}"),
    ("Slides: choose a range from 1 to {n}", "Diapositivas: elija un intervalo de 1 a {n}"),
    ("Text/Background - Dark {n}", "Texto/fondo - Oscuro {n}"),
    ("Text/Background - Light {n}", "Texto/fondo - Claro {n}"),
    ("Accent {n}", "Énfasis {n}"),
    ("{} (Custom)", "{} (personalizado)"),
];

enum Part<'a> {
    Text(&'a str),
    /// A changing part; `number` when only digits (with a sign, a point or a comma) fit it.
    Hole {
        number: bool,
    },
}

fn parts(pattern: &str) -> Vec<Part<'_>> {
    let mut out = Vec::new();
    let mut rest = pattern;
    while !rest.is_empty() {
        let any = rest.find("{}");
        let number = rest.find("{n}");
        let (at, len, is_number) = match (any, number) {
            (Some(a), Some(n)) if n < a => (n, 3, true),
            (Some(a), _) => (a, 2, false),
            (None, Some(n)) => (n, 3, true),
            (None, None) => {
                out.push(Part::Text(rest));
                break;
            }
        };
        if at > 0 {
            out.push(Part::Text(rest.get(..at).unwrap_or_default()));
        }
        out.push(Part::Hole { number: is_number });
        rest = rest.get(at + len..).unwrap_or_default();
    }
    out
}

fn is_number(text: &str) -> bool {
    !text.is_empty()
        && text.chars().any(|c| c.is_ascii_digit())
        && text.chars().all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '−' | '.' | ','))
}

/// The changing parts of `text` if it has the shape of `pattern`.
fn captures<'a>(pattern: &str, text: &'a str) -> Option<Vec<&'a str>> {
    let parts = parts(pattern);
    let mut rest = text;
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(part) = parts.get(i) {
        match part {
            Part::Text(literal) => rest = rest.strip_prefix(literal)?,
            Part::Hole { number } => {
                let found = match parts.get(i + 1) {
                    // Up to the text that follows the hole.
                    Some(Part::Text(next)) => {
                        let at = if parts.len() == i + 2 {
                            rest.len().checked_sub(next.len()).filter(|at| rest.get(*at..) == Some(*next))?
                        } else {
                            rest.find(next)?
                        };
                        let (found, after) = rest.split_at_checked(at)?;
                        rest = after;
                        found
                    }
                    // Two holes in a row cannot be told apart.
                    Some(Part::Hole { .. }) => return None,
                    None => std::mem::take(&mut rest),
                };
                if *number && !is_number(found) {
                    return None;
                }
                out.push(found);
            }
        }
        i += 1;
    }
    rest.is_empty().then_some(out)
}

fn fill(pattern: &str, values: &[&str], map: &HashMap<&'static str, &'static str>) -> String {
    let mut out = String::new();
    let mut values = values.iter();
    for part in parts(pattern) {
        match part {
            Part::Text(literal) => out.push_str(literal),
            Part::Hole { .. } => {
                let value = values.next().copied().unwrap_or_default();
                out.push_str(map.get(value).copied().unwrap_or(value));
            }
        }
    }
    out
}

/// The Spanish for `text` when it has one of the patterns' shapes.
fn spanish(text: &str, map: &HashMap<&'static str, &'static str>) -> Option<String> {
    // A ribbon label broken over two lines is one line here; the ribbon wraps the translation.
    let one_line;
    let text = if text.contains('\n') {
        one_line = text.replace('\n', " ");
        one_line.as_str()
    } else {
        text
    };
    ES.iter().find_map(|(en, es)| captures(en, text).map(|values| fill(es, &values, map)))
}

/// `text` in Spanish by one of the patterns, or `None`. The result lives as long as the program:
/// `tr` hands out plain `&str`, as for the fixed texts, so each new translation is kept once.
pub(super) fn translate(text: &str, map: &HashMap<&'static str, &'static str>) -> Option<&'static str> {
    static KEPT: OnceLock<Mutex<HashMap<String, Option<&'static str>>>> = OnceLock::new();
    if text.len() > MAX_LEN {
        return None;
    }
    let mut kept = KEPT.get_or_init(Default::default).lock().ok()?;
    if let Some(known) = kept.get(text) {
        return *known;
    }
    if kept.len() >= MAX_KEPT {
        return None;
    }
    let made: Option<&'static str> = spanish(text, map).map(|s| &*Box::leak(s.into_boxed_str()));
    kept.insert(text.to_owned(), made);
    made
}

#[cfg(test)]
mod tests {
    use super::*;

    fn es(text: &str) -> Option<String> {
        spanish(text, &HashMap::from([("Top", "superior"), ("Chart 2", "Gráfico 2")]))
    }

    #[test]
    fn changing_parts_are_kept_and_the_rest_reads_spanish() {
        assert_eq!(es("Saved Informe final.pptx").as_deref(), Some("Se guardó Informe final.pptx"));
        assert_eq!(es("Slide 3 of 8").as_deref(), Some("Diapositiva 3 de 8"));
        assert_eq!(es("Couldn't save a.pptx: disk full").as_deref(), Some("No se pudo guardar a.pptx: disk full"));
        assert_eq!(es("Replaced “uno” with “dos”.").as_deref(), Some("Se reemplazó «uno» por «dos»."));
        assert_eq!(es("Brightness +20%").as_deref(), Some("Brillo +20%"));
        assert_eq!(es("Medium Style 2 - Accent 1").as_deref(), Some("Estilo medio 2 - Énfasis 1"));
        assert_eq!(es("Medium Style 2").as_deref(), Some("Estilo medio 2"));
        assert_eq!(es("Slides Per\nPage: 6").as_deref(), Some("Diapositivas por página: 6"));
    }

    #[test]
    fn a_part_with_its_own_spanish_is_translated_too() {
        assert_eq!(es("Align text to the Top of the cell").as_deref(), Some("Alinear el texto en la parte superior de la celda"));
        assert_eq!(es("Slide 5: Missing alternative text — Chart 2").as_deref(), Some("Diapositiva 5: falta el texto alternativo — Gráfico 2"));
    }

    #[test]
    fn a_number_part_takes_only_numbers_and_other_text_is_left_alone() {
        // «Slide Master» is not «Slide {n}».
        assert_eq!(es("Slide Master"), None);
        assert_eq!(es("Slide 4 deleted").as_deref(), Some("Diapositiva 4 eliminada"));
        assert_eq!(es("Slide 4").as_deref(), Some("Diapositiva 4"));
        assert_eq!(es("Saved"), None);
        assert_eq!(es(""), None);
        assert_eq!(es("Nothing like a pattern"), None);
    }

    #[test]
    fn every_pattern_has_as_many_parts_in_spanish() {
        for (en, es) in ES {
            let holes = |p: &str| parts(p).iter().filter(|x| matches!(x, Part::Hole { .. })).count();
            assert_eq!(holes(en), holes(es), "{en:?} → {es:?}");
            assert!(holes(en) > 0, "{en:?} has no changing part: it belongs in the fixed texts");
        }
    }

    #[test]
    fn translations_are_kept_once_and_long_text_is_not_looked_at() {
        let map = HashMap::new();
        let a = translate("Saved kept-once.pptx", &map);
        let b = translate("Saved kept-once.pptx", &map);
        assert_eq!(a, Some("Se guardó kept-once.pptx"));
        assert!(matches!((a, b), (Some(x), Some(y)) if std::ptr::eq(x, y)));
        assert_eq!(translate(&format!("Saved {}", "x".repeat(MAX_LEN)), &map), None);
    }
}
