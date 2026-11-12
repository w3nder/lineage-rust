use killer_ui::font::{BitmapFont, Glyphs};

const SMALL: &[u8] = include_bytes!("fixtures/smallfont-e.gly");

#[test]
fn small_font_fixture_has_95_ascii_glyphs_of_13_px() {
    let glyphs = Glyphs::parse(SMALL).unwrap();
    assert_eq!(SMALL.len(), 1560);
    assert_eq!((glyphs.tex_width, glyphs.tex_height), (1024, 128));
    assert_eq!(glyphs.glyphs.len(), 95);
    assert!(glyphs.glyphs.values().all(|g| g.height == 13 && g.y == 0));
    assert_eq!(glyphs.glyphs[&' '].width, 3);
    assert_eq!(glyphs.glyphs[&'0'].width, 6);
    assert_eq!(glyphs.glyphs[&'W'].width, 10);
    assert_eq!(glyphs.glyphs[&'~'].x, 646);
    assert!(!glyphs.glyphs.contains_key(&'\u{7f}'));
}

#[test]
fn small_font_fixture_measures_like_the_client() {
    let font =
        BitmapFont { atlas: "L2Font-e.SmallFont-e".into(), glyphs: Glyphs::parse(SMALL).unwrap(), line_height: 13 };
    assert_eq!(font.measure("HP 100/100"), 7 + 6 + 3 + 6 * 6 + 4);
    assert_eq!(font.measure("ab\nWWW"), 30);
}
