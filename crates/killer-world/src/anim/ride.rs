use std::collections::HashMap;

use l2_dat::animnames::{AnimKind, AnimSet};

use super::names::PawnView;
use super::player::{AnimEvent, AnimPlayer, Trigger, UpdateMovementAnim};
use super::{Sequences, Tables};
use crate::world::Mount;

pub const RIDE_STRIDER: i32 = 1;
pub const RIDE_WYVERN: i32 = 2;
pub const ATTACH_BONE: &str = "Bone15";
pub const MOUNT_NPC_BASE: i32 = 1_000_000;
pub const STRIDER_CLASS: &str = "LineageMonster.strider";
pub const WYVERN_CLASS: &str = "LineageMonster.wyvern";
pub const STRIDER_LABEL_LIFT: f32 = 8.0;
pub const WYVERN_LABEL_LIFT: f32 = 28.0;
pub const MOUNT_WEAPON_TYPE: usize = 0;

pub fn mount_class(ride_type: i32) -> Option<&'static str> {
    match ride_type {
        RIDE_STRIDER => Some(STRIDER_CLASS),
        RIDE_WYVERN => Some(WYVERN_CLASS),
        _ => None,
    }
}

pub fn mount_npc_id(ride_class_id: i32) -> i32 {
    ride_class_id - MOUNT_NPC_BASE
}

pub fn label_lift(ride_type: i32) -> f32 {
    match ride_type {
        RIDE_STRIDER => STRIDER_LABEL_LIFT,
        RIDE_WYVERN => WYVERN_LABEL_LIFT,
        _ => 0.0,
    }
}

pub fn valid(mount: Option<Mount>) -> Option<Mount> {
    mount.filter(|mount| mount_class(mount.ride_type).is_some())
}

#[derive(Debug, Clone, PartialEq)]
pub struct MountAnim {
    pub mount: Mount,
    pub player: AnimPlayer,
}

impl MountAnim {
    pub fn enter(mount: Mount) -> Self {
        let mut player = AnimPlayer::default();
        player.push(Trigger::Spawn);
        MountAnim { mount, player }
    }

    pub fn view(rider: &PawnView, npc_speed: f32) -> PawnView {
        PawnView {
            weapon_type: MOUNT_WEAPON_TYPE,
            npc: true,
            ride_type: None,
            ride_pawn_type: rider.ride_type,
            npc_speed,
            ..*rider
        }
    }

    pub fn tick(&mut self, dt: f32, rider: &PawnView, npc_speed: f32, set: &AnimSet, tables: &dyn Tables, (moving, yaw): (bool, f32)) -> Vec<AnimEvent> {
        let view = Self::view(rider, npc_speed);
        self.player.tick(dt, &view, set, tables, moving, yaw)
    }

    pub fn attack(&mut self, kind: AnimKind, rider: &PawnView, set: &AnimSet, seqs: &dyn Sequences) -> bool {
        let rate = set.rate(kind, MOUNT_WEAPON_TYPE) * rider.attack_speed_rate;
        let name = set.name(kind, MOUNT_WEAPON_TYPE).map(str::to_string);
        self.player.play_anim(name.as_deref(), rate * super::player::SWORD_RATE_BONUS, 0.1 / rate, false, UpdateMovementAnim::Once, seqs)
    }
}

pub fn sync(mounts: &mut HashMap<i32, MountAnim>, object_id: i32, mount: Option<Mount>, rider: &mut AnimPlayer) {
    match (valid(mount), mounts.get(&object_id).map(|current| current.mount)) {
        (Some(now), Some(before)) if now == before => {}
        (Some(now), _) => {
            mounts.insert(object_id, MountAnim::enter(now));
            rider.push(Trigger::Spawn);
        }
        (None, Some(_)) => {
            mounts.remove(&object_id);
            rider.push(Trigger::Spawn);
        }
        (None, None) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::channel::SeqInfo;
    use crate::anim::names::MoveType;
    use crate::anim::player::SkillData;
    use l2_dat::animnames::parse;

    struct Seqs;

    impl Sequences for Seqs {
        fn seq(&self, _name: &str) -> Option<SeqInfo> {
            Some(SeqInfo::new(10, 3.0))
        }
    }

    impl Tables for Seqs {
        fn skill(&self, _skill_id: i32, _level: i32) -> Option<SkillData> {
            None
        }

        fn npc_skill_anim(&self, _npc_class_id: i32, _skill_id: i32) -> Option<String> {
            None
        }
    }

    const STRIDER: &str = "[strider]\nWaitAnimName[0]=wait\nWalkAnimName[0]=walk\nRunAnimName[0]=run\nAtk01AnimName[0]=atk01\nAtk02AnimName[0]=atk02\nAtk03AnimName[0]=atk03\nDeathWaitAnimName[0]=deathwait\n";

    fn set() -> AnimSet {
        parse(STRIDER).remove("strider").unwrap()
    }

    fn rider_view(ride: usize) -> PawnView {
        PawnView { ride_type: Some(ride), move_type: MoveType::Fast, walk_anim_frame_in_move: 12, ..PawnView::standing(3) }
    }

    #[test]
    fn rider_enter_103020ea_picks_the_class_and_the_npc_id_from_the_packet() {
        assert_eq!(mount_class(RIDE_STRIDER), Some("LineageMonster.strider"));
        assert_eq!(mount_class(RIDE_WYVERN), Some("LineageMonster.wyvern"));
        assert_eq!(mount_class(0), None);
        assert_eq!(mount_class(3), None);
        assert_eq!(mount_npc_id(1_012_526), 12_526);
        assert_eq!(ATTACH_BONE, "Bone15");
        assert_eq!((label_lift(1), label_lift(2), label_lift(0)), (8.0, 28.0, 0.0));
    }

    #[test]
    fn the_mount_pawn_is_not_a_rider_but_its_ride_pawn_has_the_ride_type() {
        let view = MountAnim::view(&rider_view(2), 1.5);
        assert_eq!((view.ride_type, view.ride_pawn_type, view.weapon_type, view.npc, view.npc_speed), (None, Some(2), 0, true, 1.5));
    }

    #[test]
    fn the_mount_waits_runs_and_stops_with_its_own_section() {
        let set = set();
        let rider = rider_view(1);
        let mut mount = MountAnim::enter(Mount { ride_type: 1, ride_class_id: 1_012_526 });
        mount.tick(0.0, &rider, 0.0, &set, &Seqs, (false, 0.0));
        assert_eq!(mount.player.current(), Some("wait"));
        mount.tick(0.05, &rider, 0.0, &set, &Seqs, (true, 0.0));
        assert_eq!(mount.player.current(), Some("run"));
        mount.tick(0.05, &rider, 0.0, &set, &Seqs, (false, 0.0));
        assert_eq!(mount.player.current(), Some("wait"));
    }

    #[test]
    fn only_the_wyvern_mount_keeps_animating_when_the_controller_is_dead() {
        let set = set();
        let mut strider = rider_view(1);
        strider.dead = true;
        let mut mount = MountAnim::enter(Mount { ride_type: 1, ride_class_id: 1_012_526 });
        mount.tick(0.0, &strider, 0.0, &set, &Seqs, (false, 0.0));
        assert_eq!(mount.player.current(), Some("deathwait"));
        mount.tick(0.05, &strider, 0.0, &set, &Seqs, (true, 0.0));
        assert_eq!(mount.player.current(), Some("deathwait"));
        let mut wyvern = rider_view(2);
        wyvern.dead = true;
        let mut flyer = MountAnim::enter(Mount { ride_type: 2, ride_class_id: 1_012_527 });
        flyer.tick(0.0, &wyvern, 0.0, &set, &Seqs, (false, 0.0));
        flyer.tick(0.05, &wyvern, 0.0, &set, &Seqs, (true, 0.0));
        assert_eq!(flyer.player.current(), Some("run"));
    }

    #[test]
    fn the_mount_attack_uses_the_mount_section_with_the_sword_formula() {
        let set = set();
        let rider = PawnView { attack_speed_rate: 2.0, ..rider_view(1) };
        let mut mount = MountAnim::enter(Mount { ride_type: 1, ride_class_id: 1_012_526 });
        assert!(mount.attack(AnimKind::Atk02, &rider, &set, &Seqs));
        assert_eq!(mount.player.current(), Some("atk02"));
        assert_eq!(mount.player.update_movement_anim, UpdateMovementAnim::Once);
        assert!((mount.player.channel.rate - 0.3 * 2.0 * 1.1).abs() < 1e-5);
    }

    #[test]
    fn sync_creates_replaces_and_drops_the_mount_and_replays_the_rider_wait() {
        let mut mounts = HashMap::new();
        let mut rider = AnimPlayer::default();
        sync(&mut mounts, 7, None, &mut rider);
        assert!(mounts.is_empty() && rider.pending.is_empty());
        let strider = Mount { ride_type: 1, ride_class_id: 1_012_526 };
        sync(&mut mounts, 7, Some(strider), &mut rider);
        assert_eq!(mounts[&7].mount, strider);
        assert_eq!(rider.pending.len(), 1);
        sync(&mut mounts, 7, Some(strider), &mut rider);
        assert_eq!(rider.pending.len(), 1);
        let wyvern = Mount { ride_type: 2, ride_class_id: 1_012_527 };
        sync(&mut mounts, 7, Some(wyvern), &mut rider);
        assert_eq!(mounts[&7].mount, wyvern);
        assert_eq!(rider.pending.len(), 2);
        sync(&mut mounts, 7, Some(Mount { ride_type: 0, ride_class_id: 5 }), &mut rider);
        assert!(mounts.is_empty());
        assert_eq!(rider.pending.len(), 3);
    }
}
