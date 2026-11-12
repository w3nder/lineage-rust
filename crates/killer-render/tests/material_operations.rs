use glam::Vec3;
use killer_assets::{
    scene::{DrawKind, MeshDraw, Pass, Scene, Vertex, texture_material},
    textures::{TextureData, TextureFormatKind, TextureSource},
};
use killer_render::{camera::Camera, environment::Environment, headless::Headless};
use l2_catalog::{AlphaOp, ColorOp, StageSource};
use std::sync::Arc;

struct Colors([[u8; 4]; 2]);

impl TextureSource for Colors {
    fn texture(&mut self, path: &str) -> Arc<TextureData> {
        let index = match path {
            "first" => 0,
            "second" => 1,
            _ => panic!("unexpected texture {path}"),
        };
        Arc::new(TextureData {
            width: 1,
            height: 1,
            format: TextureFormatKind::Rgba8,
            mips: vec![self.0[index].to_vec()],
        })
    }
}

fn scene(op: ColorOp, swapped: bool) -> Scene {
    let mut material = texture_material("first");
    let mut second = texture_material("second").stages.remove(0);
    second.color_op = op;
    second.alpha_op = AlphaOp::Current;
    second.swapped = swapped;
    material.stages.push(second);
    material.passes[0].stages = 0..2;
    material.passes[0].lit = false;
    material.passes[0].two_sided = true;
    let vertices = [(-100.0, -100.0), (100.0, -100.0), (100.0, 100.0), (-100.0, 100.0)].map(|(x, z)| Vertex {
        position: [x, 0.0, z],
        normal: [0.0, -1.0, 0.0],
        uv: [0.5; 2],
        mask_uv: [0.0; 2],
        color: [1.0; 4],
    });
    Scene {
        draws: vec![MeshDraw {
            instances: Vec::new(), vertices: vertices.to_vec(),
            indices: vec![0, 2, 1, 0, 3, 2],
            material,
            mask: None,
            kind: DrawKind::StaticMesh,
            sun_affect: false,
            pass: Pass::Opaque,
            zone: 0,
            lightmap: None,
            modulate_2x: false,
            sector: None,
            fade: None,
            tint: None,
            actor: None,
        }],
        ..Default::default()
    }
}

#[test]
#[ignore]
fn catalog_l2_modifier_reaches_gpu_with_current_before_factor() {
    use l2_catalog::l2_color_stage::{Modifier, ModifierFlags};
    let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut catalog = l2_catalog::Catalog::open(&root);
    let path = "LineageMonstersTex.gremlin_t00";
    let original = catalog.material(path).unwrap();
    assert_eq!(original.stages.len(), 1);
    let mut gpu = Headless::new(32, 32).unwrap();
    let camera =
        Camera { eye: Vec3::new(0.0, -300.0, 0.0), target: Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    let bytes = [[70, 140, 210, 180], [0; 4]];
    let current = bytes[0].map(|v| f64::from(v) / 255.0);
    let factor = [16, 64, 240, 128].map(|v| f64::from(v) / 255.0);
    for op in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 16, 17, 18, 19, 20, 23, 24, 255] {
        let modifier = Modifier { color: 0x801040f0, color_op: op, alpha_op: 1 };
        let mut material =
            catalog.l2_modified_material(path, modifier, ModifierFlags { alpha_blend: 0, two_sided: 0 }).unwrap();
        assert_eq!(material.stages.len(), 2);
        if let StageSource::Texture(texture) = &mut material.stages[0].source {
            texture.path = "first".into();
            texture.frames.clear();
            texture.total_frames = 0;
        } else {
            panic!("textura inicial esperada");
        }
        material.passes[0].lit = false;
        material.passes[0].alpha_blend = false;
        material.passes[0].alpha_test = None;
        material.passes[0].two_sided = true;
        let expected = std::array::from_fn::<_, 3, _>(|i| {
            let [a, b] = [current[i], factor[i]];
            let value = match op {
                0 | 1 | 11 | 16 | 255 => a,
                2 => b,
                3 => a * b,
                4 => 2.0 * a * b,
                5 => 4.0 * a * b,
                6 => a + b,
                7 => a + b - 0.5,
                8 => 2.0 * (a + b - 0.5),
                9 => a - b,
                10 => a + b * (1.0 - a),
                13 => a * factor[3] + b * (1.0 - factor[3]),
                15 => a * current[3] + b * (1.0 - current[3]),
                17 => a + current[3] * b,
                18 => a * b + current[3],
                19 => a + (1.0 - current[3]) * b,
                20 => (1.0 - a) * b + current[3],
                23 => (0..3).map(|c| (2.0 * current[c] - 1.0) * (2.0 * factor[c] - 1.0)).sum(),
                24 => a + a * b,
                _ => unreachable!(),
            };
            (value.clamp(0.0, 1.0) * 255.0).round() as u8
        });
        let expected = [expected[0], expected[1], expected[2], if op == 23 { expected[0] } else { bytes[0][3] }];
        let mut scene = scene(ColorOp::SelectCurrent, false);
        scene.draws[0].material = material;
        gpu.upload(&scene, &mut Colors(bytes));
        let pixels = gpu.render(&camera, &Environment::default()).unwrap();
        let actual = &pixels[(16 * 32 + 16) * 4..][..4];
        assert!(
            actual.iter().zip(expected).all(|(&a, b)| a.abs_diff(b) <= 1),
            "ColorOp={op}: {actual:?} != {expected:?}"
        );
    }
    assert_eq!(catalog.material(path).unwrap(), original);
}

#[test]
fn l2_texture_ops_100049a0_arithmetic_reaches_gpu_with_both_argument_orders() {
    let operations = [
        ColorOp::AddSigned,
        ColorOp::AddSigned2X,
        ColorOp::AddSmooth,
        ColorOp::BlendTextureAlphaPremultiplied,
        ColorOp::ModulateColorAddAlpha,
        ColorOp::ModulateInvAlphaAddColor,
        ColorOp::ModulateInvColorAddAlpha,
        ColorOp::MultiplyAddCurrent,
        ColorOp::DotProduct3,
    ];
    let samples = [
        [[0, 0, 0, 0], [0, 0, 0, 0]],
        [[255; 4], [255; 4]],
        [[10, 90, 210, 27], [240, 70, 20, 190]],
        [[64, 127, 192, 255], [63, 128, 191, 0]],
        [[130, 65, 32, 0], [20, 91, 222, 255]],
    ];
    let mut gpu = Headless::new(32, 32).unwrap();
    let camera =
        Camera { eye: Vec3::new(0.0, -300.0, 0.0), target: Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    for op in operations {
        for swapped in [false, true] {
            let scene = scene(op, swapped);
            for bytes in samples {
                let [current, texture] = bytes.map(|color| color.map(|v| f64::from(v) / 255.0));
                let [a, b] = if swapped { [current, texture] } else { [texture, current] };
                let expected: [u8; 3] = std::array::from_fn(|i| {
                    let value = match op {
                        ColorOp::AddSigned => a[i] + b[i] - 0.5,
                        ColorOp::AddSigned2X => 2.0 * a[i] + 2.0 * b[i] - 1.0,
                        ColorOp::AddSmooth => a[i] + b[i] - a[i] * b[i],
                        ColorOp::BlendTextureAlphaPremultiplied => a[i] + b[i] - b[i] * texture[3],
                        ColorOp::ModulateColorAddAlpha => a[i] * b[i] + a[3],
                        ColorOp::ModulateInvAlphaAddColor => a[i] + b[i] - a[3] * b[i],
                        ColorOp::ModulateInvColorAddAlpha => b[i] - a[i] * b[i] + a[3],
                        ColorOp::MultiplyAddCurrent => current[i] + a[i] * b[i],
                        ColorOp::DotProduct3 => (0..3).map(|c| 4.0 * (a[c] - 0.5) * (b[c] - 0.5)).sum(),
                        _ => unreachable!(),
                    };
                    (value.clamp(0.0, 1.0) * 255.0).round() as u8
                });
                gpu.upload(&scene, &mut Colors(bytes));
                let pixels = gpu.render(&camera, &Environment::default()).unwrap();
                let actual = &pixels[(16 * 32 + 16) * 4..][..4];
                let expected = [
                    expected[0],
                    expected[1],
                    expected[2],
                    if op == ColorOp::DotProduct3 { expected[0] } else { bytes[0][3] },
                ];
                assert!(
                    actual.iter().zip(expected).all(|(&a, b)| a.abs_diff(b) <= 1),
                    "{op:?} swapped={swapped} bytes={bytes:?}: {actual:?} != {expected:?}"
                );
            }
        }
    }
}

#[test]
fn disable_preserves_lighting_of_prior_stages_but_does_not_reach_later_lighting() {
    let mut gpu = Headless::new(32, 32).unwrap();
    let camera =
        Camera { eye: Vec3::new(0.0, -300.0, 0.0), target: Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    for light_after in [1, 2] {
        let mut tested = scene(ColorOp::Disable, false);
        for vertex in &mut tested.draws[0].vertices {
            vertex.color = [0.3, 0.4, 0.7, 0.5];
        }
        tested.draws[0].material.passes[0].lit = true;
        tested.draws[0].material.passes[0].light_after = light_after;
        let mut reference = tested.clone();
        reference.draws[0].material.stages.truncate(1);
        reference.draws[0].material.passes[0].stages = 0..1;
        reference.draws[0].material.passes[0].lit = light_after == 1;
        reference.draws[0].material.passes[0].light_after = 1;
        let bytes = [[120, 150, 180, 200], [255; 4]];
        gpu.upload(&reference, &mut Colors(bytes));
        let expected = gpu.render(&camera, &Environment::default()).unwrap();
        gpu.upload(&tested, &mut Colors(bytes));
        let actual = gpu.render(&camera, &Environment::default()).unwrap();
        assert_eq!(actual, expected, "light_after={light_after}");
    }
}

#[test]
fn disable_stops_later_stages_and_premodulate_changes_current_only_for_a_following_texture() {
    let bytes = [[70, 140, 210, 180], [180, 60, 120, 90]];
    let factor = [30, 110, 200, 240];
    let mut gpu = Headless::new(32, 32).unwrap();
    let camera =
        Camera { eye: Vec3::new(0.0, -300.0, 0.0), target: Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    for operation in [ColorOp::Disable, ColorOp::PreModulate] {
        for swapped in [false, true] {
            for next_texture in [false, true] {
                for next_operation in [ColorOp::SelectCurrent, ColorOp::BlendCurrentAlpha] {
                    let mut scene = scene(operation, swapped);
                    let material = &mut scene.draws[0].material;
                    material.stages[1].source = StageSource::Factor;
                    let mut last = texture_material("second").stages.remove(0);
                    if !next_texture {
                        last.source = StageSource::Factor;
                    }
                    last.color_op = next_operation;
                    last.alpha_op = AlphaOp::Current;
                    material.stages.push(last);
                    material.passes[0].stages = 0..3;
                    material.passes[0].tfactor = Some(factor);
                    let mut expected = bytes[0].map(|v| f64::from(v) / 255.0);
                    if operation == ColorOp::PreModulate {
                        if !swapped {
                            for i in 0..3 {
                                expected[i] = f64::from(factor[i]) / 255.0;
                            }
                        }
                        if next_texture {
                            for i in 0..4 {
                                expected[i] *= f64::from(bytes[1][i]) / 255.0;
                            }
                        }
                        if next_operation == ColorOp::BlendCurrentAlpha {
                            let alpha = f64::from(bytes[0][3]) / 255.0;
                            let source = if next_texture { bytes[1] } else { factor };
                            for i in 0..3 {
                                expected[i] = f64::from(source[i]) / 255.0 * alpha + expected[i] * (1.0 - alpha);
                            }
                        }
                    }
                    let expected = expected.map(|v| (v * 255.0).round() as u8);
                    gpu.upload(&scene, &mut Colors(bytes));
                    let pixels = gpu.render(&camera, &Environment::default()).unwrap();
                    let actual = &pixels[(16 * 32 + 16) * 4..][..4];
                    assert!(
                        actual.iter().zip(expected).all(|(&a, b)| a.abs_diff(b) <= 1),
                        "{operation:?} swapped={swapped} next_texture={next_texture}: {actual:?} != {expected:?}"
                    );
                }
            }
        }
    }
}
