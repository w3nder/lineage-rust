use crate::names::Names;
use killer_world::{Entity, NetCommand, World};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

pub struct Env<'a> {
    pub world: &'a World,
    pub names: &'a dyn Names,
    pub now: Duration,
    pub sweepable: &'a HashSet<i32>,
    pub spoiled: &'a HashSet<i32>,
}

impl Env<'_> {
    pub fn me(&self) -> Option<&Entity> {
        self.world.me.as_ref()
    }

    pub fn origin(&self) -> Option<[i32; 3]> {
        self.me().map(|me| me.position.to_array().map(|value| value as i32))
    }

    pub fn my_mp(&self) -> i32 {
        self.me().and_then(|me| me.mp).map_or(0, |(current, _)| current)
    }

    pub fn knows(&self, skill_id: i32) -> bool {
        self.world.skills.iter().any(|skill| skill.id == skill_id && !skill.passive)
    }

    pub fn ready(&self, skill_id: i32) -> bool {
        self.knows(skill_id) && !self.world.cooldowns.iter().any(|reuse| reuse.skill_id == skill_id && reuse.remaining > 0)
    }

    pub fn item_object(&self, item_id: i32) -> Option<i32> {
        self.world.inventory.items.iter().find(|item| item.item_id == item_id && item.count > 0).map(|item| item.object_id)
    }

    pub fn select(&self, object_id: i32) -> Option<NetCommand> {
        Some(NetCommand::Action { object_id, origin: self.origin()?, shift: false })
    }

    pub fn skill_by_name(&self, name: &str) -> Option<i32> {
        self.names.find_skill(name).filter(|id| self.knows(*id))
    }
}

#[derive(Debug, Default)]
pub struct Timers {
    last: HashMap<String, Duration>,
}

impl Timers {
    pub fn ready(&self, key: &str, now: Duration, delay: Duration) -> bool {
        self.last.get(key).is_none_or(|at| now.saturating_sub(*at) >= delay)
    }

    pub fn mark(&mut self, key: &str, now: Duration) {
        self.last.insert(key.to_owned(), now);
    }

    pub fn clear(&mut self) {
        self.last.clear();
    }
}

pub fn distance(a: glam::Vec3, b: glam::Vec3) -> f32 {
    a.distance(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timers_gate_by_key_and_delay() {
        let mut timers = Timers::default();
        assert!(timers.ready("a", Duration::ZERO, Duration::from_secs(1)));
        timers.mark("a", Duration::ZERO);
        assert!(!timers.ready("a", Duration::from_millis(500), Duration::from_secs(1)));
        assert!(timers.ready("a", Duration::from_secs(1), Duration::from_secs(1)));
        assert!(timers.ready("b", Duration::ZERO, Duration::from_secs(60)));
        timers.clear();
        assert!(timers.ready("a", Duration::ZERO, Duration::from_secs(60)));
    }
}
