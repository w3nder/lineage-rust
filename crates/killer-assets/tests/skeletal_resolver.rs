use killer_assets::pawn_textures::PawnTextures;
use killer_assets::skeletal_material::{
    AttachedMaterialContext, InvalidTextureIndex, MaterialContext, TextureNode, base_texture,
    get_attached_weapon_texture, get_extra_sub_material, get_material, get_sub_material, sub_skin,
};
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

struct Context {
    mesh: Vec<Option<usize>>,
    skins: Option<Vec<Option<usize>>>,
    sub_skins: Option<[Option<usize>; 7]>,
    modified: bool,
    modifier: u8,
    shader_flags: u32,
    color_flags: u32,
    material: Option<usize>,
    calls: [usize; 3],
    second_skin: Option<Option<usize>>,
    attached: [Option<usize>; 8],
}

impl AttachedMaterialContext for Context {
    fn is_pawn(&self) -> bool {
        self.sub_skins.is_some()
    }
    fn attached_skin(&mut self, slot: i32, index: usize) -> Option<usize> {
        self.calls[1] += 1;
        if self.calls[1] == 2
            && let Some(value) = self.second_skin
        {
            return value;
        }
        if !self.is_pawn() {
            return None;
        }
        PawnTextures { attached: self.attached, ..Default::default() }.attached_skin(slot, index)
    }
}

impl MaterialContext for Context {
    fn has_actor(&self) -> bool {
        self.skins.is_some()
    }
    fn skin(&mut self, index: usize) -> Option<usize> {
        self.calls[0] += 1;
        if self.calls[0] == 2
            && let Some(value) = self.second_skin
        {
            return value;
        }
        self.skins.as_ref()?.get(index).copied().flatten()
    }
    fn first_skin(&self) -> Option<Option<usize>> {
        self.skins.as_ref()?.first().copied()
    }
    fn sub_skin(&mut self, index: i32) -> Option<usize> {
        self.calls[1] += 1;
        if self.calls[1] == 2
            && let Some(value) = self.second_skin
        {
            return value;
        }
        sub_skin(self.sub_skins.as_ref(), index)
    }
    fn mesh_texture(&self, index: usize) -> Option<Option<usize>> {
        self.mesh.get(index).copied()
    }
    fn modification_enabled(&self) -> bool {
        self.modified
    }
    fn modifier(&mut self) -> Option<usize> {
        self.calls[2] += 1;
        (self.modifier != 0).then_some(6)
    }
    fn shader_flags(&self, material: usize) -> Option<u32> {
        matches!(material, 1 | 2).then_some(self.shader_flags)
    }
    fn color_modifier_flags(&self, material: usize) -> Option<u32> {
        (material == 6 && self.modifier == 2).then_some(self.color_flags)
    }
    fn set_color_modifier_flags(&mut self, material: usize, flags: u32) {
        assert_eq!(material, 6);
        self.color_flags = flags;
    }
    fn set_modifier_material(&mut self, modifier: usize, material: Option<usize>) {
        assert_eq!(modifier, 6);
        self.material = material;
    }
    fn base_texture(&self, material: usize) -> Option<usize> {
        base_texture(
            &[
                TextureNode::Texture,
                TextureNode::Shader { diffuse: Some(0) },
                TextureNode::Shader { diffuse: None },
                TextureNode::FinalBlend { material: Some(1) },
                TextureNode::Other,
                TextureNode::Combiner { material_4c: Some(3) },
            ],
            Some(material),
        )
    }
}

fn context() -> Context {
    Context {
        mesh: vec![Some(0)],
        skins: Some(vec![Some(1)]),
        sub_skins: Some([Some(1), None, Some(4), Some(5), None, None, Some(2)]),
        modified: true,
        modifier: 2,
        shader_flags: 1,
        color_flags: 0xa5a5a5a0,
        material: Some(4),
        calls: [0; 3],
        second_skin: None,
        attached: [Some(0); 8],
    }
}

#[test]
fn second_getter_result_is_authoritative_and_negative_texture_index_is_rejected() {
    for sub in [false, true] {
        let mut context = context();
        context.second_skin = Some(None);
        let result = if sub { get_sub_material(&mut context, 0, 0) } else { get_material(&mut context, 0) };
        assert_eq!(result, Ok(None));
        assert_eq!(context.calls, if sub { [0, 2, 0] } else { [2, 0, 0] });
        assert_eq!(context.material, Some(4));
        for index in [i32::MIN, -1] {
            let calls = context.calls;
            assert_eq!(get_material(&mut context, index), Err(InvalidTextureIndex(index)));
            assert_eq!(get_sub_material(&mut context, index, 0), Err(InvalidTextureIndex(index)));
            assert_eq!(context.calls, calls);
        }
    }
}

#[test]
#[ignore]
fn get_material_106799d0_and_slot118_1067ec80_match_native_result_writes_and_calls() {
    let arrays = [
        vec![],
        vec![None],
        vec![Some(0)],
        vec![Some(1)],
        vec![Some(2)],
        vec![Some(3)],
        vec![Some(4)],
        vec![Some(5)],
        vec![None, Some(1)],
        vec![Some(2), None],
        vec![Some(3), Some(4), Some(5)],
    ];
    let skins = [
        None,
        Some(vec![]),
        Some(vec![None]),
        Some(vec![Some(0)]),
        Some(vec![Some(2)]),
        Some(vec![Some(4)]),
        Some(vec![None, Some(3)]),
        Some(vec![Some(1), None, Some(5)]),
    ];
    let mut cases = Vec::new();
    let mut expected = Vec::new();
    for mesh in &arrays {
        for skins in &skins {
            for index in [0, 1, 2, 3, i32::MAX] {
                for sub_index in [None, Some(-1), Some(0), Some(6), Some(7)] {
                    for modifier in 0..3 {
                        for modified in [false, true] {
                            for shader_bit in 0..2 {
                                for extra in [false, true] {
                                    let mut context = context();
                                    context.mesh = mesh.clone();
                                    context.skins = skins.clone();
                                    context.modifier = modifier;
                                    context.modified = modified;
                                    context.shader_flags = 0xfffffff0 | shader_bit;
                                    context.color_flags = 0xa5a5a5a0 | ((index as u32) & 1);
                                    let pawn = (mesh.len() + index as usize).is_multiple_of(2);
                                    let sub_skins = context.sub_skins.unwrap();
                                    if !pawn {
                                        context.sub_skins = None;
                                    }
                                    let extra_material =
                                        ((mesh.len() + index as usize + shader_bit as usize + modifier as usize) % 7)
                                            .checked_sub(1);
                                    cases.push(json!({
                                    "resolver": match (extra, sub_index.is_some()) {
                                        (true, true) => "extra_sub", (true, false) => "extra",
                                        (false, true) => "sub", (false, false) => "normal",
                                    },
                                    "extra_material": extra_material,
                                    "mesh": mesh, "skins": skins, "sub_skins": sub_skins, "pawn": pawn,
                                    "index": index, "sub_index": sub_index.unwrap_or(0), "modifier": modifier,
                                    "modified": modified, "shader_flags": context.shader_flags, "color_flags": context.color_flags,
                                }));
                                    let result = match sub_index {
                                        Some(_) if extra => get_extra_sub_material(&mut context, index, extra_material),
                                        Some(sub) => get_sub_material(&mut context, index, sub),
                                        None => get_material(&mut context, index),
                                    }
                                    .unwrap();
                                    expected.push(json!([
                                        result,
                                        context.material,
                                        context.color_flags,
                                        context.calls[0],
                                        context.calls[1],
                                        context.calls[2]
                                    ]));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    compare(cases, expected);
}

#[test]
#[ignore]
fn attached_weapon_texture_10676300_matches_all_slots_and_fallbacks() {
    let mut cases = Vec::new();
    let mut expected = Vec::new();
    for pattern in [0u32, 255, 170, 85, 1, 2, 4, 8, 16, 32, 64, 128] {
        for mesh in [vec![], vec![None], vec![Some(1), None, Some(4)]] {
            for slot in [-1, 7, 8, 9, 10, 11, 12, 13, i32::MAX] {
                for index in [0, 1, 2, 3, i32::MAX] {
                    for actor in 0..3 {
                        for modifier in 0..3 {
                            for modified in [false, true] {
                                let mut context = context();
                                context.mesh = mesh.clone();
                                context.attached = std::array::from_fn(|i| (pattern & (1 << i) != 0).then_some(i % 6));
                                context.modifier = modifier;
                                context.modified = modified;
                                if actor == 0 {
                                    context.skins = None;
                                }
                                if actor != 2 {
                                    context.sub_skins = None;
                                }
                                cases.push(json!({
                                    "resolver": "attached", "mesh": mesh, "skins": context.skins,
                                    "sub_skins": ([None::<usize>; 7]), "pawn": actor == 2,
                                    "attached": context.attached, "index": index, "slot": slot,
                                    "modifier": modifier, "modified": modified,
                                    "shader_flags": context.shader_flags, "color_flags": context.color_flags,
                                }));
                                let result = get_attached_weapon_texture(&mut context, index, slot).unwrap();
                                expected.push(json!([
                                    result,
                                    context.material,
                                    context.color_flags,
                                    context.calls[0],
                                    context.calls[1],
                                    context.calls[2]
                                ]));
                            }
                        }
                    }
                }
            }
        }
    }
    compare(cases, expected);
}

#[test]
fn attached_getter_second_result_null_does_not_fall_back_or_modify() {
    let mut context = context();
    context.second_skin = Some(None);
    assert_eq!(get_attached_weapon_texture(&mut context, 0, 8), Ok(None));
    assert_eq!(context.calls, [0, 2, 0]);
    assert_eq!(context.material, Some(4));
    assert_eq!(get_attached_weapon_texture(&mut context, -1, 8), Err(InvalidTextureIndex(-1)));
    assert_eq!(get_attached_weapon_texture(&mut context, -1, 7), Ok(None));
}

fn compare(cases: Vec<Value>, expected: Vec<Value>) {
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/skeletal_material_reference.py"))
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
    eprintln!("{} resoluções comparadas: retorno, mutações e contagem de chamadas", cases.len());
}
