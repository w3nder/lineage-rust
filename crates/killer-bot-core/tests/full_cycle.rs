use killer_bot_core::clock::ManualClock;
use killer_bot_core::profile::Profile;
use killer_bot_core::session::{BotEvent, BotSession, BotState};
use killer_bot_core::testkit::{Vitals, char_entry, ground_item, npc_info, user_info};
use killer_bot_core::transport::{FakeTransport, Transport};
use killer_world::{NetCommand, NetEvent, SessionEvent, WorldEvent};
use l2_net::game::{DeleteObject, Die, MyTargetSelected, Revive, ShortCutInit, Shortcut, ShortcutKind, SkillEntry, SkillList};
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
        self.0.lock().unwrap().sent.iter().filter(|c| **c != NetCommand::RequestSkillCoolTime).cloned().collect()
    }

    fn cool_time_requests(&self) -> usize {
        self.0.lock().unwrap().sent.iter().filter(|c| **c == NetCommand::RequestSkillCoolTime).count()
    }
}

const PROFILE: &str = r#"
account = "conta"
character = "Wender"

[server]
host = "server.l2killer.com"
port = 2106

[options.attack]
active_target = true
active_attack = true

[options.pickup]
pickup = true

[[options.combat]]
page = 1
slot = 1
trait_ = "always"
delay_ms = 1000
mp_above = 10
"#;

fn die(object_id: i32) -> SessionEvent {
    SessionEvent::World(WorldEvent::Die(Die { object_id, to_village: true, to_hideout: false, to_castle: false, to_siege_hq: false, sweepable: false, to_fixed_spot: false }))
}

#[test]
fn login_attack_loot_die_revive() {
    let wire = Wire::default();
    let clock = ManualClock::new();
    let mut bot = BotSession::new(Profile::parse(PROFILE).unwrap(), Box::new(wire.clone()), Arc::new(clock.clone()));
    let step = |bot: &mut BotSession| {
        clock.advance(Duration::from_millis(250));
        bot.tick()
    };

    wire.push(SessionEvent::Characters(vec![char_entry("Outro"), char_entry("Wender")]));
    wire.push(user_info(1, "Wender", &Vitals::full(), [0, 0, 0]));
    wire.push(SessionEvent::World(WorldEvent::SkillList(SkillList { skills: vec![SkillEntry { passive: false, level: 1, id: 56 }] })));
    wire.push(SessionEvent::World(WorldEvent::ShortCutInit(ShortCutInit { shortcuts: vec![Shortcut { slot: 0, kind: ShortcutKind::Skill { skill_id: 56, level: 1 }, character_type: 1 }] })));
    wire.push(npc_info(9, 20001, "Wolf", true, [200, 0, 0]));
    let events = step(&mut bot);
    assert_eq!(bot.state(), BotState::InWorld);
    assert!(events.contains(&BotEvent::State(BotState::InWorld)));
    assert_eq!(wire.sent()[..2], [NetCommand::Select(1), NetCommand::Appearing]);
    assert_eq!(wire.cool_time_requests(), 2);
    assert_eq!(wire.sent()[2], NetCommand::Action { object_id: 9, origin: [0, 0, 0], shift: false });

    wire.push(SessionEvent::World(WorldEvent::MyTargetSelected(MyTargetSelected { object_id: 9, color: 0 })));
    step(&mut bot);
    assert_eq!(wire.sent()[3], NetCommand::UseSkill { skill_id: 56, ctrl: false, shift: false });

    wire.push(die(9));
    wire.push(ground_item(30, 57, [210, 0, 0]));
    step(&mut bot);
    assert_eq!(wire.sent()[4], NetCommand::Action { object_id: 30, origin: [0, 0, 0], shift: false });

    wire.push(SessionEvent::World(WorldEvent::DeleteObject(DeleteObject { object_id: 30 })));
    wire.push(SessionEvent::World(WorldEvent::DeleteObject(DeleteObject { object_id: 9 })));
    step(&mut bot);

    wire.push(die(1));
    assert!(step(&mut bot).contains(&BotEvent::Died));
    assert_eq!(wire.sent().last(), Some(&NetCommand::TargetCancel));
    let after_loot = wire.sent().len();
    wire.push(npc_info(10, 20001, "Wolf", true, [100, 0, 0]));
    step(&mut bot);
    step(&mut bot);
    assert_eq!(wire.sent().len(), after_loot);

    wire.push(SessionEvent::World(WorldEvent::Revive(Revive { object_id: 1 })));
    assert!(step(&mut bot).contains(&BotEvent::Revived));
    step(&mut bot);
    assert!(wire.sent().len() > after_loot);
    assert_eq!(wire.sent().last(), Some(&NetCommand::Action { object_id: 10, origin: [0, 0, 0], shift: false }));
}
