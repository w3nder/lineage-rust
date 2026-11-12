use crate::timeenv::fgethsv;
use glam::Vec3;
use ue2_level::Light;

pub const LIGHT_TYPE_STEADY: u8 = 1;
pub const LIGHT_EFFECT_SUNLIGHT: u8 = 19;
pub const STATIC_MESH_POINT_SCALE: f32 = 0.5;
pub const ACTOR_POINT_SCALE: f32 = 1.0;
pub const MAX_HARDWARE_LIGHTS: usize = 8;
pub const TERRAIN_POINT_SCALE: f32 = 1.0;
pub const EMITTER_LIGHT_WORLD_ONLY: u8 = 1;
pub const EMITTER_LIGHT_ALL: u8 = 2;
pub const LIGHT_TYPE_PULSE: u8 = 2;
pub const LIGHT_TYPE_BLINK: u8 = 3;
pub const LIGHT_TYPE_FLICKER: u8 = 4;
pub const LIGHT_TYPE_STROBE: u8 = 5;
pub const LIGHT_TYPE_SUBTLE_PULSE: u8 = 7;
pub const LIGHT_TYPE_TEXTURE_PALETTE_ONCE: u8 = 8;
pub const LIGHT_TYPE_TEXTURE_PALETTE_LOOP: u8 = 9;
pub const LIGHT_TYPE_FADE_OUT: u8 = 10;

#[derive(Clone, Debug, PartialEq)]
pub struct CoronaLight {
    pub position: Vec3,
    pub radius: f32,
    pub hue: u8,
    pub saturation: u8,
    pub min_size: f32,
    pub max_size: f32,
    pub draw_scale: f32,
    pub scale_glow: f32,
    pub material: l2_catalog::Material,
}

pub fn coronas(client: &mut crate::client::Client, lights: &[Light]) -> Vec<CoronaLight> {
    lights.iter().filter_map(|light| {
        let corona = light.corona.as_ref()?;
        let mut material = super::resolve_material(client, &corona.skin);
        if !corona.own_blend {
            for pass in &mut material.passes {
                pass.src = l2_catalog::BlendFactor::One;
                pass.dst = l2_catalog::BlendFactor::InvSrcColor;
                pass.alpha_blend = true;
                pass.z_write = false;
                pass.z_test = false;
                pass.two_sided = true;
            }
        }
        Some(CoronaLight { position: Vec3::from(light.location), radius: light.radius, hue: light.hue, saturation: light.saturation, min_size: corona.min_size, max_size: corona.max_size, draw_scale: corona.draw_scale, scale_glow: corona.scale_glow, material })
    }).collect()
}

pub fn effect(light: &Light, time: f32, life_fraction: f32, flicker: f32, strobe: bool) -> f32 {
    let period = f32::from(light.period).max(1.0);
    let phase = f32::from(light.phase);
    let x = time * 2_293_760.0 / period + phase * 256.0;
    let sine = ((((x as i32 >> 2) & 0x3fff) as f32) * std::f32::consts::TAU / 16384.0).sin();
    match light.light_type {
        LIGHT_TYPE_STEADY => 1.0,
        LIGHT_TYPE_PULSE => 0.6 + 0.39 * sine,
        LIGHT_TYPE_BLINK => {
            let blink = time * 2_293_760.0 / (f32::from(light.period) + 1.0) + f32::from(light.phase) * 256.0;
            if blink as i32 & 1 == 0 { 1.0 } else { 0.0 }
        }
        LIGHT_TYPE_FLICKER => if flicker < 0.5 { 0.0 } else { flicker },
        LIGHT_TYPE_STROBE => f32::from(!strobe),
        LIGHT_TYPE_SUBTLE_PULSE => 0.9 + 0.09 * sine,
        LIGHT_TYPE_FADE_OUT => ((1.0 - life_fraction) * 1.5).min(1.0),
        LIGHT_TYPE_TEXTURE_PALETTE_ONCE | LIGHT_TYPE_TEXTURE_PALETTE_LOOP => 0.0,
        _ => 0.0,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct PointLight {
    pub source: usize,
    pub position: Vec3,
    pub radius: f32,
    pub color: [f32; 3],
    pub brightness: f32,
    pub direction: Vec3,
    pub effect: u8,
    pub cone: u8,
    pub special_lit: bool,
}

pub fn point_lights(lights: &[Light]) -> Vec<PointLight> {
    lights
        .iter()
        .enumerate()
        .filter(|(_, light)| {
            light.effect != LIGHT_EFFECT_SUNLIGHT
                && !matches!(light.light_type, LIGHT_TYPE_TEXTURE_PALETTE_ONCE | LIGHT_TYPE_TEXTURE_PALETTE_LOOP)
                && light.radius > 0.0
        })
        .map(|(source, light)| PointLight {
            source,
            position: Vec3::from(light.location),
            radius: light.radius,
            brightness: light.brightness,
            direction: Vec3::from(light.direction),
            effect: light.effect,
            cone: light.cone,
            special_lit: light.special_lit,
            color: fgethsv(light.hue, light.saturation, 255)
                .map(|channel| channel * light.brightness / 255.0 * static_effect(light)),
        })
        .collect()
}

pub fn static_effect(light: &Light) -> f32 {
    match light.light_type {
        LIGHT_TYPE_PULSE => 0.6,
        LIGHT_TYPE_SUBTLE_PULSE => 0.9,
        LIGHT_TYPE_BLINK | LIGHT_TYPE_STROBE => 0.5,
        LIGHT_TYPE_FLICKER => 0.375,
        _ => effect(light, 0.0, 0.0, 0.0, false),
    }
}

pub fn terrain_lights(lights: &[Light]) -> Vec<PointLight> {
    let world: Vec<Light> = lights
        .iter()
        .filter(|light| !light.time_light && matches!(light.emitter_light_type, EMITTER_LIGHT_WORLD_ONLY | EMITTER_LIGHT_ALL))
        .cloned()
        .collect();
    point_lights(&world)
}

pub fn sector_lights(lights: &[PointLight], min: Vec3, max: Vec3) -> Vec<PointLight> {
    lights
        .iter()
        .filter(|light| {
            let low = min - Vec3::splat(light.radius);
            let high = max + Vec3::splat(light.radius);
            light.position.cmpge(low).all() && light.position.cmple(high).all()
        })
        .take(MAX_HARDWARE_LIGHTS)
        .copied()
        .collect()
}

pub fn point_falloff(distance: f32, radius: f32) -> f32 {
    let a = distance / radius;
    if a < 1.0 { 2.0 * a * a * a - 3.0 * a * a + 1.0 } else { 0.0 }
}

pub fn hardware_attenuation(light: &PointLight, center: Vec3, radius: f32) -> [f32; 2] {
    let distance = center.distance(light.position);
    let near = f64::from((distance - radius).max(0.0) + 1.0);
    let far = f64::from(light.radius.min(distance + radius) - 1.0);
    let falloff = |distance: f64| {
        if distance >= f64::from(light.radius) { return 0.0; }
        let a = distance / f64::from(light.radius);
        2.0 * (2.0 * a * a * a - 3.0 * a * a + 1.0)
    };
    let inverse_near = (1.0 / falloff(near)) as f32;
    let intercept = f64::from(inverse_near) - ((1.0 / falloff(far) - f64::from(inverse_near)) / (far - near)) * near;
    let intercept = if intercept <= 0.0 { 0.0 } else { intercept };
    [intercept as f32, ((f64::from(inverse_near) - intercept) / near) as f32]
}

pub fn point_term(position: Vec3, normal: Vec3, lights: &[PointLight], scale: f32) -> [f32; 3] {
    let mut sum = Vec3::ZERO;
    for light in lights {
        let offset = light.position - position;
        let distance = offset.length();
        if distance >= light.radius {
            continue;
        }
        if !hardware_cone(light, position) {
            continue;
        }
        let lambert = normal.dot(offset / distance.max(f32::EPSILON)).max(0.0);
        sum += Vec3::from(light.color) * (scale * lambert * point_falloff(distance, light.radius));
    }
    sum.to_array()
}

pub fn fitted_point_term(position: Vec3, normal: Vec3, lights: &[PointLight], scale: f32, sphere: (Vec3, f32)) -> [f32; 3] {
    let mut sum = Vec3::ZERO;
    for light in lights {
        let offset = light.position - position;
        let distance = offset.length();
        if distance >= light.radius || !hardware_cone(light, position) {
            continue;
        }
        let [intercept, slope] = hardware_attenuation(light, sphere.0, sphere.1);
        let fitted = 1.0 / (intercept + slope * distance);
        let attenuation = if fitted.is_finite() && fitted >= 0.0 { fitted } else { 2.0 * point_falloff(distance, light.radius) };
        let lambert = normal.dot(offset / distance.max(f32::EPSILON)).max(0.0);
        sum += Vec3::from(light.color) * (scale * 0.5 * lambert * attenuation);
    }
    sum.to_array()
}

pub fn nearest(lights: &[PointLight], position: Vec3) -> Vec<PointLight> {
    select(lights, position, 0.0, false, 4, false)
}

pub fn leaf_candidates(level: &ue2_level::Level, lights: &[PointLight], center: Vec3, radius: f32) -> Vec<PointLight> {
    leaf_candidates_with(level, lights, center, true, |normal| radius * normal.length())
}

pub fn box_leaf_candidates(level: &ue2_level::Level, lights: &[PointLight], center: Vec3, extents: Vec3) -> Vec<PointLight> {
    leaf_candidates_with(level, lights, center, false, |normal| normal.x.abs() * extents.x + normal.y.abs() * extents.y + normal.z.abs() * extents.z)
}

fn leaf_candidates_with(level: &ue2_level::Level, lights: &[PointLight], center: Vec3, inclusive: bool, push: impl Fn(Vec3) -> f32) -> Vec<PointLight> {
    let mut leaves = Vec::new();
    let mut stack = vec![0usize];
    let mut visited = std::collections::HashSet::new();
    while let Some(index) = stack.pop() {
        if !visited.insert(index) { continue; }
        let Some(node) = level.bsp_tree.nodes.get(index) else { continue };
        let normal = Vec3::new(node.plane[0], node.plane[1], node.plane[2]);
        let distance = normal.dot(center) - node.plane[3];
        let extent = push(normal);
        let (front, back) = if inclusive { (distance >= -extent, distance <= extent) } else { (-distance < extent, distance < extent) };
        for (touches, child, leaf) in [(front, node.front, node.leaf_front), (back, node.back, node.leaf_back)] {
            if !touches { continue; }
            if let Ok(child) = usize::try_from(child) { stack.push(child); }
            else if let Ok(leaf) = usize::try_from(leaf) { leaves.push(leaf); }
        }
    }
    let mut names = std::collections::HashSet::new();
    for leaf in leaves {
        let Some(leaf) = level.bsp_render.leaves.get(leaf) else { continue };
        let Ok(start) = usize::try_from(leaf.permeating) else { continue };
        for name in level.bsp_render.lights.iter().skip(start) {
            let Some(name) = name else { break };
            names.insert(name.as_str());
        }
    }
    lights.iter().filter(|light| level.lights.get(light.source).is_some_and(|source| names.contains(source.name.as_str()))).copied().collect()
}

pub fn hardware_cone(light: &PointLight, position: Vec3) -> bool {
    !matches!(light.effect, 8 | 12)
        || (position - light.position).normalize_or_zero().dot(light.direction) > std::f32::consts::FRAC_1_SQRT_2
}

pub fn sample_intensity(light: &PointLight, position: Vec3, normal: Vec3) -> f32 {
    if light.effect == LIGHT_EFFECT_SUNLIGHT {
        return 2.0 * (-normal.dot(light.direction)).max(0.0);
    }
    let offset = light.position - position;
    let distance = offset.length();
    if distance >= light.radius { return 0.0; }
    let dot = normal.dot(offset);
    match light.effect {
        17 => 2.0 * (1.0 - (offset.x * offset.x + offset.y * offset.y) / light.radius.powi(2)).max(0.0),
        13 if dot > 0.0 => 2.0 * (1.02 - distance / light.radius).sqrt(),
        20 if dot > 0.0 => 2.0 * (1.02 - distance.powi(2) / light.radius.powi(2)),
        13 | 20 => 0.0,
        _ => {
            if dot <= 0.0 { return 0.0; }
            let falloff = 2.0 * dot / distance * point_falloff(distance, light.radius);
            if !matches!(light.effect, 8 | 12) { return falloff; }
            let c = f32::from(light.cone) / 256.0;
            let p = -offset.dot(light.direction);
            let k = 1.0 - c;
            if p > 0.0 && k * k * distance * distance < p * p {
                ((p / distance - k) / c).powi(2) * falloff
            } else { 0.0 }
        }
    }
}

pub fn priority(light: &PointLight, center: Vec3, radius: f32) -> i32 {
    if light.effect == LIGHT_EFFECT_SUNLIGHT { return i32::MAX - 1; }
    let offset = center - light.position;
    if matches!(light.effect, 8 | 12) {
        let dot = offset.normalize_or_zero().dot(light.direction);
        let k = 1.0 - f32::from(light.cone) / 256.0;
        if dot <= 0.0 || dot <= k * k { return 0; }
    }
    let reach = radius + light.radius;
    ((1.0 - offset.length_squared() / (reach * reach)) * light.brightness * 1024.0).round_ties_even() as i32
}

pub fn select(lights: &[PointLight], position: Vec3, radius: f32, special_lit: bool, max_lights: usize, sunlight: bool) -> Vec<PointLight> {
    let mut reaching: Vec<(i32, PointLight)> = lights
        .iter()
        .filter(|light| light.special_lit == special_lit)
        .map(|light| (priority(light, position, radius), *light))
        .filter(|(priority, _)| *priority > 0)
        .collect();
    reaching.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    reaching.into_iter().take(max_lights.min(MAX_HARDWARE_LIGHTS).saturating_sub(usize::from(sunlight))).map(|(_, light)| light).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lamp(location: [f32; 3], radius: f32, brightness: f32) -> Light {
        Light {
            name: String::new(),
            location,
            radius,
            brightness,
            hue: 26,
            saturation: 153,
            light_type: LIGHT_TYPE_STEADY,
            effect: 0,
            emitter_light_type: 0,
            time_light: false,
            period: 32,
            phase: 0,
            cone: 128,
            direction: [1.0, 0.0, 0.0],
            rotation: [0; 3],
            special_lit: false,
            corona: None,
        }
    }

    #[test]
    fn steady_lights_get_their_hsv_color_scaled_by_brightness() {
        let lights = point_lights(&[lamp([1.0, 2.0, 3.0], 775.0, 120.0)]);
        assert_eq!(lights.len(), 1);
        assert_eq!(lights[0].position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(lights[0].radius, 775.0);
        let expected = fgethsv(26, 153, 255).map(|c| c * 120.0 / 255.0);
        assert_eq!(lights[0].color, expected);
        assert!(expected[0] > expected[1] && expected[1] > expected[2], "{expected:?}");
    }

    #[test]
    fn sunlight_and_palette_lights_are_left_out() {
        let sun = Light { effect: LIGHT_EFFECT_SUNLIGHT, ..lamp([0.0; 3], 100.0, 70.0) };
        let palette = Light { light_type: LIGHT_TYPE_TEXTURE_PALETTE_LOOP, ..lamp([0.0; 3], 100.0, 70.0) };
        let pulse = Light { light_type: LIGHT_TYPE_PULSE, ..lamp([0.0; 3], 100.0, 70.0) };
        assert_eq!(point_lights(&[sun, palette, pulse]).len(), 1);
    }

    #[test]
    fn effects_follow_the_engine_factors() {
        let base = lamp([0.0; 3], 100.0, 70.0);
        assert_eq!(effect(&base, 0.0, 0.0, 0.0, false), 1.0);
        assert!((effect(&Light { light_type: LIGHT_TYPE_PULSE, ..base.clone() }, 0.0, 0.0, 0.0, false) - 0.6).abs() < 1e-6);
        assert!((effect(&Light { light_type: LIGHT_TYPE_SUBTLE_PULSE, ..base.clone() }, 0.0, 0.0, 0.0, false) - 0.9).abs() < 1e-6);
        assert_eq!(effect(&Light { light_type: LIGHT_TYPE_FADE_OUT, ..base }, 1.0, 1.0, 0.0, false), 0.0);
    }

    #[test]
    fn effects_preserve_discrete_light_states() {
        let base = lamp([0.0; 3], 100.0, 70.0);
        assert_eq!(effect(&Light { light_type: LIGHT_TYPE_FLICKER, ..base.clone() }, 0.0, 0.0, 0.4, false), 0.0);
        assert_eq!(effect(&Light { light_type: LIGHT_TYPE_FLICKER, ..base.clone() }, 0.0, 0.0, 0.75, false), 0.75);
        assert_eq!(effect(&Light { light_type: LIGHT_TYPE_STROBE, ..base.clone() }, 0.0, 0.0, 0.0, true), 0.0);
        assert_eq!(effect(&Light { light_type: LIGHT_TYPE_STROBE, ..base }, 0.0, 0.0, 0.0, false), 1.0);
    }

    #[test]
    fn falloff_is_the_engines_smoothstep_from_one_to_zero() {
        assert_eq!(point_falloff(0.0, 100.0), 1.0);
        assert!((point_falloff(50.0, 100.0) - 0.5).abs() < 1e-6);
        assert!((point_falloff(30.0, 100.0) - 0.784).abs() < 1e-6);
        assert_eq!(point_falloff(100.0, 100.0), 0.0);
        assert_eq!(point_falloff(150.0, 100.0), 0.0);
    }

    #[test]
    fn point_term_scales_lambert_by_falloff_and_skips_lights_out_of_reach() {
        let lights = point_lights(&[lamp([0.0, 0.0, 50.0], 100.0, 255.0), lamp([0.0, 500.0, 0.0], 100.0, 255.0)]);
        let facing = point_term(Vec3::ZERO, Vec3::Z, &lights, STATIC_MESH_POINT_SCALE);
        let color = fgethsv(26, 153, 255);
        for channel in 0..3 {
            assert!((facing[channel] - color[channel] * 0.5 * 0.5).abs() < 1e-5, "{facing:?}");
        }
        assert_eq!(point_term(Vec3::ZERO, -Vec3::Z, &lights, STATIC_MESH_POINT_SCALE), [0.0; 3]);
        let actor = point_term(Vec3::ZERO, Vec3::Z, &lights, ACTOR_POINT_SCALE);
        assert!((actor[0] - facing[0] * 2.0).abs() < 1e-5);
    }

    #[test]
    fn fitted_point_term_follows_the_line_through_the_sphere_ends_and_matches_the_polynomial_there() {
        let lights = point_lights(&[lamp([0.0, 0.0, 0.0], 100.0, 255.0)]);
        let sphere = (Vec3::new(40.0, 0.0, 0.0), 10.0);
        let at = |x: f32| fitted_point_term(Vec3::new(x, 0.0, 0.0), -Vec3::X, &lights, STATIC_MESH_POINT_SCALE, sphere)[0];
        let polynomial = |x: f32| point_term(Vec3::new(x, 0.0, 0.0), -Vec3::X, &lights, STATIC_MESH_POINT_SCALE)[0];
        for end in [31.0, 49.0] {
            assert!((at(end) - polynomial(end)).abs() < 1e-4, "{end}: {} {}", at(end), polynomial(end));
        }
        assert!((at(40.0) - polynomial(40.0)).abs() > 1e-4);
        assert!(at(40.0) > at(45.0));
        let degenerate = (Vec3::new(40.0, 0.0, 0.0), 0.5);
        let value = fitted_point_term(Vec3::new(40.0, 0.0, 0.0), -Vec3::X, &lights, STATIC_MESH_POINT_SCALE, degenerate)[0];
        assert!((value - polynomial(40.0)).abs() < 1e-4);
    }

    #[test]
    fn terrain_lights_need_a_world_emitter_type_and_no_time_switch() {
        let world = Light { emitter_light_type: EMITTER_LIGHT_WORLD_ONLY, ..lamp([0.0; 3], 100.0, 70.0) };
        let all = Light { emitter_light_type: EMITTER_LIGHT_ALL, ..lamp([1.0; 3], 100.0, 70.0) };
        let pawn_only = lamp([2.0; 3], 100.0, 70.0);
        let timed = Light { time_light: true, ..world.clone() };
        let picked = terrain_lights(&[world, all, pawn_only, timed]);
        assert_eq!(picked.len(), 2);
        assert_eq!(picked[1].position, Vec3::ONE);
    }

    #[test]
    fn sector_lights_take_the_first_eight_inside_the_box_grown_by_each_radius() {
        let inside = |x: f32| PointLight { position: Vec3::new(x, 0.0, 0.0), radius: 100.0, color: [1.0; 3], brightness: 64.0, ..Default::default() };
        let lights: Vec<PointLight> = (0..10).map(|i| inside(1000.0 + 50.0 * i as f32)).chain([inside(-101.0), inside(-99.0)]).collect();
        let picked = sector_lights(&lights, Vec3::ZERO, Vec3::new(2000.0, 2000.0, 100.0));
        assert_eq!(picked.len(), MAX_HARDWARE_LIGHTS);
        assert_eq!(picked[0].position.x, 1000.0);
        let edge = sector_lights(&lights[10..], Vec3::ZERO, Vec3::new(2000.0, 2000.0, 100.0));
        assert_eq!(edge.len(), 1);
        assert_eq!(edge[0].position.x, -99.0);
    }

    #[test]
    fn set_light_1001c270_fits_inverse_falloff_and_recomputes_slope_after_clamping() {
        let light = PointLight { radius: 100.0, ..Default::default() };
        let [a0, a1] = hardware_attenuation(&light, Vec3::X * 40.0, 10.0);
        for distance in [31.0, 49.0] {
            let fit = 1.0 / (a0 + a1 * distance);
            assert!((fit - 2.0 * point_falloff(distance, light.radius)).abs() < 1e-5);
        }
        let [a0, a1] = hardware_attenuation(&light, Vec3::X * 80.0, 30.0);
        assert_eq!(a0, 0.0);
        assert!((a1 - 1.0 / (2.0 * point_falloff(51.0, 100.0) * 51.0)).abs() < 1e-6);
    }

    #[test]
    fn selection_reserves_sun_matches_special_lit_and_uses_the_actor_radius() {
        let light = PointLight { position: Vec3::new(110.0, 0.0, 0.0), radius: 100.0, brightness: 64.0, ..Default::default() };
        assert!(select(&[light], Vec3::ZERO, 0.0, false, 4, false).is_empty());
        assert_eq!(select(&[light], Vec3::ZERO, 20.0, false, 4, false), [light]);
        assert!(select(&[light], Vec3::ZERO, 20.0, true, 4, false).is_empty());
        assert_eq!(select(&[light; 10], Vec3::ZERO, 20.0, false, 4, true).len(), 3);
        assert_eq!(select(&[light; 10], Vec3::ZERO, 20.0, false, 20, false).len(), 8);
        assert!(select(&[light], Vec3::ZERO, 20.0, false, 0, true).is_empty());
    }

    #[test]
    fn software_spots_use_light_cone_but_hardware_spots_use_45_degrees() {
        for effect in [8, 12] {
            let light = PointLight { radius: 100.0, direction: Vec3::Z, effect, cone: 64, ..Default::default() };
            for (degrees, expected) in [(40.0_f32, true), (50.0, false)] {
                let a = degrees.to_radians();
                let position = Vec3::new(a.sin(), 0.0, a.cos()) * 50.0;
                assert_eq!(hardware_cone(&light, position), expected);
                assert_eq!(hardware_cone(&PointLight { cone: 255, ..light }, position), expected);
            }
            assert!(sample_intensity(&light, Vec3::Z * 50.0, -Vec3::Z) > 0.0);
            assert_eq!(sample_intensity(&light, Vec3::X * 50.0, -Vec3::X), 0.0);
        }
    }

    #[test]
    fn fun_10648630_selects_four_by_priority() {
        let lamps: Vec<Light> = (0..10).map(|i| lamp([0.0, 0.0, 10.0 * (10 - i) as f32], 200.0, 64.0)).collect();
        let lights = point_lights(&lamps);
        let picked = nearest(&lights, Vec3::ZERO);
        assert_eq!(picked.len(), 4);
        assert_eq!(picked[0].position.z, 10.0);
        assert_eq!(picked[3].position.z, 40.0);
        assert!(nearest(&lights, Vec3::new(0.0, 1000.0, 0.0)).is_empty());
    }
}
