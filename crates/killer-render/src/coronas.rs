use crate::camera::Camera;
use crate::sky::body_quad;
use glam::Vec3;
use killer_assets::scene::Vertex;
use killer_assets::scene::lights::CoronaLight;

pub const MAX_CORONAS: usize = 64;

pub fn app_frand(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(0x343fd).wrapping_add(0x269ec3);
    ((*state >> 16) & 0x7fff) as f32 * f32::from_bits(0x38000100)
}

#[derive(Default, Debug)]
pub struct CoronaState {
    pub fade: f32,
    pub visible: bool,
    pub checked: Option<f32>,
}

impl CoronaState {
    pub fn update(&mut self, light: &CoronaLight, now: f32, dt: f32, candidate: bool, visible: bool) {
        self.fade = (self.fade - light.scale_glow * (dt * 3.0).min(1.0)).max(0.0);
        if candidate && visible {
            self.fade = (self.fade + 2.0 * light.scale_glow * (dt * 3.0).min(1.0)).min(1.0);
        }
        if candidate && self.checked.is_none() { self.checked = Some(now); }
        if self.fade == 0.0 { self.checked = None; }
    }

    pub fn needs_trace(&self, now: f32, random: f32) -> bool {
        self.checked.is_none_or(|last| now - last > (random + 1.0) * 0.33)
    }
}

pub fn half_size(light: &CoronaLight, eye: Vec3) -> f32 {
    (512.0 * light.draw_scale * (light.position.distance(eye) * 0.001).clamp(0.01, 10.0)).clamp(light.min_size, light.max_size)
}

pub fn color(hue: u8, saturation: u8, fade: f32) -> [f32; 4] {
    let h = f32::from(hue);
    let rgb = if hue < 86 { [(85.0 - h) / 85.0, h / 85.0, 0.0] }
        else if hue < 170 { [0.0, (170.0 - h) / 85.0, (h - 85.0) / 85.0] }
        else { [(h - 170.0) / 85.0, 0.0, (255.0 - h) / 84.0] };
    let s = f32::from(saturation) / 255.0;
    let rgb = rgb.map(|c| (c + (1.0 - c) * s) * fade);
    [rgb[0], rgb[1], rgb[2], 0.0]
}

pub fn quad(light: &CoronaLight, camera: &Camera, aspect: f32, fade: f32) -> [Vertex; 4] {
    body_quad(light.position, half_size(light, camera.eye), camera.view_proj(aspect), color(light.hue, light.saturation, fade))
}

pub fn scene_clear(scene: &killer_assets::scene::Scene, from: Vec3, to: Vec3) -> bool {
    let direction = to - from;
    for draw in scene.draws.iter().filter(|draw| draw.pass != killer_assets::scene::Pass::Translucent && !scene.zone(draw.zone).is_sky) {
        for [a, b, c] in draw.world_triangles() {
            let a = Vec3::from(a.position);
            let e1 = Vec3::from(b.position) - a;
            let e2 = Vec3::from(c.position) - a;
            let p = direction.cross(e2);
            let determinant = e1.dot(p);
            if determinant.abs() < 1e-6 { continue; }
            let inverse = 1.0 / determinant;
            let offset = from - a;
            let u = offset.dot(p) * inverse;
            if !(0.0..=1.0).contains(&u) { continue; }
            let q = offset.cross(e1);
            let v = direction.dot(q) * inverse;
            if v < 0.0 || u + v > 1.0 { continue; }
            let t = e2.dot(q) * inverse;
            if t > 0.0 && t <= 1.0 { return false; }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn lamp() -> CoronaLight {
        CoronaLight { position: Vec3::ZERO, radius: 30000.0, hue: 0, saturation: 255, min_size: 0.0, max_size: 1000.0, draw_scale: 0.25, scale_glow: 1.0, material: killer_assets::scene::sky::flare_material("test") }
    }

    #[test]
    fn fcorona_render_1061cb70_size_and_color_ignore_light_brightness() {
        assert_eq!(half_size(&lamp(), Vec3::new(1000.0, 0.0, 0.0)), 128.0);
        assert_eq!(half_size(&lamp(), Vec3::new(20000.0, 0.0, 0.0)), 1000.0);
        assert_eq!(color(0, 0, 0.5), [0.5, 0.0, 0.0, 0.0]);
        assert_eq!(color(0, 255, 0.5), [0.5, 0.5, 0.5, 0.0]);
    }

    #[test]
    fn fcorona_render_10616e60_reconfirms_and_fades_occluded_entries() {
        let mut state = CoronaState::default();
        state.update(&lamp(), 0.1, 0.1, true, true);
        assert!((state.fade - 0.6).abs() < 1e-6);
        state.update(&lamp(), 0.2, 0.1, false, false);
        assert!((state.fade - 0.3).abs() < 1e-6);
        state.update(&lamp(), 0.3, 0.1, false, false);
        assert_eq!(state.fade, 0.0);
        assert!(state.needs_trace(0.4, 1.0));
        assert!(state.needs_trace(0.8, 1.0));
    }
}
