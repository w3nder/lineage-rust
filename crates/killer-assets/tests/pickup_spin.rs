use killer_assets::pickup::Spin;
use serde_json::{Value,json};
use std::{io::Write,path::PathBuf,process::{Command,Stdio}};

#[test]
#[ignore]
fn pickup_rotation_and_completion_match_native_tick() {
    let mut cases = Vec::new();
    for rotation in [[0;3],[-19000,131072,32768],[65535,4000,-16384]] {
        for target in [[0;3],[40000,-65536,16384],[65535,4000,-16384]] {
            for rate in [[0;3],[-540000,0,0],[540000,0,0],[0,0,-540000],[0,0,540000],[1,-1,300],[12345,-54321,91234]] {
                for falling in [false,true] {
                    let mut frames = if falling { vec![(0.016,false),(0.25,false),(0.031,false)] } else { vec![] };
                    frames.extend([(0.0,true),(0.016,true),(0.025,true),(0.2,true),(1.0,true),(1.0,true)]);
                    cases.push(json!({"rotation":rotation,"target":target,"rate":rate,"frames":frames}));
                }
            }
        }
    }
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/pickup_spin_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script).arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(),"{}",json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<Vec<Value>> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(expected.len(),cases.len());
    for (index,(case,native)) in cases.iter().zip(expected).enumerate() {
        let mut spin = Spin { rotation:serde_json::from_value(case["rotation"].clone()).unwrap(),
            target:serde_json::from_value(case["target"].clone()).unwrap(),rate:serde_json::from_value(case["rate"].clone()).unwrap() };
        let mut finished = false;
        for (frame,(step,native)) in case["frames"].as_array().unwrap().iter().zip(native).enumerate() {
            if !finished { finished = spin.tick(step[0].as_f64().unwrap() as f32,step[1].as_bool().unwrap()); }
            let rotation: [i32;3] = serde_json::from_value(native["rotation"].clone()).unwrap();
            let target: [i32;3] = serde_json::from_value(native["target"].clone()).unwrap();
            assert_eq!(spin.rotation,rotation,"case {index}, frame {frame}, {case}");
            assert_eq!(spin.target,target,"case {index}, frame {frame}");
            assert_eq!(finished,native["finished"].as_bool().unwrap(),"case {index}, frame {frame}");
        }
    }
    eprintln!("{} giros comparados com engine.dll",cases.len());
}
