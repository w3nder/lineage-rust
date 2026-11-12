use crate::clock::Clock;
use crate::brain::{Brain, BrainEvent};
use crate::env::Env;
use crate::names::{IdNames, Names};
use crate::pacing::Pacing;
use crate::profile::Profile;
use crate::script_host::{ScriptRun, ScriptStatus, SessionHost};
use crate::transport::Transport;
use killer_bot_script::{EventKind, ScriptError, Status, Value, Vm};
use killer_world::chat::{ChatLine, ChatLog};
use std::collections::HashSet;
use killer_world::{NetCommand, NetEvent, SessionEvent, World, WorldEvent};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotState {
    Connecting,
    ChoosingServer,
    ChoosingCharacter,
    EnteringWorld,
    InWorld,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BotEvent {
    State(BotState),
    Log(String),
    Died,
    Revived,
    Refused(String),
    Alert(String),
    BottingStopped,
}

pub fn create_failure(reason: i32) -> &'static str {
    match reason {
        0 => "criação falhou",
        1 => "muitos personagens",
        2 => "nome já existe",
        3 => "nome com 16 caracteres ou menos",
        4 => "nome incorreto",
        5 => "não pode criar personagem",
        _ => "recusado",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BotStats {
    pub commands_sent: u64,
    pub deaths: u32,
    pub online_seconds: u64,
}

pub struct BotSession {
    profile: Profile,
    transport: Box<dyn Transport>,
    clock: Arc<dyn Clock>,
    pacing: Pacing,
    world: World,
    state: BotState,
    brain: Brain,
    names: Box<dyn Names>,
    sweepable: HashSet<i32>,
    spoiled: HashSet<i32>,
    fresh_chat: Vec<ChatLine>,
    pub english: bool,
    dead: bool,
    pub(crate) manual_pause: bool,
    pub(crate) automation: bool,
    chat: ChatLog,
    packet_log: crate::packets::PacketLog,
    pub(crate) servers: Vec<l2_net::login::GameServer>,
    pub(crate) characters: Vec<l2_net::game::CharSelectEntry>,
    templates: Vec<l2_net::game::CharTemplate>,
    commands_sent: u64,
    deaths: u32,
    entered_at: Option<Duration>,
    script: Option<ScriptRun>,
    script_events: Vec<(EventKind, Value)>,
    watch: Watch,
    stats: crate::stats::SessionStats,
    world_ticked_at: Option<Duration>,
    ping: Ping,
    own_casts: Vec<(i32, i32)>,
}

#[derive(Debug, Default)]
struct Watch {
    in_combat: bool,
    target: Option<i32>,
    target_dead: bool,
    moving: bool,
}

impl BotSession {
    pub fn new(profile: Profile, transport: Box<dyn Transport>, clock: Arc<dyn Clock>) -> Self {
        let pacing = Pacing::new(Duration::from_millis(profile.pacing_ms));
        let brain = Brain::new(profile.options.clone());
        Self {
            profile,
            transport,
            clock,
            pacing,
            world: World::default(),
            state: BotState::Connecting,
            brain,
            names: Box::new(IdNames),
            sweepable: HashSet::new(),
            spoiled: HashSet::new(),
            fresh_chat: Vec::new(),
            english: false,
            dead: false,
            manual_pause: false,
            automation: true,
            chat: ChatLog::default(),
            packet_log: crate::packets::PacketLog::default(),
            servers: Vec::new(),
            characters: Vec::new(),
            templates: Vec::new(),
            commands_sent: 0,
            deaths: 0,
            entered_at: None,
            script: None,
            script_events: Vec::new(),
            watch: Watch::default(),
            stats: Default::default(),
            world_ticked_at: None,
            ping: Ping::default(),
            own_casts: Vec::new(),
        }
    }

    pub fn load_script(&mut self, source: &str) -> Result<(), ScriptError> {
        let compiled = killer_bot_script::build(source)?;
        self.script_events.clear();
        self.script = Some(ScriptRun { vm: Vm::new(compiled), queue: Default::default(), status: ScriptStatus::Running });
        Ok(())
    }

    pub fn stop_script(&mut self) {
        self.script = None;
        self.script_events.clear();
    }

    pub fn script_variables(&self) -> Vec<(String, Value)> {
        self.script.as_ref().map(|run| run.vm.variables()).unwrap_or_default()
    }

    pub fn script_paused(&self) -> bool {
        self.script.as_ref().is_some_and(|run| run.vm.paused())
    }

    pub fn pause_script(&mut self, paused: bool) {
        if let Some(run) = self.script.as_mut() {
            run.vm.set_paused(paused);
        }
    }

    pub fn script_tasks(&self) -> Vec<killer_bot_script::TaskInfo> {
        self.script.as_ref().map(|run| run.vm.tasks()).unwrap_or_default()
    }

    pub fn script_now(&self) -> Duration {
        self.clock.now()
    }

    pub fn script_status(&self) -> ScriptStatus {
        self.script.as_ref().map(|run| run.status.clone()).unwrap_or_default()
    }

    fn watch_world(&mut self) {
        let me = self.world.me.as_ref();
        let in_combat = me.is_some_and(|me| me.in_combat);
        let moving = me.is_some_and(|me| me.dest.is_some());
        let target = self.world.targeted().map(|target| (target.object_id, target.dead, target.name.clone()));
        if in_combat != self.watch.in_combat {
            self.script_events.push((if in_combat { EventKind::EnterCombat } else { EventKind::LeaveCombat }, Value::Bool(true)));
        }
        if self.watch.moving && !moving {
            self.script_events.push((EventKind::Arrived, Value::Bool(true)));
        }
        match (self.watch.target, &target) {
            (None, Some((_, _, name))) => self.script_events.push((EventKind::TargetSelected, Value::Text(name.clone()))),
            (Some(_), None) => self.script_events.push((EventKind::TargetLost, Value::Bool(true))),
            (Some(old), Some((id, _, name))) if old != *id => self.script_events.push((EventKind::TargetSelected, Value::Text(name.clone()))),
            _ => {}
        }
        let dead = target.as_ref().is_some_and(|(_, dead, _)| *dead);
        if dead && !self.watch.target_dead {
            let name = target.as_ref().map(|(_, _, name)| name.clone()).unwrap_or_default();
            self.stats.last_kill = name.clone();
            self.script_events.push((EventKind::TargetDead, Value::Text(name)));
        }
        self.watch = Watch { in_combat, target: target.as_ref().map(|(id, _, _)| *id), target_dead: dead, moving };
    }

    fn run_script(&mut self, chat: &[ChatLine], out: &mut Vec<BotEvent>) {
        if self.state != BotState::InWorld {
            return;
        }
        self.watch_world();
        for line in chat {
            if self.world.me.as_ref().is_some_and(|me| me.name == line.sender) {
                continue;
            }
            self.script_events.push((EventKind::Chat, Value::Text(format!("{}: {}", line.sender, line.text))));
        }
        let events = std::mem::take(&mut self.script_events);
        let Some(run) = self.script.as_mut() else { return };
        let now = self.clock.now();
        for (event, payload) in events {
            run.vm.fire(event, payload);
        }
        let mut log = Vec::new();
        let status = {
            let env = Env { world: &self.world, names: self.names.as_ref(), now, sweepable: &self.sweepable, spoiled: &self.spoiled };
            let mut host = SessionHost { env: &env, out: &mut run.queue, log: &mut log };
            run.vm.step(&mut host, now)
        };
        run.status = ScriptStatus::of(&status);
        for line in log {
            out.push(BotEvent::Log(format!("script: {line}")));
        }
        if let Status::Failed(error) = &status {
            out.push(BotEvent::Log(format!("script parou: {error}")));
        }
        if !run.queue.is_empty() && self.pacing.allow(now)
            && let Some(command) = run.queue.pop_front()
        {
            out.push(BotEvent::Log(format!("script: › {}", crate::script_host::describe(&command))));
            self.send(command);
        }
    }

    pub fn set_names(&mut self, names: Box<dyn Names>) {
        self.names = names;
    }

    pub fn automation_on(&self) -> bool {
        self.automation
    }

    pub fn is_paused(&self) -> bool {
        self.manual_pause
    }

    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    pub fn update_options(&mut self, change: impl FnOnce(&mut crate::options::BotOptions)) {
        change(&mut self.profile.options);
        self.brain.set_options(self.profile.options.clone());
    }

    pub(crate) fn blacklist(&mut self, object_id: i32) {
        let now = self.clock.now();
        self.brain.blacklist_target(object_id, now);
    }

    pub fn set_profile(&mut self, profile: Profile) {
        self.pacing = Pacing::new(Duration::from_millis(profile.pacing_ms));
        self.brain.set_options(profile.options.clone());
        self.profile = profile;
    }

    pub fn state(&self) -> BotState {
        self.state
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn snapshot(&self) -> crate::snapshot::Snapshot {
        let mut snapshot = crate::snapshot::Snapshot::with_login(self.state, &self.world, &self.servers, &self.characters);
        snapshot.gains = self.stats.gains(self.clock.now());
        snapshot.loot = self.stats.loot();
        snapshot.bad_mobs = self.brain.blacklisted();
        snapshot
    }

    pub fn take_own_casts(&mut self) -> Vec<(i32, i32)> {
        std::mem::take(&mut self.own_casts)
    }

    pub fn ping_ms(&self) -> Option<u32> {
        self.ping.millis()
    }

    pub fn chat_total(&self) -> u64 {
        self.chat.pushed()
    }

    pub fn chat(&self) -> crate::chat::ChatBoxes {
        crate::chat::ChatBoxes::from_log(&self.chat)
    }

    pub fn clear_stats(&mut self) {
        let now = self.clock.now();
        self.stats.clear(now);
    }

    pub fn close_store(&mut self) {
        self.world.stores.sell = None;
        self.world.stores.buy = None;
        self.world.stores.manage_sell = None;
        self.world.stores.manage_buy = None;
    }

    pub fn close_shop(&mut self) {
        self.world.windows.buy = None;
        self.world.windows.sell = None;
    }

    pub fn close_warehouse(&mut self) {
        self.world.windows.warehouse = None;
    }

    pub fn close_learn(&mut self) {
        self.world.windows.learnable = None;
        self.world.windows.skill_info = None;
    }

    pub fn close_multisell(&mut self) {
        self.world.windows.multisell = None;
    }

    pub fn close_npc_dialog(&mut self) {
        self.world.npc_html = None;
    }

    pub fn npc_dialog(&self) -> Option<crate::npc::NpcDialog> {
        self.world.npc_html.as_ref().map(|message| crate::npc::parse_npc_html(message.npc_object_id, &message.html))
    }

    pub(crate) fn take_question(&mut self, index: usize, accept: bool) -> Option<NetCommand> {
        use killer_world::state::Dialog;
        let dialog = self.world.dialogs.pending.get(index)?;
        let command = match dialog {
            Dialog::JoinParty(_) => NetCommand::AnswerJoinParty { accept },
            Dialog::JoinPledge(_) => NetCommand::AnswerJoinPledge { accept },
            Dialog::JoinAlliance(_) => NetCommand::AnswerJoinAlly { accept },
            Dialog::TradeRequest { .. } => NetCommand::AnswerTrade { accept },
            Dialog::FriendRequest { .. } => NetCommand::AnswerFriendInvite { accept },
            Dialog::Confirm(message) => NetCommand::DlgAnswer { message_id: message.id, answer: i32::from(accept) },
            _ => return None,
        };
        self.world.dialogs.pending.remove(index);
        Some(command)
    }

    pub fn packet_log(&self) -> &crate::packets::PacketLog {
        &self.packet_log
    }

    pub fn packet_log_mut(&mut self) -> &mut crate::packets::PacketLog {
        &mut self.packet_log
    }

    pub fn send_raw(&mut self, body: Vec<u8>) -> bool {
        if self.state == BotState::Disconnected || body.is_empty() {
            return false;
        }
        self.send(NetCommand::Raw(body));
        true
    }

    pub fn inject_to_client(&mut self, body: &[u8]) -> Result<(), String> {
        match killer_world::session::decode_world(body) {
            Some(Ok(event)) => {
                self.on_session(SessionEvent::World(event), &mut Vec::new());
                Ok(())
            }
            Some(Err(error)) => Err(error.to_string()),
            None => Err("pacote desconhecido para o mundo".to_owned()),
        }
    }

    pub fn servers(&self) -> &[l2_net::login::GameServer] {
        &self.servers
    }

    pub fn char_templates(&self) -> &[l2_net::game::CharTemplate] {
        &self.templates
    }

    pub fn characters(&self) -> &[l2_net::game::CharSelectEntry] {
        &self.characters
    }

    pub fn bot_stats(&self) -> BotStats {
        BotStats {
            commands_sent: self.commands_sent,
            deaths: self.deaths,
            online_seconds: self.entered_at.map_or(0, |at| self.clock.now().saturating_sub(at).as_secs()),
        }
    }

    fn flush_world_requests(&mut self) {
        for request in self.world.take_requests() {
            self.send(request);
        }
    }

    pub(crate) fn send(&mut self, command: NetCommand) {
        self.commands_sent += 1;
        self.transport.send(command);
    }

    fn set_state(&mut self, state: BotState, out: &mut Vec<BotEvent>) {
        if self.state != state {
            self.state = state;
            out.push(BotEvent::State(state));
        }
    }

    pub fn tick(&mut self) -> Vec<BotEvent> {
        let mut out = Vec::new();
        let now = self.clock.now();
        for record in self.transport.packets() {
            self.packet_log.add(record, now);
        }
        for event in self.transport.poll() {
            self.on_event(event, &mut out);
        }
        let elapsed = self.world_ticked_at.replace(now).map_or(0.0, |before| now.saturating_sub(before).as_secs_f32());
        self.world.tick(elapsed.min(1.0));
        if self.state == BotState::InWorld {
            if self.ping.due(now) {
                self.send(NetCommand::RequestQuestList);
            }
            self.stats.observe(now, &self.world);
            let chat = self.fresh_chat.clone();
            self.act(&mut out);
            self.run_script(&chat, &mut out);
        }
        out
    }

    fn on_event(&mut self, event: NetEvent, out: &mut Vec<BotEvent>) {
        if self.state == BotState::Disconnected {
            return;
        }
        match event {
            NetEvent::Connecting => {}
            NetEvent::Notice(_) => {
                if self.profile.accept_notice {
                    self.send(NetCommand::NoticeAnswer(true));
                } else {
                    out.push(BotEvent::Log("o servidor pediu aceite dos termos e o perfil não aceita".into()));
                }
            }
            NetEvent::CardCheck { .. } => out.push(BotEvent::Log("o login server pediu o cartão de segurança; o bot não o digita".into())),
            NetEvent::ServerList(servers) => {
                let chosen = match self.profile.server_id {
                    Some(id) => servers.iter().find(|server| server.id == id).map(|server| server.id),
                    None if servers.len() == 1 => servers.first().map(|server| server.id),
                    None => None,
                };
                match chosen {
                    Some(id) => self.send(NetCommand::ChooseServer(id)),
                    None => {
                        out.push(BotEvent::Log("escolha o servidor (ou defina server_id no perfil)".into()));
                        self.set_state(BotState::ChoosingServer, out);
                    }
                }
                self.servers = servers;
            }
            NetEvent::LoginOk(_) => {
                if self.state != BotState::ChoosingCharacter {
                    self.set_state(BotState::EnteringWorld, out);
                }
            }
            NetEvent::LoginRefused { text, .. } | NetEvent::LoginBlocked { text, .. } => {
                out.push(BotEvent::Refused(text));
                self.set_state(BotState::Disconnected, out);
            }
            NetEvent::Error(text) => {
                out.push(BotEvent::Log(text));
                self.set_state(BotState::Disconnected, out);
            }
            NetEvent::Closed => self.set_state(BotState::Disconnected, out),
            NetEvent::Session(event) => self.on_session(event, out),
        }
    }

    fn on_session(&mut self, event: SessionEvent, out: &mut Vec<BotEvent>) {
        match &event {
            SessionEvent::Characters(list) => {
                self.characters = list.clone();
                if self.profile.character.trim().is_empty() {
                    out.push(BotEvent::Log("escolha o personagem (ou defina character no perfil)".into()));
                    self.set_state(BotState::ChoosingCharacter, out);
                } else {
                    match list.iter().position(|entry| entry.name.eq_ignore_ascii_case(&self.profile.character)) {
                        Some(slot) => self.send(NetCommand::Select(slot as i32)),
                        None => {
                            out.push(BotEvent::Refused(format!("personagem {} não existe na conta", self.profile.character)));
                            self.set_state(BotState::Disconnected, out);
                        }
                    }
                }
                return;
            }
            SessionEvent::CharTemplates(list) => {
                self.templates = list.templates.clone();
                return;
            }
            SessionEvent::CharCreated => {
                out.push(BotEvent::Log("personagem criado".into()));
                return;
            }
            SessionEvent::CharCreateFailed(reason) => {
                out.push(BotEvent::Log(format!("criação recusada: {}", create_failure(*reason))));
                return;
            }
            SessionEvent::UserInfo(_) => {
                let first = self.world.me.is_none();
                self.world.apply(&event);
                if first {
                    self.send(NetCommand::Appearing);
                }
                if self.entered_at.is_none() {
                    self.entered_at = Some(self.clock.now());
                }
                self.set_state(BotState::InWorld, out);
                return;
            }
            SessionEvent::ServerClosed => {
                self.set_state(BotState::Disconnected, out);
                return;
            }
            SessionEvent::Refused(text) => {
                out.push(BotEvent::Refused(text.clone()));
                self.set_state(BotState::Disconnected, out);
                return;
            }
            _ => {}
        }
        if let SessionEvent::World(world_event) = &event {
            self.chat.apply(world_event);
            if let Some(line) = ChatLine::from_event(world_event) {
                self.fresh_chat.push(line);
            }
            match world_event {
                WorldEvent::Attack(attack) => {
                    let me = self.world.me.as_ref().map(|me| me.object_id);
                    if me.is_some_and(|me| attack.hits.iter().any(|hit| hit.target_id == me))
                        && let Some(attacker) = self.world.entity(attack.attacker_id)
                        && matches!(attacker.kind, killer_world::EntityKind::Player { .. })
                    {
                        self.script_events.push((EventKind::PlayerAttack, Value::Text(attacker.name.clone())));
                    }
                }
                WorldEvent::Die(die) if self.world.me.as_ref().is_none_or(|me| me.object_id != die.object_id) && die.sweepable => {
                    self.sweepable.insert(die.object_id);
                }
                WorldEvent::SystemMessage(message) if message.id == 612 => {
                    if let Some(target) = self.world.target {
                        self.spoiled.insert(target);
                    }
                }
                WorldEvent::DeleteObject(gone) => {
                    self.sweepable.remove(&gone.object_id);
                    self.spoiled.remove(&gone.object_id);
                }
                _ => {}
            }
        }
        if let SessionEvent::World(WorldEvent::MagicSkillUse(cast)) = &event
            && self.world.is_me(cast.caster_id)
        {
            self.own_casts.push((cast.skill_id, cast.hit_time));
        }
        if matches!(&event, SessionEvent::World(WorldEvent::QuestList(_))) {
            self.ping.answered(self.clock.now());
        }
        let teleported = matches!(&event, SessionEvent::World(WorldEvent::TeleportToLocation(t)) if self.world.me.as_ref().is_some_and(|me| me.object_id == t.target_id));
        self.world.apply(&event);
        self.flush_world_requests();
        if teleported && let Some(me) = &self.world.me {
            let command = NetCommand::Validate { position: me.position.to_array().map(|value| value as i32), heading: me.heading };
            self.sweepable.clear();
            self.spoiled.clear();
            self.send(command);
            self.send(NetCommand::Appearing);
        }
        let dead_now = self.world.death.is_some() || self.world.me.as_ref().is_some_and(|me| me.dead);
        if dead_now && !self.dead {
            self.deaths += 1;
            out.push(BotEvent::Died);
            self.script_events.push((EventKind::Death, Value::Bool(true)));
        } else if !dead_now && self.dead {
            out.push(BotEvent::Revived);
            self.script_events.push((EventKind::Revive, Value::Bool(true)));
        }
        self.dead = dead_now;
    }

    fn act(&mut self, out: &mut Vec<BotEvent>) {
        let fresh = std::mem::take(&mut self.fresh_chat);
        if self.dead && self.world.death.is_none() {
            return;
        }
        let suspended = self.manual_pause || !self.automation;
        if suspended {
            return;
        }
        let now = self.clock.now();
        let decision = {
            let env = Env { world: &self.world, names: self.names.as_ref(), now, sweepable: &self.sweepable, spoiled: &self.spoiled };
            self.brain.decide(&env, &fresh)
        };
        for event in decision.events {
            match event {
                BrainEvent::StopBotting => {
                    self.automation = false;
                    out.push(BotEvent::BottingStopped);
                }
                BrainEvent::Alert(alert) => out.push(BotEvent::Alert(alert.text(self.english))),
                BrainEvent::LoggedOut(reason) => out.push(BotEvent::Log(format!("saindo do jogo: {reason}"))),
            }
        }
        if let Some(command) = decision.command
            && self.pacing.allow(now)
        {
            self.send(command);
        }
    }
}

const PING_FIRST: Duration = Duration::from_secs(5);
const PING_EVERY: Duration = Duration::from_secs(15);
const PING_GIVE_UP: Duration = Duration::from_secs(10);
const PING_WINDOW: usize = 5;

#[derive(Debug)]
struct Ping {
    sent: Option<Duration>,
    next: Duration,
    samples: std::collections::VecDeque<u32>,
}

impl Default for Ping {
    fn default() -> Self {
        Self { sent: None, next: PING_FIRST, samples: std::collections::VecDeque::new() }
    }
}

impl Ping {
    fn due(&mut self, now: Duration) -> bool {
        if self.sent.is_some_and(|sent| now.saturating_sub(sent) > PING_GIVE_UP) {
            self.sent = None;
        }
        if self.sent.is_some() || now < self.next {
            return false;
        }
        self.sent = Some(now);
        self.next = now + PING_EVERY;
        true
    }

    fn answered(&mut self, now: Duration) {
        let Some(sent) = self.sent.take() else { return };
        self.samples.push_back(now.saturating_sub(sent).as_millis().min(u128::from(u32::MAX)) as u32);
        while self.samples.len() > PING_WINDOW {
            self.samples.pop_front();
        }
    }

    fn millis(&self) -> Option<u32> {
        self.samples.iter().copied().min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::ManualClock;
    use crate::testkit::{Vitals, char_entry, character, ground_item, npc_info, user_info};
    use crate::transport::FakeTransport;
    use killer_world::SessionEvent;
    use std::sync::Mutex;

    #[derive(Clone, Default)]
    struct Shared(Arc<Mutex<FakeTransport>>);

    impl Transport for Shared {
        fn send(&mut self, command: NetCommand) {
            self.0.lock().unwrap().send(command);
        }

        fn poll(&mut self) -> Vec<NetEvent> {
            self.0.lock().unwrap().poll()
        }
    }

    impl Shared {
        fn push(&self, event: NetEvent) {
            self.0.lock().unwrap().push(event);
        }

        fn sent(&self) -> Vec<NetCommand> {
            self.0.lock().unwrap().sent.clone()
        }
    }

    const PROFILE: &str = "account = \"a\"\ncharacter = \"Wender\"\n[server]\nhost = \"h\"\nport = 1\n[options.attack]\nactive_target = true\nactive_attack = true\n";

    fn session(text: &str) -> (BotSession, Shared, ManualClock) {
        let shared = Shared::default();
        let clock = ManualClock::new();
        let session = BotSession::new(Profile::parse(text).unwrap(), Box::new(shared.clone()), Arc::new(clock.clone()));
        (session, shared, clock)
    }

    fn enter(shared: &Shared, session: &mut BotSession) {
        shared.push(NetEvent::Session(SessionEvent::Characters(vec![char_entry("Outro"), char_entry("Wender")])));
        shared.push(NetEvent::Session(user_info(1, "Wender", &Vitals::full(), [0, 0, 0])));
        session.tick();
    }

    #[test]
    fn choosing_the_character_by_name_sends_select_with_its_slot() {
        let (mut session, shared, _) = session(PROFILE);
        shared.push(NetEvent::Session(SessionEvent::Characters(vec![char_entry("Outro"), char_entry("wender")])));
        session.tick();
        assert_eq!(shared.sent(), vec![NetCommand::Select(1)]);
    }

    #[test]
    fn without_a_character_in_the_profile_it_waits_for_the_choice() {
        let text = "account = \"a\"\n[server]\nhost = \"h\"\nport = 1\n";
        let (mut session, shared, _) = session(text);
        shared.push(NetEvent::Session(SessionEvent::Characters(vec![char_entry("Um"), char_entry("Dois")])));
        session.tick();
        assert_eq!(session.state(), BotState::ChoosingCharacter);
        assert!(shared.sent().is_empty());
        assert_eq!(session.characters().len(), 2);
        assert!(session.command(crate::command::UserCommand::ChooseCharacter(1)).is_ok());
        assert_eq!(shared.sent(), vec![NetCommand::Select(1)]);
        assert!(session.command(crate::command::UserCommand::ChooseCharacter(9)).is_err());
    }

    #[test]
    fn several_servers_wait_for_the_choice_and_one_server_is_picked_alone() {
        let text = "account = \"a\"\n[server]\nhost = \"h\"\nport = 1\n";
        let (mut session, shared, _) = session(text);
        let server = |id: u8| l2_net::login::GameServer { id, ip: [1, 2, 3, 4], port: 7777, players: 3, max_players: 100, online: true, extra: Default::default() };
        shared.push(NetEvent::ServerList(vec![server(1), server(2)]));
        session.tick();
        assert_eq!(session.state(), BotState::ChoosingServer);
        assert!(shared.sent().is_empty());
        assert!(session.command(crate::command::UserCommand::ChooseServer(2)).is_ok());
        assert_eq!(shared.sent(), vec![NetCommand::ChooseServer(2)]);
        let (mut single, shared, _) = self::session(text);
        shared.push(NetEvent::ServerList(vec![server(5)]));
        single.tick();
        assert_eq!(shared.sent(), vec![NetCommand::ChooseServer(5)]);
        let with_id = text.replace("account = \"a\"", "account = \"a\"\nserver_id = 2");
        let (mut chosen, shared, _) = self::session(&with_id);
        shared.push(NetEvent::ServerList(vec![server(1), server(2)]));
        chosen.tick();
        assert_eq!(shared.sent(), vec![NetCommand::ChooseServer(2)]);
    }

    #[test]
    fn unknown_character_emits_refused_and_never_selects() {
        let (mut session, shared, _) = session(PROFILE);
        shared.push(NetEvent::Session(SessionEvent::Characters(vec![char_entry("Outro")])));
        let events = session.tick();
        assert!(events.iter().any(|event| matches!(event, BotEvent::Refused(_))));
        assert!(shared.sent().is_empty());
        assert_eq!(session.state(), BotState::Disconnected);
    }

    #[test]
    fn user_info_moves_to_in_world_and_announces_appearing_once() {
        let (mut session, shared, _) = session(PROFILE);
        enter(&shared, &mut session);
        assert_eq!(session.state(), BotState::InWorld);
        shared.push(NetEvent::Session(user_info(1, "Wender", &Vitals::full(), [5, 5, 5])));
        session.tick();
        assert_eq!(shared.sent().iter().filter(|c| **c == NetCommand::Appearing).count(), 1);
    }

    #[test]
    fn an_unknown_clan_on_a_nearby_player_is_asked_for_once() {
        let (mut session, shared, _) = session(PROFILE);
        enter(&shared, &mut session);
        for _ in 0..2 {
            let mut event = character(50, "Gatoso", [10, 0, 0]);
            if let SessionEvent::World(WorldEvent::CharInfo(info)) = &mut event {
                info.clan_id = 45269;
            }
            shared.push(NetEvent::Session(event));
            session.tick();
        }
        let asked = shared.sent().iter().filter(|command| **command == NetCommand::RequestPledgeInfo { pledge_id: 45269 }).count();
        assert_eq!(asked, 1);
    }

    #[test]
    fn the_ping_is_the_round_trip_of_a_probe_and_is_asked_again_only_after_the_interval() {
        let (mut session, shared, clock) = session(PROFILE);
        enter(&shared, &mut session);
        clock.advance(Duration::from_secs(5));
        session.tick();
        let probes = |shared: &Shared| shared.sent().iter().filter(|command| **command == NetCommand::RequestQuestList).count();
        assert_eq!(probes(&shared), 1);
        assert_eq!(session.ping_ms(), None);
        clock.advance(Duration::from_millis(80));
        shared.push(NetEvent::Session(SessionEvent::World(WorldEvent::QuestList(l2_net::game::QuestList { quests: Vec::new(), extras: Vec::new() }))));
        session.tick();
        assert_eq!(session.ping_ms(), Some(80));
        clock.advance(Duration::from_secs(5));
        session.tick();
        assert_eq!(probes(&shared), 1);
        clock.advance(Duration::from_secs(11));
        session.tick();
        assert_eq!(probes(&shared), 2);
    }

    #[test]
    fn closing_the_connection_goes_to_disconnected_and_stops_sending() {
        let (mut session, shared, clock) = session(PROFILE);
        enter(&shared, &mut session);
        shared.push(NetEvent::Session(npc_info(9, 1, "Wolf", true, [100, 0, 0])));
        shared.push(NetEvent::Closed);
        let events = session.tick();
        assert_eq!(events.iter().filter(|event| **event == BotEvent::State(BotState::Disconnected)).count(), 1);
        let sent = shared.sent().len();
        for _ in 0..5 {
            clock.advance(Duration::from_secs(1));
            assert!(session.tick().is_empty());
        }
        assert_eq!(shared.sent().len(), sent);
    }

    #[test]
    fn in_world_it_selects_then_attacks_the_monster() {
        let (mut session, shared, clock) = session(PROFILE);
        enter(&shared, &mut session);
        shared.push(NetEvent::Session(npc_info(9, 1, "Wolf", true, [100, 0, 0])));
        clock.advance(Duration::from_millis(200));
        session.tick();
        assert_eq!(shared.sent().last(), Some(&NetCommand::Action { object_id: 9, origin: [0, 0, 0], shift: false }));
    }

    #[test]
    fn death_stops_commands_and_emits_died_once_and_revive_resumes() {
        let (mut session, shared, clock) = session(PROFILE);
        enter(&shared, &mut session);
        shared.push(NetEvent::Session(npc_info(9, 1, "Wolf", true, [100, 0, 0])));
        shared.push(NetEvent::Session(SessionEvent::World(WorldEvent::Die(l2_net::game::Die {
            object_id: 1,
            to_village: true,
            to_hideout: false,
            to_castle: false,
            to_siege_hq: false,
            sweepable: false,
            to_fixed_spot: false,
        }))));
        let events = session.tick();
        assert_eq!(events.iter().filter(|event| **event == BotEvent::Died).count(), 1);
        let sent = shared.sent().len();
        clock.advance(Duration::from_secs(2));
        assert!(session.tick().is_empty());
        assert_eq!(shared.sent().len(), sent);
        shared.push(NetEvent::Session(SessionEvent::World(WorldEvent::Revive(l2_net::game::Revive { object_id: 1 }))));
        let events = session.tick();
        assert_eq!(events.iter().filter(|event| **event == BotEvent::Revived).count(), 1);
        clock.advance(Duration::from_secs(2));
        session.tick();
        assert!(shared.sent().len() > sent);
    }

    #[test]
    fn loot_is_picked_when_nothing_is_targeted() {
        let text = format!("{PROFILE}[options.pickup]\npickup = true\n");
        let (mut session, shared, clock) = session(&text);
        enter(&shared, &mut session);
        shared.push(NetEvent::Session(ground_item(30, 57, [60, 0, 0])));
        clock.advance(Duration::from_millis(200));
        session.tick();
        assert_eq!(shared.sent().last(), Some(&NetCommand::Action { object_id: 30, origin: [0, 0, 0], shift: false }));
    }

    #[test]
    fn a_script_command_is_not_starved_by_a_busy_bot() {
        let (mut session, shared, clock) = session(PROFILE);
        enter(&shared, &mut session);
        shared.push(NetEvent::Session(npc_info(9, 20001, "Wolf", true, [50, 0, 0])));
        session.tick();
        assert!(session.command(crate::command::UserCommand::SetAutomation(true)).is_ok());
        session.load_script("sit\nstand\n").unwrap();
        for _ in 0..40 {
            clock.advance(Duration::from_millis(60));
            session.tick();
        }
        let sent = shared.sent();
        assert!(sent.iter().any(|command| matches!(command, NetCommand::ChangeWaitType { standing: false })), "{sent:?}");
        assert!(sent.iter().any(|command| matches!(command, NetCommand::ChangeWaitType { standing: true })), "{sent:?}");
    }
}
