struct Params { mode: vec4<f32>, glow: vec4<f32>, bloom: vec4<f32>, hdr: vec4<f32>, adaptation: vec4<f32> }
@group(0) @binding(0) var a: texture_2d<f32>;
@group(0) @binding(1) var b: texture_2d<f32>;
@group(0) @binding(2) var c: texture_2d<f32>;
@group(0) @binding(3) var<uniform> p: Params;
struct Vertex { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }
@vertex fn vs_main(@builtin(vertex_index) index: u32) -> Vertex {
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return Vertex(vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.0, 1.0), uv);
}
fn texel(t: texture_2d<f32>, xy: vec2<i32>) -> vec4<f32> {
    return textureLoad(t, clamp(xy, vec2<i32>(0), vec2<i32>(textureDimensions(t)) - 1), 0);
}
fn sample(t: texture_2d<f32>, uv: vec2<f32>) -> vec4<f32> {
    let position = uv * vec2<f32>(textureDimensions(t));
    if p.mode.y == 0.0 { return texel(t, vec2<i32>(floor(position))); }
    let base = floor(position - 0.5);
    let f = position - 0.5 - base;
    let xy = vec2<i32>(base);
    return mix(mix(texel(t, xy), texel(t, xy + vec2<i32>(1, 0)), f.x),
        mix(texel(t, xy + vec2<i32>(0, 1)), texel(t, xy + vec2<i32>(1, 1)), f.x), f.y);
}
@fragment fn fs_main(input: Vertex) -> @location(0) vec4<f32> {
    let uv = input.uv;
    let mode = u32(p.mode.x);
    let size = vec2<f32>(textureDimensions(a));
    if mode == 0u { return texel(a, vec2<i32>(floor(uv * size))); }
    if mode == 1u {
        var rgb = sample(a, uv).rgb * (p.glow.y / (p.glow.x + 0.001));
        rgb *= 1.0 + rgb / (p.glow.z * p.glow.z);
        rgb = max(rgb - 5.0, vec3<f32>(0.0));
        rgb /= 10.0 + rgb;
        return vec4<f32>(min(rgb, vec3<f32>(p.glow.w)), 1.0);
    }
    if mode == 2u { return vec4<f32>(sample(a, uv).rgb - p.bloom.y, 1.0); }
    if mode == 3u || mode == 4u {
        let weights = array<f32, 13>(0.002216,0.008764,0.026995,0.064759,0.120985,0.176033,0.199471,0.176033,0.120985,0.064759,0.026995,0.008764,0.002216);
        var color = vec4<f32>(0.0);
        for(var i = 0u; i < 13u; i++) {
            let offset = select(vec2<f32>(0.0, f32(i) - 6.0), vec2<f32>(f32(i) - 6.0, 0.0), mode == 3u);
            color += sample(a, uv + offset / size) * weights[i];
        }
        return color * p.bloom.x;
    }
    if mode == 5u || mode == 6u {
        let first = array<vec2<f32>,4>(vec2<f32>(-0.5,-0.5),vec2<f32>(-0.5,1.5),vec2<f32>(1.5,1.5),vec2<f32>(1.5,-0.5));
        let second = array<vec2<f32>,4>(vec2<f32>(-0.5,-0.5),vec2<f32>(0.5,-0.5),vec2<f32>(-0.5,0.5),vec2<f32>(0.5,0.5));
        var rgb = vec3<f32>(0.0);
        for(var i = 0u; i < 4u; i++) { rgb += sample(a, uv + select(second[i], first[i], mode == 5u) / size).rgb / 4.0; }
        return vec4<f32>(rgb, 1.0);
    }
    if mode == 7u { return clamp(sample(a, uv) + sample(b, uv), vec4<f32>(0.0), vec4<f32>(1.0)); }
    if mode == 8u { return mix(sample(a, uv), vec4<f32>(sample(b, uv).rgb, p.bloom.z), p.bloom.z); }
    if mode == 9u {
        let offsets = array<vec2<f32>,5>(vec2<f32>(-1.0,0.0),vec2<f32>(0.0,-1.0),vec2<f32>(0.0),vec2<f32>(0.0,1.0),vec2<f32>(1.0,0.0));
        var value = 0.0;
        for(var i = 0u; i < 5u; i++) { value += log(dot(sample(a, uv + offsets[i] / size).rgb, vec3<f32>(0.27,0.67,0.06)) + 0.0001); }
        return vec4<f32>(vec3<f32>(value / 5.0), 1.0);
    }
    if mode == 10u || mode == 11u {
        var color = vec4<f32>(0.0);
        for(var y = 0u; y < 4u; y++) { for(var x = 0u; x < 4u; x++) {
            color += sample(a, uv + vec2<f32>(1.5-f32(x),f32(y)-1.5) / size);
        } }
        return select(color / 16.0, exp(color / 16.0), mode == 11u);
    }
    if mode == 12u {
        let previous = texel(a, vec2<i32>(0)).r;
        let current = texel(b, vec2<i32>(0)).r;
        let value = previous + (current - previous) * (1.0 - pow(p.adaptation.x, p.adaptation.y * p.adaptation.z));
        return vec4<f32>(vec3<f32>(value), 1.0);
    }
    if mode == 13u {
        let source = sample(a, uv);
        let luminance = clamp(texel(b, vec2<i32>(0)).r, p.hdr.z, p.hdr.w);
        let current = texel(c, vec2<i32>(0)).r;
        var rgb = source.rgb * (0.166 * (2.0*p.hdr.x + 3.0*luminance + current) / (luminance + 0.001));
        rgb /= 1.0 + rgb;
        return vec4<f32>(p.hdr.y * rgb * rgb, source.a);
    }
    return sample(a, uv);
}
