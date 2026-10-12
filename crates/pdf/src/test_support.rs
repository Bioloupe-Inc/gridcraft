//! Reading exported PDFs back, for tests here and in the crates that export (feature
//! `test-support`).

use hayro_interpret::font::Glyph;
use hayro_interpret::hayro_cmap::BfString;
use hayro_interpret::hayro_syntax::Pdf;
use hayro_interpret::{
    BlendMode, ClipPath, Context, Device, GlyphDrawMode, Image, InterpreterCache, InterpreterSettings, Paint, PathDrawMode, SoftMask, interpret_page,
};
use kurbo::{Affine, BezPath, Rect};

/// One glyph drawn on a page.
#[derive(Clone, Debug)]
pub struct DrawnGlyph {
    /// The text it stands for.
    pub text: String,
    /// Direction of its baseline in degrees, counter-clockwise on the page: 0 reads left to right,
    /// 90 bottom to top, -90 top to bottom.
    pub angle: f64,
    /// Its origin in PDF user space (y grows up).
    pub origin: (f64, f64),
}

/// What one page draws.
#[derive(Clone, Debug, Default)]
pub struct PageContent {
    /// Every glyph, in drawing order.
    pub glyphs: Vec<DrawnGlyph>,
    /// How many images.
    pub images: usize,
}

impl Device<'_> for PageContent {
    fn set_soft_mask(&mut self, _: Option<SoftMask<'_>>) {}
    fn set_blend_mode(&mut self, _: BlendMode) {}
    fn draw_path(&mut self, _: &BezPath, _: Affine, _: &Paint<'_>, _: &PathDrawMode) {}
    fn push_clip_path(&mut self, _: &ClipPath) {}
    fn push_transparency_group(&mut self, _: f32, _: Option<SoftMask<'_>>, _: BlendMode) {}
    fn draw_glyph(&mut self, g: &Glyph<'_>, transform: Affine, _: Affine, _: &Paint<'_>, mode: &GlyphDrawMode) {
        if matches!(mode, GlyphDrawMode::Stroke(_)) {
            return;
        }
        let text = match g.as_unicode() {
            Some(BfString::Char(c)) => c.to_string(),
            Some(BfString::String(s)) => s,
            None => '\u{FFFD}'.to_string(),
        };
        let [a, b, _, _, e, f] = transform.as_coeffs();
        self.glyphs.push(DrawnGlyph { text, angle: b.atan2(a).to_degrees(), origin: (e, f) });
    }
    fn draw_image(&mut self, _: Image<'_, '_>, _: Affine) {
        self.images += 1;
    }
    fn pop_clip_path(&mut self) {}
    fn pop_transparency_group(&mut self) {}
}

/// The text of each page, as a PDF reader extracts it through each font's ToUnicode map: glyph
/// by glyph in drawing order, nothing inferred. `None` when the PDF doesn't parse.
pub fn page_texts(pdf: &[u8]) -> Option<Vec<String>> {
    Some(pages(pdf)?.into_iter().map(|p| p.glyphs.into_iter().map(|g| g.text).collect()).collect())
}

/// What each page draws. `None` when the PDF doesn't parse.
pub fn pages(pdf: &[u8]) -> Option<Vec<PageContent>> {
    let pdf = Pdf::new(pdf.to_vec()).ok()?;
    let cache = InterpreterCache::new();
    let pages = pdf
        .pages()
        .iter()
        .map(|page| {
            let mut ctx = Context::new(Affine::IDENTITY, Rect::new(0.0, 0.0, 1.0, 1.0), &cache, pdf.xref(), InterpreterSettings::default());
            let mut content = PageContent::default();
            interpret_page(page, &mut ctx, &mut content);
            content
        })
        .collect();
    Some(pages)
}
