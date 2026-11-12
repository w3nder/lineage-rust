use killer_ui::{delivery, shop};
use killer_ui::window::Rect;
use std::process::Command;

#[test]
#[ignore]
fn delivery_layout_and_background_match_executed_c4_oncreate_and_onpaint() {
    let output = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/delivery_layout_reference.py"))
        .output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let mut expected = Vec::new();
    let mut rects = |controls: &[(u32, Rect)]| {
        for (field, rect) in controls {
            expected.push(format!("rect {field} {} {} {} {}", rect.x, rect.y, rect.w, rect.h));
        }
    };
    rects(&[(0x7c, shop::grip()), (0xe8, shop::TOP_RECT), (0xec, shop::BOTTOM_RECT),
        (0xf4, shop::UP_RECT), (0xf8, shop::DOWN_RECT), (0xfc, shop::OK_RECT), (0x100, shop::CANCEL_RECT)]);
    let back = shop::BACK_RECT;
    expected.push(format!("back {} {} {} {} 0 0 {} {} {}", back.x, back.y, back.w, back.h, back.w, back.h, shop::STORE_BACK));
    for (field, rect) in [(0x7c, shop::grip()), (0xec, delivery::COMBO_RECT), (0xf0, delivery::OK_RECT), (0xf4, delivery::CANCEL_RECT)] {
        expected.push(format!("rect {field} {} {} {} {}", rect.x, rect.y, rect.w, rect.h));
    }
    expected.push(format!("back 0 20 {} {} 0 0 {} {} {}", delivery::SELECT_SIZE.0, delivery::SELECT_SIZE.1 - 20,
        delivery::SELECT_SIZE.0, delivery::SELECT_SIZE.1 - 20, delivery::SELECT_BACK));
    let actual = String::from_utf8(output.stdout).unwrap();
    assert_eq!(actual.lines().collect::<Vec<_>>(), expected);
}
