use killer_assets::client::Client;
use std::path::PathBuf;

fn test_client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

#[test]
#[ignore]
fn giran_lightmap_bitmaps_follow_the_layout_the_generator_reads() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let render = &loaded.level.bsp_render;
    let mut with_bitmaps = 0;
    let mut sequences = std::collections::BTreeMap::new();
    for (index, light_map) in render.light_maps.iter().enumerate() {
        let (sx, sy) = (light_map.size[0], light_map.size[1]);
        for bitmap in &light_map.bitmaps {
            assert_eq!(bitmap.bits.len() as i32, bitmap.stride * bitmap.size[1], "lightmap {index}");
            assert!(bitmap.min[0] >= 0 && bitmap.min[1] >= 0 && bitmap.max[0] < sx && bitmap.max[1] < sy, "lightmap {index} {bitmap:?}");
            assert!(bitmap.stride * 8 > bitmap.max[0] - bitmap.min[0], "lightmap {index}");
            assert_eq!(bitmap.size[1], bitmap.max[1] - bitmap.min[1] + 1, "lightmap {index}");
        }
        if !light_map.bitmaps.is_empty() {
            with_bitmaps += 1;
        }
        let mut run = 0;
        for pair in light_map.bitmaps.windows(2) {
            run = if pair[0].actor == pair[1].actor { run + 1 } else { 0 };
            *sequences.entry(run).or_insert(0usize) += 1;
        }
    }
    eprintln!("lightmaps {} com bitmaps {with_bitmaps}; superficies {}; execucoes de mesma luz {sequences:?}", render.light_maps.len(), render.surfaces.len());
}

#[test]
#[ignore]
fn zone_info_class_defaults_give_the_ambient_color_the_generator_reads() {
    let Some(mut client) = test_client() else { return };
    let file = client.read_system_file("Engine.u").unwrap();
    let package = ue2_package::Package::parse(&file).unwrap();
    let export = package.exports().iter().find(|export| package.class_name(export) == "Class" && package.name(export.name) == "ZoneInfo").unwrap();
    let properties = ue2_assets::class_default_properties(&package, &file, export).unwrap();
    for name in ["AmbientBrightness", "AmbientHue", "AmbientSaturation"] {
        eprintln!("ZoneInfo.{name} = {:?}", ue2_assets::find(&properties, name).and_then(|property| property.byte()));
    }
}

mod oracle_input {
    use killer_assets::lightmap_texture::LightMapSource;
    use serde_json::{Value, json};

    pub fn actor_json(entry: &killer_assets::lightmap_texture::BitmapEntry) -> Value {
        let Some(actor) = entry.light.actor.as_ref().filter(|_| entry.actor_present) else { return Value::Null };
        json!({
            "effect": actor.effect, "period": actor.period, "phase": actor.phase, "cone": actor.cone, "time": actor.time_seconds,
            "rotation": actor.rotation, "deleted": false, "editor_hidden": entry.editor_hidden, "controller": actor.controller_rotation,
        })
    }

    pub fn scenario(source: &LightMapSource, lm_index: usize, zone: i32, slice: usize, count: usize, hour: f32, deleted: &[Vec<bool>]) -> Value {
        let bitmaps: Vec<Value> = source
            .bitmaps
            .iter()
            .map(|entry| {
                let light = &entry.light;
                json!({
                    "actor": actor_json(entry),
                    "light": {"pos": light.position, "dir": light.direction, "radius": light.radius, "dynamic": light.dynamic, "color": light.color},
                    "bits": entry.bitmap.bits, "rows": entry.bitmap.rows, "stride": entry.bitmap.stride,
                    "bounds": [entry.bitmap.min_x, entry.bitmap.min_y, entry.bitmap.max_x, entry.bitmap.max_y], "time": entry.time,
                })
            })
            .collect();
        let calls: Vec<Value> = deleted
            .iter()
            .map(|flags| json!({"editor": false, "time_index": slice, "time_count": count, "grow": [0, 0], "deleted": flags, "env_time": hour, "level_time": 0.0}))
            .collect();
        let matrix: Vec<f32> = source.matrix.iter().flatten().copied().collect();
        json!({
            "variant": "timed", "size": source.size, "lm_index": lm_index, "zone": zone, "surf_index": 0,
            "surf": {"flags": source.surface.flags, "normal": source.surface.normal, "plane": source.surface.distance},
            "geometry": {"matrix": matrix, "base": source.geometry.base, "axis_x": source.geometry.axis_x, "axis_y": source.geometry.axis_y},
            "zone_actor": {"sun": source.zone.sun, "hsv": [source.zone.hue, source.zone.saturation, source.zone.brightness]},
            "first_actor": {"sun": true, "hsv": [0, 255, 0]},
            "bitmaps": bitmaps, "dynamic": [], "calls": calls, "results": [],
        })
    }
}

fn stand_in_sun(hour: f32) -> [f32; 3] {
    let angle = hour * std::f32::consts::PI / 12.0;
    [angle.cos() * 0.6, angle.sin() * 0.6, -0.8]
}

fn picks(level: &ue2_level::Level, regeneration: &killer_assets::scene::lightmap_regen::Regeneration) -> Vec<(usize, usize)> {
    let mut picked = Vec::new();
    let mut sun_zone = 0;
    let mut other = 0;
    for (index, light_map) in level.bsp_render.light_maps.iter().enumerate() {
        let small = light_map.size[0] <= 32 && light_map.size[1] <= 32;
        let bitmaps = light_map.bitmaps.len();
        let sun = regeneration.zone_light(light_map.zone).sun;
        if small && (6..=36).contains(&bitmaps) {
            if sun && sun_zone < 3 && sun_zone < picked.len() + 1 {
                picked.push((index, 4 + sun_zone % 2));
                sun_zone += 1;
            } else if !sun && other < 4 {
                picked.push((index, 3));
                other += 1;
            }
        }
    }
    picked
}

#[test]
#[ignore]
fn giran_lightmaps_for_the_emulator_oracle() {
    use killer_assets::scene::lightmap_regen::{Regeneration, slice_hour};
    use killer_assets::timeenv::TimeEnv;
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let env = TimeEnv::load(&mut client, 0).unwrap();
    let sun = stand_in_sun;
    let regeneration = Regeneration::new(&loaded.level, &env, &sun);
    let mut scenarios = Vec::new();
    for (index, slice) in picks(&loaded.level, &regeneration) {
        let hour = slice_hour(slice, 8);
        let source = regeneration.source(index, hour, &|_| false).unwrap();
        let count = source.bitmaps.len();
        let lit = |entry: &killer_assets::lightmap_texture::BitmapEntry| entry.bitmap.bits.iter().map(|byte| byte.count_ones()).sum::<u32>();
        let target = (0..count).filter(|&bitmap| source.bitmaps[bitmap].time == slice as i32 || !source.zone.sun).max_by_key(|&bitmap| lit(&source.bitmaps[bitmap])).unwrap_or(0);
        let changed: Vec<bool> = (0..count).map(|bitmap| bitmap == target).collect();
        let zone = loaded.level.bsp_render.light_maps[index].zone;
        scenarios.push(oracle_input::scenario(&source, index, zone, slice, 8, hour, &[vec![false; count], changed]));
    }
    let Ok(path) = std::env::var("LIGHTMAP_ORACLE_INPUT") else { return };
    std::fs::write(path, serde_json::to_string(&serde_json::json!({"texture": scenarios})).unwrap()).unwrap();
}

fn same_json(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    use serde_json::Value;
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64().zip(y.as_f64()).is_some_and(|(x, y)| x as f32 == y as f32),
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(x, y)| same_json(x, y)),
        (Value::Object(x), Value::Object(y)) => x.keys().filter(|key| *key != "results").eq(y.keys().filter(|key| *key != "results")) && x.iter().filter(|(key, _)| *key != "results").all(|(key, value)| same_json(value, &y[key])),
        _ => a == b,
    }
}

#[test]
#[ignore]
fn the_committed_oracle_inputs_are_what_the_app_builds_from_giran() {
    use killer_assets::scene::lightmap_regen::{Regeneration, slice_hour};
    use killer_assets::timeenv::TimeEnv;
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let env = TimeEnv::load(&mut client, 0).unwrap();
    let sun = stand_in_sun;
    let regeneration = Regeneration::new(&loaded.level, &env, &sun);
    let fixture: serde_json::Value = serde_json::from_str(include_str!("data/lightmap_real_oracle.json")).unwrap();
    let committed = fixture["texture"].as_array().unwrap();
    let selection = picks(&loaded.level, &regeneration);
    assert_eq!(selection.len(), committed.len());
    for ((index, slice), expected) in selection.into_iter().zip(committed) {
        let hour = slice_hour(slice, 8);
        let source = regeneration.source(index, hour, &|_| false).unwrap();
        let count = source.bitmaps.len();
        let lit = |entry: &killer_assets::lightmap_texture::BitmapEntry| entry.bitmap.bits.iter().map(|byte| byte.count_ones()).sum::<u32>();
        let target = (0..count).filter(|&bitmap| source.bitmaps[bitmap].time == slice as i32 || !source.zone.sun).max_by_key(|&bitmap| lit(&source.bitmaps[bitmap])).unwrap_or(0);
        let changed: Vec<bool> = (0..count).map(|bitmap| bitmap == target).collect();
        let zone = loaded.level.bsp_render.light_maps[index].zone;
        let built = oracle_input::scenario(&source, index, zone, slice, 8, hour, &[vec![false; count], changed]);
        assert!(same_json(&built, expected), "lightmap {index}");
    }
}
