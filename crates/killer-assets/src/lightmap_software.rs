use crate::engine_math::{rotator_forward, sin_tab};
use crate::lightmap_tables::{attenuation, inverse_root_exponent, inverse_root_mantissa, shadow_colors, sqrt_table};
use crate::pawn_rotation::ftol;
use std::sync::OnceLock;

const EFFECT_SUN: u8 = 0x13;
const EFFECT_SPOT: u8 = 4;
const EFFECT_SEARCHLIGHT: u8 = 0x12;
const EFFECT_PULSE_SLOW: u8 = 5;
const EFFECT_PULSE_FAST: u8 = 6;
const EFFECT_FLICKER: u8 = 9;
const EFFECT_RIPPLE: u8 = 0xa;
const EFFECT_QUADRATIC: u8 = 0x11;
const EFFECT_WAVE: u8 = 0x10;
const EFFECT_CONE_A: u8 = 0xc;
const EFFECT_CONE_B: u8 = 8;
const EFFECT_LINEAR: u8 = 0xd;
const EFFECT_INVERSE_RADIUS: u8 = 0x14;
const EFFECT_RING: u8 = 0xe;
const ATTENUATION_LIMIT: u32 = 0x1000;
const FOUR_PI_TWICE: f64 = f64::from_bits(0x4039_21fb_5444_2d18);
const PI: f64 = f64::from_bits(0x4009_21fb_5444_2d18);
const SPOT_PHASE_STEP: f64 = f64::from_bits(0x3fb9_21fb_5444_2d18);
const TWO_PI: f64 = f64::from_bits(0x4019_21fb_5444_2d18);
const SPOT_LIMIT: f64 = 9.42477796076938;
const ANGLE_SCALE: f32 = 65536.0;
const ANGLE_FACTOR: f64 = f64::from_bits(0xbfc4_5f30_6dc9_c883);
const SHADOW_SCALE: f32 = 256.0;
const MAGIC_ROUND: f32 = 8_388_608.0;

fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| Tables {
        attenuation: attenuation(),
        colors: shadow_colors(),
        sqrt: sqrt_table(),
        inverse_mantissa: inverse_root_mantissa(),
        inverse_exponent: inverse_root_exponent(),
    })
}

struct Tables {
    attenuation: Vec<f32>,
    colors: Vec<[u32; 4]>,
    sqrt: Vec<u32>,
    inverse_mantissa: Vec<f32>,
    inverse_exponent: Vec<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LightBitmap {
    pub bits: Vec<u8>,
    pub rows: i32,
    pub stride: i32,
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LightActor {
    pub effect: u8,
    pub period: u8,
    pub phase: u8,
    pub cone: u8,
    pub time_seconds: f32,
    pub rotation: [i32; 3],
    pub is_pawn: bool,
    pub controller_rotation: Option<[i32; 3]>,
}

impl LightActor {
    pub fn view_rotation(&self) -> [i32; 3] {
        match (self.is_pawn, self.controller_rotation) {
            (true, Some(rotation)) => rotation,
            _ => self.rotation,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LightRender {
    pub actor: Option<LightActor>,
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub radius: f32,
    pub dynamic: i32,
    pub color: [f32; 3],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Surface {
    pub flags: u32,
    pub normal: [f32; 3],
    pub distance: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LightMapGeometry {
    pub size: [i32; 2],
    pub base: [f32; 3],
    pub axis_x: [f32; 3],
    pub axis_y: [f32; 3],
}

pub fn app_floor(value: f32) -> i32 {
    ftol(f64::from(value).floor())
}

pub fn app_round(value: f32) -> i32 {
    ftol((f64::from(value) + 0.5).floor())
}

pub fn app_ceil_log_two(value: u32) -> u8 {
    if value == 0 { 0 } else { (32 - (value - 1).leading_zeros()) as u8 }
}

fn cos_float(angle: f32) -> f32 {
    let units = ftol(f64::from(angle) * f64::from(ANGLE_SCALE) * ANGLE_FACTOR);
    sin_tab(0x4000_i32.wrapping_sub(units))
}

fn size_squared(v: [f32; 3]) -> f64 {
    f64::from(v[0]) * f64::from(v[0]) + f64::from(v[1]) * f64::from(v[1]) + f64::from(v[2]) * f64::from(v[2])
}

fn fast_sqrt(bits: u32) -> f32 {
    let table = &tables().sqrt;
    f32::from_bits(table[((bits >> 11) & 0x1ffc) as usize / 4].wrapping_add((bits & 0x7f00_0000) >> 1))
}

fn fast_inverse_root(bits: u32) -> f64 {
    let t = tables();
    f64::from(t.inverse_mantissa[((bits >> 13) & 0x3ff) as usize] * t.inverse_exponent[(bits >> 23) as usize])
}

fn greater_of(first: f32, second: f32) -> f32 {
    if first < second { second } else { first }
}

pub fn expand_shadow_bits(bitmap: Option<&LightBitmap>, min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Vec<u8> {
    let Some(bitmap) = bitmap.filter(|_| max_y != min_y) else {
        let length = ((max_x - min_x) + 1) * ((max_y - min_y) + 1);
        return vec![0xff; length.max(0) as usize];
    };
    let stride = bitmap.stride.max(0) as usize;
    let rows = bitmap.rows.max(0) as usize;
    let row_ints = stride * 2;
    let mut buffer = vec![0u32; rows * row_ints];
    let colors = &tables().colors;
    let (mut upper, mut middle, mut lower) = (0usize, 0usize, row_ints);
    for row in 0..rows {
        let base = row * stride;
        let byte = |index: usize| u32::from(bitmap.bits.get(index).copied().unwrap_or(0));
        let mut window = byte(base) << 10;
        let mut next = base + 1;
        if window & 0x400 != 0 {
            window |= 0x300;
        }
        for column in 0..stride {
            let following = if column + 1 < stride {
                let value = byte(next) << 10;
                next += 1;
                value
            } else if (window >> 8) & 0x200 != 0 {
                0xc00
            } else {
                0
            };
            window = (window >> 8).wrapping_add(following);
            for (half, shift) in [(0, 0), (1, 4)] {
                let entry = colors[((window >> shift) & 0x7f) as usize];
                buffer[upper + half] = buffer[upper + half].wrapping_add(entry[0]);
                buffer[middle + half] = buffer[middle + half].wrapping_add(entry[1]);
                buffer[lower + half] = buffer[lower + half].wrapping_add(entry[2]);
            }
            upper += 2;
            middle += 2;
            lower += 2;
        }
        if row == 0 {
            upper -= row_ints;
        }
        if row + 2 == rows {
            lower -= row_ints;
        }
    }
    buffer.iter().flat_map(|word| word.to_le_bytes()).collect()
}

pub fn accumulate_rgba(intensity: &[u8], bounds: [i32; 4], color: [f32; 3], dest: &mut [u32], stride: i32) {
    let [min_x, min_y, max_x, max_y] = bounds;
    let channel = |value: f32| {
        let scaled = ftol(f64::from(value) * f64::from(SHADOW_SCALE));
        32u8.wrapping_sub(app_ceil_log_two(scaled as u32))
    };
    let shifts = [channel(color[0]), channel(color[1]), channel(color[2])];
    let multiplier = |shift: u8, value: f32| {
        let unit = 1i32.wrapping_shl(u32::from(shift));
        app_floor((f64::from(unit) * f64::from(value)) as f32) as u32
    };
    let steps = [multiplier(shifts[0], color[0]), multiplier(shifts[1], color[1]), multiplier(shifts[2], color[2])];
    let mut accumulators = [0u32; 3];
    let mut palette = [0u32; 256];
    for entry in &mut palette {
        let mut bytes = [0u32; 3];
        for index in 0..3 {
            bytes[index] = (accumulators[index].wrapping_shr(u32::from(shifts[index]))).min(0xff);
            accumulators[index] = accumulators[index].wrapping_add(steps[index]);
        }
        *entry = bytes[2] | (bytes[1] << 8) | (bytes[0] << 16) | 0xff00_0000;
    }
    let width = max_x - min_x + 1;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let source = (width * (y - min_y) + (x - min_x)) as usize;
            let target = (y * stride + x) as usize;
            let added = palette[usize::from(intensity.get(source).copied().unwrap_or(0))];
            if let Some(slot) = dest.get_mut(target) {
                *slot = saturating_add(*slot, added);
            }
        }
    }
}

fn saturating_add(current: u32, added: u32) -> u32 {
    (0..4).fold(0u32, |out, byte| {
        let sum = ((current >> (byte * 8)) & 0xff) + ((added >> (byte * 8)) & 0xff);
        out | (sum.min(0xff) << (byte * 8))
    })
}

#[derive(Clone, Copy)]
struct Walk<'a> {
    input: &'a [u8],
    pitch: i32,
    width: i32,
    geometry: &'a LightMapGeometry,
    bounds: [i32; 4],
}

fn rounded(value: f64) -> f64 {
    f64::from(value as f32)
}

impl Walk<'_> {
    fn run(&self, start: [f32; 3], mut pixel: impl FnMut([f32; 3], u8) -> u8) -> Vec<u8> {
        let [min_x, min_y, max_x, max_y] = self.bounds;
        let height = (max_y - min_y + 1).max(0);
        let mut out = Vec::with_capacity((self.width.max(0) * height) as usize);
        let g = self.geometry;
        let mut v = start;
        let mut source: i64 = 0;
        for _ in min_y..=max_y {
            for _ in min_x..=max_x {
                let bits = self.input.get(source as usize).copied().unwrap_or(0);
                out.push(pixel(v, bits));
                source += 1;
                for (component, step) in v.iter_mut().zip(g.axis_x) {
                    *component = (f64::from(*component) + f64::from(step)) as f32;
                }
            }
            let across = f64::from(self.width);
            let steps = [across * f64::from(g.axis_x[0]), rounded(across * f64::from(g.axis_x[1])), rounded(across * f64::from(g.axis_x[2]))];
            let x = f64::from(v[0]) - steps[0];
            let y = f64::from(v[1]) - steps[1];
            let z = rounded(f64::from(v[2]) - steps[2]);
            v = [(x + f64::from(g.axis_y[0])) as f32, (y + f64::from(g.axis_y[1])) as f32, (z + f64::from(g.axis_y[2])) as f32];
            source += i64::from(self.pitch - self.width);
        }
        out
    }
}

pub fn light_intensity(input: &[u8], surface: &Surface, geometry: &LightMapGeometry, bitmap_stride: Option<i32>, bounds: [i32; 4], light: &LightRender) -> Vec<u8> {
    let [min_x, min_y, max_x, _] = bounds;
    let width = max_x - min_x + 1;
    let pitch = bitmap_stride.map_or(width, |stride| stride << 3);
    let default_actor = LightActor::default();
    let actor = light.actor.as_ref().unwrap_or(&default_actor);
    if actor.effect == EFFECT_SUN {
        return sun_intensity(input, surface, geometry, pitch, width, light);
    }
    let radius = if 1.0 < light.radius { light.radius } else { 1.0 };
    let inverse = 1.0 / f64::from(radius);
    let inverse_f = inverse as f32;
    let scale = (inverse * inverse * 4093.0) as f32;
    let distance = {
        let dx = f64::from(light.position[0]) - f64::from(geometry.base[0]);
        let dy = f64::from(light.position[1]) - f64::from(geometry.base[1]);
        let dz = f64::from(light.position[2]) - f64::from(geometry.base[2]);
        let numerator = dx * f64::from(surface.normal[0]) + (dy * f64::from(surface.normal[1]) + dz * f64::from(surface.normal[2]));
        (numerator / f64::from(light.radius)) as f32
    };
    let factor = if distance < 0.0 { -distance } else { distance };
    let fx = f64::from(min_x) + 0.5;
    let fy = f64::from(min_y) + 0.5;
    let g = geometry;
    let row_y = [rounded(fy * f64::from(g.axis_y[0])), rounded(fy * f64::from(g.axis_y[1])), rounded(fy * f64::from(g.axis_y[2]))];
    let column_z = fx * f64::from(g.axis_x[2]);
    let column_y = rounded(fx * f64::from(g.axis_x[1]));
    let column_x = rounded(fx * f64::from(g.axis_x[0]));
    let dz = f64::from(g.base[2]) - f64::from(light.position[2]);
    let dy = rounded(f64::from(g.base[1]) - f64::from(light.position[1]));
    let dx = rounded(f64::from(g.base[0]) - f64::from(light.position[0]));
    let z_sum = column_z + dz;
    let y_sum = rounded(dy + column_y);
    let x_sum = rounded(dx + column_x);
    let start = [(x_sum + row_y[0]) as f32, (y_sum + row_y[1]) as f32, ((z_sum + row_y[2]) as f32)];
    let walk = Walk { input, pitch, width, geometry, bounds };
    let attenuation = &tables().attenuation;
    let time = f64::from(actor.time_seconds);
    let factor64 = f64::from(factor);
    let floor_byte = |value: f64| app_floor(value as f32) as u8;
    let ss_index = |v: [f32; 3]| app_round((size_squared(v) * f64::from(scale)) as f32);
    let in_range = |bits: u8, index: i32| bits != 0 && (index as u32) < ATTENUATION_LIMIT;
    let att = |index: i32| f64::from(attenuation[index as usize]);
    match actor.effect {
        EFFECT_SPOT => {
            let base = if actor.period == 0 { 0.0 } else { time * 35.0 / f64::from(actor.period) };
            let phase = ((f64::from(actor.phase) * SPOT_PHASE_STEP + base) + TWO_PI) as f32;
            walk.run(start, |v, bits| {
                let index = ss_index(v);
                if !in_range(bits, index) {
                    return 0;
                }
                let angle = f64::from(v[0]).atan2(f64::from(v[1]));
                let wrapped = (angle * 4.0 + f64::from(phase)) % FOUR_PI_TWICE;
                let wrapped_f = wrapped as f32;
                if wrapped < PI || f64::from(wrapped_f) > SPOT_LIMIT {
                    return 0;
                }
                let mut shape = (f64::from(cos_float(wrapped_f)) + 1.0) * 0.5;
                let near = (f64::from(v[0]) * f64::from(v[0]) + f64::from(v[1]) * f64::from(v[1])) * 6e-05;
                if near < 1.0 {
                    shape *= near;
                }
                floor_byte(f64::from(bits) * att(index) * shape * factor64)
            })
        }
        EFFECT_SEARCHLIGHT => walk.run(start, |v, bits| {
            let index = ss_index(v);
            if !in_range(bits, index) {
                return 0;
            }
            let angle = f64::from(v[0]).atan2(f64::from(v[1]));
            let argument = (angle * 6.0 + time * 3.5) as f32;
            let mut shape = (f64::from(cos_float(argument)) + 1.0) * 0.5;
            let near = (f64::from(v[0]) * f64::from(v[0]) + f64::from(v[1]) * f64::from(v[1])) * 0.0001;
            if near < 1.0 {
                shape = shape * near + (1.0 - near);
            }
            floor_byte(f64::from(bits) * att(index) * shape * factor64)
        }),
        EFFECT_PULSE_SLOW | EFFECT_PULSE_FAST => {
            let slow = actor.effect == EFFECT_PULSE_SLOW;
            walk.run(start, |v, bits| {
                let index = ss_index(v);
                if !in_range(bits, index) {
                    return 0;
                }
                let root = ftol(f64::from(fast_sqrt((size_squared(v) as f32).to_bits())));
                let (base, scale) = if slow { (root, 1024.0) } else { (root >> 2, 2048.0) };
                let units = ftol((f64::from(base) - time * 35.0) * scale);
                let wave = f64::from(sin_tab(units)) * 0.3 + 0.7;
                floor_byte(f64::from(bits) * att(index) * wave * factor64)
            })
        }
        EFFECT_FLICKER => walk.run(start, |v, bits| {
            let index = ss_index(v);
            if !in_range(bits, index) {
                return 0;
            }
            let root = ftol(f64::from(fast_sqrt((size_squared(v) as f32).to_bits())) * 8.0);
            let phase = time * 4000.0;
            let first = sin_tab(ftol((f64::from(root.wrapping_add(root)) - phase) * 16.0));
            let second = sin_tab(0x4000_i32.wrapping_sub(ftol((f64::from(root) + phase) * -16.0)));
            let third = sin_tab(ftol((f64::from(root >> 1) - phase) * 16.0));
            let blend = |value: f32| f64::from(value) * 0.1 + 0.9;
            let product = blend(third) * (blend(first) * blend(second));
            floor_byte(f64::from(bits) * att(index) * product * factor64)
        }),
        EFFECT_RIPPLE => walk.run(start, |v, bits| {
            let index = ss_index(v);
            if !in_range(bits, index) {
                return 0;
            }
            let first = f64::from(v[0]).atan2(f64::from(v[1]));
            let first_term = (first * 11.0) as f32;
            let flat = (f64::from(v[0]) * f64::from(v[0]) + f64::from(v[1]) * f64::from(v[1])).sqrt();
            let second = flat.atan2(f64::from(v[2]));
            let second_term = (second * 11.0) as f32;
            let first_wave = ((f64::from(cos_float((time * 5.0 + f64::from(first_term)) as f32)) + 1.0) * 0.5) as f32;
            let second_wave = (f64::from(cos_float((time * 5.0 + f64::from(second_term)) as f32)) + 1.0) * 0.5;
            let mut mix = (f64::from(first_wave) + second_wave) - second_wave * f64::from(first_wave);
            let near = (f64::from(v[0]) * f64::from(v[0]) + f64::from(v[1]) * f64::from(v[1])) * 5e-05;
            if near < 1.0 {
                mix *= near;
            }
            floor_byte(f64::from(bits) * (att(index) * (1.0 - mix)) * factor64)
        }),
        EFFECT_QUADRATIC => walk.run(start, |v, bits| {
            let flat = (f64::from(v[0]) * f64::from(v[0]) + f64::from(v[1]) * f64::from(v[1])) * f64::from(inverse_f) * f64::from(inverse_f);
            let value = app_floor(((1.0 - flat) * f64::from(bits)) as f32);
            if value <= 0 { 0 } else { value as u8 }
        }),
        EFFECT_WAVE => walk.run(start, |v, bits| {
            let index = ss_index(v);
            if !in_range(bits, index) {
                return 0;
            }
            let flat = ((f64::from(v[0]) * f64::from(v[0]) + f64::from(v[1]) * f64::from(v[1])) as f32).to_bits();
            let angle = f64::from(fast_sqrt(flat)).atan2(f64::from(v[2]));
            let argument = (angle * 11.0 + time * 5.0) as f32;
            let shape = (f64::from(cos_float(argument)) + 1.0) * 0.5;
            floor_byte(f64::from(bits) * att(index) * shape * factor64)
        }),
        EFFECT_CONE_A | EFFECT_CONE_B => {
            let direction = rotator_forward(actor.view_rotation());
            let outer = 1.0 - f64::from(actor.cone) * 0.003_906_25;
            let reciprocal = 1.0 / (1.0 - outer);
            let reciprocal_f = reciprocal as f32;
            let offset = (reciprocal * outer) as f32;
            let outer_squared = (outer * outer) as f32;
            walk.run(start, |v, bits| {
                let index = ss_index(v);
                if !in_range(bits, index) {
                    return 0;
                }
                let squared = size_squared(v) as f32;
                let along = f64::from(v[2]) * f64::from(direction.z) + f64::from(direction.y) * f64::from(v[1]) + f64::from(v[0]) * f64::from(direction.x);
                let along_f = along as f32;
                if along <= 0.0 {
                    return 0;
                }
                if f64::from(squared) * f64::from(outer_squared) >= f64::from(along_f) * f64::from(along_f) {
                    return 0;
                }
                let root = fast_inverse_root(squared.to_bits());
                let shaped = root * f64::from(along_f) * f64::from(reciprocal_f) - f64::from(offset);
                floor_byte(f64::from(bits) * att(index) * (shaped * shaped) * factor64)
            })
        }
        EFFECT_LINEAR => walk.run(start, |v, bits| {
            let index = ss_index(v);
            if !in_range(bits, index) {
                return 0;
            }
            let root = fast_sqrt((size_squared(v) as f32).to_bits());
            let value = greater_of((1.0 - f64::from(root) * f64::from(inverse_f)) as f32, 0.0);
            floor_byte(f64::from(value) * f64::from(bits))
        }),
        EFFECT_INVERSE_RADIUS => walk.run(start, |v, bits| {
            let index = ss_index(v);
            if !in_range(bits, index) {
                return 0;
            }
            let limited = f64::from(greater_of(light.radius, 1.0));
            let reciprocal = (1.0 / (limited * limited)) as f32;
            let squared = size_squared(v) as f32;
            let value = greater_of((1.0 - f64::from(reciprocal) * f64::from(squared)) as f32, 0.0);
            floor_byte(f64::from(value) * f64::from(bits))
        }),
        EFFECT_RING => walk.run(start, |v, bits| {
            let index = ss_index(v);
            if !in_range(bits, index) {
                return 0;
            }
            let root = fast_sqrt((size_squared(v) as f32).to_bits());
            let ratio = f64::from(root) * f64::from(inverse_f);
            let ratio_f = ratio as f32;
            if ratio >= 1.0 || f64::from(ratio_f) <= 0.8 {
                return 0;
            }
            let offset = (f64::from(ratio_f) - 0.9).abs();
            floor_byte((1.0 - offset * 10.0) * f64::from(bits))
        }),
        _ => fixed_point_intensity(&walk, start, scale, factor),
    }
}

fn sun_intensity(input: &[u8], surface: &Surface, geometry: &LightMapGeometry, pitch: i32, width: i32, light: &LightRender) -> Vec<u8> {
    let d = light.direction;
    let mut facing = ((-f64::from(d[0])) * f64::from(surface.normal[0]) + ((-f64::from(d[1])) * f64::from(surface.normal[1]) + (-f64::from(d[2])) * f64::from(surface.normal[2]))) as f32;
    if surface.flags & 0x100 == 0 {
        if facing <= 0.0 {
            facing = 0.0;
        }
    } else if facing < 0.0 {
        facing = -facing;
    }
    let rows = geometry.size[1].max(0);
    let columns = geometry.size[0].max(0);
    let mut out = vec![0u8; ((rows - 1).max(0) * width + columns).max(0) as usize];
    let mut source: i64 = 0;
    let mut target: i64 = 0;
    for _ in 0..rows {
        for _ in 0..columns {
            let bits = input.get(source as usize).copied().unwrap_or(0);
            out[target as usize] = app_floor((f64::from(bits) * f64::from(facing)) as f32) as u8;
            source += 1;
            target += 1;
        }
        target += i64::from(width - columns);
        source += i64::from(pitch - columns);
    }
    out
}

fn fixed_point_intensity(walk: &Walk, start: [f32; 3], scale: f32, factor: f32) -> Vec<u8> {
    let Walk { input, pitch, width, geometry, bounds } = *walk;
    let [min_x, min_y, max_x, _] = bounds;
    let max_y = bounds[3];
    let g = geometry;
    let scale = (f64::from(scale) * 4096.0) as f32;
    let dot = |a: [f32; 3], b: [f32; 3]| (f64::from(a[2]) * f64::from(b[2]) + f64::from(a[1]) * f64::from(b[1])) + f64::from(a[0]) * f64::from(b[0]);
    let squared = |a: [f32; 3]| (f64::from(a[2]) * f64::from(a[2]) + f64::from(a[1]) * f64::from(a[1])) + f64::from(a[0]) * f64::from(a[0]);
    let round_scaled = |value: f64| app_round((value * f64::from(scale)) as f32);
    let v = start;
    let a = round_scaled((f64::from(v[0]) * f64::from(v[0]) + f64::from(v[1]) * f64::from(v[1])) + f64::from(v[2]) * f64::from(v[2])) as f32;
    let b = round_scaled(dot(v, g.axis_x)) as f32;
    let c = round_scaled(dot(v, g.axis_y)) as f32;
    let d = round_scaled(squared(g.axis_x)) as f32;
    let e = round_scaled(squared(g.axis_y)) as f32;
    let f = round_scaled(dot(g.axis_y, g.axis_x));
    let mut row_start = ftol(f64::from(a)) as u32;
    let mut row_delta = ftol(f64::from(c) * 2.0 + f64::from(e)) as u32;
    let row_second = ftol(f64::from(e) * 2.0) as u32;
    let mut column_delta_start = ftol(f64::from(b) * 2.0 + f64::from(d)) as u32;
    let column_delta_step = ftol(f64::from(f) * 2.0) as u32;
    let column_second = ftol(f64::from(d) * 2.0) as u32;
    let attenuation = &tables().attenuation;
    let mut out = Vec::new();
    let mut source: i64 = 0;
    for _ in min_y..=max_y {
        let mut value = row_start;
        let mut delta = column_delta_start;
        for _ in min_x..=max_x {
            let bits = input.get(source as usize).copied().unwrap_or(0);
            if bits == 0 || value >= 0x100_0000 {
                out.push(0);
            } else {
                let product = f64::from(bits) * f64::from(attenuation[(value >> 12) as usize]);
                let sum = ((product * f64::from(factor)) + f64::from(MAGIC_ROUND)) as f32;
                out.push(sum.to_bits() as u8);
            }
            source += 1;
            value = value.wrapping_add(delta);
            delta = delta.wrapping_add(column_second);
        }
        row_start = row_start.wrapping_add(row_delta);
        row_delta = row_delta.wrapping_add(row_second);
        column_delta_start = column_delta_start.wrapping_add(column_delta_step);
        source += i64::from(pitch - width);
    }
    out
}
