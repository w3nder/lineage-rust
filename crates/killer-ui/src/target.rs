use crate::color::{Argb, TEXT, target_name_color};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::status::{ellipsize, fill_width};
use crate::window::Rect;

pub const TARGET_SIZE: (i32, i32) = (172, 46);

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    (crate::window_restore::x87_to_int(f64::from(viewport.0 as f32) * 0.5 - 86.0), 0)
}

pub const TARGET_EXPANDED_HEIGHT: i32 = 76;
pub const TARGET_MIN_WIDTH: i32 = 172;
pub const SMALLWINDOW2_BACK1: &str = "L2UI_CH3.SmallWnd.SmallWindow2_Back1";
pub const SMALLWINDOW2_BACK2: &str = "L2UI_CH3.SmallWnd.SmallWindow2_Back2";
pub const SMALLWINDOW2_BACK3_EX: &str = "L2UI_CH3.SmallWnd.SmallWindow2_Back3_ex";
pub const TARGET_INFO_BUTTON_DOWN: &str = "L2UI_CH3.TargetWnd.Target_InfoButton_Down";
pub const CLAN_SYSSTRING: u32 = 604;
pub const ALLY_SYSSTRING: u32 = 605;
pub const NO_CLAN_SYSSTRING: u32 = 431;
pub const NO_ALLY_SYSSTRING: u32 = 591;
pub const INFO_LABEL_X: i32 = 20;
pub const INFO_CLAN_Y: i32 = 43;
pub const INFO_ALLY_Y: i32 = 59;
pub const INFO_NAME_X: i32 = 45;
pub const INFO_NAME_AFTER_CREST_X: i32 = 63;
pub const INFO_ALLY_CREST_X: i32 = 53;
pub const PLEDGE_CREST_SIZE: (i32, i32) = (16, 12);
pub const ALLY_CREST_SIZE: (i32, i32) = (8, 12);
pub const CREST_V: i32 = 4;
pub const INFO_LABEL_COLOR: Argb = Argb(0xFFA3A3A3);
pub const INFO_NAME_COLOR: Argb = Argb(0xFFB09B79);
pub const INFO_NONE_COLOR: Argb = Argb(0xFF808080);

pub const SMALLWINDOW_BACK1: &str = "L2UI_CH3.SmallWnd.SmallWindow_Back1";
pub const SMALLWINDOW_BACK2: &str = "L2UI_CH3.SmallWnd.SmallWindow_Back2";
pub const SMALLWINDOW_BACK3_EX: &str = "L2UI_CH3.SmallWnd.SmallWindow_Back3_ex";
pub const FRAME_CLOSE_BTN: &str = "L2UI_CH3.FrameCtrl.FrameCloseBtn";
pub const TARGET_INFO_BUTTON: &str = "L2UI_CH3.TargetWnd.Target_InfoButton";
pub const HP_BAR: &str = "L2UI_CH3.SmallWnd.HpBar";
pub const HP_BAR_BACK: &str = "L2UI_CH3.SmallWnd.HpBar_back";
pub const MP_BAR: &str = "L2UI_CH3.SmallWnd.MpBar";
pub const MP_BAR_BACK: &str = "L2UI_CH3.SmallWnd.MpBar_back";

pub const BODY_X: i32 = 12;
pub const BODY_END_WIDTH: i32 = 16;
pub const BODY_UW: i32 = 16;
pub const BODY_RIGHT_INSET: i32 = 16;
pub const BODY_RIGHT_WIDTH: i32 = 19;
pub const CLOSE_INSET: i32 = 19;
pub const CLOSE_Y: i32 = 6;
pub const CLOSE_SIZE: i32 = 15;
pub const INFO_BUTTON_POS: (i32, i32) = (1, 6);
pub const INFO_BUTTON_SIZE: (i32, i32) = (11, 13);
pub const BAR_X: i32 = 16;
pub const BAR_RIGHT_INSET: i32 = 22;
pub const BAR_HEIGHT: i32 = 6;
pub const BAR_UW: i32 = 8;
pub const HP_BAR_Y: i32 = 26;
pub const MP_BAR_Y: i32 = 33;
pub const NAME_X: i32 = 12;
pub const NAME_Y: i32 = 9;
pub const NAME_INSET: i32 = 30;
pub const SKILL_ICONS_PER_ROW: i32 = 8;
pub const SKILL_ICON_STEP: i32 = 18;
pub const SKILL_ICON_Y: i32 = 40;
pub const SKILL_ICON_SIZE: i32 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    Me,
    Player,
    Monster,
    Barred,
    Npc,
    Door,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetView {
    pub name: String,
    pub kind: TargetKind,
    pub hp: Option<(i32, i32)>,
    pub mp: Option<(i32, i32)>,
    pub level_difference: i32,
    pub expanded: bool,
    pub clan: Option<String>,
    pub ally: Option<String>,
    pub clan_crest: Option<String>,
    pub ally_crest: Option<String>,
    pub labels: [String; 4],
    pub skill_icons: Vec<String>,
}

pub fn info_labels(lookup: impl Fn(u32) -> Option<String>) -> [String; 4] {
    let text = |id: u32| lookup(id).unwrap_or_else(|| format!("sysstring {id}"));
    [text(CLAN_SYSSTRING), text(ALLY_SYSSTRING), text(NO_CLAN_SYSSTRING), text(NO_ALLY_SYSSTRING)]
}

pub fn target_height(expanded: bool) -> i32 {
    if expanded { TARGET_EXPANDED_HEIGHT } else { TARGET_SIZE.1 }
}

impl TargetView {
    pub fn new(name: String, kind: TargetKind, hp: Option<(i32, i32)>, mp: Option<(i32, i32)>) -> TargetView {
        TargetView {
            name,
            kind,
            hp,
            mp,
            level_difference: 0,
            expanded: false,
            clan: None,
            ally: None,
            clan_crest: None,
            ally_crest: None,
            labels: info_labels(|_| None),
            skill_icons: Vec::new(),
        }
    }

    pub fn height(&self) -> i32 {
        target_height(self.expanded)
    }

    pub fn shows_bars(&self) -> bool {
        matches!(self.kind, TargetKind::Me | TargetKind::Monster | TargetKind::Barred | TargetKind::Door)
    }

    pub fn colors_name_by_level(&self) -> bool {
        matches!(self.kind, TargetKind::Me | TargetKind::Monster | TargetKind::Barred)
    }

    pub fn shows_mp(&self) -> bool {
        matches!(self.kind, TargetKind::Me | TargetKind::Barred)
    }
}

pub fn close_button() -> Rect {
    close_button_sized(TARGET_SIZE.0)
}

pub fn close_button_sized(w: i32) -> Rect {
    Rect::new(w - CLOSE_INSET, CLOSE_Y, CLOSE_SIZE, CLOSE_SIZE)
}

pub fn info_button() -> Rect {
    Rect::new(INFO_BUTTON_POS.0, INFO_BUTTON_POS.1, INFO_BUTTON_SIZE.0, INFO_BUTTON_SIZE.1)
}

pub fn paint_target(list: &mut DrawList, x: i32, y: i32, view: &TargetView, font: &BitmapFont, alpha: f32) {
    paint_target_sized(list, x, y, TARGET_SIZE.0, view, font, alpha);
}

#[allow(clippy::too_many_arguments)]
pub fn paint_target_sized(list: &mut DrawList, x: i32, y: i32, w: i32, view: &TargetView, font: &BitmapFont, alpha: f32) {
    let h = view.height();
    let (back1, back2, back3) = if view.expanded {
        (SMALLWINDOW2_BACK1, SMALLWINDOW2_BACK2, SMALLWINDOW2_BACK3_EX)
    } else {
        (SMALLWINDOW_BACK1, SMALLWINDOW_BACK2, SMALLWINDOW_BACK3_EX)
    };
    FrameCtrl::new(FrameMode::VerticalGrip).paint(list, x, y, w, h, font, alpha);
    list.draw_texture(x + BODY_X, y, BODY_END_WIDTH, h, 0, 0, BODY_UW, h, back1, alpha);
    let mid_x = BODY_X + BODY_END_WIDTH;
    let mid_w = w - mid_x - BODY_RIGHT_INSET;
    list.draw_texture(x + mid_x, y, mid_w, h, 0, 0, BODY_UW, h, back2, alpha);
    let right_x = w - BODY_RIGHT_INSET;
    list.draw_texture(x + right_x, y, BODY_RIGHT_WIDTH, h, 0, 0, BODY_RIGHT_WIDTH, h, back3, alpha);

    let (bw, bh) = INFO_BUTTON_SIZE;
    let info = if view.expanded { TARGET_INFO_BUTTON_DOWN } else { TARGET_INFO_BUTTON };
    list.draw_texture(x + INFO_BUTTON_POS.0, y + INFO_BUTTON_POS.1, bw, bh, 0, 0, bw, bh, info, 1.0);
    let close_x = x + w - CLOSE_INSET;
    list.draw_texture(close_x, y + CLOSE_Y, CLOSE_SIZE, CLOSE_SIZE, 0, 0, CLOSE_SIZE, CLOSE_SIZE, FRAME_CLOSE_BTN, 1.0);

    let bar_w = w - BAR_RIGHT_INSET;
    if view.shows_bars() {
        paint_bar(list, x + BAR_X, y + HP_BAR_Y, bar_w, HP_BAR_BACK, HP_BAR, fill_width(view.hp.unwrap_or((0, 0)), bar_w));
        if view.shows_mp() {
            paint_bar(list, x + BAR_X, y + MP_BAR_Y, bar_w, MP_BAR_BACK, MP_BAR, fill_width(view.mp.unwrap_or((0, 0)), bar_w));
        }
    }

    let color = if view.colors_name_by_level() { target_name_color(view.level_difference) } else { TEXT };
    let name_w = w - NAME_INSET;
    let name = ellipsize(&view.name, name_w, font);
    let text_x = x + NAME_X + name_w / 2 - font.measure(&name) as i32 / 2;
    list.draw_text(text_x, y + NAME_Y, color, &name, font);
    if view.expanded && matches!(view.kind, TargetKind::Me | TargetKind::Player) {
        paint_info(list, x, y, view, font);
    } else if view.expanded {
        for (i, icon) in view.skill_icons.iter().enumerate() {
            let i = i as i32;
            list.draw_texture(x + (i % SKILL_ICONS_PER_ROW + 1) * SKILL_ICON_STEP, y + i / SKILL_ICONS_PER_ROW * SKILL_ICON_STEP + SKILL_ICON_Y, SKILL_ICON_SIZE, SKILL_ICON_SIZE, 0, 0, 32, 32, icon, 1.0);
        }
    }
}

fn paint_info(list: &mut DrawList, x: i32, y: i32, view: &TargetView, font: &BitmapFont) {
    let [clan_label, ally_label, no_clan, no_ally] = &view.labels;
    list.draw_text(x + INFO_LABEL_X, y + INFO_CLAN_Y, INFO_LABEL_COLOR, clan_label, font);
    list.draw_text(x + INFO_LABEL_X, y + INFO_ALLY_Y, INFO_LABEL_COLOR, ally_label, font);
    match &view.clan {
        None => {
            list.draw_text(x + INFO_NAME_X, y + INFO_CLAN_Y, INFO_NONE_COLOR, no_clan, font);
            list.draw_text(x + INFO_NAME_X, y + INFO_ALLY_Y, INFO_NONE_COLOR, no_ally, font);
        }
        Some(clan) => {
            let mut name_x = INFO_NAME_X;
            if let Some(crest) = &view.clan_crest {
                let (cw, ch) = PLEDGE_CREST_SIZE;
                list.draw_texture(x + INFO_NAME_X, y + INFO_CLAN_Y, cw, ch, 0, CREST_V, cw, ch, crest, 1.0);
                name_x = INFO_NAME_AFTER_CREST_X;
            }
            list.draw_text(x + name_x, y + INFO_CLAN_Y, INFO_NAME_COLOR, clan, font);
            let mut ally_x = INFO_NAME_X;
            if let Some(crest) = &view.ally_crest {
                let (cw, ch) = ALLY_CREST_SIZE;
                list.draw_texture(x + INFO_ALLY_CREST_X, y + INFO_ALLY_Y, cw, ch, 0, CREST_V, cw, ch, crest, 1.0);
                ally_x = INFO_NAME_AFTER_CREST_X;
            }
            match &view.ally {
                Some(ally) => list.draw_text(x + ally_x, y + INFO_ALLY_Y, INFO_NAME_COLOR, ally, font),
                None => list.draw_text(x + INFO_NAME_X, y + INFO_ALLY_Y, INFO_NONE_COLOR, no_ally, font),
            };
        }
    }
}

fn paint_bar(list: &mut DrawList, x: i32, y: i32, w: i32, back: &str, bar: &str, fill: i32) {
    list.draw_texture(x, y, w, BAR_HEIGHT, 0, 0, BAR_UW, BAR_HEIGHT, back, 1.0);
    if fill > 0 {
        list.draw_texture(x, y, fill, BAR_HEIGHT, 0, 0, BAR_UW, BAR_HEIGHT, bar, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Argb;
    use crate::draw::Quad;
    use crate::font::{Glyphs, gly};

    const TEXTURES: &str = include_str!("../tests/fixtures/nwindow-texturas-target.txt");

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        let bytes = gly(&[(0x20, &glyphs)], (1024, 128));
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&bytes).unwrap())
    }

    fn view() -> TargetView {
        TargetView { level_difference: -9, ..TargetView::new("Gremlin".into(), TargetKind::Monster, Some((50, 100)), None) }
    }

    fn painted(view: &TargetView) -> Vec<Quad> {
        let mut list = DrawList::default();
        paint_target(&mut list, 0, 0, view, &font(), 1.0);
        list.quads
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    #[test]
    fn paint_target_sized_stretches_body_bars_name_and_close_button_with_the_width() {
        let mut list = DrawList::default();
        paint_target_sized(&mut list, 0, 0, 300, &view(), &font(), 1.0);
        let quads = list.quads;
        let back2 = with_texture(&quads, SMALLWINDOW_BACK2)[0];
        assert_eq!(back2.w, (300 - BODY_X - BODY_END_WIDTH - BODY_RIGHT_INSET) as f32);
        assert_eq!(with_texture(&quads, HP_BAR_BACK)[0].w, (300 - BAR_RIGHT_INSET) as f32);
        assert_eq!(with_texture(&quads, FRAME_CLOSE_BTN)[0].x, (300 - CLOSE_INSET) as f32);
        assert_eq!(close_button_sized(300).x, 300 - CLOSE_INSET);
        assert_eq!(painted(&view()), {
            let mut base = DrawList::default();
            paint_target_sized(&mut base, 0, 0, TARGET_SIZE.0, &view(), &font(), 1.0);
            base.quads
        });
    }

    #[test]
    fn the_info_button_expands_the_window_to_76_with_clan_and_ally_lines_and_crests() {
        let mut view = TargetView::new("Killer".into(), TargetKind::Player, None, None);
        view.expanded = true;
        view.labels = ["Clan".into(), "Ally".into(), "No Clan".into(), "No alliance".into()];
        assert_eq!(view.height(), 76);
        let quads = painted(&view);
        assert_eq!(with_texture(&quads, SMALLWINDOW2_BACK2)[0].h, 76.0);
        assert_eq!(with_texture(&quads, TARGET_INFO_BUTTON_DOWN).len(), 1);
        let font = font();
        let text: Vec<(f32, f32, [f32; 4])> = quads.iter().filter(|q| q.texture == font.atlas).map(|q| (q.x, q.y, q.color)).collect();
        let gray = INFO_NONE_COLOR.rgba();
        assert!(text.iter().any(|&(x, y, c)| x == 20.0 && y == 43.0 && c == INFO_LABEL_COLOR.rgba()));
        assert!(text.iter().any(|&(x, y, c)| x == 20.0 && y == 59.0 && c == INFO_LABEL_COLOR.rgba()));
        assert!(text.iter().any(|&(x, y, c)| x == 45.0 && y == 43.0 && c == gray));
        assert!(text.iter().any(|&(x, y, c)| x == 45.0 && y == 59.0 && c == gray));
        view.clan = Some("Canastra".into());
        view.clan_crest = Some("crest.crest_5".into());
        view.ally = Some("Baralho".into());
        view.ally_crest = Some("crest.ally_6".into());
        let quads = painted(&view);
        let crest = with_texture(&quads, "crest.crest_5");
        assert_eq!((crest[0].x, crest[0].y, crest[0].w, crest[0].h, crest[0].v, crest[0].vh), (45.0, 43.0, 16.0, 12.0, 4.0, 12.0));
        let ally = with_texture(&quads, "crest.ally_6");
        assert_eq!((ally[0].x, ally[0].y, ally[0].w, ally[0].h), (53.0, 59.0, 8.0, 12.0));
        let gold = INFO_NAME_COLOR.rgba();
        assert!(quads.iter().any(|q| q.texture == font.atlas && q.x == 63.0 && q.y == 43.0 && q.color == gold));
        assert!(quads.iter().any(|q| q.texture == font.atlas && q.x == 63.0 && q.y == 59.0 && q.color == gold));
        view.expanded = false;
        assert_eq!(view.height(), 46);
        assert!(with_texture(&painted(&view), "crest.crest_5").is_empty());
        let npc = TargetView { expanded: true, ..TargetView::new("Gremlin".into(), TargetKind::Npc, Some((1, 2)), None) };
        assert!(painted(&npc).iter().all(|q| !(q.texture == font.atlas && q.y == 43.0)));
    }

    #[test]
    fn body_is_the_grip_three_slices_and_the_two_buttons_over_172_by_46() {
        let quads = painted(&view());
        let grip = with_texture(&quads, crate::frame::SMALLBAR_2);
        assert_eq!((grip[0].x, grip[0].y, grip[0].w, grip[0].h), (0.0, 8.0, 12.0, 30.0));
        let back1 = with_texture(&quads, SMALLWINDOW_BACK1);
        assert_eq!((back1[0].x, back1[0].y, back1[0].w, back1[0].h, back1[0].uw, back1[0].vh), (12.0, 0.0, 16.0, 46.0, 16.0, 46.0));
        let back2 = with_texture(&quads, SMALLWINDOW_BACK2);
        assert_eq!((back2[0].x, back2[0].w, back2[0].uw), (28.0, 128.0, 16.0));
        let back3 = with_texture(&quads, SMALLWINDOW_BACK3_EX);
        assert_eq!((back3[0].x, back3[0].w, back3[0].uw, back3[0].vh), (156.0, 19.0, 19.0, 46.0));
        let close = with_texture(&quads, FRAME_CLOSE_BTN);
        assert_eq!((close[0].x, close[0].y, close[0].w, close[0].h), (153.0, 6.0, 15.0, 15.0));
        let info = with_texture(&quads, TARGET_INFO_BUTTON);
        assert_eq!((info[0].x, info[0].y, info[0].w, info[0].h), (1.0, 6.0, 11.0, 13.0));
    }

    #[test]
    fn npc_target_shows_a_six_pixel_hp_bar_and_no_mp_bar() {
        let quads = painted(&view());
        let back = with_texture(&quads, HP_BAR_BACK);
        assert_eq!((back[0].x, back[0].y, back[0].w, back[0].h), (16.0, 26.0, 150.0, 6.0));
        assert_eq!((back[0].uw, back[0].vh), (8.0, 6.0));
        let fill = with_texture(&quads, HP_BAR);
        assert_eq!((fill[0].x, fill[0].y, fill[0].w, fill[0].h), (16.0, 26.0, 75.0, 6.0));
        assert!(with_texture(&quads, MP_BAR_BACK).is_empty());
        let mut unknown = view();
        unknown.hp = None;
        let quads = painted(&unknown);
        assert_eq!(with_texture(&quads, HP_BAR_BACK).len(), 1);
        assert!(with_texture(&quads, HP_BAR).is_empty());
    }

    #[test]
    fn me_shows_hp_and_mp_and_other_players_show_only_the_name() {
        let me = TargetView { kind: TargetKind::Me, mp: Some((30, 60)), ..view() };
        let quads = painted(&me);
        let mp = with_texture(&quads, MP_BAR);
        assert_eq!((mp[0].x, mp[0].y, mp[0].w, mp[0].h), (16.0, 33.0, 75.0, 6.0));
        assert_eq!(with_texture(&quads, MP_BAR_BACK)[0].y, 33.0);
        let player = TargetView { kind: TargetKind::Player, ..view() };
        let quads = painted(&player);
        assert!(with_texture(&quads, HP_BAR_BACK).is_empty());
        assert!(with_texture(&quads, MP_BAR_BACK).is_empty());
        let font = font();
        assert!(quads.iter().filter(|q| q.texture == font.atlas).all(|q| q.color == TEXT.rgba()));
    }

    #[test]
    fn name_is_centered_in_the_name_control_and_colored_by_level_difference() {
        let font = font();
        let quads = painted(&view());
        let name: Vec<&Quad> = quads.iter().filter(|q| q.texture == font.atlas).collect();
        assert_eq!(name.len(), 7);
        assert_eq!((name[0].x, name[0].y), ((12 + 71 - 21) as f32, 9.0));
        assert!(name.iter().all(|q| q.color == Argb(0xFFFF0000).rgba()));
        let even = TargetView { level_difference: 0, ..view() };
        let quads = painted(&even);
        assert!(quads.iter().filter(|q| q.texture == font.atlas).all(|q| q.color == TEXT.rgba()));
        let long = TargetView { name: "a".repeat(40), ..view() };
        let quads = painted(&long);
        let name: Vec<&Quad> = quads.iter().filter(|q| q.texture == font.atlas).collect();
        assert_eq!(name.len(), 23);
        assert!(name.iter().all(|q| q.x >= 12.0 && q.x + q.w <= 154.0));
    }

    #[test]
    fn frame_and_body_carry_the_window_alpha_but_not_the_widgets() {
        let mut list = DrawList::default();
        paint_target(&mut list, 5, 7, &view(), &font(), 0.5);
        assert_eq!(with_texture(&list.quads, SMALLWINDOW_BACK2)[0].color[3], 0.5);
        assert_eq!(with_texture(&list.quads, crate::frame::SMALLBAR_1)[0].color[3], 0.5);
        assert_eq!(with_texture(&list.quads, HP_BAR_BACK)[0].color[3], 1.0);
        assert_eq!(with_texture(&list.quads, FRAME_CLOSE_BTN)[0].color[3], 1.0);
        assert_eq!((with_texture(&list.quads, SMALLWINDOW_BACK1)[0].x, with_texture(&list.quads, SMALLWINDOW_BACK1)[0].y), (17.0, 7.0));
    }

    #[test]
    fn every_texture_exists_in_the_client_list() {
        let known: Vec<String> = TEXTURES.lines().map(|l| l.trim().to_lowercase()).collect();
        let me = TargetView { kind: TargetKind::Me, mp: Some((30, 60)), ..view() };
        for texture in painted(&me).iter().map(|q| q.texture.as_str()).filter(|t| *t != "L2Font-e.SmallFont-e") {
            assert!(known.contains(&texture.to_lowercase()), "textura desconhecida {texture}");
        }
    }

    #[test]
    fn an_expanded_npc_lists_its_skill_icons_eight_to_a_row_from_x_eighteen() {
        let mut view = TargetView { expanded: true, ..view() };
        view.skill_icons = (0..10).map(|i| format!("icon.s{i}")).collect();
        let quads = painted(&view);
        let first = with_texture(&quads, "icon.s0")[0];
        assert_eq!((first.x, first.y, first.w, first.h, first.uw, first.vh), (18.0, 40.0, 15.0, 15.0, 32.0, 32.0));
        let eighth = with_texture(&quads, "icon.s7")[0];
        assert_eq!((eighth.x, eighth.y), (144.0, 40.0));
        let ninth = with_texture(&quads, "icon.s8")[0];
        assert_eq!((ninth.x, ninth.y), (18.0, 58.0));
        view.expanded = false;
        assert!(with_texture(&painted(&view), "icon.s0").is_empty());
    }

    #[test]
    fn a_flagged_npc_shows_both_bars_like_the_player() {
        let quads = painted(&TargetView { kind: TargetKind::Barred, mp: Some((1, 2)), ..view() });
        assert_eq!(with_texture(&quads, MP_BAR_BACK).len(), 1);
    }

    #[test]
    fn only_the_player_and_monsters_get_bars_and_the_rest_keep_the_plain_name() {
        let monster = painted(&view());
        assert_eq!(with_texture(&monster, HP_BAR_BACK).len(), 1);
        assert!(with_texture(&monster, MP_BAR_BACK).is_empty());
        let npc = painted(&TargetView { kind: TargetKind::Npc, ..view() });
        assert!(with_texture(&npc, HP_BAR_BACK).is_empty());
        let text: Vec<_> = npc.iter().filter(|q| q.texture == "L2Font-e.SmallFont-e").collect();
        assert!(text.iter().all(|q| q.color == TEXT.rgba()));
    }

    #[test]
    fn a_door_shows_the_hp_bar_alone_with_the_plain_name_color() {
        let door = painted(&TargetView { kind: TargetKind::Door, level_difference: -9, hp: Some((50, 100)), ..view() });
        assert_eq!(with_texture(&door, HP_BAR_BACK).len(), 1);
        assert!(with_texture(&door, MP_BAR_BACK).is_empty());
        let fill = with_texture(&door, HP_BAR);
        assert_eq!(fill.len(), 1);
        assert_eq!(fill[0].w, fill_width((50, 100), TARGET_SIZE.0 - BAR_RIGHT_INSET) as f32);
        let text: Vec<_> = door.iter().filter(|q| q.texture == "L2Font-e.SmallFont-e").collect();
        assert!(text.iter().all(|q| q.color == TEXT.rgba()));
    }
}
