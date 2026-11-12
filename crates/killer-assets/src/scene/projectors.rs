use super::props::{mirror_winding, section_material};
use super::terrain_normals::safe_normal;
use super::{DrawKind, MeshDraw, Neighbours, Pass, Vertex, terrain, terrain_normals, transform::placement_matrix};
use crate::client::Client;
use glam::{Mat4, Vec3, Vec4};
use l2_catalog::{AlphaOp, BlendFactor, ColorOp, CoordSource, DynamicProjectorParameters, FogMode, Material, Stage, StageSource};
use ue2_level::projectors::Projector;
use ue2_level::projectors::{GRADIENT, ON_BACKFACES, STATIC_MESH, TERRAIN};
use std::collections::HashMap;

pub const MAX_DYNAMIC_PROJECTORS: usize = 256;

#[derive(Clone, Debug, PartialEq)]
pub struct BspProjectorVertex {
    pub position: [f32; 3],
    pub uvz: [f32; 3],
    pub color: [u8; 4],
    pub base_uv: Option<[f32; 2]>,
}

pub fn bsp_vertex_stream(
    p: &Projector,
    projection: &Projection,
    points: &[Vec3],
    normal: Vec3,
    base_material: bool,
    time: f64,
    expires: f64,
) -> Vec<BspProjectorVertex> {
    let direction = rotation(p.placement.rotation).x_axis.as_dvec3();
    let normal = normal.as_dvec3();
    let facing = if p.flags & ON_BACKFACES != 0 {
        1.0
    } else {
        (-direction.x * normal.x - direction.y * normal.y - direction.z * normal.z).max(0.0)
    };
    let lifetime = if expires == 0.0 { 1.0 } else { (expires - time).clamp(0.0, 1.0) };
    let inverse_distance = f64::from((1.0 / f64::from(p.max_trace_distance)) as f32);
    let matrix = projection.matrix.to_cols_array_2d();
    points
        .iter()
        .map(|point| {
            let delta = point.as_dvec3() - Vec3::from(p.placement.location).as_dvec3();
            let gradient = if p.flags & GRADIENT != 0 && p.gradient_texture.is_some() {
                (1.0 - (delta.x * direction.x + delta.z * direction.z + delta.y * direction.y) * inverse_distance)
                    .clamp(0.0, 1.0)
            } else {
                1.0
            };
            let color = ((gradient * facing * lifetime * 255.0) as f32).floor().clamp(0.0, 255.0) as u8;
            let uvz = std::array::from_fn(|i| {
                (f64::from(point.x) * f64::from(matrix[0][i])
                    + f64::from(point.y) * f64::from(matrix[1][i])
                    + f64::from(point.z) * f64::from(matrix[2][i])
                    + f64::from(matrix[3][i])) as f32
            });
            BspProjectorVertex {
                position: point.to_array(),
                uvz,
                color: [color; 4],
                base_uv: base_material.then_some([0.0; 2]),
            }
        })
        .collect()
}

pub fn bsp_fan_indices(vertices: usize, offset: u16) -> Vec<u16> {
    (2..vertices)
        .flat_map(|i| [offset, offset.wrapping_add(i as u16).wrapping_sub(1), offset.wrapping_add(i as u16)])
        .collect()
}

pub fn clip_bsp_polygon(points: &[Vec3], planes: &[Vec4], no_overflow: bool) -> Vec<Vec3> {
    let mut points = points.to_vec();
    for plane in planes {
        let n = plane.truncate().as_dvec3();
        let base = (plane.truncate() * plane.w).as_dvec3();
        let w = f64::from((n.y * base.y + n.z * base.z + n.x * base.x) as f32);
        let distances: Vec<_> = points
            .iter()
            .map(|point| {
                let p = point.as_dvec3();
                n.y * p.y + n.x * p.x + n.z * p.z - w
            })
            .collect();
        let front = distances.iter().any(|d| *d > 0.25);
        let back = distances.iter().any(|d| *d < -0.25);
        if back && !front {
            return Vec::new();
        }
        if !back || !front || (no_overflow && points.len() >= 14) {
            continue;
        }
        let mut clipped = Vec::with_capacity(points.len() + 1);
        for i in 0..points.len() {
            let previous = (i + points.len() - 1) % points.len();
            if (distances[previous] < 0.0) != (distances[i] < 0.0) {
                let a = points[previous].as_dvec3();
                let delta = points[i].as_dvec3() - a;
                let rounded = delta.as_vec3().as_dvec3();
                let numerator = w - (a.x * n.x + a.z * n.z + a.y * n.y);
                let denominator = rounded.z * n.z + rounded.y * n.y + delta.x * n.x;
                let t = numerator / denominator;
                clipped.push(Vec3::new(
                    (rounded.x * t) as f32 + a.x as f32,
                    (rounded.y * t) as f32 + a.y as f32,
                    (rounded.z * t + a.z) as f32,
                ));
            }
            if distances[i] >= 0.0 {
                clipped.push(points[i]);
            }
        }
        points = clipped;
    }
    points
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectorState {
    pub enabled: bool,
    pub visibility: ue2_level::projectors::Visibility,
    pub owners: Vec<String>,
    pub detail: u8,
    pub draws: Vec<usize>,
}

impl ProjectorState {
    fn from_level(level: &ue2_level::Level, projector: &Projector) -> Self {
        let mut owners = vec![projector.name.clone()];
        while let Some(owner) = owners
            .last()
            .and_then(|name| level.actors.iter().find(|actor| actor.name == *name))
            .and_then(|actor| actor.owner.as_ref())
        {
            if owners.contains(owner) {
                break;
            }
            owners.push(owner.clone());
        }
        Self {
            enabled: projector.texture.is_some(),
            visibility: projector.visibility,
            owners,
            detail: level.detail_mode,
            draws: Vec::new(),
        }
    }
}

pub fn visible_projectors(
    states: &[ProjectorState],
    view_actor: Option<&str>,
    behind_view: bool,
    wireframe: bool,
) -> Vec<bool> {
    let mut count = 0;
    states
        .iter()
        .map(|state| {
            let owned = view_actor.is_some_and(|actor| state.owners.iter().any(|owner| owner == actor));
            let visible = state.enabled
                && state.visibility.visible(owned, behind_view, wireframe, state.detail)
                && count < MAX_DYNAMIC_PROJECTORS;
            count += usize::from(visible);
            visible
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct Projection {
    pub matrix: Mat4,
    pub gradient: Mat4,
    pub vertices: [Vec3; 8],
    pub planes: [Vec4; 6],
    pub min: Vec3,
    pub max: Vec3,
}

impl Projection {
    pub fn outcode(&self, point: Vec3) -> u8 {
        self.planes.iter().enumerate().fold(0, |bits, (i, p)| {
            let dot = f64::from(p.y) * f64::from(point.y)
                + f64::from(p.z) * f64::from(point.z)
                + f64::from(p.x) * f64::from(point.x);
            bits | (u8::from(dot <= f64::from(p.w)) << i)
        })
    }

    pub fn normal_matrix(&self) -> Mat4 {
        let transposed = self.matrix.transpose();
        coords(
            Vec3::ZERO,
            transposed.x_axis.truncate().normalize_or_zero(),
            transposed.y_axis.truncate().normalize_or_zero(),
            -transposed.z_axis.truncate().normalize_or_zero(),
        )
    }
}

pub fn terrain_material(
    p: &Projector,
    projection: &Projection,
    mut material: Material,
    gradient: Option<Stage>,
) -> Material {
    for stage in &mut material.stages {
        stage.coord_source =
            CoordSource::Projection { matrix: projection.matrix.to_cols_array_2d(), divide_z: p.fov != 0 };
    }
    if matches!(p.material_blend, 1 | 2)
        && let Some(stage) = material.stages.first_mut()
    {
        stage.color_op = if p.material_blend == 1 { ColorOp::Modulate2X } else { ColorOp::SelectTexture };
        stage.alpha_op = AlphaOp::Texture;
    }
    let attenuation = |source, coord_source| Stage {
        source,
        coord_source,
        uv: Vec::new(),
        swapped: false,
        color_op: match p.frame_blend {
            2 => ColorOp::SelectCurrent,
            3 => ColorOp::Modulate,
            _ => ColorOp::BlendCurrentFactorByTextureAlpha,
        },
        alpha_op: if p.frame_blend == 2 { AlphaOp::Modulate } else { AlphaOp::Current },
    };
    if let Some(gradient) = gradient {
        let mut stage = attenuation(
            gradient.source,
            CoordSource::Projection { matrix: projection.gradient.to_cols_array_2d(), divide_z: false },
        );
        if p.frame_blend == 3 { stage.color_op = ColorOp::ModulateCurrentByTextureAlpha; }
        material.stages.push(stage);
    }
    if p.flags & ON_BACKFACES == 0 {
        material.stages.push(attenuation(
            StageSource::ProjectorFacing,
            CoordSource::ProjectorNormal { matrix: projection.normal_matrix().to_cols_array_2d() },
        ));
    }
    let stage_count = material.stages.len();
    for pass in &mut material.passes {
        pass.stages.end = stage_count;
        (pass.src, pass.dst) = match p.frame_blend {
            2 => (BlendFactor::SrcAlpha, BlendFactor::InvSrcAlpha),
            3 => (BlendFactor::One, BlendFactor::One),
            _ => (BlendFactor::DstColor, BlendFactor::SrcColor),
        };
        pass.alpha_blend = true;
        pass.z_test = true;
        pass.z_write = false;
        pass.lit = false;
        pass.light_after = stage_count;
        pass.fog = match p.frame_blend {
            2 => FogMode::Zone,
            3 => FogMode::Black,
            _ => FogMode::Gray127,
        };
        pass.tfactor = Some([127, 127, 127, 0]);
    }
    material.transparent = true;
    material
}

pub fn build_terrain(client: &mut Client, level: &ue2_level::Level, neighbours: Neighbours<'_>) -> Vec<MeshDraw> {
    build_terrain_with_states(client, level, neighbours).0
}

pub fn build_with_states(
    client: &mut Client,
    level: &ue2_level::Level,
    neighbours: Neighbours<'_>,
) -> (Vec<MeshDraw>, Vec<ProjectorState>) {
    let (mut draws, mut states) = build_terrain_with_states(client, level, neighbours);
    for (index, p) in level.projectors.iter().enumerate() {
        let Some(path) = &p.texture else { continue };
        let Some(projected) = client.catalog().material(path) else { continue };
        let Some(texture) = projected.first_texture() else { continue };
        let projection = calc_matrix(p, texture.size.map(|n| n as u32), None);
        for polygon in &level.bsp {
            let Some(section) = level.bsp_render.node_draws.get(polygon.node_index)
                .and_then(|node| usize::try_from(node.section).ok())
                .and_then(|index| level.bsp_render.sections.get(index)) else { continue };
            if polygon.flags & 0x8000 != 0
                || (section.poly_flags & 0x400000 != 0 && p.flags & ue2_level::projectors::ON_UNLIT == 0)
            {
                continue;
            }
            let points: Vec<_> = polygon.corners.iter().copied().map(Vec3::from).collect();
            let points: Vec<_> = clip_bsp_polygon(&points, &projection.planes, true).into_iter().rev().collect();
            if points.len() < 3 { continue; }
            let Some(mut material) = client.catalog().bsp_projector_material(path, section.material.as_deref(),
                p.material_blend, p.frame_blend, projection.matrix.to_cols_array_2d()) else { continue };
            for stage in &mut material.stages {
                if matches!(stage.coord_source, CoordSource::Projection { .. }) {
                    stage.coord_source = CoordSource::ProjectorStream;
                }
            }
            let stream = bsp_vertex_stream(p, &projection, &points, Vec3::from(polygon.normal), true, 0.0, 0.0);
            let vertices = stream.into_iter().map(|v| Vertex {
                position: v.position, normal: v.uvz, uv: [0.0; 2],
                mask_uv: v.base_uv.unwrap_or_default(), color: v.color.map(|c| f32::from(c) / 255.0),
            }).collect();
            states[index].draws.push(draws.len());
            draws.push(MeshDraw {
                instances: Vec::new(), vertices, indices: bsp_fan_indices(points.len(), 0).into_iter().map(u32::from).collect(),
                material, mask: None, kind: DrawKind::Projector, sun_affect: false,
                pass: Pass::Projector, zone: polygon.zone, lightmap: None, modulate_2x: false,
                sector: None, fade: None, tint: None, actor: None,
            });
        }
    }
    (draws, states)
}

pub fn build_static_mesh_with_states(
    client: &mut Client,
    level: &ue2_level::Level,
    start_index: usize,
    states: &mut [ProjectorState],
) -> Vec<MeshDraw> {
    let mut meshes: HashMap<String, Option<l2_catalog::Mesh>> = HashMap::new();
    let mut draws = Vec::new();
    for (projector_index, projector) in level.projectors.iter().enumerate() {
        if projector.flags & STATIC_MESH == 0 {
            continue;
        }
        let Some(projection) = projector_projection(client, projector) else { continue };
        for actor in level.actors.iter().filter(|actor| actor.l2_movement.is_none() && actor.mover.is_none() && !actor.hidden) {
            let Some(mesh_path) = actor.static_mesh.as_deref() else { continue };
            let key = mesh_path.to_lowercase();
            if !meshes.contains_key(&key) {
                meshes.insert(key.clone(), client.catalog().static_mesh(mesh_path));
            }
            let Some(mesh) = meshes.get(&key).and_then(|mesh| mesh.as_ref()) else { continue };
            let world = placement_matrix(actor.placement.location, actor.placement.rotation, actor.scale);
            let [min, max] = mesh.mesh.bounding_box;
            let corners: Vec<Vec3> = [min[0], max[0]]
                .into_iter()
                .flat_map(|x| [min[1], max[1]].into_iter().flat_map(move |y| [min[2], max[2]].map(move |z| world.transform_point3(Vec3::new(x, y, z)))))
                .collect();
            let outside = (0..6).any(|plane| corners.iter().all(|corner| projection.outcode(*corner) & (1 << plane) != 0));
            if outside {
                continue;
            }
            let normal_matrix = glam::Mat3::from_mat4(world).inverse().transpose();
            let mirrored = world.determinant() < 0.0;
            let zone = level.bsp_tree.zone_at(actor.placement.location);
            for (section_index, section) in mesh.mesh.sections.iter().enumerate() {
                if section.primitives == 0 || mesh.mesh.no_dynamic_shadow_cast.get(section_index).copied().unwrap_or(false) {
                    continue;
                }
                let material_path = section_material(&actor.skins, &mesh.materials, section_index);
                let Some(material) = projector_material(client, projector, material_path.as_deref()) else { continue };
                let first = section.first_index as usize;
                let count = section.index_count() as usize;
                let Some(section_indices) = mesh.mesh.indices.get(first..first + count) else { continue };
                let mut remap = HashMap::new();
                let mut vertices = Vec::new();
                let mut indices = Vec::with_capacity(count);
                for &index in section_indices {
                    let next = vertices.len() as u32;
                    let local = *remap.entry(index).or_insert_with(|| {
                        let index = index as usize;
                        let position = world.transform_point3(Vec3::from(mesh.mesh.positions[index]));
                        let normal = (normal_matrix * Vec3::from(mesh.mesh.normals.get(index).copied().unwrap_or([0.0, 0.0, 1.0]))).normalize_or(Vec3::Z);
                        vertices.push(Vertex {
                            position: position.to_array(),
                            normal: normal.to_array(),
                            uv: mesh.mesh.uvs.get(index).copied().unwrap_or([0.0, 0.0]),
                            mask_uv: mesh.mesh.uvs2.get(index).copied().or_else(|| mesh.mesh.uvs.get(index).copied()).unwrap_or([0.0, 0.0]),
                            color: [1.0; 4],
                        });
                        next
                    });
                    indices.push(local);
                }
                if mirrored {
                    mirror_winding(&mut indices);
                }
                states[projector_index].draws.push(start_index + draws.len());
                draws.push(MeshDraw {
                    instances: Vec::new(), vertices,
                    indices,
                    material,
                    mask: None,
                    kind: DrawKind::Projector,
                    sun_affect: false,
                    pass: Pass::Projector,
                    zone,
                    lightmap: None,
                    modulate_2x: false,
                    sector: None,
                    fade: None,
                    tint: None,
                    actor: None,
                });
            }
        }
    }
    draws
}

pub fn projector_material(client: &mut Client, projector: &Projector, base: Option<&str>) -> Option<Material> {
    let path = projector.texture.as_deref()?;
    let projection = projector_projection(client, projector)?;
    let gradient = if projector.flags & GRADIENT != 0 {
        projector
            .gradient_texture
            .as_deref()
            .and_then(|gradient| client.catalog().material(gradient))
            .and_then(|material| material.stages.into_iter().next())
    } else {
        None
    };
    let parameters = DynamicProjectorParameters {
        material_blend: projector.material_blend,
        frame_blend: projector.frame_blend,
        projected: projector.fov != 0,
        on_alpha: projector.flags & ue2_level::projectors::ON_ALPHA != 0,
        on_backfaces: projector.flags & ON_BACKFACES != 0,
        projection: projection.matrix.to_cols_array_2d(),
        normal: projection.normal_matrix().to_cols_array_2d(),
        gradient: gradient.and_then(|_| projector.gradient_texture.clone()).map(|gradient| (gradient, projection.gradient.to_cols_array_2d())),
    };
    client.catalog().dynamic_projector_material(path, base, &parameters)
}

fn projector_projection(client: &mut Client, projector: &Projector) -> Option<Projection> {
    let path = projector.texture.as_deref()?;
    let size = client.catalog().material(path)?.first_texture()?.size.map(|value| value as u32);
    let gradient = if projector.flags & GRADIENT != 0 {
        projector
            .gradient_texture
            .as_deref()
            .and_then(|gradient| client.catalog().material(gradient))
            .and_then(|material| material.stages.into_iter().next())
    } else {
        None
    };
    let gradient_size = gradient.as_ref().and_then(|stage| match &stage.source {
        StageSource::Texture(texture) => Some(texture.size[1] as i32),
        _ => None,
    });
    Some(calc_matrix(projector, size, gradient_size))
}

pub fn build_terrain_with_states(
    client: &mut Client,
    level: &ue2_level::Level,
    neighbours: Neighbours<'_>,
) -> (Vec<MeshDraw>, Vec<ProjectorState>) {
    let mut draws = Vec::new();
    let mut states: Vec<_> = level.projectors.iter().map(|p| ProjectorState::from_level(level, p)).collect();
    if !level.projectors.iter().any(|p| p.flags & TERRAIN != 0 && p.texture.is_some()) {
        return (draws, states);
    }
    let mut materials = Vec::new();
    for (index, p) in level.projectors.iter().enumerate().filter(|(_, p)| p.flags & TERRAIN != 0) {
        let Some(path) = &p.texture else { continue };
        let Some(material) = client.catalog().material(path) else { continue };
        let Some(size) = material.first_texture().map(|t| t.size.map(|v| v as u32)) else { continue };
        let gradient = if p.flags & GRADIENT != 0 {
            p.gradient_texture
                .as_deref()
                .and_then(|path| client.catalog().material(path))
                .and_then(|m| m.stages.into_iter().next())
        } else {
            None
        };
        let gradient_size = gradient.as_ref().and_then(|s| match &s.source {
            StageSource::Texture(t) => Some(t.size[1] as i32),
            _ => None,
        });
        let projection = calc_matrix(p, size, gradient_size);
        let parameters = DynamicProjectorParameters {
            material_blend: p.material_blend,
            frame_blend: p.frame_blend,
            projected: p.fov != 0,
            on_alpha: p.flags & ue2_level::projectors::ON_ALPHA != 0,
            on_backfaces: p.flags & ON_BACKFACES != 0,
            projection: projection.matrix.to_cols_array_2d(),
            normal: projection.normal_matrix().to_cols_array_2d(),
            gradient: gradient.and_then(|_| p.gradient_texture.clone())
                .map(|path| (path, projection.gradient.to_cols_array_2d())),
        };
        let Some(material) = client.catalog().dynamic_projector_material(path, None, &parameters) else { continue };
        materials.push((index, projection, material));
    }
    for terrain in &level.terrains {
        let Some(grid) = terrain::seamed_grid(client, terrain, &neighbours.terrain_seam()) else { continue };
        let Some(normals) = terrain_normals::face_buffer(terrain, &grid) else { continue };
        for sector in &terrain.sectors {
            let quads = terrain::runtime_quads(sector).map(|v| v as usize);
            let [ox, oy] = sector.offset.map(|v| v as usize);
            if ox + quads[0] >= grid.width || oy + quads[1] >= grid.height {
                continue;
            }
            let columns = quads[0] + 1;
            let mut vertices = Vec::new();
            for y in 0..=quads[1] {
                for x in 0..=quads[0] {
                    vertices.push(Vertex {
                        position: grid.position(ox + x, oy + y),
                        normal: terrain_normals::vertex_normal(
                            &normals,
                            grid.heightmap[0],
                            grid.heightmap[0] * grid.heightmap[1],
                            terrain_normals::sector_point([ox, oy], [x, y]),
                            terrain.inverted,
                        ),
                        uv: [0.0; 2],
                        mask_uv: [0.0; 2],
                        color: [1.0; 4],
                    });
                }
            }
            let mut bounds = terrain::sector_bounds(&vertices);
            bounds.location = terrain::sector_location(
                grid.position(ox, oy),
                grid.position(ox + quads[0], oy),
                grid.position(ox, oy + quads[1]),
            );
            let applicable: Vec<_> = materials
                .iter()
                .filter(|(_, p, _)| {
                    Vec3::from(bounds.min).cmple(p.max).all() && Vec3::from(bounds.max).cmpge(p.min).all()
                })
                .collect();
            if applicable.is_empty() {
                continue;
            }
            let mut all_indices = Vec::new();
            for y in 0..quads[1] {
                for x in 0..quads[0] {
                    let a = (y * columns + x) as u32;
                    let w = columns as u32;
                    all_indices.extend_from_slice(&terrain::quad_triangles(
                        a,
                        a + 1,
                        a + w,
                        a + w + 1,
                        terrain::edge_turned(&terrain.edge_turn, grid.heightmap[0], ox + x, oy + y),
                    ));
                }
            }
            for (index, projection, material) in applicable {
                let codes: Vec<_> = vertices.iter().map(|v| projection.outcode(Vec3::from(v.position))).collect();
                let indices: Vec<_> = all_indices
                    .as_chunks::<3>().0.iter()
                    .filter(|tri| tri.iter().fold(63u8, |code, i| code & codes[*i as usize]) == 0)
                    .flatten()
                    .copied()
                    .collect();
                if indices.is_empty() {
                    continue;
                }
                states[*index].draws.push(draws.len());
                draws.push(MeshDraw {
                    instances: Vec::new(), vertices: vertices.clone(),
                    indices,
                    material: material.clone(),
                    mask: None,
                    kind: DrawKind::Projector,
                    sun_affect: false,
                    pass: Pass::Projector,
                    zone: super::terrain_zone(level, terrain),
                    lightmap: None,
                    modulate_2x: false,
                    sector: Some(bounds),
                    fade: None,
                    tint: None,
                    actor: None,
                });
            }
        }
    }
    (draws, states)
}

fn coords(origin: Vec3, x: Vec3, y: Vec3, z: Vec3) -> Mat4 {
    Mat4::from_cols(
        Vec4::new(x.x, y.x, z.x, 0.0),
        Vec4::new(x.y, y.y, z.y, 0.0),
        Vec4::new(x.z, y.z, z.z, 0.0),
        Vec4::new(-origin.dot(x), -origin.dot(y), -origin.dot(z), 1.0),
    )
}

fn plane(normal: Vec3, point: Vec3) -> Vec4 {
    normal.extend(normal.as_dvec3().dot(point.as_dvec3()) as f32)
}

fn rotation(rotator: [i32; 3]) -> glam::Mat3 {
    let sin = |v: i32| f64::from(((v & 65532) as f64 * std::f64::consts::TAU / 65536.0).sin() as f32);
    let [p, y, r] = rotator;
    let (sp, cp) = (sin(p), sin(p.wrapping_add(16384)));
    let (sy, cy) = (sin(y), sin(y.wrapping_add(16384)));
    let (sr, cr) = (sin(r), sin(r.wrapping_add(16384)));
    glam::Mat3::from_cols(
        glam::DVec3::new(cp * cy, cp * sy, sp).as_vec3(),
        glam::DVec3::new(sr * sp * cy - cr * sy, sr * sp * sy + cr * cy, -sr * cp).as_vec3(),
        glam::DVec3::new(-(cr * sp * cy + sr * sy), cy * sr - cr * sp * sy, cr * cp).as_vec3(),
    )
}

pub fn facing_cube(size: u32) -> [Vec<u8>; 6] {
    let faces = [
        (Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 0.0, -2.0), Vec3::new(0.0, -2.0, 0.0)),
        (Vec3::new(-1.0, 1.0, -1.0), Vec3::new(0.0, 0.0, 2.0), Vec3::new(0.0, -2.0, 0.0)),
        (Vec3::new(-1.0, 1.0, -1.0), Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 2.0)),
        (Vec3::new(-1.0, -1.0, 1.0), Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -2.0)),
        (Vec3::new(-1.0, 1.0, 1.0), Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.0, -2.0, 0.0)),
        (Vec3::new(1.0, 1.0, -1.0), Vec3::new(-2.0, 0.0, 0.0), Vec3::new(0.0, -2.0, 0.0)),
    ];
    faces.map(|(origin, u, v)| {
        let mut data = Vec::with_capacity((size * size * 4) as usize);
        let du = u / (size as f32 - 1.0);
        let dv = v / (size as f32 - 1.0);
        for y in 0..size {
            let mut point = origin + dv * (y as f32 + 0.5) + du * 0.5;
            for _ in 0..size {
                let value = (point.normalize_or_zero().z * 255.0).floor().clamp(0.0, 255.0) as u8;
                data.extend_from_slice(&[value; 4]);
                point += du;
            }
        }
        data
    })
}

pub fn calc_matrix(p: &Projector, size: [u32; 2], gradient_size: Option<i32>) -> Projection {
    let rotation = rotation(p.placement.rotation);
    let dir = rotation.x_axis;
    let up = -rotation.y_axis;
    let right = rotation.z_axis;
    let location = Vec3::from_array(p.placement.location);
    let width = size[0] as f32 * p.draw_scale;
    let height = size[1] as f32 * p.draw_scale;
    let radius = if p.radius > 0.0 { p.radius * 1.5 } else { (width * width * 0.25 + height * height * 0.25).sqrt() };
    let diagonals = [up + right, right - up, -up - right, up - right];
    let near = diagonals.map(|v| location + (if p.physics == 5 { v } else { safe_normal(v) }) * radius);
    let mut vertices = [Vec3::ZERO; 8];
    vertices[..4].copy_from_slice(&near);
    let mut planes = [Vec4::ZERO; 6];
    planes[0] = plane(dir, location);
    let distance = p.max_trace_distance as f32;
    let matrix = if p.fov == 0 {
        for i in 0..4 {
            planes[i + 1] = plane(-safe_normal(diagonals[i]), near[i]);
            vertices[i + 4] = near[i] + dir * distance;
        }
        coords(location + up * (0.5 * width) + right * (0.5 * height), -up / width, -right / height, dir)
    } else {
        let angle = f64::from(p.fov) * 0.5 * std::f64::consts::PI / 180.0;
        let tangent = angle.tan() as f32;
        let cosine = angle.cos() as f32;
        let origin = location - dir * (width * 0.5 / tangent);
        for i in 0..4 {
            let a = (near[(i + 1) % 4] - origin).as_dvec3();
            let b = (near[i] - origin).as_dvec3();
            let normal = safe_normal(a.cross(b).as_vec3());
            planes[i + 1] = plane(normal, origin);
            vertices[i + 4] = near[i] + safe_normal(near[i] - origin) * distance / cosine;
        }
        let k = 0.5 / tangent;
        let perspective = Mat4::from_cols(
            Vec4::new(k, 0.0, 0.0, 0.0),
            Vec4::new(0.0, k, 0.0, 0.0),
            Vec4::new(0.5, 0.5, 1.0, 0.0),
            Vec4::W,
        );
        perspective * coords(origin, -up, -right, dir)
    };
    planes[5] = plane(-dir, vertices[7]);
    let gradient = if let Some(n) = gradient_size.filter(|n| *n != 0) {
        let offset = (p.max_trace_distance / n) as f32;
        let divisor = (f64::from(n.wrapping_mul(p.max_trace_distance)) / f64::from(n - 2)) as f32;
        coords(location - dir * offset, Vec3::ZERO, dir / divisor, Vec3::ZERO)
    } else {
        Mat4::ZERO
    };
    Projection {
        matrix,
        gradient,
        vertices,
        planes,
        min: vertices.into_iter().fold(Vec3::splat(f32::INFINITY), Vec3::min),
        max: vertices.into_iter().fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fpoly_split_1054c690_preserves_tolerance_and_no_overflow() {
        let points = [
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(-1.0, 1.0, 0.0),
        ];
        assert_eq!(
            clip_bsp_polygon(&points, &[Vec4::X], true),
            vec![Vec3::new(0.0, -1.0, 0.0), points[1], points[2], Vec3::new(0.0, 1.0, 0.0)]
        );
        assert_eq!(clip_bsp_polygon(&points, &[Vec4::new(0.0, 0.0, 1.0, 0.25)], true), points);
        assert!(clip_bsp_polygon(&points, &[Vec4::new(0.0, 0.0, 1.0, 0.25001)], true).is_empty());
        let polygon: Vec<_> = (0..14)
            .map(|i| {
                let angle = i as f32 * std::f32::consts::TAU / 14.0;
                Vec3::new(angle.cos(), angle.sin(), 0.0)
            })
            .collect();
        assert_eq!(clip_bsp_polygon(&polygon, &[Vec4::X], true), polygon);
        assert!(clip_bsp_polygon(&polygon, &[Vec4::X], false).len() < 14);
    }

    #[test]
    fn dynamic_bsp_projector_stream_106279c0_attenuation_and_indices() {
        let p = Projector { flags: GRADIENT, ..Default::default() };
        let projection = calc_matrix(&p, [128, 128], None);
        let points = [Vec3::ZERO, Vec3::new(500.0, 0.0, 0.0), Vec3::new(1000.0, 0.0, 0.0)];
        let vertices = bsp_vertex_stream(&p, &projection, &points, -Vec3::X, true, 5.0, 0.0);
        assert_eq!(vertices.iter().map(|v| v.color[0]).collect::<Vec<_>>(), [255, 127, 0]);
        assert!(vertices.iter().all(|v| v.base_uv == Some([0.0; 2])));
        assert_eq!(bsp_fan_indices(4, 65534), [65534, 65535, 0, 65534, 0, 1]);
        assert!(
            bsp_vertex_stream(&p, &projection, &points, Vec3::X, false, 0.0, 0.0).iter().all(|v| v.color == [0; 4])
        );
    }

    #[test]
    fn render_level_1064e500_caps_projectors_not_receiver_draws() {
        let state = ProjectorState {
            enabled: true,
            visibility: Default::default(),
            owners: vec!["self".into(), "owner".into()],
            detail: 2,
            draws: Vec::new(),
        };
        let mut states = vec![state; 260];
        states[0].enabled = false;
        states[1].visibility.only_owner = true;
        states[2].visibility.high_detail = true;
        states[2].detail = 0;
        states[259].draws = vec![0, 1, 2];
        let visible = visible_projectors(&states, None, false, false);
        assert_eq!(visible.iter().filter(|&&v| v).count(), 256);
        assert!(!visible[259]);
        states[258].enabled = false;
        assert!(visible_projectors(&states, None, false, false)[259]);
        assert!(visible_projectors(&states, Some("owner"), false, false)[1]);
        assert!(visible_projectors(&states, None, false, true).iter().all(|v| !v));
    }

    #[test]
    fn filter_projector_106159d0_owner_branch_takes_precedence() {
        let flags = ue2_level::projectors::Visibility { only_owner: true, owner_no_see: true, ..Default::default() };
        assert!(!flags.visible(false, true, false, 2));
        assert!(!flags.visible(true, false, false, 2));
        assert!(flags.visible(true, true, false, 2));
        assert!(!flags.visible(true, true, true, 2));
    }

    #[test]
    fn calc_matrix_1060c540_orthographic_orientation_and_gradient() {
        let p = Projector {
            placement: ue2_level::Placement { location: [10.0, 20.0, 30.0], rotation: [0; 3] },
            ..Default::default()
        };
        let result = calc_matrix(&p, [128, 256], Some(256));
        let center = result.matrix * Vec4::new(10.0, 20.0, 30.0, 1.0);
        assert_eq!(center, Vec4::new(0.5, 0.5, 0.0, 1.0));
        assert_eq!((result.matrix * Vec4::new(10.0, 84.0, 158.0, 1.0)).truncate(), Vec3::new(1.0, 0.0, 0.0));
        assert!((result.gradient.x_axis.y - 0.0009921875).abs() < 1e-9);
        assert!((result.vertices[0].y + 81.19289).abs() < 0.0001);
    }

    #[test]
    fn calc_matrix_1060c540_perspective_divides_by_z_not_w() {
        let p = Projector { fov: 45, ..Default::default() };
        let result = calc_matrix(&p, [128, 128], None);
        for point in [Vec3::ZERO, Vec3::X * 1000.0] {
            let uvz = result.matrix * point.extend(1.0);
            assert!((uvz.x / uvz.z - 0.5).abs() < 1e-6);
            assert_eq!(uvz.w, 1.0);
        }
        assert!(result.planes.iter().all(|p| p.truncate().dot(Vec3::X * 100.0) >= p.w));
    }
}
