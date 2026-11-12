#[path = "warehouse_layout_generated.rs"]
mod warehouse_layout;

#[path = "delivery_controls_generated.rs"]
mod delivery_controls;
#[path = "delivery_layout_generated.rs"]
mod delivery_layout;
#[path = "delivery_background_generated.rs"]
mod delivery_background;

use crate::button::TextButton;
use crate::color::{Argb, GOLD, TEXT};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{BUTTON_SIZE, BUTTON_Y, CLOSE_INSET, FrameCtrl, TITLE_HEIGHT};
use crate::inventory::thousands;
use crate::item_grid::{GridEntry, GridHit, ItemGrid};
use crate::window::Rect;

pub const SHOP_SIZE: (i32, i32) = (256, 401);
pub const SHOP_DEFAULT_Y_RISE: i32 = 252;
pub const WAREHOUSE_DEFAULT_POS: (i32, i32) = (80, 156);
pub const WAREHOUSE_INITIAL_POSITION: crate::initial_position::InitialPosition = crate::initial_position::InitialPosition {
    enabled: true,
    horizontal_mode: 0,
    vertical_mode: 3,
    x: WAREHOUSE_DEFAULT_POS.0,
    y: WAREHOUSE_DEFAULT_POS.1,
};
pub const STORE_BACK: &str = "L2UI_CH3.StoreWnd.Store1_back";
pub const UP_BUTTON: &str = "L2UI_CH3.Button.UpButton";
pub const DOWN_BUTTON: &str = "L2UI_CH3.Button.DownButton";
pub const BTN_NORMAL: &str = "L2UI_CH3.Button.Btn1_Normal";
pub const BTN_DOWN: &str = "L2UI_CH3.Button.Btn1_NormalOn";
pub const BACK_RECT: Rect = Rect::new(0, 20, 256, 381);
pub const TOP_RECT: Rect = Rect::new(9, 48, 239, 139);
pub const BOTTOM_RECT: Rect = Rect::new(9, 215, 239, 104);
pub const GRID_COLS: usize = 6;
pub const TOP_ROWS: usize = 4;
pub const BOTTOM_ROWS: usize = 3;
pub const UP_RECT: Rect = Rect::new(112, 194, 15, 15);
pub const DOWN_RECT: Rect = Rect::new(130, 194, 15, 15);
pub const OK_RECT: Rect = Rect::new(51, 372, 76, 23);
pub const CANCEL_RECT: Rect = Rect::new(131, 372, 76, 23);
pub const PRIVATE_SIZE: (i32, i32) = (256, 409);
pub const PRIVATE_ABORT_RECT: Rect = Rect::new(10, 372, 76, 23);
pub const PRIVATE_MESSAGE_RECT: Rect = Rect::new(87, 372, 76, 23);
pub const PRIVATE_START_RECT: Rect = Rect::new(164, 372, 76, 23);
pub const PRIVATE_PACKAGE_RECT: Rect = Rect::new(11, 350, 100, 23);
pub const PRIVATE_BUY_COVER: &str = "NWindow.ChatBack";
pub const PRIVATE_TITLE_SYSSTRING: u32 = 498;
pub const PRIVATE_PACKAGE_SYSSTRING: u32 = 1198;
pub const PRIVATE_WISH_LIST_SYSSTRING: u32 = 502;
pub const PRIVATE_FOR_SALE_SYSSTRING: u32 = 503;
pub const LABEL_X: i32 = 11;
pub const TOP_LABEL_Y: i32 = 32;
pub const BOTTOM_LABEL_Y: i32 = 198;
pub const FOOTER_LABEL_RIGHT: i32 = 156;
pub const FOOTER_VALUE_RIGHT: i32 = 247;
pub const FOOTER_TOP_Y: i32 = 331;
pub const FOOTER_BOTTOM_Y: i32 = 350;
pub const STORE_SYSSTRING: u32 = 136;
pub const WAREHOUSE_SYSSTRING: u32 = 131;
pub const PRIVATE_WAREHOUSE_SYSSTRING: u32 = 1216;
pub const CLAN_WAREHOUSE_SYSSTRING: u32 = 1217;
pub const CASTLE_WAREHOUSE_SYSSTRING: u32 = 1218;
pub const ITEMS_ON_SALE_SYSSTRING: u32 = 137;
pub const INVENTORY_SYSSTRING: u32 = 138;
pub const ITEMS_TO_BUY_SYSSTRING: u32 = 139;
pub const STORED_ITEMS_SYSSTRING: u32 = 132;
pub const ITEMS_TO_RETRIEVE_SYSSTRING: u32 = 133;
pub const ADENA_SYSSTRING: u32 = 134;
pub const FEE_SYSSTRING: u32 = 135;
pub const PRICE_BUY_SYSSTRING: u32 = 142;
pub const PRICE_SELL_SYSSTRING: u32 = 143;
pub const OK_SYSSTRING: u32 = 140;
pub const CANCEL_SYSSTRING: u32 = 141;
pub const PRIVATE_ABORT_SYSSTRING: u32 = 0x181;
pub const PRIVATE_MESSAGE_SYSSTRING: u32 = 0x180;
pub const PRIVATE_START_SYSSTRING: u32 = 0x1ac;
pub const MOVE_COUNT_SYSMSG: u32 = 72;
pub const MANOR_SYSSTRING: u32 = 738;
pub const MANOR_CAPACITY: usize = 160;
pub const TOO_MANY_SYSMSG: u32 = 1338;
pub const WAREHOUSE_FAILED_SYSMSG: u32 = 307;
pub const WAREHOUSE_PRIVATE_TYPE: i32 = 1;
pub const WAREHOUSE_CLAN_TYPE: i32 = 2;
pub const WAREHOUSE_CASTLE_TYPE: i32 = 3;
pub const WAREHOUSE_FREIGHT_TYPE: i32 = 4;
pub const WAREHOUSE_DEFAULT_LIMIT: i32 = 200;
pub const WAREHOUSE_GRID_CAPACITY: usize = 200;
pub const DEPOSIT_FEE_PER_ITEM: i64 = 30;
pub const CONSUME_TYPE_STACKABLE: [u32; 3] = [1, 2, 3];
pub const CONSUME_TYPE_ASSET: u32 = 2;
pub const CONSUME_TYPE_ADENA: u32 = 3;
pub const DIALOG_TYPE_COUNT: i32 = 2;
pub const DIALOG_TYPE_TOO_MANY: i32 = 0x1f;
pub const CLOSE_SOUND: i32 = 6;
pub const PREVIEW_LIST_SYSSTRING: u32 = 811;
pub const PREVIEW_SELECTION_SYSSTRING: u32 = 812;
pub const PREVIEW_COST_SYSSTRING: u32 = 813;
pub const PREVIEW_TITLE_SYSSTRING: u32 = 847;
pub const PREVIEW_CONFIRM_SYSMSG: u32 = 1157;
pub const PREVIEW_DIALOG_TYPE: i32 = 0x47;
pub const PREVIEW_SOURCE_CAPACITY: usize = GRID_COLS * TOP_ROWS;
pub const PREVIEW_SELECTION_CAPACITY: usize = GRID_COLS * BOTTOM_ROWS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShopMode {
    #[default]
    Buy,
    Sell,
    Deposit,
    Withdraw,
    Deliver,
    Preview,
    ManorSeed,
    ManorProcure,
}

impl ShopMode {
    pub fn is_warehouse(self) -> bool {
        matches!(self, ShopMode::Deposit | ShopMode::Withdraw)
    }

    pub fn manor_tip_flags(self, destination: Side) -> u32 {
        match (self, destination) {
            (ShopMode::ManorSeed, Side::Top) => TIP_PRICE_BIT | 1,
            (ShopMode::ManorSeed, Side::Bottom) => TIP_PRICE_BIT,
            (ShopMode::ManorProcure, _) => TIP_PRICE_PREFIXED_BIT,
            (ShopMode::Buy | ShopMode::Preview, Side::Top) => TIP_NO_ENCHANT_BIT | TIP_NO_COUNT_BIT | TIP_PRICE_BIT,
            (ShopMode::Buy | ShopMode::Preview, Side::Bottom) => TIP_PRICE_BIT,
            (ShopMode::Sell, _) => TIP_PRICE_PREFIXED_BIT,
            _ => 0,
        }
    }

    pub fn is_manor(self) -> bool {
        matches!(self, ShopMode::ManorSeed | ShopMode::ManorProcure)
    }

    pub fn capacity(self) -> usize {
        if self.is_warehouse() { WAREHOUSE_GRID_CAPACITY } else if self == Self::Deliver { 160 } else if self.is_manor() { MANOR_CAPACITY } else { usize::MAX }
    }

    pub fn consumes_source(self) -> bool {
        self != ShopMode::Buy
    }

    pub fn labels(self) -> (u32, u32, u32) {
        match self {
            ShopMode::Buy => (ITEMS_ON_SALE_SYSSTRING, ITEMS_TO_BUY_SYSSTRING, PRICE_BUY_SYSSTRING),
            ShopMode::Sell => (INVENTORY_SYSSTRING, ITEMS_ON_SALE_SYSSTRING, PRICE_SELL_SYSSTRING),
            ShopMode::Deposit => (INVENTORY_SYSSTRING, STORED_ITEMS_SYSSTRING, FEE_SYSSTRING),
            ShopMode::Withdraw => (STORED_ITEMS_SYSSTRING, ITEMS_TO_RETRIEVE_SYSSTRING, FEE_SYSSTRING),
            ShopMode::Deliver => (INVENTORY_SYSSTRING, 560, 561),
            ShopMode::Preview => (PREVIEW_LIST_SYSSTRING, PREVIEW_SELECTION_SYSSTRING, PREVIEW_COST_SYSSTRING),
            ShopMode::ManorSeed => (ITEMS_ON_SALE_SYSSTRING, ITEMS_TO_BUY_SYSSTRING, PRICE_BUY_SYSSTRING),
            ShopMode::ManorProcure => (INVENTORY_SYSSTRING, ITEMS_ON_SALE_SYSSTRING, PRICE_SELL_SYSSTRING),
        }
    }
}

pub fn warehouse_title(warehouse_type: i32) -> u32 {
    match warehouse_type {
        WAREHOUSE_PRIVATE_TYPE => PRIVATE_WAREHOUSE_SYSSTRING,
        WAREHOUSE_CLAN_TYPE => CLAN_WAREHOUSE_SYSSTRING,
        WAREHOUSE_CASTLE_TYPE => CASTLE_WAREHOUSE_SYSSTRING,
        _ => WAREHOUSE_SYSSTRING,
    }
}

pub fn is_stackable(consume_type: u32) -> bool {
    CONSUME_TYPE_STACKABLE.contains(&consume_type)
}

pub fn count_limit(consume_type: u32) -> Option<i64> {
    match consume_type {
        CONSUME_TYPE_ADENA => Some(2_000_000_000),
        CONSUME_TYPE_ASSET => Some(1_000_000),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ShopEntry {
    pub grid: GridEntry,
    pub quantity: i64,
    pub stock: i32,
    pub price: i64,
    pub reference_price: i64,
    pub wanted: i32,
    pub consume_type: u32,
    pub warehouse_id: i32,
    pub tip_flags: u32,
}

pub const TIP_NO_ENCHANT_BIT: u32 = 1;
pub const TIP_NO_COUNT_BIT: u32 = 2;
pub const TIP_PRICE_BIT: u32 = 4;
pub const TIP_PRICE_PREFIXED_BIT: u32 = 8;
pub const TIP_VALUE_BIT: u32 = 0x20;
pub const TIP_PRICE_SYSSTRING: u32 = 322;
pub const TIP_ADENA_SYSSTRING: u32 = 469;
pub const TIP_PREFIX_SYSSTRING: u32 = 468;
pub const TIP_VALUE_SYSSTRING: u32 = 641;

pub fn price_tip_lines(flags: u32, price: i64, lookup: &dyn Fn(u32) -> Option<String>) -> Vec<crate::tooltip::TooltipLine> {
    use crate::tooltip::TooltipLine;
    let text = |id: u32| lookup(id).unwrap_or_default();
    let number = crate::inventory::thousands(i64::from(price as i32));
    let units: Vec<u16> = number.encode_utf16().collect();
    let color = crate::edit_box::numeric_tint(&units);
    let mut lines = Vec::new();
    if flags & TIP_PRICE_BIT != 0 {
        lines.push(TooltipLine { label: text(TIP_PRICE_SYSSTRING), value: format!("{number} {}", text(TIP_ADENA_SYSSTRING)), color });
    }
    if flags & TIP_PRICE_PREFIXED_BIT != 0 {
        lines.push(TooltipLine { label: text(TIP_PRICE_SYSSTRING), value: format!("{} {number} {}", text(TIP_PREFIX_SYSSTRING), text(TIP_ADENA_SYSSTRING)), color });
    }
    if flags & TIP_VALUE_BIT != 0 {
        lines.push(TooltipLine { label: text(TIP_VALUE_SYSSTRING), value: number, color });
    }
    lines
}

impl ShopEntry {
    pub fn stackable(&self) -> bool {
        is_stackable(self.consume_type)
    }

    pub(crate) fn with_quantity(&self, quantity: i64) -> ShopEntry {
        ShopEntry { quantity, ..self.clone() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopHit {
    Close,
    Slot(Side, usize),
    Scroll(Side, crate::scroll::ScrollHit),
    Up,
    Down,
    Ok,
    Cancel,
    Message,
    Package,
    Title,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShopAction {
    None,
    AskCount { side: Side, index: usize, item_name: String, max: i64, consume_type: u32 },
    Buy { list_id: i32, orders: Vec<(i32, i32)> },
    Sell { list_id: i32, orders: Vec<(i32, i32, i32)> },
    Deposit { orders: Vec<(i32, i32)> },
    Withdraw { orders: Vec<(i32, i32)> },
    Deliver { target_id: i32, orders: Vec<(i32, i32)> },
    TooMany,
    AskPreview,
    BuySeed { manor_id: i32, orders: Vec<(i32, i32)> },
    Procure { manor_id: i32, orders: Vec<(i32, i32, i32)> },
    Dismiss,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ShopStrings {
    pub store: String,
    pub manor: String,
    pub warehouse: [String; 4],
    pub items_on_sale: String,
    pub inventory: String,
    pub items_to_buy: String,
    pub stored_items: String,
    pub items_to_retrieve: String,
    pub adena: String,
    pub fee: String,
    pub price_buy: String,
    pub price_sell: String,
    pub ok: String,
    pub cancel: String,
    pub private_abort: String,
    pub private_message: String,
    pub private_start: String,
    pub private_title: String,
    pub private_package: String,
    pub private_wish_list: String,
    pub private_for_sale: String,
    pub preview_list: String,
    pub preview_selection: String,
    pub preview_cost: String,
    pub preview_title: String,
    pub delivery: [String; 3],
}

impl ShopStrings {
    pub fn with_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> ShopStrings {
        let text = |id: u32| lookup(id).unwrap_or_else(|| format!("sysstring {id}"));
        ShopStrings {
            store: text(STORE_SYSSTRING),
            manor: text(MANOR_SYSSTRING),
            warehouse: [
                text(WAREHOUSE_SYSSTRING),
                text(PRIVATE_WAREHOUSE_SYSSTRING),
                text(CLAN_WAREHOUSE_SYSSTRING),
                text(CASTLE_WAREHOUSE_SYSSTRING),
            ],
            items_on_sale: text(ITEMS_ON_SALE_SYSSTRING),
            inventory: text(INVENTORY_SYSSTRING),
            items_to_buy: text(ITEMS_TO_BUY_SYSSTRING),
            stored_items: text(STORED_ITEMS_SYSSTRING),
            items_to_retrieve: text(ITEMS_TO_RETRIEVE_SYSSTRING),
            adena: text(ADENA_SYSSTRING),
            fee: text(FEE_SYSSTRING),
            price_buy: text(PRICE_BUY_SYSSTRING),
            price_sell: text(PRICE_SELL_SYSSTRING),
            ok: text(OK_SYSSTRING),
            cancel: text(CANCEL_SYSSTRING),
            private_abort: text(PRIVATE_ABORT_SYSSTRING),
            private_message: text(PRIVATE_MESSAGE_SYSSTRING),
            private_start: text(PRIVATE_START_SYSSTRING),
            private_title: text(PRIVATE_TITLE_SYSSTRING),
            private_package: text(PRIVATE_PACKAGE_SYSSTRING),
            private_wish_list: text(PRIVATE_WISH_LIST_SYSSTRING),
            private_for_sale: text(PRIVATE_FOR_SALE_SYSSTRING),
            preview_list: text(PREVIEW_LIST_SYSSTRING),
            preview_selection: text(PREVIEW_SELECTION_SYSSTRING),
            preview_cost: text(PREVIEW_COST_SYSSTRING),
            preview_title: text(PREVIEW_TITLE_SYSSTRING),
            delivery: [557, 560, 561].map(text),
        }
    }

    pub fn of(&self, id: u32) -> &str {
        match id {
            557 => &self.delivery[0],
            560 => &self.delivery[1],
            561 => &self.delivery[2],
            STORE_SYSSTRING => &self.store,
            MANOR_SYSSTRING => &self.manor,
            WAREHOUSE_SYSSTRING => &self.warehouse[0],
            PRIVATE_WAREHOUSE_SYSSTRING => &self.warehouse[1],
            CLAN_WAREHOUSE_SYSSTRING => &self.warehouse[2],
            CASTLE_WAREHOUSE_SYSSTRING => &self.warehouse[3],
            ITEMS_ON_SALE_SYSSTRING => &self.items_on_sale,
            INVENTORY_SYSSTRING => &self.inventory,
            ITEMS_TO_BUY_SYSSTRING => &self.items_to_buy,
            STORED_ITEMS_SYSSTRING => &self.stored_items,
            ITEMS_TO_RETRIEVE_SYSSTRING => &self.items_to_retrieve,
            ADENA_SYSSTRING => &self.adena,
            FEE_SYSSTRING => &self.fee,
            PRICE_BUY_SYSSTRING => &self.price_buy,
            PRICE_SELL_SYSSTRING => &self.price_sell,
            OK_SYSSTRING => &self.ok,
            CANCEL_SYSSTRING => &self.cancel,
            PRIVATE_ABORT_SYSSTRING => &self.private_abort,
            PRIVATE_MESSAGE_SYSSTRING => &self.private_message,
            PRIVATE_START_SYSSTRING => &self.private_start,
            PRIVATE_TITLE_SYSSTRING => &self.private_title,
            PRIVATE_PACKAGE_SYSSTRING => &self.private_package,
            PRIVATE_WISH_LIST_SYSSTRING => &self.private_wish_list,
            PRIVATE_FOR_SALE_SYSSTRING => &self.private_for_sale,
            PREVIEW_LIST_SYSSTRING => &self.preview_list,
            PREVIEW_SELECTION_SYSSTRING => &self.preview_selection,
            PREVIEW_COST_SYSSTRING => &self.preview_cost,
            PREVIEW_TITLE_SYSSTRING => &self.preview_title,
            _ => "",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrivateWindow {
    #[default]
    Shop,
    Buy,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ShopView {
    pub mode: ShopMode,
    pub list_id: i32,
    pub header: i32,
    pub adena: i64,
    pub warehouse_type: i32,
    pub warehouse_limit: i32,
    pub top: Vec<ShopEntry>,
    pub bottom: Vec<ShopEntry>,
    pub top_grid: ItemGrid,
    pub bottom_grid: ItemGrid,
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub ok_pressed: bool,
    pub cancel_pressed: bool,
    pub private_window: PrivateWindow,
    pub private_manage: bool,
    pub package_sale: bool,
    pub item_limit: i32,
    pub private_sell_title: String,
    pub strings: ShopStrings,
    pub gm: bool,
}

pub fn close_button() -> Rect {
    Rect::new(SHOP_SIZE.0 - CLOSE_INSET, BUTTON_Y, BUTTON_SIZE, BUTTON_SIZE)
}

pub fn grip() -> Rect {
    Rect::new(0, 0, SHOP_SIZE.0, TITLE_HEIGHT)
}

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    crate::mainwnd::default_position(viewport)
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    crate::mainwnd::initial_position(viewport)
}

pub fn trade_default_position(viewport: (i32, i32)) -> (i32, i32) {
    crate::inventory::default_position(viewport)
}

pub fn private_default_position(viewport: (i32, i32)) -> (i32, i32) {
    crate::inventory::default_position(viewport)
}

pub fn private_initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    crate::inventory::initial_position(viewport)
}

impl ShopView {
    pub fn with_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> ShopView {
        ShopView {
            top_grid: ItemGrid::new(TOP_RECT, GRID_COLS, TOP_ROWS),
            bottom_grid: ItemGrid::new(BOTTOM_RECT, GRID_COLS, BOTTOM_ROWS),
            strings: ShopStrings::with_sysstrings(lookup),
            ..Default::default()
        }
    }

    pub fn open(&mut self, mode: ShopMode, list_id: i32, adena: i64, items: Vec<ShopEntry>) {
        self.mode = mode;
        self.list_id = list_id;
        self.adena = adena;
        self.top = items;
        for entry in &mut self.top {
            entry.tip_flags = mode.manor_tip_flags(Side::Top);
        }
        self.bottom.clear();
        let (top, bottom) = if mode == ShopMode::Deliver { (delivery_layout::RECT_E8, delivery_layout::RECT_EC) } else if mode.is_warehouse() { (warehouse_layout::RECT_EC, warehouse_layout::RECT_E8) } else { (TOP_RECT, BOTTOM_RECT) };
        for (grid, rect) in [(&mut self.top_grid, top), (&mut self.bottom_grid, bottom)] {
            grid.rect = rect;
            grid.scroll.slider.height = crate::scroll::scroll_rects(rect.w, rect.h).slider.h;
            grid.clear();
        }
        self.top_grid.empty_texture = None;
        self.bottom_grid.empty_texture = None;
        self.top_grid.show_count = mode == ShopMode::Buy;
        self.top_grid.update_rows(self.top.len());
        self.bottom_grid.update_rows(0);
    }

    pub fn open_preview(&mut self, header: i32, list_id: i32, adena: i64, items: Vec<ShopEntry>) {
        let items = items.into_iter().take(PREVIEW_SOURCE_CAPACITY).collect();
        self.open(ShopMode::Preview, list_id, adena, items);
        self.header = header;
    }

    pub fn use_private_buy_grids(&mut self) {
        self.top_grid.show_count = false;
        self.bottom_grid.show_count = false;
    }

    pub fn clear(&mut self) {
        self.list_id = -1;
        self.adena = 0;
        self.top.clear();
        self.bottom.clear();
        self.top_grid.clear();
        self.bottom_grid.clear();
    }

    pub fn title(&self) -> &str {
        if self.gm {
            ""
        } else if self.mode == ShopMode::Preview {
            self.strings.of(PREVIEW_TITLE_SYSSTRING)
        } else if self.mode.is_manor() {
            self.strings.of(MANOR_SYSSTRING)
        } else if self.mode == ShopMode::Deliver {
            self.strings.of(557)
        } else if self.mode.is_warehouse() {
            self.strings.of(warehouse_title(self.warehouse_type))
        } else {
            self.strings.of(STORE_SYSSTRING)
        }
    }

    pub fn total(&self) -> i64 {
        self.bottom.iter().map(|e| e.price * e.quantity).sum()
    }

    pub fn fee(&self) -> i64 {
        match self.mode {
            ShopMode::Deposit => self.bottom.len() as i64 * DEPOSIT_FEE_PER_ITEM,
            ShopMode::Deliver => i64::from((self.bottom.len() as i32).wrapping_mul(1000)),
            _ => 0,
        }
    }

    pub fn entries_mut(&mut self, side: Side) -> &mut Vec<ShopEntry> {
        match side {
            Side::Top => &mut self.top,
            Side::Bottom => &mut self.bottom,
        }
    }

    pub fn entries(&self, side: Side) -> &[ShopEntry] {
        match side {
            Side::Top => &self.top,
            Side::Bottom => &self.bottom,
        }
    }

    fn grid_entries(&self, side: Side) -> Vec<GridEntry> {
        self.entries(side)
            .iter()
            .map(|e| GridEntry { count: if side == Side::Top { e.stock } else { e.quantity.min(i32::MAX as i64) as i32 }, ..e.grid.clone() })
            .collect()
    }

    pub fn counter(&self) -> Option<(String, i32)> {
        if !self.mode.is_warehouse() || self.warehouse_type == WAREHOUSE_FREIGHT_TYPE {
            return None;
        }
        let limit = if self.warehouse_type == WAREHOUSE_PRIVATE_TYPE { self.warehouse_limit } else { WAREHOUSE_DEFAULT_LIMIT };
        let (stored, y) = match self.mode {
            ShopMode::Deposit => (self.bottom.len(), BOTTOM_LABEL_Y),
            _ => (self.top.len(), TOP_LABEL_Y),
        };
        Some((format!("({stored}/{limit})"), y))
    }

    pub fn selected(&self, side: Side) -> Option<usize> {
        let grid = match side {
            Side::Top => &self.top_grid,
            Side::Bottom => &self.bottom_grid,
        };
        grid.selected.filter(|i| *i < self.entries(side).len())
    }

    pub fn select(&mut self, side: Side, index: usize) {
        self.grid_of(side).selected = Some(index);
    }

    pub(crate) fn grid_of(&mut self, side: Side) -> &mut ItemGrid {
        match side {
            Side::Top => &mut self.top_grid,
            Side::Bottom => &mut self.bottom_grid,
        }
    }

    fn unavailable(&self, side: Side, entry: &ShopEntry) -> bool {
        side == Side::Top && self.mode == ShopMode::Sell && entry.stock < 1
    }

    pub fn press_move(&mut self, side: Side) -> ShopAction {
        let Some(index) = self.selected(side) else { return ShopAction::None };
        let entry = &self.entries(side)[index];
        if self.unavailable(side, entry) {
            return ShopAction::None;
        }
        let ask = match (self.mode, side) {
            (ShopMode::Buy | ShopMode::Preview | ShopMode::ManorSeed | ShopMode::ManorProcure, Side::Top) => entry.stackable(),
            _ => entry.stackable() && entry.quantity > 1,
        };
        if ask {
            let max = if (self.mode == ShopMode::Buy && side == Side::Top) || self.mode.is_manor() { i64::from(entry.stock) } else { entry.quantity };
            return ShopAction::AskCount { side, index, item_name: entry.grid.name.clone(), max, consume_type: entry.consume_type };
        }
        self.move_entry(side, index, 1);
        ShopAction::None
    }

    pub fn press_double(&mut self, side: Side) -> ShopAction {
        let Some(index) = self.selected(side) else { return ShopAction::None };
        let entry = &self.entries(side)[index];
        let ask = match self.mode {
            ShopMode::ManorSeed => entry.stackable(),
            ShopMode::ManorProcure => entry.stackable() && entry.quantity > 1,
            ShopMode::Buy | ShopMode::Preview => {
                if entry.stackable() {
                    return ShopAction::AskCount { side, index, item_name: entry.grid.name.clone(), max: entry.quantity, consume_type: entry.consume_type };
                }
                self.move_entry(side, index, 1);
                return ShopAction::None;
            }
            _ => return self.press_move(side),
        };
        if ask {
            return ShopAction::AskCount { side, index, item_name: entry.grid.name.clone(), max: i64::from(entry.stock), consume_type: entry.consume_type };
        }
        self.move_entry(side, index, 1);
        ShopAction::None
    }

    pub fn press_drop(&mut self, from: Side) -> ShopAction {
        if self.mode != ShopMode::ManorProcure {
            return self.press_move(from);
        }
        let Some(index) = self.selected(from) else { return ShopAction::None };
        let entry = &self.entries(from)[index];
        if entry.stackable() && entry.quantity > 1 {
            return ShopAction::AskCount { side: from, index, item_name: entry.grid.name.clone(), max: i64::from(entry.stock), consume_type: entry.consume_type };
        }
        self.move_entry(from, index, 1);
        ShopAction::None
    }

    pub fn move_entry(&mut self, side: Side, index: usize, count: i64) {
        if count <= 0 || index >= self.entries(side).len() {
            return;
        }
        self.grid_of(side).selected = None;
        let entry = self.entries(side)[index].clone();
        if self.mode == ShopMode::Preview {
            self.move_preview_entry(side, index, count);
            return;
        }
        let count = if self.mode.consumes_source() { count.min(entry.quantity) } else { count };
        let dest_gains = !matches!((self.mode, side), (ShopMode::Buy, Side::Bottom));
        if dest_gains {
            let dest = match side {
                Side::Top => &mut self.bottom,
                Side::Bottom => &mut self.top,
            };
            let full = dest.len() >= self.mode.capacity();
            let destination = if side == Side::Top { Side::Bottom } else { Side::Top };
            match dest.iter_mut().find(|d| entry.stackable() && d.grid.item_id == entry.grid.item_id) {
                Some(existing) => {
                    existing.quantity += count;
                    if self.mode.is_manor() {
                        existing.tip_flags = 0;
                    }
                }
                None if full => {}
                None => {
                    let mut moved = entry.with_quantity(count);
                    moved.tip_flags = self.mode.manor_tip_flags(destination);
                    dest.push(moved);
                }
            }
        }
        let source_loses = !matches!((self.mode, side), (ShopMode::Buy, Side::Top));
        if source_loses {
            let source = match side {
                Side::Top => &mut self.top,
                Side::Bottom => &mut self.bottom,
            };
            if entry.stackable() && entry.quantity > count {
                source[index].quantity -= count;
                if self.mode.is_manor() {
                    source[index].tip_flags = 0;
                }
            } else {
                source.remove(index);
            }
        }
        self.top_grid.update_rows(self.top.len());
        self.bottom_grid.update_rows(self.bottom.len());
    }

    fn move_preview_entry(&mut self, side: Side, index: usize, count: i64) {
        let entry = self.entries(side)[index].clone();
        let (dest, capacity) = match side {
            Side::Top => (&mut self.bottom, PREVIEW_SELECTION_CAPACITY),
            Side::Bottom => (&mut self.top, PREVIEW_SOURCE_CAPACITY),
        };
        let merge = entry.stackable().then(|| dest.iter().position(|d| d.grid.item_id == entry.grid.item_id)).flatten();
        let flags = ShopMode::Preview.manor_tip_flags(if side == Side::Top { Side::Bottom } else { Side::Top });
        match merge {
            Some(at) => dest[at].quantity += count,
            None if dest.len() < capacity => dest.push(ShopEntry { tip_flags: flags, ..entry.with_quantity(count) }),
            None => {}
        }
        let source = match side {
            Side::Top => &mut self.top,
            Side::Bottom => &mut self.bottom,
        };
        if entry.stackable() && count < entry.quantity {
            source[index].quantity -= count;
        } else {
            source.remove(index);
        }
        self.top_grid.update_rows(self.top.len());
        self.bottom_grid.update_rows(self.bottom.len());
    }

    pub fn preview_request(&self) -> Option<(i32, i32, Vec<i32>)> {
        let ids: Vec<i32> = self.bottom.iter().filter(|e| e.quantity > 0).map(|e| e.grid.item_id).collect();
        (!ids.is_empty()).then_some((self.header, self.list_id, ids))
    }

    pub fn confirm(&self) -> ShopAction {
        let chosen = |e: &ShopEntry| e.quantity > 0;
        match self.mode {
            ShopMode::Preview => if self.bottom.is_empty() { ShopAction::None } else { ShopAction::AskPreview },
            ShopMode::Buy => {
                for stocked in self.top.iter().filter(|e| e.stock > 0) {
                    let wanted: i64 = self
                        .bottom
                        .iter()
                        .filter(|b| b.grid.item_id == stocked.grid.item_id)
                        .map(|b| if b.stackable() { b.quantity } else { 1 })
                        .sum();
                    if wanted > i64::from(stocked.stock) {
                        return ShopAction::TooMany;
                    }
                }
                let orders = self.bottom.iter().filter(|e| chosen(e)).map(|e| (e.grid.item_id, e.quantity as i32)).collect();
                ShopAction::Buy { list_id: self.list_id, orders }
            }
            ShopMode::Sell => {
                let orders = self.bottom.iter().filter(|e| chosen(e)).map(|e| (e.grid.object_id, e.grid.item_id, e.quantity as i32)).collect();
                ShopAction::Sell { list_id: self.list_id, orders }
            }
            ShopMode::Deposit => {
                ShopAction::Deposit { orders: self.bottom.iter().filter(|e| chosen(e)).map(|e| (e.warehouse_id, e.quantity as i32)).collect() }
            }
            ShopMode::Deliver => ShopAction::Deliver { target_id: self.list_id, orders: self.bottom.iter().filter(|e| chosen(e)).map(|e| (e.warehouse_id, e.quantity as i32)).collect() },
            ShopMode::Withdraw => {
                ShopAction::Withdraw { orders: self.bottom.iter().filter(|e| chosen(e)).map(|e| (e.warehouse_id, e.quantity as i32)).collect() }
            }
            ShopMode::ManorSeed => {
                let orders: Vec<_> = self.bottom.iter().filter(|e| chosen(e)).map(|e| (e.grid.item_id, e.quantity as i32)).collect();
                if orders.is_empty() { ShopAction::Dismiss } else { ShopAction::BuySeed { manor_id: self.list_id, orders } }
            }
            ShopMode::ManorProcure => ShopAction::Procure {
                manor_id: self.list_id,
                orders: self.bottom.iter().filter(|e| chosen(e)).map(|e| (e.grid.object_id, e.grid.item_id, e.quantity as i32)).collect(),
            },
        }
    }

    fn delivery_frame(&self) -> FrameCtrl { delivery_controls::FRAME_7C.build(|id| self.strings.of(id).to_string()) }

    pub fn hit(&self, x: i32, y: i32) -> Option<ShopHit> {
        let size = if self.mode == ShopMode::Deliver { delivery_layout::SIZE } else if self.mode.is_warehouse() { warehouse_layout::SIZE } else { SHOP_SIZE };
        if !Rect::new(0, 0, size.0, size.1).contains(x, y) {
            return None;
        }
        let close = if self.mode == ShopMode::Deliver {
            delivery_controls::FRAME_7C.hit(&self.delivery_frame(), (0, 0), (x, y)) == Some(crate::frame_contract::FrameHit::Close)
        } else { close_button().contains(x, y) };
        if close {
            return Some(ShopHit::Close);
        }
        if y < TITLE_HEIGHT {
            return Some(ShopHit::Title);
        }
        if !self.gm && UP_RECT.contains(x, y) {
            return Some(ShopHit::Up);
        }
        if !self.gm && DOWN_RECT.contains(x, y) {
            return Some(ShopHit::Down);
        }
        let ok = if self.mode == ShopMode::Deliver {
            delivery_controls::BUTTON_FC.hit(&delivery_controls::BUTTON_FC.build((0, 0), |_| String::new()), x, y).is_some()
        } else { OK_RECT.contains(x, y) };
        if ok && !self.gm {
            return Some(ShopHit::Ok);
        }
        let cancel = if self.mode == ShopMode::Deliver {
            delivery_controls::BUTTON_100.hit(&delivery_controls::BUTTON_100.build((0, 0), |_| String::new()), x, y).is_some()
        } else { CANCEL_RECT.contains(x, y) };
        if cancel && !self.gm {
            return Some(ShopHit::Cancel);
        }
        for (side, grid) in [(Side::Top, &self.top_grid), (Side::Bottom, &self.bottom_grid)] {
            match grid.hit(x, y) {
                Some(GridHit::Slot(i)) => return Some(ShopHit::Slot(side, i)),
                Some(GridHit::Scroll(hit)) => return Some(ShopHit::Scroll(side, hit)),
                None => {}
            }
        }
        Some(ShopHit::Body)
    }

    pub fn private_hit(&self, x: i32, y: i32) -> Option<ShopHit> {
        if !Rect::new(0, 0, PRIVATE_SIZE.0, PRIVATE_SIZE.1).contains(x, y) {
            return None;
        }
        if close_button().contains(x, y) {
            return Some(ShopHit::Close);
        }
        if y < TITLE_HEIGHT {
            return Some(ShopHit::Title);
        }
        if UP_RECT.contains(x, y) {
            return Some(ShopHit::Up);
        }
        if DOWN_RECT.contains(x, y) {
            return Some(ShopHit::Down);
        }
        if self.private_manage {
            if PRIVATE_ABORT_RECT.contains(x, y) {
                return Some(ShopHit::Cancel);
            }
            if PRIVATE_MESSAGE_RECT.contains(x, y) {
                return Some(ShopHit::Message);
            }
            if self.private_window == PrivateWindow::Shop && PRIVATE_PACKAGE_RECT.contains(x, y) {
                return Some(ShopHit::Package);
            }
        }
        if PRIVATE_START_RECT.contains(x, y) {
            return Some(ShopHit::Ok);
        }
        for (side, grid) in [(Side::Top, &self.top_grid), (Side::Bottom, &self.bottom_grid)] {
            match grid.hit(x, y) {
                Some(GridHit::Slot(i)) => return Some(ShopHit::Slot(side, i)),
                Some(GridHit::Scroll(hit)) => return Some(ShopHit::Scroll(side, hit)),
                None => {}
            }
        }
        Some(ShopHit::Body)
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, _alpha: f32) {
        let alpha = 1.0;
        let (w, h) = if self.mode.is_warehouse() { warehouse_layout::SIZE } else { SHOP_SIZE };
        if self.mode == ShopMode::Deliver {
            delivery_controls::FRAME_7C.paint(&self.delivery_frame(), list, (x, y), font, alpha);
            let _ = delivery_background::BACKGROUND.paint(list, (x, y), alpha);
        } else {
            FrameCtrl::titled(self.title(), true).paint(list, x, y, w, h, font, alpha);
            let back = BACK_RECT;
            list.draw_texture(x + back.x, y + back.y, back.w, back.h, 0, 0, back.w, back.h, STORE_BACK, alpha);
        }
        let (top, bottom, price) = self.mode.labels();
        list.draw_text(x + LABEL_X, y + TOP_LABEL_Y, TEXT, self.strings.of(top), font);
        list.draw_text(x + LABEL_X, y + BOTTOM_LABEL_Y, TEXT, self.strings.of(bottom), font);
        if let Some((counter, cy)) = self.counter() {
            let cw = font.measure(&counter) as i32;
            list.draw_text(x + w - 10 - cw, y + cy, GOLD, &counter, font);
        }
        let right = |list: &mut DrawList, right_x: i32, ly: i32, text: &str| {
            let tw = font.measure(text) as i32;
            list.draw_text(x + right_x - tw, y + ly, TEXT, text, font);
        };
        right(list, FOOTER_LABEL_RIGHT, FOOTER_TOP_Y, self.strings.of(price));
        right(list, FOOTER_LABEL_RIGHT, FOOTER_BOTTOM_Y, &self.strings.adena);
        let low32 = |value: i64| if matches!(self.mode, ShopMode::Preview | ShopMode::Deliver | ShopMode::ManorSeed | ShopMode::ManorProcure) { i64::from(value as i32) } else { value };
        let top_value = if self.mode.is_warehouse() || self.mode == ShopMode::Deliver { self.fee() } else { self.total() };
        right(list, FOOTER_VALUE_RIGHT, FOOTER_TOP_Y, &thousands(low32(top_value)));
        right(list, FOOTER_VALUE_RIGHT, FOOTER_BOTTOM_Y, &thousands(low32(self.adena)));
        self.top_grid.paint(list, x, y, &self.grid_entries(Side::Top), font);
        self.bottom_grid.paint(list, x, y, &self.grid_entries(Side::Bottom), font);
        for (rect, texture) in [(UP_RECT, UP_BUTTON), (DOWN_RECT, DOWN_BUTTON)] {
            list.draw_button_texture(x + rect.x, y + rect.y, rect.w, rect.h, 0, 0, rect.w, rect.h, texture, 1.0, !self.gm);
        }
        if self.mode == ShopMode::Deliver {
            let text = |id| self.strings.of(id).to_string();
            delivery_controls::BUTTON_FC.paint(&delivery_controls::BUTTON_FC.build((0, 0), text), list, (x, y), self.ok_pressed, font);
            delivery_controls::BUTTON_100.paint(&delivery_controls::BUTTON_100.build((0, 0), text), list, (x, y), self.cancel_pressed, font);
        } else {
            let mut ok = TextButton::new(OK_RECT, BTN_NORMAL, BTN_DOWN, &self.strings.ok);
            let mut cancel = TextButton::new(CANCEL_RECT, BTN_NORMAL, BTN_DOWN, &self.strings.cancel);
            ok.enabled = !self.gm;
            cancel.enabled = !self.gm;
            ok.paint(list, x, y, self.ok_pressed, font);
            cancel.paint(list, x, y, self.cancel_pressed, font);
        }
    }


    pub fn set_flag_title(&mut self) {
        self.private_sell_title = if self.package_sale {
            format!("{} - {}", self.strings.private_title, self.strings.private_package)
        } else {
            self.strings.private_title.clone()
        };
    }

    pub fn private_title(&self) -> &str {
        match self.private_window {
            PrivateWindow::Shop => &self.private_sell_title,
            PrivateWindow::Buy => "",
        }
    }

    pub fn private_labels(&self) -> (u32, u32, u32) {
        match (self.private_window, self.private_manage) {
            (PrivateWindow::Shop, false) => (ITEMS_ON_SALE_SYSSTRING, ITEMS_TO_BUY_SYSSTRING, PRICE_BUY_SYSSTRING),
            (PrivateWindow::Shop, true) => (INVENTORY_SYSSTRING, ITEMS_ON_SALE_SYSSTRING, PRICE_SELL_SYSSTRING),
            (PrivateWindow::Buy, false) => (PRIVATE_FOR_SALE_SYSSTRING, ITEMS_ON_SALE_SYSSTRING, PRICE_SELL_SYSSTRING),
            (PrivateWindow::Buy, true) => (INVENTORY_SYSSTRING, PRIVATE_WISH_LIST_SYSSTRING, PRICE_BUY_SYSSTRING),
        }
    }

    pub fn private_paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, _alpha: f32) {
        let alpha = 1.0;
        let (w, h) = PRIVATE_SIZE;
        FrameCtrl::titled(self.private_title(), true).paint(list, x, y, w, h, font, alpha);
        list.draw_texture(x, y + BACK_RECT.y, BACK_RECT.w, BACK_RECT.h, 0, 0, BACK_RECT.w, BACK_RECT.h, STORE_BACK, alpha);
        let (top, bottom, price) = self.private_labels();
        list.draw_text(x + LABEL_X, y + TOP_LABEL_Y, TEXT, self.strings.of(top), font);
        list.draw_text(x + LABEL_X, y + BOTTOM_LABEL_Y, TEXT, self.strings.of(bottom), font);
        let right = |list: &mut DrawList, right_x: i32, ly: i32, color: Argb, text: &str| {
            let tw = font.measure(text) as i32;
            list.draw_text(x + right_x - tw, y + ly, color, text, font);
        };
        right(list, FOOTER_LABEL_RIGHT, FOOTER_TOP_Y, TEXT, self.strings.of(price));
        right(list, FOOTER_LABEL_RIGHT, FOOTER_BOTTOM_Y, TEXT, &self.strings.adena);
        if self.private_manage {
            right(list, w - 5, BOTTOM_LABEL_Y, GOLD, &format!("({}/{})", self.bottom.len(), self.item_limit));
        }
        right(list, FOOTER_VALUE_RIGHT, FOOTER_TOP_Y, TEXT, &thousands(self.total()));
        right(list, FOOTER_VALUE_RIGHT, FOOTER_BOTTOM_Y, TEXT, &thousands(self.adena));
        self.top_grid.paint(list, x, y, &self.grid_entries(Side::Top), font);
        self.bottom_grid.paint(list, x, y, &self.grid_entries(Side::Bottom), font);
        if self.private_window == PrivateWindow::Buy && !self.private_manage {
            for (grid, entries) in [(&self.top_grid, &self.top), (&self.bottom_grid, &self.bottom)] {
                grid.paint_cover(list, x, y, entries.len(), |index| entries[index].stock == 0, PRIVATE_BUY_COVER);
            }
        }
        for (rect, texture) in [(UP_RECT, UP_BUTTON), (DOWN_RECT, DOWN_BUTTON)] {
            list.draw_texture(x + rect.x, y + rect.y, rect.w, rect.h, 0, 0, rect.w, rect.h, texture, alpha);
        }
        let start = if self.private_manage { &self.strings.private_start } else { &self.strings.ok };
        TextButton::new(PRIVATE_START_RECT, BTN_NORMAL, BTN_DOWN, start).paint(list, x, y, self.ok_pressed, font);
        if self.private_manage {
            TextButton::new(PRIVATE_MESSAGE_RECT, BTN_NORMAL, BTN_DOWN, &self.strings.private_message).paint(list, x, y, false, font);
            TextButton::new(PRIVATE_ABORT_RECT, BTN_NORMAL, BTN_DOWN, &self.strings.private_abort).paint(list, x, y, self.cancel_pressed, font);
            if self.private_window == PrivateWindow::Shop {
                let mut check = crate::button::CheckBox::new(PRIVATE_PACKAGE_RECT, &self.strings.private_package);
                check.checked = self.package_sale;
                check.paint(list, x, y, font);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};
    use crate::scroll::ScrollHit;

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn entry(object_id: i32, item_id: i32, quantity: i64, stock: i32, price: i64, consume_type: u32, icon: &str) -> ShopEntry {
        ShopEntry {
            reference_price: 0,
            wanted: -1,
            grid: GridEntry { object_id, item_id, count: stock, icon: icon.into(), name: format!("item{item_id}"), ..Default::default() },
            quantity,
            stock,
            price,
            consume_type,
            warehouse_id: object_id + 1000,
            tip_flags: 0,
        }
    }

    fn buy_view() -> ShopView {
        let mut view = ShopView::with_sysstrings(|id| Some(format!("s{id}")));
        view.open(ShopMode::Buy, 7, 152_340, vec![entry(0, 1, 0, 0, 1_200, 0, "a"), entry(0, 2, 5, 5, 30, 1, "b")]);
        view
    }

    #[test]
    fn delivery_fee_empty_confirmation_and_capacity_follow_c4() {
        let mut view = ShopView::with_sysstrings(|id| Some(id.to_string()));
        view.open(ShopMode::Deliver, 77, 9000, vec![]);
        assert_eq!(view.title(), "557");
        assert_eq!(view.mode.labels(), (138, 560, 561));
        assert_eq!(view.confirm(), ShopAction::Deliver { target_id: 77, orders: vec![] });
        assert_eq!(view.counter(), None);
        let item = ShopEntry { grid: GridEntry { item_id: 9, object_id: 90, ..Default::default() }, warehouse_id: 900, quantity: 50, consume_type: 1, ..Default::default() };
        view.top.push(item.clone());
        view.move_entry(Side::Top, 0, 7);
        assert_eq!(view.fee(), 1000);
        view.move_entry(Side::Bottom, 0, 7);
        assert_eq!(view.fee(), 0);
        assert_eq!(view.top[0].quantity, 50);
        view.bottom = (0..160).map(|id| ShopEntry { grid: GridEntry { item_id: 1000 + id, ..Default::default() }, ..item.clone() }).collect();
        view.move_entry(Side::Top, 0, 7);
        assert_eq!(view.bottom.len(), 160);
        assert_eq!(view.top[0].quantity, 43);
        assert_eq!(view.fee(), 160000);
    }

    #[test]
    fn the_modes_pick_the_labels_and_titles_of_the_original_paint() {
        assert_eq!(ShopMode::Buy.labels(), (137, 139, 142));
        assert_eq!(ShopMode::Sell.labels(), (138, 137, 143));
        assert_eq!(ShopMode::Deposit.labels(), (138, 132, 135));
        assert_eq!(ShopMode::Withdraw.labels(), (132, 133, 135));
        assert_eq!([warehouse_title(1), warehouse_title(2), warehouse_title(3), warehouse_title(4)], [1216, 1217, 1218, 131]);
        assert!(is_stackable(1) && is_stackable(3) && !is_stackable(0));
        assert_eq!((count_limit(3), count_limit(2), count_limit(1)), (Some(2_000_000_000), Some(1_000_000), None));
    }

    #[test]
    fn shop_paint_keeps_the_original_opaque_alpha() {
        let view = buy_view();
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font(), 0.4);
        assert!(list.quads.iter().filter(|quad| quad.texture == STORE_BACK).all(|quad| quad.color[3] == 1.0));
        let mut list = DrawList::default();
        view.private_paint(&mut list, 0, 0, &font(), 0.4);
        assert!(list.quads.iter().filter(|quad| quad.texture == STORE_BACK).all(|quad| quad.color[3] == 1.0));
    }

    #[test]
    fn buying_copies_into_the_lower_grid_without_touching_the_npc_list() {
        let mut view = buy_view();
        view.select(Side::Top, 0);
        assert_eq!(view.press_move(Side::Top), ShopAction::None);
        view.select(Side::Top, 0);
        view.press_move(Side::Top);
        assert_eq!(view.top.len(), 2);
        assert_eq!(view.bottom.len(), 2);
        view.select(Side::Top, 1);
        assert_eq!(
            view.press_move(Side::Top),
            ShopAction::AskCount { side: Side::Top, index: 1, item_name: "item2".into(), max: 5, consume_type: 1 }
        );
        view.move_entry(Side::Top, 1, 3);
        view.move_entry(Side::Top, 1, 2);
        assert_eq!(view.bottom.len(), 3);
        assert_eq!(view.bottom[2].quantity, 5);
        assert_eq!(view.total(), 2 * 1_200 + 5 * 30);
        view.select(Side::Bottom, 2);
        assert_eq!(
            view.press_move(Side::Bottom),
            ShopAction::AskCount { side: Side::Bottom, index: 2, item_name: "item2".into(), max: 5, consume_type: 1 }
        );
        view.move_entry(Side::Bottom, 2, 2);
        assert_eq!(view.bottom[2].quantity, 3);
        view.select(Side::Bottom, 0);
        view.press_move(Side::Bottom);
        assert_eq!(view.bottom.len(), 2);
        assert_eq!(view.top.len(), 2);
        assert_eq!(view.confirm(), ShopAction::Buy { list_id: 7, orders: vec![(1, 1), (2, 3)] });
        view.move_entry(Side::Top, 1, 3);
        assert_eq!(view.confirm(), ShopAction::TooMany);
    }

    #[test]
    fn selling_and_the_warehouse_move_counts_between_the_grids() {
        let mut view = ShopView::with_sysstrings(|id| Some(format!("s{id}")));
        view.open(ShopMode::Sell, 3, 100, vec![entry(10, 5, 1, 1, 50, 0, "s"), entry(11, 6, 20, 20, 2, 1, "t")]);
        view.select(Side::Top, 1);
        assert!(matches!(view.press_move(Side::Top), ShopAction::AskCount { max: 20, .. }));
        view.move_entry(Side::Top, 1, 15);
        assert_eq!((view.top[1].quantity, view.bottom[0].quantity), (5, 15));
        view.select(Side::Top, 0);
        view.press_move(Side::Top);
        assert_eq!(view.top.len(), 1);
        assert_eq!(view.bottom.len(), 2);
        view.move_entry(Side::Top, 0, 5);
        assert_eq!(view.top.len(), 0);
        assert_eq!(view.bottom[0].quantity, 20);
        assert_eq!(view.confirm(), ShopAction::Sell { list_id: 3, orders: vec![(11, 6, 20), (10, 5, 1)] });
        view.select(Side::Bottom, 0);
        assert!(matches!(view.press_move(Side::Bottom), ShopAction::AskCount { max: 20, .. }));
        view.move_entry(Side::Bottom, 0, 20);
        assert_eq!((view.top.len(), view.bottom.len()), (1, 1));
        view.open(ShopMode::Deposit, 0, 9, vec![entry(10, 5, 1, 1, 0, 0, "s"), entry(11, 6, 3, 3, 0, 1, "t")]);
        view.warehouse_type = WAREHOUSE_PRIVATE_TYPE;
        view.warehouse_limit = 80;
        view.select(Side::Top, 0);
        view.press_move(Side::Top);
        assert_eq!((view.fee(), view.counter()), (30, Some(("(1/80)".into(), BOTTOM_LABEL_Y))));
        assert_eq!(view.confirm(), ShopAction::Deposit { orders: vec![(1010, 1)] });
        view.mode = ShopMode::Withdraw;
        view.warehouse_type = WAREHOUSE_CLAN_TYPE;
        assert_eq!((view.fee(), view.counter()), (0, Some(("(1/200)".into(), TOP_LABEL_Y))));
        assert_eq!(view.title(), "s1217");
        view.warehouse_type = WAREHOUSE_FREIGHT_TYPE;
        assert_eq!((view.counter(), view.title()), (None, "s131"));
        assert_eq!(view.confirm(), ShopAction::Withdraw { orders: vec![(1010, 1)] });
    }

    #[test]
    fn the_hit_test_follows_the_oncreate_rectangles() {
        let view = buy_view();
        assert_eq!(view.hit(240, 10), Some(ShopHit::Close));
        assert_eq!(view.hit(100, 10), Some(ShopHit::Title));
        assert_eq!(view.hit(10, 49), Some(ShopHit::Slot(Side::Top, 0)));
        assert_eq!(view.hit(10 + 37, 216), Some(ShopHit::Slot(Side::Bottom, 1)));
        assert_eq!(view.hit(115, 200), Some(ShopHit::Up));
        assert_eq!(view.hit(135, 200), Some(ShopHit::Down));
        assert_eq!(view.hit(60, 380), Some(ShopHit::Ok));
        assert_eq!(view.hit(140, 380), Some(ShopHit::Cancel));
        assert_eq!(view.hit(9 + 230, 50), Some(ShopHit::Scroll(Side::Top, ScrollHit::Up)));
        assert_eq!(view.hit(128, 340), Some(ShopHit::Body));
        assert_eq!(view.hit(300, 10), None);
    }

    #[test]
    fn the_paint_puts_labels_prices_and_grids_where_the_original_does() {
        let font = font();
        let mut view = buy_view();
        view.move_entry(Side::Top, 1, 4);
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 20, &font, 1.0);
        let back = list.quads.iter().find(|q| q.texture == STORE_BACK).unwrap();
        assert_eq!((back.x, back.y, back.w, back.h), (0.0, 40.0, 256.0, 381.0));
        let text: Vec<_> = list.quads.iter().filter(|q| q.texture == "f").collect();
        let at = |x: f32, y: f32| text.iter().any(|q| q.x == x && q.y == y);
        assert!(at(11.0, 52.0), "s137 em (11, 32)");
        assert!(at(11.0, 218.0), "s139 em (11, 198)");
        let total = thousands(4 * 30);
        let tw = font.measure(&total) as i32;
        assert!(at((247 - tw) as f32, 351.0), "total à direita de 247 em y 331");
        let adena = thousands(152_340);
        let aw = font.measure(&adena) as i32;
        assert!(at((247 - aw) as f32, 370.0), "adena à direita de 247 em y 350");
        let label_w = font.measure("s142") as i32;
        assert!(at((156 - label_w) as f32, 351.0), "Price à direita de 156");
        let b = list.quads.iter().find(|q| q.texture == "b").unwrap();
        assert_eq!((b.x, b.y), (9.0 + 1.0 + 37.0, 20.0 + 48.0 + 1.0));
        let chosen = list.quads.iter().filter(|q| q.texture == "b").nth(1).unwrap();
        assert_eq!((chosen.x, chosen.y), (10.0, 20.0 + 215.0 + 1.0));
        assert!(list.quads.iter().any(|q| q.texture == UP_BUTTON && q.x == 112.0 && q.y == 214.0));
        assert!(list.quads.iter().any(|q| q.texture == BTN_NORMAL && q.x == 131.0 && q.y == 392.0));
        let red = list.quads.iter().filter(|q| q.color == crate::item_grid::COUNT_COLOR.rgba()).count();
        assert_eq!(red, 1, "só a grade de cima da compra mostra o estoque (5)");
        view.open(ShopMode::Sell, 1, 0, vec![entry(10, 5, 9, 9, 1, 1, "s")]);
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font, 1.0);
        assert!(list.quads.iter().all(|q| q.color != crate::item_grid::COUNT_COLOR.rgba()));
    }

    fn private_view(window: PrivateWindow, manage: bool) -> ShopView {
        let mut view = ShopView::with_sysstrings(|id| Some(format!("s{id}")));
        view.private_window = window;
        view.private_manage = manage;
        view
    }

    fn private_texts(view: &ShopView) -> Vec<String> {
        let font = font();
        let mut list = DrawList { capture_text: true, ..Default::default() };
        view.private_paint(&mut list, 0, 0, &font, 1.0);
        list.text_runs.iter().map(|run| run.text.clone()).collect()
    }

    #[test]
    fn private_hits_follow_the_buttons_each_mode_shows_100e97e0_100e6bf0() {
        let manage = private_view(PrivateWindow::Shop, true);
        assert_eq!(manage.private_hit(100, 380), Some(ShopHit::Message));
        assert_eq!(manage.private_hit(20, 380), Some(ShopHit::Cancel));
        assert_eq!(manage.private_hit(200, 380), Some(ShopHit::Ok));
        assert_eq!(manage.private_hit(20, 355), Some(ShopHit::Package));
        let buy = private_view(PrivateWindow::Buy, true);
        assert_eq!(buy.private_hit(20, 355), Some(ShopHit::Body), "a janela de compra não tem a caixa de venda em pacote");
        let visit = private_view(PrivateWindow::Shop, false);
        assert_eq!(visit.private_hit(100, 380), Some(ShopHit::Body));
        assert_eq!(visit.private_hit(20, 380), Some(ShopHit::Body));
        assert_eq!(visit.private_hit(20, 355), Some(ShopHit::Body));
        assert_eq!(visit.private_hit(200, 380), Some(ShopHit::Ok));
    }

    #[test]
    fn private_paint_texts_per_window_and_mode_100e6d20_100e3b00() {
        let mut shop_manage = private_view(PrivateWindow::Shop, true);
        shop_manage.item_limit = 5;
        shop_manage.adena = 1234;
        let texts = private_texts(&shop_manage);
        for expected in ["s138", "s137", "s143", "s134", "s428", "s384", "s385", "s1198", "(0/5)", "1,234"] {
            assert!(texts.iter().any(|text| text == expected), "{expected} em {texts:?}");
        }
        let shop_visit = private_texts(&private_view(PrivateWindow::Shop, false));
        for expected in ["s137", "s139", "s142", "s140"] {
            assert!(shop_visit.iter().any(|text| text == expected), "{expected} em {shop_visit:?}");
        }
        assert!(["s384", "s385", "s1198"].iter().all(|hidden| !shop_visit.iter().any(|text| text == hidden)));
        let buy_manage = private_texts(&private_view(PrivateWindow::Buy, true));
        for expected in ["s138", "s502", "s142", "s428", "s384", "s385", "(0/0)"] {
            assert!(buy_manage.iter().any(|text| text == expected), "{expected} em {buy_manage:?}");
        }
        assert!(!buy_manage.iter().any(|text| text == "s1198"));
        let buy_visit = private_texts(&private_view(PrivateWindow::Buy, false));
        for expected in ["s503", "s137", "s143", "s140"] {
            assert!(buy_visit.iter().any(|text| text == expected), "{expected} em {buy_visit:?}");
        }
        assert!(!buy_visit.iter().any(|text| text.starts_with('(')));
    }

    #[test]
    fn the_title_is_set_only_by_the_flag_of_a_visited_sell_store_and_the_buy_window_has_none_100e7550() {
        let mut view = private_view(PrivateWindow::Shop, false);
        assert_eq!(view.private_title(), "");
        view.set_flag_title();
        assert_eq!(view.private_title(), "s498");
        view.package_sale = true;
        view.set_flag_title();
        assert_eq!(view.private_title(), "s498 - s1198");
        view.private_manage = true;
        assert_eq!(view.private_title(), "s498 - s1198", "o modo 1 não troca o título");
        view.private_window = PrivateWindow::Buy;
        assert_eq!(view.private_title(), "");
    }

    fn preview_entry(item_id: i32, price: i64) -> ShopEntry {
        entry(0, item_id, 0, 0, price, 0, "p")
    }

    #[test]
    fn preview_mode_moves_single_pieces_both_ways_and_sums_price_times_quantity() {
        let mut view = ShopView::with_sysstrings(|_| None);
        view.open_preview(77, 5, 1_000, vec![preview_entry(1, 100), preview_entry(2, 250)]);
        assert_eq!(view.mode, ShopMode::Preview);
        assert_eq!(ShopMode::Preview.labels(), (811, 812, 813));
        assert!(ShopMode::Preview.consumes_source());
        view.select(Side::Top, 0);
        assert_eq!(view.press_move(Side::Top), ShopAction::None);
        assert_eq!((view.top.len(), view.bottom.len()), (1, 1));
        assert_eq!(view.bottom[0].quantity, 1);
        view.select(Side::Top, 0);
        view.press_move(Side::Top);
        assert_eq!(view.total(), 350);
        view.select(Side::Bottom, 0);
        view.press_move(Side::Bottom);
        assert_eq!((view.top.len(), view.bottom.len()), (1, 1));
        assert_eq!(view.total(), 250);
    }

    #[test]
    fn preview_stackable_pieces_ask_the_amount_and_merge_like_10104ac0_10103c70() {
        let mut view = ShopView::with_sysstrings(|_| None);
        view.open_preview(1, 2, 0, vec![entry(0, 7, 0, 0, 5, 1, "e"), entry(0, 8, 0, 0, 5, 0, "g")]);
        view.select(Side::Top, 1);
        assert_eq!(view.press_move(Side::Top), ShopAction::None);
        assert_eq!(view.bottom.len(), 1);
        view.select(Side::Top, 0);
        assert!(matches!(view.press_move(Side::Top), ShopAction::AskCount { side: Side::Top, index: 0, .. }));
        view.move_entry(Side::Top, 0, 3);
        assert_eq!((view.top.len(), view.bottom.len(), view.bottom[1].quantity), (0, 2, 3));
        view.open_preview(1, 2, 0, vec![entry(0, 7, 9, 0, 5, 1, "e")]);
        view.move_entry(Side::Top, 0, 2);
        assert_eq!((view.top[0].quantity, view.bottom[0].quantity), (7, 2));
        view.move_entry(Side::Top, 0, 4);
        assert_eq!((view.top[0].quantity, view.bottom[0].quantity), (3, 6));
    }

    #[test]
    fn shop_entries_carry_the_tip_options_of_10102aa0_shopfilltooltip() {
        let mut view = ShopView::with_sysstrings(|_| None);
        view.open(ShopMode::Buy, 1, 0, vec![entry(0, 7, 5, 5, 9, 1, "e")]);
        assert_eq!(view.top[0].tip_flags, 7);
        view.select(Side::Top, 0);
        view.move_entry(Side::Top, 0, 2);
        assert_eq!(view.bottom[0].tip_flags, 4);
        view.open(ShopMode::Sell, 1, 0, vec![entry(5, 7, 5, 5, 9, 1, "e")]);
        assert_eq!(view.top[0].tip_flags, 8);
        view.move_entry(Side::Top, 0, 2);
        assert_eq!(view.bottom[0].tip_flags, 8);
        view.open_preview(0, 1, 0, vec![entry(0, 7, 0, 0, 9, 0, "e")]);
        assert_eq!(view.top[0].tip_flags, 7);
        view.move_entry(Side::Top, 0, 1);
        assert_eq!(view.bottom[0].tip_flags, 4);
    }

    #[test]
    fn preview_request_carries_the_header_the_list_id_and_the_selected_item_ids() {
        let mut view = ShopView::with_sysstrings(|_| None);
        view.open_preview(77, 5, 0, vec![preview_entry(1, 100), preview_entry(2, 250)]);
        assert_eq!(view.confirm(), ShopAction::None);
        assert_eq!(view.preview_request(), None);
        view.select(Side::Top, 1);
        view.press_move(Side::Top);
        assert_eq!(view.confirm(), ShopAction::AskPreview);
        assert_eq!(view.preview_request(), Some((77, 5, vec![2])));
        view.clear();
        assert_eq!(view.preview_request(), None);
    }

    #[test]
    fn preview_source_holds_24_pieces_and_a_19th_selection_vanishes_like_the_original() {
        let mut view = ShopView::with_sysstrings(|_| None);
        view.open_preview(0, 1, 0, (0..40).map(|id| preview_entry(id + 1, 1)).collect());
        assert_eq!(view.top.len(), PREVIEW_SOURCE_CAPACITY);
        for _ in 0..PREVIEW_SELECTION_CAPACITY {
            view.select(Side::Top, 0);
            view.press_move(Side::Top);
        }
        assert_eq!(view.bottom.len(), PREVIEW_SELECTION_CAPACITY);
        let before = view.top.len();
        view.select(Side::Top, 0);
        view.press_move(Side::Top);
        assert_eq!(view.bottom.len(), PREVIEW_SELECTION_CAPACITY);
        assert_eq!(view.top.len(), before - 1);
    }

    #[test]
    fn preview_footer_prints_the_32_low_bits_and_the_title_is_trying_on() {
        let mut view = ShopView::with_sysstrings(|id| Some(format!("s{id}")));
        view.open_preview(0, 1, 0x1_0000_0005, vec![]);
        assert_eq!(view.title(), "s847");
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font(), 1.0);
        assert!(!list.quads.is_empty());
    }

    fn manor_view(mode: ShopMode) -> ShopView {
        let mut view = ShopView::with_sysstrings(|id| Some(format!("s{id}")));
        view.open(mode, 3, 5_000, vec![entry(10, 5016, 40, 40, 7, 1, "seed"), entry(11, 1, 2, 2, 9, 0, "sword")]);
        view
    }

    #[test]
    fn nc_item_wnd_100307a0_manor_tip_flags_follow_the_destination_list_and_reset_on_stack_changes() {
        let mut seed = manor_view(ShopMode::ManorSeed);
        assert_eq!(seed.top.iter().map(|e| e.tip_flags).collect::<Vec<_>>(), vec![5, 5]);
        seed.move_entry(Side::Top, 0, 4);
        assert_eq!(seed.bottom[0].tip_flags, TIP_PRICE_BIT);
        assert_eq!(seed.top[0].tip_flags, 0);
        seed.move_entry(Side::Top, 0, 4);
        assert_eq!(seed.bottom[0].tip_flags, 0);
        let mut procure = manor_view(ShopMode::ManorProcure);
        assert_eq!(procure.top[0].tip_flags, TIP_PRICE_PREFIXED_BIT);
        procure.move_entry(Side::Top, 1, 1);
        assert_eq!(procure.bottom[0].tip_flags, TIP_PRICE_PREFIXED_BIT);
    }

    #[test]
    fn nc_item_wnd_100307a0_price_rows_format_the_label_value_and_tint_like_1002e960_1002ea10() {
        let lookup = |id: u32| Some(format!("s{id}"));
        let lines = price_tip_lines(TIP_PRICE_BIT | 1, 1_234_567, &lookup);
        assert_eq!(lines.len(), 1);
        assert_eq!((lines[0].label.as_str(), lines[0].value.as_str()), ("s322", "1,234,567 s469"));
        assert_eq!(lines[0].color, crate::edit_box::numeric_tint(&"1,234,567".encode_utf16().collect::<Vec<_>>()));
        let lines = price_tip_lines(TIP_PRICE_PREFIXED_BIT, 90, &lookup);
        assert_eq!((lines[0].value.as_str(), lines[0].color), ("s468 90 s469", crate::color::TEXT));
        let lines = price_tip_lines(TIP_PRICE_BIT | TIP_VALUE_BIT, 100_000, &lookup);
        assert_eq!(lines.iter().map(|l| (l.label.as_str(), l.value.as_str())).collect::<Vec<_>>(), vec![("s322", "100,000 s469"), ("s641", "100,000")]);
        assert!(price_tip_lines(0, 5, &lookup).is_empty());
    }

    #[test]
    fn manor_modes_use_the_native_title_labels_capacity_and_no_counts() {
        let view = manor_view(ShopMode::ManorSeed);
        assert_eq!(view.title(), "s738");
        assert_eq!(ShopMode::ManorSeed.labels(), (137, 139, 142));
        assert_eq!(ShopMode::ManorProcure.labels(), (138, 137, 143));
        assert_eq!((ShopMode::ManorSeed.capacity(), ShopMode::ManorProcure.capacity()), (160, 160));
        assert!(!view.top_grid.show_count && !view.bottom_grid.show_count);
        assert_eq!((view.list_id, view.adena), (3, 5_000));
    }

    #[test]
    fn manor_buttons_ask_the_count_with_the_original_stock_as_the_maximum() {
        let mut view = manor_view(ShopMode::ManorSeed);
        view.select(Side::Top, 0);
        assert_eq!(
            view.press_move(Side::Top),
            ShopAction::AskCount { side: Side::Top, index: 0, item_name: "item5016".into(), max: 40, consume_type: 1 }
        );
        view.move_entry(Side::Top, 0, 15);
        assert_eq!((view.top[0].quantity, view.bottom[0].quantity), (25, 15));
        view.select(Side::Bottom, 0);
        assert_eq!(
            view.press_move(Side::Bottom),
            ShopAction::AskCount { side: Side::Bottom, index: 0, item_name: "item5016".into(), max: 40, consume_type: 1 }
        );
        view.move_entry(Side::Bottom, 0, 100);
        assert_eq!((view.top[0].quantity, view.bottom.len()), (40, 0));
        view.select(Side::Top, 1);
        assert_eq!(view.press_move(Side::Top), ShopAction::None);
        assert_eq!((view.top.len(), view.bottom.len()), (1, 1));
    }

    #[test]
    fn buy_double_click_asks_the_count_for_any_stack_even_of_one() {
        let mut view = ShopView::with_sysstrings(|id| Some(format!("s{id}")));
        view.open(ShopMode::Buy, 7, 1000, vec![entry(0, 2, 5, 5, 30, 1, "b")]);
        view.select(Side::Top, 0);
        view.move_entry(Side::Top, 0, 1);
        view.select(Side::Bottom, 0);
        assert_eq!(view.bottom[0].quantity, 1);
        assert!(matches!(view.press_double(Side::Bottom), ShopAction::AskCount { max: 1, .. }));
        assert_eq!(view.press_move(Side::Bottom), ShopAction::None);
    }

    #[test]
    fn manor_double_click_and_drop_rules_differ_per_mode() {
        let mut seed = manor_view(ShopMode::ManorSeed);
        seed.select(Side::Top, 0);
        seed.move_entry(Side::Top, 0, 1);
        seed.select(Side::Bottom, 0);
        assert!(matches!(seed.press_double(Side::Bottom), ShopAction::AskCount { .. }));
        let mut seed = manor_view(ShopMode::ManorSeed);
        seed.select(Side::Top, 0);
        seed.move_entry(Side::Top, 0, 1);
        seed.select(Side::Bottom, 0);
        assert_eq!(seed.press_drop(Side::Bottom), ShopAction::None);
        let mut procure = manor_view(ShopMode::ManorProcure);
        procure.select(Side::Top, 0);
        assert!(matches!(procure.press_double(Side::Top), ShopAction::AskCount { .. }));
        procure.select(Side::Top, 0);
        procure.move_entry(Side::Top, 0, 1);
        procure.select(Side::Bottom, 0);
        assert_eq!(procure.press_double(Side::Bottom), ShopAction::None);
        procure.select(Side::Top, 0);
        assert!(matches!(procure.press_drop(Side::Top), ShopAction::AskCount { .. }));
    }

    #[test]
    fn manor_ok_sends_only_chosen_rows_and_seed_purchases_are_skipped_when_empty() {
        let mut seed = manor_view(ShopMode::ManorSeed);
        assert_eq!(seed.confirm(), ShopAction::Dismiss);
        seed.select(Side::Top, 0);
        seed.move_entry(Side::Top, 0, 4);
        assert_eq!(seed.confirm(), ShopAction::BuySeed { manor_id: 3, orders: vec![(5016, 4)] });
        assert_eq!(seed.total(), 28);
        let mut procure = manor_view(ShopMode::ManorProcure);
        assert_eq!(procure.confirm(), ShopAction::Procure { manor_id: 3, orders: vec![] });
        procure.select(Side::Top, 0);
        procure.move_entry(Side::Top, 0, 6);
        assert_eq!(procure.confirm(), ShopAction::Procure { manor_id: 3, orders: vec![(10, 5016, 6)] });
    }

    #[test]
    fn manor_lists_stop_at_160_entries_but_still_consume_the_source() {
        let mut view = manor_view(ShopMode::ManorSeed);
        view.bottom = (0..160).map(|id| entry(id, 1000 + id, 1, 1, 1, 0, "x")).collect();
        view.select(Side::Top, 0);
        view.move_entry(Side::Top, 0, 5);
        assert_eq!(view.bottom.len(), 160);
        assert_eq!(view.top[0].quantity, 35);
    }

    #[test]
    fn the_gm_warehouse_paints_up_down_ok_and_cancel_disabled() {
        let paint = |gm: bool| {
            let mut view = buy_view();
            view.gm = gm;
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font(), 1.0);
            list
        };
        let regions = |list: &DrawList| {
            [UP_RECT, DOWN_RECT, OK_RECT, CANCEL_RECT]
                .map(|rect| list.buttons.iter().find(|b| (b.x, b.y, b.w, b.h) == (rect.x, rect.y, rect.w, rect.h)).map(|b| b.enabled))
        };
        assert_eq!(regions(&paint(false)), [Some(true); 4]);
        assert_eq!(regions(&paint(true)), [Some(false); 4]);
        let grey = crate::color::BUTTON_TEXT_DISABLED.rgba();
        assert!(paint(true).quads.iter().any(|q| q.color == grey));
        assert!(paint(false).quads.iter().all(|q| q.color != grey));
    }
}
