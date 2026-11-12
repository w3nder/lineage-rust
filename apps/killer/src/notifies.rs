use glam::Vec3;
use killer_assets::engine_math::{Coords, coords_div_rotator, coords_mul, coords_mul_rotator, ortho_rotation, transform_vector_by};
use crate::world_sound::{DEFAULT_RADIUS, SoundConfig, SoundPlan, VOLUME_SCALE};
use killer_world::anim::{AttackVoice, EffectNotify, NotifyKind, ScreenFade, SeqNotify, SoundNotify, ViewShake};
use ue2_assets::{Notify, NotifyValue};

const SOUND_TABLES: [&str; 8] = [
    "DefaultWalkSound",
    "DefaultRunSound",
    "GrassWalkSound",
    "GrassRunSound",
    "WaterWalkSound",
    "WaterRunSound",
    "DefaultActorWalkSound",
    "DefaultActorRunSound",
];

fn value<'a>(notify: &'a Notify, name: &str, index: u32) -> Option<&'a NotifyValue> {
    notify.properties.iter().find(|property| property.index == index && property.name.eq_ignore_ascii_case(name)).map(|property| &property.value)
}

fn float(notify: &Notify, name: &str) -> f32 {
    match value(notify, name, 0) {
        Some(NotifyValue::Float(value)) => *value,
        _ => 0.0,
    }
}

fn int(notify: &Notify, name: &str) -> i32 {
    match value(notify, name, 0) {
        Some(NotifyValue::Int(value)) => *value,
        _ => 0,
    }
}

fn flag(notify: &Notify, name: &str) -> bool {
    matches!(value(notify, name, 0), Some(NotifyValue::Bool(true)))
}

fn object(notify: &Notify, name: &str, index: u32) -> Option<String> {
    match value(notify, name, index) {
        Some(NotifyValue::Object(path)) if !path.is_empty() => Some(path.clone()),
        _ => None,
    }
}

fn name(notify: &Notify, name: &str) -> Option<String> {
    match value(notify, name, 0) {
        Some(NotifyValue::Name(value)) if !value.eq_ignore_ascii_case("none") => Some(value.clone()),
        _ => None,
    }
}

fn vector(notify: &Notify, name: &str) -> [f32; 3] {
    match value(notify, name, 0) {
        Some(NotifyValue::Vector(value)) => *value,
        _ => [0.0; 3],
    }
}

fn rotator(notify: &Notify, name: &str) -> [i32; 3] {
    match value(notify, name, 0) {
        Some(NotifyValue::Rotator(value)) => *value,
        _ => [0; 3],
    }
}

fn is_a(notify: &Notify, class: &str) -> bool {
    notify.classes.iter().any(|candidate| candidate.name.eq_ignore_ascii_case(class))
}

pub fn notify_kind(notify: &Notify) -> Option<NotifyKind> {
    if is_a(notify, "AnimNotify_AttackShot") {
        Some(NotifyKind::AttackShot)
    } else if is_a(notify, "AnimNotify_AttackPreShot") {
        Some(NotifyKind::AttackPreShot)
    } else if is_a(notify, "AnimNotify_AttackItem") {
        Some(NotifyKind::AttackItem)
    } else if is_a(notify, "AnimNotify_AttackVoice") {
        Some(NotifyKind::AttackVoice(AttackVoice { volume: float(notify, "Volume"), radius: float(notify, "Radius"), random: int(notify, "Random") }))
    } else if is_a(notify, "AnimNotify_SwimSound") {
        Some(NotifyKind::SwimSound)
    } else if is_a(notify, "AnimNotify_Sound") {
        let mut sound = SoundNotify {
            sound: object(notify, "Sound", 0),
            volume: float(notify, "Volume"),
            radius: int(notify, "Radius"),
            random: int(notify, "Random"),
            walk_run: Default::default(),
        };
        for (table, property) in SOUND_TABLES.iter().enumerate() {
            for variant in 0..sound.walk_run[table].len() {
                sound.walk_run[table][variant] = object(notify, property, variant as u32);
            }
        }
        Some(NotifyKind::Sound(Box::new(sound)))
    } else if is_a(notify, "AnimNotify_Effect") {
        Some(NotifyKind::Effect(Box::new(EffectNotify {
            effect_class: object(notify, "EffectClass", 0),
            bone: name(notify, "Bone"),
            offset_location: vector(notify, "OffsetLocation"),
            offset_rotation: rotator(notify, "OffsetRotation"),
            attach: flag(notify, "Attach"),
            tag: name(notify, "Tag"),
            draw_scale: float(notify, "DrawScale"),
            draw_scale_3d: vector(notify, "DrawScale3D"),
            trail_camera: flag(notify, "TrailCamera"),
            independent_rotation: flag(notify, "IndependentRotation"),
            effect_scale: float(notify, "EffectScale"),
        })))
    } else if is_a(notify, "AnimNotify_ViewShake") {
        Some(NotifyKind::ViewShake(ViewShake {
            shake_type: match value(notify, "ShakeType", 0) {
                Some(NotifyValue::Byte(value)) => *value,
                _ => 0,
            },
            intensity: float(notify, "ShakeIntensity"),
            vector: vector(notify, "ShakeVector"),
            range: float(notify, "ShakeRange"),
            count: int(notify, "ShakeCount"),
        }))
    } else if is_a(notify, "AnimNotify_ScreenFade") {
        Some(NotifyKind::ScreenFade(ScreenFade {
            fade_out_duration: float(notify, "FadeOutDuration"),
            fade_out_color: match value(notify, "FadeOutColor", 0) {
                Some(NotifyValue::Color(value)) => *value,
                _ => [0; 4],
            },
            black_out_duration: float(notify, "BlackOutDuration"),
            fade_in_duration: float(notify, "FadeInDuration"),
        }))
    } else if notify.classes.is_empty() {
        None
    } else {
        Some(NotifyKind::Other(notify.class.clone()))
    }
}

pub fn seq_notifies(notifies: &[Notify], frames: f32) -> Vec<SeqNotify> {
    notifies
        .iter()
        .filter_map(|notify| {
            let kind = notify_kind(notify)?;
            let time = if notify.time > 1.0 { notify.time / frames } else { notify.time };
            Some(SeqNotify { time, kind })
        })
        .collect()
}

pub const TERRAIN_ONLY_EFFECT_TAG: &str = "e_u009_a";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectPawn {
    pub position: Vec3,
    pub rotation: [i32; 3],
    pub draw_scale: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoneFrame {
    pub origin: Vec3,
    pub axes: [Vec3; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EffectMesh {
    Skeletal(Option<BoneFrame>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct EffectAttach {
    pub bone: String,
    pub relative: Vec3,
    pub relative_rotation: [i32; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct EffectSpawn {
    pub class: String,
    pub location: Vec3,
    pub rotation: [i32; 3],
    pub size_scale: f32,
    pub tag: Option<String>,
    pub draw_scale: f32,
    pub draw_scale_3d: [f32; 3],
    pub attach: Option<EffectAttach>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectPlan {
    Camera { class: String, draw_scale: f32, draw_scale_3d: [f32; 3] },
    Spawn(EffectSpawn),
}

fn coords_of(frame: &BoneFrame) -> Coords {
    Coords { origin: frame.origin.to_array(), x: frame.axes[0].to_array(), y: frame.axes[1].to_array(), z: frame.axes[2].to_array() }
}

pub fn plan_effect(notify: &EffectNotify, pawn: &EffectPawn, mesh: EffectMesh, camera_controller: bool, on_terrain: bool) -> Option<EffectPlan> {
    let class = notify.effect_class.clone()?;
    if notify.trail_camera && camera_controller {
        return Some(EffectPlan::Camera { class, draw_scale: notify.draw_scale, draw_scale_3d: notify.draw_scale_3d });
    }
    let mut location = pawn.position;
    let mut rotation = pawn.rotation;
    let scaled = notify.offset_location.map(|value| value * pawn.draw_scale);
    match (&notify.bone, mesh) {
        (None, _) => {
            let basis = coords_mul_rotator(&Coords::UNIT, pawn.rotation);
            let moved = transform_vector_by(scaled, &basis);
            location = pawn.position + Vec3::from(moved);
            if !notify.independent_rotation {
                let offset = coords_div_rotator(&Coords::UNIT, notify.offset_rotation);
                rotation = ortho_rotation(&coords_mul(&offset, &basis));
            }
        }
        (Some(_), EffectMesh::Skeletal(frame)) if !notify.attach => {
            let basis = frame.as_ref().map_or(Coords::UNIT, coords_of);
            let moved = transform_vector_by(scaled, &basis);
            location = Vec3::from(basis.origin) + Vec3::from(moved);
            if !notify.independent_rotation {
                let offset = coords_div_rotator(&Coords::UNIT, notify.offset_rotation);
                rotation = ortho_rotation(&coords_mul(&offset, &basis));
            }
        }
        _ => {}
    }
    let attach = match (&notify.bone, mesh) {
        (Some(bone), EffectMesh::Skeletal(_)) if notify.attach => Some(EffectAttach {
            bone: bone.clone(),
            relative: Vec3::from(scaled),
            relative_rotation: notify.offset_rotation,
        }),
        _ => None,
    };
    if notify.tag.as_deref().is_some_and(|tag| tag.eq_ignore_ascii_case(TERRAIN_ONLY_EFFECT_TAG)) && !on_terrain {
        return None;
    }
    Some(EffectPlan::Spawn(EffectSpawn {
        class,
        location,
        rotation,
        size_scale: notify.effect_scale * pawn.draw_scale,
        tag: notify.tag.clone(),
        draw_scale: notify.draw_scale,
        draw_scale_3d: notify.draw_scale_3d,
        attach,
    }))
}

pub fn sound_radius(radius: i32) -> f32 {
    if radius == 0 { DEFAULT_RADIUS } else { radius as f32 }
}

pub fn plan_sound(notify: &SoundNotify, pawn: i32, position: Vec3, roll: i32) -> Option<SoundPlan> {
    if notify.random <= roll % 100 {
        return None;
    }
    let name = notify.sound.clone()?;
    Some(SoundPlan { actor: pawn, name, position, volume: notify.volume * VOLUME_SCALE, radius: sound_radius(notify.radius), skip_if_playing: false })
}

pub const SMOKE_NPC_CLASS_ID: i32 = 12211;
const SMOKE_DEFAULT_RANGE: f32 = 1800.0;

pub fn smoke_range(npc_class_id: i32, shake: &ViewShake) -> Option<f32> {
    (npc_class_id == SMOKE_NPC_CLASS_ID).then_some(if shake.range == 0.0 { SMOKE_DEFAULT_RANGE } else { shake.range })
}

pub fn shake_notify(shake: &ViewShake) -> killer_world::camera_shake::ShakeNotify {
    killer_world::camera_shake::ShakeNotify { kind: shake.shake_type, intensity: shake.intensity, direction: shake.vector, range: shake.range, count: shake.count }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Grass,
    Land,
    Water,
    Actor,
}

pub struct FootstepContext<'a> {
    pub object_id: i32,
    pub position: Vec3,
    pub npc: bool,
    pub running: bool,
    pub boots: &'a [String],
    pub terrain: bool,
    pub grass: bool,
    pub water: bool,
    pub step_volume: bool,
    pub step_names: Option<&'a [String]>,
}

pub fn footstep_surface(context: &FootstepContext<'_>) -> Surface {
    if context.water {
        Surface::Water
    } else if !context.terrain {
        Surface::Actor
    } else if context.grass {
        Surface::Grass
    } else {
        Surface::Land
    }
}

pub fn footstep_name(config: &SoundConfig, running: bool, surface: Surface, pick: i32) -> Option<&String> {
    let table = match (surface, running) {
        (Surface::Land, false) => 0,
        (Surface::Land, true) => 1,
        (Surface::Grass, false) => 2,
        (Surface::Grass, true) => 3,
        (Surface::Water, false) => 4,
        (Surface::Water, true) => 5,
        (Surface::Actor, _) => 6,
    };
    config.footsteps[table][(pick % 3) as usize].as_ref()
}

pub fn plan_footsteps(notify: &SoundNotify, context: &FootstepContext<'_>, config: &SoundConfig, mut rand: impl FnMut() -> i32) -> Vec<SoundPlan> {
    if context.npc {
        return Vec::new();
    }
    let plan = |name: &String| SoundPlan {
        actor: context.object_id,
        name: name.clone(),
        position: context.position,
        volume: notify.volume * VOLUME_SCALE,
        radius: sound_radius(notify.radius),
        skip_if_playing: false,
    };
    let mut plans = Vec::new();
    if !context.boots.is_empty() {
        let name = &context.boots[(rand() % context.boots.len() as i32) as usize];
        if real_name(name) {
            plans.push(plan(name));
        }
    }
    if context.step_volume && !context.water {
        if let Some(name) = crate::walk_world::step_sound_name(context.step_names, &mut rand) {
            plans.push(plan(name));
        }
        return plans;
    }
    let surface = footstep_surface(context);
    if let Some(name) = footstep_name(config, context.running, surface, rand()) {
        plans.push(plan(name));
    }
    plans
}

pub struct SwimContext {
    pub object_id: i32,
    pub position: Vec3,
    pub in_water_volume: bool,
    pub afloat: bool,
    pub moving_in_water: bool,
}

pub fn plan_swim(context: &SwimContext, config: &SoundConfig, mut rand: impl FnMut() -> i32) -> Option<SoundPlan> {
    if !context.in_water_volume {
        return None;
    }
    let set = if context.afloat && context.moving_in_water { &config.surface_swim } else { &config.under_water };
    if rand() < set.random {
        return None;
    }
    let name = set.names[(rand() % 3) as usize].as_ref()?;
    Some(SoundPlan {
        actor: context.object_id,
        name: name.clone(),
        position: context.position,
        volume: set.volume * VOLUME_SCALE,
        radius: sound_radius(set.radius as i32),
        skip_if_playing: false,
    })
}

pub struct VoiceContext<'a> {
    pub object_id: i32,
    pub position: Vec3,
    pub npc: bool,
    pub has_target: bool,
    pub weapon_type: usize,
    pub voices: Option<&'a [Vec<String>; l2_dat::chargrp::VOICE_GROUPS]>,
}

fn real_name(name: &str) -> bool {
    !name.is_empty() && !name.eq_ignore_ascii_case("none")
}

pub fn plan_attack_voice(voice: &AttackVoice, context: &VoiceContext<'_>, gate: i32, pick: i32) -> Option<SoundPlan> {
    if context.npc || !context.has_target {
        return None;
    }
    if (gate % 100) as f32 > voice.random as f32 {
        return None;
    }
    let group = context.voices?.get(context.weapon_type)?;
    let count = i32::try_from(group.len()).ok().filter(|count| *count > 0)?;
    let name = group.get((pick % count) as usize).filter(|name| real_name(name))?;
    Some(SoundPlan { actor: context.object_id, name: name.clone(), position: context.position, volume: voice.volume * VOLUME_SCALE, radius: voice.radius, skip_if_playing: true })
}

pub struct DamageContext<'a> {
    pub object_id: i32,
    pub position: Vec3,
    pub npc: bool,
    pub npc_record: Option<&'a l2_dat::Npc>,
    pub char_sounds: &'a [String],
}

pub fn plan_damage_sound(context: &DamageContext<'_>, config: &SoundConfig, mut rand: impl FnMut() -> i32) -> Option<SoundPlan> {
    let plan = |name: &String, volume: f32, radius: f32| SoundPlan { actor: context.object_id, name: name.clone(), position: context.position, volume: volume * VOLUME_SCALE, radius, skip_if_playing: false };
    if context.npc {
        let record = context.npc_record?;
        if record.damage_sounds.is_empty() {
            return None;
        }
        let gate = (rand() % 100) as f32;
        if gate > record.sound_random {
            return None;
        }
        let name = &record.damage_sounds[(rand() % record.damage_sounds.len() as i32) as usize];
        return real_name(name).then(|| plan(name, record.sound_volume, record.sound_radius));
    }
    if rand() % 100 > config.char_random || context.char_sounds.is_empty() {
        return None;
    }
    let name = &context.char_sounds[(rand() % context.char_sounds.len() as i32) as usize];
    real_name(name).then(|| plan(name, config.char_volume, config.char_radius))
}

pub struct DefenseContext<'a> {
    pub object_id: i32,
    pub position: Vec3,
    pub npc: bool,
    pub item_sounds: &'a [String],
    pub npc_record: Option<&'a l2_dat::Npc>,
    pub char_sounds: &'a [String],
}

pub fn plan_defense_item_sound(context: &DefenseContext<'_>, config: &SoundConfig, mut rand: impl FnMut() -> i32) -> Option<SoundPlan> {
    let plan = |name: &String, volume: f32, radius: f32| SoundPlan { actor: context.object_id, name: name.clone(), position: context.position, volume: volume * VOLUME_SCALE, radius, skip_if_playing: false };
    if !context.item_sounds.is_empty() {
        let name = &context.item_sounds[(rand() % context.item_sounds.len() as i32) as usize];
        if real_name(name) {
            return Some(plan(name, config.item_volume, config.item_radius));
        }
    }
    if context.npc {
        let record = context.npc_record?;
        if record.defense_sounds.is_empty() {
            return None;
        }
        let name = &record.defense_sounds[(rand() % record.defense_sounds.len() as i32) as usize];
        return real_name(name).then(|| plan(name, record.sound_volume, record.sound_radius));
    }
    if context.char_sounds.is_empty() {
        return None;
    }
    let name = &context.char_sounds[(rand() % context.char_sounds.len() as i32) as usize];
    real_name(name).then(|| plan(name, config.char_volume, config.char_radius))
}

pub fn plan_etc_sound(actor: i32, position: Vec3, sound: &crate::world_sound::EtcSound) -> Option<SoundPlan> {
    real_name(&sound.name).then(|| SoundPlan { actor, name: sound.name.clone(), position, volume: sound.volume * VOLUME_SCALE, radius: sound.radius, skip_if_playing: false })
}

pub struct ItemContext<'a> {
    pub object_id: i32,
    pub position: Vec3,
    pub npc: bool,
    pub has_target: bool,
    pub attacking: bool,
    pub weapon_sounds: &'a [String],
    pub npc_record: Option<&'a l2_dat::Npc>,
    pub char_sounds: &'a [String],
}

pub fn plan_attack_item(context: &ItemContext<'_>, config: &SoundConfig, mut rand: impl FnMut() -> i32) -> Option<SoundPlan> {
    if !context.attacking || !context.has_target {
        return None;
    }
    let plan = |name: &String, volume: f32, radius: f32| SoundPlan { actor: context.object_id, name: name.clone(), position: context.position, volume: volume * VOLUME_SCALE, radius, skip_if_playing: true };
    if !context.weapon_sounds.is_empty() {
        let name = &context.weapon_sounds[(rand() % context.weapon_sounds.len() as i32) as usize];
        if real_name(name) {
            return Some(plan(name, config.item_volume, config.item_radius));
        }
    }
    if context.npc {
        let record = context.npc_record?;
        if record.attack_sounds.is_empty() {
            return None;
        }
        let name = &record.attack_sounds[(rand() % record.attack_sounds.len() as i32) as usize];
        return real_name(name).then(|| plan(name, record.sound_volume, record.sound_radius));
    }
    if context.char_sounds.is_empty() {
        return None;
    }
    let name = &context.char_sounds[(rand() % context.char_sounds.len() as i32) as usize];
    real_name(name).then(|| plan(name, config.char_volume, config.char_radius))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defense_item_sound_105e69d0_prefers_the_armor_list_then_the_npc_or_class_list() {
        let config = SoundConfig { item_volume: 200.0, item_radius: 40.0, char_volume: 100.0, char_radius: 30.0, ..SoundConfig::default() };
        let names = |list: &[&str]| list.iter().map(|name| (*name).to_owned()).collect::<Vec<_>>();
        let (armor, chars) = (names(&["ArmorSound.hit", "None"]), names(&["ChrSound.defense"]));
        let base = DefenseContext { object_id: 7, position: Vec3::X, npc: false, item_sounds: &armor, npc_record: None, char_sounds: &chars };
        let plan = plan_defense_item_sound(&base, &config, || 0).expect("armadura");
        assert_eq!((plan.name.as_str(), plan.volume, plan.radius, plan.skip_if_playing), ("ArmorSound.hit", 200.0 * VOLUME_SCALE, 40.0, false));
        let plan = plan_defense_item_sound(&base, &config, || 1).expect("classe");
        assert_eq!((plan.name.as_str(), plan.volume, plan.radius), ("ChrSound.defense", 100.0 * VOLUME_SCALE, 30.0));
        assert!(plan_defense_item_sound(&DefenseContext { char_sounds: &[], ..base }, &config, || 1).is_none());
        assert!(plan_defense_item_sound(&DefenseContext { item_sounds: &[], npc: true, ..base }, &config, || 0).is_none());
    }

    #[test]
    fn view_shake_1067bf00_marks_the_smoke_only_for_npc_12211_with_1800_when_range_is_zero() {
        let shake = ViewShake { range: 0.0, ..ViewShake::default() };
        assert_eq!(smoke_range(SMOKE_NPC_CLASS_ID, &shake), Some(1800.0));
        assert_eq!(smoke_range(SMOKE_NPC_CLASS_ID, &ViewShake { range: 2500.0, ..shake }), Some(2500.0));
        assert_eq!(smoke_range(12210, &shake), None);
        assert_eq!(smoke_range(0, &ViewShake { range: 2500.0, ..shake }), None);
    }

    fn effect(bone: Option<&str>, attach: bool) -> EffectNotify {
        EffectNotify {
            effect_class: Some("LineageEffect.e_u021_a".into()),
            bone: bone.map(str::to_owned),
            offset_location: [10.0, 20.0, 30.0],
            offset_rotation: [0, 0, 0],
            attach,
            tag: None,
            draw_scale: 1.0,
            draw_scale_3d: [1.0; 3],
            trail_camera: false,
            independent_rotation: false,
            effect_scale: 1.0,
        }
    }

    fn pawn() -> EffectPawn {
        EffectPawn { position: Vec3::new(100.0, 200.0, 300.0), rotation: [0, 16384, 0], draw_scale: 2.0 }
    }

    fn spawn(plan: Option<EffectPlan>) -> EffectSpawn {
        match plan {
            Some(EffectPlan::Spawn(spawn)) => spawn,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn effect_notify_1067cd00_without_bone_offsets_by_the_pawn_rotation_and_draw_scale() {
        let plan = spawn(plan_effect(&effect(None, false), &pawn(), EffectMesh::Skeletal(None), true, true));
        let basis = coords_mul_rotator(&Coords::UNIT, [0, 16384, 0]);
        let moved = transform_vector_by([20.0, 40.0, 60.0], &basis);
        assert_eq!(plan.location, Vec3::new(100.0, 200.0, 300.0) + Vec3::from(moved));
        assert_eq!(plan.size_scale, 2.0);
        assert_eq!(plan.rotation, ortho_rotation(&coords_mul(&coords_div_rotator(&Coords::UNIT, [0, 0, 0]), &basis)));
        assert_eq!(plan.rotation, [0, 16384, 0]);
        assert!(plan.attach.is_none());
    }

    #[test]
    fn effect_notify_1067cd00_independent_rotation_keeps_the_pawn_rotation() {
        let mut notify = effect(None, false);
        notify.independent_rotation = true;
        notify.offset_rotation = [0, 4096, 0];
        assert_eq!(spawn(plan_effect(&notify, &pawn(), EffectMesh::Skeletal(None), true, true)).rotation, [0, 16384, 0]);
        notify.independent_rotation = false;
        assert_ne!(spawn(plan_effect(&notify, &pawn(), EffectMesh::Skeletal(None), true, true)).rotation, [0, 16384, 0]);
    }

    #[test]
    fn effect_notify_1067cd00_attached_bone_effects_spawn_at_the_pawn_and_carry_the_relative_offset() {
        let plan = spawn(plan_effect(&effect(Some("Bip01 R Hand"), true), &pawn(), EffectMesh::Skeletal(None), true, true));
        assert_eq!(plan.location, pawn().position);
        assert_eq!(plan.rotation, pawn().rotation);
        let attach = plan.attach.expect("osso");
        assert_eq!(attach.bone, "Bip01 R Hand");
        assert_eq!(attach.relative, Vec3::new(20.0, 40.0, 60.0));
    }

    #[test]
    fn effect_notify_1067cd00_free_bone_effects_use_the_bone_frame_and_unit_coords_when_the_bone_is_unknown() {
        let frame = BoneFrame { origin: Vec3::new(5.0, 6.0, 7.0), axes: [Vec3::Y, -Vec3::X, Vec3::Z] };
        let known = spawn(plan_effect(&effect(Some("Bone01"), false), &pawn(), EffectMesh::Skeletal(Some(frame)), true, true));
        let moved = transform_vector_by([20.0, 40.0, 60.0], &coords_of(&frame));
        assert_eq!(known.location, Vec3::new(5.0, 6.0, 7.0) + Vec3::from(moved));
        let unknown = spawn(plan_effect(&effect(Some("NoBone"), false), &pawn(), EffectMesh::Skeletal(None), true, true));
        assert_eq!(unknown.location, Vec3::new(20.0, 40.0, 60.0));
        assert_eq!(unknown.rotation, [0, 0, 0]);
    }

    #[test]
    fn effect_notify_1067cd00_terrain_only_tag_is_dropped_off_terrain_and_trail_camera_follows_the_controller() {
        let mut notify = effect(None, false);
        notify.tag = Some("e_u009_a".into());
        assert!(plan_effect(&notify, &pawn(), EffectMesh::Skeletal(None), true, false).is_none());
        assert!(plan_effect(&notify, &pawn(), EffectMesh::Skeletal(None), true, true).is_some());
        notify.tag = None;
        notify.trail_camera = true;
        assert!(matches!(plan_effect(&notify, &pawn(), EffectMesh::Skeletal(None), true, true), Some(EffectPlan::Camera { .. })));
        assert!(matches!(plan_effect(&notify, &pawn(), EffectMesh::Skeletal(None), false, true), Some(EffectPlan::Spawn(_))));
        notify.effect_class = None;
        assert!(plan_effect(&notify, &pawn(), EffectMesh::Skeletal(None), true, true).is_none());
    }

    #[test]
    fn sound_notify_1067da20_plays_the_named_sound_with_the_random_gate_and_the_default_radius() {
        let notify = SoundNotify { sound: Some("ChrSound.Breath_1".into()), volume: 250.0, radius: 0, random: 5, ..SoundNotify::default() };
        assert!(plan_sound(&notify, 7, Vec3::ZERO, 5).is_none());
        assert!(plan_sound(&notify, 7, Vec3::ZERO, 105).is_none());
        let plan = plan_sound(&notify, 7, Vec3::new(1.0, 2.0, 3.0), 4).expect("toca");
        assert_eq!((plan.actor, plan.radius, plan.position), (7, DEFAULT_RADIUS, Vec3::new(1.0, 2.0, 3.0)));
        assert!((plan.volume - 250.0 / 255.0).abs() < 1e-6);
        let explicit = SoundNotify { radius: 60, random: 100, ..notify.clone() };
        assert_eq!(plan_sound(&explicit, 7, Vec3::ZERO, 99).expect("toca").radius, 60.0);
        assert!(plan_sound(&SoundNotify::default(), 7, Vec3::ZERO, 0).is_none());
    }

    fn voices() -> [Vec<String>; l2_dat::chargrp::VOICE_GROUPS] {
        let mut groups: [Vec<String>; l2_dat::chargrp::VOICE_GROUPS] = Default::default();
        groups[1] = vec!["ChrSound.A".into(), "ChrSound.B".into(), "ChrSound.C".into()];
        groups
    }

    fn voice_context(voices: &[Vec<String>; l2_dat::chargrp::VOICE_GROUPS]) -> VoiceContext<'_> {
        VoiceContext { object_id: 3, position: Vec3::new(1.0, 2.0, 3.0), npc: false, has_target: true, weapon_type: 1, voices: Some(voices) }
    }

    #[test]
    fn attack_voice_notify_1067cb40_plays_a_player_voice_of_the_weapon_group_under_the_random_gate() {
        let groups = voices();
        let voice = AttackVoice { volume: 250.0, radius: 60.0, random: 50 };
        let plan = plan_attack_voice(&voice, &voice_context(&groups), 50, 4).expect("toca");
        assert_eq!((plan.actor, plan.name.as_str(), plan.radius), (3, "ChrSound.B", 60.0));
        assert!((plan.volume - 250.0 / 255.0).abs() < 1e-6);
        assert!(plan_attack_voice(&voice, &voice_context(&groups), 51, 4).is_none());
        assert!(plan_attack_voice(&voice, &voice_context(&groups), 151, 0).is_none());
        assert!(plan_attack_voice(&AttackVoice { random: 0, ..voice }, &voice_context(&groups), 0, 2).is_some());
    }

    #[test]
    fn attack_voice_notify_1067cb40_is_silent_for_npcs_without_a_target_or_without_voices_for_the_weapon() {
        let groups = voices();
        let voice = AttackVoice { volume: 250.0, radius: 60.0, random: 100 };
        assert!(plan_attack_voice(&voice, &VoiceContext { npc: true, ..voice_context(&groups) }, 0, 0).is_none());
        assert!(plan_attack_voice(&voice, &VoiceContext { has_target: false, ..voice_context(&groups) }, 0, 0).is_none());
        assert!(plan_attack_voice(&voice, &VoiceContext { weapon_type: 4, ..voice_context(&groups) }, 0, 0).is_none());
        assert!(plan_attack_voice(&voice, &VoiceContext { voices: None, ..voice_context(&groups) }, 0, 0).is_none());
    }

    fn npc_with(attack: &[&str], radius: f32, volume: f32) -> l2_dat::Npc {
        l2_dat::Npc {
            id: 1,
            class: String::new(),
            mesh: String::new(),
            textures: vec![],
            textures2: vec![],
            speed: 1.0,
            effect: String::new(),
            level_min: 0,
            level_max: 0,
            attack_sounds: attack.iter().map(|name| (*name).to_owned()).collect(),
            defense_sounds: vec![],
            damage_sounds: vec![],
            sound_radius: radius,
            sound_volume: volume,
            sound_random: 0.0, skills: vec![], shows_bars: 0,
        }
    }

    #[test]
    fn damage_sound_105e7290_gates_by_random_and_picks_from_the_damage_list() {
        let mut npc = npc_with(&[], 90.0, 200.0);
        npc.damage_sounds = vec!["MonSound.hurt_1".to_owned(), "MonSound.hurt_2".to_owned()];
        npc.sound_random = 30.0;
        let class = vec!["ChrSound.hurt".to_owned()];
        let config = SoundConfig { char_random: 70, ..SoundConfig::default() };
        let monster = DamageContext { object_id: 4, position: Vec3::ZERO, npc: true, npc_record: Some(&npc), char_sounds: &class };
        let mut rolls = [130, 3].into_iter();
        let plan = plan_damage_sound(&monster, &config, || rolls.next().unwrap()).expect("npc");
        assert_eq!((plan.name.as_str(), plan.radius, plan.skip_if_playing), ("MonSound.hurt_2", 90.0, false));
        assert!(plan_damage_sound(&monster, &config, || 31).is_none());
        let player = DamageContext { npc: false, ..monster };
        assert_eq!(plan_damage_sound(&player, &config, || 70).expect("jogador").name, "ChrSound.hurt");
        assert!(plan_damage_sound(&player, &config, || 71).is_none());
        let empty = DamageContext { char_sounds: &[], ..player };
        assert!(plan_damage_sound(&empty, &config, || 0).is_none());
    }

    #[test]
    fn attack_item_notify_1067c2f0_prefers_the_weapon_then_the_npc_record_then_the_class_sounds() {
        let weapon = vec!["ItemSound.sword_1".to_owned(), "ItemSound.sword_2".to_owned()];
        let class = vec!["ChrSound.class_1".to_owned()];
        let npc = npc_with(&["MonSound.bite"], 90.0, 200.0);
        let config = SoundConfig::default();
        let base = ItemContext { object_id: 4, position: Vec3::ZERO, npc: false, has_target: true, attacking: true, weapon_sounds: &weapon, npc_record: Some(&npc), char_sounds: &class };
        let plan = plan_attack_item(&base, &config, || 3).expect("arma");
        assert_eq!((plan.name.as_str(), plan.radius), ("ItemSound.sword_2", 50.0));
        assert!((plan.volume - 1.0).abs() < 1e-6);
        let unarmed = ItemContext { weapon_sounds: &[], ..base };
        assert_eq!(plan_attack_item(&unarmed, &config, || 0).expect("classe").name, "ChrSound.class_1");
        let monster = ItemContext { weapon_sounds: &[], npc: true, ..base };
        let plan = plan_attack_item(&monster, &config, || 0).expect("npc");
        assert_eq!((plan.name.as_str(), plan.radius), ("MonSound.bite", 90.0));
        assert!((plan.volume - 200.0 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn attack_item_notify_1067c2f0_needs_an_attack_with_a_target() {
        let class = vec!["ChrSound.class_1".to_owned()];
        let config = SoundConfig::default();
        let base = ItemContext { object_id: 4, position: Vec3::ZERO, npc: false, has_target: true, attacking: true, weapon_sounds: &[], npc_record: None, char_sounds: &class };
        assert!(plan_attack_item(&ItemContext { attacking: false, ..base }, &config, || 0).is_none());
        assert!(plan_attack_item(&ItemContext { has_target: false, ..base }, &config, || 0).is_none());
        assert!(plan_attack_item(&ItemContext { npc: true, ..base }, &config, || 0).is_none());
        let none_named = vec!["None".to_owned()];
        assert!(plan_attack_item(&ItemContext { char_sounds: &none_named, ..base }, &config, || 0).is_none());
    }

    fn footsteps_config() -> SoundConfig {
        SoundConfig::read(&killer_ui::ini::parse_ini(
            "[L2FootStepSound]\nDefWalkSound1=S.def_walk_1\nDefWalkSound2=S.def_walk_2\nDefWalkSound3=S.def_walk_3\nDefRunSound1=S.def_run_1\nGrassWalkSound1=S.grass_walk_1\nGrassRunSound3=S.grass_run_2\nWaterWalkSound2=S.water_2\nDefActorWalkSound1=S.stone_walk_1\nDefActorRunSound1=S.stone_run_1\n[SwimSound]\nWaterSurfaceSound1=C.surface_1\nWaterSurfaceSoundVol=150\nWaterSurfaceSoundRadius=60\nUnderWaterSound2=C.under_2\nUnderWaterSoundVol=250\n",
        ))
    }

    fn step_context(boots: &[String]) -> FootstepContext<'_> {
        FootstepContext { object_id: 8, position: Vec3::new(1.0, 2.0, 3.0), npc: false, running: false, boots, terrain: true, grass: false, water: false, step_volume: false, step_names: None }
    }

    #[test]
    fn alaudio_int_footstep_and_swim_sections_are_read_by_key() {
        let config = footsteps_config();
        assert_eq!(config.footsteps[0][0].as_deref(), Some("S.def_walk_1"));
        assert_eq!(config.footsteps[3][2].as_deref(), Some("S.grass_run_2"));
        assert_eq!(config.footsteps[1][2], None);
        assert_eq!((config.surface_swim.volume, config.surface_swim.radius, config.surface_swim.random), (150.0, 60.0, 100));
        assert_eq!(config.under_water.names[1].as_deref(), Some("C.under_2"));
        assert_eq!(config.under_water.volume, 250.0);
    }

    #[test]
    fn get_sound_106710c0_picks_the_table_by_move_type_and_surface_with_the_actor_run_quirk() {
        let config = footsteps_config();
        assert_eq!(footstep_name(&config, false, Surface::Land, 5).map(String::as_str), Some("S.def_walk_3"));
        assert_eq!(footstep_name(&config, true, Surface::Land, 3).map(String::as_str), Some("S.def_run_1"));
        assert_eq!(footstep_name(&config, false, Surface::Grass, 0).map(String::as_str), Some("S.grass_walk_1"));
        assert_eq!(footstep_name(&config, true, Surface::Grass, 2).map(String::as_str), Some("S.grass_run_2"));
        assert_eq!(footstep_name(&config, false, Surface::Water, 1).map(String::as_str), Some("S.water_2"));
        assert_eq!(footstep_name(&config, true, Surface::Actor, 0).map(String::as_str), Some("S.stone_walk_1"));
        assert_eq!(footstep_name(&config, false, Surface::Actor, 0).map(String::as_str), Some("S.stone_walk_1"));
    }

    #[test]
    fn sound_notify_1067da20_footsteps_layer_the_boots_sound_over_the_surface_sound_and_skip_npcs() {
        let config = footsteps_config();
        let notify = SoundNotify { volume: 250.0, radius: 60, random: 100, ..SoundNotify::default() };
        let boots = vec!["ItemSound.boots_1".to_owned(), "ItemSound.boots_2".to_owned()];
        let plans = plan_footsteps(&notify, &step_context(&boots), &config, || 1);
        assert_eq!(plans.iter().map(|plan| plan.name.as_str()).collect::<Vec<_>>(), ["ItemSound.boots_2", "S.def_walk_2"]);
        assert!(plans.iter().all(|plan| plan.radius == 60.0 && !plan.skip_if_playing));
        let bare = plan_footsteps(&notify, &step_context(&[]), &config, || 0);
        assert_eq!(bare.len(), 1);
        let indoor = plan_footsteps(&notify, &FootstepContext { terrain: false, ..step_context(&[]) }, &config, || 0);
        assert_eq!(indoor[0].name, "S.stone_walk_1");
        let grass = plan_footsteps(&notify, &FootstepContext { grass: true, ..step_context(&[]) }, &config, || 0);
        assert_eq!(grass[0].name, "S.grass_walk_1");
        assert!(plan_footsteps(&notify, &FootstepContext { npc: true, ..step_context(&boots) }, &config, || 0).is_empty());
    }

    #[test]
    fn sound_notify_1067da20_a_step_volume_plays_its_music_info_name_instead_of_the_surface() {
        let config = footsteps_config();
        let notify = SoundNotify { volume: 250.0, radius: 60, random: 100, ..SoundNotify::default() };
        let names = vec!["S.step_a".to_owned(), "S.step_b".to_owned(), "S.step_c".to_owned()];
        let boots = vec!["ItemSound.boots_1".to_owned()];
        let stepped = FootstepContext { step_volume: true, step_names: Some(names.as_slice()), ..step_context(&boots) };
        let mut rolls = [0, 5].into_iter();
        let plans = plan_footsteps(&notify, &stepped, &config, || rolls.next().unwrap_or(0));
        assert_eq!(plans.iter().map(|plan| plan.name.as_str()).collect::<Vec<_>>(), ["ItemSound.boots_1", "S.step_c"]);
        let silent = FootstepContext { step_volume: true, step_names: None, ..step_context(&[]) };
        assert!(plan_footsteps(&notify, &silent, &config, || 0).is_empty());
        let wet = FootstepContext { step_volume: true, step_names: Some(names.as_slice()), water: true, ..step_context(&[]) };
        assert_eq!(plan_footsteps(&notify, &wet, &config, || 1)[0].name, "S.water_2");
    }

    #[test]
    fn swim_sound_notify_1067d5d0_needs_a_water_volume_and_uses_the_surface_set_only_when_afloat_and_moving() {
        let config = footsteps_config();
        let underwater = SwimContext { object_id: 9, position: Vec3::ZERO, in_water_volume: true, afloat: false, moving_in_water: true };
        let mut values = [200, 4].into_iter();
        let plan = plan_swim(&underwater, &config, || values.next().unwrap()).expect("toca");
        assert_eq!((plan.name.as_str(), plan.radius), ("C.under_2", 50.0));
        assert!((plan.volume - 250.0 / 255.0).abs() < 1e-6);
        let mut quiet = [99].into_iter();
        assert!(plan_swim(&underwater, &config, || quiet.next().unwrap()).is_none());
        assert!(plan_swim(&SwimContext { in_water_volume: false, ..underwater }, &config, || 500).is_none());
        let surface = SwimContext { afloat: true, ..underwater };
        let mut values = [150, 0].into_iter();
        let plan = plan_swim(&surface, &config, || values.next().unwrap()).expect("toca");
        assert_eq!((plan.name.as_str(), plan.radius), ("C.surface_1", 60.0));
    }

    #[test]
    #[ignore]
    fn every_footstep_swim_voice_and_item_sound_of_the_client_resolves() {
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
        let ini = killer_ui::ini::parse_ini(&client.read_system_text("alaudio.int").unwrap());
        let config = SoundConfig::read(&ini);
        let mut missing = Vec::new();
        let mut names: Vec<String> = config.footsteps.iter().flatten().flatten().cloned().collect();
        names.extend(config.surface_swim.names.iter().chain(&config.under_water.names).flatten().cloned());
        assert_eq!(names.len(), 24 + 6);
        let tables = client.tables().unwrap();
        for template in &tables.chargrp {
            names.extend(template.attack_sounds.iter().chain(template.voice_sounds.iter().flatten()).cloned());
        }
        names.extend(tables.weapongrp.values().flat_map(|weapon| weapon.attack_sounds.iter().cloned()));
        names.extend(tables.armorgrp.values().flat_map(|armor| armor.boots_sounds.iter().cloned()));
        names.extend(tables.npcgrp.iter().flat_map(|npc| npc.attack_sounds.iter().cloned()));
        names.retain(|name| real_name(name));
        names.sort();
        names.dedup();
        for name in &names {
            if client.sound(name).is_none() {
                missing.push(name.clone());
            }
        }
        eprintln!("sons conferidos: {} ausentes: {} {:?}", names.len(), missing.len(), &missing[..missing.len().min(10)]);
        assert!(names.len() > 300);
        assert!(missing.is_empty(), "{} de {} ausentes", missing.len(), names.len());
    }
}
