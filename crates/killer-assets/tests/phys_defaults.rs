use std::path::PathBuf;
use ue2_assets::class_default_properties;
use ue2_package::Package;

#[test]
#[ignore]
fn dump_physics_classes() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let path = root.join("system/Engine.u");
    let raw = std::fs::read(&path).unwrap();
    let file = l2_crypto::decrypt(&raw, &path).unwrap().into_owned();
    let package = Package::parse(&file).unwrap();
    for class in ["ZoneInfo", "LevelInfo", "Pawn", "PhysicsVolume", "Actor"] {
        let Some(export) = package.exports().iter().find(|e| package.class_name(e) == "Class" && package.name(e.name) == class) else { eprintln!("sem {class}"); continue };
        let properties = class_default_properties(&package, &file, export).unwrap();
        eprintln!("== {class} super={}", package.object_path(export.super_class));
        for property in &properties {
            eprintln!("  default {} [{}] f={:?} i={:?} b={:?} v={:?}", property.name, property.index, property.float(), property.int(), property.bool(), property.vector());
        }
        let fields: Vec<_> = package.exports().iter().filter(|f| package.object_path(f.outer) == class && package.class_name(f).ends_with("Property")).map(|f| format!("{}:{}", package.name(f.name), package.class_name(f))).collect();
        eprintln!("  campos: {fields:?}");
    }
}

#[test]
#[ignore]
fn dump_bool_property_bytes() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let path = root.join("system/Engine.u");
    let raw = std::fs::read(&path).unwrap();
    let file = l2_crypto::decrypt(&raw, &path).unwrap().into_owned();
    let package = Package::parse(&file).unwrap();
    for (class, name) in [("Pawn","bTryToUncrouch"),("Pawn","bReducedSpeed"),("Pawn","bJumpCapable"),("Pawn","bCanWalkOffLedges"),("Pawn","bSimulateGravity"),("Pawn","bStopAtLedges"),("Actor","bJustTeleported"),("Actor","bScriptInitialized")] {
        let export = package.exports().iter().find(|f| package.object_path(f.outer) == class && package.name(f.name) == name).unwrap();
        let bytes = &file[export.serial_offset..export.serial_offset + export.serial_size];
        eprintln!("{class}.{name}: {bytes:02x?}");
    }
}

#[test]
#[ignore]
fn ledge_defaults_across_packages() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let wanted = ["bCanWalkOffLedges", "bAvoidLedges", "bStopAtLedges", "bSimulateGravity", "bCanFly", "bCanSwim", "bIsWalking"];
    for entry in std::fs::read_dir(root.join("system")).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("u") { continue; }
        let Ok(raw) = std::fs::read(&path) else { continue };
        let Ok(file) = l2_crypto::decrypt(&raw, &path).map(|c| c.into_owned()) else { continue };
        let Ok(package) = Package::parse(&file) else { continue };
        for export in package.exports().iter().filter(|e| package.class_name(e) == "Class") {
            let Ok(properties) = class_default_properties(&package, &file, export) else { continue };
            for property in &properties {
                if wanted.contains(&property.name) {
                    eprintln!("LEDGE {} {} = {:?}", path.file_name().unwrap().to_string_lossy(), package.name(export.name), property.bool());
                }
            }
        }
    }
}

#[test]
#[ignore]
fn ledge_names_in_script_text() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    for entry in std::fs::read_dir(root.join("system")).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("u") { continue; }
        let Ok(raw) = std::fs::read(&path) else { continue };
        let Ok(file) = l2_crypto::decrypt(&raw, &path).map(|c| c.into_owned()) else { continue };
        for needle in ["bCanWalkOffLedges", "bAvoidLedges", "bStopAtLedges"] {
            let bytes = needle.as_bytes();
            let count = file.windows(bytes.len()).filter(|w| *w == bytes).count();
            if count > 0 { eprintln!("TEXTO {} {needle}: {count}", path.file_name().unwrap().to_string_lossy()); }
        }
    }
}

#[test]
#[ignore]
fn ledge_context() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let path = root.join("system/engine.u");
    let raw = std::fs::read(&path).unwrap();
    let file = l2_crypto::decrypt(&raw, &path).unwrap().into_owned();
    for needle in ["bCanWalkOffLedges", "bAvoidLedges", "bStopAtLedges"] {
        let bytes = needle.as_bytes();
        for (at, _) in file.windows(bytes.len()).enumerate().filter(|(_, w)| *w == bytes) {
            let lo = at.saturating_sub(60);
            let text: String = file[lo..(at + 80).min(file.len())].iter().map(|&b| if (32..127).contains(&b) { b as char } else { '.' }).collect();
            eprintln!("CTX {needle}@{at:x}: {text}");
        }
    }
}

#[test]
#[ignore]
fn step_up_defaults_across_packages() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let wanted = ["bCanJump", "bIsWalking", "bIsCrouched", "bCanCrouch", "bDirectHitWall", "bCanWalkOffLedges", "bAvoidLedges", "bStopAtLedges", "bCanFly", "bImmediatelyStop", "bPushable", "MinHitWall", "bMyController", "WalkingPct", "CrouchedPct", "Mass"];
    for entry in std::fs::read_dir(root.join("system")).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("u") { continue; }
        let Ok(raw) = std::fs::read(&path) else { continue };
        let Ok(file) = l2_crypto::decrypt(&raw, &path).map(|c| c.into_owned()) else { continue };
        let Ok(package) = Package::parse(&file) else { continue };
        for export in package.exports().iter().filter(|e| package.class_name(e) == "Class") {
            let Ok(properties) = class_default_properties(&package, &file, export) else { eprintln!("FALHA {} {}", path.file_name().unwrap().to_string_lossy(), package.name(export.name)); continue };
            for property in &properties {
                if wanted.contains(&property.name) {
                    eprintln!("STEPUP {} {} super={} {} = b{:?} f{:?}", path.file_name().unwrap().to_string_lossy(), package.name(export.name), package.object_path(export.super_class), property.name, property.bool(), property.float());
                }
            }
        }
    }
}

#[test]
#[ignore]
fn volume_classes_over_the_143_tiles() {
    use std::collections::BTreeMap;
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = killer_assets::client::Client::open(&root).unwrap();
    let mut classes: BTreeMap<String, usize> = BTreeMap::new();
    let mut tiles = 0;
    for x in 15..=26 {
        for y in 10..=26 {
            let Ok(loaded) = client.load_level(&format!("{x}_{y}")) else { continue };
            tiles += 1;
            for actor in &loaded.level.actors {
                if actor.class.to_ascii_lowercase().contains("volume") || actor.class.to_ascii_lowercase().contains("water") {
                    *classes.entry(actor.class.clone()).or_default() += 1;
                }
            }
        }
    }
    eprintln!("VOL {tiles} tiles: {classes:?}");
}

#[test]
#[ignore]
fn physics_volume_actor_properties() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = killer_assets::client::Client::open(&root).unwrap();
    let mut shown = 0;
    for x in 15..=26 {
        for y in 10..=26 {
            let name = format!("{x}_{y}");
            let Ok(loaded) = client.load_level(&name) else { continue };
            for actor in loaded.level.actors.iter().filter(|a| a.class == "PhysicsVolume") {
                if shown >= 6 { return; }
                let path = format!("{name}.{}", actor.name);
                let text = client.catalog().inspect_object(&path, "", |package, file, export| {
                    let properties = ue2_assets::object_properties(package, file, export).ok()?;
                    Some(properties.iter().map(|p| format!("{}={:?}/{:?}/{:?}", p.name, p.float(), p.bool(), p.int())).collect::<Vec<_>>().join(" | "))
                });
                eprintln!("PV {path}: {text:?}");
                shown += 1;
            }
        }
    }
}

#[test]
#[ignore]
fn alaudio_sections() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = killer_assets::client::Client::open(&root).unwrap();
    let text = client.read_system_text("alaudio.int").unwrap();
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if line.starts_with('[') || lower.contains("step") { eprintln!("AUD {line}"); }
    }
}

#[test]
#[ignore]
fn find_step_sound_table() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    for entry in std::fs::read_dir(root.join("system")).unwrap().flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        if !matches!(ext.as_str(), "dat" | "ini" | "int") { continue; }
        let Ok(raw) = std::fs::read(&path) else { continue };
        let data = l2_crypto::decrypt(&raw, &path).map(|c| c.into_owned()).unwrap_or(raw);
        let ascii = b"StepSound".as_slice();
        let wide: Vec<u8> = "StepSound".encode_utf16().flat_map(|c| c.to_le_bytes()).collect();
        let a = data.windows(ascii.len()).filter(|w| *w == ascii).count();
        let w = data.windows(wide.len()).filter(|x| *x == wide.as_slice()).count();
        if a + w > 0 { eprintln!("STEP {} ascii={a} wide={w}", path.file_name().unwrap().to_string_lossy()); }
    }
}

#[test]
#[ignore]
fn every_player_and_npc_pawn_ignores_and_is_ignored_in_collision() {
    use std::collections::{BTreeMap, HashMap};
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let wanted = ["CanIngnoreCollision", "CanBeIngnoredCollision"];
    let mut supers: HashMap<String, String> = HashMap::new();
    let mut package_of: HashMap<String, String> = HashMap::new();
    let mut defaults: Vec<(String, String, Option<bool>)> = Vec::new();
    let mut text: BTreeMap<(String, &str), usize> = BTreeMap::new();
    for entry in std::fs::read_dir(root.join("system")).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("u") { continue; }
        let Ok(raw) = std::fs::read(&path) else { continue };
        let Ok(file) = l2_crypto::decrypt(&raw, &path).map(|c| c.into_owned()) else { continue };
        let Ok(package) = Package::parse(&file) else { continue };
        let name_of_file = path.file_name().unwrap().to_string_lossy().to_string();
        for needle in wanted {
            let count = file.windows(needle.len()).filter(|w| *w == needle.as_bytes()).count();
            if count > 0 { text.insert((name_of_file.clone(), needle), count); }
        }
        for export in package.exports().iter().filter(|e| package.class_name(e) == "Class") {
            let name = package.name(export.name).to_string();
            let parent = package.object_path(export.super_class).to_string();
            supers.insert(name.clone(), parent.rsplit('.').next().unwrap_or("").to_string());
            package_of.insert(name.clone(), name_of_file.clone());
            let Ok(properties) = class_default_properties(&package, &file, export) else { continue };
            for property in properties.iter().filter(|p| wanted.contains(&p.name)) {
                defaults.push((name.clone(), property.name.to_string(), property.bool()));
            }
        }
    }
    defaults.sort();
    assert_eq!(
        defaults,
        vec![
            ("LineagePawn".to_owned(), "CanBeIngnoredCollision".to_owned(), Some(true)),
            ("LineagePawn".to_owned(), "CanIngnoreCollision".to_owned(), Some(true)),
        ]
    );
    let expected: BTreeMap<(String, &str), usize> = [
        (("engine.u".to_owned(), "CanBeIngnoredCollision"), 2),
        (("engine.u".to_owned(), "CanIngnoreCollision"), 2),
        (("lineagewarrior.u".to_owned(), "CanBeIngnoredCollision"), 1),
        (("lineagewarrior.u".to_owned(), "CanIngnoreCollision"), 1),
    ]
    .into_iter()
    .collect();
    assert_eq!(text, expected);
    let mut lineage_pawns = 0;
    for (name, file) in &package_of {
        let mut chain = vec![name.clone()];
        let mut at = name.clone();
        while let Some(parent) = supers.get(&at) {
            if parent.is_empty() || chain.contains(parent) { break; }
            chain.push(parent.clone());
            at = parent.clone();
        }
        if !chain.iter().any(|c| c == "Pawn") { continue; }
        let lineage = chain.iter().any(|c| c == "LineagePawn");
        if matches!(file.as_str(), "lineagewarrior.u" | "lineagenpc.u" | "lineagemonster.u" | "lineagemonster2.u" | "lineagedeco.u") {
            assert!(lineage, "{file} {name}: {chain:?}");
            lineage_pawns += 1;
        }
    }
    assert_eq!(lineage_pawns, 16 + 192 + 524 + 138 + 10);
}
