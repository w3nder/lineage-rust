use crate::button::TextButton;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{BUTTON_SIZE, BUTTON_Y, CLOSE_INSET, FrameCtrl, FrameMode};
use crate::scroll::{ScrollHit, ScrollWnd, line_down, line_up, rows_for};
use crate::window::Rect;

pub const SIZE: (i32, i32) = (256, 401);

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    crate::mainwnd::default_position(viewport)
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    crate::mainwnd::initial_position(viewport)
}

pub const BACK_RECT: Rect = Rect::new(0, 20, 256, 381);
pub const LIST_BACK: &str = "L2UI_ch3.HennaWnd.henna1_back";
pub const INFO_BACK: &str = "L2UI_ch3.HennaWnd.Henna2_back";
pub const BTN_NORMAL: &str = "L2UI_ch3.button.btn1_normal";
pub const BTN_DOWN: &str = "L2UI_ch3.button.btn1_normalon";
pub const EMPTY_ICON: &str = "NWindow.BlackTexture";

pub const TEXT: Argb = Argb(0xffdcdcdc);
pub const STAT_LABEL: Argb = Argb(0xffa3a3a3);
pub const STAT_VALUE: Argb = Argb(0xffb09b79);
pub const ICON_SIZE: i32 = 32;
pub const STAT_ARROW: &str = ">";

pub const LIST_RECT: Rect = Rect::new(7, 46, 241, 309);
pub const LIST_ROWS: usize = 6;
pub const ROW_PITCH: i32 = 48;
pub const ROW_TOP: i32 = 9;
pub const ROW_ICON_POS: (i32, i32) = (5, 13);
pub const ROW_TEXT_X: i32 = 47;
pub const ROW_LINE_Y: [i32; 3] = [11, 23, 36];
pub const HEADING_POS: (i32, i32) = (15, 28);
pub const LIST_ADENA_RIGHT: i32 = 230;
pub const LIST_ADENA_Y: i32 = 374;

pub const BACK_BTN_RECT: Rect = Rect::new(51, 372, 76, 23);
pub const OK_BTN_RECT: Rect = Rect::new(131, 372, 76, 23);
pub const BLOCK_LABEL_X: i32 = 18;
pub const BLOCK_ICON_X: i32 = 16;
pub const BLOCK_TEXT_X: i32 = 58;
pub const FIRST_LABEL_Y: i32 = 43;
pub const FIRST_ICON_Y: i32 = 71;
pub const FIRST_NAME_Y: i32 = 66;
pub const FIRST_ADD_Y: i32 = 80;
pub const SECOND_LABEL_Y: i32 = 132;
pub const SECOND_ICON_Y: i32 = 161;
pub const SECOND_NAME_Y: i32 = 156;
pub const SECOND_ADD_Y: i32 = 170;
pub const DRAW_FEE_Y: i32 = 94;
pub const REMOVE_FEE_Y: i32 = 184;
pub const ABILITY_LABEL_POS: (i32, i32) = (18, 221);
pub const STAT_LABEL_X: [i32; 2] = [17, 135];
pub const STAT_ROW_Y: [i32; 3] = [247, 263, 279];
pub const STAT_FROM_CENTER: [i32; 2] = [64, 182];
pub const STAT_ARROW_X: [i32; 2] = [78, 196];
pub const STAT_TO_CENTER: [i32; 2] = [97, 215];
pub const INFO_ADENA_RIGHT: i32 = 229;
pub const INFO_ADENA_Y: i32 = 329;

pub const TITLE_DRAW: u32 = 0x28b;
pub const TITLE_REMOVE: u32 = 0x28c;
pub const HEADING_DRAW: u32 = 0x293;
pub const HEADING_REMOVE: u32 = 0x294;
pub const FEE_SYSSTRING: u32 = 0x27d;
pub const DYE_SYSSTRING: u32 = 0x27e;
pub const SYMBOL_SYSSTRING: u32 = 0x27f;
pub const ABILITY_SYSSTRING: u32 = 0x280;
pub const BACK_SYSSTRING: u32 = 0x21f;
pub const OK_SYSSTRING: u32 = 0x8c;

/// Rótulos das seis colunas de atributo, em `(coluna, linha)` da grade de `0x10092370`, cada um
/// com o índice do par `(d, c)` do pacote que ele mostra. `STR` fica com o 2º par, `DEX` com o 5º,
/// `CON` com o 3º, `INT` com o 1º, `WIT` com o 6º e `MEN` com o 4º.
pub const STATS: [(u32, usize, usize, usize); 6] =
    [(0x68, 0, 0, 1), (0x69, 0, 1, 4), (0x6a, 0, 2, 2), (0x6b, 1, 0, 0), (0x6c, 1, 1, 5), (0x6d, 1, 2, 3)];

/// O `+0xf4` da lista e o `+0x39a0` da janela de informação: `0` desenha um símbolo (pedido por
/// `RequestHennaItemList`), `1` apaga um (por `RequestHennaUnequipList`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HennaMode {
    Draw,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HennaListHit {
    Close,
    Row(usize),
    Scroll(ScrollHit),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HennaInfoHit {
    Close,
    Ok,
    Back,
}

/// Uma linha do `NCHennaListBox`: as três cadeias de `+0x0000`, `+0x0200` e `+0x0400` e os cinco
/// inteiros de `+0x1404` a `+0x1414`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HennaRow {
    pub symbol_id: i32,
    pub dye_item_id: i32,
    pub dye_amount: i32,
    pub price: i32,
    pub available: i32,
    pub name: String,
    pub icon: String,
    pub add_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HennaStrings {
    pub title_draw: String,
    pub title_remove: String,
    pub heading_draw: String,
    pub heading_remove: String,
    pub fee: String,
    pub dye: String,
    pub symbol: String,
    pub ability: String,
    pub back: String,
    pub ok: String,
    pub stats: [String; 6],
}

impl HennaStrings {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> HennaStrings {
        let text = |id: u32| lookup(id).unwrap_or_else(|| format!("sysstring {id}"));
        HennaStrings {
            title_draw: text(TITLE_DRAW),
            title_remove: text(TITLE_REMOVE),
            heading_draw: text(HEADING_DRAW),
            heading_remove: text(HEADING_REMOVE),
            fee: text(FEE_SYSSTRING),
            dye: text(DYE_SYSSTRING),
            symbol: text(SYMBOL_SYSSTRING),
            ability: text(ABILITY_SYSSTRING),
            back: text(BACK_SYSSTRING),
            ok: text(OK_SYSSTRING),
            stats: STATS.map(|(id, _, _, _)| text(id)),
        }
    }

    pub fn title(&self, mode: HennaMode) -> &str {
        match mode {
            HennaMode::Draw => &self.title_draw,
            HennaMode::Remove => &self.title_remove,
        }
    }

    pub fn heading(&self, mode: HennaMode) -> &str {
        match mode {
            HennaMode::Draw => &self.heading_draw,
            HennaMode::Remove => &self.heading_remove,
        }
    }
}

/// O `\n` literal que o `0x1006b290` troca por quebra de linha de verdade antes de desenhar os
/// nomes adicionais.
pub fn expand_newlines(text: &str) -> String {
    text.replace("\\n", "\r\n")
}

/// O botão de fechar da barra de título, no lugar que o `FrameCtrl` desenha.
fn close_button(x: i32, y: i32) -> bool {
    let left = SIZE.0 - CLOSE_INSET;
    (left..left + BUTTON_SIZE).contains(&x) && (BUTTON_Y..BUTTON_Y + BUTTON_SIZE).contains(&y)
}

fn icon_or_black(name: &str) -> &str {
    match name.is_empty() {
        true => EMPTY_ICON,
        false => name,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HennaListView {
    pub mode: HennaMode,
    pub adena: i32,
    pub rows: Vec<HennaRow>,
    pub selected: Option<usize>,
    pub first_row: usize,
    pub scroll: ScrollWnd,
    pub frame: FrameCtrl,
    pub strings: HennaStrings,
}

impl HennaListView {
    pub fn new(strings: HennaStrings) -> HennaListView {
        let frame = FrameCtrl::titled(strings.title(HennaMode::Draw), true);
        HennaListView {
            mode: HennaMode::Draw,
            adena: 0,
            rows: Vec::new(),
            selected: None,
            first_row: 0,
            scroll: ScrollWnd::new(LIST_RECT.w, LIST_RECT.h),
            frame,
            strings,
        }
    }

    /// O cabeçalho do pacote (`0x10070d70` e `0x10071120`): esvazia a lista, guarda a adena e o
    /// modo. O segundo campo do cabeçalho não é usado pelo original.
    pub fn open(&mut self, mode: HennaMode, adena: i32) {
        self.mode = mode;
        self.adena = adena;
        self.rows.clear();
        self.selected = None;
        self.first_row = 0;
        self.frame = FrameCtrl::titled(self.strings.title(mode), true);
        self.update_scroll();
    }

    pub fn push(&mut self, row: HennaRow) {
        self.rows.push(row);
        self.update_scroll();
    }

    fn update_scroll(&mut self) {
        self.scroll.slider.set_range(rows_for(self.rows.len(), 1), LIST_ROWS as i32);
        self.scroll.slider.set_pos(self.first_row as i32);
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<HennaListHit> {
        if close_button(x, y) {
            return Some(HennaListHit::Close);
        }
        if let Some(hit) = self.scroll.hit(LIST_RECT.w, LIST_RECT.h, x - LIST_RECT.x, y - LIST_RECT.y) {
            return Some(HennaListHit::Scroll(hit));
        }
        let (rx, ry) = (x - LIST_RECT.x, y - LIST_RECT.y);
        if (0..=LIST_RECT.w).contains(&rx) && (0..=LIST_RECT.h).contains(&ry) && rx < LIST_RECT.w {
            let below = ry - ROW_TOP;
            let line = usize::try_from(below).ok()? / ROW_PITCH as usize;
            let index = self.first_row + line;
            if line < LIST_ROWS && index < self.rows.len() {
                return Some(HennaListHit::Row(index));
            }
        }
        None
    }

    pub fn scroll(&mut self, hit: ScrollHit) {
        let first = self.first_row as i32;
        let total = rows_for(self.rows.len(), 1);
        let next = match hit {
            ScrollHit::Up => line_up(first, LIST_ROWS as i32, total),
            ScrollHit::Down => line_down(first, LIST_ROWS as i32, total),
            _ => first,
        };
        self.first_row = next.max(0) as usize;
        self.scroll.slider.set_pos(next);
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        self.frame.paint(list, x, y, SIZE.0, SIZE.1, font, alpha);
        let Rect { x: bx, y: by, w, h } = BACK_RECT;
        list.draw_texture(x + bx, y + by, w, h, 0, 0, w, h, LIST_BACK, alpha);
        let (hx, hy) = HEADING_POS;
        list.draw_text(x + hx, y + hy, TEXT, self.strings.heading(self.mode), font);
        for (index, row) in self.rows.iter().enumerate().skip(self.first_row).take(LIST_ROWS) {
            let top = y + LIST_RECT.y + (index - self.first_row) as i32 * ROW_PITCH;
            let (ix, iy) = ROW_ICON_POS;
            let icon = icon_or_black(&row.icon);
            list.draw_texture(
                x + LIST_RECT.x + ix,
                top + iy,
                ICON_SIZE,
                ICON_SIZE,
                0,
                0,
                ICON_SIZE,
                ICON_SIZE,
                icon,
                alpha,
            );
            let tx = x + LIST_RECT.x + ROW_TEXT_X;
            list.draw_text(tx, top + ROW_LINE_Y[0], TEXT, &row.name, font);
            list.draw_text(tx, top + ROW_LINE_Y[1], TEXT, &row.add_name, font);
            let fee = format!("{} {}", self.strings.fee, row.price);
            list.draw_text(tx, top + ROW_LINE_Y[2], TEXT, &fee, font);
        }
        self.scroll.paint(list, x + LIST_RECT.x, y + LIST_RECT.y, LIST_RECT.w, LIST_RECT.h);
        let adena = self.adena.to_string();
        let width = font.measure(&adena) as i32;
        list.draw_text(x + LIST_ADENA_RIGHT - width, y + LIST_ADENA_Y, TEXT, &adena, font);
    }
}

/// Um dos dois blocos da janela de informação: o do corante e o do símbolo.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HennaBlock {
    pub name: String,
    pub icon: String,
    pub add_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HennaInfoView {
    pub mode: HennaMode,
    pub symbol_id: i32,
    pub price: i32,
    pub dye_amount: i32,
    pub adena: i32,
    pub dye: HennaBlock,
    pub symbol: HennaBlock,
    pub stats: [(i32, u8); 6],
    pub ok: TextButton,
    pub back: TextButton,
    pub pressed: Option<HennaInfoHit>,
    pub frame: FrameCtrl,
    pub strings: HennaStrings,
}

impl HennaInfoView {
    pub fn new(strings: HennaStrings) -> HennaInfoView {
        let frame = FrameCtrl::titled(strings.title(HennaMode::Draw), true);
        let back = TextButton::new(BACK_BTN_RECT, BTN_NORMAL, BTN_DOWN, &strings.back);
        let ok = TextButton::new(OK_BTN_RECT, BTN_NORMAL, BTN_DOWN, &strings.ok);
        HennaInfoView {
            mode: HennaMode::Draw,
            symbol_id: 0,
            price: 0,
            dye_amount: 0,
            adena: 0,
            dye: HennaBlock::default(),
            symbol: HennaBlock::default(),
            stats: [(0, 0); 6],
            ok,
            back,
            pressed: None,
            frame,
            strings,
        }
    }

    /// O que os handlers `0x10070fa0` e `0x10071390` fazem: zeram os doze atributos, guardam a
    /// adena e o modo e chamam o `SetInfo`.
    #[allow(clippy::too_many_arguments)]
    pub fn open(
        &mut self,
        mode: HennaMode,
        symbol_id: i32,
        dye_amount: i32,
        price: i32,
        adena: i32,
        dye: HennaBlock,
        symbol: HennaBlock,
        stats: [(i32, u8); 6],
    ) {
        self.mode = mode;
        self.symbol_id = symbol_id;
        self.dye_amount = dye_amount;
        self.price = price;
        self.adena = adena;
        self.dye = dye;
        self.symbol = symbol;
        self.stats = stats;
        self.pressed = None;
        self.frame = FrameCtrl::titled(self.strings.title(mode), true);
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<HennaInfoHit> {
        if close_button(x, y) {
            return Some(HennaInfoHit::Close);
        }
        if self.ok.contains(x, y) {
            return Some(HennaInfoHit::Ok);
        }
        if self.back.contains(x, y) {
            return Some(HennaInfoHit::Back);
        }
        None
    }

    fn block(&self, mode: HennaMode) -> (&HennaBlock, &HennaBlock) {
        match mode {
            HennaMode::Draw => (&self.dye, &self.symbol),
            HennaMode::Remove => (&self.symbol, &self.dye),
        }
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        self.frame.paint(list, x, y, SIZE.0, SIZE.1, font, alpha);
        let Rect { x: bx, y: by, w, h } = BACK_RECT;
        list.draw_texture(x + bx, y + by, w, h, 0, 0, w, h, INFO_BACK, alpha);
        let (first, second) = self.block(self.mode);
        let (first_label, second_label) = match self.mode {
            HennaMode::Draw => (&self.strings.dye, &self.strings.symbol),
            HennaMode::Remove => (&self.strings.symbol, &self.strings.dye),
        };
        let first_name = match self.mode {
            HennaMode::Draw => first.name.clone(),
            HennaMode::Remove => format!("{} {}", self.strings.title_remove, first.name),
        };
        list.draw_text(x + BLOCK_LABEL_X, y + FIRST_LABEL_Y, TEXT, first_label, font);
        self.icon(list, x, y + FIRST_ICON_Y, &first.icon, alpha);
        list.draw_text(x + BLOCK_TEXT_X, y + FIRST_NAME_Y, TEXT, &first_name, font);
        list.draw_text(x + BLOCK_TEXT_X, y + FIRST_ADD_Y, TEXT, &expand_newlines(&first.add_name), font);
        if self.mode == HennaMode::Draw {
            self.fee(list, x, y + DRAW_FEE_Y, font);
        }
        list.draw_text(x + BLOCK_LABEL_X, y + SECOND_LABEL_Y, TEXT, second_label, font);
        self.icon(list, x, y + SECOND_ICON_Y, &second.icon, alpha);
        list.draw_text(x + BLOCK_TEXT_X, y + SECOND_NAME_Y, TEXT, &second.name, font);
        list.draw_text(x + BLOCK_TEXT_X, y + SECOND_ADD_Y, TEXT, &expand_newlines(&second.add_name), font);
        if self.mode == HennaMode::Remove {
            self.fee(list, x, y + REMOVE_FEE_Y, font);
        }
        let (ax, ay) = ABILITY_LABEL_POS;
        list.draw_text(x + ax, y + ay, TEXT, &self.strings.ability, font);
        for (slot, (_, column, row, pair)) in STATS.iter().enumerate() {
            let ly = y + STAT_ROW_Y[*row];
            list.draw_text(x + STAT_LABEL_X[*column], ly, STAT_LABEL, &self.strings.stats[slot], font);
            let (from, to) = self.stats[*pair];
            let from = from.to_string();
            let to = to.to_string();
            let from_x = STAT_FROM_CENTER[*column] - font.measure(&from) as i32 / 2;
            let to_x = STAT_TO_CENTER[*column] - font.measure(&to) as i32 / 2;
            list.draw_text(x + from_x, ly, STAT_VALUE, &from, font);
            list.draw_text(x + STAT_ARROW_X[*column], ly, STAT_VALUE, STAT_ARROW, font);
            list.draw_text(x + to_x, ly, STAT_VALUE, &to, font);
        }
        let adena = self.adena.to_string();
        let width = font.measure(&adena) as i32;
        list.draw_text(x + INFO_ADENA_RIGHT - width, y + INFO_ADENA_Y, TEXT, &adena, font);
        self.back.paint(list, x, y, self.pressed == Some(HennaInfoHit::Back), font);
        self.ok.paint(list, x, y, self.pressed == Some(HennaInfoHit::Ok), font);
    }

    fn fee(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let fee = format!("{} {}", self.strings.fee, self.price);
        list.draw_text(x + BLOCK_TEXT_X, y, TEXT, &fee, font);
    }

    fn icon(&self, list: &mut DrawList, x: i32, y: i32, name: &str, alpha: f32) {
        let icon = icon_or_black(name);
        list.draw_texture(x + BLOCK_ICON_X, y, ICON_SIZE, ICON_SIZE, 0, 0, ICON_SIZE, ICON_SIZE, icon, alpha);
    }
}

pub fn grip() -> Rect {
    FrameCtrl::new(FrameMode::Title).grip(SIZE.0, SIZE.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Quad;
    use crate::font::{Glyphs, gly};
    use std::collections::HashMap;

    const CELL: i32 = 7;
    const ADVANCE: i32 = 6;

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * CELL, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    /// Remonta os textos de uma linha a partir dos quadriláteros dos glifos: no atlas de teste
    /// cada caractere tem largura 6 e fica em `u = (c - 0x20) * 7`, e o espaço não gera glifo, logo
    /// um vão de exatamente 6 é um espaço e um vão maior começa outro texto.
    fn line(quads: &[Quad], font: &BitmapFont, y: i32) -> Vec<(i32, String)> {
        let mut glyphs: Vec<(i32, char)> = quads
            .iter()
            .filter(|q| q.texture == font.atlas && q.y as i32 == y)
            .map(|q| (q.x as i32, char::from_u32(q.u as u32 / CELL as u32 + 0x20).unwrap_or('?')))
            .collect();
        glyphs.sort_by_key(|glyph| glyph.0);
        let mut runs: Vec<(i32, String)> = Vec::new();
        let mut end: Option<i32> = None;
        for (x, c) in glyphs {
            match end {
                Some(prev) if x == prev => runs.last_mut().expect("texto aberto").1.push(c),
                Some(prev) if x == prev + ADVANCE => {
                    let run = runs.last_mut().expect("texto aberto");
                    run.1.push(' ');
                    run.1.push(c);
                }
                _ => runs.push((x, c.to_string())),
            }
            end = Some(x + ADVANCE);
        }
        runs
    }

    fn at(quads: &[Quad], font: &BitmapFont, x: i32, y: i32, text: &str) -> bool {
        line(quads, font, y).iter().any(|(rx, run)| *rx == x && run == text)
    }

    fn strings() -> HennaStrings {
        let texts: HashMap<u32, &str> = [
            (0x28b, "Symbol Drawing"),
            (0x28c, "Deleting a Symbol"),
            (0x293, "Dye List"),
            (0x294, "List of Symbols"),
            (0x27d, "Fee"),
            (0x27e, "Dye Information"),
            (0x27f, "Symbol Information"),
            (0x280, "Change of Basic Ability"),
            (0x21f, "Back"),
            (0x8c, "OK"),
            (0x68, "STR"),
            (0x69, "DEX"),
            (0x6a, "CON"),
            (0x6b, "INT"),
            (0x6c, "WIT"),
            (0x6d, "MEN"),
        ]
        .into_iter()
        .collect();
        HennaStrings::new(move |id| texts.get(&id).map(|text| text.to_string()))
    }

    fn row(symbol_id: i32) -> HennaRow {
        HennaRow {
            symbol_id,
            dye_item_id: 4444 + symbol_id,
            dye_amount: 10,
            price: 33000,
            available: 1,
            name: "Symbol".into(),
            icon: "icon.etc_str_symbol_i01".into(),
            add_name: "Str+2".into(),
        }
    }

    /// Geometria conferida contra `NCHennaListWnd::OnCreate @ 0x10093930` (lista em
    /// `(7, 46, 241, 309)`, 6 linhas visíveis), `NCHennaListBox::OnPaint @ 0x10091c70` (ícone
    /// 32 × 32 em `(5, 13)`, textos em `x = 47` e `y = 11/23/36`, passo 48) e
    /// `NCHennaListWnd::OnPaint @ 0x10092100` (fundo `henna1_back` em `(0, 20, 256, 381)`,
    /// cabeçalho em `(15, 28)`, adena alinhada à direita de `x = 230`, `y = 374`, sem separador
    /// de milhar).
    #[test]
    fn the_list_follows_the_original_geometry() {
        let mut view = HennaListView::new(strings());
        view.open(HennaMode::Draw, 1234567);
        for id in 1..=8 {
            view.push(row(id));
        }
        let font = font();
        let quads = {
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font, 1.0);
            list.quads
        };
        let back = quads.iter().find(|q| q.texture == LIST_BACK).expect("fundo");
        assert_eq!((back.x, back.y, back.w, back.h), (0.0, 20.0, 256.0, 381.0));
        let icons: Vec<_> = quads.iter().filter(|q| q.texture == "icon.etc_str_symbol_i01").collect();
        assert_eq!(icons.len(), LIST_ROWS, "só as seis linhas visíveis são pintadas");
        assert_eq!((icons[0].x, icons[0].y, icons[0].w, icons[0].h), (12.0, 59.0, 32.0, 32.0));
        assert_eq!(icons[1].y - icons[0].y, 48.0);

        assert!(at(&quads, &font, 15, 28, "Dye List"));
        assert!(at(&quads, &font, 54, 57, "Symbol"));
        assert!(at(&quads, &font, 54, 69, "Str+2"));
        assert!(at(&quads, &font, 54, 82, "Fee 33000"));
        assert!(at(&quads, &font, 54, 105, "Symbol"), "segunda linha um passo abaixo");
        assert!(at(&quads, &font, 230 - font.measure("1234567") as i32, 374, "1234567"));
    }

    /// O clique numa linha vira `Row`, a barra vira `Scroll`, a barra de título vira `Close`, e a
    /// janela não tem mais nenhum controle.
    #[test]
    fn the_list_hit_test_only_answers_rows_the_scroll_bar_and_the_close_button() {
        let mut view = HennaListView::new(strings());
        view.open(HennaMode::Draw, 0);
        view.push(row(1));
        view.push(row(2));
        assert_eq!(view.hit(LIST_RECT.x + 1, LIST_RECT.y + 1), None, "as linhas começam em TextY = 9");
        assert_eq!(view.hit(LIST_RECT.x + 1, LIST_RECT.y + ROW_TOP), Some(HennaListHit::Row(0)));
        assert_eq!(view.hit(LIST_RECT.x + 1, LIST_RECT.y + ROW_TOP + 48), Some(HennaListHit::Row(1)));
        assert_eq!(view.hit(LIST_RECT.x + 1, LIST_RECT.y + ROW_TOP + 96), None, "linha vazia não responde");
        assert_eq!(view.hit(2, 300), None);
        assert_eq!(view.hit(SIZE.0 - CLOSE_INSET, BUTTON_Y), Some(HennaListHit::Close));
        assert!(matches!(view.hit(LIST_RECT.x + LIST_RECT.w - 3, LIST_RECT.y + 2), Some(HennaListHit::Scroll(_))));
    }

    /// A rolagem anda uma linha por vez e para no fim, como o `ScrollWnd` que o `0x1011b930`
    /// alimenta com a contagem e a posição.
    #[test]
    fn the_list_scrolls_one_row_at_a_time() {
        let mut view = HennaListView::new(strings());
        view.open(HennaMode::Draw, 0);
        for id in 1..=8 {
            view.push(row(id));
        }
        view.scroll(ScrollHit::Down);
        assert_eq!(view.first_row, 1);
        view.scroll(ScrollHit::Down);
        view.scroll(ScrollHit::Down);
        assert_eq!(view.first_row, 2, "oito linhas com seis visíveis param no topo 2");
        view.scroll(ScrollHit::Up);
        assert_eq!(view.first_row, 1);
        assert_eq!(view.hit(LIST_RECT.x + 1, LIST_RECT.y + ROW_TOP + 1), Some(HennaListHit::Row(1)));
    }

    /// Abrir de novo esvazia a lista e troca o título, como o cabeçalho do pacote faz.
    #[test]
    fn opening_again_empties_the_list_and_changes_the_title() {
        let mut view = HennaListView::new(strings());
        view.open(HennaMode::Draw, 10);
        view.push(row(1));
        assert_eq!(view.frame.title, "Symbol Drawing");
        view.first_row = 1;
        view.open(HennaMode::Remove, 20);
        assert!(view.rows.is_empty());
        assert_eq!((view.first_row, view.adena, view.mode), (0, 20, HennaMode::Remove));
        assert_eq!(view.frame.title, "Deleting a Symbol");
    }

    fn info() -> HennaInfoView {
        let mut view = HennaInfoView::new(strings());
        view.open(
            HennaMode::Draw,
            145,
            10,
            33000,
            999,
            HennaBlock { name: "Dye".into(), icon: "icon.dye".into(), add_name: "Str+2".into() },
            HennaBlock { name: "Symbol".into(), icon: "icon.sym".into(), add_name: "Con-2".into() },
            [(11, 12), (30, 32), (43, 41), (10, 10), (20, 20), (25, 25)],
        );
        view
    }

    /// Posições conferidas contra `NCHennaInfoWnd::OnCreate @ 0x100934a0` (`Back` em `(51, 372)`,
    /// `OK` em `(131, 372)`) e `OnPaint @ 0x10092370` (bloco do corante em `43/71/66/80`, taxa em
    /// `94`, bloco do símbolo em `132/161/156/170`, rótulo dos atributos em `(18, 221)`, adena à
    /// direita de `x = 229`, `y = 329`). O ícone de adena não é pintado porque o `+0xec` desta
    /// classe nunca recebe valor.
    #[test]
    fn the_info_window_draws_the_dye_block_first_when_drawing() {
        let view = info();
        let font = font();
        let quads = {
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font, 1.0);
            list.quads
        };
        let back = quads.iter().find(|q| q.texture == INFO_BACK).expect("fundo");
        assert_eq!((back.x, back.y, back.w, back.h), (0.0, 20.0, 256.0, 381.0));
        let dye = quads.iter().find(|q| q.texture == "icon.dye").expect("ícone do corante");
        assert_eq!((dye.x, dye.y, dye.w, dye.h), (16.0, 71.0, 32.0, 32.0));
        let symbol = quads.iter().find(|q| q.texture == "icon.sym").expect("ícone do símbolo");
        assert_eq!((symbol.x, symbol.y), (16.0, 161.0));
        assert_eq!(quads.iter().filter(|q| q.texture == "L2UI.WindowIcon.WindowIconAdena").count(), 0);
        assert_eq!(quads.iter().filter(|q| q.texture == BTN_NORMAL).count(), 2);

        assert!(at(&quads, &font, 18, 43, "Dye Information"));
        assert!(at(&quads, &font, 58, 66, "Dye"));
        assert!(at(&quads, &font, 58, 80, "Str+2"));
        assert!(at(&quads, &font, 58, 94, "Fee 33000"));
        assert!(at(&quads, &font, 18, 132, "Symbol Information"));
        assert!(at(&quads, &font, 58, 156, "Symbol"));
        assert!(at(&quads, &font, 58, 170, "Con-2"));
        assert!(at(&quads, &font, 18, 221, "Change of Basic Ability"));
        assert!(at(&quads, &font, 229 - font.measure("999") as i32, 329, "999"));
        assert_eq!(view.hit(OK_BTN_RECT.x + 2, OK_BTN_RECT.y + 2), Some(HennaInfoHit::Ok));
        assert_eq!(view.hit(BACK_BTN_RECT.x + 2, BACK_BTN_RECT.y + 2), Some(HennaInfoHit::Back));
        assert_eq!(view.hit(SIZE.0 - CLOSE_INSET, BUTTON_Y), Some(HennaInfoHit::Close));
        assert_eq!(view.hit(2, 2), None);
    }

    /// No modo de apagar os dois blocos trocam de lugar, o nome do símbolo vem com a sysstring
    /// `0x28c` na frente (formato `"%s %s"`) e a taxa desce para `y = 184`.
    #[test]
    fn the_info_window_swaps_the_blocks_when_removing() {
        let mut view = info();
        view.mode = HennaMode::Remove;
        let font = font();
        let quads = {
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font, 1.0);
            list.quads
        };
        let symbol = quads.iter().find(|q| q.texture == "icon.sym").expect("ícone do símbolo");
        assert_eq!((symbol.x, symbol.y), (16.0, 71.0));
        let dye = quads.iter().find(|q| q.texture == "icon.dye").expect("ícone do corante");
        assert_eq!((dye.x, dye.y), (16.0, 161.0));
        assert!(at(&quads, &font, 18, 43, "Symbol Information"));
        assert!(at(&quads, &font, 58, 66, "Deleting a Symbol Symbol"));
        assert!(at(&quads, &font, 58, 80, "Con-2"));
        assert!(at(&quads, &font, 18, 132, "Dye Information"));
        assert!(at(&quads, &font, 58, 156, "Dye"));
        assert!(at(&quads, &font, 58, 170, "Str+2"));
        assert!(at(&quads, &font, 58, 184, "Fee 33000"));
        assert!(!at(&quads, &font, 58, 94, "Fee 33000"), "a taxa não fica nos dois lugares");
    }

    /// Cada rótulo de atributo fica com o par que o original desenha ao lado dele: `STR` com o 2º
    /// par, `DEX` com o 5º, `CON` com o 3º, `INT` com o 1º, `WIT` com o 6º e `MEN` com o 4º; os
    /// valores são centrados e separados pelo `>` de `0x101aa958`.
    #[test]
    fn each_stat_label_shows_the_pair_the_original_puts_beside_it() {
        let view = info();
        let font = font();
        let quads = {
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font, 1.0);
            list.quads
        };
        let here = |x: i32, y: i32, text: &str| at(&quads, &font, x, y, text);
        assert!(here(17, 247, "STR") && here(17, 263, "DEX") && here(17, 279, "CON"));
        assert!(here(135, 247, "INT") && here(135, 263, "WIT") && here(135, 279, "MEN"));
        let centered = |value: &str, center: i32| center - font.measure(value) as i32 / 2;
        assert!(here(centered("30", 64), 247, "30") && here(centered("32", 97), 247, "32"), "STR = 2º par");
        assert!(here(centered("20", 64), 263, "20"), "DEX = 5º par");
        assert!(here(centered("43", 64), 279, "43") && here(centered("41", 97), 279, "41"), "CON = 3º par");
        assert!(here(centered("11", 182), 247, "11") && here(centered("12", 215), 247, "12"), "INT = 1º par");
        assert!(here(centered("25", 182), 263, "25"), "WIT = 6º par");
        assert!(here(centered("10", 182), 279, "10"), "MEN = 4º par");
        assert!(here(78, 247, ">") && here(196, 279, ">"));
        for y in STAT_ROW_Y {
            assert_eq!(line(&quads, &font, y).iter().filter(|(_, run)| run == ">").count(), 2);
        }
    }

    /// O `0x1006b290` troca o `\n` literal por quebra de linha antes de desenhar.
    #[test]
    fn literal_backslash_n_becomes_a_line_break() {
        assert_eq!(expand_newlines("a\\nb"), "a\r\nb");
        assert_eq!(expand_newlines("Str+2 Con-2"), "Str+2 Con-2");
    }

    /// Ícone vazio cai na textura preta que o `SetInfo` e o `AddEntry` usam de reserva.
    #[test]
    fn an_empty_icon_falls_back_to_the_black_texture() {
        let mut view = HennaListView::new(strings());
        view.open(HennaMode::Remove, 0);
        view.push(HennaRow { icon: String::new(), ..row(1) });
        let quads = {
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font(), 1.0);
            list.quads
        };
        assert!(quads.iter().any(|q| q.texture == EMPTY_ICON));
        let mut info = info();
        info.dye.icon = String::new();
        let quads = {
            let mut list = DrawList::default();
            info.paint(&mut list, 0, 0, &font(), 1.0);
            list.quads
        };
        assert!(quads.iter().any(|q| q.texture == EMPTY_ICON));
    }
}
