use super::*;
use std::fmt::Write as _;
use std::io::Write as _;
use std::process::{Command, Stdio};

fn info(color_op: u32, alpha_op: u32) -> Info {
    Info { l2_modifier: Some(Modifier { color: 0x801040f0, color_op, alpha_op }), ..Info::new() }
}

#[test]
fn l2_stage_translates_current_factor_arguments_without_sampling_a_texture() {
    let operations = [
        Some(ColorOp::Disable),
        Some(ColorOp::SelectCurrent),
        Some(ColorOp::SelectTexture),
        Some(ColorOp::Modulate),
        Some(ColorOp::Modulate2X),
        Some(ColorOp::Modulate4X),
        Some(ColorOp::Add),
        Some(ColorOp::AddSigned),
        Some(ColorOp::AddSigned2X),
        Some(ColorOp::Subtract),
        Some(ColorOp::AddSmooth),
        Some(ColorOp::BlendDiffuseAlpha),
        None,
        Some(ColorOp::BlendFactorAlpha),
        None,
        Some(ColorOp::BlendCurrentAlpha),
        Some(ColorOp::PreModulate),
        Some(ColorOp::ModulateAlphaAddColor),
        Some(ColorOp::ModulateColorAddAlpha),
        Some(ColorOp::ModulateInvAlphaAddColor),
        Some(ColorOp::ModulateInvColorAddAlpha),
        None,
        None,
        Some(ColorOp::DotProduct3),
        Some(ColorOp::MultiplyAddCurrent),
    ];
    for op in 0..=255 {
        let expected = operations.get(op as usize).copied().unwrap_or(Some(ColorOp::Disable));
        for path in [MaterialPath::Simple, MaterialPath::Shader] {
            let mut pass = Pass::new(0);
            let before = pass.clone();
            let mut stages = Vec::new();
            let result = l2_color_modifier(&mut pass, &mut stages, &info(op, 1), path);
            if let Some(color_op) = expected {
                assert_eq!(result, Some(()), "{op} {path:?}");
                assert_eq!(stages.len(), 1);
                assert_eq!(stages[0].color_op, color_op);
                assert_eq!(stages[0].alpha_op, AlphaOp::Current);
                assert_eq!(stages[0].source, StageSource::Factor);
                assert!(stages[0].swapped);
                assert_eq!(
                    pass.tfactor,
                    if path == MaterialPath::Simple || op != 1 { Some([16, 64, 240, 128]) } else { None }
                );
                assert_eq!(tuple(&pass), tuple(&before));
            } else {
                assert_eq!(result, None);
                assert_eq!(pass, before);
                assert!(stages.is_empty());
            }
        }
    }
}

fn tuple(pass: &Pass) -> (BlendFactor, BlendFactor, bool, Option<u8>, bool) {
    (pass.src, pass.dst, pass.alpha_blend, pass.alpha_test, pass.two_sided)
}

#[test]
fn l2_stage_keeps_both_caller_factor_rules_and_promotes_alpha_before_transforming() {
    for path in [MaterialPath::Simple, MaterialPath::Shader] {
        let mut pass = Pass::new(0);
        pass.dst = BlendFactor::InvSrcColor;
        let mut stages = Vec::new();
        l2_color_modifier(&mut pass, &mut stages, &info(21, 0), path).unwrap();
        assert_eq!(
            pass.tfactor,
            Some(if path == MaterialPath::Simple { [16, 64, 240, 128] } else { [8, 32, 120, 128] })
        );
        assert_eq!(stages[0].color_op, ColorOp::Modulate);
        assert_eq!(stages[0].alpha_op, AlphaOp::Modulate);
        assert_eq!(pass.alpha_test, Some(0));

        let mut pass = Pass::new(0);
        let modifier = Info { alpha_blend: true, two_sided: true, ..info(3, 0) };
        l2_color_modifier(&mut pass, &mut Vec::new(), &modifier, path).unwrap();
        assert_eq!(tuple(&pass), (BlendFactor::SrcAlpha, BlendFactor::InvSrcAlpha, true, Some(0), true));
    }
}

#[test]
fn l2_stage_preserves_fade_when_shader_does_not_write_factor_and_refuses_overflow_atomically() {
    let fade = Fade { color1: [1, 2, 3, 4], color2: [4, 3, 2, 1], period: 2.0, phase: 0.0, sinusoidal: false };
    let mut pass = Pass::new(0);
    pass.fade = Some(fade);
    pass.fade_alpha_replicate = true;
    l2_color_modifier(&mut pass, &mut Vec::new(), &info(1, 1), MaterialPath::Shader).unwrap();
    assert_eq!(pass.fade, Some(fade));
    assert!(pass.fade_alpha_replicate);
    l2_color_modifier(&mut pass, &mut Vec::new(), &info(1, 1), MaterialPath::Simple).unwrap();
    assert!(pass.fade.is_none());
    assert!(!pass.fade_alpha_replicate);
    for first in [0, 3, 10] {
        let mut pass = Pass::new(first);
        let mut stages = vec![Stage::factor(ColorOp::SelectCurrent, AlphaOp::Current); first + 7];
        l2_color_modifier(&mut pass, &mut stages, &info(3, 0), MaterialPath::Simple).unwrap();
        let before = (pass.clone(), stages.clone());
        assert_eq!(l2_color_modifier(&mut pass, &mut stages, &info(6, 0), MaterialPath::Simple), None);
        assert_eq!((pass, stages), before);
    }
}

#[test]
fn simple_material_appends_l2_after_regular_modifier_and_shares_final_factor() {
    let mut catalog = Catalog::default();
    let node = Node { class: "texture".into(), size: [64.0; 2], ..Node::null() };
    let mut info = Info { modify_color: true, tfactor: [64; 4], two_sided: true, ..info(9, 1) };
    let leaf = Leaf { path: "Synthetic.Texture".into(), node, info: info.clone() };
    let mut stages = Vec::new();
    let mut passes = Vec::new();
    catalog.simple_material(&leaf, &mut info, &mut stages, &mut passes).unwrap();
    assert_eq!(stages.len(), 3);
    assert_eq!(stages[1].color_op, ColorOp::Modulate);
    assert!(!stages[1].swapped);
    assert_eq!(stages[2].color_op, ColorOp::Subtract);
    assert!(stages[2].swapped);
    assert_eq!(passes[0].stages, 0..3);
    assert_eq!(passes[0].light_after, 1);
    assert_eq!(passes[0].tfactor, Some([16, 64, 240, 128]));
    assert!(passes[0].two_sided);
}

#[test]
fn opaque_texture_keeps_l2_two_sided_but_not_an_unrelated_initial_flag() {
    let mut with_l2 = Info { two_sided: true, ..info(1, 1) };
    texture_flags(&mut with_l2, false, false, false);
    assert!(with_l2.two_sided);
    let mut without_l2 = Info { two_sided: true, ..Info::new() };
    texture_flags(&mut without_l2, false, false, false);
    assert!(!without_l2.two_sided);
    texture_flags(&mut with_l2, true, false, false);
    assert!(!with_l2.two_sided);
}

#[test]
#[ignore]
fn catalog_l2_snapshot_does_not_leak_to_other_material_requests() {
    let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut catalog = Catalog::open(&root);
    let path = "LineageEffectsTextures.Etc.Enchant_Aura001_Shader01";
    let original = catalog.material(path).unwrap();
    let modifier = info(3, 1).l2_modifier.unwrap();
    let flags = ModifierFlags { alpha_blend: 0, two_sided: 0 };
    let modified = catalog.l2_modified_material(path, modifier, flags).unwrap();
    assert_eq!(modified.stages.len(), original.stages.len() + 1);
    assert_eq!(modified.passes[0].stages.end, original.passes[0].stages.end + 1);
    let stage = &modified.stages[modified.passes[0].stages.end - 1];
    assert_eq!((stage.color_op, stage.alpha_op, stage.swapped), (ColorOp::Modulate, AlphaOp::Current, true));
    assert_eq!(modified.passes[0].tfactor, Some([16, 64, 240, 128]));
    assert!(modified.passes[0].fade.is_none());
    assert_eq!(catalog.material(path).unwrap(), original);
    assert_eq!(catalog.l2_modified_material(path, modifier, flags).unwrap(), modified);
}

fn d3d_color(op: ColorOp) -> u32 {
    match op {
        ColorOp::Disable => 1,
        ColorOp::SelectCurrent => 2,
        ColorOp::SelectTexture => 3,
        ColorOp::Modulate => 4,
        ColorOp::Modulate2X => 5,
        ColorOp::Modulate4X => 6,
        ColorOp::Add => 7,
        ColorOp::AddSigned => 8,
        ColorOp::AddSigned2X => 9,
        ColorOp::Subtract => 10,
        ColorOp::AddSmooth => 11,
        ColorOp::BlendDiffuseAlpha => 12,
        ColorOp::BlendFactorAlpha => 14,
        ColorOp::BlendCurrentAlpha => 16,
        ColorOp::PreModulate => 17,
        ColorOp::ModulateAlphaAddColor => 18,
        ColorOp::ModulateColorAddAlpha => 19,
        ColorOp::ModulateInvAlphaAddColor => 20,
        ColorOp::ModulateInvColorAddAlpha => 21,
        ColorOp::DotProduct3 => 24,
        ColorOp::MultiplyAddCurrent => 25,
        other => panic!("operação inesperada {other:?}"),
    }
}

fn d3d_blend(blend: BlendFactor) -> u32 {
    match blend {
        BlendFactor::Zero => 1,
        BlendFactor::One => 2,
        BlendFactor::SrcColor => 3,
        BlendFactor::InvSrcColor => 4,
        BlendFactor::SrcAlpha => 5,
        BlendFactor::InvSrcAlpha => 6,
        BlendFactor::DstColor => 9,
    }
}

#[test]
#[ignore]
fn translated_catalog_passes_match_original_10009b0e_1000a0f9() {
    let mut input = String::new();
    let mut expected = Vec::new();
    let blends = [
        (BlendFactor::One, BlendFactor::Zero),
        (BlendFactor::One, BlendFactor::InvSrcColor),
        (BlendFactor::SrcAlpha, BlendFactor::InvSrcAlpha),
        (BlendFactor::One, BlendFactor::InvSrcAlpha),
        (BlendFactor::DstColor, BlendFactor::SrcColor),
        (BlendFactor::Zero, BlendFactor::InvSrcColor),
    ];
    for (path_id, path) in [MaterialPath::Shader, MaterialPath::Simple].into_iter().enumerate() {
        for color_op in (0..=25).chain([255, u32::MAX]) {
            for alpha_op in [0, 1, 255] {
                for (src, dst) in blends {
                    for bits in 0..16 {
                        for reference in [None, Some(127), Some(255)] {
                            let mut pass = Pass::new(0);
                            pass.src = src;
                            pass.dst = dst;
                            pass.alpha_blend = bits & 1 != 0;
                            pass.two_sided = bits & 2 != 0;
                            pass.alpha_test = reference;
                            pass.tfactor = Some([0xcd, 0xef, 1, 0xab]);
                            let info = Info {
                                alpha_blend: bits & 4 != 0,
                                two_sided: bits & 8 != 0,
                                ..info(color_op, alpha_op)
                            };
                            writeln!(
                                input,
                                "{} {alpha_op} {color_op} {} {} {} {} {} 0 2 2 {path_id} {} {} {} {} 1",
                                0x801040f0u32,
                                0xabcdef01u32,
                                d3d_blend(src),
                                d3d_blend(dst),
                                u32::from(pass.alpha_blend),
                                reference.unwrap_or(0),
                                u32::from(info.alpha_blend),
                                u32::from(info.two_sided),
                                u32::from(reference.is_some()),
                                u32::from(pass.two_sided)
                            )
                            .unwrap();
                            let mut stages = Vec::new();
                            if l2_color_modifier(&mut pass, &mut stages, &info, path).is_none() {
                                expected.push(None);
                                continue;
                            }
                            let stage = &stages[0];
                            assert_eq!(stage.source, StageSource::Factor);
                            assert!(stage.swapped);
                            let [r, g, b, a] = pass.tfactor.unwrap();
                            let alpha = match stage.alpha_op {
                                AlphaOp::Current => 2,
                                AlphaOp::TFactor => 3,
                                AlphaOp::Modulate => 4,
                                other => panic!("alpha inesperado {other:?}"),
                            };
                            expected.push(Some([
                                u32::from_le_bytes([b, g, r, a]),
                                d3d_blend(pass.src),
                                d3d_blend(pass.dst),
                                u32::from(pass.alpha_blend),
                                u32::from(pass.alpha_test.unwrap_or(0)),
                                0,
                                d3d_color(stage.color_op),
                                alpha,
                                1,
                                3,
                                1,
                                3,
                                1,
                                u32::from(pass.alpha_test.is_some()),
                                u32::from(pass.two_sided),
                            ]));
                        }
                    }
                }
            }
        }
    }
    let mut child = Command::new(std::env::var_os("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/l2_color_stage_reference.py"))
        .arg(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), expected.len());
    let mut accepted = 0;
    for (index, (line, expected)) in output.lines().zip(&expected).enumerate() {
        let actual: Vec<u32> = line.split_whitespace().map(|value| value.parse().unwrap()).collect();
        if let Some(expected) = expected {
            assert_eq!(actual, *expected, "caso {index}");
            accepted += 1;
        } else {
            assert!(matches!(actual[6], 13 | 15 | 22 | 23), "recusa indevida no caso {index}: {actual:?}");
        }
    }
    eprintln!(
        "{accepted} passes traduzidos e {} recusas explícitas comparados com o original",
        expected.len() - accepted
    );
}
