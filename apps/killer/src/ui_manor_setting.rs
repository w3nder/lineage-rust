use killer_assets::client::DatTables;
use killer_ui::dialog::{DialogKind, DialogView};
use killer_ui::font::BitmapFont;
use killer_ui::manor_crop_sell::{self as sell, ChangeAction, CropSellView, DetailRow, ProcureChangeView, SellAction, SellRow};
use killer_ui::manor_crop_setting::{
    self as crop, CropChangeView, CropRow, CropSettingView,
};
use killer_ui::manor_setting::{
    CHANGE_ID, ChangeOutcome, Confirm, SETTING_ID, SeedChangeView, SeedSettingView, SettingAction, SettingRow,
};
use killer_ui::window::{Desktop, WindowId};
use killer_world::{NetCommand, WorldEvent};

use crate::ui_keys::UiCommand;

pub struct ManorSettingWindow {
    pub setting: SeedSettingView,
    pub change: SeedChangeView,
    pub crop_setting: CropSettingView,
    pub crop_change: CropChangeView,
    pub crop_sell: CropSellView,
    pub procure_change: ProcureChangeView,
    castle_texts: std::collections::HashMap<u32, String>,
    ask: Option<(bool, Confirm)>,
    answering: Option<(bool, Confirm)>,
    last_tick: Option<killer_platform::time::Instant>,
    pub commands: Vec<UiCommand>,
}

impl ManorSettingWindow {
    pub fn new(lookup: impl Fn(u32) -> Option<String> + Copy) -> Self {
        Self {
            setting: SeedSettingView::new(lookup),
            change: SeedChangeView::new(lookup),
            crop_setting: CropSettingView::new(lookup),
            crop_change: CropChangeView::new(lookup),
            crop_sell: CropSellView::new(lookup),
            procure_change: ProcureChangeView::new(lookup),
            castle_texts: killer_ui::manor_info::CASTLE_NAMES
                .iter()
                .filter_map(|(_, sysstring)| lookup(*sysstring).map(|text| (*sysstring, text)))
                .collect(),
            ask: None,
            answering: None,
            last_tick: None,
            commands: Vec::new(),
        }
    }

    pub fn world_event(&mut self, event: &WorldEvent, tables: Option<&DatTables>, desktop: &mut Desktop) -> bool {
        let item_name = |id: i32| {
            u32::try_from(id)
                .ok()
                .and_then(|id| tables.and_then(|tables| tables.itemname.get(&id)))
                .map(|item| item.name.clone())
                .unwrap_or_default()
        };
        let castles = self.castle_texts.clone();
        let castle_lookup = move |id: u32| castles.get(&id).cloned();
        if let WorldEvent::ShowSellCropList(list) = event {
            let rows: Vec<SellRow> = list
                .entries
                .iter()
                .map(|entry| SellRow {
                    object: entry.values[0],
                    crop: entry.values[1],
                    value2: entry.values[2],
                    first: entry.first.clone(),
                    second: entry.second.clone(),
                    manor: entry.tail[0],
                    tail: [entry.tail[1], entry.tail[2]],
                    flag: i32::from(entry.flag as i8),
                    have: entry.value,
                })
                .collect();
            self.crop_sell.set_data(&rows, &item_name, &castle_lookup);
            show(desktop, sell::SELL_ID, true);
            return true;
        }
        if let WorldEvent::ShowProcureCropDetail(detail) = event {
            let rows: Vec<DetailRow> = detail
                .entries
                .iter()
                .map(|entry| DetailRow { manor: entry.values[0], values: [entry.values[1], entry.values[2], i32::from(entry.flag as i8)] })
                .collect();
            self.procure_change.set_data(&rows, &castle_lookup);
            show(desktop, sell::CHANGE_ID, true);
            return true;
        }
        if let WorldEvent::ShowCropSetting(setting) = event {
            let rows: Vec<CropRow> = setting
                .entries
                .iter()
                .map(|entry| CropRow {
                    item: entry.values[0],
                    second: entry.values[1],
                    limit: entry.settings[0],
                    numbers: entry.settings,
                    flag: i32::from(entry.flag as i8),
                    tail: entry.tail,
                    last: i32::from(entry.last as i8),
                    first_items: entry.first.clone(),
                    second_items: entry.second.clone(),
                })
                .collect();
            self.crop_setting.set_data(setting.value, &rows, &item_name);
            show(desktop, crop::SETTING_ID, true);
            return true;
        }
        let WorldEvent::ShowSeedSetting(setting) = event else { return false };
        let rows: Vec<SettingRow> = setting
            .entries
            .iter()
            .map(|entry| SettingRow {
                item: entry.values[0],
                values: entry.values[1],
                limit: entry.tail[0],
                unit: entry.tail[1],
                numbers: entry.tail,
                first: entry.first.clone(),
                second: entry.second.clone(),
            })
            .collect();
        self.setting.set_data(setting.value, &rows, &item_name);
        show(desktop, SETTING_ID, true);
        true
    }

    pub fn mouse_down_setting(&mut self, x: i32, y: i32, desktop: &mut Desktop) -> bool {
        if self.setting.mouse_down(x, y) == Some(killer_ui::manor_setting::SettingHit::Close) {
            show(desktop, SETTING_ID, false);
        }
        true
    }

    pub fn mouse_down_change(&mut self, x: i32, y: i32, font: &BitmapFont, desktop: &mut Desktop) -> bool {
        if self.change.mouse_down(x, y, font) == Some(killer_ui::manor_setting::ChangeHit::Close) {
            show(desktop, CHANGE_ID, false);
        }
        true
    }

    pub fn mouse_down_crop_setting(&mut self, x: i32, y: i32, desktop: &mut Desktop) -> bool {
        if self.crop_setting.mouse_down(x, y) == Some(crop::SettingHit::Close) {
            show(desktop, crop::SETTING_ID, false);
        }
        true
    }

    pub fn mouse_down_crop_change(&mut self, x: i32, y: i32, font: &BitmapFont, desktop: &mut Desktop) -> bool {
        if self.crop_change.mouse_down(x, y, font) == Some(crop::ChangeHit::Close) {
            show(desktop, crop::CHANGE_ID, false);
        }
        true
    }

    pub fn mouse_down_sell(&mut self, x: i32, y: i32, desktop: &mut Desktop) -> bool {
        if self.crop_sell.mouse_down(x, y) == Some(sell::SellHit::Close) {
            show(desktop, sell::SELL_ID, false);
        }
        true
    }

    pub fn mouse_down_procure(&mut self, x: i32, y: i32, font: &BitmapFont, desktop: &mut Desktop) -> bool {
        if self.procure_change.mouse_down(x, y, font) == Some(sell::ChangeHit::Close) {
            show(desktop, sell::CHANGE_ID, false);
        }
        true
    }

    pub fn double_click(&mut self, id: WindowId, x: i32, y: i32, desktop: &mut Desktop) -> bool {
        match id {
            SETTING_ID => match self.setting.double_click(x, y) {
                Some(action) => {
                    self.setting_action(action, desktop);
                    true
                }
                None => false,
            },
            crop::SETTING_ID => match self.crop_setting.double_click(x, y) {
                Some(action) => {
                    self.crop_setting_action(action, desktop);
                    true
                }
                None => false,
            },
            sell::SELL_ID => match self.crop_sell.double_click(x, y) {
                Some(action) => {
                    self.sell_action(action, desktop);
                    true
                }
                None => false,
            },
            _ => false,
        }
    }

    pub fn blur_outside(&mut self, over: Option<WindowId>) {
        if over != Some(sell::CHANGE_ID) {
            self.procure_change.blur();
        }
        if over != Some(CHANGE_ID) {
            self.change.blur();
        }
        if over != Some(crop::CHANGE_ID) {
            self.crop_change.blur();
        }
    }

    pub fn mouse_move(&mut self, id: WindowId, x: i32, y: i32, font: &BitmapFont) -> bool {
        match id {
            SETTING_ID => {
                self.setting.mouse_move(x, y);
                true
            }
            CHANGE_ID => {
                self.change.mouse_move(x, font);
                true
            }
            crop::SETTING_ID => {
                self.crop_setting.mouse_move(x, y);
                true
            }
            crop::CHANGE_ID => {
                self.crop_change.mouse_move(x, y, font);
                true
            }
            sell::SELL_ID => {
                self.crop_sell.mouse_move(x, y);
                true
            }
            sell::CHANGE_ID => {
                self.procure_change.mouse_move(x, y, font);
                true
            }
            _ => false,
        }
    }

    pub fn wheel(&mut self, id: WindowId, delta: i16) -> bool {
        match id {
            SETTING_ID => self.setting.wheel(delta),
            crop::SETTING_ID => self.crop_setting.wheel(delta),
            sell::SELL_ID => self.crop_sell.wheel(delta),
            sell::CHANGE_ID => self.procure_change.wheel(delta),
            _ => return false,
        }
        true
    }

    pub fn mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        let (x, y) = over.filter(|(id, _, _)| *id == SETTING_ID).map_or((i32::MIN / 2, i32::MIN / 2), |(_, x, y)| (x, y));
        if let Some(action) = self.setting.mouse_up(x, y) {
            self.setting_action(action, desktop);
        }
        let (x, y) = over.filter(|(id, _, _)| *id == CHANGE_ID).map_or((i32::MIN / 2, i32::MIN / 2), |(_, x, y)| (x, y));
        if let Some(outcome) = self.change.mouse_up(x, y) {
            self.change_outcome(outcome, desktop);
        }
        let (x, y) = over.filter(|(id, _, _)| *id == crop::SETTING_ID).map_or((i32::MIN / 2, i32::MIN / 2), |(_, x, y)| (x, y));
        if let Some(action) = self.crop_setting.mouse_up(x, y) {
            self.crop_setting_action(action, desktop);
        }
        let (x, y) = over.filter(|(id, _, _)| *id == crop::CHANGE_ID).map_or((i32::MIN / 2, i32::MIN / 2), |(_, x, y)| (x, y));
        if let Some(outcome) = self.crop_change.mouse_up(x, y) {
            self.crop_change_outcome(outcome, desktop);
        }
        let (x, y) = over.filter(|(id, _, _)| *id == sell::SELL_ID).map_or((i32::MIN / 2, i32::MIN / 2), |(_, x, y)| (x, y));
        if let Some(action) = self.crop_sell.mouse_up(x, y) {
            self.sell_action(action, desktop);
        }
        let (x, y) = over.filter(|(id, _, _)| *id == sell::CHANGE_ID).map_or((i32::MIN / 2, i32::MIN / 2), |(_, x, y)| (x, y));
        let have = self.crop_sell.have();
        if let Some(action) = self.procure_change.mouse_up(x, y, have) {
            self.procure_action(action, desktop);
        }
    }

    fn sell_action(&mut self, action: SellAction, desktop: &mut Desktop) {
        match action {
            SellAction::Close => show(desktop, sell::SELL_ID, false),
            SellAction::OpenChange(open) => {
                self.procure_change.set_texts(&open);
                self.commands.push(UiCommand::Net(NetCommand::Bypass { command: open.bypass }));
            }
            SellAction::Submit(orders) => {
                let orders = orders
                    .into_iter()
                    .map(|(object_id, crop_id, manor_id, count)| l2_net::game::CropProcure { object_id, crop_id, manor_id, count })
                    .collect();
                self.commands.push(UiCommand::Net(NetCommand::ProcureCropList { orders }));
                show(desktop, sell::SELL_ID, false);
            }
        }
    }

    fn procure_action(&mut self, action: ChangeAction, desktop: &mut Desktop) {
        match action {
            ChangeAction::Close => show(desktop, sell::CHANGE_ID, false),
            ChangeAction::Apply { count, manor, values } => {
                let castles = self.castle_texts.clone();
                let castle_lookup = move |id: u32| castles.get(&id).cloned();
                self.crop_sell.apply_item(count, manor, values, &castle_lookup);
                show(desktop, sell::CHANGE_ID, false);
                show(desktop, sell::SELL_ID, true);
            }
        }
    }

    fn crop_setting_action(&mut self, action: crop::SettingAction, desktop: &mut Desktop) {
        match action {
            crop::SettingAction::Close => show(desktop, crop::SETTING_ID, false),
            crop::SettingAction::OpenChange => {
                if let Some(selected) = self.crop_setting.change_crop() {
                    self.crop_change.open(&selected);
                    show(desktop, crop::CHANGE_ID, true);
                }
            }
            crop::SettingAction::Confirm(confirm) => self.ask = Some((true, confirm)),
            crop::SettingAction::Submit { manor, crops } => {
                let crops = crops
                    .into_iter()
                    .map(|(crop_id, amount, price, reward)| l2_net::game::CropSetting { crop_id, amount, price, reward })
                    .collect();
                self.commands.push(UiCommand::Net(NetCommand::SetCrop { manor_id: manor, crops }));
                show(desktop, crop::SETTING_ID, false);
            }
        }
    }

    fn crop_change_outcome(&mut self, outcome: crop::ChangeOutcome, desktop: &mut Desktop) {
        match outcome {
            crop::ChangeOutcome::Close => show(desktop, crop::CHANGE_ID, false),
            crop::ChangeOutcome::Apply { amount, price, reward } => {
                show(desktop, crop::CHANGE_ID, false);
                self.crop_setting.apply_change(amount, price, reward);
                show(desktop, crop::SETTING_ID, true);
            }
            crop::ChangeOutcome::Rejected { message, numbers } => {
                self.commands.push(UiCommand::WarningMessage { id: message, numbers: numbers.to_vec() });
            }
        }
    }

    fn setting_action(&mut self, action: SettingAction, desktop: &mut Desktop) {
        match action {
            SettingAction::Close => show(desktop, SETTING_ID, false),
            SettingAction::OpenChange => {
                if let Some(seed) = self.setting.change_seed() {
                    self.change.open(&seed);
                    show(desktop, CHANGE_ID, true);
                }
            }
            SettingAction::Confirm(confirm) => self.ask = Some((false, confirm)),
            SettingAction::Submit { manor, seeds } => {
                self.commands.push(UiCommand::Net(NetCommand::SetSeed { manor_id: manor, seeds }));
                show(desktop, SETTING_ID, false);
            }
        }
    }

    fn change_outcome(&mut self, outcome: ChangeOutcome, desktop: &mut Desktop) {
        match outcome {
            ChangeOutcome::Close => show(desktop, CHANGE_ID, false),
            ChangeOutcome::Apply { quantity, price } => {
                show(desktop, CHANGE_ID, false);
                self.setting.apply_change(quantity, price);
                show(desktop, SETTING_ID, true);
            }
            ChangeOutcome::Rejected { message, numbers } => {
                self.commands.push(UiCommand::WarningMessage { id: message, numbers: numbers.to_vec() });
            }
        }
    }

    pub fn key(&mut self, key: killer_ui::chat::InputKey, font: &BitmapFont, desktop: &Desktop) -> bool {
        if desktop.window(CHANGE_ID).is_some_and(|window| window.visible) && self.change.key(key, font) {
            return true;
        }
        if desktop.window(crop::CHANGE_ID).is_some_and(|window| window.visible) && self.crop_change.key(key, font) {
            return true;
        }
        desktop.window(sell::CHANGE_ID).is_some_and(|window| window.visible) && self.procure_change.key(key, font)
    }

    pub fn shift(&mut self, pressed: bool) -> bool {
        let seed = self.change.shift(pressed);
        let procure = self.procure_change.shift(pressed);
        self.crop_change.shift(pressed) || seed || procure
    }

    pub fn tick(&mut self, now: killer_platform::time::Instant) {
        if let Some(last) = self.last_tick.replace(now) {
            let elapsed = now.saturating_duration_since(last).as_millis().min(u128::from(u32::MAX)) as u32;
            self.change.tick(elapsed);
            self.crop_change.tick(elapsed);
            self.procure_change.tick(elapsed);
        }
    }

    pub fn take_dialog(&mut self, tables: Option<&DatTables>) -> Option<DialogView> {
        let (is_crop, confirm) = self.ask.take()?;
        let mut view = DialogView::new(DialogKind::OkCancel, &crate::ui2d::system_text(tables, confirm.message(), &[]));
        view.dialog_type = confirm.dialog_type();
        self.answering = Some((is_crop, confirm));
        Some(view)
    }

    pub fn answer(&mut self, accept: bool) {
        let Some((is_crop, confirm)) = self.answering.take() else { return };
        if accept {
            match is_crop {
                true => self.crop_setting.answer(confirm),
                false => self.setting.answer(confirm),
            }
        }
    }
}

fn show(desktop: &mut Desktop, id: WindowId, visible: bool) {
    if let Some(window) = desktop.window_mut(id) {
        window.visible = visible;
    }
    if visible && let Some(index) = desktop.windows.iter().position(|window| window.id == id) {
        let window = desktop.windows.remove(index);
        desktop.windows.push(window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::manor_setting::{self, SettingHit};
    use killer_ui::window::{Rect, Window};
    use l2_net::game::{SeedSetting, SeedSettingEntry};

    fn font() -> BitmapFont {
        BitmapFont::new(
            "L2Font-e.SmallFont-e".into(),
            killer_ui::font::Glyphs::parse(include_bytes!("../../../crates/killer-ui/tests/fixtures/smallfont-e.gly")).unwrap(),
        )
    }

    fn window() -> (ManorSettingWindow, Desktop) {
        let mut desktop = Desktop::new((1280, 800));
        for (id, size, grip) in [
            (SETTING_ID, manor_setting::SETTING_SIZE, manor_setting::setting_grip()),
            (CHANGE_ID, manor_setting::CHANGE_SIZE, manor_setting::change_grip()),
        ] {
            let mut window = Window::new(id, Rect::new(0, 0, size.0, size.1), grip);
            window.visible = false;
            desktop.windows.push(window);
        }
        (ManorSettingWindow::new(|id| Some(format!("s{id}"))), desktop)
    }

    fn entry(item: i32) -> SeedSettingEntry {
        SeedSettingEntry { values: [item, 9], first: vec![1], second: vec![2], tail: [50, 2, 10, 20, 13, 14, 15, 16] }
    }

    fn show(window: &mut ManorSettingWindow, desktop: &mut Desktop) {
        let event = WorldEvent::ShowSeedSetting(SeedSetting { value: 3, entries: vec![entry(100), entry(101)] });
        assert!(window.world_event(&event, None, desktop));
    }

    fn click(window: &mut ManorSettingWindow, desktop: &mut Desktop, rect: Rect) {
        let (x, y) = (rect.x + 2, rect.y + 2);
        window.mouse_down_setting(x, y, desktop);
        window.mouse_up(Some((SETTING_ID, x, y)), desktop);
    }

    #[test]
    fn the_packet_fills_the_rows_and_shows_the_window() {
        let (mut window, mut desktop) = window();
        assert!(!window.world_event(&WorldEvent::SendManorList(l2_net::game::ManorList { entries: Vec::new() }), None, &mut desktop));
        show(&mut window, &mut desktop);
        assert!(desktop.window(SETTING_ID).unwrap().visible);
        assert_eq!(window.setting.manor, 3);
        assert_eq!(window.setting.list.count(), 2);
        assert_eq!(window.setting.data, vec![(100, 50, 2), (101, 50, 2)]);
        assert_eq!(window.setting.total, 15 * 2 * 2);
    }

    #[test]
    fn change_button_opens_the_change_window_for_the_selected_row_and_ok_applies_it() {
        let (mut window, mut desktop) = window();
        show(&mut window, &mut desktop);
        let change_button = Rect::new(8, 297, 76, 23);
        click(&mut window, &mut desktop, change_button);
        assert!(!desktop.window(CHANGE_ID).unwrap().visible);
        window.setting.list.select(1);
        click(&mut window, &mut desktop, change_button);
        assert!(desktop.window(CHANGE_ID).unwrap().visible);
        assert_eq!(window.change.limit, 50);
        assert!(window.change.quantity.focused);
        let font = font();
        assert!(window.key(killer_ui::chat::InputKey::Backspace, &font, &desktop));
        assert!(window.key(killer_ui::chat::InputKey::Backspace, &font, &desktop));
        assert!(window.key(killer_ui::chat::InputKey::Char('9'), &font, &desktop));
        let ok = Rect::new(51, 172, 76, 23);
        window.mouse_down_change(ok.x + 2, ok.y + 2, &font, &mut desktop);
        window.mouse_up(Some((CHANGE_ID, ok.x + 2, ok.y + 2)), &mut desktop);
        assert!(!desktop.window(CHANGE_ID).unwrap().visible);
        assert_eq!(window.setting.cell(1, manor_setting::CELL_QUANTITY), "9");
        assert_eq!(window.setting.cell(1, manor_setting::CELL_PRICE), "16");
        assert_eq!(window.setting.total, 15 * 2 + 9 * 2);
        assert!(window.commands.is_empty());
    }

    #[test]
    fn an_out_of_range_quantity_keeps_the_window_and_reports_system_message_0x616() {
        let (mut window, mut desktop) = window();
        show(&mut window, &mut desktop);
        window.setting.list.select(0);
        click(&mut window, &mut desktop, Rect::new(8, 297, 76, 23));
        let font = font();
        for key in [killer_ui::chat::InputKey::Backspace, killer_ui::chat::InputKey::Backspace, killer_ui::chat::InputKey::Char('9'), killer_ui::chat::InputKey::Char('9')] {
            window.key(key, &font, &desktop);
        }
        let ok = Rect::new(51, 172, 76, 23);
        window.mouse_down_change(ok.x + 2, ok.y + 2, &font, &mut desktop);
        window.mouse_up(Some((CHANGE_ID, ok.x + 2, ok.y + 2)), &mut desktop);
        assert!(desktop.window(CHANGE_ID).unwrap().visible);
        assert!(matches!(window.commands.as_slice(), [UiCommand::WarningMessage { id: 0x616, numbers }] if numbers == &[0, 50]));
    }

    #[test]
    fn ok_sends_every_row_and_hides_the_window() {
        let (mut window, mut desktop) = window();
        show(&mut window, &mut desktop);
        click(&mut window, &mut desktop, Rect::new(438, 297, 76, 23));
        assert!(!desktop.window(SETTING_ID).unwrap().visible);
        assert!(matches!(
            window.commands.as_slice(),
            [UiCommand::Net(NetCommand::SetSeed { manor_id: 3, seeds })] if seeds == &[(100, 15, 16), (101, 15, 16)]
        ));
    }

    #[test]
    fn cancel_and_close_only_hide_the_window() {
        let (mut window, mut desktop) = window();
        show(&mut window, &mut desktop);
        click(&mut window, &mut desktop, Rect::new(518, 297, 76, 23));
        assert!(!desktop.window(SETTING_ID).unwrap().visible);
        assert!(window.commands.is_empty());
        show(&mut window, &mut desktop);
        let close = Rect::new(577, 3, 15, 15);
        assert_eq!(window.setting.hit(close.x + 2, close.y + 2), Some(SettingHit::Close));
        window.mouse_down_setting(close.x + 2, close.y + 2, &mut desktop);
        assert!(!desktop.window(SETTING_ID).unwrap().visible);
    }

    #[test]
    fn the_confirmation_dialogs_use_the_native_types_and_only_accept_changes_the_rows() {
        let (mut window, mut desktop) = window();
        show(&mut window, &mut desktop);
        click(&mut window, &mut desktop, Rect::new(168, 297, 76, 23));
        let view = window.take_dialog(None).unwrap();
        assert_eq!((view.dialog_type, view.kind), (0x56, DialogKind::OkCancel));
        assert!(window.take_dialog(None).is_none());
        window.answer(false);
        assert_eq!(window.setting.total, 60);
        click(&mut window, &mut desktop, Rect::new(168, 297, 76, 23));
        window.take_dialog(None).unwrap();
        window.answer(true);
        assert_eq!(window.setting.total, 0);
        click(&mut window, &mut desktop, Rect::new(88, 297, 76, 23));
        let view = window.take_dialog(None).unwrap();
        assert_eq!(view.dialog_type, 0x57);
        window.answer(true);
        assert_eq!(window.setting.total, 13 * 2 * 2);
    }

    fn crop_window() -> (ManorSettingWindow, Desktop) {
        let (window, mut desktop) = window();
        for (id, size, grip) in [
            (crop::SETTING_ID, crop::SETTING_SIZE, crop::setting_grip()),
            (crop::CHANGE_ID, crop::CHANGE_SIZE, crop::change_grip()),
        ] {
            let mut entry = Window::new(id, Rect::new(0, 0, size.0, size.1), grip);
            entry.visible = false;
            desktop.windows.push(entry);
        }
        (window, desktop)
    }

    fn crop_entry(item: i32) -> l2_net::game::CropSettingInfoEntry {
        l2_net::game::CropSettingInfoEntry {
            values: [item, 9],
            first: vec![1],
            second: vec![2],
            settings: [50, 2, 10, 20, 13, 14],
            flag: 3,
            tail: [15, 16],
            last: 2,
        }
    }

    fn crop_show(window: &mut ManorSettingWindow, desktop: &mut Desktop) {
        let event = WorldEvent::ShowCropSetting(l2_net::game::CropSettingInfo { value: 3, entries: vec![crop_entry(100), crop_entry(101)] });
        assert!(window.world_event(&event, None, desktop));
    }

    fn crop_click(window: &mut ManorSettingWindow, desktop: &mut Desktop, rect: Rect) {
        let (x, y) = (rect.x + 2, rect.y + 2);
        window.mouse_down_crop_setting(x, y, desktop);
        window.mouse_up(Some((crop::SETTING_ID, x, y)), desktop);
    }

    #[test]
    fn the_crop_packet_fills_the_rows_and_ok_sends_set_crop() {
        let (mut window, mut desktop) = crop_window();
        crop_show(&mut window, &mut desktop);
        assert!(desktop.window(crop::SETTING_ID).unwrap().visible);
        assert_eq!(window.crop_setting.total, 2 * 15 * 16);
        crop_click(&mut window, &mut desktop, Rect::new(438, 297, 76, 23));
        assert!(!desktop.window(crop::SETTING_ID).unwrap().visible);
        assert!(matches!(
            window.commands.as_slice(),
            [UiCommand::Net(NetCommand::SetCrop { manor_id: 3, crops })]
                if crops.iter().map(|crop| (crop.crop_id, crop.amount, crop.price, crop.reward)).collect::<Vec<_>>() == [(100, 15, 16, 2), (101, 15, 16, 2)]
        ));
    }

    #[test]
    fn the_crop_change_window_applies_amount_price_and_reward() {
        let (mut window, mut desktop) = crop_window();
        crop_show(&mut window, &mut desktop);
        window.crop_setting.list.select(0);
        crop_click(&mut window, &mut desktop, Rect::new(8, 297, 76, 23));
        assert!(desktop.window(crop::CHANGE_ID).unwrap().visible);
        assert_eq!(window.crop_change.reward_value(), 2);
        let font = font();
        window.key(killer_ui::chat::InputKey::Backspace, &font, &desktop);
        window.key(killer_ui::chat::InputKey::Backspace, &font, &desktop);
        window.key(killer_ui::chat::InputKey::Char('6'), &font, &desktop);
        let ok = Rect::new(51, 172, 76, 23);
        window.mouse_down_crop_change(ok.x + 2, ok.y + 2, &font, &mut desktop);
        window.mouse_up(Some((crop::CHANGE_ID, ok.x + 2, ok.y + 2)), &mut desktop);
        assert!(!desktop.window(crop::CHANGE_ID).unwrap().visible);
        assert_eq!(window.crop_setting.cell(0, crop::CELL_AMOUNT), "6");
        assert_eq!(window.crop_setting.total, 6 * 16 + 15 * 16);
        assert!(window.commands.is_empty());
    }

    #[test]
    fn crop_dialogs_use_their_own_target_and_a_rejected_change_reports_0x618() {
        let (mut window, mut desktop) = crop_window();
        crop_show(&mut window, &mut desktop);
        crop_click(&mut window, &mut desktop, Rect::new(168, 297, 76, 23));
        let view = window.take_dialog(None).unwrap();
        assert_eq!(view.dialog_type, 0x56);
        window.answer(true);
        assert_eq!(window.crop_setting.total, 0);
        assert_eq!(window.setting.total, 0);
        window.crop_setting.list.select(0);
        crop_click(&mut window, &mut desktop, Rect::new(8, 297, 76, 23));
        let font = font();
        for key in [killer_ui::chat::InputKey::Backspace, killer_ui::chat::InputKey::Backspace, killer_ui::chat::InputKey::Char('9'), killer_ui::chat::InputKey::Char('9')] {
            window.key(key, &font, &desktop);
        }
        let ok = Rect::new(51, 172, 76, 23);
        window.mouse_down_crop_change(ok.x + 2, ok.y + 2, &font, &mut desktop);
        window.mouse_up(Some((crop::CHANGE_ID, ok.x + 2, ok.y + 2)), &mut desktop);
        assert!(desktop.window(crop::CHANGE_ID).unwrap().visible);
        assert!(matches!(window.commands.as_slice(), [UiCommand::WarningMessage { id: 0x618, numbers }] if numbers == &[0, 50]));
    }

    fn sell_window() -> (ManorSettingWindow, Desktop) {
        let (window, mut desktop) = window();
        for (id, size, grip) in [
            (sell::SELL_ID, sell::SELL_SIZE, sell::sell_grip()),
            (sell::CHANGE_ID, sell::CHANGE_SIZE, sell::change_grip()),
        ] {
            let mut entry = Window::new(id, Rect::new(0, 0, size.0, size.1), grip);
            entry.visible = false;
            desktop.windows.push(entry);
        }
        (window, desktop)
    }

    fn sell_entry(object: i32, crop: i32) -> l2_net::game::SellCropEntry {
        l2_net::game::SellCropEntry { values: [object, crop, 7], first: vec![1], second: vec![2], tail: [1, 11, 12], flag: 1, value: 40 }
    }

    fn show_sell(window: &mut ManorSettingWindow, desktop: &mut Desktop) {
        let event = WorldEvent::ShowSellCropList(l2_net::game::SellCropList { value: 0, entries: vec![sell_entry(10, 100), sell_entry(11, 101)] });
        assert!(window.world_event(&event, None, desktop));
    }

    #[test]
    fn the_sell_list_opens_the_change_window_through_a_bypass_and_the_detail_packet() {
        let (mut window, mut desktop) = sell_window();
        show_sell(&mut window, &mut desktop);
        assert!(desktop.window(sell::SELL_ID).unwrap().visible);
        assert_eq!(window.crop_sell.list.count(), 2);
        assert_eq!(window.crop_sell.cell(0, sell::CELL_CASTLE), "s1049");
        window.crop_sell.list.select(1);
        let change = Rect::new(8, 297, 76, 23);
        window.mouse_down_sell(change.x + 2, change.y + 2, &mut desktop);
        window.mouse_up(Some((sell::SELL_ID, change.x + 2, change.y + 2)), &mut desktop);
        assert!(matches!(
            window.commands.as_slice(),
            [UiCommand::Net(NetCommand::Bypass { command })] if command == "manor_menu_select?ask=9&state=101&time=0"
        ));
        assert!(!desktop.window(sell::CHANGE_ID).unwrap().visible);
        assert_eq!(window.procure_change.name, window.crop_sell.cell(1, 0));
        let detail = WorldEvent::ShowProcureCropDetail(l2_net::game::ProcureCropDetail {
            value: 101,
            entries: vec![l2_net::game::ProcureCropDetailEntry { values: [1, 30, 3], flag: 4 }],
        });
        assert!(window.world_event(&detail, None, &mut desktop));
        assert!(desktop.window(sell::CHANGE_ID).unwrap().visible);
        assert_eq!(window.procure_change.rows, vec![DetailRow { manor: 1, values: [30, 3, 4] }]);
    }

    #[test]
    fn change_ok_rewrites_the_sell_row_and_the_sell_ok_sends_the_procure_list() {
        let (mut window, mut desktop) = sell_window();
        show_sell(&mut window, &mut desktop);
        window.crop_sell.list.select(1);
        let detail = WorldEvent::ShowProcureCropDetail(l2_net::game::ProcureCropDetail {
            value: 101,
            entries: vec![l2_net::game::ProcureCropDetailEntry { values: [1, 30, 3], flag: 4 }],
        });
        window.world_event(&detail, None, &mut desktop);
        let font = font();
        window.key(killer_ui::chat::InputKey::Char('5'), &font, &desktop);
        let ok = Rect::new(91, 277, 76, 23);
        window.mouse_down_procure(ok.x + 2, ok.y + 2, &font, &mut desktop);
        window.mouse_up(Some((sell::CHANGE_ID, ok.x + 2, ok.y + 2)), &mut desktop);
        assert!(!desktop.window(sell::CHANGE_ID).unwrap().visible);
        assert_eq!(window.crop_sell.cell(1, sell::CELL_SELL), "5");
        assert_eq!(window.crop_sell.cell(1, sell::CELL_PRICE_A), "30");
        let submit = Rect::new(438, 297, 76, 23);
        window.mouse_down_sell(submit.x + 2, submit.y + 2, &mut desktop);
        window.mouse_up(Some((sell::SELL_ID, submit.x + 2, submit.y + 2)), &mut desktop);
        assert!(!desktop.window(sell::SELL_ID).unwrap().visible);
        assert!(matches!(
            window.commands.as_slice(),
            [UiCommand::Net(NetCommand::ProcureCropList { orders })]
                if orders.iter().map(|o| (o.object_id, o.crop_id, o.manor_id, o.count)).collect::<Vec<_>>() == [(11, 101, 1, 5)]
        ));
    }

    #[test]
    fn double_clicking_a_list_row_opens_the_change_flow_of_each_window() {
        let (mut window, mut desktop) = sell_window();
        show_sell(&mut window, &mut desktop);
        let rect = window.crop_sell.list.rect;
        let y = rect.y + window.crop_sell.list.header_height + 2;
        assert!(!window.double_click(sell::SELL_ID, 1, 1, &mut desktop));
        assert!(window.double_click(sell::SELL_ID, rect.x + 3, y, &mut desktop));
        assert!(matches!(window.commands.as_slice(), [UiCommand::Net(NetCommand::Bypass { .. })]));
        show(&mut window, &mut desktop);
        let rect = window.setting.list.rect;
        let y = rect.y + window.setting.list.header_height + 2;
        assert!(window.double_click(SETTING_ID, rect.x + 3, y, &mut desktop));
        assert!(desktop.window(CHANGE_ID).unwrap().visible);
    }
}
