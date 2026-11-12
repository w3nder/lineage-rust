use super::lights::{PointLight, STATIC_MESH_POINT_SCALE, fitted_point_term, point_lights};
use super::{ActorCull, DrawKind, LodSelect, MeshDraw, Vertex, draw_pass, resolve_material, transform::placement_matrix};
use crate::client::Client;
use crate::options::{ClippingRange, range_units};
use glam::Vec3;
use std::collections::HashMap;
use ue2_level::Level;

pub fn vertex_color(color: Option<[u8; 4]>) -> [f32; 4] {
    match color {
        Some([r, g, b, a]) => [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0],
        None => [0.0, 0.0, 0.0, 1.0],
    }
}

pub fn hardware_light(position: Vec3, normal: Vec3, lights: &[PointLight], sphere: (Vec3, f32)) -> [f32; 4] {
    let [r, g, b] = fitted_point_term(position, normal, lights, STATIC_MESH_POINT_SCALE, sphere);
    [r, g, b, 1.0]
}

pub fn section_material(skins: &[Option<String>], materials: &[Option<String>], index: usize) -> Option<String> {
    skins.get(index).and_then(|skin| skin.clone()).or_else(|| materials.get(index).cloned().flatten())
}

/// Which mesh of a `bStaticMeshLod` actor draws at 2D distance `d` from the pawn (`RenderStaticMesh`).
pub fn lod_for(distance: f32, range01: f32, range02: f32) -> u8 {
    if distance <= range01 {
        0
    } else if distance <= range02 {
        1
    } else {
        2
    }
}

/// `bTimeReactor`: shown while `ShowTime <= hour < HideTime`, both cyclic over 24 h.
pub fn time_reactor_shows(show: f32, hide: f32, hour: f32) -> bool {
    let mut hide = hide;
    while hide < show {
        hide += HOURS_PER_DAY;
    }
    let mut hour = hour.rem_euclid(HOURS_PER_DAY);
    while hour < show {
        hour += HOURS_PER_DAY;
    }
    hide >= hour
}

struct LoadedMeshes {
    meshes: HashMap<String, Option<l2_catalog::Mesh>>,
    materials: HashMap<String, l2_catalog::Material>,
}

impl LoadedMeshes {
    fn mesh(&mut self, client: &mut Client, path: &str) -> Option<&l2_catalog::Mesh> {
        let key = path.to_lowercase();
        if !self.meshes.contains_key(&key) {
            let loaded = client.catalog().static_mesh(path);
            self.meshes.insert(key.clone(), loaded);
        }
        self.meshes.get(&key).and_then(|m| m.as_ref())
    }

    fn material(&mut self, client: &mut Client, path: &str) -> l2_catalog::Material {
        let key = path.to_lowercase();
        if !self.materials.contains_key(&key) {
            let loaded = resolve_material(client, path);
            self.materials.insert(key.clone(), loaded);
        }
        self.materials[&key].clone()
    }
}

#[expect(clippy::too_many_arguments, reason = "one actor's mesh at one LOD level with the lights and range it is drawn under")]
fn mesh_draws(client: &mut Client, loaded: &mut LoadedMeshes, actor: &ue2_level::Actor, mesh_path: &str, level: u8, lights: &[PointLight], zone: u8, cull: ActorCull, sun_affect: bool, sphere: (Vec3, f32)) -> Vec<MeshDraw> {
    let Some(mesh) = loaded.mesh(client, mesh_path).cloned() else { return Vec::new() };
    let world = placement_matrix(actor.placement.location, actor.placement.rotation, actor.scale);
    let normal_matrix = glam::Mat3::from_mat4(world).inverse().transpose();
    let mirrored = world.determinant() < 0.0;
    let mut draws = Vec::new();
    for (section_index, section) in mesh.mesh.sections.iter().enumerate() {
        let material_path = section_material(&actor.skins, &mesh.materials, section_index).unwrap_or_default();
        let material = loaded.material(client, &material_path);
        let first = section.first_index as usize;
        let count = section.index_count() as usize;
        let Some(section_indices) = mesh.mesh.indices.get(first..first + count) else { continue };
        let mut remap: HashMap<u16, u32> = HashMap::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::with_capacity(count);
        for &index in section_indices {
            let next = vertices.len() as u32;
            let local = *remap.entry(index).or_insert_with(|| {
                let i = index as usize;
                let position = world.transform_point3(Vec3::from(mesh.mesh.positions[i])).to_array();
                let normal = (normal_matrix * Vec3::from(mesh.mesh.normals.get(i).copied().unwrap_or([0.0, 0.0, 1.0]))).normalize_or(Vec3::Z).to_array();
                let uv = mesh.mesh.uvs.get(i).copied().unwrap_or([0.0, 0.0]);
                let mask_uv = mesh.mesh.uvs2.get(i).copied().unwrap_or(uv);
                let color = if level == LOD_UNLIT || actor.unlit {
                    vertex_color(None)
                } else if level == 0 && !actor.lighting.is_empty() {
                    vertex_color(actor.lighting.get(i).copied())
                } else {
                    hardware_light(Vec3::from(position), Vec3::from(normal), lights, sphere)
                };
                vertices.push(Vertex { position, normal, uv, mask_uv, color });
                next
            });
            indices.push(local);
        }
        if mirrored {
            mirror_winding(&mut indices);
        }
        let pass = draw_pass(&material);
        draws.push(MeshDraw { instances: Vec::new(), vertices, indices, material, mask: None, kind: DrawKind::StaticMesh, sun_affect, pass, zone, lightmap: None, modulate_2x: !actor.unlit, sector: None, fade: None, tint: None, actor: Some(cull) });
    }
    draws
}

/// A mirrored placement (`det(LocalToWorld) < 0`) makes the engine flip the cull mode for the actor; with world-space vertices that is a reversed winding.
pub fn mirror_winding(indices: &mut [u32]) {
    for tri in indices.as_chunks_mut::<3>().0 {
        tri.swap(1, 2);
    }
}

/// `lod == 2` switches every hardware light off: the mesh keeps only the ambient.
const LOD_UNLIT: u8 = 2;
const HOURS_PER_DAY: f32 = 24.0;

pub fn build(client: &mut Client, level: &Level, clipping: &ClippingRange) -> Vec<MeshDraw> {
    Builder::new(level).build(client, level, clipping, level.actors.iter().filter(|a| a.l2_movement.is_none() && a.mover.is_none()))
}

pub struct SpawnedMesh {
    pub mesh: String,
    pub placement: ue2_level::Placement,
    pub actor_id: u32,
}

pub fn spawnable_static_mesh_actor(mesh: &str, placement: ue2_level::Placement) -> ue2_level::Actor {
    ue2_level::Actor {
        class: "SpawnableStaticMeshActor".into(),
        l2_events: Vec::new(),
        name: String::new(),
        tag: None,
        groups: Vec::new(),
        placement,
        scale: [1.0; 3],
        draw_scale: 1.0,
        draw_scale_3d: [1.0; 3],
        static_mesh: Some(mesh.into()),
        skins: Vec::new(),
        unlit: false,
        sun_affect: false,
        special_lit: false,
        max_lights: 4,
        ambient_glow: 0,
        owner: None,
        use_owner_ambient: false,
        lighting: Vec::new(),
        movement: None,
        l2_movement: None,
        movement_tags: Vec::new(),
        movement_event: None,
        mover: None,
        server_id: None,
        hidden: false,
        is_static: false,
        ignored_range: false,
        time_reactor: None,
    }
}

pub fn build_spawned(client: &mut Client, level: &Level, clipping: &ClippingRange, spawned: &[SpawnedMesh]) -> Vec<Vec<MeshDraw>> {
    let mut builder = Builder::new(level);
    spawned
        .iter()
        .map(|piece| {
            builder.next_actor = piece.actor_id;
            let actor = spawnable_static_mesh_actor(&piece.mesh, piece.placement);
            builder.build(client, level, clipping, std::iter::once(&actor))
        })
        .collect()
}

pub(super) struct Builder {
    next_actor: u32,
    lights: Vec<PointLight>,
    light_map: super::actor_lighting::LightMap,
    loaded: LoadedMeshes,
}

impl Builder {
    pub(super) fn new(level: &Level) -> Self {
        let lights = point_lights(&level.lights);
        let light_map = super::actor_lighting::LightMap::new(level, &lights);
        Self { next_actor: 0, lights, light_map, loaded: LoadedMeshes { meshes: HashMap::new(), materials: HashMap::new() } }
    }

pub(super) fn build<'a>(&mut self, client: &mut Client, level: &Level, clipping: &ClippingRange, actors: impl Iterator<Item = &'a ue2_level::Actor>) -> Vec<MeshDraw> {
    let lights = &self.lights;
    let light_map = &self.light_map;
    let loaded = &mut self.loaded;
    let next_actor = &mut self.next_actor;
    let mut draws = Vec::new();
    for actor in actors {
        let Some(path) = actor.static_mesh.as_deref() else { continue };
        if actor.hidden {
            continue;
        }
        let Some(mesh) = loaded.mesh(client, path).cloned() else { continue };
        let zone = level.bsp_tree.zone_at(actor.placement.location);
        let world = placement_matrix(actor.placement.location, actor.placement.rotation, actor.scale);
        let (center, radius) = super::transform::sphere(mesh.mesh.bounding_sphere, world);
        let bounds = super::actor_lighting::Bounds::transformed(mesh.mesh.bounding_box, world);
        let candidates = super::lights::box_leaf_candidates(level, lights, bounds.center(), bounds.extent());
        let sunlight = level.zones.get(usize::from(zone)).and_then(|zone| zone.actor.as_ref()).and_then(|name| level.zone_infos.get(name)).is_none_or(|info| info.sun_affect);
        let selected = super::lights::select(&candidates, center, radius, actor.special_lit, usize::from(actor.max_lights), sunlight);
        let lights = selected;
        let xy = [actor.placement.location[0], actor.placement.location[1]];
        let range = range_units(if mesh.mesh.lod.is_some() { clipping.static_mesh_lod } else { clipping.static_mesh });
        let (vector, sun_affect) = light_map.zone_ambient(bounds);
        let ambient = super::actor_lighting::Ambient { vector: vector.to_array(), glow: super::actor_lighting::actor_glow(level, actor) };
        let id = *next_actor;
        *next_actor += 1;
        let base = ActorCull { id, ambient, xy, range, ignored_range: actor.ignored_range, time_reactor: actor.time_reactor, lod: None };
        match mesh.mesh.lod {
            None => draws.extend(mesh_draws(client, loaded, actor, path, 0, &lights, zone, base, sun_affect, (center, radius))),
            Some(lod) => {
                let ranges = [range_units(lod.range01), range_units(lod.range02)];
                let levels = [Some(path.to_owned()), mesh.lod_meshes[0].clone(), mesh.lod_meshes[1].clone()];
                for (level_index, mesh_path) in levels.iter().enumerate() {
                    let Some(mesh_path) = mesh_path else { continue };
                    let cull = ActorCull { lod: Some(LodSelect { level: level_index as u8, blend: lod.blend, ranges }), ..base };
                    draws.extend(mesh_draws(client, loaded, actor, mesh_path, level_index as u8, &lights, zone, cull, sun_affect, (center, radius)));
                }
            }
        }
    }
    draws
}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_color_defaults_to_no_color_stream_and_scales_bytes() {
        assert_eq!(vertex_color(None), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(vertex_color(Some([255, 0, 128, 255])), [1.0, 0.0, 128.0 / 255.0, 1.0]);
    }

    #[test]
    fn meshes_without_a_color_stream_take_the_point_lights_they_reach() {
        let lights = point_lights(&[ue2_level::Light {
            name: String::new(),
            location: [0.0, 0.0, 100.0],
            radius: 400.0,
            brightness: 255.0,
            hue: 0,
            saturation: 255,
            light_type: 1,
            effect: 0,
            emitter_light_type: 0,
            time_light: false,
            period: 32,
            phase: 0,
            cone: 128,
            direction: [1.0, 0.0, 0.0],
            rotation: [0; 3],
            special_lit: false,
            corona: None,
        }]);
        let lit = hardware_light(Vec3::ZERO, Vec3::Z, &lights, (Vec3::new(0.0, 0.0, 0.0), 10.0));
        assert_eq!(lit[3], 1.0);
        assert!(lit[0] > 0.3 && lit[0] < 0.4, "{lit:?}");
        assert_eq!(lit[0], lit[1]);
        assert_eq!(hardware_light(Vec3::new(1000.0, 0.0, 0.0), Vec3::Z, &lights, (Vec3::new(1000.0, 0.0, 0.0), 10.0)), [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn a_mirrored_placement_reverses_every_triangle() {
        let mut indices = vec![0, 1, 2, 3, 4, 5];
        mirror_winding(&mut indices);
        assert_eq!(indices, [0, 2, 1, 3, 5, 4]);
        let mirrored = placement_matrix([0.0; 3], [0, -16384, 0], [-1.0, 1.0, 1.0]);
        assert!(mirrored.determinant() < 0.0);
        assert!(placement_matrix([0.0; 3], [0, -16384, 0], [1.0, 1.0, 1.0]).determinant() > 0.0);
    }

    #[test]
    fn lod_picks_by_the_two_ranges_and_time_reactors_wrap_around_midnight() {
        assert_eq!(lod_for(4000.0, 4096.0, 8192.0), 0);
        assert_eq!(lod_for(4097.0, 4096.0, 8192.0), 1);
        assert_eq!(lod_for(8193.0, 4096.0, 8192.0), 2);
        assert!(time_reactor_shows(20.0, 6.0, 23.0));
        assert!(time_reactor_shows(20.0, 6.0, 3.0));
        assert!(!time_reactor_shows(20.0, 6.0, 12.0));
        assert!(time_reactor_shows(8.0, 18.0, 8.0));
        assert!(!time_reactor_shows(8.0, 18.0, 19.0));
    }

    #[test]
    fn draws_sort_only_when_the_material_requires_sorting() {
        let mut material = super::super::texture_material("Pack.Tex");
        assert_eq!(draw_pass(&material), super::super::Pass::Opaque);
        material.sorting_level = 2;
        assert_eq!(draw_pass(&material), super::super::Pass::Translucent);
    }

    #[test]
    fn section_material_falls_back_to_mesh_material_when_skin_is_null() {
        let skins = vec![None, Some("A.b".to_owned())];
        let materials = vec![Some("Mesh.Mat0".to_owned()), Some("Mesh.Mat1".to_owned())];
        assert_eq!(section_material(&skins, &materials, 0), Some("Mesh.Mat0".to_owned()));
        assert_eq!(section_material(&skins, &materials, 1), Some("A.b".to_owned()));
    }

    #[test]
    #[ignore = "reads the client"]
    fn every_unlit_actor_of_the_lobby_sky_draws_with_opaque_vertex_alpha() {
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let loaded = client.load_level("lobby").unwrap();
        let options = crate::options::ClientOptions { shared_sky: false, ..Default::default() };
        let scene = super::super::build_scene_with(&mut client, &loaded.level, &options, super::super::Neighbours::default());
        let sky: Vec<_> = scene.draws.iter().filter(|draw| scene.zone(draw.zone).is_sky && draw.kind == DrawKind::StaticMesh && draw.actor.is_some()).collect();
        assert!(sky.len() >= 3);
        for draw in sky {
            assert!(draw.vertices.iter().all(|vertex| vertex.color[3] == 1.0), "{:?}", draw.material.root_class);
        }
    }
}
