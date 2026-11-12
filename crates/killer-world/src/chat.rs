use std::collections::VecDeque;

use l2_net::game::{Say2Body, SystemMessageArg};

use crate::world::WorldEvent;

pub const CHAT_HISTORY: usize = 200;
pub const CHAT_SYSTEM: i32 = -1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatLine {
    pub kind: i32,
    pub sender: String,
    pub text: String,
    pub group: u32,
    pub color: Option<u32>,
    pub system: Option<(i32, Vec<SystemMessageArg>)>,
    pub tables: Option<(i32, i32)>,
}

impl ChatLine {
    pub fn new(kind: i32, sender: impl Into<String>, text: impl Into<String>) -> ChatLine {
        ChatLine { kind, sender: sender.into(), text: text.into(), group: 0, color: None, system: None, tables: None }
    }

    pub fn from_event(event: &WorldEvent) -> Option<ChatLine> {
        match event {
            WorldEvent::Say2(s) => Some(match &s.body {
                Say2Body::Text { name, text } => ChatLine::new(s.kind, name.clone(), text.clone()),
                Say2Body::Tables { sys_string_id, sys_msg_id } => {
                    let mut line = ChatLine::new(s.kind, format!("sysstring {sys_string_id}"), format!("sysmsg {sys_msg_id}"));
                    line.tables = Some((*sys_string_id, *sys_msg_id));
                    line
                }
            }),
            WorldEvent::NpcSay(n) => Some(ChatLine::new(n.kind, format!("npc {}", n.npc_id), n.text.clone())),
            WorldEvent::SystemMessage(m) => {
                let mut text = format!("#{}", m.id);
                for arg in &m.args {
                    text.push(' ');
                    text.push_str(&describe(arg));
                }
                let mut line = ChatLine::new(CHAT_SYSTEM, String::new(), text);
                line.system = Some((m.id, m.args.clone()));
                Some(line)
            }
            _ => None,
        }
    }
}

pub const NOTICE_COLOR: u32 = 0xffb0_9b79;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Notice {
    pub id: i32,
    pub color: Option<u32>,
}

impl Notice {
    const fn table(id: i32) -> Option<Notice> {
        Some(Notice { id, color: None })
    }

    const fn colored(id: i32) -> Option<Notice> {
        Some(Notice { id, color: Some(NOTICE_COLOR) })
    }
}

pub fn notice_of(event: &WorldEvent) -> Option<Notice> {
    let pick = |value: i32, yes: i32, no: i32| if value != 0 { yes } else { no };
    match event {
        WorldEvent::CreatePledge(r) => Notice::table(pick(r.pledge_id, 0xbd, 0xbe)),
        WorldEvent::OustPledgeMember(r) => Notice::table(pick(r.pledge_id, 0x135, 0x136)),
        WorldEvent::WithdrawalPledge(r) => Notice::table(pick(r.pledge_id, 0xc5, 0x460)),
        WorldEvent::JoinPledge(r) if r.pledge_id == 0 => Notice::table(0x134),
        WorldEvent::DismissPledge(r) => Notice::table(pick(r.pledge_id, 0xc1, 0xc2)),
        WorldEvent::SetOustPledgeMemberNoop(_) | WorldEvent::SetDismissPledgeNoop(_) => Notice::table(0xc7),
        WorldEvent::JoinParty(r) if r.response == 0 => Notice::table(0x131),
        WorldEvent::DismissParty(r) => Notice::colored(pick(r.member_id, 0xcb, 0x13e)),
        WorldEvent::SetDismissPartyNoop(_) => Notice::table(0xcb),
        WorldEvent::WithdrawalParty(r) => Notice::colored(pick(r.member_id, 0xc8, 0x13c)),
        WorldEvent::OustPartyMember(r) if r.member_id == 0 => Notice::table(0x13d),
        WorldEvent::SetOustPartyMemberNoop(_) => Notice::table(0xca),
        WorldEvent::JoinAllianceResult(r) => Notice::table(pick(r.alliance_id, 0x205, 0x206)),
        WorldEvent::WithdrawAllianceResult(r) => Notice::colored(pick(r.alliance_id, 0x207, 0x208)),
        WorldEvent::OustAllianceMemberPledgeResult(r) => Notice::colored(pick(r.alliance_id, 0x209, 0x20a)),
        WorldEvent::DismissAllianceResult(r) => Notice::colored(pick(r.alliance_id, 0x20b, 0x20c)),
        WorldEvent::FriendAddRequestResult(r) => Notice::table(pick(r.result, 0x20d, 0x20e)),
        WorldEvent::WarehouseDone(r) if !r.success => Notice::table(0x133),
        _ => None,
    }
}

fn describe(arg: &SystemMessageArg) -> String {
    match arg {
        SystemMessageArg::Text(text) => text.clone(),
        SystemMessageArg::Number(n) => n.to_string(),
        SystemMessageArg::NpcName(id) => format!("npc {id}"),
        SystemMessageArg::ItemName(id) => format!("item {id}"),
        SystemMessageArg::SkillName { id, level } => format!("skill {id} level {level}"),
        SystemMessageArg::CastleName(id) => format!("castle {id}"),
        SystemMessageArg::Unknown { kind } => format!("?{kind}"),
    }
}

#[derive(Debug, Default)]
pub struct ChatLog {
    lines: VecDeque<ChatLine>,
    pushed: u64,
}

impl ChatLog {
    pub fn push(&mut self, line: ChatLine) {
        if self.lines.len() == CHAT_HISTORY {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
        self.pushed += 1;
    }

    pub fn pushed(&self) -> u64 {
        self.pushed
    }

    pub fn apply(&mut self, event: &WorldEvent) {
        if let Some(line) = ChatLine::from_event(event) {
            self.push(line);
        }
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ChatLine> {
        self.lines.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::WorldEvent;
    use l2_net::game::packets::{CHAT_ALL, CHAT_BOAT, CHAT_SHOUT};
    use l2_net::game::{NpcSay, Say2, Say2Body, SystemMessage, SystemMessageArg};

    #[test]
    fn push_keeps_the_last_200_lines_in_order() {
        let mut log = ChatLog::default();
        for i in 0..250 {
            log.push(ChatLine::new(CHAT_ALL, "a", i.to_string()));
        }
        assert_eq!(log.len(), CHAT_HISTORY);
        assert_eq!(CHAT_HISTORY, 200);
        let lines: Vec<&ChatLine> = log.iter().collect();
        assert_eq!(lines[0].text, "50");
        assert_eq!(lines[199].text, "249");
    }

    #[test]
    fn say2_npc_say_and_system_message_become_lines() {
        let say = ChatLine::from_event(&WorldEvent::Say2(Say2::text(1, CHAT_SHOUT, "Luzal", "oi"))).unwrap();
        assert_eq!((say.kind, say.sender.as_str(), say.text.as_str()), (CHAT_SHOUT, "Luzal", "oi"));
        let boat = ChatLine::from_event(&WorldEvent::Say2(Say2 {
            object_id: 1,
            kind: CHAT_BOAT,
            body: Say2Body::Tables { sys_string_id: 1500, sys_msg_id: 1200 },
        }))
        .unwrap();
        assert_eq!((boat.kind, boat.sender.as_str(), boat.text.as_str()), (CHAT_BOAT, "sysstring 1500", "sysmsg 1200"));
        let npc = ChatLine::from_event(&WorldEvent::NpcSay(NpcSay {
            object_id: 1,
            kind: CHAT_ALL,
            npc_id: 30001,
            text: "bem-vindo".into(),
        }))
        .unwrap();
        assert_eq!((npc.kind, npc.sender.as_str(), npc.text.as_str()), (CHAT_ALL, "npc 30001", "bem-vindo"));
        let system = ChatLine::from_event(&WorldEvent::SystemMessage(SystemMessage {
            id: 30,
            args: vec![
                SystemMessageArg::Text("Luzal".into()),
                SystemMessageArg::Number(42),
                SystemMessageArg::SkillName { id: 3, level: 2 },
                SystemMessageArg::CastleName(1),
                SystemMessageArg::ItemName(57),
                SystemMessageArg::NpcName(1),
                SystemMessageArg::Unknown { kind: 9 },
            ],
        }))
        .unwrap();
        assert_eq!(system.kind, CHAT_SYSTEM);
        assert_eq!(system.sender, "");
        assert_eq!(system.text, "#30 Luzal 42 skill 3 level 2 castle 1 item 57 npc 1 ?9");
        assert_eq!(
            ChatLine::from_event(&WorldEvent::SystemMessage(SystemMessage { id: 7, args: vec![] })).unwrap().text,
            "#7"
        );
        assert!(ChatLine::from_event(&WorldEvent::Revive(l2_net::game::Revive { object_id: 1 })).is_none());
    }

    #[test]
    fn pledge_party_and_alliance_notices_use_the_message_ids_of_the_console_methods() {
        use l2_net::game::{AllianceIdResult, PartyMemberResult, PledgeIdResult, PledgePartyConfirmationNoop};
        let notice = |event: WorldEvent| notice_of(&event);
        let table = |id| Some(Notice { id, color: None });
        let colored = |id| Some(Notice { id, color: Some(0xffb0_9b79) });
        let pledge = |pledge_id| PledgeIdResult { pledge_id };
        let member = |member_id| PartyMemberResult { member_id };
        let alliance = |alliance_id| AllianceIdResult { alliance_id };
        assert_eq!(notice(WorldEvent::CreatePledge(pledge(1))), table(189));
        assert_eq!(notice(WorldEvent::CreatePledge(pledge(0))), table(190));
        assert_eq!(notice(WorldEvent::OustPledgeMember(pledge(5))), table(309));
        assert_eq!(notice(WorldEvent::OustPledgeMember(pledge(0))), table(310));
        assert_eq!(notice(WorldEvent::WithdrawalPledge(pledge(1))), table(197));
        assert_eq!(notice(WorldEvent::WithdrawalPledge(pledge(0))), table(1120));
        assert_eq!(notice(WorldEvent::SetOustPledgeMemberNoop(PledgePartyConfirmationNoop)), table(199));
        assert_eq!(notice(WorldEvent::SetDismissPledgeNoop(PledgePartyConfirmationNoop)), table(199));
        assert_eq!(notice(WorldEvent::WithdrawalParty(member(1))), colored(200));
        assert_eq!(notice(WorldEvent::WithdrawalParty(member(0))), colored(316));
        assert_eq!(notice(WorldEvent::OustPartyMember(member(0))), table(317));
        assert_eq!(notice(WorldEvent::OustPartyMember(member(3))), None);
        assert_eq!(notice(WorldEvent::SetOustPartyMemberNoop(PledgePartyConfirmationNoop)), table(202));
        assert_eq!(notice(WorldEvent::JoinAllianceResult(alliance(1))), table(517));
        assert_eq!(notice(WorldEvent::JoinAllianceResult(alliance(0))), table(518));
        assert_eq!(notice(WorldEvent::WithdrawAllianceResult(alliance(1))), colored(519));
        assert_eq!(notice(WorldEvent::WithdrawAllianceResult(alliance(0))), colored(520));
        assert_eq!(notice(WorldEvent::OustAllianceMemberPledgeResult(alliance(1))), colored(521));
        assert_eq!(notice(WorldEvent::OustAllianceMemberPledgeResult(alliance(0))), colored(522));
        assert_eq!(notice(WorldEvent::DismissAllianceResult(alliance(1))), colored(523));
        assert_eq!(notice(WorldEvent::DismissAllianceResult(alliance(0))), colored(524));
        assert_eq!(notice(WorldEvent::JoinPledge(l2_net::game::JoinPledge { pledge_id: 0 })), table(308));
        assert_eq!(notice(WorldEvent::JoinPledge(l2_net::game::JoinPledge { pledge_id: 4 })), None);
        assert_eq!(notice(WorldEvent::DismissPledge(pledge(1))), table(193));
        assert_eq!(notice(WorldEvent::DismissPledge(pledge(0))), table(194));
        assert_eq!(notice(WorldEvent::JoinParty(l2_net::game::JoinParty { response: 0 })), table(305));
        assert_eq!(notice(WorldEvent::JoinParty(l2_net::game::JoinParty { response: 1 })), None);
        assert_eq!(notice(WorldEvent::DismissParty(member(1))), colored(203));
        assert_eq!(notice(WorldEvent::DismissParty(member(0))), colored(318));
        assert_eq!(notice(WorldEvent::SetDismissPartyNoop(PledgePartyConfirmationNoop)), table(203));
        assert_eq!(notice(WorldEvent::Revive(l2_net::game::Revive { object_id: 1 })), None);
        assert_eq!(notice(WorldEvent::FriendAddRequestResult(l2_net::game::FriendAddRequestResult { result: 1 })), table(525));
        assert_eq!(notice(WorldEvent::FriendAddRequestResult(l2_net::game::FriendAddRequestResult { result: 0 })), table(526));
        assert_eq!(notice(WorldEvent::WarehouseDone(l2_net::game::WarehouseDone { success: false })), table(307));
        assert_eq!(notice(WorldEvent::WarehouseDone(l2_net::game::WarehouseDone { success: true })), None);
    }

    #[test]
    fn log_applies_events_directly() {
        let mut log = ChatLog::default();
        log.apply(&WorldEvent::Say2(Say2::text(1, CHAT_ALL, "a", "b")));
        log.apply(&WorldEvent::Revive(l2_net::game::Revive { object_id: 1 }));
        assert_eq!(log.len(), 1);
    }
}
