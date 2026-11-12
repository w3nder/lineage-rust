use glam::Vec3;
use killer_assets::pickup::mover::{Host, Mover, Target};
use serde_json::{Value, json};
use std::{io::Write, path::PathBuf, process::{Command, Stdio}};

struct FreeMovement { fraction: f32, child: Option<Mover>, destroyed: usize, last_child_position: Option<Vec3> }
impl Host for FreeMovement {
    fn tick(&mut self, _: u64, delta: f32) {
        let mut child = self.child.take().unwrap();
        child.tick(delta,self);
        self.last_child_position = Some(child.position);
        self.child = Some(child);
    }
    fn position(&self, _: u64) -> Option<Vec3> { self.child.as_ref().map(|child| child.position) }
    fn ended(&self, _: u64) -> Option<bool> { self.child.as_ref().map(Mover::ended) }
    fn destroy(&mut self, _: u64) { self.child = None; self.destroyed += 1; }
    fn move_actor(&mut self, position: Vec3, delta: Vec3) -> Vec3 {
        Vec3::from(std::array::from_fn::<_,3,_>(|i| (position[i] as f64 + delta[i] as f64 * self.fraction as f64) as f32))
    }

}

#[test]
#[ignore]
fn pickup_mover_matches_original_integration_and_waypoint_completion() {
    let mut cases = Vec::new();
    for targets in [vec![], vec![[0.0;3]], vec![[10.0,30.0,-100.0]], vec![[1.0,2.0,3.0],[8.0,4.0,20.0],[100.0,200.0,-300.0]]] {
        for (speed, acceleration, rate) in [(100.0,0.0,0.0),(300.0,100.0,0.0),(-1.0,-100.0,50.0),(2100.0,1100.0,-4000.0)] {
            for looping in [false,true] {
                for flags in [0,4,8] {
                    for fraction in [1.0,0.25] {
                        cases.push(json!({"position":[0.0,0.0,0.0],"targets":targets,"speed":speed,"acceleration":acceleration,"rate":rate,
                            "max_speed":2000.0,"min_speed":1.0,"max_acceleration":1000.0,"min_acceleration":-1000.0,
                            "delay":0.4,"loop":looping,"extra_flags":flags,"fraction":fraction,"deltas":[0.0,-0.1,0.016,0.25,0.7,1.0,2.0]}));
                    }
                }
            }
        }
    }
    for base in cases.clone().into_iter().filter(|case| case["targets"].as_array().unwrap().len() == 1) {
        for tick_child in [false,true] {
            let mut case = base.clone();
            case["child"] = json!([80.0,120.0,-300.0]);
            case["tick_child"] = json!(tick_child);
            cases.push(case);
        }
    }
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/pickup_mover_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script).arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<Vec<Value>> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(expected.len(), cases.len());
    for (index, (case, snapshots)) in cases.iter().zip(expected).enumerate() {
        let mut mover = Mover::new(Vec3::ZERO);
        let float = |name: &str| case[name].as_f64().unwrap() as f32;
        mover.speed = float("speed"); mover.acceleration = float("acceleration"); mover.acceleration_rate = float("rate");
        let targets: Vec<[f32;3]> = serde_json::from_value(case["targets"].clone()).unwrap();
        mover.targets = targets.into_iter().map(|position| Target { position: Vec3::from(position), actor: None, tick: false }).collect();
        mover.start(float("delay"), case["loop"].as_bool().unwrap());
        mover.flags |= case["extra_flags"].as_u64().unwrap() as u32;
        let child = if case["child"].is_null() { None } else {
            let mut child = Mover::new(mover.targets[0].position);
            child.acceleration = 100.0;
            let position: [f32;3] = serde_json::from_value(case["child"].clone()).unwrap();
            child.targets.push(Target { position: Vec3::from(position), actor: None, tick: false });
            child.start(0.0,false);
            mover.targets[0].actor = Some(1);
            mover.targets[0].tick = case["tick_child"].as_bool().unwrap();
            Some(child)
        };
        let mut host = FreeMovement { fraction: float("fraction"), last_child_position: child.as_ref().map(|c| c.position), child, destroyed: 0 };
        for (frame, (delta,native)) in case["deltas"].as_array().unwrap().iter().zip(snapshots).enumerate() {
            let advanced = mover.tick(delta.as_f64().unwrap() as f32, &mut host);
            assert_eq!(advanced, native["advanced"].as_bool().unwrap(), "case {index}, frame {frame}");
            assert_eq!(mover.index, native["index"].as_u64().unwrap() as usize, "case {index}, frame {frame}");
            assert_eq!(mover.flags, native["flags"].as_u64().unwrap() as u32, "case {index}, frame {frame}");
            assert_eq!(mover.targets.len(), native["targets"].as_u64().unwrap() as usize, "case {index}, frame {frame}");
            assert_eq!(host.destroyed,native["destroyed"].as_u64().unwrap() as usize,"case {index}, frame {frame}");
            if let Some(position) = host.last_child_position {
                let expected: [f32;3] = serde_json::from_value(native["child"].clone()).unwrap();
                assert_eq!(position.to_array().map(f32::to_bits),expected.map(f32::to_bits),"child case {index}, frame {frame}");
            }
            let position: [f32;3] = serde_json::from_value(native["position"].clone()).unwrap();
            for (name,actual,expected) in [("x",mover.position.x,position[0]),("y",mover.position.y,position[1]),("z",mover.position.z,position[2]),
                ("speed",mover.current_speed,native["speed"].as_f64().unwrap() as f32),("acceleration",mover.current_acceleration,native["acceleration"].as_f64().unwrap() as f32)] {
                assert_eq!(actual.to_bits(),expected.to_bits(),"case {index}, frame {frame}, {name}: {actual} != {expected}, {case}");
            }
        }
    }
    eprintln!("{} trajetórias comparadas com engine.dll", cases.len());
}
