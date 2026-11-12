use ue2_assets::{Property, find};
use ue2_package::{ObjectRef, Package};

use crate::Range;
use crate::emitters::range_of;
use crate::placement;

pub const FOG_RANGE_LEVELS: usize = 5;

/// One `L2EnvironmentColorInfo`: the colors an `L2FogInfo` imposes at `time`.
#[derive(Debug, Clone, PartialEq)]
pub struct FogColorInfo {
    pub time: f32,
    pub fog: [u8; 4],
    pub sky: [u8; 4],
    pub cloud: Vec<[u8; 4]>,
    pub haze_ring: Vec<[u8; 4]>,
}

/// `L2FogInfo`: a local fog that blends over the global one by distance to its location.
#[derive(Debug, Clone, PartialEq)]
pub struct L2FogInfo {
    pub location: [f32; 3],
    pub affect_range: Range,
    /// `FogRange1..5`, already in world units, picked by `ClippingRange.Terrain`.
    pub fog_range: [Range; FOG_RANGE_LEVELS],
    pub texture_distance: f32,
    pub colors: Vec<FogColorInfo>,
    pub cloud_texture: Option<String>,
}

pub(crate) fn read(package: &Package, properties: &[Property<'_>]) -> L2FogInfo {
    let range = |name: &str| find(properties, name).map_or([0.0; 2], |range| range_of(package, range));
    let mut fog_range = [[0.0; 2]; FOG_RANGE_LEVELS];
    for (index, slot) in fog_range.iter_mut().enumerate() {
        *slot = range(&format!("FogRange{}", index + 1));
    }
    let colors = find(properties, "Colors")
        .and_then(|colors| colors.structs(package))
        .unwrap_or_default()
        .iter()
        .map(|fields| FogColorInfo {
            time: find(fields, "Time").and_then(Property::float).unwrap_or(0.0),
            fog: find(fields, "FogColor").and_then(Property::color).unwrap_or([0, 0, 0, 255]),
            sky: find(fields, "SkyColor").and_then(Property::color).unwrap_or([0, 0, 0, 255]),
            cloud: find(fields, "CloudColor").and_then(Property::colors).unwrap_or_default(),
            haze_ring: find(fields, "HazeRingColor").and_then(Property::colors).unwrap_or_default(),
        })
        .collect();
    L2FogInfo {
        location: placement(properties).location,
        affect_range: range("AffectRange"),
        fog_range,
        texture_distance: find(properties, "TextureDistance").and_then(Property::float).unwrap_or(0.0),
        colors,
        cloud_texture: find(properties, "CloudTexture").and_then(|p| p.object(package)).and_then(|object| match object {
            ObjectRef::Null => None,
            object => Some(package.object_path(object)),
        }),
    }
}
