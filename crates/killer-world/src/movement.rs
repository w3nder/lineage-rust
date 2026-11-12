use glam::Vec3;

use std::collections::HashMap;

use crate::world::{Entity, Follow};

pub const POSITION_TICK_MS: u64 = 1200;
pub const REACHED_RADIUS_CAP: f32 = 30.0;
pub const ADJUST_DISTANCE: f32 = 200.0;
pub const VALIDATE_WALK_DISTANCE: f32 = 2.0;
pub const HEADING_TURN: f32 = 65536.0;
pub const ROTATION_UNITS_PER_RADIAN: f64 = 10430.219195527361;
pub const YAW_MASK: i32 = 0xFFFF;

pub fn step(entity: &mut Entity, dt: f32) {
    expire_speed_bonus(entity, dt);
    walk(entity, dt);
    physics_rotation(entity, dt);
}

pub fn physics_rotation(entity: &mut Entity, dt: f32) {
    if entity.heading == entity.desired_heading {
        return;
    }
    entity.heading = match entity.turn_rate {
        Some(rate) => killer_assets::pawn_rotation::turn_yaw(entity.heading, entity.desired_heading, rate, dt),
        None => entity.desired_heading,
    };
}

fn walk(entity: &mut Entity, dt: f32) {
    let Some(dest) = entity.dest else { return };
    let delta = (dest - entity.position).with_z(0.0);
    if delta.x != 0.0 || delta.y != 0.0 {
        entity.desired_heading = heading_of(delta);
    }
    let remaining = delta.length();
    let advance = entity.speed() * dt;
    if let Some(follow) = entity.follow {
        if remaining <= follow.distance {
            entity.dest = None;
            entity.follow = None;
            return;
        }
        entity.position += delta * (advance / remaining);
        return;
    }
    let reached = remaining <= entity.collision_radius.min(REACHED_RADIUS_CAP);
    if reached || advance >= remaining {
        if !reached {
            entity.position = dest.with_z(entity.position.z);
        }
        entity.dest = None;
        if let Some(heading) = entity.stop_heading.take() {
            entity.set_heading(heading);
        }
        return;
    }
    entity.position += delta * (advance / remaining);
}

fn expire_speed_bonus(entity: &mut Entity, dt: f32) {
    if entity.speed_bonus_left == 0.0 {
        return;
    }
    entity.speed_bonus_left -= dt;
    if entity.speed_bonus_left <= 0.0 {
        entity.speed_bonus_left = 0.0;
        entity.speed_bonus = 0.0;
    }
}

pub fn follow(entity: &mut Entity, origin: Vec3, target_id: i32, target: Vec3, distance: f32) {
    if planar_distance(entity.position, target) <= distance {
        stop(entity);
        return;
    }
    add_move_to(entity, origin, target);
    entity.follow = Some(Follow { target_id, distance });
}

pub fn track(entity: &mut Entity, targets: &HashMap<i32, Vec3>) {
    let Some(follow) = entity.follow else { return };
    match targets.get(&follow.target_id) {
        Some(position) => entity.dest = Some(*position),
        None => stop(entity),
    }
}

pub fn add_move_to(entity: &mut Entity, origin: Vec3, dest: Vec3) {
    entity.follow = None;
    entity.stop_heading = None;
    entity.blocked = false;
    let base = entity.base_speed();
    if base <= 0.0 {
        entity.speed_bonus = 0.0;
        entity.speed_bonus_left = 0.0;
        return;
    }
    let path = planar_distance(dest, origin);
    let actual = planar_distance(entity.position, dest);
    let multiplier = entity.speed_multiplier as f32;
    if path <= 0.0 || multiplier <= 0.0 {
        entity.speed_bonus = 0.0;
        entity.speed_bonus_left = 0.0;
    } else {
        entity.speed_bonus_left = multiplier / base * path;
        entity.speed_bonus = (actual - path) / entity.speed_bonus_left;
    }
    entity.dest = Some(dest);
}

pub fn adjust_pawn_location(entity: &mut Entity, position: Vec3) -> bool {
    if planar_distance(entity.position, position) <= ADJUST_DISTANCE {
        return false;
    }
    entity.position = position;
    true
}

pub fn validate_me(entity: &mut Entity, position: Vec3, heading: i32) -> Option<f32> {
    if entity.sitting || entity.dead {
        return None;
    }
    let delta = planar_distance(entity.position, position);
    entity.position = position;
    entity.set_heading(heading);
    Some(delta)
}

pub const VALIDATE_CATCH_UP_SECONDS: f32 = 3.0;

pub fn validate_other(entity: &mut Entity, position: Vec3) {
    adjust_pawn_location(entity, position);
    if let Some(dest) = entity.dest {
        entity.speed_bonus_left = VALIDATE_CATCH_UP_SECONDS;
        entity.speed_bonus = (planar_distance(dest, entity.position) - planar_distance(dest, position)) / VALIDATE_CATCH_UP_SECONDS;
        return;
    }
    if planar_distance(entity.position, position) > VALIDATE_WALK_DISTANCE {
        add_move_to(entity, position, position);
    }
}

pub fn stop(entity: &mut Entity) {
    entity.follow = None;
    entity.blocked = false;
    entity.dest = None;
    entity.stop_heading = None;
    entity.speed_bonus = 0.0;
    entity.speed_bonus_left = 0.0;
}

pub fn stop_at(entity: &mut Entity, position: Vec3, heading: i32) {
    entity.speed_bonus = 0.0;
    entity.speed_bonus_left = 0.0;
    entity.dest = Some(position);
    entity.stop_heading = Some(heading);
}

pub fn approach(from: Vec3, target: Vec3, distance: f32) -> Option<Vec3> {
    let delta = target - from;
    let planar = delta.truncate().length();
    if planar <= distance.max(0.0) {
        return None;
    }
    let fraction = (planar - distance.max(0.0)) / planar;
    Some(from + delta * fraction)
}

pub fn planar_distance(a: Vec3, b: Vec3) -> f32 {
    (a - b).truncate().length()
}

pub fn heading_of(direction: Vec3) -> i32 {
    let yaw = (f64::from(direction.y).atan2(f64::from(direction.x)) * ROTATION_UNITS_PER_RADIAN) as i32;
    yaw & YAW_MASK
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::AnimPlayer;
    use crate::world::{EntityKind, WHITE_NAME_COLOR};

    fn walker(x: f32, y: f32) -> Entity {
        Entity {
            item_origin: None,
            abnormal_effect: 0,
            object_id: 1,
            kind: EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: false },
            name: String::new(),
            title: String::new(),
            position: Vec3::new(x, y, 0.0),
            heading: 0,
            dest: None,
            stop_heading: None,
            desired_heading: 0,
            turn_rate: None,
            run_speed: 120,
            walk_speed: 60,
            speed_multiplier: 1.0,
            attack_speed_multiplier: 1.0,
            m_atk_speed: 0,
            skill_aim: [0; 3],
            p_atk_speed: 0,
            speed_bonus: 0.0,
            speed_bonus_left: 0.0,
            blocked: false,
            follow: None,
            running: true,
            sitting: false,
            fake_dead: false,
            dead: false,
            walk_anim_frame_in_move: 12,
            hp: None,
            mp: None,
            cp: None,
            level: 0,
            exp: 0,
            collision_radius: 8.0,
            collision_height: 23.0,
            base_collision: [0.0; 2],
            karma: 0,
            pvp_flag: 0,
            relation: 0, auto_attack: false, surrender_war_id: 0,
            target: None,
            casting: None,
            magic: Default::default(),
            anim: AnimPlayer::default(),
            spawned: 0,
            name_color: WHITE_NAME_COLOR,
            recommendations: 0,
            name_blink: false,
            name_blink_elapsed: 0.0,
            clan_id: 0,
            clan_crest_id: 0,
            ally_id: 0,
            ally_crest_id: 0,
            store_type: 0,
            in_combat: false,
            fishing: 0,
            mount: None,
            vehicle: None,
            chair: None,
            rotation: None,
        }
    }

    #[test]
    fn one_second_at_120_covers_120_units() {
        let mut e = walker(0.0, 0.0);
        e.dest = Some(Vec3::new(1000.0, 0.0, 0.0));
        step(&mut e, 1.0);
        assert!((e.position.x - 120.0).abs() < 0.01);
        assert_eq!(e.dest, Some(Vec3::new(1000.0, 0.0, 0.0)));
        e.running = false;
        step(&mut e, 0.5);
        assert!((e.position.x - 150.0).abs() < 0.01);
        e.speed_multiplier = 1.5;
        step(&mut e, 1.0);
        assert!((e.position.x - 240.0).abs() < 0.01);
    }

    #[test]
    fn zero_or_negative_multiplier_counts_as_one() {
        let mut e = walker(0.0, 0.0);
        e.speed_multiplier = 0.0;
        assert_eq!(e.speed(), 120.0);
    }

    #[test]
    fn arrival_snaps_to_dest_in_xy_keeps_the_local_z_and_clears_it() {
        let mut e = walker(0.0, 0.0);
        e.dest = Some(Vec3::new(100.0, 0.0, -5.0));
        step(&mut e, 2.0);
        assert_eq!(e.position, Vec3::new(100.0, 0.0, 0.0));
        assert!(e.dest.is_none());
        step(&mut e, 1.0);
        assert_eq!(e.position, Vec3::new(100.0, 0.0, 0.0));
    }

    #[test]
    fn inside_the_collision_radius_counts_as_reached_and_stops_where_it_is() {
        let mut e = walker(0.0, 0.0);
        e.collision_radius = 8.0;
        e.dest = Some(Vec3::new(7.5, 0.0, 0.0));
        step(&mut e, 0.001);
        assert!(e.dest.is_none());
        assert_eq!(e.position.x, 0.0);
    }

    #[test]
    fn the_reached_radius_is_capped_at_thirty_units() {
        let mut e = walker(0.0, 0.0);
        e.collision_radius = 80.0;
        e.dest = Some(Vec3::new(31.0, 0.0, 0.0));
        step(&mut e, 0.001);
        assert!(e.dest.is_some());
        let stand = e.position.x;
        e.dest = Some(Vec3::new(stand + 29.0, 0.0, 0.0));
        step(&mut e, 0.001);
        assert!(e.dest.is_none());
        assert_eq!(e.position.x, stand);
    }

    #[test]
    fn a_step_longer_than_the_distance_left_still_lands_on_the_destination() {
        let mut e = walker(0.0, 0.0);
        e.collision_radius = 8.0;
        e.run_speed = 1000;
        e.running = true;
        e.dest = Some(Vec3::new(20.0, 0.0, 0.0));
        step(&mut e, 0.5);
        assert!(e.dest.is_none());
        assert_eq!(e.position.x, 20.0);
    }

    #[test]
    fn a_walker_with_a_rotation_rate_turns_toward_the_destination_by_the_rate_times_the_frame() {
        let mut e = walker(0.0, 0.0);
        e.run_speed = 100;
        e.running = true;
        e.turn_rate = Some(65000);
        e.dest = Some(Vec3::new(0.0, 1000.0, 0.0));
        step(&mut e, 0.01);
        assert_eq!((e.desired_heading, e.heading), (16383, 649));
        for _ in 0..60 {
            step(&mut e, 0.01);
        }
        assert_eq!(e.heading, 16383);
    }

    #[test]
    fn a_class_with_a_zero_rotation_rate_never_turns() {
        let mut e = walker(0.0, 0.0);
        e.turn_rate = Some(0);
        e.dest = Some(Vec3::new(0.0, 1000.0, 0.0));
        step(&mut e, 0.1);
        assert_eq!((e.desired_heading, e.heading), (16383, 0));
    }

    #[test]
    fn heading_follows_the_direction_in_ue_units() {
        for (dx, dy, expected) in
            [(1.0, 0.0, 0), (0.0, 1.0, 16383), (-1.0, 0.0, 32767), (0.0, -1.0, 49153), (1.0, 1.0, 8191), (-1.0, -1.0, 40961)]
        {
            let mut e = walker(0.0, 0.0);
            e.dest = Some(Vec3::new(dx * 1000.0, dy * 1000.0, 0.0));
            step(&mut e, 0.1);
            assert_eq!(e.heading, expected, "direcao {dx},{dy}");
        }
        assert_eq!(heading_of(Vec3::new(0.0, -1.0, 0.0)), 49153);
        assert_eq!(heading_of(Vec3::ZERO), 0);
        assert_eq!(heading_of(Vec3::new(1.0, -0.0001, 0.0)), 65535);
        assert_eq!(heading_of(Vec3::new(1.0, 0.00009, 0.0)), 0);
        assert_eq!(heading_of(Vec3::new(1.0, -0.001, 0.0)), 65526);
        assert_eq!(ROTATION_UNITS_PER_RADIAN, 65535.0 / std::f64::consts::TAU);
    }

    #[test]
    fn speed_is_measured_in_the_xy_plane_and_z_is_left_to_the_floor() {
        let mut e = walker(0.0, 0.0);
        e.position.z = 7.0;
        e.dest = Some(Vec3::new(300.0, 400.0, 1000.0));
        step(&mut e, 1.0);
        assert!((e.position.x - 72.0).abs() < 0.01, "{}", e.position.x);
        assert!((e.position.y - 96.0).abs() < 0.01, "{}", e.position.y);
        assert_eq!(e.position.z, 7.0);
        assert!(e.dest.is_some());
        let mut e = walker(0.0, 0.0);
        e.dest = Some(Vec3::new(0.0, 0.0, 500.0));
        step(&mut e, 0.001);
        assert_eq!(e.position, Vec3::ZERO);
        assert!(e.dest.is_none());
    }

    #[test]
    fn a_short_click_turns_the_entity_even_when_it_arrives_at_once() {
        let mut e = walker(0.0, 0.0);
        e.dest = Some(Vec3::new(0.0, 3.0, 0.0));
        step(&mut e, 0.016);
        assert_eq!(e.heading, 16383);
        assert!(e.dest.is_none());
        let mut e = walker(0.0, 0.0);
        e.set_heading(777);
        e.dest = Some(Vec3::ZERO);
        step(&mut e, 0.016);
        assert_eq!(e.heading, 777);
    }

    #[test]
    fn heading_is_kept_when_standing_still() {
        let mut e = walker(0.0, 0.0);
        e.set_heading(777);
        step(&mut e, 1.0);
        assert_eq!(e.heading, 777);
    }

    #[test]
    fn add_move_to_keeps_the_local_position_and_matches_the_server_arrival_time() {
        let mut e = walker(100.0, 0.0);
        add_move_to(&mut e, Vec3::new(40.0, 0.0, 0.0), Vec3::new(1000.0, 0.0, 0.0));
        assert_eq!(e.position, Vec3::new(100.0, 0.0, 0.0));
        assert_eq!(e.dest, Some(Vec3::new(1000.0, 0.0, 0.0)));
        assert!((e.speed_bonus_left - 8.0).abs() < 1e-4, "{}", e.speed_bonus_left);
        assert!((e.speed_bonus + 7.5).abs() < 1e-4, "{}", e.speed_bonus);
        assert!((e.speed() - 112.5).abs() < 1e-4);
        for _ in 0..85 {
            step(&mut e, 0.1);
        }
        assert!((e.position.x - 1000.0).abs() < 1.0, "{}", e.position.x);
        assert_eq!(e.speed_bonus, 0.0);
        assert_eq!(e.speed_bonus_left, 0.0);
        assert_eq!(e.speed(), 120.0);
    }

    #[test]
    fn add_move_to_speeds_up_a_pawn_that_is_behind_the_server() {
        let mut e = walker(0.0, 0.0);
        add_move_to(&mut e, Vec3::new(60.0, 0.0, 0.0), Vec3::new(660.0, 0.0, 0.0));
        assert!((e.speed_bonus_left - 5.0).abs() < 1e-4);
        assert!((e.speed_bonus - 12.0).abs() < 1e-4);
        step(&mut e, 5.0);
        assert!((e.position.x - 660.0).abs() < 1.0, "{}", e.position.x);
    }

    #[test]
    fn add_move_to_without_a_path_or_speed_carries_no_correction() {
        let mut e = walker(30.0, 0.0);
        e.speed_bonus = 9.0;
        e.speed_bonus_left = 9.0;
        add_move_to(&mut e, Vec3::new(50.0, 0.0, 0.0), Vec3::new(50.0, 0.0, 0.0));
        assert_eq!((e.speed_bonus, e.speed_bonus_left), (0.0, 0.0));
        assert_eq!(e.dest, Some(Vec3::new(50.0, 0.0, 0.0)));
        let mut e = walker(0.0, 0.0);
        e.speed_multiplier = 0.0;
        add_move_to(&mut e, Vec3::ZERO, Vec3::new(50.0, 0.0, 0.0));
        assert_eq!((e.speed_bonus, e.speed_bonus_left), (0.0, 0.0));
        let mut e = walker(0.0, 0.0);
        e.run_speed = 0;
        add_move_to(&mut e, Vec3::ZERO, Vec3::new(50.0, 0.0, 0.0));
        assert!(e.dest.is_none());
    }

    #[test]
    fn a_negative_correction_never_walks_backwards() {
        let mut e = walker(10.0, 0.0);
        e.speed_multiplier = 0.5;
        add_move_to(&mut e, Vec3::new(-2000.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 20.0));
        assert!((e.speed_bonus + 240.0).abs() < 1e-3, "{}", e.speed_bonus);
        assert_eq!(e.speed(), 0.0);
        e.dest = Some(Vec3::new(500.0, 0.0, 0.0));
        step(&mut e, 1.0);
        assert_eq!(e.position.x, 10.0);
    }

    #[test]
    fn adjust_pawn_location_only_moves_beyond_200_units_in_xy() {
        let mut e = walker(0.0, 0.0);
        e.position.z = 23.0;
        assert!(!adjust_pawn_location(&mut e, Vec3::new(199.0, 0.0, -500.0)));
        assert_eq!(e.position, Vec3::new(0.0, 0.0, 23.0));
        assert!(adjust_pawn_location(&mut e, Vec3::new(201.0, 0.0, -500.0)));
        assert_eq!(e.position, Vec3::new(201.0, 0.0, -500.0));
    }

    #[test]
    fn validate_me_teleports_in_xy_while_standing_and_reports_the_error() {
        let mut e = walker(100.0, 0.0);
        e.position.z = 23.0;
        e.dest = Some(Vec3::new(1000.0, 0.0, 0.0));
        let delta = validate_me(&mut e, Vec3::new(97.0, 4.0, 0.0), 123);
        assert!((delta.unwrap() - 5.0).abs() < 1e-4);
        assert_eq!(e.position, Vec3::new(97.0, 4.0, 0.0));
        assert_eq!(e.heading, 123);
        assert_eq!(e.dest, Some(Vec3::new(1000.0, 0.0, 0.0)));
    }

    #[test]
    fn validate_me_is_ignored_while_sitting_or_dead() {
        let mut e = walker(100.0, 0.0);
        e.sitting = true;
        assert_eq!(validate_me(&mut e, Vec3::ZERO, 5), None);
        assert_eq!(e.position, Vec3::new(100.0, 0.0, 0.0));
        assert_eq!(e.heading, 0);
        e.sitting = false;
        e.dead = true;
        assert_eq!(validate_me(&mut e, Vec3::ZERO, 5), None);
        assert_eq!(e.position, Vec3::new(100.0, 0.0, 0.0));
    }

    #[test]
    fn validate_other_leaves_a_walker_alone_below_200_units() {
        let mut e = walker(100.0, 0.0);
        e.position.z = 23.0;
        e.dest = Some(Vec3::new(1000.0, 0.0, 0.0));
        validate_other(&mut e, Vec3::new(97.0, 0.0, 0.0));
        assert_eq!(e.position, Vec3::new(100.0, 0.0, 23.0));
        assert_eq!(e.heading, 0);
        assert_eq!(e.dest, Some(Vec3::new(1000.0, 0.0, 0.0)));
        validate_other(&mut e, Vec3::new(400.0, 0.0, 0.0));
        assert_eq!(e.position, Vec3::new(400.0, 0.0, 0.0));
        assert_eq!(e.dest, Some(Vec3::new(1000.0, 0.0, 0.0)));
    }

    #[test]
    fn validate_other_catches_a_walker_up_by_a_third_of_the_distance_error_per_second() {
        let mut e = walker(100.0, 0.0);
        e.dest = Some(Vec3::new(1000.0, 0.0, 0.0));
        validate_other(&mut e, Vec3::new(130.0, 0.0, 0.0));
        assert_eq!(e.position, Vec3::new(100.0, 0.0, 0.0));
        assert_eq!(e.speed_bonus_left, 3.0);
        assert!((e.speed_bonus - 10.0).abs() < 1e-4);
        validate_other(&mut e, Vec3::new(70.0, 0.0, 0.0));
        assert!((e.speed_bonus + 10.0).abs() < 1e-4);
    }

    #[test]
    fn validate_other_walks_an_idle_pawn_to_the_server_spot_beyond_2_units() {
        let mut e = walker(0.0, 0.0);
        validate_other(&mut e, Vec3::new(1.5, 0.0, 0.0));
        assert!(e.dest.is_none());
        validate_other(&mut e, Vec3::new(30.0, 0.0, 0.0));
        assert_eq!(e.position, Vec3::ZERO);
        assert_eq!(e.dest, Some(Vec3::new(30.0, 0.0, 0.0)));
        assert_eq!((e.speed_bonus, e.speed_bonus_left), (0.0, 0.0));
    }

    #[test]
    fn stop_at_walks_to_the_stop_point_and_turns_there() {
        let mut e = walker(0.0, 0.0);
        e.dest = Some(Vec3::new(1000.0, 0.0, 0.0));
        e.speed_bonus = 5.0;
        e.speed_bonus_left = 5.0;
        stop_at(&mut e, Vec3::new(60.0, 0.0, 0.0), 4321);
        assert_eq!(e.dest, Some(Vec3::new(60.0, 0.0, 0.0)));
        assert_eq!(e.speed(), 120.0);
        step(&mut e, 0.25);
        assert_eq!(e.heading, 0);
        assert!(e.dest.is_some());
        step(&mut e, 0.25);
        assert_eq!(e.position, Vec3::new(60.0, 0.0, 0.0));
        assert_eq!(e.heading, 4321);
        assert!(e.dest.is_none() && e.stop_heading.is_none());
    }

    #[test]
    fn a_new_move_forgets_the_stop_heading() {
        let mut e = walker(0.0, 0.0);
        stop_at(&mut e, Vec3::new(60.0, 0.0, 0.0), 4321);
        add_move_to(&mut e, Vec3::ZERO, Vec3::new(0.0, 100.0, 0.0));
        assert!(e.stop_heading.is_none());
        step(&mut e, 10.0);
        assert_eq!(e.heading, 16383);
    }

    #[test]
    fn follow_chases_the_live_target_and_stops_inside_the_distance() {
        let mut e = walker(0.0, 0.0);
        let mut targets = HashMap::new();
        targets.insert(7, Vec3::new(1000.0, 0.0, 0.0));
        follow(&mut e, Vec3::ZERO, 7, Vec3::new(1000.0, 0.0, 0.0), 50.0);
        assert_eq!(e.dest, Some(Vec3::new(1000.0, 0.0, 0.0)));
        targets.insert(7, Vec3::new(0.0, 1000.0, 0.0));
        track(&mut e, &targets);
        assert_eq!(e.dest, Some(Vec3::new(0.0, 1000.0, 0.0)));
        targets.insert(7, Vec3::new(40.0, 0.0, 0.0));
        track(&mut e, &targets);
        step(&mut e, 0.01);
        assert!(e.dest.is_none() && e.follow.is_none());
        assert!(e.position.length() < 40.0);
        follow(&mut e, Vec3::ZERO, 7, Vec3::new(1000.0, 0.0, 0.0), 50.0);
        targets.clear();
        track(&mut e, &targets);
        assert!(e.dest.is_none() && e.follow.is_none());
    }

    #[test]
    fn a_new_move_or_a_stop_clears_the_blocked_mark() {
        let mut e = walker(0.0, 0.0);
        e.blocked = true;
        add_move_to(&mut e, Vec3::ZERO, Vec3::new(50.0, 0.0, 0.0));
        assert!(!e.blocked);
        e.blocked = true;
        stop(&mut e);
        assert!(!e.blocked);
    }

    #[test]
    fn stop_clears_the_movement_without_moving() {
        let mut e = walker(10.0, 0.0);
        e.dest = Some(Vec3::new(1000.0, 0.0, 0.0));
        e.stop_heading = Some(1);
        e.speed_bonus = 2.0;
        stop(&mut e);
        assert_eq!(e.position, Vec3::new(10.0, 0.0, 0.0));
        assert!(e.dest.is_none() && e.stop_heading.is_none());
        assert_eq!(e.speed_bonus, 0.0);
    }

    #[test]
    fn approach_stops_the_given_distance_before_the_target_in_xy() {
        let dest = approach(Vec3::ZERO, Vec3::new(100.0, 0.0, 50.0), 40.0).unwrap();
        assert!((dest - Vec3::new(60.0, 0.0, 30.0)).length() < 0.01, "{dest}");
        let dest = approach(Vec3::ZERO, Vec3::new(30.0, 40.0, 0.0), 10.0).unwrap();
        assert!((dest - Vec3::new(24.0, 32.0, 0.0)).length() < 0.01, "{dest}");
        assert_eq!(approach(Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0), 0.0), Some(Vec3::new(100.0, 0.0, 0.0)));
        assert_eq!(approach(Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0), -5.0), Some(Vec3::new(100.0, 0.0, 0.0)));
    }

    #[test]
    fn approach_is_none_when_already_within_the_distance() {
        assert!(approach(Vec3::ZERO, Vec3::new(30.0, 0.0, 0.0), 40.0).is_none());
        assert!(approach(Vec3::ZERO, Vec3::new(40.0, 0.0, 0.0), 40.0).is_none());
        assert!(approach(Vec3::ZERO, Vec3::new(0.0, 0.0, 500.0), 1.0).is_none());
        assert!(approach(Vec3::ZERO, Vec3::ZERO, 0.0).is_none());
    }
}
