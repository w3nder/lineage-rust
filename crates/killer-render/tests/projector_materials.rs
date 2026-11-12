use glam::Vec3;
use killer_assets::{
    client::Client,
    scene::{DrawKind, MeshDraw, Pass, Scene, Vertex, projectors::{calc_matrix, clip_bsp_polygon}},
    textures::{TextureData, TextureFormatKind, TextureSource},
};
use killer_render::{camera::Camera, environment::Environment, headless::{CLEAR_RGBA, Headless}};
use l2_catalog::{Material, StageSource};
use serde_json::{Value, json};
use std::{collections::HashMap, io::Write, path::Path, process::{Command, Stdio}, sync::Arc};

fn sample(name: &str) -> [f32; 4] {
    let bytes = match name {
        "projected" => [80, 140, 200, 160],
        "base" => [60, 90, 120, 220],
        "opacity" => [70, 110, 150, 128],
        "gradient" => [180, 90, 40, 96],
        "facing" => [254; 4],
        "none" => [255; 4],
        _ => panic!("unknown native texture: {name}"),
    };
    bytes.map(|v| v as f32 / 255.0)
}

#[test]
#[ignore]
fn dynamic_projector_10005f20_material_pixels_match_original_driver() {
    use l2_catalog::DynamicProjectorParameters;
    let root = std::env::var("KILLER_CLIENT_DIR").unwrap();
    let mut client = Client::open(Path::new(&root)).unwrap();
    let loaded = client.load_level("16_10").unwrap();
    let projected = loaded.level.projectors[0].texture.as_deref().unwrap();
    let gradient = "Engine.GRADIENT_Fade";
    assert!(client.catalog().material(gradient).unwrap().first_texture().is_some());
    let mut bases = HashMap::new();
    for section in &loaded.level.bsp_render.sections {
        if let Some(path) = &section.material {
            let class = client.catalog().object_class(path).unwrap();
            if matches!(class.as_str(), "Texture" | "Shader") {
                bases.entry(class).or_insert_with(|| path.clone());
            }
        }
    }
    assert_eq!(bases.len(), 2);
    let mut cases = Vec::new();
    let mut materials = Vec::new();
    let projection = [[0.0; 4], [0.0; 4], [0.0; 4], [0.5, 0.5, 1.0, 1.0]];
    let normal = [[0.0; 4], [0.0, 0.0, -1.0, 0.0], [0.0; 4], [0.0; 4]];
    for (class, base) in bases {
        for material_blend in 0..3 {
            for frame_blend in 0..4 {
                for flags in 0..16u8 {
                    let perspective = flags & 1 != 0;
                    let with_gradient = flags & 2 != 0;
                    let on_alpha = flags & 4 != 0;
                    let on_backfaces = flags & 8 != 0;
                    cases.push(json!({"static":false,"base":if class == "Shader" {"shader_opacity"} else {"texture"},
                        "material_blend":material_blend,"frame_blend":frame_blend,"gradient":with_gradient,
                        "flags":u8::from(perspective) | (u8::from(with_gradient) << 2) | (u8::from(on_alpha) << 3) | (u8::from(on_backfaces) << 4)}));
                    materials.push(client.catalog().dynamic_projector_material(projected, Some(&base), &DynamicProjectorParameters {
                        material_blend, frame_blend, projected: perspective, on_alpha, on_backfaces,
                        projection, normal, gradient: with_gradient.then(|| (gradient.into(), projection)),
                    }).unwrap());
                }
            }
        }
    }
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/projector_material_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script).arg(root).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let native: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(native.len(), 384);
    let mut gpu = Headless::new(32, 32).unwrap();
    let camera = Camera { eye: Vec3::new(0.0, -300.0, 0.0), target: Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    for (index, (material, original)) in materials.into_iter().zip(native).enumerate() {
        assert_eq!(original["success"], 1, "{}", cases[index]);
        let stages = original["stages"].as_array().unwrap();
        assert_eq!(material.stages.len(), stages.len(), "{}", cases[index]);
        let mut textures = Colors(HashMap::new());
        for (stage, native) in material.stages.iter().zip(stages) {
            if let StageSource::Texture(texture) = &stage.source {
                textures.0.insert(texture.path.clone(), sample(native["texture"].as_str().unwrap()));
            }
        }
        gpu.upload(&quad(material, 1.0), &mut textures);
        let pixels = gpu.render(&camera, &Environment::default()).unwrap();
        let actual = &pixels[(16 * 32 + 16) * 4..][..3];
        let expected = original_pixel(&original, 1.0);
        assert!(actual.iter().zip(expected).all(|(&a, b)| a.abs_diff(b) <= 2), "case {index} {}: {actual:?} != {expected:?}", cases[index]);
    }
}

fn argument(id: u64, texture: [f32; 4], current: [f32; 4], diffuse: [f32; 4], factor: [f32; 4]) -> [f32; 4] {
    let value = match id & 15 {
        0 => diffuse,
        1 => current,
        2 => texture,
        3 => factor,
        _ => panic!("unknown native argument: {id}"),
    };
    if id & 0x20 != 0 { [value[3]; 4] } else { value }
}

fn operation(op: u64, a: f32, b: f32, texture: [f32; 4], diffuse: [f32; 4]) -> f32 {
    match op {
        2 => a,
        3 => b,
        4 => a * b,
        5 => 2.0 * a * b,
        12 => a * diffuse[3] + b * (1.0 - diffuse[3]),
        13 => a * texture[3] + b * (1.0 - texture[3]),
        _ => panic!("unknown native operation: {op}"),
    }.clamp(0.0, 1.0)
}

fn original_pixel(native: &Value, attenuation: f32) -> [u8; 3] {
    let diffuse = [attenuation; 4];
    let mut current = diffuse;
    let packed = native["factor"].as_u64().unwrap();
    let factor = [16, 8, 0, 24].map(|shift| ((packed >> shift) & 255) as f32 / 255.0);
    for stage in native["stages"].as_array().unwrap() {
        let texture = sample(stage["texture"].as_str().unwrap());
        let args = |key: &str| [0, 1].map(|i| argument(stage[key][i].as_u64().unwrap(), texture, current, diffuse, factor));
        let [a, b] = args("color_args");
        let [aa, ab] = args("alpha_args");
        let rgb: [f32; 3] = std::array::from_fn(|i| operation(stage["color_op"].as_u64().unwrap(), a[i], b[i], texture, diffuse));
        current = [rgb[0], rgb[1], rgb[2], operation(stage["alpha_op"].as_u64().unwrap(), aa[3], ab[3], texture, diffuse)];
    }
    std::array::from_fn(|i| {
        let dst = f32::from(CLEAR_RGBA[i]) / 255.0;
        let color = match (native["blend"][0].as_u64().unwrap(), native["blend"][1].as_u64().unwrap()) {
            (5, 6) => current[i] * current[3] + dst * (1.0 - current[3]),
            (2, 2) => current[i] + dst,
            (9, 3) => 2.0 * current[i] * dst,
            blend => panic!("unknown native blend: {blend:?}"),
        };
        (color.clamp(0.0, 1.0) * 255.0).round() as u8
    })
}

struct Colors(HashMap<String, [f32; 4]>);

impl TextureSource for Colors {
    fn texture(&mut self, path: &str) -> Arc<TextureData> {
        let color = self.0.get(path).unwrap_or_else(|| panic!("unbound {path}"));
        let bytes = color.map(|v| (v * 255.0).round() as u8);
        Arc::new(TextureData { width: 2, height: 2, format: TextureFormatKind::Rgba8, mips: vec![bytes.repeat(4)] })
    }
}

fn quad(material: Material, attenuation: f32) -> Scene {
    let vertices = [(-100.0, -100.0), (100.0, -100.0), (100.0, 100.0), (-100.0, 100.0)]
        .map(|(x, z)| Vertex { position: [x, 0.0, z], normal: [0.0, -1.0, 0.0], uv: [0.0; 2], mask_uv: [0.0; 2], color: [attenuation; 4] });
    Scene { draws: vec![MeshDraw {
        instances: Vec::new(), vertices: vertices.to_vec(), indices: vec![0, 2, 1, 0, 3, 2], material, mask: None,
        kind: DrawKind::Projector, sun_affect: false, pass: Pass::Projector, zone: 0,
        lightmap: None, modulate_2x: false, sector: None, fade: None, tint: None, actor: None,
    }], ..Default::default() }
}

#[test]
#[ignore]
fn all_bsp_receiver_material_pixels_match_original_driver_states() {
    let root = std::env::var("KILLER_CLIENT_DIR").unwrap();
    let mut client = Client::open(Path::new(&root)).unwrap();
    let mut cases = Vec::new();
    let mut materials = Vec::new();
    let mut sampled = std::collections::HashSet::new();
    let matrix = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0; 4], [0.0, 0.0, 1.0, 1.0]];
    for tile in ["16_10", "20_17", "24_20"] {
        let loaded = client.load_level(tile).unwrap();
        for p in &loaded.level.projectors {
            let path = p.texture.as_deref().unwrap();
            let projected = client.catalog().material(path).unwrap();
            let projection = calc_matrix(p, projected.first_texture().unwrap().size.map(|n| n as u32), None);
            for polygon in &loaded.level.bsp {
                let points: Vec<_> = polygon.corners.iter().copied().map(Vec3::from).collect();
                if clip_bsp_polygon(&points, &projection.planes, true).is_empty() { continue; }
                let node = loaded.level.bsp_render.node_draws[polygon.node_index];
                let section = &loaded.level.bsp_render.sections[usize::try_from(node.section).unwrap()];
                let base = section.material.as_deref().unwrap();
                let class = client.catalog().object_class(base).unwrap();
                assert!(matches!(class.as_str(), "Texture" | "Shader"), "{base}: {class}");
                cases.push(json!({"base":if class == "Shader" { "shader_opacity" } else { "texture" },
                    "material_blend":p.material_blend,"frame_blend":p.frame_blend}));
                materials.push(client.catalog().bsp_projector_material(path, Some(base), p.material_blend, p.frame_blend, matrix).unwrap());
                if sampled.insert(class.clone()) {
                    for material_blend in 0..3 {
                        for frame_blend in 0..4 {
                            cases.push(json!({"base":if class == "Shader" { "shader_opacity" } else { "texture" },
                                "material_blend":material_blend,"frame_blend":frame_blend}));
                            materials.push(client.catalog().bsp_projector_material(path, Some(base), material_blend, frame_blend, matrix).unwrap());
                        }
                    }
                }
            }
        }
    }
    assert_eq!(cases.len(), 335);
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/projector_material_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script).arg(root).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let native: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(native.len(), materials.len());
    let mut gpu = Headless::new(32, 32).unwrap();
    let camera = Camera { eye: Vec3::new(0.0, -300.0, 0.0), target: Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    for (index, (material, original)) in materials.into_iter().zip(native).enumerate() {
        assert_eq!(original["success"], 1);
        let stages = original["stages"].as_array().unwrap();
        assert_eq!(material.stages.len(), stages.len(), "case {index}");
        let mut textures = Colors(HashMap::new());
        for (stage, native) in material.stages.iter().zip(stages) {
            if let StageSource::Texture(texture) = &stage.source {
                textures.0.insert(texture.path.clone(), sample(native["texture"].as_str().unwrap()));
            }
        }
        for attenuation in [0.0, 0.25, 1.0] {
            gpu.upload(&quad(material.clone(), attenuation), &mut textures);
            let pixels = gpu.render(&camera, &Environment::default()).unwrap();
            let actual = &pixels[(16 * 32 + 16) * 4..][..3];
            let expected = original_pixel(&original, attenuation);
            assert!(actual.iter().zip(expected).all(|(&a, b)| a.abs_diff(b) <= 2), "case {index} {} attenuation={attenuation}: {actual:?} != {expected:?}", cases[index]);
        }
    }
}
