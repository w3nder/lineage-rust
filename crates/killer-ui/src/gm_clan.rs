use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::frame_contract::{FrameContract, FrameHit, FrameTitle};
use crate::pledge::{MemberRow, PledgeHit, PledgeView};
use crate::window::{Rect, WindowId};

pub const ID: WindowId = WindowId::GM_CLAN;
pub const SIZE: (i32, i32) = (256, 355);
pub const FRAME_ID: u32 = 0xf0f018;
pub const GRIP: Rect = Rect::new(0, 0, 256, 20);
pub const PAGE_ORIGIN: (i32, i32) = (0, 20);
pub const INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 1, vertical_mode: 3, x: 209, y: 100 };

const FRAME: FrameContract = FrameContract {
    rect: GRIP,
    id: FRAME_ID,
    mode: FrameMode::Title,
    title: FrameTitle::None,
    close_button: true,
    minimize_button: false,
};

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 as f32 - 209.0) as i32, 100)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GmPledgeInfo {
    pub clan_id: i32,
    pub name: String,
    pub leader: String,
    pub level: i32,
    pub castle: bool,
    pub hideout: bool,
}

#[derive(Clone, Debug)]
pub struct GmClanView {
    pub frame: FrameCtrl,
    pub pledge: PledgeView,
    pub clan_id: i32,
    pressed_close: bool,
}

impl GmClanView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let mut pledge = PledgeView::with_sysstrings(&lookup);
        pledge.gm = true;
        pledge.name = Some(String::new());
        Self { frame: FRAME.build(|id| lookup(id).unwrap_or_default()), pledge, clan_id: 0, pressed_close: false }
    }

    pub fn apply_info(&mut self, info: &GmPledgeInfo) {
        self.clan_id = info.clan_id;
        if !info.name.is_empty() {
            self.pledge.name = Some(info.name.clone());
        }
        if !info.leader.is_empty() {
            self.pledge.leader = info.leader.clone();
        }
        self.pledge.level = info.level;
        self.pledge.gm_castle = info.castle;
        self.pledge.gm_hideout = info.hideout;
    }

    pub fn clear_members(&mut self) {
        self.pledge.members.clear();
        self.pledge.selected = None;
        self.pledge.scroll_row = 0;
        self.pledge.clamp_scroll();
    }

    pub fn update_member(&mut self, row: MemberRow) {
        match self.pledge.members.iter_mut().find(|member| member.name == row.name) {
            Some(existing) => *existing = row,
            None => self.pledge.members.push(row),
        }
        self.pledge.clamp_scroll();
    }

    pub fn selected_member(&self) -> Option<&MemberRow> {
        self.pledge.selected.and_then(|index| self.pledge.members.get(index))
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<crate::pledge::PledgeButton> {
        if FRAME.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            self.pressed_close = true;
            return None;
        }
        let (px, py) = (x - PAGE_ORIGIN.0, y - PAGE_ORIGIN.1);
        if self.pledge.scroll_down(px, py) {
            return None;
        }
        match self.pledge.hit(px, py) {
            Some(PledgeHit::Button(button)) => {
                self.pledge.pressed = Some(button);
                None
            }
            Some(PledgeHit::Row(row)) => {
                self.pledge.selected = Some(row);
                None
            }
            _ => None,
        }
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> GmClanRelease {
        let (px, py) = (x - PAGE_ORIGIN.0, y - PAGE_ORIGIN.1);
        self.pledge.scroll_up(Some((px, py)));
        let pressed = self.pledge.pressed.take();
        if std::mem::take(&mut self.pressed_close) && FRAME.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return GmClanRelease::Close;
        }
        match (pressed, self.pledge.hit(px, py)) {
            (Some(button), Some(PledgeHit::Button(over))) if button == over => GmClanRelease::Button(button),
            _ => GmClanRelease::None,
        }
    }

    pub fn mouse_move(&mut self, y: i32) -> bool {
        self.pledge.scroll_move(y - PAGE_ORIGIN.1)
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        if !crate::pledge::LIST_RECT.contains(x - PAGE_ORIGIN.0, y - PAGE_ORIGIN.1) {
            return false;
        }
        self.pledge.wheel(delta);
        true
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        FRAME.paint(&self.frame, list, origin, font, alpha);
        self.pledge.paint(list, origin.0 + PAGE_ORIGIN.0, origin.1 + PAGE_ORIGIN.1, font);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GmClanRelease {
    None,
    Close,
    Button(crate::pledge::PledgeButton),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pledge::PledgeButton;

    fn view() -> GmClanView {
        GmClanView::new(|id| Some(format!("s{id}")))
    }

    fn row(name: &str, level: i32, object_id: i32) -> MemberRow {
        MemberRow { name: name.into(), level, class_id: 1, online: object_id > 0, object_id }
    }

    #[test]
    fn ten_buttons_are_disabled_and_authorize_is_relabeled_view_rights() {
        let view = view();
        let buttons = view.pledge.buttons();
        let enabled: Vec<PledgeButton> =
            crate::pledge::BUTTONS.iter().zip(&buttons).filter(|(_, button)| button.enabled).map(|(kind, _)| *kind).collect();
        assert_eq!(enabled.len(), 5);
        let authorize = crate::pledge::BUTTONS.iter().position(|b| *b == PledgeButton::Authorize).unwrap();
        assert_eq!(buttons[authorize].label, "s883");
        let invite = crate::pledge::button_rect(0);
        assert_eq!(view.pledge.hit(invite.x + 1, invite.y + 1), Some(PledgeHit::Page));
    }

    #[test]
    fn a_packet_without_names_keeps_the_previous_clan_name_and_leader() {
        let mut view = view();
        view.apply_info(&GmPledgeInfo { clan_id: 5, name: "Canastra".into(), leader: "Ana".into(), level: 3, castle: true, hideout: false });
        view.apply_info(&GmPledgeInfo { clan_id: 0, level: 0, ..Default::default() });
        assert_eq!((view.pledge.name.as_deref(), view.pledge.leader.as_str(), view.pledge.level), (Some("Canastra"), "Ana", 0));
    }

    #[test]
    fn members_update_by_name_instead_of_duplicating() {
        let mut view = view();
        view.update_member(row("Bo", 10, 0));
        view.update_member(row("Bo", 11, 77));
        view.update_member(row("Cy", 5, 0));
        assert_eq!(view.pledge.members.len(), 2);
        assert_eq!((view.pledge.members[0].level, view.pledge.members[0].object_id), (11, 77));
    }
}
