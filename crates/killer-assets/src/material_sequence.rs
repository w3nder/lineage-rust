pub const ACTION_RESET: u8 = 1;
pub const ACTION_TOGGLE_PAUSE: u8 = 2;
pub const ACTION_PAUSE: u8 = 3;
pub const FLAG_LOOP: u32 = 1;
pub const FLAG_PAUSED: u32 = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MaterialSequence {
    pub trigger_action: u8,
    pub flags: u32,
    pub current_time: f32,
    pub last_time: f32,
}

impl MaterialSequence {
    pub fn reset(&mut self) {
        self.current_time = 0.0;
        self.last_time = 0.0;
    }

    pub fn trigger(&mut self) {
        match self.trigger_action {
            ACTION_RESET => self.reset(),
            ACTION_TOGGLE_PAUSE => self.flags ^= FLAG_PAUSED,
            ACTION_PAUSE => self.flags |= FLAG_PAUSED,
            _ => {}
        }
    }
}

pub const ITEM_CUT: u8 = 0;
pub const ITEM_BLEND: u8 = 1;
pub const BLEND_MASK_OPERATION: u8 = 5;

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub material: String,
    pub time: f32,
    pub action: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selection {
    Keep,
    Show(usize),
    Blend { from: usize, to: usize, alpha: u8 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Timeline {
    pub items: Vec<Item>,
    pub total_time: f32,
}

impl MaterialSequence {
    pub fn pre_set_material(&mut self, timeline: &Timeline, time: f32) -> Selection {
        if self.flags & FLAG_PAUSED != 0 {
            return Selection::Keep;
        }
        if self.last_time <= time {
            if 0.0 < self.last_time {
                self.current_time += time - self.last_time;
            }
        } else {
            self.last_time = 0.0;
            self.current_time = 0.0;
        }
        self.last_time = time;
        let looping = self.flags & FLAG_LOOP != 0;
        let total = timeline.total_time;
        if looping && 0.0 < total && total < self.current_time {
            let turns = (self.current_time / total).floor();
            let mut remainder = self.current_time - turns * total;
            loop {
                self.current_time = remainder;
                if total >= self.current_time {
                    break;
                }
                remainder = self.current_time - total;
            }
        }
        let count = timeline.items.len();
        let mut start = 0.0f32;
        for (index, item) in timeline.items.iter().enumerate() {
            if start <= self.current_time {
                let end = start + item.time;
                if self.current_time < end || (!looping && index + 1 == count) {
                    if item.action != ITEM_CUT {
                        if item.action != ITEM_BLEND {
                            return Selection::Keep;
                        }
                        if self.current_time < end {
                            let ratio = (self.current_time - start) / item.time * 255.0;
                            let alpha = (ratio.round() as i32).clamp(0, 255) as u8;
                            let from = if index == 0 { count - 1 } else { index - 1 };
                            return Selection::Blend { from, to: index, alpha };
                        }
                    }
                    return Selection::Show(index);
                }
            }
            start += item.time;
        }
        Selection::Keep
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(time: f32, action: u8) -> Item {
        Item { material: String::new(), time, action }
    }

    #[test]
    fn pre_set_material_105ac930_walks_the_items_by_accumulated_time() {
        let timeline = Timeline { items: vec![item(1.0, ITEM_CUT), item(1.0, ITEM_BLEND), item(0.0, ITEM_CUT)], total_time: 2.0 };
        let mut sequence = MaterialSequence { trigger_action: 0, flags: FLAG_LOOP, current_time: 0.0, last_time: 0.0 };
        assert_eq!(sequence.pre_set_material(&timeline, 0.5), Selection::Show(0));
        assert_eq!(sequence.pre_set_material(&timeline, 1.0), Selection::Show(0));
        assert_eq!(sequence.pre_set_material(&timeline, 1.5), Selection::Blend { from: 0, to: 1, alpha: 0 });
        assert_eq!(sequence.pre_set_material(&timeline, 1.75), Selection::Blend { from: 0, to: 1, alpha: 64 });
        assert_eq!(sequence.pre_set_material(&timeline, 3.0), Selection::Show(0));
        assert!((sequence.current_time - 0.5).abs() < 1.0e-5);
        sequence.flags |= FLAG_PAUSED;
        let before = sequence;
        assert_eq!(sequence.pre_set_material(&timeline, 9.0), Selection::Keep);
        assert_eq!(sequence, before);
    }

    #[test]
    fn pre_set_material_without_loop_stays_on_the_last_item_and_a_time_going_backwards_resets() {
        let timeline = Timeline { items: vec![item(1.0, ITEM_CUT), item(1.0, ITEM_CUT)], total_time: 2.0 };
        let mut sequence = MaterialSequence { trigger_action: 0, flags: 0, current_time: 0.0, last_time: 0.0 };
        sequence.pre_set_material(&timeline, 1.0);
        assert_eq!(sequence.pre_set_material(&timeline, 5.0), Selection::Show(1));
        assert_eq!(sequence.pre_set_material(&timeline, 9.0), Selection::Show(1));
        assert_eq!(sequence.pre_set_material(&timeline, 2.0), Selection::Show(0));
        assert_eq!((sequence.last_time, sequence.current_time), (2.0, 0.0));
    }

    #[test]
    fn pre_set_material_leaves_the_material_alone_for_unknown_item_actions_and_wraps_the_first_blend_to_the_last_item() {
        let timeline = Timeline { items: vec![item(2.0, ITEM_BLEND), item(1.0, 2)], total_time: 3.0 };
        let mut sequence = MaterialSequence { trigger_action: 0, flags: FLAG_LOOP, current_time: 0.0, last_time: 1.0 };
        assert_eq!(sequence.pre_set_material(&timeline, 1.5), Selection::Blend { from: 1, to: 0, alpha: 64 });
        assert_eq!(sequence.pre_set_material(&timeline, 3.0), Selection::Keep);
        assert_eq!(sequence.pre_set_material(&timeline, 3.0001), Selection::Keep);
        let mut late = MaterialSequence { trigger_action: 0, flags: FLAG_LOOP, current_time: 2.2, last_time: 1.0 };
        assert_eq!(late.pre_set_material(&timeline, 1.0), Selection::Keep);
    }

    fn sequence(trigger_action: u8, flags: u32) -> MaterialSequence {
        MaterialSequence { trigger_action, flags, current_time: 2.5, last_time: 1.5 }
    }

    #[test]
    fn trigger_105aa020_follows_each_action() {
        let mut reset = sequence(ACTION_RESET, 3);
        reset.trigger();
        assert_eq!(reset, MaterialSequence { trigger_action: 1, flags: 3, current_time: 0.0, last_time: 0.0 });
        let mut toggle = sequence(ACTION_TOGGLE_PAUSE, 0xffff_fffd);
        toggle.trigger();
        assert_eq!(toggle.flags, u32::MAX);
        toggle.trigger();
        assert_eq!(toggle.flags, 0xffff_fffd);
        let mut pause = sequence(ACTION_PAUSE, 1);
        pause.trigger();
        pause.trigger();
        assert_eq!((pause.flags, pause.current_time), (3, 2.5));
        for action in [0, 4, 255] {
            let mut ignored = sequence(action, 1);
            ignored.trigger();
            assert_eq!(ignored, sequence(action, 1));
        }
    }
}
