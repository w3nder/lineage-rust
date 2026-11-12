use killer_assets::client::Client;
use std::path::{Path, PathBuf};
use ue2_package::{ObjectRef, Package};

const PACKAGES: [(&str, &str); 2] =
    [("L2killer_HeadGear", "animations/L2killer_HeadGear.ukx"), ("L2Killer_Assassin_light", "animations/L2Killer_Assassin_light.ukx")];

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
}

#[test]
#[ignore]
fn decoded_packages_have_consistent_tables_and_contiguous_exports() {
    for (name, relative) in PACKAGES {
        let path = root().join(relative);
        let raw = std::fs::read(&path).unwrap();
        assert_eq!(l2_crypto::detect(&raw).unwrap(), l2_crypto::Scheme::CafeF00d, "{name}");
        let payload = l2_crypto::decrypt(&raw, &path).unwrap();
        assert_eq!(payload.len(), raw.len() - 28);
        assert_eq!(payload[..4], [0xC1, 0x83, 0x2A, 0x9E], "{name}");
        let package = Package::parse(&payload).unwrap();
        assert!(!package.names().is_empty() && !package.exports().is_empty() && !package.imports().is_empty());
        for name in package.names() {
            assert!(name.text.bytes().all(|b| (0x20..0x7F).contains(&b)), "{}", name.text);
        }
        let mut spans: Vec<_> = package.exports().iter().filter(|e| e.serial_size > 0).map(|e| (e.serial_offset, e.serial_size)).collect();
        spans.sort();
        for pair in spans.windows(2) {
            assert_eq!(pair[0].0 + pair[0].1, pair[1].0, "{name}: exportação não contígua");
        }
        for export in package.exports() {
            assert!(matches!(export.outer, ObjectRef::Null | ObjectRef::Export(_) | ObjectRef::Import(_)));
        }
    }
}

#[test]
#[ignore]
fn decoded_meshes_and_textures_load_through_the_client() {
    let mut client = Client::open(&root()).unwrap();
    let mut meshes = 0;
    let mut textures = 0;
    for (name, relative) in PACKAGES {
        let path = root().join(relative);
        let payload = l2_crypto::decrypt(&std::fs::read(&path).unwrap(), Path::new(relative)).unwrap().into_owned();
        let package = Package::parse(&payload).unwrap();
        for export in package.exports() {
            let class = package.class_name(export);
            let full = format!("{name}.{}", package.name(export.name));
            match class {
                "SkeletalMesh" => {
                    let mesh = client.catalog().try_skeletal_mesh(&full).unwrap_or_else(|e| panic!("{full}: {e:?}"));
                    assert!(!mesh.mesh.indices.is_empty() && mesh.mesh.indices.len().is_multiple_of(3), "{full}");
                    meshes += 1;
                }
                "Texture" => {
                    let raw = client.catalog().try_texture_raw(&full).unwrap_or_else(|e| panic!("{full}: {e:?}"));
                    assert!(raw.mips.first().is_some_and(|m| m.width > 0 && m.height > 0), "{full}");
                    let image = client.catalog().texture(&full).unwrap_or_else(|| panic!("{full}: sem decodificação"));
                    assert_eq!(image.rgba.len(), (image.width * image.height * 4) as usize, "{full}");
                    textures += 1;
                }
                _ => {}
            }
        }
        client.catalog().release_files();
    }
    eprintln!("{meshes} malhas, {textures} texturas");
    assert!(meshes > 0 && textures > 0);
}
