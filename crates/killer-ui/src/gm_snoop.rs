use crate::button::{CheckBox, TextButton};
use crate::color::Argb;
use crate::draw::DrawList;
use crate::edit_box::FontMetrics;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::frame_contract::{FrameContract, FrameTitle};
use crate::text_list::TextList;
use crate::window::{Rect, WindowId};

pub const COUNT: usize = 4;
pub const SIZE: (i32, i32) = (300, 301);
pub const POSITION: (i32, i32) = (15, 38);
pub const BACK: &str = "L2UI.etcwnd.Petitionback";
pub const BACK_ORIGIN: (i32, i32) = (0, 21);
pub const BACK_SIZE: (i32, i32) = (300, 280);
pub const FRAME_ID: u32 = 0xFCA0B5;
pub const LIST_ID: u32 = 0xFCA0B1;
pub const CLOSE_ID: u32 = 0xFCA0B4;
pub const FILTER_ID: u32 = 0xF0F01450;
pub const GRIP: Rect = Rect::new(0, 0, 300, 21);
pub const LIST_RECT: Rect = Rect::new(15, 38, 282, 195);
pub const LIST_MAX_LINES: usize = 200;
pub const LIST_VISIBLE: i32 = 16;
pub const CLOSE_RECT: Rect = Rect::new(110, 270, 64, 19);
pub const CLOSE_NORMAL: &str = "L2UI.control.DefaultButton";
pub const CLOSE_DOWN: &str = "L2UI.control.DefaultButton_click";
pub const CLOSE_SYSSTRING: u32 = 141;
pub const TITLE_SYSSTRING: u32 = 693;
pub const WINDOWS_FULL_SYSMSG: u32 = 0x322;
pub const FILTER_SIZE: (i32, i32) = (64, 12);
pub const FILTERS: [(u32, i32, i32); 7] =
    [(0x16A, 15, 240), (0x168, 84, 240), (0x169, 153, 240), (0x167, 222, 240), (0x16B, 15, 254), (0x17F, 84, 254), (0x250, 153, 254)];
pub const WINDOW_IDS: [WindowId; COUNT] =
    [WindowId(0x10000 + 40), WindowId(0x10000 + 41), WindowId(0x10000 + 42), WindowId(0x10000 + 43)];

const FRAME: FrameContract = FrameContract {
    rect: GRIP,
    id: FRAME_ID,
    mode: FrameMode::Title,
    title: FrameTitle::None,
    close_button: false,
    minimize_button: false,
};

pub fn filter_of(kind: i32) -> Option<usize> {
    match kind {
        0 => Some(5),
        1 => Some(3),
        2 => Some(4),
        3 => Some(2),
        4 => Some(1),
        8 => Some(0),
        9 => Some(6),
        _ => None,
    }
}

pub fn line_color(kind: i32) -> Option<Argb> {
    match kind {
        1 => Some(Argb(0xFFFF7200)),
        3 => Some(Argb(0xFF00FF00)),
        4 => Some(Argb(0xFF7D77FF)),
        2 => Some(Argb(0xFFFF00FF)),
        8 => Some(Argb(0xFFEAA5F5)),
        9 => Some(Argb(0xFF77FF99)),
        0 => Some(Argb(0xFFDCDCDC)),
        _ => None,
    }
}

pub fn console_line(name: &str, text: &str, kind: i32) -> Option<(String, Argb)> {
    Some((format!("{name}: {text}"), line_color(kind)?))
}

pub fn title(prefix: &str, target: &str) -> String {
    format!("{prefix} - {target}")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnoopHit {
    Close,
    Filter(usize),
}

#[derive(Clone, Debug)]
pub struct SnoopWnd {
    pub frame: FrameCtrl,
    pub list: TextList,
    pub close: TextButton,
    pub filters: [CheckBox; 7],
    pub snoop_id: i32,
    pressed: Option<SnoopHit>,
}

impl SnoopWnd {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        let filters = FILTERS.map(|(label, x, y)| {
            let mut check = CheckBox::new(Rect::new(x, y, FILTER_SIZE.0, FILTER_SIZE.1), &text(label));
            check.checked = true;
            check
        });
        Self {
            frame: FRAME.build(text),
            list: TextList::new(LIST_ID, LIST_RECT, LIST_MAX_LINES, LIST_VISIBLE, true),
            close: TextButton::new(CLOSE_RECT, CLOSE_NORMAL, CLOSE_DOWN, &text(CLOSE_SYSSTRING)),
            filters,
            snoop_id: -1,
            pressed: None,
        }
    }

    pub fn set_title(&mut self, title: &str) {
        self.frame.title = title.to_string();
    }

    pub fn reset(&mut self) {
        self.list.clear();
        self.snoop_id = -1;
    }

    pub fn add_line(&mut self, text: &str, color: Argb, kind: i32, font: &BitmapFont) {
        let Some(filter) = filter_of(kind) else { return };
        if self.filters[filter].checked {
            self.list.add_string(text, color, &FontMetrics(font));
        }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<SnoopHit> {
        if self.close.contains(x, y) {
            return Some(SnoopHit::Close);
        }
        self.filters.iter().position(|check| check.contains(x, y)).map(SnoopHit::Filter)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<SnoopHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        if let Some(SnoopHit::Filter(index)) = hit {
            self.filters[index].mouse_down();
        }
        hit
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<i32> {
        let pressed = self.pressed.take()?;
        if pressed != SnoopHit::Close || self.hit(x, y) != Some(pressed) {
            return None;
        }
        let snoop_id = self.snoop_id;
        self.reset();
        Some(snoop_id)
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        let start = list.quads.len();
        list.draw_texture(x + BACK_ORIGIN.0, y + BACK_ORIGIN.1, BACK_SIZE.0, BACK_SIZE.1, 0, 0, BACK_SIZE.0, BACK_SIZE.1, BACK, alpha);
        list.clip_from(start, Rect::new(x, y, SIZE.0, SIZE.1));
        FRAME.paint(&self.frame, list, origin, font, alpha);
        self.list.paint(list, origin, font);
        self.close.paint(list, x, y, self.pressed == Some(SnoopHit::Close), font);
        for check in &self.filters {
            let start = list.quads.len();
            check.paint(list, x, y, font);
            list.clip_from(start, check.rect.offset(x, y));
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opened {
    Already,
    Window(usize),
    Full,
}

#[derive(Clone, Debug)]
pub struct SnoopManager {
    pub windows: [SnoopWnd; COUNT],
    title_prefix: String,
}

impl SnoopManager {
    pub fn new(lookup: impl Fn(u32) -> Option<String> + Copy) -> Self {
        Self {
            windows: std::array::from_fn(|_| SnoopWnd::new(lookup)),
            title_prefix: lookup(TITLE_SYSSTRING).unwrap_or_default(),
        }
    }

    pub fn open_snoop(&mut self, snoop_id: i32, target: &str) -> Opened {
        if self.windows.iter().any(|window| window.snoop_id == snoop_id) {
            return Opened::Already;
        }
        match self.windows.iter().position(|window| window.snoop_id == -1) {
            Some(index) => {
                self.windows[index].snoop_id = snoop_id;
                self.windows[index].set_title(&title(&self.title_prefix, target));
                Opened::Window(index)
            }
            None => Opened::Full,
        }
    }

    pub fn add_line(&mut self, snoop_id: i32, text: &str, color: Argb, kind: i32, font: &BitmapFont) -> Vec<usize> {
        let mut shown = Vec::new();
        for (index, window) in self.windows.iter_mut().enumerate() {
            if window.snoop_id == snoop_id {
                window.add_line(text, color, kind, font);
                shown.push(index);
            }
        }
        shown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn manager() -> SnoopManager {
        SnoopManager::new(|id| Some(format!("s{id}")))
    }

    #[test]
    fn the_window_is_made_as_the_oncreate_0x100e1b50_makes_it() {
        let window = SnoopWnd::new(|id| Some(format!("s{id}")));
        assert_eq!((window.list.rect, window.list.max_lines, window.list.visible), (Rect::new(15, 38, 282, 195), 200, 16));
        assert_eq!(window.close.rect, Rect::new(110, 270, 64, 19));
        assert_eq!(window.close.label, "s141");
        assert!(!window.frame.close_button && !window.frame.minimize_button);
        assert_eq!(window.snoop_id, -1);
        assert!(window.filters.iter().all(|check| check.checked));
        assert_eq!(window.filters[3].rect, Rect::new(222, 240, 64, 12));
        assert_eq!(window.filters[6].rect, Rect::new(153, 254, 64, 12));
        assert_eq!(window.filters.iter().map(|check| check.label.as_str()).collect::<Vec<_>>(), ["s362", "s360", "s361", "s359", "s363", "s383", "s592"]);
    }

    #[test]
    fn the_type_picks_the_check_box_as_in_addline_0x100e0790() {
        let kinds = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, -1];
        assert_eq!(kinds.map(filter_of), [Some(5), Some(3), Some(4), Some(2), Some(1), None, None, None, Some(0), Some(6), None, None]);
        let mut window = SnoopWnd::new(|_| None);
        window.filters[5].checked = false;
        window.add_line("a", Argb(1), 0, &font());
        window.add_line("b", Argb(1), 1, &font());
        window.add_line("c", Argb(1), 5, &font());
        assert_eq!(window.list.lines().iter().map(|line| line.text.as_str()).collect::<Vec<_>>(), ["b"]);
    }

    #[test]
    fn the_console_colors_the_line_by_its_chat_type_0x10069bf0() {
        assert_eq!(console_line("Ana", "oi", 1), Some(("Ana: oi".into(), Argb(0xFFFF7200))));
        let colors = [0, 1, 2, 3, 4, 8, 9].map(|kind| line_color(kind).map(|color| color.0));
        assert_eq!(colors, [0xFFDCDCDC, 0xFFFF7200, 0xFFFF00FF, 0xFF00FF00, 0xFF7D77FF, 0xFFEAA5F5, 0xFF77FF99].map(Some));
        assert_eq!(console_line("Ana", "oi", 5), None);
    }

    #[test]
    fn the_manager_gives_the_first_free_window_or_reports_all_four_in_use_0x100e09c0() {
        let mut manager = manager();
        assert_eq!(manager.open_snoop(10, "Bo"), Opened::Window(0));
        assert_eq!(manager.windows[0].frame.title, "s693 - Bo");
        assert_eq!(manager.open_snoop(10, "Bo"), Opened::Already);
        for id in 11..14 {
            assert!(matches!(manager.open_snoop(id, "x"), Opened::Window(_)));
        }
        assert_eq!(manager.open_snoop(99, "y"), Opened::Full);
        assert_eq!(manager.add_line(11, "t", Argb(1), 0, &font()), vec![1]);
        assert_eq!(manager.add_line(99, "t", Argb(1), 0, &font()), Vec::<usize>::new());
    }

    #[test]
    fn the_close_button_resets_the_window_and_gives_the_id_to_tell_the_server() {
        let mut manager = manager();
        manager.open_snoop(42, "Bo");
        manager.add_line(42, "t", Argb(1), 0, &font());
        let window = &mut manager.windows[0];
        let (x, y) = (CLOSE_RECT.x + 3, CLOSE_RECT.y + 3);
        window.mouse_down(x, y);
        assert_eq!(window.mouse_up(x + 200, y), None);
        window.mouse_down(x, y);
        assert_eq!(window.mouse_up(x, y), Some(42));
        assert_eq!((window.snoop_id, window.list.lines().len()), (-1, 0));
        let (fx, fy) = (FILTERS[0].1 + 2, FILTERS[0].2 + 2);
        window.mouse_down(fx, fy);
        window.mouse_up(fx, fy);
        assert!(!window.filters[0].checked);
    }

    #[test]
    fn paint_draws_the_panel_under_the_title_bar_before_the_children() {
        let manager = manager();
        let mut list = DrawList::default();
        manager.windows[0].paint(&mut list, (15, 38), &font(), 1.0);
        let first = &list.quads[0];
        assert_eq!((first.texture.as_str(), first.x, first.y, first.w, first.h), (BACK, 15.0, 59.0, 300.0, 280.0));
        assert!(list.quads.iter().any(|quad| quad.texture == CLOSE_NORMAL));
    }
}
