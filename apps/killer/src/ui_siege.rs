use std::collections::HashMap;

use killer_assets::client::DatTables;
use killer_ui::siege::{self, SiegeAction, SiegeClan, SiegeInfo, SiegeRequest, SiegeTab, SiegeView};
use killer_ui::window::{Desktop, WindowId};
use killer_world::{Dialog, NetCommand, World, WorldEvent};
use killer_world::state::SiegeDecision;

pub const WINDOW_ID: WindowId = WindowId(62);

pub struct SiegeWindow {
    pub view: SiegeView,
    castles: HashMap<i32, String>,
    time_template: String,
}

impl SiegeWindow {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self { view: SiegeView::new(&lookup(siege::TITLE_SYSSTRING).unwrap_or_default(), |id| lookup(id).unwrap_or_default(), crate::ui_keys::DEFAULT_LINE_HEIGHT), castles: HashMap::new(), time_template: String::new() }
    }

    pub fn set_tables(&mut self, tables: &DatTables) {
        self.view.frame.title = tables.sysstring.get(&siege::TITLE_SYSSTRING).cloned().unwrap_or_default();
        self.view.labels = siege::SYSSTRINGS.into_iter().filter_map(|id| tables.sysstring.get(&id).cloned().map(|text| (id, text))).collect();
        self.castles = tables.castlename.values().filter(|castle| castle.present != 0 && castle.index < 100).map(|castle| (castle.id as i32, castle.name.clone())).collect();
        self.time_template = tables.systemmsg.get(&713).map(|message| message.text.clone()).unwrap_or_default();
    }

    pub fn world_event(&mut self, event: &WorldEvent, world: &World, desktop: &mut Desktop) {
        match event {
            WorldEvent::CastleSiegeInfo(info) => {
                self.view.on_show();
                if let Some(window) = desktop.window_mut(WINDOW_ID) { window.visible = true; }
                desktop.bring_to_front(WINDOW_ID);
                let template = &self.time_template;
                self.view.receive_info(SiegeInfo {
                    castle_id: info.castle_id, owner_flags: info.owner_flags, owner_id: info.owner_id,
                    leader_name: info.leader_name.clone(), pledge_name: info.pledge_name.clone(),
                    ally_id: info.ally_id, ally_name: info.ally_name.clone(), now: info.now,
                    siege_time: info.siege_time, choices: info.choices.clone(),
                }, self.castles.get(&info.castle_id).cloned().unwrap_or_default(), |seconds| format_time(template, seconds));
                if let Some(window) = desktop.window_mut(WINDOW_ID) { window.visible = true; }
            }
            WorldEvent::CastleSiegeAttackerList(list) | WorldEvent::CastleSiegeDefenderList(list) => {
                let tab = if matches!(event, WorldEvent::CastleSiegeAttackerList(_)) { SiegeTab::Attackers } else { SiegeTab::Defenders };
                let rows: Vec<_> = list.clans.iter().map(|clan| SiegeClan {
                    pledge_id: clan.pledge_id, name: clan.name.clone(), leader_name: clan.leader_name.clone(),
                    ally_name: clan.ally_name.clone(), ally_leader_name: clan.ally_leader_name.clone(), kind: clan.kind,
                    crest_texture: (clan.crest_id > 0).then(|| crate::ui2d::crest_texture(clan.crest_id)),
                    ally_crest_texture: (clan.ally_crest_id > 0).then(|| crate::ui2d::ally_crest_texture(clan.ally_crest_id)),
                }).collect();
                self.view.receive_list(tab, list.castle_id, &rows, world.social.clan_id);
            }
            _ => {}
        }
        if let Some(owner) = world.pledges.get(&self.view.info.owner_id) {
            self.view.owner_crest = (owner.crest_id > 0).then(|| crate::ui2d::crest_texture(owner.crest_id));
            self.view.owner_ally_crest = (owner.ally_crest_id > 0).then(|| crate::ui2d::ally_crest_texture(owner.ally_crest_id));
        }
    }

    pub fn confirm_time(&self) -> Option<NetCommand> {
        let time = if self.view.date.selected <= 0 { 0 } else {
            *self.view.info.choices.get((self.view.date.selected - 1) as usize)?
        };
        Some(NetCommand::SetCastleSiegeTime { castle_id: self.view.info.castle_id, time })
    }

    pub fn action(&mut self, action: SiegeAction, desktop: &mut Desktop) -> (Option<NetCommand>, Option<Dialog>) {
        match action {
            SiegeAction::Request(SiegeRequest::Attackers { castle_id }) => (Some(NetCommand::CastleSiegeAttackerList { castle_id }), None),
            SiegeAction::Request(SiegeRequest::Defenders { castle_id }) => (Some(NetCommand::CastleSiegeDefenderList { castle_id }), None),
            SiegeAction::Request(_) => (None, None),
            SiegeAction::Confirm(confirmation) => {
                let dialog_type = confirmation.dialog_type();
                let decision = match confirmation.request {
                    SiegeRequest::Join { castle_id, attacker, join } => SiegeDecision::Join { castle_id, attacker, join },
                    SiegeRequest::Approve { castle_id, pledge_id, approved } => SiegeDecision::Approve { castle_id, pledge_id, approved },
                    SiegeRequest::SetTime { castle_id, time } => SiegeDecision::SetTime { castle_id, time },
                    _ => return (None, None),
                };
                (None, Some(Dialog::Siege { message_id: confirmation.system_message, dialog_type, parameter: confirmation.text_parameter, decision }))
            }
            SiegeAction::Close => {
                if let Some(window) = desktop.window_mut(WINDOW_ID) { window.visible = false; }
                self.view.on_hide();
                (None, None)
            }
        }
    }
}

fn format_time(template: &str, seconds: i32) -> String {
    let Some(date) = killer_platform::calendar::from_seconds(seconds) else { return String::new(); };
    format_date(template, date)
}

fn format_date(template: &str, date: killer_platform::calendar::LocalDate) -> String {
    let values = [date.year.to_string(), format!("{:02}", date.month), format!("{:02}", date.day), format!("{:02}", date.hour), format!("{:02}", date.minute)];
    let mut text = template.to_owned();
    for (index, value) in values.iter().enumerate() {
        for letter in ['s', 'c'] { text = text.replace(&format!("${letter}{}", index + 1), value); }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_uses_catalog_order_and_native_padding() {
        let date = killer_platform::calendar::LocalDate { year: 2006, month: 2, day: 3, hour: 4, minute: 5 };
        assert_eq!(format_date("$s3/$s2/$s1 $s4:$s5", date), "03/02/2006 04:05");
    }
}

#[cfg(test)]
mod confirmation_tests {
    use super::*;
    use killer_ui::siege::SiegeConfirmation;

    #[test]
    fn all_seven_confirmations_keep_native_dialog_and_only_send_after_acceptance() {
        let requests = [
            SiegeRequest::SetTime { castle_id: 7, time: 1_140_086_400 },
            SiegeRequest::Join { castle_id: 7, attacker: true, join: true },
            SiegeRequest::Join { castle_id: 7, attacker: true, join: false },
            SiegeRequest::Join { castle_id: 7, attacker: false, join: true },
            SiegeRequest::Join { castle_id: 7, attacker: false, join: false },
            SiegeRequest::Approve { castle_id: 7, pledge_id: 13, approved: true },
            SiegeRequest::Approve { castle_id: 7, pledge_id: 13, approved: false },
        ];
        for (index, request) in requests.into_iter().enumerate() {
            let mut siege = SiegeWindow::new(|_| None);
            let mut desktop = Desktop::new((1024, 768));
            let (command, dialog) = siege.action(SiegeAction::Confirm(SiegeConfirmation {
                system_message: 1, text_parameter: "Cla".into(), request: request.clone(),
            }), &mut desktop);
            assert!(command.is_none());
            let (view, answer, window) = crate::ui2d::dialog_request(&dialog.unwrap(), &World::default(), None);
            assert_eq!(view.kind, killer_ui::dialog::DialogKind::OkCancel);
            assert_eq!(view.dialog_type, 61 + index as i32);
            assert_eq!(window, WindowId::DIALOG);
            assert!(crate::ui2d::dialog_command(answer.clone(), false).is_none());
            let command = crate::ui2d::dialog_command(answer, true).unwrap();
            match (request, command) {
                (SiegeRequest::SetTime { castle_id, time }, NetCommand::SetCastleSiegeTime { castle_id: actual_id, time: actual_time }) => assert_eq!((castle_id, time), (actual_id, actual_time)),
                (SiegeRequest::Join { castle_id, attacker, join }, NetCommand::JoinCastleSiege { castle_id: actual_id, attacker: actual_attacker, join: actual_join }) => assert_eq!((castle_id, attacker, join), (actual_id, actual_attacker, actual_join)),
                (SiegeRequest::Approve { castle_id, pledge_id, approved }, NetCommand::ConfirmCastleSiegeWaitingList { castle_id: actual_id, pledge_id: actual_pledge, approved: actual_approved }) => assert_eq!((castle_id, pledge_id, approved), (actual_id, actual_pledge, actual_approved)),
                _ => panic!("wrong siege command"),
            }
        }
    }
}
