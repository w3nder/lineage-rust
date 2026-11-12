use killer_assets::client::DatTables;
use killer_ui::broadcast::{self, BroadcastObserver, BuffCell, GM_MSG_HOLD_MS, PartyHit, TEAMS, party_hit, special_hold_ms};
use killer_ui::chat::InputKey;
use killer_ui::draw::DrawList;
use killer_ui::font::BitmapFont;
use killer_ui::window::{Desktop, Rect, WindowId};
use killer_world::WorldEvent;
use killer_world::appendix::{AppendixOp, CanvasOp, IDENTIFIER_BLUE, IDENTIFIER_RED, SELECTED_BLUE, SELECTED_RED};

use crate::ui_olympiad::OlympiadWindows;

pub const STATE_OBSERVER: i32 = 1;
pub const PLAYERS: [WindowId; TEAMS] = [WindowId::BROADCAST_PLAYER_0, WindowId::BROADCAST_PLAYER_1];
pub const PARTIES: [WindowId; TEAMS] = [WindowId::BROADCAST_PARTY_0, WindowId::BROADCAST_PARTY_1];
pub const CHILDREN: [WindowId; 6] = [
    WindowId::BROADCAST_TOP,
    WindowId::BROADCAST_PLAYER_0,
    WindowId::BROADCAST_PLAYER_1,
    WindowId::BROADCAST_PARTY_0,
    WindowId::BROADCAST_PARTY_1,
    WindowId::BROADCAST_MSG,
];
pub const BLINK_MS: i32 = 100;

fn identifier_bit(team: usize) -> u32 {
    if team == 0 { IDENTIFIER_RED } else { IDENTIFIER_BLUE }
}

fn selected_bit(team: usize) -> u32 {
    if team == 0 { SELECTED_RED } else { SELECTED_BLUE }
}

pub struct BroadcastWindows {
    pub observer: BroadcastObserver,
    pub gm_text: String,
    pub special_code: i32,
    pub buffs: [Vec<BuffCell>; TEAMS],
    gm_left: Option<i32>,
    special_left: Option<i32>,
    icon_size: [i32; TEAMS],
    blink_ms: i32,
    second_ms: i32,
    viewport: (i32, i32),
    pub marks: Vec<AppendixOp>,
    pub controller: Vec<bool>,
    pub canvas_ops: Vec<CanvasOp>,
    pub true_fonts: killer_ui::true_font::TrueFonts,
}

impl BroadcastWindows {
    pub fn new(viewport: (i32, i32)) -> Self {
        Self {
            observer: BroadcastObserver::default(),
            gm_text: String::new(),
            special_code: 0,
            buffs: Default::default(),
            gm_left: None,
            special_left: None,
            icon_size: [broadcast::BUFF_INITIAL_ICON_SIZE; TEAMS],
            blink_ms: 0,
            second_ms: 0,
            viewport,
            marks: Vec::new(),
            controller: Vec::new(),
            canvas_ops: Vec::new(),
            true_fonts: Default::default(),
        }
    }

    fn user_id(&self, team: usize, index: usize) -> Option<i32> {
        self.observer.user(team, index).map(|user| user.id)
    }

    fn show_marks(&mut self) {
        self.controller.push(true);
        for team in 0..TEAMS {
            for index in 0..broadcast::USERS {
                if let Some(object_id) = self.user_id(team, index) {
                    self.marks.push(AppendixOp::Add { object_id, bit: identifier_bit(team) });
                }
            }
            if let Some(index) = self.observer.selected[team] {
                self.select_marks(team, index);
            }
        }
    }

    fn hide_marks(&mut self) {
        self.controller.push(false);
        for team in 0..TEAMS {
            for index in 0..broadcast::USERS {
                if let Some(object_id) = self.user_id(team, index) {
                    self.marks.push(AppendixOp::Flag { object_id, set: false });
                    self.marks.push(AppendixOp::DeleteAll { object_id });
                }
            }
        }
    }

    fn select_marks(&mut self, team: usize, index: usize) {
        if team >= TEAMS {
            return;
        }
        if let Some(object_id) = self.observer.selected_user(team).map(|user| user.id) {
            self.marks.push(AppendixOp::Flag { object_id, set: false });
            self.marks.push(AppendixOp::Delete { object_id, bit: selected_bit(team) });
        }
        self.observer.select_user(team, index);
        if let Some(object_id) = self.user_id(team, index) {
            self.marks.push(AppendixOp::Flag { object_id, set: true });
            self.marks.push(AppendixOp::Add { object_id, bit: selected_bit(team) });
        }
    }

    fn setup_target_marks(&mut self) {
        let Some(team) = self.observer.team_info_end() else { return };
        for index in 0..broadcast::USERS {
            if let Some(object_id) = self.user_id(team, index) {
                self.marks.push(AppendixOp::Index { object_id, team: team as i32, index: index as i32 });
                self.marks.push(AppendixOp::Add { object_id, bit: identifier_bit(team) });
            }
        }
    }

    fn char_info_marks(&mut self, object_id: i32) {
        let Some((team, index)) = self.observer.find_user(object_id) else { return };
        self.marks.push(AppendixOp::Index { object_id, team: team as i32, index: index as i32 });
        self.marks.push(AppendixOp::Add { object_id, bit: identifier_bit(team) });
    }

    fn target_marks(&mut self, target: Option<i32>) {
        let Some((team, index)) = target.and_then(|id| self.observer.find_user(id)) else { return };
        self.select_marks(team, index);
    }

    pub fn set_viewport(&mut self, viewport: (i32, i32)) {
        self.viewport = viewport;
    }

    fn visible(desktop: &Desktop, id: WindowId) -> bool {
        desktop.window(id).is_some_and(|window| window.visible)
    }

    pub fn active(&self, desktop: &Desktop) -> bool {
        Self::visible(desktop, WindowId::BROADCAST_OBSERVER)
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

    fn set_rect(desktop: &mut Desktop, id: WindowId, rect: Rect) {
        if let Some(window) = desktop.window_mut(id) {
            window.rect = rect;
        }
    }

    pub fn layout(&self, desktop: &mut Desktop) {
        let width = self.viewport.0;
        let top = broadcast::top_rect(width);
        Self::set_rect(desktop, WindowId::BROADCAST_TOP, Rect::new(top.x, self.observer.top.y, top.w, top.h));
        for (team, id) in PARTIES.into_iter().enumerate() {
            let list = desktop.window(id).map(|window| window.rect);
            let fresh = broadcast::party_list_rect(team, width, self.observer.visible_rows(team));
            if let Some(list) = list {
                Self::set_rect(desktop, id, Rect::new(list.x, list.y, fresh.w, fresh.h));
            }
        }
        Self::set_rect(desktop, WindowId::BROADCAST_MSG, broadcast::msg_rect(width));
    }

    pub fn init_positions(&self, desktop: &mut Desktop) {
        let width = self.viewport.0;
        for team in 0..TEAMS {
            Self::set_rect(desktop, PLAYERS[team], broadcast::player_rect(team, width));
            let rows = self.observer.visible_rows(team);
            Self::set_rect(desktop, PARTIES[team], broadcast::party_list_rect(team, width, rows));
        }
    }

    pub fn switch_state(&mut self, state: i32, desktop: &mut Desktop, olympiad: &mut OlympiadWindows) {
        let was_on = self.active(desktop);
        olympiad.set_state(state, desktop);
        self.canvas_ops.append(&mut olympiad.canvas_ops);
        let on = state == STATE_OBSERVER;
        if was_on && !on {
            self.hide_marks();
        }
        Self::show(desktop, WindowId::BROADCAST_OBSERVER, on);
        for id in CHILDREN {
            Self::show(desktop, id, on);
        }
        if on {
            self.init_positions(desktop);
            self.layout(desktop);
            self.observer.top.show();
            self.refresh_party_visibility(desktop);
            if !was_on {
                self.show_marks();
            }
        } else {
            self.observer.top.phase = broadcast::TopPhase::Idle;
        }
    }

    fn refresh_party_visibility(&self, desktop: &mut Desktop) {
        for (team, id) in PARTIES.into_iter().enumerate() {
            let visible = self.observer.visible_rows(team) > 0;
            Self::show(desktop, id, visible);
        }
    }

    pub fn world_event(
        &mut self,
        event: &WorldEvent,
        desktop: &mut Desktop,
        olympiad: &mut OlympiadWindows,
        tables: Option<&DatTables>,
        target: Option<i32>,
    ) {
        match event {
            WorldEvent::EventMatchObserver(observer) => match observer.flags[0] {
                0 => self.switch_state(0, desktop, olympiad),
                1 => {
                    self.switch_state(STATE_OBSERVER, desktop, olympiad);
                    self.observer.start_observer(observer.value, &observer.names[0], &observer.names[1]);
                    self.refresh_party_visibility(desktop);
                }
                _ => {}
            },
            WorldEvent::EventMatchMessage(message) => match message.flag {
                0 => {
                    self.gm_text = message.message.clone();
                    self.gm_left = Some(GM_MSG_HOLD_MS);
                    Self::set_rect(desktop, WindowId::BROADCAST_GM_MSG, broadcast::gm_msg_rect(self.viewport, olympiad.state()));
                    Self::show(desktop, WindowId::BROADCAST_GM_MSG, true);
                }
                code @ 1..=8 => {
                    self.special_code = i32::from(code);
                    self.special_left = Some(special_hold_ms(i32::from(code)));
                    Self::set_rect(desktop, WindowId::BROADCAST_SPECIAL_MSG, broadcast::special_rect(self.viewport));
                    Self::show(desktop, WindowId::BROADCAST_SPECIAL_MSG, true);
                }
                _ => {}
            },
            WorldEvent::EventMatchScore(score) if self.observer.score(score.values[0], score.values[1], score.values[2]) => {
                Self::show(desktop, WindowId::BROADCAST_TOP, true);
            }
            WorldEvent::EventMatchTeamInfo(info) => {
                if olympiad.state() != STATE_OBSERVER {
                    return;
                }
                self.observer.team_info_start(info.header[0], info.header[1]);
                for (index, entry) in info.entries.iter().enumerate() {
                    self.observer.team_info(index as i32, entry.id, &entry.name, &entry.values);
                }
                self.setup_target_marks();
                self.refresh_party_visibility(desktop);
                self.layout(desktop);
            }
            WorldEvent::CharInfo(info) if self.active(desktop) => self.char_info_marks(info.object_id),
            WorldEvent::TargetSelected(_) | WorldEvent::MyTargetSelected(_) => self.target_marks(target),
            WorldEvent::EventMatchUserInfo(info) => self.observer.user_info(info.id, &info.name, &info.values),
            WorldEvent::EventMatchSpelledInfo(info) => {
                self.observer.spell_info_start(info.values[0], info.values[1]);
                for entry in &info.entries {
                    self.observer.spell_info(entry.value, entry.amount);
                }
                if let Some((team, _)) = self.observer.spell_info_end() {
                    self.refresh_buffs_of(team, tables);
                }
            }
            WorldEvent::EventMatchTeamUnlocked(unlocked) => {
                if let Some((_, ids)) = self.observer.team_unlocked(unlocked.value, i32::from(unlocked.flag)) {
                    for object_id in ids {
                        self.marks.push(AppendixOp::Flag { object_id, set: false });
                        self.marks.push(AppendixOp::DeleteAll { object_id });
                    }
                    self.refresh_party_visibility(desktop);
                    self.layout(desktop);
                }
            }
            WorldEvent::MagicSkillUse(cast) => self.magic_skill_use(cast.caster_id, cast.target_id, cast.skill_id, tables),
            _ => {}
        }
    }

    fn magic_skill_use(&mut self, caster_id: i32, target_id: i32, skill_id: i32, tables: Option<&DatTables>) {
        if !(0..=0x7cf).contains(&skill_id) {
            return;
        }
        let (Some(caster), Some(target)) = (self.observer.find_user(caster_id), self.observer.find_user(target_id)) else { return };
        let name = tables
            .and_then(|tables| tables.skillname.get(&(skill_id as u32, 1)))
            .map(|skill| skill.name.clone())
            .unwrap_or_default();
        let name_of = |(team, index): (usize, usize)| self.observer.user(team, index).map(|user| user.name.clone()).unwrap_or_default();
        let names = (name_of(caster), name_of(target));
        self.observer.add_skill_msg((caster_id, caster.0, caster.1), (target_id, target.0, target.1), &name, (&names.0, &names.1));
    }

    pub fn refresh_buffs_of(&mut self, team: usize, tables: Option<&DatTables>) {
        {
            let Some(user) = self.observer.selected_user(team) else {
                self.buffs[team].clear();
                return;
            };
            self.buffs[team] = user
                .skills
                .iter()
                .zip(&user.times)
                .take(broadcast::BUFF_MAX)
                .map(|(&skill, &time)| BuffCell {
                    skill,
                    icon: crate::ui2d::skill_icon(tables, skill, 1).unwrap_or_default(),
                    debuff: crate::ui2d::skill_is_debuff(tables, skill, 1),
                    alpha: 255,
                    blink_down: true,
                    time,
                })
                .collect();
        }
    }

    pub fn tick(&mut self, elapsed_ms: i32, desktop: &mut Desktop) {
        if self.active(desktop) {
            self.observer.tick(elapsed_ms);
            Self::set_rect(
                desktop,
                WindowId::BROADCAST_TOP,
                Rect::new(broadcast::TOP_MARGIN, self.observer.top.y, self.viewport.0 - 2 * broadcast::TOP_MARGIN, broadcast::TOP_HEIGHT),
            );
            self.blink_ms += elapsed_ms;
            while self.blink_ms >= BLINK_MS {
                self.blink_ms -= BLINK_MS;
                for cells in &mut self.buffs {
                    for cell in cells {
                        broadcast::blink_step(cell);
                    }
                }
            }
            self.second_ms += elapsed_ms;
            while self.second_ms >= 1000 {
                self.second_ms -= 1000;
                for cells in &mut self.buffs {
                    for cell in cells {
                        if cell.time > 0 {
                            cell.time -= 1;
                        }
                    }
                }
            }
        }
        for (left, id) in [(&mut self.gm_left, WindowId::BROADCAST_GM_MSG), (&mut self.special_left, WindowId::BROADCAST_SPECIAL_MSG)] {
            if let Some(remaining) = left {
                *remaining -= elapsed_ms;
                if *remaining <= 0 {
                    *left = None;
                    if let Some(window) = desktop.window_mut(id) {
                        window.visible = false;
                    }
                }
            }
        }
    }

    pub fn key(&mut self, key: InputKey, desktop: &mut Desktop) -> bool {
        if !self.active(desktop) {
            return false;
        }
        let InputKey::Char(character) = key else { return false };
        match character.to_ascii_lowercase() {
            'g' => self.observer.switch_msg_mode(),
            'i' => self.init_positions(desktop),
            'l' => self.observer.toggle_lock(),
            'm' => self.canvas_ops.push(CanvasOp::ToggleTargetName),
            'n' => self.observer.toggle_class_or_name(),
            'p' => {
                let visible = !Self::visible(desktop, PARTIES[0]);
                for id in PARTIES {
                    Self::show(desktop, id, visible);
                }
            }
            _ => return false,
        }
        true
    }

    pub fn select(&mut self, team: usize, index: usize, tables: Option<&DatTables>) {
        self.select_marks(team, index);
        self.refresh_buffs_of(team, tables);
    }

    pub fn hover_buff(&mut self, id: WindowId, x: i32, y: i32) -> Option<BuffCell> {
        let team = PLAYERS.iter().position(|player| *player == id)?;
        let index = broadcast::buff_hit(self.buffs[team].len(), team, broadcast::PLAYER_SIZE, self.icon_size[team], x, y);
        self.icon_size[team] = broadcast::BUFF_TOOLTIP_ICON_SIZE;
        self.buffs[team].get(index?).cloned()
    }

    pub fn mouse_down(&mut self, id: WindowId, x: i32, y: i32, tables: Option<&DatTables>) -> bool {
        let Some(team) = PARTIES.iter().position(|party| *party == id) else { return false };
        if let Some(PartyHit::Row(index)) = party_hit(&self.observer, team, x, y) {
            self.select(team, index, tables);
        }
        true
    }

    pub fn paint(
        &self,
        id: WindowId,
        list: &mut DrawList,
        origin: (i32, i32),
        font: &BitmapFont,
        lookup: &dyn Fn(u32) -> Option<String>,
    ) -> bool {
        let width = match id {
            WindowId::BROADCAST_TOP => self.viewport.0 - 2 * broadcast::TOP_MARGIN,
            WindowId::BROADCAST_MSG => self.viewport.0,
            WindowId::BROADCAST_GM_MSG => broadcast::GM_MSG_SIZE.0,
            WindowId::BROADCAST_SPECIAL_MSG => broadcast::SPECIAL_SIZE.0,
            _ => broadcast::PLAYER_SIZE.0,
        };
        let text_font = |font_type: i32| -> &dyn killer_ui::true_font::LineFont {
            match self.true_fonts.for_type(font_type) {
                Some(true_font) => true_font,
                None => font,
            }
        };
        match id {
            WindowId::BROADCAST_TOP => broadcast::paint_top(&self.observer, list, origin, width, text_font(broadcast::TOP_FONT_TYPE)),
            WindowId::BROADCAST_PLAYER_0 | WindowId::BROADCAST_PLAYER_1 => {
                let team = usize::from(id == WindowId::BROADCAST_PLAYER_1);
                let fonts = (text_font(broadcast::PLAYER_LEVEL_FONT_TYPE), text_font(broadcast::PLAYER_NAME_FONT_TYPE));
                broadcast::paint_player(&self.observer, team, list, origin, width, fonts, lookup);
                let (bx, by) = broadcast::buff_origin(team, broadcast::PLAYER_SIZE);
                broadcast::paint_buffs(list, (origin.0 + bx, origin.1 + by), &self.buffs[team]);
            }
            WindowId::BROADCAST_PARTY_0 | WindowId::BROADCAST_PARTY_1 => {
                let team = usize::from(id == WindowId::BROADCAST_PARTY_1);
                broadcast::paint_party(&self.observer, team, list, origin, text_font(broadcast::PARTY_FONT_TYPE), lookup);
            }
            WindowId::BROADCAST_MSG => broadcast::paint_msg(&self.observer, list, origin, width, text_font(broadcast::MSG_FONT_TYPE)),
            WindowId::BROADCAST_GM_MSG => broadcast::paint_gm_msg(list, origin, width, &self.gm_text, text_font(broadcast::GM_MSG_FONT_TYPE)),
            WindowId::BROADCAST_SPECIAL_MSG => broadcast::paint_special(list, origin, width, self.special_code),
            _ => return false,
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::window::Window;
    use killer_ui::window_owner::original_owner;

    fn setup() -> (BroadcastWindows, Desktop, OlympiadWindows) {
        let mut desktop = Desktop::new((1280, 800));
        let ids = [WindowId::BROADCAST_OBSERVER, WindowId::BROADCAST_GM_MSG, WindowId::BROADCAST_SPECIAL_MSG]
            .into_iter()
            .chain(CHILDREN)
            .chain([WindowId::INVENTORY]);
        for id in ids {
            let mut window = Window::new(id, Rect::new(0, 0, 100, 50), Rect::default());
            window.owner = original_owner(id);
            window.visible = id == WindowId::INVENTORY;
            desktop.windows.push(window);
        }
        (BroadcastWindows::new((1280, 800)), desktop, OlympiadWindows::new(|id| Some(format!("s{id}"))))
    }

    fn observer(flag: u8) -> WorldEvent {
        WorldEvent::EventMatchObserver(l2_net::game::EventMatchObserver {
            value: 7,
            flags: [flag, 0],
            names: ["Red".into(), "Blue".into()],
            values: [0; 3],
        })
    }

    fn visible(desktop: &Desktop, id: WindowId) -> bool {
        desktop.window(id).is_some_and(|window| window.visible)
    }

    #[test]
    fn the_observer_event_switches_to_state_one_showing_the_panel_and_hiding_the_rest() {
        let (mut windows, mut desktop, mut olympiad) = setup();
        windows.world_event(&observer(1), &mut desktop, &mut olympiad, None, None);
        assert_eq!(olympiad.state(), STATE_OBSERVER);
        assert!(visible(&desktop, WindowId::BROADCAST_OBSERVER));
        assert!(visible(&desktop, WindowId::BROADCAST_TOP));
        assert!(!visible(&desktop, WindowId::INVENTORY));
        assert_eq!(windows.observer.teams[0].name, "Red");
        windows.world_event(&observer(0), &mut desktop, &mut olympiad, None, None);
        assert_eq!(olympiad.state(), 0);
        assert!(!visible(&desktop, WindowId::BROADCAST_OBSERVER));
        assert!(visible(&desktop, WindowId::INVENTORY));
    }

    fn team_info(flag: i32, ids: &[i32]) -> WorldEvent {
        WorldEvent::EventMatchTeamInfo(l2_net::game::EventMatchTeamInfo {
            header: [7, flag, ids.len() as i32],
            entries: ids
                .iter()
                .map(|id| l2_net::game::EventMatchTeamEntry { id: *id, name: format!("u{id}"), values: [50, 100, 10, 20, 5, 6, 40, 2, 0, 0] })
                .collect(),
        })
    }

    #[test]
    fn team_info_fills_the_lists_only_in_state_one_and_resizes_them_by_row_count() {
        let (mut windows, mut desktop, mut olympiad) = setup();
        windows.world_event(&team_info(1, &[1, 2]), &mut desktop, &mut olympiad, None, None);
        assert_eq!(windows.observer.visible_rows(0), 0);
        windows.world_event(&observer(1), &mut desktop, &mut olympiad, None, None);
        windows.world_event(&team_info(1, &[1, 2, 3]), &mut desktop, &mut olympiad, None, None);
        assert_eq!(windows.observer.visible_rows(0), 3);
        assert_eq!(desktop.window(WindowId::BROADCAST_PARTY_0).unwrap().rect.h, 210);
        assert!(visible(&desktop, WindowId::BROADCAST_PARTY_0));
        assert!(!visible(&desktop, WindowId::BROADCAST_PARTY_1));
    }

    #[test]
    fn messages_show_the_gm_text_and_the_special_picture_and_hide_them_by_timer() {
        let (mut windows, mut desktop, mut olympiad) = setup();
        let message = |flag, text: &str| WorldEvent::EventMatchMessage(l2_net::game::EventMatchMessage { flag, message: text.into() });
        windows.world_event(&message(0, "Hello"), &mut desktop, &mut olympiad, None, None);
        windows.world_event(&message(2, ""), &mut desktop, &mut olympiad, None, None);
        assert!(visible(&desktop, WindowId::BROADCAST_GM_MSG) && visible(&desktop, WindowId::BROADCAST_SPECIAL_MSG));
        assert_eq!(windows.gm_text, "Hello");
        windows.tick(1000, &mut desktop);
        assert!(visible(&desktop, WindowId::BROADCAST_GM_MSG));
        assert!(!visible(&desktop, WindowId::BROADCAST_SPECIAL_MSG));
        windows.tick(4000, &mut desktop);
        assert!(!visible(&desktop, WindowId::BROADCAST_GM_MSG));
    }

    #[test]
    fn keys_only_act_while_the_panel_is_visible_and_p_toggles_both_lists() {
        let (mut windows, mut desktop, mut olympiad) = setup();
        assert!(!windows.key(InputKey::Char('g'), &mut desktop));
        windows.world_event(&observer(1), &mut desktop, &mut olympiad, None, None);
        windows.world_event(&team_info(1, &[1]), &mut desktop, &mut olympiad, None, None);
        assert!(windows.key(InputKey::Char('G'), &mut desktop));
        assert_eq!(windows.observer.msg_mode, broadcast::MsgMode::Teams);
        assert!(windows.key(InputKey::Char('n'), &mut desktop));
        assert!(!windows.observer.show_class_or_name);
        assert!(windows.key(InputKey::Char('p'), &mut desktop));
        assert!(!visible(&desktop, WindowId::BROADCAST_PARTY_0));
        assert!(!windows.key(InputKey::Char('z'), &mut desktop));
    }

    #[test]
    fn clicking_a_row_selects_that_user_for_the_player_panel() {
        let (mut windows, mut desktop, mut olympiad) = setup();
        windows.world_event(&observer(1), &mut desktop, &mut olympiad, None, None);
        windows.world_event(&team_info(2, &[11, 12]), &mut desktop, &mut olympiad, None, None);
        assert!(windows.mouse_down(WindowId::BROADCAST_PARTY_1, 20, 75, None));
        assert_eq!(windows.observer.selected[1], Some(1));
    }

    #[test]
    fn the_buff_hover_uses_the_constructor_icon_size_once_and_then_the_tooltip_one() {
        let (mut windows, _, _) = setup();
        let cell = |skill| BuffCell { skill, icon: String::new(), debuff: false, alpha: 255, blink_down: true, time: 5 };
        windows.buffs[0] = vec![cell(1), cell(2), cell(3)];
        let (bx, by) = broadcast::buff_origin(0, broadcast::PLAYER_SIZE);
        let x = bx + 12 + 25 * 2;
        assert_eq!(windows.hover_buff(PLAYERS[0], x, by).map(|cell| cell.skill), Some(3));
        assert_eq!(windows.hover_buff(PLAYERS[0], x, by).map(|cell| cell.skill), Some(2));
        assert!(windows.hover_buff(WindowId::CHAT, x, by).is_none());
    }

    fn drain(windows: &mut BroadcastWindows) -> Vec<AppendixOp> {
        std::mem::take(&mut windows.marks)
    }

    #[test]
    fn team_info_end_marks_each_listed_pawn_with_its_slot_and_the_team_circle() {
        let (mut windows, mut desktop, mut olympiad) = setup();
        windows.world_event(&observer(1), &mut desktop, &mut olympiad, None, None);
        drain(&mut windows);
        windows.world_event(&team_info(2, &[11, 12]), &mut desktop, &mut olympiad, None, None);
        assert_eq!(
            drain(&mut windows),
            vec![
                AppendixOp::Index { object_id: 11, team: 1, index: 0 },
                AppendixOp::Add { object_id: 11, bit: IDENTIFIER_BLUE },
                AppendixOp::Index { object_id: 12, team: 1, index: 1 },
                AppendixOp::Add { object_id: 12, bit: IDENTIFIER_BLUE },
            ]
        );
        assert_eq!(windows.observer.filling, None);
    }

    #[test]
    fn entering_and_leaving_state_one_toggles_the_controller_the_canvas_and_the_circles() {
        let (mut windows, mut desktop, mut olympiad) = setup();
        windows.observer.teams[0].users[0] = Some(killer_ui::broadcast::BroadcastUser { id: 5, ..Default::default() });
        windows.switch_state(STATE_OBSERVER, &mut desktop, &mut olympiad);
        assert_eq!(windows.controller, vec![true]);
        assert_eq!(windows.canvas_ops, vec![CanvasOp::Start]);
        assert_eq!(
            drain(&mut windows),
            vec![AppendixOp::Add { object_id: 5, bit: IDENTIFIER_RED }]
        );
        windows.switch_state(0, &mut desktop, &mut olympiad);
        assert_eq!(windows.controller, vec![true, false]);
        assert_eq!(windows.canvas_ops, vec![CanvasOp::Start, CanvasOp::Finish]);
        assert_eq!(drain(&mut windows), vec![AppendixOp::Flag { object_id: 5, set: false }, AppendixOp::DeleteAll { object_id: 5 }]);
    }

    #[test]
    fn selecting_a_row_moves_the_selected_circle_and_the_target_selects_the_matching_row() {
        let (mut windows, mut desktop, mut olympiad) = setup();
        windows.world_event(&observer(1), &mut desktop, &mut olympiad, None, None);
        windows.world_event(&team_info(1, &[21, 22, 23]), &mut desktop, &mut olympiad, None, None);
        drain(&mut windows);
        windows.select(0, 2, None);
        assert_eq!(
            drain(&mut windows),
            vec![
                AppendixOp::Flag { object_id: 23, set: true },
                AppendixOp::Add { object_id: 23, bit: SELECTED_RED },
            ]
        );
        let target = WorldEvent::MyTargetSelected(l2_net::game::MyTargetSelected { object_id: 22, color: 0 });
        windows.world_event(&target, &mut desktop, &mut olympiad, None, Some(22));
        assert_eq!(windows.observer.selected[0], Some(1));
        windows.world_event(&target, &mut desktop, &mut olympiad, None, Some(99));
        assert_eq!(windows.observer.selected[0], Some(1));
    }

    #[test]
    fn unlocking_a_team_clears_its_pawns_and_rows_and_a_char_info_marks_a_listed_user() {
        let (mut windows, mut desktop, mut olympiad) = setup();
        windows.world_event(&observer(1), &mut desktop, &mut olympiad, None, None);
        windows.world_event(&team_info(1, &[31, 32]), &mut desktop, &mut olympiad, None, None);
        drain(&mut windows);
        let unlocked = WorldEvent::EventMatchTeamUnlocked(l2_net::game::EventMatchTeamUnlocked { value: 7, flag: 1 });
        windows.world_event(&unlocked, &mut desktop, &mut olympiad, None, None);
        assert_eq!(windows.observer.visible_rows(0), 0);
        assert_eq!(
            drain(&mut windows),
            vec![
                AppendixOp::Flag { object_id: 31, set: false },
                AppendixOp::DeleteAll { object_id: 31 },
                AppendixOp::Flag { object_id: 32, set: false },
                AppendixOp::DeleteAll { object_id: 32 },
            ]
        );
        windows.world_event(&team_info(2, &[41]), &mut desktop, &mut olympiad, None, None);
        drain(&mut windows);
        windows.char_info_marks(41);
        assert_eq!(drain(&mut windows), vec![AppendixOp::Index { object_id: 41, team: 1, index: 0 }, AppendixOp::Add { object_id: 41, bit: IDENTIFIER_BLUE }]);
        windows.char_info_marks(500);
        assert!(drain(&mut windows).is_empty());
    }

    #[test]
    fn the_m_key_queues_the_canvas_target_name_toggle() {
        let (mut windows, mut desktop, mut olympiad) = setup();
        windows.world_event(&observer(1), &mut desktop, &mut olympiad, None, None);
        windows.canvas_ops.clear();
        assert!(windows.key(InputKey::Char('m'), &mut desktop));
        assert_eq!(windows.canvas_ops, vec![CanvasOp::ToggleTargetName]);
    }
    #[test]
    #[ignore]
    fn top_player_and_msg_use_the_truetype_slot_zero_and_render_headless() {
        use killer_assets::client::Client;
        use killer_assets::textures::{BankSource, TextureBank, write_png};
        use killer_render::camera::Camera;
        use killer_render::environment::Environment;
        use killer_render::headless::Headless;
        use killer_ui::draw::DrawList;
        use killer_ui::font::{BitmapFont, Glyphs};
        use killer_ui::true_font::{TrueFontTable, TrueFonts};
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let small = BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap());
        let table = TrueFontTable::from_ini(&killer_ui::ini::parse_ini(&client.read_system_text("ttfontinfo.ini").unwrap()));
        let fonts = TrueFonts::load(table, |name| client.read_system_file(name).ok());
        for font_type in [broadcast::TOP_FONT_TYPE, broadcast::PLAYER_LEVEL_FONT_TYPE, broadcast::MSG_FONT_TYPE] {
            assert_eq!(fonts.table.slot_for_type(font_type), 0);
            assert!(fonts.for_type(font_type).is_some());
        }
        let (mut windows, _, _) = setup();
        windows.true_fonts = fonts.clone();
        windows.observer.teams[0].name = "Red Team".into();
        windows.observer.teams[1].name = "Blue Team".into();
        let mut list = DrawList::default();
        assert!(windows.paint(WindowId::BROADCAST_TOP, &mut list, (0, 0), &small, &|_| None));
        let names: Vec<_> = list.quads.iter().filter(|quad| quad.texture.starts_with(killer_font::TEXTURE_PREFIX)).collect();
        assert_eq!(names.len(), 2);
        let font = fonts.for_type(broadcast::TOP_FONT_TYPE).unwrap();
        assert_eq!(names[0].h as i32, killer_ui::true_font::LineFont::height(font));
        assert_eq!(names[0].w as i32, killer_ui::true_font::LineFont::measure_text(font, "Red Team"));
        let mut headless = Headless::new(1280, 400).unwrap();
        let mut bank = TextureBank::new();
        headless.renderer_mut().set_ui(&list, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        assert_eq!(bank.missing().count(), 0);
        let camera = Camera { eye: glam::vec3(0.0, -300.0, 0.0), target: glam::Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
        let rgba = headless.render(&camera, &Environment::default()).unwrap();
        let quad = names[0];
        let (x0, y0, w, h) = (quad.x as usize, quad.y as usize, quad.w as usize, quad.h as usize);
        let red = (0..h).flat_map(|row| (0..w).map(move |col| (row, col))).filter(|&(row, col)| {
            let at = ((y0 + row) * 1280 + x0 + col) * 4;
            rgba[at] > 200 && rgba[at + 1] < 80 && rgba[at + 2] < 80
        }).count();
        assert!(red > 50, "pixels vermelhos do texto: {red}");
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-broadcast-truefont.png");
        write_png(&path, 1280, 400, &rgba).unwrap();
    }

}
