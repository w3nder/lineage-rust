use glam::{Mat4, Vec3, Vec4};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub fov_x_deg: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn view(&self) -> Mat4 {
        glam::camera::lh::view::look_at_mat4(self.eye, self.target, Vec3::Z)
    }

    pub fn view_proj(&self, aspect: f32) -> Mat4 {
        let view = self.view();
        let fov_y = 2.0 * ((self.fov_x_deg.to_radians() / 2.0).tan() / aspect).atan();
        let proj = glam::camera::lh::proj::directx::perspective(fov_y, aspect, self.near, self.far);
        proj * view
    }

    pub fn frustum(&self, aspect: f32) -> Frustum {
        let m = self.view_proj(aspect);
        let (x, y, z, w) = (m.row(0), m.row(1), m.row(2), m.row(3));
        let planes = [w + x, w - x, w + y, w - y, z, w - z];
        Frustum { planes, lengths: planes.map(|plane| plane.truncate().length()) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frustum {
    planes: [Vec4; 6],
    lengths: [f32; 6],
}

impl Frustum {
    pub fn contains_sphere(&self, center: Vec3, radius: f32) -> bool {
        let point = center.extend(1.0);
        self.planes.iter().zip(&self.lengths).all(|(plane, length)| plane.dot(point) >= -radius * length)
    }

    pub fn contains_box(&self, min: Vec3, max: Vec3) -> bool {
        self.planes.iter().all(|plane| {
            let normal = plane.truncate();
            let farthest = Vec3::select(normal.cmpge(Vec3::ZERO), max, min);
            plane.dot(farthest.extend(1.0)) >= 0.0
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec4, vec3};

    #[test]
    fn target_projects_to_the_center_with_positive_depth() {
        let camera = Camera { eye: vec3(0.0, -1000.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 10.0, far: 10000.0 };
        let clip = camera.view_proj(1.0) * Vec4::new(0.0, 0.0, 0.0, 1.0);
        let ndc = clip / clip.w;
        assert!(ndc.x.abs() < 1e-4 && ndc.y.abs() < 1e-4);
        assert!(ndc.z > 0.0 && ndc.z < 1.0);
    }

    #[test]
    fn the_frustum_keeps_spheres_in_view_and_drops_the_rest() {
        let camera = Camera { eye: vec3(0.0, -1000.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 10.0, far: 10000.0 };
        let frustum = camera.frustum(16.0 / 9.0);
        assert!(frustum.contains_sphere(Vec3::ZERO, 1.0));
        assert!(frustum.contains_sphere(vec3(0.0, 8000.0, 0.0), 1.0));
        assert!(!frustum.contains_sphere(vec3(0.0, -1200.0, 0.0), 50.0));
        assert!(!frustum.contains_sphere(vec3(0.0, 12000.0, 0.0), 50.0));
        assert!(!frustum.contains_sphere(vec3(700.0, 0.0, 0.0), 10.0));
        assert!(frustum.contains_sphere(vec3(700.0, 0.0, 0.0), 200.0));
        assert!(!frustum.contains_sphere(vec3(0.0, 0.0, 600.0), 10.0));
        assert!(frustum.contains_sphere(vec3(0.0, 0.0, 600.0), 400.0));
    }

    #[test]
    fn the_frustum_keeps_boxes_that_touch_it_and_drops_the_rest() {
        let camera = Camera { eye: vec3(0.0, -1000.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 10.0, far: 10000.0 };
        let frustum = camera.frustum(16.0 / 9.0);
        assert!(frustum.contains_box(vec3(-10.0, -10.0, -10.0), vec3(10.0, 10.0, 10.0)));
        assert!(frustum.contains_box(vec3(500.0, -10.0, -10.0), vec3(900.0, 10.0, 10.0)));
        assert!(!frustum.contains_box(vec3(700.0, -10.0, -10.0), vec3(900.0, 10.0, 10.0)));
        assert!(!frustum.contains_box(vec3(-10.0, -1300.0, -10.0), vec3(10.0, -1200.0, 10.0)));
        assert!(!frustum.contains_box(vec3(-10.0, 11000.0, -10.0), vec3(10.0, 12000.0, 10.0)));
    }

    #[test]
    fn east_is_to_the_left_when_looking_south() {
        let camera = Camera { eye: vec3(0.0, -1000.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 10.0, far: 10000.0 };
        let clip = camera.view_proj(1.0) * Vec4::new(100.0, 0.0, 0.0, 1.0);
        assert!(clip.x / clip.w < 0.0);
        let up = camera.view_proj(1.0) * Vec4::new(0.0, 0.0, 100.0, 1.0);
        assert!(up.y / up.w > 0.0);
    }
}
