use crate::button::{CheckBox, TextButton};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;
use crate::option_game::GamePage;
use crate::option_video::VideoPage;
use crate::option_audio::AudioPage;
use crate::tab_strip::{TabStrip, TabSpec};
use crate::tab_control::{TabEvent, TabEvents};
use crate::combo::ComboBox;

pub const SIZE: (i32, i32) = (256, 480);
pub const INITIAL_POSITION: crate::initial_position::InitialPosition = crate::initial_position::DISABLED_INITIAL_POSITION;

pub fn creation_position(viewport: (i32, i32)) -> (i32, i32) {
    (
        crate::window_restore::x87_to_int(f64::from(viewport.0 as f32) * 0.5 - 128.0),
        crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) * 0.5 - 240.0),
    )
}

pub const BACK: &str = "L2ui_ch3.OptionWnd.option1_back";
pub const AUDIO_BACK: &str = "L2ui_ch3.OptionWnd.option2_back";
pub const GAME_BACK: &str = "L2ui_ch3.OptionWnd.option3_back";
pub const BACK_RECT: Rect = Rect::new(0, 48, 256, 432);
pub fn close_rect() -> Rect {
    Rect::new(SIZE.0 - crate::frame::CLOSE_INSET, crate::frame::BUTTON_Y, crate::frame::BUTTON_SIZE, crate::frame::BUTTON_SIZE)
}

pub const HEADER: &str = "L2UI_ch3.SSQWnd.ssq_back";
pub const HEADER_RECT: Rect = Rect::new(0, 20, 256, 28);
pub const PAGE_RECT: Rect = Rect::new(0, 48, 256, 400);
pub const TABS_RECT: Rect = Rect::new(12, 25, 224, 23);
pub const OK_RECT: Rect = Rect::new(11, 450, 76, 23);
pub const CANCEL_RECT: Rect = Rect::new(91, 450, 76, 23);
pub const APPLY_RECT: Rect = Rect::new(171, 450, 76, 23);

pub const TAB_VIDEO: usize = 0;
pub const TAB_AUDIO: usize = 1;
pub const TAB_GAME: usize = 2;
pub const GOD_FLAG: u32 = 1;

struct StripFlags<'a>(&'a mut Vec<crate::tab_strip::StripButton>);

impl crate::tab_control::TabEnableBackend for StripFlags<'_> {
    fn enable_button(&mut self, index: usize, enabled: bool) {
        if let Some(button) = self.0.get_mut(index) { button.paint.enabled = enabled; }
    }
    fn button_flags(&mut self, index: usize) -> u32 {
        self.0.get(index).map_or(0, |button| button.attachment_flags)
    }
    fn set_button_flags(&mut self, index: usize, flags: u32) {
        if let Some(button) = self.0.get_mut(index) { button.attachment_flags = flags; }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionCombo {
    pub page: usize,
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionNotification {
    pub page: usize,
    pub id: u32,
    pub selection: i32,
}

#[derive(Debug, Clone)]
pub struct OptionCheck {
    pub id: u32,
    pub sysstring: u32,
    pub check: CheckBox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionHit {
    Close,
    Ok,
    Cancel,
    Apply,
    Tab(usize),
    Check(usize),
    VideoCheck(usize),
    Combo(OptionCombo),
    ComboArrow(OptionCombo),
    Initialize,
}

#[derive(Debug, Clone)]
pub struct OptionView {
    pub initialize_pressed: bool,
    pub tabs: TabStrip,
    pub focused_page: Option<usize>,
    pub focused_combo: Option<OptionCombo>,
    pub pressed_arrow: Option<OptionCombo>,
    pub pressed_arrow_inside: bool,
    pub title: String,
    pub game_page: GamePage,
    pub video_page: VideoPage,
    pub audio_page: AudioPage,
    pub ok_label: String,
    pub cancel_label: String,
    pub apply_label: String,
}

impl OptionView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> OptionView {
        Self::with_joystick(lookup, false)
    }

    pub fn with_joystick(lookup: impl Fn(u32) -> Option<String>, can_use_joystick: bool) -> OptionView {
        Self::with_god(lookup, can_use_joystick, 0)
    }

    pub fn with_god(lookup: impl Fn(u32) -> Option<String>, can_use_joystick: bool, god: u32) -> OptionView {
        let text = |id: u32, default: &str| lookup(id).unwrap_or_else(|| default.to_string());
        let mut tabs = TabStrip::new(0xf006, 0x00fa0000);
        for (page, label_id) in [206, 207, 208].into_iter().enumerate() {
            tabs.add_fixed(TabSpec {
                label: lookup(label_id), page: Some(page), size: (74, 23),
                textures: [Some("l2ui_ch3.tabbutton.normal_tab_on".into()), Some("l2ui_ch3.tabbutton.normal_tab".into())],
                spacing: i32::from(page < 2), tooltip: -1, special_attachment: false,
            });
        }
        for button in &mut tabs.buttons { button.visible = true; }
        let mut view = OptionView {
            tabs,
            initialize_pressed: false,
            focused_page: None,
            focused_combo: None,
            pressed_arrow: None,
            pressed_arrow_inside: false,
            title: text(146, ""),
            game_page: GamePage::new(can_use_joystick, &lookup),
            video_page: VideoPage::new(&lookup),
            audio_page: AudioPage::new(&lookup),
            ok_label: text(0x8c, "OK"),
            cancel_label: text(141, ""),
            apply_label: text(218, ""),
        };
        view.sync_check_dependencies();
        if god != 0 {
            let mut backend = StripFlags(&mut view.tabs.buttons);
            view.tabs.control.disable_tab(TAB_GAME, true, &mut backend);
        }
        view
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<OptionHit> {
        if let Some(key) = self.focused_combo
            && let Some(hit) = self.combo_hit(key, x, y)
        { return Some(hit); }
        if close_rect().contains(x, y) {
            return Some(OptionHit::Close);
        }
        for (index, button) in self.tabs.buttons.iter().enumerate() {
            if button.visible && button.paint.rect.contains(x - TABS_RECT.x, y - TABS_RECT.y) {
                return Some(OptionHit::Tab(index));
            }
        }
        for (rect, hit) in [(OK_RECT, OptionHit::Ok), (CANCEL_RECT, OptionHit::Cancel), (APPLY_RECT, OptionHit::Apply)] {
            if rect.contains(x, y) {
                return Some(hit);
            }
        }
        if let Some(page) = self.active_page() {
            for index in 0..self.combo_count() {
                if let Some(hit) = self.combo_hit(OptionCombo { page, index }, x, y) { return Some(hit); }
            }
        }
        if self.active_page() == Some(TAB_GAME)
            && self.game_page.initialize.enabled
            && self.game_page.initialize.rect.contains(x - PAGE_RECT.x, y - PAGE_RECT.y)
        {
            return Some(OptionHit::Initialize);
        }
        match self.active_page() {
            Some(TAB_VIDEO) => self.video_page.checks.iter().enumerate().find_map(|(index, check)| {
                (check.enabled && check.contains(x - PAGE_RECT.x, y - PAGE_RECT.y)).then_some(OptionHit::VideoCheck(index))
            }),
            Some(TAB_GAME) => self.game_page.checks.iter().enumerate().find_map(|(index, check)| {
                (check.check.enabled && check.check.contains(x - PAGE_RECT.x, y - PAGE_RECT.y)).then_some(OptionHit::Check(index))
            }),
            _ => None,
        }
    }

    pub fn active_page(&self) -> Option<usize> { self.tabs.control.current }

    fn combo_count(&self) -> usize {
        match self.active_page() { Some(TAB_VIDEO) => 13, Some(TAB_AUDIO) => 4, Some(TAB_GAME) => 2, _ => 0 }
    }

    pub fn combo(&self, key: OptionCombo) -> Option<&ComboBox> {
        match key.page {
            TAB_VIDEO => self.video_page.combos.get(key.index),
            TAB_AUDIO => self.audio_page.combos.get(key.index),
            TAB_GAME => match key.index { 0 => Some(&self.game_page.language), 1 => Some(&self.game_page.looting), _ => None },
            _ => None,
        }
    }

    pub fn combo_mut(&mut self, key: OptionCombo) -> Option<&mut ComboBox> {
        match key.page {
            TAB_VIDEO => self.video_page.combos.get_mut(key.index),
            TAB_AUDIO => self.audio_page.combos.get_mut(key.index),
            TAB_GAME => match key.index { 0 => Some(&mut self.game_page.language), 1 => Some(&mut self.game_page.looting), _ => None },
            _ => None,
        }
    }

    fn combo_hit(&self, key: OptionCombo, x: i32, y: i32) -> Option<OptionHit> {
        if self.active_page() != Some(key.page) { return None; }
        let combo = self.combo(key)?;
        let (x, y) = (x - PAGE_RECT.x, y - PAGE_RECT.y);
        if !combo.expanded_rect().contains(x, y) { return None; }
        Some(if combo.arrow_rect().contains(x, y) { OptionHit::ComboArrow(key) } else { OptionHit::Combo(key) })
    }

    pub fn open_combo_hit(&self, x: i32, y: i32) -> Option<OptionHit> {
        let key = self.focused_combo?;
        if !self.combo(key)?.open { return None; }
        self.combo_hit(key, x, y)
    }

    pub fn set_font(&mut self, font: &BitmapFont) {
        self.video_page.set_font(font);
        let height = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
        for combo in &mut self.audio_page.combos { combo.text_height = height; }
        self.game_page.language.text_height = height;
        self.game_page.looting.text_height = height;
    }

    pub fn lose_combo_focus(&mut self) {
        if let Some(key) = self.focused_combo.take()
            && let Some(combo) = self.combo_mut(key)
        { combo.lose_focus(); }
        self.pressed_arrow = None;
        self.pressed_arrow_inside = false;
    }

    pub fn combo_mouse_move(&mut self, x: i32, y: i32) {
        if let Some(key) = self.pressed_arrow {
            self.pressed_arrow_inside = self.combo(key).is_some_and(|combo| combo.arrow_rect().contains(x - PAGE_RECT.x, y - PAGE_RECT.y));
        }
        if let Some(key) = self.focused_combo
            && let Some(combo) = self.combo_mut(key)
        { combo.mouse_move(x - PAGE_RECT.x, y - PAGE_RECT.y); }
    }

    pub fn combo_mouse_down(&mut self, key: OptionCombo, arrow: bool) -> Option<OptionNotification> {
        if self.active_page() != Some(key.page) || !self.combo(key).is_some_and(|combo| combo.enabled) { return None; }
        if arrow {
            self.pressed_arrow = Some(key);
            self.pressed_arrow_inside = true;
            return None;
        }
        if self.focused_combo != Some(key) { self.lose_combo_focus(); }
        self.focused_page = None;
        self.focused_combo = Some(key);
        let selected = self.combo_mut(key)?.mouse_down();
        if !self.combo(key)?.open { self.focused_combo = None; }
        let selection = selected?;
        let id = match key.page {
            TAB_VIDEO => *crate::option_video::CONTROL_IDS.get(key.index)?,
            TAB_AUDIO => *crate::option_audio::CONTROL_IDS.get(key.index)?,
            TAB_GAME => match key.index { 0 => crate::option_game::LANGUAGE_ID, 1 => crate::option_game::LOOTING_ID, _ => return None },
            _ => return None,
        };
        if key.page == TAB_VIDEO && id == 0x1f25 { self.video_page.post_process_changed(selection); }
        if key.page == TAB_GAME { self.game_page.message(0x501, id, false); }
        Some(OptionNotification { page: key.page, id, selection })
    }

    pub fn combo_mouse_up(&mut self, released: Option<OptionHit>) {
        self.pressed_arrow_inside = false;
        if let Some(key) = self.pressed_arrow.take()
            && released == Some(OptionHit::ComboArrow(key))
            && self.active_page() == Some(key.page)
            && self.combo(key).is_some_and(|combo| combo.enabled)
        {
            if self.focused_combo != Some(key) { self.lose_combo_focus(); }
            self.focused_page = None;
            self.focused_combo = Some(key);
            if let Some(combo) = self.combo_mut(key) { combo.open_list(); }
        }
    }

    pub fn press_tab(&mut self, index: usize) -> Vec<TabEvent> {
        let mut events = TabEvents::default();
        self.tabs.control.press(index, true, &mut events);
        for event in &events.0 {
            if let TabEvent::FocusPage(page) = event {
                self.lose_combo_focus();
                self.focused_page = Some(*page);
                for combo in &mut self.video_page.combos { combo.lose_focus(); }
                for combo in &mut self.audio_page.combos { combo.lose_focus(); }
                self.game_page.language.lose_focus();
                self.game_page.looting.lose_focus();
            }
        }
        events.0
    }

    pub fn toggle(&mut self, index: usize) {
        if let Some(c) = self.game_page.checks.get_mut(index)
            && c.check.mouse_down().is_some()
        {
            let id = c.id;
            self.game_page.message(0x501, id, false);
        }
    }

    pub fn toggle_video(&mut self, index: usize) -> Option<OptionNotification> {
        let checked = self.video_page.checks.get_mut(index)?.mouse_down()?;
        Some(OptionNotification {
            page: TAB_VIDEO,
            id: *crate::option_video::CHECK_IDS.get(index)?,
            selection: i32::from(checked),
        })
    }

    pub fn set_checked(&mut self, id: u32, value: bool) {
        if let Some(c) = self.game_page.checks.iter_mut().find(|c| c.id == id) {
            c.check.checked = value;
        }
        self.sync_check_dependencies();
    }

    fn sync_check_dependencies(&mut self) {
        for (parent, children) in [
            (0xf0f01461, &[0xf0f01462, 0xf0f01463, 0xf0f01464][..]),
            (0xf0f01479, &[0xf0f01480, 0xf0f01481][..]),
        ] {
            let enabled = self.checked(parent);
            for child in self.game_page.checks.iter_mut().filter(|c| children.contains(&c.id)) {
                child.check.enabled = enabled;
            }
        }
    }

    pub fn checked(&self, id: u32) -> bool {
        self.game_page.checks.iter().find(|c| c.id == id).is_some_and(|c| c.check.checked)
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        self.paint_with_body(list, (x, y), font, alpha, |list| {
            list.draw_texture(x + HEADER_RECT.x, y + HEADER_RECT.y, HEADER_RECT.w, HEADER_RECT.h, 0, 0, HEADER_RECT.w, HEADER_RECT.h, HEADER, alpha);
            list.draw_texture(x + BACK_RECT.x, y + BACK_RECT.y, BACK_RECT.w, BACK_RECT.h, 0, 0, BACK_RECT.w, BACK_RECT.h, self.background(), alpha);
        });
    }

    pub fn paint_with_body(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32, body: impl FnOnce(&mut DrawList)) {
        let (x, y) = origin;
        let (w, h) = SIZE;
        crate::frame::FrameCtrl::titled(&self.title, true).paint(list, x, y, w, h, font, alpha);
        body(list);
        self.tabs.paint(list, (x + TABS_RECT.x, y + TABS_RECT.y), font);
        let origin = (x + PAGE_RECT.x, y + PAGE_RECT.y);
        match self.active_page() {
            Some(TAB_VIDEO) => {
                self.video_page.paint_labels(list, origin, font);
                for check in &self.video_page.checks { check.paint(list, origin.0, origin.1, font); }
            }
            Some(TAB_AUDIO) => {
                self.audio_page.paint_labels(list, origin, font);
            }
            Some(TAB_GAME) => {
                for ((label_x, label_y, right_aligned), label) in [(24, 15, false), (79, 49, true), (76, 82, true), (79, 377, true)].into_iter().zip(&self.game_page.labels) {
                    let width = if right_aligned { font.measure(label) as i32 } else { 0 };
                    list.draw_text(x + PAGE_RECT.x + label_x - width, y + PAGE_RECT.y + label_y, crate::color::TEXT, label, font);
                }
                for c in &self.game_page.checks {
                    c.check.paint(list, x + PAGE_RECT.x, y + PAGE_RECT.y, font);
                }
                self.game_page.initialize.paint(list, x + PAGE_RECT.x, y + PAGE_RECT.y, self.initialize_pressed, font);
            }
            _ => {}
        }
        if let Some(page) = self.active_page() {
            for index in 0..self.combo_count() {
                let key = OptionCombo { page, index };
                if self.focused_combo != Some(key) { self.paint_combo(key, list, origin, font); }
            }
        }
        for (rect, label) in [(OK_RECT, &self.ok_label), (CANCEL_RECT, &self.cancel_label), (APPLY_RECT, &self.apply_label)] {
            TextButton::new(rect, "L2UI_CH3.BUTTON.BTN1_normal", "L2UI_CH3.BUTTON.BTN1_normalon", label)
                .paint(list, x, y, false, font);
        }
        if let Some(key) = self.focused_combo.filter(|key| self.active_page() == Some(key.page)) {
            self.paint_combo(key, list, origin, font);
        }
    }

    fn paint_combo(&self, key: OptionCombo, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        if let Some(combo) = self.combo(key) {
            let mut combo = combo.clone();
            combo.text_height = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
            combo.paint(list, origin, font, self.focused_combo == Some(key), self.pressed_arrow == Some(key) && self.pressed_arrow_inside);
        }
    }

    pub fn background(&self) -> &'static str {
        match self.active_page() {
            Some(TAB_AUDIO) => AUDIO_BACK,
            Some(TAB_GAME) => GAME_BACK,
            _ => BACK,
        }
    }
}

pub fn grip() -> Rect {
    crate::frame::FrameCtrl::new(crate::frame::FrameMode::Title).grip(SIZE.0, SIZE.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};
    use std::collections::HashMap;

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn game_view(lookup: impl Fn(u32) -> Option<String>) -> OptionView {
        let mut view = OptionView::with_god(|id| lookup(id).or_else(|| [206, 207, 208].contains(&id).then(String::new)), false, 0);
        view.press_tab(TAB_GAME);
        view
    }

    #[test]
    fn the_game_tab_lists_the_checkboxes_from_buildcontrols_b() {
        let texts: HashMap<u32, String> =
            [(927, "Transparent"), (442, "Enter Chat"), (357, "Damage"), (146, "Options"), (478, "Interface")]
                .into_iter()
                .map(|(id, t)| (id, t.to_string()))
                .collect();
        let view = OptionView::new(|id| texts.get(&id).cloned());
        assert_eq!(view.game_page.checks.len(), 17);
        // Transparent (id 0xf0f01475) existe e casa com a sysstring 927
        assert!(view.game_page.checks.iter().any(|c| c.id == 0xf0f01475 && c.sysstring == 927));
        assert_eq!(view.title, "Options");
    }

    #[test]
    fn container_100cb4c0_creates_three_external_buttons_with_the_original_labels() {
        let view = OptionView::new(|id| match id {
            140 => Some("OK".into()),
            141 => Some("Cancel".into()),
            218 => Some("Apply".into()),
            _ => None,
        });
        assert_eq!((&*view.ok_label, &*view.cancel_label, &*view.apply_label), ("OK", "Cancel", "Apply"));
        let mut list = DrawList::default();
        view.paint(&mut list, 10, 20, &font(), 1.0);
        for (rect, hit) in [(OK_RECT, OptionHit::Ok), (CANCEL_RECT, OptionHit::Cancel), (APPLY_RECT, OptionHit::Apply)] {
            assert_eq!(view.hit(rect.x + 1, rect.y + 1), Some(hit));
            assert!(list.quads.iter().any(|q| q.texture == "L2UI_CH3.BUTTON.BTN1_normal" && q.x == (10 + rect.x) as f32 && q.y == (20 + rect.y) as f32));
        }
        assert_eq!(view.hit(88, 451), None);
        assert_eq!(view.hit(168, 451), None);
    }

    #[test]
    fn video_checks_receive_mouse_down_only_when_enabled() {
        let mut view = OptionView::new(|id| [206, 207, 208].contains(&id).then(|| id.to_string()));
        view.press_tab(TAB_VIDEO);
        for (index, check) in view.video_page.checks.iter().enumerate() {
            let (x, y) = (PAGE_RECT.x + check.rect.x + 1, PAGE_RECT.y + check.rect.y + 1);
            assert_eq!(view.hit(x, y), Some(OptionHit::VideoCheck(index)));
        }
        let before = view.video_page.checks[0].checked;
        assert_eq!(view.toggle_video(0), Some(OptionNotification { page: TAB_VIDEO, id: 0xf0f01465, selection: i32::from(!before) }));
        assert_eq!(view.video_page.checks[0].checked, !before);
        view.video_page.checks[1].enabled = false;
        let check = &view.video_page.checks[1];
        assert_eq!(view.hit(PAGE_RECT.x + check.rect.x + 1, PAGE_RECT.y + check.rect.y + 1), None);
        assert_eq!(view.toggle_video(1), None);
    }

    #[test]
    fn toggling_and_reading_a_checkbox_by_id() {
        let view0 = OptionView::new(|_| None);
        let mut view = view0;
        view.set_checked(0xf0f01475, true);
        assert!(view.checked(0xf0f01475));
        // clicar no Transparent alterna
        let idx = view.game_page.checks.iter().position(|c| c.id == 0xf0f01475).unwrap();
        view.toggle(idx);
        assert!(!view.checked(0xf0f01475));
    }

    #[test]
    fn ncoptionwnd_100cb4c0_places_the_outer_ok_below_the_pages() {
        assert_eq!(OK_RECT, Rect::new(11, 450, 76, 23));
        let view = game_view(|_| None);
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font(), 1.0);
        assert!(list.quads.iter().any(|quad| quad.texture == GAME_BACK));
    }

    #[test]
    fn ncoptionwnd_100c8c30_uses_its_own_textures_and_unscaled_source_rectangles() {
        assert_eq!(SIZE, (256, 480));
        let view = game_view(|_| None);
        let mut list = DrawList::default();
        view.paint(&mut list, 31, 47, &font(), 1.0);
        for (texture, rect) in [(HEADER, HEADER_RECT), (GAME_BACK, BACK_RECT)] {
            let quads: Vec<_> = list.quads.iter().filter(|q| q.texture == texture).collect();
            assert_eq!(quads.len(), 1);
            let q = quads[0];
            assert_eq!((q.x, q.y, q.w, q.h), ((31 + rect.x) as f32, (47 + rect.y) as f32, rect.w as f32, rect.h as f32));
            assert_eq!((q.u, q.v, q.uw, q.vh), (0.0, 0.0, rect.w as f32, rect.h as f32));
        }
        assert!(!list.quads.iter().any(|q| q.texture == "L2UI.etcWnd.OptionBack" || q.texture.contains("QuestBack3")));
    }

    #[test]
    fn ncoptionwnd_100cb4c0_offsets_game_paint_and_input_by_the_child_origin() {
        let mut view = game_view(|_| None);
        view.set_checked(0xf0f01461, true);
        view.set_checked(0xf0f01479, true);
        let mut list = DrawList::default();
        view.paint(&mut list, 31, 47, &font(), 1.0);
        let checks: Vec<_> = list.quads.iter().filter(|q| [crate::button::CHECK_BOX, crate::button::CHECK_BOX_CHECKED].contains(&q.texture.as_str())).collect();
        assert_eq!(checks.len(), view.game_page.checks.len());
        for (index, (control, quad)) in view.game_page.checks.iter().zip(checks).enumerate() {
            let rect = control.check.rect;
            assert_eq!((quad.x, quad.y), ((31 + rect.x + PAGE_RECT.x) as f32, (47 + rect.y + PAGE_RECT.y) as f32));
            assert_eq!(view.hit(rect.x + PAGE_RECT.x + 1, rect.y + PAGE_RECT.y + 1), Some(OptionHit::Check(index)));
        }
        assert_eq!(view.hit(165, 16), None);
        assert_eq!(view.hit(89, 12), None);
        assert_eq!(view.hit(12, 451), Some(OptionHit::Ok));
    }

    #[test]
    fn option_page_show_100c4140_100c4870_100c4960_replaces_the_container_background() {
        let mut view = game_view(|_| None);
        for (tab, texture) in [(TAB_VIDEO, BACK), (TAB_AUDIO, AUDIO_BACK), (TAB_GAME, GAME_BACK)] {
            view.press_tab(tab);
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font(), 1.0);
            assert_eq!(view.background(), texture);
            for candidate in [BACK, AUDIO_BACK, GAME_BACK] {
                assert_eq!(list.quads.iter().filter(|q| q.texture == candidate).count(), usize::from(candidate == texture));
            }
        }
    }

    #[test]
    fn game_100c8b70_100c8bc0_disable_children_without_erasing_their_checks() {
        let mut view = game_view(|_| None);
        view.set_checked(0xf0f01461, false);
        view.set_checked(0xf0f01479, false);
        for (parent, children) in [
            (0xf0f01461, &[0xf0f01462, 0xf0f01463, 0xf0f01464][..]),
            (0xf0f01479, &[0xf0f01480, 0xf0f01481][..]),
        ] {
            for child in children {
                view.set_checked(*child, true);
                let index = view.game_page.checks.iter().position(|c| c.id == *child).unwrap();
                assert!(!view.game_page.checks[index].check.enabled);
                view.toggle(index);
                assert!(view.checked(*child));
                let r = view.game_page.checks[index].check.rect;
                assert_eq!(view.hit(r.x + 1, r.y + PAGE_RECT.y + 1), None);
            }
            view.set_checked(parent, true);
            assert!(view.game_page.checks.iter().filter(|c| children.contains(&c.id)).all(|c| c.check.enabled && c.check.checked));
            let index = view.game_page.checks.iter().position(|c| c.id == parent).unwrap();
            view.toggle(index);
            assert!(view.game_page.checks.iter().filter(|c| children.contains(&c.id)).all(|c| !c.check.enabled && c.check.checked));
        }
    }

    #[test]
    fn game_onpaint_100c7720_keeps_interface_as_a_page_label_and_aligns_the_other_labels() {
        let view = game_view(|id| [478, 329, 435, 486].contains(&id).then(|| "A".into()));
        assert_eq!(view.title, "");
        let mut list = DrawList::default();
        view.paint(&mut list, 10, 20, &font(), 1.0);
        let labels: Vec<_> = list.quads.iter().filter(|q| q.texture == "f" && q.color == crate::color::TEXT.rgba()).collect();
        assert_eq!(labels.len(), 4);
        for (q, (x, y)) in labels.into_iter().zip([(34.0, 83.0), (83.0, 117.0), (80.0, 150.0), (83.0, 445.0)]) {
            assert_eq!((q.x, q.y), (x, y));
        }
    }

    #[test]
    fn option_combos_10005d70_select_last_hover_in_every_page_and_lose_focus() {
        let mut view = OptionView::with_god(|id| Some(id.to_string()), false, 0);
        view.set_font(&font());
        for (page, count) in [(TAB_VIDEO, 13), (TAB_AUDIO, 4), (TAB_GAME, 2)] {
            view.press_tab(page);
            for index in 0..count {
                let key = OptionCombo { page, index };
                let combo = view.combo_mut(key).unwrap();
                combo.enabled = true;
                combo.entries = vec!["First".into(), "Second".into(), "Third".into()];
                let rect = combo.rect;
                assert_eq!(view.hit(rect.x + 1, PAGE_RECT.y + rect.y + 1), Some(OptionHit::Combo(key)));
                assert_eq!(view.combo_mouse_down(key, false), None);
                assert!(view.combo(key).unwrap().open);
                view.combo_mouse_move(rect.x + 1, PAGE_RECT.y + rect.y + rect.h + 2 * crate::combo::ROW_HEIGHT + 1);
                let event = view.combo_mouse_down(key, false).unwrap();
                assert_eq!((event.page, event.selection), (page, 2));
                assert_eq!(event.id, match page {
                    TAB_VIDEO => crate::option_video::CONTROL_IDS[index],
                    TAB_AUDIO => crate::option_audio::CONTROL_IDS[index],
                    _ => [crate::option_game::LANGUAGE_ID, crate::option_game::LOOTING_ID][index],
                });
                assert_eq!(view.combo(key).unwrap().selected, 2);
                assert!(!view.combo(key).unwrap().open);
                assert_eq!(view.focused_combo, None);
                view.combo_mouse_down(key, false);
                view.lose_combo_focus();
                assert_eq!(view.combo(key).unwrap().selected, 2);
                assert!(!view.combo(key).unwrap().open);
            }
        }
    }

    #[test]
    fn option_combo_arrow_10005c10_opens_on_release_and_never_confirms_hover() {
        let mut view = OptionView::with_god(|id| Some(id.to_string()), false, 0);
        view.press_tab(TAB_AUDIO);
        let key = OptionCombo { page: TAB_AUDIO, index: 0 };
        let next = OptionCombo { page: TAB_AUDIO, index: 1 };
        view.combo_mouse_down(key, true);
        assert!(!view.combo(key).unwrap().open);
        assert_eq!(view.focused_combo, None);
        assert!(view.pressed_arrow_inside);
        view.combo_mouse_move(0, 0);
        assert!(!view.pressed_arrow_inside);
        assert_eq!(view.pressed_arrow, Some(key));
        let arrow = view.combo(key).unwrap().arrow_rect();
        view.combo_mouse_move(arrow.x + 1, PAGE_RECT.y + arrow.y + 1);
        assert!(view.pressed_arrow_inside);
        view.combo_mouse_up(None);
        assert!(!view.combo(key).unwrap().open);
        view.combo_mouse_down(key, true);
        view.combo_mouse_up(Some(OptionHit::ComboArrow(key)));
        assert!(view.combo(key).unwrap().open);
        view.combo_mut(key).unwrap().hovered = 2;
        view.combo_mouse_down(key, true);
        view.combo_mouse_up(Some(OptionHit::ComboArrow(key)));
        assert_eq!(view.combo(key).unwrap().selected, -1);
        assert!(view.combo(key).unwrap().open);
        view.combo_mouse_down(next, false);
        assert!(!view.combo(key).unwrap().open);
        assert!(view.combo(next).unwrap().open);
        view.press_tab(TAB_GAME);
        assert!(!view.combo(next).unwrap().open);
        assert_eq!(view.combo_mouse_down(key, false), None);
        assert_eq!(view.focused_combo, None);
        view.press_tab(TAB_AUDIO);
        view.combo_mut(key).unwrap().enabled = false;
        view.combo_mouse_down(key, false);
        assert!(!view.combo(key).unwrap().open);
        assert_eq!(view.focused_combo, None);
    }

    #[test]
    fn option_combo_notification_100c3ec0_runs_the_original_shader_handler() {
        let mut view = OptionView::with_god(|id| Some(id.to_string()), false, 0);
        view.set_font(&font());
        let key = OptionCombo { page: TAB_VIDEO, index: 12 };
        view.video_page.combos[12].entries = (0..6).map(|index| index.to_string()).collect();
        for selection in 0..6 {
            view.video_page.combos[10].enabled = true;
            view.video_page.combos[10].selected = 2;
            let rect = view.combo(key).unwrap().rect;
            view.combo_mouse_down(key, false);
            view.combo_mouse_move(rect.x + 1, PAGE_RECT.y + rect.y + rect.h + selection * crate::combo::ROW_HEIGHT + 1);
            assert_eq!(view.combo_mouse_down(key, false), Some(OptionNotification { page: TAB_VIDEO, id: 0x1f25, selection }));
            assert_eq!(view.video_page.combos[10].enabled, selection < 3);
            assert_eq!(view.video_page.combos[10].selected, if selection < 3 { 2 } else { 0 });
        }
    }

    #[test]
    fn option_focused_looting_combo_covers_external_buttons_and_paints_last() {
        let mut view = game_view(|id| Some(id.to_string()));
        view.set_font(&font());
        view.game_page.append_choices(Some(1), 1, |id| Some(id.to_string()));
        let key = OptionCombo { page: TAB_GAME, index: 1 };
        view.combo_mouse_down(key, false);
        view.combo_mouse_move(100, 510);
        assert_eq!(view.hit(100, 451), Some(OptionHit::Combo(key)));
        assert_eq!(view.open_combo_hit(100, 510), Some(OptionHit::Combo(key)));
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font(), 1.0);
        let button = list.quads.iter().rposition(|quad| quad.texture == "L2UI_CH3.BUTTON.BTN1_normal").unwrap();
        let selection = list.quads.iter().position(|quad| quad.texture == crate::combo::SELECT).unwrap();
        assert!(selection > button);
        assert_eq!(view.combo_mouse_down(key, false).unwrap().selection, 4);
    }

    #[test]
    fn option_100cb4c0_disables_the_game_tab_when_the_god_flag_is_set_at_100cb897() {
        let mut view = OptionView::with_god(|id| [206, 207, 208].contains(&id).then(|| id.to_string()), false, GOD_FLAG);
        assert!(!view.tabs.control.entries[TAB_GAME].enabled);
        assert!(!view.tabs.buttons[TAB_GAME].paint.enabled);
        assert!(view.tabs.control.entries[TAB_VIDEO].enabled && view.tabs.control.entries[TAB_AUDIO].enabled);
        assert!(view.press_tab(TAB_GAME).is_empty());
        assert_eq!(view.active_page(), Some(TAB_VIDEO));
        assert!(!view.press_tab(TAB_AUDIO).is_empty());
        let open = OptionView::with_god(|id| [206, 207, 208].contains(&id).then(|| id.to_string()), false, 0);
        assert!(open.tabs.control.entries[TAB_GAME].enabled);
    }

    #[test]
    fn option_tabs_100cb4c0_and_1002e5a0_share_selection_paint_and_input() {
        let mut view = OptionView::with_god(|id| Some(id.to_string()), false, 0);
        assert_eq!(view.active_page(), Some(TAB_VIDEO));
        assert_eq!(view.tabs.buttons.len(), 3);
        for tab in [TAB_AUDIO, TAB_GAME, TAB_VIDEO] {
            let x = TABS_RECT.x + 75 * tab as i32;
            assert_eq!(view.hit(x + 1, 26), Some(OptionHit::Tab(tab)));
            let events = view.press_tab(tab);
            assert_eq!(events.iter().filter(|event| **event == TabEvent::Sound(5)).count(), 2);
            assert_eq!(view.active_page(), Some(tab));
            assert_eq!(view.focused_page, Some(tab));
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font(), 0.5);
            let tabs: Vec<_> = list.quads.iter().filter(|quad| quad.texture.starts_with("l2ui_ch3.tabbutton.")).collect();
            assert_eq!(tabs.len(), 3);
            for (index, quad) in tabs.iter().enumerate() {
                assert_eq!((quad.x, quad.y, quad.w, quad.h), ((12 + 75 * index) as f32, 25.0, 74.0, 23.0));
                assert_eq!(quad.texture, if tab == index { "l2ui_ch3.tabbutton.normal_tab" } else { "l2ui_ch3.tabbutton.normal_tab_on" });
            }
            assert_eq!(view.press_tab(tab).iter().filter(|event| **event == TabEvent::Sound(5)).count(), 1);
        }
        view.tabs.control.entries[TAB_GAME].enabled = false;
        assert!(view.press_tab(TAB_GAME).is_empty());
        assert_eq!(view.active_page(), Some(TAB_VIDEO));
        assert!(view.press_tab(3).is_empty());
    }

    #[test]
    fn option_container_100cb4c0_paints_only_the_selected_complete_page() {
        let mut view = OptionView::with_god(|id| Some(id.to_string()), true, 0);
        for (tab, combos, checks) in [(TAB_VIDEO, 13, 4), (TAB_AUDIO, 4, 0), (TAB_GAME, 2, 18)] {
            view.press_tab(tab);
            let mut list = DrawList::default();
            view.paint(&mut list, 30, 40, &font(), 1.0);
            let combo_quads: Vec<_> = list.quads.iter().filter(|q| q.texture == crate::combo::NORMAL[0]).collect();
            assert_eq!(combo_quads.len(), combos);
            let check_quads: Vec<_> = list.quads.iter().filter(|q| [crate::button::CHECK_BOX, crate::button::CHECK_BOX_CHECKED, crate::button::CHECK_BOX_UNABLE, crate::button::CHECK_BOX_CHECKED_UNABLE].contains(&q.texture.as_str())).collect();
            assert_eq!(check_quads.len(), checks);
            assert_eq!(list.quads.iter().filter(|q| q.texture == "L2UI_ch3.button.smallbutton2").count(), usize::from(tab == TAB_GAME));
            assert!(combo_quads.iter().all(|q| q.y >= 40.0 + PAGE_RECT.y as f32));
            if tab != TAB_GAME {
                let game_check = &view.game_page.checks[0].check;
                assert_eq!(view.hit(game_check.rect.x + 1, game_check.rect.y + PAGE_RECT.y + 1), None);
            }
        }
        assert_eq!(view.game_page.checks.len(), 18);
        assert_eq!(OptionView::new(|_| None).game_page.checks.len(), 17);
    }
}
