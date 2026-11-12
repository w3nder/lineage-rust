use crate::lobby::{Lobby, LobbyCommand, LobbyScreen};
use killer_assets::client::DatTables;
use killer_ui::chat::InputKey;
use killer_ui::draw::DrawList;
use killer_ui::font::BitmapFont;
use killer_ui::lobby_create::{CreateAction, CreateScreen, CreateTexts};
use killer_ui::lobby_credit::{CreditEntry, CreditScreen};
use killer_ui::lobby_keycrypt::KeyCryptWarning;
use killer_ui::lobby_login::{Field, LoginAction, LoginLabels, LoginScreen};
use killer_ui::lobby_notice::{Kind, Notice, NoticeAction, NoticeLabels};
use killer_ui::lobby_replay::{ReplayAction, ReplayLabels, ReplayWnd};
use killer_ui::replay_menu::{FunctionAction, ReplayLogoWnd, ReplayMenuWnd, ReplayPlayWnd};
use killer_ui::lobby_select::{CharCard, NameTag, SecondLine, SelectAction, SelectScreen, SelectTexts, karma_color, remaining_text};
use killer_ui::lobby_servers::{ServerAction, ServerEntry, ServerLabels, ServerSelect, server_name};
use l2_net::game::NewCharacter;

pub const COPYRIGHT: &str = "Copyright 2002 NCsoft Corporation. All Rights Reserved.";
pub const REPLAY_LOGO_TEXTURE: &str = "L2Font-e.Replay_logo-e";
pub const MIN_EULA_CHARS: usize = 2;
pub const LANGUAGE_CHINA: i32 = 4;
pub const DOUBLE_CLICK_MS: u128 = 400;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Overlay {
    Eula,
    Warning,
    Announce,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayRequest {
    List,
    Load { name: String, option: bool, chat: bool },
    PrivateStart { benchmark: bool },
    Delete { path: String },
}

pub struct NameRules {
    obscene: Vec<String>,
    npc_names: std::collections::HashSet<String>,
}

impl NameRules {
    pub fn from_tables(tables: &DatTables) -> NameRules {
        NameRules {
            obscene: tables.obscene.clone(),
            npc_names: tables.npcname.values().map(|npc| npc.name.to_lowercase()).collect(),
        }
    }

    pub fn refuses(&self, name: &str) -> bool {
        name.is_empty() || l2_dat::obscene::contains(&self.obscene, name) || self.npc_names.contains(&name.to_lowercase())
    }
}

pub struct LobbyC4 {
    name_rules: NameRules,
    size: (i32, i32),
    buttons: std::cell::RefCell<Vec<killer_ui::draw::ButtonRegion>>,
    pub sounds: Vec<&'static str>,
    login: LoginScreen,
    replay: ReplayWnd,
    replay_menu: Option<ReplayMenuWnd>,
    pub replay_state: i32,
    pub benchmark: killer_world::replay_private::BenchmarkData,
    replay_logo: ReplayLogoWnd,
    replay_play: ReplayPlayWnd,
    replay_requests: Vec<ReplayRequest>,
    servers: ServerSelect,
    select: SelectScreen,
    create: CreateScreen,
    eula: Notice,
    warning: Notice,
    announce: Notice,
    key_crypt: KeyCryptWarning,
    use_key_crypt: bool,
    language_type: i32,
    auto_logon: Option<killer_assets::login_ini::AutoLogOn>,
    has_eula: bool,
    has_warning: bool,
    overlay: Option<Overlay>,
    screen: Option<LobbyScreen>,
    revision: u32,
    status_seen: String,
    commands: Vec<LobbyCommand>,
    messages: std::collections::HashMap<u32, String>,
    messages_sys: std::collections::HashMap<u32, String>,
    last_click: Option<(killer_platform::time::Instant, i32, i32)>,
    units: [String; 3],
    help_request: Option<String>,
    credits: CreditScreen,
    credit_font: Option<BitmapFont>,
    credit_font_loaded: bool,
    options_requested: bool,
    options_close_requested: bool,
    servers_seen: Option<u32>,
    ping_requests: Vec<(u8, [u8; 4], u16)>,
    ping_cache: crate::ping_cache::PingCache,
    ping_edits: Vec<crate::ui_option_config::ConfigEdit>,
    cursor: (i32, i32),
}

fn message_table(tables: &DatTables) -> std::collections::HashMap<u32, String> {
    tables.systemmsg.iter().map(|(id, message)| (*id, message.text.clone())).collect()
}

impl LobbyC4 {
    pub fn new(size: (i32, i32), tables: &DatTables, class_info: &[l2_dat::ClassInfo]) -> LobbyC4 {
        let sysstring = |id: u32| tables.sysstring.get(&id).cloned();
        let systemmsg = |id: u32| tables.systemmsg.get(&id).map(|message| message.text.clone());
        let mut info = vec![String::new(); 10];
        for row in class_info {
            if let Some(slot) = info.get_mut(row.id as usize) {
                *slot = row.text.clone();
            }
        }
        let select_texts = SelectTexts::from_tables(sysstring, systemmsg);
        let units = select_texts.units.clone();
        let mut eula = Notice::new(size, Kind::Eula, NoticeLabels::from_sysstrings(sysstring));
        let has_eula = tables.eula.as_ref().is_some_and(|eula| eula.eula.chars().count() >= MIN_EULA_CHARS);
        if let Some(text) = tables.eula.as_ref() {
            eula.set_text(text.text(tables.login_ini.test_server));
        }
        let mut warning = Notice::new(size, Kind::Warning, NoticeLabels::from_sysstrings(sysstring));
        let has_warning = tables.warning_notice.as_ref().is_some_and(|text| text.chars().count() >= MIN_EULA_CHARS);
        if let Some(text) = tables.warning_notice.as_ref() {
            warning.set_text(text);
        }
        let announce = Notice::new(size, Kind::Announce, NoticeLabels::from_sysstrings(sysstring));
        let mut key_crypt = KeyCryptWarning::new(size, systemmsg(killer_ui::lobby_keycrypt::SYSMSG_TEXT).unwrap_or_default(), sysstring(killer_ui::lobby_keycrypt::SYSSTRING_OK).unwrap_or_default());
        let use_key_crypt = tables.login_ini.use_key_crypt;
        if use_key_crypt {
            key_crypt.show();
        }
        let mut login = LoginScreen::new(size, LoginLabels::from_sysstrings(sysstring));
        if !tables.logo_textures.0.is_empty() {
            login.logo = tables.logo_textures.0.clone();
            login.mini_logo = tables.logo_textures.1.clone();
        }
        let mut create = CreateScreen::new(size, CreateTexts::from_tables(sysstring, info));
        create.language_type = tables.language_type;
        LobbyC4 {
            name_rules: NameRules::from_tables(tables),
            size,
            buttons: std::cell::RefCell::new(Vec::new()),
            sounds: Vec::new(),
            login,
            replay: ReplayWnd::new(size, ReplayLabels::from_sysstrings(sysstring)),
            replay_menu: tables.login_ini.replay_mode.then(|| ReplayMenuWnd::new(size, sysstring)),
            replay_state: killer_world::replay_private::STATE_MENU,
            benchmark: killer_world::replay_private::BenchmarkData::default(),
            replay_logo: ReplayLogoWnd::new(size, REPLAY_LOGO_TEXTURE, killer_ui::lobby_login::MINI_LOGO),
            replay_play: ReplayPlayWnd::new(size, REPLAY_LOGO_TEXTURE, killer_ui::lobby_login::MINI_LOGO),
            replay_requests: Vec::new(),
            servers: ServerSelect::new(size, ServerLabels::from_sysstrings(sysstring)),
            select: {
                let mut select = SelectScreen::new(size, select_texts);
                select.locked = tables.login_ini.auto_logon.is_some();
                select
            },
            create,
            eula,
            warning,
            announce,
            key_crypt,
            use_key_crypt,
            language_type: tables.language_type,
            auto_logon: tables.login_ini.auto_logon.clone(),
            has_eula,
            has_warning,
            overlay: None,
            screen: None,
            revision: u32::MAX,
            status_seen: String::new(),
            commands: Vec::new(),
            messages: message_table(tables),
            messages_sys: tables.sysstring.iter().map(|(id, text)| (*id, text.clone())).collect(),
            last_click: None,
            units,
            help_request: None,
            credits: CreditScreen::new(size, tables.credits.iter().map(|credit| CreditEntry { id: credit.id, html: credit.html.clone(), image: credit.image.clone(), time: credit.time, align: credit.align }).collect()),
            credit_font: None,
            credit_font_loaded: false,
            options_requested: false,
            options_close_requested: false,
            servers_seen: None,
            ping_requests: Vec::new(),
            ping_cache: Default::default(),
            ping_edits: Vec::new(),
            cursor: (0, 0),
        }
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        if size == self.size {
            return;
        }
        self.size = size;
        self.login.screen = size;
        self.replay.screen = size;
        if let Some(menu) = self.replay_menu.as_mut() {
            menu.resize(size);
        }
        self.replay_logo.screen = size;
        self.replay_play.screen = size;
        self.servers.screen = size;
        self.select.screen = size;
        self.create.screen = size;
        self.eula.screen = size;
        self.warning.screen = size;
        self.announce.screen = size;
        self.key_crypt.screen = size;
        self.credits.screen = size;
    }

    pub fn take_help_request(&mut self) -> Option<String> {
        self.help_request.take()
    }

    pub fn take_replay_requests(&mut self) -> Vec<ReplayRequest> {
        std::mem::take(&mut self.replay_requests)
    }

    pub fn replay_menu_shown(&self) -> bool {
        self.replay_menu.is_some() && killer_world::replay_private::screen_for_state(self.replay_state, false) == killer_world::replay_private::ReplayScreen::Menu
    }

    pub fn set_replay_files(&mut self, files: &[String]) {
        self.replay.fill(files);
    }

    pub fn take_options_request(&mut self) -> bool {
        std::mem::take(&mut self.options_requested)
    }

    pub fn take_options_close(&mut self) -> bool {
        std::mem::take(&mut self.options_close_requested)
    }

    pub fn credits_visible(&self) -> bool {
        self.credits.visible
    }

    pub fn needs_credit_font(&self) -> bool {
        !self.credit_font_loaded
    }

    pub fn set_credit_font(&mut self, font: Option<BitmapFont>) {
        self.credit_font = font;
        self.credit_font_loaded = true;
    }

    pub fn take_credit_request(&mut self) -> Option<String> {
        self.credits.html_request.take()
    }

    pub fn set_credit_html(&mut self, text: &str, fallback: &BitmapFont, resolve: &dyn Fn(killer_ui::html::Marker, i32) -> Option<String>) {
        let font = self.credit_font.as_ref().unwrap_or(fallback);
        self.credits.set_html_with(text, font, resolve);
    }

    pub fn take_ping_requests(&mut self) -> Vec<(u8, [u8; 4], u16)> {
        std::mem::take(&mut self.ping_requests)
    }

    pub fn set_ping(&mut self, id: u8, ms: i32) {
        self.ping_cache.record(id, ms);
        self.servers.set_ping(id, ms);
    }

    pub fn set_ping_cache(&mut self, cache: crate::ping_cache::PingCache) {
        self.ping_cache = cache;
    }

    pub fn take_ping_edits(&mut self) -> Vec<crate::ui_option_config::ConfigEdit> {
        std::mem::take(&mut self.ping_edits)
    }

    pub fn set_help(&mut self, text: &str, font: &BitmapFont, resolve: &dyn Fn(killer_ui::html::Marker, i32) -> Option<String>) {
        self.servers.set_help_with(text, font, resolve);
    }

    pub fn take_commands(&mut self) -> Vec<LobbyCommand> {
        std::mem::take(&mut self.commands)
    }

    fn say(&mut self, id: u32) {
        let text = self.messages.get(&id).cloned().unwrap_or_default();
        self.login.set_message(&text);
    }

    fn name_blocked(font: &BitmapFont, name: &str) -> bool {
        name.chars().any(|c| !font.has_glyph(c))
    }

    fn create_name_blocked(rules: &NameRules, font: &BitmapFont, name: &str) -> bool {
        Self::name_blocked(font, name) || rules.refuses(name)
    }

    pub fn update(&mut self, lobby: &mut Lobby, dt_ms: u32) {
        self.login.tick(dt_ms);
        self.servers.tick(dt_ms);
        self.credits.tick(dt_ms as f32 / 1000.0);
        if self.screen != Some(lobby.screen) || self.revision != lobby.revision {
            let entered = self.screen != Some(lobby.screen);
            self.screen = Some(lobby.screen);
            self.revision = lobby.revision;
            self.sync(lobby, entered);
        }
        if lobby.login.status != self.status_seen {
            self.status_seen = lobby.login.status.clone();
            if lobby.screen == LobbyScreen::Login {
                let text = if self.status_seen.is_empty() { COPYRIGHT.to_string() } else { self.status_seen.clone() };
                self.login.set_message(&text);
            }
        }
    }

    fn sync(&mut self, lobby: &mut Lobby, entered: bool) {
        match lobby.screen {
            LobbyScreen::Login => {
                if entered {
                    self.login.reset();
                    if let Some(auto) = self.auto_logon.take() {
                        self.login.prefill(&auto.id, &auto.password);
                    }
                    if self.use_key_crypt {
                        self.key_crypt.show();
                    }
                    self.credits.visible = false;
                    self.replay.visible = false;
                    self.overlay = None;
                    let text = if lobby.login.status.is_empty() { COPYRIGHT.to_string() } else { lobby.login.status.clone() };
                    self.login.set_message(&text);
                }
                self.sync_notice(lobby);
                self.sync_card(lobby);
            }
            LobbyScreen::Servers => {
                self.sync_servers(lobby);
                if entered {
                    self.help_request = Some(killer_ui::lobby_servers::HELP_FILE.to_string());
                }
                self.overlay = None;
                self.login.set_message("");
            }
            LobbyScreen::Characters => {
                self.overlay = None;
                self.login.set_message("");
                if let Some(id) = lobby.system_message.take() {
                    self.say(id);
                }
                let cards = lobby.characters.iter().map(card).collect();
                match self.select.set_cards(cards) {
                    Some(SelectAction::Select(Some(slot))) => lobby.select(slot),
                    _ => lobby.deselect(),
                }
            }
            LobbyScreen::Create => {
                if entered {
                    self.create.reset();
                }
                if let Some(code) = lobby.create_failure.take() {
                    if let Some(id) = self.create.fail(code, false) {
                        self.say(id);
                    }
                    if code == 1 {
                        lobby.create.race = None;
                        lobby.create.class = None;
                        lobby.create.female = None;
                        lobby.apply_choice_with(None, false);
                    }
                }
            }
        }
    }

    fn sync_notice(&mut self, lobby: &mut Lobby) {
        match lobby.pending_notice {
            Some(_) if self.overlay.is_none() => {
                if self.has_eula {
                    self.overlay = Some(Overlay::Eula);
                } else if self.language_type == LANGUAGE_CHINA {
                    self.overlay = Some(Overlay::Announce);
                } else {
                    self.answer(lobby, true);
                }
            }
            None => self.overlay = None,
            Some(_) => {}
        }
    }

    fn sync_card(&mut self, lobby: &mut Lobby) {
        let Some((card_id, position, retry)) = lobby.pending_card.take() else { return };
        self.login.card_prompt = self.messages.get(&killer_ui::lobby_login::SYSMSG_CARD_PROMPT).cloned().unwrap_or_default();
        self.login.set_card(card_id, i32::from(position));
        if retry {
            self.say(killer_ui::lobby_login::refusal_ids(killer_ui::lobby_login::CARD_WRONG)[0]);
        }
    }

    fn answer(&mut self, lobby: &mut Lobby, agreed: bool) {
        self.overlay = None;
        lobby.pending_notice = None;
        self.commands.push(if agreed { LobbyCommand::NoticeAnswer(true) } else { LobbyCommand::Back });
    }

    fn sync_servers(&mut self, lobby: &Lobby) {
        if self.servers_seen == Some(lobby.servers_revision) {
            return;
        }
        self.servers_seen = Some(lobby.servers_revision);
        self.options_close_requested = true;
        let names = lobby.view.as_ref().map(|view| &view.server_names);
        let idc = lobby.view.as_ref().map(|view| &view.idc_names);
        self.servers.last_server = lobby.raw_servers.iter().find(|server| server.extra.last).map(|server| server.id);
        self.servers.clear();
        self.servers.entries = lobby
            .raw_servers
            .iter()
            .map(|server| {
                let name = server_name(server.id, server.extra.flags, |index| names.and_then(|names| names.get(&index)).cloned());
                let name = if name.is_empty() { format!("Server {}", server.id) } else { name };
                let name = killer_ui::lobby_servers::bracketed_name(server.extra.brackets, |index| idc.and_then(|idc| idc.get(&index)).cloned(), &name);
                ServerEntry {
                    id: server.id,
                    ip: u32::from_be_bytes(server.ip),
                    port: u32::from(server.port),
                    age: server.extra.age,
                    pvp: server.extra.pvp,
                    current: i32::from(server.players),
                    max: i32::from(server.max_players),
                    status: server.extra.status,
                    flags: server.extra.flags,
                    brackets: server.extra.brackets,
                    name,
                    forbidden: server.id >= 1 && lobby.forbidden.get(usize::from(server.id - 1) / 8).is_some_and(|byte| byte >> ((server.id - 1) % 8) & 1 != 0),
                    ping: killer_ui::lobby_servers::NO_PING,
                }
            })
            .collect();
        let mut requests = Vec::new();
        for server in &lobby.raw_servers {
            match self.ping_cache.cached(server.id) {
                Some(ms) => {
                    if let Some(entry) = self.servers.entries.iter_mut().find(|entry| entry.id == server.id) {
                        entry.ping = ms;
                    }
                }
                None => requests.push((server.id, server.ip, server.port)),
            }
        }
        self.servers.pending_pings = requests.len();
        self.ping_requests = requests;
        self.servers.finish();
    }

    pub fn frame(&self, lobby: &Lobby, font: &BitmapFont, tags: &[NameTag]) -> DrawList {
        let mut list = DrawList::default();
        self.replay_logo.paint(&mut list);
        self.replay_play.paint(&mut list);
        match lobby.screen {
            LobbyScreen::Login if self.credits.visible => self.credits.paint(&mut list, self.credit_font.as_ref().unwrap_or(font)),
            LobbyScreen::Login if self.replay_menu_shown() => {
                if let Some(menu) = &self.replay_menu {
                    menu.paint(&mut list, font);
                }
            }
            LobbyScreen::Login if self.replay.visible => {
                self.login.paint_logo(&mut list, font, false);
                self.replay.paint(&mut list, font, &|id| self.replay_label(id));
            }
            LobbyScreen::Login => {
                match self.overlay {
                    Some(overlay) => {
                        self.login.paint_backdrop(&mut list, font);
                        self.notice(overlay).paint(&mut list, font);
                    }
                    None => self.login.paint(&mut list, font),
                }
                self.key_crypt.paint(&mut list, font);
            }
            LobbyScreen::Servers => {
                self.login.paint_backdrop(&mut list, font);
                self.servers.paint(&mut list, font);
            }
            LobbyScreen::Characters => {
                self.select.paint(&mut list, font, tags);
                self.login.paint_message(&mut list, font, self.size.0, self.size.1);
            }
            LobbyScreen::Create => {
                self.create.paint(&mut list, font);
                self.login.paint_message(&mut list, font, self.size.0, self.size.1);
            }
        }
        self.buttons.borrow_mut().clone_from(&list.buttons);
        list
    }

    pub fn name_tags(&mut self, lobby: &Lobby, slots: &[crate::lobby::panels::CharacterSlot]) -> Vec<NameTag> {
        slots
            .iter()
            .zip(&lobby.characters)
            .filter_map(|(slot, entry)| {
                let [x, y] = slot.label?;
                let second_line = if entry.delete_days > 0 {
                    let [lx, ly] = slot.delete_label?;
                    let text = format!("{} : {}", self.select.texts.delete_line, remaining_text(entry.delete_days, [self.units[0].as_str(), self.units[1].as_str(), self.units[2].as_str()]));
                    Some(SecondLine { x: lx as i32, y: ly as i32, text })
                } else {
                    None
                };
                Some(NameTag { x: x as i32, y: y as i32, name: slot.name.clone(), color: karma_color(entry.karma), second_line })
            })
            .collect()
    }

    pub fn snap(&self, screen: LobbyScreen, x: i32, y: i32, radius: i32) -> (i32, i32) {
        if radius <= 0 {
            return (x, y);
        }
        crate::touch::nearest(x, y, radius, |px, py| self.has_target(screen, px, py))
    }

    fn has_target(&self, screen: LobbyScreen, x: i32, y: i32) -> bool {
        match screen {
            LobbyScreen::Login if self.credits.visible => false,
            LobbyScreen::Login if self.key_crypt.contains(x, y) => true,
            LobbyScreen::Login => match self.overlay {
                Some(overlay) => self.notice(overlay).hit(x, y).is_some(),
                None => self.login.hit(x, y).is_some(),
            },
            LobbyScreen::Servers => self.servers.hit(x, y).is_some(),
            LobbyScreen::Characters => self.select.hit(x, y).is_some(),
            LobbyScreen::Create => self.create.hit(x, y).is_some(),
        }
    }

    pub fn wants_text(&self, screen: LobbyScreen) -> bool {
        match screen {
            LobbyScreen::Login => {
                self.overlay.is_none() && !self.credits.visible && self.login.auth_visible && matches!(self.login.focus, Field::Id | Field::Password)
            }
            LobbyScreen::Create => self.create.name_focus,
            _ => false,
        }
    }

    pub fn mouse_move(&mut self, x: i32, y: i32, lobby: &Lobby) {
        self.cursor = (x, y);
        if lobby.screen == LobbyScreen::Characters {
            self.select.mouse_move(x, y);
        }
        if lobby.screen == LobbyScreen::Servers {
            self.servers.help_bar_move(y);
        }
        if lobby.screen == LobbyScreen::Login
            && let Some(overlay) = self.overlay
        {
            self.notice_mut(overlay).mouse_move(y);
        }
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, lobby: &mut Lobby, font: &BitmapFont) {
        if let Some(enabled) = self.buttons.borrow().iter().rev().find(|button| button.contains(x, y)).map(|button| button.enabled) {
            self.sounds.push(killer_ui::interface_sound::button_click(enabled));
        }
        match lobby.screen {
            LobbyScreen::Login if self.credits.visible => {
                self.credits.click();
            }
            LobbyScreen::Login if self.replay_menu_shown() => {
                if let Some(menu) = self.replay_menu.as_mut() {
                    menu.function.mouse_down(x, y);
                }
            }
            LobbyScreen::Login if self.replay.visible => self.replay.mouse_down(x, y),
            LobbyScreen::Login if self.key_crypt.contains(x, y) => self.key_crypt.mouse_down(x, y),
            LobbyScreen::Login => match self.overlay {
                Some(overlay) => {
                    self.notice_mut(overlay).prepare(font);
                    self.notice_mut(overlay).mouse_down(x, y);
                }
                None => self.login.mouse_down(x, y),
            },
            LobbyScreen::Servers => {
                if !self.servers.help_bar_down(x, y) {
                    self.servers.mouse_down(x, y);
                    self.servers.mouse_down_help(x, y, font);
                }
            }
            LobbyScreen::Characters => {
                let on_window = self.select.hit(x, y).is_some() || self.select.combo.open || self.select.dialog.is_some();
                if let Some(action) = self.select.mouse_down(x, y) {
                    self.select_action(action, lobby);
                } else if !on_window && let Some(slot) = lobby.pick_slot((self.size.0.max(0) as u32, self.size.1.max(0) as u32), (x as f32, y as f32)) {
                    lobby.replay_selected(slot);
                    if let Some(action) = self.select.choose(slot) {
                        self.select_action(action, lobby);
                    }
                }
            }
            LobbyScreen::Create => {
                let actions = self.create.mouse_down(x, y);
                self.create_actions(actions, lobby);
            }
        }
    }

    pub fn mouse_up(&mut self, x: i32, y: i32, lobby: &mut Lobby, font: &BitmapFont) {
        let now = killer_platform::time::Instant::now();
        let double = self.last_click.is_some_and(|(at, lx, ly)| now.duration_since(at).as_millis() <= DOUBLE_CLICK_MS && (lx - x).abs() <= 3 && (ly - y).abs() <= 3);
        self.last_click = if double { None } else { Some((now, x, y)) };
        match lobby.screen {
            LobbyScreen::Login if self.credits.visible => {}
            LobbyScreen::Login if self.replay_menu_shown() => {
                let action = self.replay_menu.as_mut().and_then(|menu| menu.function.mouse_up(x, y));
                match action {
                    Some(FunctionAction::Options) => self.options_requested = true,
                    Some(FunctionAction::Exit) => self.commands.push(LobbyCommand::Exit),
                    Some(FunctionAction::Resume) => {
                        self.replay_state = 0;
                        self.replay_requests.push(ReplayRequest::PrivateStart { benchmark: false });
                    }
                    Some(FunctionAction::Benchmark) => {
                        self.replay_state = 0;
                        self.benchmark.reset();
                        self.replay_requests.push(ReplayRequest::PrivateStart { benchmark: true });
                    }
                    None => {}
                }
            }
            LobbyScreen::Login if self.replay.visible => {
                let action = if double { self.replay.double_click(x, y) } else { self.replay.mouse_up(x, y) };
                if let Some(action) = action {
                    self.replay_action(action);
                }
            }
            LobbyScreen::Login if self.key_crypt.pressed => {
                self.key_crypt.mouse_up(x, y);
            }
            LobbyScreen::Login => match self.overlay {
                Some(overlay) => {
                    if let Some(action) = self.notice_mut(overlay).mouse_up(x, y) {
                        self.notice_action(action, lobby);
                    }
                }
                None => {
                    if let Some(action) = self.login.mouse_up(x, y) {
                        self.login_action(action, lobby);
                    }
                }
            },
            LobbyScreen::Servers if self.servers.help_bar_up(x, y) => {}
            LobbyScreen::Servers => {
                let action = if double { self.servers.double_click(x, y) } else { self.servers.mouse_up(x, y) };
                if let Some(action) = action.or_else(|| self.servers.mouse_up_help(x, y, font)) {
                    self.server_action(action);
                }
            }
            LobbyScreen::Characters => {
                let action = self.select.mouse_up(x, y, |name| !Self::name_blocked(font, name));
                lobby.stop_voices |= std::mem::take(&mut self.select.stop_voices);
                if let Some(action) = action {
                    self.select_action(action, lobby);
                }
            }
            LobbyScreen::Create => {
                let actions = self.create.mouse_up(x, y, |name| Self::create_name_blocked(&self.name_rules, font, name));
                self.create_actions(actions, lobby);
            }
        }
    }

    pub fn wheel(&mut self, delta: f32, lobby: &Lobby, font: &BitmapFont) {
        let sign = if delta > 0.0 { 1 } else { -1 };
        match (lobby.screen, self.overlay) {
            (LobbyScreen::Login, Some(overlay)) => {
                self.notice_mut(overlay).prepare(font);
                self.notice_mut(overlay).wheel(sign * 120);
            }
            (LobbyScreen::Login, _) if self.credits.visible => {}
            (LobbyScreen::Login, _) if self.replay.visible => self.replay.wheel(if sign > 0 { 120 } else { -120 }),
            (LobbyScreen::Servers, _) => {
                let (x, y) = self.cursor;
                self.servers.wheel_at(x, y, sign);
            }
            _ => {}
        }
    }

    pub fn key(&mut self, key: InputKey, shift: bool, lobby: &mut Lobby, font: &BitmapFont) {
        match lobby.screen {
            LobbyScreen::Login if self.credits.visible => {
                if key == InputKey::Escape {
                    self.credits.key(killer_ui::lobby_credit::ESCAPE);
                }
            }
            LobbyScreen::Login if self.replay.visible => {
                let code = match key {
                    InputKey::Enter => Some(killer_ui::lobby_replay::KEY_ENTER),
                    InputKey::Escape => Some(killer_ui::lobby_replay::KEY_ESCAPE),
                    _ => None,
                };
                if let Some(action) = code.and_then(|code| self.replay.key(code)) {
                    self.replay_action(action);
                }
            }
            LobbyScreen::Login => match self.overlay {
                Some(overlay) => {
                    if key == InputKey::Enter {
                        let action = self.notice(overlay).enter();
                        self.notice_action(action, lobby);
                        if overlay != Overlay::Announce {
                            self.sounds.push(killer_ui::interface_sound::button_click(true));
                        }
                    }
                }
                None => {
                    if let Some(action) = self.login.key(key, shift) {
                        self.login_action(action, lobby);
                    }
                }
            },
            LobbyScreen::Servers => {
                if key == InputKey::Enter
                    && let Some(action) = self.servers.enter()
                {
                    self.server_action(action);
                }
            }
            LobbyScreen::Characters => {
                if key == InputKey::Enter {
                    let action = self.select.enter(|name| !Self::name_blocked(font, name));
                    lobby.stop_voices |= std::mem::take(&mut self.select.stop_voices);
                    if let Some(action) = action {
                        self.select_action(action, lobby);
                    }
                }
            }
            LobbyScreen::Create => self.create.key(key),
        }
    }

    fn login_action(&mut self, action: LoginAction, lobby: &mut Lobby) {
        match action {
            LoginAction::Login { account, password } => match lobby.login.config() {
                Some(config) => {
                    lobby.login.connecting = true;
                    lobby.login.status.clear();
                    self.login.set_message("");
                    self.commands.push(LobbyCommand::Connect { config, account, password });
                }
                None => self.say(killer_ui::lobby_login::SYSMSG_CONNECT_FAILED),
            },
            LoginAction::Exit => self.commands.push(LobbyCommand::Exit),
            LoginAction::Message(id) => self.say(id),
            LoginAction::CardCheck { value, .. } => self.commands.push(LobbyCommand::CardAnswer(value)),
            LoginAction::Credits => self.credits.show(),
            LoginAction::Options => self.options_requested = true,
            LoginAction::Replay => {
                self.replay.visible = true;
                self.replay_requests.push(ReplayRequest::List);
            }
        }
    }

    fn replay_label(&self, id: i32) -> Option<String> {
        self.messages_sys.get(&(id as u32)).cloned()
    }

    fn replay_action(&mut self, action: ReplayAction) {
        match action {
            ReplayAction::Load { name, option, chat } => self.replay_requests.push(ReplayRequest::Load { name, option, chat }),
            ReplayAction::Delete { path } => {
                self.replay_requests.push(ReplayRequest::Delete { path });
                self.replay_requests.push(ReplayRequest::List);
            }
            ReplayAction::Refresh => self.replay_requests.push(ReplayRequest::List),
            ReplayAction::Back => self.replay.visible = false,
        }
    }

    fn notice(&self, overlay: Overlay) -> &Notice {
        match overlay {
            Overlay::Eula => &self.eula,
            Overlay::Warning => &self.warning,
            Overlay::Announce => &self.announce,
        }
    }

    fn notice_mut(&mut self, overlay: Overlay) -> &mut Notice {
        match overlay {
            Overlay::Eula => &mut self.eula,
            Overlay::Warning => &mut self.warning,
            Overlay::Announce => &mut self.announce,
        }
    }

    fn notice_action(&mut self, action: NoticeAction, lobby: &mut Lobby) {
        match action {
            NoticeAction::Agree if self.language_type == LANGUAGE_CHINA => {
                self.overlay = Some(Overlay::Announce);
            }
            NoticeAction::Agree | NoticeAction::Announce if self.has_warning && lobby.pending_notice.is_some_and(|notice| notice.shows_warning()) => {
                self.overlay = Some(Overlay::Warning);
            }
            NoticeAction::Agree | NoticeAction::Announce | NoticeAction::Ok => self.answer(lobby, true),
            NoticeAction::Disagree => self.answer(lobby, false),
        }
    }

    fn server_action(&mut self, action: ServerAction) {
        match action {
            ServerAction::Choose { id, .. } => {
                self.ping_edits = self.ping_cache.finish(crate::ping_cache::now_key());
                self.commands.push(LobbyCommand::ChooseServer(id));
            }
            ServerAction::Cancel => {
                self.ping_edits = self.ping_cache.finish(crate::ping_cache::now_key());
                self.commands.push(LobbyCommand::Back);
            }
            ServerAction::Help(link) => self.help_request = Some(link),
        }
    }

    fn select_action(&mut self, action: SelectAction, lobby: &mut Lobby) {
        match action {
            SelectAction::Select(Some(slot)) => lobby.select(slot),
            SelectAction::Select(None) => lobby.deselect(),
            SelectAction::Start(slot) => {
                lobby.busy = true;
                self.commands.push(LobbyCommand::Select(slot));
            }
            SelectAction::Create => self.commands.push(LobbyCommand::NewCharacter),
            SelectAction::Delete(slot) => self.commands.push(LobbyCommand::Delete(slot)),
            SelectAction::Restore(slot) => self.commands.push(LobbyCommand::Restore(slot)),
            SelectAction::Exit => self.commands.push(LobbyCommand::Back),
            SelectAction::Message(id) => self.say(id),
        }
    }

    fn create_actions(&mut self, actions: Vec<CreateAction>, lobby: &mut Lobby) {
        let mut camera: Option<(String, bool)> = None;
        for action in actions {
            match action {
                CreateAction::Message(id) => self.say(id),
                CreateAction::Camera { tag, reverse } => camera = Some((tag, reverse)),
                CreateAction::Changed => {
                    lobby.create.race = self.create.race();
                    lobby.create.class = self.create.class();
                    lobby.create.female = self.create.sex().map(|sex| sex == 1);
                    lobby.apply_choice_with(camera.take(), false);
                    lobby.apply_look(self.create.model_index(), 0, 0, 0);
                    lobby.turn(None, 0.0);
                    self.login.set_message("");
                }
                CreateAction::Look => {
                    let value = |index: usize| self.create.controls[index].index.unwrap_or(0) as i32;
                    lobby.create.hair_style = value(3);
                    lobby.create.hair_color = value(4);
                    lobby.create.face = value(5);
                    lobby.apply_look(self.create.model_index(), value(3), value(4), value(5));
                }
                CreateAction::Zoom => lobby.zoom(self.create.zoomed),
                CreateAction::Turn(rate) => lobby.turn(self.create.model_index(), rate),
                CreateAction::Stop => lobby.turn(None, 0.0),
                CreateAction::Previous => {
                    lobby.stop_voices = true;
                    lobby.turn(None, 0.0);
                    lobby.enter(LobbyScreen::Characters);
                }
                CreateAction::Create { name, race, sex, class_id, style, color, face } => {
                    lobby.create.sending = true;
                    self.commands.push(LobbyCommand::Create(Box::new(NewCharacter {
                        name,
                        race: race as i32,
                        sex: sex as i32,
                        class_id,
                        int_: lobby.template.int_,
                        str_: lobby.template.str_,
                        con: lobby.template.con,
                        men: lobby.template.men,
                        dex: lobby.template.dex,
                        wit: lobby.template.wit,
                        hair_style: style as i32,
                        hair_color: color as i32,
                        face: face as i32,
                    })));
                }
            }
        }
    }
}

fn card(entry: &l2_net::game::CharSelectEntry) -> CharCard {
    CharCard {
        name: entry.name.clone(),
        level: entry.level,
        class_name: crate::lobby::classes::class_name(entry.class_id),
        hp: entry.hp as i32,
        max_hp: entry.max_hp as i32,
        mp: entry.mp as i32,
        max_mp: entry.max_mp as i32,
        exp: i64::from(entry.exp),
        sp: entry.sp,
        karma: entry.karma,
        delete_seconds: entry.delete_days.max(0),
        dead: entry.delete_days < 0,
        last_used: entry.last_used != 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lobby::LobbyScreen;
    use glam::Vec3;
    use killer_assets::client::Client;
    use killer_assets::textures::{BankSource, TextureBank, write_png};
    use killer_ui::lobby_select::NameTag;
    use l2_net::game::CharSelectEntry;
    use l2_net::login::{GameServer, ServerExtra};
    use std::path::PathBuf;

    fn entry(name: &str, class_id: i32, level: i32, delete_days: i32, last_used: bool) -> CharSelectEntry {
        CharSelectEntry {
            name: name.into(), char_id: 1, login: String::new(), clan_id: 0, sex: 0, race: 0, base_class_id: class_id, active: 0,
            x: 0, y: 0, z: 0, hp: 800.0, mp: 300.0, sp: 1234, exp: 9000, level, karma: 0, paperdoll_objects: [0; 16],
            paperdoll_items: [0; 16], hair_style: 0, hair_color: 0, face: 0, max_hp: 1000.0, max_mp: 400.0, delete_days, class_id,
            last_used: i32::from(last_used), enchant_effect: 0,
        }
    }

    fn font() -> BitmapFont {
        crate::ui2d::tests::small_font_for_lobby()
    }

    fn host() -> (LobbyC4, Lobby, DatTables) {
        let tables = DatTables::default();
        let c4 = LobbyC4::new((1024, 768), &tables, &[]);
        let config = crate::lobby::config::ServerConfig { host: "127.0.0.1".into(), port: 2106 };
        (c4, Lobby::new(config), tables)
    }

    fn click(c4: &mut LobbyC4, lobby: &mut Lobby, font: &BitmapFont, x: i32, y: i32) {
        c4.mouse_down(x, y, lobby, font);
        c4.mouse_up(x, y, lobby, font);
    }

    fn type_text(c4: &mut LobbyC4, lobby: &mut Lobby, font: &BitmapFont, text: &str) {
        for c in text.chars() {
            c4.key(InputKey::Char(c), false, lobby, font);
        }
    }

    #[test]
    fn typing_the_account_and_the_password_and_pressing_enter_asks_to_connect() {
        let (mut c4, mut lobby, _tables) = host();
        let font = font();
        c4.update(&mut lobby, 16);
        type_text(&mut c4, &mut lobby, &font, "conta");
        c4.key(InputKey::Char('\t'), false, &mut lobby, &font);
        type_text(&mut c4, &mut lobby, &font, "senha");
        c4.key(InputKey::Enter, false, &mut lobby, &font);
        let commands = c4.take_commands();
        assert!(matches!(commands.as_slice(), [LobbyCommand::Connect { account, password, .. }] if account == "conta" && password == "senha"));
        assert!(lobby.login.connecting);
    }

    fn host_with_notices(warning: bool) -> (LobbyC4, Lobby, DatTables) {
        let tables = DatTables {
            eula: Some(l2_dat::eula::Eula { eula: "termos".into(), test_server: String::new() }),
            warning_notice: warning.then(|| "aviso".to_string()),
            ..Default::default()
        };
        let c4 = LobbyC4::new((1024, 768), &tables, &[]);
        let config = crate::lobby::config::ServerConfig { host: "127.0.0.1".into(), port: 2106 };
        (c4, Lobby::new(config), tables)
    }

    #[test]
    fn the_eula_comes_after_login_ok_and_the_warning_after_agree_when_the_flag_asks_for_it() {
        let (mut c4, mut lobby, _tables) = host_with_notices(true);
        let font = font();
        c4.update(&mut lobby, 16);
        lobby.show_notice(l2_net::login::LoginNotice { flags: 1, forbidden: [0; 16] });
        c4.update(&mut lobby, 16);
        assert_eq!(c4.overlay, Some(Overlay::Eula));
        assert!(c4.take_commands().is_empty());
        c4.key(InputKey::Enter, false, &mut lobby, &font);
        assert_eq!(c4.overlay, Some(Overlay::Warning));
        assert!(c4.take_commands().is_empty());
        c4.key(InputKey::Enter, false, &mut lobby, &font);
        assert_eq!(c4.overlay, None);
        assert!(lobby.pending_notice.is_none());
        assert!(matches!(c4.take_commands().as_slice(), [LobbyCommand::NoticeAnswer(true)]));
    }

    #[test]
    fn the_chinese_client_shows_the_announce_after_agree_and_its_ok_follows_the_flag() {
        let (mut c4, mut lobby, tables) = host_with_notices(true);
        c4.language_type = LANGUAGE_CHINA;
        let _ = tables;
        let font = font();
        c4.update(&mut lobby, 16);
        lobby.show_notice(l2_net::login::LoginNotice { flags: 1, forbidden: [0; 16] });
        c4.update(&mut lobby, 16);
        assert_eq!(c4.overlay, Some(Overlay::Eula));
        c4.key(InputKey::Enter, false, &mut lobby, &font);
        assert_eq!(c4.overlay, Some(Overlay::Announce));
        assert!(c4.take_commands().is_empty());
        let (ox, oy) = c4.announce.origin();
        let ok = killer_ui::lobby_notice::OK_RECT;
        click(&mut c4, &mut lobby, &font, ox + ok.x + 3, oy + ok.y + 3);
        assert_eq!(c4.overlay, Some(Overlay::Warning));
        c4.key(InputKey::Enter, false, &mut lobby, &font);
        assert_eq!(c4.overlay, None);
        assert!(matches!(c4.take_commands().as_slice(), [LobbyCommand::NoticeAnswer(true)]));
    }

    #[test]
    fn without_eula_the_chinese_client_shows_the_announce_and_the_others_answer_at_once() {
        let tables = DatTables::default();
        let config = crate::lobby::config::ServerConfig { host: "127.0.0.1".into(), port: 2106 };
        let mut lobby = Lobby::new(config);
        let mut c4 = LobbyC4::new((1024, 768), &tables, &[]);
        c4.update(&mut lobby, 16);
        lobby.show_notice(l2_net::login::LoginNotice { flags: 0x3ea, forbidden: [0; 16] });
        c4.update(&mut lobby, 16);
        assert_eq!(c4.overlay, None);
        assert!(matches!(c4.take_commands().as_slice(), [LobbyCommand::NoticeAnswer(true)]));
        c4.language_type = LANGUAGE_CHINA;
        lobby.show_notice(l2_net::login::LoginNotice { flags: 0x3ea, forbidden: [0; 16] });
        c4.update(&mut lobby, 16);
        assert_eq!(c4.overlay, Some(Overlay::Announce));
        assert!(c4.take_commands().is_empty());
        c4.key(InputKey::Enter, false, &mut lobby, &font());
        assert!(matches!(c4.take_commands().as_slice(), [LobbyCommand::NoticeAnswer(true)]));
    }

    #[test]
    fn the_replay_menu_buttons_move_the_manager_state_and_the_benchmark_clears_its_frames_0x100f8e00() {
        let font = font();
        let config = crate::lobby::config::ServerConfig { host: "127.0.0.1".into(), port: 2106 };
        let mut lobby = Lobby::new(config);
        let tables = DatTables { login_ini: killer_assets::login_ini::LoginIni { replay_mode: true, ..Default::default() }, ..Default::default() };
        let mut c4 = LobbyC4::new((1024, 768), &tables, &[]);
        c4.update(&mut lobby, 16);
        assert!(c4.replay_menu_shown());
        c4.benchmark.update_frame_time(0.01);
        let (x, y) = (1024 - 154 + 14, 768 - 236 + 46);
        c4.mouse_down(x, y, &mut lobby, &font);
        c4.mouse_up(x, y, &mut lobby, &font);
        assert!(!c4.replay_menu_shown() && c4.replay_state == 0);
        assert!(c4.benchmark.frame_times.is_empty());
        assert_eq!(c4.take_replay_requests(), vec![ReplayRequest::PrivateStart { benchmark: true }]);
        c4.replay_state = killer_world::replay_private::STATE_MENU;
        assert!(c4.replay_menu_shown());
    }

    #[test]
    fn the_key_crypt_warning_shows_only_with_the_ini_flags_and_ok_hides_it() {
        let font = font();
        let config = crate::lobby::config::ServerConfig { host: "127.0.0.1".into(), port: 2106 };
        let mut lobby = Lobby::new(config);
        let quiet = LobbyC4::new((1024, 768), &DatTables::default(), &[]);
        assert!(!quiet.key_crypt.visible);
        let tables = DatTables { login_ini: killer_assets::login_ini::LoginIni { use_key_crypt: true, replay_mode: false, test_server: false, debug_log: false, auto_logon: None }, ..Default::default() };
        let mut c4 = LobbyC4::new((1024, 768), &tables, &[]);
        c4.update(&mut lobby, 16);
        assert!(c4.key_crypt.visible);
        let (ox, oy) = c4.key_crypt.origin();
        let ok = killer_ui::lobby_keycrypt::OK_RECT;
        click(&mut c4, &mut lobby, &font, ox + ok.x + 3, oy + ok.y + 3);
        assert!(!c4.key_crypt.visible);
    }

    #[test]
    fn pressing_a_painted_lobby_button_plays_the_ncbutton_click() {
        let (mut c4, mut lobby, _) = host();
        let font = font();
        let list = c4.frame(&lobby, &font, &[]);
        let button = *list.buttons.first().expect("o login pinta botões");
        c4.mouse_down(button.x + 1, button.y + 1, &mut lobby, &font);
        assert_eq!(c4.sounds, vec!["InterfaceSound.click_01"]);
    }

    #[test]
    fn the_auto_log_on_credentials_fill_the_login_once_and_focus_the_ok_button() {
        let auto = killer_assets::login_ini::AutoLogOn { id: "conta".into(), password: "senha".into() };
        let tables = DatTables { login_ini: killer_assets::login_ini::LoginIni { use_key_crypt: false, replay_mode: false, test_server: false, debug_log: false, auto_logon: Some(auto) }, ..Default::default() };
        let config = crate::lobby::config::ServerConfig { host: "127.0.0.1".into(), port: 2106 };
        let mut lobby = Lobby::new(config);
        let mut c4 = LobbyC4::new((1024, 768), &tables, &[]);
        c4.update(&mut lobby, 16);
        assert_eq!(c4.login.id.text, "conta");
        assert_eq!(c4.login.password.text, "senha");
        assert_eq!(c4.login.focus, killer_ui::lobby_login::Field::Login);
        assert!(c4.auto_logon.is_none());
        assert!(c4.select.locked, "AutoLog trava Criar e o botão +0xE8 do lobby");
    }

    #[test]
    fn an_even_flag_skips_the_warning_and_disagree_goes_back_without_a_message() {
        let (mut c4, mut lobby, _tables) = host_with_notices(true);
        let font = font();
        c4.update(&mut lobby, 16);
        lobby.show_notice(l2_net::login::LoginNotice { flags: 0x3ea, forbidden: [0; 16] });
        c4.update(&mut lobby, 16);
        c4.key(InputKey::Enter, false, &mut lobby, &font);
        assert!(matches!(c4.take_commands().as_slice(), [LobbyCommand::NoticeAnswer(true)]));
        lobby.show_notice(l2_net::login::LoginNotice::default());
        c4.update(&mut lobby, 16);
        assert_eq!(c4.overlay, Some(Overlay::Eula));
        let (ox, oy) = c4.eula.origin();
        let disagree = killer_ui::lobby_notice::DISAGREE_RECT;
        click(&mut c4, &mut lobby, &font, ox + disagree.x + 3, oy + disagree.y + 3);
        assert!(matches!(c4.take_commands().as_slice(), [LobbyCommand::Back]));
        assert_eq!(c4.overlay, None);
        assert!(lobby.login.status.is_empty());
    }

    #[test]
    fn without_an_eula_text_the_notice_is_answered_at_once_and_forbidden_servers_come_from_the_bitmap() {
        let (mut c4, mut lobby, _tables) = host();
        c4.update(&mut lobby, 16);
        let mut forbidden = [0u8; 16];
        forbidden[0] = 0b0000_0010;
        lobby.show_notice(l2_net::login::LoginNotice { flags: 1, forbidden });
        c4.update(&mut lobby, 16);
        assert_eq!(c4.overlay, None);
        assert!(matches!(c4.take_commands().as_slice(), [LobbyCommand::NoticeAnswer(true)]));
        let server = |id: u8| GameServer { id, ip: [127, 0, 0, 1], port: 7777, players: 1, max_players: 100, online: true, extra: ServerExtra { status: 1, ..Default::default() } };
        lobby.set_servers(&[server(1), server(2)]);
        c4.update(&mut lobby, 16);
        assert_eq!(c4.servers.entries.iter().map(|entry| entry.forbidden).collect::<Vec<_>>(), [false, true]);
    }

    #[test]
    fn a_fresh_s_info_cache_answers_pings_and_confirming_writes_the_date_and_new_times() {
        let (mut c4, mut lobby, _tables) = host();
        let now = crate::ping_cache::now_key();
        let ini = killer_ui::ini::parse_ini(&format!("[Date]\nTime={now}\n[1]\nTime=33\n[2]\nTime=9999\n"));
        c4.set_ping_cache(crate::ping_cache::PingCache::from_ini(&ini, now));
        c4.update(&mut lobby, 16);
        let server = |id: u8| GameServer { id, ip: [10, 0, 0, id], port: 7777, players: 1, max_players: 100, online: true, extra: ServerExtra { status: 1, ..Default::default() } };
        lobby.set_servers(&[server(1), server(2)]);
        c4.update(&mut lobby, 16);
        assert_eq!(c4.take_ping_requests(), [(2, [10, 0, 0, 2], 7777)]);
        assert_eq!(c4.servers.entries.iter().find(|entry| entry.id == 1).map(|entry| entry.ping), Some(33));
        assert_eq!(c4.servers.pending_pings, 1);
        c4.set_ping(2, 70);
        assert!(c4.take_ping_edits().is_empty());
    }

    #[test]
    fn the_server_list_asks_for_one_ping_per_server_once_and_shows_the_replies() {
        let (mut c4, mut lobby, _tables) = host();
        c4.update(&mut lobby, 16);
        let server = |id: u8| GameServer { id, ip: [10, 0, 0, id], port: 7777, players: 1, max_players: 100, online: true, extra: ServerExtra { status: 1, ..Default::default() } };
        lobby.set_servers(&[server(1), server(2)]);
        c4.update(&mut lobby, 16);
        assert_eq!(c4.take_ping_requests(), [(1, [10, 0, 0, 1], 7777), (2, [10, 0, 0, 2], 7777)]);
        lobby.revision = lobby.revision.wrapping_add(1);
        c4.update(&mut lobby, 16);
        assert!(c4.take_ping_requests().is_empty());
        c4.set_ping(2, 80);
        c4.set_ping(1, 9999);
        assert_eq!(c4.servers.entries.iter().find(|entry| entry.id == 2).map(|entry| entry.ping), Some(80));
        assert_eq!(c4.servers.pending_pings, 0);
    }

    #[test]
    fn replay_button_opens_the_list_window_and_load_delete_back_become_requests() {
        let (mut c4, mut lobby, _tables) = host();
        let font = font();
        c4.update(&mut lobby, 16);
        let (fx, fy) = c4.login.function_origin();
        let rect = killer_ui::lobby_login::FUNCTION_BUTTON_RECTS[4];
        c4.mouse_down(fx + rect.x + 1, fy + rect.y + 1, &mut lobby, &font);
        c4.mouse_up(fx + rect.x + 1, fy + rect.y + 1, &mut lobby, &font);
        assert!(c4.replay.visible);
        assert_eq!(c4.take_replay_requests(), [ReplayRequest::List]);
        c4.set_replay_files(&["Rep00000.l2r".to_string(), "Rep00001.l2r".to_string()]);
        let (ox, oy) = c4.replay.origin();
        let row = (ox + 40, oy + 11 + 19 + 3);
        c4.mouse_down(row.0, row.1, &mut lobby, &font);
        c4.mouse_up(row.0, row.1, &mut lobby, &font);
        c4.key(InputKey::Enter, false, &mut lobby, &font);
        assert_eq!(c4.take_replay_requests(), [ReplayRequest::Load { name: "Rep00000".into(), option: false, chat: false }]);
        let delete = killer_ui::lobby_replay::DELETE_RECT;
        c4.mouse_down(ox + delete.x + 1, oy + delete.y + 1, &mut lobby, &font);
        c4.mouse_up(ox + delete.x + 1, oy + delete.y + 1, &mut lobby, &font);
        assert_eq!(c4.take_replay_requests(), [ReplayRequest::Delete { path: "..\\REPLAY\\Rep00000.L2R".into() }, ReplayRequest::List]);
        let frame = c4.frame(&lobby, &font, &[]);
        assert!(frame.quads.iter().any(|quad| quad.texture.as_str() == killer_ui::lobby_replay::BACK));
        c4.key(InputKey::Escape, false, &mut lobby, &font);
        assert!(!c4.replay.visible);
        let frame = c4.frame(&lobby, &font, &[]);
        assert!(frame.quads.iter().all(|quad| quad.texture.as_str() != killer_ui::lobby_replay::BACK));
    }

    #[test]
    fn production_team_opens_the_credits_which_rotate_by_timer_and_click_and_close_with_escape() {
        let tables = DatTables {
            credits: (0..3).map(|id| l2_dat::creditgrp::Credit { id, html: format!("c{id}.htm"), image: "L2UI_CH3.credit".into(), time: 10, align: 5 }).collect(),
            ..Default::default()
        };
        let mut c4 = LobbyC4::new((1024, 768), &tables, &[]);
        let mut lobby = Lobby::new(crate::lobby::config::ServerConfig { host: "127.0.0.1".into(), port: 2106 });
        let font = font();
        c4.update(&mut lobby, 16);
        assert!(!c4.credits_visible());
        c4.login_action(LoginAction::Credits, &mut lobby);
        assert!(c4.credits_visible());
        assert_eq!(c4.take_credit_request().as_deref(), Some("..\\L2Text\\c0.htm"));
        c4.set_credit_html("<html><body>Produção</body></html>", &font, &|_, _| None);
        let list = c4.frame(&lobby, &font, &[]);
        assert!(list.quads.iter().any(|quad| quad.texture == "L2UI_CH3.credit" && quad.w == 1024.0));
        c4.update(&mut lobby, 10_100);
        assert_eq!(c4.take_credit_request().as_deref(), Some("..\\L2Text\\c1.htm"));
        c4.mouse_down(5, 5, &mut lobby, &font);
        assert_eq!(c4.take_credit_request().as_deref(), Some("..\\L2Text\\c2.htm"));
        c4.key(InputKey::Char('x'), false, &mut lobby, &font);
        assert!(c4.credits_visible());
        c4.key(InputKey::Escape, false, &mut lobby, &font);
        assert!(!c4.credits_visible());
        assert!(c4.take_commands().is_empty());
        c4.login_action(LoginAction::Credits, &mut lobby);
        c4.mouse_down(5, 5, &mut lobby, &font);
        c4.mouse_down(5, 5, &mut lobby, &font);
        c4.mouse_down(5, 5, &mut lobby, &font);
        assert!(!c4.credits_visible());
    }

    #[test]
    fn the_server_list_chooses_the_server_with_enter_and_cancel_goes_back() {
        let (mut c4, mut lobby, _tables) = host();
        let font = font();
        let server = |id: u8, last: bool| GameServer {
            id, ip: [127, 0, 0, 1], port: 7777, players: 10, max_players: 100, online: true,
            extra: ServerExtra { age: 15, pvp: 0, status: 1, flags: 0, brackets: 0, last },
        };
        lobby.set_servers(&[server(1, false), server(2, true)]);
        c4.update(&mut lobby, 16);
        assert_eq!(c4.servers.entries.len(), 2);
        assert_eq!(c4.servers.entries[0].id, 2);
        c4.key(InputKey::Enter, false, &mut lobby, &font);
        assert_eq!(c4.take_commands(), vec![LobbyCommand::ChooseServer(2)]);
        c4.servers.tick(killer_ui::lobby_servers::LOGIN_LOCK_MS);
        let (ox, oy) = c4.servers.origin();
        let cancel = killer_ui::lobby_servers::CANCEL_RECT;
        click(&mut c4, &mut lobby, &font, ox + cancel.x + 2, oy + cancel.y + 2);
        assert_eq!(c4.take_commands(), vec![LobbyCommand::Back]);
    }

    #[test]
    fn the_character_screen_preselects_the_last_used_one_and_start_enters_the_game() {
        let (mut c4, mut lobby, _tables) = host();
        let font = font();
        lobby.set_characters(vec![entry("A", 0, 10, 0, false), entry("B", 0, 20, 0, true)]);
        c4.update(&mut lobby, 16);
        assert_eq!(c4.select.selected, Some(1));
        c4.key(InputKey::Enter, false, &mut lobby, &font);
        assert_eq!(c4.take_commands(), vec![LobbyCommand::Select(1)]);
    }

    #[test]
    fn deleting_asks_first_and_creating_a_character_walks_through_the_form() {
        let (mut c4, mut lobby, _tables) = host();
        let font = font();
        lobby.set_characters(vec![entry("A", 0, 10, 0, true)]);
        c4.update(&mut lobby, 16);
        let function = c4.select.function_origin();
        let delete = killer_ui::lobby_select::DELETE_RECT;
        click(&mut c4, &mut lobby, &font, function.0 + delete.x + 2, function.1 + delete.y + 2);
        assert!(c4.select.dialog.is_some());
        assert!(c4.take_commands().is_empty());
        let origin = c4.select.dialog_origin();
        let yes = c4.select.dialog_view().unwrap().buttons()[0].rect;
        click(&mut c4, &mut lobby, &font, origin.0 + yes.x + 2, origin.1 + yes.y + 2);
        assert_eq!(c4.take_commands(), vec![LobbyCommand::Delete(0)]);
        let create = killer_ui::lobby_select::CREATE_RECT;
        click(&mut c4, &mut lobby, &font, function.0 + create.x + 2, function.1 + create.y + 2);
        assert_eq!(lobby.screen, LobbyScreen::Characters);
        assert_eq!(c4.take_commands(), vec![LobbyCommand::NewCharacter]);
        lobby.new_character_result(false);
        assert_eq!((lobby.screen, lobby.system_message), (LobbyScreen::Characters, Some(crate::lobby::NEW_CHARACTER_FAIL_SYSMSG)));
        c4.update(&mut lobby, 16);
        assert_eq!(lobby.system_message, None);
        for (reason, id) in [(1, Some(306)), (2, Some(541)), (3, Some(540)), (0, None), (4, None)] {
            lobby.delete_failed(reason);
            assert_eq!(lobby.system_message, id);
            c4.update(&mut lobby, 16);
            assert_eq!(lobby.system_message, None);
        }
        let first = l2_net::game::CharTemplate { race: 0, class_id: 0, str_: 1, dex: 2, con: 3, int_: 4, wit: 5, men: 6 };
        let last = l2_net::game::CharTemplate { race: 4, class_id: 53, str_: 40, dex: 30, con: 43, int_: 21, wit: 11, men: 25 };
        lobby.set_templates(&l2_net::game::CharTemplates { templates: vec![first, last] });
        lobby.set_templates(&l2_net::game::CharTemplates { templates: Vec::new() });
        lobby.new_character_result(true);
        assert_eq!(lobby.screen, LobbyScreen::Create);
        c4.update(&mut lobby, 16);
        c4.create.choose_race(0);
        c4.create.choose_class(0);
        c4.create.choose_sex(0);
        type_text(&mut c4, &mut lobby, &font, "Novo");
        let (fx, fy) = c4.create.function_origin();
        let button = killer_ui::lobby_create::CREATE_RECT;
        click(&mut c4, &mut lobby, &font, fx + button.x + 2, fy + button.y + 2);
        let commands = c4.take_commands();
        assert!(matches!(commands.as_slice(), [LobbyCommand::Create(character)] if character.name == "Novo" && character.class_id == 0
            && [character.int_, character.str_, character.con, character.men, character.dex, character.wit] == [21, 40, 43, 25, 30, 11]));
    }

    #[test]
    fn a_new_name_is_refused_when_it_hits_the_obscene_list_or_an_npc_name() {
        let mut tables = DatTables { obscene: vec!["bad".to_string()], ..Default::default() };
        tables.npcname.insert(1, l2_dat::npcname::NpcName { id: 1, name: "Gatekeeper".into(), title: String::new(), color: [0; 3] });
        let rules = NameRules::from_tables(&tables);
        assert!(rules.refuses(""));
        assert!(rules.refuses("xBADx"));
        assert!(rules.refuses("GATEKEEPER"));
        assert!(!rules.refuses("Gatekeepers"));
        assert!(!rules.refuses("Novo"));
    }

    #[test]
    #[ignore]
    fn the_c4_lobby_screens_render_to_png() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let out = std::env::var_os("KILLER_LOBBY_OUT").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"));
        let size = std::env::var("KILLER_LOBBY_SIZE").ok().and_then(|text| text.split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))).unwrap_or((1024u32, 768u32));
        let mut client = Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        let ui = crate::ui2d::GameUi::load(&mut client, (size.0 as i32, size.1 as i32)).unwrap();
        let mut c4 = LobbyC4::new((size.0 as i32, size.1 as i32), &tables, &[]);
        let mut lobby = Lobby::new(Default::default());
        let mut headless = killer_render::headless::Headless::new(size.0, size.1).unwrap();
        let mut bank = TextureBank::new();
        let servers: Vec<GameServer> = (1..=4)
            .map(|id| GameServer {
                id,
                ip: [127, 0, 0, 1],
                port: 7777,
                players: 400 * u16::from(id),
                max_players: 1000,
                online: id != 3,
                extra: ServerExtra { age: 15, pvp: u8::from(id == 2), status: u8::from(id != 3), flags: 0, brackets: 0, last: id == 2 },
            })
            .collect();
        let tags = vec![NameTag { x: 300, y: 300, name: "Heroi".into(), color: karma_color(0), second_line: None }];
        for (name, screen) in [("login", LobbyScreen::Login), ("servers", LobbyScreen::Servers), ("select", LobbyScreen::Characters), ("create", LobbyScreen::Create)] {
            match screen {
                LobbyScreen::Servers => lobby.set_servers(&servers),
                LobbyScreen::Characters => lobby.set_characters(vec![entry("Heroi", 0, 20, 0, true), entry("Mago", 10, 40, 7200, false), entry("Morto", 0, 5, -1, false)]),
                other => lobby.enter(other),
            }
            c4.update(&mut lobby, 16);
            if screen == LobbyScreen::Servers {
                c4.overlay = None;
                if let Some(link) = c4.take_help_request() {
                    c4.set_help(&client.read_help_text(&link).unwrap(), &ui.small, &|_, _| None);
                }
            }
            if screen == LobbyScreen::Create {
                c4.create.choose_race(0);
                c4.create.choose_class(0);
                c4.create.choose_sex(0);
            }
            if screen == LobbyScreen::Login {
                c4.login.id.text = "conta".into();
                c4.login.password.text = "segredo".into();
                c4.login.set_message("Hello");
            }
            let list = c4.frame(&lobby, &ui.small, &tags);
            headless.renderer_mut().set_ui(&list, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
            let environment = killer_render::environment::Environment::default();
            let camera = crate::camera::OrbitCamera::default().camera();
            let rgba = headless.render(&camera, &environment).unwrap();
            write_png(&out.join(format!("lobby_c4_{name}.png")), size.0, size.1, &rgba).unwrap();
        }
        lobby.enter(LobbyScreen::Login);
        c4.update(&mut lobby, 16);
        c4.set_credit_font(crate::ui2d::load_named_font(&mut client, killer_ui::lobby_credit::FONT_TEXTURE, killer_ui::lobby_credit::FONT_FILE).ok());
        c4.credits.show();
        let link = c4.take_credit_request().unwrap();
        c4.set_credit_html(&client.read_help_text(&link).unwrap_or_else(|e| panic!("{link}: {e}")), &ui.small, &|_, _| None);
        let list = c4.frame(&lobby, &ui.small, &tags);
        headless.renderer_mut().set_ui(&list, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        let rgba = headless.render(&crate::camera::OrbitCamera::default().camera(), &killer_render::environment::Environment::default()).unwrap();
        write_png(&out.join("lobby_c4_credits.png"), size.0, size.1, &rgba).unwrap();
        let mut ui = ui;
        ui.set_tab_names(&tables);
        c4.credits.visible = false;
        if let Some(window) = ui.desktop.window_mut(killer_ui::window::WindowId::OPTION) {
            window.visible = true;
        }
        ui.sync_option_window();
        let mut list = c4.frame(&lobby, &ui.small, &tags);
        if let Some(window) = ui.desktop.window(killer_ui::window::WindowId::OPTION) {
            ui.paint_window(&mut list, window, &killer_world::World::default(), &[], None, &crate::ui2d::WorldViews::default());
        }
        headless.renderer_mut().set_ui(&list, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        let rgba = headless.render(&crate::camera::OrbitCamera::default().camera(), &killer_render::environment::Environment::default()).unwrap();
        write_png(&out.join("lobby_c4_options.png"), size.0, size.1, &rgba).unwrap();
        let _ = Vec3::ZERO;
    }
}
