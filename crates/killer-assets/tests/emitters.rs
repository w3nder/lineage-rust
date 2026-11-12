use killer_assets::client::Client;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[test]
#[ignore]
fn particle_emitter_105ce750_all_143_tiles() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR obrigatório"));
    let mut client = Client::open(&root).unwrap();
    let mut classes = BTreeMap::<String, usize>::new();
    let mut features = BTreeMap::<String, usize>::new();
    let mut styles = [0usize; 7];
    let mut directions = [0usize; 8];
    let mut coordinates = [0usize; 5];
    let mut tiles = 0;
    let mut exports = BTreeMap::<String, usize>::new();
    let mut missing_mesh = 0;
    let mut vertex_meshes = BTreeMap::<String, usize>::new();
    let mut missing_vertex_meshes = BTreeMap::<String, usize>::new();
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            if client.level_path(&tile).is_err() { continue; }
            let loaded = client.load_level(&tile).unwrap();
            let path = client.level_path(&tile).unwrap();
            let raw = std::fs::read(&path).unwrap();
            let file = l2_crypto::decrypt(&raw, &path).unwrap();
            let package = ue2_package::Package::parse(&file).unwrap();
            for export in package.exports() {
                let class = package.class_name(export);
                if ["SpriteEmitter", "MeshEmitter", "BeamEmitter", "VertMeshEmitter"].contains(&class) {
                    *exports.entry(class.to_owned()).or_default() += 1;
                }
            }
            tiles += 1;
            for actor in &loaded.level.emitters {
                for e in &actor.sprites {
                    if e.disabled { continue; }
                    *classes.entry(e.class.clone()).or_default() += 1;
                    missing_mesh += usize::from(e.mesh.as_ref().is_some_and(|m| m.mesh.is_none()));
                    if let Some(shape) = &e.vertex_mesh {
                        let path = shape.mesh.as_deref().expect("VertMeshEmitter sem VertexMesh");
                        if let Some(mesh) = client.catalog().vertex_mesh(path) {
                            assert!(!mesh.mesh.positions.is_empty());
                        } else {
                            *missing_vertex_meshes.entry(format!("{path}: {:?}", client.catalog().object_class(path))).or_default() += 1;
                        }
                        *vertex_meshes.entry(format!("{tile}: {path}")).or_default() += 1;
                    }
                    styles[e.draw_style as usize] += 1;
                    if e.class == "SpriteEmitter" { directions[e.direction as usize] += 1; }
                    coordinates[e.coordinate_system as usize] += 1;
                    for (name, used) in [
                        ("collision", e.use_collision), ("planes", e.use_collision_planes),
                        ("mesh_spawn", e.mesh_spawning != 0), ("skeletal", e.use_skeletal_location != 0),
                        ("forces", e.use_actor_forces), ("refraction", e.refraction != 0),
                        ("projectors", e.accepts_projectors), ("add_location", e.add_location_from_other >= 0),
                        ("add_velocity", e.add_velocity_from_other >= 0), ("collision_spawn", e.spawn_from_other >= 0),
                        ("actor_rotate", actor.rotate), ("auto_reset", actor.auto_reset || e.auto_reset),
                        ("mesh_blend", e.mesh.as_ref().is_some_and(|m| m.own_blend)),
                        ("beam_branch", e.beam.as_ref().is_some_and(|b| b.use_branching)),
                    ] { if used { *features.entry(name.to_owned()).or_default() += 1; } }
                    assert!(e.lifetime.iter().all(|v| v.is_finite()), "{tile}");
                    assert!(e.start_size.iter().flatten().all(|v| v.is_finite()), "{tile}");
                    if let Some(beam) = &e.beam { assert!(beam.high_points >= 2 && beam.low_points >= 2, "{tile}"); }
                }
            }
            client.catalog().release_files();
        }
    }
    eprintln!("tiles={tiles}; classes={classes:?}; exports={exports:?}; missing_mesh={missing_mesh}; styles={styles:?}; directions={directions:?}; coordinates={coordinates:?}; features={features:?}");
    assert_eq!(tiles, 143);
    assert_eq!(classes.get("SpriteEmitter"), Some(&5588));
    assert_eq!(classes.get("MeshEmitter"), Some(&415));
    assert_eq!(classes.get("BeamEmitter"), Some(&70));
    assert_eq!(missing_mesh, 4);
    assert_eq!(classes.get("VertMeshEmitter"), Some(&39));
    assert_eq!(vertex_meshes.values().sum::<usize>(), 39);
    eprintln!("VertMeshEmitter: {vertex_meshes:?}; indisponíveis={missing_vertex_meshes:?}");
}

#[test]
#[ignore]
fn vertex_mesh_references_in_all_lineage_effect_classes() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let file = client.read_system_file("LineageEffect.u").unwrap();
    let package = ue2_package::Package::parse(&file).unwrap();
    let mut references = BTreeMap::<String, usize>::new();
    let mut total = 0;
    let mut class_count = 0;
    let mut unreadable = Vec::new();
    for export in package.exports().iter().filter(|e| package.class_name(e) == "Class") {
        let name = package.name(export.name);
        class_count += 1;
        let Some(actor) = client.emitter_class(&format!("LineageEffect.{name}")) else { unreadable.push(name.to_owned()); continue };
        for emitter in &actor.sprites {
            let Some(shape) = &emitter.vertex_mesh else { continue };
            total += 1;
            let path = shape.mesh.as_deref().expect("VertexMesh");
            let mesh = client.catalog().vertex_mesh(path).unwrap_or_else(|| panic!("{name}: {path}"));
            let texture_classes = mesh.textures.iter().map(|p| p.as_deref().and_then(|p| client.catalog().object_class(p))).collect::<Vec<_>>();
            eprintln!("{name}: {path}; own_blend={} particle_color={} textures={:?} classes={texture_classes:?} sequences={:?}", shape.own_blend, shape.use_particle_color, mesh.textures, mesh.mesh.sequences.iter().map(|s| (&s.name, s.notifies.len())).collect::<Vec<_>>());
            *references.entry(path.to_owned()).or_default() += 1;
        }
    }
    eprintln!("{total} VertMeshEmitter em LineageEffect.u: {references:?}");
    eprintln!("classes={class_count}; ilegíveis={unreadable:?}");
    assert!(unreadable.is_empty());
    assert_eq!(total, 7);
}

#[test]
#[ignore]
fn vertex_emitter_defaults_from_engine_package() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let file = client.read_system_file("Engine.u").unwrap();
    let package = ue2_package::Package::parse(&file).unwrap();
    let export = package.exports().iter().find(|e| package.class_name(e) == "Class" && package.name(e.name) == "VertMeshEmitter").unwrap();
    let properties = ue2_assets::class_default_properties(&package, &file, export).unwrap();
    assert_eq!(ue2_assets::find(&properties, "UseMeshBlendMode").and_then(|p| p.bool()), Some(true));
    assert_eq!(ue2_assets::find(&properties, "RenderTwoSided").and_then(|p| p.bool()), Some(true));
    assert!(!ue2_assets::find(&properties, "UseParticleColor").and_then(|p| p.bool()).unwrap_or(false));
}
