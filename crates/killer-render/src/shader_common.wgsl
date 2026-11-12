struct Camera {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    eye: vec4<f32>,
};

struct ZoneUniform {
    sun: vec4<f32>,
};

struct Environment {
    sun_dir: vec4<f32>,
    sun: vec4<f32>,
    static_mesh_ambient: vec4<f32>,
    bsp_ambient: vec4<f32>,
    terrain_ambient: vec4<f32>,
    actor_ambient: vec4<f32>,
    terrain_sun: vec4<f32>,
    hour: vec4<f32>,
    fog_color: vec4<f32>,
    fog_range: vec4<f32>,
};

struct StageGpu {
    color_op: u32,
    alpha_op: u32,
    flags: u32,
    frame: u32,
};

struct Material {
    ambient: vec4<f32>,
    stages: array<StageGpu, 8>,
    uv_u: array<vec4<f32>, 8>,
    uv_v: array<vec4<f32>, 8>,
    tfactor: vec4<f32>,
    alpha_ref: f32,
    alpha_test: u32,
    fog_mode: u32,
    stage_count: u32,
    kind: u32,
    sun_affect: u32,
    zone: u32,
    lit: u32,
    light_after: u32,
    has_mask: u32,
    lightmap_slices: u32,
    modulate_2x: u32,
    fade: vec2<f32>,
    padding: vec2<u32>,
    projection: array<mat4x4<f32>, 8>,
};

const KIND_STATIC_MESH: u32 = 0u;
const KIND_BSP: u32 = 1u;
const KIND_TERRAIN: u32 = 2u;
const KIND_ACTOR: u32 = 3u;
const KIND_SPRITE: u32 = 4u;
const MODULATE_2X: f32 = 2.0;
const STATIC_MESH_LIGHT_HALF: f32 = 0.5;
const D3D_SUN_SCALE: f32 = 0.875;
const BSP_LIGHTMAP: f32 = 0.5;
const DEFAULT_TEX_MIP_BIAS: f32 = -0.5;

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> environment: Environment;
@group(0) @binding(2) var<storage, read> zones: array<ZoneUniform>;
@group(1) @binding(0) var stage_texture0: texture_2d_array<f32>;
@group(1) @binding(1) var stage_sampler0: sampler;
@group(1) @binding(2) var stage_texture1: texture_2d_array<f32>;
@group(1) @binding(3) var stage_sampler1: sampler;
@group(1) @binding(4) var stage_texture2: texture_2d_array<f32>;
@group(1) @binding(5) var stage_sampler2: sampler;
@group(1) @binding(6) var cube_texture: texture_cube<f32>;
@group(1) @binding(7) var cube_sampler: sampler;
@group(1) @binding(8) var mask_texture: texture_2d<f32>;
@group(1) @binding(9) var mask_sampler: sampler;
@group(1) @binding(10) var<uniform> material: Material;
@group(1) @binding(11) var lightmap_texture: texture_2d_array<f32>;
@group(1) @binding(12) var lightmap_sampler: sampler;
@group(1) @binding(13) var stage_texture3: texture_2d_array<f32>;
@group(1) @binding(14) var stage_sampler3: sampler;
@group(1) @binding(15) var stage_texture4: texture_2d_array<f32>;
@group(1) @binding(16) var stage_sampler4: sampler;
@group(1) @binding(17) var stage_texture5: texture_2d_array<f32>;
@group(1) @binding(18) var stage_sampler5: sampler;
@group(1) @binding(19) var stage_texture6: texture_2d_array<f32>;
@group(1) @binding(20) var stage_sampler6: sampler;
@group(1) @binding(21) var stage_texture7: texture_2d_array<f32>;
@group(1) @binding(22) var stage_sampler7: sampler;

struct VertexOutput {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) mask_uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) normal: vec3<f32>,
    @location(4) world_pos: vec3<f32>,
    @location(5) actor_fade: f32,
};

fn sun_term(normal: vec3<f32>) -> vec3<f32> {
    let lambert = max(dot(normalize(normal), -environment.sun_dir.xyz), 0.0);
    return environment.sun.rgb * environment.sun.a * lambert;
}

const HOURS_PER_DAY: f32 = 24.0;

fn lightmap_slice() -> u32 {
    let slices = material.lightmap_slices;
    return u32(trunc(environment.hour.x * f32(slices) / HOURS_PER_DAY)) % slices;
}

fn bsp_light(lightmap_uv: vec2<f32>) -> vec3<f32> {
    if (material.lightmap_slices == 0u) {
        return vec3<f32>(BSP_LIGHTMAP);
    }
    return textureSample(lightmap_texture, lightmap_sampler, lightmap_uv, lightmap_slice()).rgb;
}

const INTENSITY_SLICE_HOURS: f32 = 3.0;
const INTENSITY_FIRST_HOUR: f32 = 1.5;

fn terrain_intensity(mask_uv: vec2<f32>, vertex_alpha: f32) -> f32 {
    let slices = material.lightmap_slices;
    if (slices == 0u) {
        return vertex_alpha;
    }
    let dims = vec2<f32>(textureDimensions(lightmap_texture));
    let uv = mask_uv * (dims - 1.0) / dims;
    let hour = environment.hour.x - floor(environment.hour.x / HOURS_PER_DAY) * HOURS_PER_DAY;
    var first = slices - 1u;
    if (hour >= INTENSITY_FIRST_HOUR) {
        first = min(u32((hour - INTENSITY_FIRST_HOUR) / INTENSITY_SLICE_HOURS), slices - 1u);
    }
    let second = (first + 1u) % slices;
    let start = INTENSITY_FIRST_HOUR + INTENSITY_SLICE_HOURS * f32(first);
    var shifted = hour;
    if (hour < start) {
        shifted = hour + HOURS_PER_DAY;
    }
    let fraction = (shifted - start) / INTENSITY_SLICE_HOURS;
    let a = textureSampleLevel(lightmap_texture, lightmap_sampler, uv, first, 0.0).r * 255.0;
    let b = textureSampleLevel(lightmap_texture, lightmap_sampler, uv, second, 0.0).r * 255.0;
    return trunc(a * (1.0 - fraction) + b * fraction + 1e-3) / 255.0;
}

fn light_factor(normal: vec3<f32>, color: vec4<f32>, lightmap_uv: vec2<f32>) -> vec3<f32> {
    let sun_affect = f32(object_sun_affect());
    switch material.kind {
        case KIND_BSP: {
            let lm = bsp_light(lightmap_uv);
            if (material.modulate_2x == 0u) {
                return lm;
            }
            let tint = mix(vec3<f32>(1.0), MODULATE_2X * environment.bsp_ambient.rgb, sun_affect);
            return tint * MODULATE_2X * lm;
        }
        case KIND_TERRAIN: {
            let sun = environment.terrain_sun.rgb * environment.terrain_sun.a * terrain_intensity(lightmap_uv, color.a);
            let d = color.rgb + 0.5 * sun + 0.5 * environment.terrain_ambient.rgb;
            return MODULATE_2X * clamp(d, vec3<f32>(0.0), vec3<f32>(1.0));
        }
        case KIND_ACTOR: {
            let ambient = select(object_ambient(), environment.actor_ambient.rgb * 0.5, sun_affect != 0.0);
            let d = ambient + sun_term(normal) * D3D_SUN_SCALE * object_sun_visible() + color.rgb;
            return MODULATE_2X * clamp(d, vec3<f32>(0.0), vec3<f32>(1.0));
        }
        case KIND_SPRITE: {
            return color.rgb;
        }
        case 5u, 6u: {
            return 2.0 * color.rgb;
        }
        case KIND_STATIC_MESH, default: {
            if (material.modulate_2x == 0u) {
                return vec3<f32>(1.0);
            }
            let ambient = select(material.ambient.rgb, environment.static_mesh_ambient.rgb * 0.5, sun_affect != 0.0);
            let sun = sun_term(normal) * STATIC_MESH_LIGHT_HALF * D3D_SUN_SCALE * zones[object_zone()].sun.x;
            let d = ambient + sun + color.rgb;
            return MODULATE_2X * clamp(d, vec3<f32>(0.0), vec3<f32>(1.0));
        }
    }
}

fn fog_target() -> vec4<f32> {
    if (material.padding.x != 0u) {
        return vec4<f32>(0.0);
    }
    switch material.fog_mode {
        case 1u: {
            return vec4<f32>(0.0, 0.0, 0.0, environment.fog_color.a);
        }
        case 2u: {
            return vec4<f32>(vec3<f32>(127.0 / 255.0), environment.fog_color.a);
        }
        default: {
            return environment.fog_color;
        }
    }
}

fn distance_fog(color: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let fog_to = fog_target();
    if (fog_to.a == 0.0 || environment.fog_range.y <= environment.fog_range.x) {
        return color;
    }
    let depth = (camera.view * vec4<f32>(world_pos, 1.0)).z;
    let fog = clamp((depth - environment.fog_range.x) / (environment.fog_range.y - environment.fog_range.x), 0.0, 1.0);
    return mix(color, fog_to.rgb, fog);
}
