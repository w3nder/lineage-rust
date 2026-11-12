use l2_catalog::{Oscillator, Trigger, UvModifier};
use serde_json::Value;

const FIXTURE: &str = include_str!("data/tex_triggered_oracle.json");

fn floats(value: &Value) -> Vec<f32> {
    value.as_array().unwrap().iter().map(|v| v.as_f64().unwrap() as f32).collect()
}

fn bits(value: &Value) -> Vec<f32> {
    value.as_array().unwrap().iter().map(|v| f32::from_bits(v.as_u64().unwrap() as u32)).collect()
}

fn rotator(value: &Value) -> [i32; 3] {
    let v = value.as_array().unwrap();
    [v[0].as_i64().unwrap() as i32, v[1].as_i64().unwrap() as i32, v[2].as_i64().unwrap() as i32]
}

fn close(got: f32, want: f32, wrap: f32) -> bool {
    let tolerance = if wrap > 0.0 { 1.0e-2 } else { 2.0e-3 } + want.abs() * 1.0e-5;
    let difference = (got - want).abs();
    difference <= tolerance || (wrap > 0.0 && ((difference + tolerance) % wrap <= 2.0 * tolerance || wrap - (difference % wrap) <= 2.0 * tolerance))
}

fn compare(modifier: UvModifier, time: f32, expected: &[f32], label: &str, wrap: f32) {
    let matrix = modifier.matrix(time).0;
    for row in 0..3 {
        for column in 0..3 {
            let want = expected[row * 4 + column];
            let got = matrix[row][column];
            let same = if want.is_nan() { got.is_nan() } else if want.is_infinite() { got == want } else { close(got, want, wrap) };
            assert!(same, "{label} [{row}][{column}] {got} != {want}");
        }
    }
}

#[test]
fn tex_pannertriggered_getmatrix_10307d74_matches_the_engine_for_every_oracle_case() {
    let cases: Value = serde_json::from_str(FIXTURE).unwrap();
    let cases = cases["panner"].as_array().unwrap();
    assert_eq!(cases.len(), 600);
    for (index, case) in cases.iter().enumerate() {
        let trigger = Trigger {
            stop_after: case["stop"].as_f64().unwrap() as f32,
            triggered_time: case["triggered"].as_f64().unwrap() as f32,
            reverse: case["reverse"].as_bool().unwrap(),
        };
        let modifier = UvModifier::PannerTriggered { direction: rotator(&case["rotator"]), rate: case["rate"].as_f64().unwrap() as f32, trigger };
        compare(modifier, case["time"].as_f64().unwrap() as f32, &bits(&case["matrix"]), &format!("panner {index}"), 1024.0);
    }
}

#[test]
fn tex_oscillatortriggered_getmatrix_1030dd69_matches_the_engine_for_every_oracle_case() {
    let cases: Value = serde_json::from_str(FIXTURE).unwrap();
    let cases = cases["oscillator"].as_array().unwrap();
    assert_eq!(cases.len(), 600);
    for (index, case) in cases.iter().enumerate() {
        let pair = |name: &str| -> [f32; 2] {
            let v = floats(&case[name]);
            [v[0], v[1]]
        };
        let kinds = case["kinds"].as_array().unwrap();
        let oscillator = Oscillator {
            rate: pair("rate"),
            phase: pair("phase"),
            amplitude: pair("amplitude"),
            kind: [kinds[0].as_u64().unwrap() as u8, kinds[1].as_u64().unwrap() as u8],
            offset: [0.0; 2],
        };
        let trigger = Trigger {
            stop_after: case["stop"].as_f64().unwrap() as f32,
            triggered_time: case["triggered"].as_f64().unwrap() as f32,
            reverse: case["reverse"].as_bool().unwrap(),
        };
        compare(UvModifier::OscillatorTriggered(oscillator, [64.0, 64.0], trigger), case["time"].as_f64().unwrap() as f32, &bits(&case["matrix"]), &format!("oscillator {index}"), 0.0);
    }
}
