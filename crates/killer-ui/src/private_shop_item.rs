pub const DIALOG_PRICE: u32 = 0x25;
pub const DIALOG_QUANTITY: u32 = 0x26;
pub const DIALOG_CONFIRM_PRICE: u32 = 0x28;
pub const MESSAGE_PRICE: u32 = 0x142;
pub const MESSAGE_QUANTITY: u32 = 0x48;
pub const MESSAGE_PRICE_RANGE: u32 = 0x239;
pub const MESSAGE_NOT_ENOUGH: u32 = 0x40c;
pub const NOT_ENOUGH_COLOR: u32 = 0xffdcdcdc;
pub const REPLY_PRICE: u32 = 0xf000_0006;
pub const REPLY_QUANTITY: u32 = 0xf000_0007;
pub const REPLY_CONFIRM_PRICE: u32 = 0xf000_0008;
pub const MODE_VISIT: i32 = 0;
pub const MODE_MANAGE: i32 = 1;
pub const KIND_ORIGIN: i32 = 0;
pub const KIND_SELECTION: i32 = 1;
pub const PRICE_FLOOR_DIVISOR: i64 = 5;
pub const PRICE_CEILING_FACTOR: i64 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    pub mode: i32,
    pub peer_kind: i32,
    pub package_sale: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Item {
    pub item_id: i32,
    pub name: String,
    pub stackable: bool,
    pub count: i64,
    pub stock: i32,
    pub price: i32,
    pub reference_price: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dialog {
    pub kind: u32,
    pub message: u32,
    pub state: i32,
    pub item_name: String,
    pub item_id: i32,
    pub max: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Nothing,
    Move(i64),
    MoveAll,
    Dialog(Dialog),
    Message { id: u32, color: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Release {
    OverSelf,
    OverPeer,
    Elsewhere,
}

pub fn quantity_dialog(item: &Item, state: i32) -> Outcome {
    Outcome::Dialog(Dialog {
        kind: DIALOG_QUANTITY,
        message: MESSAGE_QUANTITY,
        state,
        item_name: item.name.clone(),
        item_id: item.item_id,
        max: i64::from(item.count as u32),
    })
}

pub fn price_dialog(item: &Item) -> Outcome {
    Outcome::Dialog(Dialog {
        kind: DIALOG_PRICE,
        message: MESSAGE_PRICE,
        state: 1,
        item_name: item.name.clone(),
        item_id: item.item_id,
        max: 0,
    })
}

fn range_dialog() -> Outcome {
    Outcome::Dialog(Dialog {
        kind: DIALOG_CONFIRM_PRICE,
        message: MESSAGE_PRICE_RANGE,
        state: 0,
        item_name: String::new(),
        item_id: 0,
        max: 0,
    })
}

fn more_than_one(item: &Item) -> bool {
    item.count > 1
}

pub fn drop_action(grid: Grid, item: &Item) -> Outcome {
    if grid.mode == MODE_VISIT {
        if grid.package_sale {
            return Outcome::MoveAll;
        }
        if !item.stackable {
            return Outcome::Move(1);
        }
        return quantity_dialog(item, 0);
    }
    if grid.peer_kind == KIND_ORIGIN {
        return price_dialog(item);
    }
    if !item.stackable || item.stock <= 1 {
        return Outcome::Move(1);
    }
    quantity_dialog(item, 2)
}

pub fn on_lbutton_up(grid: Grid, item: Option<&Item>, dragging: bool, release: Release) -> Outcome {
    let Some(item) = item else { return Outcome::Nothing };
    if !dragging || release != Release::OverPeer {
        return Outcome::Nothing;
    }
    drop_action(grid, item)
}

pub fn on_double_click(grid: Grid, item: Option<&Item>) -> Outcome {
    let Some(item) = item else { return Outcome::Nothing };
    if grid.mode == MODE_MANAGE && grid.peer_kind != KIND_ORIGIN {
        return if item.stackable { quantity_dialog(item, 2) } else { Outcome::Move(1) };
    }
    drop_action(grid, item)
}

pub fn on_price_answer(mode: i32, item: Option<&mut Item>, value: i64) -> Outcome {
    let Some(item) = item else { return Outcome::Nothing };
    if value <= 0 || mode != MODE_MANAGE {
        return Outcome::Nothing;
    }
    item.price = value as i32;
    let reference = i64::from(item.reference_price);
    if reference > 0 {
        let floor = reference / PRICE_FLOOR_DIVISOR;
        let ceiling = reference * PRICE_CEILING_FACTOR;
        if value <= floor || value >= ceiling {
            return range_dialog();
        }
    }
    if item.stackable && more_than_one(item) {
        return quantity_dialog(item, 1);
    }
    Outcome::Move(1)
}

pub fn on_quantity_answer(mode: i32, item: Option<&Item>, value: i64) -> Outcome {
    let Some(item) = item else { return Outcome::Nothing };
    if value <= 0 || !(mode == MODE_MANAGE || mode == MODE_VISIT) {
        return Outcome::Nothing;
    }
    if value <= item.count {
        Outcome::Move(value)
    } else {
        Outcome::Message { id: MESSAGE_NOT_ENOUGH, color: NOT_ENOUGH_COLOR }
    }
}

pub fn on_price_confirmed(item: Option<&Item>) -> Outcome {
    let Some(item) = item else { return Outcome::Nothing };
    if item.stackable && more_than_one(item) {
        return quantity_dialog(item, 0);
    }
    Outcome::Move(1)
}

pub fn reply_for(kind: u32) -> Option<u32> {
    match kind {
        DIALOG_PRICE => Some(REPLY_PRICE),
        DIALOG_QUANTITY => Some(REPLY_QUANTITY),
        DIALOG_CONFIRM_PRICE => Some(REPLY_CONFIRM_PRICE),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(stackable: bool, count: i64, stock: i32, reference: i32) -> Item {
        Item { item_id: 57, name: "Adena".into(), stackable, count, stock, price: 0, reference_price: reference }
    }

    const VISIT: Grid = Grid { mode: MODE_VISIT, peer_kind: KIND_SELECTION, package_sale: false };
    const MANAGE_TO_SELECTION: Grid = Grid { mode: MODE_MANAGE, peer_kind: KIND_SELECTION, package_sale: false };
    const MANAGE_TO_ORIGIN: Grid = Grid { mode: MODE_MANAGE, peer_kind: KIND_ORIGIN, package_sale: false };

    #[test]
    fn releasing_over_the_own_grid_or_outside_the_peer_does_nothing_100e8ab0() {
        let item = item(true, 10, 10, 0);
        for release in [Release::OverSelf, Release::Elsewhere] {
            assert_eq!(on_lbutton_up(VISIT, Some(&item), true, release), Outcome::Nothing);
        }
        assert_eq!(on_lbutton_up(VISIT, Some(&item), false, Release::OverPeer), Outcome::Nothing);
        assert_eq!(on_lbutton_up(VISIT, None, true, Release::OverPeer), Outcome::Nothing);
    }

    #[test]
    fn visit_mode_asks_the_quantity_of_a_stackable_item_with_the_count_as_maximum() {
        let outcome = on_lbutton_up(VISIT, Some(&item(true, 7, 3, 0)), true, Release::OverPeer);
        assert_eq!(
            outcome,
            Outcome::Dialog(Dialog { kind: 0x26, message: 0x48, state: 0, item_name: "Adena".into(), item_id: 57, max: 7 })
        );
        assert_eq!(on_lbutton_up(VISIT, Some(&item(false, 1, 1, 0)), true, Release::OverPeer), Outcome::Move(1));
        let package = Grid { package_sale: true, ..VISIT };
        assert_eq!(on_lbutton_up(package, Some(&item(true, 7, 7, 0)), true, Release::OverPeer), Outcome::MoveAll);
    }

    #[test]
    fn manage_mode_asks_the_price_when_the_peer_is_the_origin_grid_and_the_quantity_otherwise() {
        let stack = item(true, 9, 4, 100);
        assert_eq!(
            on_lbutton_up(MANAGE_TO_ORIGIN, Some(&stack), true, Release::OverPeer),
            Outcome::Dialog(Dialog { kind: 0x25, message: 0x142, state: 1, item_name: "Adena".into(), item_id: 57, max: 0 })
        );
        assert_eq!(
            on_lbutton_up(MANAGE_TO_SELECTION, Some(&stack), true, Release::OverPeer),
            Outcome::Dialog(Dialog { kind: 0x26, message: 0x48, state: 2, item_name: "Adena".into(), item_id: 57, max: 9 })
        );
        assert_eq!(on_lbutton_up(MANAGE_TO_SELECTION, Some(&item(true, 9, 1, 0)), true, Release::OverPeer), Outcome::Move(1));
        assert_eq!(on_lbutton_up(MANAGE_TO_SELECTION, Some(&item(false, 9, 9, 0)), true, Release::OverPeer), Outcome::Move(1));
    }

    #[test]
    fn double_click_uses_the_remembered_peer_without_testing_the_release_100e8e30() {
        assert_eq!(on_double_click(MANAGE_TO_ORIGIN, Some(&item(false, 1, 1, 0))), price_dialog(&item(false, 1, 1, 0)));
        assert_eq!(on_double_click(VISIT, Some(&item(false, 1, 1, 0))), Outcome::Move(1));
        assert_eq!(on_double_click(VISIT, None), Outcome::Nothing);
        assert_eq!(
            on_double_click(MANAGE_TO_SELECTION, Some(&item(true, 9, 1, 0))),
            Outcome::Dialog(Dialog { kind: 0x26, message: 0x48, state: 2, item_name: "Adena".into(), item_id: 57, max: 9 })
        );
        assert_eq!(on_double_click(MANAGE_TO_SELECTION, Some(&item(false, 9, 9, 0))), Outcome::Move(1));
    }

    #[test]
    fn the_price_must_lie_strictly_between_a_fifth_and_five_times_the_reference_100e9280() {
        let mut entry = item(false, 1, 1, 1000);
        assert_eq!(on_price_answer(MODE_MANAGE, Some(&mut entry), 200), range_dialog());
        assert_eq!(entry.price, 200);
        assert_eq!(on_price_answer(MODE_MANAGE, Some(&mut entry), 5000), range_dialog());
        assert_eq!(on_price_answer(MODE_MANAGE, Some(&mut entry), 201), Outcome::Move(1));
        assert_eq!(on_price_answer(MODE_MANAGE, Some(&mut entry), 4999), Outcome::Move(1));
        assert_eq!(entry.price, 4999);
        let mut free = item(false, 1, 1, 0);
        assert_eq!(on_price_answer(MODE_MANAGE, Some(&mut free), 1), Outcome::Move(1));
    }

    #[test]
    fn a_valid_price_on_a_stack_asks_the_quantity_with_state_1() {
        let mut stack = item(true, 5, 5, 0);
        assert_eq!(
            on_price_answer(MODE_MANAGE, Some(&mut stack), 10),
            Outcome::Dialog(Dialog { kind: 0x26, message: 0x48, state: 1, item_name: "Adena".into(), item_id: 57, max: 5 })
        );
        let mut single = item(true, 1, 1, 0);
        assert_eq!(on_price_answer(MODE_MANAGE, Some(&mut single), 10), Outcome::Move(1));
    }

    #[test]
    fn the_price_answer_ignores_zero_other_modes_and_missing_items() {
        let mut entry = item(false, 1, 1, 10);
        assert_eq!(on_price_answer(MODE_MANAGE, Some(&mut entry), 0), Outcome::Nothing);
        assert_eq!(on_price_answer(MODE_VISIT, Some(&mut entry), 50), Outcome::Nothing);
        assert_eq!(entry.price, 0);
        assert_eq!(on_price_answer(MODE_MANAGE, None, 50), Outcome::Nothing);
    }

    #[test]
    fn the_quantity_answer_moves_up_to_the_count_and_reports_1036_above_it_100e94a0() {
        let entry = item(true, 20, 20, 0);
        for mode in [MODE_VISIT, MODE_MANAGE] {
            assert_eq!(on_quantity_answer(mode, Some(&entry), 20), Outcome::Move(20));
            assert_eq!(on_quantity_answer(mode, Some(&entry), 1), Outcome::Move(1));
            assert_eq!(on_quantity_answer(mode, Some(&entry), 21), Outcome::Message { id: 0x40c, color: 0xffdcdcdc });
            assert_eq!(on_quantity_answer(mode, Some(&entry), 0), Outcome::Nothing);
        }
        assert_eq!(on_quantity_answer(2, Some(&entry), 5), Outcome::Nothing);
        assert_eq!(on_quantity_answer(MODE_MANAGE, None, 5), Outcome::Nothing);
    }

    #[test]
    fn confirming_the_price_range_asks_the_quantity_of_a_stack_or_moves_one_100e9140() {
        assert_eq!(
            on_price_confirmed(Some(&item(true, 3, 3, 0))),
            Outcome::Dialog(Dialog { kind: 0x26, message: 0x48, state: 0, item_name: "Adena".into(), item_id: 57, max: 3 })
        );
        assert_eq!(on_price_confirmed(Some(&item(true, 1, 1, 0))), Outcome::Move(1));
        assert_eq!(on_price_confirmed(Some(&item(false, 9, 9, 0))), Outcome::Move(1));
        assert_eq!(on_price_confirmed(None), Outcome::Nothing);
    }

    #[test]
    fn the_dialog_ok_table_1000afc0_answers_with_the_f0000006_to_f0000008_messages() {
        assert_eq!(reply_for(DIALOG_PRICE), Some(0xf0000006));
        assert_eq!(reply_for(DIALOG_QUANTITY), Some(0xf0000007));
        assert_eq!(reply_for(DIALOG_CONFIRM_PRICE), Some(0xf0000008));
        assert_eq!(reply_for(2), None);
    }
}
