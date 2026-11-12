use std::collections::HashMap;
use std::path::PathBuf;
use ue2_assets::SkeletalMesh;
use ue2_package::Package;

const PACKAGES: [&str; 20] = [
    "Orc", "Magic", "DarkElf", "Dwarf", "Elf", "Fighter", "Shaman", "LineageWeapons", "LineageAccessory", "LineageNpcs", "LineageNpcs2",
    "LineageMonsters", "LineageMonsters2", "LineageMonsters3", "LineageMonsters4", "LineageMonsters5", "LineageMonsters6", "LineageMonsters7", "LineageMonsters8", "LineageMonsters9",
];

fn load(root: &std::path::Path, name: &str) -> Option<Vec<u8>> {
    let path = root.join("animations").join(format!("{name}.ukx"));
    let raw = std::fs::read(&path).ok()?;
    Some(l2_crypto::decrypt(&raw, &path).unwrap().into_owned())
}

fn sane(mesh: &SkeletalMesh) -> bool {
    let finite = mesh.vertices.iter().all(|v| v.position.iter().all(|c| c.is_finite() && c.abs() < 5000.0));
    let indexed = !mesh.indices.is_empty() && mesh.indices.iter().all(|&i| usize::from(i) < mesh.vertices.len());
    let covered: u32 = mesh.sections.iter().map(|s| s.primitives * 3).sum();
    let ordered = mesh.sections.windows(2).all(|w| w[0].first_index + w[0].primitives * 3 == w[1].first_index);
    finite && indexed && !mesh.vertices.is_empty() && covered as usize == mesh.indices.len() && ordered && !mesh.bones.is_empty()
}

fn bounds(mesh: &SkeletalMesh) -> [[f32; 3]; 2] {
    let mut b = [[f32::MAX; 3], [f32::MIN; 3]];
    for v in &mesh.vertices {
        for (axis, &c) in v.position.iter().enumerate() {
            b[0][axis] = b[0][axis].min(c);
            b[1][axis] = b[1][axis].max(c);
        }
    }
    b
}

#[test]
#[ignore]
fn interlude_skeletal_meshes_read_to_the_last_byte_with_sane_geometry() {
    let root = PathBuf::from(std::env::var_os("KILLER_INTERLUDE_DIR").expect("KILLER_INTERLUDE_DIR"));
    let c4 = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from);
    let mut failures = 0;
    for name in PACKAGES {
        let Some(file) = load(&root, name) else { continue };
        let package = Package::parse(&file).unwrap();
        let mut total = 0;
        let (mut ok, mut good) = (0, 0);
        let mut meshes = HashMap::new();
        for export in package.exports().iter().filter(|export| package.class_name(export) == "SkeletalMesh") {
            total += 1;
            match ue2_assets::read_skeletal_mesh(&package, file.as_slice(), export) {
                Ok(mesh) => {
                    ok += 1;
                    if sane(&mesh) { good += 1; } else { eprintln!("  insana {name}.{}", package.name(export.name)); }
                    meshes.insert(package.name(export.name).to_owned(), mesh);
                }
                Err(error) => eprintln!("  {name}.{}: {error:?}", package.name(export.name)),
            }
        }
        let mut report = format!("{name} v{} licensee {}: {total} malhas, {ok} lidas ao último byte, {good} sãs", package.version, package.licensee);
        if let Some(c4_file) = c4.as_deref().and_then(|dir| load(dir, name)) {
            let c4_package = Package::parse(&c4_file).unwrap();
            let (mut common, mut same_bones, mut near) = (0, 0, 0);
            for export in c4_package.exports().iter().filter(|export| c4_package.class_name(export) == "SkeletalMesh") {
                let Some(new) = meshes.get(c4_package.name(export.name)) else { continue };
                let Ok(old) = ue2_assets::read_skeletal_mesh(&c4_package, c4_file.as_slice(), export) else { continue };
                common += 1;
                if old.bones.len() == new.bones.len() && old.bones.iter().zip(&new.bones).all(|(a, b)| a.name == b.name) { same_bones += 1; }
                let (a, b) = (bounds(&old), bounds(new));
                let size = (0..3).map(|axis| a[1][axis] - a[0][axis]).fold(1.0_f32, f32::max);
                if (0..3).all(|axis| (a[0][axis] - b[0][axis]).abs() < 0.25 * size && (a[1][axis] - b[1][axis]).abs() < 0.25 * size) { near += 1; }
            }
            report.push_str(&format!("; vs C4: {common} em comum, {same_bones} com os mesmos ossos, {near} com caixa próxima"));
        }
        eprintln!("{report}");
        failures += total - good;
    }
    assert_eq!(failures, 0);
}

#[test]
#[ignore]
fn interlude_animations_read_to_the_last_byte_with_tracks_matching_the_bones() {
    let root = PathBuf::from(std::env::var_os("KILLER_INTERLUDE_DIR").expect("KILLER_INTERLUDE_DIR"));
    let mut failures = 0;
    for name in PACKAGES.into_iter().chain(["Magic", "Fighter2", "Mystic", "Dance"]) {
        let Some(file) = load(&root, name) else { continue };
        let package = Package::parse(&file).unwrap();
        let (mut total, mut ok, mut good, mut sequences) = (0, 0, 0, 0);
        for export in package.exports().iter().filter(|export| package.class_name(export) == "MeshAnimation") {
            total += 1;
            match ue2_assets::read_mesh_animation(&package, file.as_slice(), export) {
                Ok(animation) => {
                    ok += 1;
                    sequences += animation.sequences.len();
                    let finite = animation.sequences.iter().all(|s| {
                        s.frames > 0 && s.rate.is_finite() && s.rate > 0.0 && s.tracks.iter().all(|t| {
                            t.bone < animation.bones.len()
                                && t.positions.iter().all(|p| p.iter().all(|c| c.is_finite() && c.abs() < 100_000.0))
                                && t.rotations.iter().all(|q| q.iter().all(|c| c.is_finite()))
                                && t.rotations.len() <= s.frames as usize + 1
                        })
                    });
                    if finite { good += 1; } else { eprintln!("  insana {name}.{}", package.name(export.name)); }
                }
                Err(error) => eprintln!("  {name}.{}: {error:?}", package.name(export.name)),
            }
        }
        if total > 0 {
            eprintln!("{name} v{} licensee {}: {total} animações, {ok} lidas ao último byte, {good} sãs, {sequences} sequências", package.version, package.licensee);
        }
        failures += total - good;
    }
    assert_eq!(failures, 0);
}

#[test]
#[ignore]
fn interlude_class_census_of_the_character_packages() {
    let root = PathBuf::from(std::env::var_os("KILLER_INTERLUDE_DIR").expect("KILLER_INTERLUDE_DIR"));
    let mut classes: HashMap<String, (usize, usize)> = HashMap::new();
    for name in PACKAGES {
        let Some(file) = load(&root, name) else { continue };
        let package = Package::parse(&file).unwrap();
        for export in package.exports() {
            let class = package.class_name(export).to_owned();
            let entry = classes.entry(class.clone()).or_default();
            entry.0 += 1;
            let ok = match class.as_str() {
                "VertMesh" | "LodMesh" | "Mesh" => ue2_assets::read_vertex_mesh(&package, file.as_slice(), export).is_ok(),
                "StaticMesh" => ue2_assets::read_static_mesh(&package, file.as_slice(), export).is_ok(),
                _ => true,
            };
            entry.1 += usize::from(ok);
        }
    }
    let mut classes: Vec<_> = classes.into_iter().collect();
    classes.sort();
    for (class, (total, ok)) in classes {
        eprintln!("{class}: {total} ({ok} lidas)");
    }
}

#[test]
#[ignore]
fn interlude_mesh_materials_resolve_to_decodable_textures() {
    let root = PathBuf::from(std::env::var_os("KILLER_INTERLUDE_DIR").expect("KILLER_INTERLUDE_DIR"));
    let mut client = killer_assets::client::Client::open(&root).unwrap();
    for name in ["Orc", "LineageWeapons", "LineageNpcs", "LineageAccessory", "LineageMonsters"] {
        let Some(file) = load(&root, name) else { continue };
        let package = Package::parse(&file).unwrap();
        let mut paths = std::collections::BTreeSet::new();
        for export in package.exports().iter().filter(|export| package.class_name(export) == "SkeletalMesh") {
            if let Ok(mesh) = ue2_assets::read_skeletal_mesh(&package, file.as_slice(), export) {
                paths.extend(mesh.materials.iter().filter(|m| **m != ue2_package::ObjectRef::Null).map(|m| package.object_path(*m)));
            }
        }
        let decoded = paths.iter().filter(|p| client.catalog().texture(p).is_some()).count();
        let missing: Vec<_> = paths.iter().filter(|p| client.catalog().texture(p).is_none()).take(3).collect();
        eprintln!("{name}: {} materiais distintos, {decoded} texturas decodificadas; exemplos sem textura: {missing:?}", paths.len());
    }
}
