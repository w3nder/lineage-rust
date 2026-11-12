use killer_assets::scene::projectors::calc_matrix;
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};
use ue2_level::{Placement, projectors::Projector};

#[test]
#[ignore]
fn bsp_projector_materials_cover_every_clipped_receiver() {
    use glam::Vec3;
    use killer_assets::scene::projectors::clip_bsp_polygon;
    let mut client = killer_assets::client::Client::open(std::path::Path::new(
        &std::env::var("KILLER_CLIENT_DIR").unwrap(),
    )).unwrap();
    let mut counts = std::collections::BTreeMap::new();
    for tile in ["16_10", "20_17", "24_20"] {
        let loaded = client.load_level(tile).unwrap();
        let mut receivers = 0;
        for p in &loaded.level.projectors {
            let path = p.texture.as_deref().unwrap();
            let material = client.catalog().material(path).unwrap();
            let projection = calc_matrix(p, material.first_texture().unwrap().size.map(|n| n as u32), None);
            for polygon in &loaded.level.bsp {
                let points: Vec<_> = polygon.corners.iter().copied().map(Vec3::from).collect();
                if clip_bsp_polygon(&points, &projection.planes, true).is_empty() { continue; }
                receivers += 1;
                let node = loaded.level.bsp_render.node_draws[polygon.node_index];
                let section = &loaded.level.bsp_render.sections[usize::try_from(node.section).unwrap()];
                let material = client.catalog().bsp_projector_material(path, section.material.as_deref(),
                    p.material_blend, p.frame_blend, projection.matrix.to_cols_array_2d())
                    .unwrap_or_else(|| panic!("{tile}: projected={path}; base={:?}", polygon.material));
                assert_eq!(material.passes.len(), 1);
                assert!(material.stages.len() <= 4, "{tile}: {material:?}");
                assert!(!material.passes[0].lit);
                assert!(!material.passes[0].z_write);
                let base_class = section.material.as_deref().and_then(|path| client.catalog().object_class(path));
                *counts.entry((tile, material.stages.len(), base_class)).or_insert(0) += 1;
            }
        }
        assert_eq!(receivers, match tile { "16_10" => 77, "20_17" => 28, _ => 206 });
    }
    eprintln!("BSP receiver material census: {counts:?}");
}

#[test]
#[ignore]
fn dynamic_bsp_projector_stream_106279c0_matches_native() {
    use glam::Vec3;
    use killer_assets::scene::projectors::{bsp_fan_indices, bsp_vertex_stream, clip_bsp_polygon};
    use ue2_level::projectors::{GRADIENT, ON_BACKFACES};
    let mut cases = Vec::new();
    for rotation in [[0, 0, 0], [16384, 0, 0], [-7167, 3938, 879], [13413, -1839, 1024]] {
        for flags in [0, GRADIENT, ON_BACKFACES, GRADIENT | ON_BACKFACES] {
            for normal in [Vec3::X, -Vec3::X, Vec3::Z, Vec3::new(0.3, 0.4, -0.5)] {
                for (time, expires) in [(0.0, 0.0), (2.0, 1.0), (1.5, 2.0), (1.0, 2.0), (1.0, 3.0)] {
                    for base_material in [false, true] {
                        for gradient_texture in [None, Some("Engine.GRADIENT_Fade".into())] {
                            let p = Projector {
                                flags,
                                fov: 40,
                                placement: Placement { rotation, location: [120000.0, -98000.0, 50.0] },
                                gradient_texture,
                                ..Default::default()
                            };
                            let projection = calc_matrix(&p, [128, 256], None);
                            let points: Vec<_> = [-1.0, 0.0, 0.25, 0.5, 0.99999, 1.0, 2.0]
                                .into_iter()
                                .map(|d| Vec3::from(p.placement.location) + Vec3::new(300.0, -400.0, 500.0) * d)
                                .collect();
                            cases.push((p, projection, points, normal, base_material, time, expires));
                        }
                    }
                }
            }
        }
    }
    let mut client =
        killer_assets::client::Client::open(std::path::Path::new(&std::env::var("KILLER_CLIENT_DIR").unwrap()))
            .unwrap();
    for tile in ["16_10", "20_17", "24_20"] {
        let loaded = client.load_level(tile).unwrap();
        for p in &loaded.level.projectors {
            let material = client.catalog().material(p.texture.as_deref().unwrap()).unwrap();
            let projection = calc_matrix(p, material.first_texture().unwrap().size.map(|v| v as u32), None);
            for polygon in &loaded.level.bsp {
                let points: Vec<_> = polygon.corners.iter().copied().map(Vec3::from).collect();
                let points: Vec<_> = clip_bsp_polygon(&points, &projection.planes, true).into_iter().rev().collect();
                if points.is_empty() {
                    continue;
                }
                for base_material in [false, true] {
                    cases.push((
                        p.clone(),
                        projection.clone(),
                        points.clone(),
                        Vec3::from(polygon.normal),
                        base_material,
                        0.0,
                        0.0,
                    ));
                }
            }
        }
    }
    let input: Vec<_> = cases.iter().map(|(p, projection, points, normal, base_material, time, expires)| json!({"bsp_stream": {
        "location": p.placement.location, "rotation": p.placement.rotation,
        "distance": p.max_trace_distance, "matrix": projection.matrix.to_cols_array(),
        "flags": u32::from(p.flags & ON_BACKFACES != 0) * 16 + u32::from(p.flags & GRADIENT != 0 && p.gradient_texture.is_some()) * 4,
        "normal": normal.to_array(), "time":time, "expires":expires, "base_material":base_material,
        "points": points.iter().map(|p| p.to_array()).collect::<Vec<_>>(), "index_offset": 65533,
    }})).collect();
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/projector_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script)
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(input)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(cases.len(), expected.len());
    for (i, ((p, projection, points, normal, base_material, time, expires), native)) in
        cases.iter().zip(&expected).enumerate()
    {
        let vertices = bsp_vertex_stream(p, projection, points, *normal, *base_material, *time, *expires);
        assert_eq!(json!(bsp_fan_indices(points.len(), 65533)), native["indices"], "caso {i}");
        for (v, native) in vertices.iter().zip(native["vertices"].as_array().unwrap()) {
            assert_eq!(json!(v.position), native["position"], "caso {i}");
            assert_eq!(json!(v.color), native["color"], "caso {i}");
            assert_eq!(json!(v.base_uv), native["base_uv"], "caso {i}");
            for (a, b) in v.uvz.iter().zip(native["uvz"].as_array().unwrap()) {
                let b = b.as_f64().unwrap() as f32;
                assert!((a - b).abs() <= 0.00001 + b.abs() * f32::EPSILON * 2.0, "caso {i}: {a} != {b}");
            }
        }
    }
    eprintln!("Stream BSP: {} casos, {} vértices", cases.len(), cases.iter().map(|c| c.2.len()).sum::<usize>());
}

#[test]
#[ignore]
fn fpoly_split_1054c690_matches_native_bsp_polygons() {
    use glam::{Vec3, Vec4};
    use killer_assets::{client::Client, scene::projectors::clip_bsp_polygon};
    let mut cases = Vec::new();
    for count in [3, 4, 8, 13, 14, 15, 16] {
        for offset in [-100000.0f32, 0.0, 100000.0] {
            for distance in [-2.0, -0.25001, -0.25, 0.0, 0.25, 0.25001, 2.0] {
                for no_overflow in [false, true] {
                    let points: Vec<_> = (0..count)
                        .map(|i| {
                            let angle = i as f32 * std::f32::consts::TAU / count as f32;
                            Vec3::new(offset + angle.cos(), angle.sin(), 1.0)
                        })
                        .collect();
                    let planes = vec![Vec4::new(1.0, 0.0, 0.0, offset + distance)];
                    cases.push((points, planes, no_overflow));
                }
            }
        }
    }
    let mut client = Client::open(std::path::Path::new(&std::env::var("KILLER_CLIENT_DIR").unwrap())).unwrap();
    for tile in ["16_10", "20_17", "24_20"] {
        let loaded = client.load_level(tile).unwrap();
        let mut receivers = 0;
        for projector in &loaded.level.projectors {
            let material = client.catalog().material(projector.texture.as_deref().unwrap()).unwrap();
            let size = material.first_texture().unwrap().size.map(|v| v as u32);
            let projection = calc_matrix(projector, size, None);
            for polygon in &loaded.level.bsp {
                assert!(polygon.corners.len() <= 16);
                let points: Vec<_> = polygon.corners.iter().copied().map(Vec3::from).collect();
                receivers += usize::from(!clip_bsp_polygon(&points, &projection.planes, true).is_empty());
                cases.push((
                    polygon.corners.iter().copied().map(Vec3::from).collect(),
                    projection.planes.to_vec(),
                    true,
                ));
            }
        }
        eprintln!("{tile}: {receivers} interseções BSP antes dos filtros de material");
    }
    let input: Vec<Value> = cases
        .iter()
        .map(|(points, planes, no_overflow)| {
            json!({"bsp_clip": {
                "points": points.iter().map(|p| p.to_array()).collect::<Vec<_>>(),
                "planes": planes.iter().map(|p| p.to_array()).collect::<Vec<_>>(),
                "no_overflow": no_overflow,
            }})
        })
        .collect();
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/projector_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script)
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(input)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<Vec<[f32; 3]>> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(cases.len(), expected.len());
    let mut receivers = 0;
    for (index, ((points, planes, no_overflow), native)) in cases.iter().zip(&expected).enumerate() {
        let actual = clip_bsp_polygon(points, planes, *no_overflow);
        assert_eq!(actual.len(), native.len(), "caso {index}");
        receivers += usize::from(!native.is_empty());
        for (actual, native) in actual.iter().zip(native) {
            for (a, b) in actual.to_array().into_iter().zip(native) {
                assert!((a - b).abs() <= 0.000002 + b.abs() * f32::EPSILON * 2.0, "caso {index}: {a} != {b}");
            }
        }
    }
    eprintln!("FPoly::Split: {} polígonos comparados, {receivers} receptores", cases.len());
}

#[test]
#[ignore]
fn projector_outcode_106cf820_matches_native_at_boundaries() {
    use glam::{Vec3, Vec4};
    let mut projection = calc_matrix(&Projector::default(), [128, 256], None);
    projection.planes = [
        Vec4::new(1.0, 0.0, 0.0, -100.0),
        Vec4::new(-1.0, 0.0, 0.0, -100.0),
        Vec4::new(0.0, 1.0, 0.0, -100.0),
        Vec4::new(0.0, -1.0, 0.0, -100.0),
        Vec4::new(0.0, 0.0, 1.0, -100.0),
        Vec4::new(0.0, 0.0, -1.0, -100.0),
    ];
    let points: Vec<[f32; 3]> = [-101.0, -100.0, -99.99999, 0.0, 99.99999, 100.0, 101.0]
        .into_iter()
        .flat_map(|x| {
            [-101.0, -100.0, 0.0, 100.0, 101.0]
                .into_iter()
                .flat_map(move |y| [-101.0, -100.0, 0.0, 100.0, 101.0].map(|z| [x, y, z]))
        })
        .collect();
    let mut tilted = projection.clone();
    tilted.planes[0] = Vec4::new(0.6, -0.8, 0.0, 120000.0);
    let far_points = vec![[100000.0, -75000.0, 0.0], [100000.01, -75000.0, 0.0], [99999.99, -75000.0, 0.0]];
    let cases = [(projection, points), (tilted, far_points)];
    let input: Vec<Value> =
        cases.iter().map(|(p, points)| json!({"planes":p.planes.map(|p|p.to_array()), "points":points})).collect();
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/projector_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script)
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(input)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<Vec<u8>> = serde_json::from_slice(&output.stdout).unwrap();
    for ((p, points), native) in cases.into_iter().zip(expected) {
        assert_eq!(points.into_iter().map(|v| p.outcode(Vec3::from(v))).collect::<Vec<_>>(), native);
    }
}

#[test]
#[ignore]
fn projector_facing_cube_1000c9a0_matches_all_native_texels() {
    use killer_assets::scene::projectors::facing_cube;
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/projector_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script)
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!([{ "cube": 32 }])).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let results: Vec<Vec<String>> = serde_json::from_slice(&output.stdout).unwrap();
    for (face, (actual, expected)) in facing_cube(32).iter().zip(&results[0]).enumerate() {
        let expected: Vec<u8> = expected
            .as_bytes()
            .as_chunks::<2>().0.iter()
            .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
            .collect();
        assert_eq!(actual, &expected, "face {face}");
    }
}

#[test]
#[ignore]
fn projector_properties_across_all_143_tiles() {
    use killer_assets::client::Client;
    use std::{collections::BTreeMap, path::PathBuf};
    let mut client = Client::open(&PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").unwrap())).unwrap();
    let mut counts = BTreeMap::new();
    let mut configurations = BTreeMap::new();
    let mut maps = 0;
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            if client.level_path(&tile).is_err() {
                continue;
            }
            maps += 1;
            let loaded = client.load_level(&tile).unwrap();
            for terrain in &loaded.level.terrains {
                let grid = killer_assets::scene::terrain::load_grid(&mut client, terrain)
                    .unwrap_or_else(|| panic!("{tile}: heightmap {}", terrain.heightmap));
                assert_eq!(grid.heightmap, [256, 256], "{tile}");
                let normals = killer_assets::scene::terrain_normals::face_buffer(terrain, &grid).unwrap();
                assert!(normals.len() >= 256 * 256);
                if !terrain.native.face_normals.is_empty() {
                    assert_eq!(normals, terrain.native.face_normals, "{tile}: normais serializadas");
                }
            }
            assert_eq!(
                loaded.level.projectors.len(),
                loaded
                    .level
                    .actors
                    .iter()
                    .filter(|a| matches!(a.class.as_str(), "Projector" | "DynamicProjector"))
                    .count(),
                "{tile}"
            );
            if !loaded.level.projectors.is_empty() {
                let (draws, states) = killer_assets::scene::projectors::build_terrain_with_states(
                    &mut client,
                    &loaded.level,
                    Default::default(),
                );
                assert_eq!(states.len(), loaded.level.projectors.len());
                assert_eq!(loaded.level.detail_mode, 2, "{tile}");
                assert!(states.iter().all(|s| s.visibility == Default::default()), "{tile}: {states:?}");
                assert!(
                    killer_assets::scene::projectors::visible_projectors(&states, None, false, false)
                        .iter()
                        .all(|&v| v)
                );
                let mut indices: Vec<_> = states.iter().flat_map(|s| s.draws.iter().copied()).collect();
                indices.sort_unstable();
                assert_eq!(indices, (0..draws.len()).collect::<Vec<_>>());
                for draw in &draws {
                    assert!(draw.material.stages.len() <= 3, "{tile}: {:?}", draw.material);
                    assert!(draw.vertices.iter().all(|v| v.position.iter().chain(&v.normal).all(|f| f.is_finite())));
                    assert!(draw.indices.iter().all(|i| (*i as usize) < draw.vertices.len()));
                }
                eprintln!(
                    "{tile}: {} passes de projetor, {} triângulos",
                    draws.len(),
                    draws.iter().map(|d| d.indices.len() / 3).sum::<usize>()
                );
            }
            for p in &loaded.level.projectors {
                *counts.entry(tile.clone()).or_insert(0usize) += 1;
                *configurations
                    .entry((p.texture.clone(), p.flags, p.material_blend, p.frame_blend, p.fov, p.max_trace_distance))
                    .or_insert(0usize) += 1;
                assert!(p.draw_scale.is_finite() && p.draw_scale > 0.0);
                if let Some(path) = &p.texture {
                    let material =
                        client.catalog().material(path).unwrap_or_else(|| panic!("{tile}/{}: {path}", p.name));
                    assert!(!material.stages.is_empty(), "{path}");
                }
            }
        }
    }
    eprintln!("{maps} maps; projectors={counts:?}; configurations={configurations:#?}");
    assert_eq!(maps, 143);
    assert_eq!(counts.values().sum::<usize>(), 55);
}

#[test]
#[ignore]
fn calc_matrix_1060c540_matches_original_engine_and_core() {
    let mut cases = Vec::new();
    for rotation in [[0, 0, 0], [0, 16384, 0], [-16384, 0, 0], [1403, -3245, 6589]] {
        for fov in [0, 1, 6, 45, 90] {
            for physics in [0, 5] {
                cases.push(json!({"location":[-800.0,450.0,95.0], "rotation":rotation, "size":[128,256], "scale":4.0, "radius":-1.0, "fov":fov, "distance":6000, "physics":physics, "gradient_size":256}));
            }
        }
    }
    cases.push(json!({"location":[10.0,20.0,30.0],"rotation":[-1989,45790,-45786],"size":[64,128],"scale":0.1,"radius":30.0,"fov":6,"distance":1000,"physics":0,"gradient_size":128}));
    let mut client =
        killer_assets::client::Client::open(std::path::Path::new(&std::env::var("KILLER_CLIENT_DIR").unwrap()))
            .unwrap();
    for tile in ["16_10", "20_17", "24_20"] {
        let loaded = client.load_level(tile).unwrap();
        for p in &loaded.level.projectors {
            let material = client.catalog().material(p.texture.as_deref().unwrap()).unwrap();
            let size = material.first_texture().unwrap().size.map(|v| v as u32);
            assert!(size.iter().all(|v| *v > 0));
            cases.push(json!({"location":p.placement.location,"rotation":p.placement.rotation,"size":size,"scale":p.draw_scale,"radius":p.radius,"fov":p.fov,"distance":p.max_trace_distance,"physics":p.physics}));
        }
    }
    assert_eq!(cases.len(), 96);
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/projector_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script)
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let results: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(results.len(), cases.len());
    for (case, native) in cases.iter().zip(results) {
        let p = Projector {
            placement: Placement {
                location: serde_json::from_value(case["location"].clone()).unwrap(),
                rotation: serde_json::from_value(case["rotation"].clone()).unwrap(),
            },
            fov: case["fov"].as_i64().unwrap() as i32,
            physics: case["physics"].as_u64().unwrap() as u8,
            draw_scale: case["scale"].as_f64().unwrap() as f32,
            radius: case["radius"].as_f64().unwrap() as f32,
            max_trace_distance: case["distance"].as_i64().unwrap() as i32,
            ..Default::default()
        };
        let result = calc_matrix(
            &p,
            serde_json::from_value(case["size"].clone()).unwrap(),
            case["gradient_size"].as_i64().map(|n| n as i32),
        );
        for (name, actual) in [
            ("matrix", result.matrix.to_cols_array().to_vec()),
            ("gradient", result.gradient.to_cols_array().to_vec()),
            ("vertices", result.vertices.iter().flat_map(|v| v.to_array()).collect()),
            ("planes", result.planes.iter().flat_map(|v| v.to_array()).collect()),
            ("bounds", result.min.to_array().into_iter().chain(result.max.to_array()).collect()),
        ] {
            let expected: Vec<f64> = native[name]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|v| match v.as_array() {
                    Some(values) => values.iter().map(|v| v.as_f64().unwrap()).collect(),
                    None => vec![v.as_f64().unwrap()],
                })
                .collect();
            for (index, (a, b)) in actual.iter().zip(expected).enumerate() {
                let translation_error =
                    p.placement.location.into_iter().map(f32::abs).fold(0.0, f32::max) * f32::EPSILON * 16.0;
                let tolerance = if name == "planes" && index % 4 == 3 {
                    0.003 + f64::from(translation_error)
                } else {
                    0.003 + b.abs() * 0.000003
                };
                assert!((f64::from(*a) - b).abs() < tolerance, "{name}[{index}]: {a} != {b}; {case}");
            }
        }
    }
}
