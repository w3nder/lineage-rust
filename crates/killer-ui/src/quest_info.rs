#[path = "quest_info_controls_generated.rs"]
mod controls;
#[path = "quest_info_layout_generated.rs"]
mod layout;

use crate::button::TextButton;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::list_ctrl::{BigIcon, ListCell, ListCtrl, ListRow};
use crate::window::{Rect, WindowId};

pub const ID: WindowId = WindowId::QUEST_LIST_INFO;
pub const SIZE: (i32, i32) = layout::SIZE;
pub const BACK: &str = "L2UI_CH3.QUESTWND.questinfo_back";
pub const BACK_ORIGIN: (i32, i32) = (0, 20);
pub const TOOLTIP_BUTTON: &str = "L2UI_CH3.QUESTWND.QuestWndToolTipBtn";
pub const TOOLTIP_BUTTON_RECT: Rect = Rect::new(578, 30, 15, 15);
pub const TOOLTIP_ICONS: [&str; 2] = [ICON_REPEATABLE, ICON_ONE_TIME];
pub const ICON_REPEATABLE: &str = "L2UI_CH3.QUESTWND.QuestWndInfoIcon_1";
pub const ICON_ONE_TIME: &str = "L2UI_CH3.QUESTWND.QuestWndInfoIcon_2";
pub const ICON_SIZE: i32 = 15;
pub const TOOLTIP_SYSSTRINGS: [u32; 2] = [861, 862];
pub const TOOLTIP_COLORS: [Argb; 2] = [Argb(0xFFA3A3A3), Argb(0xFFB09B79)];
pub const NO_LIMIT_SYSSTRING: u32 = 866;
pub const AND_ABOVE_SYSSTRING: u32 = 859;
pub const LISTED_STATE: i32 = 1;
pub const MAX_ROWS: usize = 2000;
pub const CELLS: usize = 5;

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    (0, (viewport.1 as f32 * 0.5 - 252.0) as i32)
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    let (x, y) = default_position(viewport);
    crate::initial_position::InitialPosition {
        enabled: true,
        horizontal_mode: 0,
        vertical_mode: 5,
        x,
        y: (viewport.1 / 2).wrapping_sub(y),
    }
}

pub fn grip() -> Rect {
    layout::RECT_7C
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuestInfoRow {
    pub quest_id: i32,
    pub title: String,
    pub requirement: String,
    pub level: i32,
    pub max_level: i32,
    pub quest_type: i32,
    pub npc_name: String,
    pub npc_loc: [f32; 3],
    pub intro: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum QuestInfoAction {
    Close,
    Location(crate::quest::QuestTarget),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestInfoHit {
    Close,
    Location,
    List,
}

pub fn level_text(level: i32, max_level: i32, no_limit: &str, and_above: &str) -> String {
    if level < 1 {
        no_limit.to_string()
    } else if max_level < 1 {
        format!("{level} {and_above}")
    } else {
        format!("{level}~{max_level}")
    }
}

pub fn type_icon(quest_type: i32) -> Option<&'static str> {
    match quest_type {
        0 | 2 => Some(ICON_REPEATABLE),
        1 | 3 => Some(ICON_ONE_TIME),
        _ => None,
    }
}

#[derive(Clone, Debug)]
pub struct QuestInfoView {
    pub frame: FrameCtrl,
    pub list: ListCtrl,
    pub button: TextButton,
    pub tooltip: [String; 2],
    pub tooltip_hover: bool,
    pub screen: (i32, i32),
    no_limit: String,
    and_above: String,
    quests: Vec<i32>,
    intros: Vec<String>,
    targets: Vec<crate::quest::QuestTarget>,
    pressed: Option<QuestInfoHit>,
}

impl QuestInfoView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        Self {
            frame: controls::FRAME_7C.build(text),
            list: ListCtrl::from_contract(&controls::LIST_E8),
            button: controls::BUTTON_F0.build((0, 0), text),
            tooltip: TOOLTIP_SYSSTRINGS.map(text),
            tooltip_hover: false,
            screen: (0, 0),
            no_limit: text(NO_LIMIT_SYSSTRING),
            and_above: text(AND_ABOVE_SYSSTRING),
            quests: Vec::new(),
            intros: Vec::new(),
            targets: Vec::new(),
            pressed: None,
        }
    }

    pub fn clear(&mut self) {
        self.list.clear();
        self.quests.clear();
        self.intros.clear();
        self.targets.clear();
    }

    pub fn set_rows(&mut self, rows: &[QuestInfoRow]) {
        self.clear();
        let mut icon: Option<&'static str> = None;
        for row in rows.iter().take(MAX_ROWS) {
            let mut cells = vec![ListCell::default(); CELLS];
            cells[0] = ListCell::text(&row.title);
            cells[1] = ListCell::text(&row.requirement);
            cells[2] = ListCell::text(&level_text(row.level, row.max_level, &self.no_limit, &self.and_above));
            cells[3] = ListCell::text(&row.quest_type.to_string());
            cells[3].text_enabled = false;
            icon = type_icon(row.quest_type).or(icon);
            if let Some(texture) = icon {
                cells[3].big_icon = Some(BigIcon { texture: texture.to_string(), w: ICON_SIZE, h: ICON_SIZE });
            }
            cells[4] = ListCell::text(&row.npc_name);
            if self.list.add_item(ListRow { cells }) {
                self.quests.push(row.quest_id);
                self.intros.push(row.intro.clone());
                self.targets.push(crate::quest::QuestTarget {
                    name: row.npc_name.clone(),
                    loc: row.npc_loc,
                    id: row.quest_id,
                    title: row.title.clone(),
                    minimap_only: false,
                });
            }
        }
    }

    pub fn selected_quest(&self) -> Option<i32> {
        self.list.selected_origin().and_then(|index| self.quests.get(index).copied())
    }

    pub fn row_tooltip(&self, text: &dyn Fn(i32) -> String) -> Option<crate::tooltip::Tooltip> {
        let origin = self.list.origins().get(usize::try_from(self.list.hovered()).ok()?).copied()?;
        let description = self.intros.get(origin).map_or("", String::as_str);
        self.list.row_tip(crate::list_ctrl::ROW_TIP_QUEST, description, text)
    }

    pub fn selected_intro(&self) -> Option<&str> {
        self.list.selected_origin().and_then(|index| self.intros.get(index)).map(String::as_str)
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<QuestInfoHit> {
        if controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(QuestInfoHit::Close);
        }
        if controls::BUTTON_F0.hit(&self.button, x, y).is_some() {
            return Some(QuestInfoHit::Location);
        }
        self.list.rect.contains(x, y).then_some(QuestInfoHit::List)
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        let button = TOOLTIP_BUTTON_RECT;
        self.tooltip_hover = Rect::new(button.x, button.y, button.w + 1, button.h + 1).contains(x, y);
        self.list.mouse_move(x, y);
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<QuestInfoHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        if hit == Some(QuestInfoHit::List) {
            self.list.mouse_down(x, y);
        }
        hit
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<QuestInfoAction> {
        self.list.mouse_up(x, y);
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            QuestInfoHit::Close => Some(QuestInfoAction::Close),
            QuestInfoHit::Location => self.list.selected_origin().and_then(|index| self.targets.get(index)).cloned().map(QuestInfoAction::Location),
            QuestInfoHit::List => None,
        }
    }

    pub fn wheel(&mut self, delta: i16) -> bool {
        self.list.wheel(delta)
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32, lookup: &dyn Fn(u32) -> Option<String>) {
        let (x, y) = origin;
        controls::FRAME_7C.paint(&self.frame, list, origin, font, alpha);
        let start = list.quads.len();
        list.draw_texture(x + BACK_ORIGIN.0, y + BACK_ORIGIN.1, SIZE.0, SIZE.1 - BACK_ORIGIN.1, 0, 0, SIZE.0, SIZE.1 - BACK_ORIGIN.1, BACK, alpha);
        list.draw_texture(
            x + TOOLTIP_BUTTON_RECT.x,
            y + TOOLTIP_BUTTON_RECT.y,
            TOOLTIP_BUTTON_RECT.w,
            TOOLTIP_BUTTON_RECT.h,
            0,
            0,
            TOOLTIP_BUTTON_RECT.w,
            TOOLTIP_BUTTON_RECT.h,
            TOOLTIP_BUTTON,
            alpha,
        );
        list.clip_from(start, Rect::new(x, y, SIZE.0, SIZE.1));
        self.list.paint_header(list, origin, font, &|id| lookup(id as u32));
        self.list.paint(list, origin, font);
        controls::BUTTON_F0.paint(&self.button, list, origin, self.pressed == Some(QuestInfoHit::Location), font);
        if self.tooltip_hover {
            let rows: Vec<(&str, &str)> = TOOLTIP_ICONS.iter().copied().zip(self.tooltip.iter().map(String::as_str)).collect();
            crate::quest::paint_icon_tip(list, origin, (TOOLTIP_BUTTON_RECT.x, TOOLTIP_BUTTON_RECT.y), self.screen, &rows, font);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> QuestInfoView {
        QuestInfoView::new(|id| Some(format!("s{id}")))
    }

    fn row(id: i32, level: i32, max: i32, kind: i32) -> QuestInfoRow {
        QuestInfoRow {
            quest_id: id,
            title: format!("q{id}"),
            requirement: "req".into(),
            level,
            max_level: max,
            quest_type: kind,
            npc_name: "npc".into(),
            npc_loc: [1.0, 2.0, 3.0],
            intro: format!("intro{id}"),
        }
    }

    #[test]
    fn the_layout_matches_the_on_create_snapshot() {
        let view = view();
        assert_eq!(SIZE, (600, 326));
        assert_eq!(view.list.rect, Rect::new(7, 49, 586, 237));
        assert_eq!(view.list.columns.iter().map(|column| column.width).collect::<Vec<_>>(), vec![212, 130, 52, 40, 152]);
        assert_eq!(view.button.rect, Rect::new(518, 297, 76, 23));
        assert_eq!(view.button.label, "s860");
        assert_eq!(view.frame.title, "s1204");
        assert_eq!(view.tooltip, ["s861".to_string(), "s862".to_string()]);
    }

    #[test]
    fn level_text_follows_the_three_branches() {
        assert_eq!(level_text(0, 5, "none", "up"), "none");
        assert_eq!(level_text(10, 0, "none", "up"), "10 up");
        assert_eq!(level_text(10, 20, "none", "up"), "10~20");
    }

    #[test]
    fn the_type_icon_has_no_branch_above_three() {
        assert_eq!(type_icon(0), Some(ICON_REPEATABLE));
        assert_eq!(type_icon(2), Some(ICON_REPEATABLE));
        assert_eq!(type_icon(1), Some(ICON_ONE_TIME));
        assert_eq!(type_icon(3), Some(ICON_ONE_TIME));
        assert_eq!(type_icon(4), None);
    }

    #[test]
    fn rows_fill_five_cells_and_keep_the_quest_ids_and_intros() {
        let mut view = view();
        view.set_rows(&[row(7, 10, 20, 0), row(9, 0, 0, 1)]);
        assert_eq!(view.list.count(), 2);
        let first = &view.list.rows[0].cells;
        assert_eq!(first.len(), 5);
        assert_eq!(first[0].text, "q7");
        assert_eq!(first[2].text, "10~20");
        assert_eq!(first[3].big_icon.as_ref().map(|icon| (icon.texture.as_str(), icon.w, icon.h)), Some((ICON_REPEATABLE, 15, 15)));
        assert_eq!((first[3].text.as_str(), first[3].text_enabled), ("0", false));
        assert_eq!(view.list.rows[1].cells[3].text, "1");
        assert_eq!(view.list.rows[1].cells[2].text, "s866");
        view.list.select(1);
        assert_eq!(view.selected_quest(), Some(9));
        assert_eq!(view.selected_intro(), Some("intro9"));
        view.set_rows(&[]);
        assert_eq!(view.list.count(), 0);
        assert_eq!(view.selected_quest(), None);
    }

    #[test]
    fn location_needs_a_selection_and_reports_its_quest() {
        let mut view = view();
        view.set_rows(&[row(7, 1, 2, 0)]);
        let rect = view.button.rect;
        view.mouse_down(rect.x + 2, rect.y + 2);
        assert_eq!(view.mouse_up(rect.x + 2, rect.y + 2), None);
        view.list.select(0);
        view.mouse_down(rect.x + 2, rect.y + 2);
        assert_eq!(
            view.mouse_up(rect.x + 2, rect.y + 2),
            Some(QuestInfoAction::Location(crate::quest::QuestTarget { name: "npc".into(), loc: [1.0, 2.0, 3.0], id: 7, title: "q7".into(), minimap_only: false }))
        );
    }

    #[test]
    fn location_follows_the_sorted_row_through_the_list_origin() {
        let mut view = view();
        view.set_rows(&[row(30, 1, 2, 0), row(10, 1, 2, 0), row(20, 1, 2, 0)]);
        view.list.sort(0);
        assert_eq!(view.list.rows.iter().map(|r| r.cells[0].text.clone()).collect::<Vec<_>>(), vec!["q10", "q20", "q30"]);
        view.list.select(0);
        assert_eq!(view.selected_quest(), Some(10));
        assert_eq!(view.selected_intro(), Some("intro10"));
    }

    #[test]
    fn nc_quest_list_info_wnd_100eb870_matches_the_unicorn_paint() {
        let view = view();
        let font = crate::font::BitmapFont::new(
            "font".into(),
            crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &(0..95).map(|i| [i * 7, 6, 0, 13]).collect::<Vec<_>>())], (1024, 128))).unwrap(),
        );
        let mut list = DrawList::default();
        view.paint(&mut list, (0, 0), &font, 1.0, &|id| Some(format!("s{id}")));
        let quad = |texture: &str| list.quads.iter().find(|q| q.texture == texture).map(|q| (q.x, q.y, q.w, q.h, q.uw, q.vh)).unwrap();
        assert_eq!(quad(BACK), (0.0, 20.0, 600.0, 306.0, 600.0, 306.0));
        assert_eq!(quad(TOOLTIP_BUTTON), (578.0, 30.0, 15.0, 15.0, 15.0, 15.0));
    }

    #[test]
    fn the_tooltip_flag_follows_the_help_button() {
        let mut view = view();
        view.mouse_move(580, 32);
        assert!(view.tooltip_hover);
        view.mouse_move(10, 10);
        assert!(!view.tooltip_hover);
        view.mouse_move(578 + 15, 30 + 15);
        assert!(view.tooltip_hover);
        view.mouse_move(578 + 16, 30);
        assert!(!view.tooltip_hover);
    }

    #[test]
    fn nc_quest_list_info_wnd_100ece50_draws_the_two_row_balloon_above_the_help_button() {
        let mut view = view();
        view.screen = (1024, 768);
        view.mouse_move(580, 32);
        let font = crate::font::BitmapFont::new(
            "font".into(),
            crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &(0..95).map(|i| [i * 7, 6, 0, 13]).collect::<Vec<_>>())], (1024, 128))).unwrap(),
        );
        let mut list = DrawList::default();
        view.paint(&mut list, (100, 200), &font, 1.0, &|id| Some(format!("s{id}")));
        let icons: Vec<_> = list.quads.iter().filter(|q| q.texture == ICON_REPEATABLE || q.texture == ICON_ONE_TIME).filter(|q| q.w == 16.0).collect();
        assert_eq!(icons.len(), 2);
        let top = 200.0 + 30.0 - 48.0 + 5.0;
        assert_eq!((icons[0].x, icons[0].y, icons[1].y), (100.0 + 578.0 + 5.0, top, top + 19.0));
        assert!(list.quads.iter().any(|q| q.texture == crate::tooltip::FRAME_TEXTURES[0] && q.y == 200.0 + 30.0 - 48.0 && q.x == 100.0 + 578.0));
        let mut near_top = DrawList::default();
        view.paint(&mut near_top, (100, 10), &font, 1.0, &|id| Some(format!("s{id}")));
        let below = near_top.quads.iter().find(|q| q.texture == ICON_REPEATABLE && q.w == 16.0).unwrap();
        assert_eq!(below.y, 10.0 + 30.0 + 5.0);
        view.screen = (700, 768);
        let mut shifted = DrawList::default();
        view.paint(&mut shifted, (100, 200), &font, 1.0, &|id| Some(format!("s{id}")));
        let icon = shifted.quads.iter().find(|q| q.texture == ICON_REPEATABLE && q.w == 16.0).unwrap();
        assert_eq!(icon.x, 700.0 - 184.0 + 5.0);
    }

    #[test]
    fn nc_quest_list_info_wnd_row_tooltip_0x10000_lists_four_lines_and_the_summary() {
        let mut view = view();
        let mut quest = row(9, 10, 20, 1);
        quest.intro = "intro".into();
        view.set_rows(&[quest]);
        let x = view.list.rect.x + 2;
        let y = view.list.rect.y + 19 + 2;
        view.mouse_move(x, y);
        assert!(view.row_tooltip(&|id| format!("s{id}")).is_none());
        view.mouse_down(x, y);
        view.mouse_up(x, y);
        view.mouse_move(x, y);
        let tip = view.row_tooltip(&|id| format!("s{id}")).unwrap();
        assert!(tip.row && tip.row_style == crate::list_ctrl::ROW_TIP_QUEST);
        let lines: Vec<_> = tip.lines.iter().map(|line| (line.label.as_str(), line.value.as_str())).collect();
        assert_eq!(lines, [("s1200", "q9"), ("s1201", "req"), ("s922", "10~20"), ("s1202", "s862")]);
        assert_eq!(tip.description, "intro");
    }

    #[test]
    fn a_type_the_icons_do_not_cover_keeps_the_icon_of_the_previous_row() {
        let mut view = view();
        view.set_rows(&[row(1, 1, 2, 1), row(2, 1, 2, 7)]);
        let icons: Vec<_> = view.list.rows.iter().map(|row| row.cells[3].big_icon.as_ref().map(|icon| icon.texture.clone())).collect();
        assert_eq!(icons[0].as_deref(), Some(ICON_ONE_TIME));
        assert_eq!(icons[1].as_deref(), Some(ICON_ONE_TIME));
    }
}
