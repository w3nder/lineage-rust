use killer_assets::{
    client::Client,
    scene::{
        terrain::load_grid,
        terrain_normals::{face_buffer, sector_point, update_faces, vertex_normal},
    },
};
use serde_json::{Value, json};
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

fn oracle(case: &Value) -> Value {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/projector_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script)
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!([case])).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    serde_json::from_slice::<Vec<Value>>(&output.stdout).unwrap().remove(0)
}

fn compare(actual: [f32; 3], expected: [f32; 3], context: &str) {
    for (a, e) in actual.into_iter().zip(expected) {
        assert!((a - e).abs() <= 2e-6, "{context}: {actual:?} != {expected:?}");
    }
}

#[test]
#[ignore]
fn get_heightmap_106d2130_matches_native_formats_and_entire_rgba_heightmap() {
    use ue2_assets::TextureFormat;
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").unwrap());
    let mut client = Client::open(&root).unwrap();
    let loaded = client.load_level("16_13").unwrap();
    let terrain = &loaded.level.terrains[0];
    let raw = client.catalog().texture_raw(&terrain.heightmap).unwrap();
    assert_eq!(raw.format, TextureFormat::Rgba8);
    let mip = &raw.mips[0];
    let mut cases = vec![
        (TextureFormat::Rgba8, 5, mip.data.clone(), terrain.base_height, [mip.width, mip.height]),
        (TextureFormat::G16, 10, (0..=u16::MAX).flat_map(u16::to_le_bytes).collect(), 0, [256, 256]),
        (TextureFormat::P8, 0, (0..=u8::MAX).collect(), 1024, [256, 1]),
        (TextureFormat::Dxt1, 3, vec![255; 4], 1024, [4, 1]),
    ];
    let mut colors = Vec::new();
    for b in [0, 1, 127, 128, 254, 255] {
        for g in [0, 1, 127, 128, 254, 255] {
            for r in [0, 1, 127, 128, 254, 255] {
                for alpha in [0, 255] {
                    colors.extend([b, g, r, alpha]);
                }
            }
        }
    }
    for base in [-65537, -32768, -1, 0, 1, 32767, 65535, 65536] {
        cases.push((TextureFormat::Rgba8, 5, colors.clone(), base, [432, 1]));
    }
    let mut count = 0;
    for (format, code, bytes, base, [width, height]) in cases {
        let points: Vec<_> = (0..height).flat_map(|y| (0..width).map(move |x| [x, y])).collect();
        let data: String = bytes.iter().map(|v| format!("{v:02x}")).collect();
        let expected: Vec<u16> = serde_json::from_value(oracle(&json!({"heightmap":{
            "format":code, "data":data, "size":[width,height], "base":base, "points":points
        }})))
        .unwrap();
        let actual = l2_catalog::heightmap::samples(format, &bytes, base, (width * height) as usize).unwrap();
        assert_eq!(actual, expected, "{format:?}, base {base}");
        count += actual.len();
    }
    eprintln!("{count} amostras contra GetHeightmap; inclui todos os pixels RGBA8 de 16_13");
}

#[test]
#[ignore]
fn update_vertices_106d6df0_and_sector_normals_match_entire_projector_heightmaps() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").unwrap());
    let mut client = Client::open(&root).unwrap();
    for tile in ["16_10", "20_17", "24_20"] {
        let loaded = client.load_level(tile).unwrap();
        for terrain in &loaded.level.terrains {
            let grid = load_grid(&mut client, terrain).unwrap();
            let [width, height] = grid.heightmap;
            assert_eq!([width, height], [256, 256]);
            let vertices: Vec<_> = (0..height)
                .flat_map(|y| {
                    (0..width).map({
                        let grid = &grid;
                        move |x| grid.position(x, y)
                    })
                })
                .collect();
            let mut points: Vec<_> = (0..height)
                .flat_map(|y| {
                    (0..width).map(move |x| {
                        json!({
                            "offset":[0,0], "local":[x,y], "inverted":false
                        })
                    })
                })
                .collect();
            for inverted in [false, true] {
                for available in [false, true] {
                    for offset in [[0, 0], [224, 240], [240, 224], [240, 240]] {
                        for local in [[0, 0], [0, 16], [16, 0], [16, 16]] {
                            points
                                .push(json!({"offset":offset,"local":local,"inverted":inverted,"available":available}));
                        }
                    }
                }
            }
            let case = json!({"terrain_normals":{"size":[width,height],"vertices":vertices,"edge_turn":terrain.edge_turn,"points":points}});
            let expected = oracle(&case);
            let expected_faces: Vec<[f32; 6]> = serde_json::from_value(expected["faces"].clone()).unwrap();
            let expected_normals: Vec<[f32; 3]> = serde_json::from_value(expected["normals"].clone()).unwrap();
            let mut faces = vec![[[0.0; 3]; 2]; width * height];
            update_faces(&vertices, [width, height], &terrain.edge_turn, &mut faces);
            assert_eq!(faces.len(), expected_faces.len());
            for (i, (actual, expected)) in faces.iter().zip(expected_faces).enumerate() {
                compare(actual[0], [expected[0], expected[1], expected[2]], &format!("{tile} face {i}/0"));
                compare(actual[1], [expected[3], expected[4], expected[5]], &format!("{tile} face {i}/1"));
            }
            assert_eq!(points.len(), expected_normals.len());
            for (point, expected) in points.iter().zip(expected_normals) {
                let offset = serde_json::from_value(point["offset"].clone()).unwrap();
                let local = serde_json::from_value(point["local"].clone()).unwrap();
                let source = if point["available"] == false { &[][..] } else { &faces };
                compare(
                    vertex_normal(
                        source,
                        width,
                        width * height,
                        sector_point(offset, local),
                        point["inverted"].as_bool().unwrap(),
                    ),
                    expected,
                    &format!("{tile}: {point}"),
                );
            }
            assert_eq!(face_buffer(terrain, &grid).unwrap(), faces);
            eprintln!("{tile}: {} pares de faces e {} normais contra engine/core", faces.len(), points.len());
        }
    }
}
