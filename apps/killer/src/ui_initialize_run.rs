use killer_ui::interface_initialize::{InterfaceInitializeBackend, RootWindowSlot, initialize_interface};
use killer_ui::positioning_rect::PositioningRect;
use killer_ui::secondary_layout::{SecondaryLayoutBackend, default_position};
use killer_ui::window::WindowId;
use killer_ui::window_children::ChildNodeId;
use killer_ui::window_restore::x87_to_int;

use crate::ui2d::GameUi;
use crate::ui_initialize::{InitializeError, InitializeGeometry, child_instance};

const CHAT_VTABLE: u32 = 0x101adc48;

struct Runner<'a> {
    ui: &'a mut GameUi,
    geometry: InitializeGeometry,
    error: Option<InitializeError>,
}

impl Runner<'_> {
    fn screen(&self) -> PositioningRect {
        let (w, h) = self.ui.desktop.size;
        PositioningRect { x: 0, y: 0, width: w as f32, height: h as f32 }
    }

    fn fail(&mut self, error: InitializeError) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }

    fn sync_geometry(&mut self) {
        if let Err(error) = self.geometry.import_desktop(&self.ui.desktop, &self.ui.window_registry) {
            self.fail(error);
        }
    }
}

impl SecondaryLayoutBackend for Runner<'_> {
    type Window = WindowId;
    fn root_rect(&mut self) -> PositioningRect { self.screen() }
    fn settings_window(&mut self) -> Option<WindowId> { self.ui.desktop.window(WindowId::OPTION).map(|w| w.id) }
    fn message_window(&mut self) -> Option<WindowId> { None }
    fn set_position(&mut self, window: WindowId, x: i32, y: i32, root: PositioningRect) {
        if let Some(window) = self.ui.desktop.window_mut(window) {
            window.rect.x = root.x.wrapping_add(x);
            window.rect.y = root.y.wrapping_add(y);
        }
    }
    fn resize(&mut self, _window: WindowId, _width: f32, _height: f32) {}
}

impl InterfaceInitializeBackend for Runner<'_> {
    type ChildNode = ChildNodeId;

    fn has_root(&mut self) -> bool { true }

    fn minimized_visible(&mut self, _slot: RootWindowSlot) -> bool { false }

    fn restore_minimized(&mut self, _slot: RootWindowSlot) {}

    fn has_secondary_root(&mut self) -> bool { true }

    fn layout_secondary_root(&mut self) {
        let _ = default_position(self);
    }

    fn resize_root_to_screen(&mut self) {}

    fn reset_shortcut_orientation(&mut self) {
        if !self.ui.shortcut.vertical
            && let Some(window) = self.ui.desktop.window_mut(WindowId::SHORTCUT)
        {
            window.rect = self.ui.shortcut.rotate(window.rect);
        }
    }

    fn rearrange_abnormal_status(&mut self) {}

    fn request_width(&mut self, slot: RootWindowSlot, width: i32) {
        if slot != RootWindowSlot(0x128) {
            return;
        }
        let Some(window) = self.ui.desktop.window_mut(WindowId::STATUS) else { return };
        let delta = x87_to_int(f64::from(width) - f64::from(window.rect.w as f32));
        window.rect.w = crate::ui_status_resize::status_width_after(window.rect.w, delta);
    }

    fn first_child(&mut self) -> Option<ChildNodeId> {
        self.sync_geometry();
        let registry = &self.ui.window_registry;
        registry.node(&registry.primary)?.children.first()
    }

    fn child_is_chat(&mut self, node: ChildNodeId) -> bool {
        let registry = &self.ui.window_registry;
        child_instance(registry, &registry.primary, node)
            .ok()
            .and_then(|instance| registry.node(&instance).map(|child| child.vtable))
            == Some(CHAT_VTABLE)
    }

    fn restore_initial_position(&mut self, node: ChildNodeId) {
        let registry = &self.ui.window_registry;
        let instance = match child_instance(registry, &registry.primary, node) {
            Ok(instance) => instance,
            Err(error) => return self.fail(error),
        };
        if let Err(error) = self.geometry.restore_initial_position(&mut self.ui.desktop, &self.ui.window_registry, &instance) {
            self.fail(error);
        }
    }

    fn next_child(&mut self, node: ChildNodeId) -> Option<ChildNodeId> {
        let registry = &self.ui.window_registry;
        registry.node(&registry.primary)?.children.next(node)
    }

    fn reset_chat_geometry(&mut self, height: i32) {
        let screen_height = self.ui.desktop.size.1;
        let Some(window) = self.ui.desktop.window_mut(WindowId::CHAT) else { return };
        let captured = window.rect.h;
        window.rect.x = 0;
        window.rect.y = x87_to_int(f64::from(screen_height) - f64::from(captured as f32));
        let delta = x87_to_int(f64::from(captured as f32) - f64::from(height as f32));
        if let Some(step) = self.ui.chat.resize_by(delta) {
            window.rect.y = window.rect.y.wrapping_sub(step);
            window.rect.h = self.ui.chat.height;
        }
    }

    fn restore_chat_tabs(&mut self) {}

    fn reposition_notifications(&mut self) {}
}

pub fn initialize(ui: &mut GameUi) -> Result<(), InitializeError> {
    let geometry = InitializeGeometry::capture(&ui.desktop, &ui.window_registry)?;
    let mut runner = Runner { ui, geometry, error: None };
    initialize_interface(&mut runner);
    if let Some(error) = runner.error {
        return Err(error);
    }
    runner.ui.follow_chat();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::font::{BitmapFont, Glyphs};
    use killer_ui::ini::{ChatFilter, Options, parse_ini};

    fn new_ui() -> GameUi {
        let font = BitmapFont::new(String::new(), Glyphs { tex_width: 1, tex_height: 1, glyphs: Default::default() });
        GameUi::new(font.clone(), font, Options::default(), Default::default(), ChatFilter::from_ini(&parse_ini("")), (1558, 933)).unwrap()
    }

    #[test]
    fn initialize_10073b70_restores_the_creation_layout_without_touching_saved_options() {
        let mut ui = new_ui();
        let fresh = new_ui();
        let options = ui.options.clone();
        for window in ui.desktop.windows.iter_mut() {
            window.rect.x += 37;
            window.rect.y -= 11;
        }
        if let Some(status) = ui.desktop.window_mut(WindowId::STATUS) { status.rect.w = 300; }
        if ui.shortcut.vertical {
            let rect = ui.desktop.window(WindowId::SHORTCUT).unwrap().rect;
            let rotated = ui.shortcut.rotate(rect);
            ui.desktop.window_mut(WindowId::SHORTCUT).unwrap().rect = rotated;
        }
        initialize(&mut ui).unwrap();
        let rect = |ui: &GameUi, id| ui.desktop.window(id).unwrap().rect;
        for id in [WindowId::MAIN, WindowId::INVENTORY, WindowId::TARGET, WindowId::MENU, WindowId::RADAR, WindowId::PARTY_STATUS, WindowId::MAP_COVER] {
            let initial = ui.desktop.window(id).unwrap().initial_position.unwrap();
            assert!(initial.enabled, "{id:?}");
            let expected = rect(&fresh, id);
            let actual = rect(&ui, id);
            assert_eq!((actual.x, actual.y), (expected.x, expected.y), "{id:?}");
        }
        assert_eq!(rect(&ui, WindowId::STATUS).w, 172);
        assert!(ui.shortcut.vertical);
        assert_eq!((rect(&ui, WindowId::SHORTCUT).w, rect(&ui, WindowId::SHORTCUT).h), (46, 504));
        let chat = rect(&ui, WindowId::CHAT);
        assert_eq!((chat.x, chat.y + chat.h), (0, 933));
        let settings = rect(&ui, WindowId::OPTION);
        assert_eq!((settings.x, settings.y), (1558 / 2 - 128, x87_to_int(933.0 * 0.5 - 200.0)));
        assert_eq!(ui.options, options);
    }

    #[test]
    fn initialize_10073b70_chat_height_goes_through_the_10054da0_handler() {
        for (height, expected) in [(187, 187), (196, 196), (202, 217), (400, 400), (480, 480)] {
            let mut ui = new_ui();
            ui.chat.height = height;
            ui.desktop.window_mut(WindowId::CHAT).unwrap().rect.h = height;
            initialize(&mut ui).unwrap();
            let chat = ui.desktop.window(WindowId::CHAT).unwrap().rect;
            assert_eq!((ui.chat.height, chat.h, chat.y + chat.h), (expected, expected, 933), "{height}");
        }
    }

    #[test]
    fn the_initialize_button_of_the_game_page_runs_the_command() {
        let mut ui = new_ui();
        ui.desktop.window_mut(WindowId::OPTION).unwrap().visible = true;
        ui.keys.option.tabs.control.current = Some(killer_ui::option::TAB_GAME);
        let button = ui.keys.option.game_page.initialize.rect;
        let page = killer_ui::option::PAGE_RECT;
        let (x, y) = (page.x + button.x + 2, page.y + button.y + 2);
        assert_eq!(ui.keys.option.hit(x, y), Some(killer_ui::option::OptionHit::Initialize));
    }
}
