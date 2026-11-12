use glam::vec3;
use killer_assets::client::Client;
use killer_assets::textures::{BankSource, TextureBank, TextureSource, write_png};
use killer_render::camera::Camera;
use killer_render::environment::Environment;
use killer_render::headless::{CLEAR_RGBA, Headless};
use killer_ui::chat::{
    CHAT_HEADER, CHAT_LINE, ChatEntry, ChatView, INPUT_BOTTOM_INSET, INPUT_TEXT_INSET, INPUT_X, InputKey, LINE_TEXT_X,
    LINE_TEXT_Y, TAB_SIZE, TAB_X,
};
use killer_ui::color::{CHAT_SYSTEM, TEXT};
use killer_ui::draw::DrawList;
use killer_ui::font::{BitmapFont, Glyphs};
use killer_ui::frame::{BodyKind, FrameCtrl, TITLE_HEIGHT, paint_body};
use killer_ui::draw::LINE_TEXTURE;
use killer_ui::html::LINK_COLOR;
use killer_ui::ini::{ChatFilter, parse_ini};
use killer_ui::npc_viewer::{CONTENT_POS, NPC_HTML_BACK, NPC_VIEWER_SIZE, NpcViewer};
use killer_ui::status::{
    BAR_HEIGHT, BAR_WIDTH, BAR_X, HP_BAR_Y, LEVEL_BACK_POS, LEVEL_BACK_SIZE, NAME_POS, STATUS_SIZE, StatusView,
    fill_width, paint_status,
};
use std::path::{Path, PathBuf};

const WIDTH: u32 = 400;
const HEIGHT: u32 = 160;
const SMALL_FONT: &str = "L2Font-e.SmallFont-e";
const SMALL_FONT_HEIGHT: u32 = 13;
const FRAME: (i32, i32, i32, i32) = (50, 30, 300, 84);
const STATUS_CANVAS: (u32, u32) = (240, 110);
const STATUS_POS: (i32, i32) = (19, 13);
const CHAT_CANVAS: (u32, u32) = (360, 200);
const CHAT_POS: (i32, i32) = (6, 6);
const CHAT_HEIGHT: i32 = 187;
const NPC_CANVAS: (u32, u32) = (330, 421);
const NPC_POS: (i32, i32) = (10, 10);
const NPC_HTML: &str = include_str!("../../killer-ui/tests/fixtures/npc-7829.htm");

fn pixel(rgba: &[u8], x: u32, y: u32) -> [u8; 4] {
    pixel_in(rgba, WIDTH, x, y)
}

fn pixel_in(rgba: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let p = &rgba[((y * width + x) * 4) as usize..][..4];
    [p[0], p[1], p[2], p[3]]
}

fn small_font(client: &mut Client) -> BitmapFont {
    let glyphs = Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap();
    BitmapFont::new(SMALL_FONT.into(), glyphs)
}

fn render_list(client: &mut Client, headless: &mut Headless, list: &DrawList) -> Vec<u8> {
    let mut bank = TextureBank::new();
    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
    headless.renderer_mut().set_ui(list, &mut source);
    let camera =
        Camera { eye: vec3(0.0, -300.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    headless.render(&camera, &Environment::default()).unwrap()
}

#[test]
#[ignore]
fn captures_option_100cb4c0_three_pages_with_original_assets() {
    use killer_ui::option::{OptionView, SIZE, TABS_RECT};
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let font = small_font(&mut client);
    let strings: std::collections::HashMap<_, _> = l2_dat::sysstring::read(&client.read_system_file("sysstring-e.dat").unwrap())
        .unwrap().into_iter().map(|row| (row.id, row.text)).collect();
    let mut view = OptionView::with_god(|id| strings.get(&id).cloned(), false, 0);
    assert_eq!(view.tabs.buttons.len(), 3);
    let (width, height) = (SIZE.0 as u32 * 3, SIZE.1 as u32);
    let mut headless = Headless::new(width, height).unwrap();
    let mut list = DrawList::default();
    for page in 0..3 {
        assert_eq!(view.hit(TABS_RECT.x + page as i32 * 75 + 1, TABS_RECT.y + 1), Some(killer_ui::option::OptionHit::Tab(page)));
        view.press_tab(page);
        view.paint(&mut list, page as i32 * SIZE.0, 0, &font, 1.0);
    }
    let mut bank = TextureBank::new();
    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
    headless.renderer_mut().set_ui(&list, &mut source);
    assert!(bank.missing().collect::<Vec<_>>().is_empty());
    let camera = Camera { eye: vec3(0.0, -300.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    let rgba = headless.render(&camera, &Environment::default()).unwrap();
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-option-pages.png");
    write_png(&out, width, height, &rgba).unwrap();
    for page in 0..3 {
        assert_ne!(pixel_in(&rgba, width, page * SIZE.0 as u32 + 20, 27), CLEAR_RGBA);
    }
    view.set_font(&font);
    view.press_tab(killer_ui::option::TAB_AUDIO);
    let key = killer_ui::option::OptionCombo { page: killer_ui::option::TAB_AUDIO, index: 0 };
    let arrow = view.combo(key).unwrap().arrow_rect();
    assert_eq!(view.hit(arrow.x + 1, arrow.y + 49), Some(killer_ui::option::OptionHit::ComboArrow(key)));
    view.combo_mouse_down(key, true);
    view.combo_mouse_up(Some(killer_ui::option::OptionHit::ComboArrow(key)));
    view.combo_mouse_move(110, 48 + 11 + 17 + 3 * 17 + 1);
    let mut open = DrawList::default();
    view.paint(&mut open, 0, 0, &font, 1.0);
    view.combo_mouse_down(key, false);
    assert_eq!(view.combo(key).unwrap().selected, 3);
    view.paint(&mut open, SIZE.0, 0, &font, 1.0);
    view.combo_mouse_down(key, true);
    view.paint(&mut open, SIZE.0 * 2, 0, &font, 1.0);
    let rgba = render_list(&mut client, &mut headless, &open);
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-option-combos.png");
    write_png(&out, width, height, &rgba).unwrap();
}

#[test]
#[ignore]
fn captures_nccombobox_10005f50_closed_disabled_and_open_with_real_textures() {
    use killer_ui::combo::{ComboBox, DISABLED, NORMAL, SELECT, SELECT_END, ARROW, ARROW_DOWN};
    use killer_ui::window::Rect;
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let font = small_font(&mut client);
    let mut headless = Headless::new(WIDTH, HEIGHT).unwrap();
    let text_height = font.glyph('A').unwrap().height as i32;
    let mut list = DrawList::default();
    for (index, (enabled, open)) in [(true, false), (false, false), (true, true), (true, true)].into_iter().enumerate() {
        let mut combo = ComboBox::new(Rect::new(20 + (index % 2) as i32 * 180, 15 + (index / 2) as i32 * 50, 125, 17),
            vec!["One".into(), "Two".into(), "Three".into()], text_height);
        combo.enabled = enabled;
        combo.selected = 1;
        if open {
            combo.open_list();
            combo.hovered = 1;
        }
        combo.paint(&mut list, (0, 0), &font, true, index == 3);
    }
    let mut bank = TextureBank::new();
    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
    for name in NORMAL.into_iter().chain(DISABLED).chain([SELECT, SELECT_END, ARROW, ARROW_DOWN]) {
        let tex = source.texture(name);
        assert!(tex.width > 0 && tex.height > 0, "{name}");
    }
    headless.renderer_mut().set_ui(&list, &mut source);
    assert!(bank.missing().collect::<Vec<_>>().is_empty());
    let camera = Camera { eye: vec3(0.0, -300.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    let rgba = headless.render(&camera, &Environment::default()).unwrap();
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-combobox.png");
    write_png(&out, WIDTH, HEIGHT, &rgba).unwrap();
    assert_eq!(pixel(&rgba, 2, 2), CLEAR_RGBA);
    assert_ne!(pixel(&rgba, 22, 17), CLEAR_RGBA);
    assert_ne!(pixel(&rgba, 22, 100), CLEAR_RGBA);
    assert_eq!(pixel(&rgba, 146, 100), CLEAR_RGBA);
}

#[test]
#[ignore]
fn captures_a_title_frame_with_the_small_font() {
    let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
    let mut client = Client::open(&root).unwrap();
    let Ok(mut headless) = Headless::new(WIDTH, HEIGHT) else { return };
    let glyphs = Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap();
    let font = BitmapFont { atlas: SMALL_FONT.into(), glyphs, line_height: SMALL_FONT_HEIGHT };
    let (x, y, w, h) = FRAME;
    let mut list = DrawList::default();
    FrameCtrl::titled("Killer", true).paint(&mut list, x, y, w, h, &font, 1.0);
    paint_body(&mut list, BodyKind::Small2, x, y + TITLE_HEIGHT, w, h, 1.0);
    list.draw_text(x + 20, y + TITLE_HEIGHT + 10, TEXT, "Lineage II C4", &font);

    let mut bank = TextureBank::new();
    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
    let atlas = source.texture(SMALL_FONT);
    assert_eq!((atlas.width, atlas.height), (1024, 128));
    assert!(atlas.mips[0].chunks(4).any(|p| p[3] == 0), "atlas da fonte veio opaco");
    headless.renderer_mut().set_ui(&list, &mut source);
    let camera =
        Camera { eye: vec3(0.0, -300.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    let rgba = headless.render(&camera, &Environment::default()).unwrap();

    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-frame.png");
    write_png(&out, WIDTH, HEIGHT, &rgba).unwrap();

    assert_eq!(pixel(&rgba, 10, 10), CLEAR_RGBA);
    assert_ne!(pixel(&rgba, (x + 8) as u32, (y + 10) as u32), CLEAR_RGBA);
    assert_ne!(pixel(&rgba, (x + w / 2) as u32, (y + 10) as u32), CLEAR_RGBA);
    assert_ne!(pixel(&rgba, (x + w / 2) as u32, (y + TITLE_HEIGHT + 40) as u32), CLEAR_RGBA);
    let title = (0..font.measure("Killer"))
        .flat_map(|dx| (0..SMALL_FONT_HEIGHT).map(move |dy| (dx, dy)))
        .map(|(dx, dy)| pixel(&rgba, (x + 20) as u32 + dx, (y + 3) as u32 + dy));
    assert!(title.clone().any(|p| p[0] > 150), "título ilegível");
    assert!(title.clone().any(|p| p[0] < 100), "título sem fundo");
}

#[test]
#[ignore]
fn captures_the_status_window_with_the_client_textures() {
    let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
    let mut client = Client::open(&root).unwrap();
    let (width, height) = STATUS_CANVAS;
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = small_font(&mut client);
    let view = StatusView {
        name: "Killer".into(),
        level: 52,
        hp: (400, 900),
        mp: (200, 300),
        cp: (100, 100),
        exp: 52262490,
        regen: 0.0,
    };
    let (x, y) = STATUS_POS;
    let mut list = DrawList::default();
    paint_status(&mut list, x, y, &view, &font, 1.0);
    let rgba = render_list(&mut client, &mut headless, &list);

    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-status.png");
    write_png(&out, width, height, &rgba).unwrap();

    let at = |dx: i32, dy: i32| pixel_in(&rgba, width, (x + dx) as u32, (y + dy) as u32);
    let (w, h) = STATUS_SIZE;
    assert_eq!(pixel_in(&rgba, width, 2, 2), CLEAR_RGBA);
    assert_eq!(pixel_in(&rgba, width, (x + w + 4) as u32, (y + h / 2) as u32), CLEAR_RGBA);
    assert_ne!(at(6, h / 2), CLEAR_RGBA, "pegador vertical");
    assert_ne!(at(w / 2, 4), CLEAR_RGBA, "corpo Smallwindow2");
    assert_ne!(at(LEVEL_BACK_POS.0 + LEVEL_BACK_SIZE.0 / 2, LEVEL_BACK_POS.1 + 2), CLEAR_RGBA, "ps_levelback");
    let hp_fill = fill_width(view.hp, BAR_WIDTH);
    assert_eq!(hp_fill, 66);
    let filled = at(BAR_X + hp_fill / 2, HP_BAR_Y + BAR_HEIGHT / 2);
    let empty = at(BAR_X + hp_fill + 40, HP_BAR_Y + BAR_HEIGHT / 2);
    assert_ne!(filled, empty, "preenchimento da barra de HP igual ao fundo");
    assert!(filled[0] > empty[0], "barra de HP não é mais vermelha que o fundo");
    let name = (0..font.measure("Killer") as i32)
        .flat_map(|dx| (0..font.line_height as i32).map(move |dy| (dx, dy)))
        .map(|(dx, dy)| at(NAME_POS.0 + dx, NAME_POS.1 + dy));
    assert!(name.clone().any(|p| p[0] > 150), "nome ilegível");
    assert!(name.clone().any(|p| p[0] < 100), "nome sem fundo");
}

#[test]
#[ignore]
fn captures_the_chat_window_with_the_client_textures() {
    let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
    let mut client = Client::open(&root).unwrap();
    let (width, height) = CHAT_CANVAS;
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = small_font(&mut client);
    let filter = ChatFilter::from_ini(&parse_ini(&client.read_system_text("chatfilter.ini").unwrap()));
    let mut view = ChatView::new(CHAT_HEIGHT, filter);
    view.focused = true;
    for c in "hello giran".chars() {
        view.input.key(InputKey::Char(c));
    }
    let entry = |kind: i32, name: &str, text: &str| ChatEntry::new(kind, name, text);
    let lines = [
        entry(0, "Killer", "bom dia Giran"),
        entry(1, "Luzal", "WTS Sword of Limit 2kk"),
        entry(2, "Bob", "vem pra party?"),
        entry(3, "Alice", "boss em 5 min"),
        entry(CHAT_SYSTEM, "", "Welcome to the World of Lineage II."),
    ];
    let (x, y) = CHAT_POS;
    let mut list = DrawList::default();
    view.paint(&mut list, x, y, &lines, &font, 1.0);
    let rgba = render_list(&mut client, &mut headless, &list);

    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-chat.png");
    write_png(&out, width, height, &rgba).unwrap();

    let at = |dx: i32, dy: i32| pixel_in(&rgba, width, (x + dx) as u32, (y + dy) as u32);
    let (w, h) = view.size();
    assert_eq!(h, CHAT_HEIGHT);
    assert_eq!(pixel_in(&rgba, width, 1, 1), CLEAR_RGBA);
    assert_eq!(pixel_in(&rgba, width, (x + w + 4) as u32, (y + h / 2) as u32), CLEAR_RGBA);
    assert_eq!(pixel_in(&rgba, width, (x + w / 2) as u32, (y + h + 4) as u32), CLEAR_RGBA);
    assert_ne!(at(w / 2, CHAT_HEADER / 2), CLEAR_RGBA, "cabeçalho Chatting_Back2");
    assert_ne!(at(w - 20, CHAT_HEADER + CHAT_LINE * 3 + 7), CLEAR_RGBA, "faixa Chatting_Back3");
    let footer_y = h - 46;
    assert_ne!(at(w / 2, footer_y + 40), CLEAR_RGBA, "rodapé Chatting_Back1");
    assert_ne!(at(TAB_X[0] + TAB_SIZE.0 / 2, footer_y + TAB_SIZE.1 / 2), CLEAR_RGBA, "aba Chatting_Tab1");

    let row = |i: usize, text: &str| {
        let row_y = CHAT_LINE * i as i32 + LINE_TEXT_Y;
        (0..font.measure(text) as i32)
            .flat_map(|dx| (0..font.line_height as i32).map(move |dy| (dx, dy)))
            .map(|(dx, dy)| at(LINE_TEXT_X + dx, row_y + dy))
            .collect::<Vec<_>>()
    };
    let strongest = |pixels: &[[u8; 4]], channel: usize| pixels.iter().copied().max_by_key(|p| p[channel]).unwrap();
    let all = strongest(&row(0, "Killer: bom dia Giran"), 0);
    assert!(all[0] > 150 && all[1] > 150 && all[2] > 150, "linha ALL sem texto claro: {all:?}");
    let shout = strongest(&row(1, "Luzal: WTS Sword of Limit 2kk"), 0);
    assert!(shout[0] > 150 && shout[2] < 100, "linha SHOUT não é laranja: {shout:?}");
    let tell = strongest(&row(2, "Bob: vem pra party?"), 2);
    assert!(tell[0] > 150 && tell[1] < 100 && tell[2] > 150, "linha TELL não é magenta: {tell:?}");
    let party = strongest(&row(3, "Alice: boss em 5 min"), 1);
    assert!(party[1] > 150 && party[0] < 100 && party[2] < 100, "linha PARTY não é verde: {party:?}");
    let input = (0..font.measure("hello giran|") as i32)
        .flat_map(|dx| (0..font.line_height as i32).map(move |dy| (dx, dy)))
        .map(|(dx, dy)| at(INPUT_X + INPUT_TEXT_INSET + dx, h - INPUT_BOTTOM_INSET + INPUT_TEXT_INSET + dy));
    assert!(input.clone().any(|p| p[0] > 150), "entrada ilegível");
    assert!(input.clone().any(|p| p[0] < 100), "entrada sem fundo");
}

#[test]
#[ignore]
fn captures_the_npc_dialog_with_the_client_textures() {
    let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
    let mut client = Client::open(&root).unwrap();
    let (width, height) = NPC_CANVAS;
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = small_font(&mut client);
    assert_eq!(client.catalog().texture_size(NPC_HTML_BACK), Some([512.0, 256.0]));
    assert!(client.catalog().texture_size(LINE_TEXTURE).is_some(), "{LINE_TEXTURE} ausente");
    let html = NPC_HTML.replace("%objectId%", "268476000");
    let viewer = NpcViewer::new("Chat", &html, &font);
    let (x, y) = NPC_POS;
    let mut list = DrawList::default();
    viewer.paint(&mut list, x, y, &font, 1.0);
    let rgba = render_list(&mut client, &mut headless, &list);

    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-npc.png");
    write_png(&out, width, height, &rgba).unwrap();

    let at = |dx: i32, dy: i32| pixel_in(&rgba, width, (x + dx) as u32, (y + dy) as u32);
    let (w, h) = NPC_VIEWER_SIZE;
    assert_eq!(pixel_in(&rgba, width, 1, 1), CLEAR_RGBA);
    assert_eq!(pixel_in(&rgba, width, (x + w + 4) as u32, (y + h / 2) as u32), CLEAR_RGBA);
    assert_ne!(at(w / 2, 10), CLEAR_RGBA, "moldura de título");
    assert_ne!(at(w / 2, 200), CLEAR_RGBA, "NPCHtml_BACK");
    assert_ne!(at(w / 2, h - 5), CLEAR_RGBA, "NPCHtml_BACK até o pé");
    let row = |line_y: i32, text: &str| {
        (0..font.measure(text) as i32)
            .flat_map(|dx| (0..font.line_height as i32).map(move |dy| (dx, dy)))
            .map(|(dx, dy)| at(CONTENT_POS.0 + dx, CONTENT_POS.1 + line_y + dy))
            .collect::<Vec<_>>()
    };
    let strongest = |pixels: &[[u8; 4]], channel: usize| pixels.iter().copied().max_by_key(|p| p[channel]).unwrap();
    let name = strongest(&row(0, "Pet Manager Cooper:"), 0);
    assert!(name[0] > 150 && name[1] > 150 && name[2] > 150, "nome do NPC ilegível: {name:?}");
    let sell = viewer.lines.iter().find(|l| l.spans[0].text == "Sell item").unwrap();
    let link = strongest(&row(sell.y, "Sell item"), 2);
    let expected = LINK_COLOR.rgba().map(|c| (c * 255.0) as i32);
    assert!((i32::from(link[2]) - expected[2]).abs() < 20 && link[0] < 160, "link não é azul: {link:?}");
    let underline = at(CONTENT_POS.0 + 3, CONTENT_POS.1 + sell.y + font.line_height as i32);
    assert!(underline[2] > 200 && underline[0] < 160, "sublinhado do link: {underline:?}");
    let speech: Vec<[u8; 4]> = row(28, "I'm glad you came");
    assert!(strongest(&speech, 0)[0] > 150, "fala do NPC ilegível");
}

#[test]
#[ignore]
fn captures_a_detached_chat_tab_with_its_floating_list() {
    let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
    let mut client = Client::open(&root).unwrap();
    let (width, height) = (420, 520);
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = small_font(&mut client);
    let filter = ChatFilter::from_ini(&parse_ini(&client.read_system_text("chatfilter.ini").unwrap()));
    let mut view = ChatView::new(CHAT_HEIGHT, filter);
    view.filter.tabs[2].filters.insert("chat".to_string(), true);
    for (kind, name, text) in [(0, "Killer", "bom dia Giran"), (3, "Alice", "boss em 5 min"), (3, "Bob", "pronto")] {
        view.ingest(&ChatEntry::new(kind, name, text));
    }
    view.detach(2);
    let mut list = DrawList::default();
    view.paint(&mut list, 20, 320, view.active_list(), &font, 1.0);
    view.paint_split(&mut list, 40, 40, 2, &font, 1.0);
    let rgba = render_list(&mut client, &mut headless, &list);
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-chat-aba-solta.png");
    write_png(&out, width, height, &rgba).unwrap();
    assert_ne!(pixel_in(&rgba, width, 40 + 174, 40 + 9), CLEAR_RGBA, "cabeçalho da lista solta");
    assert_ne!(pixel_in(&rgba, width, 40 + 151 + 32, 40 + 141 + 11), CLEAR_RGBA, "aba da lista solta");
}

#[test]
#[ignore]
fn captures_the_npc_dialog_with_an_edit_box_and_an_open_combo() {
    let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
    let mut client = Client::open(&root).unwrap();
    let (width, height) = NPC_CANVAS;
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let font = small_font(&mut client);
    let html = "<html><body>Clan name:<br><edit var=\"name\" width=120 height=15 length=16 action=\"bypass -h npc_1_create $name\"><br><br>Weapon:<br><combobox var=\"kind\" list=\"Sword;Bow;Staff\" sel=\"Bow\" width=100><br><br><br><br><br><br><a action=\"bypass -h npc_1_create $name $kind\">Create</a></body></html>";
    let mut viewer = NpcViewer::new("Chat", html, &font);
    viewer.form.focus_var("name");
    for c in "Killers".chars() {
        viewer.form.key(InputKey::Char(c), &font);
    }
    let combo_y = 30 + viewer.lines.iter().flat_map(|line| line.widgets.iter().map(move |w| (line.y, w))).find(|(_, w)| matches!(w.kind, killer_ui::html::WidgetKind::ComboBox { .. })).map_or(0, |(y, _)| y);
    assert!(viewer.combo_press(CONTENT_POS.0 + 10, combo_y + 5));
    viewer.combo_move(CONTENT_POS.0 + 10, combo_y + 17 * 3 + 5);
    let (x, y) = NPC_POS;
    let mut list = DrawList::default();
    viewer.paint(&mut list, x, y, &font, 1.0);
    let rgba = render_list(&mut client, &mut headless, &list);
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-npc-edit-combo.png");
    write_png(&out, width, height, &rgba).unwrap();
    assert_ne!(pixel_in(&rgba, width, (x + CONTENT_POS.0 + 10) as u32, (y + combo_y + 17 * 2 + 8) as u32), CLEAR_RGBA, "lista do combo aberta");
}
