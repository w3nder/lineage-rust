use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};
use ue2_assets::wet::{FireRandom, WaterSimulation, fire_sine};

#[test]
#[ignore]
fn wettexture_matches_fire_dll_machine_code() {
    let client = std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
    let python = std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into());
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/wettexture_reference.py");
    let kinds: Vec<u8> = (0..20).chain([64, 65, 255]).collect();
    for (width, height) in [(8, 8), (16, 32), (32, 16), (128, 128), (256, 256), (1024,8)] {
        for amplitude in [0, 1, 112, 128, 255] {
            for parity in [0, 1, 254, 255] {
                let drops: Vec<[u8; 8]> = kinds
                    .iter()
                    .enumerate()
                    .map(|(i, &kind)| {
                        [
                            kind,
                            (i * 29) as u8,
                            (i % (width / 2)) as u8,
                            ((i * 7) % (height / 2)) as u8,
                            253,
                            249,
                            17,
                            30,
                        ]
                    })
                    .collect();
                let fields: Vec<u8> = (0..width * height / 2).map(|i| ((i * 73 + i / 7 * 19) & 255) as u8).collect();
                let case = json!({"width":width,"height":height,"amplitude":amplitude,"parity":parity,"drops":drops,"fields":fields,"steps":[1,2,3,30]});
                let mut child = Command::new(&python)
                    .arg(&script)
                    .arg(&client)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("python oracle");
                writeln!(child.stdin.take().unwrap(), "{case}").unwrap();
                let output = child.wait_with_output().unwrap();
                assert!(output.status.success());
                let original: Value = serde_json::from_slice(&output.stdout).unwrap();
                let mut simulation = WaterSimulation::new(width, height, amplitude, drops).unwrap();
                simulation.fields = fields;
                simulation.parity = parity;
                assert_eq!(json!(simulation.refraction.as_slice()), original["refraction"]);
                assert_eq!(json!(fire_sine().as_slice()), original["sine"]);
                let source: Vec<u8> = (0..width * height).map(|i| i as u8).collect();
                let mut random = FireRandom::from_seed(1);
                for step in 1..=30 {
                    simulation.redraw_drops(&mut random);
                    simulation.calculate_water();
                    let expected = original["frames"].as_array().unwrap().iter().find(|v| v["step"] == step);
                    if let Some(expected) = expected {
                        let label = format!("{width}x{height} amp={amplitude} parity={parity} step={step}");
                        assert_eq!(json!(simulation.fields), expected["fields"], "fields {label}");
                        assert_eq!(json!(simulation.pixels), expected["gradients"], "gradients {label}");
                        assert_eq!(
                            json!(simulation.drops.iter().flatten().copied().collect::<Vec<_>>()),
                            expected["drops"],
                            "drops {label}"
                        );
                        assert_eq!(json!(simulation.parity), expected["parity"]);
                    }
                    assert!(simulation.apply_wet_texture(&source));
                    if let Some(expected) = expected {
                        assert_eq!(json!(simulation.pixels), expected["pixels"]);
                    }
                }
                eprintln!("native parity {width}x{height} amplitude={amplitude} initial parity={parity}");
            }
        }
    }
}
