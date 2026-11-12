use crate::animation::{find_sequence, palette, pose};
use crate::characters::{CharacterMesh, load_part};
use crate::client::Client;
use crate::pawn::Part;
use glam::{Mat4, Vec3};
use ue2_assets::MeshAnimation;

pub const MESH: &str = "LineageDecos.float_m00";
pub const CLASS: &str = "L2Float";
pub const STATE_NONE: u8 = 0;
pub const STATE_CAST: u8 = 1;
pub const STATE_COMBAT: u8 = 2;
pub const TRACE_RATE: f32 = 20.0;
pub const DIVE_SPEED: f32 = 40.0;
pub const DIVE_DECAY: f32 = 160.0;
pub const RISE_SPEED: f32 = 20.0;
pub const RISE_DECAY: f32 = 80.0;
pub const MOTION_PERIOD: f32 = 0.5;
pub const RING_SIZE_SCALE: f32 = 0.5;
pub const RING_FIRST_SPEED: f32 = 1.0;
pub const RING_NEXT_SPEED: f32 = 2.0;
pub const RING_FIRST_TIMER: f32 = 1.0;
pub const RING_NEXT_TIMER: f32 = 0.05;
pub const RING_CLASS: &str = "LineageEffect.e_u075_w";
pub const LINE_FLOAT_BONE: usize = 0;
pub const LINE_ROD_BONE: usize = 6;
pub const ANIM_RATE: f32 = 1.0;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FloatNames {
    pub wait: String,
    pub battle: String,
    pub battle_wait: [String; 6],
}

impl FloatNames {
    pub fn from_defaults(defaults: &[(String, u32, String)]) -> FloatNames {
        let mut names = FloatNames::default();
        for (name, index, value) in defaults {
            match (name.to_ascii_lowercase().as_str(), *index as usize) {
                ("waitanimname", 0) => names.wait.clone_from(value),
                ("battleanimname", 0) => names.battle.clone_from(value),
                ("battlewaitanimname", slot) if slot < 6 => names.battle_wait[slot].clone_from(value),
                _ => {}
            }
        }
        names
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FloatActor {
    pub state: u8,
    pub fish_type: i32,
    pub gut: bool,
    pub location: Vec3,
    pub base_z: f32,
    pub elapsed: f32,
    pub mode: i32,
    pub effect_timer: f32,
    pub effect_point: Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RingSpawn {
    pub location: Vec3,
    pub size_scale: f32,
    pub speed_scale: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TickOutput {
    pub play: Option<String>,
    pub ring: Option<RingSpawn>,
}

impl FloatActor {
    pub fn cast(fish_type: i32, location: Vec3) -> FloatActor {
        FloatActor { state: STATE_CAST, fish_type, location, base_z: location.z, ..Default::default() }
    }

    pub fn cur_anim_name<'a>(&self, names: &'a FloatNames) -> Option<&'a str> {
        match self.state {
            STATE_CAST => Some(&names.wait),
            STATE_COMBAT => {
                let fish = self.fish_type.rem_euclid(3) as usize;
                Some(&names.battle_wait[if self.gut { fish } else { 3 + fish }])
            }
            _ => None,
        }
    }

    fn move_z(&mut self, delta: f32) {
        self.elapsed += delta;
        let scaled = self.elapsed;
        match self.mode {
            0 => {
                let scaled = scaled * TRACE_RATE;
                let difference = self.base_z - self.location.z;
                if difference != 0.0 {
                    if difference <= 0.0 {
                        if -scaled <= difference { self.location.z = self.base_z } else { self.location.z -= scaled }
                    } else if difference <= scaled {
                        self.location.z = self.base_z;
                    } else {
                        self.location.z += scaled;
                    }
                }
            }
            1 => {
                if scaled < MOTION_PERIOD {
                    self.location.z = self.base_z - (scaled * DIVE_SPEED - scaled * scaled * DIVE_DECAY);
                } else {
                    self.elapsed = 0.0;
                    self.mode = 0;
                }
            }
            2 => {
                if scaled < MOTION_PERIOD {
                    self.location.z = (scaled * RISE_SPEED - scaled * scaled * RISE_DECAY) + self.base_z;
                } else {
                    self.elapsed = 0.0;
                    self.mode = 0;
                }
            }
            _ => {}
        }
    }

    pub fn tick(&mut self, delta: f32, names: &FloatNames, anim_finished: bool, attachment: Option<Vec3>) -> TickOutput {
        let mut out = TickOutput::default();
        if self.state == STATE_COMBAT {
            if anim_finished {
                out.play = self.cur_anim_name(names).map(str::to_owned);
            }
            self.move_z(delta);
        }
        if self.effect_timer.is_nan() || self.effect_timer > 0.0 {
            self.effect_timer -= delta;
            return out;
        }
        let Some(point) = attachment else { return out };
        if self.state == STATE_CAST {
            out.ring = Some(RingSpawn { location: point, size_scale: RING_SIZE_SCALE, speed_scale: RING_FIRST_SPEED });
            self.effect_point = point;
            self.effect_timer = RING_FIRST_TIMER;
        } else {
            out.ring = Some(RingSpawn { location: point, size_scale: RING_SIZE_SCALE, speed_scale: RING_NEXT_SPEED });
            self.effect_point = point;
            self.effect_timer = RING_NEXT_TIMER;
        }
        self.effect_timer -= delta;
        out
    }
}

pub struct FloatAsset {
    pub mesh: CharacterMesh,
    pub animation: MeshAnimation,
    pub names: FloatNames,
}

pub fn load_names(client: &mut Client) -> Option<FloatNames> {
    let file = client.read_system_file("Engine.u").ok()?;
    let package = ue2_package::Package::parse(&file).ok()?;
    let export = package.exports().iter().find(|export| package.class_name(export) == "Class" && package.name(export.name).eq_ignore_ascii_case(CLASS))?;
    let properties = ue2_assets::class_default_properties(&package, &file[..], export).ok()?;
    let defaults: Vec<(String, u32, String)> = properties
        .iter()
        .filter_map(|property| Some((property.name.to_owned(), property.index, property.name_value(&package)?.to_owned())))
        .collect();
    Some(FloatNames::from_defaults(&defaults))
}

pub fn load(client: &mut Client) -> Option<FloatAsset> {
    let mesh = load_part(client, &Part::new(MESH, Vec::new()))?;
    let animation = client.catalog().mesh_animation(mesh.animation.as_deref()?)?;
    let names = load_names(client)?;
    Some(FloatAsset { mesh, animation, names })
}

pub fn bone_position(pose: &[Mat4], world: Mat4, index: usize) -> Option<Vec3> {
    let bone = pose.get(index)?;
    Some((world * *bone).transform_point3(Vec3::ZERO))
}

pub fn float_pose(mesh: &CharacterMesh, animation: &MeshAnimation, name: &str, time: f32) -> (Vec<Mat4>, Vec<Mat4>) {
    let bind_pose = || (mesh.bind.clone(), vec![Mat4::IDENTITY; mesh.bones.len()]);
    let Some(sequence) = find_sequence(animation, name) else { return bind_pose() };
    let frames = sequence.frames.max(1) as f32;
    let frame = (time * ANIM_RATE * sequence.rate).rem_euclid(frames);
    let posed = pose(animation, sequence, &mesh.bones, frame);
    let skin = palette(&posed, &mesh.inverse_bind);
    (posed, skin)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> FloatNames {
        FloatNames {
            wait: "Wait".into(),
            battle: "battle".into(),
            battle_wait: ["battlewait01", "battlewait03", "battlewait05", "battlewait02", "battlewait04", "battlewait06"].map(String::from),
        }
    }

    #[test]
    #[ignore]
    fn the_float_of_the_client_has_every_animation_the_engine_asks_for() {
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = Client::open(&root).unwrap();
        let asset = load(&mut client).unwrap();
        assert_eq!(asset.names.wait, "Wait");
        assert_eq!(asset.names.battle, "battle");
        assert_eq!(asset.names.battle_wait[3], "battlewait02");
        for name in std::iter::once(&asset.names.wait).chain(&asset.names.battle_wait) {
            assert!(find_sequence(&asset.animation, name).is_some(), "{name}");
        }
        assert_eq!(asset.mesh.bones.len(), 2);
        let (posed, skin) = float_pose(&asset.mesh, &asset.animation, &asset.names.wait, 0.7);
        assert_eq!((posed.len(), skin.len()), (asset.mesh.bones.len(), asset.mesh.bones.len()));
        let point = bone_position(&posed, Mat4::IDENTITY, LINE_FLOAT_BONE);
        assert!(point.is_some_and(|p| p.is_finite()));
    }

    #[test]
    fn get_cur_anim_name_1031146e_picks_by_state_fish_type_and_gut() {
        let names = names();
        let mut float = FloatActor::default();
        assert_eq!(float.cur_anim_name(&names), None);
        float.state = STATE_CAST;
        assert_eq!(float.cur_anim_name(&names), Some("Wait"));
        float.state = STATE_COMBAT;
        for (fish, plain, gut) in [(0, "battlewait02", "battlewait01"), (1, "battlewait04", "battlewait03"), (2, "battlewait06", "battlewait05"), (3, "battlewait02", "battlewait01"), (-1, "battlewait06", "battlewait05")] {
            float.fish_type = fish;
            float.gut = false;
            assert_eq!(float.cur_anim_name(&names), Some(plain), "fish {fish}");
            float.gut = true;
            assert_eq!(float.cur_anim_name(&names), Some(gut), "fish {fish} gut");
        }
        float.state = 3;
        assert_eq!(float.cur_anim_name(&names), None);
    }

    #[test]
    fn tick_1030ca04_dives_with_the_quadratic_and_returns_to_the_base_height() {
        let names = names();
        let mut float = FloatActor::cast(0, Vec3::new(0.0, 0.0, 100.0));
        float.state = STATE_COMBAT;
        float.mode = 1;
        float.tick(0.1, &names, false, None);
        let expected = 100.0 - (0.1 * 40.0 - 0.1 * 0.1 * 160.0);
        assert!((float.location.z - expected).abs() < 1.0e-4, "{}", float.location.z);
        float.tick(0.5, &names, false, None);
        assert_eq!((float.mode, float.elapsed), (0, 0.0));
        float.location.z = 80.0;
        float.tick(0.5, &names, false, None);
        assert_eq!(float.location.z, 90.0);
        float.tick(0.5, &names, false, None);
        assert_eq!(float.location.z, 100.0);
    }

    #[test]
    fn tick_rises_in_mode_two_and_ignores_unknown_modes() {
        let names = names();
        let mut float = FloatActor::cast(0, Vec3::new(0.0, 0.0, 10.0));
        float.state = STATE_COMBAT;
        float.mode = 2;
        float.tick(0.25, &names, false, None);
        assert!((float.location.z - (10.0 + 0.25 * 20.0 - 0.0625 * 80.0)).abs() < 1.0e-4);
        float.mode = 7;
        float.location.z = 3.0;
        float.tick(0.25, &names, false, None);
        assert_eq!(float.location.z, 3.0);
    }

    #[test]
    fn tick_replays_the_current_animation_when_it_ends_in_combat_only() {
        let names = names();
        let mut float = FloatActor::cast(1, Vec3::ZERO);
        assert_eq!(float.tick(0.1, &names, true, None).play, None);
        float.state = STATE_COMBAT;
        assert_eq!(float.tick(0.1, &names, true, None).play.as_deref(), Some("battlewait04"));
        assert_eq!(float.tick(0.1, &names, false, None).play, None);
    }

    #[test]
    fn the_water_ring_uses_the_two_cadences_of_the_engine() {
        let names = names();
        let mut float = FloatActor::cast(0, Vec3::ZERO);
        let ring = float.tick(0.1, &names, false, Some(Vec3::new(1.0, 2.0, 3.0))).ring.unwrap();
        assert_eq!((ring.size_scale, ring.speed_scale, ring.location), (0.5, 1.0, Vec3::new(1.0, 2.0, 3.0)));
        assert!((float.effect_timer - 0.9).abs() < 1.0e-6);
        float.state = STATE_COMBAT;
        float.effect_timer = -0.001;
        let ring = float.tick(0.01, &names, false, Some(Vec3::ZERO)).ring.unwrap();
        assert_eq!(ring.speed_scale, 2.0);
        assert!((float.effect_timer - 0.04).abs() < 1.0e-6);
        float.effect_timer = 0.5;
        assert!(float.tick(0.01, &names, false, Some(Vec3::ZERO)).ring.is_none());
        assert!((float.effect_timer - 0.49).abs() < 1.0e-6);
        float.effect_timer = 0.0;
        assert!(float.tick(0.01, &names, false, None).ring.is_none());
        assert_eq!(float.effect_timer, 0.0);
    }

    #[test]
    fn get_bone_position_1030411a_takes_the_posed_bone_origin_to_the_world_and_rejects_bad_indices() {
        let pose = vec![Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0)), Mat4::from_translation(Vec3::new(0.0, 5.0, 0.0))];
        let world = Mat4::from_translation(Vec3::new(0.0, 100.0, 0.0));
        assert_eq!(bone_position(&pose, world, 0), Some(Vec3::new(10.0, 100.0, 0.0)));
        assert_eq!(bone_position(&pose, world, 1), Some(Vec3::new(0.0, 105.0, 0.0)));
        assert_eq!(bone_position(&pose, world, 2), None);
    }
}
