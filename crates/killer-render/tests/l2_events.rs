use killer_render::particles::{ParticleSystem, random_range};
use ue2_level::{Emitter, LEC_PLAY, LEC_SHOW, SpriteEmitter};

fn sprite(disabled: bool) -> SpriteEmitter {
    SpriteEmitter { class: "SpriteEmitter".into(), disabled, max_particles: 4, ..Default::default() }
}

fn system(sprites: Vec<SpriteEmitter>) -> ParticleSystem {
    ParticleSystem::new(&Emitter { sprites, ..Default::default() }, 1)
}

#[test]
fn emitter_l2_event_show_105d42c0_sets_disabled_of_every_sub_emitter_to_not_on() {
    let null = SpriteEmitter { class: String::new(), disabled: true, ..Default::default() };
    let mut system = system(vec![sprite(false), sprite(true), null]);
    system.l2_event(LEC_SHOW, 0);
    assert_eq!(system.actor.sprites.iter().map(|e| e.disabled).collect::<Vec<_>>(), [true, true, true]);
    system.l2_event(LEC_SHOW, 1);
    assert_eq!(system.actor.sprites.iter().map(|e| e.disabled).collect::<Vec<_>>(), [false, false, true]);
    system.l2_event(LEC_SHOW, 0);
    system.l2_event(LEC_SHOW, -1);
    assert_eq!(system.actor.sprites.iter().map(|e| e.disabled).collect::<Vec<_>>(), [false, false, true]);
    system.l2_event(LEC_PLAY, 0);
    system.l2_event(3, 0);
    assert_eq!(system.actor.sprites.iter().map(|e| e.disabled).collect::<Vec<_>>(), [false, false, true]);
}

#[test]
fn particle_emitter_exec_trigger_105cddd0_only_acts_on_a_disabled_sub_emitter() {
    let toggles = SpriteEmitter { spawn_on_trigger: [3.0, 3.0], ..sprite(true) };
    let stays = SpriteEmitter { trigger_disabled: false, spawn_on_trigger: [2.0, 2.0], ..sprite(true) };
    let enabled = SpriteEmitter { spawn_on_trigger: [5.0, 5.0], ..sprite(false) };
    let empty = SpriteEmitter { spawn_on_trigger: [0.0, 0.0], ..sprite(true) };
    let null = SpriteEmitter { class: String::new(), disabled: true, spawn_on_trigger: [4.0, 4.0], ..Default::default() };
    let mut system = system(vec![toggles, stays, enabled, empty, null]);
    for state in &mut system.states {
        state.all_dead = true;
        state.inactive = true;
    }
    system.trigger();
    assert_eq!(system.actor.sprites.iter().map(|e| e.disabled).collect::<Vec<_>>(), [false, true, false, false, true]);
    assert_eq!(system.states.iter().map(|s| s.trigger_count).collect::<Vec<_>>(), [3, 2, 0, 0, 0]);
    assert_eq!(system.states.iter().map(|s| (s.all_dead, s.inactive)).collect::<Vec<_>>(), [(false, false), (false, false), (true, true), (true, true), (true, true)]);
    system.trigger();
    assert_eq!(system.states.iter().map(|s| s.trigger_count).collect::<Vec<_>>(), [3, 4, 0, 0, 0]);
}

#[test]
fn particle_emitter_exec_trigger_105cddd0_resets_and_draws_the_spawn_range_from_the_shared_random() {
    let e = SpriteEmitter { reset_on_trigger: true, spawn_on_trigger: [1.0, 9.0], ..sprite(true) };
    let mut system = system(vec![e]);
    system.states[0].active = 3;
    system.states[0].trigger_count = 2;
    system.random = 77;
    let mut expected = 77;
    let delay = random_range(&mut expected, [0.0, 0.0]);
    let reset = random_range(&mut expected, [0.0, 0.0]);
    assert_eq!((delay, reset), (0.0, 0.0));
    let count = (random_range(&mut expected, [1.0, 9.0]) + 2.0) as i32 as u32;
    system.trigger();
    assert_eq!(system.states[0].active, 0);
    assert_eq!(system.states[0].trigger_count, count);
    assert_eq!(system.random, expected);
}
