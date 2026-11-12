use crate::button::{CheckBox, TextButton};
use crate::combo::ComboBox;
use crate::option::OptionCheck;
use crate::window::Rect;

pub const LANGUAGE_ID: u32 = 0x1f15;
pub const LOOTING_ID: u32 = 0xffff1401;
pub const INITIALIZE_ID: u32 = 0xffff1400;
pub const LOOTING_CHOICES: [u32; 5] = [487, 488, 798, 799, 800];
pub const CHECK_SPECS: [(u32, u32, i32, i32, i32); 17] = [
    (0xf0f01459, 436, 84, 85, 100),
    (0xf0f01460, 437, 84, 99, 100),
    (0xf0f01461, 438, 84, 116, 100),
    (0xf0f01462, 439, 95, 133, 100),
    (0xf0f01463, 440, 95, 150, 100),
    (0xf0f01464, 441, 95, 167, 100),
    (0xf0f01471, 442, 25, 200, 100),
    (0xf0f01472, 566, 135, 200, 100),
    (0xf0f01470, 463, 25, 240, 100),
    (0x1f0f, 305, 135, 240, 100),
    (0xf0f01476, 928, 25, 257, 100),
    (0xf0f01478, 1196, 135, 257, 100),
    (0xf0f01479, 1197, 25, 327, 150),
    (0xf0f01480, 357, 36, 344, 70),
    (0xf0f01481, 796, 120, 344, 130),
    (0xf0f01475, 927, 164, 15, 100),
    (0xf0f01474, 926, 25, 217, 100),
];

#[derive(Debug, Clone)]
pub struct GamePage {
    pub checks: Vec<OptionCheck>,
    pub language: ComboBox,
    pub looting: ComboBox,
    pub initialize: TextButton,
    pub labels: [String; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMessageResult {
    Unhandled,
    Handled,
    InitializeInterface,
}

impl GamePage {
    pub fn clear(&mut self, can_use_joystick: bool) {
        for control in &mut self.checks {
            if control.id != 0xf0f01473 || can_use_joystick {
                control.check.checked = false;
            }
        }
        self.language.clear_entries();
        self.looting.clear_entries();
    }

    pub fn message(&mut self, message: u32, id: u32, base_handled: bool) -> GameMessageResult {
        if base_handled {
            return GameMessageResult::Handled;
        }
        if message != 0x501 {
            return GameMessageResult::Unhandled;
        }
        let children: &[u32] = match id {
            0xf0f01461 => &[0xf0f01462, 0xf0f01463, 0xf0f01464],
            0xf0f01479 => &[0xf0f01480, 0xf0f01481],
            INITIALIZE_ID => return GameMessageResult::InitializeInterface,
            _ => return GameMessageResult::Unhandled,
        };
        let enabled = self.checks.iter().any(|control| control.id == id && control.check.checked);
        for control in &mut self.checks {
            if children.contains(&control.id) {
                control.check.enabled = enabled;
            }
        }
        GameMessageResult::Handled
    }

    pub fn new(can_use_joystick: bool, lookup: impl Fn(u32) -> Option<String>) -> Self {
        let mut specs = CHECK_SPECS.to_vec();
        if can_use_joystick {
            specs.insert(8, (0xf0f01473, 606, 135, 217, 100));
        }
        let checks = specs.into_iter().map(|(id, sysstring, x, y, width)| {
            let mut check = CheckBox::new(Rect::new(x, y, width, 12), &lookup(sysstring).unwrap_or_default());
            check.checked = id != 0x1f0f;
            OptionCheck { id, sysstring, check }
        }).collect();
        Self {
            checks,
            language: ComboBox::new(Rect::new(87, 46, 125, 17), Vec::new(), 0),
            looting: ComboBox::new(Rect::new(87, 374, 125, 17), Vec::new(), 0),
            initialize: TextButton::new(Rect::new(88, 11, 66, 21), "L2UI_ch3.button.smallbutton2", "L2UI_ch3.button.smallbutton2_down", &lookup(479).unwrap_or_default()),
            labels: [478, 329, 435, 486].map(|id| lookup(id).unwrap_or_default()),
        }
    }

    pub fn append_choices(&mut self, language: Option<i32>, enable_english: i32, lookup: impl Fn(u32) -> Option<String>) {
        let native = match language.unwrap_or(0) {
            0 => Some("Korean"),
            1 => None,
            2 => Some("Japanese"),
            3 => Some("Chinese(Taiwan)"),
            4 => Some("China"),
            5 => Some("Thai"),
            _ => {
                self.looting.entries.extend(LOOTING_CHOICES.map(|id| lookup(id).unwrap_or_default()));
                return;
            }
        };
        if let Some(native) = native {
            self.language.entries.push(native.into());
        }
        self.language.entries.push("English".into());
        if native.is_some() && enable_english == 0 {
            self.language.enabled = false;
        }
        self.looting.entries.extend(LOOTING_CHOICES.map(|id| lookup(id).unwrap_or_default()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_clear_100c7d80_resets_choices_without_reenabling_or_closing_controls() {
        for can_use in [false, true] {
            let mut page = GamePage::new(true, |_| None);
            for check in &mut page.checks {
                check.check.checked = true;
                check.check.enabled = false;
            }
            for combo in [&mut page.language, &mut page.looting] {
                combo.entries = vec!["one".into(), "two".into()];
                combo.selected = 1;
                combo.hovered = 1;
                combo.open_list();
                combo.enabled = false;
            }
            let heights = (page.language.expanded_rect().h, page.looting.expanded_rect().h);
            page.clear(can_use);
            for control in &page.checks {
                assert_eq!(control.check.checked, control.id == 0xf0f01473 && !can_use);
                assert!(!control.check.enabled);
            }
            for combo in [&page.language, &page.looting] {
                assert!(combo.entries.is_empty());
                assert_eq!((combo.selected, combo.hovered), (-1, -1));
                assert!(combo.open);
                assert!(!combo.enabled);
            }
            assert_eq!((page.language.expanded_rect().h, page.looting.expanded_rect().h), heights);
        }
    }

    #[test]
    fn game_wndproc_100c48a0_dispatches_only_native_messages_after_base() {
        for parent in [0xf0f01461, 0xf0f01479] {
            for checked in [false, true] {
                for message in [0x500, 0x501, 0x502] {
                    for base in [false, true] {
                        let mut page = GamePage::new(false, |_| None);
                        page.set_checked(parent, checked);
                        let before: Vec<_> = page.checks.iter().map(|c| c.check.checked).collect();
                        let result = page.message(message, parent, base);
                        assert_eq!(result, if base || message == 0x501 { GameMessageResult::Handled } else { GameMessageResult::Unhandled });
                        for control in &page.checks {
                            let child = match parent {
                                0xf0f01461 => [0xf0f01462, 0xf0f01463, 0xf0f01464].contains(&control.id),
                                _ => [0xf0f01480, 0xf0f01481].contains(&control.id),
                            };
                            assert_eq!(control.check.enabled, !child || base || message != 0x501 || checked);
                        }
                        assert_eq!(page.checks.iter().map(|c| c.check.checked).collect::<Vec<_>>(), before);
                    }
                }
            }
        }
        let mut page = GamePage::new(false, |_| None);
        assert_eq!(page.message(0x501, INITIALIZE_ID, false), GameMessageResult::InitializeInterface);
        assert_eq!(page.message(0x501, INITIALIZE_ID, true), GameMessageResult::Handled);
        for id in [LANGUAGE_ID, LOOTING_ID, 0xf002, 0xf003, 0xf004, 0xf0f01459] {
            assert_eq!(page.message(0x501, id, false), GameMessageResult::Unhandled);
        }
    }

    #[test]
    fn game_create_100ca3e0_has_all_fixed_controls_and_conditional_joystick() {
        for joystick in [false, true] {
            let page = GamePage::new(joystick, |id| Some(id.to_string()));
            assert_eq!(page.checks.len(), if joystick { 18 } else { 17 });
            for (id, text, x, y, width) in CHECK_SPECS {
                let check = &page.checks.iter().find(|check| check.id == id).unwrap().check;
                assert_eq!(check.rect, Rect::new(x, y, width, 12));
                assert_eq!(check.label, text.to_string());
                assert_eq!(check.checked, id != 0x1f0f);
                assert!(check.enabled);
            }
            let optional = page.checks.iter().find(|check| check.id == 0xf0f01473);
            assert_eq!(optional.is_some(), joystick);
            if let Some(check) = optional { assert_eq!(check.check.rect, Rect::new(135, 217, 100, 12)); assert_eq!(check.sysstring, 606); }
            assert_eq!(page.language.rect, Rect::new(87, 46, 125, 17));
            assert_eq!(page.looting.rect, Rect::new(87, 374, 125, 17));
            assert_eq!(page.initialize.rect, Rect::new(88, 11, 66, 21));
            assert_eq!(page.initialize.label, "479");
            assert_eq!(page.initialize.normal, "L2UI_ch3.button.smallbutton2");
            assert_eq!(page.initialize.down, "L2UI_ch3.button.smallbutton2_down");
            assert_eq!(page.labels, ["478", "329", "435", "486"]);
        }
    }

    #[test]
    fn game_fill_100c9700_languages_append_without_guessing_or_reenabling() {
        for (language, expected) in [
            (None, vec!["Korean", "English"]), (Some(0), vec!["Korean", "English"]),
            (Some(1), vec!["English"]), (Some(2), vec!["Japanese", "English"]),
            (Some(3), vec!["Chinese(Taiwan)", "English"]), (Some(4), vec!["China", "English"]),
            (Some(5), vec!["Thai", "English"]), (Some(-1), vec![]), (Some(6), vec![]),
        ] {
            for enabled in [0, 1, -1] {
                let mut page = GamePage::new(false, |_| None);
                page.append_choices(language, enabled, |id| Some(id.to_string()));
                assert_eq!(page.language.entries, expected);
                assert_eq!(page.language.enabled, enabled != 0 || expected.len() != 2);
                assert_eq!(page.looting.entries, ["487", "488", "798", "799", "800"]);
                page.language.enabled = false;
                page.language.selected = 17;
                page.append_choices(language, 1, |id| Some(id.to_string()));
                assert!(!page.language.enabled);
                assert_eq!(page.language.selected, 17);
                assert_eq!(page.language.entries.len(), expected.len() * 2);
                assert_eq!(page.looting.entries.len(), 10);
            }
        }
    }
}
