use l2_catalog::l2_color_stage::{
    MaterialPath, Modifier, ModifierFlags, PassFlags, PassState, StageState, apply, apply_pass, texture_operation,
};
use std::fmt::Write as _;
use std::io::Write as _;
use std::process::{Command, Stdio};

#[test]
fn operation_table_covers_all_byte_values_and_invalid_words() {
    for operation in 0..=255 {
        assert_eq!(texture_operation(operation), if operation <= 24 { operation + 1 } else { 1 });
    }
    assert_eq!(texture_operation(u32::MAX), 1);
}

fn native(input: &str) -> Vec<Vec<u32>> {
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/l2_color_stage_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| line.split_whitespace().map(|v| v.parse().unwrap()).collect())
        .collect()
}

#[test]
#[ignore]
fn l2_color_stage_10004fe0_matches_original_including_all_channel_alpha_pairs() {
    let mut cases = Vec::new();
    for alpha in 0..=255u32 {
        for channel in 0..=255u32 {
            cases.push([(channel * 0x010101) | (alpha << 24), 0, 3, 0xabcdef01, 2, 4, 0, 0, 42, 2, 3]);
        }
    }
    for op in (0..=25).chain([255, u32::MAX]) {
        for alpha_op in [0, 1, 2, 255, u32::MAX] {
            for [src, dst] in [[2, 1], [2, 4], [5, 6], [2, 6], [5, 4], [1, 1]] {
                for blend in [0, 1, 0x80000000] {
                    for reference in [0, 126, 127, 128, 255] {
                        for initial_op in [1, 2, 3, 4, 25] {
                            cases.push([
                                0x7f12fe80, alpha_op, op, 0xabcdef01, src, dst, blend, reference, 42, initial_op, 9,
                            ]);
                        }
                    }
                }
            }
        }
    }
    let mut input = String::new();
    let mut expected = Vec::new();
    for &[color, alpha_op, color_op, factor, src, dst, alpha_blend, alpha_ref, texture, stage_color, stage_alpha] in
        &cases
    {
        writeln!(input, "{color} {alpha_op} {color_op} {factor} {src} {dst} {alpha_blend} {alpha_ref} {texture} {stage_color} {stage_alpha}").unwrap();
        let mut pass = PassState { factor, src, dst, alpha_blend, alpha_ref: alpha_ref as u8 };
        let mut stage = StageState {
            texture,
            color_op: stage_color,
            alpha_op: stage_alpha,
            color_args: [7, 8],
            color_arg0: color_op ^ 0x31,
            alpha_args: [9, 10],
        };
        apply(Modifier { color, alpha_op, color_op }, &mut pass, &mut stage);
        expected.push([
            pass.factor,
            pass.src,
            pass.dst,
            pass.alpha_blend,
            u32::from(pass.alpha_ref),
            stage.texture,
            stage.color_op,
            stage.alpha_op,
            stage.color_args[0],
            stage.color_args[1],
            stage.alpha_args[0],
            stage.alpha_args[1],
            stage.color_arg0,
        ]);
    }
    let actual = native(&input);
    assert_eq!(actual.len(), cases.len());
    for (index, actual) in actual.iter().enumerate() {
        assert_eq!(*actual, expected[index], "case {:?}", cases[index]);
    }
    eprintln!("{} estados comparados com d3ddrv 10004fe0", cases.len());
}

#[test]
#[ignore]
fn shader_and_simple_l2_modifier_blocks_10009b0e_1000a0f9_match_original() {
    let mut input = String::new();
    let mut expected = Vec::new();
    for (path_id, path) in [MaterialPath::Shader, MaterialPath::Simple].into_iter().enumerate() {
        for color_op in (0..=25).chain([255, u32::MAX]) {
            for alpha_op in [0, 1, 255] {
                for [src, dst] in [[2, 1], [2, 4], [5, 6], [2, 6], [5, 4], [1, 1]] {
                    for modifier_blend in [0, 1, 0x80000000] {
                        for initial_blend in [0, 1, 0x40000000] {
                            for alpha_ref in [0, 127, 255] {
                                let modifier = Modifier { color: 0x7f12fe80, alpha_op, color_op };
                                let modifier_flags =
                                    ModifierFlags { alpha_blend: modifier_blend, two_sided: modifier_blend };
                                let mut pass =
                                    PassState { factor: 0xabcdef01, src, dst, alpha_blend: initial_blend, alpha_ref };
                                let mut flags = PassFlags { alpha_test: 7, two_sided: 2 };
                                let mut stage = StageState {
                                    texture: 42,
                                    color_op: 2,
                                    alpha_op: 9,
                                    color_args: [7, 8],
                                    color_arg0: color_op ^ 0x31,
                                    alpha_args: [9, 10],
                                };
                                writeln!(input, "{} {alpha_op} {color_op} {} {src} {dst} {initial_blend} {alpha_ref} 42 2 9 {path_id} {modifier_blend} {modifier_blend} 7 2", modifier.color, pass.factor).unwrap();
                                apply_pass(modifier, modifier_flags, &mut pass, &mut flags, &mut stage, path);
                                expected.push([
                                    pass.factor,
                                    pass.src,
                                    pass.dst,
                                    pass.alpha_blend,
                                    u32::from(pass.alpha_ref),
                                    stage.texture,
                                    stage.color_op,
                                    stage.alpha_op,
                                    stage.color_args[0],
                                    stage.color_args[1],
                                    stage.alpha_args[0],
                                    stage.alpha_args[1],
                                    stage.color_arg0,
                                    flags.alpha_test,
                                    flags.two_sided,
                                ]);
                            }
                        }
                    }
                }
            }
        }
    }
    let actual = native(&input);
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
        assert_eq!(*actual, *expected, "case {index}");
    }
    eprintln!("{} estados comparados nos dois trechos chamadores", expected.len());
}
