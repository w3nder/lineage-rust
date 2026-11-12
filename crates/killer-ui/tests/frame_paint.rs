use std::process::Command;

use killer_ui::draw::DrawList;
use killer_ui::font::{BitmapFont, Glyphs};
use killer_ui::frame::{FrameCtrl, FrameMode};

#[test]
#[ignore]
fn frame_on_paint_10011d80_matches_native_slices_for_all_sampled_sizes() {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/frame_paint_oracle.py");
    let output = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let text = String::from_utf8(output.stdout).unwrap();
    let font = BitmapFont { atlas: "font".into(), glyphs: Glyphs {
        tex_width: 1, tex_height: 1, glyphs: Default::default(),
    }, line_height: 13 };
    let mut cases = 0;
    for case in text.split("case\t").skip(1) {
        let mut lines = case.lines();
        let header: Vec<i32> = lines.next().unwrap().split('\t').map(|v| v.parse().unwrap()).collect();
        let mode = match (header[0], header[1]) {
            (0, 0) => FrameMode::Title,
            (0, 1) => FrameMode::ThinBar,
            (1, 1) => FrameMode::VerticalGrip,
            _ => panic!("unexpected mode"),
        };
        let mut draw = DrawList::default();
        FrameCtrl::new(mode).paint(&mut draw, 0, 0, header[2], header[3], &font, 1.0);
        let mut expected: Vec<(String, [f32; 8])> = lines.map(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            assert_eq!(fields.len(), 9);
            (fields[8].to_string(), std::array::from_fn(|index| fields[index].parse().unwrap()))
        }).collect();
        let mut actual: Vec<_> = draw.quads.iter().filter(|q| q.w > 0.0 && q.h > 0.0).map(|q| (q.texture.to_string(), [q.x, q.y, q.w, q.h, q.u, q.v, q.uw, q.vh])).collect();
        expected.sort_by(|a, b| a.0.cmp(&b.0));
        actual.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(actual, expected, "{header:?}");
        cases += 1;
    }
    assert_eq!(cases, 1475);
}
