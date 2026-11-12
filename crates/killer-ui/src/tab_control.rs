#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabEntry {
    pub page: Option<usize>,
    pub enabled: bool,
    pub detached: bool,
    pub pressed: bool,
    pub org: (i32, i32),
}

impl TabEntry {
    pub fn return_delta(&self, screen: (i32, i32)) -> (i32, i32) {
        (self.org.0.wrapping_sub(screen.0), self.org.1.wrapping_sub(screen.1))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TabControl {
    pub entries: Vec<TabEntry>,
    pub current: Option<usize>,
    pub flags: u32,
}

pub trait TabBackend {
    fn sound(&mut self, id: i32);
    fn show_page(&mut self, page: usize, shown: bool);
    fn focus_page(&mut self, page: usize);
    fn focus_button(&mut self, index: usize);
    fn raise_page_in_parent(&mut self, page: usize);
    fn notify_parent(&mut self, index: usize);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabEvent {
    Sound(i32),
    ShowPage(usize, bool),
    FocusPage(usize),
    FocusButton(usize),
    RaisePage(usize),
    NotifyParent(usize),
}

#[derive(Debug, Default)]
pub struct TabEvents(pub Vec<TabEvent>);

impl TabBackend for TabEvents {
    fn sound(&mut self, id: i32) { self.0.push(TabEvent::Sound(id)); }
    fn show_page(&mut self, page: usize, shown: bool) { self.0.push(TabEvent::ShowPage(page, shown)); }
    fn focus_page(&mut self, page: usize) { self.0.push(TabEvent::FocusPage(page)); }
    fn focus_button(&mut self, index: usize) { self.0.push(TabEvent::FocusButton(index)); }
    fn raise_page_in_parent(&mut self, page: usize) { self.0.push(TabEvent::RaisePage(page)); }
    fn notify_parent(&mut self, index: usize) { self.0.push(TabEvent::NotifyParent(index)); }
}

pub trait TabReleaseBackend: TabBackend {
    fn move_button(&mut self, index: usize, delta: (i32, i32));
    fn move_page(&mut self, page: usize, delta: (i32, i32));
}

pub trait TabMoveBackend: TabBackend {
    fn move_page_to(&mut self, page: usize, x: i32, y: i32);
}

pub trait TabRestoreBackend: TabBackend {
    fn restore_button_position(&mut self, index: usize) -> (i32, i32);
    fn move_restored_page(&mut self, page: usize, delta: (i32, i32));
}

pub trait TabEnableBackend {
    fn enable_button(&mut self, index: usize, enabled: bool);
    fn button_flags(&mut self, index: usize) -> u32;
    fn set_button_flags(&mut self, index: usize, flags: u32);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TabRect {
    pub x: i32,
    pub y: i32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabControlError {
    MissingPage(usize),
}

impl TabRect {
    fn contains(self, x: i32, y: i32) -> bool {
        x >= self.x && (x as f64) <= self.x as f64 + self.width as f64
            && y >= self.y && (y as f64) <= self.y as f64 + self.height as f64
    }

    pub fn contains_saved_corner(self, saved: Self) -> bool {
        let low_word = |value: f64| {
            if (-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&value) {
                value as i64 as i32
            } else {
                0
            }
        };
        let right = low_word(saved.x as f64 + saved.width as f64);
        let bottom = low_word(saved.y as f64 + saved.height as f64);
        [(saved.x, saved.y), (right, saved.y), (saved.x, bottom), (right, bottom)]
            .into_iter().any(|(x, y)| self.contains(x, y))
    }
}

impl TabControl {
    pub fn restore_detached(&mut self, backend: &mut impl TabRestoreBackend) -> Result<(), TabControlError> {
        for index in 0..self.entries.len() {
            if self.entries[index].detached {
                self.entries[index].detached = false;
                let delta = backend.restore_button_position(index);
                let page = self.entries[index].page.ok_or(TabControlError::MissingPage(index))?;
                backend.move_restored_page(page, delta);
                backend.notify_parent(index + self.entries.len());
            }
        }
        Ok(())
    }

    pub fn disable_tab(&mut self, index: usize, disabled: bool, backend: &mut impl TabEnableBackend) {
        let Some(entry) = self.entries.get_mut(index) else { return; };
        entry.enabled = !disabled;
        backend.enable_button(index, !disabled);
        if self.flags & 1 != 0 {
            let flags = backend.button_flags(index);
            backend.set_button_flags(index, if disabled { flags & !0x400000 } else { flags | 0x400000 });
        }
    }

    pub fn select(&mut self, index: usize, notify: bool, backend: &mut impl TabBackend) -> Result<(), TabControlError> {
        let Some(selected) = self.entries.get(index).filter(|entry| entry.enabled) else { return Ok(()); };
        let selected = selected.page;
        if let Some(index) = self.entries.iter().position(|entry| entry.page.is_none()) {
            return Err(TabControlError::MissingPage(index));
        }
        for entry in &self.entries {
            if let Some(page) = entry.page { backend.show_page(page, false); }
        }
        self.current = selected;
        if let Some(page) = selected { backend.show_page(page, true); }
        for entry in &mut self.entries { entry.pressed = false; }
        self.entries[index].pressed = true;
        if notify { backend.notify_parent(index); }
        Ok(())
    }

    pub fn release(&mut self, index: usize, current_rect: TabRect, saved_rect: TabRect, backend: &mut impl TabReleaseBackend) -> Result<bool, TabControlError> {
        let Some(entry) = self.entries.get(index).filter(|entry| entry.enabled) else {
            return Ok(false);
        };
        if entry.detached && current_rect.contains_saved_corner(saved_rect) {
            let page = entry.page.ok_or(TabControlError::MissingPage(index))?;
            self.entries[index].detached = false;
            let delta = (saved_rect.x.wrapping_sub(current_rect.x), saved_rect.y.wrapping_sub(current_rect.y));
            backend.move_button(index, delta);
            self.press(index, true, backend);
            backend.move_page(page, delta);
            backend.notify_parent(index + self.entries.len());
        }
        Ok(true)
    }

    pub fn press(&mut self, index: usize, notify: bool, backend: &mut impl TabBackend) -> bool {
        let Some(entry) = self.entries.get(index).filter(|entry| entry.enabled) else {
            return true;
        };
        let page = entry.page;
        if self.current != page {
            backend.sound(5);
        }
        let detached = self.entries.iter().find(|entry| entry.page == page).is_some_and(|entry| entry.detached);
        if !detached {
            for entry in &mut self.entries {
                if !entry.detached {
                    if let Some(page) = entry.page {
                        backend.show_page(page, false);
                    }
                    entry.pressed = false;
                }
            }
            self.current = page;
        }
        if let Some(page) = page {
            backend.show_page(page, true);
            self.entries[index].pressed = true;
            backend.focus_page(page);
            if self.flags & 1 != 0 {
                backend.focus_button(index);
                backend.raise_page_in_parent(page);
            }
        }
        if notify {
            backend.notify_parent(index);
        }
        backend.sound(5);
        true
    }

    pub fn move_tabs(&mut self, dx: i32, dy: i32) {
        for entry in &mut self.entries {
            entry.org = (entry.org.0.wrapping_add(dx), entry.org.1.wrapping_add(dy));
        }
    }

    pub fn is_split(&self, index: usize) -> bool {
        self.entries.get(index).is_some_and(|entry| entry.detached)
    }

    pub fn is_page_split(&self, page: usize) -> bool {
        self.entries.iter().position(|entry| entry.page == Some(page)).is_some_and(|index| self.is_split(index))
    }

    pub fn tab_move(&mut self, index: usize, x: i32, y: i32, backend: &mut impl TabMoveBackend) -> Result<bool, TabControlError> {
        if !self.entries.get(index).is_some_and(|entry| entry.enabled) {
            return Ok(false);
        }
        if self.flags & 1 != 0 {
            if !self.entries[index].detached {
                self.entries[index].detached = true;
                if let Some(first) = (0..self.entries.len()).find(|&i| !self.is_split(i)) {
                    self.press(first, true, backend);
                }
                self.press(index, true, backend);
            }
            let page = self.entries[index].page.ok_or(TabControlError::MissingPage(index))?;
            backend.move_page_to(page, x, y);
        }
        Ok(true)
    }

    pub fn refresh_pages(&self, backend: &mut impl TabBackend) {
        for entry in &self.entries {
            if let Some(page) = entry.page {
                backend.show_page(page, entry.detached);
            }
        }
        if let Some(page) = self.current {
            backend.show_page(page, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Backend(Vec<String>);

    impl TabBackend for Backend {
        fn sound(&mut self, id: i32) { self.0.push(format!("sound {id}")); }
        fn show_page(&mut self, page: usize, shown: bool) { self.0.push(format!("show {page} {shown}")); }
        fn focus_page(&mut self, page: usize) { self.0.push(format!("page {page}")); }
        fn focus_button(&mut self, index: usize) { self.0.push(format!("button {index}")); }
        fn raise_page_in_parent(&mut self, page: usize) { self.0.push(format!("raise {page}")); }
        fn notify_parent(&mut self, index: usize) { self.0.push(format!("notify {index}")); }
    }

    impl TabReleaseBackend for Backend {
        fn move_button(&mut self, index: usize, delta: (i32, i32)) { self.0.push(format!("move button {index} {delta:?}")); }
        fn move_page(&mut self, page: usize, delta: (i32, i32)) { self.0.push(format!("move page {page} {delta:?}")); }
    }

    impl TabRestoreBackend for Backend {
        fn restore_button_position(&mut self, index: usize) -> (i32, i32) {
            self.0.push(format!("restore button {index}"));
            (index as i32 + 11, -(index as i32) - 12)
        }
        fn move_restored_page(&mut self, page: usize, delta: (i32, i32)) {
            self.0.push(format!("move page {page} {delta:?}"));
        }
    }

    impl TabMoveBackend for Backend {
        fn move_page_to(&mut self, page: usize, x: i32, y: i32) { self.0.push(format!("page {page} to ({x}, {y})")); }
    }

    #[test]
    fn tab_move_1002da20_detaches_once_selects_a_bar_tab_first_and_moves_the_page() {
        let mut control = control();
        let mut backend = Backend::default();
        assert_eq!(control.tab_move(2, 7, 8, &mut backend), Ok(true));
        assert!(control.entries[2].detached && control.is_split(2) && control.is_page_split(2) && !control.is_page_split(1));
        assert_eq!(control.current, Some(0));
        assert_eq!(backend.0.iter().filter(|line| line.starts_with("notify")).collect::<Vec<_>>(), ["notify 0", "notify 2"]);
        assert_eq!(backend.0.last().unwrap(), "page 2 to (7, 8)");
        backend.0.clear();
        assert_eq!(control.tab_move(2, 9, 9, &mut backend), Ok(true));
        assert_eq!(backend.0, ["page 2 to (9, 9)"]);
        control.entries[1].enabled = false;
        assert_eq!(control.tab_move(1, 0, 0, &mut backend), Ok(false));
        assert_eq!(control.tab_move(9, 0, 0, &mut backend), Ok(false));
        control.flags = 0;
        backend.0.clear();
        assert_eq!(control.tab_move(0, 1, 1, &mut backend), Ok(true));
        assert!(backend.0.is_empty() && !control.entries[0].detached);
    }

    #[test]
    fn restore_detached_1002d980_preserves_selection_and_disabled_pressed_states() {
        for detached in 0..8 {
            for disabled in 0..8 {
                for current in [None, Some(10), Some(30)] {
                    let mut control = TabControl {
                        entries: (0..3).map(|index| TabEntry { page: Some((index + 1) * 10),
                            detached: detached & (1 << index) != 0, enabled: disabled & (1 << index) == 0,
                            pressed: index % 2 == 0, org: (0, 0) }).collect(), current, flags: 1,
                    };
                    let previous = control.clone();
                    let mut backend = Backend::default();
                    control.restore_detached(&mut backend).unwrap();
                    let expected: Vec<String> = (0..3).filter(|index| detached & (1 << index) != 0)
                        .flat_map(|index| [format!("restore button {index}"),
                            format!("move page {} {:?}", (index + 1) * 10, (index + 11, -index - 12)),
                            format!("notify {}", index + 3)]).collect();
                    assert_eq!(backend.0, expected);
                    assert_eq!(control.current, previous.current);
                    assert_eq!(control.flags, previous.flags);
                    for (entry, before) in control.entries.iter().zip(&previous.entries) {
                        assert!(!entry.detached);
                        assert_eq!((entry.page, entry.enabled, entry.pressed), (before.page, before.enabled, before.pressed));
                    }
                    backend.0.clear();
                    control.restore_detached(&mut backend).unwrap();
                    assert!(backend.0.is_empty());
                }
            }
        }
    }

    #[test]
    fn restore_detached_1002d980_reports_missing_page_after_restoring_button() {
        let mut control = TabControl { entries: vec![TabEntry { page: None, enabled: false, detached: true, pressed: true, org: (0, 0) }], ..Default::default() };
        let mut backend = Backend::default();
        assert_eq!(control.restore_detached(&mut backend), Err(TabControlError::MissingPage(0)));
        assert!(!control.entries[0].detached);
        assert_eq!(backend.0, ["restore button 0"]);
        TabControl::default().restore_detached(&mut Backend::default()).unwrap();
    }

    #[test]
    fn tab_release_1002dba0_reattaches_before_press_and_moves_page_after_notification() {
        let mut control = control();
        control.entries[2].detached = true;
        let mut backend = Backend::default();
        let current = TabRect { x: 10, y: 20, width: 74.0, height: 23.0 };
        let saved = TabRect { x: 20, y: 30, ..current };
        assert_eq!(control.release(2, current, saved, &mut backend), Ok(true));
        assert!(!control.entries[2].detached);
        assert_eq!(control.current, Some(2));
        assert_eq!(backend.0, ["move button 2 (10, 10)", "sound 5", "show 0 false", "show 1 false", "show 2 false", "show 2 true", "page 2", "button 2", "raise 2", "notify 2", "sound 5", "move page 2 (10, 10)", "notify 5"]);
    }

    #[test]
    fn tab_release_1002dba0_does_nothing_for_attached_disabled_or_nonintersecting() {
        for enabled in [false, true] {
            for detached in [false, true] {
                let mut control = control();
                control.entries[1].enabled = enabled;
                control.entries[1].detached = detached;
                let before = control.clone();
                let mut backend = Backend::default();
                let current = TabRect { x: 0, y: 0, width: 74.0, height: 23.0 };
                let saved = TabRect { x: 100, ..current };
                assert_eq!(control.release(1, current, saved, &mut backend), Ok(enabled));
                assert_eq!(control.release(99, current, current, &mut backend), Ok(false));
                assert_eq!(control, before);
                assert!(backend.0.is_empty());
            }
        }
    }

    #[test]
    fn tab_rect_1002d770_includes_borders_but_is_not_symmetric_overlap() {
        let outer = TabRect { x: 0, y: 0, width: 10.0, height: 10.0 };
        let inner = TabRect { x: 2, y: 2, width: 2.0, height: 2.0 };
        assert!(outer.contains_saved_corner(inner));
        assert!(!inner.contains_saved_corner(outer));
        for (x, y) in [(10, 0), (0, 10), (10, 10), (-10, -10)] {
            assert!(outer.contains_saved_corner(TabRect { x, y, ..outer }));
        }
        assert!(!outer.contains_saved_corner(TabRect { x: 11, ..outer }));
        assert!(outer.contains_saved_corner(TabRect { x: -2, y: -2, width: 2.9, height: 2.9 }));
        assert!(!outer.contains_saved_corner(TabRect { x: -2, y: -2, width: 0.9, height: 0.9 }));
    }

    fn control() -> TabControl {
        TabControl {
            entries: (0..3).map(|page| TabEntry { page: Some(page), enabled: true, detached: false, pressed: page == 0, org: (0, 0) }).collect(),
            current: Some(0), flags: 1,
        }
    }

    impl TabEnableBackend for Backend {
        fn enable_button(&mut self, index: usize, enabled: bool) { self.0.push(format!("enable {index} {enabled}")); }
        fn button_flags(&mut self, index: usize) -> u32 { self.0.push(format!("flags {index}")); 0x80000011 }
        fn set_button_flags(&mut self, index: usize, flags: u32) { self.0.push(format!("set flags {index} {flags:x}")); }
    }

    #[test]
    fn tab_ctor_1002e7a0_starts_empty_and_select_1002ddc0_has_no_press_effects() {
        let empty = TabControl::default();
        assert!(empty.entries.is_empty());
        assert_eq!(empty.current, None);
        assert_eq!(empty.flags, 0);
        let mut control = control();
        control.entries[1].detached = true;
        control.entries[1].pressed = true;
        let mut backend = Backend::default();
        assert_eq!(control.select(2, true, &mut backend), Ok(()));
        assert_eq!(backend.0, ["show 0 false", "show 1 false", "show 2 false", "show 2 true", "notify 2"]);
        assert_eq!(control.current, Some(2));
        assert!(control.entries[1].detached);
        assert!(!control.entries[1].pressed);
        assert!(control.entries[2].pressed);
        backend.0.clear();
        control.entries[2].enabled = false;
        let before = control.clone();
        assert_eq!(control.select(2, true, &mut backend), Ok(()));
        assert_eq!(control.select(99, true, &mut backend), Ok(()));
        assert_eq!(control, before);
        assert!(backend.0.is_empty());
        control.entries[1].page = None;
        assert_eq!(control.select(0, false, &mut backend), Err(TabControlError::MissingPage(1)));
        assert!(backend.0.is_empty());
    }

    #[test]
    fn tab_disable_1002d7f0_changes_only_enable_and_conditional_button_flag() {
        for flags in [0, 1, 2, 3] {
            for disabled in [false, true] {
                let mut control = control();
                control.flags = flags;
                let mut backend = Backend::default();
                control.disable_tab(0, disabled, &mut backend);
                assert_eq!(control.entries[0].enabled, !disabled);
                assert_eq!(control.current, Some(0));
                assert!(control.entries[0].pressed);
                let mut expected = vec![format!("enable 0 {}", !disabled)];
                if flags & 1 != 0 {
                    expected.push("flags 0".into());
                    expected.push(format!("set flags 0 {:x}", if disabled { 0x80000011_u32 } else { 0x80400011_u32 }));
                }
                assert_eq!(backend.0, expected);
                let before = control.clone();
                backend.0.clear();
                control.disable_tab(99, disabled, &mut backend);
                assert_eq!(control, before);
                assert!(backend.0.is_empty());
            }
        }
    }

    #[test]
    fn tab_press_1002e5a0_orders_visibility_focus_notification_and_both_sounds() {
        let mut control = control();
        let mut backend = Backend::default();
        assert!(control.press(2, true, &mut backend));
        assert_eq!(backend.0, ["sound 5", "show 0 false", "show 1 false", "show 2 false", "show 2 true", "page 2", "button 2", "raise 2", "notify 2", "sound 5"]);
        assert_eq!(control.current, Some(2));
        assert_eq!(control.entries.iter().map(|e| e.pressed).collect::<Vec<_>>(), [false, false, true]);
        backend.0.clear();
        control.flags = 0;
        control.press(2, false, &mut backend);
        assert_eq!(backend.0, ["show 0 false", "show 1 false", "show 2 false", "show 2 true", "page 2", "sound 5"]);
    }

    #[test]
    fn tab_press_1002e5a0_preserves_detached_pages_and_checks_first_identity() {
        let mut control = control();
        control.entries[1].detached = true;
        let mut backend = Backend::default();
        control.press(1, false, &mut backend);
        assert_eq!(control.current, Some(0));
        assert!(control.entries[0].pressed && control.entries[1].pressed);
        assert_eq!(backend.0, ["sound 5", "show 1 true", "page 1", "button 1", "raise 1", "sound 5"]);
        backend.0.clear();
        control.entries[2].page = Some(1);
        control.press(2, false, &mut backend);
        assert_eq!(control.current, Some(0));
        assert!(!backend.0.iter().any(|e| e.ends_with("false")));
        backend.0.clear();
        control.entries[2].page = Some(2);
        control.press(2, false, &mut backend);
        assert!(!backend.0.contains(&"show 1 false".into()));
        assert!(control.entries[1].pressed);
        assert_eq!(control.current, Some(2));
    }

    #[test]
    fn tab_press_1002e5a0_null_disabled_and_out_of_range_branches() {
        let mut control = control();
        let mut backend = Backend::default();
        control.entries[1].enabled = false;
        let before = control.clone();
        for index in [1, 3, usize::MAX] { assert!(control.press(index, true, &mut backend)); }
        assert_eq!(control, before);
        assert!(backend.0.is_empty());
        control.entries[2].page = None;
        control.press(2, true, &mut backend);
        assert_eq!(control.current, None);
        assert_eq!(backend.0, ["sound 5", "show 0 false", "show 1 false", "notify 2", "sound 5"]);
    }

    #[test]
    fn tab_refresh_1002e380_preserves_state_and_repeats_show_for_detached_current() {
        let mut control = control();
        control.entries[0].detached = true;
        control.entries[1].page = None;
        let before = control.clone();
        let mut backend = Backend::default();
        control.refresh_pages(&mut backend);
        assert_eq!(backend.0, ["show 0 true", "show 2 false", "show 0 true"]);
        assert_eq!(control, before);
        control.current = None;
        backend.0.clear();
        control.refresh_pages(&mut backend);
        assert_eq!(backend.0, ["show 0 true", "show 2 false"]);
    }

    #[test]
    fn move_tabs_1002dd10_shifts_every_bar_position_and_return_delta_reads_it() {
        let mut control = TabControl {
            entries: (0..3)
                .map(|i| TabEntry { page: Some(i), enabled: true, detached: false, pressed: false, org: (10 * i as i32, 5) })
                .collect(),
            ..Default::default()
        };
        control.move_tabs(7, -3);
        let orgs: Vec<_> = control.entries.iter().map(|e| e.org).collect();
        assert_eq!(orgs, [(7, 2), (17, 2), (27, 2)]);
        assert_eq!(control.entries[1].return_delta((20, 0)), (-3, 2));
        control.move_tabs(i32::MAX, 0);
        assert_eq!(control.entries[0].org.0, 7i32.wrapping_add(i32::MAX));
    }
}
