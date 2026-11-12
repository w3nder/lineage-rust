use killer_assets::pawn_light::{Actor, Init, PawnLight};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

struct Random(u64);

impl Random {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 16) as u32
    }

    fn float(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * (self.next() % 1_000_000) as f32 / 1_000_000.0
    }

    fn rotation(&mut self) -> [i32; 3] {
        [self.next() as i32 % 70000, self.next() as i32 % 70000, self.next() as i32 % 70000]
    }

    fn vector(&mut self, range: f32) -> [f32; 3] {
        [self.float(-range, range), self.float(-range, range), self.float(-range, range)]
    }
}

fn bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

fn dwords(value: &Value) -> Vec<u32> {
    value.as_array().expect("lista").iter().map(|item| item.as_u64().expect("dword") as u32).collect()
}

fn check_state(light: &PawnLight, native: &[u32], label: &str) {
    let bytes = native[2].to_le_bytes();
    assert_eq!([light.kind, light.light_type, light.effect, light.attach], bytes, "{label}: bytes +8");
    assert_eq!(bits(&light.color), native[3..7], "{label}: cor +0xc");
    assert_eq!(bits(&light.plane), native[7..11], "{label}: plano +0x1c");
    assert_eq!(bits(&light.location), native[11..14], "{label}: posição +0x2c");
    assert_eq!(bits(&light.direction), native[14..17], "{label}: direção +0x38");
    assert_eq!(light.radius.to_bits(), native[17], "{label}: raio +0x44");
    assert_eq!(light.elapsed.to_bits(), native[18], "{label}: tempo +0x48");
    assert_eq!(light.life.to_bits(), native[19], "{label}: vida +0x4c");
    assert_eq!([light.period, light.phase], [native[20].to_le_bytes()[0], native[20].to_le_bytes()[1]], "{label}: período/fase +0x50");
    assert_eq!(bits(&light.target), native[21..24], "{label}: alvo +0x54");
    assert_eq!(light.revision, native[24], "{label}: revisão +0x60");
}

#[test]
#[ignore]
fn fnpawnlight_init_update_intensity_position_direction_match_engine_dll() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/pawn_light_reference.py");
    let python = std::env::var_os("KILLER_ORACLE_PYTHON").unwrap_or_else(|| "python3".into());
    let mut child = Command::new(python).arg(script).arg(&root).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().expect("python");
    let mut input = child.stdin.take().expect("stdin");
    let mut output = BufReader::new(child.stdout.take().expect("stdout"));
    let mut random = Random(0x9e37_79b9_7f4a_7c15);
    let types = [0u8, 1, 2, 3, 7];
    let effects = [8u8, 12, 0x11, 0x13, 0];
    let cases = 3000;
    for case in 0..cases {
        let init = Init {
            kind: (random.next() % 4) as i32,
            location: random.vector(300.0),
            rotation: random.rotation(),
            light_type: types[random.next() as usize % types.len()],
            effect: effects[random.next() as usize % effects.len()],
            plane: [random.float(0.0, 1.0), random.float(0.0, 1.0), random.float(0.0, 1.0), random.float(0.0, 1.0)],
            life: if case % 17 == 0 { 0.0 } else { random.float(0.01, 3.0) },
            radius: random.float(0.0, 200.0),
            attach: (random.next() % 4) as u8,
        };
        let period = (random.next() % 256) as u8;
        let phase = (random.next() % 256) as u8;
        let target = random.vector(3000.0);
        let mut dts: Vec<f32> = (0..(random.next() % 12 + 1)).map(|_| random.float(0.0, 0.25)).collect();
        if case % 11 == 0 {
            dts.insert(0, 0.0);
        }
        if case % 13 == 0 {
            dts.insert(0, init.life);
        }
        let times: Vec<f32> = (0..6).map(|_| random.float(0.0, 40.0)).collect();
        let present = case % 5 != 0;
        let frames: Vec<(Actor, u8)> = (0..4).map(|attach| (Actor { location: random.vector(5000.0), rotation: random.rotation() }, attach as u8)).collect();
        let request = json!({
            "actor": present, "period": period, "phase": phase, "target": target,
            "init": {"kind": init.kind, "location": init.location, "rotation": init.rotation, "light_type": init.light_type, "effect": init.effect,
                     "plane": init.plane, "life": init.life, "radius": init.radius, "attach": init.attach},
            "dts": dts, "times": times,
            "frames": frames.iter().map(|(actor, attach)| json!({"location": actor.location, "rotation": actor.rotation, "attach": attach})).collect::<Vec<_>>(),
        });
        writeln!(input, "{request}").expect("escrita");
        input.flush().expect("flush");
        let mut line = String::new();
        output.read_line(&mut line).expect("leitura");
        let native: Value = serde_json::from_str(&line).expect("json");
        let states: Vec<Vec<u32>> = native["states"].as_array().expect("estados").iter().map(dwords).collect();
        let mut light = PawnLight { period, phase, target, ..PawnLight::default() };
        light.init(&init);
        check_state(&light, &states[0], &format!("caso {case} Init"));
        let alive = dwords(&native["alive"]);
        for (step, dt) in dts.iter().enumerate() {
            let result = light.update(*dt);
            assert_eq!(u32::from(result), alive[step], "caso {case} Update {step} dt {dt}");
            check_state(&light, &states[step + 1], &format!("caso {case} Update {step}"));
        }
        let intensities = dwords(&native["intensities"]);
        for (time, expected) in times.iter().zip(&intensities) {
            assert_eq!(light.intensity(*time).to_bits(), *expected, "caso {case} GetIntensity({time})");
        }
        for (index, ((actor, attach), result)) in frames.iter().zip(native["frames"].as_array().expect("quadros")).enumerate() {
            light.attach = *attach;
            let actor = present.then_some(*actor);
            assert_eq!(bits(&light.position(actor)), dwords(&result[0]), "caso {case} GetPosition {index}");
            assert_eq!(bits(&light.aim(actor)), dwords(&result[1]), "caso {case} GetDirection {index}");
        }
    }
    drop(input);
    assert!(child.wait().expect("fim").success());
    eprintln!("{cases} casos de FNPawnLight conferidos bit a bit com engine.dll");
}
