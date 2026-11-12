struct PresentOutput {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var frame_texture: texture_2d<f32>;
@group(0) @binding(1) var frame_sampler: sampler;

@group(0) @binding(2) var<uniform> gamma_ramp: array<vec4<f32>, 256>;
@group(0) @binding(3) var<uniform> flash: array<vec4<f32>, 2>;

fn apply_flash(c: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(mix(c.rgb, flash[1].rgb, 1.0 - flash[0].x), c.a);
}

fn apply_gamma(c: vec4<f32>) -> vec4<f32> {
    let indices = vec3<u32>(round(clamp(c.rgb, vec3<f32>(0.0), vec3<f32>(1.0)) * 255.0));
    return vec4<f32>(gamma_ramp[indices.r].x, gamma_ramp[indices.g].x, gamma_ramp[indices.b].x, c.a);
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> PresentOutput {
    var out: PresentOutput;
    let x = f32(i32(index & 1u) * 4 - 1);
    let y = f32(i32(index >> 1u) * 4 - 1);
    out.clip = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

@fragment
fn fs_copy(input: PresentOutput) -> @location(0) vec4<f32> {
    return apply_gamma(apply_flash(textureSample(frame_texture, frame_sampler, input.uv)));
}

@fragment
fn fs_srgb(input: PresentOutput) -> @location(0) vec4<f32> {
    let c = apply_gamma(apply_flash(textureSample(frame_texture, frame_sampler, input.uv)));
    return vec4<f32>(srgb_to_linear(c.rgb), c.a);
}
