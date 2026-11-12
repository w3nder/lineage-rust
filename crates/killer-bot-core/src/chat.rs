use killer_world::chat::{CHAT_SYSTEM, ChatLog};
use l2_net::game::packets::{
    CHAT_ALL, CHAT_ALLIANCE, CHAT_ANNOUNCEMENT, CHAT_CLAN, CHAT_HERO_VOICE, CHAT_PARTY, CHAT_PARTY_ROOM, CHAT_SHOUT, CHAT_TELL, CHAT_TRADE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Channel {
    All,
    Shout,
    Whisper,
    Party,
    Clan,
    Ally,
    Trade,
    Hero,
    PartyRoom,
    Announcement,
    System,
    Unknown(i32),
}

impl Channel {
    pub const SENDABLE: [Channel; 8] =
        [Channel::All, Channel::Shout, Channel::Whisper, Channel::Party, Channel::Clan, Channel::Ally, Channel::Trade, Channel::Hero];

    pub fn kind(self) -> Option<i32> {
        match self {
            Channel::All => Some(CHAT_ALL),
            Channel::Shout => Some(CHAT_SHOUT),
            Channel::Whisper => Some(CHAT_TELL),
            Channel::Party => Some(CHAT_PARTY),
            Channel::Clan => Some(CHAT_CLAN),
            Channel::Ally => Some(CHAT_ALLIANCE),
            Channel::Trade => Some(CHAT_TRADE),
            Channel::Hero => Some(CHAT_HERO_VOICE),
            Channel::PartyRoom => Some(CHAT_PARTY_ROOM),
            Channel::Announcement | Channel::System | Channel::Unknown(_) => None,
        }
    }
}

pub fn channel_of(kind: i32) -> Channel {
    match kind {
        CHAT_ALL => Channel::All,
        CHAT_SHOUT => Channel::Shout,
        CHAT_TELL => Channel::Whisper,
        CHAT_PARTY => Channel::Party,
        CHAT_CLAN => Channel::Clan,
        CHAT_ALLIANCE => Channel::Ally,
        CHAT_TRADE => Channel::Trade,
        CHAT_HERO_VOICE => Channel::Hero,
        CHAT_PARTY_ROOM => Channel::PartyRoom,
        CHAT_ANNOUNCEMENT => Channel::Announcement,
        CHAT_SYSTEM => Channel::System,
        other => Channel::Unknown(other),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatEntry {
    pub channel: Channel,
    pub sender: String,
    pub text: String,
    pub system: Option<(i32, Vec<l2_net::game::SystemMessageArg>)>,
    pub tables: Option<(i32, i32)>,
}

pub const NPC_NAME_BASE: i32 = 1_000_000;
pub const CENTER_WARNING_GROUP: u32 = 6;
pub const SKIPPED_GROUPS: std::ops::RangeInclusive<u32> = 0xb..=0x10;

pub fn system_text(id: i32, args: &[l2_net::game::SystemMessageArg], names: &dyn crate::names::Names) -> Option<crate::names::SystemText> {
    use l2_net::game::SystemMessageArg;
    let mut message = names.system_message(id)?;
    for (index, arg) in args.iter().enumerate() {
        let value = match arg {
            SystemMessageArg::Text(text) => text.clone(),
            SystemMessageArg::Number(number) => number.to_string(),
            SystemMessageArg::NpcName(id) => names.npc(if *id >= NPC_NAME_BASE { id - NPC_NAME_BASE } else { *id }),
            SystemMessageArg::ItemName(id) => names.item(*id),
            SystemMessageArg::SkillName { id, level } => names.skill(*id, *level),
            SystemMessageArg::CastleName(id) => format!("castle {id}"),
            SystemMessageArg::Unknown { kind } => format!("?{kind}"),
        };
        for marker in ['s', 'c'] {
            message.text = message.text.replace(&format!("${marker}{}", index + 1), &value);
        }
    }
    Some(message)
}

pub fn system_hidden(group: u32) -> bool {
    group == CENTER_WARNING_GROUP || SKIPPED_GROUPS.contains(&group)
}

#[derive(Debug, Clone, Default)]
pub struct ChatBoxes {
    entries: Vec<ChatEntry>,
}

impl ChatBoxes {
    pub fn from_log(log: &ChatLog) -> Self {
        let entries = log.iter().map(|line| ChatEntry { channel: channel_of(line.kind), sender: line.sender.clone(), text: line.text.clone(), system: line.system.clone(), tables: line.tables }).collect();
        Self { entries }
    }

    pub fn lines(&self, channel: Channel) -> Vec<&ChatEntry> {
        self.entries.iter().filter(|entry| entry.channel == channel).collect()
    }

    pub fn all(&self) -> &[ChatEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_world::chat::ChatLine;

    fn log(lines: &[(i32, &str, &str)]) -> ChatLog {
        let mut log = ChatLog::default();
        for (kind, sender, text) in lines {
            log.push(ChatLine::new(*kind, *sender, *text));
        }
        log
    }

    #[test]
    fn each_chat_kind_goes_to_its_channel() {
        let boxes = ChatBoxes::from_log(&log(&[
            (CHAT_ALL, "a", "oi"),
            (CHAT_SHOUT, "b", "grito"),
            (CHAT_TELL, "c", "pm"),
            (CHAT_PARTY, "d", "grupo"),
            (CHAT_TRADE, "e", "vendo"),
            (CHAT_SYSTEM, "", "#1"),
        ]));
        assert_eq!(boxes.lines(Channel::All).len(), 1);
        assert_eq!(boxes.lines(Channel::Whisper)[0].text, "pm");
        assert_eq!(boxes.lines(Channel::Trade)[0].sender, "e");
        assert_eq!(boxes.lines(Channel::System).len(), 1);
    }

    struct Table;

    impl crate::names::Names for Table {
        fn item(&self, id: i32) -> String {
            format!("Item{id}")
        }
        fn skill(&self, id: i32, _level: i32) -> String {
            format!("Skill{id}")
        }
        fn npc(&self, id: i32) -> String {
            format!("Npc{id}")
        }
        fn class(&self, id: i32) -> String {
            format!("Class{id}")
        }
        fn find_skill(&self, _name: &str) -> Option<i32> {
            None
        }
        fn system_message(&self, id: i32) -> Option<crate::names::SystemText> {
            (id == 34).then(|| crate::names::SystemText { text: "You picked up $s2 $c1".to_owned(), color: [1, 2, 3], group: 2 })
        }
    }

    #[test]
    fn system_messages_fill_their_template_with_named_arguments() {
        use l2_net::game::SystemMessageArg;
        let args = [SystemMessageArg::ItemName(57), SystemMessageArg::Number(10)];
        let message = system_text(34, &args, &Table).unwrap();
        assert_eq!((message.text.as_str(), message.color, message.group), ("You picked up 10 Item57", [1, 2, 3], 2));
        assert_eq!(system_text(35, &args, &Table), None);
        let npc = [SystemMessageArg::NpcName(NPC_NAME_BASE + 5)];
        assert_eq!(system_text(34, &npc, &Table).map(|m| m.text), Some("You picked up $s2 Npc5".to_owned()));
    }

    #[test]
    fn center_warnings_and_skipped_groups_stay_out_of_the_chat() {
        assert!(system_hidden(6));
        assert!(system_hidden(0xb) && system_hidden(0x10));
        assert!(!system_hidden(2) && !system_hidden(3) && !system_hidden(0x11));
    }

    #[test]
    fn unknown_kind_goes_to_unknown_channel_instead_of_being_dropped() {
        let boxes = ChatBoxes::from_log(&log(&[(99, "x", "estranho")]));
        assert_eq!(boxes.lines(Channel::Unknown(99)).len(), 1);
        assert_eq!(boxes.all().len(), 1);
    }

    #[test]
    fn empty_text_lines_are_kept_with_empty_text() {
        let boxes = ChatBoxes::from_log(&log(&[(CHAT_ALL, "a", "")]));
        assert_eq!(boxes.lines(Channel::All)[0].text, "");
    }

    #[test]
    fn only_real_channels_can_be_sent_to() {
        assert!(Channel::SENDABLE.iter().all(|channel| channel.kind().is_some()));
        assert_eq!(Channel::System.kind(), None);
        assert_eq!(Channel::Whisper.kind(), Some(CHAT_TELL));
    }
}
