use std::path::PathBuf;
use ue2_assets::read_static_mesh;
use ue2_core::ByteWindow;
use ue2_package::Package;

#[test]
#[ignore]
fn collision_arrays_consume_original_exports_exactly_in_full_and_windowed_reads() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut total = [0; 3];
    for name in ["L2_Lobby", "LineageEffectsStaticmeshes"] {
        let path = root.join("staticmeshes").join(format!("{name}.usx"));
        let encoded = std::fs::read(&path).unwrap();
        let file = l2_crypto::decrypt(&encoded, &path).unwrap().into_owned();
        let package = Package::parse(&file).unwrap();
        let mut counts = [0; 3];
        for export in package.exports().iter().filter(|export| package.class_name(export).eq_ignore_ascii_case("StaticMesh")) {
            let mesh = read_static_mesh(&package, &file, export).unwrap();
            let collision = &mesh.collision;
            let triangles = collision.read_triangles(&file).unwrap();
            let nodes = collision.read_nodes(&file).unwrap();
            let window = ByteWindow { data: &file[export.serial_offset..export.serial_offset + export.serial_size], base: export.serial_offset };
            let windowed = read_static_mesh(&package, window, export).unwrap();
            assert_eq!(&windowed.collision, collision);
            assert_eq!(collision.read_triangles(window).unwrap(), triangles);
            assert_eq!(collision.read_nodes(window).unwrap(), nodes);
            counts[0] += 1;
            counts[1] += triangles.len();
            counts[2] += nodes.len();
        }
        assert!(counts[0] > 0);
        eprintln!("{name}: {} meshes, {} triângulos, {} nós", counts[0], counts[1], counts[2]);
        for (sum, count) in total.iter_mut().zip(counts) { *sum += count; }
    }
    assert!(total.iter().all(|value| *value > 0));
}
