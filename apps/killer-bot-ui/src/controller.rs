use crate::create_char;
use crate::data::DataLoader;
use crate::dto::{self, CharacterEntryDto, Extras, Frame, SavedDto, ServerDto, SessionDto};
use crate::profiles;
use crate::packet_tools::{self, PacketUi};
use crate::scripts;
use crate::secrets::{SecretStore, secret_key};
use killer_bot_core::clock::SystemClock;
use killer_bot_core::command::UserCommand;
use killer_bot_core::macros::{MacroEffect, MacroRunner};
use killer_bot_core::manager::BotManager;
use killer_bot_core::names::{IdNames, Names};
use killer_bot_core::profile::Profile;
use killer_bot_core::session::{BotEvent, BotSession, BotState};
use killer_bot_core::transport::{NetTransport, Transport};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub type Connector = Box<dyn Fn(&Profile, String) -> Box<dyn Transport> + Send>;

pub fn network_connector() -> Connector {
    Box::new(|profile, password| Box::new(NetTransport::connect(profile.server.host.clone(), profile.server.port, profile.account.clone(), password)))
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Command {
    ToggleBot,
    SelectTarget { id: i32 },
    MoveTo { x: i32, y: i32, z: i32 },
    Say { channel: String, text: String },
    SelectSession { id: u32 },
    CloseSession { id: u32 },
    AddAccount { host: String, port: u16, account: String, password: String, character: String, remember: bool },
    ConnectSaved { label: String },
    ForgetSaved { label: String },
    RetryData,
    ChooseServer { id: u8 },
    ChooseCharacter { slot: usize },
    UseItem { id: i32 },
    DropItem { id: i32, count: i32 },
    DestroyItem { id: i32, count: i32 },
    Crystallize { id: i32, count: i32 },
    UseSkill { id: i32 },
    RestartPoint { kind: i32 },
    Answer { index: usize, accept: bool },
    RequestClan,
    ClearStats,
    CancelTarget,
    Sit,
    Stand,
    Restart,
    Logout,
    IgnoreItem { item_id: i32 },
    IgnoreNpc { npc_id: i32 },
    Blacklist { id: i32 },
    PartyInvite { name: String },
    PartyLeave,
    PartyKick { name: String },
    PartyLeader { name: String },
    PartyDismiss,
    ClanInvite { id: i32 },
    FriendInvite { name: String },
    TradeRequest { id: i32 },
    NpcAction { action: String },
    Buy { list_id: i32, orders: Vec<(i32, i32)> },
    Sell { list_id: i32, orders: Vec<(i32, i32, i32)> },
    CloseShop,
    MultiSellChoose { list_id: i32, entry_id: i32, amount: i32 },
    CloseMultisell,
    Warehouse { deposit: bool, orders: Vec<(i32, i32)> },
    CloseWarehouse,
    LearnSkill { skill_id: i32, level: i32, mode: i32 },
    CloseLearn,
    TradeAdd { object_id: i32, count: i32 },
    TradeConfirm,
    TradeCancel,
    RequestQuests,
    AbortQuest { quest_id: i32 },
    StoreBuy { orders: Vec<(i32, i32)> },
    StoreSell { orders: Vec<(i32, i32)> },
    StoreSetSell { package_sale: bool, offers: Vec<(i32, i32, i32)> },
    StoreSetBuy { offers: Vec<(i32, i32, i32)> },
    StoreMessage { buy: bool, text: String },
    CloseStore,
    PetGive { object_id: i32, count: i32 },
    PetTake { object_id: i32, count: i32 },
    Action { id: i32 },
    Talk { id: i32 },
    RequestFriends,
    DeleteFriend { name: String },
    RunMacro { id: i32 },
    StopMacro,
    UseShortcut { slot: usize },
    RegisterShortcut { slot: usize, kind: String, id: i32 },
    DeleteShortcut { slot: usize },
    ToggleAutoShot { item_id: i32, enable: bool },
    SaveMacro { id: i32, name: String, acronym: String, description: String, icon: u8, lines: Vec<String> },
    DeleteMacro { id: i32 },
    SetOptions { options: serde_json::Value, save: bool },
    AddPolygonPoint { x: i32, y: i32 },
    RunScript { text: String },
    StopScript,
    PauseScript { paused: bool },
    SaveScript { name: String, text: String },
    LoadScript { name: String },
    RequestNewCharacter,
    DeleteCharacter { slot: usize },
    CreateCharacter { name: String, race: usize, female: bool, mage: bool, hair_style: i32, hair_color: i32, face: i32 },
    PkRecord { on: bool },
    PkClear,
    PkSave { only_visible: bool },
    PkLoad,
    PkSelect { number: u64 },
    PkSearch { query: String, client: bool, by_name: bool, direction: String },
    PkInject { hex: String, to_client: bool },
    PkParse { format: String },
    PkHelp,
    PkView { filter_on: bool, hide_server: bool, hide_client: bool },
    PkFilters { server: Vec<u16>, client: Vec<u16>, whitelist: bool },
    PkFiltersSave,
    PkFiltersLoad,
    PkConvert { text: String },
    Bypass { command: String },
    CloseNpc,
}

struct Account {
    id: u32,
    label: String,
    saved: Option<String>,
    started: Instant,
    deaths: u32,
    bot_lines: Vec<(u64, String)>,
    chat_stamps: std::collections::VecDeque<u64>,
    chat_seen: u64,
    macro_runner: MacroRunner,
    script_lines: Vec<String>,
}

struct Saved {
    label: String,
    profile: Profile,
    has_password: bool,
}

pub struct Controller {
    manager: BotManager,
    accounts: Vec<Account>,
    selected: Option<u32>,
    next_id: u32,
    connector: Connector,
    names: Box<dyn Names>,
    names_ready: bool,
    status: String,
    store: Box<dyn SecretStore>,
    profile_dir: Option<PathBuf>,
    saved: Vec<Saved>,
    loader: Option<Arc<Mutex<DataLoader>>>,
    script_message: String,
    talk: Option<(u32, i32, Instant)>,
    script_loaded: Option<scripts::LoadedDto>,
    script_dir: Option<PathBuf>,
    config_dir: Option<PathBuf>,
    packet_ui: PacketUi,
}

impl Controller {
    pub fn new(connector: Connector, store: Box<dyn SecretStore>, profile_dir: Option<PathBuf>, loader: Option<Arc<Mutex<DataLoader>>>) -> Self {
        let mut controller = Self {
            manager: BotManager::new(),
            accounts: Vec::new(),
            selected: None,
            next_id: 1,
            connector,
            names: Box::new(IdNames),
            names_ready: false,
            status: String::new(),
            store,
            profile_dir,
            saved: Vec::new(),
            loader,
            script_message: String::new(),
            talk: None,
            script_loaded: None,
            script_dir: crate::secrets::config_dir(|name| std::env::var(name).ok()).map(|dir| dir.join("scripts")),
            config_dir: crate::secrets::config_dir(|name| std::env::var(name).ok()),
            packet_ui: PacketUi::default(),
        };
        controller.reload_saved();
        controller
    }

    fn reload_saved(&mut self) {
        let Some(dir) = self.profile_dir.clone() else { return };
        self.saved = profiles::load_profiles(&dir)
            .into_iter()
            .filter_map(|(label, result)| result.ok().map(|profile| (label, profile)))
            .map(|(label, profile)| Saved { label, profile, has_password: true })
            .collect();
    }

    pub fn apply(&mut self, command: Command) {
        self.status.clear();
        match command {
            Command::SelectSession { id } => {
                if self.accounts.iter().any(|account| account.id == id) {
                    self.selected = Some(id);
                }
            }
            Command::CloseSession { id } => self.close(id),
            Command::AddAccount { host, port, account, password, character, remember } => self.add(host, port, account, password, character, remember),
            Command::ConnectSaved { label } => self.connect_saved(&label),
            Command::ForgetSaved { label } => self.forget_saved(&label),
            Command::CreateCharacter { name, race, female, mage, hair_style, hair_color, face } => match create_char::build(&name, create_char::Choice { race, female, mage, hair_style, hair_color, face }) {
                Ok(new) => {
                    if let Some(id) = self.selected
                        && let Some(Err(refusal)) = self.manager.session_mut(id).map(|session| session.command(UserCommand::CreateCharacter(Box::new(new))))
                    {
                        self.status = format!("comando recusado: {refusal:?}");
                    }
                }
                Err(message) => self.status = message,
            },
            Command::SetOptions { options, save } => self.set_options(options, save),
            Command::RunScript { text } => {
                let message = match self.selected.and_then(|id| self.manager.session_mut(id)) {
                    Some(session) => match session.load_script(&text) {
                        Ok(()) => "rodando".to_owned(),
                        Err(error) => error.to_string(),
                    },
                    None => String::new(),
                };
                if message == "rodando"
                    && let Some(account) = self.accounts.iter_mut().find(|account| self.selected == Some(account.id))
                {
                    account.script_lines.push(format!("[{}] script iniciado", dto::uptime_text(account.started.elapsed().as_secs())));
                }
                self.script_message = message;
            }
            Command::StopScript => {
                self.with_session(BotSession::stop_script);
                self.script_message = "script parado".to_owned();
            }
            Command::PauseScript { paused } => self.with_session(|session| session.pause_script(paused)),
            Command::SaveScript { name, text } => {
                self.script_message = match &self.script_dir {
                    Some(dir) => scripts::save_script(dir, &name, &text).map_or_else(|error| error, |path| format!("salvo em {}", path.display())),
                    None => "sem pasta de configuração".to_owned(),
                };
            }
            Command::LoadScript { name } => match self.script_dir.as_ref().map(|dir| scripts::load_script(dir, &name)) {
                Some(Ok(text)) => {
                    let seq = self.script_loaded.as_ref().map_or(1, |loaded| loaded.seq + 1);
                    self.script_loaded = Some(scripts::LoadedDto { seq, name: scripts::clean_name(&name), text });
                    self.script_message = "script carregado".to_owned();
                }
                Some(Err(error)) => self.script_message = error,
                None => self.script_message = "sem pasta de configuração".to_owned(),
            },
            Command::PkRecord { .. } | Command::PkClear | Command::PkSave { .. } | Command::PkLoad | Command::PkSelect { .. } | Command::PkSearch { .. } | Command::PkInject { .. } | Command::PkParse { .. } | Command::PkHelp | Command::PkView { .. } | Command::PkFilters { .. } | Command::PkFiltersSave | Command::PkFiltersLoad | Command::PkConvert { .. } => self.packets_command(command),
            Command::AddPolygonPoint { x, y } => self.with_session(|session| session.update_options(|options| options.polygon.points.push(killer_bot_core::options::Point { x, y }))),
            Command::CloseNpc => self.with_session(|session| session.close_npc_dialog()),
            Command::CloseShop => self.with_session(BotSession::close_shop),
            Command::CloseMultisell => self.with_session(BotSession::close_multisell),
            Command::CloseWarehouse => self.with_session(BotSession::close_warehouse),
            Command::CloseLearn => self.with_session(BotSession::close_learn),
            Command::CloseStore => self.with_session(BotSession::close_store),
            Command::RetryData => {
                if let Some(loader) = &self.loader
                    && let Ok(mut loader) = loader.lock()
                {
                    loader.retry();
                }
            }
            Command::RunMacro { id: macro_id } => self.macro_step(|runner, now, world, names, out| runner.start(now, world, names, macro_id, out)),
            Command::Action { id: action } => self.macro_step(|_, _, world, names, out| killer_bot_core::macros::use_action(world, names, action, out)),
            Command::StopMacro => {
                let Some(id) = self.selected else { return };
                if let Some(account) = self.accounts.iter_mut().find(|account| account.id == id) {
                    account.macro_runner.stop();
                }
            }
            Command::UseShortcut { slot } => self.macro_step(|runner, now, world, names, out| runner.use_shortcut(now, world, names, slot, out)),
            Command::RegisterShortcut { slot, kind, id: target } => {
                let (kind, character_type) = match kind.as_str() {
                    "skill" => (l2_net::game::SHORTCUT_SKILL, 1),
                    "action" => (l2_net::game::SHORTCUT_ACTION, killer_ui::actions::shortcut_character_type(target)),
                    "macro" => (l2_net::game::SHORTCUT_MACRO, 1),
                    "recipe" => (l2_net::game::SHORTCUT_RECIPE, 1),
                    _ => (l2_net::game::SHORTCUT_ITEM, 1),
                };
                self.send_net(killer_world::NetCommand::ShortcutReg { kind, slot: slot as i32, id: target, character_type });
            }
            Command::ToggleAutoShot { item_id, enable } => self.send_net(killer_world::NetCommand::AutoSoulShot { item_id, enable }),
            Command::DeleteShortcut { slot } => self.send_net(killer_world::NetCommand::ShortcutDel { slot: slot as i32 }),
            Command::DeleteMacro { id: macro_id } => self.send_net(killer_world::NetCommand::DeleteMacro { id: macro_id }),
            Command::SaveMacro { id: macro_id, name, acronym, description, icon, lines } => self.save_macro(macro_id, name, acronym, description, icon, lines),
            Command::Talk { id: object_id } => {
                let Some(id) = self.selected else { return };
                let Some(session) = self.manager.session_mut(id) else { return };
                let targeted = session.world().targeted().is_some_and(|entity| entity.object_id == object_id);
                if let Err(refusal) = session.command(UserCommand::SelectTarget(object_id)) {
                    self.status = format!("comando recusado: {refusal:?}");
                } else if !targeted {
                    self.talk = Some((id, object_id, Instant::now()));
                }
            }
            other => {
                let Some(id) = self.selected else { return };
                if let Some(user) = self.user_command(id, other)
                    && let Some(Err(refusal)) = self.manager.session_mut(id).map(|session| session.command(user))
                {
                    self.status = format!("comando recusado: {refusal:?}");
                }
            }
        }
    }

    fn send_net(&mut self, command: killer_world::NetCommand) {
        let Some(id) = self.selected else { return };
        if let Some(Err(refusal)) = self.manager.session_mut(id).map(|session| session.command(UserCommand::Net(command))) {
            self.status = format!("comando recusado: {refusal:?}");
        }
    }

    fn macro_step(&mut self, step: impl FnOnce(&mut MacroRunner, Duration, &killer_world::World, &dyn Names, &mut Vec<MacroEffect>)) {
        let Some(id) = self.selected else { return };
        let Some(account) = self.accounts.iter_mut().find(|account| account.id == id) else { return };
        let Some(session) = self.manager.session(id) else { return };
        let mut effects = Vec::new();
        step(&mut account.macro_runner, account.started.elapsed(), session.world(), self.names.as_ref(), &mut effects);
        self.run_effects(id, effects);
    }

    fn run_effects(&mut self, id: u32, effects: Vec<MacroEffect>) {
        for effect in effects {
            match effect {
                MacroEffect::Net(command) => {
                    if let Some(session) = self.manager.session_mut(id) {
                        let _ = session.command(UserCommand::Net(command));
                    }
                }
                MacroEffect::Select(object_id) => {
                    if let Some(session) = self.manager.session_mut(id) {
                        let _ = session.command(UserCommand::SelectTarget(object_id));
                    }
                }
                MacroEffect::Log(text) => {
                    if let Some(account) = self.accounts.iter_mut().find(|account| account.id == id) {
                        account.bot_lines.push((dto::now_secs(), format!("macro: {text}")));
                    }
                }
            }
        }
    }

    fn save_macro(&mut self, macro_id: i32, name: String, acronym: String, description: String, icon: u8, lines: Vec<String>) {
        use l2_net::game::packets::{MACRO_ACRONYM_MAX_CHARS, MACRO_DESCRIPTION_MAX_CHARS, MACRO_NAME_MAX_CHARS};
        let Some(id) = self.selected else { return };
        let Some(session) = self.manager.session(id) else { return };
        let (name, acronym) = (name.trim().to_owned(), acronym.trim().to_owned());
        if name.is_empty() {
            self.status = "informe o nome do macro".to_owned();
            return;
        }
        if acronym.is_empty() {
            self.status = "informe a sigla do macro".to_owned();
            return;
        }
        let existing = session.world().macros.macros.iter().find(|entry| entry.id == macro_id).cloned();
        let names = self.names.as_ref();
        let mut commands = Vec::new();
        for line in lines.iter().take(killer_bot_core::macros::COMMAND_LINES) {
            let line = line.trim();
            if line.is_empty() {
                break;
            }
            let index = commands.len() as u8;
            let kept = existing.as_ref().and_then(|entry| entry.commands.iter().find(|command| killer_bot_core::macros::line_of(command, names) == line));
            let command = match kept {
                Some(command) => Some(l2_net::game::packets::MacroCommand { index, ..command.clone() }),
                None => killer_bot_core::macros::command_of_line(line, index, names),
            };
            commands.extend(command);
        }
        let entry = l2_net::game::packets::Macro {
            id: macro_id,
            name: name.chars().take(MACRO_NAME_MAX_CHARS).collect(),
            description: description.chars().take(MACRO_DESCRIPTION_MAX_CHARS).collect(),
            acronym: acronym.chars().take(MACRO_ACRONYM_MAX_CHARS).collect(),
            icon: icon.clamp(1, 7),
            commands,
        };
        self.send_net(killer_world::NetCommand::MakeMacro(Box::new(entry)));
    }

    fn packets_command(&mut self, command: Command) {
        let Some(id) = self.selected else { return };
        let dir = self.config_dir.clone();
        let Some(session) = self.manager.session_mut(id) else { return };
        let ui = &mut self.packet_ui;
        match command {
            Command::PkRecord { on } => session.packet_log_mut().recording = on,
            Command::PkClear => {
                session.packet_log_mut().clear();
                ui.selected = None;
                ui.detail = None;
            }
            Command::PkSave { only_visible } => {
                let text = if only_visible {
                    let mut copy = killer_bot_core::packets::PacketLog::default();
                    copy.recording = true;
                    for entry in ui.visible(session.packet_log()) {
                        copy.add(killer_world::PacketRecord { outgoing: entry.outgoing, body: entry.body.clone() }, entry.at);
                    }
                    copy.save()
                } else {
                    session.packet_log().save()
                };
                ui.status = match dir.map(|dir| std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(dir.join(packet_tools::LOG_FILE), text))) {
                    Some(Ok(())) => "log salvo".to_owned(),
                    Some(Err(error)) => error.to_string(),
                    None => "sem pasta de configuração".to_owned(),
                };
            }
            Command::PkLoad => {
                ui.status = match dir.map(|dir| std::fs::read_to_string(dir.join(packet_tools::LOG_FILE))) {
                    Some(Ok(text)) => format!("{} pacotes carregados", session.packet_log_mut().load(&text)),
                    Some(Err(error)) => error.to_string(),
                    None => "sem pasta de configuração".to_owned(),
                };
            }
            Command::PkSelect { number } => ui.select(number),
            Command::PkSearch { query, client, by_name, direction } => ui.find(session.packet_log(), &query, client, by_name, &direction),
            Command::PkInject { hex, to_client } => {
                ui.status = match killer_bot_core::packets::parse_hex(&hex) {
                    Some(body) => {
                        let outcome = if to_client { session.inject_to_client(&body) } else if session.send_raw(body) { Ok(()) } else { Err("sem conexão".to_owned()) };
                        outcome.map_or_else(|error| error, |()| "enviado".to_owned())
                    }
                    None => "hexadecimal inválido".to_owned(),
                };
            }
            Command::PkParse { format } => ui.parse(session.packet_log(), &format),
            Command::PkHelp => {
                ui.selected = None;
                ui.detail = Some(packet_tools::PARSER_HELP.to_owned());
            }
            Command::PkView { filter_on, hide_server, hide_client } => {
                ui.filter_on = filter_on;
                ui.hide_server = hide_server;
                ui.hide_client = hide_client;
            }
            Command::PkFilters { server, client, whitelist } => ui.filters = packet_tools::Filters { server, client, whitelist },
            Command::PkFiltersSave => {
                if let Some(dir) = dir {
                    let _ = std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(dir.join(packet_tools::FILTER_FILE), packet_tools::save_filters(&ui.filters)));
                }
            }
            Command::PkFiltersLoad => {
                if let Some(Ok(text)) = dir.map(|dir| std::fs::read_to_string(dir.join(packet_tools::FILTER_FILE))) {
                    ui.filters = packet_tools::load_filters(&text);
                }
            }
            Command::PkConvert { text } => ui.converter = packet_tools::convert(&text),
            _ => {}
        }
    }

    fn script_dto(&self, session: &BotSession) -> scripts::ScriptDto {
        let status = session.script_status();
        scripts::ScriptDto {
            tasks: scripts::tasks(session),
            kind: scripts::status_kind(&status),
            status: status.text(false),
            paused: session.script_paused(),
            message: self.script_message.clone(),
            variables: scripts::variables(&session.script_variables()),
            files: self.script_dir.as_ref().map(|dir| scripts::list_scripts(dir)).unwrap_or_default(),
            loaded: self.script_loaded.clone(),
            log: self.accounts.iter().find(|account| self.selected == Some(account.id)).map(|account| account.script_lines[account.script_lines.len().saturating_sub(120)..].to_vec()).unwrap_or_default(),
        }
    }

    fn saved_options(&self, id: u32) -> Option<serde_json::Value> {
        let label = self.accounts.iter().find(|account| account.id == id)?.saved.as_ref()?;
        let entry = self.saved.iter().find(|entry| &entry.label == label)?;
        serde_json::to_value(&entry.profile.options).ok()
    }

    fn set_options(&mut self, options: serde_json::Value, save: bool) {
        let options: killer_bot_core::options::BotOptions = match serde_json::from_value(options) {
            Ok(options) => options,
            Err(error) => {
                self.status = format!("opções inválidas: {error}");
                return;
            }
        };
        let Some(id) = self.selected else { return };
        let Some(session) = self.manager.session_mut(id) else { return };
        let mut profile = session.profile().clone();
        profile.options = options;
        if let Err(error) = profile.check() {
            self.status = error.to_string();
            return;
        }
        session.set_profile(profile.clone());
        if !save {
            return;
        }
        let Some(label) = self.accounts.iter().find(|account| account.id == id).and_then(|account| account.saved.clone()) else {
            self.status = "salve a conta (marque guardar a senha) para guardar as opções".to_owned();
            return;
        };
        if let Some(dir) = &self.profile_dir
            && let Err(error) = profiles::save_profile(dir, &label, &profile)
        {
            self.status = error.to_string();
        }
        self.reload_saved();
    }

    fn with_session(&mut self, change: impl FnOnce(&mut BotSession)) {
        if let Some(id) = self.selected
            && let Some(session) = self.manager.session_mut(id)
        {
            change(session);
        }
    }

    fn clan_names(&self, session: &BotSession, snapshot: &killer_bot_core::snapshot::Snapshot) -> std::collections::HashMap<i32, String> {
        let world = session.world();
        snapshot.nearby.iter().map(|view| view.clan_id).filter(|id| *id != 0).filter_map(|id| world.pledge_name(id).map(|name| (id, name.to_owned()))).collect()
    }

    fn pet_actions(&self) -> Vec<dto::PetActionDto> {
        let Some(loader) = &self.loader else { return Vec::new() };
        let Ok(mut loader) = loader.lock() else { return Vec::new() };
        let Some(data) = loader.ready() else { return Vec::new() };
        data.tables.actions.iter().filter(|action| action.category == 4 || action.category == 5).take(24).map(|action| dto::PetActionDto { id: action.id as i32, name: action.name.clone(), icon: action.icon.clone() }).collect()
    }

    fn user_command(&self, id: u32, command: Command) -> Option<UserCommand> {
        Some(match command {
            Command::ToggleBot => UserCommand::SetAutomation(!self.manager.session(id)?.automation_on()),
            Command::SelectTarget { id } => UserCommand::SelectTarget(id),
            Command::MoveTo { x, y, z } => UserCommand::MoveTo([x, y, z]),
            Command::Say { channel, text } => {
                let channel = dto::channel_of_name(&channel);
                let (target, text) = if channel == killer_bot_core::chat::Channel::Whisper {
                    text.split_once(' ').map(|(target, rest)| (target.to_owned(), rest.to_owned()))?
                } else {
                    (String::new(), text)
                };
                UserCommand::Say { channel, target, text }
            }
            Command::UseItem { id } => UserCommand::UseItem(id),
            Command::DropItem { id, count } => UserCommand::DropItem { object_id: id, count },
            Command::DestroyItem { id, count } => UserCommand::DestroyItem { object_id: id, count },
            Command::Crystallize { id, count } => UserCommand::Crystallize { object_id: id, count },
            Command::UseSkill { id } => UserCommand::UseSkill(id),
            Command::RestartPoint { kind } => UserCommand::RestartPoint(kind),
            Command::Answer { index, accept } => UserCommand::Answer { index, accept },
            Command::RequestClan => UserCommand::RequestClan,
            Command::ClearStats => UserCommand::ClearStats,
            Command::CancelTarget => UserCommand::CancelTarget,
            Command::Sit => UserCommand::Sit,
            Command::Stand => UserCommand::Stand,
            Command::Restart => UserCommand::Restart,
            Command::Logout => UserCommand::Logout,
            Command::IgnoreItem { item_id } => UserCommand::IgnoreItem(item_id),
            Command::IgnoreNpc { npc_id } => UserCommand::IgnoreNpc(npc_id),
            Command::Blacklist { id } => UserCommand::Blacklist(id),
            Command::PartyInvite { name } => UserCommand::PartyInvite(name),
            Command::PartyLeave => UserCommand::PartyLeave,
            Command::PartyKick { name } => UserCommand::PartyKick(name),
            Command::PartyLeader { name } => UserCommand::PartyLeader(name),
            Command::PartyDismiss => UserCommand::PartyDismiss,
            Command::ClanInvite { id } => UserCommand::ClanInvite(id),
            Command::FriendInvite { name } => UserCommand::FriendInvite(name),
            Command::TradeRequest { id } => UserCommand::TradeRequest(id),
            Command::Buy { list_id, orders } => UserCommand::Buy { list_id, orders },
            Command::Sell { list_id, orders } => UserCommand::Sell { list_id, orders },
            Command::MultiSellChoose { list_id, entry_id, amount } => UserCommand::MultiSellChoose { list_id, entry_id, amount },
            Command::Warehouse { deposit, orders } => UserCommand::Warehouse { deposit, orders },
            Command::LearnSkill { skill_id, level, mode } => UserCommand::LearnSkill { skill_id, level, mode },
            Command::TradeAdd { object_id, count } => UserCommand::TradeAdd { object_id, count },
            Command::TradeConfirm => UserCommand::TradeConfirm,
            Command::TradeCancel => UserCommand::TradeCancel,
            Command::RequestQuests => UserCommand::RequestQuests,
            Command::AbortQuest { quest_id } => UserCommand::AbortQuest(quest_id),
            Command::StoreBuy { orders } => UserCommand::StoreBuy { orders },
            Command::StoreSell { orders } => UserCommand::StoreSell { orders },
            Command::StoreSetSell { package_sale, offers } => UserCommand::StoreSetSell { package_sale, offers },
            Command::StoreSetBuy { offers } => UserCommand::StoreSetBuy { offers },
            Command::StoreMessage { buy, text } => UserCommand::StoreMessage { buy, text },
            Command::PetGive { object_id, count } => UserCommand::PetGive { object_id, count },
            Command::PetTake { object_id, count } => UserCommand::PetTake { object_id, count },
            Command::RequestFriends => UserCommand::FriendList,
            Command::DeleteFriend { name } => UserCommand::FriendDelete(name),
            Command::NpcAction { action } => crate::html::user_command(&action)?,
            Command::Bypass { command } => UserCommand::Bypass(command),
            Command::CreateCharacter { .. } | Command::SetOptions { .. } | Command::AddPolygonPoint { .. } | Command::PkRecord { .. } | Command::PkClear | Command::PkSave { .. } | Command::PkLoad | Command::PkSelect { .. } | Command::PkSearch { .. } | Command::PkInject { .. } | Command::PkParse { .. } | Command::PkHelp | Command::PkView { .. } | Command::PkFilters { .. } | Command::PkFiltersSave | Command::PkFiltersLoad | Command::PkConvert { .. } | Command::RunScript { .. } | Command::StopScript | Command::PauseScript { .. } | Command::SaveScript { .. } | Command::LoadScript { .. } | Command::CloseNpc | Command::CloseShop | Command::CloseMultisell | Command::CloseWarehouse | Command::CloseLearn | Command::CloseStore => return None,
            Command::RequestNewCharacter => UserCommand::RequestNewCharacter,
            Command::DeleteCharacter { slot } => UserCommand::DeleteCharacter(slot),
            Command::ChooseServer { id } => UserCommand::ChooseServer(id),
            Command::ChooseCharacter { slot } => UserCommand::ChooseCharacter(slot),
            Command::SelectSession { .. } | Command::CloseSession { .. } | Command::AddAccount { .. } | Command::ConnectSaved { .. } | Command::ForgetSaved { .. } | Command::RetryData | Command::Talk { .. } | Command::RunMacro { .. } | Command::StopMacro | Command::UseShortcut { .. } | Command::RegisterShortcut { .. } | Command::DeleteShortcut { .. } | Command::ToggleAutoShot { .. } | Command::SaveMacro { .. } | Command::DeleteMacro { .. } | Command::Action { .. } => return None,
        })
    }

    fn add(&mut self, host: String, port: u16, account: String, password: String, character: String, remember: bool) {
        if host.trim().is_empty() || account.trim().is_empty() || password.is_empty() {
            self.status = "preencha servidor, conta e senha".to_owned();
            return;
        }
        let text = format!("account = {:?}\ncharacter = {:?}\n[server]\nhost = {:?}\nport = {port}\n", account.trim(), character.trim(), host.trim());
        let profile = match Profile::parse(&text) {
            Ok(profile) => profile,
            Err(error) => {
                self.status = error.to_string();
                return;
            }
        };
        let label = if profile.character.is_empty() { profile.account.clone() } else { format!("{}-{}", profile.character, profile.account) };
        let mut saved = None;
        if remember {
            if let Err(error) = self.store.set(&secret_key(&profile.server.host, profile.server.port, &profile.account), &password) {
                self.status = error;
                return;
            }
            if let Some(dir) = &self.profile_dir {
                if let Err(error) = profiles::save_profile(dir, &label, &profile) {
                    self.status = error.to_string();
                    return;
                }
                saved = Some(label.clone());
            }
        }
        self.connect(label, saved, profile, password);
        self.reload_saved();
    }

    fn connect(&mut self, label: String, saved: Option<String>, profile: Profile, password: String) {
        let id = self.next_id;
        self.next_id += 1;
        let shown = if profile.character.is_empty() { profile.account.clone() } else { profile.character.clone() };
        let transport = (self.connector)(&profile, password);
        let mut session = BotSession::new(profile, transport, Arc::new(SystemClock::default()));
        if self.names_ready {
            session.set_names(self.names_for_session());
        }
        self.manager.add(id, session);
        self.accounts.push(Account { id, label: if shown.is_empty() { label } else { shown }, saved, started: Instant::now(), deaths: 0, bot_lines: Vec::new(), chat_stamps: std::collections::VecDeque::new(), chat_seen: 0, macro_runner: MacroRunner::default(), script_lines: Vec::new() });
        self.selected = Some(id);
        self.status.clear();
    }

    fn connect_saved(&mut self, label: &str) {
        let Some(entry) = self.saved.iter().find(|entry| entry.label == label) else { return };
        let profile = entry.profile.clone();
        let Some(password) = self.store.get(&secret_key(&profile.server.host, profile.server.port, &profile.account)) else {
            self.status = "senha não guardada: entre de novo pelo formulário".to_owned();
            return;
        };
        self.connect(label.to_owned(), Some(label.to_owned()), profile, password);
    }

    fn forget_saved(&mut self, label: &str) {
        let Some(position) = self.saved.iter().position(|entry| entry.label == label) else { return };
        let entry = self.saved.remove(position);
        self.store.forget(&secret_key(&entry.profile.server.host, entry.profile.server.port, &entry.profile.account));
        if let Some(dir) = &self.profile_dir {
            let _ = std::fs::remove_file(dir.join(profiles::file_name(label)));
        }
    }

    fn names_for_session(&self) -> Box<dyn Names> {
        match &self.loader {
            Some(loader) => loader.lock().ok().and_then(|mut loader| loader.ready().map(|data| Box::new(data.names()) as Box<dyn Names>)).unwrap_or_else(|| Box::new(IdNames)),
            None => Box::new(IdNames),
        }
    }

    fn close(&mut self, id: u32) {
        self.manager.remove(id);
        self.accounts.retain(|account| account.id != id);
        if self.selected == Some(id) {
            self.selected = self.accounts.first().map(|account| account.id);
        }
    }

    fn poll_data(&mut self) {
        let Some(loader) = self.loader.clone() else { return };
        let Ok(mut loader) = loader.lock() else { return };
        loader.poll();
        if !self.names_ready
            && let Some(data) = loader.ready()
        {
            self.names = Box::new(data.names());
            self.names_ready = true;
            for id in self.manager.ids() {
                if let Some(session) = self.manager.session_mut(id) {
                    session.set_names(Box::new(data.names()));
                }
            }
        }
    }

    pub fn tick(&mut self) {
        self.poll_data();
        for (id, event) in self.manager.tick() {
            let in_world = self.manager.session(id).is_some_and(|session| session.state() == BotState::InWorld);
            let Some(account) = self.accounts.iter_mut().find(|account| account.id == id) else { continue };
            match event {
                BotEvent::Died => account.deaths += 1,
                BotEvent::Log(text) => {
                    if in_world && text.starts_with("script") {
                        let stamp = dto::uptime_text(account.started.elapsed().as_secs());
                        account.script_lines.push(format!("[{stamp}] {}", text.trim_start_matches("script: ").trim_start_matches("script ")));
                        if account.script_lines.len() > 300 {
                            account.script_lines.remove(0);
                        }
                        if text.starts_with("script parou") {
                            account.bot_lines.push((dto::now_secs(), text));
                        }
                    } else if in_world {
                        account.bot_lines.push((dto::now_secs(), text));
                    } else if self.selected == Some(id) && text.starts_with("criação recusada") {
                        self.status = text;
                    }
                }
                BotEvent::Alert(text) => account.bot_lines.push((dto::now_secs(), format!("alerta: {text}"))),
                BotEvent::BottingStopped => account.bot_lines.push((dto::now_secs(), "bot parado".to_owned())),
                BotEvent::Refused(text) => account.bot_lines.push((dto::now_secs(), format!("recusado: {text}"))),
                BotEvent::State(BotState::Disconnected) => account.bot_lines.push((dto::now_secs(), "desconectado".to_owned())),
                BotEvent::State(_) | BotEvent::Revived => {}
            }
        }
        if let Some((id, object_id, since)) = self.talk {
            let targeted = self.manager.session(id).is_some_and(|session| session.world().targeted().is_some_and(|entity| entity.object_id == object_id));
            if targeted {
                self.talk = None;
                if let Some(session) = self.manager.session_mut(id) {
                    let _ = session.command(UserCommand::SelectTarget(object_id));
                }
            } else if since.elapsed() > Duration::from_secs(6) {
                self.talk = None;
            }
        }
        for account in &mut self.accounts {
            let Some(session) = self.manager.session_mut(account.id) else { continue };
            for (skill_id, hit_time) in session.take_own_casts() {
                account.macro_runner.note_cast(account.started.elapsed(), skill_id, hit_time);
            }
        }
        let mut macro_effects = Vec::new();
        for account in &mut self.accounts {
            let Some(session) = self.manager.session(account.id) else { continue };
            if account.macro_runner.running().is_some() {
                let mut effects = Vec::new();
                account.macro_runner.tick(account.started.elapsed(), session.world(), self.names.as_ref(), &mut effects);
                macro_effects.push((account.id, effects));
            }
        }
        for (id, effects) in macro_effects {
            self.run_effects(id, effects);
        }
        let now = dto::now_secs();
        for account in &mut self.accounts {
            let Some(session) = self.manager.session(account.id) else { continue };
            let total = session.chat_total();
            let fresh = total.saturating_sub(account.chat_seen).min(256) as usize;
            account.chat_stamps.extend(std::iter::repeat_n(now, fresh));
            while account.chat_stamps.len() > 256 {
                account.chat_stamps.pop_front();
            }
            account.chat_seen = total;
        }
    }

    pub fn frame(&self) -> Frame {
        let sessions = self
            .accounts
            .iter()
            .map(|account| SessionDto { id: account.id, label: self.label_of(account), state: self.manager.session(account.id).map_or("offline", |session| dto::state_of(session.state())) })
            .collect();
        let selected_state = self.selected.and_then(|id| self.manager.session(id)).map(BotSession::state);
        let screen = selected_state.map_or("accounts", dto::screen_of);
        let snapshot = self.selected.and_then(|id| self.manager.snapshot(id));
        let (servers, characters) = snapshot.as_ref().map_or((Vec::new(), Vec::new()), |snapshot| {
            (
                snapshot.servers.iter().map(|server| ServerDto { id: server.id, name: self.names.server(server.id, server.flags).unwrap_or_default(), address: server.address.clone(), players: server.players, max_players: server.max_players, online: server.online }).collect(),
                snapshot.characters.iter().map(|entry| CharacterEntryDto { slot: entry.slot, name: entry.name.clone(), class_name: self.names.class(entry.base_class_id), level: entry.level }).collect(),
            )
        });
        let world = (selected_state == Some(BotState::InWorld))
            .then(|| {
                let id = self.selected?;
                let snapshot = snapshot.as_ref()?;
                let session = self.manager.session(id)?;
                let account = self.accounts.iter().find(|account| account.id == id)?;
                let extras = Extras { macros: dto::macro_dtos(session.world(), self.names.as_ref()), macro_run: account.macro_runner.running().map(|(id, line)| dto::MacroRunDto { id, line }), shortcuts: dto::shortcut_dtos(session.world(), self.names.as_ref(), &snapshot.skills), ping: session.ping_ms(), henna: dto::henna_dto(session.world()), friends: session.world().friends.list.iter().map(|friend| dto::FriendDto { name: friend.name.clone(), online: friend.online }).collect(), chat_stamps: account.chat_stamps.iter().copied().collect(), clan_names: self.clan_names(session, snapshot), packets: self.packet_ui.dto(session.packet_log()), script: self.script_dto(session), options: serde_json::to_value(&session.profile().options).unwrap_or_default(), saved_options: self.saved_options(id), windows: crate::windows::windows(session.world(), self.names.as_ref()), pet_actions: if session.world().pet.is_some() { self.pet_actions() } else { Vec::new() }, npc: dto::npc_dialog(session.world(), snapshot, self.names.as_ref()), target_info: dto::target_info(session.world(), self.names.as_ref()), bot_on: session.automation_on() && !session.is_paused(), uptime: dto::uptime_text(account.started.elapsed().as_secs()), deaths: account.deaths };
                let mut world = dto::world(snapshot, &session.chat(), self.names.as_ref(), &extras);
                world.chat.extend(account.bot_lines.iter().map(|(at, text)| dto::ChatLineDto { at: *at, channel: "bot", sender: "Bot".to_owned(), text: text.clone() }));
                world.chat.sort_by_key(|line| line.at);
                Some(world)
            })
            .flatten();
        let saved = self
            .saved
            .iter()
            .map(|entry| SavedDto {
                label: entry.label.clone(),
                account: entry.profile.account.clone(),
                character: entry.profile.character.clone(),
                host: entry.profile.server.host.clone(),
                port: entry.profile.server.port,
                has_password: entry.has_password,
                online: self.accounts.iter().any(|account| account.saved.as_deref() == Some(entry.label.as_str())),
            })
            .collect();
        let data_status = self.loader.as_ref().and_then(|loader| loader.lock().ok().map(|loader| loader.status(false))).unwrap_or_default();
        Frame { screen, sessions, selected: self.selected, status: self.status.clone(), servers, characters, saved, data_status, data_ready: self.names_ready, world }
    }

    fn label_of(&self, account: &Account) -> String {
        self.manager.snapshot(account.id).map(|snapshot| snapshot.name).filter(|name| !name.is_empty()).unwrap_or_else(|| account.label.clone())
    }
}

pub fn run(receiver: std::sync::mpsc::Receiver<Command>, mut controller: Controller, mut publish: impl FnMut(&Frame)) {
    let mut last_frame = Instant::now() - Duration::from_secs(1);
    let mut last_published: Option<Frame> = None;
    loop {
        match receiver.recv_timeout(Duration::from_millis(60)) {
            Ok(command) => controller.apply(command),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
        }
        while let Ok(command) = receiver.try_recv() {
            controller.apply(command);
        }
        controller.tick();
        if last_frame.elapsed() >= Duration::from_millis(250) {
            let frame = controller.frame();
            if last_published.as_ref() != Some(&frame) {
                publish(&frame);
                last_published = Some(frame);
            }
            last_frame = Instant::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secrets::MemoryStore;
    use killer_bot_core::transport::FakeTransport;

    fn controller() -> Controller {
        Controller::new(Box::new(|_, _| Box::new(FakeTransport::default())), Box::new(MemoryStore::default()), None, None)
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("killer-bot-ui-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn add(controller: &mut Controller, account: &str) {
        controller.apply(Command::AddAccount { host: "127.0.0.1".into(), port: 2106, account: account.into(), password: "x".into(), character: "Wender".into(), remember: false });
    }

    #[test]
    fn without_accounts_the_screen_asks_for_one() {
        let frame = controller().frame();
        assert_eq!(frame.screen, "accounts");
        assert!(frame.sessions.is_empty() && frame.world.is_none() && frame.saved.is_empty());
    }

    #[test]
    fn adding_an_account_selects_it_and_shows_the_connecting_screen() {
        let mut controller = controller();
        add(&mut controller, "conta1");
        let frame = controller.frame();
        assert_eq!(frame.screen, "connecting");
        assert_eq!(frame.sessions.len(), 1);
        assert_eq!(frame.selected, Some(1));
        assert_eq!(frame.sessions[0].label, "Wender");
    }

    #[test]
    fn a_bad_host_reports_the_problem_without_creating_a_session() {
        let mut controller = controller();
        controller.apply(Command::AddAccount { host: String::new(), port: 2106, account: "a".into(), password: "x".into(), character: String::new(), remember: false });
        let frame = controller.frame();
        assert!(frame.sessions.is_empty());
        assert!(!frame.status.is_empty());
        assert_eq!(frame.screen, "accounts");
    }

    #[test]
    fn selecting_and_closing_sessions_keeps_a_valid_selection() {
        let mut controller = controller();
        add(&mut controller, "a");
        add(&mut controller, "b");
        assert_eq!(controller.frame().selected, Some(2));
        controller.apply(Command::SelectSession { id: 1 });
        assert_eq!(controller.frame().selected, Some(1));
        controller.apply(Command::SelectSession { id: 99 });
        assert_eq!(controller.frame().selected, Some(1));
        controller.apply(Command::CloseSession { id: 1 });
        assert_eq!(controller.frame().selected, Some(2));
        controller.apply(Command::CloseSession { id: 2 });
        assert_eq!(controller.frame().screen, "accounts");
    }

    #[test]
    fn commands_from_the_page_parse_with_camel_case_fields() {
        let command: Command = serde_json::from_str(r#"{"type":"addAccount","host":"h","port":1,"account":"a","password":"p","character":"c","remember":true}"#).unwrap();
        assert!(matches!(command, Command::AddAccount { port: 1, remember: true, .. }));
        let command: Command = serde_json::from_str(r#"{"type":"moveTo","x":1,"y":2,"z":3}"#).unwrap();
        assert_eq!(command, Command::MoveTo { x: 1, y: 2, z: 3 });
        let command: Command = serde_json::from_str(r#"{"type":"connectSaved","label":"x"}"#).unwrap();
        assert_eq!(command, Command::ConnectSaved { label: "x".into() });
        assert!(serde_json::from_str::<Command>(r#"{"type":"nope"}"#).is_err());
    }

    #[test]
    fn a_remembered_account_is_saved_without_the_password_and_connects_again() {
        let dir = scratch("saved");
        let mut controller = Controller::new(Box::new(|_, _| Box::new(FakeTransport::default())), Box::new(MemoryStore::default()), Some(dir.clone()), None);
        controller.apply(Command::AddAccount { host: "h".into(), port: 2106, account: "conta".into(), password: "senha-secreta".into(), character: String::new(), remember: true });
        let frame = controller.frame();
        assert_eq!(frame.saved.len(), 1);
        assert!(frame.saved[0].has_password && frame.saved[0].online);
        let raw: String = std::fs::read_dir(&dir).unwrap().filter_map(Result::ok).map(|entry| std::fs::read_to_string(entry.path()).unwrap()).collect();
        assert!(raw.contains("conta") && !raw.contains("senha-secreta"));
        controller.apply(Command::CloseSession { id: 1 });
        assert!(!controller.frame().saved[0].online);
        controller.apply(Command::ConnectSaved { label: "conta".into() });
        assert_eq!(controller.frame().sessions.len(), 1);
        controller.apply(Command::ForgetSaved { label: "conta".into() });
        assert!(controller.frame().saved.is_empty());
        assert!(std::fs::read_dir(&dir).unwrap().next().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    struct UntouchableStore;

    impl SecretStore for UntouchableStore {
        fn get(&self, _: &str) -> Option<String> {
            panic!("keychain read");
        }

        fn set(&self, _: &str, _: &str) -> Result<(), String> {
            panic!("keychain write");
        }

        fn forget(&self, _: &str) {
            panic!("keychain delete");
        }
    }

    #[test]
    fn an_account_not_remembered_never_touches_the_keychain_even_with_saved_profiles() {
        let dir = scratch("untouched");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("conta.toml"), "account = \"conta\"\ncharacter = \"\"\n[server]\nhost = \"h\"\nport = 1\n").unwrap();
        let mut controller = Controller::new(Box::new(|_, _| Box::new(FakeTransport::default())), Box::new(UntouchableStore), Some(dir.clone()), None);
        controller.apply(Command::AddAccount { host: "h".into(), port: 2106, account: "outra".into(), password: "senha".into(), character: String::new(), remember: false });
        assert_eq!(controller.frame().sessions.len(), 1);
        assert_eq!(controller.frame().saved.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn connecting_a_saved_account_without_a_stored_password_explains_why() {
        let dir = scratch("nopass");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("conta.toml"), "account = \"conta\"\ncharacter = \"\"\n[server]\nhost = \"h\"\nport = 1\n").unwrap();
        let mut controller = Controller::new(Box::new(|_, _| Box::new(FakeTransport::default())), Box::new(MemoryStore::default()), Some(dir.clone()), None);
        controller.apply(Command::ConnectSaved { label: "conta".into() });
        let frame = controller.frame();
        assert!(frame.sessions.is_empty() && frame.status.contains("senha"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn options_from_the_page_are_validated_applied_and_saved_with_the_account() {
        let dir = scratch("options");
        let mut controller = Controller::new(Box::new(|_, _| Box::new(FakeTransport::default())), Box::new(MemoryStore::default()), Some(dir.clone()), None);
        controller.apply(Command::AddAccount { host: "h".into(), port: 2106, account: "conta".into(), password: "x".into(), character: String::new(), remember: true });
        let mut options = serde_json::to_value(killer_bot_core::options::BotOptions::default()).unwrap();
        options["follow"]["distance"] = serde_json::json!(222.0);
        controller.apply(Command::SetOptions { options: options.clone(), save: true });
        let distance = controller.manager.session(1).unwrap().profile().options.follow.distance;
        assert_eq!(distance, 222.0);
        assert_eq!(controller.saved_options(1).unwrap()["follow"]["distance"], serde_json::json!(222.0));
        options["campo_inexistente"] = serde_json::json!(1);
        controller.apply(Command::SetOptions { options, save: false });
        assert!(controller.frame().status.contains("opções inválidas"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn saving_options_without_a_saved_account_explains_what_to_do() {
        let mut controller = controller();
        add(&mut controller, "conta");
        let options = serde_json::to_value(killer_bot_core::options::BotOptions::default()).unwrap();
        controller.apply(Command::SetOptions { options, save: true });
        assert!(controller.frame().status.contains("salve a conta"));
    }

    #[test]
    fn polygon_points_clicked_on_the_map_are_appended_to_the_options() {
        let mut controller = controller();
        add(&mut controller, "conta");
        controller.apply(Command::AddPolygonPoint { x: 10, y: -20 });
        controller.apply(Command::AddPolygonPoint { x: 30, y: 40 });
        let points = &controller.manager.session(1).unwrap().profile().options.polygon.points;
        assert_eq!(points.iter().map(|point| (point.x, point.y)).collect::<Vec<_>>(), vec![(10, -20), (30, 40)]);
    }

    #[test]
    fn a_new_character_is_checked_before_it_is_sent() {
        let mut controller = controller();
        add(&mut controller, "conta");
        controller.apply(Command::CreateCharacter { name: String::new(), race: 0, female: false, mage: false, hair_style: 0, hair_color: 0, face: 0 });
        assert_eq!(controller.frame().status, "digite o nome");
        controller.apply(Command::CreateCharacter { name: "Anao".into(), race: 4, female: false, mage: true, hair_style: 0, hair_color: 0, face: 0 });
        assert!(controller.frame().status.contains("anão"));
    }

    #[test]
    fn login_prompts_do_not_pile_up_in_the_bot_chat() {
        use killer_bot_core::testkit::char_entry;
        use killer_world::{NetEvent, SessionEvent};
        let mut controller = Controller::new(
            Box::new(|_, _| {
                let mut transport = FakeTransport::default();
                transport.push(NetEvent::Session(SessionEvent::Characters(vec![char_entry("A"), char_entry("B")])));
                Box::new(transport)
            }),
            Box::new(MemoryStore::default()),
            None,
            None,
        );
        controller.apply(Command::AddAccount { host: "h".into(), port: 1, account: "conta".into(), password: "x".into(), character: String::new(), remember: false });
        controller.tick();
        let frame = controller.frame();
        assert_eq!(frame.screen, "characters");
        assert!(frame.status.is_empty());
        assert!(controller.accounts[0].bot_lines.is_empty());
    }

    fn in_world_controller() -> Controller {
        use killer_bot_core::testkit::{Vitals, char_entry, user_info};
        use killer_world::{NetEvent, SessionEvent};
        let mut controller = Controller::new(
            Box::new(|_, _| {
                let mut transport = FakeTransport::default();
                transport.push(NetEvent::Session(SessionEvent::Characters(vec![char_entry("Wender")])));
                transport.push(NetEvent::Session(user_info(1, "Wender", &Vitals::full(), [0, 0, 0])));
                Box::new(transport)
            }),
            Box::new(MemoryStore::default()),
            None,
            None,
        );
        controller.apply(Command::AddAccount { host: "h".into(), port: 1, account: "conta".into(), password: "x".into(), character: "Wender".into(), remember: false });
        controller.tick();
        controller
    }

    #[test]
    fn talking_to_an_object_that_is_not_around_is_refused_and_does_not_wait() {
        let mut controller = in_world_controller();
        controller.apply(Command::Talk { id: 4242 });
        assert!(controller.frame().status.contains("recusado"));
        assert!(controller.talk.is_none());
    }

    #[test]
    fn a_macro_without_name_or_acronym_is_refused_before_anything_is_sent() {
        let mut controller = in_world_controller();
        controller.apply(Command::SaveMacro { id: 0, name: " ".into(), acronym: "M".into(), description: String::new(), icon: 1, lines: vec!["/sit".into()] });
        assert!(controller.frame().status.contains("nome"));
        controller.apply(Command::SaveMacro { id: 0, name: "Sentar".into(), acronym: String::new(), description: String::new(), icon: 1, lines: Vec::new() });
        assert!(controller.frame().status.contains("sigla"));
    }

    #[test]
    fn running_a_macro_the_server_never_sent_is_reported_in_the_bot_chat() {
        let mut controller = in_world_controller();
        controller.apply(Command::RunMacro { id: 5 });
        let frame = controller.frame();
        let chat = &frame.world.as_ref().unwrap().chat;
        assert!(chat.iter().any(|line| line.channel == "bot" && line.text.contains("macro 5")));
    }

    #[test]
    fn chat_lines_get_the_time_they_were_first_seen() {
        use killer_world::{NetEvent, SessionEvent, WorldEvent};
        let mut controller = Controller::new(
            Box::new(|_, _| {
                use killer_bot_core::testkit::{Vitals, char_entry, user_info};
                let mut transport = FakeTransport::default();
                transport.push(NetEvent::Session(SessionEvent::Characters(vec![char_entry("Wender")])));
                transport.push(NetEvent::Session(user_info(1, "Wender", &Vitals::full(), [0, 0, 0])));
                transport.push(NetEvent::Session(SessionEvent::World(WorldEvent::Say2(l2_net::game::Say2::text(1, 0, "Luzal", "oi")))));
                Box::new(transport)
            }),
            Box::new(MemoryStore::default()),
            None,
            None,
        );
        controller.apply(Command::AddAccount { host: "h".into(), port: 1, account: "conta".into(), password: "x".into(), character: "Wender".into(), remember: false });
        controller.tick();
        let frame = controller.frame();
        let chat = &frame.world.as_ref().unwrap().chat;
        let line = chat.iter().find(|line| line.text == "oi").unwrap();
        assert!(line.at.abs_diff(dto::now_secs()) <= 2);
    }

    #[test]
    fn a_script_started_from_the_page_runs_and_reports_its_state() {
        let mut controller = in_world_controller();
        assert_eq!(controller.frame().screen, "world");
        controller.apply(Command::RunScript { text: "while true\n  print \"oi\"\n  wait 1000\nend\n".into() });
        controller.tick();
        let script = controller.frame().world.unwrap().script;
        assert!(matches!(script.kind, "running" | "waiting"), "{} / {}", script.kind, script.message);
        controller.apply(Command::StopScript);
        assert_eq!(controller.frame().world.unwrap().script.kind, "none");
        controller.apply(Command::RunScript { text: "if hp < 3\nsit\n".into() });
        let script = controller.frame().world.unwrap().script;
        assert!(!script.message.is_empty() && script.kind == "none");
    }

    #[test]
    fn every_command_the_page_can_send_exists_in_the_controller() {
        let model = include_str!("../ui/src/model.ts");
        let start = model.find("export type Command =").unwrap();
        let mut names = Vec::new();
        for part in model[start..].split("{ type: \"").skip(1) {
            let name: String = part.chars().take_while(|ch| *ch != '"').collect();
            names.push(name);
        }
        assert!(names.len() > 60, "{}", names.len());
        for name in names {
            if name == "quit" {
                continue;
            }
            let error = serde_json::from_str::<Command>(&format!("{{\"type\":\"{name}\"}}")).err().map(|error| error.to_string()).unwrap_or_default();
            assert!(!error.contains("unknown variant"), "comando sem equivalente no Rust: {name}");
        }
    }

    #[test]
    fn script_commands_are_traced_in_the_script_output_with_a_time() {
        let mut controller = in_world_controller();
        controller.apply(Command::RunScript { text: "sit\nprint \"oi\"\nstand\n".into() });
        for _ in 0..40 {
            controller.tick();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let log = controller.frame().world.unwrap().script.log;
        let joined = log.join("\n");
        assert!(joined.contains("script iniciado"), "{joined}");
        assert!(joined.contains("oi"), "{joined}");
        assert!(joined.contains("› sit") && joined.contains("› stand"), "{joined}");
        assert!(log.iter().all(|line| line.starts_with('[')));
    }

    #[test]
    fn a_buff_loop_with_a_wait_keeps_casting_and_logging() {
        use killer_bot_core::testkit::{Vitals, char_entry, user_info};
        use killer_world::{NetEvent, SessionEvent, WorldEvent};
        let mut controller = Controller::new(
            Box::new(|_, _| {
                let mut transport = FakeTransport::default();
                transport.push(NetEvent::Session(SessionEvent::Characters(vec![char_entry("Wender")])));
                transport.push(NetEvent::Session(user_info(1, "Wender", &Vitals::full(), [0, 0, 0])));
                transport.push(NetEvent::Session(SessionEvent::World(WorldEvent::SkillList(l2_net::game::SkillList { skills: vec![l2_net::game::SkillEntry { passive: false, level: 2, id: 1204 }] }))));
                Box::new(transport)
            }),
            Box::new(MemoryStore::default()),
            None,
            None,
        );
        controller.apply(Command::AddAccount { host: "h".into(), port: 1, account: "conta".into(), password: "x".into(), character: "Wender".into(), remember: false });
        controller.tick();
        controller.apply(Command::RunScript { text: "while true\n  use_skill 1204\n  print \"buff enviado\"\n  wait 200\nend\n".into() });
        for _ in 0..60 {
            controller.tick();
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let log = controller.frame().world.unwrap().script.log.join("\n");
        assert!(log.matches("› use_skill 1204").count() >= 3, "{log}");
        assert!(log.matches("buff enviado").count() >= 3, "{log}");
    }

    #[test]
    fn a_waiting_script_reports_its_line_and_the_time_left() {
        let mut controller = in_world_controller();
        controller.apply(Command::RunScript { text: "while true\n  print \"a\"\n  wait 10000\nend\n".into() });
        controller.tick();
        let script = controller.frame().world.unwrap().script;
        assert_eq!(script.kind, "waiting");
        assert_eq!(script.tasks.len(), 1);
        assert_eq!(script.tasks[0].line, 3);
        assert!(script.tasks[0].wait_left_ms.is_some_and(|left| left > 8000 && left <= 10000));
    }
}
