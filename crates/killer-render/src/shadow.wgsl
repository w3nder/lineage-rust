struct Camera {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    eye: vec4<f32>,
};

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
    light_position: array<vec4<f32>, 8>,
    light_color: array<vec4<f32>, 8>,
    light_direction: array<vec4<f32>, 8>,
    light_attenuation: array<vec4<f32>, 8>,
    ambient: vec4<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var<uniform> object: Object;
@group(1) @binding(1) var<storage, read> palette: array<mat4x4<f32>>;

struct Input {
    @location(0) position: vec3<f32>,
    @location(3) bones: vec4<u32>,
    @location(4) weights: vec4<f32>,
};

@vertex
fn vs_main(input: Input) -> @builtin(position) vec4<f32> {
    let p = vec4<f32>(input.position, 1.0);
    let w = input.weights;
    let m0 = palette[object.palette_offset + min(input.bones.x, object.bone_count - 1u)];
    let m1 = palette[object.palette_offset + min(input.bones.y, object.bone_count - 1u)];
    let m2 = palette[object.palette_offset + min(input.bones.z, object.bone_count - 1u)];
    let m3 = palette[object.palette_offset + min(input.bones.w, object.bone_count - 1u)];
    let skinned = w.x * (m0 * p) + w.y * (m1 * p) + w.z * (m2 * p) + w.w * (m3 * p);
    return camera.view_proj * object.world * skinned;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
