use crate::draw::DrawList;
use crate::font::{BitmapFont, Glyphs, gly};
use crate::manor_crop_sell::{CropSellView, ProcureChangeView};
use crate::manor_crop_setting::{CropChangeView, CropSettingView};
use crate::manor_info::{ManorInfoView, ManorPage};
use crate::manor_setting::{SeedChangeView, SeedSettingView};

const ORIGIN: (i32, i32) = (100, 200);

type Texture = (&'static str, i32, i32, i32, i32, i32, i32);

struct Oracle {
    textures: &'static [Texture],
    texts: &'static [(i32, i32)],
}

fn font() -> BitmapFont {
    let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
    BitmapFont::new("font".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
}

fn lookup(id: u32) -> Option<String> {
    Some(format!("s{id}"))
}

fn check(list: &DrawList, oracle: &Oracle, origin: (i32, i32)) {
    for &(texture, x, y, w, h, uw, vh) in oracle.textures {
        let found = list.quads.iter().any(|q| {
            q.texture == texture
                && (q.x, q.y, q.w, q.h, q.uw, q.vh) == ((origin.0 + x) as f32, (origin.1 + y) as f32, w as f32, h as f32, uw as f32, vh as f32)
        });
        assert!(found, "textura do oráculo ausente: {texture} em ({x}, {y}, {w}, {h}, {uw}, {vh})");
    }
    for &(x, y) in oracle.texts {
        let found = list.quads.iter().any(|q| q.texture == "font" && (q.x, q.y) == ((origin.0 + x) as f32, (origin.1 + y) as f32));
        assert!(found, "texto do oráculo ausente em ({x}, {y})");
    }
}

#[test]
fn nc_manor_seed_info_setting_wnd_101395b0_matches_the_unicorn_paint() {
    const ORACLE: Oracle = Oracle {
        textures: &[("L2UI_CH3.Manor.manor2_back", 0, 20, 600, 306, 600, 306), ("L2UI.WindowIcon.WindowIconAdena", 107, 278, 16, 12, 16, 12)],
        texts: &[(16, 47), (16, 279), (130, 279)],
    };
    let mut view = SeedSettingView::new(lookup);
    view.manor = 1;
    view.total = 12345;
    let mut list = DrawList::default();
    view.paint(&mut list, ORIGIN, &font(), 1.0, &lookup);
    check(&list, &ORACLE, ORIGIN);
}

#[test]
fn nc_manor_crop_info_setting_wnd_101311b0_matches_the_unicorn_paint() {
    const ORACLE: Oracle = Oracle {
        textures: &[("L2UI_CH3.Manor.manor2_back", 0, 20, 600, 306, 600, 306), ("L2UI.WindowIcon.WindowIconAdena", 107, 278, 16, 12, 16, 12)],
        texts: &[(16, 47), (16, 279), (130, 279)],
    };
    let mut view = CropSettingView::new(lookup);
    view.manor = 1;
    view.total = 12345;
    let mut list = DrawList::default();
    view.paint(&mut list, ORIGIN, &font(), 1.0, &lookup);
    check(&list, &ORACLE, ORIGIN);
}

#[test]
fn nc_manor_seed_info_change_wnd_10138d90_matches_the_unicorn_paint() {
    const ORACLE: Oracle = Oracle {
        textures: &[("L2UI_CH3.Manor.manor4_back", 0, 20, 256, 181, 256, 181)],
        texts: &[(21, 40), (31, 71), (31, 96), (31, 121), (145, 96)],
    };
    let mut view = SeedChangeView::new(lookup);
    view.name = "Seed".into();
    view.limit = 5;
    let mut list = DrawList::default();
    view.paint(&mut list, ORIGIN, &font(), 1.0, &lookup);
    check(&list, &ORACLE, ORIGIN);
}

#[test]
fn nc_manor_crop_info_change_wnd_10130530_matches_the_unicorn_paint() {
    const ORACLE: Oracle = Oracle {
        textures: &[("L2UI_CH3.Manor.manor4_back", 0, 20, 256, 181, 256, 181)],
        texts: &[(21, 40), (31, 71), (31, 96), (31, 121), (31, 146), (145, 96)],
    };
    let mut view = CropChangeView::new(lookup);
    view.name = "Crop".into();
    view.limit = 5;
    let mut list = DrawList::default();
    view.paint(&mut list, ORIGIN, &font(), 1.0, &lookup);
    check(&list, &ORACLE, ORIGIN);
}

#[test]
fn nc_manor_crop_sell_wnd_10135880_matches_the_unicorn_paint() {
    const ORACLE: Oracle = Oracle { textures: &[("L2UI_CH3.Manor.manor2_back", 0, 20, 600, 306, 600, 306)], texts: &[] };
    let view = CropSellView::new(lookup);
    let mut list = DrawList::default();
    view.paint(&mut list, ORIGIN, &font(), 1.0, &lookup);
    check(&list, &ORACLE, ORIGIN);
}

#[test]
fn nc_manor_crop_sell_change_wnd_10134350_matches_the_unicorn_paint() {
    const ORACLE: Oracle = Oracle {
        textures: &[("L2UI_CH3.Manor.manor3_back", 0, 20, 336, 286, 336, 286)],
        texts: &[(15, 38), (76, 38), (15, 54), (76, 54), (15, 70), (76, 70), (75, 219), (75, 244)],
    };
    let mut view = ProcureChangeView::new(lookup);
    view.name = "Name".into();
    view.first = "First".into();
    view.second = "Second".into();
    let mut list = DrawList::default();
    view.paint(&mut list, ORIGIN, &font(), 1.0, &lookup);
    check(&list, &ORACLE, ORIGIN);
}

#[test]
fn nc_manor_info_pages_match_the_unicorn_paint_of_up_default_and_crop() {
    const UP: Oracle = Oracle { textures: &[("L2UI_CH3.Manor.manor1_back1", 0, 20, 600, 28, 600, 28)], texts: &[] };
    const PAGE: Oracle = Oracle { textures: &[("L2UI_CH3.Manor.manor1_back2", 0, 48, 600, 278, 600, 278)], texts: &[(16, 63)] };
    let mut view = ManorInfoView::new(lookup);
    view.set_castles(vec![1, 2]);
    assert_eq!(view.page(), Some(ManorPage::Seed));
    let mut list = DrawList::default();
    view.paint(&mut list, ORIGIN, &font(), 1.0, &lookup);
    check(&list, &UP, ORIGIN);
    check(&list, &PAGE, ORIGIN);
}
