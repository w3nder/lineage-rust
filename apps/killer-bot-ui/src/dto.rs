use killer_bot_core::chat::{Channel, ChatBoxes};
use killer_bot_core::names::Names;
use killer_bot_core::session::BotState;
use killer_bot_core::snapshot::{NearbyKind, Snapshot};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub screen: &'static str,
    pub sessions: Vec<SessionDto>,
    pub selected: Option<u32>,
    pub status: String,
    pub servers: Vec<ServerDto>,
    pub characters: Vec<CharacterEntryDto>,
    pub saved: Vec<SavedDto>,
    pub data_status: String,
    pub data_ready: bool,
    pub world: Option<WorldDto>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SavedDto {
    pub label: String,
    pub account: String,
    pub character: String,
    pub host: String,
    pub port: u16,
    pub has_password: bool,
    pub online: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionDto {
    pub id: u32,
    pub label: String,
    pub state: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServerDto {
    pub id: u8,
    pub name: String,
    pub address: String,
    pub players: u16,
    pub max_players: u16,
    pub online: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CharacterEntryDto {
    pub slot: usize,
    pub name: String,
    pub class_name: String,
    pub level: i32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GaugeDto {
    pub current: i32,
    pub max: i32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatsDto {
    pub p_atk: i32,
    pub m_atk: i32,
    pub p_def: i32,
    pub m_def: i32,
    pub accuracy: i32,
    pub evasion: i32,
    pub critical: i32,
    pub speed: i32,
    pub atk_spd: i32,
    pub cast_spd: i32,
    pub str: i32,
    pub dex: i32,
    pub con: i32,
    pub int: i32,
    pub wit: i32,
    pub men: i32,
    pub sp: i32,
    pub load: i32,
    pub max_load: i32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CharacterDto {
    pub name: String,
    pub title: String,
    pub class_name: String,
    pub level: i32,
    pub hp: GaugeDto,
    pub mp: GaugeDto,
    pub cp: GaugeDto,
    pub xp_percent: f32,
    pub karma: i32,
    pub pvp: i32,
    pub pk: i32,
    pub recommend_have: i32,
    pub recommend_left: i32,
    pub adena: i32,
    pub position: [i32; 3],
    pub stats: StatsDto,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NearbyDto {
    pub id: i32,
    pub name: String,
    pub title: String,
    pub npc_id: i32,
    pub item_id: i32,
    pub count: i32,
    pub add_name: String,
    pub class_name: String,
    pub clan_id: i32,
    pub clan: String,
    pub kind: &'static str,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub distance: f32,
    pub dead: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TargetDto {
    pub id: i32,
    pub name: String,
    pub kind: &'static str,
    pub hp_percent: Option<u8>,
    pub dead: bool,
    pub distance: i32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChatLineDto {
    pub at: u64,
    pub channel: &'static str,
    pub sender: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ItemDto {
    pub id: i32,
    pub item_id: i32,
    pub name: String,
    pub add_name: String,
    pub grade: Option<&'static str>,
    pub known: bool,
    pub count: i32,
    pub equipped: bool,
    pub body_part: i32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SkillDto {
    pub id: i32,
    pub level: i32,
    pub name: String,
    pub passive: bool,
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BuffDto {
    pub skill_id: i32,
    pub level: i32,
    pub name: String,
    pub seconds: i32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MemberDto {
    pub id: i32,
    pub name: String,
    pub hp: GaugeDto,
    pub mp: GaugeDto,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClanMemberDto {
    pub name: String,
    pub level: i32,
    pub class_name: String,
    pub online: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClanDto {
    pub name: String,
    pub leader: String,
    pub level: i32,
    pub castle_id: i32,
    pub castle: Option<String>,
    pub hideout_id: i32,
    pub ally: String,
    pub at_war: bool,
    pub members: Vec<ClanMemberDto>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LootDto {
    pub source: String,
    pub item_id: i32,
    pub name: String,
    pub add_name: String,
    pub count: i32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GainsDto {
    pub exp_per_hour: i64,
    pub sp_per_hour: i64,
    pub adena_per_hour: i64,
    pub adena_total: i64,
    pub bad_mobs: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeathDto {
    pub town: bool,
    pub hideout: bool,
    pub castle: bool,
    pub siege_hq: bool,
    pub fortress: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MacroDto {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub acronym: String,
    pub icon: u8,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MacroRunDto {
    pub id: i32,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutDto {
    pub slot: usize,
    pub kind: &'static str,
    pub id: i32,
    pub item_id: i32,
    pub level: i32,
    pub name: String,
    pub icon: String,
    pub count: Option<i32>,
    pub ready: bool,
    pub acronym: String,
    pub shot: bool,
    pub auto: bool,
}

pub fn macro_dtos(world: &killer_world::World, names: &dyn Names) -> Vec<MacroDto> {
    world
        .macros
        .macros
        .iter()
        .map(|entry| MacroDto { id: entry.id, name: entry.name.clone(), description: entry.description.clone(), acronym: entry.acronym.clone(), icon: entry.icon.clamp(1, 7), lines: entry.commands.iter().map(|command| killer_bot_core::macros::line_of(command, names)).collect() })
        .collect()
}

pub fn shortcut_dtos(world: &killer_world::World, names: &dyn Names, skills: &[killer_bot_core::snapshot::SkillView]) -> Vec<ShortcutDto> {
    use l2_net::game::packets::ShortcutKind;
    let blank = |slot: usize, kind: &'static str, id: i32| ShortcutDto { slot, kind, id, item_id: 0, level: 0, name: String::new(), icon: String::new(), count: None, ready: true, acronym: String::new(), shot: false, auto: false };
    world
        .shortcuts
        .iter()
        .filter_map(|shortcut| {
            let slot = usize::try_from(shortcut.slot).ok()?;
            Some(match shortcut.kind {
                ShortcutKind::Item { object_id } => {
                    let item = world.inventory.find(object_id);
                    ShortcutDto { item_id: item.map_or(0, |item| item.item_id), name: item.map(|item| names.item(item.item_id)).unwrap_or_default(), count: item.filter(|item| item.count > 1).map(|item| item.count), shot: item.is_some_and(|item| killer_ui::shortcut::is_soulshot_item(item.item_id)), auto: item.is_some_and(|item| world.combat.auto_soulshots.get(&item.item_id).copied().unwrap_or(false)), ..blank(slot, "item", object_id) }
                }
                ShortcutKind::Skill { skill_id, level } => ShortcutDto { level, name: names.skill(skill_id, level), ready: skills.iter().find(|skill| skill.id == skill_id).is_none_or(|skill| skill.ready), ..blank(slot, "skill", skill_id) },
                ShortcutKind::Action { id } => ShortcutDto { name: names.action_name(id).unwrap_or_default(), icon: names.action_icon(id).unwrap_or_default(), ..blank(slot, "action", id) },
                ShortcutKind::Macro { id } => {
                    let entry = world.macros.macros.iter().find(|entry| entry.id == id);
                    ShortcutDto { name: entry.map(|entry| entry.name.clone()).unwrap_or_default(), acronym: entry.map(|entry| entry.acronym.clone()).unwrap_or_default(), level: entry.map_or(1, |entry| i32::from(entry.icon.clamp(1, 7))), ..blank(slot, "macro", id) }
                }
                ShortcutKind::Recipe { id } => blank(slot, "recipe", id),
                ShortcutKind::Other { .. } => return None,
            })
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HennaDto {
    pub slots: i32,
    pub used: usize,
    pub bonuses: Vec<(&'static str, i32)>,
}

pub fn henna_dto(world: &killer_world::World) -> Option<HennaDto> {
    let info = world.henna.info.as_ref()?;
    let bonuses = [("STR", info.str_), ("DEX", info.dex), ("CON", info.con), ("INT", info.int_), ("WIT", info.wit), ("MEN", info.men)].into_iter().filter(|(_, value)| *value != 0).map(|(name, value)| (name, i32::from(value))).collect();
    Some(HennaDto { slots: info.slots, used: info.symbols.len(), bonuses })
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FriendDto {
    pub name: String,
    pub online: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QuestionDto {
    pub index: usize,
    pub kind: &'static str,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TargetInfoDto {
    pub name: String,
    pub title: String,
    pub object_id: i32,
    pub npc_id: Option<i32>,
    pub npc_name: Option<String>,
    pub attackable: Option<bool>,
    pub hp: Option<GaugeDto>,
    pub dead: bool,
    pub distance: i32,
    pub position: [i32; 3],
    pub clan_id: i32,
    pub clan: Option<String>,
    pub ally: Option<String>,
    pub pvp_flag: i32,
    pub karma: i32,
    pub abnormal_mask: String,
    pub kind: &'static str,
    pub level: i32,
    pub class_name: Option<String>,
    pub mp: Option<GaugeDto>,
    pub cp: Option<GaugeDto>,
    pub equipment: Vec<EquipDto>,
    pub name_color: String,
    pub hero: bool,
    pub running: bool,
    pub sitting: bool,
    pub in_combat: bool,
    pub store: bool,
    pub run_speed: i32,
    pub p_atk_speed: i32,
    pub m_atk_speed: i32,
    pub recommendations: i32,
    pub casting: Option<String>,
    pub target_name: Option<String>,
    pub item_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EquipDto {
    pub slot: &'static str,
    pub item_id: i32,
    pub name: String,
    pub add_name: String,
    pub grade: Option<&'static str>,
    pub known: bool,
}

const EQUIP_LABELS: [&str; 11] = ["Underwear", "Head", "Right hand", "Left hand", "Gloves", "Chest", "Legs", "Feet", "Back", "Two-handed", "Hair"];

fn equip_dto(slot: &'static str, item_id: i32, names: &dyn Names) -> EquipDto {
    EquipDto { slot, item_id, name: names.item(item_id), add_name: names.item_add_name(item_id), grade: names.item_grade(item_id), known: names.is_known_item(item_id) }
}

fn equipment_of(kind: &killer_world::EntityKind, names: &dyn Names) -> Vec<EquipDto> {
    match kind {
        killer_world::EntityKind::Player { equipped, .. } => equipped.iter().zip(EQUIP_LABELS).map(|(id, label)| if *id > 0 { equip_dto(label, *id, names) } else { EquipDto { slot: label, item_id: 0, name: String::new(), add_name: String::new(), grade: None, known: false } }).collect(),
        killer_world::EntityKind::Npc { rhand, chest, lhand, .. } => [(*rhand, "Right hand"), (*chest, "Chest"), (*lhand, "Left hand")].into_iter().filter(|(id, _)| *id > 0).map(|(id, label)| equip_dto(label, id, names)).collect(),
        killer_world::EntityKind::Item { .. } => Vec::new(),
    }
}

fn target_level(world: &killer_world::World, known: i32, mine: i32) -> i32 {
    if known > 0 {
        return known;
    }
    world.target_color.map_or(0, |color| (mine - i32::from(color)).max(0))
}

pub fn target_info(world: &killer_world::World, names: &dyn Names) -> Option<TargetInfoDto> {
    let target = world.targeted()?;
    let me = world.me.as_ref()?;
    let (npc_id, attackable) = match target.kind {
        killer_world::EntityKind::Npc { npc_id, attackable, .. } => (Some(npc_id), Some(attackable)),
        _ => (None, None),
    };
    Some(TargetInfoDto {
        name: target.name.clone(),
        title: target.title.clone(),
        object_id: target.object_id,
        npc_id,
        npc_name: npc_id.map(|id| names.npc(id)),
        attackable,
        hp: target.hp.map(|(current, max)| GaugeDto { current, max }),
        dead: target.dead,
        distance: target.position.distance(me.position) as i32,
        position: target.position.to_array().map(|value| value as i32),
        clan_id: target.clan_id,
        clan: world.pledge_name(target.clan_id).map(str::to_owned),
        ally: world.ally_name(target.clan_id).map(str::to_owned),
        pvp_flag: target.pvp_flag,
        karma: target.karma,
        abnormal_mask: format!("{:#x}", target.abnormal_effect),
        kind: match target.kind {
            killer_world::EntityKind::Player { .. } => "player",
            killer_world::EntityKind::Npc { .. } => "npc",
            killer_world::EntityKind::Item { .. } => "item",
        },
        level: target_level(world, target.level, me.level),
        class_name: match target.kind {
            killer_world::EntityKind::Player { class_id, .. } => Some(names.class(class_id)),
            _ => None,
        },
        mp: target.mp.map(|(current, max)| GaugeDto { current, max }),
        cp: target.cp.map(|(current, max)| GaugeDto { current, max }),
        equipment: equipment_of(&target.kind, names),
        name_color: format!("#{:02x}{:02x}{:02x}", target.name_color & 0xff, (target.name_color >> 8) & 0xff, (target.name_color >> 16) & 0xff),
        hero: matches!(target.kind, killer_world::EntityKind::Player { hero: true, .. }),
        running: target.running,
        sitting: target.sitting,
        in_combat: target.in_combat,
        store: target.store_type != 0,
        run_speed: target.run_speed,
        p_atk_speed: target.p_atk_speed,
        m_atk_speed: target.m_atk_speed,
        recommendations: target.recommendations,
        casting: target.casting.map(|casting| names.skill(casting.skill_id, casting.skill_level)),
        target_name: target.target.and_then(|id| world.entity(id)).map(|entity| entity.name.clone()).filter(|name| !name.is_empty()),
        item_count: match target.kind {
            killer_world::EntityKind::Item { count, .. } => Some(count),
            _ => None,
        },
    })
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NpcDto {
    pub object_id: i32,
    pub name: String,
    pub title: String,
    pub nodes: Vec<crate::html::HtmlNode>,
}

pub fn npc_dialog(world: &killer_world::World, snapshot: &Snapshot, names: &dyn Names) -> Option<NpcDto> {
    let message = world.npc_html.as_ref()?;
    let document = killer_ui::html::parse(&message.html);
    let near = snapshot.nearby.iter().find(|view| view.object_id == message.npc_object_id);
    let name = near.map(|view| nearby_name(view, names)).unwrap_or_default();
    let title = near.map(|view| view.title.clone()).unwrap_or_default();
    Some(NpcDto { object_id: message.npc_object_id, name, title, nodes: crate::html::convert(&document.body, message.npc_object_id) })
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PetActionDto {
    pub id: i32,
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogDto {
    pub skills: Vec<(i32, String)>,
    pub items: Vec<(i32, String)>,
    pub npcs: Vec<(i32, String)>,
    pub listed_items: Vec<(i32, String)>,
    pub classes: Vec<(i32, String)>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StaticDto {
    pub actions: Vec<ActionDto>,
    pub classes: Vec<(i32, String)>,
    pub examples: Vec<crate::scripts::ExampleDto>,
}

pub fn static_dto(data: Option<&crate::data::GameData>) -> StaticDto {
    let Some(data) = data else { return StaticDto { actions: Vec::new(), classes: Vec::new(), examples: crate::scripts::examples() } };
    let names = data.names();
    let actions = data
        .tables
        .actions
        .iter()
        .filter_map(|action| {
            let group = match action.category {
                1 => 0,
                2 => 1,
                3 => 2,
                _ => return None,
            };
            Some(ActionDto { id: action.id as i32, name: action.name.clone(), icon: action.icon.clone(), group })
        })
        .collect();
    let classes = (0..=140).filter_map(|id| Some((id, names.class(id))).filter(|(_, name)| !name.starts_with("Class #"))).collect();
    StaticDto { actions, classes, examples: crate::scripts::examples() }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionDto {
    pub id: i32,
    pub name: String,
    pub icon: String,
    pub group: u8,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorldDto {
    pub packets: crate::packet_tools::PacketsDto,
    pub script: crate::scripts::ScriptDto,
    pub options: serde_json::Value,
    pub saved_options: Option<serde_json::Value>,
    pub catalog: CatalogDto,
    pub windows: crate::windows::WindowsDto,
    pub pet_actions: Vec<PetActionDto>,
    pub npc: Option<NpcDto>,
    pub target_info: Option<TargetInfoDto>,
    pub inventory: Vec<ItemDto>,
    pub skills: Vec<SkillDto>,
    pub buffs: Vec<BuffDto>,
    pub party: Vec<MemberDto>,
    pub friends: Vec<FriendDto>,
    pub henna: Option<HennaDto>,
    pub ping: Option<u32>,
    pub macros: Vec<MacroDto>,
    pub macro_run: Option<MacroRunDto>,
    pub shortcuts: Vec<ShortcutDto>,
    pub clan: Option<ClanDto>,
    pub clan_id: i32,
    pub loot: Vec<LootDto>,
    pub gains: GainsDto,
    pub death: Option<DeathDto>,
    pub questions: Vec<QuestionDto>,
    pub bot_on: bool,
    pub character: CharacterDto,
    pub target: Option<TargetDto>,
    pub nearby: Vec<NearbyDto>,
    pub chat: Vec<ChatLineDto>,
    pub uptime: String,
    pub deaths: u32,
    pub adena_per_hour: i64,
    pub xp_per_hour: f32,
    pub version: &'static str,
}

pub fn screen_of(state: BotState) -> &'static str {
    match state {
        BotState::Connecting | BotState::EnteringWorld => "connecting",
        BotState::ChoosingServer => "servers",
        BotState::ChoosingCharacter => "characters",
        BotState::InWorld => "world",
        BotState::Disconnected => "disconnected",
    }
}

pub fn state_of(state: BotState) -> &'static str {
    match state {
        BotState::InWorld => "world",
        BotState::Disconnected => "offline",
        _ => "connecting",
    }
}

pub fn channel_name(channel: Channel) -> &'static str {
    match channel {
        Channel::All => "local",
        Channel::Shout => "shout",
        Channel::Whisper => "whisper",
        Channel::Party | Channel::PartyRoom => "party",
        Channel::Clan => "clan",
        Channel::Ally => "alliance",
        Channel::Trade => "trade",
        Channel::Hero => "hero",
        Channel::Announcement | Channel::System => "system",
        Channel::Unknown(_) => "local",
    }
}

pub fn channel_of_name(name: &str) -> Channel {
    match name {
        "trade" => Channel::Trade,
        "party" => Channel::Party,
        "clan" => Channel::Clan,
        "alliance" => Channel::Ally,
        "shout" => Channel::Shout,
        "whisper" => Channel::Whisper,
        "hero" => Channel::Hero,
        _ => Channel::All,
    }
}

fn kind_name(kind: NearbyKind) -> &'static str {
    match kind {
        NearbyKind::Monster => "monster",
        NearbyKind::Npc => "npc",
        NearbyKind::Player => "player",
        NearbyKind::Item => "item",
    }
}

fn nearby_name(view: &killer_bot_core::snapshot::NearbyView, names: &dyn Names) -> String {
    if !view.name.is_empty() {
        view.name.clone()
    } else if view.kind == NearbyKind::Item {
        names.item(view.item_id)
    } else if view.npc_id != 0 {
        names.npc(view.npc_id)
    } else {
        format!("#{}", view.object_id)
    }
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_secs())
}

pub fn chat_lines(chat: &ChatBoxes, stamps: &[u64], names: &dyn Names) -> Vec<ChatLineDto> {
    use killer_bot_core::chat::{system_hidden, system_text};
    let mut out = Vec::new();
    let entries = chat.all();
    let missing = entries.len().saturating_sub(stamps.len());
    for (index, entry) in entries.iter().enumerate() {
        let at = index.checked_sub(missing).and_then(|position| stamps.get(position)).copied().unwrap_or(0);
        let channel = channel_name(entry.channel);
        if let Some((id, args)) = &entry.system
            && let Some(message) = system_text(*id, args, names)
        {
            if !system_hidden(message.group) {
                out.push(ChatLineDto { at, channel, sender: String::new(), text: message.text });
            }
            continue;
        }
        if let Some((string_id, message_id)) = entry.tables
            && let (Some(sender), Some(message)) = (names.system_string(string_id), names.system_message(message_id))
        {
            out.push(ChatLineDto { at, channel, sender, text: message.text });
            continue;
        }
        let sender = entry.sender.strip_prefix("npc ").and_then(|id| id.parse::<i32>().ok()).map_or_else(|| entry.sender.clone(), |id| names.npc(if id >= killer_bot_core::chat::NPC_NAME_BASE { id - killer_bot_core::chat::NPC_NAME_BASE } else { id }));
        out.push(ChatLineDto { at, channel, sender, text: entry.text.clone() });
    }
    out
}

pub struct Extras {
    pub macros: Vec<MacroDto>,
    pub macro_run: Option<MacroRunDto>,
    pub shortcuts: Vec<ShortcutDto>,
    pub ping: Option<u32>,
    pub friends: Vec<FriendDto>,
    pub henna: Option<HennaDto>,
    pub chat_stamps: Vec<u64>,
    pub clan_names: std::collections::HashMap<i32, String>,
    pub packets: crate::packet_tools::PacketsDto,
    pub script: crate::scripts::ScriptDto,
    pub options: serde_json::Value,
    pub saved_options: Option<serde_json::Value>,
    pub windows: crate::windows::WindowsDto,
    pub pet_actions: Vec<PetActionDto>,
    pub npc: Option<NpcDto>,
    pub target_info: Option<TargetInfoDto>,
    pub bot_on: bool,
    pub uptime: String,
    pub deaths: u32,
}

pub fn world(snapshot: &Snapshot, chat: &ChatBoxes, names: &dyn Names, extras: &Extras) -> WorldDto {
    let near = |id: i32| snapshot.nearby.iter().find(|view| view.object_id == id);
    let target = snapshot.target.as_ref().map(|target| {
        let view = near(target.object_id);
        TargetDto {
            id: target.object_id,
            name: view.map_or_else(|| if target.name.is_empty() { format!("#{}", target.object_id) } else { target.name.clone() }, |view| if target.name.is_empty() { nearby_name(view, names) } else { target.name.clone() }),
            kind: view.map_or("npc", |view| kind_name(view.kind)),
            hp_percent: target.hp.percent,
            dead: target.dead,
            distance: view.map_or(0, |view| view.distance as i32),
        }
    });
    let st = &snapshot.stats;
    WorldDto {
        packets: extras.packets.clone(),
        script: extras.script.clone(),
        options: extras.options.clone(),
        saved_options: extras.saved_options.clone(),
        catalog: CatalogDto {
            skills: snapshot.skills.iter().filter(|skill| !skill.passive).map(|skill| (skill.id, names.skill(skill.id, skill.level))).collect(),
            items: snapshot.inventory.iter().map(|item| (item.item_id, names.item(item.item_id))).collect(),
            npcs: ids_at(&extras.options, &["do_not", "npcs"]).into_iter().map(|id| (id, names.npc(id))).collect(),
            listed_items: ids_at(&extras.options, &["do_not", "items"]).into_iter().map(|id| (id, names.item(id))).collect(),
            classes: ids_at_key(&extras.options, &["player_sorting", "priorities"], "class_id").into_iter().map(|id| (id, names.class(id))).collect(),
        },
        windows: extras.windows.clone(),
        pet_actions: extras.pet_actions.clone(),
        npc: extras.npc.clone(),
        target_info: extras.target_info.clone(),
        inventory: snapshot.inventory.iter().map(|item| ItemDto { id: item.object_id, item_id: item.item_id, name: names.item(item.item_id), add_name: names.item_add_name(item.item_id), grade: names.item_grade(item.item_id), known: names.is_known_item(item.item_id), count: item.count, equipped: item.equipped, body_part: item.body_part }).collect(),
        skills: snapshot.skills.iter().map(|skill| SkillDto { id: skill.id, level: skill.level, name: names.skill(skill.id, skill.level), passive: skill.passive, ready: skill.ready }).collect(),
        buffs: snapshot.buffs.iter().map(|buff| BuffDto { skill_id: buff.skill_id, level: i32::from(buff.level), name: names.skill(buff.skill_id, i32::from(buff.level)), seconds: buff.seconds }).collect(),
        friends: extras.friends.clone(),
        henna: extras.henna.clone(),
        ping: extras.ping,
        macros: extras.macros.clone(),
        macro_run: extras.macro_run.clone(),
        shortcuts: extras.shortcuts.clone(),
        party: snapshot.party.iter().map(|member| MemberDto { id: member.object_id, name: member.name.clone(), hp: GaugeDto { current: member.hp.current, max: member.hp.max }, mp: GaugeDto { current: member.mp.current, max: member.mp.max } }).collect(),
        clan: snapshot.clan.as_ref().map(|clan| ClanDto {
            name: clan.name.clone(),
            leader: clan.leader.clone(),
            level: clan.level,
            castle_id: clan.castle_id,
            castle: names.castle(clan.castle_id),
            hideout_id: clan.hideout_id,
            ally: clan.ally.clone(),
            at_war: clan.at_war,
            members: clan.members.iter().map(|member| ClanMemberDto { name: member.name.clone(), level: member.level, class_name: names.class(member.class_id), online: member.online }).collect(),
        }),
        clan_id: snapshot.clan_id,
        loot: snapshot.loot.iter().map(|row| LootDto { source: row.source.clone(), item_id: row.item_id, name: names.item(row.item_id), add_name: names.item_add_name(row.item_id), count: row.count }).collect(),
        gains: GainsDto { exp_per_hour: snapshot.gains.exp_per_hour, sp_per_hour: snapshot.gains.sp_per_hour, adena_per_hour: snapshot.gains.adena_per_hour, adena_total: snapshot.gains.adena_total, bad_mobs: snapshot.bad_mobs },
        death: snapshot.death.map(|death| DeathDto { town: death.town, hideout: death.hideout, castle: death.castle, siege_hq: death.siege_hq, fortress: death.fortress }),
        questions: snapshot
            .questions
            .iter()
            .map(|question| QuestionDto {
                index: question.index,
                kind: match question.kind {
                    killer_bot_core::snapshot::QuestionKind::Party => "party",
                    killer_bot_core::snapshot::QuestionKind::Pledge => "pledge",
                    killer_bot_core::snapshot::QuestionKind::Alliance => "alliance",
                    killer_bot_core::snapshot::QuestionKind::Trade => "trade",
                    killer_bot_core::snapshot::QuestionKind::Friend => "friend",
                    killer_bot_core::snapshot::QuestionKind::Confirm => "confirm",
                },
                text: question.text.clone(),
            })
            .collect(),
        bot_on: extras.bot_on,
        character: CharacterDto {
            name: snapshot.name.clone(),
            title: snapshot.title.clone(),
            class_name: names.class(snapshot.class_id),
            level: snapshot.level,
            hp: GaugeDto { current: snapshot.hp.current, max: snapshot.hp.max },
            mp: GaugeDto { current: snapshot.mp.current, max: snapshot.mp.max },
            cp: GaugeDto { current: snapshot.cp.current, max: snapshot.cp.max },
            xp_percent: killer_ui::exp::exp_percent(snapshot.level, i64::from(snapshot.exp)).clamp(0.0, 100.0) as f32,
            karma: snapshot.karma,
            pvp: snapshot.pvp_kills,
            pk: snapshot.pk_kills,
            recommend_have: snapshot.recommend_have,
            recommend_left: snapshot.recommend_left,
            adena: snapshot.adena,
            position: snapshot.position,
            stats: StatsDto {
                p_atk: st.p_atk,
                m_atk: st.m_atk,
                p_def: st.p_def,
                m_def: st.m_def,
                accuracy: st.accuracy,
                evasion: st.evasion,
                critical: st.critical,
                speed: snapshot.run_speed,
                atk_spd: st.atk_spd,
                cast_spd: st.cast_spd,
                str: st.str_,
                dex: st.dex,
                con: st.con,
                int: st.int_,
                wit: st.wit,
                men: st.men,
                sp: st.sp,
                load: st.load,
                max_load: st.max_load,
            },
        },
        target,
        nearby: snapshot
            .nearby
            .iter()
            .map(|view| NearbyDto {
                id: view.object_id,
                name: nearby_name(view, names),
                title: view.title.clone(),
                npc_id: view.npc_id,
                item_id: view.item_id,
                count: view.count,
                add_name: if view.kind == NearbyKind::Item { names.item_add_name(view.item_id) } else { String::new() },
                class_name: if view.kind == NearbyKind::Player { names.class(view.class_id) } else { String::new() },
                clan_id: view.clan_id,
                clan: extras.clan_names.get(&view.clan_id).cloned().unwrap_or_default(),
                kind: kind_name(view.kind),
                x: view.position[0],
                y: view.position[1],
                z: view.position[2],
                distance: view.distance,
                dead: view.dead,
            })
            .collect(),
        chat: chat_lines(chat, &extras.chat_stamps, names),
        uptime: extras.uptime.clone(),
        deaths: extras.deaths,
        adena_per_hour: snapshot.gains.adena_per_hour,
        xp_per_hour: killer_ui::exp::level_bounds(snapshot.level).map_or(0.0, |(start, next)| snapshot.gains.exp_per_hour as f32 / (next - start).max(1) as f32 * 100.0),
        version: env!("CARGO_PKG_VERSION"),
    }
}

fn ids_at(options: &serde_json::Value, path: &[&str]) -> Vec<i32> {
    let mut current = options;
    for key in path {
        current = &current[*key];
    }
    current.as_array().map(|list| list.iter().filter_map(|value| value.as_i64().and_then(|id| i32::try_from(id).ok())).collect()).unwrap_or_default()
}

fn ids_at_key(options: &serde_json::Value, path: &[&str], key: &str) -> Vec<i32> {
    let mut current = options;
    for part in path {
        current = &current[*part];
    }
    current.as_array().map(|list| list.iter().filter_map(|value| value[key].as_i64().and_then(|id| i32::try_from(id).ok())).collect()).unwrap_or_default()
}

pub fn uptime_text(seconds: u64) -> String {
    format!("{:02}:{:02}:{:02}", seconds / 3600, (seconds / 60) % 60, seconds % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uptime_is_clock_formatted() {
        assert_eq!(uptime_text(0), "00:00:00");
        assert_eq!(uptime_text(9678), "02:41:18");
    }

    #[test]
    fn every_bot_state_maps_to_a_screen_and_a_dot() {
        assert_eq!(screen_of(BotState::InWorld), "world");
        assert_eq!(screen_of(BotState::ChoosingServer), "servers");
        assert_eq!(state_of(BotState::Disconnected), "offline");
        assert_eq!(state_of(BotState::EnteringWorld), "connecting");
    }

    #[test]
    fn target_level_comes_from_the_server_difference_when_the_entity_has_none() {
        let mut world = killer_world::World::default();
        assert_eq!(target_level(&world, 0, 78), 0);
        world.target_color = Some(-2);
        assert_eq!(target_level(&world, 0, 78), 80);
        world.target_color = Some(5);
        assert_eq!(target_level(&world, 0, 78), 73);
        world.target_color = Some(200);
        assert_eq!(target_level(&world, 0, 78), 0);
        assert_eq!(target_level(&world, 64, 78), 64);
    }

    #[test]
    fn player_equipment_lists_every_visible_slot_and_marks_the_empty_ones() {
        struct Plain;
        impl Names for Plain {
            fn item(&self, id: i32) -> String {
                format!("item {id}")
            }
            fn skill(&self, _id: i32, _level: i32) -> String {
                String::new()
            }
            fn npc(&self, _id: i32) -> String {
                String::new()
            }
            fn class(&self, _id: i32) -> String {
                String::new()
            }
            fn find_skill(&self, _name: &str) -> Option<i32> {
                None
            }
        }
        let mut equipped = [0; 11];
        equipped[2] = 25;
        let kind = killer_world::EntityKind::Player { class_id: 0, race: 0, sex: 0, equipped, hair_style: 0, hair_color: 0, face: 0, enchant_effect: 0, hero: false };
        let pieces = equipment_of(&kind, &Plain);
        assert_eq!(pieces.len(), EQUIP_LABELS.len());
        assert_eq!(pieces.iter().filter(|piece| piece.item_id > 0).count(), 1);
        assert_eq!(pieces[2].slot, "Right hand");
        assert_eq!(pieces[0].item_id, 0);
    }

    #[test]
    fn channels_round_trip_through_their_names() {
        for name in ["trade", "party", "clan", "alliance", "shout", "whisper", "hero", "local"] {
            assert_eq!(channel_name(channel_of_name(name)), if name == "local" { "local" } else { name });
        }
    }
}
