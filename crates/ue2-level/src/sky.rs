use ue2_assets::{Property, find};
use ue2_package::{ObjectRef, Package};

use crate::placement;

pub const LENS_FLARE_SLOTS: usize = 12;
const SUN_DEFAULT_RADIUS: f32 = 32768.0;
const BODY_DEFAULT_SCALE: f32 = 1.0;

/// `SkyZoneInfo`: the sky box's origin and the lens flare table `RenderLensFlare` reads.
#[derive(Debug, Clone, PartialEq)]
pub struct SkyZoneInfo {
    pub location: [f32; 3],
    pub lens_flare: Vec<Option<String>>,
    pub lens_flare_offset: Vec<f32>,
    pub lens_flare_scale: Vec<f32>,
}

impl SkyZoneInfo {
    pub fn flares(&self) -> impl Iterator<Item = (&str, f32, f32)> + '_ {
        (0..LENS_FLARE_SLOTS).filter_map(|i| {
            let texture = self.lens_flare.get(i)?.as_deref()?;
            Some((texture, self.lens_flare_offset.get(i).copied().unwrap_or(0.0), self.lens_flare_scale.get(i).copied().unwrap_or(0.0)))
        })
    }
}

/// `NSun` / `NMoon`: a sprite at `Radius` from the sky zone, textured by `Skins[0]`.
#[derive(Debug, Clone, PartialEq)]
pub struct SkyBody {
    pub location: [f32; 3],
    pub radius: f32,
    pub skin: Option<String>,
    pub scale: f32,
    /// Weather the moon belongs to (`NMoon.EnvType`); 0 for the sun.
    pub env_type: u8,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SkyActors {
    pub zone: Option<SkyZoneInfo>,
    pub sun: Option<SkyBody>,
    pub moons: Vec<SkyBody>,
}

fn fixed_array<T>(properties: &[Property<'_>], name: &str, slots: usize, read: impl Fn(&Property<'_>) -> Option<T>) -> Vec<Option<T>> {
    let mut out: Vec<Option<T>> = (0..slots).map(|_| None).collect();
    for property in properties.iter().filter(|property| property.name.eq_ignore_ascii_case(name)) {
        if let Some(slot) = out.get_mut(property.index as usize) {
            *slot = read(property);
        }
    }
    out
}

pub(crate) fn sky_zone(package: &Package, properties: &[Property<'_>]) -> SkyZoneInfo {
    let path = |property: &Property<'_>| match property.object(package)? {
        ObjectRef::Null => None,
        object => Some(package.object_path(object)),
    };
    SkyZoneInfo {
        location: placement(properties).location,
        lens_flare: fixed_array(properties, "LensFlare", LENS_FLARE_SLOTS, path),
        lens_flare_offset: fixed_array(properties, "LensFlareOffset", LENS_FLARE_SLOTS, |p| p.float()).into_iter().map(Option::unwrap_or_default).collect(),
        lens_flare_scale: fixed_array(properties, "LensFlareScale", LENS_FLARE_SLOTS, |p| p.float()).into_iter().map(Option::unwrap_or_default).collect(),
    }
}

pub(crate) fn sky_body(package: &Package, properties: &[Property<'_>], moon: bool) -> SkyBody {
    let float = |name: &str, default: f32| find(properties, name).and_then(Property::float).unwrap_or(default);
    let skin = find(properties, "Skins")
        .and_then(|skins| skins.objects(package))
        .and_then(|skins| skins.into_iter().next())
        .filter(|skin| !matches!(skin, ObjectRef::Null))
        .map(|skin| package.object_path(skin));
    SkyBody {
        location: placement(properties).location,
        radius: float("Radius", SUN_DEFAULT_RADIUS),
        skin,
        scale: float(if moon { "MoonScale" } else { "SunScale" }, BODY_DEFAULT_SCALE),
        env_type: if moon { find(properties, "EnvType").and_then(|p| p.byte().or_else(|| p.int().and_then(|v| u8::try_from(v).ok()))).unwrap_or(0) } else { 0 },
    }
}
