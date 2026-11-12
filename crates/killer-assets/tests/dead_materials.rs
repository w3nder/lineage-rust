use killer_assets::client::Client;
use std::path::PathBuf;
use ue2_package::{ObjectRef, Package};

const SCRIPT_PACKAGES: [&str; 16] = [
    "Engine", "Core", "Editor", "GamePlay", "Fire", "IpDrv", "LineageEnv", "LineageDeco", "LineageEffect", "LineageWarrior", "LineageNpc", "LineageMonster", "LineageMonster2", "LineageVehicle", "UWindow", "UnrealShare",
];

fn class_of(package: &Package, object: &ObjectRef) -> Option<String> {
    match object {
        ObjectRef::Export(index) => Some(package.class_name(&package.exports()[*index]).to_owned()),
        ObjectRef::Import(index) => Some(package.name(package.imports()[*index].class_name).to_owned()),
        ObjectRef::Null => None,
    }
}

#[test]
#[ignore]
fn glow_modifier_has_no_class_in_any_script_package_and_no_object_references_the_nine_instances() {
    let root = PathBuf::from(std::env::var("KILLER_CLIENT_DIR").unwrap());
    let mut client = Client::open(&root).unwrap();
    for name in SCRIPT_PACKAGES {
        let Ok(file) = client.read_system_file(&format!("{name}.u")) else { continue };
        let package = Package::parse(&file).unwrap();
        assert!(
            !package.exports().iter().any(|export| package.class_name(export) == "Class" && package.name(export.name).eq_ignore_ascii_case("GlowModifier")),
            "{name} define GlowModifier"
        );
    }
    let paths: Vec<PathBuf> = client.catalog().source_paths().map(|path| path.to_owned()).collect();
    let mut instances = Vec::new();
    let mut references = Vec::new();
    for path in paths {
        let Ok(raw) = std::fs::read(&path) else { continue };
        if raw.starts_with(&[0xCA, 0xFE, 0xF0, 0x0D]) {
            continue;
        }
        let Ok(bytes) = l2_crypto::decrypt(&raw, &path).map(|bytes| bytes.into_owned()) else { continue };
        let Ok(package) = Package::parse(&bytes) else { continue };
        let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
        for (index, export) in package.exports().iter().enumerate() {
            if package.class_name(export) == "GlowModifier" {
                instances.push(format!("{stem}.{}", package.object_path(ObjectRef::Export(index))));
            }
            let Ok(properties) = ue2_assets::object_properties(&package, &bytes[..], export) else { continue };
            for property in properties {
                let mut objects: Vec<ObjectRef> = property.objects(&package).unwrap_or_default();
                objects.extend(property.object(&package));
                if objects.iter().any(|object| class_of(&package, object).as_deref() == Some("GlowModifier")) {
                    references.push(format!("{stem}.{} .{}", package.object_path(ObjectRef::Export(index)), property.name));
                }
            }
        }
    }
    assert_eq!(instances.len(), 9, "{instances:?}");
    assert!(references.is_empty(), "{references:?}");
    for instance in &instances {
        assert!(client.catalog().material(instance).is_none(), "{instance}");
    }
}
