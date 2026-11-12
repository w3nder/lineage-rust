use std::path::PathBuf;
use ue2_assets::{Error, read_skeletal_mesh, read_skeletal_mesh_lod};
use ue2_package::Package;

#[test]
#[ignore]
fn serialize_105a8550_preserves_lod_settings_across_all_core_c4_meshes() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut total = 0;
    for name in ["Fighter", "LineageNpcs", "LineageMonsters"] {
        let path = root.join("animations").join(format!("{name}.ukx"));
        let encoded = std::fs::read(&path).unwrap();
        let file = l2_crypto::decrypt(&encoded, &path).unwrap().into_owned();
        let package = Package::parse(&file).unwrap();
        let mut count = 0;
        for export in package.exports().iter().filter(|export| package.class_name(export).eq_ignore_ascii_case("SkeletalMesh")) {
            let (mesh, info) = ue2_assets::read_skeletal_mesh_lod_with_info(&package, &file, export, 0).unwrap();
            assert!(mesh.lod_settings.impostor.is_some());
            assert!(mesh.lod_settings.reduction.is_some());
            for index in 1..info.len() {
                let lower = read_skeletal_mesh_lod(&package, &file, export, index).unwrap();
                assert_eq!(mesh.lod_settings, lower.lod_settings, "{name}.{} LOD {index}", package.name(export.name));
            }
            count += 1;
        }
        assert!(count > 0, "{name}");
        eprintln!("{name}: {count} SkeletalMesh, todos os níveis");
        total += count;
    }
    assert_eq!(total, 553);
}

#[test]
#[ignore]
fn render_1069a520_can_load_original_lods_without_replacing_the_default_reader() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let path = root.join("animations/Fighter.ukx");
    let encoded = std::fs::read(&path).unwrap();
    let file = l2_crypto::decrypt(&encoded, &path).unwrap().into_owned();
    let package = Package::parse(&file).unwrap();
    let mut count = 0;
    for export in package.exports().iter().filter(|export| package.class_name(export).eq_ignore_ascii_case("SkeletalMesh")) {
        let original = read_skeletal_mesh(&package, &file, export).unwrap();
        let selected = read_skeletal_mesh_lod(&package, &file, export, 0).unwrap();
        assert_eq!(original, selected, "{}", package.name(export.name));
        let (with_info, info) = ue2_assets::read_skeletal_mesh_lod_with_info(&package, &file, export, 0).unwrap();
        assert_eq!(original, with_info);
        assert!(!info.is_empty());
        for index in 1..info.len() {
            let (lower, lower_info) = ue2_assets::read_skeletal_mesh_lod_with_info(&package, &file, export, index).unwrap();
            assert_eq!(original.lod_settings, lower.lod_settings);
            assert_eq!(info.len(), lower_info.len());
            for (first, lower) in info.iter().zip(lower_info) {
                assert_eq!(first.distance_factor.to_bits(), lower.distance_factor.to_bits());
                assert_eq!(first.hysteresis.to_bits(), lower.hysteresis.to_bits());
                assert_eq!(first.soft_wedges, lower.soft_wedges);
            }
        }
        assert!(matches!(read_skeletal_mesh_lod(&package, &file, export, usize::MAX), Err(Error::BadMesh)));
        assert!(original.lod_settings.reduction.is_some());
        assert!(original.lod_settings.impostor.is_some());
        count += 1;
    }
    assert!(count > 0);
    let body = package.exports().iter().find(|export| package.name(export.name).eq_ignore_ascii_case("MFighter_m000_u")).unwrap();
    let first = read_skeletal_mesh_lod(&package, &file, body, 0).unwrap();
    for index in [1, 2] {
        let lower = read_skeletal_mesh_lod(&package, &file, body, index).unwrap();
        assert_eq!(lower.bones, first.bones);
        assert_eq!(lower.animation, first.animation);
        assert_eq!(lower.aliases, first.aliases);
        assert!(!lower.indices.is_empty());
        assert!(lower.indices.len() < first.indices.len());
        assert!(lower.indices.iter().all(|&vertex| usize::from(vertex) < lower.vertices.len()));
        assert_eq!(lower.sections.len(), lower.materials.len());
    }
}
