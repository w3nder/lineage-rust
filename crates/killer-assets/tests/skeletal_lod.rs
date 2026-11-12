use killer_assets::skeletal_lod::{LodFrame, LodInstance, LodSelectionInput, select};
use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};
use ue2_assets::SkeletalLodInfo;

#[test]
fn render_context_uses_base_wedges_viewer_projection_and_preserves_invalid_state() {
    use killer_assets::skeletal_lod::{LodRenderInput, ProjectedFactorInput, half_fov_tangent, projected_factor};
    let settings = ue2_assets::SkeletalLodSettings {
        scale_max: 1.0, hysteresis: 0.0, strength: 1.0, min_vertices: 0, morph: 0.0,
        z_displace: 0.0, impostor: Some(0), reduction: Some(1),
    };
    let levels = [
        SkeletalLodInfo { soft_wedges: 1221, distance_factor: 1.0, hysteresis: 0.1 },
        SkeletalLodInfo { soft_wedges: 1, distance_factor: 0.5, hysteresis: 0.1 },
    ];
    let identity = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let mut projection = identity;
    projection[0][3] = 1.0;
    projection[3][3] = 0.0;
    let input = LodRenderInput {
        world_to_screen: projection, mesh_to_world: identity, location: [2000.0, 0.0, 0.0],
        soft_root: Some([3000.0, 0.0, 0.0]), viewer_pawn: Some([1000.0, 0.0, 0.0]),
        screen_x: 1558, controller_fov: 60.0, draw_scale: 1.0, draw_scale_3d: [0.5, 1.0, 0.75],
        lod_bias: 1.0, reduce_pawn_lod: false, keep_min_frame_rate: false, pawn_controller_flags: Some(0),
    };
    let factor_input = ProjectedFactorInput {
        soft_wedges: 1221, screen_x: 1558, half_fov_tangent: half_fov_tangent(60.0),
        projected_w: 1000.0, z_displace: 0.0, strength: 1.0, scale_max: 1.0,
        draw_scale: 1.0, lod_bias: 1.0, max_draw_scale_3d: 1.0,
    };
    let expected = projected_factor(factor_input);
    let mut instance = LodInstance { index: 1, ..Default::default() };
    assert!(instance.update_render(&levels, &settings, input).is_some());
    assert_eq!(instance.factor.to_bits(), expected.to_bits());
    let no_viewer = LodRenderInput { viewer_pawn: None, ..input };
    assert!(instance.update_render(&levels, &settings, no_viewer).is_some());
    assert_eq!(instance.factor.to_bits(), projected_factor(ProjectedFactorInput { projected_w: 2000.0, ..factor_input }).to_bits());
    assert!(instance.update_render(&levels, &ue2_assets::SkeletalLodSettings { impostor: Some(1), ..settings }, no_viewer).is_some());
    assert_eq!(instance.factor.to_bits(), projected_factor(ProjectedFactorInput { projected_w: 3000.0, ..factor_input }).to_bits());
    let previous = instance;
    assert!(instance.update_render(&levels, &ue2_assets::SkeletalLodSettings { reduction: None, ..settings }, input).is_none());
    assert_eq!(instance, previous);
    assert!(instance.update_render(&[], &settings, input).is_none());
    assert_eq!(instance, previous);
    instance.index = levels.len();
    let invalid = instance;
    assert!(instance.update_render(&levels, &settings, input).is_none());
    assert_eq!(instance, invalid);
}

#[test]
fn render_1069a520_instance_keeps_final_index_and_is_independent() {
    let mut instance = LodInstance::default();
    assert_eq!(instance.index, 0);
    assert_eq!(instance.forced, 0);
    assert_eq!(instance.factor.to_bits(), 0);
    let levels = [
        SkeletalLodInfo { soft_wedges: 0, distance_factor: 1.0, hysteresis: 0.25 },
        SkeletalLodInfo { soft_wedges: 0, distance_factor: 0.5, hysteresis: 0.1 },
        SkeletalLodInfo { soft_wedges: 0, distance_factor: 0.25, hysteresis: 0.0 },
    ];
    let other = LodInstance::default();
    let mut frame = LodFrame { factor: 0.6, reduce_pawn_lod: false, keep_min_frame_rate: false,
        pawn_controller_flags: Some(0), mesh_reduction: true, mesh_impostor: false };
    assert_eq!(instance.update(&levels, frame).unwrap().index, 1);
    frame.factor = 0.45;
    assert_eq!(instance.update(&levels, frame).unwrap().index, 1);
    frame.reduce_pawn_lod = true;
    assert_eq!(instance.update(&levels, frame).unwrap().index, 2);
    assert_eq!(instance.index, 2);
    assert_eq!(instance.factor.to_bits(), frame.factor.to_bits());
    assert_eq!(other, LodInstance::default());
    instance.forced = 1;
    frame.reduce_pawn_lod = false;
    assert_eq!(instance.update(&levels, frame).unwrap().index, 0);
    assert_eq!(instance.forced, 1);
    let previous = instance;
    assert!(instance.update(&[], frame).is_none());
    assert_eq!(instance, previous);
}

#[test]
fn render_1069a520_rejects_invalid_adapter_state() {
    let input = LodSelectionInput {
        factor: 0.0,
        previous: 0,
        forced: 0,
        reduce_pawn_lod: false,
        keep_min_frame_rate: false,
        pawn_controller_flags: None,
        mesh_reduction: false,
        mesh_impostor: false,
    };
    assert_eq!(select(&[], input), None);
    let levels = [SkeletalLodInfo { soft_wedges: 0, distance_factor: 1.0, hysteresis: 0.1 }];
    assert_eq!(select(&levels, LodSelectionInput { previous: 1, ..input }), None);
}

#[test]
#[ignore]
fn render_1069a520_lod_selection_matches_original_instructions() {
    let mut cases = Vec::new();
    let mut expected = Vec::new();
    let mut add = |levels: Vec<SkeletalLodInfo>, input: LodSelectionInput| {
        let result = select(&levels, input).unwrap();
        cases.push(json!({
            "levels": levels.iter().map(|level| [level.distance_factor.to_bits(), level.hysteresis.to_bits()]).collect::<Vec<_>>(),
            "factor": input.factor.to_bits(), "previous": input.previous, "forced": input.forced,
            "reduce": input.reduce_pawn_lod, "keep": input.keep_min_frame_rate,
            "owner": input.pawn_controller_flags, "reduction": input.mesh_reduction,
            "impostor": input.mesh_impostor,
        }));
        expected.push((result.index, result.impostor));
    };
    for count in 1..=3 {
        for previous in 0..count {
            for forced in [-1, 0, 1, 2, 3, 4] {
                for bits in [
                    0, 0x80000000, 0x3f000000, 0x3f400000, 0x3f800000, 0x3f7fffff, 0x3f800001, 0x7f800000, 0xff800000,
                    0x7fc00000,
                ] {
                    for flags in 0..32 {
                        let levels = (0..count)
                            .map(|index| SkeletalLodInfo {
                                soft_wedges: 0,
                                distance_factor: 1.0 - index as f32 * 0.25,
                                hysteresis: 0.25,
                            })
                            .collect();
                        add(
                            levels,
                            LodSelectionInput {
                                factor: f32::from_bits(bits),
                                previous,
                                forced,
                                reduce_pawn_lod: flags & 1 != 0,
                                keep_min_frame_rate: flags & 2 != 0,
                                pawn_controller_flags: match flags & 12 {
                                    0 => None,
                                    4 => Some(0),
                                    8 => Some(1),
                                    _ => Some(2),
                                },
                                mesh_reduction: flags & 16 != 0,
                                mesh_impostor: true,
                            },
                        );
                    }
                }
            }
        }
    }
    let mut random = 0x12345678u32;
    for index in 0..2048 {
        let mut next = || {
            random = random.wrapping_mul(1664525).wrapping_add(1013904223);
            f32::from_bits(random)
        };
        let levels = (0..3).map(|_| SkeletalLodInfo { soft_wedges: 0, distance_factor: next(), hysteresis: next() }).collect();
        add(
            levels,
            LodSelectionInput {
                factor: next(),
                previous: index % 3,
                forced: 0,
                reduce_pawn_lod: index & 1 != 0,
                keep_min_frame_rate: false,
                pawn_controller_flags: Some(0),
                mesh_reduction: true,
                mesh_impostor: index & 2 != 0,
            },
        );
    }
    for flags in 0..=u8::MAX {
        for keep in [false, true] {
            add(
                vec![SkeletalLodInfo { soft_wedges: 0, distance_factor: 0.5, hysteresis: 0.0 }; 3],
                LodSelectionInput {
                    factor: 1.0,
                    previous: 0,
                    forced: 0,
                    reduce_pawn_lod: !keep,
                    keep_min_frame_rate: keep,
                    pawn_controller_flags: Some(flags),
                    mesh_reduction: true,
                    mesh_impostor: false,
                },
            );
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/skeletal_lod_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&serde_json::to_vec(&cases).unwrap()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let actual: Vec<(usize, bool)> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
        assert_eq!(actual, expected, "case {}: {}", index, cases[index]);
    }
    eprintln!("{} casos comparados com engine.dll", expected.len());
}
