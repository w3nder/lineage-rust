use glam::Vec3;
use killer_assets::scene::emitters::{EmitterAsset, EmitterPart};
use killer_render::particles::{EmptyWorld, ParticleSystem, random_range, random_vector};
use ue2_level::{DrawStyle, Emitter, SpriteEmitter};

fn fixture(case: usize) -> EmitterAsset {
    let e = SpriteEmitter {
        coordinate_system: [0, 1, 4][case % 3], max_particles: 12,
        start_location: [[5.0; 2], [2.0; 2], [1.0; 2]],
        start_velocity: [[10.0; 2], [4.0; 2], [3.0; 2]], acceleration: [0.0, 0.0, 2.0],
        start_size: [[5.0; 2], [7.0; 2], [9.0; 2]], direction: (case % 8) as u8,
        use_size_scale: true, use_regular_size_scale: case.is_multiple_of(2), size_scale: vec![[0.0, 1.0], [1.0, 2.0]], size_scale_repeats: 1.0,
        use_color_scale: true, color_scale: vec![(0.0, [255; 4]), (1.0, [64, 128, 255, 128])],
        fade_in: true, fade_in_end: 0.5, fade_out: true, fade_out_start: 2.0, opacity: 0.5,
        subdivisions: [2, 4], blend_subdivisions: case.is_multiple_of(2),
        draw_style: [DrawStyle::Regular, DrawStyle::AlphaBlend, DrawStyle::Modulated, DrawStyle::Translucent, DrawStyle::AlphaModulate, DrawStyle::Darken, DrawStyle::Brighten][case % 7],
        ..Default::default()
    };
    EmitterAsset { definition: Emitter { location: [100.0, 200.0, 300.0], rotation: [0, 16384, 0], sprites: vec![e], ..Default::default() }, parts: vec![EmitterPart::default()], zone: 0, range: 12288.0, lighting: None }
}

#[test]
fn particle_emitter_105ce750_105d0170_twenty_synthetic_cases_against_python() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts/particle_reference.py");
    let output = std::process::Command::new("python3").arg(script).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let text = String::from_utf8(output.stdout).unwrap();
    let expected: Vec<Vec<f32>> = text.lines().map(|line| line.split_whitespace().map(|v| v.parse().unwrap()).collect()).collect();
    let mut checked = 0;
    for case in 0..20 {
        let asset = fixture(case);
        let mut system = ParticleSystem::new(&asset.definition, 1);
        for tick in 1..=360 {
            system.tick(1.0 / 30.0, true, &asset, &mut EmptyWorld);
            if ![15, 60, 300].contains(&tick) { continue; }
            let rows: Vec<_> = expected.iter().filter(|r| r[0] as usize == case && r[1] as usize == tick).collect();
            assert_eq!(system.states[0].active, rows.len());
            for row in rows {
                let p = &system.states[0].particles[row[2] as usize];
                let position = Vec3::from_slice(&row[3..6]);
                let size = Vec3::from_slice(&row[6..9]);
                assert!((p.location - position).abs().max_element() < 1e-3, "case={case}, tick={tick}, slot={} actual={:?} expected={position:?}", row[2], p.location);
                assert!((p.size - size).abs().max_element() < 1e-3);
                for (channel, reference) in p.color.iter().zip(&row[9..13]) { assert!((f32::from(*channel) - reference).abs() <= 1.0, "case={case}, tick={tick} {:?} {:?}", p.color, &row[9..13]); }
                checked += 1;
            }
        }
        assert_eq!(system.states[0].particles.iter().filter(|p| p.alive()).count(), 12);
    }
    assert!(checked >= 300);
}

#[test]
fn frange_1010fb20_vector_101101e0_preserve_random_order() {
    let mut state = 1;
    let mut reference = 1;
    let ranges = [[1.0, 10.0], [2.0, 20.0], [3.0, 30.0]];
    let y = random_range(&mut reference, ranges[1]);
    let x = random_range(&mut reference, ranges[0]);
    let z = random_range(&mut reference, ranges[2]);
    assert_eq!(random_vector(&mut state, ranges), Vec3::new(x, y, z));
    assert_eq!(state, reference);
    let before = state;
    assert_eq!(random_range(&mut state, [4.0; 2]), 4.0);
    assert_ne!(state, before);
    let mut seeded = 1;
    let bits = 1_u32.wrapping_mul(0x0bb38435).wrapping_add(0x3619636b) & 0x7fffff;
    assert_eq!(killer_render::particles::app_srand(&mut seeded), bits as f32 / 8388608.0);
    let rotation = killer_render::particles::rotator_matrix;
    assert_eq!(rotation([100, 1200, 4000]), rotation([103, 1203, 4003]));
    assert_ne!(rotation([100, 1200, 4000]), rotation([104, 1204, 4004]));
}

#[test]
fn sprite_106aa500_all_eight_axes_spin_and_column_major_atlas() {
    use killer_render::{camera::Camera, particles::{Particle, sprite_vertices}};
    let camera = Camera { eye: Vec3::new(0.0, -1000.0, 0.0), target: Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 10000.0 };
    let p = Particle { location: Vec3::ZERO, old_location: -Vec3::X, size: Vec3::new(50.0, 20.0, 1.0), lifetime: 8.0, time: 3.5, subdivision: -1, color: [255; 4], ..Default::default() };
    for direction in 0..8 {
        let e = SpriteEmitter { coordinate_system: 0, direction, projection_normal: [0.0, 1.0, 0.0], subdivisions: [2, 4], blend_subdivisions: true, ..Default::default() };
        let actor = Emitter::default();
        let vertices = sprite_vertices(&e, &actor, &p, &camera, 1.0);
        let a = (Vec3::from(vertices[2].position) - Vec3::from(vertices[0].position)) * 0.5;
        let b = (Vec3::from(vertices[1].position) - Vec3::from(vertices[0].position)) * 0.5;
        assert!((a.length() - if direction == 0 { 50.0 } else { 20.0 }).abs() < 1e-3, "direction={direction}: {a:?}");
        assert!((b.length() - 50.0).abs() < 1e-3);
        assert!(a.dot(b).abs() < 1e-3);
        assert_eq!(vertices[2].uv, [0.0, 0.75]);
        assert_eq!(vertices[2].mask_uv, [0.5, 0.0]);
        assert_eq!(vertices[2].color[3], 127.0 / 255.0);
        let spun = sprite_vertices(&SpriteEmitter { spin_particles: true, ..e }, &actor, &Particle { start_spin: Vec3::new(16384.0, 123.0, 456.0), ..p.clone() }, &camera, 1.0);
        let before = Vec3::from(vertices[0].position).length();
        let after = Vec3::from(spun[0].position).length();
        assert!((before - after).abs() < 1e-3);
    }
}

#[test]
fn mesh_emitter_105b0700_scale_prepivot_spin_and_coordinate_systems() {
    use killer_render::particles::{Particle, mesh_matrix};
    let actor = Emitter { pre_pivot: [1.0, 0.0, 0.0], location: [100.0, 200.0, 300.0], rotation: [0, 16384, 0], ..Default::default() };
    let p = Particle { location: Vec3::new(10.0, 20.0, 30.0), size: Vec3::new(2.0, 3.0, 4.0), rotation: [0, 16384, 0], ..Default::default() };
    for (coordinate_system, expected) in [(0, [12.0, 20.0, 30.0]), (1, [80.0, 212.0, 330.0]), (2, [12.0, 20.0, 30.0]), (3, [12.0, 20.0, 30.0]), (4, [10.0, 22.0, 30.0])] {
        let e = SpriteEmitter { coordinate_system, ..Default::default() };
        let actual = mesh_matrix(&e, &actor, &p).transform_point3(Vec3::new(2.0, 0.0, 0.0));
        assert!((actual - Vec3::from(expected)).length() < 1e-3, "{coordinate_system}: {actual:?}");
    }
    let e = SpriteEmitter { coordinate_system: 0, start_size: [[2.0; 2], [3.0; 2], [4.0; 2]], use_size_scale: true, use_regular_size_scale: false, size_scale: vec![[1.0, 2.0]], ..Default::default() };
    let bounds = killer_render::particles::mesh_bounds(&e, &actor, &[Particle { flags: 1, ..p }], [[-1.0, -2.0, -3.0], [3.0, 2.0, 5.0]]).unwrap();
    assert!((bounds.min - Vec3::new(4.2, 5.8, 1.8)).length() < 1e-4);
    assert!((bounds.max - Vec3::new(23.8, 34.2, 74.2)).length() < 1e-4);
}

#[test]
fn beam_emitter_10504c30_six_endpoints_and_105065f0_sheets() {
    use killer_render::particles::{Collision, ParticleWorld, beam_vertices};
    use ue2_level::{BeamEndpoint, BeamShape};
    struct World { triggers: usize, traces: usize }
    impl ParticleWorld for World {
        fn trigger(&mut self, tag: &str) { assert_eq!(tag, "target"); self.triggers += 1; }
        fn trace(&mut self, from: Vec3, to: Vec3, flags: u32) -> Option<Collision> { assert_eq!(flags, 0xdf); self.traces += 1; Some(Collision { location: from.lerp(to, 0.5), normal: Vec3::Z }) }
    }
    for endpoint_by in 0..6 {
        let beam = BeamShape { endpoint_by, distance: [50.0; 2], low_points: 3, high_points: 5, rotating_sheets: 2, trigger_endpoint: true, endpoints: vec![BeamEndpoint { actor_tag: Some("target".into()), offset: [[100.0; 2], [0.0; 2], [0.0; 2]], weight: 1.0 }], ..Default::default() };
        let e = SpriteEmitter { class: "BeamEmitter".into(), coordinate_system: 0, start_offset: [10.0, 0.0, 0.0], start_velocity: [[10.0; 2], [0.0; 2], [0.0; 2]], lifetime: [4.0; 2], beam: Some(beam), ..Default::default() };
        let asset = EmitterAsset { definition: Emitter { sprites: vec![e.clone()], ..Default::default() }, parts: vec![EmitterPart { targets: vec![Some([70.0, 0.0, 0.0])], ..Default::default() }], zone: 0, range: 12000.0, lighting: None };
        let mut system = ParticleSystem::new(&asset.definition, 1);
        let mut world = World { triggers: 0, traces: 0 };
        system.spawn(0, 0, 0.0, 0, None, &asset, &mut world);
        let mut p = system.states[0].particles[0].clone();
        let end = [50.0, 60.0, 110.0, 70.0, 60.0, 100.0][endpoint_by as usize];
        for (i, point) in p.beam.iter().enumerate() { assert!((point.x - (10.0 + (end - 10.0) * i as f32 / 4.0)).abs() < 1e-3, "mode={endpoint_by} {point:?}"); }
        p.size = Vec3::splat(2.0);
        let (vertices, indices) = beam_vertices(&e, &asset.definition, &mut p, Vec3::new(0.0, -1000.0, 0.0), &mut 0.0, &mut 1);
        assert_eq!(vertices.len(), 20);
        assert_eq!(indices.len(), 48);
        assert!(indices.iter().all(|i| (*i as usize) < vertices.len()));
        assert_eq!(vertices[8].uv, [1.0, 0.0]);
        assert_eq!(world.traces, usize::from(endpoint_by == 4));
        assert_eq!(world.triggers, usize::from(endpoint_by == 3));
    }
}

#[test]
fn particle_emitter_handle_actor_force_105d2640_radius_mass_stopped_and_direction() {
    use killer_render::particles::{ActorForce, Particle};
    let asset = fixture(0);
    let mut system = ParticleSystem::new(&asset.definition, 1);
    system.states[0].active = 4;
    for p in &mut system.states[0].particles[..4] { *p = Particle { location: Vec3::new(5.0, 0.0, 0.0), inverse_mass: 1.0, flags: 3, ..Default::default() }; }
    system.states[0].particles[1].inverse_mass = 0.0;
    system.states[0].particles[2].location.x = 10.0;
    system.states[0].particles[3].location.x = -5.0;
    system.handle_actor_force(0, ActorForce { kind: 1, location: Vec3::ZERO, velocity: Vec3::new(20.0, 0.0, 0.0), radius: 10.0, scale: 2.0 }, 0.5);
    for (i, expected) in [5.0, 0.0, 0.0, 5.0].iter().enumerate() { assert_eq!(system.states[0].particles[i].velocity.x, *expected); }
    assert_eq!(system.states[0].particles[0].flags, 1);
    assert_eq!(system.states[0].particles[1].flags, 3);
}

#[test]
fn beam_10505249_scaled_low_noise_uses_high_range_and_branches_link_lifetime() {
    use ue2_level::BeamShape;
    let e = SpriteEmitter { class: "BeamEmitter".into(), coordinate_system: 0, max_particles: 1, start_velocity: [[10.0; 2], [0.0; 2], [0.0; 2]], beam: Some(BeamShape {
        low_points: 3, high_points: 3, low_noise: [[100.0; 2]; 3], high_noise: [[0.0; 2], [3.0; 2], [0.0; 2]],
        use_low_scale: true, low_scale: vec![(1.0, [4.0; 3])],
        use_branching: true, branch_emitter: 1, branch_points: [0.0, 2.0], branch_probability: [1.0; 2], branch_amount: [1.0; 2], linkup_lifetime: true,
        dynamic_noise: [[0.0; 2], [1.0; 2], [0.0; 2]], dynamic_points: [0.0, 2.0], dynamic_time: [0.5; 2],
        ..Default::default()
    }), ..Default::default() };
    let asset = EmitterAsset { definition: Emitter { sprites: vec![e.clone(), SpriteEmitter { coordinate_system: 0, lifetime: [10.0; 2], ..Default::default() }], ..Default::default() }, parts: vec![EmitterPart::default(); 2], zone: 0, range: 12000.0, lighting: None };
    let mut system = ParticleSystem::new(&asset.definition, 1);
    system.spawn(0, 0, 0.0, 0, None, &asset, &mut EmptyWorld);
    let p = &system.states[0].particles[0];
    assert!((p.beam[1].y - 9.0).abs() < 1e-3);
    assert_eq!(system.states[1].active, 1);
    let child = &system.states[1].particles[0];
    assert_eq!(child.location, p.beam[1]);
    assert_eq!(child.lifetime, p.lifetime);
    let mut particle = p.clone();
    particle.size = Vec3::ONE;
    let mut time = 1.25;
    killer_render::particles::beam_vertices(&e, &asset.definition, &mut particle, Vec3::new(0.0, -100.0, 0.0), &mut time, &mut 1);
    assert_eq!(time, 0.25);
    assert!((particle.beam[1].y - 11.0).abs() < 1e-3);
    assert_eq!(particle.beam[0], p.beam[0]);
    assert_eq!(particle.beam[2], p.beam[2]);
}
