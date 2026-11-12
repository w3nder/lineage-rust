use crate::option_video::{VideoPage, VideoSettings, applied_fog_ranges};

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    Text(String),
    Integer(i32),
    Decimal(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolutionRequest {
    pub width: i32,
    pub height: i32,
    pub bytes_per_pixel: i32,
    pub refresh_rate: Option<i32>,
    pub fullscreen: i32,
    pub viewport_argument: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolutionParams {
    pub bytes_per_pixel: i32,
    pub width: i32,
    pub height: i32,
    pub refresh_rate: i32,
    pub fullscreen: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoApplyState {
    pub settings: VideoSettings,
    pub actor_clipping: f32,
    pub static_mesh_clipping: f32,
    pub static_mesh_lod_clipping: f32,
    pub fog: (f32, f32),
    pub actor_count: i32,
    pub screenshot_quality: i32,
}

pub trait VideoApplyBackend {
    fn fullscreen(&mut self) -> i32;
    fn uses_trilinear(&mut self) -> i32;
    fn set_trilinear(&mut self, value: i32);
    fn delete_all_textures(&mut self, settings: &VideoSettings);
    fn modes(&mut self) -> Vec<(i32, i32, i32)>;
    fn refresh_rates(&mut self, mode: (i32, i32, i32)) -> Vec<i32>;
    fn set_resolution(&mut self, request: ResolutionRequest, settings: &mut VideoSettings) -> bool;
    fn resolution_params(&mut self) -> ResolutionParams;
    fn update_gamma(&mut self, value: f32);
    fn set_option(&mut self, section: &str, key: &str, value: ConfigValue);
    fn read_float(&mut self, file: &str, section: &str, key: &str) -> Option<f32>;
}

fn boolean(value: bool) -> ConfigValue {
    ConfigValue::Text(value.to_string())
}

fn integer_text(value: i32) -> ConfigValue {
    ConfigValue::Text(value.to_string())
}

fn effective_texture(settings: &VideoSettings) -> i32 {
    if settings.keep_min_frame_rate != 0 { 2 } else { settings.texture_detail }
}

impl VideoPage {
    pub fn apply(&mut self, state: &mut VideoApplyState, backend: &mut impl VideoApplyBackend) {
        let fullscreen = backend.fullscreen();
        let settings = &mut state.settings;
        let old_texture = effective_texture(settings);
        backend.set_option("Video", "IsKeepMinFrameRate", boolean(self.checks[0].checked));
        settings.keep_min_frame_rate = i32::from(self.checks[0].checked);
        let old_trilinear = backend.uses_trilinear();
        settings.trilinear = i32::from(self.checks[3].checked);
        backend.set_option("Video", "UseTrilinear", boolean(self.checks[3].checked));
        if settings.trilinear != old_trilinear {
            backend.set_trilinear(settings.trilinear);
        }
        settings.texture_detail = match self.combos[3].selected { 0 => 0, 1 => 1, _ => 2 };
        backend.set_option("Video", "TextureDetail", integer_text(settings.texture_detail));
        if old_texture != effective_texture(settings) {
            backend.delete_all_textures(settings);
        }
        settings.reduce_pawn_lod = i32::from(self.combos[4].selected != 0);
        backend.set_option("Video", "ModelDetail", integer_text(settings.reduce_pawn_lod));
        settings.skip_anim = i32::from(self.combos[5].selected != 0);
        backend.set_option("Video", "SkipAnim", integer_text(settings.skip_anim));
        self.apply_resolution(settings, fullscreen, backend);
        if let Some(previous) = settings.client_brightness {
            let value = [1.2, 1.0, 0.8, 0.6, 0.4].get(self.combos[2].selected as usize).copied().unwrap_or(previous);
            if value != previous {
                settings.client_brightness = Some(value);
                backend.set_option("Video", "Gamma", ConfigValue::Decimal(value));
                backend.update_gamma(value);
            }
        }
        if let Some(value) = [3.0, 2.5, 2.0, 1.5, 1.0].get(self.combos[7].selected as usize) {
            settings.pawn_clipping = *value;
        }
        backend.set_option("ClippingRange", "Pawn", ConfigValue::Decimal(settings.pawn_clipping));
        let post = self.combos[12].selected;
        if (0..=5).contains(&post) {
            settings.post_process_type = post;
        }
        backend.set_option("Video", "PostProc", ConfigValue::Integer(post));
        if let Some((terrain, mesh, actor)) = [(8.0, 4.0, 6.0), (7.0, 3.5, 5.5), (6.0, 3.0, 5.0), (5.0, 2.5, 4.5), (4.0, 2.0, 4.0)].get(self.combos[6].selected as usize) {
            settings.terrain_clipping = *terrain;
            state.static_mesh_clipping = *mesh;
            state.static_mesh_lod_clipping = *actor;
            state.actor_clipping = *actor;
        }
        for (key, value) in [("Terrain", settings.terrain_clipping), ("Actor", state.actor_clipping), ("StaticMesh", state.static_mesh_clipping), ("StaticMeshLod", state.static_mesh_lod_clipping)] {
            backend.set_option("ClippingRange", key, ConfigValue::Decimal(value));
        }
        settings.render_deco = i32::from(self.checks[2].checked);
        backend.set_option("Video", "RenderDeco", boolean(self.checks[2].checked));
        settings.projector = i32::from(self.checks[1].checked);
        backend.set_option("Video", "PawnShadow", boolean(self.checks[1].checked));
        settings.water_effect = i32::from(self.combos[9].selected != 0);
        backend.set_option("L2WaterEffect", "IsUseEffect", boolean(settings.water_effect != 0));
        settings.water_effect_type = self.combos[9].selected;
        backend.set_option("L2WaterEffect", "EffectType", integer_text(settings.water_effect_type));
        backend.set_option("Video", "IsKeepMinFrameRate", boolean(self.checks[0].checked));
        settings.keep_min_frame_rate = i32::from(self.checks[0].checked);
        state.fog = applied_fog_ranges(settings.terrain_clipping, settings.keep_min_frame_rate, |file, section, key| backend.read_float(file, section, key));
        if let Some((option, count)) = [(6, 10000), (5, 400), (4, 300), (3, 200), (2, 100), (1, 50)].get(self.combos[8].selected as usize) {
            settings.actor_limit = *option;
            state.actor_count = *count;
        }
        backend.set_option("Video", "RenderActorLimited", integer_text(settings.actor_limit));
        if self.combos[10].selected != settings.antialiasing {
            settings.antialiasing = self.combos[10].selected;
            backend.set_option("Video", "AntiAliasing", ConfigValue::Integer(settings.antialiasing));
            let params = backend.resolution_params();
            backend.set_resolution(ResolutionRequest {
                width: params.width, height: params.height, bytes_per_pixel: params.bytes_per_pixel,
                refresh_rate: None, fullscreen: params.fullscreen, viewport_argument: 1,
            }, settings);
        }
        state.screenshot_quality = self.combos[11].selected;
        backend.set_option("Game", "ScreenShotQuality", ConfigValue::Integer(state.screenshot_quality));
    }

    fn apply_resolution(&mut self, settings: &mut VideoSettings, fullscreen: i32, backend: &mut impl VideoApplyBackend) {
        let modes = backend.modes();
        let index = self.combos[0].selected as usize;
        let Some(&mode) = modes.get(..modes.len().min(100)).and_then(|modes| modes.get(index)) else { return };
        let rates = backend.refresh_rates(mode);
        let index = self.combos[1].selected as usize;
        let Some(&rate) = rates.get(..rates.len().min(100)).and_then(|rates| rates.get(index)) else { return };
        if mode != settings.mode || rate != settings.refresh_rate {
            let request = ResolutionRequest { width: mode.0, height: mode.1, bytes_per_pixel: mode.2 / 8, refresh_rate: Some(rate), fullscreen, viewport_argument: 1 };
            if backend.set_resolution(request, settings) {
                settings.mode.2 = mode.2;
                settings.refresh_rate = rate;
            } else if backend.set_resolution(ResolutionRequest { width: 1024, height: 768, bytes_per_pixel: 4, refresh_rate: Some(60), fullscreen, viewport_argument: 1 }, settings) {
                settings.mode.2 = 32;
                settings.refresh_rate = 60;
                self.combos[0].clear_entries();
                self.combos[1].clear_entries();
                let modes = backend.modes();
                self.fill_resolution_modes(settings, modes);
                let rates = backend.refresh_rates(settings.mode);
                self.fill_resolution_rates(settings, rates);
            }
        }
        for (key, value) in [("GamePlayViewportX", settings.mode.0), ("GamePlayViewportY", settings.mode.1), ("ColorBits", settings.mode.2), ("RefreshRate", settings.refresh_rate)] {
            backend.set_option("Video", key, integer_text(value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Debug, PartialEq)]
    enum Event {
        Fullscreen,
        ReadTrilinear,
        Trilinear(i32),
        Textures(i32),
        Modes,
        Rates((i32, i32, i32)),
        Resolution(ResolutionRequest),
        Params,
        Gamma(f32),
        Write(String, String, ConfigValue),
        Read(String, String, String),
    }

    struct Backend {
        events: Vec<Event>,
        outcomes: VecDeque<bool>,
        modes: Vec<(i32, i32, i32)>,
        rates: Vec<i32>,
        trilinear: i32,
        params: ResolutionParams,
        reported_dimensions: Option<(i32, i32)>,
    }

    impl Backend {
        fn new(outcomes: &[bool]) -> Self {
            Self {
                events: Vec::new(), outcomes: outcomes.iter().copied().collect(),
                modes: vec![(800, 600, 32), (1920, 1080, 32), (1024, 768, 32)],
                rates: vec![60, 75], trilinear: 1,
                reported_dimensions: None,
                params: ResolutionParams { width: 1280, height: 720, bytes_per_pixel: 2, fullscreen: 0, refresh_rate: 85 },
            }
        }

        fn writes(&self, key: &str) -> Vec<&ConfigValue> {
            self.events.iter().filter_map(|event| match event {
                Event::Write(_, actual, value) if actual == key => Some(value),
                _ => None,
            }).collect()
        }

        fn resolutions(&self) -> Vec<ResolutionRequest> {
            self.events.iter().filter_map(|event| match event { Event::Resolution(value) => Some(*value), _ => None }).collect()
        }
    }

    impl VideoApplyBackend for Backend {
        fn fullscreen(&mut self) -> i32 { self.events.push(Event::Fullscreen); 1 }
        fn uses_trilinear(&mut self) -> i32 { self.events.push(Event::ReadTrilinear); self.trilinear }
        fn set_trilinear(&mut self, value: i32) { self.events.push(Event::Trilinear(value)); }
        fn delete_all_textures(&mut self, settings: &VideoSettings) { self.events.push(Event::Textures(effective_texture(settings))); }
        fn modes(&mut self) -> Vec<(i32, i32, i32)> { self.events.push(Event::Modes); self.modes.clone() }
        fn refresh_rates(&mut self, mode: (i32, i32, i32)) -> Vec<i32> { self.events.push(Event::Rates(mode)); self.rates.clone() }
        fn set_resolution(&mut self, request: ResolutionRequest, settings: &mut VideoSettings) -> bool {
            self.events.push(Event::Resolution(request));
            let success = self.outcomes.pop_front().expect("unexpected renderer call");
            if let Some((width, height)) = self.reported_dimensions {
                settings.mode.0 = width;
                settings.mode.1 = height;
            } else if success {
                settings.mode.0 = request.width;
                settings.mode.1 = request.height;
            }
            success
        }
        fn resolution_params(&mut self) -> ResolutionParams { self.events.push(Event::Params); self.params }
        fn update_gamma(&mut self, value: f32) { self.events.push(Event::Gamma(value)); }
        fn set_option(&mut self, section: &str, key: &str, value: ConfigValue) { self.events.push(Event::Write(section.into(), key.into(), value)); }
        fn read_float(&mut self, file: &str, section: &str, key: &str) -> Option<f32> {
            self.events.push(Event::Read(file.into(), section.into(), key.into()));
            None
        }
    }

    fn setup() -> (VideoPage, VideoApplyState) {
        let mut page = VideoPage::new(|id| Some(id.to_string()));
        for (combo, value) in page.combos.iter_mut().zip([0, 0, 1, 1, 0, 1, 2, 2, 2, 2, 2, 1, 2]) { combo.selected = value; }
        for (check, value) in page.checks.iter_mut().zip([false, true, true, true]) { check.checked = value; }
        let state = VideoApplyState {
            settings: VideoSettings {
                mode: (800, 600, 32), refresh_rate: 60, client_brightness: Some(1.0),
                pawn_clipping: 2.0, terrain_clipping: 6.0, render_deco: 1,
                post_process_type: 2, projector: 1, texture_detail: 1,
                reduce_pawn_lod: 0, skip_anim: 1, water_effect: 1, water_effect_type: 2,
                trilinear: 1, actor_limit: 4, keep_min_frame_rate: 0, antialiasing: 2,
            },
            actor_clipping: 5.0, static_mesh_clipping: 3.0, static_mesh_lod_clipping: 5.0,
            fog: (1.5, 5.5), actor_count: 300, screenshot_quality: 1,
        };
        (page, state)
    }

    #[test]
    fn video_apply_100c5da0_preserves_write_order_even_without_changes() {
        let (mut page, mut state) = setup();
        let before = state.clone();
        let mut backend = Backend::new(&[]);
        page.apply(&mut state, &mut backend);
        assert_eq!(state, before);
        assert!(backend.resolutions().is_empty());
        let keys: Vec<_> = backend.events.iter().filter_map(|event| match event {
            Event::Write(section, key, _) => Some(format!("{section}.{key}")), _ => None,
        }).collect();
        assert_eq!(keys, [
            "Video.IsKeepMinFrameRate", "Video.UseTrilinear", "Video.TextureDetail", "Video.ModelDetail", "Video.SkipAnim",
            "Video.GamePlayViewportX", "Video.GamePlayViewportY", "Video.ColorBits", "Video.RefreshRate",
            "ClippingRange.Pawn", "Video.PostProc", "ClippingRange.Terrain", "ClippingRange.Actor",
            "ClippingRange.StaticMesh", "ClippingRange.StaticMeshLod", "Video.RenderDeco", "Video.PawnShadow",
            "L2WaterEffect.IsUseEffect", "L2WaterEffect.EffectType", "Video.IsKeepMinFrameRate",
            "Video.RenderActorLimited", "Game.ScreenShotQuality",
        ]);
        assert_eq!(backend.events[0], Event::Fullscreen);
        assert_eq!(backend.events[2], Event::ReadTrilinear);
        assert_eq!(backend.writes("PostProc"), [&ConfigValue::Integer(2)]);
        assert_eq!(backend.writes("ModelDetail"), [&ConfigValue::Text("0".into())]);
        assert_eq!(backend.writes("Pawn"), [&ConfigValue::Decimal(2.0)]);
        assert!(!backend.events.iter().any(|event| matches!(event, Event::Textures(_) | Event::Gamma(_) | Event::Trilinear(_) | Event::Params)));
    }

    #[test]
    fn video_apply_100c5da0_resolution_success_fallback_and_double_failure() {
        for outcomes in [&[true][..], &[false, true], &[false, false]] {
            let (mut page, mut state) = setup();
            page.combos[0].selected = 1;
            page.combos[1].selected = 1;
            let mut backend = Backend::new(outcomes);
            page.apply(&mut state, &mut backend);
            let calls = backend.resolutions();
            assert_eq!(calls[0], ResolutionRequest { width: 1920, height: 1080, bytes_per_pixel: 4, refresh_rate: Some(75), fullscreen: 1, viewport_argument: 1 });
            if outcomes.len() == 2 {
                assert_eq!(calls[1], ResolutionRequest { width: 1024, height: 768, bytes_per_pixel: 4, refresh_rate: Some(60), fullscreen: 1, viewport_argument: 1 });
            }
            let (mode, hz) = if outcomes[0] { ((1920, 1080, 32), 75) } else if outcomes[1] { ((1024, 768, 32), 60) } else { ((800, 600, 32), 60) };
            assert_eq!((state.settings.mode, state.settings.refresh_rate), (mode, hz));
            assert_eq!(backend.writes("GamePlayViewportX"), [&integer_text(mode.0)]);
            assert_eq!(backend.writes("RefreshRate"), [&integer_text(hz)]);
            let fallback_success = !outcomes[0] && outcomes[1];
            assert_eq!(backend.events.iter().filter(|event| **event == Event::Modes).count(), if fallback_success { 2 } else { 1 });
            assert_eq!((page.combos[0].selected, page.combos[1].selected), if fallback_success { (2, 0) } else { (1, 1) });
            assert!(backend.outcomes.is_empty());
        }
    }

    #[test]
    fn video_apply_100c5da0_invalid_modes_and_hz_skip_resolution_writes() {
        for (mode, rate) in [(-1, 0), (3, 0), (100, 0), (0, -1), (0, 2), (0, 100)] {
            let (mut page, mut state) = setup();
            page.combos[0].selected = mode;
            page.combos[1].selected = rate;
            let mut backend = Backend::new(&[]);
            page.apply(&mut state, &mut backend);
            assert!(backend.resolutions().is_empty());
            for key in ["GamePlayViewportX", "GamePlayViewportY", "ColorBits", "RefreshRate"] { assert!(backend.writes(key).is_empty()); }
            assert_eq!(backend.events.iter().any(|event| matches!(event, Event::Rates(_))), mode == 0);
            assert_eq!(backend.writes("ScreenShotQuality"), [&ConfigValue::Integer(1)]);
        }
    }

    #[test]
    fn video_apply_100c5da0_keeps_actual_backend_dimensions_even_after_failure() {
        for outcomes in [&[true][..], &[false, false]] {
            let (mut page, mut state) = setup();
            page.combos[0].selected = 1;
            let mut backend = Backend::new(outcomes);
            backend.reported_dimensions = Some((1111, 777));
            page.apply(&mut state, &mut backend);
            assert_eq!(state.settings.mode, (1111, 777, 32));
            assert_eq!(backend.writes("GamePlayViewportX"), [&integer_text(1111)]);
            assert_eq!(backend.writes("GamePlayViewportY"), [&integer_text(777)]);
        }
    }

    #[test]
    fn video_apply_100c5da0_applies_all_checks_postprocess_and_water_even_disabled() {
        for mask in 0..16 {
            for post in 0..6 {
                for water in 0..3 {
                    let (mut page, mut state) = setup();
                    for (index, check) in page.checks.iter_mut().enumerate() {
                        check.checked = mask & (1 << index) != 0;
                        check.enabled = false;
                    }
                    page.combos[12].selected = post;
                    page.combos[12].enabled = false;
                    page.combos[9].selected = water;
                    page.combos[9].enabled = false;
                    let mut backend = Backend::new(&[]);
                    page.apply(&mut state, &mut backend);
                    assert_eq!(state.settings.keep_min_frame_rate, i32::from(mask & 1 != 0));
                    assert_eq!(state.settings.projector, i32::from(mask & 2 != 0));
                    assert_eq!(state.settings.render_deco, i32::from(mask & 4 != 0));
                    assert_eq!(state.settings.trilinear, i32::from(mask & 8 != 0));
                    assert_eq!(state.settings.post_process_type, post);
                    assert_eq!((state.settings.water_effect, state.settings.water_effect_type), (i32::from(water != 0), water));
                    assert_eq!(backend.writes("RenderDeco"), [&boolean(mask & 4 != 0)]);
                    assert_eq!(backend.writes("PawnShadow"), [&boolean(mask & 2 != 0)]);
                    assert_eq!(backend.writes("IsUseEffect"), [&boolean(water != 0)]);
                    assert_eq!(state.fog, if mask & 1 != 0 { (0.5, 3.5) } else { (1.5, 5.5) });
                }
            }
        }
    }

    #[test]
    fn video_apply_100c5da0_texture_effective_quality_and_backend_trilinear() {
        for old_keep in [0, 1] {
            for new_keep in [false, true] {
                for old_detail in [0, 1, 2] {
                    for selected in [-1, 0, 1, 2, 3] {
                        let (mut page, mut state) = setup();
                        state.settings.keep_min_frame_rate = old_keep;
                        state.settings.texture_detail = old_detail;
                        page.checks[0].checked = new_keep;
                        page.combos[3].selected = selected;
                        let mut backend = Backend::new(&[]);
                        backend.trilinear = 0;
                        page.apply(&mut state, &mut backend);
                        let detail = match selected { 0 => 0, 1 => 1, _ => 2 };
                        let changed = (if old_keep != 0 { 2 } else { old_detail }) != (if new_keep { 2 } else { detail });
                        assert_eq!(backend.events.iter().any(|event| matches!(event, Event::Textures(_))), changed);
                        assert_eq!(state.settings.texture_detail, detail);
                        assert!(backend.events.contains(&Event::Trilinear(1)));
                        assert_eq!(backend.writes("IsKeepMinFrameRate").len(), 2);
                    }
                }
            }
        }
    }

    #[test]
    fn video_apply_100c5da0_raw_indices_and_antialiasing_reset_without_rollback() {
        let (mut page, mut state) = setup();
        for index in [2, 4, 5, 6, 7, 8, 9, 10, 11, 12] { page.combos[index].selected = -1; }
        let mut backend = Backend::new(&[false]);
        page.apply(&mut state, &mut backend);
        assert_eq!(state.settings.post_process_type, 2);
        assert_eq!(backend.writes("PostProc"), [&ConfigValue::Integer(-1)]);
        assert_eq!((state.settings.reduce_pawn_lod, state.settings.skip_anim), (1, 1));
        assert_eq!((state.settings.water_effect, state.settings.water_effect_type), (1, -1));
        assert_eq!((state.settings.actor_limit, state.actor_count), (4, 300));
        assert_eq!((state.settings.pawn_clipping, state.settings.terrain_clipping), (2.0, 6.0));
        assert_eq!((state.actor_clipping, state.static_mesh_clipping, state.static_mesh_lod_clipping), (5.0, 3.0, 5.0));
        assert_eq!(state.settings.client_brightness, Some(1.0));
        assert_eq!(state.settings.antialiasing, -1);
        assert_eq!(state.screenshot_quality, -1);
        assert_eq!(backend.resolutions(), [ResolutionRequest { width: 1280, height: 720, bytes_per_pixel: 2, refresh_rate: None, fullscreen: 0, viewport_argument: 1 }]);
        assert_eq!(backend.writes("AntiAliasing"), [&ConfigValue::Integer(-1)]);
        assert_eq!(backend.writes("ScreenShotQuality"), [&ConfigValue::Integer(-1)]);
    }

    #[test]
    fn video_apply_100c5da0_all_distance_brightness_and_population_choices() {
        for index in 0..6 {
            let (mut page, mut state) = setup();
            for control in [2, 6, 7, 8] { page.combos[control].selected = index; }
            let mut backend = Backend::new(&[]);
            page.apply(&mut state, &mut backend);
            let i = index as usize;
            assert_eq!(state.settings.client_brightness, Some([1.2, 1.0, 0.8, 0.6, 0.4, 1.0][i]));
            assert_eq!(state.settings.pawn_clipping, [3.0, 2.5, 2.0, 1.5, 1.0, 2.0][i]);
            assert_eq!(state.settings.terrain_clipping, [8.0, 7.0, 6.0, 5.0, 4.0, 6.0][i]);
            assert_eq!(state.static_mesh_clipping, [4.0, 3.5, 3.0, 2.5, 2.0, 3.0][i]);
            assert_eq!(state.actor_clipping, [6.0, 5.5, 5.0, 4.5, 4.0, 5.0][i]);
            assert_eq!(state.static_mesh_lod_clipping, state.actor_clipping);
            assert_eq!(state.settings.actor_limit, 6 - index);
            assert_eq!(state.actor_count, [10000, 400, 300, 200, 100, 50][i]);
            assert_eq!(backend.writes("Gamma").len(), usize::from(index != 1 && index != 5));
        }
        for previous in [None, Some(f32::NAN)] {
            let (mut page, mut state) = setup();
            state.settings.client_brightness = previous;
            page.combos[2].selected = -1;
            let mut backend = Backend::new(&[]);
            page.apply(&mut state, &mut backend);
            assert_eq!(backend.writes("Gamma").len(), usize::from(previous.is_some()));
            assert_eq!(state.settings.client_brightness.is_some(), previous.is_some());
            if let Some(value) = state.settings.client_brightness { assert!(value.is_nan()); }
        }
    }
}
