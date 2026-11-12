const MAX_HARDWARE_LIGHTS: u32 = 8u;
const ACTOR_POINT_SCALE: f32 = 0.5;

struct Object {
    world: mat4x4<f32>,
    normal0: vec4<f32>,
    normal1: vec4<f32>,
    normal2: vec4<f32>,
    palette_offset: u32,
    bone_count: u32,
    zone: u32,
    sun_affect: u32,
    light_count: u32,
    sun_visible: u32,
    pad1: u32,
    pad2: u32,
    light_position: array<vec4<f32>, MAX_HARDWARE_LIGHTS>,
    light_color: array<vec4<f32>, MAX_HARDWARE_LIGHTS>,
    light_direction: array<vec4<f32>, MAX_HARDWARE_LIGHTS>,
    light_attenuation: array<vec4<f32>, MAX_HARDWARE_LIGHTS>,
    ambient: vec4<f32>,
    overlay: vec4<f32>,
};

@group(2) @binding(0) var<uniform> object: Object;
@group(2) @binding(1) var<storage, read> palette: array<mat4x4<f32>>;

struct SkinnedInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) bones: vec4<u32>,
    @location(4) weights: vec4<f32>,
};

fn bone_matrix(index: u32) -> mat4x4<f32> {
    return palette[object.palette_offset + min(index, object.bone_count - 1u)];
}

fn rotation_of(m: mat4x4<f32>) -> mat3x3<f32> {
    return mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);
}

fn point_lights(world_pos: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    var sum = vec3<f32>(0.0);
    for (var i = 0u; i < object.light_count; i = i + 1u) {
        let light = object.light_position[i];
        let offset = light.xyz - world_pos;
        let distance = length(offset);
        if (distance >= light.w) {
            continue;
        }
        let direction = object.light_direction[i];
        if (direction.w != 0.0 && dot(-offset / max(distance, 1e-6), direction.xyz) <= 0.7071067811865476) {
            continue;
        }
        let lambert = max(dot(normal, offset / max(distance, 1e-6)), 0.0);
        let attenuation = object.light_attenuation[i];
        sum = sum + object.light_color[i].rgb * (ACTOR_POINT_SCALE * lambert / (attenuation.x + attenuation.y * distance));
    }
    return sum;
}

@vertex
fn vs_main(input: SkinnedInput) -> VertexOutput {
    let m0 = bone_matrix(input.bones.x);
    let m1 = bone_matrix(input.bones.y);
    let m2 = bone_matrix(input.bones.z);
    let m3 = bone_matrix(input.bones.w);
    let p = vec4<f32>(input.position, 1.0);
    let w = input.weights;
    let skinned = w.x * (m0 * p) + w.y * (m1 * p) + w.z * (m2 * p) + w.w * (m3 * p);
    let skinned_normal = w.x * (rotation_of(m0) * input.normal) + w.y * (rotation_of(m1) * input.normal) + w.z * (rotation_of(m2) * input.normal) + w.w * (rotation_of(m3) * input.normal);
    let world_pos = object.world * vec4<f32>(skinned.xyz, 1.0);
    let normal_matrix = mat3x3<f32>(object.normal0.xyz, object.normal1.xyz, object.normal2.xyz);
    var out: VertexOutput;
    out.clip = camera.view_proj * world_pos;
    out.uv = input.uv;
    out.mask_uv = input.uv;
    out.normal = normalize(normal_matrix * skinned_normal);
    out.color = vec4<f32>(point_lights(world_pos.xyz, out.normal), 1.0);
    out.world_pos = world_pos.xyz;
    return out;
}

fn object_zone() -> u32 {
    return object.zone;
}

fn object_sun_affect() -> u32 {
    return object.sun_affect;
}

fn object_ambient() -> vec3<f32> {
    return object.ambient.rgb;
}

fn object_overlay() -> vec4<f32> {
    return object.overlay;
}

fn object_sun_visible() -> f32 {
    return f32(object.sun_visible);
}
