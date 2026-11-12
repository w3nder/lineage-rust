use glam::Vec3;
use killer_assets::scene::transform::vector_rotation;

pub const ARROW_CLASS: &str = "LineageEffect.e_u088_a";
pub const ARRIVAL_CLASS: &str = "LineageEffect.e_u088_b";
pub const ARRIVE_DISTANCE: f32 = 128.0;
pub const HEIGHT_CUT: f32 = 0.25;
pub const ARROW_CHECK: u32 = 0xf0f0_1476;
pub const ARRIVAL_PHYSICS: u8 = 10;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Frame {
    Idle,
    Spawn { location: Vec3 },
    Place { location: Vec3, rotation: [i32; 3], arrived: bool },
    Despawn { burst: Vec3, pre_pivot: Vec3 },
}

pub fn lift(height: f32) -> f32 {
    let height = f64::from(height);
    (height - height * f64::from(HEIGHT_CUT)) as f32
}

fn lifted(location: Vec3, height: f32) -> Vec3 {
    let height = f64::from(height);
    let z = height - height * f64::from(HEIGHT_CUT) + f64::from(location.z);
    Vec3::new(location.x, location.y, z as f32)
}

pub fn frame(active: bool, spawned: bool, pawn: Vec3, height: f32, target: Vec3) -> Frame {
    match (active, spawned) {
        (false, false) => Frame::Idle,
        (false, true) => Frame::Despawn { burst: lifted(pawn, height), pre_pivot: Vec3::new(0.0, 0.0, lift(height)) },
        (true, false) => Frame::Spawn { location: pawn },
        (true, true) => {
            let delta = target - pawn;
            let yaw = vector_rotation(delta)[1];
            Frame::Place { location: lifted(pawn, height), rotation: [0, yaw, 0], arrived: delta.length() <= ARRIVE_DISTANCE }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_quest_arrow_105191e0_spawns_first_and_draws_from_the_next_frame() {
        let pawn = Vec3::new(1000.0, 2000.0, -300.0);
        assert_eq!(frame(false, false, pawn, 40.0, Vec3::ZERO), Frame::Idle);
        assert_eq!(frame(true, false, pawn, 40.0, Vec3::ZERO), Frame::Spawn { location: pawn });
    }

    #[test]
    fn the_arrow_sits_three_quarters_up_and_turns_only_in_yaw() {
        let pawn = Vec3::new(1000.0, 2000.0, -300.0);
        let target = Vec3::new(1000.0, 2500.0, 900.0);
        let Frame::Place { location, rotation, arrived } = frame(true, true, pawn, 40.0, target) else { panic!("Place") };
        assert_eq!(location, Vec3::new(1000.0, 2000.0, -270.0));
        assert_eq!(rotation, [0, 0x3fff, 0]);
        assert!(!arrived);
    }

    #[test]
    fn it_arrives_at_128_units_counting_height() {
        let pawn = Vec3::new(0.0, 0.0, 0.0);
        let at = |target: Vec3| matches!(frame(true, true, pawn, 30.0, target), Frame::Place { arrived: true, .. });
        assert!(at(Vec3::new(128.0, 0.0, 0.0)));
        assert!(!at(Vec3::new(128.5, 0.0, 0.0)));
        assert!(!at(Vec3::new(100.0, 0.0, 90.0)));
        assert!(at(Vec3::new(-60.0, 60.0, 60.0)));
    }

    #[test]
    fn switching_off_swaps_the_arrow_for_the_trailer_burst() {
        let pawn = Vec3::new(5.0, 6.0, 7.0);
        assert_eq!(
            frame(false, true, pawn, 44.0, Vec3::ZERO),
            Frame::Despawn { burst: Vec3::new(5.0, 6.0, 40.0), pre_pivot: Vec3::new(0.0, 0.0, 33.0) }
        );
    }
}
