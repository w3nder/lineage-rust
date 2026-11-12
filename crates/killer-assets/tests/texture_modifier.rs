use killer_assets::texture_modifier::{L2ColorModifier, TextureModifierCache, TextureModifyInfo};
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn actor_color_setters_preserve_alpha_until_explicitly_cleared() {
    let initial = TextureModifyInfo { flags: 0xa0, color: 0x12345678, value_08: 77, value_0c: 1 };
    let mut info = initial;
    info.set_alpha(42);
    info.set_overlay([11, 22, 33], 4);
    assert_eq!(info, TextureModifyInfo { flags: 0xa5, color: 0x2a0b1621, value_08: 2, value_0c: 5 });
    info.clear_alpha();
    assert_eq!(info, TextureModifyInfo { flags: 0xa1, color: 0xff0b1621, value_08: 1, value_0c: 5 });
    info = initial;
    info.set_alpha(0);
    info.clear_alpha();
    assert_eq!(info, TextureModifyInfo { flags: 0xa0, color: u32::MAX, value_08: 1, value_0c: 1 });
}

#[test]
#[ignore]
fn actor_texture_setters_104f4560_match_native_sequences() {
    let mut cases = Vec::new();
    let mut expected = Vec::new();
    for mode in 0..=255u8 {
        for flags in [u32::from(mode), !u32::from(mode), 0xa5a5a5a5, 0] {
            for color_op in [1, u32::MAX] {
                let mut info = TextureModifyInfo { flags, color: 0x12345678, value_08: 0xabcdef01, value_0c: color_op };
                let initial = [info.flags, info.color, info.value_08, info.value_0c];
                let rgb = [mode, 255 - mode, mode ^ 0x5a];
                let mut steps = Vec::new();
                let mut snapshots = Vec::new();
                for operation in [1, 2, 3, 1, 4, 5, 6, 7, 2, 0] {
                    let arguments: Vec<u32> = match operation {
                        0 | 1 => Vec::new(),
                        2 => vec![u32::from(mode)],
                        3 => vec![u32::from(rgb[0]), u32::from(rgb[1]), u32::from(rgb[2]), u32::from(mode)],
                        _ => rgb.map(u32::from).to_vec(),
                    };
                    match operation {
                        0 => info.clear(),
                        1 => info.clear_alpha(),
                        2 => info.set_alpha(mode),
                        3 => info.set_overlay(rgb, mode),
                        4 => info.set_subtract(rgb),
                        5 => info.set_add(rgb),
                        6 => info.set_change_color(rgb),
                        7 => info.set_overlay_depend_alpha(rgb),
                        _ => unreachable!(),
                    }
                    steps.push(json!([operation, arguments]));
                    snapshots.push(json!([info.flags, info.color, info.value_08, info.value_0c]));
                }
                cases.push(json!({"initial": initial, "steps": steps}));
                expected.push(json!(snapshots));
            }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/texture_modifier_setters_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&serde_json::to_vec(&cases).unwrap()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let actual: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(*actual, expected, "{}", cases[index]);
    }
    eprintln!("{} sequências / {} alterações comparadas com os oito setters originais", cases.len(), cases.len() * 10);
}

#[test]
fn creation_failure_leaves_cache_empty_and_retry_then_reuses_the_object() {
    let mut cache = TextureModifierCache::default();
    let info = TextureModifyInfo { flags: 5, color: 42, value_08: 257, value_0c: 258 };
    assert_eq!(cache.get_modified_texture_material(info, || Err("failed")), Err("failed"));
    let initial = L2ColorModifier { material: Some(99), color: 0, flags: 0xa0, byte_50: 0, byte_51: 0 };
    let mut created = 0;
    for _ in 0..2 {
        let result = cache
            .get_modified_texture_material(info, || {
                created += 1;
                Ok::<_, ()>(initial)
            })
            .unwrap();
        assert_eq!(*result, L2ColorModifier { material: Some(99), color: 42, flags: 0xa2, byte_50: 1, byte_51: 2 });
    }
    assert_eq!(created, 1);
    assert!(info.is_modified_texture());
    assert!(info.uses_modifier());
    let info = TextureModifyInfo { flags: 1, ..info };
    assert!(!info.is_modified_texture());
    assert!(info.uses_modifier());
}

#[test]
#[ignore]
fn get_modified_texture_104ff6f0_and_clear_1063a2f0_match_native_sequences() {
    let mut cases = Vec::new();
    let mut expected = Vec::new();
    for flag in 0..256u32 {
        for initial_flags in [0, u32::MAX, 0xa5a5a5a5] {
            for present in [false, true] {
                let initial = L2ColorModifier {
                    material: (flag & 1 != 0).then_some(42),
                    color: 0x12345678,
                    flags: initial_flags,
                    byte_50: 77,
                    byte_51: 88,
                };
                let mut cache = if present {
                    TextureModifierCache::with_material(initial)
                } else {
                    TextureModifierCache::default()
                };
                let actors = [
                    [flag, flag.wrapping_mul(0x01010101), flag | 0xff000000, (255 - flag) | 0x7000],
                    [!flag, 0x98765432, 0x1234, 0xabcdef01],
                    [flag & !7, 0, 0, u32::MAX],
                ];
                let mut created = 0;
                let mut snapshots = Vec::new();
                for [flags, color, value_08, value_0c] in actors {
                    let mut info = TextureModifyInfo { flags, color, value_08, value_0c };
                    let modified = info.is_modified_texture();
                    let snapshot = *cache
                        .get_modified_texture_material(info, || {
                            created += 1;
                            Ok::<_, ()>(initial)
                        })
                        .unwrap();
                    info.clear();
                    snapshots.push(json!([
                        [
                            snapshot.material.unwrap_or(0) as u32,
                            snapshot.color,
                            snapshot.flags,
                            u32::from(snapshot.byte_50),
                            u32::from(snapshot.byte_51)
                        ],
                        modified,
                        [info.flags, info.color, info.value_08, info.value_0c]
                    ]));
                }
                cases.push(json!({"present": present, "initial": [initial.material.unwrap_or(0) as u32,
                    initial.color, initial.flags, u32::from(initial.byte_50), u32::from(initial.byte_51)], "actors": actors}));
                expected.push(json!([snapshots, created, created]));
            }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/texture_modifier_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&serde_json::to_vec(&cases).unwrap()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let actual: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(*actual, expected, "{}", cases[index]);
    }
    eprintln!("{} sequências de três atores comparadas com o modificador original", cases.len());
}
