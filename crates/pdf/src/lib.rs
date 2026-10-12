//! GridCraft's PDF writer for print and PDF export, built on [krilla].
//!
//! - Text is set in embedded, subset TrueType/OpenType fonts (CID-keyed, with a ToUnicode map),
//!   so any Unicode text prints and extracts exactly as written. Faces come from
//!   [`gridcraft_fonts::resolve`]: the workbook's font, else its metric-compatible twin or role
//!   stand-in, with script fallbacks (CJK, Thai, Hebrew, symbols) for characters it lacks.
//!   [`text_width`] measures with the same faces and shaping the page draws with.
//! - Vector drawing: rectangles, lines (solid or dashed), polygons, circles (Bézier), clipping.
//! - RGBA images.
//! - Coordinates are points with a **top-left origin** (y grows down); text `y` is the baseline.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

use gridcraft_fonts::FontData;
use krilla::geom::{Path, PathBuilder, Point, Rect, Size, Transform};
use krilla::image::Image;
use krilla::metadata::Metadata;
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule, LineJoin, Stroke, StrokeDash};
use krilla::surface::Surface;
use krilla::text::{GlyphId, KrillaGlyph};
use rustybuzz::ttf_parser::Tag;
use rustybuzz::{Direction, Feature, Language, Script, ShapePlan};

/// RGB colour, 0–255 per channel.
pub type Rgb = [u8; 3];

/// A font as a workbook names it: family, weight and slant.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Font {
    family: Arc<str>,
    bold: bool,
    italic: bool,
}

impl Font {
    pub fn new(family: &str, bold: bool, italic: bool) -> Font {
        Font { family: family.into(), bold, italic }
    }
}

/// Text without any face to draw it with (no font on the machine or registered) is still laid
/// out, at this many ems per character, and left out of the page.
const NO_FACE_ADVANCE: f32 = 0.5;

/// A run of `text` set in one face.
struct Run {
    face: FontData,
    range: Range<usize>,
}

/// Splits `text` into runs by face: each character goes to the first face in the font's chain that
/// has a glyph for it (spaces and combining marks stay with the run they're in), so a Latin cell
/// with a few superscripts and a Han word uses the cell font, a symbol face and a CJK face, and
/// goes back to the cell font after them. A character no face covers stays in the current run.
/// Empty when no face is available at all.
fn runs(font: &Font, text: &str) -> Vec<Run> {
    let chain = gridcraft_fonts::resolve(&font.family, font.bold, font.italic);
    let covering = |c: char| (0..chain.len()).find_map(|i| chain.face(i).filter(|f| covers(f, c)).map(|f| (i, f)));
    let Some(first) = (0..chain.len()).find_map(|i| chain.face(i).map(|f| (i, f))) else { return Vec::new() };
    let mut out: Vec<(usize, &FontData, Range<usize>)> = Vec::new();
    for (at, c) in text.char_indices() {
        let end = at + c.len_utf8();
        let keep = c.is_whitespace() || c.is_control() || is_mark(c);
        let pick = match out.last() {
            Some((i, f, _)) if keep => Some((*i, *f)),
            _ => covering(c),
        };
        let (i, f) = pick.or_else(|| out.last().map(|(i, f, _)| (*i, *f))).unwrap_or(first);
        match out.last_mut() {
            Some((j, _, r)) if *j == i => r.end = end,
            _ => out.push((i, f, at..end)),
        }
    }
    out.into_iter().map(|(_, face, range)| Run { face: face.clone(), range }).collect()
}

/// Combining marks (accents written as separate code points) draw with their base character.
fn is_mark(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F | 0xFE00..=0xFE0F | 0x200C | 0x200D)
}

/// A parsed face and the shaping plans compiled for it so far (compiling a plan costs more than
/// shaping a cell's text with it).
struct Shaper {
    face: rustybuzz::Face<'static>,
    plans: HashMap<(Direction, Script, Option<Language>), ShapePlan>,
}

thread_local! {
    /// One shaper per font file and face index. Their bytes stay alive for the rest of the run (the
    /// font database never drops a face it read), so each face is parsed once.
    static SHAPERS: RefCell<HashMap<(usize, u32), Option<Shaper>>> = RefCell::new(HashMap::new());
}

/// Runs `f` on the face's shaper; `None` when the face doesn't parse.
fn with_shaper<R>(face: &FontData, f: impl FnOnce(&mut Shaper) -> R) -> Option<R> {
    SHAPERS.with(|cache| {
        let mut cache = cache.borrow_mut();
        let shaper = cache.entry((face.data.as_ptr() as usize, face.index)).or_insert_with(|| {
            let bytes: &'static Arc<[u8]> = Box::leak(Box::new(face.data.clone()));
            rustybuzz::Face::from_slice(bytes, face.index).map(|face| Shaper { face, plans: HashMap::new() })
        });
        shaper.as_mut().map(f)
    })
}

/// Whether the face has a glyph for `c`.
fn covers(face: &FontData, c: char) -> bool {
    with_shaper(face, |s| s.face.glyph_index(c).is_some()).unwrap_or(false)
}

/// `text` shaped in one face: its glyphs (advances in ems, each mapped to the text it stands for)
/// and its total advance in ems. Script and direction are guessed from the text. Kerning and
/// standard ligatures are off, as a spreadsheet sets cell text: a string is as wide as its glyphs'
/// advances, so text measures (and wraps) the same in every face with the same widths.
struct Shaped {
    glyphs: Vec<KrillaGlyph>,
    advance: f32,
}

fn shape(face: &FontData, text: &str) -> Option<Shaped> {
    with_shaper(face, |s| {
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(text);
        buffer.guess_segment_properties();
        let dir = buffer.direction();
        if dir == Direction::Invalid {
            return None;
        }
        let key = (dir, buffer.script(), buffer.language());
        let plan = s.plans.entry(key.clone()).or_insert_with(|| {
            let off = |tag: &[u8; 4]| Feature::new(Tag::from_bytes(tag), 0, ..);
            ShapePlan::new(&s.face, key.0, Some(key.1), key.2.as_ref(), &[off(b"kern"), off(b"liga"), off(b"clig")])
        });
        let out = rustybuzz::shape_with_plan(&s.face, plan, buffer);
        let upem = s.face.units_per_em().max(1) as f32;
        let infos = out.glyph_infos();
        let forward = matches!(dir, Direction::LeftToRight | Direction::TopToBottom);
        let mut glyphs = Vec::with_capacity(infos.len());
        let mut advance = 0.0;
        for (i, (info, pos)) in infos.iter().zip(out.glyph_positions()).enumerate() {
            // The text a glyph stands for runs to the next cluster in logical order.
            let (before, after) = infos.split_at(i);
            let next =
                if forward { after.iter().find(|g| g.cluster != info.cluster) } else { before.iter().rev().find(|g| g.cluster != info.cluster) };
            let start = (info.cluster as usize).min(text.len());
            let end = next.map_or(text.len(), |g| g.cluster as usize).clamp(start, text.len());
            let x_advance = pos.x_advance as f32 / upem;
            advance += x_advance;
            glyphs.push(KrillaGlyph::new(
                GlyphId::new(info.glyph_id),
                x_advance,
                pos.x_offset as f32 / upem,
                pos.y_offset as f32 / upem,
                pos.y_advance as f32 / upem,
                start..end,
                None,
            ));
        }
        Some(Shaped { glyphs, advance })
    })
    .flatten()
}

/// Advance width of `text` set in `face`, in ems.
fn run_advance(face: &FontData, text: &str) -> f32 {
    shape(face, text).map_or(text.chars().count() as f32 * NO_FACE_ADVANCE, |s| s.advance)
}

/// Width of `text` in points when set in `font` at `size` points.
pub fn text_width(font: &Font, size: f32, text: &str) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let runs = runs(font, text);
    let ems: f32 = if runs.is_empty() {
        text.chars().count() as f32 * NO_FACE_ADVANCE
    } else {
        runs.iter().map(|r| run_advance(&r.face, text.get(r.range.clone()).unwrap_or_default())).sum()
    };
    ems * size
}

/// Most drawing operations per page before further drawing is dropped (keeps hostile inputs from
/// exhausting memory).
const MAX_PAGE_OPS: usize = 2_000_000;
/// Largest image (pixels) embedded.
const MAX_IMAGE_PIXELS: u64 = 40_000_000;

enum Op {
    Save,
    Restore,
    Clip(Path),
    Draw { path: Path, fill: Option<Rgb>, stroke: Option<Line> },
    Text { x: f32, y: f32, size: f32, font: Font, color: Rgb, text: String, angle: f32 },
    Image { x: f32, y: f32, size: Size, image: Image },
}

/// How a path is stroked.
struct Line {
    color: Rgb,
    width: f32,
    dash: Vec<f32>,
    /// Round joins (polylines and polygons) instead of mitred ones.
    round: bool,
}

/// A page under construction. Obtain one with [`PdfDoc::add_page`].
pub struct Page {
    width: f32,
    height: f32,
    ops: Vec<Op>,
    depth: u32,
}

fn rect_path(x: f32, y: f32, w: f32, h: f32) -> Option<Path> {
    let mut pb = PathBuilder::new();
    pb.push_rect(Rect::from_xywh(x, y, w, h)?);
    pb.finish()
}

fn poly_path(pts: &[[f32; 2]], close: bool) -> Option<Path> {
    let mut pb = PathBuilder::new();
    let (first, rest) = pts.split_first()?;
    pb.move_to(first[0], first[1]);
    for p in rest {
        pb.line_to(p[0], p[1]);
    }
    if close {
        pb.close();
    }
    pb.finish()
}

fn finite(v: f32) -> bool {
    v.is_finite() && v.abs() < 1.0e7
}

impl Page {
    pub fn width(&self) -> f32 {
        self.width
    }
    pub fn height(&self) -> f32 {
        self.height
    }
    fn op(&mut self, op: Op) {
        if self.ops.len() < MAX_PAGE_OPS {
            self.ops.push(op);
        }
    }

    /// Saves the graphics state (pair with [`Page::restore`]).
    pub fn save(&mut self) {
        self.depth = self.depth.saturating_add(1);
        self.ops.push(Op::Save);
    }
    /// Restores the graphics state saved by [`Page::save`]; extra calls are ignored.
    pub fn restore(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
            self.ops.push(Op::Restore);
        }
    }
    /// Intersects the clip with a rectangle (until the next [`Page::restore`]).
    pub fn clip_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        // An empty or invalid clip hides everything drawn under it.
        let path = rect_path(x, y, w.max(0.0), h.max(0.0)).or_else(|| rect_path(0.0, 0.0, 0.0001, 0.0001));
        if let Some(p) = path {
            self.op(Op::Clip(p));
        }
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Rgb) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        if let Some(p) = rect_path(x, y, w, h) {
            self.op(Op::Draw { path: p, fill: Some(color), stroke: None });
        }
    }

    pub fn rect_stroke(&mut self, x: f32, y: f32, w: f32, h: f32, width: f32, color: Rgb) {
        if let Some(path) = rect_path(x, y, w, h) {
            self.op(Op::Draw { path, fill: None, stroke: Some(Line { color, width, dash: Vec::new(), round: false }) });
        }
    }

    pub fn stroke_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, width: f32, color: Rgb) {
        self.stroke_line_dashed(x0, y0, x1, y1, width, color, &[]);
    }

    /// A line with a dash pattern (`[on, off, …]` in points; empty = solid).
    pub fn stroke_line_dashed(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, width: f32, color: Rgb, dash: &[f32]) {
        if let Some(path) = poly_path(&[[x0, y0], [x1, y1]], false) {
            self.op(Op::Draw { path, fill: None, stroke: Some(Line { color, width, dash: dash.to_vec(), round: false }) });
        }
    }

    /// An open polyline.
    pub fn polyline(&mut self, pts: &[[f32; 2]], width: f32, color: Rgb, dash: &[f32]) {
        if pts.len() < 2 {
            return;
        }
        if let Some(path) = poly_path(pts, false) {
            self.op(Op::Draw { path, fill: None, stroke: Some(Line { color, width, dash: dash.to_vec(), round: true }) });
        }
    }

    /// A closed polygon, filled and/or stroked.
    pub fn polygon(&mut self, pts: &[[f32; 2]], fill: Option<Rgb>, stroke: Option<(Rgb, f32)>) {
        if pts.len() < 2 {
            return;
        }
        let Some(path) = poly_path(pts, true) else { return };
        self.fill_stroke(path, fill, stroke, true);
    }

    fn fill_stroke(&mut self, path: Path, fill: Option<Rgb>, stroke: Option<(Rgb, f32)>, round: bool) {
        if fill.is_some() || stroke.is_some() {
            self.op(Op::Draw { path, fill, stroke: stroke.map(|(color, width)| Line { color, width, dash: Vec::new(), round }) });
        }
    }

    /// A circle approximated by four cubic Béziers.
    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, fill: Option<Rgb>, stroke: Option<(Rgb, f32)>) {
        if r <= 0.0 || (fill.is_none() && stroke.is_none()) {
            return;
        }
        const K: f32 = 0.552_284_8;
        let k = r * K;
        let mut pb = PathBuilder::new();
        pb.move_to(cx + r, cy);
        pb.cubic_to(cx + r, cy + k, cx + k, cy + r, cx, cy + r);
        pb.cubic_to(cx - k, cy + r, cx - r, cy + k, cx - r, cy);
        pb.cubic_to(cx - r, cy - k, cx - k, cy - r, cx, cy - r);
        pb.cubic_to(cx + k, cy - r, cx + r, cy - k, cx + r, cy);
        pb.close();
        if let Some(path) = pb.finish() {
            self.fill_stroke(path, fill, stroke, false);
        }
    }

    /// Text with its baseline starting at `(x, y)`.
    pub fn text(&mut self, x: f32, y: f32, size: f32, font: &Font, color: Rgb, text: &str) {
        self.text_rotated(x, y, size, font, color, text, 0.0);
    }

    /// Text rotated by `angle` radians (clockwise on the page, like the y-down coordinate
    /// system) about its baseline origin.
    pub fn text_rotated(&mut self, x: f32, y: f32, size: f32, font: &Font, color: Rgb, text: &str, angle: f32) {
        let drawable = !text.is_empty() && size.is_finite() && size > 0.0 && finite(x) && finite(y);
        if !drawable {
            return;
        }
        let angle = if angle.is_finite() { angle } else { 0.0 };
        self.op(Op::Text { x, y, size: size.min(10_000.0), font: font.clone(), color, text: text.to_string(), angle });
    }

    /// Draws an RGBA image (`pw`×`ph` pixels, row-major, straight alpha) into the box
    /// `(x, y, w, h)`. Malformed or oversized buffers are ignored.
    pub fn image_rgba(&mut self, x: f32, y: f32, w: f32, h: f32, rgba: &[u8], pw: u32, ph: u32) {
        let px = pw as u64 * ph as u64;
        if pw == 0 || ph == 0 || px > MAX_IMAGE_PIXELS || (rgba.len() as u64) < px * 4 || !finite(x) || !finite(y) {
            return;
        }
        let Some(size) = Size::from_wh(w, h) else { return };
        let data = rgba.get(..(px * 4) as usize).unwrap_or_default().to_vec();
        self.op(Op::Image { x, y, size, image: Image::from_rgba8(data, pw, ph) });
    }
}

/// Document metadata.
#[derive(Clone, Debug, Default)]
pub struct Info {
    pub title: String,
    pub author: String,
    pub creator: String,
}

/// Why a document could not be written.
#[derive(Debug)]
pub struct PdfError(String);

impl std::fmt::Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PDF export failed: {}", self.0)
    }
}

impl std::error::Error for PdfError {}

/// A PDF document under construction.
#[derive(Default)]
pub struct PdfDoc {
    pages: Vec<Page>,
    pub info: Info,
}

fn fill_of(c: Rgb) -> Fill {
    Fill { paint: krilla::color::rgb::Color::new(c[0], c[1], c[2]).into(), ..Default::default() }
}

fn stroke_of(l: &Line) -> Stroke {
    let dash: Vec<f32> = l.dash.iter().take(8).map(|v| if v.is_finite() { v.max(0.0) } else { 0.0 }).collect();
    Stroke {
        paint: krilla::color::rgb::Color::new(l.color[0], l.color[1], l.color[2]).into(),
        width: if l.width.is_finite() { l.width.clamp(0.0, 1.0e4) } else { 1.0 },
        line_join: if l.round { LineJoin::Round } else { LineJoin::Miter },
        dash: dash.iter().any(|v| *v > 0.0).then_some(StrokeDash { array: dash, offset: 0.0 }),
        ..Default::default()
    }
}

/// Font bytes shared with krilla without a copy.
struct Bytes(Arc<[u8]>);

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// The krilla fonts of one document, one per face.
#[derive(Default)]
struct Fonts(HashMap<(usize, u32), Option<krilla::text::Font>>);

impl Fonts {
    fn get(&mut self, face: &FontData) -> Option<krilla::text::Font> {
        let key = (face.data.as_ptr() as usize, face.index);
        self.0
            .entry(key)
            .or_insert_with(|| {
                let data: Arc<dyn AsRef<[u8]> + Send + Sync> = Arc::new(Bytes(face.data.clone()));
                krilla::text::Font::new(data.into(), face.index)
            })
            .clone()
    }
}

fn draw_text(s: &mut Surface<'_>, fonts: &mut Fonts, x: f32, y: f32, size: f32, font: &Font, text: &str) {
    let mut at = x;
    for run in runs(font, text) {
        let part = text.get(run.range.clone()).unwrap_or_default();
        let Some(shaped) = shape(&run.face, part) else { continue };
        if let Some(f) = fonts.get(&run.face) {
            s.draw_glyphs(Point::from_xy(at, y), &shaped.glyphs, f, part, size, false);
        }
        at += shaped.advance * size;
    }
}

fn draw_page(s: &mut Surface<'_>, fonts: &mut Fonts, ops: &[Op]) {
    // Pushes (clips) since each open save, so a restore pops exactly what its save opened.
    let mut saves: Vec<usize> = vec![0];
    for op in ops {
        match op {
            Op::Save => saves.push(0),
            Op::Restore => {
                for _ in 0..saves.pop().unwrap_or(0) {
                    s.pop();
                }
                if saves.is_empty() {
                    saves.push(0);
                }
            }
            Op::Clip(path) => {
                s.push_clip_path(path, &FillRule::NonZero);
                if let Some(n) = saves.last_mut() {
                    *n += 1;
                }
            }
            Op::Draw { path, fill, stroke } => {
                s.set_fill(fill.map(fill_of));
                s.set_stroke(stroke.as_ref().map(stroke_of));
                s.draw_path(path);
            }
            Op::Text { x, y, size, font, color, text, angle } => {
                s.set_stroke(None);
                s.set_fill(Some(fill_of(*color)));
                if *angle != 0.0 {
                    s.push_transform(&Transform::from_rotate_at(angle.to_degrees(), *x, *y));
                    draw_text(s, fonts, *x, *y, *size, font, text);
                    s.pop();
                } else {
                    draw_text(s, fonts, *x, *y, *size, font, text);
                }
            }
            Op::Image { x, y, size, image } => {
                s.push_transform(&Transform::from_translate(*x, *y));
                s.draw_image(image.clone(), *size);
                s.pop();
            }
        }
    }
    for n in saves {
        for _ in 0..n {
            s.pop();
        }
    }
}

impl PdfDoc {
    pub fn new() -> PdfDoc {
        PdfDoc::default()
    }

    /// Adds a page of `width_pt` × `height_pt` points and returns it for drawing.
    pub fn add_page(&mut self, width_pt: f32, height_pt: f32) -> &mut Page {
        let width = if width_pt.is_finite() { width_pt.clamp(3.0, 14_400.0) } else { 612.0 };
        let height = if height_pt.is_finite() { height_pt.clamp(3.0, 14_400.0) } else { 792.0 };
        self.pages.push(Page { width, height, ops: Vec::new(), depth: 0 });
        let i = self.pages.len() - 1;
        &mut self.pages[i]
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// The page at `i`, to keep drawing on it.
    pub fn page_mut(&mut self, i: usize) -> Option<&mut Page> {
        self.pages.get_mut(i)
    }

    /// Serializes the document. A document without pages gets one blank Letter page.
    pub fn finish(mut self) -> Result<Vec<u8>, PdfError> {
        if self.pages.is_empty() {
            self.add_page(612.0, 792.0);
        }
        let mut pdf = krilla::Document::new();
        let creator = if self.info.creator.is_empty() { "GridCraft" } else { &self.info.creator };
        let mut meta = Metadata::new().producer("GridCraft PDF".into()).creator(creator.into());
        if !self.info.title.is_empty() {
            meta = meta.title(self.info.title.clone());
        }
        if !self.info.author.is_empty() {
            meta = meta.authors(vec![self.info.author.clone()]);
        }
        pdf.set_metadata(meta);
        let mut fonts = Fonts::default();
        for page in &self.pages {
            let size = Size::from_wh(page.width, page.height).ok_or_else(|| PdfError("page size".into()))?;
            let mut p = pdf.start_page_with(PageSettings::new(size));
            let mut s = p.surface();
            draw_page(&mut s, &mut fonts, &page.ops);
            s.finish();
            p.finish();
        }
        pdf.finish().map_err(|e| PdfError(format!("{e:?}")))
    }
}

#[cfg(feature = "test-support")]
pub mod test_support;

#[cfg(test)]
mod tests;
