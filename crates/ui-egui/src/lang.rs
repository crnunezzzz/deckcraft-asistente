//! Interface language (fork addition). DeckCraft upstream is English only; this module adds a
//! Spanish interface without touching the strings where they are written: the shared ribbon
//! widgets, the tab row, the File tab's menu and dialog titles pass their visible text through
//! [`tr`]. Accessible names stay English so tests and automation keep working; untranslated text
//! falls back to English.
//!
//! The choice lives in `language.json` beside `ui.json` ([`load`], [`save`]); with no saved
//! choice the system language decides (`LC_ALL`, `LC_MESSAGES`, `LANG`).

mod es_more;

use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

/// The interface languages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    En,
    Es,
}

impl Language {
    pub const ALL: [Self; 2] = [Self::En, Self::Es];

    /// The language's own name, shown in the picker.
    pub fn name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Es => "Español",
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Es => "es",
        }
    }

    /// The language a code or locale tag names (`es`, `es_PE.UTF-8`, `es-419`…).
    pub fn parse(tag: &str) -> Option<Self> {
        let base = tag.split(['_', '-', '.', '@']).next().unwrap_or_default().to_ascii_lowercase();
        match base.as_str() {
            "en" => Some(Self::En),
            "es" => Some(Self::Es),
            _ => None,
        }
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

/// The interface language in use.
pub fn current() -> Language {
    if CURRENT.load(Ordering::Relaxed) == 1 { Language::Es } else { Language::En }
}

/// Switch the interface language (takes effect on the next frame).
pub fn set(lang: Language) {
    CURRENT.store(if lang == Language::Es { 1 } else { 0 }, Ordering::Relaxed);
    // The renderer's own words (placeholder prompts) follow the interface too.
    deckcraft_model::display_text::set(tr_static);
}

fn tr_static(text: &'static str) -> &'static str {
    tr(text)
}

/// The system's interface language, when it is one we have.
pub fn system() -> Option<Language> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|v| std::env::var(v).ok())
        .find(|v| !v.is_empty() && v != "C" && v != "POSIX")
        .and_then(|v| Language::parse(&v))
}

const FILE: &str = "language.json";

/// Apply the saved language in `dir`, or the system's when none is saved.
pub fn load(dir: &Path) {
    let saved = std::fs::read(dir.join(FILE))
        .ok()
        .filter(|b| b.len() < 4096)
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .and_then(|v| v.get("language").and_then(|l| l.as_str()).and_then(Language::parse));
    if let Some(lang) = saved.or_else(system) {
        set(lang);
    }
}

/// Remember the current language in `dir`. Failures are logged, not fatal.
pub fn save(dir: &Path) {
    let body = serde_json::json!({ "language": current().code() }).to_string();
    if let Err(e) = std::fs::create_dir_all(dir).and_then(|()| std::fs::write(dir.join(FILE), body)) {
        log::warn!("{}: {e}", dir.join(FILE).display());
    }
}

/// The picker shown in Options.
pub fn picker(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let label = ui.label(tr("Interface language:"));
        let now = current();
        craft_ui::ComboBox::from_id_salt("interface_language")
            .selected_text(now.name())
            .show_ui(ui, |ui| {
                for l in Language::ALL {
                    if ui.selectable_label(l == now, l.name()).clicked() {
                        set(l);
                    }
                }
            })
            .response
            .labelled_by(label.id);
    });
}

/// A gallery tile's name centred on `at` within `width`: one line when it fits at `size`, else
/// two lines split at a space, smaller as needed (down to 7 pt). Spanish names are longer than
/// English ones ("Desplazar hacia dentro", "Barras aleatorias").
pub fn paint_tile_label(p: &egui::Painter, at: egui::Pos2, text: &str, size: f32, width: f32, color: egui::Color32) {
    let wide = |t: &str, s: f32| p.layout_no_wrap(t.to_owned(), crate::theme::font(s), color).size().x;
    if width <= 0.0 || wide(text, size) <= width {
        p.text(at, egui::Align2::CENTER_CENTER, text, crate::theme::font(size), color);
        return;
    }
    // The split that leaves the longer line shortest.
    let split = text
        .match_indices(' ')
        .map(|(i, _)| (text.get(..i).unwrap_or_default(), text.get(i + 1..).unwrap_or_default()))
        .min_by(|a, b| wide(a.0, size).max(wide(a.1, size)).total_cmp(&wide(b.0, size).max(wide(b.1, size))));
    let Some((first, second)) = split else {
        let s = (size * width / wide(text, size).max(1.0)).max(7.0);
        p.text(at, egui::Align2::CENTER_CENTER, text, crate::theme::font(s), color);
        return;
    };
    let longest = wide(first, size).max(wide(second, size)).max(1.0);
    let s = (size * 0.9).min(size * width / longest).max(7.0);
    let line = s + 1.0;
    p.text(at - egui::vec2(0.0, line / 2.0), egui::Align2::CENTER_CENTER, first, crate::theme::font(s), color);
    p.text(at + egui::vec2(0.0, line / 2.0), egui::Align2::CENTER_CENTER, second, crate::theme::font(s), color);
}

/// The status bar's slide counter ("Slide 3 of 8").
pub fn slide_of(n: usize, total: usize) -> String {
    match current() {
        Language::En => format!("Slide {n} of {total}"),
        Language::Es => format!("Diapositiva {n} de {total}"),
    }
}

fn spanish() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    // The first catalogue wins where both have a text.
    MAP.get_or_init(|| es_more::ES.iter().chain(ES).copied().collect())
}

/// `text` in the interface language: its translation, or `text` itself.
pub fn tr(text: &str) -> &str {
    if current() == Language::En || text.is_empty() {
        return text;
    }
    let map = spanish();
    if let Some(t) = map.get(text) {
        return t;
    }
    // A label broken over two lines ("New\nSlide") is looked up as one line; the ribbon wraps
    // the translation itself.
    if text.contains('\n')
        && let Some(t) = map.get(text.replace('\n', " ").as_str())
    {
        return t;
    }
    text
}

/// Spanish interface text, in PowerPoint's Spanish terminology where there is one.
const ES: &[(&str, &str)] = &[
    // Tabs and pages.
    ("File", "Archivo"),
    ("Home", "Inicio"),
    ("Insert", "Insertar"),
    ("Draw", "Dibujar"),
    ("Design", "Diseño"),
    ("Transitions", "Transiciones"),
    ("Animations", "Animaciones"),
    ("Slide Show", "Presentación"),
    ("Record", "Grabar"),
    ("Review", "Revisar"),
    ("View", "Vista"),
    ("Help", "Ayuda"),
    ("Shape Format", "Formato de forma"),
    ("Picture Format", "Formato de imagen"),
    ("Table Design", "Diseño de tabla"),
    ("Layout", "Diseño"),
    ("Present", "Presentar"),
    ("Chart Design", "Diseño de gráfico"),
    ("SmartArt Design", "Diseño de SmartArt"),
    ("Playback", "Reproducción"),
    ("Slide Master", "Patrón de diapositivas"),
    ("Handout Master", "Patrón de documentos"),
    ("Notes Master", "Patrón de notas"),
    ("New", "Nuevo"),
    ("Open", "Abrir"),
    ("Info", "Información"),
    ("Save", "Guardar"),
    ("Save As", "Guardar como"),
    ("Print", "Imprimir"),
    ("Export", "Exportar"),
    ("Close", "Cerrar"),
    ("About", "Acerca de"),
    ("Options", "Opciones"),
    ("Preferences", "Opciones"),
    ("Preferences…", "Opciones…"),
    ("General", "General"),
    ("Editing", "Edición"),
    ("Interface language:", "Idioma de la interfaz:"),
    ("Recent", "Recientes"),
    ("Presentations", "Presentaciones"),
    ("Blank Presentation", "Presentación en blanco"),
    ("New Presentation", "Nueva presentación"),
    ("More presentations", "Más presentaciones"),
    ("More templates", "Más plantillas"),
    ("More themes", "Más temas"),
    ("Templates", "Plantillas"),
    ("Search for themes and templates", "Buscar temas y plantillas"),
    ("Search for a file", "Buscar un archivo"),
    ("Not saved yet", "Aún no guardado"),
    ("Open file location", "Abrir ubicación del archivo"),
    ("Copy path", "Copiar ruta"),
    ("Copied the path", "Ruta copiada"),
    ("Remove from Recent", "Quitar de recientes"),
    ("Recover Unsaved Presentations", "Recuperar presentaciones no guardadas"),
    ("Properties", "Propiedades"),
    ("Title", "Título"),
    ("Tags", "Etiquetas"),
    ("Categories", "Categorías"),
    ("Subject", "Asunto"),
    ("Company", "Organización"),
    ("Author", "Autor"),
    ("Add a title", "Agregar un título"),
    ("Add a tag", "Agregar una etiqueta"),
    ("Add a category", "Agregar una categoría"),
    ("Add an author", "Agregar un autor"),
    ("Add comments", "Agregar comentarios"),
    ("Specify the subject", "Especificar el asunto"),
    ("Specify the company", "Especificar la organización"),
    ("Inspect Presentation", "Inspeccionar presentación"),
    ("Check for Issues", "Comprobar si hay problemas"),
    ("Manage Presentation", "Administrar presentación"),
    ("There are no unsaved changes.", "No hay cambios sin guardar."),
    ("This presentation has never been saved.", "Esta presentación nunca se ha guardado."),
    ("Create PDF", "Crear PDF"),
    ("Create PDF Document", "Crear documento PDF"),
    ("Create a PDF Document", "Crear un documento PDF"),
    ("Create Handouts", "Crear documentos"),
    ("Change File Type", "Cambiar el tipo de archivo"),
    ("Opens on any device with a PDF viewer", "Se abre en cualquier dispositivo con un visor de PDF"),
    ("Print them for your audience", "Imprímalos para su audiencia"),
    ("Several slides on each page of a PDF", "Varias diapositivas en cada página de un PDF"),
    ("The current slide as a picture", "La diapositiva actual como imagen"),
    ("The current slide as a compressed picture", "La diapositiva actual como imagen comprimida"),
    ("The presentation keeps its own file", "La presentación conserva su propio archivo"),
    ("Keeps the layout, fonts and pictures", "Conserva el diseño, las fuentes y las imágenes"),
    ("Three a page leaves lines for notes", "Tres por página deja líneas para notas"),
    ("PNG images (all slides)", "Imágenes PNG (todas las diapositivas)"),
    ("PNG images (current slide)", "Imágenes PNG (diapositiva actual)"),
    ("JPEG image (current slide)", "Imagen JPEG (diapositiva actual)"),
    ("Picture (PNG)", "Imagen (PNG)"),
    ("Save as type", "Tipo"),
    ("Choose where to save", "Elegir dónde guardar"),
    ("Saved", "Guardado"),
    ("Saved a copy", "Copia guardada"),
    ("Print All Slides", "Imprimir todas las diapositivas"),
    ("Print Current Slide", "Imprimir la diapositiva actual"),
    ("Print Hidden Slides", "Imprimir diapositivas ocultas"),
    ("Print layout", "Diseño de impresión"),
    ("Print range", "Intervalo de impresión"),
    ("Custom Range", "Intervalo personalizado"),
    ("Full Page Slides", "Diapositivas de página completa"),
    ("Notes Pages", "Páginas de notas"),
    ("Handouts (1 slide per page)", "Documentos (1 diapositiva por página)"),
    ("Handouts (2 slides per page)", "Documentos (2 diapositivas por página)"),
    ("Handouts (3 slides per page)", "Documentos (3 diapositivas por página)"),
    ("Handouts (4 slides per page)", "Documentos (4 diapositivas por página)"),
    ("Handouts (6 slides per page)", "Documentos (6 diapositivas por página)"),
    ("Handouts (9 slides per page)", "Documentos (9 diapositivas por página)"),
    ("Slides per page", "Diapositivas por página"),
    ("Include hidden slides", "Incluir diapositivas ocultas"),
    ("Opened the presentation as a PDF to print", "Se abrió la presentación como PDF para imprimir"),
    ("Paper", "Papel"),
    ("Paper size", "Tamaño del papel"),
    ("Orientation", "Orientación"),
    ("Portrait", "Vertical"),
    ("Landscape", "Horizontal"),
    ("Portrait Orientation", "Orientación vertical"),
    ("Landscape Orientation", "Orientación horizontal"),
    ("Letter", "Carta"),
    ("Resolution", "Resolución"),
    ("Small (640 px wide)", "Pequeña (640 px de ancho)"),
    ("Medium (960 px wide)", "Mediana (960 px de ancho)"),
    ("Large (1280 px wide)", "Grande (1280 px de ancho)"),
    ("Extra Large (1920 px wide)", "Muy grande (1920 px de ancho)"),
    ("Product Information", "Información del producto"),
    ("About DeckCraft", "Acerca de DeckCraft"),
    ("Check for Updates…", "Buscar actualizaciones…"),
    ("Credits", "Créditos"),
    ("Contributors", "Colaboradores"),
    ("Source on GitHub", "Código fuente en GitHub"),
    ("Community Links", "Enlaces de la comunidad"),
    ("Join our community on Discord", "Únase a nuestra comunidad en Discord"),
    ("Join the ArtCraft community on Discord", "Únase a la comunidad de ArtCraft en Discord"),
    ("Back", "Atrás"),
    ("Back (Esc)", "Atrás (Esc)"),
    // Common actions.
    ("OK", "Aceptar"),
    ("Cancel", "Cancelar"),
    ("Done", "Listo"),
    ("Apply", "Aplicar"),
    ("Apply to All", "Aplicar a todo"),
    ("Reset", "Restablecer"),
    ("Reset to Defaults", "Restablecer valores predeterminados"),
    ("Delete", "Eliminar"),
    ("Remove", "Quitar"),
    ("Rename", "Cambiar nombre"),
    ("Add", "Agregar"),
    ("Edit", "Editar"),
    ("Create", "Crear"),
    ("Update", "Actualizar"),
    ("Update automatically", "Actualizar automáticamente"),
    ("Browse", "Examinar"),
    ("Download", "Descargar"),
    ("Download the file", "Descargar el archivo"),
    ("Undo", "Deshacer"),
    ("Redo", "Rehacer"),
    ("Cut", "Cortar"),
    ("Copy", "Copiar"),
    ("Paste", "Pegar"),
    ("Paste Special", "Pegado especial"),
    ("Paste Special…", "Pegado especial…"),
    ("Paste options", "Opciones de pegado"),
    ("Paste Formatting", "Pegar formato"),
    ("Paste and Match Formatting", "Pegar y combinar formato"),
    ("Keep Source Formatting", "Mantener formato de origen"),
    ("Keep source formatting", "Mantener formato de origen"),
    ("Keep Text Only", "Mantener solo texto"),
    ("Use Destination Theme", "Usar tema de destino"),
    ("Select", "Seleccionar"),
    ("Select All", "Seleccionar todo"),
    ("Select None", "No seleccionar nada"),
    ("Selection", "Selección"),
    ("Selection Pane", "Panel de selección"),
    ("Selection Pane…", "Panel de selección…"),
    ("Find", "Buscar"),
    ("Find…", "Buscar…"),
    ("Find Next", "Buscar siguiente"),
    ("Find and Replace", "Buscar y reemplazar"),
    ("Find whole words only", "Solo palabras completas"),
    ("Match case", "Coincidir mayúsculas y minúsculas"),
    ("Replace", "Reemplazar"),
    ("Replace…", "Reemplazar…"),
    ("Replace All", "Reemplazar todo"),
    ("Replace Font", "Reemplazar fuente"),
    ("Replace Fonts…", "Reemplazar fuentes…"),
    ("Search commands…", "Buscar comandos…"),
    ("Command Palette", "Paleta de comandos"),
    ("Settings", "Configuración"),
    ("More options", "Más opciones"),
    ("Show", "Mostrar"),
    ("Hide", "Ocultar"),
    ("Show All", "Mostrar todo"),
    ("Hide All", "Ocultar todo"),
    ("Previous", "Anterior"),
    ("Next", "Siguiente"),
    ("Go to", "Ir a"),
    ("Lock", "Bloquear"),
    ("Unlock", "Desbloquear"),
    ("Clear", "Borrar"),
    ("Clear All", "Borrar todo"),
    ("Name", "Nombre"),
    ("Type", "Tipo"),
    ("Size", "Tamaño"),
    ("Position", "Posición"),
    ("Text", "Texto"),
    ("None", "Ninguno"),
    ("Automatic", "Automático"),
    ("Custom", "Personalizado"),
    ("All", "Todo"),
    ("Both", "Ambos"),
    ("Normal", "Normal"),
    ("Collapse the Ribbon", "Contraer la cinta"),
    ("Pin the ribbon", "Fijar la cinta"),
    ("Scroll ribbon left", "Desplazar la cinta a la izquierda"),
    ("Scroll ribbon right", "Desplazar la cinta a la derecha"),
    ("Quick Access Toolbar", "Barra de herramientas de acceso rápido"),
    ("Customize Quick Access Toolbar", "Personalizar barra de acceso rápido"),
    ("Add to Quick Access Toolbar", "Agregar a la barra de acceso rápido"),
    ("Remove from Quick Access Toolbar", "Quitar de la barra de acceso rápido"),
    ("Right-click any ribbon button to add it.", "Haga clic derecho en un botón de la cinta para agregarlo."),
    ("Ribbon Tab", "Pestaña de la cinta"),
    // Home.
    ("New Slide", "Nueva diapositiva"),
    ("Duplicate Slide", "Duplicar diapositiva"),
    ("Duplicate Selected Slides", "Duplicar diapositivas seleccionadas"),
    ("Delete Slide", "Eliminar diapositiva"),
    ("Reset Slide", "Restablecer diapositiva"),
    ("Hide Slide", "Ocultar diapositiva"),
    ("Show Slide", "Mostrar diapositiva"),
    ("Layouts", "Diseños"),
    ("Reuse Slides", "Reutilizar diapositivas"),
    ("Reuse Slides…", "Reutilizar diapositivas…"),
    ("Reuse Slides (Keep Source Formatting)…", "Reutilizar diapositivas (mantener formato de origen)…"),
    ("Slides from Outline", "Diapositivas del esquema"),
    ("Slides from Outline…", "Diapositivas del esquema…"),
    ("Section", "Sección"),
    ("Add Section", "Agregar sección"),
    ("Rename Section", "Cambiar nombre de sección"),
    ("Rename Section…", "Cambiar nombre de sección…"),
    ("Remove Section", "Quitar sección"),
    ("Remove Section & Slides", "Quitar sección y diapositivas"),
    ("Remove All Sections", "Quitar todas las secciones"),
    ("Move Section Up", "Subir sección"),
    ("Move Section Down", "Bajar sección"),
    ("Font", "Fuente"),
    ("Font…", "Fuente…"),
    ("Fonts", "Fuentes"),
    ("Font name", "Nombre de fuente"),
    ("Font Color", "Color de fuente"),
    ("Increase Font Size", "Aumentar tamaño de fuente"),
    ("Decrease Font Size", "Disminuir tamaño de fuente"),
    ("Clear All Formatting", "Borrar todo el formato"),
    ("Bold", "Negrita"),
    ("Italic", "Cursiva"),
    ("Underline", "Subrayado"),
    ("Strikethrough", "Tachado"),
    ("Subscript", "Subíndice"),
    ("Superscript", "Superíndice"),
    ("Small caps", "Versalitas"),
    ("All caps", "Mayúsculas"),
    ("Shadow", "Sombra"),
    ("Character Spacing", "Espaciado entre caracteres"),
    ("Change Case", "Cambiar mayúsculas y minúsculas"),
    ("Sentence case.", "Tipo oración."),
    ("UPPERCASE", "MAYÚSCULAS"),
    ("Capitalize Each Word", "Poner en mayúsculas cada palabra"),
    ("Text Highlight Color", "Color de resaltado del texto"),
    ("Paragraph", "Párrafo"),
    ("Paragraph…", "Párrafo…"),
    ("Bullets", "Viñetas"),
    ("Bullets options", "Opciones de viñetas"),
    ("Numbering", "Numeración"),
    ("Numbering options", "Opciones de numeración"),
    ("Bullets and Numbering", "Numeración y viñetas"),
    ("Bullets and Numbering…", "Numeración y viñetas…"),
    ("Bulleted", "Con viñetas"),
    ("Numbered", "Numerada"),
    ("Remove bullets and numbering", "Quitar viñetas y numeración"),
    ("Increase List Level", "Aumentar nivel de lista"),
    ("Decrease List Level", "Disminuir nivel de lista"),
    ("Line Spacing", "Interlineado"),
    ("Line Spacing Options…", "Opciones de interlineado…"),
    ("Line spacing", "Interlineado"),
    ("Align Left", "Alinear a la izquierda"),
    ("Align Center", "Centrar"),
    ("Align Right", "Alinear a la derecha"),
    ("Justify", "Justificar"),
    ("Justify low", "Justificar bajo"),
    ("Distributed", "Distribuido"),
    ("Columns", "Columnas"),
    ("Columns…", "Columnas…"),
    ("More Columns…", "Más columnas…"),
    ("Text Direction", "Dirección del texto"),
    ("Align Text", "Alinear texto"),
    ("Convert to SmartArt", "Convertir en SmartArt"),
    ("Right-to-Left Text Direction", "Dirección del texto de derecha a izquierda"),
    ("Shapes", "Formas"),
    ("Arrange", "Organizar"),
    ("Quick Styles", "Estilos rápidos"),
    ("Shape Fill", "Relleno de forma"),
    ("Shape Outline", "Contorno de forma"),
    ("Shape Effects", "Efectos de forma"),
    ("Bring to Front", "Traer al frente"),
    ("Bring Forward", "Traer adelante"),
    ("Send to Back", "Enviar al fondo"),
    ("Send Backward", "Enviar atrás"),
    ("Send Backward options", "Opciones de Enviar atrás"),
    ("Order Objects", "Ordenar objetos"),
    ("Group Objects", "Agrupar objetos"),
    ("Position Objects", "Posicionar objetos"),
    ("Group", "Agrupar"),
    ("Ungroup", "Desagrupar"),
    ("Regroup", "Reagrupar"),
    ("Align", "Alinear"),
    ("Align Top", "Alinear arriba"),
    ("Align Middle", "Alinear al medio"),
    ("Align Bottom", "Alinear abajo"),
    ("Align to Slide", "Alinear a la diapositiva"),
    ("Align Selected Objects", "Alinear objetos seleccionados"),
    ("Distribute Horizontally", "Distribuir horizontalmente"),
    ("Distribute Vertically", "Distribuir verticalmente"),
    ("Rotate", "Girar"),
    ("Rotate Right", "Girar a la derecha 90°"),
    ("Rotate Left", "Girar a la izquierda 90°"),
    ("Flip Vertical", "Voltear verticalmente"),
    ("Flip Horizontal", "Voltear horizontalmente"),
    ("Replace Fonts", "Reemplazar fuentes"),
    ("Dictate", "Dictar"),
    ("Design Ideas", "Ideas de diseño"),
    // Insert.
    ("Table", "Tabla"),
    ("Table…", "Tabla…"),
    ("Insert Table", "Insertar tabla"),
    ("Insert Table…", "Insertar tabla…"),
    ("Draw Table", "Dibujar tabla"),
    ("Pictures", "Imágenes"),
    ("Picture", "Imagen"),
    ("Insert Picture", "Insertar imagen"),
    ("Picture from File…", "Imagen desde archivo…"),
    ("Insert picture from file…", "Insertar imagen desde archivo…"),
    ("Screenshot", "Captura"),
    ("Screenshot (not available yet)", "Captura (aún no disponible)"),
    ("Photo Album", "Álbum de fotografías"),
    ("Photo Album…", "Álbum de fotografías…"),
    ("New Photo Album…", "Nuevo álbum de fotografías…"),
    ("Edit Photo Album…", "Editar álbum de fotografías…"),
    ("Icons", "Iconos"),
    ("Search icons", "Buscar iconos"),
    ("No icons match.", "Ningún icono coincide."),
    ("Models", "Modelos 3D"),
    ("SmartArt", "SmartArt"),
    ("Chart", "Gráfico"),
    ("Zoom", "Zoom"),
    ("Zoom…", "Zoom…"),
    ("Summary Zoom", "Zoom de resumen"),
    ("Section Zoom", "Zoom de sección"),
    ("Slide Zoom", "Zoom de diapositiva"),
    ("Link", "Vínculo"),
    ("Link…", "Vínculo…"),
    ("Insert Hyperlink", "Insertar hipervínculo"),
    ("Action", "Acción"),
    ("Action Settings", "Configuración de la acción"),
    ("Action Settings…", "Configuración de la acción…"),
    ("Action on click", "Acción al hacer clic"),
    ("Action on mouse over", "Acción al pasar el mouse"),
    ("Mouse Click", "Clic del mouse"),
    ("Mouse Over", "Pasar el mouse por encima"),
    ("Comment", "Comentario"),
    ("Comments", "Comentarios"),
    ("New Comment", "Nuevo comentario"),
    ("Text Box", "Cuadro de texto"),
    ("New Text Box", "Nuevo cuadro de texto"),
    ("Header & Footer", "Encabezado y pie de página"),
    ("Header and Footer", "Encabezado y pie de página"),
    ("Edit Header & Footer", "Editar encabezado y pie de página"),
    ("Header", "Encabezado"),
    ("Footer", "Pie de página"),
    ("Footers", "Pies de página"),
    ("WordArt", "WordArt"),
    ("Date & Time", "Fecha y hora"),
    ("Date and Time", "Fecha y hora"),
    ("Date and time", "Fecha y hora"),
    ("Date", "Fecha"),
    ("Slide Number", "Número de diapositiva"),
    ("Slide number", "Número de diapositiva"),
    ("Page Number", "Número de página"),
    ("Object", "Objeto"),
    ("Equation", "Ecuación"),
    ("Symbol", "Símbolo"),
    ("Video", "Vídeo"),
    ("Audio", "Audio"),
    ("Video from File…", "Vídeo desde archivo…"),
    ("Audio from File…", "Audio desde archivo…"),
    ("Media", "Multimedia"),
    ("Insert…", "Insertar…"),
    // Draw.
    ("Pen", "Lápiz"),
    ("Pencil", "Lapicero"),
    ("Highlighter", "Marcador de resaltado"),
    ("Eraser", "Borrador"),
    ("Black Pen", "Lápiz negro"),
    ("Red Pen", "Lápiz rojo"),
    ("Blue Pen", "Lápiz azul"),
    ("Pen Color", "Color del lápiz"),
    ("Pen Style", "Estilo del lápiz"),
    ("Pen Weight", "Grosor del lápiz"),
    ("Ink to Shape", "Entrada de lápiz a forma"),
    ("Ink to Text (coming soon)", "Entrada de lápiz a texto (próximamente)"),
    ("Ink to Math (coming soon)", "Entrada de lápiz a matemáticas (próximamente)"),
    ("Ink to Math", "Entrada de lápiz a matemáticas"),
    ("Ink to Text", "Entrada de lápiz a texto"),
    // Design.
    ("Themes", "Temas"),
    ("Themes (whole presentation)", "Temas (toda la presentación)"),
    ("Browse for Themes…", "Buscar temas…"),
    ("Variants", "Variantes"),
    ("Colors", "Colores"),
    ("Theme Colors", "Colores del tema"),
    ("Customize Colors…", "Personalizar colores…"),
    ("Customize Fonts…", "Personalizar fuentes…"),
    ("Create New Theme Colors", "Crear nuevos colores del tema"),
    ("Create New Theme Fonts", "Crear nuevas fuentes del tema"),
    ("Heading font", "Fuente de encabezado"),
    ("Body font", "Fuente de cuerpo"),
    ("Choose a body font.", "Elija una fuente de cuerpo."),
    ("Choose a heading font.", "Elija una fuente de encabezado."),
    ("Choose a font", "Elija una fuente"),
    ("Scheme name", "Nombre del esquema"),
    ("Enter a scheme name.", "Escriba un nombre para el esquema."),
    ("Background Styles", "Estilos de fondo"),
    ("Format Background", "Formato del fondo"),
    ("Format Background…", "Formato del fondo…"),
    ("Reset Background", "Restablecer fondo"),
    ("Reset Slide Background", "Restablecer fondo de la diapositiva"),
    ("Hide background graphics", "Ocultar gráficos de fondo"),
    ("Slide background fill", "Relleno del fondo de la diapositiva"),
    ("Slide Size", "Tamaño de diapositiva"),
    ("Page Setup…", "Configurar página…"),
    ("Background", "Fondo"),
    ("Foreground", "Primer plano"),
    // Transitions and animations.
    ("Preview", "Vista previa"),
    ("Effect Options", "Opciones de efectos"),
    ("Sound", "Sonido"),
    ("Duration", "Duración"),
    ("Delay", "Retraso"),
    ("Apply To All", "Aplicar a todo"),
    ("On Mouse Click", "Al hacer clic con el mouse"),
    ("After", "Después de"),
    ("Fade", "Desvanecer"),
    ("Appear", "Aparecer"),
    ("Wipe", "Barrido"),
    ("Split", "Dividir"),
    ("Cut", "Cortar"),
    ("Push", "Empujar"),
    ("Cover", "Cubrir"),
    ("Morph", "Transformación"),
    ("Add Animation", "Agregar animación"),
    ("Animation Pane", "Panel de animación"),
    ("Animation Painter", "Copiar animación"),
    ("Preview Animations", "Vista previa de animaciones"),
    ("Previewing animations", "Vista previa de animaciones"),
    ("This slide has no animations", "Esta diapositiva no tiene animaciones"),
    ("Trigger", "Desencadenador"),
    ("On Click of… (none)", "Al hacer clic en… (ninguno)"),
    ("Start On Click", "Iniciar al hacer clic"),
    ("Start With Previous", "Iniciar con la anterior"),
    ("Start After Previous", "Iniciar después de la anterior"),
    ("On Click", "Al hacer clic"),
    ("With Previous", "Con la anterior"),
    ("After Previous", "Después de la anterior"),
    ("Entrance", "Entrada"),
    ("Emphasis", "Énfasis"),
    ("Exit", "Salir"),
    ("Motion Paths", "Trayectorias de la animación"),
    ("Direction", "Dirección"),
    ("Move Up", "Subir"),
    ("Move Down", "Bajar"),
    ("By Paragraph", "Por párrafo"),
    ("As One Object", "Como un objeto"),
    // Slide Show.
    ("From Beginning", "Desde el principio"),
    ("From Current Slide", "Desde la diapositiva actual"),
    ("Play from Start", "Reproducir desde el inicio"),
    ("Play from Current Slide", "Reproducir desde la diapositiva actual"),
    ("Start from Beginning", "Comenzar desde el principio"),
    ("Custom Show", "Presentación personalizada"),
    ("Custom Show…", "Presentación personalizada…"),
    ("Custom Shows", "Presentaciones personalizadas"),
    ("Set Up Slide Show", "Configurar presentación"),
    ("Set Up Show…", "Configurar presentación…"),
    ("Rehearse Timings", "Ensayar intervalos"),
    ("Use Timings", "Usar intervalos"),
    ("Use timings, if present", "Usar intervalos guardados, si existen"),
    ("Slide timings recorded.", "Intervalos de diapositivas grabados."),
    ("Play Narrations", "Reproducir narraciones"),
    ("Show Media Controls", "Mostrar controles multimedia"),
    ("Presenter View", "Vista del moderador"),
    ("Keep Mouse Cursor Visible", "Mantener visible el puntero"),
    ("Keep mouse cursor visible", "Mantener visible el puntero"),
    ("End Show", "Finalizar presentación"),
    ("End of slide show", "Fin de la presentación"),
    ("End of slide show, click to exit.", "Fin de la presentación, haga clic para salir."),
    ("Show type", "Tipo de presentación"),
    ("Presented by a speaker (full screen)", "Realizada por un orador (pantalla completa)"),
    ("Browsed by an individual (window)", "Examinada de forma individual (ventana)"),
    ("Browsed at a kiosk (full screen)", "Examinada en exposición (pantalla completa)"),
    ("Show without narration", "Presentación sin narración"),
    ("Show without animation", "Presentación sin animación"),
    ("Loop forever", "Repetir el ciclo hasta presionar Esc"),
    ("All slides", "Todas las diapositivas"),
    ("Next Slide", "Siguiente diapositiva"),
    ("Previous Slide", "Diapositiva anterior"),
    ("First Slide", "Primera diapositiva"),
    ("Last Slide", "Última diapositiva"),
    ("Last Slide Viewed", "Última diapositiva vista"),
    ("Hidden slides", "Diapositivas ocultas"),
    // Review.
    ("Spelling", "Ortografía"),
    ("Thesaurus", "Sinónimos"),
    ("Check Accessibility", "Comprobar accesibilidad"),
    ("Accessibility Checker", "Comprobador de accesibilidad"),
    ("Translate", "Traducir"),
    ("Language", "Idioma"),
    ("Show Comments", "Mostrar comentarios"),
    ("Compare", "Comparar"),
    ("Compare…", "Comparar…"),
    ("Accept", "Aceptar"),
    ("Accept All", "Aceptar todo"),
    ("Reject", "Rechazar"),
    ("End Review", "Finalizar revisión"),
    ("Revisions", "Revisiones"),
    ("Add to Dictionary", "Agregar al diccionario"),
    ("Ignore All", "Omitir todo"),
    ("Delete Repeated Word", "Eliminar palabra repetida"),
    ("Repeated word", "Palabra repetida"),
    ("Look up", "Buscar"),
    ("Reply…", "Responder…"),
    ("Resolve", "Resolver"),
    ("Post", "Publicar"),
    ("No comments.", "No hay comentarios."),
    ("Do not check spelling or grammar", "No revisar la ortografía ni la gramática"),
    // View.
    ("Outline View", "Vista Esquema"),
    ("Slide Sorter", "Clasificador de diapositivas"),
    ("Notes Page", "Página de notas"),
    ("Reading View", "Vista de lectura"),
    ("Ruler", "Regla"),
    ("Gridlines", "Líneas de cuadrícula"),
    ("Guides", "Guías"),
    ("Notes", "Notas"),
    ("Fit to Window", "Ajustar a la ventana"),
    ("Fit slide to current window", "Ajustar la diapositiva a la ventana"),
    ("Zoom In", "Acercar"),
    ("Zoom Out", "Alejar"),
    ("Set Zoom", "Establecer zoom"),
    ("Zoom level", "Nivel de zoom"),
    ("Enter a zoom percent", "Escriba un porcentaje de zoom"),
    ("Color", "Color"),
    ("Grayscale", "Escala de grises"),
    ("Black and White", "Blanco y negro"),
    ("New Window", "Nueva ventana"),
    ("Dark Mode", "Modo oscuro"),
    ("Dark appearance", "Apariencia oscura"),
    ("Light/Dark appearance", "Apariencia clara/oscura"),
    ("Grid and Guides", "Cuadrícula y guías"),
    ("Grid Options", "Opciones de cuadrícula"),
    ("Grid Options…", "Opciones de cuadrícula…"),
    ("Snap objects to grid", "Ajustar objetos a la cuadrícula"),
    ("Snap objects to other objects", "Ajustar objetos a otros objetos"),
    ("Display grid on screen", "Mostrar cuadrícula en la pantalla"),
    ("Display drawing guides on screen", "Mostrar guías de dibujo en la pantalla"),
    ("Add Horizontal Guide", "Agregar guía horizontal"),
    ("Add Vertical Guide", "Agregar guía vertical"),
    ("Show smart guides", "Mostrar guías inteligentes"),
    ("Format Pane", "Panel de formato"),
    ("Task Pane", "Panel de tareas"),
    ("Close Master View", "Cerrar vista Patrón"),
    ("Insert Layout", "Insertar diseño"),
    ("Insert Placeholder", "Insertar marcador de posición"),
    ("Rename Layout", "Cambiar nombre del diseño"),
    // Shape / picture format.
    ("Format Shape", "Formato de forma"),
    ("Format Shape…", "Formato de forma…"),
    ("Edit Shape", "Editar forma"),
    ("Change Shape", "Cambiar forma"),
    ("Edit Points", "Modificar puntos"),
    ("Exit Edit Points", "Salir de Modificar puntos"),
    ("Merge Shapes", "Combinar formas"),
    ("Shape Styles", "Estilos de forma"),
    ("Text Fill", "Relleno de texto"),
    ("Text Outline", "Contorno de texto"),
    ("Text Effects", "Efectos de texto"),
    ("Text Options", "Opciones de texto"),
    ("Text Pane", "Panel de texto"),
    ("Set as Default Shape", "Establecer como forma predeterminada"),
    ("Edit Alt Text…", "Editar texto alternativo…"),
    ("Alt Text", "Texto alternativo"),
    ("Mark as decorative", "Marcar como decorativo"),
    ("Height", "Alto"),
    ("Width", "Ancho"),
    ("Lock aspect ratio", "Bloquear relación de aspecto"),
    ("Size and Position", "Tamaño y posición"),
    ("Size and Position…", "Tamaño y posición…"),
    ("Size & Properties", "Tamaño y propiedades"),
    ("Fill & Line", "Relleno y línea"),
    ("Effects", "Efectos"),
    ("Fill", "Relleno"),
    ("Line", "Línea"),
    ("No Fill", "Sin relleno"),
    ("No fill", "Sin relleno"),
    ("Solid fill", "Relleno sólido"),
    ("Gradient fill", "Relleno degradado"),
    ("Picture or texture fill", "Relleno con imagen o textura"),
    ("Pattern fill", "Relleno de trama"),
    ("No line", "Sin línea"),
    ("Solid line", "Línea sólida"),
    ("No Outline", "Sin contorno"),
    ("No Color", "Sin color"),
    ("More Colors", "Más colores"),
    ("More Colors…", "Más colores…"),
    ("Standard Colors", "Colores estándar"),
    ("Eyedropper", "Cuentagotas"),
    ("Transparency", "Transparencia"),
    ("Weight", "Grosor"),
    ("Dashes", "Guiones"),
    ("Arrows", "Flechas"),
    ("Glow", "Iluminado"),
    ("Reflection", "Reflexión"),
    ("Soft Edges", "Bordes suaves"),
    ("Bevel", "Bisel"),
    ("Rotation", "Giro"),
    ("No Shadow", "Sin sombra"),
    ("No Glow", "Sin iluminado"),
    ("No Reflection", "Sin reflexión"),
    ("No Soft Edges", "Sin bordes suaves"),
    ("Transform", "Transformar"),
    ("No Transform", "Sin transformación"),
    ("Wrap text in shape", "Ajustar texto en forma"),
    ("Resize shape to fit text", "Ajustar tamaño de la forma al texto"),
    ("Shrink text on overflow", "Comprimir el texto al desbordarse"),
    ("Do not Autofit", "No autoajustar"),
    ("Vertical alignment", "Alineación vertical"),
    ("Crop", "Recortar"),
    ("Crop to Shape", "Recortar a la forma"),
    ("Reset Crop", "Restablecer recorte"),
    ("Remove Background", "Quitar fondo"),
    ("Corrections", "Correcciones"),
    ("Artistic Effects", "Efectos artísticos"),
    ("Compress Pictures", "Comprimir imágenes"),
    ("Change Picture", "Cambiar imagen"),
    ("Change Picture…", "Cambiar imagen…"),
    ("Reset Picture", "Restablecer imagen"),
    ("Picture Border", "Borde de imagen"),
    ("Picture Effects", "Efectos de imagen"),
    ("Brightness", "Brillo"),
    ("Contrast", "Contraste"),
    ("Saturation", "Saturación"),
    ("Recolor", "Volver a colorear"),
    ("Set Transparent Color", "Definir color transparente"),
    ("Delete cropped areas of pictures", "Eliminar áreas recortadas de las imágenes"),
    // Tables and charts.
    ("Insert Above", "Insertar arriba"),
    ("Insert Below", "Insertar debajo"),
    ("Insert Left", "Insertar a la izquierda"),
    ("Insert Right", "Insertar a la derecha"),
    ("Delete Rows", "Eliminar filas"),
    ("Delete Columns", "Eliminar columnas"),
    ("Delete Table", "Eliminar tabla"),
    ("Merge Cells", "Combinar celdas"),
    ("Split Cells", "Dividir celdas"),
    ("Distribute Rows", "Distribuir filas"),
    ("Distribute Columns", "Distribuir columnas"),
    ("Cell Margins", "Márgenes de celda"),
    ("Header Row", "Fila de encabezado"),
    ("Total Row", "Fila de totales"),
    ("Banded Rows", "Filas con bandas"),
    ("Banded Columns", "Columnas con bandas"),
    ("First Column", "Primera columna"),
    ("Last Column", "Última columna"),
    ("Shading", "Sombreado"),
    ("Borders", "Bordes"),
    ("No Border", "Sin borde"),
    ("All Borders", "Todos los bordes"),
    ("Outside Borders", "Bordes externos"),
    ("Inside Borders", "Bordes internos"),
    ("Top Border", "Borde superior"),
    ("Bottom Border", "Borde inferior"),
    ("Left Border", "Borde izquierdo"),
    ("Right Border", "Borde derecho"),
    ("Table Eraser", "Borrador de tabla"),
    ("Edit Data", "Editar datos"),
    ("Chart Data", "Datos del gráfico"),
    ("Change Chart Type", "Cambiar tipo de gráfico"),
    ("Change Colors", "Cambiar colores"),
    ("Add Shape", "Agregar forma"),
    ("Promote", "Promover"),
    ("Demote", "Disminuir nivel"),
    ("Convert to Text", "Convertir en texto"),
    ("Convert to Shapes", "Convertir en formas"),
    ("Choose a SmartArt Graphic", "Elegir un gráfico SmartArt"),
    // Media.
    ("Play", "Reproducir"),
    ("Pause", "Pausa"),
    ("Mute", "Silenciar"),
    ("Volume", "Volumen"),
    ("Trim Media", "Recortar multimedia"),
    ("Trim", "Recortar"),
    ("Play Full Screen", "Reproducir a pantalla completa"),
    ("Hide During Show", "Ocultar durante la presentación"),
    ("Loop until Stopped", "Repetir la reproducción hasta su interrupción"),
    ("Rewind after Playing", "Rebobinar después de la reproducción"),
    ("Play Across Slides", "Reproducir en todas las diapositivas"),
    ("Start Automatically", "Iniciar automáticamente"),
    // Dialogs, panes, status.
    ("Tabs", "Tabulaciones"),
    ("Tabs…", "Tabulaciones…"),
    ("AutoCorrect", "Autocorrección"),
    ("AutoCorrect Options…", "Opciones de Autocorrección…"),
    ("AutoCorrect as you type", "Autocorrección mientras escribe"),
    ("Replace text as you type", "Reemplazar texto mientras escribe"),
    ("Replace straight quotes with smart quotes", "Reemplazar comillas rectas por tipográficas"),
    ("Capitalize first letter of sentences", "Poner en mayúscula la primera letra de una oración"),
    ("Capitalize names of days", "Poner en mayúscula los nombres de días"),
    ("Correct TWo INitial CApitals", "Corregir DOs MAyúsculas SEguidas"),
    ("Correct accidental use of cAPS LOCK key", "Corregir el uso accidental de bLOQ mAYÚS"),
    ("Automatic bulleted and numbered lists", "Listas automáticas con viñetas y numeración"),
    ("Ordinals (1st) with superscript", "Ordinales (1º) en superíndice"),
    ("Measurement units:", "Unidades de medida:"),
    ("Inches", "Pulgadas"),
    ("Centimeters", "Centímetros"),
    ("Your name (comments):", "Su nombre (comentarios):"),
    ("Interface size", "Tamaño de la interfaz"),
    ("Indentation", "Sangría"),
    ("Spacing", "Espaciado"),
    ("Spacing before", "Espaciado anterior"),
    ("Spacing after", "Espaciado posterior"),
    ("Before text", "Antes del texto"),
    ("After text", "Después del texto"),
    ("Special", "Especial"),
    ("First line", "Primera línea"),
    ("Hanging", "Francesa"),
    ("Exactly", "Exacto"),
    ("Multiple", "Múltiple"),
    ("Single", "Sencillo"),
    ("Double", "Doble"),
    ("Alignment", "Alineación"),
    ("Left", "Izquierda"),
    ("Right", "Derecha"),
    ("Center", "Centro"),
    ("Centered", "Centrado"),
    ("Top", "Arriba"),
    ("Middle", "Medio"),
    ("Bottom", "Abajo"),
    ("Top Left", "Superior izquierda"),
    ("Top Right", "Superior derecha"),
    ("Bottom Left", "Inferior izquierda"),
    ("Bottom Right", "Inferior derecha"),
    ("Horizontal", "Horizontal"),
    ("Vertical", "Vertical"),
    ("Left Tab", "Tabulación izquierda"),
    ("Right Tab", "Tabulación derecha"),
    ("Center Tab", "Tabulación centrada"),
    ("Decimal Tab", "Tabulación decimal"),
    ("Slide", "Diapositiva"),
    ("Slides", "Diapositivas"),
    ("Presentation", "Presentación"),
    ("Whole presentation", "Toda la presentación"),
    ("Selected text", "Texto seleccionado"),
    ("Click to add title", "Haga clic para agregar título"),
    ("Click to add text", "Haga clic para agregar texto"),
    ("Click to add notes", "Haga clic para agregar notas"),
    ("Click to add first slide", "Haga clic para agregar la primera diapositiva"),
    ("No presentation is open.", "No hay ninguna presentación abierta."),
    ("No notes.", "No hay notas."),
    ("No slides.", "No hay diapositivas."),
    ("Select a shape to format it.", "Seleccione una forma para darle formato."),
    ("Select a chart first.", "Seleccione primero un gráfico."),
    ("Type your text here", "Escriba aquí el texto"),
    ("Type the text for the new text box", "Escriba el texto del nuevo cuadro de texto"),
    ("Search for a word", "Buscar una palabra"),
    ("Window title bar", "Barra de título de la ventana"),
    ("Appearance", "Apariencia"),
    ("Display", "Pantalla"),
    ("Light", "Claro"),
    ("Dark", "Oscuro"),
    ("Format", "Formato"),
    ("Pane", "Panel"),
    ("Content", "Contenido"),
    ("Outline", "Esquema"),
    ("Pattern", "Trama"),
    ("Gradient…", "Degradado…"),
    ("Presets", "Preestablecidos"),
    ("Material", "Material"),
    ("Lighting", "Iluminación"),
    ("Depth", "Profundidad"),
    ("Angle", "Ángulo"),
    ("Distance", "Distancia"),
    ("Blur", "Desenfoque"),
    ("Offset X", "Desplazamiento X"),
    ("Offset Y", "Desplazamiento Y"),
    ("Scale Height", "Escala de alto"),
    ("Scale Width", "Escala de ancho"),
    ("Scale height", "Escala de alto"),
    ("Scale width", "Escala de ancho"),
    ("Rotate with shape", "Girar con la forma"),
    ("Tile picture as texture", "Mosaico de imagen como textura"),
    ("Snap", "Ajustar"),
    ("Grid", "Cuadrícula"),
    ("Language", "Idioma"),
    ("Web Page or File", "Página web o archivo"),
    ("This Document", "Este documento"),
    ("Email Address", "Dirección de correo electrónico"),
    ("Help", "Ayuda"),
];

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn tags_parse_to_languages() {
        assert_eq!(Language::parse("es_PE.UTF-8"), Some(Language::Es));
        assert_eq!(Language::parse("es-419"), Some(Language::Es));
        assert_eq!(Language::parse("en_US"), Some(Language::En));
        assert_eq!(Language::parse("fr"), None);
        assert_eq!(Language::parse(""), None);
    }

    #[test]
    fn spanish_table_has_no_empty_entries() {
        for (en, es) in ES {
            assert!(!en.is_empty() && !es.is_empty(), "{en:?} → {es:?}");
        }
        assert_eq!(spanish().get("Home"), Some(&"Inicio"));
    }

    #[test]
    fn second_table_has_no_empty_or_repeated_entries() {
        let mut seen = std::collections::HashSet::new();
        for (en, es) in es_more::ES {
            assert!(!en.is_empty() && !es.is_empty(), "{en:?} → {es:?}");
            assert!(seen.insert(*en), "{en:?} is listed twice");
        }
    }

    /// The ribbon's galleries show these names: each one reads Spanish in the Spanish interface.
    #[test]
    fn every_gallery_name_has_a_spanish_name() {
        let map = spanish();
        let missing: Vec<&str> = deckcraft_model::anim::TRANSITIONS
            .iter()
            .map(|t| t.1)
            .chain(deckcraft_model::anim::ANIMATIONS.iter().map(|a| a.1))
            .chain(deckcraft_engine::cmd::smartart::LAYOUTS.iter().map(|l| l.1))
            .filter(|name| !map.contains_key(name))
            .collect();
        assert!(missing.is_empty(), "no Spanish for {missing:?}");
    }

    #[test]
    fn placeholder_prompts_have_spanish_names() {
        use deckcraft_model::PhType;
        for kind in [PhType::Title, PhType::SubTitle, PhType::Body, PhType::Chart, PhType::Table, PhType::Diagram, PhType::Media, PhType::Picture] {
            assert!(spanish().contains_key(kind.prompt()), "{:?}", kind.prompt());
        }
    }
}
