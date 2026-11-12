use crate::abnormal_effects::AbnormalEffectHost;
use crate::pawn_light::{AbnormalColor, PawnLight, add_abnormal_light};

pub const STATE_BLEED: u32 = 0x1;
pub const STATE_POISON: u32 = 0x2;
pub const STATE_RED_CIRCLE: u32 = 0x4;
pub const STATE_ICE: u32 = 0x8;
pub const STATE_WIND: u32 = 0x10;
pub const STATE_FEAR: u32 = 0x20;
pub const STATE_STUN: u32 = 0x40;
pub const STATE_SLEEP: u32 = 0x80;
pub const STATE_MUTED: u32 = 0x100;
pub const STATE_ROOT: u32 = 0x200;
pub const STATE_HOLD: u32 = 0x400;
pub const STATE_OVERLAY: u32 = 0x800;
pub const STATE_BIG_HEAD: u32 = 0x2000;
pub const STATE_FLAME: u32 = 0x4000;
pub const STATE_MATERIAL_SEQUENCE: u32 = 0x8000;
pub const STATE_GROW: u32 = 0x10000;

pub const GROW_IDLE: u32 = 0;
pub const GROW_GROWING: u32 = 1;
pub const GROW_SHRINKING: u32 = 2;
pub const GROW_HELD: u32 = 3;
pub const GROW_TARGET: f32 = f32::from_bits(0x3f99_999a);
pub const GROW_UNIT_BITS: u32 = 0x3f80_0000;
pub const SHRINK_RATE: f64 = 3.0;

pub const RENEW_SECONDS: f32 = 3.0;
pub const BIG_HEAD_RATE: f64 = 3.0;
pub const SOIL_SHOT_SOUND: &str = "skillsound3.soil_shot";
pub const CURSE_POISON_SOUND: &str = "skillsound.curse_poison_shot";
pub const EXTRA_SOUND_VOLUME: f32 = 1.0;
pub const EXTRA_SOUND_RADIUS: f32 = 5.0;
pub const BIG_HEAD_DEFAULT_MAX: f32 = 3.0;
pub const BIG_HEAD_FILE: &str = "bighead.int";
pub const OVERLAY_COLOR: [u8; 3] = [0x48, 0x69, 0x8c];
pub const ALIAS_BONE: &str = "e_bone";
pub const SIZE_RADIUS_DIVISOR: f32 = 9.0;
pub const SCALE_REFERENCE: f64 = 207.0;
pub const CREATION_LIMIT: u32 = 0x20000;

pub fn effect_class(mask: u32) -> Option<&'static str> {
    match mask {
        STATE_BLEED | STATE_RED_CIRCLE | STATE_ICE | STATE_WIND => Some("LineageEffect.a_u001_a"),
        STATE_POISON => Some("LineageEffect.a_u000_a"),
        STATE_FEAR => Some("LineageEffect.a_u008_a"),
        STATE_STUN => Some("LineageEffect.a_u002_a"),
        STATE_SLEEP => Some("LineageEffect.a_u004_a"),
        STATE_MUTED => Some("LineageEffect.a_u005_a"),
        STATE_ROOT => Some("LineageEffect.a_u006_a"),
        STATE_FLAME => Some("LineageEffect.a_u007_a"),
        _ => None,
    }
}

pub trait AbnormalStatHost: AbnormalEffectHost {
    type Class: Copy;

    fn has_skeletal_mesh(&self, owner: Self::Actor) -> bool;
    fn pawn_drawn(&self, owner: Self::Actor) -> bool;
    fn load_effect_class(&mut self, name: &str) -> Option<Self::Class>;
    fn spawn_emitter(&mut self, class: Self::Class, owner: Self::Actor, location: [f32; 3], rotation: [i32; 3]) -> Option<Self::Emitter>;
    fn location_rotation(&self, owner: Self::Actor) -> ([f32; 3], [i32; 3]);
    fn collision(&self, owner: Self::Actor) -> (f32, f32);
    fn draw_scale(&self, owner: Self::Actor) -> f32;
    fn alias_bone(&self, owner: Self::Actor, alias: &str) -> Option<Self::Name>;
    fn alias_origin(&self, owner: Self::Actor, alias: &str) -> [f32; 3];
    fn mesh_origin_z(&self, owner: Self::Actor) -> f32;
    fn attach_to_bone(&mut self, owner: Self::Actor, emitter: Self::Emitter, bone: Self::Name, relative: [f32; 3]) -> bool;
    fn attach_to_root(&mut self, owner: Self::Actor, emitter: Self::Emitter);
    fn set_pre_pivot_z(&mut self, emitter: Self::Emitter, value: f32);
    fn set_size_scale(&mut self, emitter: Self::Emitter, scale: f32);
    fn set_overlay(&mut self, owner: Self::Actor, rgb: [u8; 3], mode: u8);
    fn stop_action(&mut self, owner: Self::Actor);
    fn head_scale(&self, owner: Self::Actor) -> f32;
    fn set_bone_scale(&mut self, owner: Self::Actor, scale: f32, head: bool);
    fn big_head_max(&mut self, owner: Self::Actor) -> f32;
    fn play_damage_sound(&mut self, owner: Self::Actor);
    fn pawn_lights(&mut self, owner: Self::Actor) -> Option<&mut Vec<PawnLight>>;
    fn play_sound(&mut self, owner: Self::Actor, name: &str, volume: f32, radius: f32);
    fn is_npc(&self, actor: Self::Actor) -> bool;
    fn npc_texture_lists(&mut self, actor: Self::Actor) -> Option<NpcTextureLists<Self::Name>>;
    fn grow_dimensions(&self, actor: Self::Actor) -> GrowDimensions;
    fn set_collision_size(&mut self, actor: Self::Actor, size: [f32; 2]);
    fn set_location_z(&mut self, actor: Self::Actor, z: f32);
    fn set_weapon_scale(&mut self, actor: Self::Actor, scale: f32);
}

pub struct NpcTextureLists<Name> {
    pub normal: Vec<Name>,
    pub abnormal: Vec<Name>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GrowDimensions {
    pub collision: [f32; 2],
    pub base_collision: [f32; 2],
    pub z: f32,
    pub weapon_scale: f32,
}

pub struct AbnormalStat<Actor, Emitter> {
    pub owner: Actor,
    pub mask: u32,
    pub elapsed: f32,
    pub duration: f32,
    pub class_name: Option<&'static str>,
    pub emitter: Option<Emitter>,
    pub stopped: bool,
    pub scale: f32,
    pub state: u32,
    pub saved_collision: [f32; 2],
    pub saved_weapon_scale: f32,
}

impl<Actor: Copy, Emitter: Copy> AbnormalStat<Actor, Emitter> {
    fn new(owner: Actor) -> Self {
        Self { owner, mask: 0, elapsed: 0.0, duration: 0.0, class_name: None, emitter: None, stopped: false, scale: 1.0, state: GROW_IDLE, saved_collision: [0.0; 2], saved_weapon_scale: 0.0 }
    }

    pub fn into_effect(self, resource: String) -> crate::abnormal_effects::AbnormalEffect<Actor, Emitter> {
        crate::abnormal_effects::AbnormalEffect { owner: Some(self.owner), emitter: self.emitter, mask: self.mask, resource, saved_field_4bc: self.saved_weapon_scale.to_bits() }
    }
}

pub fn check_state<H: AbnormalStatHost>(host: &H, stat: &AbnormalStat<H::Actor, H::Emitter>, mask: u32) -> bool {
    host.is_pawn(stat.owner) && stat.mask & mask != 0
}

pub fn big_head_active<H: AbnormalStatHost>(host: &H, stats: &[AbnormalStat<H::Actor, H::Emitter>]) -> bool {
    stats.iter().any(|stat| check_state(host, stat, STATE_BIG_HEAD))
}

fn pawn_scale(radius: f32, height: f32) -> f32 {
    ((f64::from(height) * f64::from(radius)).sqrt() / SCALE_REFERENCE.sqrt()) as f32
}

fn alias_offset<H: AbnormalStatHost>(host: &H, owner: H::Actor, bighead: bool) -> [f32; 3] {
    let origin = host.alias_origin(owner, ALIAS_BONE);
    if bighead {
        let scale = host.head_scale(owner);
        origin.map(|value| value * scale)
    } else {
        origin
    }
}

fn spawn_at_pawn<H: AbnormalStatHost>(host: &mut H, name: &str, owner: H::Actor) -> Option<H::Emitter> {
    let class = host.load_effect_class(name)?;
    let (location, rotation) = host.location_rotation(owner);
    host.spawn_emitter(class, owner, location, rotation)
}

fn attach_alias<H: AbnormalStatHost>(host: &mut H, owner: H::Actor, emitter: H::Emitter, bone: H::Name, bighead: bool) {
    let relative = alias_offset(host, owner, bighead);
    if !host.attach_to_bone(owner, emitter, bone, relative) {
        host.attach_to_root(owner, emitter);
    }
}

pub fn init<H: AbnormalStatHost>(host: &mut H, owner: H::Actor, bit: u32, bighead: bool) -> AbnormalStat<H::Actor, H::Emitter> {
    let mut stat = AbnormalStat::new(owner);
    if !host.has_skeletal_mesh(owner) {
        return stat;
    }
    stat.mask = bit;
    match bit {
        STATE_BLEED | STATE_POISON | STATE_RED_CIRCLE | STATE_ICE | STATE_WIND | STATE_FEAR | STATE_FLAME => {
            stat.class_name = effect_class(bit);
            stat.duration = RENEW_SECONDS;
        }
        STATE_STUN | STATE_SLEEP => init_attached_stop(host, &mut stat, bit, bighead),
        STATE_MUTED => init_attached(host, &mut stat, bit, bighead),
        STATE_ROOT => init_ground(host, &mut stat),
        STATE_HOLD if host.is_pawn(owner) && host.has_mesh_instance(owner) => {
            host.stop_action(owner);
            stat.stopped = true;
        }
        STATE_OVERLAY => init_overlay(host, &mut stat),
        STATE_BIG_HEAD if host.is_pawn(owner) && host.has_mesh_instance(owner) => {
            host.set_bone_scale(owner, stat.scale, true);
            stat.duration = host.big_head_max(owner);
        }
        STATE_MATERIAL_SEQUENCE => init_material_sequence(host, owner),
        STATE_GROW => init_grow(host, &mut stat),
        _ => {}
    }
    stat
}

fn init_material_sequence<H: AbnormalStatHost>(host: &mut H, owner: H::Actor) {
    if !host.is_pawn(owner) || !host.has_mesh_instance(owner) || !host.is_npc(owner) {
        return;
    }
    let Some(lists) = host.npc_texture_lists(owner) else { return };
    for (index, name) in abnormal_texture_slots(&lists) {
        if let Some(material) = host.find_material_sequence(name).or_else(|| host.load_material_sequence(name)) {
            host.trigger_material_sequence(material);
        }
        host.set_npc_texture(owner, name, index);
    }
}

pub fn abnormal_texture_slots<Name: Copy>(lists: &NpcTextureLists<Name>) -> Vec<(usize, Name)> {
    (0..lists.normal.len()).filter_map(|index| lists.abnormal.get(index).map(|name| (index, *name))).collect()
}

fn grow_pawn<H: AbnormalStatHost>(host: &H, owner: H::Actor) -> Option<H::Actor> {
    host.is_pawn(owner).then(|| host.active_associated_pawn(owner).unwrap_or(owner))
}

fn init_grow<H: AbnormalStatHost>(host: &mut H, stat: &mut AbnormalStat<H::Actor, H::Emitter>) {
    let Some(pawn) = grow_pawn(host, stat.owner) else { return };
    if !host.is_npc(pawn) || host.skeletal_instance(pawn).is_none() {
        return;
    }
    stat.elapsed = 0.0;
    stat.scale = 1.0;
    stat.duration = GROW_TARGET;
    stat.state = GROW_GROWING;
    let dimensions = host.grow_dimensions(pawn);
    stat.saved_collision = dimensions.collision;
    host.set_collision_size(pawn, dimensions.base_collision);
    stat.saved_weapon_scale = dimensions.weapon_scale;
}

fn grow_factor<Actor, Emitter>(stat: &AbnormalStat<Actor, Emitter>) -> f64 {
    if stat.duration.to_bits() == GROW_UNIT_BITS { 0.0 } else { (f64::from(stat.scale) - 1.0) / (f64::from(stat.duration) - 1.0) }
}

fn scale_weapon<H: AbnormalStatHost>(host: &mut H, stat: &AbnormalStat<H::Actor, H::Emitter>, pawn: H::Actor) {
    if host.grow_dimensions(pawn).weapon_scale != 0.0 {
        host.set_weapon_scale(pawn, (f64::from(stat.saved_weapon_scale) * f64::from(stat.scale)) as f32);
    }
}

fn grow_step<H: AbnormalStatHost>(host: &mut H, stat: &mut AbnormalStat<H::Actor, H::Emitter>, pawn: H::Actor, dt: f32) {
    let scale = f64::from(dt) * f64::from(stat.elapsed) + f64::from(stat.scale);
    stat.scale = scale as f32;
    if scale >= f64::from(stat.duration) {
        stat.scale = stat.duration;
        stat.state = GROW_HELD;
    }
    host.set_bone_scale(pawn, stat.scale, false);
    scale_weapon(host, stat, pawn);
    let dimensions = host.grow_dimensions(pawn);
    let old_height = f64::from(dimensions.collision[1]);
    let factor = grow_factor(stat);
    let [base_radius, base_height] = dimensions.base_collision.map(f64::from);
    let [saved_radius, saved_height] = stat.saved_collision.map(f64::from);
    let radius = ((saved_radius - base_radius) * factor + base_radius) as f32;
    let height = (saved_height - base_height) * factor + base_height;
    host.set_collision_size(pawn, [radius, height as f32]);
    host.set_location_z(pawn, ((height - old_height) + f64::from(dimensions.z)) as f32);
}

fn shrink_step<H: AbnormalStatHost>(host: &mut H, stat: &mut AbnormalStat<H::Actor, H::Emitter>, pawn: H::Actor, dt: f32) {
    let scale = f64::from(stat.scale) - f64::from(dt) * f64::from(stat.elapsed) * SHRINK_RATE;
    stat.scale = scale as f32;
    if scale <= 1.0 {
        stat.scale = 1.0;
        stat.state = GROW_IDLE;
    }
    host.set_bone_scale(pawn, stat.scale, false);
    scale_weapon(host, stat, pawn);
    if stat.elapsed == 0.0 {
        let base = host.grow_dimensions(pawn).base_collision;
        host.set_collision_size(pawn, base);
    }
    let dimensions = host.grow_dimensions(pawn);
    let old_height = f64::from(dimensions.collision[1]);
    let factor = grow_factor(stat);
    let [base_radius, base_height] = dimensions.base_collision.map(f64::from);
    let [saved_radius, saved_height] = stat.saved_collision.map(f64::from);
    let radius = ((base_radius - saved_radius) * factor + saved_radius) as f32;
    let height = (base_height - saved_height) * factor + saved_height;
    host.set_collision_size(pawn, [radius, height as f32]);
    host.set_location_z(pawn, (f64::from(dimensions.z) - (old_height - height)) as f32);
}

fn update_grow<H: AbnormalStatHost>(host: &mut H, stat: &mut AbnormalStat<H::Actor, H::Emitter>, dt: f32) -> bool {
    if let Some(pawn) = grow_pawn(host, stat.owner).filter(|pawn| host.skeletal_instance(*pawn).is_some()) {
        match stat.state {
            GROW_GROWING => grow_step(host, stat, pawn, dt),
            GROW_SHRINKING => shrink_step(host, stat, pawn, dt),
            GROW_HELD => {
                stat.saved_collision = host.grow_dimensions(pawn).base_collision;
                stat.elapsed = 0.0;
                return true;
            }
            _ => {}
        }
    }
    stat.elapsed += dt;
    true
}

fn init_attached_stop<H: AbnormalStatHost>(host: &mut H, stat: &mut AbnormalStat<H::Actor, H::Emitter>, bit: u32, bighead: bool) {
    let owner = stat.owner;
    let Some(name) = effect_class(bit) else { return };
    if !host.is_pawn(owner) || !host.has_mesh_instance(owner) {
        return;
    }
    let bone = host.alias_bone(owner, ALIAS_BONE);
    let emitter = bone.and_then(|_| spawn_at_pawn(host, name, owner));
    if let (Some(emitter), Some(bone)) = (emitter, bone) {
        stat.emitter = Some(emitter);
        attach_alias(host, owner, emitter, bone, bighead);
    }
    stat.stopped = true;
    host.stop_action(owner);
}

fn init_attached<H: AbnormalStatHost>(host: &mut H, stat: &mut AbnormalStat<H::Actor, H::Emitter>, bit: u32, bighead: bool) {
    let owner = stat.owner;
    let Some(name) = effect_class(bit) else { return };
    if !host.is_pawn(owner) || !host.has_mesh_instance(owner) {
        return;
    }
    let Some(bone) = host.alias_bone(owner, ALIAS_BONE) else { return };
    if let Some(emitter) = spawn_at_pawn(host, name, owner) {
        stat.emitter = Some(emitter);
        attach_alias(host, owner, emitter, bone, bighead);
    }
}

fn init_ground<H: AbnormalStatHost>(host: &mut H, stat: &mut AbnormalStat<H::Actor, H::Emitter>) {
    let owner = stat.owner;
    let Some(name) = effect_class(STATE_ROOT) else { return };
    if !host.is_pawn(owner) || !host.has_mesh_instance(owner) {
        return;
    }
    let Some(class) = host.load_effect_class(name) else {
        stat.stopped = true;
        return;
    };
    let (mut location, rotation) = host.location_rotation(owner);
    let (radius, height) = host.collision(owner);
    location[2] -= height;
    if let Some(emitter) = host.spawn_emitter(class, owner, location, rotation) {
        stat.emitter = Some(emitter);
        host.set_pre_pivot_z(emitter, -host.draw_scale(owner) * host.mesh_origin_z(owner));
        host.set_size_scale(emitter, radius / SIZE_RADIUS_DIVISOR);
    }
    stat.stopped = true;
}

fn init_overlay<H: AbnormalStatHost>(host: &mut H, stat: &mut AbnormalStat<H::Actor, H::Emitter>) {
    let owner = stat.owner;
    host.set_overlay(owner, OVERLAY_COLOR, 1);
    if !host.is_pawn(owner) {
        return;
    }
    if host.has_mesh_instance(owner) {
        stat.stopped = true;
        host.stop_action(owner);
    }
    if let Some(associated) = host.active_associated_pawn(owner) {
        host.set_overlay(associated, OVERLAY_COLOR, 1);
    }
}

pub fn update<H: AbnormalStatHost>(host: &mut H, stat: &mut AbnormalStat<H::Actor, H::Emitter>, dt: f32, bighead: bool) -> bool {
    let owner = stat.owner;
    if !host.has_skeletal_mesh(owner) {
        return false;
    }
    match stat.mask {
        STATE_BLEED | STATE_RED_CIRCLE | STATE_ICE | STATE_WIND | STATE_FEAR => periodic(host, stat, dt, bighead, Periodic::Root),
        STATE_POISON => periodic(host, stat, dt, bighead, Periodic::Alias),
        STATE_FLAME => periodic(host, stat, dt, bighead, Periodic::Flame),
        STATE_BIG_HEAD => {
            if host.is_pawn(owner) && host.has_mesh_instance(owner) && stat.scale < stat.duration {
                stat.scale = ((f64::from(dt) * f64::from(stat.elapsed)) * BIG_HEAD_RATE + f64::from(stat.scale)) as f32;
                host.set_bone_scale(owner, stat.scale, true);
            }
            stat.elapsed += dt;
            stat.elapsed += dt;
            true
        }
        STATE_GROW => update_grow(host, stat, dt),
        _ => {
            stat.elapsed += dt;
            true
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Periodic {
    Root,
    Alias,
    Flame,
}

fn periodic<H: AbnormalStatHost>(host: &mut H, stat: &mut AbnormalStat<H::Actor, H::Emitter>, dt: f32, bighead: bool, kind: Periodic) -> bool {
    let mut rest = 0.0;
    if stat.elapsed > stat.duration {
        rest = stat.elapsed - stat.duration;
        stat.elapsed = 0.0;
    }
    if stat.elapsed != 0.0 {
        stat.elapsed += dt;
        return true;
    }
    let owner = stat.owner;
    if let Some(emitter) = stat.emitter.take() {
        host.destroy_emitter(owner, emitter);
    }
    if !host.is_pawn(owner) || !host.has_mesh_instance(owner) || !host.pawn_drawn(owner) {
        return false;
    }
    let Some(name) = stat.class_name else { return false };
    let bone = match kind {
        Periodic::Root => None,
        Periodic::Alias | Periodic::Flame => host.alias_bone(owner, ALIAS_BONE),
    };
    if kind == Periodic::Root || bone.is_some() {
        let Some(class) = host.load_effect_class(name) else { return false };
        let (location, rotation) = host.location_rotation(owner);
        let (radius, height) = host.collision(owner);
        if let Some(emitter) = host.spawn_emitter(class, owner, location, rotation) {
            stat.emitter = Some(emitter);
            match kind {
                Periodic::Root => {
                    host.attach_to_root(owner, emitter);
                    let scale = pawn_scale(radius, height);
                    if scale > 1.0 {
                        host.set_size_scale(emitter, scale);
                    }
                }
                Periodic::Alias => {
                    if let Some(bone) = bone {
                        attach_alias(host, owner, emitter, bone, bighead);
                    }
                }
                Periodic::Flame => {
                    let head = if bighead { host.head_scale(owner) } else { 1.0 };
                    host.set_pre_pivot_z(emitter, -head * host.mesh_origin_z(owner));
                    host.set_size_scale(emitter, radius / SIZE_RADIUS_DIVISOR);
                }
            }
        }
    }
    let (radius, height) = host.collision(owner);
    let color = if stat.mask == STATE_POISON { AbnormalColor::Purple } else { AbnormalColor::Red };
    if let Some(lights) = host.pawn_lights(owner) {
        add_abnormal_light(lights, radius, height, color);
    }
    host.play_damage_sound(owner);
    let extra = match stat.mask {
        STATE_FEAR => Some(SOIL_SHOT_SOUND),
        STATE_POISON => Some(CURSE_POISON_SOUND),
        _ => None,
    };
    if let Some(name) = extra {
        host.play_sound(owner, name, EXTRA_SOUND_VOLUME, EXTRA_SOUND_RADIUS);
    }
    stat.elapsed = rest + dt;
    true
}

pub fn reconcile<H: AbnormalStatHost>(host: &mut H, owner: H::Actor, stats: &mut Vec<AbnormalStat<H::Actor, H::Emitter>>, pawn_mask: u32, dt: f32) -> Vec<AbnormalStat<H::Actor, H::Emitter>> {
    let mut removed = Vec::new();
    let mut local = pawn_mask;
    let mut index = stats.len();
    while index > 0 {
        index -= 1;
        if check_state(host, &stats[index], pawn_mask) {
            local &= !stats[index].mask;
        } else if stats[index].state == GROW_IDLE {
            removed.push(stats.remove(index));
        } else if stats[index].state == GROW_HELD {
            stats[index].state = GROW_SHRINKING;
        }
    }
    if (local as i32) > 0 {
        let mut bit = 1u32;
        while bit < CREATION_LIMIT {
            if local & bit != 0 {
                let bighead = big_head_active(host, stats);
                stats.insert(0, init(host, owner, bit, bighead));
            }
            bit <<= 1;
        }
    }
    let mut index = stats.len();
    while index > 0 {
        index -= 1;
        let bighead = big_head_active(host, stats);
        let mut stat = stats.remove(index);
        if update(host, &mut stat, dt, bighead) {
            stats.insert(index, stat);
        } else {
            removed.push(stat);
        }
    }
    removed
}
