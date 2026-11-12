use killer_assets::weapon_effects::adjust_particle_life;
use serde_json::Value;
use ue2_level::{Emitter, SpriteEmitter};

const FIXTURE: &str = include_str!("data/emitter_life_oracle.json");

fn number(value: &Value) -> f32 {
    value.as_f64().expect("número") as f32
}

#[test]
fn adjust_particle_life_matches_the_engine_executed_in_the_oracle() {
    let cases: Vec<Value> = serde_json::from_str(FIXTURE).expect("fixture");
    for (index, case) in cases.iter().enumerate() {
        let input = &case["input"];
        let mut emitter = Emitter::default();
        for sub in input["subs"].as_array().expect("subs") {
            let mut sprite = SpriteEmitter::default();
            if sub.is_null() {
                sprite.forced_life_time = false;
            } else {
                let flags = sub["flags"].as_u64().expect("flags");
                sprite.forced_life_time = flags & 1 != 0;
                sprite.forced_fade = flags & 2 != 0;
                sprite.forced_max_particles = flags & 4 != 0;
                sprite.max_particles = sub["max_particles"].as_u64().expect("max") as u32;
                sprite.lifetime = [number(&sub["lifetime"][0]), number(&sub["lifetime"][1])];
                sprite.initial_particles_per_second = number(&sub["ipps"]);
                sprite.fade_out_start = number(&sub["fade_out_start"]);
            }
            emitter.sprites.push(sprite);
        }
        adjust_particle_life(&mut emitter, number(&input["life"]));
        for (sprite, expected) in emitter.sprites.iter().zip(case["output"]["subs"].as_array().expect("saída")) {
            if expected.is_null() {
                continue;
            }
            assert_eq!(sprite.max_particles, expected["max_particles"].as_u64().expect("max") as u32, "caso {index}: partículas");
            for axis in 0..2 {
                assert!((sprite.lifetime[axis] - number(&expected["lifetime"][axis])).abs() < 1e-4, "caso {index}: vida {:?} != {}", sprite.lifetime, expected["lifetime"]);
            }
            assert!((sprite.fade_out_start - number(&expected["fade_out_start"])).abs() < 1e-4, "caso {index}: fade");
        }
    }
}
