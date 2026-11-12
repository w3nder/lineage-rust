use killer_assets::lightmap_software::{LightActor, LightBitmap, LightMapGeometry, LightRender, Surface, accumulate_rgba, expand_shadow_bits, light_intensity};
use serde_json::Value;

const EXPAND: &str = include_str!("data/lightmap_expand_oracle.json");
const EFFECTS: &str = include_str!("data/lightmap_effects_oracle.json");

fn ints(value: &Value) -> Vec<i32> {
    value.as_array().unwrap().iter().map(|v| v.as_i64().unwrap() as i32).collect()
}

fn bytes(value: &Value) -> Vec<u8> {
    value.as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u8).collect()
}

fn floats<const N: usize>(value: &Value) -> [f32; N] {
    let list = value.as_array().unwrap();
    std::array::from_fn(|index| list[index].as_f64().unwrap() as f32)
}

#[test]
fn fun_10634d70_expansion_equals_the_engine_function_run_in_the_emulator() {
    let fixture: Value = serde_json::from_str(EXPAND).unwrap();
    for (index, case) in fixture["expand"].as_array().unwrap().iter().enumerate() {
        let bounds = ints(&case["bounds"]);
        let bitmap = LightBitmap {
            bits: bytes(&case["bits"]),
            rows: case["rows"].as_i64().unwrap() as i32,
            stride: case["stride"].as_i64().unwrap() as i32,
            ..LightBitmap::default()
        };
        let bitmap = (!case["none"].as_bool().unwrap()).then_some(&bitmap);
        let got = expand_shadow_bits(bitmap, bounds[0], bounds[1], bounds[2], bounds[3]);
        assert_eq!(got, bytes(&case["out"]), "caso {index}");
    }
}

#[test]
fn fun_10634a80_accumulation_equals_the_engine_function_run_in_the_emulator() {
    let fixture: Value = serde_json::from_str(EXPAND).unwrap();
    for (index, case) in fixture["accumulate"].as_array().unwrap().iter().enumerate() {
        let bounds = ints(&case["bounds"]);
        let mut dest: Vec<u32> = case["dest"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u32).collect();
        accumulate_rgba(&bytes(&case["intensity"]), [bounds[0], bounds[1], bounds[2], bounds[3]], floats::<3>(&case["color"]), &mut dest, case["size_x"].as_i64().unwrap() as i32);
        let want: Vec<u32> = case["out"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u32).collect();
        assert_eq!(dest, want, "caso {index}");
    }
}

#[test]
fn fun_10635020_every_light_effect_equals_the_engine_function_run_in_the_emulator() {
    let fixture: Value = serde_json::from_str(EFFECTS).unwrap();
    let mut failures = Vec::new();
    for (index, case) in fixture["effects"].as_array().unwrap().iter().enumerate() {
        let effect = case["effect"].as_u64().unwrap() as u8;
        let light = LightRender {
            actor: Some(LightActor {
                effect,
                period: case["period"].as_u64().unwrap() as u8,
                phase: case["phase"].as_u64().unwrap() as u8,
                cone: case["cone"].as_u64().unwrap() as u8,
                time_seconds: case["time"].as_f64().unwrap() as f32,
                rotation: ints(&case["rotation"]).try_into().unwrap(),
                is_pawn: case["pawn"].as_bool().unwrap(),
                controller_rotation: case["controller"].as_array().map(|_| ints(&case["controller"]).try_into().unwrap()),
            }),
            position: floats(&case["pos"]),
            direction: floats(&case["direction"]),
            radius: case["radius"].as_f64().unwrap() as f32,
            ..LightRender::default()
        };
        let surface = Surface { flags: case["surf_flags"].as_u64().unwrap() as u32, normal: floats(&case["normal"]), distance: 0.0 };
        let lm_size = ints(&case["lm_size"]);
        let geometry = LightMapGeometry { size: [lm_size[0], lm_size[1]], base: floats(&case["base"]), axis_x: floats(&case["axis_x"]), axis_y: floats(&case["axis_y"]) };
        let b = ints(&case["bounds"]);
        let pitch = case["pitch"].as_i64().unwrap() as i32;
        let stride = case["use_bitmap"].as_bool().unwrap().then_some(pitch / 8);
        let got = light_intensity(&bytes(&case["intensity"]), &surface, &geometry, stride, [b[0], b[1], b[2], b[3]], &light);
        let want = bytes(&case["out"]);
        if got != want {
            let shown = got.len().min(60);
            failures.push(format!("caso {index} efeito {effect:#x}: {:?} != {:?}", &got[..shown], &want[..shown]));
        }
    }
    let mut per_effect = std::collections::BTreeMap::new();
    for failure in &failures {
        let effect = failure.split("efeito ").nth(1).and_then(|rest| rest.split(':').next()).unwrap_or("?").to_string();
        *per_effect.entry(effect).or_insert(0) += 1;
    }
    let mut seen = std::collections::BTreeSet::new();
    let firsts: Vec<&String> = failures.iter().filter(|f| seen.insert(f.split(':').next().unwrap_or("").split("efeito").nth(1).unwrap_or("").to_string())).collect();
    assert!(failures.is_empty(), "{} divergencias {per_effect:?}\n{}", failures.len(), firsts.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n"));
}
