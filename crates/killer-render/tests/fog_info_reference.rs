use glam::Vec3;
use killer_render::fog::{FogValues, affect_factor, blend_update, color_interval, is_active, lerp_byte};
use serde_json::{Value, json};
use std::{io::Write, path::PathBuf, process::{Command, Stdio}};
use ue2_level::{FogColorInfo, L2FogInfo};

struct Random(u64);

impl Random {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }

    fn byte(&mut self) -> u8 {
        (self.next() * 256.0) as u8
    }
}

fn oracle(cases: &[Value]) -> Option<Vec<Value>> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/l2_fog_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script).arg(root).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    Some(serde_json::from_slice(&output.stdout).unwrap())
}

fn fog(location: [f32; 3], range: [f32; 2], texture_distance: f32, times: &[f32]) -> L2FogInfo {
    let color = |time| FogColorInfo { time, fog: [0; 4], sky: [0; 4], cloud: Vec::new(), haze_ring: Vec::new() };
    L2FogInfo {
        location,
        affect_range: range,
        fog_range: [[0.0; 2]; 5],
        texture_distance,
        colors: times.iter().map(|&time| color(time)).collect(),
        cloud_texture: None,
    }
}

#[test]
#[ignore]
fn lerp_byte_truncates_like_the_original_lerp_byte() {
    let mut random = Random(7);
    let mut cases = Vec::new();
    let mut inputs = Vec::new();
    for index in 0..4000 {
        let (a, b) = ([random.byte(), random.byte(), random.byte(), 255], [random.byte(), random.byte(), random.byte(), 255]);
        let f = match index % 4 { 0 => 0.5, 1 => 0.0, 2 => 1.0, _ => random.next() };
        cases.push(json!({"lerp": {"a": a, "b": b, "f": f}}));
        inputs.push((a, b, f));
    }
    let Some(native) = oracle(&cases) else { return };
    let bytes = |c: [u8; 4]| [f32::from(c[0]) / 255.0, f32::from(c[1]) / 255.0, f32::from(c[2]) / 255.0];
    for ((a, b, f), native) in inputs.into_iter().zip(native) {
        let expected: Vec<u64> = native["color"].as_array().unwrap().iter().take(3).map(|v| v.as_u64().unwrap()).collect();
        let actual: Vec<u64> = lerp_byte(bytes(a), bytes(b), f).iter().map(|v| (v * 255.0).round() as u64).collect();
        assert_eq!(actual, expected, "a={a:?} b={b:?} f={f}");
    }
}

#[test]
#[ignore]
fn distance_and_time_natives_match_the_original() {
    let mut random = Random(11);
    let mut cases = Vec::new();
    let mut inputs = Vec::new();
    for index in 0..1500 {
        let times: Vec<f32> = match index % 5 {
            0 => vec![0.0, 6.0, 12.0, 18.0],
            1 => vec![1.0, 6.0, 12.0, 18.0],
            2 => vec![3.5, 9.25, 20.0],
            3 => vec![0.0, 12.0],
            _ => vec![0.0],
        };
        let location = [(random.next() - 0.5) * 20000.0, (random.next() - 0.5) * 20000.0, random.next() * 1000.0];
        let min = random.next() * 4000.0;
        let range = [min, min + random.next() * 8000.0];
        let texture_distance = random.next() * 6000.0;
        let position = [(random.next() - 0.5) * 20000.0, (random.next() - 0.5) * 20000.0, random.next() * 1000.0];
        let hour = if index % 7 == 0 { (random.next() * 24.0).floor() } else { random.next() * 24.0 };
        let info = fog(location, range, texture_distance, &times);
        cases.push(json!({"actor": {"location": location, "affect_range": range, "texture_distance": texture_distance, "times": times}, "position": position, "hour": hour}));
        inputs.push((info, position, hour));
    }
    let Some(native) = oracle(&cases) else { return };
    for (index, ((info, position, hour), native)) in inputs.into_iter().zip(native).enumerate() {
        let view = Vec3::from(position);
        let affect = native["affect"].as_f64().unwrap() as f32;
        assert!((affect_factor(&info, view) - affect).abs() <= 1e-5 * affect.abs().max(1.0), "case {index} affect");
        assert_eq!(is_active(&info, view), native["active"].as_u64().unwrap() != 0, "case {index} active");
        if info.colors.is_empty() {
            continue;
        }
        match (color_interval(&info.colors, hour), native["found"].as_u64().unwrap() != 0) {
            (Some((a, b, f)), true) => {
                assert_eq!((a.time, b.time), (native["a"].as_f64().unwrap() as f32, native["b"].as_f64().unwrap() as f32), "case {index} interval");
                let expected = native["fraction"].as_f64().map_or(f32::NAN, |value| value as f32);
                assert!((f - expected).abs() <= 1e-6 || (f.is_nan() && expected.is_nan()), "case {index} fraction {f} vs {expected}");
            }
            (None, false) => {}
            (rust, found) => panic!("case {index}: hour {hour} times {:?}: rust {} native found {found}", info.colors.iter().map(|c| c.time).collect::<Vec<_>>(), rust.is_some()),
        }
    }
}

#[test]
#[ignore]
fn distance_fog_update_matches_the_original_including_the_overshoot() {
    let mut random = Random(23);
    let mut cases = Vec::new();
    let mut inputs = Vec::new();
    let bytes = |c: [u8; 4]| [f32::from(c[0]) / 255.0, f32::from(c[1]) / 255.0, f32::from(c[2]) / 255.0];
    for index in 0..3000 {
        let saturated = |random: &mut Random| match index % 3 { 0 => [255, random.byte(), 0, 255], _ => [random.byte(), random.byte(), random.byte(), 255] };
        let (old, target) = (saturated(&mut random), saturated(&mut random));
        let (old_range, target_range) = ([random.next() * 20000.0, random.next() * 20000.0], [random.next() * 20000.0, random.next() * 20000.0]);
        let blend = 0.25 + random.next() * 2.0;
        let remaining = if index % 4 == 0 { random.next() * 0.05 } else { random.next() * blend };
        let dt = random.next() * 0.1;
        cases.push(json!({"update": {
            "old": {"color": old, "start": old_range[0], "end": old_range[1]},
            "target": {"color": target, "start": target_range[0], "end": target_range[1]},
            "blend": blend, "remaining": remaining, "dt": dt}}));
        inputs.push((FogValues { start: old_range[0], end: old_range[1], color: bytes(old) }, FogValues { start: target_range[0], end: target_range[1], color: bytes(target) }, blend, remaining, dt));
    }
    let Some(native) = oracle(&cases) else { return };
    let mut overshoots = 0;
    for (index, ((old, target, blend, remaining, dt), native)) in inputs.into_iter().zip(native).enumerate() {
        let (values, left) = blend_update(old, target, blend, remaining, dt);
        overshoots += usize::from(left < 0.0);
        let float = |key: &str| native[key].as_f64().unwrap() as f32;
        assert_eq!(left, float("remaining"), "case {index} remaining");
        assert_eq!((values.start, values.end), (float("start"), float("end")), "case {index}");
        let color: Vec<u64> = native["color"].as_array().unwrap().iter().take(3).map(|v| v.as_u64().unwrap()).collect();
        let actual: Vec<u64> = values.color.iter().map(|v| (v * 255.0).round() as u64).collect();
        assert_eq!(actual, color, "case {index}: {old:?} -> {target:?} left {left}");
    }
    assert!(overshoots > 0);
}
