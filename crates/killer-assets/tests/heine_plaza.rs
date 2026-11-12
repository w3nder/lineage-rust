use killer_assets::client::Client;
use killer_assets::scene::props::section_material;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn test_client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

const PLAZA: [f32; 3] = [112_409.0, 220_670.0, -3594.0];
const PLAZA_RADIUS: f32 = 3000.0;

#[test]
#[ignore]
fn heine_plaza_static_meshes_all_become_draws() {
    survey_static_meshes(&["23_24", "22_24", "24_24", "23_23", "23_25"], PLAZA);
}

#[test]
#[ignore]
fn giran_static_meshes_all_become_draws() {
    survey_static_meshes(&["22_22", "21_22", "23_22", "22_21", "22_23"], [83400.0, 148000.0, -3400.0]);
}

fn survey_static_meshes(tiles: &[&str], plaza: [f32; 3]) {
    let Some(mut client) = test_client() else { return };
    let mut missing = Vec::new();
    for tile in tiles {
        let Ok(loaded) = client.load_level(tile) else { continue };
        let scene = killer_assets::scene::build_scene(&mut client, &loaded.level);
        let mut drawn: BTreeMap<[i32; 3], usize> = BTreeMap::new();
        for draw in scene.draws.iter().filter(|draw| draw.kind == killer_assets::scene::DrawKind::StaticMesh) {
            let sum = draw.vertices.iter().fold([0.0f64; 3], |acc, v| [acc[0] + f64::from(v.position[0]), acc[1] + f64::from(v.position[1]), acc[2] + f64::from(v.position[2])]);
            let n = draw.vertices.len().max(1) as f64;
            let key = [(sum[0] / n / 100.0) as i32, (sum[1] / n / 100.0) as i32, (sum[2] / n / 100.0) as i32];
            *drawn.entry(key).or_default() += 1;
        }
        let mut near = 0;
        let (mut mesh_missing, mut section_bad, mut material_unresolved, mut zero_scale) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for actor in &loaded.level.actors {
            let Some(path) = actor.static_mesh.as_deref() else { continue };
            let l = actor.placement.location;
            let d = ((l[0] - plaza[0]).powi(2) + (l[1] - plaza[1]).powi(2)).sqrt();
            let is_near = d <= PLAZA_RADIUS;
            if is_near {
                near += 1;
            }
            let label = format!("{tile} {} ({}) @ {:?} scale {:?} d {:.0}", actor.name, path, l, actor.scale, d);
            if actor.scale.contains(&0.0) {
                zero_scale.push(label.clone());
            }
            let Some(mesh) = client.catalog().static_mesh(path) else {
                mesh_missing.push(label.clone());
                continue;
            };
            for (index, section) in mesh.mesh.sections.iter().enumerate() {
                let first = section.first_index as usize;
                let count = section.index_count() as usize;
                if mesh.mesh.indices.get(first..first + count).is_none() {
                    section_bad.push(format!("{label} seção {index}: índices {first}..{} de {}", first + count, mesh.mesh.indices.len()));
                }
                let material = section_material(&actor.skins, &mesh.materials, index).unwrap_or_default();
                if client.catalog().material(&material).is_none() {
                    material_unresolved.push(format!("{label} seção {index}: material {material:?} sem resolução"));
                }
            }
        }
        eprintln!("{tile}: {} actors, {near} static meshes a menos de {PLAZA_RADIUS} u da praça, {} draws de static mesh", loaded.level.actors.len(), drawn.values().sum::<usize>());
        for (title, list) in [("malha ausente/ilegível", &mesh_missing), ("seção com índices inválidos", &section_bad), ("material sem resolução", &material_unresolved), ("escala zero", &zero_scale)] {
            eprintln!("  {title}: {}", list.len());
            for line in list.iter().take(60) {
                eprintln!("    {line}");
            }
        }
        missing.extend(mesh_missing);
        missing.extend(section_bad);
    }
    assert!(missing.is_empty(), "{} static meshes sem draw", missing.len());
}

#[test]
#[ignore]
fn heine_plaza_actors_by_name() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("23_24").unwrap();
    let mut rows: Vec<(f32, String)> = loaded
        .level
        .actors
        .iter()
        .filter(|actor| actor.static_mesh.is_some())
        .map(|actor| {
            let l = actor.placement.location;
            let d = ((l[0] - PLAZA[0]).powi(2) + (l[1] - PLAZA[1]).powi(2) + (l[2] - PLAZA[2]).powi(2)).sqrt();
            (d, format!("{:.0} {} {} {} @ {:?} rot {:?} scale {:?} skins {}", d, actor.class, actor.name, actor.static_mesh.as_deref().unwrap_or(""), l, actor.placement.rotation, actor.scale, actor.skins.len()))
        })
        .collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (_, row) in rows.iter().take(80) {
        eprintln!("{row}");
    }
}

#[test]
#[ignore]
fn every_static_mesh_of_the_143_tiles_loads() {
    let Some(mut client) = test_client() else { return };
    let mut failed: BTreeMap<String, usize> = BTreeMap::new();
    let mut checked: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut strips = 0usize;
    let mut sections = 0usize;
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            let Ok(loaded) = client.load_level(&tile) else { continue };
            for actor in &loaded.level.actors {
                let Some(path) = actor.static_mesh.as_deref() else { continue };
                let key = path.to_lowercase();
                if !checked.insert(key) {
                    continue;
                }
                match client.catalog().static_mesh(path) {
                    Some(mesh) => {
                        sections += mesh.mesh.sections.len();
                        strips += mesh.mesh.sections.iter().filter(|section| section.strip).count();
                    }
                    None => *failed.entry(format!("{tile} {path}")).or_default() += 1,
                }
            }
        }
    }
    eprintln!("{} malhas distintas, {sections} seções ({strips} strips), {} sem leitura", checked.len(), failed.len());
    for path in failed.keys() {
        eprintln!("  {path}");
    }
    assert!(failed.len() <= KNOWN_ABSENT_MESHES, "{failed:?}");
}

const KNOWN_ABSENT_MESHES: usize = 11;

#[test]
#[ignore]
fn static_mesh_lod_sway_and_billboard_census() {
    let Some(mut client) = test_client() else { return };
    let mut checked: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let (mut lod, mut blend, mut sway, mut billboard, mut lod_missing) = (0, 0, 0, 0, 0);
    let mut actors_lod = 0usize;
    let mut actors = 0usize;
    let mut ranges: BTreeMap<String, usize> = BTreeMap::new();
    let mut examples = Vec::new();
    for x in 15..=26 {
        for y in 10..=26 {
            let Ok(loaded) = client.load_level(&format!("{x}_{y}")) else { continue };
            for actor in &loaded.level.actors {
                let Some(path) = actor.static_mesh.as_deref() else { continue };
                actors += 1;
                let key = path.to_lowercase();
                let first = checked.insert(key);
                let Some(mesh) = client.catalog().static_mesh(path) else { continue };
                if mesh.mesh.lod.is_some() {
                    actors_lod += 1;
                }
                if !first {
                    continue;
                }
                if let Some(l) = mesh.mesh.lod {
                    lod += 1;
                    blend += usize::from(l.blend);
                    *ranges.entry(format!("{}/{}", l.range01, l.range02)).or_default() += 1;
                    if mesh.lod_meshes[0].is_none() || mesh.lod_meshes[1].is_none() {
                        lod_missing += 1;
                    }
                    if examples.len() < 5 {
                        examples.push(format!("{path}: {:?} r {}/{} blend {}", mesh.lod_meshes, l.range01, l.range02, l.blend));
                    }
                }
                if let Some(s) = mesh.mesh.sway {
                    sway += 1;
                    if sway <= 3 {
                        examples.push(format!("{path}: sway f {} angle {}", s.frequency, s.max_angle));
                    }
                }
                if mesh.mesh.billboard {
                    billboard += 1;
                    if billboard <= 3 {
                        examples.push(format!("{path}: billboard"));
                    }
                }
            }
        }
    }
    eprintln!("{} malhas distintas em {actors} actors: {lod} com LOD ({blend} com blend, {lod_missing} sem uma das malhas; {actors_lod} actors), {sway} com sway, {billboard} billboard", checked.len());
    eprintln!("faixas de LOD: {ranges:?}");
    for line in &examples {
        eprintln!("  {line}");
    }
    assert!(checked.len() > 5000);
}

#[test]
#[ignore]
fn tile_22_24_static_mesh_lighting_survey() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_24").unwrap();
    let zones = killer_assets::scene::zone_lighting(&loaded.level);
    let mut by_key: BTreeMap<String, usize> = BTreeMap::new();
    for actor in &loaded.level.actors {
        if actor.static_mesh.is_none() {
            continue;
        }
        let zone = loaded.level.bsp_tree.zone_at(actor.placement.location);
        let z = zones.get(usize::from(zone));
        let key = format!("zona {zone} (sun_affect da zona {:?}, ambiente {:?}) actor bSunAffect {} ColorStream {} unlit {}", z.map(|z| z.sun_affect), z.map(|z| z.ambient_brightness), actor.sun_affect, !actor.lighting.is_empty(), actor.unlit);
        *by_key.entry(key).or_default() += 1;
    }
    for (key, n) in &by_key {
        eprintln!("{n:6} {key}");
    }
    eprintln!("zonas: {}", zones.len());
    for (i, z) in zones.iter().enumerate() {
        eprintln!("  {i}: sun_affect {} ambient {} sky {} fog {:?}", z.sun_affect, z.ambient_brightness, z.is_sky, z.fog.map(|f| (f.start, f.end, f.color)));
    }
    let giran = client.load_level("22_22").unwrap();
    let mut giran_keys: BTreeMap<(bool, bool), usize> = BTreeMap::new();
    for actor in giran.level.actors.iter().filter(|a| a.static_mesh.is_some()) {
        *giran_keys.entry((actor.sun_affect, !actor.lighting.is_empty())).or_default() += 1;
    }
    eprintln!("Giran (bSunAffect, ColorStream): {giran_keys:?}");
}

#[test]
#[ignore]
fn giran_lod_levels_from_the_player_start() {
    use killer_assets::scene::props::lod_for;
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let start = loaded.level.actors.iter().find(|a| a.class.eq_ignore_ascii_case("PlayerStart")).unwrap().placement.location;
    let scene = killer_assets::scene::build_scene(&mut client, &loaded.level);
    let mut per_level = [0usize; 3];
    let mut culled = 0usize;
    let mut kept = 0usize;
    let mut lod_draws = 0usize;
    for draw in scene.draws.iter().filter(|d| d.kind == killer_assets::scene::DrawKind::StaticMesh) {
        let Some(actor) = draw.actor else { continue };
        let d = ((actor.xy[0] - start[0]).powi(2) + (actor.xy[1] - start[1]).powi(2)).sqrt();
        if !actor.ignored_range && d >= actor.range {
            culled += 1;
            continue;
        }
        match actor.lod {
            Some(lod) => {
                lod_draws += 1;
                if lod_for(d, lod.ranges[0], lod.ranges[1]) == lod.level {
                    per_level[usize::from(lod.level)] += 1;
                    kept += 1;
                }
            }
            None => kept += 1,
        }
    }
    eprintln!("Giran do PlayerStart: {kept} draws de static mesh visíveis pela regra, {culled} cortados por distância, {lod_draws} draws de LOD dos quais por nível {per_level:?}");
    assert!(culled > 0 && per_level.iter().sum::<usize>() > 0);
}

#[test]
#[ignore]
fn kruma_tower_materials_survey() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("20_21").unwrap();
    let center = [5897.0f32, 104907.0, -1361.0];
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for actor in &loaded.level.actors {
        let Some(path) = actor.static_mesh.as_deref() else { continue };
        let l = actor.placement.location;
        let d = ((l[0] - center[0]).powi(2) + (l[1] - center[1]).powi(2)).sqrt();
        if d > 2500.0 {
            continue;
        }
        let Some(mesh) = client.catalog().static_mesh(path) else { continue };
        for (i, _) in mesh.mesh.sections.iter().enumerate() {
            let material = section_material(&actor.skins, &mesh.materials, i).unwrap_or_default();
            let resolved = client.catalog().material(&material);
            let key = format!("{path} sec {i}: {material} -> {} unlit {} sun {} colorstream {}", resolved.map_or("?".to_owned(), |m| m.summary()), actor.unlit, actor.sun_affect, !actor.lighting.is_empty());
            *seen.entry(key).or_default() += 1;
        }
    }
    for (key, n) in seen.iter().take(40) {
        eprintln!("{n:3} {key}");
    }
}

#[test]
#[ignore]
fn kruma_far_vertices_survey() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("20_21").unwrap();
    let scene = killer_assets::scene::build_scene(&mut client, &loaded.level);
    let center = [5897.0f32, 104907.0];
    let mut rows = Vec::new();
    for draw in scene.draws.iter().filter(|d| d.kind == killer_assets::scene::DrawKind::StaticMesh) {
        let Some(actor) = draw.actor else { continue };
        let d = ((actor.xy[0] - center[0]).powi(2) + (actor.xy[1] - center[1]).powi(2)).sqrt();
        if d > 2500.0 {
            continue;
        }
        let far = draw.vertices.iter().filter(|v| ((v.position[0] - center[0]).powi(2) + (v.position[1] - center[1]).powi(2)).sqrt() > 10000.0 || v.position[2].abs() > 50000.0).count();
        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];
        for v in draw.world_vertices() {
            for a in 0..3 {
                min[a] = min[a].min(v.position[a]);
                max[a] = max[a].max(v.position[a]);
            }
        }
        if far > 0 || max[2] - min[2] > 8000.0 {
            rows.push(format!("actor {:?} lod {:?} {} vértices, {far} longe, caixa {:?}..{:?} material {}", actor.xy, actor.lod.map(|l| l.level), draw.vertices.len(), min, max, draw.material.first_texture().map(|t| t.path.clone()).unwrap_or_default()));
        }
    }
    eprintln!("{} draws suspeitos", rows.len());
    for row in rows.iter().take(20) {
        eprintln!("  {row}");
    }
    let actors: Vec<_> = loaded.level.actors.iter().filter(|a| a.static_mesh.is_some() && ((a.placement.location[0] - center[0]).powi(2) + (a.placement.location[1] - center[1]).powi(2)).sqrt() < 2500.0).collect();
    for a in actors.iter().take(40) {
        eprintln!("  {} {} @ {:?} scale {:?} rot {:?}", a.class, a.static_mesh.as_deref().unwrap_or(""), a.placement.location, a.scale, a.placement.rotation);
    }
}

#[test]
#[ignore]
fn kruma_white_pixels_survey() {
    use glam::{Vec3, Vec4};
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("20_21").unwrap();
    let scene = killer_assets::scene::build_scene(&mut client, &loaded.level);
    let eye = Vec3::new(5897.0, 104907.0, -1100.0);
    let target = Vec3::new(9000.0, 107000.0, -1300.0);
    let view = glam::camera::lh::view::look_at_mat4(eye, target, Vec3::Z);
    let fov_y = 2.0 * ((60f32.to_radians() / 2.0).tan() / (1280.0 / 720.0)).atan();
    let proj = glam::camera::lh::proj::directx::perspective(fov_y, 1280.0 / 720.0, 10.0, 60000.0);
    let vp = proj * view;
    let mut hits: BTreeMap<String, (usize, f32, f32)> = BTreeMap::new();
    for draw in &scene.draws {
        for v in draw.world_vertices() {
            let clip = vp * Vec4::new(v.position[0], v.position[1], v.position[2], 1.0);
            if clip.w <= 0.0 {
                continue;
            }
            let px = (clip.x / clip.w + 1.0) * 0.5 * 1280.0;
            let py = (1.0 - clip.y / clip.w) * 0.5 * 720.0;
            if (690.0..750.0).contains(&px) && (150.0..250.0).contains(&py) {
                let depth = (view * Vec4::new(v.position[0], v.position[1], v.position[2], 1.0)).z;
                let key = format!("{:?} zona {} sky {} {}", draw.kind, draw.zone, scene.zone(draw.zone).is_sky, draw.material.first_texture().map(|t| t.path.clone()).unwrap_or_default());
                let e = hits.entry(key).or_insert((0, f32::MAX, f32::MIN));
                e.0 += 1;
                e.1 = e.1.min(depth);
                e.2 = e.2.max(depth);
            }
        }
    }
    for (key, (n, near, far)) in &hits {
        eprintln!("{n:6} profundidade {near:.0}..{far:.0} {key}");
    }
}

#[test]
#[ignore]
fn giran_stairs_survey() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let start = loaded.level.actors.iter().find(|a| a.class.eq_ignore_ascii_case("PlayerStart")).unwrap().placement.location;
    let mut rows = Vec::new();
    for actor in &loaded.level.actors {
        let Some(path) = actor.static_mesh.as_deref() else { continue };
        let lower = path.to_lowercase();
        if !(lower.contains("stair") || lower.contains("step") || lower.contains("stiar") || lower.contains("stare")) {
            continue;
        }
        let l = actor.placement.location;
        let d = ((l[0] - start[0]).powi(2) + (l[1] - start[1]).powi(2)).sqrt();
        let Some(mesh) = client.catalog().static_mesh(path) else { continue };
        let mut mats = Vec::new();
        for (i, _) in mesh.mesh.sections.iter().enumerate() {
            let material = section_material(&actor.skins, &mesh.materials, i).unwrap_or_default();
            let resolved = client.catalog().material(&material);
            mats.push(format!("{material} [{}]", resolved.map_or("?".to_owned(), |m| format!("{} two_sided={:?}", m.root_class, m.passes.iter().map(|p| p.two_sided).collect::<Vec<_>>()))));
        }
        rows.push((d, format!("{d:.0} {} {path} @ {:?} rot {:?} lod {:?}: {}", actor.name, l, actor.placement.rotation, mesh.mesh.lod.is_some(), mats.join(" | "))));
    }
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (_, row) in rows.iter().take(30) {
        eprintln!("{row}");
    }
    eprintln!("PlayerStart {start:?}");
}

#[test]
#[ignore]
fn giran_plaza_stair_faces() {
    use glam::Vec3;
    let Some(mut client) = test_client() else { return };
    for path in ["Giran_Village_S.Giran_V_Plaza_Stair01", "Giran_Village_S.Giran_V_Stair", "Giran_Village_S.Giran_V_LxShop_Stair"] {
        let mesh = client.catalog().static_mesh(path).unwrap();
        let m = &mesh.mesh;
        let (mut along, mut against, mut down_along, mut down_against, mut up_along, mut up_against) = (0, 0, 0, 0, 0, 0);
        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];
        for p in &m.positions {
            for a in 0..3 {
                min[a] = min[a].min(p[a]);
                max[a] = max[a].max(p[a]);
            }
        }
        for tri in m.indices.as_chunks::<3>().0 {
            let [a, b, c] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
            let pa = Vec3::from(m.positions[a]);
            let pb = Vec3::from(m.positions[b]);
            let pc = Vec3::from(m.positions[c]);
            let cross = (pb - pa).cross(pc - pa).normalize_or_zero();
            let stored = (Vec3::from(m.normals[a]) + Vec3::from(m.normals[b]) + Vec3::from(m.normals[c])).normalize_or_zero();
            let agrees = cross.dot(stored) >= 0.0;
            if agrees { along += 1 } else { against += 1 }
            if cross.z < -0.5 {
                if agrees { down_along += 1 } else { down_against += 1 }
            }
            if cross.z > 0.5 {
                if agrees { up_along += 1 } else { up_against += 1 }
            }
        }
        eprintln!("{path}: {} triângulos, cross a favor da normal {along} / contra {against}; faces com cross para baixo: a favor {down_along} contra {down_against}; para cima: a favor {up_along} contra {up_against}; caixa {min:?}..{max:?}", m.indices.len() / 3);
        let mut down_normals = 0;
        for n in &m.normals {
            if n[2] < -0.5 {
                down_normals += 1;
            }
        }
        eprintln!("  vértices com normal para baixo: {down_normals} de {}", m.normals.len());
    }
}

#[test]
#[ignore]
fn giran_two_sided_census() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let scene = killer_assets::scene::build_scene(&mut client, &loaded.level);
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for draw in &scene.draws {
        let key = format!("{:?} two_sided={} root={}", draw.kind, draw.material.passes.first().is_some_and(|p| p.two_sided), draw.material.root_class);
        *counts.entry(key).or_default() += 1;
    }
    for (k, n) in &counts {
        eprintln!("{n:6} {k}");
    }
}

#[test]
#[ignore]
fn giran_plaza_stair_world_winding() {
    use glam::Vec3;
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let scene = killer_assets::scene::build_scene(&mut client, &loaded.level);
    let stair = [83172.0f32, 148618.0];
    let mut along = 0;
    let mut against = 0;
    for draw in scene.draws.iter().filter(|d| d.actor.is_some_and(|a| a.xy == stair)) {
        for tri in draw.indices.as_chunks::<3>().0 {
            let v = |i: u32| &draw.vertices[i as usize];
            let (a, b, c) = (v(tri[0]), v(tri[1]), v(tri[2]));
            let cross = (Vec3::from(b.position) - Vec3::from(a.position)).cross(Vec3::from(c.position) - Vec3::from(a.position));
            let stored = Vec3::from(a.normal) + Vec3::from(b.normal) + Vec3::from(c.normal);
            if cross.dot(stored) > 0.0 { along += 1 } else { against += 1 }
        }
        eprintln!("draw {} vértices {} índices material {}", draw.vertices.len(), draw.indices.len(), draw.material.first_texture().map(|t| t.path.clone()).unwrap_or_default());
    }
    eprintln!("escada no mundo: a favor {along}, contra {against}");
    assert_eq!((along, against), (0, 842), "a escada espelhada (DrawScale3D.X = -1) tem de enrolar contra as normais como as outras malhas");
    let actor = loaded.level.actors.iter().find(|a| a.placement.location[0] == 83172.0 && a.placement.location[1] == 148618.0).unwrap();
    eprintln!("actor {:?} scale {:?} rot {:?} hidden {} static {}", actor.name, actor.scale, actor.placement.rotation, actor.hidden, actor.is_static);
    let m = killer_assets::scene::transform::placement_matrix(actor.placement.location, actor.placement.rotation, actor.scale);
    eprintln!("det {}", m.determinant());
}
