use glam::Vec3;
use killer_assets::skill_effects::{Actor, Context, Phase, SELF_TARGET, SITES, Site, Step, TARGET_LIST, evaluate_site, steps};
use serde_json::Value;

const FIXTURE: &str = include_str!("data/skill_effect_oracle.json");

fn number(value: &Value, key: &str, default: f64) -> f32 {
    value.get(key).and_then(Value::as_f64).unwrap_or(default) as f32
}

fn vec3(value: &Value) -> Vec3 {
    let array = value.as_array().expect("vetor");
    Vec3::new(array[0].as_f64().expect("x") as f32, array[1].as_f64().expect("y") as f32, array[2].as_f64().expect("z") as f32)
}

fn phase(name: &str) -> Phase {
    match name {
        "init" => Phase::Cast,
        "preshot" => Phase::PreShot,
        "shot" => Phase::Shot,
        other => panic!("fase {other}"),
    }
}

fn context(geometry: &Value, bits: u8) -> Context {
    let aim = geometry["aim"].as_array().expect("aim");
    let caster = Actor {
        position: vec3(&geometry["location"]),
        rotation: [0, geometry["yaw"].as_i64().expect("yaw") as i32, 0],
        radius: number(geometry, "radius", 20.0),
        height: number(geometry, "height", 40.0),
        aim: [aim[0].as_i64().expect("aim") as i32, aim[1].as_i64().expect("aim") as i32, aim[2].as_i64().expect("aim") as i32],
        skill_rate: number(geometry, "rate", 1.0),
        draw_scale: 1.0,
        mesh_origin_z: None,
        mover: None,
    };
    let own = bits & SELF_TARGET != 0 && bits & TARGET_LIST == 0;
    let target = Actor {
        position: vec3(&geometry["target"]),
        rotation: [0, 0, 0],
        radius: number(geometry, "target_radius", 30.0),
        height: number(geometry, "target_height", 60.0),
        aim: [0, 0, 0],
        skill_rate: 0.0,
        draw_scale: 1.0,
        mesh_origin_z: None,
        mover: None,
    };
    Context { caster: Some(caster), target: Some(if own { caster } else { target }), cast_time: number(geometry, "cast", 1.5), ..Context::default() }
}

#[test]
fn every_site_expression_equals_the_arguments_the_engine_computed_in_the_oracle() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("fixture");
    let geometries = fixture["geometries"].as_array().expect("geometrias");
    let mut checked = 0;
    let mut unsupported = 0;
    let mut worst = 0.0_f32;
    let mut reasons = std::collections::BTreeSet::new();
    for (key, per_geometry) in fixture["records"].as_object().expect("registros") {
        let parts: Vec<&str> = key.split('/').collect();
        let skill = i32::from_str_radix(parts[1].trim_start_matches("0x"), 16).expect("skill");
        let bits: u8 = parts[2].parse().expect("ambiente");
        let list = steps(phase(parts[0]), skill, bits);
        let sites: Vec<&Site> = list.iter().filter_map(|step| if let Step::Spawn { site, .. } = step { Some(&SITES[usize::from(*site)]) } else { None }).collect();
        for (index, geometry) in geometries.iter().enumerate() {
            let Some(expected) = per_geometry.get(index.to_string()).and_then(Value::as_array) else { continue };
            assert_eq!(sites.len(), expected.len(), "{key} geometria {index}: quantidade de spawns");
            let context = context(geometry, bits);
            for (site, want) in sites.iter().zip(expected) {
                if bits & TARGET_LIST != 0 {
                    continue;
                }
                if let Site::Unsupported(reason) = site {
                    unsupported += 1;
                    reasons.insert(*reason);
                    continue;
                }
                let values = evaluate_site(site, &context).unwrap_or_else(|| panic!("{key} geometria {index}: avaliação falhou"));
                let want_vector = vec3(&want[3]);
                let error = (values.vector - want_vector).abs().max_element();
                worst = worst.max(error);
                assert!(error < 0.01, "{key} geometria {index}: {:?} != {want_vector:?}", values.vector);
                let rotator: Vec<i64> = want[4].as_array().expect("rotator").iter().map(|v| v.as_i64().expect("inteiro")).collect();
                for axis in 0..3 {
                    assert_eq!(i64::from(values.rotator[axis]), rotator[axis], "{key} geometria {index}: rotação {:?} != {rotator:?}", values.rotator);
                }
                checked += 1;
            }
        }
    }
    eprintln!("{checked} conferidos, {unsupported} sem leitura fora do modo lista, pior erro {worst}");
    for reason in reasons {
        eprintln!("  sem leitura: {reason}");
    }
    assert!(checked > 3000, "poucos casos conferidos: {checked}");
}
