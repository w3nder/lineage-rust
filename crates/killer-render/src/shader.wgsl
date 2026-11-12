struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) mask_uv: vec2<f32>,
    @location(4) color: vec4<f32>,
};

fn static_vertex(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip = camera.view_proj * vec4<f32>(input.position, 1.0);
    out.uv = input.uv;
    out.mask_uv = input.mask_uv;
    out.color = input.color;
    out.normal = input.normal;
    out.world_pos = input.position;
    return out;
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    return static_vertex(input);
}

fn object_zone() -> u32 {
    return material.zone;
}

fn object_ambient() -> vec3<f32> {
    return vec3<f32>(0.0);
}

fn object_overlay() -> vec4<f32> {
    return vec4<f32>(0.0);
}

fn object_sun_affect() -> u32 {
    return material.sun_affect;
}

fn object_sun_visible() -> f32 {
    return 1.0;
}

struct DecorationInput {
    @location(5) world_x: vec4<f32>,
    @location(6) world_y: vec4<f32>,
    @location(7) world_z: vec4<f32>,
    @location(8) world_w: vec4<f32>,
    @location(9) mask_uv: vec2<f32>,
    @location(10) color: vec4<f32>,
};

@vertex
fn vs_instanced(input: VertexInput, instance: DecorationInput) -> VertexOutput {
    let world = mat4x4<f32>(instance.world_x, instance.world_y, instance.world_z, instance.world_w);
    let position = (world * vec4<f32>(input.position, 1.0)).xyz;
    var out: VertexOutput;
    out.clip = camera.view_proj * vec4<f32>(position, 1.0);
    out.uv = input.uv;
    out.mask_uv = instance.mask_uv;
    out.color = instance.color;
    out.normal = input.normal;
    out.world_pos = position;
    return out;
}

struct FadeInput {
    @location(11) fade: f32,
};

@vertex
fn vs_faded(input: VertexInput, instance: FadeInput) -> VertexOutput {
    var out = static_vertex(input);
    out.actor_fade = instance.fade + 1.0;
    return out;
}
