use crate::session::{BotEvent, BotSession};
use crate::snapshot::Snapshot;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct BotManager {
    sessions: BTreeMap<u32, BotSession>,
}

impl BotManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, id: u32, session: BotSession) {
        self.sessions.insert(id, session);
    }

    pub fn remove(&mut self, id: u32) -> bool {
        self.sessions.remove(&id).is_some()
    }

    pub fn ids(&self) -> Vec<u32> {
        self.sessions.keys().copied().collect()
    }

    pub fn tick(&mut self) -> Vec<(u32, BotEvent)> {
        let mut out = Vec::new();
        for (id, session) in &mut self.sessions {
            out.extend(session.tick().into_iter().map(|event| (*id, event)));
        }
        out
    }

    pub fn snapshot(&self, id: u32) -> Option<Snapshot> {
        self.sessions.get(&id).map(BotSession::snapshot)
    }

    pub fn session(&self, id: u32) -> Option<&BotSession> {
        self.sessions.get(&id)
    }

    pub fn session_mut(&mut self, id: u32) -> Option<&mut BotSession> {
        self.sessions.get_mut(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::ManualClock;
    use crate::profile::Profile;
    use crate::session::BotState;
    use crate::testkit::{Vitals, char_entry, user_info};
    use crate::transport::{FakeTransport, Transport};
    use killer_world::{NetCommand, NetEvent, SessionEvent};
    use std::sync::{Arc, Mutex};

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

    fn session(name: &str) -> (BotSession, Wire) {
        let text = format!("account = \"a\"\ncharacter = \"{name}\"\n[server]\nhost = \"h\"\nport = 1\n");
        let wire = Wire::default();
        (BotSession::new(Profile::parse(&text).unwrap(), Box::new(wire.clone()), Arc::new(ManualClock::new())), wire)
    }

    fn enter(wire: &Wire, name: &str) {
        let mut inbox = wire.0.lock().unwrap();
        inbox.push(NetEvent::Session(SessionEvent::Characters(vec![char_entry(name)])));
        inbox.push(NetEvent::Session(user_info(1, name, &Vitals::full(), [0, 0, 0])));
    }

    #[test]
    fn ticking_returns_events_tagged_with_the_account() {
        let mut manager = BotManager::new();
        let (first, wire) = session("Um");
        manager.add(7, first);
        enter(&wire, "Um");
        let events = manager.tick();
        assert!(events.contains(&(7, BotEvent::State(BotState::InWorld))));
        assert_eq!(manager.snapshot(7).unwrap().name, "Um");
        assert!(manager.snapshot(8).is_none());
    }

    #[test]
    fn one_account_disconnecting_does_not_stop_the_others() {
        let mut manager = BotManager::new();
        let (a, wire_a) = session("A");
        let (b, wire_b) = session("B");
        manager.add(1, a);
        manager.add(2, b);
        enter(&wire_a, "A");
        enter(&wire_b, "B");
        manager.tick();
        wire_a.0.lock().unwrap().push(NetEvent::Closed);
        let events = manager.tick();
        assert!(events.contains(&(1, BotEvent::State(BotState::Disconnected))));
        assert_eq!(manager.session(2).unwrap().state(), BotState::InWorld);
    }

    #[test]
    fn removing_an_account_drops_its_session() {
        let mut manager = BotManager::new();
        manager.add(1, session("A").0);
        assert_eq!(manager.ids(), vec![1]);
        assert!(manager.remove(1));
        assert!(!manager.remove(1));
        assert!(manager.session(1).is_none());
    }
}
