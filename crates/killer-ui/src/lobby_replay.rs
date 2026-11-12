use crate::button::{CheckBox, TextButton};
use crate::child_contract::{ChildContract, ChildKind};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::list_ctrl::{ColumnContract, ListCell, ListContract, ListCtrl, ListRow};
use crate::window::Rect;

pub const SIZE: (i32, i32) = (316, 350);
pub const BACK: &str = "L2UI_CH3.ReplayListWnd.replay_back";
pub const DIRECTORY: &str = "..\\REPLAY\\";
pub const PATTERN: &str = "..\\REPLAY\\*.L2R";
pub const EXTENSION: &str = ".L2R";
pub const LIST_ID: u32 = 0xf030;
pub const LOAD_ID: u32 = 0xf031;
pub const DELETE_ID: u32 = 0xf032;
pub const CLOSE_ID: u32 = 0xf033;
pub const OPTION_ID: u32 = 0xf035;
pub const CHAT_ID: u32 = 0xf036;
pub const OPTION_RECT: Rect = Rect::new(11, 295, 140, 16);
pub const CHAT_RECT: Rect = Rect::new(161, 295, 140, 16);
pub const LOAD_RECT: Rect = Rect::new(41, 317, 76, 23);
pub const DELETE_RECT: Rect = Rect::new(121, 317, 76, 23);
pub const CLOSE_RECT: Rect = Rect::new(201, 317, 76, 23);
pub const BUTTON_NORMAL: &str = "L2UI_CH3.Button.Btn1_normal";
pub const BUTTON_DOWN: &str = "L2UI_CH3.Button.Btn1_normalOn";
pub const SYSSTRINGS: [u32; 7] = [416, 1250, 1249, 1253, 140, 1272, 141];
pub const KEY_ENTER: u32 = 0xd;
pub const KEY_ESCAPE: u32 = 0x1b;
pub const NAME_COLUMN: usize = 1;

const TABS: &[(u16, &str)] = &[
    (0x18, "L2UI_CH3.ListCtrl.tab1"),
    (0x818, "L2UI_CH3.ListCtrl.tab2"),
    (0x1018, "L2UI_CH3.ListCtrl.tab3"),
    (0x1818, "L2UI_CH3.ListCtrl.tab1_down"),
    (0x2018, "L2UI_CH3.ListCtrl.tab2_down"),
    (0x2818, "L2UI_CH3.ListCtrl.tab3_down"),
];

pub const CHILD_E8: ChildContract = ChildContract {
    kind: ChildKind::ListCtrl,
    rect: Rect::new(8, 11, 300, 273),
    id: LIST_ID,
    vtable: 0x101c9650,
    registration: (0x2, 0x0, 0x0),
    handles: &[0x230, 0x234, 0x238],
    state: &[
        (0x23c, 0xf),
        (0x240, 0x100),
        (0x244, 0x2),
        (0x250, 0xffffffff),
        (0x254, 0xffffffff),
        (0x258, 0xf),
        (0x25c, 0x13),
        (0x264, 0x3),
        (0x268, 0x1),
        (0x26c, 0x11),
    ],
    textures: &[],
};

pub const COLUMNS_E8: &[ColumnContract] = &[
    ColumnContract { width: 32, height: 19, sysstring: 416, flag_c: 1, flag_10: 0, textures: TABS },
    ColumnContract { width: 268, height: 19, sysstring: 1250, flag_c: 1, flag_10: 0, textures: TABS },
];

pub const LIST_E8: ListContract = ListContract { child: CHILD_E8, columns: COLUMNS_E8 };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    Load,
    Delete,
    Close,
    Option,
    Chat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayAction {
    Load { name: String, option: bool, chat: bool },
    Delete { path: String },
    Refresh,
    Back,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReplayLabels {
    pub header: [String; 2],
    pub option: String,
    pub chat: String,
    pub load: String,
    pub delete: String,
    pub close: String,
}

impl ReplayLabels {
    pub fn from_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> ReplayLabels {
        let text = |id: u32| lookup(id).unwrap_or_default();
        ReplayLabels {
            header: [text(SYSSTRINGS[0]), text(SYSSTRINGS[1])],
            option: text(SYSSTRINGS[2]),
            chat: text(SYSSTRINGS[3]),
            load: text(SYSSTRINGS[4]),
            delete: text(SYSSTRINGS[5]),
            close: text(SYSSTRINGS[6]),
        }
    }
}

pub fn name_of(file: &str) -> Option<&str> {
    let dot = file.find('.')?;
    Some(&file[..dot])
}

pub fn delete_path(name: &str) -> String {
    format!("{DIRECTORY}{name}{EXTENSION}")
}

pub fn matches_pattern(file: &str) -> bool {
    file.len() > EXTENSION.len() && file.to_uppercase().ends_with(EXTENSION)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayWnd {
    pub screen: (i32, i32),
    pub labels: ReplayLabels,
    pub list: ListCtrl,
    pub option: CheckBox,
    pub chat: CheckBox,
    pub pressed: Option<Hit>,
    pub visible: bool,
}

impl ReplayWnd {
    pub fn new(screen: (i32, i32), labels: ReplayLabels) -> ReplayWnd {
        let option = CheckBox::new(OPTION_RECT, &labels.option);
        let chat = CheckBox::new(CHAT_RECT, &labels.chat);
        let mut list = ListCtrl::from_contract(&LIST_E8);
        list.sortable = false;
        ReplayWnd { screen, labels, list, option, chat, pressed: None, visible: false }
    }

    pub fn origin(&self) -> (i32, i32) {
        (
            (self.screen.0 as f32 * 0.5 - 158.0) as i32,
            (self.screen.1 as f32 * 0.5 - 175.0) as i32,
        )
    }

    pub fn fill(&mut self, files: &[String]) {
        self.list.clear();
        let mut matching: Vec<&String> = files.iter().filter(|file| matches_pattern(file)).collect();
        matching.sort_by_cached_key(|file| file.to_uppercase());
        for (index, name) in matching.into_iter().filter_map(|file| name_of(file)).enumerate() {
            self.list.add_item(ListRow { cells: vec![ListCell::text(&index.to_string()), ListCell::text(name)] });
        }
    }

    pub fn show(&mut self, files: &[String]) {
        self.visible = true;
        self.fill(files);
    }

    pub fn selected_name(&self) -> Option<String> {
        let index = self.list.selected()?;
        let row = self.list.rows.get(index)?;
        let name = &row.cells.get(NAME_COLUMN)?.text;
        (!name.is_empty()).then(|| name.clone())
    }

    pub fn load(&self) -> Option<ReplayAction> {
        let name = self.selected_name()?;
        Some(ReplayAction::Load { name, option: self.option.checked, chat: self.chat.checked })
    }

    pub fn delete(&self) -> Option<ReplayAction> {
        match self.selected_name() {
            Some(name) => Some(ReplayAction::Delete { path: delete_path(&name) }),
            None => Some(ReplayAction::Refresh),
        }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<Hit> {
        let (ox, oy) = self.origin();
        let (x, y) = (x - ox, y - oy);
        if LOAD_RECT.contains(x, y) {
            Some(Hit::Load)
        } else if DELETE_RECT.contains(x, y) {
            Some(Hit::Delete)
        } else if CLOSE_RECT.contains(x, y) {
            Some(Hit::Close)
        } else if self.option.contains(x, y) {
            Some(Hit::Option)
        } else if self.chat.contains(x, y) {
            Some(Hit::Chat)
        } else {
            None
        }
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        let (ox, oy) = self.origin();
        self.pressed = self.hit(x, y);
        match self.pressed {
            Some(Hit::Option) => {
                self.option.mouse_down();
            }
            Some(Hit::Chat) => {
                self.chat.mouse_down();
            }
            _ => {
                let inside = Rect::new(CHILD_E8.rect.x, CHILD_E8.rect.y, CHILD_E8.rect.w, CHILD_E8.rect.h).contains(x - ox, y - oy);
                if inside {
                    self.list.mouse_down(x - ox, y - oy);
                }
            }
        }
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<ReplayAction> {
        let (ox, oy) = self.origin();
        self.list.mouse_up(x - ox, y - oy);
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            Hit::Load => self.load(),
            Hit::Delete => self.delete(),
            Hit::Close => Some(ReplayAction::Back),
            Hit::Option | Hit::Chat => None,
        }
    }

    pub fn double_click(&mut self, x: i32, y: i32) -> Option<ReplayAction> {
        let (ox, oy) = self.origin();
        self.list.double_click(x - ox, y - oy);
        if self.list.notices.drain(..).any(|notice| notice.id == LIST_ID) { self.load() } else { None }
    }

    pub fn key(&mut self, key: u32) -> Option<ReplayAction> {
        match key {
            KEY_ENTER => self.load(),
            KEY_ESCAPE => Some(ReplayAction::Back),
            _ => None,
        }
    }

    pub fn wheel(&mut self, delta: i16) {
        self.list.wheel(delta);
    }

    pub fn paint(&self, list: &mut DrawList, font: &BitmapFont, lookup: &dyn Fn(i32) -> Option<String>) {
        let (ox, oy) = self.origin();
        list.draw_texture(ox, oy, SIZE.0, SIZE.1, 0, 0, SIZE.0, SIZE.1, BACK, 1.0);
        self.list.paint_header(list, (ox, oy), font, lookup);
        self.list.paint(list, (ox, oy), font);
        self.option.paint(list, ox, oy, font);
        self.chat.paint(list, ox, oy, font);
        let pressed = |hit: Hit| self.pressed == Some(hit);
        TextButton::new(LOAD_RECT, BUTTON_NORMAL, BUTTON_DOWN, &self.labels.load).paint(list, ox, oy, pressed(Hit::Load), font);
        TextButton::new(DELETE_RECT, BUTTON_NORMAL, BUTTON_DOWN, &self.labels.delete).paint(list, ox, oy, pressed(Hit::Delete), font);
        TextButton::new(CLOSE_RECT, BUTTON_NORMAL, BUTTON_DOWN, &self.labels.close).paint(list, ox, oy, pressed(Hit::Close), font);
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

    fn window() -> ReplayWnd {
        let mut window = ReplayWnd::new((1024, 768), ReplayLabels::from_sysstrings(|id| Some(format!("s{id}"))));
        window.show(&["b.l2r".to_string(), "a.L2R".to_string(), "x.txt".to_string(), "Rep00001.l2r".to_string()]);
        window
    }

    #[test]
    fn layout_matches_the_executed_oncreate() {
        assert_eq!(CHILD_E8.rect, Rect::new(8, 11, 300, 273));
        assert_eq!(COLUMNS_E8.iter().map(|c| c.width).sum::<i32>(), CHILD_E8.rect.w);
        assert_eq!((OPTION_RECT, CHAT_RECT), (Rect::new(11, 295, 140, 16), Rect::new(161, 295, 140, 16)));
        assert_eq!((LOAD_RECT, DELETE_RECT, CLOSE_RECT).0.x + 80, DELETE_RECT.x);
        assert_eq!(SYSSTRINGS, [416, 1250, 1249, 1253, 140, 1272, 141]);
    }

    #[test]
    fn window_is_centered_like_login_wnd() {
        let wnd = window();
        assert_eq!(wnd.origin(), (354, 209));
    }

    #[test]
    fn list_holds_sorted_names_without_extension_and_index_column() {
        let wnd = window();
        let rows: Vec<(String, String)> = wnd.list.rows.iter().map(|row| (row.cells[0].text.clone(), row.cells[1].text.clone())).collect();
        assert_eq!(rows, vec![("0".into(), "a".into()), ("1".into(), "b".into()), ("2".into(), "Rep00001".into())]);
    }

    #[test]
    fn load_needs_a_selection_and_carries_the_checkbox_states() {
        let mut wnd = window();
        assert_eq!(wnd.load(), None);
        let (ox, oy) = wnd.origin();
        wnd.mouse_down(ox + 30, oy + 11 + 19 + 17 + 5);
        wnd.mouse_up(ox + 30, oy + 11 + 19 + 17 + 5);
        assert_eq!(wnd.selected_name().as_deref(), Some("b"));
        wnd.mouse_down(ox + CHAT_RECT.x + 3, oy + CHAT_RECT.y + 3);
        wnd.mouse_up(ox + CHAT_RECT.x + 3, oy + CHAT_RECT.y + 3);
        assert_eq!(wnd.load(), Some(ReplayAction::Load { name: "b".into(), option: false, chat: true }));
        assert_eq!(wnd.key(KEY_ENTER), wnd.load());
        assert_eq!(wnd.key(KEY_ESCAPE), Some(ReplayAction::Back));
        assert_eq!(wnd.key(0x41), None);
    }

    #[test]
    fn delete_builds_the_engine_path_and_buttons_need_press_and_release_inside() {
        let mut wnd = window();
        let (ox, oy) = wnd.origin();
        wnd.mouse_down(ox + 30, oy + 11 + 19 + 5);
        wnd.mouse_up(ox + 30, oy + 11 + 19 + 5);
        assert_eq!(wnd.delete(), Some(ReplayAction::Delete { path: "..\\REPLAY\\a.L2R".into() }));
        let (bx, by) = (ox + DELETE_RECT.x + 3, oy + DELETE_RECT.y + 3);
        wnd.mouse_down(bx, by);
        assert_eq!(wnd.mouse_up(bx + 200, by), None);
        wnd.mouse_down(bx, by);
        assert!(matches!(wnd.mouse_up(bx, by), Some(ReplayAction::Delete { .. })));
        wnd.mouse_down(ox + CLOSE_RECT.x + 1, oy + CLOSE_RECT.y + 1);
        assert_eq!(wnd.mouse_up(ox + CLOSE_RECT.x + 1, oy + CLOSE_RECT.y + 1), Some(ReplayAction::Back));
    }

    #[test]
    fn the_name_is_cut_at_the_first_dot_like_0x100f7d58_and_a_delete_without_a_row_only_refreshes() {
        let mut wnd = window();
        wnd.fill(&["Rep.1.L2R".to_string()]);
        assert_eq!(wnd.list.rows[0].cells[1].text, "Rep");
        assert_eq!(wnd.delete(), Some(ReplayAction::Refresh));
    }

    #[test]
    fn double_click_loads_the_row() {
        let mut wnd = window();
        let (ox, oy) = wnd.origin();
        assert_eq!(wnd.double_click(ox + 30, oy + 11 + 19 + 17 * 2 + 3), Some(ReplayAction::Load { name: "Rep00001".into(), option: false, chat: false }));
        assert_eq!(wnd.double_click(ox + 30, oy + 340), None);
    }

    #[test]
    fn paint_draws_back_header_rows_checks_and_three_buttons() {
        let wnd = window();
        let mut draws = DrawList::default();
        wnd.paint(&mut draws, &font(), &|id| Some(format!("s{id}")));
        let textures: Vec<&str> = draws.quads.iter().map(|quad| quad.texture.as_str()).collect();
        assert_eq!(textures[0], BACK);
        assert_eq!(textures.iter().filter(|name| **name == BUTTON_NORMAL).count(), 3);
        assert!(textures.contains(&"L2UI_CH3.ListCtrl.tab1"));
    }

    #[test]
    fn a_header_click_is_taken_and_never_sorts_the_replay_list_0x100f76f0() {
        let mut wnd = window();
        let (ox, oy) = wnd.origin();
        let before: Vec<String> = wnd.list.rows.iter().map(|row| row.cells[1].text.clone()).collect();
        let (x, y) = (ox + CHILD_E8.rect.x + 40, oy + CHILD_E8.rect.y + 3);
        for _ in 0..2 {
            wnd.mouse_down(x, y);
            wnd.mouse_up(x, y);
        }
        let after: Vec<String> = wnd.list.rows.iter().map(|row| row.cells[1].text.clone()).collect();
        assert_eq!(before, after);
        assert!(wnd.list.sort_forward.iter().all(|forward| *forward));
        let mut sorting = window();
        sorting.list.sortable = true;
        for _ in 0..2 {
            sorting.mouse_down(x, y);
            sorting.mouse_up(x, y);
        }
        let reordered: Vec<String> = sorting.list.rows.iter().map(|row| row.cells[1].text.clone()).collect();
        assert_ne!(before, reordered);
    }
}
