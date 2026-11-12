use std::path::{Path, PathBuf};

use ue2_assets::{Error, SkeletalMesh};
use ue2_package::Package;

const ENCRYPTED_CONTAINER_MAGIC: [u8; 4] = [0xCA, 0xFE, 0xF0, 0x0D];

fn client() -> Option<PathBuf> {
    let client = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from);
    if client.is_none() {
        eprintln!("KILLER_CLIENT_DIR not set; skipping");
    }
    client
}

fn package_path(client: &Path, name: &str) -> PathBuf {
    client.join("animations").join(format!("{name}.ukx"))
}

fn load_package(client: &Path, name: &str) -> (Vec<u8>, Package) {
    let path = package_path(client, name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let file = l2_crypto::decrypt(&bytes, &path).unwrap().into_owned();
    let package = Package::parse(&file).unwrap();
    (file, package)
}

fn read_mesh(file: &[u8], package: &Package, name: &str) -> Result<SkeletalMesh, Error> {
    let export = package
        .exports()
        .iter()
        .find(|export| package.name(export.name).eq_ignore_ascii_case(name))
        .unwrap_or_else(|| panic!("{name} não exportado"));
    ue2_assets::read_skeletal_mesh(package, file, export)
}

#[test]
#[ignore]
fn beret_of_the_unlicensed_helmet_package_reads_to_its_end() {
    let Some(client) = client() else { return };
    let (file, package) = load_package(&client, "L2killer_Helmet_UN");
    assert_eq!((package.version, package.licensee), (123, 0));

    let mesh = read_mesh(&file, &package, "MW_viRUS_Beret_1_m002_a").unwrap();
    assert_eq!(mesh.bones.len(), 1);
    assert_eq!(mesh.vertices.len(), 272);
    assert_eq!(mesh.indices.len(), 385 * 3);
    assert_eq!(mesh.sections.len(), 1);
    assert!(mesh.vertices.iter().all(|vertex| vertex.bones == [0; 4] && vertex.weights[0] == 1.0));
}

#[test]
#[ignore]
fn every_unlicensed_l2killer_mesh_reads_except_the_one_without_a_bone_to_bind_to() {
    let Some(client) = client() else { return };
    let mut failed = Vec::new();
    let mut read = 0;
    for name in ["L2killer_Helmet_UN", "L2Killer_Armor", "lineageMonsters_L2Killer", "LineageNPCs_l2Killer02"] {
        let (file, package) = load_package(&client, name);
        assert_eq!((package.version, package.licensee), (123, 0), "{name}");
        for export in package.exports() {
            if !package.class_name(export).eq_ignore_ascii_case("SkeletalMesh") {
                continue;
            }
            match ue2_assets::read_skeletal_mesh(&package, &file, export) {
                Ok(_) => read += 1,
                Err(error) => failed.push(format!("{name}.{}: {error:?}", package.name(export.name))),
            }
        }
    }
    assert_eq!(read, 138);
    assert_eq!(failed, ["L2Killer_Armor.viRUS_FS_050_m002_a: BadMesh"]);
}

#[test]
#[ignore]
fn headgear_package_is_a_cafef00d_container() {
    let Some(client) = client() else { return };
    let path = package_path(&client, "L2killer_HeadGear");
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(bytes[..4], ENCRYPTED_CONTAINER_MAGIC);
    assert!(l2_crypto::decrypt(&bytes, &path).is_ok());
}
