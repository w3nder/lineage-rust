use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Vec3};
use killer_assets::timeenv::{AmbientKind, LightKind, PHASE_NIGHT, TimeEnv, phase};
use killer_assets::weather::Weather;
use std::f32::consts::{FRAC_PI_2, PI};

const SUN_AZIMUTH_DEG: f32 = 30.0;
const SLOPE_SUN_ANGLE_DEG: f32 = 30.0;
pub const MAX_CLOUD_LAYERS: usize = 4;
const SUN_SCALE_UNSET: f32 = 1.0;
const SUN_SCALE_NIGHT_HOUR: f32 = 6.0;
const MOON_SCALE_DAY_HOUR: f32 = 0.0;
const SUN_COLOR_DAWN_HOUR: f32 = 6.0;
const SUN_COLOR_LATE_HOUR: f32 = 23.0;
const BYTE: f32 = 255.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Environment {
    pub sun_dir: Vec3,
    pub sun_color: [f32; 3],
    pub sun_brightness: f32,
    pub static_mesh_ambient: [f32; 3],
    pub bsp_ambient: [f32; 3],
    pub terrain_ambient: [f32; 3],
    pub actor_ambient: [f32; 3],
    pub terrain_sun: ([f32; 3], f32),
    pub hour: f32,
    pub sky: SkyState,
}

/// What `UpdateSkyColors`, `ANSun::Tick`/`RenderSun` and `ANMoon::Tick`/`RenderMoon` derive from the hour.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkyState {
    pub phase: u8,
    pub sky_box: [f32; 3],
    pub hazering: [f32; 3],
    pub clouds: [[f32; 3]; MAX_CLOUD_LAYERS],
    /// `RenderSun` color in 0..1; black in the night phase.
    pub sun_sprite: [f32; 3],
    /// `GetSunScale` after the night-phase override; multiplies `NSun.SunScale · 8 · 16` into the half width.
    pub sun_scale: f32,
    /// `GetMoonScale(0)` by day, `GetMoonScale(h)` in the night phase.
    pub moon_scale: f32,
    pub sun_visible: bool,
    pub moon_visible: bool,
    /// Unit vector from the sky zone to the sun sprite, slope applied.
    pub sun_body: Vec3,
    pub moon_body: Vec3,
    /// `NSun.Position` direction: `(sinθ·cosφ, sinθ·sinφ, cosθ)` without the slope, for the lens flare trace.
    pub sun_trace: Vec3,
}

impl Environment {
    pub fn from_weather(weather: &Weather, hour: f32) -> Environment {
        let light = |kind: LightKind| {
            (weather.mix3(hour, |env| env.sun_light(kind, hour).0), weather.mix1(hour, |env| env.sun_light(kind, hour).1))
        };
        let ambient = |kind: AmbientKind| weather.mix3(hour, |env| env.ambient(kind, hour));
        let (sun_color, sun_brightness) = light(LightKind::Actor);
        Environment {
            hour,
            sun_dir: sun_direction(hour),
            sun_color,
            sun_brightness,
            static_mesh_ambient: ambient(AmbientKind::StaticMesh),
            bsp_ambient: ambient(AmbientKind::Bsp),
            terrain_ambient: ambient(AmbientKind::Terrain),
            actor_ambient: ambient(AmbientKind::Actor),
            terrain_sun: light(LightKind::Terrain),
            sky: sky_state_weather(weather, hour),
        }
    }

    pub fn from_time(env: &TimeEnv, hour: f32) -> Environment {
        let (sun_color, sun_brightness) = env.sun_light(LightKind::Actor, hour);
        Environment {
            hour,
            sun_dir: sun_direction(hour),
            sun_color,
            sun_brightness,
            static_mesh_ambient: env.ambient(AmbientKind::StaticMesh, hour),
            bsp_ambient: env.ambient(AmbientKind::Bsp, hour),
            terrain_ambient: env.ambient(AmbientKind::Terrain, hour),
            actor_ambient: env.ambient(AmbientKind::Actor, hour),
            terrain_sun: env.sun_light(LightKind::Terrain, hour),
            sky: sky_state(env, hour),
        }
    }
}

fn bytes(rgb: [f32; 3]) -> [f32; 3] {
    rgb.map(|c| c / BYTE)
}

pub fn sky_state(env: &TimeEnv, hour: f32) -> SkyState {
    sky_state_with(hour, &|get| get(env), &|scale, at| scale(env, at))
}

pub fn sky_state_weather(weather: &Weather, hour: f32) -> SkyState {
    sky_state_with(hour, &|get| weather.mix3(hour, get), &|scale, at| weather.scale(at, scale))
}

type ColorGetter<'a> = &'a dyn Fn(&TimeEnv) -> [f32; 3];
type ColorMixer<'a> = &'a dyn Fn(ColorGetter) -> [f32; 3];
type ScaleGetter<'a> = &'a dyn Fn(&TimeEnv, f32) -> Option<f32>;
type ScaleMixer<'a> = &'a dyn Fn(ScaleGetter, f32) -> Option<f32>;

fn sky_state_with(hour: f32, color: ColorMixer, scale: ScaleMixer) -> SkyState {
    let hour = hour.rem_euclid(24.0);
    let phase = phase(hour);
    let night = phase == PHASE_NIGHT;
    let mut sun_scale = scale(&|env, at| env.sun_scale(at), hour).unwrap_or(SUN_SCALE_UNSET);
    if night {
        sun_scale = scale(&|env, at| env.sun_scale(at), SUN_SCALE_NIGHT_HOUR).unwrap_or(sun_scale);
    }
    let mut moon_scale = scale(&|env, at| env.moon_scale(at), hour).unwrap_or(SUN_SCALE_UNSET);
    if !night {
        moon_scale = scale(&|env, at| env.moon_scale(at), MOON_SCALE_DAY_HOUR).unwrap_or(moon_scale);
    }
    let sun_sprite = if !night {
        bytes(color(&|env| env.sun_color(hour)))
    } else if hour > 5.0 && hour < 7.0 {
        bytes(color(&|env| env.sun_color(SUN_COLOR_DAWN_HOUR)))
    } else if hour > 0.0 && hour < 1.0 {
        bytes(color(&|env| env.sun_color(SUN_COLOR_LATE_HOUR)))
    } else {
        [0.0; 3]
    };
    let mut clouds = [[0.0; 3]; MAX_CLOUD_LAYERS];
    for (index, cloud) in clouds.iter_mut().enumerate() {
        *cloud = bytes(color(&|env| env.cloud_color(index, hour)));
    }
    SkyState {
        phase,
        sky_box: bytes(color(&|env| env.sky_box_color(hour))),
        hazering: bytes(color(&|env| env.hazering_color(hour))),
        clouds,
        sun_sprite,
        sun_scale,
        moon_scale,
        sun_visible: !(hour > 1.0 && hour < 5.0),
        moon_visible: !(hour > 7.0 && hour < 23.0),
        sun_body: body_direction(sun_theta(hour)),
        moon_body: body_direction(moon_theta(hour)),
        sun_trace: unit_direction(sun_theta(hour)),
    }
}

fn unit_direction(theta: f32) -> Vec3 {
    let phi = SUN_AZIMUTH_DEG.to_radians();
    Vec3::new(theta.sin() * phi.cos(), theta.sin() * phi.sin(), theta.cos())
}

fn body_direction(theta: f32) -> Vec3 {
    (Mat3::from_rotation_x(SLOPE_SUN_ANGLE_DEG.to_radians()) * unit_direction(theta)).normalize_or(Vec3::Z)
}

pub fn sun_direction(hour: f32) -> Vec3 {
    let hour = hour.rem_euclid(24.0);
    let theta = if hour >= 22.0 { moon_theta(hour) } else { sun_theta(hour) };
    let phi = SUN_AZIMUTH_DEG.to_radians();
    let v = Vec3::new(theta.sin() * phi.cos(), theta.sin() * phi.sin(), theta.cos());
    -(Mat3::from_rotation_x(SLOPE_SUN_ANGLE_DEG.to_radians()) * v)
}

fn sun_theta(hour: f32) -> f32 {
    if hour >= 5.0 {
        (hour - 6.0) * PI / 18.0 - FRAC_PI_2
    } else if hour < 1.0 {
        (hour + 18.0) * PI / 18.0 - FRAC_PI_2
    } else {
        -PI
    }
}

fn moon_theta(hour: f32) -> f32 {
    if (7.0..23.0).contains(&hour) { -PI } else { hour * PI / 6.0 - FRAC_PI_2 }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct EnvironmentUniform {
    pub sun_dir: [f32; 4],
    pub sun: [f32; 4],
    pub static_mesh_ambient: [f32; 4],
    pub bsp_ambient: [f32; 4],
    pub terrain_ambient: [f32; 4],
    pub actor_ambient: [f32; 4],
    pub terrain_sun: [f32; 4],
    pub hour: [f32; 4],
    /// The frame's `SetDistanceFog` color; `a = 1` when enabled.
    pub fog_color: [f32; 4],
    /// `(start, end)` in view depth.
    pub fog_range: [f32; 4],
}

fn with_w(rgb: [f32; 3], w: f32) -> [f32; 4] {
    [rgb[0], rgb[1], rgb[2], w]
}

impl From<&Environment> for EnvironmentUniform {
    fn from(e: &Environment) -> Self {
        let d = e.sun_dir.normalize_or(Vec3::Z);
        Self {
            sun_dir: [d.x, d.y, d.z, 0.0],
            sun: with_w(e.sun_color, e.sun_brightness),
            static_mesh_ambient: with_w(e.static_mesh_ambient, 0.0),
            bsp_ambient: with_w(e.bsp_ambient, 0.0),
            terrain_ambient: with_w(e.terrain_ambient, 0.0),
            actor_ambient: with_w(e.actor_ambient, 0.0),
            terrain_sun: with_w(e.terrain_sun.0, e.terrain_sun.1),
            hour: [e.hour, 0.0, 0.0, 0.0],
            fog_color: [0.0; 4],
            fog_range: [0.0; 4],
        }
    }
}

#[cfg(test)]
pub(crate) const TIMEENV_FIXTURE: &str = "\
[SunColor]
Color1=(T=6,R=255,G=122,B=23)
Color2=(T=8,R=255,G=219,B=151)
Color3=(T=15,R=255,G=219,B=151)
Color4=(T=20,R=255,G=174,B=125)
Color5=(T=24,R=162,G=65,B=0)

[SunScale]
Scale1=(T=6,S=2.0)
Scale2=(T=8,S=0.5)
Scale3=(T=15,S=0.2)
Scale4=(T=20,S=0.2)
Scale5=(T=24,S=2.0)

[MoonScale]
Scale1=(T=0,S=4.5)
Scale2=(T=2,S=3.5)
Scale3=(T=3,S=2.5)
Scale4=(T=4,S=3.5)
Scale5=(T=6,S=4.5)

[SkyBoxColor]
COLOR1=(T=0,R=40,G=49,B=70)
COLOR12=(T=11,R=66,G=124,B=176)
COLOR13=(T=12,R=66,G=124,B=176)
COLOR14=(T=13,R=66,G=124,B=176)

[HazeringColor]
COLOR1=(T=0,R=48,G=62,B=84)
COLOR13=(T=12,R=188,G=203,B=224)

[CloudColor1]
COLOR1=(T=0,R=68,G=79,B=115)
COLOR13=(T=12,R=255,G=255,B=255)

[TerrainAmbient]
COLOR1=(T=0,R=51,G=98,B=122)
COLOR13=(T=12,R=120,G=120,B=120)
COLOR14=(T=13,R=120,G=120,B=120)

[ActorAmbient]
COLOR1=(T=0,R=75,G=103,B=131)
COLOR13=(T=12,R=150,G=150,B=150)
COLOR14=(T=13,R=150,G=150,B=150)

[StaticMeshAmbient]
COLOR1=(T=0,R=75,G=103,B=131)
COLOR13=(T=12,R=120,G=120,B=120)
COLOR14=(T=13,R=120,G=120,B=120)

[BSPAmbient]
COLOR1=(T=0,R=64,G=88,B=111)
COLOR13=(T=12,R=120,G=120,B=120)
COLOR14=(T=13,R=120,G=120,B=120)

[HSVTerrainLight]
Light1=( T=0, Hue=138, Sat=182, Bri=0 )
Light13=( T=12, Hue=1, Sat=255, Bri=200 )
Light14=( T=13, Hue=1, Sat=255, Bri=200 )

[HSVActorLight]
Light1=( T=0, Hue=138, Sat=182, Bri=0 )
Light13=( T=12, Hue=1, Sat=255, Bri=160 )
Light14=( T=13, Hue=1, Sat=255, Bri=160 )
";

#[cfg(test)]
mod tests {
    use super::*;
    use killer_assets::timeenv::fgethsv;

    fn close(actual: Vec3, expected: [f32; 3]) {
        close3(actual.to_array(), expected);
    }

    fn close3(actual: [f32; 3], expected: [f32; 3]) {
        for (a, e) in actual.iter().zip(expected) {
            assert!((a - e).abs() < 1e-4, "{actual:?} vs {expected:?}");
        }
    }

    fn tilt_and_negate(v: [f32; 3]) -> [f32; 3] {
        let (s, c) = (30f32.to_radians().sin(), 30f32.to_radians().cos());
        [-v[0], -(c * v[1] - s * v[2]), -(s * v[1] + c * v[2])]
    }

    fn from_theta(theta: f32) -> [f32; 3] {
        let (s, c) = (30f32.to_radians().sin(), 30f32.to_radians().cos());
        tilt_and_negate([theta.sin() * c, theta.sin() * s, theta.cos()])
    }

    #[test]
    fn at_fifteen_the_sun_is_at_the_zenith_before_the_slope_and_points_down_after_it() {
        let (s, c) = (30f32.to_radians().sin(), 30f32.to_radians().cos());
        let dir = sun_direction(15.0);
        close(dir, [0.0, s, -c]);
        assert!(dir.z < 0.0);
    }

    #[test]
    fn at_six_the_sun_is_on_the_horizon_before_the_slope() {
        let (s, c) = (30f32.to_radians().sin(), 30f32.to_radians().cos());
        close(sun_direction(6.0), tilt_and_negate([-c, -s, 0.0]));
        close(sun_direction(6.0), [c, s * c, s * s]);
    }

    #[test]
    fn at_noon_theta_is_minus_thirty_degrees() {
        close(sun_direction(12.0), from_theta(-30f32.to_radians()));
    }

    #[test]
    fn between_one_and_five_the_sun_sits_at_the_nadir() {
        close(sun_direction(1.0), from_theta(-PI));
        close(sun_direction(4.999), from_theta(-PI));
        close(sun_direction(5.0), from_theta(-PI / 18.0 - FRAC_PI_2));
    }

    #[test]
    fn before_one_the_sun_continues_the_previous_day() {
        close(sun_direction(0.5), from_theta(18.5 * PI / 18.0 - FRAC_PI_2));
        assert!((sun_direction(0.5) - sun_direction(24.5)).length() < 1e-5);
    }

    #[test]
    fn from_twenty_two_the_moon_phase_drives_the_direction() {
        close(sun_direction(22.0), from_theta(-PI));
        close(sun_direction(23.5), from_theta(23.5 * PI / 6.0 - FRAC_PI_2));
    }

    #[test]
    fn the_sky_state_follows_render_sun_and_render_moon_by_phase() {
        let env = TimeEnv::parse(TIMEENV_FIXTURE);
        let noon = sky_state(&env, 12.0);
        assert_eq!(noon.phase, 2);
        assert!((noon.sun_scale - 0.329).abs() < 1e-3, "{}", noon.sun_scale);
        assert_eq!(noon.moon_scale, 4.5);
        assert!(noon.sun_visible && !noon.moon_visible);
        close3(noon.sun_sprite, [1.0, 219.0 / 255.0, 151.0 / 255.0]);
        close3(noon.sky_box, [66.0 / 255.0, 124.0 / 255.0, 176.0 / 255.0]);
        assert_eq!(noon.sun_body, -sun_direction(12.0));
        let night = sky_state(&env, 23.0);
        assert_eq!(night.phase, 3);
        assert_eq!(night.sun_scale, 2.0);
        assert_eq!(night.moon_scale, 1.0);
        assert_eq!(night.sun_sprite, [0.0; 3]);
        assert!(night.sun_visible && night.moon_visible);
        let small_hours = sky_state(&env, 3.0);
        assert!(!small_hours.sun_visible && small_hours.moon_visible);
        assert_eq!(small_hours.sun_scale, 1.0);
        assert_eq!(small_hours.moon_scale, 4.5);
    }

    #[test]
    fn from_time_reads_every_table_at_noon() {
        let env = Environment::from_time(&TimeEnv::parse(TIMEENV_FIXTURE), 12.0);
        let expected_sun = fgethsv(1, 255, 255);
        close3(env.sun_color, expected_sun);
        close3(env.terrain_sun.0, expected_sun);
        close3(env.static_mesh_ambient, [120.0 / 255.0; 3]);
        close3(env.bsp_ambient, [120.0 / 255.0; 3]);
        close3(env.terrain_ambient, [120.0 / 255.0; 3]);
        close3(env.actor_ambient, [150.0 / 255.0; 3]);
        assert!((env.sun_brightness - 160.0 / 255.0).abs() < 1e-4);
        assert!((env.terrain_sun.1 - 200.0 / 255.0).abs() < 1e-4);
        assert_eq!(env.sun_dir, sun_direction(12.0));
    }

    #[test]
    fn at_midnight_the_static_mesh_ambient_is_the_night_blue_and_the_sun_is_off() {
        let env = Environment::from_time(&TimeEnv::parse(TIMEENV_FIXTURE), 0.0);
        close3(env.static_mesh_ambient, [75.0 / 255.0, 103.0 / 255.0, 131.0 / 255.0]);
        assert_eq!(env.sun_brightness, 0.0);
        assert_eq!(env.terrain_sun.1, 0.0);
    }

    #[test]
    fn uniform_packs_colors_with_brightness_in_the_fourth_lane() {
        let env = Environment::from_time(&TimeEnv::parse(TIMEENV_FIXTURE), 12.0);
        let uniform = EnvironmentUniform::from(&env);
        assert_eq!(uniform.sun[3], env.sun_brightness);
        assert_eq!(uniform.terrain_sun[3], env.terrain_sun.1);
        assert_eq!(uniform.sun_dir[3], 0.0);
        assert!((uniform.sun_dir[2] - env.sun_dir.z).abs() < 1e-6);
    }
}
