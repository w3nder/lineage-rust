use std::path::PathBuf;
use ue2_assets::{Property, class_default_properties, find, object_data};
use ue2_core::Reader;
use ue2_package::Package;

fn engine() -> (Package, Vec<u8>) {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let path = root.join("system/Engine.u");
    let raw = std::fs::read(&path).unwrap();
    let file = l2_crypto::decrypt(&raw, &path).unwrap().into_owned();
    let package = Package::parse(&file).unwrap();
    (package, file)
}

fn defaults<'a>(package: &'a Package, file: &'a [u8], class: &str) -> (String, Vec<Property<'a>>, Option<String>) {
    let export = package
        .exports()
        .iter()
        .find(|export| package.class_name(export) == "Class" && package.name(export.name) == class)
        .unwrap_or_else(|| panic!("classe {class} ausente"));
    let properties = class_default_properties(package, file, export).unwrap();
    let source = package
        .exports()
        .iter()
        .filter(|field| package.object_path(field.outer) == class && package.class_name(field) == "TextBuffer")
        .map(|field| {
            let (_, data) = object_data(package, file, field).unwrap();
            let mut reader = Reader::at(data, 0);
            let _ = [reader.i32().unwrap(), reader.i32().unwrap()];
            reader.string().unwrap()
        })
        .next();
    (package.object_path(export.super_class), properties, source)
}

#[test]
#[ignore]
fn server_primitive_and_fence_actor_classes_match_the_engine_package() {
    let (package, file) = engine();
    let (parent, properties, source) = defaults(&package, &file, "ServerPrimitive");
    assert_eq!(parent, "Info");
    assert_eq!(find(&properties, "bHidden").and_then(|p| p.bool()), Some(false));
    assert_eq!(find(&properties, "bAlwaysVisible").and_then(|p| p.bool()), Some(true));
    assert_eq!(find(&properties, "LineColor").and_then(|p| p.color()), Some([10, 10, 255, 255]));
    let source = source.unwrap().split_whitespace().collect::<Vec<_>>().join(" ");
    for declaration in [
        "struct ServerPointStruct { var() string Name; var() vector Point; var() plane Color; };",
        "struct ServerLineStruct { var() string Name; var() vector Start; var() vector End; var() plane Color; };",
        "array<ServerPointStruct>",
        "array<ServerLineStruct>",
    ] {
        assert!(source.contains(declaration), "{declaration}");
    }
    let (parent, properties, _) = defaults(&package, &file, "SpawnableStaticMeshActor");
    assert_eq!(parent, "StaticMeshActor");
    assert_eq!(find(&properties, "bStatic").and_then(|p| p.bool()), Some(false));
    let (_, properties, _) = defaults(&package, &file, "StaticMeshActor");
    assert_eq!(find(&properties, "DrawType").and_then(|p| p.byte()), Some(8));
    let (_, properties, _) = defaults(&package, &file, "FinalBlend");
    assert_eq!(find(&properties, "ZWrite").and_then(|p| p.bool()), Some(true));
    assert_eq!(find(&properties, "ZTest").and_then(|p| p.bool()), Some(true));
    let (_, _, source) = defaults(&package, &file, "Actor");
    let source = source.unwrap();
    let view_types = source.split_once("enum EActorViewType").unwrap().1.split_once('{').unwrap().1.split_once('}').unwrap().0;
    let names: Vec<_> = view_types.split(',').map(str::trim).filter(|name| !name.is_empty()).collect();
    assert_eq!(names[5], "EAVT_Spawning");
    assert_eq!(names[11], "EAVT_Despawning");
}
