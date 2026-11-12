use super::LightmapGroup;
use crate::lightmap_software::{LightActor, LightBitmap, LightMapGeometry, LightRender, Surface};
use crate::lightmap_texture::{BitmapEntry, Environment, LightMapCache, LightMapSource, TimeSlice, ZoneLight, cache_key, fget_hsv, get_texture_data};
use crate::textures::TextureFormatKind;
use crate::timeenv::{LightKind, TimeEnv};
use std::collections::{HashMap, HashSet};
use ue2_level::{Level, Light};

pub const ATLAS_SIZE: usize = 512;
pub const SUN_LIGHT_EFFECT: u8 = 19;
const SUN_LIGHT_CLASS: &str = "NMovableSunLight";
const LEVEL_BRIGHTNESS: f32 = 1.0;
const MODEL_CACHE_INDEX: u32 = 0;

pub fn slice_hour(slice: usize, count: usize) -> f32 {
    let count = count.max(1) as f32;
    slice as f32 * 24.0 / count + 12.0 / count
}

pub struct Regeneration<'a> {
    level: &'a Level,
    env: &'a TimeEnv,
    lights: HashMap<&'a str, &'a Light>,
    suns: HashSet<&'a str>,
    sun_direction: &'a dyn Fn(f32) -> [f32; 3],
    pub cache: LightMapCache,
}

impl<'a> Regeneration<'a> {
    pub fn new(level: &'a Level, env: &'a TimeEnv, sun_direction: &'a dyn Fn(f32) -> [f32; 3]) -> Self {
        let lights = level.lights.iter().map(|light| (light.name.as_str(), light)).collect();
        let suns = level.actors.iter().filter(|actor| actor.class.eq_ignore_ascii_case(SUN_LIGHT_CLASS)).map(|actor| actor.name.as_str()).collect();
        Self { level, env, lights, suns, sun_direction, cache: LightMapCache::default() }
    }

    pub fn sun_render(&self, hour: f32) -> LightRender {
        let (plane, brightness) = self.env.sun_light(LightKind::Actor, hour);
        LightRender {
            actor: Some(LightActor { effect: SUN_LIGHT_EFFECT, ..LightActor::default() }),
            direction: (self.sun_direction)(hour),
            color: plane.map(|channel| channel * brightness * LEVEL_BRIGHTNESS),
            ..LightRender::default()
        }
    }

    pub fn light_render(&self, light: &Light) -> LightRender {
        let brightness = light.brightness / 255.0;
        let hsv = fget_hsv(light.hue, light.saturation, 255);
        let color = [hsv[0], hsv[1], hsv[2]].map(|channel| channel * LEVEL_BRIGHTNESS * brightness * crate::scene::lights::static_effect(light));
        LightRender {
            actor: Some(LightActor {
                effect: light.effect,
                period: light.period,
                phase: light.phase,
                cone: light.cone,
                time_seconds: 0.0,
                rotation: light.rotation,
                is_pawn: false,
                controller_rotation: None,
            }),
            position: light.location,
            direction: light.direction,
            radius: light.radius,
            dynamic: 0,
            color,
        }
    }

    pub fn zone_light(&self, zone: i32) -> ZoneLight {
        let named = usize::try_from(zone).ok().and_then(|zone| self.level.zones.get(zone)).and_then(|zone| zone.actor.as_deref());
        let first = self.level.actors.first().map(|actor| actor.name.as_str());
        let properties = named.and_then(|actor| self.level.zone_infos.get(actor)).or_else(|| first.and_then(|actor| self.level.zone_infos.get(actor)));
        properties.map_or(ZoneLight { sun: true, hue: 0, saturation: 255, brightness: 0 }, |info| ZoneLight { sun: info.sun_affect, hue: info.ambient_hue, saturation: info.ambient_saturation, brightness: info.ambient_brightness })
    }

    pub fn source(&self, index: usize, hour: f32, deleted: &dyn Fn(&str) -> bool) -> Option<LightMapSource> {
        let render = &self.level.bsp_render;
        let light_map = render.light_maps.get(index)?;
        let surface = render.surfaces.get(usize::try_from(light_map.surface).ok()?)?;
        let size = light_map.size;
        let mut run = 0;
        let mut previous: Option<&str> = None;
        let bitmaps = light_map
            .bitmaps
            .iter()
            .map(|record| {
                let name = record.actor.as_deref();
                run = if name.is_some() && name == previous { run + 1 } else { 0 };
                previous = name;
                let sun = name.is_some_and(|name| self.suns.contains(name));
                let light = name.and_then(|name| self.lights.get(name));
                BitmapEntry {
                    bitmap: LightBitmap { bits: record.bits.clone(), rows: record.size[1], stride: record.stride, min_x: record.min[0], min_y: record.min[1], max_x: record.max[0], max_y: record.max[1] },
                    time: run,
                    actor_present: sun || light.is_some(),
                    deleted: name.is_some_and(deleted),
                    editor_hidden: false,
                    light: if sun { self.sun_render(hour) } else { light.map_or_else(LightRender::default, |light| self.light_render(light)) },
                }
            })
            .collect();
        Some(LightMapSource {
            size,
            surface: Surface { flags: surface.flags, normal: [surface.plane[0], surface.plane[1], surface.plane[2]], distance: surface.plane[3] },
            geometry: LightMapGeometry { size, base: light_map.base, axis_x: light_map.axis_x, axis_y: light_map.axis_y },
            matrix: light_map.world_to_lightmap,
            zone: self.zone_light(light_map.zone),
            bitmaps,
            dynamic: Vec::new(),
        })
    }

    pub fn atlas(&mut self, group: usize, slice: usize, count: usize, deleted: &dyn Fn(&str) -> bool) -> Vec<u32> {
        let hour = slice_hour(slice, count);
        let mut atlas = vec![0u32; ATLAS_SIZE * ATLAS_SIZE];
        let env = self.env;
        let ambient = move |time: f32| env.bsp_ambient_color(time);
        let environment = Environment { editor: false, time: f64::from(hour), ambient: &ambient };
        let slice_key = TimeSlice::Timed { index: slice as i32, count: count as i32 };
        let members: Vec<usize> = self.level.bsp_render.light_maps.iter().enumerate().filter(|(_, light_map)| usize::try_from(light_map.texture).is_ok_and(|texture| texture == group)).map(|(index, _)| index).collect();
        for index in members {
            let Some(source) = self.source(index, hour, deleted) else { continue };
            let light_map = &self.level.bsp_render.light_maps[index];
            let (Ok(x), Ok(y)) = (usize::try_from(light_map.offset[0]), usize::try_from(light_map.offset[1])) else { continue };
            let key = cache_key(MODEL_CACHE_INDEX, index as i32, light_map.zone, slice_key);
            get_texture_data(&mut self.cache, key, &source, slice_key, &environment, &mut atlas[y * ATLAS_SIZE + x..], ATLAS_SIZE);
        }
        atlas
    }

    pub fn group(&mut self, group: usize, deleted: &dyn Fn(&str) -> bool) -> Option<LightmapGroup> {
        let count = self.level.bsp_render.groups.get(group)?.textures.len();
        let slices = (0..count)
            .map(|slice| {
                let rgba: Vec<u8> = self.atlas(group, slice, count, deleted).iter().flat_map(|pixel| [(pixel >> 16) as u8, (pixel >> 8) as u8, *pixel as u8, (pixel >> 24) as u8]).collect();
                vec![rgba]
            })
            .collect();
        Some(LightmapGroup { width: ATLAS_SIZE as u32, height: ATLAS_SIZE as u32, format: TextureFormatKind::Rgba8, slices })
    }
}
