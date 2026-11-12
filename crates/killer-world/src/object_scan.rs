use glam::Vec3;

use crate::world::{Entity, EntityKind, World};

pub const KIND_ITEM: i32 = 1;
pub const KIND_ENEMY: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ObjectCycle {
    kind: i32,
    last_id: i32,
}

pub fn gno_match(kind: i32, entity: &Entity) -> bool {
    match kind {
        KIND_ITEM => matches!(entity.kind, EntityKind::Item { .. }),
        KIND_ENEMY => matches!(entity.kind, EntityKind::Npc { attackable: true, .. }) && !entity.dead,
        _ => false,
    }
}

impl World {
    pub fn nearest_object_from(&self, from: Vec3, kind: i32, range: f32, exclude: i32) -> Option<i32> {
        let mut best = range;
        let mut found = None;
        let mut candidates: Vec<&Entity> = self.entities.values().filter(|entity| gno_match(kind, entity)).collect();
        candidates.sort_by_key(|entity| entity.object_id);
        for entity in candidates {
            let distance = entity.position.distance(from);
            if distance > 0.0 && distance < best && entity.object_id != exclude {
                best = distance;
                found = Some(entity.object_id);
            }
        }
        found
    }

    pub fn nearest_object(&self, kind: i32, range: f32, exclude: i32) -> Option<i32> {
        let me = self.me.as_ref()?;
        self.nearest_object_from(me.position, kind, range, exclude)
    }

    pub fn pet_nearest_object(&self, kind: i32, range: f32, exclude: i32) -> Option<i32> {
        let pet = self.pet.as_ref()?;
        let from = self.entity(pet.info.object_id)?.position;
        self.nearest_object_from(from, kind, range, exclude)
    }

    pub fn next_object(&self, kind: i32, range: f32, exclude: i32) -> Option<i32> {
        let me = self.me.as_ref()?;
        let mut cycle = self.object_cycle.get();
        if cycle.kind != kind {
            cycle = ObjectCycle { kind, last_id: -1 };
        }
        let mut last_distance = 0.0f32;
        if cycle.last_id > 0 {
            last_distance = self.entity(cycle.last_id).map_or(-1.0, |entity| entity.position.distance(me.position)).max(0.0);
        }
        let mut best = range;
        let mut found = None;
        let mut candidates: Vec<&Entity> = self.entities.values().filter(|entity| gno_match(kind, entity)).collect();
        candidates.sort_by_key(|entity| entity.object_id);
        for entity in candidates {
            let distance = entity.position.distance(me.position);
            if distance > 0.0 && distance > last_distance && distance < best && entity.object_id != exclude {
                best = distance;
                found = Some(entity.object_id);
            }
        }
        let found = found.or_else(|| self.nearest_object(kind, range, exclude));
        if let Some(id) = found {
            cycle.last_id = id;
        }
        self.object_cycle.set(cycle);
        found
    }

    pub fn next_enemy(&self, range: f32, exclude: i32) -> Option<i32> {
        self.next_object(KIND_ENEMY, range, exclude)
    }

    pub fn nearest_item(&self, range: f32, exclude: i32) -> Option<i32> {
        self.nearest_object(KIND_ITEM, range, exclude)
    }

    pub fn pet_nearest_item(&self, range: f32, exclude: i32) -> Option<i32> {
        self.pet_nearest_object(KIND_ITEM, range, exclude)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    fn world_with(entities: &[(i32, f32, bool)]) -> World {
        let mut world = World::default();
        let mut me = Entity::item(1, 0, 1, false, Vec3::ZERO);
        me.kind = EntityKind::Player {
            class_id: 0,
            race: 0,
            sex: 0,
            equipped: [0; l2_net::game::packets::VISIBLE_EQUIP_SLOTS],
            hair_style: 0,
            hair_color: 0,
            face: 0,
            enchant_effect: 0,
            hero: false,
        };
        world.me = Some(me);
        for &(id, x, attackable) in entities {
            let mut entity = Entity::item(id, 0, 1, false, Vec3::new(x, 0.0, 0.0));
            entity.kind = EntityKind::Npc { npc_id: 0, rhand: 0, chest: 0, lhand: 0, attackable };
            world.entities.insert(id, entity);
        }
        world
    }

    #[test]
    fn next_enemy_103f9a60_cycles_by_increasing_distance_and_wraps_to_the_nearest() {
        let world = world_with(&[(10, 50.0, true), (11, 100.0, true), (12, 150.0, true), (13, 20.0, false), (14, 300.0, true)]);
        assert_eq!(world.next_enemy(200.0, -1), Some(10));
        assert_eq!(world.next_enemy(200.0, 10), Some(11));
        assert_eq!(world.next_enemy(200.0, 11), Some(12));
        assert_eq!(world.next_enemy(200.0, 12), Some(10));
        assert_eq!(world.next_enemy(200.0, 10), Some(11));
    }

    #[test]
    fn next_enemy_skips_the_dead_and_the_friendly_and_ignores_distance_zero() {
        let mut world = world_with(&[(10, 0.0, true), (11, 80.0, true), (12, 90.0, true)]);
        world.entities.get_mut(&11).unwrap().dead = true;
        assert_eq!(world.next_enemy(200.0, -1), Some(12));
        assert_eq!(world.next_enemy(200.0, 12), None);
    }

    #[test]
    fn nearest_item_103f99a0_takes_distances_strictly_between_zero_and_the_range() {
        let mut world = World { me: world_with(&[]).me, ..World::default() };
        world.entities.insert(20, Entity::item(20, 57, 1, true, Vec3::new(200.0, 0.0, 0.0)));
        world.entities.insert(21, Entity::item(21, 57, 1, true, Vec3::new(199.0, 0.0, 0.0)));
        world.entities.insert(22, Entity::item(22, 57, 1, true, Vec3::ZERO));
        assert_eq!(world.nearest_item(200.0, -1), Some(21));
        assert_eq!(world.nearest_item(200.0, 21), None);
    }

    #[test]
    fn a_new_kind_resets_the_cycle() {
        let world = world_with(&[(10, 50.0, true), (11, 100.0, true)]);
        assert_eq!(world.next_object(KIND_ENEMY, 200.0, -1), Some(10));
        assert_eq!(world.next_object(KIND_ITEM, 200.0, -1), None);
        assert_eq!(world.next_object(KIND_ENEMY, 200.0, -1), Some(10));
    }
}
