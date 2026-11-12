use crate::Placement;
use ue2_assets::{Property, find};
use ue2_package::{ObjectRef, Package};

#[derive(Debug, Clone, PartialEq)]
pub struct Projector {
    pub name: String,
    pub placement: Placement,
    pub draw_scale: f32,
    pub physics: u8,
    pub texture: Option<String>,
    pub gradient_texture: Option<String>,
    pub material_blend: u8,
    pub frame_blend: u8,
    pub fov: i32,
    pub max_trace_distance: i32,
    pub radius: f32,
    pub flags: u32,
    pub project_tag: Option<String>,
    pub dynamic_attach: bool,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Visibility {
    pub only_owner: bool,
    pub owner_no_see: bool,
    pub high_detail: bool,
    pub super_high_detail: bool,
}

impl Visibility {
    pub fn visible(self, owned: bool, behind_view: bool, wireframe: bool, detail: u8) -> bool {
        !wireframe
            && if owned { !self.owner_no_see || behind_view } else { !self.only_owner }
            && (!self.high_detail || detail != 0)
            && (!self.super_high_detail || detail == 2)
    }
}

pub const BSP: u32 = 1;
pub const TERRAIN: u32 = 2;
pub const STATIC_MESH: u32 = 4;
pub const PARTICLES: u32 = 8;
pub const ACTOR: u32 = 0x10;
pub const LEVEL_STATIC: u32 = 0x20;
pub const CLIP_BSP: u32 = 0x40;
pub const CLIP_STATIC_MESH: u32 = 0x80;
pub const ON_UNLIT: u32 = 0x100;
pub const GRADIENT: u32 = 0x200;
pub const ON_BACKFACES: u32 = 0x400;
pub const ON_ALPHA: u32 = 0x800;
pub const ON_PARALLEL_BSP: u32 = 0x1000;

impl Default for Projector {
    fn default() -> Self {
        Self {
            name: String::new(),
            placement: Placement { location: [0.0; 3], rotation: [0; 3] },
            draw_scale: 1.0,
            physics: 0,
            texture: None,
            gradient_texture: Some("Engine.GRADIENT_Fade".into()),
            material_blend: 0,
            frame_blend: 1,
            fov: 0,
            max_trace_distance: 1000,
            radius: -1.0,
            flags: BSP | TERRAIN | STATIC_MESH | PARTICLES | ACTOR,
            project_tag: None,
            dynamic_attach: false,
            visibility: Visibility::default(),
        }
    }
}

pub(crate) fn read(
    package: &Package,
    properties: &[Property<'_>],
    package_name: Option<&str>,
    class: &str,
    name: &str,
) -> Option<Projector> {
    if !class.eq_ignore_ascii_case("Projector") && !class.eq_ignore_ascii_case("DynamicProjector") {
        return None;
    }
    let get = |name| find(properties, name);
    let object = |name| {
        get(name).and_then(|p| p.object(package)).and_then(|object| match object {
            ObjectRef::Null => None,
            ObjectRef::Export(index) if package_name.is_some() => {
                package_name.map(|name| format!("{name}.{}#{index}", package.object_path(object)))
            }
            _ => Some(package.object_path(object)),
        })
    };
    let mut p = Projector::default();
    p.name = name.to_owned();
    p.placement = crate::placement(properties);
    p.draw_scale = get("DrawScale").and_then(Property::float).unwrap_or(p.draw_scale);
    p.physics = get("Physics").and_then(Property::byte).unwrap_or(p.physics);
    p.texture = object("ProjTexture");
    if get("GradientTexture").is_some() {
        p.gradient_texture = object("GradientTexture");
    }
    p.material_blend = get("MaterialBlendingOp").and_then(Property::byte).unwrap_or(p.material_blend);
    p.frame_blend = get("FrameBufferBlendingOp").and_then(Property::byte).unwrap_or(p.frame_blend);
    p.fov = get("FOV").and_then(Property::int).unwrap_or(p.fov);
    p.max_trace_distance = get("MaxTraceDistance").and_then(Property::int).unwrap_or(p.max_trace_distance);
    p.radius = get("ProjectorRadius").and_then(Property::float).unwrap_or(p.radius);
    p.project_tag = get("ProjectTag")
        .and_then(|p| p.name_value(package))
        .filter(|s| !s.eq_ignore_ascii_case("None"))
        .map(str::to_owned);
    p.dynamic_attach =
        get("bDynamicAttach").and_then(Property::bool).unwrap_or(class.eq_ignore_ascii_case("DynamicProjector"));
    p.visibility = Visibility {
        only_owner: get("bOnlyOwnerSee").and_then(Property::bool).unwrap_or(false),
        owner_no_see: get("bOwnerNoSee").and_then(Property::bool).unwrap_or(false),
        high_detail: get("bHighDetail").and_then(Property::bool).unwrap_or(false),
        super_high_detail: get("bSuperHighDetail").and_then(Property::bool).unwrap_or(false),
    };
    for (name, bit) in [
        ("bProjectBSP", BSP),
        ("bProjectTerrain", TERRAIN),
        ("bProjectStaticMesh", STATIC_MESH),
        ("bProjectParticles", PARTICLES),
        ("bProjectActor", ACTOR),
        ("bLevelStatic", LEVEL_STATIC),
        ("bClipBSP", CLIP_BSP),
        ("bClipStaticMesh", CLIP_STATIC_MESH),
        ("bProjectOnUnlit", ON_UNLIT),
        ("bGradient", GRADIENT),
        ("bProjectOnBackfaces", ON_BACKFACES),
        ("bProjectOnAlpha", ON_ALPHA),
        ("bProjectOnParallelBSP", ON_PARALLEL_BSP),
    ] {
        if let Some(value) = get(name).and_then(Property::bool) {
            p.flags = (p.flags & !bit) | if value { bit } else { 0 };
        }
    }
    Some(p)
}
