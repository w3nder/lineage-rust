use killer_assets::abnormal_effects::{AbnormalDestroyError, AbnormalEffect, AbnormalEffectHost};
use killer_assets::texture_modifier::TextureModifyInfo;
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

struct Host {
    case: Value,
    events: Vec<Value>,
    info: [TextureModifyInfo; 2],
    mesh_fields: [[u32; 2]; 2],
    field_4bc: [u32; 2],
}

impl Host {
    fn flag(&self, name: &str) -> bool {
        self.case[name].as_bool().unwrap()
    }
    fn resource(&self, name: u32, mode: u64) -> Option<u32> {
        let present = self.case["resources"][name.to_string()].as_u64().unwrap_or(0);
        (present == 1 || present == mode).then_some(name)
    }
    fn new(case: Value) -> Self {
        let info = std::array::from_fn(|i| {
            let values: [u32; 4] = serde_json::from_value(case["info"][i].clone()).unwrap();
            TextureModifyInfo { flags: values[0], color: values[1], value_08: values[2], value_0c: values[3] }
        });
        Self { case, events: Vec::new(), info, mesh_fields: [[123, 456]; 2], field_4bc: [0x41100000, 0x41100001] }
    }
    fn result(mut self) -> Value {
        self.events.push(json!(["string_destroy"]));
        json!({"events": self.events,
            "info": self.info.map(|i| [i.flags, i.color, i.value_08, i.value_0c]),
            "mesh_fields": self.mesh_fields, "field_4bc": self.field_4bc})
    }
}

impl AbnormalEffectHost for Host {
    type Actor = usize;
    type Mesh = usize;
    type Emitter = u32;
    type Name = u32;
    type Material = u32;
    fn destroy_emitter(&mut self, _: usize, emitter: u32) {
        self.events.push(json!(["destroy_emitter", emitter, 0]));
    }
    fn report_missing_emitter_owner(&mut self) {
        if self.flag("log") {
            self.events.push(json!(["log"]));
        }
    }
    fn is_pawn(&self, _: usize) -> bool {
        self.flag("pawn")
    }
    fn active_associated_pawn(&self, _: usize) -> Option<usize> {
        (self.flag("associated") && self.flag("associated_active")).then_some(1)
    }
    fn skeletal_instance(&self, actor: usize) -> Option<usize> {
        (self.flag("mesh_present") && self.flag("skeletal")).then_some(actor)
    }
    fn texture_info(&mut self, actor: usize) -> &mut TextureModifyInfo {
        &mut self.info[actor]
    }
    fn clear_texture_modifier(&mut self, actor: usize) {
        self.events.push(json!(["clear", actor]));
        self.info[actor].clear();
    }
    fn head_bone(&mut self, actor: usize) -> u32 {
        self.events.push(json!(["head_name", actor]));
        77
    }
    fn clear_bone_scaler(&mut self, mesh: usize, name: u32) {
        self.events.push(json!(["clear_bone", mesh, name]));
    }
    fn clear_scale_fields(&mut self, mesh: usize) {
        self.mesh_fields[mesh] = [0, 0];
    }
    fn root_scaler_index(&self, _: usize) -> Option<usize> {
        self.case["scalers"].as_array().unwrap().iter().position(|name| name.as_u64() == Some(0))
    }
    fn remove_scaler(&mut self, mesh: usize, index: usize) {
        self.events.push(json!(["remove", mesh, index, 1]));
    }
    fn restore_field_4bc(&mut self, actor: usize, value: u32) {
        self.field_4bc[actor] = value;
    }
    fn has_game_data(&self) -> bool {
        self.flag("manager")
    }
    fn has_mesh_instance(&self, _: usize) -> bool {
        self.flag("mesh_present")
    }
    fn has_npc_texture_table(&self, _: usize) -> bool {
        self.flag("npc")
    }
    fn normal_npc_textures(&mut self, actor: usize) -> Vec<u32> {
        self.events.push(json!(["table", actor]));
        serde_json::from_value(self.case["names"].clone()).unwrap()
    }
    fn find_material_sequence(&mut self, name: u32) -> Option<u32> {
        self.events.push(json!(["find", name]));
        self.resource(name, 1)
    }
    fn load_material_sequence(&mut self, name: u32) -> Option<u32> {
        self.events.push(json!(["load", name]));
        self.resource(name, 2)
    }
    fn trigger_material_sequence(&mut self, material: u32) {
        self.events.push(json!(["trigger", material]));
    }
    fn set_npc_texture(&mut self, actor: usize, name: u32, index: usize) {
        self.events.push(json!(["set_texes", actor, 7, name, index]));
    }
}

fn baseline(mask: u32) -> Value {
    json!({"mask": mask, "owner": true, "pawn": true, "associated": true,
        "associated_active": true, "skeletal": true, "manager": true,
        "npc": true, "mesh_present": true, "emitter": true, "log": true,
        "resources": {"101": 1, "102": 2}, "names": [101, 102, 103],
        "scalers": [1, 0, 0], "info": [[5, 0x12345678u32, 2, 3], [1, 0x87654321u32, 1, 3]]})
}

fn effect(case: &Value) -> AbnormalEffect<usize, u32> {
    AbnormalEffect {
        owner: case["owner"].as_bool().unwrap().then_some(0),
        emitter: case["emitter"].as_bool().unwrap().then_some(42),
        mask: case["mask"].as_u64().unwrap() as u32,
        resource: String::from("original-resource"),
        saved_field_4bc: 0x42200000,
    }
}

#[test]
fn overlay_restoration_preserves_alpha_and_operations_when_flags_require_it() {
    let case = baseline(0x800);
    let mut host = Host::new(case.clone());
    effect(&case).destroy(&mut host).unwrap();
    assert_eq!(host.info[0], TextureModifyInfo { flags: 5, color: 0x12ffffff, value_08: 2, value_0c: 3 });
    assert_eq!(host.info[1], TextureModifyInfo { flags: 0, color: u32::MAX, value_08: 1, value_0c: 1 });
}

#[test]
fn invalid_overlay_owner_is_reported_instead_of_dereferenced() {
    let mut case = baseline(0x800);
    case["owner"] = json!(false);
    let mut host = Host::new(case.clone());
    assert_eq!(effect(&case).destroy(&mut host), Err(AbnormalDestroyError::OverlayWithoutOwner));
    assert_eq!(host.events, [json!(["log"])]);
}

#[test]
#[ignore]
fn abnormal_destructor_104924d0_matches_original_calls_and_writes() {
    let mut cases = Vec::new();
    for mask in (0..17).map(|bit| 1u32 << bit).chain([0, 0x8001, 0x80000000, u32::MAX]) {
        for variant in 0..1024 {
            let mut case = baseline(mask);
            for (bit, name) in [
                "owner",
                "pawn",
                "associated",
                "associated_active",
                "skeletal",
                "manager",
                "npc",
                "mesh_present",
                "emitter",
                "log",
            ]
            .iter()
            .enumerate()
            {
                case[*name] = json!(variant & (1 << bit) != 0);
            }
            if mask == 0x800 && !case["owner"].as_bool().unwrap() {
                continue;
            }
            case["scalers"] = match variant % 3 {
                0 => json!([]),
                1 => json!([2, 3]),
                _ => json!([1, 0, 0]),
            };
            cases.push(case);
        }
    }
    for flags in 0..256u32 {
        let mut case = baseline(0x800);
        case["info"] = json!([[flags | 0xa5000000, 0x12345678u32, 77, 88], [!flags, 0x98765432u32, 99, 111]]);
        cases.push(case);
    }
    let expected: Vec<Value> = cases
        .iter()
        .map(|case| {
            let mut host = Host::new(case.clone());
            effect(case).destroy(&mut host).unwrap();
            host.result()
        })
        .collect();
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/abnormal_destroy_reference.py"))
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
    eprintln!("{} destruições comparadas com a DLL original", cases.len());
}
