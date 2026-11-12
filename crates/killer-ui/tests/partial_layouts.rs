use killer_ui::{board, command_channel, delivery, macros, multisell, party_match, pet_wnd, recipe,
    recipe_sell, recipe_shop, recipe_tree, shop, siege, special_maps, ssq, trade, train, tutorial};
use std::collections::BTreeSet;
use std::process::Command;

#[test]
#[ignore]
fn integrated_layouts_match_every_selected_field_in_executed_c4_profiles() {
    let output = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/partial_layout_reference.py"))
        .output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let actual: BTreeSet<_> = stdout.lines().collect();
    let mut delivery = shop::ShopView::with_sysstrings(|_| None);
    delivery.open(shop::ShopMode::Deliver, 0, 0, Vec::new());
    let mut warehouse = shop::ShopView::with_sysstrings(|_| None);
    warehouse.open(shop::ShopMode::Deposit, 0, 0, Vec::new());
    for (address, size) in [
        (0x1003f9e0u32, board::SIZE),
        (0x10052480u32, command_channel::SIZE),
        (0x10085d80u32, delivery::SELECT_SIZE),
        (0x100865c0u32, shop::SHOP_SIZE),
        (0x100a3bc0u32, multisell::SIZE),
        (0x100acc50u32, macros::EDIT_SIZE),
        (0x100b45e0u32, party_match::SIZE),
        (0x100bb360u32, special_maps::SIZE),
        (0x100deef0u32, pet_wnd::SIZE),
        (0x100df640u32, pet_wnd::summoned::SIZE),
        (0x100f1d70u32, recipe::SIZE),
        (0x100f2ae0u32, recipe_shop::SIZE),
        (0x100f32a0u32, recipe_sell::SIZE),
        (0x100f6100u32, recipe::SIZE),
        (0x100f6940u32, recipe::SIZE),
        (0x100f7100u32, recipe::SIZE),
        (0x100fc7c0u32, ssq::SIZE),
        (0x1010ced0u32, siege::SIZE),
        (0x10119780u32, shop::SHOP_SIZE),
        (0x1011b2b0u32, trade::TRADE_SIZE),
        (0x1011e780u32, train::SIZE),
        (0x1011f410u32, (tutorial::VIEWER_WIDTH, tutorial::VIEWER_CREATE_HEIGHT)),
        (0x10121010u32, special_maps::SIZE),
    ] {
        assert!(actual.contains(format!("size {address:08x} {} {}", size.0, size.1).as_str()), "size {address:08x}");
    }
    for (address, field, rect) in [
        (0x1003f9e0u32, 0xe8u32, board::VIEWER_RECT),
        (0x1003f9e0u32, 0xf4u32, board::FAVORITE_RECT),
        (0x10052480u32, 0xf8u32, command_channel::INPUT_RECT),
        (0x10052480u32, 0xfcu32, command_channel::CHECK_RECT),
        (0x10085d80u32, 0xecu32, delivery::COMBO_RECT),
        (0x100865c0u32, 0xe8u32, delivery.top_grid.rect),
        (0x100865c0u32, 0xecu32, delivery.bottom_grid.rect),
        (0x100a3bc0u32, 0xe8u32, multisell::LIST_RECT),
        (0x100a3bc0u32, 0xecu32, multisell::PRODUCT_RECT),
        (0x100a3bc0u32, 0xf0u32, multisell::NEEDED_RECT),
        (0x100a3bc0u32, 0xf4u32, multisell::QUANTITY_RECT),
        (0x100acc50u32, 0x108u32, macros::EDIT_NAME_RECT),
        (0x100acc50u32, 0x10cu32, macros::EDIT_ACRONYM_RECT),
        (0x100acc50u32, 0x104u32, macros::EDIT_MORE_INFO_RECT),
        (0x100acc50u32, 0xfcu32, macros::EDIT_PREV_RECT),
        (0x100acc50u32, 0x100u32, macros::EDIT_NEXT_RECT),
        (0x100acc50u32, 0xecu32, macros::EDIT_COMMAND_LIST_RECT),
        (0x100acc50u32, 0x118u32, macros::EDIT_HELP_RECT),
        (0x100acc50u32, 0x114u32, macros::EDIT_SAVE_RECT),
        (0x100acc50u32, 0x11cu32, macros::EDIT_CANCEL_RECT),
        (0x100b45e0u32, 0xf4u32, party_match::LIST_RECT),
        (0x100b45e0u32, 0xecu32, party_match::LOCATION_RECT),
        (0x100b45e0u32, 0xf0u32, party_match::LEVEL_RECT),
        (0x100b45e0u32, 0xf8u32, party_match::BUTTON_RECTS[0]),
        (0x100b45e0u32, 0xfcu32, party_match::BUTTON_RECTS[1]),
        (0x100b45e0u32, 0x100u32, party_match::BUTTON_RECTS[2]),
        (0x100b45e0u32, 0x104u32, party_match::BUTTON_RECTS[3]),
        (0x100b45e0u32, 0x108u32, party_match::BUTTON_RECTS[4]),
        (0x100deef0u32, 0x1d4u32, pet_wnd::NAME_BUTTON),
        (0x100deef0u32, 0x1c4u32, pet_wnd::PAGE_RECT),
        (0x100df640u32, 0xf8u32, pet_wnd::summoned::PAGE_RECT),
        (0x100f6100u32, 0x174u32, recipe::BOOK_GRID_RECT),
        (0x100f6940u32, 0xe8u32, recipe::MATERIAL_RECT),
        (0x100f6940u32, 0x1d64u32, recipe::TREE_BUTTON_RECT),
        (0x100f6940u32, 0x1d68u32, recipe::CREATE_RECT),
        (0x100f6940u32, 0x1d6cu32, recipe::BACK_BUTTON_RECT),
        (0x100f6940u32, 0x1d70u32, recipe::CLOSE_BUTTON_RECT),
        (0x100f1d70u32, 0xe8u32, recipe::MATERIAL_RECT),
        (0x100f1d70u32, 0x1d68u32, recipe::TREE_BUTTON_RECT),
        (0x100f1d70u32, 0x1d6cu32, recipe::CREATE_RECT),
        (0x100f1d70u32, 0x1d70u32, recipe::BACK_BUTTON_RECT),
        (0x100f1d70u32, 0x1d74u32, recipe::CLOSE_BUTTON_RECT),
        (0x100f2ae0u32, 0xe8u32, recipe_shop::TOP_RECT),
        (0x100f2ae0u32, 0xecu32, recipe_shop::BOTTOM_RECT),
        (0x100f2ae0u32, 0xfcu32, recipe_shop::UP_RECT),
        (0x100f2ae0u32, 0x100u32, recipe_shop::DOWN_RECT),
        (0x100f2ae0u32, 0x10cu32, recipe_shop::ABORT_RECT),
        (0x100f2ae0u32, 0x108u32, recipe_shop::MESSAGE_RECT),
        (0x100f2ae0u32, 0x104u32, recipe_shop::START_RECT),
        (0x100f32a0u32, 0xe8u32, recipe_sell::LIST),
        (0x100f32a0u32, 0xecu32, recipe_sell::CLOSE),
        (0x100f7100u32, 0xecu32, recipe_tree::CONTROL),
        (0x100f7100u32, 0x1d3cu32, recipe_tree::CLOSE),
        (0x100fc7c0u32, 0xf0u32, ssq::PAGE),
        (0x1010ced0u32, 0x70cu32, siege::PAGE),
        (0x1010ced0u32, 0x708u32, siege::TABS),
        (0x10119780u32, 0xecu32, warehouse.top_grid.rect),
        (0x10119780u32, 0xe8u32, warehouse.bottom_grid.rect),
        (0x1011b2b0u32, 0xe8u32, trade::MINE_RECT),
        (0x1011b2b0u32, 0xecu32, trade::OFFERED_RECT),
        (0x1011b2b0u32, 0xf0u32, trade::THEIRS_RECT),
        (0x1011b2b0u32, 0xf8u32, trade::ADD_RECT),
        (0x1011b2b0u32, 0xfcu32, trade::OK_RECT),
        (0x1011b2b0u32, 0x100u32, trade::CANCEL_RECT),
        (0x1011e780u32, 0xe8u32, train::LIST_PAGE),
        (0x1011e780u32, 0xecu32, train::INFO_PAGE),
        (0x1011f410u32, 0xe8u32, tutorial::TutorialViewer::default().html_bounds),
    ] {
        let line = format!("rect {address:08x} {field:x} {} {} {} {}", rect.x, rect.y, rect.w, rect.h);
        assert!(actual.contains(line.as_str()), "{line}");
    }
    assert_ne!(killer_ui::npc_viewer::NPC_VIEWER_SIZE, board::SIZE);
}
