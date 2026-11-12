use killer_assets::lightmap_tables::{attenuation, shadow_colors};
use serde_json::Value;

const FIXTURE: &str = include_str!("data/lightmap_tables_oracle.json");

#[test]
fn fun_106345f0_tables_match_the_engine_constructor_run_in_the_emulator() {
    let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
    let expected: Vec<f32> = fixture["attenuation"].as_array().unwrap().iter().map(|v| f32::from_bits(v.as_u64().unwrap() as u32)).collect();
    let ours = attenuation();
    assert_eq!(ours.len(), expected.len());
    for (index, (a, b)) in ours.iter().zip(&expected).enumerate() {
        assert!((a - b).abs() <= b.abs() * 2.0e-6 + 1.0e-6, "{index}: {a} != {b}");
    }
    let colors: Vec<u32> = fixture["colors"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u32).collect();
    let ours: Vec<u32> = shadow_colors().into_iter().flatten().collect();
    assert_eq!(ours, colors);
}

#[test]
fn fast_root_tables_match_the_engine_constructor_run_in_the_emulator() {
    use killer_assets::lightmap_tables::{inverse_root_exponent, inverse_root_mantissa, sqrt_table};
    let fixture: Value = serde_json::from_str(FIXTURE).unwrap();
    let words = |key: &str| -> Vec<u32> { fixture[key].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u32).collect() };
    assert_eq!(sqrt_table(), words("sqrt"));
    let mantissa: Vec<u32> = inverse_root_mantissa().into_iter().map(f32::to_bits).collect();
    assert_eq!(mantissa, words("inverse_root_mantissa"));
    let exponent: Vec<u32> = inverse_root_exponent().into_iter().map(f32::to_bits).collect();
    let expected = words("inverse_root_exponent");
    for (index, (got, want)) in exponent.iter().zip(&expected).enumerate() {
        if matches!(index, 0 | 255 | 256 | 511) {
            continue;
        }
        assert_eq!(got, want, "{index}");
    }
    assert!(f32::from_bits(exponent[0]).is_infinite() && f32::from_bits(exponent[256]).is_infinite());
    assert_eq!((exponent[255], exponent[511]), (0, 0));
}
