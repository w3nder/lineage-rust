use killer_assets::client::Client;
use killer_ui::font::Glyphs;
use std::path::PathBuf;

const SMALL_FIXTURE: &[u8] = include_bytes!("../../killer-ui/tests/fixtures/smallfont-e.gly");

fn test_client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

#[test]
#[ignore]
fn small_font_gly_matches_the_evidence() {
    let Some(mut client) = test_client() else { return };
    let bytes = client.read_system_file("SmallFont-e.gly").unwrap();
    assert_eq!(bytes, SMALL_FIXTURE);
    let glyphs = Glyphs::parse(&bytes).unwrap();
    assert_eq!((glyphs.tex_width, glyphs.tex_height), (1024, 128));
    assert_eq!(glyphs.glyphs.len(), 95);
    assert_eq!(glyphs.glyphs[&' '].width, 3);
    assert_eq!(glyphs.glyphs[&'0'].width, 6);
    assert!(glyphs.glyphs.values().all(|g| g.height == 13));
}

#[test]
#[ignore]
fn large_font_gly_matches_the_evidence() {
    let Some(mut client) = test_client() else { return };
    let glyphs = Glyphs::parse(&client.read_system_file("LargeFont-e.gly").unwrap()).unwrap();
    assert_eq!(glyphs.glyphs.len(), 95);
    assert_eq!(glyphs.glyphs[&' '].width, 4);
    assert!(glyphs.glyphs.values().all(|g| g.height == 14));
}
