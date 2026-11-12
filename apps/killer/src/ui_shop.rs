use killer_assets::client::DatTables;
use killer_ui::dialog::{DialogKind, DialogView};
use killer_ui::item_grid::{GridEntry, ItemGrid};
use killer_ui::private_shop_item as psi;
use killer_ui::private_store as ps;
use killer_ui::scroll::{ScrollHit, line_down, line_up};
use killer_ui::shop::{self, PrivateWindow, ShopAction, ShopEntry, ShopHit, ShopMode, ShopView, Side};
use killer_ui::window::{Desktop, WindowId};
use killer_world::{NetCommand, WorldEvent};
use l2_net::game::SystemMessageArg;

use crate::ui_keys::UiCommand;

pub struct ShopWindow {
    pub id: WindowId,
    pub view: ShopView,
    cursor_drag: bool,
    pressed: Option<ShopHit>,
    pending_dialog: Option<(u32, String, Option<i64>)>,
    count_target: Option<(Side, usize)>,
    pub consume_type: u32,
    pub private_store_id: Option<i32>,
    pub private_manage: bool,
    pending_preview: bool,
    private_flow: Option<(Side, usize, u32)>,
    private_dialog: Option<psi::Dialog>,
    item_limits: (i32, i32),
    store_messages: std::collections::HashMap<(bool, i32), String>,
    pending_message: bool,
    message_flow: Option<bool>,
    pub messages: Vec<u32>,
    pub sounds: Vec<&'static str>,
}

impl ShopWindow {
    pub fn demo_event(sell: bool) -> WorldEvent {
        let list = l2_net::game::ShopList {
            money: crate::ui_keys::DEMO_ADENA,
            list_id: 7,
            items: [1, 17, 57, 1060].into_iter().map(|id| l2_net::game::PricedItem {
                item: l2_net::game::TradeItem {
                    type1: 0, object_id: id + 100000, item_id: id, count: 50, type2: 0,
                    custom_type1: 0, body_part: 0, enchant: 0, custom_type2: 0,
                },
                price: 10,
            }).collect(),
        };
        if sell { WorldEvent::SellList(list) } else { WorldEvent::BuyList(list) }
    }

    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self {
            id: WindowId::SHOP,
            view: ShopView::with_sysstrings(lookup),
            pressed: None, cursor_drag: false,
            pending_dialog: None,
            count_target: None,
            consume_type: 0,
            private_store_id: None,
            private_manage: false,
            pending_preview: false,
            private_flow: None,
            private_dialog: None,
            item_limits: (0, 0),
            store_messages: std::collections::HashMap::new(),
            pending_message: false,
            message_flow: None,
            messages: Vec::new(),
            sounds: Vec::new(),
        }
    }

    pub fn world_event(&mut self, event: &WorldEvent, tables: Option<&DatTables>, desktop: &mut Desktop) -> bool {
        if let WorldEvent::StorageMaxCount(limits) = event {
            self.view.warehouse_limit = limits.warehouse_limit();
            return false;
        }
        let manor_items: Vec<(l2_net::game::TradeItem, i32)> = match event {
            WorldEvent::BuyListSeed(list) => list.items.iter().map(|item| (manor_trade_item(item.object_id, item.item_id, item.count, item.enchant), item.price)).collect(),
            WorldEvent::SellListProcure(list) => list
                .items
                .iter()
                .map(|item| (manor_trade_item(item.values[0], item.values[1], item.values[2], 0), item.price))
                .collect(),
            _ => Vec::new(),
        };
        let gm_items: Vec<l2_net::game::TradeItem> = match event {
            WorldEvent::GmViewWarehouseWithdrawList(list) => list
                .items
                .iter()
                .map(|item| l2_net::game::TradeItem {
                    type1: item.kind,
                    object_id: item.values[0],
                    item_id: item.values[1],
                    count: item.values[2],
                    type2: item.shorts[0],
                    custom_type1: item.shorts[1],
                    body_part: item.details.map_or(0, |details| details.0),
                    enchant: item.details.map_or(0, |details| details.1[0]),
                    custom_type2: 0,
                })
                .collect(),
            _ => Vec::new(),
        };
        let (mode, list_id, money, items): (_, _, _, Vec<_>) = match event {
            WorldEvent::BuyList(list) if self.id == WindowId::SHOP =>
                (ShopMode::Buy, list.list_id, list.money, list.items.iter().map(|p| (&p.item, p.price, 0)).collect()),
            WorldEvent::SellList(list) if self.id == WindowId::SHOP =>
                (ShopMode::Sell, list.list_id, list.money, list.items.iter().map(|p| (&p.item, p.price, 0)).collect()),
            WorldEvent::BuyListSeed(list) if self.id == WindowId::MANOR_SHOP =>
                (ShopMode::ManorSeed, list.manor_id, list.money, manor_items.iter().map(|(item, price)| (item, *price, 0)).collect()),
            WorldEvent::SellListProcure(list) if self.id == WindowId::MANOR_SHOP =>
                (ShopMode::ManorProcure, list.header[1], list.header[0], manor_items.iter().map(|(item, price)| (item, *price, 0)).collect()),
            WorldEvent::GmViewWarehouseWithdrawList(list) if self.id == WindowId::GM_STORE => {
                self.view.gm = true;
                self.view.warehouse_type = 0;
                (ShopMode::Withdraw, -1, list.header_value, gm_items.iter().map(|item| (item, 0, 0)).collect())
            }
            WorldEvent::WarehouseDepositList(list) | WorldEvent::WarehouseWithdrawList(list) if self.id == WindowId::WAREHOUSE => {
                self.view.warehouse_type = i32::from(list.warehouse_type);
                let mode = if matches!(event, WorldEvent::WarehouseDepositList(_)) { ShopMode::Deposit } else { ShopMode::Withdraw };
                (mode, -1, list.adena, list.items.iter().map(|p| (&p.item, 0, p.warehouse_id)).collect())
            }
            WorldEvent::PackageSendableList(list) if self.id == WindowId::DELIVERY && !list.items.is_empty() =>
                (ShopMode::Deliver, list.target_id, list.adena, list.items.iter().map(|p| (&p.item, 0, p.warehouse_id)).collect()),
            _ => return false,
        };
        self.clear();
        let items = items.into_iter().filter_map(|(item, price, warehouse_id)| {
            let id = u32::try_from(item.item_id).ok()?;
            let tables = tables?;
            let icon = tables.item_icons.get(id)?;
            let name = tables.itemname.get(&id);
            Some(ShopEntry {
                reference_price: 0,
                wanted: -1,
                grid: GridEntry {
                    object_id: item.object_id,
                    item_id: item.item_id,
                    count: item.count,
                    enchant: i32::from(item.enchant),
                    icon: icon.icons[0].trim().to_string(),
                    name: name.map(|n| n.name.clone()).unwrap_or_default(),
                    add_name: name.map(|n| n.add_name.clone()).unwrap_or_default(),
                    description: name.map(|n| n.description.clone()).unwrap_or_default(),
                    weight: icon.weight as i32,
                },
                quantity: i64::from(item.count),
                stock: item.count,
                price: i64::from(price),
                consume_type: tables.etcitemgrp.get(&id).map_or(0, |e| e.stackable),
                warehouse_id,
                tip_flags: 0,
            })
        }).take(mode.capacity()).collect();
        self.view.open(mode, list_id, i64::from(money), items);
        if matches!(mode, ShopMode::Sell | ShopMode::Deposit | ShopMode::ManorProcure) {
            visible(desktop, WindowId::INVENTORY, false);
        }
        if mode != ShopMode::Deliver {
            visible(desktop, WindowId::NPC_HTML, false);
        }
        visible(desktop, self.id, true);
        true
    }

    pub fn private_event(&mut self, event: &WorldEvent, tables: Option<&DatTables>, desktop: &mut Desktop) -> bool {
        match event {
            WorldEvent::StorageMaxCount(limits) => {
                self.item_limits = (limits.private_sell_limit(), limits.private_buy_limit());
                return false;
            }
            WorldEvent::PrivateStoreMsgSell(msg) => {
                self.store_messages.insert((false, msg.object_id), msg.message.clone());
                return false;
            }
            WorldEvent::PrivateStoreMsgBuy(msg) => {
                self.store_messages.insert((true, msg.object_id), msg.message.clone());
                return false;
            }
            _ => {}
        }
        let Some(tables) = tables else { return false };
        let (window, mode, store_id, adena, package_sale, entries): (PrivateWindow, ShopMode, i32, i32, bool, Vec<ShopEntry>) = match event {
            WorldEvent::PrivateStoreListSell(list) => (
                PrivateWindow::Shop,
                ShopMode::Buy,
                list.object_id,
                list.adena,
                list.package_sale,
                list.items.iter().filter_map(|item| private_sell_entry(item, tables, true)).take(ShopMode::Buy.capacity()).collect(),
            ),
            WorldEvent::PrivateStoreListBuy(list) => (
                PrivateWindow::Buy,
                ShopMode::Sell,
                list.object_id,
                list.adena,
                false,
                list.items.iter().filter_map(|item| private_buy_entry(item, tables, true)).take(ShopMode::Sell.capacity()).collect(),
            ),
            _ => return false,
        };
        self.clear();
        self.private_store_id = Some(store_id);
        self.view.open(mode, store_id, i64::from(adena), entries);
        self.set_private(window, false);
        self.view.package_sale = package_sale;
        if window == PrivateWindow::Shop {
            self.view.set_flag_title();
        }
        visible(desktop, self.id, true);
        true
    }

    fn set_private(&mut self, window: PrivateWindow, manage: bool) {
        self.private_manage = manage;
        self.view.private_window = window;
        self.view.private_manage = manage;
        self.view.item_limit = if window == PrivateWindow::Shop { self.item_limits.0 } else { self.item_limits.1 };
        if window == PrivateWindow::Buy {
            self.view.use_private_buy_grids();
        }
    }

    pub fn private_manage_event(&mut self, event: &WorldEvent, tables: Option<&DatTables>, desktop: &mut Desktop) -> bool {
        let Some(tables) = tables else { return false };
        let (window, mode, store_id, adena) = match event {
            WorldEvent::PrivateStoreManageListSell(list) => (PrivateWindow::Shop, ShopMode::Sell, list.object_id, list.adena),
            WorldEvent::PrivateStoreManageListBuy(list) => (PrivateWindow::Buy, ShopMode::Buy, list.object_id, list.adena),
            _ => return false,
        };
        self.clear();
        self.private_store_id = Some(store_id);
        self.view.open(mode, store_id, i64::from(adena), Vec::new());
        self.set_private(window, true);
        match event {
            WorldEvent::PrivateStoreManageListSell(list) => {
                self.view.package_sale = list.package_sale;
                self.view.top = list.available.iter().filter_map(|item| private_sell_entry(item, tables, false)).collect();
                self.view.bottom = list.selling.iter().filter_map(|item| private_sell_entry(item, tables, true)).collect();
            }
            WorldEvent::PrivateStoreManageListBuy(list) => {
                self.view.top = list.available.iter().filter_map(|item| private_buy_entry(item, tables, false)).collect();
                self.view.bottom = list.buying.iter().filter_map(|item| private_buy_entry(item, tables, true)).collect();
            }
            _ => {}
        }
        self.view.top_grid.update_rows(self.view.top.len());
        self.view.bottom_grid.update_rows(self.view.bottom.len());
        visible(desktop, self.id, true);
        true
    }

    pub fn private_quit(&self) -> Option<NetCommand> {
        self.private_manage.then(|| if self.view.mode == ShopMode::Sell { NetCommand::PrivateStoreQuitSell } else { NetCommand::PrivateStoreQuitBuy })
    }

    pub fn clear(&mut self) {
        self.view.clear();
        self.pressed = None;
        self.pending_dialog = None;
        self.count_target = None;
        self.private_store_id = None;
        self.private_manage = false;
        self.pending_preview = false;
        self.private_flow = None;
        self.private_dialog = None;
        self.pending_message = false;
        self.message_flow = None;
        self.view.private_manage = false;
        self.view.package_sale = false;
    }

    pub fn preview_event(&mut self, event: &WorldEvent, tables: Option<&DatTables>, desktop: &mut Desktop) -> bool {
        let WorldEvent::ShopPreviewList(list) = event else { return false };
        if self.id != WindowId::SHOP_PREVIEW {
            return false;
        }
        let Some(tables) = tables else { return false };
        self.clear();
        let items = list.items.iter().filter_map(|item| {
            let id = u32::try_from(item.item_id).ok()?;
            let icon = tables.item_icons.get(id)?;
            let name = tables.itemname.get(&id);
            Some(ShopEntry {
                reference_price: 0,
                wanted: -1,
                grid: GridEntry {
                    object_id: 0,
                    item_id: item.item_id,
                    count: 0,
                    enchant: 0,
                    icon: icon.icons[0].trim().to_string(),
                    name: name.map(|n| n.name.clone()).unwrap_or_default(),
                    add_name: name.map(|n| n.add_name.clone()).unwrap_or_default(),
                    description: name.map(|n| n.description.clone()).unwrap_or_default(),
                    weight: icon.weight as i32,
                },
                quantity: 0,
                stock: 0,
                price: i64::from(item.price),
                consume_type: tables.etcitemgrp.get(&id).map_or(0, |e| e.stackable),
                warehouse_id: 0,
                tip_flags: 0,
            })
        }).collect();
        self.view.open_preview(list.header, list.list_id, i64::from(list.money), items);
        visible(desktop, WindowId::NPC_HTML, false);
        visible(desktop, WindowId::SHOP, false);
        visible(desktop, self.id, true);
        true
    }

    pub fn answer_confirm(&mut self, accept: bool, desktop: &mut Desktop) -> Option<NetCommand> {
        if !accept {
            return None;
        }
        let (header, list_id, items) = self.view.preview_request()?;
        self.clear();
        visible(desktop, self.id, false);
        Some(NetCommand::PreviewItem { header, list_id, items })
    }

    fn hit(&self, x: i32, y: i32) -> Option<ShopHit> {
        if self.id == WindowId::PRIVATE_SHOP { self.view.private_hit(x, y) } else { self.view.hit(x, y) }
    }

    fn grid_mut(&mut self, side: Side) -> &mut ItemGrid {
        match side {
            Side::Top => &mut self.view.top_grid,
            Side::Bottom => &mut self.view.bottom_grid,
        }
    }

    pub fn cursor_hover(&mut self) { self.cursor_drag = self.pressed.is_some(); }

    pub fn drag_icon(&self) -> Option<String> {
        if !self.cursor_drag { return None; }
        let Some(ShopHit::Slot(side, index)) = self.pressed else { return None; };
        self.view.entries(side).get(index).map(|entry| entry.grid.icon.clone()).filter(|icon| !icon.is_empty())
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, desktop: &mut Desktop) {
        self.cursor_drag = false;
        self.pressed = self.hit(x, y);
        match self.pressed {
            Some(ShopHit::Close) => {
                self.clear();
                visible(desktop, self.id, false);
                if self.id == WindowId::MANOR_SHOP {
                    self.sounds.push("itemsound.window_close");
                }
            }
            Some(ShopHit::Slot(side, index)) => {
                if index < self.view.entries(side).len() {
                    self.sounds.push("itemsound.pickup");
                }
                self.view.select(side, index)
            }
            Some(ShopHit::Scroll(side, hit)) => {
                let grid = self.grid_mut(side);
                grid.scroll.up_pressed = hit == ScrollHit::Up;
                grid.scroll.down_pressed = hit == ScrollHit::Down;
                if let ScrollHit::Slider(pos) = hit {
                    grid.scroll.slider.mouse_down(pos);
                }
            }
            Some(ShopHit::Ok) => self.view.ok_pressed = true,
            Some(ShopHit::Cancel) => self.view.cancel_pressed = true,
            Some(ShopHit::Package) if self.private_manage => self.view.package_sale = !self.view.package_sale,
            Some(ShopHit::Up) => self.view.up_pressed = true,
            Some(ShopHit::Down) => self.view.down_pressed = true,
            _ => {}
        }
    }

    pub fn mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop, commands: &mut Vec<UiCommand>) {
        self.view.ok_pressed = false;
        self.view.cancel_pressed = false;
        self.view.up_pressed = false;
        self.view.down_pressed = false;
        let released = over.filter(|(id, _, _)| *id == self.id).and_then(|(_, x, y)| self.hit(x, y));
        let Some(pressed) = self.pressed.take() else { return };
        if self.private_active()
            && let ShopHit::Slot(from, index) = pressed
        {
            if matches!(released, Some(ShopHit::Slot(to, _)) if to != from) {
                self.private_drop(from, index, false);
            }
            return;
        }
        if let ShopHit::Scroll(side, hit) = pressed {
            let count = self.view.entries(side).len();
            let grid = self.grid_mut(side);
            grid.scroll.up_pressed = false;
            grid.scroll.down_pressed = false;
            grid.scroll.slider.mouse_up();
            if released == Some(pressed) {
                let first = grid.scroll_row as i32;
                let rows = grid.rows as i32;
                let total = grid.total_rows(count);
                match hit {
                    ScrollHit::Up => grid.set_first_row(line_up(first, rows, total)),
                    ScrollHit::Down => grid.set_first_row(line_down(first, rows, total)),
                    _ => {}
                }
            }
            return;
        }
        let action = match (pressed, released) {
            (ShopHit::Slot(from, index), Some(ShopHit::Slot(to, _))) if from != to => {
                self.view.select(from, index);
                self.view.press_drop(from)
            }
            (ShopHit::Slot(side, index), Some(ShopHit::Slot(_, _)))
                if matches!(self.id, WindowId::SHOP | WindowId::PRIVATE_SHOP | WindowId::SHOP_PREVIEW) && !self.private_manage =>
            {
                self.view.select(side, index);
                self.view.press_move(side)
            }
            _ if released != Some(pressed) => return,
            (ShopHit::Down, _) if self.private_active() => {
                self.private_arrow(Side::Top, true);
                return;
            }
            (ShopHit::Up, _) if self.private_active() => {
                self.private_arrow(Side::Bottom, false);
                return;
            }
            (ShopHit::Ok, _) if self.private_active() => {
                self.private_start(desktop, commands);
                return;
            }
            (ShopHit::Message, _) if self.private_manage => {
                self.pending_message = true;
                return;
            }
            (ShopHit::Down, _) => self.view.press_move(Side::Top),
            (ShopHit::Up, _) => self.view.press_move(Side::Bottom),
            (ShopHit::Ok, _) => self.view.confirm(),
            (ShopHit::Cancel, _) => {
                if let Some(command) = self.private_quit() {
                    commands.push(UiCommand::Net(command));
                }
                self.clear();
                visible(desktop, self.id, false);
                return;
            }
            _ => return,
        };
        self.action(action, desktop, commands);
    }

    fn action(&mut self, action: ShopAction, desktop: &mut Desktop, commands: &mut Vec<UiCommand>) {
        let command = match action {
            ShopAction::AskCount { side, index, item_name, max, consume_type } => {
                self.count_target = Some((side, index));
                self.consume_type = consume_type;
                self.pending_dialog = Some((shop::MOVE_COUNT_SYSMSG, item_name, Some(max)));
                return;
            }
            ShopAction::AskPreview => {
                self.pending_preview = true;
                return;
            }
            ShopAction::TooMany => {
                self.count_target = None;
                self.pending_dialog = Some((shop::TOO_MANY_SYSMSG, String::new(), None));
                return;
            }
            ShopAction::Buy { list_id, orders } => NetCommand::BuyItems { list_id, orders },
            ShopAction::Dismiss => {
                self.clear();
                visible(desktop, self.id, false);
                self.count_target = None;
                return;
            }
            ShopAction::BuySeed { manor_id, orders } => NetCommand::BuySeed { manor_id, orders },
            ShopAction::Procure { manor_id, orders } => NetCommand::ProcureCrop {
                manor_id,
                orders: orders.into_iter().map(|(object_id, item_id, count)| l2_net::game::ItemOrder { object_id, item_id, count }).collect(),
            },
            ShopAction::Deliver { target_id, orders } => NetCommand::PackageSend { target_id, orders },
            ShopAction::Deposit { orders } => NetCommand::WarehouseDeposit { orders },
            ShopAction::Withdraw { orders } => NetCommand::WarehouseWithdraw { orders },
            ShopAction::Sell { list_id, orders } => NetCommand::SellItems {
                list_id,
                orders: orders.into_iter().map(|(object_id, item_id, count)| l2_net::game::ItemOrder { object_id, item_id, count }).collect(),
            },
            _ => return,
        };
        if matches!(self.id, WindowId::DELIVERY | WindowId::MANOR_SHOP) { self.clear(); }
        commands.push(UiCommand::Net(command));
        visible(desktop, self.id, false);
        self.count_target = None;
    }

    pub fn double_click(&mut self, x: i32, y: i32) -> bool {
        let Some(ShopHit::Slot(side, index)) = self.hit(x, y) else { return false };
        self.pressed = None;
        self.view.select(side, index);
        if self.private_active() {
            self.private_drop(side, index, true);
            return true;
        }
        if let ShopAction::AskCount { item_name, max, consume_type, .. } = self.view.press_double(side) {
            self.count_target = Some((side, index));
            self.consume_type = consume_type;
            self.pending_dialog = Some((shop::MOVE_COUNT_SYSMSG, item_name, Some(max)));
        }
        true
    }

    pub fn mouse_move(&mut self, y: i32) -> bool {
        let Some(ShopHit::Scroll(side, ScrollHit::Slider(_))) = self.pressed else { return false };
        let grid = self.grid_mut(side);
        if let Some(first) = grid.scroll.slider.mouse_move(y - grid.rect.y - killer_ui::scroll::BUTTON_SIZE) {
            grid.scroll_row = first.max(0) as usize;
        }
        true
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        for side in [Side::Top, Side::Bottom] {
            let count = self.view.entries(side).len();
            let grid = self.grid_mut(side);
            if grid.rect.contains(x, y) {
                let first = killer_ui::inventory::item_wnd_wheel(grid.scroll_row as i32, grid.rows as i32, grid.total_rows(count), delta);
                grid.set_first_row(first);
                return true;
            }
        }
        false
    }

    fn private_active(&self) -> bool {
        self.id == WindowId::PRIVATE_SHOP && self.private_store_id.is_some()
    }

    fn private_buy_wnd(&self) -> bool {
        self.view.private_window == PrivateWindow::Buy
    }

    fn private_mode(&self) -> i32 {
        if self.private_manage { psi::MODE_MANAGE } else { psi::MODE_VISIT }
    }

    fn private_kind(side: Side) -> i32 {
        match side {
            Side::Top => psi::KIND_SELECTION,
            Side::Bottom => psi::KIND_ORIGIN,
        }
    }

    fn peer(side: Side) -> Side {
        match side {
            Side::Top => Side::Bottom,
            Side::Bottom => Side::Top,
        }
    }

    fn private_grid(&self, from: Side) -> psi::Grid {
        psi::Grid {
            mode: self.private_mode(),
            peer_kind: Self::private_kind(Self::peer(from)),
            package_sale: self.view.package_sale,
        }
    }

    fn private_item(&self, side: Side, index: usize) -> Option<psi::Item> {
        let entry = self.view.entries(side).get(index)?;
        Some(psi::Item {
            item_id: entry.grid.item_id,
            name: entry.grid.name.clone(),
            stackable: entry.stackable(),
            count: entry.quantity,
            stock: entry.stock,
            price: entry.price as i32,
            reference_price: entry.reference_price as i32,
        })
    }

    fn private_buy_item(&self, side: Side, index: usize) -> Option<ps::BuyItem> {
        let entry = self.view.entries(side).get(index)?;
        Some(ps::BuyItem {
            item_id: entry.grid.item_id,
            name: entry.grid.name.clone(),
            stackable: entry.stackable(),
            stock: i64::from(entry.stock),
            quantity: entry.quantity,
            wanted: i64::from(entry.wanted),
            price: entry.price as i32,
            reference_price: entry.reference_price as i32,
        })
    }

    fn private_drop(&mut self, side: Side, index: usize, double_click: bool) {
        let outcome = if self.private_buy_wnd() {
            let Some(item) = self.private_buy_item(side, index) else { return };
            ps::buy_drop(self.private_mode(), Self::private_kind(Self::peer(side)), &item, double_click)
        } else {
            let item = self.private_item(side, index);
            if double_click {
                psi::on_double_click(self.private_grid(side), item.as_ref())
            } else {
                psi::on_lbutton_up(self.private_grid(side), item.as_ref(), true, psi::Release::OverPeer)
            }
        };
        self.apply_private(outcome, side, index);
    }

    fn private_arrow(&mut self, side: Side, down: bool) {
        let index = self.view.selected(side);
        let mode = self.private_mode();
        let outcome = if self.private_buy_wnd() {
            let item = index.and_then(|index| self.private_buy_item(side, index));
            if down { ps::buy_arrow_down(mode, item.as_ref()) } else { ps::buy_arrow_up(mode, item.as_ref()) }
        } else {
            let item = index.and_then(|index| self.private_item(side, index));
            if down {
                ps::shop_arrow_down(mode, self.view.package_sale, item.as_ref())
            } else {
                ps::shop_arrow_up(mode, self.view.package_sale, item.as_ref())
            }
        };
        self.apply_private(outcome, side, index.unwrap_or(0));
    }

    fn private_start(&mut self, desktop: &mut Desktop, commands: &mut Vec<UiCommand>) {
        if !self.private_manage {
            let unusual = self
                .view
                .bottom
                .iter()
                .filter(|entry| entry.quantity > 0)
                .any(|entry| ps::is_unusual_price(entry.price as i32, entry.reference_price as i32));
            if unusual {
                let dialog = ps::caution_for_start(self.private_buy_wnd());
                self.private_flow = Some((Side::Top, 0, dialog.kind));
                self.private_dialog = Some(dialog);
                return;
            }
        }
        self.private_request(desktop, commands);
    }

    fn private_request(&mut self, desktop: &mut Desktop, commands: &mut Vec<UiCommand>) {
        let store_id = self.private_store_id.unwrap_or(-1);
        let chosen = self.view.bottom.iter().filter(|entry| entry.quantity > 0);
        let command = match (self.private_buy_wnd(), self.private_manage) {
            (false, true) => NetCommand::SetPrivateStoreListSell {
                package_sale: self.view.package_sale,
                offers: chosen
                    .map(|entry| l2_net::game::StoreOffer { object_id: entry.grid.object_id, count: entry.quantity as i32, price: entry.price as i32 })
                    .collect(),
            },
            (false, false) => NetCommand::PrivateStoreBuy {
                store_id,
                orders: chosen
                    .map(|entry| l2_net::game::StoreBuyOrder { object_id: entry.grid.object_id, count: entry.quantity as i32, price: entry.price as i32 })
                    .collect(),
            },
            (true, true) => NetCommand::SetPrivateStoreListBuy {
                offers: chosen
                    .map(|entry| l2_net::game::BuyOffer {
                        item_id: entry.grid.item_id,
                        enchant: entry.grid.enchant as i16,
                        custom_type2: entry.warehouse_id as i16,
                        count: entry.quantity as i32,
                        price: entry.price as i32,
                    })
                    .collect(),
            },
            (true, false) => NetCommand::PrivateStoreSell {
                store_id,
                items: chosen
                    .map(|entry| l2_net::game::SellToStoreItem {
                        object_id: entry.grid.object_id,
                        item_id: entry.grid.item_id,
                        enchant: entry.grid.enchant as i16,
                        custom_type2: entry.warehouse_id as i16,
                        count: entry.quantity as i32,
                        price: entry.price as i32,
                    })
                    .collect(),
            },
        };
        commands.push(UiCommand::Net(command));
        self.clear();
        visible(desktop, self.id, false);
    }

    fn private_move(&mut self, side: Side, index: usize, count: i64) -> bool {
        if self.private_buy_wnd() { self.view.buy_move(side, index, count) } else { self.view.sell_move(side, index, count) }
    }

    fn apply_private(&mut self, outcome: psi::Outcome, side: Side, index: usize) -> bool {
        match outcome {
            psi::Outcome::Nothing => false,
            psi::Outcome::Move(count) => !self.private_move(side, index, count),
            psi::Outcome::MoveAll => {
                self.view.move_all(side);
                false
            }
            psi::Outcome::Dialog(dialog) => {
                self.private_flow = Some((side, index, dialog.kind));
                self.private_dialog = Some(dialog);
                false
            }
            psi::Outcome::Message { id, .. } => {
                self.messages.push(id);
                false
            }
        }
    }

    pub fn answer_dialog(&mut self, accept: bool, text: Option<&str>, desktop: &mut Desktop, commands: &mut Vec<UiCommand>) {
        if let Some(buy) = self.message_flow.take() {
            if let Some(message) = text.filter(|text| accept && !text.is_empty()) {
                let message = message.to_string();
                commands.push(UiCommand::Net(if buy { NetCommand::SetPrivateStoreMsgBuy { message } } else { NetCommand::SetPrivateStoreMsgSell { message } }));
            }
            return;
        }
        let Some((side, _, kind)) = self.private_flow.take() else {
            self.answer_text(text);
            return;
        };
        if !accept {
            if kind == ps::DIALOG_BUY_CAUTION {
                self.clear();
                visible(desktop, self.id, false);
            }
            return;
        }
        if matches!(kind, ps::DIALOG_BUY_CAUTION | ps::DIALOG_SELL_CAUTION) {
            self.private_request(desktop, commands);
            return;
        }
        let Some(index) = self.view.selected(side) else { return };
        let value = text.map_or(0, |text| text.bytes().take_while(u8::is_ascii_digit).fold(0i64, |n, c| n.saturating_mul(10).saturating_add(i64::from(c - b'0'))));
        let mode = self.private_mode();
        let (outcome, notice_on_refusal) = match kind {
            psi::DIALOG_PRICE => {
                let mut item = self.private_item(side, index);
                let outcome = psi::on_price_answer(mode, item.as_mut(), value);
                if let (Some(item), Some(entry)) = (item, self.view.entries_mut(side).get_mut(index)) {
                    entry.price = i64::from(item.price);
                }
                (outcome, true)
            }
            psi::DIALOG_QUANTITY => (psi::on_quantity_answer(mode, self.private_item(side, index).as_ref(), value), true),
            psi::DIALOG_CONFIRM_PRICE => (psi::on_price_confirmed(self.private_item(side, index).as_ref()), true),
            ps::DIALOG_BUY_PRICE => {
                let mut item = self.private_buy_item(side, index);
                let outcome = ps::on_buy_price(mode, item.as_mut(), value);
                if let (Some(item), Some(entry)) = (item, self.view.entries_mut(side).get_mut(index)) {
                    entry.price = i64::from(item.price);
                }
                (outcome, false)
            }
            ps::DIALOG_BUY_AMOUNT => (ps::on_buy_amount(mode, self.private_buy_item(side, index).as_ref(), value), true),
            ps::DIALOG_BUY_PRICE_CAUTION => (ps::on_buy_price_ok(self.private_buy_item(side, index).as_ref()), false),
            _ => (psi::Outcome::Nothing, false),
        };
        if self.apply_private(outcome, side, index) && notice_on_refusal {
            self.pending_dialog = Some((ps::NOTICE_REFUSED, String::new(), None));
        }
    }

    pub fn take_dialog(&mut self, tables: Option<&DatTables>) -> Option<DialogView> {
        if std::mem::take(&mut self.pending_message) {
            let buy = self.private_buy_wnd();
            self.message_flow = Some(buy);
            let text = crate::ui2d::system_text(tables, ps::MESSAGE_STORE, &[]);
            let mut view = DialogView::with_text(DialogKind::OkCancel, &text);
            if let Some(input) = view.input.as_mut() {
                input.text = self.store_messages.get(&(buy, self.private_store_id.unwrap_or(-1))).cloned().unwrap_or_default();
            }
            view.dialog_type = if buy { ps::DIALOG_MESSAGE_BUY } else { ps::DIALOG_MESSAGE_SELL } as i32;
            return Some(view);
        }
        if let Some(dialog) = self.private_dialog.take() {
            let plain = matches!(dialog.kind, psi::DIALOG_CONFIRM_PRICE | ps::DIALOG_BUY_CAUTION | ps::DIALOG_BUY_PRICE_CAUTION | ps::DIALOG_SELL_CAUTION);
            let args = if plain { vec![] } else { vec![SystemMessageArg::Text(dialog.item_name.clone())] };
            let text = crate::ui2d::system_text(tables, dialog.message, &args);
            let mut view = match dialog.kind {
                _ if plain => DialogView::new(DialogKind::OkCancel, &text),
                psi::DIALOG_PRICE | ps::DIALOG_BUY_PRICE => DialogView::with_count(DialogKind::OkCancel, &text, i64::from(i32::MAX)),
                _ => DialogView::with_count(DialogKind::OkCancel, &text, dialog.max),
            };
            view.dialog_type = dialog.kind as i32;
            return Some(view);
        }
        if std::mem::take(&mut self.pending_preview) {
            let text = crate::ui2d::system_text(tables, shop::PREVIEW_CONFIRM_SYSMSG, &[]);
            let mut view = DialogView::new(DialogKind::OkCancel, &text);
            view.dialog_type = shop::PREVIEW_DIALOG_TYPE;
            return Some(view);
        }
        let (id, name, max) = self.pending_dialog.take()?;
        let args = if max.is_some() { vec![SystemMessageArg::Text(name)] } else { vec![] };
        let text = crate::ui2d::system_text(tables, id, &args);
        let mut view = match max {
            Some(max) => DialogView::with_count(DialogKind::OkCancel, &text, max),
            None => DialogView::new(DialogKind::OkCancel, &text),
        };
        view.dialog_type = if max.is_none() { shop::DIALOG_TYPE_TOO_MANY }
            else if matches!(self.id, WindowId::WAREHOUSE | WindowId::GM_STORE | WindowId::DELIVERY) { 3 } else { shop::DIALOG_TYPE_COUNT };
        Some(view)
    }

    pub fn answer_count(&mut self, count: Option<i64>) {
        if let Some((side, _)) = self.count_target.take()
            && let Some(count) = count
            && let Some(index) = self.view.selected(side)
        {
            self.view.move_entry(side, index, count);
        }
    }

    pub fn answer_text(&mut self, text: Option<&str>) {
        let Some(text) = text else {
            self.answer_count(None);
            return;
        };
        let count = text.bytes().take_while(u8::is_ascii_digit).fold(0i64, |n, c| n.saturating_mul(10).saturating_add(i64::from(c - b'0')));
        if shop::count_limit(self.consume_type).is_some_and(|limit| count > limit) {
            self.count_target = None;
            let message = if self.consume_type == shop::CONSUME_TYPE_ASSET { 1338 } else { 1369 };
            self.pending_dialog = Some((message, String::new(), None));
            return;
        }
        self.answer_count(Some(i64::from(count as i32)));
    }
}

fn manor_trade_item(object_id: i32, item_id: i32, count: i32, enchant: i16) -> l2_net::game::TradeItem {
    l2_net::game::TradeItem { type1: 0, object_id, item_id, count, type2: 0, custom_type1: 0, body_part: 0, enchant, custom_type2: 0 }
}

fn visible(desktop: &mut Desktop, id: WindowId, visible: bool) {
    if let Some(window) = desktop.window_mut(id) {
        window.visible = visible;
    }
    if visible && let Some(index) = desktop.windows.iter().position(|w| w.id == id) {
        let window = desktop.windows.remove(index);
        desktop.windows.push(window);
    }
}

fn private_grid_entry(object_id: i32, item_id: i32, count: i32, enchant: i16, tables: &DatTables) -> Option<(GridEntry, u32)> {
    let id = u32::try_from(item_id).ok()?;
    let icon = tables.item_icons.get(id)?;
    let name = tables.itemname.get(&id);
    Some((
        GridEntry {
            object_id,
            item_id,
            count,
            enchant: i32::from(enchant),
            icon: icon.icons[0].trim().to_string(),
            name: name.map(|n| n.name.clone()).unwrap_or_default(),
            add_name: name.map(|n| n.add_name.clone()).unwrap_or_default(),
            description: name.map(|n| n.description.clone()).unwrap_or_default(),
            weight: icon.weight as i32,
        },
        tables.etcitemgrp.get(&id).map_or(0, |e| e.stackable),
    ))
}

fn private_sell_entry(item: &l2_net::game::StoreSellItem, tables: &DatTables, listed: bool) -> Option<ShopEntry> {
    let (grid, consume_type) = private_grid_entry(item.object_id, item.item_id, item.count, item.enchant, tables)?;
    let (price, reference_price) = if listed { (item.price, item.store_price) } else { (0, item.price) };
    Some(ShopEntry {
        grid,
        quantity: i64::from(item.count),
        stock: item.count,
        price: i64::from(price),
        reference_price: i64::from(reference_price),
        wanted: -1,
        consume_type,
        warehouse_id: i32::from(item.custom_type2),
        tip_flags: 0,
    })
}

fn private_buy_entry(item: &l2_net::game::StoreBuyItem, tables: &DatTables, with_wanted: bool) -> Option<ShopEntry> {
    let (grid, consume_type) = private_grid_entry(item.object_id, item.item_id, item.count, item.enchant, tables)?;
    Some(ShopEntry {
        grid,
        quantity: i64::from(item.count),
        stock: item.count,
        price: i64::from(item.price),
        reference_price: i64::from(item.store_price),
        wanted: if with_wanted { item.max_count } else { -1 },
        consume_type,
        warehouse_id: i32::from(item.type2),
        tip_flags: 0,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use killer_ui::window::{Rect, Window};
    use l2_dat::itemicons::{ItemIcon, ItemKind};
    use l2_net::game::{PricedItem, ShopList, TradeItem};

    pub fn tables() -> DatTables {
        let mut tables = DatTables::default();
        tables.item_icons.insert_all(vec![ItemIcon {
            id: 57, kind: ItemKind::Etc, drop: Default::default(), icons: std::array::from_fn(|_| "icon.etc_adena_i00".into()), weight: 10, type1: 4,
        }]);
        tables.etcitemgrp.insert(57, l2_dat::etcitemgrp::EtcItem {
            id: 57, icons: std::array::from_fn(|_| String::new()), weight: 10, type1: 4,
            meshes: vec![], textures: vec![], item_sound: String::new(), equip_sound: String::new(), stackable: 3, family: 0, grade: 0, crystallizable: false,
        });
        tables
    }

    pub fn event(buy: bool, count: i32) -> WorldEvent {
        let list = ShopList { money: 500, list_id: 7, items: vec![PricedItem {
            item: TradeItem { type1: 4, object_id: 900, item_id: 57, count, type2: 5, custom_type1: 0, body_part: 0, enchant: 3, custom_type2: 0 },
            price: 12,
        }] };
        if buy { WorldEvent::BuyList(list) } else { WorldEvent::SellList(list) }
    }

    fn setup(buy: bool) -> (ShopWindow, Desktop, Vec<UiCommand>) {
        let mut shop = ShopWindow::new(|_| None);
        let mut desktop = Desktop::new((1000, 800));
        for id in [WindowId::SHOP, WindowId::INVENTORY, WindowId::NPC_HTML] {
            desktop.windows.push(Window::new(id, Rect::new(0, 0, 256, 401), shop::grip()));
        }
        shop.world_event(&event(buy, 50), Some(&tables()), &mut desktop);
        (shop, desktop, vec![])
    }

    #[test]
    fn on_lbutton_up_10104d70_a_single_click_on_a_buy_item_opens_the_count_dialog() {
        let (mut shop, mut desktop, mut commands) = setup(true);
        click(&mut shop, &mut desktop, &mut commands, 12, 52);
        let dialog = shop.take_dialog(None).expect("clique simples de compra abre a contagem");
        assert_eq!(dialog.input.unwrap().max, 50);
        assert!(commands.is_empty());
        assert!(desktop.window(WindowId::SHOP).unwrap().visible);
    }

    #[test]
    fn the_count_answer_moves_the_item_selected_when_it_arrives_not_the_one_asked() {
        let (mut shop, _, _) = setup(false);
        let mut second = shop.view.top[0].clone();
        second.grid.item_id = 58;
        second.grid.object_id = 901;
        second.quantity = 30;
        shop.view.top.push(second);
        shop.view.top_grid.update_rows(2);
        shop.double_click(12, 52);
        assert!(shop.count_target.is_some());
        shop.view.select(Side::Top, 1);
        shop.answer_text(Some("5"));
        assert_eq!((shop.view.bottom.len(), shop.view.bottom[0].grid.item_id, shop.view.bottom[0].quantity), (1, 58, 5));
        assert_eq!((shop.view.top[0].quantity, shop.view.top[1].quantity), (50, 25));
        shop.double_click(12, 52);
        shop.view.top_grid.selected = None;
        shop.answer_text(Some("5"));
        assert_eq!(shop.view.bottom.len(), 1);
    }

    #[test]
    fn clicking_the_other_grid_keeps_the_owner_selection_and_the_answer_still_lands() {
        let (mut shop, mut desktop, _) = setup(false);
        shop.view.bottom.push(shop.view.top[0].clone());
        shop.view.bottom_grid.update_rows(1);
        shop.double_click(12, 52);
        shop.mouse_down(12, 218, &mut desktop);
        assert_eq!((shop.view.top_grid.selected, shop.view.bottom_grid.selected), (Some(0), Some(0)));
        shop.answer_text(Some("5"));
        assert_eq!(shop.view.top[0].quantity, 45);
    }

    #[test]
    fn a_second_quantity_request_replaces_the_open_box() {
        use killer_platform::time::Instant;
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        let mut world = killer_world::World::default();
        ui.shop_event(&event(false, 50), Some(&tables()));
        let mut second = ui.keys.shop.view.top[0].clone();
        second.grid.item_id = 58;
        ui.keys.shop.view.top.push(second);
        ui.keys.shop.view.top_grid.update_rows(2);
        assert!(ui.keys.shop.double_click(12, 52));
        ui.tick_dialog(&mut world, Some(&tables()), Instant::now());
        ui.dialog.as_mut().unwrap().view.input.as_mut().unwrap().text = "9".into();
        ui.keys.shop.view.select(Side::Top, 1);
        assert!(ui.keys.shop.double_click(12 + 37, 52));
        ui.tick_dialog(&mut world, Some(&tables()), Instant::now());
        assert_ne!(ui.dialog.as_ref().unwrap().view.input.as_ref().unwrap().text, "9");
    }

    #[test]
    fn clicking_a_buy_item_through_the_game_ui_opens_the_count_dialog() {
        use killer_platform::time::Instant;
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        let mut world = killer_world::World::default();
        ui.shop_event(&event(true, 50), Some(&tables()));
        let rect = ui.desktop.window(WindowId::SHOP).unwrap().rect;
        let (sx, sy) = (rect.x + 12, rect.y + 52);
        assert!(ui.mouse_down_at(sx, sy, Instant::now(), &world));
        ui.mouse_up_in(&world);
        ui.tick_dialog(&mut world, Some(&tables()), Instant::now());
        assert!(ui.dialog.is_some(), "clicar no item da loja pelo app deve abrir a contagem");
        assert!(ui.dialog.as_ref().unwrap().view.input.is_some());
    }

    #[test]
    fn on_lbutton_up_10104d70_a_single_click_buys_one_of_a_non_stackable_item() {
        let (mut shop, mut desktop, mut commands) = setup(true);
        shop.view.top[0].consume_type = 0;
        click(&mut shop, &mut desktop, &mut commands, 12, 52);
        assert!(shop.take_dialog(None).is_none());
        assert_eq!(shop.view.bottom[0].quantity, 1);
    }

    #[test]
    fn buying_from_a_private_store_through_the_game_ui_shows_the_count_and_sends() {
        use killer_platform::time::Instant;
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        let mut world = killer_world::World::default();
        let list = l2_net::game::PrivateStoreListSell {
            object_id: 77, package_sale: false, adena: 500,
            items: vec![l2_net::game::StoreSellItem {
                type2: 5, object_id: 900, item_id: 57, count: 40, enchant: 0, custom_type2: 0,
                body_part: 0, price: 12, store_price: 12,
            }],
        };
        ui.private_shop_event(&WorldEvent::PrivateStoreListSell(list), Some(&tables()));
        let rect = ui.desktop.window(WindowId::PRIVATE_SHOP).unwrap().rect;
        let (sx, sy) = (rect.x + 12, rect.y + 52);
        assert!(ui.mouse_down_at(sx, sy, Instant::now(), &world));
        ui.mouse_up_in(&world);
        ui.tick_dialog(&mut world, Some(&tables()), Instant::now());
        assert!(ui.dialog.is_none(), "NCPrivateShopItemWnd::OnLButtonUp 0x100e8ab0: soltar sobre a própria grade não faz nada");
        assert!(ui.keys.private_shop.double_click(12, 52));
        ui.tick_dialog(&mut world, Some(&tables()), Instant::now());
        assert!(ui.dialog.is_some(), "loja particular: clicar no item deve abrir a contagem");
        assert!(ui.dialog.as_ref().unwrap().view.input.is_some());
        ui.key(killer_ui::chat::InputKey::Char('5'));
        ui.key(killer_ui::chat::InputKey::Enter);
        assert_eq!(ui.keys.private_shop.view.bottom[0].quantity, 5);
        let mut commands = Vec::new();
        ui.keys.private_shop.mouse_down(200, 380, &mut ui.desktop);
        ui.keys.private_shop.mouse_up(Some((WindowId::PRIVATE_SHOP, 200, 380)), &mut ui.desktop, &mut commands);
        assert!(matches!(commands.first(), Some(UiCommand::Net(NetCommand::PrivateStoreBuy { store_id: 77, .. }))));
    }

    #[test]
    fn selling_to_a_private_buy_store_keeps_the_packet_count_and_bounds_the_amount_by_the_wish_100e5fb0() {
        let mut shop = ShopWindow::new(|_| None);
        shop.id = WindowId::PRIVATE_SHOP;
        let mut desktop = Desktop::new((1000, 800));
        desktop.windows.push(Window::new(WindowId::PRIVATE_SHOP, Rect::new(0, 0, 256, 409), shop::grip()));
        let buy = |object_id, item_id, count, max_count| l2_net::game::StoreBuyItem {
            object_id, item_id, enchant: 0, count, store_price: 1, body_part: 0, type2: 5, price: 9, max_count,
        };
        let list = l2_net::game::PrivateStoreListBuy { object_id: 77, adena: 500, items: vec![buy(900, 57, 40, 25), buy(901, 57, 40, 0)] };
        assert!(shop.private_event(&WorldEvent::PrivateStoreListBuy(list), Some(&tables()), &mut desktop));
        assert_eq!(shop.view.top.iter().map(|entry| (entry.stock, entry.wanted)).collect::<Vec<_>>(), [(40, 25), (40, 0)]);
        let mut commands = Vec::new();
        let bottom = shop.view.bottom_grid.rect;
        shop.view.select(Side::Top, 0);
        shop.pressed = Some(ShopHit::Slot(Side::Top, 0));
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, bottom.x + 4, bottom.y + 4)), &mut desktop, &mut commands);
        let amount = shop.take_dialog(None).expect("NCPrivateBuyItemWnd::OnLButtonUp 0x100e5910 pede a quantidade");
        assert_eq!((amount.dialog_type, amount.input.as_ref().map(|input| input.max)), (0x2b, Some(40)));
        shop.answer_dialog(true, Some("26"), &mut desktop, &mut commands);
        assert_eq!(shop.messages, vec![1036]);
        assert!(shop.view.bottom.is_empty());
        shop.view.select(Side::Top, 0);
        shop.pressed = Some(ShopHit::Slot(Side::Top, 0));
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, bottom.x + 4, bottom.y + 4)), &mut desktop, &mut commands);
        assert!(shop.take_dialog(None).is_some());
        shop.answer_dialog(true, Some("25"), &mut desktop, &mut commands);
        assert_eq!((shop.view.bottom[0].quantity, shop.view.bottom[0].wanted), (25, 25));
        assert_eq!((shop.view.top[0].quantity, shop.view.top[0].wanted), (15, 0));
        shop.mouse_down(200, 380, &mut desktop);
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, 200, 380)), &mut desktop, &mut commands);
        assert!(commands.is_empty());
        assert_eq!(shop.take_dialog(None).map(|dialog| dialog.dialog_type), Some(0x2e), "preço 9 contra referência 1: caução 0x2e");
        shop.answer_dialog(true, None, &mut desktop, &mut commands);
        assert!(matches!(
            commands.as_slice(),
            [UiCommand::Net(NetCommand::PrivateStoreSell { store_id: 77, items })] if items.len() == 1 && items[0].count == 25 && items[0].price == 9
        ));
        assert!(!desktop.window(WindowId::PRIVATE_SHOP).unwrap().visible);
    }

    #[test]
    fn clicking_a_private_store_item_opens_the_count_dialog() {
        let mut shop = ShopWindow::new(|_| None);
        shop.id = WindowId::PRIVATE_SHOP;
        let mut desktop = Desktop::new((1000, 800));
        desktop.windows.push(Window::new(WindowId::PRIVATE_SHOP, Rect::new(0, 0, 256, 409), shop::grip()));
        let list = l2_net::game::PrivateStoreListSell {
            object_id: 77, package_sale: false, adena: 500,
            items: vec![l2_net::game::StoreSellItem {
                type2: 5, object_id: 900, item_id: 57, count: 40, enchant: 0, custom_type2: 0,
                body_part: 0, price: 12, store_price: 12,
            }],
        };
        assert!(shop.private_event(&WorldEvent::PrivateStoreListSell(list), Some(&tables()), &mut desktop));
        assert_eq!(shop.view.top.len(), 1);
        let mut commands = Vec::new();
        click(&mut shop, &mut desktop, &mut commands, 12, 52);
        assert!(shop.take_dialog(None).is_none(), "soltar o clique sobre a própria grade não abre nada (0x100e8ab0)");
        assert!(shop.double_click(12, 52));
        let dialog = shop.take_dialog(None).expect("o duplo clique (0x100e8e30) abre a contagem");
        assert_eq!(dialog.input.unwrap().max, 40);
        assert!(commands.is_empty());
    }

    fn manage_shop() -> (ShopWindow, Desktop) {
        let mut shop = ShopWindow::new(|_| None);
        shop.id = WindowId::PRIVATE_SHOP;
        let mut desktop = Desktop::new((1000, 800));
        desktop.windows.push(Window::new(WindowId::PRIVATE_SHOP, Rect::new(0, 0, 256, 409), shop::grip()));
        let item = l2_net::game::StoreSellItem {
            type2: 0, object_id: 900, item_id: 57, count: 40, enchant: 0, custom_type2: 0,
            body_part: 0, price: 1000, store_price: 0,
        };
        let event = WorldEvent::PrivateStoreManageListSell(l2_net::game::PrivateStoreManageListSell {
            object_id: 77, package_sale: false, adena: 500, available: vec![item], selling: vec![],
        });
        assert!(shop.private_manage_event(&event, Some(&tables()), &mut desktop));
        (shop, desktop)
    }

    #[test]
    fn managing_a_store_drag_to_the_sale_list_asks_the_price_then_the_quantity_100e8ab0_100e9280_100e94a0() {
        let (mut shop, mut desktop) = manage_shop();
        shop.view.select(Side::Top, 0);
        let mut commands = Vec::new();
        shop.pressed = Some(ShopHit::Slot(Side::Top, 0));
        let bottom = shop.view.bottom_grid.rect;
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, bottom.x + 4, bottom.y + 4)), &mut desktop, &mut commands);
        let price = shop.take_dialog(None).expect("o peer é a grade de baixo (tipo 0): pede o preço");
        assert_eq!(price.dialog_type, 0x25);
        shop.answer_dialog(true, Some("100"), &mut desktop, &mut commands);
        assert_eq!(shop.view.top[0].price, 100);
        assert!(shop.take_dialog(None).is_some_and(|dialog| dialog.dialog_type == 0x28), "100 não passa de 1000/5: pede confirmação");
        shop.answer_dialog(true, None, &mut desktop, &mut commands);
        let quantity = shop.take_dialog(None).expect("a confirmação leva à quantidade");
        assert_eq!((quantity.dialog_type, quantity.input.as_ref().map(|input| input.max)), (0x26, Some(40)));
        shop.answer_dialog(true, Some("41"), &mut desktop, &mut commands);
        assert_eq!(shop.messages, vec![0x40c]);
        assert!(shop.view.bottom.is_empty());
        shop.view.select(Side::Top, 0);
        shop.pressed = Some(ShopHit::Slot(Side::Top, 0));
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, bottom.x + 4, bottom.y + 4)), &mut desktop, &mut commands);
        assert!(shop.take_dialog(None).is_some());
        shop.answer_dialog(true, Some("1200"), &mut desktop, &mut commands);
        assert!(shop.take_dialog(None).is_some_and(|dialog| dialog.dialog_type == 0x26));
        shop.answer_dialog(true, Some("7"), &mut desktop, &mut commands);
        assert_eq!((shop.view.bottom.len(), shop.view.bottom[0].quantity), (1, 7));
    }

    #[test]
    fn a_cancelled_private_dialog_changes_nothing() {
        let (mut shop, mut desktop) = manage_shop();
        shop.view.select(Side::Top, 0);
        shop.pressed = Some(ShopHit::Slot(Side::Top, 0));
        let bottom = shop.view.bottom_grid.rect;
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, bottom.x + 4, bottom.y + 4)), &mut desktop, &mut Vec::new());
        assert!(shop.take_dialog(None).is_some());
        shop.answer_dialog(false, None, &mut desktop, &mut Vec::new());
        assert!(shop.view.bottom.is_empty() && shop.messages.is_empty() && shop.take_dialog(None).is_none());
    }

    #[test]
    fn private_manage_lists_100e97e0_keep_available_and_active_items_separate() {
        let mut shop = ShopWindow::new(|_| None);
        shop.id = WindowId::PRIVATE_SHOP;
        let mut desktop = Desktop::new((1000, 800));
        desktop.windows.push(Window::new(WindowId::PRIVATE_SHOP, Rect::new(0, 0, 256, 409), shop::grip()));
        let item = l2_net::game::StoreSellItem {
            type2: 0, object_id: 900, item_id: 57, count: 12, enchant: 3, custom_type2: 4,
            body_part: 0, price: 25, store_price: 25,
        };
        let event = WorldEvent::PrivateStoreManageListSell(l2_net::game::PrivateStoreManageListSell {
            object_id: 77, package_sale: false, adena: 500, available: vec![item], selling: vec![item],
        });
        shop.private_manage_event(&event, Some(&tables()), &mut desktop);
        assert_eq!(shop.private_store_id, Some(77));
        assert_eq!(shop.view.top[0].grid.object_id, 900);
        assert_eq!(shop.view.bottom[0].price, 25);
        assert!(desktop.window(WindowId::PRIVATE_SHOP).unwrap().visible);
    }

    #[test]
    fn private_manage_controls_100e69a0_start_and_abort_100e6bf0_quit_the_store() {
        let mut shop = ShopWindow::new(|_| None);
        shop.id = WindowId::PRIVATE_SHOP;
        let mut desktop = Desktop::new((1000, 800));
        desktop.windows.push(Window::new(WindowId::PRIVATE_SHOP, Rect::new(0, 0, 256, 409), shop::grip()));
        let item = l2_net::game::StoreSellItem {
            type2: 0, object_id: 900, item_id: 57, count: 12, enchant: 3, custom_type2: 4,
            body_part: 0, price: 25, store_price: 25,
        };
        let event = WorldEvent::PrivateStoreManageListSell(l2_net::game::PrivateStoreManageListSell {
            object_id: 77, package_sale: false, adena: 500, available: vec![], selling: vec![item],
        });
        shop.private_manage_event(&event, Some(&tables()), &mut desktop);
        let mut commands = Vec::new();
        shop.mouse_down(200, 380, &mut desktop);
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, 200, 380)), &mut desktop, &mut commands);
        assert_eq!(commands, vec![UiCommand::Net(NetCommand::SetPrivateStoreListSell {
            package_sale: false,
            offers: vec![l2_net::game::StoreOffer { object_id: 900, count: 12, price: 25 }],
        })]);
        shop.private_manage_event(&event, Some(&tables()), &mut desktop);
        commands.clear();
        shop.mouse_down(20, 380, &mut desktop);
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, 20, 380)), &mut desktop, &mut commands);
        assert_eq!(commands, vec![UiCommand::Net(NetCommand::PrivateStoreQuitSell)]);
        assert_eq!(shop.private_quit(), None);
        shop.private_manage_event(&event, Some(&tables()), &mut desktop);
        assert_eq!(shop.private_quit(), Some(NetCommand::PrivateStoreQuitSell));
        shop.view.mode = ShopMode::Buy;
        assert_eq!(shop.private_quit(), Some(NetCommand::PrivateStoreQuitBuy));
    }

    #[test]
    fn private_buy_window_100e3410_paints_no_counts_and_covers_the_items_the_visitor_lacks() {
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        let sell = WorldEvent::PrivateStoreListSell(l2_net::game::PrivateStoreListSell { object_id: 77, package_sale: false, adena: 500, items: vec![] });
        ui.private_shop_event(&sell, Some(&tables()));
        assert_eq!((ui.keys.private_shop.private_store_id, ui.keys.private_shop.private_manage), (Some(77), false));
        assert!(ui.keys.private_shop.view.top_grid.show_count);
        let item = |object_id, count| l2_net::game::StoreBuyItem {
            object_id, item_id: 57, enchant: 0, count, store_price: 1, body_part: 0, type2: 5, price: 9, max_count: 3,
        };
        let buy = WorldEvent::PrivateStoreListBuy(l2_net::game::PrivateStoreListBuy { object_id: 78, adena: 500, items: vec![item(900, 0), item(901, 4)] });
        ui.private_shop_event(&buy, Some(&tables()));
        let view = &ui.keys.private_shop.view;
        assert!(!view.top_grid.show_count && !view.bottom_grid.show_count);
        let mut list = killer_ui::draw::DrawList::default();
        view.private_paint(&mut list, 0, 0, &ui.small, 1.0);
        assert_eq!(list.quads.iter().filter(|quad| quad.texture == shop::PRIVATE_BUY_COVER).count(), 1);
        let manage = WorldEvent::PrivateStoreManageListBuy(l2_net::game::PrivateStoreManageListBuy { object_id: 79, adena: 5, available: vec![item(900, 0)], buying: vec![] });
        ui.private_shop_event(&manage, Some(&tables()));
        let mut list = killer_ui::draw::DrawList::default();
        ui.keys.private_shop.view.private_paint(&mut list, 0, 0, &ui.small, 1.0);
        assert!(list.quads.iter().all(|quad| quad.texture != shop::PRIVATE_BUY_COVER));
    }

    #[test]
    fn private_message_button_100e7c20_opens_dialog_27_with_the_current_message_and_sends_it() {
        let (mut shop, mut desktop) = manage_shop();
        let mut commands = Vec::new();
        shop.private_event(&WorldEvent::PrivateStoreMsgSell(l2_net::game::PrivateStoreMsg { object_id: 77, message: "vendo".into() }), None, &mut desktop);
        shop.mouse_down(100, 380, &mut desktop);
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, 100, 380)), &mut desktop, &mut commands);
        assert!(commands.is_empty());
        let dialog = shop.take_dialog(None).expect("o botão 0xf9 abre a caixa");
        assert_eq!(dialog.dialog_type, 0x27);
        assert_eq!(dialog.input.as_ref().map(|input| (input.free_text, input.text.as_str())), Some((true, "vendo")));
        shop.answer_dialog(true, Some("barato"), &mut desktop, &mut commands);
        assert_eq!(commands, vec![UiCommand::Net(NetCommand::SetPrivateStoreMsgSell { message: "barato".into() })]);
        commands.clear();
        shop.mouse_down(100, 380, &mut desktop);
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, 100, 380)), &mut desktop, &mut commands);
        assert!(shop.take_dialog(None).is_some());
        shop.answer_dialog(true, Some(""), &mut desktop, &mut commands);
        shop.answer_dialog(false, Some("x"), &mut desktop, &mut commands);
        assert!(commands.is_empty(), "OK 0x10069a40 só envia texto não vazio");
    }

    #[test]
    fn buy_store_message_button_100e4920_opens_dialog_2c_and_sends_the_buy_message() {
        let mut shop = ShopWindow::new(|_| None);
        shop.id = WindowId::PRIVATE_SHOP;
        let mut desktop = Desktop::new((1000, 800));
        desktop.windows.push(Window::new(WindowId::PRIVATE_SHOP, Rect::new(0, 0, 256, 409), shop::grip()));
        let event = WorldEvent::PrivateStoreManageListBuy(l2_net::game::PrivateStoreManageListBuy { object_id: 79, adena: 5, available: vec![], buying: vec![] });
        assert!(shop.private_manage_event(&event, Some(&tables()), &mut desktop));
        let mut commands = Vec::new();
        shop.mouse_down(100, 380, &mut desktop);
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, 100, 380)), &mut desktop, &mut commands);
        assert_eq!(shop.take_dialog(None).map(|dialog| dialog.dialog_type), Some(0x2c));
        shop.answer_dialog(true, Some("compro"), &mut desktop, &mut commands);
        assert_eq!(commands, vec![UiCommand::Net(NetCommand::SetPrivateStoreMsgBuy { message: "compro".into() })]);
    }

    #[test]
    fn package_sale_check_box_1f0_toggles_on_mouse_down_and_goes_in_the_list_request() {
        let (mut shop, mut desktop) = manage_shop();
        assert!(!shop.view.package_sale);
        shop.mouse_down(20, 355, &mut desktop);
        assert!(shop.view.package_sale);
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, 20, 355)), &mut desktop, &mut Vec::new());
        let mut commands = Vec::new();
        shop.mouse_down(200, 380, &mut desktop);
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, 200, 380)), &mut desktop, &mut commands);
        assert_eq!(commands, vec![UiCommand::Net(NetCommand::SetPrivateStoreListSell { package_sale: true, offers: vec![] })]);
    }

    fn visit_sell_shop(price: i32, reference: i32, package_sale: bool) -> (ShopWindow, Desktop) {
        let mut shop = ShopWindow::new(|_| None);
        shop.id = WindowId::PRIVATE_SHOP;
        let mut desktop = Desktop::new((1000, 800));
        desktop.windows.push(Window::new(WindowId::PRIVATE_SHOP, Rect::new(0, 0, 256, 409), shop::grip()));
        let items = [900, 901].map(|object_id| l2_net::game::StoreSellItem {
            type2: 5, object_id, item_id: 57, count: 40, enchant: 0, custom_type2: 0, body_part: 0, price, store_price: reference,
        });
        let list = l2_net::game::PrivateStoreListSell { object_id: 77, package_sale, adena: 500, items: items.to_vec() };
        assert!(shop.private_event(&WorldEvent::PrivateStoreListSell(list), Some(&tables()), &mut desktop));
        (shop, desktop)
    }

    #[test]
    fn buying_at_an_unusual_price_asks_dialog_29_first_and_cancel_closes_the_window_100e76d0() {
        let (mut shop, mut desktop) = visit_sell_shop(10_000, 1000, false);
        assert_eq!((shop.view.top[0].price, shop.view.top[0].reference_price), (10_000, 1000));
        shop.view.select(Side::Top, 0);
        let mut commands = Vec::new();
        shop.private_arrow(Side::Top, true);
        assert_eq!(shop.take_dialog(None).map(|dialog| dialog.dialog_type), Some(0x26));
        shop.answer_dialog(true, Some("3"), &mut desktop, &mut commands);
        shop.mouse_down(200, 380, &mut desktop);
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, 200, 380)), &mut desktop, &mut commands);
        assert!(commands.is_empty());
        let caution = shop.take_dialog(None).expect("preço 10 vezes a referência: caução 0x29");
        assert_eq!((caution.dialog_type, caution.input.is_none()), (0x29, true));
        shop.answer_dialog(true, None, &mut desktop, &mut commands);
        assert!(matches!(
            commands.as_slice(),
            [UiCommand::Net(NetCommand::PrivateStoreBuy { store_id: 77, orders })] if orders == &[l2_net::game::StoreBuyOrder { object_id: 900, count: 3, price: 10_000 }]
        ));
        assert!(!desktop.window(WindowId::PRIVATE_SHOP).unwrap().visible);
        let (mut shop, mut desktop) = visit_sell_shop(10_000, 1000, false);
        shop.view.select(Side::Top, 0);
        shop.private_arrow(Side::Top, true);
        shop.answer_dialog(true, Some("1"), &mut desktop, &mut commands);
        shop.mouse_down(200, 380, &mut desktop);
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, 200, 380)), &mut desktop, &mut commands);
        assert!(shop.take_dialog(None).is_some());
        shop.answer_dialog(false, None, &mut desktop, &mut commands);
        assert!(!desktop.window(WindowId::PRIVATE_SHOP).unwrap().visible, "OnCancel 0x29 esconde a janela dona");
        assert_eq!(commands.len(), 1);
    }

    #[test]
    fn a_package_sale_arrow_moves_the_whole_list_and_the_title_names_it_100e7600() {
        let (mut shop, mut desktop) = visit_sell_shop(10, 10, true);
        assert!(shop.view.package_sale);
        assert_eq!(shop.view.private_title(), "sysstring 498 - sysstring 1198");
        shop.private_arrow(Side::Top, true);
        assert!(shop.view.top.is_empty());
        assert_eq!(shop.view.bottom.len(), 2);
        shop.private_arrow(Side::Bottom, false);
        assert_eq!((shop.view.top.len(), shop.view.bottom.len()), (2, 0));
        let mut commands = Vec::new();
        shop.mouse_down(200, 380, &mut desktop);
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, 200, 380)), &mut desktop, &mut commands);
        assert!(matches!(commands.as_slice(), [UiCommand::Net(NetCommand::PrivateStoreBuy { orders, .. })] if orders.is_empty()));
    }

    #[test]
    fn the_buy_manage_window_asks_price_then_caution_then_amount_and_sends_the_wish_list() {
        let mut shop = ShopWindow::new(|_| None);
        shop.id = WindowId::PRIVATE_SHOP;
        let mut desktop = Desktop::new((1000, 800));
        desktop.windows.push(Window::new(WindowId::PRIVATE_SHOP, Rect::new(0, 0, 256, 409), shop::grip()));
        let item = l2_net::game::StoreBuyItem {
            object_id: 0, item_id: 57, enchant: 0, count: 100, store_price: 1000, body_part: 0, type2: 5, price: 0, max_count: 0,
        };
        let event = WorldEvent::PrivateStoreManageListBuy(l2_net::game::PrivateStoreManageListBuy {
            object_id: 77, adena: 5, available: vec![item], buying: vec![],
        });
        assert!(shop.private_manage_event(&event, Some(&tables()), &mut desktop));
        shop.private_event(&WorldEvent::StorageMaxCount(l2_net::game::StorageMaxCount { values: [0, 0, 0, 20, 30, 0, 0] }), None, &mut desktop);
        assert!(shop.private_manage_event(&event, Some(&tables()), &mut desktop));
        assert_eq!(shop.view.item_limit, 30);
        let mut commands = Vec::new();
        shop.view.select(Side::Top, 0);
        shop.private_arrow(Side::Top, true);
        assert_eq!(shop.take_dialog(None).map(|dialog| dialog.dialog_type), Some(0x2a));
        shop.answer_dialog(true, Some("100"), &mut desktop, &mut commands);
        assert_eq!(shop.take_dialog(None).map(|dialog| dialog.dialog_type), Some(0x2d));
        shop.answer_dialog(true, None, &mut desktop, &mut commands);
        let amount = shop.take_dialog(None).expect("a caução leva à quantidade");
        assert_eq!((amount.dialog_type, amount.input.as_ref().map(|input| input.max)), (0x2b, Some(100)));
        shop.answer_dialog(true, Some("60"), &mut desktop, &mut commands);
        assert_eq!((shop.view.bottom.len(), shop.view.bottom[0].quantity, shop.view.bottom[0].price), (1, 60, 100));
        assert_eq!(shop.view.top.len(), 1);
        shop.mouse_down(200, 380, &mut desktop);
        shop.mouse_up(Some((WindowId::PRIVATE_SHOP, 200, 380)), &mut desktop, &mut commands);
        assert_eq!(commands, vec![UiCommand::Net(NetCommand::SetPrivateStoreListBuy {
            offers: vec![l2_net::game::BuyOffer { item_id: 57, enchant: 0, custom_type2: 5, count: 60, price: 100 }],
        })]);
    }

    #[test]
    fn a_refused_amount_shows_the_notice_559() {
        let (mut shop, mut desktop) = manage_shop();
        shop.view.top[0].price = 2_100_000_000;
        shop.view.select(Side::Top, 0);
        let mut commands = Vec::new();
        shop.private_flow = Some((Side::Top, 0, 0x26));
        shop.answer_dialog(true, Some("1"), &mut desktop, &mut commands);
        let notice = shop.take_dialog(None).expect("CheckTotal falhou: aviso 0x559");
        assert_eq!(notice.dialog_type, 0x1f);
    }

    fn click(shop: &mut ShopWindow, desktop: &mut Desktop, commands: &mut Vec<UiCommand>, x: i32, y: i32) {
        shop.mouse_down(x, y, desktop);
        shop.mouse_up(Some((shop.id, x, y)), desktop, commands);
    }

    #[test]
    fn on_buy_list_10066e70_and_on_sell_list_10067060_populate_focus_and_close_html() {
        for buy in [true, false] {
            let (mut shop, mut desktop, _) = setup(buy);
            assert_eq!(shop.view.top.len(), 1);
            assert_eq!(shop.view.top[0].consume_type, 3);
            assert_eq!(shop.view.top[0].grid.enchant, 3);
            assert_eq!((shop.view.list_id, shop.view.adena), (7, 500));
            assert!(!desktop.window(WindowId::NPC_HTML).unwrap().visible);
            assert_eq!(desktop.window(WindowId::INVENTORY).unwrap().visible, buy);
            assert_eq!(desktop.windows.last().unwrap().id, WindowId::SHOP);
            shop.world_event(&event(buy, 3), Some(&DatTables::default()), &mut desktop);
            assert!(shop.view.top.is_empty());
        }
    }

    fn warehouse_event(deposit: bool, warehouse_type: i16, count: i32) -> WorldEvent {
        let WorldEvent::BuyList(list) = event(true, count) else { unreachable!() };
        let list = l2_net::game::WarehouseList {
            warehouse_type, adena: 500,
            items: list.items.into_iter().map(|p| l2_net::game::WarehouseItem { item: p.item, warehouse_id: 1234 }).collect(),
        };
        if deposit { WorldEvent::WarehouseDepositList(list) } else { WorldEvent::WarehouseWithdrawList(list) }
    }

    fn delivery_event(count: i32) -> WorldEvent {
        let WorldEvent::WarehouseDepositList(list) = warehouse_event(true, 1, count) else { unreachable!() };
        WorldEvent::PackageSendableList(l2_net::game::PackageSendableList { target_id: 77, adena: list.adena, items: list.items })
    }

    #[test]
    fn delivery_selects_target_counts_items_sends_final_identifier_and_clears() {
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        let mut world = killer_world::World::default();
        ui.shop_event(&WorldEvent::PackageToList(l2_net::game::PackageToList {
            targets: (0..12).map(|i| l2_net::game::PackageTarget { object_id: 70 + i, name: format!("character{i}") }).collect(),
        }), Some(&tables()));
        assert_eq!(ui.desktop.windows.last().unwrap().id, WindowId::SELECT_DELIVERY);
        assert_eq!(ui.keys.select_delivery.targets.len(), 8);
        let select = ui.desktop.window(WindowId::SELECT_DELIVERY).unwrap().rect;
        ui.keys.select_delivery.combo.open_list();
        assert_eq!(ui.window_under(select.x + 40, select.y + 210), Some((WindowId::SELECT_DELIVERY, 40, 210)));
        ui.keys.mouse_down(WindowId::SELECT_DELIVERY, 40, 210, &mut ui.desktop);
        assert_eq!(ui.keys.select_delivery.combo.selected, 7);
        ui.keys.mouse_down(WindowId::SELECT_DELIVERY, 60, 175, &mut ui.desktop);
        ui.keys.mouse_up(Some((WindowId::SELECT_DELIVERY, 60, 175)), &mut ui.desktop, &mut ui.shortcut, &world);
        assert_eq!(ui.take_commands(), vec![UiCommand::Net(NetCommand::PackageSendableItemList { target_id: 77 })]);
        assert!(!ui.desktop.window(WindowId::SELECT_DELIVERY).unwrap().visible);
        assert!(ui.keys.select_delivery.targets.is_empty());
        ui.desktop.window_mut(WindowId::INVENTORY).unwrap().visible = true;
        ui.desktop.window_mut(WindowId::NPC_HTML).unwrap().visible = true;
        ui.shop_event(&delivery_event(50), Some(&tables()));
        assert_eq!(ui.desktop.windows.last().unwrap().id, WindowId::DELIVERY);
        assert!(ui.desktop.window(WindowId::INVENTORY).unwrap().visible);
        assert!(ui.desktop.window(WindowId::NPC_HTML).unwrap().visible);
        assert!(ui.keys.double_click(WindowId::DELIVERY, 12, 52, &world));
        ui.tick_dialog(&mut world, Some(&tables()), killer_platform::time::Instant::now());
        let dialog = ui.dialog.as_ref().unwrap();
        assert_eq!(dialog.answer, crate::ui2d::DialogAnswer::Delivery);
        assert_eq!(dialog.view.dialog_type, 3);
        ui.key(killer_ui::chat::InputKey::Char('7'));
        ui.key(killer_ui::chat::InputKey::Enter);
        assert_eq!(ui.keys.delivery.view.bottom[0].quantity, 7);
        assert_eq!(ui.keys.delivery.view.top[0].quantity, 43);
        assert_eq!(ui.keys.delivery.view.fee(), 1000);
        ui.keys.mouse_down(WindowId::DELIVERY, 60, 380, &mut ui.desktop);
        ui.keys.mouse_up(Some((WindowId::DELIVERY, 60, 380)), &mut ui.desktop, &mut ui.shortcut, &world);
        assert_eq!(ui.take_commands(), vec![UiCommand::Net(NetCommand::PackageSend { target_id: 77, orders: vec![(1234, 7)] })]);
        assert!(!ui.desktop.window(WindowId::DELIVERY).unwrap().visible);
        assert!(ui.keys.delivery.view.top.is_empty());
        assert!(ui.keys.delivery.view.bottom.is_empty());
        assert_eq!(ui.keys.delivery.view.list_id, -1);
    }

    #[test]
    fn delivery_selector_and_preview_are_painted_by_the_live_window_dispatch() {
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        ui.shop_event(&delivery_event(50), Some(&tables()));
        ui.shop_event(&WorldEvent::ShopPreviewList(l2_net::game::ShopPreviewList {
            header: 0, money: 500, list_id: 1, items: vec![l2_net::game::PreviewItem { item_id: 57, type2: 5, body_part: 0, price: 10 }],
        }), Some(&tables()));
        for (id, background) in [(WindowId::DELIVERY, shop::STORE_BACK), (WindowId::SELECT_DELIVERY, killer_ui::delivery::SELECT_BACK), (WindowId::SHOP_PREVIEW, shop::STORE_BACK)] {
            let mut draw = killer_ui::draw::DrawList::default();
            ui.paint_window(&mut draw, ui.desktop.window(id).unwrap(), &killer_world::World::default(), &[], None, &crate::ui2d::WorldViews::default());
            assert!(draw.quads.iter().any(|quad| quad.texture == background), "{id:?}");
            assert!(draw.quads.iter().any(|quad| quad.texture == killer_ui::frame::FRAME_CLOSE_BTN), "{id:?}");
        }
    }

    #[test]
    fn delivery_empty_packets_are_noops_and_replacement_cancels_quantity_dialog() {
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        let mut world = killer_world::World::default();
        ui.shop_event(&delivery_event(50), Some(&tables()));
        ui.keys.double_click(WindowId::DELIVERY, 12, 52, &world);
        ui.tick_dialog(&mut world, Some(&tables()), killer_platform::time::Instant::now());
        ui.shop_event(&WorldEvent::PackageSendableList(l2_net::game::PackageSendableList { target_id: 0, adena: 0, items: vec![] }), Some(&tables()));
        assert!(ui.dialog.is_some());
        assert_eq!(ui.keys.delivery.view.list_id, 77);
        ui.shop_event(&delivery_event(20), Some(&tables()));
        assert!(ui.dialog.is_none());
        assert_eq!(ui.keys.delivery.view.top[0].quantity, 20);
        ui.keys.mouse_down(WindowId::DELIVERY, 60, 380, &mut ui.desktop);
        ui.keys.mouse_up(Some((WindowId::DELIVERY, 60, 380)), &mut ui.desktop, &mut ui.shortcut, &world);
        assert_eq!(ui.take_commands(), vec![UiCommand::Net(NetCommand::PackageSend { target_id: 77, orders: vec![] })]);
    }

    #[test]
    fn warehouse_10067150_10067210_and_10118680_are_wired_to_the_app() {
        for deposit in [true, false] {
            let mut ui = crate::ui2d::tests::game_ui((1000, 800));
            let mut world = killer_world::World::default();
            ui.desktop.window_mut(WindowId::INVENTORY).unwrap().visible = true;
            ui.shop_event(&WorldEvent::StorageMaxCount(l2_net::game::StorageMaxCount { values: [999, 120, 0, 0, 0, 0, 0] }), Some(&tables()));
            ui.shop_event(&warehouse_event(deposit, 1, 50), Some(&tables()));
            assert_eq!(ui.desktop.windows.last().unwrap().id, WindowId::WAREHOUSE);
            assert_eq!(ui.desktop.window(WindowId::INVENTORY).unwrap().visible, !deposit);
            assert_eq!(ui.keys.warehouse.view.warehouse_limit, 120);
            assert_eq!(ui.keys.warehouse.view.top[0].warehouse_id, 1234);
            assert!(ui.keys.double_click(WindowId::WAREHOUSE, 12, 52, &world));
            ui.tick_dialog(&mut world, Some(&tables()), killer_platform::time::Instant::now());
            assert_eq!(ui.dialog.as_ref().unwrap().view.dialog_type, 3);
            assert_eq!(ui.dialog.as_ref().unwrap().answer, crate::ui2d::DialogAnswer::Warehouse);
            ui.key(killer_ui::chat::InputKey::Char('7'));
            ui.key(killer_ui::chat::InputKey::Enter);
            assert_eq!(ui.keys.warehouse.view.bottom[0].quantity, 7);
            assert_eq!(ui.keys.warehouse.view.top[0].quantity, 43);
            assert_eq!(ui.keys.warehouse.view.fee(), if deposit { 30 } else { 0 });
            assert!(ui.keys.shop.view.bottom.is_empty());
            ui.keys.mouse_down(WindowId::WAREHOUSE, 60, 380, &mut ui.desktop);
            let mut commands = vec![];
            ui.keys.warehouse.mouse_up(Some((WindowId::WAREHOUSE, 60, 380)), &mut ui.desktop, &mut commands);
            let expected = if deposit { NetCommand::WarehouseDeposit { orders: vec![(1234, 7)] } }
                else { NetCommand::WarehouseWithdraw { orders: vec![(1234, 7)] } };
            assert_eq!(commands, vec![UiCommand::Net(expected)]);
            assert!(!ui.desktop.window(WindowId::WAREHOUSE).unwrap().visible);
        }
    }

    #[test]
    fn warehouse_101195e0_clamps_counts_replacement_cancels_and_types_keep_native_limits() {
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        let mut world = killer_world::World::default();
        for warehouse_type in [1, 2, 3, 4] {
            ui.shop_event(&warehouse_event(false, warehouse_type, 50), Some(&tables()));
            ui.keys.warehouse.double_click(12, 52);
            ui.keys.warehouse.answer_text(Some("99"));
            assert_eq!(ui.keys.warehouse.view.bottom[0].quantity, 50);
            assert!(ui.keys.warehouse.view.top.is_empty());
            assert_eq!(ui.keys.warehouse.view.counter().is_none(), warehouse_type == 4);
            if matches!(warehouse_type, 2 | 3) {
                assert_eq!(ui.keys.warehouse.view.counter().unwrap().0, "(0/200)");
            }
            ui.keys.warehouse.double_click(12, 218);
            ui.tick_dialog(&mut world, Some(&tables()), killer_platform::time::Instant::now());
            assert!(ui.dialog.is_some());
            ui.shop_event(&warehouse_event(true, warehouse_type, 9), Some(&tables()));
            assert!(ui.dialog.is_none());
            ui.keys.warehouse.answer_text(Some("3"));
            assert!(ui.keys.warehouse.view.bottom.is_empty());
            let mut commands = vec![];
            click(&mut ui.keys.warehouse, &mut ui.desktop, &mut commands, 140, 380);
            assert!(commands.is_empty());
            assert!(ui.keys.warehouse.view.top.is_empty());
        }
    }

    #[test]
    fn warehouse_10118d20_caps_valid_catalog_entries_at_200() {
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        let WorldEvent::WarehouseDepositList(mut list) = warehouse_event(true, 1, 3) else { unreachable!() };
        let item = list.items[0];
        list.items = (0..205).map(|index| l2_net::game::WarehouseItem {
            warehouse_id: index, ..item
        }).collect();
        list.items[0].item.item_id = -1;
        ui.shop_event(&WorldEvent::WarehouseDepositList(list), Some(&tables()));
        assert_eq!(ui.keys.warehouse.view.top.len(), 200);
        assert_eq!(ui.keys.warehouse.view.top[0].warehouse_id, 1);
        assert_eq!(ui.keys.warehouse.view.top[199].warehouse_id, 200);
    }

    #[test]
    fn slot128_10103600_sends_the_selected_purchase_or_sale_and_hides() {
        for buy in [true, false] {
            let (mut shop, mut desktop, mut commands) = setup(buy);
            click(&mut shop, &mut desktop, &mut commands, 12, 52);
            click(&mut shop, &mut desktop, &mut commands, 133, 198);
            let dialog = shop.take_dialog(None).unwrap();
            assert_eq!(dialog.dialog_type, 2);
            assert_eq!(dialog.input.unwrap().max, 50);
            shop.answer_text(Some("12"));
            assert!(commands.is_empty());
            assert_eq!(shop.view.total(), 144);
            click(&mut shop, &mut desktop, &mut commands, 60, 380);
            let expected = if buy {
                NetCommand::BuyItems { list_id: 7, orders: vec![(57, 12)] }
            } else {
                NetCommand::SellItems { list_id: 7, orders: vec![l2_net::game::ItemOrder { object_id: 900, item_id: 57, count: 12 }] }
            };
            assert_eq!(commands, vec![UiCommand::Net(expected)]);
            assert!(!desktop.window(WindowId::SHOP).unwrap().visible);
        }
    }

    #[test]
    fn slot128_10103600_rejects_overstock_without_sending_and_cancel_clears() {
        let (mut shop, mut desktop, mut commands) = setup(true);
        assert!(shop.double_click(12, 52));
        shop.answer_text(Some("51"));
        click(&mut shop, &mut desktop, &mut commands, 60, 380);
        assert!(commands.is_empty());
        assert!(desktop.window(WindowId::SHOP).unwrap().visible);
        let dialog = shop.take_dialog(None).unwrap();
        assert_eq!((dialog.message.as_str(), dialog.dialog_type), ("systemmsg 1338", 31));
        click(&mut shop, &mut desktop, &mut commands, 140, 380);
        assert!(shop.view.top.is_empty() && shop.view.bottom.is_empty());
        assert!(!desktop.window(WindowId::SHOP).unwrap().visible);
    }

    #[test]
    fn slot148_10105290_clamps_sale_counts_and_does_not_apply_a_stale_reply() {
        let (mut shop, mut desktop, _) = setup(false);
        shop.double_click(12, 52);
        shop.answer_text(Some("80"));
        assert_eq!(shop.view.bottom[0].quantity, 50);
        shop.double_click(12, 218);
        shop.world_event(&event(false, 4), Some(&tables()), &mut desktop);
        shop.answer_text(Some("2"));
        assert!(shop.view.bottom.is_empty());
        assert_eq!(shop.view.top[0].quantity, 4);
    }

    #[test]
    fn on_ok_10009750_uses_separate_asset_and_adena_limits() {
        for (consume, text, expected) in [(2, "1000001", 1338), (3, "2000000001", 1369)] {
            let (mut shop, _, _) = setup(true);
            shop.view.top[0].consume_type = consume;
            shop.double_click(12, 52);
            shop.answer_text(Some(text));
            assert!(shop.view.bottom.is_empty());
            assert_eq!(shop.take_dialog(None).unwrap().message, format!("systemmsg {expected}"));
        }
        let (mut shop, _, _) = setup(true);
        shop.view.top[0].consume_type = 1;
        shop.double_click(12, 52);
        shop.answer_text(Some("2000000001"));
        assert_eq!(shop.view.bottom[0].quantity, 2_000_000_001);
    }

    #[test]
    fn releasing_away_does_not_confirm_and_dragging_between_grids_opens_count() {
        let (mut shop, mut desktop, mut commands) = setup(true);
        shop.mouse_down(60, 380, &mut desktop);
        shop.mouse_up(None, &mut desktop, &mut commands);
        assert!(commands.is_empty());
        shop.view.top[0].grid.icon = "icon.test".into();
        shop.mouse_down(12, 52, &mut desktop);
        assert!(shop.drag_icon().is_none());
        shop.cursor_hover();
        assert_eq!(shop.drag_icon().as_deref(), Some("icon.test"));
        shop.mouse_up(Some((WindowId::SHOP, 12, 218)), &mut desktop, &mut commands);
        assert!(shop.drag_icon().is_none());
        assert!(shop.take_dialog(None).unwrap().input.is_some());
        shop.answer_text(None);
        assert!(shop.view.bottom.is_empty());
    }

    #[test]
    fn gara_dialog_10008e90_edits_without_closing_and_enter_10007e30_accepts_input() {
        use killer_ui::chat::InputKey;
        use crate::ui2d::KeyAction;
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        let mut world = killer_world::World::default();
        let now = killer_platform::time::Instant::now();
        ui.shop_event(&event(true, 50), Some(&tables()));
        ui.keys.shop.double_click(12, 52);
        ui.tick_dialog(&mut world, Some(&tables()), now);
        let rect = ui.desktop.window(WindowId::DIALOG).unwrap().rect;
        assert_eq!(rect.w, 364);
        let key = killer_ui::dialog::keypad_rect(3);
        ui.mouse_down(rect.x + key.x + 2, rect.y + key.y + 2);
        assert_eq!(ui.dialog_released(), Some(KeyAction::Consumed));
        assert_eq!(ui.dialog.as_ref().unwrap().view.input.as_ref().unwrap().text, "4");
        assert!(!ui.hook_key(killer_ui::keys::WM_KEYDOWN, 0x31, 0, &world));
        assert_eq!(ui.key(InputKey::Char('2')), KeyAction::Consumed);
        assert_eq!(ui.key(InputKey::Backspace), KeyAction::Consumed);
        assert!(ui.tick_dialog(&mut world, Some(&tables()), now + killer_platform::time::Duration::from_secs(60)).is_none());
        assert!(ui.dialog.is_some());
        assert_eq!(ui.key(InputKey::Enter), KeyAction::Consumed);
        assert!(ui.dialog.is_none());
        assert_eq!(ui.keys.shop.view.bottom[0].quantity, 4);
        assert!(ui.take_commands().is_empty());
    }

    #[test]
    fn a_replacement_list_invalidates_the_visible_quantity_dialog() {
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        let mut world = killer_world::World::default();
        ui.shop_event(&event(true, 50), Some(&tables()));
        ui.keys.shop.double_click(12, 52);
        ui.tick_dialog(&mut world, Some(&tables()), killer_platform::time::Instant::now());
        assert!(ui.dialog.is_some());
        ui.shop_event(&event(false, 20), Some(&tables()));
        assert!(ui.dialog.is_none());
        assert!(ui.keys.shop.view.bottom.is_empty());
        assert_eq!(ui.keys.shop.view.mode, ShopMode::Sell);
    }

    #[test]
    #[ignore]
    fn delivery_all_client_items_scroll_transfer_confirm_and_capture() {
        use killer_assets::client::Client;
        use killer_assets::textures::{BankSource, TextureBank, write_png};
        use killer_render::camera::Camera;
        use killer_render::environment::Environment;
        use killer_render::headless::Headless;
        use killer_ui::draw::DrawList;
        use killer_ui::font::{BitmapFont, Glyphs};
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let font = BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap());
        let ids: std::collections::BTreeSet<_> = tables.weapongrp.keys().chain(tables.armorgrp.keys()).chain(tables.etcitemgrp.keys()).copied().collect();
        assert!(!ids.is_empty());
        let items: Vec<_> = ids.iter().map(|&id| l2_net::game::WarehouseItem {
            item: TradeItem { type1: 0, object_id: id as i32 + 100000, item_id: id as i32, count: 50, type2: 0, custom_type1: 0, body_part: 0, enchant: 0, custom_type2: 0 },
            warehouse_id: id as i32 + 200000,
        }).collect();
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        ui.set_tab_names(&tables);
        for batch in items.chunks(ShopMode::Deliver.capacity()) {
            ui.shop_event(&WorldEvent::PackageSendableList(l2_net::game::PackageSendableList { target_id: 77, adena: 123456, items: batch.to_vec() }), Some(&tables));
            let view = &mut ui.keys.delivery.view;
            assert_eq!(view.top.len(), batch.len());
            for entry in &view.top {
                let id = entry.grid.item_id as u32;
                assert_eq!(entry.warehouse_id, id as i32 + 200000);
                assert_eq!(entry.grid.icon, tables.item_icons.get(id).unwrap().icons[0].trim());
                assert_eq!(entry.grid.name, tables.itemname.get(&id).map(|n| n.name.as_str()).unwrap_or_default());
                assert_eq!(entry.consume_type, tables.etcitemgrp.get(&id).map_or(0, |e| e.stackable));
            }
            for side in [Side::Top, Side::Bottom] {
                if side == Side::Bottom {
                    while !view.top.is_empty() { view.move_entry(Side::Top, 0, 1); }
                    assert_eq!(view.bottom.len(), batch.len());
                    assert_eq!(view.fee(), batch.len() as i64 * 1000);
                }
                let rows = if side == Side::Top { 4 } else { 3 };
                let total = batch.len().div_ceil(6) as i32;
                for row in (0..total).step_by(rows) {
                    let grid = if side == Side::Top { &mut view.top_grid } else { &mut view.bottom_grid };
                    grid.set_first_row(row);
                    let first = grid.scroll_row * 6;
                    let mut draw = DrawList::default();
                    view.paint(&mut draw, 0, 0, &font, 1.0);
                    for entry in view.entries(side).iter().skip(first).take(rows * 6) {
                        assert!(draw.quads.iter().any(|q| q.texture == entry.grid.icon()));
                    }
                }
            }
            let ShopAction::Deliver { target_id, orders } = view.confirm() else { panic!("delivery action") };
            assert_eq!(target_id, 77);
            assert_eq!(orders, batch.iter().map(|p| (p.warehouse_id, if shop::is_stackable(tables.etcitemgrp.get(&(p.item.item_id as u32)).map_or(0, |e| e.stackable)) { 50 } else { 1 })).collect::<Vec<_>>());
        }
        let sample: Vec<_> = items.iter().take(72).cloned().collect();
        ui.shop_event(&WorldEvent::PackageSendableList(l2_net::game::PackageSendableList { target_id: 77, adena: 123456, items: sample }), Some(&tables));
        for _ in 0..24 { ui.keys.delivery.view.move_entry(Side::Top, 0, 1); }
        ui.keys.delivery.view.top_grid.set_first_row(2);
        ui.keys.delivery.view.bottom_grid.set_first_row(1);
        ui.keys.select_delivery.open((0..8).map(|i| (i, format!("Character{}", i + 1))));
        ui.keys.select_delivery.message = crate::ui2d::system_text(Some(&tables), killer_ui::delivery::SELECT_MESSAGE, &[]);
        ui.keys.select_delivery.combo.open_list();
        ui.keys.select_delivery.combo.mouse_move(40, 96);
        let mut draw = DrawList::default();
        ui.small = font;
        for (id, x) in [(WindowId::DELIVERY, 10), (WindowId::SELECT_DELIVERY, 286)] {
            let window = ui.desktop.window_mut(id).unwrap();
            window.rect.x = x;
            window.rect.y = 10;
            ui.paint_window(&mut draw, ui.desktop.window(id).unwrap(), &killer_world::World::default(), &[], None, &crate::ui2d::WorldViews::default());
        }
        let mut headless = Headless::new(552, 430).unwrap();
        let mut bank = TextureBank::new();
        headless.renderer_mut().set_ui(&draw, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        assert_eq!(bank.missing().count(), 0);
        let camera = Camera { eye: glam::vec3(0.0, -300.0, 0.0), target: glam::Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
        let rgba = headless.render(&camera, &Environment::default()).unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-delivery.png");
        write_png(&path, 552, 430, &rgba).unwrap();
        eprintln!("{} itens conferidos nas duas grades, transferência, taxa, confirmação e captura {}", ids.len(), path.display());
    }

    #[test]
    #[ignore]
    fn private_store_windows_capture_the_four_modes_with_every_client_item_checked() {
        use killer_assets::client::Client;
        use killer_assets::textures::{BankSource, TextureBank, write_png};
        use killer_render::camera::Camera;
        use killer_render::environment::Environment;
        use killer_render::headless::Headless;
        use killer_ui::draw::DrawList;
        use killer_ui::font::{BitmapFont, Glyphs};
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let font = BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap());
        let mut ui = crate::ui2d::tests::game_ui((1100, 440));
        ui.set_tab_names(&tables);
        ui.small = font;
        let ids: Vec<i32> = tables.weapongrp.keys().chain(tables.armorgrp.keys()).chain(tables.etcitemgrp.keys()).map(|id| *id as i32).collect();
        assert!(ids.len() > 100);
        let sell = |index: usize, id: i32| l2_net::game::StoreSellItem {
            type2: 5, object_id: 1000 + index as i32, item_id: id, count: 3 + index as i32, enchant: 0, custom_type2: 0, body_part: 0, price: 1500 * (index as i32 + 1), store_price: 1000,
        };
        let buy = |index: usize, id: i32| l2_net::game::StoreBuyItem {
            object_id: 2000 + index as i32, item_id: id, enchant: 0, count: 9 + index as i32, store_price: 100, body_part: 0, type2: 5, price: 120 * (index as i32 + 1), max_count: 4,
        };
        for (batch, chunk) in ids.chunks(18).enumerate() {
            let list = l2_net::game::PrivateStoreListSell {
                object_id: 77, package_sale: batch % 2 == 0, adena: 123_456, items: chunk.iter().enumerate().map(|(i, id)| sell(i, *id)).collect(),
            };
            ui.private_shop_event(&WorldEvent::PrivateStoreListSell(list), Some(&tables));
            let shop = &ui.keys.private_shop;
            assert_eq!(shop.view.top.len(), chunk.len());
            assert!(shop.view.top.iter().zip(chunk).all(|(entry, id)| entry.grid.item_id == *id));
        }
        let first: Vec<i32> = ids.iter().copied().take(14).collect();
        let mut draw = DrawList::default();
        let place = |ui: &mut crate::ui2d::GameUi, x: i32, draw: &mut DrawList| {
            let window = ui.desktop.window_mut(WindowId::PRIVATE_SHOP).unwrap();
            window.rect.x = x;
            window.rect.y = 10;
            ui.paint_window(draw, ui.desktop.window(WindowId::PRIVATE_SHOP).unwrap(), &killer_world::World::default(), &[], None, &crate::ui2d::WorldViews::default());
        };
        ui.private_shop_event(&WorldEvent::PrivateStoreListSell(l2_net::game::PrivateStoreListSell {
            object_id: 77, package_sale: true, adena: 123_456, items: first.iter().enumerate().map(|(i, id)| sell(i, *id)).collect(),
        }), Some(&tables));
        place(&mut ui, 10, &mut draw);
        ui.private_shop_event(&WorldEvent::PrivateStoreManageListSell(l2_net::game::PrivateStoreManageListSell {
            object_id: 77, package_sale: true, adena: 123_456,
            available: first.iter().enumerate().map(|(i, id)| sell(i, *id)).collect(),
            selling: first.iter().take(4).enumerate().map(|(i, id)| sell(i, *id)).collect(),
        }), Some(&tables));
        place(&mut ui, 280, &mut draw);
        ui.private_shop_event(&WorldEvent::PrivateStoreListBuy(l2_net::game::PrivateStoreListBuy {
            object_id: 78, adena: 123_456, items: first.iter().enumerate().map(|(i, id)| buy(i, *id)).collect(),
        }), Some(&tables));
        place(&mut ui, 550, &mut draw);
        ui.private_shop_event(&WorldEvent::PrivateStoreManageListBuy(l2_net::game::PrivateStoreManageListBuy {
            object_id: 79, adena: 123_456,
            available: first.iter().enumerate().map(|(i, id)| buy(i, *id)).collect(),
            buying: first.iter().take(4).enumerate().map(|(i, id)| buy(i, *id)).collect(),
        }), Some(&tables));
        place(&mut ui, 820, &mut draw);
        let mut headless = Headless::new(1090, 430).unwrap();
        let mut bank = TextureBank::new();
        headless.renderer_mut().set_ui(&draw, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        assert_eq!(bank.missing().count(), 0);
        let camera = Camera { eye: glam::vec3(0.0, -300.0, 0.0), target: glam::Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
        let rgba = headless.render(&camera, &Environment::default()).unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-private-store.png");
        write_png(&path, 1090, 430, &rgba).unwrap();
        eprintln!("{} itens conferidos, captura {}", ids.len(), path.display());
    }

    #[test]
    #[ignore]
    fn private_store_dialogs_capture_with_the_client_texts() {
        use killer_assets::client::Client;
        use killer_assets::textures::{BankSource, TextureBank, write_png};
        use killer_render::camera::Camera;
        use killer_render::environment::Environment;
        use killer_render::headless::Headless;
        use killer_ui::dialog::{DialogKind, DialogView, paint_dialog};
        use killer_ui::draw::DrawList;
        use killer_ui::font::{BitmapFont, Glyphs};
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let font = BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap());
        let text = |id: u32, args: &[SystemMessageArg]| crate::ui2d::system_text(Some(&tables), id, args);
        let label = |id: u32| tables.sysstring.get(&id).cloned().unwrap_or_default();
        let mut views = Vec::new();
        let mut message = DialogView::with_text(DialogKind::OkCancel, &text(ps::MESSAGE_STORE, &[]));
        message.input.as_mut().unwrap().text = "Vendo barato".into();
        message.dialog_type = 0x27;
        views.push(message);
        let mut caution = DialogView::new(DialogKind::OkCancel, &text(ps::MESSAGE_PRICE_CAUTION, &[]));
        caution.dialog_type = 0x29;
        views.push(caution);
        let mut amount = DialogView::with_count(DialogKind::OkCancel, &text(ps::MESSAGE_BUY_HOW_MANY, &[SystemMessageArg::Text("Adena".into())]), 5000);
        amount.input.as_mut().unwrap().text = "250".into();
        amount.dialog_type = 0x2b;
        views.push(amount);
        let mut link = DialogView::new(DialogKind::OkCancel, &killer_ui::html::expand_system_msg_code("570;Adena", |id| tables.systemmsg.get(&id).map(|m| m.text.clone())));
        link.dialog_type = 0x3c;
        views.push(link);
        let mut draw = DrawList::default();
        for (index, mut view) in views.into_iter().enumerate() {
            let (first, second) = view.kind.sysstrings();
            view.labels = (label(first), label(second));
            let (x, y) = (10 + (index as i32 % 2) * 380, 10 + (index as i32 / 2) * 120);
            paint_dialog(&mut draw, x, y, &view, None, &font);
        }
        let mut headless = Headless::new(760, 380).unwrap();
        let mut bank = TextureBank::new();
        headless.renderer_mut().set_ui(&draw, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        assert_eq!(bank.missing().count(), 0);
        let camera = Camera { eye: glam::vec3(0.0, -300.0, 0.0), target: glam::Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
        let rgba = headless.render(&camera, &Environment::default()).unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-private-store-dialogs.png");
        write_png(&path, 760, 380, &rgba).unwrap();
    }

    #[test]
    #[ignore]
    fn on_buy_list_item_10078840_covers_all_client_items_and_captures_shop() {
        use killer_assets::client::Client;
        use killer_assets::textures::{BankSource, TextureBank, write_png};
        use killer_render::camera::Camera;
        use killer_render::environment::Environment;
        use killer_render::headless::Headless;
        use killer_ui::draw::DrawList;
        use killer_ui::font::{BitmapFont, Glyphs};
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let font = BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap());
        let ids: std::collections::BTreeSet<_> = tables.weapongrp.keys().chain(tables.armorgrp.keys()).chain(tables.etcitemgrp.keys()).copied().collect();
        assert!(!ids.is_empty());
        let items: Vec<_> = ids.iter().map(|&id| PricedItem {
            item: TradeItem { type1: 0, object_id: id as i32 + 100000, item_id: id as i32, count: 50, type2: 0, custom_type1: 0, body_part: 0, enchant: 0, custom_type2: 0 },
            price: 10,
        }).collect();
        let list = ShopList { money: 123456, list_id: 7, items };
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        ui.set_tab_names(&tables);
        ui.shop_event(&WorldEvent::BuyList(list.clone()), Some(&tables));
        assert_eq!(ui.keys.shop.view.top.len(), ids.len());
        for entry in &ui.keys.shop.view.top {
            let id = entry.grid.item_id as u32;
            assert_eq!(entry.grid.icon, tables.item_icons.get(id).unwrap().icons[0].trim());
            assert_eq!(entry.consume_type, tables.etcitemgrp.get(&id).map_or(0, |e| e.stackable));
            assert_eq!(entry.grid.name, tables.itemname.get(&id).map(|n| n.name.as_str()).unwrap_or_default());
        }
        for row in (0..ui.keys.shop.view.top_grid.total_rows(ids.len())).step_by(4) {
            ui.keys.shop.view.top_grid.set_first_row(row);
            let mut draw = DrawList::default();
            ui.keys.shop.view.paint(&mut draw, 0, 0, &font, 1.0);
            let first = row as usize * 6;
            for entry in ui.keys.shop.view.top.iter().skip(first).take(24) {
                assert!(draw.quads.iter().any(|q| q.texture == entry.grid.icon()));
            }
        }
        let all_warehouse = l2_net::game::WarehouseList {
            warehouse_type: 1, adena: 123456,
            items: list.items.iter().map(|p| l2_net::game::WarehouseItem {
                item: p.item, warehouse_id: p.item.object_id + 100000,
            }).collect(),
        };
        for deposit in [true, false] {
          for batch in all_warehouse.items.chunks(shop::WAREHOUSE_GRID_CAPACITY) {
            let warehouse = l2_net::game::WarehouseList { items: batch.to_vec(), ..all_warehouse.clone() };
            let event = if deposit { WorldEvent::WarehouseDepositList(warehouse) }
                else { WorldEvent::WarehouseWithdrawList(warehouse) };
            ui.shop_event(&event, Some(&tables));
            assert_eq!(ui.keys.warehouse.view.top.len(), batch.len());
            for entry in &ui.keys.warehouse.view.top {
                let id = entry.grid.item_id as u32;
                assert_eq!(entry.warehouse_id, entry.grid.object_id + 100000);
                assert_eq!(entry.grid.icon, tables.item_icons.get(id).unwrap().icons[0].trim());
                assert_eq!(entry.consume_type, tables.etcitemgrp.get(&id).map_or(0, |e| e.stackable));
                assert_eq!(entry.grid.name, tables.itemname.get(&id).map(|n| n.name.as_str()).unwrap_or_default());
            }
            for row in (0..ui.keys.warehouse.view.top_grid.total_rows(batch.len())).step_by(4) {
                ui.keys.warehouse.view.top_grid.set_first_row(row);
                let mut draw = DrawList::default();
                ui.keys.warehouse.view.paint(&mut draw, 0, 0, &font, 1.0);
                for entry in ui.keys.warehouse.view.top.iter().skip(row as usize * 6).take(24) {
                    assert!(draw.quads.iter().any(|q| q.texture == entry.grid.icon()));
                }
            }
          }
        }
        let sample = ShopList { items: list.items.iter().filter(|p| [1, 17, 57, 1060].contains(&p.item.item_id)).copied().collect(), ..list };
        assert_eq!(sample.items.len(), 4);
        let mut headless = Headless::new(400, 430).unwrap();
        for deposit in [true, false] {
            let warehouse = l2_net::game::WarehouseList {
                warehouse_type: 2, adena: 123456,
                items: sample.items.iter().map(|p| l2_net::game::WarehouseItem { item: p.item, warehouse_id: p.item.object_id }).collect(),
            };
            let event = if deposit { WorldEvent::WarehouseDepositList(warehouse) } else { WorldEvent::WarehouseWithdrawList(warehouse) };
            ui.shop_event(&event, Some(&tables));
            ui.keys.warehouse.view.move_entry(Side::Top, 0, 1);
            let mut draw = DrawList::default();
            ui.keys.warehouse.view.paint(&mut draw, 10, 10, &font, 1.0);
            let mut bank = TextureBank::new();
            headless.renderer_mut().set_ui(&draw, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
            assert_eq!(bank.missing().count(), 0);
            let camera = Camera { eye: glam::vec3(0.0, -300.0, 0.0), target: glam::Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
            let rgba = headless.render(&camera, &Environment::default()).unwrap();
            let name = if deposit { "ui-warehouse-deposit.png" } else { "ui-warehouse-withdraw.png" };
            write_png(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders").join(name), 400, 430, &rgba).unwrap();
        }
        for buy in [true, false] {
            let event = if buy { WorldEvent::BuyList(sample.clone()) } else { WorldEvent::SellList(sample.clone()) };
            ui.shop_event(&event, Some(&tables));
            ui.keys.shop.view.move_entry(Side::Top, 0, 1);
            let mut draw = DrawList::default();
            ui.keys.shop.view.paint(&mut draw, 10, 10, &font, 1.0);
            let mut bank = TextureBank::new();
            headless.renderer_mut().set_ui(&draw, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
            assert_eq!(bank.missing().count(), 0);
            let camera = Camera { eye: glam::vec3(0.0, -300.0, 0.0), target: glam::Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
            let rgba = headless.render(&camera, &Environment::default()).unwrap();
            let name = if buy { "ui-shop-buy.png" } else { "ui-shop-sell.png" };
            write_png(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders").join(name), 400, 430, &rgba).unwrap();
        }
        ui.keys.shop.double_click(48, 52);
        let mut world = killer_world::World::default();
        ui.tick_dialog(&mut world, Some(&tables), killer_platform::time::Instant::now());
        let dialog = ui.dialog.as_ref().unwrap();
        assert!(dialog.view.input.is_some());
        let mut draw = DrawList::default();
        killer_ui::dialog::paint_dialog(&mut draw, 10, 10, &dialog.view, None, &font);
        let mut bank = TextureBank::new();
        headless.renderer_mut().set_ui(&draw, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        assert_eq!(bank.missing().count(), 0);
        let camera = Camera { eye: glam::vec3(0.0, -300.0, 0.0), target: glam::Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
        let rgba = headless.render(&camera, &Environment::default()).unwrap();
        write_png(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-shop-count.png"), 400, 430, &rgba).unwrap();
        eprintln!("{} itens do cliente conferidos e pintados em loja/depósito/retirada; cinco capturas GPU", ids.len());
    }

    #[test]
    fn the_shop_scrolls_and_returning_items_undoes_the_selection() {
        let (mut shop, mut desktop, mut commands) = setup(false);
        let entry = shop.view.top[0].clone();
        shop.view.top = vec![entry; 60];
        shop.view.top_grid.update_rows(60);
        assert!(shop.wheel(12, 52, -120));
        assert!(shop.view.top_grid.scroll_row > 0);
        shop.view.top_grid.set_first_row(0);
        shop.double_click(12, 52);
        shop.answer_text(Some("12"));
        click(&mut shop, &mut desktop, &mut commands, 12, 218);
        click(&mut shop, &mut desktop, &mut commands, 116, 198);
        assert!(shop.take_dialog(None).unwrap().input.is_some());
        shop.answer_text(Some("12"));
        assert!(shop.view.bottom.is_empty());
        assert!(commands.is_empty());
    }

    #[test]
    fn the_preview_window_lists_the_offered_pieces_and_sends_the_header_after_the_confirmation() {
        let mut tables = DatTables::default();
        tables.item_icons.insert_all(vec![ItemIcon {
            id: 21, kind: ItemKind::Armor, drop: Default::default(), icons: std::array::from_fn(|_| "icon.armor_i00".into()), weight: 5, type1: 1,
        }]);
        let mut window = ShopWindow::new(|_| None);
        window.id = WindowId::SHOP_PREVIEW;
        let mut desktop = Desktop::new((800, 600));
        let mut preview = killer_ui::window::Window::new(WindowId::SHOP_PREVIEW, Rect::new(0, 0, 256, 401), shop::grip());
        preview.visible = false;
        desktop.windows.push(preview);
        let mut npc = Window::new(WindowId::NPC_HTML, Rect::new(0, 0, 10, 10), Rect::new(0, 0, 10, 10));
        npc.visible = true;
        desktop.windows.push(npc);
        let event = WorldEvent::ShopPreviewList(l2_net::game::ShopPreviewList {
            header: 77,
            money: 500,
            list_id: 5,
            items: vec![
                l2_net::game::PreviewItem { item_id: 21, type2: 1, body_part: 10, price: 100 },
                l2_net::game::PreviewItem { item_id: 999, type2: 1, body_part: 10, price: 1 },
            ],
        });
        assert!(!window.preview_event(&WorldEvent::BuyList(ShopList { money: 0, list_id: 1, items: vec![] }), Some(&tables), &mut desktop));
        assert!(window.preview_event(&event, Some(&tables), &mut desktop));
        assert_eq!((window.view.mode, window.view.header, window.view.list_id, window.view.top.len()), (ShopMode::Preview, 77, 5, 1));
        assert_eq!(window.view.top[0].quantity, 0);
        assert!(desktop.window(WindowId::SHOP_PREVIEW).unwrap().visible);
        assert!(!desktop.window(WindowId::NPC_HTML).unwrap().visible);
        let mut commands = Vec::new();
        let ok = (shop::OK_RECT.x + 1, shop::OK_RECT.y + 1);
        window.mouse_down(ok.0, ok.1, &mut desktop);
        window.mouse_up(Some((WindowId::SHOP_PREVIEW, ok.0, ok.1)), &mut desktop, &mut commands);
        assert!(window.take_dialog(Some(&tables)).is_none());
        window.view.select(Side::Top, 0);
        assert_eq!(window.view.press_move(Side::Top), ShopAction::None);
        window.mouse_down(ok.0, ok.1, &mut desktop);
        window.mouse_up(Some((WindowId::SHOP_PREVIEW, ok.0, ok.1)), &mut desktop, &mut commands);
        let dialog = window.take_dialog(Some(&tables)).unwrap();
        assert_eq!(dialog.dialog_type, shop::PREVIEW_DIALOG_TYPE);
        assert!(window.answer_confirm(false, &mut desktop).is_none());
        assert!(desktop.window(WindowId::SHOP_PREVIEW).unwrap().visible);
        assert_eq!(
            window.answer_confirm(true, &mut desktop),
            Some(NetCommand::PreviewItem { header: 77, list_id: 5, items: vec![21] })
        );
        assert!(!desktop.window(WindowId::SHOP_PREVIEW).unwrap().visible);
        assert!(window.view.top.is_empty() && window.view.bottom.is_empty());
        assert!(commands.is_empty());
    }

    fn manor_setup(procure: bool) -> (ShopWindow, Desktop, Vec<UiCommand>) {
        let mut shop = ShopWindow::new(|_| None);
        shop.id = WindowId::MANOR_SHOP;
        let mut desktop = Desktop::new((1000, 800));
        for id in [WindowId::MANOR_SHOP, WindowId::INVENTORY, WindowId::NPC_HTML] {
            desktop.windows.push(Window::new(id, Rect::new(0, 0, 256, 401), shop::grip()));
        }
        let event = if procure {
            WorldEvent::SellListProcure(l2_net::game::SellListProcure {
                header: [800, 5],
                header_short: 1,
                items: vec![l2_net::game::SellListProcureItem { kind: 4, values: [900, 57, 50], shorts: [0, 0], details: None, price: 12 }],
            })
        } else {
            WorldEvent::BuyListSeed(l2_net::game::BuyListSeed {
                money: 800,
                manor_id: 5,
                items: vec![l2_net::game::SeedItem { type1: 4, object_id: 900, item_id: 57, count: 50, type2: 5, custom_type1: 0, body_part: 0, enchant: 0, price: 12 }],
            })
        };
        assert!(shop.world_event(&event, Some(&tables()), &mut desktop));
        (shop, desktop, vec![])
    }

    #[test]
    fn manor_shop_opens_on_the_seed_and_procure_packets_and_hides_the_competing_windows() {
        let (shop, desktop, _) = manor_setup(false);
        assert_eq!((shop.view.mode, shop.view.list_id, shop.view.adena), (ShopMode::ManorSeed, 5, 800));
        assert!(desktop.window(WindowId::MANOR_SHOP).unwrap().visible);
        assert_eq!(shop.view.top[0].stock, 50);
        assert!(desktop.window(WindowId::INVENTORY).unwrap().visible);
        assert!(!desktop.window(WindowId::NPC_HTML).unwrap().visible);
        let (shop, desktop, _) = manor_setup(true);
        assert_eq!((shop.view.mode, shop.view.list_id, shop.view.adena), (ShopMode::ManorProcure, 5, 800));
        assert!(!desktop.window(WindowId::INVENTORY).unwrap().visible);
        assert_eq!(shop.view.top[0].grid.object_id, 900);
    }

    #[test]
    fn manor_seed_purchase_sends_buy_seed_and_clears_the_window() {
        let (mut shop, mut desktop, mut commands) = manor_setup(false);
        click(&mut shop, &mut desktop, &mut commands, 12, 52);
        assert!(shop.take_dialog(None).is_none(), "clicar só seleciona na loja do Manor");
        let down = killer_ui::shop::DOWN_RECT;
        click(&mut shop, &mut desktop, &mut commands, down.x + 2, down.y + 2);
        let dialog = shop.take_dialog(None).expect("a contagem é perguntada para item empilhável");
        assert_eq!(dialog.input.unwrap().max, 50);
        shop.answer_text(Some("4"));
        let ok = killer_ui::shop::OK_RECT;
        click(&mut shop, &mut desktop, &mut commands, ok.x + 2, ok.y + 2);
        assert!(matches!(
            commands.as_slice(),
            [UiCommand::Net(NetCommand::BuySeed { manor_id: 5, orders })] if orders == &[(57, 4)]
        ));
        assert!(!desktop.window(WindowId::MANOR_SHOP).unwrap().visible);
        assert!(shop.view.top.is_empty() && shop.view.bottom.is_empty());
    }

    #[test]
    fn manor_ok_without_a_choice_only_hides_the_seed_window_and_the_procure_window_always_sends() {
        let (mut shop, mut desktop, mut commands) = manor_setup(false);
        let ok = killer_ui::shop::OK_RECT;
        click(&mut shop, &mut desktop, &mut commands, ok.x + 2, ok.y + 2);
        assert!(commands.is_empty());
        assert!(!desktop.window(WindowId::MANOR_SHOP).unwrap().visible);
        let (mut shop, mut desktop, mut commands) = manor_setup(true);
        click(&mut shop, &mut desktop, &mut commands, ok.x + 2, ok.y + 2);
        assert!(matches!(
            commands.as_slice(),
            [UiCommand::Net(NetCommand::ProcureCrop { manor_id: 5, orders })] if orders.is_empty()
        ));
        assert!(!desktop.window(WindowId::MANOR_SHOP).unwrap().visible);
    }

    #[test]
    fn manor_procure_sends_the_chosen_object_item_and_count() {
        let (mut shop, mut desktop, mut commands) = manor_setup(true);
        shop.view.select(Side::Top, 0);
        shop.view.move_entry(Side::Top, 0, 6);
        let ok = killer_ui::shop::OK_RECT;
        click(&mut shop, &mut desktop, &mut commands, ok.x + 2, ok.y + 2);
        assert!(matches!(
            commands.as_slice(),
            [UiCommand::Net(NetCommand::ProcureCrop { manor_id: 5, orders })]
                if orders.iter().map(|o| (o.object_id, o.item_id, o.count)).collect::<Vec<_>>() == [(900, 57, 6)]
        ));
    }
}
