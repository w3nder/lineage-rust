use std::time::Duration;

use killer_world::{Entity, EntityKind, NetCommand, World};
use l2_net::game::packets::{MacroCommand, ShortcutKind};

use crate::names::Names;

pub const SEARCH_RADIUS: f32 = 200.0;
pub const PAGES: usize = 10;
pub const SLOTS_PER_PAGE: usize = 12;
pub const COMMAND_LINES: usize = 12;
pub const LINE_MAX_CHARS: usize = 32;

const SKILL_WAIT_GIVE_UP: f32 = 5.0;
const TOKEN_MAX: u32 = 0x6e;
const VERB_USESKILL: u32 = 0x18;
const VERB_USESHORTCUT: u32 = 0x1b;
const VERB_DELAY: u32 = 0x4c;
const ACTION_SITSTAND: i32 = 0;

const ACTION_COMMANDS: &[(u32, i32)] = &[
    (0x39, 0x00),
    (0x3a, 0x01),
    (0x3c, 0x26),
    (0x3f, 0x0f),
    (0x40, 0x10),
    (0x41, 0x11),
    (0x43, 0x13),
    (0x45, 0x15),
    (0x46, 0x16),
    (0x47, 0x17),
    (0x4b, 0x28),
    (0x69, 0x37),
    (0x6a, 0x36),
    (0x6b, 0x35),
    (0x6c, 0x34),
];

const SOCIAL_COMMANDS: &[(u32, i32)] = &[
    (0x15, 2),
    (0x16, 3),
    (0x17, 4),
    (0x2e, 5),
    (0x2f, 6),
    (0x30, 7),
    (0x31, 8),
    (0x32, 9),
    (0x33, 10),
    (0x36, 11),
    (0x37, 12),
    (0x38, 13),
];

#[derive(Debug, Clone, PartialEq)]
pub enum MacroEffect {
    Net(NetCommand),
    Select(i32),
    Log(String),
}

pub fn shortcut_slot(page: i64, slot: i64) -> Option<usize> {
    let inside = (1..=PAGES as i64).contains(&page) && (1..=SLOTS_PER_PAGE as i64).contains(&slot);
    inside.then(|| usize::try_from((page - 1) * SLOTS_PER_PAGE as i64 + (slot - 1)).ok())?
}

fn shortcut_slot_of(args: &[&str]) -> Option<usize> {
    shortcut_slot(args.first()?.parse().ok()?, args.get(1)?.parse().ok()?)
}

pub fn line_of(command: &MacroCommand, names: &dyn Names) -> String {
    let verb = |token: u32| names.command_verb(token).unwrap_or_default();
    match command.kind {
        1 => {
            let name = names.skill(command.skill_id, i32::from(command.shortcut_id));
            format!("/{} {name}", verb(VERB_USESKILL))
        }
        2 => match ACTION_COMMANDS.iter().find(|(_, action)| *action == command.skill_id) {
            Some((token, _)) => format!("/{}", verb(*token)),
            None => format!("/{}", command.skill_id),
        },
        4 => format!("/{} {} {}", verb(VERB_USESHORTCUT), command.skill_id + 1, i32::from(command.shortcut_id) + 1),
        6 => format!("/{} {}", verb(VERB_DELAY), command.skill_id),
        _ => command.command.clone(),
    }
}

pub fn command_of_line(line: &str, index: u8, names: &dyn Names) -> Option<MacroCommand> {
    let command = |kind: u8, skill_id: i32, shortcut_id: u8, text: &str| Some(MacroCommand { index, kind, skill_id, shortcut_id, command: text.to_owned() });
    let free_text = || command(3, 0, 0, line);
    let mut words = line.split_whitespace();
    let Some(verb) = words.next() else { return free_text() };
    let Some(token) = names.command_token(&verb.trim_start_matches('/').to_ascii_lowercase()) else {
        return free_text();
    };
    match token {
        VERB_USESKILL => {
            let name = words.collect::<Vec<_>>().join(" ");
            match names.find_skill(&name) {
                Some(id) => command(1, id, 1, ""),
                None => command(1, 0, 0, ""),
            }
        }
        VERB_USESHORTCUT => {
            let page: i32 = words.next()?.parse().ok()?;
            let slot: i32 = words.next()?.parse().ok()?;
            ((1..=PAGES as i32).contains(&page) && (1..=SLOTS_PER_PAGE as i32).contains(&slot)).then(|| command(4, page - 1, u8::try_from(slot - 1).ok()?, ""))?
        }
        VERB_DELAY => {
            let seconds: i32 = words.next()?.parse().ok()?;
            (seconds >= 1).then(|| command(6, seconds, 0, ""))?
        }
        token if token >= TOKEN_MAX => None,
        _ => free_text(),
    }
}

fn placeholder(world: &World, arg: &str) -> Option<String> {
    match arg {
        "%target" => world.target.and_then(|id| world.entity(id)).map(|entity| entity.name.clone()),
        "%self" => world.me.as_ref().map(|entity| entity.name.clone()),
        "%pet" => world.pet.as_ref().and_then(|pet| world.entity(pet.info.object_id)).map(|entity| entity.name.clone()),
        _ if arg.starts_with("%party") => {
            let index: usize = arg["%party".len()..].parse().ok()?;
            world.party.members.get(index.checked_sub(1)?).map(|member| member.name.clone())
        }
        _ => Some(arg.to_owned()),
    }
}

fn nearest(world: &World, skip: Option<i32>, accept: impl Fn(&Entity) -> bool) -> Option<i32> {
    let me = world.me.as_ref()?;
    world
        .entities
        .values()
        .filter(|entity| Some(entity.object_id) != skip && accept(entity))
        .map(|entity| (entity.object_id, me.position.distance(entity.position)))
        .filter(|(_, distance)| *distance <= SEARCH_RADIUS)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(object_id, _)| object_id)
}

fn is_item(entity: &Entity) -> bool {
    matches!(entity.kind, EntityKind::Item { .. })
}

fn attack(world: &World, force: bool, stand: bool, out: &mut Vec<MacroEffect>) {
    let (Some(object_id), Some(me)) = (world.target, world.me.as_ref()) else { return };
    let origin = [me.position.x as i32, me.position.y as i32, me.position.z as i32];
    out.push(MacroEffect::Net(if force { NetCommand::AttackRequest { object_id, origin, shift: stand } } else { NetCommand::Action { object_id, origin, shift: stand } }));
}

pub fn use_action(world: &World, names: &dyn Names, id: i32, out: &mut Vec<MacroEffect>) {
    let net = |command: NetCommand| MacroEffect::Net(command);
    let player = world.targeted().filter(|target| matches!(target.kind, EntityKind::Player { .. }));
    match id {
        2 => attack(world, false, false, out),
        3 => {
            if let Some(target) = world.targeted().filter(|target| !is_item(target)) {
                out.push(net(NetCommand::Trade { object_id: target.object_id }));
            }
        }
        4 => out.extend(nearest(world, world.target, |entity| !is_item(entity)).map(MacroEffect::Select)),
        5 => out.extend(nearest(world, None, is_item).map(MacroEffect::Select)),
        6 => {
            if let Some(object_id) = world.targeted().and_then(|target| target.target)
                && world.entity(object_id).is_some()
                && let Some(me) = &world.me
            {
                out.push(net(NetCommand::Action { object_id, origin: me.position.to_array().map(|value| value as i32), shift: false }));
            }
        }
        7 | 9 | 50 | 56 => match player {
            Some(target) => {
                let name = target.name.clone();
                out.push(net(match id {
                    7 => NetCommand::JoinParty { name, item_distribution: 0 },
                    9 => NetCommand::OustPartyMember { name },
                    50 => NetCommand::ChangePartyLeader { name },
                    _ => NetCommand::ExAskJoinMpcc { name },
                }));
            }
            None => out.push(MacroEffect::Log("selecione um jogador".into())),
        },
        8 => out.push(net(NetCommand::WithdrawParty)),
        10 => out.push(net(NetCommand::PrivateStoreManageSell)),
        11 => out.push(net(NetCommand::PartyMatchList { level: 1, location: -1, mode: 1 })),
        12..=14 | 24..=26 | 29..=31 | 33..=35 => {
            let action = names.action_social(id);
            if action > 0 {
                out.push(net(NetCommand::SocialAction { action }));
            }
        }
        18 => {
            if let Some(pet) = world.pet.as_ref().and_then(|pet| world.entity(pet.info.object_id))
                && let Some(item) = world
                    .entities
                    .values()
                    .filter(|entity| is_item(entity))
                    .map(|entity| (entity.object_id, pet.position.distance(entity.position)))
                    .filter(|(_, distance)| *distance < SEARCH_RADIUS)
                    .min_by(|a, b| a.1.total_cmp(&b.1))
            {
                out.push(net(NetCommand::PetGetItem { object_id: item.0 }));
            }
        }
        28 => out.push(net(NetCommand::PrivateStoreManageBuy)),
        40 => match player {
            Some(target) => out.push(net(NetCommand::VoteSociality { target_id: target.object_id })),
            None => out.push(MacroEffect::Log("selecione um jogador".into())),
        },
        _ => out.push(net(NetCommand::UseAction { id, ctrl: false, shift: false })),
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Run {
    macro_id: i32,
    index: usize,
    started: Duration,
    delay: f32,
    waiting_skill: Option<i32>,
}

#[derive(Debug, Default)]
pub struct MacroRunner {
    run: Option<Run>,
    social_durations: Vec<f32>,
}

impl MacroRunner {
    pub fn set_social_durations(&mut self, durations: Vec<f32>) {
        self.social_durations = durations;
    }

    pub fn running(&self) -> Option<(i32, usize)> {
        self.run.as_ref().map(|run| (run.macro_id, run.index))
    }

    pub fn stop(&mut self) {
        self.run = None;
    }

    pub fn start(&mut self, now: Duration, world: &World, names: &dyn Names, id: i32, out: &mut Vec<MacroEffect>) {
        if !world.macros.macros.iter().any(|entry| entry.id == id) {
            out.push(MacroEffect::Log(format!("macro {id} não está na lista do servidor")));
            return;
        }
        self.run = Some(Run { macro_id: id, index: 0, started: now, delay: 0.0, waiting_skill: None });
        self.fire(now, world, names, 0, out);
    }

    pub fn use_shortcut(&mut self, now: Duration, world: &World, names: &dyn Names, slot: usize, out: &mut Vec<MacroEffect>) {
        let Some(shortcut) = world.shortcut(slot as i32) else { return };
        match shortcut.kind {
            ShortcutKind::Item { object_id } => out.push(MacroEffect::Net(NetCommand::UseItem { object_id, ctrl: false })),
            ShortcutKind::Skill { skill_id, .. } => out.push(MacroEffect::Net(NetCommand::UseSkill { skill_id, ctrl: false, shift: false })),
            ShortcutKind::Action { id } => use_action(world, names, id, out),
            ShortcutKind::Macro { id } => self.start(now, world, names, id, out),
            ShortcutKind::Recipe { id } => out.push(MacroEffect::Net(NetCommand::RecipeItemMakeInfo { recipe_id: id })),
            ShortcutKind::Other { kind } => out.push(MacroEffect::Log(format!("atalho de tipo {kind} desconhecido"))),
        }
    }

    pub fn note_cast(&mut self, now: Duration, skill_id: i32, hit_time: i32) {
        let Some(run) = self.run.as_mut() else { return };
        if run.waiting_skill != Some(skill_id) {
            return;
        }
        run.waiting_skill = None;
        run.started = now;
        run.delay = hit_time.max(0) as f32 / 1000.0;
    }

    pub fn tick(&mut self, now: Duration, world: &World, names: &dyn Names, out: &mut Vec<MacroEffect>) {
        let Some(run) = self.run.as_mut() else { return };
        if run.waiting_skill.is_some() {
            if now.saturating_sub(run.started).as_secs_f32() < SKILL_WAIT_GIVE_UP {
                return;
            }
            run.waiting_skill = None;
            run.delay = 0.0;
        }
        if now.saturating_sub(run.started).as_secs_f32() < run.delay {
            return;
        }
        let (macro_id, next) = (run.macro_id, run.index + 1);
        let Some(count) = world.macros.macros.iter().find(|entry| entry.id == macro_id).map(|entry| entry.commands.len()) else {
            self.run = None;
            return;
        };
        if next >= count {
            self.run = None;
            return;
        }
        self.run = Some(Run { macro_id, index: next, started: now, delay: 0.0, waiting_skill: None });
        self.fire(now, world, names, next, out);
    }

    fn fire(&mut self, now: Duration, world: &World, names: &dyn Names, index: usize, out: &mut Vec<MacroEffect>) {
        let Some(macro_id) = self.run.as_ref().map(|run| run.macro_id) else { return };
        let Some(command) = world.macros.macros.iter().find(|entry| entry.id == macro_id).and_then(|entry| entry.commands.get(index)).cloned() else { return };
        let delay = match command.kind {
            1 => {
                out.push(MacroEffect::Net(NetCommand::UseSkill { skill_id: command.skill_id, ctrl: false, shift: false }));
                if let Some(run) = self.run.as_mut() {
                    run.waiting_skill = Some(command.skill_id);
                }
                0.0
            }
            2 => {
                use_action(world, names, command.skill_id, out);
                self.action_delay(world, command.skill_id)
            }
            3 => {
                self.text_command(now, world, names, &command.command, out);
                0.0
            }
            4 => {
                let slot = shortcut_slot(i64::from(command.skill_id) + 1, i64::from(command.shortcut_id) + 1);
                if let Some(slot) = slot {
                    self.use_shortcut(now, world, names, slot, out);
                }
                0.0
            }
            6 => command.skill_id.max(0) as f32,
            kind => {
                out.push(MacroEffect::Log(format!("comando de macro \"{}\" (tipo {kind}) não suportado", command.command)));
                0.0
            }
        };
        if let Some(run) = self.run.as_mut()
            && run.macro_id == macro_id
            && run.index == index
        {
            run.delay = delay;
            run.started = now;
        }
    }

    fn action_delay(&self, world: &World, action: i32) -> f32 {
        let sitting = world.me.as_ref().is_some_and(|me| me.sitting);
        let anim = match action {
            ACTION_SITSTAND => Some(usize::from(sitting)),
            _ => SOCIAL_ACTION_ANIMS.iter().find(|(id, _)| *id == action).map(|(_, anim)| *anim),
        };
        anim.and_then(|anim| self.social_durations.get(anim).copied()).filter(|duration| *duration > 0.0).unwrap_or(0.0)
    }

    fn text_command(&mut self, now: Duration, world: &World, names: &dyn Names, command: &str, out: &mut Vec<MacroEffect>) {
        let mut words = command.split_whitespace();
        let Some(verb) = words.next() else { return };
        let key = verb.trim_start_matches('/').to_ascii_lowercase();
        let Some(token) = names.command_token(&key) else {
            out.push(MacroEffect::Log(format!("comando de macro \"{command}\": verbo \"{key}\" desconhecido")));
            return;
        };
        let args: Vec<&str> = words.collect();
        let name = || -> Option<String> {
            let parts: Vec<String> = args.iter().filter_map(|arg| placeholder(world, arg)).collect();
            (!parts.is_empty()).then(|| parts.join(" "))
        };
        let target_name = || name().or_else(|| world.target.and_then(|id| world.entity(id)).map(|entity| entity.name.clone()));
        let net = |command: NetCommand| MacroEffect::Net(command);
        if let Some((_, action)) = ACTION_COMMANDS.iter().find(|(entry, _)| *entry == token) {
            use_action(world, names, *action, out);
            return;
        }
        if let Some((_, social)) = SOCIAL_COMMANDS.iter().find(|(entry, _)| *entry == token) {
            out.push(net(NetCommand::SocialAction { action: *social }));
            return;
        }
        match token {
            0x4 => out.push(net(NetCommand::ChangeWaitType { standing: false })),
            0x5 => out.push(net(NetCommand::ChangeWaitType { standing: true })),
            0x6 => out.push(net(NetCommand::ChangeMoveType { running: false })),
            0x7 => out.push(net(NetCommand::ChangeMoveType { running: true })),
            0x8..=0xa => attack(world, token == 0x9, token == 0xa, out),
            0xb => {
                if let Some(target) = world.target {
                    out.push(net(NetCommand::Trade { object_id: target }));
                }
            }
            0xc | 0xf => {
                if let Some(name) = name() {
                    match world.entities.values().find(|entity| entity.name.eq_ignore_ascii_case(&name)) {
                        Some(entity) => out.push(MacroEffect::Select(entity.object_id)),
                        None => out.push(MacroEffect::Log(format!("\"{name}\" não está visível"))),
                    }
                }
            }
            0xd => out.extend(nearest(world, world.target, |entity| !is_item(entity)).map(MacroEffect::Select)),
            0xe | 0x42 => out.extend(nearest(world, None, is_item).map(MacroEffect::Select)),
            0x10 => match target_name() {
                Some(name) => out.push(net(NetCommand::JoinParty { name, item_distribution: 0 })),
                None => out.push(MacroEffect::Log("selecione um alvo para convidar".into())),
            },
            0x11 => out.push(net(NetCommand::WithdrawParty)),
            0x12 => match target_name() {
                Some(name) => out.push(net(NetCommand::OustPartyMember { name })),
                None => out.push(MacroEffect::Log("selecione um membro do grupo".into())),
            },
            0x13 => out.push(net(NetCommand::PrivateStoreManageSell)),
            0x27 => out.push(net(NetCommand::PrivateStoreManageBuy)),
            0x3 => out.push(net(NetCommand::RequestGmList)),
            0x1e => match world.target {
                Some(object_id) => out.push(net(NetCommand::JoinAlly { object_id })),
                None => out.push(MacroEffect::Log("selecione um alvo para convidar à aliança".into())),
            },
            0x14 => out.push(net(NetCommand::PartyMatchList { level: 1, location: -1, mode: 1 })),
            0x5b => out.push(net(NetCommand::SetAllyCrest { data: Vec::new() })),
            0x23 => out.push(net(NetCommand::RequestAllyInfo)),
            0x24 => out.extend(name().map(|name| net(NetCommand::FriendInvite { name }))),
            0x25 => out.push(net(NetCommand::FriendList)),
            0x26 => out.extend(name().map(|name| net(NetCommand::FriendDel { name }))),
            0x3b => out.push(net(NetCommand::UseAction { id: 0x25, ctrl: false, shift: false })),
            0x65 => out.push(net(NetCommand::UseAction { id: 0x33, ctrl: false, shift: false })),
            0x18..=0x1a => match name().and_then(|name| names.find_skill(&name)) {
                Some(skill_id) => out.push(net(NetCommand::UseSkill { skill_id, ctrl: token == 0x19, shift: token == 0x1a })),
                None => out.push(MacroEffect::Log(format!("skill não encontrada em \"{command}\""))),
            },
            0x1b..=0x1d => {
                if let Some(slot) = shortcut_slot_of(&args) {
                    self.use_shortcut(now, world, names, slot, out);
                }
            }
            0x50 => {
                if let Some(slot) = shortcut_slot_of(&args)
                    && let Some(shortcut) = world.shortcut(slot as i32)
                    && let ShortcutKind::Item { object_id } = shortcut.kind
                {
                    out.push(net(NetCommand::UseItem { object_id, ctrl: false }));
                }
            }
            0x2b => out.push(net(NetCommand::Block { kind: 2, name: String::new() })),
            0x2c | 0x2d => out.extend(name().map(|name| net(NetCommand::Block { kind: i32::from(token == 0x2d), name }))),
            0x4e => out.push(net(NetCommand::Block { kind: 3, name: String::new() })),
            0x4f => out.push(net(NetCommand::Block { kind: 4, name: String::new() })),
            0x55 => out.extend(target_name().map(|name| net(NetCommand::ChangePartyLeader { name }))),
            0x35 => out.push(net(NetCommand::RequestRemainTime)),
            0x2 => out.push(net(NetCommand::PetitionCancel { content: name().unwrap_or_default() })),
            0x5e | 0x5f => match target_name() {
                Some(name) if token == 0x5e => out.push(net(NetCommand::ExAskJoinMpcc { name })),
                Some(name) => out.push(net(NetCommand::ExOustFromMpcc { name })),
                None => out.push(MacroEffect::Log("selecione um alvo ou informe o nome".into())),
            },
            0x28 | 0x29 | 0x2a | 0x44 | 0x61 => {}
            0x1 | 0x20 | 0x21 | 0x1f | 0x22 | 0x54 | 0x56 | 0x57 | 0x67 | 0x68 => out.push(MacroEffect::Log(format!("comando de macro \"{command}\" não é suportado no bot"))),
            _ => out.push(net(NetCommand::UserCommand { command: token as i32 })),
        }
    }
}

const SOCIAL_ACTION_ANIMS: &[(i32, usize)] = &[(12, 2), (13, 3), (14, 4), (24, 6), (25, 5), (26, 7), (29, 8), (30, 9), (31, 10), (33, 11), (34, 12), (35, 13)];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{Vitals, npc_info, user_info};
    use killer_world::WorldEvent;
    use l2_net::game::SendMacroList;
    use std::collections::HashMap;

    struct Table(HashMap<String, u32>);

    impl Names for Table {
        fn item(&self, id: i32) -> String {
            id.to_string()
        }
        fn skill(&self, id: i32, _level: i32) -> String {
            if id == 56 { "Power Strike".to_owned() } else { id.to_string() }
        }
        fn npc(&self, id: i32) -> String {
            id.to_string()
        }
        fn class(&self, id: i32) -> String {
            id.to_string()
        }
        fn find_skill(&self, name: &str) -> Option<i32> {
            name.eq_ignore_ascii_case("power strike").then_some(56)
        }
        fn command_token(&self, verb: &str) -> Option<u32> {
            self.0.get(verb).copied()
        }
        fn command_verb(&self, token: u32) -> Option<String> {
            self.0.iter().find(|(_, entry)| **entry == token).map(|(name, _)| name.clone())
        }
    }

    fn table() -> Table {
        Table(
            [("useskill", 0x18), ("useshortcut", 0x1b), ("delay", 0x4c), ("target", 0xc), ("sit", 0x4), ("stand", 0x5), ("attack", 0x8), ("socialhello", 0x15), ("sitstand", 0x39), ("loc", 0x68 - 1)]
                .into_iter()
                .map(|(name, token)| (name.to_owned(), token))
                .collect(),
        )
    }

    fn world_with(commands: Vec<MacroCommand>) -> World {
        let mut world = crate::testkit::world_of(&[user_info(1, "Wender", &Vitals::full(), [0, 0, 0]), npc_info(9, 20001, "Wolf", true, [50, 0, 0])]);
        let entry = l2_net::game::Macro { id: 7, name: "m".into(), description: String::new(), acronym: "M".into(), icon: 1, commands };
        world.apply_world(&WorldEvent::SendMacroList(SendMacroList { flag: 0, revision: 1, total_count: 1, macros: vec![entry] }));
        world
    }

    fn cmd(index: u8, kind: u8, skill_id: i32, shortcut_id: u8, text: &str) -> MacroCommand {
        MacroCommand { index, kind, skill_id, shortcut_id, command: text.to_owned() }
    }

    fn secs(value: f32) -> Duration {
        Duration::from_secs_f32(value)
    }

    #[test]
    fn lines_round_trip_through_the_editor_parser() {
        let names = table();
        assert_eq!(command_of_line("/useskill Power Strike", 0, &names), Some(cmd(0, 1, 56, 1, "")));
        assert_eq!(command_of_line("/useshortcut 2 3", 1, &names), Some(cmd(1, 4, 1, 2, "")));
        assert_eq!(command_of_line("/delay 4", 2, &names), Some(cmd(2, 6, 4, 0, "")));
        assert_eq!(command_of_line("/delay 0", 2, &names), None);
        assert_eq!(command_of_line("/useshortcut 11 1", 1, &names), None);
        assert_eq!(command_of_line("/target Wolf", 3, &names), Some(cmd(3, 3, 0, 0, "/target Wolf")));
        assert_eq!(command_of_line("texto solto", 4, &names), Some(cmd(4, 3, 0, 0, "texto solto")));
        assert_eq!(line_of(&cmd(0, 1, 56, 1, ""), &names), "/useskill Power Strike");
        assert_eq!(line_of(&cmd(1, 4, 1, 2, ""), &names), "/useshortcut 2 3");
        assert_eq!(line_of(&cmd(2, 6, 4, 0, ""), &names), "/delay 4");
    }

    #[test]
    fn a_macro_fires_one_command_at_a_time_and_waits_the_delay() {
        let names = table();
        let world = world_with(vec![cmd(0, 3, 0, 0, "/sit"), cmd(1, 6, 2, 0, ""), cmd(2, 3, 0, 0, "/stand")]);
        let mut runner = MacroRunner::default();
        let mut out = Vec::new();
        runner.start(secs(0.0), &world, &names, 7, &mut out);
        assert_eq!(out, vec![MacroEffect::Net(NetCommand::ChangeWaitType { standing: false })]);
        out.clear();
        runner.tick(secs(0.1), &world, &names, &mut out);
        assert!(out.is_empty());
        assert_eq!(runner.running(), Some((7, 1)));
        runner.tick(secs(1.9), &world, &names, &mut out);
        assert!(out.is_empty());
        runner.tick(secs(2.2), &world, &names, &mut out);
        assert_eq!(out, vec![MacroEffect::Net(NetCommand::ChangeWaitType { standing: true })]);
        runner.tick(secs(2.3), &world, &names, &mut out);
        assert_eq!(runner.running(), None);
    }

    #[test]
    fn a_skill_command_waits_for_the_cast_then_for_its_duration_before_the_next_line() {
        let names = table();
        let world = world_with(vec![cmd(0, 1, 56, 1, ""), cmd(1, 3, 0, 0, "/sit")]);
        let mut runner = MacroRunner::default();
        let mut out = Vec::new();
        runner.start(secs(0.0), &world, &names, 7, &mut out);
        assert_eq!(out, vec![MacroEffect::Net(NetCommand::UseSkill { skill_id: 56, ctrl: false, shift: false })]);
        out.clear();
        runner.tick(secs(1.0), &world, &names, &mut out);
        assert!(out.is_empty());
        runner.note_cast(secs(1.0), 99, 500);
        runner.tick(secs(2.0), &world, &names, &mut out);
        assert!(out.is_empty(), "outra skill não destrava");
        runner.note_cast(secs(1.2), 56, 2000);
        runner.tick(secs(2.5), &world, &names, &mut out);
        assert!(out.is_empty());
        runner.tick(secs(3.3), &world, &names, &mut out);
        assert_eq!(out, vec![MacroEffect::Net(NetCommand::ChangeWaitType { standing: false })]);
    }

    #[test]
    fn a_skill_that_never_starts_does_not_freeze_the_macro_forever() {
        let names = table();
        let world = world_with(vec![cmd(0, 1, 56, 1, ""), cmd(1, 3, 0, 0, "/sit")]);
        let mut runner = MacroRunner::default();
        let mut out = Vec::new();
        runner.start(secs(0.0), &world, &names, 7, &mut out);
        out.clear();
        runner.tick(secs(4.0), &world, &names, &mut out);
        assert!(out.is_empty());
        runner.tick(secs(5.5), &world, &names, &mut out);
        assert_eq!(out, vec![MacroEffect::Net(NetCommand::ChangeWaitType { standing: false })]);
    }

    #[test]
    fn target_resolves_the_name_and_unknown_verbs_are_reported() {
        let names = table();
        let world = world_with(vec![cmd(0, 3, 0, 0, "/target wolf"), cmd(1, 3, 0, 0, "/target Ghost"), cmd(2, 3, 0, 0, "/nonsense")]);
        let mut runner = MacroRunner::default();
        let mut out = Vec::new();
        runner.start(secs(0.0), &world, &names, 7, &mut out);
        assert_eq!(out, vec![MacroEffect::Select(9)]);
        out.clear();
        runner.tick(secs(0.1), &world, &names, &mut out);
        assert!(matches!(out.as_slice(), [MacroEffect::Log(text)] if text.contains("Ghost")));
        out.clear();
        runner.tick(secs(0.2), &world, &names, &mut out);
        assert!(matches!(out.as_slice(), [MacroEffect::Log(text)] if text.contains("nonsense")));
    }

    #[test]
    fn useshortcut_flattens_page_and_slot_and_rejects_out_of_range() {
        assert_eq!(shortcut_slot(1, 1), Some(0));
        assert_eq!(shortcut_slot(2, 3), Some(14));
        assert_eq!(shortcut_slot(10, 12), Some(119));
        assert_eq!(shortcut_slot(0, 1), None);
        assert_eq!(shortcut_slot(1, 13), None);
    }

    #[test]
    fn running_an_unknown_macro_says_so() {
        let names = table();
        let world = world_with(Vec::new());
        let mut runner = MacroRunner::default();
        let mut out = Vec::new();
        runner.start(secs(0.0), &world, &names, 99, &mut out);
        assert!(matches!(out.as_slice(), [MacroEffect::Log(_)]));
        assert_eq!(runner.running(), None);
    }
}
