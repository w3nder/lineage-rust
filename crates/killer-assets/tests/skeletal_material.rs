use killer_assets::skeletal_material::{TextureNode, base_texture, sub_skin};
use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore]
fn find_base_texture_and_get_sub_skin_match_original_all_routes() {
    let mut cases = Vec::new();
    let mut expected = Vec::new();
    for combination in 0..625 {
        let mut selector = combination;
        let mut nodes = Vec::new();
        let mut encoded = Vec::new();
        for index in 0..4 {
            let kind = selector % 5;
            selector /= 5;
            let child = (index < 3).then_some(index + 1);
            nodes.push(match kind {
                0 => TextureNode::Texture,
                1 => TextureNode::FinalBlend { material: child },
                2 => TextureNode::Shader { diffuse: child },
                3 => TextureNode::Combiner { material_4c: child },
                _ => TextureNode::Other,
            });
            encoded.push((kind, child));
        }
        for root in [None, Some(0), Some(1), Some(2), Some(3)] {
            cases.push(json!({"nodes": encoded, "root": root}));
            expected.push(base_texture(&nodes, root));
        }
    }
    for pattern in 0..128 {
        let skins = std::array::from_fn(|index| (pattern & (1 << index) != 0).then_some(index));
        for skins in [None, Some(&skins)] {
            for index in [i32::MIN, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, i32::MAX] {
                cases.push(json!({"skins": skins, "index": index}));
                expected.push(sub_skin(skins, index));
            }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/skeletal_material_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&serde_json::to_vec(&cases).unwrap()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let actual: Vec<Option<usize>> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(*actual, expected, "{}", cases[index]);
    }
    eprintln!("{} cadeias comparadas com função original, IsA fornecido pelo harness", cases.len());
}
