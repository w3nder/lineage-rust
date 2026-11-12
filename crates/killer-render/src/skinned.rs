use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Mat4};
use killer_assets::characters::{CharacterMesh, CharacterSection};
use killer_assets::scene::lights::{MAX_HARDWARE_LIGHTS, PointLight};
use ue2_assets::SkinVertex;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, PartialEq)]
pub struct SkinnedVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub bones: [u32; 4],
    pub weights: [f32; 4],
}

impl From<&SkinVertex> for SkinnedVertex {
    fn from(v: &SkinVertex) -> Self {
        Self { position: v.position, normal: v.normal, uv: v.uv, bones: v.bones.map(u32::from), weights: v.weights }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CharacterId(pub usize);

#[derive(Clone, Debug, PartialEq)]
pub struct CharacterInstance {
    pub actor_id: i32,
    pub actor_position: glam::Vec3,
    pub collision_height: f32,
    pub shadow_caster: bool,
    pub light_sphere: [f32; 4],
    pub mesh: CharacterId,
    pub world: Mat4,
    pub palette: Vec<Mat4>,
    pub zone: u8,
    pub lights: Vec<PointLight>,
    pub pawn_lights: Vec<PointLight>,
    pub ambient: [u8; 3],
    pub sun_affect: bool,
    pub overlay: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, PartialEq)]
pub struct ObjectUniform {
    pub world: [[f32; 4]; 4],
    pub normal: [[f32; 4]; 3],
    pub palette_offset: u32,
    pub bone_count: u32,
    pub zone: u32,
    pub sun_affect: u32,
    pub light_count: u32,
    pub sun_visible: u32,
    pub _pad: [u32; 2],
    pub light_position: [[f32; 4]; MAX_HARDWARE_LIGHTS],
    pub light_color: [[f32; 4]; MAX_HARDWARE_LIGHTS],
    pub light_direction: [[f32; 4]; MAX_HARDWARE_LIGHTS],
    pub light_attenuation: [[f32; 4]; MAX_HARDWARE_LIGHTS],
    pub ambient: [f32; 4],
    pub overlay: [f32; 4],
}

impl ObjectUniform {
    pub fn new(world: Mat4, palette_offset: u32, bone_count: u32, zone: u32, sun_affect: bool, lights: &[PointLight]) -> Self {
        let normal = Mat3::from_mat4(world).inverse().transpose();
        let mut light_position = [[0.0; 4]; MAX_HARDWARE_LIGHTS];
        let mut light_color = [[0.0; 4]; MAX_HARDWARE_LIGHTS];
        let mut light_direction = [[0.0; 4]; MAX_HARDWARE_LIGHTS];
        for (slot, light) in lights.iter().take(MAX_HARDWARE_LIGHTS).enumerate() {
            light_position[slot] = light.position.extend(light.radius).to_array();
            light_color[slot] = [light.color[0], light.color[1], light.color[2], 0.0];
            light_direction[slot] = light.direction.extend(f32::from(matches!(light.effect, 8 | 12))).to_array();
        }
        Self {
            world: world.to_cols_array_2d(),
            normal: [normal.x_axis.extend(0.0).to_array(), normal.y_axis.extend(0.0).to_array(), normal.z_axis.extend(0.0).to_array()],
            palette_offset,
            bone_count,
            zone,
            sun_affect: u32::from(sun_affect),
            light_count: lights.len().min(MAX_HARDWARE_LIGHTS) as u32,
            sun_visible: 1,
            _pad: [0; 2],
            light_position,
            light_color,
            light_direction,
            light_attenuation: [[0.0; 4]; MAX_HARDWARE_LIGHTS],
            ambient: [0.0; 4],
            overlay: [0.0; 4],
        }
    }

    pub fn set_light_sphere(&mut self, sphere: [f32; 4], lights: &[PointLight]) {
        let center = glam::Vec3::new(sphere[0], sphere[1], sphere[2]);
        for (slot, light) in lights.iter().take(MAX_HARDWARE_LIGHTS).enumerate() {
            let [a0, a1] = killer_assets::scene::lights::hardware_attenuation(light, center, sphere[3]);
            self.light_attenuation[slot] = [a0, a1, 0.0, 0.0];
        }
    }
}

pub fn aligned_stride(size: u64, alignment: u64) -> u64 {
    size.div_ceil(alignment.max(1)) * alignment.max(1)
}

pub fn valid_sections(mesh: &CharacterMesh) -> impl Iterator<Item = &CharacterSection> {
    let total = mesh.indices.len() as u64;
    mesh.sections.iter().filter(move |s| s.index_count > 0 && u64::from(s.first_index) + u64::from(s.index_count) <= total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::Camera;
    use crate::environment::{Environment, TIMEENV_FIXTURE};
    use crate::headless::{CLEAR_RGBA, Headless};
    use crate::textures::TextureSource;
    use glam::{Vec3, vec3};
    use killer_assets::scene::{Pass, Scene, ZoneLighting, texture_material};
    use killer_assets::textures::{TextureData, TextureFormatKind, checker_image};
    use killer_assets::timeenv::TimeEnv;
    use std::sync::Arc;
    use std::time::Instant;
    use ue2_assets::Bone;

    struct Checker;

    impl TextureSource for Checker {
        fn texture(&mut self, _: &str) -> Arc<TextureData> {
            static DATA: std::sync::OnceLock<Arc<TextureData>> = std::sync::OnceLock::new();
            DATA.get_or_init(|| Arc::new(TextureData { width: 8, height: 8, format: TextureFormatKind::Rgba8, mips: vec![checker_image().rgba] })).clone()
        }
    }

    fn noon() -> Environment {
        Environment::from_time(&TimeEnv::parse(TIMEENV_FIXTURE), 12.0)
    }

    fn camera() -> Camera {
        Camera { eye: vec3(0.0, -300.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 1.0, far: 5000.0 }
    }

    fn bone(name: &str) -> Bone {
        Bone { name: name.into(), parent: 0, rotation: [0.0, 0.0, 0.0, 1.0], position: [0.0; 3] }
    }

    fn vertex(x: f32, z: f32, bone: u16) -> SkinVertex {
        SkinVertex { position: [x, 0.0, z], normal: [0.0, -1.0, 0.0], uv: [0.0, 0.0], bones: [bone, 0, 0, 0], weights: [1.0, 0.0, 0.0, 0.0] }
    }

    fn section(first_index: u32, index_count: u32) -> CharacterSection {
        CharacterSection { first_index, index_count, material: texture_material(""), pass: Pass::Opaque }
    }

    fn two_quads_two_bones() -> CharacterMesh {
        let quad = |x0: f32, x1: f32, bone: u16| vec![vertex(x0, -50.0, bone), vertex(x1, -50.0, bone), vertex(x1, 50.0, bone), vertex(x0, 50.0, bone)];
        let mut vertices = quad(20.0, 60.0, 0);
        vertices.extend(quad(-60.0, -20.0, 1));
        CharacterMesh {
            bones: vec![bone("Bip01"), bone("Bip01_Spine")],
            vertices,
            indices: vec![0, 2, 1, 0, 3, 2, 4, 6, 5, 4, 7, 6],
            sections: vec![section(0, 12)],
            bind: vec![Mat4::IDENTITY; 2],
            inverse_bind: vec![Mat4::IDENTITY; 2],
            ..CharacterMesh::empty()
        }
    }

    fn covered(rgba: &[u8], x: usize, y: usize) -> bool {
        rgba[(y * 64 + x) * 4..][..4] != CLEAR_RGBA
    }

    fn instance(id: CharacterId, x: f32, palette: Vec<Mat4>) -> CharacterInstance {
        CharacterInstance { overlay: [0.0; 4], actor_id: 1, actor_position: Vec3::new(x, 0.0, 0.0), collision_height: 22.0, shadow_caster: true, light_sphere: [x, 0.0, 0.0, 78.1025], mesh: id, world: Mat4::from_translation(Vec3::new(x, 0.0, 0.0)), palette, zone: 0, ambient: [0; 3], sun_affect: true, lights: Vec::new(), pawn_lights: Vec::new() }
    }

    const LIFT: f32 = 1000.0;

    #[test]
    fn two_bone_quads_move_with_the_palette() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let id = headless.renderer_mut().upload_character(&two_quads_two_bones(), &mut Checker);
        let env = noon();
        headless.renderer_mut().set_instances(&[instance(id, 0.0, vec![Mat4::IDENTITY; 2])]);
        let a = headless.render(&camera(), &env).unwrap();
        assert!(covered(&a, 24, 32) && covered(&a, 40, 32) && !covered(&a, 32, 32));
        let lifted = vec![Mat4::IDENTITY, Mat4::from_translation(Vec3::new(0.0, 0.0, LIFT))];
        headless.renderer_mut().set_instances(&[instance(id, 0.0, lifted)]);
        let b = headless.render(&camera(), &env).unwrap();
        assert_ne!(a, b);
        assert!(covered(&b, 24, 32) && !covered(&b, 40, 32));
    }

    fn blended_sheet(second_pass_tests_depth: bool) -> CharacterMesh {
        let cells = 24usize;
        let mut vertices = Vec::new();
        for row in 0..=cells {
            for column in 0..=cells {
                let x = -120.0 + 240.0 * column as f32 / cells as f32;
                let z = -120.0 + 240.0 * row as f32 / cells as f32;
                let weight = row as f32 / cells as f32;
                vertices.push(SkinVertex { position: [x, 0.0, z], normal: [0.0, -1.0, 0.0], uv: [column as f32 / cells as f32, row as f32 / cells as f32], bones: [0, 1, 0, 0], weights: [1.0 - weight, weight, 0.0, 0.0] });
            }
        }
        let mut indices = Vec::new();
        for row in 0..cells {
            for column in 0..cells {
                let a = (row * (cells + 1) + column) as u32;
                let b = a + 1;
                let c = a + (cells + 1) as u32;
                let d = c + 1;
                indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }
        let mut material = texture_material("");
        let mut shine = material.passes[0].clone();
        shine.src = l2_catalog::BlendFactor::SrcAlpha;
        shine.dst = l2_catalog::BlendFactor::One;
        shine.alpha_blend = true;
        shine.z_write = false;
        shine.z_test = second_pass_tests_depth;
        material.passes.push(shine);
        let count = indices.len() as u32;
        CharacterMesh {
            bones: vec![bone("Bip01"), bone("Bip01_Spine")],
            vertices,
            indices,
            sections: vec![CharacterSection { first_index: 0, index_count: count, material, pass: Pass::Opaque }],
            bind: vec![Mat4::IDENTITY; 2],
            inverse_bind: vec![Mat4::IDENTITY; 2],
            ..CharacterMesh::empty()
        }
    }

    fn render_sheet(second_pass_tests_depth: bool, frame: u32) -> Option<Vec<u8>> {
        let mut headless = Headless::new(160, 160).ok()?;
        let id = headless.renderer_mut().upload_character(&blended_sheet(second_pass_tests_depth), &mut Checker);
        let anchor = vec3(98000.0, 146000.0, -3300.0);
        let tilt = Mat4::from_rotation_x(0.31 + frame as f32 * 0.0007) * Mat4::from_rotation_z(0.2);
        let palette = vec![Mat4::IDENTITY, Mat4::from_translation(Vec3::new(3.0, 0.0, 0.0)) * Mat4::from_rotation_y(0.05 + frame as f32 * 0.001)];
        let mut sheet = instance(id, 0.0, palette);
        sheet.world = Mat4::from_translation(anchor) * tilt;
        headless.renderer_mut().set_instances(&[sheet]);
        let camera = Camera { eye: anchor + vec3(0.0, -700.0, 40.0), target: anchor, fov_x_deg: 60.0, near: 10.0, far: 60000.0 };
        headless.render(&camera, &noon()).ok()
    }

    #[test]
    fn a_second_pass_over_the_same_skinned_surface_is_not_cut_by_the_depth_test() {
        let mut cut = 0;
        for frame in 0..8 {
            let (Some(tested), Some(untested)) = (render_sheet(true, frame), render_sheet(false, frame)) else { return };
            cut += tested.chunks(4).zip(untested.chunks(4)).filter(|(a, b)| a != b).count();
        }
        assert_eq!(cut, 0, "{cut} pixels of the second pass lost the depth test against the first pass");
    }

    #[test]
    fn pawn_shadow_capture_uses_one_native_target_when_in_range() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let id = headless.renderer_mut().upload_character(&two_quads_two_bones(), &mut Checker);
        headless.renderer_mut().set_viewer(crate::renderer::Viewer { zone: 0, position: Vec3::new(0.0, -300.0, 0.0), dt: 0.0 });
        headless.renderer_mut().set_instances(&[instance(id, 0.0, vec![Mat4::IDENTITY; 2])]);
        headless.render(&camera(), &noon()).unwrap();
        assert_eq!(headless.renderer_mut().shadow_count(), 1);
    }

    #[test]
    fn instances_share_one_mesh_at_two_places() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let id = headless.renderer_mut().upload_character(&two_quads_two_bones(), &mut Checker);
        let env = noon();
        let at = |x: f32| instance(id, x, vec![Mat4::IDENTITY; 2]);
        headless.renderer_mut().set_instances(&[at(-80.0), at(80.0)]);
        let two = headless.render(&camera(), &env).unwrap();
        assert!(covered(&two, 10, 32) && covered(&two, 24, 32) && covered(&two, 40, 32) && covered(&two, 54, 32));
        assert!(!covered(&two, 32, 32) && !covered(&two, 17, 32) && !covered(&two, 47, 32));
        headless.renderer_mut().set_instances(&[]);
        let none = headless.render(&camera(), &env).unwrap();
        assert!(!covered(&none, 24, 32) && !covered(&none, 40, 32));
    }

    #[test]
    fn set_instances_between_frames_only_keeps_the_last_call() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let id = headless.renderer_mut().upload_character(&two_quads_two_bones(), &mut Checker);
        let at = |x: f32| instance(id, x, vec![Mat4::IDENTITY; 2]);
        headless.renderer_mut().set_instances(&[at(-80.0)]);
        let left = headless.render(&camera(), &noon()).unwrap();
        headless.renderer_mut().set_instances(&[at(80.0)]);
        let right = headless.render(&camera(), &noon()).unwrap();
        assert_ne!(left, right);
        for i in 0..2000 {
            headless.renderer_mut().set_instances(&[at(if i % 2 == 0 { 80.0 } else { -80.0 })]);
        }
        assert_eq!(headless.renderer_mut().instance_count(), 1);
        assert_eq!(headless.render(&camera(), &noon()).unwrap(), left);
    }

    #[test]
    fn palettes_of_different_sizes_are_offset_per_instance() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let id = headless.renderer_mut().upload_character(&two_quads_two_bones(), &mut Checker);
        let env = noon();
        let wide = instance(id, 80.0, vec![Mat4::IDENTITY; 5]);
        let lifted = instance(id, -80.0, vec![Mat4::IDENTITY, Mat4::from_translation(Vec3::new(0.0, 0.0, LIFT))]);
        headless.renderer_mut().set_instances(&[wide, lifted]);
        let rgba = headless.render(&camera(), &env).unwrap();
        assert!(covered(&rgba, 10, 32) && covered(&rgba, 24, 32) && covered(&rgba, 40, 32) && !covered(&rgba, 54, 32));
    }

    #[test]
    fn character_textures_remain_shared_until_the_last_user_is_removed() {
        let mut headless = Headless::new(64, 64).unwrap();
        let mut mesh = two_quads_two_bones();
        mesh.sections[0].material = texture_material("shared.character");
        let first = headless.renderer_mut().upload_character(&mesh, &mut Checker);
        let textures = headless.renderer_mut().texture_count();
        assert!(textures > 0);
        let second = headless.renderer_mut().upload_character(&mesh, &mut Checker);
        assert_eq!(headless.renderer_mut().texture_count(), textures);
        headless.renderer_mut().set_instances(&[instance(second, 0.0, vec![Mat4::IDENTITY; 2])]);
        let expected = headless.render(&camera(), &noon()).unwrap();
        headless.renderer_mut().remove_character(first);
        assert_eq!(headless.renderer_mut().texture_count(), textures);
        assert_eq!(headless.render(&camera(), &noon()).unwrap(), expected);
        headless.renderer_mut().remove_character(second);
        assert_eq!(headless.renderer_mut().texture_count(), 0);
        let reloaded = headless.renderer_mut().upload_character(&mesh, &mut Checker);
        headless.renderer_mut().set_instances(&[instance(reloaded, 0.0, vec![Mat4::IDENTITY; 2])]);
        assert_eq!(headless.render(&camera(), &noon()).unwrap(), expected);
        assert_eq!(headless.renderer_mut().texture_count(), textures);
    }

    #[test]
    fn removed_characters_and_empty_palettes_are_not_drawn() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let id = headless.renderer_mut().upload_character(&two_quads_two_bones(), &mut Checker);
        let env = noon();
        headless.renderer_mut().set_instances(&[instance(id, 0.0, vec![])]);
        let empty = headless.render(&camera(), &env).unwrap();
        assert!(!covered(&empty, 24, 32) && !covered(&empty, 40, 32));
        headless.renderer_mut().remove_character(id);
        headless.renderer_mut().set_instances(&[instance(id, 0.0, vec![Mat4::IDENTITY; 2])]);
        let gone = headless.render(&camera(), &env).unwrap();
        assert!(!covered(&gone, 24, 32) && !covered(&gone, 40, 32));
        let again = headless.renderer_mut().upload_character(&two_quads_two_bones(), &mut Checker);
        assert_eq!(again, id);
    }

    #[test]
    fn overlay_modulates_the_lit_color_and_keeps_the_rest() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let id = headless.renderer_mut().upload_character(&two_quads_two_bones(), &mut Checker);
        headless.renderer_mut().set_instances(&[instance(id, 0.0, vec![Mat4::IDENTITY; 2])]);
        let plain = headless.render(&camera(), &noon()).unwrap();
        let tinted = CharacterInstance { overlay: [0.0, 1.0, 0.0, 1.0], ..instance(id, 0.0, vec![Mat4::IDENTITY; 2]) };
        headless.renderer_mut().set_instances(&[tinted]);
        let rgba = headless.render(&camera(), &noon()).unwrap();
        let pixel = |image: &[u8]| image[(32 * 64 + 24) * 4..][..4].to_vec();
        assert_ne!(pixel(&plain), pixel(&rgba));
        assert_eq!(pixel(&rgba)[0], 0);
        assert_eq!(pixel(&rgba)[2], 0);
        assert!(pixel(&rgba)[1] <= pixel(&plain)[1]);
    }

    #[test]
    fn sections_past_the_index_buffer_are_skipped() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut mesh = two_quads_two_bones();
        mesh.sections = vec![section(0, 6), section(6, 12)];
        let id = headless.renderer_mut().upload_character(&mesh, &mut Checker);
        headless.renderer_mut().set_instances(&[instance(id, 0.0, vec![Mat4::IDENTITY; 2])]);
        let rgba = headless.render(&camera(), &noon()).unwrap();
        assert!(covered(&rgba, 24, 32) && !covered(&rgba, 40, 32));
    }

    #[test]
    fn a_crowd_of_instances_renders_and_set_instances_is_cheap() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut mesh = two_quads_two_bones();
        mesh.bones = (0..70).map(|i| bone(&format!("b{i}"))).collect();
        let id = headless.renderer_mut().upload_character(&mesh, &mut Checker);
        let crowd: Vec<CharacterInstance> = (0..100).map(|i| instance(id, i as f32 * 4.0 - 200.0, vec![Mat4::IDENTITY; 70])).collect();
        headless.renderer_mut().set_instances(&crowd);
        let start = Instant::now();
        for _ in 0..10 {
            headless.renderer_mut().set_instances(&crowd);
        }
        let per_call = start.elapsed() / 10;
        eprintln!("set_instances(100 x 70): {per_call:?}");
        let rgba = headless.render(&camera(), &noon()).unwrap();
        assert!(covered(&rgba, 32, 32));
        assert!(per_call.as_millis() < 20, "{per_call:?}");
    }

    #[test]
    fn object_uniform_is_256_byte_aligned_and_carries_the_inverse_transpose() {
        assert_eq!(size_of::<ObjectUniform>(), 688);
        assert_eq!(aligned_stride(400, 256), 512);
        assert_eq!(aligned_stride(256, 256), 256);
        assert_eq!(aligned_stride(300, 256), 512);
        let world = Mat4::from_scale(Vec3::new(2.0, 1.0, 1.0));
        let lamp = PointLight { position: Vec3::new(1.0, 2.0, 3.0), radius: 500.0, color: [0.3, 0.2, 0.1], brightness: 64.0, ..Default::default() };
        let uniform = ObjectUniform::new(world, 7, 70, 3, true, &[lamp; 9]);
        assert_eq!(uniform.normal[0], [0.5, 0.0, 0.0, 0.0]);
        assert_eq!(uniform.normal[1], [0.0, 1.0, 0.0, 0.0]);
        assert_eq!((uniform.palette_offset, uniform.bone_count, uniform.zone, uniform.sun_affect), (7, 70, 3, 1));
        assert_eq!(uniform.light_count, 8);
        assert_eq!(uniform.light_position[7], [1.0, 2.0, 3.0, 500.0]);
        assert_eq!(uniform.light_color[0], [0.3, 0.2, 0.1, 0.0]);
        assert_eq!(ObjectUniform::new(world, 0, 1, 0, false, &[]).light_count, 0);
    }

    #[test]
    fn a_point_light_lights_the_side_of_a_character_facing_it() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut mesh = two_quads_two_bones();
        for vertex in &mut mesh.vertices {
            vertex.normal = [0.0, 0.0, -1.0];
        }
        let id = headless.renderer_mut().upload_character(&mesh, &mut Checker);
        let env = noon();
        let zones = vec![ZoneLighting { sun_affect: false, ..ZoneLighting::default() }];
        headless.upload(&Scene { zones, ..Default::default() }, &mut Checker);
        let dark = |lights: Vec<PointLight>| CharacterInstance { overlay: [0.0; 4], lights, sun_affect: false, ..instance(id, 0.0, vec![Mat4::IDENTITY; 2]) };
        headless.renderer_mut().set_instances(&[dark(Vec::new())]);
        let unlit = headless.render(&camera(), &env).unwrap();
        let lamp = PointLight { position: Vec3::new(40.0, 0.0, -60.0), radius: 150.0, color: [1.0, 1.0, 1.0], brightness: 64.0, ..Default::default() };
        headless.renderer_mut().set_instances(&[dark(vec![lamp])]);
        let lit = headless.render(&camera(), &env).unwrap();
        assert!(covered(&lit, 24, 32));
        let brightness = |rgba: &[u8], x: usize| rgba[(32 * 64 + x) * 4] as i32 + rgba[(32 * 64 + x) * 4 + 1] as i32 + rgba[(32 * 64 + x) * 4 + 2] as i32;
        assert!(brightness(&lit, 24) > brightness(&unlit, 24), "{} vs {}", brightness(&lit, 24), brightness(&unlit, 24));
        assert!(brightness(&lit, 24) > brightness(&lit, 40), "{} vs {}", brightness(&lit, 24), brightness(&lit, 40));
    }

    #[test]
    fn pawn_lights_take_the_hardware_slots_after_the_level_lights_1069b051() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut mesh = two_quads_two_bones();
        for vertex in &mut mesh.vertices {
            vertex.normal = [0.0, 0.0, -1.0];
        }
        let id = headless.renderer_mut().upload_character(&mesh, &mut Checker);
        let env = noon();
        let zones = vec![ZoneLighting { sun_affect: false, ..ZoneLighting::default() }];
        headless.upload(&Scene { zones, ..Default::default() }, &mut Checker);
        let black = PointLight { position: Vec3::new(40.0, 0.0, -60.0), radius: 150.0, color: [0.0; 3], ..Default::default() };
        let lamp = PointLight { position: Vec3::new(40.0, 0.0, -60.0), radius: 150.0, color: [1.0, 1.0, 1.0], effect: 0x11, ..Default::default() };
        let with = |pawn_lights: Vec<PointLight>| CharacterInstance { overlay: [0.0; 4], lights: vec![black; 4], pawn_lights, sun_affect: false, ..instance(id, 0.0, vec![Mat4::IDENTITY; 2]) };
        headless.renderer_mut().set_instances(&[with(Vec::new())]);
        let unlit = headless.render(&camera(), &env).unwrap();
        headless.renderer_mut().set_instances(&[with(vec![lamp])]);
        let lit = headless.render(&camera(), &env).unwrap();
        headless.renderer_mut().set_instances(&[CharacterInstance { lights: vec![black; 8], ..with(vec![lamp]) }]);
        let level_only = headless.render(&camera(), &env).unwrap();
        let brightness = |rgba: &[u8], x: usize| rgba[(32 * 64 + x) * 4] as i32 + rgba[(32 * 64 + x) * 4 + 1] as i32 + rgba[(32 * 64 + x) * 4 + 2] as i32;
        assert!(brightness(&lit, 24) > brightness(&unlit, 24), "{} vs {}", brightness(&lit, 24), brightness(&unlit, 24));
        assert_eq!(level_only.len(), lit.len());
        assert!(brightness(&level_only, 24) > brightness(&unlit, 24), "MaxLights corta as do nível em 4, as do pawn ocupam o resto até 8");
    }

    #[test]
    fn occluding_the_sun_keeps_the_actor_ambient() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut mesh = two_quads_two_bones();
        for vertex in &mut mesh.vertices { vertex.normal = [0.0, 0.0, 1.0]; }
        let id = headless.renderer_mut().upload_character(&mesh, &mut Checker);
        let instances = [instance(id, 0.0, vec![Mat4::IDENTITY; 2])];
        let env = noon();
        headless.renderer_mut().set_time(1.0);
        headless.renderer_mut().update_sun_visibility(Vec3::ZERO, env.sun_dir, &instances, |_, _| false);
        headless.renderer_mut().set_instances(&instances);
        let occluded = headless.render(&camera(), &env).unwrap();
        let mut ambient = noon();
        ambient.sun_brightness = 0.0;
        let reference = headless.render(&camera(), &ambient).unwrap();
        assert_eq!(occluded, reference);
        assert!(occluded[(32 * 64 + 24) * 4..][..3].iter().any(|c| *c > 0));
    }

    #[test]
    fn skeletal_render_1069aff9_uses_full_zone_ambient_before_modulate2x() {
        struct White;
        impl TextureSource for White {
            fn texture(&mut self, _: &str) -> Arc<TextureData> {
                Arc::new(TextureData { width: 1, height: 1, format: TextureFormatKind::Rgba8, mips: vec![vec![255; 4]] })
            }
        }
        let mut headless = Headless::new(64, 64).unwrap();
        let id = headless.renderer_mut().upload_character(&two_quads_two_bones(), &mut White);
        let mut pawn = instance(id, 0.0, vec![Mat4::IDENTITY; 2]);
        pawn.sun_affect = false;
        pawn.ambient = [70, 50, 30];
        headless.renderer_mut().set_instances(&[pawn.clone()]);
        let rgba = headless.render(&camera(), &noon()).unwrap();
        let rgb = &rgba[(32 * 64 + 24) * 4..][..3];
        assert_eq!(rgb, [140, 100, 60]);
        pawn.ambient = [0; 3];
        headless.renderer_mut().set_instances(&[pawn]);
        let rgba = headless.render(&camera(), &noon()).unwrap();
        assert_eq!(&rgba[(32 * 64 + 24) * 4..][..3], [0; 3]);
    }

    #[test]
    fn skinned_vertex_widens_bone_indices() {
        let v = SkinVertex { position: [1.0; 3], normal: [0.0, 0.0, 1.0], uv: [0.5; 2], bones: [3, 700, 0, 0], weights: [0.5, 0.5, 0.0, 0.0] };
        let s = SkinnedVertex::from(&v);
        assert_eq!(s.bones, [3, 700, 0, 0]);
        assert_eq!(size_of::<SkinnedVertex>(), 64);
    }

    #[test]
    fn valid_sections_drop_ranges_past_the_indices() {
        let mut mesh = two_quads_two_bones();
        mesh.sections = vec![section(0, 6), section(6, 6), section(6, 12), section(12, 0)];
        let kept: Vec<u32> = valid_sections(&mesh).map(|s| s.first_index).collect();
        assert_eq!(kept, [0, 6]);
    }
}
