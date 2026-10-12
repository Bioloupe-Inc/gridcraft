//! Design tokens and fonts.
//!
//! Colours are our own, tuned to read like a modern desktop spreadsheet. Fonts come from the
//! operating system at runtime (nothing bundled): the UI uses the platform UI font, cells use the
//! best available match for the workbook font (Calibri/Carlito/Aptos/Arial…) with egui's built-in
//! fonts as the last resort.

use std::sync::Arc;

use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, Stroke, Visuals};
use gridcraft_fonts::{CELL, CELL_BOLD, CELL_BOLD_ITALIC, CELL_ITALIC, HanOrder, MONO, SERIF, UI, UI_BOLD, candidates, font_parses};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    pub dark: bool,
    pub window: Color32,
    pub ribbon: Color32,
    pub ribbon_border: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_disabled: Color32,
    pub accent: Color32,
    pub accent_dark: Color32,
    pub accent_soft: Color32,
    pub hover: Color32,
    pub pressed: Color32,
    pub separator: Color32,
    pub grid_bg: Color32,
    /// Behind the pages in Page Layout view.
    pub desk: Color32,
    pub gridline: Color32,
    pub header_bg: Color32,
    pub header_text: Color32,
    pub header_line: Color32,
    pub header_sel_bg: Color32,
    pub header_sel_text: Color32,
    pub header_all_bg: Color32,
    pub sel_fill: Color32,
    pub sel_border: Color32,
    pub input_bg: Color32,
    pub input_border: Color32,
    pub tab_bar: Color32,
    pub tab_active: Color32,
    pub status_bar: Color32,
    pub cell_text: Color32,
    pub menu_bg: Color32,
    pub shadow: Color32,
    pub danger: Color32,
    /// Info-bar (notice strip) fills, by severity.
    pub notice_warn_bg: Color32,
    pub notice_warn_border: Color32,
    pub notice_info_bg: Color32,
    pub notice_info_border: Color32,
}

impl Tokens {
    pub fn light() -> Tokens {
        Tokens {
            dark: false,
            window: Color32::from_rgb(0xF3, 0xF3, 0xF3),
            ribbon: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            ribbon_border: Color32::from_rgb(0xE3, 0xE3, 0xE3),
            text: Color32::from_rgb(0x24, 0x24, 0x24),
            text_dim: Color32::from_rgb(0x61, 0x61, 0x61),
            text_disabled: Color32::from_rgb(0xB0, 0xB0, 0xB0),
            accent: Color32::from_rgb(0x10, 0x7C, 0x41),
            accent_dark: Color32::from_rgb(0x0B, 0x5E, 0x31),
            accent_soft: Color32::from_rgb(0xDF, 0xF1, 0xE6),
            hover: Color32::from_rgb(0xEB, 0xEB, 0xEB),
            pressed: Color32::from_rgb(0xDD, 0xDD, 0xDD),
            separator: Color32::from_rgb(0xE0, 0xE0, 0xE0),
            grid_bg: Color32::WHITE,
            desk: Color32::from_rgb(0xDE, 0xDE, 0xDE),
            gridline: Color32::from_rgb(0xE1, 0xE1, 0xE1),
            header_bg: Color32::from_rgb(0xF8, 0xF8, 0xF8),
            header_text: Color32::from_rgb(0x42, 0x42, 0x42),
            header_line: Color32::from_rgb(0xD6, 0xD6, 0xD6),
            header_sel_bg: Color32::from_rgb(0xE1, 0xE1, 0xE1),
            header_sel_text: Color32::from_rgb(0x10, 0x7C, 0x41),
            header_all_bg: Color32::from_rgb(0xC8, 0xE6, 0xD3),
            sel_fill: Color32::from_rgba_unmultiplied(0x10, 0x7C, 0x41, 0x1C),
            sel_border: Color32::from_rgb(0x10, 0x7C, 0x41),
            input_bg: Color32::WHITE,
            input_border: Color32::from_rgb(0xD1, 0xD1, 0xD1),
            tab_bar: Color32::from_rgb(0xF3, 0xF3, 0xF3),
            tab_active: Color32::WHITE,
            status_bar: Color32::from_rgb(0xF3, 0xF3, 0xF3),
            cell_text: Color32::BLACK,
            menu_bg: Color32::WHITE,
            shadow: Color32::from_black_alpha(40),
            danger: Color32::from_rgb(0xC4, 0x2B, 0x1C),
            notice_warn_bg: Color32::from_rgb(0xFF, 0xF4, 0xCE),
            notice_warn_border: Color32::from_rgb(0xF0, 0xC3, 0x6E),
            notice_info_bg: Color32::from_rgb(0xEA, 0xF2, 0xFB),
            notice_info_border: Color32::from_rgb(0xB9, 0xD2, 0xEC),
        }
    }

    pub fn dark() -> Tokens {
        Tokens {
            dark: true,
            window: Color32::from_rgb(0x29, 0x29, 0x29),
            ribbon: Color32::from_rgb(0x33, 0x33, 0x33),
            ribbon_border: Color32::from_rgb(0x40, 0x40, 0x40),
            text: Color32::from_rgb(0xEE, 0xEE, 0xEE),
            text_dim: Color32::from_rgb(0xB0, 0xB0, 0xB0),
            text_disabled: Color32::from_rgb(0x70, 0x70, 0x70),
            accent: Color32::from_rgb(0x5C, 0xC5, 0x87),
            accent_dark: Color32::from_rgb(0x3F, 0xA8, 0x6C),
            accent_soft: Color32::from_rgb(0x23, 0x45, 0x31),
            hover: Color32::from_rgb(0x42, 0x42, 0x42),
            pressed: Color32::from_rgb(0x4D, 0x4D, 0x4D),
            separator: Color32::from_rgb(0x48, 0x48, 0x48),
            grid_bg: Color32::WHITE,
            desk: Color32::from_rgb(0x3A, 0x3A, 0x3A),
            gridline: Color32::from_rgb(0xE1, 0xE1, 0xE1),
            header_bg: Color32::from_rgb(0x30, 0x30, 0x30),
            header_text: Color32::from_rgb(0xCC, 0xCC, 0xCC),
            header_line: Color32::from_rgb(0x45, 0x45, 0x45),
            header_sel_bg: Color32::from_rgb(0x45, 0x45, 0x45),
            header_sel_text: Color32::from_rgb(0x5C, 0xC5, 0x87),
            header_all_bg: Color32::from_rgb(0x2E, 0x55, 0x3E),
            sel_fill: Color32::from_rgba_unmultiplied(0x10, 0x7C, 0x41, 0x24),
            sel_border: Color32::from_rgb(0x10, 0x7C, 0x41),
            input_bg: Color32::from_rgb(0x1F, 0x1F, 0x1F),
            input_border: Color32::from_rgb(0x50, 0x50, 0x50),
            tab_bar: Color32::from_rgb(0x29, 0x29, 0x29),
            tab_active: Color32::from_rgb(0x3A, 0x3A, 0x3A),
            status_bar: Color32::from_rgb(0x29, 0x29, 0x29),
            cell_text: Color32::BLACK,
            menu_bg: Color32::from_rgb(0x30, 0x30, 0x30),
            shadow: Color32::from_black_alpha(90),
            danger: Color32::from_rgb(0xF1, 0x70, 0x5F),
            notice_warn_bg: Color32::from_rgb(0x4A, 0x3F, 0x1E),
            notice_warn_border: Color32::from_rgb(0x7A, 0x67, 0x30),
            notice_info_bg: Color32::from_rgb(0x24, 0x33, 0x44),
            notice_info_border: Color32::from_rgb(0x3B, 0x55, 0x72),
        }
    }

    pub fn get(ctx: &egui::Context) -> Tokens {
        if ctx.global_style().visuals.dark_mode { Tokens::dark() } else { Tokens::light() }
    }
}

pub fn ui_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(UI.into()))
}
pub fn ui_bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(UI_BOLD.into()))
}

/// The font family to use for a cell font name and weight.
///
/// When the name is an installed font whose faces were bound by [`font_definitions`], that family
/// is used; otherwise the name is bucketed to one of the built-in roles (mono/serif/cell). egui
/// panics on a family that was never bound, so the installed branch is gated on
/// [`crate::fonts::is_bound`] and never names a family egui hasn't been given.
pub fn cell_family(name: &str, bold: bool, italic: bool) -> FontFamily {
    let role = crate::fonts::role_key(name, bold, italic);
    if crate::fonts::is_bound(&role) {
        return FontFamily::Name(role.into());
    }
    FontFamily::Name(gridcraft_fonts::cell_role(name, bold, italic).into())
}

/// Loads the first candidate that exists and parses. A file that exists but can't be parsed is
/// skipped with a warning and the next candidate is tried.
#[cfg(not(target_arch = "wasm32"))]
fn try_load(paths: &[(&str, u32)]) -> Option<FontData> {
    for (p, index) in paths {
        if let Ok(bytes) = std::fs::read(p) {
            if !font_parses(&bytes, *index) {
                log::warn!("skipping font {p} (face {index}): not a font file egui can read");
                continue;
            }
            let mut fd = FontData::from_owned(bytes);
            fd.index = *index;
            return Some(fd);
        }
    }
    None
}

/// Last line of defence before [`FontDefinitions`] reach egui: drops every face epaint could not
/// parse (and its key from every family), so a bad font costs a fallback, never a crash.
pub(crate) fn drop_unparseable_fonts(fonts: &mut FontDefinitions) {
    let bad: Vec<String> = fonts.font_data.iter().filter(|(_, fd)| !font_parses(&fd.font, fd.index)).map(|(k, _)| k.clone()).collect();
    for key in &bad {
        log::warn!("dropping font {key:?}: egui can't parse it");
        fonts.font_data.remove(key);
    }
    if !bad.is_empty() {
        for chain in fonts.families.values_mut() {
            chain.retain(|k| !bad.contains(k));
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn try_load(_paths: &[(&str, u32)]) -> Option<FontData> {
    None
}

/// Builds the font set from system fonts (with egui's defaults as fallback), with the default
/// (non-Japanese) Han order. Never reads the host locale, so tests and snapshots are stable.
pub fn font_definitions() -> FontDefinitions {
    font_definitions_for_language(crate::i18n::Language::En)
}

/// Which Han face leads the fallback chain: Japanese glyph shapes for a Japanese interface,
/// Simplified Chinese (the widest Han coverage) for every other language.
pub fn han_order(language: crate::i18n::Language) -> HanOrder {
    if language == crate::i18n::Language::Ja { HanOrder::Japanese } else { HanOrder::Chinese }
}

/// Builds the font set with the Han fallback faces ordered for the interface language.
pub fn font_definitions_for_language(language: crate::i18n::Language) -> FontDefinitions {
    font_definitions_with(language, &[])
}

/// [`font_definitions_for_language`] that also binds the workbook fonts in `used` that are
/// installed, so cells set in them render with them; the rest fall back to the roles. The bound
/// families are recorded in [`crate::fonts`] so [`cell_family`] only names a family egui has.
pub fn font_definitions_with(language: crate::i18n::Language, used: &[String]) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let base_prop: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let base_mono: Vec<String> = fonts.families.get(&FontFamily::Monospace).cloned().unwrap_or_default();
    let mut bound: Vec<String> = Vec::new();
    // The non-Latin fallback faces have to sit at the end of each chain: a Latin font owns Latin,
    // and epaint picks the first face that has the glyph, so they only catch their own scripts.
    let fallback = load_fallback(&mut fonts, han_order(language));
    for role in [UI, UI_BOLD, CELL, CELL_BOLD, CELL_ITALIC, CELL_BOLD_ITALIC, SERIF, MONO] {
        let paths = candidates(role);
        let refs: Vec<(&str, u32)> = paths.iter().map(|(p, i)| (p.as_str(), *i)).collect();
        let mut chain: Vec<String> = Vec::new();
        if let Some(fd) = try_load(&refs) {
            let key = format!("sys-{role}");
            fonts.font_data.insert(key.clone(), Arc::new(fd));
            chain.push(key);
        } else if role == UI_BOLD {
            chain.push(format!("sys-{UI}"));
        } else if role.starts_with("cell-") {
            chain.push(format!("sys-{CELL}"));
        }
        chain.extend(if role == MONO { base_mono.clone() } else { base_prop.clone() });
        chain.extend(fallback.iter().cloned());
        chain.retain(|k| fonts.font_data.contains_key(k));
        fonts.families.insert(FontFamily::Name(role.into()), chain);
        bound.push(role.to_string());
    }
    // Installed workbook fonts, so a cell's chosen font actually renders in it.
    for family in gridcraft_fonts::installed_among(used) {
        bound.extend(crate::fonts::bind_family(&mut fonts, &family));
    }
    // Interface text (formula bar, name box, dialogs) falls back to the cell font before the
    // script faces: whatever a cell can draw, the formula bar can draw too, even where no script
    // fallback face was found (#349: macOS UI font without Hebrew, cells in Arial with it).
    let cell_key = format!("sys-{CELL}");
    if fonts.font_data.contains_key(&cell_key) {
        for role in [UI, UI_BOLD, SERIF, MONO] {
            if let Some(chain) = fonts.families.get_mut(&FontFamily::Name(role.into()))
                && !chain.contains(&cell_key)
            {
                let at = chain.iter().position(|k| fallback.contains(k)).unwrap_or(chain.len());
                chain.insert(at, cell_key.clone());
            }
        }
    }
    // Default proportional text in widgets uses the UI font.
    if let Some(ui) = fonts.families.get(&FontFamily::Name(UI.into())).cloned() {
        fonts.families.insert(FontFamily::Proportional, ui);
    }
    drop_unparseable_fonts(&mut fonts);
    // Only families egui still has data for count as bound (a dropped face can empty a chain).
    bound.retain(|f| fonts.families.get(&FontFamily::Name(f.as_str().into())).is_some_and(|c| !c.is_empty()));
    crate::fonts::set_bound(bound);
    fonts
}

/// Loads the system non-Latin fallback faces (one file per script family) and returns their keys so
/// every family can carry them as fallback. egui's bundled fonts have no coverage beyond Latin, so
/// without this a Thai, Hebrew, Japanese or Chinese workbook shows tofu boxes even though the text
/// was read correctly. With a face installed those scripts get real glyphs instead of tofu; nothing
/// is bundled, and a machine without the face keeps the old behaviour.
#[cfg(not(target_arch = "wasm32"))]
fn load_fallback(fonts: &mut FontDefinitions, han: HanOrder) -> Vec<String> {
    let mut keys = Vec::new();
    // Files already loaded as a fallback face. A file serves at most one key: the region-merged
    // Noto Sans CJK .ttc found for Han also carries Hangul (and Tahoma carries both Thai and
    // Hebrew), so loading it again for a later script would only duplicate tens of MB in memory.
    let mut loaded: Vec<String> = Vec::new();
    for (key, cands) in gridcraft_fonts::fallback_faces(han) {
        for (path, index) in &cands {
            if loaded.contains(path) {
                break; // a face loaded for an earlier script already covers this one
            }
            if let Some(fd) = try_load(&[(path.as_str(), *index)]) {
                fonts.font_data.insert(key.to_string(), Arc::new(fd));
                keys.push(key.to_string());
                loaded.push(path.clone());
                break;
            }
        }
    }
    keys
}

#[cfg(target_arch = "wasm32")]
fn load_fallback(_fonts: &mut FontDefinitions, _han: HanOrder) -> Vec<String> {
    Vec::new()
}

pub fn apply(ctx: &egui::Context, dark: bool) {
    // Install both palettes before System can switch between the style slots.
    for theme in [egui::Theme::Light, egui::Theme::Dark] {
        let dark = theme == egui::Theme::Dark;
        let t = if dark { Tokens::dark() } else { Tokens::light() };
        let mut v = if dark { Visuals::dark() } else { Visuals::light() };
        v.panel_fill = t.window;
        v.window_fill = t.menu_bg;
        v.extreme_bg_color = t.input_bg;
        v.selection.bg_fill = t.accent_soft;
        v.selection.stroke = Stroke::new(1.0, t.accent);
        v.hyperlink_color = t.accent;
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t.text);
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, t.text);
        v.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        v.widgets.hovered.weak_bg_fill = t.hover;
        v.widgets.active.weak_bg_fill = t.pressed;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, t.input_border);
        v.window_corner_radius = egui::CornerRadius::same(10);
        v.menu_corner_radius = egui::CornerRadius::same(8);
        v.popup_shadow = egui::epaint::Shadow { offset: [0, 4], blur: 16, spread: 0, color: t.shadow };
        ctx.set_visuals_of(theme, v);
    }
    ctx.options_mut(|o| o.fallback_theme = egui::Theme::Light);
    ctx.set_theme(if dark { egui::Theme::Dark } else { egui::Theme::Light });
    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = egui::vec2(6.0, 4.0);
        s.spacing.button_padding = egui::vec2(6.0, 3.0);
        s.spacing.interact_size.y = 22.0;
        s.text_styles.insert(egui::TextStyle::Body, ui_font(13.0));
        s.text_styles.insert(egui::TextStyle::Button, ui_font(13.0));
        s.text_styles.insert(egui::TextStyle::Small, ui_font(11.0));
        s.text_styles.insert(egui::TextStyle::Heading, ui_bold(18.0));
        s.text_styles.insert(egui::TextStyle::Monospace, FontId::new(12.0, FontFamily::Name(MONO.into())));
    });
}

/// Model colour → egui colour.
pub fn color32(rgb: [u8; 3]) -> Color32 {
    Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}

#[cfg(test)]
mod font_binding_tests {
    use super::*;

    /// The families [`font_definitions`] always binds, whatever is installed.
    fn always_bound() -> Vec<String> {
        [UI, UI_BOLD, CELL, CELL_BOLD, CELL_ITALIC, CELL_BOLD_ITALIC, SERIF, MONO].iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_name_that_is_not_an_installed_font_gets_a_built_in_role() {
        let _g = crate::fonts::TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Nothing installed: every name buckets to a role that font_definitions bound.
        crate::fonts::set_bound(always_bound());
        for name in ["", "Calibri", "Courier New", "Times New Roman", "Grossly Missing Font", "\u{1F600}"] {
            for (b, i) in [(false, false), (true, false), (false, true), (true, true)] {
                let FontFamily::Name(fam) = cell_family(name, b, i) else { panic!("{name:?} should name a family") };
                assert!(crate::fonts::is_bound(&fam), "{name:?} {b} {i} → unbound {fam:?}");
            }
        }
    }

    #[test]
    fn a_bound_installed_font_is_used_by_name() {
        let _g = crate::fonts::TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let role = crate::fonts::role_key("Ubuntu", true, false);
        crate::fonts::set_bound(always_bound().into_iter().chain([role.clone()]));
        assert!(matches!(cell_family("Ubuntu", true, false), FontFamily::Name(f) if *f == *role));
        // The same family in a style that wasn't bound falls back to a built-in role that is.
        let FontFamily::Name(fallback) = cell_family("Ubuntu", false, true) else { panic!("should name a family") };
        assert!(crate::fonts::is_bound(&fallback), "the fallback is a bound role");
        crate::fonts::set_bound(always_bound());
    }

    #[test]
    fn arbitrary_names_never_yield_an_unbound_family() {
        let _g = crate::fonts::TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // A workbook can hold any font name; none may name a family egui has no data for.
        crate::fonts::set_bound(always_bound());
        for seed in 0..500u32 {
            let name: String =
                (0..(seed % 40)).map(|k| char::from_u32(0x20 + (seed.wrapping_mul(31).wrapping_add(k) % 0x5F)).unwrap_or('x')).collect();
            let FontFamily::Name(fam) = cell_family(&name, seed % 2 == 0, seed % 3 == 0) else { continue };
            assert!(crate::fonts::is_bound(&fam), "{name:?} → unbound {fam:?}");
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::i18n::Language;
    use gridcraft_fonts::fallback_faces;

    #[test]
    fn fallback_faces_are_appended_to_every_family() {
        // Structural: whatever loaded, the non-Latin fallback keys must sit at the end of each chain
        // (after the Latin/default fonts) so Latin still wins for Latin text, in the language's order.
        for han in [HanOrder::Chinese, HanOrder::Japanese] {
            let language = if han == HanOrder::Japanese { Language::Ja } else { Language::Zh };
            let fonts = font_definitions_for_language(language);
            let fb_keys: Vec<String> =
                fallback_faces(han).into_iter().map(|(k, _)| k.to_string()).filter(|k| fonts.font_data.contains_key(k)).collect();
            if fb_keys.is_empty() {
                return; // no fallback face on this machine (minimal Linux)
            }
            for role in [UI, UI_BOLD, CELL, CELL_BOLD, CELL_ITALIC, CELL_BOLD_ITALIC, SERIF, MONO] {
                let chain = fonts.families.get(&FontFamily::Name(role.into())).map(Vec::as_slice).unwrap_or_default();
                assert!(chain.len() >= fb_keys.len(), "{role} chain too short for its fallback");
                let tail = &chain[chain.len() - fb_keys.len()..];
                assert_eq!(tail, fb_keys.as_slice(), "{role} must end its chain with the fallback faces");
            }
        }
    }

    #[test]
    fn interface_families_fall_back_to_the_cell_font() {
        // Structural (#349): the formula bar, name box and dialogs draw every script the cells can,
        // so each interface chain carries the cell face, ahead of the script fallback faces.
        let fonts = font_definitions_for_language(Language::En);
        let cell = format!("sys-{CELL}");
        if !fonts.font_data.contains_key(&cell) {
            return; // no system cell font on this machine
        }
        let fb: Vec<String> = fallback_faces(HanOrder::Chinese).into_iter().map(|(k, _)| k.to_string()).collect();
        for family in [FontFamily::Proportional, FontFamily::Name(UI.into()), FontFamily::Name(UI_BOLD.into()), FontFamily::Name(MONO.into())] {
            let chain = fonts.families.get(&family).cloned().unwrap_or_default();
            let at = chain.iter().position(|k| *k == cell);
            assert!(at.is_some(), "{family:?} lacks the cell face: {chain:?}");
            let first_fb = chain.iter().position(|k| fb.contains(k)).unwrap_or(chain.len());
            assert!(at.unwrap_or(usize::MAX) < first_fb, "{family:?}: cell face must precede the script faces");
        }
    }

    #[test]
    fn only_a_japanese_interface_puts_japanese_han_first() {
        assert_eq!(han_order(Language::Ja), HanOrder::Japanese);
        for l in [Language::En, Language::Zh, Language::Ko, Language::Ru, Language::PtBr, Language::De, Language::Cs] {
            assert_eq!(han_order(l), HanOrder::Chinese, "{l:?} shares the default order (no font rebuild)");
        }
    }

    #[test]
    fn fallback_faces_load_each_file_once() {
        // The region-merged Noto Sans CJK .ttc serves Japanese, Chinese and Hangul; it must be loaded
        // once, not once per script (tens of MB each).
        for language in [Language::En, Language::Ja] {
            let fonts = font_definitions_for_language(language);
            let present: Vec<&Arc<FontData>> = fallback_faces(han_order(language)).into_iter().filter_map(|(k, _)| fonts.font_data.get(k)).collect();
            for (i, a) in present.iter().enumerate() {
                for b in present.iter().skip(i + 1) {
                    assert!(a.font != b.font, "the same font file is loaded for two fallback scripts");
                }
            }
        }
    }

    #[test]
    fn fallback_covers_its_script() {
        // Real render check, per script: the proportional family (what cells and the UI use) must
        // resolve a sample string to a real glyph, not the tofu replacement char. Skips a script
        // whose face is absent (a minimal machine) and fails if a face is present but not wired in.
        let cases: [(&str, Vec<(String, u32)>, &str); 7] = [
            ("sys-cjk-han-ja", gridcraft_fonts::cjk_han_japanese(), "カテゴリ"),
            ("sys-cjk-han-zh", gridcraft_fonts::cjk_han_simplified(), "页面布局 客户端配置"),
            ("sys-cjk-hangul", gridcraft_fonts::cjk_hangul(), "한글"),
            ("sys-thai", gridcraft_fonts::thai(), "ทดสอบ"),
            ("sys-hebrew", gridcraft_fonts::hebrew(), "עברית"),
            ("ui", candidates(UI), "Русский"),
            // Czech interface labels: every Czech accented letter, in both cases.
            ("ui", candidates(UI), "Příliš žluťoučký kůň úpěl ďábelské ódy ŘŠČŽÝÁÍÉÚŮĚŤĎŇÓ"),
        ];
        for language in [Language::En, Language::Ja] {
            let ctx = egui::Context::default();
            ctx.set_fonts(font_definitions_for_language(language));
            let mut out = ctx.run_ui(egui::RawInput::default(), |_ui| {});
            out.textures_delta.clear(); // nothing consumes the atlas in a headless test
            // `has_glyphs` is a false negative when the face that owns the script is also the face
            // that draws the replacement glyph (DejaVu Sans as the UI font owns Hebrew), so compare
            // the laid out glyphs with the tofu glyph itself instead.
            for family in [FontFamily::Proportional, FontFamily::Name(CELL.into())] {
                let font = FontId::new(14.0, family);
                let glyphs = |text: &str| {
                    let galley = ctx.fonts_mut(|f| f.layout_no_wrap(text.to_owned(), font.clone(), Color32::BLACK));
                    galley.rows.iter().flat_map(|r| r.glyphs.iter().filter(|g| g.chr != ' ').map(|g| g.uv_rect)).collect::<Vec<_>>()
                };
                let tofu = glyphs("\u{25FB}");
                for (key, cands, sample) in &cases {
                    if !cands.iter().any(|(p, _)| std::path::Path::new(p).exists()) {
                        continue; // no face for this script on this machine; nothing to assert
                    }
                    let drawn = glyphs(sample);
                    assert!(
                        !drawn.is_empty() && drawn.iter().all(|g| !tofu.contains(g)),
                        "{key} is present but {sample:?} still renders as tofu ({language:?}, {:?})",
                        font.family
                    );
                }
            }
        }
    }

    #[test]
    fn unparseable_font_files_are_rejected_not_fatal() {
        // #226: epaint panics in `set_fonts` on a face it can't parse ("An offset was out of
        // bounds"). Garbage, a truncated real font and a bad collection index must all be refused.
        let real = FontDefinitions::default().font_data.values().next().map(|fd| fd.font.to_vec()).unwrap_or_default();
        assert!(font_parses(&real, 0), "egui's own font must parse");
        assert!(!font_parses(b"", 0));
        assert!(!font_parses(&[0xAB; 64], 0));
        assert!(!font_parses(real.get(..20).unwrap_or_default(), 0), "a truncated font must be refused");
        assert!(!font_parses(&real, 3), "a single-face file has no face 3");

        // A bad face that slipped into the definitions is dropped (key and family entries), and the
        // context builds its fonts without panicking.
        let mut fonts = font_definitions();
        fonts.font_data.insert("sys-hebrew".into(), Arc::new(FontData::from_owned(vec![0, 1, 0, 0, 0, 0xFF, 0, 0])));
        for chain in fonts.families.values_mut() {
            chain.push("sys-hebrew".into());
        }
        drop_unparseable_fonts(&mut fonts);
        assert!(!fonts.font_data.contains_key("sys-hebrew"));
        assert!(fonts.families.values().all(|c| !c.iter().any(|k| k == "sys-hebrew")));
        let ctx = egui::Context::default();
        ctx.set_fonts(fonts);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.label("Hebrew עברית");
        });
        out.textures_delta.clear(); // nothing consumes the atlas in a headless test
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn try_load_skips_a_broken_candidate() {
        // A candidate that exists but is not a readable font must not win over a good one behind it.
        let dir = std::env::temp_dir().join(format!("gridcraft-font-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let bad = dir.join("broken.ttf");
        let good = dir.join("good.ttf");
        let real = FontDefinitions::default().font_data.values().next().map(|fd| fd.font.to_vec()).unwrap_or_default();
        let _ = std::fs::write(&bad, real.get(..40).unwrap_or_default());
        let _ = std::fs::write(&good, &real);
        let (bad_s, good_s) = (bad.to_string_lossy().into_owned(), good.to_string_lossy().into_owned());
        assert!(try_load(&[(bad_s.as_str(), 0)]).is_none());
        let loaded = try_load(&[(bad_s.as_str(), 0), (good_s.as_str(), 0)]);
        assert_eq!(loaded.map(|fd| fd.font.len()), Some(real.len()));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
