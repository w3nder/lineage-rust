use killer_assets::client::DatTables;
use killer_ui::item_grid::GridEntry;
use killer_ui::pet_wnd::summoned::SummonedWndView;
use killer_ui::pet_wnd::{PetHit, PetItem, PetStats, PetWndView};
use killer_ui::scroll::{ScrollHit, line_down, line_up};
use killer_ui::window::{Desktop, Rect, Window, WindowId};
use killer_world::{NetCommand, World, WorldEvent};

pub struct PetWindow {
    pub view: PetWndView,
    pub summoned: SummonedWndView,
    summoned_pressed: Option<PetHit>,
    pressed: Option<PetHit>,
    order: Vec<i32>,
    pub commands: Vec<crate::ui_keys::UiCommand>,
}

pub fn create_window(desktop: &mut Desktop, saved: Option<(i32, i32)>) {
    let (x, y) = saved.unwrap_or(killer_ui::pet_wnd::DEFAULT_POSITION);
    let (w, h) = killer_ui::pet_wnd::SIZE;
    let mut window = Window::new(WindowId::PET_WINDOW, Rect::new(x, y, w, h), killer_ui::pet_wnd::grip())
        .with_initial_position(killer_ui::pet_wnd::initial_position());
    window.visible = false;
    desktop.windows.push(window);
}

pub fn create_summoned_window(desktop: &mut Desktop, saved: Option<(i32, i32)>) {
    let (x, y) = saved.unwrap_or(killer_ui::pet_wnd::DEFAULT_POSITION);
    let (w, h) = killer_ui::pet_wnd::summoned::SIZE;
    let mut window = Window::new(WindowId::SUMMONED_WINDOW, Rect::new(x, y, w, h), killer_ui::pet_wnd::summoned::grip())
        .with_initial_position(killer_ui::pet_wnd::initial_position());
    window.visible = false;
    desktop.windows.push(window);
}

fn show(desktop: &mut Desktop, id: WindowId, visible: bool) {
    if let Some(window) = desktop.window_mut(id) {
        window.visible = visible;
    }
    if visible
        && let Some(index) = desktop.windows.iter().position(|w| w.id == id)
    {
        let window = desktop.windows.remove(index);
        desktop.windows.push(window);
    }
}

impl PetWindow {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self { view: PetWndView::new(&lookup), summoned: SummonedWndView::new(&lookup), summoned_pressed: None, pressed: None, order: Vec::new(), commands: Vec::new() }
    }

    pub fn world_event(&mut self, event: &WorldEvent, desktop: &mut Desktop) {
        match event {
            WorldEvent::PetStatusShow(packet) if packet.summon_type == l2_net::game::SUMMON_SERVITOR => show(desktop, WindowId::SUMMONED_WINDOW, true),
            WorldEvent::PetStatusShow(_) => show(desktop, WindowId::PET_WINDOW, true),
            WorldEvent::PetDelete(_) => {
                show(desktop, WindowId::PET_WINDOW, false);
                show(desktop, WindowId::SUMMONED_WINDOW, false);
            }
            _ => {}
        }
    }

    pub fn sync(&mut self, world: &World, tables: Option<&DatTables>) {
        let items: Vec<PetItem> = world.pet_inventory.items.iter().filter(|item| tables.is_none_or(|tables| u32::try_from(item.item_id).is_ok_and(|id| tables.item_icons.get(id).is_some()))).map(|item| {
            let id = u32::try_from(item.item_id).ok();
            let icon = id.and_then(|id| tables?.item_icons.get(id));
            let name = id.and_then(|id| tables?.itemname.get(&id));
            PetItem {
                grid: GridEntry {
                    object_id: item.object_id, item_id: item.item_id, count: item.count, enchant: i32::from(item.enchant),
                    icon: icon.map(|i| i.icons[0].trim().to_string()).unwrap_or_default(),
                    name: name.map(|n| n.name.clone()).unwrap_or_default(),
                    add_name: name.map(|n| n.add_name.clone()).unwrap_or_default(),
                    description: name.map(|n| n.description.clone()).unwrap_or_default(),
                    weight: icon.map_or(0, |i| i.weight as i32),
                },
                equipped: item.equipped,
            }
        }).collect();
        self.order.retain(|id| items.iter().any(|item| item.grid.object_id == *id));
        for item in &items {
            if !self.order.contains(&item.grid.object_id) {
                self.order.push(item.grid.object_id);
            }
        }
        let mut items = items;
        items.sort_by_key(|item| self.order.iter().position(|id| *id == item.grid.object_id));
        if items != self.view.items {
            self.view.set_items(items);
        }
        let Some(pet) = &world.pet else { return };
        let info = &pet.info;
        let npc_name = || {
            u32::try_from(info.npc_id).ok().and_then(|id| tables?.npcname.get(&id)).map(|n| n.name.clone()).unwrap_or_default()
        };
        self.view.name_button_visible = info.name.is_empty();
        self.summoned.stats = PetStats {
            name: if info.name.is_empty() { npc_name() } else { info.name.clone() },
            hp: info.hp, max_hp: info.max_hp, mp: info.mp, max_mp: info.max_mp, level: info.level,
            detail: [
                [info.p_atk, info.p_def, info.accuracy, info.critical, info.atk_spd, info.soulshots_per_hit],
                [info.m_atk, info.m_def, info.evasion, info.speed, info.cast_spd, info.spiritshots_per_hit],
            ],
            ..Default::default()
        };
        self.view.stats = PetStats {
            name: if info.name.is_empty() { npc_name() } else { info.name.clone() },
            hp: info.hp,
            max_hp: info.max_hp,
            mp: info.mp,
            max_mp: info.max_mp,
            sp: info.sp,
            level: info.level,
            exp: info.exp,
            exp_this_level: info.exp_this_level,
            exp_next_level: info.exp_next_level,
            fed: info.fed,
            max_fed: info.max_fed,
            load: info.load,
            max_load: info.max_load,
            detail: [
                [info.p_atk, info.p_def, info.accuracy, info.critical, info.atk_spd, info.soulshots_per_hit],
                [info.m_atk, info.m_def, info.evasion, info.speed, info.cast_spd, info.spiritshots_per_hit],
            ],
        };
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<i32> {
        self.pressed = self.view.hit(x, y);
        self.view.action_pressed = match self.pressed { Some(PetHit::Action(id)) => Some(id), _ => None };
        if let Some(PetHit::Tab(index)) = self.pressed {
            self.view.press_tab(index);
        }
        if let Some(PetHit::Item(index)) = self.pressed {
            self.view.grid.selected = Some(index);
        }
        if let Some(PetHit::ItemScroll(hit)) = self.pressed {
            let grid = &mut self.view.grid;
            grid.scroll.up_pressed = hit == ScrollHit::Up;
            grid.scroll.down_pressed = hit == ScrollHit::Down;
            if let ScrollHit::Slider(pos) = hit {
                grid.scroll.slider.mouse_down(pos);
            }
        }
        self.view.name_pressed = self.pressed == Some(PetHit::Name);
        self.view.action_pressed
    }

    pub fn mouse_move(&mut self, y: i32) -> bool {
        let Some(PetHit::ItemScroll(ScrollHit::Slider(_))) = self.pressed else { return false };
        let top = killer_ui::pet_wnd::PAGE_RECT.y + self.view.grid.rect.y + killer_ui::scroll::BUTTON_SIZE;
        if let Some(first) = self.view.grid.scroll.slider.mouse_move(y - top) {
            self.view.grid.scroll_row = first.max(0) as usize;
        }
        true
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        let rect = self.view.grid.rect.offset(killer_ui::pet_wnd::PAGE_RECT.x, killer_ui::pet_wnd::PAGE_RECT.y);
        if self.view.page() != killer_ui::pet_wnd::PAGE_ITEMS || !rect.contains(x, y) {
            return false;
        }
        let grid = &self.view.grid;
        let first = killer_ui::inventory::item_wnd_wheel(grid.scroll_row as i32, grid.rows as i32, grid.total_rows(self.view.items.len()), delta);
        self.view.grid.set_first_row(first);
        true
    }

    pub fn summoned_mouse_down(&mut self, x: i32, y: i32) -> Option<i32> {
        self.summoned_pressed = self.summoned.hit(x, y);
        if let Some(PetHit::Tab(index)) = self.summoned_pressed {
            self.summoned.press_tab(index);
        }
        self.summoned.action_pressed = match self.summoned_pressed { Some(PetHit::Action(id)) => Some(id), _ => None };
        self.summoned.action_pressed
    }

    pub fn double_click(&mut self, x: i32, y: i32) -> bool {
        let Some(PetHit::Item(index)) = self.view.hit(x, y) else { return false };
        let object_id = self.view.items[index].grid.object_id;
        self.commands.push(crate::ui_keys::UiCommand::Net(NetCommand::PetUseItem { object_id }));
        true
    }

    pub fn move_item(&mut self, object_id: i32, target: usize) -> bool {
        if !self.view.move_item(object_id, target) {
            return false;
        }
        self.order = self.view.items.iter().map(|item| item.grid.object_id).collect();
        true
    }

    pub fn mouse_up(&mut self, over: Option<(WindowId, i32, i32)>) {
        if self.pressed == Some(PetHit::Name)
            && let Some((WindowId::PET_WINDOW, x, y)) = over
            && self.view.hit(x, y) == Some(PetHit::Name)
        {
            self.commands.push(crate::ui_keys::UiCommand::OpenPetNameDialog);
        }
        if let Some(PetHit::ItemScroll(hit)) = self.pressed {
            let released = match over { Some((WindowId::PET_WINDOW, x, y)) => self.view.hit(x, y), _ => None };
            let count = self.view.items.len();
            let grid = &mut self.view.grid;
            grid.scroll.up_pressed = false;
            grid.scroll.down_pressed = false;
            grid.scroll.slider.mouse_up();
            if released == self.pressed {
                let (first, rows, total) = (grid.scroll_row as i32, grid.rows as i32, grid.total_rows(count));
                match hit {
                    ScrollHit::Up => grid.set_first_row(line_up(first, rows, total)),
                    ScrollHit::Down => grid.set_first_row(line_down(first, rows, total)),
                    ScrollHit::Slider(_) => {}
                }
            }
        }
        self.pressed = None;
        self.view.name_pressed = false;
        self.view.action_pressed = None;
        self.summoned_pressed = None;
        self.summoned.action_pressed = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2_net::game::{PetDelete, PetStatusShow, SUMMON_PET, SUMMON_SERVITOR};

    #[test]
    fn the_pet_window_opens_for_pets_only_and_closes_when_the_pet_goes_away() {
        let mut desktop = Desktop::new((1024, 768));
        create_window(&mut desktop, None);
        let mut pet = PetWindow::new(|id| Some(format!("s{id}")));
        assert_eq!(desktop.window(WindowId::PET_WINDOW).unwrap().rect, Rect::new(172, 76, 256, 229));
        create_summoned_window(&mut desktop, None);
        pet.world_event(&WorldEvent::PetStatusShow(PetStatusShow { summon_type: SUMMON_SERVITOR }), &mut desktop);
        assert!(!desktop.window(WindowId::PET_WINDOW).unwrap().visible);
        assert!(desktop.window(WindowId::SUMMONED_WINDOW).unwrap().visible);
        assert_eq!(desktop.window(WindowId::SUMMONED_WINDOW).unwrap().rect, Rect::new(172, 76, 256, 201));
        pet.world_event(&WorldEvent::PetStatusShow(PetStatusShow { summon_type: SUMMON_PET }), &mut desktop);
        assert!(desktop.window(WindowId::PET_WINDOW).unwrap().visible);
        pet.world_event(&WorldEvent::PetDelete(PetDelete { summon_type: SUMMON_PET, object_id: 1 }), &mut desktop);
        assert!(!desktop.window(WindowId::PET_WINDOW).unwrap().visible);
        assert!(!desktop.window(WindowId::SUMMONED_WINDOW).unwrap().visible);
    }

    #[test]
    fn the_stats_come_from_the_pet_info_and_tabs_switch_on_press() {
        let mut world = World::default();
        let mut info = l2_net::game::PetInfo { name: String::new(), level: 12, p_atk: 33, spiritshots_per_hit: 2, ..Default::default() };
        info.hp = 10;
        world.pet = Some(killer_world::Pet { info, max_time: 0, remaining_time: 0 });
        let mut pet = PetWindow::new(|_| None);
        pet.sync(&world, None);
        assert!(pet.view.name_button_visible);
        pet.mouse_down(170, 30);
        pet.mouse_up(Some((WindowId::PET_WINDOW, 171, 31)));
        assert_eq!(pet.commands.pop(), Some(crate::ui_keys::UiCommand::OpenPetNameDialog));
        assert_eq!((pet.view.stats.level, pet.view.stats.hp, pet.view.stats.detail[0][0], pet.view.stats.detail[1][5]), (12, 10, 33, 2));
        pet.mouse_down(12 + 2 * 75 + 5, 120);
        assert_eq!(pet.view.page(), 2);
        world.pet_inventory.items = vec![l2_net::game::InventoryItem { object_id: 77, item_id: 2375, count: 1, equipped: true, ..Default::default() }];
        pet.sync(&world, None);
        assert!(pet.view.items[0].equipped);
        assert!(pet.double_click(20, 150));
        assert!(matches!(pet.commands.as_slice(), [crate::ui_keys::UiCommand::Net(NetCommand::PetUseItem { object_id: 77 })]));
        world.pet_inventory.items = (0..30).map(|i| l2_net::game::InventoryItem { object_id: i, item_id: 57, count: 1, ..Default::default() }).collect();
        pet.sync(&world, None);
        assert!(pet.wheel(30, 160, -killer_ui::inventory::WHEEL_DELTA));
        assert_eq!(pet.view.grid.scroll_row, 1);
        let rect = pet.view.grid.rect.offset(killer_ui::pet_wnd::PAGE_RECT.x, killer_ui::pet_wnd::PAGE_RECT.y);
        let (dx, dy) = (rect.x + rect.w - 4, rect.y + rect.h - 4);
        pet.mouse_down(dx, dy);
        pet.mouse_up(Some((WindowId::PET_WINDOW, dx, dy)));
        assert_eq!(pet.view.grid.scroll_row, 2);
    }
}
