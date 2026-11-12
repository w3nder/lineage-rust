use killer_render::display_modes::{DisplayMode, refresh_rates, resolutions};
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore]
fn get_res_10011bb0_and_get_refresh_rate_10011db0_match_original_instructions() {
    let mut cases = Vec::new();
    let edge_modes = vec![
        [800_u32, 600, 16, 85], [800, 600, 32, 60], [800, 600, 32, 60],
        [1024, 768, 32, 75], [799, 600, 32, 60], [800, 599, 32, 60],
        [640, 480, 32, 0], [1024, 768, 24, 120], [u32::MAX, u32::MAX, 32, 60],
    ];
    let crowded: Vec<_> = [16, 32].into_iter().flat_map(|bits| (0..120).map(move |i| [800 + i, 600, bits, 60 + i % 3 * 15])).collect();
    let duplicates = vec![[800, 600, 32, 60]; 150];
    for modes in [vec![], edge_modes, crowded, duplicates] {
        for capacity in [0, 1, 2, 99, 100, 119, 120, 121, 300] {
            for requested in [[800, 600, 32], [800, 600, 16], [640, 480, 32], [1024, 768, 24], [-1, -1, 32]] {
                cases.push(json!({ "modes": modes, "capacity": capacity, "requested": requested }));
            }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/display_modes_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{:?}", output.status);
    let reference: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(reference.len(), cases.len());
    for (index, (case, reference)) in cases.iter().zip(reference).enumerate() {
        let modes: Vec<[u32; 4]> = serde_json::from_value(case["modes"].clone()).unwrap();
        let modes: Vec<_> = modes.into_iter().map(|[width, height, bits, refresh]| DisplayMode { width, height, bits: bits as i32, refresh: refresh as i32 }).collect();
        let capacity = case["capacity"].as_u64().unwrap() as usize;
        let requested: (i32, i32, i32) = serde_json::from_value(case["requested"].clone()).unwrap();
        assert_eq!(json!(resolutions(&modes, capacity)), reference["resolutions"], "GetRes case {index}");
        assert_eq!(json!(refresh_rates(&modes, requested, capacity)), reference["rates"], "GetRefreshRate case {index}");
    }
}
