use glam::Vec3;
use killer_assets::client::Client;
use killer_assets::skill_effects::{Actor, Context, NPC, Phase, SELF_TARGET, SHOT_STAGE_TWO, plan, skills, steps};
use std::collections::BTreeSet;

fn actor(position: Vec3) -> Actor {
    Actor { position, rotation: [0; 3], radius: 8.0, height: 23.0, aim: [0; 3], skill_rate: 1.0, draw_scale: 1.0, mesh_origin_z: Some(4.0), mover: None }
}

#[test]
#[ignore]
fn every_projectile_the_skill_table_launches_is_an_nprojectile_with_an_acceleration() {
    let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
    let mut client = Client::open(std::path::Path::new(&root)).unwrap();
    let context = Context { caster: Some(actor(Vec3::ZERO)), target: Some(actor(Vec3::new(300.0, 0.0, 0.0))), cast_time: 1.0, ..Context::default() };
    let mut launched = BTreeSet::new();
    for skill in skills(Phase::Shot).collect::<Vec<_>>() {
        for env in [SHOT_STAGE_TWO, SHOT_STAGE_TWO | SELF_TARGET, SHOT_STAGE_TWO | NPC] {
            for planned in plan(Phase::Shot, steps(Phase::Shot, skill, env), &context) {
                if planned.projectile {
                    launched.insert(planned.class);
                }
            }
        }
    }
    assert!(launched.len() >= 10, "{launched:?}");
    for class in launched {
        let flying = client.class_is_a(class, "Engine.NProjectile").unwrap_or(false);
        let acceleration = client.class_float_defaults(class, &["AccSpeed"]).unwrap().get("AccSpeed").copied();
        println!("{class}: projétil={flying} AccSpeed={acceleration:?}");
        if flying {
            assert!(acceleration.is_some_and(|value| value > 0.0), "{class}");
        }
    }
}

#[test]
#[ignore]
fn the_default_arrow_mesh_loads_with_its_sections_and_the_arrow_class_has_the_bow_acceleration() {
    let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
    let mut client = Client::open(std::path::Path::new(&root)).unwrap();
    let part = killer_assets::pawn::Part::new("LineageWeapons.wooden_arrow_m00_et", Vec::new());
    let mesh = killer_assets::characters::load_part(&mut client, &part).expect("malha da flecha");
    assert!(!mesh.sections.is_empty() && !mesh.vertices.is_empty());
    let defaults = client.class_float_defaults("LineageEffect.NArrow", &["AccSpeed", "Speed"]).unwrap();
    assert_eq!((defaults["AccSpeed"], defaults["Speed"]), (3000.0, 1500.0));
    assert!(client.class_is_a("LineageEffect.NArrow", "Engine.NProjectile").unwrap());
}
