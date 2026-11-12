use crate::env::Env;
use crate::options::DeadOptions;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadAction {
    StopBotting,
    Restart(i32),
    Logout,
}

#[derive(Debug, Default)]
pub struct Dead {
    since: Option<Duration>,
    stopped: bool,
    returned: bool,
    logged_out: bool,
}

impl Dead {
    pub fn step(&mut self, env: &Env<'_>, options: &DeadOptions) -> Option<DeadAction> {
        let dead = env.me().is_some_and(|me| me.dead) || env.world.death.is_some();
        if !dead {
            *self = Dead::default();
            return None;
        }
        let since = *self.since.get_or_insert(env.now);
        let elapsed = env.now.saturating_sub(since);
        if options.toggle_botting && !self.stopped {
            self.stopped = true;
            return Some(DeadAction::StopBotting);
        }
        if options.return_enabled && !self.returned && elapsed >= Duration::from_secs(u64::from(options.return_delay_seconds)) {
            let allowed = env.world.death.as_ref().is_some_and(|die| match options.return_point {
                0 => die.to_village,
                1 => die.to_hideout,
                2 => die.to_castle,
                3 => die.to_siege_hq,
                4 => die.to_fixed_spot,
                _ => false,
            });
            if allowed {
                self.returned = true;
                return Some(DeadAction::Restart(i32::from(options.return_point)));
            }
        }
        if options.logout_enabled && !self.logged_out && elapsed >= Duration::from_secs(u64::from(options.logout_delay_seconds)) {
            self.logged_out = true;
            return Some(DeadAction::Logout);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::IdNames;
    use crate::testkit::{Vitals, user_info, world_of};
    use killer_world::World;
    use std::collections::HashSet;

    fn dead_world(town: bool, castle: bool) -> World {
        let mut world = world_of(&[user_info(1, "Wender", &Vitals::full(), [0, 0, 0])]);
        world.me.as_mut().unwrap().dead = true;
        world.death = Some(l2_net::game::Die { object_id: 1, to_village: town, to_hideout: false, to_castle: castle, to_siege_hq: false, sweepable: false, to_fixed_spot: false });
        world
    }

    fn env<'a>(world: &'a World, sets: &'a HashSet<i32>, now: u64) -> Env<'a> {
        Env { world, names: &IdNames, now: Duration::from_secs(now), sweepable: sets, spoiled: sets }
    }

    #[test]
    fn dead_settings_run_in_order_and_each_only_once() {
        let none = HashSet::new();
        let world = dead_world(true, true);
        let options = DeadOptions { toggle_botting: true, return_enabled: true, return_delay_seconds: 10, return_point: 0, logout_enabled: true, logout_delay_seconds: 30 };
        let mut dead = Dead::default();
        assert_eq!(dead.step(&env(&world, &none, 0), &options), Some(DeadAction::StopBotting));
        assert_eq!(dead.step(&env(&world, &none, 5), &options), None);
        assert_eq!(dead.step(&env(&world, &none, 10), &options), Some(DeadAction::Restart(0)));
        assert_eq!(dead.step(&env(&world, &none, 20), &options), None);
        assert_eq!(dead.step(&env(&world, &none, 31), &options), Some(DeadAction::Logout));
        assert_eq!(dead.step(&env(&world, &none, 60), &options), None);
    }

    #[test]
    fn return_only_goes_to_a_point_the_server_offered_and_reviving_resets_everything() {
        let none = HashSet::new();
        let world = dead_world(true, false);
        let options = DeadOptions { return_enabled: true, return_delay_seconds: 1, return_point: 2, ..Default::default() };
        let mut dead = Dead::default();
        assert_eq!(dead.step(&env(&world, &none, 0), &options), None);
        assert_eq!(dead.step(&env(&world, &none, 5), &options), None);
        let alive = world_of(&[user_info(1, "Wender", &Vitals::full(), [0, 0, 0])]);
        assert_eq!(dead.step(&env(&alive, &none, 6), &options), None);
        let castle = dead_world(true, true);
        assert_eq!(dead.step(&env(&castle, &none, 7), &options), None);
        assert_eq!(dead.step(&env(&castle, &none, 9), &options), Some(DeadAction::Restart(2)));
    }
}
