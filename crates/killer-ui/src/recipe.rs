#[path = "recipe_manufacture_layout_generated.rs"]
mod manufacture_layout;

#[path = "recipe_book_layout_generated.rs"]
mod book_layout;

use crate::button::TextButton;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{BUTTON_SIZE, BUTTON_Y, CLOSE_INSET, FrameCtrl, FrameMode};
use crate::item_grid::{GridEntry, GridHit, ItemGrid};
use crate::window::Rect;

pub const SIZE: (i32, i32) = book_layout::SIZE;
pub const BACK_RECT: Rect = Rect::new(0, 20, 256, 381);
pub const LABEL_COLOR: Argb = Argb(0xffdcdcdc);
pub const VALUE_COLOR: Argb = Argb(0xffb09b79);
pub const CAPTION_COLOR: Argb = Argb(0xffa3a3a3);

pub const BOOK_BACK: &str = "L2UI_CH3.RecipeWnd.Recipe2_Back";
pub const TRASH: &str = "L2UI_CH3.InventoryWnd.Inventory_trash";
pub const TRASH_DRAG: &str = "L2UI_CH3.InventoryWnd.Inventory_trash_Drag";
pub const BOOK_GRID_RECT: Rect = book_layout::RECT_174;
pub const BOOK_GRID_COLS: usize = 6;
pub const BOOK_GRID_ROWS: usize = 8;
pub const TRASH_RECT: Rect = Rect::new(208, 347, 32, 32);
pub const TRASH_HIT: Rect = Rect::new(208, 347, 35, 35);
pub const BOOK_LABEL_SYSSTRING: u32 = 649;
pub const BOOK_LABEL_POS: (i32, i32) = (11, 32);
pub const COMMON_TITLE_SYSSTRING: u32 = 1214;
pub const DWARF_TITLE_SYSSTRING: u32 = 1215;
pub const TRASH_TOOLTIP_SYSSTRING: u32 = 890;
pub const COUNTER_Y: i32 = 33;
pub const COUNTER_RIGHT_INSET: f64 = 10.0;
pub const DELETE_SYSMSG: u32 = 74;
pub const DELETE_DIALOG_TYPE: i32 = 0x22;

pub fn book_default_position(viewport: (i32, i32)) -> (i32, i32) {
    (
        crate::window_restore::x87_to_int(f64::from(viewport.0 as f32) - 256.0 - 46.0 - 256.0),
        crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) * 0.5 - 252.0),
    )
}

pub fn book_initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    let (x, y) = book_default_position(viewport);
    crate::initial_position::InitialPosition {
        enabled: true,
        horizontal_mode: 1,
        vertical_mode: 5,
        x: viewport.0.wrapping_sub(x),
        y: (viewport.1 / 2).wrapping_sub(y),
    }
}

pub fn grip() -> Rect {
    FrameCtrl::new(FrameMode::Title).grip(SIZE.0, SIZE.1)
}

fn close_hit(x: i32, y: i32) -> bool {
    let left = SIZE.0 - CLOSE_INSET;
    (left..left + BUTTON_SIZE).contains(&x) && (BUTTON_Y..BUTTON_Y + BUTTON_SIZE).contains(&y)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookHit {
    Close,
    Slot(usize),
    Scroll(crate::scroll::ScrollHit),
    Trash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookEntry {
    pub recipe_id: i32,
    pub grid: GridEntry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeBookView {
    pub book_type: i32,
    pub max_mp: i32,
    pub common_limit: i32,
    pub dwarf_limit: i32,
    pub label: String,
    pub trash_tip: String,
    pub titles: (String, String),
    pub entries: Vec<BookEntry>,
    pub grid: ItemGrid,
    pub trash_drag: bool,
    pub frame: FrameCtrl,
}

impl RecipeBookView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let titles = (lookup(COMMON_TITLE_SYSSTRING).unwrap_or_default(), lookup(DWARF_TITLE_SYSSTRING).unwrap_or_default());
        Self {
            book_type: 0,
            max_mp: 0,
            common_limit: 0,
            dwarf_limit: 0,
            label: lookup(BOOK_LABEL_SYSSTRING).unwrap_or_default(),
            trash_tip: lookup(TRASH_TOOLTIP_SYSSTRING).unwrap_or_default(),
            frame: FrameCtrl::titled(&titles.1, true),
            titles,
            entries: Vec::new(),
            grid: ItemGrid::new(BOOK_GRID_RECT, BOOK_GRID_COLS, BOOK_GRID_ROWS),
            trash_drag: false,
        }
    }

    pub fn open(&mut self, book_type: i32, max_mp: i32) {
        self.book_type = book_type;
        let title = if book_type == 1 { &self.titles.0 } else { &self.titles.1 };
        self.frame = FrameCtrl::titled(title, true);
        self.entries.clear();
        self.grid.clear();
        self.max_mp = max_mp;
    }

    pub fn push(&mut self, entry: BookEntry) {
        if self.entries.len() >= 160 { return; }
        self.entries.push(entry);
        self.grid.update_rows(self.entries.len());
    }

    pub fn limit(&self) -> i32 {
        if self.book_type == 1 { self.common_limit } else { self.dwarf_limit }
    }

    pub fn counter(&self) -> String {
        format!("({}/{})", self.entries.len(), self.limit())
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<BookHit> {
        if close_hit(x, y) {
            return Some(BookHit::Close);
        }
        if TRASH_HIT.contains(x, y) {
            return Some(BookHit::Trash);
        }
        match self.grid.hit(x, y)? {
            GridHit::Slot(index) => (index < self.entries.len()).then_some(BookHit::Slot(index)),
            GridHit::Scroll(hit) => Some(BookHit::Scroll(hit)),
        }
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        self.frame.paint(list, x, y, SIZE.0, SIZE.1, font, alpha);
        let Rect { x: bx, y: by, w, h } = BACK_RECT;
        list.draw_texture(x + bx, y + by, w, h, 0, 0, w, h, BOOK_BACK, alpha);
        let trash = if self.trash_drag { TRASH_DRAG } else { TRASH };
        list.draw_texture(x + TRASH_RECT.x, y + TRASH_RECT.y, 32, 32, 0, 0, 32, 32, trash, alpha);
        list.draw_text(x + BOOK_LABEL_POS.0, y + BOOK_LABEL_POS.1, LABEL_COLOR, &self.label, font);
        let grid: Vec<GridEntry> = self.entries.iter().map(|entry| entry.grid.clone()).collect();
        self.grid.paint(list, x, y, &grid, font);
        let counter = self.counter();
        let width = font.measure(&counter) as i32;
        let cx = crate::window_restore::x87_to_int(f64::from(SIZE.0 as f32) - f64::from(width) - COUNTER_RIGHT_INSET);
        list.draw_text(x + cx, y + COUNTER_Y, VALUE_COLOR, &counter, font);
    }
}

pub const MANUFACTURE_BACK: &str = "L2UI_CH3.RecipeWnd.Recipe4_Back";
pub const MP_BAR: &str = "L2UI_CH3.PlayerStatusWnd.ps_mpbar";
pub const SHORTFALL: &str = "NWindow.ChatBack";
pub const TREE_BUTTON: (&str, &str) = ("L2UI.RecipeWnd.RecipeTreeBtn", "L2UI.RecipeWnd.RecipeTreeBtn_click");
pub const TREE_BUTTON_RECT: Rect = manufacture_layout::RECT_1D64;
pub const TREE_TOOLTIP_SYSSTRING: u32 = 662;
pub const BUTTON_TEXTURES: (&str, &str) = ("L2UI_CH3.Button.Btn1_Normal", "L2UI_CH3.Button.Btn1_NormalOn");
pub const CREATE_RECT: Rect = manufacture_layout::RECT_1D68;
pub const BACK_BUTTON_RECT: Rect = manufacture_layout::RECT_1D6C;
pub const CLOSE_BUTTON_RECT: Rect = manufacture_layout::RECT_1D70;
pub const CREATE_SYSSTRING: u32 = 645;
pub const BACK_SYSSTRING: u32 = 543;
pub const CLOSE_SYSSTRING: u32 = 646;
pub const MATERIAL_RECT: Rect = manufacture_layout::RECT_E8;
pub const MATERIAL_COLS: usize = 6;
pub const MATERIAL_ROWS: usize = 3;
pub const MATERIAL_CELL: i32 = 32;
pub const ICON_RECT: Rect = Rect::new(16, 49, 32, 32);
pub const NAME_POS: (i32, i32) = (58, 48);
pub const MP_COST_SYSSTRING: u32 = 320;
pub const SUCCESS_RATE_SYSSTRING: u32 = 642;
pub const RESULT_COUNT_SYSSTRING: u32 = 647;
pub const RETAINED_SYSSTRING: u32 = 648;
pub const CREATOR_MP_SYSSTRING: u32 = 644;
pub const INGREDIENTS_SYSSTRING: u32 = 742;
pub const RESULTS_SYSSTRING: u32 = 743;
pub const SUCCESS_SYSMSG: u32 = 959;
pub const FAILURE_SYSMSG: u32 = 960;
pub const STATUS_COLOR: Argb = Argb(0xffb2becf);
pub const SEPARATOR: &str = " : ";
pub const MP_BAR_X: i32 = 75;
pub const MP_BAR_Y: i32 = 261;
pub const MP_BAR_WIDTH: i32 = 165;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManufactureHit {
    Close,
    Material(usize),
    Tree,
    Create,
    Back,
    CloseButton,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Material {
    pub item_id: i32,
    pub needed: i32,
    pub owned: i32,
    pub grid: GridEntry,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ManufactureLabels {
    pub mp_cost: String,
    pub success_rate: String,
    pub result_count: String,
    pub retained: String,
    pub creator_mp: String,
    pub ingredients: String,
    pub results: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeManufactureView {
    pub recipe_id: i32,
    pub product_id: i32,
    pub product_num: i32,
    pub mp_consume: i32,
    pub success_rate: i32,
    pub product_name: String,
    pub product_icon: String,
    pub product_grade_texture: Option<String>,
    pub mp: i32,
    pub max_mp: i32,
    pub status: i32,
    pub book_type: i32,
    pub retained: i32,
    pub status_text: String,
    pub wrap_status: bool,
    pub materials: Vec<Material>,
    pub labels: ManufactureLabels,
    pub tree: TextButton,
    pub tree_hover: bool,
    pub create: TextButton,
    pub back: TextButton,
    pub close: TextButton,
    pub pressed: Option<ManufactureHit>,
    pub frame: FrameCtrl,
}

impl RecipeManufactureView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id| lookup(id).unwrap_or_default();
        Self {
            recipe_id: 0,
            product_id: 0,
            product_num: 0,
            mp_consume: 0,
            success_rate: 0,
            product_name: String::new(),
            product_icon: String::new(),
            product_grade_texture: None,
            mp: 0,
            max_mp: 0,
            status: -1,
            book_type: 0,
            retained: 0,
            status_text: String::new(),
            wrap_status: false,
            materials: Vec::new(),
            labels: ManufactureLabels {
                mp_cost: text(MP_COST_SYSSTRING),
                success_rate: text(SUCCESS_RATE_SYSSTRING),
                result_count: text(RESULT_COUNT_SYSSTRING),
                retained: text(RETAINED_SYSSTRING),
                creator_mp: text(CREATOR_MP_SYSSTRING),
                ingredients: text(INGREDIENTS_SYSSTRING),
                results: text(RESULTS_SYSSTRING),
            },
            tree_hover: false,
            tree: TextButton::new(TREE_BUTTON_RECT, TREE_BUTTON.0, TREE_BUTTON.1, ""),
            create: TextButton::new(CREATE_RECT, BUTTON_TEXTURES.0, BUTTON_TEXTURES.1, &text(CREATE_SYSSTRING)),
            back: TextButton::new(BACK_BUTTON_RECT, BUTTON_TEXTURES.0, BUTTON_TEXTURES.1, &text(BACK_SYSSTRING)),
            close: TextButton::new(CLOSE_BUTTON_RECT, BUTTON_TEXTURES.0, BUTTON_TEXTURES.1, &text(CLOSE_SYSSTRING)),
            pressed: None,
            frame: FrameCtrl::titled(&text(crate::recipe::DWARF_TITLE_SYSSTRING), true),
        }
    }

    pub fn clear(&mut self) {
        self.recipe_id = 0;
        self.product_id = 0;
        self.product_grade_texture = None;
        self.tree_hover = false;
        self.product_num = 0;
        self.mp_consume = 0;
        self.success_rate = 0;
        self.mp = 0;
        self.max_mp = 0;
        self.materials.clear();
    }

    pub fn mp_bar_width(&self) -> i32 {
        if self.max_mp == 0 {
            return 0;
        }
        let width = crate::window_restore::x87_to_int(f64::from(self.mp.wrapping_mul(MP_BAR_WIDTH)) / f64::from(self.max_mp));
        width.min(MP_BAR_WIDTH)
    }

    pub fn material_rect(index: usize) -> Rect {
        let (col, row) = ((index % MATERIAL_COLS) as i32, (index / MATERIAL_COLS) as i32);
        Rect::new(MATERIAL_RECT.x + col * (MATERIAL_CELL + 5), MATERIAL_RECT.y + row * (MATERIAL_CELL + 3), MATERIAL_CELL, MATERIAL_CELL)
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<ManufactureHit> {
        if close_hit(x, y) {
            return Some(ManufactureHit::Close);
        }
        for (button, hit) in [(&self.tree, ManufactureHit::Tree), (&self.create, ManufactureHit::Create), (&self.back, ManufactureHit::Back), (&self.close, ManufactureHit::CloseButton)] {
            if button.contains(x, y) {
                return Some(hit);
            }
        }
        (0..self.materials.len().min(MATERIAL_COLS * MATERIAL_ROWS))
            .find(|index| Self::material_rect(*index).contains(x, y))
            .map(ManufactureHit::Material)
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        self.frame.paint(list, x, y, manufacture_layout::SIZE.0, manufacture_layout::SIZE.1, font, alpha);
        let Rect { x: bx, y: by, w, h } = BACK_RECT;
        list.draw_texture(x + bx, y + by, w, h, 0, 0, w, h, MANUFACTURE_BACK, alpha);
        if !self.product_icon.is_empty() {
            list.draw_texture(x + ICON_RECT.x, y + ICON_RECT.y, 32, 32, 0, 0, 32, 32, &self.product_icon, alpha);
        }
        list.draw_text(x + NAME_POS.0, y + NAME_POS.1, LABEL_COLOR, &self.product_name, font);
        if let Some(texture) = &self.product_grade_texture {
            list.draw_texture(x + NAME_POS.0 + font.measure(&self.product_name) as i32 + 2, y + NAME_POS.1, 12, 12, 0, 0, 12, 12, texture, 1.0);
            if let Some(quad) = list.quads.last_mut() { quad.color = LABEL_COLOR.rgba(); }
        }
        for (row, (caption, value)) in [
            (&self.labels.mp_cost, self.mp_consume.to_string()),
            (&self.labels.success_rate, format!("{}%", self.success_rate)),
            (&self.labels.result_count, self.product_num.to_string()),
        ].into_iter().enumerate() {
            let line_y = y + 0x40 + 0x10 * row as i32;
            let width = font.measure(caption) as i32;
            list.draw_text(x + 0x3a, line_y, CAPTION_COLOR, caption, font);
            let separator = list.draw_text(x + 0x3a + width, line_y, LABEL_COLOR, SEPARATOR, font);
            list.draw_text(separator, line_y, VALUE_COLOR, &value, font);
        }
        list.draw_text(x + 0xb, y + 0x7c, LABEL_COLOR, &self.labels.ingredients, font);
        list.draw_text(x + 0xb, y + 0x122, LABEL_COLOR, &self.labels.results, font);
        let outline = if self.tree_hover { "L2UI_CH3.Etc.menu_outline_down" } else { "L2UI_CH3.Etc.menu_outline" };
        list.draw_texture(x + 205, y + 70, 36, 36, 0, 0, 34, 34, outline, 1.0);
        self.tree.paint(list, x, y, self.pressed == Some(ManufactureHit::Tree), font);
        if !self.status_text.is_empty() {
            if self.wrap_status {
                let mut row_y = y + 0x149;
                for (_, row) in crate::chat::wrap_line(&self.status_text, SIZE.0 - 30, 0, font) {
                    list.draw_text(x + 0xe, row_y, STATUS_COLOR, &row, font);
                    row_y += font.line_height as i32;
                }
            } else {
                list.draw_text(x + 0xe, y + 0x149, STATUS_COLOR, &self.status_text, font);
            }
        }
        let width = font.measure(&self.labels.retained) as i32;
        list.draw_text(x + 0xe, y + 0x139, CAPTION_COLOR, &self.labels.retained, font);
        let separator = list.draw_text(x + width + 0xe, y + 0x139, LABEL_COLOR, SEPARATOR, font);
        list.draw_text(separator, y + 0x139, VALUE_COLOR, &self.retained.to_string(), font);
        list.draw_text(x + 0x12, y + 0x105, LABEL_COLOR, &self.labels.creator_mp, font);
        let bar = self.mp_bar_width();
        if bar >= 1 {
            list.draw_texture(x + MP_BAR_X, y + MP_BAR_Y, bar, 12, 0, 0, 8, 12, MP_BAR, alpha);
        }
        for (index, material) in self.materials.iter().take(MATERIAL_COLS * MATERIAL_ROWS).enumerate() {
            let cell = Self::material_rect(index);
            list.draw_texture(x + cell.x, y + cell.y, cell.w, cell.h, 0, 0, cell.w, cell.h, material.grid.icon(), 1.0);
            if material.owned < material.needed {
                list.draw_texture(x + cell.x, y + cell.y, cell.w, cell.h, 0, 0, cell.w, cell.h, SHORTFALL, 1.0);
            }
        }
        self.create.paint(list, x, y, self.pressed == Some(ManufactureHit::Create), font);
        self.back.paint(list, x, y, self.pressed == Some(ManufactureHit::Back), font);
        self.close.paint(list, x, y, self.pressed == Some(ManufactureHit::CloseButton), font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(id: u32) -> Option<String> {
        Some(id.to_string())
    }

    #[test]
    fn manufacture_100f44e0_paints_the_grade_symbol_after_the_product_name() {
        let font = BitmapFont::new("f".into(), crate::font::Glyphs::parse(include_bytes!("../tests/fixtures/smallfont-e.gly")).unwrap());
        let mut view = RecipeManufactureView::new(|_| None);
        view.product_name = "Sword".into();
        for grade in -1..=6 {
            view.product_grade_texture = crate::item_name::grade_symbol_1006b5a0(grade).map(|alias| format!("symbol.{alias}"));
            let mut list = DrawList::default();
            view.paint(&mut list, 10, 20, &font, 1.0);
            let symbols: Vec<_> = list.quads.iter().filter(|q| q.texture.starts_with("symbol.")).collect();
            assert_eq!(symbols.len(), usize::from((1..=5).contains(&grade)));
            if let Some(quad) = symbols.first() {
                assert_eq!((quad.x, quad.y, quad.w, quad.h), ((70 + font.measure("Sword")) as f32, 68.0, 12.0, 12.0));
                assert_eq!(quad.color, LABEL_COLOR.rgba());
            }
        }
        view.clear();
        assert!(view.product_grade_texture.is_none());
    }

    #[test]
    fn the_own_manufacture_outcome_line_wraps_at_the_width_less_30_and_the_shop_one_does_not() {
        let font = BitmapFont::new("f".into(), crate::font::Glyphs::parse(include_bytes!("../tests/fixtures/smallfont-e.gly")).unwrap());
        let text = "Mithril Alloy Mithril Alloy Mithril Alloy (*1) manufacturing success. extra words to wrap";
        let count = |wrap| {
            let mut view = RecipeManufactureView::new(|_| None);
            view.status_text = text.into();
            view.wrap_status = wrap;
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font, 1.0);
            list.quads.iter().filter(|q| q.color == STATUS_COLOR.rgba() && q.x < 20.0).map(|q| q.y as i32).collect::<std::collections::BTreeSet<_>>().len()
        };
        assert_eq!(count(false), 1);
        assert!(count(true) > 1);
    }

    #[test]
    fn book_100f37f0_title_and_100f4040_counter_follow_the_book_type() {
        let mut view = RecipeBookView::new(lookup);
        (view.common_limit, view.dwarf_limit) = (100, 50);
        view.open(1, 300);
        assert_eq!(view.frame.title, "1214");
        view.push(BookEntry { recipe_id: 7, grid: GridEntry::default() });
        assert_eq!(view.counter(), "(1/100)");
        view.open(0, 300);
        assert_eq!(view.frame.title, "1215");
        assert!(view.entries.is_empty());
        assert_eq!(view.counter(), "(0/50)");
    }

    #[test]
    fn recipe_book_100f58b0_stops_at_the_native_capacity() {
        let mut view=RecipeBookView::new(lookup);
        for recipe_id in 0..200 { view.push(BookEntry { recipe_id,grid:GridEntry::default() }); }
        assert_eq!(view.entries.len(),160);
        assert_eq!(view.entries.last().unwrap().recipe_id,159);
        view.open(1,100);
        view.push(BookEntry { recipe_id:200,grid:GridEntry::default() });
        assert_eq!(view.entries.len(),1);
    }

    #[test]
    fn book_hits_close_trash_and_only_filled_slots() {
        let mut view = RecipeBookView::new(lookup);
        view.push(BookEntry { recipe_id: 1, grid: GridEntry::default() });
        assert_eq!(view.hit(SIZE.0 - CLOSE_INSET, BUTTON_Y), Some(BookHit::Close));
        assert_eq!(view.hit(TRASH_RECT.x + 1, TRASH_RECT.y + 1), Some(BookHit::Trash));
        assert_eq!(view.hit(BOOK_GRID_RECT.x + 2, BOOK_GRID_RECT.y + 2), Some(BookHit::Slot(0)));
        assert_eq!(view.hit(BOOK_GRID_RECT.x + 40, BOOK_GRID_RECT.y + 2), None);
    }

    #[test]
    fn manufacture_mp_bar_100f4bc5_scales_to_165_and_clamps() {
        let mut view = RecipeManufactureView::new(lookup);
        for (mp, max, width) in [(0, 0, 0), (50, 100, 82), (100, 100, 165), (200, 100, 165), (1, 1000, 0)] {
            (view.mp, view.max_mp) = (mp, max);
            assert_eq!(view.mp_bar_width(), width, "{mp}/{max}");
        }
    }

    #[test]
    fn manufacture_materials_100f42a2_step_37_by_35_and_mark_shortfall() {
        assert_eq!(RecipeManufactureView::material_rect(0), Rect::new(10, 141, 32, 32));
        assert_eq!(RecipeManufactureView::material_rect(1), Rect::new(47, 141, 32, 32));
        assert_eq!(RecipeManufactureView::material_rect(6), Rect::new(10, 176, 32, 32));
        let mut view = RecipeManufactureView::new(lookup);
        view.materials = vec![Material { needed: 3, owned: 1, ..Default::default() }, Material { needed: 1, owned: 1, ..Default::default() }];
        let mut list = DrawList::default();
        let font = BitmapFont::new(String::new(), crate::font::Glyphs { tex_width: 1, tex_height: 1, glyphs: Default::default() });
        view.paint(&mut list, 0, 0, &font, 1.0);
        assert_eq!(list.quads.iter().filter(|q| q.texture == SHORTFALL).count(), 1);
        assert_eq!(view.hit(48, 142), Some(ManufactureHit::Material(1)));
        assert_eq!(view.hit(CREATE_RECT.x + 1, CREATE_RECT.y + 1), Some(ManufactureHit::Create));
    }
}
