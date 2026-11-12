use super::terrain::{Grid, edge_turned};
use glam::Vec3;
use ue2_level::Terrain;

pub fn safe_normal(v: Vec3) -> Vec3 {
    let [x, y, z] = v.to_array().map(f64::from);
    let squared = (z * z + y * y + x * x) as f32;
    if squared < 1e-8 {
        return Vec3::ZERO;
    }
    let inverse = 1.0 / f64::from(squared).sqrt();
    Vec3::new(v.x * inverse as f32, v.y * inverse as f32, (z * inverse) as f32)
}

pub fn update_faces(vertices: &[[f32; 3]], size: [usize; 2], edge_turn: &[u32], faces: &mut [[[f32; 3]; 2]]) {
    let [width, height] = size;
    for y in 1..height {
        for x in 1..width {
            let a = (y - 1) * width + x - 1;
            let b = a + 1;
            let c = y * width + x - 1;
            let d = c + 1;
            let triangles = if edge_turned(edge_turn, width, x - 1, y - 1) {
                [[a, b, c], [c, b, d]]
            } else {
                [[a, b, d], [a, d, c]]
            };
            faces[a] = triangles.map(|[a, b, c]| {
                let ab = (Vec3::from(vertices[b]) - Vec3::from(vertices[a])).as_dvec3();
                let ac = (Vec3::from(vertices[c]) - Vec3::from(vertices[a])).as_dvec3();
                safe_normal(safe_normal(ab.cross(ac).as_vec3())).to_array()
            });
        }
    }
}

pub fn vertex_normal(
    faces: &[[[f32; 3]; 2]],
    width: usize,
    vertex_count: usize,
    point: [usize; 2],
    inverted: bool,
) -> [f32; 3] {
    if faces.len() < vertex_count {
        return [0.0, 0.0, 1.0];
    }
    let [x, y] = point;
    let Some(first) = faces.get(y * width + x) else { return [0.0, 0.0, 1.0] };
    let mut sum: [f32; 3] = std::array::from_fn(|axis| first[0][axis] + first[1][axis]);
    for index in [
        (x > 0).then(|| y * width + x - 1),
        (y > 0).then(|| (y - 1) * width + x),
        (x > 0 && y > 0).then(|| (y - 1) * width + x - 1),
    ]
    .into_iter()
    .flatten()
    {
        let pair = faces[index];
        for axis in 0..3 {
            let pair_sum = f64::from(pair[0][axis]) + f64::from(pair[1][axis]);
            let pair_sum = if axis == 2 { f64::from(pair_sum as f32) } else { pair_sum };
            sum[axis] = (f64::from(sum[axis]) + pair_sum) as f32;
        }
    }
    let normal = safe_normal(Vec3::from(sum));
    (if inverted { -normal } else { normal }).to_array()
}

pub fn sector_point(offset: [usize; 2], local: [usize; 2]) -> [usize; 2] {
    std::array::from_fn(|axis| offset[axis] + if offset[axis] == 240 && local[axis] == 16 { 15 } else { local[axis] })
}

pub fn face_buffer(terrain: &Terrain, grid: &Grid) -> Option<Vec<[[f32; 3]; 2]>> {
    let [width, height] = grid.heightmap;
    let count = width * height;
    if terrain.native.face_normals.len() >= count {
        return Some(terrain.native.face_normals.clone());
    }
    if (count + count * 3 / 8 + 32) * 24 < 32769 {
        return None;
    }
    let mut faces = terrain.native.face_normals.clone();
    faces.resize(count, [[0.0; 3]; 2]);
    let vertices = if terrain.native.vertices.len() >= count {
        terrain.native.vertices.clone()
    } else {
        (0..height).flat_map(|y| (0..width).map(move |x| grid.position(x, y))).collect()
    };
    update_faces(&vertices, [width, height], &terrain.edge_turn, &mut faces);
    Some(faces)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_vertex_normal_106d2700_missing_buffer_and_inversion() {
        assert_eq!(vertex_normal(&[], 2, 4, [1, 1], true), [0.0, 0.0, 1.0]);
        let faces = vec![[[0.0, 0.0, 1.0]; 2]; 4];
        assert_eq!(vertex_normal(&faces, 2, 4, [1, 1], true), [0.0, 0.0, -1.0]);
        assert_eq!(vertex_normal(&faces, 2, 4, [0, 0], false), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn sector_get_vertex_normal_106d3570_clamps_both_axes_only_at_seam() {
        assert_eq!(sector_point([240, 240], [16, 16]), [255, 255]);
        assert_eq!(sector_point([224, 240], [16, 16]), [240, 255]);
        assert_eq!(sector_point([240, 224], [16, 16]), [255, 240]);
        assert_eq!(sector_point([240, 240], [15, 15]), [255, 255]);
    }
}
