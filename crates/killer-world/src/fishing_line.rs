use glam::Vec3;

pub const FADE_RATE: f32 = 51.0;
pub const FADE_FULL: u8 = 0xff;
pub const VIEW_RANGE: f32 = 700.0;
pub const HALF_WIDTH: f32 = 0.1;
pub const RGB: u32 = 0x00c8_c8c8;
pub const FIRST_CONTROL_SCALE: f32 = 0.666_666_7;
pub const SAG_SCALE: f32 = 0.3;
pub const NORMALIZE_FLOOR: f32 = 1.0e-8;
pub const SEGMENTS: i32 = 30;

#[derive(Debug, Clone, PartialEq)]
pub struct SpringSimul {
    pub start: Vec3,
    pub control: Vec3,
    pub end: Vec3,
    pub base_length: f32,
    pub segments: usize,
    pub points: Vec<Vec3>,
}

impl SpringSimul {
    pub fn init(start: Vec3, end: Vec3, segments: i32) -> SpringSimul {
        let segments = usize::try_from(segments).unwrap_or(0);
        let mut spring = SpringSimul { start, control: Vec3::ZERO, end, base_length: (start - end).length(), segments, points: Vec::new() };
        spring.calc_position(start, end);
        spring
    }

    pub fn calc_position(&mut self, start: Vec3, end: Vec3) {
        self.start = start;
        self.end = end;
        let difference = start - end;
        let length = difference.length();
        let sag = (end.z - start.z) * SAG_SCALE;
        let base = self.base_length;
        let mut control = Vec3::new(difference.x * FIRST_CONTROL_SCALE + end.x, difference.y * FIRST_CONTROL_SCALE + end.y, difference.z * FIRST_CONTROL_SCALE + end.z);
        let stretched = length.is_nan() || base.is_nan() || length > base;
        control.z = if stretched {
            let slack = length - base;
            (if sag <= slack { sag } else { slack }) + start.z
        } else {
            start.z
        };
        self.control = control;
        self.points = (0..=self.segments)
            .map(|index| {
                let t = index as f32 / self.segments as f32;
                let u = 1.0 - t;
                let axis = |s: f32, c: f32, e: f32| {
                    let weighted = t * c;
                    t * e * t + (u * s + weighted + weighted) * u
                };
                Vec3::new(axis(start.x, control.x, end.x), axis(start.y, control.y, end.y), axis(start.z, control.z, end.z))
            })
            .collect();
    }
}

pub fn fade_tick(alpha: u8, delta: f32) -> u8 {
    if alpha == FADE_FULL {
        return alpha;
    }
    let value = (delta * FADE_RATE + f32::from(alpha)) as i32;
    if value < 0 {
        0
    } else if value >= i32::from(FADE_FULL) {
        FADE_FULL
    } else {
        value as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RibbonVertex {
    pub position: Vec3,
    pub argb: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ribbon {
    pub vertices: Vec<RibbonVertex>,
    pub indices: Vec<u16>,
}

fn normalize(vector: Vec3) -> Vec3 {
    let square = vector.z * vector.z + vector.y * vector.y + vector.x * vector.x;
    if NORMALIZE_FLOOR <= square {
        let scale = 1.0 / f64::from(square).sqrt();
        Vec3::new((scale * f64::from(vector.x)) as f32, (scale * f64::from(vector.y)) as f32, (scale * f64::from(vector.z)) as f32)
    } else {
        vector
    }
}

pub fn ribbon(spring: &SpringSimul, camera: Vec3, alpha: u8, location: Vec3) -> Option<Ribbon> {
    if (camera - location).length() > VIEW_RANGE || spring.points.len() < 2 {
        return None;
    }
    let argb = (u32::from(alpha) << 24) | RGB;
    let mut vertices = Vec::with_capacity(spring.segments * 2 + 2);
    let count = spring.segments;
    for index in 0..count {
        let current = spring.points[index];
        let next = spring.points[index + 1];
        let to_current = current - camera;
        let step = next - current;
        let facing = Vec3::new(
            to_current.z * step.y - to_current.y * step.z,
            to_current.x * step.z - to_current.z * step.x,
            to_current.y * step.x - to_current.x * step.y,
        );
        let offset = normalize(facing) * HALF_WIDTH;
        vertices.push(RibbonVertex { position: current + offset, argb });
        vertices.push(RibbonVertex { position: current - offset, argb });
        if index == count - 1 {
            vertices.push(RibbonVertex { position: next + offset, argb });
            vertices.push(RibbonVertex { position: next - offset, argb });
        }
    }
    let indices = (0..count)
        .flat_map(|index| {
            let base = (index * 2) as u16;
            [base, base + 1, base + 2, base + 2, base + 1, base + 3]
        })
        .collect();
    Some(Ribbon { vertices, indices })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_curve_runs_from_the_first_to_the_last_end_with_one_point_more_than_segments() {
        let spring = SpringSimul::init(Vec3::new(0.0, 0.0, 100.0), Vec3::new(300.0, 0.0, 140.0), 30);
        assert_eq!(spring.points.len(), 31);
        assert_eq!(spring.points[0], Vec3::new(0.0, 0.0, 100.0));
        assert!((spring.points[30] - Vec3::new(300.0, 0.0, 140.0)).length() < 1.0e-3);
    }

    #[test]
    fn at_the_initial_length_the_control_point_stays_at_the_start_height() {
        let spring = SpringSimul::init(Vec3::new(0.0, 0.0, 10.0), Vec3::new(30.0, 40.0, 50.0), 30);
        assert!((spring.base_length - Vec3::new(30.0, 40.0, 40.0).length()).abs() < 1.0e-4);
        assert_eq!(spring.control.z, 10.0);
        let third = Vec3::new(30.0, 40.0, 50.0);
        let expected = Vec3::new(-30.0, -40.0, -40.0) * FIRST_CONTROL_SCALE + third;
        assert!((spring.control.x - expected.x).abs() < 1.0e-4 && (spring.control.y - expected.y).abs() < 1.0e-4);
    }

    #[test]
    fn stretching_lifts_the_control_point_by_the_smaller_of_the_slack_and_a_third_of_the_rise() {
        let mut spring = SpringSimul::init(Vec3::ZERO, Vec3::new(100.0, 0.0, 10.0), 30);
        let base = spring.base_length;
        spring.calc_position(Vec3::ZERO, Vec3::new(100.0, 0.0, 10.0));
        assert_eq!(spring.control.z, 0.0);
        spring.calc_position(Vec3::ZERO, Vec3::new(100.5, 0.0, 10.0));
        let slack = Vec3::new(100.5, 0.0, 10.0).length() - base;
        assert!((spring.control.z - slack.min(3.0)).abs() < 1.0e-4, "{}", spring.control.z);
        spring.calc_position(Vec3::ZERO, Vec3::new(160.0, 0.0, 10.0));
        assert!((spring.control.z - 3.0).abs() < 1.0e-4);
    }

    #[test]
    fn a_curve_matches_the_quadratic_bezier_formula_at_the_midpoint() {
        let start = Vec3::new(1.0, 2.0, 3.0);
        let end = Vec3::new(11.0, -8.0, 23.0);
        let spring = SpringSimul::init(start, end, 30);
        let control = spring.control;
        let middle = spring.points[15];
        let expected = start * 0.25 + control * 0.5 + end * 0.25;
        assert!((middle - expected).length() < 1.0e-4, "{middle} {expected}");
    }

    #[test]
    fn the_fade_rises_by_fifty_one_a_second_and_stops_at_full() {
        assert_eq!(fade_tick(0, 0.5), 25);
        assert_eq!(fade_tick(0, 1.0), 51);
        assert_eq!(fade_tick(250, 0.5), 0xff);
        assert_eq!(fade_tick(0xff, 10.0), 0xff);
        assert_eq!(fade_tick(10, -1.0), 0);
        assert_eq!(fade_tick(0, 0.0), 0);
    }

    #[test]
    fn the_ribbon_has_two_vertices_per_point_and_six_indices_per_segment_and_faces_the_camera() {
        let spring = SpringSimul::init(Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0), 30);
        let camera = Vec3::new(50.0, -200.0, 0.0);
        let ribbon = ribbon(&spring, camera, 0x80, Vec3::new(50.0, 0.0, 0.0)).unwrap();
        assert_eq!(ribbon.vertices.len(), 62);
        assert_eq!(ribbon.indices.len(), 180);
        assert_eq!(&ribbon.indices[..6], &[0, 1, 2, 2, 1, 3]);
        assert_eq!(&ribbon.indices[174..], &[58, 59, 60, 60, 59, 61]);
        assert_eq!(ribbon.vertices[0].argb, 0x80c8_c8c8);
        let width = ribbon.vertices[0].position - ribbon.vertices[1].position;
        assert!((width.length() - 2.0 * HALF_WIDTH).abs() < 1.0e-5);
        let seen = spring.points[0] - camera;
        assert!(width.dot(seen).abs() < 1.0e-3 * seen.length());
    }

    #[test]
    fn beyond_seven_hundred_units_the_line_is_not_drawn() {
        let spring = SpringSimul::init(Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0), 30);
        assert!(ribbon(&spring, Vec3::new(0.0, 700.5, 0.0), 255, Vec3::ZERO).is_none());
        assert!(ribbon(&spring, Vec3::new(0.0, 699.5, 0.0), 255, Vec3::ZERO).is_some());
    }
}
