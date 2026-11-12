use killer_assets::client::DatTables;
use killer_ui::item_grid::GridEntry;
use killer_ui::recipe::{BookEntry, BookHit, ManufactureHit, Material, RecipeBookView, RecipeManufactureView};
use killer_ui::recipe_tree::{RecipeNode, RecipeTreeView};
use killer_ui::scroll::ScrollHit;
use killer_ui::window::{Desktop, WindowId};
use killer_world::{NetCommand, World, WorldEvent};

use crate::ui_keys::UiCommand;

pub struct RecipeWindows {
    pub book: RecipeBookView,
    pub manufacture: RecipeManufactureView,
    pub tree: RecipeTreeView,
    pub shop: crate::ui_recipe_shop::RecipeShopWindow,
    tree_pending: Option<i32>,
    pub sell: killer_ui::recipe_sell::RecipeSellView,
    pub buy: RecipeManufactureView,
    buy_manufacturer: i32,
    buy_cost: i32,
    sell_pending: bool,
    buy_pending: bool,
    pub commands: Vec<UiCommand>,
    book_pending: bool,
    make_pending: bool,
    pub pressed_book: Option<BookHit>,
}

pub(crate) fn grid_entry(item_id: i32, icon_id: i32, tables: &DatTables) -> GridEntry {
    let name = u32::try_from(item_id).ok().and_then(|id| tables.itemname.get(&id));
    let icon = u32::try_from(icon_id).ok().and_then(|id| tables.item_icons.get(id));
    GridEntry {
        item_id,
        count: 1,
        icon: icon.map(|i| i.icons[0].trim().to_string()).unwrap_or_default(),
        name: name.map(|n| n.name.clone()).unwrap_or_default(),
        add_name: name.map(|n| n.add_name.clone()).unwrap_or_default(),
        description: name.map(|n| n.description.clone()).unwrap_or_default(),
        weight: u32::try_from(item_id).ok().and_then(|id| tables.item_icons.get(id)).map_or(0, |i| i.weight as i32),
        ..Default::default()
    }
}

fn page(world: &World, quest: bool) -> Vec<&l2_net::game::InventoryItem> {
    world.inventory.items.iter().filter(|item| !item.equipped && (item.type2 == killer_ui::inventory::QUEST_TYPE2) == quest).collect()
}

fn page0_owned(world: &World, item_id: i32) -> i32 {
    page(world, false).iter().filter(|item| item.item_id == item_id).map(|item| item.count).sum()
}

fn owned(world: &World, item_id: i32) -> i32 {
    page0_owned(world, item_id) + page(world, true).iter().filter(|item| item.item_id == item_id).map(|item| item.count).sum::<i32>()
}

fn buyer_retained(world: &World, item_id: i32) -> i32 {
    let first = page(world, false);
    let quest = page(world, true);
    let second: i32 = quest
        .iter()
        .enumerate()
        .filter(|(_, item)| item.item_id == item_id)
        .map(|(index, _)| first.get(index).map_or(0, |entry| entry.count))
        .sum();
    page0_owned(world, item_id) + second
}

fn build_recipe_node_100f59d0(
    item_id: i32,
    rate: i32,
    needed: i32,
    world: &World,
    tables: &DatTables,
    ancestors: &mut Vec<(i32, i32)>,
) -> Result<Option<RecipeNode>, i32> {
    let Some(id) = u32::try_from(item_id).ok() else { return Ok(None) };
    if tables.item_icons.get(id).is_none() {
        return Ok(None);
    }
    if ancestors.contains(&(item_id, rate)) {
        return Err(item_id);
    }
    let recipe = tables
        .recipe_order
        .iter()
        .filter_map(|id| tables.recipes.get(id))
        .find(|r| r.product_id == item_id && r.success_rate == rate);
    let mut node = RecipeNode {
        item: grid_entry(item_id, item_id, tables),
        needed,
        owned: page0_owned(world, item_id),
        expanded: false,
        children: Vec::new(),
        name_width: 0,
    };
    if let Some(recipe) = recipe {
        ancestors.push((item_id, rate));
        for material in &recipe.materials {
            if let Some(child) =
                build_recipe_node_100f59d0(material.item_id, 100, material.count, world, tables, ancestors)?
            {
                node.children.push(child);
            }
        }
        ancestors.pop();
    }
    Ok(Some(node))
}

impl RecipeWindows {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self {
            book: RecipeBookView::new(&lookup),
            manufacture: RecipeManufactureView { wrap_status: true, ..RecipeManufactureView::new(&lookup) },
            tree: RecipeTreeView::new(&lookup),
            shop: crate::ui_recipe_shop::RecipeShopWindow::new(&lookup),
            tree_pending: None,
            sell: killer_ui::recipe_sell::RecipeSellView::new(&lookup),
            buy: RecipeManufactureView::new(&lookup),
            buy_manufacturer: 0,
            buy_cost: 0,
            sell_pending: false,
            buy_pending: false,
            commands: Vec::new(),
            book_pending: false,
            make_pending: false,
            pressed_book: None,
        }
    }

    pub fn world_event(&mut self, event: &WorldEvent) {
        self.shop.world_event(event);
        match event {
            WorldEvent::RecipeShopSellList(_) => self.sell_pending = true,
            WorldEvent::RecipeShopItemInfo(_) => self.buy_pending = true,
            WorldEvent::RecipeBookItemList(_) => self.book_pending = true,
            WorldEvent::RecipeItemMakeInfo(_) => self.make_pending = true,
            WorldEvent::StorageMaxCount(limits) => {
                self.book.dwarf_limit = limits.values[5];
                self.book.common_limit = limits.values[6];
            }
            _ => {}
        }
    }

    pub fn sync(&mut self, world: &World, tables: Option<&DatTables>, desktop: &mut Desktop) {
        let Some(tables) = tables else { return };
        self.shop.sync(world, tables, desktop);
        if std::mem::take(&mut self.book_pending)
            && let Some(book) = &world.recipes.book
        {
            self.open_book(book, tables, desktop);
        }
        if std::mem::take(&mut self.make_pending)
            && let Some(info) = world.recipes.make_info
        {
            self.open_manufacture(info, world, tables, desktop);
        }
        if std::mem::take(&mut self.sell_pending)
            && let Some(data) = &world.recipes.shop_sell
        {
            self.sell.clear();
            self.sell.manufacturer = data.manufacturer_id;
            (self.sell.mp, self.sell.max_mp, self.sell.adena) = (data.mp, data.max_mp, data.adena);
            self.sell.entries = data
                .items
                .iter()
                .filter_map(|item| {
                    let recipe = tables.recipes.get(&item.recipe_id)?;
                    Some(killer_ui::recipe_sell::SellEntry {
                        recipe_id: recipe.id,
                        product: grid_entry(recipe.product_id, recipe.product_id, tables),
                        cost: item.cost,
                        success: recipe.success_rate,
                    })
                })
                .collect();
            self.sell.update_scroll();
            copy_position(desktop, WindowId::RECIPE_BUY, WindowId::RECIPE_SELL);
            show_front(desktop, WindowId::RECIPE_SELL);
        }
        if std::mem::take(&mut self.buy_pending)
            && let Some(info) = world.recipes.shop_item
            && tables.recipes.contains_key(&info.recipe_id)
        {
            self.sell.clear();
            set_visible(desktop, WindowId::RECIPE_SELL, false);
            self.buy_manufacturer = info.manufacturer_id;
            self.buy_cost = info.cost;
            fill_manufacture(
                &mut self.buy,
                l2_net::game::RecipeItemMakeInfo {
                    recipe_id: info.recipe_id,
                    common: false,
                    mp: info.mp,
                    max_mp: info.max_mp,
                    status: info.status,
                },
                world,
                tables,
            );
            self.buy.retained = buyer_retained(world, self.buy.product_id);
            self.buy.frame = killer_ui::frame::FrameCtrl::titled(
                tables.sysstring.get(&663).map(String::as_str).unwrap_or_default(),
                true,
            );
            copy_position(desktop, WindowId::RECIPE_SELL, WindowId::RECIPE_BUY);
            show_front(desktop, WindowId::RECIPE_BUY);
        }
        if let Some(recipe_id) = self.tree_pending.take() {
            self.open_tree(recipe_id, world, tables, desktop);
        }
        if desktop.window(WindowId::RECIPE_BUY).is_some_and(|w| w.visible) {
            for material in &mut self.buy.materials {
                material.owned = owned(world, material.item_id);
            }
            self.buy.retained = buyer_retained(world, self.buy.product_id);
        }
        if desktop.window(WindowId::RECIPE_MANUFACTURE).is_some_and(|w| w.visible) {
            for material in &mut self.manufacture.materials {
                material.owned = owned(world, material.item_id);
            }
            self.manufacture.retained = owned(world, self.manufacture.product_id);
        }
    }

    fn open_book(&mut self, book: &l2_net::game::RecipeBookItemList, tables: &DatTables, desktop: &mut Desktop) {
        self.book_mouse_up(None);
        self.book.open(i32::from(book.common), book.max_mp);
        for (recipe_id, _) in &book.recipes {
            let Some(recipe) = tables.recipes.get(recipe_id) else { continue };
            self.book.push(BookEntry {
                recipe_id: recipe.id,
                grid: grid_entry(recipe.recipe_id, recipe.product_id, tables),
            });
        }
        if let Some((x, y)) = desktop.window(WindowId::RECIPE_MANUFACTURE).map(|w| (w.rect.x, w.rect.y))
            && let Some(window) = desktop.window_mut(WindowId::RECIPE_BOOK)
        {
            (window.rect.x, window.rect.y) = (x, y);
        }
        show_front(desktop, WindowId::RECIPE_BOOK);
    }

    fn open_manufacture(
        &mut self,
        info: l2_net::game::RecipeItemMakeInfo,
        world: &World,
        tables: &DatTables,
        desktop: &mut Desktop,
    ) {
        if !tables.recipes.contains_key(&info.recipe_id) {
            return;
        }
        set_visible(desktop, WindowId::RECIPE_BOOK, false);
        fill_manufacture(&mut self.manufacture, info, world, tables);
        if let Some((x, y)) = desktop.window(WindowId::RECIPE_BOOK).map(|w| (w.rect.x, w.rect.y))
            && let Some(window) = desktop.window_mut(WindowId::RECIPE_MANUFACTURE)
        {
            (window.rect.x, window.rect.y) = (x, y);
        }
        show_front(desktop, WindowId::RECIPE_MANUFACTURE);
    }

    fn open_tree(&mut self, recipe_id: i32, world: &World, tables: &DatTables, desktop: &mut Desktop) {
        self.tree.clear();
        if let Some(recipe) = tables.recipes.get(&recipe_id) {
            match build_recipe_node_100f59d0(recipe.product_id, recipe.success_rate, 0, world, tables, &mut Vec::new())
            {
                Ok(root) => self.tree.root = root,
                Err(item) => {
                    self.commands.push(UiCommand::Log(format!("ciclo nos materiais da receita: item {item}")));
                    return;
                }
            }
            self.tree.product = grid_entry(recipe.product_id, recipe.product_id, tables);
            let additional = &self.tree.product.add_name;
            if !additional.is_empty() && additional != "NAME_None" {
                self.tree.product.name = format!("{}-{additional}", self.tree.product.name);
            }
            self.tree.grade_texture = u32::try_from(recipe.product_id)
                .ok()
                .and_then(|id| tables.item_grade(id))
                .and_then(killer_ui::item_name::grade_symbol_1006b5a0)
                .and_then(|alias| tables.symbolname.by_alias(alias))
                .map(|symbol| symbol.texture.clone());
            self.tree.mp = recipe.mp_consume;
            self.tree.success = recipe.success_rate;
            self.tree.level = recipe.level;
            self.tree.update_scroll();
        }
        show_front(desktop, WindowId::RECIPE_TREE);
    }

    pub fn tree_mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        let over = over.filter(|(id, _, _)| *id == WindowId::RECIPE_TREE).map(|(_, x, y)| (x, y));
        if self.tree.mouse_up(over) {
            self.tree.clear();
            set_visible(desktop, WindowId::RECIPE_TREE, false);
            self.commands.push(UiCommand::InterfaceSound("itemsound.window_close"));
        }
    }

    pub fn book_mouse_down(&mut self, x: i32, y: i32, double: bool, desktop: &mut Desktop) -> Option<i32> {
        let hit = self.book.hit(x, y);
        self.pressed_book = hit;
        match hit? {
            BookHit::Close => {
                set_visible(desktop, WindowId::RECIPE_BOOK, false);
                None
            }
            BookHit::Slot(index) => {
                self.book.grid.selected = Some(index);
                let recipe_id = self.book.entries.get(index)?.recipe_id;
                if double {
                    self.commands.push(UiCommand::Net(NetCommand::RecipeItemMakeInfo { recipe_id }));
                    self.book.grid.selected = None;
                    return None;
                }
                Some(recipe_id)
            }
            BookHit::Scroll(hit) => {
                let grid = &mut self.book.grid;
                grid.scroll.up_pressed = hit == ScrollHit::Up;
                grid.scroll.down_pressed = hit == ScrollHit::Down;
                if let ScrollHit::Slider(pos) = hit {
                    grid.scroll.slider.mouse_down(pos);
                }
                None
            }
            BookHit::Trash => None,
        }
    }

    pub fn book_mouse_move(&mut self, y: i32) -> bool {
        if !matches!(self.pressed_book, Some(BookHit::Scroll(ScrollHit::Slider(_)))) {
            return false;
        }
        let grid = &mut self.book.grid;
        if let Some(first) = grid.scroll.slider.mouse_move(y - grid.rect.y - killer_ui::scroll::BUTTON_SIZE) {
            grid.scroll_row = first.max(0) as usize;
        }
        true
    }

    pub fn book_mouse_up(&mut self, over: Option<(WindowId, i32, i32)>) {
        let pressed = self.pressed_book.take();
        let released = match over {
            Some((WindowId::RECIPE_BOOK, x, y)) => self.book.hit(x, y),
            _ => None,
        };
        let grid = &mut self.book.grid;
        grid.scroll.up_pressed = false;
        grid.scroll.down_pressed = false;
        grid.scroll.slider.mouse_up();
        if pressed != released {
            return;
        }
        let (first, visible, total) =
            (grid.scroll_row as i32, grid.rows as i32, grid.total_rows(self.book.entries.len()));
        match pressed {
            Some(BookHit::Scroll(ScrollHit::Up)) => {
                grid.set_first_row(killer_ui::scroll::line_up(first, visible, total))
            }
            Some(BookHit::Scroll(ScrollHit::Down)) => {
                grid.set_first_row(killer_ui::scroll::line_down(first, visible, total))
            }
            _ => {}
        }
    }

    pub fn book_wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        let grid = &mut self.book.grid;
        if !grid.rect.contains(x, y) {
            return false;
        }
        let first = killer_ui::inventory::item_wnd_wheel(
            grid.scroll_row as i32,
            grid.rows as i32,
            grid.total_rows(self.book.entries.len()),
            delta,
        );
        grid.set_first_row(first);
        true
    }

    pub fn drag_over(&mut self, over: Option<(WindowId, i32, i32)>) {
        self.book.trash_drag =
            matches!(over, Some((WindowId::RECIPE_BOOK, x, y)) if self.book.hit(x, y) == Some(BookHit::Trash));
    }

    pub fn dropped(&mut self, recipe_id: i32, over: Option<(WindowId, i32, i32)>) {
        self.book.trash_drag = false;
        if let Some((WindowId::RECIPE_BOOK, x, y)) = over
            && self.book.hit(x, y) == Some(BookHit::Trash)
            && let Some(entry) = self.book.entries.iter().find(|entry| entry.recipe_id == recipe_id)
        {
            self.commands.push(UiCommand::ConfirmDeleteRecipe { recipe_id, name: entry.grid.name.clone() });
        }
    }

    pub fn sell_down(&mut self, x: i32, y: i32) {
        if let Some(recipe_id) = self.sell.down(x, y) {
            self.commands.push(UiCommand::Net(NetCommand::RecipeShopMakeInfo {
                manufacturer_id: self.sell.manufacturer,
                recipe_id,
            }));
        }
    }

    pub fn sell_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        if self.sell.up(over.filter(|(id, _, _)| *id == WindowId::RECIPE_SELL).map(|(_, x, y)| (x, y))) {
            set_visible(desktop, WindowId::RECIPE_SELL, false);
            self.commands.push(UiCommand::InterfaceSound("itemsound.window_close"));
        }
    }

    pub fn buy_down(&mut self, x: i32, y: i32) {
        self.buy.pressed = self.buy.hit(x, y);
        self.buy.tree_hover = self.buy.pressed == Some(ManufactureHit::Tree);
    }

    pub fn buy_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        self.buy.tree_hover = false;
        let Some(pressed) = self.buy.pressed.take() else {
            return;
        };
        if over.filter(|(id, _, _)| *id == WindowId::RECIPE_BUY).and_then(|(_, x, y)| self.buy.hit(x, y))
            != Some(pressed)
        {
            return;
        }
        match pressed {
            ManufactureHit::Close | ManufactureHit::CloseButton => {
                set_visible(desktop, WindowId::RECIPE_BUY, false);
            }
            ManufactureHit::Back => {
                set_visible(desktop, WindowId::RECIPE_BUY, false);
                self.commands
                    .push(UiCommand::Net(NetCommand::RecipeShopSellList { manufacturer_id: self.buy_manufacturer }));
                self.commands.push(UiCommand::InterfaceSound("itemsound.window_close"));
            }
            ManufactureHit::Create => self.commands.push(UiCommand::Net(NetCommand::RecipeShopMakeDo {
                manufacturer_id: self.buy_manufacturer,
                recipe_id: self.buy.recipe_id,
                cost: self.buy_cost,
            })),
            ManufactureHit::Tree => {
                if desktop.window(WindowId::RECIPE_TREE).is_some_and(|w| w.visible) {
                    set_visible(desktop, WindowId::RECIPE_TREE, false);
                } else {
                    self.tree_pending = Some(self.buy.recipe_id);
                }
            }
            ManufactureHit::Material(_) => {}
        }
    }

    pub fn manufacture_mouse_down(&mut self, x: i32, y: i32) {
        self.manufacture.pressed = self.manufacture.hit(x, y);
        self.manufacture.tree_hover = self.manufacture.pressed == Some(ManufactureHit::Tree);
    }

    pub fn manufacture_mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        self.manufacture.tree_hover = false;
        let Some(pressed) = self.manufacture.pressed.take() else { return };
        let released = match over {
            Some((WindowId::RECIPE_MANUFACTURE, x, y)) => self.manufacture.hit(x, y),
            _ => None,
        };
        if released != Some(pressed) {
            return;
        }
        match pressed {
            ManufactureHit::Close | ManufactureHit::CloseButton => {
                set_visible(desktop, WindowId::RECIPE_MANUFACTURE, false);
                self.commands.push(UiCommand::InterfaceSound("itemsound.window_close"));
            }
            ManufactureHit::Back => {
                set_visible(desktop, WindowId::RECIPE_MANUFACTURE, false);
                if (0..=1).contains(&self.manufacture.book_type) {
                    self.commands
                        .push(UiCommand::Net(NetCommand::RecipeBookOpen { common: self.manufacture.book_type == 1 }));
                }
                self.commands.push(UiCommand::InterfaceSound("itemsound.window_close"));
            }
            ManufactureHit::Create => {
                self.commands
                    .push(UiCommand::Net(NetCommand::RecipeItemMakeSelf { recipe_id: self.manufacture.recipe_id }));
            }
            ManufactureHit::Tree => {
                if desktop.window(WindowId::RECIPE_TREE).is_some_and(|w| w.visible) {
                    set_visible(desktop, WindowId::RECIPE_TREE, false);
                } else {
                    self.tree_pending = Some(self.manufacture.recipe_id);
                }
            }
            ManufactureHit::Material(_) => {}
        }
    }
}

fn copy_position(desktop: &mut Desktop, from: WindowId, to: WindowId) {
    if let Some((x, y)) = desktop.window(from).map(|w| (w.rect.x, w.rect.y))
        && let Some(w) = desktop.window_mut(to)
    {
        (w.rect.x, w.rect.y) = (x, y);
    }
}

fn fill_manufacture(
    view: &mut RecipeManufactureView,
    info: l2_net::game::RecipeItemMakeInfo,
    world: &World,
    tables: &DatTables,
) {
    let Some(recipe) = tables.recipes.get(&info.recipe_id) else {
        return;
    };
    view.clear();
    view.recipe_id = recipe.id;
    view.product_id = recipe.product_id;
    view.product_num = recipe.product_num;
    view.mp_consume = recipe.mp_consume;
    view.success_rate = recipe.success_rate;
    let product = grid_entry(recipe.product_id, recipe.product_id, tables);
    view.product_grade_texture = u32::try_from(recipe.product_id)
        .ok()
        .and_then(|id| tables.item_grade(id))
        .and_then(killer_ui::item_name::grade_symbol_1006b5a0)
        .and_then(|alias| tables.symbolname.by_alias(alias))
        .map(|symbol| symbol.texture.clone());
    view.product_name = product.name.clone();
    view.product_icon = product.icon.clone();
    view.materials = recipe
        .materials
        .iter()
        .map(|material| Material {
            item_id: material.item_id,
            needed: material.count,
            owned: owned(world, material.item_id),
            grid: grid_entry(material.item_id, material.item_id, tables),
        })
        .collect();
    view.mp = info.mp;
    view.max_mp = info.max_mp;
    view.status = info.status;
    view.book_type = i32::from(info.common);
    view.retained = owned(world, recipe.product_id);
    let message = |id: u32, args: &[String]| {
        let args: Vec<_> = args.iter().map(|value| l2_net::game::SystemMessageArg::Text(value.clone())).collect();
        crate::ui2d::system_text(Some(tables), id, &args)
    };
    view.status_text = match info.status {
        0 => message(killer_ui::recipe::FAILURE_SYSMSG, std::slice::from_ref(&product.name)),
        1 => message(killer_ui::recipe::SUCCESS_SYSMSG, &[product.name.clone(), recipe.product_num.to_string()]),
        _ => String::new(),
    };
    let title_id =
        if info.common { killer_ui::recipe::COMMON_TITLE_SYSSTRING } else { killer_ui::recipe::DWARF_TITLE_SYSSTRING };
    if let Some(title) = tables.sysstring.get(&title_id) {
        view.frame = killer_ui::frame::FrameCtrl::titled(title, true);
    }
}

pub(crate) fn set_visible(desktop: &mut Desktop, id: WindowId, visible: bool) {
    if let Some(window) = desktop.window_mut(id) {
        window.visible = visible;
    }
}

pub(crate) fn show_front(desktop: &mut Desktop, id: WindowId) {
    set_visible(desktop, id, true);
    if let Some(index) = desktop.windows.iter().position(|w| w.id == id) {
        let window = desktop.windows.remove(index);
        desktop.windows.push(window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::window::{Rect, Window};
    use l2_dat::recipe::{Recipe, RecipeMaterial};

    fn tables() -> DatTables {
        let mut tables = DatTables::default();
        tables.recipes.insert(
            1,
            Recipe {
                name: "mk_arrow".into(),
                id: 1,
                recipe_id: 1666,
                level: 1,
                product_id: 17,
                product_num: 500,
                mp_consume: 30,
                success_rate: 100,
                materials: vec![RecipeMaterial { item_id: 1864, count: 4 }],
            },
        );
        for (id, name) in [(1666, "Recipe: Wooden Arrow"), (17, "Wooden Arrow"), (1864, "Stem")] {
            tables.itemname.insert(
                id,
                l2_dat::itemname::ItemName {
                    id,
                    name: name.into(),
                    add_name: String::new(),
                    description: String::new(),
                    popup: 0,
                },
            );
        }
        tables.systemmsg.insert(
            959,
            l2_dat::SystemMessage {
                id: 959,
                present: 1,
                text: "$s1 (*$s2) manufacturing success.".into(),
                group: 0,
                color: [0; 3],
                alpha: 0xff,
                sound: String::new(),
                second_sound: String::new(),
            },
        );
        tables
    }

    fn desktop() -> Desktop {
        let mut desktop = Desktop::new((1558, 933));
        for (id, x) in [
            (WindowId::RECIPE_BOOK, 100),
            (WindowId::RECIPE_MANUFACTURE, 400),
            (WindowId::RECIPE_TREE, 700),
            (WindowId::RECIPE_SHOP, 600),
            (WindowId::RECIPE_SELL, 600),
            (WindowId::RECIPE_BUY, 600),
        ] {
            let mut window = Window::new(id, Rect::new(x, 50, 256, 401), Rect::new(0, 0, 256, 20));
            window.visible = false;
            desktop.windows.push(window);
        }
        desktop
    }

    fn inventory_item(item_id: i32, count: i32, equipped: bool) -> l2_net::game::InventoryItem {
        l2_net::game::InventoryItem {
            type1: 0,
            object_id: item_id * 10,
            item_id,
            count,
            type2: 0,
            custom_type1: 0,
            equipped,
            body_part: 0,
            enchant: 0,
            custom_type2: 0,
        }
    }

    #[test]
    #[ignore]
    fn recipe_tree_100f59d0_matches_every_material_of_every_client_recipe() {
        fn verify(node: &RecipeNode, rate: i32, raw: &[l2_dat::recipe::Recipe], depth: usize) -> usize {
            assert!(depth < raw.len());
            assert!(!node.expanded);
            assert!(!node.item.icon.is_empty());
            let expected = raw.iter().find(|r| r.product_id == node.item.item_id && r.success_rate == rate);
            if let Some(recipe) = expected {
                assert_eq!(node.children.len(), recipe.materials.len());
                let mut count = 1;
                for (child, material) in node.children.iter().zip(&recipe.materials) {
                    assert_eq!((child.item.item_id, child.needed), (material.item_id, material.count));
                    count += verify(child, 100, raw, depth + 1);
                }
                count
            } else {
                assert!(node.children.is_empty());
                1
            }
        }
        let root = std::env::var("KILLER_CLIENT_DIR").unwrap();
        let mut client = killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
        let raw = l2_dat::recipe::read(&client.read_system_file("Recipe-c.dat").unwrap()).unwrap();
        let tables = client.tables().unwrap();
        assert_eq!(tables.recipe_order, raw.iter().map(|r| r.id).collect::<Vec<_>>());
        let mut windows = RecipeWindows::new(|id| tables.sysstring.get(&id).cloned());
        let mut desktop = desktop();
        let mut count = 0;
        for recipe in &raw {
            windows.manufacture.recipe_id = recipe.id;
            windows.open_tree(recipe.id, &World::default(), &tables, &mut desktop);
            let tree = windows.tree.root.as_ref().unwrap();
            assert_eq!(tree.item.item_id, recipe.product_id);
            count += verify(tree, recipe.success_rate, &raw, 0);
            assert_eq!(
                (windows.tree.mp, windows.tree.success, windows.tree.level),
                (recipe.mp_consume, recipe.success_rate, recipe.level)
            );
        }
        assert_eq!(raw.len(), 782);
        eprintln!("{} receitas, {count} nós e todos os materiais conferidos; sem ciclos", raw.len());
    }

    #[test]
    #[ignore]
    fn manufacture_100f7460_100f44e0_resolves_grades_for_every_client_recipe() {
        let root = std::env::var("KILLER_CLIENT_DIR").unwrap();
        let mut client = killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let mut windows = RecipeWindows::new(|id| tables.sysstring.get(&id).cloned());
        let mut desktop = desktop();
        let mut grades = std::collections::BTreeSet::new();
        for recipe in tables.recipes.values() {
            let grade = tables.item_grade(recipe.product_id as u32).unwrap();
            assert!((0..=5).contains(&grade), "{}: {grade}", recipe.product_id);
            grades.insert(grade);
            let info = l2_net::game::RecipeItemMakeInfo {
                recipe_id: recipe.id,
                common: false,
                mp: 100,
                max_mp: 200,
                status: -1,
            };
            windows.open_manufacture(info, &World::default(), &tables, &mut desktop);
            let expected = match grade {
                1 => Some("symbol.grade_d"),
                2 => Some("symbol.grade_c"),
                3 => Some("symbol.grade_b"),
                4 => Some("symbol.grade_a"),
                5 => Some("symbol.grade_s"),
                _ => None,
            };
            assert_eq!(windows.manufacture.product_grade_texture.as_deref(), expected, "recipe {}", recipe.id);
        }
        assert_eq!(grades, (0..=5).collect());
        eprintln!("{} receitas, graus 0..5 conferidos no fluxo de fabricação", tables.recipes.len());
    }

    #[test]
    fn book_100707f0_replaces_a_list_while_the_slider_is_held() {
        let (tables, mut desktop) = (tables(), desktop());
        let mut windows = RecipeWindows::new(|id| Some(id.to_string()));
        for id in 0..160 {
            windows.book.push(BookEntry { recipe_id: id, grid: GridEntry::default() });
        }
        let rect = windows.book.grid.rect;
        windows.book_mouse_down(rect.x + rect.w - 3, rect.y + killer_ui::scroll::BUTTON_SIZE + 2, false, &mut desktop);
        assert!(windows.book.grid.scroll.slider.dragging);
        windows.book_mouse_move(rect.y + 1000);
        let mut world = World::default();
        let book = l2_net::game::RecipeBookItemList { common: false, max_mp: 300, recipes: vec![(1, 0)] };
        world.recipes.book = Some(book.clone());
        windows.world_event(&WorldEvent::RecipeBookItemList(book));
        windows.sync(&world, Some(&tables), &mut desktop);
        assert!(!windows.book.grid.scroll.slider.dragging);
        assert_eq!(windows.book.grid.scroll_row, 0);
        assert!(!windows.book_mouse_move(rect.y + 1000));
        assert_eq!(windows.pressed_book, None);
    }

    #[test]
    fn book_100707f0_opens_at_the_manufacture_position_with_product_icon_and_recipe_name() {
        let (tables, mut desktop) = (tables(), desktop());
        let mut world = World::default();
        world.recipes.book =
            Some(l2_net::game::RecipeBookItemList { common: false, max_mp: 300, recipes: vec![(1, 1), (99, 2)] });
        let mut windows = RecipeWindows::new(|id| Some(id.to_string()));
        windows.world_event(&WorldEvent::RecipeBookItemList(world.recipes.book.clone().unwrap()));
        windows.sync(&world, Some(&tables), &mut desktop);
        let book = desktop.window(WindowId::RECIPE_BOOK).unwrap();
        assert!(book.visible);
        assert_eq!((book.rect.x, book.rect.y), (400, 50));
        assert_eq!(windows.book.entries.len(), 1);
        assert_eq!(windows.book.entries[0].recipe_id, 1);
        assert_eq!(windows.book.entries[0].grid.name, "Recipe: Wooden Arrow");
        assert_eq!(windows.book.frame.title, "1215");
    }

    #[test]
    fn make_info_100708f0_hides_the_book_and_counts_unequipped_materials() {
        let (tables, mut desktop) = (tables(), desktop());
        let mut world = World::default();
        world.inventory.items =
            vec![inventory_item(1864, 3, false), inventory_item(1864, 2, true), inventory_item(17, 7, false)];
        world.recipes.make_info =
            Some(l2_net::game::RecipeItemMakeInfo { recipe_id: 1, common: false, mp: 100, max_mp: 200, status: 1 });
        desktop.window_mut(WindowId::RECIPE_BOOK).unwrap().visible = true;
        let mut windows = RecipeWindows::new(|id| Some(id.to_string()));
        windows.world_event(&WorldEvent::RecipeItemMakeInfo(world.recipes.make_info.unwrap()));
        windows.sync(&world, Some(&tables), &mut desktop);
        assert!(!desktop.window(WindowId::RECIPE_BOOK).unwrap().visible);
        let window = desktop.window(WindowId::RECIPE_MANUFACTURE).unwrap();
        assert!(window.visible);
        assert_eq!((window.rect.x, window.rect.y), (100, 50));
        let view = &windows.manufacture;
        assert_eq!((view.recipe_id, view.product_num, view.mp_consume, view.mp, view.max_mp), (1, 500, 30, 100, 200));
        assert_eq!((view.materials[0].needed, view.materials[0].owned, view.retained), (4, 3, 7));
        assert_eq!(view.status_text, "Wooden Arrow (*500) manufacturing success.");
    }

    #[test]
    fn manufacture_buttons_100f3a90_100f39c0_send_make_self_and_reopen_the_same_book() {
        let mut desktop = desktop();
        desktop.window_mut(WindowId::RECIPE_MANUFACTURE).unwrap().visible = true;
        let mut windows = RecipeWindows::new(|id| Some(id.to_string()));
        windows.manufacture.recipe_id = 9;
        windows.manufacture.book_type = 1;
        for (rect, expected) in [
            (killer_ui::recipe::CREATE_RECT, UiCommand::Net(NetCommand::RecipeItemMakeSelf { recipe_id: 9 })),
            (killer_ui::recipe::BACK_BUTTON_RECT, UiCommand::Net(NetCommand::RecipeBookOpen { common: true })),
        ] {
            let (x, y) = (rect.x + 2, rect.y + 2);
            windows.manufacture_mouse_down(x, y);
            windows.manufacture_mouse_up(Some((WindowId::RECIPE_MANUFACTURE, x, y)), &mut desktop);
            if matches!(expected, UiCommand::Net(NetCommand::RecipeBookOpen { .. })) {
                assert_eq!(windows.commands.pop(), Some(UiCommand::InterfaceSound("itemsound.window_close")));
            }
            assert_eq!(windows.commands.pop(), Some(expected));
        }
        assert!(!desktop.window(WindowId::RECIPE_MANUFACTURE).unwrap().visible);
    }

    #[test]
    fn inventory_pages_split_the_tree_count_the_own_retained_count_and_the_buyer_index_bug() {
        let mut world = World::default();
        let mut quest = inventory_item(5, 4, false);
        quest.type2 = killer_ui::inventory::QUEST_TYPE2;
        world.inventory.items = vec![inventory_item(7, 3, false), inventory_item(5, 2, false), quest, inventory_item(5, 9, true)];
        assert_eq!(page0_owned(&world, 5), 2);
        assert_eq!(owned(&world, 5), 6);
        assert_eq!(buyer_retained(&world, 5), 2 + 3);
    }

    #[test]
    fn close_100f39a0_100f3ba0_emits_the_original_sound_once_on_release() {
        let mut windows = RecipeWindows::new(|_| None);
        let mut desktop = desktop();
        windows.manufacture_mouse_down(180, 380);
        assert!(windows.commands.is_empty());
        windows.manufacture_mouse_up(None, &mut desktop);
        assert!(windows.commands.is_empty());
        windows.manufacture_mouse_down(180, 380);
        windows.manufacture_mouse_up(Some((WindowId::RECIPE_MANUFACTURE, 180, 380)), &mut desktop);
        assert_eq!(windows.commands.pop(), Some(UiCommand::InterfaceSound("itemsound.window_close")));
        windows.tree.mouse_down(110, 380);
        windows.tree_mouse_up(Some((WindowId::RECIPE_TREE, 110, 380)), &mut desktop);
        assert_eq!(windows.commands.pop(), Some(UiCommand::InterfaceSound("itemsound.window_close")));
        assert!(windows.commands.is_empty());
    }

    #[test]
    fn recipe_buyer_10070c00_100eff00_100efe20_keeps_the_server_price_and_back_flow() {
        let tables = tables();
        let mut desktop = desktop();
        let mut windows = RecipeWindows::new(|id| Some(id.to_string()));
        let mut world = World::default();
        world.inventory.items = vec![inventory_item(1864, 3, false), inventory_item(17, 7, false)];
        let sell = l2_net::game::RecipeShopSellList {
            manufacturer_id: 77,
            mp: 80,
            max_mp: 200,
            adena: 0,
            items: vec![l2_net::game::ManufactureItem { recipe_id: 1, unknown: 0, cost: 333 }],
        };
        world.apply_world(&WorldEvent::RecipeShopSellList(sell.clone()));
        windows.world_event(&WorldEvent::RecipeShopSellList(sell));
        windows.sync(&world, Some(&tables), &mut desktop);
        assert!(desktop.window(WindowId::RECIPE_SELL).unwrap().visible);
        windows.sell_down(20, 60);
        assert_eq!(
            windows.commands.pop(),
            Some(UiCommand::Net(NetCommand::RecipeShopMakeInfo { manufacturer_id: 77, recipe_id: 1 }))
        );
        for status in [-1, 0, 1, 2] {
            let info = l2_net::game::RecipeShopItemInfo {
                manufacturer_id: 77,
                recipe_id: 1,
                mp: 80,
                max_mp: 200,
                status,
                cost: 444,
            };
            world.apply_world(&WorldEvent::RecipeShopItemInfo(info));
            windows.world_event(&WorldEvent::RecipeShopItemInfo(info));
            windows.sync(&world, Some(&tables), &mut desktop);
            assert!(windows.sell.entries.is_empty());
            assert!(!desktop.window(WindowId::RECIPE_SELL).unwrap().visible);
            assert!(desktop.window(WindowId::RECIPE_BUY).unwrap().visible);
            assert_eq!((windows.buy.materials[0].owned, windows.buy.retained), (3, 7));
            if status == 1 {
                assert_eq!(windows.buy.status_text, "Wooden Arrow (*500) manufacturing success.");
            }
            if !(0..=1).contains(&status) {
                assert!(windows.buy.status_text.is_empty());
            }
            windows.buy_down(20, 380);
            windows.buy_up(Some((WindowId::RECIPE_BUY, 20, 380)), &mut desktop);
            assert_eq!(
                windows.commands.pop(),
                Some(UiCommand::Net(NetCommand::RecipeShopMakeDo { manufacturer_id: 77, recipe_id: 1, cost: 444 }))
            );
        }
        windows.buy_down(100, 380);
        windows.buy_up(Some((WindowId::RECIPE_BUY, 100, 380)), &mut desktop);
        assert_eq!(
            windows.commands,
            vec![
                UiCommand::Net(NetCommand::RecipeShopSellList { manufacturer_id: 77 }),
                UiCommand::InterfaceSound("itemsound.window_close")
            ]
        );
        assert!(!desktop.window(WindowId::RECIPE_BUY).unwrap().visible);
    }

    #[test]
    fn book_slot_double_click_100f3f80_requests_make_info_and_trash_drop_confirms() {
        let mut desktop = desktop();
        let mut windows = RecipeWindows::new(|id| Some(id.to_string()));
        windows.book.push(BookEntry { recipe_id: 5, grid: GridEntry { name: "R".into(), ..Default::default() } });
        let (x, y) = (killer_ui::recipe::BOOK_GRID_RECT.x + 2, killer_ui::recipe::BOOK_GRID_RECT.y + 2);
        assert_eq!(windows.book_mouse_down(x, y, false, &mut desktop), Some(5));
        assert_eq!(windows.book_mouse_down(x, y, true, &mut desktop), None);
        assert_eq!(windows.commands.pop(), Some(UiCommand::Net(NetCommand::RecipeItemMakeInfo { recipe_id: 5 })));
        assert_eq!(windows.book.grid.selected, None);
        let trash = killer_ui::recipe::TRASH_RECT;
        assert_eq!(windows.book.hit(trash.x + 34, trash.y + 34), Some(BookHit::Trash));
        assert_eq!(windows.book.hit(trash.x + 35, trash.y), None);
        let over = Some((WindowId::RECIPE_BOOK, trash.x + 3, trash.y + 3));
        windows.drag_over(over);
        assert!(windows.book.trash_drag);
        windows.dropped(5, over);
        assert!(!windows.book.trash_drag);
        assert_eq!(windows.commands.pop(), Some(UiCommand::ConfirmDeleteRecipe { recipe_id: 5, name: "R".into() }));
    }
}
