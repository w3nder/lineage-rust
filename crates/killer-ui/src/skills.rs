use crate::color::BUTTON_TEXT;
use crate::cooltime::{self, CoolFrame};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::inventory::{BLACK_TEXTURE, SLOT, grid_slot_at, grid_slot_pos_cols};
use crate::scroll::{ScrollHit, ScrollWnd, clamp_first_row, rows_for};
use crate::window::Rect;

pub const PAGE_SIZE: (i32, i32) = (256, 335);
pub const SKILL_BACK: &str = "L2UI_CH3.SkillWnd.skill_back";
pub const TAB_NORMAL: &str = "L2UI_CH3.skillwnd.skill_tab2";
pub const TAB_ACTIVE: &str = "L2UI_CH3.skillwnd.skill_tab1";
pub const ICON_BACK: &str = "l2ui.nwindow.icon_back";
pub const ICON_CLICK: &str = "l2ui.nwindow.icon_click";
pub const ITEM_CLICK: &str = "l2ui.NWindow.item_click";
pub const TOGGLE_EFFECT: &str = "L2_SkillTime.ToggleEffect.ToggleEffect001";
pub const TAB_RECT: Rect = Rect::new(12, 8, 189, 23);
pub const TAB_SIZE: (i32, i32) = (94, 23);
pub const TAB_GAP: i32 = 1;
pub const LIST_RECT: Rect = Rect::new(9, 40, 239, 280);
pub const LIST_COLS: usize = 6;
pub const LIST_ROWS: usize = 8;
pub const GM_LIST_ROWS: usize = 7;
pub const OUTLINE_SIZE: i32 = 34;
pub const TAB_ACTIVE_SYSSTRING: u32 = 120;
pub const TAB_PASSIVE_SYSSTRING: u32 = 121;
pub const TAB_CTRL_ID: u32 = 0xaf11000;
pub const LIST_CTRL_ID: u32 = 0x1f01;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkillTab {
    #[default]
    Active,
    Passive,
}

impl SkillTab {
    pub fn index(self) -> usize {
        match self {
            SkillTab::Active => 0,
            SkillTab::Passive => 1,
        }
    }
}

pub fn tab_rect(tab: SkillTab) -> Rect {
    let (w, h) = TAB_SIZE;
    Rect::new(TAB_RECT.x + (w + TAB_GAP) * tab.index() as i32, TAB_RECT.y, w, h)
}

pub fn slot_pos(index: usize) -> (i32, i32) {
    grid_slot_pos_cols(index, LIST_COLS)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillHit {
    Tab(SkillTab),
    Slot(usize),
    Scroll(ScrollHit),
    Page,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SkillView {
    pub id: i32,
    pub level: i32,
    pub passive: bool,
    pub icon: String,
    pub name: String,
    pub description: String,
    pub toggled: bool,
    pub cool: Option<CoolFrame>,
}

impl SkillView {
    pub fn icon(&self) -> &str {
        if self.icon.is_empty() { BLACK_TEXTURE } else { &self.icon }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SkillsView {
    pub tab: SkillTab,
    pub tab_names: [String; 2],
    pub skills: Vec<SkillView>,
    pub selected: Option<i32>,
    pub hovered: Option<usize>,
    pub pressed: bool,
    pub scroll_row: usize,
    pub scroll: ScrollWnd,
    pub visible_rows: usize,
}

impl SkillsView {
    pub fn with_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> SkillsView {
        let text = |id: u32| lookup(id).unwrap_or_else(|| format!("sysstring {id}"));
        SkillsView {
            tab_names: [text(TAB_ACTIVE_SYSSTRING), text(TAB_PASSIVE_SYSSTRING)],
            scroll: ScrollWnd::new(LIST_RECT.w, LIST_RECT.h),
            ..Default::default()
        }
    }

    pub fn gm_with_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> SkillsView {
        SkillsView { visible_rows: GM_LIST_ROWS, ..SkillsView::with_sysstrings(lookup) }
    }

    pub fn rows(&self) -> usize {
        if self.visible_rows == 0 { LIST_ROWS } else { self.visible_rows }
    }

    pub fn total_rows(&self) -> i32 {
        rows_for(self.listed().len(), LIST_COLS)
    }

    pub fn update_rows(&mut self) {
        let total = self.total_rows();
        let visible = self.rows() as i32;
        self.scroll_row = clamp_first_row(self.scroll_row as i32, visible, total).max(0) as usize;
        self.scroll.slider.set_range(total, visible);
        self.scroll.slider.set_pos(self.scroll_row as i32);
    }

    pub fn set_first_row(&mut self, first: i32) {
        self.scroll_row = first.max(0) as usize;
        self.scroll.slider.set_pos(first);
    }

    pub fn scroll_hit(&self, x: i32, y: i32) -> Option<ScrollHit> {
        self.scroll.hit(LIST_RECT.w, LIST_RECT.h, x - LIST_RECT.x, y - LIST_RECT.y)
    }

    pub fn listed(&self) -> Vec<&SkillView> {
        let passive = self.tab == SkillTab::Passive;
        self.skills.iter().filter(|s| s.passive == passive).collect()
    }

    pub fn slot_skill(&self, index: usize) -> Option<&SkillView> {
        self.listed().get(index).copied()
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<SkillHit> {
        if !Rect::new(0, 0, PAGE_SIZE.0, PAGE_SIZE.1).contains(x, y) {
            return None;
        }
        for tab in [SkillTab::Active, SkillTab::Passive] {
            if tab_rect(tab).contains(x, y) {
                return Some(SkillHit::Tab(tab));
            }
        }
        if let Some(hit) = self.scroll_hit(x, y) {
            return Some(SkillHit::Scroll(hit));
        }
        if LIST_RECT.contains(x, y)
            && let Some(index) = grid_slot_at(x - LIST_RECT.x, y - LIST_RECT.y, LIST_COLS, self.rows(), self.scroll_row)
        {
            return Some(SkillHit::Slot(index));
        }
        Some(SkillHit::Page)
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let (w, h) = PAGE_SIZE;
        list.draw_texture(x, y, w, h, 0, 0, w, h, SKILL_BACK, 1.0);
        for (tab, name) in [SkillTab::Active, SkillTab::Passive].into_iter().zip(&self.tab_names) {
            let rect = tab_rect(tab);
            let texture = if tab == self.tab { TAB_ACTIVE } else { TAB_NORMAL };
            list.draw_texture(x + rect.x, y + rect.y, rect.w, rect.h, 0, 0, rect.w, rect.h, texture, 1.0);
            let text_w = font.measure(name) as i32;
            list.draw_text(
                x + rect.x + rect.w / 2 - text_w / 2,
                y + rect.y + rect.h / 2 - font.line_height as i32 / 2,
                BUTTON_TEXT,
                name,
                font,
            );
        }
        let first = self.scroll_row * LIST_COLS;
        let (lx, ly) = (x + LIST_RECT.x, y + LIST_RECT.y);
        for (index, skill) in self.listed().into_iter().enumerate().skip(first).take(LIST_COLS * self.rows()) {
            let (sx, sy) = slot_pos(index - first);
            let (sx, sy) = (lx + sx, ly + sy);
            if self.selected == Some(skill.id) {
                list.draw_texture(sx, sy, SLOT, SLOT, 0, 0, SLOT, SLOT, ITEM_CLICK, 1.0);
            }
            list.draw_texture(sx, sy, SLOT, SLOT, 0, 0, SLOT, SLOT, skill.icon(), 1.0);
            if !skill.passive {
                list.draw_texture(
                    sx - 1,
                    sy - 1,
                    OUTLINE_SIZE,
                    OUTLINE_SIZE,
                    0,
                    0,
                    OUTLINE_SIZE,
                    OUTLINE_SIZE,
                    ICON_BACK,
                    1.0,
                );
                if self.pressed && self.selected == Some(skill.id) {
                    list.draw_texture(
                        sx - 1,
                        sy - 1,
                        OUTLINE_SIZE,
                        OUTLINE_SIZE,
                        0,
                        0,
                        OUTLINE_SIZE,
                        OUTLINE_SIZE,
                        ICON_CLICK,
                        1.0,
                    );
                }
            }
            if skill.toggled {
                list.draw_texture(sx, sy, SLOT, SLOT, 0, 0, SLOT, SLOT, TOGGLE_EFFECT, 1.0);
            }
            if let Some(frame) = skill.cool {
                cooltime::paint(list, sx, sy, SLOT, frame);
            }
        }
        self.scroll.paint(list, lx, ly, LIST_RECT.w, LIST_RECT.h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};


    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        let bytes = gly(&[(0x20, &glyphs)], (1024, 128));
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&bytes).unwrap())
    }

    fn skill(id: i32, passive: bool, icon: &str) -> SkillView {
        SkillView { id, level: 1, passive, icon: icon.into(), ..Default::default() }
    }

    fn view() -> SkillsView {
        let mut view = SkillsView::with_sysstrings(|id| Some(format!("s{id}")));
        view.skills = vec![
            skill(3, false, "icon.skill0003"),
            skill(1177, false, "icon.skill1177"),
            skill(194, true, "icon.skill0194"),
        ];
        view
    }

    #[test]
    fn tabs_and_list_are_where_on_create_puts_them() {
        assert_eq!(tab_rect(SkillTab::Active), Rect::new(12, 8, 94, 23));
        assert_eq!(tab_rect(SkillTab::Passive), Rect::new(107, 8, 94, 23));
        assert_eq!(LIST_RECT, Rect::new(9, 40, 239, 280));
        assert_eq!(slot_pos(0), (1, 1));
        assert_eq!(slot_pos(5), (186, 1));
        assert_eq!(slot_pos(6), (1, 36));
        assert_eq!(slot_pos(47), (186, 246));
        let view = view();
        assert_eq!(view.hit(20, 10), Some(SkillHit::Tab(SkillTab::Active)));
        assert_eq!(view.hit(110, 30), Some(SkillHit::Tab(SkillTab::Passive)));
        assert_eq!(view.hit(10, 41), Some(SkillHit::Slot(0)));
        assert_eq!(view.hit(10 + 37, 41 + 35), Some(SkillHit::Slot(7)));
        assert_eq!(view.hit(5, 100), Some(SkillHit::Page));
        assert_eq!(view.hit(300, 100), None);
        assert_eq!(view.tab_names, ["s120", "s121"]);
    }

    #[test]
    fn listed_skills_follow_the_tab() {
        let view = view();
        assert_eq!(view.listed().iter().map(|s| s.id).collect::<Vec<_>>(), vec![3, 1177]);
        assert_eq!(view.slot_skill(1).map(|s| s.id), Some(1177));
        let passive = SkillsView { tab: SkillTab::Passive, ..view };
        assert_eq!(passive.listed().iter().map(|s| s.id).collect::<Vec<_>>(), vec![194]);
        assert_eq!(passive.slot_skill(1), None);
    }

    #[test]
    fn paint_draws_back_tabs_icons_outlines_and_the_states() {
        let mut view = view();
        view.selected = Some(1177);
        view.pressed = true;
        view.skills[0].toggled = true;
        view.skills[1].cool = Some(CoolFrame::Running(180));
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 66, &font());
        assert_eq!((list.quads[0].texture.as_str(), list.quads[0].y, list.quads[0].h), (SKILL_BACK, 66.0, 335.0));
        let tab: Vec<_> = list.quads.iter().filter(|q| q.texture == TAB_ACTIVE).collect();
        assert_eq!((tab[0].x, tab[0].y, tab[0].w), (12.0, 74.0, 94.0));
        let icon: Vec<_> = list.quads.iter().filter(|q| q.texture == "icon.skill0003").collect();
        assert_eq!((icon[0].x, icon[0].y, icon[0].w), (10.0, 107.0, 32.0));
        let back: Vec<_> = list.quads.iter().filter(|q| q.texture == ICON_BACK).collect();
        assert_eq!(back.len(), 2);
        assert_eq!((back[0].x, back[0].y, back[0].w), (9.0, 106.0, 34.0));
        let click: Vec<_> = list.quads.iter().filter(|q| q.texture == ICON_CLICK).collect();
        assert_eq!((click.len(), click[0].x), (1, 46.0));
        let selected: Vec<_> = list.quads.iter().filter(|q| q.texture == ITEM_CLICK).collect();
        assert_eq!((selected[0].x, selected[0].y, selected[0].w), (47.0, 107.0, 32.0));
        let toggle: Vec<_> = list.quads.iter().filter(|q| q.texture == TOGGLE_EFFECT).collect();
        assert_eq!((toggle[0].x, toggle[0].y), (10.0, 107.0));
        let cool = list.quads.iter().find(|q| q.texture == "L2_SkillTime.Rotation_Type.cooltime180").unwrap();
        assert_eq!((cool.x, cool.y, cool.w, cool.h), (47.0, 107.0, 32.0, 32.0));
        let order: Vec<&str> = list
            .quads
            .iter()
            .filter(|q| (q.x == 47.0 || q.x == 46.0) && q.y > 100.0)
            .map(|q| q.texture.as_str())
            .collect();
        assert_eq!(
            order[..4],
            [ITEM_CLICK, "icon.skill1177", ICON_BACK, ICON_CLICK]
        );
    }

    #[test]
    fn passive_skills_get_no_outline_and_no_pressed_frame() {
        let mut view = view();
        view.tab = SkillTab::Passive;
        view.selected = Some(194);
        view.pressed = true;
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 66, &font());
        assert_eq!(list.quads.iter().filter(|q| q.texture == ICON_BACK || q.texture == ICON_CLICK).count(), 0);
        assert_eq!(list.quads.iter().filter(|q| q.texture == "icon.skill0194").count(), 1);
    }

    #[test]
    fn the_end_flash_is_tinted_with_the_cool_time_color() {
        let mut view = view();
        view.skills[0].cool = Some(CoolFrame::Flash(2));
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 66, &font());
        let flash = list.quads.iter().find(|q| q.texture == "L2_SkillTime.CooltimeEnd.CooltimeEnd003").unwrap();
        assert_eq!(flash.color, cooltime::FLASH_COLOR.rgba());
    }
}
