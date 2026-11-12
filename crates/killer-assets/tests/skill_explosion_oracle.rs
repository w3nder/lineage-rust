use glam::Vec3;
use killer_assets::skill_effects::explosion::{EXPLOSION_AREA, EXPLOSION_HIT_MOVER, EXPLOSION_HIT_NULL, EXPLOSION_OWNER_NULL, EXPLOSION_SELF_HIT, SITES, evaluate_site, plan, steps};
use killer_assets::skill_effects::{Actor, Context, Step};
use serde_json::Value;

const FIXTURE: &str = include_str!("data/skill_explosion_oracle.json");

fn number(value: &Value, key: &str, default: f64) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(default)
}

fn triple(value: &Value) -> [f64; 3] {
    let array = value.as_array().expect("vetor");
    [0, 1, 2].map(|axis| array[axis].as_f64().expect("número"))
}

fn round2(value: f64) -> f64 {
    format!("{value:.2}").parse().expect("número")
}

fn vec3(values: [f64; 3]) -> Vec3 {
    Vec3::new(values[0] as f32, values[1] as f32, values[2] as f32)
}

fn case(geometry: &Value, bits: u8) -> (Context, Option<Actor>) {
    let caster_location = triple(&geometry["location"]);
    let target_location = triple(&geometry["target"]);
    let height = number(geometry, "target_height", 60.0);
    let yaw = geometry["yaw"].as_i64().expect("yaw") as i32;
    let impact = [target_location[0], target_location[1], round2(target_location[2] + height * 0.25)];
    let delta = [0, 1, 2].map(|axis| impact[axis] - caster_location[axis]);
    let size = delta.iter().map(|value| value * value).sum::<f64>().sqrt();
    let size = if size == 0.0 { 1.0 } else { size };
    let caster = Actor {
        position: vec3(caster_location),
        rotation: [0, yaw, 0],
        radius: number(geometry, "radius", 20.0) as f32,
        height: number(geometry, "height", 40.0) as f32,
        skill_rate: number(geometry, "rate", 1.0) as f32,
        draw_scale: 1.0,
        ..Actor::default()
    };
    let target = Actor {
        position: vec3(target_location),
        rotation: [0, (yaw + 32768) & 0xffff, 0],
        radius: number(geometry, "target_radius", 30.0) as f32,
        height: height as f32,
        draw_scale: 1.0,
        mover: (bits & EXPLOSION_HIT_MOVER != 0).then(|| vec3([round2(target_location[1] * 0.1 + 3.0), round2(target_location[0] * 0.1 - 7.0), round2(height * 0.05)])),
        ..Actor::default()
    };
    let hit = if bits & EXPLOSION_HIT_NULL != 0 {
        None
    } else if bits & EXPLOSION_SELF_HIT != 0 {
        Some(caster)
    } else {
        Some(target)
    };
    let area = (bits & EXPLOSION_AREA != 0).then(|| Actor {
        position: vec3([target_location[0] + 90.0, target_location[1] - 40.0, target_location[2]]),
        radius: (number(geometry, "target_radius", 30.0) * 0.5) as f32,
        height: height as f32,
        draw_scale: 1.0,
        ..Actor::default()
    });
    let context = Context {
        caster: (bits & EXPLOSION_OWNER_NULL == 0).then_some(caster),
        target: hit,
        projectile: Some(Actor { position: vec3(impact), rotation: [0, yaw, 0], ..Actor::default() }),
        hit,
        normal: vec3(delta.map(|value| -value / size)),
        ..Context::default()
    };
    (context, area)
}

#[test]
fn every_explosion_site_equals_the_arguments_the_engine_passed_in_the_oracle() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("fixture");
    let geometries = fixture["geometries"].as_array().expect("geometrias");
    let mut checked = 0;
    let mut worst = 0.0_f32;
    for (key, per_geometry) in fixture["records"].as_object().expect("registros") {
        let parts: Vec<&str> = key.split('/').collect();
        let skill = i32::from_str_radix(parts[1].trim_start_matches("0x"), 16).expect("skill");
        let bits: u8 = parts[2].parse().expect("ambiente");
        let list = steps(bits, skill);
        for (index, geometry) in geometries.iter().enumerate() {
            let Some(expected) = per_geometry.get(index.to_string()).and_then(Value::as_array) else { continue };
            let (base, area) = case(geometry, bits);
            let context = Context { area, ..base };
            let sites: Vec<(&str, u16)> = list
                .iter()
                .filter_map(|step| match step {
                    Step::Spawn { site, .. } => Some(("spawn", *site)),
                    Step::DamageEffect { site } => Some(("spawn_actor", *site)),
                    Step::PawnLight { site } => Some(("add_pawn_light", *site)),
                    _ => None,
                })
                .collect();
            assert_eq!(sites.len(), expected.len(), "{key} geometria {index}: quantidade de chamadas");
            for ((call, site), want) in sites.iter().zip(expected) {
                let want_call = want[0].as_str().expect("chamada");
                assert!(*call == want_call || (*call == "spawn" && want_call == "spawn_actor"), "{key} geometria {index}: {call} != {want_call}");
                let values = evaluate_site(&SITES[usize::from(*site)], &context).unwrap_or_else(|| panic!("{key} geometria {index}: avaliação falhou"));
                let want_vector = vec3(triple(&want[3]));
                let error = (values.vector - want_vector).abs().max_element();
                worst = worst.max(error);
                assert!(error < 0.001, "{key} geometria {index}: {:?} != {want_vector:?}", values.vector);
                let rotator: Vec<i64> = want[4].as_array().expect("rotator").iter().map(|v| v.as_i64().expect("inteiro")).collect();
                for axis in 0..3 {
                    assert_eq!(i64::from(values.rotator[axis]), rotator[axis], "{key} geometria {index}: rotação {:?} != {rotator:?}", values.rotator);
                }
                checked += 1;
            }
            let planned = plan(list, &base, area.as_slice());
            assert_eq!(planned.spawns.len() + planned.lights.len(), expected.len(), "{key} geometria {index}: plano");
        }
    }
    eprintln!("{checked} conferidos, pior erro {worst}");
    assert!(checked > 14000, "poucos casos conferidos: {checked}");
}
