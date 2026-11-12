use crate::options::EnchantParticles;
use crate::pawn::WeaponEffect;
use ue2_level::Emitter;

pub fn enabled(show: Option<i32>, enchant: u8, hero: bool) -> bool {
    show.is_some_and(|show| show >= 0 && (hero || show <= i32::from(enchant)))
}

pub fn init_emitter(emitter: &mut Emitter, effect: &WeaponEffect, weapon_scale: f32, particles: EnchantParticles) {
    apply_enchant(emitter, effect, particles);
    apply_weapon_scale(emitter, effect, weapon_scale);
}

pub fn apply_enchant(emitter: &mut Emitter, effect: &WeaponEffect, particles: EnchantParticles) {
    if !effect.hero {
        set_opacity(emitter, particles.opacity);
        set_max_particles(emitter, particles.num);
    }
}

pub fn apply_weapon_scale(emitter: &mut Emitter, effect: &WeaponEffect, weapon_scale: f32) {
    for sprite in &mut emitter.sprites {
        for range in &mut sprite.start_size {
            for value in range {
                *value = effect.size_scale * *value * weapon_scale;
            }
        }
        for range in &mut sprite.start_velocity {
            for value in range {
                *value = effect.velocity_scale * *value * weapon_scale;
            }
        }
        for value in &mut sprite.start_offset {
            *value *= weapon_scale;
        }
    }
}

pub const HERO_AURA_SIZE_PER_RADIUS: f32 = f32::from_bits(0x3de3_8e39);

pub fn set_size_scale(emitter: &mut Emitter, scale: f32) {
    for sprite in &mut emitter.sprites {
        let ranges = sprite.start_size.iter_mut().chain(&mut sprite.start_location).chain(&mut sprite.start_location_polar).chain(std::iter::once(&mut sprite.sphere_radius)).chain(&mut sprite.start_velocity);
        for range in ranges {
            for value in range {
                *value *= scale;
            }
        }
        for value in &mut sprite.acceleration {
            *value *= scale;
        }
        if let Some(beam) = &mut sprite.beam {
            for range in beam.low_noise.iter_mut().chain(&mut beam.high_noise) {
                for value in range {
                    *value *= scale;
                }
            }
        }
    }
}

pub fn adjust_particle_life(emitter: &mut Emitter, life: f32) {
    let longest = emitter.sprites.iter().filter(|sprite| sprite.forced_life_time).fold(0.0_f32, |longest, sprite| if sprite.lifetime[1] > longest { sprite.lifetime[1] } else { longest });
    let delta = life - longest;
    for sprite in emitter.sprites.iter_mut().filter(|sprite| sprite.forced_life_time) {
        sprite.lifetime[0] += delta;
        sprite.lifetime[1] += delta;
        if sprite.forced_fade {
            sprite.fade_out_start += delta;
        }
    }
    for sprite in emitter.sprites.iter_mut().filter(|sprite| sprite.forced_max_particles) {
        let center = (sprite.lifetime[0] + sprite.lifetime[1]) * 0.5;
        let whole = sprite.initial_particles_per_second as i32;
        sprite.max_particles = ((f64::from(whole) * (f64::from(life) - f64::from(center))) as i32) as u32;
    }
}

pub fn set_delayed(emitter: &mut Emitter, delay: f32, all: bool, index: i32) {
    for (position, sprite) in emitter.sprites.iter_mut().enumerate() {
        if all || index == position as i32 {
            sprite.initial_delay[0] += delay;
            sprite.initial_delay[1] += delay;
        }
    }
}

pub fn relative_location(effect: &WeaponEffect, weapon_scale: f32) -> [f32; 3] {
    effect.offset.map(|value| weapon_scale * value)
}

fn set_opacity(emitter: &mut Emitter, opacity: f32) {
    let opacity = if opacity <= 0.0 { 0.001 } else { opacity };
    for sprite in &mut emitter.sprites {
        sprite.opacity *= opacity;
    }
}

fn set_max_particles(emitter: &mut Emitter, num: f32) {
    if num.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
        return;
    }
    for sprite in &mut emitter.sprites {
        if sprite.initial_particles_per_second != 0.0 && sprite.max_particles != 0 {
            sprite.max_particles = (f64::from(sprite.max_particles) * f64::from(num)) as u32;
            sprite.initial_particles_per_second *= num;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ue2_level::SpriteEmitter;

    fn effect(hero: bool) -> WeaponEffect {
        WeaponEffect { class: "LineageEffect.c_u007".to_owned(), bone: "Weapon_R_Bone", right: true, offset: [1.0, 2.0, 3.0], size_scale: 0.8, velocity_scale: 0.5, hero }
    }

    fn emitter() -> Emitter {
        let sprite = SpriteEmitter { max_particles: 8, initial_particles_per_second: 4.0, opacity: 0.5, start_size: [[3.0, 4.0]; 3], start_velocity: [[4.0, 4.0], [0.0, 1.0], [0.0, 0.0]], start_offset: [25.0, 0.0, 0.0], ..Default::default() };
        let idle = SpriteEmitter { max_particles: 5, initial_particles_per_second: 0.0, ..Default::default() };
        Emitter { sprites: vec![sprite, idle], ..Default::default() }
    }

    #[test]
    fn fnweaponeffect_init_10493790_applies_env_int_and_weapon_scale() {
        let mut e = emitter();
        init_emitter(&mut e, &effect(false), 2.0, EnchantParticles { opacity: 0.5, num: 6.5 });
        let s = &e.sprites[0];
        assert_eq!((s.max_particles, s.initial_particles_per_second, s.opacity), (52, 26.0, 0.25));
        assert_eq!(s.start_size, [[4.8, 6.4]; 3]);
        assert_eq!(s.start_velocity, [[4.0, 4.0], [0.0, 1.0], [0.0, 0.0]]);
        assert_eq!(s.start_offset, [50.0, 0.0, 0.0]);
        assert_eq!(e.sprites[1].max_particles, 5);
        assert_eq!(relative_location(&effect(false), 2.0), [2.0, 4.0, 6.0]);
    }

    #[test]
    fn set_opacity_105d3a60_and_set_max_particles_105d3ad0_edges() {
        let mut e = emitter();
        init_emitter(&mut e, &effect(false), 1.0, EnchantParticles { opacity: 0.0, num: 0.0 });
        assert_eq!((e.sprites[0].max_particles, e.sprites[0].initial_particles_per_second), (8, 4.0));
        assert!((e.sprites[0].opacity - 0.0005).abs() < 1e-9);
        let mut hero = emitter();
        init_emitter(&mut hero, &effect(true), 1.0, EnchantParticles { opacity: 0.1, num: 9.0 });
        assert_eq!((hero.sprites[0].max_particles, hero.sprites[0].opacity), (8, 0.5));
        let mut odd = emitter();
        set_max_particles(&mut odd, 1.3);
        assert_eq!(odd.sprites[0].max_particles, 10);
    }

    #[test]
    fn set_size_scale_105d4460_scales_every_spawn_range() {
        let beam = ue2_level::BeamShape { low_noise: [[1.0, 2.0]; 3], high_noise: [[3.0, 4.0]; 3], ..Default::default() };
        let sprite = SpriteEmitter {
            start_size: [[1.0, 2.0]; 3],
            start_location: [[-1.0, 1.0]; 3],
            start_location_polar: [[0.0, 3.0]; 3],
            sphere_radius: [15.0, 20.0],
            start_velocity: [[5.0, 10.0]; 3],
            acceleration: [0.0, 0.0, 10.0],
            start_offset: [7.0, 0.0, 0.0],
            beam: Some(beam),
            ..Default::default()
        };
        let mut e = Emitter { sprites: vec![sprite], ..Default::default() };
        set_size_scale(&mut e, 2.0);
        let s = &e.sprites[0];
        assert_eq!(s.start_size, [[2.0, 4.0]; 3]);
        assert_eq!(s.start_location, [[-2.0, 2.0]; 3]);
        assert_eq!(s.start_location_polar, [[0.0, 6.0]; 3]);
        assert_eq!(s.sphere_radius, [30.0, 40.0]);
        assert_eq!(s.start_velocity, [[10.0, 20.0]; 3]);
        assert_eq!(s.acceleration, [0.0, 0.0, 20.0]);
        assert_eq!(s.start_offset, [7.0, 0.0, 0.0]);
        let beam = s.beam.as_ref().unwrap();
        assert_eq!((beam.low_noise, beam.high_noise), ([[2.0, 4.0]; 3], [[6.0, 8.0]; 3]));
        assert!((HERO_AURA_SIZE_PER_RADIUS - 1.0 / 9.0).abs() < 1e-8);
    }

    #[test]
    fn fun_10496670_creates_from_enchant_effect_show_or_hero_weapon() {
        assert!(!enabled(None, 20, true));
        assert!(!enabled(Some(1), 0, false));
        assert!(enabled(Some(1), 1, false));
        assert!(enabled(Some(1), 0, true));
        assert!(!enabled(Some(-1), 0, true));
        assert!(enabled(Some(0), 0, false));
    }
}
