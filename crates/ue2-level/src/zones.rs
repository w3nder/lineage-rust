use ue2_assets::{Property, find};
use ue2_package::Package;

use crate::{Fog, fog, placement};

const ZONE_DEFAULT_AMBIENT_SATURATION: u8 = 255;

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneProperties {
    pub terrains: Vec<String>,
    pub ambient_brightness: u8,
    pub ambient_hue: u8,
    pub ambient_saturation: u8,
    pub ambient_vector: Option<[f32; 3]>,
    pub sun_affect: bool,
    pub distance_fog: Option<Fog>,
    pub is_sky: bool,
    pub terrain_zone: bool,
    pub location: [f32; 3],
    /// `bClearToFogColor`: `RenderLevel` clears the frame to `DistanceFogColor`.
    pub clear_to_fog_color: bool,
}

impl ZoneProperties {
    pub fn read(package: &Package, class: &str, properties: &[Property<'_>]) -> Self {
        let get = |name: &str| find(properties, name);
        Self {
            terrains: get("Terrains").and_then(|p| p.objects(package)).unwrap_or_default().into_iter().map(|object| package.object_name(object).to_owned()).collect(),
            ambient_brightness: get("AmbientBrightness").and_then(Property::byte).unwrap_or(0),
            ambient_hue: get("AmbientHue").and_then(Property::byte).unwrap_or(0),
            ambient_saturation: get("AmbientSaturation").and_then(Property::byte).unwrap_or(ZONE_DEFAULT_AMBIENT_SATURATION),
            ambient_vector: get("AmbientVector").and_then(Property::vector),
            sun_affect: get("bSunAffect").and_then(Property::bool).unwrap_or(true),
            distance_fog: fog(properties),
            is_sky: class.eq_ignore_ascii_case("SkyZoneInfo"),
            terrain_zone: get("bTerrainZone").and_then(Property::bool).unwrap_or(false),
            location: placement(properties).location,
            clear_to_fog_color: get("bClearToFogColor").and_then(Property::bool).unwrap_or(false),
        }
    }
}

pub(crate) fn is_zone_info(class: &str) -> bool {
    class.to_ascii_lowercase().ends_with("zoneinfo") || class.eq_ignore_ascii_case("LevelInfo")
}
