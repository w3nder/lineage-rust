use killer_ui::draw::DrawList;
use killer_ui::html_form::paint_background;
use killer_ui::window::Rect;
use std::process::Command;

#[test]
#[ignore]
fn html_edit_10049890_and_multiedit_10048090_backgrounds_match_native_calls() {
    let client = std::env::var("KILLER_CLIENT_DIR").unwrap();
    let output = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/html_form_paint_reference.py"))
        .env("KILLER_CLIENT_DIR", client)
        .output().unwrap();
    assert!(output.status.success(), "{:?}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    let output = String::from_utf8(output.stdout).unwrap();
    let mut count = 0;
    let mut textures = std::collections::BTreeSet::new();
    for line in output.lines() {
        let case: serde_json::Value = serde_json::from_str(line).unwrap();
        let width = case["width"].as_i64().unwrap() as i32;
        let height = case["height"].as_i64().unwrap() as i32;
        let mut list = DrawList::default();
        paint_background(&mut list, Rect::new(0, 0, width, height), case["multi"].as_bool().unwrap(), case["enabled"].as_bool().unwrap());
        let draws = case["draws"].as_array().unwrap();
        assert_eq!(list.quads.len(), draws.len(), "{case}");
        for (quad, draw) in list.quads.iter().zip(draws) {
            let args = draw["quad"].as_array().unwrap();
            let native: Vec<f32> = args[..8].iter().map(|v| v.as_str().unwrap().parse().unwrap()).collect();
            assert_eq!([quad.x, quad.y, quad.w, quad.h, quad.u, quad.v, quad.uw, quad.vh].as_slice(), native, "{case}");
            assert_eq!(quad.texture, args[8].as_str().unwrap());
            assert_eq!(quad.color, [1.0; 4]);
            assert_eq!(args[9].as_str().unwrap(), "255");
            textures.insert(quad.texture.clone());
        }
        count += 1;
    }
    assert_eq!(count, 152);
    assert_eq!(textures.len(), 24);
}
