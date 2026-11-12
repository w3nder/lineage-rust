use glam::vec3;
use killer_assets::client::Client;
use killer_assets::textures::{BankSource, TextureBank, write_png};
use killer_render::camera::Camera;
use killer_render::environment::Environment;
use killer_render::headless::{CLEAR_RGBA, Headless};
use killer_ui::detail_status::{DetailStatusView, HP_BAR, LEFT_LABEL_X, MYINFO_BACK, Sysstrings};
use killer_ui::draw::DrawList;
use killer_ui::font::{BitmapFont, Glyphs};
use killer_ui::inventory::{
    BODY_PART_CHEST, BODY_PART_L_HAND, BODY_PART_R_HAND, GRID_RECT, INVENTORY_SIZE, InventoryItemView, InventoryView,
    PAPERDOLL_RECT, paperdoll_slot_pos,
};
use killer_ui::keys::MainTab;
use killer_ui::mainwnd::{MAIN_SIZE, MainWndView, PAGE_RECT, tab_rect};
use killer_ui::skills::{LIST_RECT, SkillView, SkillsView};
use l2_dat::{ItemIcons, ItemKind, itemicons, skillgrp, sysstring};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const SMALL_FONT: &str = "L2Font-e.SmallFont-e";
const MARGIN: i32 = 10;
const DEMO_ITEMS: [(i32, i32, i32, bool, i32); 6] = [
    (57, 100, 152_340, false, 0),
    (1, 101, 1, true, BODY_PART_R_HAND),
    (18, 102, 1, true, BODY_PART_L_HAND),
    (21, 103, 1, true, BODY_PART_CHEST),
    (1060, 107, 15, false, 0),
    (17, 108, 500, false, 0),
];
const DEMO_SKILLS: [(bool, i32, i32); 4] = [(false, 9, 3), (false, 1, 1177), (true, 1, 194), (false, 1, 56)];

fn pixel_in(rgba: &[u8], width: u32, x: i32, y: i32) -> [u8; 4] {
    let p = &rgba[((y as u32 * width + x as u32) * 4) as usize..][..4];
    [p[0], p[1], p[2], p[3]]
}

fn small_font(client: &mut Client) -> BitmapFont {
    let glyphs = Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap();
    BitmapFont::new(SMALL_FONT.into(), glyphs)
}

fn sysstrings(client: &mut Client) -> HashMap<u32, String> {
    let body = client.read_system_file("sysstring-e.dat").unwrap();
    sysstring::read(&body).unwrap().into_iter().map(|s| (s.id, s.text)).collect()
}

fn item_icons(client: &mut Client) -> ItemIcons {
    let mut icons = ItemIcons::default();
    for (name, kind) in [("etcitemgrp.dat", ItemKind::Etc), ("armorgrp.dat", ItemKind::Armor), ("weapongrp.dat", ItemKind::Weapon)] {
        let body = client.read_system_file(name).unwrap();
        icons.insert_all(itemicons::read(&body, kind).unwrap());
    }
    icons
}

fn render(client: &mut Client, headless: &mut Headless, list: &DrawList, name: &str, width: u32, height: u32) -> Vec<u8> {
    let mut bank = TextureBank::new();
    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
    headless.renderer_mut().set_ui(list, &mut source);
    let missing: Vec<&str> = bank.missing().collect();
    assert!(missing.is_empty(), "texturas ausentes em {name}: {missing:?}");
    let camera = Camera { eye: vec3(0.0, -300.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    let rgba = headless.render(&camera, &Environment::default()).unwrap();
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders").join(name);
    write_png(&out, width, height, &rgba).unwrap();
    rgba
}

fn inventory_view(client: &mut Client, strings: &HashMap<u32, String>) -> InventoryView {
    let icons = item_icons(client);
    let mut view = InventoryView::with_sysstrings(|id| strings.get(&id).cloned());
    view.items = DEMO_ITEMS
        .iter()
        .map(|&(item_id, object_id, count, equipped, body_part)| InventoryItemView {
            object_id,
            item_id,
            count,
            equipped,
            body_part,
            icons: icons.get(item_id as u32).map(|i| i.icons.to_vec()).unwrap_or_default(),
            ..Default::default()
        })
        .collect();
    view.adena = 152_340;
    view.load = 2_310;
    view.max_load = 8_290;
    view.limit = 80;
    view.selected = Some(107);
    view
}

#[test]
#[ignore]
fn captures_the_inventory_with_real_icons() {
    let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
    let mut client = Client::open(&root).unwrap();
    let (w, h) = INVENTORY_SIZE;
    let (width, height) = ((w + 2 * MARGIN) as u32, (h + 2 * MARGIN) as u32);
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = small_font(&mut client);
    let strings = sysstrings(&mut client);
    let view = inventory_view(&mut client, &strings);
    assert!(view.items.iter().all(|i| !i.icons.is_empty() && !i.icons[0].is_empty()), "item sem ícone no *grp.dat");
    let mut list = DrawList::default();
    view.paint(&mut list, MARGIN, MARGIN, &font, 1.0);
    let rgba = render(&mut client, &mut headless, &list, "ui-inventory.png", width, height);
    assert_eq!(pixel_in(&rgba, width, 2, 2), CLEAR_RGBA);
    assert_ne!(pixel_in(&rgba, width, MARGIN + w / 2, MARGIN + 200), CLEAR_RGBA, "Inventory_Back");
    let (sx, sy) = paperdoll_slot_pos(5);
    let sword = pixel_in(&rgba, width, MARGIN + PAPERDOLL_RECT.x + sx + 16, MARGIN + PAPERDOLL_RECT.y + sy + 16);
    let empty = pixel_in(&rgba, width, MARGIN + PAPERDOLL_RECT.x + 130 + 16, MARGIN + PAPERDOLL_RECT.y + 7 + 16);
    assert_ne!(sword, empty, "ícone da espada igual ao slot vazio");
    let coin = pixel_in(&rgba, width, MARGIN + GRID_RECT.x + 1 + 16, MARGIN + GRID_RECT.y + 1 + 16);
    assert_ne!(coin, pixel_in(&rgba, width, MARGIN + GRID_RECT.x + 1 + 16 + 37 * 3, MARGIN + GRID_RECT.y + 1 + 16), "ícone da adena");
}

#[test]
#[ignore]
fn captures_the_main_window_with_the_skill_list() {
    let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
    let mut client = Client::open(&root).unwrap();
    let (w, h) = MAIN_SIZE;
    let (width, height) = ((w + 2 * MARGIN) as u32, (h + 2 * MARGIN) as u32);
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = small_font(&mut client);
    let strings = sysstrings(&mut client);
    let skill_table = skillgrp::by_id_level(skillgrp::read(&client.read_system_file("skillgrp.dat").unwrap()).unwrap());
    let mut main = MainWndView::with_sysstrings(|id| strings.get(&id).cloned());
    main.tab = MainTab::Skills;
    let mut skills = SkillsView::with_sysstrings(|id| strings.get(&id).cloned());
    skills.skills = DEMO_SKILLS
        .iter()
        .map(|&(passive, level, id)| SkillView {
            id,
            level,
            passive,
            icon: skill_table.get(&(id as u32, level as u32)).map(|s| s.icon.clone()).unwrap_or_default(),
            ..Default::default()
        })
        .collect();
    assert!(skills.skills.iter().all(|s| !s.icon.is_empty()), "skill sem ícone no skillgrp.dat");
    skills.skills[1].cool = Some(killer_ui::cooltime::CoolFrame::Running(180));
    skills.selected = Some(3);
    let mut list = DrawList::default();
    main.paint_chrome(&mut list, MARGIN, MARGIN, &font, 1.0);
    skills.paint(&mut list, MARGIN + PAGE_RECT.x, MARGIN + PAGE_RECT.y, &font);
    let rgba = render(&mut client, &mut headless, &list, "ui-skills.png", width, height);
    let mut list = DrawList::default();
    main.tab = MainTab::Actions;
    main.paint_chrome(&mut list, MARGIN, MARGIN, &font, 1.0);
    let rgba_main = render(&mut client, &mut headless, &list, "ui-mainwnd.png", width, height);
    assert_eq!(pixel_in(&rgba, width, 2, 2), CLEAR_RGBA);
    let tab = tab_rect(MainTab::Skills);
    assert_ne!(pixel_in(&rgba, width, MARGIN + tab.x + tab.w / 2, MARGIN + tab.y + tab.h / 2), CLEAR_RGBA, "aba de skills");
    let icon = pixel_in(&rgba, width, MARGIN + PAGE_RECT.x + LIST_RECT.x + 1 + 16, MARGIN + PAGE_RECT.y + LIST_RECT.y + 1 + 16);
    let blank = pixel_in(&rgba, width, MARGIN + PAGE_RECT.x + LIST_RECT.x + 1 + 16 + 37 * 5, MARGIN + PAGE_RECT.y + LIST_RECT.y + 1 + 16);
    assert_ne!(icon, blank, "ícone da skill igual ao fundo");
    assert_ne!(pixel_in(&rgba_main, width, MARGIN + w / 2, MARGIN + 40), CLEAR_RGBA, "MainWndTabBack");
    assert_eq!(pixel_in(&rgba_main, width, MARGIN + w / 2, MARGIN + 200), CLEAR_RGBA, "a página vazia do NCMainWnd não tem fundo próprio");
}

#[test]
#[ignore]
fn captures_the_detail_status_page() {
    let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
    let mut client = Client::open(&root).unwrap();
    let (w, h) = MAIN_SIZE;
    let (width, height) = ((w + 2 * MARGIN) as u32, (h + 2 * MARGIN) as u32);
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = small_font(&mut client);
    let strings = sysstrings(&mut client);
    let main = MainWndView::with_sysstrings(|id| strings.get(&id).cloned());
    let view = DetailStatusView {
        name: "Killer".into(),
        level: 52,
        class_id: 2,
        hp: (400, 900),
        mp: (200, 300),
        cp: (100, 100),
        exp: 52262490,
        sp: 12345,
        load: 2310,
        max_load: 8290,
        p_atk: 123,
        p_def: 456,
        accuracy: 78,
        critical: 90,
        atk_spd: 300,
        m_atk: 54,
        m_def: 321,
        evasion: 77,
        speed: 120,
        cast_spd: 333,
        stats: [40, 30, 43, 21, 11, 25],
        pvp_kills: 3,
        pk_kills: 1,
        recom_have: 7,
        recom_left: 5,
        ..Default::default()
    };
    let lookup = |id: u32| strings.get(&id).cloned();
    let mut list = DrawList::default();
    main.paint_chrome(&mut list, MARGIN, MARGIN, &font, 1.0);
    view.paint(&mut list, MARGIN + PAGE_RECT.x, MARGIN + PAGE_RECT.y, &font, &Sysstrings(&lookup));
    assert!(list.quads.iter().any(|q| q.texture == MYINFO_BACK));
    let rgba = render(&mut client, &mut headless, &list, "ui-status-detail.png", width, height);
    let filled = pixel_in(&rgba, width, MARGIN + PAGE_RECT.x + HP_BAR.0 + 10, MARGIN + PAGE_RECT.y + HP_BAR.1 + 1);
    let empty = pixel_in(&rgba, width, MARGIN + PAGE_RECT.x + HP_BAR.0 + 80, MARGIN + PAGE_RECT.y + HP_BAR.1 + 1);
    assert!(filled[0] > empty[0], "barra de HP não é mais vermelha que o fundo");
    let label = (0..font.measure(&strings[&94]) as i32)
        .flat_map(|dx| (0..font.line_height as i32).map(move |dy| (dx, dy)))
        .map(|(dx, dy)| pixel_in(&rgba, width, MARGIN + PAGE_RECT.x + LEFT_LABEL_X + dx, MARGIN + PAGE_RECT.y + 132 + dy));
    assert!(label.clone().any(|p| p[0] > 120), "rótulo P. Atk. ilegível");
    assert_eq!(pixel_in(&rgba, width, 2, 2), CLEAR_RGBA);
}
