use glam::{Mat3, Mat4, Vec3};
use killer_assets::animation::{
    LocalPose, bind_local_pose, bone_tracks, compose_pose, local_pose, sequence_index, sequence_timing, tween_pose,
};
use killer_assets::characters::{CharacterMesh, NPC_FALLBACK, fallback_npc_appearance};
use killer_assets::client::{DatTables, LANGUAGE_KOREA};
use killer_assets::pawn::{
    Appearance, DUAL_FIST_LEFT_ROTATION, DUAL_FIST_SCALE_DIVISOR, HAND_ALIAS, HEAD_BONE, LEFT_HAND_BONE, NpcPawn, Part,
    Placement, Player, RIGHT_HAND_BONE, SLOT_GLOVES, SLOT_HAIR_A, SLOT_HAIR_B, SLOT_LEFT_ARM, SLOT_LEFT_HAND,
    SLOT_RIGHT_ARM, SLOT_RIGHT_HAND, PawnEffect, WeaponEffect, is_dual_fist, weapon_scale,
};
use killer_assets::scene::actor_lighting::{Bounds, Query, Sample};
use killer_assets::scene::emitters::EmitterAsset;
use killer_assets::scene::transform::{ortho_rotation, rotator_matrix, vector_rotation};
use killer_assets::shots::{SoulShotData, activation_effects, shot_skill};
use killer_assets::textures::TextureBank;
use killer_render::camera::Camera;
use killer_render::skinned::{CharacterId, CharacterInstance};
use killer_ui::font::BitmapFont;
use killer_ui::ini::IniFile;
use killer_world::anim::names::social_anim_duration;
use killer_world::world::{DEFAULT_WALK_ANIM_FRAME_IN_MOVE, WHITE_NAME_COLOR};
use killer_world::{
    AnimEvent, AnimPlayer, Channel, Entity, EntityKind, Hit as AnimHit, SeqInfo, Sequences, SkillData, Tables, Trigger, WaitChange,
    WaitType, World, anim_section, equipped,
};
use l2_dat::animnames::{AnimSet, HideWeapons};
use l2_dat::chargrp::index_for;
use l2_dat::{Equipped, Npc, NpcName};
use l2_net::game::packets::{
    Buff, EQUIP_CHEST, EQUIP_FEET, EQUIP_GLOVES, EQUIP_LEGS, EQUIP_LHAND, EQUIP_RHAND, PartyMember,
    ShortBuffStatusUpdate, VISIBLE_EQUIP_SLOTS,
};
use std::collections::{HashMap, HashSet};
use std::f32::consts::TAU;
use std::sync::Arc;
use ue2_assets::{Bone, MeshAnimation, Sequence};

pub const HEADING_UNITS_PER_TURN: f32 = 65536.0;
pub const CLIPPING_RANGE_UNIT: f32 = 32768.0 * 0.0625;
const DEFAULT_PAWN_CLIPPING: f32 = 2.0;
const MIN_FRAME_RATE_PAWN_CLIPPING: f32 = 0.7;
const DEFAULT_RENDER_ACTOR_LIMITED: i32 = 6;
const DEFAULT_NAME_DIST: i32 = 1000;
const UNLIMITED_ACTORS: usize = 10000;
pub const IGNORED_RANGE_CLASSES: [&str; 11] = [
    "lineagemonster.orfen",
    "lineagemonster.orfen_bi",
    "lineagemonster.queen_ant",
    "lineagemonster.siege_golem",
    "lineagemonster.antaras",
    "lineagenpc.castle_kent_statue",
    "lineagenpc.broadcasting_tower",
    "lineagenpc.castle_aden_controltower_jewel",
    "lineagenpc.castle_kent_statue_jewel",
    "lineagenpc.dead_castle_aden_controltower_jewel",
    "lineagenpc.alive_castle_aden_controltower_jewel",
];
const SIT_LABEL_DROP: f32 = 1.2;
const DEAD_LABEL_DROP: f32 = 1.3;
const BOUNDS_SCALE: f32 = 2.0;
const WHITE: [u8; 3] = [255, 255, 255];
const MAGENTA: [u8; 3] = [255, 0, 255];
const DYNAMIC_HAIR_SKINNED_KIND: i32 = 4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplaySettings {
    pub show_names: bool,
    pub name_dist: i32,
    pub pawn_clipping: f32,
    pub render_limit: usize,
    pub show_my_name: bool,
    pub show_party_member_name: bool,
    pub show_pledge_member_name: bool,
    pub show_other_pc_name: bool,
    pub show_npc_name: bool,
    pub hide_names: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            show_names: false,
            name_dist: DEFAULT_NAME_DIST,
            pawn_clipping: DEFAULT_PAWN_CLIPPING,
            render_limit: render_actor_limit(DEFAULT_RENDER_ACTOR_LIMITED),
            show_my_name: true,
            show_party_member_name: true,
            show_pledge_member_name: true,
            show_other_pc_name: true,
            show_npc_name: true,
            hide_names: false,
        }
    }
}

pub fn render_actor_limit(option: i32) -> usize {
    match option {
        i32::MIN..=1 => 50,
        2 => 100,
        3 => 200,
        4 => 300,
        5 => 400,
        _ => UNLIMITED_ACTORS,
    }
}

impl DisplaySettings {
    pub fn from_ini(l2: &IniFile, option: &IniFile) -> Self {
        let defaults = Self::default();
        let keep_min_frame_rate = option.bool("Video", "IsKeepMinFrameRate").unwrap_or(false);
        let origin_pawn =
            option.get("ClippingRange", "Pawn").and_then(|v| v.parse().ok()).unwrap_or(DEFAULT_PAWN_CLIPPING);
        Self {
            show_names: l2.bool("CharacterDisplay", "Name").unwrap_or(defaults.show_names),
            name_dist: l2.int("CharacterDisplay", "Dist").unwrap_or(defaults.name_dist),
            pawn_clipping: if keep_min_frame_rate { MIN_FRAME_RATE_PAWN_CLIPPING } else { origin_pawn },
            render_limit: render_actor_limit(
                option.int("Video", "RenderActorLimited").unwrap_or(DEFAULT_RENDER_ACTOR_LIMITED),
            ),
            show_my_name: option.bool("Game", "MyName").unwrap_or(defaults.show_my_name),
            show_party_member_name: option.bool("Game", "PartyMemberName").unwrap_or(defaults.show_party_member_name),
            show_pledge_member_name: option
                .bool("Game", "PledgeMemberName")
                .unwrap_or(defaults.show_pledge_member_name),
            show_other_pc_name: option.bool("Game", "OtherPCName").unwrap_or(defaults.show_other_pc_name),
            show_npc_name: option.bool("Game", "NPCName").unwrap_or(defaults.show_npc_name),
            hide_names: l2.bool("URL", "IsL2HideName").unwrap_or(defaults.hide_names),
        }
    }

    pub fn pawn_range(&self) -> f32 {
        self.pawn_clipping * CLIPPING_RANGE_UNIT
    }

    pub fn pawn_in_range(&self, viewer: Vec3, position: Vec3) -> bool {
        (position.truncate() - viewer.truncate()).length() < self.pawn_range()
    }

    pub fn name_in_range(&self, viewer: Vec3, position: Vec3) -> bool {
        self.show_names && ((position - viewer).length() as i32) < self.name_dist
    }

    pub fn shows_name_of(&self, is_me: bool, npc: bool, attackable: bool) -> bool {
        if is_me {
            return self.show_my_name;
        }
        if npc { self.show_npc_name || !attackable } else { self.show_other_pc_name }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub display: DisplaySettings,
    pub viewer: Vec3,
    pub dt: f32,
}

pub fn ignores_range(class: Option<&str>) -> bool {
    class.is_some_and(|c| IGNORED_RANGE_CLASSES.iter().any(|k| k.eq_ignore_ascii_case(c)))
}

pub fn drawn_pawns(world: &World, tables: &DatTables, frame: &Frame) -> Vec<i32> {
    let mut candidates: Vec<(bool, u64, i32)> = Vec::new();
    for entity in world.entities.values() {
        if entity_key(entity).is_none() {
            continue;
        }
        let class = npc_record(tables, &entity.kind).map(|n| n.class.as_str());
        let exempt = ignores_range(class);
        if !exempt && !frame.display.pawn_in_range(frame.viewer, entity.position) {
            continue;
        }
        candidates.push((exempt, entity.spawned, entity.object_id));
    }
    candidates.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)));
    let mut drawn = Vec::with_capacity(candidates.len() + 1);
    let mut counted = 0usize;
    for (exempt, _, object_id) in candidates {
        if exempt {
            drawn.push(object_id);
            continue;
        }
        if counted < frame.display.render_limit {
            drawn.push(object_id);
            counted += 1;
        }
    }
    drawn
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AppearanceKey {
    Npc { npc_id: i32, equipped: Equipped },
    Player { row: usize, face: i32, hair_style: i32, hair_color: i32, equipped: Equipped, enchant_effect: u8, hero: bool, mounted: u8 },
}

pub fn appearance_key(kind: &EntityKind) -> Option<AppearanceKey> {
    match kind {
        EntityKind::Npc { npc_id, .. } => Some(AppearanceKey::Npc { npc_id: *npc_id, equipped: equipped(kind) }),
        EntityKind::Player { class_id, race, sex, face, hair_style, hair_color, enchant_effect, hero, .. } => {
            index_for(*race, *sex, *class_id).map(|row| AppearanceKey::Player {
                row,
                face: *face,
                hair_style: *hair_style,
                hair_color: *hair_color,
                equipped: equipped(kind),
                enchant_effect: *enchant_effect,
                hero: *hero,
                mounted: 0,
            })
        }
        EntityKind::Item { .. } => None,
    }
}

pub fn mount_key(entity: &Entity) -> Option<AppearanceKey> {
    let mount = killer_world::anim::ride::valid(entity.mount)?;
    Some(AppearanceKey::Npc { npc_id: killer_world::anim::ride::mount_npc_id(mount.ride_class_id), equipped: Equipped::default() })
}

pub fn entity_key(entity: &Entity) -> Option<AppearanceKey> {
    let mut key = appearance_key(&entity.kind)?;
    if let AppearanceKey::Player { mounted, .. } = &mut key {
        *mounted = killer_world::anim::ride::valid(entity.mount).map_or(0, |mount| mount.ride_type as u8);
    }
    Some(key)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    pub appearance: Appearance,
    pub draw_scale: f32,
}

pub fn look(key: AppearanceKey, tables: &DatTables) -> Option<Look> {
    match key {
        AppearanceKey::Npc { npc_id, equipped } => {
            Some(match u32::try_from(npc_id).ok().and_then(|id| tables.npcgrp.get(id)) {
                Some(npc) => {
                    let pawn = NpcPawn { npc, equipped, items: tables.items(), mounted: false, petrified: false };
                    Look { appearance: pawn.set_pawn_resource(), draw_scale: 1.0 }
                }
                None => Look { appearance: fallback_npc_appearance(), draw_scale: NPC_FALLBACK.draw_scale },
            })
        }
        AppearanceKey::Player { row, face, hair_style, hair_color, equipped, enchant_effect, hero, mounted } => {
            tables.chargrp.get(row).map(|template| {
                let player = Player {
                    template,
                    row,
                    face,
                    hair_style,
                    hair_color,
                    equipped,
                    tables: tables.pawn_tables(),
                    mounted: (mounted != 0).then_some(mounted),
                    korean: tables.language_type == LANGUAGE_KOREA,
                    korean_flag: false,
                    clan_crest: None,
                };
                let mut appearance = player.set_pawn_resource();
                appearance.enchant_effect = enchant_effect;
                appearance.hero_aura = hero;
                for (slot, submesh, bone) in [
                    (SLOT_RIGHT_HAND, l2_dat::SubMesh::RightHand, RIGHT_HAND_BONE),
                    (SLOT_LEFT_HAND, l2_dat::SubMesh::LeftHand, LEFT_HAND_BONE),
                ] {
                    if appearance.part(slot).is_none() {
                        continue;
                    }
                    let items = tables.items();
                    let id = l2_dat::equipment::item_class_id(&items, &equipped, submesh);
                    let Some(weapon) = items.weapon(id) else { continue };
                    let index = usize::from(slot == SLOT_LEFT_HAND && weapon.handness != l2_dat::weapongrp::anim_type::BOW);
                    if let Some(effect) = weapon.effects.get(index).filter(|e| !e.name.is_empty() && !e.name.eq_ignore_ascii_case("None")) {
                        appearance.weapon_effects.push(WeaponEffect {
                            class: effect.name.clone(),
                            bone,
                            right: slot == SLOT_RIGHT_HAND,
                            offset: effect.offset,
                            size_scale: effect.parameter_1c4,
                            velocity_scale: effect.parameter_1d4,
                            hero: weapon.hero_weapon >= 0,
                        });
                    }
                    let Some(enchant) = weapon.enchant_meshes.get(index).filter(|e| !e.name.is_empty() && !e.name.eq_ignore_ascii_case("None")) else { continue };
                    appearance.parts.push(Part {
                        slot,
                        mesh: enchant.name.clone(),
                        textures: Vec::new(),
                        placement: Placement::Enchanted { bone, scale: enchant.vector_124, offset: enchant.vector_154, hero: weapon.hero_weapon >= 0 },
                    });
                }
                Look { appearance, draw_scale: 1.0 }
            })
        }
    }
}

pub fn npc_record<'a>(tables: &'a DatTables, kind: &EntityKind) -> Option<&'a Npc> {
    match kind {
        EntityKind::Npc { npc_id, .. } => u32::try_from(*npc_id).ok().and_then(|id| tables.npcgrp.get(id)),
        EntityKind::Player { .. } | EntityKind::Item { .. } => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DualFist {
    pub along: f32,
    pub scale: f32,
    pub flip: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PartKind {
    Skinned,
    DynamicHair { head: Option<usize> },
    Attached { bone: Option<usize>, local: Mat4, dual_fist: Option<DualFist> },
    Enchanted { bone: Option<usize>, scale: Vec3, offset: Vec3 },
    Extra,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Collision {
    pub radius: f32,
    pub height: f32,
}

type SequenceTracks<'a> = (&'a [Bone], &'a Sequence, &'a [Option<usize>]);

pub struct LoadedEffect {
    pub effect: PawnEffect,
    pub asset: EmitterAsset,
    bone: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EffectFrame {
    pub object_id: i32,
    pub key: AppearanceKey,
    pub index: usize,
    pub location: Vec3,
    pub rotation: [i32; 3],
    pub zone: u8,
    pub visible: bool,
    pub weapon_scale: f32,
    pub collision_radius: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transient {
    pub handle: u64,
    pub object_id: i32,
    pub key: AppearanceKey,
    pub bone: Option<usize>,
    pub yaw: i32,
    pub relative: Vec3,
    pub follow: Option<OwnerFollow>,
    pub ribbon: bool,
    pub life: Option<f32>,
}

/// `AActor::physTrailer` (0x105fe4c0), ramo `Physics == 10`: posição = `Owner.Location` mais o `TrailerPrePivot` quando a
/// classe tem `bTrailerPrePivot`; rotação = `Owner.Rotation` quando tem `bTrailerSameRotation`, senão a do próprio ator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OwnerFollow {
    pub pre_pivot: Option<Vec3>,
    pub rotation: Option<[i32; 3]>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OwnerSpawn {
    pub object_id: i32,
    pub key: AppearanceKey,
    pub relative: bool,
    pub pre_pivot: Vec3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InstanceLighting {
    pub light_sphere: [f32; 4],
    pub zone: u8,
    pub lights: Vec<killer_assets::scene::lights::PointLight>,
    pub pawn_lights: Vec<killer_assets::scene::lights::PointLight>,
    pub ambient: [u8; 3],
    pub sun_affect: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EffectModifiers {
    pub speed_rate: Option<f32>,
    pub life: Option<f32>,
    pub size_scale: Option<f32>,
    pub delay: Option<(f32, bool, i32)>,
}

impl EffectModifiers {
    pub fn apply(&self, definition: &mut ue2_level::Emitter) {
        if let Some(life) = self.life {
            killer_assets::weapon_effects::adjust_particle_life(definition, life);
        }
        if let Some(scale) = self.size_scale {
            killer_assets::weapon_effects::set_size_scale(definition, scale);
        }
        if let Some((time, all, index)) = self.delay {
            killer_assets::weapon_effects::set_delayed(definition, time, all, index);
        }
    }

    fn of(planned: &killer_assets::skill_effects::Planned) -> Self {
        Self { speed_rate: planned.speed_rate, life: planned.life, size_scale: planned.size_scale, delay: planned.delayed.map(|(time, all, index)| (time, all != 0, index)) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransientSpawn {
    pub handle: u64,
    pub class: String,
    pub stick_velocity: Option<f32>,
    pub modifiers: EffectModifiers,
    pub trailer: Option<(killer_assets::ribbon_point::Trailer, &'static str)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectileSpawn {
    pub target: i32,
    pub caster: i32,
    pub skill_id: i32,
    pub level: i32,
    pub area: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Arrival {
    pub shot: ProjectileSpawn,
    pub location: Vec3,
    pub rotation: [i32; 3],
    pub normal: Vec3,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExplosionRoute {
    pub world_spawns: Vec<WorldSpawn>,
    pub bone_spawns: Vec<(i32, AppearanceKey, &'static str, &'static str, EffectModifiers)>,
    pub shakes: Vec<killer_world::camera_shake::AddShake>,
    pub sound: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArrowModel {
    pub item: i32,
    pub mesh: String,
    pub texture: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArrowSpawn {
    pub model: Option<ArrowModel>,
    pub attacker: i32,
    pub victim: i32,
    pub start: Vec3,
    pub scale: f32,
    pub effects: Vec<WorldSpawn>,
    pub light: bool,
    pub hit: AnimHit,
    pub victim_pawn: bool,
    pub curvature: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldSpawn {
    pub class: String,
    pub location: Vec3,
    pub rotation: [i32; 3],
    pub zone: u8,
    pub modifiers: EffectModifiers,
    pub owner: Option<OwnerSpawn>,
    pub projectile: Option<ProjectileSpawn>,
    pub trailer_physics: Option<(u8, bool, bool)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoneSmoke {
    pub object_id: i32,
    pub skill_id: i32,
    pub bone: &'static str,
    pub z: f32,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewShakePlan {
    pub notify: killer_world::camera_shake::ShakeNotify,
    pub location: Vec3,
    pub randoms: (f32, f32),
    pub smoke: Option<f32>,
}

pub const CRITICAL_HIT_EFFECT: &str = "Lineageeffect.p_u004_a";
const RANGED_SOULSHOT_GRADES: usize = 6;

const HIT_RADIUS_HALF: f64 = 0.5;
const HIT_LARGE_SIZE: f64 = 150.0;
const HIT_REACH_SCALE: f32 = f32::from_bits(0x3f59_999a);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitPawn {
    pub position: Vec3,
    pub rotation: [i32; 3],
    pub radius: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitKind {
    Melee,
    Ranged,
    Notify,
    Strike { notify: bool },
}

pub fn action_attack_sounds(cache: &mut CharacterCache, victim: &Entity, victim_pawn: bool, hit: AnimHit, tables: &DatTables) {
    use crate::notifies::{DamageContext, plan_damage_sound, plan_defense_item_sound, plan_etc_sound};
    let config = cache.sound_config.clone();
    let mut generator = cache.rand.unwrap_or(crate::world_sound::CrtRand::seeded(0x2545_f491));
    let equipped = killer_world::anim::equipped(&victim.kind);
    let row = match &victim.kind {
        EntityKind::Player { class_id, race, sex, .. } => l2_dat::chargrp::index_for(*race, *sex, *class_id),
        _ => None,
    };
    let template = row.and_then(|row| tables.chargrp.get(row));
    let item = |id: i32| u32::try_from(id).ok().filter(|id| *id > 0);
    let chest = item(equipped.chest).and_then(|id| tables.armorgrp.get(&id)).map_or(&[][..], |armor| armor.boots_sounds.as_slice());
    let shield = item(equipped.lhand).and_then(|id| tables.weapongrp.get(&id)).map_or(&[][..], |weapon| weapon.attack_sounds.as_slice());
    fn defense_context<'a>(victim: &Entity, tables: &'a DatTables, item_sounds: &'a [String], char_sounds: &'a [String]) -> crate::notifies::DefenseContext<'a> {
        crate::notifies::DefenseContext {
            object_id: victim.object_id,
            position: victim.position,
            npc: matches!(victim.kind, EntityKind::Npc { .. }),
            item_sounds,
            npc_record: npc_record(tables, &victim.kind),
            char_sounds,
        }
    }
    let own_defense = template.map_or(&[][..], |template| template.defense_sounds.as_slice());
    let mut plans = Vec::new();
    if hit.shield {
        if victim_pawn {
            plans.extend(plan_defense_item_sound(&defense_context(victim, tables, shield, own_defense), &config, || generator.roll()));
        }
    } else if hit.critical {
        plans.extend(plan_etc_sound(victim.object_id, victim.position, &config.critical));
    } else if hit.spirit {
        plans.extend(plan_etc_sound(victim.object_id, victim.position, &config.soul_shot));
    } else if victim_pawn {
        plans.extend(plan_defense_item_sound(&defense_context(victim, tables, chest, own_defense), &config, || generator.roll()));
    }
    if victim_pawn {
        let context = DamageContext {
            object_id: victim.object_id,
            position: victim.position,
            npc: matches!(victim.kind, EntityKind::Npc { .. }),
            npc_record: npc_record(tables, &victim.kind),
            char_sounds: template.map_or(&[][..], |template| template.damage_sounds.as_slice()),
        };
        plans.extend(plan_damage_sound(&context, &config, || generator.roll()));
    }
    cache.rand = Some(generator);
    cache.sound_plans.extend(plans);
}

pub fn arrow_model(world: &World, attacker: &Entity, tables: &DatTables) -> Option<ArrowModel> {
    use killer_assets::pawn::{ArrowOwner, arrow_item_id};
    let owner = if matches!(attacker.kind, EntityKind::Npc { .. }) {
        ArrowOwner::Npc
    } else if world.me.as_ref().is_some_and(|me| me.object_id == attacker.object_id) {
        let left_hand = world.paperdoll[l2_net::game::packets::PAPERDOLL_L_HAND].object_id;
        ArrowOwner::Own { left_hand_item: (left_hand > 0).then(|| world.inventory.find(left_hand).map(|item| item.item_id)).flatten() }
    } else {
        ArrowOwner::Pc
    };
    let item = arrow_item_id(&tables.items(), &killer_world::anim::equipped(&attacker.kind), owner).arrow_item;
    let id = u32::try_from(item).ok()?;
    let (mesh, texture) = match (tables.weapongrp.get(&id), tables.etcitemgrp.get(&id)) {
        (Some(weapon), _) => (weapon.mesh(0), weapon.texture(0)),
        (None, Some(etc)) => (etc.meshes.first().map(String::as_str).filter(|name| !name.is_empty()), etc.textures.first().map(String::as_str).filter(|name| !name.is_empty())),
        (None, None) => return None,
    };
    Some(ArrowModel { item, mesh: mesh?.to_owned(), texture: texture.map(str::to_owned) })
}

pub fn skill_hits<'a>(find: &impl Fn(i32) -> Option<&'a Entity>, caster_id: i32, skill_id: i32, target_id: i32, associated: &[i32], hits: &mut Vec<(i32, AnimHit, HitKind)>) {
    use killer_assets::skill_attacks::{Attack, Victim, attacks};
    use killer_assets::skill_effects::{NPC, SELF_TARGET, SHOT_STAGE_TWO, TARGET_LIST};
    let Some(caster) = find(caster_id) else { return };
    let mut env = SHOT_STAGE_TWO;
    if target_id == caster_id {
        env |= SELF_TARGET;
    }
    if matches!(caster.kind, EntityKind::Npc { .. }) {
        env |= NPC;
    }
    if !associated.is_empty() {
        env |= TARGET_LIST;
    }
    if caster.magic.current.effect_list_head_empty() {
        env |= killer_assets::skill_attacks::EFFECT_LIST_HEAD_EMPTY;
    }
    let victim_of = |victim: Victim| if victim == Victim::Caster { caster_id } else { target_id };
    for attack in attacks(skill_id, env) {
        let (victim, critical, damage, kind) = match *attack {
            Attack::Notify { victim } => (victim, false, 0, HitKind::Notify),
            Attack::Strike { victim, damage, critical, notify } => (victim, critical, damage, HitKind::Strike { notify }),
        };
        let target = victim_of(victim);
        hits.push((caster_id, AnimHit { target, damage, miss: false, critical, shield: false, spirit: false, soulshot_grade: 0 }, kind));
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitContext<'a> {
    pub keep_min_frame_rate: bool,
    pub rendered: bool,
    pub damage_effect: Option<&'a str>,
}

fn sub32(a: f32, b: f32) -> f32 {
    (f64::from(a) - f64::from(b)) as f32
}

pub fn associate_hit_placement(attacker: Option<&HitPawn>, victim: &HitPawn) -> (Vec3, [i32; 3]) {
    let Some(attacker) = attacker else { return (victim.position, victim.rotation) };
    let delta = Vec3::new(sub32(victim.position.x, attacker.position.x), sub32(victim.position.y, attacker.position.y), sub32(victim.position.z, attacker.position.z));
    let mut distance = (f64::from(victim.radius) * HIT_RADIUS_HALF) as f32;
    let size = (f64::from(delta.x) * f64::from(delta.x) + f64::from(delta.y) * f64::from(delta.y) + f64::from(delta.z) * f64::from(delta.z)).sqrt() as f32;
    if f64::from(victim.height) + f64::from(victim.radius) > HIT_LARGE_SIZE {
        let reach = f64::from(size) - f64::from(attacker.radius);
        let wide = f64::from(size) * f64::from(HIT_REACH_SCALE);
        let near = wide as f32;
        distance = if wide < 0.0 {
            0.0
        } else if f64::from(near) < reach {
            near
        } else {
            reach as f32
        };
    }
    let rotation = vector_rotation(delta);
    let back = killer_assets::engine_math::transform_vector_by([distance, 0.0, 0.0], &killer_assets::engine_math::coords_mul_rotator(&killer_assets::engine_math::Coords::UNIT, rotation));
    let location = Vec3::new(sub32(victim.position.x, back[0]), sub32(victim.position.y, back[1]), sub32(victim.position.z, back[2]));
    (location, rotation)
}

fn graded_soulshot(hit: AnimHit, data: &SoulShotData) -> Option<String> {
    let grade = usize::try_from(hit.soulshot_grade).ok().filter(|grade| *grade < RANGED_SOULSHOT_GRADES)?;
    let class = if hit.critical { &data.grades[grade].atk_cr } else { &data.grades[grade].atk };
    (!class.is_empty()).then(|| class.clone())
}

fn damage_effect_of(context: &HitContext) -> Option<String> {
    context.damage_effect.filter(|_| !context.keep_min_frame_rate && context.rendered).map(str::to_owned)
}

pub fn associate_attacked_notify_classes(hit: AnimHit, context: &HitContext, data: &SoulShotData) -> Vec<String> {
    let mut classes: Vec<String> = damage_effect_of(context).into_iter().collect();
    if !context.rendered {
        return classes;
    }
    if hit.spirit {
        classes.extend(graded_soulshot(hit, data));
    } else if hit.critical && !hit.shield {
        classes.push(CRITICAL_HIT_EFFECT.to_owned());
    }
    classes
}

pub fn associate_attacked_notify_effects(attacker: Option<&HitPawn>, victim: &HitPawn, hit: AnimHit, context: &HitContext, data: &SoulShotData) -> Vec<(String, Vec3, [i32; 3])> {
    let (location, rotation) = associate_hit_placement(attacker, victim);
    associate_attacked_notify_classes(hit, context, data).into_iter().map(|class| (class, location, rotation)).collect()
}

pub fn attacked_notify_ranged_classes(hit: AnimHit, context: &HitContext, data: &SoulShotData) -> Vec<String> {
    if hit.miss {
        return Vec::new();
    }
    if !hit.spirit && hit.shield && !hit.critical {
        return Vec::new();
    }
    let mut classes: Vec<String> = damage_effect_of(context).into_iter().collect();
    if !context.rendered {
        return classes;
    }
    if hit.spirit {
        classes.extend(graded_soulshot(hit, data));
    } else if hit.shield {
        classes.push(CRITICAL_HIT_EFFECT.to_owned());
    }
    classes
}

pub fn attacked_notify_ranged_effects(attacker: Vec3, victim: Vec3, victim_radius: f32, hit: AnimHit, context: &HitContext, data: &SoulShotData) -> Vec<(String, Vec3, [i32; 3])> {
    let classes = attacked_notify_ranged_classes(hit, context, data);
    if classes.is_empty() {
        return Vec::new();
    }
    let rotation = vector_rotation(victim - attacker);
    let location = victim - rotator_matrix(rotation).x_axis * victim_radius / 2.0;
    classes.into_iter().map(|class| (class, location, rotation)).collect()
}

pub fn arrow_damage_act(hit: AnimHit, attacker_pawn: bool) -> bool {
    if !attacker_pawn {
        return true;
    }
    !hit.miss && (hit.spirit || !hit.shield)
}

#[allow(clippy::too_many_arguments)]
fn run_skill_effects<'a>(
    cache: &mut CharacterCache,
    find: &impl Fn(i32) -> Option<&'a Entity>,
    phase: killer_assets::skill_effects::Phase,
    object_id: i32,
    key: AppearanceKey,
    skill_id: i32,
    level: i32,
    target_id: i32,
    shot_time: f32,
    area: &[i32],
    zone_of: &impl Fn(Vec3) -> u8,
    tables: &DatTables,
) {
    use killer_assets::skill_effects::{Context, NPC, Party, Phase, SELF_TARGET, SHOT_STAGE_TWO, plan, steps};
    let Some(caster) = find(object_id) else { return };
    let own_target = target_id == object_id;
    let mut env = 0;
    if own_target {
        env |= SELF_TARGET;
    }
    if matches!(caster.kind, EntityKind::Npc { .. }) {
        env |= NPC;
    }
    if phase == Phase::Shot {
        env |= SHOT_STAGE_TWO;
    }
    let target_pawn = if own_target { Some(caster) } else { find(target_id) };
    if phase == Phase::Shot
        && killer_assets::pawn_light::SHOT_LIGHT_SKILLS.contains(&skill_id)
        && let Some(target) = target_pawn.filter(|entity| is_pawn(&entity.kind))
    {
        let (location, rotation, init) = killer_assets::pawn_light::shot_light(caster.position.to_array(), target.position.to_array(), target.collision_radius);
        if let Some(light) = killer_assets::pawn_light::add_pawn_light(cache.pawn_lights_mut(target.object_id), location, rotation, killer_assets::pawn_light::KIND_WORLD) {
            light.init(&init);
        }
    }
    if phase == Phase::Shot {
        queue_shot_smoke(cache, caster, target_pawn, skill_id);
    }
    let steps = steps(phase, skill_id, env);
    if steps.is_empty() {
        return;
    }
    let mode = match phase {
        Phase::Cast => Some(crate::skill_sound::Mode::Cast),
        Phase::Shot => Some(crate::skill_sound::Mode::Shot),
        Phase::PreShot | Phase::Explosion => None,
    };
    if let Some(mode) = mode {
        cache.sound_plans.extend(crate::skill_sound::plans(caster, tables, mode, skill_id, level));
    }
    let Some(caster_actor) = skill_actor(cache, caster) else { return };
    let target_entity = if own_target { Some(caster) } else { find(target_id) };
    let context = Context { caster: Some(caster_actor), target: target_entity.and_then(|entity| skill_actor(cache, entity)), cast_time: shot_time, ..Context::default() };
    let phase_light = match phase {
        Phase::Cast => killer_assets::pawn_light::cast_phase_light(skill_id),
        Phase::Shot => killer_assets::pawn_light::shot_phase_light(skill_id, env & SHOT_STAGE_TWO != 0),
        Phase::PreShot | Phase::Explosion => None,
    };
    if let Some(light) = phase_light
        && let Some(planned) = killer_assets::skill_effects::plan_spawn(phase, steps, light.spawn, &context)
    {
        add_skill_phase_light(cache, &light, &planned, caster, target_entity, shot_time);
    }
    let planned_list = plan(phase, steps, &context);
    let listed = planned_list.iter().rposition(|planned| planned.projectile);
    for (index, planned) in planned_list.into_iter().enumerate() {
        if let Some((points, time, hand)) = planned.trailer
            && !cache.keep_min_frame_rate
            && !matches!(caster.kind, EntityKind::Npc { .. })
        {
            cache.attach_trailer(object_id, key, (caster.collision_radius, caster.collision_height), (points, time, hand != 0), shot_time);
        }
        if let Some(bone) = planned.bone {
            if let killer_assets::skill_effects::Bone::Index(index) = bone {
                cache.spawn_skill_effect_at_bone_index(object_id, key, planned.class, index, EffectModifiers::of(&planned));
            } else if let Some(name) = skill_bone_name(bone) {
                cache.spawn_skill_effect(object_id, key, planned.class, name, EffectModifiers::of(&planned));
            }
            continue;
        }
        let owner = match planned.owner {
            Party::Caster => Some((object_id, key)),
            Party::Target => target_entity.and_then(|entity| Some((entity.object_id, entity_key(entity)?))),
            Party::Null | Party::Projectile | Party::Hit | Party::Area => None,
        };
        cache.world_spawns.push(WorldSpawn {
            class: planned.class.to_owned(),
            location: planned.location,
            rotation: planned.rotation,
            zone: zone_of(planned.location),
            modifiers: EffectModifiers::of(&planned),
            owner: owner.map(|(object_id, key)| OwnerSpawn { object_id, key, relative: planned.relative, pre_pivot: planned.pre_pivot }),
            projectile: (phase == Phase::Shot && planned.projectile && !own_target).then(|| ProjectileSpawn {
                target: target_id,
                caster: object_id,
                skill_id,
                level,
                area: if listed == Some(index) { area.to_vec() } else { Vec::new() },
            }),
            trailer_physics: planned.trailer_physics,
        });
    }
}

fn queue_shot_smoke(cache: &mut CharacterCache, caster: &Entity, target: Option<&Entity>, skill_id: i32) {
    use killer_assets::skill_smoke::{SmokeAt, shot_smoke};
    let Some(caster_actor) = skill_actor(cache, caster) else { return };
    let target_actor = match target {
        Some(entity) => match skill_actor(cache, entity) {
            Some(actor) => Some(actor),
            None => return,
        },
        None => None,
    };
    let Some(smoke) = shot_smoke(skill_id, &caster_actor, target_actor.as_ref()) else { return };
    match smoke.at {
        SmokeAt::Point(point) => {
            let random = if killer_assets::skill_smoke::shake_draws_random(skill_id) { cache.fraction() } else { 0.0 };
            cache.explosion_shakes.extend(killer_assets::skill_smoke::smoke_shake(skill_id, point, random).as_ref().map(smoke_view_shake));
            cache.event_smokes.push((point, smoke.radius));
        }
        SmokeAt::Bone { name, z } => cache.bone_smokes.push(BoneSmoke { object_id: caster.object_id, skill_id, bone: name, z, radius: smoke.radius }),
    }
}

pub fn smoke_view_shake(shake: &killer_assets::skill_smoke::SmokeShake) -> killer_world::camera_shake::AddShake {
    killer_world::camera_shake::AddShake {
        kind: shake.kind,
        time: shake.time,
        scale: shake.scale,
        frequency: shake.frequency,
        count: shake.count,
        direction: shake.direction,
        second: shake.second,
        third: shake.third,
        origin: shake.origin,
        intensity: shake.intensity,
        range: shake.range,
    }
}

fn add_skill_phase_light(
    cache: &mut CharacterCache,
    light: &killer_assets::pawn_light::PhaseLight,
    planned: &killer_assets::skill_effects::Planned,
    caster: &Entity,
    target: Option<&Entity>,
    cast_time: f32,
) {
    use killer_assets::pawn_light::{LightActor, PhaseLightList, PhasePawn, add_phase_light};
    let owner = match light.list {
        PhaseLightList::Caster => caster,
        PhaseLightList::Target => match target.filter(|entity| is_pawn(&entity.kind)) {
            Some(entity) => entity,
            None => return,
        },
    };
    let pawn = |entity: &Entity| PhasePawn { location: entity.position.to_array(), height: entity.collision_height };
    let actor = LightActor { class: planned.class, location: planned.location.to_array(), rotation: planned.rotation, pre_pivot: planned.pre_pivot.to_array() };
    add_phase_light(cache.pawn_lights_mut(owner.object_id), light, actor, pawn(caster), pawn(target.unwrap_or(caster)), cast_time);
}

pub fn light_actor(
    trailers: &HashMap<String, killer_assets::client::TrailerClass>,
    light: &killer_assets::pawn_light::PawnLight,
    pawn: killer_assets::pawn_light::Actor,
) -> killer_assets::pawn_light::Actor {
    let Some(actor) = &light.actor else { return pawn };
    let follow = trailers
        .get(actor.class)
        .filter(|trailer| trailer.physics == killer_assets::client::PHYS_TRAILER && !trailer.relative_trail)
        .map(|trailer| killer_assets::pawn_light::TrailerFollow { pre_pivot: trailer.pre_pivot, same_rotation: trailer.same_rotation });
    killer_assets::pawn_light::follow_actor(actor, pawn, follow)
}

fn skill_actor(cache: &CharacterCache, entity: &Entity) -> Option<killer_assets::skill_effects::Actor> {
    let character = cache.get(entity_key(entity)?)?;
    Some(killer_assets::skill_effects::Actor {
        position: entity.position,
        rotation: [0, entity.heading, 0],
        radius: entity.collision_radius,
        height: entity.collision_height,
        aim: entity.skill_aim,
        skill_rate: entity.m_atk_speed as f32 * killer_world::world::SKILL_SPEED_PER_ATK_SPD,
        draw_scale: character.draw_scale,
        mesh_origin_z: Some(character.parts[character.master].1.origin.z),
        mover: None,
    })
}

pub fn explosion_shake(shake: &killer_assets::skill_effects::explosion::Shake) -> killer_world::camera_shake::AddShake {
    let values = &shake.values;
    killer_world::camera_shake::AddShake {
        kind: shake.kind as u8,
        time: values[0],
        scale: values[1],
        frequency: values[2],
        count: values[3],
        direction: [values[4], values[5], values[6]],
        second: [values[7], values[8], values[9]],
        third: [values[10], values[11], values[12]],
        origin: [values[13], values[14], values[15]],
        intensity: values[16],
        range: values[17],
    }
}

pub fn route_explosion(
    explosion: &killer_assets::skill_effects::explosion::Explosion,
    owner: impl Fn(killer_assets::skill_effects::Party, Option<usize>) -> Option<(i32, AppearanceKey)>,
    caster_present: bool,
    damage_effect: Option<&str>,
    zone_of: impl Fn(Vec3) -> u8,
) -> ExplosionRoute {
    let mut route = ExplosionRoute::default();
    for spawned in &explosion.spawns {
        let planned = &spawned.planned;
        let class = match (spawned.damage_effect, damage_effect) {
            (false, _) => planned.class,
            (true, Some(class)) => class,
            (true, None) => continue,
        };
        let owner = owner(planned.owner, spawned.area);
        if let Some(bone) = planned.bone.filter(|_| !spawned.damage_effect) {
            if let (Some(name), Some((object_id, key))) = (skill_bone_name(bone), owner) {
                route.bone_spawns.push((object_id, key, planned.class, name, EffectModifiers::of(planned)));
            }
            continue;
        }
        route.world_spawns.push(WorldSpawn {
            class: class.to_owned(),
            location: planned.location,
            rotation: planned.rotation,
            zone: zone_of(planned.location),
            modifiers: EffectModifiers::of(planned),
            owner: owner.map(|(object_id, key)| OwnerSpawn { object_id, key, relative: planned.relative, pre_pivot: planned.pre_pivot }),
            projectile: None,
            trailer_physics: planned.trailer_physics,
        });
    }
    route.shakes = explosion.shakes.iter().map(explosion_shake).collect();
    route.sound = caster_present && explosion.sounds.contains(&3);
    route
}

pub fn is_pawn(kind: &EntityKind) -> bool {
    !matches!(kind, EntityKind::Item { .. })
}

pub fn add_hit_light(list: &mut Vec<killer_assets::pawn_light::PawnLight>, attacker: Vec3, victim: Vec3, victim_radius: f32) {
    let (location, rotation) = killer_assets::pawn_light::hit_light(attacker.to_array(), victim.to_array(), victim_radius);
    killer_assets::pawn_light::add_pawn_light(list, location, rotation, killer_assets::pawn_light::KIND_WORLD);
}

pub fn add_explosion_light(list: &mut Vec<killer_assets::pawn_light::PawnLight>, light: &killer_assets::skill_effects::explosion::Light) {
    use killer_assets::pawn_light::{Init, add_pawn_light};
    let Some(added) = add_pawn_light(list, light.location.to_array(), light.rotation, light.kind) else { return };
    if let Some(init) = light.init {
        added.init(&Init {
            kind: init.kind,
            location: init.location.to_array(),
            rotation: init.rotation,
            light_type: init.byte_a as u8,
            effect: init.byte_b as u8,
            plane: init.plane,
            life: init.float_a,
            radius: init.float_b,
            attach: init.byte_c as u8,
        });
    }
}

fn run_explosion<'a>(cache: &mut CharacterCache, find: &impl Fn(i32) -> Option<&'a Entity>, arrival: &Arrival, zone_of: &impl Fn(Vec3) -> u8, tables: &DatTables) {
    use killer_assets::skill_effects::{Actor, Context, Party, Step, explosion};
    let Some(hit_entity) = find(arrival.shot.target) else { return };
    let Some(hit) = skill_actor(cache, hit_entity) else { return };
    let caster_entity = find(arrival.shot.caster);
    let caster = match caster_entity {
        Some(entity) => match skill_actor(cache, entity) {
            Some(actor) => Some(actor),
            None => return,
        },
        None => None,
    };
    let area: Vec<(&Entity, Actor)> = arrival.shot.area.iter().filter_map(|id| {
        let entity = find(*id)?;
        Some((entity, skill_actor(cache, entity)?))
    }).collect();
    let damage_effect = entity_key(hit_entity).and_then(|key| cache.get(key)).and_then(|character| character.damage_effect.clone());
    let mut env = if caster.is_none() { explosion::EXPLOSION_OWNER_NULL } else { 0 };
    if damage_effect.is_some() {
        env |= explosion::EXPLOSION_DAMAGE_EFFECT;
    }
    if !area.is_empty() {
        env |= explosion::EXPLOSION_AREA;
    }
    let steps = explosion::steps(env, arrival.shot.skill_id);
    if steps.is_empty() {
        return;
    }
    let randoms = if steps.iter().any(|step| matches!(step, Step::ViewShake { .. })) { [cache.fraction(), cache.fraction()] } else { [0.0; 2] };
    let context = Context {
        caster,
        target: Some(hit),
        projectile: Some(Actor { position: arrival.location, rotation: arrival.rotation, ..Actor::default() }),
        hit: Some(hit),
        normal: arrival.normal,
        randoms,
        ..Context::default()
    };
    let actors: Vec<Actor> = area.iter().map(|(_, actor)| *actor).collect();
    let explosion = explosion::plan(steps, &context, &actors);
    let party_entity = |party: Party, index: Option<usize>| match party {
        Party::Caster => caster_entity,
        Party::Target | Party::Hit => Some(hit_entity),
        Party::Area => index.and_then(|index| area.get(index)).map(|(entity, _)| *entity),
        Party::Null | Party::Projectile => None,
    };
    let owner = |party, index| party_entity(party, index).and_then(|entity| Some((entity.object_id, entity_key(entity)?)));
    let route = route_explosion(&explosion, owner, caster_entity.is_some(), damage_effect.as_deref(), zone_of);
    for (object_id, key, class, bone, modifiers) in route.bone_spawns {
        cache.spawn_skill_effect(object_id, key, class, bone, modifiers);
    }
    cache.world_spawns.extend(route.world_spawns);
    for light in &explosion.lights {
        let Some(entity) = party_entity(light.pawn, light.area) else { continue };
        add_explosion_light(cache.pawn_lights_mut(entity.object_id), light);
    }
    for &(party, index) in &explosion.damage_acts {
        if let Some(entity) = party_entity(party, index) {
            cache.damage_acts.entry(entity.object_id).or_default().attacked();
        }
    }
    cache.explosion_shakes.extend(route.shakes);
    if route.sound
        && let Some(caster) = caster_entity
    {
        cache.sound_plans.extend(crate::skill_sound::plans(caster, tables, crate::skill_sound::Mode::Explosion, arrival.shot.skill_id, arrival.shot.level));
    }
}

/// `APawn::MagicProcess` (0x104e4f70), bloco que copia a rotação do conjurador para `Pawn+0x320..0x328`
/// e troca o yaw pela direção do alvo (`FVector::Rotation(alvo - conjurador)`), a cada tick da magia
/// enquanto há estágio por cumprir e o alvo existe e não é o próprio conjurador.
fn update_skill_aim(world: &mut World) {
    let aims: Vec<(i32, [i32; 3])> = {
        let find = |id: i32| world.me.iter().find(|me| me.object_id == id).or_else(|| world.entities.get(&id));
        world
            .me
            .iter()
            .chain(world.entities.values())
            .filter_map(|entity| {
                let killer_world::anim::player::Action::Magic(magic) = &entity.anim.action else { return None };
                if magic.target == 0 || magic.target == entity.object_id || magic.stage.is_some_and(|stage| stage >= magic.stages.len()) {
                    return None;
                }
                let target = find(magic.target)?;
                let yaw = vector_rotation(target.position - entity.position)[1];
                Some((entity.object_id, [0, yaw, 0]))
            })
            .collect()
    };
    for (object_id, aim) in aims {
        if let Some(entity) = world.me.iter_mut().find(|me| me.object_id == object_id).or_else(|| world.entities.get_mut(&object_id)) {
            entity.skill_aim = aim;
        }
    }
}

fn skill_bone_name(bone: killer_assets::skill_effects::Bone) -> Option<&'static str> {
    use killer_assets::skill_effects::Bone;
    match bone {
        Bone::RightHand => Some(killer_assets::pawn::RIGHT_HAND_BONE),
        Bone::LeftHand => Some(killer_assets::pawn::LEFT_HAND_BONE),
        Bone::Named(name) => Some(name),
        Bone::Index(_) => None,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransientFrame {
    pub handle: u64,
    pub location: Vec3,
    pub rotation: [i32; 3],
    pub zone: u8,
    pub ribbon: Option<killer_render::renderer::RibbonInput>,
}

pub struct LoadedCharacter {
    pub render_defaults: Option<killer_assets::client::ClassRenderDefaults>,
    effects: Vec<LoadedEffect>,
    pub effect_bank: TextureBank,
    parts: Vec<(CharacterId, Arc<CharacterMesh>)>,
    animation: Option<Arc<MeshAnimation>>,
    bone_tracks: HashMap<usize, Vec<Option<usize>>>,
    sequences: HashMap<String, Option<usize>>,
    draw_scale: f32,
    weapon_type: usize,
    master: usize,
    kinds: Vec<PartKind>,
    damage_bones: killer_assets::damage_act::DamageBones,
    damage_effect: Option<String>,
}

impl LoadedCharacter {
    pub fn new(
        parts: Vec<(CharacterId, Arc<CharacterMesh>)>,
        animation: Option<Arc<MeshAnimation>>,
        draw_scale: f32,
        weapon_type: usize,
    ) -> Option<Self> {
        let master = parts.iter().position(|(_, mesh)| mesh.is_master())?;
        let master_mesh = parts[master].1.clone();
        let gloves = parts.iter().find(|(_, mesh)| mesh.slot == SLOT_GLOVES && mesh.placement == Placement::Skinned);
        let hand = gloves.and_then(|(_, mesh)| mesh.alias(HAND_ALIAS)).map(|alias| Vec3::from(alias.origin));
        let dual_fist_hand = hand.filter(|_| is_dual_fist(weapon_type as u32));
        let kinds = parts.iter().map(|(_, mesh)| part_kind(mesh, &master_mesh, draw_scale, dual_fist_hand)).collect();
        Some(Self {
            render_defaults: None,
            effects: Vec::new(),
            effect_bank: TextureBank::new(),
            parts,
            animation,
            bone_tracks: HashMap::new(),
            sequences: HashMap::new(),
            draw_scale,
            weapon_type,
            master,
            kinds,
            damage_bones: killer_assets::damage_act::DamageBones::default(),
            damage_effect: None,
        })
    }

    pub fn with_pawn(mut self, pawn: &crate::loader::PawnLoad) -> Self {
        let master = &self.parts[self.master].1;
        self.damage_bones = killer_assets::damage_act::DamageBones::resolve(&pawn.defaults, |name| master.match_ref_bone(name));
        self.damage_effect = pawn.damage_effect.clone();
        self
    }

    pub fn with_effects(mut self, effects: Vec<(PawnEffect, EmitterAsset)>, bank: TextureBank) -> Self {
        let master = &self.parts[self.master].1;
        self.effects = effects
            .into_iter()
            .map(|(effect, asset)| {
                let bone = match &effect {
                    PawnEffect::Weapon(weapon) => master.match_ref_bone(weapon.bone),
                    PawnEffect::HeroAura => (!master.bones.is_empty()).then_some(0),
                };
                LoadedEffect { bone, effect, asset }
            })
            .collect();
        self.effect_bank = bank;
        self
    }

    pub fn effect(&self, index: usize) -> Option<&LoadedEffect> {
        self.effects.get(index)
    }

    pub fn bone_world(&self, bone: usize, pose: &[Mat4], position: Vec3, heading: i32) -> Option<Mat4> {
        let master = &self.parts[self.master].1;
        let pose = *pose.get(bone)?;
        let axes = Mat3::from_mat4(pose);
        let axes = Mat3::from_cols(axes.x_axis.normalize_or_zero(), axes.y_axis.normalize_or_zero(), axes.z_axis.normalize_or_zero());
        Some(
            attached_world_matrix(position, heading, master.rotation)
                * Mat4::from_translation(master.scale * self.draw_scale * (pose.transform_point3(Vec3::ZERO) - master.origin))
                * Mat4::from_mat3(axes),
        )
    }

    pub fn effect_placement(&self, index: usize, pose: &[Mat4], position: Vec3, heading: i32, weapon_scale: f32) -> Option<(Vec3, [i32; 3])> {
        let effect = self.effects.get(index)?;
        let world = self.bone_world(effect.bone?, pose, position, heading)?;
        let axes = Mat3::from_mat4(world);
        let axes = Mat3::from_cols(axes.x_axis.normalize_or_zero(), axes.y_axis.normalize_or_zero(), axes.z_axis.normalize_or_zero());
        let relative = match &effect.effect {
            PawnEffect::Weapon(weapon) => Vec3::from(killer_assets::weapon_effects::relative_location(weapon, weapon_scale)),
            PawnEffect::HeroAura => Vec3::ZERO,
        };
        Some((world.transform_point3(relative), ortho_rotation(axes)))
    }

    #[cfg(test)]
    pub fn parts(&self) -> &[(CharacterId, Arc<CharacterMesh>)] {
        &self.parts
    }

    #[cfg(test)]
    pub fn kind(&self, part: usize) -> Option<PartKind> {
        self.kinds.get(part).copied()
    }

    #[cfg(test)]
    pub fn master(&self) -> usize {
        self.master
    }

    #[cfg(test)]
    pub fn animation(&self) -> Option<&Arc<MeshAnimation>> {
        self.animation.as_ref()
    }

    pub fn sequence(&mut self, name: &str) -> Option<usize> {
        let animation = self.animation.as_deref()?;
        *self.sequences.entry(name.to_ascii_lowercase()).or_insert_with(|| sequence_index(animation, name))
    }

    pub fn bounding_radius(&self, collision_radius: f32, collision_height: f32) -> f32 {
        (collision_height + collision_radius) * BOUNDS_SCALE * self.draw_scale
    }

    pub fn light_sphere(&self, position: Vec3, heading: i32) -> (Vec3, f32) {
        let world = self.world(self.master, position, heading);
        let mut low = Vec3::splat(f32::INFINITY);
        let mut high = Vec3::splat(f32::NEG_INFINITY);
        for (index, (_, mesh)) in self.parts.iter().enumerate() {
            if index != self.master && mesh.slot > 6 { continue; }
            let (center, radius) = killer_assets::scene::transform::sphere(mesh.bounding_sphere, world);
            low = low.min(center - Vec3::splat(radius));
            high = high.max(center + Vec3::splat(radius));
        }
        ((low + high) * 0.5, (high - low).max_element() * 0.5)
    }

    pub fn light_bounds(&self, position: Vec3, heading: i32) -> Bounds {
        let world = self.world(self.master, position, heading);
        let mut bounds = Bounds::transformed(self.parts[self.master].1.bounding_box, world);
        for (index, (_, mesh)) in self.parts.iter().enumerate() {
            if index != self.master && mesh.slot <= 6 {
                bounds = bounds.union(Bounds::transformed(mesh.bounding_box, world));
            }
        }
        bounds
    }

    pub fn scale(&self) -> f32 {
        self.parts[self.master].1.scale.x
    }

    #[cfg(test)]
    pub fn master_pose(&mut self, channel: &Channel, memory: &mut PoseMemory) -> Vec<Mat4> {
        self.scaled_master_pose(channel, memory, None, None)
    }

    pub fn scaled_master_pose(&mut self, channel: &Channel, memory: &mut PoseMemory, scalers: Option<&killer_assets::bone_scale::BoneScalers>, damage_yaw: Option<i32>) -> Vec<Mat4> {
        let sequence = channel.sequence.as_deref().and_then(|name| self.sequence(name));
        let mut local = match sequence.and_then(|index| self.tracks(index).map(|tracks| (index, tracks))) {
            Some((index, (bones, sequence, tracks))) => match channel.tween_alpha() {
                Some(alpha) => {
                    let from = memory.tween_from(index, bones);
                    tween_pose(&from, sequence, bones, tracks, alpha)
                }
                None => {
                    memory.tween_from = None;
                    local_pose(sequence, bones, tracks, channel.key_frame(), channel.looping)
                }
            },
            None => bind_local_pose(&self.parts[self.master].1.bones),
        };
        memory.remember(sequence, &local);
        if let Some(yaw) = damage_yaw {
            killer_assets::damage_act::apply(&mut local, &self.damage_bones, yaw);
        }
        let bones = &self.parts[self.master].1.bones;
        match scalers.filter(|scalers| !scalers.is_empty()) {
            Some(scalers) => killer_assets::animation::compose_scaled_pose(bones, &local, &scalers.factors(bones.len())),
            None => compose_pose(bones, &local),
        }
    }

    #[cfg(test)]
    pub fn frame_pose(&mut self, sequence: usize, frame: f32) -> Vec<Mat4> {
        match self.tracks(sequence) {
            Some((bones, sequence, tracks)) => compose_pose(bones, &local_pose(sequence, bones, tracks, frame, false)),
            None => self.parts[self.master].1.bind.clone(),
        }
    }

    fn tracks(&mut self, index: usize) -> Option<SequenceTracks<'_>> {
        let bones = &self.parts[self.master].1.bones;
        let anim = self.animation.as_deref()?;
        let sequence = anim.sequences.get(index)?;
        let tracks = self.bone_tracks.entry(index).or_insert_with(|| bone_tracks(anim, sequence, bones));
        Some((bones, sequence, tracks))
    }

    pub fn hidden(&self, part: usize, hide: HideWeapons) -> bool {
        let slot = self.parts[part].1.slot;
        (hide.right && (slot == SLOT_RIGHT_HAND || slot == SLOT_RIGHT_ARM))
            || (hide.left && (slot == SLOT_LEFT_HAND || slot == SLOT_LEFT_ARM))
    }

    pub fn world(&self, part: usize, position: Vec3, heading: i32) -> Mat4 {
        let master = &self.parts[self.master].1;
        match self.kinds[part] {
            PartKind::Attached { .. } | PartKind::Enchanted { .. } => attached_world_matrix(position, heading, master.rotation),
            _ => world_matrix(position, heading, self.draw_scale, master.local),
        }
    }

    pub fn palette(&self, part: usize, master_pose: &[Mat4], collision: Collision) -> Option<Vec<Mat4>> {
        let (_, mesh) = &self.parts[part];
        let master = &self.parts[self.master].1;
        match self.kinds[part] {
            PartKind::Skinned => Some(
                (0..mesh.bones.len())
                    .map(|b| {
                        master_pose.get(b).zip(master.inverse_bind.get(b)).map_or(Mat4::IDENTITY, |(p, i)| *p * *i)
                    })
                    .collect(),
            ),
            PartKind::Extra => Some(vec![Mat4::IDENTITY; mesh.bones.len()]),
            PartKind::DynamicHair { head } => {
                let head_pose = *master_pose.get(head?)?;
                let root_inverse = *mesh.inverse_bind.first()?;
                Some(vec![head_pose * root_inverse; mesh.bones.len()])
            }
            PartKind::Attached { bone, local, dual_fist } => {
                let bone = bone?;
                let pose = *master_pose.get(bone)?;
                let scale = dual_fist.map_or_else(|| attached_scale(mesh, collision), |d| Vec3::splat(d.scale));
                Some(vec![attached_palette(pose, master, self.draw_scale, local, scale, dual_fist); mesh.bones.len()])
            }
            PartKind::Enchanted { bone, scale, offset } => {
                let bone = bone?;
                let pose = *master_pose.get(bone)?;
                let scale = scale * weapon_scale(collision.radius, collision.height, 0.0);
                Some(vec![enchanted_palette(pose, master, self.draw_scale, mesh, scale, offset); mesh.bones.len()])
            }
        }
    }
}

fn part_kind(mesh: &CharacterMesh, master: &CharacterMesh, draw_scale: f32, dual_fist_hand: Option<Vec3>) -> PartKind {
    match mesh.placement {
        Placement::Master => PartKind::Skinned,
        Placement::Skinned => {
            let dynamic = (mesh.slot == SLOT_HAIR_A || mesh.slot == SLOT_HAIR_B)
                && mesh.hair_kind >= 1
                && mesh.hair_kind != DYNAMIC_HAIR_SKINNED_KIND;
            if dynamic { PartKind::DynamicHair { head: master.match_ref_bone(HEAD_BONE) } } else { PartKind::Skinned }
        }
        Placement::Helm { offset, rotation } => {
            let local = if offset == [0.0; 3] && rotation == [0; 3] {
                Mat4::from_translation(mesh.origin) * Mat4::from_mat3(rotator_matrix(mesh.rotation))
            } else {
                Mat4::from_translation(Vec3::from(offset)) * Mat4::from_mat3(rotator_matrix(rotation))
            };
            PartKind::Attached { bone: master.match_ref_bone(HEAD_BONE), local, dual_fist: None }
        }
        Placement::Attached { bone } | Placement::Extra { bone: Some(bone) } => {
            let local = Mat4::from_translation(-mesh.origin / draw_scale);
            let dual_fist = dual_fist_hand.map(|hand| DualFist {
                along: if mesh.slot == SLOT_LEFT_HAND { hand.x } else { -hand.x },
                scale: dual_fist_scale(hand),
                flip: mesh.slot == SLOT_LEFT_HAND,
            });
            PartKind::Attached { bone: master.match_ref_bone(bone), local, dual_fist }
        }
        Placement::Enchanted { bone, scale, offset, .. } => PartKind::Enchanted { bone: master.match_ref_bone(bone), scale: Vec3::from(scale), offset: Vec3::from(offset) },
        Placement::Extra { bone: None } => PartKind::Extra,
    }
}

fn enchanted_palette(pose: Mat4, master: &CharacterMesh, draw_scale: f32, mesh: &CharacterMesh, scale: Vec3, offset: Vec3) -> Mat4 {
    let local = Mat4::from_translation(offset - mesh.origin / draw_scale);
    Mat4::from_translation(master.scale * draw_scale * (pose.transform_point3(Vec3::ZERO) - master.origin))
        * Mat4::from_mat3(Mat3::from_mat4(pose))
        * Mat4::from_scale(scale)
        * local
}

fn attached_scale(mesh: &CharacterMesh, collision: Collision) -> Vec3 {
    match mesh.placement {
        Placement::Helm { .. } => mesh.scale,
        _ => Vec3::splat(weapon_scale(collision.radius, collision.height, 0.0)),
    }
}

pub const SWIM_BUBBLE_CLASS: &str = "LineageEffect.e_u061_bubble";
pub const SWIM_BREATH_CLASS: &str = "LineageEffect.e_u061_breath";
pub const SWIM_BREATH_BONE: &str = "e_bone";
pub const SWIM_BUBBLE_SPREAD: f32 = 0.9;

pub fn swim_bubbles(generator: &mut crate::world_sound::CrtRand, dt: f32, bones: usize, breath_bone: Option<usize>) -> Vec<(usize, &'static str)> {
    let mut plans = Vec::new();
    if generator.fraction() >= dt / (dt + SWIM_BUBBLE_SPREAD) || bones == 0 {
        return plans;
    }
    let bone = (generator.fraction() * (bones - 1) as f32) as usize;
    plans.push((bone, SWIM_BUBBLE_CLASS));
    if bone.is_multiple_of(3)
        && let Some(breath) = breath_bone
    {
        plans.push((breath, SWIM_BREATH_CLASS));
    }
    plans
}

pub fn dual_fist_scale(hand: Vec3) -> f32 {
    let scale = hand.y / DUAL_FIST_SCALE_DIVISOR;
    if scale == 0.0 { 1.0 } else { scale }
}

fn attached_palette(
    pose: Mat4,
    master: &CharacterMesh,
    draw_scale: f32,
    local: Mat4,
    scale: Vec3,
    dual_fist: Option<DualFist>,
) -> Mat4 {
    let mesh_scale = master.scale * draw_scale;
    let mut rotation = Mat3::from_mat4(pose);
    if dual_fist.is_some_and(|d| d.flip) {
        rotation *= rotator_matrix(DUAL_FIST_LEFT_ROTATION);
    }
    let along = dual_fist.map_or(0.0, |d| d.along);
    let origin = mesh_scale * (pose.transform_point3(Vec3::ZERO) + rotation * Vec3::X * along - master.origin);
    Mat4::from_translation(origin) * Mat4::from_mat3(rotation) * Mat4::from_scale(scale) * local
}

pub fn attached_world_matrix(position: Vec3, heading: i32, rot_origin: [i32; 3]) -> Mat4 {
    Mat4::from_translation(position)
        * Mat4::from_rotation_z(heading as f32 * TAU / HEADING_UNITS_PER_TURN)
        * Mat4::from_mat3(rotator_matrix(rot_origin))
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PoseMemory {
    pub sequence: Option<usize>,
    pub local: LocalPose,
    pub tween_from: Option<LocalPose>,
}

impl PoseMemory {
    pub fn tween_from(&mut self, sequence: usize, bones: &[Bone]) -> LocalPose {
        if self.tween_from.is_none() || self.sequence != Some(sequence) {
            let from = if self.local.len() == bones.len() { self.local.clone() } else { bind_local_pose(bones) };
            self.tween_from = Some(from);
        }
        self.tween_from.clone().unwrap_or_else(|| bind_local_pose(bones))
    }

    pub fn remember(&mut self, sequence: Option<usize>, local: &LocalPose) {
        self.sequence = sequence;
        self.local.clone_from(local);
    }
}

pub struct AnimTables<'a> {
    pub animation: Option<&'a MeshAnimation>,
    pub tables: &'a DatTables,
}

impl Sequences for AnimTables<'_> {
    fn seq(&self, name: &str) -> Option<SeqInfo> {
        let animation = self.animation?;
        let sequence = animation.sequences.get(sequence_index(animation, name)?)?;
        let timing = sequence_timing(sequence);
        let frames = sequence.frames as f32;
        Some(
            SeqInfo::with_notifies(timing.frames, timing.rate, crate::notifies::seq_notifies(&sequence.notifies, sequence.frames.max(1) as f32))
                .with_attack_frames(sequence.attack_eff / frames, sequence.attack_end_eff / frames),
        )
    }
}

impl Tables for AnimTables<'_> {
    fn skill(&self, skill_id: i32, level: i32) -> Option<SkillData> {
        let key = (u32::try_from(skill_id).ok()?, u32::try_from(level).ok()?);
        let skill = self.tables.skillgrp.get(&key)?;
        Some(SkillData {
            animation: skill.ani_char.clone(),
            cast_style: skill.cast_style as i32,
            visual_effect_lead: None,
        })
    }

    fn npc_skill_anim(&self, npc_class_id: i32, skill_id: i32) -> Option<String> {
        let key = (u32::try_from(npc_class_id).ok()?, u32::try_from(skill_id).ok()?);
        self.tables.mobskillanimgrp.get(&key).cloned()
    }
}

pub const DECO_WAIT: &str = "wait";

/// Quantidade de animações sociais que o mapeamento de atraso dos macros usa (ids 0..13,
/// a faixa da tabela em `0x100a87cc`; 0 e 1 são sentar/levantar).
pub const MACRO_SOCIAL_ANIMS: usize = 14;

struct PartLodState {
    slot: u8,
    mesh: String,
    state: killer_assets::skeletal_lod::LodInstance,
}

fn synchronize_lods<'a>(states: &mut Vec<PartLodState>, parts: impl Iterator<Item = (u8, &'a str)> + Clone) {
    if states.len() == parts.clone().count()
        && states.iter().zip(parts.clone()).all(|(state, (slot, mesh))| state.slot == slot && state.mesh.eq_ignore_ascii_case(mesh)) {
        return;
    }
    let mut previous = std::mem::take(states);
    states.extend(parts.map(|(slot, mesh)| {
        let old = previous.iter().position(|state| state.slot == slot && state.mesh.eq_ignore_ascii_case(mesh));
        let state = old.map(|index| previous.remove(index).state).unwrap_or_default();
        PartLodState { slot, mesh: mesh.to_owned(), state }
    }));
}

#[derive(Default)]
pub struct CharacterCache {
    lods: HashMap<i32, Vec<PartLodState>>,
    pending: HashSet<AppearanceKey>,
    failed: HashSet<AppearanceKey>,
    loaded: HashMap<AppearanceKey, LoadedCharacter>,
    wanted: Vec<AppearanceKey>,
    retried: HashSet<AppearanceKey>,
    unknown: HashSet<i32>,
    poses: HashMap<i32, PoseMemory>,
    mounts: HashMap<i32, killer_world::anim::ride::MountAnim>,
    mount_poses: HashMap<i32, PoseMemory>,
    effect_frames: Vec<EffectFrame>,
    transients: Vec<Transient>,
    transient_spawns: Vec<TransientSpawn>,
    transient_frames: Vec<TransientFrame>,
    world_spawns: Vec<WorldSpawn>,
    effect_targets: HashMap<i32, Vec3>,
    hand_positions: HashMap<i32, Vec3>,
    rod_points: HashMap<i32, Vec3>,
    instance_lighting: HashMap<i32, InstanceLighting>,
    arrow_spawns: Vec<ArrowSpawn>,
    arrow_impacts: Vec<(i32, AnimHit)>,
    notify_effects: Vec<(i32, AppearanceKey, crate::notifies::EffectSpawn, u8)>,
    sound_plans: Vec<crate::world_sound::SoundPlan>,
    foot_volumes: HashMap<i32, crate::walk_world::FootVolume>,
    swim_ticks: HashMap<i32, f32>,
    sound_config: crate::world_sound::SoundConfig,
    keep_min_frame_rate: bool,
    bone_scalers: HashMap<i32, killer_assets::bone_scale::BoneScalers>,
    overlays: HashMap<i32, [f32; 4]>,
    view_shakes: Vec<ViewShakePlan>,
    explosion_shakes: Vec<killer_world::camera_shake::AddShake>,
    arrivals: Vec<Arrival>,
    pawn_lights: HashMap<i32, Vec<killer_assets::pawn_light::PawnLight>>,
    trailer_classes: HashMap<String, killer_assets::client::TrailerClass>,
    event_smokes: Vec<(Vec3, f32)>,
    bone_smokes: Vec<BoneSmoke>,
    damage_acts: HashMap<i32, killer_assets::damage_act::DamageAct>,
    screen_fades: Vec<killer_world::anim::ScreenFade>,
    camera_effects: Vec<String>,
    rand: Option<crate::world_sound::CrtRand>,
    next_transient: u64,
    /// Duração de cada animação social do próprio jogador (índice = id social), recalculada a
    /// cada frame em `instances_with`. É o atraso que os comandos de macro de ação esperam —
    /// o mesmo que `APawn::GetSocialAnimDuration` devolve no cliente original.
    pub social_durations: Vec<f32>,
    pub me_waiting: bool,
}

impl CharacterCache {
    pub fn take_effect_frames(&mut self) -> Vec<EffectFrame> {
        std::mem::take(&mut self.effect_frames)
    }

    pub fn take_camera_effects(&mut self) -> Vec<String> {
        std::mem::take(&mut self.camera_effects)
    }

    pub fn take_screen_fades(&mut self) -> Vec<killer_world::anim::ScreenFade> {
        std::mem::take(&mut self.screen_fades)
    }

    pub fn take_view_shakes(&mut self) -> Vec<ViewShakePlan> {
        std::mem::take(&mut self.view_shakes)
    }

    pub fn take_explosion_shakes(&mut self) -> Vec<killer_world::camera_shake::AddShake> {
        std::mem::take(&mut self.explosion_shakes)
    }

    pub fn take_event_smokes(&mut self) -> Vec<(Vec3, f32)> {
        std::mem::take(&mut self.event_smokes)
    }

    pub fn note_trailer_class(&mut self, class: &str, trailer: killer_assets::client::TrailerClass) {
        self.trailer_classes.entry(class.to_owned()).or_insert(trailer);
    }

    pub fn push_arrival(&mut self, arrival: Arrival) {
        self.arrivals.push(arrival);
    }

    pub fn take_sound_plans(&mut self) -> Vec<crate::world_sound::SoundPlan> {
        std::mem::take(&mut self.sound_plans)
    }

    pub fn set_foot_volumes(&mut self, volumes: &HashMap<i32, crate::walk_world::FootVolume>) {
        self.foot_volumes.clone_from(volumes);
    }

    pub fn set_swim_ticks(&mut self, ticks: &HashMap<i32, f32>) {
        self.swim_ticks.clone_from(ticks);
    }

    pub fn set_sound_config(&mut self, config: crate::world_sound::SoundConfig) {
        self.sound_config = config;
    }

    pub fn crt_rand(&mut self) -> i32 {
        self.rand()
    }

    fn rand(&mut self) -> i32 {
        self.rand.get_or_insert(crate::world_sound::CrtRand::seeded(0x2545_f491)).roll()
    }

    fn fraction(&mut self) -> f32 {
        self.rand.get_or_insert(crate::world_sound::CrtRand::seeded(0x2545_f491)).fraction()
    }

    pub fn spawn_shot_effects(&mut self, object_id: i32, key: AppearanceKey, skill_id: i32, data: &SoulShotData) {
        let Some((kind, grade)) = shot_skill(skill_id) else { return };
        let Some(character) = self.loaded.get(&key) else { return };
        let lengths = |slot: u8| character.parts.iter().find(|(_, mesh)| mesh.slot == slot && mesh.placement != Placement::Skinned).map(|(_, mesh)| mesh.floats_150.clone());
        let (left, right) = (lengths(SLOT_LEFT_HAND), lengths(SLOT_RIGHT_HAND));
        let master = &character.parts[character.master].1;
        let effects = activation_effects(data, kind, grade, character.weapon_type, left.as_deref(), right.as_deref());
        for effect in effects {
            let Some(bone) = master.match_ref_bone(if effect.right { RIGHT_HAND_BONE } else { LEFT_HAND_BONE }) else { continue };
            let handle = self.next_transient;
            self.next_transient += 1;
            self.transients.push(Transient { handle, object_id, key, bone: Some(bone), yaw: effect.yaw, relative: Vec3::ZERO, follow: None, ribbon: false, life: None });
            self.transient_spawns.push(TransientSpawn { handle, class: effect.class, stick_velocity: effect.stick_velocity, modifiers: EffectModifiers::default(), trailer: None });
        }
    }

    pub fn spawn_skill_effect(&mut self, object_id: i32, key: AppearanceKey, class: &str, bone: &str, modifiers: EffectModifiers) -> bool {
        self.spawn_attached_effect(object_id, key, class, bone, Vec3::ZERO, modifiers)
    }

    pub fn rod_point(&self, object_id: i32) -> Option<Vec3> {
        self.rod_points.get(&object_id).copied()
    }

    pub fn instance_lighting(&self, object_id: i32) -> Option<&InstanceLighting> {
        self.instance_lighting.get(&object_id)
    }

    pub fn hand_position(&self, object_id: i32) -> Option<Vec3> {
        self.hand_positions.get(&object_id).copied()
    }

    pub fn push_arrow_impact(&mut self, victim: i32, hit: AnimHit) {
        self.arrow_impacts.push((victim, hit));
    }

    pub fn take_arrow_spawns(&mut self) -> Vec<ArrowSpawn> {
        std::mem::take(&mut self.arrow_spawns)
    }

    pub fn push_world_spawn(&mut self, spawn: WorldSpawn) {
        self.world_spawns.push(spawn);
    }

    pub fn effect_target(&self, object_id: i32) -> Option<Vec3> {
        self.effect_targets.get(&object_id).copied()
    }

    pub fn push_sound_plan(&mut self, plan: crate::world_sound::SoundPlan) {
        self.sound_plans.push(plan);
    }

    pub fn spawn_owner_effect(&mut self, object_id: i32, key: AppearanceKey, class: &str) -> u64 {
        let handle = self.next_transient;
        self.next_transient += 1;
        let follow = OwnerFollow { pre_pivot: None, rotation: None };
        self.transients.push(Transient { handle, object_id, key, bone: None, yaw: 0, relative: Vec3::ZERO, follow: Some(follow), ribbon: false, life: None });
        self.transient_spawns.push(TransientSpawn { handle, class: class.to_owned(), stick_velocity: None, modifiers: EffectModifiers::default(), trailer: None });
        handle
    }

    pub fn attach_transient_to_bone(&mut self, handle: u64, bone: usize, relative: Vec3) -> bool {
        let Some(transient) = self.transients.iter_mut().find(|t| t.handle == handle) else { return false };
        transient.bone = Some(bone);
        transient.relative = relative;
        transient.follow = None;
        true
    }

    pub fn set_transient_pre_pivot_z(&mut self, handle: u64, z: f32) {
        if let Some(follow) = self.transients.iter_mut().find(|t| t.handle == handle).and_then(|t| t.follow.as_mut()) {
            follow.pre_pivot = Some(Vec3::new(0.0, 0.0, z));
        }
    }

    pub fn set_transient_size_scale(&mut self, handle: u64, scale: f32) {
        if let Some(spawn) = self.transient_spawns.iter_mut().find(|s| s.handle == handle) {
            spawn.modifiers.size_scale = Some(scale);
        }
    }

    pub fn alias_bone(&self, key: AppearanceKey, alias: &str) -> Option<(usize, Vec3)> {
        let character = self.loaded.get(&key)?;
        let master = &character.parts[character.master].1;
        let entry = master.alias(alias)?;
        Some((master.match_ref_bone(alias)?, Vec3::from(entry.origin)))
    }

    pub fn set_bone_scale(&mut self, object_id: i32, key: AppearanceKey, scale: f32, head: bool) -> bool {
        let Some(character) = self.loaded.get(&key) else { return false };
        let master = &character.parts[character.master].1;
        let name = head.then_some(HEAD_BONE);
        self.bone_scalers.entry(object_id).or_default().set(0, scale, name, !head, |bone| master.match_ref_bone(bone))
    }

    pub fn clear_head_scaler(&mut self, object_id: i32, key: AppearanceKey) {
        let Some(character) = self.loaded.get(&key) else { return };
        let count = character.parts[character.master].1.bones.len();
        if let Some(scalers) = self.bone_scalers.get_mut(&object_id) {
            scalers.clear(HEAD_BONE, count);
        }
    }

    pub fn root_scaler(&self, object_id: i32) -> Option<usize> {
        self.bone_scalers.get(&object_id)?.root_index()
    }

    pub fn remove_scaler(&mut self, object_id: i32, index: usize) {
        if let Some(scalers) = self.bone_scalers.get_mut(&object_id) {
            scalers.remove(index);
        }
    }

    pub fn plan_damage_sound(&mut self, context: &crate::notifies::DamageContext<'_>) -> Option<crate::world_sound::SoundPlan> {
        let config = self.sound_config.clone();
        let mut generator = self.rand.unwrap_or(crate::world_sound::CrtRand::seeded(0x2545_f491));
        let plan = crate::notifies::plan_damage_sound(context, &config, || generator.roll());
        self.rand = Some(generator);
        plan
    }

    pub fn set_overlays(&mut self, overlays: impl Iterator<Item = (i32, [f32; 4])>) {
        self.overlays = overlays.filter(|(_, factor)| factor[3] > 0.0).collect();
    }

    pub fn forget_bone_scalers(&mut self, object_id: i32) {
        self.bone_scalers.remove(&object_id);
    }

    pub fn is_big_head(&self, object_id: i32, key: AppearanceKey, out: &mut f32) -> bool {
        let Some(character) = self.loaded.get(&key) else {
            return killer_assets::bone_scale::is_big_head(false, None, None, 0, out);
        };
        let master = &character.parts[character.master].1;
        killer_assets::bone_scale::is_big_head(true, master.match_ref_bone(HEAD_BONE), self.bone_scalers.get(&object_id), master.bones.len(), out)
    }

    pub fn master_mesh_name(&self, key: AppearanceKey) -> Option<String> {
        let character = self.loaded.get(&key)?;
        character.parts[character.master].1.lod.as_ref().map(|lod| lod.source.mesh.clone())
    }

    pub fn mesh_origin_z(&self, key: AppearanceKey) -> Option<f32> {
        let character = self.loaded.get(&key)?;
        Some(character.parts[character.master].1.origin.z)
    }

    pub fn draw_scale_of(&self, key: AppearanceKey) -> Option<f32> {
        self.loaded.get(&key).map(|character| character.draw_scale)
    }

    pub fn spawn_skill_effect_at_bone_index(&mut self, object_id: i32, key: AppearanceKey, class: &str, index: i32, modifiers: EffectModifiers) -> bool {
        let Some(character) = self.loaded.get(&key) else { return false };
        let master = &character.parts[character.master].1;
        let Some(bone) = usize::try_from(index).ok().filter(|bone| *bone < master.bones.len()) else { return false };
        self.attach_effect_to_bone(object_id, key, class, bone, Vec3::ZERO, modifiers)
    }

    pub fn spawn_attached_effect(&mut self, object_id: i32, key: AppearanceKey, class: &str, bone: &str, relative: Vec3, modifiers: EffectModifiers) -> bool {
        let Some(character) = self.loaded.get(&key) else { return false };
        let master = &character.parts[character.master].1;
        let Some(bone) = master.match_ref_bone(bone) else { return false };
        self.attach_effect_to_bone(object_id, key, class, bone, relative, modifiers)
    }

    fn attach_effect_to_bone(&mut self, object_id: i32, key: AppearanceKey, class: &str, bone: usize, relative: Vec3, modifiers: EffectModifiers) -> bool {
        let handle = self.next_transient;
        self.next_transient += 1;
        self.transients.push(Transient { handle, object_id, key, bone: Some(bone), yaw: 0, relative, follow: None, ribbon: false, life: None });
        self.transient_spawns.push(TransientSpawn { handle, class: class.to_owned(), stick_velocity: None, modifiers, trailer: None });
        true
    }

    pub fn spawn_notify_effect(&mut self, object_id: i32, key: AppearanceKey, spawn: crate::notifies::EffectSpawn, zone: u8) {
        let modifiers = EffectModifiers { size_scale: Some(spawn.size_scale), ..EffectModifiers::default() };
        if let Some(attach) = &spawn.attach
            && self.spawn_attached_effect(object_id, key, &spawn.class, &attach.bone, attach.relative, modifiers)
        {
            return;
        }
        self.world_spawns.push(WorldSpawn {
            class: spawn.class,
            location: spawn.location,
            rotation: spawn.rotation,
            zone,
            modifiers,
            owner: Some(OwnerSpawn { object_id, key, relative: false, pre_pivot: Vec3::ZERO }),
            projectile: None,
            trailer_physics: None,
        });
    }

    pub fn follow_owner(&mut self, object_id: i32, key: AppearanceKey, follow: OwnerFollow) -> u64 {
        let handle = self.next_transient;
        self.next_transient += 1;
        self.transients.push(Transient { handle, object_id, key, bone: None, yaw: 0, relative: Vec3::ZERO, follow: Some(follow), ribbon: false, life: None });
        handle
    }

    pub fn take_transient_spawns(&mut self) -> Vec<TransientSpawn> {
        std::mem::take(&mut self.transient_spawns)
    }

    pub fn take_world_spawns(&mut self) -> Vec<WorldSpawn> {
        std::mem::take(&mut self.world_spawns)
    }

    pub fn take_transient_frames(&mut self) -> Vec<TransientFrame> {
        std::mem::take(&mut self.transient_frames)
    }

    pub fn transient_alive(&self, handle: u64, exists: impl Fn(i32) -> bool) -> bool {
        self.transients.iter().any(|t| t.handle == handle && exists(t.object_id) && t.life.is_none_or(|life| life > 0.0))
    }

    pub fn set_keep_min_frame_rate(&mut self, keep: bool) {
        self.keep_min_frame_rate = keep;
    }

    pub fn attach_trailer(&mut self, object_id: i32, key: AppearanceKey, collision: (f32, f32), (points, time, right): (i32, f32, bool), shot_time: f32) -> bool {
        let Some(character) = self.loaded.get(&key) else { return false };
        let slot = if right { SLOT_RIGHT_HAND } else { SLOT_LEFT_HAND };
        let Some(length) = character.parts.iter().find(|(_, mesh)| mesh.slot == slot && mesh.placement != Placement::Skinned).and_then(|(_, mesh)| mesh.floats_150.first().copied()) else { return false };
        let scale = weapon_scale(collision.0, collision.1, 0.0);
        let Some(trailer) = killer_assets::ribbon_point::trailer(length, scale, shot_time, points, time) else { return false };
        let name = if right { killer_assets::pawn::RIGHT_HAND_BONE } else { killer_assets::pawn::LEFT_HAND_BONE };
        let Some(bone) = character.parts[character.master].1.match_ref_bone(name) else { return false };
        let handle = self.next_transient;
        self.next_transient += 1;
        log::debug!("trilha da arma em {object_id}: largura {} vida {}", trailer.width, trailer.life);
        self.transients.push(Transient { handle, object_id, key, bone: Some(bone), yaw: 0, relative: Vec3::ZERO, follow: None, ribbon: true, life: Some(trailer.life) });
        self.transient_spawns.push(TransientSpawn {
            handle,
            class: killer_assets::ribbon_point::TRAILER_CLASS.to_owned(),
            stick_velocity: None,
            modifiers: EffectModifiers::default(),
            trailer: Some((trailer, name)),
        });
        true
    }

    pub fn pawn_lights_mut(&mut self, object_id: i32) -> &mut Vec<killer_assets::pawn_light::PawnLight> {
        self.pawn_lights.entry(object_id).or_default()
    }

    pub fn tick_pawn_lights(&mut self, dt: f32, exists: impl Fn(i32) -> bool) {
        self.pawn_lights.retain(|object_id, lights| {
            if !exists(*object_id) {
                return false;
            }
            killer_assets::pawn_light::tick(lights, dt);
            !lights.is_empty()
        });
    }

    pub fn tick_damage_acts(&mut self, dt: f32, key_of: impl Fn(i32) -> Option<AppearanceKey>) {
        let keep_min_frame_rate = self.keep_min_frame_rate;
        let loaded = &self.loaded;
        self.damage_acts.retain(|object_id, act| {
            let Some(key) = key_of(*object_id) else { return false };
            if loaded.get(&key).is_some_and(|character| character.damage_bones.ticks(keep_min_frame_rate)) {
                act.spine_rotation(dt);
            }
            act.active
        });
    }

    pub fn attacked(&mut self, object_id: i32) {
        self.damage_acts.entry(object_id).or_default().attacked();
    }

    pub fn age_transients(&mut self, dt: f32) {
        for life in self.transients.iter_mut().filter_map(|t| t.life.as_mut()) {
            *life -= dt;
        }
    }

    pub fn forget_transient(&mut self, handle: u64) {
        self.transients.retain(|t| t.handle != handle);
    }

    pub fn loaded_mut(&mut self, key: AppearanceKey) -> Option<&mut LoadedCharacter> {
        self.loaded.get_mut(&key)
    }

    pub fn get(&self, key: AppearanceKey) -> Option<&LoadedCharacter> {
        self.loaded.get(&key)
    }

    fn want(&mut self, key: AppearanceKey) {
        if !self.pending.contains(&key) && !self.failed.contains(&key) {
            self.pending.insert(key);
            self.wanted.push(key);
        }
    }

    pub fn take_wanted(&mut self, limit: usize) -> Vec<AppearanceKey> {
        self.wanted.drain(..self.wanted.len().min(limit)).collect()
    }

    pub fn insert(&mut self, key: AppearanceKey, character: LoadedCharacter) {
        self.pending.remove(&key);
        self.loaded.insert(key, character);
    }

    pub fn fail(&mut self, key: AppearanceKey) {
        self.pending.remove(&key);
        self.failed.insert(key);
    }

    pub fn is_pending(&self, key: AppearanceKey) -> bool {
        self.pending.contains(&key)
    }

    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty() || !self.wanted.is_empty()
    }

    pub fn retry_pending(&mut self) -> Vec<AppearanceKey> {
        let mut retried = Vec::new();
        for key in std::mem::take(&mut self.pending) {
            if self.retried.insert(key) {
                self.wanted.push(key);
                self.pending.insert(key);
                retried.push(key);
            } else {
                self.failed.insert(key);
            }
        }
        retried
    }

    fn unknown_appearance(&mut self, object_id: i32) -> bool {
        self.unknown.insert(object_id)
    }

    pub fn forget_poses(&mut self, world: &World) {
        self.poses
            .retain(|id, _| world.entities.contains_key(id) || world.me.as_ref().is_some_and(|me| me.object_id == *id));
        self.lods
            .retain(|id, _| world.entities.contains_key(id) || world.me.as_ref().is_some_and(|me| me.object_id == *id));
        self.mounts
            .retain(|id, _| world.entities.contains_key(id) || world.me.as_ref().is_some_and(|me| me.object_id == *id));
        self.mount_poses.retain(|id, _| self.mounts.contains_key(id));
        let present = |id: &i32| world.entities.contains_key(id) || world.me.as_ref().is_some_and(|me| me.object_id == *id);
        self.effect_targets.retain(|id, _| present(id));
        self.hand_positions.retain(|id, _| present(id));
        self.rod_points.retain(|id, _| present(id));
        self.instance_lighting.retain(|id, _| present(id));
        self.bone_scalers.retain(|id, _| present(id));
        self.foot_volumes.retain(|id, _| present(id));
        self.swim_ticks.retain(|id, _| present(id));
        self.unknown.retain(|id| present(id));
    }

    pub fn drain(&mut self) -> Vec<CharacterId> {
        self.pending.clear();
        self.failed.clear();
        self.wanted.clear();
        self.retried.clear();
        self.unknown.clear();
        self.poses.clear();
        self.mounts.clear();
        self.mount_poses.clear();
        self.lods.clear();
        self.loaded.drain().flat_map(|(_, c)| c.parts.into_iter().map(|(id, _)| id)).collect()
    }

    #[cfg(test)]
    pub fn retained_sizes(&self) -> String {
        format!(
            "loaded {} poses {} lods {} effect_targets {} hand {} lighting {} foot {} unknown {} scalers {}",
            self.loaded.len(), self.poses.len(), self.lods.len(), self.effect_targets.len(), self.hand_positions.len(),
            self.instance_lighting.len(), self.foot_volumes.len(), self.unknown.len(), self.bone_scalers.len()
        )
    }

    pub fn loaded_count(&self) -> usize {
        self.loaded.len()
    }

    pub fn evict_drawn(&mut self, world: &World, drawn: &HashSet<i32>) -> Vec<CharacterId> {
        let me_id = world.me.as_ref().map(|me| me.object_id);
        let keep: HashSet<AppearanceKey> = world
            .me
            .iter()
            .chain(world.entities.values())
            .filter(|entity| me_id == Some(entity.object_id) || drawn.contains(&entity.object_id))
            .flat_map(|entity| entity_key(entity).into_iter().chain(mount_key(entity)))
            .collect();
        self.wanted.retain(|key| keep.contains(key));
        self.pending.retain(|key| keep.contains(key));
        self.failed.retain(|key| keep.contains(key));
        self.retried.retain(|key| keep.contains(key));
        let stale: Vec<_> = self.loaded.keys().filter(|key| !keep.contains(key)).copied().collect();
        stale.into_iter().filter_map(|key| self.loaded.remove(&key)).flat_map(|character| character.parts.into_iter().map(|(id, _)| id)).collect()
    }
}

pub fn world_matrix(position: Vec3, heading: i32, draw_scale: f32, local: Mat4) -> Mat4 {
    Mat4::from_translation(position)
        * Mat4::from_rotation_z(heading as f32 * TAU / HEADING_UNITS_PER_TURN)
        * Mat4::from_scale(Vec3::splat(draw_scale))
        * local
}

#[cfg(test)]
pub fn instances(
    world: &mut World,
    cache: &mut CharacterCache,
    tables: &DatTables,
    frame: &Frame,
    zone_of: impl Fn(Vec3) -> u8,
    lights_of: impl Fn(Query) -> Sample,
    visible: impl Fn(Vec3, f32) -> bool,
) -> Vec<CharacterInstance> {
    instances_with(world, cache, tables, frame, zone_of, lights_of, visible, |_| None)
}

pub const LOBBY_WAIT_TWEEN: f32 = 0.2;
pub const LOBBY_SOCIAL_TWEEN: f32 = 0.3;

pub fn lobby_wait(channel: &mut Channel, wait: &str, social: Option<&str>, tables: &dyn Sequences) {
    let playing = |name: &str| channel.sequence.as_deref().is_some_and(|current| current.eq_ignore_ascii_case(name));
    if let Some(social) = social
        && !playing(social)
        && let Some(seq) = tables.seq(social)
    {
        channel.play(social, &seq, 1.0, LOBBY_SOCIAL_TWEEN, false);
        return;
    }
    if playing(wait) || (!channel.looping && channel.is_animating() && !channel.finished) {
        return;
    }
    if let Some(seq) = tables.seq(wait) {
        channel.play(wait, &seq, 1.0, LOBBY_WAIT_TWEEN, true);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn instances_with(
    world: &mut World,
    cache: &mut CharacterCache,
    tables: &DatTables,
    frame: &Frame,
    zone_of: impl Fn(Vec3) -> u8,
    lights_of: impl Fn(Query) -> Sample,
    visible: impl Fn(Vec3, f32) -> bool,
    lobby_wait_of: impl Fn(i32) -> Option<(&'static str, Option<&'static str>)>,
) -> Vec<CharacterInstance> {
    let drawn: HashSet<i32> = drawn_pawns(world, tables, frame).into_iter().collect();
    instances_drawn(world, cache, tables, frame, zone_of, lights_of, visible, lobby_wait_of, &drawn)
}

#[allow(clippy::too_many_arguments)]
pub fn instances_drawn(
    world: &mut World,
    cache: &mut CharacterCache,
    tables: &DatTables,
    frame: &Frame,
    zone_of: impl Fn(Vec3) -> u8,
    lights_of: impl Fn(Query) -> Sample,
    visible: impl Fn(Vec3, f32) -> bool,
    lobby_wait_of: impl Fn(i32) -> Option<(&'static str, Option<&'static str>)>,
    drawn: &HashSet<i32>,
) -> Vec<CharacterInstance> {
    let dt = frame.dt;
    cache.age_transients(dt);
    cache.tick_pawn_lights(dt, |object_id| world.me.as_ref().is_some_and(|me| me.object_id == object_id) || world.entities.contains_key(&object_id));
    cache.tick_damage_acts(dt, |object_id| world.entity(object_id).and_then(entity_key));
    for object_id in world.effects.take_sweepable_deaths() {
        let Some(entity) = world.entity(object_id) else { continue };
        let pawn = killer_assets::pawn_light::Actor { location: entity.position.to_array(), rotation: [0, entity.heading, 0] };
        let height = entity.collision_height;
        killer_assets::pawn_light::add_sweep_light(cache.pawn_lights_mut(object_id), pawn, height);
    }
    let bone_smokes = std::mem::take(&mut cache.bone_smokes);
    let me_id = world.me.as_ref().map(|me| me.object_id);
    let mut out = Vec::new();
    cache.effect_frames.clear();
    cache.transient_frames.clear();
    let mut shots = Vec::new();
    let mut hits = Vec::new();
    let mut casts = Vec::new();
    let mut magic_shots = Vec::new();
    let mut pre_shots = Vec::new();
    let mut magic_stops = Vec::new();
    let mut notifies: Vec<(i32, AppearanceKey, killer_world::anim::NotifyKind)> = Vec::new();
    let mut mount_wants: Vec<AppearanceKey> = Vec::new();
    for entity in world.me.iter_mut().chain(world.entities.values_mut()) {
        let Some(key) = entity_key(entity) else {
            if !matches!(entity.kind, EntityKind::Item { .. }) && cache.unknown_appearance(entity.object_id) {
                log::warn!("entidade {} sem aparência: {:?}", entity.object_id, entity.kind);
            }
            continue;
        };
        if me_id != Some(entity.object_id) && !drawn.contains(&entity.object_id) {
            continue;
        }
        let npc = npc_record(tables, &entity.kind);
        let set = anim_section(&entity.kind, npc.map(|n| n.class.as_str())).and_then(|s| tables.anim_names.get(&s));
        if !visible(entity.position, (entity.collision_radius + entity.collision_height) * BOUNDS_SCALE) {
            continue;
        }
        killer_world::anim::ride::sync(&mut cache.mounts, entity.object_id, entity.mount, &mut entity.anim);
        let mount_key = mount_key(entity);
        let (rider_slot, mut mount_slot) = match mount_key {
            Some(mount_key) if mount_key != key => {
                let [rider, mount] = cache.loaded.get_disjoint_mut([&key, &mount_key]);
                (rider, mount)
            }
            _ => (cache.loaded.get_mut(&key), None),
        };
        let Some(character) = rider_slot else {
            cache.want(key);
            continue;
        };
        if let Some(mount_key) = mount_key
            && mount_slot.is_none()
        {
            mount_wants.push(mount_key);
        }
        let animation = character.animation.clone();
        synchronize_lods(cache.lods.entry(entity.object_id).or_default(), character.parts.iter().filter_map(|(_, mesh)| {
            mesh.lod.as_ref().map(|lod| (mesh.slot, lod.source.mesh.as_str()))
        }));
        let anim_tables = AnimTables { animation: animation.as_deref(), tables };
        entity.anim.npc_speed = npc.map_or(0.0, |n| n.speed);
        let weapon = character.weapon_type;
        let no_section = AnimSet::default();
        if me_id == Some(entity.object_id)
            && let Some(set) = set
        {
            // Atraso dos comandos de macro de ação: é a duração da animação social que a ação
            // toca (`APawn::GetSocialAnimDuration` no original, ver `MACRO_ACTION_SOCIAL_ANIM`).
            let view = entity.pawn_view(weapon);
            let seconds = |name: &str| anim_tables.seq(name).map(|seq| seq.seconds());
            cache.social_durations =
                (0..MACRO_SOCIAL_ANIMS).map(|id| social_anim_duration(seconds, set, &view, id)).collect();
        }
        if set.is_none()
            && entity.anim.channel.sequence.is_none()
            && let Some(seq) = anim_tables.seq(DECO_WAIT)
        {
            entity.anim.channel.play(DECO_WAIT, &seq, 1.0, 0.0, true);
        }
        let mut entity_notifies = Vec::new();
        match lobby_wait_of(entity.object_id) {
            Some((wait, social)) => {
                let mut view = entity.pawn_view(weapon);
                view.same_anim = true;
                let moving = entity.dest.is_some();
                let yaw = entity.heading as f32;
                entity.anim.tick(dt, &view, set.unwrap_or(&no_section), &anim_tables, moving, yaw);
                lobby_wait(&mut entity.anim.channel, wait, social, &anim_tables);
            }
            None => {
                for event in entity.tick_anim(dt, weapon, set.unwrap_or(&no_section), &anim_tables) {
                    log_anim_event(entity, &event);
                    match event {
                        AnimEvent::TransientEffect { skill_id, .. } => shots.push((entity.object_id, key, skill_id)),
                        AnimEvent::AttackShot(hit) => {
                            log::debug!("{} ({}): golpe corpo a corpo em {}, arma {:?}", entity.name, entity.object_id, hit.target, killer_world::anim::equipped(&entity.kind));
                            hits.push((entity.object_id, hit, HitKind::Melee))
                        }
                        AnimEvent::ArrowReleased(hit) => {
                            log::debug!("{} ({}): flecha solta em {}", entity.name, entity.object_id, hit.target);
                            hits.push((entity.object_id, hit, HitKind::Ranged))
                        }
                        AnimEvent::CastStart { skill_id, level, target, shot_time } => casts.push((entity.object_id, key, skill_id, level, target, shot_time)),
                        AnimEvent::MagicShot { skill_id, level, target, shot_time, associated } => magic_shots.push((entity.object_id, key, skill_id, level, target, shot_time, associated)),
                        AnimEvent::MagicPreShot { skill_id, target, shot_time, .. } => pre_shots.push((entity.object_id, key, skill_id, target, shot_time)),
                        AnimEvent::MagicStop => magic_stops.push(entity.object_id),
                        AnimEvent::Notify(kind) => entity_notifies.push(kind),
                        _ => {}
                    }
                }
                if me_id == Some(entity.object_id) {
                    let wait = killer_world::anim::names::cur_wait_anim_name(set.unwrap_or(&no_section), &entity.pawn_view(weapon));
                    cache.me_waiting = wait.is_some() && entity.anim.channel.sequence.as_deref() == wait;
                }
            }
        }
        let mut place = entity.position;
        let mut mount_pose = None;
        if let (Some(state), Some(mount_character)) = (cache.mounts.get_mut(&entity.object_id), mount_slot.as_deref_mut()) {
            let mount_kind = EntityKind::Npc { npc_id: killer_world::anim::ride::mount_npc_id(state.mount.ride_class_id), rhand: 0, chest: 0, lhand: 0, attackable: false };
            let mount_npc = npc_record(tables, &mount_kind);
            let mount_set = anim_section(&mount_kind, mount_npc.map(|n| n.class.as_str())).and_then(|s| tables.anim_names.get(&s)).unwrap_or(&no_section);
            let mount_tables = AnimTables { animation: mount_character.animation.as_deref(), tables };
            let rider_view = entity.pawn_view(weapon);
            if let Some(kind) = entity.anim.ride_attack.take() {
                state.attack(kind, &rider_view, mount_set, &mount_tables);
            }
            state.tick(dt, &rider_view, mount_npc.map_or(0.0, |n| n.speed), mount_set, &mount_tables, (entity.dest.is_some(), entity.heading as f32));
            let pose = mount_character.scaled_master_pose(&state.player.channel, cache.mount_poses.entry(entity.object_id).or_default(), None, None);
            if let Some(bone) = mount_character.parts[mount_character.master].1.match_ref_bone(killer_world::anim::ride::ATTACH_BONE)
                && let Some(attach) = mount_character.bone_world(bone, &pose, entity.position, entity.heading)
            {
                let pre_pivot = character.parts[character.master].1.origin.z * character.draw_scale;
                place = attach.transform_point3(Vec3::ZERO) - Vec3::Z * pre_pivot;
            }
            mount_pose = Some(pose);
        }
        let mut effect_notifies = Vec::new();
        for kind in entity_notifies {
            match kind {
                killer_world::anim::NotifyKind::Effect(effect) => effect_notifies.push(effect),
                other => notifies.push((entity.object_id, key, other)),
            }
        }
        if !visible(entity.position, character.bounding_radius(entity.collision_radius, entity.collision_height)) {
            continue;
        }
        let keep_min_frame_rate = cache.keep_min_frame_rate;
        let damage_yaw = cache.damage_acts.get(&entity.object_id).filter(|act| act.active && !keep_min_frame_rate).map(|act| act.yaw());
        let memory = cache.poses.entry(entity.object_id).or_default();
        let master_pose = character.scaled_master_pose(&entity.anim.channel, memory, cache.bone_scalers.get(&entity.object_id), damage_yaw);
        if let Some(bone) = character.bone_world(EFFECT_TARGET_BONE, &master_pose, place, entity.heading) {
            cache.effect_targets.insert(entity.object_id, bone.transform_point3(Vec3::ZERO));
        }
        if let Some(hand) = character.parts[character.master].1.match_ref_bone(RIGHT_HAND_BONE)
            && let Some(bone) = character.bone_world(hand, &master_pose, place, entity.heading)
        {
            cache.hand_positions.insert(entity.object_id, bone.transform_point3(Vec3::ZERO));
        }
        for smoke in bone_smokes.iter().filter(|smoke| smoke.object_id == entity.object_id) {
            let Some(bone) = character.parts[character.master].1.match_ref_bone(smoke.bone) else { continue };
            let Some(world) = character.bone_world(bone, &master_pose, place, entity.heading) else { continue };
            let origin = world.transform_point3(Vec3::ZERO);
            let point = Vec3::new(origin.x, origin.y, smoke.z);
            let random = if killer_assets::skill_smoke::shake_draws_random(smoke.skill_id) {
                cache.rand.get_or_insert(crate::world_sound::CrtRand::seeded(0x2545_f491)).fraction()
            } else {
                0.0
            };
            cache.explosion_shakes.extend(killer_assets::skill_smoke::smoke_shake(smoke.skill_id, point, random).as_ref().map(smoke_view_shake));
            cache.event_smokes.push((point, smoke.radius));
        }
        for effect in &effect_notifies {
            let pawn = crate::notifies::EffectPawn { position: entity.position, rotation: [0, entity.heading, 0], draw_scale: character.draw_scale };
            let mesh = crate::notifies::EffectMesh::Skeletal(effect.bone.as_deref().and_then(|name| {
                let bone = character.parts[character.master].1.match_ref_bone(name)?;
                let world = character.bone_world(bone, &master_pose, place, entity.heading)?;
                let axes = Mat3::from_mat4(world);
                Some(crate::notifies::BoneFrame {
                    origin: world.transform_point3(Vec3::ZERO),
                    axes: [axes.x_axis.normalize_or_zero(), axes.y_axis.normalize_or_zero(), axes.z_axis.normalize_or_zero()],
                })
            }));
            match crate::notifies::plan_effect(effect, &pawn, mesh, true, zone_of(entity.position) == 0) {
                Some(crate::notifies::EffectPlan::Spawn(spawn)) => cache.notify_effects.push((entity.object_id, key, spawn, zone_of(entity.position))),
                Some(crate::notifies::EffectPlan::Camera { class, .. }) => cache.camera_effects.push(class),
                None => {}
            }
        }
        let hide = entity.anim.hide_weapons;
        let zone = zone_of(entity.position);
        if let Some(tick) = cache.swim_ticks.remove(&entity.object_id) {
            let mut generator = cache.rand.unwrap_or(crate::world_sound::CrtRand::seeded(0x2545_f491));
            let master = &character.parts[character.master].1;
            let plans = swim_bubbles(&mut generator, tick, master.bones.len(), master.match_ref_bone(SWIM_BREATH_BONE));
            cache.rand = Some(generator);
            for (bone, class) in plans {
                let handle = cache.next_transient;
                cache.next_transient += 1;
                cache.transients.push(Transient { handle, object_id: entity.object_id, key, bone: Some(bone), yaw: 0, relative: Vec3::ZERO, follow: None, ribbon: false, life: None });
                cache.transient_spawns.push(TransientSpawn { handle, class: class.to_owned(), stick_velocity: None, modifiers: EffectModifiers::default(), trailer: None });
            }
        }
        let (light_center, light_radius) = character.light_sphere(entity.position, entity.heading);
        let lighting = lights_of(Query { position: entity.position, center: light_center, radius: light_radius, bounds: character.light_bounds(entity.position, entity.heading) });
        let pawn_actor = killer_assets::pawn_light::Actor { location: entity.position.to_array(), rotation: [0, entity.heading, 0] };
        let trailers = &cache.trailer_classes;
        let pawn_lights: Vec<_> = killer_assets::pawn_light::hardware_lights_with(cache.pawn_lights.get(&entity.object_id).map_or(&[], Vec::as_slice), 0, move |light| {
            Some(light_actor(trailers, light, pawn_actor))
        })
        .collect();
        let collision = Collision { radius: entity.collision_radius, height: entity.collision_height };
        let scale = weapon_scale(collision.radius, collision.height, 0.0);
        for (index, effect) in character.effects.iter().enumerate() {
            let Some((location, rotation)) = character.effect_placement(index, &master_pose, place, entity.heading, scale) else { continue };
            cache.effect_frames.push(EffectFrame {
                object_id: entity.object_id,
                key,
                index,
                location,
                rotation,
                zone,
                visible: match &effect.effect {
                    PawnEffect::Weapon(weapon) => !(if weapon.right { hide.right } else { hide.left }),
                    PawnEffect::HeroAura => true,
                },
                weapon_scale: scale,
                collision_radius: collision.radius,
            });
        }
        for transient in cache.transients.iter().filter(|t| t.object_id == entity.object_id && t.key == key) {
            let Some(bone) = transient.bone else {
                if let Some(follow) = transient.follow {
                    cache.transient_frames.push(TransientFrame {
                        handle: transient.handle,
                        location: entity.position + follow.pre_pivot.unwrap_or(Vec3::ZERO),
                        rotation: follow.rotation.unwrap_or([0, entity.heading, 0]),
                        zone,
                        ribbon: None,
                    });
                }
                continue;
            };
            let Some(world) = character.bone_world(bone, &master_pose, place, entity.heading) else { continue };
            let axes = Mat3::from_mat4(world);
            let axes = Mat3::from_cols(axes.x_axis.normalize_or_zero(), axes.y_axis.normalize_or_zero(), axes.z_axis.normalize_or_zero());
            let ribbon = transient.ribbon.then(|| {
                let bone = killer_assets::ribbon_point::Bone { origin: world.w_axis.truncate().to_array(), rotation: ortho_rotation(axes) };
                killer_render::renderer::RibbonInput {
                    start: Some(bone),
                    end: Some(bone),
                    owner: Some(killer_assets::ribbon_point::Actor { location: entity.position.to_array(), rotation: [0, entity.heading, 0] }),
                    owner_mesh: true,
                    instance: true,
                }
            });
            cache.transient_frames.push(TransientFrame {
                handle: transient.handle,
                location: world.transform_point3(transient.relative),
                rotation: ortho_rotation(axes * rotator_matrix([0, transient.yaw, 0])),
                zone,
                ribbon,
            });
        }
        for part in 0..character.parts.len() {
            if character.hidden(part, hide) {
                continue;
            }
            let Some(palette) = character.palette(part, &master_pose, collision) else { continue };
            let mesh_part = &character.parts[part].1;
            if mesh_part.slot == killer_assets::pawn::SLOT_RIGHT_HAND
                && let (Some(skin), Some(bind)) = (palette.get(killer_assets::fishing_float::LINE_ROD_BONE), mesh_part.bind.get(killer_assets::fishing_float::LINE_ROD_BONE))
            {
                cache.rod_points.insert(entity.object_id, (character.world(part, place, entity.heading) * *skin * *bind).transform_point3(Vec3::ZERO));
            } else if mesh_part.slot == killer_assets::pawn::SLOT_RIGHT_HAND {
                cache.rod_points.remove(&entity.object_id);
            }
            if part == character.master {
                cache.instance_lighting.insert(entity.object_id, InstanceLighting { light_sphere: light_center.extend(light_radius).to_array(), zone, lights: lighting.lights.clone(), pawn_lights: pawn_lights.clone(), ambient: lighting.ambient, sun_affect: lighting.sun_affect });
            }
            out.push(CharacterInstance {
                overlay: cache.overlays.get(&entity.object_id).copied().unwrap_or([0.0; 4]),
                actor_id: entity.object_id,
                actor_position: entity.position,
                collision_height: entity.collision_height,
                shadow_caster: part == character.master,
                light_sphere: light_center.extend(light_radius).to_array(),
                mesh: character.parts[part].0,
                world: character.world(part, place, entity.heading),
                palette,
                zone,
                lights: lighting.lights.clone(),
                pawn_lights: pawn_lights.clone(),
                ambient: lighting.ambient,
                sun_affect: lighting.sun_affect,
            });
        }
        if let (Some(pose), Some(mount_character)) = (mount_pose.as_ref(), mount_slot.as_deref()) {
            let (mount_center, mount_radius) = mount_character.light_sphere(entity.position, entity.heading);
            let mount_lighting = lights_of(Query { position: entity.position, center: mount_center, radius: mount_radius, bounds: mount_character.light_bounds(entity.position, entity.heading) });
            for part in 0..mount_character.parts.len() {
                if mount_character.hidden(part, HideWeapons::default()) {
                    continue;
                }
                let Some(palette) = mount_character.palette(part, pose, collision) else { continue };
                out.push(CharacterInstance {
                    overlay: cache.overlays.get(&entity.object_id).copied().unwrap_or([0.0; 4]),
                    actor_id: entity.object_id,
                    actor_position: entity.position,
                    collision_height: entity.collision_height,
                    shadow_caster: part == mount_character.master,
                    light_sphere: mount_center.extend(mount_radius).to_array(),
                    mesh: mount_character.parts[part].0,
                    world: mount_character.world(part, entity.position, entity.heading),
                    palette,
                    zone,
                    lights: mount_lighting.lights.clone(),
                    pawn_lights: pawn_lights.clone(),
                    ambient: mount_lighting.ambient,
                    sun_affect: mount_lighting.sun_affect,
                });
            }
        }
    }
    for key in mount_wants {
        cache.want(key);
    }
    for object_id in magic_stops {
        if let Some(lights) = cache.pawn_lights.get_mut(&object_id) {
            killer_assets::pawn_light::remove_kind(lights, killer_assets::pawn_light::KIND_SKILL);
        }
    }
    for (object_id, key, skill_id) in shots {
        let spawned = cache.transient_spawns.len();
        cache.spawn_shot_effects(object_id, key, skill_id, &tables.soulshot);
        if cache.transient_spawns.len() > spawned
            && let Some(entity) = world.me.iter().chain(world.entities.values()).find(|entity| entity.object_id == object_id)
        {
            cache.sound_plans.extend(crate::skill_sound::transient_plans(entity, tables, skill_id));
        }
    }
    for (object_id, _, kind) in notifies {
        let Some(entity) = world.me.iter().chain(world.entities.values()).find(|entity| entity.object_id == object_id) else { continue };
        let target = match &entity.anim.action {
            killer_world::anim::Action::Attack(attack) => attack.hit.target,
            killer_world::anim::Action::Magic(magic) => magic.target,
            killer_world::anim::Action::None => 0,
        };
        let row = match &entity.kind {
            EntityKind::Player { class_id, race, sex, .. } => l2_dat::chargrp::index_for(*race, *sex, *class_id),
            _ => None,
        };
        match kind {
            killer_world::anim::NotifyKind::Sound(sound) if sound.sound.is_some() => {
                let roll = cache.rand();
                if let Some(plan) = crate::notifies::plan_sound(&sound, object_id, entity.position, roll) {
                    cache.sound_plans.push(plan);
                }
            }
            killer_world::anim::NotifyKind::Sound(sound) => {
                if cache.rand() % 100 >= sound.random {
                    continue;
                }
                let items = tables.items();
                let boots_id = l2_dat::equipment::item_class_id(&items, &killer_world::anim::equipped(&entity.kind), l2_dat::SubMesh::Boots);
                let boots = u32::try_from(boots_id).ok().and_then(|id| tables.armorgrp.get(&id)).map_or(&[][..], |armor| armor.boots_sounds.as_slice());
                let context = crate::notifies::FootstepContext {
                    object_id,
                    position: entity.position,
                    npc: matches!(entity.kind, EntityKind::Npc { .. }),
                    running: entity.running,
                    boots,
                    terrain: zone_of(entity.position) == 0,
                    grass: cache.foot_volumes.get(&object_id).is_some_and(|foot| foot.grass),
                    water: cache.foot_volumes.get(&object_id).is_some_and(|foot| foot.water),
                    step_volume: cache.foot_volumes.get(&object_id).is_some_and(|foot| foot.step),
                    step_names: cache.foot_volumes.get(&object_id).and_then(|foot| crate::walk_world::step_sound_entry(&tables.musicinfo, foot.step_sound_id)),
                };
                let config = cache.sound_config.clone();
                let mut generator = cache.rand.unwrap_or(crate::world_sound::CrtRand::seeded(0x2545_f491));
                let plans = crate::notifies::plan_footsteps(&sound, &context, &config, || generator.roll());
                cache.rand = Some(generator);
                cache.sound_plans.extend(plans);
            }
            killer_world::anim::NotifyKind::SwimSound => {
                let context = crate::notifies::SwimContext { object_id, position: entity.position, in_water_volume: cache.foot_volumes.get(&object_id).is_some_and(|foot| foot.water), afloat: false, moving_in_water: false };
                let config = cache.sound_config.clone();
                let mut generator = cache.rand.unwrap_or(crate::world_sound::CrtRand::seeded(0x2545_f491));
                let plan = crate::notifies::plan_swim(&context, &config, || generator.roll());
                cache.rand = Some(generator);
                if let Some(plan) = plan {
                    cache.sound_plans.push(plan);
                }
            }
            killer_world::anim::NotifyKind::ScreenFade(fade) => cache.screen_fades.push(fade),
            killer_world::anim::NotifyKind::ViewShake(shake) => {
                let randoms = (cache.fraction(), cache.fraction());
                let smoke = crate::notifies::smoke_range(entity.anim.npc_class_id, &shake);
                cache.view_shakes.push(ViewShakePlan { notify: crate::notifies::shake_notify(&shake), location: entity.position, randoms, smoke });
            }
            killer_world::anim::NotifyKind::AttackVoice(voice) => {
                let context = crate::notifies::VoiceContext {
                    object_id,
                    position: entity.position,
                    npc: matches!(entity.kind, EntityKind::Npc { .. }),
                    has_target: target != 0,
                    weapon_type: killer_world::anim::weapon_type(&entity.kind, &tables.items()),
                    voices: row.and_then(|row| tables.chargrp.get(row)).map(|template| &template.voice_sounds),
                };
                let (gate, pick) = (cache.rand(), cache.rand());
                if let Some(plan) = crate::notifies::plan_attack_voice(&voice, &context, gate, pick) {
                    cache.sound_plans.push(plan);
                }
            }
            killer_world::anim::NotifyKind::AttackItem => {
                let equipped = killer_world::anim::equipped(&entity.kind);
                let weapon_id = if equipped.rhand > 0 { equipped.rhand } else { equipped.lrhand };
                let weapon_sounds = u32::try_from(weapon_id).ok().and_then(|id| tables.weapongrp.get(&id)).map_or(&[][..], |weapon| weapon.attack_sounds.as_slice());
                let context = crate::notifies::ItemContext {
                    object_id,
                    position: entity.position,
                    npc: matches!(entity.kind, EntityKind::Npc { .. }),
                    has_target: target != 0,
                    attacking: matches!(entity.anim.action, killer_world::anim::Action::Attack(_)),
                    weapon_sounds,
                    npc_record: npc_record(tables, &entity.kind),
                    char_sounds: row.and_then(|row| tables.chargrp.get(row)).map_or(&[][..], |template| template.attack_sounds.as_slice()),
                };
                let config = cache.sound_config.clone();
                let mut generator = cache.rand.unwrap_or(crate::world_sound::CrtRand::seeded(0x2545_f491));
                let plan = crate::notifies::plan_attack_item(&context, &config, || generator.roll());
                cache.rand = Some(generator);
                if let Some(plan) = plan {
                    cache.sound_plans.push(plan);
                }
            }
            _ => {}
        }
    }
    for (object_id, key, spawn, zone) in std::mem::take(&mut cache.notify_effects) {
        cache.spawn_notify_effect(object_id, key, spawn, zone);
    }
    if casts.iter().any(|cast| me_id == Some(cast.4)) {
        world.effects.camera_commands.push_back(killer_world::CameraCommand::Normal);
    }
    let find = |id: i32| world.me.iter().find(|me| me.object_id == id).or_else(|| world.entities.get(&id));
    for (object_id, key, skill_id, level, target, shot_time) in casts {
        run_skill_effects(cache, &find, killer_assets::skill_effects::Phase::Cast, object_id, key, skill_id, level, target, shot_time, &[], &zone_of, tables);
    }
    for (object_id, key, skill_id, target, shot_time) in pre_shots {
        run_skill_effects(cache, &find, killer_assets::skill_effects::Phase::PreShot, object_id, key, skill_id, 0, target, shot_time, &[], &zone_of, tables);
    }
    for (object_id, key, skill_id, level, target, shot_time, associated) in magic_shots {
        skill_hits(&find, object_id, skill_id, target, &associated, &mut hits);
        run_skill_effects(cache, &find, killer_assets::skill_effects::Phase::Shot, object_id, key, skill_id, level, target, shot_time, &associated, &zone_of, tables);
    }
    for arrival in std::mem::take(&mut cache.arrivals) {
        run_explosion(cache, &find, &arrival, &zone_of, tables);
    }
    for (victim_id, hit) in std::mem::take(&mut cache.arrow_impacts) {
        if let Some(victim) = find(victim_id).filter(|_| !hit.miss) {
            action_attack_sounds(cache, victim, is_pawn(&victim.kind), hit, tables);
        }
    }
    for (attacker_id, hit, kind) in hits {
        let ranged = kind == HitKind::Ranged;
        let (Some(attacker), Some(victim)) = (find(attacker_id), find(hit.target)) else { continue };
        if !hit.miss && matches!(kind, HitKind::Melee | HitKind::Strike { .. }) {
            action_attack_sounds(cache, victim, is_pawn(&victim.kind), hit, tables);
        }
        let silent = kind == HitKind::Strike { notify: false };
        let rendered = !silent && (me_id == Some(hit.target) || drawn.contains(&hit.target));
        let victim_pawn = is_pawn(&victim.kind);
        let damage_effect = entity_key(victim).and_then(|key| cache.get(key)).and_then(|character| character.damage_effect.clone());
        let context = HitContext { keep_min_frame_rate: silent || cache.keep_min_frame_rate, rendered, damage_effect: damage_effect.as_deref() };
        let effects = if ranged {
            attacked_notify_ranged_effects(attacker.position, victim.position, victim.collision_radius, hit, &context, &tables.soulshot)
        } else if hit.miss || !victim_pawn {
            Vec::new()
        } else {
            let pawn = |entity: &Entity| HitPawn { position: entity.position, rotation: [0, entity.heading, 0], radius: entity.collision_radius, height: entity.collision_height };
            cache.attacked(hit.target);
            associate_attacked_notify_effects(Some(&pawn(attacker)), &pawn(victim), hit, &context, &tables.soulshot)
        };
        let spawns: Vec<WorldSpawn> = effects
            .into_iter()
            .map(|(class, location, rotation)| WorldSpawn {
                class,
                location,
                rotation,
                zone: zone_of(victim.position),
                modifiers: EffectModifiers::default(),
                owner: None,
                projectile: None,
                trailer_physics: None,
            })
            .collect();
        if ranged {
            let start = cache.hand_position(attacker_id).unwrap_or(attacker.position + Vec3::Z * attacker.collision_height);
            cache.arrow_spawns.push(ArrowSpawn {
                model: arrow_model(world, attacker, tables),
                attacker: attacker_id,
                victim: hit.target,
                start,
                scale: arrow_scale(attacker.collision_radius, attacker.collision_height),
                effects: spawns,
                light: !hit.miss,
                hit,
                victim_pawn,
                curvature: l2_dat::equipment::weapon_curvature(&tables.items(), &killer_world::anim::equipped(&attacker.kind)).map_or(0, |value| value as i32),
            });
        } else {
            cache.world_spawns.extend(spawns);
            if !hit.miss && !hit.shield && is_pawn(&victim.kind) && kind != HitKind::Notify {
                add_hit_light(cache.pawn_lights_mut(hit.target), attacker.position, victim.position, victim.collision_radius);
            }
        }
    }
    cache.forget_poses(world);
    update_skill_aim(world);
    out
}

fn log_anim_event(entity: &Entity, event: &AnimEvent) {
    match event {
        AnimEvent::AttackShot(hit) => {
            log::info!("{} ({}): golpe em {} por {}", entity.name, entity.object_id, hit.target, hit.damage)
        }
        AnimEvent::ArrowReleased(hit) => {
            log::info!("{} ({}): flecha em {} por {}", entity.name, entity.object_id, hit.target, hit.damage)
        }
        AnimEvent::MagicShot { skill_id, target, .. } => {
            log::info!("{} ({}): magia {skill_id} em {target}", entity.name, entity.object_id)
        }
        AnimEvent::Social { id, duration } => {
            log::info!("{} ({}): social {id} por {duration:.2} s", entity.name, entity.object_id)
        }
        AnimEvent::TransientEffect { skill_id, target, .. } => {
            log::debug!("{} ({}): efeito da skill {skill_id} em {target}", entity.name, entity.object_id)
        }
        AnimEvent::MagicPreShot { skill_id, target, .. } => {
            log::debug!("{} ({}): pré-tiro da magia {skill_id} em {target}", entity.name, entity.object_id)
        }
        AnimEvent::CastStart { .. } | AnimEvent::MagicStop | AnimEvent::MoveStop | AnimEvent::Notify(_) => {}
    }
}

pub const DEMO_NPCS: [(i32, f32, f32, f32, i32, f32); 6] = [
    (1, 60.0, -40.0, 16.0, 0, 0.0),
    (7001, 70.0, -70.0, 24.0, 32768, 0.0),
    (999_999, 120.0, 60.0, 0.0, 49152, 0.0),
    (1, 40.0, 40.0, 16.0, 16384, DEMO_BURIED_DEPTH),
    (DEMO_SOCIAL_NPCS[0].0, 100.0, -20.0, 24.0, 32768, 0.0),
    (DEMO_SOCIAL_NPCS[1].0, 100.0, 20.0, 24.0, 32768, 0.0),
];
pub const DEMO_SOCIAL_NPCS: [(i32, u32); 2] = [(7003, 1), (7005, 2)];
pub const DEMO_SITTING_PLAYER_ID: i32 = 8;
const DEMO_SITTING_PLAYER_OFFSET: (f32, f32) = (30.0, 50.0);
const DEMO_ATK_SPD: i32 = 333;
pub const DEMO_BURIED_DEPTH: f32 = 40.0;
const DEMO_PLAYER_COLLISION_HEIGHT: f32 = 24.0;
const DEMO_HAIR_STYLE: i32 = 1;
const DEMO_HAIR_COLOR: i32 = 1;
pub const DEMO_CHEST: i32 = 21;
pub const DEMO_LEGS: i32 = 28;
pub const DEMO_GLOVES: i32 = 48;
pub const DEMO_FEET: i32 = 35;
pub const DEMO_RHAND: i32 = 1;
pub const DEMO_LHAND: i32 = 18;
pub const DEMO_LEVEL: i32 = 52;
const DEMO_EXP: i32 = 52262490;
pub const DEMO_HP: (i32, i32) = (400, 900);
pub const DEMO_TARGET_ID: i32 = 2;
const DEMO_TARGET_HP: (i32, i32) = (30, 80);
pub const DEMO_MP: (i32, i32) = (200, 300);
pub const DEMO_CP: (i32, i32) = (100, 100);
const DEMO_COLLISION_RADIUS: f32 = 8.0;
const DEMO_CROWD_SPACING: f32 = 64.0;
const DEMO_CROWD_FIRST_ID: i32 = 100;
const DEMO_ABNORMAL_MASKS: [i32; 13] = [0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80, 0x100, 0x200, 0x800, 0x2000, 0x4000];
const DEMO_BUFFS: [(i32, u16, i32); 3] = [(1204, 2, 1200), (1040, 3, -1), (1069, 1, 25)];
const DEMO_SHORT_BUFF: (i32, i32, i32) = (2231, 1, 15);
const DEMO_PARTY: [(i32, &str, i32, i32, i32, i32); 2] =
    [(501, "Luzal", 480, 600, 600, 300), (502, "Kain", 150, 600, -900, 400)];
const DEMO_PARTY_MP: (i32, i32) = (45, 90);
const DEMO_PARTY_CP: (i32, i32) = (80, 100);

pub fn demo_entity(object_id: i32, kind: EntityKind, position: Vec3, heading: i32, collision_height: f32) -> Entity {
    Entity {
        item_origin: None,
        abnormal_effect: 0,
        object_id,
        kind,
        name: String::new(),
        title: String::new(),
        position,
        heading,
        dest: None,
        stop_heading: None,
        desired_heading: heading,
        turn_rate: None,
        run_speed: 120,
        walk_speed: 80,
        speed_multiplier: 1.0,
        attack_speed_multiplier: 1.0,
        m_atk_speed: DEMO_ATK_SPD,
        skill_aim: [0; 3],
        p_atk_speed: DEMO_ATK_SPD,
        speed_bonus: 0.0,
        speed_bonus_left: 0.0,
        blocked: false,
        follow: None,
        running: true,
        sitting: false,
        fake_dead: false,
        dead: false,
        walk_anim_frame_in_move: DEFAULT_WALK_ANIM_FRAME_IN_MOVE,
        hp: None,
        mp: None,
        cp: None,
        level: 0,
        exp: 0,
        collision_radius: DEMO_COLLISION_RADIUS,
        collision_height,
        base_collision: [DEMO_COLLISION_RADIUS, collision_height],
        karma: 0,
        pvp_flag: 0,
        relation: 0, auto_attack: false, surrender_war_id: 0,
        target: None,
        casting: None,
        magic: Default::default(),
        anim: spawned_anim(),
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

fn spawned_anim() -> AnimPlayer {
    let mut anim = AnimPlayer::default();
    anim.push(Trigger::Spawn);
    anim
}

pub fn demo_world(anchor: Vec3, heading: i32, crowd: usize) -> World {
    let angle = heading as f32 * TAU / HEADING_UNITS_PER_TURN;
    let forward = Vec3::new(angle.cos(), angle.sin(), 0.0);
    let right = forward.cross(Vec3::Z);
    let mut world = World::default();
    let mut equipped = [0; VISIBLE_EQUIP_SLOTS];
    equipped[EQUIP_CHEST] = DEMO_CHEST;
    equipped[EQUIP_LEGS] = DEMO_LEGS;
    equipped[EQUIP_GLOVES] = DEMO_GLOVES;
    equipped[EQUIP_FEET] = DEMO_FEET;
    equipped[EQUIP_RHAND] = DEMO_RHAND;
    equipped[EQUIP_LHAND] = DEMO_LHAND;
    let kind = EntityKind::Player {
        class_id: 0,
        race: 0,
        sex: 0,
        equipped,
        hair_style: DEMO_HAIR_STYLE,
        hair_color: DEMO_HAIR_COLOR,
        face: 0,
        enchant_effect: 0,
        hero: false,
    };
    let mut me = demo_entity(1, kind.clone(), anchor, heading, DEMO_PLAYER_COLLISION_HEIGHT);
    me.name = "Killer".to_string();
    me.level = DEMO_LEVEL;
    me.exp = DEMO_EXP;
    me.hp = Some(DEMO_HP);
    me.mp = Some(DEMO_MP);
    me.cp = Some(DEMO_CP);
    world.me = Some(me);
    for (index, (npc_id, ahead, side, collision_height, turn, depth)) in DEMO_NPCS.into_iter().enumerate() {
        let position = anchor + forward * ahead + right * side - Vec3::Z * depth;
        let kind = EntityKind::Npc { npc_id, rhand: 0, chest: 0, lhand: 0, attackable: true };
        let mut npc =
            demo_entity(2 + index as i32, kind, position, (heading + turn).rem_euclid(65536), collision_height);
        if index == 0 {
            npc.hp = Some(DEMO_TARGET_HP);
        }
        if let Some((_, social)) = DEMO_SOCIAL_NPCS.iter().find(|(id, _)| *id == npc_id) {
            npc.anim.push(Trigger::Social(*social));
        }
        world.entities.insert(npc.object_id, npc);
    }
    let seated = anchor + forward * DEMO_SITTING_PLAYER_OFFSET.0 + right * DEMO_SITTING_PLAYER_OFFSET.1;
    let mut sitter = demo_entity(
        DEMO_SITTING_PLAYER_ID,
        kind.clone(),
        seated,
        (heading + 32768).rem_euclid(65536),
        DEMO_PLAYER_COLLISION_HEIGHT,
    );
    sitter.name = "Sentado".to_string();
    sitter.sitting = true;
    sitter.anim.push(Trigger::ChangeWaitType { change: WaitChange::Sit, from: WaitType::Stand });
    world.entities.insert(sitter.object_id, sitter);
    let side = (crowd as f32).sqrt().ceil() as usize;
    let center = (side.saturating_sub(1) as f32) * 0.5;
    for index in 0..crowd {
        let row = index / side;
        let column = index % side;
        let position = anchor
            + forward * ((row as f32 - center) * DEMO_CROWD_SPACING)
            + right * ((column as f32 - center) * DEMO_CROWD_SPACING);
        let facing = (heading + 32768).rem_euclid(HEADING_UNITS_PER_TURN as i32);
        let mut player = demo_entity(
            DEMO_CROWD_FIRST_ID + index as i32,
            kind.clone(),
            position,
            facing,
            DEMO_PLAYER_COLLISION_HEIGHT,
        );
        player.name = format!("Killer{index}");
        player.title = "L2Killer".to_string();
        if let Some(mask) = DEMO_ABNORMAL_MASKS.get((index + DEMO_ABNORMAL_MASKS.len() / 2).wrapping_sub(side * (side / 2) + side / 2)) {
            player.abnormal_effect = *mask;
        }
        world.entities.insert(player.object_id, player);
    }
    world.select(DEMO_TARGET_ID);
    world.buffs =
        DEMO_BUFFS.into_iter().map(|(skill_id, level, duration)| Buff { skill_id, level, duration }).collect();
    world.buffs_at = Some(killer_platform::time::Instant::now());
    let (skill_id, level, duration) = DEMO_SHORT_BUFF;
    world.short_buff = Some(ShortBuffStatusUpdate { skill_id, level, duration });
    world.short_buff_at = world.buffs_at;
    world.party.leader_id = DEMO_PARTY[0].0;
    for (object_id, name, hp, max_hp, dx, dy) in DEMO_PARTY {
        world.party.upsert(&PartyMember {
            object_id,
            name: name.to_string(),
            cp: DEMO_PARTY_CP.0,
            max_cp: DEMO_PARTY_CP.1,
            hp,
            max_hp,
            mp: DEMO_PARTY_MP.0,
            max_mp: DEMO_PARTY_MP.1,
            level: DEMO_LEVEL,
            class_id: 0,
        });
        world.party.positions.insert(object_id, [anchor.x as i32 + dx, anchor.y as i32 + dy, anchor.z as i32]);
    }
    world
}

pub fn label_height(collision_height: f32, scale: f32, sitting: bool, dead: bool) -> f32 {
    let mut z = collision_height * (2.0 * scale - 1.0);
    if sitting {
        z -= SIT_LABEL_DROP * collision_height;
    }
    if dead {
        z -= DEAD_LABEL_DROP * collision_height;
    }
    z
}

pub fn unique_name_color(packet: i32) -> Option<[u8; 3]> {
    let bytes = packet.to_le_bytes();
    (bytes[..3] != [0xff; 3]).then_some([bytes[0], bytes[1], bytes[2]])
}

pub fn name_color(karma: i32, pvp_flag: i32, unique: i32, recommendations: i32, blink: bool) -> [u8; 3] {
    if karma > 0 {
        let k = karma.min(255);
        let c = k.max((k >> 4) + 100) as u8;
        return [255, 255 - c, 255 - c];
    }
    if pvp_flag == 2 && blink {
        return MAGENTA;
    }
    if pvp_flag == 1 {
        return MAGENTA;
    }
    if let Some(color) = unique_name_color(unique) {
        return color;
    }
    if (1..=255).contains(&recommendations) {
        let rec = recommendations as u8;
        return [255 - rec, 255 - rec / 2, 255];
    }
    WHITE
}

pub fn entity_name_color(entity: &Entity) -> [u8; 3] {
    name_color(entity.karma, entity.pvp_flag, entity.name_color, entity.recommendations, entity.name_blink)
}

pub struct Projection {
    matrix: Mat4,
    size: (f32, f32),
}

impl Projection {
    pub fn new(camera: &Camera, size: (f32, f32)) -> Option<Self> {
        if size.0 <= 0.0 || size.1 <= 0.0 {return None;}
        Some(Self {matrix:camera.view_proj(size.0 / size.1),size})
    }

    pub fn point(&self, point: Vec3) -> Option<[f32; 2]> {
        let [x,y,_] = self.point_depth(point)?;
        Some([x,y])
    }

    pub fn point_depth(&self, point: Vec3) -> Option<[f32; 3]> {
        let clip = self.matrix * point.extend(1.0);
        if clip.w <= 0.0 {return None;}
        let ndc = clip.truncate() / clip.w;
        if ndc.x.abs() > 1.0 || ndc.y.abs() > 1.0 || !(0.0..=1.0).contains(&ndc.z) {return None;}
        Some([(ndc.x + 1.0) * 0.5 * self.size.0, (1.0 - ndc.y) * 0.5 * self.size.1, ndc.z])
    }
}

pub fn project(camera: &Camera, size: (f32, f32), point: Vec3) -> Option<[f32; 2]> {
    Projection::new(camera,size)?.point(point)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    pub object_id: i32,
    pub name: String,
    pub title: String,
    pub store_sign: Option<killer_ui::store_sign::StoreSign>,
    pub speech: Option<killer_ui::store_sign::StoreSign>,
    pub modern_text_size: Option<[[f32;2];2]>,
    pub name_visible: bool,
    pub name_color: [u8; 3],
    pub title_color: [u8; 3],
    pub screen: [f32; 2],
    pub depth: f32,
    pub clan_crest: Option<String>,
    pub ally_crest: Option<String>,
    pub mark: Option<TargetMark>,
    pub relation_icons: RelationIcons,
    pub broadcast_icon: Option<BroadcastIcon>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BroadcastIcon {
    pub team: usize,
    pub index: usize,
    pub names_on: bool,
}

pub const BROADCAST_ICON_SIZE: i32 = 64;
pub const BROADCAST_ICON_DX: i32 = -22;
pub const BROADCAST_ICON_DY: i32 = -54;

impl BroadcastIcon {
    pub fn texture(&self) -> String {
        format!("L2UI_CH3.BroadcastObs.br_party{}_{}", self.team + 1, self.index + 1)
    }

    pub fn origin(&self, x: i32, y: i32, name_y: i32) -> (i32, i32) {
        (x + BROADCAST_ICON_DX, if self.names_on { name_y } else { y } + BROADCAST_ICON_DY)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RelationIcons {
    pub base: Option<&'static str>,
    pub leader: Option<&'static str>,
}

pub const RELATION_ICON_WIDTH: i32 = 16;
pub const RELATION_ICON_HEIGHT: i32 = 12;
pub const RELATION_ROW_STEP: i32 = 14;
pub const RELATION_ICON_NAME_OVERLAP: i32 = 2;

pub const WAR_SINGLE_ICON: &str = "l2ui_ch3.Pledge.war_single";
pub const WAR_DOUBLE_ICON: &str = "l2ui_ch3.Pledge.war_double";
pub const BLUE_SHIELD_ICON: &str = "l2ui_ch3.Pledge.blueshield";
pub const BLUE_SWORD_ICON: &str = "l2ui_ch3.Pledge.bluesword";
pub const BLUE_LEADER_ICON: &str = "l2ui_ch3.Pledge.blueleader";
pub const BLUE_ATTACK_LEADER_ICON: &str = "l2ui_ch3.Pledge.blueattleader";
pub const RED_SHIELD_ICON: &str = "l2ui_ch3.Pledge.redshield";
pub const RED_SWORD_ICON: &str = "l2ui_ch3.Pledge.redsword";
pub const RED_LEADER_ICON: &str = "l2ui_ch3.Pledge.redleader";
pub const RED_ATTACK_LEADER_ICON: &str = "l2ui_ch3.Pledge.redattleader";

const RELATION_WAR: u32 = 0x10000;
const RELATION_WAR_DOUBLE: u32 = 0x8000;
const RELATION_RED: u32 = 0x1000;
const RELATION_BLUE: u32 = 0x800;
const RELATION_SHIELD: u32 = 0x200;
const RELATION_SWORD: u32 = 0x400;
const RELATION_LEADER: u32 = 0x80;
const RELATION_SELF_SWORD: u32 = 0x100;
const RELATION_SELF_LEADER: u32 = 0x40;

pub fn relation_icons(relation: i32, is_me: bool) -> RelationIcons {
    let bits = relation as u32;
    if is_me {
        if bits & RELATION_LEADER == 0 {
            return RelationIcons::default();
        }
        let sword = bits & RELATION_SELF_SWORD != 0;
        let (base, leader) = if sword { (BLUE_SWORD_ICON, BLUE_ATTACK_LEADER_ICON) } else { (BLUE_SHIELD_ICON, BLUE_LEADER_ICON) };
        return RelationIcons { base: Some(base), leader: (bits & RELATION_SELF_LEADER != 0).then_some(leader) };
    }
    let siege = |shield: &'static str, sword: &'static str, leader: &'static str, attack_leader: &'static str| {
        if bits & RELATION_SHIELD == 0 {
            return RelationIcons::default();
        }
        let attacker = bits & RELATION_SWORD != 0;
        RelationIcons {
            base: Some(if attacker { sword } else { shield }),
            leader: (bits & RELATION_LEADER != 0).then_some(if attacker { attack_leader } else { leader }),
        }
    };
    if bits & RELATION_BLUE != 0 {
        siege(BLUE_SHIELD_ICON, BLUE_SWORD_ICON, BLUE_LEADER_ICON, BLUE_ATTACK_LEADER_ICON)
    } else if bits & RELATION_RED != 0 {
        siege(RED_SHIELD_ICON, RED_SWORD_ICON, RED_LEADER_ICON, RED_ATTACK_LEADER_ICON)
    } else if bits & RELATION_WAR != 0 {
        RelationIcons { base: Some(if bits & RELATION_WAR_DOUBLE != 0 { WAR_DOUBLE_ICON } else { WAR_SINGLE_ICON }), leader: None }
    } else {
        RelationIcons::default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetMark {
    Normal,
    Target,
    Attack,
}

impl TargetMark {
    pub fn texture(self) -> &'static str {
        match self {
            TargetMark::Normal => "L2ui.NWindow.normal",
            TargetMark::Target => "L2ui.NWindow.target",
            TargetMark::Attack => "L2ui.NWindow.attack",
        }
    }
}

pub const EFFECT_TARGET_BONE: usize = 2;

pub fn arrow_scale(radius: f32, height: f32) -> f32 {
    ((f64::from(height) * f64::from(radius)).sqrt() / 207.0f64.sqrt()) as f32
}

pub const TARGET_MARK_WIDTH: i32 = 19;
pub const TARGET_MARK_HEIGHT: i32 = 16;
pub const TARGET_MARK_PLEDGE_SHIFT: i32 = 0x12;
pub const TARGET_MARK_ALLY_SHIFT: i32 = 8;

pub const LABEL_PLEDGE_CREST_WIDTH: i32 = 16;
pub const LABEL_ALLY_CREST_WIDTH: i32 = 8;
pub const LABEL_CREST_HEIGHT: i32 = 12;
pub const LABEL_CREST_V: i32 = 4;

impl Label {
    pub fn crests_width(&self) -> i32 {
        self.clan_crest.as_ref().map_or(0, |_| LABEL_PLEDGE_CREST_WIDTH)
            + self.ally_crest.as_ref().map_or(0, |_| LABEL_ALLY_CREST_WIDTH)
    }

    pub fn mark_shift(&self) -> i32 {
        self.clan_crest.as_ref().map_or(0, |_| TARGET_MARK_PLEDGE_SHIFT)
            + self.ally_crest.as_ref().map_or(0, |_| TARGET_MARK_ALLY_SHIFT)
    }

    pub fn relation_origin(&self, x: i32, y: i32, title_w: i32, line_height: i32) -> Option<(i32, i32)> {
        self.relation_icons.base?;
        let name_row = y - crate::ui2d::ENGINE_NAME_RISE;
        Some(if self.name_visible || self.mark.is_some() {
            (x - (title_w + RELATION_ICON_WIDTH) / 2, name_row - (line_height - RELATION_ICON_NAME_OVERLAP))
        } else if self.clan_crest.is_none() && self.ally_crest.is_none() {
            (x - RELATION_ICON_WIDTH / 2, name_row - RELATION_ICON_NAME_OVERLAP)
        } else {
            (x - RELATION_ICON_WIDTH / 2, y - 2 * RELATION_ROW_STEP)
        })
    }

    pub fn title_left(&self, x: i32, title_w: i32, origin: Option<(i32, i32)>) -> i32 {
        origin.map_or(x - title_w / 2, |(icon_x, _)| icon_x + RELATION_ICON_WIDTH)
    }
}

pub fn collision_height(entity: &Entity, tables: &DatTables) -> f32 {
    match &entity.kind {
        EntityKind::Npc { .. } if npc_record(tables, &entity.kind).is_none() => NPC_FALLBACK.collision_height,
        _ => entity.collision_height,
    }
}

fn npc_name<'a>(entity: &Entity, tables: &'a DatTables) -> Option<&'a NpcName> {
    match &entity.kind {
        EntityKind::Npc { npc_id, .. } => u32::try_from(*npc_id).ok().and_then(|id| tables.npcname.get(&id)),
        EntityKind::Player { .. } | EntityKind::Item { .. } => None,
    }
}

fn fallback(packet: &str, table: Option<&str>) -> String {
    if packet.is_empty() { table.unwrap_or_default().to_string() } else { packet.to_string() }
}

pub fn display_name(entity: &Entity, tables: &DatTables) -> String {
    fallback(&entity.name, npc_name(entity, tables).map(|n| n.name.as_str()))
}

pub fn label_of(entity: &Entity, tables: &DatTables, scale: f32, projection: &Projection) -> Option<Label> {
    let height = label_height(collision_height(entity, tables), scale, entity.sitting, entity.dead)
        + killer_world::anim::ride::valid(entity.mount).map_or(0.0, |mount| killer_world::anim::ride::label_lift(mount.ride_type));
    let [x,y,depth] = projection.point_depth(entity.position + Vec3::Z * height)?;
    let screen = [x,y];
    let npc_name = npc_name(entity, tables);
    Some(Label {
        object_id: entity.object_id,
        name: display_name(entity, tables),
        title: fallback(&entity.title, npc_name.map(|n| n.title.as_str())),
        store_sign: None, speech: None, modern_text_size: None, name_visible: true,
        name_color: entity_name_color(entity),
        title_color: npc_name.map_or(WHITE, |n| n.color),
        screen, depth,
        clan_crest: None,
        ally_crest: None,
        mark: None,
        relation_icons: RelationIcons::default(),
        broadcast_icon: None,
    })
}

pub fn label_rects(label: &Label, font: &BitmapFont) -> Vec<(i32, i32, i32, i32)> {
    if !label.name_visible { return Vec::new(); }
    let line_height = label.modern_text_size.map_or(font.line_height as i32,|size|size[0][1].ceil() as i32);
    let x = label.screen[0].round() as i32;
    let y = label.screen[1].round() as i32;
    let mut rects = Vec::new();
    let name_w = label.modern_text_size.map_or_else(||font.measure(&label.name) as i32,|size|size[0][0].round() as i32) + label.crests_width();
    rects.push((x - name_w / 2, y - crate::ui2d::ENGINE_NAME_RISE, name_w, line_height));
    let title = &label.title;
    if !title.is_empty() {
        let (title_w,title_h,title_y)=label.modern_text_size.map_or_else(||(font.measure(title) as i32,line_height,y-2*line_height),|size|(size[1][0].ceil() as i32,size[1][1].ceil() as i32,y-crate::ui2d::ENGINE_NAME_RISE-size[1][1].ceil() as i32-2));
        let origin=label.relation_origin(x,y,title_w,line_height);
        rects.push((label.title_left(x,title_w,origin),title_y,title_w,title_h));
    }
    rects
}

pub fn label_at(labels: &[Label], font: &BitmapFont, pixel: (f32, f32)) -> Option<i32> {
    let (px, py) = (pixel.0.round() as i32, pixel.1.round() as i32);
    labels
        .iter()
        .rev()
        .find(|label| {
            label_rects(label, font).iter().any(|&(x, y, w, h)| px >= x && px < x + w && py >= y && py < y + h)
        })
        .map(|label| label.object_id)
}

pub fn labels(
    world: &World,
    tables: &DatTables,
    cache: &CharacterCache,
    frame: &Frame,
    camera: &Camera,
    size: (f32, f32),
    hovered: Option<i32>,
) -> Vec<Label> {
    if frame.display.hide_names {
        return Vec::new();
    }
    let drawn: HashSet<i32> = drawn_pawns(world, tables, frame).into_iter().collect();
    labels_drawn(world, tables, cache, frame, camera, size, hovered, &drawn)
}

#[allow(clippy::too_many_arguments)]
pub fn labels_drawn(
    world: &World,
    tables: &DatTables,
    cache: &CharacterCache,
    frame: &Frame,
    camera: &Camera,
    size: (f32, f32),
    hovered: Option<i32>,
    drawn: &HashSet<i32>,
) -> Vec<Label> {
    if frame.display.hide_names {return Vec::new();}
    let Some(projection)=Projection::new(camera,size) else {return Vec::new();};
    let mut camera_up=None;
    let me_id = world.me.as_ref().map(|me| me.object_id);
    world
        .me
        .iter()
        .chain(world.entities.values())
        .filter(|entity| me_id == Some(entity.object_id) || drawn.contains(&entity.object_id))
        .filter(|entity| {
            world.target == Some(entity.object_id) || hovered == Some(entity.object_id) || frame.display.name_in_range(frame.viewer, entity.position)
        })
        .filter_map(|entity| {
            let scale = entity_key(entity).and_then(|key| cache.get(key)).map_or(1.0, LoadedCharacter::scale);
            let mut label = label_of(entity, tables, scale, &projection)?;
            let is_me = me_id == Some(entity.object_id);
            let (npc, attackable) = match entity.kind {
                EntityKind::Npc { attackable, .. } => (true, attackable),
                _ => (false, false),
            };
            label.name_visible = frame.display.shows_name_of(is_me, npc, attackable);
            label.relation_icons = relation_icons(entity.relation, is_me);
            label.broadcast_icon = world.appendix.icon(entity.object_id).map(|(team, index)| BroadcastIcon {
                team,
                index,
                names_on: world.appendix.canvas & killer_world::appendix::CANVAS_TARGET_NAME != 0,
            });
            label.mark = if world.target == Some(entity.object_id) {
                Some(if world.attack_intend { TargetMark::Attack } else { TargetMark::Target })
            } else if hovered == Some(entity.object_id) {
                Some(TargetMark::Normal)
            } else {
                None
            };
            if entity.mount.is_none() && frame.display.name_in_range(frame.viewer, entity.position)
                && let Some((id, color, background)) = killer_ui::store_sign::style(entity.store_type, entity.sitting, world.store_matches.get(&entity.object_id).copied().unwrap_or(0))
                && let Some(prefix) = tables.sysstring.get(&id)
            {
                let message = match entity.store_type {
                    1 | 8 => world.stores.sell_messages.get(&entity.object_id),
                    3 => world.stores.buy_messages.get(&entity.object_id),
                    5 => world.recipes.shop_messages.get(&entity.object_id),
                    _ => None,
                }.filter(|text| !text.is_empty());
                let text = message.map_or_else(|| prefix.clone(), |text| format!("{prefix}\n{text}"));
                let anchor = entity.position + Vec3::Z * label_height(collision_height(entity, tables), scale, entity.sitting, entity.dead);
                let up = *camera_up.get_or_insert_with(||camera.view().inverse().transform_vector3(Vec3::Y).normalize_or_zero());
                let offset = (camera.eye.distance(anchor) as i32) as f32 * 0.001 * if entity.title.is_empty() { 25.0 } else { 35.0 };
                label.store_sign = projection.point(anchor + up * offset).map(|screen| killer_ui::store_sign::StoreSign { text, color, background, screen });
            }
            if entity.mount.is_none() && frame.display.name_in_range(frame.viewer,entity.position)
                && matches!(entity.store_type,0 | 2 | 4 | 6)
                && let Some(text)=world.speech.text(entity.object_id) {
                let anchor=entity.position+Vec3::Z*label_height(collision_height(entity,tables),scale,entity.sitting,entity.dead);
                let up=*camera_up.get_or_insert_with(||camera.view().inverse().transform_vector3(Vec3::Y).normalize_or_zero());
                let offset=(camera.eye.distance(anchor) as i32) as f32*0.001*if entity.title.is_empty(){25.0}else{35.0};
                label.speech=projection.point(anchor+up*offset).map(|screen|killer_ui::store_sign::StoreSign {text:text.into(),color:killer_ui::color::Argb(0xffffffff),background:killer_ui::color::Argb(0xff000000),screen});
            }
            if entity.clan_id > 0 {
                let crests = &world.pledge_window;
                label.clan_crest = (entity.clan_crest_id > 0 && crests.crests.contains_key(&entity.clan_crest_id))
                    .then(|| crate::ui2d::crest_texture(entity.clan_crest_id));
                label.ally_crest = (entity.ally_crest_id > 0 && crests.ally_crests.contains_key(&entity.ally_crest_id))
                    .then(|| crate::ui2d::ally_crest_texture(entity.ally_crest_id));
            }
            if world.appendix.name_suppressed(entity.object_id) {
                label.name_visible = false;
                label.mark = None;
                label.relation_icons = RelationIcons::default();
                label.title = String::new();
                label.clan_crest = None;
                label.ally_crest = None;
            }
            (label.name_visible || label.store_sign.is_some() || label.speech.is_some() || label.mark.is_some() || label.broadcast_icon.is_some()).then_some(label)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn lod_history_tracks_mesh_identity_without_sharing_actor_state() {
        use killer_assets::skeletal_lod::LodInstance;
        let mut first = Vec::new();
        let mut second = Vec::new();
        let parts = [(1, "Fighter.Body"), (2, "Fighter.Head")];
        synchronize_lods(&mut first, parts.into_iter());
        synchronize_lods(&mut second, parts.into_iter());
        assert!(first.iter().all(|part| part.state == LodInstance::default()));
        let history = LodInstance { index: 2, forced: 1, factor: 0.5 };
        first[0].state = history;
        synchronize_lods(&mut first, [(1, "fighter.body"), (2, "fighter.head")].into_iter());
        assert_eq!(first[0].state, history);
        assert_eq!(second[0].state, LodInstance::default());
        synchronize_lods(&mut first, [(2, "Fighter.Head"), (1, "Fighter.Body")].into_iter());
        assert_eq!(first[1].state, history);
        synchronize_lods(&mut first, [(1, "Fighter.OtherBody"), (2, "Fighter.Head")].into_iter());
        assert!(first.iter().all(|part| part.state == LodInstance::default()));
        synchronize_lods(&mut first, [(2, "Fighter.Head")].into_iter());
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].slot, 2);
        synchronize_lods(&mut first, std::iter::empty());
        assert!(first.is_empty());
    }

    fn explosion_context() -> killer_assets::skill_effects::Context {
        use killer_assets::skill_effects::{Actor, Context};
        let caster = Actor { position: Vec3::new(100.0, 200.0, 300.0), radius: 20.0, height: 40.0, draw_scale: 1.0, ..Actor::default() };
        let hit = Actor { position: Vec3::new(500.0, 700.0, 300.0), radius: 30.0, height: 60.0, draw_scale: 1.0, ..Actor::default() };
        let impact = Vec3::new(500.0, 700.0, 315.0);
        Context {
            caster: Some(caster),
            target: Some(hit),
            hit: Some(hit),
            projectile: Some(Actor { position: impact, ..Actor::default() }),
            normal: -(impact - caster.position).normalize(),
            randoms: [0.25, 0.75],
            ..Context::default()
        }
    }

    fn explosion_owner(party: killer_assets::skill_effects::Party, _: Option<usize>) -> Option<(i32, AppearanceKey)> {
        use killer_assets::skill_effects::Party;
        let key = |npc_id| AppearanceKey::Npc { npc_id, equipped: Equipped::default() };
        match party {
            Party::Caster => Some((1, key(10))),
            Party::Hit | Party::Target => Some((2, key(20))),
            _ => None,
        }
    }

    #[test]
    fn explosion_routes_the_level_spawn_to_the_caster_owner_with_the_trailer_physics_of_set_physics_10() {
        use killer_assets::skill_effects::explosion;
        let plan = explosion::plan(explosion::steps(0, 0x171), &explosion_context(), &[]);
        let route = route_explosion(&plan, explosion_owner, true, None, |_| 3);
        assert_eq!(route.world_spawns.len(), 1);
        let spawn = &route.world_spawns[0];
        assert_eq!(spawn.class, "LineageEffect.sp_agility_ta");
        assert_eq!(spawn.owner.map(|owner| owner.object_id), Some(1));
        assert_eq!((spawn.zone, spawn.projectile.is_none(), spawn.trailer_physics), (3, true, Some((10, true, false))));
        assert_eq!(spawn.location, plan.spawns[0].planned.location);
        assert!(route.sound && route.shakes.is_empty());
    }

    #[test]
    fn explosion_view_shake_maps_add_view_shake_state_arguments_p3_to_p20_onto_add_shake() {
        use killer_assets::skill_effects::explosion;
        let plan = explosion::plan(explosion::steps(0, 0x100f), &explosion_context(), &[]);
        let route = route_explosion(&plan, explosion_owner, true, None, |_| 0);
        let values = plan.shakes[0].values;
        let shake = route.shakes[0];
        assert_eq!((shake.kind, shake.time, shake.scale, shake.frequency, shake.count), (0, values[0], values[1], values[2], values[3]));
        assert_eq!((shake.direction, shake.second, shake.third), ([values[4], values[5], values[6]], [values[7], values[8], values[9]], [values[10], values[11], values[12]]));
        assert_eq!((shake.origin, shake.intensity, shake.range), ([500.0, 700.0, 240.0], 200.0, 2000.0));
    }

    #[test]
    fn explosion_without_a_present_caster_plays_no_sound_and_bone_index_attaches_stay_unresolved() {
        use killer_assets::skill_effects::explosion;
        let plan = explosion::plan(explosion::steps(0, 0x18), &explosion_context(), &[]);
        let route = route_explosion(&plan, explosion_owner, false, None, |_| 0);
        assert!(!route.sound);
        assert!(route.bone_spawns.is_empty());
        assert_eq!(route.world_spawns.len(), plan.spawns.iter().filter(|spawned| spawned.planned.bone.is_none()).count());
    }

    #[test]
    fn explosion_lights_go_to_the_hit_pawn_with_add_pawn_light_then_init_104c5af0() {
        use killer_assets::skill_effects::explosion;
        let plan = explosion::plan(explosion::steps(0, 0x44c), &explosion_context(), &[]);
        assert_eq!(plan.lights.len(), 1);
        let mut list = Vec::new();
        add_explosion_light(&mut list, &plan.lights[0]);
        let light = list[0];
        assert_eq!((light.kind, light.light_type, light.effect, light.attach), (0, 1, 12, 0));
        assert_eq!((light.plane, light.life, light.radius), ([1.0, 0.0, 0.0, 1.0], f32::from_bits(0x3e99_999a), 775.0));
        assert_eq!(light.location, plan.lights[0].init.expect("Init").location.to_array());
    }

    #[test]
    fn explosion_damage_effect_spawns_the_class_set_pawn_resource_loaded_into_pawn_630() {
        use killer_assets::skill_effects::explosion;
        let plan = explosion::plan(explosion::steps(explosion::EXPLOSION_DAMAGE_EFFECT, 0x13), &explosion_context(), &[]);
        let damage = plan.spawns.iter().filter(|spawned| spawned.damage_effect).count();
        assert_eq!(damage, 1);
        let with = route_explosion(&plan, explosion_owner, true, Some("LineageEffect.p_u002_a"), |_| 0);
        let spawn = with.world_spawns.iter().find(|spawn| spawn.class == "LineageEffect.p_u002_a").expect("DamageEffect");
        assert_eq!(spawn.owner.map(|owner| owner.object_id), Some(2));
        let without = route_explosion(&plan, explosion_owner, true, None, |_| 0);
        assert_eq!(without.world_spawns.len() + 1, with.world_spawns.len());
        assert!(without.world_spawns.iter().all(|spawn| !spawn.class.is_empty()));
    }

    #[test]
    fn explosion_area_list_spawns_one_effect_per_listed_target() {
        use killer_assets::skill_effects::{Actor, explosion};
        let areas = [Actor { position: Vec3::new(600.0, 650.0, 300.0), radius: 15.0, ..Actor::default() }, Actor { position: Vec3::new(-40.0, 30.0, 10.0), radius: 15.0, ..Actor::default() }];
        let plan = explosion::plan(explosion::steps(explosion::EXPLOSION_AREA, 0x498), &explosion_context(), &areas);
        let route = route_explosion(&plan, explosion_owner, true, None, |_| 0);
        let listed: Vec<Vec3> = route.world_spawns.iter().filter(|spawn| spawn.class == "LineageEffect.m_u038_d").map(|spawn| spawn.location).collect();
        assert_eq!(listed.len(), areas.len());
        assert!(listed[0].distance(areas[0].position) < listed[0].distance(areas[1].position));
        assert!(listed[1].distance(areas[1].position) < listed[1].distance(areas[0].position));
        assert_eq!(plan.damage_acts, vec![(killer_assets::skill_effects::Party::Hit, None)]);
    }

    #[test]
    fn damage_acts_stay_while_the_pawn_exists_and_only_tick_with_spine_rotation() {
        let mut cache = CharacterCache::default();
        cache.damage_acts.entry(2).or_default().attacked();
        cache.damage_acts.entry(3).or_default().attacked();
        let key = AppearanceKey::Npc { npc_id: 20, equipped: Equipped::default() };
        cache.tick_damage_acts(0.05, |id| (id == 2).then_some(key));
        assert!(!cache.damage_acts.contains_key(&3));
        assert_eq!(cache.damage_acts[&2], killer_assets::damage_act::DamageAct { active: true, status: 0, dist: 0.0 });
    }

    #[test]
    fn pawn_lights_tick_with_the_pawn_and_vanish_with_it() {
        let mut cache = CharacterCache::default();
        add_hit_light(cache.pawn_lights_mut(2), Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0), 20.0);
        add_hit_light(cache.pawn_lights_mut(3), Vec3::ZERO, Vec3::new(0.0, 100.0, 0.0), 20.0);
        assert_eq!(cache.pawn_lights[&2][0].location, [60.0, 0.0, 0.0]);
        cache.tick_pawn_lights(0.1, |id| id == 2);
        assert!(!cache.pawn_lights.contains_key(&3));
        assert_eq!(cache.pawn_lights[&2][0].revision, 1);
        cache.tick_pawn_lights(0.11, |_| true);
        assert!(cache.pawn_lights.is_empty(), "a luz do golpe vive 0,2 s");
    }

    #[test]
    fn phase_light_actor_follows_the_owner_like_phys_trailer_once_the_class_is_known() {
        use killer_assets::pawn_light::{Actor, LightActor, PawnLight};
        let mut cache = CharacterCache::default();
        let pawn = Actor { location: [100.0, 200.0, 300.0], rotation: [0, 16384, 0] };
        let effect = LightActor { class: "LineageEffect.e_u031_a", location: [0.0; 3], rotation: [0, 8192, 0], pre_pivot: [0.0, 0.0, -40.0] };
        let light = PawnLight { actor: Some(effect), ..PawnLight::default() };
        assert_eq!(light_actor(&cache.trailer_classes, &PawnLight::default(), pawn), pawn);
        assert_eq!(light_actor(&cache.trailer_classes, &light, pawn), Actor { location: [0.0; 3], rotation: [0, 8192, 0] });
        let trailer = killer_assets::client::TrailerClass { physics: killer_assets::client::PHYS_TRAILER, pre_pivot: true, same_rotation: true, ..Default::default() };
        cache.note_trailer_class(effect.class, trailer);
        cache.note_trailer_class(effect.class, killer_assets::client::TrailerClass::default());
        assert_eq!(light_actor(&cache.trailer_classes, &light, pawn), Actor { location: [100.0, 200.0, 260.0], rotation: [0, 16384, 0] });
        cache.trailer_classes.insert(effect.class.to_owned(), killer_assets::client::TrailerClass { relative_trail: true, ..trailer });
        assert_eq!(light_actor(&cache.trailer_classes, &light, pawn).location, [0.0; 3]);
    }

    #[test]
    fn per_object_maps_of_the_cache_leave_with_the_world_actor() {
        let mut cache = CharacterCache::default();
        let mut world = World::default();
        let kind = EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true };
        let alive = demo_entity(7, kind.clone(), Vec3::ZERO, 0, 20.0);
        world.entities.insert(7, alive);
        for id in [7, 8] {
            cache.effect_targets.insert(id, Vec3::ONE);
            cache.hand_positions.insert(id, Vec3::ONE);
            cache.rod_points.insert(id, Vec3::ONE);
            cache.swim_ticks.insert(id, 1.0);
            cache.foot_volumes.insert(id, crate::walk_world::FootVolume::default());
            cache.unknown.insert(id);
        }
        cache.forget_poses(&world);
        for ids in [cache.effect_targets.keys().copied().collect::<Vec<_>>(), cache.hand_positions.keys().copied().collect(), cache.rod_points.keys().copied().collect(), cache.swim_ticks.keys().copied().collect(), cache.foot_volumes.keys().copied().collect(), cache.unknown.iter().copied().collect()] {
            assert_eq!(ids, [7]);
        }
    }

    #[test]
    fn lod_history_is_removed_with_world_actors_and_cache_drain() {
        let mut cache = CharacterCache::default();
        let mut world = World::default();
        for id in [1, 2, 3] {
            synchronize_lods(cache.lods.entry(id).or_default(), [(1, "Fighter.Body")].into_iter());
        }
        world.me = Some(demo_entity(1, player_kind(0, 0, 0, 0), Vec3::ZERO, 0, 24.0));
        world.entities.insert(2, demo_entity(2, player_kind(0, 0, 0, 0), Vec3::ZERO, 0, 24.0));
        cache.forget_poses(&world);
        assert_eq!(cache.lods.len(), 2);
        assert!(cache.lods.contains_key(&1));
        assert!(cache.lods.contains_key(&2));
        world.me = None;
        world.entities.clear();
        cache.forget_poses(&world);
        assert!(cache.lods.is_empty());
        synchronize_lods(cache.lods.entry(1).or_default(), [(1, "Fighter.Body")].into_iter());
        assert!(cache.drain().is_empty());
        assert!(cache.lods.is_empty());
    }

    #[test]
    #[ignore]
    fn weapon_effect_10496670_live_loader_and_gpu_capture() {
        use killer_assets::scene::{Scene, ZoneLighting};
        use killer_assets::textures::{BankSource, TextureBank, write_png};
        use killer_assets::timeenv::TimeEnv;
        use killer_render::environment::Environment;
        use killer_render::headless::Headless;
        use killer_render::renderer::EffectPlacement;
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        let timeenv = TimeEnv::load(&mut client, 0).unwrap();
        let mut shared = Shared::default();
        for (weapon, class) in [(80, "LineageEffect.c_u006"), (150, "LineageEffect.c_u004")] {
            let equipped = Equipped { rhand: weapon, ..Default::default() };
            let plain = AppearanceKey::Player { row: 0, face: 0, hair_style: 0, hair_color: 0, equipped, enchant_effect: 0, hero: false, mounted: 0 };
            let look_plain = look(plain, &tables).unwrap();
            assert_eq!(look_plain.appearance.weapon_effects.len(), 1);
            assert_eq!(look_plain.appearance.weapon_effects[0].class, class);
            let LoadResult::Character { effects, .. } = load_character(&mut client, &mut shared, plain, &look_plain.appearance, look_plain.draw_scale) else { panic!("carga") };
            assert!(effects.effects.is_empty(), "EnchantEffectShow=1 exige +1");
            let key = AppearanceKey::Player { row: 0, face: 0, hair_style: 0, hair_color: 0, equipped, enchant_effect: 16, hero: false, mounted: 0 };
            let look = look(key, &tables).unwrap();
            let LoadResult::Character { parts, animation, draw_scale, weapon_type, mut bank, effects: load, .. } = load_character(&mut client, &mut shared, key, &look.appearance, look.draw_scale) else { panic!("carga") };
            let crate::loader::EffectLoad { effects, bank: mut effect_bank } = *load;
            assert_eq!(effects.len(), 1);
            let base = client.emitter_class(class).unwrap();
            for (sprite, loaded) in base.sprites.iter().zip(&effects[0].1.definition.sprites) {
                if sprite.initial_particles_per_second != 0.0 && sprite.max_particles != 0 {
                    assert_eq!(loaded.max_particles, (f64::from(sprite.max_particles) * 4.5) as u32);
                }
            }
            let mut gpu = Headless::new(512, 512).unwrap();
            let mut scene_bank = TextureBank::new();
            gpu.upload(&Scene { zones: vec![ZoneLighting::default()], ..Default::default() }, &mut BankSource { bank: &mut scene_bank, catalog: client.catalog() });
            let parts = parts.into_iter().map(|mesh| {
                let id = gpu.renderer_mut().upload_character(&mesh, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
                (id, Arc::new(mesh))
            }).collect();
            let character = LoadedCharacter::new(parts, animation, draw_scale, weapon_type).unwrap().with_effects(effects, TextureBank::new());
            let pose = &character.parts[character.master].1.bind;
            let collision = Collision { radius: 8.0, height: 23.0 };
            let scale = weapon_scale(collision.radius, collision.height, 0.0);
            let instances: Vec<_> = character.parts.iter().enumerate().map(|(part, (mesh, _))| CharacterInstance {
                overlay: [0.0; 4],
                actor_id: 1,
                actor_position: Vec3::ZERO,
                collision_height: collision.height,
                shadow_caster: false,
                light_sphere: [0.0, 0.0, 20.0, 100.0],
                mesh: *mesh,
                world: character.world(part, Vec3::ZERO, 0),
                palette: character.palette(part, pose, collision).unwrap(),
                zone: 0,
                ambient: [128; 3],
                sun_affect: true,
                lights: Vec::new(),
                pawn_lights: Vec::new(),
            }).collect();
            gpu.renderer_mut().set_instances(&instances);
            let loaded = character.effect(0).unwrap();
            let mut asset = loaded.asset.clone();
            let PawnEffect::Weapon(effect) = &loaded.effect else { panic!("efeito de arma") };
            killer_assets::weapon_effects::apply_weapon_scale(&mut asset.definition, effect, scale);
            let id = gpu.renderer_mut().add_effect(&asset, &mut BankSource { bank: &mut effect_bank, catalog: client.catalog() });
            let (location, rotation) = character.effect_placement(0, pose, Vec3::ZERO, 0, scale).unwrap();
            gpu.renderer_mut().place_effect(id, EffectPlacement { location, rotation, zone: 0, visible: true });
            assert_eq!(gpu.renderer_mut().effect_count(), 1);
            gpu.renderer_mut().set_particle_delta(Some(0.1));
            let camera = killer_render::camera::Camera { eye: Vec3::new(70.0, -90.0, 45.0), target: Vec3::new(0.0, 0.0, 15.0), fov_x_deg: 45.0, near: 1.0, far: 1000.0 };
            let environment = Environment::from_time(&timeenv, 12.0);
            gpu.renderer_mut().set_effect_visible(id, false);
            let hidden = gpu.render(&camera, &environment).unwrap();
            gpu.renderer_mut().set_effect_visible(id, true);
            let mut image = Vec::new();
            for _ in 0..30 {
                image = gpu.render(&camera, &environment).unwrap();
            }
            let changed = hidden.as_chunks::<4>().0.iter().zip(image.as_chunks::<4>().0.iter()).filter(|(a, b)| a != b).count();
            let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../docs/renders/weapon-effect-{weapon}-16.png"));
            write_png(&out, 512, 512, &image).unwrap();
            eprintln!("{class}: {changed} pixels das partículas");
            assert!(changed > 20, "{class}: somente {changed} pixels");
            gpu.renderer_mut().remove_effect(id);
            assert_eq!(gpu.renderer_mut().effect_count(), 0);
        }
    }

    #[test]
    #[ignore]
    fn hero_aura_10496670_live_loader_and_gpu_capture() {
        use killer_assets::scene::{Scene, ZoneLighting};
        use killer_assets::textures::{BankSource, TextureBank, write_png};
        use killer_assets::timeenv::TimeEnv;
        use killer_render::environment::Environment;
        use killer_render::headless::Headless;
        use killer_render::renderer::EffectPlacement;
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        let timeenv = TimeEnv::load(&mut client, 0).unwrap();
        let mut shared = Shared::default();
        let plain = AppearanceKey::Player { row: 0, face: 0, hair_style: 0, hair_color: 0, equipped: Equipped::default(), enchant_effect: 0, hero: false, mounted: 0 };
        let look_plain = look(plain, &tables).unwrap();
        let LoadResult::Character { effects, .. } = load_character(&mut client, &mut shared, plain, &look_plain.appearance, look_plain.draw_scale) else { panic!("carga") };
        assert!(effects.effects.is_empty());
        let key = AppearanceKey::Player { row: 0, face: 0, hair_style: 0, hair_color: 0, equipped: Equipped::default(), enchant_effect: 0, hero: true, mounted: 0 };
        let look = look(key, &tables).unwrap();
        let LoadResult::Character { parts, animation, draw_scale, weapon_type, mut bank, effects: load, .. } = load_character(&mut client, &mut shared, key, &look.appearance, look.draw_scale) else { panic!("carga") };
        let crate::loader::EffectLoad { effects, bank: mut effect_bank } = *load;
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].0, PawnEffect::HeroAura);
        assert_eq!(effects[0].1.definition.sprites.len(), 5);
        let mut gpu = Headless::new(512, 512).unwrap();
        let mut scene_bank = TextureBank::new();
        gpu.upload(&Scene { zones: vec![ZoneLighting::default()], ..Default::default() }, &mut BankSource { bank: &mut scene_bank, catalog: client.catalog() });
        let parts = parts.into_iter().map(|mesh| {
            let id = gpu.renderer_mut().upload_character(&mesh, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
            (id, Arc::new(mesh))
        }).collect();
        let character = LoadedCharacter::new(parts, animation, draw_scale, weapon_type).unwrap().with_effects(effects, TextureBank::new());
        let pose = &character.parts[character.master].1.bind;
        let collision = Collision { radius: 8.0, height: 23.0 };
        let instances: Vec<_> = character.parts.iter().enumerate().map(|(part, (mesh, _))| CharacterInstance {
            overlay: [0.0; 4],
            actor_id: 1,
            actor_position: Vec3::ZERO,
            collision_height: collision.height,
            shadow_caster: false,
            light_sphere: [0.0, 0.0, 20.0, 100.0],
            mesh: *mesh,
            world: character.world(part, Vec3::ZERO, 0),
            palette: character.palette(part, pose, collision).unwrap(),
            zone: 0,
            ambient: [128; 3],
            sun_affect: true,
            lights: Vec::new(),
            pawn_lights: Vec::new(),
        }).collect();
        gpu.renderer_mut().set_instances(&instances);
        let loaded = character.effect(0).unwrap();
        let mut asset = loaded.asset.clone();
        killer_assets::weapon_effects::set_size_scale(&mut asset.definition, collision.radius * killer_assets::weapon_effects::HERO_AURA_SIZE_PER_RADIUS);
        let id = gpu.renderer_mut().add_effect(&asset, &mut BankSource { bank: &mut effect_bank, catalog: client.catalog() });
        let (location, rotation) = character.effect_placement(0, pose, Vec3::ZERO, 0, 1.0).unwrap();
        gpu.renderer_mut().place_effect(id, EffectPlacement { location, rotation, zone: 0, visible: false });
        gpu.renderer_mut().set_particle_delta(Some(0.1));
        let camera = killer_render::camera::Camera { eye: Vec3::new(90.0, -120.0, 60.0), target: Vec3::new(0.0, 0.0, 15.0), fov_x_deg: 45.0, near: 1.0, far: 1000.0 };
        let environment = Environment::from_time(&timeenv, 12.0);
        let hidden = gpu.render(&camera, &environment).unwrap();
        gpu.renderer_mut().set_effect_visible(id, true);
        let mut image = Vec::new();
        for _ in 0..30 {
            image = gpu.render(&camera, &environment).unwrap();
        }
        let changed = hidden.as_chunks::<4>().0.iter().zip(image.as_chunks::<4>().0.iter()).filter(|(a, b)| a != b).count();
        write_png(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/hero-aura.png"), 512, 512, &image).unwrap();
        eprintln!("aura de herói: {changed} pixels");
        assert!(changed > 100, "somente {changed} pixels");
    }

    #[test]
    #[ignore]
    fn spawn_n_transient_effect_104c84d0_soulshot_on_the_weapon_gpu_capture() {
        use killer_assets::scene::{Scene, ZoneLighting};
        use killer_assets::textures::{BankSource, TextureBank, write_png};
        use killer_assets::timeenv::TimeEnv;
        use killer_render::environment::Environment;
        use killer_render::headless::Headless;
        use killer_render::renderer::EffectPlacement;
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        let timeenv = TimeEnv::load(&mut client, 0).unwrap();
        let mut shared = Shared::default();
        let equipped = Equipped { rhand: 1, ..Default::default() };
        let key = AppearanceKey::Player { row: 0, face: 0, hair_style: 0, hair_color: 0, equipped, enchant_effect: 0, hero: false, mounted: 0 };
        let look = look(key, &tables).unwrap();
        let LoadResult::Character { parts, animation, draw_scale, weapon_type, mut bank, .. } = load_character(&mut client, &mut shared, key, &look.appearance, look.draw_scale) else { panic!("carga") };
        let mut gpu = Headless::new(512, 512).unwrap();
        let mut scene_bank = TextureBank::new();
        gpu.upload(&Scene { zones: vec![ZoneLighting::default()], ..Default::default() }, &mut BankSource { bank: &mut scene_bank, catalog: client.catalog() });
        let parts = parts.into_iter().map(|mesh| {
            let id = gpu.renderer_mut().upload_character(&mesh, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
            (id, Arc::new(mesh))
        }).collect();
        let character = LoadedCharacter::new(parts, animation, draw_scale, weapon_type).unwrap();
        let mut cache = CharacterCache::default();
        cache.insert(key, character);
        cache.spawn_shot_effects(7, key, 2150, &tables.soulshot);
        let spawns = cache.take_transient_spawns();
        assert_eq!(spawns.len(), 1);
        assert_eq!(spawns[0].class, tables.soulshot.grades[1].on_sticks);
        assert_eq!(spawns[0].stick_velocity, Some(18.0));
        let character = cache.get(key).unwrap();
        let pose = &character.parts[character.master].1.bind;
        let collision = Collision { radius: 8.0, height: 23.0 };
        let instances: Vec<_> = character.parts.iter().enumerate().map(|(part, (mesh, _))| CharacterInstance {
            overlay: [0.0; 4],
            actor_id: 7,
            actor_position: Vec3::ZERO,
            collision_height: collision.height,
            shadow_caster: false,
            light_sphere: [0.0, 0.0, 20.0, 100.0],
            mesh: *mesh,
            world: character.world(part, Vec3::ZERO, 0),
            palette: character.palette(part, pose, collision).unwrap(),
            zone: 0,
            ambient: [128; 3],
            sun_affect: true,
            lights: Vec::new(),
            pawn_lights: Vec::new(),
        }).collect();
        gpu.renderer_mut().set_instances(&instances);
        let emitter = client.emitter_class(&spawns[0].class).unwrap();
        let mut asset = killer_assets::scene::emitters::effect_asset(&mut client, &emitter, &killer_assets::options::ClientOptions::default());
        for sprite in asset.definition.sprites.iter_mut().take(2) {
            sprite.start_velocity[0] = [18.0, 18.0];
        }
        let id = gpu.renderer_mut().add_effect(&asset, &mut BankSource { bank: &mut TextureBank::new(), catalog: client.catalog() });
        let transient = &cache.transients[0];
        let world = character.bone_world(transient.bone.unwrap(), pose, Vec3::ZERO, 0).unwrap();
        let axes = Mat3::from_mat4(world);
        let axes = Mat3::from_cols(axes.x_axis.normalize(), axes.y_axis.normalize(), axes.z_axis.normalize());
        gpu.renderer_mut().place_effect(id, EffectPlacement { location: world.transform_point3(Vec3::ZERO), rotation: ortho_rotation(axes), zone: 0, visible: true });
        gpu.renderer_mut().set_particle_delta(Some(0.05));
        let camera = killer_render::camera::Camera { eye: Vec3::new(70.0, -90.0, 45.0), target: Vec3::new(0.0, 0.0, 15.0), fov_x_deg: 45.0, near: 1.0, far: 1000.0 };
        let environment = Environment::from_time(&timeenv, 12.0);
        gpu.renderer_mut().set_effect_visible(id, false);
        let hidden = gpu.render(&camera, &environment).unwrap();
        gpu.renderer_mut().set_effect_visible(id, true);
        let mut image = Vec::new();
        for _ in 0..4 {
            image = gpu.render(&camera, &environment).unwrap();
        }
        let changed = hidden.as_chunks::<4>().0.iter().zip(image.as_chunks::<4>().0.iter()).filter(|(a, b)| a != b).count();
        write_png(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/soulshot-d-small-sword.png"), 512, 512, &image).unwrap();
        eprintln!("soulshot D: {changed} pixels");
        assert!(changed > 50, "somente {changed} pixels");
        for _ in 0..60 {
            gpu.render(&camera, &environment).unwrap();
        }
        assert!(gpu.renderer_mut().effect_finished(id), "rajada AutoDestroy termina");
    }

    fn soulshot_d() -> killer_assets::shots::SoulShotData {
        killer_assets::shots::SoulShotData::parse("[D]\nAtk=LineageEffect.e_u506_c\nAtkCr=LineageEffect.e_u506_e\nAtkDarkCr=LineageEffect.e_u506_d\n")
    }

    #[test]
    #[ignore]
    fn set_atk_arrow_105f04d0_takes_the_mesh_and_texture_of_the_arrow_item_record() {
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let tables = Client::open(&root).unwrap().tables().unwrap();
        let bow = tables.weapongrp.values().find(|weapon| weapon.weapon_type == 5 && weapon.attributes_dc[0] == 2).map(|weapon| weapon.id as i32);
        let Some(bow) = bow else { panic!("sem arco de grau 2") };
        let kind = EntityKind::Player { class_id: 0, race: 0, sex: 0, equipped: [0; 11], hair_style: 0, hair_color: 0, face: 0, enchant_effect: 0, hero: false };
        let mut attacker = demo_entity(5, kind, Vec3::ZERO, 0, 20.0);
        if let EntityKind::Player { equipped, .. } = &mut attacker.kind {
            equipped[l2_net::game::packets::EQUIP_LRHAND] = bow;
        }
        let mut world = World::default();
        let model = arrow_model(&world, &attacker, &tables).expect("flecha");
        assert_eq!(model.item, 0x53e);
        assert_eq!(Some(&model.mesh), tables.etcitemgrp[&0x53e].meshes.first());
        assert_eq!(model.texture.as_ref(), tables.etcitemgrp[&0x53e].textures.first());
        let wooden = arrow_model(&World::default(), &demo_entity(6, EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true }, Vec3::ZERO, 0, 20.0), &tables).expect("flecha");
        assert_eq!((wooden.item, wooden.mesh.as_str()), (0x11, "LineageWeapons.wooden_arrow_m00_et"));
        let mut me = attacker.clone();
        me.object_id = 1;
        world.me = Some(me.clone());
        world.paperdoll[l2_net::game::packets::PAPERDOLL_L_HAND].object_id = 77;
        world.inventory.items.push(l2_net::game::InventoryItem { object_id: 77, item_id: 0x540, ..Default::default() });
        let own = arrow_model(&world, &me, &tables).expect("flecha");
        assert_eq!(own.item, 0x540);
        assert_eq!(Some(&own.mesh), tables.etcitemgrp[&0x540].meshes.first());
    }

    #[test]
    fn action_attack_105edc30_picks_the_shield_critical_soulshot_or_defense_sound_in_that_order() {
        let kind = EntityKind::Player { class_id: 0, race: 0, sex: 0, equipped: [0; 11], hair_style: 0, hair_color: 0, face: 0, enchant_effect: 0, hero: false };
        let victim = demo_entity(2, kind, Vec3::new(5.0, 6.0, 7.0), 0, 20.0);
        let tables = DatTables::default();
        let hit = |critical: bool, shield: bool, spirit: bool| AnimHit { target: 2, damage: 1, miss: false, critical, shield, spirit, soulshot_grade: 0 };
        let names = |hit: AnimHit| {
            let mut cache = CharacterCache::default();
            action_attack_sounds(&mut cache, &victim, true, hit, &tables);
            cache.take_sound_plans().into_iter().map(|plan| (plan.name, plan.position)).collect::<Vec<_>>()
        };
        let position = Vec3::new(5.0, 6.0, 7.0);
        assert_eq!(names(hit(true, false, true)), vec![("SkillSound.Critical_Hit.Critical_Hit".to_owned(), position)]);
        assert_eq!(names(hit(false, false, true)), vec![("skillsound.soul_shot_shot".to_owned(), position)]);
        assert!(names(hit(true, true, true)).is_empty());
        assert!(names(hit(false, false, false)).is_empty());
    }

    #[test]
    fn skill_effect_shot_104d4920_queues_the_strikes_and_notifies_the_emulator_found() {
        let npc = EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true };
        let caster = demo_entity(1, npc.clone(), Vec3::ZERO, 0, 20.0);
        let target = demo_entity(2, npc, Vec3::new(100.0, 0.0, 0.0), 0, 20.0);
        let find = |id: i32| [&caster, &target].into_iter().find(|entity| entity.object_id == id);
        let mut hits = Vec::new();
        skill_hits(&find, 1, 1, 2, &[], &mut hits);
        assert_eq!(hits.len(), 1);
        let (attacker, hit, kind) = hits[0];
        assert_eq!((attacker, hit.target, hit.damage, hit.critical, hit.miss, hit.shield, hit.spirit), (1, 2, 1, true, false, false, false));
        assert_eq!(kind, HitKind::Strike { notify: true });
        let mut hits = Vec::new();
        skill_hits(&find, 1, 3, 1, &[], &mut hits);
        assert_eq!(hits.len(), 1);
        assert_eq!((hits[0].1.target, hits[0].2), (1, HitKind::Notify));
        let mut hits = Vec::new();
        skill_hits(&find, 1, 2, 2, &[], &mut hits);
        skill_hits(&find, 9, 1, 2, &[], &mut hits);
        assert!(hits.is_empty());
    }

    #[test]
    fn the_effect_actor_list_received_with_the_cast_turns_on_the_head_empty_variants_of_skill_effect_shot() {
        let npc = EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true };
        let mut caster = demo_entity(1, npc.clone(), Vec3::ZERO, 0, 20.0);
        let target = demo_entity(2, npc, Vec3::new(100.0, 0.0, 0.0), 0, 20.0);
        let count = |caster: &Entity| {
            let find = |id: i32| [caster, &target].into_iter().find(|entity| entity.object_id == id);
            let mut hits = Vec::new();
            skill_hits(&find, 1, 16, 2, &[], &mut hits);
            hits.len()
        };
        assert_eq!(count(&caster), 0);
        caster.magic.current.effect_actor = vec![5];
        assert_eq!(count(&caster), 0);
        caster.magic.current.effect_actor = vec![0];
        assert!(count(&caster) > 0);
    }

    #[test]
    fn associate_attacked_notify_105ef590_places_the_effect_by_radius_or_reach() {
        let attacker = HitPawn { position: Vec3::ZERO, rotation: [0; 3], radius: 10.0, height: 20.0 };
        let victim = HitPawn { position: Vec3::new(100.0, 0.0, 0.0), rotation: [0, 777, 0], radius: 20.0, height: 30.0 };
        let (location, rotation) = associate_hit_placement(Some(&attacker), &victim);
        assert_eq!(rotation, [0, 0, 0]);
        assert!((location - Vec3::new(90.0, 0.0, 0.0)).length() < 1e-4, "{location:?}");
        let large = HitPawn { radius: 100.0, height: 60.0, ..victim };
        let (location, _) = associate_hit_placement(Some(&attacker), &large);
        assert!((location - Vec3::new(15.0, 0.0, 0.0)).length() < 1e-3, "{location:?}");
        let close = HitPawn { position: Vec3::new(50.0, 0.0, 0.0), ..large };
        let (location, _) = associate_hit_placement(Some(&attacker), &close);
        assert!((location - Vec3::new(10.0, 0.0, 0.0)).length() < 1e-3, "{location:?}");
        let edge = HitPawn { radius: 100.0, height: 50.0, ..victim };
        let (location, _) = associate_hit_placement(Some(&attacker), &edge);
        assert!((location - Vec3::new(50.0, 0.0, 0.0)).length() < 1e-3, "150 não passa do limite: {location:?}");
        let side = HitPawn { position: Vec3::new(0.0, 50.0, 0.0), radius: 8.0, ..victim };
        let (location, rotation) = associate_hit_placement(Some(&attacker), &side);
        assert_eq!(rotation, [0, 16383, 0]);
        assert!((location - (Vec3::new(0.0, 50.0, 0.0) - rotator_matrix([0, 16383, 0]).x_axis * 4.0)).length() < 1e-2, "{location:?}");
        assert_eq!(associate_hit_placement(None, &victim), (victim.position, victim.rotation));
    }

    #[test]
    fn associate_attacked_notify_105ef590_picks_damage_effect_soulshot_and_critical() {
        let data = soulshot_d();
        let context = HitContext { keep_min_frame_rate: false, rendered: true, damage_effect: Some("LineageEffect.p_u002_a") };
        let classes = |hit: AnimHit, context: &HitContext| associate_attacked_notify_classes(hit, context, &data);
        let crit = AnimHit { target: 2, critical: true, ..AnimHit::default() };
        assert_eq!(classes(crit, &context), ["LineageEffect.p_u002_a", CRITICAL_HIT_EFFECT]);
        assert_eq!(classes(AnimHit { shield: true, ..crit }, &context), ["LineageEffect.p_u002_a"]);
        assert_eq!(classes(AnimHit::default(), &context), ["LineageEffect.p_u002_a"]);
        let spirit = AnimHit { spirit: true, soulshot_grade: 1, ..AnimHit::default() };
        assert_eq!(classes(spirit, &context), ["LineageEffect.p_u002_a", "LineageEffect.e_u506_c"]);
        assert_eq!(classes(AnimHit { critical: true, ..spirit }, &context), ["LineageEffect.p_u002_a", "LineageEffect.e_u506_e"]);
        assert_eq!(classes(AnimHit { critical: true, shield: true, ..spirit }, &context), ["LineageEffect.p_u002_a", "LineageEffect.e_u506_e"]);
        assert_eq!(classes(AnimHit { soulshot_grade: 9, ..spirit }, &context), ["LineageEffect.p_u002_a"]);
        let kept = HitContext { keep_min_frame_rate: true, ..context };
        assert_eq!(classes(crit, &kept), [CRITICAL_HIT_EFFECT]);
        let bare = HitContext { damage_effect: None, ..context };
        assert_eq!(classes(spirit, &bare), ["LineageEffect.e_u506_c"]);
        let hidden = HitContext { rendered: false, ..context };
        assert!(classes(crit, &hidden).is_empty() && classes(spirit, &hidden).is_empty());
        let attacker = HitPawn { position: Vec3::ZERO, rotation: [0; 3], radius: 10.0, height: 20.0 };
        let victim = HitPawn { position: Vec3::new(100.0, 0.0, 0.0), rotation: [0; 3], radius: 20.0, height: 30.0 };
        let effects = associate_attacked_notify_effects(Some(&attacker), &victim, crit, &context, &data);
        assert_eq!(effects.len(), 2);
        assert!(effects.iter().all(|(_, location, rotation)| (*location - Vec3::new(90.0, 0.0, 0.0)).length() < 1e-4 && *rotation == [0; 3]));
    }

    #[test]
    fn attacked_notify_105ee6c0_type1_follows_the_soulshot_and_shield_branches() {
        let data = soulshot_d();
        let context = HitContext { keep_min_frame_rate: false, rendered: true, damage_effect: Some("LineageEffect.p_u002_a") };
        let classes = |hit: AnimHit, context: &HitContext| attacked_notify_ranged_classes(hit, context, &data);
        let plain = AnimHit { target: 2, spirit: true, soulshot_grade: 1, ..AnimHit::default() };
        assert_eq!(classes(plain, &context), ["LineageEffect.p_u002_a", "LineageEffect.e_u506_c"]);
        assert_eq!(classes(AnimHit { critical: true, ..plain }, &context), ["LineageEffect.p_u002_a", "LineageEffect.e_u506_e"]);
        assert_eq!(classes(AnimHit { soulshot_grade: 9, ..plain }, &context), ["LineageEffect.p_u002_a"]);
        let bare = AnimHit { target: 2, ..AnimHit::default() };
        assert_eq!(classes(bare, &context), ["LineageEffect.p_u002_a"]);
        assert_eq!(classes(AnimHit { critical: true, ..bare }, &context), ["LineageEffect.p_u002_a"]);
        assert_eq!(classes(AnimHit { shield: true, critical: true, ..bare }, &context), ["LineageEffect.p_u002_a", CRITICAL_HIT_EFFECT]);
        assert!(classes(AnimHit { shield: true, ..bare }, &context).is_empty());
        assert!(classes(AnimHit { miss: true, ..plain }, &context).is_empty());
        let hidden = HitContext { rendered: false, ..context };
        assert!(classes(plain, &hidden).is_empty());
        let kept = HitContext { keep_min_frame_rate: true, ..context };
        assert_eq!(classes(plain, &kept), ["LineageEffect.e_u506_c"]);
        let empty = killer_assets::shots::SoulShotData::default();
        assert!(attacked_notify_ranged_classes(plain, &HitContext { damage_effect: None, ..context }, &empty).is_empty());
        let effects = attacked_notify_ranged_effects(Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0), 20.0, plain, &context, &data);
        assert_eq!(effects.len(), 2);
        assert!(effects.iter().all(|(_, location, _)| (*location - Vec3::new(90.0, 0.0, 0.0)).length() < 1e-4));
        assert!(attacked_notify_ranged_effects(Vec3::ZERO, Vec3::X, 1.0, AnimHit { shield: true, ..bare }, &context, &data).is_empty());
    }

    #[test]
    fn attacked_notify_105ee6c0_type1_sets_bdemageact_unless_blocked_without_soulshot() {
        let bare = AnimHit { target: 2, ..AnimHit::default() };
        assert!(arrow_damage_act(bare, true));
        assert!(arrow_damage_act(AnimHit { critical: true, ..bare }, true));
        assert!(arrow_damage_act(AnimHit { spirit: true, shield: true, ..bare }, true));
        assert!(!arrow_damage_act(AnimHit { shield: true, ..bare }, true));
        assert!(!arrow_damage_act(AnimHit { shield: true, critical: true, ..bare }, true));
        assert!(!arrow_damage_act(AnimHit { miss: true, ..bare }, true));
        assert!(arrow_damage_act(AnimHit { shield: true, ..bare }, false));
        let mut cache = CharacterCache::default();
        cache.attacked(7);
        assert_eq!(cache.damage_acts[&7], killer_assets::damage_act::DamageAct { active: true, status: 0, dist: 0.0 });
    }

    #[test]
    fn smoke_view_shake_carries_every_add_view_shake_state_argument() {
        let shake = killer_assets::skill_smoke::smoke_shake(0x100e, Vec3::new(1.0, 2.0, 3.0), 0.0).expect("0x100e");
        let add = smoke_view_shake(&shake);
        assert_eq!((add.kind, add.time, add.scale, add.frequency, add.count, add.intensity, add.range), (5, 10.0, 1.0, 5000.0, 100.0, 100.0, 2000.0));
        assert_eq!((add.direction, add.second, add.third, add.origin), (shake.direction, shake.second, shake.third, [1.0, 2.0, 3.0]));
    }

    #[test]
    #[ignore]
    fn associate_attacked_notify_105ef590_soulshot_and_critical_hit_gpu_capture() {
        use killer_assets::scene::{Scene, ZoneLighting};
        use killer_assets::textures::{BankSource, TextureBank, write_png};
        use killer_assets::timeenv::TimeEnv;
        use killer_render::environment::Environment;
        use killer_render::headless::Headless;
        use killer_render::renderer::EffectPlacement;
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = Client::open(&root).unwrap();
        let timeenv = TimeEnv::load(&mut client, 0).unwrap();
        let mut gpu = Headless::new(512, 512).unwrap();
        let mut scene_bank = TextureBank::new();
        gpu.upload(&Scene { zones: vec![ZoneLighting::default()], ..Default::default() }, &mut BankSource { bank: &mut scene_bank, catalog: client.catalog() });
        let tables = client.tables().unwrap();
        let hit = AnimHit { target: 2, critical: true, spirit: true, soulshot_grade: 1, ..AnimHit::default() };
        let attacker = HitPawn { position: Vec3::new(-60.0, 0.0, 20.0), rotation: [0; 3], radius: 8.0, height: 24.0 };
        let victim = HitPawn { position: Vec3::new(0.0, 0.0, 20.0), rotation: [0; 3], radius: 8.0, height: 24.0 };
        let context = HitContext { keep_min_frame_rate: false, rendered: true, damage_effect: Some("LineageEffect.p_u002_a") };
        let effects = associate_attacked_notify_effects(Some(&attacker), &victim, hit, &context, &tables.soulshot);
        assert_eq!(effects.len(), 2);
        let mut ids = Vec::new();
        for (class, location, rotation) in &effects {
            let emitter = client.emitter_class(class).unwrap_or_else(|| panic!("{class}"));
            let asset = killer_assets::scene::emitters::effect_asset(&mut client, &emitter, &killer_assets::options::ClientOptions::default());
            let id = gpu.renderer_mut().add_effect(&asset, &mut BankSource { bank: &mut TextureBank::new(), catalog: client.catalog() });
            gpu.renderer_mut().place_effect(id, EffectPlacement { location: *location, rotation: *rotation, zone: 0, visible: false });
            ids.push(id);
        }
        gpu.renderer_mut().set_particle_delta(Some(0.05));
        let camera = killer_render::camera::Camera { eye: Vec3::new(40.0, -90.0, 50.0), target: Vec3::new(-4.0, 0.0, 20.0), fov_x_deg: 45.0, near: 1.0, far: 1000.0 };
        let environment = Environment::from_time(&timeenv, 12.0);
        let hidden = gpu.render(&camera, &environment).unwrap();
        for id in &ids {
            gpu.renderer_mut().set_effect_visible(*id, true);
        }
        let mut image = Vec::new();
        for _ in 0..3 {
            image = gpu.render(&camera, &environment).unwrap();
        }
        let changed = hidden.as_chunks::<4>().0.iter().zip(image.as_chunks::<4>().0.iter()).filter(|(a, b)| a != b).count();
        write_png(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/soulshot-critical-hit.png"), 512, 512, &image).unwrap();
        eprintln!("golpe crítico com soulshot: {changed} pixels");
        assert!(changed > 50, "somente {changed} pixels");
        for _ in 0..80 {
            gpu.render(&camera, &environment).unwrap();
        }
        assert!(ids.iter().all(|id| gpu.renderer_mut().effect_finished(*id)));
    }

    #[test]
    #[ignore]
    fn attacked_notify_105ee6c0_type1_arrow_soulshot_gpu_capture() {
        use killer_assets::scene::{Scene, ZoneLighting};
        use killer_assets::textures::{BankSource, TextureBank, write_png};
        use killer_assets::timeenv::TimeEnv;
        use killer_render::environment::Environment;
        use killer_render::headless::Headless;
        use killer_render::renderer::EffectPlacement;
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        let timeenv = TimeEnv::load(&mut client, 0).unwrap();
        let mut gpu = Headless::new(512, 512).unwrap();
        let mut scene_bank = TextureBank::new();
        gpu.upload(&Scene { zones: vec![ZoneLighting::default()], ..Default::default() }, &mut BankSource { bank: &mut scene_bank, catalog: client.catalog() });
        let hit = AnimHit { target: 2, critical: true, spirit: true, soulshot_grade: 1, ..AnimHit::default() };
        let context = HitContext { keep_min_frame_rate: false, rendered: true, damage_effect: None };
        let effects = attacked_notify_ranged_effects(Vec3::new(-60.0, 0.0, 20.0), Vec3::new(0.0, 0.0, 20.0), 8.0, hit, &context, &tables.soulshot);
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].0, tables.soulshot.grades[1].atk_cr);
        let mut ids = Vec::new();
        for (class, location, rotation) in &effects {
            let emitter = client.emitter_class(class).unwrap_or_else(|| panic!("{class}"));
            let asset = killer_assets::scene::emitters::effect_asset(&mut client, &emitter, &killer_assets::options::ClientOptions::default());
            let id = gpu.renderer_mut().add_effect(&asset, &mut BankSource { bank: &mut TextureBank::new(), catalog: client.catalog() });
            gpu.renderer_mut().place_effect(id, EffectPlacement { location: *location, rotation: *rotation, zone: 0, visible: false });
            ids.push(id);
        }
        gpu.renderer_mut().set_particle_delta(Some(0.05));
        let camera = killer_render::camera::Camera { eye: Vec3::new(40.0, -90.0, 50.0), target: Vec3::new(-4.0, 0.0, 20.0), fov_x_deg: 45.0, near: 1.0, far: 1000.0 };
        let environment = Environment::from_time(&timeenv, 12.0);
        let hidden = gpu.render(&camera, &environment).unwrap();
        for id in &ids {
            gpu.renderer_mut().set_effect_visible(*id, true);
        }
        let mut image = Vec::new();
        for _ in 0..3 {
            image = gpu.render(&camera, &environment).unwrap();
        }
        let changed = hidden.as_chunks::<4>().0.iter().zip(image.as_chunks::<4>().0.iter()).filter(|(a, b)| a != b).count();
        write_png(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/soulshot-arrow-hit.png"), 512, 512, &image).unwrap();
        eprintln!("golpe de flecha crítico com soulshot D: {changed} pixels");
        assert!(changed > 50, "somente {changed} pixels");
        for _ in 0..80 {
            gpu.render(&camera, &environment).unwrap();
        }
        assert!(ids.iter().all(|id| gpu.renderer_mut().effect_finished(*id)));
    }

    #[test]
    #[ignore]
    fn enchant_live_loader_and_gpu_capture() {
        use killer_assets::scene::{Scene, ZoneLighting};
        use killer_assets::textures::{BankSource, TextureBank, write_png};
        use killer_assets::timeenv::TimeEnv;
        use killer_render::environment::Environment;
        use killer_render::headless::Headless;
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        let timeenv = TimeEnv::load(&mut client, 0).unwrap();
        let mut gpu = Headless::new(512, 512).unwrap();
        let mut bank = TextureBank::new();
        gpu.upload(&Scene { zones: vec![ZoneLighting::default()], ..Default::default() }, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        let mut shared = Shared::default();
        let mut captures = Vec::new();
        for level in [0, 20] {
            let equipped = Equipped { rhand: 1, lhand: 18, ..Default::default() };
            let key = AppearanceKey::Player { row: 0, face: 0, hair_style: 0, hair_color: 0, equipped, enchant_effect: level, hero: false, mounted: 0 };
            let look = look(key, &tables).unwrap();
            let LoadResult::Character { parts, animation, draw_scale, weapon_type, mut bank, .. } = load_character(&mut client, &mut shared, key, &look.appearance, look.draw_scale) else { panic!("carga") };
            let auras = parts.iter().filter(|p| matches!(p.placement, Placement::Enchanted { .. })).count();
            assert_eq!(auras, usize::from(level == 20));
            for part in parts.iter().filter(|p| matches!(p.placement, Placement::Enchanted { .. })) {
                assert!(part.sections.iter().flat_map(|section| &section.material.passes).all(|pass| !pass.lit), "0x10690059 EnableLighting(0,0,0,0,0)");
            }
            let parts = parts.into_iter().map(|mesh| {
                let id = gpu.renderer_mut().upload_character(&mesh, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
                (id, Arc::new(mesh))
            }).collect();
            let character = LoadedCharacter::new(parts, animation, draw_scale, weapon_type).unwrap();
            let pose = &character.parts[character.master].1.bind;
            let collision = Collision { radius: 8.0, height: 23.0 };
            let instances: Vec<_> = character.parts.iter().enumerate().map(|(part, (mesh, _))| CharacterInstance {
                overlay: [0.0; 4],
                actor_id: 1,
                actor_position: Vec3::ZERO,
                collision_height: collision.height,
                shadow_caster: false,
                light_sphere: [0.0, 0.0, 20.0, 100.0],
                mesh: *mesh,
                world: character.world(part, Vec3::ZERO, 0),
                palette: character.palette(part, pose, collision).unwrap(),
                zone: 0,
                ambient: [128; 3],
                sun_affect: true,
                lights: Vec::new(),
                pawn_lights: Vec::new(),
            }).collect();
            gpu.renderer_mut().set_instances(&instances);
            let camera = killer_render::camera::Camera { eye: Vec3::new(90.0, -120.0, 60.0), target: Vec3::new(0.0, 0.0, 15.0), fov_x_deg: 45.0, near: 1.0, far: 1000.0 };
            let image = gpu.render(&camera, &Environment::from_time(&timeenv, 12.0)).unwrap();
            let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../docs/renders/enchant-audit-{level}.png"));
            write_png(&out, 512, 512, &image).unwrap();
            captures.push(image);
        }
        let changed = captures[0].as_chunks::<4>().0.iter().zip(captures[1].as_chunks::<4>().0.iter()).filter(|(a, b)| a != b).count();
        assert!(changed > 100, "somente {changed} pixels mudaram");
        eprintln!("{changed} pixels alterados pelo enchant no pipeline real");
    }

    #[test]
    #[ignore]
    fn killer_robe_renders_all_player_rows_with_every_equipped_part() {
        use killer_assets::scene::{Scene, ZoneLighting};
        use killer_assets::textures::{BankSource, TextureBank, write_png};
        use killer_assets::timeenv::TimeEnv;
        use killer_render::environment::Environment;
        use killer_render::headless::Headless;
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        let timeenv = TimeEnv::load(&mut client, 0).unwrap();
        let mut gpu = Headless::new(512, 512).unwrap();
        let mut bank = TextureBank::new();
        gpu.upload(&Scene { zones: vec![ZoneLighting::default()], ..Default::default() }, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        let mut shared = Shared::default();
        let out_dir=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/preparation-validation/killer-robe");
        std::fs::create_dir_all(&out_dir).unwrap();
        for row in 0..14 {
            let equipped = Equipped { head: 8063, chest: 8064, legs: 8065, gloves: 8066, feet: 8067, ..Default::default() };
            let key = AppearanceKey::Player { row, face: 0, hair_style: 0, hair_color: 0, equipped, enchant_effect: 0, hero: false, mounted: 0 };
            let look = look(key, &tables).unwrap();
            let LoadResult::Character { parts, animation, draw_scale, weapon_type, mut bank, .. } = load_character(&mut client, &mut shared, key, &look.appearance, look.draw_scale) else { panic!("carga") };
            assert_eq!(parts.len(),look.appearance.parts.len(),"row {row}: parte descartada");
            for (id,slot) in [(8067,0),(8065,1),(8064,2),(8066,3)] {
                let declared=tables.armorgrp[&id].rows[row].meshes.first().filter(|name|!name.is_empty());
                assert_eq!(parts.iter().any(|part|part.slot==slot),declared.is_some(),"row {row} slot {slot}");
            }
            let parts = parts.into_iter().map(|mesh| {
                let id = gpu.renderer_mut().upload_character(&mesh, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
                (id, Arc::new(mesh))
            }).collect();
            let character = LoadedCharacter::new(parts, animation, draw_scale, weapon_type).unwrap();
            let pose = &character.parts[character.master].1.bind;
            let collision = Collision { radius: 8.0, height: 23.0 };
            let instances: Vec<_> = character.parts.iter().enumerate().map(|(part, (mesh, _))| CharacterInstance {
                overlay: [0.0; 4],
                actor_id: 1,
                actor_position: Vec3::ZERO,
                collision_height: collision.height,
                shadow_caster: false,
                light_sphere: [0.0, 0.0, 20.0, 100.0],
                mesh: *mesh,
                world: character.world(part, Vec3::ZERO, 0),
                palette: character.palette(part, pose, collision).unwrap(),
                zone: 0,
                ambient: [128; 3],
                sun_affect: true,
                lights: Vec::new(),
                pawn_lights: Vec::new(),
            }).collect();
            gpu.renderer_mut().set_instances(&instances);
            let camera = killer_render::camera::Camera { eye: Vec3::new(90.0, -120.0, 60.0), target: Vec3::new(0.0, 0.0, 15.0), fov_x_deg: 45.0, near: 1.0, far: 1000.0 };
            let image = gpu.render(&camera, &Environment::from_time(&timeenv, 12.0)).unwrap();
            let out = out_dir.join(format!("row-{row:02}.png"));
            write_png(&out, 512, 512, &image).unwrap();
            for (id,_) in &character.parts {gpu.renderer_mut().remove_character(*id);}
            client.catalog().release_files();
        }
    }

    #[test]
    fn enchanted_transform_10680610_uses_dat_vectors_and_scaled_master_origin() {
        let mut master = CharacterMesh::empty();
        master.scale = Vec3::new(2.0, 3.0, 4.0);
        master.origin = Vec3::new(1.0, 2.0, 3.0);
        let mut aura = CharacterMesh::empty();
        aura.origin = Vec3::new(2.0, 4.0, 6.0);
        let pose = Mat4::from_translation(Vec3::new(11.0, 22.0, 33.0)) * Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let matrix = enchanted_palette(pose, &master, 2.0, &aura, Vec3::new(3.0, 4.0, 5.0), Vec3::new(6.0, 7.0, 8.0));
        let point = matrix.transform_point3(Vec3::new(1.0, 2.0, 3.0));
        assert!((point - Vec3::new(12.0, 138.0, 280.0)).length() < 0.0001);
    }

    #[test]
    #[ignore]
    fn enchant_appearances_follow_every_weapon_record_without_shield_auras() {
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = killer_assets::client::Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        let mut checked = 0;
        let mut effects = 0;
        for weapon in tables.weapongrp.values() {
            assert_eq!(weapon.effects.len(), weapon.meshes.len(), "{}", weapon.id);
            assert_eq!(weapon.enchant_meshes.len(), weapon.meshes.len(), "{}", weapon.id);
            let equipped = if weapon.body_part == l2_dat::armorgrp::body_part::LHAND {
                Equipped { lhand: weapon.id as i32, ..Default::default() }
            } else if weapon.body_part == l2_dat::armorgrp::body_part::LRHAND {
                Equipped { lrhand: weapon.id as i32, ..Default::default() }
            } else {
                Equipped { rhand: weapon.id as i32, ..Default::default() }
            };
            let key = AppearanceKey::Player { row: 0, face: 0, hair_style: 0, hair_color: 0, equipped, enchant_effect: 20, hero: false, mounted: 0 };
            let appearance = look(key, &tables).unwrap().appearance;
            for part in &appearance.parts {
                let Placement::Enchanted { scale, offset, hero, .. } = part.placement else { continue };
                assert!([SLOT_RIGHT_HAND, SLOT_LEFT_HAND].contains(&part.slot));
                let index = usize::from(part.slot == SLOT_LEFT_HAND && weapon.handness != l2_dat::weapongrp::anim_type::BOW);
                let data = &weapon.enchant_meshes[index];
                assert_eq!(part.mesh, data.name);
                assert_eq!(scale, data.vector_124);
                assert_eq!(offset, data.vector_154);
                assert_eq!(hero, weapon.hero_weapon >= 0);
                checked += 1;
            }
            if weapon.body_part == l2_dat::armorgrp::body_part::LHAND && weapon.handness == l2_dat::weapongrp::anim_type::HAND {
                assert!(!appearance.parts.iter().any(|part| matches!(part.placement, Placement::Enchanted { .. })), "escudo {}", weapon.id);
                assert!(appearance.weapon_effects.is_empty(), "escudo {}", weapon.id);
            }
            let mut expected = Vec::new();
            for (slot, bone) in [(SLOT_RIGHT_HAND, RIGHT_HAND_BONE), (SLOT_LEFT_HAND, LEFT_HAND_BONE)] {
                if appearance.part(slot).is_none() {
                    continue;
                }
                let index = usize::from(slot == SLOT_LEFT_HAND && weapon.handness != l2_dat::weapongrp::anim_type::BOW);
                let Some(data) = weapon.effects.get(index).filter(|e| !e.name.is_empty() && !e.name.eq_ignore_ascii_case("None")) else { continue };
                expected.push(WeaponEffect { class: data.name.clone(), bone, right: slot == SLOT_RIGHT_HAND, offset: data.offset, size_scale: data.parameter_1c4, velocity_scale: data.parameter_1d4, hero: weapon.hero_weapon >= 0 });
            }
            effects += expected.len();
            assert_eq!(appearance.weapon_effects, expected, "{}", weapon.id);
        }
        assert!(checked > 0 && effects > 0);
        eprintln!("{} armas verificadas, {checked} malhas de enchant, {effects} efeitos de partícula", tables.weapongrp.len());
    }

    struct LobbySeqs;

    impl Sequences for LobbySeqs {
        fn seq(&self, name: &str) -> Option<SeqInfo> {
            (name.starts_with("Social_SpWait") || name.starts_with("Wait")).then(|| SeqInfo::new(30, 15.0))
        }
    }

    #[test]
    fn the_lobby_wait_replaces_the_looping_wait_keeps_a_social_running_and_returns_afterwards() {
        let seqs = LobbySeqs;
        let mut channel = Channel::default();
        channel.play("Wait_MFighter", &SeqInfo::new(30, 15.0), 1.0, 0.0, true);
        lobby_wait(&mut channel, "Social_SpWait01_MFighter", None, &seqs);
        assert_eq!(channel.sequence.as_deref(), Some("Social_SpWait01_MFighter"));
        assert!(channel.looping);
        lobby_wait(&mut channel, "Social_SpWait01_MFighter", None, &seqs);
        assert_eq!(channel.sequence.as_deref(), Some("Social_SpWait01_MFighter"));
        lobby_wait(&mut channel, "Social_SpWait01_MFighter", Some("Social_SpWait02_MFighter"), &seqs);
        assert_eq!(channel.sequence.as_deref(), Some("Social_SpWait02_MFighter"));
        assert!(!channel.looping);
        for _ in 0..10 {
            channel.advance(0.1);
        }
        lobby_wait(&mut channel, "Social_SpWait01_MFighter", None, &seqs);
        assert_eq!(channel.sequence.as_deref(), Some("Social_SpWait02_MFighter"));
        while !channel.finished {
            channel.advance(0.1);
        }
        lobby_wait(&mut channel, "Social_SpWait01_MFighter", None, &seqs);
        assert_eq!(channel.sequence.as_deref(), Some("Social_SpWait01_MFighter"));
        let mut missing = Channel::default();
        lobby_wait(&mut missing, "Nope", None, &seqs);
        assert_eq!(missing.sequence, None);
    }

    use super::*;
    use crate::loader::{LoadResult, Shared, load_character};
    use killer_assets::client::Client;
    use l2_dat::animnames::AnimKind;
    use std::path::PathBuf;

    fn player_kind(class_id: i32, race: i32, sex: i32, face: i32) -> EntityKind {
        EntityKind::Player {
            class_id,
            race,
            sex,
            equipped: [0; VISIBLE_EQUIP_SLOTS],
            hair_style: 0,
            hair_color: 0,
            face,
            enchant_effect: 0,
            hero: false,
        }
    }

    fn entity(kind: EntityKind, position: Vec3) -> Entity {
        Entity {
            item_origin: None,
            abnormal_effect: 0,
            object_id: 1,
            kind,
            name: String::new(),
            title: String::new(),
            position,
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
            walk_anim_frame_in_move: DEFAULT_WALK_ANIM_FRAME_IN_MOVE,
            hp: None,
            mp: None,
            cp: None,
            level: 0,
            exp: 0,
            collision_radius: 8.0,
            collision_height: 24.0,
            base_collision: [0.0; 2],
            karma: 0,
            pvp_flag: 0,
            relation: 0, auto_attack: false, surrender_war_id: 0,
            target: None,
            casting: None,
            magic: Default::default(),
            anim: spawned_anim(),
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

    fn gremlin() -> Npc {
        Npc {
            id: 1,
            class: "LineageMonster.gremlin".into(),
            mesh: "LineageMonsters.gremlin_m00".into(),
            textures: vec!["LineageMonstersTex.gremlin_t00".into()],
            textures2: vec![],
            speed: 1.0,
            effect: String::new(),
            level_min: 0,
            level_max: 0,
            attack_sounds: vec![],
            defense_sounds: vec![],
            damage_sounds: vec![],
            sound_radius: 0.0,
            sound_volume: 0.0,
            sound_random: 0.0, skills: vec![], shows_bars: 0,
        }
    }

    fn empty_tables() -> DatTables {
        DatTables::default()
    }

    fn player_key(row: usize, face: i32) -> AppearanceKey {
        AppearanceKey::Player { row, face, hair_style: 0, hair_color: 0, equipped: Equipped::default(), enchant_effect: 0, hero: false, mounted: 0 }
    }

    fn npc_key(npc_id: i32) -> AppearanceKey {
        AppearanceKey::Npc { npc_id, equipped: Equipped::default() }
    }

    fn camera() -> Camera {
        Camera { eye: Vec3::new(0.0, -300.0, 0.0), target: Vec3::ZERO, fov_x_deg: 60.0, near: 10.0, far: 60000.0 }
    }

    fn display() -> DisplaySettings {
        DisplaySettings {
            show_names: true,
            name_dist: 1000,
            pawn_clipping: 1.5,
            render_limit: 10000,
            ..DisplaySettings::default()
        }
    }

    fn frame(dt: f32) -> Frame {
        Frame { display: display(), viewer: Vec3::ZERO, dt }
    }

    #[test]
    fn display_settings_come_from_l2_ini_and_option_ini_with_the_core_defaults() {
        let l2 = killer_ui::ini::parse_ini("[CharacterDisplay]\nName=true\nDist=800\n[URL]\nIsL2HideName=True\n");
        let option = killer_ui::ini::parse_ini(
            "[Video]\nIsKeepMinFrameRate=false\nRenderActorLimited=3\n[ClippingRange]\nPawn=1.500000\n[Game]\nMyName=False\nNPCName=False\nOtherPCName=True\n",
        );
        let settings = DisplaySettings::from_ini(&l2, &option);
        assert!(settings.show_names);
        assert_eq!(settings.name_dist, 800);
        assert_eq!(settings.pawn_clipping, 1.5);
        assert_eq!(settings.pawn_range(), 3072.0);
        assert_eq!(settings.render_limit, 200);
        assert!(
            !settings.show_my_name && !settings.show_npc_name && settings.show_other_pc_name && settings.hide_names
        );
        let empty = killer_ui::ini::parse_ini("");
        let defaults = DisplaySettings::from_ini(&empty, &empty);
        assert_eq!(defaults, DisplaySettings::default());
        assert_eq!((defaults.pawn_clipping, defaults.render_limit, defaults.name_dist), (2.0, 10000, 1000));
        assert!(!defaults.show_names && defaults.show_my_name && defaults.show_npc_name && !defaults.hide_names);
        let slow = killer_ui::ini::parse_ini("[Video]\nIsKeepMinFrameRate=true\n[ClippingRange]\nPawn=3.0\n");
        assert_eq!(DisplaySettings::from_ini(&empty, &slow).pawn_clipping, 0.7);
        assert_eq!(
            [
                render_actor_limit(0),
                render_actor_limit(1),
                render_actor_limit(2),
                render_actor_limit(4),
                render_actor_limit(5),
                render_actor_limit(6)
            ],
            [50, 50, 100, 300, 400, 10000]
        );
        assert!(settings.pawn_in_range(Vec3::ZERO, Vec3::new(3000.0, 0.0, 5000.0)));
        assert!(!settings.pawn_in_range(Vec3::ZERO, Vec3::new(3072.0, 0.0, 0.0)));
        assert!(settings.name_in_range(Vec3::ZERO, Vec3::new(0.0, 0.0, 799.9)));
        assert!(!settings.name_in_range(Vec3::ZERO, Vec3::new(0.0, 0.0, 800.0)));
        assert!(settings.shows_name_of(false, true, false) && !settings.shows_name_of(false, true, true));
        assert!(!settings.shows_name_of(true, false, false) && settings.shows_name_of(false, false, false));
    }

    #[test]
    fn pawns_beyond_the_clipping_range_or_past_the_render_limit_are_not_drawn() {
        let mut tables = empty_tables();
        let mut antaras = gremlin();
        antaras.id = 2;
        antaras.class = "LineageMonster.antaras".into();
        tables.npcgrp = [gremlin(), antaras].into_iter().collect();
        let mut world = World { me: Some(entity(player_kind(0, 0, 0, 0), Vec3::ZERO)), ..World::default() };
        let spawn = |world: &mut World, id: i32, kind: EntityKind, x: f32| {
            let mut e = entity(kind, Vec3::new(x, 0.0, 0.0));
            e.object_id = id;
            e.spawned = id as u64;
            world.entities.insert(id, e);
        };
        spawn(&mut world, 1, player_kind(0, 0, 0, 0), 100.0);
        spawn(&mut world, 2, player_kind(0, 0, 0, 0), 3100.0);
        spawn(&mut world, 3, EntityKind::Npc { npc_id: 2, rhand: 0, chest: 0, lhand: 0, attackable: true }, 9000.0);
        spawn(&mut world, 4, EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true }, 200.0);
        spawn(&mut world, 5, player_kind(0, 0, 0, 0), 300.0);
        let all = drawn_pawns(&world, &tables, &frame(0.0));
        assert_eq!(all, [5, 4, 3, 1]);
        let limited = DisplaySettings { render_limit: 2, ..display() };
        let two = drawn_pawns(&world, &tables, &Frame { display: limited, viewer: Vec3::ZERO, dt: 0.0 });
        assert_eq!(two, [5, 4, 3]);
        let far = Frame { display: display(), viewer: Vec3::new(6000.0, 0.0, 0.0), dt: 0.0 };
        assert_eq!(drawn_pawns(&world, &tables, &far), [3, 2]);
        assert!(
            ignores_range(Some("LineageNpc.Broadcasting_Tower"))
                && !ignores_range(Some("LineageMonster.gremlin"))
                && !ignores_range(None)
        );
    }

    #[test]
    fn relation_bits_pick_the_pledge_icons_the_way_draw_target_name_does() {
        let icons = |bits: i32, me: bool| relation_icons(bits, me);
        let only = |base: &'static str| RelationIcons { base: Some(base), leader: None };
        assert_eq!(icons(0, false), RelationIcons::default());
        assert_eq!(icons(0x10000, false), only(WAR_SINGLE_ICON));
        assert_eq!(icons(0x10000 | 0x8000, false), only(WAR_DOUBLE_ICON));
        assert_eq!(icons(0x8000, false), RelationIcons::default());
        assert_eq!(icons(0x800, false), RelationIcons::default());
        assert_eq!(icons(0x800 | 0x200, false), only(BLUE_SHIELD_ICON));
        assert_eq!(icons(0x800 | 0x200 | 0x400, false), only(BLUE_SWORD_ICON));
        assert_eq!(icons(0x800 | 0x200 | 0x80, false), RelationIcons { base: Some(BLUE_SHIELD_ICON), leader: Some(BLUE_LEADER_ICON) });
        assert_eq!(icons(0x800 | 0x200 | 0x400 | 0x80, false), RelationIcons { base: Some(BLUE_SWORD_ICON), leader: Some(BLUE_ATTACK_LEADER_ICON) });
        assert_eq!(icons(0x1000 | 0x200, false), only(RED_SHIELD_ICON));
        assert_eq!(icons(0x1000 | 0x200 | 0x400, false), only(RED_SWORD_ICON));
        assert_eq!(icons(0x1000 | 0x200 | 0x80, false), RelationIcons { base: Some(RED_SHIELD_ICON), leader: Some(RED_LEADER_ICON) });
        assert_eq!(icons(0x1000 | 0x200 | 0x400 | 0x80, false), RelationIcons { base: Some(RED_SWORD_ICON), leader: Some(RED_ATTACK_LEADER_ICON) });
        assert_eq!(icons(0x1000 | 0x400, false), RelationIcons::default());
        assert_eq!(icons(0x800 | 0x1000 | 0x200, false), only(BLUE_SHIELD_ICON));
        assert_eq!(icons(0x800 | 0x10000, false), RelationIcons::default());
        assert_eq!(icons(0x1000 | 0x10000 | 0x8000, false), RelationIcons::default());
    }

    #[test]
    fn the_local_player_reads_the_self_bits_of_the_relation() {
        let only = |base: &'static str| RelationIcons { base: Some(base), leader: None };
        assert_eq!(relation_icons(0, true), RelationIcons::default());
        assert_eq!(relation_icons(0x10000 | 0x200 | 0x800, true), RelationIcons::default());
        assert_eq!(relation_icons(0x80, true), only(BLUE_SHIELD_ICON));
        assert_eq!(relation_icons(0x80 | 0x100, true), only(BLUE_SWORD_ICON));
        assert_eq!(relation_icons(0x80 | 0x40, true), RelationIcons { base: Some(BLUE_SHIELD_ICON), leader: Some(BLUE_LEADER_ICON) });
        assert_eq!(relation_icons(0x80 | 0x100 | 0x40, true), RelationIcons { base: Some(BLUE_SWORD_ICON), leader: Some(BLUE_ATTACK_LEADER_ICON) });
        assert_eq!(relation_icons(0x100 | 0x40, true), RelationIcons::default());
    }

    #[test]
    fn the_relation_icon_sits_left_of_the_title_and_pushes_it_sixteen_pixels() {
        let mut label = Label { object_id: 1, name: "A".into(), title: "T".into(), store_sign: None, speech: None, modern_text_size: None, name_visible: true, name_color: WHITE, title_color: WHITE, screen: [400.0, 300.0], depth: 0.5, clan_crest: None, ally_crest: None, mark: None, relation_icons: RelationIcons::default(), broadcast_icon: None };
        assert_eq!(label.relation_origin(400, 300, 40, 14), None);
        assert_eq!(label.title_left(400, 40, None), 380);
        label.relation_icons = relation_icons(0x10000, false);
        let origin = label.relation_origin(400, 300, 40, 14);
        assert_eq!(origin, Some((400 - 28, 300 - 15 - 12)));
        assert_eq!(label.title_left(400, 40, origin), 400 - 28 + 16);
        label.name_visible = false;
        assert_eq!(label.relation_origin(400, 300, 40, 14), Some((392, 300 - 15 - 2)));
        label.clan_crest = Some("crest.crest_1".into());
        assert_eq!(label.relation_origin(400, 300, 40, 14), Some((392, 300 - 28)));
        label.mark = Some(TargetMark::Target);
        assert_eq!(label.relation_origin(400, 300, 41, 14), Some((400 - 28, 300 - 15 - 12)));
    }

    #[test]
    fn the_arrow_scale_is_the_square_root_of_height_times_radius_over_the_square_root_of_207() {
        assert_eq!(arrow_scale(9.0, 23.0), 1.0);
        assert!((arrow_scale(12.0, 30.0) - (360.0f32 / 207.0).sqrt()).abs() < 1e-6);
        assert!(arrow_scale(0.0, 20.0) == 0.0);
    }

    #[test]
    fn broadcast_marked_pawns_get_the_party_icon_and_lose_name_title_and_mark_until_the_name_toggle() {
        use killer_world::appendix::AppendixOp;
        let mut tables = empty_tables();
        tables.npcgrp = std::iter::once(gremlin()).collect();
        let mut world = World::default();
        let mut me = entity(player_kind(0, 0, 0, 0), Vec3::ZERO);
        me.object_id = 99;
        world.me = Some(me);
        for id in [1, 2] {
            let mut other = entity(player_kind(0, 0, 0, 0), Vec3::new(300.0 * id as f32, 0.0, 0.0));
            other.object_id = id;
            other.name = format!("P{id}");
            world.entities.insert(id, other);
        }
        world.apply_appendix(AppendixOp::Flag { object_id: 1, set: true });
        world.apply_appendix(AppendixOp::Index { object_id: 1, team: 1, index: 4 });
        let camera = Camera { eye: Vec3::new(500.0, -3000.0, 0.0), target: Vec3::new(500.0, 0.0, 0.0), fov_x_deg: 60.0, near: 10.0, far: 60000.0 };
        let frame = Frame { display: display(), viewer: Vec3::ZERO, dt: 0.0 };
        let run = |world: &World| labels(world, &tables, &CharacterCache::default(), &frame, &camera, (1280.0, 720.0), None);
        let find = |labels: &[Label], id: i32| labels.iter().find(|label| label.object_id == id).cloned();
        assert!(find(&run(&world), 1).unwrap().broadcast_icon.is_none());
        world.appendix.canvas_start();
        let marked = find(&run(&world), 1).unwrap();
        assert_eq!(marked.broadcast_icon, Some(BroadcastIcon { team: 1, index: 4, names_on: true }));
        assert!(marked.name_visible);
        world.appendix.canvas_toggle_target_name();
        let marked = find(&run(&world), 1).unwrap();
        assert_eq!(marked.broadcast_icon, Some(BroadcastIcon { team: 1, index: 4, names_on: false }));
        assert!(!marked.name_visible && marked.mark.is_none() && marked.title.is_empty());
        assert!(find(&run(&world), 2).unwrap().name_visible);
        assert_eq!(marked.broadcast_icon.unwrap().texture(), "L2UI_CH3.BroadcastObs.br_party2_5");
        assert_eq!(BroadcastIcon { team: 0, index: 0, names_on: true }.origin(100, 200, 185), (78, 131));
        assert_eq!(BroadcastIcon { team: 0, index: 0, names_on: false }.origin(100, 200, 185), (78, 146));
    }

    #[test]
    fn labels_follow_the_character_display_distance_the_target_and_the_option_flags() {
        let mut tables = empty_tables();
        tables.npcgrp = std::iter::once(gremlin()).collect();
        let mut world = World::default();
        let mut me = entity(player_kind(0, 0, 0, 0), Vec3::ZERO);
        me.name = "Wender".into();
        me.object_id = 99;
        world.me = Some(me);
        for (id, x, kind) in [
            (1, 300.0, player_kind(0, 0, 0, 0)),
            (2, 999.9, player_kind(0, 0, 0, 0)),
            (3, 1000.0, player_kind(0, 0, 0, 0)),
            (4, 500.0, EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true }),
            (5, 600.0, EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: false }),
        ] {
            let mut other = entity(kind, Vec3::new(x, 0.0, 0.0));
            other.object_id = id;
            other.name = format!("P{id}");
            world.entities.insert(id, other);
        }
        let camera = Camera {
            eye: Vec3::new(500.0, -3000.0, 0.0),
            target: Vec3::new(500.0, 0.0, 0.0),
            fov_x_deg: 60.0,
            near: 10.0,
            far: 60000.0,
        };
        let names = |world: &World, display: DisplaySettings| -> Vec<String> {
            let frame = Frame { display, viewer: Vec3::ZERO, dt: 0.0 };
            let mut names: Vec<String> =
                labels(world, &tables, &CharacterCache::default(), &frame, &camera, (1280.0, 720.0), None)
                    .into_iter()
                    .filter(|l| l.name_visible)
                    .map(|l| l.name)
                    .collect();
            names.sort();
            names
        };
        assert_eq!(names(&world, display()), ["P1", "P2", "P4", "P5", "Wender"]);
        assert!(world.select(3));
        assert_eq!(names(&world, display()), ["P1", "P2", "P3", "P4", "P5", "Wender"]);
        assert_eq!(names(&world, DisplaySettings { show_names: false, ..display() }), ["P3"]);
        assert_eq!(
            names(&world, DisplaySettings { name_dist: 1001, ..display() }),
            ["P1", "P2", "P3", "P4", "P5", "Wender"]
        );
        assert_eq!(
            names(&world, DisplaySettings { show_npc_name: false, ..display() }),
            ["P1", "P2", "P3", "P5", "Wender"]
        );
        assert_eq!(names(&world, DisplaySettings { show_other_pc_name: false, ..display() }), ["P4", "P5", "Wender"]);
        assert_eq!(names(&world, DisplaySettings { show_my_name: false, ..display() }), ["P1", "P2", "P3", "P4", "P5"]);
        assert!(names(&world, DisplaySettings { hide_names: true, ..display() }).is_empty());
        assert_eq!(names(&world, DisplaySettings { pawn_clipping: 0.2, ..display() }), ["P1", "Wender"]);
        let marks = |world: &World, hovered: Option<i32>| -> Vec<(i32, Option<TargetMark>)> {
            let frame = Frame { display: display(), viewer: Vec3::ZERO, dt: 0.0 };
            let mut marks: Vec<_> = labels(world, &tables, &CharacterCache::default(), &frame, &camera, (1280.0, 720.0), hovered)
                .into_iter().map(|label| (label.object_id, label.mark)).collect();
            marks.sort_by_key(|(id, _)| *id);
            marks
        };
        world.target = None;
        let hovered = marks(&world, Some(3));
        assert!(hovered.contains(&(3, Some(TargetMark::Normal))));
        assert!(hovered.iter().filter(|(id, _)| *id != 3).all(|(_, mark)| mark.is_none()));
        world.target = Some(3);
        assert!(marks(&world, Some(3)).contains(&(3, Some(TargetMark::Target))));
        world.attack_intend = true;
        assert!(marks(&world, Some(1)).contains(&(3, Some(TargetMark::Attack))));
        assert!(marks(&world, Some(1)).contains(&(1, Some(TargetMark::Normal))));
    }

    #[test]
    fn appearance_key_is_the_npc_id_or_the_chargrp_row_and_face() {
        assert_eq!(
            appearance_key(&EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: false }),
            Some(npc_key(1))
        );
        assert_eq!(
            appearance_key(&EntityKind::Npc { npc_id: 1, rhand: 5, chest: 0, lhand: 6, attackable: false }),
            Some(AppearanceKey::Npc { npc_id: 1, equipped: Equipped { rhand: 5, lhand: 6, ..Default::default() } })
        );
        assert_eq!(appearance_key(&player_kind(0, 0, 0, 2)), Some(player_key(0, 2)));
        assert_eq!(appearance_key(&player_kind(97, 0, 1, 0)), Some(player_key(9, 0)));
        assert_eq!(appearance_key(&player_kind(49, 3, 0, 1)), Some(player_key(12, 1)));
        assert_eq!(appearance_key(&player_kind(0, 9, 0, 0)), None);
    }

    #[test]
    fn a_mounted_player_gets_the_mounted_key_and_the_mount_npc_key_and_the_label_lift() {
        let mut rider = entity(player_kind(0, 0, 0, 0), Vec3::ZERO);
        assert_eq!(entity_key(&rider), Some(player_key(0, 0)));
        assert_eq!(mount_key(&rider), None);
        rider.mount = Some(killer_world::Mount { ride_type: 1, ride_class_id: 1_012_526 });
        assert!(matches!(entity_key(&rider), Some(AppearanceKey::Player { mounted: 1, .. })));
        assert_eq!(mount_key(&rider), Some(npc_key(12_526)));
        rider.mount = Some(killer_world::Mount { ride_type: 2, ride_class_id: 1_012_527 });
        assert!(matches!(entity_key(&rider), Some(AppearanceKey::Player { mounted: 2, .. })));
        rider.mount = Some(killer_world::Mount { ride_type: 3, ride_class_id: 1 });
        assert_eq!(mount_key(&rider), None);
    }

    #[test]
    fn look_falls_back_to_the_giant_rabbit_for_unknown_npcs_and_none_for_unknown_rows() {
        let tables = empty_tables();
        let rabbit = look(npc_key(424242), &tables).unwrap();
        assert_eq!(rabbit.appearance, fallback_npc_appearance());
        assert_eq!(rabbit.draw_scale, NPC_FALLBACK.draw_scale);
        assert_eq!(look(npc_key(-1), &tables).unwrap().draw_scale, NPC_FALLBACK.draw_scale);
        assert!(look(player_key(0, 0), &tables).is_none());
    }

    #[test]
    fn world_matrix_turns_plus_x_into_plus_y_at_heading_16384() {
        let m = world_matrix(Vec3::ZERO, 16384, 1.0, Mat4::IDENTITY);
        let x = m.transform_vector3(Vec3::X);
        assert!((x - Vec3::Y).length() < 1e-5, "{x}");
        let m = world_matrix(Vec3::new(10.0, 20.0, 30.0), 0, 10.0, Mat4::from_translation(Vec3::Z));
        let p = m.transform_point3(Vec3::X);
        assert!((p - Vec3::new(20.0, 20.0, 40.0)).length() < 1e-4, "{p}");
    }

    fn synthetic_bones() -> Vec<Bone> {
        vec![
            Bone { name: "Bip01".into(), parent: 0, rotation: [0.0, 0.0, 0.0, 1.0], position: [0.0; 3] },
            Bone { name: "Bip01_Spine".into(), parent: 0, rotation: [0.0, 0.0, 0.0, 1.0], position: [5.0, 0.0, 0.0] },
        ]
    }

    fn synthetic_animation() -> MeshAnimation {
        use ue2_assets::{AnimBone, Notify, Track};
        let track = |frames: usize, z: f32| Track {
            bone: 0,
            rotations: vec![[0.0, 0.0, 0.0, 1.0]; frames],
            positions: (0..frames).map(|f| [0.0, 0.0, z * f as f32]).collect(),
            times: (0..frames).map(|f| f as f32).collect(),
        };
        MeshAnimation {
            bones: vec![
                AnimBone { name: "Bip01".into(), parent: 0 },
                AnimBone { name: "Bip01_Spine".into(), parent: 0 },
            ],
            sequences: vec![
                Sequence { name: "wait".into(), frames: 10, rate: 5.0, attack_eff: 0.0, attack_end_eff: 0.0, notifies: vec![], tracks: vec![track(10, 1.0)] },
                Sequence {
                    name: "atk01".into(),
                    frames: 20,
                    rate: 20.0,
                    attack_eff: 0.0,
                    attack_end_eff: 0.0,
                    notifies: vec![Notify::new(0.5, "AnimNotify_AttackShot")],
                    tracks: vec![track(20, -1.0)],
                },
                Sequence {
                    name: "social01".into(),
                    frames: 4,
                    rate: 2.0,
                    attack_eff: 0.0,
                    attack_end_eff: 0.0,
                    notifies: vec![],
                    tracks: vec![track(4, 10.0)],
                },
            ],
        }
    }

    fn synthetic_character(weapon_type: usize) -> LoadedCharacter {
        let bones = synthetic_bones();
        let mut master = CharacterMesh::empty();
        master.bones = bones.clone();
        master.bind = compose_pose(&bones, &bind_local_pose(&bones));
        master.inverse_bind = master.bind.iter().map(|m| m.inverse()).collect();
        let mut sword = CharacterMesh::empty();
        sword.slot = SLOT_RIGHT_HAND;
        sword.placement = Placement::Attached { bone: "Bip01_Spine" };
        sword.bones = vec![bones[0].clone()];
        sword.bind = vec![Mat4::IDENTITY];
        sword.inverse_bind = vec![Mat4::IDENTITY];
        let mut shield = CharacterMesh::empty();
        shield.slot = SLOT_LEFT_HAND;
        shield.placement = Placement::Attached { bone: "Bip01_Spine" };
        shield.bones = vec![bones[0].clone()];
        shield.bind = vec![Mat4::IDENTITY];
        shield.inverse_bind = vec![Mat4::IDENTITY];
        let parts = vec![
            (CharacterId(0), Arc::new(master)),
            (CharacterId(1), Arc::new(sword)),
            (CharacterId(2), Arc::new(shield)),
        ];
        LoadedCharacter::new(parts, Some(Arc::new(synthetic_animation())), 1.0, weapon_type).unwrap()
    }

    fn synthetic_set() -> AnimSet {
        let mut set = AnimSet::default();
        set.names_mut(AnimKind::Wait)[0] = Some("wait".into());
        set.names_mut(AnimKind::AtkWait)[0] = Some("wait".into());
        set.names_mut(AnimKind::Atk01)[0] = Some("atk01".into());
        set.names_mut(AnimKind::Atk02)[0] = Some("atk01".into());
        set.names_mut(AnimKind::Atk03)[0] = Some("atk01".into());
        set.npc_social[0] = Some("social01".into());
        set.npc_social_hide[0] = HideWeapons { right: true, left: false };
        set
    }

    fn synthetic_tables() -> DatTables {
        let mut tables = empty_tables();
        tables.anim_names.insert("gremlin".into(), synthetic_set());
        tables.npcgrp = vec![gremlin()].into_iter().collect();
        tables.skillgrp.insert(
            (1177, 1),
            l2_dat::Skill {
                id: 1177,
                level: 1,
                oper_type: 0,
                mp_consume: 0,
                cast_range: 0,
                cast_style: 3,
                hit_time: 1.0,
                is_magic: 1,
                ani_char: "B".into(),
                visual_effect: String::new(),
                icon: String::new(),
                extra_eff: 0,
                is_ench: 0,
                ench_skill_id: 0,
                hp_consume: 0,
                rumble_self: 0,
                rumble_target: 0,
            },
        );
        tables.mobskillanimgrp.insert((1, 4788), "spatk01".into());
        tables
    }

    #[test]
    fn anim_tables_read_sequence_timing_skillgrp_and_mobskillanimgrp() {
        let animation = synthetic_animation();
        let tables = synthetic_tables();
        let anim_tables = AnimTables { animation: Some(&animation), tables: &tables };
        assert_eq!(anim_tables.seq("WAIT"), Some(SeqInfo::new(10, 5.0)));
        assert_eq!(anim_tables.seq("atk01"), Some(SeqInfo::with_shots(20, 20.0, &[0.5])));
        assert_eq!(anim_tables.seq("run"), None);
        assert_eq!(AnimTables { animation: None, tables: &tables }.seq("wait"), None);
        assert_eq!(
            anim_tables.skill(1177, 1),
            Some(SkillData { animation: "B".into(), cast_style: 3, visual_effect_lead: None })
        );
        assert_eq!(anim_tables.skill(1177, 2), None);
        assert_eq!(anim_tables.skill(-1, 1), None);
        assert_eq!(anim_tables.npc_skill_anim(1, 4788).as_deref(), Some("spatk01"));
        assert_eq!(anim_tables.npc_skill_anim(2, 4788), None);
    }

    #[test]
    fn pose_memory_tweens_from_the_pose_the_pawn_was_in_and_forgets_it_after_the_tween() {
        let mut character = synthetic_character(0);
        let mut memory = PoseMemory::default();
        let mut channel = Channel::default();
        assert_eq!(character.master_pose(&channel, &mut memory).len(), 2);
        assert_eq!(memory.sequence, None);
        channel.play("wait", &SeqInfo::new(10, 5.0), 1.0, 0.0, true);
        channel.advance(1.0);
        let pose = character.master_pose(&channel, &mut memory);
        assert!((pose[0].transform_point3(Vec3::ZERO).z - 5.0).abs() < 2e-3, "{:?}", pose[0]);
        assert_eq!(memory.sequence, Some(0));
        assert!(memory.tween_from.is_none());
        channel.play("atk01", &SeqInfo::with_shots(20, 20.0, &[0.5]), 1.0, 0.5, false);
        channel.advance(0.25);
        assert_eq!(channel.tween_alpha(), Some(0.5));
        let blended = character.master_pose(&channel, &mut memory);
        assert!((blended[0].transform_point3(Vec3::ZERO).z - 2.5).abs() < 2e-3, "{:?}", blended[0]);
        assert!((memory.tween_from.as_ref().map_or(0.0, |from| from[0].1.z) - 5.0).abs() < 2e-3);
        channel.advance(0.35);
        assert!(!channel.is_tweening());
        let posed = character.master_pose(&channel, &mut memory);
        assert!((posed[0].transform_point3(Vec3::ZERO).z + channel.key_frame()).abs() < 1e-3, "{:?}", posed[0]);
        assert!(memory.tween_from.is_none());
        assert_eq!(memory.sequence, Some(1));
        assert!(character.bone_tracks.contains_key(&0) && character.bone_tracks.contains_key(&1));
        assert_eq!(character.sequence("Social01"), Some(2));
        assert_eq!(character.sequence("nope"), None);
        let frame = character.frame_pose(2, 1.0);
        assert!((frame[0].transform_point3(Vec3::ZERO).z - 10.0).abs() < 1e-4);
    }

    #[test]
    fn looping_wait_blends_the_last_key_back_into_the_first() {
        let mut character = synthetic_character(0);
        let mut memory = PoseMemory::default();
        let mut channel = Channel::default();
        channel.play("wait", &SeqInfo::new(10, 5.0), 1.0, 0.0, true);
        channel.advance(1.9);
        assert!(channel.frame > channel.last && channel.frame < 1.0);
        let pose = character.master_pose(&channel, &mut memory);
        let z = pose[0].transform_point3(Vec3::ZERO).z;
        assert!(z > 0.0 && z < 9.0, "{z}");
    }

    #[test]
    fn hidden_parts_follow_the_social_hide_flags() {
        let character = synthetic_character(0);
        let none = HideWeapons::default();
        assert!((0..3).all(|part| !character.hidden(part, none)));
        let right = HideWeapons { right: true, left: false };
        assert_eq!((0..3).map(|part| character.hidden(part, right)).collect::<Vec<_>>(), [false, true, false]);
        let both = HideWeapons { right: true, left: true };
        assert_eq!((0..3).map(|part| character.hidden(part, both)).collect::<Vec<_>>(), [false, true, true]);
    }

    #[test]
    fn instances_tick_the_pawn_from_its_triggers_and_hide_the_weapon_during_a_social() {
        let tables = synthetic_tables();
        let mut cache = CharacterCache::default();
        cache.insert(npc_key(1), synthetic_character(0));
        let mut world = World::default();
        let gremlin = entity(EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true }, Vec3::ZERO);
        world.entities.insert(1, gremlin);
        let shown = instances(&mut world, &mut cache, &tables, &frame(0.1), |_| 0, |_| Sample::default(), |_, _| true);
        assert_eq!(shown.len(), 3);
        let channel = &world.entities[&1].anim.channel;
        assert_eq!(channel.sequence.as_deref(), Some("wait"));
        assert!(channel.looping && !channel.is_tweening());
        assert!((channel.rate - 0.5).abs() < 1e-6);
        assert!((channel.frame - 0.05).abs() < 1e-3, "{}", channel.frame);
        assert_eq!(world.entities[&1].anim.npc_class_id, 1);
        assert_eq!(world.entities[&1].anim.npc_speed, 1.0);
        assert_eq!(cache.poses[&1].sequence, Some(0));
        world.entities.get_mut(&1).unwrap().anim.push(Trigger::Social(1));
        let shown = instances(&mut world, &mut cache, &tables, &frame(0.0), |_| 0, |_| Sample::default(), |_, _| true);
        assert_eq!(shown.len(), 2);
        assert_eq!(world.entities[&1].anim.channel.sequence.as_deref(), Some("social01"));
        assert_eq!(world.entities[&1].anim.hide_weapons, HideWeapons { right: true, left: false });
        for _ in 0..30 {
            instances(&mut world, &mut cache, &tables, &frame(0.1), |_| 0, |_| Sample::default(), |_, _| true);
        }
        assert_eq!(world.entities[&1].anim.channel.sequence.as_deref(), Some("wait"));
        assert_eq!(world.entities[&1].anim.hide_weapons, HideWeapons::default());
        let shown = instances(&mut world, &mut cache, &tables, &frame(0.0), |_| 0, |_| Sample::default(), |_, _| true);
        assert_eq!(shown.len(), 3);
        world.entities.clear();
        instances(&mut world, &mut cache, &tables, &frame(0.0), |_| 0, |_| Sample::default(), |_, _| true);
        assert!(cache.poses.is_empty());
    }

    #[test]
    fn swim_bubbles_follow_the_chance_of_dt_over_dt_plus_nine_tenths_and_add_breath_every_third_bone() {
        let mut seen = std::collections::HashSet::new();
        let mut spawned = 0;
        let mut breaths = 0;
        let mut generator = crate::world_sound::CrtRand::seeded(7);
        for _ in 0..20_000 {
            let plans = swim_bubbles(&mut generator, 0.1, 30, Some(12));
            if let Some(&(bone, class)) = plans.first() {
                assert_eq!(class, SWIM_BUBBLE_CLASS);
                assert!(bone < 29);
                seen.insert(bone);
                spawned += 1;
                if plans.len() == 2 {
                    assert_eq!(bone % 3, 0);
                    assert_eq!(plans[1], (12, SWIM_BREATH_CLASS));
                    breaths += 1;
                }
            }
        }
        let expected = 20_000.0 * 0.1 / 1.0;
        assert!((f64::from(spawned) - expected).abs() < expected * 0.1, "spawned={spawned}");
        assert!(seen.len() > 20);
        assert!(breaths > 0 && breaths < spawned);
        assert!(swim_bubbles(&mut generator, 0.1, 0, None).is_empty());
    }

    #[test]
    fn spawn_skill_effect_attaches_to_the_named_bone_and_a_missing_bone_is_a_no_op() {
        let mut cache = CharacterCache::default();
        let key = npc_key(1);
        cache.insert(key, synthetic_character(0));
        assert!(cache.spawn_skill_effect(7, key, "LineageEffect.s_u010_a", "Bip01_Spine", EffectModifiers::default()));
        let spawns = cache.take_transient_spawns();
        assert_eq!(spawns.len(), 1);
        assert_eq!(spawns[0].class, "LineageEffect.s_u010_a");
        assert_eq!(spawns[0].stick_velocity, None);
        assert_eq!(cache.transients[0].object_id, 7);
        assert!(!cache.spawn_skill_effect(7, key, "LineageEffect.m_u018_a", "NoSuchBone", EffectModifiers::default()));
        assert!(cache.take_transient_spawns().is_empty(), "0x104cba60: sem osso correspondente, sem efeito");
    }

    #[test]
    fn a_bone_index_attaches_by_position_in_the_skeleton_and_an_out_of_range_index_is_a_no_op() {
        let mut cache = CharacterCache::default();
        let key = npc_key(1);
        cache.insert(key, synthetic_character(0));
        assert!(cache.spawn_skill_effect_at_bone_index(7, key, "LineageEffect.s_u010_a", 0, EffectModifiers::default()));
        assert_eq!(cache.transients[0].bone, Some(0));
        assert!(!cache.spawn_skill_effect_at_bone_index(7, key, "LineageEffect.s_u010_a", -1, EffectModifiers::default()));
        assert!(!cache.spawn_skill_effect_at_bone_index(7, key, "LineageEffect.s_u010_a", 100_000, EffectModifiers::default()));
    }

    #[test]
    fn cast_start_only_spawns_when_the_skill_table_has_an_entry() {
        let mut tables = synthetic_tables();
        let skill = |id: u32| l2_dat::Skill {
            id,
            level: 1,
            oper_type: 0,
            mp_consume: 0,
            cast_range: -1,
            cast_style: 1,
            hit_time: 0.0,
            is_magic: 1,
            ani_char: "S".into(),
            visual_effect: String::new(),
            icon: String::new(),
            extra_eff: 0,
            is_ench: 0,
            ench_skill_id: 0,
            hp_consume: 0,
            rumble_self: -1,
            rumble_target: -1,
        };
        tables.skillgrp.insert((10, 1), skill(10));
        tables.skillgrp.insert((0x7ffe, 1), skill(0x7ffe));
        let mut cache = CharacterCache::default();
        cache.insert(npc_key(1), synthetic_character(0));
        let mut world = World::default();
        world.entities.insert(1, entity(EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true }, Vec3::new(100.0, 200.0, 300.0)));
        world.entities.get_mut(&1).unwrap().anim.push(Trigger::MagicSkillUse { skill_id: 10, level: 1, hit_time: 3000, target: 1 });
        instances(&mut world, &mut cache, &tables, &frame(0.0), |_| 0, |_| Sample::default(), |_, _| true);
        let spawns = cache.take_world_spawns();
        assert_eq!(spawns.len(), 1, "skill 10 cai no SpawnActor padrão do SkillEffectInit");
        assert_eq!(spawns[0].class, "LineageEffect.e_u011_a");
        assert_eq!(spawns[0].location, Vec3::ZERO, "SpawnActor nasce na origem; o ator segue o dono pela física de trailer");
        assert_eq!(spawns[0].owner.map(|owner| owner.object_id), Some(1));
        world.entities.get_mut(&1).unwrap().anim.push(Trigger::MagicSkillUse { skill_id: 0x7ffe, level: 1, hit_time: 3000, target: 1 });
        instances(&mut world, &mut cache, &tables, &frame(0.0), |_| 0, |_| Sample::default(), |_, _| true);
        assert!(cache.take_world_spawns().is_empty(), "skill fora da faixa do switch não gera efeito");
    }

    fn sound_notify(time: f32, sound: &str) -> ue2_assets::Notify {
        use ue2_assets::{NotifyClass, NotifyProperty, NotifyValue};
        let property = |name: &str, value: NotifyValue| NotifyProperty { name: name.into(), index: 0, value };
        ue2_assets::Notify {
            time,
            class: "AnimNotify_Sound".into(),
            export: None,
            classes: vec![NotifyClass { name: "AnimNotify_Sound".into(), path: "Engine.AnimNotify_Sound".into(), export: None }],
            properties: vec![
                property("Sound", NotifyValue::Object(sound.into())),
                property("Volume", NotifyValue::Float(250.0)),
                property("Radius", NotifyValue::Int(60)),
                property("Random", NotifyValue::Int(100)),
            ],
        }
    }

    #[test]
    fn a_sound_notify_in_the_playing_sequence_queues_a_sound_plan_at_the_pawn() {
        let mut animation = synthetic_animation();
        animation.sequences[0].notifies = vec![sound_notify(0.05, "ChrSound.Test_1")];
        let mut character = synthetic_character(0);
        character.animation = Some(Arc::new(animation));
        let tables = synthetic_tables();
        let mut cache = CharacterCache::default();
        cache.insert(npc_key(1), character);
        let mut world = World::default();
        let position = Vec3::new(100.0, 200.0, 300.0);
        world.entities.insert(1, entity(EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true }, position));
        for _ in 0..4 {
            instances(&mut world, &mut cache, &tables, &frame(0.1), |_| 0, |_| Sample::default(), |_, _| true);
        }
        let plans = cache.take_sound_plans();
        assert!(!plans.is_empty());
        assert_eq!((plans[0].name.as_str(), plans[0].actor, plans[0].position, plans[0].radius), ("ChrSound.Test_1", 1, position, 60.0));
        assert!((plans[0].volume - 250.0 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn a_view_shake_notify_queues_a_shake_plan_with_two_fractions_at_the_pawn() {
        use ue2_assets::{NotifyClass, NotifyProperty, NotifyValue};
        let property = |name: &str, value: NotifyValue| NotifyProperty { name: name.into(), index: 0, value };
        let shake = ue2_assets::Notify {
            time: 0.05,
            class: "AnimNotify_ViewShake".into(),
            export: None,
            classes: vec![NotifyClass { name: "AnimNotify_ViewShake".into(), path: "Engine.AnimNotify_ViewShake".into(), export: None }],
            properties: vec![
                property("ShakeType", NotifyValue::Byte(4)),
                property("ShakeIntensity", NotifyValue::Float(3.0)),
                property("ShakeRange", NotifyValue::Float(800.0)),
                property("ShakeCount", NotifyValue::Int(12)),
            ],
        };
        let mut animation = synthetic_animation();
        animation.sequences[0].notifies = vec![shake];
        let mut character = synthetic_character(0);
        character.animation = Some(Arc::new(animation));
        let tables = synthetic_tables();
        let mut cache = CharacterCache::default();
        cache.insert(npc_key(1), character);
        let mut world = World::default();
        let position = Vec3::new(10.0, 20.0, 30.0);
        world.entities.insert(1, entity(EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true }, position));
        for _ in 0..4 {
            instances(&mut world, &mut cache, &tables, &frame(0.1), |_| 0, |_| Sample::default(), |_, _| true);
        }
        let plans = cache.take_view_shakes();
        assert!(!plans.is_empty());
        assert_eq!((plans[0].notify.kind, plans[0].notify.intensity, plans[0].notify.range, plans[0].notify.count, plans[0].location), (4, 3.0, 800.0, 12, position));
        assert!(plans[0].randoms.0 >= 0.0 && plans[0].randoms.0 < 1.0001);
        assert_eq!(plans[0].smoke, None);
    }

    #[test]
    fn a_screen_fade_notify_queues_the_fade_with_the_class_defaults_for_missing_properties() {
        use ue2_assets::{NotifyClass, NotifyProperty, NotifyValue};
        let fade = ue2_assets::Notify {
            time: 0.05,
            class: "AnimNotify_ScreenFade".into(),
            export: None,
            classes: vec![NotifyClass { name: "AnimNotify_ScreenFade".into(), path: "Engine.AnimNotify_ScreenFade".into(), export: None }],
            properties: vec![
                NotifyProperty { name: "FadeOutDuration".into(), index: 0, value: NotifyValue::Float(3000.0) },
                NotifyProperty { name: "FadeOutColor".into(), index: 0, value: NotifyValue::Color([10, 20, 30, 255]) },
                NotifyProperty { name: "FadeInDuration".into(), index: 0, value: NotifyValue::Float(1000.0) },
            ],
        };
        let mut animation = synthetic_animation();
        animation.sequences[0].notifies = vec![fade];
        let mut character = synthetic_character(0);
        character.animation = Some(Arc::new(animation));
        let tables = synthetic_tables();
        let mut cache = CharacterCache::default();
        cache.insert(npc_key(1), character);
        let mut world = World::default();
        world.entities.insert(1, entity(EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true }, Vec3::ZERO));
        for _ in 0..4 {
            instances(&mut world, &mut cache, &tables, &frame(0.1), |_| 0, |_| Sample::default(), |_, _| true);
        }
        let fades = cache.take_screen_fades();
        assert!(!fades.is_empty());
        assert_eq!((fades[0].fade_out_duration, fades[0].fade_out_color, fades[0].black_out_duration, fades[0].fade_in_duration), (3000.0, [10, 20, 30, 255], 0.0, 1000.0));
    }

    #[test]
    fn a_trail_camera_effect_notify_queues_a_camera_effect_instead_of_a_pawn_effect() {
        use ue2_assets::{NotifyClass, NotifyProperty, NotifyValue};
        let property = |name: &str, value: NotifyValue| NotifyProperty { name: name.into(), index: 0, value };
        let effect = ue2_assets::Notify {
            time: 0.05,
            class: "AnimNotify_Effect".into(),
            export: None,
            classes: vec![NotifyClass { name: "AnimNotify_Effect".into(), path: "Engine.AnimNotify_Effect".into(), export: None }],
            properties: vec![property("EffectClass", NotifyValue::Object("LineageEffect.e_u021_a".into())), property("TrailCamera", NotifyValue::Bool(true))],
        };
        let mut animation = synthetic_animation();
        animation.sequences[0].notifies = vec![effect];
        let mut character = synthetic_character(0);
        character.animation = Some(Arc::new(animation));
        let tables = synthetic_tables();
        let mut cache = CharacterCache::default();
        cache.insert(npc_key(1), character);
        let mut world = World::default();
        world.entities.insert(1, entity(EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true }, Vec3::ZERO));
        for _ in 0..4 {
            instances(&mut world, &mut cache, &tables, &frame(0.1), |_| 0, |_| Sample::default(), |_, _| true);
        }
        assert_eq!(cache.take_camera_effects(), ["LineageEffect.e_u021_a"]);
        assert!(cache.take_world_spawns().is_empty());
    }

    #[test]
    fn off_frustum_pawns_are_not_queued_for_mesh_loading() {
        let tables = synthetic_tables();
        let mut cache = CharacterCache::default();
        let mut world = World::default();
        world.entities.insert(1, entity(EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true }, Vec3::ZERO));
        assert!(instances(&mut world, &mut cache, &tables, &frame(0.0), |_| 0, |_| Sample::default(), |_, _| false).is_empty());
        assert!(cache.take_wanted(usize::MAX).is_empty());
    }

    #[test]
    fn pawns_without_an_int_section_loop_the_mesh_wait_like_lineage_deco() {
        let mut tables = synthetic_tables();
        tables.anim_names.clear();
        let mut cache = CharacterCache::default();
        cache.insert(npc_key(1), synthetic_character(0));
        let mut world = World::default();
        let deco = entity(EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: false }, Vec3::ZERO);
        world.entities.insert(1, deco);
        instances(&mut world, &mut cache, &tables, &frame(0.1), |_| 0, |_| Sample::default(), |_, _| true);
        let channel = &world.entities[&1].anim.channel;
        assert_eq!(channel.sequence.as_deref(), Some(DECO_WAIT));
        assert!(channel.looping);
        assert!(world.entities[&1].anim.pending.is_empty());
    }

    #[test]
    fn label_height_follows_the_client_formula() {
        assert!((label_height(24.0, 1.0, false, false) - 24.0).abs() < 1e-5);
        assert!((label_height(24.0, 1.03, false, false) - 24.0 * 1.06).abs() < 1e-4);
        assert!((label_height(24.0, 1.0, true, false) - (24.0 - 1.2 * 24.0)).abs() < 1e-4);
        assert!((label_height(24.0, 1.0, false, true) - (24.0 - 1.3 * 24.0)).abs() < 1e-4);
    }

    #[test]
    fn name_color_is_white_red_with_karma_or_magenta_in_pvp() {
        assert_eq!(name_color(0, 0, WHITE_NAME_COLOR, 0, false), [255, 255, 255]);
        assert_eq!(name_color(0, 1, WHITE_NAME_COLOR, 0, false), [255, 0, 255]);
        assert_eq!(name_color(1, 0, WHITE_NAME_COLOR, 0, false), [255, 155, 155]);
        assert_eq!(name_color(255, 0, WHITE_NAME_COLOR, 0, false), [255, 0, 0]);
        assert_eq!(name_color(1000, 1, WHITE_NAME_COLOR, 0, false), [255, 0, 0]);
        assert_eq!(name_color(0, 2, WHITE_NAME_COLOR, 0, false), [255, 255, 255]);
        assert_eq!(name_color(0, 2, WHITE_NAME_COLOR, 0, true), [255, 0, 255]);
        assert_eq!(name_color(0, 2, WHITE_NAME_COLOR, 40, false), [215, 235, 255]);
        assert_eq!(name_color(1, 2, WHITE_NAME_COLOR, 0, true), [255, 155, 155]);
    }

    #[test]
    fn a_click_on_the_name_or_title_text_hits_the_pawn_of_that_label() {
        let font = crate::ui2d::tests::small_font();
        let label = Label { object_id: 7, name: "Killer".into(), title: "Canastra".into(), store_sign: None, speech: None, modern_text_size: None, name_visible: true, name_color: WHITE, title_color: WHITE, screen: [400.0, 300.0], depth: 0.5, clan_crest: None, ally_crest: None, mark: None, relation_icons: RelationIcons::default(), broadcast_icon: None };

        let rects = label_rects(&label, &font);
        let name_w = font.measure("Killer") as i32;
        assert_eq!(rects[0], (400 - name_w / 2, 300 - crate::ui2d::ENGINE_NAME_RISE, name_w, font.line_height as i32));
        assert_eq!(rects[1].1, 300 - 2 * font.line_height as i32);
        let labels = vec![label.clone(), Label { object_id: 9, title: String::new(), screen: [400.0, 200.0], ..label.clone() }];
        assert_eq!(label_at(&labels, &font, (400.0, 300.0 - 10.0)), Some(7));
        assert_eq!(label_at(&labels, &font, (400.0, 300.0 - 2.0 * font.line_height as f32 + 1.0)), Some(7));
        assert_eq!(label_at(&labels, &font, (400.0, 200.0 - 10.0)), Some(9));
        assert_eq!(label_at(&labels, &font, (400.0, 200.0 - 2.0 * font.line_height as f32 + 1.0)), None);
        assert_eq!(label_at(&labels, &font, (400.0 + name_w as f32, 290.0)), None);
        assert_eq!(label_at(&labels, &font, (400.0, 320.0)), None);
    }

    #[test]
    fn received_speech_projects_for_self_and_neighbour_without_replacing_titles() {
        let tables=empty_tables();
        let mut world=World::default();
        let mut me=entity(player_kind(0,0,0,0),Vec3::ZERO);
        me.object_id=1;me.title="Título local".into();
        let mut neighbour=entity(player_kind(0,0,0,0),Vec3::new(0.0,100.0,0.0));
        neighbour.object_id=2;neighbour.title="Título vizinho".into();
        world.me=Some(me);world.entities.insert(2,neighbour);
        world.receive_speech(1,0,"Olá, pessoal!");world.receive_speech(2,0,"Vamos para Giran?");
        let shown=labels(&world,&tables,&CharacterCache::default(),&frame(0.0),&camera(),(1280.0,720.0),None);
        for (id,title,text) in [(1,"Título local","Olá, pessoal!"),(2,"Título vizinho","Vamos para Giran?")] {
            let label=shown.iter().find(|label|label.object_id==id).unwrap();
            assert_eq!(label.title,title);
            assert_eq!(label.speech.as_ref().unwrap().text,text);
            assert!(label.speech.as_ref().unwrap().screen[1]<label.screen[1]);
        }
        world.speech.tick(6.0);
        let shown=labels(&world,&tables,&CharacterCache::default(),&frame(0.0),&camera(),(1280.0,720.0),None);
        assert!(shown.iter().all(|label|label.speech.is_none()));
        assert!(shown.iter().all(|label|!label.title.is_empty()));
    }

    #[test]
    fn labels_keep_titles_and_build_separate_c4_store_signs() {
        let mut tables = empty_tables();
        tables.sysstring.insert(500, "Sell".into());
        tables.sysstring.insert(499, "Buy".into());
        let mut world = World::default();
        let mut seller = entity(player_kind(0, 0, 0, 0), Vec3::ZERO);
        seller.object_id = 1;
        seller.name = "Vendedor".into();
        seller.title = "Titulo original".into();
        seller.store_type = 1;
        seller.sitting = true;
        let mut buyer = entity(player_kind(0, 0, 0, 0), Vec3::new(0.0, 300.0, 0.0));
        buyer.object_id = 2;
        buyer.name = "Comprador".into();
        buyer.store_type = 3;
        buyer.sitting = true;
        world.entities.insert(seller.object_id, seller);
        world.entities.insert(buyer.object_id, buyer);
        world.stores.sell_messages.insert(1, "Vendo materiais".into());
        world.stores.buy_messages.insert(2, "Compro materiais".into());
        let shown = labels(&world, &tables, &CharacterCache::default(), &frame(0.0), &camera(), (1280.0, 720.0), None);
        let seller = shown.iter().find(|label| label.object_id == 1).unwrap();
        assert_eq!(seller.title, "Titulo original");
        assert_eq!(seller.store_sign.as_ref().map(|sign| sign.text.as_str()), Some("Sell\nVendo materiais"));
        assert_eq!(shown.iter().find(|label| label.object_id == 2).and_then(|label| label.store_sign.as_ref().map(|sign| sign.text.as_str())), Some("Buy\nCompro materiais"));
        let font = crate::ui2d::tests::small_font();
        let seller_rects = label_rects(seller, &font);
        assert_eq!(seller_rects.len(), 2);
        assert_eq!(seller_rects[1].1, seller.screen[1].round() as i32 - 2 * font.line_height as i32);
    }

    #[test]
    fn name_color_prefers_karma_then_pvp_then_the_unique_color_then_recommendations() {
        assert_eq!(unique_name_color(WHITE_NAME_COLOR), None);
        assert_eq!(unique_name_color(0x00ff00), Some([0, 255, 0]));
        assert_eq!(unique_name_color(0x123456), Some([0x56, 0x34, 0x12]));
        assert_eq!(unique_name_color(0x7fffffff_u32 as i32), None);
        assert_eq!(name_color(0, 0, 0x00ff00, 100, false), [0, 255, 0]);
        assert_eq!(name_color(0, 1, 0x00ff00, 100, false), [255, 0, 255]);
        assert_eq!(name_color(5, 1, 0x00ff00, 100, false), [255, 155, 155]);
        assert_eq!(name_color(0, 0, WHITE_NAME_COLOR, 1, false), [254, 255, 255]);
        assert_eq!(name_color(0, 0, WHITE_NAME_COLOR, 255, false), [0, 128, 255]);
        assert_eq!(name_color(0, 0, WHITE_NAME_COLOR, 256, false), [255, 255, 255]);
        assert_eq!(name_color(0, 0, WHITE_NAME_COLOR, -3, false), [255, 255, 255]);
        let mut me = entity(EntityKind::Player { class_id: 0, race: 0, sex: 0, equipped: [0; VISIBLE_EQUIP_SLOTS], hair_style: 0, hair_color: 0, face: 0, enchant_effect: 0, hero: false }, Vec3::ZERO);
        me.recommendations = 20;
        assert_eq!(entity_name_color(&me), [235, 245, 255]);
        me.pvp_flag = 2;
        me.tick_name_blink(0.1);
        assert_eq!(entity_name_color(&me), [235, 245, 255]);
        me.tick_name_blink(0.11);
        assert_eq!(entity_name_color(&me), [255, 0, 255]);
        me.tick_name_blink(0.21);
        assert_eq!(entity_name_color(&me), [235, 245, 255]);
    }

    #[test]
    fn projection_puts_the_target_at_the_center_and_drops_points_behind_or_outside() {
        let camera = camera();
        let center = project(&camera, (1280.0, 720.0), Vec3::ZERO).unwrap();
        assert!((center[0] - 640.0).abs() < 1e-2 && (center[1] - 360.0).abs() < 1e-2, "{center:?}");
        let above = project(&camera, (1280.0, 720.0), Vec3::new(0.0, 0.0, 50.0)).unwrap();
        assert!(above[1] < 360.0 && (above[0] - 640.0).abs() < 1e-2, "{above:?}");
        assert_eq!(project(&camera, (1280.0, 720.0), Vec3::new(0.0, -600.0, 0.0)), None);
        assert_eq!(project(&camera, (1280.0, 720.0), Vec3::new(5000.0, 0.0, 0.0)), None);
        assert_eq!(project(&camera, (0.0, 720.0), Vec3::ZERO), None);
    }

    #[test]
    fn labels_take_npc_names_and_title_colors_from_the_table_when_the_packet_is_empty() {
        let mut tables = empty_tables();
        tables.npcgrp = std::iter::once(gremlin()).collect();
        tables
            .npcname
            .insert(1, NpcName { id: 1, name: "Gremlin".into(), title: String::new(), color: [0x3f, 0x8b, 0xfe] });
        let mut world = World::default();
        let npc = entity(EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: false }, Vec3::ZERO);
        world.entities.insert(1, npc);
        let mut player = entity(player_kind(0, 0, 0, 0), Vec3::new(0.0, -600.0, 0.0));
        player.name = "Luzal".into();
        player.karma = 20;
        world.entities.insert(2, player);
        let shown = labels(&world, &tables, &CharacterCache::default(), &frame(0.0), &camera(), (1280.0, 720.0), None);
        assert_eq!(shown.len(), 1);
        assert_eq!(shown[0].name, "Gremlin");
        assert_eq!(shown[0].title_color, [0x3f, 0x8b, 0xfe]);
        assert_eq!(shown[0].name_color, [255, 255, 255]);
        let expected = project(&camera(), (1280.0, 720.0), Vec3::Z * 24.0).unwrap();
        assert_eq!(shown[0].screen, expected);
        let mut me = entity(player_kind(0, 0, 0, 0), Vec3::ZERO);
        me.name = "Wender".into();
        me.collision_height = 23.0;
        me.pvp_flag = 1;
        world.me = Some(me);
        let shown = labels(&world, &tables, &CharacterCache::default(), &frame(0.0), &camera(), (1280.0, 720.0), None);
        assert_eq!(shown[0].name, "Wender");
        assert_eq!(shown[0].name_color, [255, 0, 255]);
        assert_eq!(shown[0].title_color, [255, 255, 255]);
        assert_eq!(shown[0].screen, project(&camera(), (1280.0, 720.0), Vec3::Z * 23.0).unwrap());
    }

    #[test]
    fn collision_height_comes_from_the_packet_or_the_client_fallbacks() {
        let mut tables = empty_tables();
        let unknown =
            entity(EntityKind::Npc { npc_id: 7, rhand: 0, chest: 0, lhand: 0, attackable: false }, Vec3::ZERO);
        assert_eq!(collision_height(&unknown, &tables), NPC_FALLBACK.collision_height);
        tables.npcgrp = std::iter::once(gremlin()).collect();
        let known = entity(EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: false }, Vec3::ZERO);
        assert_eq!(collision_height(&known, &tables), 24.0);
        let me = entity(player_kind(0, 0, 0, 0), Vec3::ZERO);
        assert_eq!(collision_height(&me, &tables), 24.0);
    }

    #[test]
    fn demo_world_puts_the_fighter_at_the_anchor_and_the_npcs_ahead() {
        let world = demo_world(Vec3::new(100.0, 200.0, -30.0), 16384, 0);
        let me = world.me.as_ref().unwrap();
        assert_eq!(me.position, Vec3::new(100.0, 200.0, -30.0));
        assert_eq!(
            appearance_key(&me.kind),
            Some(AppearanceKey::Player {
                row: 0,
                face: 0,
                hair_style: 1,
                hair_color: 1,
                equipped: Equipped {
                    chest: DEMO_CHEST,
                    legs: DEMO_LEGS,
                    gloves: DEMO_GLOVES,
                    feet: DEMO_FEET,
                    rhand: DEMO_RHAND,
                    lhand: DEMO_LHAND,
                    ..Default::default()
                },
                enchant_effect: 0,
                hero: false,
                mounted: 0,
            })
        );
        assert_eq!((me.name.as_str(), me.level), ("Killer", 52));
        assert_eq!((me.hp, me.mp, me.cp), (Some((400, 900)), Some((200, 300)), Some((100, 100))));
        assert_eq!(world.entities.len(), 7);
        assert_eq!(world.targeted().map(|e| (e.object_id, e.hp)), Some((2, Some((30, 80)))));
        assert_eq!(me.anim.pending, [Trigger::Spawn]);
        let gremlin = &world.entities[&2];
        assert!(matches!(gremlin.kind, EntityKind::Npc { npc_id: 1, .. }));
        assert!((gremlin.position - Vec3::new(60.0, 260.0, -30.0)).length() < 1e-3, "{}", gremlin.position);
        let lector = &world.entities[&3];
        assert_eq!(lector.heading, 49152);
        assert!((lector.position - Vec3::new(30.0, 270.0, -30.0)).length() < 1e-3);
        assert!((world.entities[&4].position - Vec3::new(160.0, 320.0, -30.0)).length() < 1e-3);
        assert!(matches!(world.entities[&4].kind, EntityKind::Npc { npc_id: 999_999, .. }));
        let buried = &world.entities[&5];
        assert!(matches!(buried.kind, EntityKind::Npc { npc_id: 1, .. }));
        assert_eq!(buried.collision_height, 16.0);
        assert!(
            (buried.position - Vec3::new(140.0, 240.0, -30.0 - DEMO_BURIED_DEPTH)).length() < 1e-3,
            "{}",
            buried.position
        );
        assert_eq!(buried.heading, 32768);
        assert!(world.entities.values().all(|e| e.dest.is_none() && !e.dead));
        let trader = &world.entities[&6];
        assert!(matches!(trader.kind, EntityKind::Npc { npc_id: 7003, .. }));
        assert_eq!(trader.anim.pending, [Trigger::Spawn, Trigger::Social(1)]);
        assert!((trader.position - Vec3::new(80.0, 300.0, -30.0)).length() < 1e-3, "{}", trader.position);
        let keeper = &world.entities[&7];
        assert!(matches!(keeper.kind, EntityKind::Npc { npc_id: 7005, .. }));
        assert_eq!(keeper.anim.pending, [Trigger::Spawn, Trigger::Social(2)]);
        let sitter = &world.entities[&DEMO_SITTING_PLAYER_ID];
        assert!(matches!(sitter.kind, EntityKind::Player { .. }));
        assert!(sitter.sitting);
        assert_eq!((sitter.name.as_str(), sitter.heading), ("Sentado", 49152));
        assert_eq!(
            sitter.anim.pending,
            [Trigger::Spawn, Trigger::ChangeWaitType { change: WaitChange::Sit, from: WaitType::Stand }]
        );
        assert!((sitter.position - Vec3::new(150.0, 230.0, -30.0)).length() < 1e-3, "{}", sitter.position);
        assert!(world.entities.values().filter(|e| e.object_id != DEMO_SITTING_PLAYER_ID).all(|e| !e.sitting));
    }

    #[test]
    fn demo_crowd_uses_the_requested_count_and_a_centered_grid() {
        let world = demo_world(Vec3::ZERO, 0, 5);
        let mut crowd: Vec<_> = world.entities.values().filter(|entity| entity.object_id >= DEMO_CROWD_FIRST_ID).collect();
        crowd.sort_by_key(|entity| entity.object_id);
        assert_eq!(crowd.len(), 5);
        assert_eq!(crowd[0].position, Vec3::new(-64.0, 64.0, 0.0));
        assert_eq!(crowd[4].position, Vec3::ZERO);
        assert!(crowd.windows(2).all(|pair| appearance_key(&pair[0].kind) == appearance_key(&pair[1].kind)));
    }

    #[test]
    fn missing_characters_are_wanted_once_and_failures_are_not_retried() {
        let tables = empty_tables();
        let mut cache = CharacterCache::default();
        let mut world = World::default();
        world.entities.insert(
            1,
            entity(EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: false }, Vec3::ZERO),
        );
        world.entities.insert(2, entity(player_kind(0, 9, 0, 0), Vec3::ZERO));
        assert!(
            instances(&mut world, &mut cache, &tables, &frame(0.016), |_| 0, |_| Sample::default(), |_, _| true).is_empty()
        );
        assert_eq!(cache.take_wanted(usize::MAX), [npc_key(1)]);
        assert!(cache.is_pending(npc_key(1)));
        assert!(
            instances(&mut world, &mut cache, &tables, &frame(0.016), |_| 0, |_| Sample::default(), |_, _| true).is_empty()
        );
        assert!(cache.take_wanted(usize::MAX).is_empty());
        cache.fail(npc_key(1));
        instances(&mut world, &mut cache, &tables, &frame(0.016), |_| 0, |_| Sample::default(), |_, _| true);
        assert!(cache.take_wanted(usize::MAX).is_empty());
        world.entities.insert(
            3,
            entity(EntityKind::Npc { npc_id: 2, rhand: 0, chest: 0, lhand: 0, attackable: false }, Vec3::ZERO),
        );
        instances(&mut world, &mut cache, &tables, &frame(0.016), |_| 0, |_| Sample::default(), |_, _| true);
        assert_eq!(cache.take_wanted(usize::MAX), [npc_key(2)]);
        assert_eq!(cache.retry_pending(), [npc_key(2)]);
        assert!(cache.is_pending(npc_key(2)));
        assert_eq!(cache.take_wanted(usize::MAX), [npc_key(2)]);
        assert!(cache.retry_pending().is_empty());
        assert!(!cache.is_pending(npc_key(2)));
        instances(&mut world, &mut cache, &tables, &frame(0.016), |_| 0, |_| Sample::default(), |_, _| true);
        assert!(cache.take_wanted(usize::MAX).is_empty());
        assert_eq!(cache.loaded_count(), 0);
        assert!(!cache.is_pending(npc_key(1)));
    }

    #[test]
    fn character_requests_follow_the_original_resource_load_limit() {
        let mut cache = CharacterCache::default();
        cache.want(npc_key(1));
        cache.want(npc_key(2));
        cache.want(npc_key(3));
        assert_eq!(cache.take_wanted(1), [npc_key(1)]);
        assert_eq!(cache.take_wanted(2), [npc_key(2), npc_key(3)]);
    }

    fn bone_z(mesh: &CharacterMesh, palette: &[Mat4], name: &str) -> f32 {
        let index = mesh.bones.iter().position(|b| b.name.eq_ignore_ascii_case(name)).unwrap();
        (palette[index] * mesh.bind[index]).transform_point3(Vec3::ZERO).z
    }

    fn load_into(
        client: &mut Client,
        shared: &mut Shared,
        cache: &mut CharacterCache,
        tables: &DatTables,
        next_id: &mut usize,
    ) {
        for key in cache.take_wanted(usize::MAX) {
            let look = look(key, tables).unwrap();
            match load_character(client, shared, key, &look.appearance, look.draw_scale) {
                LoadResult::Character { parts, animation, draw_scale, weapon_type, .. } => {
                    let parts = parts
                        .into_iter()
                        .map(|mesh| {
                            *next_id += 1;
                            (CharacterId(*next_id), Arc::new(mesh))
                        })
                        .collect();
                    cache.insert(key, LoadedCharacter::new(parts, animation, draw_scale, weapon_type).unwrap());
                }
                LoadResult::Failed { reason, .. } => panic!("{reason}"),
                _ => panic!("resultado não relacionado ao personagem"),
            }
        }
    }

    #[test]
    #[ignore]
    fn gremlin_and_fighter_instances_from_the_real_tables() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let mut client = Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        let mut cache = CharacterCache::default();
        let mut world = World::default();
        world.entities.insert(
            1,
            entity(EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: false }, Vec3::ZERO),
        );
        let mut me = entity(player_kind(0, 0, 0, 0), Vec3::new(100.0, 0.0, 0.0));
        me.object_id = 2;
        world.me = Some(me);
        assert!(instances(&mut world, &mut cache, &tables, &frame(0.0), |_| 0, |_| Sample::default(), |_, _| true).is_empty());
        assert_eq!(cache.take_wanted(usize::MAX).len(), 2);
        cache.wanted = vec![npc_key(1), player_key(0, 0)];
        let mut shared = Shared::default();
        let mut next_id = 0usize;
        load_into(&mut client, &mut shared, &mut cache, &tables, &mut next_id);
        assert_eq!(shared.animations.len(), 2);
        let gremlin = cache.get(npc_key(1)).unwrap();
        let fighter = cache.get(player_key(0, 0)).unwrap();
        assert_eq!((gremlin.parts.len(), fighter.parts.len()), (1, 7));
        assert_eq!((gremlin.draw_scale, fighter.draw_scale), (1.0, 1.0));
        for character in [gremlin, fighter] {
            let bounds = character.light_bounds(Vec3::ZERO, 0);
            let moved_bounds = character.light_bounds(Vec3::new(100.0, 20.0, 30.0), 0);
            assert!(bounds.min.is_finite() && bounds.max.is_finite());
            assert!(bounds.extent().cmpgt(Vec3::ZERO).all());
            assert!((moved_bounds.min - bounds.min - Vec3::new(100.0, 20.0, 30.0)).length() < 1e-3);
            assert!((moved_bounds.max - bounds.max - Vec3::new(100.0, 20.0, 30.0)).length() < 1e-3);
            let (center, radius) = character.light_sphere(Vec3::ZERO, 0);
            let (shifted, shifted_radius) = character.light_sphere(Vec3::new(100.0, 20.0, 30.0), 0);
            assert!(radius.is_finite() && radius > 0.0);
            assert!((radius - shifted_radius).abs() < 1e-4);
            assert!((shifted - center - Vec3::new(100.0, 20.0, 30.0)).length() < 1e-3);
            eprintln!("esfera nativa {:?}: centro {center:?}, raio {radius}", character.parts[character.master].1.slot);
        }
        assert!((fighter.scale() - 1.03).abs() < 1e-3);
        assert_eq!(fighter.parts[fighter.master()].1.slot, 7);
        assert!(fighter.parts.iter().all(|(_, mesh)| mesh.bones.len() == 70));
        assert!((0..7).all(|part| fighter.kind(part) == Some(PartKind::Skinned)));
        let gremlin_wait = AnimTables { animation: gremlin.animation().map(Arc::as_ref), tables: &tables }
            .seq(tables.anim_names["gremlin"].name(AnimKind::Wait, 0).unwrap())
            .unwrap();
        assert_eq!((gremlin_wait.frames, gremlin_wait.rate, gremlin_wait.shots.len()), (61, 30.0, 0));
        let fighter_wait = AnimTables { animation: fighter.animation().map(Arc::as_ref), tables: &tables }
            .seq(tables.anim_names["mfighter"].name(AnimKind::Wait, 0).unwrap())
            .unwrap();
        assert_eq!((fighter_wait.frames, fighter_wait.rate), (10, 3.0));
        let fighter_atk = AnimTables { animation: fighter.animation().map(Arc::as_ref), tables: &tables }
            .seq(tables.anim_names["mfighter"].name(AnimKind::Atk01, 1).unwrap())
            .unwrap();
        assert_eq!(fighter_atk.shots.len(), 1);
        assert!(fighter_atk.shots[0] > 0.0 && fighter_atk.shots[0] < 1.0);
        let (gremlin_id, upper) =
            (gremlin.parts[0].0, fighter.parts.iter().find(|(_, m)| m.slot == 2).unwrap().clone());
        assert!(
            instances(&mut world, &mut cache, &tables, &frame(0.0), |_| 3, |_| Sample::default(), |_, _| false).is_empty()
        );
        let shown = instances(&mut world, &mut cache, &tables, &frame(0.5), |_| 3, |_| Sample::default(), |_, _| true);
        assert_eq!(shown.len(), 8);
        assert!(shown.iter().all(|i| i.zone == 3));
        let body = shown.iter().find(|i| i.mesh == upper.0).unwrap();
        assert_eq!(body.palette.len(), 70);
        assert!(bone_z(&upper.1, &body.palette, "Bip01_head") > bone_z(&upper.1, &body.palette, "Bip01_Pelvis"));
        let origin = body.world.transform_point3(Vec3::new(0.0, 0.0, 25.2));
        assert!((origin - Vec3::new(100.0, 0.0, 0.0)).length() < 1e-2, "{origin}");
        let feet = body.world.transform_point3(Vec3::ZERO).z;
        assert!((feet + 25.2 * 1.03).abs() < 1e-2, "{feet}");
        let front = body.world.transform_vector3(Vec3::Y);
        assert!(front.x > 1.0 && front.y.abs() < 1e-3, "{front}");
        let gremlin_instance = shown.iter().find(|i| i.mesh == gremlin_id).unwrap();
        let gremlin = cache.get(npc_key(1)).unwrap();
        assert_eq!(gremlin_instance.palette.len(), gremlin.parts[0].1.bones.len());
        let channel = &world.entities[&1].anim.channel;
        assert_eq!(channel.sequence.as_deref(), Some("wait"));
        assert!(channel.looping && !channel.is_tweening());
        let advanced = 0.5 * 30.0 / 61.0;
        let interrupted = channel.notifies.iter().any(|notify| notify.time > 0.0 && notify.time <= advanced);
        assert!(
            (channel.frame - advanced).abs() < 2e-4 || interrupted && channel.frame > 0.0 && channel.frame < advanced,
            "{channel:?}"
        );
        assert_eq!(world.me.as_ref().unwrap().anim.channel.sequence.as_deref(), Some("wait_Hand_MFighter"));
        assert!(gremlin.bone_tracks.contains_key(&gremlin.sequences["wait"].unwrap()));
        assert_eq!(cache.poses.len(), 2);
        assert_eq!(cache.poses[&1].local.len(), gremlin.parts[0].1.bones.len());
    }

    fn bone_world(instance: &CharacterInstance, mesh: &CharacterMesh, bone: usize) -> Vec3 {
        (instance.world * instance.palette[bone] * mesh.bind[bone]).transform_point3(Vec3::ZERO)
    }

    #[test]
    #[ignore]
    fn the_demo_killer_holds_the_sword_in_the_right_hand_and_the_shield_on_the_left_arm() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let mut client = Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        let mut cache = CharacterCache::default();
        let mut world = demo_world(Vec3::ZERO, 0, 0);
        world.entities.clear();
        assert!(instances(&mut world, &mut cache, &tables, &frame(0.0), |_| 0, |_| Sample::default(), |_, _| true).is_empty());
        let wanted = cache.take_wanted(usize::MAX);
        assert_eq!(wanted.len(), 1);
        let key = wanted[0];
        let look = look(key, &tables).unwrap();
        assert_eq!(look.appearance.weapon_type, 1);
        let is_effect = |placement: &Placement| matches!(placement, Placement::Enchanted { .. });
        let slots: Vec<u8> = look.appearance.parts.iter().filter(|p| !is_effect(&p.placement)).map(|p| p.slot).collect();
        assert_eq!(slots, [0, 1, 2, 3, 4, 5, 7, 8, 11]);
        let effects = look.appearance.parts.iter().filter(|p| is_effect(&p.placement)).count();
        cache.wanted = vec![key];
        let mut shared = Shared::default();
        let mut next_id = 0usize;
        load_into(&mut client, &mut shared, &mut cache, &tables, &mut next_id);
        let killer = cache.get(key).unwrap();
        assert_eq!(killer.parts.len(), 9, "{effects} peças de efeito da arma ficam fora das peças carregadas");
        assert_eq!(killer.parts[2].1.sections[0].texture(), "MFighter.MFighter_m001_t02_u");
        let hair: Vec<&str> = look.appearance.parts[4..6].iter().map(|p| p.mesh.as_str()).collect();
        assert_eq!(hair, ["Fighter.MFighter_m001_m00_bh", "Fighter.MFighter_m001_m00_ah"]);
        assert_eq!(look.appearance.parts[5].textures, [Some("MFighter.MFighter_m001_t01_m00_ah".into())]);
        let master = &killer.parts[killer.master()].1;
        assert_eq!((master.slot, master.bones.len()), (7, 70));
        let hand = master.bones.iter().position(|b| b.name == "Weapon_R_Bone").unwrap();
        let shield_bone = master.bones.iter().position(|b| b.name == "Shield_L_Bone").unwrap();
        assert!(matches!(killer.kind(7), Some(PartKind::Attached { bone: Some(b), dual_fist: None, .. }) if b == hand));
        assert!(matches!(killer.kind(8), Some(PartKind::Attached { bone: Some(b), .. }) if b == shield_bone));
        assert!((0..7).all(|part| killer.kind(part) == Some(PartKind::Skinned)));
        assert_eq!(killer.weapon_type, 1);
        let shown = instances(&mut world, &mut cache, &tables, &frame(0.5), |_| 0, |_| Sample::default(), |_, _| true);
        assert_eq!(shown.len(), 9);
        let me_wait = world.me.as_ref().unwrap().anim.channel.sequence.clone().unwrap();
        assert!(me_wait.eq_ignore_ascii_case("Wait_1HS_MFighter"), "{me_wait}");
        let killer = cache.get(key).unwrap();
        let body = &killer.parts[2].1;
        let sword = &killer.parts[7].1;
        let sword_instance = &shown[7];
        assert_eq!(sword_instance.palette.len(), 1);
        let hand_world = bone_world(&shown[2], body, hand);
        let to_world = sword_instance.world * sword_instance.palette[0];
        let grip_world = to_world.transform_point3(Vec3::ZERO);
        assert!((hand_world - grip_world).length() < 1e-3, "{hand_world} {grip_world}");
        let along_blade = |v: &ue2_assets::SkinVertex| v.position[0];
        let tip = sword.vertices.iter().max_by(|a, b| along_blade(a).total_cmp(&along_blade(b))).unwrap();
        let tip_world = to_world.transform_point3(Vec3::from(tip.position));
        assert!(tip_world.z < grip_world.z - 5.0, "ponta {tip_world} punho {grip_world}");
        let reach = (tip_world - grip_world).length();
        let expected =
            weapon_scale(DEMO_COLLISION_RADIUS, DEMO_PLAYER_COLLISION_HEIGHT, 0.0) * Vec3::from(tip.position).length();
        assert!((reach - expected).abs() < 1e-3, "{reach} {expected}");
        let pelvis = body.bones.iter().position(|b| b.name == "Bip01_Pelvis").unwrap();
        let pelvis_world = bone_world(&shown[2], body, pelvis);
        assert!((grip_world.z - pelvis_world.z).abs() < 15.0, "{grip_world} {pelvis_world}");
        let shield = &shown[8];
        let arm_world = bone_world(&shown[2], body, shield_bone);
        let shield_origin = (shield.world * shield.palette[0]).transform_point3(Vec3::ZERO);
        assert!((arm_world - shield_origin).length() < 1e-3, "{arm_world} {shield_origin}");
        assert_eq!(shield.palette.len(), killer.parts[8].1.bones.len());
    }

    #[test]
    #[ignore]
    fn the_sitting_demo_player_keeps_the_sword_in_hand_through_the_sit_tween() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let mut client = Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        let mut cache = CharacterCache::default();
        let mut world = demo_world(Vec3::ZERO, 0, 0);
        world.entities.retain(|id, _| *id == DEMO_SITTING_PLAYER_ID);
        world.me = None;
        instances(&mut world, &mut cache, &tables, &frame(0.0), |_| 0, |_| Sample::default(), |_, _| true);
        let mut shared = Shared::default();
        let mut next_id = 0usize;
        load_into(&mut client, &mut shared, &mut cache, &tables, &mut next_id);
        let key = appearance_key(&world.entities[&DEMO_SITTING_PLAYER_ID].kind).unwrap();
        let killer = cache.get(key).unwrap();
        let body = killer.parts[2].1.clone();
        let sword = killer.parts[7].1.clone();
        let hairs = [killer.parts[4].1.clone(), killer.parts[5].1.clone()];
        let kinds = [killer.kind(4), killer.kind(5)];
        eprintln!("cabelos: {kinds:?}");
        let hand = body.bones.iter().position(|b| b.name == "Weapon_R_Bone").unwrap();
        let expected = weapon_scale(DEMO_COLLISION_RADIUS, DEMO_PLAYER_COLLISION_HEIGHT, 0.0)
            * sword.vertices.iter().map(|v| Vec3::from(v.position).length()).fold(0.0, f32::max);
        for step in 0..70 {
            let dt = if step < 10 { 0.002 } else { 0.05 };
            let shown = instances(&mut world, &mut cache, &tables, &frame(dt), |_| 0, |_| Sample::default(), |_, _| true);
            assert_eq!(shown.len(), 9, "passo {step}");

            let channel = &world.entities[&DEMO_SITTING_PLAYER_ID].anim.channel;
            let hand_world = bone_world(&shown[2], &body, hand);
            let to_world = shown[7].world * shown[7].palette[0];
            let grip_world = to_world.transform_point3(Vec3::ZERO);
            assert!((hand_world - grip_world).length() < 1e-3, "passo {step} {channel:?}: {hand_world} {grip_world}");
            let reach = sword
                .vertices
                .iter()
                .map(|v| (to_world.transform_point3(Vec3::from(v.position)) - grip_world).length())
                .fold(0.0, f32::max);
            assert!((reach - expected).abs() < 1e-2, "passo {step} {channel:?}: alcance {reach} esperado {expected}");
            let head = body.bones.iter().position(|b| b.name.eq_ignore_ascii_case("Bip01_head")).unwrap();
            let head_world = bone_world(&shown[2], &body, head);
            for (mesh, part) in hairs.iter().zip([4, 5]) {
                let instance = &shown[part];
                let farthest = mesh
                    .vertices
                    .iter()
                    .map(|v| {
                        let skinned: Vec3 = v
                            .bones
                            .iter()
                            .zip(v.weights)
                            .filter(|(_, w)| *w > 0.0)
                            .map(|(b, w)| {
                                instance.palette[usize::from(*b)].transform_point3(Vec3::from(v.position)) * w
                            })
                            .sum();
                        (instance.world.transform_point3(skinned) - head_world).length()
                    })
                    .fold(0.0, f32::max);
                assert!(farthest < 40.0, "passo {step} peça {part} {channel:?}: cabelo a {farthest} da cabeça");
            }
        }
        let channel = &world.entities[&DEMO_SITTING_PLAYER_ID].anim.channel;
        assert!(channel.sequence.as_deref().is_some_and(|s| s.eq_ignore_ascii_case("SitWait_MFighter")), "{channel:?}");
    }

    fn posed_feet_below_center(character: &mut LoadedCharacter, sequence: usize) -> f32 {
        let mut lowest = f32::INFINITY;
        let master_pose = character.frame_pose(sequence, 0.0);
        let collision = Collision { radius: 8.0, height: 24.0 };
        for part in 0..character.parts.len() {
            let palette = character.palette(part, &master_pose, collision).unwrap();
            let mesh = character.parts[part].1.clone();
            for vertex in &mesh.vertices {
                let bind = Vec3::from(vertex.position);
                let skinned = vertex
                    .bones
                    .iter()
                    .zip(vertex.weights)
                    .filter(|(_, weight)| *weight > 0.0)
                    .map(|(bone, weight)| palette[*bone as usize].transform_point3(bind) * weight)
                    .sum::<Vec3>();
                lowest = lowest.min(character.world(part, Vec3::ZERO, 0).transform_point3(skinned).z);
            }
        }
        -lowest
    }

    #[test]
    #[ignore]
    fn every_player_mesh_stands_with_its_feet_three_units_below_the_collision_cylinder() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let mut client = Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        let mut shared = Shared::default();
        let classes = [
            (0, "mfighter", 23.0),
            (1, "ffighter", 23.5),
            (2, "mdarkelf", 24.0),
            (3, "fdarkelf", 23.5),
            (4, "mdwarf", 18.0),
            (5, "fdwarf", 19.0),
            (6, "melf", 24.0),
            (7, "felf", 23.0),
            (8, "mmagic", 22.8),
            (9, "fmagic", 22.5),
            (10, "morc", 28.0),
            (11, "forc", 27.0),
            (12, "mshaman", 27.0),
            (13, "fshaman", 25.5),
        ];
        for (row, section, collision_height) in classes {
            let key = player_key(row, 0);
            let look = look(key, &tables).unwrap();
            let LoadResult::Character { parts, animation, draw_scale, .. } =
                load_character(&mut client, &mut shared, key, &look.appearance, look.draw_scale)
            else {
                panic!("{section}")
            };
            let parts = parts.into_iter().enumerate().map(|(i, mesh)| (CharacterId(i), Arc::new(mesh))).collect();
            let mut character = LoadedCharacter::new(parts, animation, draw_scale, 0).unwrap();
            let wait = character.sequence(tables.anim_names[section].name(AnimKind::Wait, 0).unwrap()).unwrap();
            let feet = posed_feet_below_center(&mut character, wait);
            let sink = feet - collision_height;
            assert!((sink - 3.0).abs() < 0.6, "{section}: pés {feet:.2} abaixo do centro, CH {collision_height}");
            let on_floor = sink - crate::ground::FLOOR_HOVER;
            assert!(on_floor > 0.0 && on_floor < 1.5, "{section}: {on_floor:.2}");
        }
    }
}
