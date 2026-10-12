//! GridCraft's font database, shared by the screen (`ui-egui`) and the PDF export (`pdf`).
//!
//! - **Installed faces**: the platform's font folders, cataloged by family name ([`catalog`]).
//! - **Registered faces**: font files a host hands over at run time ([`register_font`]). A WASM
//!   build has no font folders, so this is where its fonts come from.
//! - **Substitution**: a workbook font that isn't available is replaced by its metric-compatible
//!   open twin ([`metric_twin`]: Calibri → Carlito, Arial → Liberation Sans…), then by the role it
//!   falls in ([`cell_role`]: cell, serif or mono, each with its candidate files).
//! - **Script fallbacks**: faces for CJK, Thai, Hebrew and symbols ([`fallback_faces`]), so text in
//!   those scripts renders instead of turning into boxes.
//!
//! [`resolve`] puts the four together into the ordered chain of faces one font name draws with.
//! Nothing here is bundled: every face comes from the machine or the host.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

// ---------------------------------------------------------------- roles and substitution

/// The interface font.
pub const UI: &str = "ui";
/// The bold interface font.
pub const UI_BOLD: &str = "ui-bold";
/// Cell text whose font isn't available (Calibri's stand-ins).
pub const CELL: &str = "cell";
pub const CELL_BOLD: &str = "cell-bold";
pub const CELL_ITALIC: &str = "cell-italic";
pub const CELL_BOLD_ITALIC: &str = "cell-bold-italic";
/// Serif cell fonts (Times, Cambria, Georgia…).
pub const SERIF: &str = "serif";
/// Monospaced cell fonts (Courier, Consolas…).
pub const MONO: &str = "mono";

/// The role a cell font name falls back to when the font itself isn't available: monospaced and
/// serif names keep their look, everything else uses the cell font in the right weight and slant.
pub fn cell_role(name: &str, bold: bool, italic: bool) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if lower.contains("courier") || lower.contains("mono") || lower.contains("consolas") {
        return MONO;
    }
    if lower.contains("times")
        || lower.contains("georgia")
        || lower.contains("cambria")
        || lower.contains("serif") && !lower.contains("sans")
        || lower.contains("garamond")
    {
        return SERIF;
    }
    match (bold, italic) {
        (true, true) => CELL_BOLD_ITALIC,
        (true, false) => CELL_BOLD,
        (false, true) => CELL_ITALIC,
        (false, false) => CELL,
    }
}

/// The open font drawn with the same advance widths as an Office font, so text set in it lines up
/// and wraps like the original.
pub fn metric_twin(name: &str) -> Option<&'static str> {
    Some(match name.trim().to_ascii_lowercase().as_str() {
        "calibri" => "Carlito",
        "cambria" => "Caladea",
        "arial" | "helvetica" => "Liberation Sans",
        "arial narrow" => "Liberation Sans Narrow",
        "times new roman" | "times" => "Liberation Serif",
        "courier new" | "courier" => "Liberation Mono",
        _ => return None,
    })
}

/// The families a role substitutes, in preference order, for faces matched by name (registered
/// faces). The same order as the files [`candidates`] lists for installed ones.
pub fn role_families(role: &str) -> &'static [&'static str] {
    match role {
        UI | UI_BOLD => &["Segoe UI", "Helvetica Neue", "Cantarell", "Noto Sans", "DejaVu Sans", "Liberation Sans"],
        CELL | CELL_BOLD | CELL_ITALIC | CELL_BOLD_ITALIC => &["Calibri", "Carlito", "Arial", "Liberation Sans", "DejaVu Sans"],
        SERIF => &["Times New Roman", "Liberation Serif", "DejaVu Serif"],
        MONO => &["SF Mono", "Menlo", "Consolas", "DejaVu Sans Mono", "Liberation Mono"],
        _ => &[],
    }
}

fn windows_fonts() -> String {
    std::env::var("WINDIR").map(|w| format!("{w}\\Fonts")).unwrap_or_else(|_| "C:\\Windows\\Fonts".into())
}

fn home_fonts(file: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{home}/Library/Fonts/{file}")
}

/// Candidate font files per role, in preference order, as `(path, face index)`.
pub fn candidates(role: &str) -> Vec<(String, u32)> {
    let mac_sys = "/System/Library/Fonts";
    let mac_sup = "/System/Library/Fonts/Supplemental";
    let win = windows_fonts();
    let lin = ["/usr/share/fonts/truetype", "/usr/share/fonts", "/usr/local/share/fonts", "/usr/share/fonts/TTF"];
    let mut v: Vec<(String, u32)> = Vec::new();
    let mut add = |s: String, i: u32| v.push((s, i));
    match role {
        UI => {
            add(format!("{mac_sys}/SFNS.ttf"), 0);
            add(format!("{mac_sys}/HelveticaNeue.ttc"), 0);
            add(format!("{win}\\segoeui.ttf"), 0);
            for d in lin {
                add(format!("{d}/cantarell/Cantarell-Regular.otf"), 0);
                add(format!("{d}/noto/NotoSans-Regular.ttf"), 0);
                add(format!("{d}/dejavu/DejaVuSans.ttf"), 0);
                add(format!("{d}/liberation/LiberationSans-Regular.ttf"), 0);
            }
            // Fedora keeps each family in its own package folder (#54).
            add("/usr/share/fonts/abattis-cantarell-fonts/Cantarell-Regular.otf".into(), 0);
            add("/usr/share/fonts/google-noto/NotoSans-Regular.ttf".into(), 0);
            add("/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf".into(), 0);
            add("/usr/share/fonts/liberation-sans-fonts/LiberationSans-Regular.ttf".into(), 0);
        }
        UI_BOLD => {
            add(format!("{mac_sys}/HelveticaNeue.ttc"), 1);
            add(format!("{win}\\segoeuib.ttf"), 0);
            for d in lin {
                add(format!("{d}/noto/NotoSans-Bold.ttf"), 0);
                add(format!("{d}/dejavu/DejaVuSans-Bold.ttf"), 0);
                add(format!("{d}/liberation/LiberationSans-Bold.ttf"), 0);
            }
            add("/usr/share/fonts/google-noto/NotoSans-Bold.ttf".into(), 0);
            add("/usr/share/fonts/dejavu-sans-fonts/DejaVuSans-Bold.ttf".into(), 0);
            add("/usr/share/fonts/liberation-sans-fonts/LiberationSans-Bold.ttf".into(), 0);
        }
        CELL | CELL_BOLD | CELL_ITALIC | CELL_BOLD_ITALIC => {
            let (carlito, calibri_win, calibri_mac, arial_mac, arial_win, lib, dejavu) = match role {
                CELL => {
                    ("Carlito-Regular.ttf", "calibri.ttf", "Calibri.ttf", "Arial.ttf", "arial.ttf", "LiberationSans-Regular.ttf", "DejaVuSans.ttf")
                }
                CELL_BOLD => (
                    "Carlito-Bold.ttf",
                    "calibrib.ttf",
                    "Calibri Bold.ttf",
                    "Arial Bold.ttf",
                    "arialbd.ttf",
                    "LiberationSans-Bold.ttf",
                    "DejaVuSans-Bold.ttf",
                ),
                CELL_ITALIC => (
                    "Carlito-Italic.ttf",
                    "calibrii.ttf",
                    "Calibri Italic.ttf",
                    "Arial Italic.ttf",
                    "ariali.ttf",
                    "LiberationSans-Italic.ttf",
                    "DejaVuSans-Oblique.ttf",
                ),
                _ => (
                    "Carlito-BoldItalic.ttf",
                    "calibriz.ttf",
                    "Calibri Bold Italic.ttf",
                    "Arial Bold Italic.ttf",
                    "arialbi.ttf",
                    "LiberationSans-BoldItalic.ttf",
                    "DejaVuSans-BoldOblique.ttf",
                ),
            };
            // Calibri if the user has it installed system-wide (never from an Office bundle).
            add(format!("/Library/Fonts/{calibri_mac}"), 0);
            add(home_fonts(calibri_mac), 0);
            add(format!("{win}\\{calibri_win}"), 0);
            for d in lin {
                add(format!("{d}/crosextra/{carlito}"), 0);
                add(format!("{d}/carlito/{carlito}"), 0);
            }
            add(home_fonts(carlito), 0);
            add(format!("{mac_sup}/{arial_mac}"), 0);
            add(format!("{win}\\{arial_win}"), 0);
            for d in lin {
                add(format!("{d}/liberation/{lib}"), 0);
                add(format!("{d}/dejavu/{dejavu}"), 0);
            }
        }
        SERIF => {
            add(format!("{mac_sup}/Times New Roman.ttf"), 0);
            add(format!("{win}\\times.ttf"), 0);
            for d in lin {
                add(format!("{d}/liberation/LiberationSerif-Regular.ttf"), 0);
                add(format!("{d}/dejavu/DejaVuSerif.ttf"), 0);
            }
        }
        MONO => {
            add(format!("{mac_sys}/SFNSMono.ttf"), 0);
            add(format!("{mac_sys}/Menlo.ttc"), 0);
            add(format!("{win}\\consola.ttf"), 0);
            for d in lin {
                add(format!("{d}/dejavu/DejaVuSansMono.ttf"), 0);
                add(format!("{d}/liberation/LiberationMono-Regular.ttf"), 0);
            }
        }
        _ => {}
    }
    v
}

// ---------------------------------------------------------------- script fallbacks

/// Which Han face leads the fallback chain. Both Han faces cover kana and most kanji, so this only
/// decides glyph shapes (and which face catches a glyph the other lacks).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HanOrder {
    /// Japanese glyph shapes first.
    Japanese,
    /// Simplified Chinese first: the widest Han coverage.
    Chinese,
}

/// Every fallback face we look for, in chain order: `(key, candidate files)`. One file per script
/// family is enough: a renderer picks the first face with the glyph, and a script's glyphs live in
/// one file. Order only matters between faces that both cover a glyph (Japanese vs Chinese kanji),
/// so [`HanOrder`] reorders the two Han entries and nothing else.
pub fn fallback_faces(han: HanOrder) -> Vec<(&'static str, Vec<(String, u32)>)> {
    let ja = ("sys-cjk-han-ja", cjk_han_japanese());
    let zh = ("sys-cjk-han-zh", cjk_han_simplified());
    let (first, second) = match han {
        HanOrder::Japanese => (ja, zh),
        HanOrder::Chinese => (zh, ja),
    };
    vec![
        first,
        second,
        ("sys-cjk-hangul", cjk_hangul()),
        ("sys-thai", thai()),
        ("sys-hebrew", hebrew()),
        ("sys-symbols", symbols()),
        ("sys-symbols-2", symbols_more()),
    ]
}

/// Linux directories that hold CJK fonts across distributions: Debian/Ubuntu (`opentype/noto`,
/// `truetype/wqy`), Fedora (`google-noto-cjk`), Arch (`noto-cjk`, `wenquanyi`), plus local and
/// flat layouts.
const LINUX_CJK_DIRS: [&str; 10] = [
    "/usr/share/fonts/opentype/noto",
    "/usr/share/fonts/google-noto-cjk",
    "/usr/share/fonts/noto-cjk",
    "/usr/share/fonts/truetype/noto",
    "/usr/share/fonts/noto",
    "/usr/share/fonts/truetype",
    "/usr/share/fonts/TTF",
    "/usr/share/fonts",
    "/usr/local/share/fonts",
    "/usr/local/share/fonts/noto-cjk",
];

/// Japanese Han + Kana faces (Japanese kanji glyph shapes).
///
/// The region-merged `NotoSansCJK-Regular.ttc` holds one face per locale (JP 0, KR 1, SC 2, TC 3,
/// HK 4) with the same glyph coverage, so the index only picks glyph shapes. Whichever Han list
/// comes first loads it; a file already loaded for an earlier script is not loaded again.
pub fn cjk_han_japanese() -> Vec<(String, u32)> {
    let mac = "/System/Library/Fonts";
    let win = windows_fonts();
    let mut v: Vec<(String, u32)> = Vec::new();
    v.push((format!("{win}\\YuGothR.ttc"), 0)); // Yu Gothic
    v.push((format!("{win}\\msgothic.ttc"), 0)); // MS Gothic
    v.push((format!("{mac}/ヒラギノ角ゴシック W3.ttc"), 0)); // Hiragino Sans
    for d in LINUX_CJK_DIRS {
        v.push((format!("{d}/NotoSansCJKjp-Regular.otf"), 0));
        v.push((format!("{d}/NotoSansCJK-Regular.ttc"), 0)); // JP face
    }
    v
}

/// Simplified Chinese Han faces. Kept apart from the Japanese ones so a glyph one family lacks
/// (simplified-only hanzi in a Japanese font) still falls through to a real glyph.
pub fn cjk_han_simplified() -> Vec<(String, u32)> {
    let mac = "/System/Library/Fonts";
    let win = windows_fonts();
    let mut v: Vec<(String, u32)> = Vec::new();
    v.push((format!("{win}\\msyh.ttc"), 0)); // Microsoft YaHei
    v.push((format!("{win}\\simsun.ttc"), 0)); // SimSun
    v.push((format!("{win}\\simhei.ttf"), 0)); // SimHei
    v.push((format!("{mac}/Hiragino Sans GB.ttc"), 0)); // ships on every recent macOS
    v.push((format!("{mac}/PingFang.ttc"), 0)); // older macOS only; now a downloadable asset
    for d in LINUX_CJK_DIRS {
        v.push((format!("{d}/NotoSansCJKsc-Regular.otf"), 0));
        v.push((format!("{d}/NotoSansCJK-Regular.ttc"), 2)); // SC face
    }
    for d in LINUX_CJK_DIRS {
        v.push((format!("{d}/wqy/wqy-microhei.ttc"), 0));
        v.push((format!("{d}/wenquanyi/wqy-microhei.ttc"), 0));
        v.push((format!("{d}/wqy-microhei.ttc"), 0));
    }
    v
}

/// Hangul faces (Korean). The region-merged Noto Sans CJK file found for Han already carries
/// Hangul; it is then not loaded twice.
pub fn cjk_hangul() -> Vec<(String, u32)> {
    let mac = "/System/Library/Fonts";
    let win = windows_fonts();
    let mut v: Vec<(String, u32)> = Vec::new();
    v.push((format!("{win}\\malgun.ttf"), 0)); // Malgun Gothic
    v.push((format!("{mac}/AppleSDGothicNeo.ttc"), 0));
    for d in LINUX_CJK_DIRS {
        v.push((format!("{d}/NotoSansCJKkr-Regular.otf"), 0));
        v.push((format!("{d}/NotoSansCJK-Regular.ttc"), 1)); // KR face
    }
    v
}

/// Thai faces. Windows ships Leelawadee UI and Tahoma/Microsoft Sans Serif (both carry Thai);
/// macOS has Thonburi; Linux usually has Noto Sans Thai or Garuda.
pub fn thai() -> Vec<(String, u32)> {
    let mac_sys = "/System/Library/Fonts";
    let mac_sup = "/System/Library/Fonts/Supplemental";
    let win = windows_fonts();
    let lin = ["/usr/share/fonts/truetype/noto", "/usr/share/fonts/opentype/noto", "/usr/share/fonts/truetype", "/usr/share/fonts"];
    let mut v: Vec<(String, u32)> = Vec::new();
    v.push((format!("{win}\\LeelawUI.ttf"), 0)); // Leelawadee UI
    v.push((format!("{win}\\tahoma.ttf"), 0)); // Tahoma
    v.push((format!("{win}\\micross.ttf"), 0)); // Microsoft Sans Serif
    v.push((format!("{mac_sup}/Thonburi.ttc"), 0));
    v.push((format!("{mac_sys}/SukhumvitSet.ttc"), 0));
    for d in lin {
        v.push((format!("{d}/NotoSansThai-Regular.ttf"), 0));
        v.push((format!("{d}/tlwg/Garuda.ttf"), 0));
        v.push((format!("{d}/garuda/Garuda.ttf"), 0));
    }
    v
}

/// Hebrew faces. Windows has David and FrankRuehl (and Tahoma/Courier New carry Hebrew); macOS has
/// Arial Hebrew and David; Linux usually has Noto Sans Hebrew or DejaVu Sans.
pub fn hebrew() -> Vec<(String, u32)> {
    let mac_sys = "/System/Library/Fonts";
    let mac_sup = "/System/Library/Fonts/Supplemental";
    let win = windows_fonts();
    let lin = ["/usr/share/fonts/truetype/noto", "/usr/share/fonts/opentype/noto", "/usr/share/fonts/truetype", "/usr/share/fonts"];
    let mut v: Vec<(String, u32)> = Vec::new();
    v.push((format!("{win}\\david.ttf"), 0)); // David
    v.push((format!("{win}\\frank.ttf"), 0)); // FrankRuehl
    v.push((format!("{win}\\tahoma.ttf"), 0)); // Tahoma
    v.push((format!("{mac_sys}/ArialHB.ttc"), 0)); // Arial Hebrew
    v.push((format!("{mac_sys}/SFHebrew.ttf"), 0));
    v.push((format!("{mac_sup}/Arial Hebrew.ttf"), 0));
    v.push((format!("{mac_sup}/David.ttf"), 0));
    v.push((format!("{mac_sys}/LucidaGrande.ttc"), 0));
    v.push((format!("{mac_sys}/Lucida Grande.ttc"), 0));
    for d in lin {
        v.push((format!("{d}/NotoSansHebrew-Regular.ttf"), 0));
        v.push((format!("{d}/dejavu/DejaVuSans.ttf"), 0));
        v.push((format!("{d}/freefont/FreeSans.ttf"), 0));
    }
    v
}

/// Symbol faces. Interface fonts such as Noto Sans and Cantarell lack arrows, check marks,
/// geometric shapes and keyboard symbols (▾ ✓ ✕ ▲ ↳ ⇧), which then render as empty boxes (#197,
/// #54). DejaVu Sans covers most of them; the paths cover Debian/Ubuntu, Fedora and Arch layouts.
pub fn symbols() -> Vec<(String, u32)> {
    let win = windows_fonts();
    let mut v: Vec<(String, u32)> = vec![(format!("{win}\\seguisym.ttf"), 0), ("/System/Library/Fonts/Apple Symbols.ttf".into(), 0)];
    for d in [
        "/usr/share/fonts/truetype/dejavu",
        "/usr/share/fonts/dejavu-sans-fonts",
        "/usr/share/fonts/TTF",
        "/usr/share/fonts/dejavu",
        "/usr/share/fonts/truetype/ttf-dejavu",
        "/usr/local/share/fonts/dejavu",
        "/usr/local/share/fonts",
    ] {
        v.push((format!("{d}/DejaVuSans.ttf"), 0));
    }
    v
}

/// A second symbol face for what the first lacks (⌘ and other technical symbols): Noto Sans
/// Symbols 2 where it is installed.
pub fn symbols_more() -> Vec<(String, u32)> {
    let mut v: Vec<(String, u32)> = Vec::new();
    for d in [
        "/usr/share/fonts/noto",
        "/usr/share/fonts/truetype/noto",
        "/usr/share/fonts/opentype/noto",
        "/usr/share/fonts/google-noto-sans-symbols-2-fonts",
        "/usr/share/fonts/google-noto",
        "/usr/local/share/fonts/noto",
    ] {
        v.push((format!("{d}/NotoSansSymbols2-Regular.ttf"), 0));
    }
    v
}

/// Whether any symbol fallback face is installed (tests skip their glyph checks without one).
pub fn symbol_face_installed() -> bool {
    symbols().iter().chain(symbols_more().iter()).any(|(p, _)| Path::new(p).exists())
}

// ---------------------------------------------------------------- faces

/// Whether face `index` of this font file opens and maps characters to glyphs. A corrupt file,
/// something that is not an sfnt at all, a collection index past the end or an unreadable
/// character map (a truncated file) is refused, so a bad file costs a fallback, never a crash.
pub fn font_parses(bytes: &[u8], index: u32) -> bool {
    use skrifa::MetadataProvider;
    skrifa::FontRef::from_index(bytes, index).is_ok_and(|font| font.charmap().has_map())
}

/// Weight implied by a style name (`"Bold"` → 700, `"Light"` → 300).
fn style_weight(style: &str) -> f32 {
    let s: String = style.chars().filter(|c| c.is_alphanumeric()).flat_map(|c| c.to_lowercase()).collect();
    const TABLE: &[(&str, f32)] = &[
        ("hairline", 100.0),
        ("thin", 100.0),
        ("extralight", 200.0),
        ("ultralight", 200.0),
        ("light", 300.0),
        ("medium", 500.0),
        ("demibold", 600.0),
        ("semibold", 600.0),
        ("extrabold", 800.0),
        ("ultrabold", 800.0),
        ("bold", 700.0),
        ("black", 900.0),
        ("heavy", 900.0),
    ];
    TABLE.iter().find(|(k, _)| s.contains(k)).map(|(_, w)| *w).unwrap_or(400.0)
}

/// Whether a style name reads as italic or oblique.
fn style_italic(style: &str) -> bool {
    let s: String = style.chars().filter(|c| c.is_alphanumeric()).flat_map(|c| c.to_lowercase()).collect();
    s.contains("italic") || s.contains("oblique")
}

/// Every face in `data` as `(index, family, style)`. Damaged data yields nothing, never a panic.
fn file_faces(data: &[u8]) -> Vec<(u32, String, String)> {
    use skrifa::raw::FileRef;
    use skrifa::{MetadataProvider, string::StringId};
    let count = match FileRef::new(data) {
        Ok(FileRef::Font(_)) => 1,
        Ok(FileRef::Collection(c)) => c.len(),
        Err(_) => return Vec::new(),
    };
    let name = |f: &skrifa::FontRef<'_>, ids: &[StringId]| -> Option<String> {
        ids.iter().find_map(|id| f.localized_strings(*id).english_or_first().map(|s| s.to_string()).filter(|s| !s.is_empty()))
    };
    (0..count)
        .filter_map(|i| {
            let f = skrifa::FontRef::from_index(data, i).ok()?;
            let family = name(&f, &[StringId::TYPOGRAPHIC_FAMILY_NAME, StringId::FAMILY_NAME])?;
            let style = name(&f, &[StringId::TYPOGRAPHIC_SUBFAMILY_NAME, StringId::SUBFAMILY_NAME]).unwrap_or_else(|| "Regular".into());
            Some((i, family, style))
        })
        .collect()
}

/// A face's weight and slant, for [`pick_face`].
pub trait Styled {
    fn weight(&self) -> f32;
    fn italic(&self) -> bool;
}

/// The face to use for a weight and slant, or `None` when there are no faces. Missing bold/italic
/// faces fall back to the regular face, then to any face.
pub fn pick_face<T: Styled>(faces: &[T], bold: bool, italic: bool) -> Option<&T> {
    let regular = || faces.iter().find(|f| !f.italic() && (f.weight() - 400.0).abs() < 1.0);
    let exact = |wanted_bold: bool, wanted_italic: bool| {
        faces.iter().find(|f| f.italic() == wanted_italic && if wanted_bold { f.weight() >= 600.0 } else { f.weight() < 600.0 })
    };
    (match (bold, italic) {
        (true, true) => exact(true, true).or_else(|| exact(true, false)).or_else(|| exact(false, true)),
        (true, false) => exact(true, false),
        (false, true) => exact(false, true),
        (false, false) => None,
    })
    .or_else(regular)
    .or_else(|| faces.first())
}

// ---------------------------------------------------------------- installed faces

/// A font file is skipped by the scan past this size: a catalog only needs each face's name, and a
/// damaged or enormous file must not turn startup into a long read.
#[cfg(not(target_arch = "wasm32"))]
const MAX_FONT_FILE: u64 = 64 << 20;

/// One face of one installed font file.
#[derive(Clone, Debug)]
pub struct Face {
    /// Typographic family name, e.g. `"Segoe UI"`.
    pub family: String,
    /// The file the face lives in.
    pub path: PathBuf,
    /// Face index within the file (collections hold several, e.g. a family's Regular and Bold).
    pub index: u32,
    /// Weight implied by the face's style name (400 = regular).
    pub weight: f32,
    /// Whether the style name reads as italic or oblique.
    pub italic: bool,
}

impl Styled for Face {
    fn weight(&self) -> f32 {
        self.weight
    }
    fn italic(&self) -> bool {
        self.italic
    }
}

/// The font folders of the platform (the system's and the user's). Empty on wasm.
pub fn system_font_dirs() -> Vec<PathBuf> {
    #[cfg(target_arch = "wasm32")]
    {
        Vec::new()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut dirs: Vec<PathBuf> = Vec::new();
        let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from);
        if cfg!(target_os = "macos") {
            dirs.extend(["/System/Library/Fonts", "/System/Library/Fonts/Supplemental", "/Library/Fonts"].map(Into::into));
            if let Some(h) = &home {
                dirs.push(h.join("Library/Fonts"));
            }
        } else if cfg!(windows) {
            let root = std::env::var_os("WINDIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("C:\\Windows"));
            dirs.push(root.join("Fonts"));
            if let Some(l) = std::env::var_os("LOCALAPPDATA") {
                dirs.push(PathBuf::from(l).join("Microsoft\\Windows\\Fonts"));
            }
        } else {
            dirs.extend(["/usr/share/fonts", "/usr/local/share/fonts", "/usr/share/fonts/TTF"].map(Into::into));
            if let Some(h) = &home {
                dirs.push(h.join(".fonts"));
                dirs.push(h.join(".local/share/fonts"));
            }
        }
        dirs
    }
}

/// Catalog every face under `dirs`, recursing into subfolders. A folder reached twice (through a
/// symlink) is read once, so a link back to a parent ends the walk.
#[cfg(not(target_arch = "wasm32"))]
pub fn scan_dirs(dirs: &[PathBuf]) -> Vec<Face> {
    let mut out: Vec<Face> = Vec::new();
    let mut stack: Vec<PathBuf> = dirs.to_vec();
    let mut seen_dirs: HashSet<PathBuf> = HashSet::new();
    while let Some(dir) = stack.pop() {
        let key = std::fs::canonicalize(&dir).unwrap_or_else(|_| dir.clone());
        if !seen_dirs.insert(key) {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let is_font =
                path.extension().and_then(|e| e.to_str()).is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc" | "otc"));
            if !is_font || std::fs::metadata(&path).map(|m| m.len() > MAX_FONT_FILE).unwrap_or(true) {
                continue;
            }
            let Ok(data) = std::fs::read(&path) else { continue };
            for (index, family, style) in file_faces(&data) {
                out.push(Face { family, path: path.clone(), index, weight: style_weight(&style), italic: style_italic(&style) });
            }
        }
    }
    out
}

#[cfg(target_arch = "wasm32")]
pub fn scan_dirs(_dirs: &[PathBuf]) -> Vec<Face> {
    Vec::new()
}

static CATALOG: OnceLock<Vec<Face>> = OnceLock::new();

/// The catalog of installed faces, scanned on first use. Blocks until the scan finishes.
pub fn catalog() -> &'static [Face] {
    CATALOG.get_or_init(|| scan_dirs(&system_font_dirs()))
}

/// The catalog if the scan has finished, else `None`: never blocks, for use while painting.
pub fn catalog_now() -> Option<&'static [Face]> {
    CATALOG.get().map(Vec::as_slice)
}

/// Start cataloging the installed fonts on a background thread, so the first frame doesn't wait for
/// it. A no-op on wasm and after the scan has started.
pub fn start_scan() {
    static STARTED: OnceLock<()> = OnceLock::new();
    STARTED.get_or_init(|| {
        #[cfg(not(target_arch = "wasm32"))]
        if CATALOG.get().is_none() {
            // A failed spawn leaves the scan to the first blocking lookup.
            let _ = std::thread::Builder::new().name("font-scan".into()).spawn(|| {
                catalog();
            });
        }
    });
}

/// Whether the background scan has finished.
pub fn scan_done() -> bool {
    CATALOG.get().is_some()
}

/// The faces of `family` in `cat`, regular-weight first.
fn family_faces(cat: &[Face], family: &str) -> Vec<Face> {
    let mut v: Vec<Face> = cat.iter().filter(|f| f.family.eq_ignore_ascii_case(family)).cloned().collect();
    v.sort_by(|a, b| a.weight.total_cmp(&b.weight).then(a.italic.cmp(&b.italic)).then(a.index.cmp(&b.index)));
    v
}

/// The faces of one installed family, regular-weight first. Never blocks: empty until the
/// background scan finishes.
pub fn faces_of(family: &str) -> Vec<Face> {
    catalog_now().map(|cat| family_faces(cat, family)).unwrap_or_default()
}

/// The subset of `names` that is installed, sorted and deduped. Never blocks.
pub fn installed_among(names: &[String]) -> Vec<String> {
    let Some(cat) = catalog_now() else { return Vec::new() };
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for wanted in names {
        if !seen.insert(wanted.to_lowercase()) {
            continue;
        }
        if let Some(f) = cat.iter().find(|f| f.family.eq_ignore_ascii_case(wanted)) {
            out.push(f.family.clone());
        }
    }
    out.sort_by_key(|a| a.to_lowercase());
    out
}

// ---------------------------------------------------------------- registered faces

/// One face of a font file registered with [`register_font`].
struct Registered {
    family: String,
    weight: f32,
    italic: bool,
    font: FontData,
}

static REGISTERED: RwLock<Vec<Registered>> = RwLock::new(Vec::new());

/// Largest font file [`register_font`] accepts (a full Noto Sans CJK collection is ~120 MB).
pub const MAX_REGISTERED_FONT: usize = 256 << 20;

/// Adds the faces of a font file (TTF, OTF, TTC or OTC bytes) to the database for the rest of the
/// run, and returns how many faces it added: 0 for data that isn't a readable font. A WASM build
/// has no font folders, so its host registers the fonts workbooks should render with (Carlito,
/// Liberation, Noto Sans CJK…) this way at start.
pub fn register_font(bytes: Vec<u8>) -> usize {
    if bytes.len() > MAX_REGISTERED_FONT {
        return 0;
    }
    let data: Arc<[u8]> = bytes.into();
    let faces: Vec<Registered> = file_faces(&data)
        .into_iter()
        .filter(|(index, _, _)| font_parses(&data, *index))
        .map(|(index, family, style)| Registered {
            family,
            weight: style_weight(&style),
            italic: style_italic(&style),
            font: FontData { data: data.clone(), index },
        })
        .collect();
    let added = faces.len();
    if added > 0 {
        REGISTERED.write().unwrap_or_else(|e| e.into_inner()).extend(faces);
        chains().lock().unwrap_or_else(|e| e.into_inner()).clear();
    }
    added
}

// ---------------------------------------------------------------- resolution

/// The bytes of one face, ready to measure and draw with.
#[derive(Clone)]
pub struct FontData {
    pub data: Arc<[u8]>,
    pub index: u32,
}

#[derive(Clone, PartialEq)]
enum Slot {
    File(PathBuf, u32),
    Registered(usize),
}

/// The ordered faces one font name draws with: the font itself (or its stand-in) first, then the
/// script fallbacks. Faces load on first use, so a chain that never meets a Han character never
/// reads a CJK font.
pub struct Chain {
    slots: Vec<(Slot, OnceLock<Option<FontData>>)>,
}

impl Chain {
    pub fn len(&self) -> usize {
        self.slots.len()
    }
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }
    /// Face `i`, or `None` when it can't be read or parsed.
    pub fn face(&self, i: usize) -> Option<&FontData> {
        let (slot, face) = self.slots.get(i)?;
        face.get_or_init(|| match slot {
            Slot::File(path, index) => load_file(path).filter(|d| font_parses(d, *index)).map(|data| FontData { data, index: *index }),
            Slot::Registered(r) => REGISTERED.read().unwrap_or_else(|e| e.into_inner()).get(*r).map(|f| f.font.clone()),
        })
        .as_ref()
    }
}

/// Font files read so far, shared by every chain (a CJK collection is read once per run).
fn load_file(path: &Path) -> Option<Arc<[u8]>> {
    static FILES: OnceLock<Mutex<HashMap<PathBuf, Option<Arc<[u8]>>>>> = OnceLock::new();
    let files = FILES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut files = files.lock().unwrap_or_else(|e| e.into_inner());
    files
        .entry(path.to_path_buf())
        .or_insert_with(|| {
            let ok = std::fs::metadata(path).is_ok_and(|m| m.len() as usize <= MAX_REGISTERED_FONT);
            ok.then(|| std::fs::read(path).ok()).flatten().map(Into::into)
        })
        .clone()
}

type ChainKey = (String, bool, bool);

fn chains() -> &'static Mutex<HashMap<ChainKey, Arc<Chain>>> {
    static CHAINS: OnceLock<Mutex<HashMap<ChainKey, Arc<Chain>>>> = OnceLock::new();
    CHAINS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// The faces text in font `family` (bold, italic) draws with, best first:
/// 1. the family itself, registered or installed (this scans the installed fonts once);
/// 2. its metric-compatible twin ([`metric_twin`]);
/// 3. its role's stand-in ([`cell_role`]), registered by name or from the role's candidate files;
/// 4. the script fallbacks ([`fallback_faces`]), then every other registered face.
///
/// Empty only when the machine and the host offer no font at all.
pub fn resolve(family: &str, bold: bool, italic: bool) -> Arc<Chain> {
    let key = (family.to_lowercase(), bold, italic);
    if let Some(c) = chains().lock().unwrap_or_else(|e| e.into_inner()).get(&key) {
        return c.clone();
    }
    let chain = Arc::new(build_chain(family, bold, italic));
    chains().lock().unwrap_or_else(|e| e.into_inner()).insert(key, chain.clone());
    chain
}

fn build_chain(family: &str, bold: bool, italic: bool) -> Chain {
    let reg = REGISTERED.read().unwrap_or_else(|e| e.into_inner());
    let registered = |name: &str| -> Option<Slot> {
        let faces: Vec<(usize, &Registered)> = reg.iter().enumerate().filter(|(_, f)| f.family.eq_ignore_ascii_case(name)).collect();
        let styled: Vec<&Registered> = faces.iter().map(|(_, f)| *f).collect();
        let pick = pick_face(&styled, bold, italic)?;
        faces.iter().find(|(_, f)| std::ptr::eq(*f, *pick)).map(|(i, _)| Slot::Registered(*i))
    };
    let installed = |name: &str| -> Option<Slot> {
        let faces = family_faces(catalog(), name);
        pick_face(&faces, bold, italic).map(|f| Slot::File(f.path.clone(), f.index))
    };
    let mut slots: Vec<Slot> = Vec::new();
    let push = |slots: &mut Vec<Slot>, s: Slot| {
        if !slots.contains(&s) {
            slots.push(s);
        }
    };
    if let Some(s) = [Some(family), metric_twin(family)].into_iter().flatten().find_map(|n| registered(n).or_else(|| installed(n))) {
        push(&mut slots, s);
    }
    let role = cell_role(family, bold, italic);
    if let Some(s) = role_families(role).iter().find_map(|n| registered(n)) {
        push(&mut slots, s);
    }
    if let Some((p, i)) = candidates(role).into_iter().find(|(p, _)| Path::new(p).is_file()) {
        push(&mut slots, Slot::File(p.into(), i));
    }
    for (_, cands) in fallback_faces(HanOrder::Chinese) {
        for (p, i) in cands {
            let path = PathBuf::from(&p);
            if slots.iter().any(|s| matches!(s, Slot::File(q, _) if *q == path)) {
                break; // a file already in the chain covers this script
            }
            if path.is_file() {
                push(&mut slots, Slot::File(path, i));
                break;
            }
        }
    }
    for i in 0..reg.len() {
        push(&mut slots, Slot::Registered(i));
    }
    Chain { slots: slots.into_iter().map(|s| (s, OnceLock::new())).collect() }
}

impl Styled for &Registered {
    fn weight(&self) -> f32 {
        self.weight
    }
    fn italic(&self) -> bool {
        self.italic
    }
}

#[cfg(test)]
mod tests;
