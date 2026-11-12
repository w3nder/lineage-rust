#[path = "olympiad_control_controls_generated.rs"]
mod control_controls;
#[path = "olympiad_control_layout_generated.rs"]
mod control_layout;
#[path = "olympiad_player_layout_generated.rs"]
mod player_layout;
#[path = "olympiad_target_controls_generated.rs"]
mod target_controls;
#[path = "olympiad_target_layout_generated.rs"]
mod target_layout;

use crate::button::TextButton;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::window::{Rect, WindowId};

pub const CONTROL_ID: WindowId = WindowId::OLYMPIAD_CONTROL;
pub const PLAYER_IDS: [WindowId; 2] = [WindowId::OLYMPIAD_PLAYER_1, WindowId::OLYMPIAD_PLAYER_2];
pub const ABNORMAL_IDS: [WindowId; 2] = [WindowId::OLYMPIAD_ABNORMAL_1, WindowId::OLYMPIAD_ABNORMAL_2];
pub const TARGET_ID: WindowId = WindowId::OLYMPIAD_TARGET;
pub const CONTROL_SIZE: (i32, i32) = control_layout::SIZE;
pub const PLAYER_SIZE: (i32, i32) = player_layout::SIZE;
pub const TARGET_SIZE: (i32, i32) = target_layout::SIZE;
pub const PLAYER_EXPANDED_HEIGHT: i32 = 0x7e;
pub const ABNORMAL_SIZE: (i32, i32) = (252, 50);
pub const ABNORMAL_SHIFT: i32 = 0x50;
pub const CONTROL_BACK: &str = "L2UI_CH3.Olympiad.olympiad_back";
pub const BACK_PARTS: [&str; 3] = [
    "L2UI_CH3.SmallWnd.SmallWindow_Back1",
    "L2UI_CH3.SmallWnd.SmallWindow_Back2",
    "L2UI_CH3.SmallWnd.SmallWindow_Back3",
];
pub const EXPANDED_BACK_PARTS: [&str; 3] = [
    "L2UI_CH3.SmallWnd.SmallWindow3_Back1",
    "L2UI_CH3.SmallWnd.SmallWindow3_Back2",
    "L2UI_CH3.SmallWnd.SmallWindow3_Back3",
];
pub const CP_BAR: &str = "L2UI_CH3.SmallWnd.CpBar";
pub const HP_BAR: &str = "L2UI_CH3.SmallWnd.HpBar";
pub const CP_BAR_BACK: &str = "L2UI_CH3.SmallWnd.CpBar_back";
pub const HP_BAR_BACK: &str = "L2UI_CH3.SmallWnd.HpBar_back";
pub const EXPAND_BUTTON: &str = "L2UI_CH3.TargetWnd.Target_InfoButton";
pub const EXPAND_BUTTON_DOWN: &str = "L2UI_CH3.TargetWnd.Target_InfoButton_Down";
pub const EXPAND_RECT: Rect = Rect::new(1, 6, 11, 13);
pub const PLAYER_BAR_WIDTH: i32 = 326;
pub const TARGET_BAR_WIDTH: i32 = 150;
pub const CP_BAR_Y: i32 = 26;
pub const HP_BAR_Y: i32 = 33;
pub const BAR_HEIGHT: i32 = 6;
pub const BAR_X: i32 = 16;
pub const NAME_POSITION: (i32, i32) = (25, 6);
pub const NAME_COLOR: Argb = Argb(0xFFDCDCDC);
pub const LOG_ROWS: usize = 5;
pub const LOG_X: i32 = 18;
pub const LOG_FIRST_Y: i32 = 45;
pub const LOG_STEP: i32 = 15;
pub const SIDE_FIRST: i32 = 1;
pub const SIDE_SECOND: i32 = 2;
pub const NO_OBJECT: i32 = -1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OlympiadUser {
    pub object_id: i32,
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub cp: i32,
    pub max_cp: i32,
}

impl Default for OlympiadUser {
    fn default() -> Self {
        Self { object_id: NO_OBJECT, name: String::new(), hp: 0, max_hp: 0, cp: 0, max_cp: 0 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OlympiadUserInfoView {
    pub side: i32,
    pub user: OlympiadUser,
}

pub fn control_default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 as f32 * 0.5 - 38.0) as i32, 0)
}

pub const CONTROL_INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 3, x: 102, y: 0 };

pub fn player_default_position(index: usize, viewport: (i32, i32)) -> (i32, i32) {
    if index == 0 { (0, 0) } else { ((viewport.0 as f32 - 348.0) as i32, 0) }
}

pub fn player_initial_position(index: usize) -> crate::initial_position::InitialPosition {
    crate::initial_position::InitialPosition {
        enabled: true,
        horizontal_mode: index as i32,
        vertical_mode: 3,
        x: if index == 0 { 0 } else { 348 },
        y: 0,
    }
}

pub fn abnormal_default_position(index: usize, viewport: (i32, i32)) -> (i32, i32) {
    if index == 0 { (0, 47) } else { ((viewport.0 as f32 - 348.0) as i32, 47) }
}

pub fn abnormal_initial_position(index: usize) -> crate::initial_position::InitialPosition {
    crate::initial_position::InitialPosition {
        enabled: true,
        horizontal_mode: index as i32,
        vertical_mode: 3,
        x: if index == 0 { 0 } else { 348 },
        y: 47,
    }
}

pub fn target_default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 as f32 * 0.5 - 86.0) as i32, (viewport.1 as f32 * 0.5 - 23.0) as i32)
}

pub const TARGET_INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 2, x: 172, y: 46 };

pub fn control_grip() -> Rect {
    control_layout::RECT_7C
}

pub fn player_grip() -> Rect {
    player_layout::RECT_7C
}

pub fn target_grip() -> Rect {
    target_layout::RECT_7C
}

pub fn target_name_rect() -> Rect {
    target_layout::RECT_104
}

pub fn bar_width(value: i32, maximum: i32, full: i32) -> i32 {
    if maximum == 0 { 0 } else { value.wrapping_mul(full).checked_div(maximum).unwrap_or(0) }
}

fn paint_bars(list: &mut DrawList, origin: (i32, i32), values: &OlympiadUser, full: i32) {
    let (x, y) = origin;
    list.draw_texture(x + BAR_X, y + CP_BAR_Y, full, BAR_HEIGHT, 0, 0, 8, BAR_HEIGHT, CP_BAR_BACK, 1.0);
    list.draw_texture(x + BAR_X, y + HP_BAR_Y, full, BAR_HEIGHT, 0, 0, 8, BAR_HEIGHT, HP_BAR_BACK, 1.0);
    if values.max_cp != 0 {
        list.draw_texture(x + BAR_X, y + CP_BAR_Y, bar_width(values.cp, values.max_cp, full), BAR_HEIGHT, 0, 0, 8, BAR_HEIGHT, CP_BAR, 1.0);
    }
    if values.max_hp != 0 {
        list.draw_texture(x + BAR_X, y + HP_BAR_Y, bar_width(values.hp, values.max_hp, full), BAR_HEIGHT, 0, 0, 8, BAR_HEIGHT, HP_BAR, 1.0);
    }
}

fn paint_back(list: &mut DrawList, origin: (i32, i32), size: (i32, i32), parts: &[&str; 3], source_height: i32) {
    let (x, y) = origin;
    list.draw_texture(x + 12, y, 16, size.1, 0, 0, 16, source_height, parts[0], 1.0);
    list.draw_texture(x + 28, y, size.0 - 32 - 12, size.1, 0, 0, 16, source_height, parts[1], 1.0);
    list.draw_texture(x + size.0 - 16, y, 16, size.1, 0, 0, 16, source_height, parts[2], 1.0);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlAction {
    Return,
    MatchList,
}

#[derive(Clone, Debug)]
pub struct OlympiadControl {
    pub frame: FrameCtrl,
    pub buttons: [TextButton; 2],
    pressed: Option<usize>,
}

impl OlympiadControl {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        Self {
            frame: control_controls::FRAME_7C.build(text),
            buttons: [control_controls::BUTTON_EC.build((0, 0), text), control_controls::BUTTON_F0.build((0, 0), text)],
            pressed: None,
        }
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> bool {
        let contracts = [&control_controls::BUTTON_EC, &control_controls::BUTTON_F0];
        self.pressed = contracts.iter().zip(&self.buttons).position(|(contract, button)| contract.hit(button, x, y).is_some());
        self.pressed.is_some()
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<ControlAction> {
        let pressed = self.pressed.take()?;
        if !self.buttons[pressed].contains(x, y) {
            return None;
        }
        Some(if pressed == 0 { ControlAction::Return } else { ControlAction::MatchList })
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        let start = list.quads.len();
        let (w, h) = CONTROL_SIZE;
        list.draw_texture(x + 12, y, w - 12, h, 0, 0, w - 12, h, CONTROL_BACK, alpha);
        list.clip_from(start, Rect::new(x, y, w, h));
        control_controls::FRAME_7C.paint(&self.frame, list, origin, font, alpha);
        control_controls::BUTTON_EC.paint(&self.buttons[0], list, origin, self.pressed == Some(0), font);
        control_controls::BUTTON_F0.paint(&self.buttons[1], list, origin, self.pressed == Some(1), font);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OlympiadPlayer {
    pub frame: FrameCtrl,
    pub user: OlympiadUser,
    pub expanded: bool,
    ring: [String; LOG_ROWS],
    head: usize,
    pressed: bool,
}

impl Default for OlympiadPlayer {
    fn default() -> Self {
        Self {
            frame: FrameCtrl { mode: FrameMode::VerticalGrip, title: String::new(), close_button: false, minimize_button: false },
            user: OlympiadUser::default(),
            expanded: false,
            ring: Default::default(),
            head: 0,
            pressed: false,
        }
    }
}

impl OlympiadPlayer {
    pub fn height(&self) -> i32 {
        if self.expanded { PLAYER_EXPANDED_HEIGHT } else { PLAYER_SIZE.1 }
    }

    pub fn initialize(&mut self) {
        self.user = OlympiadUser::default();
        self.ring = Default::default();
        self.head = 0;
    }

    pub fn object_id(&self) -> i32 {
        self.user.object_id
    }

    pub fn set_user_info(&mut self, info: &OlympiadUserInfoView) {
        self.user = info.user.clone();
    }

    pub fn add_message(&mut self, text: &str) {
        self.ring[self.head] = text.to_string();
        self.head = (self.head + 1) % LOG_ROWS;
    }

    pub fn log_lines(&self) -> Vec<&str> {
        (0..LOG_ROWS).map(|index| self.ring[(self.head + index) % LOG_ROWS].as_str()).collect()
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> bool {
        self.pressed = EXPAND_RECT.contains(x, y);
        self.pressed
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<bool> {
        let pressed = std::mem::take(&mut self.pressed);
        if !pressed || !EXPAND_RECT.contains(x, y) {
            return None;
        }
        self.expanded = !self.expanded;
        Some(self.expanded)
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        let size = (PLAYER_SIZE.0, self.height());
        let start = list.quads.len();
        if self.expanded {
            paint_back(list, origin, size, &EXPANDED_BACK_PARTS, size.1);
        } else {
            paint_back(list, origin, size, &BACK_PARTS, PLAYER_SIZE.1);
        }
        list.clip_from(start, Rect::new(x, y, size.0, size.1));
        self.frame.paint(list, x, y, 12, size.1, font, alpha);
        let texture = if self.expanded { EXPAND_BUTTON_DOWN } else { EXPAND_BUTTON };
        list.draw_texture(x + EXPAND_RECT.x, y + EXPAND_RECT.y, EXPAND_RECT.w, EXPAND_RECT.h, 0, 0, EXPAND_RECT.w, EXPAND_RECT.h, texture, 1.0);
        paint_bars(list, origin, &self.user, PLAYER_BAR_WIDTH);
        list.draw_text(x + NAME_POSITION.0, y + NAME_POSITION.1, NAME_COLOR, &self.user.name, font);
        if self.expanded {
            for (index, line) in self.log_lines().into_iter().enumerate() {
                if !line.is_empty() {
                    list.draw_text(x + LOG_X, y + (index as i32 + 3) * LOG_STEP, NAME_COLOR, line, font);
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OlympiadTarget {
    pub frame: FrameCtrl,
    pub user: OlympiadUser,
    pub side: i32,
}

impl Default for OlympiadTarget {
    fn default() -> Self {
        Self {
            frame: target_controls::FRAME_7C.build(|_| String::new()),
            user: OlympiadUser { object_id: 0, ..OlympiadUser::default() },
            side: 0,
        }
    }
}

impl OlympiadTarget {
    pub fn initialize(&mut self) {
        self.user = OlympiadUser { object_id: 0, ..OlympiadUser::default() };
    }

    pub fn set_side(&mut self, side: i32) {
        self.side = side;
    }

    pub fn set_user_info(&mut self, info: &OlympiadUserInfoView) {
        self.user = info.user.clone();
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        let start = list.quads.len();
        paint_back(list, origin, TARGET_SIZE, &BACK_PARTS, TARGET_SIZE.1);
        list.clip_from(start, Rect::new(x, y, TARGET_SIZE.0, TARGET_SIZE.1));
        target_controls::FRAME_7C.paint(&self.frame, list, origin, font, alpha);
        paint_bars(list, origin, &self.user, TARGET_BAR_WIDTH);
        list.draw_text(x + NAME_POSITION.0, y + NAME_POSITION.1, NAME_COLOR, &self.user.name, font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(object_id: i32, name: &str, hp: i32, max_hp: i32, cp: i32, max_cp: i32) -> OlympiadUserInfoView {
        OlympiadUserInfoView { side: 1, user: OlympiadUser { object_id, name: name.into(), hp, max_hp, cp, max_cp } }
    }

    #[test]
    fn the_layouts_match_the_on_create_snapshots() {
        assert_eq!(CONTROL_SIZE, (102, 56));
        assert_eq!(PLAYER_SIZE, (348, 46));
        assert_eq!(TARGET_SIZE, (172, 46));
        assert_eq!(control_grip(), Rect::new(0, 0, 12, 56));
        assert_eq!(player_grip(), Rect::new(0, 0, 12, 46));
        assert_eq!(target_name_rect(), Rect::new(12, 6, 160, 14));
        let control = OlympiadControl::new(|id| Some(format!("s{id}")));
        assert_eq!(control.buttons[0].rect, Rect::new(19, 5, 76, 23));
        assert_eq!(control.buttons[1].rect, Rect::new(19, 29, 76, 23));
        assert_eq!((control.buttons[0].label.as_str(), control.buttons[1].label.as_str()), ("s611", "s1252"));
    }

    #[test]
    fn the_initial_positions_restore_the_creation_coordinates() {
        use crate::initial_position::InitialAxis;
        let viewport = (1280, 800);
        let check = |descriptor: crate::initial_position::InitialPosition, expected: (i32, i32)| {
            let x = InitialAxis::horizontal(descriptor.horizontal_mode, descriptor.x).unwrap().coordinate(viewport.0);
            let y = InitialAxis::vertical(descriptor.vertical_mode, descriptor.y).unwrap().coordinate(viewport.1);
            assert_eq!((x, y), expected);
        };
        check(CONTROL_INITIAL_POSITION, (640 - 51, 0));
        check(player_initial_position(0), player_default_position(0, viewport));
        check(player_initial_position(1), player_default_position(1, viewport));
        check(abnormal_initial_position(0), abnormal_default_position(0, viewport));
        check(abnormal_initial_position(1), abnormal_default_position(1, viewport));
        check(TARGET_INITIAL_POSITION, (640 - 86, 400 - 23));
    }

    #[test]
    fn the_bar_width_is_the_signed_integer_ratio_and_zero_without_a_maximum() {
        assert_eq!(bar_width(50, 100, 326), 163);
        assert_eq!(bar_width(1, 3, 326), 108);
        assert_eq!(bar_width(5, 0, 326), 0);
        assert_eq!(bar_width(-50, 100, 326), -163);
    }

    #[test]
    fn the_log_ring_keeps_five_lines_oldest_first() {
        let mut player = OlympiadPlayer::default();
        for index in 0..7 {
            player.add_message(&format!("m{index}"));
        }
        assert_eq!(player.log_lines(), vec!["m2", "m3", "m4", "m5", "m6"]);
        player.initialize();
        assert!(player.log_lines().iter().all(|line| line.is_empty()));
        assert_eq!(player.object_id(), NO_OBJECT);
    }

    #[test]
    fn the_expand_button_toggles_between_46_and_126() {
        let mut player = OlympiadPlayer::default();
        assert_eq!(player.height(), 46);
        assert!(player.mouse_down(3, 8));
        assert_eq!(player.mouse_up(3, 8), Some(true));
        assert_eq!(player.height(), 126);
        assert!(player.mouse_down(3, 8));
        assert_eq!(player.mouse_up(3, 8), Some(false));
        assert!(!player.mouse_down(100, 8));
        assert_eq!(player.mouse_up(100, 8), None);
    }

    #[test]
    fn user_info_fills_name_and_the_four_gauges() {
        let mut player = OlympiadPlayer::default();
        player.set_user_info(&info(7, "Ana", 10, 20, 3, 4));
        assert_eq!((player.object_id(), player.user.name.as_str(), player.user.hp, player.user.max_cp), (7, "Ana", 10, 4));
        let mut target = OlympiadTarget::default();
        target.set_side(2);
        target.set_user_info(&info(9, "Bo", 1, 2, 3, 4));
        assert_eq!((target.side, target.user.max_hp), (2, 2));
        target.initialize();
        assert_eq!((target.side, target.user.object_id, target.user.name.as_str()), (2, 0, ""));
    }

    #[test]
    fn the_control_buttons_report_return_and_match_list_on_release() {
        let mut control = OlympiadControl::new(|_| None);
        let first = control.buttons[0].rect;
        let second = control.buttons[1].rect;
        assert!(control.mouse_down(first.x + 2, first.y + 2));
        assert_eq!(control.mouse_up(first.x + 2, first.y + 2), Some(ControlAction::Return));
        assert!(control.mouse_down(second.x + 2, second.y + 2));
        assert_eq!(control.mouse_up(second.x + 2, second.y + 2), Some(ControlAction::MatchList));
        assert!(control.mouse_down(second.x + 2, second.y + 2));
        assert_eq!(control.mouse_up(0, 0), None);
    }
}
