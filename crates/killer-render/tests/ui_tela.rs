use glam::vec3;
use killer_assets::client::Client;
use killer_assets::textures::{BankSource, TextureBank, write_png};
use killer_render::camera::Camera;
use killer_render::environment::Environment;
use killer_render::headless::{CLEAR_RGBA, Headless};
use killer_ui::abnormal::{ABNORMAL_BACK, AbnormalView, BuffIcon, CELL, ICONS_X, ROW_STEP, paint_abnormal};
use killer_ui::chat::ChatEntry;
use killer_ui::color::{Argb, CHAT_SYSTEM};
use killer_ui::dialog::{
    BUTTON_Y as DIALOG_BUTTON_Y, DIALOG_SIZE, DialogButton, DialogKind, DialogView, FIRST_BUTTON_X, TIMER_RECT,
    paint_dialog,
};
use killer_ui::draw::DrawList;
use killer_ui::font::{BitmapFont, Glyphs};
use killer_ui::gauge::{GAUGE_BACK, GAUGE_FILL, GaugeView, paint_gauges};
use killer_ui::ini::parse_ini;
use killer_ui::loading::{loading_texture, paint_loading};
use killer_ui::menu::{BUTTON_XS, BUTTON_Y, MENU_SIZE, MenuButton, paint_menu};
use killer_ui::party::{
    BAR_X as PARTY_BAR_X, CELL_STEP, HP_BAR_Y as PARTY_HP_BAR_Y, PARTY_WIDTH, PartyMemberView, PartyView, paint_party,
};
use killer_ui::target::{TARGET_SIZE, TargetKind, TargetView, paint_target};
use killer_ui::pet_status::{
    BAR_X as PET_BAR_X, HP_BAR_Y as PET_HP_BAR_Y, PET_BUTTON_RECT, PET_STATUS_SIZE, PetStatusView, THIRD_BAR_Y,
    ThirdBar, paint_pet_status,
};
use killer_ui::restart::{BUTTON_X as RESTART_BUTTON_X, FIRST_BUTTON_Y, RESTART_BACK_SIZE, RestartView, paint_restart};
use killer_ui::sysmsg::{
    LINE_TEXT_X as SYSMSG_TEXT_X, LINE_TEXT_Y as SYSMSG_TEXT_Y, SYSTEM_MSG_SIZE, SystemMsgOptions, SystemMsgView,
};
use killer_ui::zone_title::{FONT_DECO, ZoneTitleView, paint_zone_title};
use l2_dat::skillgrp;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const SMALL_FONT: &str = "L2Font-e.SmallFont-e";
const ABNORMAL_CANVAS: (u32, u32) = (300, 100);
const ABNORMAL_POS: (i32, i32) = (10, 10);
const MENU_CANVAS: (u32, u32) = (200, 70);
const MENU_POS: (i32, i32) = (10, 10);
const DIALOG_CANVAS: (u32, u32) = (280, 130);
const DIALOG_POS: (i32, i32) = (10, 10);
const PET_CANVAS: (u32, u32) = (360, 130);
const PET_POS: (i32, i32) = (10, 10);
const RESTART_CANVAS: (u32, u32) = (160, 150);
const RESTART_POS: (i32, i32) = (10, 10);
const SYSMSG_CANVAS: (u32, u32) = (368, 145);
const SYSMSG_POS: (i32, i32) = (10, 10);
const ZONE_CANVAS: (u32, u32) = (420, 200);
const ZONE_POS: (i32, i32) = (110, 90);
const LARGE_FONT: &str = "L2Font-e.LargeFont-e";
const PARTY_CANVAS: (u32, u32) = (360, 160);
const PARTY_POS: (i32, i32) = (10, 10);
const LOADING_CANVAS: (u32, u32) = (256, 192);
const GAUGE_CANVAS: (u32, u32) = (140, 80);
const FIRST_NPC_SKILL: u32 = 4408;
const MISSING_NPC_SKILL_ICONS: usize = 59;

fn client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Some(Client::open(&root).unwrap())
}

fn small_font(client: &mut Client) -> BitmapFont {
    let glyphs = Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap();
    BitmapFont::new(SMALL_FONT.into(), glyphs)
}

fn large_font(client: &mut Client) -> BitmapFont {
    let glyphs = Glyphs::parse(&client.read_system_file("LargeFont-e.gly").unwrap()).unwrap();
    BitmapFont::new(LARGE_FONT.into(), glyphs)
}

fn pixel_in(rgba: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let p = &rgba[((y * width + x) * 4) as usize..][..4];
    [p[0], p[1], p[2], p[3]]
}

fn render_list(client: &mut Client, headless: &mut Headless, list: &DrawList) -> Vec<u8> {
    let mut bank = TextureBank::new();
    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
    headless.renderer_mut().set_ui(list, &mut source);
    let camera =
        Camera { eye: vec3(0.0, -300.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    let rgba = headless.render(&camera, &Environment::default()).unwrap();
    let missing: Vec<&str> = bank.missing().collect();
    assert!(missing.is_empty(), "texturas ausentes no cliente: {missing:?}");
    rgba
}

fn save(name: &str, size: (u32, u32), rgba: &[u8]) {
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders").join(name);
    write_png(&out, size.0, size.1, rgba).unwrap();
}

fn skill_icon(client: &mut Client, id: u32, level: u32) -> String {
    let table: skillgrp::SkillTable =
        skillgrp::read(&client.read_system_file("skillgrp.dat").unwrap()).unwrap().into_iter().collect();
    table.lookup(id, level).unwrap().icon.clone()
}

#[test]
#[ignore]
fn every_skillgrp_icon_exists_in_the_client_textures() {
    let Some(mut client) = client() else { return };
    let skills = skillgrp::read(&client.read_system_file("skillgrp.dat").unwrap()).unwrap();
    let icons: BTreeSet<String> = skills.iter().map(|s| s.icon.to_lowercase()).collect();
    assert!(icons.len() > 900, "{} ícones distintos", icons.len());
    let missing: Vec<&String> = icons.iter().filter(|icon| client.catalog().texture_raw(icon).is_none()).collect();
    let player_icons: BTreeSet<&str> =
        skills.iter().filter(|s| s.id < FIRST_NPC_SKILL).map(|s| s.icon.as_str()).collect();
    let missing_player: Vec<&&str> =
        player_icons.iter().filter(|icon| missing.iter().any(|m| m.eq_ignore_ascii_case(icon))).collect();
    assert!(missing_player.is_empty(), "ícones de skills de jogador sem textura: {missing_player:?}");
    assert_eq!(missing.len(), MISSING_NPC_SKILL_ICONS, "ícones do skillgrp sem textura: {missing:?}");
}

#[test]
#[ignore]
fn captures_the_abnormal_window_with_the_client_textures() {
    let Some(mut client) = client() else { return };
    let (width, height) = ABNORMAL_CANVAS;
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = small_font(&mut client);
    let icon = |client: &mut Client, id: u32, level: u32, remaining, debuff| BuffIcon {
        skill_id: id as i32,
        level: level as i32,
        icon: Some(skill_icon(client, id, level)),
        remaining,
        debuff,
        fade_ticks: 0,
    };
    let view = AbnormalView {
        buffs: vec![
            icon(&mut client, 1204, 2, Some(1190), false),
            icon(&mut client, 1040, 3, None, false),
            icon(&mut client, 1069, 1, Some(20), true),
            BuffIcon { fade_ticks: 3, ..icon(&mut client, 1045, 1, Some(7), false) },
        ],
        short_buff: Some(icon(&mut client, 2231, 1, Some(12), false)),
        etc: Vec::new(),
        etc_skills: Vec::new(),
    };
    let (x, y) = ABNORMAL_POS;
    let mut list = DrawList::default();
    paint_abnormal(&mut list, x, y, &view, &font, 1.0);
    assert_eq!(list.quads.iter().filter(|q| q.texture == ABNORMAL_BACK).count(), 5);
    let rgba = render_list(&mut client, &mut headless, &list);
    save("ui-abnormal.png", ABNORMAL_CANVAS, &rgba);

    let at = |dx: i32, dy: i32| pixel_in(&rgba, width, (x + dx) as u32, (y + dy) as u32);
    assert_eq!(pixel_in(&rgba, width, 2, 2), CLEAR_RGBA);
    assert_ne!(at(6, 13), CLEAR_RGBA, "pegador");
    assert_ne!(at(ICONS_X + CELL / 2, CELL / 2), CLEAR_RGBA, "ícone do primeiro buff");
    assert_ne!(at(ICONS_X + CELL + CELL / 2, CELL / 2), CLEAR_RGBA, "segundo buff");
    assert_ne!(at(ICONS_X + CELL / 2, ROW_STEP + CELL / 2), CLEAR_RGBA, "buff curto na segunda linha");
    assert_eq!(at(ICONS_X + 5 * CELL + 3, CELL / 2), CLEAR_RGBA, "nada depois do quarto buff");
}

#[test]
#[ignore]
fn captures_the_target_window_on_a_door_with_the_client_textures() {
    let Some(mut client) = client() else { return };
    let canvas = (200u32, 70u32);
    let Ok(mut headless) = Headless::new(canvas.0, canvas.1) else { return };
    let font = small_font(&mut client);
    let view = TargetView::new("Clan Hall Door".into(), TargetKind::Door, Some((60, 100)), None);
    let mut list = DrawList::default();
    paint_target(&mut list, 10, 10, &view, &font, 1.0);
    let rgba = render_list(&mut client, &mut headless, &list);
    save("ui-target-door.png", canvas, &rgba);
    assert_eq!(pixel_in(&rgba, canvas.0, 2, 2), CLEAR_RGBA);
    assert_ne!(pixel_in(&rgba, canvas.0, (10 + TARGET_SIZE.0 / 2) as u32, 20), CLEAR_RGBA);
}

#[test]
#[ignore]
fn captures_the_menu_window_with_the_client_textures() {
    let Some(mut client) = client() else { return };
    let (width, height) = MENU_CANVAS;
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = small_font(&mut client);
    let (x, y) = MENU_POS;
    let mut list = DrawList::default();
    paint_menu(&mut list, x, y, Some(MenuButton::Map), &font, 1.0);
    let rgba = render_list(&mut client, &mut headless, &list);
    save("ui-menu.png", MENU_CANVAS, &rgba);

    let at = |dx: i32, dy: i32| pixel_in(&rgba, width, (x + dx) as u32, (y + dy) as u32);
    assert_eq!(pixel_in(&rgba, width, 2, 2), CLEAR_RGBA);
    assert_ne!(at(6, 23), CLEAR_RGBA, "pegador");
    assert_ne!(at(MENU_SIZE.0 - 8, 23), CLEAR_RGBA, "fatia direita do corpo");
    for bx in BUTTON_XS {
        assert_ne!(at(bx + 17, BUTTON_Y + 17), CLEAR_RGBA, "botão em {bx}");
    }
    assert_eq!(pixel_in(&rgba, width, (x + MENU_SIZE.0 + 4) as u32, (y + 23) as u32), CLEAR_RGBA);
}

#[test]
#[ignore]
fn captures_the_dialog_box_with_the_client_textures() {
    let Some(mut client) = client() else { return };
    let (width, height) = DIALOG_CANVAS;
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = small_font(&mut client);
    let mut view =
        DialogView::new(DialogKind::YesNo, "Do you wish to join Luzal's party? (Item distribution: Finders Keepers)");
    view.timer_elapsed_ms = Some(4000);
    let (x, y) = DIALOG_POS;
    let mut list = DrawList::default();
    paint_dialog(&mut list, x, y, &view, Some(DialogButton::First), &font);
    let rgba = render_list(&mut client, &mut headless, &list);
    save("ui-dialog.png", DIALOG_CANVAS, &rgba);

    let at = |dx: i32, dy: i32| pixel_in(&rgba, width, (x + dx) as u32, (y + dy) as u32);
    assert_eq!(pixel_in(&rgba, width, 2, 2), CLEAR_RGBA);
    assert_ne!(at(DIALOG_SIZE.0 / 2, 5), CLEAR_RGBA, "fundo system_back");
    assert_ne!(at(23, 27), CLEAR_RGBA, "warningicon");
    assert_ne!(at(FIRST_BUTTON_X + 10, DIALOG_BUTTON_Y + 11), CLEAR_RGBA, "botão Yes");
    assert_ne!(at(TIMER_RECT.x + 20, TIMER_RECT.y + 3), CLEAR_RGBA, "barra de tempo");
    let text = (0..font.measure("Do you") as i32)
        .flat_map(|dx| (0..font.line_height as i32).map(move |dy| (dx, dy)))
        .map(|(dx, dy)| at(41 + dx, 16 + dy));
    assert!(text.clone().any(|p| p[0] > 150), "texto ilegível");
}

#[test]
#[ignore]
fn captures_the_pet_and_summon_windows_with_the_client_textures() {
    let Some(mut client) = client() else { return };
    let (width, height) = PET_CANVAS;
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = small_font(&mut client);
    let pet = PetStatusView {
        name: "Wolf".into(),
        hp: (300, 600),
        mp: (45, 90),
        third: ThirdBar::Fed { fed: 80, max_fed: 100 },
        buffs: vec![Some(skill_icon(&mut client, 1204, 2)), Some(skill_icon(&mut client, 1040, 3))],
        show_buffs: true,
    };
    let summon = PetStatusView {
        name: "Shadow".into(),
        third: ThirdBar::Time { remaining: 300, max: 1200 },
        show_buffs: false,
        ..pet.clone()
    };
    let (x, y) = PET_POS;
    let mut list = DrawList::default();
    paint_pet_status(&mut list, x, y, &pet, &font, 1.0);
    paint_pet_status(&mut list, x, y + PET_STATUS_SIZE.1 + 10, &summon, &font, 1.0);
    let rgba = render_list(&mut client, &mut headless, &list);
    save("ui-pet.png", PET_CANVAS, &rgba);

    let at = |dx: i32, dy: i32| pixel_in(&rgba, width, (x + dx) as u32, (y + dy) as u32);
    assert_eq!(pixel_in(&rgba, width, 2, 2), CLEAR_RGBA);
    assert_ne!(at(PET_BUTTON_RECT.x + 5, PET_BUTTON_RECT.y + 6), CLEAR_RGBA, "pet_button");
    assert_ne!(at(PET_BAR_X + 10, PET_HP_BAR_Y + 2), CLEAR_RGBA, "barra de HP");
    assert_ne!(at(PET_BAR_X + 10, THIRD_BAR_Y + 2), CLEAR_RGBA, "barra de fome");
    assert_ne!(at(PET_STATUS_SIZE.0 + 8, 12), CLEAR_RGBA, "ícone de buff à direita");
    assert_ne!(at(PET_BAR_X + 10, PET_STATUS_SIZE.1 + 10 + THIRD_BAR_Y + 2), CLEAR_RGBA, "barra de tempo do summon");
}

#[test]
#[ignore]
fn captures_the_restart_menu_with_the_client_textures() {
    let Some(mut client) = client() else { return };
    let (width, height) = RESTART_CANVAS;
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = small_font(&mut client);
    let view = RestartView::new([true, false, true, false, false]);
    let (x, y) = RESTART_POS;
    let mut list = DrawList::default();
    paint_restart(&mut list, x, y, &view, Some(0), &font);
    let rgba = render_list(&mut client, &mut headless, &list);
    save("ui-restart.png", RESTART_CANVAS, &rgba);

    let at = |dx: i32, dy: i32| pixel_in(&rgba, width, (x + dx) as u32, (y + dy) as u32);
    assert_eq!(pixel_in(&rgba, width, 2, 2), CLEAR_RGBA);
    assert_ne!(at(RESTART_BACK_SIZE.0 - 5, RESTART_BACK_SIZE.1 - 5), CLEAR_RGBA, "fundo system2_back");
    assert_ne!(at(RESTART_BUTTON_X + 5, FIRST_BUTTON_Y + 11), CLEAR_RGBA, "botão To Village");
    assert_ne!(at(RESTART_BUTTON_X + 5, FIRST_BUTTON_Y + 24 * 4 + 11), CLEAR_RGBA, "botão Fixed");
}

#[test]
#[ignore]
fn captures_the_system_message_panel_with_the_client_textures() {
    let Some(mut client) = client() else { return };
    let (width, height) = SYSMSG_CANVAS;
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = small_font(&mut client);
    let system = |text: &str, group: u32, color: u32| ChatEntry {
        kind: CHAT_SYSTEM,
        name: String::new(),
        text: text.into(),
        group,
        color: Some(Argb(color)),
    };
    let lines = [
        system("Welcome to the World of Lineage II.", 2, 0xFFB09B79),
        system("You have obtained 42 Adena.", 0, 0xFFB09B79),
        system("You hit for 37 damage.", 3, 0xFFDCDCDC),
        system("The critical hit damage was 74.", 3, 0xFFDCDCDC),
        system("Soulshot use: Soulshot: No Grade.", 7, 0xFFB09B79),
    ];
    let view = SystemMsgView::new(SystemMsgOptions::default());
    let (x, y) = SYSMSG_POS;
    let mut list = DrawList::default();
    view.paint(&mut list, x, y, &lines, &font, 1.0);
    let rgba = render_list(&mut client, &mut headless, &list);
    save("ui-sysmsg.png", SYSMSG_CANVAS, &rgba);

    let at = |dx: i32, dy: i32| pixel_in(&rgba, width, (x + dx) as u32, (y + dy) as u32);
    assert_eq!(pixel_in(&rgba, width, 2, 2), CLEAR_RGBA);
    assert_ne!(at(SYSTEM_MSG_SIZE.0 / 2, SYSTEM_MSG_SIZE.1 - 3), CLEAR_RGBA, "fundo chatting_system");
    let text = (0..font.measure("Welcome") as i32)
        .flat_map(|dx| (0..font.line_height as i32).map(move |dy| (dx, dy)))
        .map(|(dx, dy)| at(SYSMSG_TEXT_X + dx, SYSMSG_TEXT_Y + dy));
    assert!(text.clone().any(|p| p[0] > 120), "primeira linha ilegível");
}

#[test]
#[ignore]
fn captures_the_zone_title_with_the_client_textures() {
    let Some(mut client) = client() else { return };
    let (width, height) = ZONE_CANVAS;
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = large_font(&mut client);
    let deco = client.catalog().texture_raw(FONT_DECO).unwrap();
    assert_eq!((deco.mips[0].width, deco.mips[0].height), (256, 256));
    let view = ZoneTitleView::shown("Giran Castle Town");
    let (x, y) = ZONE_POS;
    let mut list = DrawList::default();
    paint_zone_title(&mut list, x, y, &view, &font);
    let rgba = render_list(&mut client, &mut headless, &list);
    save("ui-zone-title.png", ZONE_CANVAS, &rgba);

    let at = |dx: i32, dy: i32| pixel_in(&rgba, width, (x + dx) as u32, (y + dy) as u32);
    assert_eq!(pixel_in(&rgba, width, 2, 2), CLEAR_RGBA);
    let text_w = font.measure("Giran Castle Town") as i32;
    let text = (0..text_w)
        .flat_map(|dx| (0..font.line_height as i32).map(move |dy| (dx, dy)))
        .map(|(dx, dy)| at(100 - text_w / 2 + dx, dy));
    assert!(text.clone().any(|p| p[0] > 200 && p[2] > 180), "título ilegível");
}

#[test]
#[ignore]
fn captures_the_party_window_with_the_client_textures() {
    let Some(mut client) = client() else { return };
    let (width, height) = PARTY_CANVAS;
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = small_font(&mut client);
    let member = |client: &mut Client, id, name: &str, leader, hp| PartyMemberView {
        object_id: id,
        name: name.into(),
        cp: (80, 100),
        hp: (hp, 600),
        mp: (45, 90),
        leader,
        buffs: vec![
            BuffIcon { skill_id: 1204, level: 2, icon: Some(skill_icon(client, 1204, 2)), remaining: Some(100), debuff: false, ..Default::default() },
            BuffIcon { skill_id: 1069, level: 1, icon: Some(skill_icon(client, 1069, 1)), remaining: Some(10), debuff: true, ..Default::default() },
        ],
    };
    let view = PartyView {
        members: vec![
            member(&mut client, 7, "Luzal", true, 300),
            member(&mut client, 8, "Kain", false, 120),
            member(&mut client, 9, "Sylph", false, 600),
        ],
        show_buffs: true,
        loot_rule: 0,
    };
    let (x, y) = PARTY_POS;
    let mut list = DrawList::default();
    paint_party(&mut list, x, y, PARTY_WIDTH, &view, &font, 1.0);
    let rgba = render_list(&mut client, &mut headless, &list);
    save("ui-party.png", PARTY_CANVAS, &rgba);

    let at = |dx: i32, dy: i32| pixel_in(&rgba, width, (x + dx) as u32, (y + dy) as u32);
    assert_eq!(pixel_in(&rgba, width, 2, 2), CLEAR_RGBA);
    assert_ne!(at(6, 60), CLEAR_RGBA, "pegador");
    assert_ne!(at(12 + PARTY_BAR_X + 10, PARTY_HP_BAR_Y + 2), CLEAR_RGBA, "barra de HP do líder");
    assert_ne!(at(12 + PARTY_BAR_X + 10, 2 * CELL_STEP + PARTY_HP_BAR_Y + 2), CLEAR_RGBA, "barra de HP do terceiro");
    assert_ne!(at(12 + 161 + 8, 12), CLEAR_RGBA, "buff à direita da célula");
}

#[test]
#[ignore]
fn captures_the_loading_screen_with_the_client_texture() {
    let Some(mut client) = client() else { return };
    let (width, height) = LOADING_CANVAS;
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let localization = parse_ini(&client.read_system_text("localization.ini").unwrap());
    let texture = loading_texture(&localization).unwrap();
    assert_eq!(texture, "L2Font-e.loading02-e");
    let raw = client.catalog().texture_raw(&texture).unwrap();
    assert_eq!((raw.mips[0].width, raw.mips[0].height), (1024, 1024));
    let mut list = DrawList::default();
    paint_loading(&mut list, 0, 0, width as i32, height as i32, &texture);
    let rgba = render_list(&mut client, &mut headless, &list);
    save("ui-loading.png", LOADING_CANVAS, &rgba);
    assert_ne!(pixel_in(&rgba, width, width / 2, height / 2), CLEAR_RGBA);
    assert_ne!(pixel_in(&rgba, width, 2, 2), CLEAR_RGBA);
}

#[test]
#[ignore]
fn captures_the_setup_gauges_with_the_client_textures() {
    let Some(mut client) = client() else { return };
    let (width, height) = GAUGE_CANVAS;
    let Ok(mut headless) = Headless::new(width, height) else { return };
    for name in GAUGE_BACK.iter().chain(GAUGE_FILL.iter()) {
        assert!(client.catalog().texture_raw(name).is_some(), "{name}");
    }
    let gauges = [
        GaugeView { kind: 0, remaining_ms: 4000, max_ms: 5000 },
        GaugeView { kind: 1, remaining_ms: 3000, max_ms: 5000 },
        GaugeView { kind: 2, remaining_ms: 2000, max_ms: 5000 },
        GaugeView { kind: 3, remaining_ms: 1000, max_ms: 5000 },
    ];
    let mut list = DrawList::default();
    paint_gauges(&mut list, 70, 76, 8, &gauges);
    let rgba = render_list(&mut client, &mut headless, &list);
    save("ui-gauge.png", GAUGE_CANVAS, &rgba);
    assert_ne!(pixel_in(&rgba, width, 30, 76 - 44 + 3), CLEAR_RGBA, "primeira barra");
    assert_ne!(pixel_in(&rgba, width, 30, 76 - 44 - 3 * 8 + 3), CLEAR_RGBA, "quarta barra");
}

#[test]
#[ignore]
fn captures_the_login_screen_with_the_logos_from_localization() {
    let Some(mut client) = client() else { return };
    let canvas = (1024u32, 768u32);
    let Ok(mut headless) = Headless::new(canvas.0, canvas.1) else { return };
    let font = small_font(&mut client);
    let language = client.client_language();
    let (logo, mini_logo) =
        killer_assets::language::logo_textures(&client.read_system_text("localization.ini").unwrap_or_default(), &language.country);
    let mut screen = killer_ui::lobby_login::LoginScreen::new(
        (canvas.0 as i32, canvas.1 as i32),
        killer_ui::lobby_login::LoginLabels::from_sysstrings(|id| Some(format!("t{id}"))),
    );
    screen.logo = logo;
    screen.mini_logo = mini_logo;
    let mut list = DrawList::default();
    screen.paint(&mut list, &font);
    let rgba = render_list(&mut client, &mut headless, &list);
    save("lobby-login-logos.png", canvas, &rgba);
    let lit = |x0: u32, x1: u32, y0: u32, y1: u32| (y0..y1).any(|y| (x0..x1).any(|x| pixel_in(&rgba, canvas.0, x, y)[0] > 200));
    assert!(lit(0, 232, 0, 120), "logo pequeno");
    assert!(lit(384, 640, 40, 290), "logo grande");
}

#[test]
#[ignore]
fn captures_the_create_screen_with_a_dark_elf_without_class_type() {
    let Some(mut client) = client() else { return };
    let canvas = (1024u32, 768u32);
    let Ok(mut headless) = Headless::new(canvas.0, canvas.1) else { return };
    let font = small_font(&mut client);
    let mut screen = killer_ui::lobby_create::CreateScreen::new(
        (canvas.0 as i32, canvas.1 as i32),
        killer_ui::lobby_create::CreateTexts::default(),
    );
    screen.choose_race(2);
    let mut list = DrawList::default();
    screen.paint(&mut list, &font);
    let rgba = render_list(&mut client, &mut headless, &list);
    save("lobby-create-race.png", canvas, &rgba);
    assert_eq!(screen.race(), Some(2));
}

#[test]
#[ignore]
fn captures_the_skill_window_with_the_cool_time_frames_and_the_end_flash() {
    use killer_ui::cooltime::{CoolFrame, END_FRAMES, END_PREFIX, ROTATION_FRAMES, ROTATION_PREFIX};
    use killer_ui::skills::{SkillView, SkillsView};
    let Some(mut client) = client() else { return };
    for index in 0..ROTATION_FRAMES {
        assert!(client.catalog().texture_raw(&format!("{ROTATION_PREFIX}{index}")).is_some(), "quadro {index}");
    }
    for index in 1..=END_FRAMES {
        assert!(client.catalog().texture_raw(&format!("{END_PREFIX}{index}")).is_some(), "fim {index}");
    }
    let canvas = (256u32 * 2 + 20, 345u32);
    let Ok(mut headless) = Headless::new(canvas.0, canvas.1) else { return };
    let font = small_font(&mut client);
    let skill = |client: &mut Client, id: u32, passive: bool| SkillView {
        id: id as i32,
        level: 1,
        passive,
        icon: skill_icon(client, id, 1),
        ..Default::default()
    };
    let mut view = SkillsView::with_sysstrings(|id| Some(id.to_string()));
    let mut skills = vec![
        skill(&mut client, 1204, false),
        skill(&mut client, 1040, false),
        skill(&mut client, 1069, false),
        skill(&mut client, 1045, false),
        skill(&mut client, 1068, false),
    ];
    skills[0].cool = Some(CoolFrame::Running(120));
    skills[1].cool = Some(CoolFrame::Running(270));
    skills[2].cool = Some(CoolFrame::Flash(3));
    skills[3].toggled = true;
    view.skills = skills.clone();
    view.selected = Some(skills[4].id);
    view.pressed = true;
    view.update_rows();

    let mut before = SkillsView::with_sysstrings(|id| Some(id.to_string()));
    before.skills = skills.iter().map(|s| SkillView { cool: None, ..s.clone() }).collect();
    before.selected = Some(skills[4].id);
    before.hovered = Some(4);

    let mut list = DrawList::default();
    before.paint(&mut list, 5, 5, &font);
    for (index, fraction) in [(0usize, 0.5f32), (1, 0.25), (2, 0.7)] {
        let (sx, sy) = killer_ui::skills::slot_pos(index);
        let (x, y) = (5 + killer_ui::skills::LIST_RECT.x + sx, 5 + killer_ui::skills::LIST_RECT.y + sy);
        let height = (32.0 * fraction).round() as i32;
        list.draw_fill(x, y + 32 - height, 32, height, Argb(0xFF008080));
    }
    view.paint(&mut list, 266, 5, &font);
    let rgba = render_list(&mut client, &mut headless, &list);
    save("ui-skills-cooltime.png", canvas, &rgba);
    assert_ne!(pixel_in(&rgba, canvas.0, 266 + 9 + 16, 5 + 40 + 16), CLEAR_RGBA);
}
