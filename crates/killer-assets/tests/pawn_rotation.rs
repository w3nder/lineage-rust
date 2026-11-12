use killer_assets::pawn_rotation::*;
use serde_json::{Value, json};
use std::{io::Write, path::PathBuf, process::{Command, Stdio}};

struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, count: u64) -> u64 {
        self.next() % count
    }

    fn pick<T: Copy>(&mut self, values: &[T]) -> T {
        values[self.below(values.len() as u64) as usize]
    }

    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }

    fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.unit()
    }

    fn angle(&mut self) -> i32 {
        match self.below(6) {
            0 => self.below(65536) as i32,
            1 => self.below(65536) as i32 - 32768,
            2 => self.pick(&[0, 1, -1, 32767, 32768, 32769, 65535, 65536, -32768, -65536, 98304]),
            3 => self.below(0x100000) as i32 - 0x80000,
            4 => (self.below(16) * 4096) as i32,
            _ => self.below(200) as i32 - 100,
        }
    }

    fn rotator(&mut self) -> Rotator {
        [self.angle(), self.angle(), self.angle()]
    }

    fn delta(&mut self) -> f32 {
        let any = self.range(0.0005, 0.4);
        self.pick(&[0.0005_f32, 0.0083, 0.0166, 0.0333, 0.05, 0.1, 0.25, 0.4, any])
    }

    fn vector(&mut self, scale: f32) -> [f32; 3] {
        [self.range(-scale, scale), self.range(-scale, scale), self.range(-scale, scale)]
    }
}

fn run(cases: &[Value]) -> Vec<Value> {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/pawn_rotation_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into())).arg(script).arg(std::env::var("KILLER_CLIENT_DIR").unwrap()).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(expected.len(), cases.len());
    expected
}

fn rotator(value: &Value) -> Rotator {
    serde_json::from_value(value.clone()).unwrap()
}

#[test]
#[ignore]
fn fixed_turn_matches_native_afixedturn() {
    let mut random = Random(0x9e3779b97f4a7c15);
    let mut cases = Vec::new();
    for _ in 0..40000 {
        let delta = match random.below(4) {
            0 => 0,
            1 => random.below(2000) as i32 - 1000,
            2 => random.below(131072) as i32 - 65536,
            _ => random.angle(),
        };
        cases.push(json!({"mode": "fixed", "flags": random.pick(&[0u32, 0x2000, 0x4000, 0x6000]), "current": random.angle(), "desired": random.angle(), "delta": delta}));
    }
    let expected = run(&cases);
    for (i, (case, expected)) in cases.iter().zip(&expected).enumerate() {
        let value = fixed_turn(case["flags"].as_u64().unwrap() as u32, case["current"].as_i64().unwrap() as i32, case["desired"].as_i64().unwrap() as i32, case["delta"].as_i64().unwrap() as i32);
        assert_eq!(value as i64, expected["value"].as_i64().unwrap(), "{i}: {case}");
    }
}

#[test]
#[ignore]
fn actor_physics_rotation_matches_native_aactor_physicsrotation() {
    let mut random = Random(0xd1b54a32d192ed03);
    let mut cases = Vec::new();
    for _ in 0..30000 {
        let flags = random.pick(&[0u32, 0x2000, 0x4000, 0x6000, 0x8000]);
        let rotation = random.rotator();
        let near = [rotation[0] + random.below(600) as i32 - 300, rotation[1] + random.below(600) as i32 - 300, rotation[2] + random.below(600) as i32 - 300];
        let desired = match random.below(4) {
            0 => rotation,
            1 => near,
            _ => random.rotator(),
        };
        let rate = [random.below(70000) as i32 - 3000, random.below(70000) as i32 - 3000, random.below(70000) as i32 - 3000];
        cases.push(json!({"mode": "actor", "delta": random.delta(), "actor": {"flags": flags, "rotation": rotation, "desired": desired, "rate": rate}}));
    }
    let expected = run(&cases);
    let (mut moved, mut ended) = (0, 0);
    for (i, (case, expected)) in cases.iter().zip(&expected).enumerate() {
        let actor = &case["actor"];
        let mut body = ActorBody { flags: actor["flags"].as_u64().unwrap() as u32, rotation: rotator(&actor["rotation"]), desired: rotator(&actor["desired"]), rate: rotator(&actor["rate"]) };
        let mut moves = 0;
        let did_end = actor_physics_rotation(&mut body, case["delta"].as_f64().unwrap() as f32, |next| {
            moves += 1;
            next
        });
        assert_eq!(body.rotation, rotator(&expected["rotation"]), "{i}: {case} {expected}");
        assert_eq!(moves, expected["moves"].as_u64().unwrap(), "{i}: {case} {expected}");
        assert_eq!(did_end as u64, expected["ended"].as_u64().unwrap(), "{i}: {case} {expected}");
        moved += moves;
        ended += did_end as u64;
    }
    assert!(moved > 3000 && ended > 100, "{moved} {ended}");
}

fn pawn_case(random: &mut Random) -> Value {
    let physics = random.pick(&[0u8, 1, 1, 1, 2, 3, 9, 12, 13]);
    let rotation = random.rotator();
    let desired = random.rotator();
    let controller = match random.below(4) {
        0 => Value::Null,
        _ => {
            let flags = random.pick(&[0u32, 0x4000, 0x4000, 0x2000, 0x6000]);
            let any = random.below(40000) as i32 - 2000;
            let player = random.pick(&[None, Some(any), Some(0), Some(20000)]);
            json!({"flags": flags, "rotation": random.rotator(), "desired": random.rotator(), "spider_axis": random.vector(1.0), "player_turn_rate": player})
        }
    };
    let velocity = if random.below(3) == 0 { random.vector(120.0) } else { random.vector(600.0) };
    let normal = match random.below(5) {
        0 => [0.0, 0.0, 1.0],
        1 => [0.00005, -0.00005, 0.00003],
        _ => random.vector(1.0),
    };
    let any_rate = random.range(-500.0, 4000.0);
    let pitch_rate = random.below(8000) as i32;
    let yaw_rate = random.below(70000) as i32 - 3000;
    let roll_rate = random.pick(&[0, 0, 1, 3072, 8000, 40000, -5]);
    let accel_rate = random.pick(&[0.0_f32, 1.0, 100.0, 2048.0, any_rate]);
    json!({"physics": physics, "flags": random.pick(&[0u32, 0x2000, 0x4000, 0x6000]), "rotation": rotation, "desired": desired,
        "rate": [pitch_rate, yaw_rate, roll_rate],
        "velocity": velocity, "accel_rate": accel_rate,
        "bits_1400": random.pick(&[0u32, 0x100, 0, 0x100, 0x10]), "bits_1404": random.pick(&[0u32, 0, 0x4000]),
        "floor_normal": normal, "controller": controller})
}

fn body(pawn: &Value) -> PawnBody {
    let controller = (!pawn["controller"].is_null()).then(|| {
        let controller = &pawn["controller"];
        ControllerBody { flags: controller["flags"].as_u64().unwrap() as u32, rotation: rotator(&controller["rotation"]), desired: rotator(&controller["desired"]), player_turn_rate: controller["player_turn_rate"].as_i64().map(|rate| rate as i32), spider_axis: serde_json::from_value(controller["spider_axis"].clone()).unwrap() }
    });
    PawnBody {
        physics: pawn["physics"].as_u64().unwrap() as u8,
        flags: pawn["flags"].as_u64().unwrap() as u32,
        rotation: rotator(&pawn["rotation"]),
        desired: rotator(&pawn["desired"]),
        rate: rotator(&pawn["rate"]),
        velocity: serde_json::from_value(pawn["velocity"].clone()).unwrap(),
        accel_rate: pawn["accel_rate"].as_f64().unwrap() as f32,
        bits_1400: pawn["bits_1400"].as_u64().unwrap() as u32,
        bits_1404: pawn["bits_1404"].as_u64().unwrap() as u32,
        floor_normal: serde_json::from_value(pawn["floor_normal"].clone()).unwrap(),
        controller,
    }
}

#[test]
#[ignore]
fn pawn_physics_rotation_matches_native_apawn_physicsrotation() {
    let mut random = Random(0x2545f4914f6cdd1d);
    let mut cases = Vec::new();
    for _ in 0..40000 {
        let pawn = pawn_case(&mut random);
        let old: [f32; 3] = if random.below(3) == 0 { random.vector(120.0) } else { random.vector(600.0) };
        cases.push(json!({"mode": "pawn", "delta": random.delta(), "old_velocity": old, "pawn": pawn}));
    }
    let expected = run(&cases);
    let mut moved = 0;
    for (i, (case, expected)) in cases.iter().zip(&expected).enumerate() {
        let mut pawn = body(&case["pawn"]);
        let old: [f32; 3] = serde_json::from_value(case["old_velocity"].clone()).unwrap();
        let next = pawn_physics_rotation(&mut pawn, case["delta"].as_f64().unwrap() as f32, old);
        if let Some(next) = next {
            pawn.rotation = next;
            moved += 1;
        }
        assert_eq!(pawn.rotation, rotator(&expected["rotation"]), "{i}: {case} {expected}");
        assert_eq!(next.is_some() as u64, expected["moves"].as_u64().unwrap(), "{i}: {case} {expected}");
        assert_eq!(pawn.flags as u64, expected["flags"].as_u64().unwrap(), "{i}: {case} {expected}");
        if let Some(controller) = pawn.controller {
            assert_eq!(controller.rotation, rotator(&expected["controller_rotation"]), "{i}: {case} {expected}");
        }
    }
    assert!(moved > 5000, "{moved}");
}

#[test]
#[ignore]
fn find_slope_rotation_matches_native_apawn_findsloperotation() {
    let mut random = Random(0x94d049bb133111eb);
    let mut cases = Vec::new();
    for _ in 0..30000 {
        let mut pawn = pawn_case(&mut random);
        if random.below(2) == 0 {
            pawn["physics"] = json!(1);
        }
        let normal: [f32; 3] = match random.below(6) {
            0 => [0.0, 0.0, 1.0],
            1 => [0.00005, -0.00005, 0.00003],
            2 => [0.0, 0.0, 0.95],
            3 => [0.3, 0.2, 0.94999],
            _ => random.vector(1.0),
        };
        cases.push(json!({"mode": "slope", "pawn": pawn, "normal": normal, "rotation": random.rotator()}));
    }
    let expected = run(&cases);
    let mut sloped = 0;
    for (i, (case, expected)) in cases.iter().zip(&expected).enumerate() {
        let pawn = body(&case["pawn"]);
        let normal: [f32; 3] = serde_json::from_value(case["normal"].clone()).unwrap();
        let rotation = pawn_find_slope_rotation(&pawn, normal, rotator(&case["rotation"]));
        assert_eq!(rotation, rotator(&expected["rotation"]), "{i}: {case} {expected}");
        sloped += (rotation[0] != 0) as u32;
    }
    assert!(sloped > 3000, "{sloped}");
}

#[test]
#[ignore]
fn pawn_tick_turn_matches_native_apawn_tick() {
    let mut random = Random(0xbf58476d1ce4e5b9);
    let mut cases = Vec::new();
    for _ in 0..40000 {
        let turning = random.below(2) == 0;
        let flags = (if turning { 2u32 } else { 0 }) | (if random.below(3) != 0 { 1 } else { 0 });
        let rotation = random.rotator();
        let wanted = if random.below(4) == 0 { rotation[1] } else { random.angle() };
        let any_rate = random.below(100000) as i32 - 5000;
        let yaw_rate = random.pick(&[0, 2000, 20000, 65536, any_rate]);
        let tick = json!({"net_login": random.below(5) != 0, "rotation": rotation, "yaw_rate": yaw_rate,
            "yaw_override": random.pick(&[0, 0, 7000, -300]), "busy": random.below(6) == 0, "wanted_yaw": wanted, "flags": flags,
            "turning_dir": random.pick(&[0, 1, -1, 2, -2, 5]), "desired": random.rotator(), "controller": random.below(6) != 0, "latent": random.below(5) == 0,
            "lineage_player": random.below(2) == 0, "lineage_yaw": random.range(-90000.0, 90000.0)});
        cases.push(json!({"mode": "tick", "delta": random.delta(), "tick": tick}));
    }
    let expected = run(&cases);
    let (mut turned, mut wanted_hits) = (0, 0);
    for (i, (case, expected)) in cases.iter().zip(&expected).enumerate() {
        let tick = &case["tick"];
        let mut state = PawnTurnState { flags: tick["flags"].as_u64().unwrap() as u32, turning_dir: tick["turning_dir"].as_i64().unwrap() as i32, wanted_yaw: tick["wanted_yaw"].as_i64().unwrap() as i32, yaw_rate: tick["yaw_rate"].as_i64().unwrap() as i32, yaw_override: tick["yaw_override"].as_i64().unwrap() as i32 };
        let mut rotation = rotator(&tick["rotation"]);
        let mut desired = rotator(&tick["desired"]);
        let mut lineage = tick["lineage_yaw"].as_f64().unwrap() as f32;
        let gate = TurnGate { net_login: tick["net_login"].as_bool().unwrap(), controller: tick["controller"].as_bool().unwrap(), latent: tick["latent"].as_bool().unwrap(), busy: tick["busy"].as_bool().unwrap() };
        let lineage_player = tick["lineage_player"].as_bool().unwrap();
        pawn_tick_turn(&mut state, &mut rotation, &mut desired, lineage_player.then_some(&mut lineage), gate, case["delta"].as_f64().unwrap() as f32);
        assert_eq!(rotation, rotator(&expected["rotation"]), "{i}: {case} {expected}");
        assert_eq!(desired, rotator(&expected["desired"]), "{i}: {case} {expected}");
        assert_eq!(state.wanted_yaw as i64, expected["wanted_yaw"].as_i64().unwrap(), "{i}: {case} {expected}");
        assert_eq!(state.flags as i64, expected["flags"].as_i64().unwrap(), "{i}: {case} {expected}");
        assert_eq!(state.yaw_override as i64, expected["yaw_override"].as_i64().unwrap(), "{i}: {case} {expected}");
        assert_eq!(lineage.to_bits(), (expected["lineage_yaw"].as_f64().unwrap() as f32).to_bits(), "{i}: {case} {expected}");
        turned += (desired != rotator(&tick["desired"])) as u32;
        wanted_hits += (state.flags != tick["flags"].as_u64().unwrap() as u32) as u32;
    }
    assert!(turned > 5000 && wanted_hits > 500, "{turned} {wanted_hits}");
}

#[test]
#[ignore]
fn player_classes_take_their_rotation_defaults_from_lineagewarrior_u() {
    let mut client = killer_assets::client::Client::open(&PathBuf::from(std::env::var("KILLER_CLIENT_DIR").unwrap())).unwrap();
    for row in 0..14 {
        let name = killer_assets::pawn::ClassSource::Player { row }.name().unwrap();
        let defaults = client.pawn_rotation_defaults(&name).unwrap();
        assert_eq!(defaults, RotationDefaults { rate: [0, 65000, 0], keyboard_rate: [0, 10000, 0], rotate_to_desired: true, accel_rate: 2048.0 }, "{name}");
    }
}

#[test]
#[ignore]
fn every_npc_class_resolves_a_rotation_rate_from_the_script_packages() {
    let mut client = killer_assets::client::Client::open(&PathBuf::from(std::env::var("KILLER_CLIENT_DIR").unwrap())).unwrap();
    let tables = client.tables().unwrap();
    let started = std::time::Instant::now();
    let rates = client.rotation_rates(&tables.npcgrp);
    let mut seen = std::collections::BTreeMap::new();
    for rate in rates.npcs.values() {
        *seen.entry(*rate).or_insert(0) += 1;
    }
    println!("{} jogadores, {} npcs em {:?}: {seen:?}", rates.players.len(), rates.npcs.len(), started.elapsed());
    assert_eq!(rates.players.len(), 14);
    assert!(rates.players.values().all(|rate| *rate == 65000));
    assert!(rates.npcs.len() * 10 > tables.npcgrp.len() * 9, "{} de {}", rates.npcs.len(), tables.npcgrp.len());
}
