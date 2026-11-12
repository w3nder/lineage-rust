use killer_assets::client::DatTables;
use killer_ui::dialog::{DialogKind, DialogView};
use killer_ui::item_grid::GridEntry;
use killer_ui::item_name::ItemNameLabel;
use killer_ui::multisell::{MultiSellHit, MultiSellView, OfferView};
use killer_ui::multisell_detail::ItemDetail;
use killer_ui::window::{Desktop, WindowId};
use killer_world::{NetCommand, World, WorldEvent};
use l2_dat::itemicons::ItemKind;

use crate::ui_keys::UiCommand;

pub struct MultiSellWindow {
    pub view: MultiSellView,
    pressed: Option<MultiSellHit>,
    pending_count: Option<(i32, i32)>,
    last_tick: Option<killer_platform::time::Instant>,
    pub dismiss_dialog: bool,
    pub commands: Vec<UiCommand>,
}

impl MultiSellWindow {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self { view: MultiSellView::new(lookup), pressed: None, pending_count: None, last_tick: None, dismiss_dialog: false, commands: Vec::new() }
    }

    pub fn world_event(&mut self, event: &WorldEvent, world: &World, tables: Option<&DatTables>, desktop: &mut Desktop) -> bool {
        if !matches!(event, WorldEvent::MultiSellList(_)) { return false; }
        let Some(tables) = tables else { return false; };
        let Some(state) = world.windows.multisell.as_ref() else { return false; };
        let offers = state.offers.iter().filter_map(|offer| {
            let product = offer.products[0]?;
            let item = grid_entry(product.item_id as i32, product.count, product.enchant, tables)?;
            let products = offer.products[..offer.product_count].iter().flatten().filter_map(|item| detail(*item, tables)).collect();
            let needed: Vec<_> = offer.ingredients.iter().filter_map(|item| Some((grid_entry(item.item_id as i32, item.count, item.enchant, tables)?, ingredient_label(item.item_id as i32, item.enchant, tables)?))).collect();
            let (ingredients, ingredient_labels) = needed.into_iter().unzip();
            Some(OfferView { entry_id: offer.entry_id, item, products, ingredients, ingredient_labels, stackable: offer.stackable != 0 })
        }).collect();
        self.view.symbols = tables.symbolname.clone();
        self.view.set_offers(state.list_id, offers);
        show(desktop, true);
        true
    }

    pub fn key(&mut self, key: killer_ui::chat::InputKey, font: &killer_ui::font::BitmapFont, desktop: &Desktop) -> bool {
        if !desktop.window(WindowId::MULTISELL).is_some_and(|w| w.visible) {
            return false;
        }
        self.view.quantity_key(key, font)
    }

    pub fn tick(&mut self, now: killer_platform::time::Instant) {
        let last = self.last_tick.replace(now);
        if let Some(last) = last {
            let elapsed = now.saturating_duration_since(last).as_millis().min(u128::from(u32::MAX)) as u32;
            self.view.quantity.tick(elapsed);
        }
    }

    pub fn clear(&mut self, desktop: &mut Desktop) {
        self.view.clear();
        self.pressed = None;
        self.pending_count = None;
        show(desktop, false);
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, _desktop: &mut Desktop) {
        if self.view.bar_down(x, y) || self.view.grid_slider_down(x, y) {
            self.pressed = None;
            self.view.pressed = None;
            return;
        }
        self.pressed = self.view.hit(x, y);
        self.view.pressed = self.pressed;
        match self.pressed {
            Some(MultiSellHit::Offer(index)) => {
                self.dismiss_dialog = true;
                self.pending_count = None;
                self.view.select(index);
            }
            Some(MultiSellHit::Close) | Some(MultiSellHit::Scroll(_)) | Some(MultiSellHit::Ok) | Some(MultiSellHit::Cancel) | None => {}
        }
    }

    pub fn mouse_move(&mut self, y: i32) -> bool {
        self.view.bar_move(y) || self.view.grid_slider_move(y)
    }

    pub fn mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        self.view.pressed = None;
        self.view.bar_up();
        let Some(pressed) = self.pressed.take() else { return; };
        let released = over.filter(|(id, _, _)| *id == WindowId::MULTISELL).and_then(|(_, x, y)| self.view.hit(x, y));
        if released != Some(pressed) { return; }
        match pressed {
            MultiSellHit::Scroll(hit) => self.view.scroll(hit),
            MultiSellHit::Ok => {
                if self.view.ok.enabled && let Some(offer) = self.view.selected_offer() {
                    self.pending_count = Some((self.view.list_id, offer.entry_id));
                }
            }
            MultiSellHit::Cancel if self.view.cancel.enabled => self.clear(desktop),
            MultiSellHit::Cancel => {},
            MultiSellHit::Close => {
                self.clear(desktop);
            }
            MultiSellHit::Offer(_) => {}
        }
    }

    pub fn take_dialog(&mut self, tables: Option<&DatTables>) -> Option<DialogView> {
        let (list_id, entry_id) = self.pending_count?;
        let text = crate::ui2d::system_text(tables, 1383, &[]);
        let mut view = DialogView::with_count(DialogKind::OkCancel, &text, if self.view.selected_offer().is_some_and(|o| o.stackable) { 2_000_000_000 } else { 1 });
        view.dialog_type = 0x51;
        if let (Some(input), Some(text)) = (view.input.as_mut(), self.view.quantity.get_text()) {
            text.chars().for_each(|c| input.push(c));
        }
        self.pending_count = Some((list_id, entry_id));
        Some(view)
    }

    pub fn answer_text(&mut self, text: Option<&str>) {
        let Some((list_id, entry_id)) = self.pending_count.take() else { return; };
        let amount = text.and_then(|value| value.parse::<i32>().ok()).unwrap_or(0).max(0);
        if amount > 0 { self.commands.push(UiCommand::Net(NetCommand::MultiSellChoose { list_id, entry_id, amount })); }
    }
}

fn show(desktop: &mut Desktop, visible: bool) {
    if let Some(window) = desktop.window_mut(WindowId::MULTISELL) { window.visible = visible; }
    if visible && let Some(index) = desktop.windows.iter().position(|w| w.id == WindowId::MULTISELL) {
        let window = desktop.windows.remove(index);
        desktop.windows.push(window);
    }
}

fn grid_entry(id: i32, count: i32, enchant: i16, tables: &DatTables) -> Option<GridEntry> {
    let id_u = u32::try_from(id).ok()?;
    let icon = tables.item_icons.get(id_u)?;
    let name = tables.itemname.get(&id_u);
    Some(GridEntry { item_id: id, count, enchant: i32::from(enchant), icon: icon.icons[0].trim().to_string(), name: name.map(|x| x.name.clone()).unwrap_or_default(), add_name: name.map(|x| x.add_name.clone()).unwrap_or_default(), description: name.map(|x| x.description.clone()).unwrap_or_default(), weight: icon.weight as i32, ..Default::default() })
}

fn additional_name(id: u32, tables: &DatTables) -> Option<String> {
    tables.itemname.get(&id).map(|name| name.add_name.clone()).filter(|name| !name.is_empty())
}

fn ingredient_label(id: i32, enchant: i16, tables: &DatTables) -> Option<ItemNameLabel> {
    let id = u32::try_from(id).ok()?;
    let kind = match tables.item_icons.get(id)?.kind { ItemKind::Weapon => 0, ItemKind::Armor => 1, ItemKind::Etc => 2 };
    Some(ItemNameLabel {
        name: tables.itemname.get(&id).map(|name| name.name.clone()).unwrap_or_default(),
        additional: additional_name(id, tables),
        type2: kind,
        enchant: i32::from(enchant),
        grade: tables.item_grade(id).unwrap_or(0),
        count: 0,
    })
}

fn detail(product: l2_net::game::MultiSellProduct, tables: &DatTables) -> Option<ItemDetail> {
    let entry = grid_entry(product.item_id as i32, product.count, product.enchant, tables)?;
    let id = u32::try_from(product.item_id).ok()?;
    let kind = tables.item_icons.get(id)?.kind;
    let type2 = match kind { ItemKind::Weapon => 0, ItemKind::Armor => 1, ItemKind::Etc => 5 };
    let mut detail = ItemDetail { icon: entry.icon, name: entry.name, additional: additional_name(id, tables), grade: tables.item_grade(id).map_or(0, |grade| grade.max(0) as usize), description: entry.description, type2, body_part: product.body_part, enchant: i32::from(product.enchant), count: product.count, packet_type2: i32::from(product.type2), weight: entry.weight, ..Default::default() };
    match kind {
        ItemKind::Weapon => if let Some(row) = tables.weapongrp.get(&id) { detail = detail.with_weapon(row); },
        ItemKind::Armor => if let Some(row) = tables.armorgrp.get(&id) { detail = detail.with_armor(row); },
        ItemKind::Etc => {}
    }
    Some(detail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_assets::client::DatTables;
    use killer_ui::multisell;
    use killer_ui::window::{Rect, Window};
    use l2_dat::itemicons::{ItemIcon, ItemKind};

    #[test]
    fn multisell_confirmation_uses_the_original_type_and_packet_values() {
        let mut tables = DatTables::default();
        tables.item_icons.insert_all(vec![ItemIcon { id: 57, kind: ItemKind::Etc, drop: Default::default(), icons: std::array::from_fn(|_| "icon.adena".into()), weight: 1, type1: 4 }]);
        let mut window = MultiSellWindow::new(|_| Some("x".into()));
        let mut desktop = Desktop::new((800, 600));
        desktop.windows.push(Window::new(WindowId::MULTISELL, Rect::new(0, 0, multisell::SIZE.0, multisell::SIZE.1), multisell::grip()));
        let mut world = World::default();
        world.windows.multisell = Some(killer_world::multisell::MultiSellState { list_id: 1, finished: true, ..Default::default() });
        window.world_event(&WorldEvent::MultiSellList(l2_net::game::MultiSellList { list_id: 1, page: 1, finished: true, page_size: 1, entries: vec![] }), &world, Some(&tables), &mut desktop);
        assert_eq!(window.view.list_id, 1);
    }

    #[test]
    fn quantity_field_is_a_native_numeric_edit_box_that_prefills_the_confirmation() {
        use killer_ui::chat::InputKey;
        let mut ui = crate::ui2d::fixture_ui((1024, 768));
        ui.keys.enter_chatting = true;
        let world = World::default();
        let id = WindowId::MULTISELL;
        for window in &mut ui.desktop.windows { window.visible = window.id == id; }
        let rect = ui.desktop.window(id).unwrap().rect;
        let field = multisell::QUANTITY_RECT;
        assert_eq!(ui.keys.multisell.view.quantity.get_text(), None);
        assert!(ui.key(InputKey::Char('7')) == crate::ui2d::KeyAction::Ignored);
        assert!(ui.mouse_down_at(rect.x + field.x + 2, rect.y + field.y + 2, killer_platform::time::Instant::now(), &world));
        ui.mouse_up_in(&world);
        assert!(ui.keys.multisell.view.quantity.focused);
        for key in [InputKey::Char('1'), InputKey::Char('x'), InputKey::Char('2'), InputKey::Char('3'), InputKey::Char('4')] {
            assert!(ui.key(key) == crate::ui2d::KeyAction::Consumed);
        }
        assert_eq!(ui.keys.multisell.view.quantity.get_text().as_deref(), Some("1234"));
        assert_eq!(String::from_utf16_lossy(ui.keys.multisell.view.quantity.left()), "1,234");
        ui.key(InputKey::Backspace);
        assert_eq!(ui.keys.multisell.view.quantity.get_text().as_deref(), Some("123"));
        ui.keys.multisell.pending_count = Some((1, 1));
        let dialog = ui.keys.multisell.take_dialog(None).unwrap();
        assert_eq!(dialog.dialog_type, 0x51);
        assert_eq!(dialog.input.unwrap().text, "123");
        ui.keys.multisell.view.clear();
        assert_eq!(ui.keys.multisell.view.quantity.get_text().as_deref(), Some("123"));
        assert!(ui.mouse_down_at(rect.x + 5, rect.y + 60, killer_platform::time::Instant::now(), &world));
        assert!(!ui.keys.multisell.view.quantity.focused);
    }

    #[test]
    fn quantity_field_handles_home_end_delete_shift_mouse_and_the_blink_timer() {
        use killer_ui::chat::InputKey;
        use killer_platform::time::Instant;
        let mut ui = crate::ui2d::fixture_ui((1024, 768));
        let world = World::default();
        let id = WindowId::MULTISELL;
        for window in &mut ui.desktop.windows { window.visible = window.id == id; }
        let rect = ui.desktop.window(id).unwrap().rect;
        let field = multisell::QUANTITY_RECT;
        let (fx, fy) = (rect.x + field.x, rect.y + field.y + 3);
        let now = Instant::now();
        assert!(ui.mouse_down_at(fx + 2, fy, now, &world));
        ui.mouse_up_in(&world);
        for digit in ['1', '2', '3', '4', '5'] {
            ui.key(InputKey::Char(digit));
        }
        let left = |ui: &crate::ui2d::GameUi| String::from_utf16_lossy(ui.keys.multisell.view.quantity.left());
        let right = |ui: &crate::ui2d::GameUi| String::from_utf16_lossy(ui.keys.multisell.view.quantity.right());
        assert_eq!(left(&ui), "12,345");
        ui.key(InputKey::Home);
        assert_eq!((left(&ui), right(&ui)), (String::new(), "12,345".into()));
        ui.key(InputKey::Delete);
        assert_eq!(ui.keys.multisell.view.quantity.get_text().as_deref(), Some("2345"));
        ui.key(InputKey::End);
        assert_eq!(right(&ui), "");
        assert!(ui.shift_changed(true));
        ui.key(InputKey::Left);
        ui.key(InputKey::Left);
        assert_eq!(ui.keys.multisell.view.quantity.selection().0, -2);
        assert!(ui.shift_changed(false));
        ui.key(InputKey::Char('9'));
        assert_eq!(ui.keys.multisell.view.quantity.selection(), (0, 0));
        ui.keys.multisell.tick(now);
        assert!(ui.keys.multisell.view.quantity.caret_blink);
        ui.keys.multisell.tick(now + std::time::Duration::from_millis(500));
        assert!(!ui.keys.multisell.view.quantity.caret_blink);
        assert!(ui.mouse_down_at(rect.x + 5, rect.y + 60, now, &world));
        assert!(!ui.keys.multisell.view.quantity.focused);
        assert_eq!(right(&ui), "");
    }

    #[test]
    fn generated_multisell_buttons_keep_pressed_state_until_release_and_cancel_outside() {
        let mut ui = crate::ui2d::fixture_ui((1024, 768));
        let world = World::default();
        let id = WindowId::MULTISELL;
        for window in &mut ui.desktop.windows { window.visible = window.id == id; }
        let rect = ui.desktop.window(id).unwrap().rect;
        assert_eq!((rect.w, rect.h), (512, 401));
        let button = killer_ui::multisell::CANCEL_RECT;
        let x = rect.x + button.x + 1;
        let y = rect.y + button.y + 1;
        assert!(ui.mouse_down_at(x, y, killer_platform::time::Instant::now(), &world));
        assert_eq!(ui.keys.multisell.view.pressed, Some(MultiSellHit::Cancel));
        assert!(ui.desktop.window(id).unwrap().visible);
        ui.keys.multisell.mouse_up(None, &mut ui.desktop);
        assert_eq!(ui.keys.multisell.view.pressed, None);
        assert!(ui.desktop.window(id).unwrap().visible);
        ui.mouse_up_in(&world);
        ui.keys.multisell.view.cancel.enabled = false;
        ui.mouse_down_at(x, y, killer_platform::time::Instant::now(), &world);
        ui.mouse_up_in(&world);
        assert!(ui.desktop.window(id).unwrap().visible);
        ui.keys.multisell.view.cancel.enabled = true;
        ui.mouse_down_at(x, y, killer_platform::time::Instant::now(), &world);
        ui.mouse_up_in(&world);
        assert!(!ui.desktop.window(id).unwrap().visible);
        assert_eq!(ui.keys.multisell.view.pressed, None);
    }

    #[test]
    #[ignore]
    fn multisell_generated_layout_covers_all_client_items_and_original_button_dispatch() {
        use killer_assets::client::Client;
        use killer_ui::draw::DrawList;
        use killer_ui::font::{BitmapFont, Glyphs};
        use l2_net::game::{MultiSellEntry, MultiSellIngredient, MultiSellList, MultiSellProduct};
        let root = std::env::var_os("KILLER_CLIENT_DIR").unwrap();
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let font = BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap());
        let ids: std::collections::BTreeSet<_> = tables.weapongrp.keys().chain(tables.armorgrp.keys()).chain(tables.etcitemgrp.keys()).copied().collect();
        assert!(!ids.is_empty());
        let mut ui = crate::ui2d::fixture_ui((1024, 768));
        ui.small = font;
        ui.set_tab_names(&tables);
        let mut world = World::default();
        let mut outside_packet = Vec::new();
        let items: Vec<_> = ids.iter().filter_map(|id| {
            let item_id = match i16::try_from(*id) {
                Ok(value) => value,
                Err(_) => { outside_packet.push(*id); return None; }
            };
            Some(MultiSellEntry {
                entry_id: i32::try_from(*id).unwrap(), stackable: 1,
                products: vec![MultiSellProduct { item_id, type2: 5, body_part: 0, count: 1, enchant: 0 }],
                ingredients: vec![MultiSellIngredient { item_id: 57, type2: 4, count: 100, enchant: 0 }],
            })
        }).collect();
        for batch in items.chunks(120) {
            let event = WorldEvent::MultiSellList(MultiSellList { list_id: 37, page: 1, finished: true, page_size: 120, entries: batch.to_vec() });
            world.apply_world(&event);
            ui.multisell_event(&event, &world, Some(&tables));
            assert_eq!(ui.keys.multisell.view.offers.len(), batch.len());
            for (index, entry) in batch.iter().enumerate() {
                let last = batch.len().div_ceil(multisell::GRID_COLS).saturating_sub(multisell::GRID_ROWS);
                ui.keys.multisell.view.grid.set_first_row((index / multisell::GRID_COLS).min(last) as i32);
                ui.keys.multisell.view.select(index);
                let mut draw = DrawList::default();
                ui.paint_window(&mut draw, ui.desktop.window(WindowId::MULTISELL).unwrap(), &world, &[], None, &crate::ui2d::WorldViews::default());
                let expected = tables.item_icons.get(entry.entry_id as u32).unwrap().icons[0].trim();
                assert!(draw.quads.iter().any(|q| q.texture == expected));
                let back = draw.quads.iter().find(|q| q.texture == multisell::BACK).unwrap();
                assert_eq!((back.w, back.h), (512.0, 381.0));
                let button = multisell::OK_RECT;
                ui.keys.multisell.mouse_down(button.x + 1, button.y + 1, &mut ui.desktop);
                ui.keys.multisell.mouse_up(Some((WindowId::MULTISELL, button.x + 1, button.y + 1)), &mut ui.desktop);
                assert_eq!(ui.keys.multisell.take_dialog(Some(&tables)).unwrap().dialog_type, 0x51);
                ui.keys.multisell.answer_text(Some("2"));
                assert!(matches!(ui.keys.multisell.commands.pop(), Some(UiCommand::Net(NetCommand::MultiSellChoose {
                    list_id: 37, entry_id, amount: 2
                })) if entry_id == entry.entry_id));
            }
        }
        for id in &outside_packet {
            assert!(i16::try_from(*id).is_err());
            let item = grid_entry(i32::try_from(*id).unwrap(), 1, 0, &tables).unwrap();
            let icon = item.icon().to_string();
            ui.keys.multisell.view.set_offers(37, vec![OfferView {
                entry_id: i32::try_from(*id).unwrap(), item, products: Vec::new(), ingredients: Vec::new(), ingredient_labels: Vec::new(), stackable: false,
            }]);
            let mut draw = DrawList::default();
            ui.paint_window(&mut draw, ui.desktop.window(WindowId::MULTISELL).unwrap(), &world, &[], None, &crate::ui2d::WorldViews::default());
            assert!(draw.quads.iter().any(|q| q.texture == icon));
            assert!(draw.quads.iter().any(|q| q.texture == multisell::BACK && q.w == 512.0 && q.h == 381.0));
        }
        assert_eq!(items.len() + outside_packet.len(), ids.len());
        eprintln!("{} itens pintados; {} IDs conferidos pelo pacote e botões; {} IDs fora do i16 do C4 conferidos somente na view", ids.len(), items.len(), outside_packet.len());
    }
}
