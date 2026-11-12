use killer_ui::draw::DrawList;
use killer_ui::font::BitmapFont;
use killer_ui::minimized::{self, IconView, MinimizeBackend};
use killer_ui::window::{Desktop, Rect, WindowId};
use killer_ui::window_restore::{self, WindowRestoreBackend};

pub struct MinimizedWindows {
    pub party_match: IconView,
    pub party_room: IconView,
    pub gm_event: IconView,
    captured: Option<WindowId>,
    match_position: Option<(i32, i32)>,
    room_position: Option<(i32, i32)>,
    gm_position: Option<(i32, i32)>,
}

impl Default for MinimizedWindows {
    fn default() -> Self {
        Self {
            party_match: IconView::new(WindowId::PARTY_MATCH, minimized::PARTY_MATCH_TEXTURE, ""),
            party_room: IconView::new(WindowId::PARTY_ROOM, minimized::PARTY_MATCH_TEXTURE, ""),
            gm_event: IconView::new(WindowId::GM_EVENT, minimized::GM_EVENT_TEXTURE, ""),
            captured: None,
            match_position: None,
            room_position: None,
            gm_position: None,
        }
    }
}

const OWNERS: [WindowId; 3] = [WindowId::PARTY_MATCH, WindowId::PARTY_ROOM, WindowId::GM_EVENT];

impl MinimizedWindows {
    fn icon(&self, owner: WindowId) -> &IconView {
        match owner {
            WindowId::PARTY_ROOM => &self.party_room,
            WindowId::GM_EVENT => &self.gm_event,
            _ => &self.party_match,
        }
    }

    fn parts(&mut self, owner: WindowId) -> (&mut IconView, &mut Option<(i32, i32)>) {
        match owner {
            WindowId::PARTY_ROOM => (&mut self.party_room, &mut self.room_position),
            WindowId::GM_EVENT => (&mut self.gm_event, &mut self.gm_position),
            _ => (&mut self.party_match, &mut self.match_position),
        }
    }

    fn sync(&mut self, desktop: &Desktop, owner: WindowId) {
        let Some(window) = desktop.window(owner) else { return; };
        let position = (window.rect.x, window.rect.y);
        let (icon, previous) = self.parts(owner);
        if let Some(previous) = *previous {
            icon.rect.x = icon.rect.x.wrapping_add(position.0.wrapping_sub(previous.0));
            icon.rect.y = icon.rect.y.wrapping_add(position.1.wrapping_sub(previous.1));
        } else {
            icon.rect.x = position.0;
            icon.rect.y = position.1;
        }
        *previous = Some(position);
    }

    pub fn sync_owner(&mut self, desktop: &Desktop) {
        for owner in OWNERS {
            self.sync(desktop, owner);
        }
    }

    fn minimize(&mut self, desktop: &mut Desktop, owner: WindowId, title: &str) {
        self.sync(desktop, owner);
        let (icon, _) = self.parts(owner);
        let mut backend = MatchMinimize { icon, desktop, title, owner };
        if owner == WindowId::GM_EVENT {
            minimized::minimize_gm_event(&mut backend);
        } else {
            minimized::minimize_party_match(&mut backend);
        }
    }

    pub fn minimize_gm(&mut self, desktop: &mut Desktop, title: &str) {
        self.minimize(desktop, WindowId::GM_EVENT, title);
    }

    pub fn minimize_match(&mut self, desktop: &mut Desktop, title: &str) {
        self.minimize(desktop, WindowId::PARTY_MATCH, title);
    }

    pub fn minimize_room(&mut self, desktop: &mut Desktop, title: &str) {
        self.minimize(desktop, WindowId::PARTY_ROOM, title);
    }

    pub fn captured(&self) -> bool { self.captured.is_some() }

    fn hit_owner(&self, desktop: &Desktop, x: i32, y: i32) -> Option<WindowId> {
        for window in desktop.windows.iter().rev() {
            if OWNERS.contains(&window.id) && self.icon(window.id).hit(x, y) { return Some(window.id); }
            if window.visible && window.rect.contains(x, y) { return None; }
        }
        None
    }

    pub fn hit(&self, desktop: &Desktop, x: i32, y: i32) -> bool {
        self.hit_owner(desktop, x, y).is_some()
    }

    pub fn hide_match(&mut self) {
        self.party_match.hide();
        if self.captured == Some(WindowId::PARTY_MATCH) { self.captured = None; }
    }

    pub fn room_icon_visible(&self) -> bool { self.party_room.visible }

    pub fn blink_room(&mut self) { self.party_room.start_blink(); }

    pub fn tick(&mut self, elapsed_ms: u32) {
        let seconds = elapsed_ms as f32 * 0.001;
        self.party_match.tick(seconds);
        self.party_room.tick(seconds);
        self.gm_event.tick(seconds);
    }

    pub fn mouse_down(&mut self, desktop: &mut Desktop, x: i32, y: i32) -> bool {
        let Some(owner) = self.hit_owner(desktop, x, y) else { return false; };
        desktop.bring_to_front(owner);
        self.parts(owner).0.mouse_down(x, y);
        self.captured = Some(owner);
        true
    }

    pub fn mouse_move(&mut self, desktop: &Desktop, x: i32, y: i32) -> bool {
        for owner in OWNERS {
            let captured = self.captured == Some(owner);
            let icon = self.parts(owner).0;
            if icon.visible {
                icon.mouse_move(x, y, captured);
                icon.update_drag(x, y);
            }
        }
        let over = self.hit_owner(desktop, x, y);
        for owner in OWNERS {
            self.parts(owner).0.hovered = over == Some(owner);
        }
        self.captured.is_some() || over.is_some()
    }

    pub fn mouse_up(&mut self, desktop: &mut Desktop, commands: &mut Vec<crate::ui_keys::UiCommand>) -> bool {
        let Some(owner) = self.captured.take() else { return false; };
        let (icon, position) = self.parts(owner);
        if icon.mouse_up().is_some() {
            window_restore::restore_from_minimized(&mut MatchRestore { icon, desktop, commands, owner });
            *position = desktop.window(owner).map(|window| (window.rect.x, window.rect.y));
        }
        true
    }

    pub fn paint(&self, list: &mut DrawList, font: &BitmapFont, width: i32) {
        for owner in OWNERS {
            let icon = self.icon(owner);
            icon.paint(list);
            icon.paint_tooltip(list, font, width);
        }
    }
}

struct MatchMinimize<'a> {
    icon: &'a mut IconView,
    desktop: &'a mut Desktop,
    title: &'a str,
    owner: WindowId,
}

impl MinimizeBackend for MatchMinimize<'_> {
    fn has_icon(&self) -> bool { true }
    fn set_icon(&mut self, texture: &str, _title_sysstring: u32, tooltip_above: bool) {
        self.icon.texture = texture.to_owned();
        self.icon.title = self.title.to_owned();
        self.icon.tooltip_above = tooltip_above;
    }
    fn owner_x(&self) -> i32 { self.desktop.window(self.owner).map_or(0, |window| window.rect.x) }
    fn move_icon(&mut self, dx: i32, dy: i32) { self.icon.rect.x = self.icon.rect.x.wrapping_add(dx); self.icon.rect.y = self.icon.rect.y.wrapping_add(dy); }
    fn hide_owner(&mut self) { if let Some(window) = self.desktop.window_mut(self.owner) { window.visible = false; } }
    fn show_icon(&mut self) { self.icon.visible = true; }
    fn focus_icon(&mut self) { self.desktop.end_drag(); self.desktop.bring_to_front(self.owner); }
}

struct MatchRestore<'a> {
    commands: &'a mut Vec<crate::ui_keys::UiCommand>,
    icon: &'a mut IconView,
    desktop: &'a mut Desktop,
    owner: WindowId,
}

impl WindowRestoreBackend for MatchRestore<'_> {
    fn play_sound(&mut self, id: u32) {
        if id == 5 { self.commands.push(crate::ui_keys::UiCommand::InterfaceSound("itemsound.window_open")); }
    }
    fn owner_is_inventory(&mut self) -> bool { false }
    fn visible(&mut self, window: WindowId) -> bool { self.desktop.window(window).is_some_and(|window| window.visible) }
    fn shop_mode(&mut self, _window: WindowId) -> i32 { 0 }
    fn minimized_position(&mut self) -> (i32, i32) { (self.icon.rect.x, self.icon.rect.y) }
    fn window_size(&mut self) -> (f32, f32) { self.desktop.window(self.owner).map_or((0.0, 0.0), |window| (window.rect.w as f32, window.rect.h as f32)) }
    fn set_window_size(&mut self, minimized: bool, rect: Rect, _modes: (i32, i32, i32)) {
        if minimized {
            if let Some(window) = self.desktop.window(self.owner) {
                self.icon.rect.x = window.rect.x.wrapping_add(rect.x);
                self.icon.rect.y = window.rect.y.wrapping_add(rect.y);
            }
            if rect.w != 0 { self.icon.rect.w = rect.w; }
            if rect.h != 0 { self.icon.rect.h = rect.h; }
        }
        else if let Some(window) = self.desktop.window_mut(self.owner) { window.rect = rect; }
    }
    fn show_window(&mut self) { if let Some(window) = self.desktop.window_mut(self.owner) { window.visible = true; } }
    fn focus_window(&mut self) { self.desktop.bring_to_front(self.owner); }
    fn hide_minimized(&mut self) { self.icon.hide(); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::window::Window;

    #[test]
    fn minimizing_the_gm_event_window_hides_it_and_shows_the_menuicon4_icon_with_the_tip_above() {
        let mut desktop = Desktop::new((1280, 800));
        let mut window = Window::new(WindowId::GM_EVENT, Rect::new(-10, 40, 700, 500), Rect::default());
        window.visible = true;
        desktop.windows.push(window);
        let mut minimized = MinimizedWindows::default();
        minimized.minimize_gm(&mut desktop, "tip");
        assert!(!desktop.window(WindowId::GM_EVENT).is_some_and(|window| window.visible));
        assert!(minimized.gm_event.visible);
        assert_eq!(minimized.gm_event.texture, minimized::GM_EVENT_TEXTURE);
        assert_eq!((minimized.gm_event.title.as_str(), minimized.gm_event.tooltip_above), ("tip", true));
        assert_eq!(minimized.gm_event.rect.x, 0);
        assert!(minimized.hit(&desktop, 5, 45));
    }
}
