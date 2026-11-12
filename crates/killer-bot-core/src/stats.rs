use killer_world::World;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

pub const ADENA_ITEM_ID: i32 = 57;
pub const LOG_LIMIT: usize = 500;
pub const MIN_WINDOW: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootRow {
    pub source: String,
    pub item_id: i32,
    pub count: i32,
    pub object_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Gains {
    pub exp_per_hour: i64,
    pub sp_per_hour: i64,
    pub adena_per_hour: i64,
    pub adena_total: i64,
}

#[derive(Debug, Default)]
pub struct SessionStats {
    started: Option<Duration>,
    last_exp: Option<i64>,
    last_sp: Option<i64>,
    exp: i64,
    sp: i64,
    adena: i64,
    inventory: HashMap<i32, (i32, i32)>,
    primed: bool,
    loads: u32,
    last_gain: Option<Duration>,
    loot: VecDeque<LootRow>,
    pub last_kill: String,
}

impl SessionStats {
    pub fn observe(&mut self, now: Duration, world: &World) {
        let exp = world.me.as_ref().map(|me| i64::from(me.exp));
        let sp = Some(i64::from(world.stats.sp));
        let current: HashMap<i32, (i32, i32)> = world.inventory.items.iter().map(|item| (item.object_id, (item.item_id, item.count))).collect();
        if !self.primed {
            self.started = Some(now);
            self.last_exp = exp;
            self.last_sp = sp;
            self.inventory = current;
            self.loads = world.inventory.loads;
            self.primed = true;
            return;
        }
        if let (Some(before), Some(after)) = (self.last_exp, exp)
            && after > before
        {
            self.exp += after - before;
            self.last_gain = Some(now);
        }
        if let (Some(before), Some(after)) = (self.last_sp, sp)
            && after > before
        {
            self.sp += after - before;
            self.last_gain = Some(now);
        }
        self.last_exp = exp.or(self.last_exp);
        self.last_sp = sp;
        if world.inventory.loads != self.loads {
            self.loads = world.inventory.loads;
            self.inventory = current;
            return;
        }
        for (object_id, (item_id, count)) in &current {
            let before = self.inventory.get(object_id).map_or(0, |(_, count)| *count);
            let gained = count - before;
            if gained <= 0 {
                continue;
            }
            if *item_id == ADENA_ITEM_ID {
                self.adena += i64::from(gained);
                self.last_gain = Some(now);
            }
            if self.loot.len() == LOG_LIMIT {
                self.loot.pop_front();
            }
            self.loot.push_back(LootRow { source: self.last_kill.clone(), item_id: *item_id, count: gained, object_id: *object_id });
        }
        self.inventory = current;
    }

    pub fn clear(&mut self, now: Duration) {
        *self = SessionStats { started: Some(now), last_exp: self.last_exp, last_sp: self.last_sp, inventory: std::mem::take(&mut self.inventory), primed: self.primed, loads: self.loads, last_kill: std::mem::take(&mut self.last_kill), ..Default::default() };
    }

    pub fn gains(&self, now: Duration) -> Gains {
        let started = self.started.unwrap_or(now);
        let end = self.last_gain.unwrap_or(now).min(now);
        let elapsed = end.saturating_sub(started).max(MIN_WINDOW);
        let hours = elapsed.as_secs_f64() / 3600.0;
        let per_hour = |value: i64| (value as f64 / hours).round() as i64;
        Gains { exp_per_hour: per_hour(self.exp), sp_per_hour: per_hour(self.sp), adena_per_hour: per_hour(self.adena), adena_total: self.adena }
    }

    pub fn loot(&self) -> Vec<LootRow> {
        self.loot.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{Vitals, user_info};
    use killer_world::World;

    fn world_with(items: &[(i32, i32, i32)]) -> World {
        let mut world = crate::testkit::world_of(&[user_info(1, "Wender", &Vitals::full(), [0, 0, 0])]);
        world.inventory.items = items
            .iter()
            .map(|(object_id, item_id, count)| l2_net::game::InventoryItem { type1: 0, object_id: *object_id, item_id: *item_id, count: *count, type2: 0, custom_type1: 0, equipped: false, body_part: 0, enchant: 0, custom_type2: 0 })
            .collect();
        world
    }

    #[test]
    fn loot_rows_and_adena_total_follow_inventory_increases_only() {
        let mut stats = SessionStats::default();
        stats.observe(Duration::ZERO, &world_with(&[(1, 57, 1000), (2, 1060, 5)]));
        stats.last_kill = "Wolf".into();
        stats.observe(Duration::from_secs(10), &world_with(&[(1, 57, 1300), (2, 1060, 4), (3, 735, 2)]));
        let loot = stats.loot();
        assert_eq!(loot.len(), 2);
        assert!(loot.iter().any(|row| row.item_id == 57 && row.count == 300 && row.source == "Wolf"));
        assert!(loot.iter().any(|row| row.item_id == 735 && row.count == 2));
        assert_eq!(stats.gains(Duration::from_secs(10)).adena_total, 300);
        stats.observe(Duration::from_secs(20), &world_with(&[(1, 57, 1100)]));
        assert_eq!(stats.gains(Duration::from_secs(20)).adena_total, 300);
    }

    #[test]
    fn a_full_item_list_arriving_after_the_start_is_not_counted_as_loot() {
        let mut stats = SessionStats::default();
        let mut empty = world_with(&[]);
        stats.observe(Duration::ZERO, &empty);
        empty.inventory.apply_list(&l2_net::game::ItemList {
            show_window: false,
            items: vec![l2_net::game::InventoryItem { type1: 0, object_id: 1, item_id: 57, count: 1_600_000_000, type2: 0, custom_type1: 0, equipped: false, body_part: 0, enchant: 0, custom_type2: 0 }],
        });
        stats.observe(Duration::from_secs(5), &empty);
        assert_eq!(stats.gains(Duration::from_secs(5)).adena_total, 0);
        assert!(stats.loot().is_empty());
        empty.inventory.items[0].count += 250;
        stats.observe(Duration::from_secs(10), &empty);
        assert_eq!(stats.gains(Duration::from_secs(10)).adena_total, 250);
    }

    #[test]
    fn hourly_rates_use_at_least_a_minute_and_clearing_resets_the_counters() {
        let mut stats = SessionStats::default();
        stats.observe(Duration::ZERO, &world_with(&[(1, 57, 0)]));
        stats.observe(Duration::from_secs(30), &world_with(&[(1, 57, 100)]));
        assert_eq!(stats.gains(Duration::from_secs(30)).adena_per_hour, 6000);
        assert_eq!(stats.gains(Duration::from_secs(3600)).adena_per_hour, 6000);
        stats.observe(Duration::from_secs(1800), &world_with(&[(1, 57, 100)]));
        assert_eq!(stats.gains(Duration::from_secs(3600)).adena_per_hour, 6000);
        stats.observe(Duration::from_secs(3600), &world_with(&[(1, 57, 200)]));
        assert_eq!(stats.gains(Duration::from_secs(7200)).adena_per_hour, 200);
        stats.clear(Duration::from_secs(40));
        assert_eq!(stats.gains(Duration::from_secs(40)), Gains::default());
        assert!(stats.loot().is_empty());
    }
}
