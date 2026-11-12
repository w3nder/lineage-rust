use killer_assets::pawn_textures::{InvalidSkinIndex, PawnTextures, TextureResources};
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

struct Resources {
    loaded: bool,
    mutation: Option<Option<usize>>,
    events: Vec<Value>,
}

impl TextureResources for Resources {
    fn load(&mut self, slot: i32, target: &mut Option<usize>) -> bool {
        self.events.push(json!(["load", slot, target]));
        if let Some(value) = self.mutation {
            *target = value;
        }
        self.loaded
    }
    fn enqueue(&mut self, slot: i32, index: i32) {
        self.events.push(json!(["enqueue", slot, index]));
    }
}

#[test]
fn sparse_skin_array_keeps_zeroed_gaps_without_large_allocations() {
    let mut textures = PawnTextures::default();
    let mut resources = Resources { loaded: true, mutation: Some(Some(42)), events: Vec::new() };
    textures.set_texes(7, true, i32::MAX - 1, &mut resources).unwrap();
    assert_eq!(textures.skins.len(), i32::MAX as usize);
    assert_eq!(textures.skins.get(0), Some(None));
    assert_eq!(textures.skins.get(i32::MAX as usize - 1), Some(Some(42)));
    assert_eq!(textures.skins.get(i32::MAX as usize), None);
    let old = textures.clone();
    for (slot, index) in [(7, -1), (7, i32::MAX), (11, -1), (11, 3)] {
        assert_eq!(textures.set_texes(slot, true, index, &mut resources), Err(InvalidSkinIndex { slot, index }));
        assert_eq!(textures, old);
    }
    resources.mutation = Some(None);
    resources.loaded = false;
    textures.set_texes(7, true, 0, &mut resources).unwrap();
    assert!(textures.skins.is_empty());
}

#[test]
#[ignore]
fn set_texes_105e5fc0_matches_all_slots_resource_states_and_array_updates() {
    let mut cases = Vec::new();
    let mut expected = Vec::new();
    for slot in [-1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, i32::MAX] {
        for index in [-1, 0, 1, 2, 3, 4] {
            for has_name in [false, true] {
                if (slot == 7 && has_name && index < 0) || (slot == 11 && !(0..3).contains(&index)) {
                    continue;
                }
                for old_len in [0, 3] {
                    for loaded in [false, true] {
                        for mutation in 0..3 {
                            let mut textures =
                                PawnTextures { sub_skins: [Some(5); 7], attached: [Some(7); 8], ..Default::default() };
                            for index in 0..old_len {
                                let mut resources =
                                    Resources { loaded: true, mutation: Some(Some(5 + index)), events: Vec::new() };
                                textures.set_texes(7, true, index as i32, &mut resources).unwrap();
                            }
                            let mut resources = Resources {
                                loaded,
                                mutation: match mutation {
                                    0 => None,
                                    1 => Some(None),
                                    _ => Some(Some(9)),
                                },
                                events: Vec::new(),
                            };
                            textures.set_texes(slot, has_name, index, &mut resources).unwrap();
                            cases.push(json!({"slot": slot, "index": index, "has_name": has_name,
                                "old_len": old_len, "loaded": loaded, "mutation": mutation}));
                            let skins: Vec<_> =
                                (0..textures.skins.len()).map(|i| textures.skins.get(i).unwrap()).collect();
                            expected.push(json!({"sub": textures.sub_skins, "attached": textures.attached,
                                "skins": skins, "events": resources.events}));
                        }
                    }
                }
            }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/pawn_textures_reference.py"))
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
    eprintln!("{} transações SetTexes comparadas com o corpo original", cases.len());
}
