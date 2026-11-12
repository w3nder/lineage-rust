use glam::Vec3;
use killer_assets::collision::cylinder;
use killer_world::walk::{AI_CONTROLLER_MIN_HIT_WALL, CONTROLLER_MIN_HIT_WALL, Controller, ControllerCall, HitActor, Outcome, Sweep, SweepHit, Walker, phys_walking};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

const RADIUS: f32 = 7.5;
const HEIGHT: f32 = 25.0;
const SPEED: f32 = 120.0;
const FRAMES: usize = 120;
const DT: f32 = 1.0 / 60.0;
const PAWN_Z: f32 = 27.15;
const TOLERANCE: f32 = 5e-3;

struct Field {
    cylinders: Vec<Vec3>,
    blocks: bool,
    cannot_move: usize,
}

impl Sweep for Field {
    fn sweep(&mut self, start: Vec3, end: Vec3, extent: Vec3, _flags: u32) -> Option<SweepHit> {
        let bottom_start = start.z - extent.z;
        let bottom_end = end.z - extent.z;
        let mut best = None;
        if bottom_start >= 0.0 && bottom_end < 0.0 {
            best = Some(SweepHit::world(bottom_start / (bottom_start - bottom_end), Vec3::Z));
        }
        if self.blocks {
            for center in &self.cylinders {
                if let Some(hit) = cylinder::trace(center.to_array(), RADIUS, HEIGHT, start.to_array(), end.to_array(), extent.to_array())
                    && best.is_none_or(|current| hit.time < current.time)
                {
                    best = Some(SweepHit { time: hit.time, normal: Vec3::from(hit.normal), actor: HitActor::Pawn });
                }
            }
        }
        best
    }

    fn controller_call(&mut self, call: ControllerCall) {
        if matches!(call, ControllerCall::CannotMoveAnymore { from_hit_wall: false, .. }) {
            self.cannot_move += 1;
        }
    }
}

fn walk(cylinders: &[[f32; 3]], heading: f32, blocks: bool, player: bool) -> (Vec3, [usize; 3]) {
    let direction = Vec3::new(heading.to_radians().cos(), heading.to_radians().sin(), 0.0);
    let mut field = Field { cylinders: cylinders.iter().map(|center| Vec3::from(*center)).collect(), blocks, cannot_move: 0 };
    let mut walker = Walker::new(Vec3::new(0.0, 0.0, PAWN_Z), RADIUS, HEIGHT);
    walker.controller = if player { Controller::Player } else { Controller::Ai };
    walker.my_controller = player;
    walker.human = player;
    walker.min_hit_wall = if player { CONTROLLER_MIN_HIT_WALL } else { AI_CONTROLLER_MIN_HIT_WALL };
    walker.ground_speed = SPEED;
    walker.accel_rate = 2048.0;
    let (mut falling, mut blocked) = (0, 0);
    for _ in 0..FRAMES {
        walker.velocity = direction * SPEED;
        walker.acceleration = direction;
        walker.destination = Some(walker.location + direction * 500.0);
        walker.moving = player;
        match phys_walking(&mut field, &mut walker, DT) {
            Outcome::Falling { .. } => {
                falling += 1;
                break;
            }
            Outcome::Blocked => {
                blocked += 1;
                break;
            }
            _ => {}
        }
    }
    (walker.location, [field.cannot_move, falling, blocked])
}

fn check(mover_ignores: bool, others_ignorable: bool, player: bool, heading: f32, cylinders: &[[f32; 3]], expected: [f32; 3], counts: [usize; 3]) {
    let blocks = killer_world::walk::pawn_blocks_mover(mover_ignores, others_ignorable);
    let (end, found) = walk(cylinders, heading, blocks, player);
    let case = format!("mover_ignores={mover_ignores} others_ignorable={others_ignorable} player={player} heading={heading} cylinders={cylinders:?}");
    assert_eq!(found, counts, "{case}: [CannotMoveAnymore, Falling, Blocked] do porte contra o original");
    for axis in 0..3 {
        assert!((end[axis] - expected[axis]).abs() <= TOLERANCE, "{case} axis={axis}: porte {end:?} original {expected:?}");
    }
}

type Row = (bool, bool, bool, f32, &'static [[f32; 3]], [f32; 3], [usize; 3]);

const ORIGINAL: &[Row] = &[
    (false, false, false, 0.0, &[[1.0, 0.0, 27.15]], [0.000, 0.000, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[1.0, 0.3, 27.15]], [237.184, -1.550, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[1.0, 1.0, 27.15]], [239.000, -1.000, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[10.0, 0.0, 27.15]], [0.000, 0.000, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[10.0, 2.5, 27.15]], [226.389, -8.764, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[10.0, 10.0, 27.15]], [236.893, -4.866, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[15.4, 0.0, 27.15]], [0.000, 0.000, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[15.4, 4.0, 27.15]], [223.173, -11.041, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[30.0, 0.0, 27.15]], [12.997, 0.000, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[30.0, 5.0, 27.15]], [226.628, -10.016, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[20.0, 20.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[0.0, 10.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[14.0, 4.0, 27.15], [14.0, -4.0, 27.15]], [0.000, 0.000, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[14.0, 10.0, 27.15], [14.0, -10.0, 27.15]], [0.816, 0.000, 27.000], [0, 0, 0]),
    (false, false, false, 20.0, &[[30.0, 0.0, 27.15]], [220.778, 85.404, 27.000], [0, 0, 0]),
    (false, false, false, -35.0, &[[20.0, -10.0, 27.15], [40.0, -20.0, 27.15]], [173.588, -135.912, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[20.0, 0.0, 67.15]], [2.997, 0.000, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[20.0, 0.0, -12.85]], [2.997, 0.000, 27.000], [0, 0, 0]),
    (false, false, false, 0.0, &[[20.0, 5.0, 27.15], [25.0, -5.0, 27.15], [22.0, 0.0, 27.15]], [205.668, -20.019, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[1.0, 0.0, 27.15]], [0.000, 0.000, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[1.0, 0.3, 27.15]], [237.184, -1.550, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[1.0, 1.0, 27.15]], [239.000, -1.000, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[10.0, 0.0, 27.15]], [0.000, 0.000, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[10.0, 2.5, 27.15]], [226.389, -8.764, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[10.0, 10.0, 27.15]], [236.893, -4.866, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[15.4, 0.0, 27.15]], [0.000, 0.000, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[15.4, 4.0, 27.15]], [223.173, -11.041, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[30.0, 0.0, 27.15]], [12.997, 0.000, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[30.0, 5.0, 27.15]], [226.628, -10.016, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[20.0, 20.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[0.0, 10.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[14.0, 4.0, 27.15], [14.0, -4.0, 27.15]], [0.000, 0.000, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[14.0, 10.0, 27.15], [14.0, -10.0, 27.15]], [0.816, 0.000, 27.000], [0, 0, 0]),
    (false, true, false, 20.0, &[[30.0, 0.0, 27.15]], [220.778, 85.404, 27.000], [0, 0, 0]),
    (false, true, false, -35.0, &[[20.0, -10.0, 27.15], [40.0, -20.0, 27.15]], [173.588, -135.912, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[20.0, 0.0, 67.15]], [2.997, 0.000, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[20.0, 0.0, -12.85]], [2.997, 0.000, 27.000], [0, 0, 0]),
    (false, true, false, 0.0, &[[20.0, 5.0, 27.15], [25.0, -5.0, 27.15], [22.0, 0.0, 27.15]], [205.668, -20.019, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[1.0, 0.0, 27.15]], [0.000, 0.000, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[1.0, 0.3, 27.15]], [237.184, -1.550, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[1.0, 1.0, 27.15]], [239.000, -1.000, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[10.0, 0.0, 27.15]], [0.000, 0.000, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[10.0, 2.5, 27.15]], [226.389, -8.764, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[10.0, 10.0, 27.15]], [236.893, -4.866, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[15.4, 0.0, 27.15]], [0.000, 0.000, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[15.4, 4.0, 27.15]], [223.173, -11.041, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[30.0, 0.0, 27.15]], [12.997, 0.000, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[30.0, 5.0, 27.15]], [226.628, -10.016, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[20.0, 20.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[0.0, 10.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[14.0, 4.0, 27.15], [14.0, -4.0, 27.15]], [0.000, 0.000, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[14.0, 10.0, 27.15], [14.0, -10.0, 27.15]], [0.816, 0.000, 27.000], [0, 0, 0]),
    (true, false, false, 20.0, &[[30.0, 0.0, 27.15]], [220.778, 85.404, 27.000], [0, 0, 0]),
    (true, false, false, -35.0, &[[20.0, -10.0, 27.15], [40.0, -20.0, 27.15]], [173.588, -135.912, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[20.0, 0.0, 67.15]], [2.997, 0.000, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[20.0, 0.0, -12.85]], [2.997, 0.000, 27.000], [0, 0, 0]),
    (true, false, false, 0.0, &[[20.0, 5.0, 27.15], [25.0, -5.0, 27.15], [22.0, 0.0, 27.15]], [205.668, -20.019, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[1.0, 0.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[1.0, 0.3, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[1.0, 1.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[10.0, 0.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[10.0, 2.5, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[10.0, 10.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[15.4, 0.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[15.4, 4.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[30.0, 0.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[30.0, 5.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[20.0, 20.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[0.0, 10.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[14.0, 4.0, 27.15], [14.0, -4.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[14.0, 10.0, 27.15], [14.0, -10.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 20.0, &[[30.0, 0.0, 27.15]], [225.526, 82.085, 27.000], [0, 0, 0]),
    (true, true, false, -35.0, &[[20.0, -10.0, 27.15], [40.0, -20.0, 27.15]], [196.597, -137.659, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[20.0, 0.0, 67.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[20.0, 0.0, -12.85]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, false, 0.0, &[[20.0, 5.0, 27.15], [25.0, -5.0, 27.15], [22.0, 0.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (false, false, true, 0.0, &[[1.0, 0.0, 27.15]], [0.000, 0.000, 37.150], [1, 1, 0]),
    (false, false, true, 0.0, &[[1.0, 0.3, 27.15]], [237.184, -1.550, 27.000], [0, 0, 0]),
    (false, false, true, 0.0, &[[1.0, 1.0, 27.15]], [239.000, -1.000, 27.000], [0, 0, 0]),
    (false, false, true, 0.0, &[[10.0, 0.0, 27.15]], [0.000, 0.000, 37.150], [1, 1, 0]),
    (false, false, true, 0.0, &[[10.0, 2.5, 27.15]], [226.389, -8.764, 27.000], [0, 0, 0]),
    (false, false, true, 0.0, &[[10.0, 10.0, 27.15]], [236.893, -4.866, 27.000], [0, 0, 0]),
    (false, false, true, 0.0, &[[15.4, 0.0, 27.15]], [0.000, 0.000, 37.150], [1, 1, 0]),
    (false, false, true, 0.0, &[[15.4, 4.0, 27.15]], [223.173, -11.041, 27.000], [0, 0, 0]),
    (false, false, true, 0.0, &[[30.0, 0.0, 27.15]], [12.997, 0.000, 37.000], [1, 1, 0]),
    (false, false, true, 0.0, &[[30.0, 5.0, 27.15]], [226.628, -10.016, 27.000], [0, 0, 0]),
    (false, false, true, 0.0, &[[20.0, 20.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (false, false, true, 0.0, &[[0.0, 10.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (false, false, true, 0.0, &[[14.0, 4.0, 27.15], [14.0, -4.0, 27.15]], [0.000, 0.000, 37.150], [1, 1, 0]),
    (false, false, true, 0.0, &[[14.0, 10.0, 27.15], [14.0, -10.0, 27.15]], [0.816, 0.000, 37.150], [1, 1, 0]),
    (false, false, true, 20.0, &[[30.0, 0.0, 27.15]], [220.778, 85.404, 27.000], [0, 0, 0]),
    (false, false, true, -35.0, &[[20.0, -10.0, 27.15], [40.0, -20.0, 27.15]], [173.588, -135.912, 27.000], [0, 0, 0]),
    (false, false, true, 0.0, &[[20.0, 0.0, 67.15]], [2.997, 0.000, 37.000], [1, 1, 0]),
    (false, false, true, 0.0, &[[20.0, 0.0, -12.85]], [2.997, 0.000, 37.000], [1, 1, 0]),
    (false, false, true, 0.0, &[[20.0, 5.0, 27.15], [25.0, -5.0, 27.15], [22.0, 0.0, 27.15]], [205.668, -20.019, 27.000], [0, 0, 0]),
    (false, true, true, 0.0, &[[1.0, 0.0, 27.15]], [0.000, 0.000, 37.150], [1, 1, 0]),
    (false, true, true, 0.0, &[[1.0, 0.3, 27.15]], [237.184, -1.550, 27.000], [0, 0, 0]),
    (false, true, true, 0.0, &[[1.0, 1.0, 27.15]], [239.000, -1.000, 27.000], [0, 0, 0]),
    (false, true, true, 0.0, &[[10.0, 0.0, 27.15]], [0.000, 0.000, 37.150], [1, 1, 0]),
    (false, true, true, 0.0, &[[10.0, 2.5, 27.15]], [226.389, -8.764, 27.000], [0, 0, 0]),
    (false, true, true, 0.0, &[[10.0, 10.0, 27.15]], [236.893, -4.866, 27.000], [0, 0, 0]),
    (false, true, true, 0.0, &[[15.4, 0.0, 27.15]], [0.000, 0.000, 37.150], [1, 1, 0]),
    (false, true, true, 0.0, &[[15.4, 4.0, 27.15]], [223.173, -11.041, 27.000], [0, 0, 0]),
    (false, true, true, 0.0, &[[30.0, 0.0, 27.15]], [12.997, 0.000, 37.000], [1, 1, 0]),
    (false, true, true, 0.0, &[[30.0, 5.0, 27.15]], [226.628, -10.016, 27.000], [0, 0, 0]),
    (false, true, true, 0.0, &[[20.0, 20.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (false, true, true, 0.0, &[[0.0, 10.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (false, true, true, 0.0, &[[14.0, 4.0, 27.15], [14.0, -4.0, 27.15]], [0.000, 0.000, 37.150], [1, 1, 0]),
    (false, true, true, 0.0, &[[14.0, 10.0, 27.15], [14.0, -10.0, 27.15]], [0.816, 0.000, 37.150], [1, 1, 0]),
    (false, true, true, 20.0, &[[30.0, 0.0, 27.15]], [220.778, 85.404, 27.000], [0, 0, 0]),
    (false, true, true, -35.0, &[[20.0, -10.0, 27.15], [40.0, -20.0, 27.15]], [173.588, -135.912, 27.000], [0, 0, 0]),
    (false, true, true, 0.0, &[[20.0, 0.0, 67.15]], [2.997, 0.000, 37.000], [1, 1, 0]),
    (false, true, true, 0.0, &[[20.0, 0.0, -12.85]], [2.997, 0.000, 37.000], [1, 1, 0]),
    (false, true, true, 0.0, &[[20.0, 5.0, 27.15], [25.0, -5.0, 27.15], [22.0, 0.0, 27.15]], [205.668, -20.019, 27.000], [0, 0, 0]),
    (true, false, true, 0.0, &[[1.0, 0.0, 27.15]], [0.000, 0.000, 37.150], [1, 1, 0]),
    (true, false, true, 0.0, &[[1.0, 0.3, 27.15]], [237.184, -1.550, 27.000], [0, 0, 0]),
    (true, false, true, 0.0, &[[1.0, 1.0, 27.15]], [239.000, -1.000, 27.000], [0, 0, 0]),
    (true, false, true, 0.0, &[[10.0, 0.0, 27.15]], [0.000, 0.000, 37.150], [1, 1, 0]),
    (true, false, true, 0.0, &[[10.0, 2.5, 27.15]], [226.389, -8.764, 27.000], [0, 0, 0]),
    (true, false, true, 0.0, &[[10.0, 10.0, 27.15]], [236.893, -4.866, 27.000], [0, 0, 0]),
    (true, false, true, 0.0, &[[15.4, 0.0, 27.15]], [0.000, 0.000, 37.150], [1, 1, 0]),
    (true, false, true, 0.0, &[[15.4, 4.0, 27.15]], [223.173, -11.041, 27.000], [0, 0, 0]),
    (true, false, true, 0.0, &[[30.0, 0.0, 27.15]], [12.997, 0.000, 37.000], [1, 1, 0]),
    (true, false, true, 0.0, &[[30.0, 5.0, 27.15]], [226.628, -10.016, 27.000], [0, 0, 0]),
    (true, false, true, 0.0, &[[20.0, 20.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, false, true, 0.0, &[[0.0, 10.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, false, true, 0.0, &[[14.0, 4.0, 27.15], [14.0, -4.0, 27.15]], [0.000, 0.000, 37.150], [1, 1, 0]),
    (true, false, true, 0.0, &[[14.0, 10.0, 27.15], [14.0, -10.0, 27.15]], [0.816, 0.000, 37.150], [1, 1, 0]),
    (true, false, true, 20.0, &[[30.0, 0.0, 27.15]], [220.778, 85.404, 27.000], [0, 0, 0]),
    (true, false, true, -35.0, &[[20.0, -10.0, 27.15], [40.0, -20.0, 27.15]], [173.588, -135.912, 27.000], [0, 0, 0]),
    (true, false, true, 0.0, &[[20.0, 0.0, 67.15]], [2.997, 0.000, 37.000], [1, 1, 0]),
    (true, false, true, 0.0, &[[20.0, 0.0, -12.85]], [2.997, 0.000, 37.000], [1, 1, 0]),
    (true, false, true, 0.0, &[[20.0, 5.0, 27.15], [25.0, -5.0, 27.15], [22.0, 0.0, 27.15]], [205.668, -20.019, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[1.0, 0.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[1.0, 0.3, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[1.0, 1.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[10.0, 0.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[10.0, 2.5, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[10.0, 10.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[15.4, 0.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[15.4, 4.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[30.0, 0.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[30.0, 5.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[20.0, 20.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[0.0, 10.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[14.0, 4.0, 27.15], [14.0, -4.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[14.0, 10.0, 27.15], [14.0, -10.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 20.0, &[[30.0, 0.0, 27.15]], [225.526, 82.085, 27.000], [0, 0, 0]),
    (true, true, true, -35.0, &[[20.0, -10.0, 27.15], [40.0, -20.0, 27.15]], [196.597, -137.659, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[20.0, 0.0, 67.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[20.0, 0.0, -12.85]], [240.000, 0.000, 27.000], [0, 0, 0]),
    (true, true, true, 0.0, &[[20.0, 5.0, 27.15], [25.0, -5.0, 27.15], [22.0, 0.0, 27.15]], [240.000, 0.000, 27.000], [0, 0, 0]),
];

#[test]
fn walking_pawn_against_a_pawn_cylinder_matches_the_recorded_original_move_actor() {
    for &(mover_ignores, others_ignorable, player, heading, cylinders, expected, counts) in ORIGINAL {
        check(mover_ignores, others_ignorable, player, heading, cylinders, expected, counts);
    }
}

#[test]
#[ignore]
fn walking_pawn_against_a_pawn_cylinder_matches_the_native_move_actor() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/phys_walking_reference.py");
    let output = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script)
        .arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .arg("--json")
        .output()
        .unwrap();
    assert!(output.status.success());
    let cases: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(cases.len(), ORIGINAL.len());
    for case in &cases {
        let end: [f32; 3] = serde_json::from_value(case["end"].clone()).unwrap();
        let cylinders: Vec<[f32; 3]> = serde_json::from_value(case["cylinders"].clone()).unwrap();
        check(
            case["mover_ignores"].as_bool().unwrap(),
            case["others_ignorable"].as_bool().unwrap(),
            case["player"].as_bool().unwrap(),
            case["direction"].as_f64().unwrap() as f32,
            &cylinders,
            end,
            [case["notifies"].as_u64().unwrap() as usize, case["falling"].as_u64().unwrap() as usize, case["blocked"].as_u64().unwrap() as usize],
        );
    }
}
