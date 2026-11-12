use killer_render::display_modes::{DisplayMode, refresh_rates, resolutions};
use killer_render::renderer::Renderer;
use killer_render::video::{GammaSettings, TextureFiltering};
use killer_ui::option_video::{VideoCapabilities, VideoChoices, VideoPage, VideoSettings};
use killer_ui::option_video_apply::{ResolutionParams, ResolutionRequest, VideoApplyState};
use winit::dpi::PhysicalSize;
use winit::window::{Fullscreen, Window};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolutionProgress {
    Waiting,
    Confirmed(ResolutionParams),
    Rejected,
}

pub struct VideoDisplay {
    modes: Vec<winit::monitor::VideoModeHandle>,
    pending: Option<ResolutionRequest>,
    exiting_fullscreen: bool,
    pub params: ResolutionParams,
}

impl VideoDisplay {
    pub fn capture(window: &Window, surface: &wgpu::SurfaceConfiguration) -> Option<Self> {
        if !matches!(
            surface.format,
            wgpu::TextureFormat::Bgra8Unorm
                | wgpu::TextureFormat::Bgra8UnormSrgb
                | wgpu::TextureFormat::Rgba8Unorm
                | wgpu::TextureFormat::Rgba8UnormSrgb
        ) {
            return None;
        }
        let monitor = window.current_monitor().or_else(|| window.primary_monitor())?;
        let refresh = match window.fullscreen() {
            Some(Fullscreen::Exclusive(mode)) => Some(mode.refresh_rate_millihertz()),
            _ => monitor.refresh_rate_millihertz(),
        }?;
        let size = window.inner_size();
        Some(Self {
            modes: monitor.video_modes().collect(),
            pending: None,
            exiting_fullscreen: false,
            params: ResolutionParams {
                bytes_per_pixel: 4,
                width: size.width as i32,
                height: size.height as i32,
                refresh_rate: (refresh / 1000) as i32,
                fullscreen: i32::from(window.fullscreen().is_some()),
            },
        })
    }

    fn enumerated(&self) -> Vec<DisplayMode> {
        self.modes
            .iter()
            .map(|mode| DisplayMode {
                width: mode.size().width,
                height: mode.size().height,
                bits: i32::from(mode.bit_depth()),
                refresh: (mode.refresh_rate_millihertz() / 1000) as i32,
            })
            .collect()
    }

    pub fn modes(&self) -> Vec<(i32, i32, i32)> {
        resolutions(&self.enumerated(), 100)
    }

    pub fn rates(&self, mode: (i32, i32, i32)) -> Vec<i32> {
        refresh_rates(&self.enumerated(), mode, 100)
    }

    pub fn request(&mut self, window: &Window, request: ResolutionRequest) -> ResolutionProgress {
        log::info!("Settings: SetRes {}x{} fullscreen {} refresh {:?}; janela {:?}", request.width, request.height, request.fullscreen, request.refresh_rate, window.inner_size());
        if self.pending.is_some() {
            return ResolutionProgress::Waiting;
        }
        if request.width <= 0 || request.height <= 0 || request.bytes_per_pixel != 4 {
            return ResolutionProgress::Rejected;
        }
        let size = PhysicalSize::new(request.width as u32, request.height as u32);
        let current_fullscreen = i32::from(window.fullscreen().is_some());
        let current_rate = match window.fullscreen() {
            Some(Fullscreen::Exclusive(mode)) => Some((mode.refresh_rate_millihertz() / 1000) as i32),
            _ => window
                .current_monitor()
                .and_then(|monitor| monitor.refresh_rate_millihertz())
                .map(|rate| (rate / 1000) as i32),
        };
        if window.inner_size() == size
            && current_fullscreen == i32::from(request.fullscreen != 0)
            && (request.fullscreen == 0 || request.refresh_rate.is_none_or(|rate| Some(rate) == current_rate))
        {
            self.pending = Some(request);
            return self.confirm_size(window, size);
        }

        if request.fullscreen != 0 {
            let rate = request.refresh_rate.unwrap_or(self.params.refresh_rate);
            let Some(mode) = self
                .modes
                .iter()
                .find(|mode| {
                    mode.size() == size
                        && i32::from(mode.bit_depth()) == request.bytes_per_pixel * 8
                        && (mode.refresh_rate_millihertz() / 1000) as i32 == rate
                })
                .cloned()
            else {
                return ResolutionProgress::Rejected;
            };
            self.pending = Some(request);
            window.set_fullscreen(Some(Fullscreen::Exclusive(mode)));
            ResolutionProgress::Waiting
        } else {
            self.pending = Some(request);
            if window.fullscreen().is_some() {
                self.exiting_fullscreen = true;
                window.set_fullscreen(None);
                return ResolutionProgress::Waiting;
            }
            match window.request_inner_size(size) {
                Some(actual) => self.confirm_size(window, actual),
                None => ResolutionProgress::Waiting,
            }
        }
    }

    pub fn resized(&mut self, window: &Window, actual: PhysicalSize<u32>) -> ResolutionProgress {
        log::info!("Settings: Resized {actual:?} com pedido {:?}", self.pending.map(|r| (r.width, r.height)));
        if actual.width == 0 || actual.height == 0 {
            return ResolutionProgress::Waiting;
        }
        let Some(request) = self.pending else {
            return ResolutionProgress::Rejected;
        };
        if request.fullscreen == 0 && window.fullscreen().is_some() {
            return ResolutionProgress::Waiting;
        }
        if self.exiting_fullscreen {
            self.exiting_fullscreen = false;
            return match window.request_inner_size(PhysicalSize::new(request.width as u32, request.height as u32)) {
                Some(size) => self.confirm_size(window, size),
                None => ResolutionProgress::Waiting,
            };
        }
        self.confirm_size(window, actual)
    }

    fn confirm_size(&mut self, window: &Window, actual: PhysicalSize<u32>) -> ResolutionProgress {
        let Some(request) = self.pending.take() else {
            return ResolutionProgress::Rejected;
        };
        self.params.width = actual.width as i32;
        self.params.height = actual.height as i32;
        self.params.fullscreen = i32::from(window.fullscreen().is_some());
        if self.params.width != request.width
            || self.params.height != request.height
            || self.params.fullscreen != i32::from(request.fullscreen != 0)
        {
            return ResolutionProgress::Rejected;
        }
        let rate = match window.fullscreen() {
            Some(Fullscreen::Exclusive(mode)) => Some(mode.refresh_rate_millihertz()),
            _ => window.current_monitor().and_then(|monitor| monitor.refresh_rate_millihertz()),
        };
        if let Some(rate) = rate {
            self.params.refresh_rate = (rate / 1000) as i32;
        }
        if request.fullscreen != 0 && request.refresh_rate.is_some_and(|rate| rate != self.params.refresh_rate) {
            return ResolutionProgress::Rejected;
        }
        ResolutionProgress::Confirmed(self.params)
    }

    pub fn is_pending(&self) -> bool {
        self.pending.is_some()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RendererVideoCapabilities {
    pub water: i32,
    pub shaders: (i32, i32),
    pub hdr: bool,
}

impl RendererVideoCapabilities {
    pub fn query(renderer: &Renderer) -> Self {
        let hdr = killer_render::postprocess::supported(renderer.device());
        Self { water: 0, shaders: if hdr { (20, 20) } else { (0, 0) }, hdr }
    }
}

pub trait DisplayBackend {
    fn params(&self) -> ResolutionParams;
    fn modes(&self) -> Vec<(i32,i32,i32)>;
    fn rates(&self,mode: (i32,i32,i32)) -> Vec<i32>;
}

impl DisplayBackend for VideoDisplay {
    fn params(&self) -> ResolutionParams {self.params}
    fn modes(&self) -> Vec<(i32,i32,i32)> {self.modes()}
    fn rates(&self,mode: (i32,i32,i32)) -> Vec<i32> {self.rates(mode)}
}

pub struct VideoOptionRuntime<D=VideoDisplay> {
    pub capabilities: RendererVideoCapabilities,
    pub state: VideoApplyState,
    pub gamma: GammaSettings,
    pub display: D,
}

impl<D: DisplayBackend> VideoOptionRuntime<D> {
    pub fn from_ini(
        option: &killer_ui::ini::IniFile,
        l2: &killer_ui::ini::IniFile,
        options: &killer_assets::options::ClientOptions,
        display: D,
        capabilities: RendererVideoCapabilities,
    ) -> Result<Self, String> {
        let float = |key| {
            l2.get("WinDrv.WindowsClient", key)
                .and_then(|value| value.parse::<f32>().ok())
                .ok_or_else(|| format!("WinDrv.WindowsClient.{key} ausente ou inválido"))
        };
        let gamma = GammaSettings {
            gamma: option.get("Video", "Gamma").and_then(|v| v.parse().ok()).map_or_else(|| float("Gamma"), Ok)?,
            brightness: float("Brightness")?,
            contrast: float("Contrast")?,
        };
        let integer = |key, default| option.int("Video", key).unwrap_or(default);
        let boolean = |key, default| -> Result<i32, String> {
            let units = option.get("Video", key).map(|value| value.encode_utf16().collect::<Vec<_>>());
            if units.as_ref().is_some_and(|units| units.iter().take(79).any(|unit| *unit > 255)) {
                return Err(format!("Video.{key}: comparação/classificação de locale nativo não disponível"));
            }
            Ok(i32::from(
                killer_ui::ini_value::config_bool(
                    units.as_deref(),
                    |units| {
                        units
                            .iter()
                            .copied()
                            .map(|unit| if (65..=90).contains(&unit) { unit + 32 } else { unit })
                            .eq("true".encode_utf16())
                    },
                    |unit| matches!(unit, 9..=13 | 32 | 160),
                )
                .unwrap_or(default),
            ))
        };
        let actor_limit = integer("RenderActorLimited", 6);
        let params = display.params();
        let fog = options.fog_range();
        Ok(Self {
            capabilities,
            display,
            gamma,
            state: VideoApplyState {
                settings: VideoSettings {
                    mode: (params.width, params.height, params.bytes_per_pixel * 8),
                    refresh_rate: params.refresh_rate,
                    client_brightness: Some(gamma.gamma),
                    pawn_clipping: options.clipping.pawn,
                    terrain_clipping: options.clipping.terrain,
                    render_deco: i32::from(options.render_deco),
                    post_process_type: i32::from(options.post_proc),
                    projector: i32::from(options.pawn_shadow),
                    texture_detail: integer("TextureDetail", 0),
                    reduce_pawn_lod: boolean("ModelDetail", false)?,
                    skip_anim: boolean("SkipAnim", false)?,
                    water_effect: i32::from(options.water_effect),
                    water_effect_type: i32::from(options.water_effect_type),
                    trilinear: boolean("UseTrilinear", true)?,
                    actor_limit,
                    keep_min_frame_rate: i32::from(options.keep_min_frame_rate),
                    antialiasing: integer("AntiAliasing", 0),
                },
                actor_clipping: options.clipping.actor,
                static_mesh_clipping: options.clipping.static_mesh,
                static_mesh_lod_clipping: options.clipping.static_mesh_lod,
                fog: (fog.0 / 2048.0, fog.1 / 2048.0),
                actor_count: match actor_limit {
                    ..=1 => 50,
                    2 => 100,
                    3 => 200,
                    4 => 300,
                    5 => 400,
                    _ => 10000,
                },
                screenshot_quality: 0,
            },
        })
    }

    pub fn show(&mut self, page: &mut VideoPage, antialiasing: i32, lookup: impl Fn(u32) -> Option<String>) {
        let modes = self.display.modes();
        let params = self.display.params();
        self.state.settings.mode = (params.width, params.height, params.bytes_per_pixel * 8);
        self.state.settings.refresh_rate = params.refresh_rate;
        let rates = self.display.rates(self.state.settings.mode);
        page.clear();
        page.append_choices(
            &VideoChoices {
                modes: &modes,
                refresh_rates: &rates,
                capability_10: self.capabilities.hdr,
                capability_cc: antialiasing,
                screenshot_quality: self.state.screenshot_quality,
            },
            lookup,
        );
        page.read_settings(
            &mut self.state.settings,
            &VideoCapabilities {
                modes: &modes,
                refresh_rates: &rates,
                water: self.capabilities.water,
                shaders: self.capabilities.shaders,
                antialiasing,
            },
        );
    }

    pub fn resolution_changed(&self, page: &mut VideoPage, selection: i32) {
        page.resolution_changed(selection, &self.display.modes(), self.display.params().refresh_rate, |mode| {
            self.display.rates(mode)
        });
    }

    pub fn apply_render_state(&mut self, renderer: &mut Renderer, options: &mut killer_assets::options::ClientOptions) {
        let settings = &self.state.settings;
        options.clipping.pawn = settings.pawn_clipping;
        options.clipping.terrain = settings.terrain_clipping;
        options.clipping.actor = self.state.actor_clipping;
        options.clipping.static_mesh = self.state.static_mesh_clipping;
        options.clipping.static_mesh_lod = self.state.static_mesh_lod_clipping;
        options.keep_min_frame_rate = settings.keep_min_frame_rate != 0;
        options.render_deco = settings.render_deco != 0;
        options.pawn_shadow = settings.projector != 0;
        renderer.set_pawn_shadow(options.pawn_shadow, options.keep_min_frame_rate);
        let mut effective = options.clipping;
        if options.keep_min_frame_rate {
            effective.pawn = f32::from_bits(0x3f333333);
            effective.terrain = 4.0;
            effective.static_mesh = 2.0;
            effective.static_mesh_lod = 4.0;
        }
        renderer.set_video_clipping(
            &effective,
            killer_assets::scene::GlobalFog {
                start: self.state.fog.0 * 2048.0,
                end: self.state.fog.1 * 2048.0,
                speed: options.fog.speed,
                level: killer_assets::options::fog_index(settings.terrain_clipping, options.keep_min_frame_rate) - 1,
            },
        );
        if let Some(gamma) = settings.client_brightness {
            self.gamma.gamma = gamma;
            renderer.set_gamma(self.gamma);
        }
        renderer.set_texture_filtering(TextureFiltering {
            trilinear: settings.trilinear != 0,
            ..renderer.texture_filtering()
        });
    }

    pub fn settings(&self) -> &VideoSettings {
        &self.state.settings
    }
}

pub trait VideoResourceEffects {
    fn reload_textures(&mut self, renderer: &mut Renderer, settings: &VideoSettings) -> Result<(), String>;
    fn apply_scene(
        &mut self,
        renderer: &mut Renderer,
        previous: &VideoApplyState,
        next: &VideoApplyState,
    ) -> Result<(), String>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum ApplyProgress {
    Waiting,
    Complete,
    Failed(String),
}

pub struct VideoApplyTransaction {
    page: VideoPage,
    initial: VideoApplyState,
    trilinear: i32,
    fullscreen: i32,
    completed: usize,
    resolutions: std::collections::HashMap<usize, (bool, ResolutionParams)>,
    waiting: Option<usize>,
    edits: Vec<crate::ui_option_config::ConfigEdit>,
}

pub struct PendingVideoApply {
    pub transaction: VideoApplyTransaction,
    pub page: VideoPage,
    pub container: Option<bool>,
}

#[cfg(not(target_arch = "wasm32"))]
pub struct WorldVideoResources<'a> {
    pub client: &'a mut killer_assets::client::Client,
    pub bank: &'a mut killer_assets::textures::TextureBank,
    pub view: &'a mut crate::app::WorldView,
}

#[cfg(not(target_arch = "wasm32"))]
pub struct LobbyVideoResources<'a> {
    pub client: &'a mut killer_assets::client::Client,
    pub bank: &'a mut killer_assets::textures::TextureBank,
}

#[cfg(not(target_arch = "wasm32"))]
impl VideoResourceEffects for LobbyVideoResources<'_> {
    fn reload_textures(&mut self, renderer: &mut Renderer, settings: &VideoSettings) -> Result<(), String> {
        let detail = if settings.keep_min_frame_rate != 0 { 2 } else { settings.texture_detail };
        let mut bank = reload_bank(self.bank);
        let mut source = killer_assets::textures::BankSource { bank: &mut bank, catalog: self.client.catalog() };
        renderer.reload_texture_detail(detail, &mut source);
        Ok(())
    }

    fn apply_scene(&mut self, _renderer: &mut Renderer, previous: &VideoApplyState, next: &VideoApplyState) -> Result<(), String> {
        validate_pawn_options(previous, next)
    }
}

pub fn reload_bank(ui: &killer_assets::textures::TextureBank) -> killer_assets::textures::TextureBank {
    let mut bank = killer_assets::textures::TextureBank::new();
    for (name, data) in ui.textures() { bank.insert_shared(&name, data); }
    for (name, data) in ui.alpha_maps() { bank.insert_shared_alpha_map(&name, data); }
    bank
}

#[cfg(not(target_arch = "wasm32"))]
impl VideoResourceEffects for WorldVideoResources<'_> {
    fn reload_textures(&mut self, renderer: &mut Renderer, settings: &VideoSettings) -> Result<(), String> {
        let detail = if settings.keep_min_frame_rate != 0 { 2 } else { settings.texture_detail };
        let mut bank = reload_bank(self.bank);
        let mut source = killer_assets::textures::BankSource { bank: &mut bank, catalog: self.client.catalog() };
        renderer.reload_texture_detail(detail, &mut source);
        Ok(())
    }

    fn apply_scene(&mut self, renderer: &mut Renderer, previous: &VideoApplyState, next: &VideoApplyState) -> Result<(), String> {
        validate_pawn_options(previous, next)?;
        if previous.settings.render_deco != next.settings.render_deco
            || previous.settings.keep_min_frame_rate != next.settings.keep_min_frame_rate {
            let mut options = self.view.options.clone();
            options.render_deco = next.settings.render_deco != 0;
            options.keep_min_frame_rate = next.settings.keep_min_frame_rate != 0;
            let levels = std::iter::once((String::new(), self.view.neighbours.main_level.clone()))
                .chain(self.view.neighbours.tiles.iter().map(|(name, tile)| (name.clone(), tile.level.clone())))
                .collect::<Vec<_>>();
            for (name, level) in levels {
                let zones = killer_assets::scene::zone_lighting(&level);
                let draws = killer_assets::scene::decoration_draws(self.client, &level, &zones, &options);
                let mut bank = reload_bank(self.bank);
                let mut source = killer_assets::textures::BankSource { bank: &mut bank, catalog: self.client.catalog() };
                renderer.replace_decorations(&name, &draws, &mut source);
            }
        }
        self.view.display.pawn_clipping = if next.settings.keep_min_frame_rate != 0 {
            f32::from_bits(0x3f333333)
        } else { next.settings.pawn_clipping };
        self.view.display.render_limit = next.actor_count.max(0) as usize;
        Ok(())
    }
}

pub fn validate_pawn_options(previous: &VideoApplyState, next: &VideoApplyState) -> Result<(), String> {
    if previous.settings.reduce_pawn_lod != next.settings.reduce_pawn_lod {
        return Err("ModelDetail: integração do LOD de personagens ainda não portada".into());
    }
    if previous.settings.skip_anim != next.settings.skip_anim {
        return Err("SkipAnim: integração da seleção de animações ainda não portada".into());
    }
    Ok(())
}

impl VideoApplyTransaction {
    pub fn new(page: &VideoPage, runtime: &VideoOptionRuntime, renderer: &Renderer) -> Self {
        Self {
            page: page.clone(),
            initial: runtime.state.clone(),
            trilinear: i32::from(renderer.texture_filtering().trilinear),
            fullscreen: runtime.display.params.fullscreen,
            completed: 0,
            resolutions: Default::default(),
            waiting: None,
            edits: Vec::new(),
        }
    }

    pub fn take_edits(&mut self) -> Vec<crate::ui_option_config::ConfigEdit> {
        std::mem::take(&mut self.edits)
    }

    pub fn confirm_resolution(&mut self, result: ResolutionProgress, actual: ResolutionParams) {
        if result == ResolutionProgress::Waiting {
            return;
        }
        if let Some(index) = self.waiting.take() {
            self.resolutions.insert(index, (matches!(result, ResolutionProgress::Confirmed(_)), actual));
            self.completed = index + 1;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn advance(
        &mut self,
        runtime: &mut VideoOptionRuntime,
        page: &mut VideoPage,
        window: &Window,
        renderer: &mut Renderer,
        adapter: &wgpu::Adapter,
        env: &killer_ui::ini::IniFile,
        resources: &mut impl VideoResourceEffects,
    ) -> ApplyProgress {
        if self.waiting.is_some() {
            return ApplyProgress::Waiting;
        }
        let mut next = self.initial.clone();
        let mut next_page = self.page.clone();
        let mut backend = ApplyBackend {
            transaction: self,
            runtime,
            window,
            renderer,
            adapter,
            env,
            resources,
            cursor: 0,
            stopped: false,
            error: None,
        };
        next_page.apply(&mut next, &mut backend);
        if let Some(error) = backend.error {
            return ApplyProgress::Failed(error);
        }
        if backend.stopped {
            return ApplyProgress::Waiting;
        }
        if let Err(error) = backend.resources.apply_scene(backend.renderer, &backend.transaction.initial, &next) {
            return ApplyProgress::Failed(error);
        }
        let postprocess = killer_render::postprocess::PostProcessSettings::read(
            next.settings.post_process_type,
            next.settings.keep_min_frame_rate != 0,
            backend.env,
        );
        if let Err(error) = backend.renderer.set_postprocess(postprocess) {
            return ApplyProgress::Failed(error.into());
        }
        backend.runtime.state = next;
        *page = next_page;
        ApplyProgress::Complete
    }
}

struct ApplyBackend<'a, R> {
    transaction: &'a mut VideoApplyTransaction,
    runtime: &'a mut VideoOptionRuntime,
    window: &'a Window,
    renderer: &'a mut Renderer,
    adapter: &'a wgpu::Adapter,
    env: &'a killer_ui::ini::IniFile,
    resources: &'a mut R,
    cursor: usize,
    stopped: bool,
    error: Option<String>,
}

impl<R> ApplyBackend<'_, R> {
    fn operation(&mut self) -> bool {
        let index = self.cursor;
        self.cursor += 1;
        if self.stopped || index < self.transaction.completed {
            return false;
        }
        self.transaction.completed = self.cursor;
        true
    }

    fn fail(&mut self, error: String) {
        self.error = Some(error);
        self.stopped = true;
    }
}

impl<R: VideoResourceEffects> killer_ui::option_video_apply::VideoApplyBackend for ApplyBackend<'_, R> {
    fn fullscreen(&mut self) -> i32 {
        self.transaction.fullscreen
    }
    fn uses_trilinear(&mut self) -> i32 {
        self.transaction.trilinear
    }
    fn set_trilinear(&mut self, value: i32) {
        if self.operation() {
            self.renderer
                .set_texture_filtering(TextureFiltering { trilinear: value != 0, ..self.renderer.texture_filtering() });
        }
    }
    fn delete_all_textures(&mut self, settings: &VideoSettings) {
        if self.operation()
            && let Err(error) = self.resources.reload_textures(self.renderer, settings)
        {
            self.fail(error);
        }
    }
    fn modes(&mut self) -> Vec<(i32, i32, i32)> {
        self.runtime.display.modes()
    }
    fn refresh_rates(&mut self, mode: (i32, i32, i32)) -> Vec<i32> {
        self.runtime.display.rates(mode)
    }
    fn set_resolution(&mut self, request: ResolutionRequest, settings: &mut VideoSettings) -> bool {
        let index = self.cursor;
        self.cursor += 1;
        if self.stopped {
            return false;
        }
        if let Some((success, actual)) = self.transaction.resolutions.get(&index) {
            settings.mode.0 = actual.width;
            settings.mode.1 = actual.height;
            return *success;
        }
        if request.refresh_rate.is_none() {
            let postprocess = killer_render::postprocess::PostProcessSettings::read(
                settings.post_process_type,
                settings.keep_min_frame_rate != 0,
                self.env,
            );
            if !postprocess.active()
                && let Err(error) = self.renderer.set_postprocess(postprocess)
            {
                self.fail(error.into());
                return false;
            }
        }
        if request.refresh_rate.is_none()
            && let Err(error) = self.renderer.set_antialiasing(self.adapter, settings.antialiasing)
        {
            self.fail(error.into());
            return false;
        }
        match self.runtime.display.request(self.window, request) {
            ResolutionProgress::Waiting => {
                self.transaction.waiting = Some(index);
                self.stopped = true;
                false
            }
            result => {
                let actual = self.runtime.display.params;
                let success = matches!(result, ResolutionProgress::Confirmed(_));
                self.transaction.resolutions.insert(index, (success, actual));
                self.transaction.completed = self.cursor;
                settings.mode.0 = actual.width;
                settings.mode.1 = actual.height;
                success
            }
        }
    }
    fn resolution_params(&mut self) -> ResolutionParams {
        self.runtime.display.params
    }
    fn update_gamma(&mut self, value: f32) {
        if self.operation() {
            self.runtime.gamma.gamma = value;
            self.renderer.set_gamma(self.runtime.gamma);
        }
    }
    fn set_option(&mut self, section: &str, key: &str, value: killer_ui::option_video_apply::ConfigValue) {
        if self.operation() {
            let value = match value {
                killer_ui::option_video_apply::ConfigValue::Text(value) => value,
                killer_ui::option_video_apply::ConfigValue::Integer(value) => value.to_string(),
                killer_ui::option_video_apply::ConfigValue::Decimal(value) => format!("{value:.6}"),
            };
            self.transaction.edits.push(crate::ui_option_config::ConfigEdit::new(section, key, value));
        }
    }
    fn read_float(&mut self, file: &str, section: &str, key: &str) -> Option<f32> {
        if !file.eq_ignore_ascii_case("env.int") {
            return None;
        }
        self.env.get(section, key)?.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reload_source_shares_ui_assets_without_retaining_world_assets_in_ui() {
        use killer_assets::textures::{TextureBank, TextureData, TextureFormatKind};
        use std::sync::Arc;
        let ui_texture = Arc::new(TextureData {
            width: 1, height: 1, format: TextureFormatKind::Rgba8, mips: vec![vec![255; 4]],
        });
        let world_texture = Arc::new(TextureData {
            width: 1, height: 1, format: TextureFormatKind::Rgba8, mips: vec![vec![127; 4]],
        });
        let mut ui = TextureBank::new();
        ui.insert_shared("ui.icon", ui_texture.clone());
        ui.insert_shared_alpha_map("ui.mask", ui_texture.clone());
        {
            let mut temporary = reload_bank(&ui);
            assert!(temporary.textures().any(|(name, data)| name == "ui.icon" && Arc::ptr_eq(&data, &ui_texture)));
            assert!(temporary.alpha_maps().any(|(name, data)| name == "ui.mask" && Arc::ptr_eq(&data, &ui_texture)));
            temporary.insert_shared("world.texture", world_texture.clone());
            assert_eq!(Arc::strong_count(&world_texture), 2);
            assert!(!ui.has_texture("world.texture"));
        }
        assert_eq!(Arc::strong_count(&world_texture), 1);
        assert!(ui.has_texture("ui.icon"));
    }

    #[test]
    #[ignore]
    fn video_apply_without_changes_rewrites_the_same_values_from_the_client_files() {
        use killer_ui::option_video_apply::{ConfigValue, VideoApplyBackend};
        struct Recorder { edits: Vec<(String, String, ConfigValue)>, resolution: Vec<ResolutionRequest>, textures: bool, params: ResolutionParams }
        impl VideoApplyBackend for Recorder {
            fn fullscreen(&mut self) -> i32 { 0 }
            fn uses_trilinear(&mut self) -> i32 { 1 }
            fn set_trilinear(&mut self, _: i32) {}
            fn delete_all_textures(&mut self, _: &VideoSettings) { self.textures = true; }
            fn modes(&mut self) -> Vec<(i32, i32, i32)> { vec![(800, 600, 32), (1024, 768, 32), (1920, 1080, 32)] }
            fn refresh_rates(&mut self, _: (i32, i32, i32)) -> Vec<i32> { vec![60] }
            fn set_resolution(&mut self, request: ResolutionRequest, _: &mut VideoSettings) -> bool { self.resolution.push(request); true }
            fn resolution_params(&mut self) -> ResolutionParams { self.params }
            fn update_gamma(&mut self, _: f32) {}
            fn set_option(&mut self, section: &str, key: &str, value: ConfigValue) { self.edits.push((section.into(), key.into(), value)); }
            fn read_float(&mut self, _: &str, _: &str, _: &str) -> Option<f32> { None }
        }
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = killer_assets::client::Client::open(&root).unwrap();
        let option = killer_ui::ini::parse_ini(&client.read_system_text("option.ini").unwrap());
        let l2 = killer_ui::ini::parse_ini(&client.read_system_text("l2.ini").unwrap());
        let options = killer_assets::options::ClientOptions::load(&mut client).unwrap();
        let mut display = display();
        display.params = ResolutionParams { bytes_per_pixel: 4, width: 1558, height: 933, refresh_rate: 180, fullscreen: 0 };
        let mut runtime = VideoOptionRuntime::from_ini(&option, &l2, &options, display,
            RendererVideoCapabilities { water: 0, shaders: (20, 20), hdr: true }).unwrap();
        runtime.state.screenshot_quality = option.int("Game", "ScreenShotQuality").unwrap_or(0);
        let mut page = VideoPage::new(|id| Some(id.to_string()));
        runtime.show(&mut page, 4, |id| Some(id.to_string()));
        let mut state = runtime.state.clone();
        let mut recorder = Recorder { edits: Vec::new(), resolution: Vec::new(), textures: false, params: runtime.display.params };
        page.apply(&mut state, &mut recorder);
        let text = |value: &ConfigValue| match value {
            ConfigValue::Text(t) => t.to_ascii_lowercase(),
            ConfigValue::Integer(i) => i.to_string(),
            ConfigValue::Decimal(d) => format!("{d}"),
        };
        let mut differences = Vec::new();
        for (section, key, value) in &recorder.edits {
            let before = option.get(section, key).map(|v| v.to_ascii_lowercase());
            let same = match (value, &before) {
                (ConfigValue::Decimal(d), Some(b)) => b.parse::<f32>().is_ok_and(|b| (b - d).abs() < 1e-4),
                (_, Some(b)) => *b == text(value),
                _ => false,
            };
            if !same { differences.push(format!("{section}/{key}: {before:?} -> {}", text(value))); }
        }
        eprintln!("resolução pedida: {:?}; texturas recarregadas: {}", recorder.resolution, recorder.textures);
        assert!(differences.is_empty(), "{differences:#?}");
        assert!(recorder.resolution.is_empty() && !recorder.textures);
    }

    fn display() -> VideoDisplay {
        VideoDisplay {
            modes: Vec::new(),
            pending: None,
            exiting_fullscreen: false,
            params: ResolutionParams { bytes_per_pixel: 4, width: 1280, height: 720, refresh_rate: 60, fullscreen: 0 },
        }
    }

    #[test]
    fn runtime_reads_native_initial_values_and_actual_display() {
        let option = killer_ui::ini::parse_ini(
            "[Video]\nTextureDetail=2\nModelDetail=True\nSkipAnim=1\nRenderActorLimited=3\nGamma=0.6\n",
        );
        let l2 = killer_ui::ini::parse_ini(killer_assets::options::L2_INI_RENDER);
        let runtime = VideoOptionRuntime::from_ini(
            &option,
            &l2,
            &Default::default(),
            display(),
            RendererVideoCapabilities { water: 0, shaders: (20, 20), hdr: true },
        )
        .unwrap();
        assert_eq!(runtime.settings().mode, (1280, 720, 32));
        assert_eq!(runtime.settings().refresh_rate, 60);
        assert_eq!(runtime.gamma, GammaSettings { gamma: 0.6, brightness: 0.8, contrast: 0.7 });
        assert_eq!(
            (runtime.settings().texture_detail, runtime.settings().reduce_pawn_lod, runtime.settings().skip_anim),
            (2, 1, 1)
        );
        assert_eq!(runtime.state.actor_count, 200);
        assert_eq!(runtime.settings().trilinear, 1);
        let missing = killer_ui::ini::parse_ini("");
        assert!(
            VideoOptionRuntime::from_ini(&option, &missing, &Default::default(), display(), runtime.capabilities)
                .is_err()
        );
    }

    #[test]
    fn resolution_wait_does_not_commit_and_duplicate_confirmation_does_not_replace_result() {
        let runtime = VideoOptionRuntime::from_ini(
            &killer_ui::ini::parse_ini(""),
            &killer_ui::ini::parse_ini(killer_assets::options::L2_INI_RENDER),
            &Default::default(), display(),
            RendererVideoCapabilities { water: 0, shaders: (0, 0), hdr: false },
        ).unwrap();
        let mut transaction = VideoApplyTransaction {
            page: VideoPage::new(|_| None), initial: runtime.state.clone(),
            trilinear: 1, fullscreen: 0, completed: 3,
            resolutions: Default::default(), waiting: Some(3), edits: Vec::new(),
        };
        let actual = runtime.display.params;
        transaction.confirm_resolution(ResolutionProgress::Waiting, actual);
        assert_eq!(transaction.waiting, Some(3));
        assert_eq!(transaction.completed, 3);
        assert!(transaction.resolutions.is_empty());
        transaction.confirm_resolution(ResolutionProgress::Confirmed(actual), actual);
        assert_eq!(transaction.waiting, None);
        assert_eq!(transaction.completed, 4);
        assert_eq!(transaction.resolutions.get(&3), Some(&(true, actual)));
        transaction.confirm_resolution(ResolutionProgress::Rejected, ResolutionParams { width: 1, ..actual });
        assert_eq!(transaction.resolutions.get(&3), Some(&(true, actual)));
    }
}
