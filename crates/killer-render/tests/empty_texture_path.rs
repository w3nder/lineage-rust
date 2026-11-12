use killer_assets::client::Client;
use killer_assets::scene::build_scene;
use killer_assets::textures::{BankSource, TextureBank, TextureData};
use killer_render::headless::Headless;
use killer_render::textures::TextureSource;
use std::path::PathBuf;
use std::sync::Arc;

struct Spy<'a> {
    inner: BankSource<'a>,
    empty: usize,
}

impl TextureSource for Spy<'_> {
    fn texture_lod(&mut self, path: &str) -> Option<(i32, i32)> {
        self.empty += usize::from(path.is_empty());
        self.inner.texture_lod(path)
    }

    fn wet_texture(&mut self, path: &str) -> Option<Arc<l2_catalog::WetTexture>> {
        self.inner.wet_texture(path)
    }

    fn texture(&mut self, path: &str) -> Arc<TextureData> {
        self.empty += usize::from(path.is_empty());
        self.inner.texture(path)
    }

    fn alpha_map(&mut self, path: &str) -> Arc<TextureData> {
        self.empty += usize::from(path.is_empty());
        self.inner.alpha_map(path)
    }
}

#[test]
#[ignore]
fn a_null_material_reference_never_reaches_the_texture_bank_in_the_lobby_or_a_tile() {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from).expect("KILLER_CLIENT_DIR");
    let mut client = Client::open(&root).unwrap();
    for name in ["lobby", "22_22"] {
        let loaded = client.load_level(name).unwrap();
        let scene = build_scene(&mut client, &loaded.level);
        let mut headless = Headless::new(64, 64).unwrap();
        let mut bank = TextureBank::new();
        let mut spy = Spy { inner: BankSource { bank: &mut bank, catalog: client.catalog() }, empty: 0 };
        headless.upload(&scene, &mut spy);
        let empty = spy.empty;
        assert_eq!(empty, 0, "{name}");
        assert_eq!(bank.missing().count(), 0, "{name}: {:?}", bank.missing().collect::<Vec<_>>());
    }
}
