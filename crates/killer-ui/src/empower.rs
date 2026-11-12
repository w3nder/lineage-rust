#[path = "empower_controls_generated.rs"]
mod controls;
#[path = "empower_layout_generated.rs"]
mod layout;

use crate::button::{CheckBox, TextButton};
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::window::{Rect, WindowId};

pub const ID: WindowId = WindowId::EMPOWER;
pub const SIZE: (i32, i32) = layout::SIZE;
pub const BACK: &str = "l2ui_ch3.bloodhoodwnd.empower_back";
pub const BACK_ORIGIN: (i32, i32) = (0, 20);
pub const BACK_SIZE: (i32, i32) = (256, 381);
pub const NAME_POS: (i32, i32) = (23, 54);
pub const NAME_COLOR: Argb = Argb(0xFFFFFF00);
pub const RIGHT_OF_COLOR: Argb = Argb(0xFFDCDCDC);
pub const RIGHT_OF_SYSSTRING: u32 = 686;
pub const NOT_FOUND_SYSMSG: u32 = 145;
pub const MODE_VIEW: i32 = 1;
pub const AUTHORIZE_RELATION_BIT: u32 = 6;

pub fn may_authorize(relation: i32) -> bool {
    (relation as u32) >> AUTHORIZE_RELATION_BIT & 1 != 0
}
pub const MODE_EDIT: i32 = 2;
pub const ACTION_WRITE: i32 = 3;
pub const POWER_BYTES: usize = 32;
pub const LEAVES: usize = 11;
pub const BOXES: usize = 14;

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

#[derive(Clone, Copy)]
struct Entry {
    contract: &'static crate::child_contract::ChildContract,
    sysstring: u32,
    bit: Option<usize>,
    group: Option<usize>,
}

const fn leaf(contract: &'static crate::child_contract::ChildContract, sysstring: u32, bit: usize, group: usize) -> Entry {
    Entry { contract, sysstring, bit: Some(bit), group: Some(group) }
}

const fn group(contract: &'static crate::child_contract::ChildContract, sysstring: u32) -> Entry {
    Entry { contract, sysstring, bit: None, group: None }
}

const ENTRIES: [Entry; BOXES] = [
    leaf(&controls::CHILD_E8, 674, 0, 11),
    leaf(&controls::CHILD_EC, 675, 1, 11),
    leaf(&controls::CHILD_F0, 676, 2, 11),
    leaf(&controls::CHILD_F4, 677, 3, 11),
    leaf(&controls::CHILD_F8, 1210, 10, 11),
    leaf(&controls::CHILD_FC, 679, 4, 12),
    leaf(&controls::CHILD_100, 684, 5, 12),
    leaf(&controls::CHILD_104, 685, 6, 12),
    leaf(&controls::CHILD_108, 683, 7, 13),
    leaf(&controls::CHILD_10C, 684, 8, 13),
    leaf(&controls::CHILD_110, 685, 9, 13),
    group(&controls::CHILD_114, 673),
    group(&controls::CHILD_118, 678),
    group(&controls::CHILD_11C, 682),
];

const GROUP_LEAVES: [&[usize]; 3] = [&[0, 1, 2, 3, 4], &[5, 6, 7], &[8, 9, 10]];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmpowerAction {
    Close,
    Write { target: i32, powers: [u8; POWER_BYTES] },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmpowerHit {
    Close,
    Box(usize),
    Confirm(usize),
}

#[derive(Clone, Debug)]
pub struct EmpowerView {
    pub frame: FrameCtrl,
    pub boxes: Vec<CheckBox>,
    pub buttons: [TextButton; 2],
    pub powers: [u8; POWER_BYTES],
    pub target: i32,
    pub mode: i32,
    pub name: String,
    pub right_of: String,
    pressed: Option<EmpowerHit>,
}

fn bit_set(powers: &[u8; POWER_BYTES], bit: usize) -> bool {
    powers[bit / 8] & (1 << (bit % 8)) != 0
}

impl EmpowerView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        let boxes = ENTRIES.iter().map(|entry| CheckBox::new(entry.contract.rect, &text(entry.sysstring))).collect();
        Self {
            frame: controls::FRAME_7C.build(text),
            boxes,
            buttons: [controls::BUTTON_120.build((0, 0), text), controls::BUTTON_124.build((0, 0), text)],
            powers: [0; POWER_BYTES],
            target: 0,
            mode: 0,
            name: String::new(),
            right_of: text(RIGHT_OF_SYSSTRING),
            pressed: None,
        }
    }

    pub fn clear_marks(&mut self) {
        for check in &mut self.boxes {
            check.checked = false;
        }
    }

    pub fn apply(&mut self) {
        self.clear_marks();
        for (index, entry) in ENTRIES.iter().enumerate().take(LEAVES) {
            if entry.bit.is_some_and(|bit| bit_set(&self.powers, bit)) {
                self.boxes[index].checked = true;
                if let Some(group) = entry.group {
                    self.boxes[group].checked = true;
                }
            }
        }
    }

    pub fn collect(&mut self) {
        self.powers = [0; POWER_BYTES];
        for (index, entry) in ENTRIES.iter().enumerate().take(LEAVES) {
            if let (Some(bit), true) = (entry.bit, self.boxes[index].checked) {
                self.powers[bit / 8] |= 1 << (bit % 8);
            }
        }
    }

    pub fn set_powers(&mut self, powers: [u8; POWER_BYTES]) {
        self.powers = powers;
        self.apply();
    }

    pub fn set_target(&mut self, target: i32, name: &str, mode: i32) {
        self.target = target;
        self.name = name.to_string();
        self.mode = mode;
    }

    pub fn reset(&mut self) {
        self.clear_marks();
        self.powers = [0; POWER_BYTES];
        self.target = 0;
        self.mode = 0;
        self.name.clear();
        for check in &mut self.boxes {
            check.enabled = true;
        }
    }

    pub fn show(&mut self) {
        if self.mode == MODE_VIEW {
            for check in &mut self.boxes {
                check.enabled = false;
            }
        }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<EmpowerHit> {
        if controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(EmpowerHit::Close);
        }
        if controls::BUTTON_120.hit(&self.buttons[0], x, y).is_some() {
            return Some(EmpowerHit::Confirm(0));
        }
        if controls::BUTTON_124.hit(&self.buttons[1], x, y).is_some() {
            return Some(EmpowerHit::Confirm(1));
        }
        self.boxes.iter().position(|check| check.contains(x, y)).map(EmpowerHit::Box)
    }

    fn group_clicked(&mut self, group: usize) {
        if !self.boxes[LEAVES + group].enabled {
            return;
        }
        let marked = self.boxes[LEAVES + group].checked;
        for &leaf in GROUP_LEAVES[group] {
            self.boxes[leaf].checked = marked;
        }
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<EmpowerHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        if let Some(EmpowerHit::Box(index)) = hit
            && self.boxes[index].mouse_down().is_some()
            && index >= LEAVES
        {
            self.group_clicked(index - LEAVES);
        }
        hit
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<EmpowerAction> {
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            EmpowerHit::Close => Some(EmpowerAction::Close),
            EmpowerHit::Confirm(_) => Some(self.confirm()),
            EmpowerHit::Box(_) => None,
        }
    }

    pub fn confirm(&mut self) -> EmpowerAction {
        let write = (self.target > 0 && self.mode == MODE_EDIT).then(|| {
            self.collect();
            EmpowerAction::Write { target: self.target, powers: self.powers }
        });
        self.reset();
        write.unwrap_or(EmpowerAction::Close)
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        self.frame.paint(list, x, y, SIZE.0, SIZE.1, font, alpha);
        let start = list.quads.len();
        list.draw_texture(x + BACK_ORIGIN.0, y + BACK_ORIGIN.1, BACK_SIZE.0, BACK_SIZE.1, 0, 0, BACK_SIZE.0, BACK_SIZE.1, BACK, alpha);
        list.clip_from(start, Rect::new(x, y, SIZE.0, SIZE.1));
        if !self.name.is_empty() {
            let width = list.draw_text(x + NAME_POS.0, y + NAME_POS.1, NAME_COLOR, &self.name, font);
            list.draw_text(x + NAME_POS.0 + width, y + NAME_POS.1, RIGHT_OF_COLOR, &self.right_of, font);
        }
        for check in &self.boxes {
            check.paint(list, x, y, font);
        }
        controls::BUTTON_120.paint(&self.buttons[0], list, origin, self.pressed == Some(EmpowerHit::Confirm(0)), font);
        controls::BUTTON_124.paint(&self.buttons[1], list, origin, self.pressed == Some(EmpowerHit::Confirm(1)), font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> EmpowerView {
        EmpowerView::new(|id| Some(format!("s{id}")))
    }

    fn center(rect: Rect) -> (i32, i32) {
        (rect.x + 2, rect.y + 2)
    }

    #[test]
    fn the_initial_position_restores_the_creation_coordinates() {
        let viewport = (1280, 800);
        let descriptor = initial_position(viewport);
        assert_eq!((descriptor.horizontal_mode, descriptor.vertical_mode), (0, 5));
        let (x, y) = default_position(viewport);
        assert_eq!(crate::initial_position::InitialAxis::horizontal(0, descriptor.x).unwrap().coordinate(viewport.0), x);
        assert_eq!(crate::initial_position::InitialAxis::vertical(5, descriptor.y).unwrap().coordinate(viewport.1), y);
    }

    #[test]
    fn the_layout_matches_the_on_create_snapshot() {
        let view = view();
        assert_eq!(SIZE, (256, 401));
        assert_eq!(view.boxes[0].rect, Rect::new(32, 107, 100, 12));
        assert_eq!(view.boxes[11].rect, Rect::new(21, 87, 200, 12));
        assert_eq!(view.buttons[0].rect, Rect::new(51, 372, 76, 23));
        assert_eq!(view.buttons[1].rect, Rect::new(131, 372, 76, 23));
        assert_eq!(view.boxes[4].label, "s1210");
        assert_eq!(view.boxes[6].label, view.boxes[9].label);
    }

    #[test]
    fn the_bitmap_marks_leaves_and_their_groups_and_ignores_the_other_bits() {
        let mut view = view();
        let mut powers = [0u8; POWER_BYTES];
        powers[0] = 0b0001_0001;
        powers[1] = 0b0000_0100;
        powers[31] = 0xff;
        view.set_powers(powers);
        let marked: Vec<usize> = (0..BOXES).filter(|&i| view.boxes[i].checked).collect();
        assert_eq!(marked, vec![0, 4, 5, 11, 12]);
        view.collect();
        let mut expected = [0u8; POWER_BYTES];
        expected[0] = 0b0001_0001;
        expected[1] = 0b0000_0100;
        assert_eq!(view.powers, expected);
    }

    #[test]
    fn every_leaf_round_trips_through_the_bitmap() {
        for bit in 0..LEAVES {
            let mut view = view();
            let mut powers = [0u8; POWER_BYTES];
            powers[bit / 8] |= 1 << (bit % 8);
            view.set_powers(powers);
            assert_eq!(view.boxes.iter().filter(|check| check.checked).count(), 2);
            view.collect();
            assert_eq!(view.powers, powers);
        }
    }

    #[test]
    fn a_group_click_marks_or_clears_its_leaves_and_a_leaf_click_leaves_the_group_alone() {
        let mut view = view();
        let (x, y) = center(view.boxes[11].rect);
        view.mouse_down(x, y);
        assert!((0..5).all(|i| view.boxes[i].checked));
        assert!(!view.boxes[5].checked);
        let (x, y) = center(view.boxes[0].rect);
        view.mouse_down(x, y);
        assert!(!view.boxes[0].checked);
        assert!(view.boxes[11].checked);
        let (x, y) = center(view.boxes[11].rect);
        view.mouse_down(x, y);
        assert!((0..5).all(|i| !view.boxes[i].checked));
    }

    #[test]
    fn view_mode_makes_every_box_read_only_and_confirm_only_closes() {
        let mut view = view();
        view.set_target(7, "Ana", MODE_VIEW);
        view.set_powers([1; POWER_BYTES]);
        view.show();
        assert!(view.boxes.iter().all(|check| !check.enabled));
        let (x, y) = center(view.boxes[1].rect);
        let before = view.boxes[1].checked;
        view.mouse_down(x, y);
        assert_eq!(view.boxes[1].checked, before);
        assert_eq!(view.confirm(), EmpowerAction::Close);
        assert_eq!(view.target, 0);
        assert!(view.name.is_empty());
        assert!(view.boxes.iter().all(|check| check.enabled && !check.checked));
    }

    #[test]
    fn edit_mode_confirm_collects_the_boxes_and_both_buttons_share_the_handler() {
        for button in 0..2 {
            let mut view = view();
            view.set_target(9, "Bia", MODE_EDIT);
            view.show();
            let (x, y) = center(view.boxes[2].rect);
            view.mouse_down(x, y);
            let rect = view.buttons[button].rect;
            let (bx, by) = center(rect);
            view.mouse_down(bx, by);
            let mut expected = [0u8; POWER_BYTES];
            expected[0] = 1 << 2;
            assert_eq!(view.mouse_up(bx, by), Some(EmpowerAction::Write { target: 9, powers: expected }));
            assert_eq!(view.mode, 0);
        }
        let mut view = view();
        view.set_target(0, "x", MODE_EDIT);
        assert_eq!(view.confirm(), EmpowerAction::Close);
    }

    #[test]
    fn the_target_name_paints_before_the_right_of_label() {
        let font = BitmapFont::new(
            "L2Font-e.SmallFont-e".into(),
            crate::font::Glyphs::parse(include_bytes!("../tests/fixtures/smallfont-e.gly")).unwrap(),
        );
        let mut view = view();
        view.set_target(1, "Ana", MODE_VIEW);
        let mut list = DrawList::default();
        view.paint(&mut list, (10, 20), &font, 1.0);
        let yellow = list.quads.iter().filter(|q| q.color == NAME_COLOR.rgba()).count();
        assert_eq!(yellow, 3);
        assert!(list.quads.iter().any(|q| q.texture == BACK));
    }

    #[test]
    fn authorize_1004e900_tests_bit_6_of_the_own_relation_flags_which_is_clan_member_not_leader() {
        assert!(may_authorize(0x40));
        assert!(may_authorize(0x40 | 0x80 | 0x100));
        assert!(!may_authorize(0x80));
        assert!(!may_authorize(0x20 | 0x100));
        assert!(!may_authorize(0));
    }
}
