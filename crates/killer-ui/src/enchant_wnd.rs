use crate::button::TextButton;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::item_grid::{GridEntry, GridHit, ItemGrid};
use crate::window::Rect;

pub const SIZE: (i32, i32) = (256, 401);

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    crate::inventory::default_position(viewport)
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    crate::inventory::initial_position(viewport)
}

pub const BACK: &str = "L2UI_CH3.InventoryWnd.itemenchant_back";
pub const BACK_RECT: Rect = Rect::new(0, 20, 256, 381);
pub const GRID_RECT: Rect = Rect::new(9, 48, 240, 318);
pub const GRID_COLS: usize = 6;
pub const GRID_ROWS: usize = 9;
pub const OK_RECT: Rect = Rect::new(51, 372, 76, 23);
pub const CANCEL_RECT: Rect = Rect::new(131, 372, 76, 23);
pub const LABEL_POS: (i32, i32) = (11, 32);
pub const BTN_NORMAL: &str = "L2UI_ch3.button.btn1_normal";
pub const BTN_DOWN: &str = "L2UI_ch3.button.btn1_normalon";

/// Rótulo dentro da janela, desenhado pelo `NCItemEnchantWnd::OnPaint @ 0x10095cd0` a partir de
/// `GL2GameData + 0x9c058`.
pub const LABEL_SYSSTRING: u32 = 0x4c5;
/// Primeira parte do título, que o abridor (`0x100666c0`) junta com o nome do pergaminho pelo
/// formato `"%s(%s)"` de `0x101b0b28`, usando a string de `GL2GameData + 0x9c04c`.
pub const TITLE_SYSSTRING: u32 = 0x4c4;
pub const OK_SYSSTRING: u32 = 0x8c;
pub const CANCEL_SYSSTRING: u32 = 0x8d;

/// Valor que o `Cancel` manda no lugar do id do item (`pushl $-1` em `0x100940cb`).
pub const CANCEL_OBJECT_ID: i32 = -1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnchantHit {
    Close,
    Slot(usize),
    Scroll(crate::scroll::ScrollHit),
    Ok,
    Cancel,
}

/// Janela de escolha do item a encantar (`NCItemEnchantWnd`). O servidor manda
/// `ChooseInventoryItem` com o id do pergaminho; o cliente mostra a janela, copia **todos** os
/// itens do inventário pra grade (o filtro de `0x10099432` testa `+0x1b14` contra 0, 1 e 2, e
/// esse campo nunca recebe valor diferente de zero em nenhuma DLL do cliente) e põe no título o
/// nome do pergaminho.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantView {
    pub title: String,
    pub label: String,
    pub entries: Vec<GridEntry>,
    pub grid: ItemGrid,
    pub ok: TextButton,
    pub cancel: TextButton,
    pub pressed: Option<EnchantHit>,
    pub frame: FrameCtrl,
}

impl EnchantView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> EnchantView {
        let title = lookup(TITLE_SYSSTRING).unwrap_or_else(|| "Enchant Item".into());
        let label = lookup(LABEL_SYSSTRING).unwrap_or_default();
        let ok = lookup(OK_SYSSTRING).unwrap_or_else(|| "OK".into());
        let cancel = lookup(CANCEL_SYSSTRING).unwrap_or_else(|| "Cancel".into());
        EnchantView {
            title: title.clone(),
            label,
            entries: Vec::new(),
            grid: ItemGrid::new(GRID_RECT, GRID_COLS, GRID_ROWS),
            ok: TextButton::new(OK_RECT, BTN_NORMAL, BTN_DOWN, &ok),
            cancel: TextButton::new(CANCEL_RECT, BTN_NORMAL, BTN_DOWN, &cancel),
            pressed: None,
            frame: FrameCtrl::titled(&title, true),
        }
    }

    /// Título do original: a string `0x4c4` e, entre parênteses, o nome do pergaminho
    /// (`appSprintf(buf, "%s(%s)", …)` em `0x10066735`). Sem nome, fica só a primeira parte.
    pub fn set_scroll(&mut self, base: &str, scroll_name: &str) {
        self.title = match scroll_name.is_empty() {
            true => base.to_string(),
            false => format!("{base}({scroll_name})"),
        };
        self.frame = FrameCtrl::titled(&self.title, true);
    }

    pub fn set_entries(&mut self, entries: Vec<GridEntry>) {
        self.grid.clear();
        self.grid.update_rows(entries.len());
        self.entries = entries;
    }

    pub fn selected(&self) -> Option<&GridEntry> {
        self.entries.get(self.grid.selected?)
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<EnchantHit> {
        let left = SIZE.0 - crate::frame::CLOSE_INSET;
        if (left..left + crate::frame::BUTTON_SIZE).contains(&x)
            && (crate::frame::BUTTON_Y..crate::frame::BUTTON_Y + crate::frame::BUTTON_SIZE).contains(&y)
        {
            return Some(EnchantHit::Close);
        }
        if self.ok.contains(x, y) {
            return Some(EnchantHit::Ok);
        }
        if self.cancel.contains(x, y) {
            return Some(EnchantHit::Cancel);
        }
        match self.grid.hit(x, y)? {
            GridHit::Slot(index) => (index < self.entries.len()).then_some(EnchantHit::Slot(index)),
            GridHit::Scroll(hit) => Some(EnchantHit::Scroll(hit)),
        }
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        self.frame.paint(list, x, y, SIZE.0, SIZE.1, font, alpha);
        let Rect { x: bx, y: by, w, h } = BACK_RECT;
        list.draw_texture(x + bx, y + by, w, h, 0, 0, w, h, BACK, alpha);
        let (lx, ly) = LABEL_POS;
        list.draw_text(x + lx, y + ly, Argb(0xffdcdcdc), &self.label, font);
        self.grid.paint(list, x, y, &self.entries, font);
        self.ok.paint(list, x, y, self.pressed == Some(EnchantHit::Ok), font);
        self.cancel.paint(list, x, y, self.pressed == Some(EnchantHit::Cancel), font);
    }
}

pub fn grip() -> Rect {
    FrameCtrl::new(crate::frame::FrameMode::Title).grip(SIZE.0, SIZE.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};
    use std::collections::HashMap;

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn entry(object_id: i32) -> GridEntry {
        GridEntry { object_id, item_id: 100, count: 1, icon: "icon.x".into(), ..Default::default() }
    }

    /// Geometria e ids conferidos contra `NCItemEnchantWnd::OnCreate @ 0x1009a3c0`: barra de
    /// título `(0, 0, 256, 20)`, grade `(9, 48, 240, 318)` de `FUN_1009a2e0(6, 9, …)`, `OK`
    /// `0xf0f012` em `(51, 372, 76, 23)` com a sysstring `0x8c` e `Cancel` `0xf0f013` em
    /// `(131, 372)` com a `0x8d`; fundo `itemenchant_back` em `(0, 20, 256, 381)` e rótulo
    /// `0x4c5` em `(11, 32)` pelo `OnPaint @ 0x10095cd0`.
    #[test]
    fn the_window_follows_the_original_geometry_and_sysstrings() {
        let texts: HashMap<u32, String> = [(0x4c4, "Enchant"), (0x4c5, "Select an item"), (0x8c, "OK"), (0x8d, "Cancel")]
            .into_iter()
            .map(|(id, text)| (id, text.to_string()))
            .collect();
        let mut view = EnchantView::new(|id| texts.get(&id).cloned());
        assert_eq!((view.ok.label.as_str(), view.cancel.label.as_str()), ("OK", "Cancel"));
        assert_eq!(view.label, "Select an item");
        assert_eq!((OK_RECT, CANCEL_RECT), (Rect::new(51, 372, 76, 23), Rect::new(131, 372, 76, 23)));
        assert_eq!((GRID_RECT, GRID_COLS, GRID_ROWS), (Rect::new(9, 48, 240, 318), 6, 9));
        assert_eq!(SIZE, (256, 401));

        view.set_scroll("Enchant", "Scroll: Enchant Weapon");
        assert_eq!(view.title, "Enchant(Scroll: Enchant Weapon)");
        view.set_scroll("Enchant", "");
        assert_eq!(view.title, "Enchant", "sem nome fica só a primeira parte");

        let quads = {
            let mut list = DrawList::default();
            view.paint(&mut list, 10, 20, &font(), 1.0);
            list.quads
        };
        let back = quads.iter().find(|q| q.texture == BACK).expect("fundo desenhado");
        assert_eq!((back.x, back.y, back.w, back.h), (10.0, 40.0, 256.0, 381.0));
        assert_eq!(quads.iter().filter(|q| q.texture == BTN_NORMAL).count(), 2);
    }

    /// O clique acerta as células, a rolagem e os dois botões; célula sem item não responde.
    #[test]
    fn the_hit_test_covers_slots_scroll_and_both_buttons() {
        let mut view = EnchantView::new(|_| None);
        view.set_entries(vec![entry(1), entry(2)]);
        assert_eq!(view.hit(GRID_RECT.x + 1, GRID_RECT.y + 1), Some(EnchantHit::Slot(0)));
        assert_eq!(view.hit(GRID_RECT.x + 1 + 37, GRID_RECT.y + 1), Some(EnchantHit::Slot(1)));
        assert_eq!(view.hit(GRID_RECT.x + 1 + 37 * 2, GRID_RECT.y + 1), None, "célula vazia não responde");
        assert_eq!(view.hit(OK_RECT.x + 2, OK_RECT.y + 2), Some(EnchantHit::Ok));
        assert_eq!(view.hit(CANCEL_RECT.x + 2, CANCEL_RECT.y + 2), Some(EnchantHit::Cancel));
        assert_eq!(view.hit(SIZE.0 - crate::frame::CLOSE_INSET, crate::frame::BUTTON_Y), Some(EnchantHit::Close));
        assert_eq!(view.hit(2, 2), None);

        view.grid.selected = Some(1);
        assert_eq!(view.selected().map(|e| e.object_id), Some(2));
        view.grid.selected = None;
        assert_eq!(view.selected(), None);
    }
}
