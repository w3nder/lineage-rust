use crate::camera::UE_UNITS_PER_TURN;
use glam::Vec3;
use killer_render::camera::Camera;
use l2_dat::logongrp::{Logongrp, STAGE_SLOTS, STAGE_TEMPLATES, Spot};
use std::collections::BTreeMap;
use std::f32::consts::TAU;
use ue2_level::{Level, Placement, Rotator, Shot};

pub const NEAR: f32 = 10.0;
pub const FAR: f32 = 60000.0;
pub const ENGINE_LOBBY_FOV_X_DEG: f32 = 50.0;
pub const ENGINE_LOBBY_START_HOUR: f32 = 12.0;
pub const ENGINE_LOBBY_TIME_RATIO: f32 = 6.0;
pub const ENGINE_STAGE_SPAWN_LIFT: f32 = 60.0;
pub const ENGINE_STAGE_SOCIAL_PERIOD: f32 = 5.0;
pub const ENGINE_STAGE_SOCIAL_CHANCE: u32 = 30;
pub const TAG_LOGON: &str = "LogOn_Warp";
pub const RACE_TAGS: [&str; 5] = ["HUMAN", "ELF", "DARKELF", "ORC", "DWARF"];
const CLASS_TAGS: [&str; 2] = ["KNIGHT", "WIZARD"];
const SEX_TAGS: [&str; 2] = ["MAN", "WOMAN"];
const CLASS_LETTERS: [&str; 2] = ["_K", "_W"];
const ROTATOR_HALF_TURN: f32 = 32768.0;

pub const LOBBY_WAIT_ANIMS: [[&str; 2]; STAGE_SLOTS] = [
    ["Social_SpWait01_MFighter", "Social_SpWait02_MFighter"],
    ["Social_SpWait01_FFighter", "Social_SpWait02_FFighter"],
    ["Social_SpWait01_MMagic", "Social_SpWait02_MMagic"],
    ["Social_SpWait01_FMagic", "Social_SpWait02_FMagic"],
    ["Social_SpWait01_MElf", "Social_SpWait02_MElf"],
    ["Social_SpWait01_FElf", "Social_SpWait02_FElf"],
    ["Social_SpWait03_MElf", "Social_SpWait04_MElf"],
    ["Social_SpWait03_FElf", "Social_SpWait04_FElf"],
    ["Social_SpWait01_MDarkElf", "Social_SpWait02_MDarkElf"],
    ["Social_SpWait01_FDarkElf", "Social_SpWait02_FDarkElf"],
    ["Social_SpWait03_MDarkElf", "Social_SpWait04_MDarkElf"],
    ["Social_SpWait03_FDarkElf", "Social_SpWait04_FDarkElf"],
    ["Social_SpWait01_MOrc", "Social_SpWait02_MOrc"],
    ["Social_SpWait01_FOrc", "Social_SpWait02_FOrc"],
    ["Social_SpWait01_MShaman", "Social_SpWait02_MShaman"],
    ["Social_SpWait01_FShaman", "Social_SpWait02_FShaman"],
    ["Social_SpWait01_MDwarf", "Social_SpWait02_MDwarf"],
    ["Social_SpWait01_FDwarf", "Social_SpWait02_FDwarf"],
];

pub fn class_collision(race: i32, sex: i32, class_id: i32) -> (f32, f32) {
    let mage = matches!((race, class_id), (0, 10..=17 | 94..=98) | (1, 25..=30 | 103..=105) | (2, 38..=43 | 110..=112) | (3, 49..=52 | 115 | 116));
    match (race, mage, sex != 0) {
        (0, false, false) => (9.0, 23.0),
        (0, false, true) => (8.0, 23.5),
        (0, true, false) => (7.5, 22.8),
        (0, true, true) => (6.5, 22.5),
        (1, false, false) => (7.5, 24.0),
        (1, false, true) => (7.5, 23.0),
        (1, true, false) => (7.5, 24.0),
        (1, true, true) => (7.0, 23.5),
        (2, false, false) => (7.5, 24.0),
        (2, false, true) => (7.0, 23.5),
        (2, true, false) => (7.5, 24.0),
        (2, true, true) => (7.0, 23.5),
        (3, false, false) => (11.0, 28.0),
        (3, false, true) => (7.0, 27.0),
        (3, true, false) => (7.0, 27.0),
        (3, true, true) => (8.0, 25.5),
        (4, _, false) => (9.0, 18.0),
        (4, _, true) => (5.0, 19.0),
        _ => (9.0, 23.0),
    }
}

pub fn rotator_direction(rotation: Rotator) -> Vec3 {
    let pitch = rotation[0] as f32 / UE_UNITS_PER_TURN * TAU;
    let yaw = rotation[1] as f32 / UE_UNITS_PER_TURN * TAU;
    Vec3::new(pitch.cos() * yaw.cos(), pitch.cos() * yaw.sin(), pitch.sin())
}

pub fn placement_camera(placement: Placement) -> Camera {
    let eye = Vec3::from(placement.location);
    Camera {
        eye,
        target: eye + rotator_direction(placement.rotation) * 1000.0,
        fov_x_deg: ENGINE_LOBBY_FOV_X_DEG,
        near: NEAR,
        far: FAR,
    }
}

pub fn normalize_delta(delta: i32) -> f32 {
    let mut delta = delta as f32;
    while delta > ROTATOR_HALF_TURN {
        delta -= UE_UNITS_PER_TURN;
    }
    while delta < -ROTATOR_HALF_TURN {
        delta += UE_UNITS_PER_TURN;
    }
    delta
}

pub fn heading_of(yaw: f32) -> i32 {
    (yaw.rem_euclid(UE_UNITS_PER_TURN)) as i32
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraMove {
    pub from: Placement,
    pub to: Placement,
    pub handles: [[f32; 3]; 2],
    pub curved: bool,
    pub seconds: f32,
    pub reverse: bool,
    pub elapsed: f32,
}

impl CameraMove {
    pub fn warp(placement: Placement) -> Self {
        Self {
            from: placement,
            to: placement,
            handles: [[0.0; 3]; 2],
            curved: false,
            seconds: 0.0,
            reverse: false,
            elapsed: 0.0,
        }
    }

    pub fn between(from: &Shot, to: &Shot, reverse: bool) -> Self {
        Self {
            from: from.placement,
            to: to.placement,
            handles: to.handles,
            curved: to.curved,
            seconds: to.seconds,
            reverse,
            elapsed: 0.0,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.elapsed = (self.elapsed + dt).min(self.seconds.max(0.0));
    }

    pub fn done(&self) -> bool {
        self.elapsed >= self.seconds
    }

    pub fn pct(&self) -> f32 {
        let pct = if self.seconds > 0.0 { (self.elapsed / self.seconds).clamp(0.0, 1.0) } else { 1.0 };
        if self.reverse { 1.0 - pct } else { pct }
    }

    pub fn placement_at(&self, pct: f32) -> Placement {
        let a = Vec3::from(self.from.location);
        let b = Vec3::from(self.to.location);
        let location = if self.curved {
            let p1 = a + Vec3::from(self.handles[0]);
            let p2 = b + Vec3::from(self.handles[1]);
            let u = 1.0 - pct;
            a * (u * u * u) + p1 * (3.0 * u * u * pct) + p2 * (3.0 * u * pct * pct) + b * (pct * pct * pct)
        } else {
            a.lerp(b, pct)
        };
        let rotation = std::array::from_fn(|axis| {
            let delta = normalize_delta(self.to.rotation[axis].wrapping_sub(self.from.rotation[axis]));
            (self.from.rotation[axis] as f32 + delta * pct).round() as i32
        });
        Placement { location: location.to_array(), rotation }
    }

    pub fn current(&self) -> Placement {
        self.placement_at(self.pct())
    }

    pub fn camera(&self) -> Camera {
        placement_camera(self.current())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SceneBook {
    shots: BTreeMap<String, Vec<Shot>>,
}

impl SceneBook {
    pub fn from_level(level: &Level) -> Self {
        Self { shots: level.shots.iter().map(|(tag, shots)| (tag.to_lowercase(), shots.clone())).collect() }
    }

    pub fn play(&self, tag: &str, reverse: bool) -> Option<CameraMove> {
        let shots = self.shots.get(&tag.to_lowercase())?;
        match shots.as_slice() {
            [] => None,
            [only] => Some(CameraMove::warp(only.placement)),
            [first, .., last] => Some(CameraMove::between(first, last, reverse)),
        }
    }

    pub fn warp_placement(&self, tag: &str) -> Option<Placement> {
        self.shots.get(&tag.to_lowercase())?.first().map(|shot| shot.placement)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CreateChoice {
    pub race: Option<usize>,
    pub mage: Option<bool>,
    pub female: Option<bool>,
}

impl CreateChoice {
    fn level(&self) -> usize {
        match (self.race, self.mage, self.female) {
            (Some(_), Some(_), Some(_)) => 3,
            (Some(_), Some(_), None) => 2,
            (Some(_), None, _) => 1,
            (None, _, _) => 0,
        }
    }

    fn sex_piece(mage: bool, female: bool) -> String {
        format!("{}{}", CLASS_LETTERS[usize::from(mage)], SEX_TAGS[usize::from(female)])
    }
}

pub fn create_transition(prev: CreateChoice, next: CreateChoice) -> Option<(String, bool)> {
    let race = RACE_TAGS.get(next.race?)?;
    let (old_level, new_level) = (prev.level(), next.level());
    if prev.race != next.race || new_level == 1 {
        return Some((race.to_string(), false));
    }
    let mage = next.mage?;
    let class = CLASS_TAGS[usize::from(mage)];
    if new_level > old_level {
        return Some(match next.female {
            None => (format!("{race}_{class}"), false),
            Some(female) => (format!("{race}_{class}{}", CreateChoice::sex_piece(mage, female)), false),
        });
    }
    if new_level < old_level {
        let female = prev.female?;
        return Some((format!("{race}_{class}{}", CreateChoice::sex_piece(mage, female)), true));
    }
    if prev.mage != next.mage {
        let other = CLASS_TAGS[usize::from(!mage)];
        return Some((format!("{race}_{other}_{class}"), false));
    }
    if prev.female != next.female {
        let female = next.female?;
        return Some((
            format!("{race}{}{}", CreateChoice::sex_piece(mage, !female), CreateChoice::sex_piece(mage, female)),
            false,
        ));
    }
    None
}

pub fn chest_tag(choice: CreateChoice) -> Option<String> {
    let race = RACE_TAGS.get(choice.race?)?;
    Some(format!("{race}{}_CHEST", CreateChoice::sex_piece(choice.mage?, choice.female?)))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StageModel {
    pub index: usize,
    pub race: i32,
    pub sex: i32,
    pub class_id: i32,
    pub spot: Spot,
    pub wait_anim: &'static str,
    pub social_anim: &'static str,
}

pub fn stage_models(table: &Logongrp) -> Vec<StageModel> {
    STAGE_TEMPLATES
        .iter()
        .enumerate()
        .map(|(index, (race, sex, class_id))| StageModel {
            index,
            race: *race,
            sex: *sex,
            class_id: *class_id,
            spot: table.stage[index],
            wait_anim: LOBBY_WAIT_ANIMS[index][0],
            social_anim: LOBBY_WAIT_ANIMS[index][1],
        })
        .collect()
}

pub fn stage_visible(model: &StageModel, choice: CreateChoice) -> bool {
    match (choice.race, choice.mage, choice.female) {
        (Some(race), Some(mage), Some(female)) => model.index == (race * 2 + usize::from(mage)) * 2 + usize::from(female),
        _ => true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LobbyPose {
    Standing,
    Sitting,
    Dead,
}

pub fn lobby_pose(delete_days: i32) -> LobbyPose {
    if delete_days < 0 {
        LobbyPose::Dead
    } else if delete_days > 0 {
        LobbyPose::Sitting
    } else {
        LobbyPose::Standing
    }
}

pub fn spot_position(spot: Spot) -> Vec3 {
    Vec3::new(spot.x, spot.y, spot.z)
}

pub fn lobby_hour(elapsed: f32) -> f32 {
    (ENGINE_LOBBY_START_HOUR + elapsed * ENGINE_LOBBY_TIME_RATIO / 3600.0).rem_euclid(24.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shot(location: [f32; 3], rotation: Rotator, seconds: f32, curved: bool, handles: [[f32; 3]; 2]) -> Shot {
        Shot { placement: Placement { location, rotation }, seconds, curved, handles }
    }

    #[test]
    fn a_rotator_looks_along_its_yaw_and_pitch() {
        let east = rotator_direction([0, 0, 0]);
        assert!((east - Vec3::X).length() < 1e-5);
        let north = rotator_direction([0, 16384, 0]);
        assert!((north - Vec3::Y).length() < 1e-5);
        let up = rotator_direction([16384, 0, 0]);
        assert!((up - Vec3::Z).length() < 1e-5);
        let logon =
            placement_camera(Placement { location: [-9051.31, 241254.9, -6419.37], rotation: [2792, -25224, 0] });
        assert_eq!(logon.fov_x_deg, 50.0);
        let dir = (logon.target - logon.eye).normalize();
        assert!(dir.x < -0.7 && dir.y < -0.6 && dir.z > 0.2, "{dir:?}");
    }

    #[test]
    fn a_warp_is_instant_and_a_move_follows_the_bezier_and_interpolates_the_rotation() {
        let mut level = Level::default();
        level
            .shots
            .insert("Char_Select_Warp".into(), vec![shot([1.0, 2.0, 3.0], [0, 100, 0], 0.0, false, [[0.0; 3]; 2])]);
        level.shots.insert(
            "Human_Knight".into(),
            vec![
                shot([0.0, 0.0, 0.0], [0, 32000, 0], 0.0, false, [[-17.9, 19.7, 0.0], [17.9, -19.7, 0.0]]),
                shot([100.0, 0.0, 0.0], [1000, -32000, 0], 2.0, true, [[0.0, 50.0, 0.0], [0.0, 50.0, 0.0]]),
            ],
        );
        let book = SceneBook::from_level(&level);
        assert!(book.play("Human", false).is_none());
        let warp = book.play("CHAR_SELECT_WARP", false).unwrap();
        assert!(warp.done());
        assert_eq!(warp.current(), Placement { location: [1.0, 2.0, 3.0], rotation: [0, 100, 0] });
        let mut walk = book.play("Human_Knight", false).unwrap();
        assert_eq!(walk.current(), Placement { location: [0.0, 0.0, 0.0], rotation: [0, 32000, 0] });
        walk.tick(1.0);
        let half = walk.current();
        assert!((half.location[0] - 50.0).abs() < 1e-3 && half.location[1] > 30.0, "{half:?}");
        assert_eq!(half.rotation, [500, 32768, 0]);
        walk.tick(5.0);
        assert!(walk.done());
        let end = walk.current();
        assert_eq!(end.location, [100.0, 0.0, 0.0]);
        assert_eq!(end.rotation.map(|r| r.rem_euclid(65536)), [1000, 33536, 0]);
        let mut back = book.play("Human_Knight", true).unwrap();
        assert_eq!(back.current().location, [100.0, 0.0, 0.0]);
        back.tick(2.0);
        assert!(back.done());
        assert_eq!(back.current().location, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn the_create_panel_composes_the_scene_tags_of_the_pawn_setup_window() {
        let none = CreateChoice::default();
        let human = CreateChoice { race: Some(0), ..none };
        let knight = CreateChoice { mage: Some(false), ..human };
        let wizard = CreateChoice { mage: Some(true), ..human };
        let kman = CreateChoice { female: Some(false), ..knight };
        let kwoman = CreateChoice { female: Some(true), ..knight };
        assert_eq!(create_transition(none, human), Some(("HUMAN".into(), false)));
        assert_eq!(create_transition(human, knight), Some(("HUMAN_KNIGHT".into(), false)));
        assert_eq!(create_transition(knight, wizard), Some(("HUMAN_KNIGHT_WIZARD".into(), false)));
        assert_eq!(create_transition(knight, kman), Some(("HUMAN_KNIGHT_KMAN".into(), false)));
        assert_eq!(create_transition(kman, kwoman), Some(("HUMAN_KMAN_KWOMAN".into(), false)));
        assert_eq!(create_transition(kwoman, knight), Some(("HUMAN_KNIGHT_KWOMAN".into(), true)));
        assert_eq!(
            create_transition(kwoman, CreateChoice { race: Some(2), ..kwoman }),
            Some(("DARKELF".into(), false))
        );
        assert_eq!(
            create_transition(human, CreateChoice { mage: Some(true), female: Some(true), ..human }),
            Some(("HUMAN_WIZARD_WWOMAN".into(), false))
        );
        assert_eq!(create_transition(kman, kman), None);
        assert_eq!(create_transition(none, none), None);
        assert_eq!(chest_tag(kwoman), Some("HUMAN_KWOMAN_CHEST".into()));
        assert_eq!(chest_tag(knight), None);
    }

    #[test]
    fn the_chest_tag_names_the_race_the_class_letter_and_the_sex() {
        let kwoman = CreateChoice { race: Some(0), mage: Some(false), female: Some(true) };
        assert_eq!(chest_tag(kwoman), Some("HUMAN_KWOMAN_CHEST".into()));
        assert_eq!(chest_tag(CreateChoice { female: None, ..kwoman }), None);
    }

    #[test]
    fn stage_models_follow_the_engine_table_and_hide_by_choice() {
        let table = Logongrp {
            slots: [Spot::default(); 7],
            center: Spot::default(),
            stage: std::array::from_fn(|i| Spot { x: i as f32, ..Spot::default() }),
        };
        let models = stage_models(&table);
        assert_eq!(models.len(), 18);
        assert_eq!((models[3].race, models[3].sex, models[3].class_id, models[3].spot.x), (0, 1, 10, 3.0));
        assert_eq!(models[3].wait_anim, "Social_SpWait01_FMagic");
        assert_eq!(models[7].social_anim, "Social_SpWait04_FElf");
        let elf = CreateChoice { race: Some(1), ..CreateChoice::default() };
        assert!(models.iter().all(|m| stage_visible(m, elf)));
        assert!(models.iter().all(|m| stage_visible(m, CreateChoice::default())));
        let elf_mage_f = CreateChoice { mage: Some(true), female: Some(true), ..elf };
        let shown: Vec<usize> = models.iter().filter(|m| stage_visible(m, elf_mage_f)).map(|m| m.index).collect();
        assert_eq!(shown, [7]);
        let dwarf_f = CreateChoice { race: Some(4), mage: Some(false), female: Some(true) };
        let shown: Vec<usize> = models.iter().filter(|m| stage_visible(m, dwarf_f)).map(|m| m.index).collect();
        assert_eq!(shown, [17]);
    }

    #[test]
    fn poses_and_collision_follow_the_engine() {
        assert_eq!(lobby_pose(0), LobbyPose::Standing);
        assert_eq!(lobby_pose(3), LobbyPose::Sitting);
        assert_eq!(lobby_pose(-1), LobbyPose::Dead);
        assert_eq!(class_collision(0, 0, 0), (9.0, 23.0));
        assert_eq!(class_collision(0, 1, 97), (6.5, 22.5));
        assert_eq!(class_collision(3, 0, 49), (7.0, 27.0));
        assert_eq!(class_collision(4, 1, 57), (5.0, 19.0));
        assert_eq!(heading_of(-16384.0), 49152);
        assert_eq!(heading_of(47848.0), 47848);
        assert_eq!(normalize_delta(-64000), 1536.0);
        assert!((lobby_hour(600.0) - 13.0).abs() < 1e-5);
        assert!((lobby_hour(7200.0) - 0.0).abs() < 1e-5);
    }
}
