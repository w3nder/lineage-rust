use crate::lightmap_software::{LightBitmap, LightMapGeometry, LightRender, Surface, accumulate_rgba, app_floor, expand_shadow_bits, light_intensity};
use crate::pawn_rotation::ftol;
use std::collections::HashMap;

const EFFECT_SUN: u8 = 0x13;
const HUE_STEP: f32 = 0.011_764_706;
const HUE_STEP_LAST: f32 = 0.011_904_762;
const BYTE_RECIPROCAL: f32 = f32::from_bits(0x3b80_8081);
const BRIGHTNESS_SCALE: f32 = 0.005_490_196;
const BRIGHTNESS_OFFSET: f64 = 0.009_999_999_776_482_582;
const BRIGHTNESS_GAIN: f64 = 0.699_999_988_079_071;
const SPHERE_MARGIN: f32 = 1.05;

pub fn fget_hsv(hue: u8, saturation: u8, brightness: u8) -> [f32; 4] {
    let t = f64::from(brightness) * f64::from(BRIGHTNESS_SCALE);
    let mut gain = (t / (t.sqrt() + BRIGHTNESS_OFFSET)) * BRIGHTNESS_GAIN;
    gain = gain.clamp(0.0, 1.0);
    let h = i32::from(hue);
    let step = |value: i32| (f64::from(value) * f64::from(HUE_STEP)) as f32;
    let wheel: [f32; 3] = if hue < 0x56 {
        [step(0x55 - h), step(h), 0.0]
    } else if hue < 0xab {
        [0.0, step(0xaa - h), step(h - 0x55)]
    } else {
        [step(h - 0xaa), 0.0, (f64::from(0xff - h) * f64::from(HUE_STEP_LAST)) as f32]
    };
    let lift = [1.0 - f64::from(wheel[0]), 1.0 - f64::from(wheel[1]), (1.0 - f64::from(wheel[2])) as f32 as f64];
    let k = (f64::from(saturation) * f64::from(BYTE_RECIPROCAL)) as f32 as f64;
    let p0 = (lift[0] * k) as f32;
    let a1k = lift[1] * k;
    let a2k = lift[2] * k;
    let red = (f64::from(p0) + f64::from(wheel[0])) as f32;
    let green = (a1k + f64::from(wheel[1])) as f32;
    let blue = a2k + f64::from(wheel[2]);
    [(f64::from(red) * gain) as f32, (f64::from(green) * gain) as f32, (blue * gain) as f32, 1.0]
}

pub fn color_from_plane(plane: [f32; 4]) -> u32 {
    let channel = |value: f32| app_floor((f64::from(value) * 255.0) as f32);
    let clamp = |value: i32| value.clamp(0, 0xff) as u32;
    let blue = clamp(channel(plane[2]));
    let green = clamp(channel(plane[1]));
    let red = clamp(channel(plane[0]));
    let alpha = clamp(channel(plane[3]));
    blue | (green << 8) | (red << 16) | (alpha << 24)
}

pub fn ambient_color_from_hsv(hue: u8, saturation: u8, brightness: u8) -> u32 {
    let plane = fget_hsv(hue, saturation, brightness);
    color_from_plane(plane.map(|component| (f64::from(component) * 0.5) as f32))
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BitmapEntry {
    pub bitmap: LightBitmap,
    pub time: i32,
    pub actor_present: bool,
    pub deleted: bool,
    pub editor_hidden: bool,
    pub light: LightRender,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ZoneLight {
    pub sun: bool,
    pub hue: u8,
    pub saturation: u8,
    pub brightness: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LightMapSource {
    pub size: [i32; 2],
    pub surface: Surface,
    pub geometry: LightMapGeometry,
    pub matrix: [[f32; 4]; 4],
    pub zone: ZoneLight,
    pub bitmaps: Vec<BitmapEntry>,
    pub dynamic: Vec<LightRender>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimeSlice {
    Single,
    Timed { index: i32, count: i32 },
}

pub struct Environment<'a> {
    pub editor: bool,
    pub time: f64,
    pub ambient: &'a dyn Fn(f32) -> [u8; 3],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CacheEntry {
    pub size: [i32; 2],
    pub bitmaps: i32,
    pub dynamic: i32,
    pub flags: Vec<i32>,
    pub pixels: Vec<u32>,
}

#[derive(Default)]
pub struct LightMapCache {
    pub entries: HashMap<u64, CacheEntry>,
}

pub fn cache_key(model_index: u32, lightmap: i32, zone: i32, slice: TimeSlice) -> u64 {
    let base = match slice {
        TimeSlice::Single => lightmap.wrapping_mul(0x100),
        TimeSlice::Timed { index, count } => lightmap.wrapping_mul(count).wrapping_add(index).wrapping_mul(0x100),
    };
    let low = (base.wrapping_add(zone) as u32).wrapping_mul(0x100).wrapping_add(0x18);
    (u64::from(model_index) << 32).wrapping_add(u64::from(low))
}

fn editor_state(entry: &BitmapEntry) -> i32 {
    i32::from(entry.editor_hidden || entry.deleted)
}

fn base_color(source: &LightMapSource, slice: TimeSlice, env: &Environment) -> u32 {
    let zone = source.zone;
    let TimeSlice::Timed { index, count } = slice else {
        return ambient_color_from_hsv(zone.hue, zone.saturation, zone.brightness);
    };
    if !zone.sun {
        return ambient_color_from_hsv(zone.hue, zone.saturation, zone.brightness);
    }
    let time = if env.editor {
        let half = if count == 0 { 0.0 } else { (24.0 / f64::from(count)) * 0.5 };
        (f64::from(index) * 24.0 / f64::from(count) + half) as f32
    } else {
        env.time as f32
    };
    let [red, green, blue] = (env.ambient)(time);
    let halve = |value: u8| ftol(f64::from(value) * 0.5) as u32 & 0xff;
    halve(blue) | (halve(green) << 8) | (halve(red) << 16)
}

fn regenerate(entry: &mut CacheEntry, source: &LightMapSource, slice: TimeSlice, env: &Environment) {
    let color = base_color(source, slice, env);
    entry.pixels.iter_mut().for_each(|pixel| *pixel = color);
    for (index, item) in source.bitmaps.iter().enumerate() {
        let selected = match slice {
            TimeSlice::Single => true,
            TimeSlice::Timed { index: time, .. } => item.time == time || !source.zone.sun,
        };
        if !selected {
            continue;
        }
        if !item.actor_present || item.deleted || (env.editor && item.editor_hidden) {
            entry.flags[index] = 0;
            continue;
        }
        let bitmap = &item.bitmap;
        let bounds = [bitmap.min_x, bitmap.min_y, bitmap.max_x, bitmap.max_y];
        let expanded = expand_shadow_bits(Some(bitmap), bounds[0], bounds[1], bounds[2], bounds[3]);
        let intensity = light_intensity(&expanded, &source.surface, &source.geometry, Some(bitmap.stride), bounds, &item.light);
        accumulate_rgba(&intensity, bounds, item.light.color, &mut entry.pixels, source.size[0]);
        entry.flags[index] = 1;
    }
}

fn clamp_to(value: i32, limit: i32) -> i32 {
    if value < 0 {
        0
    } else if value >= limit {
        limit
    } else {
        value
    }
}

fn vector_size(v: [f32; 3]) -> f32 {
    (f64::from(v[0]) * f64::from(v[0]) + f64::from(v[1]) * f64::from(v[1]) + f64::from(v[2]) * f64::from(v[2])).sqrt() as f32
}

fn sphere_bounds(source: &LightMapSource, light: &LightRender) -> [i32; 4] {
    let [size_x, size_y] = source.size;
    let surface = &source.surface;
    let n = surface.normal;
    let p = light.position;
    let distance = (((f64::from(p[2]) * f64::from(n[2]) + f64::from(p[1]) * f64::from(n[1])) + f64::from(n[0]) * f64::from(p[0])) - f64::from(surface.distance)) as f32;
    let radius = f64::from(light.radius);
    let mut reach = (radius * radius) * f64::from(SPHERE_MARGIN) - f64::from(distance) * f64::from(distance);
    if reach < 0.0 {
        reach = 0.0;
    }
    let root = reach.sqrt();
    let root_f = root as f32;
    let scaled = [n[0] * distance, n[1] * distance, n[2] * distance];
    let projected = [
        (f64::from(p[0]) - f64::from(scaled[0])) as f32,
        (f64::from(p[1]) - f64::from(scaled[1])) as f32,
        (f64::from(p[2]) - f64::from(scaled[2])) as f32,
    ];
    let vector = [projected[0], projected[1], projected[2], 1.0];
    let column = |j: usize| {
        let m = &source.matrix;
        let mut sum = f64::from(vector[0]) * f64::from(m[0][j]) + f64::from(vector[1]) * f64::from(m[1][j]);
        sum += f64::from(vector[2]) * f64::from(m[2][j]);
        sum += f64::from(vector[3]) * f64::from(m[3][j]);
        sum as f32
    };
    let (u, v) = (column(0), column(1));
    let along_x = vector_size(source.geometry.axis_x);
    let along_y = vector_size(source.geometry.axis_y);
    let reach_x = (f64::from(root_f) / f64::from(along_x)) as f32;
    let reach_y = f64::from(root_f) / f64::from(along_y);
    let low_x = clamp_to(ftol(f64::from(u) - f64::from(reach_x)), size_x - 1);
    let high_x = clamp_to(ftol(f64::from(u) + f64::from(reach_x)), size_x - 1);
    let low_y = clamp_to(ftol(f64::from(v) - reach_y), size_y - 1);
    let high_y = clamp_to(ftol(f64::from(v) + reach_y), size_y - 1);
    [low_x, low_y, high_x, high_y]
}

fn dynamic_lights(source: &LightMapSource, env: &Environment, dynamic: &mut i32, dest: &mut [u32], stride: usize) {
    for light in &source.dynamic {
        if !env.editor || light.dynamic == 0 {
            continue;
        }
        let effect = light.actor.as_ref().map_or(0, |actor| actor.effect);
        let bounds = if effect == EFFECT_SUN { [0, 0, source.size[0] - 1, source.size[1] - 1] } else { sphere_bounds(source, light) };
        let expanded = expand_shadow_bits(None, bounds[0], bounds[1], bounds[2], bounds[3]);
        let intensity = light_intensity(&expanded, &source.surface, &source.geometry, None, bounds, light);
        accumulate_rgba(&intensity, bounds, light.color, dest, stride as i32);
        *dynamic = 1;
    }
}

pub fn get_texture_data(cache: &mut LightMapCache, key: u64, source: &LightMapSource, slice: TimeSlice, env: &Environment, dest: &mut [u32], stride: usize) {
    let [size_x, size_y] = source.size;
    let count = source.bitmaps.len() as i32;
    if cache.entries.get(&key).is_some_and(|entry| entry.size != source.size || entry.bitmaps != count) {
        cache.entries.remove(&key);
    }
    let created = !cache.entries.contains_key(&key);
    let entry = cache.entries.entry(key).or_insert_with(|| CacheEntry {
        size: source.size,
        bitmaps: count,
        dynamic: 0,
        flags: vec![0; count.max(0) as usize],
        pixels: vec![0; (size_x * size_y).max(0) as usize],
    });
    let expected = |index: usize| if env.editor { editor_state(&source.bitmaps[index]) } else { 0 };
    let stale = (0..source.bitmaps.len()).any(|index| entry.flags[index] == expected(index));
    if stale || created {
        regenerate(entry, source, slice, env);
    }
    for row in 0..size_y.max(0) as usize {
        let from = row * size_x as usize;
        let to = row * stride;
        dest[to..to + size_x as usize].copy_from_slice(&entry.pixels[from..from + size_x as usize]);
    }
    entry.dynamic = 0;
    dynamic_lights(source, env, &mut entry.dynamic, dest, stride);
}
