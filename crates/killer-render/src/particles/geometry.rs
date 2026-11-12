use super::{Particle, byte, rotate_axis, rotator_matrix, safe_normal};
use crate::camera::Camera;
use glam::{Mat4, Vec3};
use killer_assets::scene::Vertex;
use ue2_level::{Emitter, SpriteEmitter};

pub fn atlas(e: &SpriteEmitter, p: &Particle) -> ([f32; 4], [f32; 4], [u8; 4]) {
    let (u, v) = (e.subdivisions[0], e.subdivisions[1]);
    let mut color = p.color;
    if u == 0 || v == 0 { return ([0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0], color); }
    let total = u.saturating_mul(v);
    let rel = p.relative_time();
    let count = if e.subdivision_range[1] == 0 { total } else { e.subdivision_range[1].saturating_sub(e.subdivision_range[0]).max(1) };
    let mut index = p.subdivision;
    if index == -1 {
        if p.lifetime == 0.0 { return ([0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 1.0, 1.0], color); }
        index = if e.use_subdivision_scale { e.subdivision_scale.iter().take_while(|t| **t < rel).count() as i32 }
            else { (count as f32 * rel).floor() as i32 + if e.subdivision_range[1] == 0 { 0 } else { e.subdivision_range[0] as i32 } };
        index = index.clamp(0, total.saturating_sub(1) as i32);
    }
    let cell = |index: i32| {
        let x = (index / v as i32) as f32 / u as f32;
        let y = (index % v as i32) as f32 / v as f32;
        [x, y, (x + 1.0 / u as f32).min(1.0), (y + 1.0 / v as f32).min(1.0)]
    };
    let current = cell(index);
    if !e.blend_subdivisions || p.subdivision != -1 { return (current, [0.0, 0.0, 1.0, 1.0], color); }
    let next = if index + 1 == count as i32 { current } else { cell(index + 1) };
    let fraction = if !e.use_subdivision_scale || e.subdivision_scale.len() <= count as usize {
        let t = count as f32 * rel; t - t.floor()
    } else {
        let previous = index.checked_sub(1).and_then(|i| e.subdivision_scale.get(i as usize)).copied().unwrap_or(0.0);
        let end = e.subdivision_scale.get(index as usize).copied().unwrap_or(1.0).min(1.0);
        (rel - previous) / (end - previous)
    };
    color[3] = byte(fraction * 255.0);
    (current, next, color)
}

pub fn sprite_vertices(e: &SpriteEmitter, actor: &Emitter, p: &Particle, camera: &Camera, aspect: f32) -> [Vertex; 4] {
    let inverse = camera.view_proj(aspect).inverse();
    let origin = (inverse * Vec3::ZERO.extend(1.0)).truncate();
    let mut up = safe_normal((inverse * Vec3::new(0.0, -1000.0, 0.0).extend(1.0)).truncate() - origin);
    let mut right = safe_normal((inverse * Vec3::new(1000.0, 0.0, 0.0).extend(1.0)).truncate() - origin);
    let mut front = safe_normal(right.cross(up));
    let rotation = rotator_matrix(actor.rotation);
    let mut eye = camera.eye;
    if e.coordinate_system == 1 {
        eye = rotation.transpose() * (eye - Vec3::from(actor.location));
        up = rotation.transpose() * up;
        right = rotation.transpose() * right;
        front = rotation.transpose() * front;
    }
    let normal = safe_normal(Vec3::from(e.projection_normal));
    let directed_normal = if e.use_rotation_from == 1 { rotation * normal } else { normal };
    let dir = if e.direction == 4 { directed_normal } else { safe_normal(p.location - p.old_location) };
    let perpendicular = if matches!(e.direction, 5 | 6) { dir.cross(normal) } else { safe_normal(dir.cross(p.location - eye)) };
    let (mut a, mut b) = match e.direction {
        0 => (up * p.size.x, right * p.size.x),
        1 | 5 => (dir * p.size.y, perpendicular * p.size.x),
        2 | 6 => (perpendicular * p.size.y, dir * p.size.x),
        3 | 4 => {
            let other = if dir.x.abs() > 0.57 { Vec3::Y } else if dir.y.abs() > 0.57 { Vec3::Z } else { Vec3::X };
            let a = dir.cross(other) * p.size.y;
            front = dir;
            (a, safe_normal(dir.cross(a)) * p.size.x)
        }
        7 => (up * p.size.y, right * p.size.x),
        _ => (Vec3::ZERO, Vec3::ZERO),
    };
    if e.spin_particles {
        let angle = if p.spins.x >= 0.0 { (p.time * p.spins.x + p.start_spin.x).round_ties_even() as i32 & 65535 }
            else { 65535 - ((p.start_spin.x - p.time * p.spins.x).round_ties_even() as i32 & 65535) };
        a = rotate_axis(a, angle, front);
        b = rotate_axis(b, angle, front);
    }
    let (uv, next, color) = atlas(e, p);
    let color = color.map(|c| f32::from(c) / 255.0);
    let positions = [p.location - a - b, p.location - a + b, p.location + a - b, p.location + a + b];
    let uv = [[uv[0], uv[3]], [uv[2], uv[3]], [uv[0], uv[1]], [uv[2], uv[1]]];
    let next = [[next[0], next[3]], [next[2], next[3]], [next[0], next[1]], [next[2], next[1]]];
    std::array::from_fn(|i| Vertex { position: if e.coordinate_system == 1 { rotation * positions[i] + Vec3::from(actor.location) } else { positions[i] }.to_array(), normal: front.to_array(), uv: uv[i], mask_uv: next[i], color })
}

pub fn mesh_matrix(e: &SpriteEmitter, actor: &Emitter, p: &Particle) -> Mat4 {
    let spin = if e.spin_particles { let s = p.spins * p.time + p.start_spin; rotator_matrix([s.y as i32, s.x as i32, s.z as i32]) } else { glam::Mat3::IDENTITY };
    let mut m = Mat4::from_mat3(spin) * Mat4::from_scale(p.size) * Mat4::from_translation(-Vec3::from(actor.pre_pivot));
    if e.coordinate_system == 4 { m = Mat4::from_mat3(rotator_matrix(p.rotation)) * m; }
    m = Mat4::from_translation(p.location) * m;
    if e.coordinate_system == 1 { m = Mat4::from_translation(Vec3::from(actor.location)) * Mat4::from_mat3(rotator_matrix(actor.rotation)) * m; }
    m
}

pub fn mesh_bounds(e: &SpriteEmitter, actor: &Emitter, particles: &[Particle], mesh: [[f32; 3]; 2]) -> Option<killer_assets::scene::actor_lighting::Bounds> {
    use killer_assets::scene::actor_lighting::Bounds;
    let mut bounds = Bounds { min: Vec3::splat(f32::INFINITY), max: Vec3::splat(f32::NEG_INFINITY) };
    let rotation = rotator_matrix(actor.rotation);
    for p in particles.iter().filter(|p| p.alive()) {
        let position = if matches!(e.coordinate_system, 1 | 3) { rotation * p.location + Vec3::from(actor.location) } else { p.location };
        bounds.min = bounds.min.min(position);
        bounds.max = bounds.max.max(position);
    }
    if !bounds.min.is_finite() { return None; }
    let max_scale = if e.use_size_scale && !e.use_regular_size_scale { e.size_scale.iter().map(|p| p[1]).fold(1.0, f32::max) } else { 1.0 };
    let mut scale = Vec3::from(e.start_size.map(|r| r[0].max(r[1]))) * max_scale;
    if e.uniform_size { scale = Vec3::splat(scale.x); }
    let center = (Vec3::from(mesh[0]) + Vec3::from(mesh[1])) * 0.5;
    let extent = Vec3::from(mesh[0]).abs().max(Vec3::from(mesh[1]).abs()) - center;
    let offset = scale * center;
    bounds.min += offset;
    bounds.max += offset;
    let enlarged = bounds.union(Bounds { min: bounds.min - 1.1 * scale * extent, max: bounds.max + 1.1 * scale * extent });
    let pivot = Vec3::splat(Vec3::from(actor.pre_pivot).length());
    Some(Bounds { min: enlarged.min - pivot, max: enlarged.max + pivot })
}

pub fn beam_vertices(e: &SpriteEmitter, actor: &Emitter, p: &mut Particle, eye: Vec3, noise_time: &mut f32, random: &mut u32) -> (Vec<Vertex>, Vec<u32>) {
    let Some(beam) = &e.beam else { return (Vec::new(), Vec::new()); };
    if p.beam.len() < 2 { return (Vec::new(), Vec::new()); }
    let rotation = rotator_matrix(actor.rotation);
    let eye = if e.coordinate_system == 1 { rotation.transpose() * (eye - Vec3::from(actor.location)) } else { eye };
    let period = super::random_range(random, beam.dynamic_time);
    let ticks = if period != 0.0 { let n = (*noise_time / period) as i32; *noise_time %= period; n } else { 1 };
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let sheets = beam.rotating_sheets.max(1);
    let color = p.color.map(|c| f32::from(c) / 255.0);
    for sheet in 0..sheets {
        let base = vertices.len() as u32;
        let mut previous = 2.0 * p.beam[0] - p.beam[1];
        let count = p.beam.len();
        for (i, point) in p.beam.iter_mut().enumerate() {
            if ticks != 0 && beam.dynamic_points[0] < i as f32 && (i as f32) < beam.dynamic_points[1] { *point += ticks as f32 * super::random_vector(random, beam.dynamic_noise); }
            let dir = *point - previous;
            let mut perpendicular = safe_normal(dir.cross(previous - eye));
            if sheet != 0 { perpendicular = rotate_axis(perpendicular, (sheet * (32768 / sheets)) as i32, safe_normal(dir)); }
            let width = perpendicular * p.size.x;
            for (position, v) in [(*point + width, 0.0), (*point - width, beam.v_scale)] {
                let uv = [beam.u_scale * i as f32 / (count - 1) as f32, v];
                vertices.push(Vertex { position: if e.coordinate_system == 1 { rotation * position + Vec3::from(actor.location) } else { position }.to_array(), normal: [0.0; 3], uv, mask_uv: uv, color });
            }
            previous = *point;
            if i + 1 < count { let b = base + 2 * i as u32; indices.extend([b, b + 1, b + 2, b + 1, b + 3, b + 2]); }
        }
    }
    (vertices, indices)
}
