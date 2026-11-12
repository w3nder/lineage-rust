use crate::characters::{self, CharacterCache};
use glam::Vec3;
use killer_assets::abnormal_effects::AbnormalEffectHost;
use killer_assets::abnormal_state::{AbnormalStat, AbnormalStatHost, BIG_HEAD_DEFAULT_MAX, BIG_HEAD_FILE, GrowDimensions, NpcTextureLists, reconcile};
use killer_assets::client::Client;
use killer_assets::texture_modifier::TextureModifyInfo;
use killer_world::{Trigger, World};
use std::collections::{HashMap, HashSet};

type Stat = AbnormalStat<i32, u64>;

const IDENTITY_OVERLAY: TextureModifyInfo = TextureModifyInfo { flags: 0, color: u32::MAX, value_08: 1, value_0c: 1 };

#[derive(Default)]
pub struct AbnormalStates {
    stats: HashMap<i32, Vec<Stat>>,
    overlays: HashMap<i32, TextureModifyInfo>,
    classes: HashMap<String, usize>,
    names: Vec<String>,
    missing: HashSet<String>,
    big_head: Option<Option<killer_ui::ini::IniFile>>,
}

struct Host<'a> {
    cache: &'a mut CharacterCache,
    world: &'a mut World,
    drawn: &'a HashSet<i32>,
    client: &'a mut Client,
    overlays: &'a mut HashMap<i32, TextureModifyInfo>,
    classes: &'a mut HashMap<String, usize>,
    names: &'a mut Vec<String>,
    missing: &'a mut HashSet<String>,
    big_head: &'a mut Option<Option<killer_ui::ini::IniFile>>,
    tables: &'a killer_assets::client::DatTables,
}

impl Host<'_> {
    fn key(&self, id: i32) -> Option<characters::AppearanceKey> {
        let entity = self.world.me.as_ref().filter(|me| me.object_id == id).or_else(|| self.world.entities.get(&id))?;
        characters::entity_key(entity)
    }

    fn loaded(&self, id: i32) -> bool {
        self.key(id).is_some_and(|key| self.cache.get(key).is_some())
    }

    fn entity(&self, id: i32) -> Option<&killer_world::Entity> {
        self.world.me.as_ref().filter(|me| me.object_id == id).or_else(|| self.world.entities.get(&id))
    }

    fn npc(&self, id: i32) -> Option<&l2_dat::npcgrp::Npc> {
        characters::npc_record(self.tables, &self.entity(id)?.kind)
    }

    fn intern(&mut self, name: &str) -> usize {
        match self.names.iter().position(|known| known == name) {
            Some(index) => index,
            None => {
                self.names.push(name.to_owned());
                self.names.len() - 1
            }
        }
    }

    fn intern_all(&mut self, names: &[String]) -> Vec<usize> {
        names.iter().map(|name| self.intern(name)).collect()
    }
}

impl AbnormalEffectHost for Host<'_> {
    type Actor = i32;
    type Mesh = i32;
    type Emitter = u64;
    type Name = usize;
    type Material = usize;

    fn destroy_emitter(&mut self, _: i32, emitter: u64) {
        self.cache.forget_transient(emitter);
    }
    fn report_missing_emitter_owner(&mut self) {}
    fn is_pawn(&self, actor: i32) -> bool {
        self.key(actor).is_some()
    }
    fn active_associated_pawn(&self, _: i32) -> Option<i32> {
        None
    }
    fn skeletal_instance(&self, actor: i32) -> Option<i32> {
        self.loaded(actor).then_some(actor)
    }
    fn texture_info(&mut self, actor: i32) -> &mut TextureModifyInfo {
        self.overlays.entry(actor).or_insert(IDENTITY_OVERLAY)
    }
    fn clear_texture_modifier(&mut self, actor: i32) {
        self.overlays.remove(&actor);
    }
    fn head_bone(&mut self, _: i32) -> usize {
        0
    }
    fn clear_bone_scaler(&mut self, actor: i32, _: usize) {
        if let Some(key) = self.key(actor) {
            self.cache.clear_head_scaler(actor, key);
        }
    }
    fn clear_scale_fields(&mut self, _: i32) {}
    fn root_scaler_index(&self, actor: i32) -> Option<usize> {
        self.cache.root_scaler(actor)
    }
    fn remove_scaler(&mut self, actor: i32, index: usize) {
        self.cache.remove_scaler(actor, index);
    }
    fn restore_field_4bc(&mut self, _: i32, _: u32) {}
    fn has_game_data(&self) -> bool {
        !self.tables.npcgrp.is_empty()
    }
    fn has_mesh_instance(&self, actor: i32) -> bool {
        self.loaded(actor)
    }
    fn has_npc_texture_table(&self, actor: i32) -> bool {
        self.is_npc(actor)
    }
    fn normal_npc_textures(&mut self, actor: i32) -> Vec<usize> {
        let textures = self.npc(actor).map(|npc| npc.textures.clone()).unwrap_or_default();
        self.intern_all(&textures)
    }
    fn find_material_sequence(&mut self, _: usize) -> Option<usize> {
        None
    }
    fn load_material_sequence(&mut self, _: usize) -> Option<usize> {
        None
    }
    fn trigger_material_sequence(&mut self, _: usize) {}
    fn set_npc_texture(&mut self, actor: i32, name: usize, index: usize) {
        log::debug!("estado anormal 0x8000: SetTexes(7, {}, {index}) em {actor} sem consumidor no desenho", self.names.get(name).map_or("", String::as_str));
    }
}

impl AbnormalStatHost for Host<'_> {
    type Class = usize;

    fn has_skeletal_mesh(&self, owner: i32) -> bool {
        self.loaded(owner)
    }

    fn pawn_drawn(&self, owner: i32) -> bool {
        self.world.me.as_ref().is_some_and(|me| me.object_id == owner) || self.drawn.contains(&owner)
    }

    fn load_effect_class(&mut self, name: &str) -> Option<usize> {
        if let Some(index) = self.classes.get(name) {
            return Some(*index);
        }
        if self.missing.contains(name) {
            return None;
        }
        if self.client.emitter_class(name).is_none() {
            self.missing.insert(name.to_owned());
            return None;
        }
        let index = self.names.len();
        self.names.push(name.to_owned());
        self.classes.insert(name.to_owned(), index);
        Some(index)
    }

    fn spawn_emitter(&mut self, class: usize, owner: i32, _: [f32; 3], _: [i32; 3]) -> Option<u64> {
        let key = self.key(owner)?;
        let name = self.names.get(class)?.clone();
        log::debug!("estado anormal: {name} em {owner}");
        Some(self.cache.spawn_owner_effect(owner, key, &name))
    }

    fn location_rotation(&self, owner: i32) -> ([f32; 3], [i32; 3]) {
        let entity = self.world.me.as_ref().filter(|me| me.object_id == owner).or_else(|| self.world.entities.get(&owner));
        entity.map_or(([0.0; 3], [0; 3]), |entity| (entity.position.to_array(), [0, entity.heading, 0]))
    }

    fn collision(&self, owner: i32) -> (f32, f32) {
        let entity = self.world.me.as_ref().filter(|me| me.object_id == owner).or_else(|| self.world.entities.get(&owner));
        entity.map_or((0.0, 0.0), |entity| (entity.collision_radius, entity.collision_height))
    }

    fn draw_scale(&self, owner: i32) -> f32 {
        self.key(owner).and_then(|key| self.cache.draw_scale_of(key)).unwrap_or(1.0)
    }

    fn alias_bone(&self, owner: i32, alias: &str) -> Option<usize> {
        self.cache.alias_bone(self.key(owner)?, alias).map(|(bone, _)| bone)
    }

    fn alias_origin(&self, owner: i32, alias: &str) -> [f32; 3] {
        self.key(owner).and_then(|key| self.cache.alias_bone(key, alias)).map_or([0.0; 3], |(_, origin)| origin.to_array())
    }

    fn mesh_origin_z(&self, owner: i32) -> f32 {
        self.key(owner).and_then(|key| self.cache.mesh_origin_z(key)).unwrap_or(0.0)
    }

    fn attach_to_bone(&mut self, _: i32, emitter: u64, bone: usize, relative: [f32; 3]) -> bool {
        self.cache.attach_transient_to_bone(emitter, bone, Vec3::from(relative))
    }

    fn attach_to_root(&mut self, _: i32, emitter: u64) {
        self.cache.attach_transient_to_bone(emitter, 0, Vec3::ZERO);
    }

    fn set_pre_pivot_z(&mut self, emitter: u64, value: f32) {
        self.cache.set_transient_pre_pivot_z(emitter, value);
    }

    fn set_size_scale(&mut self, emitter: u64, scale: f32) {
        self.cache.set_transient_size_scale(emitter, scale);
    }

    fn set_overlay(&mut self, owner: i32, rgb: [u8; 3], mode: u8) {
        self.overlays.entry(owner).or_insert(IDENTITY_OVERLAY).set_overlay(rgb, mode);
    }

    fn stop_action(&mut self, owner: i32) {
        if let Some(entity) = self.world.entity_mut(owner) {
            entity.anim.push(Trigger::ActionStop);
        }
    }

    fn head_scale(&self, owner: i32) -> f32 {
        let mut scale = 0.0;
        if let Some(key) = self.key(owner) {
            self.cache.is_big_head(owner, key, &mut scale);
        }
        scale
    }

    fn set_bone_scale(&mut self, owner: i32, scale: f32, head: bool) {
        if let Some(key) = self.key(owner) {
            self.cache.set_bone_scale(owner, key, scale, head);
        }
    }

    fn pawn_lights(&mut self, owner: i32) -> Option<&mut Vec<killer_assets::pawn_light::PawnLight>> {
        Some(self.cache.pawn_lights_mut(owner))
    }

    fn play_damage_sound(&mut self, owner: i32) {
        let entity = self.world.me.as_ref().filter(|me| me.object_id == owner).or_else(|| self.world.entities.get(&owner));
        let Some(entity) = entity else { return };
        let row = match &entity.kind {
            killer_world::EntityKind::Player { class_id, race, sex, .. } => l2_dat::chargrp::index_for(*race, *sex, *class_id),
            _ => None,
        };
        let context = crate::notifies::DamageContext {
            object_id: owner,
            position: entity.position,
            npc: matches!(entity.kind, killer_world::EntityKind::Npc { .. }),
            npc_record: characters::npc_record(self.tables, &entity.kind),
            char_sounds: row.and_then(|row| self.tables.chargrp.get(row)).map_or(&[][..], |template| template.damage_sounds.as_slice()),
        };
        if let Some(plan) = self.cache.plan_damage_sound(&context) {
            self.cache.push_sound_plan(plan);
        }
    }

    fn play_sound(&mut self, owner: i32, name: &str, volume: f32, radius: f32) {
        let entity = self.world.me.as_ref().filter(|me| me.object_id == owner).or_else(|| self.world.entities.get(&owner));
        let Some(position) = entity.map(|entity| entity.position) else { return };
        self.cache.push_sound_plan(crate::world_sound::SoundPlan { actor: owner, name: name.to_owned(), position, volume, radius, skip_if_playing: false });
    }

    fn is_npc(&self, actor: i32) -> bool {
        self.entity(actor).is_some_and(|entity| matches!(entity.kind, killer_world::EntityKind::Npc { .. }))
    }

    fn npc_texture_lists(&mut self, actor: i32) -> Option<NpcTextureLists<usize>> {
        let (normal, abnormal) = self.npc(actor).map(|npc| (npc.textures.clone(), npc.textures2.clone()))?;
        Some(NpcTextureLists { normal: self.intern_all(&normal), abnormal: self.intern_all(&abnormal) })
    }

    fn grow_dimensions(&self, actor: i32) -> GrowDimensions {
        self.entity(actor).map_or_else(GrowDimensions::default, |entity| GrowDimensions {
            collision: [entity.collision_radius, entity.collision_height],
            base_collision: entity.base_collision,
            z: entity.position.z,
            weapon_scale: 0.0,
        })
    }

    fn set_collision_size(&mut self, actor: i32, size: [f32; 2]) {
        if let Some(entity) = self.world.entity_mut(actor) {
            [entity.collision_radius, entity.collision_height] = size;
        }
    }

    fn set_location_z(&mut self, actor: i32, z: f32) {
        if let Some(entity) = self.world.entity_mut(actor) {
            entity.position.z = z;
        }
    }

    fn set_weapon_scale(&mut self, _: i32, _: f32) {}

    fn big_head_max(&mut self, owner: i32) -> f32 {
        let Some(name) = self.key(owner).and_then(|key| self.cache.master_mesh_name(key)) else { return 0.0 };
        let section = killer_assets::bone_scale::big_head_section(&name).to_owned();
        let client = &mut *self.client;
        let ini = self.big_head.get_or_insert_with(|| client.read_system_text(BIG_HEAD_FILE).ok().map(|text| killer_ui::ini::parse_ini(&text)));
        let Some(ini) = ini.as_ref().filter(|ini| !section.is_empty() && ini.has_section(&section)) else { return 0.0 };
        ini.get(&section, "MaxSize").and_then(|value| value.trim().parse().ok()).unwrap_or(BIG_HEAD_DEFAULT_MAX)
    }
}

impl AbnormalStates {
    pub fn clear(&mut self) {
        self.stats.clear();
        self.overlays.clear();
    }

    pub fn sync(&mut self, cache: &mut CharacterCache, world: &mut World, drawn: &HashSet<i32>, client: &mut Client, tables: &killer_assets::client::DatTables, dt: f32) {
        let pawns: Vec<(i32, u32)> = world
            .me
            .iter()
            .chain(world.entities.values())
            .filter(|entity| characters::entity_key(entity).is_some())
            .map(|entity| (entity.object_id, entity.abnormal_effect as u32))
            .collect();
        let present: HashSet<i32> = pawns.iter().map(|(id, _)| *id).collect();
        let gone: Vec<i32> = self.stats.keys().filter(|id| !present.contains(id)).copied().collect();
        for id in gone {
            for stat in self.stats.remove(&id).unwrap_or_default() {
                if let Some(handle) = stat.emitter {
                    cache.forget_transient(handle);
                }
            }
            self.overlays.remove(&id);
            cache.forget_bone_scalers(id);
        }
        for (id, mask) in pawns {
            let mut stats = self.stats.remove(&id).unwrap_or_default();
            if mask == 0 && stats.is_empty() {
                continue;
            }
            let mut host = Host {
                cache: &mut *cache,
                world: &mut *world,
                drawn,
                client: &mut *client,
                overlays: &mut self.overlays,
                classes: &mut self.classes,
                names: &mut self.names,
                missing: &mut self.missing,
                big_head: &mut self.big_head,
                tables,
            };
            let removed = reconcile(&mut host, id, &mut stats, mask, dt);
            for stat in removed {
                if let Err(error) = stat.into_effect(String::new()).destroy(&mut host) {
                    log::warn!("estado anormal {id}: {error:?}");
                }
            }
            if !stats.is_empty() {
                self.stats.insert(id, stats);
            }
        }
        cache.set_overlays(self.overlays.iter().map(|(id, info)| (*id, info.overlay_factor())));
    }
}
