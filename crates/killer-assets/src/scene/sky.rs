use super::{Scene, build_scene_with, resolve_material, texture_material};
use crate::client::Client;
use crate::options::{ClientOptions, SkyNames};
use l2_catalog::{BlendFactor, FogMode, Material};
use ue2_level::{SkyActors, SkyBody};

pub const SHARED_SKY_LEVEL: &str = "skylevel";
/// `Engine.S_Actor`, the default `Actor.Texture` whose `USize` sizes the sun and moon quads.
pub const SPRITE_TEXTURE_SIZE: f32 = 32.0;
/// `ANSun::Tick`: `DrawScale = scale · SunScale · 8`.
pub const SUN_DRAW_SCALE: f32 = 8.0;
pub const MOON_DRAW_SCALE: f32 = 1.0;

/// Which per-hour table `UpdateSkyColors` writes into a material's `ColorModifier`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkyTint {
    SkyBox,
    Hazering,
    Cloud(usize),
}

fn object_name(path: &str) -> &str {
    path.rsplit('.').next().unwrap_or(path)
}

pub fn sky_tint(material: &Material, names: &SkyNames) -> Option<SkyTint> {
    let modifier = object_name(material.color_modifier.as_deref()?);
    if !names.skybox.is_empty() && modifier.eq_ignore_ascii_case(&names.skybox) {
        return Some(SkyTint::SkyBox);
    }
    if !names.hazering.is_empty() && modifier.eq_ignore_ascii_case(&names.hazering) {
        return Some(SkyTint::Hazering);
    }
    names.clouds.iter().position(|cloud| !cloud.is_empty() && modifier.eq_ignore_ascii_case(cloud)).map(SkyTint::Cloud)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyKind {
    Sun,
    Moon,
}

/// A sun or moon sprite: a camera-facing quad the renderer places from the hour every frame.
#[derive(Clone, Debug, PartialEq)]
pub struct SkyBodyDraw {
    pub kind: BodyKind,
    pub env_type: u8,
    /// `SkyZoneInfo.Location`, the origin the quad orbits.
    pub origin: [f32; 3],
    pub radius: f32,
    /// `NSun.SunScale · 8 · 16` or `NMoon.MoonScale · 16`: half width per unit of the hour's scale.
    pub half_width_per_scale: f32,
    pub material: Material,
    pub zone: u8,
}

/// One `LensFlare[i]` slot of the `SkyZoneInfo`.
#[derive(Clone, Debug, PartialEq)]
pub struct Flare {
    pub texture: String,
    pub offset: f32,
    pub scale: f32,
}

/// `FinalBlend { TwoSided, FB_Brighten, ZWrite 0, ZTest 1, AlphaTest 0 }` over `Skins[0]`, lit only by the vertex color (`EnableLighting(0, 1, 0, 0, 0)`).
pub fn body_material(skin: &str) -> Material {
    body_material_over(texture_material(skin))
}

pub fn body_material_over(mut material: Material) -> Material {
    if let Some(pass) = material.passes.first_mut() {
        pass.src = BlendFactor::SrcAlpha;
        pass.dst = BlendFactor::One;
        pass.alpha_blend = true;
        pass.z_write = false;
        pass.z_test = true;
        pass.two_sided = true;
        pass.fog = FogMode::Black;
    }
    material.sorting_level = 2;
    material
}

/// `FinalBlend { TwoSided, FB_Translucent, ZWrite 0, ZTest 0 }` over a flare texture, white.
pub fn flare_material(texture: &str) -> Material {
    let mut material = texture_material(texture);
    if let Some(pass) = material.passes.first_mut() {
        pass.src = BlendFactor::One;
        pass.dst = BlendFactor::InvSrcColor;
        pass.alpha_blend = true;
        pass.z_write = false;
        pass.z_test = false;
        pass.two_sided = true;
        pass.fog = FogMode::Black;
    }
    material.sorting_level = 2;
    material
}

fn body_draw(client: &mut Client, body: &SkyBody, kind: BodyKind, origin: [f32; 3], zone: u8) -> Option<SkyBodyDraw> {
    let skin = body.skin.as_deref()?;
    let draw_scale = match kind {
        BodyKind::Sun => SUN_DRAW_SCALE,
        BodyKind::Moon => MOON_DRAW_SCALE,
    };
    Some(SkyBodyDraw {
        kind,
        env_type: body.env_type,
        origin,
        radius: body.radius,
        half_width_per_scale: body.scale * draw_scale * SPRITE_TEXTURE_SIZE / 2.0,
        material: body_material_over(resolve_material(client, skin)),
        zone,
    })
}

pub fn sky_bodies(client: &mut Client, sky: &SkyActors, zone: u8) -> Vec<SkyBodyDraw> {
    let Some(info) = &sky.zone else { return Vec::new() };
    let mut bodies = Vec::new();
    if let Some(sun) = sky.sun.as_ref().and_then(|sun| body_draw(client, sun, BodyKind::Sun, info.location, zone)) {
        bodies.push(sun);
    }
    for moon in &sky.moons {
        bodies.extend(body_draw(client, moon, BodyKind::Moon, info.location, zone));
    }
    bodies
}

pub fn flares(sky: &SkyActors) -> Vec<Flare> {
    sky.zone.as_ref().map_or_else(Vec::new, |info| {
        info.flares().map(|(texture, offset, scale)| Flare { texture: texture.to_owned(), offset, scale }).collect()
    })
}

/// The scene of `maps/skylevel.unr`, the one sky every tile shares when `SharedSky` is on.
pub fn load_shared(client: &mut Client, options: &ClientOptions) -> Option<Scene> {
    let loaded = client.load_level(SHARED_SKY_LEVEL).ok()?;
    let mut scene = build_scene_with(client, &loaded.level, &ClientOptions { shared_sky: false, ..options.clone() }, super::Neighbours::default());
    let zones = scene.zones.clone();
    scene.retain_draws(|draw| zones.get(usize::from(draw.zone)).is_some_and(|zone| zone.is_sky));
    Some(scene)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> SkyNames {
        SkyNames { skybox: "SkybackgroundColor".into(), hazering: "HazeRing_Final".into(), clouds: vec!["Cloud_Final".into(), "StarField_Final01".into(), "StarField_Final02".into()] }
    }

    #[test]
    fn tints_are_matched_by_the_color_modifiers_object_name() {
        let mut material = texture_material("L2_Skies.Textures.WhiteChip");
        assert_eq!(sky_tint(&material, &names()), None);
        material.color_modifier = Some("L2_Skies.Shaders.SkybackgroundColor".into());
        assert_eq!(sky_tint(&material, &names()), Some(SkyTint::SkyBox));
        material.color_modifier = Some("L2_Skies.Shaders.hazering_final".into());
        assert_eq!(sky_tint(&material, &names()), Some(SkyTint::Hazering));
        material.color_modifier = Some("L2_Skies.Shaders.StarField_Final02".into());
        assert_eq!(sky_tint(&material, &names()), Some(SkyTint::Cloud(2)));
        material.color_modifier = Some("L2_Skies.Shaders.Other".into());
        assert_eq!(sky_tint(&material, &names()), None);
    }

    #[test]
    fn sun_and_moons_take_their_half_width_from_the_draw_scale_rule() {
        let zone = ue2_level::SkyZoneInfo { location: [1.0, 2.0, 3.0], lens_flare: vec![None; 12], lens_flare_offset: vec![0.0; 12], lens_flare_scale: vec![0.0; 12] };
        let sun = SkyBody { location: [0.0; 3], radius: 350.0, skin: Some("L2_Skies.FlareSet.Sun01".into()), scale: 1.0, env_type: 0 };
        let moon = SkyBody { location: [0.0; 3], radius: 245.0, skin: Some("L2_Skies.moon.moon".into()), scale: 1.0, env_type: 1 };
        let bare = SkyBody { skin: None, ..moon.clone() };
        let sky = SkyActors { zone: Some(zone), sun: Some(sun), moons: vec![moon, bare] };
        let mut client = Client::open(&std::env::temp_dir()).unwrap();
        let bodies = sky_bodies(&mut client, &sky, 5);
        assert_eq!(bodies.len(), 2);
        assert_eq!((bodies[0].kind, bodies[0].radius, bodies[0].half_width_per_scale), (BodyKind::Sun, 350.0, 128.0));
        assert_eq!((bodies[1].kind, bodies[1].radius, bodies[1].half_width_per_scale, bodies[1].env_type), (BodyKind::Moon, 245.0, 16.0, 1));
        assert_eq!(bodies[0].origin, [1.0, 2.0, 3.0]);
        let pass = &bodies[0].material.passes[0];
        assert_eq!((pass.src, pass.dst, pass.z_write, pass.z_test, pass.two_sided), (BlendFactor::SrcAlpha, BlendFactor::One, false, true, true));
        assert!(sky_bodies(&mut client, &SkyActors::default(), 0).is_empty());
    }
}
