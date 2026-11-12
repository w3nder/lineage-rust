#[path = "macro_edit_layout_generated.rs"]
mod layout;

use crate::button::TextButton;
use crate::chat::{InputKey, TextInput};
use crate::edit_box::{EditBox, FontMetrics};
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::window::Rect;

pub const LIST_SIZE: (i32, i32) = (256, 401);
pub const INFO_SIZE: (i32, i32) = (256, 161);

pub fn list_default_position(viewport: (i32, i32)) -> (i32, i32) {
    crate::mainwnd::default_position(viewport)
}

pub fn list_initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    crate::mainwnd::initial_position(viewport)
}

pub fn edit_default_position(viewport: (i32, i32)) -> (i32, i32) {
    crate::mainwnd::default_position(viewport)
}

pub fn edit_initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    crate::mainwnd::initial_position(viewport)
}

pub fn info_default_position(viewport: (i32, i32)) -> (i32, i32) {
    (20, crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) * 0.5 - 252.0 + 80.0))
}

pub fn info_initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    let (x, y) = info_default_position(viewport);
    crate::initial_position::InitialPosition {
        enabled: true,
        horizontal_mode: 0,
        vertical_mode: 5,
        x,
        y: (viewport.1 / 2).wrapping_sub(y),
    }
}

pub const LIST_BACK: &str = "l2ui_ch3.macrownd.macro2_back";
pub const INFO_BACK: &str = "l2ui_ch3.macrownd.macro3_back";
pub const ICON_PREFIX: &str = "L2UI.MacroWnd.Macro_Icon";
pub const EDIT: &str = "L2UI_CH3.macrownd.macro_edit";
pub const EDIT_DRAG: &str = "L2UI_CH3.macrownd.macro_edit_Drag";
pub const TRASH: &str = "L2UI_CH3.InventoryWnd.Inventory_trash";
pub const TRASH_DRAG: &str = "L2UI_CH3.InventoryWnd.Inventory_trash_drag";
/// Ícones de soltar da janela de lista, com as posições e o tamanho do
/// `NCMacroListWnd::OnCreate @ 0x100ad500` (`+0x10c`/`+0x110` e `+0xfc`/`+0x100`, os dois com
/// `+0x104`..`+0x118` = 34.0). A dica de cada um é `0x290` e `0x37c` (`FUN_100324c0`).
pub const EDIT_ICON_RECT: Rect = Rect::new(169, 347, 34, 34);
pub const TRASH_ICON_RECT: Rect = Rect::new(208, 347, 34, 34);
pub const EDIT_ICON_SYSSTRING: u32 = 0x290;
pub const TRASH_ICON_SYSSTRING: u32 = 0x37c;
/// Teto de macros do cliente: o `OnPaint` escreve `(n/24)` com o 24 fixo (`0x100a8f9f`), e o
/// botão de adicionar recusa a partir do 24º (`cmpl $0x18` em `0x100a90b1`).
pub const MACROS_MAX: usize = 24;
pub const ARRAY_RECT: Rect = Rect::new(6, 45, 244, 285);
pub const COLS: usize = 6;
pub const ROWS: usize = 8;
pub const CELL_STEP: i32 = 37;
pub const CELL_ROW_STEP: i32 = 35;
pub const CELL_ORIGIN: i32 = 4;
pub const CELL_COUNT: usize = 42;

fn inclusive_box(rect: Rect, x: i32, y: i32) -> bool {
    (rect.x..=rect.x + rect.w).contains(&x) && (rect.y..=rect.y + rect.h).contains(&y)
}
pub const ICON_SIZE: i32 = 32;
pub const ADD_RECT: Rect = Rect::new(11, 347, 76, 23);
pub const HELP_RECT: Rect = Rect::new(11, 372, 76, 23);
/// Caixa de texto da janela de descrição, id `0x1514` em `(9, 48, 238, 77)`
/// (`NCMacroInfoWnd::OnCreate @ 0x100abe30`). É o único conteúdo da janela: nome e lista de
/// comandos ali dentro eram invenção nossa.
pub const INFO_TEXT_RECT: Rect = Rect::new(9, 48, 238, 77);
pub const INFO_LINE_STEP: i32 = 13;
pub const INFO_OK_RECT: Rect = Rect::new(51, 132, 76, 23);
pub const INFO_CANCEL_RECT: Rect = Rect::new(131, 132, 76, 23);
pub const EDIT_SIZE: (i32, i32) = layout::SIZE;
pub const EDIT_NAME_RECT: Rect = layout::RECT_108;
pub const EDIT_ACRONYM_RECT: Rect = layout::RECT_10C;
pub const EDIT_MORE_INFO_RECT: Rect = layout::RECT_104;
pub const EDIT_PREV_RECT: Rect = layout::RECT_FC;
pub const EDIT_NEXT_RECT: Rect = layout::RECT_100;
pub const EDIT_COMMAND_LIST_RECT: Rect = layout::RECT_EC;
pub const EDIT_COMMAND_ROWS: usize = 9;
pub const MACRO_COMMANDS_MAX: usize = 12;
/// Texto de comando que o original recusa acima de 32 caracteres (`cmpl $0x20` em `0x100ae20b`).
pub const MACRO_COMMAND_MAX_CHARS: usize = 0x20;
pub const EDIT_COMMAND_STEP: i32 = 26;
/// Caixas de texto das linhas, criadas por `NCMacroEditList::OnCreate @ 0x100ab6a0` em
/// `(20, 7 + 26·i, 202, 17)`, ids `0x1506 + i`, doze linhas com as três últimas escondidas.
pub const EDIT_LINE_BOX: Rect = Rect::new(20, 7, 202, 17);
/// Número da linha (`"0%d"`/`"%d"`) de `NCMacroEditList::OnPaint @ 0x100ab830` em `(2, 11 + 26·i)`.
pub const EDIT_NUMBER_X: i32 = 2;
pub const EDIT_NUMBER_Y: i32 = 11;
pub const EDIT_HELP_RECT: Rect = layout::RECT_118;
pub const EDIT_SAVE_RECT: Rect = layout::RECT_114;
pub const EDIT_CANCEL_RECT: Rect = layout::RECT_11C;

/// Um comando de macro como ele viaja nos dois sentidos: o `MacroListPacket` e o
/// `RequestMakeMacro` levam `ccdcS` por comando (`docs/evidence/ui/packet-formats.txt`), ou seja
/// índice, tipo, skill, atalho e texto. A janela de edição só mexe no texto, mas precisa
/// devolver os outros três iguais — um comando de skill (`kind = 1`) não tem texto nenhum, e
/// zerar o tipo apaga o comando no servidor.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MacroCommandView {
    pub kind: u8,
    pub skill_id: i32,
    pub shortcut_id: u8,
    pub command: String,
    /// Linha que a janela mostra. Só o comando de texto livre (`kind == 3`) é mostrado como
    /// veio; os outros o original remonta a partir dos campos numéricos (ver `macro_line`).
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroView { pub id: i32, pub name: String, pub description: String, pub acronym: String, pub icon: u8, pub commands: Vec<MacroCommandView> }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroHit { Macro(usize), CommandLine(usize), Add, EditIcon, Trash, Close, InfoOk, InfoCancel, MoreInfo, Help, Prev, Next }

/// Campo de texto com o foco no editor. As doze linhas de comando são caixas de texto da mesma
/// classe do nome e da sigla (`NCMacroEditList::OnCreate @ 0x100ab6a0` chama
/// `StaticAllocateObject(&DAT_102bdca0, …)` e `FUN_10011420(0, 0, 1, 0x108, 0)`, os mesmos do
/// `NCMacroEditWnd::OnCreate`), com o limite de 32 caracteres de `FUN_1000b790(0x20)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditFocus { Name, Acronym, Command(usize) }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroListView {
    pub title: String,
    pub count: usize,
    pub total: usize,
    pub macros: Vec<MacroView>,
    pub selected: Option<usize>,
    pub pressed: Option<MacroHit>,
    pub add: TextButton,
    pub help: TextButton,
    pub frame: FrameCtrl,
    pub dragging: Option<usize>,
    /// Ícone sob o cursor enquanto um macro está sendo arrastado — é o que troca a arte pro
    /// `_Drag` (`+0x11c`/`+0x120` no `NCMacroListWnd::OnPaint @ 0x100a8dd0`).
    pub hover: Option<MacroHit>,
}

impl MacroListView {
    /// Título e rótulos conferidos contra o original: `NCMacroListWnd::OnPaint @ 0x100a8dd0`
    /// desenha o título de `GL2GameData + 0x9a5d0`, que é a sysstring `0x28f` ("Macro List") —
    /// a tabela de strings da `FL2GameData` é um array de `FString` de 12 bytes começando em
    /// `+0x9871c`, então `id = (offset - 0x9871c) / 12` (fórmula conferida com os rótulos
    /// "Name"=`0x32` e "Acronym"=`0x28d` da janela de edição).
    ///
    /// A janela tem dois botões de texto — `0x100f` em `(11, 347)` com a sysstring `0x28e`
    /// ("Add") e `0x100c` em `(11, 372)` com a `0x91` ("Help") — e dois ícones de soltar, o de
    /// edição e a lixeira.
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let title = lookup(0x28f).unwrap_or_else(|| "Macro List".into());
        let add_label = lookup(0x28e).unwrap_or_else(|| "Add".into());
        let help_label = lookup(0x91).unwrap_or_else(|| "Help".into());
        Self {
            title: title.clone(), count: 0, total: 0, macros: Vec::new(), selected: None, pressed: None,
            add: TextButton::new(ADD_RECT, "L2UI_ch3.button.btn1_Normal", "L2UI_ch3.button.btn1_NormalOn", &add_label),
            help: TextButton::new(HELP_RECT, "L2UI_ch3.button.btn1_Normal", "L2UI_ch3.button.btn1_NormalOn", &help_label),
            frame: FrameCtrl::titled(&title, true),
            dragging: None,
            hover: None,
        }
    }

    pub fn set_macros(&mut self, macros: Vec<MacroView>, total: usize) {
        let selected_id = self.selected.and_then(|index| self.macros.get(index)).map(|macro_entry| macro_entry.id);
        self.macros = macros;
        self.count = self.macros.len();
        self.total = total;
        self.selected = selected_id.and_then(|id| self.macros.iter().position(|macro_entry| macro_entry.id == id));
        self.pressed = None;
    }

    pub fn cell_at(&self, x: i32, y: i32) -> Option<usize> {
        (0..CELL_COUNT).find(|index| {
            let cx = ARRAY_RECT.x + CELL_ORIGIN + (index % COLS) as i32 * CELL_STEP;
            let cy = ARRAY_RECT.y + CELL_ORIGIN + (index / COLS) as i32 * CELL_ROW_STEP;
            (cx..=cx + ICON_SIZE).contains(&x) && (cy..=cy + ICON_SIZE).contains(&y)
        })
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<MacroHit> {
        if x >= 233 && y < 20 { return Some(MacroHit::Close); }
        if self.add.contains(x, y) { return Some(MacroHit::Add); }
        if self.help.contains(x, y) { return Some(MacroHit::Help); }
        if inclusive_box(EDIT_ICON_RECT, x, y) { return Some(MacroHit::EditIcon); }
        if inclusive_box(TRASH_ICON_RECT, x, y) { return Some(MacroHit::Trash); }
        self.cell_at(x, y).filter(|index| *index < self.macros.len()).map(MacroHit::Macro)
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, _alpha: f32) {
        let alpha = 1.0;
        self.frame.paint(list, x, y, LIST_SIZE.0, LIST_SIZE.1, font, alpha);
        list.draw_texture(x, y + 20, 256, 381, 0, 0, 256, 381, LIST_BACK, alpha);
        list.draw_text(x + 11, y + 32, Argb(0xffdcdcdc), &self.title, font);
        // `(0%d/%d)` abaixo de dez, `(%d/%d)` daí pra cima, e o segundo número é sempre o teto
        // do cliente — `appSprintf(…, +0x130, 0x18)` em `0x100a8f9f`.
        let counter = if self.count < 10 {
            format!("(0{}/{})", self.count, MACROS_MAX)
        } else {
            format!("({}/{})", self.count, MACROS_MAX)
        };
        list.draw_text(x + 200, y + 32, Argb(0xffdcdcdc), &counter, font);
        let icon_art = |hit: MacroHit, normal: &'static str, drag: &'static str| {
            if self.dragging.is_some() && self.hover == Some(hit) { drag } else { normal }
        };
        let (ew, eh) = (EDIT_ICON_RECT.w, EDIT_ICON_RECT.h);
        list.draw_texture(x + EDIT_ICON_RECT.x, y + EDIT_ICON_RECT.y, ew, eh, 0, 0, ew, eh, icon_art(MacroHit::EditIcon, EDIT, EDIT_DRAG), alpha);
        let (tw, th) = (TRASH_ICON_RECT.w, TRASH_ICON_RECT.h);
        list.draw_texture(x + TRASH_ICON_RECT.x, y + TRASH_ICON_RECT.y, tw, th, 0, 0, tw, th, icon_art(MacroHit::Trash, TRASH, TRASH_DRAG), alpha);
        for (index, macro_entry) in self.macros.iter().take(COLS * ROWS).enumerate() {
            let col = index % COLS;
            let row = index / COLS;
            let ix = x + ARRAY_RECT.x + CELL_ORIGIN + col as i32 * CELL_STEP;
            let iy = y + ARRAY_RECT.y + CELL_ORIGIN + row as i32 * CELL_ROW_STEP;
            let icon = macro_icon(macro_entry);
            list.draw_texture(ix, iy, ICON_SIZE, ICON_SIZE, 0, 0, ICON_SIZE, ICON_SIZE, &icon, alpha);
            list.draw_texture(ix - 1, iy - 1, 34, 34, 0, 0, 34, 34, crate::inventory::OUTLINE, alpha);
            paint_acronym(list, ix + 4, iy + 4, &macro_entry.acronym, font);
            if self.selected == Some(index) { list.draw_texture(ix - 1, iy - 1, 34, 34, 0, 0, 34, 34, crate::inventory::OUTLINE_DOWN, alpha); }
        }
        self.add.paint(list, x, y, self.pressed == Some(MacroHit::Add), font);
        self.help.paint(list, x, y, self.pressed == Some(MacroHit::Help), font);
    }
}

#[derive(Debug, Clone)]
pub struct MacroInfoView {
    pub title: String,
    pub macro_entry: Option<MacroView>,
    pub ok: TextButton,
    pub cancel: TextButton,
    pub frame: FrameCtrl,
    pub edit: crate::multi_edit::MultiEdit,
    pub stored: String,
    pub font: Option<BitmapFont>,
    pub pressed: Option<MacroHit>,
}

pub const INFO_EDIT_ID: u32 = 0x1514;
pub const INFO_DESCRIPTION_MAX_CHARS: usize = 32;
pub const INFO_TOO_LONG_SYSMSG: u32 = 0x345;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroEditView {
    pub title: String,
    pub name_label: String,
    pub acronym_label: String,
    pub macro_entry: MacroView,
    pub name: EditBox,
    pub description: TextInput,
    pub acronym: EditBox,
    pub lines: Vec<EditBox>,
    pub font: Option<BitmapFont>,
    pub command_scroll: usize,
    pub scroll: crate::scroll::ScrollWnd,
    pub scroll_pressed: Option<crate::scroll::ScrollHit>,
    pub focus: EditFocus,
    pub icon: u8,
    pub save: TextButton,
    pub cancel: TextButton,
    pub help: TextButton,
    pub more_info: TextButton,
    pub prev: TextButton,
    pub next: TextButton,
    pub frame: FrameCtrl,
    pub pressed: Option<MacroHit>,
}

pub const EDIT_NAME_ID: u32 = 0x1512;
pub const EDIT_ACRONYM_ID: u32 = 0x1513;
pub const EDIT_LINE_ID: u32 = 0x1506;
pub const EDIT_NAME_MAX: i32 = 0xc;
pub const EDIT_ACRONYM_MAX: i32 = 4;

fn name_box() -> EditBox {
    let mut edit = EditBox::native(EDIT_NAME_ID, EDIT_NAME_RECT);
    edit.set_max_len(EDIT_NAME_MAX);
    edit
}

fn acronym_box() -> EditBox {
    let mut edit = EditBox::native(EDIT_ACRONYM_ID, EDIT_ACRONYM_RECT);
    edit.set_max_len(EDIT_ACRONYM_MAX);
    edit
}

fn line_rect(row: i32) -> Rect {
    Rect::new(
        EDIT_COMMAND_LIST_RECT.x + EDIT_LINE_BOX.x,
        EDIT_COMMAND_LIST_RECT.y + EDIT_LINE_BOX.y + row * EDIT_COMMAND_STEP,
        EDIT_LINE_BOX.w,
        EDIT_LINE_BOX.h,
    )
}

fn line_boxes() -> Vec<EditBox> {
    (0..MACRO_COMMANDS_MAX)
        .map(|line| {
            let mut edit = EditBox::native(EDIT_LINE_ID + line as u32, line_rect(line as i32));
            edit.set_max_len(MACRO_COMMAND_MAX_CHARS as i32);
            edit
        })
        .collect()
}

struct NoMetrics;

impl crate::edit_box::Metrics for NoMetrics {
    fn text_width(&self, _: &[u16]) -> i32 {
        0
    }

    fn char_width(&self, _: u16) -> i32 {
        0
    }
}

fn fill_box(edit: &mut EditBox, text: &str, m: &dyn crate::edit_box::Metrics) {
    edit.clear();
    edit.insert_text(text, true, m);
}

impl MacroEditView {
    pub fn scroll_down(&mut self, x: i32, y: i32) -> bool {
        let r = EDIT_COMMAND_LIST_RECT;
        let Some(hit) = self.scroll.hit(r.w, r.h, x - r.x, y - r.y) else { return false };
        self.scroll_pressed = Some(hit);
        self.scroll.up_pressed = hit == crate::scroll::ScrollHit::Up;
        self.scroll.down_pressed = hit == crate::scroll::ScrollHit::Down;
        if let crate::scroll::ScrollHit::Slider(y) = hit { self.scroll.slider.mouse_down(y); }
        true
    }

    pub fn scroll_move(&mut self, y: i32) -> bool {
        if !self.scroll.slider.dragging { return false; }
        if let Some(first) = self.scroll.slider.mouse_move(y - EDIT_COMMAND_LIST_RECT.y - crate::scroll::BUTTON_SIZE) {
            self.command_scroll = first.clamp(0, 3) as usize;
            self.layout_lines();
            self.settle_scroll();
        }
        true
    }

    pub fn scroll_up(&mut self, over: Option<(i32, i32)>) {
        let pressed = self.scroll_pressed.take();
        let r = EDIT_COMMAND_LIST_RECT;
        let hit = over.and_then(|(x, y)| self.scroll.hit(r.w, r.h, x - r.x, y - r.y));
        self.scroll.up_pressed = false;
        self.scroll.down_pressed = false;
        self.scroll.slider.mouse_up();
        if pressed == hit {
            match pressed {
                Some(crate::scroll::ScrollHit::Up) => self.scroll_commands(1),
                Some(crate::scroll::ScrollHit::Down) => self.scroll_commands(-1),
                _ => {}
            }
        }
    }

    pub fn hit_line(&self, x: i32, y: i32) -> Option<usize> {
        self.command_line_at(x, y).filter(|line| self.lines[*line].rect.contains(x, y))
    }

    pub fn mark_line(&mut self, line: usize) {
        if let Some(edit) = self.lines.get_mut(line) {
            edit.marked = true;
        }
    }

    pub fn clear_marks(&mut self) {
        for edit in &mut self.lines {
            edit.marked = false;
        }
    }

    pub fn clear_mark(&mut self, line: usize) {
        if let Some(edit) = self.lines.get_mut(line) {
            edit.marked = false;
        }
    }

    pub fn command_line_at(&self, x: i32, y: i32) -> Option<usize> {
        if !EDIT_COMMAND_LIST_RECT.contains(x, y) {
            return None;
        }
        if x >= EDIT_COMMAND_LIST_RECT.x + EDIT_COMMAND_LIST_RECT.w - crate::scroll::SCROLL_WIDTH {
            return None;
        }
        let row = (y - EDIT_COMMAND_LIST_RECT.y) / EDIT_COMMAND_STEP;
        let line = self.command_scroll + usize::try_from(row).ok()?;
        (row < EDIT_COMMAND_ROWS as i32 && line < MACRO_COMMANDS_MAX).then_some(line)
    }

    /// Rola a lista de comandos. O original cria a lista com `FUN_100acb00(9, 0xc)` no
    /// `NCMacroEditWnd::OnCreate @ 0x100acc50`: nove linhas visíveis de doze, e a `NCMacroEditList`
    /// tem os slots de rolagem `0x12c`/`0x130`/`0x134` mais a roda em `0x124`. Sem isso as três
    /// últimas linhas ficam inalcançáveis.
    pub fn scroll_commands(&mut self, lines: i32) {
        let last = MACRO_COMMANDS_MAX - EDIT_COMMAND_ROWS;
        let next = self.command_scroll as i32 - lines;
        self.command_scroll = next.clamp(0, last as i32) as usize;
        self.scroll.slider.set_pos(self.command_scroll as i32);
        self.layout_lines();
        self.settle_scroll();
    }

    /// Põe um comando na linha, como `0x100ae210`..`0x100ae23f` faz: limpa a linha e escreve o
    /// texto. Linhas que ainda não existem são preenchidas com comandos vazios, porque a lista do
    /// original tem sempre as doze.
    pub fn set_command(&mut self, line: usize, command: MacroCommandView) -> bool {
        if line >= MACRO_COMMANDS_MAX || command.label.chars().count() > MACRO_COMMAND_MAX_CHARS {
            return false;
        }
        let label = command.label.clone();
        self.set_command_label(line, command);
        self.scope(|view, m| fill_box(&mut view.lines[line], &label, m));
        true
    }
    /// Ids conferidos um a um contra `NCMacroEditWnd::OnCreate @ 0x100acc50`, que chama
    /// `FUN_100014a0(<sysstring>)` logo depois de criar cada botão, e contra
    /// `NCMacroEditWnd::OnPaint @ 0x100aa400` para o título e os dois rótulos cinzas:
    /// botão em x=11 -> `0x91` "Help", x=91 -> `0x292` "Save", x=171 -> `0x8d` "Cancel",
    /// o botãozinho em (177, 82) -> `0x291` "More Info.", título -> `0x290` "Macro Edit",
    /// rótulos -> `0x32` "Name" e `0x28d` "Acronym". A versão anterior tinha três deles
    /// trocados (`0x91` usado como "salvar" e `0x90`, que é "All", como "ajuda"), o que fazia a
    /// janela aparecer com "All | Help | Cancel" em vez de "Help | Save | Cancel".
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let title = lookup(0x290).unwrap_or_else(|| "Macro Edit".into());
        let save = lookup(0x292).unwrap_or_else(|| "Save".into());
        let cancel = lookup(0x8d).unwrap_or_else(|| "Cancel".into());
        let help = lookup(0x91).unwrap_or_else(|| "Help".into());
        let more_info = lookup(0x291).unwrap_or_else(|| "More Info.".into());
        let name_label = lookup(0x32).unwrap_or_else(|| "Name".into());
        let acronym_label = lookup(0x28d).unwrap_or_else(|| "Acronym".into());
        let empty = MacroView { id: 0, name: String::new(), description: String::new(), acronym: String::new(), icon: 1, commands: Vec::new() };
        Self { title: title.clone(), name_label, acronym_label, macro_entry: empty, name: name_box(), description: TextInput::default(), acronym: acronym_box(), lines: line_boxes(), font: None, command_scroll: 0, scroll: { let mut scroll = crate::scroll::ScrollWnd::new(242, 239); scroll.slider.set_range(12, 9); scroll }, scroll_pressed: None, focus: EditFocus::Name, icon: 1, save: TextButton::new(EDIT_SAVE_RECT, "L2UI_ch3.button.btn1_Normal", "L2UI_ch3.button.btn1_NormalOn", &save), cancel: TextButton::new(EDIT_CANCEL_RECT, "L2UI_ch3.button.btn1_Normal", "L2UI_ch3.button.btn1_NormalOn", &cancel), help: TextButton::new(EDIT_HELP_RECT, "L2UI_ch3.button.btn1_Normal", "L2UI_ch3.button.btn1_NormalOn", &help), more_info: TextButton::new(EDIT_MORE_INFO_RECT, "L2UI_ch3.button.SmallButton2", "L2UI_ch3.button.SmallButton2_Down", &more_info), prev: TextButton::new(EDIT_PREV_RECT, "L2UI_ch3.Button.Prev1", "L2UI_ch3.Button.Prev1_Down", ""), next: TextButton::new(EDIT_NEXT_RECT, "L2UI_ch3.Button.Next1", "L2UI_ch3.Button.Next1_Down", ""), frame: FrameCtrl::titled(&title, true), pressed: None }
    }

    pub fn set_font(&mut self, font: &BitmapFont) {
        self.font = Some(font.clone());
    }

    pub fn set_language(&mut self, language: crate::ime::Language) {
        for edit in self.lines.iter_mut().chain([&mut self.name, &mut self.acronym]) {
            edit.set_language(language);
        }
    }

    fn scope<R>(&mut self, f: impl FnOnce(&mut Self, &dyn crate::edit_box::Metrics) -> R) -> R {
        let font = self.font.take();
        let result = match &font {
            Some(font) => f(self, &FontMetrics(font)),
            None => f(self, &NoMetrics),
        };
        self.font = font;
        result
    }

    pub fn set_macro(&mut self, macro_entry: Option<MacroView>) {
        self.macro_entry = macro_entry.unwrap_or_else(|| MacroView { id: 0, name: String::new(), description: String::new(), acronym: String::new(), icon: 1, commands: Vec::new() });
        self.description.text = self.macro_entry.description.clone();
        self.description.cursor = self.description.text.chars().count();
        self.icon = self.macro_entry.icon.clamp(1, 7);
        self.command_scroll = 0;
        self.scroll.slider.set_pos(0);
        self.scroll.slider.mouse_up();
        self.scroll_pressed = None;
        self.scroll.up_pressed = false;
        self.scroll.down_pressed = false;
        self.focus = EditFocus::Name;
        self.scope(|view, m| {
            for edit in view.lines.iter_mut().chain([&mut view.name, &mut view.acronym]) {
                edit.kill_focus();
            }
            let (name, acronym) = (view.macro_entry.name.clone(), view.macro_entry.acronym.clone());
            fill_box(&mut view.name, &name, m);
            fill_box(&mut view.acronym, &acronym, m);
            for line in 0..MACRO_COMMANDS_MAX {
                let text = view.macro_entry.commands.get(line).map(|command| command.label.clone()).unwrap_or_default();
                fill_box(&mut view.lines[line], &text, m);
            }
            view.layout_lines();
        });
    }

    fn layout_lines(&mut self) {
        for (line, edit) in self.lines.iter_mut().enumerate() {
            edit.rect = line_rect(line as i32 - self.command_scroll as i32);
        }
    }

    fn focused_line(&self) -> Option<usize> {
        self.lines.iter().position(|edit| edit.focused)
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<MacroHit> {
        if x >= 233 && y < 20 { return Some(MacroHit::Close); }
        if self.save.contains(x, y) { return Some(MacroHit::InfoOk); }
        if self.cancel.contains(x, y) { return Some(MacroHit::InfoCancel); }
        if self.more_info.contains(x, y) { return Some(MacroHit::MoreInfo); }
        if self.help.contains(x, y) { return Some(MacroHit::Help); }
        if self.prev.contains(x, y) { return Some(MacroHit::Prev); }
        if self.next.contains(x, y) { return Some(MacroHit::Next); }
        if EDIT_NAME_RECT.contains(x, y) { return Some(MacroHit::Macro(0)); }
        if EDIT_ACRONYM_RECT.contains(x, y) { return Some(MacroHit::Macro(1)); }
        if let Some(line) = self.command_line_at(x, y) { return Some(MacroHit::CommandLine(line)); }
        None
    }

    pub fn settle_scroll(&mut self) {
        let Some(line) = self.focused_line() else { return };
        let first = self.command_scroll;
        let target = if (first as i32) < line as i32 - 8 {
            line - 8
        } else if first <= line {
            return;
        } else {
            line
        };
        self.command_scroll = target;
        self.scroll.slider.set_pos(target as i32);
        self.layout_lines();
    }

    fn blur_all(&mut self) {
        for edit in self.lines.iter_mut().chain([&mut self.name, &mut self.acronym]) {
            if edit.focused {
                edit.kill_focus();
            }
        }
    }

    pub fn focus(&mut self, focus: EditFocus) {
        self.flush_command();
        self.blur_all();
        self.focus = focus;
        match focus {
            EditFocus::Name => self.name.focus(),
            EditFocus::Acronym => self.acronym.focus(),
            EditFocus::Command(line) => {
                if let Some(edit) = self.lines.get_mut(line) {
                    edit.focus();
                }
            }
        }
        self.settle_scroll();
    }

    fn box_at(&self, x: i32, y: i32) -> Option<EditFocus> {
        if EDIT_NAME_RECT.contains(x, y) {
            return Some(EditFocus::Name);
        }
        if EDIT_ACRONYM_RECT.contains(x, y) {
            return Some(EditFocus::Acronym);
        }
        self.command_line_at(x, y)
            .filter(|line| self.lines[*line].rect.contains(x, y))
            .map(EditFocus::Command)
    }

    fn focused_box(&mut self) -> Option<&mut EditBox> {
        match self.focus {
            EditFocus::Name => Some(&mut self.name),
            EditFocus::Acronym => Some(&mut self.acronym),
            EditFocus::Command(line) => self.lines.get_mut(line),
        }
    }

    pub fn mouse_down_box(&mut self, x: i32, y: i32) -> bool {
        let Some(target) = self.box_at(x, y) else {
            let list = EDIT_COMMAND_LIST_RECT;
            let on_scrollbar = self.scroll.hit(list.w, list.h, x - list.x, y - list.y).is_some();
            if list.contains(x, y) && !on_scrollbar {
                self.focus(EditFocus::Command(self.command_scroll));
                return true;
            }
            self.flush_command();
            self.blur_all();
            return false;
        };
        if self.focus != target || self.focused_box().is_some_and(|edit| !edit.focused) {
            self.focus(target);
        }
        self.scope(|view, m| {
            if let Some(edit) = view.focused_box() {
                edit.mouse_down(x, m);
                edit.layout(m);
            }
        });
        true
    }

    pub fn mouse_move_box(&mut self, x: i32) {
        self.scope(|view, m| {
            if let Some(edit) = view.focused_box().filter(|edit| edit.focused) {
                edit.mouse_move(x, m);
                edit.layout(m);
            }
        });
    }

    pub fn mouse_up_box(&mut self) {
        for edit in self.lines.iter_mut().chain([&mut self.name, &mut self.acronym]) {
            edit.mouse_up();
        }
    }

    pub fn shift(&mut self, pressed: bool) -> bool {
        let Some(edit) = self.focused_box().filter(|edit| edit.focused) else { return false };
        if pressed {
            edit.key_down_shift();
        } else {
            edit.key_up(crate::edit_box::KEY_SHIFT);
        }
        true
    }

    pub fn tick(&mut self, elapsed_ms: u32) {
        for edit in self.lines.iter_mut().chain([&mut self.name, &mut self.acronym]) {
            edit.tick(elapsed_ms);
        }
    }

    pub fn flush_command(&mut self) {
        let EditFocus::Command(line) = self.focus else { return };
        let text = self.lines.get(line).and_then(EditBox::get_text).unwrap_or_default();
        if text.is_empty() {
            if let Some(command) = self.macro_entry.commands.get_mut(line) {
                *command = MacroCommandView::default();
            }
            return;
        }
        self.set_command_label(line, MacroCommandView { label: text, ..Default::default() });
    }

    fn set_command_label(&mut self, line: usize, command: MacroCommandView) {
        while self.macro_entry.commands.len() <= line {
            self.macro_entry.commands.push(MacroCommandView::default());
        }
        self.macro_entry.commands[line] = command;
    }

    pub fn key(&mut self, key: InputKey) {
        if self.font.is_none() {
            return;
        }
        self.scope(|view, m| {
            if let Some(edit) = view.focused_box().filter(|edit| edit.focused) {
                edit.input_key(key, m);
            }
        });
        self.flush_command();
    }

    pub fn select_previous_icon(&mut self) {
        self.icon = if self.icon <= 1 { 7 } else { self.icon - 1 };
    }

    pub fn select_next_icon(&mut self) {
        self.icon = if self.icon >= 7 { 1 } else { self.icon + 1 };
    }

    pub fn line_text(&self, line: usize) -> String {
        self.lines.get(line).and_then(EditBox::get_text).unwrap_or_default()
    }

    pub fn result(&self) -> MacroView {
        MacroView { id: self.macro_entry.id, name: self.name.get_text().unwrap_or_default(), description: self.description.text.clone(), acronym: self.acronym.get_text().unwrap_or_default(), icon: self.icon, commands: self.macro_entry.commands.clone() }
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, _alpha: f32) {
        let alpha = 1.0;
        self.frame.paint(list, x, y, EDIT_SIZE.0, EDIT_SIZE.1, font, alpha);
        list.draw_texture(x, y + 20, 256, 381, 0, 0, 256, 381, "l2ui_ch3.macrownd.macro1_back", alpha);
        let icon = format!("{ICON_PREFIX}{}", self.icon);
        list.draw_texture(x + 17, y + 55, 32, 32, 0, 0, 32, 32, &icon, alpha);
        let (ax, ay) = EDIT_ACRONYM_OVER_ICON;
        paint_acronym(list, x + ax, y + ay, &self.acronym.get_text().unwrap_or_default(), font);
        list.draw_text(x + 11, y + 32, Argb(0xffdcdcdc), &self.title, font);
        list.draw_text(x + 65, y + 63, Argb(0xffa3a3a3), &self.name_label, font);
        let metrics = FontMetrics(font);
        self.name.paint(list, (x, y), &metrics, font);
        list.draw_text(x + 65, y + 86, Argb(0xffa3a3a3), &self.acronym_label, font);
        self.acronym.paint(list, (x, y), &metrics, font);
        for row in 0..EDIT_COMMAND_ROWS {
            let command_index = self.command_scroll + row;
            let number = if command_index < 10 { format!("0{command_index}") } else { command_index.to_string() };
            let line_y = y + EDIT_COMMAND_LIST_RECT.y + row as i32 * EDIT_COMMAND_STEP + EDIT_NUMBER_Y;
            list.draw_digits(x + EDIT_COMMAND_LIST_RECT.x + EDIT_NUMBER_X, line_y, &number, Argb(0xffa3a3a3));
            if let Some(edit) = self.lines.get(command_index) {
                edit.paint(list, (x, y), &metrics, font);
            }
        }
        let r = EDIT_COMMAND_LIST_RECT;
        self.scroll.paint(list, x + r.x, y + r.y, r.w, r.h);
        let down = |hit| self.pressed == Some(hit);
        self.save.paint(list, x, y, down(MacroHit::InfoOk), font);
        self.cancel.paint(list, x, y, down(MacroHit::InfoCancel), font);
        self.help.paint(list, x, y, down(MacroHit::Help), font);
        self.more_info.paint(list, x, y, down(MacroHit::MoreInfo), font);
        self.prev.paint(list, x, y, down(MacroHit::Prev), font);
        self.next.paint(list, x, y, down(MacroHit::Next), font);
    }
}

impl MacroInfoView {
    /// `NCMacroInfoWnd::OnPaint @ 0x100aafe0` tira o título de `GL2GameData + 0x989c8`, que pela
    /// mesma conta dos outros é a sysstring `0x39` ("Description"); os dois botões vêm de
    /// `NCMacroInfoWnd::OnCreate @ 0x100abe30` (`0x8c` "OK" e `0x8d` "Cancel").
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let title = lookup(0x39).unwrap_or_else(|| "Description".into());
        let ok = lookup(0x8c).unwrap_or_else(|| "OK".into());
        let cancel = lookup(0x8d).unwrap_or_else(|| "Cancel".into());
        Self {
            title: title.clone(),
            macro_entry: None,
            ok: TextButton::new(INFO_OK_RECT, "L2UI_ch3.button.btn1_Normal", "L2UI_ch3.button.btn1_NormalOn", &ok),
            cancel: TextButton::new(INFO_CANCEL_RECT, "L2UI_ch3.button.btn1_Normal", "L2UI_ch3.button.btn1_NormalOn", &cancel),
            frame: FrameCtrl::titled(&title, true),
            edit: crate::multi_edit::MultiEdit::new(INFO_EDIT_ID, INFO_TEXT_RECT, INFO_LINE_STEP),
            stored: String::new(),
            font: None,
            pressed: None,
        }
    }

    pub fn set_font(&mut self, font: &BitmapFont) {
        self.font = Some(font.clone());
        self.edit.set_line_height(font.line_height as i32);
    }

    fn scope<R>(&mut self, f: impl FnOnce(&mut Self, &dyn crate::edit_box::Metrics) -> R) -> R {
        let font = self.font.take();
        let result = match &font {
            Some(font) => f(self, &FontMetrics(font)),
            None => f(self, &NoMetrics),
        };
        self.font = font;
        result
    }

    pub fn set_description(&mut self, text: &str) {
        self.scope(|view, m| {
            view.edit.clear();
            if !text.is_empty() {
                view.edit.set_text(text, m);
            }
        });
    }

    pub fn open(&mut self) {
        let stored = self.stored.clone();
        self.set_description(&stored);
    }

    pub fn accept(&mut self) -> bool {
        let text = self.edit.get_text();
        if text.as_ref().is_some_and(|text| text.chars().count() > INFO_DESCRIPTION_MAX_CHARS) {
            return false;
        }
        self.stored = text.unwrap_or_default();
        true
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<MacroHit> {
        if self.ok.contains(x, y) {
            return Some(MacroHit::InfoOk);
        }
        if self.cancel.contains(x, y) {
            return Some(MacroHit::InfoCancel);
        }
        None
    }

    pub fn mouse_down_edit(&mut self, x: i32, y: i32) -> bool {
        if !INFO_TEXT_RECT.contains(x, y) {
            self.edit.kill_focus();
            return false;
        }
        self.edit.focus();
        self.edit.mouse_down(x, y);
        true
    }

    pub fn key(&mut self, key: InputKey) -> bool {
        if !self.edit.focused {
            return false;
        }
        self.scope(|view, m| view.edit.input_key(key, m));
        true
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, _alpha: f32) {
        let alpha = 1.0;
        self.frame.paint(list, x, y, INFO_SIZE.0, INFO_SIZE.1, font, alpha);
        list.draw_texture(x, y + 20, 256, 141, 0, 0, 256, 141, INFO_BACK, alpha);
        list.draw_text(x + 11, y + 32, Argb(0xffdcdcdc), &self.title, font);
        self.edit.paint(list, (x, y), &FontMetrics(font), font);
        self.ok.paint(list, x, y, self.pressed == Some(MacroHit::InfoOk), font);
        self.cancel.paint(list, x, y, self.pressed == Some(MacroHit::InfoCancel), font);
    }
}

/// Textura do ícone de um macro, do jeito que a lista e o cursor de arrasto usam.
/// Desenha a sigla por cima do ícone do jeito do original: um caractere por vez, dois por
/// linha, descendo 13 pixels a cada par (`NCMacroArrayWnd::OnPaint @ 0x100a93d0` a partir de
/// `(célula + 4, célula + 4)`, e `NCMacroEditWnd::OnPaint @ 0x100aa400` a partir de `(21, 59)`).
/// A cor é `0xffdcdcdc` nos dois.
pub fn paint_acronym(list: &mut DrawList, x: i32, y: i32, acronym: &str, font: &BitmapFont) {
    let (mut cx, mut cy) = (x, y);
    for (index, letter) in acronym.chars().enumerate() {
        list.draw_text(cx, cy, Argb(0xffdcdcdc), &letter.to_string(), font);
        if index % 2 == 0 {
            cx += ACRONYM_STEP;
        } else {
            cx -= ACRONYM_STEP;
            cy += ACRONYM_STEP;
        }
    }
}

/// Passo entre as letras da sigla, nos dois eixos (`0xd` em `0x100a93d0` e `0x100aa400`).
pub const ACRONYM_STEP: i32 = 13;
/// Canto da sigla desenhada sobre o ícone do editor (`iVar8 = 0x15`, `iVar7 = 0x3b`).
pub const EDIT_ACRONYM_OVER_ICON: (i32, i32) = (21, 59);

pub fn macro_icon(macro_entry: &MacroView) -> String { format!("{ICON_PREFIX}{}", macro_entry.icon.clamp(1, 7)) }

pub fn list_grip() -> Rect { FrameCtrl::new(FrameMode::Title).grip(LIST_SIZE.0, LIST_SIZE.1) }
pub fn info_grip() -> Rect { FrameCtrl::new(FrameMode::Title).grip(INFO_SIZE.0, INFO_SIZE.1) }
pub fn edit_grip() -> Rect { FrameCtrl::new(FrameMode::Title).grip(EDIT_SIZE.0, EDIT_SIZE.1) }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Quad;
    use crate::font::{Glyphs, gly};
    use std::collections::HashMap;

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn entry(id: i32) -> MacroView {
        MacroView { id, name: format!("macro{id}"), description: "description".into(), acronym: format!("M{id}"), icon: 4, commands: vec![MacroCommandView { kind: 3, command: "/target".into(), label: "/target".into(), ..Default::default() }] }
    }

    #[test]
    fn reopening_or_adding_a_macro_cannot_flush_the_previous_command_into_it() {
        let mut view = MacroEditView::new(|_| None);
        view.set_font(&font());
        view.set_macro(Some(entry(1)));
        view.focus(EditFocus::Command(0));
        view.key(InputKey::Char('x'));
        assert_eq!(view.result().commands[0].label, "/targetx");
        let next = entry(2);
        view.set_macro(Some(next.clone()));
        view.focus(EditFocus::Name);
        assert_eq!(view.result(), next);
        view.focus(EditFocus::Command(0));
        view.set_macro(None);
        view.focus(EditFocus::Name);
        assert!(view.result().commands.is_empty());
        assert!(view.lines.iter().all(|edit| edit.get_text().is_none()));
    }

    #[test]
    fn every_visible_command_including_empty_lines_has_the_native_edit_box() {
        let mut view = MacroEditView::new(|_| None);
        let font = font();
        for first in 0..=3 {
            view.scroll_commands(view.command_scroll as i32 - first);
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font, 1.0);
            for (part, width) in [("1", 8.0), ("2", 186.0), ("3", 8.0)] {
                let quads: Vec<_> = list.quads.iter().filter(|q| q.y >= 125.0 && q.texture == format!("L2UI_CH3.Etc.inputbox{part}_disable")).collect();
                assert_eq!(quads.len(), 9);
                for (row, quad) in quads.iter().enumerate() {
                    assert_eq!((quad.y, quad.w, quad.h), (132.0 + row as f32 * 26.0, width, 17.0));
                    assert_eq!(view.command_line_at(30, quad.y as i32), Some(first as usize + row));
                }
            }
            assert_eq!(view.command_line_at(30, 363), None);
            assert_eq!(view.command_line_at(240, 133), None);
            assert!(list.quads.iter().any(|q| q.texture == crate::scroll::UP_BTN));
        }
    }

    #[test]
    fn arrows_and_drag_reach_the_last_command_and_release_cancels_capture() {
        let mut view = MacroEditView::new(|_| None);
        assert!(view.scroll_down(240, 355));
        view.scroll_up(None);
        assert_eq!(view.command_scroll, 0);
        for row in 1..=3 {
            assert!(view.scroll_down(240, 355));
            view.scroll_up(Some((240, 355)));
            assert_eq!(view.command_scroll, row);
        }
        assert_eq!(view.command_line_at(30, 345), Some(11));
        view.scroll_commands(3);
        assert!(view.scroll_down(240, 150));
        assert!(view.scroll_move(1000));
        assert_eq!(view.command_scroll, 3);
        view.scroll_up(None);
        assert!(!view.scroll_move(150));
        view.focus(EditFocus::Command(0));
        assert_eq!(view.command_scroll, 0);
        view.focus(EditFocus::Command(11));
        assert_eq!(view.command_scroll, 3);
    }

    #[test]
    fn list_uses_original_grid_and_count() {
        let mut view = MacroListView::new(|id| Some(format!("s{id}")));
        view.set_macros(vec![entry(7)], 19);
        assert_eq!(LIST_SIZE, (256, 401));
        assert_eq!(ARRAY_RECT, Rect::new(6, 45, 244, 285));
        assert_eq!(view.hit(10, 49), Some(MacroHit::Macro(0)));
        assert_eq!(view.hit(11, 347), Some(MacroHit::Add));
        assert_eq!(view.hit(11, 372), Some(MacroHit::Help));
        // os dois ícones de soltar do original: edição em (169, 347) e lixeira em (208, 347)
        assert_eq!(view.hit(170, 348), Some(MacroHit::EditIcon));
        assert_eq!(view.hit(209, 348), Some(MacroHit::Trash));
        assert_eq!(view.hit(205, 348), None, "sobra entre os dois ícones");
        assert_eq!((view.count, view.total), (1, 19));
    }

    /// Enquanto um macro está sendo arrastado, o ícone sob o cursor troca pra arte `_Drag`
    /// (`+0x11c`/`+0x120` no `NCMacroListWnd::OnPaint @ 0x100a8dd0`).
    #[test]
    fn the_drop_icons_switch_to_the_drag_art_while_a_macro_is_dragged() {
        let font = font();
        let mut view = MacroListView::new(|_| None);
        view.set_macros(vec![entry(7)], 1);
        let painted = |view: &MacroListView| {
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font, 1.0);
            list.quads
        };
        let quads = painted(&view);
        let icon = |quads: &[Quad], texture: &str| quads.iter().find(|q| q.texture == texture).cloned();
        let edit = icon(&quads, EDIT).expect("ícone de edição desenhado");
        assert_eq!((edit.x, edit.y, edit.w, edit.h), (169.0, 347.0, 34.0, 34.0));
        let trash = icon(&quads, TRASH).expect("lixeira desenhada");
        assert_eq!((trash.x, trash.y, trash.w, trash.h), (208.0, 347.0, 34.0, 34.0));
        assert!(icon(&quads, EDIT_DRAG).is_none() && icon(&quads, TRASH_DRAG).is_none());
        view.dragging = Some(0);
        view.hover = Some(MacroHit::EditIcon);
        let quads = painted(&view);
        assert!(icon(&quads, EDIT_DRAG).is_some() && icon(&quads, EDIT).is_none());
        assert!(icon(&quads, TRASH).is_some(), "a lixeira segue normal");
        view.hover = Some(MacroHit::Trash);
        let quads = painted(&view);
        assert!(icon(&quads, TRASH_DRAG).is_some() && icon(&quads, EDIT).is_some());
        view.dragging = None;
        assert!(icon(&painted(&view), TRASH_DRAG).is_none(), "sem arrasto, arte normal");
    }

    /// A janela de descrição do original tem uma caixa de texto só, em `(9, 48, 238, 77)`
    /// (`NCMacroInfoWnd::OnCreate @ 0x100abe30`, id `0x1514`), mais os botões de OK e Cancel.
    /// Nome em dourado e lista de comandos ali dentro eram invenção nossa.
    #[test]
    fn the_description_window_shows_only_the_description_in_the_original_box() {
        let font = font();
        let mut view = MacroInfoView::new(|_| None);
        view.set_font(&font);
        view.set_description(&"x ".repeat(60));
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font, 1.0);
        // só o texto da caixa: abaixo dela ficam os rótulos dos dois botões, também em fonte
        let text: Vec<&Quad> =
            list.quads.iter().filter(|q| q.texture == font.atlas && q.y >= 48.0 && q.y < INFO_OK_RECT.y as f32).collect();
        assert!(!text.is_empty());
        assert!(text.iter().all(|q| q.x >= 9.0 && q.x < (9 + 238) as f32), "o texto fica na caixa");
        assert!(text.iter().all(|q| q.y < (48 + 77) as f32), "e não passa da altura dela");
        let names: Vec<&Quad> = list.quads.iter().filter(|q| q.color == crate::color::GOLD.rgba()).collect();
        assert!(names.is_empty(), "o nome do macro não vai na janela de descrição");
        assert_eq!(INFO_OK_RECT, Rect::new(51, 132, 76, 23));
        assert_eq!(INFO_CANCEL_RECT, Rect::new(131, 132, 76, 23));
    }

    /// `NCMacroInfoWnd::OnOK` (`NCMacroInfoWnd.cpp`): a descrição vem de uma caixa de várias linhas
    /// (`NCMultiEdit`, id `0x1514`); acima de 32 caracteres é recusada (mensagem `0x345`), senão
    /// fica guardada e o `ShowWindow` a põe de volta na caixa.
    #[test]
    fn the_description_is_typed_in_the_box_kept_by_ok_and_refused_above_32_characters() {
        let font = font();
        let mut view = MacroInfoView::new(|_| None);
        view.set_font(&font);
        view.set_description("antiga");
        assert!(view.mouse_down_edit(20, 60));
        for c in "nova".chars() {
            assert!(view.key(InputKey::Char(c)));
        }
        assert_eq!(view.edit.get_text().as_deref(), Some("antiganova"));
        assert!(view.accept());
        assert_eq!(view.stored, "antiganova");
        view.set_description("");
        assert_eq!(view.edit.get_text(), None);
        view.open();
        assert_eq!(view.edit.get_text().as_deref(), Some("antiganova"));
        view.set_description(&"y".repeat(INFO_DESCRIPTION_MAX_CHARS + 1));
        assert!(!view.accept());
        assert_eq!(view.stored, "antiganova");
        view.set_description("");
        assert!(view.accept());
        assert_eq!(view.stored, "");
        assert!(!view.mouse_down_edit(2, 2));
        assert!(!view.key(InputKey::Char('z')));
    }

    /// A lista de comandos do editor mostra nove das doze linhas e rola
    /// (`FUN_100acb00(9, 0xc)` em `NCMacroEditWnd::OnCreate @ 0x100acc50`).
    #[test]
    fn the_command_list_shows_nine_of_twelve_lines_and_scrolls() {
        let font = font();
        let mut view = MacroEditView::new(|_| None);
        assert_eq!((EDIT_COMMAND_ROWS, MACRO_COMMANDS_MAX), (9, 12));
        for line in 0..MACRO_COMMANDS_MAX {
            let command = MacroCommandView { label: format!("linha{line}"), ..Default::default() };
            assert!(view.set_command(line, command), "linha {line} aceita comando");
        }
        assert!(!view.set_command(MACRO_COMMANDS_MAX, MacroCommandView::default()), "a décima terceira não");
        let shown = |view: &MacroEditView| {
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font, 1.0);
            list.quads.iter().filter(|q| q.texture == font.atlas).count()
        };
        let first = shown(&view);
        view.scroll_commands(-1);
        assert_eq!(view.command_scroll, 1);
        view.scroll_commands(-5);
        assert_eq!(view.command_scroll, 3, "para na última página");
        assert_ne!(shown(&view), first, "a lista rolou de verdade");
        view.scroll_commands(9);
        assert_eq!(view.command_scroll, 0, "e volta ao topo");
        // o texto acima de 32 caracteres é recusado, como em 0x100ae20b
        let long = MacroCommandView { label: "x".repeat(MACRO_COMMAND_MAX_CHARS + 1), ..Default::default() };
        assert!(!view.set_command(0, long));
    }

    /// A sigla é desenhada por cima do ícone, um caractere por vez, dois por linha, descendo 13
    /// pixels a cada par — na célula da lista a partir de `(x + 4, y + 4)`
    /// (`NCMacroArrayWnd::OnPaint @ 0x100a93d0`) e no editor a partir de `(21, 59)`
    /// (`NCMacroEditWnd::OnPaint @ 0x100aa400`). Cor `0xffdcdcdc` nos dois.
    #[test]
    fn the_acronym_is_drawn_over_the_icon_two_letters_per_row() {
        let font = font();
        let mut list = DrawList::default();
        paint_acronym(&mut list, 10, 20, "ABCD", &font);
        let places: Vec<(f32, f32)> = list.quads.iter().map(|q| (q.x, q.y)).collect();
        assert_eq!(places, [(10.0, 20.0), (23.0, 20.0), (10.0, 33.0), (23.0, 33.0)]);
        assert!(list.quads.iter().all(|q| q.color == Argb(0xffdcdcdc).rgba()));
        assert_eq!(ACRONYM_STEP, 13);

        // na célula da lista: canto da célula mais quatro
        let mut view = MacroListView::new(|_| None);
        view.set_macros(vec![MacroView { acronym: "MM".into(), ..entry(7) }], 1);
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font, 1.0);
        let letters: Vec<(f32, f32)> = list
            .quads
            .iter()
            .filter(|q| q.texture == font.atlas && q.color == Argb(0xffdcdcdc).rgba() && q.y >= ARRAY_RECT.y as f32)
            .map(|q| (q.x, q.y))
            .collect();
        let (cx, cy) = ((ARRAY_RECT.x + 4 + 4) as f32, (ARRAY_RECT.y + 4 + 4) as f32);
        assert_eq!(letters, [(cx, cy), (cx + 13.0, cy)]);

        // no editor: sobre o ícone, em (21, 59)
        let mut edit = MacroEditView::new(|_| None);
        edit.acronym.set_text("AB");
        let mut list = DrawList::default();
        edit.paint(&mut list, 0, 0, &font, 1.0);
        let over: Vec<(f32, f32)> = list
            .quads
            .iter()
            .filter(|q| q.texture == font.atlas && q.color == Argb(0xffdcdcdc).rgba() && q.y < 70.0)
            .map(|q| (q.x, q.y))
            .collect();
        assert!(over.contains(&(21.0, 59.0)) && over.contains(&(34.0, 59.0)), "saiu {over:?}");
    }

    #[test]
    fn list_keeps_the_selected_macro_when_the_server_refreshes_the_chunks() {
        let mut view = MacroListView::new(|_| None);
        view.set_macros(vec![entry(7), entry(8)], 2);
        view.selected = Some(1);
        view.set_macros(vec![entry(8), entry(7)], 2);
        assert_eq!(view.selected, Some(0));
    }

    /// Trava os ids de sysstring de cada texto das três janelas de macro contra o que o original
    /// usa (ver os comentários de cada `new`): `NCMacroEditWnd::OnCreate` chama
    /// `FUN_100014a0(<id>)` por botão, e os títulos/rótulos saem de `GL2GameData + offset`, que
    /// vira id por `(offset - 0x9871c) / 12`. Já foram trocados uma vez — daí o teste.
    #[test]
    fn the_three_macro_windows_use_the_sysstring_ids_of_the_original() {
        let texts: HashMap<u32, String> = [
            (0x32, "Name"), (0x39, "Description"), (0x8c, "OK"), (0x8d, "Cancel"),
            (0x90, "All"), (0x91, "Help"), (0x28d, "Acronym"), (0x28e, "Add"),
            (0x28f, "Macro List"), (0x290, "Macro Edit"), (0x291, "More Info."),
            (0x292, "Save"), (0x37c, "Macro Deletion"),
        ]
        .into_iter()
        .map(|(id, text)| (id, text.to_string()))
        .collect();
        let lookup = |id: u32| texts.get(&id).cloned();

        let list = MacroListView::new(lookup);
        assert_eq!(list.title, "Macro List");
        assert_eq!(list.add.label, "Add");
        assert_eq!(list.help.label, "Help");

        let info = MacroInfoView::new(lookup);
        assert_eq!(info.title, "Description");
        assert_eq!((info.ok.label.as_str(), info.cancel.label.as_str()), ("OK", "Cancel"));

        let edit = MacroEditView::new(lookup);
        assert_eq!(edit.title, "Macro Edit");
        assert_eq!((edit.name_label.as_str(), edit.acronym_label.as_str()), ("Name", "Acronym"));
        // ordem na tela, da esquerda pra direita: Help (x=11), Save (x=91), Cancel (x=171)
        assert_eq!(edit.help.label, "Help");
        assert_eq!(edit.save.label, "Save");
        assert_eq!(edit.cancel.label, "Cancel");
        assert_eq!(edit.more_info.label, "More Info.");
        assert!(edit.help.contains(11, 372) && edit.save.contains(91, 372) && edit.cancel.contains(171, 372));
    }

    #[test]
    fn info_uses_original_size_and_button_positions() {
        let view = MacroInfoView::new(|id| Some(format!("s{id}")));
        assert_eq!(INFO_SIZE, (256, 161));
        assert!(view.ok.contains(51, 132));
        assert!(view.cancel.contains(131, 132));
    }

    #[test]
    fn editor_keeps_macro_commands_when_text_fields_are_changed() {
        let mut view = MacroEditView::new(|id| Some(format!("s{id}")));
        view.set_macro(Some(entry(7)));
        view.set_font(&font());
        view.focus(EditFocus::Name);
        view.key(InputKey::Char('X'));
        assert_eq!(view.result().commands[0].command, "/target");
        assert_eq!(view.result().name, "macro7X");
        assert_eq!(view.hit(93, 60), Some(MacroHit::Macro(0)));
    }

    #[test]
    fn editor_uses_original_command_list_geometry_and_numbering() {
        let view = MacroEditView::new(|id| Some(format!("s{id}")));
        assert_eq!(EDIT_COMMAND_LIST_RECT, Rect::new(7, 125, 242, 239));
        assert_eq!(EDIT_COMMAND_ROWS, 9);
        assert_eq!(EDIT_COMMAND_STEP, 26);
        assert_eq!(view.command_scroll, 0);
    }

    #[test]
    fn editor_cycles_the_original_seven_macro_icons() {
        let mut view = MacroEditView::new(|_| None);
        view.set_macro(Some(entry(7)));
        view.icon = 1;
        view.select_previous_icon();
        assert_eq!(view.icon, 7);
        view.select_next_icon();
        assert_eq!(view.icon, 1);
        view.icon = 7;
        view.select_next_icon();
        assert_eq!(view.icon, 1);
    }

    #[test]
    fn editor_applies_the_window_alpha_to_the_original_background_and_icon() {
        let mut view = MacroEditView::new(|_| None);
        view.set_macro(Some(entry(7)));
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font(), 0.4);
        assert!(list.quads.iter().filter(|quad| quad.texture == "l2ui_ch3.macrownd.macro1_back").all(|quad| quad.color[3] == 1.0));
        assert!(list.quads.iter().filter(|quad| quad.texture == "L2UI.MacroWnd.Macro_Icon7").all(|quad| quad.color[3] == 1.0));
    }

    #[test]
    fn the_boxes_are_the_native_ctor_10011420_with_the_ids_and_limits_of_100acc50_and_100ab6a0() {
        let view = MacroEditView::new(|_| None);
        assert_eq!((view.name.id, view.name.max_len, view.name.rect), (0x1512, 12, EDIT_NAME_RECT));
        assert_eq!((view.acronym.id, view.acronym.max_len, view.acronym.rect), (0x1513, 4, EDIT_ACRONYM_RECT));
        assert_eq!(view.lines.len(), 12);
        for (line, edit) in view.lines.iter().enumerate() {
            assert_eq!((edit.id, edit.max_len), (0x1506 + line as u32, 32));
            assert_eq!(edit.rect, Rect::new(27, 132 + 26 * line as i32, 202, 17));
            assert_eq!((edit.kind, edit.extra_width, edit.accepts, edit.caret_enabled, edit.caret_blink), (0, 264, true, true, true));
        }
    }

    #[test]
    fn a_focused_line_keeps_the_list_on_screen_even_after_the_wheel_moves_it_away_100ab830() {
        let mut view = MacroEditView::new(|_| None);
        view.set_font(&font());
        view.focus(EditFocus::Command(5));
        assert_eq!(view.command_scroll, 0);
        view.scroll_commands(-3);
        assert_eq!(view.command_scroll, 3, "first <= idx: a linha continua visível, nada a corrigir");
        view.focus(EditFocus::Command(0));
        view.scroll_commands(-3);
        assert_eq!(view.command_scroll, 0, "first > idx volta para idx");
        view.focus(EditFocus::Command(11));
        assert_eq!(view.command_scroll, 3, "first < idx - 8");
        view.scroll_commands(3);
        assert_eq!(view.command_scroll, 3, "a roda não afasta a linha com foco");
        view.focus(EditFocus::Name);
        view.scroll_commands(3);
        assert_eq!(view.command_scroll, 0);
    }

    #[test]
    fn dragging_inside_a_line_selects_text_and_typing_replaces_it() {
        let mut view = MacroEditView::new(|_| None);
        view.set_font(&font());
        view.set_macro(Some(entry(1)));
        let line = view.lines[0].rect;
        assert!(view.mouse_down_box(line.x + 2 + 7 * 7, line.y + 2));
        assert_eq!(view.focus, EditFocus::Command(0));
        assert!(view.lines[0].focused);
        view.mouse_move_box(line.x + 2);
        assert_eq!(view.lines[0].selection().0, -7);
        view.mouse_up_box();
        view.key(InputKey::Char('x'));
        assert_eq!(view.result().commands[0].label, "x");
        assert!(!view.name.focused);
    }

    #[test]
    fn clicking_outside_every_box_drops_the_focus_and_keeps_the_text() {
        let mut view = MacroEditView::new(|_| None);
        view.set_font(&font());
        view.set_macro(Some(entry(1)));
        view.focus(EditFocus::Command(0));
        view.key(InputKey::Char('y'));
        assert!(!view.mouse_down_box(5, 5));
        assert!(view.lines.iter().all(|edit| !edit.focused));
        assert_eq!(view.result().commands[0].label, "/targety");
    }

    #[test]
    fn the_caret_blinks_every_five_hundred_milliseconds_and_the_limit_of_32_stops_the_line() {
        let mut view = MacroEditView::new(|_| None);
        view.set_font(&font());
        view.focus(EditFocus::Command(0));
        let before = view.lines[0].caret_blink;
        view.tick(499);
        assert_eq!(view.lines[0].caret_blink, before);
        view.tick(1);
        assert_ne!(view.lines[0].caret_blink, before);
        for _ in 0..32 {
            view.key(InputKey::Char('W'));
        }
        assert_eq!(view.result().commands[0].label.chars().count(), 32);
        view.key(InputKey::Char('W'));
        assert_eq!(view.result().commands[0].label.chars().count(), 32);
    }

    #[test]
    fn dropping_a_command_on_a_line_writes_it_into_the_box_like_100ae210() {
        let mut view = MacroEditView::new(|_| None);
        view.set_font(&font());
        assert!(view.set_command(3, MacroCommandView { label: "/useskill x".into(), ..Default::default() }));
        assert_eq!(view.lines[3].get_text().as_deref(), Some("/useskill x"));
        assert!(view.set_command(3, MacroCommandView { label: "/b".into(), ..Default::default() }));
        assert_eq!(view.lines[3].get_text().as_deref(), Some("/b"));
    }

    #[test]
    fn a_click_in_the_list_outside_every_box_focuses_the_first_visible_line_10034730() {
        let mut view = MacroEditView::new(|_| None);
        view.set_font(&font());
        assert!(view.mouse_down_box(EDIT_COMMAND_LIST_RECT.x + 2, EDIT_COMMAND_LIST_RECT.y + 2));
        assert_eq!(view.focus, EditFocus::Command(0));
        assert!(view.lines[0].focused);
        view.scroll_commands(0);
        view.focus(EditFocus::Name);
        view.scroll_commands(-2);
        assert!(view.mouse_down_box(EDIT_COMMAND_LIST_RECT.x + 2, EDIT_COMMAND_LIST_RECT.y + 2));
        assert_eq!(view.focus, EditFocus::Command(2));
        let bar = (EDIT_COMMAND_LIST_RECT.x + EDIT_COMMAND_LIST_RECT.w - 5, EDIT_COMMAND_LIST_RECT.y + 100);
        assert!(!view.mouse_down_box(bar.0, bar.1));
        assert!(view.lines.iter().all(|edit| !edit.focused));
    }
}
