use killer_world::World;
use killer_world::movement::POSITION_TICK_MS;
use std::time::Duration;

pub const ENGINE_MIN_DELTA: f32 = 0.0005;
pub const ENGINE_MAX_DELTA: f32 = 0.4;

pub fn engine_delta(elapsed: Duration) -> f32 {
    elapsed.as_secs_f32().clamp(ENGINE_MIN_DELTA, ENGINE_MAX_DELTA)
}

pub fn run(world: &mut World, elapsed: Duration) -> f32 {
    let dt = engine_delta(elapsed);
    world.tick(dt);
    dt
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValidatePacer {
    left: f32,
}

impl ValidatePacer {
    pub const TICK: Duration = Duration::from_millis(POSITION_TICK_MS);

    pub fn new() -> Self {
        Self { left: Self::TICK.as_secs_f32() }
    }

    pub fn poll(&mut self, dt: f32, walking: bool) -> bool {
        if !walking {
            return false;
        }
        self.left -= dt;
        if self.left > 0.0 {
            return false;
        }
        self.left += Self::TICK.as_secs_f32();
        true
    }
}

pub fn moved(before: Option<glam::Vec3>, after: Option<glam::Vec3>) -> bool {
    matches!((before, after), (Some(before), Some(after)) if before != after)
}

impl Default for ValidatePacer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;
    use killer_world::{SessionEvent, WorldEvent};
    use l2_net::game::packets::PAPERDOLL_SLOTS;
    use l2_net::game::{MoveToLocation, PaperdollSlot, UserInfoHead, UserStats};

    fn me() -> UserInfoHead {
        UserInfoHead {
            abnormal_effect: 0,
            x: 0,
            y: 0,
            z: 0,
            heading: 0,
            object_id: 1,
            name: "Wender".into(),
            race: 0,
            sex: 1,
            class_id: 109,
            level: 52,
            exp: 12345,
            max_hp: 900,
            hp: 400,
            max_mp: 300,
            mp: 200,
            run_speed: 120,
            walk_speed: 80,
            speed_multiplier: 1.0,
            attack_speed_multiplier: 1.0,
            pvp_flag: 0,
            karma: 0,
            collision_radius: 9.0,
            collision_height: 23.0,
            hair_style: 2,
            hair_color: 1,
            face: 3,
            builder_level: 0,
            title: "Titulo".into(),
            max_cp: 500,
            cp: 450,
            stats: UserStats::default(),
            paperdoll: [PaperdollSlot::default(); PAPERDOLL_SLOTS],
            inventory_limit: 80,
            name_color: 0xffffff,
            social: Default::default(),
            enchant_effect: 0,
        }
    }

    #[test]
    fn the_frame_delta_is_clamped_like_the_engine() {
        assert_eq!(engine_delta(Duration::from_millis(16)), 0.016);
        assert_eq!(engine_delta(Duration::ZERO), ENGINE_MIN_DELTA);
        assert_eq!(engine_delta(Duration::from_secs(2)), ENGINE_MAX_DELTA);
        assert_eq!(ENGINE_MAX_DELTA, 0.4);
        assert_eq!(ENGINE_MIN_DELTA, 0.0005);
    }

    #[test]
    fn irregular_frames_cover_speed_times_the_elapsed_second() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(me())));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation { object_id: 1, to_x: 10_000, to_y: 0, to_z: 0, x: 0, y: 0, z: 0 }));
        let frames = [16u64, 33, 9, 21, 16, 40, 12];
        let mut total = 0u64;
        for &ms in frames.iter().cycle() {
            let ms = ms.min(1000 - total);
            total += ms;
            let dt = run(&mut world, Duration::from_millis(ms));
            assert!((dt - ms as f32 / 1000.0).abs() < 1e-6);
            if total == 1000 {
                break;
            }
        }
        let x = world.me.as_ref().unwrap().position.x;
        assert!((x - 120.0).abs() < 1.0, "{x}");
        assert_eq!(world.me.as_ref().unwrap().dest, Some(Vec3::new(10_000.0, 0.0, 0.0)));
    }

    #[test]
    fn a_stalled_frame_advances_at_most_the_engine_cap() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(me())));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation { object_id: 1, to_x: 10_000, to_y: 0, to_z: 0, x: 0, y: 0, z: 0 }));
        run(&mut world, Duration::from_secs(3));
        let x = world.me.as_ref().unwrap().position.x;
        assert!((x - 48.0).abs() < 0.01, "{x}");
    }

    #[test]
    fn validate_is_due_every_1200_ms_of_walking_and_pauses_while_standing() {
        let mut pacer = ValidatePacer::new();
        assert!(!pacer.poll(0.6, true));
        assert!(!pacer.poll(0.5, true));
        assert!(pacer.poll(0.2, true));
        assert!(!pacer.poll(5.0, false));
        assert!(!pacer.poll(1.0, true));
        assert!(pacer.poll(0.2, true));
        assert!(!pacer.poll(1.09, true));
        assert!(pacer.poll(0.02, true));
        assert_eq!(ValidatePacer::TICK, Duration::from_millis(1200));
    }

    #[test]
    fn validate_runs_on_velocity_every_position_tick_ms() {
        assert_eq!(POSITION_TICK_MS, 1200);
        let tick = POSITION_TICK_MS as f32 / 1000.0;
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(me())));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation { object_id: 1, to_x: 10_000, to_y: 0, to_z: 0, x: 0, y: 0, z: 0 }));
        let mut pacer = ValidatePacer::new();
        let mut sent = Vec::new();
        let mut elapsed = 0.0f32;
        for _ in 0..250 {
            let before = world.me.as_ref().map(|me| me.position);
            let dt = run(&mut world, Duration::from_millis(16));
            elapsed += dt;
            if pacer.poll(dt, moved(before, world.me.as_ref().map(|me| me.position))) {
                sent.push(elapsed);
            }
        }
        assert_eq!(sent.len(), 3, "{sent:?}");
        for (index, at) in sent.iter().enumerate() {
            assert!((at - tick * (index + 1) as f32).abs() <= 0.016 + 1e-4, "{at}");
        }
        assert!(!moved(Some(Vec3::ONE), Some(Vec3::ONE)));
        assert!(!moved(None, Some(Vec3::ONE)));
        assert!(moved(Some(Vec3::ZERO), Some(Vec3::X)));
        let mut standing = World::default();
        standing.apply(&SessionEvent::UserInfo(Box::new(me())));
        let mut idle = ValidatePacer::new();
        for _ in 0..200 {
            let before = standing.me.as_ref().map(|me| me.position);
            let dt = run(&mut standing, Duration::from_millis(16));
            assert!(!idle.poll(dt, moved(before, standing.me.as_ref().map(|me| me.position))));
        }
    }
}
