use crate::shortcut::{PAGES, SLOTS};
use crate::window::Rect;

pub const WIDTH: i32 = 556;
pub const ROW: i32 = 52;
pub const TOP: i32 = 14;
pub const AUTO: i32 = 28;
pub const AUTO_ICON: i32 = 22;
pub const AUTO_GAP: i32 = 4;
pub const AUTO_LEVEL: i32 = 60;
pub const AUTO_CHIP: i32 = 94;
pub const AUTO_MARGIN: i32 = 8;
pub const AUTO_STEP: i32 = 1;
pub const AUTO_MIN: i32 = 1;
pub const AUTO_MAX: i32 = 99;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModernShortcuts {
    pub count: usize,
    pub pages: [i32; 4],
    pub locked: bool,
}

impl Default for ModernShortcuts {
    fn default() -> Self { Self { count: 3, pages: [1, 2, 3, 4], locked: false } }
}

const SOUL_BEST_FIRST: [i32; 6] = [1467, 1466, 1465, 1464, 1463, 1835];
const SPIRIT_BEST_FIRST: [i32; 12] = [3952, 2514, 3951, 2513, 3950, 2512, 3949, 2511, 3948, 2510, 3947, 2509];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShotKind {
    Soul,
    Spirit,
}

impl ShotKind {
    pub const ALL: [ShotKind; 2] = [ShotKind::Soul, ShotKind::Spirit];

    pub fn index(self) -> usize {
        match self {
            ShotKind::Soul => 0,
            ShotKind::Spirit => 1,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ShotKind::Soul => "Auto Soulshot",
            ShotKind::Spirit => "Auto Spiritshot",
        }
    }

    pub fn best_first(self) -> &'static [i32] {
        match self {
            ShotKind::Soul => &SOUL_BEST_FIRST,
            ShotKind::Spirit => &SPIRIT_BEST_FIRST,
        }
    }

    pub fn accepts(self, item_id: i32) -> bool {
        self.best_first().contains(&item_id) || self == ShotKind::Soul && crate::shortcut::is_soulshot_item(item_id) && !ShotKind::Spirit.best_first().contains(&item_id)
    }

    pub fn best(self, held: impl Fn(i32) -> bool) -> i32 {
        self.best_first().iter().copied().find(|&id| held(id)).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vital {
    Hp,
    Cp,
    Mp,
}

impl Vital {
    pub const ALL: [Vital; 3] = [Vital::Hp, Vital::Cp, Vital::Mp];

    pub fn index(self) -> usize {
        match self {
            Vital::Hp => 0,
            Vital::Cp => 1,
            Vital::Mp => 2,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Vital::Hp => "HP",
            Vital::Cp => "CP",
            Vital::Mp => "MP",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoHit {
    Potion(Vital),
    Level(Vital),
    Shot(ShotKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModernBarButton {
    Expand,
    Collapse,
    Lock,
    Next(usize),
    Prev(usize),
}

impl ModernShortcuts {
    pub fn normalized(mut self) -> Self {
        self.count = self.count.clamp(1, 5);
        for page in &mut self.pages { *page = (*page).clamp(0, PAGES - 1); }
        self
    }

    pub fn height(&self) -> i32 { TOP + AUTO + self.count.clamp(1, 5) as i32 * ROW }

    pub fn row_y(&self, row: usize) -> i32 { TOP + AUTO + (self.count.clamp(1, 5) - 1 - row) as i32 * ROW }

    pub fn page(&self, row: usize, primary: i32) -> i32 {
        if row == 0 { primary } else { self.pages.get(row - 1).copied().unwrap_or(0) }
    }

    pub fn slot_rect(&self, row: usize, column: usize) -> Rect {
        Rect::new(34 + column as i32 * 40, self.row_y(row) + 7, 34, 34)
    }

    pub fn slot_at(&self, x: i32, y: i32, primary: i32) -> Option<usize> {
        (0..self.count.clamp(1, 5)).find_map(|row| {
            (0..SLOTS).find(|&col| self.slot_rect(row, col).contains(x, y))
                .map(|col| self.page(row, primary) as usize * SLOTS + col)
        })
    }

    pub fn button_rect(&self, button: ModernBarButton) -> Rect {
        match button {
            ModernBarButton::Expand => Rect::new(526, self.height() - 52, 22, 16),
            ModernBarButton::Lock => Rect::new(526, self.height() - 36, 22, 16),
            ModernBarButton::Collapse => Rect::new(526, self.height() - 20, 22, 16),
            ModernBarButton::Prev(row) => Rect::new(8, self.row_y(row) + 1, 18, 14),
            ModernBarButton::Next(row) => Rect::new(8, self.row_y(row) + 31, 18, 14),
        }
    }

    pub fn auto_rect(&self, hit: AutoHit) -> Rect {
        let y = (AUTO - AUTO_ICON) / 2 - 1;
        let chip = |vital: Vital| AUTO_MARGIN + vital.index() as i32 * AUTO_CHIP;
        match hit {
            AutoHit::Potion(vital) => Rect::new(chip(vital), y, AUTO_ICON, AUTO_ICON),
            AutoHit::Level(vital) => Rect::new(chip(vital) + AUTO_ICON + 2, y, AUTO_LEVEL, AUTO_ICON),
            AutoHit::Shot(kind) => {
                let step = AUTO_ICON + AUTO_GAP;
                Rect::new(WIDTH - AUTO_MARGIN - AUTO_ICON - (1 - kind.index() as i32) * step, y, AUTO_ICON, AUTO_ICON)
            }
        }
    }

    pub fn auto_hits() -> impl Iterator<Item = AutoHit> {
        Vital::ALL.into_iter().flat_map(|v| [AutoHit::Potion(v), AutoHit::Level(v)]).chain(ShotKind::ALL.into_iter().map(AutoHit::Shot))
    }

    pub fn auto_percent(&self, vital: Vital, x: i32) -> i32 {
        let area = self.auto_rect(AutoHit::Level(vital));
        let span = (area.w - 1).max(1);
        (((x - area.x) * 100 + span / 2) / span).clamp(AUTO_MIN, AUTO_MAX)
    }

    pub fn auto_at(&self, x: i32, y: i32) -> Option<AutoHit> {
        Self::auto_hits().find(|&hit| self.auto_rect(hit).contains(x, y))
    }

    pub fn button_at(&self, x: i32, y: i32) -> Option<ModernBarButton> {
        use ModernBarButton::*;
        [Expand, Lock, Collapse].into_iter()
            .chain((0..self.count.clamp(1,5)).flat_map(|row| [Prev(row), Next(row)]))
            .find(|&button| self.button_rect(button).contains(x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{ShortcutKeys, VK_F1, VK_F12};
    use crate::shortcut::{BarEvent, DropTarget, PageButton, ShortcutSlot, ShortcutView, SlotKind};

    #[test]
    fn five_rows_address_existing_pages_and_only_primary_receives_function_keys() {
        let mut bar=ShortcutView::new(false);
        bar.modern=Some(ModernShortcuts { count:5,pages:[9,5,3,1],locked:false });
        bar.set_page(2);
        for index in 0..120 { bar.set(index,ShortcutSlot::new(SlotKind::Item { object_id:index as i32+100 },"icon")); }
        bar.take_events();
        for row in 0..5 {
            for column in 0..SLOTS {
                let modern=bar.modern.unwrap();
                let area=modern.slot_rect(row,column);
                let expected=modern.page(row,bar.page) as usize*SLOTS+column;
                assert_eq!(bar.slot_at(area.x+1,area.y+1),Some(expected));
                bar.mouse_down(area.x+1,area.y+1);
                bar.mouse_up(DropTarget::Bar(area.x+1,area.y+1));
                assert_eq!(bar.take_events(),vec![BarEvent::Use(expected)]);
            }
        }
        bar.key_down(VK_F1);bar.key_down(VK_F12);
        assert_eq!(bar.take_events(),vec![BarEvent::Use(24),BarEvent::Use(35)]);
        bar.press_page_button(PageButton::Modern(ModernBarButton::Next(1)));
        assert_eq!(bar.page,2);
        assert_eq!(bar.modern.unwrap().pages,[0,5,3,1]);
    }

    #[test]
    fn lock_prevents_editing_but_keeps_click_use_and_row_controls() {
        let mut bar=ShortcutView::new(false);
        bar.modern=Some(ModernShortcuts::default());
        bar.set(0,ShortcutSlot::new(SlotKind::Item { object_id:10 },"icon"));
        bar.press_page_button(PageButton::Modern(ModernBarButton::Lock));
        let area=bar.slot_rect(0);
        bar.mouse_down(area.x+1,area.y+1);bar.mouse_up(DropTarget::Outside(-50,-50));
        assert!(bar.take_events().is_empty());
        bar.mouse_down(area.x+1,area.y+1);bar.mouse_up(DropTarget::Bar(area.x+1,area.y+1));
        assert_eq!(bar.take_events(),vec![BarEvent::Use(0)]);
        for _ in 0..8 { bar.press_page_button(PageButton::Modern(ModernBarButton::Expand)); }
        assert_eq!(bar.modern.unwrap().count,5);
        for _ in 0..8 { bar.press_page_button(PageButton::Modern(ModernBarButton::Collapse)); }
        let modern=bar.modern.unwrap();
        assert_eq!(modern.count,1);
        for button in [ModernBarButton::Expand,ModernBarButton::Collapse,ModernBarButton::Lock] {
            let area=modern.button_rect(button);
            assert!(area.y>=0 && area.y+area.h<=modern.height());
            assert_eq!(modern.button_at(area.x+1,area.y+1),Some(button));
        }
    }

    #[test]
    fn auto_chips_float_above_the_panel_and_never_overlap_slots_buttons_or_each_other() {
        let hits: Vec<AutoHit> = ModernShortcuts::auto_hits().collect();
        assert_eq!(hits.len(), 8);
        for count in 1..=5 {
            let modern=ModernShortcuts { count,..Default::default() };
            for &hit in &hits {
                let area=modern.auto_rect(hit);
                assert!(area.y>=0 && area.y+area.h<=AUTO && area.x>=0 && area.x+area.w<=WIDTH);
                assert_eq!(modern.auto_at(area.x+1,area.y+1),Some(hit));
                assert_eq!(modern.slot_at(area.x+1,area.y+1,0),None);
                assert_eq!(modern.button_at(area.x+1,area.y+1),None);
            }
            assert_eq!(modern.row_y(count-1),AUTO+TOP);
            for pair in hits.windows(2) {
                let (a,b)=(modern.auto_rect(pair[0]),modern.auto_rect(pair[1]));
                assert!(a.x+a.w<=b.x);
            }
        }
    }

    #[test]
    fn clicking_the_level_bar_sets_any_threshold_from_one_to_ninety_nine() {
        let mut bar=ShortcutView::new(false);
        bar.modern=Some(ModernShortcuts::default());
        let modern=bar.modern.unwrap();
        for vital in Vital::ALL {
            let area=modern.auto_rect(AutoHit::Level(vital));
            bar.mouse_down(area.x,area.y+1);bar.mouse_up(DropTarget::Bar(area.x,area.y+1));
            assert_eq!(bar.auto.potions[vital.index()].percent,AUTO_MIN);
            bar.mouse_down(area.x+area.w-1,area.y+1);bar.mouse_up(DropTarget::Bar(area.x+area.w-1,area.y+1));
            assert_eq!(bar.auto.potions[vital.index()].percent,AUTO_MAX);
            bar.mouse_down(area.x+area.w/2,area.y+1);bar.mouse_up(DropTarget::Bar(area.x+area.w/2,area.y+1));
            assert_eq!(bar.auto.potions[vital.index()].percent,51);
        }
        let hp=modern.auto_rect(AutoHit::Level(Vital::Hp));
        bar.mouse_down(hp.x+hp.w-1,hp.y+1);
        assert_eq!(bar.auto.potions[0].percent,99);
        bar.mouse_down(hp.x+5,hp.y+1);
        assert_eq!(bar.auto.potions[0].percent,8);
    }

    #[test]
    fn each_vital_has_its_own_potion_item_limit_and_switch() {
        let mut bar=ShortcutView::new(false);
        bar.modern=Some(ModernShortcuts::default());
        let modern=bar.modern.unwrap();
        let click=|bar:&mut ShortcutView,hit:AutoHit| {
            let area=modern.auto_rect(hit);
            bar.mouse_down(area.x+1,area.y+1);
            bar.mouse_up(DropTarget::Bar(area.x+1,area.y+1));
        };
        assert_eq!(bar.auto.potions.clone().map(|p|p.on),[true,true,true]);
        click(&mut bar,AutoHit::Potion(Vital::Cp));
        assert_eq!(bar.auto.potions.clone().map(|p|p.on),[true,false,true]);
        assert!(bar.assign_auto(AutoHit::Potion(Vital::Cp),5592));
        assert!(!bar.auto.potions[1].on);
        let mp=modern.auto_rect(AutoHit::Potion(Vital::Mp));
        assert!(bar.auto_wheel(mp.x+1,mp.y+1,-2));
        assert_eq!(bar.auto.potions.clone().map(|p|p.percent),[70,70,48]);
        let hp=modern.auto_rect(AutoHit::Level(Vital::Hp));
        for _ in 0..120 { bar.auto_wheel(hp.x+1,hp.y+1,1); }
        assert_eq!(bar.auto.potions[0].percent,AUTO_MAX);
        for _ in 0..120 { bar.auto_wheel(hp.x+1,hp.y+1,-1); }
        assert_eq!(bar.auto.potions[0].percent,AUTO_MIN);
        assert!(!bar.auto_wheel(modern.auto_rect(AutoHit::Shot(ShotKind::Soul)).x+1,mp.y+1,1));
        bar.press_page_button(PageButton::Modern(ModernBarButton::Lock));
        assert!(!bar.assign_auto(AutoHit::Potion(Vital::Hp),1061));
        click(&mut bar,AutoHit::Potion(Vital::Cp));
        assert!(bar.auto.potions[1].on);
    }

    #[test]
    fn dragging_the_level_bar_follows_the_pointer_even_outside_it() {
        let mut bar=ShortcutView::new(false);
        bar.modern=Some(ModernShortcuts::default());
        let modern=bar.modern.unwrap();
        let area=modern.auto_rect(AutoHit::Level(Vital::Mp));
        bar.auto_drag(area.x+10);
        assert_eq!(bar.auto.potions[2].percent,50);
        bar.mouse_down(area.x+5,area.y+1);
        assert_eq!(bar.auto_dragging(),Some(Vital::Mp));
        bar.auto_drag(area.x+30);
        assert_eq!(bar.auto.potions[2].percent,51);
        bar.auto_drag(area.x+500);
        assert_eq!(bar.auto.potions[2].percent,AUTO_MAX);
        bar.auto_drag(area.x-500);
        assert_eq!(bar.auto.potions[2].percent,AUTO_MIN);
        bar.mouse_up(DropTarget::Outside(0,0));
        assert_eq!(bar.auto_dragging(),None);
        bar.auto_drag(area.x+30);
        assert_eq!(bar.auto.potions[2].percent,AUTO_MIN);
    }

    #[test]
    fn a_potion_slot_only_holds_the_item_the_player_dragged_and_right_click_empties_it() {
        let mut bar=ShortcutView::new(false);
        bar.modern=Some(ModernShortcuts::default());
        let area=bar.modern.unwrap().auto_rect(AutoHit::Potion(Vital::Hp));
        assert_eq!(bar.auto.potions[0].item_id,0);
        assert!(bar.assign_auto(AutoHit::Potion(Vital::Hp),1061));
        assert_eq!(bar.auto.potions[0].item_id,1061);
        bar.right_click(area.x+1,area.y+1);
        assert_eq!(bar.auto.potions[0].item_id,0);
        assert!(bar.assign_auto(AutoHit::Potion(Vital::Hp),1061));
        bar.press_page_button(PageButton::Modern(ModernBarButton::Lock));
        bar.right_click(area.x+1,area.y+1);
        assert_eq!(bar.auto.potions[0].item_id,1061);
    }

    #[test]
    fn each_shot_kind_accepts_only_its_own_items_and_toggles_through_the_server() {
        let mut bar=ShortcutView::new(false);
        bar.modern=Some(ModernShortcuts::default());
        let soul=AutoHit::Shot(ShotKind::Soul);
        let spirit=AutoHit::Shot(ShotKind::Spirit);
        let area=bar.modern.unwrap().auto_rect(soul);
        assert!(!bar.assign_auto(soul,1060));
        assert!(!bar.assign_auto(soul,2509));
        assert!(!bar.assign_auto(spirit,1463));
        assert!(bar.assign_auto(spirit,3947));
        assert!(bar.assign_auto(soul,1463));
        bar.mouse_down(area.x+1,area.y+1);bar.mouse_up(DropTarget::Bar(area.x+1,area.y+1));
        assert_eq!(bar.take_events(),vec![BarEvent::AutoShot { item_id:1463,enable:true }]);
        bar.auto.shots[0].on=true;
        bar.mouse_down(area.x+1,area.y+1);bar.mouse_up(DropTarget::Bar(area.x+1,area.y+1));
        assert_eq!(bar.take_events(),vec![BarEvent::AutoShot { item_id:1463,enable:false }]);
        assert!(bar.assign_auto(soul,1835));
        assert_eq!(bar.take_events(),vec![BarEvent::AutoShot { item_id:1463,enable:false }]);
        assert_eq!(bar.auto.shots[0].item_id,1835);
        assert_eq!(bar.auto.shots[1].item_id,3947);
    }

    #[test]
    fn the_best_shot_held_fills_each_slot_without_dragging() {
        assert_eq!(ShotKind::Soul.best(|id| id==1463||id==1465),1465);
        assert_eq!(ShotKind::Soul.best(|id| id==1835||id==1463),1463);
        assert_eq!(ShotKind::Soul.best(|id| id==2514),0);
        assert_eq!(ShotKind::Spirit.best(|id| id==2512||id==3949),2512);
        assert_eq!(ShotKind::Spirit.best(|id| id==2512||id==3950),3950);
        assert_eq!(ShotKind::Spirit.best(|id| id==1467),0);
    }
}
