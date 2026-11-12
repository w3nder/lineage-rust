use crate::characters::{self, AppearanceKey, CharacterCache};
use killer_assets::client::Client;
use killer_world::World;
use killer_world::appendix::{ALL_BITS, MARKER_SIZE_DIVISOR, class_of, pre_pivot_base, scales_with_radius};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct AppendixEffects {
    active: HashMap<(i32, u32), (u64, AppearanceKey)>,
    missing: HashSet<&'static str>,
}

impl AppendixEffects {
    pub fn clear(&mut self) {
        self.active.clear();
    }

    pub fn sync(&mut self, cache: &mut CharacterCache, world: &mut World, client: &mut Client) {
        let exists = |world: &World, id: i32| world.is_me(id) || world.entities.contains_key(&id);
        let known: HashSet<i32> = world.appendix.ids().filter(|id| exists(world, *id)).collect();
        world.appendix.retain(|id| known.contains(&id));
        let mut wanted: HashMap<(i32, u32), (AppearanceKey, f32, f32)> = HashMap::new();
        for id in world.appendix.ids().collect::<Vec<_>>() {
            let mask = world.appendix.effects(id);
            if mask == 0 {
                continue;
            }
            let Some(entity) = world.me.as_ref().filter(|me| me.object_id == id).or_else(|| world.entities.get(&id)) else { continue };
            let Some(key) = characters::entity_key(entity) else { continue };
            if cache.get(key).is_none() {
                continue;
            }
            let origin = cache.draw_scale_of(key).unwrap_or(1.0) * cache.mesh_origin_z(key).unwrap_or(0.0);
            for bit in ALL_BITS {
                if mask & bit != 0 {
                    wanted.insert((id, bit), (key, pre_pivot_base(bit) - origin, entity.collision_radius));
                }
            }
        }
        self.active.retain(|slot, (handle, key)| {
            let keep = wanted.get(slot).is_some_and(|(wanted_key, _, _)| wanted_key == key);
            if !keep {
                cache.forget_transient(*handle);
            }
            keep
        });
        for (slot, (key, z, radius)) in wanted {
            if self.active.contains_key(&slot) {
                continue;
            }
            let Some(class) = class_of(slot.1) else { continue };
            if self.missing.contains(class) {
                continue;
            }
            if client.emitter_class(class).is_none() {
                log::warn!("efeito appendix {class}: classe ausente no cliente");
                self.missing.insert(class);
                continue;
            }
            let handle = cache.spawn_owner_effect(slot.0, key, class);
            cache.set_transient_pre_pivot_z(handle, z);
            if scales_with_radius(slot.1) {
                cache.set_transient_size_scale(handle, radius / MARKER_SIZE_DIVISOR);
            }
            self.active.insert(slot, (handle, key));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_world::appendix::{IDENTIFIER_BLUE, IDENTIFIER_RED, SELECTED_BLUE, SELECTED_RED};

    #[test]
    #[ignore]
    fn the_client_has_every_appendix_class_and_the_four_marker_ones_follow_their_owner_with_a_pre_pivot() {
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        for bit in ALL_BITS {
            let class = class_of(bit).unwrap();
            assert!(client.emitter_class(class).is_some(), "{class}");
        }
        for bit in [IDENTIFIER_RED, IDENTIFIER_BLUE, SELECTED_RED, SELECTED_BLUE] {
            let trailer = client.trailer_class(class_of(bit).unwrap()).unwrap();
            assert_eq!(trailer.physics, killer_assets::client::PHYS_TRAILER, "{bit}");
            assert!(trailer.pre_pivot, "{bit}");
        }
    }

    #[test]
    fn mapping_covers_the_six_bits_with_the_engine_pre_pivot_bases() {
        assert_eq!(ALL_BITS.map(|bit| class_of(bit).is_some()), [true; 6]);
        assert_eq!(class_of(5), None);
        assert_eq!(pre_pivot_base(IDENTIFIER_RED), 0.5);
        assert_eq!(pre_pivot_base(SELECTED_BLUE), 2.0);
        assert_eq!(pre_pivot_base(1), 0.0);
        assert!(scales_with_radius(2) && !scales_with_radius(IDENTIFIER_BLUE));
    }
}
