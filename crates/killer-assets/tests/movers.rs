use killer_assets::{client::Client, options::ClientOptions, scene::movers};
use std::{collections::BTreeMap, path::PathBuf};
use ue2_level::movers::{MovementState, MoverState};

#[test]
#[ignore]
fn name_to_array_10135250_preserves_legacy_movement_tags() {
    let mut client = Client::open(&PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("cliente"))).unwrap();
    let loaded = client.load_level("17_20").unwrap();
    let actor = loaded.level.actors.iter().find(|a| a.name == "MovableStaticMeshActor20").unwrap();
    assert_eq!(actor.movement_tags, ["BlueEmitor101"]);
    let attached = movers::attached_actors(&loaded.level, actor);
    assert!(!attached.is_empty());
    assert!(attached.iter().all(|a| a.emitter.is_some()));
}

#[test]
#[ignore]
fn phys_l2_movement_10603bf0_matches_original_x86_including_random_cycles() {
    use serde_json::{Value, json};
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let mut cases = Vec::new();
    for max in [[524, 0, 1224], [150, 150, 150], [1, 1, 1], [0, -1, 512]] {
        for random_max in [false, true] {
            for random_start in [false, true] {
                cases.push(json!({"origin":[-345, 3072, 112], "initial":[1,2,3], "max":max, "rate":[524,150,1224], "accel":[0.0,1.2,2.0], "random_max":random_max, "random_start":random_start, "steps":[0.016,0.25,0.5,1.0,4.0,8.0,16.0,0.016,32.0], "seed":7}));
            }
        }
    }
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/mover_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script)
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{:?}", output.status);
    let frames: Value = serde_json::from_slice(&output.stdout).unwrap();
    for (case, frames) in cases.iter().zip(frames.as_array().unwrap()) {
        let def = ue2_level::movers::L2Movement {
            origin: ue2_level::Placement { location: [0.0; 3], rotation: [-345, 3072, 112] },
            initial_rotation: [1, 2, 3],
            max: serde_json::from_value(case["max"].clone()).unwrap(),
            rate: [524, 150, 1224],
            accel: [0.0, 1.2, 2.0],
            random_max: case["random_max"].as_bool().unwrap(),
            random_start: case["random_start"].as_bool().unwrap(),
        };
        let mut state = MovementState::default();
        let mut random = 7u32;
        for (step, (dt, frame)) in case["steps"].as_array().unwrap().iter().zip(frames.as_array().unwrap()).enumerate()
        {
            let pose = state.phys_l2_movement(&def, dt.as_f64().unwrap() as f32, || {
                random = random.wrapping_mul(214013).wrapping_add(2531011);
                ((random >> 16) & 0x7fff) as i32
            });
            assert_eq!(json!(pose.rotation), frame["rotation"], "step {step}: {case}");
            assert_eq!(json!(random), frame["seed"], "random step {step}: {case}");
            for (name, values) in [("current", state.current), ("cycle", state.cycle), ("time", state.time)] {
                for (native_axis, axis) in [0, 2, 1].into_iter().enumerate() {
                    if let Some(native) = frame[name][native_axis].as_f64() {
                        assert!(
                            (f64::from(values[axis]) - native).abs() <= 0.0001,
                            "{name} axis {axis} step {step}: {} != {native} {case}",
                            values[axis]
                        );
                    }
                }
            }
        }
    }
}

#[test]
#[ignore]
fn phys_l2_movement_and_mover_parity_across_all_143_tiles() {
    let mut client = Client::open(&PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("cliente"))).unwrap();
    let options = ClientOptions::load(&mut client).unwrap();
    let mut maps = 0;
    let mut counts = [0; 6];
    let mut rates = BTreeMap::new();
    let mut attachments = BTreeMap::<String, usize>::new();
    let mut tagged = 0;
    let mut textures = std::collections::BTreeSet::new();
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            if client.level_path(&tile).is_err() {
                continue;
            }
            maps += 1;
            eprintln!("mapa {maps}: {tile}");
            let loaded = client.load_level(&tile).unwrap();
            for parent in &loaded.level.actors {
                tagged += usize::from(!parent.movement_tags.is_empty());
                for tag in parent.movement_tags.iter().take(12).take_while(|tag| !tag.eq_ignore_ascii_case("None")) {
                    for child in loaded
                        .level
                        .actors
                        .iter()
                        .filter(|a| a.movement_event.as_ref().is_some_and(|t| t.eq_ignore_ascii_case(tag)))
                    {
                        *attachments.entry(child.class.clone()).or_default() += 1;
                    }
                }
            }
            for actor in &loaded.level.actors {
                if let Some(def) = &actor.l2_movement {
                    counts[0] += 1;
                    *rates.entry(def.rate).or_insert(0) += 1;
                    let mut state = MovementState::default();
                    state.phys_l2_movement(def, 0.0, || 500);
                    for _ in 0..120 {
                        let pose = state.phys_l2_movement(def, 1.0 / 60.0, || 500);
                        assert_eq!(pose.location, actor.placement.location);
                        for i in 0..3 {
                            assert!((pose.rotation[i] - def.origin.rotation[i]).abs() <= def.max[i].abs() + 1);
                        }
                    }
                }
                if let Some(def) = &actor.mover {
                    counts[1] += 1;
                    counts[2] += usize::from(def.key_rot[1] != [0; 3]);
                    counts[3] += usize::from(def.key_pos[1] != [0.0; 3]);
                    counts[4] += usize::from(def.meshes.iter().any(Option::is_some));
                    counts[5] += usize::from(def.server_id.is_some());
                    assert_eq!(def.move_time, 1.0);
                    assert!(def.glide);
                    let mut state = MoverState::new(def);
                    assert_eq!(state.phys_moving_brush(def, 5.0), actor.placement);
                    state.on_door_status_update(def, false, 0, false);
                    assert_eq!(state.phys_moving_brush(def, 1.0), def.key(1));
                    state.on_door_status_update(def, true, 0, false);
                    assert_eq!(state.phys_moving_brush(def, 1.0), def.key(0));
                    for path in def.meshes.iter().flatten() {
                        assert!(client.catalog().static_mesh(path).is_some(), "{tile} {} {path}", actor.name);
                    }
                }
            }
            let assets = movers::build(&mut client, &loaded.level, &options.clipping);
            for asset in &assets {
                let source = loaded.level.actors.iter().find(|a| a.name == asset.name).unwrap();
                if source.static_mesh.is_some() && !source.hidden {
                    assert!(!asset.draws.is_empty(), "{tile}: {} {:?} sem desenho", asset.name, source.static_mesh);
                }
                for child in &asset.attached {
                    assert!(child.emitter.is_some_and(|i| i < loaded.level.emitters.len()));
                }
                for draw in asset.draws.iter().chain(asset.grades.iter().flatten()) {
                    for path in draw.material.texture_paths() {
                        if !path.is_empty() && textures.insert(path.to_owned()) {
                            assert!(client.catalog().texture_raw(path).is_some(), "textura ausente: {path}");
                        }
                    }
                }
            }
            if tile == "24_18" {
                let door = assets.iter().find(|a| a.name == "Mover33").unwrap().mover.as_ref().unwrap();
                let mut state = MoverState::new(door);
                state.on_door_status_update(door, false, 0, false);
                for yaw in [-13344, -6656, 32, 3072] {
                    assert_eq!(state.phys_moving_brush(door, 0.25).rotation[1], yaw);
                }
                let wall = assets.iter().find(|a| a.name == "Mover0").unwrap();
                assert!(wall.mover.as_ref().unwrap().meshes[3].as_ref().unwrap().ends_with("Aden_castle_wallhole01d"));
                assert!(!wall.grades[3].is_empty());
                let scene = killer_assets::scene::Scene { moving: assets, ..Default::default() };
                let cached = killer_assets::cache::CachedScene::from_bank(scene, &Default::default());
                let stamp = killer_assets::cache::SourceStamp::of(b"movers");
                let bytes = killer_assets::cache::encode_to_vec(&stamp, &cached);
                let back = killer_assets::cache::decode(bytes.as_slice(), &stamp).unwrap();
                assert_eq!(back, cached);
            }
            client.catalog().release_files();
        }
    }
    eprintln!("{maps} mapas; contagens {counts:?}; pais com tags: {tagged}");
    eprintln!("vínculos: {attachments:?}");
    eprintln!("texturas resolvidas: {}", textures.len());
    assert_eq!(maps, 143);
    assert_eq!(counts, [711, 240, 115, 122, 74, 235]);
    assert_eq!(rates[&[0, 0, 1224]], 220);
    assert_eq!(rates[&[524, 0, 1024]], 191);
    assert_eq!(tagged, 163);
    assert_eq!(attachments.keys().map(String::as_str).collect::<Vec<_>>(), ["Emitter"]);
    assert_eq!(attachments["Emitter"], 158);
}
