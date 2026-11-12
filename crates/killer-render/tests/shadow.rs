use killer_render::shadow::{LightContext, LightInfo, Sun, Viewer, hidden};
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
#[ignore]
fn check_visible_1066de80_matches_native_c4_for_viewers_and_boundaries() {
    let mut cases = Vec::new();
    for previous in [false, true] {
        for has_actor in [false, true] {
            for owner in [None, Some([100000.0f32, -75000.0, -3000.0])] {
                for range in [0.0f32, 0.2, 1.5] {
                    let projector = [250.0f32, -400.0, -3000.0];
                    let base = owner.unwrap_or(projector);
                    for delta in [
                        None,
                        Some([0.0f32; 3]),
                        Some([range * 2048.0, 0.0, 0.0]),
                        Some([range * 2048.0 + 0.02, 0.0, 0.0]),
                        Some([0.0, 0.0, 200.0]),
                        Some([0.0, 0.0, -200.0]),
                        Some([0.0, 0.0, 199.99]),
                        Some([0.0, 0.0, -199.99]),
                    ] {
                        let viewer = delta.map(|v| std::array::from_fn::<_, 3, _>(|i| base[i] + v[i]));
                        cases.push(json!({"visibility": {"previous":previous, "has_actor":has_actor,
                            "projector":projector, "owner":owner, "viewer":viewer, "range":range}}));
                    }
                }
            }
        }
    }
    let controller_cases = cases.clone();
    for mut case in controller_cases {
        if !case["visibility"]["viewer"].is_null() {
            case["visibility"]["pawn"] = case["visibility"]["viewer"].clone();
            case["visibility"]["viewer"] = json!([900000.0, -800000.0, 500000.0]);
            cases.push(case);
        } else {
            for missing in ["viewport", "controller"] {
                let mut missing_case = case.clone();
                missing_case["visibility"]["missing"] = json!(missing);
                cases.push(missing_case);
            }
        }
    }
    let results = oracle(&cases);
    for (case, expected) in cases.iter().zip(results) {
        let case: &Value = &case["visibility"];
        let actual = hidden(
            case["previous"].as_bool().unwrap(),
            case["has_actor"].as_bool().unwrap(),
            serde_json::from_value(case["projector"].clone()).unwrap(),
            serde_json::from_value(case["owner"].clone()).unwrap(),
            serde_json::from_value::<Option<[f32; 3]>>(case["viewer"].clone())
                .unwrap()
                .map(|controller| Viewer { controller, pawn: serde_json::from_value(case["pawn"].clone()).unwrap() }),
            case["range"].as_f64().unwrap() as f32,
        );
        assert_eq!(actual, expected.as_bool().unwrap(), "{case}");
    }
    eprintln!("{} casos contra AShadowProjector::execCheckVisible", cases.len());
}

fn oracle(cases: &[Value]) -> Vec<Value> {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/projector_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script)
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let results: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(results.len(), cases.len());
    results
}

#[test]
#[ignore]
fn filter_projector_106159d0_matches_native_owner_detail_and_wireframe_branches() {
    let mut cases = Vec::new();
    for flags in 0..8 {
        for owner_no_see in [false, true] {
            for owner in -1..3 {
                for detail in 0..4 {
                    for behind_view in [false, true] {
                        for wireframe in [false, true] {
                            for has_actor in [false, true] {
                                cases.push(json!({"filter": {"flags":flags, "owner_no_see":owner_no_see,
                                    "owner":owner, "detail":detail, "behind_view":behind_view,
                                    "wireframe":wireframe, "has_actor":has_actor}}));
                            }
                        }
                    }
                }
            }
        }
    }
    let results = oracle(&cases);
    for (case, expected) in cases.iter().zip(results) {
        let case = &case["filter"];
        let flags = case["flags"].as_u64().unwrap();
        let visibility = ue2_level::projectors::Visibility {
            only_owner: flags & 1 != 0,
            high_detail: flags & 2 != 0,
            super_high_detail: flags & 4 != 0,
            owner_no_see: case["owner_no_see"].as_bool().unwrap(),
        };
        let actual = !case["has_actor"].as_bool().unwrap()
            || visibility.visible(
                case["owner"].as_i64().unwrap() >= 0,
                case["behind_view"].as_bool().unwrap(),
                case["wireframe"].as_bool().unwrap(),
                case["detail"].as_u64().unwrap() as u8,
            );
        assert_eq!(actual, expected.as_bool().unwrap(), "{case}");
    }
    eprintln!("{} casos contra FLevelSceneNode::FilterProjector", cases.len());
}

#[test]
#[ignore]
fn update_light_106665d0_matches_native_c4_modes_sun_and_distance() {
    let mut cases = Vec::new();
    for previous in [false, true] {
        for default_shadow in [-1, 0, 1, 2, 3] {
            for (viewer, pawn, missing) in [
                (None, None, "none"),
                (None, None, "viewport"),
                (None, None, "controller"),
                (Some([0.0; 3]), None, "none"),
                (Some([900000.0; 3]), Some([0.0; 3]), "none"),
                (Some([0.0; 3]), Some([10000.0; 3]), "none"),
            ] {
                for sun in [
                    Value::Null,
                    json!({"hour":5.999, "direction":[1.0,2.0,3.0]}),
                    json!({"hour":6.0, "direction":[1.0,2.0,3.0]}),
                    json!({"hour":24.0, "direction":[1.0,2.0,3.0]}),
                    json!({"hour":24.001, "direction":[1.0,2.0,3.0]}),
                    json!({"hour":12.0, "direction":[0.0,0.0,0.0]}),
                    json!({"hour":12.0, "direction":[1.0,0.0,0.001]}),
                    json!({"hour":12.0, "direction":[1.0,0.0,-0.001]}),
                    json!({"hour":12.0, "direction":[0.00001,0.0,0.0]}),
                ] {
                    for has_actor in [false, true] {
                        cases.push(json!({"light": {"previous":previous,"default_shadow":default_shadow,
                            "viewer":viewer,"pawn":pawn,"missing":missing,"sun":sun,"has_actor":has_actor,
                            "projector":[20.0,-30.0,0.0],"owner":[0.0,0.0,0.0],"range":0.2,
                            "collision_height":22.123,"direction":[0.1,0.2,0.3],"distance":1520.0,"max_trace_distance":1520}}));
                    }
                }
            }
        }
    }
    for owner in [None, Some([100000.0f32, -75000.0, 0.0])] {
        for range in [0.0f32, 0.2, 1.5] {
            let base = owner.unwrap_or([20.0, -30.0, 0.0]);
            let threshold = (f64::from(range) * f64::from(f32::from_bits(0x3f2aaaab)) * 2048.0) as f32;
            for offset in [-0.02f32, 0.0, 0.02] {
                for height in [-22.123f32, 0.0, 22.0, 22.123, 500.99] {
                    let mut case = cases[0].clone();
                    let c = &mut case["light"];
                    c["default_shadow"] = json!(1);
                    c["viewer"] = json!([500000.0, 500000.0, 500000.0]);
                    c["pawn"] = json!([base[0] + threshold + offset, base[1], base[2]]);
                    c["owner"] = json!(owner);
                    c["range"] = json!(range);
                    c["sun"] = json!({"hour":12.0,"direction":[1.0,2.0,3.0]});
                    c["has_actor"] = json!(true);
                    c["collision_height"] = json!(height);
                    cases.push(case);
                }
            }
        }
    }
    let results = oracle(&cases);
    for (case, expected) in cases.iter().zip(results) {
        let c = &case["light"];
        let mut state = LightInfo {
            blob: c["previous"].as_bool().unwrap(),
            direction: serde_json::from_value(c["direction"].clone()).unwrap(),
            distance: c["distance"].as_f64().unwrap() as f32,
            max_trace_distance: c["max_trace_distance"].as_i64().unwrap() as i32,
        };
        state.update(LightContext {
            default_shadow: c["default_shadow"].as_i64().unwrap() as i32,
            has_viewport: !c["viewer"].is_null() || c["missing"] != "none",
            pawn: serde_json::from_value(c["pawn"].clone()).unwrap(),
            projector: serde_json::from_value(c["projector"].clone()).unwrap(),
            owner: serde_json::from_value(c["owner"].clone()).unwrap(),
            clipping_range: c["range"].as_f64().unwrap() as f32,
            sun: (!c["sun"].is_null()).then(|| Sun {
                hour: c["sun"]["hour"].as_f64().unwrap(),
                direction: serde_json::from_value(c["sun"]["direction"].clone()).unwrap(),
            }),
            has_actor: c["has_actor"].as_bool().unwrap(),
            collision_height: c["collision_height"].as_f64().unwrap() as f32,
        });
        assert_eq!(state.blob, expected["blob"].as_bool().unwrap(), "{case}");
        assert_eq!(state.distance, expected["distance"].as_f64().unwrap() as f32, "{case}");
        assert_eq!(state.max_trace_distance, expected["max_trace_distance"].as_i64().unwrap() as i32, "{case}");
        for i in 0..3 {
            let native = expected["direction"][i].as_f64().unwrap() as f32;
            assert!((state.direction[i] - native).abs() < 2e-7, "{case}: {state:?} != {expected}");
        }
    }
    eprintln!("{} casos contra AShadowProjector::execUpdateLightInfo", cases.len());
}
