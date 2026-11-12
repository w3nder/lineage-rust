use killer_assets::client::DatTables;
use killer_ui::dialog::{DialogKind, DialogView};
use killer_ui::item_grid::GridEntry;
use killer_ui::scroll::{ScrollHit, line_down, line_up};
use killer_ui::trade::{self, TradeAction, TradeEntry, TradeGrid, TradeHit, TradeView};
use killer_ui::window::{Desktop, WindowId};
use killer_world::{NetCommand, World, WorldEvent};
use l2_net::game::{SystemMessageArg, TradeItem};

use crate::ui_keys::UiCommand;

pub struct TradeWindow {
    pub view: TradeView,
    cursor_drag: bool,
    pressed: Option<TradeHit>,
    count_index: Option<usize>,
    consume_type: u32,
    dialog_type: i32,
    pending_dialog: Option<(u32, String, Option<i64>)>,
    pub commands: Vec<UiCommand>,
}

impl TradeWindow {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self {
            view: TradeView::with_sysstrings(lookup), pressed: None, cursor_drag: false, count_index: None,
            consume_type: 0, dialog_type: 8, pending_dialog: None, commands: vec![],
        }
    }

    pub fn entry(item: &TradeItem, tables: Option<&DatTables>) -> Option<TradeEntry> {
        let id = u32::try_from(item.item_id).ok()?;
        let tables = tables?;
        let icon = tables.item_icons.get(id)?;
        let name = tables.itemname.get(&id);
        Some(TradeEntry {
            grid: GridEntry {
                object_id: item.object_id, item_id: item.item_id, count: item.count,
                enchant: i32::from(item.enchant), icon: icon.icons[0].trim().to_string(),
                name: name.map(|n| n.name.clone()).unwrap_or_default(),
                add_name: name.map(|n| n.add_name.clone()).unwrap_or_default(),
                description: name.map(|n| n.description.clone()).unwrap_or_default(),
                weight: icon.weight as i32,
            },
            quantity: i64::from(item.count),
            consume_type: tables.etcitemgrp.get(&id).map_or(0, |e| e.stackable),
            tip: Some(crate::ui_keys::item_tip_data(tables, item.item_id, i32::from(item.type2), item.body_part, i32::from(item.custom_type1), i32::from(item.custom_type2))),
        })
    }

    pub fn world_event(&mut self, event: &WorldEvent, world: &World, tables: Option<&DatTables>, desktop: &mut Desktop) -> bool {
        match event {
            WorldEvent::TradeStart(start) => {
                self.clear();
                if let Some(window) = desktop.window_mut(WindowId::INVENTORY) { window.visible = false; }
                if let (Some(me), Some(partner)) = (world.me.as_ref(), world.entities.get(&start.partner_id)) {
                    self.view.my_name = me.name.clone();
                    self.view.partner_name = partner.name.clone();
                    if let Some(window) = desktop.window_mut(WindowId::TRADE) { window.visible = true; }
                    if let Some(index) = desktop.windows.iter().position(|w| w.id == WindowId::TRADE) {
                        let window = desktop.windows.remove(index);
                        desktop.windows.push(window);
                    }
                } else {
                    self.send(NetCommand::TradeDone { confirm: false });
                }
                for item in &start.items {
                    if let Some(entry) = Self::entry(item, tables) { self.view.receive_add(TradeGrid::Mine, entry); }
                }
                true
            }
            WorldEvent::TradeOwnAdd(add) | WorldEvent::TradeOtherAdd(add) => {
                let grid = if matches!(event, WorldEvent::TradeOwnAdd(_)) { TradeGrid::Offered } else { TradeGrid::Theirs };
                for item in &add.items {
                    if let Some(entry) = Self::entry(item, tables) { self.view.receive_add(grid, entry); }
                }
                false
            }
            WorldEvent::TradeUpdate(update) => {
                for item in &update.items {
                    if let Some(entry) = Self::entry(&item.item, tables) { self.view.receive_update(item.update_type, entry); }
                }
                false
            }
            WorldEvent::TradeDone(_) => {
                self.clear();
                if let Some(window) = desktop.window_mut(WindowId::TRADE) { window.visible = false; }
                true
            }
            _ => false,
        }
    }

    fn send(&mut self, command: NetCommand) { self.commands.push(UiCommand::Net(command)); }

    pub fn clear(&mut self) {
        self.view.clear();
        self.pressed = None;
        self.count_index = None;
        self.pending_dialog = None;
    }

    fn action(&mut self, action: TradeAction, dialog_type: i32) {
        match action {
            TradeAction::None => {}
            TradeAction::AddItem { object_id, count } => self.send(NetCommand::AddTradeItem { trade_id: trade::TRADE_ID, object_id, count }),
            TradeAction::AskCount { index, item_name, max, consume_type } => {
                self.count_index = Some(index);
                self.consume_type = consume_type;
                self.dialog_type = dialog_type;
                self.pending_dialog = Some((trade::MOVE_COUNT_SYSMSG, item_name, Some(max)));
            }
            TradeAction::TooMany => self.commands.push(UiCommand::ColoredSystemMessage { id: trade::TOO_MANY_INPUT_SYSMSG, color: 0xffdcdcdc }),
        }
    }

    pub fn cursor_hover(&mut self) { self.cursor_drag = self.pressed.is_some(); }

    pub fn drag_icon(&self) -> Option<String> {
        if !self.cursor_drag { return None; }
        let Some(TradeHit::Slot(TradeGrid::Mine, index)) = self.pressed else { return None; };
        self.view.entries(TradeGrid::Mine).get(index).map(|entry| entry.grid.icon.clone()).filter(|icon| !icon.is_empty())
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        self.cursor_drag = false;
        self.pressed = self.view.hit(x, y);
        match self.pressed {
            Some(TradeHit::Slot(grid, index)) => self.view.select(grid, index),
            Some(TradeHit::Ok) => self.view.ok_pressed = true,
            Some(TradeHit::Cancel) => self.view.cancel_pressed = true,
            Some(TradeHit::Add) => self.view.add_pressed = true,
            Some(TradeHit::Scroll(which, hit)) => {
                let grid = self.view.grid_mut(which);
                grid.scroll.up_pressed = hit == ScrollHit::Up;
                grid.scroll.down_pressed = hit == ScrollHit::Down;
                if let ScrollHit::Slider(pos) = hit { grid.scroll.slider.mouse_down(pos); }
            }
            _ => {}
        }
    }

    pub fn mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        self.view.ok_pressed = false;
        self.view.cancel_pressed = false;
        self.view.add_pressed = false;
        let released = over.filter(|(id, _, _)| *id == WindowId::TRADE).and_then(|(_, x, y)| self.view.hit(x, y));
        let Some(pressed) = self.pressed.take() else { return };
        if let TradeHit::Scroll(which, hit) = pressed {
            let count = self.view.entries(which).len();
            let grid = self.view.grid_mut(which);
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
        match (pressed, released) {
            (TradeHit::Slot(TradeGrid::Mine, index), Some(TradeHit::Slot(TradeGrid::Offered, _))) => {
                let action = self.view.add_entry(index);
                self.action(action, 8);
            }
            _ if released != Some(pressed) => {}
            (TradeHit::Close, _) => {
                self.send(NetCommand::TradeDone { confirm: false });
                if let Some(window) = desktop.window_mut(WindowId::TRADE) { window.visible = false; }
            }
            (TradeHit::Add, _) => {
                let action = self.view.press_add();
                self.action(action, 2);
            }
            (TradeHit::Ok, _) => self.send(NetCommand::TradeDone { confirm: true }),
            (TradeHit::Cancel, _) => self.send(NetCommand::TradeDone { confirm: false }),
            _ => {}
        }
    }

    pub fn double_click(&mut self, x: i32, y: i32) -> bool {
        let Some(TradeHit::Slot(TradeGrid::Mine, index)) = self.view.hit(x, y) else { return false };
        self.pressed = None;
        self.view.select(TradeGrid::Mine, index);
        let action = self.view.add_entry(index);
        self.action(action, 8);
        true
    }

    pub fn mouse_move(&mut self, y: i32) -> bool {
        let Some(TradeHit::Scroll(which, ScrollHit::Slider(_))) = self.pressed else { return false };
        let grid = self.view.grid_mut(which);
        if let Some(first) = grid.scroll.slider.mouse_move(y - grid.rect.y - killer_ui::scroll::BUTTON_SIZE) { grid.scroll_row = first.max(0) as usize; }
        true
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        for which in [TradeGrid::Mine, TradeGrid::Offered, TradeGrid::Theirs] {
            let count = self.view.entries(which).len();
            let grid = self.view.grid_mut(which);
            if grid.rect.contains(x, y) {
                let first = killer_ui::inventory::item_wnd_wheel(grid.scroll_row as i32, grid.rows as i32, grid.total_rows(count), delta);
                grid.set_first_row(first);
                return true;
            }
        }
        false
    }

    pub fn take_dialog(&mut self, tables: Option<&DatTables>) -> Option<DialogView> {
        let (id, name, max) = self.pending_dialog.take()?;
        let args = if max.is_some() { vec![SystemMessageArg::Text(name)] } else { vec![] };
        let text = crate::ui2d::system_text(tables, id, &args);
        let mut view = match max {
            Some(max) => DialogView::with_count(DialogKind::OkCancel, &text, max),
            None => DialogView::new(DialogKind::OkCancel, &text),
        };
        view.dialog_type = if max.is_some() { self.dialog_type } else { 31 };
        Some(view)
    }

    pub fn answer_text(&mut self, text: Option<&str>) {
        if self.count_index.take().is_none() { return; }
        let Some(index) = self.view.mine_grid.selected.filter(|i| *i < self.view.mine.len()) else { return };
        let Some(text) = text else { return };
        let count = text.bytes().take_while(u8::is_ascii_digit).fold(0i64, |n, c| n.saturating_mul(10).saturating_add(i64::from(c - b'0')));
        if killer_ui::shop::count_limit(self.consume_type).is_some_and(|limit| count > limit) {
            let message = if self.consume_type == 2 { 1338 } else { 1369 };
            self.pending_dialog = Some((message, String::new(), None));
            return;
        }
        let action = self.view.answer_count(index, i64::from(count as i32));
        self.action(action, self.dialog_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::chat::InputKey;
    use l2_net::game::{TradeAdd, TradeDone, TradeStart, TradeUpdate, TradeUpdateItem};

    fn item() -> TradeItem {
        TradeItem { type1: 4, object_id: 900, item_id: 57, count: 50, type2: 5, custom_type1: 0,
            body_part: 0, enchant: 0, custom_type2: 0 }
    }

    fn world() -> World {
        let mut world = crate::characters::demo_world(glam::Vec3::ZERO, 0, 0);
        let mut partner = world.me.clone().unwrap();
        partner.object_id = 7;
        partner.name = "Parceiro".into();
        world.entities.insert(7, partner);
        world
    }

    #[test]
    fn receive_trade_start_10077510_cancels_without_partner_and_does_not_show() {
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        ui.trade_event(&WorldEvent::TradeStart(TradeStart { partner_id: 7, items: vec![] }), &World::default(), None);
        assert_eq!(ui.take_commands(), vec![UiCommand::Net(NetCommand::TradeDone { confirm: false })]);
        assert!(!ui.desktop.window(WindowId::TRADE).unwrap().visible);
    }

    #[test]
    fn trade_grid_items_carry_the_full_item_tip_of_the_item_wnd_tooltip_10020370() {
        let tables = crate::ui_shop::tests::tables();
        let entry = TradeWindow::entry(&item(), Some(&tables)).unwrap();
        let tip = entry.tip.unwrap();
        assert_eq!((tip.type2, tip.body_part), (i32::from(item().type2), item().body_part));
        assert_eq!(tip.weight, entry.grid.weight);
    }

    #[test]
    fn trade_quantity_1011a660_sends_only_then_updates_from_server_and_waits_for_done() {
        let tables = crate::ui_shop::tests::tables();
        let mut world = world();
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        ui.trade_event(&WorldEvent::TradeStart(TradeStart { partner_id: 7, items: vec![item()] }), &world, Some(&tables));
        assert!(ui.desktop.window(WindowId::TRADE).unwrap().visible);
        assert_eq!(ui.desktop.windows.last().unwrap().id, WindowId::TRADE);
        assert_eq!(ui.keys.trade.view.partner_name, "Parceiro");
        assert!(!ui.desktop.window(WindowId::INVENTORY).unwrap().visible);
        ui.keys.double_click(WindowId::TRADE, 12, 40, &world);
        ui.tick_dialog(&mut world, Some(&tables), killer_platform::time::Instant::now());
        assert_eq!(ui.dialog.as_ref().unwrap().view.dialog_type, 8);
        ui.key(InputKey::Char('7'));
        ui.key(InputKey::Enter);
        assert_eq!(ui.take_commands(), vec![UiCommand::Net(NetCommand::AddTradeItem { trade_id: 1, object_id: 900, count: 7 })]);
        assert_eq!(ui.keys.trade.view.mine[0].quantity, 50);
        assert!(ui.keys.trade.view.offered.is_empty());
        let added = TradeItem { count: 7, ..item() };
        for event in [WorldEvent::TradeOwnAdd(TradeAdd { items: vec![added] }), WorldEvent::TradeOtherAdd(TradeAdd { items: vec![added] })] {
            ui.trade_event(&event, &world, Some(&tables));
        }
        ui.trade_event(&WorldEvent::TradeUpdate(TradeUpdate { items: vec![TradeUpdateItem {
            update_type: 3, item: TradeItem { count: 43, ..item() },
        }] }), &world, Some(&tables));
        assert_eq!(ui.keys.trade.view.mine[0].quantity, 43);
        assert_eq!(ui.keys.trade.view.offered[0].quantity, 7);
        assert_eq!(ui.keys.trade.view.theirs[0].quantity, 7);
        for (x, confirm) in [(60, true), (140, false)] {
            ui.keys.trade.mouse_down(x, 380);
            ui.keys.trade.mouse_up(Some((WindowId::TRADE, x, 380)), &mut ui.desktop);
            assert_eq!(ui.take_commands(), vec![UiCommand::Net(NetCommand::TradeDone { confirm })]);
            assert!(ui.desktop.window(WindowId::TRADE).unwrap().visible);
        }
        ui.trade_event(&WorldEvent::TradeDone(TradeDone { success: true }), &world, Some(&tables));
        assert!(!ui.desktop.window(WindowId::TRADE).unwrap().visible);
        assert!(ui.keys.trade.view.mine.is_empty() && ui.keys.trade.view.offered.is_empty() && ui.keys.trade.view.theirs.is_empty());
    }

    #[test]
    fn trade_quantity_answer_offers_the_item_selected_when_it_arrives() {
        let tables = crate::ui_shop::tests::tables();
        let mut world = world();
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        ui.trade_event(&WorldEvent::TradeStart(TradeStart { partner_id: 7, items: vec![item()] }), &world, Some(&tables));
        let mut second = ui.keys.trade.view.mine[0].clone();
        second.grid.object_id = 901;
        second.quantity = 30;
        ui.keys.trade.view.mine.push(second);
        ui.keys.trade.view.mine_grid.update_rows(2);
        ui.keys.double_click(WindowId::TRADE, 12, 40, &world);
        ui.tick_dialog(&mut world, Some(&tables), killer_platform::time::Instant::now());
        ui.keys.trade.view.select(TradeGrid::Mine, 1);
        ui.key(InputKey::Char('2'));
        ui.key(InputKey::Enter);
        assert_eq!(ui.take_commands(), vec![UiCommand::Net(NetCommand::AddTradeItem { trade_id: 1, object_id: 901, count: 2 })]);
    }

    #[test]
    fn trade_frame_close_10119ba0_answers_no_on_release_and_hides_the_window() {
        let tables = crate::ui_shop::tests::tables();
        let world = world();
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        ui.trade_event(&WorldEvent::TradeStart(TradeStart { partner_id: 7, items: vec![item()] }), &world, Some(&tables));
        ui.keys.trade.mouse_down(240, 10);
        assert!(ui.take_commands().is_empty());
        ui.keys.trade.mouse_up(Some((WindowId::TRADE, 100, 100)), &mut ui.desktop);
        assert!(ui.take_commands().is_empty() && ui.desktop.window(WindowId::TRADE).unwrap().visible);
        ui.keys.trade.mouse_down(240, 10);
        ui.keys.trade.mouse_up(Some((WindowId::TRADE, 240, 10)), &mut ui.desktop);
        assert_eq!(ui.take_commands(), vec![UiCommand::Net(NetCommand::TradeDone { confirm: false })]);
        assert!(!ui.desktop.window(WindowId::TRADE).unwrap().visible);
    }

    #[test]
    fn trade_button_1011aa10_uses_type_two_and_excess_is_a_chat_message() {
        let tables = crate::ui_shop::tests::tables();
        let world = world();
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        ui.trade_event(&WorldEvent::TradeStart(TradeStart { partner_id: 7, items: vec![item()] }), &world, Some(&tables));
        ui.keys.trade.mouse_down(12, 40);
        ui.keys.trade.mouse_up(Some((WindowId::TRADE, 12, 40)), &mut ui.desktop);
        ui.keys.trade.mouse_down(145, 185);
        ui.keys.trade.mouse_up(Some((WindowId::TRADE, 145, 185)), &mut ui.desktop);
        assert_eq!(ui.keys.trade.take_dialog(Some(&tables)).unwrap().dialog_type, 2);
        ui.keys.trade.answer_text(Some("51"));
        assert_eq!(ui.take_commands(), vec![UiCommand::ColoredSystemMessage { id: 1036, color: 0xffdcdcdc }]);
        assert_eq!(ui.keys.trade.view.mine[0].quantity, 50);
    }

    #[test]
    #[ignore]
    fn trade_10077660_covers_all_client_items_three_grids_and_gpu_capture() {
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
        let items: Vec<_> = ids.iter().map(|&id| TradeItem { item_id: id as i32, object_id: id as i32 + 100000, ..item() }).collect();
        let world = world();
        let mut ui = crate::ui2d::tests::game_ui((1000, 800));
        ui.set_tab_names(&tables);
        for batch in items.chunks(trade::GRID_CAPACITY) {
            ui.trade_event(&WorldEvent::TradeStart(TradeStart { partner_id: 7, items: batch.to_vec() }), &world, Some(&tables));
            ui.trade_event(&WorldEvent::TradeOwnAdd(TradeAdd { items: batch.to_vec() }), &world, Some(&tables));
            ui.trade_event(&WorldEvent::TradeOtherAdd(TradeAdd { items: batch.to_vec() }), &world, Some(&tables));
            for which in [TradeGrid::Mine, TradeGrid::Offered, TradeGrid::Theirs] {
                assert_eq!(ui.keys.trade.view.entries(which).len(), batch.len());
                for entry in ui.keys.trade.view.entries(which) {
                    let id = entry.grid.item_id as u32;
                    assert_eq!(entry.grid.icon, tables.item_icons.get(id).unwrap().icons[0].trim());
                    assert_eq!(entry.grid.name, tables.itemname.get(&id).map(|n| n.name.as_str()).unwrap_or_default());
                    assert_eq!(entry.consume_type, tables.etcitemgrp.get(&id).map_or(0, |e| e.stackable));
                }
                let rows = ui.keys.trade.view.grid_mut(which).rows;
                let total = ui.keys.trade.view.grid_mut(which).total_rows(batch.len());
                for row in (0..total).step_by(rows) {
                    ui.keys.trade.view.grid_mut(which).set_first_row(row);
                    let mut draw = DrawList::default();
                    ui.keys.trade.view.paint(&mut draw, 0, 0, &font, 1.0);
                    let rect = ui.keys.trade.view.grid_mut(which).rect;
                    for entry in ui.keys.trade.view.entries(which).iter().skip(row as usize * 6).take(rows * 6) {
                        assert!(draw.quads.iter().any(|q| q.texture == entry.grid.icon() && q.y >= rect.y as f32 && q.y < (rect.y + rect.h) as f32));
                    }
                }
            }
        }
        let sample: Vec<_> = items.into_iter().filter(|i| [1, 17, 57, 1060].contains(&i.item_id)).collect();
        assert_eq!(sample.len(), 4);
        ui.trade_event(&WorldEvent::TradeStart(TradeStart { partner_id: 7, items: sample.clone() }), &world, Some(&tables));
        ui.trade_event(&WorldEvent::TradeOwnAdd(TradeAdd { items: sample[..2].to_vec() }), &world, Some(&tables));
        ui.trade_event(&WorldEvent::TradeOtherAdd(TradeAdd { items: sample[2..].to_vec() }), &world, Some(&tables));
        let mut draw = DrawList::default();
        ui.keys.trade.view.paint(&mut draw, 10, 10, &font, 1.0);
        let mut headless = Headless::new(400, 430).unwrap();
        let mut bank = TextureBank::new();
        headless.renderer_mut().set_ui(&draw, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        assert_eq!(bank.missing().count(), 0);
        let camera = Camera { eye: glam::vec3(0.0, -300.0, 0.0), target: glam::Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
        let rgba = headless.render(&camera, &Environment::default()).unwrap();
        write_png(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-trade.png"), 400, 430, &rgba).unwrap();
        eprintln!("{} itens nas três grades e captura GPU de troca", ids.len());
    }
}
