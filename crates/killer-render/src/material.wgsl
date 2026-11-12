const OP_SELECT_TEXTURE: u32 = 0u;
const OP_SELECT_CURRENT: u32 = 1u;
const OP_MODULATE: u32 = 2u;
const OP_MODULATE_2X: u32 = 3u;
const OP_MODULATE_4X: u32 = 4u;
const OP_ADD: u32 = 5u;
const OP_SUBTRACT: u32 = 6u;
const OP_BLEND_TEXTURE_ALPHA: u32 = 7u;
const OP_BLEND_DIFFUSE_ALPHA: u32 = 8u;
const OP_BLEND_CURRENT_ALPHA: u32 = 9u;
const OP_BLEND_FACTOR_ALPHA: u32 = 10u;
const OP_MODULATE_ALPHA_ADD_COLOR: u32 = 11u;
const OP_MULTIPLY_ADD: u32 = 12u;
const ALPHA_TEXTURE: u32 = 0u;
const ALPHA_CURRENT: u32 = 1u;
const ALPHA_DIFFUSE: u32 = 2u;
const ALPHA_TFACTOR: u32 = 3u;
const ALPHA_MODULATE: u32 = 4u;
const ALPHA_ADD: u32 = 5u;
const SOURCE_TEXTURE: u32 = 0u;
const SOURCE_CUBE: u32 = 1u;
const SOURCE_FACTOR: u32 = 2u;
const SOURCE_DIFFUSE: u32 = 3u;
const COORD_STREAM0: u32 = 0u;
const COORD_STREAM1: u32 = 1u;
const COORD_CAMERA_ENV: u32 = 2u;
const COORD_WORLD_ENV: u32 = 3u;

fn stage_uv(slot: u32, uv: vec2<f32>) -> vec2<f32> {
    let p = vec3<f32>(uv, 1.0);
    return vec2<f32>(dot(material.uv_u[slot].xyz, p), dot(material.uv_v[slot].xyz, p));
}

fn reflection(input: VertexOutput, camera_space: bool) -> vec3<f32> {
    let view_dir = normalize(input.world_pos - camera.eye.xyz);
    let r = reflect(view_dir, normalize(input.normal));
    if (camera_space) {
        return (camera.view * vec4<f32>(r, 0.0)).xyz;
    }
    return r;
}

fn sample_stage(slot: u32, input: VertexOutput, coord: u32, frame: u32) -> vec4<f32> {
    var uv = input.uv;
    if (coord == COORD_STREAM1) {
        uv = input.mask_uv;
    } else if (coord == 7u) {
        uv = input.normal.xy / input.normal.z;
    } else if (coord == 4u || coord == 5u) {
        let projected = material.projection[slot] * vec4<f32>(input.world_pos, 1.0);
        uv = projected.xy / select(1.0, projected.z, coord == 5u);
    } else if (coord == COORD_CAMERA_ENV || coord == COORD_WORLD_ENV) {
        let r = reflection(input, coord == COORD_CAMERA_ENV);
        uv = r.xy * 0.5 + 0.5;
    }
    uv = stage_uv(slot, uv);
    switch slot {
        case 0u: {
            return textureSampleBias(stage_texture0, stage_sampler0, uv, frame, DEFAULT_TEX_MIP_BIAS);
        }
        case 1u: {
            return textureSampleBias(stage_texture1, stage_sampler1, uv, frame, DEFAULT_TEX_MIP_BIAS);
        }
        case 2u: {
            return textureSampleBias(stage_texture2, stage_sampler2, uv, frame, DEFAULT_TEX_MIP_BIAS);
        }
        case 3u: {
            return textureSampleBias(stage_texture3, stage_sampler3, uv, frame, DEFAULT_TEX_MIP_BIAS);
        }
        case 4u: {
            return textureSampleBias(stage_texture4, stage_sampler4, uv, frame, DEFAULT_TEX_MIP_BIAS);
        }
        case 5u: {
            return textureSampleBias(stage_texture5, stage_sampler5, uv, frame, DEFAULT_TEX_MIP_BIAS);
        }
        case 6u: {
            return textureSampleBias(stage_texture6, stage_sampler6, uv, frame, DEFAULT_TEX_MIP_BIAS);
        }
        default: {
            return textureSampleBias(stage_texture7, stage_sampler7, uv, frame, DEFAULT_TEX_MIP_BIAS);
        }
    }
}

fn stage_source(slot: u32, input: VertexOutput, diffuse: vec4<f32>) -> vec4<f32> {
    let stage = material.stages[slot];
    let source = (stage.flags >> 1u) & 7u;
    let coord = (stage.flags >> 4u) & 7u;
    switch source {
        case SOURCE_CUBE: {
            var direction = reflection(input, coord == COORD_CAMERA_ENV);
            if (coord == COORD_CAMERA_ENV) {
                direction = (material.projection[slot] * vec4<f32>(direction, 1.0)).xyz;
            }
            return textureSample(cube_texture, cube_sampler, direction);
        }
        case SOURCE_FACTOR: {
            return material.tfactor;
        }
        case SOURCE_DIFFUSE: {
            return diffuse;
        }
        case 4u: {
            let normal = (material.projection[slot] * vec4<f32>(input.normal, 0.0)).xyz;
            return textureSample(cube_texture, cube_sampler, normal);
        }
        default: {
            return sample_stage(slot, input, coord, stage.frame);
        }
    }
}

fn combine_color(op: u32, arg1: vec4<f32>, arg2: vec4<f32>, src: vec4<f32>, current: vec4<f32>, diffuse: vec4<f32>, current_alpha: f32) -> vec3<f32> {
    switch op {
        case OP_SELECT_TEXTURE: {
            return src.rgb;
        }
        case OP_SELECT_CURRENT: {
            return current.rgb;
        }
        case OP_MODULATE: {
            return arg1.rgb * arg2.rgb;
        }
        case OP_MODULATE_2X: {
            return arg1.rgb * arg2.rgb * 2.0;
        }
        case OP_MODULATE_4X: {
            return arg1.rgb * arg2.rgb * 4.0;
        }
        case OP_ADD: {
            return arg1.rgb + arg2.rgb;
        }
        case OP_SUBTRACT: {
            return arg1.rgb - arg2.rgb;
        }
        case OP_BLEND_TEXTURE_ALPHA: {
            return mix(arg2.rgb, arg1.rgb, src.a);
        }
        case OP_BLEND_DIFFUSE_ALPHA: {
            return mix(arg2.rgb, arg1.rgb, diffuse.a);
        }
        case OP_BLEND_CURRENT_ALPHA: {
            return mix(arg2.rgb, arg1.rgb, current_alpha);
        }
        case OP_BLEND_FACTOR_ALPHA: {
            return mix(arg2.rgb, arg1.rgb, material.tfactor.a);
        }
        case OP_MODULATE_ALPHA_ADD_COLOR: {
            return arg1.rgb + arg1.a * arg2.rgb;
        }
        case OP_MULTIPLY_ADD: {
            return material.tfactor.rgb * src.rgb + current.rgb;
        }
        case 13u: {
            return mix(material.tfactor.rgb, current.rgb, src.a);
        }
        case 14u: {
            return mix(material.tfactor.rgb, src.rgb, diffuse.a);
        }
        case 15u: {
            return current.rgb * src.a;
        }
        case 16u: {
            return arg1.rgb + arg2.rgb - vec3<f32>(0.5);
        }
        case 17u: {
            return (arg1.rgb + arg2.rgb - vec3<f32>(0.5)) * 2.0;
        }
        case 18u: {
            return arg1.rgb + arg2.rgb * (vec3<f32>(1.0) - arg1.rgb);
        }
        case 19u: {
            return arg1.rgb + arg2.rgb * (1.0 - src.a);
        }
        case 20u: {
            return arg1.rgb * arg2.rgb + vec3<f32>(arg1.a);
        }
        case 21u: {
            return arg1.rgb + (1.0 - arg1.a) * arg2.rgb;
        }
        case 22u: {
            return (vec3<f32>(1.0) - arg1.rgb) * arg2.rgb + vec3<f32>(arg1.a);
        }
        case 24u: {
            return arg1.rgb;
        }
        case 25u: {
            return current.rgb + arg1.rgb * arg2.rgb;
        }
        case 26u: {
            return vec3<f32>(dot(arg1.rgb * 2.0 - vec3<f32>(1.0), arg2.rgb * 2.0 - vec3<f32>(1.0)));
        }
        default: {
            return current.rgb;
        }
    }
}

fn combine_alpha(op: u32, src: vec4<f32>, current: vec4<f32>, diffuse: vec4<f32>) -> f32 {
    switch op {
        case ALPHA_TEXTURE: {
            return src.a;
        }
        case ALPHA_DIFFUSE: {
            return diffuse.a;
        }
        case ALPHA_TFACTOR: {
            return material.tfactor.a;
        }
        case ALPHA_MODULATE: {
            return src.a * current.a;
        }
        case ALPHA_ADD: {
            return src.a + current.a;
        }
        case 6u: {
            return mix(current.a, src.a, diffuse.a);
        }
        default: {
            return current.a;
        }
    }
}

fn apply_light(current: vec4<f32>, diffuse: vec4<f32>) -> vec4<f32> {
    var lit = current;
    lit = vec4<f32>(clamp(current.rgb * MODULATE_2X * diffuse.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), current.a);
    if (material.light_after == 1u && material.kind != KIND_TERRAIN) {
        lit.a = current.a * diffuse.a;
    }
    return lit;
}

fn deco_fade(dist: f32) -> f32 {
    if (dist > material.fade.y) {
        return 0.0;
    }
    if (dist <= material.fade.x) {
        return 1.0;
    }
    return 1.0 - (dist - material.fade.x) / (material.fade.y - material.fade.x);
}

fn shade_material(input: VertexOutput) -> vec4<f32> {
    if (material.kind == 6u && ((material.stages[0].flags >> 4u) & 7u) == 5u) {
        let projected = material.projection[0] * vec4<f32>(input.world_pos, 1.0);
        let uv = projected.xy / projected.z;
        if (projected.z <= 0.0 || uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
            discard;
        }
    }
    let diffuse = vec4<f32>(light_factor(input.normal, input.color, input.mask_uv) * 0.5, input.color.a);
    var current = diffuse;
    var premodulate = false;
    var disabled = false;
    for (var slot = 0u; slot < material.stage_count; slot = slot + 1u) {
        let stage = material.stages[slot];
        if (slot == material.light_after && material.lit == 1u) {
            current = apply_light(current, diffuse);
        }
        if (stage.color_op == 23u) {
            disabled = true;
            break;
        }
        let src = stage_source(slot, input, diffuse);
        let source = (stage.flags >> 1u) & 7u;
        let current_argument = select(current, current * src, premodulate && source != SOURCE_FACTOR && source != SOURCE_DIFFUSE);
        premodulate = stage.color_op == 24u;
        let swapped = (stage.flags & 1u) == 1u;
        let arg1 = select(src, current_argument, swapped);
        let arg2 = select(current_argument, src, swapped);
        let rgb = combine_color(stage.color_op, arg1, arg2, src, current_argument, diffuse, current.a);
        let a = select(combine_alpha(stage.alpha_op, src, current_argument, diffuse), rgb.x, stage.color_op == 26u);
        current = vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), clamp(a, 0.0, 1.0));
    }
    if (!disabled && material.stage_count == material.light_after && material.lit == 1u) {
        current = apply_light(current, diffuse);
    }
    let overlay = object_overlay();
    if (overlay.w > 0.0) {
        current = vec4<f32>(clamp(current.rgb * overlay.rgb * overlay.w, vec3<f32>(0.0), vec3<f32>(1.0)), current.a);
    }
    if (material.has_mask == 1u) {
        current.a = textureSampleBias(mask_texture, mask_sampler, input.mask_uv, DEFAULT_TEX_MIP_BIAS).a;
    }
    if (material.fade.y > 0.0) {
        current.a = current.a * deco_fade(distance(input.world_pos, camera.eye.xyz));
    }
    if (input.actor_fade > 0.5) {
        let modifier = input.actor_fade - 1.0;
        let modifier_kind = material.padding.y & 3u;
        if (modifier_kind == 1u) {
            current = vec4<f32>(current.rgb * modifier, current.a);
        } else if (modifier_kind == 2u) {
            current.a = select(modifier, current.a * modifier, (material.padding.y & 4u) != 0u);
        }
    }
    if (material.alpha_test == 1u && current.a < material.alpha_ref) {
        discard;
    }
    return vec4<f32>(distance_fog(current.rgb, input.world_pos), current.a);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return shade_material(input);
}

struct ProjectorOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_bsp_projector(input: VertexOutput) -> ProjectorOutput {
    let slope = max(abs(dpdx(input.clip.z)), abs(dpdy(input.clip.z)));
    let depth = input.clip.z + (-4.0 * slope + bitcast<f32>(0xb58637bdu));
    return ProjectorOutput(shade_material(input), clamp(depth, 0.0, 1.0));
}
