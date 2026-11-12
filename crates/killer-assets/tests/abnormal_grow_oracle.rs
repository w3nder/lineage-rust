use killer_assets::abnormal_effects::AbnormalEffectHost;
use killer_assets::abnormal_state::{AbnormalStat, AbnormalStatHost, GrowDimensions, NpcTextureLists, init, update};
use killer_assets::material_sequence::MaterialSequence;
use killer_assets::texture_modifier::TextureModifyInfo;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const FIXTURE: &str = include_str!("data/abnormal_grow_oracle.json");
const RADIUS: usize = 0;
const HEIGHT: usize = 1;
const BASE_RADIUS: usize = 2;
const BASE_HEIGHT: usize = 3;
const Z: usize = 4;
const WEAPON_SCALE: usize = 5;
const PAWN_KEYS: [&str; 6] = ["radius", "height", "base_radius", "base_height", "z", "weapon_scale"];

struct Host {
    case: Value,
    events: Vec<Value>,
    pawns: [[u32; 6]; 2],
    sequences: BTreeMap<u32, MaterialSequence>,
    info: TextureModifyInfo,
}

impl Host {
    fn new(case: &Value) -> Self {
        let pawns = std::array::from_fn(|pawn| std::array::from_fn(|field| case["pawns"][pawn][PAWN_KEYS[field]].as_u64().unwrap() as u32));
        let sequences = case["sequences"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(name, state)| {
                let state: [u32; 4] = serde_json::from_value(state.clone()).unwrap();
                let sequence = MaterialSequence {
                    trigger_action: state[0] as u8,
                    flags: state[1],
                    current_time: f32::from_bits(state[2]),
                    last_time: f32::from_bits(state[3]),
                };
                (name.parse().unwrap(), sequence)
            })
            .collect();
        Self { case: case.clone(), events: Vec::new(), pawns, sequences, info: TextureModifyInfo { flags: 0, color: 0, value_08: 0, value_0c: 0 } }
    }

    fn flag(&self, name: &str) -> bool {
        self.case[name].as_bool().unwrap()
    }

    fn flag_of(&self, name: &str, actor: usize) -> bool {
        self.case[name][actor].as_bool().unwrap()
    }

    fn resource(&self, name: u32, load: bool) -> Option<u32> {
        let mode = self.case["resources"][name.to_string()].as_u64().unwrap_or(0);
        (mode == 1 || (load && mode == 2)).then_some(name)
    }

    fn names(&self, key: &str) -> Vec<u32> {
        serde_json::from_value(self.case[key].clone()).unwrap()
    }

    fn float(&self, actor: usize, field: usize) -> f32 {
        f32::from_bits(self.pawns[actor][field])
    }
}

impl AbnormalEffectHost for Host {
    type Actor = usize;
    type Mesh = usize;
    type Emitter = u32;
    type Name = u32;
    type Material = u32;

    fn destroy_emitter(&mut self, _: usize, _: u32) {
        unreachable!()
    }
    fn report_missing_emitter_owner(&mut self) {}
    fn is_pawn(&self, _: usize) -> bool {
        self.flag("pawn")
    }
    fn active_associated_pawn(&self, _: usize) -> Option<usize> {
        (self.flag("associated") && self.flag_of("ride", 1)).then_some(1)
    }
    fn skeletal_instance(&self, actor: usize) -> Option<usize> {
        (self.flag_of("inst", actor) && self.flag("inst_cast")).then_some(actor)
    }
    fn texture_info(&mut self, _: usize) -> &mut TextureModifyInfo {
        &mut self.info
    }
    fn clear_texture_modifier(&mut self, _: usize) {}
    fn head_bone(&mut self, _: usize) -> u32 {
        unreachable!()
    }
    fn clear_bone_scaler(&mut self, _: usize, _: u32) {
        unreachable!()
    }
    fn clear_scale_fields(&mut self, _: usize) {}
    fn root_scaler_index(&self, _: usize) -> Option<usize> {
        self.case["scalers"].as_array().unwrap().iter().position(|bone| bone.as_u64() == Some(0))
    }
    fn remove_scaler(&mut self, mesh: usize, index: usize) {
        self.events.push(json!(["remove", mesh, index, 1]));
    }
    fn restore_field_4bc(&mut self, actor: usize, value: u32) {
        self.pawns[actor][WEAPON_SCALE] = value;
    }
    fn has_game_data(&self) -> bool {
        true
    }
    fn has_mesh_instance(&self, actor: usize) -> bool {
        self.flag_of("inst", actor)
    }
    fn has_npc_texture_table(&self, actor: usize) -> bool {
        self.flag_of("npc", actor)
    }
    fn normal_npc_textures(&mut self, _: usize) -> Vec<u32> {
        unreachable!()
    }
    fn find_material_sequence(&mut self, name: u32) -> Option<u32> {
        self.events.push(json!(["find_object", name]));
        self.resource(name, false)
    }
    fn load_material_sequence(&mut self, name: u32) -> Option<u32> {
        self.events.push(json!(["load_object", name]));
        self.resource(name, true)
    }
    fn trigger_material_sequence(&mut self, material: u32) {
        if let Some(sequence) = self.sequences.get_mut(&material) {
            sequence.trigger();
        }
    }
    fn set_npc_texture(&mut self, actor: usize, name: u32, index: usize) {
        self.events.push(json!(["set_texes", actor, 7, name, index]));
    }
}

impl AbnormalStatHost for Host {
    type Class = u32;

    fn pawn_lights(&mut self, _: usize) -> Option<&mut Vec<killer_assets::pawn_light::PawnLight>> {
        None
    }

    fn has_skeletal_mesh(&self, _: usize) -> bool {
        self.flag("mesh") && self.flag("skeletal")
    }
    fn pawn_drawn(&self, _: usize) -> bool {
        unreachable!()
    }
    fn load_effect_class(&mut self, _: &str) -> Option<u32> {
        unreachable!()
    }
    fn spawn_emitter(&mut self, _: u32, _: usize, _: [f32; 3], _: [i32; 3]) -> Option<u32> {
        unreachable!()
    }
    fn location_rotation(&self, _: usize) -> ([f32; 3], [i32; 3]) {
        unreachable!()
    }
    fn collision(&self, _: usize) -> (f32, f32) {
        unreachable!()
    }
    fn draw_scale(&self, _: usize) -> f32 {
        unreachable!()
    }
    fn alias_bone(&self, _: usize, _: &str) -> Option<u32> {
        unreachable!()
    }
    fn alias_origin(&self, _: usize, _: &str) -> [f32; 3] {
        unreachable!()
    }
    fn mesh_origin_z(&self, _: usize) -> f32 {
        unreachable!()
    }
    fn attach_to_bone(&mut self, _: usize, _: u32, _: u32, _: [f32; 3]) -> bool {
        unreachable!()
    }
    fn attach_to_root(&mut self, _: usize, _: u32) {
        unreachable!()
    }
    fn set_pre_pivot_z(&mut self, _: u32, _: f32) {
        unreachable!()
    }
    fn set_size_scale(&mut self, _: u32, _: f32) {
        unreachable!()
    }
    fn set_overlay(&mut self, _: usize, _: [u8; 3], _: u8) {
        unreachable!()
    }
    fn stop_action(&mut self, _: usize) {
        unreachable!()
    }
    fn head_scale(&self, _: usize) -> f32 {
        unreachable!()
    }
    fn set_bone_scale(&mut self, owner: usize, scale: f32, head: bool) {
        assert!(!head);
        self.events.push(json!(["bone_scale", owner, 0, scale.to_bits(), 0, 1]));
    }
    fn big_head_max(&mut self, _: usize) -> f32 {
        unreachable!()
    }
    fn play_damage_sound(&mut self, _: usize) {
        unreachable!()
    }
    fn play_sound(&mut self, _: usize, _: &str, _: f32, _: f32) {
        unreachable!()
    }
    fn is_npc(&self, actor: usize) -> bool {
        self.flag_of("npc", actor)
    }
    fn npc_texture_lists(&mut self, actor: usize) -> Option<NpcTextureLists<u32>> {
        self.events.push(json!(["table", actor]));
        Some(NpcTextureLists { normal: self.names("textures"), abnormal: self.names("textures2") })
    }
    fn grow_dimensions(&self, actor: usize) -> GrowDimensions {
        GrowDimensions {
            collision: [self.float(actor, RADIUS), self.float(actor, HEIGHT)],
            base_collision: [self.float(actor, BASE_RADIUS), self.float(actor, BASE_HEIGHT)],
            z: self.float(actor, Z),
            weapon_scale: self.float(actor, WEAPON_SCALE),
        }
    }
    fn set_collision_size(&mut self, actor: usize, size: [f32; 2]) {
        self.pawns[actor][RADIUS] = size[0].to_bits();
        self.pawns[actor][HEIGHT] = size[1].to_bits();
    }
    fn set_location_z(&mut self, actor: usize, z: f32) {
        self.pawns[actor][Z] = z.to_bits();
    }
    fn set_weapon_scale(&mut self, actor: usize, scale: f32) {
        self.pawns[actor][WEAPON_SCALE] = scale.to_bits();
    }
}

fn same(actual: u32, expected: u64) -> bool {
    let expected = expected as u32;
    actual == expected || (f32::from_bits(actual).is_nan() && f32::from_bits(expected).is_nan())
}

fn stat_fields(stat: &AbnormalStat<usize, u32>) -> [u32; 8] {
    [
        stat.mask,
        stat.elapsed.to_bits(),
        stat.duration.to_bits(),
        stat.scale.to_bits(),
        stat.state,
        stat.saved_collision[0].to_bits(),
        stat.saved_collision[1].to_bits(),
        stat.saved_weapon_scale.to_bits(),
    ]
}

fn compare(label: &str, host: &Host, fields: [u32; 8], expected: &Value, events: &[Value]) {
    for (field, (actual, wanted)) in fields.iter().zip(expected["stat"].as_array().unwrap()).enumerate() {
        let wanted = wanted.as_u64().unwrap();
        let exact = field == 0 || field == 4;
        assert!(if exact { u64::from(*actual) == wanted } else { same(*actual, wanted) }, "{label}: campo {field} do stat {actual:#x} != {wanted:#x}");
    }
    for pawn in 0..2 {
        for (field, wanted) in expected["pawns"][pawn].as_array().unwrap().iter().enumerate() {
            let actual = host.pawns[pawn][field];
            assert!(same(actual, wanted.as_u64().unwrap()), "{label}: pawn {pawn} {} {actual:#x} != {wanted}", PAWN_KEYS[field]);
        }
    }
    let wanted_events: Vec<Value> = expected["events"].as_array().unwrap().clone();
    assert_eq!(events.len(), wanted_events.len(), "{label}: eventos {events:?} != {wanted_events:?}");
    for (actual, wanted) in events.iter().zip(&wanted_events) {
        if actual[0] == "bone_scale" {
            let (actual, wanted) = (actual.as_array().unwrap(), wanted.as_array().unwrap());
            assert_eq!(actual[..3], wanted[..3], "{label}");
            assert!(same(actual[3].as_u64().unwrap() as u32, wanted[3].as_u64().unwrap()), "{label}: escala {actual:?} != {wanted:?}");
            assert_eq!(actual[4..], wanted[4..], "{label}");
        } else {
            assert_eq!(actual, wanted, "{label}");
        }
    }
    if let Some(sequences) = expected["sequences"].as_object() {
        for (name, wanted) in sequences {
            let sequence = host.sequences[&name.parse::<u32>().unwrap()];
            let actual = [u32::from(sequence.trigger_action), sequence.flags, sequence.current_time.to_bits(), sequence.last_time.to_bits()];
            let wanted: [u32; 4] = serde_json::from_value(wanted.clone()).unwrap();
            assert_eq!(actual, wanted, "{label}: sequência {name}");
        }
    }
}

#[test]
fn grow_and_material_sequence_match_init_update_and_destructor_run_in_the_oracle() {
    let cases: Vec<Value> = serde_json::from_str(FIXTURE).unwrap();
    let mut steps = 0;
    for (index, case) in cases.iter().enumerate() {
        assert!(case.get("fault").is_none(), "caso {index}: {}", case["fault"]);
        let input = &case["input"];
        let output = case["output"].as_array().unwrap();
        let mut host = Host::new(input);
        let bit = input["bit"].as_u64().unwrap() as u32;
        let mut stat = init(&mut host, 0, bit, false);
        let events = std::mem::take(&mut host.events);
        compare(&format!("caso {index} init"), &host, stat_fields(&stat), &output[0], &events);
        for (step, (entry, expected)) in input["steps"].as_array().unwrap().iter().zip(&output[1..]).enumerate() {
            let label = format!("caso {index} passo {step}");
            if let Some(state) = entry["force_state"].as_u64() {
                stat.state = state as u32;
            }
            if let Some(duration) = entry["force_duration"].as_u64() {
                stat.duration = f32::from_bits(duration as u32);
            }
            if entry["destroy"].as_bool() == Some(true) {
                let fields = stat_fields(&stat);
                stat.into_effect(String::new()).destroy(&mut host).unwrap();
                let events = std::mem::take(&mut host.events);
                compare(&label, &host, fields, expected, &events);
                break;
            }
            let dt = f32::from_bits(entry["dt"].as_u64().unwrap() as u32);
            let result = update(&mut host, &mut stat, dt, false);
            assert_eq!(u64::from(result), expected["result"].as_u64().unwrap(), "{label}");
            let events = std::mem::take(&mut host.events);
            compare(&label, &host, stat_fields(&stat), expected, &events);
            steps += 1;
        }
    }
    eprintln!("{} casos, {steps} quadros comparados com a DLL", cases.len());
}
