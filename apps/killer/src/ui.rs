use crate::app::WorldView;
use egui::{Align2, Area, Color32, Context, Frame, Id, Label, RichText};
use killer_world::{Entity, EntityKind, World};
use killer_ui::status::STATUS_SIZE;
use l2_net::game::describe_server_opcode;

const HUD_NEAREST: usize = 10;
const HUD_MARGIN: f32 = 8.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum HudMode {
    #[default]
    Collapsed,
    Expanded,
    Hidden,
}

impl HudMode {
    pub fn next(self) -> HudMode {
        match self {
            HudMode::Collapsed => HudMode::Expanded,
            HudMode::Expanded => HudMode::Hidden,
            HudMode::Hidden => HudMode::Collapsed,
        }
    }
}

pub fn entity_line(entity: &Entity, distance: f32) -> String {
    let kind = match entity.kind {
        EntityKind::Npc { .. } => "npc",
        EntityKind::Player { .. } => "jogador",
        EntityKind::Item { .. } => "item",
    };
    let state = if entity.dead {
        "morto"
    } else if entity.dest.is_some() {
        "andando"
    } else if entity.sitting {
        "sentado"
    } else {
        "parado"
    };
    let name = if entity.name.is_empty() { "?" } else { entity.name.as_str() };
    format!("{name:<16} {distance:>6.0}  {kind:<7} {state}")
}

pub fn target_line(world: &World) -> Option<String> {
    let id = world.target?;
    let name = match world.target_entity() {
        Some(entity) if !entity.name.is_empty() => entity.name.clone(),
        Some(_) => format!("#{id}"),
        None => format!("#{id} (fora de vista)"),
    };
    Some(format!("alvo: {name}"))
}

pub struct HudInfo<'a> {
    pub fps: f32,
    pub hour: f32,
    pub ignored_total: u32,
    pub ignored_distinct: usize,
    pub ignored_top: Vec<(u8, u32)>,
    pub tables_error: Option<&'a str>,
    pub ui_error: Option<&'a str>,
}

pub fn hud(ctx: &Context, view: &WorldView, world: &World, info: &HudInfo, mode: HudMode) -> bool {
    match mode {
        HudMode::Hidden => false,
        HudMode::Collapsed => {
            Area::new(Id::new("hud")).anchor(Align2::RIGHT_TOP, [-HUD_MARGIN, HUD_MARGIN]).show(ctx, |ui| {
                Frame::popup(ui.style()).show(ui, |ui| {
                    ui.add(Label::new(RichText::new(format!("fps {:.0}  versão {}  H: mais", info.fps, crate::app::version())).monospace()).extend());
                });
            });
            false
        }
        HudMode::Expanded => expanded_hud(ctx, view, world, info),
    }
}

pub fn expanded_hud_offset(pixels_per_point: f32) -> [f32; 2] {
    [HUD_MARGIN, STATUS_SIZE.1 as f32 / pixels_per_point + HUD_MARGIN]
}

fn expanded_hud(ctx: &Context, view: &WorldView, world: &World, info: &HudInfo) -> bool {
    let HudInfo { fps, hour, ignored_total, ignored_distinct, ignored_top, tables_error, ui_error } = info;
    let camera = view.current();
    let me = world.me.as_ref();
    let mut leave = false;
    Area::new(Id::new("hud")).anchor(Align2::LEFT_TOP, expanded_hud_offset(ctx.pixels_per_point())).show(ctx, |ui| {
        Frame::popup(ui.style()).show(ui, |ui| {
            ui.monospace(format!("fps {fps:.0}  versão {}  H: recolher/esconder", crate::app::version()));
            if let Some(me) = me {
                let state = if me.dest.is_some() { "andando" } else { "parado" };
                ui.monospace(format!("{}  {:.0} {:.0} {:.0}  h {}  {state}", me.name, me.position.x, me.position.y, me.position.z, me.heading));
            }
            ui.monospace(format!("hora {:02}:{:02}", hour.floor() as u32 % 24, (hour.fract() * 60.0) as u32));
            let loaded: Vec<&str> = view.neighbours.tiles.keys().map(String::as_str).collect();
            ui.monospace(format!("tile {}  vizinhos {}", view.tile, loaded.join(" ")));
            let (draws, triangles) = view.total_draws();
            ui.monospace(format!("draws {draws}  triângulos {triangles}"));
            ui.monospace(format!("personagens {}  instâncias {}", view.characters, view.instances));
            if let Some(error) = tables_error {
                ui.colored_label(Color32::LIGHT_RED, format!("tabelas: {error}"));
            }
            if let Some(error) = ui_error {
                ui.colored_label(Color32::LIGHT_RED, format!("interface: {error}"));
            }
            ui.monospace(format!("câmera {:.0} {:.0} {:.0}", camera.eye.x, camera.eye.y, camera.eye.z));
            let mode = if view.use_free { "livre (WASD, Shift, arrastar com o mouse)" } else { "órbita (arrastar com o mouse, roda)" };
            ui.monospace(format!("F alterna: {mode}"));
            ui.monospace("F12: captura em ~/Pictures/killer");
            ui.monospace(format!("pacotes ignorados: {ignored_total} (opcodes distintos: {ignored_distinct})"));
            if !ignored_top.is_empty() {
                ui.monospace(format!("mais frequentes: {}", ignored_line(ignored_top)));
            }
            if me.is_some() {
                if let Some(line) = target_line(world) {
                    ui.monospace(line);
                }
                ui.monospace("clique esquerdo no chão: andar; Enter: chat");
                ui.monospace(format!("entidades {}", world.entities.len()));
                for (entity, distance) in world.nearest(HUD_NEAREST) {
                    ui.monospace(entity_line(entity, distance));
                }
                if ui.button("Sair").clicked() {
                    leave = true;
                }
            }
        });
    });
    leave
}

pub fn ignored_line(top: &[(u8, u32)]) -> String {
    top.iter().map(|(opcode, count)| format!("{}×{count}", describe_server_opcode(*opcode))).collect::<Vec<_>>().join(" ")
}

pub fn loading(ctx: &Context, tile: &str) {
    crate::lobby::panels::loading_world(ctx, tile);
}

pub fn error(ctx: &Context, message: &str) {
    Area::new(Id::new("erro")).anchor(Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
        Frame::popup(ui.style()).show(ui, |ui| {
            ui.heading("erro ao carregar o tile");
            ui.monospace(message);
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;
    use killer_world::world::WHITE_NAME_COLOR;

    fn npc(name: &str) -> Entity {
        Entity {
            item_origin: None,
            abnormal_effect: 0,
            object_id: 1,
            kind: EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: false },
            name: name.into(),
            title: String::new(),
            position: Vec3::ZERO,
            heading: 0,
            dest: None,
            stop_heading: None,
            desired_heading: 0,
            turn_rate: None,
            run_speed: 120,
            walk_speed: 60,
            speed_multiplier: 1.0,
            attack_speed_multiplier: 1.0,
            m_atk_speed: 0,
            skill_aim: [0; 3],
            p_atk_speed: 0,
            speed_bonus: 0.0,
            speed_bonus_left: 0.0,
            blocked: false,
            follow: None,
            running: true,
            sitting: false,
            fake_dead: false,
            dead: false,
            walk_anim_frame_in_move: 12,
            hp: None,
            mp: None,
            cp: None,
            level: 0,
            exp: 0,
            collision_radius: 8.0,
            collision_height: 23.0,
            base_collision: [0.0; 2],
            karma: 0,
            pvp_flag: 0,
            relation: 0, auto_attack: false, surrender_war_id: 0,
            target: None,
            casting: None,
            magic: Default::default(),
            anim: Default::default(),
            spawned: 0,
            name_color: WHITE_NAME_COLOR,
            recommendations: 0,
            name_blink: false,
            name_blink_elapsed: 0.0,
            clan_id: 0,
            clan_crest_id: 0,
            ally_id: 0,
            ally_crest_id: 0,
            store_type: 0,
            in_combat: false,
            fishing: 0,
            mount: None,
            vehicle: None,
            chair: None,
            rotation: None,
        }
    }

    #[test]
    fn the_hud_starts_collapsed_and_h_cycles_expanded_then_hidden() {
        let mode = HudMode::default();
        assert_eq!(mode, HudMode::Collapsed);
        assert_eq!(mode.next(), HudMode::Expanded);
        assert_eq!(mode.next().next(), HudMode::Hidden);
        assert_eq!(mode.next().next().next(), HudMode::Collapsed);
    }

    #[test]
    fn the_expanded_hud_sits_below_the_status_window_in_points() {
        assert_eq!(expanded_hud_offset(1.0), [8.0, 92.0]);
        assert_eq!(expanded_hud_offset(2.0), [8.0, 50.0]);
    }

    #[test]
    fn the_hud_line_lists_the_top_opcodes_with_their_counts() {
        assert_eq!(ignored_line(&[(0x4a, 1200), (0x0e, 3)]), "Say2 (0x4a)×1200 StatusUpdate (0x0e)×3");
        assert_eq!(ignored_line(&[]), "");
    }

    #[test]
    fn entity_lines_show_name_distance_kind_and_state() {
        let mut gremlin = npc("Gremlin");
        assert_eq!(entity_line(&gremlin, 12.4), "Gremlin              12  npc     parado");
        gremlin.dest = Some(Vec3::X);
        assert!(entity_line(&gremlin, 1.0).ends_with("andando"));
        gremlin.dead = true;
        assert!(entity_line(&gremlin, 1.0).ends_with("morto"));
        let mut player = npc("");
        player.kind = EntityKind::Player { class_id: 0, race: 0, sex: 0, equipped: [0; 11], hair_style: 0, hair_color: 0, face: 0, enchant_effect: 0, hero: false };
        player.sitting = true;
        assert_eq!(entity_line(&player, 1000.0), "?                  1000  jogador sentado");
        let mut adena = npc("");
        adena.kind = EntityKind::Item { item_id: 57, count: 10, stackable: true };
        assert_eq!(entity_line(&adena, 3.0), "?                     3  item    parado");
    }

    #[test]
    fn target_line_names_the_target_or_falls_back_to_its_id() {
        let mut world = World::default();
        assert!(target_line(&world).is_none());
        world.entities.insert(1, npc("Gremlin"));
        world.target = Some(1);
        assert_eq!(target_line(&world).as_deref(), Some("alvo: Gremlin"));
        world.entities.insert(2, npc(""));
        world.entities.get_mut(&2).unwrap().object_id = 2;
        world.target = Some(2);
        assert_eq!(target_line(&world).as_deref(), Some("alvo: #2"));
        world.target = Some(3);
        assert_eq!(target_line(&world).as_deref(), Some("alvo: #3 (fora de vista)"));
    }
}
