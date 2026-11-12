use killer_assets::{client::Client, options::ClientOptions, scene};
use std::{collections::BTreeSet, path::PathBuf};

#[test]
#[ignore]
fn giran_pickup_queries_use_original_world_geometry_with_bounded_mesh_cache() {
    use killer_assets::{collision::{resources::Resources,world::{View,trace}},pickup::TraceQuery};
    use glam::Vec3;
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let loaded = client.load_level("22_22").unwrap();
    let views = [View { name:"22_22".into(),level:std::sync::Arc::new(loaded.level),poses:Default::default() }];
    let mut resources = Resources::default();
    for x in -2..=2 {
        for y in -2..=2 {
            let start = Vec3::new(83400.0+x as f32*32.0,148000.0+y as f32*32.0,-3300.0);
            let query = TraceQuery { start,end:start-Vec3::Z*500.0,extent:Vec3::new(5.0,5.0,10.0),flags:0x86 };
            let hit = trace(&mut client,&mut resources,&views,query).unwrap().unwrap_or_else(|| panic!("sem chão em {start:?}"));
            assert!(hit.position.is_finite() && hit.normal.is_finite());
            assert!((0.0..=1.0).contains(&hit.time));
            assert!(hit.position.z <= start.z && hit.position.z >= query.end.z);
            eprintln!("{x},{y}: {hit:?}");
        }
    }
    assert!(resources.cached_bytes() <= 64*1024*1024);
    eprintln!("cache de meshes de colisão: {} bytes",resources.cached_bytes());
}

#[test]
#[ignore]
fn pickup_single_line_check_gives_level_bsp_the_item_extent() {
    use killer_assets::{collision::{model::Model,resources::Resources,world::{View,trace}},pickup::TraceQuery};
    use glam::Vec3;
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let loaded = client.load_level("22_22").unwrap();
    let level = std::sync::Arc::new(loaded.level);
    let views = [View { name:"22_22".into(),level:level.clone(),poses:Default::default() }];
    let mut resources = Resources::default();
    let model = Model { tree:&level.bsp_tree,data:&level.bsp_render,transform:None,unscaled:true };
    let mut floors = 0;
    for x in -2..=2 {
        for y in -2..=2 {
            let start = Vec3::new(83400.0+x as f32*32.0,148000.0+y as f32*32.0,-3300.0);
            let end = start-Vec3::Z*500.0;
            let line = model.trace_line(start.to_array(),end.to_array(),0x486).unwrap();
            for height in [0.2,1.0,5.75,13.74] {
                let extent = Vec3::new(5.0,5.0,height);
                let hit = trace(&mut client,&mut resources,&views,TraceQuery { start,end,extent,flags:0x86 }).unwrap().unwrap();
                let bsp = model.trace(start.to_array(),end.to_array(),extent.to_array(),0x86).unwrap();
                assert!(hit.time <= bsp.time,"{start:?} {height}: {hit:?} {bsp:?}");
                if !hit.non_level_actor {
                    assert_eq!(hit.position.to_array(),bsp.position,"{start:?} {height}");
                    assert!(hit.position.z > line.position[2]+height-0.6,"{start:?} {height}: {hit:?} vs linha {line:?}");
                    floors += 1;
                }
            }
        }
    }
    assert!(floors > 0);
    eprintln!("{floors} colocações no BSP com extensão");
}

struct NativeBsp {
    child: std::process::Child,
    output: std::io::BufReader<std::process::ChildStdout>,
}

impl NativeBsp {
    fn open(root: &std::path::Path) -> Self {
        use std::process::{Command,Stdio};
        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/bsp_level_reference.py");
        let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
            .arg(script).arg(root).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        let output = std::io::BufReader::new(child.stdout.take().unwrap());
        NativeBsp { child,output }
    }

    fn compare(&mut self, tile: &str, level: &ue2_level::Level, queries: &[([f32;3],[f32;3],[f32;3])]) -> usize {
        self.compare_model(tile,&level.bsp_tree,&level.bsp_render,None,queries)
    }

    fn compare_model(&mut self, tile: &str, tree: &ue2_level::BspTree, render: &ue2_level::BspRender, owner: Option<(serde_json::Value,glam::Mat4)>, queries: &[([f32;3],[f32;3],[f32;3])]) -> usize {
        use killer_assets::collision::model::Model;
        use serde_json::{Value,json};
        use std::io::{BufRead,Write};
        let mut bytes = Vec::new();
        for node in &tree.nodes {
            let mut raw = [0u8;0x78];
            for (index,value) in node.plane.iter().enumerate() { raw[index*4..index*4+4].copy_from_slice(&value.to_le_bytes()); }
            raw[0x20..0x24].copy_from_slice(&node.back.to_le_bytes());
            raw[0x24..0x28].copy_from_slice(&node.front.to_le_bytes());
            raw[0x4c..0x50].copy_from_slice(&node.collision_bound.to_le_bytes());
            raw[0x56] = node.vertices;
            raw[0x57] = node.flags;
            bytes.extend_from_slice(&raw);
        }
        for value in &render.leaf_hulls { bytes.extend_from_slice(&value.to_le_bytes()); }
        let file = std::env::temp_dir().join(format!("killer-level-bsp-{}-{:?}.bin",std::process::id(),std::thread::current().id()));
        std::fs::write(&file,&bytes).unwrap();
        let mut request = json!({"file":file,"nodes":tree.nodes.len(),"hulls":render.leaf_hulls.len(),
            "outside":render.root_outside,"fpcw":0x27f,
            "queries":queries.iter().map(|(start,end,extent)| json!({"start":start,"end":end,"extent":extent})).collect::<Vec<_>>()});
        if let Some((actor,_)) = &owner { request["actor"] = actor.clone(); }
        writeln!(self.child.stdin.as_mut().unwrap(),"{request}").unwrap();
        let mut line = String::new();
        self.output.read_line(&mut line).unwrap();
        let expected: Vec<Value> = serde_json::from_str(&line).unwrap_or_else(|error| panic!("{tile}: oráculo falhou: {error}"));
        assert_eq!(expected.len(),queries.len(),"{tile}");
        let model = Model { tree,data:render,transform:owner.as_ref().map(|(_,matrix)| *matrix),unscaled:owner.as_ref().is_none_or(|(actor,_)| actor["draw_scale"] == 1.0 && actor["scale"] == serde_json::json!([1.0,1.0,1.0])) };
        let mut hits = 0;
        for (index,(&(start,end,extent),native)) in queries.iter().zip(expected).enumerate() {
            let ours = model.trace(start,end,extent,0);
            assert_eq!(ours.is_some(),native["hit"].as_bool().unwrap(),"{tile} {index}: {start:?} {end:?} {extent:?} {native} {ours:?}");
            if let Some(hit) = ours {
                hits += 1;
                let position: [f32;3] = serde_json::from_value(native["position"].clone()).unwrap();
                let normal: [f32;3] = serde_json::from_value(native["normal"].clone()).unwrap();
                assert_eq!(hit.position.map(f32::to_bits),position.map(f32::to_bits),"{tile} {index}: nativo {native} nosso {hit:?} bits {:x?} vs {:x?} start {start:?} end {end:?} extent {extent:?}",position.map(f32::to_bits),hit.position.map(f32::to_bits));
                assert_eq!(hit.normal.map(f32::to_bits),normal.map(f32::to_bits),"{tile} {index}: nativo {native} nosso {hit:?}");
                assert_eq!(hit.time.to_bits(),(native["time"].as_f64().unwrap() as f32).to_bits(),"{tile} {index}: {start:?} {end:?} {extent:?} {native} {hit:?}");
            }
        }
        hits
    }
}

impl Drop for NativeBsp {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        let _ = self.child.wait();
    }
}

#[test]
#[ignore]
fn giran_level_bsp_box_check_matches_native_model_line_check() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let level = client.load_level("22_22").unwrap().level;
    let mut queries = Vec::new();
    for x in -2..=2 {
        for y in -2..=2 {
            for (above,below) in [(40.0,50.0),(120.0,300.0),(260.0,500.0)] {
                for height in [0.2,1.0,5.0,5.75,13.74] {
                    let base = glam::Vec3::new(83400.0+x as f32*32.0+0.37,148000.0+y as f32*32.0-0.61,-3430.0);
                    queries.push(((base+glam::Vec3::Z*(above+height)).to_array(),(base-glam::Vec3::Z*below).to_array(),[5.0,5.0,height]));
                }
            }
        }
    }
    let hits = NativeBsp::open(&root).compare("22_22",&level,&queries);
    assert!(hits > 0);
    eprintln!("{} traços no BSP real de Giran comparados ao UModel::LineCheck nativo, {hits} acertos",queries.len());
}

fn level_floor_queries(tile: &str, level: &ue2_level::Level) -> Vec<([f32;3],[f32;3],[f32;3])> {
    let floors: Vec<_> = level.bsp.iter().filter(|polygon| polygon.normal[2] > 0.7 && polygon.corners.len() >= 3).collect();
    let step = floors.len().div_ceil(24).max(1);
    let mut points: Vec<glam::Vec3> = floors.iter().step_by(step).map(|polygon| {
        let sum = polygon.corners.iter().fold(glam::Vec3::ZERO,|sum,corner| sum+glam::Vec3::from(*corner));
        let center = sum/polygon.corners.len() as f32;
        center+(glam::Vec3::from(polygon.corners[0])-center)*0.37
    }).collect();
    let center = killer_assets::tiles::TileId::parse(tile).unwrap().center();
    for x in [-0.25f32,0.0,0.25] {
        for y in [-0.25f32,0.0,0.25] { points.push(glam::Vec3::new(center[0]+x*32768.0,center[1]+y*32768.0,0.0)); }
    }
    let mut queries = Vec::new();
    let walls: Vec<_> = level.bsp.iter().filter(|polygon| polygon.normal[2].abs() < 0.3 && polygon.corners.len() >= 3).collect();
    let wall_step = walls.len().div_ceil(40).max(1);
    for polygon in walls.iter().step_by(wall_step) {
        let sum = polygon.corners.iter().fold(glam::Vec3::ZERO,|sum,corner| sum+glam::Vec3::from(*corner));
        let center = sum/polygon.corners.len() as f32;
        let normal = glam::Vec3::from(polygon.normal);
        for extent in [[9.0f32,9.0,23.0],[34.0,34.0,78.0],[5.0,5.0,10.0]] {
            for tilt in [0.0f32,0.5,-0.5] {
                let side = glam::Vec3::new(-normal.y,normal.x,0.0)*tilt*120.0;
                queries.push(((center+normal*160.0+side).to_array(),(center-normal*160.0+side).to_array(),extent));
            }
        }
    }
    for point in points.iter().step_by(2) {
        let point = *point;
        for angle in [0.0f32,0.8,1.571,2.4,std::f32::consts::PI,4.0,4.712,5.5] {
            let direction = glam::Vec3::new(angle.cos(),angle.sin(),0.0);
            for extent in [[9.0f32,9.0,23.0],[34.0,34.0,78.0]] {
                let from = point+glam::Vec3::Z*(extent[2]+3.0);
                queries.push((from.to_array(),(from+direction*300.0).to_array(),extent));
            }
        }
    }
    for point in points {
        for (above,below) in [(40.0,50.0),(260.0,500.0)] {
            for height in [1.0,5.75,13.74] {
                queries.push(((point+glam::Vec3::Z*(above+height)).to_array(),(point-glam::Vec3::Z*below).to_array(),[5.0,5.0,height]));
            }
            queries.push(((point+glam::Vec3::Z*above).to_array(),(point-glam::Vec3::Z*below).to_array(),[0.0;3]));
        }
    }
    queries
}

#[test]
#[ignore]
fn all_143_tiles_level_bsp_box_check_matches_native_model_line_check() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let mut native = NativeBsp::open(&root);
    let (mut tiles,mut traces,mut hits,mut empty) = (0,0,0,Vec::new());
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            let Ok(loaded) = client.load_level(&tile) else { continue; };
            tiles += 1;
            let queries = level_floor_queries(&tile,&loaded.level);
            let found = native.compare(&tile,&loaded.level,&queries);
            if found == 0 { empty.push(tile.clone()); }
            traces += queries.len();
            hits += found;
            client.catalog().release_files();
        }
    }
    assert_eq!(tiles,143);
    assert!(hits > 0);
    eprintln!("{tiles} tiles, {traces} traços de colocação no BSP comparados ao UModel::LineCheck nativo, {hits} acertos; sem piso BSP: {empty:?}");
}

#[test]
#[ignore]
fn all_143_tiles_preserve_original_terrain_zone_membership() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let mut tiles = 0;
    let mut references = 0;
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            let Ok(loaded) = client.load_level(&tile) else { continue; };
            tiles += 1;
            for (name,zone) in &loaded.level.zone_infos {
                let expected = client.catalog().inspect_object(&format!("{tile}.{name}"),"",|package,file,export| {
                    let properties = ue2_assets::object_properties(package,file,export).ok()?;
                    Some(ue2_assets::find(&properties,"Terrains").and_then(|p| p.objects(package)).unwrap_or_default()
                        .into_iter().map(|object| package.object_name(object).to_owned()).collect::<Vec<_>>())
                }).unwrap();
                assert_eq!(zone.terrains,expected,"{tile}.{name}");
                for terrain in &zone.terrains {
                    assert!(loaded.level.terrains.iter().any(|candidate| candidate.name == *terrain),"{tile}.{name}: {terrain}");
                    references += 1;
                }
            }
            client.catalog().release_files();
        }
    }
    assert_eq!(tiles,143);
    assert_eq!(references,144);
}

#[test]
#[ignore]
fn giran_actor_collision_defaults_and_overrides_come_from_original_classes() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let loaded = client.load_level("22_22").unwrap();
    let names = ["bCollideActors","bBlockActors","bBlockPlayers","bWorldGeometry","bBlockNonZeroExtentTraces","bBlockZeroExtentTraces","bUseCylinderCollision"];
    let mut classes = std::collections::BTreeMap::new();
    let mut total = 0;
    for actor in &loaded.level.actors {
        if actor.static_mesh.is_none() { continue; }
        let path = format!("22_22.{}",actor.name);
        let (class,properties) = client.catalog().inspect_object(&path,"",|package,file,export| {
            let class = package.object_path(export.class);
            let properties = ue2_assets::object_properties(package,file,export).ok()?;
            let values: Vec<_> = names.iter().map(|name| ue2_assets::find(&properties,name).and_then(ue2_assets::Property::bool)).collect();
            Some((class,values))
        }).unwrap_or_else(|| panic!("ator {path}"));
        if !classes.contains_key(&class) {
            classes.insert(class.clone(),client.class_boolean_defaults(&class,&names).unwrap_or_else(|e| panic!("{class}: {e}")));
        }
        let defaults = &classes[&class];
        let values: Vec<_> = names.iter().zip(properties).map(|(name,value)| value.or_else(|| defaults.get(*name).copied()).unwrap_or(false)).collect();
        if values[0] && values[3] && values[4] { total += 1; }
    }
    for (class,defaults) in classes { eprintln!("{class}: {defaults:?}"); }
    eprintln!("{total} atores estáticos com geometria do mundo e extensão não zero");
    assert!(total > 0);
}

#[test]
#[ignore]
fn all_tiles_collision_flags_preserve_class_defaults_and_instance_overrides() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let names = ["bCollideActors","bWorldGeometry","bBlockNonZeroExtentTraces","bUseCylinderCollision"];
    let mut classes = std::collections::BTreeMap::new();
    let mut counts = std::collections::BTreeMap::<String,[usize;3]>::new();
    let mut tiles = 0;
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            let Ok(loaded) = client.load_level(&tile) else { continue; };
            tiles += 1;
            for actor in &loaded.level.actors {
                let path = format!("{tile}.{}",actor.name);
                let (class,properties) = client.catalog().inspect_object(&path,"",|package,file,export| {
                    let properties = ue2_assets::object_properties(package,file,export).ok()?;
                    Some((package.object_path(export.class),names.map(|name| ue2_assets::find(&properties,name).and_then(ue2_assets::Property::bool))))
                }).unwrap_or_else(|| panic!("ator {path}"));
                if !classes.contains_key(&class) {
                    classes.insert(class.clone(),client.class_boolean_defaults(&class,&names).unwrap_or_else(|e| panic!("{class}: {e}")));
                }
                let defaults = &classes[&class];
                let flags: Vec<_> = names.iter().zip(properties).map(|(name,value)| value.or_else(|| defaults.get(*name).copied()).unwrap_or(false)).collect();
                let count = counts.entry(format!("{class} (static_mesh={})",actor.static_mesh.is_some())).or_default();
                count[0] += 1;
                if flags[0] && flags[2] && (flags[1] || actor.mover.is_some()) { count[1] += 1; }
                if flags[3] { count[2] += 1; }
            }
        }
    }
    assert_eq!(tiles,143);
    for (class,count) in counts { eprintln!("{class}: {} atores, {} elegíveis, {} cilindros",count[0],count[1],count[2]); }
}

#[test]
#[ignore]
fn giran_collision_meshes_and_simple_models_are_read_without_visual_buffers() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let loaded = client.load_level("22_22").unwrap();
    let meshes: BTreeSet<_> = loaded.level.actors.iter().filter_map(|actor| actor.static_mesh.as_deref()).collect();
    let mut models = 0;
    for path in &meshes {
        let mesh = client.catalog().static_mesh_collision(path).unwrap_or_else(|| panic!("colisão: {path}"));
        if let Some(path) = mesh.model {
            let model = client.catalog().inspect_object(&path,"Model",|package,file,export| ue2_level::read_model(package,file,export).ok().flatten())
                .unwrap_or_else(|| panic!("modelo simples: {path}"));
            assert!(model.render.complete,"{path}");
            models += 1;
        }
    }
    assert_eq!(meshes.len(),288);
    eprintln!("{} meshes de colisão e {models} modelos simples",meshes.len());
}

#[test]
#[ignore]
fn giran_preserves_all_mesh_references_and_geometry_difference_is_minimum_frame_decoration() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let loaded = client.load_level("22_22").unwrap();
    let meshes: BTreeSet<_> = loaded.level.actors.iter().filter_map(|actor| actor.static_mesh.as_deref()).collect();
    assert_eq!(meshes.len(), 288);
    for path in &meshes {
        let mesh = client.catalog().static_mesh(path).unwrap_or_else(|| panic!("mesh perdido: {path}"));
        assert!(!mesh.mesh.collision.triangles.is_empty());
        assert!(!mesh.mesh.collision.nodes.is_empty());
        for lod in mesh.lod_meshes.iter().flatten() {
            assert!(client.catalog().static_mesh(lod).is_some(), "LOD perdido: {lod}");
        }
    }
    let zones = scene::zone_lighting(&loaded.level);
    let mut options = ClientOptions { keep_min_frame_rate: true, render_deco: true, ..Default::default() };
    assert!(scene::decoration_draws(&mut client, &loaded.level, &zones, &options).is_empty());
    options.keep_min_frame_rate = false;
    let decoration = scene::decoration_draws(&mut client, &loaded.level, &zones, &options);
    let triangles: usize = decoration.iter().map(|draw| draw.world_index_count()/3).sum();
    assert_eq!(decoration.len(), 10354 - 9610);
    assert_eq!(triangles, 13109969 - 928980);
    eprintln!("{} meshes e LODs preservados; diferença: {} draws / {triangles} triângulos de decoração", meshes.len(), decoration.len());
}


struct NativeTerrain {
    child: std::process::Child,
    output: std::io::BufReader<std::process::ChildStdout>,
}

impl NativeTerrain {
    fn open(root: &std::path::Path) -> Self {
        use std::process::{Command,Stdio};
        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/terrain_level_reference.py");
        let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
            .arg(script).arg(root).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        let output = std::io::BufReader::new(child.stdout.take().unwrap());
        NativeTerrain { child,output }
    }

    fn ask(&mut self, request: serde_json::Value) -> Vec<serde_json::Value> {
        use std::io::{BufRead,Write};
        writeln!(self.child.stdin.as_mut().unwrap(),"{request}").unwrap();
        let mut line = String::new();
        self.output.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap_or_else(|error| panic!("oráculo de terreno falhou: {error}"))
    }
}

impl Drop for NativeTerrain {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        let _ = self.child.wait();
    }
}

#[test]
#[ignore]
fn all_143_tiles_terrain_collision_matches_native_vertices_visit_and_quads() {
    use killer_assets::collision::terrain::{Quad,QuadTrace,Vertices,transform_point_by};
    use killer_assets::scene::terrain::{edge_turned,quad_visible};
    use serde_json::json;
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let mut native = NativeTerrain::open(&root);
    let coords_json = |c: ue2_level::Coords| json!([c.origin,c.x_axis,c.y_axis,c.z_axis]);
    let bits3 = |v: [f32;3]| v.map(f32::to_bits);
    let (mut tiles,mut terrains,mut serialized,mut points,mut visits,mut quads,mut hits) = (0,0,0,0,0,0,0);
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            let Ok(loaded) = client.load_level(&tile) else { continue; };
            tiles += 1;
            for terrain in &loaded.level.terrains {
                terrains += 1;
                let owner = client.catalog().inspect_object(&format!("{tile}.{}",terrain.name),"",|package,file,export| {
                    let properties = ue2_assets::object_properties(package,file,export).ok()?;
                    Some(ue2_assets::find(&properties,"Owner").is_some())
                }).unwrap();
                assert!(!owner,"{tile}.{}: Owner presente",terrain.name);
                let heightmap = client.catalog().heightmap_with_base(&terrain.heightmap,terrain.base_height).unwrap();
                let size = [heightmap.width,heightmap.height];
                let vertices = Vertices::new(terrain,size,&heightmap.samples).unwrap();
                if terrain.native.vertices.len() >= size[0]*size[1] {
                    serialized += 1;
                } else {
                    let to_world = terrain.native.to_world.unwrap();
                    let heights: Vec<u16> = heightmap.samples.iter().copied().collect::<BTreeSet<_>>().into_iter().collect();
                    let sample: Vec<[f32;3]> = (0..size[0].max(heights.len())).map(|k| [(k%size[0]) as f32,(k%size[1]) as f32,f32::from(heights[k%heights.len()])]).collect();
                    let expected = native.ask(json!({"kind":"vertices","fpcw":0x27f,"coords":coords_json(to_world),"points":sample}));
                    for (point,native) in sample.iter().zip(expected) {
                        let native: [f32;3] = serde_json::from_value(native).unwrap();
                        assert_eq!(bits3(transform_point_by(to_world,*point)),bits3(native),"{tile}.{}: {point:?}",terrain.name);
                    }
                    points += sample.len();
                }
                let to_heightmap = terrain.native.to_heightmap.unwrap();
                let mut traces = Vec::new();
                for gx in 1..=6 {
                    for gy in 1..=6 {
                        let (cx,cy) = (gx*size[0]/7,gy*size[1]/7);
                        let corners = [vertices.position(cx,cy),vertices.position(cx+1,cy),vertices.position(cx+1,cy+1),vertices.position(cx,cy+1)];
                        let top = corners.iter().map(|v| v[2]).fold(f32::MIN,f32::max);
                        let low = corners.iter().map(|v| v[2]).fold(f32::MAX,f32::min);
                        let px = corners[0][0]+(corners[2][0]-corners[0][0])*0.37;
                        let py = corners[0][1]+(corners[2][1]-corners[0][1])*0.61;
                        for height in [1.0f32,5.75,13.74] {
                            traces.push(([cx,cy],[px,py,top+40.0+height],[px,py,low-50.0],[5.0,5.0,height]));
                        }
                    }
                }
                let mut visit_cases = Vec::new();
                for (cell,start,end,extent) in &traces {
                    for contact in [[cell[0] as i32,cell[1] as i32],[-1,-1]] {
                        visit_cases.push(json!({"coords":coords_json(to_heightmap),"size":size,"start":start,"end":end,"extent":extent,"contact":contact}));
                    }
                }
                let expected = native.ask(json!({"kind":"visit","fpcw":0x27f,"cases":visit_cases}));
                for (case,native) in visit_cases.iter().zip(expected) {
                    let trace = QuadTrace { start:serde_json::from_value(case["start"].clone()).unwrap(),end:serde_json::from_value(case["end"].clone()).unwrap(),
                        extent:serde_json::from_value(case["extent"].clone()).unwrap(),to_heightmap,alternate:None,size };
                    let contact: [i32;2] = serde_json::from_value(case["contact"].clone()).unwrap();
                    let mut visited = Vec::new();
                    let hit = trace.visit(|cx,cy| { visited.push([cx,cy]); [cx as i32,cy as i32] == contact });
                    let native_visited: Vec<[usize;2]> = serde_json::from_value(native["visited"].clone()).unwrap();
                    assert_eq!(hit,native["hit"].as_bool().unwrap(),"{tile}.{}: {case}",terrain.name);
                    assert_eq!(visited,native_visited,"{tile}.{}: {case}",terrain.name);
                }
                visits += visit_cases.len();
                let mut quad_cases = Vec::new();
                let mut quad_values = Vec::new();
                for (cell,start,end,extent) in &traces {
                    let [cx,cy] = *cell;
                    let quad = Quad { vertices:[vertices.position(cx,cy),vertices.position(cx+1,cy),vertices.position(cx+1,cy+1),vertices.position(cx,cy+1)],
                        turned:edge_turned(&terrain.edge_turn,size[0],cx,cy),inverted:terrain.inverted,
                        visible:quad_visible(&terrain.visible_quads,size[0],cx,cy),deleted:false };
                    quad_cases.push(json!({"vertices":quad.vertices,"turned":quad.turned,"inverted":quad.inverted,"visible":quad.visible,"deleted":false,
                        "skip_visibility":false,"start":start,"end":end,"extent":extent}));
                    quad_values.push((quad,*start,*end,*extent));
                }
                let expected = native.ask(json!({"kind":"quad","fpcw":0x27f,"cases":quad_cases}));
                for ((quad,start,end,extent),native) in quad_values.iter().zip(expected) {
                    let mut hit = None;
                    let accepted = quad.trace(*start,*end,*extent,false,&mut hit);
                    assert_eq!(!accepted,native.is_null(),"{tile}.{}: {start:?} {end:?} {extent:?} {native} {hit:?}",terrain.name);
                    if let Some(hit) = hit {
                        hits += 1;
                        let normal: [f32;3] = serde_json::from_value(native["normal"].clone()).unwrap();
                        let position: [f32;3] = serde_json::from_value(native["position"].clone()).unwrap();
                        assert_eq!(hit.time.to_bits(),(native["time"].as_f64().unwrap() as f32).to_bits(),"{tile}.{}: {start:?} {end:?} {extent:?} {native} {hit:?}",terrain.name);
                        assert_eq!(bits3(hit.normal),bits3(normal),"{tile}.{}: {start:?} {native} {hit:?}",terrain.name);
                        assert_eq!(bits3(hit.position),bits3(position),"{tile}.{}: {start:?} {native} {hit:?}",terrain.name);
                    }
                }
                quads += quad_values.len();
            }
            client.catalog().release_files();
        }
    }
    assert_eq!(tiles,143);
    assert!(hits > 0);
    eprintln!("{tiles} tiles, {terrains} terrenos ({serialized} com vértices serializados), {points} vértices, {visits} travessias, {quads} quads, {hits} contatos comparados ao engine.dll");
}

#[test]
#[ignore]
fn all_143_tiles_build_collision_with_original_primitives() {
    use killer_assets::collision::resources::{Resources,Primitive};
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let (mut tiles,mut meshes,mut simple,mut models,mut cylinders,mut unresolved) = (0,0,0,0,0,Vec::new());
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            let Ok(loaded) = client.load_level(&tile) else { continue; };
            tiles += 1;
            let level = std::sync::Arc::new(loaded.level);
            let mut resources = Resources::default();
            let built = resources.tile(&mut client,&tile,&level).unwrap_or_else(|error| panic!("{tile}: {error}"));
            for shape in &built.actors {
                let actor = &level.actors[shape.index];
                match &shape.primitive {
                    Primitive::Mesh(path) => {
                        meshes += 1;
                        let mesh = resources.mesh(&mut client,path).unwrap_or_else(|error| panic!("{tile}: {error}"));
                        if mesh.simple_box && mesh.model.is_some() { simple += 1; }
                    }
                    Primitive::Model(_) => models += 1,
                    Primitive::Cylinder { .. } => {
                        cylinders += 1;
                        if let Some(mesh) = &actor.static_mesh { unresolved.push(format!("{tile}:{mesh}")); }
                    }
                }
            }
            client.catalog().release_files();
        }
    }
    assert_eq!(tiles,143);
    unresolved.sort();
    unresolved.dedup();
    eprintln!("{meshes} malhas ({simple} com modelo simples), {models} brushes, {cylinders} cilindros; StaticMesh sem resolução: {unresolved:?}");
}

#[test]
#[ignore]
fn all_143_tiles_actor_matrices_match_native_local_and_world() {
    use killer_assets::collision::{resources::{Resources,Primitive},transform};
    use serde_json::{Value,json};
    use std::{io::Write,process::{Command,Stdio}};
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let mut seen = std::collections::HashSet::new();
    let mut actors = Vec::new();
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            let Ok(loaded) = client.load_level(&tile) else { continue; };
            let level = std::sync::Arc::new(loaded.level);
            let mut resources = Resources::default();
            let built = resources.tile(&mut client,&tile,&level).unwrap();
            for shape in &built.actors {
                if matches!(shape.primitive,Primitive::Cylinder { .. }) { continue; }
                let actor = &level.actors[shape.index];
                let key = (actor.placement.location.map(f32::to_bits),actor.placement.rotation,actor.draw_scale.to_bits(),actor.draw_scale_3d.map(f32::to_bits),shape.pre_pivot.map(f32::to_bits));
                if seen.insert(key) { actors.push((actor.placement,actor.draw_scale,actor.draw_scale_3d,actor.scale,shape.pre_pivot)); }
            }
            client.catalog().release_files();
        }
    }
    let mut cases = Vec::new();
    for (placement,draw_scale,scale3d,_,pivot) in &actors {
        for kind in ["local","world"] {
            cases.push(json!({"kind":kind,"fpcw":0x27f,"bits":true,"location":placement.location,"rotation":placement.rotation,"draw_scale":draw_scale,"scale":scale3d,"pivot":pivot}));
        }
    }
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/collision_transform_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script).arg(&root).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(),"{}",json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(expected.len(),cases.len());
    for (index,((placement,draw_scale,scale3d,scale,pivot),pair)) in actors.iter().zip(expected.chunks(2)).enumerate() {
        let local = transform::actor(*placement,*scale,*pivot).to_cols_array();
        let world = transform::world_to_local(*placement,*draw_scale,*scale3d,*pivot).to_cols_array();
        let native_local: [u32;16] = serde_json::from_value(pair[0].clone()).unwrap();
        let native_world: [u32;16] = serde_json::from_value(pair[1].clone()).unwrap();
        let canonical = |bits: u32| if f32::from_bits(bits).is_nan() { 0x7fc0_0000 } else { bits };
        let (local,native_local,world,native_world) = (local.map(|v| canonical(v.to_bits())),native_local.map(canonical),world.map(|v| canonical(v.to_bits())),native_world.map(canonical));
        assert_eq!(local,native_local,"{index} LocalToWorld {placement:?} {draw_scale} {scale3d:?} {pivot:?}\n{local:?}\n{native_local:?}");
        assert_eq!(world,native_world,"{index} WorldToLocal {placement:?} {draw_scale} {scale3d:?} {pivot:?}\n{world:?}\n{native_world:?}");
    }
    eprintln!("{} atores distintos: LocalToWorld e WorldToLocal idênticas ao engine.dll",actors.len());
}

struct NativeMesh {
    child: std::process::Child,
    output: std::io::BufReader<std::process::ChildStdout>,
}

impl NativeMesh {
    fn open(root: &std::path::Path) -> Self {
        use std::process::{Command,Stdio};
        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/static_mesh_level_reference.py");
        let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
            .arg(script).arg(root).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        let output = std::io::BufReader::new(child.stdout.take().unwrap());
        NativeMesh { child,output }
    }

    fn ask(&mut self, request: &serde_json::Value) -> Vec<Vec<serde_json::Value>> {
        use std::io::{BufRead,Write};
        writeln!(self.child.stdin.as_mut().unwrap(),"{request}").unwrap();
        let mut line = String::new();
        self.output.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap_or_else(|error| panic!("oráculo de malha falhou: {error}"))
    }
}

impl Drop for NativeMesh {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        let _ = self.child.wait();
    }
}

#[test]
#[ignore]
fn all_143_tiles_static_mesh_line_check_matches_native_wrapper() {
    use killer_assets::collision::{resources::{Resources,Primitive},static_mesh::Mesh,transform};
    use serde_json::json;
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let only = std::env::var("KILLER_TILES").ok();
    let per_mesh: usize = std::env::var("KILLER_MESH_ACTORS").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
    let mut client = Client::open(&root).unwrap();
    let mut native = NativeMesh::open(&root);
    let (mut tiles,mut meshes,mut traces,mut hits) = (0,0,0,0);
    let mut unreadable = std::collections::BTreeSet::new();
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            if only.as_deref().is_some_and(|only| !only.split(',').any(|t| t == tile)) { continue; }
            let Ok(loaded) = client.load_level(&tile) else { continue; };
            tiles += 1;
            let level = std::sync::Arc::new(loaded.level);
            let mut resources = Resources::default();
            let built = resources.tile(&mut client,&tile,&level).unwrap();
            let mut groups: std::collections::BTreeMap<String,Vec<usize>> = Default::default();
            for (index,shape) in built.actors.iter().enumerate() {
                if let Primitive::Mesh(path) = &shape.primitive { groups.entry(path.to_lowercase()).or_default().push(index); }
            }
            for (path,members) in groups {
                let mesh = match resources.mesh(&mut client,&path) {
                    Ok(mesh) => mesh,
                    Err(why) => { unreadable.insert(format!("{path}: {why}")); continue; }
                };
                if mesh.simple_box && mesh.model.is_some() { continue; }
                meshes += 1;
                let [lo,hi] = mesh.bounding_box;
                let mut actors = Vec::new();
                let mut checks = Vec::new();
                for &member in members.iter().take(per_mesh) {
                    let shape = &built.actors[member];
                    let actor = &level.actors[shape.index];
                    let world = transform::actor(actor.placement,actor.scale,shape.pre_pivot);
                    let mut queries = Vec::new();
                    for (fx,fy) in [(0.5f32,0.5f32),(0.25,0.75),(0.8,0.3)] {
                        let local = glam::Vec3::new(lo[0]+(hi[0]-lo[0])*fx,lo[1]+(hi[1]-lo[1])*fy,hi[2]);
                        let point = world.transform_point3(local);
                        let bottom = world.transform_point3(glam::Vec3::new(local.x,local.y,lo[2]));
                        let (top,low) = (point.z.max(bottom.z),point.z.min(bottom.z));
                        for height in [1.0f32,13.74] {
                            queries.push(([point.x,point.y,top+40.0+height],[point.x,point.y,low-50.0],[5.0,5.0,height]));
                        }
                        queries.push(([point.x,point.y,top+40.0],[point.x,point.y,low-50.0],[0.0;3]));
                    }
                    actors.push(json!({"location":actor.placement.location,"rotation":actor.placement.rotation,"draw_scale":actor.draw_scale,
                        "scale":actor.draw_scale_3d,"pivot":shape.pre_pivot,
                        "queries":queries.iter().map(|(s,e,x)| json!({"start":s,"end":e,"extent":x})).collect::<Vec<_>>()}));
                    checks.push((world,transform::world_to_local(actor.placement,actor.draw_scale,actor.draw_scale_3d,shape.pre_pivot),queries));
                }
                let request = json!({"fpcw":0x27f,"vertices":mesh.positions,
                    "triangles":mesh.triangles.iter().map(|t| json!({"planes":t.planes,"indices":t.indices})).collect::<Vec<_>>(),
                    "nodes":mesh.nodes.iter().map(|n| json!({"indices":n.indices,"bounds":n.bounds,"valid":n.valid})).collect::<Vec<_>>(),
                    "actors":actors});
                let expected = native.ask(&request);
                for ((world,world_to_local,queries),natives) in checks.iter().zip(expected) {
                    let shape = Mesh { vertices:&mesh.positions,triangles:&mesh.triangles,nodes:&mesh.nodes,transform:*world,world_to_local:*world_to_local };
                    for (&(start,end,extent),native) in queries.iter().zip(natives) {
                        traces += 1;
                        let ours = shape.trace(start,end,extent);
                        let native_hit = native["hit"].as_bool().unwrap();
                        assert_eq!(ours.is_some(),native_hit,"{tile} {path}: {start:?} {end:?} {extent:?} nativo {native} nosso {ours:?}");
                        if let Some(hit) = ours {
                            hits += 1;
                            let normal: [u32;3] = serde_json::from_value(native["normal"].clone()).unwrap();
                            let position: [u32;3] = serde_json::from_value(native["position"].clone()).unwrap();
                            let time = native["time"].as_u64().unwrap() as u32;
                            assert_eq!((hit.time.to_bits(),hit.normal.map(f32::to_bits),hit.position.map(f32::to_bits)),(time,normal,position),
                                "{tile} {path}: {start:?} {end:?} {extent:?} nativo t={} n={:?} p={:?} nosso {hit:?}",f32::from_bits(time),normal.map(f32::from_bits),position.map(f32::from_bits));
                        }
                    }
                }
            }
            client.catalog().release_files();
        }
    }
    assert!(tiles > 0 && hits > 0);
    assert_eq!(unreadable.into_iter().collect::<Vec<_>>(),["field_deco_s.diongiranstone.dgstone09: colisão ausente: field_deco_s.diongiranstone.dgstone09"]);
    eprintln!("{tiles} tiles, {meshes} malhas, {traces} traços, {hits} contatos idênticos ao UStaticMesh::LineCheck");
}


#[test]
#[ignore]
fn all_143_tiles_blocking_volumes_match_native_model_line_check_with_owner() {
    use killer_assets::collision::{resources::{Resources,Primitive},transform};
    use serde_json::json;
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let only = std::env::var("KILLER_TILES").ok();
    let mut client = Client::open(&root).unwrap();
    let mut native = NativeBsp::open(&root);
    let (mut volumes,mut traces,mut hits) = (0,0,0);
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            if only.as_deref().is_some_and(|only| !only.split(',').any(|t| t == tile)) { continue; }
            let Ok(loaded) = client.load_level(&tile) else { continue; };
            let level = std::sync::Arc::new(loaded.level);
            let mut resources = Resources::default();
            let built = resources.tile(&mut client,&tile,&level).unwrap();
            for shape in &built.actors {
                let Primitive::Model(path) = &shape.primitive else { continue; };
                let actor = &level.actors[shape.index];
                let model = resources.model(&mut client,path).unwrap();
                let world = transform::actor(actor.placement,actor.scale,shape.pre_pivot);
                let [lo,hi] = model.bounding_box;
                let mut queries = Vec::new();
                for (fx,fy) in [(0.5f32,0.5f32),(0.2,0.8),(0.85,0.15)] {
                    let top = world.transform_point3(glam::Vec3::new(lo[0]+(hi[0]-lo[0])*fx,lo[1]+(hi[1]-lo[1])*fy,hi[2]));
                    let bottom = world.transform_point3(glam::Vec3::new(lo[0]+(hi[0]-lo[0])*fx,lo[1]+(hi[1]-lo[1])*fy,lo[2]));
                    let (high,low) = (top.z.max(bottom.z),top.z.min(bottom.z));
                    for extent in [[5.0f32,5.0,1.0],[5.0,5.0,13.74],[0.0;3]] {
                        queries.push(([top.x,top.y,high+40.0+extent[2]],[top.x,top.y,low-50.0],extent));
                    }
                }
                let mid = world.transform_point3(glam::Vec3::new((lo[0]+hi[0])*0.5,(lo[1]+hi[1])*0.5,(lo[2]+hi[2])*0.5));
                let reach = glam::Vec3::from(hi).distance(glam::Vec3::from(lo)).max(100.0) * 1.5;
                for angle in [0.0f32, 0.7, 1.571, 2.4, std::f32::consts::PI, 4.0, 4.712, 5.5] {
                    let direction = glam::Vec3::new(angle.cos(),angle.sin(),0.0);
                    for extent in [[9.0f32,9.0,23.0],[34.0,34.0,78.0],[5.0,5.0,10.0]] {
                        for lift in [0.0f32, 15.0] {
                            let from = mid - direction*reach + glam::Vec3::Z*lift;
                            let to = mid + direction*reach + glam::Vec3::Z*lift;
                            queries.push((from.to_array(),to.to_array(),extent));
                        }
                    }
                }
                let owner = json!({"location":actor.placement.location,"rotation":actor.placement.rotation,"draw_scale":actor.draw_scale,
                    "scale":actor.draw_scale_3d,"pivot":shape.pre_pivot});
                hits += native.compare_model(&format!("{tile}:{}",actor.name),&model.tree,&model.render,Some((owner,world)),&queries);
                traces += queries.len();
                volumes += 1;
            }
            client.catalog().release_files();
        }
    }
    assert!(volumes > 0);
    eprintln!("{volumes} BlockingVolume, {traces} traços, {hits} acertos idênticos ao UModel::LineCheck com Owner");
}
