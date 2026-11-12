use crate::ui2d::GameUi;
use killer_world::{EntityKind, World};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Request {
    System,
    #[default]
    Normal,
    Resource(u8),
    Texture(String),
}

impl Request {
    #[cfg(target_arch = "wasm32")]
    pub fn key(&self) -> String {
        match self {
            Self::System => "system".into(),
            Self::Normal => "0".into(),
            Self::Resource(index) => index.to_string(),
            Self::Texture(path) => format!("@{path}"),
        }
    }
}

pub fn select(ui: &GameUi, world: &World, hovered: Option<(i32, u64)>) -> Request {
    if ui.option_game_runtime.as_ref().is_some_and(|runtime| runtime.state.color_cursor == 0) {
        return Request::System;
    }
    if let Some(index) = ui.resize_cursor() {
        return Request::Resource(index);
    }
    if let Some(icon) = ui.keys.macros.drag_icon.clone().or_else(|| ui.keys.drag_icon(&ui.shortcut)) {
        return Request::Texture(icon);
    }
    if ui.keys.special_maps.town.is_dragging() {
        return Request::Resource(3);
    }
    if ui.window_under(ui.cursor.0, ui.cursor.1).is_some()
        || ui.keys.minimized.captured()
        || ui.keys.minimized.hit(&ui.desktop, ui.cursor.0, ui.cursor.1) {
        return Request::Normal;
    }
    world_cursor(world, hovered, ui.keys.ctrl)
}

pub fn world_cursor(world: &World, hovered: Option<(i32, u64)>, force: bool) -> Request {
    let attack = if force { Request::Resource(1) } else { Request::Normal };
    let Some((id, spawned)) = hovered else { return attack; };
    let Some(target) = world.entity(id).filter(|entity| entity.spawned == spawned) else { return Request::Normal; };
    if matches!(target.kind, EntityKind::Item { .. }) { return Request::Resource(3); }
    if world.target != Some(id) { return attack; }
    let me_pledge = world.me.as_ref().map_or(0, |me| me.clan_id);
    if me_pledge > 0 && target.clan_id > 0
        && world.pledges.get(&me_pledge).is_some_and(|pledge| pledge.enemy_pledge_id == target.clan_id)
    {
        let surrendered = |clan_id, war_id| world.pledges.get(&clan_id).is_some_and(|pledge| pledge.surrender_war_id == war_id);
        return if surrendered(me_pledge, world.me.as_ref().map_or(0, |me| me.surrender_war_id))
            || surrendered(target.clan_id, target.surrender_war_id)
        { Request::Normal } else { Request::Resource(1) };
    }
    if force || (!target.dead && target.auto_attack) { return Request::Resource(1); }
    if (!target.dead && !target.auto_attack && matches!(target.kind, EntityKind::Npc { .. }))
        || matches!(target.store_type, 1 | 3 | 5 | 8)
    { return Request::Resource(2); }
    Request::Normal
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn update_cursor_ui_capture_precedes_world_and_restores_after_release() {
        let mut ui = crate::ui2d::tests::game_ui((1280, 720));
        let world = World::default();
        ui.keys.ctrl = true;
        ui.cursor = (640, 360);
        ui.keys.special_maps.town.mouse_down(80, 80);
        assert_eq!(select(&ui, &world, None), Request::Resource(3));
        ui.keys.macros.drag_icon = Some("icon.skill0001".into());
        assert_eq!(select(&ui, &world, None), Request::Texture("icon.skill0001".into()));
        ui.keys.macros.drag_icon = None;
        ui.keys.special_maps.town.mouse_up();
        assert_eq!(select(&ui, &world, None), Request::Resource(1));
    }

    #[test]
    fn resize_frame_10012200_selects_cursor_6_over_the_grip_and_while_dragging() {
        use killer_ui::window::WindowId;
        let mut ui = crate::ui2d::tests::game_ui((1280, 720));
        let world = World::default();
        let target = ui.desktop.window(WindowId::TARGET).unwrap().rect;
        ui.cursor = (target.x + target.w - 5, target.y + 10);
        assert_eq!(select(&ui, &world, None), Request::Resource(6));
        ui.mouse_down(ui.cursor.0, ui.cursor.1);
        ui.cursor = (640, 600);
        assert_eq!(select(&ui, &world, None), Request::Resource(6));
        ui.mouse_up();
        assert_eq!(select(&ui, &world, None), Request::Normal);
    }

    #[test]
    fn minimized_window_and_its_capture_keep_world_cursor_out() {
        let mut ui = crate::ui2d::tests::game_ui((1280, 720));
        let world = World::default();
        ui.keys.ctrl = true;
        for window in &mut ui.desktop.windows { window.visible = false; }
        ui.keys.minimized.minimize_match(&mut ui.desktop, "Party");
        let icon = ui.keys.minimized.party_match.rect;
        ui.cursor = (icon.x + 4, icon.y + 4);
        assert_eq!(select(&ui, &world, None), Request::Normal);
        ui.mouse_down(ui.cursor.0, ui.cursor.1);
        ui.cursor = (1279, 719);
        assert_eq!(select(&ui, &world, None), Request::Normal);
        ui.keys.minimized.hide_match();
        assert_eq!(select(&ui, &world, None), Request::Resource(1));
    }

    #[test]
    fn update_cursor_1007d930_covers_selection_ctrl_dead_npc_shop_pickup_and_generation() {
        let mut world = crate::characters::demo_world(Vec3::ZERO, 0, 0);
        let mut target = world.me.clone().unwrap();
        target.object_id = 432;
        target.spawned = 72;
        target.auto_attack = true;
        world.entities.insert(432, target);
        let hover = Some((432, 72));
        assert_eq!(world_cursor(&world, hover, false), Request::Normal);
        assert_eq!(world_cursor(&world, hover, true), Request::Resource(1));
        world.target = Some(432);
        assert_eq!(world_cursor(&world, hover, false), Request::Resource(1));
        world.entities.get_mut(&432).unwrap().dead = true;
        assert_eq!(world_cursor(&world, hover, false), Request::Normal);
        assert_eq!(world_cursor(&world, hover, true), Request::Resource(1));
        let target = world.entities.get_mut(&432).unwrap();
        target.dead = false;
        target.auto_attack = false;
        target.kind = EntityKind::Npc { npc_id: 30001, rhand: 0, chest: 0, lhand: 0, attackable: false };
        assert_eq!(world_cursor(&world, hover, false), Request::Resource(2));
        world.entities.get_mut(&432).unwrap().dead = true;
        for kind in 0..=9 {
            world.entities.get_mut(&432).unwrap().store_type = kind;
            assert_eq!(world_cursor(&world, hover, false), if matches!(kind, 1 | 3 | 5 | 8) {Request::Resource(2)} else {Request::Normal});
        }
        world.entities.get_mut(&432).unwrap().kind = EntityKind::Item { item_id: 57, count: 1, stackable: true };
        assert_eq!(world_cursor(&world, hover, true), Request::Resource(3));
        assert_eq!(world_cursor(&world, Some((432, 71)), true), Request::Normal);
        assert_eq!(world_cursor(&world, None, false), Request::Normal);
        assert_eq!(world_cursor(&world, None, true), Request::Resource(1));
    }

    #[test]
    fn update_cursor_1007d930_checks_both_surrenders_before_ctrl() {
        let mut world = crate::characters::demo_world(Vec3::ZERO, 0, 0);
        world.me.as_mut().unwrap().clan_id = 10;
        let mut target = world.me.clone().unwrap();
        target.object_id = 432;
        target.clan_id = 20;
        let hover = Some((432, target.spawned));
        world.entities.insert(432, target);
        world.target = Some(432);
        world.pledges.insert(10, killer_world::PledgeRecord { enemy_pledge_id: 20, surrender_war_id: 1, ..Default::default() });
        world.pledges.insert(20, killer_world::PledgeRecord { surrender_war_id: 1, ..Default::default() });
        assert_eq!(world_cursor(&world, hover, false), Request::Resource(1));
        world.me.as_mut().unwrap().surrender_war_id = 1;
        assert_eq!(world_cursor(&world, hover, true), Request::Normal);
        world.me.as_mut().unwrap().surrender_war_id = 0;
        world.entities.get_mut(&432).unwrap().surrender_war_id = 1;
        assert_eq!(world_cursor(&world, hover, false), Request::Normal);
    }

    #[test]
    fn update_cursor_1007d880_leaves_the_normal_cursor_over_a_door_even_with_the_attack_key() {
        let mut world = crate::characters::demo_world(Vec3::ZERO, 0, 0);
        world.apply_world(&killer_world::WorldEvent::DoorInfo(l2_net::game::DoorInfo { object_id: 100, door_id: 24190001 }));
        assert!(world.select(100));
        assert_eq!(world_cursor(&world, Some((100, 0)), false), Request::Normal);
        assert_eq!(world_cursor(&world, Some((100, 0)), true), Request::Normal);
    }
}
