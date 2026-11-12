use killer_ui::font::BitmapFont;
use killer_ui::hero_tower::{CONTENT_ID, HeroContentView, HeroRow, HeroTowerView, TOWER_ID, TowerAction, TowerHit};
use killer_ui::window::{Desktop, WindowId};
use killer_world::{EntityKind, NetCommand, World, WorldEvent};

use crate::ui_keys::UiCommand;

pub struct HeroTowerWindow {
    pub tower: HeroTowerView,
    pub content: HeroContentView,
    last_tick: Option<killer_platform::time::Instant>,
    pub commands: Vec<UiCommand>,
}

impl HeroTowerWindow {
    pub fn new(lookup: impl Fn(u32) -> Option<String> + Copy) -> Self {
        Self { tower: HeroTowerView::new(lookup), content: HeroContentView::new(lookup), last_tick: None, commands: Vec::new() }
    }

    pub fn world_event(&mut self, event: &WorldEvent, world: &World, lookup: &dyn Fn(u32) -> Option<String>, desktop: &mut Desktop) -> bool {
        let WorldEvent::HeroList(list) = event else { return false };
        let crest = |id: i32, map: &std::collections::HashMap<i32, Vec<u8>>, texture: fn(i32) -> String| {
            (id > 0 && map.contains_key(&id)).then(|| texture(id))
        };
        let rows: Vec<HeroRow> = list
            .entries
            .iter()
            .map(|entry| HeroRow {
                name: entry.names[0].clone(),
                class_id: entry.values[0],
                class_name: u32::try_from(entry.values[0])
                    .ok()
                    .and_then(|class| lookup(killer_ui::detail_status::class_name_sysstring(class)))
                    .unwrap_or_default(),
                clan: entry.names[1].clone(),
                clan_crest: crest(entry.values[1], &world.pledge_window.crests, crate::ui2d::crest_texture),
                ally: entry.names[2].clone(),
                ally_crest: crest(entry.values[2], &world.pledge_window.ally_crests, crate::ui2d::ally_crest_texture),
                count: entry.values[3],
            })
            .collect();
        self.tower.set_heroes(&rows);
        show(desktop, TOWER_ID, true);
        let hero = world.me.as_ref().map(|me| matches!(me.kind, EntityKind::Player { hero: true, .. }));
        self.tower.set_owner_hero(hero);
        true
    }

    pub fn mouse_down_tower(&mut self, x: i32, y: i32, font: &BitmapFont, desktop: &mut Desktop) -> bool {
        if self.tower.mouse_down(x, y, font) == Some(TowerHit::Close) {
            show(desktop, TOWER_ID, false);
        }
        true
    }

    pub fn mouse_down_content(&mut self, x: i32, y: i32, desktop: &mut Desktop) -> bool {
        if self.content.mouse_down(x, y) {
            show(desktop, CONTENT_ID, false);
        }
        true
    }

    pub fn blur_outside(&mut self, over: Option<WindowId>) {
        if over != Some(TOWER_ID) {
            self.tower.blur();
        }
    }

    pub fn mouse_move(&mut self, id: WindowId, x: i32, y: i32, font: &BitmapFont) -> bool {
        if id != TOWER_ID {
            return false;
        }
        self.tower.mouse_move(x, y, font);
        true
    }

    pub fn wheel(&mut self, id: WindowId, delta: i16) -> bool {
        if id != TOWER_ID {
            return false;
        }
        self.tower.wheel(delta);
        true
    }

    pub fn double_click(&mut self, x: i32, y: i32, desktop: &mut Desktop) -> bool {
        match self.tower.double_click(x, y) {
            Some(action) => {
                self.action(action, desktop);
                true
            }
            None => false,
        }
    }

    pub fn mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        let (x, y) = over.filter(|(id, _, _)| *id == TOWER_ID).map_or((i32::MIN / 2, i32::MIN / 2), |(_, x, y)| (x, y));
        if let Some(action) = self.tower.mouse_up(x, y) {
            self.action(action, desktop);
        }
        self.content.mouse_up();
    }

    fn action(&mut self, action: TowerAction, desktop: &mut Desktop) {
        match action {
            TowerAction::Close => show(desktop, TOWER_ID, false),
            TowerAction::Words(words) => self.commands.push(UiCommand::Net(NetCommand::WriteHeroWords { words })),
            TowerAction::Diary(command) => self.commands.push(UiCommand::Net(NetCommand::Bypass { command })),
        }
    }

    pub fn key(&mut self, key: killer_ui::chat::InputKey, font: &BitmapFont, desktop: &Desktop) -> bool {
        desktop.window(TOWER_ID).is_some_and(|window| window.visible) && self.tower.key(key, font)
    }

    pub fn shift(&mut self, pressed: bool) -> bool {
        self.tower.shift(pressed)
    }

    pub fn tick(&mut self, now: killer_platform::time::Instant) {
        if let Some(last) = self.last_tick.replace(now) {
            let elapsed = now.saturating_duration_since(last).as_millis().min(u128::from(u32::MAX)) as u32;
            self.tower.tick(elapsed);
        }
    }
}

fn show(desktop: &mut Desktop, id: WindowId, visible: bool) {
    if let Some(window) = desktop.window_mut(id) {
        window.visible = visible;
    }
    if visible && let Some(index) = desktop.windows.iter().position(|window| window.id == id) {
        let window = desktop.windows.remove(index);
        desktop.windows.push(window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::window::{Rect, Window};
    use l2_net::game::{HeroEntry, HeroList};

    fn font() -> BitmapFont {
        BitmapFont::new(
            "L2Font-e.SmallFont-e".into(),
            killer_ui::font::Glyphs::parse(include_bytes!("../../../crates/killer-ui/tests/fixtures/smallfont-e.gly")).unwrap(),
        )
    }

    fn window() -> (HeroTowerWindow, Desktop) {
        let mut desktop = Desktop::new((1280, 800));
        for (id, size, grip) in [
            (TOWER_ID, killer_ui::hero_tower::TOWER_SIZE, killer_ui::hero_tower::tower_grip()),
            (CONTENT_ID, killer_ui::hero_tower::CONTENT_SIZE, killer_ui::hero_tower::content_grip()),
        ] {
            let mut entry = Window::new(id, Rect::new(0, 0, size.0, size.1), grip);
            entry.visible = false;
            desktop.windows.push(entry);
        }
        (HeroTowerWindow::new(|id| Some(format!("s{id}"))), desktop)
    }

    fn list() -> WorldEvent {
        let hero = |name: &str, class: i32, clan_crest: i32| HeroEntry {
            names: [name.into(), "Clan".into(), "Ally".into()],
            values: [class, clan_crest, 0, 5],
        };
        WorldEvent::HeroList(HeroList { entries: vec![hero("A", 88, 7), hero("B", 90, 0)] })
    }

    #[test]
    fn the_packet_fills_the_list_resolves_loaded_crests_and_opens_the_window_every_time() {
        let (mut window, mut desktop) = window();
        let mut world = World::default();
        world.pledge_window.crests.insert(7, vec![1]);
        let lookup = |id: u32| Some(format!("s{id}"));
        assert!(!window.world_event(&WorldEvent::SendManorList(l2_net::game::ManorList { entries: Vec::new() }), &world, &lookup, &mut desktop));
        assert!(window.world_event(&list(), &world, &lookup, &mut desktop));
        assert!(desktop.window(TOWER_ID).unwrap().visible);
        assert_eq!(window.tower.list.count(), 2);
        assert_eq!(window.tower.list.rows[0].cells[1].text, format!("s{}", killer_ui::detail_status::class_name_sysstring(88)));
        assert_eq!(window.tower.list.rows[0].cells[2].icons.len(), 1);
        assert!(window.tower.list.rows[1].cells[2].icons.is_empty());
        desktop.window_mut(TOWER_ID).unwrap().visible = false;
        window.world_event(&list(), &world, &lookup, &mut desktop);
        assert!(desktop.window(TOWER_ID).unwrap().visible);
    }

    #[test]
    fn message_and_view_record_emit_the_native_commands() {
        let (mut window, mut desktop) = window();
        let world = World::default();
        window.world_event(&list(), &world, &|id| Some(format!("s{id}")), &mut desktop);
        let font = font();
        window.tower.words.set_text("hail");
        let message = Rect::new(440, 297, 76, 23);
        window.mouse_down_tower(message.x + 2, message.y + 2, &font, &mut desktop);
        window.mouse_up(Some((TOWER_ID, message.x + 2, message.y + 2)), &mut desktop);
        window.tower.list.select(1);
        let record = Rect::new(518, 297, 76, 23);
        window.mouse_down_tower(record.x + 2, record.y + 2, &font, &mut desktop);
        window.mouse_up(Some((TOWER_ID, record.x + 2, record.y + 2)), &mut desktop);
        assert!(matches!(
            window.commands.as_slice(),
            [UiCommand::Net(NetCommand::WriteHeroWords { words }), UiCommand::Net(NetCommand::Bypass { command })]
                if words == "hail" && command == "_diary?class=90&page=1"
        ));
    }

    #[test]
    fn close_buttons_hide_the_windows() {
        let (mut window, mut desktop) = window();
        window.world_event(&list(), &World::default(), &|id| Some(format!("s{id}")), &mut desktop);
        show(&mut desktop, CONTENT_ID, true);
        let font = font();
        window.mouse_down_tower(580, 5, &font, &mut desktop);
        assert!(!desktop.window(TOWER_ID).unwrap().visible);
        window.mouse_down_content(290, 5, &mut desktop);
        assert!(!desktop.window(CONTENT_ID).unwrap().visible);
    }
}
