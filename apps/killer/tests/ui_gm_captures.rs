use glam::vec3;
use killer_assets::client::Client;
use killer_assets::textures::{BankSource, TextureBank, write_png};
use killer_render::camera::Camera;
use killer_render::environment::Environment;
use killer_render::headless::Headless;
use killer_ui::detail_status::DetailStatusView;
use killer_ui::draw::DrawList;
use killer_ui::font::{BitmapFont, Glyphs};
use killer_ui::gm_clan::{GmClanView, GmPledgeInfo};
use killer_ui::gm_inventory::GmInventoryView;
use killer_ui::gm_status::GmStatusView;
use killer_ui::hero_tower::{HeroRow, HeroTowerView};
use killer_ui::inventory::InventoryItemView;
use killer_ui::lobby_replay::{ReplayLabels, ReplayWnd};
use killer_ui::pledge::MemberRow;
use l2_dat::sysstring;
use std::collections::HashMap;
use std::path::PathBuf;

fn setup() -> Option<(Client, BitmapFont, HashMap<u32, String>)> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    let mut client = Client::open(&root).ok()?;
    let glyphs = Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap();
    let font = BitmapFont::new("L2Font-e.SmallFont-e".into(), glyphs);
    let body = client.read_system_file("sysstring-e.dat").unwrap();
    let strings = sysstring::read(&body).unwrap().into_iter().map(|s| (s.id, s.text)).collect();
    Some((client, font, strings))
}

fn capture(client: &mut Client, list: &DrawList, name: &str, size: (u32, u32)) {
    let Ok(mut headless) = Headless::new(size.0, size.1) else { return };
    let mut bank = TextureBank::new();
    headless.renderer_mut().set_ui(list, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
    let camera = Camera { eye: vec3(0.0, -300.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    let rgba = headless.render(&camera, &Environment::default()).unwrap();
    let dir = PathBuf::from(std::env::var("KILLER_CAPTURE_DIR").unwrap_or_else(|_| "/tmp/killer-captures".into()));
    std::fs::create_dir_all(&dir).unwrap();
    write_png(&dir.join(format!("{name}.png")), size.0, size.1, &rgba).unwrap();
}

fn item(object_id: i32, item_id: i32, body_part: i32, equipped: bool, icon: &str) -> InventoryItemView {
    InventoryItemView { object_id, item_id, count: 1, body_part, equipped, icons: vec![icon.into(), "icon.weapon_long_bow_i01".into(), "icon.weapon_long_bow_i02".into()], ..Default::default() }
}

fn icon_of(tables: &killer_assets::client::DatTables, id: u32) -> String {
    tables.item_icons.get(id).and_then(|icons| icons.icons.first().map(|name| name.trim().to_string())).unwrap_or_default()
}

fn bow_and_ammunition(view: &mut GmInventoryView, tables: &killer_assets::client::DatTables) {
    let bow_id = *tables.weapongrp.iter().filter(|(_, weapon)| weapon.weapon_type == 6).map(|(id, _)| id).min().unwrap();
    let arrow_id = *tables.etcitemgrp.iter().filter(|(_, etc)| etc.family == 2).map(|(id, _)| id).min().unwrap();
    let mut bow = item(1, bow_id as i32, 0x4000, true, &icon_of(tables, bow_id));
    bow.tip = Some(killer_ui::item_tip::ItemTipData { weapon_kind: 6, ..Default::default() });
    let mut arrow = item(2, arrow_id as i32, 0x80, true, &icon_of(tables, arrow_id));
    arrow.tip = Some(killer_ui::item_tip::ItemTipData { etc_family: 2, ..Default::default() });
    view.add_item(bow);
    view.add_item(arrow);
}

fn tattoos(view: &mut GmInventoryView, tables: &killer_assets::client::DatTables) {
    let mut ids: Vec<u32> = tables.hennagrp.keys().copied().collect();
    ids.sort_unstable();
    view.add_henna(tables.hennagrp[&ids[0]].icon.trim(), true);
    view.add_henna(tables.hennagrp[&ids[1]].icon.trim(), false);
}

fn clan_of_status(status: &mut DetailStatusView) {
    status.clan_name = Some("Killers".into());
    status.clan_crest_id = 7;
    status.clan_crest = Some("L2UI_CH3.Tooltip.Tooltip1".into());
}

#[test]
#[ignore]
fn captures_the_gm_windows_and_the_hero_tower_and_the_replay_list() {
    let Some((mut client, font, strings)) = setup() else { return };
    let lookup = |id: u32| strings.get(&id).cloned();
    let tables = client.tables().unwrap();

    let mut inventory = GmInventoryView::new(lookup);
    bow_and_ammunition(&mut inventory, &tables);
    tattoos(&mut inventory, &tables);
    inventory.add_item(item(3, 57, 0, false, &icon_of(&tables, 57)));
    inventory.limit = 80;
    let mut list = DrawList::default();
    inventory.paint(&mut list, (10, 10), &font, 1.0);
    capture(&mut client, &list, "gm-inventory", (280, 420));

    let mut status = GmStatusView::new(lookup);
    let mut view = DetailStatusView { name: "Gm Target".into(), level: 40, class_id: 10, ..Default::default() };
    clan_of_status(&mut view);
    status.set(view, [120, 80, 50, 40, 150, 100], 1.0);
    let mut list = DrawList::default();
    status.paint(&mut list, (10, 10), &font, 1.0, &lookup);
    capture(&mut client, &list, "gm-status", (280, 380));

    let mut clan = GmClanView::new(lookup);
    clan.apply_info(&GmPledgeInfo { clan_id: 5, name: "Killers".into(), leader: "Luzal".into(), level: 3, castle: true, hideout: false });
    clan.update_member(MemberRow { name: "Luzal".into(), level: 40, class_id: 2, online: true, object_id: 1 });
    let mut list = DrawList::default();
    clan.paint(&mut list, (10, 10), &font, 1.0);
    capture(&mut client, &list, "gm-clan", (280, 380));

    let mut tower = HeroTowerView::new(lookup);
    tower.set_font(&font);
    tower.set_heroes(&[HeroRow { name: "Luzal".into(), class_id: 88, class_name: "Duelist".into(), clan: "Killers".into(), clan_crest: None, ally: "Allies".into(), ally_crest: None, count: 7 }]);
    let mut list = DrawList::default();
    tower.paint(&mut list, (10, 10), &font, 1.0, &lookup);
    capture(&mut client, &list, "hero-tower", (620, 350));

    let mut replay = ReplayWnd::new((340, 380), ReplayLabels::from_sysstrings(lookup));
    replay.show(&["Rep00001.l2r".to_string(), "Rep00002.l2r".to_string()]);
    let mut list = DrawList::default();
    replay.paint(&mut list, &font, &|id| strings.get(&(id as u32)).cloned());
    capture(&mut client, &list, "replay-list", (340, 380));
}
