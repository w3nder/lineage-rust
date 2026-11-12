use crate::child_contract::{ChildContract, ChildKind};
use crate::window::Rect;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;

pub const ROW_HEIGHT: i32 = 17;
pub const INDENT: i32 = 5;
pub const STATE_INDENT: u16 = 0x1f4;
pub const STATE_SELECTED: u16 = 0x240;
pub const STATE_HOVERED: u16 = 0x244;
pub const STATE_ROW_HEIGHT: u16 = 0x248;
pub const NORMAL: [&str; 3] = ["L2UI_CH3.Etc.textbox1", "L2UI_CH3.Etc.textbox2", "L2UI_CH3.Etc.textbox3"];
pub const DISABLED: [&str; 3] = ["L2UI_CH3.Etc.textbox1_disable", "L2UI_CH3.Etc.textbox2_disable", "L2UI_CH3.Etc.textbox3_disable"];
pub const SELECT: &str = "L2UI_CH3.ListCtrl.TextSelect";
pub const SELECT_END: &str = "L2UI_CH3.ListCtrl.TextSelect2";
pub const ARROW: &str = "L2UI_CH3.Button.downbutton";
pub const ARROW_DOWN: &str = "L2UI_CH3.Button.downbutton_down";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComboBox {
    pub rect: Rect,
    pub enabled: bool,
    pub entries: Vec<String>,
    pub selected: i32,
    pub hovered: i32,
    pub open: bool,
    pub text_height: i32,
    pub indent: i32,
    pub row_height: i32,
    open_height: i32,
}

impl ComboBox {
    pub fn new(rect: Rect, entries: Vec<String>, text_height: i32) -> Self {
        Self { rect, entries, text_height, indent: INDENT, row_height: ROW_HEIGHT, enabled: true, selected: -1, hovered: -1, open: false, open_height: rect.h }
    }

    pub fn from_contract(contract: &ChildContract, entries: Vec<String>, text_height: i32) -> Self {
        debug_assert_eq!(contract.kind, ChildKind::ComboBox);
        let rect = contract.rect;
        Self {
            rect,
            entries,
            text_height,
            indent: contract.field(STATE_INDENT) as i32,
            row_height: contract.field(STATE_ROW_HEIGHT) as i32,
            enabled: true,
            selected: contract.field(STATE_SELECTED) as i32,
            hovered: contract.field(STATE_HOVERED) as i32,
            open: false,
            open_height: rect.h,
        }
    }

    pub fn expanded_rect(&self) -> Rect {
        Rect::new(self.rect.x, self.rect.y, self.rect.w, if self.open { self.open_height } else { self.rect.h })
    }

    pub fn open_list(&mut self) {
        if self.enabled {
            self.open = true;
            self.open_height = self.rect.h + self.row_height * self.entries.len() as i32;
        }
    }

    pub fn clear_entries(&mut self) {
        self.entries.clear();
        self.selected = -1;
        self.hovered = -1;
    }

    pub fn lose_focus(&mut self) {
        self.open = false;
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        let local_x = (x as u16).wrapping_sub(self.rect.x as u16) as i32;
        let local_y = (y as u16).wrapping_sub(self.rect.y as u16) as i32;
        self.hovered = if local_x > 0 && local_x < self.rect.w
            && local_y > self.rect.h && local_y < self.expanded_rect().h
        {
            let index = if self.text_height == 0 { 0 } else { (local_y - self.rect.h).checked_div(self.row_height).unwrap_or(0) };
            index.min(self.entries.len() as i32 - 1)
        } else {
            -1
        };
    }

    pub fn mouse_down(&mut self) -> Option<i32> {
        if !self.enabled {
            return None;
        }
        if !self.open {
            self.open_list();
            return None;
        }
        let notification = if self.hovered >= 0 {
            self.selected = self.hovered;
            Some(self.selected)
        } else {
            None
        };
        self.lose_focus();
        notification
    }

    pub fn selected_text(&self) -> Option<&str> {
        usize::try_from(self.selected).ok().and_then(|index| self.entries.get(index)).map(String::as_str)
    }

    pub fn arrow_rect(&self) -> Rect {
        Rect::new(self.rect.x + self.rect.w - 16, self.rect.y + (self.rect.h as f32 / 2.0 - 7.0) as i32, 15, 15)
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, focused: bool, arrow_pressed: bool) {
        let x = origin.0 + self.rect.x;
        let y = origin.1 + self.rect.y;
        let expanded = self.expanded_rect();
        let mut body = DrawList {capture_text:list.capture_text,..Default::default()};
        let middle = (expanded.h as f32 / 2.0 - (self.row_height / 2) as f32) as i32;
        self.paint_row(&mut body, x, y + middle, if self.enabled { NORMAL } else { DISABLED });
        if let Some(text) = self.selected_text() {
            let color = Argb(if self.enabled { 0xffdcdcdc } else { 0xff7e7e7e });
            body.draw_text(x + self.indent, y + self.rect.h / 2 - self.text_height / 2, color, text, font);
        }
        if self.open && focused {
            for index in 0..self.entries.len() {
                self.paint_row(&mut body, x, y + self.rect.h + self.row_height * index as i32, NORMAL);
            }
            for (index, text) in self.entries.iter().enumerate() {
                let row_y = y + self.rect.h + self.row_height * index as i32;
                if self.hovered == index as i32 {
                    body.draw_texture(x + 1, row_y, self.rect.w - 32, 16, 0, 0, 16, 13, SELECT, 1.0);
                    body.draw_texture(x + self.rect.w - 31, row_y, 32, 16, 0, 0, 32, 13, SELECT_END, 1.0);
                }
                body.draw_text(x + self.indent, row_y + self.row_height / 2 - self.text_height / 2, Argb(0xffdcdcdc), text, font);
            }
        }
        let arrow = self.arrow_rect();
        body.draw_texture(origin.0 + arrow.x, origin.1 + arrow.y, arrow.w, arrow.h, 0, 0, arrow.w, arrow.h,
            if self.enabled && arrow_pressed { ARROW_DOWN } else { ARROW }, 1.0);
        list.append_clipped(body,crate::window::Rect::new(x,y,expanded.w,expanded.h));
    }

    fn paint_row(&self, list: &mut DrawList, x: i32, y: i32, textures: [&str; 3]) {
        for (dx, width, texture) in [(0, 8, textures[0]), (8, self.rect.w - 16, textures[1]), (self.rect.w - 8, 8, textures[2])] {
            list.draw_texture(x + dx, y, width, self.row_height, 0, 0, 8, ROW_HEIGHT, texture, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("font".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    #[test]
    fn contract_state_matches_the_constructor_defaults() {
        const CONTRACT: ChildContract = ChildContract {
            kind: ChildKind::ComboBox,
            rect: Rect::new(39, 61, 178, 17),
            id: 0xe433,
            vtable: 0,
            registration: (2, 0, 0),
            handles: &[],
            state: &[(0x1f4, 5), (0x240, 0xffff_ffff), (0x244, 0xffff_ffff), (0x248, 17)],
            textures: &[],
        };
        let from_contract = ComboBox::from_contract(&CONTRACT, Vec::new(), 13);
        assert_eq!(from_contract, ComboBox::new(Rect::new(39, 61, 178, 17), Vec::new(), 13));
        let tall = ChildContract { state: &[(0x1f4, 7), (0x248, 20)], ..CONTRACT };
        let mut custom = ComboBox::from_contract(&tall, vec!["a".into(), "b".into()], 13);
        assert_eq!((custom.indent, custom.row_height, custom.selected, custom.hovered), (7, 20, 0, 0));
        custom.open_list();
        assert_eq!(custom.expanded_rect().h, 17 + 40);
        let zero = ChildContract { state: &[(0x1f4, 5)], ..CONTRACT };
        let mut broken = ComboBox::from_contract(&zero, vec!["a".into()], 13);
        broken.open_list();
        broken.mouse_move(40, 80);
        assert_eq!(broken.hovered, -1);
    }

    fn combo() -> ComboBox {
        ComboBox::new(Rect::new(107, 59, 125, 17), vec!["A".into(), "B".into(), "C".into()], 13)
    }

    #[test]
    fn nccombobox_10006990_clears_entries_without_closing_or_resizing() {
        let mut c = combo();
        c.open_list();
        let rect = c.expanded_rect();
        c.selected = 1;
        c.hovered = 2;
        c.clear_entries();
        assert!(c.entries.is_empty());
        assert_eq!((c.selected, c.hovered), (-1, -1));
        assert!(c.open);
        assert_eq!(c.expanded_rect(), rect);
        c.entries.push("D".into());
        assert_eq!(c.expanded_rect(), rect);
        c.open_list();
        assert_eq!(c.expanded_rect().h, c.rect.h + ROW_HEIGHT);
        c.lose_focus();
        assert_eq!(c.expanded_rect(), c.rect);
    }

    #[test]
    fn nccombobox_10006bd0_starts_closed_without_selection_or_hover() {
        let c = combo();
        assert!(!c.open);
        assert_eq!((c.selected, c.hovered), (-1, -1));
        assert_eq!(c.expanded_rect(), c.rect);
        assert_eq!(c.selected_text(), None);
    }

    #[test]
    fn nccombobox_10006550_opens_instead_of_toggling_and_10005de0_restores_size() {
        let mut c = combo();
        c.open_list();
        assert_eq!(c.expanded_rect(), Rect::new(107, 59, 125, 68));
        c.open_list();
        assert!(c.open);
        c.selected = 1;
        c.hovered = 2;
        c.lose_focus();
        assert_eq!(c.expanded_rect(), c.rect);
        assert_eq!((c.selected, c.hovered), (1, 2));
    }

    #[test]
    fn nccombobox_10005cc0_checks_strict_bounds_and_every_row_boundary() {
        let mut c = combo();
        c.open_list();
        for (x, y, expected) in [
            (107, 77, -1), (232, 77, -1), (106, 77, -1),
            (108, 76, -1), (108, 77, 0), (108, 92, 0),
            (108, 93, 1), (108, 109, 1), (108, 110, 2),
            (108, 126, 2), (108, 127, -1), (108, 58, -1),
        ] {
            c.mouse_move(x, y);
            assert_eq!(c.hovered, expected, "({x},{y})");
        }
        c.text_height = 0;
        c.mouse_move(108, 110);
        assert_eq!(c.hovered, 0);
        c.lose_focus();
        c.mouse_move(108, 77);
        assert_eq!(c.hovered, -1);
    }

    #[test]
    fn nccombobox_10005d70_confirms_on_press_not_release_and_also_notifies_same_selection() {
        let mut c = combo();
        assert_eq!(c.mouse_down(), None);
        assert!(c.open);
        c.mouse_move(108, 93);
        assert_eq!(c.selected, -1);
        assert_eq!(c.mouse_down(), Some(1));
        assert_eq!(c.selected_text(), Some("B"));
        assert!(!c.open);
        assert_eq!(c.mouse_down(), None);
        c.mouse_move(108, 93);
        assert_eq!(c.mouse_down(), Some(1));
        c.open_list();
        c.mouse_move(0, 0);
        assert_eq!(c.mouse_down(), None);
        assert!(!c.open);
        assert_eq!(c.selected, 1);
    }

    #[test]
    fn nccombobox_disabled_empty_and_invalid_selection_paths() {
        let mut c = combo();
        c.enabled = false;
        c.open_list();
        assert_eq!(c.mouse_down(), None);
        assert!(!c.open);
        c.enabled = true;
        c.entries.clear();
        c.open_list();
        assert_eq!(c.expanded_rect(), c.rect);
        c.mouse_move(108, 77);
        assert_eq!(c.hovered, -1);
        assert_eq!(c.mouse_down(), None);
        c.selected = 99;
        assert_eq!(c.selected_text(), None);
        c.selected = -2;
        assert_eq!(c.selected_text(), None);
    }

    #[test]
    fn nccombobox_10005cc0_uses_unsigned_sixteen_bit_mouse_coordinates() {
        let mut c = combo();
        c.rect.x = -10;
        c.rect.y = -20;
        c.open_list();
        c.mouse_move(-9, -2);
        assert_eq!(c.hovered, 0);
        c.mouse_move(-11, -2);
        assert_eq!(c.hovered, -1);
    }

    #[test]
    fn nccombobox_10005f50_paints_three_slices_with_original_uvs_and_disabled_color() {
        let mut c = combo();
        c.selected = 1;
        for enabled in [true, false] {
            c.enabled = enabled;
            let mut list = DrawList::default();
            c.paint(&mut list, (10, 20), &font(), false, true);
            let textures = if enabled { NORMAL } else { DISABLED };
            for (texture, dx, width) in [(textures[0], 0, 8), (textures[1], 8, 109), (textures[2], 117, 8)] {
                let q = list.quads.iter().find(|q| q.texture == texture).unwrap();
                assert_eq!((q.x, q.y, q.w, q.h, q.uw, q.vh), ((117 + dx) as f32, 79.0, width as f32, 17.0, 8.0, 17.0));
            }
            let label = list.quads.iter().find(|q| q.texture == "font").unwrap();
            assert_eq!((label.x, label.y), (122.0, 81.0));
            assert_eq!(label.color, Argb(if enabled { 0xffdcdcdc } else { 0xff7e7e7e }).rgba());
            let arrow = list.quads.iter().find(|q| q.texture == if enabled { ARROW_DOWN } else { ARROW }).unwrap();
            assert_eq!((arrow.x, arrow.y, arrow.w, arrow.h), (226.0, 80.0, 15.0, 15.0));
        }
    }

    #[test]
    fn nccombobox_10005f50_draws_dropdown_only_open_and_focused_and_clips_highlight_end() {
        let mut c = combo();
        c.open_list();
        c.hovered = 1;
        for focused in [false, true] {
            let mut list = DrawList::default();
            c.paint(&mut list, (0, 0), &font(), focused, false);
            let left: Vec<_> = list.quads.iter().filter(|q| q.texture == NORMAL[0]).collect();
            assert_eq!(left.len(), if focused { 4 } else { 1 });
            assert_eq!(left[0].y, 85.0);
            if focused {
                assert_eq!(left.iter().skip(1).map(|q| q.y).collect::<Vec<_>>(), vec![76.0, 93.0, 110.0]);
                let highlight = list.quads.iter().find(|q| q.texture == SELECT).unwrap();
                assert_eq!((highlight.x, highlight.y, highlight.w, highlight.h, highlight.uw, highlight.vh), (108.0, 93.0, 93.0, 16.0, 16.0, 13.0));
                let end = list.quads.iter().find(|q| q.texture == SELECT_END).unwrap();
                assert_eq!((end.x, end.y, end.w, end.h, end.uw, end.vh), (201.0, 93.0, 31.0, 16.0, 31.0, 13.0));
            } else {
                assert!(!list.quads.iter().any(|q| q.texture == SELECT || q.texture == "font"));
            }
        }
    }

    #[test]
    fn nccombobox_10005f50_preserves_uvs_when_clipping_long_labels_and_invalid_selection_has_no_text() {
        let mut c = combo();
        c.rect.w = 126;
        c.entries[0] = "A".repeat(40);
        c.selected = 0;
        let mut list = DrawList::default();
        c.paint(&mut list, (0, 0), &font(), false, false);
        for q in &list.quads {
            assert!(q.x >= 107.0 && q.y >= 59.0 && q.x + q.w <= 233.0 && q.y + q.h <= 76.0);
        }
        let glyphs: Vec<_> = list.quads.iter().filter(|q| q.texture == "font").collect();
        let last = glyphs.last().unwrap();
        assert_eq!((last.x + last.w, last.w, last.uw), (233.0, 1.0, 1.0));
        c.selected = 40;
        let mut invalid = DrawList::default();
        c.paint(&mut invalid, (0, 0), &font(), false, false);
        assert!(!invalid.quads.iter().any(|q| q.texture == "font"));
    }
}
