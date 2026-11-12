use killer_bot_core::clock::ManualClock;
use killer_bot_core::profile::Profile;
use killer_bot_core::script_host::ScriptStatus;
use killer_bot_core::session::{BotEvent, BotSession};
use killer_bot_core::testkit::{Vitals, char_entry, npc_info, user_info};
use killer_bot_core::transport::{FakeTransport, Transport};
use killer_world::{NetCommand, NetEvent, SessionEvent, WorldEvent};
use l2_net::game::{Die, MyTargetSelected, SkillEntry, SkillList};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Default)]
struct Wire(Arc<Mutex<FakeTransport>>);

impl Transport for Wire {
    fn send(&mut self, command: NetCommand) {
        self.0.lock().unwrap().send(command);
    }

    fn poll(&mut self) -> Vec<NetEvent> {
        self.0.lock().unwrap().poll()
    }
}

impl Wire {
    fn push(&self, event: SessionEvent) {
        self.0.lock().unwrap().push(NetEvent::Session(event));
    }

    fn sent(&self) -> Vec<NetCommand> {
        self.0.lock().unwrap().sent.clone()
    }
}

const PROFILE: &str = "account = \"conta\"\ncharacter = \"Wender\"\npacing_ms = 400\n[server]\nhost = \"h\"\nport = 1\n";

struct Rig {
    bot: BotSession,
    wire: Wire,
    clock: ManualClock,
}

impl Rig {
    fn new() -> Rig {
        let wire = Wire::default();
        let clock = ManualClock::new();
        let mut bot = BotSession::new(Profile::parse(PROFILE).unwrap(), Box::new(wire.clone()), Arc::new(clock.clone()));
        wire.push(SessionEvent::Characters(vec![char_entry("Wender")]));
        wire.push(user_info(1, "Wender", &Vitals::full(), [0, 0, 0]));
        wire.push(SessionEvent::World(WorldEvent::SkillList(SkillList { skills: vec![SkillEntry { passive: false, level: 1, id: 56 }] })));
        wire.push(npc_info(9, 20001, "Wolf", true, [200, 0, 0]));
        bot.tick();
        wire.0.lock().unwrap().sent.clear();
        Rig { bot, wire, clock }
    }

    fn advance(&mut self, millis: u64) -> Vec<BotEvent> {
        self.clock.advance(Duration::from_millis(millis));
        self.bot.tick()
    }
}

#[test]
fn script_attacks_and_waits_through_the_session() {
    let mut rig = Rig::new();
    rig.bot.load_script("target_nearest\nwait 1000\nuse_skill 56\nwait 100000\n").unwrap();
    rig.advance(500);
    assert_eq!(rig.wire.sent(), vec![NetCommand::Action { object_id: 9, origin: [0, 0, 0], shift: false }]);
    assert_eq!(rig.bot.script_status(), ScriptStatus::Waiting);
    rig.advance(500);
    assert_eq!(rig.wire.sent().len(), 1);
    rig.advance(600);
    assert_eq!(rig.wire.sent()[1], NetCommand::UseSkill { skill_id: 56, ctrl: false, shift: false });
}

#[test]
fn script_commands_respect_the_pacing_gap() {
    let mut rig = Rig::new();
    rig.bot.load_script("sit\nstand\nsit\nwait 100000\n").unwrap();
    rig.advance(1000);
    assert_eq!(rig.wire.sent().len(), 1);
    rig.advance(100);
    assert_eq!(rig.wire.sent().len(), 1);
    rig.advance(400);
    assert_eq!(rig.wire.sent().len(), 2);
    rig.advance(400);
    assert_eq!(rig.wire.sent().len(), 3);
}

#[test]
fn replacing_a_script_drops_pending_tasks_and_commands() {
    let mut rig = Rig::new();
    rig.bot.load_script("sit\nstand\nstand\nwait 100000\n").unwrap();
    rig.advance(1000);
    rig.bot.load_script("wait 100000\n").unwrap();
    rig.advance(1000);
    rig.advance(1000);
    assert_eq!(rig.wire.sent().len(), 1);
    rig.bot.stop_script();
    assert_eq!(rig.bot.script_status(), ScriptStatus::None);
}

#[test]
fn death_event_runs_the_on_death_handler() {
    let mut rig = Rig::new();
    rig.bot.load_script("on death\nsay \"all\" \"morri\"\nend\nwait 100000\n").unwrap();
    rig.advance(1000);
    rig.wire.push(SessionEvent::World(WorldEvent::Die(Die { object_id: 1, to_village: true, to_hideout: false, to_castle: false, to_siege_hq: false, sweepable: false, to_fixed_spot: false })));
    rig.advance(1000);
    assert!(rig.wire.sent().iter().any(|command| matches!(command, NetCommand::Say { text, .. } if text == "morri")));
}

#[test]
fn target_events_fire_handlers() {
    let mut rig = Rig::new();
    rig.bot.load_script("on target_selected\nprint target_name\nend\nwait 100000\n").unwrap();
    rig.advance(500);
    rig.wire.push(SessionEvent::World(WorldEvent::MyTargetSelected(MyTargetSelected { object_id: 9, color: 0 })));
    let events = rig.advance(500);
    assert!(events.contains(&BotEvent::Log("script: Wolf".into())));
}

#[test]
fn a_failing_script_stops_but_the_bot_keeps_ticking() {
    let mut rig = Rig::new();
    rig.bot.load_script("sit\nset a = 1 / 0\n").unwrap();
    let events = rig.advance(500);
    assert!(matches!(rig.bot.script_status(), ScriptStatus::Failed(error) if error.line == 2));
    assert!(events.iter().any(|event| matches!(event, BotEvent::Log(text) if text.starts_with("script parou: linha 2"))));
    rig.advance(500);
    assert_eq!(rig.bot.state(), killer_bot_core::session::BotState::InWorld);
}

#[test]
fn queries_read_the_world() {
    let mut rig = Rig::new();
    rig.bot.load_script("set a = hp\nset b = nearby_count 500\nset c = has_target\nset d = skill_level 56\nset e = distance_to 300 400 0\nprint a b c d e\n").unwrap();
    let events = rig.advance(500);
    assert!(events.contains(&BotEvent::Log("script: 100 1 false 1 500".into())), "{events:?}");
    assert_eq!(rig.bot.script_status(), ScriptStatus::Finished);
}

#[test]
fn script_syntax_errors_are_reported_when_loading() {
    let mut rig = Rig::new();
    let error = rig.bot.load_script("if hp < 3\nsit\n").unwrap_err();
    assert_eq!(error.line, 1);
    assert_eq!(rig.bot.script_status(), ScriptStatus::None);
}

#[test]
fn unknown_commands_and_bad_arguments_fail_with_the_line() {
    let mut rig = Rig::new();
    rig.bot.load_script("sit\nvoar 1\n").unwrap();
    rig.advance(500);
    assert!(matches!(rig.bot.script_status(), ScriptStatus::Failed(error) if error.line == 2 && error.message.contains("voar")));
    rig.bot.load_script("use_item 999\n").unwrap();
    rig.advance(500);
    assert!(matches!(rig.bot.script_status(), ScriptStatus::Failed(error) if error.message.contains("999")));
    rig.bot.load_script("say \"gritar\" \"oi\"\n").unwrap();
    rig.advance(500);
    assert!(matches!(rig.bot.script_status(), ScriptStatus::Failed(error) if error.message.contains("canal")));
}
