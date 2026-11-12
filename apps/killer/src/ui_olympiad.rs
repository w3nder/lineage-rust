use killer_ui::abnormal::AbnormalView;
use killer_ui::olympiad::{
    ABNORMAL_IDS, ABNORMAL_SHIFT, CONTROL_ID, ControlAction, OlympiadControl, OlympiadPlayer, OlympiadTarget, OlympiadUser,
    OlympiadUserInfoView, PLAYER_IDS, SIDE_FIRST, SIDE_SECOND, TARGET_ID,
};
use killer_ui::window::{Desktop, WindowId};
use killer_ui::window_owner::WindowParent;
use killer_world::{NetCommand, WorldEvent};

use crate::ui_keys::UiCommand;

pub const STATE_NORMAL: i32 = 0;
pub const STATE_SPECTATOR: i32 = 2;
pub const STATES: i32 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkOp {
    SaveZero,
    Restore,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConsoleMark {
    pub saved: Option<i32>,
}

impl ConsoleMark {
    pub fn apply(&mut self, op: MarkOp, value: &mut i32) {
        match op {
            MarkOp::SaveZero => {
                self.saved = Some(*value);
                *value = 0;
            }
            MarkOp::Restore => {
                if let Some(saved) = self.saved.take() {
                    *value = saved;
                }
            }
        }
    }
}

pub struct OlympiadWindows {
    pub mark_ops: Vec<MarkOp>,
    pub canvas_ops: Vec<killer_world::appendix::CanvasOp>,
    pub control: OlympiadControl,
    pub players: [OlympiadPlayer; 2],
    pub target: OlympiadTarget,
    pub abnormal: [AbnormalView; 2],
    pub commands: Vec<UiCommand>,
    state: i32,
    lists: [Vec<WindowId>; STATES as usize],
}

impl OlympiadWindows {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self {
            control: OlympiadControl::new(lookup),
            players: [OlympiadPlayer::default(), OlympiadPlayer::default()],
            target: OlympiadTarget::default(),
            abnormal: [AbnormalView::default(), AbnormalView::default()],
            commands: Vec::new(),
            mark_ops: Vec::new(),
            canvas_ops: Vec::new(),
            state: STATE_NORMAL,
            lists: Default::default(),
        }
    }

    pub fn state(&self) -> i32 {
        self.state
    }

    pub fn set_state(&mut self, new: i32, desktop: &mut Desktop) {
        if new == self.state || !(0..STATES).contains(&new) {
            return;
        }
        let shown: Vec<WindowId> = desktop
            .windows
            .iter()
            .filter(|window| window.visible && window.owner.is_some_and(|owner| owner.parent == WindowParent::PrimaryRoot))
            .map(|window| window.id)
            .collect();
        self.lists[self.state as usize] = shown.clone();
        for id in shown {
            show(desktop, id, false);
        }
        for id in self.lists[new as usize].clone() {
            show(desktop, id, true);
        }
        self.state = new;
        self.canvas_ops.push(if new == 1 { killer_world::appendix::CanvasOp::Start } else { killer_world::appendix::CanvasOp::Finish });
        self.mark_ops.push(if new == STATE_NORMAL { MarkOp::Restore } else { MarkOp::SaveZero });
        if new == STATE_SPECTATOR {
            for id in [CONTROL_ID, PLAYER_IDS[0], PLAYER_IDS[1], ABNORMAL_IDS[0], ABNORMAL_IDS[1], WindowId::CHAT] {
                show(desktop, id, true);
            }
        }
    }

    pub fn world_event(&mut self, event: &WorldEvent, desktop: &mut Desktop) {
        match event {
            WorldEvent::OlympiadUserInfo(info) => {
                let view = OlympiadUserInfoView {
                    side: i32::from(info.flag),
                    user: OlympiadUser {
                        object_id: info.value,
                        name: info.name.clone(),
                        hp: info.values[1],
                        max_hp: info.values[2],
                        cp: info.values[3],
                        max_cp: info.values[4],
                    },
                };
                if self.state == STATE_SPECTATOR {
                    match view.side {
                        SIDE_FIRST => self.players[0].set_user_info(&view),
                        SIDE_SECOND => self.players[1].set_user_info(&view),
                        _ => {}
                    }
                } else if self.target.side == view.side {
                    self.target.set_user_info(&view);
                }
            }
            WorldEvent::OlympiadMode(mode) => match mode.mode {
                0 => {
                    if self.state != STATE_SPECTATOR {
                        show(desktop, TARGET_ID, false);
                    } else {
                        self.set_state(STATE_NORMAL, desktop);
                    }
                }
                1 => {
                    show(desktop, TARGET_ID, true);
                    self.target.set_side(SIDE_SECOND);
                }
                2 => {
                    show(desktop, TARGET_ID, true);
                    self.target.set_side(SIDE_FIRST);
                }
                3 => self.set_state(STATE_SPECTATOR, desktop),
                _ => {}
            },
            WorldEvent::OlympiadMatchEnd => {
                for player in &mut self.players {
                    player.initialize();
                }
                self.abnormal = [AbnormalView::default(), AbnormalView::default()];
                self.target.initialize();
            }
            _ => {}
        }
    }

    pub fn spelled_target(&self, object_id: i32) -> Option<usize> {
        if self.players[0].object_id() == object_id {
            Some(0)
        } else if self.players[1].object_id() == object_id {
            Some(1)
        } else {
            None
        }
    }

    pub fn mouse_down(&mut self, id: WindowId, x: i32, y: i32) -> bool {
        if id == CONTROL_ID {
            self.control.mouse_down(x, y);
            return true;
        }
        if let Some(index) = PLAYER_IDS.iter().position(|player| *player == id) {
            self.players[index].mouse_down(x, y);
            return true;
        }
        false
    }

    pub fn mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        let outside = (i32::MIN / 2, i32::MIN / 2);
        let at = |id: WindowId| over.filter(|(over_id, _, _)| *over_id == id).map_or(outside, |(_, x, y)| (x, y));
        let (x, y) = at(CONTROL_ID);
        match self.control.mouse_up(x, y) {
            Some(ControlAction::Return) => self.commands.push(UiCommand::Net(NetCommand::OlympiadObserverEnd)),
            Some(ControlAction::MatchList) => self.commands.push(UiCommand::Net(NetCommand::OlympiadMatchList)),
            None => {}
        }
        for index in 0..2 {
            let (x, y) = at(PLAYER_IDS[index]);
            if let Some(expanded) = self.players[index].mouse_up(x, y) {
                let height = self.players[index].height();
                if let Some(window) = desktop.window_mut(PLAYER_IDS[index]) {
                    window.rect.h = height;
                }
                if let Some(window) = desktop.window_mut(ABNORMAL_IDS[index]) {
                    window.rect.y += if expanded { ABNORMAL_SHIFT } else { -ABNORMAL_SHIFT };
                }
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
    use killer_ui::window::{Rect, Window};
    use killer_ui::window_owner::original_owner;

    fn setup() -> (OlympiadWindows, Desktop) {
        let mut desktop = Desktop::new((1280, 800));
        for id in [CONTROL_ID, PLAYER_IDS[0], PLAYER_IDS[1], ABNORMAL_IDS[0], ABNORMAL_IDS[1], TARGET_ID, WindowId::CHAT, WindowId::INVENTORY] {
            let mut window = Window::new(id, Rect::new(0, 47, 100, 50), Rect::default());
            window.owner = original_owner(id);
            window.visible = id == WindowId::INVENTORY;
            desktop.windows.push(window);
        }
        (OlympiadWindows::new(|id| Some(format!("s{id}"))), desktop)
    }

    fn visible(desktop: &Desktop, id: WindowId) -> bool {
        desktop.window(id).is_some_and(|window| window.visible)
    }

    fn mode(value: u8) -> WorldEvent {
        WorldEvent::OlympiadMode(l2_net::game::OlympiadMode { mode: value })
    }

    fn user_info(flag: u8, object_id: i32, name: &str) -> WorldEvent {
        WorldEvent::OlympiadUserInfo(l2_net::game::OlympiadUserInfo {
            flag,
            value: object_id,
            name: name.into(),
            values: [0, 10, 20, 3, 4],
        })
    }

    #[test]
    fn the_console_mark_saves_overwrites_and_restores_like_the_original_machine() {
        let mut mark = ConsoleMark::default();
        let mut value = 2;
        mark.apply(MarkOp::SaveZero, &mut value);
        assert_eq!((value, mark.saved), (0, Some(2)));
        mark.apply(MarkOp::SaveZero, &mut value);
        assert_eq!(mark.saved, Some(0));
        mark.apply(MarkOp::Restore, &mut value);
        assert_eq!((value, mark.saved), (0, None));
        mark.apply(MarkOp::Restore, &mut value);
        assert_eq!(value, 0);
    }

    #[test]
    fn state_changes_queue_the_mark_operations() {
        let (mut windows, mut desktop) = setup();
        windows.world_event(&mode(3), &mut desktop);
        windows.world_event(&mode(0), &mut desktop);
        assert_eq!(windows.mark_ops, vec![MarkOp::SaveZero, MarkOp::Restore]);
    }

    #[test]
    fn mode_3_enters_the_spectator_state_showing_the_seven_windows_and_hiding_the_rest() {
        let (mut windows, mut desktop) = setup();
        windows.world_event(&mode(3), &mut desktop);
        assert_eq!(windows.state(), STATE_SPECTATOR);
        for id in [CONTROL_ID, PLAYER_IDS[0], PLAYER_IDS[1], ABNORMAL_IDS[0], ABNORMAL_IDS[1], WindowId::CHAT] {
            assert!(visible(&desktop, id));
        }
        assert!(!visible(&desktop, WindowId::INVENTORY));
        windows.world_event(&mode(0), &mut desktop);
        assert_eq!(windows.state(), STATE_NORMAL);
        assert!(visible(&desktop, WindowId::INVENTORY));
        assert!(!visible(&desktop, CONTROL_ID));
    }

    #[test]
    fn user_info_goes_to_the_player_by_side_while_spectating() {
        let (mut windows, mut desktop) = setup();
        windows.world_event(&mode(3), &mut desktop);
        windows.world_event(&user_info(1, 11, "Ana"), &mut desktop);
        windows.world_event(&user_info(2, 22, "Bo"), &mut desktop);
        assert_eq!((windows.players[0].user.name.as_str(), windows.players[1].user.name.as_str()), ("Ana", "Bo"));
        assert_eq!(windows.players[0].user.max_hp, 20);
        assert_eq!(windows.spelled_target(22), Some(1));
        assert_eq!(windows.spelled_target(33), None);
    }

    #[test]
    fn a_fighter_sees_only_the_opponent_side_in_the_target_window() {
        let (mut windows, mut desktop) = setup();
        windows.world_event(&mode(2), &mut desktop);
        assert!(visible(&desktop, TARGET_ID));
        assert_eq!(windows.target.side, SIDE_FIRST);
        windows.world_event(&user_info(2, 5, "x"), &mut desktop);
        assert_eq!(windows.target.user.name, "");
        windows.world_event(&user_info(1, 5, "Cy"), &mut desktop);
        assert_eq!(windows.target.user.name, "Cy");
        windows.world_event(&mode(0), &mut desktop);
        assert!(!visible(&desktop, TARGET_ID));
    }

    #[test]
    fn match_end_resets_players_target_and_buffs_without_changing_state() {
        let (mut windows, mut desktop) = setup();
        windows.world_event(&mode(3), &mut desktop);
        windows.world_event(&user_info(1, 11, "Ana"), &mut desktop);
        windows.players[0].add_message("m");
        windows.world_event(&WorldEvent::OlympiadMatchEnd, &mut desktop);
        assert_eq!(windows.players[0].object_id(), -1);
        assert_eq!(windows.state(), STATE_SPECTATOR);
    }

    #[test]
    fn expanding_a_player_resizes_it_and_moves_its_buff_window_by_80() {
        let (mut windows, mut desktop) = setup();
        windows.mouse_down(PLAYER_IDS[0], 3, 8);
        windows.mouse_up(Some((PLAYER_IDS[0], 3, 8)), &mut desktop);
        assert_eq!(desktop.window(PLAYER_IDS[0]).unwrap().rect.h, 126);
        assert_eq!(desktop.window(ABNORMAL_IDS[0]).unwrap().rect.y, 127);
        windows.mouse_down(PLAYER_IDS[0], 3, 8);
        windows.mouse_up(Some((PLAYER_IDS[0], 3, 8)), &mut desktop);
        assert_eq!(desktop.window(PLAYER_IDS[0]).unwrap().rect.h, 46);
        assert_eq!(desktop.window(ABNORMAL_IDS[0]).unwrap().rect.y, 47);
    }

    #[test]
    fn the_control_buttons_send_the_extended_requests() {
        let (mut windows, mut desktop) = setup();
        let first = windows.control.buttons[0].rect;
        windows.mouse_down(CONTROL_ID, first.x + 2, first.y + 2);
        windows.mouse_up(Some((CONTROL_ID, first.x + 2, first.y + 2)), &mut desktop);
        assert!(matches!(windows.commands.as_slice(), [UiCommand::Net(NetCommand::OlympiadObserverEnd)]));
    }
}
