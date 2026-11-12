use killer_assets::client::DatTables;
use killer_ui::henna::{
    HennaBlock, HennaInfoHit, HennaInfoView, HennaListHit, HennaListView, HennaMode, HennaRow, HennaStrings,
};
use killer_ui::scroll::ScrollHit;
use killer_ui::window::{Desktop, WindowId};
use killer_world::{NetCommand, WorldEvent};
use l2_net::game::{HennaEquipList, HennaItemInfo};

use crate::ui_keys::UiCommand;

pub struct HennaWindows {
    pub list: HennaListView,
    pub info: HennaInfoView,
    pub commands: Vec<UiCommand>,
    pub list_pressed: Option<HennaListHit>,
}

impl HennaWindows {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> HennaWindows {
        let strings = HennaStrings::new(lookup);
        HennaWindows {
            list: HennaListView::new(strings.clone()),
            info: HennaInfoView::new(strings),
            commands: Vec::new(),
            list_pressed: None,
        }
    }

    /// Os quatro pacotes que mexem nas duas janelas. O cabeçalho da lista esvazia a lista, guarda a
    /// adena, esconde a janela de informação e mostra a lista (`0x10070d70` e `0x10071120`); cada
    /// linha vira uma entrada (`0x10070e80` e `0x10071230`); a informação esconde a lista e mostra a
    /// janela de informação (`0x10070fa0` e `0x10071390`).
    pub fn world_event(&mut self, event: &WorldEvent, tables: Option<&DatTables>, desktop: &mut Desktop) {
        match event {
            WorldEvent::HennaEquipList(list) => self.open_list(HennaMode::Draw, list, tables, desktop),
            WorldEvent::HennaUnequipList(list) => self.open_list(HennaMode::Remove, list, tables, desktop),
            WorldEvent::HennaItemInfo(info) => self.open_info(HennaMode::Draw, info, tables, desktop),
            WorldEvent::HennaUnequipInfo(info) => self.open_info(HennaMode::Remove, info, tables, desktop),
            _ => {}
        }
    }

    fn open_list(&mut self, mode: HennaMode, list: &HennaEquipList, tables: Option<&DatTables>, desktop: &mut Desktop) {
        self.list.open(mode, list.adena);
        for offer in &list.offers {
            let Some(row) = row(mode, offer.symbol_id, offer.dye_item_id, tables) else { continue };
            self.list.push(HennaRow {
                symbol_id: offer.symbol_id,
                dye_item_id: offer.dye_item_id,
                dye_amount: offer.dye_amount,
                price: offer.price,
                available: i32::from(offer.available),
                ..row
            });
        }
        visible(desktop, WindowId::HENNA_INFO, false);
        visible(desktop, WindowId::HENNA_LIST, true);
    }

    fn open_info(&mut self, mode: HennaMode, info: &HennaItemInfo, tables: Option<&DatTables>, desktop: &mut Desktop) {
        let Some(dye) = item_block(info.dye_item_id, tables) else { return };
        let Some(symbol) = symbol_block(info.symbol_id, tables) else { return };
        let stats = [info.int_, info.str_, info.con, info.men, info.dex, info.wit];
        self.info.open(mode, info.symbol_id, info.dye_amount, info.price, info.adena, dye, symbol, stats);
        visible(desktop, WindowId::HENNA_LIST, false);
        visible(desktop, WindowId::HENNA_INFO, true);
    }

    /// O `NCHennaListBox::OnLButtonDown @ 0x100917e0` escolhe a linha e avisa o pai na hora, que
    /// pede a informação do símbolo (`RequestHennaItemInfo` ou `RequestHennaUnequipInfo`).
    pub fn list_mouse_down(&mut self, x: i32, y: i32, desktop: &mut Desktop) {
        self.list_pressed = self.list.hit(x, y);
        match self.list_pressed {
            Some(HennaListHit::Row(index)) => {
                self.list.selected = Some(index);
                let Some(row) = self.list.rows.get(index) else { return };
                let symbol_id = row.symbol_id;
                self.commands.push(UiCommand::Net(match self.list.mode {
                    HennaMode::Draw => NetCommand::HennaItemInfo { symbol_id },
                    HennaMode::Remove => NetCommand::HennaUnequipInfo { symbol_id },
                }));
            }
            Some(HennaListHit::Close) => visible(desktop, WindowId::HENNA_LIST, false),
            _ => {}
        }
    }

    pub fn list_mouse_up(&mut self, over: Option<(WindowId, i32, i32)>) {
        let Some(pressed) = self.list_pressed.take() else { return };
        let released = over.filter(|(id, _, _)| *id == WindowId::HENNA_LIST).and_then(|(_, x, y)| self.list.hit(x, y));
        if released != Some(pressed) {
            return;
        }
        if let HennaListHit::Scroll(hit) = pressed {
            self.list.scroll(hit);
        }
    }

    pub fn info_mouse_down(&mut self, x: i32, y: i32, desktop: &mut Desktop) {
        self.info.pressed = self.info.hit(x, y);
        if self.info.pressed == Some(HennaInfoHit::Close) {
            self.info.pressed = None;
            visible(desktop, WindowId::HENNA_INFO, false);
        }
    }

    /// `OK` (`0xf02`, `0x10091b80`) manda o pedido e esconde a janela; `Back` (`0xf03`,
    /// `0x10091be0`) esconde primeiro e pede a lista de novo ao servidor.
    pub fn info_mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        let Some(pressed) = self.info.pressed.take() else { return };
        let released = over.filter(|(id, _, _)| *id == WindowId::HENNA_INFO).and_then(|(_, x, y)| self.info.hit(x, y));
        if released != Some(pressed) {
            return;
        }
        let symbol_id = self.info.symbol_id;
        match pressed {
            HennaInfoHit::Ok => {
                self.commands.push(UiCommand::Net(match self.info.mode {
                    HennaMode::Draw => NetCommand::HennaEquip { symbol_id },
                    HennaMode::Remove => NetCommand::HennaUnequip { symbol_id },
                }));
                visible(desktop, WindowId::HENNA_INFO, false);
            }
            HennaInfoHit::Back => {
                visible(desktop, WindowId::HENNA_INFO, false);
                self.commands.push(UiCommand::Net(match self.info.mode {
                    HennaMode::Draw => NetCommand::HennaList,
                    HennaMode::Remove => NetCommand::HennaUnequipList,
                }));
            }
            HennaInfoHit::Close => visible(desktop, WindowId::HENNA_INFO, false),
        }
    }

    pub fn wheel(&mut self, delta: i32) {
        self.list.scroll(match delta > 0 {
            true => ScrollHit::Up,
            false => ScrollHit::Down,
        });
    }
}

/// No modo de desenhar as três cadeias da linha vêm do item do corante (`GetItemData`), no de
/// apagar vêm do registro do `hennagrp` (`GetHennaData`).
fn row(mode: HennaMode, symbol_id: i32, dye_item_id: i32, tables: Option<&DatTables>) -> Option<HennaRow> {
    let block = match mode {
        HennaMode::Draw => item_block(dye_item_id, tables)?,
        HennaMode::Remove => symbol_block(symbol_id, tables)?,
    };
    Some(HennaRow { name: block.name, icon: block.icon, add_name: block.add_name, ..HennaRow::default() })
}

fn item_block(item_id: i32, tables: Option<&DatTables>) -> Option<HennaBlock> {
    let tables = tables?;
    let id = u32::try_from(item_id).ok()?;
    let name = tables.itemname.get(&id)?;
    let icon = tables.item_icons.texture(id).unwrap_or_default();
    Some(HennaBlock { name: name.name.clone(), icon, add_name: name.add_name.clone() })
}

fn symbol_block(symbol_id: i32, tables: Option<&DatTables>) -> Option<HennaBlock> {
    let tables = tables?;
    let henna = tables.hennagrp.get(&u32::try_from(symbol_id).ok()?)?;
    Some(HennaBlock { name: henna.name.clone(), icon: henna.icon.clone(), add_name: henna.add_name.clone() })
}

fn visible(desktop: &mut Desktop, id: WindowId, value: bool) {
    if let Some(window) = desktop.window_mut(id) {
        window.visible = value;
    }
    if value && let Some(index) = desktop.windows.iter().position(|w| w.id == id) {
        let window = desktop.windows.remove(index);
        desktop.windows.push(window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::window::{Rect, Window};
    use l2_dat::{HennaGrp, ItemName};
    use l2_net::game::HennaOffer;

    fn tables() -> DatTables {
        let mut tables = DatTables::default();
        tables.itemname.insert(
            4589,
            ItemName {
                id: 4589,
                name: "Greater Dye of STR".into(),
                add_name: "Str+2 Con-2".into(),
                description: String::new(),
                popup: 0,
            },
        );
        tables.hennagrp.insert(
            145,
            HennaGrp {
                symbol_id: 145,
                dye_item_id: 4589,
                name: "Greater Symbol of Strength".into(),
                icon: "icon.etc_str_symbol_i01".into(),
                description: "Str+2 Con-2".into(),
                add_name: "Str+2 Con-2".into(),
            },
        );
        tables
    }

    fn desktop() -> Desktop {
        let mut desktop = Desktop::new((800, 600));
        let grip = killer_ui::henna::grip();
        desktop.windows.push(Window::new(WindowId::HENNA_LIST, Rect::new(0, 0, 256, 401), grip));
        desktop.windows.push(Window::new(WindowId::HENNA_INFO, Rect::new(0, 0, 256, 401), grip));
        for window in &mut desktop.windows {
            window.visible = false;
        }
        desktop
    }

    fn offer() -> HennaOffer {
        HennaOffer { symbol_id: 145, dye_item_id: 4589, dye_amount: 10, price: 33000, available: true }
    }

    fn info(symbol_id: i32) -> HennaItemInfo {
        HennaItemInfo {
            symbol_id,
            dye_item_id: 4589,
            dye_amount: 10,
            price: 33000,
            available: true,
            adena: 999,
            int_: (11, 12),
            str_: (30, 32),
            con: (43, 41),
            men: (10, 10),
            dex: (20, 20),
            wit: (25, 25),
        }
    }

    fn windows() -> HennaWindows {
        HennaWindows::new(|_| None)
    }

    /// `HennaEquipList` abre a lista com os nomes do **item** do corante, esconde a janela de
    /// informação e guarda a adena do cabeçalho.
    #[test]
    fn the_equip_list_fills_the_rows_from_the_dye_item() {
        let (tables, mut desktop, mut henna) = (tables(), desktop(), windows());
        let list = HennaEquipList { adena: 5000, slots: 3, offers: vec![offer()] };
        henna.world_event(&WorldEvent::HennaEquipList(list), Some(&tables), &mut desktop);
        assert!(desktop.window(WindowId::HENNA_LIST).is_some_and(|w| w.visible));
        assert!(desktop.window(WindowId::HENNA_INFO).is_some_and(|w| !w.visible));
        assert_eq!((henna.list.mode, henna.list.adena), (HennaMode::Draw, 5000));
        let row = &henna.list.rows[0];
        assert_eq!((row.name.as_str(), row.add_name.as_str()), ("Greater Dye of STR", "Str+2 Con-2"));
        assert_eq!((row.symbol_id, row.dye_item_id, row.price, row.available), (145, 4589, 33000, 1));
    }

    /// `HennaUnequipList` usa a mesma janela em modo de apagar, com os nomes do `hennagrp`.
    #[test]
    fn the_unequip_list_fills_the_rows_from_the_henna_table() {
        let (tables, mut desktop, mut henna) = (tables(), desktop(), windows());
        let list = HennaEquipList { adena: 7000, slots: 3, offers: vec![offer()] };
        henna.world_event(&WorldEvent::HennaUnequipList(list), Some(&tables), &mut desktop);
        assert_eq!(henna.list.mode, HennaMode::Remove);
        let row = &henna.list.rows[0];
        assert_eq!(row.name, "Greater Symbol of Strength");
        assert_eq!(row.icon, "icon.etc_str_symbol_i01");
    }

    /// Uma linha sem item nem símbolo na tabela é descartada, como o `GetItemData`/`GetHennaData`
    /// nulo faz o original descartar.
    #[test]
    fn a_row_without_a_record_is_dropped() {
        let (tables, mut desktop, mut henna) = (tables(), desktop(), windows());
        let other = HennaOffer { symbol_id: 9999, dye_item_id: 9999, ..offer() };
        let list = HennaEquipList { adena: 0, slots: 0, offers: vec![offer(), other] };
        henna.world_event(&WorldEvent::HennaEquipList(list), Some(&tables), &mut desktop);
        assert_eq!(henna.list.rows.len(), 1);
    }

    /// Apertar numa linha pede a informação no canal do modo, e a janela de informação troca de
    /// lugar com a lista quando a resposta chega.
    #[test]
    fn pressing_a_row_asks_for_the_symbol_information() {
        let (tables, mut desktop, mut henna) = (tables(), desktop(), windows());
        let list = HennaEquipList { adena: 5000, slots: 3, offers: vec![offer()] };
        henna.world_event(&WorldEvent::HennaEquipList(list.clone()), Some(&tables), &mut desktop);
        let rect = killer_ui::henna::LIST_RECT;
        henna.list_mouse_down(rect.x + 1, rect.y + killer_ui::henna::ROW_TOP + 1, &mut desktop);
        assert_eq!(henna.list.selected, Some(0));
        assert_eq!(henna.commands.len(), 1);
        assert!(matches!(
            henna.commands[0],
            UiCommand::Net(NetCommand::HennaItemInfo { symbol_id: 145 })
        ));

        henna.world_event(&WorldEvent::HennaItemInfo(info(145)), Some(&tables), &mut desktop);
        assert!(desktop.window(WindowId::HENNA_LIST).is_some_and(|w| !w.visible));
        assert!(desktop.window(WindowId::HENNA_INFO).is_some_and(|w| w.visible));
        assert_eq!((henna.info.mode, henna.info.symbol_id, henna.info.adena), (HennaMode::Draw, 145, 999));
        assert_eq!(henna.info.stats, [(11, 12), (30, 32), (43, 41), (10, 10), (20, 20), (25, 25)]);
        assert_eq!(henna.info.dye.name, "Greater Dye of STR");
        assert_eq!(henna.info.symbol.name, "Greater Symbol of Strength");

        henna.world_event(&WorldEvent::HennaUnequipList(list), Some(&tables), &mut desktop);
        henna.list_mouse_down(rect.x + 1, rect.y + killer_ui::henna::ROW_TOP + 1, &mut desktop);
        assert!(matches!(
            henna.commands[1],
            UiCommand::Net(NetCommand::HennaUnequipInfo { symbol_id: 145 })
        ));
    }

    /// `OK` manda `RequestHennaEquip` no modo de desenhar e `RequestHennaUnequip` no de apagar, e
    /// fecha; `Back` fecha e pede a lista do modo de novo.
    #[test]
    fn the_buttons_send_the_request_of_the_current_mode() {
        let (tables, mut desktop, mut henna) = (tables(), desktop(), windows());
        henna.world_event(&WorldEvent::HennaItemInfo(info(145)), Some(&tables), &mut desktop);
        let ok = killer_ui::henna::OK_BTN_RECT;
        let back = killer_ui::henna::BACK_BTN_RECT;
        henna.info_mouse_down(ok.x + 2, ok.y + 2, &mut desktop);
        henna.info_mouse_up(Some((WindowId::HENNA_INFO, ok.x + 2, ok.y + 2)), &mut desktop);
        assert!(matches!(henna.commands[0], UiCommand::Net(NetCommand::HennaEquip { symbol_id: 145 })));
        assert!(desktop.window(WindowId::HENNA_INFO).is_some_and(|w| !w.visible));

        henna.world_event(&WorldEvent::HennaUnequipInfo(info(145)), Some(&tables), &mut desktop);
        henna.info_mouse_down(ok.x + 2, ok.y + 2, &mut desktop);
        henna.info_mouse_up(Some((WindowId::HENNA_INFO, ok.x + 2, ok.y + 2)), &mut desktop);
        assert!(matches!(henna.commands[1], UiCommand::Net(NetCommand::HennaUnequip { symbol_id: 145 })));

        henna.world_event(&WorldEvent::HennaUnequipInfo(info(145)), Some(&tables), &mut desktop);
        henna.info_mouse_down(back.x + 2, back.y + 2, &mut desktop);
        henna.info_mouse_up(Some((WindowId::HENNA_INFO, back.x + 2, back.y + 2)), &mut desktop);
        assert!(matches!(henna.commands[2], UiCommand::Net(NetCommand::HennaUnequipList)));
        assert!(desktop.window(WindowId::HENNA_INFO).is_some_and(|w| !w.visible));

        henna.world_event(&WorldEvent::HennaItemInfo(info(145)), Some(&tables), &mut desktop);
        henna.info_mouse_down(back.x + 2, back.y + 2, &mut desktop);
        henna.info_mouse_up(Some((WindowId::HENNA_INFO, back.x + 2, back.y + 2)), &mut desktop);
        assert!(matches!(henna.commands[3], UiCommand::Net(NetCommand::HennaList)));
    }

    /// Soltar fora do controle apertado não faz nada, e o botão de fechar da barra de título
    /// esconde cada janela.
    #[test]
    fn releasing_elsewhere_does_nothing_and_the_title_bar_closes() {
        let (tables, mut desktop, mut henna) = (tables(), desktop(), windows());
        henna.world_event(&WorldEvent::HennaItemInfo(info(145)), Some(&tables), &mut desktop);
        let ok = killer_ui::henna::OK_BTN_RECT;
        henna.info_mouse_down(ok.x + 2, ok.y + 2, &mut desktop);
        henna.info_mouse_up(Some((WindowId::HENNA_INFO, 2, 300)), &mut desktop);
        assert!(henna.commands.is_empty());
        assert!(desktop.window(WindowId::HENNA_INFO).is_some_and(|w| w.visible));

        let close = (256 - killer_ui::frame::CLOSE_INSET, killer_ui::frame::BUTTON_Y);
        henna.info_mouse_down(close.0, close.1, &mut desktop);
        assert!(desktop.window(WindowId::HENNA_INFO).is_some_and(|w| !w.visible));

        let list = HennaEquipList { adena: 0, slots: 0, offers: vec![offer()] };
        henna.world_event(&WorldEvent::HennaEquipList(list), Some(&tables), &mut desktop);
        henna.list_mouse_down(close.0, close.1, &mut desktop);
        assert!(desktop.window(WindowId::HENNA_LIST).is_some_and(|w| !w.visible));
        assert!(henna.commands.is_empty());
    }

    /// A roda e a barra rolam a lista pela produção.
    #[test]
    fn the_wheel_and_the_bar_scroll_the_list() {
        let (tables, mut desktop, mut henna) = (tables(), desktop(), windows());
        let offers = (0..8).map(|_| offer()).collect();
        henna.world_event(&WorldEvent::HennaEquipList(HennaEquipList { adena: 0, slots: 0, offers }), Some(&tables), &mut desktop);
        assert_eq!(henna.list.rows.len(), 8);
        henna.wheel(-1);
        assert_eq!(henna.list.first_row, 1);
        henna.wheel(1);
        assert_eq!(henna.list.first_row, 0);
        let rect = killer_ui::henna::LIST_RECT;
        let (x, y) = (rect.x + rect.w - 3, rect.y + rect.h - 3);
        henna.list_mouse_down(x, y, &mut desktop);
        henna.list_mouse_up(Some((WindowId::HENNA_LIST, x, y)));
        assert_eq!(henna.list.first_row, 1);
    }
}
