use super::{DrawKind, MeshDraw, Pass, Vertex, resolve_material, texture_material};
use crate::client::Client;
use crate::options::{ClientOptions, range_units};
use l2_catalog::{AlphaOp, BlendFactor, ColorOp, CoordSource, FogMode, Material, Stage, StageSource};
use ue2_level::{DrawStyle, Emitter, Level, SpriteEmitter};
use ue2_level::projectors::PARTICLES;

#[derive(Clone, Debug, PartialEq)]
pub struct EmitterAsset {
    pub definition: Emitter,
    pub parts: Vec<EmitterPart>,
    pub zone: u8,
    pub range: f32,
    pub lighting: Option<std::sync::Arc<super::actor_lighting::LightMap>>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct EmitterPart {
    pub vertex_mesh: Option<std::sync::Arc<l2_catalog::VertexAnimated>>,
    pub mesh_bounds: Option<[[f32; 3]; 2]>,
    pub draws: Vec<MeshDraw>,
    pub projector_materials: Vec<Vec<(usize, Material)>>,
    pub spawn_vertices: Vec<Vertex>,
    pub targets: Vec<Option<[f32; 3]>>,
}

pub fn build(client: &mut Client, level: &Level, options: &ClientOptions) -> Vec<EmitterAsset> {
    let lighting = level.emitters.iter().flat_map(|a| &a.sprites).any(|e| e.mesh.as_ref().or(e.vertex_mesh.as_ref()).is_some_and(|m| m.own_blend)).then(|| std::sync::Arc::new(super::actor_lighting::LightMap::new(level, &super::lights::point_lights(&level.lights))));
    let mut bitmap_materials = std::collections::HashMap::new();
    level.emitters.iter().map(|actor| {
        let zone = level.bsp_tree.zone_at(actor.location);
        emitter_asset(client, actor, Some(level), zone, lighting.clone(), &mut bitmap_materials, range_units(options.clipping.actor))
    }).collect()
}

pub fn effect_asset(client: &mut Client, actor: &Emitter, options: &ClientOptions) -> EmitterAsset {
    emitter_asset(client, actor, None, 0, None, &mut std::collections::HashMap::new(), range_units(options.clipping.actor))
}

fn emitter_asset(
    client: &mut Client,
    actor: &Emitter,
    level: Option<&Level>,
    zone: u8,
    lighting: Option<std::sync::Arc<super::actor_lighting::LightMap>>,
    bitmap_materials: &mut std::collections::HashMap<String, Option<l2_catalog::TextureRef>>,
    range: f32,
) -> EmitterAsset {
    let parts = actor.sprites.iter().map(|e| {
        let mut part = EmitterPart::default();
        if let Some(beam) = &e.beam {
            part.targets = beam.endpoints.iter().map(|endpoint| {
                let tag = endpoint.actor_tag.as_deref()?;
                level?.actors.iter().find(|a| a.tag.as_deref().is_some_and(|t| t.eq_ignore_ascii_case(tag))).map(|a| a.placement.location)
            }).collect();
        }
        if let Some(shape) = &e.mesh {
            if let Some(mesh) = shape.mesh.as_deref().and_then(|p| client.catalog().static_mesh(p)) {
                part.mesh_bounds = Some(mesh.mesh.bounding_box);
                for (i, section) in mesh.mesh.sections.iter().enumerate() {
                    let Some(path) = super::props::section_material(&actor.skins, &mesh.materials, i) else { continue };
                    let material = if shape.own_blend { resolve_material(client, &path) } else {
                        let bitmap = client.catalog().object_class(&path).is_some_and(|class| matches!(class.to_ascii_lowercase().as_str(), "texture" | "cubemap" | "scriptedtexture" | "firetexture" | "wettexture" | "icetexture" | "watertexture" | "wavetexture"));
                        resolved_particle_material(client, bitmap_materials, e, if bitmap { &path } else { "" }, false, shape.two_sided)
                    };
                    let projectors = particle_projector_materials(client, level, actor, e, Some(&path));
                    let first = section.first_index as usize;
                    let Some(source) = mesh.mesh.indices.get(first..first + section.index_count() as usize) else { continue };
                    let indices = if section.strip {
                        source.windows(3).enumerate().flat_map(|(i, t)| if i % 2 == 0 { [u32::from(t[0]), u32::from(t[1]), u32::from(t[2])] } else { [u32::from(t[1]), u32::from(t[0]), u32::from(t[2])] }).collect()
                    } else { source.iter().map(|i| u32::from(*i)).collect() };
                    part.draws.push(MeshDraw { instances: Vec::new(), vertices: mesh_vertices(&mesh), indices, material, mask: None,
                        kind: if shape.own_blend { DrawKind::StaticMesh } else { DrawKind::Particle },
                        sun_affect: actor.sun_affect,
                        pass: Pass::Translucent, zone, lightmap: None, modulate_2x: shape.own_blend, sector: None, fade: None, tint: None, actor: None });
                    part.projector_materials.push(projectors);
                }
            }
        } else if let Some(shape) = &e.vertex_mesh {
            if let Some(mesh) = shape.mesh.as_deref().and_then(|path| client.catalog().vertex_mesh(path)) {
                let mut pose = ue2_assets::VertexPose::new(&mesh.mesh);
                if let Ok(points) = pose.sample(None, 0.0, mesh.mesh.vertex_count) {
                    let vertices: Vec<Vertex> = mesh.mesh.wedges.iter().map(|w| {
                        let p = points[usize::from(w.vertex)];
                        Vertex { position: p.position, normal: p.normal, uv: w.uv, mask_uv: [0.0; 2], color: [1.0; 4] }
                    }).collect();
                    let mut face_offset = 0;
                    for faces in mesh.mesh.faces.chunk_by(|a, b| a[3] == b[3]) {
                        let texture = mesh.mesh.materials[usize::from(faces[0][3])].1;
                        let path = vertex_material(&actor.skins, &mesh.textures, texture).unwrap_or_default();
                        let material = if shape.own_blend { resolve_material(client, &path) } else {
                            let bitmap = client.catalog().object_class(&path).is_some_and(|class| matches!(class.to_ascii_lowercase().as_str(), "texture" | "cubemap" | "scriptedtexture" | "firetexture" | "wettexture" | "icetexture" | "watertexture" | "wavetexture"));
                            resolved_particle_material(client, bitmap_materials, e, if bitmap { &path } else { "" }, false, shape.two_sided)
                        };
                        let indices = faces.iter().enumerate().filter(|(i, _)| mesh.mesh.face_levels.get(face_offset + i).is_some_and(|&level| usize::from(level) <= mesh.mesh.vertex_count))
                            .flat_map(|(_, face)| [u32::from(face[2]), u32::from(face[1]), u32::from(face[0])]).collect();
                        face_offset += faces.len();
                        part.draws.push(MeshDraw { instances: Vec::new(), vertices: vertices.clone(), indices, material, mask: None,
                            kind: if shape.own_blend { DrawKind::StaticMesh } else { DrawKind::Particle }, sun_affect: actor.sun_affect,
                            pass: Pass::Translucent, zone, lightmap: None, modulate_2x: shape.own_blend, sector: None, fade: None, tint: None, actor: None });
                        part.projector_materials.push(particle_projector_materials(client, level, actor, e, Some(&path)));
                    }
                }
                part.vertex_mesh = Some(std::sync::Arc::new(mesh));
            }
        } else if (e.class == "SpriteEmitter" || e.beam.is_some()) && let Some(path) = &e.texture {
            part.draws.push(MeshDraw { instances: Vec::new(), vertices: Vec::new(), indices: Vec::new(), material: resolved_particle_material(client, bitmap_materials, e, path, true, true), mask: None,
                kind: DrawKind::Particle, sun_affect: false, pass: Pass::Translucent, zone, lightmap: None, modulate_2x: false,
                sector: None, fade: None, tint: None, actor: None });
            part.projector_materials.push(particle_projector_materials(client, level, actor, e, Some(path)));
        }
        if let Some(mesh) = e.mesh_spawning_mesh.as_deref().and_then(|p| client.catalog().static_mesh(p)) { part.spawn_vertices = mesh_vertices(&mesh); }
        part
    }).collect();
    EmitterAsset { definition: actor.clone(), parts, zone, range, lighting }
}

fn vertex_material(skins: &[Option<String>], textures: &[Option<String>], index: i32) -> Option<String> {
    let slot = usize::try_from(index).ok();
    if let Some(skin) = slot.and_then(|i| skins.get(i)).and_then(Option::as_ref) { return Some(skin.clone()); }
    if index != 0 && let Some(texture) = slot.and_then(|i| textures.get(i)) { return texture.clone(); }
    if let Some(skin) = skins.first() { return skin.clone(); }
    if let Some(texture) = slot.and_then(|i| textures.get(i)) { return texture.clone(); }
    if slot.is_some_and(|i| i >= textures.len()) { return textures.first().cloned().flatten(); }
    None
}

fn particle_projector_materials(
    client: &mut Client,
    level: Option<&Level>,
    actor: &Emitter,
    emitter: &SpriteEmitter,
    base: Option<&str>,
) -> Vec<(usize, Material)> {
    let Some(level) = level.filter(|_| emitter.accepts_projectors) else {
        return Vec::new();
    };
    level
        .projectors
        .iter()
        .enumerate()
        .filter(|(_, projector)| projector.flags & PARTICLES != 0 && projector.project_tag.as_deref().is_none_or(|tag| actor.tag.as_deref().is_some_and(|actor_tag| tag.eq_ignore_ascii_case(actor_tag))))
        .take(4)
        .filter_map(|(index, projector)| super::projectors::projector_material(client, projector, base).map(|material| (index, material)))
        .collect()
}

fn mesh_vertices(mesh: &l2_catalog::Mesh) -> Vec<Vertex> {
    mesh.mesh.positions.iter().enumerate().map(|(i, p)| Vertex { position: *p,
        normal: mesh.mesh.normals.get(i).copied().unwrap_or([0.0; 3]),
        uv: mesh.mesh.uvs.get(i).copied().unwrap_or([0.0; 2]), mask_uv: mesh.mesh.uvs2.get(i).copied().unwrap_or([0.0; 2]), color: mesh.mesh.colors.get(i).copied().unwrap_or([255; 4]).map(|c| f32::from(c) / 255.0) }).collect()
}

fn resolved_particle_material(client: &mut Client, cache: &mut std::collections::HashMap<String, Option<l2_catalog::TextureRef>>, e: &SpriteEmitter, path: &str, subdivisions: bool, two_sided: bool) -> Material {
    let mut material = particle_material(e, path, subdivisions, two_sided);
    if let Some(texture) = cache.entry(path.to_ascii_lowercase()).or_insert_with(|| client.catalog().material(path).and_then(|m| m.first_texture().cloned())) {
        for stage in &mut material.stages { if matches!(stage.source, StageSource::Texture(_)) { stage.source = StageSource::Texture(texture.clone()); } }
    }
    material
}

pub fn particle_material(e: &SpriteEmitter, path: &str, sprite: bool, two_sided: bool) -> Material {
    let mut material = texture_material(path);
    let first = &mut material.stages[0];
    first.color_op = if e.draw_style == DrawStyle::Modulated { ColorOp::BlendDiffuseAlpha } else { ColorOp::Modulate };
    first.alpha_op = if e.draw_style == DrawStyle::Modulated { AlphaOp::BlendDiffuseAlpha } else { AlphaOp::Modulate };
    if sprite && e.blend_subdivisions && !e.random_subdivision {
        first.color_op = ColorOp::SelectTexture;
        first.alpha_op = AlphaOp::Texture;
        let mut next = first.clone();
        next.coord_source = CoordSource::Stream(1);
        next.color_op = ColorOp::BlendDiffuseAlpha;
        next.alpha_op = AlphaOp::BlendDiffuseAlpha;
        material.stages.push(next);
        material.stages.push(Stage { source: StageSource::Diffuse, uv: Vec::new(), coord_source: CoordSource::Stream(0), color_op: ColorOp::Modulate, alpha_op: AlphaOp::Current, swapped: false });
    }
    let pass = &mut material.passes[0];
    (pass.src, pass.dst) = match e.draw_style {
        DrawStyle::Regular => (BlendFactor::One, BlendFactor::Zero),
        DrawStyle::AlphaBlend => (BlendFactor::SrcAlpha, BlendFactor::InvSrcAlpha),
        DrawStyle::Modulated => (BlendFactor::DstColor, BlendFactor::SrcColor),
        DrawStyle::Translucent => (BlendFactor::One, BlendFactor::One),
        DrawStyle::AlphaModulate => (BlendFactor::One, BlendFactor::InvSrcAlpha),
        DrawStyle::Darken => (BlendFactor::Zero, BlendFactor::InvSrcColor),
        DrawStyle::Brighten => (BlendFactor::One, BlendFactor::InvSrcColor),
    };
    pass.stages = 0..material.stages.len();
    pass.lit = false;
    pass.alpha_blend = e.draw_style != DrawStyle::Regular;
    pass.z_test = e.z_test;
    pass.z_write = e.z_write;
    pass.alpha_test = e.alpha_test.then_some(e.alpha_ref.min(255) as u8);
    pass.two_sided = two_sided;
    pass.fog = match e.draw_style { DrawStyle::Regular | DrawStyle::AlphaBlend => FogMode::Zone, DrawStyle::Modulated => FogMode::Gray127, _ => FogMode::Black };
    material.transparent = pass.alpha_blend;
    material
}
