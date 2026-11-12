use glam::vec3;
use killer_assets::client::Client;
use killer_assets::textures::{BankSource, TextureBank, write_png};
use killer_render::camera::Camera;
use killer_render::environment::Environment;
use killer_render::headless::{CLEAR_RGBA, Headless};
use killer_ui::draw::DrawList;
use killer_ui::font::{BitmapFont, Glyphs};
use killer_ui::keys::MainTab;
use killer_ui::mainwnd::{MAIN_SIZE, MainWndView, PAGE_RECT};
use killer_ui::minimap::{self, MINIMAP_SIZE, MinimapView};
use killer_ui::pledge::{MemberRow, PledgeRights, PledgeView, button_rect};
use killer_ui::quest::{QuestNode, QuestStep, QuestView, states_of};
use killer_ui::target::{TARGET_SIZE, TargetKind, TargetView, info_labels, paint_target};
use killer_ui::tooltip::{Tooltip, TooltipLine, VALUE_COLOR};
use l2_dat::{questname, sysstring};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const SMALL_FONT: &str = "L2Font-e.SmallFont-e";
const MARGIN: i32 = 10;

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

fn open() -> Option<(Client, BitmapFont, HashMap<u32, String>)> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    let mut client = Client::open(&root).ok()?;
    let font = small_font(&mut client);
    let strings = sysstrings(&mut client);
    Some((client, font, strings))
}

#[test]
#[ignore]
fn captures_the_pledge_page_by_rights_and_residence() {
    let Some((mut client, font, strings)) = open() else { return };
    let (w, h) = MAIN_SIZE;
    let (width, height) = ((w + 2 * MARGIN) as u32, (h + 2 * MARGIN) as u32);
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let lookup = |id: u32| strings.get(&id).cloned();
    let mut main = MainWndView::with_sysstrings(lookup);
    main.tab = MainTab::Clan;
    let mut capture = |rights: PledgeRights, castle: Option<&str>, hideout: Option<&str>, name: &str| {
        let mut pledge = PledgeView::with_sysstrings(lookup);
        pledge.name = Some("Killer".into());
        pledge.leader = "Luzal".into();
        pledge.level = 5;
        pledge.castle = castle.map(str::to_owned);
        pledge.hideout = hideout.map(str::to_owned);
        pledge.members = (0..9)
            .map(|i| MemberRow { name: format!("Membro{i}"), level: 20 + i, class_id: i, online: i % 3 != 0, object_id: 0 })
            .collect();
        pledge.selected = Some(1);
        pledge.show(&rights);
        let mut list = DrawList::default();
        main.paint_chrome(&mut list, MARGIN, MARGIN, &font, 1.0);
        pledge.paint(&mut list, MARGIN + PAGE_RECT.x, MARGIN + PAGE_RECT.y, &font);
        render(&mut client, &mut headless, &list, name, width, height)
    };
    let member = PledgeRights { in_clan: true, ..Default::default() };
    let leader = capture(PledgeRights { leader: true, in_clan: true, ..Default::default() }, Some("Gludio Castle"), Some("Hideout"), "ui-pledge-leader.png");
    let plain = capture(member, None, None, "ui-pledge-member.png");
    let invite = button_rect(0);
    let probe = |rgba: &[u8], dx: i32| pixel_in(rgba, width, MARGIN + PAGE_RECT.x + invite.x + dx, MARGIN + PAGE_RECT.y + invite.y + 11);
    let differing = (4..invite.w - 4).filter(|dx| probe(&leader, *dx) != probe(&plain, *dx)).count();
    assert!(differing > 0, "Invite habilitado (líder) e desabilitado (membro) pintam igual");
    let leave = button_rect(8);
    let same = (4..leave.w - 4).filter(|dx| pixel_in(&leader, width, MARGIN + PAGE_RECT.x + leave.x + dx, MARGIN + PAGE_RECT.y + leave.y + 11) != pixel_in(&plain, width, MARGIN + PAGE_RECT.x + leave.x + dx, MARGIN + PAGE_RECT.y + leave.y + 11)).count();
    assert!(same > 0, "Leave desabilitado (líder) e habilitado (membro) pintam igual");
    let castle_only = capture(member, Some("Gludio Castle"), None, "ui-pledge-castle.png");
    let hideout_only = capture(member, None, Some("Hideout"), "ui-pledge-hideout.png");
    let base = |rgba: &[u8]| (60..180).filter(|dx| pixel_in(rgba, width, MARGIN + PAGE_RECT.x + dx, MARGIN + PAGE_RECT.y + 42 + 6) != pixel_in(&plain, width, MARGIN + PAGE_RECT.x + dx, MARGIN + PAGE_RECT.y + 42 + 6)).count();
    assert!(base(&castle_only) > 0 && base(&hideout_only) > 0 && base(&leader) > 0);
}

#[test]
#[ignore]
fn captures_the_minimap_centred_on_giran() {
    let Some((mut client, font, strings)) = open() else { return };
    let (w, h) = MINIMAP_SIZE;
    let (width, height) = ((w + 2 * MARGIN) as u32, (h + 2 * MARGIN) as u32);
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let mut view = MinimapView::with_sysstrings(|id| strings.get(&id).cloned());
    let player = minimap::player_map_pos(83400.0, 148000.0, -3400.0);
    view.player = Some(player);
    view.targets = vec![(84500, 149200)];
    view.party = vec![minimap::PartyMark { id: 1, world: (82000, 147000, 0), in_window: true, name: String::new() }];
    view.center_on(player);
    view.zone = "Town of Giran".into();
    view.period = 1;
    view.hour = Some(13.5);
    let mut list = DrawList::default();
    view.paint(&mut list, MARGIN, MARGIN, &font, 1.0);
    let rgba = render(&mut client, &mut headless, &list, "ui-minimap.png", width, height);
    assert_eq!(pixel_in(&rgba, width, 2, 2), CLEAR_RGBA);
    let cover = minimap::COVER_RECT;
    assert_ne!(pixel_in(&rgba, width, MARGIN + cover.x + 40, MARGIN + cover.y + 40), CLEAR_RGBA, "tile do mapa");
    let bar = minimap::BOTTOM_BAR_RECT;
    assert_ne!(pixel_in(&rgba, width, MARGIN + cover.x + 100, MARGIN + cover.y + bar.y + 5), CLEAR_RGBA, "faixa MapShadow");
}

#[test]
#[ignore]
fn captures_the_pledge_and_quest_pages() {
    let Some((mut client, font, strings)) = open() else { return };
    let (w, h) = MAIN_SIZE;
    let (width, height) = ((w + 2 * MARGIN) as u32, (h + 2 * MARGIN) as u32);
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let lookup = |id: u32| strings.get(&id).cloned();
    let mut main = MainWndView::with_sysstrings(lookup);
    main.tab = MainTab::Clan;
    let mut pledge = PledgeView::with_sysstrings(lookup);
    pledge.name = Some("Killer".into());
    pledge.leader = "Luzal".into();
    pledge.level = 3;
    pledge.members = vec![
        MemberRow { name: "Luzal".into(), level: 40, class_id: 2, online: true, object_id: 0 },
        MemberRow { name: "Kerchief".into(), level: 31, class_id: 11, online: false, object_id: 0 },
        MemberRow { name: "Bremmer".into(), level: 25, class_id: 22, online: true, object_id: 0 },
    ];
    pledge.selected = Some(0);
    let mut list = DrawList::default();
    main.paint_chrome(&mut list, MARGIN, MARGIN, &font, 1.0);
    pledge.paint(&mut list, MARGIN + PAGE_RECT.x, MARGIN + PAGE_RECT.y, &font);
    let rgba = render(&mut client, &mut headless, &list, "ui-pledge.png", width, height);
    assert_ne!(pixel_in(&rgba, width, MARGIN + w / 2, MARGIN + PAGE_RECT.y + 100), CLEAR_RGBA, "BloodHood_Back");
    let table = questname::by_id_state(questname::read(&client.read_system_file("questname-e.dat").unwrap()).unwrap());
    let mut quest = QuestView::with_sysstrings(lookup);
    for (quest_id, cond, expanded) in [(1, 2, true), (2, 1, false)] {
        let steps: Vec<QuestStep> = states_of(cond)
            .into_iter()
            .filter_map(|st| table.get(&(quest_id, st)).map(|q| QuestStep { state: st, sub_name: q.sub_name.clone(), description: q.description.clone(), ..Default::default() }))
            .collect();
        let title = table.get(&(quest_id, 1)).map(|q| q.title.clone()).unwrap_or_default();
        assert!(!title.is_empty(), "quest {quest_id} sem título no questname-e.dat");
        quest.quests.push(QuestNode { quest_id, title, steps, expanded, ..Default::default() });
    }
    quest.selected = Some(0);
    main.tab = MainTab::Quests;
    let mut list = DrawList::default();
    main.paint_chrome(&mut list, MARGIN, MARGIN, &font, 1.0);
    quest.paint(&mut list, MARGIN + PAGE_RECT.x, MARGIN + PAGE_RECT.y, &font);
    let rgba = render(&mut client, &mut headless, &list, "ui-quest.png", width, height);
    assert_ne!(pixel_in(&rgba, width, MARGIN + w / 2, MARGIN + PAGE_RECT.y + 100), CLEAR_RGBA, "QuestWndBack");
}

#[test]
#[ignore]
fn captures_the_expanded_target_and_a_tooltip() {
    let Some((mut client, font, strings)) = open() else { return };
    let (w, _) = TARGET_SIZE;
    let (width, height) = ((w + 2 * MARGIN) as u32, (killer_ui::target::TARGET_EXPANDED_HEIGHT + 2 * MARGIN) as u32);
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let mut view = TargetView::new("Luzal".into(), TargetKind::Player, Some((400, 900)), Some((100, 300)));
    view.expanded = true;
    view.clan = Some("Killer".into());
    view.ally = Some("Unk".into());
    view.labels = info_labels(|id| strings.get(&id).cloned());
    let mut list = DrawList::default();
    paint_target(&mut list, MARGIN, MARGIN, &view, &font, 1.0);
    let rgba = render(&mut client, &mut headless, &list, "ui-target-info.png", width, height);
    assert_ne!(pixel_in(&rgba, width, MARGIN + 30, MARGIN + 60), CLEAR_RGBA, "SmallWindow2_Back");
    let tip = Tooltip {
        enchant: 3,
        name: "Short Sword".into(),
        grade: None,
        count: None,
        level: None,
        add_name: String::new(),
        lines: vec![
            TooltipLine { label: "P. Atk.".into(), value: "8".into(), color: VALUE_COLOR },
            TooltipLine { label: strings.get(&52).cloned().unwrap_or_default(), value: "1,600".into(), color: VALUE_COLOR },
        ],
        description: "A basic sword. Nothing fancy.".into(),
        ..Default::default()
    };
    let (tw, th) = tip.size(&font);
    let (width, height) = ((tw + 2 * MARGIN) as u32, (th + 2 * MARGIN) as u32);
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let mut list = DrawList::default();
    tip.paint(&mut list, MARGIN, MARGIN, &font);
    let rgba = render(&mut client, &mut headless, &list, "ui-tooltip.png", width, height);
    assert_ne!(pixel_in(&rgba, width, MARGIN + tw / 2, MARGIN + th / 2), CLEAR_RGBA, "Tooltip5");
}
