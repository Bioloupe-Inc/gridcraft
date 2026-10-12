use super::*;

/// The test font (Ubuntu Light, compiled into epaint), registered once.
fn ubuntu() -> Font {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        gridcraft_fonts::register_font(epaint_default_fonts::UBUNTU_LIGHT.to_vec());
    });
    Font::new("Ubuntu", false, false)
}

fn pages_text(pdf: &[u8]) -> Vec<String> {
    test_support::page_texts(pdf).unwrap()
}

fn find(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

#[test]
fn a_document_without_pages_gets_one_blank_page() {
    let pdf = PdfDoc::new().finish().unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    assert_eq!(pages_text(&pdf).len(), 1);
}

#[test]
fn drawing_and_text_round_trip_with_the_font_embedded() {
    let font = ubuntu();
    let mut d = PdfDoc::new();
    d.info.title = "Report (Q1)".into();
    let p = d.add_page(612.0, 792.0);
    p.fill_rect(10.0, 10.0, 100.0, 20.0, [255, 255, 0]);
    p.rect_stroke(10.0, 10.0, 100.0, 20.0, 0.5, [0, 0, 0]);
    p.stroke_line(0.0, 0.0, 612.0, 792.0, 1.0, [255, 0, 0]);
    p.stroke_line_dashed(0.0, 5.0, 100.0, 5.0, 1.0, [0, 0, 0], &[3.0, 2.0]);
    p.polygon(&[[0.0, 0.0], [10.0, 0.0], [5.0, 8.0]], Some([0, 0, 255]), Some(([0, 0, 0], 1.0)));
    p.circle(300.0, 300.0, 20.0, Some([0, 128, 0]), None);
    p.save();
    p.clip_rect(0.0, 0.0, 400.0, 400.0);
    p.text(72.0, 72.0, 12.0, &font, [0, 0, 0], "Hello (world) \\ café €5 αβγ");
    p.restore();
    p.text_rotated(100.0, 300.0, 9.0, &font, [0, 0, 0], "Up", -std::f32::consts::FRAC_PI_2);
    let p2 = d.add_page(842.0, 595.0);
    p2.text(10.0, 20.0, 10.0, &font, [0, 0, 0], "Second");
    assert_eq!(d.page_count(), 2);
    let pdf = d.finish().unwrap();
    let pages = pages_text(&pdf);
    assert_eq!(pages.len(), 2);
    assert_eq!(pages, ["Hello (world) \\ café €5 αβγUp", "Second"]);
    assert!(find(&pdf, b"/FontFile2"), "the face travels with the document");
}

#[test]
fn text_is_measured_with_the_face_it_draws_with() {
    let font = ubuntu();
    assert_eq!(text_width(&font, 10.0, ""), 0.0);
    let w = text_width(&font, 10.0, "Hello");
    assert!(w > 10.0 && w < 50.0, "{w}");
    assert!((text_width(&font, 20.0, "Hello") - 2.0 * w).abs() < 1e-3);
    assert!(text_width(&font, 10.0, "WWW") > text_width(&font, 10.0, "iii"), "proportional advances");
}

#[test]
fn images_with_alpha() {
    let mut d = PdfDoc::new();
    let p = d.add_page(200.0, 200.0);
    let mut px = Vec::new();
    for i in 0..16u8 {
        px.extend_from_slice(&[i * 10, 0, 0, if i < 8 { 255 } else { 128 }]);
    }
    p.image_rgba(10.0, 10.0, 40.0, 40.0, &px, 4, 4);
    p.image_rgba(10.0, 10.0, 40.0, 40.0, &[255; 16], 2, 2);
    p.image_rgba(0.0, 0.0, 1.0, 1.0, &[1, 2, 3], 4, 4); // too short: ignored
    let pdf = d.finish().unwrap();
    assert_eq!(test_support::pages(&pdf).unwrap()[0].images, 2);
    assert!(find(&pdf, b"/SMask"), "alpha travels as a soft mask");
}

#[test]
fn hostile_inputs_never_panic() {
    let font = ubuntu();
    let mut d = PdfDoc::new();
    let p = d.add_page(f32::NAN, -5.0);
    p.text(f32::INFINITY, f32::NAN, 1e30, &font, [0, 0, 0], "x");
    p.text(1.0, 1.0, 10.0, &Font::new("", true, true), [0, 0, 0], "\u{0}\u{FFFF}\u{10FFFF}\u{301}");
    p.fill_rect(0.0, 0.0, -1.0, 5.0, [0, 0, 0]);
    p.polygon(&[], None, None);
    p.circle(0.0, 0.0, -3.0, Some([0, 0, 0]), None);
    p.clip_rect(f32::NAN, 0.0, 1.0, 1.0);
    p.stroke_line_dashed(0.0, 0.0, 1.0, 1.0, f32::NAN, [0, 0, 0], &[f32::NAN, -1.0]);
    p.restore();
    p.restore();
    p.save();
    p.image_rgba(0.0, 0.0, 10.0, 10.0, &[], u32::MAX, u32::MAX);
    assert!(d.finish().is_ok());
}
