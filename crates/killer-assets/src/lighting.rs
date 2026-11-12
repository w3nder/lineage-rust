use crate::scene::transform::rotator_matrix;
use glam::Vec3;
use ue2_assets::{Property, find, object_properties};
use ue2_package::Package;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneLighting {
    pub sun_dir: Vec3,
    pub sun_brightness: f32,
    pub ambient: f32,
}

impl Default for SceneLighting {
    fn default() -> Self {
        Self { sun_dir: Vec3::Z, sun_brightness: 0.0, ambient: 0.5 }
    }
}

pub fn sun_direction(rotation: [i32; 3]) -> Vec3 {
    (rotator_matrix(rotation) * Vec3::X).normalize_or(Vec3::Z)
}

pub fn brightness(byte: f32) -> f32 {
    (byte / 255.0).clamp(0.0, 4.0)
}

impl SceneLighting {
    pub fn read(package: &Package, file: &[u8]) -> Self {
        let mut lighting = Self::default();
        let mut zone_found = false;
        for export in package.exports() {
            let class = package.class_name(export);
            let is_sun = class.eq_ignore_ascii_case("NMovableSunLight");
            let is_zone = class.eq_ignore_ascii_case("ZoneInfo");
            if !is_sun && !is_zone {
                continue;
            }
            let Ok(props) = object_properties(package, file, export) else { continue };
            if is_sun && lighting.sun_brightness == 0.0 {
                let rotation = find(&props, "Rotation")
                    .and_then(Property::rotator)
                    .or_else(|| find(&props, "SwayRotationOrig").and_then(Property::rotator))
                    .unwrap_or_default();
                let value =
                    find(&props, "LightBrightness").and_then(|p| p.float().or_else(|| p.byte().map(f32::from))).unwrap_or(0.0);
                if value > 0.0 {
                    lighting.sun_dir = sun_direction(rotation);
                    lighting.sun_brightness = brightness(value);
                }
            }
            if is_zone
                && !zone_found
                && find(&props, "bTerrainZone").and_then(Property::bool).unwrap_or(false)
                && let Some(value) = find(&props, "AmbientBrightness").and_then(|p| p.byte().map(f32::from).or_else(|| p.float()))
            {
                lighting.ambient = brightness(value);
                zone_found = true;
            }
        }
        lighting
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn sun_direction_is_the_local_x_axis_of_the_rotation() {
        let d = sun_direction([0, 16384, 0]);
        assert!((d - Vec3::Y).length() < 1e-3, "{d:?}");
        let d = sun_direction([-16384, 0, 0]);
        assert!((d - (-Vec3::Z)).length() < 1e-3, "{d:?}");
    }

    #[test]
    fn brightness_bytes_scale_to_unit() {
        assert!((brightness(70.0) - 70.0 / 255.0).abs() < 1e-6);
        assert!((brightness(50.0) - 50.0 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn defaults_without_actors() {
        let l = SceneLighting::default();
        assert_eq!(l.sun_brightness, 0.0);
        assert_eq!(l.ambient, 0.5);
    }
}
