pub mod model;
pub mod point_check;
pub mod static_point;
pub mod static_mesh;
pub mod terrain;
pub mod resources;
pub mod world;
pub mod cylinder;
pub mod transform;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxCheck {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub extent: [f32; 3],
    pub normal: [f32; 3],
    pub enter: f32,
    pub exit: f32,
    pub contact: bool,
}

impl BoxCheck {
    pub fn new(start: [f32; 3], end: [f32; 3], extent: [f32; 3], limit: f32) -> Self {
        Self { start, end, extent, normal: [0.0; 3], enter: -1.0, exit: limit, contact: false }
    }

    pub fn clip_plane(&mut self, plane: [f32; 4]) -> bool {
        let projected = (self.extent[2] * plane[2]).abs() as f64
            + (self.extent[1] * plane[1]).abs() as f64
            + (self.extent[0] * plane[0]).abs() as f64;
        let distance = |point: [f32; 3]| {
            (point[1] as f64 * plane[1] as f64 + point[2] as f64 * plane[2] as f64)
                + point[0] as f64 * plane[0] as f64 - plane[3] as f64
        };
        let start = distance(self.start);
        let end = distance(self.end);
        let direction = start - end;
        let end = end as f32 as f64;
        let epsilon = f32::from_bits(0x3727_c5ac) as f64;
        if direction > epsilon {
            let time = (projected - start) / (end - start);
            if (self.enter as f64) < time {
                self.enter = time as f32;
                self.normal = [plane[0], plane[1], plane[2]];
                self.contact = true;
            }
        } else if direction < -epsilon {
            let time = (projected - start) / (end - start);
            if time < self.exit as f64 { self.exit = time as f32; }
        } else if projected < start && projected < end {
            return false;
        }
        self.enter < self.exit && self.exit > 0.0
    }

    pub fn clip_edge(&mut self, point: [f32; 3], edge: [f32; 3], orientation: [f32; 3], axis: [f32; 3]) -> bool {
        let a = axis.map(f64::from);
        let e = edge.map(f64::from);
        let cross = [a[1]*e[2] - a[2]*e[1], a[2]*e[0] - a[0]*e[2], a[0]*e[1] - a[1]*e[0]].map(|v| v as f32 as f64);
        let inverse = 1.0 / ((cross[0]*cross[0] + cross[1]*cross[1]) + cross[2]*cross[2]).sqrt();
        let rounded = inverse as f32 as f64;
        let mut normal = [(cross[0]*rounded) as f32, (cross[1]*rounded) as f32, (cross[2]*inverse) as f32];
        let dot = |point: [f32; 3], normal: [f32; 3]| {
            (point[2] as f64 * normal[2] as f64 + point[1] as f64 * normal[1] as f64)
                + point[0] as f64 * normal[0] as f64
        };
        if dot(orientation, normal) < 0.0 { normal = normal.map(|v| -v); }
        let w = dot(point, normal) as f32;
        self.clip_plane([normal[0], normal[1], normal[2], w])
    }

    pub fn clip_box(&mut self, bounds: [[f32; 3]; 2]) -> bool {
        let [min, max] = bounds;
        [
            [-1.0, 0.0, 0.0, -min[0]], [1.0, 0.0, 0.0, max[0]],
            [0.0, -1.0, 0.0, -min[1]], [0.0, 1.0, 0.0, max[1]],
            [0.0, 0.0, -1.0, -min[2]], [0.0, 0.0, 1.0, max[2]],
        ].into_iter().all(|plane| self.clip_plane(plane))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extended_box_hits_before_its_center_and_parallel_outside_is_rejected() {
        let mut state = BoxCheck::new([20.0,0.0,0.0], [0.0;3], [5.0;3], 1.0);
        assert!(state.clip_box([[-10.0;3], [10.0;3]]));
        assert_eq!(state.enter.to_bits(), 0.25_f32.to_bits());
        assert!(state.contact);
        assert_eq!(state.normal.map(f32::to_bits), [1.0_f32,0.0,0.0].map(f32::to_bits));
        let mut outside = BoxCheck::new([20.0,0.0,0.0], [20.0,10.0,0.0], [5.0;3], 1.0);
        assert!(!outside.clip_box([[-10.0;3], [10.0;3]]));
    }
}
