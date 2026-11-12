use killer_assets::skeletal_lod::{ProjectedFactorInput, projected_factor};
use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore]
fn render_1069a520_projected_factor_matches_original_arithmetic() {
    let baseline = ProjectedFactorInput {
        soft_wedges: 1221,
        screen_x: 1558,
        half_fov_tangent: 1.0,
        projected_w: 1000.0,
        z_displace: 0.0,
        strength: 1.0,
        scale_max: 1.0,
        draw_scale: 1.0,
        lod_bias: 1.0,
        max_draw_scale_3d: 1.0,
    };
    let mut inputs = Vec::new();
    for screen_x in [i32::MIN, -1, 0, 819, 820, 821, 1024, 1558, 3840, i32::MAX] {
        for soft_wedges in [i32::MIN, -1, 0, 1, 1221, i32::MAX] {
            for bits in
                [0, 0x80000000, 0x3f7fffff, 0x3f800000, 0x3f800001, 0x447a0000, 0x7f800000, 0xff800000, 0x7fc00000]
            {
                inputs.push(ProjectedFactorInput {
                    screen_x,
                    soft_wedges,
                    projected_w: f32::from_bits(bits),
                    ..baseline
                });
            }
        }
    }
    let mut seed = 0x87654321u32;
    for _ in 0..4096 {
        let mut next = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            f32::from_bits(seed)
        };
        inputs.push(ProjectedFactorInput {
            projected_w: next(),
            z_displace: next(),
            strength: next(),
            scale_max: next(),
            draw_scale: next(),
            lod_bias: next(),
            max_draw_scale_3d: next(),
            half_fov_tangent: next(),
            ..baseline
        });
    }
    let cases: Vec<_> = inputs
        .iter()
        .map(|input| {
            json!({
                "soft_wedges": input.soft_wedges, "screen_x": input.screen_x,
                "half_fov_tangent": input.half_fov_tangent.to_bits(),
                "projected_w": input.projected_w.to_bits(), "z_displace": input.z_displace.to_bits(),
                "strength": input.strength.to_bits(), "scale_max": input.scale_max.to_bits(),
        "draw_scale": input.draw_scale.to_bits(), "actor_factor_144": input.lod_bias.to_bits(),
                "max_draw_scale_3d": input.max_draw_scale_3d.to_bits(),
            })
        })
        .collect();
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/skeletal_factor_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&serde_json::to_vec(&cases).unwrap()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let native: Vec<u32> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(native.len(), inputs.len());
    for (input, native) in inputs.iter().zip(native) {
        let actual = projected_factor(*input);
        if f32::from_bits(native).is_nan() {
            assert!(actual.is_nan(), "{input:?}");
        } else {
            assert_eq!(actual.to_bits(), native, "{input:?}");
        }
    }
    eprintln!("{} fatores comparados com engine.dll", inputs.len());
}
