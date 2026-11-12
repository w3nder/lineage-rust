use crate::private_shop_item::{self as psi, Dialog, Outcome};
use crate::shop::{ShopEntry, ShopView, Side};

pub const DIALOG_MESSAGE_SELL: u32 = 0x27;
pub const DIALOG_BUY_CAUTION: u32 = 0x29;
pub const DIALOG_BUY_PRICE: u32 = 0x2a;
pub const DIALOG_BUY_AMOUNT: u32 = 0x2b;
pub const DIALOG_MESSAGE_BUY: u32 = 0x2c;
pub const DIALOG_BUY_PRICE_CAUTION: u32 = 0x2d;
pub const DIALOG_SELL_CAUTION: u32 = 0x2e;
pub const REPLY_BUY_CONFIRMED: u32 = 0xf000_000a;
pub const REPLY_BUY_PRICE: u32 = 0xf000_000b;
pub const REPLY_BUY_AMOUNT: u32 = 0xf000_000c;
pub const REPLY_BUY_PRICE_OK: u32 = 0xf000_000d;
pub const REPLY_SELL_CONFIRMED: u32 = 0xf000_000f;
pub const MESSAGE_MOVE_HOW_MANY: u32 = 72;
pub const MESSAGE_STORE: u32 = 334;
pub const MESSAGE_PRICE_CAUTION: u32 = 569;
pub const MESSAGE_BUY_HOW_MANY: u32 = 570;
pub const MESSAGE_BUY_HOW_MANY_RETURN: u32 = 571;
pub const MESSAGE_SET_BUY_PRICE: u32 = 585;
pub const MESSAGE_TOO_MANY_WANTED: u32 = 1036;
pub const NOTICE_REFUSED: u32 = 0x559;
pub const NOTICE_REFUSED_DIALOG: u32 = 0x1f;
pub const ITEM_CAPACITY: usize = 160;
pub const TOTAL_LIMIT: i64 = 2_000_000_000;
pub const BUY_AMOUNT_LIMIT: i64 = 100_000_000;
pub const NON_STACKABLE_COPIES: i64 = 100;
pub const PRICE_FLOOR_DIVISOR: i32 = 5;
pub const PRICE_CEILING_FACTOR: i32 = 5;

pub fn is_unusual_price(price: i32, reference: i32) -> bool {
    reference > 0 && (price <= reference / PRICE_FLOOR_DIVISOR || price >= reference.wrapping_mul(PRICE_CEILING_FACTOR))
}

pub fn check_total(key: i32, count: i64, price: i32, entries: &[ShopEntry]) -> bool {
    let mut sum: i32 = 0;
    let mut step = 0;
    while step < count {
        sum = sum.wrapping_add(price);
        if i64::from(sum) > TOTAL_LIMIT || sum < 0 {
            return false;
        }
        step += 1;
    }
    let mut total: i32 = 0;
    for entry in entries {
        let entry_price = if entry.grid.item_id == key { price } else { entry.price as i32 };
        total = total.wrapping_add((entry.quantity as i32).wrapping_mul(entry_price));
    }
    let both = sum.wrapping_add(total);
    i64::from(both) <= TOTAL_LIMIT && both >= 0
}

pub fn shop_arrow_down(mode: i32, package_sale: bool, item: Option<&psi::Item>) -> Outcome {
    if mode == psi::MODE_VISIT {
        if package_sale {
            return Outcome::MoveAll;
        }
        let Some(item) = item else { return Outcome::Nothing };
        if !item.stackable {
            return Outcome::Move(1);
        }
        return psi::quantity_dialog(item, 0);
    }
    match item {
        Some(item) if mode == psi::MODE_MANAGE => psi::price_dialog(item),
        _ => Outcome::Nothing,
    }
}

pub fn shop_arrow_up(mode: i32, package_sale: bool, item: Option<&psi::Item>) -> Outcome {
    if mode == psi::MODE_VISIT && package_sale {
        return Outcome::MoveAll;
    }
    if mode != psi::MODE_VISIT && mode != psi::MODE_MANAGE {
        return Outcome::Nothing;
    }
    let Some(item) = item else { return Outcome::Nothing };
    if item.stackable && item.count > 1 {
        psi::quantity_dialog(item, 0)
    } else {
        Outcome::Move(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct BuyItem {
    pub item_id: i32,
    pub name: String,
    pub stackable: bool,
    pub stock: i64,
    pub quantity: i64,
    pub wanted: i64,
    pub price: i32,
    pub reference_price: i32,
}

fn amount_dialog(item: &BuyItem, message: u32, max: i64) -> Outcome {
    Outcome::Dialog(Dialog {
        kind: DIALOG_BUY_AMOUNT,
        message,
        state: 0,
        item_name: item.name.clone(),
        item_id: item.item_id,
        max: i64::from(max as i32),
    })
}

fn buy_price_dialog(item: &BuyItem) -> Outcome {
    Outcome::Dialog(Dialog {
        kind: DIALOG_BUY_PRICE,
        message: MESSAGE_SET_BUY_PRICE,
        state: 0,
        item_name: String::new(),
        item_id: item.item_id,
        max: 0,
    })
}

fn caution_dialog(kind: u32) -> Dialog {
    Dialog { kind, message: MESSAGE_PRICE_CAUTION, state: 0, item_name: String::new(), item_id: 0, max: 0 }
}

pub fn buy_drop(mode: i32, peer_kind: i32, item: &BuyItem, double_click: bool) -> Outcome {
    if mode == psi::MODE_VISIT {
        if item.stock > 0 {
            if item.stackable {
                return amount_dialog(item, MESSAGE_MOVE_HOW_MANY, item.quantity);
            }
            return Outcome::Move(1);
        }
        return Outcome::Nothing;
    }
    if peer_kind == psi::KIND_ORIGIN {
        return buy_price_dialog(item);
    }
    let asks = item.stackable && (double_click || item.stock > 1);
    if asks { amount_dialog(item, MESSAGE_BUY_HOW_MANY_RETURN, item.quantity) } else { Outcome::Move(1) }
}

pub fn buy_arrow_up(mode: i32, item: Option<&BuyItem>) -> Outcome {
    let Some(item) = item else { return Outcome::Nothing };
    if mode == psi::MODE_VISIT {
        if item.stock < 1 {
            return Outcome::Nothing;
        }
        if item.stackable && item.quantity > 1 {
            return amount_dialog(item, MESSAGE_MOVE_HOW_MANY, item.quantity);
        }
        return Outcome::Move(1);
    }
    if mode != psi::MODE_MANAGE {
        return Outcome::Nothing;
    }
    if item.stackable && item.quantity > 1 {
        amount_dialog(item, MESSAGE_BUY_HOW_MANY_RETURN, item.quantity)
    } else {
        Outcome::Move(1)
    }
}

pub fn buy_arrow_down(mode: i32, item: Option<&BuyItem>) -> Outcome {
    let Some(item) = item else { return Outcome::Nothing };
    if mode == psi::MODE_VISIT {
        if item.stock < 1 {
            return Outcome::Nothing;
        }
        if !item.stackable {
            return Outcome::Move(1);
        }
        return amount_dialog(item, MESSAGE_MOVE_HOW_MANY, item.quantity);
    }
    if mode == psi::MODE_MANAGE { buy_price_dialog(item) } else { Outcome::Nothing }
}

pub fn on_buy_price(mode: i32, item: Option<&mut BuyItem>, value: i64) -> Outcome {
    let Some(item) = item else { return Outcome::Nothing };
    if value == 0 || mode != psi::MODE_MANAGE || value < 0 {
        return Outcome::Nothing;
    }
    let price = value as i32;
    item.price = price;
    let reference = item.reference_price;
    if reference < 1 || (reference / PRICE_FLOOR_DIVISOR < price && price < reference.wrapping_mul(PRICE_CEILING_FACTOR)) {
        return amount_dialog(item, MESSAGE_BUY_HOW_MANY, item.stock);
    }
    Outcome::Dialog(caution_dialog(DIALOG_BUY_PRICE_CAUTION))
}

pub fn on_buy_amount(mode: i32, item: Option<&BuyItem>, value: i64) -> Outcome {
    let Some(item) = item else { return Outcome::Nothing };
    let amount = value as i32;
    if amount == 0 {
        return Outcome::Nothing;
    }
    if mode == psi::MODE_MANAGE && amount > 0 {
        return Outcome::Move(i64::from(amount).min(BUY_AMOUNT_LIMIT));
    }
    if mode == psi::MODE_VISIT && amount > 0 {
        if i64::from(amount) <= item.wanted {
            return Outcome::Move(i64::from(amount));
        }
        return Outcome::Message { id: MESSAGE_TOO_MANY_WANTED, color: psi::NOT_ENOUGH_COLOR };
    }
    Outcome::Nothing
}

pub fn on_buy_price_ok(item: Option<&BuyItem>) -> Outcome {
    match item {
        Some(item) => amount_dialog(item, MESSAGE_BUY_HOW_MANY, item.stock),
        None => Outcome::Nothing,
    }
}

pub fn buy_reply_for(kind: u32) -> Option<u32> {
    match kind {
        DIALOG_BUY_PRICE => Some(REPLY_BUY_PRICE),
        DIALOG_BUY_AMOUNT => Some(REPLY_BUY_AMOUNT),
        DIALOG_BUY_PRICE_CAUTION => Some(REPLY_BUY_PRICE_OK),
        _ => None,
    }
}

pub fn caution_for_start(window_buy: bool) -> Dialog {
    caution_dialog(if window_buy { DIALOG_SELL_CAUTION } else { DIALOG_BUY_CAUTION })
}

impl ShopView {
    pub fn sell_move(&mut self, side: Side, index: usize, count: i64) -> bool {
        if count <= 0 || index >= self.entries(side).len() {
            return false;
        }
        let item = self.entries(side)[index].clone();
        self.grid_of(side).selected = None;
        let going_on_sale = self.private_manage && side == Side::Top;
        if going_on_sale && !check_total(item.grid.item_id, count, item.price as i32, &self.bottom) {
            return false;
        }
        let (source, dest) = match side {
            Side::Top => (&mut self.top, &mut self.bottom),
            Side::Bottom => (&mut self.bottom, &mut self.top),
        };
        let same = if item.stackable() { dest.iter_mut().find(|entry| entry.grid.item_id == item.grid.item_id) } else { None };
        if let Some(same) = same {
            same.quantity += count;
            if going_on_sale {
                same.price = item.price;
            }
        } else if dest.len() < ITEM_CAPACITY {
            dest.push(item.with_quantity(count));
        }
        if item.stackable() && count < item.quantity {
            source[index].quantity -= count;
        } else {
            source.remove(index);
        }
        self.top_grid.update_rows(self.top.len());
        self.bottom_grid.update_rows(self.bottom.len());
        true
    }

    pub fn move_all(&mut self, side: Side) {
        let moved = std::mem::take(self.entries_mut(side));
        *self.entries_mut(if side == Side::Top { Side::Bottom } else { Side::Top }) = moved;
        self.top_grid.selected = None;
        self.bottom_grid.selected = None;
        self.top_grid.update_rows(self.top.len());
        self.bottom_grid.update_rows(self.bottom.len());
    }

    pub fn buy_move(&mut self, side: Side, index: usize, count: i64) -> bool {
        if count <= 0 || index >= self.entries(side).len() {
            return false;
        }
        let item = self.entries(side)[index].clone();
        self.grid_of(side).selected = None;
        if self.private_manage {
            if side == Side::Top {
                if !check_total(item.grid.object_id, count, item.price as i32, &self.bottom) {
                    return false;
                }
                if !item.stackable() {
                    self.bottom.retain(|entry| !(entry.grid.item_id == item.grid.item_id && entry.grid.enchant == item.grid.enchant));
                    for _ in 0..count.min(NON_STACKABLE_COPIES) {
                        if self.bottom.len() < ITEM_CAPACITY {
                            self.bottom.push(item.with_quantity(1));
                        }
                    }
                } else if let Some(same) = self.bottom.iter_mut().find(|entry| entry.grid.item_id == item.grid.item_id) {
                    same.quantity += count;
                    same.price = item.price;
                } else if self.bottom.len() < ITEM_CAPACITY {
                    self.bottom.push(item.with_quantity(count));
                }
            } else if item.stackable() && count < item.quantity {
                self.bottom[index].quantity -= count;
            } else {
                self.bottom.remove(index);
            }
        } else {
            let (source, dest) = match side {
                Side::Top => (&mut self.top, &mut self.bottom),
                Side::Bottom => (&mut self.bottom, &mut self.top),
            };
            let same = if item.stackable() { dest.iter_mut().find(|entry| entry.grid.item_id == item.grid.item_id) } else { None };
            if let Some(same) = same {
                same.quantity += count;
                same.wanted += count as i32;
            } else if dest.len() < ITEM_CAPACITY {
                dest.push(item.with_quantity(count));
            }
            if item.stackable() && count < item.quantity {
                source[index].wanted -= count as i32;
                source[index].quantity -= count;
            } else {
                source.remove(index);
            }
        }
        self.top_grid.update_rows(self.top.len());
        self.bottom_grid.update_rows(self.bottom.len());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item_grid::GridEntry;

    fn item(stackable: bool, stock: i64, quantity: i64, wanted: i64, reference: i32) -> BuyItem {
        BuyItem { item_id: 57, name: "Adena".into(), stackable, stock, quantity, wanted, price: 0, reference_price: reference }
    }

    fn entry(item_id: i32, quantity: i64, stock: i32, price: i64, stackable: bool) -> ShopEntry {
        ShopEntry {
            grid: GridEntry { item_id, count: stock, ..Default::default() },
            quantity,
            stock,
            price,
            consume_type: u32::from(stackable),
            wanted: -1,
            ..Default::default()
        }
    }

    #[test]
    fn an_unusual_price_is_a_fifth_or_less_or_five_times_or_more_of_the_reference_100e6c70() {
        assert!(!is_unusual_price(1, 0));
        assert!(is_unusual_price(200, 1000));
        assert!(!is_unusual_price(201, 1000));
        assert!(!is_unusual_price(4999, 1000));
        assert!(is_unusual_price(5000, 1000));
    }

    #[test]
    fn check_total_refuses_a_sum_or_a_list_total_beyond_two_billion_100e7a40() {
        let listed = [entry(1, 3, 3, 500_000_000, true)];
        assert!(check_total(0, 2, 100_000_000, &listed));
        assert!(!check_total(0, 1, 600_000_000, &listed));
        assert!(!check_total(0, 3, 1_000_000_000, &[]));
        assert!(check_total(0, 0, 5, &[]));
        assert!(check_total(1, 1, 100, &listed));
    }

    #[test]
    fn the_sell_window_arrows_follow_the_native_up_and_down_100e8650_100e8920() {
        let stack = psi::Item { item_id: 57, name: "A".into(), stackable: true, count: 5, stock: 5, price: 0, reference_price: 0 };
        let single = psi::Item { stackable: false, count: 1, ..stack.clone() };
        assert_eq!(shop_arrow_down(psi::MODE_VISIT, true, None), Outcome::MoveAll);
        assert_eq!(shop_arrow_down(psi::MODE_VISIT, false, Some(&single)), Outcome::Move(1));
        assert_eq!(shop_arrow_down(psi::MODE_VISIT, false, Some(&stack)), psi::quantity_dialog(&stack, 0));
        assert_eq!(shop_arrow_down(psi::MODE_MANAGE, false, Some(&single)), psi::price_dialog(&single));
        assert_eq!(shop_arrow_down(psi::MODE_MANAGE, false, None), Outcome::Nothing);
        assert_eq!(shop_arrow_up(psi::MODE_VISIT, true, None), Outcome::MoveAll);
        assert_eq!(shop_arrow_up(psi::MODE_MANAGE, true, Some(&stack)), psi::quantity_dialog(&stack, 0));
        assert_eq!(shop_arrow_up(psi::MODE_MANAGE, false, Some(&single)), Outcome::Move(1));
        assert_eq!(shop_arrow_up(psi::MODE_VISIT, false, None), Outcome::Nothing);
    }

    #[test]
    fn buy_window_drops_pick_the_dialog_of_the_native_grid_100e5910_100e5c90() {
        let stack = item(true, 4, 9, 6, 100);
        let single = item(false, 1, 1, 1, 100);
        let ask = |message, max| {
            Outcome::Dialog(Dialog { kind: 0x2b, message, state: 0, item_name: "Adena".into(), item_id: 57, max })
        };
        assert_eq!(buy_drop(psi::MODE_VISIT, psi::KIND_SELECTION, &stack, false), ask(72, 9));
        assert_eq!(buy_drop(psi::MODE_VISIT, psi::KIND_SELECTION, &single, false), Outcome::Move(1));
        assert_eq!(buy_drop(psi::MODE_VISIT, psi::KIND_SELECTION, &item(true, 0, 9, 6, 0), false), Outcome::Nothing);
        assert_eq!(
            buy_drop(psi::MODE_MANAGE, psi::KIND_ORIGIN, &stack, false),
            Outcome::Dialog(Dialog { kind: 0x2a, message: 585, state: 0, item_name: String::new(), item_id: 57, max: 0 })
        );
        assert_eq!(buy_drop(psi::MODE_MANAGE, psi::KIND_SELECTION, &stack, false), ask(571, 9));
        assert_eq!(buy_drop(psi::MODE_MANAGE, psi::KIND_SELECTION, &item(true, 1, 1, 0, 0), false), Outcome::Move(1));
        assert_eq!(buy_drop(psi::MODE_MANAGE, psi::KIND_SELECTION, &item(true, 1, 1, 0, 0), true), ask(571, 1));
    }

    #[test]
    fn the_price_answer_asks_the_amount_or_the_caution_100e38c0() {
        let mut entry = item(true, 12, 12, 0, 1000);
        assert_eq!(
            on_buy_price(psi::MODE_MANAGE, Some(&mut entry), 500),
            Outcome::Dialog(Dialog { kind: 0x2b, message: 570, state: 0, item_name: "Adena".into(), item_id: 57, max: 12 })
        );
        assert_eq!(entry.price, 500);
        assert_eq!(on_buy_price(psi::MODE_MANAGE, Some(&mut entry), 200), Outcome::Dialog(caution_dialog(0x2d)));
        assert_eq!(on_buy_price(psi::MODE_MANAGE, Some(&mut entry), 5000), Outcome::Dialog(caution_dialog(0x2d)));
        assert_eq!(on_buy_price(psi::MODE_MANAGE, Some(&mut entry), 0), Outcome::Nothing);
        assert_eq!(on_buy_price(psi::MODE_VISIT, Some(&mut entry), 10), Outcome::Nothing);
        let mut free = item(true, 3, 3, 0, 0);
        assert!(matches!(on_buy_price(psi::MODE_MANAGE, Some(&mut free), 7), Outcome::Dialog(Dialog { kind: 0x2b, .. })));
        assert!(matches!(on_buy_price_ok(Some(&entry)), Outcome::Dialog(Dialog { kind: 0x2b, message: 570, max: 12, .. })));
    }

    #[test]
    fn the_amount_answer_is_clamped_when_setting_up_and_bounded_by_the_wish_when_visiting_100e5fb0() {
        let entry = item(true, 12, 12, 5, 0);
        assert_eq!(on_buy_amount(psi::MODE_MANAGE, Some(&entry), 500_000_000), Outcome::Move(100_000_000));
        assert_eq!(on_buy_amount(psi::MODE_MANAGE, Some(&entry), 7), Outcome::Move(7));
        assert_eq!(on_buy_amount(psi::MODE_VISIT, Some(&entry), 5), Outcome::Move(5));
        assert_eq!(on_buy_amount(psi::MODE_VISIT, Some(&entry), 6), Outcome::Message { id: 1036, color: 0xffdc_dcdc });
        assert_eq!(on_buy_amount(psi::MODE_VISIT, Some(&entry), 0), Outcome::Nothing);
        assert_eq!(on_buy_amount(psi::MODE_VISIT, None, 3), Outcome::Nothing);
        assert_eq!(buy_reply_for(0x2a), Some(0xf000_000b));
        assert_eq!(buy_reply_for(0x2b), Some(0xf000_000c));
        assert_eq!(buy_reply_for(0x2d), Some(0xf000_000d));
        assert_eq!(buy_reply_for(0x25), None);
    }

    #[test]
    fn buy_arrows_follow_the_native_up_and_down_100e5410_100e56e0() {
        let stack = item(true, 4, 9, 6, 0);
        assert_eq!(buy_arrow_up(psi::MODE_VISIT, Some(&item(true, 0, 9, 6, 0))), Outcome::Nothing);
        assert!(matches!(buy_arrow_up(psi::MODE_VISIT, Some(&stack)), Outcome::Dialog(Dialog { message: 72, .. })));
        assert_eq!(buy_arrow_up(psi::MODE_VISIT, Some(&item(false, 1, 1, 1, 0))), Outcome::Move(1));
        assert!(matches!(buy_arrow_up(psi::MODE_MANAGE, Some(&stack)), Outcome::Dialog(Dialog { message: 571, .. })));
        assert_eq!(buy_arrow_up(psi::MODE_MANAGE, Some(&item(true, 1, 1, 0, 0))), Outcome::Move(1));
        assert_eq!(buy_arrow_down(psi::MODE_VISIT, Some(&item(false, 1, 1, 1, 0))), Outcome::Move(1));
        assert!(matches!(buy_arrow_down(psi::MODE_VISIT, Some(&stack)), Outcome::Dialog(Dialog { message: 72, .. })));
        assert_eq!(buy_arrow_down(psi::MODE_VISIT, Some(&item(true, 0, 9, 6, 0))), Outcome::Nothing);
        assert!(matches!(buy_arrow_down(psi::MODE_MANAGE, Some(&stack)), Outcome::Dialog(Dialog { kind: 0x2a, .. })));
        assert_eq!(buy_arrow_down(psi::MODE_MANAGE, None), Outcome::Nothing);
    }

    fn view(manage: bool) -> ShopView {
        ShopView { private_window: crate::shop::PrivateWindow::Buy, private_manage: manage, ..ShopView::with_sysstrings(|_| None) }
    }

    #[test]
    fn managing_a_wish_list_keeps_the_catalog_joins_stacks_and_replaces_single_items_100e4e20() {
        let mut shop = view(true);
        shop.top = vec![entry(57, 100, 100, 7, true), entry(10, 1, 1, 9, false)];
        assert!(shop.buy_move(Side::Top, 0, 30));
        assert!(shop.buy_move(Side::Top, 0, 10));
        assert_eq!(shop.top.len(), 2);
        assert_eq!((shop.bottom.len(), shop.bottom[0].quantity), (1, 40));
        shop.top[1].price = 4;
        assert!(shop.buy_move(Side::Top, 1, 3));
        assert_eq!(shop.bottom.iter().filter(|e| e.grid.item_id == 10).count(), 3);
        assert!(shop.buy_move(Side::Top, 1, 2));
        assert_eq!(shop.bottom.iter().filter(|e| e.grid.item_id == 10).count(), 2);
        assert!(shop.buy_move(Side::Bottom, 0, 15));
        assert_eq!(shop.bottom[0].quantity, 25);
        assert!(shop.buy_move(Side::Bottom, 0, 25));
        assert!(shop.bottom.iter().all(|e| e.grid.item_id != 57));
        shop.top[0].price = 1_500_000_000;
        assert!(!shop.buy_move(Side::Top, 0, 2));
    }

    #[test]
    fn the_sell_window_move_takes_the_price_and_leaves_the_store_list_100e8150() {
        let mut shop = ShopView { private_manage: true, ..ShopView::with_sysstrings(|_| None) };
        shop.top = vec![entry(57, 10, 10, 3, true), entry(10, 1, 1, 9, false)];
        shop.top[0].price = 5;
        assert!(shop.sell_move(Side::Top, 0, 4));
        shop.top[0].price = 8;
        assert!(shop.sell_move(Side::Top, 0, 6));
        assert_eq!((shop.bottom[0].quantity, shop.bottom[0].price), (10, 8));
        assert_eq!(shop.top.len(), 1);
        shop.top[0].price = 1_999_999_900;
        assert!(shop.sell_move(Side::Top, 0, 1));
        shop.top.push(entry(11, 1, 1, 9, false));
        shop.top[0].price = 2_000_000_000;
        assert!(!shop.sell_move(Side::Top, 0, 1));
        let mut visit = ShopView::with_sysstrings(|_| None);
        visit.top = vec![entry(57, 10, 10, 3, true)];
        assert!(visit.sell_move(Side::Top, 0, 4));
        assert_eq!((visit.top[0].quantity, visit.bottom[0].quantity), (6, 4));
        visit.move_all(Side::Top);
        assert!(visit.top.is_empty());
        assert_eq!(visit.bottom[0].quantity, 6);
    }

    #[test]
    fn selling_to_a_store_moves_the_amount_wanted_along_with_the_stack_100e4e20() {
        let mut shop = view(false);
        let mut stack = entry(57, 10, 10, 3, true);
        stack.wanted = 8;
        shop.top = vec![stack];
        assert!(shop.buy_move(Side::Top, 0, 4));
        assert_eq!((shop.top[0].quantity, shop.top[0].wanted), (6, 4));
        assert_eq!((shop.bottom[0].quantity, shop.bottom[0].wanted), (4, 8));
        assert!(shop.buy_move(Side::Top, 0, 2));
        assert_eq!((shop.bottom[0].quantity, shop.bottom[0].wanted), (6, 10));
        assert!(shop.buy_move(Side::Top, 0, 4));
        assert!(shop.top.is_empty());
        assert!(!shop.buy_move(Side::Top, 0, 1));
    }
}
