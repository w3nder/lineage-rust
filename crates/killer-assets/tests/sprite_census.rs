use killer_assets::client::Client;
use std::path::PathBuf;
use ue2_package::{ObjectRef, Package};

const CLASSES: [&str; 16] = [
    "Light", "Camera", "PlayerStart", "PathNode", "AmbientSound", "Spotlight", "NMovableSunLight", "NSun", "NMoon", "SkyZoneInfo",
    "ZoneInfo", "LevelInfo", "Brush", "Emitter", "AntiPortalActor", "L2SeamlessInfo",
];

struct Effective {
    draw_type: Option<u8>,
    hidden: Option<bool>,
    texture: Option<String>,
    actor: bool,
}

type Packages = Vec<(Package, Vec<u8>)>;

fn load_packages(client: &mut Client) -> Packages {
    ["Engine", "Core", "GamePlay", "Fire", "IpDrv", "LineageEnv", "LineageDeco", "LineageEffect", "LineageWarrior", "LineageNpc", "LineageMonster", "LineageMonster2", "LineageVehicle", "UWindow"]
        .into_iter()
        .filter_map(|name| {
            let file = client.read_system_file(&format!("{name}.u")).ok()?;
            Some((Package::parse(&file).ok()?, file))
        })
        .collect()
}

fn effective(packages: &Packages, class: &str) -> Option<Effective> {
    let mut result = Effective { draw_type: None, hidden: None, texture: None, actor: false };
    let mut current = Some(class.to_owned());
    let mut found_any = false;
    while let Some(name) = current.take() {
        let Some((package, file, export)) = packages.iter().find_map(|(package, file)| {
            package.exports().iter().find(|export| package.class_name(export) == "Class" && package.name(export.name).eq_ignore_ascii_case(&name)).map(|export| (package, file, export))
        }) else {
            break;
        };
        found_any = true;
        if name.eq_ignore_ascii_case("Actor") {
            result.actor = true;
        }
        let properties = ue2_assets::class_default_properties(package, &file[..], export).ok()?;
        if result.draw_type.is_none() {
            result.draw_type = ue2_assets::find(&properties, "DrawType").and_then(|p| p.byte());
        }
        if result.hidden.is_none() {
            result.hidden = ue2_assets::find(&properties, "bHidden").and_then(|p| p.bool());
        }
        if result.texture.is_none() {
            result.texture = ue2_assets::find(&properties, "Texture").and_then(|p| p.object(package)).map(|o| package.object_path(o));
        }
        current = match export.super_class {
            ObjectRef::Null => None,
            other => Some(package.object_name(other).to_owned()),
        };
    }
    found_any.then_some(result)
}

#[test]
#[ignore]
fn actor_classes_of_the_tiles_by_effective_draw_type_hidden_and_texture() {
    let root = PathBuf::from(std::env::var("KILLER_CLIENT_DIR").unwrap());
    let mut client = Client::open(&root).unwrap();
    let packages = load_packages(&mut client);
    for class in CLASSES {
        match effective(&packages, class) {
            Some(e) => eprintln!("CENSO {class}: DrawType {:?} bHidden {:?} Texture {:?}", e.draw_type, e.hidden, e.texture),
            None => eprintln!("CENSO {class}: fora do Engine.u"),
        }
    }
}

#[test]
#[ignore]
fn no_actor_of_the_143_tiles_is_a_visible_dt_sprite() {
    use std::collections::BTreeMap;
    let root = PathBuf::from(std::env::var("KILLER_CLIENT_DIR").unwrap());
    let mut client = Client::open(&root).unwrap();
    let packages = load_packages(&mut client);
    let mut seen: BTreeMap<String, (usize, usize, usize, usize)> = BTreeMap::new();
    let mut outside: BTreeMap<String, usize> = BTreeMap::new();
    let mut visible_sprites = Vec::new();
    for x in 10..=30 {
        for y in 10..=30 {
            let name = format!("{x}_{y}");
            let Ok(path) = client.level_path(&name) else { continue };
            let Ok(raw) = std::fs::read(&path) else { continue };
            let Ok(bytes) = l2_crypto::decrypt(&raw, &path).map(|b| b.into_owned()) else { continue };
            let Ok(package) = Package::parse(&bytes) else { continue };
            for export in package.exports() {
                let class = package.class_name(export).to_owned();
                if matches!(class.as_str(), "StaticMeshActor" | "MovableStaticMeshActor" | "Mover" | "Brush" | "Texture" | "Model" | "Polys" | "Level" | "StaticMeshInstance" | "TerrainSector" | "AmbientSoundObject") {
                    continue;
                }
                let Some(effective) = effective(&packages, &class).filter(|effective| effective.actor) else {
                    *outside.entry(class).or_default() += 1;
                    continue;
                };
                let Ok(properties) = ue2_assets::object_properties(&package, &bytes[..], export) else { continue };
                let draw_type = ue2_assets::find(&properties, "DrawType").and_then(|p| p.byte()).or(effective.draw_type).unwrap_or(1);
                let hidden = ue2_assets::find(&properties, "bHidden").and_then(|p| p.bool()).or(effective.hidden).unwrap_or(false);
                let entry = seen.entry(class.clone()).or_default();
                entry.0 += 1;
                if draw_type == 1 {
                    entry.1 += 1;
                }
                if hidden {
                    entry.2 += 1;
                }
                if draw_type == 1 && !hidden {
                    entry.3 += 1;
                    visible_sprites.push(format!("{name} {class}"));
                }
            }
        }
    }
    for (class, (total, sprites, hidden, visible)) in &seen {
        eprintln!("CENSO {class}: {total} atores, {sprites} DT_Sprite, {hidden} ocultos, {visible} sprites visíveis");
    }
    for (class, count) in &outside {
        eprintln!("CENSO sem classe de ator nos .u {class}: {count}");
    }
    assert!(visible_sprites.is_empty(), "{visible_sprites:?}");
    assert!(seen.keys().all(|class| !matches!(class.as_str(), "Light" | "Camera" | "PlayerStart" | "PathNode" | "AmbientSound" | "ZoneInfo")) || seen["Light"].0 == seen["Light"].2);
}
