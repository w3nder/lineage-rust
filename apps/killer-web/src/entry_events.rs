use killer_world::session::SessionEvent;

#[derive(Default)]
pub struct EntryEvents(Vec<SessionEvent>);

impl EntryEvents {
    pub fn push(&mut self, event: SessionEvent) {
        if matches!(event, SessionEvent::World(_) | SessionEvent::Time(_)) {
            self.0.push(event);
        }
    }

    pub fn take(&mut self) -> Vec<SessionEvent> {
        std::mem::take(&mut self.0)
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_world::{World, WorldEvent};
    use l2_net::game::packets::{ShortCutInit, Shortcut, ShortcutKind, SkillEntry, SkillList};

    fn shortcut(id: i32) -> Shortcut {
        Shortcut {slot:0,kind:ShortcutKind::Skill {skill_id:id,level:1},character_type:1}
    }

    fn skills(id: i32) -> SessionEvent {
        SessionEvent::World(WorldEvent::SkillList(SkillList {skills:vec![SkillEntry {id,level:1,passive:false}]}))
    }

    #[test]
    fn entry_preserves_lists_and_updates_in_wire_order_before_user_info() {
        let mut pending=EntryEvents::default();
        pending.push(skills(100));
        pending.push(SessionEvent::World(WorldEvent::ShortCutInit(ShortCutInit {shortcuts:vec![shortcut(100)]})));
        pending.push(SessionEvent::World(WorldEvent::ShortCutRegister(shortcut(200))));
        pending.push(skills(200));
        let mut world=World::default();
        for event in pending.take() {world.apply(&event);}
        assert!(world.me.is_none());
        assert_eq!(world.skills[0].id,200);
        assert_eq!(world.shortcuts,vec![shortcut(200)]);
        assert!(pending.take().is_empty());
    }

    #[test]
    fn new_session_discards_old_lists_and_lobby_events() {
        let mut pending=EntryEvents::default();
        pending.push(skills(100));
        pending.push(SessionEvent::World(WorldEvent::ShortCutInit(ShortCutInit {shortcuts:vec![shortcut(100)]})));
        pending.clear();
        pending.push(SessionEvent::Characters(Vec::new()));
        pending.push(SessionEvent::Selected);
        pending.push(skills(300));
        let mut world=World::default();
        for event in pending.take() {world.apply(&event);}
        assert_eq!(world.skills.len(),1);
        assert_eq!(world.skills[0].id,300);
        assert!(world.shortcuts.is_empty());
    }
}
