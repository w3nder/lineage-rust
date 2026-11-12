struct Tap {
    origin: vec2<f32>,
    extent: vec2<f32>,
    size: f32,
    weight: f32,
    pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> tap: Tap;
@group(0) @binding(1) var source: texture_2d_array<f32>;
@group(0) @binding(2) var tile_sampler: sampler;

struct Output {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> Output {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    var out: Output;
    out.position = vec4<f32>(corner.x * 2.0 - 1.0, 1.0 - corner.y * 2.0, 0.0, 1.0);
    out.uv = corner;
    return out;
}

@fragment
fn fs_main(input: Output) -> @location(0) vec4<f32> {
    let texel = tap.origin + input.uv * tap.extent;
    let color = textureSampleLevel(source, tile_sampler, texel / tap.size, 0, 0.0);
    return vec4<f32>(color.rgb, color.a * tap.weight);
}
