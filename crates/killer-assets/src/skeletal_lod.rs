use ue2_assets::{SkeletalLodInfo, SkeletalLodSettings};

pub fn half_fov_tangent(fov: f32) -> f32 {
    (f64::from(fov) * f64::from_bits(0x3f81_df46_a252_9d39)).tan() as f32
}

#[derive(Debug, Clone, Copy)]
pub struct LodRenderInput {
    pub world_to_screen: [[f32; 4]; 4],
    pub mesh_to_world: [[f32; 4]; 4],
    pub location: [f32; 3],
    pub soft_root: Option<[f32; 3]>,
    pub viewer_pawn: Option<[f32; 3]>,
    pub screen_x: i32,
    pub controller_fov: f32,
    pub draw_scale: f32,
    pub draw_scale_3d: [f32; 3],
    pub lod_bias: f32,
    pub reduce_pawn_lod: bool,
    pub keep_min_frame_rate: bool,
    pub pawn_controller_flags: Option<u8>,
}

pub struct CameraProjectionInput {
    pub render_mode: i32,
    pub fov: f32,
    pub target_size: [i32; 2],
    pub viewport_width: i32,
    pub unscaled_ortho: bool,
    pub ortho_zoom: f32,
    pub near: f32,
    pub far: f32,
}

pub fn camera_projection_matrix(input: CameraProjectionInput) -> [[f32; 4]; 4] {
    let [width, height] = input.target_size;
    if matches!(input.render_mode, 13..=17 | 35 | 36) {
        let scale = if input.unscaled_ortho { 1.0 } else {
            (f64::from(input.ortho_zoom) / (f64::from(input.viewport_width) * 15.0)) as f32
        };
        let x = (1.0 / (f64::from(width) * f64::from(scale) * 0.5)) as f32;
        let y = (1.0 / (f64::from(height) * f64::from(scale) * 0.5)) as f32;
        [[x, 0.0, 0.0, 0.0], [0.0, y, 0.0, 0.0],
            [0.0, 0.0, f32::from_bits(0x3630_0000), 0.0], [0.0, 0.0, f32::from_bits(0x3f72_0000), 1.0]]
    } else {
        let half_fov = (f64::from(input.fov) * f64::from_bits(0x3f81_df46_a252_9d39)) as f32;
        let aspect = (f64::from(width as f32) / f64::from(height)) as f32;
        perspective_matrix([half_fov; 2], [1.0, aspect], input.near, input.far)
    }
}

pub fn sine_table() -> [f32; 16384] {
    std::array::from_fn(|index| ((index as f64 * 2.0) * f64::from_bits(0x3f29_21fb_5444_2d18)).sin() as f32)
}

pub fn camera_view_matrix(location: [f32; 3], rotation: [i32; 3], render_mode: i32, viewport_scale: [f32; 2], sine_table: &[f32; 16384]) -> [[f32; 4]; 4] {
    let [x, y, z] = location;
    let [sx, sy] = viewport_scale;
    let mut view = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [-x, -y, -z, 1.0]];
    if !matches!(render_mode, 13..=17 | 35 | 36) {
        view = multiply_matrices(view, inverse_rotation_matrix(rotation, sine_table));
    }
    let axes = match render_mode {
        13 | 35 | 36 => {
            let scale_y = if render_mode == 36 { sy } else { -sy };
            [[sx, 0.0, 0.0, 0.0], [0.0, scale_y, 0.0, 0.0], [0.0, 0.0, -1.0, 0.0], [0.0, 0.0, -z, 1.0]]
        }
        14 | 15 => {
            let direction = if render_mode == 14 { -1.0 } else { 1.0 };
            [[sx, 0.0, 0.0, 0.0], [0.0, 0.0, direction, 0.0], [0.0, sy, 0.0, 0.0], [0.0, 0.0, -y, 1.0]]
        }
        16 | 17 => {
            let direction = if render_mode == 16 { 1.0 } else { -1.0 };
            let offset = if render_mode == 16 { x } else { -x };
            [[0.0, 0.0, direction, 0.0], [sx, 0.0, 0.0, 0.0], [0.0, sy, 0.0, 0.0], [0.0, 0.0, offset, 1.0]]
        }
        _ => [[0.0, 0.0, 1.0, 0.0], [sx, 0.0, 0.0, 0.0], [0.0, sy, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
    };
    multiply_matrices(view, axes)
}

pub fn inverse_rotation_matrix([pitch, yaw, roll]: [i32; 3], sine_table: &[f32; 16384]) -> [[f32; 4]; 4] {
    let sin_cos = |angle: i32| {
        let sine = ((angle.wrapping_neg() >> 2) & 0x3fff) as usize;
        let cosine = ((0x4000i32.wrapping_sub(angle) >> 2) & 0x3fff) as usize;
        (sine_table[sine], sine_table[cosine])
    };
    let (sp, cp) = sin_cos(pitch);
    let (sy, cy) = sin_cos(yaw);
    let (sr, cr) = sin_cos(roll);
    let yaw = [[cy, sy, 0.0, 0.0], [-sy, cy, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let pitch = [[cp, 0.0, sp, 0.0], [0.0, 1.0, 0.0, 0.0], [-sp, 0.0, cp, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let roll = [[1.0, 0.0, 0.0, 0.0], [0.0, cr, -sr, 0.0], [0.0, sr, cr, 0.0], [0.0, 0.0, 0.0, 1.0]];
    multiply_matrices(multiply_matrices(yaw, pitch), roll)
}

pub fn multiply_matrices(left: [[f32; 4]; 4], right: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    std::array::from_fn(|row| std::array::from_fn(|column| {
        let term = |k: usize| f64::from(left[row][k]) * f64::from(right[k][column]);
        let [a, b, c, d] = if row == 0 { [1, 0, 3, 2] } else { [3, 2, 0, 1] };
        (((term(a) + term(b)) + term(c)) + term(d)) as f32
    }))
}

pub fn perspective_matrix(half_fov: [f32; 2], multiplier: [f32; 2], near: f32, far: f32) -> [[f32; 4]; 4] {
    let z = f64::from(far) / (f64::from(far) - f64::from(near));
    [
        [(f64::from(multiplier[0]) / f64::from(half_fov[0]).tan()) as f32, 0.0, 0.0, 0.0],
        [0.0, (f64::from(multiplier[1]) / f64::from(half_fov[1]).tan()) as f32, 0.0, 0.0],
        [0.0, 0.0, z as f32, 1.0],
        [0.0, 0.0, (-f64::from(near) * z) as f32, 0.0],
    ]
}

pub fn projection_reference_point(point: [f32; 3], viewer_pawn: Option<[f32; 3]>) -> [f32; 3] {
    match viewer_pawn {
        Some(viewer) if viewer != point => viewer,
        _ => point,
    }
}

pub fn scene_project(matrix: [[f32; 4]; 4], point: [f32; 3]) -> [f32; 4] {
    let transformed = transform_point(matrix, point);
    let reciprocal = 1.0 / f64::from(transformed[3]);
    [
        (f64::from(transformed[0]) * reciprocal) as f32,
        (f64::from(transformed[1]) * reciprocal) as f32,
        (f64::from(transformed[2]) * reciprocal) as f32,
        transformed[3],
    ]
}

pub fn lod_world_point(location: [f32; 3], soft_root: Option<[f32; 3]>, mesh_to_world: [[f32; 4]; 4]) -> [f32; 3] {
    let Some(root) = soft_root else { return location };
    let [x, y, z, _] = transform_point(mesh_to_world, root);
    [x, y, z]
}

fn transform_point(matrix: [[f32; 4]; 4], point: [f32; 3]) -> [f32; 4] {
    std::array::from_fn(|column| {
        (f64::from(point[0]) * f64::from(matrix[0][column])
            + f64::from(point[1]) * f64::from(matrix[1][column])
            + f64::from(point[2]) * f64::from(matrix[2][column])
            + f64::from(matrix[3][column])) as f32
    })
}

pub fn max_draw_scale([x, y, z]: [f32; 3]) -> f32 {
    let xy = if x >= y { x } else { y };
    if xy >= z { xy } else { z }
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectedFactorInput {
    pub soft_wedges: i32,
    pub screen_x: i32,
    pub half_fov_tangent: f32,
    pub projected_w: f32,
    pub z_displace: f32,
    pub strength: f32,
    pub scale_max: f32,
    pub draw_scale: f32,
    pub lod_bias: f32,
    pub max_draw_scale_3d: f32,
}

pub fn projected_factor(input: ProjectedFactorInput) -> f32 {
    let complexity = (f64::from(input.soft_wedges) * f64::from(0.003f32) + 0.25) as f32;
    let screen = (f64::from(input.screen_x.max(820)) * f64::from(0.00109375f32) + f64::from(0.3f32)) as f32;
    let distance = f64::from(input.projected_w) - f64::from(input.z_displace);
    let distance = if distance <= 1.0 { 1.0 } else { distance };
    let denominator =
        (distance * f64::from(input.strength) * f64::from(complexity) * f64::from(input.half_fov_tangent)) as f32;
    (f64::from(input.draw_scale) / f64::from(denominator)
        * f64::from(input.lod_bias)
        * f64::from(input.scale_max)
        * f64::from(input.max_draw_scale_3d)
        * f64::from(screen)
        * 420.0) as f32
}

#[derive(Debug, Clone, Copy)]
pub struct LodSelectionInput {
    pub factor: f32,
    pub previous: usize,
    pub forced: i32,
    pub reduce_pawn_lod: bool,
    pub keep_min_frame_rate: bool,
    pub pawn_controller_flags: Option<u8>,
    pub mesh_reduction: bool,
    pub mesh_impostor: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LodSelection {
    pub index: usize,
    pub impostor: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LodInstance {
    pub index: usize,
    pub forced: i32,
    pub factor: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct LodFrame {
    pub factor: f32,
    pub reduce_pawn_lod: bool,
    pub keep_min_frame_rate: bool,
    pub pawn_controller_flags: Option<u8>,
    pub mesh_reduction: bool,
    pub mesh_impostor: bool,
}

impl LodInstance {
    pub fn update_render(&mut self, levels: &[SkeletalLodInfo], settings: &SkeletalLodSettings, input: LodRenderInput) -> Option<LodSelection> {
        let base = levels.first()?;
        let mesh_reduction = settings.reduction? != 0;
        let mesh_impostor = settings.impostor? != 0;
        let point = lod_world_point(input.location, input.soft_root.filter(|_| mesh_impostor), input.mesh_to_world);
        let point = projection_reference_point(point, input.viewer_pawn);
        let factor = projected_factor(ProjectedFactorInput {
            soft_wedges: base.soft_wedges,
            screen_x: input.screen_x,
            half_fov_tangent: half_fov_tangent(input.controller_fov),
            projected_w: scene_project(input.world_to_screen, point)[3],
            z_displace: settings.z_displace,
            strength: settings.strength,
            scale_max: settings.scale_max,
            draw_scale: input.draw_scale,
            lod_bias: input.lod_bias,
            max_draw_scale_3d: max_draw_scale(input.draw_scale_3d),
        });
        self.update(levels, LodFrame {
            factor, reduce_pawn_lod: input.reduce_pawn_lod, keep_min_frame_rate: input.keep_min_frame_rate,
            pawn_controller_flags: input.pawn_controller_flags, mesh_reduction, mesh_impostor,
        })
    }

    pub fn update(&mut self, levels: &[SkeletalLodInfo], frame: LodFrame) -> Option<LodSelection> {
        let selection = select(levels, LodSelectionInput {
            factor: frame.factor,
            previous: self.index,
            forced: self.forced,
            reduce_pawn_lod: frame.reduce_pawn_lod,
            keep_min_frame_rate: frame.keep_min_frame_rate,
            pawn_controller_flags: frame.pawn_controller_flags,
            mesh_reduction: frame.mesh_reduction,
            mesh_impostor: frame.mesh_impostor,
        })?;
        self.factor = frame.factor;
        self.index = selection.index;
        Some(selection)
    }
}

pub fn select(levels: &[SkeletalLodInfo], input: LodSelectionInput) -> Option<LodSelection> {
    if levels.is_empty() || input.previous >= levels.len() {
        return None;
    }
    let mut candidate = 0;
    let mut index = input.previous;
    if input.forced > 0 && input.forced as usize <= levels.len() {
        index = input.forced as usize - 1;
    } else if levels.len() > 1 {
        while candidate < levels.len() - 1 && input.factor <= levels[candidate].distance_factor {
            candidate += 1;
        }
        if input.previous < candidate
            && f64::from(levels[input.previous].distance_factor)
                <= f64::from(levels[input.previous].hysteresis) + f64::from(input.factor)
        {
            candidate = input.previous;
        }
        index = candidate;
    }
    if (input.reduce_pawn_lod || input.keep_min_frame_rate)
        && input.pawn_controller_flags.is_some_and(|flags| flags & 1 == 0)
        && levels.len() > 1
        && input.mesh_reduction
    {
        index = (index + 1).min(levels.len() - 1);
    }
    let impostor = input.forced == 0
        && input.mesh_impostor
        && candidate == levels.len() - 1
        && input.factor <= levels[candidate].distance_factor;
    Some(LodSelection { index, impostor })
}
