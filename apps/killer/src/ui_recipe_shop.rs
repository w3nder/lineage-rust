use crate::ui_keys::UiCommand;
use killer_assets::client::DatTables;
use killer_ui::dialog::{DialogKind, DialogView};
use killer_ui::item_grid::ItemGrid;
use killer_ui::recipe_shop::{self, RecipeShopHit as Hit, RecipeShopView, ShopEntry};
use killer_ui::scroll::{ScrollHit, line_down, line_up};
use killer_ui::window::{Desktop, WindowId};
use killer_world::{NetCommand, World, WorldEvent};

#[derive(Clone, Copy)]
enum Prompt {
    Price(usize),
    Message,
}

pub struct RecipeShopWindow {
    pub view: RecipeShopView,
    pub commands: Vec<UiCommand>,
    pub mp: i32,
    pub max_mp: i32,
    pending: bool,
    prompt: Option<Prompt>,
    answer: Option<Prompt>,
    message: String,
    drag_moved: bool,
}

impl RecipeShopWindow {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self {
            view: RecipeShopView::new(lookup),
            commands: Vec::new(),
            mp: 0,
            max_mp: 0,
            pending: false,
            prompt: None,
            answer: None,
            message: String::new(),
            drag_moved: false,
        }
    }

    pub fn world_event(&mut self, event: &WorldEvent) {
        if matches!(event, WorldEvent::RecipeShopManageList(_)) {
            self.pending = true;
        }
    }

    pub fn sync(&mut self, world: &World, tables: &DatTables, desktop: &mut Desktop) {
        self.message = world
            .me
            .as_ref()
            .and_then(|me| world.recipes.shop_messages.get(&me.object_id))
            .cloned()
            .unwrap_or_default();
        if !std::mem::take(&mut self.pending) {
            return;
        }
        let Some(data) = &world.recipes.shop_manage else { return };
        self.clear();
        (self.mp, self.max_mp) = (data.mp, data.max_mp);
        let entry = |id: i32, cost: i32| {
            let recipe = tables.recipes.get(&id)?;
            Some(ShopEntry {
                recipe_id: id,
                cost,
                grid: crate::ui_recipe::grid_entry(recipe.recipe_id, recipe.product_id, tables),
            })
        };
        let top = data.recipes.iter().filter_map(|(id, _)| entry(*id, 0)).collect();
        let bottom = data.items.iter().filter_map(|item| entry(item.recipe_id, item.cost)).collect();
        self.view.open(data.common, top, bottom);
        crate::ui_recipe::show_front(desktop, WindowId::RECIPE_SHOP);
    }

    pub fn clear(&mut self) {
        self.view.top.clear();
        self.view.bottom.clear();
        self.view.top_grid.clear();
        self.view.bottom_grid.clear();
        for grid in [&mut self.view.top_grid, &mut self.view.bottom_grid] {
            grid.scroll.up_pressed = false;
            grid.scroll.down_pressed = false;
            grid.scroll.slider.mouse_up();
        }
        self.view.pressed = None;
        self.prompt = None;
        self.answer = None;
        self.drag_moved = false;
    }

    fn grid_mut(&mut self, bottom: bool) -> &mut ItemGrid {
        if bottom { &mut self.view.bottom_grid } else { &mut self.view.top_grid }
    }

    fn price(&mut self, index: usize) {
        if index < self.view.top.len() {
            self.prompt = Some(Prompt::Price(index));
        }
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        self.drag_moved = false;
        self.view.pressed = self.view.hit(x, y);
        match self.view.pressed {
            Some(Hit::Top(i)) => self.view.top_grid.selected = Some(i),
            Some(Hit::Bottom(i)) => self.view.bottom_grid.selected = Some(i),
            Some(Hit::Scroll(bottom, hit)) => {
                let scroll = &mut self.grid_mut(bottom).scroll;
                scroll.up_pressed = hit == ScrollHit::Up;
                scroll.down_pressed = hit == ScrollHit::Down;
                if let ScrollHit::Slider(pos) = hit {
                    scroll.slider.mouse_down(pos);
                }
            }
            _ => {}
        }
    }

    pub fn mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        let pressed = self.view.pressed.take();
        let local = over.filter(|(id, _, _)| *id == WindowId::RECIPE_SHOP).map(|(_, x, y)| (x, y));
        let released = local.and_then(|(x, y)| self.view.hit(x, y));
        for grid in [&mut self.view.top_grid, &mut self.view.bottom_grid] {
            grid.scroll.up_pressed = false;
            grid.scroll.down_pressed = false;
            grid.scroll.slider.mouse_up();
        }
        self.drag_moved = false;
        match pressed {
            Some(Hit::Top(i)) if local.is_some_and(|(x, y)| recipe_shop::BOTTOM_RECT.contains(x, y)) => {
                self.price(i);
                return;
            }
            Some(Hit::Bottom(i)) if local.is_some_and(|(x, y)| recipe_shop::TOP_RECT.contains(x, y)) => {
                self.view.bottom_grid.selected = Some(i);
                self.view.remove_selected_production();
                return;
            }
            _ => {}
        }
        if pressed != released {
            return;
        }
        match pressed {
            Some(Hit::Down) => {
                if let Some(i) = self.view.top_grid.selected {
                    self.price(i);
                }
            }
            Some(Hit::Up) => self.view.remove_selected_production(),
            Some(Hit::Message) => self.prompt = Some(Prompt::Message),
            Some(Hit::Start | Hit::Abort | Hit::Close) => {
                let command = match pressed {
                    Some(Hit::Start) => NetCommand::RecipeShopListSet {
                        items: self.view.bottom.iter().map(|e| (e.recipe_id, e.cost)).collect(),
                    },
                    _ => NetCommand::RecipeShopManageQuit,
                };
                self.commands.push(UiCommand::Net(command));
                if pressed != Some(Hit::Start) {
                    self.commands.push(UiCommand::InterfaceSound("itemsound.window_close"));
                }
                self.clear();
                crate::ui_recipe::set_visible(desktop, WindowId::RECIPE_SHOP, false);
            }
            Some(Hit::Scroll(bottom, hit)) => {
                let count = if bottom { self.view.bottom.len() } else { self.view.top.len() };
                let grid = self.grid_mut(bottom);
                let (first, rows, total) = (grid.scroll_row as i32, grid.rows as i32, grid.total_rows(count));
                let next = match hit {
                    ScrollHit::Up => line_up(first, rows, total),
                    ScrollHit::Down => line_down(first, rows, total),
                    _ => first,
                };
                grid.set_first_row(next);
            }
            _ => {}
        }
    }

    pub fn double_click(&mut self, x: i32, y: i32) -> bool {
        self.view.pressed = None;
        match self.view.hit(x, y) {
            Some(Hit::Top(i)) => {
                self.view.top_grid.selected = Some(i);
                self.price(i);
            }
            Some(Hit::Bottom(i)) => {
                self.view.bottom_grid.selected = Some(i);
                self.view.remove_selected_production();
            }
            _ => {}
        }
        true
    }

    pub fn mouse_move(&mut self, y: i32) -> bool {
        let Some(Hit::Scroll(bottom, ScrollHit::Slider(_))) = self.view.pressed else { return false };
        let grid = self.grid_mut(bottom);
        if let Some(row) = grid.scroll.slider.mouse_move(y - grid.rect.y - killer_ui::scroll::BUTTON_SIZE) {
            grid.scroll_row = row.max(0) as usize;
        }
        true
    }

    pub fn hover(&mut self) {
        self.drag_moved |= matches!(self.view.pressed, Some(Hit::Top(_) | Hit::Bottom(_)));
    }

    pub fn drag_icon(&self) -> Option<String> {
        if !self.drag_moved {
            return None;
        }
        let entry = match self.view.pressed? {
            Hit::Top(i) => self.view.top.get(i),
            Hit::Bottom(i) => self.view.bottom.get(i),
            _ => None,
        }?;
        Some(entry.grid.icon.clone())
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        for bottom in [false, true] {
            let count = if bottom { self.view.bottom.len() } else { self.view.top.len() };
            let grid = self.grid_mut(bottom);
            if grid.rect.contains(x, y) {
                let first = killer_ui::inventory::item_wnd_wheel(
                    grid.scroll_row as i32,
                    grid.rows as i32,
                    grid.total_rows(count),
                    delta,
                );
                grid.set_first_row(first);
                return true;
            }
        }
        false
    }

    pub fn take_dialog(&mut self, tables: Option<&DatTables>) -> Option<DialogView> {
        let prompt = self.prompt.take()?;
        self.answer = Some(prompt);
        let (mut view, kind) = match prompt {
            Prompt::Price(i) => {
                let entry = self.view.top.get(i)?;
                let args = [l2_net::game::SystemMessageArg::Text(entry.grid.name.clone())];
                let text = crate::ui2d::system_text(tables, recipe_shop::PRICE_SYSMSG, &args);
                (
                    DialogView::with_count(DialogKind::OkCancel, &text, killer_ui::dialog::COUNT_LIMIT),
                    recipe_shop::PRICE_DIALOG_TYPE,
                )
            }
            Prompt::Message => {
                let text = crate::ui2d::system_text(tables, recipe_shop::MESSAGE_SYSMSG, &[]);
                let mut view = DialogView::with_text(DialogKind::OkCancel, &text);
                if let Some(input) = &mut view.input {
                    input.text = self.message.clone();
                }
                (view, recipe_shop::MESSAGE_DIALOG_TYPE)
            }
        };
        view.dialog_type = kind;
        Some(view)
    }

    pub fn answer_text(&mut self, text: Option<&str>) {
        let prompt = self.answer.take();
        let Some(text) = text else { return };
        match prompt {
            Some(Prompt::Price(_)) => {
                let text = text.trim_start();
                let count = if text.starts_with('-') {
                    0
                } else {
                    text.strip_prefix('+')
                        .unwrap_or(text)
                        .bytes()
                        .take_while(u8::is_ascii_digit)
                        .fold(0i64, |n, c| n.saturating_mul(10).saturating_add(i64::from(c - b'0')))
                };
                if count <= killer_ui::dialog::COUNT_LIMIT
                    && let Some(entry) = self.view.top_grid.selected.and_then(|i| self.view.top.get(i))
                {
                    self.view.add_production(entry.recipe_id, count as i32);
                }
            }
            Some(Prompt::Message) if !text.is_empty() => {
                self.commands.push(UiCommand::Net(NetCommand::RecipeShopMessageSet { message: text.into() }))
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::item_grid::GridEntry;

    fn shop(count: i32) -> RecipeShopWindow {
        let mut shop = RecipeShopWindow::new(|id| Some(format!("s{id}")));
        let entries = (0..count)
            .map(|id| ShopEntry {
                recipe_id: id + 1000,
                cost: 0,
                grid: GridEntry { item_id: id + 2000, name: format!("recipe{id}"), count: 1, ..Default::default() },
            })
            .collect();
        shop.view.open(false, entries, Vec::new());
        shop
    }

    fn click(shop: &mut RecipeShopWindow, rect: killer_ui::window::Rect) {
        shop.mouse_down(rect.x + 1, rect.y + 1);
        shop.mouse_up(Some((WindowId::RECIPE_SHOP, rect.x + 1, rect.y + 1)), &mut Desktop::new((1024, 768)));
    }

    #[test]
    fn recipe_shop_price_answer_goes_to_the_recipe_selected_when_it_arrives() {
        let mut shop = shop(2);
        shop.double_click(11, 50);
        assert!(shop.take_dialog(None).is_some());
        shop.view.top_grid.selected = Some(1);
        shop.answer_text(Some("600"));
        assert_eq!((shop.view.bottom.len(), shop.view.bottom[0].recipe_id, shop.view.bottom[0].cost), (1, 1001, 600));
    }

    #[test]
    fn recipe_shop_100f2850_accepts_zero_cancels_and_keeps_the_original_duplicate_price() {
        let mut shop = shop(2);
        for (text, expected) in [(None, None), (Some("-4"), Some(0)), (Some("600"), Some(0))] {
            shop.double_click(11, 50);
            let dialog = shop.take_dialog(None).unwrap();
            assert_eq!(dialog.dialog_type, 0x23);
            shop.answer_text(text);
            assert_eq!(shop.view.bottom.first().map(|e| e.cost), expected);
        }
        assert_eq!(shop.view.top[0].cost, 600);
        shop.double_click(48, 50);
        shop.take_dialog(None).unwrap();
        shop.answer_text(Some("2000000001"));
        assert_eq!(shop.view.bottom.len(), 1);
        shop.double_click(48, 50);
        shop.take_dialog(None).unwrap();
        shop.answer_text(Some("2000000000"));
        assert_eq!(shop.view.bottom[1].cost, 2_000_000_000);
    }

    #[test]
    fn recipe_shop_100f2560_drag_and_100f2700_double_click_use_the_same_transfer() {
        let mut shop = shop(2);
        let mut desktop = Desktop::new((1024, 768));
        shop.mouse_down(11, 50);
        shop.hover();
        assert!(shop.drag_icon().is_some());
        shop.mouse_up(Some((WindowId::RECIPE_SHOP, 11, 220)), &mut desktop);
        shop.take_dialog(None).unwrap();
        shop.answer_text(Some("123"));
        assert_eq!(shop.view.top.len(), 2);
        assert_eq!(shop.view.bottom[0].cost, 123);
        shop.mouse_down(11, 220);
        shop.mouse_up(Some((WindowId::RECIPE_SHOP, 11, 50)), &mut desktop);
        assert!(shop.view.bottom.is_empty());
        shop.double_click(11, 50);
        shop.take_dialog(None).unwrap();
        shop.answer_text(Some("456"));
        shop.double_click(11, 220);
        assert!(shop.view.bottom.is_empty());
    }

    #[test]
    fn recipe_shop_100f0250_100efab0_100efaf0_send_distinct_commands_and_clear() {
        for rect in [recipe_shop::START_RECT, recipe_shop::ABORT_RECT, killer_ui::window::Rect::new(237, 3, 14, 14)] {
            let mut shop = shop(1);
            shop.view.add_production(1000, 987);
            click(&mut shop, rect);
            assert!(shop.view.top.is_empty() && shop.view.bottom.is_empty());
            match &shop.commands[0] {
                UiCommand::Net(NetCommand::RecipeShopListSet { items }) => {
                    assert_eq!(items, &[(1000, 987)]);
                    assert_eq!(shop.commands.len(), 1);
                }
                UiCommand::Net(NetCommand::RecipeShopManageQuit) => {
                    assert!(matches!(shop.commands[1], UiCommand::InterfaceSound("itemsound.window_close")))
                }
                _ => panic!("comando inesperado"),
            }
        }
        let mut shop = shop(0);
        click(&mut shop, recipe_shop::START_RECT);
        assert!(
            matches!(&shop.commands[0], UiCommand::Net(NetCommand::RecipeShopListSet { items }) if items.is_empty())
        );
    }

    #[test]
    fn recipe_shop_100f0460_prefills_the_message_and_sends_only_accepted_nonempty_text() {
        let mut shop = shop(0);
        shop.message = "mensagem atual".into();
        for text in [None, Some(""), Some("fabricação C4")] {
            click(&mut shop, recipe_shop::MESSAGE_RECT);
            let dialog = shop.take_dialog(None).unwrap();
            assert_eq!(dialog.dialog_type, 0x24);
            assert_eq!(dialog.input.unwrap().text, "mensagem atual");
            shop.answer_text(text);
        }
        assert_eq!(shop.commands.len(), 1);
        assert!(
            matches!(&shop.commands[0], UiCommand::Net(NetCommand::RecipeShopMessageSet { message }) if message == "fabricação C4")
        );
    }

    #[test]
    fn recipe_shop_100f29c0_capacity_and_wheel_cover_both_grids_at_every_size() {
        for count in 0..=160 {
            let mut shop = shop(count);
            for id in 1000..1000 + count {
                shop.view.add_production(id, id);
            }
            assert_eq!(shop.view.bottom.len(), count as usize);
            for (x, y, bottom) in [(11, 50, false), (11, 220, true)] {
                shop.wheel(x, y, -120000);
                let grid = shop.grid_mut(bottom);
                assert_eq!(grid.scroll_row, (count as usize).div_ceil(6).saturating_sub(4));
                shop.wheel(x, y, 120000);
                assert_eq!(shop.grid_mut(bottom).scroll_row, 0);
            }
        }
    }
}
