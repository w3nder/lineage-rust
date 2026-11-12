use killer_assets::client::DatTables;
use killer_ui::train::{TrainHit, TrainView, LearnableView, TrainInfoView, TrainRequirement, EnchantInfo};
use killer_ui::window::{Desktop, WindowId};
use killer_world::{NetCommand, World, WorldEvent};

use crate::ui_keys::UiCommand;

pub struct TrainWindow {
    pub view: TrainView,
    pub commands: Vec<UiCommand>,
    mode: i32,
}

impl TrainWindow {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self { view: TrainView::new(lookup), commands: Vec::new(), mode: 0 }
    }

    pub fn world_event(&mut self, event: &WorldEvent, world: &World, tables: Option<&DatTables>, desktop: &mut Desktop) {
        self.view.player_sp = world.me.as_ref().map(|_| world.stats.sp);
        self.view.player_exp = world.me.as_ref().and_then(|me| killer_ui::train::enchant_available_exp(me.level, me.exp));
        match event {
            WorldEvent::AcquireSkillList(list) => {
                self.mode = list.mode;
                if !list.skills.is_empty() { self.view.enchant_list = false; }
                self.view.set_list(list.skills.iter().map(|skill| {
                    let key = (skill.id as u32, skill.next_level as u32);
                    let row = tables.and_then(|t| t.skillgrp.get(&key));
                    let name = tables.and_then(|t| t.skillname.get(&key));
                    LearnableView {
                        enchant_description: None,
                        id: skill.id, level: skill.next_level, sp_cost: skill.sp_cost,
                        icon: row.map(|r| r.icon.clone()).filter(|s| !s.is_empty()).unwrap_or_else(|| "NWindow.BlackTexture".into()),
                        name: row.map(|_| name.map(|n| n.name.as_str()).unwrap_or("NoSkillName")).unwrap_or_default().to_owned(),
                    }
                }).collect());
                visible(desktop, true);
            }
            WorldEvent::AcquireSkillInfo(info) => {
                let key = (info.id as u32, info.level as u32);
                let row = tables.and_then(|t| t.skillgrp.get(&key));
                let name = tables.and_then(|t| t.skillname.get(&key));
                let category = row.and_then(|r| {
                    let id = if r.is_magic != 0 { 313 } else if r.oper_type == 2 { 312 } else { 311 };
                    tables.and_then(|t| t.sysstring.get(&id)).cloned()
                }).unwrap_or_default();
                let requirements = info.requirements.iter().filter_map(|req| {
                    let id = u32::try_from(req.item_id).ok()?;
                    let tables = tables?;
                    let item = tables.item_icons.get(id)?;
                    Some(TrainRequirement {
                        item_id: req.item_id, count: req.count,
                        name: tables.itemname.get(&id).map(|n| n.name.clone()).unwrap_or_default(),
                        icon: if item.icons[0].is_empty() { "NWindow.BlackTexture".into() } else { item.icons[0].clone() },
                    })
                }).take(4).collect();
                self.view.show_info(TrainInfoView {
                    enchant: None,
                    id: info.id, level: info.level, mode: info.mode, sp_cost: info.sp_cost,
                    name: row.map(|_| name.map(|n| n.name.as_str()).unwrap_or("NoSkillName")).unwrap_or_default().to_owned(),
                    description: name.map(|n| n.description.replace("\\n", "\r\n")).unwrap_or_default(),
                    icon: row.map(|r| r.icon.clone()).filter(|s| !s.is_empty()).unwrap_or_else(|| "NWindow.BlackTexture".into()),
                    category, mp: row.map(|r| r.mp_consume as i32).unwrap_or(0),
                    range: row.map(|r| r.cast_range).unwrap_or(0), requirements,
                });
                visible(desktop, true);
            }
            WorldEvent::EnchantSkillList(list) => {
                let skills = list.entries.iter().map(|row| l2_net::game::LearnableSkill {
                    id: row[0], next_level: row[1], max_level: 0, sp_cost: row[2], requirements: 0,
                }).collect();
                self.world_event(&WorldEvent::AcquireSkillList(l2_net::game::AcquireSkillList { mode: 0, skills }), world, tables, desktop);
                if !list.entries.is_empty() { self.view.enchant_list = true; }
                for row in &mut self.view.list {
                    row.enchant_description = Some(tables.and_then(|t| t.skillname.get(&(row.id as u32, row.level as u32))).map(|n| n.desc_add1.clone()).unwrap_or_default());
                }
            }
            WorldEvent::EnchantSkillInfo(info) => {
                let [id, level, sp_cost, exp_cost, chance] = info.header;
                let requirements = info.entries.iter().map(|r| l2_net::game::SkillRequirement {
                    kind: r[0], item_id: r[1], count: r[2], unknown: r[3],
                }).collect();
                self.world_event(&WorldEvent::AcquireSkillInfo(l2_net::game::AcquireSkillInfo { id, level, sp_cost, mode: 0, requirements }), world, tables, desktop);
                let name = tables.and_then(|t| t.skillname.get(&(id as u32, level as u32)));
                if let Some(view) = &mut self.view.info {
                    view.enchant = Some(EnchantInfo {
                        exp_cost, chance,
                        route: name.map(|n| n.desc_add1.clone()).unwrap_or_default(),
                        description: name.map(|n| n.desc_add2.clone()).unwrap_or_default(),
                    });
                }
            }
            WorldEvent::AcquireSkillDone(_) => {
                self.view.pressed = None;
                self.view.scroll.slider.mouse_up();
                visible(desktop, false);
            }
            _ => {}
        }
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, _desktop: &mut Desktop) {
        self.view.pressed = self.view.hit(x, y);
        match self.view.pressed {
            Some(TrainHit::Skill(index)) => {
                self.view.select(index);
                if let Some(skill) = self.view.list.get(index) {
                    let command = if skill.enchant_description.is_some() {
                        NetCommand::ExEnchantSkillInfo { skill_id: skill.id, level: skill.level }
                    } else {
                        NetCommand::AcquireSkillInfo { skill_id: skill.id, level: skill.level, mode: self.mode }
                    };
                    self.commands.push(UiCommand::Net(command));
                }
            }
            Some(TrainHit::Scroll(killer_ui::scroll::ScrollHit::Slider(y))) => { self.view.scroll.slider.mouse_down(y); }
            _ => {}
        }
    }

    pub fn mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        self.view.scroll.slider.mouse_up();
        let Some(pressed) = self.view.pressed.take() else { return; };
        let released = over.filter(|(id, _, _)| *id == WindowId::TRAIN).and_then(|(_, x, y)| self.view.hit(x, y));
        if released != Some(pressed) { return; }
        match pressed {
            TrainHit::Scroll(hit) => self.view.scroll(hit),
            TrainHit::Learn => {
                if let Some(info) = &self.view.info {
                    let command = if info.enchant.is_some() {
                        NetCommand::ExEnchantSkill { skill_id: info.id, level: info.level }
                    } else {
                        NetCommand::AcquireSkill { skill_id: info.id, level: info.level, mode: info.mode }
                    };
                    self.commands.push(UiCommand::Net(command));
                }
            }
            TrainHit::Cancel => self.view.back(),
            TrainHit::Close => {
                visible(desktop, false);
            }
            TrainHit::Skill(_) => {}
        }
    }

    pub fn wheel(&mut self, delta: i32) {
        if delta != 0 { self.view.scroll(if delta > 0 { killer_ui::scroll::ScrollHit::Up } else { killer_ui::scroll::ScrollHit::Down }); }
    }
}

fn visible(desktop: &mut Desktop, value: bool) {
    if let Some(window) = desktop.window_mut(WindowId::TRAIN) { window.visible = value; }
    if value && let Some(index) = desktop.windows.iter().position(|w| w.id == WindowId::TRAIN) {
        let window = desktop.windows.remove(index);
        desktop.windows.push(window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::train::{LIST_RECT, OK_RECT, CANCEL_RECT};
    use killer_ui::window::{Rect, Window};
    use l2_net::game::{AcquireSkillInfo, AcquireSkillList, LearnableSkill};

    fn desktop() -> Desktop {
        let mut desktop = Desktop::new((800, 600));
        desktop.windows.push(Window::new(WindowId::TRAIN, Rect::new(0, 0, 256, 401), killer_ui::train::grip()));
        desktop
    }

    fn click(train: &mut TrainWindow, desktop: &mut Desktop, rect: Rect) {
        train.mouse_down(rect.x + 1, rect.y + 1, desktop);
        train.mouse_up(Some((WindowId::TRAIN, rect.x + 1, rect.y + 1)), desktop);
    }

    #[test]
    fn trainer_100837f0_1011c480_1011c570_receives_details_without_selection_and_returns_to_list() {
        let mut train = TrainWindow::new(|_| None);
        let mut desktop = desktop();
        let world = World::default();
        let info = AcquireSkillInfo { id: 55, level: 3, mode: -7, sp_cost: 120, requirements: vec![] };
        train.world_event(&WorldEvent::AcquireSkillInfo(info), &world, None, &mut desktop);
        assert!(desktop.window(WindowId::TRAIN).unwrap().visible);
        assert_eq!(train.view.selected, None);
        click(&mut train, &mut desktop, OK_RECT);
        assert!(matches!(train.commands.pop(), Some(UiCommand::Net(NetCommand::AcquireSkill { skill_id: 55, level: 3, mode: -7 }))));
        assert!(desktop.window(WindowId::TRAIN).unwrap().visible);
        click(&mut train, &mut desktop, CANCEL_RECT);
        assert!(train.view.info.is_none());
        assert!(desktop.window(WindowId::TRAIN).unwrap().visible);
        assert!(train.commands.is_empty());
        assert_eq!(train.view.hit(OK_RECT.x + 1, OK_RECT.y + 1), None);
    }

    #[test]
    fn trainer_frame_close_10011920_hides_on_release_inside() {
        let mut train = TrainWindow::new(|_| None);
        let mut desktop = desktop();
        let world = World::default();
        train.world_event(&WorldEvent::AcquireSkillList(AcquireSkillList { mode: 0, skills: vec![] }), &world, None, &mut desktop);
        train.mouse_down(245, 10, &mut desktop);
        assert!(desktop.window(WindowId::TRAIN).unwrap().visible);
        train.mouse_up(Some((WindowId::TRAIN, 100, 100)), &mut desktop);
        assert!(desktop.window(WindowId::TRAIN).unwrap().visible && train.commands.is_empty());
        train.mouse_down(245, 10, &mut desktop);
        train.mouse_up(Some((WindowId::TRAIN, 245, 10)), &mut desktop);
        assert!(!desktop.window(WindowId::TRAIN).unwrap().visible);
    }

    #[test]
    fn trainer_100680d0_10068160_preserves_mode_and_hides_without_discarding_pages() {
        for mode in [i32::MIN, -1, 0, 1, 2, i32::MAX] {
            let mut train = TrainWindow::new(|_| None);
            let mut desktop = desktop();
            let world = World::default();
            let list = AcquireSkillList { mode, skills: vec![LearnableSkill { id: 3, next_level: 2, max_level: 9, sp_cost: 10, requirements: 0 }; 20] };
            train.world_event(&WorldEvent::AcquireSkillList(list.clone()), &world, None, &mut desktop);
            train.wheel(-1);
            assert_eq!(train.view.first_row, 1);
            click(&mut train, &mut desktop, LIST_RECT);
            assert!(matches!(train.commands.pop(), Some(UiCommand::Net(NetCommand::AcquireSkillInfo { skill_id: 3, level: 2, mode: sent })) if sent == mode));
            train.world_event(&WorldEvent::AcquireSkillInfo(AcquireSkillInfo { id: 9, level: 5, sp_cost: 400, mode, requirements: vec![] }), &world, None, &mut desktop);
            assert_eq!(train.view.hit(LIST_RECT.x + 1, LIST_RECT.y + 1), None);
            click(&mut train, &mut desktop, CANCEL_RECT);
            assert_eq!((train.view.first_row, train.view.selected), (1, Some(1)));
            train.world_event(&WorldEvent::AcquireSkillDone(l2_net::game::AcquireSkillDone), &world, None, &mut desktop);
            assert!(!desktop.window(WindowId::TRAIN).unwrap().visible);
            assert_eq!(train.view.list.len(), 20);
            train.world_event(&WorldEvent::AcquireSkillList(list), &world, None, &mut desktop);
            assert_eq!((train.view.first_row, train.view.selected), (0, None));
            assert!(desktop.window(WindowId::TRAIN).unwrap().visible);
        }
    }

    #[test]
    fn trainer_replacing_a_page_cancels_pressed_buttons_and_slider_capture() {
        let mut train = TrainWindow::new(|_| None);
        let mut desktop = desktop();
        let world = World::default();
        let info = WorldEvent::AcquireSkillInfo(AcquireSkillInfo { id: 7, level: 1, mode: 0, sp_cost: 0, requirements: vec![] });
        train.world_event(&info, &world, None, &mut desktop);
        train.mouse_down(OK_RECT.x + 1, OK_RECT.y + 1, &mut desktop);
        train.world_event(&WorldEvent::AcquireSkillList(AcquireSkillList { mode: 0, skills: vec![] }), &world, None, &mut desktop);
        train.mouse_up(Some((WindowId::TRAIN, OK_RECT.x + 1, OK_RECT.y + 1)), &mut desktop);
        assert!(train.commands.is_empty());
        train.view.set_list(vec![LearnableView::default(); 30]);
        let thumb_y = LIST_RECT.y + killer_ui::scroll::BUTTON_SIZE + 2;
        train.mouse_down(LIST_RECT.x + LIST_RECT.w - 3, thumb_y, &mut desktop);
        assert!(train.view.scroll.slider.dragging);
        assert!(train.view.mouse_move(thumb_y + 100));
        assert!(train.view.first_row > 0);
        train.mouse_up(None, &mut desktop);
        assert!(!train.view.scroll.slider.dragging);
        let first = train.view.first_row;
        train.wheel(0);
        assert_eq!(train.view.first_row, first);
    }

    #[test]
    fn trainer_enchant_packets_open_the_same_window_and_use_extended_requests() {
        let mut train = TrainWindow::new(|_| None);
        let mut desktop = desktop();
        let world = World::default();
        train.world_event(&WorldEvent::EnchantSkillList(l2_net::game::EnchantSkillList { entries: vec![[3, 101, 40, 60]] }), &world, None, &mut desktop);
        assert!(desktop.window(WindowId::TRAIN).unwrap().visible);
        assert!(train.view.enchant_list);
        click(&mut train, &mut desktop, LIST_RECT);
        assert!(matches!(train.commands.pop(), Some(UiCommand::Net(NetCommand::ExEnchantSkillInfo { skill_id: 3, level: 101 }))));
        train.world_event(&WorldEvent::EnchantSkillInfo(l2_net::game::EnchantSkillInfo { header: [4, 102, 50, 70, 90], entries: vec![] }), &world, None, &mut desktop);
        let detail = train.view.info.as_ref().unwrap();
        let enchant = detail.enchant.as_ref().unwrap();
        assert_eq!((detail.sp_cost, enchant.exp_cost, enchant.chance), (50, 70, 90));
        click(&mut train, &mut desktop, OK_RECT);
        assert!(matches!(train.commands.pop(), Some(UiCommand::Net(NetCommand::ExEnchantSkill { skill_id: 4, level: 102 }))));
        click(&mut train, &mut desktop, CANCEL_RECT);
        assert!(train.view.info.is_none());
        assert_eq!(train.view.list[0].id, 3);
        assert!(desktop.window(WindowId::TRAIN).unwrap().visible);
        train.world_event(&WorldEvent::AcquireSkillList(AcquireSkillList { mode: 2, skills: vec![LearnableSkill { id: 5, next_level: 1, max_level: 1, sp_cost: 0, requirements: 0 }] }), &world, None, &mut desktop);
        assert!(!train.view.enchant_list);
        click(&mut train, &mut desktop, LIST_RECT);
        assert!(matches!(train.commands.pop(), Some(UiCommand::Net(NetCommand::AcquireSkillInfo { skill_id: 5, level: 1, mode: 2 }))));
    }

    #[test]
    #[ignore]
    fn trainer_matches_c4_selection_and_acquire_requests() {
        let output = std::process::Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
            .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/train_reference.py"))
            .output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let mut train = TrainWindow::new(|_| None);
        let mut desktop = desktop();
        let mut count = 0;
        for line in String::from_utf8(output.stdout).unwrap().lines() {
            let case: serde_json::Value = serde_json::from_str(line).unwrap();
            let n = |key: &str| case[key].as_i64().unwrap() as i32;
            if case["kind"] == "hit" {
                train.view.set_list(vec![LearnableView::default(); n("count") as usize]);
                train.view.first_row = n("first") as usize;
                train.view.selected = usize::try_from(n("selected")).ok();
                assert_eq!(train.view.skill_at(n("x"), n("y")), usize::try_from(n("result")).ok(), "{case}");
            } else if case["kind"] == "exp" {
                let values: Vec<_> = killer_ui::train::enchant_available_exp(n("level"), n("exp")).map(|n| n.to_string()).into_iter().collect();
                assert_eq!(case["values"], serde_json::json!(values), "{case}");
            } else if case["kind"] == "enchant" {
                train.world_event(&WorldEvent::EnchantSkillInfo(l2_net::game::EnchantSkillInfo { header: [n("skill"), n("level"), 1, 2, 3], entries: vec![] }), &World::default(), None, &mut desktop);
                click(&mut train, &mut desktop, OK_RECT);
                let Some(UiCommand::Net(NetCommand::ExEnchantSkill { skill_id, level })) = train.commands.pop() else { panic!("missing enchant request"); };
                assert_eq!(case["calls"], serde_json::json!([format!("UNetworkHandler::RequestExEnchantSkill([{skill_id}, {level}])")]));
            } else {
                train.view.show_info(TrainInfoView { id: n("skill"), level: n("level"), mode: n("mode"), ..Default::default() });
                click(&mut train, &mut desktop, OK_RECT);
                let Some(UiCommand::Net(NetCommand::AcquireSkill { skill_id, level, mode })) = train.commands.pop() else { panic!("missing request"); };
                assert_eq!(case["calls"], serde_json::json!([format!("UNetworkHandler::RequestAcquireSkill([{skill_id}, {level}, {mode}])")]));
            }
            count += 1;
        }
        assert_eq!(count, 2800);
    }

    #[test]
    #[ignore]
    fn trainer_all_client_skills_and_requirements_render_with_original_assets() {
        use killer_assets::{client::Client, textures::{BankSource, TextureBank, write_png}};
        use killer_ui::{draw::DrawList, font::{BitmapFont, Glyphs}};
        use killer_render::{camera::Camera, environment::Environment, headless::Headless};
        let root = std::env::var_os("KILLER_CLIENT_DIR").unwrap();
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let font = BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap());
        let mut train = TrainWindow::new(|id| tables.sysstring.get(&id).cloned());
        let mut desktop = desktop();
        let world = World::default();
        let mut skills: Vec<_> = tables.skillgrp.keys().copied().collect();
        skills.sort();
        let list = AcquireSkillList { mode: 2, skills: skills.iter().map(|&(id, level)| LearnableSkill {
            id: id as i32, next_level: level as i32, max_level: level as i32, sp_cost: 123, requirements: 0,
        }).collect() };
        train.world_event(&WorldEvent::AcquireSkillList(list), &world, Some(&tables), &mut desktop);
        assert_eq!(train.view.list.len(), skills.len());
        let mut painted = 0;
        for first in (0..skills.len()).step_by(killer_ui::train::LIST_ROWS) {
            train.view.first_row = first;
            let mut draw = DrawList::default();
            train.view.paint(&mut draw, 0, 0, &font, 1.0);
            assert!(!draw.quads.is_empty());
            painted += train.view.list[first..].len().min(killer_ui::train::LIST_ROWS);
        }
        assert_eq!(painted, skills.len());
        for &(id, level) in &skills {
            train.world_event(&WorldEvent::AcquireSkillInfo(AcquireSkillInfo { id: id as i32, level: level as i32, sp_cost: 123, mode: 2, requirements: vec![] }), &world, Some(&tables), &mut desktop);
            let info = train.view.info.as_ref().unwrap();
            assert_eq!(info.mp, tables.skillgrp[&(id, level)].mp_consume as i32);
            assert_eq!(info.range, tables.skillgrp[&(id, level)].cast_range);
            assert_eq!(info.name, tables.skillname.get(&(id, level)).map(|s| s.name.as_str()).unwrap_or("NoSkillName"));
            let mut draw = DrawList::default();
            train.view.paint(&mut draw, 0, 0, &font, 1.0);
            assert!(!draw.quads.is_empty());
            train.world_event(&WorldEvent::EnchantSkillInfo(l2_net::game::EnchantSkillInfo { header: [id as i32, level as i32, 123, 456, 78], entries: vec![] }), &world, Some(&tables), &mut desktop);
            let enchant = train.view.info.as_ref().unwrap().enchant.as_ref().unwrap();
            assert_eq!(enchant.route, tables.skillname.get(&(id, level)).map(|s| s.desc_add1.as_str()).unwrap_or_default());
            draw.quads.clear();
            train.view.paint(&mut draw, 0, 0, &font, 1.0);
            assert!(!draw.quads.is_empty());
        }
        let mut ids: Vec<_> = tables.item_icons.items.keys().copied().collect();
        ids.sort_unstable();
        let mut requirements = 0;
        for chunk in ids.chunks(4) {
            train.world_event(&WorldEvent::AcquireSkillInfo(AcquireSkillInfo {
                id: 3, level: 1, sp_cost: 123, mode: 2,
                requirements: chunk.iter().map(|&id| l2_net::game::SkillRequirement { kind: 99, item_id: id as i32, count: 2, unknown: 0 }).collect(),
            }), &world, Some(&tables), &mut desktop);
            let info = train.view.info.as_ref().unwrap();
            assert_eq!(info.requirements.len(), chunk.len());
            for (req, id) in info.requirements.iter().zip(chunk) { assert_eq!(req.name, tables.itemname.get(id).map(|n| n.name.as_str()).unwrap_or_default()); }
            let mut draw = DrawList::default();
            train.view.paint(&mut draw, 0, 0, &font, 1.0);
            requirements += chunk.len();
        }
        train.view.back();
        train.view.first_row = 0;
        train.view.selected = Some(1);
        train.view.player_sp = Some(12345);
        train.view.scroll.slider.set_pos(0);
        let mut draw = DrawList::default();
        train.view.paint(&mut draw, 10, 10, &font, 1.0);
        train.world_event(&WorldEvent::AcquireSkillInfo(AcquireSkillInfo {
            id: 3, level: 1, sp_cost: 123, mode: 2,
            requirements: [57, 1830, 4037, 8615, 57].map(|id| l2_net::game::SkillRequirement { kind: 99, item_id: id, count: 2, unknown: 0 }).to_vec(),
        }), &world, Some(&tables), &mut desktop);
        assert_eq!(train.view.info.as_ref().unwrap().requirements.len(), 4);
        train.view.info.as_mut().unwrap().requirements.truncate(2);
        train.view.player_sp = Some(12345);
        train.view.paint(&mut draw, 286, 10, &font, 1.0);
        let &(id, level) = skills.iter().find(|&&(id, level)| tables.skillgrp[&(id, level)].is_ench != 0 && tables.skillname.get(&(id, level)).is_some_and(|s| !s.desc_add1.is_empty())).unwrap();
        train.world_event(&WorldEvent::EnchantSkillInfo(l2_net::game::EnchantSkillInfo {
            header: [id as i32, level as i32, 123, 456, 78], entries: vec![[99, 57, 2, 0]],
        }), &world, Some(&tables), &mut desktop);
        train.view.player_sp = Some(12345);
        train.view.player_exp = Some(98765);
        train.view.paint(&mut draw, 562, 10, &font, 1.0);
        let mut headless = Headless::new(828, 422).unwrap();
        let mut bank = TextureBank::new();
        headless.renderer_mut().set_ui(&draw, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        assert_eq!(bank.missing().count(), 0);
        let camera = Camera { eye: glam::vec3(0.0, -300.0, 0.0), target: glam::Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
        let rgba = headless.render(&camera, &Environment::default()).unwrap();
        write_png(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-train.png"), 828, 422, &rgba).unwrap();
        eprintln!("trainer: {} skills, {painted} list rows, {requirements} requirement items", skills.len());
    }
}
