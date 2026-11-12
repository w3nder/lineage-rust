#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WindowId(pub u32);

impl WindowId {
    pub const MAIN: WindowId = WindowId(1);
    pub const SSQ: WindowId = WindowId(2);
    pub const PARTY_MATCH: WindowId = WindowId(34);
    pub const SIEGE: WindowId = WindowId(62);
    pub const CALC: WindowId = WindowId(3);
    pub const GM: WindowId = WindowId(55);
    pub const GM_EVENT: WindowId = WindowId(64);
    pub const GM_FENCE: WindowId = WindowId(65);
    pub const GM_INVENTORY: WindowId = WindowId(56);
    pub const GM_CLAN: WindowId = WindowId(61);
    pub const GM_STORE: WindowId = WindowId(60);
    pub const GM_QUEST: WindowId = WindowId(59);
    pub const GM_STATUS: WindowId = WindowId(58);
    pub const GM_SKILLS: WindowId = WindowId(57);
    pub const PETITION_GM: WindowId = WindowId(4);
    pub const PETITION_BEGIN: WindowId = WindowId(5);
    pub const EMPOWER: WindowId = WindowId(6);
    pub const NPC_HTML: WindowId = WindowId(7);
    pub const ITEM_HTML: WindowId = WindowId(8);
    pub const HELP: WindowId = WindowId(26);
    pub const BOARD: WindowId = WindowId(9);
    pub const INVENTORY: WindowId = WindowId(10);
    pub const STATUS: WindowId = WindowId(12);
    pub const TARGET: WindowId = WindowId(13);
    pub const ABNORMAL_STATUS: WindowId = WindowId(14);
    pub const PARTY_STATUS: WindowId = WindowId(15);
    pub const PARTY_ROOM_MAKING: WindowId = WindowId(35);
    pub const PARTY_ROOM: WindowId = WindowId(36);
    pub const PET_WINDOW: WindowId = WindowId(42);
    pub const SUMMONED_WINDOW: WindowId = WindowId(43);
    pub const PET_STATUS: WindowId = WindowId(16);
    pub const SUMMONED_STATUS: WindowId = WindowId(17);
    pub const DELIVERY: WindowId = WindowId(19);
    pub const SELECT_DELIVERY: WindowId = WindowId(20);
    pub const SHOP: WindowId = WindowId(21);
    pub const SHOP_PREVIEW: WindowId = WindowId(22);
    pub const PRIVATE_SHOP: WindowId = WindowId(40);
    pub const MULTISELL: WindowId = WindowId(23);
    pub const QUEST_LIST_INFO: WindowId = WindowId(67);
    pub const MANOR_INFO: WindowId = WindowId(68);
    pub const OLYMPIAD_CONTROL: WindowId = WindowId(78);
    pub const OLYMPIAD_PLAYER_1: WindowId = WindowId(79);
    pub const OLYMPIAD_PLAYER_2: WindowId = WindowId(80);
    pub const OLYMPIAD_ABNORMAL_1: WindowId = WindowId(81);
    pub const OLYMPIAD_ABNORMAL_2: WindowId = WindowId(82);
    pub const OLYMPIAD_TARGET: WindowId = WindowId(83);
    pub const MANOR_SEED_SETTING: WindowId = WindowId(69);
    pub const MANOR_SEED_CHANGE: WindowId = WindowId(73);
    pub const MANOR_CROP_SETTING: WindowId = WindowId(70);
    pub const MANOR_CROP_CHANGE: WindowId = WindowId(74);
    pub const MANOR_CROP_SELL: WindowId = WindowId(71);
    pub const MANOR_CROP_SELL_CHANGE: WindowId = WindowId(72);
    pub const MANOR_SHOP: WindowId = WindowId(54);
    pub const HERO_TOWER: WindowId = WindowId(76);
    pub const HERO_TOWER_CONTENT: WindowId = WindowId(77);
    pub const FISHING: WindowId = WindowId(0x10000 + 0x400a);
    pub const TRAIN: WindowId = WindowId(33);
    pub const MACRO_LIST: WindowId = WindowId(29);
    pub const MACRO_INFO: WindowId = WindowId(30);
    pub const MACRO_EDIT: WindowId = WindowId(31);
    pub const ITEM_ENCHANT: WindowId = WindowId(11);
    pub const HENNA_LIST: WindowId = WindowId(51);
    pub const HENNA_INFO: WindowId = WindowId(52);
    pub const RECIPE_BOOK: WindowId = WindowId(45);
    pub const RECIPE_SHOP: WindowId = WindowId(48);
    pub const RECIPE_SELL: WindowId = WindowId(49);
    pub const RECIPE_BUY: WindowId = WindowId(50);
    pub const RECIPE_TREE: WindowId = WindowId(46);
    pub const RECIPE_MANUFACTURE: WindowId = WindowId(47);
    pub const WAREHOUSE: WindowId = WindowId(18);
    pub const TRADE: WindowId = WindowId(24);
    pub const MENU: WindowId = WindowId(25);
    pub const DIALOG: WindowId = WindowId(27);
    pub const DIALOG_BOTTOM: WindowId = WindowId(28);
    pub const SHORTCUT: WindowId = WindowId(32);
    pub const TOWN_MAP: WindowId = WindowId(38);
    pub const XMAS_SEAL: WindowId = WindowId(63);
    pub const MAP_COVER: WindowId = WindowId(37);
    pub const OBSERVER: WindowId = WindowId(39);
    pub const CHAT: WindowId = WindowId(44);
    pub const SYSTEM_MSG: WindowId = WindowId(0x1a01);
    pub const RADAR: WindowId = WindowId(0x10000 + 1);
    pub const ZONE_TITLE: WindowId = WindowId(0x10000 + 2);
    pub const RESTART_MENU: WindowId = WindowId(0x10000 + 3);
    pub const LOADING: WindowId = WindowId(0x10000 + 4);
    pub const OPTION: WindowId = WindowId(0x10000 + 5);
    pub const TUTORIAL_VIEWER: WindowId = WindowId(0x10000 + 6);
    pub const TUTORIAL_BUTTON: WindowId = WindowId(0x10000 + 7);
    pub const PETITION: WindowId = WindowId(0x10000 + 8);
    pub const PETITION_FEEDBACK: WindowId = WindowId(0x10000 + 9);
    pub const MESSENGER: WindowId = WindowId(53);
    pub const BROADCAST_OBSERVER: WindowId = WindowId(66);
    pub const BROADCAST_GM_MSG: WindowId = WindowId(0x10000 + 10);
    pub const BROADCAST_SPECIAL_MSG: WindowId = WindowId(0x10000 + 11);
    pub const BROADCAST_TOP: WindowId = WindowId(0x10000 + 12);
    pub const BROADCAST_PLAYER_0: WindowId = WindowId(0x10000 + 13);
    pub const BROADCAST_PLAYER_1: WindowId = WindowId(0x10000 + 14);
    pub const BROADCAST_PARTY_0: WindowId = WindowId(0x10000 + 15);
    pub const BROADCAST_PARTY_1: WindowId = WindowId(0x10000 + 16);
    pub const BROADCAST_MSG: WindowId = WindowId(0x10000 + 17);
    pub const QUEST_BUTTON: WindowId = WindowId(0x10000 + 18);
    pub const MAIL_BUTTON: WindowId = WindowId(0x10000 + 19);
    pub const WARNING_DIALOG: WindowId = WindowId(0x10000 + 20);
    pub const SYSTEM_MENU: WindowId = WindowId(0xf0f011);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h
    }

    pub fn pt_in_rect(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x <= self.x + self.w && y <= self.y + self.h
    }

    pub fn offset(&self, dx: i32, dy: i32) -> Rect {
        Rect { x: self.x + dx, y: self.y + dy, w: self.w, h: self.h }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Window {
    instance: crate::window_instance::WindowInstance,
    pub id: WindowId,
    pub rect: Rect,
    pub grip: Rect,
    pub visible: bool,
    pub initial_position: Option<crate::initial_position::InitialPosition>,
    pub owner: Option<crate::window_owner::WindowOwner>,
    pub registration: Option<crate::window_owner::WindowRegistration>,
    rearrange_scale: (u32, u32),
}

pub const STYLE_VISIBLE: u32 = 0x2;
pub const STYLE_CLICK_THROUGH: u32 = 0x4;
pub const STYLE_NOHITTEST: u32 = 0x8;
pub const STYLE_NOFOCUSNEXT: u32 = 0x80000;
pub const STYLE_FIRSTCHILD: u32 = 0x800000;

impl Window {
    pub fn style(&self) -> u32 {
        self.registration.map_or(STYLE_VISIBLE, |registration| registration.flags_argument)
    }

    pub fn new(id: WindowId, rect: Rect, grip: Rect) -> Window {
        Window { instance: crate::window_instance::WindowInstance::default(), id, rect, grip, visible: true, initial_position: None, owner: None, registration: None, rearrange_scale: (0, 0) }
    }

    pub fn instance(&self) -> crate::window_instance::WindowInstance { self.instance.clone() }

    pub fn with_initial_position(mut self, position: crate::initial_position::InitialPosition) -> Self {
        self.initial_position = Some(position);
        self
    }

    pub fn grip_rect(&self) -> Rect {
        self.grip.offset(self.rect.x, self.rect.y)
    }
}

#[derive(Debug, Default)]
pub struct Desktop {
    pub windows: Vec<Window>,
    drag: Option<(crate::window_instance::WindowInstance, i32, i32)>,
    pub size: (i32, i32),
    shown: Option<Vec<WindowId>>,
    focused: Option<crate::window_instance::WindowInstance>,
}

impl Desktop {
    pub fn new(size: (i32, i32)) -> Desktop {
        Desktop { windows: Vec::new(), drag: None, size, shown: None, focused: None }
    }

    pub fn window(&self, id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    pub fn window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == id)
    }

    pub fn instance(&self, instance: &crate::window_instance::WindowInstance) -> Option<&Window> {
        self.windows.iter().find(|window| &window.instance == instance)
    }

    pub fn instance_mut(&mut self, instance: &crate::window_instance::WindowInstance) -> Option<&mut Window> {
        self.windows.iter_mut().find(|window| &window.instance == instance)
    }

    pub fn window_at(&self, x: i32, y: i32) -> Option<WindowId> {
        self.windows.iter().rev().find(|w| w.visible && w.style() & STYLE_CLICK_THROUGH == 0 && w.rect.pt_in_rect(x, y)).map(|w| w.id)
    }

    pub fn raise_newly_shown(&mut self) {
        let now: Vec<WindowId> = self.windows.iter().filter(|window| window.visible).map(|window| window.id).collect();
        if let Some(before) = self.shown.take() {
            for id in now.iter().filter(|id| !before.contains(id)) {
                self.bring_to_front(*id);
                if self.window(*id).is_some_and(|window| window.style() & STYLE_NOFOCUSNEXT == 0) {
                    self.focused = self.window(*id).map(Window::instance);
                }
            }
        }
        self.shown = Some(now);
        self.settle_focus();
    }

    pub fn sort_by_creation_order(&mut self) {
        self.windows.sort_by_key(|window| std::cmp::Reverse(window.registration.map_or(u32::MAX, |registration| registration.call_address)));
    }

    pub fn focused(&self) -> Option<WindowId> {
        self.focused.as_ref().and_then(|instance| self.instance(instance)).map(|window| window.id)
    }

    pub fn clear_focus(&mut self) {
        self.focused = None;
    }

    pub fn focus_window(&mut self, id: WindowId) {
        let Some(index) = self.windows.iter().rposition(|window| window.id == id) else { return };
        let instance = self.windows[index].instance();
        if self.focused.as_ref() == Some(&instance) {
            return;
        }
        self.focused = Some(instance);
        let window = self.windows.remove(index);
        let mut at = self.windows.len();
        while at > 0 && self.windows[at - 1].style() & STYLE_FIRSTCHILD != 0 {
            at -= 1;
        }
        self.windows.insert(at, window);
    }

    pub fn settle_focus(&mut self) {
        let Some(instance) = self.focused.clone() else { return };
        if self.instance(&instance).is_some_and(|window| window.visible) {
            return;
        }
        self.focused = None;
        let next = self.windows.iter().rev().skip(1).find(|window| window.visible && window.style() & STYLE_NOFOCUSNEXT == 0).map(|window| window.id);
        if let Some(id) = next {
            self.focus_window(id);
        }
    }

    pub fn bring_to_front(&mut self, id: WindowId) {
        if let Some(index) = self.windows.iter().position(|window| window.id == id) {
            let window = self.windows.remove(index);
            self.windows.push(window);
        }
    }

    pub fn dragging(&self) -> Option<WindowId> {
        self.drag.as_ref().and_then(|(instance, _, _)| self.instance(instance)).map(|window| window.id)
    }

    pub fn begin_drag(&mut self, x: i32, y: i32) -> bool {
        let Some(window) = self.windows.iter().rev().find(|window| window.visible && window.style() & STYLE_CLICK_THROUGH == 0 && window.rect.pt_in_rect(x, y)) else { return false };
        if !window.grip_rect().pt_in_rect(x, y) {
            return false;
        }
        self.drag = Some((window.instance(), x - window.rect.x, y - window.rect.y));
        true
    }

    pub fn drag(&mut self, x: i32, y: i32) {
        let Some((instance, dx, dy)) = self.drag.clone() else { return };
        if let Some(window) = self.instance_mut(&instance) {
            window.rect.x = x - dx;
            window.rect.y = y - dy;
        }
    }

    pub fn end_drag(&mut self) {
        self.drag = None;
    }

    pub fn rearrange(&mut self, new: (i32, i32)) {
        let old = self.size;
        for window in &mut self.windows {
            let (dx, scale_x) = rearranged(window.rect.x, window.rect.w, old.0, new.0, f32::from_bits(window.rearrange_scale.0));
            let (dy, scale_y) = rearranged(window.rect.y, window.rect.h, old.1, new.1, f32::from_bits(window.rearrange_scale.1));
            window.rearrange_scale = (scale_x.to_bits(), scale_y.to_bits());
            window.rect.x = window.rect.x.wrapping_add(dx);
            window.rect.y = window.rect.y.wrapping_add(dy);
        }
        self.size = new;
    }

    pub fn positions(&self) -> Vec<(WindowId, i32, i32)> {
        self.windows.iter().map(|w| (w.id, w.rect.x, w.rect.y)).collect()
    }
}

fn rearranged(pos: i32, extent: i32, old: i32, new: i32, mut scale: f32) -> (i32, f32) {
    let extent_f = extent as f32;
    if f64::from(extent_f) + 200.0 < f64::from(old as f32) {
        scale = (f64::from(pos) / (f64::from(old as f32) - f64::from(extent_f))) as f32;
    }
    let mut delta = pos.wrapping_neg() as f32;
    if extent_f < new as f32 {
        delta = ((f64::from(new as f32) - f64::from(extent_f)) * f64::from(scale) - f64::from(pos)) as f32;
    }
    (crate::window_restore::float_size_to_int(delta), scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn desktop() -> Desktop {
        let mut desktop = Desktop::new((1000, 800));
        desktop.windows.push(Window::new(WindowId::STATUS, Rect::new(400, 100, 200, 84), Rect::new(0, 0, 12, 84)));
        desktop.windows.push(Window::new(WindowId::CHAT, Rect::new(0, 613, 348, 187), Rect::new(0, 0, 348, 18)));
        desktop
    }

    #[test]
    fn window_at_prefers_the_last_drawn_and_skips_hidden() {
        let mut desktop = desktop();
        desktop.windows.push(Window::new(WindowId::TARGET, Rect::new(450, 120, 100, 50), Rect::new(0, 0, 12, 50)));
        assert_eq!(desktop.window_at(460, 130), Some(WindowId::TARGET));
        assert_eq!(desktop.window_at(405, 105), Some(WindowId::STATUS));
        assert_eq!(desktop.window_at(10, 10), None);
        desktop.window_mut(WindowId::TARGET).unwrap().visible = false;
        assert_eq!(desktop.window_at(460, 130), Some(WindowId::STATUS));
    }

    #[test]
    fn window_hit_tests_include_the_far_edges_like_ncwnd_ptinrect() {
        let mut desktop = desktop();
        assert_eq!(desktop.window_at(600, 184), Some(WindowId::STATUS));
        assert_eq!(desktop.window_at(601, 184), None);
        assert_eq!(desktop.window_at(600, 185), None);
        assert!(Rect::new(0, 0, 10, 10).pt_in_rect(10, 10));
        assert!(!Rect::new(0, 0, 10, 10).contains(10, 10));
        assert!(desktop.begin_drag(412, 150));
        desktop.end_drag();
        assert!(!desktop.begin_drag(413, 150));
    }

    #[test]
    fn a_window_that_becomes_visible_goes_to_the_head_of_the_list() {
        let mut desktop = desktop();
        desktop.windows.push(Window::new(WindowId::NPC_HTML, Rect::new(400, 100, 200, 84), Rect::new(0, 0, 12, 84)));
        desktop.window_mut(WindowId::NPC_HTML).unwrap().visible = false;
        desktop.raise_newly_shown();
        desktop.window_mut(WindowId::NPC_HTML).unwrap().visible = true;
        desktop.bring_to_front(WindowId::CHAT);
        desktop.raise_newly_shown();
        assert_eq!(desktop.windows.last().map(|window| window.id), Some(WindowId::NPC_HTML));
        desktop.bring_to_front(WindowId::CHAT);
        desktop.raise_newly_shown();
        assert_eq!(desktop.windows.last().map(|window| window.id), Some(WindowId::CHAT));
    }

    fn styled(id: WindowId, x: i32, style: u32) -> Window {
        let mut window = Window::new(id, Rect::new(x, 0, 100, 100), Rect::new(0, 0, 100, 10));
        window.registration = Some(crate::window_owner::WindowRegistration { call_address: 0x10060000 + id.0, flags_argument: style, vtable: 0 });
        window
    }

    #[test]
    fn creation_order_puts_the_first_created_window_in_front_like_the_head_of_the_child_list() {
        let mut desktop = Desktop::new((1000, 800));
        desktop.windows.push(styled(WindowId(3), 0, 0x1002));
        desktop.windows.push(styled(WindowId(1), 0, 0x1002));
        desktop.windows.push(Window::new(WindowId(99), Rect::new(0, 0, 10, 10), Rect::new(0, 0, 1, 1)));
        desktop.windows.push(styled(WindowId(2), 0, 0x1002));
        desktop.sort_by_creation_order();
        assert_eq!(desktop.windows.iter().map(|w| w.id.0).collect::<Vec<_>>(), [99, 3, 2, 1]);
    }

    #[test]
    fn a_press_focuses_and_raises_the_window_and_the_focused_one_is_not_raised_twice() {
        let mut desktop = Desktop::new((1000, 800));
        desktop.windows.push(styled(WindowId(1), 0, 0x1002));
        desktop.windows.push(styled(WindowId(2), 0, 0x1002));
        desktop.windows.push(styled(WindowId(3), 0, 0x1002));
        desktop.focus_window(WindowId(1));
        assert_eq!(desktop.windows.last().map(|w| w.id.0), Some(1));
        assert_eq!(desktop.focused(), Some(WindowId(1)));
        desktop.bring_to_front(WindowId(3));
        desktop.focus_window(WindowId(1));
        assert_eq!(desktop.windows.last().map(|w| w.id.0), Some(3));
        desktop.clear_focus();
        assert_eq!(desktop.focused(), None);
    }

    #[test]
    fn hiding_the_focused_window_passes_the_focus_to_the_next_visible_one_that_is_not_nofocusnext() {
        let mut desktop = Desktop::new((1000, 800));
        desktop.windows.push(styled(WindowId(1), 0, 0x1002));
        desktop.windows.push(styled(WindowId(2), 0, 0x85002));
        desktop.windows.push(styled(WindowId(3), 0, 0x1002));
        desktop.windows.push(styled(WindowId(4), 0, 0x1002));
        desktop.focus_window(WindowId(4));
        desktop.window_mut(WindowId(4)).unwrap().visible = false;
        desktop.settle_focus();
        assert_eq!(desktop.focused(), Some(WindowId(3)));
        assert_eq!(desktop.windows.last().map(|w| w.id.0), Some(3));
        desktop.window_mut(WindowId(3)).unwrap().visible = false;
        desktop.settle_focus();
        assert_eq!(desktop.focused(), Some(WindowId(1)));
    }

    #[test]
    fn a_click_through_window_never_takes_the_press() {
        let mut desktop = Desktop::new((1000, 800));
        desktop.windows.push(styled(WindowId(1), 0, 0x1002));
        desktop.windows.push(styled(WindowId(2), 0, 0x6));
        assert_eq!(desktop.window_at(5, 5), Some(WindowId(1)));
        assert!(desktop.begin_drag(5, 5));
        assert_eq!(desktop.dragging(), Some(WindowId(1)));
    }

    #[test]
    fn begin_drag_outside_the_grip_is_refused() {
        let mut desktop = desktop();
        assert!(!desktop.begin_drag(500, 150));
        assert!(!desktop.begin_drag(10, 10));
        assert_eq!(desktop.dragging(), None);
        assert!(desktop.begin_drag(405, 150));
        assert_eq!(desktop.dragging(), Some(WindowId::STATUS));
    }

    #[test]
    fn dragging_moves_the_rect_by_the_mouse_delta() {
        let mut desktop = desktop();
        assert!(desktop.begin_drag(405, 150));
        desktop.drag(425, 160);
        assert_eq!(desktop.window(WindowId::STATUS).unwrap().rect, Rect::new(420, 110, 200, 84));
        assert_eq!(desktop.window(WindowId::STATUS).unwrap().grip_rect(), Rect::new(420, 110, 12, 84));
        desktop.end_drag();
        desktop.drag(900, 900);
        assert_eq!(desktop.window(WindowId::STATUS).unwrap().rect, Rect::new(420, 110, 200, 84));
        assert_eq!(desktop.positions()[0], (WindowId::STATUS, 420, 110));
    }

    #[test]
    fn dragging_keeps_the_hit_instance_with_duplicate_ids_and_reordered_painting() {
        let mut desktop = Desktop::new((800, 600));
        let grip = Rect::new(0, 0, 100, 10);
        desktop.windows.push(Window::new(WindowId(12), Rect::new(0, 20, 100, 80), grip));
        desktop.windows.push(Window::new(WindowId(12), Rect::new(200, 20, 100, 80), grip));
        let first = desktop.windows[0].instance();
        let second = desktop.windows[1].instance();
        assert_ne!(first, second);
        assert!(desktop.begin_drag(205, 25));
        desktop.windows.swap(0, 1);
        desktop.drag(305, 45);
        assert_eq!(desktop.instance(&second).unwrap().rect, Rect::new(300, 40, 100, 80));
        assert_eq!(desktop.instance(&first).unwrap().rect, Rect::new(0, 20, 100, 80));
        assert_eq!(desktop.dragging(), Some(WindowId(12)));
    }

    #[test]
    fn removed_drag_instance_does_not_retarget_a_replacement_with_the_same_id() {
        let mut desktop = desktop();
        let old = desktop.windows[0].instance();
        assert!(desktop.begin_drag(405, 150));
        desktop.windows.remove(0);
        desktop.windows.push(Window::new(WindowId::STATUS, Rect::new(20, 30, 200, 84), Rect::new(0, 0, 12, 84)));
        desktop.drag(500, 500);
        assert_eq!(desktop.instance(&old), None);
        assert_eq!(desktop.dragging(), None);
        assert_eq!(desktop.window(WindowId::STATUS).unwrap().rect, Rect::new(20, 30, 200, 84));
    }

    #[test]
    fn concrete_registry_resolves_the_same_live_window_after_reordering() {
        let mut desktop = desktop();
        let instance = desktop.windows[0].instance();
        let mut manager = crate::window_manager::WindowManager::default();
        let record = manager.register(instance.clone(), 0x1002, None, |_, _, _| Ok::<_, std::convert::Infallible>(()), |_, _| Ok(0x1002)).unwrap().unwrap();
        desktop.windows.swap(0, 1);
        let stored = manager.records.get(record).unwrap().window.as_ref().unwrap();
        assert_eq!(stored, &instance);
        desktop.instance_mut(stored).unwrap().rect.x = 17;
        assert_eq!(desktop.window(WindowId::STATUS).unwrap().rect.x, 17);
        assert_eq!(desktop.window(WindowId::CHAT).unwrap().rect.x, 0);
    }

    #[test]
    fn rearrange_keeps_the_position_proportional_to_the_free_space() {
        let mut desktop = desktop();
        desktop.rearrange((2000, 800));
        assert_eq!(desktop.window(WindowId::STATUS).unwrap().rect.x, 900);
        assert_eq!(desktop.window(WindowId::STATUS).unwrap().rect.y, 100);
        assert_eq!(desktop.window(WindowId::CHAT).unwrap().rect.y, 613);
        assert_eq!(desktop.size, (2000, 800));
        desktop.rearrange((2000, 1000));
        assert_eq!(desktop.window(WindowId::CHAT).unwrap().rect.y, 813);
    }

    #[test]
    fn rearrange_remembers_the_scale_when_the_window_is_too_wide_to_measure_it_and_restores_the_place_when_the_screen_grows_back() {
        let mut desktop = Desktop::new((1000, 800));
        desktop.windows.push(Window::new(WindowId::STATUS, Rect::new(400, 100, 600, 84), Rect::new(0, 0, 12, 84)));
        desktop.rearrange((500, 800));
        assert_eq!(desktop.window(WindowId::STATUS).unwrap().rect.x, 0);
        desktop.rearrange((1000, 800));
        assert_eq!(desktop.window(WindowId::STATUS).unwrap().rect.x, 400);
        assert_eq!(desktop.window(WindowId::STATUS).unwrap().rect.y, 100);
    }

    #[test]
    fn rearrange_clamps_at_zero_and_handles_a_window_as_wide_as_the_screen() {
        let mut desktop = Desktop::new((200, 100));
        desktop.windows.push(Window::new(WindowId::CHAT, Rect::new(50, 40, 200, 20), Rect::new(0, 0, 200, 20)));
        desktop.windows.push(Window::new(WindowId::SHORTCUT, Rect::new(-30, 40, 20, 20), Rect::new(0, 0, 20, 20)));
        desktop.rearrange((400, 100));
        assert_eq!(desktop.window(WindowId::CHAT).unwrap().rect.x, 0);
        assert_eq!(desktop.window(WindowId::SHORTCUT).unwrap().rect.x, 0);
        assert_eq!(desktop.window(WindowId::SHORTCUT).unwrap().rect.y, 0);
    }
}
