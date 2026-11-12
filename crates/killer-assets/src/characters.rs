use crate::client::Client;
use crate::pawn::SLOT_FACE;
pub use crate::pawn::{Appearance, Part, Placement};
use crate::scene::transform::rotator_matrix;
use crate::scene::{Pass, draw_pass, texture_material};
use glam::{Mat4, Quat, Vec3};
use l2_catalog::Material;
use ue2_assets::{Alias, Bone, SkinVertex};

pub use crate::pawn::{
    HEAD_BONE, LEFT_ARM_BONE as SHIELD_L_BONE, LEFT_HAND_BONE as WEAPON_L_BONE, RIGHT_ARM_BONE as SHIELD_R_BONE,
    RIGHT_HAND_BONE as WEAPON_R_BONE,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NpcFallback {
    pub draw_scale: f32,
    pub collision_height: f32,
    pub collision_radius: f32,
}

pub const NPC_FALLBACK: NpcFallback = NpcFallback { draw_scale: 10.0, collision_height: 100.0, collision_radius: 50.0 };

pub fn fallback_npc_appearance() -> Appearance {
    let part = Part::new("LineageMonsters.rabbit_m00", vec!["LineageMonstersTex.rabbit_t00".to_string()]);
    Appearance { class_name: Some("LineageMonster.Rabbit".into()), parts: vec![part], weapon_type: 0, cloak: None, enchant_effect: 0, weapon_effects: Vec::new(), hero_aura: false, damage_effect: Some(crate::pawn::NPC_DAMAGE_EFFECT.into()), arrow: None }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharacterSection {
    pub first_index: u32,
    pub index_count: u32,
    pub material: Material,
    pub pass: Pass,
}

impl CharacterSection {
    pub fn texture(&self) -> &str {
        self.material.first_texture().map_or("", |texture| texture.path.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharacterLod {
    pub source: Part,
    pub index: usize,
    pub settings: ue2_assets::SkeletalLodSettings,
    pub levels: Vec<ue2_assets::SkeletalLodInfo>,
    pub section_material_slots: Vec<u16>,
    pub material_texture_indices: Vec<i32>,
    pub textures: Vec<Option<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharacterMesh {
    pub lod: Option<CharacterLod>,
    pub bounding_box: [[f32; 3]; 2],
    pub bounding_sphere: [f32; 4],
    pub slot: u8,
    pub placement: Placement,
    pub bones: Vec<Bone>,
    pub vertices: Vec<SkinVertex>,
    pub indices: Vec<u32>,
    pub sections: Vec<CharacterSection>,
    pub bind: Vec<Mat4>,
    pub inverse_bind: Vec<Mat4>,
    pub animation: Option<String>,
    pub local: Mat4,
    pub origin: Vec3,
    pub rotation: [i32; 3],
    pub scale: Vec3,
    pub aliases: Vec<Alias>,
    pub hair_kind: i32,
    pub floats_150: Vec<f32>,
}

impl CharacterMesh {
    pub fn empty() -> Self {
        Self {
            lod: None,
            bounding_box: [[0.0; 3]; 2],
            bounding_sphere: [0.0; 4],
            slot: SLOT_FACE,
            placement: Placement::Master,
            bones: Vec::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
            sections: Vec::new(),
            bind: Vec::new(),
            inverse_bind: Vec::new(),
            animation: None,
            local: Mat4::IDENTITY,
            origin: Vec3::ZERO,
            rotation: [0; 3],
            scale: Vec3::ONE,
            aliases: Vec::new(),
            hair_kind: 0,
            floats_150: Vec::new(),
        }
    }

    pub fn is_master(&self) -> bool {
        self.placement == Placement::Master
    }

    pub fn alias(&self, name: &str) -> Option<&Alias> {
        self.aliases.iter().find(|a| a.name.eq_ignore_ascii_case(name))
    }

    pub fn match_ref_bone(&self, name: &str) -> Option<usize> {
        let name = self.alias(name).map_or(name, |a| a.bone.as_str());
        self.bones.iter().position(|b| b.name.eq_ignore_ascii_case(name))
    }
}

pub fn load_part(client: &mut Client, part: &Part) -> Option<CharacterMesh> {
    load_part_lod(client, part, 0)
}

pub fn load_part_lod(client: &mut Client, part: &Part, lod: usize) -> Option<CharacterMesh> {
    let (packed, lod_info) = match client.catalog().try_skeletal_mesh_lod_with_info(&part.mesh, lod) {
        Ok(packed) => packed,
        Err(why) => {
            log::warn!("malha {} (slot {}, LOD {lod}): {why}", part.mesh, part.slot);
            return None;
        }
    };
    let mesh = packed.mesh;
    let mut bindings = match crate::part_materials::PartMaterials::load(part, &packed.textures, |name| {
        client.catalog().object_class(name).is_some()
    }) {
        Ok(bindings) => bindings,
        Err(error) => {
            log::warn!("malha {} (slot {}): materiais inválidos: {error:?}", part.mesh, part.slot);
            return None;
        }
    };
    for (index, name) in &bindings.missing {
        log::warn!("malha {} (slot {}): textura {index} ausente no catálogo: {name}", part.mesh, part.slot);
    }
    let mut sections = Vec::with_capacity(mesh.sections.len());
    for (index, section) in mesh.sections.iter().enumerate() {
        let path = match bindings.section(index, &mesh.section_material_slots, &mesh.material_texture_indices) {
            Ok(path) => path.map(str::to_owned),
            Err(error) => {
                log::warn!("malha {} (slot {}) seção {index}: {error:?}", part.mesh, part.slot);
                return None;
            }
        };
        let Some(path) = path.or_else(|| client.catalog().default_material_path()) else {
            log::warn!("malha {} (slot {}) seção {index}: DefaultMaterial original indisponível", part.mesh, part.slot);
            return None;
        };
        let material = client.catalog().material(&path);
        if material.is_none()
            && let Err(why) = client.catalog().try_texture_raw(&path)
        {
            log::warn!("malha {} (slot {}) seção {index}: textura {path}: {why}", part.mesh, part.slot);
        }
        let material = material.unwrap_or_else(|| texture_material(&path));
        let pass = draw_pass(&material);
        sections.push(CharacterSection { first_index: section.first_index, index_count: section.index_count(), material, pass });
    }
    let bind = bind_pose(&mesh.bones);
    let inverse_bind = bind.iter().map(Mat4::inverse).collect();
    Some(CharacterMesh {
        lod: Some(CharacterLod {
            source: part.clone(),
            index: lod,
            settings: mesh.lod_settings,
            levels: lod_info,
            section_material_slots: mesh.section_material_slots,
            material_texture_indices: mesh.material_texture_indices,
            textures: packed.textures,
        }),
        bounding_box: mesh.bounding_box,
        bounding_sphere: mesh.bounding_sphere,
        slot: part.slot,
        placement: part.placement,
        local: mesh_local(mesh.origin, mesh.rotation, mesh.scale),
        origin: Vec3::from(mesh.origin),
        rotation: mesh.rotation,
        scale: Vec3::from(mesh.scale),
        bones: mesh.bones,
        vertices: mesh.vertices,
        indices: mesh.indices.iter().map(|&i| u32::from(i)).collect(),
        sections,
        bind,
        inverse_bind,
        animation: packed.animation,
        aliases: mesh.aliases,
        hair_kind: mesh.hair_kind,
        floats_150: mesh.floats_150.clone(),
    })
}

pub fn load_part_lod_with_info(
    client: &mut Client,
    part: &Part,
    lod: usize,
) -> Option<(CharacterMesh, Vec<ue2_assets::SkeletalLodInfo>)> {
    let mesh = load_part_lod(client, part, lod)?;
    let info = mesh.lod.as_ref()?.levels.clone();
    Some((mesh, info))
}

pub fn mesh_local(origin: [f32; 3], rotation: [i32; 3], scale: [f32; 3]) -> Mat4 {
    Mat4::from_mat3(rotator_matrix(rotation))
        * Mat4::from_scale(Vec3::from(scale))
        * Mat4::from_translation(-Vec3::from(origin))
}

pub fn bind_pose(bones: &[Bone]) -> Vec<Mat4> {
    let mut world: Vec<(Quat, Vec3)> = Vec::with_capacity(bones.len());
    for (index, bone) in bones.iter().enumerate() {
        world.push(compose(&world, index, bone.parent, bone.rotation, bone.position));
    }
    world.iter().map(|&(rotation, origin)| Mat4::from_rotation_translation(rotation, origin)).collect()
}

pub(crate) fn compose(
    world: &[(Quat, Vec3)],
    index: usize,
    parent: usize,
    rotation: [f32; 4],
    position: [f32; 3],
) -> (Quat, Vec3) {
    let rotation = Quat::from_array(rotation);
    let position = Vec3::from(position);
    match world.get(parent).filter(|_| index != 0) {
        Some(&(parent_rotation, parent_origin)) => {
            (parent_rotation * rotation.conjugate(), parent_origin + parent_rotation * position)
        }
        None => (rotation, position),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;
    use ue2_assets::Bone;

    fn bone(name: &str, parent: usize, rotation: [f32; 4], position: [f32; 3]) -> Bone {
        Bone { name: name.into(), parent, rotation, position }
    }

    #[test]
    fn bind_pose_composes_parent_then_conjugated_child() {
        let half = std::f32::consts::FRAC_PI_4;
        let root = bone("Bip01", 0, [0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 10.0]);
        let child = bone("Bip01_Spine", 0, [0.0, 0.0, half.sin(), half.cos()], [5.0, 0.0, 0.0]);
        let pose = bind_pose(&[root, child]);
        let origin = pose[1].transform_point3(Vec3::ZERO);
        assert!((origin - Vec3::new(5.0, 0.0, 10.0)).length() < 1e-5);
        let x = pose[1].transform_vector3(Vec3::X);
        assert!((x - Vec3::new(0.0, -1.0, 0.0)).length() < 1e-5);
    }

    #[test]
    fn bind_pose_root_rotation_is_used_as_stored() {
        let half = std::f32::consts::FRAC_PI_4;
        let root = bone("Bip01", 0, [0.0, 0.0, half.sin(), half.cos()], [1.0, 2.0, 3.0]);
        let child = bone("Bip01_Spine", 0, [0.0, 0.0, 0.0, 1.0], [10.0, 0.0, 0.0]);
        let pose = bind_pose(&[root, child]);
        assert!((pose[0].transform_vector3(Vec3::X) - Vec3::Y).length() < 1e-5);
        assert!((pose[1].transform_point3(Vec3::ZERO) - Vec3::new(1.0, 12.0, 3.0)).length() < 1e-5);
    }

    #[test]
    fn local_matrix_translates_by_minus_origin_then_scales_then_rotates() {
        let local = mesh_local([1.0, 2.0, 3.0], [0, 16384, 0], [2.0, 2.0, 2.0]);
        let p = local.transform_point3(Vec3::new(2.0, 2.0, 3.0));
        assert!((p - Vec3::new(0.0, 2.0, 0.0)).length() < 1e-4, "{p}");
        let stretched = mesh_local([1.0, 2.0, 3.0], [0, 16384, 0], [2.0, 3.0, 4.0]);
        let p = stretched.transform_point3(Vec3::new(2.0, 2.0, 3.0));
        assert!((p - Vec3::new(0.0, 2.0, 0.0)).length() < 1e-4, "{p}");
        let p = stretched.transform_point3(Vec3::new(1.0, 3.0, 4.0));
        assert!((p - Vec3::new(-3.0, 0.0, 4.0)).length() < 1e-4, "{p}");
        let identity = mesh_local([0.0; 3], [0; 3], [1.0; 3]);
        assert!((identity.transform_point3(Vec3::new(1.0, 2.0, 3.0)) - Vec3::new(1.0, 2.0, 3.0)).length() < 1e-6);
    }

    #[test]
    fn fallback_npc_is_the_giant_rabbit() {
        let a = fallback_npc_appearance();
        assert_eq!(a.parts.len(), 1);
        assert_eq!(a.parts[0].mesh, "LineageMonsters.rabbit_m00");
        assert_eq!(a.parts[0].textures, [Some("LineageMonstersTex.rabbit_t00".into())]);
        assert_eq!(a.parts[0].slot, SLOT_FACE);
        assert_eq!(NPC_FALLBACK, NpcFallback { draw_scale: 10.0, collision_height: 100.0, collision_radius: 50.0 });
    }

    #[test]
    fn match_ref_bone_resolves_aliases_first_like_the_engine() {
        let mut mesh = CharacterMesh::empty();
        mesh.bones = vec![bone("Bip01", 0, [0.0; 4], [0.0; 3]), bone("Weapon_R_Bone", 0, [0.0; 4], [0.0; 3])];
        mesh.aliases = vec![Alias {
            name: "e_Hand".into(),
            bone: "Weapon_R_Bone".into(),
            origin: [0.8, 2.3, 0.0],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }];
        assert_eq!(mesh.match_ref_bone("weapon_r_bone"), Some(1));
        assert_eq!(mesh.match_ref_bone("E_HAND"), Some(1));
        assert_eq!(mesh.match_ref_bone("Bip01 Pelvis"), None);
        assert_eq!(mesh.match_ref_bone(""), None);
        assert_eq!(mesh.alias("e_hand").map(|a| a.origin), Some([0.8, 2.3, 0.0]));
    }
}
