use super::*;

const UBUNTU: &[u8] = epaint_default_fonts::UBUNTU_LIGHT;

#[test]
fn style_names_map_to_weight_and_slant() {
    assert_eq!(style_weight("Bold"), 700.0);
    assert_eq!(style_weight("SemiBold Italic"), 600.0);
    assert_eq!(style_weight("Regular"), 400.0);
    assert_eq!(style_weight("Light Oblique"), 300.0);
    assert!(style_italic("Bold Italic") && style_italic("Oblique"));
    assert!(!style_italic("Bold"));
    // The weight table is checked in order, so "extralight" wins over "light".
    assert_eq!(style_weight("ExtraLight"), 200.0);
}

#[test]
fn damaged_or_non_font_files_contribute_nothing() {
    assert!(file_faces(b"ttcf\x00\x01\x00\x00\xff\xff\xff\xff").is_empty());
    assert!(file_faces(b"not a font at all").is_empty());
    assert!(file_faces(&[]).is_empty());
    assert_eq!(register_font(b"not a font at all".to_vec()), 0);
}

#[test]
fn unparseable_font_files_are_rejected() {
    assert!(font_parses(UBUNTU, 0));
    assert!(!font_parses(b"", 0));
    assert!(!font_parses(&[0xAB; 64], 0));
    assert!(!font_parses(UBUNTU.get(..20).unwrap_or_default(), 0), "a truncated font must be refused");
    assert!(!font_parses(UBUNTU, 3), "a single-face file has no face 3");
}

#[test]
fn names_fall_into_roles_and_office_fonts_have_open_twins() {
    assert_eq!(cell_role("Courier New", true, false), MONO);
    assert_eq!(cell_role("Cambria", false, false), SERIF);
    assert_eq!(cell_role("Noto Sans", false, false), CELL);
    assert_eq!(cell_role("Calibri", true, true), CELL_BOLD_ITALIC);
    assert_eq!(metric_twin(" calibri"), Some("Carlito"));
    assert_eq!(metric_twin("Arial"), Some("Liberation Sans"));
    assert_eq!(metric_twin("Aptos"), None);
}

#[test]
fn a_registered_font_is_used_by_name_and_as_a_fallback() {
    assert_eq!(register_font(UBUNTU.to_vec()), 1);
    let first = |family: &str| resolve(family, false, false).face(0).map(|f| f.data.len());
    assert_eq!(first("Ubuntu"), Some(UBUNTU.len()), "the registered family is drawn with first");
    // Any other name still reaches it, as a fallback for what the machine's fonts lack.
    let chain = resolve("No Such Font", true, false);
    assert!((0..chain.len()).filter_map(|i| chain.face(i)).any(|f| f.data.len() == UBUNTU.len()));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn the_scan_catalogs_fonts_and_reads_each_folder_once() {
    let dir = std::env::temp_dir().join(format!("gridcraft-fonts-scan-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("Sub")).expect("temp dir");
    std::fs::write(dir.join("Ubuntu-Light.ttf"), UBUNTU).expect("write");
    std::fs::write(dir.join("damaged.ttf"), b"ttcf\x00\x01\x00\x00\xff\xff\xff\xff").expect("write");
    std::fs::write(dir.join("readme.txt"), b"not a font").expect("write");
    // The same folder listed twice (once through `Sub/..`) is read once.
    let faces = scan_dirs(&[dir.clone(), dir.join("Sub/..")]);
    let families: Vec<&str> = faces.iter().map(|f| f.family.as_str()).collect();
    assert_eq!(families, ["Ubuntu"], "one face, family read from the name table");
    assert_eq!(faces[0].weight, 300.0, "“Light” reads as weight 300");
    assert!(!faces[0].italic);
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn a_link_back_to_a_parent_folder_ends_the_scan() {
    let dir = std::env::temp_dir().join(format!("gridcraft-fonts-loop-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("Sub")).expect("temp dir");
    std::fs::write(dir.join("Sub/Ubuntu-Light.ttf"), UBUNTU).expect("write");
    std::os::unix::fs::symlink(&dir, dir.join("Sub/loop")).expect("symlink");
    assert_eq!(scan_dirs(std::slice::from_ref(&dir)).len(), 1, "the loop doesn't multiply faces");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn language_reorders_only_the_han_faces() {
    let keys = |han| fallback_faces(han).into_iter().map(|(k, _)| k).collect::<Vec<_>>();
    assert_eq!(
        keys(HanOrder::Japanese),
        ["sys-cjk-han-ja", "sys-cjk-han-zh", "sys-cjk-hangul", "sys-thai", "sys-hebrew", "sys-symbols", "sys-symbols-2"]
    );
    assert_eq!(
        keys(HanOrder::Chinese),
        ["sys-cjk-han-zh", "sys-cjk-han-ja", "sys-cjk-hangul", "sys-thai", "sys-hebrew", "sys-symbols", "sys-symbols-2"]
    );
}
