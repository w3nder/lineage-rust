use killer_assets::lightmap_software::{LightActor, LightBitmap, LightMapGeometry, LightRender, Surface};
use killer_assets::lightmap_texture::{BitmapEntry, Environment, LightMapCache, LightMapSource, TimeSlice, ZoneLight, cache_key, fget_hsv, get_texture_data};
use serde_json::Value;

const HSV: &str = include_str!("data/lightmap_hsv_oracle.json");
const TEXTURE: &str = include_str!("data/lightmap_texture_oracle.json");
const REAL: &str = include_str!("data/lightmap_real_oracle.json");
const MODEL_CACHE_INDEX: u32 = 0x1234567;

fn ints(value: &Value) -> Vec<i32> {
    value.as_array().unwrap().iter().map(|v| v.as_i64().unwrap() as i32).collect()
}

fn floats<const N: usize>(value: &Value) -> [f32; N] {
    let list = value.as_array().unwrap();
    std::array::from_fn(|index| list[index].as_f64().unwrap() as f32)
}

fn render(actor: &Value, light: &Value, time: f32) -> LightRender {
    LightRender {
        actor: Some(LightActor {
            effect: actor["effect"].as_u64().unwrap() as u8,
            period: actor["period"].as_u64().unwrap() as u8,
            phase: actor["phase"].as_u64().unwrap() as u8,
            cone: actor["cone"].as_u64().unwrap() as u8,
            time_seconds: time,
            rotation: ints(&actor["rotation"]).try_into().unwrap(),
            is_pawn: false,
            controller_rotation: actor["controller"].as_array().map(|_| ints(&actor["controller"]).try_into().unwrap()),
        }),
        position: floats(&light["pos"]),
        direction: floats(&light["dir"]),
        radius: light["radius"].as_f64().unwrap() as f32,
        dynamic: light["dynamic"].as_i64().unwrap() as i32,
        color: floats(&light["color"]),
    }
}

fn ambient(time: f32) -> [u8; 3] {
    let time = f64::from(time);
    [((time * 7.3) as i64 & 0xff) as u8, ((time * 3.1 + 50.0) as i64 & 0xff) as u8, ((time * 11.7 + 9.0) as i64 & 0xff) as u8]
}

#[test]
fn fgethsv_equals_the_engine_function_run_in_the_emulator() {
    let fixture: Value = serde_json::from_str(HSV).unwrap();
    for case in fixture["hsv"].as_array().unwrap() {
        let hsv = ints(&case["hsv"]);
        let want: Vec<u32> = case["plane"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u32).collect();
        let got = fget_hsv(hsv[0] as u8, hsv[1] as u8, hsv[2] as u8);
        let got: Vec<u32> = got.iter().map(|v| v.to_bits()).collect();
        assert_eq!(got, want, "{hsv:?}");
    }
}

#[test]
fn flightmap_get_texture_data_equals_the_engine_function_run_in_the_emulator() {
    let failures = divergences(TEXTURE);
    assert!(failures.is_empty(), "{} divergencias; primeiras: {:#?}", failures.len(), &failures[..failures.len().min(8)]);
}

#[test]
fn real_giran_lightmaps_built_by_the_app_equal_the_engine_function_run_in_the_emulator() {
    let failures = divergences(REAL);
    assert!(failures.is_empty(), "{} divergencias; primeiras: {:#?}", failures.len(), &failures[..failures.len().min(8)]);
    let fixture: Value = serde_json::from_str(REAL).unwrap();
    let changed = fixture["texture"].as_array().unwrap().iter().filter(|scenario| scenario["results"][0]["dest"] != scenario["results"][1]["dest"]).count();
    assert!(changed >= 3, "{changed}");
}

fn divergences(text: &str) -> Vec<String> {
    let fixture: Value = serde_json::from_str(text).unwrap();
    let mut failures = Vec::new();
    for (number, scenario) in fixture["texture"].as_array().unwrap().iter().enumerate() {
        let mut cache = LightMapCache::default();
        let timed = scenario["variant"] == "timed";
        let zone_json = if scenario["zone_actor"].is_null() { &scenario["first_actor"] } else { &scenario["zone_actor"] };
        let hsv = ints(&zone_json["hsv"]);
        let zone = ZoneLight { sun: zone_json["sun"].as_bool().unwrap(), hue: hsv[0] as u8, saturation: hsv[1] as u8, brightness: hsv[2] as u8 };
        let g = &scenario["geometry"];
        let m: [f32; 16] = floats(&g["matrix"]);
        let matrix = std::array::from_fn(|row| std::array::from_fn(|column| m[row * 4 + column]));
        let surf = &scenario["surf"];
        let surface = Surface { flags: surf["flags"].as_u64().unwrap() as u32, normal: floats(&surf["normal"]), distance: surf["plane"].as_f64().unwrap() as f32 };
        let geometry = LightMapGeometry { size: [0, 0], base: floats(&g["base"]), axis_x: floats(&g["axis_x"]), axis_y: floats(&g["axis_y"]) };
        for (call_number, (call, result)) in scenario["calls"].as_array().unwrap().iter().zip(scenario["results"].as_array().unwrap()).enumerate() {
            let grow = ints(&call["grow"]);
            let base_size = ints(&scenario["size"]);
            let size = [base_size[0] + grow[0], base_size[1] + grow[1]];
            let level_time = call["level_time"].as_f64().unwrap() as f32;
            let deleted: Vec<bool> = call["deleted"].as_array().unwrap().iter().map(|v| v.as_bool().unwrap()).collect();
            let bitmaps = scenario["bitmaps"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let present = !item["actor"].is_null();
                    let actor = if present { &item["actor"] } else { &scenario["bitmaps"][0]["actor"] };
                    let b = ints(&item["bounds"]);
                    let editor_hidden = present && item["actor"]["editor_hidden"].as_bool().unwrap();
                    BitmapEntry {
                        bitmap: LightBitmap {
                            bits: item["bits"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u8).collect(),
                            rows: item["rows"].as_i64().unwrap() as i32,
                            stride: item["stride"].as_i64().unwrap() as i32,
                            min_x: b[0],
                            min_y: b[1],
                            max_x: b[2],
                            max_y: b[3],
                        },
                        time: item["time"].as_i64().unwrap() as i32,
                        actor_present: present,
                        deleted: present && deleted[index],
                        editor_hidden,
                        light: if present { render(actor, &item["light"], level_time) } else { LightRender::default() },
                    }
                })
                .collect();
            let dynamic = scenario["dynamic"].as_array().unwrap().iter().map(|item| render(&item["actor"], &item["light"], level_time)).collect();
            let source = LightMapSource { size, surface, geometry: LightMapGeometry { size, ..geometry }, matrix, zone, bitmaps, dynamic };
            let slice = if timed { TimeSlice::Timed { index: call["time_index"].as_i64().unwrap() as i32, count: call["time_count"].as_i64().unwrap() as i32 } } else { TimeSlice::Single };
            let editor = call["editor"].as_bool().unwrap();
            let env_time = call["env_time"].as_f64().unwrap();
            let env = Environment { editor, time: env_time, ambient: &ambient };
            let stride = result["stride"].as_u64().unwrap() as usize / 4;
            let mut dest = vec![0u32; stride * size[1] as usize];
            let key = cache_key(MODEL_CACHE_INDEX, scenario["lm_index"].as_i64().unwrap() as i32, scenario["zone"].as_i64().unwrap() as i32, slice);
            get_texture_data(&mut cache, key, &source, slice, &env, &mut dest, stride);
            let want: Vec<u32> = result["dest"].as_array().unwrap().iter().flat_map(|row| row.as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u32)).collect();
            let got: Vec<u32> = (0..size[1] as usize).flat_map(|row| dest[row * stride..row * stride + size[0] as usize].to_vec()).collect();
            let entry = cache.entries.get(&key);
            let cached = &result["cache"];
            let cache_ok = entry.is_some_and(|entry| {
                entry.flags == ints(&cached["flags"])
                    && entry.dynamic as i64 == cached["dynamic"].as_i64().unwrap()
                    && entry.pixels == cached["pixels"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u32).collect::<Vec<_>>()
                    && format!("{key:#x}") == cached["key"].as_str().unwrap()
            });
            if got != want || !cache_ok {
                failures.push(format!("cenario {number} chamada {call_number} ({}) editor={editor}: pixels_ok={} cache_ok={cache_ok}", scenario["variant"], got == want));
            }
        }
    }
    failures
}
