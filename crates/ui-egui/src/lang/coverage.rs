//! Finds interface text that still reads English in the Spanish interface (fork addition).
//!
//! `spanish_interface_report` walks the ribbon's tabs, the File tab's pages, the dialogs and the
//! panes with the interface in Spanish and prints, for each, the text that went through
//! [`super::tr`] without a translation and the text that was drawn without going through it.
//! It changes the interface language, which every test of this process shares, so it is ignored
//! by default. Run it alone when new interface text arrives with a merge of the original:
//!
//! ```text
//! cargo test -p deckcraft-ui-egui spanish_interface_report -- --ignored --nocapture --test-threads=1
//! ```
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::cell::RefCell;
use std::collections::BTreeSet;

use deckcraft_engine::Session;
use serde_json::json;

use crate::{Services, SlideApp};

thread_local! {
    /// Text `tr` had no Spanish for, while a report is collecting.
    static MISSED: RefCell<Option<BTreeSet<String>>> = const { RefCell::new(None) };
}

/// Called by `tr` for text it returned unchanged.
pub(super) fn missed(text: &str) {
    MISSED.with(|m| {
        if let Some(set) = m.borrow_mut().as_mut() {
            set.insert(text.to_owned());
        }
    });
}

fn drawn(shape: &egui::Shape, out: &mut BTreeSet<String>) {
    match shape {
        egui::Shape::Text(t) => {
            out.insert(t.galley.job.text.clone());
        }
        egui::Shape::Vec(v) => v.iter().for_each(|s| drawn(s, out)),
        _ => {}
    }
}

/// Reads English: a word of three ASCII letters or more that is not one of our Spanish texts,
/// a line of one (the ribbon wraps its labels), or one with its shortcut after it.
fn looks_english(text: &str, spanish: &BTreeSet<&str>) -> bool {
    let t = text.trim();
    let base = t.rfind(" (").filter(|_| t.ends_with(')')).map_or(t, |i| &t[..i]);
    if t.is_empty() || spanish.contains(t) || spanish.contains(base) || t.lines().all(|l| spanish.contains(l.trim())) {
        return false;
    }
    if !t.split(|c: char| !c.is_ascii_alphabetic()).any(|w| w.len() >= 3) {
        return false;
    }
    // A wrapped label's line is part of a Spanish text.
    !(t.len() <= 24 && spanish.iter().any(|s| s.split_whitespace().collect::<Vec<_>>().join(" ").contains(t)))
}

type Harness = egui_kittest::Harness<'static, SlideApp>;

fn harness() -> Harness {
    let mut session = Session::with_new();
    let _ = deckcraft_engine::sample::open_sample(&mut session);
    let app = SlideApp::new(session, Services { host_toolbar: Some(Default::default()), ..Default::default() });
    egui_kittest::Harness::builder().with_size(egui::vec2(2200.0, 1300.0)).build_ui_state(
        |ui, app: &mut SlideApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    )
}

#[test]
#[ignore = "changes the interface language of the whole test process; run it alone"]
fn spanish_interface_report() {
    super::set(super::Language::Es);
    let spanish: BTreeSet<&str> = super::spanish().values().copied().collect();
    let mut seen = BTreeSet::new();
    let mut look = |name: &str, h: &mut Harness| {
        MISSED.with(|m| *m.borrow_mut() = Some(BTreeSet::new()));
        h.run_steps(4);
        let mut texts = BTreeSet::new();
        for clipped in &h.output().shapes {
            drawn(&clipped.shape, &mut texts);
        }
        let missed = MISSED.with(|m| m.borrow_mut().take()).unwrap_or_default();
        let fresh: Vec<String> = missed.into_iter().chain(texts).filter(|t| looks_english(t, &spanish)).filter(|t| seen.insert(t.clone())).collect();
        if !fresh.is_empty() {
            println!("== {name}");
            for t in fresh {
                println!("   {t:?}");
            }
        }
    };

    let mut h = harness();
    for tab in ["Home", "Insert", "Draw", "Design", "Transitions", "Animations", "Slide Show", "Record", "Review", "View", "Help"] {
        h.state_mut().ui.tab = tab.into();
        look(tab, &mut h);
    }
    // Contextual tabs, each with its kind of object selected.
    let png = deckcraft_engine::cmd::base64_encode(&[
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 8, 0, 0, 0, 8, 8, 2, 0, 0, 0, 75, 109, 41, 220, 0, 0, 0, 17, 73, 68,
        65, 84, 120, 156, 99, 184, 163, 161, 129, 21, 49, 12, 45, 9, 0, 123, 123, 75, 1, 251, 162, 81, 240, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96,
        130,
    ]);
    let objects: [(&str, serde_json::Value, &[&str]); 5] = [
        ("shape.insert", json!({"preset": "rect", "rect": [100, 100, 200, 100]}), &["Shape Format"]),
        ("insert.picture", json!({"name": "a.png", "data": png, "rect": [100, 100, 200, 200]}), &["Picture Format"]),
        ("insert.table", json!({"rows": 3, "cols": 3}), &["Table Design", "Layout"]),
        ("insert.chart", json!({}), &["Chart Design"]),
        ("insert.smartArt", json!({"layout": "basicBlockList"}), &["SmartArt Design"]),
    ];
    for (command, params, tabs) in objects {
        if let Err(e) = h.state_mut().run(command, params) {
            println!("== {command} did not run: {e}");
            continue;
        }
        for tab in tabs {
            h.state_mut().ui.tab = (*tab).into();
            look(tab, &mut h);
        }
        for pane in ["format", "selection", "animation", "comments", "smartart", "thesaurus", "designer"] {
            h.state_mut().ui.pane = Some(pane.into());
            look(&format!("{command} + pane {pane}"), &mut h);
        }
        h.state_mut().ui.pane = None;
    }
    h.state_mut().ui.tab = "Home".into();
    for (command, name) in [("view.sorter", "Slide Sorter"), ("view.notesPage", "Notes Page"), ("view.outline", "Outline"), ("view.normal", "Normal")]
    {
        let _ = h.state_mut().run(command, json!({}));
        look(name, &mut h);
    }
    for (command, tab) in [("view.slideMaster", "Slide Master"), ("view.handoutMaster", "Handout Master"), ("view.notesMaster", "Notes Master")] {
        if h.state_mut().run(command, json!({})).is_ok() {
            h.state_mut().ui.tab = tab.into();
            look(tab, &mut h);
        }
    }
    let _ = h.state_mut().run("view.normal", json!({}));
    {
        use crate::backstage::Page;
        for page in [Page::Home, Page::New, Page::Open, Page::Info, Page::SaveAs, Page::Print, Page::Export, Page::About] {
            h.state_mut().open_backstage(page);
            look(&format!("File › {page:?}"), &mut h);
        }
        h.state_mut().close_backstage();
    }
    for id in [
        "table",
        "slideSize",
        "sizePosition",
        "headerFooter",
        "hyperlink",
        "setupShow",
        "zoom",
        "comment",
        "altText",
        "outline",
        "renameSection",
        "reuseSlides",
        "renameShape",
        "renameLayout",
        "paragraph",
        "tabs",
        "font",
        "about",
        "preferences",
        "autocorrect",
        "gridOptions",
        "columns",
        "export",
        "symbol",
        "photoAlbum",
        "replaceFonts",
        "equation",
        "dateTime",
        "spelling",
        "pasteSpecial",
        "find",
        "accessibility",
        "language",
        "customShow",
        "chartData",
        "trim",
        "smartart",
        "bullets",
        "action",
        "quit",
        "close",
        "themeColors",
        "themeFonts",
    ] {
        // A window of its own, and a panic of the test fonts (a glyph they lack) ends only it.
        let mut own = harness();
        own.state_mut().dialog = Some(crate::dialogs::Dialog::new(id));
        let name = format!("dialog {id}");
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| look(&name, &mut own))).is_err() {
            println!("== {name}: could not be drawn with the test fonts");
        }
        // Dropping a harness after a panic inside its frame would panic again.
        std::mem::forget(own);
    }
    super::set(super::Language::En);
}
