use std::collections::HashMap;

pub const IDENTIFIER_RED: u32 = 4;
pub const IDENTIFIER_BLUE: u32 = 8;
pub const SELECTED_RED: u32 = 0x10;
pub const SELECTED_BLUE: u32 = 0x20;
pub const ALL_BITS: [u32; 6] = [1, 2, 4, 8, 0x10, 0x20];

pub const MARK_FLAG: u32 = 1;
pub const CANVAS_BROADCAST_MODE: u32 = 2;
pub const CANVAS_TARGET_NAME: u32 = 4;
pub const ICON_COUNT: i32 = 9;

pub const IDENTIFIER_CLASS_RED: &str = "LineageEffect.e_u520_identifier_r";
pub const IDENTIFIER_CLASS_BLUE: &str = "LineageEffect.e_u520_identifier_b";
pub const SELECTED_CLASS_RED: &str = "LineageEffect.e_u520_selected_r";
pub const SELECTED_CLASS_BLUE: &str = "LineageEffect.e_u520_selected_b";
pub const IDENTIFIER_PRE_PIVOT: f32 = 0.5;
pub const SELECTED_PRE_PIVOT: f32 = 2.0;

pub const MARKER_CLASS_A: &str = "LineageEffect.e_u080_a";
pub const MARKER_CLASS_B: &str = "LineageEffect.e_u080_b";
pub const MARKER_SIZE_DIVISOR: f32 = 9.0;

pub fn class_of(bit: u32) -> Option<&'static str> {
    match bit {
        1 => Some(MARKER_CLASS_A),
        2 => Some(MARKER_CLASS_B),
        IDENTIFIER_RED => Some(IDENTIFIER_CLASS_RED),
        IDENTIFIER_BLUE => Some(IDENTIFIER_CLASS_BLUE),
        SELECTED_RED => Some(SELECTED_CLASS_RED),
        SELECTED_BLUE => Some(SELECTED_CLASS_BLUE),
        _ => None,
    }
}

pub fn pre_pivot_base(bit: u32) -> f32 {
    match bit {
        SELECTED_RED | SELECTED_BLUE => SELECTED_PRE_PIVOT,
        IDENTIFIER_RED | IDENTIFIER_BLUE => IDENTIFIER_PRE_PIVOT,
        _ => 0.0,
    }
}

pub fn scales_with_radius(bit: u32) -> bool {
    bit == 1 || bit == 2
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PawnMarks {
    pub effects: u32,
    pub flags: u32,
    pub team: i32,
    pub index: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppendixOp {
    Flag { object_id: i32, set: bool },
    Index { object_id: i32, team: i32, index: i32 },
    Add { object_id: i32, bit: u32 },
    Delete { object_id: i32, bit: u32 },
    DeleteAll { object_id: i32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasOp {
    Start,
    Finish,
    ToggleTargetName,
}

impl AppendixOp {
    pub fn object_id(self) -> i32 {
        match self {
            AppendixOp::Flag { object_id, .. }
            | AppendixOp::Index { object_id, .. }
            | AppendixOp::Add { object_id, .. }
            | AppendixOp::Delete { object_id, .. }
            | AppendixOp::DeleteAll { object_id } => object_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Appendix {
    pawns: HashMap<i32, PawnMarks>,
    pub canvas: u32,
}

impl Default for Appendix {
    fn default() -> Self {
        Self { pawns: HashMap::new(), canvas: CANVAS_TARGET_NAME }
    }
}

impl Appendix {
    pub fn effects(&self, object_id: i32) -> u32 {
        self.pawns.get(&object_id).map_or(0, |marks| marks.effects)
    }

    pub fn marks(&self, object_id: i32) -> PawnMarks {
        self.pawns.get(&object_id).copied().unwrap_or_default()
    }

    pub fn ids(&self) -> impl Iterator<Item = i32> + '_ {
        self.pawns.keys().copied()
    }

    pub fn add(&mut self, object_id: i32, bit: u32) {
        self.pawns.entry(object_id).or_default().effects |= bit;
    }

    pub fn delete(&mut self, object_id: i32, bit: u32) {
        if let Some(marks) = self.pawns.get_mut(&object_id) {
            marks.effects &= !bit;
        }
    }

    pub fn delete_all(&mut self, object_id: i32) {
        if let Some(marks) = self.pawns.get_mut(&object_id) {
            marks.effects = 0;
        }
    }

    pub fn set_team(&mut self, object_id: i32, team: i32) {
        match team {
            0 => {
                self.delete(object_id, IDENTIFIER_RED);
                self.delete(object_id, IDENTIFIER_BLUE);
            }
            1 => {
                self.delete(object_id, IDENTIFIER_RED);
                self.add(object_id, IDENTIFIER_BLUE);
            }
            2 => {
                self.delete(object_id, IDENTIFIER_BLUE);
                self.add(object_id, IDENTIFIER_RED);
            }
            _ => {}
        }
    }

    pub fn forget(&mut self, object_id: i32) {
        self.pawns.remove(&object_id);
    }

    pub fn retain(&mut self, exists: impl Fn(i32) -> bool) {
        self.pawns.retain(|object_id, _| exists(*object_id));
    }

    pub fn apply(&mut self, op: AppendixOp) {
        match op {
            AppendixOp::Flag { object_id, set } => {
                let marks = self.pawns.entry(object_id).or_default();
                if set {
                    marks.flags |= MARK_FLAG;
                } else {
                    marks.flags &= !MARK_FLAG;
                }
            }
            AppendixOp::Index { object_id, team, index } => {
                let marks = self.pawns.entry(object_id).or_default();
                marks.team = team;
                marks.index = index;
            }
            AppendixOp::Add { object_id, bit } => self.add(object_id, bit),
            AppendixOp::Delete { object_id, bit } => self.delete(object_id, bit),
            AppendixOp::DeleteAll { object_id } => self.delete_all(object_id),
        }
    }

    pub fn canvas_apply(&mut self, op: CanvasOp) {
        match op {
            CanvasOp::Start => self.canvas_start(),
            CanvasOp::Finish => self.canvas_finish(),
            CanvasOp::ToggleTargetName => self.canvas_toggle_target_name(),
        }
    }

    pub fn canvas_start(&mut self) {
        self.canvas |= CANVAS_BROADCAST_MODE;
    }

    pub fn canvas_finish(&mut self) {
        self.canvas &= !CANVAS_BROADCAST_MODE;
    }

    pub fn canvas_toggle_target_name(&mut self) {
        self.canvas ^= CANVAS_TARGET_NAME;
    }

    pub fn icon(&self, object_id: i32) -> Option<(usize, usize)> {
        if self.canvas & CANVAS_BROADCAST_MODE == 0 {
            return None;
        }
        let marks = self.pawns.get(&object_id)?;
        if marks.flags & MARK_FLAG == 0 || !(0..ICON_COUNT).contains(&marks.index) || !(0..2).contains(&marks.team) {
            return None;
        }
        Some((marks.team as usize, marks.index as usize))
    }

    pub fn name_suppressed(&self, object_id: i32) -> bool {
        self.canvas & CANVAS_BROADCAST_MODE != 0
            && self.pawns.get(&object_id).is_some_and(|marks| marks.flags & MARK_FLAG != 0)
            && self.canvas & CANVAS_TARGET_NAME == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_starts_with_target_names_on_and_broadcast_mode_off() {
        let appendix = Appendix::default();
        assert_eq!(appendix.canvas, CANVAS_TARGET_NAME);
    }

    #[test]
    fn team_rule_matches_the_char_info_switch() {
        let mut appendix = Appendix::default();
        appendix.set_team(7, 1);
        assert_eq!(appendix.effects(7), IDENTIFIER_BLUE);
        appendix.set_team(7, 2);
        assert_eq!(appendix.effects(7), IDENTIFIER_RED);
        appendix.add(7, SELECTED_RED);
        appendix.set_team(7, 0);
        assert_eq!(appendix.effects(7), SELECTED_RED);
        appendix.set_team(7, 5);
        assert_eq!(appendix.effects(7), SELECTED_RED);
    }

    #[test]
    fn icon_needs_canvas_mode_flag_and_index_below_nine() {
        let mut appendix = Appendix::default();
        appendix.apply(AppendixOp::Flag { object_id: 3, set: true });
        appendix.apply(AppendixOp::Index { object_id: 3, team: 1, index: 4 });
        assert_eq!(appendix.icon(3), None);
        appendix.canvas_start();
        assert_eq!(appendix.icon(3), Some((1, 4)));
        appendix.apply(AppendixOp::Index { object_id: 3, team: 1, index: 9 });
        assert_eq!(appendix.icon(3), None);
        appendix.apply(AppendixOp::Index { object_id: 3, team: 2, index: 1 });
        assert_eq!(appendix.icon(3), None);
        appendix.canvas_finish();
        assert_eq!(appendix.canvas & CANVAS_BROADCAST_MODE, 0);
    }

    #[test]
    fn marked_pawns_hide_their_name_until_the_target_name_toggle_flips() {
        let mut appendix = Appendix::default();
        appendix.canvas_start();
        appendix.apply(AppendixOp::Flag { object_id: 3, set: true });
        assert!(!appendix.name_suppressed(3));
        appendix.canvas_toggle_target_name();
        assert!(appendix.name_suppressed(3));
        assert!(!appendix.name_suppressed(4));
        appendix.canvas_toggle_target_name();
        assert!(!appendix.name_suppressed(3));
    }

    #[test]
    fn delete_all_clears_the_mask_but_keeps_the_marker_fields() {
        let mut appendix = Appendix::default();
        appendix.apply(AppendixOp::Index { object_id: 1, team: 0, index: 2 });
        appendix.add(1, IDENTIFIER_RED | SELECTED_RED);
        appendix.delete_all(1);
        assert_eq!(appendix.effects(1), 0);
        assert_eq!(appendix.marks(1).index, 2);
    }
}
