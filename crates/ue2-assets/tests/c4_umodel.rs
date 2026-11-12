use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use ue2_assets::{MeshAnimation, SkeletalMesh};
use ue2_package::Package;

const POSITION_TOLERANCE: f32 = 1e-3;
const WEIGHT_TOLERANCE: f32 = 2.0 / 255.0;
const DEFAULT_UMODEL_OUT: &str = "/tmp/umodel-c4";

struct Oracle {
    client: PathBuf,
    out: PathBuf,
}

fn oracle() -> Option<Oracle> {
    let Some(client) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else {
        eprintln!("KILLER_CLIENT_DIR not set; skipping");
        return None;
    };
    let out = std::env::var_os("UMODEL_OUT").map_or_else(|| PathBuf::from(DEFAULT_UMODEL_OUT), PathBuf::from);
    Some(Oracle { client, out })
}

fn package_path(client: &Path, name: &str) -> PathBuf {
    let file = format!("{name}.ukx");
    let upper = client.join("Animations").join(&file);
    if upper.exists() { upper } else { client.join("animations").join(file) }
}

fn load_package(client: &Path, name: &str) -> (Vec<u8>, Package) {
    let path = package_path(client, name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let file = l2_crypto::decrypt(&bytes, &path).unwrap().into_owned();
    let package = Package::parse(&file).unwrap();
    (file, package)
}

struct Chunks(HashMap<String, (usize, Vec<u8>)>);

impl Chunks {
    fn read(path: &Path) -> Self {
        let data = std::fs::read(path).unwrap();
        let mut chunks = HashMap::new();
        let mut pos = 0;
        while pos + 32 <= data.len() {
            let id = String::from_utf8_lossy(&data[pos..pos + 20]).trim_end_matches('\0').to_owned();
            let size = u32::from_le_bytes(data[pos + 24..pos + 28].try_into().unwrap()) as usize;
            let count = u32::from_le_bytes(data[pos + 28..pos + 32].try_into().unwrap()) as usize;
            pos += 32;
            chunks.insert(id, (size, data[pos..pos + size * count].to_vec()));
            pos += size * count;
        }
        Self(chunks)
    }

    fn records(&self, id: &str) -> Vec<&[u8]> {
        let (size, data) = &self.0[id];
        if *size == 0 { Vec::new() } else { data.chunks_exact(*size).collect() }
    }
}

fn f32_at(record: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(record[offset..offset + 4].try_into().unwrap())
}

fn u16_at(record: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(record[offset..offset + 2].try_into().unwrap())
}

fn i32_at(record: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(record[offset..offset + 4].try_into().unwrap())
}

fn name_at(record: &[u8]) -> String {
    String::from_utf8_lossy(&record[..64]).trim_end_matches('\0').to_owned()
}

fn vec3_at(record: &[u8], offset: usize) -> [f32; 3] {
    [f32_at(record, offset), f32_at(record, offset + 4), f32_at(record, offset + 8)]
}

fn quat_at(record: &[u8], offset: usize) -> [f32; 4] {
    [f32_at(record, offset), f32_at(record, offset + 4), f32_at(record, offset + 8), f32_at(record, offset + 12)]
}

struct PskBone {
    name: String,
    parent: usize,
    rotation: [f32; 4],
    position: [f32; 3],
}

fn bones_of(chunks: &Chunks, id: &str) -> Vec<PskBone> {
    chunks
        .records(id)
        .iter()
        .map(|record| PskBone {
            name: name_at(record),
            parent: usize::try_from(i32_at(record, 72)).unwrap_or(0),
            rotation: quat_at(record, 76),
            position: vec3_at(record, 92),
        })
        .collect()
}

struct Psk {
    points: Vec<[f32; 3]>,
    wedges: Vec<(usize, [f32; 2])>,
    faces: Vec<[u16; 3]>,
    bones: Vec<PskBone>,
    weights: Vec<(f32, usize, usize)>,
}

fn read_psk(path: &Path) -> Psk {
    let chunks = Chunks::read(path);
    let points = chunks.records("PNTS0000").iter().map(|r| vec3_at(r, 0)).collect();
    let wedges =
        chunks.records("VTXW0000").iter().map(|r| (usize::from(u16_at(r, 0)), [f32_at(r, 4), f32_at(r, 8)])).collect();
    let faces = chunks.records("FACE0000").iter().map(|r| [u16_at(r, 0), u16_at(r, 2), u16_at(r, 4)]).collect();
    let weights = chunks
        .records("RAWWEIGHTS")
        .iter()
        .map(|r| (f32_at(r, 0), usize::try_from(i32_at(r, 4)).unwrap(), usize::try_from(i32_at(r, 8)).unwrap()))
        .collect();
    Psk { points, wedges, faces, bones: bones_of(&chunks, "REFSKELT"), weights }
}

struct PsaSequence {
    name: String,
    frames: usize,
    rate: f32,
    first_frame: usize,
}

struct Psa {
    bones: Vec<PskBone>,
    sequences: Vec<PsaSequence>,
    keys: Vec<([f32; 3], [f32; 4], f32)>,
}

fn read_psa(path: &Path) -> Psa {
    let chunks = Chunks::read(path);
    let sequences = chunks
        .records("ANIMINFO")
        .iter()
        .map(|r| PsaSequence {
            name: name_at(r),
            frames: usize::try_from(i32_at(r, 164)).unwrap(),
            rate: f32_at(r, 152),
            first_frame: usize::try_from(i32_at(r, 160)).unwrap(),
        })
        .collect();
    let keys = chunks.records("ANIMKEYS").iter().map(|r| (vec3_at(r, 0), quat_at(r, 12), f32_at(r, 28))).collect();
    Psa { bones: bones_of(&chunks, "BONENAMES"), sequences, keys }
}

fn mirrored(position: [f32; 3]) -> [f32; 3] {
    [position[0], -position[1], position[2]]
}

fn mirrored_rotation(rotation: [f32; 4]) -> [f32; 4] {
    [rotation[0], -rotation[1], rotation[2], -rotation[3]]
}

fn close(a: &[f32], b: &[f32], tolerance: f32) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| (a - b).abs() <= tolerance)
}

fn same_rotation(a: [f32; 4], b: [f32; 4]) -> bool {
    close(&a, &b, POSITION_TOLERANCE) || close(&a.map(|x| -x), &b, POSITION_TOLERANCE)
}

fn canonical(triangle: [u16; 3]) -> [u16; 3] {
    let start = (0..3).min_by_key(|&i| triangle[i]).unwrap();
    [triangle[start], triangle[(start + 1) % 3], triangle[(start + 2) % 3]]
}

fn assert_bones(ours: &[(String, usize)], theirs: &[PskBone], what: &str) {
    let theirs: Vec<(String, usize)> = theirs
        .iter()
        .zip(ours)
        .map(|(bone, ours)| {
            let name = if bone.name.starts_with("__name_") { ours.0.clone() } else { bone.name.clone() };
            (name, bone.parent)
        })
        .collect();
    assert_eq!(ours, &theirs[..], "{what}: bones");
}

fn compare_mesh(name: &str, mesh: &SkeletalMesh, psk: &Psk) {
    let ours: Vec<(String, usize)> = mesh.bones.iter().map(|bone| (bone.name.clone(), bone.parent)).collect();
    assert_bones(&ours, &psk.bones, name);
    for (index, (ours, theirs)) in mesh.bones.iter().zip(&psk.bones).enumerate() {
        assert!(
            close(&ours.position, &mirrored(theirs.position), POSITION_TOLERANCE),
            "{name}: bone {index} position {:?} vs {:?}",
            ours.position,
            theirs.position
        );
        assert!(
            same_rotation(mirrored_rotation(ours.rotation), theirs.rotation),
            "{name}: bone {index} rotation {:?} vs {:?}",
            ours.rotation,
            theirs.rotation
        );
    }

    assert_eq!(mesh.vertices.len(), psk.wedges.len(), "{name}: wedges");
    assert_eq!(mesh.indices.len(), psk.faces.len() * 3, "{name}: faces");
    let mut point_weights: BTreeMap<usize, BTreeMap<usize, f32>> = BTreeMap::new();
    for &(weight, point, bone) in &psk.weights {
        *point_weights.entry(point).or_default().entry(bone).or_default() += weight;
    }
    for (index, (vertex, &(point, uv))) in mesh.vertices.iter().zip(&psk.wedges).enumerate() {
        let position = mirrored(psk.points[point]);
        assert!(close(&vertex.position, &position, POSITION_TOLERANCE), "{name}: vertex {index} position");
        assert!(close(&vertex.uv, &uv, POSITION_TOLERANCE), "{name}: vertex {index} uv");
        let mut ours: BTreeMap<usize, f32> = BTreeMap::new();
        for (&bone, &weight) in vertex.bones.iter().zip(&vertex.weights).filter(|&(_, &weight)| weight > 0.0) {
            *ours.entry(usize::from(bone)).or_default() += weight;
        }
        let theirs = point_weights.get(&point).cloned().unwrap_or_default();
        let sum: f32 = theirs.values().sum();
        let theirs: BTreeMap<usize, f32> = theirs.into_iter().map(|(bone, weight)| (bone, weight / sum)).collect();
        let bones: Vec<usize> = ours.keys().copied().collect();
        let their_bones: Vec<usize> = theirs.keys().copied().collect();
        assert_eq!(bones, their_bones, "{name}: vertex {index} bones {ours:?} vs {theirs:?}");
        for (bone, weight) in &ours {
            assert!(
                (weight - theirs[bone]).abs() <= WEIGHT_TOLERANCE,
                "{name}: vertex {index} weights {ours:?} vs {theirs:?}"
            );
        }
    }
    let mut ours: Vec<[u16; 3]> = mesh.indices.as_chunks::<3>().0.iter().map(|t| canonical([t[1], t[0], t[2]])).collect();
    let mut theirs: Vec<[u16; 3]> = psk.faces.iter().map(|&t| canonical(t)).collect();
    ours.sort_unstable();
    theirs.sort_unstable();
    assert_eq!(ours, theirs, "{name}: triangles");
}

fn compare_animation(name: &str, animation: &MeshAnimation, psa: &Psa) -> usize {
    let mut compared = 0;
    let ours: Vec<&str> = animation.bones.iter().map(|bone| bone.name.as_str()).collect();
    let theirs: Vec<&str> = psa
        .bones
        .iter()
        .zip(&ours)
        .map(|(bone, &ours)| if bone.name.starts_with("__name_") { ours } else { bone.name.as_str() })
        .collect();
    assert_eq!(ours, theirs, "{name}: bones");
    let bones = psa.bones.len();
    assert_eq!(animation.sequences.len(), psa.sequences.len(), "{name}: sequences");
    for (sequence, theirs) in animation.sequences.iter().zip(&psa.sequences) {
        let what = format!("{name}.{}", sequence.name);
        assert_eq!(sequence.name, theirs.name, "{what}: name");
        assert_eq!(sequence.frames as usize, theirs.frames, "{what}: frames");
        assert!((sequence.rate - theirs.rate).abs() <= POSITION_TOLERANCE, "{what}: rate");
        for track in &sequence.tracks {
            for frame in 0..theirs.frames {
                let key = match track.times.iter().position(|&time| time == frame as f32) {
                    Some(key) => key,
                    None if track.times.len() == 1 => 0,
                    None => continue,
                };
                let rotation = track.rotations.get(key).or(track.rotations.first()).copied().unwrap();
                let position = track.positions.get(key).or(track.positions.first()).copied().unwrap();
                let (their_position, their_rotation, _) = psa.keys[(theirs.first_frame + frame) * bones + track.bone];
                compared += 1;
                assert!(
                    close(&position, &mirrored(their_position), POSITION_TOLERANCE),
                    "{what}: bone {} frame {frame} position {position:?} vs {their_position:?}",
                    track.bone
                );
                assert!(
                    same_rotation(mirrored_rotation(rotation), their_rotation),
                    "{what}: bone {} frame {frame} rotation {rotation:?} vs {their_rotation:?}",
                    track.bone
                );
            }
        }
    }
    compared
}

struct Expected {
    exports: usize,
    meshes: usize,
    animations: usize,
    keys: usize,
}

fn check_package(oracle: &Oracle, package_name: &str, expected: Expected) {
    let (file, package) = load_package(&oracle.client, package_name);
    let (mut exports, mut meshes, mut animations, mut keys) = (0, 0, 0, 0);
    for export in package.exports() {
        let class = package.class_name(export);
        let name = package.name(export.name);
        if class == "SkeletalMesh" {
            exports += 1;
            let path = oracle.out.join(package_name).join("SkeletalMesh").join(format!("{name}.psk"));
            let mesh =
                ue2_assets::read_skeletal_mesh(&package, &file, export).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(!mesh.vertices.is_empty() && !mesh.indices.is_empty() && !mesh.bones.is_empty(), "{name}: empty");
            if path.exists() {
                compare_mesh(name, &mesh, &read_psk(&path));
                meshes += 1;
            }
        } else if class == "MeshAnimation" {
            let path = oracle.out.join(package_name).join("MeshAnimation").join(format!("{name}.psa"));
            let animation =
                ue2_assets::read_mesh_animation(&package, &file, export).unwrap_or_else(|e| panic!("{name}: {e}"));
            if path.exists() {
                let compared = compare_animation(name, &animation, &read_psa(&path));
                assert!(compared > 0, "{name}: no keys compared");
                keys += compared;
                animations += 1;
            }
        }
    }
    println!("{package_name}: {meshes} meshes and {animations} animations ({keys} keys) match umodel");
    assert_eq!(exports, expected.exports, "{package_name}: skeletal meshes");
    assert_eq!(meshes, expected.meshes, "{package_name}: meshes compared");
    assert_eq!(animations, expected.animations, "{package_name}: animations compared");
    assert!(keys >= expected.keys, "{package_name}: {keys} keys compared");
}

#[test]
#[ignore]
fn fighter_matches_umodel() {
    let Some(oracle) = oracle() else { return };
    check_package(&oracle, "Fighter", Expected { exports: 141, meshes: 77, animations: 3, keys: 548_091 });
}

#[test]
#[ignore]
fn npcs_match_umodel() {
    let Some(oracle) = oracle() else { return };
    check_package(&oracle, "LineageNpcs", Expected { exports: 175, meshes: 171, animations: 114, keys: 4_294_570 });
}

#[test]
#[ignore]
fn monsters_match_umodel() {
    let Some(oracle) = oracle() else { return };
    let expected = Expected { exports: 237, meshes: 237, animations: 208, keys: 7_396_404 };
    check_package(&oracle, "LineageMonsters", expected);
}
