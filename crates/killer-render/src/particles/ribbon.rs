use glam::Vec3;
use ue2_level::RibbonShape;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RibbonPoint {
    pub edges: [Vec3; 2],
    pub tangents: [Vec3; 2],
    pub width: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RibbonState {
    pub points: Vec<RibbonPoint>,
    pub last: RibbonPoint,
    pub timer: f32,
    pub sample_rate: f32,
    pub decay_rate: f32,
    pub limit: i32,
    pub killed: bool,
    pub decaying: bool,
}

impl RibbonState {
    pub fn new(shape: &RibbonShape) -> Self {
        Self {
            points: Vec::new(),
            last: RibbonPoint::default(),
            timer: shape.sample_rate,
            sample_rate: shape.sample_rate,
            decay_rate: shape.decay_rate,
            limit: shape.num_points,
            killed: false,
            decaying: false,
        }
    }

    pub fn spawn(&mut self, shape: &RibbonShape) {
        if shape.flags & 1 != 0 {
            self.points.clear();
        }
        self.sample_rate = shape.sample_rate;
        self.decay_rate = shape.decay_rate;
    }

    pub fn update(
        &mut self,
        shape: &RibbonShape,
        delta: f32,
        stopped: bool,
        age: f32,
        lifetime: f32,
        sample: RibbonPoint,
    ) {
        if shape.flags & 1 == 0 || stopped {
            if stopped && !self.killed {
                self.killed = true;
                if shape.flags & 8 != 0 {
                    self.sample_rate = (lifetime - age) / (self.points.len() as i32 - 1) as f32;
                }
            }
            if shape.flags & 4 != 0 && !self.points.is_empty() {
                self.decaying = true;
                self.timer += delta;
                if self.timer >= self.sample_rate {
                    self.timer = 0.0;
                    self.points.pop();
                }
            }
            return;
        }
        if sample.edges[0] == Vec3::ZERO {
            return;
        }
        let had_points = !self.points.is_empty();
        if !had_points {
            self.points.push(RibbonPoint::default());
        }
        self.points[0].edges = sample.edges;
        self.timer += delta;
        let mut stationary = false;
        if self.timer >= self.sample_rate {
            let distance = self.last.edges[0].distance(sample.edges[0]) + self.last.edges[1].distance(sample.edges[1]);
            if had_points && distance >= shape.min_sample_dist {
                self.decaying = false;
                self.timer = 0.0;
                self.points.insert(1, RibbonPoint { edges: sample.edges, width: sample.width, ..Default::default() });
                self.last = self.points[1];
                if shape.interpolate {
                    if self.points.len() > 3 {
                        self.points[2].tangents = std::array::from_fn(|edge| {
                            (sample.edges[edge] - self.points[3].edges[edge])
                                / (shape.scale_ratio * if edge == 0 { 1.0 } else { 2.0 })
                        });
                    }
                    if self.points.len() > 4 && shape.control_points > 0 {
                        let next = self.points[2];
                        let previous = self.points[3];
                        for i in 1..shape.control_points {
                            let time = 1.0 - f64::from(i as f32) / f64::from(shape.control_points as f32);
                            self.points.insert(
                                (i + 2) as usize,
                                RibbonPoint {
                                    edges: std::array::from_fn(|edge| {
                                        hermite(
                                            previous.edges[edge],
                                            next.edges[edge],
                                            previous.tangents[edge],
                                            next.tangents[edge],
                                            time,
                                        )
                                    }),
                                    width: sample.width,
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
            } else {
                self.timer = self.sample_rate;
                stationary = true;
            }
        }
        if shape.flags & 2 != 0 && stationary {
            self.decaying = true;
            if self.points.len() > 1 {
                self.points.pop();
            }
            self.timer = 0.0;
        }
        let count = self.points.len() as i32;
        if count >= shape.min_points + shape.drop_rate
            && ((shape.acc_drop == 1 && count >= self.limit)
                || (matches!(shape.acc_drop, 2 | 3) && age >= shape.threshold))
        {
            self.limit -= shape.drop_rate;
        } else if shape.acc_drop == 3 && age >= shape.threshold2 {
            self.limit = (self.limit - 1).max(0);
        }
        self.points.truncate(self.limit.max(0) as usize);
    }

    pub fn vertices(
        &self,
        shape: &RibbonShape,
        stopped: bool,
        color: [u8; 4],
    ) -> (Vec<killer_assets::scene::Vertex>, Vec<u32>) {
        use killer_assets::scene::Vertex;
        let count = self.points.len();
        if count < 3 {
            return (Vec::new(), Vec::new());
        }
        let fraction = self.timer.max(0.0).min(self.sample_rate) / self.sample_rate;
        let adjust = (shape.flags & 1 != 0 && count as i32 == self.limit)
            || ((shape.flags & 1 == 0 || stopped) && shape.flags & 4 != 0)
            || (shape.flags & 3 == 3 && self.decaying);
        let growing = shape.flags & 1 != 0 && !self.decaying && (count as i32) < self.limit;
        let mut vertices = Vec::with_capacity((count - 1) * 2);
        let mut indices = Vec::with_capacity((count - 2) * 6);
        let mut previous = [Vec3::ZERO; 2];
        for _ in 0..shape.sheets.max(1) {
            let start = vertices.len();
            let mut lengths = [0.0; 2];
            for (i, point) in self.points.iter().enumerate().skip(1) {
                let u = if i == count - 1 {
                    1.0
                } else if shape.flags & 16 != 0 {
                    0.0
                } else if (adjust && self.decaying) || growing {
                    let step = 1.0 / (count - 1) as f32;
                    if self.decaying {
                        ((1.0 / (count - 2) as f32 - step) * fraction + step) * i as f32
                    } else {
                        (i as f32 + fraction) * ((1.0 / count as f32 - step) * fraction + step)
                    }
                } else {
                    (i as f32 + fraction) / (count - 1) as f32
                }
                .clamp(0.0, 1.0);
                let smooth = adjust && ((i == 1 && !self.decaying) || (i == count - 1 && !growing));
                for edge in 0..2 {
                    let p = if smooth {
                        (previous[edge] - point.edges[edge]) * fraction + point.edges[edge]
                    } else {
                        point.edges[edge]
                    };
                    lengths[edge] += previous[edge].distance(p);
                    previous[edge] = p;
                    vertices.push(Vertex {
                        position: p.to_array(),
                        normal: [0.0; 3],
                        uv: [u * shape.uv_scale[0], edge as f32 * shape.uv_scale[1]],
                        mask_uv: [0.0; 2],
                        color: color.map(|v| f32::from(v) / 255.0),
                    });
                }
            }
            if shape.flags & 16 != 0 {
                let mut totals = [0.0; 2];
                for i in start + 2..vertices.len() {
                    let edge = i % 2;
                    totals[edge] +=
                        Vec3::from(vertices[i - 2].position).distance(Vec3::from(vertices[i].position)) / lengths[edge];
                    vertices[i].uv[0] = totals[edge] * shape.uv_scale[0];
                }
            }
            for i in 0..count as u32 - 2 {
                let base = start as u32 + i * 2;
                indices.extend([base, base + 1, base + 2, base + 1, base + 3, base + 2]);
            }
        }
        (vertices, indices)
    }
}

fn hermite(a: Vec3, b: Vec3, ta: Vec3, tb: Vec3, t: f64) -> Vec3 {
    let t2 = t * t;
    let t3 = t2 * t;
    a * (2.0 * t3 - 3.0 * t2 + 1.0) as f32
        + b * (3.0 * t2 - 2.0 * t3) as f32
        + ta * (t3 - 2.0 * t2 + t) as f32
        + tb * (t3 - t2) as f32
}
