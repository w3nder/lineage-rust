use crate::ui2d::dialog_command;
use crate::camera::{CameraSettings, FreeCamera, Movement, OrbitCamera, UE_UNITS_PER_TURN};
use crate::characters::{self, CharacterCache, DisplaySettings, Frame, Label, LoadedCharacter};
use crate::gpu::Gpu;
use crate::ground::GroundGrid;
use crate::ignored::{HUD_TOP, IgnoredOpcodes};
use crate::input::{self, ClickTracker, target_action};
use crate::loader::{LoadRequest, LoadResult, Loader};
use crate::lobby::config::ServerConfig;
use crate::lobby::demo::LobbyDemo;
use crate::lobby::{Lobby, LobbyCommand, LobbyPart};
use crate::pace::{self, ValidatePacer};
use crate::profile::FrameProfile;
use crate::tiles::{Job, NeighbourResult, NeighbourSet, load_neighbour, reseam};
use crate::trace::TraceMesh;
use crate::ui::{self, HudInfo, HudMode};
use crate::ui2d::{self, DialogAnswer, GameUi, KeyAction};
use crate::ui_keys::{self, UiCommand};
use egui_wgpu::{Renderer as EguiRenderer, RendererOptions, ScreenDescriptor};
use glam::{Mat4,Vec3};
use killer_assets::cache::SceneCache;
use killer_assets::client::{Client, DatTables};
use killer_assets::scene::actor_lighting::{LightMap, Query, Sample};
use killer_assets::options::ClientOptions;
use killer_assets::options::range_units;
use killer_assets::scene::{DrawKind, Neighbours, Scene, build_scene_with, sky, terrain, terrain_zone};
use killer_assets::textures::{BankSource, TextureBank, write_png};
use killer_assets::timeenv::TimeEnv;
use killer_assets::weather::Weather;
use killer_assets::tiles::{TILE_SIZE, TileId, tile_name};
use killer_render::camera::Camera;
use killer_render::environment::Environment;
use killer_render::headless::RenderError;
use killer_assets::scene::emitters::EmitterAsset;
use killer_render::renderer::{EffectId, EffectPlacement, Renderer, Viewer};
use killer_ui::chat::{ChatEntry, InputKey};
use killer_ui::ini::parse_ini;
use killer_world::chat::CHAT_SYSTEM;

const CONNECTION_LOST_SYSMSG: u32 = 0;
const PROTOCOL_QUIT_SYSMSG: u32 = 88;
use killer_world::{
    ChatLine, ChatLog, NetCommand, NetEvent, NetHandle, SessionEvent, World, WorldEvent, spawn_choosing,
};
use l2_net::game::SystemMessage;
use std::collections::HashMap;
use std::f32::consts::TAU;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use ue2_level::{BspTree, Level};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use winit::window::{Window, WindowId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Login,
    Servers,
    Characters,
    Create,
    World,
}

pub fn next_screen(current: Screen, event: &NetEvent) -> Screen {
    match event {
        NetEvent::Connecting | NetEvent::LoginOk(_) | NetEvent::Notice(_) | NetEvent::CardCheck { .. } => current,
        NetEvent::ServerList(_) => Screen::Servers,
        NetEvent::Session(SessionEvent::Characters(_)) => Screen::Characters,
        NetEvent::Session(SessionEvent::RestartResponse(_)) => Screen::Characters,
        NetEvent::Session(SessionEvent::UserInfo(_)) => Screen::World,
        NetEvent::Session(
            SessionEvent::Selected
            | SessionEvent::World(_)
            | SessionEvent::Time(_)
            | SessionEvent::Unknown { .. }
            | SessionEvent::Malformed { .. }
            | SessionEvent::CharTemplates(_)
            | SessionEvent::CharCreateFailed(_)
            | SessionEvent::CharDeleteFailed(_)
            | SessionEvent::GameGuardQuery(_),
        ) => current,
        NetEvent::Session(SessionEvent::CharCreated | SessionEvent::CharDeleted) => Screen::Characters,
        NetEvent::Session(SessionEvent::LeftWorld) => Screen::Characters,
        NetEvent::Session(SessionEvent::ServerClosed) if current == Screen::World => current,
        NetEvent::Session(SessionEvent::Refused(_) | SessionEvent::ServerClosed) | NetEvent::Error(_) | NetEvent::LoginRefused { .. } | NetEvent::LoginBlocked { .. } | NetEvent::Closed => Screen::Login,
    }
}

enum Action {
    Leave,
}

pub struct WorldView {
    pub moving_trace: crate::trace::MovingTrace,
    pub tile: String,
    pub trace: TraceMesh,
    pub zones: usize,
    pub lighting: LightMap,
    pub camera: OrbitCamera,
    pub display: DisplaySettings,
    pub free: FreeCamera,
    pub use_free: bool,
    pub timeenv: TimeEnv,
    pub weather: Weather,
    pub bsp_tree: BspTree,
    pub ground: GroundGrid,
    pub shakes: killer_world::camera_shake::ShakeSet,
    pub fade: killer_world::screen_fade::ScreenFade,
    pub fade_shown: bool,
    pub draws: usize,
    pub triangles: usize,
    pub characters: usize,
    pub instances: usize,
    pub neighbours: NeighbourSet,
    pub options: ClientOptions,
}

impl WorldView {
    pub fn current(&self) -> Camera {
        if self.use_free { self.free.camera() } else { crate::camera::apply_shake(self.camera.camera(), self.shakes.rotation_offset()) }
    }

    pub fn zone_at(&self, position: Vec3) -> u8 {
        if let Some(zone) = self.neighbours.zone_at(position) {
            return zone;
        }
        let zone = self.bsp_tree.zone_at(position.to_array());
        if usize::from(zone) < self.zones { zone } else { 0 }
    }

    pub fn nearest(&self, from: Vec3, to: Vec3) -> Option<crate::trace::Hit> {
        self.neighbours.nearest(from, to, &self.trace).into_iter().chain(self.moving_trace.nearest(from, to)).min_by(|a,b| a.distance.total_cmp(&b.distance))
    }

    pub fn ground_for(&self, position: Vec3) -> &GroundGrid {
        self.neighbours.ground_at(position, &self.ground)
    }

    pub fn actor_lighting(&self, query: Query) -> Sample {
        let map = self.neighbours.tile_at(query.position).map_or(&self.lighting, |tile| &tile.lighting);
        map.sample(query, 0, 0.0)
    }

    pub fn total_draws(&self) -> (usize, usize) {
        let (draws, triangles) = self.neighbours.draw_totals();
        (self.draws + draws, self.triangles + triangles)
    }

    fn toggle_free(&mut self) {
        if !self.use_free {
            self.free = FreeCamera::from_orbit(&self.camera);
        }
        self.use_free = !self.use_free;
    }
}

struct LoadedTile {
    tile: String,
    scene: Scene,
    trace: TraceMesh,
    weather: Weather,
    bsp_tree: BspTree,
    ground: GroundGrid,
    bank: TextureBank,
    camera: OrbitCamera,
    display: DisplaySettings,
    tables: Option<Result<DatTables, String>>,
    ui: Option<Result<GameUi, String>>,
    elapsed: Duration,
    level: Arc<Level>,
    collision: killer_assets::collision::resources::Resources,
    options: ClientOptions,
    sky: Option<Scene>,
    binds: killer_world::engine_input::KeyBinds,
}

struct EguiLayer {
    ctx: egui::Context,
    state: egui_winit::State,
    renderer: EguiRenderer,
}

impl EguiLayer {
    fn new(window: &Window, device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let ctx = egui::Context::default();
        let state = egui_winit::State::new(ctx.clone(), egui::ViewportId::ROOT, window, None, None, None);
        let renderer = EguiRenderer::new(device, format, RendererOptions::default());
        Self { ctx, state, renderer }
    }
}

struct Stage {
    window: Arc<Window>,
    gpu: Gpu,
    renderer: Renderer,
    egui: EguiLayer,
}

enum PendingPick {
    Move { ctrl: bool, shift: bool },
    Drop(killer_world::Dialog),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Hovered {
    object_id: i32,
    spawned: u64,
    slot: Option<usize>,
}

struct Hover<'a> {
    hovered: Option<Hovered>,
    cursor: Option<(f32,f32)>,
    items: &'a crate::items::Items,
    world: &'a World,
    tables: Option<&'a DatTables>,
}

fn item_label(hover: Hover<'_>, ui: &ui2d::GameUi, camera: &Camera, size: (f32,f32)) -> Option<ui2d::ItemLabel> {
    let hovered = hover.hovered?;
    let slot = hovered.slot?;
    let (x,y) = hover.cursor?;
    if !(0.0..=size.0).contains(&x) || !(0.0..=size.1).contains(&y) { return None; }
    if ui.window_under(x as i32,y as i32).is_some() { return None; }
    let (location,text) = hover.items.label(hover.world,hover.tables?,hovered.object_id,hovered.spawned,slot)?;
    let screen = characters::project(camera,size,location)?;
    Some(ui2d::ItemLabel { screen,text })
}

fn same_target(a: &killer_assets::collision::world::Target, b: &killer_assets::collision::world::Target) -> bool {
    a.object_id == b.object_id && a.spawned == b.spawned && a.position == b.position && a.radius == b.radius && a.height == b.height
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ServerTransfer {
    server: u8,
    slot: i32,
    position: [i32; 3],
}
const MOBILE_UI_HEIGHT: f32 = 720.0;
const TOUCH_RADIUS_POINTS: f32 = 22.0;

pub struct App {
    client_root: PathBuf,
    touches: crate::touch::Touches,
    applied_ui: Option<(u32, u32, u32)>,
    walk: crate::walk_world::WalkPhysics,
    music_zone: Option<killer_assets::collision::world::MusicZone>,
    fade_in_after_load: bool,
    splash: Option<crate::splash::Splash>,
    tile: Option<String>,
    hour: Option<f32>,
    stage: Option<Stage>,
    screen: Screen,
    world_view: Option<WorldView>,
    pending: Option<Receiver<Result<LoadedTile, String>>>,
    neighbour_pending: Option<Receiver<Result<NeighbourResult, String>>>,
    load_error: Option<String>,
    net: Option<NetHandle>,
    lobby: Lobby,
    world: World,
    labels: Vec<Label>,
    chat: ChatLog,
    pending_chat_system: Vec<SystemMessage>,
    validate: ValidatePacer,
    click: ClickTracker,
    cursor: Option<(f32, f32)>,
    game_cursors: Vec<Option<winit::window::CustomCursor>>,
    active_cursor: Option<winit::window::CustomCursor>,
    texture_cursor: Option<(String, Option<winit::window::CustomCursor>)>,
    ui_scale: f32,
    moves: Movement,
    dragging: bool,
    right_pressed_at: Option<Instant>,
    last_cursor: Option<(f32, f32)>,
    last_frame: Instant,
    material_clock: Instant,
    fps: f32,
    screenshot: Option<PathBuf>,
    capture: Option<PathBuf>,
    frames_ready: u32,
    ignored: IgnoredOpcodes,
    tables: Option<Arc<DatTables>>,
    tables_error: Option<String>,
    lobby_c4: Option<crate::lobby_c4::LobbyC4>,
    ping_rx: Option<std::sync::mpsc::Receiver<(u8, i32)>>,
    lobby_shift: bool,
    cache: CharacterCache,
    items: crate::items::Items,
    fences: crate::server_objects::Fences,
    sound_bank: crate::world_sound::SoundBank,
    pick_ticket: u64,
    pending_pick: Option<PendingPick>,
    hover_ticket: u64,
    hover_pending: bool,
    hover_pieces: Vec<(killer_assets::collision::world::Target,Option<usize>)>,
    hovered: Option<Hovered>,
    effects: crate::pawn_effects::Effects,
    abnormal: crate::abnormal_host::AbnormalStates,
    appendix: crate::appendix_effects::AppendixEffects,
    last_login: Option<(ServerConfig, String, String)>,
    last_slot: i32,
    server_transfer: Option<ServerTransfer>,
    disconnect_pending: bool,
    landmark: crate::landmark::LandMark,
    console_mark: crate::ui_olympiad::ConsoleMark,
    landmark_effects: HashMap<crate::landmark::Slot, EffectId>,
    landmark_loose: Vec<(EffectId, f32)>,
    landmark_spans: HashMap<&'static str, f32>,
    loader: Option<Loader>,
    uploader: Option<Client>,
    demo: bool,
    demo_crowd: usize,
    demo_cast: f32,
    demo_cast_index: usize,
    profile: FrameProfile,
    game_ui: Option<GameUi>,
    video_options: Option<crate::ui_video::VideoOptionRuntime>,
    video_env: killer_ui::ini::IniFile,
    video_apply: Option<crate::ui_video::PendingVideoApply>,
    lobby_options: ClientOptions,
    ui_error: Option<String>,
    ui_bank: TextureBank,
    banks_trimmed: Instant,
    fishing_target: Option<killer_render::viewport::ViewportTarget>,
    replay_logo: killer_ui::replay_menu::ReplayLogoWnd,
    fishing_host: crate::fishing_host::FishingHost,
    radar_toggle: bool,
    binds: killer_world::engine_input::KeyBinds,
    modern_hud: crate::ui_modern::ModernHud,
    ui_captured: bool,
    hud: HudMode,
    lobby_demo: Option<LobbyDemo>,
    download: Option<crate::download::Download>,
    download_source: Option<(String, Option<String>)>,
    keyboard_shown: bool,
    joystick: crate::joystick::Joystick,
    touch_hud: crate::touch_hud::TouchHud,
    joystick_sent: Option<(Instant, (f32, f32))>,
    quit: bool,
    started: Instant,
    first_frame: bool,
    audio: Option<crate::ui_audio_host::AudioHost>,
}

const USER_INI: &str = "user.ini";
const SKY_TILE: &str = "sky";
const L2_INI: &str = "l2.ini";
const BANK_TRIM_INTERVAL: Duration = Duration::from_secs(30);
const OPTION_INI: &str = "option.ini";
const SCREENSHOT_FRAME: u32 = 30;
const DEMO_CAST_SECONDS: f32 = 6.0;
const DEMO_CAST_SKILLS: [i32; 2] = [1177, 3];
const SCREENSHOT_FRAME_ENV: &str = "KILLER_SCREENSHOT_FRAME";
const DEFAULT_UI_SIZE: (i32, i32) = (1280, 720);
const OCCLUDED_PAUSE: Duration = Duration::from_millis(50);
const DEFAULT_HOUR: f32 = 12.0;
const DEMO_GIRAN_CENTER: (f32, f32, f32, i32) = (83400.0, 148000.0, -3400.0, 16384);
const CAPTURE_DIR: [&str; 2] = ["Pictures", "killer"];
const MIN_FRAME_RATE_ENABLED_SYSMSG: u32 = 1283;
const MIN_FRAME_RATE_DISABLED_SYSMSG: u32 = 1284;

fn min_frame_rate_sysstring(was_enabled: bool) -> u32 {
    if was_enabled { MIN_FRAME_RATE_DISABLED_SYSMSG } else { MIN_FRAME_RATE_ENABLED_SYSMSG }
}

pub fn version() -> &'static str {
    option_env!("KILLER_GIT_HASH").filter(|hash| !hash.is_empty()).unwrap_or("sem git")
}

pub fn capture_path(home: Option<PathBuf>, now: SystemTime) -> PathBuf {
    let seconds = now.duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_secs());
    let mut path = home.unwrap_or_default();
    path.extend(CAPTURE_DIR);
    path.join(format!("killer-{seconds}.png"))
}

fn save_capture(path: &Path, size: (u32, u32), rgba: &[u8]) -> Result<(), RenderError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| RenderError::Readback(e.to_string()))?;
    }
    write_png(path, size.0, size.1, rgba).map_err(|e| RenderError::Readback(e.to_string()))
}

const SPLASH_TIMEOUT: Duration = Duration::from_secs(120);

fn coords(v: Vec3) -> [i32; 3] {
    [v.x.round() as i32, v.y.round() as i32, v.z.round() as i32]
}

impl App {
    pub fn new(
        client: Option<PathBuf>,
        tile: Option<String>,
        hour: Option<f32>,
        screenshot: Option<PathBuf>,
        demo: bool,
        demo_crowd: usize,
        lobby_demo: Option<LobbyDemo>,
    ) -> Self {
        let home = PathBuf::from(std::env::var("HOME").unwrap_or_default());
        let default_root = if cfg!(target_os = "ios") { home.join("Documents").join("L2Killer") } else { home.join("L2Killer") };
        let client_root = client.unwrap_or(default_root);
        killer_platform::set_item_order_root(client_root.clone());
        let screen = if tile.is_some() { Screen::World } else { Screen::Login };
        Self {
            walk: crate::walk_world::WalkPhysics::new(client_root.clone()),
            music_zone: None,
            fade_in_after_load: false,
            splash: None,
            client_root,
            touches: crate::touch::Touches::default(),
            applied_ui: None,
            tile,
            hour,
            stage: None,
            screen,
            world_view: None,
            pending: None,
            neighbour_pending: None,
            load_error: None,
            net: None,
            lobby: Lobby::new(ServerConfig::load()),
            world: World::default(),
            labels: Vec::new(),
            chat: ChatLog::default(),
            pending_chat_system: Vec::new(),
            validate: ValidatePacer::new(),
            click: ClickTracker::default(),
            cursor: None,
            game_cursors: Vec::new(),
            texture_cursor: None,
            active_cursor: None,
            ui_scale: 1.0,
            moves: Movement::default(),
            dragging: false,
            right_pressed_at: None,
            last_cursor: None,
            last_frame: Instant::now(),
            material_clock: Instant::now(),
            fps: 0.0,
            screenshot,
            capture: None,
            frames_ready: 0,
            ignored: IgnoredOpcodes::new(Instant::now()),
            tables: None,
            tables_error: None,
            lobby_c4: None,
            ping_rx: None,
            lobby_shift: false,
            cache: CharacterCache::default(),
            items: crate::items::Items::default(),
            fences: crate::server_objects::Fences::default(),
            sound_bank: Default::default(),
            pick_ticket: 0,
            pending_pick: None,
            hover_ticket: 0,
            hover_pending: false,
            hover_pieces: Vec::new(),
            hovered: None,
            effects: Default::default(),
            abnormal: Default::default(),
            appendix: Default::default(),
            last_login: None,
            last_slot: 0,
            server_transfer: None,
            disconnect_pending: false,
            landmark: crate::landmark::LandMark::default(),
            console_mark: crate::ui_olympiad::ConsoleMark::default(),
            landmark_effects: HashMap::new(),
            landmark_loose: Vec::new(),
            landmark_spans: HashMap::new(),
            loader: None,
            uploader: None,
            demo,
            demo_crowd,
            demo_cast: 0.0,
            demo_cast_index: 0,
            profile: FrameProfile::new(Instant::now()),
            game_ui: None,
            video_options: None,
            video_env: parse_ini(""),
            video_apply: None,
            lobby_options: ClientOptions::default(),
            ui_error: None,
            ui_bank: TextureBank::new(),
            banks_trimmed: Instant::now(),
            fishing_target: None,
            replay_logo: killer_ui::replay_menu::ReplayLogoWnd::new((0, 0), crate::lobby_c4::REPLAY_LOGO_TEXTURE, killer_ui::lobby_login::MINI_LOGO),
            fishing_host: Default::default(),
            radar_toggle: false,
            binds: killer_world::engine_input::KeyBinds::default(),
            modern_hud: crate::ui_modern::ModernHud::default(),
            ui_captured: false,
            hud: HudMode::default(),
            lobby_demo,
            download: None,
            download_source: None,
            keyboard_shown: false,
            joystick: Default::default(),
            touch_hud: Default::default(),
            joystick_sent: None,
            quit: false,
            started: Instant::now(),
            first_frame: true,
            audio: None,
        }
    }

    fn start_loading(&mut self, tile: String, anchor: Option<(Vec3, i32)>) {
        self.drop_lobby();
        let (tx, rx) = mpsc::channel();
        let root = self.client_root.clone();
        let want_tables = self.tables.is_none();
        let ui_size = self.game_ui.is_none().then(|| self.ui_size().unwrap_or(DEFAULT_UI_SIZE));
        self.tile = Some(tile.clone());
        self.load_error = None;
        self.neighbour_pending = None;
        if let Some(view) = self.world_view.as_mut() {
            view.neighbours.cancel_all();
        }
        if self.world_view.is_some()
            && let Some(stage) = self.stage.as_mut()
        {
            self.effects.clear(&mut stage.renderer);
            stage.renderer.clear();
        }
        thread::spawn(move || {
            let _ = tx.send(load_tile(&root, &tile, anchor, want_tables, ui_size));
        });
        self.pending = Some(rx);
    }

    fn connect(&mut self, config: ServerConfig, account: String, password: String) {
        self.last_login = Some((config.clone(), account.clone(), password.clone()));
        self.net = Some(spawn_choosing(config.host, config.port, account, password, true));
    }

    fn begin_server_transfer(&mut self, id: i32) {
        let Some(me) = self.world.me.as_ref() else { return };
        let Ok(server) = u8::try_from(id) else { return };
        let Some((config, account, password)) = self.last_login.clone() else { return };
        let position = [me.position.x as i32, me.position.y as i32, me.position.z as i32];
        self.server_transfer = Some(ServerTransfer { server, slot: self.last_slot, position });
        self.net = None;
        self.drop_world();
        self.lobby.back_to_login("troca de servidor".to_string());
        self.screen = Screen::Login;
        self.connect(config, account, password);
    }

    fn select(&mut self, slot: usize) {
        self.last_slot = slot as i32;
        self.send(NetCommand::Select(slot as i32));
    }

    fn run_lobby_commands(&mut self, commands: Vec<LobbyCommand>, event_loop_exit: &mut bool) {
        for command in commands {
            match command {
                LobbyCommand::Connect { config, account, password } => self.connect(config, account, password),
                LobbyCommand::ChooseServer(id) => self.send(NetCommand::ChooseServer(id)),
                LobbyCommand::NoticeAnswer(agreed) => self.send(NetCommand::NoticeAnswer(agreed)),
                LobbyCommand::CardAnswer(value) => self.send(NetCommand::CardAnswer(value)),
                LobbyCommand::Select(slot) => self.select(slot),
                LobbyCommand::NewCharacter => self.send(NetCommand::NewCharacter),
                LobbyCommand::Create(character) => self.send(NetCommand::CharacterCreate(character)),
                LobbyCommand::Delete(slot) => self.send(NetCommand::CharacterDelete { slot: slot as i32 }),
                LobbyCommand::Restore(slot) => self.send(NetCommand::CharacterRestore { slot: slot as i32 }),
                LobbyCommand::Back => self.leave(),
                LobbyCommand::Exit => *event_loop_exit = true,
            }
        }
    }

    fn start_lobby(&mut self) {
        if self.lobby.ready() || self.lobby.loading() || self.stage.is_none() {
            return;
        }
        let want_tables = self.tables.is_none();
        let ui_size = self.game_ui.is_none().then(|| self.ui_size().unwrap_or(DEFAULT_UI_SIZE));
        self.lobby.start_loading(self.client_root.clone(), want_tables, ui_size);
    }

    fn poll_lobby(&mut self) {
        let mut loaded = match self.lobby.poll_loaded() {
            Some(LobbyPart::Scene(loaded)) => loaded,
            Some(LobbyPart::Ground(ground)) => {
                self.lobby.install_ground(*ground);
                return;
            }
            Some(LobbyPart::Extras(mut extras)) => {
                self.install_tables(extras.tables.take(), extras.ui.take());
                log::info!("lobby: extras instalados em {:.1?}", extras.elapsed);
                return;
            }
            None => return,
        };
        let Some(stage) = self.stage.as_mut() else { return };
        let client = match open_uploader(&mut self.uploader, &self.client_root) {
            Ok(c) => c,
            Err(e) => {
                log::error!("cliente: {e}");
                self.lobby.load_error = Some(e.to_string());
                return;
            }
        };
        let started = Instant::now();
        let render_options = ClientOptions::load(client).unwrap_or_default();
        self.lobby_options = render_options.clone();
        stage.renderer.set_no_fractal_anim(render_options.no_fractal_anim);
        stage.renderer.set_projectors(render_options.projectors);
        stage.renderer.set_pawn_shadow(render_options.pawn_shadow, render_options.keep_min_frame_rate);
        self.cache.set_keep_min_frame_rate(render_options.keep_min_frame_rate);
        let mut source = BankSource { bank: &mut loaded.bank, catalog: client.catalog() };
        self.effects.clear(&mut stage.renderer);
        stage.renderer.upload(&loaded.scene, &mut source);
        client.catalog().release_files();
        log::info!(
            "lobby: {} draws, carga {:.1?}, upload {:.1?}",
            loaded.scene.draws.len(),
            loaded.elapsed,
            started.elapsed()
        );
        self.lobby.install(&mut loaded);
        if let Some(demo) = self.lobby_demo.take() {
            self.screen = crate::lobby::demo::apply(&mut self.lobby, demo);
        }
    }

    fn drop_lobby(&mut self) {
        if !self.lobby.ready() {
            return;
        }
        self.lobby.drop_view();
        let removed = self.cache.drain();
        self.abnormal.clear();
        self.appendix.clear();
        if let Some(stage) = self.stage.as_mut() {
            self.effects.clear(&mut stage.renderer);
            stage.renderer.clear();
            for id in removed {
                stage.renderer.remove_character(id);
            }
            stage.renderer.set_instances(&[]);
        }
    }

    fn install_tables(&mut self, tables: Option<Result<DatTables, String>>, ui: Option<Result<GameUi, String>>) {
        match tables {
            Some(Ok(tables)) => {
                self.world.rotation_rates = tables.rotation_rates.clone();
                self.world.enter_events = tables.entereventgrp.clone();
                self.world.walk_anim_frames = tables.walk_anim_frames();
                self.tables = Some(Arc::new(tables));
                self.tables_error = None;
                let pending = std::mem::take(&mut self.pending_chat_system);
                if let Some(tables) = &self.tables {
                    for message in pending {
                        if let Some(line) = ui2d::system_line(tables, &message) {
                            self.chat.push(line);
                        }
                    }
                }
            }
            Some(Err(e)) => {
                log::error!("tabelas: {e}");
                self.tables_error = Some(e);
            }
            None => {}
        }
        match ui {
            Some(Ok(mut ui)) => {
                if let Some(size) = self.ui_size() {
                    ui.resized(size);
                }
                if let Some(tables) = &self.tables {
                    ui.set_tab_names(tables);
                }
                for (path, data) in ui.texture_bank.textures() {
                    if ui.texture_bank.texture_available(&path) == Some(true) {
                        self.ui_bank.insert_shared(&path, data);
                    }
                }
                self.game_ui = Some(ui);
                self.ui_error = None;
            }
            Some(Err(e)) => {
                log::error!("interface: {e}");
                self.ui_error = Some(e);
            }
            None => {}
        }
    }

    fn leave(&mut self) {
        if let Some(net) = &self.net {
            let _ = net.commands.send(NetCommand::Logout);
        }
        self.back_to_login(String::new());
    }

    fn send(&self, command: NetCommand) {
        if let Some(net) = &self.net {
            let _ = net.commands.send(command);
        }
    }

    /// O espaço em que a interface e os rótulos são montados: o quadro dividido pelo fator de
    /// escala da tela, ou seja, a resolução que o cliente teria. A GPU amplia na hora de desenhar.
    fn ui_size(&self) -> Option<(i32, i32)> {
        let (w, h) = self.stage.as_ref().map(|s| s.gpu.size())?;
        let scale = self.ui_scale.max(0.01);
        Some((((w as f32 / scale).round() as i32).max(1), ((h as f32 / scale).round() as i32).max(1)))
    }

    fn drop_world(&mut self) {
        if self.world_view.is_none() && self.pending.is_none() && self.tile.is_none() {
            return;
        }
        if let Some(report) = self.ignored.take_report() {
            log::info!("ao sair do mundo, {report}");
        }
        self.world = World { rotation_rates: std::mem::take(&mut self.world.rotation_rates), enter_events: std::mem::take(&mut self.world.enter_events), walk_anim_frames: std::mem::take(&mut self.world.walk_anim_frames), ..World::default() };
        self.items.clear();
        self.fences.clear();
        self.pending_pick = None;
        self.pick_ticket = self.pick_ticket.wrapping_add(1);
        self.hover_pending = false;
        self.hover_ticket = self.hover_ticket.wrapping_add(1);
        self.hovered = None;
        self.chat = ChatLog::default();
        self.pending_chat_system.clear();
        if let Some(ui) = self.game_ui.as_mut() {
            ui.unfocus_chat();
            ui.mouse_up();
        }
        self.ui_captured = false;
        self.world_view = None;
        self.pending = None;
        self.load_error = None;
        self.tile = None;
        let removed = self.cache.drain();
        self.abnormal.clear();
        self.appendix.clear();
        if let Some(stage) = self.stage.as_mut() {
            self.effects.clear(&mut stage.renderer);
            stage.renderer.clear();
            stage.renderer.clear_ui();
            for id in removed {
                stage.renderer.remove_character(id);
            }
            stage.renderer.set_instances(&[]);
        }
    }

    fn refusal_status(&self, code: u8, host_text: String) -> String {
        if self.modern_hud.preferences.enabled {
            return host_text;
        }
        self.tables
            .as_ref()
            .and_then(|tables| killer_ui::lobby_login::refusal_text(code, |id| tables.systemmsg.get(&id).map(|message| message.text.clone())))
            .unwrap_or(host_text)
    }

    fn blocked_status(&self, flags: Option<u32>, messages: &[String], host_text: String) -> String {
        if self.modern_hud.preferences.enabled {
            return host_text;
        }
        let Some(tables) = self.tables.as_ref() else { return host_text };
        let lookup = |id: u32| tables.systemmsg.get(&id).map(|message| message.text.clone());
        let text = match (flags, messages.last()) {
            (_, Some(message)) if !message.is_empty() => killer_ui::lobby_login::blocked_message_text(message, lookup),
            (Some(flags), None) => killer_ui::lobby_login::blocked_text(flags, lookup),
            _ => None,
        };
        text.unwrap_or_default()
    }

    fn system_message_status(&self, id: u32, host_text: &str) -> String {
        if self.modern_hud.preferences.enabled {
            return host_text.to_string();
        }
        self.tables
            .as_ref()
            .and_then(|tables| tables.systemmsg.get(&id).map(|message| message.text.clone()))
            .unwrap_or_else(|| host_text.to_string())
    }

    fn server_closed(&mut self) {
        if self.screen == Screen::World && self.game_ui.is_some() {
            self.disconnect_pending = true;
            self.world.dialogs.push(killer_world::Dialog::Disconnected { reason: 2 });
            return;
        }
        let status = self.system_message_status(ui2d::DISCONNECTED_RETRY_SYSMSG, "servidor encerrando");
        self.back_to_login(status);
    }

    fn back_to_login(&mut self, status: String) {
        self.disconnect_pending = false;
        self.net = None;
        self.drop_world();
        self.lobby.back_to_login(status);
        self.screen = Screen::Login;
    }

    fn replay_dir(&self) -> PathBuf {
        self.client_root.join("Replay")
    }

    fn run_replay_requests(&mut self, requests: Vec<crate::lobby_c4::ReplayRequest>) {
        use crate::lobby_c4::ReplayRequest;
        for request in requests {
            match request {
                ReplayRequest::List => {
                    let files: Vec<String> = std::fs::read_dir(self.replay_dir())
                        .map(|entries| entries.filter_map(|entry| entry.ok()?.file_name().into_string().ok()).collect())
                        .unwrap_or_default();
                    if let Some(c4) = self.lobby_c4.as_mut() {
                        c4.set_replay_files(&files);
                    }
                }
                ReplayRequest::Delete { path } => {
                    let file = path.rsplit('\\').next().unwrap_or(&path).to_string();
                    let target = self.replay_dir().join(&file);
                    let found = std::fs::read_dir(self.replay_dir())
                        .ok()
                        .and_then(|entries| entries.filter_map(|entry| entry.ok()).find(|entry| entry.file_name().to_string_lossy().eq_ignore_ascii_case(&file)))
                        .map(|entry| entry.path())
                        .unwrap_or(target);
                    if let Err(e) = std::fs::remove_file(&found) {
                        log::warn!("replay {}: {e}", found.display());
                    }
                }
                ReplayRequest::Load { name, chat, .. } => self.play_replay(&name, chat),
                ReplayRequest::PrivateStart { benchmark } => {
                    log::warn!("replay privado ({}): o reprodutor de .rpd/.rcd não existe neste porte, voltando ao menu", if benchmark { "benchmark" } else { "resume" });
                    if let Some(c4) = self.lobby_c4.as_mut() {
                        c4.replay_state = killer_world::replay_private::STATE_MENU;
                    }
                }
            }
        }
    }

    fn read_private_rep(&self, name: &str) -> Option<killer_world::replay_private::RepFile> {
        let wanted = format!("{name}.rep");
        let path = std::fs::read_dir(self.replay_dir())
            .ok()?
            .filter_map(|entry| entry.ok())
            .find(|entry| entry.file_name().to_string_lossy().eq_ignore_ascii_case(&wanted))?
            .path();
        killer_world::replay_private::parse_rep(&std::fs::read_to_string(path).ok()?)
    }

    fn play_replay(&mut self, name: &str, _chat: bool) {
        let wanted = format!("{name}{}", killer_world::replay::EXTENSION);
        let path = std::fs::read_dir(self.replay_dir())
            .ok()
            .and_then(|entries| entries.filter_map(|entry| entry.ok()).find(|entry| entry.file_name().to_string_lossy().eq_ignore_ascii_case(&wanted)))
            .map(|entry| entry.path());
        let Some(path) = path else {
            match self.read_private_rep(name) {
                Some(rep) => log::warn!(
                    "replay {name}: .rep versão {} com {} pacotes lido; o reprodutor de .rpd/.rcd não existe neste porte",
                    rep.header.version,
                    rep.packet_ids.len()
                ),
                None => log::warn!("replay {name}: arquivo ausente"),
            }
            return;
        };
        match std::fs::read(&path).map_err(|e| e.to_string()).and_then(|bytes| killer_world::spawn_replay(&bytes).map_err(|e| e.to_string())) {
            Ok(handle) => {
                self.world = World::default();
                self.net = Some(handle);
            }
            Err(e) => log::warn!("replay {}: {e}", path.display()),
        }
    }

    fn replay_snapshot(&self) -> Option<killer_world::RecordStart> {
        let me = self.world.me.as_ref()?;
        let hour = self.world.clock.map_or(0.0, |clock| clock.hour(Instant::now()));
        let user_ids = self
            .world
            .entities
            .values()
            .filter(|entity| matches!(entity.kind, killer_world::EntityKind::Player { .. }))
            .map(|entity| entity.object_id)
            .chain(std::iter::once(me.object_id))
            .collect();
        Some(killer_world::RecordStart {
            start: me.position.to_array(),
            yaw: me.heading as f32,
            camera: [0.0; 4],
            camera_mode: false,
            first_person: false,
            observer: 0,
            view_mode: 0,
            hour: hour as u8,
            minute: ((hour.fract()) * 60.0) as u8,
            second: ((hour * 3600.0).fract()) * 60.0,
            user_ids,
        })
    }

    fn replay_record(&mut self, command: crate::ui_keys::ReplayRecord) {
        use crate::ui_keys::ReplayRecord;
        use killer_world::RecorderNotice;
        let Some(recorder) = self.net.as_ref().map(|net| std::sync::Arc::clone(&net.recorder)) else { return };
        let Some(snapshot) = self.replay_snapshot() else { return };
        let dir = self.replay_dir();
        let _ = std::fs::create_dir_all(&dir);
        let exists = |name: &str| dir.join(name).metadata().is_ok_and(|meta| meta.len() > 0);
        let write = |name: &str, bytes: &[u8]| std::fs::write(dir.join(name), bytes).is_ok();
        let now = killer_world::now_seconds();
        let Ok(mut recorder) = recorder.lock() else { return };
        let recording = recorder.state() == killer_world::replay::RecorderState::Recording;
        let notice = match command {
            ReplayRecord::Start if !recording => recorder.start(exists, snapshot),
            ReplayRecord::Stop if recording => recorder.stop(now, write),
            ReplayRecord::Toggle => recorder.toggle(now, exists, snapshot, write),
            _ => None,
        };
        drop(recorder);
        let (id, args) = match notice {
            Some(RecorderNotice::Started) => (killer_world::replay::SYSMSG_RECORD_STARTED, Vec::new()),
            Some(RecorderNotice::Saved(file)) => (killer_world::replay::SYSMSG_RECORD_SAVED, vec![l2_net::game::SystemMessageArg::Text(file)]),
            Some(RecorderNotice::Failed) => (killer_world::replay::SYSMSG_RECORD_FAILED, Vec::new()),
            None => return,
        };
        let text = ui2d::system_text(self.tables.as_deref(), id as u32, &args);
        self.chat.push(ChatLine::new(CHAT_SYSTEM, String::new(), text));
    }

    fn expire_recording(&mut self) {
        let Some(recorder) = self.net.as_ref().map(|net| std::sync::Arc::clone(&net.recorder)) else { return };
        let expired = recorder.lock().is_ok_and(|recorder| recorder.should_auto_stop(killer_world::now_seconds()));
        if expired {
            self.replay_record(crate::ui_keys::ReplayRecord::Stop);
        }
    }

    fn pump_replay(&mut self) {
        self.expire_recording();
        let ready = self.world.me.is_some();
        let Some(link) = self.net.as_ref().and_then(|net| net.replay.as_ref()) else { return };
        let _ = link.control.send(killer_world::ReplayCommand::Ready(ready));
        let mut events = Vec::new();
        while let Ok(event) = link.events.try_recv() {
            events.push(event);
        }
        for event in events {
            match event {
                killer_world::PlayEvent::FadeIn(ms) => {
                    if let Some(view) = self.world_view.as_mut() {
                        view.fade.fade_in(ms, killer_world::observer::TELEPORT_FADE_ID);
                    }
                }
                killer_world::PlayEvent::Logo(visible) => self.replay_logo.visible = visible,
                killer_world::PlayEvent::FadeOut => {
                    if let Some(view) = self.world_view.as_mut() {
                        view.fade.fade_out(killer_world::observer::FADE_MS, killer_world::observer::FADE_COLOR, killer_world::observer::TELEPORT_FADE_ID);
                    }
                }
                _ => {}
            }
        }
    }

    fn poll_net(&mut self) {
        loop {
            let event = match &self.net {
                Some(net) => match net.events.try_recv() {
                    Ok(event) => event,
                    Err(TryRecvError::Empty) => return,
                    Err(TryRecvError::Disconnected) => NetEvent::Closed,
                },
                None => return,
            };
            if self.disconnect_pending && matches!(event, NetEvent::Closed | NetEvent::Error(_)) {
                return;
            }
            let screen = next_screen(self.screen, &event);
            match event {
                NetEvent::Connecting => self.lobby.login.status = "conectando…".to_string(),
                NetEvent::Notice(notice) => {
                    if self.lobby_c4_active() {
                        self.lobby.show_notice(notice);
                    } else {
                        self.send(NetCommand::NoticeAnswer(true));
                    }
                }
                NetEvent::CardCheck { card_id, position, retry } => self.lobby.pending_card = Some((card_id, position, retry)),
                NetEvent::ServerList(_) if self.server_transfer.is_some() => {
                    if let Some(transfer) = self.server_transfer.as_ref() {
                        self.send(NetCommand::ChooseServer(transfer.server));
                    }
                }
                NetEvent::ServerList(servers) => {
                    self.lobby.login.connecting = false;
                    self.lobby.set_servers(&servers);
                }
                NetEvent::LoginOk(server) => self.lobby.status = format!("servidor {} em {}: entrando…", server.id, server.address()),
                NetEvent::Session(SessionEvent::Characters(list)) => {
                    self.drop_world();
                    self.lobby.set_characters(list);
                    if let Some(transfer) = self.server_transfer.take() {
                        self.send(NetCommand::SelectTransfer { slot: transfer.slot, position: transfer.position });
                    }
                }
                NetEvent::Session(SessionEvent::RestartResponse(_)) => {
                    self.drop_world();
                    self.lobby.status = "reinício confirmado; aguardando personagens…".to_string();
                }
                NetEvent::Session(SessionEvent::Selected) => self.lobby.status = "entrando…".to_string(),
                NetEvent::Session(event @ SessionEvent::Time(_)) => self.world.apply(&event),
                NetEvent::Session(event @ SessionEvent::UserInfo(_)) => {
                    self.world.apply(&event);
                    if let Some(me) = &self.world.me {
                        self.place(me.object_id);
                    }
                    self.follow_tile();
                }
                NetEvent::Session(SessionEvent::CharCreateFailed(reason)) => self.lobby.create_failed(reason),
                NetEvent::Session(SessionEvent::CharDeleteFailed(reason)) => self.lobby.delete_failed(reason),
                NetEvent::Session(SessionEvent::ServerClosed) => self.server_closed(),
                NetEvent::Session(SessionEvent::CharCreated) => self.lobby.character_created(),
                NetEvent::Session(SessionEvent::CharDeleted) => self.lobby.status = "personagem excluído".to_string(),
                NetEvent::Session(SessionEvent::CharTemplates(templates)) => {
                    self.lobby.set_templates(&templates);
                    self.lobby.new_character_result(true);
                }
                NetEvent::Session(SessionEvent::World(WorldEvent::NewCharacterFailNoop(_))) => self.lobby.new_character_result(false),
                NetEvent::Session(SessionEvent::LeftWorld | SessionEvent::GameGuardQuery(_)) => {}
                NetEvent::Session(SessionEvent::World(event)) => {
                    self.log_chat(&event);
                    log_session_lists(&event);
                    let teleported = matches!(&event, WorldEvent::TeleportToLocation(t) if self.world.me.as_ref().is_some_and(|me| me.object_id == t.target_id));
                    self.world.apply_world(&event);
                    let deferred = self.world.teleport_announce;
                    if let WorldEvent::TeleportToLocation(t) = &event
                        && !deferred
                        && let Some(pawn) = self.world.entity_mut(t.target_id)
                    {
                        pawn.position.z += pawn.collision_height + input::TELEPORT_RAISE;
                    }
                    if let (WorldEvent::ShopPreviewInfo(info), Some(tables)) = (&event, &self.tables) {
                        self.world.apply_shop_preview_info(info, |id| u32::try_from(id).ok().and_then(|id| tables.armorgrp.get(&id)).map(|armor| armor.body_part));
                    }
                    if let (WorldEvent::SsqInfo(info), Some(view)) = (&event, self.world_view.as_mut()) {
                        let hour = self.hour.or_else(|| self.world.clock.map(|clock| clock.hour(Instant::now()))).unwrap_or(DEFAULT_HOUR);
                        if let Err(error) = view.weather.change_env(info.env(), info.immediate(), SSQ_ENV_END, SSQ_ENV_RAMP, SSQ_ENV_TARGET, hour) {
                            log::warn!("SSQInfo: ChangeEnv recusado: {error:?}");
                        }
                    }
                    if let Some(stage) = self.stage.as_mut() {
                        match &event {
                            WorldEvent::DoorInfo(d) => stage.renderer.door_info(d.object_id, d.door_id),
                            WorldEvent::DoorStatusUpdate(d) => stage.renderer.door_status(d.object_id, d.closed, d.damage, d.show_hp),
                            WorldEvent::EventTrigger(trigger) => {
                                let (id, on) = killer_world::world::event_trigger(trigger);
                                stage.renderer.l2_event_trigger(id, on);
                            }
                            _ => {}
                        }
                    }
                    if let Some(object_id) = event.placed_object() {
                        self.place(object_id);
                    }
                    if teleported && !deferred {
                        self.teleported();
                    }
                    if let (WorldEvent::NpcHtmlMessage(message), Some(ui)) = (&event, self.game_ui.as_mut()) {
                        ui.open_npc_html_with(message, self.tables.as_deref());
                    }
                    match &event {
                        WorldEvent::PledgeCrest(crest) => self.register_crest(&ui2d::crest_texture(crest.crest_id), &crest.data),
                        WorldEvent::AllianceCrest(crest) => self.register_crest(&ui2d::ally_crest_texture(crest.crest_id), &crest.data),
                        _ => {}
                    }
                    if let WorldEvent::NpcHtmlMessage(html) = &event {
                        log::info!("html do NPC {}: {} bytes", html.npc_object_id, html.html.len());
                    }
                    if let Some(ui) = self.game_ui.as_mut() {
                        let resolved = ui2d::channel_say_text(self.tables.as_deref(), &event);
                        ui.html_tables.clone_from(&self.tables);
                        ui.world_event(resolved.as_ref().unwrap_or(&event), &self.world);
                        ui.keys.special_maps.world_event(&event, self.tables.as_deref(), &mut ui.desktop);
                        ui.shop_event(&event, self.tables.as_deref());
                        ui.private_shop_event(&event, self.tables.as_deref());
                        ui.multisell_event(&event, &self.world, self.tables.as_deref());
                        ui.manor_event(&event, self.tables.as_deref());
                        ui.hero_event(&event, &self.world);
                        ui.empower_event(&event);
                        ui.observer_event(&event);
                        ui.calc_event(&event);
                        ui.gm_event_world(&event);
                        ui.gm_inventory_event(&event, self.tables.as_deref());
                        ui.gm_status_event(&event, &self.world);
                        ui.gm_skills_event(&event, self.tables.as_deref());
                        ui.gm_quest_event(&event, self.tables.as_deref());
                        ui.gm_clan_event(&event);
                        ui.broadcast_event(&event, self.tables.as_deref(), self.world.target);
                        ui.cast_event(&event, &self.world, self.tables.as_deref());
                        ui.olympiad_event(&event, &self.world, self.tables.as_deref());
                        ui.quest_info_event(&event, self.tables.as_deref());
                        ui.petition_event(&event);
                        ui.snoop_event(&event);
                        ui.train_event(&event, &self.world, self.tables.as_deref());
                        ui.henna_event(&event, self.tables.as_deref());
                        ui.macros_event(&event, &self.world);
                        ui.trade_event(&event, &self.world, self.tables.as_deref());
                    }
                }
                NetEvent::Session(SessionEvent::Unknown { opcode, size }) => {
                    log::debug!("opcode desconhecido {opcode:#04x} ({size} bytes)");
                    self.ignored.note(opcode);
                    if let Some(notice) = input::npc_window_notice(opcode, size) {
                        log::info!("{}", notice.text);
                        self.chat.push(notice);
                    }
                }
                NetEvent::Session(SessionEvent::Malformed { opcode, size }) => {
                    log::debug!("pacote malformado {opcode:#04x} ({size} bytes)");
                    self.ignored.note(opcode);
                }
                NetEvent::Session(SessionEvent::Refused(reason)) => {
                    log::error!("sessão recusada: {reason}");
                    let status = if reason == killer_world::session::PROTOCOL_MISMATCH {
                        self.system_message_status(PROTOCOL_QUIT_SYSMSG, &reason)
                    } else {
                        reason
                    };
                    self.back_to_login(status);
                }
                NetEvent::Error(reason) => {
                    log::error!("rede: {reason}");
                    self.back_to_login(reason);
                }
                NetEvent::LoginRefused { code, text } => {
                    log::error!("rede: {text}");
                    let status = self.refusal_status(code, text);
                    self.back_to_login(status);
                }
                NetEvent::LoginBlocked { flags, messages, text } => {
                    log::error!("rede: {text}");
                    let status = self.blocked_status(flags, &messages, text);
                    self.back_to_login(status);
                }
                NetEvent::Closed => {
                    let status = self.system_message_status(CONNECTION_LOST_SYSMSG, "conexão encerrada pelo servidor");
                    self.back_to_login(status);
                }
            }
            if let Some(action) = self.world.pending_client_action.take() {
                let shift = self.game_ui.as_ref().is_some_and(|ui| ui.keys.shift);
                if let Some(command) = self.world.client_action(action, shift) {
                    self.world.requests.push(command);
                }
            }
            for request in self.world.take_requests() {
                log::debug!("pedido do mundo: {request:?}");
                self.send(request);
            }
            self.screen = screen;
        }
    }

    fn play_interface_sound(&mut self, name: &str) {
        if let Some(host) = &mut self.audio
            && let Ok(client) = open_uploader(&mut self.uploader, &self.client_root)
            && let Err(error) = host.play_interface_sound(client, name)
        {
            log::warn!("áudio: {name}: {error}");
        }
    }

    fn push_system_message(&mut self, message: &l2_net::game::SystemMessage) {
        let Some(tables) = &self.tables else {
            self.pending_chat_system.push(message.clone());
            return;
        };
        let notice = ui2d::is_notice(tables, message);
        let sound = if notice { None } else { ui2d::system_sound(tables, message) };
        let voice = if notice { None } else { ui2d::system_voice(tables, message) };
        if notice {
            self.world.dialogs.push(killer_world::Dialog::Notice(message.clone()));
        }
        if let Some(line) = ui2d::system_line(tables, message) {
            let line = match &mut self.game_ui {
                Some(ui) => ui.route_system_line(line),
                None => Some(line),
            };
            if let Some(line) = line {
                self.chat.push(line);
            }
        }
        if let Some(name) = sound {
            self.play_interface_sound(&name);
        }
        if let Some(name) = voice {
            self.request_system_voice(&name);
        }
    }

    fn play_wav_voice(&mut self, name: &str) {
        if name.is_empty() || name == "None" {
            return;
        }
        if let Some(host) = &mut self.audio
            && let Ok(client) = open_uploader(&mut self.uploader, &self.client_root)
            && let Err(error) = host.play_wav_voice(client, name)
        {
            log::warn!("áudio: voz {name}: {error}");
        }
    }

    fn request_system_voice(&mut self, name: &str) {
        if self.world.me.is_none() {
            return;
        }
        if let Some(host) = &mut self.audio
            && let Ok(client) = open_uploader(&mut self.uploader, &self.client_root)
            && let Err(error) = host.request_wav_voice(client, name)
        {
            log::warn!("áudio: voz {name}: {error}");
        }
    }

    fn log_chat(&mut self, event: &WorldEvent) {
        if let Some(notice) = killer_world::notice_of(event) {
            match notice.color {
                None => self.push_system_message(&l2_net::game::SystemMessage { id: notice.id, args: Vec::new() }),
                Some(color) => {
                    let text = ui2d::system_text(self.tables.as_deref(), notice.id as u32, &[]);
                    let mut line = ChatLine::new(CHAT_SYSTEM, String::new(), text);
                    line.color = Some(color);
                    self.chat.push(line);
                }
            }
            return;
        }
        match event {
            WorldEvent::SystemMessage(message) => self.push_system_message(message),
            WorldEvent::NpcSay(say) => match &self.tables {
                Some(tables) => {
                    if let Some(line) = ui2d::npc_say_line(tables, say) {
                        self.world.receive_speech(say.object_id, line.kind, &line.text);
                        if line.kind == 2 && let Some(ui) = &mut self.game_ui {
                            ui.chat.tells.add(&line.sender);
                        }
                        self.chat.push(line);
                    }
                }
                None => self.chat.apply(event),
            },
            WorldEvent::Say2(message) => {
                let to_channel = matches!(message.kind, killer_ui::command_channel::SAY_ALL_MEMBERS | killer_ui::command_channel::SAY_LEADER | killer_ui::party_room::SAY_PARTY_ROOM);
                if let Some(tables)=&self.tables {
                    if let Some(line)=ui2d::say_line(tables,message) {
                        self.world.receive_speech(message.object_id,message.kind,&line.text);
                        if message.kind==2 && let Some(ui)=&mut self.game_ui {ui.chat.tells.add(&line.sender);}
                        if !to_channel {self.chat.push(line);}
                    }
                } else if let l2_net::game::Say2Body::Text {text,..}=&message.body {
                    self.world.receive_speech(message.object_id,message.kind,text);
                    if !to_channel {self.chat.apply(event);}
                }
            },
            _ => self.chat.apply(event),
        }
    }

    fn register_crest(&mut self, name: &str, data: &[u8]) {
        match self.ui_bank.insert_dds(name, data) {
            Ok((w, h)) => log::debug!("brasão {name}: {w}×{h} ({} bytes)", data.len()),
            Err(why) => log::warn!("brasão {name}: {why}"),
        }
    }

    fn place(&mut self, object_id: i32) {
        if let (Some(view), Some(entity)) = (&self.world_view, self.world.entity_mut(object_id)) {
            view.ground_for(entity.position).place(entity);
        }
    }

    fn follow_tile(&mut self) {
        let Some(me) = &self.world.me else { return };
        let tile = tile_name(me.position.x, me.position.y);
        let no_tile_loaded = self.world_view.is_none() && self.pending.is_none();
        let inside_loaded = self.world_view.as_ref().is_some_and(|view| TileId::parse(&tile).is_some_and(|id| view.neighbours.is_loaded(id)));
        let tile_changed = self.tile.as_deref() != Some(tile.as_str()) && !inside_loaded && self.pending.is_none();
        if no_tile_loaded || tile_changed {
            log::info!("no mundo como {} em {:.0} {:.0} {:.0} (tile {tile})", me.name, me.position.x, me.position.y, me.position.z);
            log::debug!("colisão de {}: raio {} altura {}", me.name, me.collision_radius, me.collision_height);
            let anchor = (me.position, me.heading);
            self.start_loading(tile, Some(anchor));
        }
    }

    fn apply_controller_calls(&mut self, calls: Vec<(i32, killer_world::walk::ControllerCall)>) {
        let (commands, stopped) = crate::walk_world::controller_commands(&mut self.world, &calls);
        if stopped {
            self.landmark.delete(false);
        }
        for command in commands {
            self.send(command);
        }
    }

    fn teleported(&mut self) {
        let Some(me) = &self.world.me else { return };
        let server = me.position - Vec3::Z * (me.collision_height + input::TELEPORT_RAISE);
        log::info!("teleporte para {:.0} {:.0} {:.0}", server.x, server.y, server.z);
        self.send(NetCommand::Validate { position: coords(server), heading: me.heading });
        self.send(NetCommand::Appearing);
        if let Some(host) = &mut self.audio {
            host.zone.reset();
        }
        if let Some(id) = self.world.me.as_ref().map(|me| me.object_id) {
            self.walk.forget(id);
        }
        self.follow_tile();
    }

    fn click_to_move(&mut self, pixel: (f32, f32)) {
        if self.pending.is_some() {
            return;
        }
        if self.world.me.is_none() {
            return;
        }
        if let Some(object_id) = self.game_ui.as_ref().and_then(|ui| characters::label_at(&self.labels, &ui.large, pixel)) {
            self.pending_pick = None;
            self.pick_ticket = self.pick_ticket.wrapping_add(1);
            self.select_target(object_id);
            return;
        }
        let (ctrl,shift) = self.game_ui.as_ref().map_or((false,false),|ui| (ui.keys.ctrl,ui.keys.shift));
        self.request_pick(pixel,PendingPick::Move { ctrl,shift });
    }

    fn request_pick(&mut self, pixel: (f32,f32), pending: PendingPick) {
        let Some(size) = self.game_ui.as_ref().map(|ui| ui.desktop.size).or_else(|| self.ui_size()).map(|(w,h)| (w as u32,h as u32)) else { return; };
        let Some(view) = &self.world_view else { return; };
        let camera = view.current();
        let Some((origin, dir)) = input::pixel_ray(&camera, size, pixel) else { return };
        let Some(loader) = &self.loader else { return; };
        if self.pending_pick.is_some() { return; }
        self.pick_ticket = self.pick_ticket.wrapping_add(1);
        let request = crate::loader::PickRequest {
            views:crate::items::views(view,self.stage.as_ref().map(|stage| &stage.renderer),origin),
            query:input::pick_query(origin,dir,self.game_ui.as_ref().is_some_and(|ui| ui.keys.shift)),
            targets:self.items.targets(&self.world),
            spawned:self.fences.colliders(&self.world),
        };
        if loader.requests.send(LoadRequest::Pick { ticket:self.pick_ticket,request }).is_ok() {
            self.pending_pick = Some(pending);
        }
    }

    fn request_hover(&mut self) {
        if self.screen != Screen::World || self.hover_pending { return; }
        let Some(pixel) = self.cursor else { self.hovered = None; return; };
        let Some(size) = self.game_ui.as_ref().map(|ui| ui.desktop.size).map(|(w,h)| (w as u32,h as u32)) else { return; };
        let Some(view) = &self.world_view else { return; };
        let Some((origin, dir)) = input::pixel_ray(&view.current(), size, pixel) else { self.hovered = None; return; };
        let Some(loader) = &self.loader else { return; };
        let pieces = self.items.targets_with_pieces(&self.world);
        self.hover_ticket = self.hover_ticket.wrapping_add(1);
        let request = crate::loader::PickRequest {
            views:crate::items::views(view,self.stage.as_ref().map(|stage| &stage.renderer),origin),
            query:input::pick_query(origin,dir,self.game_ui.as_ref().is_some_and(|ui| ui.keys.shift)),
            targets:pieces.iter().map(|(target,_)| *target).collect(),
            spawned:self.fences.colliders(&self.world),
        };
        if loader.requests.send(LoadRequest::Hover { ticket:self.hover_ticket,request }).is_ok() {
            self.hover_pending = true;
            self.hover_pieces = pieces;
        }
    }

    fn finish_hover(&mut self, ticket: u64, selection: Result<Option<killer_assets::collision::world::Selection>,String>) {
        if ticket != self.hover_ticket { return; }
        self.hover_pending = false;
        let pieces = std::mem::take(&mut self.hover_pieces);
        self.hovered = match selection.map(|selection| selection.map(|selection| input::bind_server_object(&self.world, selection))) {
            Ok(selection) => selection.and_then(|selection| selection.target).map(|target| Hovered {
                object_id:target.object_id,
                spawned:target.spawned,
                slot:pieces.iter().find(|(piece,_)| same_target(piece,&target)).and_then(|(_,slot)| *slot),
            }),
            Err(error) => { log::warn!("ator sob o mouse: {error}"); None }
        };
    }

    fn targets_a_player(&self) -> bool {
        self.world.target.is_some_and(|id| self.world.entity(id).is_some_and(|entity| matches!(entity.kind,killer_world::EntityKind::Player { .. })))
    }

    fn finish_pick(&mut self, ticket: u64, selection: Result<Option<killer_assets::collision::world::Selection>,String>) {
        if ticket != self.pick_ticket { return; }
        let Some(pending) = self.pending_pick.take() else { return; };
        let hit = match selection {
            Ok(Some(hit)) => input::bind_server_object(&self.world, hit),
            Ok(None) => return,
            Err(error) => { log::warn!("seleção no mundo: {error}"); return; }
        };
        let (ctrl,shift) = match pending {
            PendingPick::Move { ctrl,shift } => (ctrl,shift),
            PendingPick::Drop(mut dialog) => {
                if let killer_world::Dialog::DropItem { object_id,position,.. } = &mut dialog {
                    if self.world.inventory.find(*object_id).is_none() { return; }
                    if self.targets_a_player() { return; }
                    *position = coords(hit.hit.position);
                    self.world.dialogs.push(dialog);
                }
                return;
            }
        };
        if self.world_view.as_ref().is_some_and(|view| view.options.l2_mark == 2) && self.world.me.is_some() {
            let life = self.landmark_life(crate::landmark::MARK_CLASS);
            self.landmark.add(hit.hit.position,hit.hit.normal,hit.surface,life,crate::landmark::sine());
        }
        if let Some(target) = hit.target {
            if (self.world.entity(target.object_id).is_some_and(|entity| entity.spawned == target.spawned) || self.world.static_target(target.object_id).is_some())
                && let Some(command) = target_action(&mut self.world,target.object_id,ctrl,shift) {
                self.send(command);
                self.landmark.delete(false);
            }
            return;
        }
        if !input::ground_click_allowed(&self.world, self.moves.forward || self.moves.back, shift) {
            return;
        }
        let Some(me) = self.world.me.as_mut() else { return };
        let target = coords(hit.hit.position);
        let command = NetCommand::MoveTo { target, origin: coords(me.position) };
        me.dest = Some(Vec3::new(target[0] as f32, target[1] as f32, target[2] as f32));
        log::debug!("andar para {} {} {}", target[0], target[1], target[2]);
        self.send(command);
        if let Some(command) = self.game_ui.as_mut().and_then(|ui| ui.tutorial.fire(killer_ui::tutorial::EVENT_MOVE)) {
            self.send(command);
        }
    }

    fn press_touch_button(&mut self, button: crate::touch_hud::Button) {
        if let Some(ui) = self.game_ui.as_mut() {
            ui.keys.run_macro_text_command(&self.world, button.command());
        }
        self.run_ui_commands();
    }

    fn drive_joystick(&mut self) {
        const STEP: f32 = 500.0;
        const REPEAT: Duration = Duration::from_millis(250);
        let vector = self.joystick.vector();
        let (Some(view), Some(me)) = (self.world_view.as_ref(), self.world.me.as_ref()) else { return };
        if view.use_free || self.pending.is_some() {
            return;
        }
        let Some((right, forward)) = vector else {
            if self.joystick_sent.take().is_some() {
                let position = me.position;
                self.send(NetCommand::MoveTo { target: coords(position), origin: coords(position) });
                if let Some(me) = self.world.me.as_mut() {
                    me.dest = None;
                }
            }
            return;
        };
        let due = self.joystick_sent.is_none_or(|(at, last)| at.elapsed() >= REPEAT || last.0 * right + last.1 * forward < 0.96);
        if !due {
            return;
        }
        let camera = view.current();
        let ahead = (camera.target - camera.eye).truncate().normalize_or_zero();
        let side = glam::Vec2::new(-ahead.y, ahead.x);
        let direction = ahead * forward + side * right;
        let from = me.position;
        let x = from.x + direction.x * STEP;
        let y = from.y + direction.y * STEP;
        let z = view.ground_for(Vec3::new(x, y, from.z)).ground_at(x, y, from.z + 200.0, from.z - 400.0).unwrap_or(from.z);
        let target = Vec3::new(x, y, z);
        self.joystick_sent = Some((Instant::now(), (right, forward)));
        self.send(NetCommand::MoveTo { target: coords(target), origin: coords(from) });
        if let Some(me) = self.world.me.as_mut() {
            me.dest = Some(target);
        }
    }

    fn landmark_life(&mut self, class: &'static str) -> f32 {
        if let Some(life) = self.landmark_spans.get(class) { return *life; }
        let Ok(client) = open_uploader(&mut self.uploader,&self.client_root) else { return 0.0; };
        let life = client.class_float_defaults(class,&["LifeSpan"]).ok().and_then(|values| values.get("LifeSpan").copied()).unwrap_or(0.0);
        self.landmark_spans.insert(class,life);
        life
    }

    fn item_drop(&mut self, object_id: i32, item_id: i32, count: i32, name: String, target: crate::ui_keys::ItemDropTarget) {
        use crate::ui_keys::ItemDropTarget;
        let etc = self.tables.as_deref().and_then(|tables| u32::try_from(item_id).ok().and_then(|id| tables.etcitemgrp.get(&id)));
        let stackable = etc.is_some_and(|etc| etc.stackable != 0) && count >= 2;
        let (kind, message_id, count, position) = match target {
            ItemDropTarget::World => {
                let Some(pixel) = self.cursor else { return; };
                if self.targets_a_player() { return; }
                let (message_id,count) = if etc.is_some_and(|etc| etc.family == ui2d::DROP_PLACE_FAMILY) {
                    (ui2d::DROP_PLACE_SYSMSG,None)
                } else if stackable { (ui2d::DROP_COUNT_SYSMSG,Some(count)) }
                else { (ui2d::DROP_CONFIRM_SYSMSG,None) };
                self.request_pick(pixel,PendingPick::Drop(killer_world::Dialog::DropItem {
                    object_id,name,message_id,count,position:[0;3],kind:killer_world::ItemDropKind::Drop,
                }));
                return;
            }
            ItemDropTarget::Trash if stackable => (killer_world::ItemDropKind::Destroy, ui2d::DESTROY_COUNT_SYSMSG, Some(count), [0; 3]),
            ItemDropTarget::Trash => (killer_world::ItemDropKind::Destroy, ui2d::DESTROY_CONFIRM_SYSMSG, None, [0; 3]),
            ItemDropTarget::Crystallize => (killer_world::ItemDropKind::Crystallize, ui2d::CRYSTALLIZE_SYSMSG, None, [0; 3]),
            ItemDropTarget::Pet | ItemDropTarget::PetEquipped | ItemDropTarget::FromPet => {
                let give = target != ItemDropTarget::FromPet;
                if !stackable {
                    self.send(if give { NetCommand::GiveItemToPet { object_id, count: 1 } } else { NetCommand::GetItemFromPet { object_id, count: 1 } });
                    return;
                }
                let kind = if give { killer_world::ItemDropKind::GiveToPet } else { killer_world::ItemDropKind::GetFromPet };
                let message_id = if target == ItemDropTarget::PetEquipped { ui2d::PET_EQUIPPED_COUNT_SYSMSG } else { ui2d::PET_MOVE_COUNT_SYSMSG };
                (kind, message_id, Some(count), [0; 3])
            }
        };
        self.world.dialogs.push(killer_world::Dialog::DropItem { object_id, name, message_id, count, position, kind });
    }

    fn initialize_video_options(&mut self) -> Result<(), String> {
        let Some(stage) = &mut self.stage else { return Ok(()) };
        let options = match self.world_view.as_mut() {
            Some(view) => &mut view.options,
            None => &mut self.lobby_options,
        };
        if self.video_options.is_none() {
            let client = open_uploader(&mut self.uploader, &self.client_root).map_err(|e| e.to_string())?;
            let option = parse_ini(&client.read_system_text(OPTION_INI).map_err(|e| e.to_string())?);
            let l2 = parse_ini(&client.read_system_text(L2_INI).map_err(|e| e.to_string())?);
            self.video_env = parse_ini(&client.read_system_text("env.int").map_err(|e| e.to_string())?);
            let display = crate::ui_video::VideoDisplay::capture(&stage.window, &stage.gpu.config)
                .ok_or("monitor não forneceu resolução/frequência para Settings")?;
            let caps = crate::ui_video::RendererVideoCapabilities::query(&stage.renderer);
            let mut runtime = crate::ui_video::VideoOptionRuntime::from_ini(&option, &l2, options, display, caps)?;
            runtime.state.screenshot_quality = option.int("Game", "ScreenShotQuality").unwrap_or(0);
            let settings = runtime.settings();
            let post = killer_render::postprocess::PostProcessSettings::read(settings.post_process_type, settings.keep_min_frame_rate != 0, &self.video_env);
            stage.renderer.set_antialiasing(&stage.gpu.adapter, if post.active() { 0 } else { settings.antialiasing })?;
            stage.renderer.set_postprocess(post)?;
            let detail = if settings.keep_min_frame_rate != 0 { 2 } else { settings.texture_detail };
            let mut bank = crate::ui_video::reload_bank(&self.ui_bank);
            let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
            stage.renderer.reload_texture_detail(detail, &mut source);
            self.video_options = Some(runtime);
        }
        if let Some(runtime) = &mut self.video_options {
            runtime.apply_render_state(&mut stage.renderer, options);
        }
        Ok(())
    }

    fn settings_error(&mut self, error: impl std::fmt::Display) {
        let text = format!("Settings: {error}");
        log::warn!("{text}");
        self.chat.push(ChatLine::new(CHAT_SYSTEM, String::new(), text));
    }

    fn toggle_min_frame_rate(&mut self) {
        if self.video_apply.is_some() {
            return;
        }
        if let Err(error) = self.initialize_video_options() {
            self.settings_error(error);
            return;
        }
        let Some(stage) = self.stage.as_ref() else {
            self.settings_error("janela de vídeo indisponível");
            return;
        };
        let multisample = killer_render::video::multisample_capability(&stage.gpu.adapter, stage.renderer.device());
        let (Some(runtime), Some(ui)) = (&mut self.video_options, &mut self.game_ui) else {
            self.settings_error("estado de vídeo indisponível");
            return;
        };
        let was_enabled = runtime.state.settings.keep_min_frame_rate != 0;
        runtime.show(&mut ui.keys.option.video_page, multisample, |id| ui.keys.sysstrings.get(&id).cloned());
        ui.keys.option.video_page.checks[0].checked = !was_enabled;
        ui.keys.option.video_page.keep_min_frame_changed(runtime.capabilities.water, runtime.capabilities.shaders);
        let text = ui2d::system_text(self.tables.as_deref(), min_frame_rate_sysstring(was_enabled), &[]);
        let mut line = ChatLine::new(CHAT_SYSTEM, String::new(), text);
        line.color = Some(0xffb09b79);
        self.chat.push(line);
        self.begin_video_apply(None);
    }

    fn begin_video_apply(&mut self, container: Option<bool>) {
        if self.video_apply.is_some() { return; }
        if let Err(error) = self.initialize_video_options() {
            self.settings_error(error);
            return;
        }
        let (Some(runtime), Some(stage), Some(ui)) = (&self.video_options, &self.stage, &self.game_ui) else {
            self.settings_error("estado de vídeo indisponível");
            return;
        };
        let page = &ui.keys.option.video_page;
        let mut next = runtime.state.clone();
        next.settings.reduce_pawn_lod = i32::from(page.combos[4].selected != 0);
        next.settings.skip_anim = i32::from(page.combos[5].selected != 0);
        if let Err(error) = crate::ui_video::validate_pawn_options(&runtime.state, &next) {
            self.settings_error(error);
            return;
        }
        self.video_apply = Some(crate::ui_video::PendingVideoApply {
            transaction: crate::ui_video::VideoApplyTransaction::new(page, runtime, &stage.renderer),
            page: page.clone(), container,
        });
        self.advance_video_apply();
    }

    fn advance_video_apply(&mut self) {
        use crate::ui_video::{ApplyProgress, WorldVideoResources};
        let Some(mut pending) = self.video_apply.take() else { return };
        if let Some(ui) = &mut self.game_ui { ui.keys.set_option_applying(false); }
        let (Some(runtime), Some(stage)) = (&mut self.video_options, &mut self.stage) else {
            self.settings_error("estado de vídeo removido durante a aplicação");
            return;
        };
        let client = match open_uploader(&mut self.uploader, &self.client_root) {
            Ok(client) => client,
            Err(error) => { self.settings_error(error); return; }
        };
        let progress = match self.world_view.as_mut() {
            Some(view) => {
                let mut resources = WorldVideoResources { client, bank: &mut self.ui_bank, view };
                pending.transaction.advance(runtime, &mut pending.page, &stage.window,
                    &mut stage.renderer, &stage.gpu.adapter, &self.video_env, &mut resources)
            }
            None => {
                let mut resources = crate::ui_video::LobbyVideoResources { client, bank: &mut self.ui_bank };
                pending.transaction.advance(runtime, &mut pending.page, &stage.window,
                    &mut stage.renderer, &stage.gpu.adapter, &self.video_env, &mut resources)
            }
        };
        client.catalog().release_files();
        match progress {
            ApplyProgress::Waiting => {
                log::info!("Settings: Apply de vídeo aguardando a janela confirmar a resolução");
                if let Some(ui) = &mut self.game_ui { ui.keys.set_option_applying(true); }
                self.video_apply = Some(pending);
            }
            ApplyProgress::Failed(error) => self.settings_error(error),
            ApplyProgress::Complete => {
                log::info!("Settings: Apply de vídeo concluído ({}x{})", runtime.display.params.width, runtime.display.params.height);
                let options = match self.world_view.as_mut() {
                    Some(view) => &mut view.options,
                    None => &mut self.lobby_options,
                };
                runtime.apply_render_state(&mut stage.renderer, options);
                if let Some(ui) = &mut self.game_ui {
                    ui.keys.option.video_page = pending.page;
                }
                let edits = pending.transaction.take_edits();
                match pending.container {
                    Some(close) => self.apply_container_pages(close, edits),
                    None => self.save_option_edits(edits),
                }
            }
        }
    }

    fn apply_container_pages(&mut self, close: bool, mut edits: Vec<crate::ui_option_config::ConfigEdit>) {
        log::info!("Settings: Apply do container (fechar: {close}), áudio presente: {}", self.audio.as_ref().is_some_and(|a| a.device.is_some()));
        use crate::ui_option_apply::{AbsentAudio, FIRST_RUN, GameApplyEffects, GameApplyHost};
        let mut effects = GameApplyEffects::default();
        let mut native = None;
        if let Some(ui) = &mut self.game_ui {
            match &mut self.audio {
                Some(host) => {
                    let mut backend = crate::ui_audio_host::AudioApplyHost {
                        host, root: &self.client_root, controller_present: self.world.me.is_some(), edits: &mut edits,
                    };
                    ui.keys.option.audio_page.apply(&mut backend);
                }
                None => ui.keys.option.audio_page.apply(&mut AbsentAudio),
            }
            match &mut ui.option_game_runtime {
                Some(runtime) => {
                    let mut state = runtime.state.clone();
                    let mut host = GameApplyHost { runtime, shortcut: &mut ui.shortcut, edits: &mut edits, effects: &mut effects };
                    ui.keys.option.game_page.apply(&mut state, &mut host);
                    native = Some(state.is_native != 0);
                    runtime.state = state;
                }
                None => log::warn!("Settings: estado de Game indisponível"),
            }
        }
        let (section, key, value) = FIRST_RUN;
        edits.push(crate::ui_option_config::ConfigEdit::new(section, key, value));
        if let (true, Some(native)) = (effects.clear_game_data, native) {
            self.apply_language_effects(native, &effects);
        }
        if let Some(ui) = &mut self.game_ui {
            crate::ui_option_apply::apply_system_message(&mut ui.desktop, &effects);
        }
        if effects.reset_cursor && let Some(stage) = &self.stage {
            stage.window.set_cursor(winit::window::CursorIcon::Default);
        }
        self.save_option_edits(edits);
        if close && let Some(ui) = &mut self.game_ui {
            ui.keys.option.lose_combo_focus();
            if let Some(window) = ui.desktop.window_mut(killer_ui::window::WindowId::OPTION) {
                window.visible = false;
            }
        }
    }

    fn apply_language_effects(&mut self, native: bool, effects: &crate::ui_option_apply::GameApplyEffects) {
        self.tables = Some(Arc::new(DatTables::default()));
        let loaded = open_uploader(&mut self.uploader, &self.client_root).map_err(|e| e.to_string()).and_then(|client| {
            let mut language = client.client_language();
            language.is_native = native;
            let tables = client.tables_for_language(&language).map_err(|e| e.to_string());
            client.catalog().release_files();
            tables
        });
        match loaded {
            Ok(tables) => self.tables = Some(Arc::new(tables)),
            Err(error) => self.settings_error(format!("FL2GameData::Load: {error}")),
        }
        if effects.refresh_localized_windows && let (Some(ui), Some(tables)) = (&mut self.game_ui, self.tables.as_deref()) {
            crate::ui_option_apply::relocalize_windows(&mut ui.keys, tables);
        }
    }

    fn save_option_edits(&mut self, edits: Vec<crate::ui_option_config::ConfigEdit>) {
        let path = self.client_root.join("system").join(OPTION_INI);
        match crate::ui_option_config::save(&path, &edits) {
            Ok(saved) => {
                if let Some(ui) = &mut self.game_ui {
                    let ini = parse_ini(&saved);
                    crate::ui_options::synchronize_game_options(ui, ini);
                    if let Some(view) = &mut self.world_view {
                        crate::ui_options::synchronize_name_options(&mut view.display, &ui.options);
                    }
                }
                log::info!("opções gravadas em {}", path.display());
            }
            Err(e) => log::warn!("opções: falha ao gravar {}: {e}", path.display()),
        }
    }

    fn save_windows_info(&mut self) {
        let Some(ui) = &self.game_ui else { return };
        let path = self.client_root.join("system").join("WindowsInfo.ini");
        match crate::ui_windows_info::save(&path, ui) {
            Ok(()) => log::info!("posições e larguras gravadas em {}", path.display()),
            Err(e) => log::warn!("WindowsInfo.ini: falha ao gravar {}: {e}", path.display()),
        }
    }

    fn assist_member(&mut self, member: i32) {
        let Some(target) = self.world.entity(member).and_then(|entity| entity.target) else { return };
        if self.world.entity(target).is_none() && self.world.static_target(target).is_none() {
            return;
        }
        let Some(me) = self.world.me.as_ref() else { return };
        let origin = me.position.to_array().map(|v| v as i32);
        let shift = self.game_ui.as_ref().is_some_and(|ui| ui.keys.shift);
        self.send(NetCommand::Action { object_id: target, origin, shift });
    }

    fn select_target(&mut self, object_id: i32) {
        let (ctrl, shift) = self.game_ui.as_ref().map_or((false, false), |ui| (ui.keys.ctrl, ui.keys.shift));
        let name = self.world.entity(object_id).map(|e| e.name.clone()).unwrap_or_default();
        match target_action(&mut self.world, object_id, ctrl, shift) {
            Some(command) => {
                log::info!("alvo {object_id} \"{name}\": {command:?}");
                self.send(command);
            }
            None => log::info!("alvo {object_id} \"{name}\": nenhum pedido enviado"),
        }
    }

    fn say(&mut self, typed: String) {
        let Some(ui) = self.game_ui.as_mut() else { return };
        let effects = ui.chat_line(&typed, &self.world);
        for effect in effects {
            match effect {
                killer_ui::chat_line::LineEffect::Chat { kind, text, target } => self.send(NetCommand::Say { text, kind, target }),
                killer_ui::chat_line::LineEffect::Command(line) => {
                    if let Some(ui) = self.game_ui.as_mut() {
                        ui.keys.chat_command(&self.world, &line);
                    }
                }
                killer_ui::chat_line::LineEffect::StoreSearch(word) => self.world.search_stores(&word),
                killer_ui::chat_line::LineEffect::Builder(command) => self.send(NetCommand::BypassBuildCmd { command }),
                killer_ui::chat_line::LineEffect::Console(command) => {
                    let captured = self.game_ui.as_mut().is_some_and(|ui| ui.chat.capture.exec(&command));
                    if !captured {
                        log::info!("console: {command}");
                    }
                }
                killer_ui::chat_line::LineEffect::Text(text) => {
                    let mut line = ChatLine::new(CHAT_SYSTEM, String::new(), text);
                    line.color = Some(killer_ui::chat_line::CONSOLE_TEXT_COLOR);
                    self.chat.push(line);
                }
                killer_ui::chat_line::LineEffect::System(id) => {
                    if let Some(ui) = self.game_ui.as_mut() {
                        ui.keys.chat_system(id);
                    }
                }
            }
        }
        self.run_ui_commands();
    }

    /// O `OK` do painel de filtro grava o `chatfilter.ini` do cliente, como `0x100555a0`.
    fn save_chat_filter(&mut self, text: &str) {
        let Ok(client) = open_uploader(&mut self.uploader, &self.client_root) else { return };
        if let Err(error) = client.write_system_text("chatfilter.ini", text) {
            log::warn!("não deu pra salvar o chatfilter.ini: {error}");
        }
    }

    fn poll_loaded(&mut self) {
        let Some(rx) = &self.pending else { return };
        match rx.try_recv() {
            Ok(Ok(loaded)) => {
                self.pending = None;
                self.install(loaded);
            }
            Ok(Err(e)) => {
                self.pending = None;
                log::error!("tile: {e}");
                self.load_error = Some(e);
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
                let e = "carga do tile terminou sem resposta".to_string();
                log::error!("{e}");
                self.load_error = Some(e);
            }
        }
    }

    fn install(&mut self, mut loaded: LoadedTile) {
        let tile = loaded.tile;
        self.install_tables(loaded.tables.take(), loaded.ui.take());
        self.walk.absorb(std::mem::take(&mut loaded.collision));
        self.binds = std::mem::take(&mut loaded.binds);
        let Some(stage) = self.stage.as_mut() else {
            return;
        };
        let client = match open_uploader(&mut self.uploader, &self.client_root) {
            Ok(c) => c,
            Err(e) => {
                log::error!("cliente: {e}");
                self.load_error = Some(e.to_string());
                return;
            }
        };
        let started = Instant::now();
        let mut source = BankSource { bank: &mut loaded.bank, catalog: client.catalog() };
        self.effects.clear(&mut stage.renderer);
        stage.renderer.upload(&loaded.scene, &mut source);
        if let Some(sky) = &loaded.sky {
            stage.renderer.add_tile(SKY_TILE, sky, &mut source);
        }
        stage.renderer.set_no_fractal_anim(loaded.options.no_fractal_anim);
        stage.renderer.set_projectors(loaded.options.projectors);
        stage.renderer.set_pawn_shadow(loaded.options.pawn_shadow, loaded.options.keep_min_frame_rate);
        for (&object_id, door) in &self.world.doors {
            let Some(door_id) = door.info_door_id else { continue };
            stage.renderer.door_info(object_id, door_id);
            if door.status_received { stage.renderer.door_status(object_id, door.closed, door.damage, door.show_hp); }
        }
        client.catalog().release_files();
        let draws = loaded.scene.draws.len();
        let triangles = loaded.scene.draws.iter().map(|d| d.world_index_count() / 3).sum();
        let zones = loaded.scene.zones.len();
        let trace = loaded.trace;
        let lighting = LightMap::new(&loaded.level, &loaded.scene.lights);
        drop(loaded.scene);
        log::info!("tile {tile}: {draws} draws, {triangles} triângulos ({} no traço), carga {:.1?}, upload {:.1?}", trace.triangles(), loaded.elapsed, started.elapsed());
        let mut camera = loaded.camera;
        if let Some(previous) = self.world_view.as_ref().map(|view| &view.camera) {
            camera.yaw = previous.yaw;
            camera.pitch = previous.pitch;
            camera.zoom = previous.zoom;
            camera.desired_zoom = previous.desired_zoom;
            camera.distance = previous.distance;
        }
        let ground = loaded.ground;
        let neighbours = NeighbourSet::new(TileId::parse(&tile).unwrap_or(TileId { x: 0, y: 0 }), loaded.level.clone());
        self.neighbour_pending = None;
        if self.demo && self.net.is_none() && self.world.me.is_none() {
            self.world = characters::demo_world(camera.target, camera.facing(), self.demo_crowd);
            ui_keys::demo_fill(&mut self.world);
            if let Some(ui) = self.game_ui.as_mut() {
                let windows = std::env::var("KILLER_DEMO_WINDOWS").unwrap_or_default();
                ui.keys.open_demo_windows(&mut ui.desktop, &windows);
                for window in windows.split(',').map(str::trim) {
                    if matches!(window, "shop" | "sell") {
                        ui.shop_event(&crate::ui_shop::ShopWindow::demo_event(window == "sell"), self.tables.as_deref());
                    }
                    if window == "pet" || window == "summon" {
                        let info = l2_net::game::PetInfo {
                            summon_type: if window == "summon" { l2_net::game::SUMMON_SERVITOR } else { l2_net::game::SUMMON_PET }, object_id: 0x7fff_0001, npc_id: 12077, level: 15,
                            hp: 420, max_hp: 600, mp: 180, max_mp: 240, sp: 1200, exp: 6000, exp_this_level: 5000, exp_next_level: 8000,
                            fed: 700, max_fed: 1000, load: 540, max_load: 1000, p_atk: 110, p_def: 95, m_atk: 60, m_def: 70,
                            accuracy: 40, evasion: 38, critical: 44, speed: 140, atk_spd: 278, cast_spd: 333, soulshots_per_hit: 1, spiritshots_per_hit: 1,
                            ..Default::default()
                        };
                        self.world.pet = Some(killer_world::Pet { info, max_time: 0, remaining_time: 0 });
                        ui.world_event(&WorldEvent::PetStatusShow(l2_net::game::PetStatusShow { summon_type: if window == "summon" { l2_net::game::SUMMON_SERVITOR } else { l2_net::game::SUMMON_PET } }), &self.world);
                    }
                    if window == "tutorial" {
                        let html = "<html><body>Tutorial<br><br>Click the <font color=\"LEVEL\">question mark</font> to continue.<br><a action=\"link tutorial_1.htm\">Next</a></body></html>";
                        ui.world_event(&WorldEvent::TutorialShowHtml(l2_net::game::TutorialShowHtml { html: html.into() }), &self.world);
                        ui.world_event(&WorldEvent::TutorialShowQuestionMark(l2_net::game::TutorialShowQuestionMark { mark_id: 1 }), &self.world);
                    }
                    if let (Some(tables), "recipe" | "manufacture") = (self.tables.as_deref(), window) {
                        let mut ids: Vec<i32> = tables.recipes.keys().copied().collect();
                        ids.sort_unstable();
                        let book = l2_net::game::RecipeBookItemList { common: false, max_mp: 500, recipes: ids.iter().take(20).map(|id| (*id, 1)).collect() };
                        self.world.recipes.book = Some(book.clone());
                        ui.keys.recipes.world_event(&WorldEvent::RecipeBookItemList(book));
                        if window == "manufacture" && let Some(id) = ids.first() {
                            let info = l2_net::game::RecipeItemMakeInfo { recipe_id: *id, common: false, mp: 320, max_mp: 500, status: 1 };
                            self.world.recipes.make_info = Some(info);
                            ui.keys.recipes.world_event(&WorldEvent::RecipeItemMakeInfo(info));
                        }
                    }
                }
            }
            log::info!("modo demo: personagem e {} entidades em volta do PlayerStart", self.world.entities.len());
        }
        for entity in self.world.me.iter_mut().chain(self.world.entities.values_mut()) {
            ground.place(entity);
        }
        if let Some(ui) = self.game_ui.as_mut() {
            ui.reset_zone();
        }
        let (fade, fade_shown) = self.world_view.as_ref().map_or_else(Default::default, |view| (view.fade, view.fade_shown));
        self.world_view = Some(WorldView {
            moving_trace: crate::trace::MovingTrace::default(),
            tile,
            trace,
            zones,
            lighting,
            camera,
            display: loaded.display,
            free: FreeCamera::from_orbit(&camera),
            use_free: false,
            timeenv: loaded.weather.base().clone(),
            weather: loaded.weather,
            bsp_tree: loaded.bsp_tree,
            ground,
            shakes: Default::default(),
            fade,
            fade_shown,
            draws,
            triangles,
            characters: 0,
            instances: 0,
            neighbours,
            options: loaded.options.clone(),
        });
        if std::mem::take(&mut self.fade_in_after_load)
            && let Some(view) = self.world_view.as_mut()
        {
            view.fade.fade_in(killer_world::observer::FADE_MS, 0);
        }
        if let Err(error) = self.initialize_video_options() {
            log::warn!("Settings Video: {error}");
        }
    }

    fn advance_neighbours(&mut self) {
        if self.pending.is_some() {
            return;
        }
        let Some(me) = self.world.me.as_ref().map(|me| me.position) else { return };
        let Some(view) = self.world_view.as_mut() else { return };
        for name in view.neighbours.follow(me) {
            if let Some(stage) = self.stage.as_mut() {
                stage.renderer.remove_tile(&name);
            }
            log::info!("tile {name} descartado a mais de 35000 u");
        }
        if view.neighbours.running.is_some() || self.neighbour_pending.is_some() {
            return;
        }
        let Some(job) = view.neighbours.jobs.pop_front() else { return };
        let root = self.client_root.clone();
        let options = view.options.clone();
        let (tx, rx) = mpsc::channel();
        match &job {
            Job::Load(tile) => {
                let tile = *tile;
                let seam = view.neighbours.seam_levels(tile);
                thread::spawn(move || {
                    let _ = tx.send(load_neighbour(&root, tile, &options, seam).map(|loaded| NeighbourResult::Loaded(Box::new(loaded))));
                });
            }
            Job::Reseam(name) => {
                let Some(id) = TileId::parse(name) else { return };
                let Some(level) = view.neighbours.level_of(id) else { return };
                let seam = view.neighbours.seam_levels(id);
                let name = name.clone();
                thread::spawn(move || {
                    let _ = tx.send(reseam(&root, &name, level, seam).map(NeighbourResult::Reseamed));
                });
            }
        }
        view.neighbours.running = Some(job);
        self.neighbour_pending = Some(rx);
    }

    fn poll_neighbours(&mut self) {
        let Some(rx) = &self.neighbour_pending else { return };
        let result = match rx.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err("carga do vizinho terminou sem resposta".to_string()),
        };
        self.neighbour_pending = None;
        let Some(view) = self.world_view.as_mut() else { return };
        let job = view.neighbours.running.take();
        let Some(me) = self.world.me.as_ref().map(|me| me.position) else { return };
        match result {
            Ok(NeighbourResult::Loaded(mut loaded)) => {
                if let (Some(stage), Ok(client)) = (self.stage.as_mut(), open_uploader(&mut self.uploader, &self.client_root)) {
                    let started = Instant::now();
                    let mut source = BankSource { bank: &mut loaded.bank, catalog: client.catalog() };
                    stage.renderer.add_tile(&loaded.tile.name, &loaded.scene, &mut source);
                    client.catalog().release_files();
                    log::info!("vizinho {}: {} draws, {} triângulos, carga {:.1?}, upload {:.1?}", loaded.tile.name, loaded.tile.draws, loaded.tile.triangles, loaded.elapsed, started.elapsed());
                }
                self.walk.absorb(std::mem::take(&mut loaded.collision));
                for name in view.neighbours.attach(loaded.tile, me) {
                    if let Some(stage) = self.stage.as_mut() {
                        stage.renderer.remove_tile(&name);
                    }
                    log::info!("tile {name} expulso: mais de 4 vizinhos");
                }
            }
            Ok(NeighbourResult::Reseamed(mut seamed)) => {
                if let (Some(stage), Ok(client)) = (self.stage.as_mut(), open_uploader(&mut self.uploader, &self.client_root)) {
                    let mut source = BankSource { bank: &mut seamed.bank, catalog: client.catalog() };
                    let tile = if seamed.name == view.neighbours.window.main.name() { String::new() } else { seamed.name.clone() };
                    stage.renderer.replace_draws(&tile, DrawKind::Terrain, &seamed.draws, &mut source);
                    client.catalog().release_files();
                    log::info!("tile {} costurado de novo ({} draws de terreno)", seamed.name, seamed.draws.len());
                }
            }
            Err(e) => {
                log::error!("vizinho: {e}");
                if let Some(Job::Load(tile)) = job {
                    view.neighbours.fail(tile);
                }
            }
        }
    }

    fn update_lens_flare(&mut self, hour: f32, dt: f32) {
        let (Some(view), Some(stage)) = (self.world_view.as_ref(), self.stage.as_mut()) else { return };
        let size = stage.gpu.size();
        stage.renderer.update_coronas(&view.current(), size.0 as f32 / size.1 as f32, dt, |a, b| view.nearest(a, b).is_none());
        let Some(me) = self.world.me.as_ref() else { return };
        let sky = killer_render::environment::sky_state_weather(&view.weather, hour);
        let end = me.position + sky.sun_trace * range_units(view.options.clipping.static_mesh);
        stage.renderer.set_lens_flare_clear(view.nearest(me.position, end).is_none());
    }

    fn update_tile_visibility(&mut self) {
        let (Some(view), Some(stage)) = (self.world_view.as_ref(), self.stage.as_mut()) else { return };
        let target = viewer_position(&self.world, &view.camera);
        let drawn = view.neighbours.drawn(target, view.options.clipping.actor);
        let main = view.neighbours.window.main.name();
        for name in view.neighbours.tiles.keys().chain(std::iter::once(&main)) {
            let key = if *name == main { "" } else { name.as_str() };
            stage.renderer.set_tile_visible(key, drawn.contains(name));
        }
    }

    fn request_characters(&mut self) {
        let Some(tables) = &self.tables else { return };
        if let (Some(view),Some(stage)) = (&self.world_view,&mut self.stage) {
            self.items.sync(&self.world,&mut stage.renderer);
            let loader = self.loader.get_or_insert_with(|| Loader::spawn(self.client_root.clone()));
            self.items.request(tables,view,&stage.renderer,&loader.requests);
            self.fences.sync(&self.world,&mut stage.renderer);
            self.fences.request_with(&self.world,view,&stage.renderer,view.options.clipping,|request| loader.requests.send(request).is_ok());
        }
        let max_loads = self
            .world_view
            .as_ref()
            .map_or(1, |view| view.options.max_pawn_resource_load.max(1.0).floor() as usize);
        let wanted = self.cache.take_wanted(max_loads);
        if wanted.is_empty() {
            return;
        }
        let loader = self.loader.get_or_insert_with(|| Loader::spawn(self.client_root.clone()));
        for key in wanted {
            match characters::look(key, tables) {
                Some(look) => {
                    if loader.requests.send(LoadRequest::Character { key, appearance: look.appearance, draw_scale: look.draw_scale }).is_err() {
                        self.cache.fail(key);
                    }
                }
                None => {
                    log::warn!("aparência {key:?} sem linha no chargrp");
                    self.cache.fail(key);
                }
            }
        }
    }

    fn poll_characters(&mut self) {
        loop {
            let result = match &self.loader {
                Some(loader) => match loader.results.try_recv() {
                    Ok(result) => result,
                    Err(TryRecvError::Empty) => return,
                    Err(TryRecvError::Disconnected) => {
                        self.loader = None;
                        self.items.retry_pending();
                        self.fences.retry_pending();
                        self.pending_pick = None;
                        let retried = self.cache.retry_pending();
                        log::error!("carga de personagens terminou sem resposta; repetindo {retried:?}");
                        return;
                    }
                },
                None => return,
            };
            self.install_character(result);
        }
    }

    fn install_character(&mut self, result: LoadResult) {
        if let LoadResult::Pick { ticket,selection } = result {
            self.finish_pick(ticket,selection);
            return;
        }
        if let LoadResult::Hover { ticket,selection } = result {
            self.finish_hover(ticket,selection);
            return;
        }
        let Some(stage) = self.stage.as_mut() else { return };
        match result {
            LoadResult::Pick { .. } | LoadResult::Hover { .. } => {},
            LoadResult::Item { item_id,asset } => {
                match open_uploader(&mut self.uploader,&self.client_root) {
                    Ok(client) => self.items.install_asset(item_id,asset,&mut stage.renderer,client),
                    Err(error) => { log::error!("cliente para item {item_id}: {error}"); self.items.retry_pending(); }
                }
            }
            LoadResult::PlaceItem { ticket,pieces } => self.items.install_placement(ticket,pieces),
            LoadResult::Fence { batch } => {
                match open_uploader(&mut self.uploader,&self.client_root) {
                    Ok(client) => self.fences.install(batch,&mut stage.renderer,client),
                    Err(error) => { log::error!("cliente para a cerca do coliseu: {error}"); self.fences.retry_pending(); }
                }
            }
            LoadResult::Failed { key, reason } => {
                log::warn!("personagem {key:?}: {reason}");
                self.cache.fail(key);
            }
            LoadResult::Character { key, render_defaults, pawn, parts, animation, draw_scale, weapon_type, mut bank, effects } => {
                if !self.cache.is_pending(key) {
                    return;
                }
                let client = match open_uploader(&mut self.uploader, &self.client_root) {
                    Ok(client) => client,
                    Err(e) => {
                        log::error!("cliente: {e}");
                        self.cache.fail(key);
                        return;
                    }
                };
                let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
                let uploaded = parts
                    .into_iter()
                    .map(|mesh| {
                        let id = stage.renderer.upload_character(&mesh, &mut source);
                        (id, Arc::new(mesh))
                    })
                    .collect();
                client.catalog().release_files();
                match LoadedCharacter::new(uploaded, animation, draw_scale, weapon_type) {
                    Some(mut character) => {
                        character.render_defaults = render_defaults;
                        log::info!("personagem {key:?} carregado");
                        self.cache.insert(key, character.with_effects(effects.effects, effects.bank).with_pawn(&pawn));
                    }
                    None => {
                        log::warn!("personagem {key:?} sem malha mestra (slot 7); não desenhado");
                        self.cache.fail(key);
                    }
                }
            }
        }
    }

    fn animate_lobby(&mut self, dt: f32) {
        if self.screen == Screen::World || !self.lobby.ready() {
            return;
        }
        let (Some(tables), Some(stage)) = (&self.tables, self.stage.as_mut()) else { return };
        let size = stage.gpu.size();
        let instances = self.lobby.instances(&mut self.cache, tables, dt, size);
        stage.renderer.set_instances(&instances);
        let world = &self.lobby.world;
        if let Ok(client)=open_uploader(&mut self.uploader,&self.client_root) {
            self.effects.sync(&mut stage.renderer,&mut self.cache,client,world,&ClientOptions::default());
        }
    }

    fn demo_cast(&mut self, dt: f32) {
        if !self.demo || self.net.is_some() {
            return;
        }
        self.demo_cast += dt;
        if self.demo_cast < DEMO_CAST_SECONDS {
            return;
        }
        let Some(me) = self.world.me.as_ref() else { return };
        self.demo_cast = 0.0;
        let skill_id = DEMO_CAST_SKILLS[self.demo_cast_index % DEMO_CAST_SKILLS.len()];
        self.demo_cast_index += 1;
        let event = WorldEvent::MagicSkillUse(l2_net::game::MagicSkillUse {
            caster_id: me.object_id,
            target_id: characters::DEMO_TARGET_ID,
            skill_id,
            skill_level: 1,
            hit_time: 1500,
            reuse_delay: 0,
            x: me.position.x as i32,
            y: me.position.y as i32,
            z: me.position.z as i32,
            extra: Vec::new(),
            positions: Vec::new(),
        });
        self.world.apply_world(&event);
    }

    fn animate(&mut self, dt: f32) {
        self.demo_cast(dt);
        let hour = self.game_hour(Instant::now());
        let time = self.material_clock.elapsed().as_secs_f32();
        let arrival_life = self.landmark_life(crate::landmark::ARRIVAL_CLASS);
        let tile_loading = self.pending.is_some();
        let (Some(view), Some(tables), Some(stage)) = (self.world_view.as_mut(), &self.tables, self.stage.as_mut()) else {
            return;
        };
        let size = stage.gpu.size();
        let frustum = view.current().frustum(size.0 as f32 / size.1.max(1) as f32);
        let frame = Frame { display: view.display, viewer: viewer_position(&self.world, &view.camera), dt };
        let drawn: std::collections::HashSet<i32> = characters::drawn_pawns(&self.world, tables, &frame).into_iter().collect();
        self.world.rendered.clone_from(&drawn);
        for character in self.cache.evict_drawn(&self.world, &drawn) {
            stage.renderer.remove_character(character);
        }
        let mut instances = characters::instances_drawn(
            &mut self.world,
            &mut self.cache,
            tables,
            &frame,
            |p| view.zone_at(p),
            |query| view.actor_lighting(query),
            |center, radius| frustum.contains_sphere(center, radius),
            |_| None,
            &drawn,
        );
        if !view.use_free
            && (view.camera.default_move.hide_player || self.world.observer.hides_player())
            && let Some(me) = self.world.me.as_ref().map(|me| me.object_id)
        {
            instances.retain(|instance| instance.actor_id != me);
        }
        self.items.sync(&self.world,&mut stage.renderer);
        self.fences.sync(&self.world,&mut stage.renderer);
        stage.renderer.set_lines(&crate::server_objects::lines(&self.world));
        instances.extend(self.items.instances(dt,view,&mut stage.renderer,open_uploader(&mut self.uploader,&self.client_root).ok()));
        instances.extend(self.effects.arrow_instances(&*view));
        for request in self.items.take_sounds() {
            if let Some(host) = &mut self.audio && let Some(device) = &mut host.device
                && let Err(error) = device.play_pickup(&request,host.frame) { log::warn!("som do item {}: {error}",request.sound.name); }
        }
        for sound in self.world.effects.take_enter_sounds() {
            self.cache.push_sound_plan(crate::world_sound::SoundPlan {
                actor: sound.object_id,
                name: sound.name,
                position: glam::Vec3::from(sound.position),
                volume: sound.volume * crate::world_sound::VOLUME_SCALE,
                radius: sound.radius,
                skip_if_playing: false,
            });
        }
        if let Ok(client) = open_uploader(&mut self.uploader,&self.client_root) {
            for plan in self.cache.take_sound_plans() {
                let Some(sound) = self.sound_bank.resolve(client,&plan) else { continue };
                if let Some(host) = &mut self.audio && let Some(device) = &mut host.device
                    && !(sound.skip_if_playing && device.is_playing(sound.actor,sound.key))
                    && let Err(error) = device.play_world(&sound,host.frame) { log::warn!("som {}: {error}",sound.name); }
            }
        }
        let eye = view.camera.camera().eye.to_array();
        view.shakes.measure(dt);
        for quake in self.world.effects.take_earthquakes() {
            let (shake, sound) = killer_world::camera_shake::earthquake([quake.x, quake.y, quake.z], quake.intensity, quake.duration);
            view.shakes.add(&shake, eye);
            if let Some(me) = self.world.me.as_ref() {
                self.cache.push_sound_plan(crate::world_sound::SoundPlan {
                    actor: me.object_id,
                    name: sound.to_owned(),
                    position: Vec3::from(eye),
                    volume: killer_world::camera_shake::EARTHQUAKE_SOUND_VOLUME * crate::world_sound::VOLUME_SCALE,
                    radius: killer_world::camera_shake::EARTHQUAKE_SOUND_RADIUS,
                    skip_if_playing: false,
                });
            }
        }
        for item_id in self.world.effects.take_equip_sounds() {
            if let Some(name) = tables.equip_sound(item_id)
                && let Some(host) = &mut self.audio
                && let Ok(client) = open_uploader(&mut self.uploader, &self.client_root)
                && let Err(error) = host.play_interface_sound(client, name)
            {
                log::warn!("áudio: {name}: {error}");
            }
        }
        for sound in self.world.effects.take_sounds() {
            if (sound.kind == 1 || sound.kind == 2)
                && !sound.sound_file.is_empty()
                && self.world.me.is_some()
                && let Some(host) = &mut self.audio
            {
                let delay = (sound.delay / 1000) as f32;
                if sound.kind == 1 {
                    host.zone.request_server_music(&sound.sound_file, delay);
                } else {
                    host.zone.request_server_voice(&sound.sound_file, delay, true);
                }
                continue;
            }
            if sound.kind == 0
                && !sound.sound_file.is_empty()
                && let Some(me) = self.world.me.as_ref()
            {
                self.cache.push_sound_plan(crate::world_sound::SoundPlan {
                    actor: me.object_id,
                    name: sound.sound_file,
                    position: me.position,
                    volume: 1.0,
                    radius: crate::world_sound::DEFAULT_RADIUS,
                    skip_if_playing: false,
                });
            }
        }
        for plan in self.cache.take_view_shakes() {
            view.shakes.add_notify(&plan.notify,plan.location.to_array(),eye,plan.randoms);
            if let Some(range) = plan.smoke {
                stage.renderer.l2_event_smoke(plan.location.to_array(), range);
            }
        }
        for shake in self.cache.take_explosion_shakes() {
            view.shakes.add(&shake,eye);
        }
        for (location, radius) in self.cache.take_event_smokes() {
            stage.renderer.l2_event_smoke(location.to_array(), radius);
        }
        view.shakes.update(dt);
        for fade in self.cache.take_screen_fades() {
            view.fade.blink(fade.fade_out_duration, fade.fade_out_color, fade.fade_in_duration, fade.black_out_duration);
        }
        if std::mem::take(&mut self.world.observer.fade_out_pending) {
            view.fade.fade_out(killer_world::observer::FADE_MS, killer_world::observer::FADE_COLOR, killer_world::observer::TELEPORT_FADE_ID);
        }
        if std::mem::take(&mut self.world.observer.camera_default_pending) {
            view.camera.observer_default_camera();
        }
        let mut observer_moved = false;
        if let Some(ui) = self.game_ui.as_mut() {
            for op in std::mem::take(&mut ui.keys.olympiad.mark_ops) {
                self.console_mark.apply(op, &mut view.options.l2_mark);
            }
            for on in std::mem::take(&mut ui.keys.broadcast.controller) {
                self.world.observer.ctrl_broadcast = on;
            }
            for op in std::mem::take(&mut ui.keys.broadcast.canvas_ops).into_iter().chain(std::mem::take(&mut ui.keys.olympiad.canvas_ops)) {
                self.world.appendix.canvas_apply(op);
            }
            for op in std::mem::take(&mut ui.keys.broadcast.marks) {
                self.world.apply_appendix(op);
            }
        }
        if let Some(done) = view.fade.update(dt)
            && done.id == killer_world::observer::TELEPORT_FADE_ID
        {
            observer_moved = self.world.observer_fade_done();
            if self.world.teleport_announce {
                self.fade_in_after_load = true;
            } else {
                view.fade.fade_in(killer_world::observer::FADE_MS, 0);
            }
        }
        if tile_loading {
            stage.renderer.set_flash(0.0, [0.0; 3]);
            view.fade_shown = true;
        } else if view.fade.active() || view.fade_shown {
            let (scale, fog) = view.fade.flash();
            stage.renderer.set_flash(scale, fog);
            view.fade_shown = view.fade.active();
        }
        view.characters = self.cache.loaded_count();
        view.instances = instances.len();
        // Atraso entre comandos de macro: as durações saem do AnimSet do próprio jogador,
        // recalculadas no tick de personagens (ver `MACRO_ACTION_SOCIAL_ANIM` em ui_keys).
        if let Some(ui) = self.game_ui.as_mut() {
            ui.keys.social_durations.clone_from(&self.cache.social_durations);
        }
        self.profile.lap("instâncias", Instant::now());
        stage.renderer.set_time(time);
        stage.renderer.update_sun_visibility(frame.viewer, killer_render::environment::Environment::from_time(&view.timeenv, hour).sun_dir, &instances, |a, b| view.nearest(a, b).is_none());
        if let Some(me) = self.world.me.as_ref().map(|me| me.object_id)
            && let Ok(client) = open_uploader(&mut self.uploader, &self.client_root)
        {
            let eye = view.current().eye;
            let zone = view.zone_at(eye);
            let fishing = self.fishing_host.update(&mut self.world.fishing_view, client, &mut stage.renderer, &mut self.cache, crate::fishing_host::FishingContext { me, camera: eye, zone, dt });
            instances.extend(fishing.instance);
            let mut lines = crate::server_objects::lines(&self.world);
            lines.extend(fishing.lines);
            stage.renderer.set_lines(&lines);
        }
        stage.renderer.set_instances(&instances);
        if let Ok(client)=open_uploader(&mut self.uploader,&self.client_root) {
            let shaken = view.current();
            self.effects.set_viewer(shaken.eye,killer_assets::scene::transform::vector_rotation(shaken.target-shaken.eye),view.zone_at(shaken.eye));
            if let Some(tables) = self.tables.as_deref() { self.abnormal.sync(&mut self.cache,&mut self.world,&drawn,client,tables,dt); }
            self.appendix.sync(&mut self.cache, &mut self.world, client);
            self.effects.sync(&mut stage.renderer,&mut self.cache,client,&self.world,&view.options);
            self.effects.fly(&mut stage.renderer,&mut self.cache,&self.world,dt);
            self.effects.firecrackers(&mut stage.renderer,&mut self.cache,client,&mut self.world,&view.options,dt);
            self.effects.enter_events(&mut stage.renderer,client,&mut self.world,&view.options);
            let arrow_option = self.game_ui.as_ref().is_some_and(|ui| ui.keys.option.checked(crate::quest_arrow::ARROW_CHECK));
            let zone = self.world.me.as_ref().map_or(0, |me| view.zone_at(me.position));
            self.effects.quest_arrow(&mut stage.renderer,&mut self.cache,client,&mut self.world,&view.options,arrow_option,zone);
        }
        let pawn = self.world.me.as_ref().map(|me| crate::landmark::Pawn { location: me.position, waiting: self.cache.me_waiting });
        self.landmark.update(dt, pawn.as_ref(), arrival_life);
        let mut landmark = LandmarkEffects { active: &mut self.landmark_effects, loose: &mut self.landmark_loose, assets: &mut self.effects.assets, bank: &mut self.effects.bank };
        sync_landmark(&mut stage.renderer, self.landmark.take_requests(), &mut landmark, &mut self.uploader, &self.client_root, view, dt);
        self.profile.lap("set_instances", Instant::now());
        if observer_moved {
            if std::mem::take(&mut self.world.teleport_announce) {
                if let Some(id) = self.world.me.as_ref().map(|me| me.object_id) {
                    self.place(id);
                }
                self.teleported();
                if self.pending.is_none() && std::mem::take(&mut self.fade_in_after_load)
                    && let Some(view) = self.world_view.as_mut()
                {
                    view.fade.fade_in(killer_world::observer::FADE_MS, 0);
                }
            } else {
                self.follow_tile();
            }
        }
    }

    fn update_cursor(&mut self, event_loop: &ActiveEventLoop) {
        use crate::cursor::Request;
        if self.cursor.is_none() { return; }
        let request = self.game_ui.as_ref().map_or(Request::Normal, |ui| crate::cursor::select(ui, &self.world, self.hovered.map(|hover| (hover.object_id, hover.spawned))));
        if let Request::Texture(path) = &request
            && self.texture_cursor.as_ref().is_none_or(|(cached, _)| cached != path)
        {
            let cursor = open_uploader(&mut self.uploader, &self.client_root).ok()
                .and_then(|client| client.catalog().texture_raw(path))
                .and_then(|raw| killer_assets::cursor::from_texture(&raw))
                .and_then(|cursor| winit::window::CustomCursor::from_rgba(cursor.rgba, cursor.width, cursor.height, cursor.hotspot[0], cursor.hotspot[1]).ok())
                .map(|source| event_loop.create_custom_cursor(source));
            self.texture_cursor = Some((path.clone(), cursor));
        }
        let cursor = match request {
            Request::System => None,
            Request::Normal => self.game_cursors.first().and_then(Option::as_ref),
            Request::Resource(index) => self.game_cursors.get(usize::from(index)).and_then(Option::as_ref),
            Request::Texture(_) => self.texture_cursor.as_ref().and_then(|(_, cursor)| cursor.as_ref()),
        };
        if !matches!(request, Request::Texture(_)) || cursor.is_some() {
            self.active_cursor = cursor.cloned();
        }
        if let Some(stage) = &self.stage {
            match &self.active_cursor {
                Some(cursor) => stage.window.set_cursor(cursor.clone()),
                None => stage.window.set_cursor(winit::window::CursorIcon::Default),
            }
        }
    }

    fn input(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::CursorLeft { .. } => { self.cursor = None; self.last_cursor = None; }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Middle, .. } => {
                let heading = self.world.me.as_ref().map(|me| me.heading);
                if let (Some(view), Some(heading)) = (self.world_view.as_mut(), heading)
                    && !view.use_free
                    && view.camera.settings.turn_back_on_middle
                    && view.camera.turn_back(heading)
                    && let Some(command) = self.game_ui.as_mut().and_then(|ui| ui.tutorial.fire(killer_ui::tutorial::EVENT_TURN_BACK))
                {
                    self.send(command);
                }
            }
            WindowEvent::MouseInput { state, button: button @ (MouseButton::Left | MouseButton::Right), .. } => {
                self.dragging = state.is_pressed();
                self.last_cursor = self.cursor;
                if button == MouseButton::Right {
                    if state.is_pressed() {
                        self.right_pressed_at = Some(Instant::now());
                    } else if let Some(since) = self.right_pressed_at.take() {
                        if let (Some(ui), Some((cx, cy))) = (self.game_ui.as_mut(), self.cursor) {
                            ui.right_released(cx as i32, cy as i32, &self.world);
                        }
                        let held = since.elapsed().as_secs_f32() * 1000.0;
                        let heading = self.world.me.as_ref().map(|me| me.heading);
                        if let (Some(view), Some(heading)) = (self.world_view.as_mut(), heading)
                            && !view.use_free
                            && view.camera.settings.fixed_on_right_release_ms.is_some_and(|max| held <= max)
                            && view.camera.fixed_default_camera(heading, 0)
                            && let Some(command) = self.game_ui.as_mut().and_then(|ui| ui.tutorial.fire(killer_ui::tutorial::EVENT_DEFAULT_CAMERA))
                        {
                            self.send(command);
                        }
                    }
                }
                if button == MouseButton::Left {
                    if state.is_pressed() {
                        self.click.press(self.cursor);
                    } else if let Some(pixel) = self.click.release() {
                        self.click_to_move(pixel);
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale = f64::from(self.ui_scale.max(0.01));
                let cursor = ((position.x / scale) as f32, (position.y / scale) as f32);
                self.cursor = Some(cursor);
                self.click.moved(cursor);
                if self.dragging
                    && let (Some(last), Some(view)) = (self.last_cursor, self.world_view.as_mut())
                {
                    let (dx, dy) = (cursor.0 - last.0, cursor.1 - last.1);
                    if view.use_free {
                        view.free.look(dx, dy);
                    } else if !(self.world.camera_fixed && self.world.pending_click_target.is_some()) {
                        view.camera.drag(dx, dy);
                        let (yaw, pitch) = view.camera.settings.drag_radians(dx, dy);
                        let fired: Vec<_> = self.game_ui.as_mut().map(|ui| [ui.tutorial.camera_rotated(yaw), ui.tutorial.camera_rotated(pitch)].into_iter().flatten().collect()).unwrap_or_default();
                        for command in fired {
                            self.send(command);
                        }
                    }
                }
                self.last_cursor = Some(cursor);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let ticks = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 40.0,
                };
                if let Some(view) = self.world_view.as_mut() {
                    view.camera.wheel(ticks, self.world.me.as_ref().map(|me| me.collision_radius));
                    if let Some(command) = self.game_ui.as_mut().and_then(|ui| ui.tutorial.fire(killer_ui::tutorial::EVENT_ZOOM)) {
                        self.send(command);
                    }
                }
            }
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(code), state, repeat, .. }, .. } => {
                self.key(code, state.is_pressed(), repeat);
            }
            _ => {}
        }
    }

    fn exec_engine_commands(&mut self, commands: Vec<String>, action: killer_world::engine_input::InputAction, delta: f32) {
        let has_pawn = self.world.me.is_some();
        for command in commands {
            killer_world::engine_input::exec_radar(&mut self.world.windows.radar, &command, has_pawn);
            if let Some(view) = self.world_view.as_mut() {
                view.camera.exec_input(&command, action, delta);
                let pawn = self.world.me.as_ref().map(|me| (me.heading, 0));
                let fired = match view.camera.exec_pawn_camera(&command, action, 0.0, pawn, self.world.camera_fixed) {
                    crate::camera::PawnCameraExec::FixedView => Some(killer_ui::tutorial::EVENT_DEFAULT_CAMERA),
                    crate::camera::PawnCameraExec::TurnBack => Some(killer_ui::tutorial::EVENT_TURN_BACK),
                    crate::camera::PawnCameraExec::Ignored => None,
                };
                if let Some(event) = fired
                    && let Some(command) = self.game_ui.as_mut().and_then(|ui| ui.tutorial.fire(event))
                {
                    self.send(command);
                }
            }
            let mut cmd = command.as_str();
            if killer_world::engine_input::parse_command(&mut cmd, "CAMERAMODESELECTINGCANCEL") && self.world.camera_fixed {
                self.world.pending_click_target = None;
            }
        }
    }

    fn key(&mut self, code: KeyCode, pressed: bool, repeat: bool) {
        if let Some(vk) = ui_keys::virtual_key(code)
            && let Some(link) = self.net.as_ref().and_then(|net| net.replay.as_ref())
        {
            let action = if pressed { killer_world::replay::ACTION_PRESS } else { killer_world::replay::ACTION_RELEASE };
            let _ = link.control.send(killer_world::ReplayCommand::Input { key: vk, action, level_loaded: self.world_view.is_some() });
        }
        if let Some(vk) = ui_keys::virtual_key(code) {
            let (commands, action) = if pressed {
                (self.binds.press(vk), killer_world::engine_input::InputAction::Press)
            } else {
                (self.binds.release(vk), killer_world::engine_input::InputAction::Release)
            };
            self.exec_engine_commands(commands, action, 0.0);
        }
        match code {
            KeyCode::KeyW => self.moves.forward = pressed,
            KeyCode::KeyS => self.moves.back = pressed,
            KeyCode::KeyA => self.moves.left = pressed,
            KeyCode::KeyD => self.moves.right = pressed,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => self.moves.fast = pressed,
            KeyCode::KeyF if pressed && !repeat => {
                if let Some(view) = self.world_view.as_mut() {
                    view.toggle_free();
                }
            }
            KeyCode::KeyH if pressed && !repeat => self.hud = self.hud.next(),
            _ => {}
        }
    }

    fn tick_dialogs(&mut self) {
        let Some(ui) = self.game_ui.as_mut() else { return };
        ui.show_pet_window(&ui2d::world_views(&self.world, self.tables.as_deref(), Instant::now()));
        ui.tick_zone_title(Instant::now());
        ui.tick_tutorial(Instant::now());
        ui.tick_edit_boxes(Instant::now());
        ui.tick_command_channel(Instant::now());
        if let Some(KeyAction::Dialog(answer, accept)) =
            ui.tick_dialog(&mut self.world, self.tables.as_deref(), Instant::now())
        {
            self.answer_dialog(answer, accept);
        }
    }

    fn answer_dialog(&mut self, answer: DialogAnswer, accept: bool) {
        if answer == DialogAnswer::Disconnected {
            if accept {
                let status = self.system_message_status(CONNECTION_LOST_SYSMSG, "conexão encerrada pelo servidor");
                self.back_to_login(status);
            }
            return;
        }
        if let DialogAnswer::SystemMenu { restart } = answer {
            if accept && !restart {
                self.leave();
                return;
            }
            if accept && let Some(ui) = self.game_ui.as_mut() {
                ui.system_menu_restart();
            }
        }
        if let Some(command) = dialog_command(answer, accept) {
            if let Some(name) = input::confirm_sound(&command) {
                self.play_interface_sound(name);
            }
            self.send(command);
        }
    }

    fn game_ui_event(&mut self, event: &WindowEvent) -> KeyAction {
        let in_world = self.screen == Screen::World && self.world_view.is_some();
        let Some(ui) = self.game_ui.as_mut().filter(|_| in_world) else {
            return KeyAction::Ignored;
        };
        let consumed = |yes: bool| if yes { KeyAction::Consumed } else { KeyAction::Ignored };
        match event {
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                let Some((x, y)) = self.cursor else {
                    return KeyAction::Ignored;
                };
                if !ui.mouse_down_at(x as i32, y as i32, Instant::now(), &self.world) {
                    return KeyAction::Ignored;
                }
                self.ui_captured = true;
                self.click.cancel();
                if let Some(action) = ui.take_board_command() {
                    return action;
                }
                if std::mem::take(&mut ui.target_clicked)
                    && let Some(target) = self.world.target
                {
                    return KeyAction::Select(target);
                }
                if let Some(action) = ui.take_status_click(&self.world) {
                    return action;
                }
                KeyAction::Consumed
            }
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                let clicked = ui.mouse_up_in(&self.world).and_then(|action| {
                    if killer_ui::html::split_message(&action).0.is_some() {
                        ui.html_confirm(self.tables.as_deref(), &action, Instant::now());
                        return None;
                    }
                    ui2d::html_action(&action)
                });
                let chat_filter = ui.chat_filter_released();
                let menu = ui.menu_released(&self.world);
                let warning = ui.warning_released();
                let dialog = ui.dialog_released();
                let warning_exit = std::mem::take(&mut ui.warning_exit);
                let views = ui2d::world_views(&self.world, self.tables.as_deref(), Instant::now());
                let pet = ui.pet_released(&views);
                let restart = ui.restart_released(&views);
                let party = ui.party_released(&views);
                let board = ui.board_released();
                let captured = std::mem::take(&mut self.ui_captured);
                let action = warning.or(clicked).or(menu).or(dialog).or(pet).or(restart).or(party).or(board);
                if warning_exit {
                    self.quit = true;
                }
                if let Some(text) = chat_filter {
                    self.save_chat_filter(&text);
                }
                action.unwrap_or_else(|| consumed(captured))
            }
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Right, .. } => {
                let Some((x, y)) = self.cursor else {
                    return KeyAction::Ignored;
                };
                let views = ui2d::world_views(&self.world, self.tables.as_deref(), Instant::now());
                ui.party_assist_at(x as i32, y as i32, &views).unwrap_or(KeyAction::Ignored)
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Right, .. } => {
                let Some((x, y)) = self.cursor else {
                    return KeyAction::Ignored;
                };
                consumed(ui.right_click(x as i32, y as i32, &self.world))
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                let state = modifiers.state();
                ui.keys.alt = state.alt_key();
                ui.keys.ctrl = state.control_key();
                ui.keys.shift = state.shift_key();
                ui.shift_changed(state.shift_key());
                KeyAction::Ignored
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let Some((x, y)) = self.cursor else {
                    return KeyAction::Ignored;
                };
                let ticks = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 40.0,
                };
                let lines: Vec<ChatEntry> = self.chat.iter().map(ui2d::chat_entry).collect();
                consumed(ui.wheel_over(x as i32, y as i32, ticks, &lines))
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale = f64::from(self.ui_scale.max(0.01));
                let cursor = ((position.x / scale) as f32, (position.y / scale) as f32);
                self.cursor = Some(cursor);
                consumed(ui.mouse_move(cursor.0 as i32, cursor.1 as i32))
            }
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(code), state, repeat, .. }, .. }
                if ui_keys::hooked_key(*code, ui.chat_focused(), ui.keys.alt).is_some_and(|vk| {
                    let (msg, lparam) = ui_keys::key_message(*code, ui.keys.alt, state.is_pressed(), *repeat);
                    ui.hook_key(msg, vk, lparam, &self.world)
                }) =>
            {
                self.moves = Movement::default();
                KeyAction::Consumed
            }
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state: ElementState::Pressed, .. }, .. } => {
                let mut action = KeyAction::Ignored;
                for key in ui2d::input_keys(logical_key) {
                    match ui.key(key) {
                        KeyAction::Ignored => {}
                        KeyAction::Consumed if action == KeyAction::Ignored => action = KeyAction::Consumed,
                        KeyAction::Consumed => {}
                        other => action = other,
                    }
                }
                if action != KeyAction::Ignored {
                    self.moves = Movement::default();
                }
                action
            }
            WindowEvent::KeyboardInput { .. } => consumed(ui.chat_focused()),
            _ => KeyAction::Ignored,
        }
    }

    fn lobby_c4_active(&self) -> bool {
        Self::c4_ready(self.screen, self.modern_hud.preferences.enabled, self.tables.is_some(), self.game_ui.is_some(), self.lobby.view.is_some())
    }

    fn splash_finished(&self) -> bool {
        let interface_ready = self.lobby.settled() && (self.modern_hud.preferences.enabled || self.tables.is_some() && self.game_ui.is_some()) || self.screen == Screen::World && self.world_view.is_some();
        let failed = self.lobby.load_error.is_some() || self.tables_error.is_some() || self.ui_error.is_none() && self.load_error.is_some();
        self.stage.is_some() && (interface_ready || failed || self.started.elapsed() > SPLASH_TIMEOUT)
    }

    fn c4_ready(screen: Screen, modern: bool, tables: bool, ui: bool, view: bool) -> bool {
        matches!(screen, Screen::Login | Screen::Servers | Screen::Characters | Screen::Create) && !modern && tables && ui && view
    }

    fn lobby_option_event(&mut self, event: &WindowEvent) -> bool {
        let Some(ui) = self.game_ui.as_mut().filter(|ui| ui.option_open()) else { return false };
        let over = |ui: &GameUi, (x, y): (f32, f32)| ui.desktop.window(killer_ui::window::WindowId::OPTION).is_some_and(|window| window.rect.contains(x as i32, y as i32));
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let scale = f64::from(self.ui_scale.max(0.01));
                let cursor = ((position.x / scale) as f32, (position.y / scale) as f32);
                self.cursor = Some(cursor);
                let dragging = ui.mouse_move(cursor.0 as i32, cursor.1 as i32);
                dragging || self.ui_captured
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                let Some(cursor) = self.cursor.filter(|cursor| over(ui, *cursor)) else { return false };
                if ui.mouse_down_at(cursor.0 as i32, cursor.1 as i32, Instant::now(), &self.world) {
                    self.ui_captured = true;
                    return true;
                }
                false
            }
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                if !std::mem::take(&mut self.ui_captured) {
                    return false;
                }
                ui.mouse_up_in(&self.world);
                self.run_ui_commands();
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let Some(cursor) = self.cursor.filter(|cursor| over(ui, *cursor)) else { return false };
                let ticks = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 40.0,
                };
                ui.wheel_over(cursor.0 as i32, cursor.1 as i32, ticks, &[])
            }
            _ => false,
        }
    }

    fn toggle_lobby_options(&mut self) {
        let Some(ui) = self.game_ui.as_mut() else { return };
        let show = !ui.option_open();
        if let Some(window) = ui.desktop.window_mut(killer_ui::window::WindowId::OPTION) {
            window.visible = show;
        }
        self.play_interface_sound(if show { "itemsound.window_open" } else { "itemsound.window_close" });
        if show && let Err(error) = self.initialize_video_options() {
            self.settings_error(error);
        }
    }

    fn sync_lobby_options(&mut self) {
        let Some(ui) = self.game_ui.as_mut().filter(|ui| ui.option_open()) else { return };
        if !ui.option_synced
            && !ui.keys.option_applying
            && let Some(stage) = &self.stage
            && let Some(runtime) = &mut self.video_options
        {
            if !runtime.display.is_pending()
                && let Some(display) = crate::ui_video::VideoDisplay::capture(&stage.window, &stage.gpu.config)
            {
                runtime.display = display;
            }
            let multisample = killer_render::video::multisample_capability(&stage.gpu.adapter, stage.renderer.device());
            runtime.show(&mut ui.keys.option.video_page, multisample, |id| ui.keys.sysstrings.get(&id).cloned());
        }
        ui.sync_option_window();
        self.run_ui_commands();
    }

    fn lobby_c4_event(&mut self, event: &WindowEvent) -> bool {
        if !self.lobby_c4_active() {
            return false;
        }
        if self.lobby_option_event(event) {
            return true;
        }
        let (Some(c4), Some(ui)) = (self.lobby_c4.as_mut(), self.game_ui.as_ref()) else { return false };
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let scale = f64::from(self.ui_scale.max(0.01));
                let mut cursor = ((position.x / scale) as f32, (position.y / scale) as f32);
                if cfg!(any(target_os = "ios", target_os = "android")) {
                    let factor = self.stage.as_ref().map_or(1.0, |stage| stage.window.scale_factor() as f32);
                    let radius = (TOUCH_RADIUS_POINTS * factor / self.ui_scale.max(0.01)).round() as i32;
                    let (x, y) = c4.snap(self.lobby.screen, cursor.0 as i32, cursor.1 as i32, radius);
                    cursor = (x as f32, y as f32);
                }
                self.cursor = Some(cursor);
                c4.mouse_move(cursor.0 as i32, cursor.1 as i32, &self.lobby);
                true
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                let Some((x, y)) = self.cursor else { return true };
                if state.is_pressed() {
                    c4.mouse_down(x as i32, y as i32, &mut self.lobby, &ui.small);
                    for name in std::mem::take(&mut c4.sounds) {
                        self.play_interface_sound(name);
                    }
                } else {
                    c4.mouse_up(x as i32, y as i32, &mut self.lobby, &ui.small);
                }
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let ticks = match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 40.0,
                };
                c4.wheel(ticks, &self.lobby, &ui.small);
                true
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.lobby_shift = modifiers.state().shift_key();
                false
            }
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state: ElementState::Pressed, .. }, .. } => {
                let shift = self.lobby_shift;
                if matches!(logical_key, Key::Named(NamedKey::Tab)) {
                    c4.key(InputKey::Char('\t'), shift, &mut self.lobby, &ui.small);
                } else {
                    for key in ui2d::input_keys(logical_key) {
                        c4.key(key, shift, &mut self.lobby, &ui.small);
                    }
                }
                true
            }
            WindowEvent::KeyboardInput { .. } => true,
            _ => false,
        }
    }

    fn dispatch(&mut self, event: WindowEvent) {
        if self.lobby_c4_event(&event) {
            return;
        }
        if let WindowEvent::MouseInput { state, button, .. } = &event {
            log::debug!("mouse {button:?} {state:?} em {:?}", self.cursor);
        }
        let released = matches!(event, WindowEvent::MouseInput { state: ElementState::Released, .. });
        if released {
            self.dragging = false;
        }
        let action = self.game_ui_event(&event);
        self.run_ui_commands();
        match action {
            KeyAction::Ignored => {}
            KeyAction::Consumed => return,
            KeyAction::Say(text) => {
                self.say(text);
                return;
            }
            KeyAction::Bypass(command) => {
                log::debug!("bypass {command}");
                self.send(NetCommand::Bypass { command });
                return;
            }
            KeyAction::LinkHtml(link) => {
                log::debug!("link {link}");
                self.send(NetCommand::LinkHtml { link });
                return;
            }
            KeyAction::Dialog(answer, accept) => {
                self.answer_dialog(answer, accept);
                return;
            }
            KeyAction::RestartPoint(point) => {
                self.send(NetCommand::RestartPoint { point });
                return;
            }
            KeyAction::Select(object_id) => {
                self.select_target(object_id);
                return;
            }
            KeyAction::Assist(member) => {
                self.assist_member(member);
                return;
            }
        }
        let consumed = self.stage.as_mut().is_some_and(|stage| {
            let response = stage.egui.state.on_window_event(&stage.window, &event);
            egui_consumes(&stage.egui.ctx, &event, response.consumed)
        });
        if !consumed {
            self.input(event);
        } else if released {
            self.click.cancel();
        } else if matches!(event, WindowEvent::KeyboardInput { .. }) {
            self.moves = Movement::default();
        } else if let WindowEvent::CursorMoved { position, .. } = event {
            let scale = f64::from(self.ui_scale.max(0.01));
            self.cursor = Some(((position.x / scale) as f32, (position.y / scale) as f32));
        }
    }

    fn run_ui_commands(&mut self) {
        let commands = self.game_ui.as_mut().map(GameUi::take_commands).unwrap_or_default();
        for command in commands {
            match command {
                UiCommand::Net(command) => {
                    log::debug!("janela: {command:?}");
                    self.send(command);
                }
                UiCommand::CrestFile { kind, path } => {
                    let wanted = Path::new(&path);
                    let full = if wanted.is_absolute() { wanted.to_path_buf() } else { self.client_root.join("system").join(wanted) };
                    let bytes = std::fs::read(&full).ok();
                    if let Some(ui) = &mut self.game_ui {
                        ui.keys.crest_file(kind, bytes.as_deref());
                    }
                }
                UiCommand::ServerTransfer { id } => self.begin_server_transfer(id),
                UiCommand::ToggleHelp { macro_help } => {
                    let path = if macro_help { crate::ui_help::MACRO_FILE } else { crate::ui_help::HELP_FILE };
                    let shown = self.game_ui.as_ref().is_some_and(|ui| ui.desktop.window(killer_ui::window::WindowId::HELP).is_some_and(|w| w.visible));
                    let text = if shown { Ok(String::new()) } else {
                        open_uploader(&mut self.uploader, &self.client_root).map_err(|e| e.to_string())
                            .and_then(|client| client.read_help_text(path).map_err(|e| e.to_string()))
                    };
                    match text {
                        Ok(text) => if let Some(ui) = &mut self.game_ui { { let tables = self.tables.clone(); ui.help.toggle(&mut ui.desktop, &ui.small, path, &text, &|marker, id| ui2d::html_marker(tables.as_deref(), marker, id)); } },
                        Err(error) => log::warn!("ajuda: {path}: {error}"),
                    }
                }
                UiCommand::LoadHelp(path) => {
                    let text = open_uploader(&mut self.uploader, &self.client_root).map_err(|e| e.to_string())
                        .and_then(|client| client.read_help_text(&path).map_err(|e| e.to_string()));
                    match text {
                        Ok(text) => if let Some(ui) = &mut self.game_ui { { let tables = self.tables.clone(); ui.help.load(&mut ui.desktop, &ui.small, &text, &|marker, id| ui2d::html_marker(tables.as_deref(), marker, id)); } },
                        Err(error) => log::warn!("ajuda: {path}: {error}"),
                    }
                }
                UiCommand::GmEventFile { file, commands } => {
                    let text = open_uploader(&mut self.uploader, &self.client_root).map_err(|e| e.to_string())
                        .and_then(|client| client.read_system_text(&file).map_err(|e| e.to_string()));
                    match text {
                        Ok(text) => {
                            let ini = killer_ui::ini::parse_ini(&text);
                            if let Some(ui) = &mut self.game_ui {
                                if commands {
                                    let html = killer_ui::gm_event::command_html(&ini);
                                    ui.show_html("", &html);
                                } else {
                                    let actions = match killer_ui::gm_event::read_options(&ini) {
                                        Ok(options) => ui.keys.gm_event.start_race(options),
                                        Err(error) => vec![error],
                                    };
                                    ui.gm_event_actions(actions, &self.world);
                                }
                            }
                        }
                        Err(error) => log::warn!("event match: {file}: {error}"),
                    }
                }
                UiCommand::ReplayRecord(command) => self.replay_record(command),
                UiCommand::DebugCommand { window } => {
                    if let Some(ui) = &mut self.game_ui {
                        ui.keys.debug_command(window, &mut ui.desktop);
                    }
                }
                UiCommand::OpenPetition(text) => {
                    if let Some(ui) = &mut self.game_ui {
                        let preset = text.as_deref().map(|text| (text, &ui.small));
                        ui.keys.petition.open_begin(preset, &mut ui.desktop);
                    }
                }
                UiCommand::LoadPetitionPage(path) => {
                    let text = open_uploader(&mut self.uploader, &self.client_root).map_err(|e| e.to_string())
                        .and_then(|client| client.read_help_text(&path).map_err(|e| e.to_string()));
                    match text {
                        Ok(text) => if let Some(ui) = &mut self.game_ui { { let tables = self.tables.clone(); ui.keys.petition.begin.set_page_with(&text, &ui.small, &|marker, id| ui2d::html_marker(tables.as_deref(), marker, id)); } },
                        Err(error) => log::warn!("petição: {path}: {error}"),
                    }
                }
                UiCommand::Log(message) => log::info!("{message}"),
                UiCommand::InterfaceSound(name) => self.play_interface_sound(name),
                UiCommand::SystemMessage(id) => {
                    let text = ui2d::system_text(self.tables.as_deref(), id, &[]);
                    self.chat.push(ChatLine::new(CHAT_SYSTEM, String::new(), text));
                }
                UiCommand::WarningMessage { id, numbers } => {
                    let args: Vec<_> = numbers.into_iter().map(l2_net::game::SystemMessageArg::Number).collect();
                    let text = ui2d::system_text(self.tables.as_deref(), id, &args);
                    if let Some(ui) = self.game_ui.as_mut() {
                        ui.show_warning(&text, false);
                    }
                }
                UiCommand::ColoredSystemMessage { id, color } => {
                    let text = ui2d::system_text(self.tables.as_deref(), id, &[]);
                    let mut line = ChatLine::new(CHAT_SYSTEM, String::new(), text);
                    line.color = Some(color);
                    self.chat.push(line);
                }
                UiCommand::DeleteLandMark => self.landmark.delete(false),
                UiCommand::ConfirmDeleteMacro { id, name } => self.world.dialogs.push(killer_world::Dialog::DeleteMacro { id, name }),
                UiCommand::ItemDrop { object_id, item_id, count, name, target } => self.item_drop(object_id, item_id, count, name, target),
                UiCommand::ConfirmDeleteRecipe { recipe_id, name } => self.world.dialogs.push(killer_world::Dialog::DeleteRecipe { recipe_id, name }),
                UiCommand::OpenPetNameDialog => self.world.dialogs.push(killer_world::Dialog::PetName),
                UiCommand::ApplyOptions { close } => self.begin_video_apply(Some(close)),
                UiCommand::InitializeInterface => {
                    if let Some(ui) = &mut self.game_ui
                        && let Err(error) = crate::ui_initialize_run::initialize(ui)
                    {
                        self.settings_error(format!("Initialize: {error}"));
                    }
                }
                UiCommand::OptionResolutionChanged(selection) => {
                    if let (Some(runtime), Some(ui)) = (&self.video_options, &mut self.game_ui) {
                        runtime.resolution_changed(&mut ui.keys.option.video_page, selection);
                    }
                }
                UiCommand::OptionKeepMinFrameRateChanged => {
                    if let (Some(runtime), Some(ui)) = (&self.video_options, &mut self.game_ui) {
                        let capabilities = runtime.capabilities;
                        ui.keys.option.video_page.keep_min_frame_changed(capabilities.water, capabilities.shaders);
                    }
                }
                UiCommand::ToggleMinFrameRate => self.toggle_min_frame_rate(),
            }
        }
    }

    fn tick_audio(&mut self, dt: f32) {
        if self.audio.is_none() {
            self.audio = Some(self.open_audio());
        }
        let in_world = self.screen == Screen::World;
        let location = if in_world {
            self.world.me.as_ref().map(|me| [me.position.x, me.position.y, me.position.z])
        } else {
            self.lobby.ready().then(|| self.lobby.camera().eye.to_array())
        };
        let frame = location.map(|view_actor_location| crate::ui_audio::AudioFrame {
            time: self.started.elapsed().as_secs_f64(),
            view_actor_location,
            max_radius_multiplier: crate::ui_audio_host::MAX_RADIUS_MULTIPLIER,
            realtime: false,
            replay_recording: false,
        });
        let state = if in_world {
            6
        } else {
            match self.lobby.screen {
                crate::lobby::LobbyScreen::Login | crate::lobby::LobbyScreen::Servers => 2,
                crate::lobby::LobbyScreen::Characters => 4,
                crate::lobby::LobbyScreen::Create => 3,
            }
        };
        let credits = self.lobby_c4.as_ref().is_some_and(|c4| c4.credits_visible());
        let roll = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.subsec_nanos());
        let Some(host) = &mut self.audio else { return };
        let camera_to_world = if in_world { self.world_view.as_ref().map(WorldView::current) } else { Some(self.lobby.camera()) }
            .map_or(Mat4::IDENTITY,|camera| camera.view().inverse());
        if let Some(device) = &mut host.device
            && let Err(error) = device.update_spatial(frame,camera_to_world,&{ let mut positions = self.items.audio_positions(&self.world); positions.extend(crate::world_sound::pawn_positions(&self.world)); positions }) {
                log::warn!("áudio espacial: {error}");
            }
        host.update(frame);
        if frame.is_some() && !host.intro_started {
            host.intro_started = true;
            host.play_intro_music(&self.client_root);
        }
        if host.console.state_38 != state {
            host.set_console_state(&self.client_root, state);
        }
        host.set_credits(&self.client_root, credits, roll);
        if state == 3
            && let Some(race) = self.lobby.create.race
            && host.console.create_44 != race as i32
        {
            host.set_current_making_race(&self.client_root, race as i32);
        }
        if in_world {
            if std::mem::take(&mut self.world.music_stop) {
                host.stop_all_music();
            }
            if std::mem::take(&mut self.world.music_reset) {
                host.reset_music();
            }
            let pawn = self.world.me.as_ref().map(|me| crate::zone_music::Pawn {
                location: [me.position.x, me.position.y, me.position.z],
                zone: self.music_zone.clone(),
                on_vehicle: me.vehicle.is_some(),
            });
            let titles = self.tables.as_ref().map(|tables| &tables.musicinfo);
            let today = crate::zone_music::civil_date(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs()));
            let cache = &mut self.cache;
            host.zone.set_vehicle_status(pawn.as_ref().is_some_and(|pawn| pawn.on_vehicle));
            host.tick_world_music(&self.client_root, dt, pawn.as_ref(), |id| {
                let empty = std::collections::HashMap::new();
                crate::zone_music::MusicNames { titles: titles.unwrap_or(&empty), today }.get(id, || cache.crt_rand())
            });
        }
        if let Some(ui) = &mut self.game_ui {
            ui.audio_volumes = host.volumes();
        }
    }

    fn open_audio(&mut self) -> crate::ui_audio_host::AudioHost {
        use crate::ui_audio_host::{AudioHost, AudioIni, LobbyConsole, channels, lobby_sound_names};
        let read = |client: &mut Client, name: &str| client.read_system_text(name).map(|text| parse_ini(&text)).unwrap_or_else(|error| {
            log::warn!("áudio: {name}: {error}");
            parse_ini("")
        });
        let (option, alaudio, system, voice_suffix) = match open_uploader(&mut self.uploader, &self.client_root) {
            Ok(client) => (read(client, OPTION_INI), read(client, "alaudio.int"), read(client, L2_INI), client.client_language().voice_suffix()),
            Err(error) => {
                log::warn!("áudio: cliente indisponível: {error}");
                (parse_ini(""), parse_ini(""), parse_ini(""), String::new())
            }
        };
        let music_times = crate::zone_music::MusicTimes::read(|section, key| alaudio.get(section, key).map(str::to_owned));
        self.cache.set_sound_config(crate::world_sound::SoundConfig::read(&alaudio));
        let mut device = None;
        log::info!("áudio: Channels={} volumes do option.ini {:?}", channels(&system), crate::ui_audio::InitConfig::read(&AudioIni { option: &option, alaudio: &alaudio }).volumes);
        if let Err(error) = crate::ui_audio::AudioDevice::init(&mut device, &AudioIni { option: &option, alaudio: &alaudio }, channels(&system)) {
            log::warn!("áudio: subsistema não iniciado: {error}");
        }
        if let Some(device) = &mut device
            && let Err(error) = crate::ui_audio::spatial::Config::read(&system).and_then(|config| device.configure_spatial(config)) {
                log::warn!("áudio espacial não iniciado: {error}");
            }
        AudioHost {
            device,
            console: LobbyConsole::new(lobby_sound_names(&alaudio)).with_credit_names(crate::ui_audio_host::credit_sound_names(&alaudio)),
            controller: crate::ui_audio::MusicController::Lineage { field_a34: 0, flags_a38: 0 },
            frame: None,
            intro_started: false,
            zone: crate::zone_music::ZoneMusic::default(),
            music_times,
            voice_suffix,
            wav_voices: std::collections::HashMap::new(),
        }
    }

    fn tick(&mut self) -> f32 {
        let now = Instant::now();
        self.pump_replay();
        if self.screen != Screen::World {
            if self.lobby_c4.as_mut().is_some_and(|c4| c4.take_options_close())
                && let Some(window) = self.game_ui.as_mut().and_then(|ui| ui.desktop.window_mut(killer_ui::window::WindowId::OPTION))
            {
                window.visible = false;
            }
            if self.lobby_c4.as_mut().is_some_and(|c4| c4.take_options_request()) {
                self.toggle_lobby_options();
            }
            let requests = self.lobby_c4.as_mut().map(|c4| c4.take_replay_requests()).unwrap_or_default();
            self.run_replay_requests(requests);
            self.sync_lobby_options();
        }
        let elapsed = now - self.last_frame;
        self.last_frame = now;
        self.tick_audio(elapsed.as_secs_f32());
        if self.screen != Screen::World {
            self.lobby.tick(elapsed);
            for name in std::mem::take(&mut self.lobby.voices) {
                self.play_wav_voice(name);
            }
            if std::mem::take(&mut self.lobby.stop_voices)
                && let Some(host) = &mut self.audio
            {
                host.stop_all_voice(killer_ui::lobby_select::VOICE_FADE, false);
            }
        }
        let before = self.walk.snapshot(&self.world);
        let me_before = self.world.me.as_ref().map(|me| me.position);
        if let Some(view) = self.world_view.as_ref() {
            self.world.pawn_clipping = view.display.pawn_clipping;
            self.world.keep_min_frame_rate = view.options.keep_min_frame_rate;
        }
        self.world.teleport_fades = true;
        let dt = pace::run(&mut self.world, elapsed);
        let me_tile_ready = self.pending.is_none() && self.world.me.as_ref().is_none_or(|me| {
            let tile = tile_name(me.position.x, me.position.y);
            self.tile.as_deref() == Some(tile.as_str()) || self.world_view.as_ref().is_some_and(|view| TileId::parse(&tile).is_some_and(|id| view.neighbours.is_loaded(id)))
        });
        if let (Some(view), Some(stage), true) = (self.world_view.as_ref(), self.stage.as_ref(), me_tile_ready) {
            let origin = self.world.me.as_ref().map_or(Vec3::ZERO, |me| me.position);
            let views = crate::items::views(view, Some(&stage.renderer), origin);
            let spawned = self.fences.colliders(&self.world);
            self.walk.apply(&mut self.world, &before, &views, &spawned, dt, &view.options);
            self.music_zone = self.walk.music_zone(&views, origin);
            let calls = self.walk.take_calls();
            self.apply_controller_calls(calls);
            self.cache.set_foot_volumes(self.walk.foot_volumes());
            self.cache.set_swim_ticks(self.walk.swim_ticks());
        }
        self.profile.lap("física", Instant::now());
        if let Some(stage) = self.stage.as_mut() {
            stage.renderer.set_movers_auto_tick(false);
            stage.renderer.tick_movers(dt);
            if let Some(view) = self.world_view.as_mut() { view.moving_trace.sync(&stage.renderer); }
        }
        let instant = 1.0 / dt;
        self.fps = if self.fps == 0.0 { instant } else { self.fps * 0.9 + instant * 0.1 };
        if let Some(report) = self.ignored.due(now) {
            log::info!("{report}");
        }
        if now.duration_since(self.banks_trimmed) >= BANK_TRIM_INTERVAL {
            self.banks_trimmed = now;
            let dropped = self.ui_bank.trim_over(crate::pawn_effects::BANK_TRIM_BYTES) + self.effects.trim_banks();
            if dropped > 0 {
                log::info!("bancos de textura: {dropped} cópias de CPU soltas");
            }
        }
        let walking = pace::moved(me_before, self.world.me.as_ref().map(|me| me.position));
        if self.validate.poll(dt, walking)
            && let Some(me) = &self.world.me
        {
            self.send(NetCommand::Validate { position: coords(me.position), heading: me.heading });
        }
        if let Some(view) = self.world_view.as_mut() {
            if view.use_free {
                view.free.advance(&self.moves, dt);
            } else {
                view.camera.player_tick(dt);
                if let Some(ui) = &self.game_ui {
                    view.camera.settings.auto_tracking = ui.options.auto_tracking_pawn;
                }
                if let Some(me) = &self.world.me {
                    view.camera.follow(me.position);
                }
                view.camera.camera_commands(&mut self.world, dt);
                let origin = self.world.me.as_ref().map_or(Vec3::ZERO, |me| me.position);
                let views = self.stage.as_ref().map(|stage| crate::items::views(&*view, Some(&stage.renderer), origin)).unwrap_or_default();
                let extent = if view.camera.settings.hit_check && !view.camera.settings.volume_camera { 0.1 } else { 0.0 };
                let spawned = self.fences.colliders(&self.world);
                let walk = std::cell::RefCell::new(&mut self.walk);
                view.camera.collide(|from, to| walk.borrow_mut().camera_hit(&views, &spawned, from, to, extent).flatten());
            }
        }
        dt
    }

    fn game_hour(&self, now: Instant) -> f32 {
        self.hour.or_else(|| self.world.clock.map(|clock| clock.hour(now))).unwrap_or(DEFAULT_HOUR)
    }

    pub fn configure_hud(&mut self, style: Option<&str>) {
        self.modern_hud.configure(style);
        if self.modern_hud.preferences.enabled { self.hud=HudMode::Hidden; }
    }

    pub fn configure_download(&mut self, url: Option<String>, token: Option<String>) {
        let Some(url) = url.filter(|url| !url.trim().is_empty()) else { return };
        self.download = Some(crate::download::Download::start(url.clone(), token.clone(), self.client_root.clone()));
        self.download_source = Some((url, token));
    }

    fn downloading(&self) -> bool {
        self.download.as_ref().is_some_and(|download| download.snapshot().phase != crate::download::Phase::Done)
    }

    fn apply_ui_scale(&mut self) {
        let Some(stage) = self.stage.as_mut() else { return };
        let size = crate::gpu::surface_size(&stage.window);
        let next = ui_scale_for(&stage.window);
        let key = (size.width, size.height, next.to_bits());
        if self.applied_ui == Some(key) && stage.gpu.size() == (size.width.max(1), size.height.max(1)) {
            return;
        }
        self.applied_ui = Some(key);
        if stage.gpu.size() != (size.width.max(1), size.height.max(1)) {
            stage.gpu.resize(size.width, size.height);
        }
        self.ui_scale = next;
        let scale = self.ui_scale.max(0.01);
        let logical = (((size.width as f32 / scale).round() as u32).max(1), ((size.height as f32 / scale).round() as u32).max(1));
        stage.renderer.set_ui_size(Some(logical));
        if let Some(ui) = self.game_ui.as_mut() {
            ui.resized((logical.0 as i32, logical.1 as i32));
        }
    }

    fn frame(&mut self) {
        self.apply_ui_scale();
        if std::mem::take(&mut self.first_frame) {
            log::info!("primeiro quadro {:.1?} depois do início", self.started.elapsed());
        }
        self.profile.begin(Instant::now());
        self.poll_net();
        self.poll_loaded();
        self.poll_neighbours();
        self.advance_neighbours();
        self.update_tile_visibility();
        let downloading = self.downloading();
        if self.screen != Screen::World && !downloading {
            self.start_lobby();
            self.poll_lobby();
        }
        self.poll_characters();
        self.profile.lap("poll", Instant::now());
        let dt = self.tick();
        let held = self.binds.hold(dt);
        self.exec_engine_commands(held, killer_world::engine_input::InputAction::Hold, dt);
        self.profile.lap("tick", Instant::now());
        self.drive_joystick();
        self.tick_dialogs();
        self.animate(dt);
        self.request_hover();
        self.animate_lobby(dt);
        self.request_characters();
        let hour = self.game_hour(Instant::now());
        if let Some(view) = self.world_view.as_mut() {
            view.weather.state.update(hour);
        }
        self.update_lens_flare(hour, dt);
        // O `sync_windows` faz o macro em andamento avançar, e o que ele emite precisa ser
        // drenado aqui: fora daqui, `run_ui_commands` só roda em evento de janela, então os
        // comandos ficavam parados até o usuário mexer o mouse ou apertar uma tecla.
        if self.screen == Screen::World
            && self.world_view.is_some()
            && let Some(ui) = self.game_ui.as_mut()
        {
            if !ui.option_synced && !ui.keys.option_applying && ui.desktop.window(killer_ui::window::WindowId::OPTION).is_some_and(|window| window.visible)
                && let Some(stage) = &self.stage
                && let Some(runtime) = &mut self.video_options
            {
                    if !runtime.display.is_pending()
                        && let Some(display) = crate::ui_video::VideoDisplay::capture(&stage.window, &stage.gpu.config) {
                        runtime.display = display;
                    }
                    let multisample = killer_render::video::multisample_capability(&stage.gpu.adapter, stage.renderer.device());
                    runtime.show(&mut ui.keys.option.video_page, multisample, |id| ui.keys.sysstrings.get(&id).cloned());
            }
            self.modern_hud.sync(ui);
            ui.ingest_chat(&self.chat);
            if let Some(stamp) = capture_stamp()
                && let Some((name, bytes)) = ui.tick_text_capture(stamp)
                && let Err(error) = killer_platform::fs::write(self.client_root.join("system").join(&name), bytes)
            {
                log::warn!("captura de texto: falha ao gravar {name}: {error}");
            }
            ui.sync_windows(&self.world, self.tables.as_deref(), Instant::now());
            let wanted = ui.take_zone_texture_requests();
            if !wanted.is_empty()
                && let Ok(client) = open_uploader(&mut self.uploader, &self.client_root)
            {
                for cell in wanted {
                    let texture = client.catalog().texture_raw(&killer_ui::zone_title::zone_texture_name(cell)).and_then(|raw| {
                        let mip = raw.mips.into_iter().next()?;
                        Some(killer_ui::zone_title::ZoneTexture { width: mip.width as i32, height: mip.height as i32, pixels: mip.data })
                    });
                    ui.set_zone_texture(cell, texture);
                }
            }
        }
        self.run_ui_commands();
        let Some(stage) = self.stage.as_mut() else {
            return;
        };
        let surface = match stage.gpu.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                stage.gpu.reconfigure();
                return;
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                std::thread::sleep(OCCLUDED_PAUSE);
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                log::error!("superfície inválida");
                return;
            }
        };
        self.profile.lap("superfície", Instant::now());
        let color = surface.texture.create_view(&Default::default());
        let depth = stage.gpu.depth_view();
        let size = stage.gpu.size();
        let device = &stage.gpu.device;
        let queue = &stage.gpu.queue;
        let mut encoder = device.create_command_encoder(&Default::default());
        let (camera, environment) = match (&self.world_view, self.screen != Screen::World, self.lobby.view.as_ref()) {
            (Some(view), _, _) => (view.current(), Environment::from_weather(&view.weather, hour)),
            (None, true, Some(lobby)) => (
                self.lobby.camera(),
                Environment::from_time(&lobby.timeenv, self.hour.unwrap_or_else(|| self.lobby.hour())),
            ),
            (None, _, _) => (OrbitCamera::default().camera(), Environment::default()),
        };
        // Os rótulos entram na mesma `DrawList` da interface, então vivem no espaço dela.
        if cfg!(any(target_os = "ios", target_os = "android")) {
            stage.egui.ctx.set_zoom_factor(self.ui_scale / stage.window.scale_factor() as f32);
        }
        let input=stage.egui.state.take_egui_input(&stage.window);
        stage.egui.ctx.begin_pass(input);
        self.joystick.set_scale(stage.window.scale_factor());
        if cfg!(any(target_os = "ios", target_os = "android")) && self.joystick.knob().is_none() && self.screen == Screen::World && self.world_view.is_some() {
            let per_point = 1.0 / f64::from(stage.egui.ctx.pixels_per_point());
            let radius = (self.joystick.radius() * per_point) as f32;
            let center = egui::pos2((f64::from(size.0) * 0.16 * per_point) as f32, (f64::from(size.1) * 0.74 * per_point) as f32);
            let painter = stage.egui.ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("joystick")));
            painter.circle(center, radius, egui::Color32::from_rgba_unmultiplied(20, 24, 32, 45), egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(220, 230, 255, 90)));
            painter.circle(center, radius * 0.4, egui::Color32::from_rgba_unmultiplied(235, 242, 255, 70), egui::Stroke::NONE);
        }
        if cfg!(any(target_os = "ios", target_os = "android")) && self.screen == Screen::World && self.world_view.is_some() {
            let per_point = 1.0 / f64::from(stage.egui.ctx.pixels_per_point());
            let screen = (f64::from(size.0), f64::from(size.1));
            let scale = stage.window.scale_factor();
            let painter = stage.egui.ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("botões de toque")));
            for button in crate::touch_hud::Button::ALL {
                let circle = crate::touch_hud::circle(button, screen, scale);
                let held = self.touch_hud.pressed() == Some(button);
                let (fill, edge) = match button {
                    crate::touch_hud::Button::Attack => ((205, 70, 45), (255, 205, 160)),
                    crate::touch_hud::Button::Target => ((55, 95, 175), (170, 205, 255)),
                    crate::touch_hud::Button::Pickup => ((55, 140, 85), (175, 235, 195)),
                };
                let alpha = if held { 235 } else { 175 };
                let center = egui::pos2((circle.center.x * per_point) as f32, (circle.center.y * per_point) as f32);
                let radius = (circle.radius * per_point) as f32 * if held { 0.94 } else { 1.0 };
                painter.circle(center, radius, egui::Color32::from_rgba_unmultiplied(fill.0, fill.1, fill.2, alpha), egui::Stroke::new(3.0, egui::Color32::from_rgba_unmultiplied(edge.0, edge.1, edge.2, 230)));
                let size_text = if button == crate::touch_hud::Button::Attack { 15.0 } else { 11.0 };
                painter.text(center, egui::Align2::CENTER_CENTER, button.label(), egui::FontId::proportional(size_text), egui::Color32::WHITE);
            }
        }
        if let Some(knob) = self.joystick.knob() {
            let per_point = 1.0 / f64::from(stage.egui.ctx.pixels_per_point());
            let at = |p: winit::dpi::PhysicalPosition<f64>| egui::pos2((p.x * per_point) as f32, (p.y * per_point) as f32);
            let painter = stage.egui.ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("joystick")));
            let radius = (knob.radius * per_point) as f32;
            painter.circle(at(knob.origin), radius, egui::Color32::from_rgba_unmultiplied(20, 24, 32, 90), egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(220, 230, 255, 150)));
            painter.circle(at(knob.origin), radius * 0.38, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 25), egui::Stroke::NONE);
            painter.circle(at(knob.current), radius * 0.46, egui::Color32::from_rgba_unmultiplied(235, 242, 255, 190), egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(120, 150, 220, 220)));
        }
        let mut retry = false;
        if let Some(download) = &self.download {
            let snapshot = download.snapshot();
            if snapshot.phase != crate::download::Phase::Done {
                retry = download_panel(&stage.egui.ctx, &snapshot);
            }
        }
        if retry && let Some((url, token)) = self.download_source.clone() {
            self.download = Some(crate::download::Download::start(url, token, self.client_root.clone()));
        }
        let ui_space = {
            let scale = self.ui_scale.max(0.01);
            (size.0 as f32 / scale, size.1 as f32 / scale)
        };
        let in_world = self.screen == Screen::World && self.world_view.is_some();
        if let Some(ui) = self.game_ui.as_mut() {
            ui.desktop.raise_newly_shown();
        }
        let lobby_c4 = Self::c4_ready(self.screen, self.modern_hud.preferences.enabled, self.tables.is_some(), self.game_ui.is_some(), self.lobby.view.is_some());
        stage.renderer.set_radar(Vec::new());
        match (in_world, &self.game_ui) {
            (true, Some(ui)) if self.pending.is_some() => {
                match open_uploader(&mut self.uploader, &self.client_root) {
                    Ok(client) => stage.renderer.set_ui(&ui.loading_frame(), &mut BankSource { bank: &mut self.ui_bank, catalog: client.catalog() }),
                    Err(e) => log::error!("cliente: {e}"),
                }
            }
            (true, Some(ui)) => {
                let frame = self.world_view.as_ref().map(|view| Frame { display: view.display, viewer: viewer_position(&self.world, &view.camera), dt: 0.0 });
                let mut labels = self
                    .tables
                    .as_ref()
                    .zip(frame)
                    .map(|(tables, frame)| characters::labels(&self.world, tables, &self.cache, &frame, &camera, ui_space, self.hovered.filter(|hovered| hovered.slot.is_none()).map(|hovered| hovered.object_id)))
                    .unwrap_or_default();
                let lines: Vec<ChatEntry> = self.chat.iter().map(ui2d::chat_entry).collect();
                let target = self.world.targeted().map(|entity| {
                    let name = self.tables.as_ref().map_or_else(|| entity.name.clone(), |tables| characters::display_name(entity, tables));
                    ui2d::target_view(&self.world, entity, name, self.tables.as_deref())
                }).or_else(|| ui2d::static_target_view(&self.world, self.tables.as_deref()));
                self.profile.lap("rótulos", Instant::now());
                let mut views = ui2d::world_views(&self.world, self.tables.as_deref(), Instant::now());
                if let Some(view) = self.world_view.as_ref() {
                    views.radar.camera_yaw = view.camera.facing();
                    views.radar.camera_pitch = -(view.camera.pitch / TAU * UE_UNITS_PER_TURN) as i32;
                }
                views.primitives = crate::server_objects::marks(&self.world,|point| characters::project(&camera,ui_space,point));
                views.item_label = item_label(Hover { hovered:self.hovered,cursor:self.cursor,items:&self.items,world:&self.world,tables:self.tables.as_deref() },ui,&camera,ui_space);
                self.modern_hud.capture_windows(ui,&self.world,&lines,target.as_ref(),&views);
                crate::ui_modern::ModernHud::measure_labels(&stage.egui.ctx,ui,&mut labels,self.ui_scale);
                self.world.fishing_view.refresh(size.0 as i32);
                if self.world.fishing_view.param.is_some() {
                    if self.fishing_target.is_none() {
                        let (width, height) = killer_world::fishing_view::VIEWPORT_SIZE;
                        let target = stage.renderer.viewport_target((width as u32, height as u32));
                        stage.renderer.set_ui_external_texture(killer_ui::fishing::VIEWPORT_TEXTURE, target.view(), [width as f32, height as f32]);
                        self.fishing_target = Some(target);
                    }
                } else if self.fishing_target.take().is_some() {
                    stage.renderer.clear_ui_external_texture(killer_ui::fishing::VIEWPORT_TEXTURE);
                }
                let mut list = ui.frame(&self.world, &labels, &lines, target.as_ref(), &views);
                self.replay_logo.screen = ui.desktop.size;
                self.replay_logo.paint(&mut list);
                self.labels = labels;
                self.profile.lap("ui", Instant::now());
                match open_uploader(&mut self.uploader, &self.client_root) {
                    Ok(client) => {
                        stage.renderer.set_ui(&list, &mut BankSource { bank: &mut self.ui_bank, catalog: client.catalog() });
                        if !stage.renderer.radar_meshes_loaded() {
                            let meshes = killer_render::radar::load_meshes(client);
                            if meshes.len() < killer_render::radar::RadarMesh::ALL.len() {
                                log::warn!("radar: {} de {} malhas LineageDecos carregadas", meshes.len(), killer_render::radar::RadarMesh::ALL.len());
                            }
                            let mut bank = TextureBank::new();
                            stage.renderer.set_radar_meshes(&meshes, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
                        }
                    }
                    Err(e) => log::error!("cliente: {e}"),
                }
                if ui.radar_visible(&self.world) {
                    let renderer = &stage.renderer;
                    let steps = killer_render::radar::radar_steps(&views.radar, |mesh| renderer.radar_mesh_ready(mesh), &mut self.radar_toggle);
                    stage.renderer.set_radar(steps);
                }
                self.profile.lap("set_ui", Instant::now());
            }
            (false, Some(ui)) if lobby_c4 => {
                let tables = self.tables.clone();
                if let Some(tables) = tables {
                    let ui_size = (ui_space.0 as i32, ui_space.1 as i32);
                    let class_info: &[l2_dat::ClassInfo] = self.lobby.view.as_ref().map_or(&[], |view| view.class_info.as_slice());
                    let created = self.lobby_c4.is_none();
                    let c4 = self.lobby_c4.get_or_insert_with(|| crate::lobby_c4::LobbyC4::new(ui_size, &tables, class_info));
                    if created {
                        let ini = open_uploader(&mut self.uploader, &self.client_root)
                            .ok()
                            .and_then(|client| client.read_system_text("s_info.ini").ok())
                            .map(|text| parse_ini(&text))
                            .unwrap_or_else(|| parse_ini(""));
                        c4.set_ping_cache(crate::ping_cache::PingCache::from_ini(&ini, crate::ping_cache::now_key()));
                    }
                    c4.resize(ui_size);
                    c4.update(&mut self.lobby, (dt * 1000.0) as u32);
                    if c4.needs_credit_font() {
                        let font = open_uploader(&mut self.uploader, &self.client_root)
                            .map_err(|e| e.to_string())
                            .and_then(|client| ui2d::load_named_font(client, killer_ui::lobby_credit::FONT_TEXTURE, killer_ui::lobby_credit::FONT_FILE).map_err(|e| e.to_string()));
                        if let Err(e) = &font {
                            log::warn!("fonte dos créditos: {e}");
                        }
                        c4.set_credit_font(font.ok());
                    }
                    if let Some(link) = c4.take_credit_request() {
                        match open_uploader(&mut self.uploader, &self.client_root) {
                            Ok(client) => match client.read_help_text(&link) {
                                Ok(text) => c4.set_credit_html(&text, &ui.small, &|marker, id| ui2d::html_marker(self.tables.as_deref(), marker, id)),
                                Err(e) => log::warn!("créditos {link}: {e}"),
                            },
                            Err(e) => log::warn!("créditos {link}: {e}"),
                        }
                    }
                    let ping_edits = c4.take_ping_edits();
                    if !ping_edits.is_empty() {
                        let path = self.client_root.join("system").join("s_info.ini");
                        if let Err(e) = crate::ui_option_config::save_or_create(&path, &ping_edits) {
                            log::warn!("s_info.ini: falha ao gravar {}: {e}", path.display());
                        }
                    }
                    let requests = c4.take_ping_requests();
                    if !requests.is_empty() {
                        self.ping_rx = Some(crate::server_ping::spawn(
                            requests.into_iter().map(|(id, ip, port)| crate::server_ping::PingRequest { id, ip, port }).collect(),
                        ));
                    }
                    if let Some(rx) = &self.ping_rx {
                        while let Ok((id, ms)) = rx.try_recv() {
                            c4.set_ping(id, ms);
                        }
                    }
                    if let Some(link) = c4.take_help_request() {
                        match open_uploader(&mut self.uploader, &self.client_root) {
                            Ok(client) => match client.read_help_text(&link) {
                                Ok(text) => c4.set_help(&text, &ui.small, &|marker, id| ui2d::html_marker(self.tables.as_deref(), marker, id)),
                                Err(e) => log::warn!("ajuda do servidor {link}: {e}"),
                            },
                            Err(e) => log::warn!("ajuda do servidor {link}: {e}"),
                        }
                    }
                    let slots = self.lobby.slots(ui_space, 1.0);
                    let tags = c4.name_tags(&self.lobby, &slots);
                    let mut list = c4.frame(&self.lobby, &ui.small, &tags);
                    if let Some(window) = ui.desktop.window(killer_ui::window::WindowId::OPTION).filter(|window| window.visible) {
                        ui.paint_window(&mut list, window, &self.world, &[], None, &ui2d::WorldViews::default());
                    }
                    match open_uploader(&mut self.uploader, &self.client_root) {
                        Ok(client) => stage.renderer.set_ui(&list, &mut BankSource { bank: &mut self.ui_bank, catalog: client.catalog() }),
                        Err(e) => log::error!("cliente: {e}"),
                    }
                }
            }
            (false, Some(ui)) if self.screen == Screen::World && self.tile.is_some() && self.load_error.is_none() => {
                match open_uploader(&mut self.uploader, &self.client_root) {
                    Ok(client) => stage.renderer.set_ui(&ui.loading_frame(), &mut BankSource { bank: &mut self.ui_bank, catalog: client.catalog() }),
                    Err(e) => log::error!("cliente: {e}"),
                }
            }
            _ => stage.renderer.clear_ui(),
        }
        stage.renderer.set_time(self.material_clock.elapsed().as_secs_f32());
        stage.renderer.set_view_target(self.world.me.as_ref().map(|me| me.position));
        if let Some(view) = &self.world_view {
            let position = viewer_position(&self.world, &view.camera);
            stage.renderer.set_viewer(Viewer { zone: view.zone_at(position), position, dt });
        }

        let egui = &mut stage.egui;
        let mut action = None;
        let mut lobby_commands = Vec::new();
        match (self.screen, &self.world_view, &self.tile) {
            (Screen::Login | Screen::Servers | Screen::Characters | Screen::Create, _, _) if lobby_c4 => {
                if let Some(c4) = self.lobby_c4.as_mut() {
                    lobby_commands = c4.take_commands();
                }
            }
            (Screen::Login | Screen::Servers | Screen::Characters | Screen::Create, _, _)
                if !self.modern_hud.preferences.enabled && self.lobby.load_error.is_none() && self.tables_error.is_none() && self.ui_error.is_none() => {}
            (Screen::Login | Screen::Servers | Screen::Characters | Screen::Create, _, _) => {
                let points_per_pixel = 1.0 / egui.ctx.pixels_per_point();
                lobby_commands = self.lobby.panels(&egui.ctx, version(), (size.0 as f32, size.1 as f32), points_per_pixel);
            }
            (Screen::World, Some(_), _) if self.pending.is_some() => {}
            (Screen::World, Some(view), _) => {
                if let Some(ui) = &self.game_ui {
                    if let Ok(client) = open_uploader(&mut self.uploader, &self.client_root) {
                        self.modern_hud.prepare_icons(&egui.ctx, ui, client);
                        self.modern_hud.prepare_combat(&egui.ctx, &self.world, self.tables.as_deref(), client);
                    }
                    let lines: Vec<ChatEntry> = self.chat.iter().map(ui2d::chat_entry).collect();
                    self.modern_hud.paint(&egui.ctx, ui, &self.world, &lines, self.ui_scale);
                    self.modern_hud.paint_combat(&egui.ctx, ui, &self.world, &self.labels, self.ui_scale);
                }
                let info = HudInfo {
                    fps: self.fps,
                    hour,
                    ignored_total: self.ignored.total(),
                    ignored_distinct: self.ignored.distinct(),
                    ignored_top: self.ignored.top(HUD_TOP),
                    tables_error: self.tables_error.as_deref(),
                    ui_error: self.ui_error.as_deref(),
                };
                if ui::hud(&egui.ctx, view, &self.world, &info, self.hud) {
                    action = Some(Action::Leave);
                }
            }
            (Screen::World, None, _) if self.load_error.is_some() => ui::error(&egui.ctx, self.load_error.as_deref().unwrap_or_default()),
            (Screen::World, None, Some(tile)) if self.game_ui.is_none() => ui::loading(&egui.ctx, tile),
            (Screen::World, None, _) => {}
        }
        self.profile.lap("egui", Instant::now());
        let mut output = egui.ctx.end_pass();
        egui.state.handle_platform_output(&stage.window, output.platform_output);
        let chat_focused = self.game_ui.as_ref().is_some_and(|ui| ui.chat_focused());
        if cfg!(any(target_os = "ios", target_os = "android")) {
            let lobby_text = lobby_c4 && self.lobby_c4.as_ref().is_some_and(|c4| c4.wants_text(self.lobby.screen));
            let wanted = chat_focused || lobby_text;
            if wanted != self.keyboard_shown {
                stage.window.set_ime_allowed(wanted);
                self.keyboard_shown = wanted;
            }
        } else if chat_focused {
            stage.window.set_ime_allowed(true);
        }
        let jobs = egui.ctx.tessellate(output.shapes, output.pixels_per_point);
        let screen = ScreenDescriptor { size_in_pixels: [size.0, size.1], pixels_per_point: output.pixels_per_point };
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                egui.renderer.update_texture(device, queue, *id, delta);
            }
        }
        if self.screen==Screen::World && self.world_view.is_some() && self.game_ui.is_some() {
            self.modern_hud.upload_names(&egui.ctx,&mut stage.renderer,&egui.renderer);
        } else {stage.renderer.clear_world_text();}
        if let (Some(target), Some(param)) = (self.fishing_target.as_mut(), self.world.fishing_view.param) {
            let forward = killer_assets::engine_math::rotator_forward(param.rotation);
            let fishing = killer_render::camera::Camera { eye: param.location, target: param.location + forward, fov_x_deg: param.fov, near: camera.near, far: camera.far };
            stage.renderer.render_viewport(target, &fishing, &environment);
        }
        stage.renderer.render(&mut encoder, &color, &depth, size, &camera, &environment);
        self.profile.lap("render", Instant::now());
        let extra = egui.renderer.update_buffers(device, queue, &mut encoder, &jobs, &screen);
        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &color,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                })
                .forget_lifetime();
            egui.renderer.render(&mut pass, &jobs, &screen);
        }
        self.profile.lap("egui_gpu", Instant::now());
        queue.submit(extra.into_iter().chain([encoder.finish()]));
        self.profile.lap("submit", Instant::now());
        for id in &output.textures_delta.free {
            egui.renderer.free_texture(id);
        }
        output.textures_delta.clear();
        let settled = (self.world_view.is_some() || (self.screen != Screen::World && self.lobby.settled()))
            && !self.cache.has_pending();
        if settled || self.load_error.is_some() || self.lobby.load_error.is_some() {
            self.frames_ready += 1;
            if self.frames_ready == std::env::var(SCREENSHOT_FRAME_ENV).ok().and_then(|value| value.parse().ok()).unwrap_or(SCREENSHOT_FRAME)
                && let Some(path) = self.screenshot.take()
            {
                self.capture = Some(path);
            }
        }
        if let Some(path) = self.capture.take() {
            match stage.gpu.read_back(&surface.texture).and_then(|rgba| save_capture(&path, size, &rgba)) {
                Ok(()) => log::info!("captura em {} ({:.0} fps)", path.display(), self.fps),
                Err(e) => log::error!("captura: {e}"),
            }
        }
        stage.window.pre_present_notify();
        queue.present(surface);
        if let Some(report) = self.profile.end(Instant::now()) {
            let draws = stage.renderer.visible_draw_count(&camera, size.0 as f32 / size.1.max(1) as f32);
            log::debug!("{report} | {draws} draws, {} instâncias, {:.0} fps", stage.renderer.instance_count(), self.fps);
        }
        match action {
            Some(Action::Leave) => self.leave(),
            None => {}
        }
        let mut quit = false;
        self.run_lobby_commands(lobby_commands, &mut quit);
        self.quit |= quit;
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.stage.is_some() {
            return;
        }
        let size = client_viewport(&self.client_root);
        self.splash = crate::splash::Splash::open(event_loop, &self.client_root);
        log::info!("splash: {} em {:?} depois do início", if self.splash.is_some() { "aberta" } else { "não aberta" }, self.started.elapsed());
        let attributes = if cfg!(any(target_os = "ios", target_os = "android")) {
            Window::default_attributes().with_title("Killer").with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        } else {
            Window::default_attributes().with_title("Killer").with_inner_size(size).with_visible(self.splash.is_none())
        };
        let window = match event_loop.create_window(attributes) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                log::error!("janela: {e}");
                event_loop.exit();
                return;
            }
        };
        self.game_cursors = killer_assets::client::Client::open(&self.client_root).ok()
            .and_then(|mut client| client.read_system_file("nwindow.dll").ok())
            .and_then(|bytes| killer_assets::cursor::load(&bytes))
            .unwrap_or_default().into_iter()
            .map(|cursor| winit::window::CustomCursor::from_rgba(cursor.rgba, cursor.width, cursor.height, cursor.hotspot[0], cursor.hotspot[1]).ok()
                .map(|source| event_loop.create_custom_cursor(source))).collect();
        let real = crate::gpu::surface_size(&window);
        self.ui_scale = ui_scale_for(&window);
        log::info!(
            "janela: pedido {} x {} lógicos (viewport do cliente); quadro {} x {} físicos, fator de escala {} (interface montada no espaço do cliente e ampliada pela GPU)",
            size.width,
            size.height,
            real.width,
            real.height,
            window.scale_factor()
        );
        let gpu = match Gpu::new(window.clone()) {
            Ok(g) => g,
            Err(e) => {
                log::error!("gpu: {e}");
                event_loop.exit();
                return;
            }
        };
        let renderer = Renderer::new(gpu.device.clone(), gpu.queue.clone(), gpu.config.format);
        let egui = EguiLayer::new(&window, &gpu.device, gpu.config.format);
        log::info!("janela, GPU e pipelines em {:.1?}", self.started.elapsed());
        self.stage = Some(Stage { window, gpu, renderer, egui });
        self.last_frame = Instant::now();
        if let Some(tile) = self.tile.clone()
            && self.pending.is_none()
            && self.world_view.is_none()
        {
            let anchor = (self.demo && tile == "22_22").then(|| {
                let (x, y, z, heading) = DEMO_GIRAN_CENTER;
                (Vec3::new(x, y, z), heading)
            });
            self.start_loading(tile, anchor);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if let Some(splash) = self.splash.as_mut().filter(|splash| splash.window.id() == window_id) {
            match event {
                WindowEvent::RedrawRequested => splash.paint(),
                WindowEvent::Resized(_) => splash.resize(),
                WindowEvent::CloseRequested => event_loop.exit(),
                _ => {}
            }
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                self.save_windows_info();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.frame();
                self.update_cursor(event_loop);
                if self.quit {
                    self.save_windows_info();
                    event_loop.exit();
                }
            }
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key, state, repeat: false, .. }, .. }
                if state.is_pressed() && (physical_key == PhysicalKey::Code(KeyCode::PrintScreen)
                    || physical_key == PhysicalKey::Code(KeyCode::F12) && !self.modern_hud.preferences.enabled) =>
            {
                self.capture = Some(capture_path(std::env::var_os("HOME").map(PathBuf::from), SystemTime::now()));
            }
            WindowEvent::Resized(size) => {
                self.apply_ui_scale();
                let Some(stage) = self.stage.as_mut() else {
                    return;
                };
                let _ = stage.egui.state.on_window_event(&stage.window, &event);
                stage.gpu.resize(size.width, size.height);
                if let (Some(runtime), Some(pending)) = (&mut self.video_options, &mut self.video_apply) {
                    let result = runtime.display.resized(&stage.window, size);
                    pending.transaction.confirm_resolution(result, runtime.display.params);
                }
                self.advance_video_apply();
            }
            WindowEvent::Touch(touch) if self.stage.is_some() => {
                if let Some(stage) = &self.stage {
                    self.touches.set_scale(stage.window.scale_factor());
                }
                self.touches.set_direct(self.lobby_c4_active());
                let in_world = self.screen == Screen::World && self.world_view.is_some() && !self.lobby_c4_active();
                let touches = match (&self.stage, in_world) {
                    (Some(stage), true) => {
                        let size = crate::gpu::surface_size(&stage.window);
                        let screen = (f64::from(size.width), f64::from(size.height));
                        let scale = stage.window.scale_factor();
                        let (consumed, fired) = self.touch_hud.handle(touch, screen, scale);
                        if let Some(button) = fired {
                            self.press_touch_button(button);
                        }
                        if consumed { Vec::new() } else { self.joystick.handle(touch, screen, scale) }
                    }
                    _ => {
                        self.joystick.reset();
                        self.touch_hud.reset();
                        vec![touch]
                    }
                };
                for touch in touches {
                    for event in self.touches.handle(touch, Instant::now()) {
                        self.dispatch(event);
                    }
                }
            }
            event if self.stage.is_some() => self.dispatch(event),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.splash.is_some() && self.splash_finished() {
            if let Some(stage) = &self.stage {
                stage.window.set_visible(true);
                stage.window.focus_window();
            }
            self.splash = None;
            log::info!("splash: fechada em {:?} depois do início", self.started.elapsed());
        }
        if let Some(splash) = &self.splash {
            splash.window.request_redraw();
        }
        if self.stage.is_some() {
            for touch in self.joystick.tick(Instant::now()) {
                for event in self.touches.handle(touch, Instant::now()) {
                    self.dispatch(event);
                }
            }
            for event in self.touches.tick(Instant::now()) {
                self.dispatch(event);
            }
        }
        if let Some(stage) = &self.stage {
            stage.window.request_redraw();
        }
    }
}

fn download_panel(ctx: &egui::Context, snapshot: &crate::download::Snapshot) -> bool {
    use crate::download::Phase;
    let gigabytes = |bytes: u64| bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let mut retry = false;
    egui::Window::new("Arquivos do cliente")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.set_min_width(320.0);
            match &snapshot.phase {
                Phase::Starting => {
                    ui.label("Conectando ao servidor de arquivos...");
                }
                Phase::Running | Phase::Done => {
                    let fraction = if snapshot.total_bytes == 0 { 0.0 } else { snapshot.done_bytes as f32 / snapshot.total_bytes as f32 };
                    ui.add(egui::ProgressBar::new(fraction.clamp(0.0, 1.0)).show_percentage());
                    ui.label(format!(
                        "{:.2} GB de {:.2} GB ({} de {} arquivos)",
                        gigabytes(snapshot.done_bytes),
                        gigabytes(snapshot.total_bytes),
                        snapshot.done_files,
                        snapshot.total_files
                    ));
                }
                Phase::Failed(message) => {
                    ui.colored_label(egui::Color32::LIGHT_RED, message);
                    retry = ui.button("Tentar novamente").clicked();
                }
            }
        });
    retry
}

fn log_session_lists(event: &WorldEvent) {
    match event {
        WorldEvent::ShortCutInit(init) => {
            let mut pages = [0usize; 10];
            for shortcut in &init.shortcuts {
                if let Some(page) = usize::try_from(shortcut.slot / 12).ok().filter(|p| *p < pages.len()) {
                    pages[page] += 1;
                }
            }
            log::info!("atalhos do servidor: {} em páginas {pages:?}", init.shortcuts.len());
        }
        WorldEvent::ItemList(list) => log::info!("inventário do servidor: {} itens", list.items.len()),
        WorldEvent::InventoryUpdate(update) => log::info!("atualização do inventário: {:?}", update.changes.iter().map(|change| (change.change, change.item.item_id, change.item.count)).collect::<Vec<_>>()),
        WorldEvent::SkillList(list) => log::info!("skills do servidor: {}", list.skills.len()),
        WorldEvent::SendMacroList(list) => log::debug!(
            "macros do servidor: flag {} revisão {} total {} neste pacote {}",
            list.flag,
            list.revision,
            list.total_count,
            list.macros.len()
        ),
        _ => {}
    }
}

pub fn egui_consumes(ctx: &egui::Context, event: &WindowEvent, reported: bool) -> bool {
    match event {
        WindowEvent::MouseInput { .. } | WindowEvent::MouseWheel { .. } => {
            ctx.egui_is_using_pointer() || (pointer_over_egui_area(ctx) && !ctx.input(|i| i.pointer.any_down()))
        }
        _ => reported,
    }
}

fn pointer_over_egui_area(ctx: &egui::Context) -> bool {
    ctx.input(|i| i.pointer.interact_pos()).and_then(|pos| ctx.layer_id_at(pos)).is_some_and(|layer| layer.order != egui::Order::Background)
}



struct LandmarkEffects<'a> {
    active: &'a mut HashMap<crate::landmark::Slot, EffectId>,
    loose: &'a mut Vec<(EffectId, f32)>,
    assets: &'a mut HashMap<String, Option<EmitterAsset>>,
    bank: &'a mut TextureBank,
}

fn sync_landmark(renderer: &mut Renderer, requests: Vec<crate::landmark::Request>, effects: &mut LandmarkEffects<'_>, uploader: &mut Option<Client>, root: &Path, view: &WorldView, dt: f32) {
    use crate::landmark::Request;
    for request in requests {
        match request {
            Request::Spawn { slot, class, location, rotation } => {
                if let Some(old) = effects.active.remove(&slot) { renderer.remove_effect(old); }
                let Ok(client) = open_uploader(uploader, root) else { continue };
                let loaded = effects.assets.entry(class.to_owned()).or_insert_with(|| {
                    client.emitter_class(class).map(|emitter| killer_assets::scene::emitters::effect_asset(client, &emitter, &view.options))
                });
                let Some(asset) = loaded.as_ref() else { continue };
                let id = renderer.add_effect(asset, &mut BankSource { bank: effects.bank, catalog: client.catalog() });
                renderer.place_effect(id, EffectPlacement { location, rotation, zone: view.zone_at(location), visible: true });
                effects.active.insert(slot, id);
            }
            Request::Remove(slot) => {
                if let Some(id) = effects.active.remove(&slot) { renderer.remove_effect(id); }
            }
            Request::Release { slot, life } => {
                if let Some(id) = effects.active.remove(&slot) { effects.loose.push((id, life)); }
            }
        }
    }
    effects.active.retain(|_, id| {
        let finished = renderer.effect_finished(*id);
        if finished { renderer.remove_effect(*id); }
        !finished
    });
    effects.loose.retain_mut(|(id, life)| {
        *life = (f64::from(*life)-f64::from(dt)) as f32;
        let finished = *life <= 0.0001 || renderer.effect_finished(*id);
        if finished { renderer.remove_effect(*id); }
        !finished
    });
}

fn open_uploader<'a>(uploader: &'a mut Option<Client>, root: &Path) -> std::io::Result<&'a mut Client> {
    match uploader {
        Some(client) => Ok(client),
        None => Ok(uploader.insert(Client::open(root)?)),
    }
}

fn tile_center(tile: &str) -> Option<(f32, f32)> {
    let (x, y) = tile.split_once('_')?;
    let x: i32 = x.parse().ok()?;
    let y: i32 = y.parse().ok()?;
    Some(((x - 20) as f32 * TILE_SIZE + TILE_SIZE / 2.0, (y - 18) as f32 * TILE_SIZE + TILE_SIZE / 2.0))
}

fn player_start(level: &Level) -> Option<(Vec3, i32)> {
    level.actors.iter().find(|a| a.class.eq_ignore_ascii_case("PlayerStart")).map(|a| (Vec3::from(a.placement.location), a.placement.rotation[1]))
}

fn terrain_height_near(client: &mut Client, level: &Level, x: f32, y: f32, radius: f32) -> Option<f32> {
    let mut heights: Vec<f32> = level
        .terrains
        .iter()
        .flat_map(|t| terrain::build(client, t, terrain_zone(level, t), true, None, &[], &terrain::TerrainSeam::NONE))
        .flat_map(|d| d.vertices)
        .filter(|v| (v.position[0] - x).powi(2) + (v.position[1] - y).powi(2) <= radius * radius)
        .map(|v| v.position[2])
        .collect();
    if heights.is_empty() {
        return None;
    }
    heights.sort_by(f32::total_cmp);
    Some(heights[heights.len() / 2])
}

fn initial_orbit(client: &mut Client, level: &Level, tile: &str, scene: &Scene, trace: &TraceMesh, settings: CameraSettings, anchor: Option<(Vec3, i32)>) -> OrbitCamera {
    let mut camera = match anchor.or_else(|| player_start(level)) {
        Some((location, yaw)) => OrbitCamera::behind(location, yaw, settings),
        None => {
            let (x, y) = tile_center(tile).unwrap_or(((scene.bounds_min[0] + scene.bounds_max[0]) / 2.0, (scene.bounds_min[1] + scene.bounds_max[1]) / 2.0));
            let z = terrain_height_near(client, level, x, y, 512.0).unwrap_or(scene.bounds_max[2]);
            OrbitCamera::behind(Vec3::new(x, y, z), 0, settings)
        }
    };
    camera.collide(|from, to| trace.nearest(from, to));
    camera
}

fn viewer_position(world: &World, camera: &OrbitCamera) -> Vec3 {
    world.me.as_ref().map_or(camera.target, |me| me.position)
}

/// O tamanho da janela é o que o cliente tem configurado, não um valor nosso: o
/// `GamePlayViewportX/Y` do `option.ini`, e na falta dele o `WindowedViewportX/Y` da seção
/// `[WinDrv.WindowsClient]` do `lineage2.ini`. É medida **lógica**: numa tela com fator de escala 2
/// a janela ocupa o mesmo espaço que ocuparia no original, e o quadro sai com o dobro dos pixels —
/// o mundo em 3D ganha resolução e a interface é montada no espaço do cliente e ampliada pela GPU
/// (`Renderer::set_ui_size`), ficando no mesmo tamanho de tela do original.
const SSQ_ENV_END: f32 = 0.02;
const SSQ_ENV_RAMP: f32 = 0.02;
const SSQ_ENV_TARGET: f32 = 1.0;
fn ui_scale_for(window: &Window) -> f32 {
    let factor = window.scale_factor() as f32;
    if cfg!(any(target_os = "ios", target_os = "android")) {
        factor.min(crate::gpu::surface_size(window).height as f32 / MOBILE_UI_HEIGHT).max(0.01)
    } else {
        factor
    }
}

fn client_viewport(root: &Path) -> LogicalSize<u32> {
    let fallback = LogicalSize::new(1280, 720);
    let Ok(mut client) = Client::open(root) else { return fallback };
    let read = |client: &mut Client, name: &str| client.read_system_text(name).map(|t| parse_ini(&t)).ok();
    let from_option = read(&mut client, OPTION_INI).and_then(|ini| killer_ui::ini::Options::from_ini(&ini).viewport);
    let from_l2 = || {
        let ini = read(&mut client, L2_INI)?;
        let x = ini.int("WinDrv.WindowsClient", "WindowedViewportX")?;
        let y = ini.int("WinDrv.WindowsClient", "WindowedViewportY")?;
        Some((x, y))
    };
    let (x, y) = from_option.or_else(from_l2).unwrap_or((1280, 720));
    match (u32::try_from(x), u32::try_from(y)) {
        (Ok(x), Ok(y)) if x > 0 && y > 0 => LogicalSize::new(x, y),
        _ => fallback,
    }
}

fn display_settings(client: &mut Client) -> DisplaySettings {
    let read = |client: &mut Client, name: &str| match client.read_system_text(name) {
        Ok(text) => parse_ini(&text),
        Err(e) => {
            log::warn!("{name}: {e}; exibição de personagens com os padrões do Core.dll");
            parse_ini("")
        }
    };
    let l2 = read(client, L2_INI);
    let option = read(client, OPTION_INI);
    DisplaySettings::from_ini(&l2, &option)
}


fn key_binds(client: &mut Client) -> killer_world::engine_input::KeyBinds {
    match client.read_system_text(USER_INI) {
        Ok(text) => {
            let ini = parse_ini(&text);
            killer_world::engine_input::KeyBinds::from_ini(|name| ini.get(killer_world::engine_input::INPUT_SECTION, name))
        }
        Err(e) => {
            log::warn!("{USER_INI}: {e}; sem ligações de tecla");
            killer_world::engine_input::KeyBinds::default()
        }
    }
}

fn camera_settings(client: &mut Client) -> CameraSettings {
    match client.read_system_text(USER_INI) {
        Ok(text) => CameraSettings::from_ini(&parse_ini(&text)),
        Err(e) => {
            log::warn!("{USER_INI}: {e}; câmera com os padrões de LineagePlayerController");
            CameraSettings::default()
        }
    }
}

fn load_tile(root: &Path, tile: &str, anchor: Option<(Vec3, i32)>, want_tables: bool, ui_size: Option<(i32, i32)>) -> Result<LoadedTile, String> {
    let started = Instant::now();
    let mut client = Client::open(root).map_err(|e| e.to_string())?;
    let tables = want_tables.then(|| client.tables().map_err(|e| e.to_string()));
    let ui = ui_size.map(|size| GameUi::load(&mut client, size).map_err(|e| e.to_string()));
    if let Some(Ok(ui)) = &ui {
        log::info!(
            "interface: fontes {} e {}, viewport {:?}, {} posições salvas, transparência {} em {:.1?}",
            ui.small.atlas,
            ui.large.atlas,
            ui.options.viewport,
            ui.windows_info.positions.len(),
            ui.options.transparency,
            started.elapsed()
        );
    }
    if let Some(Ok(tables)) = &tables {
        log::info!("tabelas: {} npcs, {} linhas do chargrp, {} nomes, {} seções de animação em {:.1?}", tables.npcgrp.len(), tables.chargrp.len(), tables.npcname.len(), tables.anim_names.len(), started.elapsed());
    }
    let level_path = client.level_path(tile).map_err(|e| e.to_string())?;
    let level_raw = std::fs::read(&level_path).map_err(|e| e.to_string())?;
    let level_stamp = crate::scene_cache::stamp(&mut client, &level_raw);
    let cache = SceneCache::default_dir();
    let cached_scene = cache.as_ref().and_then(|cache| cache.read(&level_path, &level_stamp));
    log::info!("cache da cena {tile}: {}", if cached_scene.is_some() { "acertou" } else { "ausente" });
    let loaded = client.parse_level(tile, &level_path, &level_raw).map_err(|e| e.to_string())?;
    drop(level_raw);
    let weather = Weather::load(&mut client).map_err(|e| e.to_string())?;
    log::info!("mapa {tile} lido em {:.1?}", started.elapsed());
    let options = ClientOptions::load(&mut client).unwrap_or_else(|e| {
        log::warn!("opções do cliente: {e}; padrões do engine");
        ClientOptions::default()
    });
    let (scene, mut bank, cache_fresh) = match cached_scene {
        Some(cached) => {
            let (mut scene, bank) = cached.into_bank();
            scene.emitters = killer_assets::scene::emitters::build(&mut client, &loaded.level, &options);
            (scene, bank, false)
        }
        None => {
            let scene = build_scene_with(&mut client, &loaded.level, &options, Neighbours::default());
            log::info!("cena {tile}: {} draws em {:.1?}", scene.draws.len(), started.elapsed());
            (scene, TextureBank::new(), true)
        }
    };
    let sky = options.shared_sky.then(|| sky::load_shared(&mut client, &options)).flatten();
    if let Some(sky) = &sky {
        log::info!("céu compartilhado: {} draws, {} astros, {} lens flares", sky.draws.len(), sky.sky_bodies.len(), sky.flares.len());
    }
    for draw in scene.material_draws().chain(sky.iter().flat_map(|sky| sky.draws.iter())) {
        for path in draw.material.texture_paths() {
            bank.texture(client.catalog(), path);
        }
        if let Some(mask) = &draw.mask {
            bank.alpha_map(client.catalog(), mask);
        }
    }
    log::info!("texturas {tile} decodificadas em {:.1?}", started.elapsed());
    let trace = TraceMesh::from_scene(&scene);
    log::info!("traço {tile}: {} triângulos em {:.1?}", trace.triangles(), started.elapsed());
    let display = display_settings(&mut client);
    log::info!(
        "exibição ({L2_INI}/{OPTION_INI}): alcance dos pawns {:.0} u, limite {} pawns, nomes {} até {} u, nomes de NPC {} / outros PCs {}",
        display.pawn_range(),
        display.render_limit,
        display.show_names,
        display.name_dist,
        display.show_npc_name,
        display.show_other_pc_name
    );
    let binds = key_binds(&mut client);
    let settings = camera_settings(&mut client);
    log::info!("câmera ({USER_INI}): zoom [{:.0}, {:.0}], roda {:+.0}/{:+.0}, HitCheck {} (mín {}), altura {:.1}", settings.min_zoom, settings.max_zoom, settings.wheel_down, settings.wheel_up, settings.hit_check, settings.hit_check_min_dist, settings.view_height_adjust);
    let camera = initial_orbit(&mut client, &loaded.level, tile, &scene, &trace, settings, anchor);
    log::info!("câmera inicial {tile}: alvo {:?}, distância {:.0} (vista {:.0})", camera.target, camera.distance, camera.view_distance());
    let ground = GroundGrid::build(&scene);
    log::info!("chão {tile}: {} triângulos andáveis em {:.1?}", ground.triangle_count(), started.elapsed());
    let bsp_tree = loaded.level.bsp_tree.clone();
    for texture in sky.iter().flat_map(|sky| sky.flares.iter().map(|flare| flare.texture.as_str()).chain(sky.sky_bodies.iter().flat_map(|body| body.material.texture_paths()))) {
        bank.texture(client.catalog(), texture);
    }
    if cache_fresh
        && let Some(cache) = cache
    {
        let textures = bank.textures().collect::<Vec<_>>();
        let alpha_maps = bank.alpha_maps().collect::<Vec<_>>();
        client.catalog().release_files();
        if let Err(error) = cache.write_parts(&level_path, &level_stamp, &scene, &textures, &alpha_maps) {
            log::warn!("cache da cena {tile}: {error}");
        }
    }
    let level = Arc::new(loaded.level);
    let mut collision = killer_assets::collision::resources::Resources::default();
    if let Err(error) = collision.tile(&mut client, tile, &level) {
        log::warn!("colisão do tile {tile}: {error}");
    }
    Ok(LoadedTile { tile: tile.to_string(), scene, trace, weather, bsp_tree, ground, bank, camera, display, tables, ui, elapsed: started.elapsed(), level, collision, options, sky, binds })
}

fn capture_stamp() -> Option<killer_ui::text_capture::Stamp> {
    let seconds = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i32;
    let date = killer_platform::calendar::from_seconds(seconds)?;
    Some(killer_ui::text_capture::Stamp { month: date.month, day: date.day, hour: date.hour, minute: date.minute, second: seconds.rem_euclid(60) })
}

#[cfg(test)]
mod tests {

    /// A janela abre com a resolução que o cliente tem no `option.ini` (`GamePlayViewportX/Y`), em
    /// medida física, e não com um valor nosso — senão a proporção da interface na tela não bate
    /// com a do original.
    #[test]
    #[ignore]
    fn the_window_opens_with_the_viewport_the_client_has_configured() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let mut client = Client::open(&root).unwrap();
        let ini = parse_ini(&client.read_system_text(OPTION_INI).unwrap());
        let expected = killer_ui::ini::Options::from_ini(&ini).viewport.expect("GamePlayViewport no option.ini");
        let size = client_viewport(&root);
        assert_eq!((size.width as i32, size.height as i32), expected);
        assert_ne!(size, winit::dpi::LogicalSize::new(1280, 720), "não é o valor de reserva");
    }
    use super::*;
    use l2_net::game::packets::CHAT_ALL;
    use crate::ui::HudInfo;
    use killer_assets::tiles::tile_name;
    use killer_world::EntityKind;
    use l2_net::game::packets::{NpcHtmlMessage, PAPERDOLL_SLOTS, VISIBLE_EQUIP_SLOTS};
    use l2_net::game::{PaperdollSlot, UserInfoHead, UserStats};
    use l2_net::login::GameServer;
    use ue2_level::{Actor, Placement};
    use winit::dpi::PhysicalPosition;
    use winit::event::{DeviceId, TouchPhase};

    const SCREEN: (f32, f32) = (1280.0, 720.0);
    const USER_INI_FIXTURE: &str = include_str!("../../../crates/killer-ui/tests/fixtures/user.ini");

    #[test]
    fn alt_p_100c6c30_emits_the_message_for_the_new_min_frame_state() {
        assert_eq!(min_frame_rate_sysstring(false), 1283);
        assert_eq!(min_frame_rate_sysstring(true), 1284);
    }

    fn test_view() -> WorldView {
        let camera = OrbitCamera::behind(Vec3::new(83400.0, 148000.0, -3400.0), 16384, CameraSettings::from_ini(&parse_ini(USER_INI_FIXTURE)));
        WorldView {
            moving_trace: crate::trace::MovingTrace::default(),
            tile: "22_22".into(),
            trace: TraceMesh::from_scene(&Scene::default()),
            zones: 0,
            lighting: LightMap::default(),
            camera,
            display: DisplaySettings::default(),
            free: FreeCamera::from_orbit(&camera),
            use_free: false,
            timeenv: TimeEnv::parse(""),
            weather: Weather::new(TimeEnv::parse("")),
            bsp_tree: BspTree::default(),
            ground: GroundGrid::default(),
            shakes: Default::default(),
            fade: Default::default(),
            fade_shown: false,
            draws: 0,
            triangles: 0,
            characters: 0,
            instances: 0,
            neighbours: NeighbourSet::new(TileId { x: 22, y: 22 }, Arc::new(Level::default())),
            options: ClientOptions::default(),
        }
    }

    fn app_in_world() -> App {
        let mut app = App::new(None, Some("22_22".into()), None, None, false, 0, None);
        app.world_view = Some(test_view());
        app
    }

    #[test]
    fn delayed_pick_rejects_previous_query_removed_inventory_and_replaced_actor() {
        use killer_assets::{collision::world::{Selection,Target},pickup::TraceHit};
        let mut app = app_in_world();
        app.world = characters::demo_world(Vec3::ZERO,0,0);
        app.world.inventory.items = vec![l2_net::game::InventoryItem { object_id:97001,item_id:57,count:5,..Default::default() }];
        let dialog = || PendingPick::Drop(killer_world::Dialog::DropItem { object_id:97001,name:"Adena".into(),message_id:ui2d::DROP_COUNT_SYSMSG,count:Some(5),position:[0;3],kind:killer_world::ItemDropKind::Drop });
        let hit = Selection { hit:TraceHit { position:Vec3::new(10.0,20.0,-30.0),normal:Vec3::Z,time:0.5,non_level_actor:false },target:None,surface:None,immediately_stop:false,pushable_mass:None,server_id:None };
        app.pick_ticket = 2;
        app.pending_pick = Some(dialog());
        app.finish_pick(1,Ok(Some(hit)));
        assert!(app.pending_pick.is_some());
        assert!(app.world.dialogs.pending.is_empty());
        app.world.inventory.items.clear();
        app.finish_pick(2,Ok(Some(hit)));
        assert!(app.pending_pick.is_none());
        assert!(app.world.dialogs.pending.is_empty());
        app.world.inventory.items.push(l2_net::game::InventoryItem { object_id:97001,item_id:57,count:5,..Default::default() });
        app.pending_pick = Some(dialog());
        app.finish_pick(2,Ok(Some(hit)));
        assert!(matches!(app.world.dialogs.pending.first(),Some(killer_world::Dialog::DropItem { position:[10,20,-30],.. })));
        let mut entity = app.world.me.clone().unwrap();
        entity.object_id = 9000;
        entity.spawned = 4;
        app.world.entities.insert(9000,entity);
        app.world.dialogs.pending.clear();
        app.pending_pick = Some(dialog());
        app.finish_pick(2,Ok(Some(Selection { target:Some(Target { object_id:9000,spawned:4,position:Vec3::ZERO,radius:5.0,height:10.0 }),..hit })));
        assert!(matches!(app.world.dialogs.pending.first(),Some(killer_world::Dialog::DropItem { .. })),"um jogador só sob o cursor não impede (GetTargetedActor(1) olha o alvo selecionado)");
        app.world.dialogs.pending.clear();
        app.world.target = Some(9000);
        app.pending_pick = Some(dialog());
        app.finish_pick(2,Ok(Some(hit)));
        assert!(app.world.dialogs.pending.is_empty(),"com um jogador como alvo selecionado soltar no mundo não faz nada (0x1006a900)");
        app.world.target = None;
        app.pending_pick = Some(PendingPick::Move { ctrl:false,shift:true });
        app.finish_pick(2,Ok(Some(Selection { target:Some(Target { object_id:9000,spawned:3,position:Vec3::ZERO,radius:5.0,height:10.0 }),..hit })));
        assert_ne!(app.world.target,Some(9000));
    }

    #[test]
    #[ignore]
    fn inventory_drag_crosses_app_ray_dialog_and_drop_packet_with_client_data() {
        let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from).expect("KILLER_CLIENT_DIR");
        let mut loaded = load_tile(&root, "22_22", Some((Vec3::new(83400.0, 148000.0, -3400.0), 16384)), true, Some((1280, 720))).unwrap();
        let trace = TraceMesh::from_scene(&loaded.scene);
        let tables = loaded.tables.take().unwrap().unwrap();
        let mut app = app_in_world();
        app.screen = Screen::World;
        let (requests,inbox) = std::sync::mpsc::channel();
        let (_outbox,results) = std::sync::mpsc::channel();
        app.loader = Some(crate::loader::Loader { requests,results });
        let view = app.world_view.as_mut().unwrap();
        view.trace = trace;
        view.neighbours = NeighbourSet::new(TileId { x:22,y:22 },loaded.level.clone());
        view.camera = loaded.camera;
        app.game_ui = Some(loaded.ui.take().unwrap().unwrap());
        app.tables = Some(Arc::new(tables));
        app.world = characters::demo_world(view.camera.target, view.camera.facing(), 0);
        app.world.inventory.items = vec![l2_net::game::InventoryItem { object_id: 97001, item_id: 57, count: 5, ..Default::default() }];
        let ui = app.game_ui.as_mut().unwrap();
        ui.set_tab_names(app.tables.as_deref().unwrap());
        ui.sync_windows(&app.world, app.tables.as_deref(), Instant::now());
        for window in &mut ui.desktop.windows { window.visible = window.id == killer_ui::window::WindowId::INVENTORY; }
        let window = ui.desktop.window_mut(killer_ui::window::WindowId::INVENTORY).unwrap();
        window.rect.x = 1000;
        window.rect.y = 100;
        let (sx, sy) = killer_ui::inventory::grid_slot_pos(0);
        let x = 1000 + killer_ui::inventory::GRID_RECT.x + sx + 4;
        let y = 100 + killer_ui::inventory::GRID_RECT.y + sy + 4;
        app.ui_scale = 2.0;
        app.dispatch(moved(x as f32 * 2.0, y as f32 * 2.0));
        app.dispatch(mouse(ElementState::Pressed, MouseButton::Left));
        app.dispatch(moved(1280.0, 720.0));
        app.dispatch(mouse(ElementState::Released, MouseButton::Left));
        let mut client = Client::open(&root).unwrap();
        let mut shared = crate::loader::Shared::default();
        let request = inbox.try_recv().expect("arraste envia consulta de colisão");
        let LoadResult::Pick { ticket,selection } = crate::loader::handle(&mut client,&mut shared,request) else { panic!("consulta de seleção"); };
        app.finish_pick(ticket,selection);
        assert_eq!(app.world.dialogs.pending.len(), 1, "arrasto precisa chegar à confirmação");
        app.tick_dialogs();
        let ui = app.game_ui.as_mut().unwrap();
        let dialog = ui.dialog.as_mut().expect("confirmação visível");
        assert_eq!(dialog.view.dialog_type, ui2d::DROP_DIALOG_TYPE);
        dialog.view.input.as_mut().unwrap().text = "3".into();
        let KeyAction::Dialog(answer, accept) = ui.key(killer_ui::chat::InputKey::Enter) else { panic!("Enter deve confirmar"); };
        assert!(dialog_command(answer.clone(), false).is_none());
        let Some(NetCommand::DropItem { object_id, count, position }) = dialog_command(answer, accept) else { panic!("comando de descarte"); };
        assert_eq!((object_id, count), (97001, 3));
        assert!((position[0] - 83400).abs() < 1000 && (position[1] - 148000).abs() < 1000);
        assert!(position[2] < -3000 && position[2] > -4000);
        let packet = killer_world::net::encode(&NetCommand::DropItem { object_id, count, position }).unwrap();
        assert_eq!(packet[0], 0x12);
        assert_eq!(packet.len(), 21);
        assert_eq!(app.world.inventory.items[0].count, 5);
    }

    #[test]
    #[ignore]
    fn installed_user_ini_drives_zoom_through_the_world_ui_at_both_scales() {
        let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from).expect("KILLER_CLIENT_DIR");
        let mut client = Client::open(&root).unwrap();
        let settings = camera_settings(&mut client);
        assert_ne!(settings.wheel_up, 0.0, "MouseWheelUp ausente no cliente instalado");
        assert_ne!(settings.wheel_down, 0.0, "MouseWheelDown ausente no cliente instalado");
        for scale in [1.0, 2.0] {
            let mut app = app_in_world();
            app.screen = Screen::World;
            app.ui_scale = scale;
            app.game_ui = Some(GameUi::load(&mut client, (1280, 720)).unwrap());
            app.world_view.as_mut().unwrap().camera = OrbitCamera::behind(Vec3::ZERO, 0, settings);
            app.dispatch(moved(640.0 * scale, 360.0 * scale));
            let start = app.world_view.as_ref().unwrap().camera.zoom;
            for (ticks, expected) in [(1.0, start + settings.wheel_up), (-1.0, start + settings.wheel_up + settings.wheel_down)] {
                app.dispatch(WindowEvent::MouseWheel { device_id: DeviceId::dummy(), delta: MouseScrollDelta::LineDelta(0.0, ticks), phase: TouchPhase::Moved });
                assert_eq!(app.world_view.as_ref().unwrap().camera.zoom, expected);
            }
        }
    }

    fn mouse(state: ElementState, button: MouseButton) -> WindowEvent {
        WindowEvent::MouseInput { device_id: DeviceId::dummy(), state, button }
    }

    fn moved(x: f32, y: f32) -> WindowEvent {
        WindowEvent::CursorMoved { device_id: DeviceId::dummy(), position: PhysicalPosition::new(f64::from(x), f64::from(y)) }
    }

    fn hud_pass(ctx: &egui::Context, pointer: egui::Pos2) {
        let view = test_view();
        let world = characters::demo_world(view.camera.target, view.camera.facing(), 0);
        let info = HudInfo { fps: 60.0, hour: 12.0, ignored_total: 0, ignored_distinct: 0, ignored_top: Vec::new(), tables_error: None, ui_error: None };
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(SCREEN.0, SCREEN.1))),
            events: vec![egui::Event::PointerMoved(pointer)],
            ..Default::default()
        };
        ctx.begin_pass(input);
        ui::hud(ctx, &view, &world, &info, HudMode::Expanded);
        ctx.end_pass().textures_delta.clear();
    }

    fn actor(class: &str, location: [f32; 3], yaw: i32) -> Actor {
        Actor {
            class: class.to_string(),
            name: class.to_string(),
            tag: None,
            groups: Vec::new(),
            placement: Placement { location, rotation: [0, yaw, 0] },
            scale: [1.0; 3],
            draw_scale: 1.0,
            draw_scale_3d: [1.0; 3],
            static_mesh: None,
            skins: Vec::new(),
            unlit: false,
            sun_affect: false,
            special_lit: false,
            max_lights: 4,
            ambient_glow: 0,
            owner: None,
            use_owner_ambient: false,
            lighting: Vec::new(),
            movement: None,
            l2_movement: None,
            movement_tags: Vec::new(),
            movement_event: None,
            mover: None,
            server_id: None,
            hidden: false,
            is_static: true,
            ignored_range: false,
            time_reactor: None,
            l2_events: Vec::new(),
        }
    }

    #[test]
    fn get_ambient_lighting_actor_10338680_follows_owner_only_with_the_flag() {
        use killer_assets::scene::actor_lighting::actor_glow;
        let mut first = actor("StaticMeshActor", [0.0; 3], 0);
        first.name = "first".into();
        first.ambient_glow = 12;
        first.owner = Some("second".into());
        let mut second = first.clone();
        second.name = "second".into();
        second.owner = Some("third".into());
        second.use_owner_ambient = true;
        second.ambient_glow = 34;
        let mut third = first.clone();
        third.name = "third".into();
        third.owner = None;
        third.ambient_glow = 255;
        let mut level = Level { actors: vec![first.clone(), second, third], ..Default::default() };
        assert_eq!(actor_glow(&level, &first), 12);
        first.use_owner_ambient = true;
        assert_eq!(actor_glow(&level, &first), 255);
        level.actors[1].use_owner_ambient = false;
        assert_eq!(actor_glow(&level, &first), 34);
        first.owner = Some("absent".into());
        assert_eq!(actor_glow(&level, &first), 12);
        first.owner = None;
        assert_eq!(actor_glow(&level, &first), 12);
    }

    fn head() -> UserInfoHead {
        UserInfoHead {
            abnormal_effect: 0,
            x: 83400,
            y: 148000,
            z: -3400,
            heading: 16384,
            object_id: 1,
            name: "Wender".into(),
            race: 0,
            sex: 1,
            class_id: 109,
            level: 52,
            exp: 12345,
            max_hp: 900,
            hp: 400,
            max_mp: 300,
            mp: 200,
            run_speed: 117,
            walk_speed: 80,
            speed_multiplier: 1.5,
            attack_speed_multiplier: 1.0,
            pvp_flag: 0,
            karma: 0,
            collision_radius: 9.0,
            collision_height: 23.0,
            hair_style: 2,
            hair_color: 1,
            face: 3,
            builder_level: 0,
            title: "Titulo".into(),
            max_cp: 500,
            cp: 450,
            stats: UserStats::default(),
            paperdoll: [PaperdollSlot::default(); PAPERDOLL_SLOTS],
            inventory_limit: 80,
            name_color: 0xffffff,
            social: Default::default(),
            enchant_effect: 0,
        }
    }

    #[test]
    fn next_screen_follows_the_session() {
        let server = GameServer { id: 1, ip: [127, 0, 0, 1], port: 7777, players: 0, max_players: 1, online: true, extra: Default::default() };
        assert_eq!(next_screen(Screen::Login, &NetEvent::Connecting), Screen::Login);
        assert_eq!(next_screen(Screen::Login, &NetEvent::LoginOk(server)), Screen::Login);
        assert_eq!(next_screen(Screen::Login, &NetEvent::Session(SessionEvent::Characters(Vec::new()))), Screen::Characters);
        assert_eq!(next_screen(Screen::Characters, &NetEvent::Session(SessionEvent::Selected)), Screen::Characters);
        assert_eq!(next_screen(Screen::Characters, &NetEvent::Session(SessionEvent::UserInfo(Box::new(head())))), Screen::World);
        assert_eq!(next_screen(Screen::World, &NetEvent::Session(SessionEvent::UserInfo(Box::new(head())))), Screen::World);
        assert_eq!(next_screen(Screen::World, &NetEvent::Session(SessionEvent::Unknown { opcode: 0x7f, size: 4 })), Screen::World);
        assert_eq!(next_screen(Screen::World, &NetEvent::Session(SessionEvent::Malformed { opcode: 0x13, size: 4 })), Screen::World);
        assert_eq!(next_screen(Screen::World, &NetEvent::Session(SessionEvent::Characters(Vec::new()))), Screen::Characters);
        assert_eq!(next_screen(Screen::World, &NetEvent::Closed), Screen::Login);
        assert_eq!(next_screen(Screen::World, &NetEvent::Session(SessionEvent::ServerClosed)), Screen::World);
        assert_eq!(next_screen(Screen::Characters, &NetEvent::Session(SessionEvent::ServerClosed)), Screen::Login);
        assert_eq!(next_screen(Screen::Characters, &NetEvent::Closed), Screen::Login);
        for screen in [Screen::Login, Screen::Characters, Screen::World] {
            assert_eq!(next_screen(screen, &NetEvent::Error("x".into())), Screen::Login);
            assert_eq!(next_screen(screen, &NetEvent::Session(SessionEvent::Refused("x".into()))), Screen::Login);
        }
    }

    #[test]
    fn egui_only_claims_the_pointer_over_its_own_areas_not_over_the_background_it_registers() {
        let ctx = egui::Context::default();
        let press = mouse(ElementState::Pressed, MouseButton::Left);
        let wheel = WindowEvent::MouseWheel { device_id: DeviceId::dummy(), delta: MouseScrollDelta::LineDelta(0.0, 1.0), phase: TouchPhase::Moved };
        hud_pass(&ctx, egui::pos2(600.0, 240.0));
        assert!(ctx.egui_wants_pointer_input());
        assert!(!egui_consumes(&ctx, &press, ctx.egui_wants_pointer_input()));
        assert!(!egui_consumes(&ctx, &wheel, ctx.egui_wants_pointer_input()));
        assert!(egui_consumes(&ctx, &moved(1.0, 1.0), true));
        hud_pass(&ctx, egui::pos2(20.0, 100.0));
        assert!(egui_consumes(&ctx, &press, ctx.egui_wants_pointer_input()));
        assert!(egui_consumes(&ctx, &wheel, ctx.egui_wants_pointer_input()));
        hud_pass(&ctx, egui::pos2(SCREEN.0 - 20.0, SCREEN.1 / 2.0));
        assert!(!egui_consumes(&ctx, &press, ctx.egui_wants_pointer_input()));
    }

    #[test]
    fn dragging_with_one_button_left_or_right_turns_the_camera_and_the_wheel_zooms() {
        for button in [MouseButton::Left, MouseButton::Right] {
            let mut app = app_in_world();
            let start = app.world_view.as_ref().unwrap().camera;
            app.dispatch(moved(100.0, 100.0));
            app.dispatch(mouse(ElementState::Pressed, button));
            assert!(app.dragging);
            app.dispatch(moved(160.0, 130.0));
            let turned = app.world_view.as_ref().unwrap().camera;
            assert!(turned.yaw < start.yaw && turned.pitch > start.pitch, "{button:?}: {turned:?}");
            app.dispatch(mouse(ElementState::Released, button));
            assert!(!app.dragging);
            app.dispatch(moved(200.0, 130.0));
            assert_eq!(app.world_view.as_ref().unwrap().camera, turned);
            app.dispatch(WindowEvent::MouseWheel { device_id: DeviceId::dummy(), delta: MouseScrollDelta::LineDelta(0.0, 1.0), phase: TouchPhase::Moved });
            let zoomed = app.world_view.as_ref().unwrap().camera;
            assert_eq!((zoomed.zoom, zoomed.distance), (turned.zoom + 20.0, turned.distance + 20.0));
            app.dispatch(WindowEvent::MouseWheel { device_id: DeviceId::dummy(), delta: MouseScrollDelta::LineDelta(0.0, -1.0), phase: TouchPhase::Moved });
            assert_eq!(app.world_view.as_ref().unwrap().camera, turned);
        }
    }

    #[test]
    fn h_cycles_the_hud_collapsed_expanded_hidden() {
        let mut app = app_in_world();
        assert_eq!(app.hud, HudMode::Collapsed);
        app.key(KeyCode::KeyH, true, false);
        assert_eq!(app.hud, HudMode::Expanded);
        app.key(KeyCode::KeyH, true, true);
        app.key(KeyCode::KeyH, false, false);
        assert_eq!(app.hud, HudMode::Expanded);
        app.key(KeyCode::KeyH, true, false);
        assert_eq!(app.hud, HudMode::Hidden);
        app.key(KeyCode::KeyH, true, false);
        assert_eq!(app.hud, HudMode::Collapsed);
    }

    #[test]
    fn a_press_on_the_shortcut_grip_is_taken_by_the_game_ui_before_egui_and_moves_the_bar() {
        let mut app = app_in_world();
        app.game_ui = Some(ui2d::fixture_ui((1558, 933)));
        let press = mouse(ElementState::Pressed, MouseButton::Left);
        let release = mouse(ElementState::Released, MouseButton::Left);
        assert_eq!(app.game_ui_event(&moved(1500.0, 173.0)), KeyAction::Ignored);
        assert_eq!(app.game_ui_event(&press), KeyAction::Consumed);
        assert!(app.ui_captured);
        assert_eq!(app.game_ui_event(&moved(1400.0, 273.0)), KeyAction::Consumed);
        assert_eq!(app.game_ui_event(&release), KeyAction::Consumed);
        assert!(!app.ui_captured);
        let rect = app.game_ui.as_ref().unwrap().desktop.window(killer_ui::window::WindowId::SHORTCUT).unwrap().rect;
        assert_eq!((rect.x, rect.y), (1378, 268));
        assert_eq!(app.game_ui_event(&moved(1400.0, 400.0)), KeyAction::Ignored);
        assert_eq!(app.game_ui_event(&press), KeyAction::Consumed);
        assert_eq!(app.game_ui_event(&release), KeyAction::Consumed);
        assert_eq!(app.game_ui_event(&moved(600.0, 400.0)), KeyAction::Ignored);
        assert_eq!(app.game_ui_event(&press), KeyAction::Ignored);
        assert_eq!(app.game_ui_event(&release), KeyAction::Ignored);
    }

    #[test]
    fn a_second_click_on_the_same_npc_sends_the_action_again() {
        let mut world = World::default();
        assert_eq!(target_action(&mut world, 2, false, false), None);
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        assert_eq!(target_action(&mut world, 2, false, false), None);
        let mut gremlin = world.me.clone().unwrap();
        gremlin.object_id = 2;
        gremlin.kind = EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: false };
        world.entities.insert(2, gremlin);
        let expected = NetCommand::Action { object_id: 2, origin: [83400, 148000, -3400], shift: false };
        assert_eq!(target_action(&mut world, 2, false, false), Some(expected.clone()));
        assert_eq!(world.target, Some(2));
        assert_eq!(target_action(&mut world, 2, false, false), Some(expected));
        assert_eq!(target_action(&mut world, 2, false, true), Some(NetCommand::Action { object_id: 2, origin: [83400, 148000, -3400], shift: true }));
        assert_eq!(target_action(&mut world, 2, true, false), Some(NetCommand::AttackRequest { object_id: 2, origin: [83400, 148000, -3400], shift: false }));
        let mut player = world.me.clone().unwrap();
        player.object_id = 3;
        world.entities.insert(3, player);
        let action_on = |id: i32| NetCommand::Action { object_id: id, origin: [83400, 148000, -3400], shift: false };
        assert_eq!(target_action(&mut world, 3, false, false), Some(action_on(3)));
        assert_eq!(world.target, Some(3));
        assert_eq!(target_action(&mut world, 3, false, false), Some(action_on(3)));
        let me_id = world.me.as_ref().unwrap().object_id;
        assert_eq!(target_action(&mut world, me_id, false, false), Some(action_on(me_id)));
        assert_eq!(world.target, Some(me_id));
        assert_eq!(target_action(&mut world, me_id, false, false), Some(action_on(me_id)));
        assert!(world.me.as_ref().unwrap().dest.is_none());
        assert!(world.entities[&3].dest.is_none());
        world.me.as_mut().unwrap().sitting = true;
        assert_eq!(target_action(&mut world, 2, false, false), Some(NetCommand::Action { object_id: 2, origin: [83400, 148000, -3400], shift: false }));
        assert_eq!(target_action(&mut world, 2, true, false), None);
        assert!(!input::ground_click_allowed(&world, false, false));
        world.me.as_mut().unwrap().sitting = false;
        assert!(input::ground_click_allowed(&world, false, false));
        assert!(!input::ground_click_allowed(&world, true, false));
        assert!(!input::ground_click_allowed(&world, false, true));
        world.camera_fixed = true;
        assert!(!input::ground_click_allowed(&world, false, false));
        assert_eq!(target_action(&mut world, 3, false, false), None);
        assert_eq!(world.pending_click_target, Some(3));
        world.camera_fixed = false;
        world.me.as_mut().unwrap().dead = true;
        world.pending_click_target = None;
        assert_eq!(target_action(&mut world, 3, false, false), None);
        assert_eq!(world.pending_click_target, None);
        assert!(!input::ground_click_allowed(&world, false, false));
        world.me.as_mut().unwrap().dead = false;
        assert_eq!(world.target, Some(2));
    }

    #[test]
    fn confirming_a_destroy_or_crystallize_plays_trash_basket() {
        assert_eq!(input::confirm_sound(&NetCommand::DestroyItem { object_id: 1, count: 1 }), Some("itemsound.trash_basket"));
        assert_eq!(input::confirm_sound(&NetCommand::CrystallizeItem { object_id: 1, count: 1 }), Some("itemsound.trash_basket"));
        assert_eq!(input::confirm_sound(&NetCommand::TargetCancel), None);
    }

    #[test]
    fn releasing_on_a_link_of_the_npc_window_yields_a_bypass_and_the_wheel_scrolls_it() {
        let mut app = app_in_world();
        app.screen = Screen::World;
        app.game_ui = Some(ui2d::fixture_ui((1558, 933)));
        let html = include_str!("../../../crates/killer-ui/tests/fixtures/npc-7829.htm");
        let message = NpcHtmlMessage { npc_object_id: 2, html: html.to_string(), item_id: 0 };
        app.game_ui.as_mut().unwrap().open_npc_html(&message);
        let sell_y = app.game_ui.as_ref().unwrap().npc_viewer.as_ref().unwrap().lines.iter().find(|l| l.spans[0].text == "Sell item").unwrap().y;
        let (x, y) = (987.0 + 7.0 + 5.0, (494 + 30 + sell_y + 5) as f32);
        let press = mouse(ElementState::Pressed, MouseButton::Left);
        let release = mouse(ElementState::Released, MouseButton::Left);
        assert_eq!(app.game_ui_event(&moved(x, y)), KeyAction::Ignored);
        assert_eq!(app.game_ui_event(&press), KeyAction::Consumed);
        assert_eq!(app.game_ui_event(&release), KeyAction::Bypass("npc_%objectId%_Sell".into()));
        assert_eq!(app.game_ui_event(&release), KeyAction::Ignored);
        app.game_ui.as_mut().unwrap().open_npc_html(&message);
        let wheel = WindowEvent::MouseWheel { device_id: DeviceId::dummy(), delta: MouseScrollDelta::LineDelta(0.0, -1.0), phase: TouchPhase::Moved };
        assert_eq!(app.game_ui_event(&wheel), KeyAction::Consumed);
        assert_eq!(app.game_ui_event(&moved(300.0, 300.0)), KeyAction::Ignored);
        assert_eq!(app.game_ui_event(&wheel), KeyAction::Ignored);
    }

    #[test]
    fn an_ignored_npc_answer_becomes_a_system_line_so_the_click_is_not_silent() {
        let notice = input::npc_window_notice(0x11, 779).unwrap();
        assert_eq!(notice.kind, CHAT_SYSTEM);
        assert_eq!(notice.text, "o NPC respondeu com BuyList (0x11) (779 bytes): janela ainda não implementada");
        assert!(input::npc_window_notice(0x10, 5).unwrap().text.starts_with("o NPC respondeu com SellList (0x10)"));
        assert!(input::npc_window_notice(0xd0, 5).unwrap().text.contains("MultiSellList"));
        assert_eq!(input::npc_window_notice(0x2c, 5), None);
        assert_eq!(input::npc_window_notice(0xf3, 21), None);
    }

    #[test]
    fn first_player_start_wins_and_class_is_case_insensitive() {
        let level = Level { actors: vec![actor("StaticMeshActor", [0.0; 3], 1), actor("playerstart", [1.0, 2.0, 3.0], 16384), actor("PlayerStart", [9.0; 3], 0)], ..Default::default() };
        assert_eq!(player_start(&level), Some((Vec3::new(1.0, 2.0, 3.0), 16384)));
        assert_eq!(player_start(&Level::default()), None);
    }

    #[test]
    fn tile_center_is_the_middle_of_the_tile_named_by_tile_name() {
        let (x, y) = tile_center("22_22").unwrap();
        assert_eq!((x, y), (81920.0, 147456.0));
        assert_eq!(tile_name(x, y), "22_22");
        assert_eq!(tile_center("giran"), None);
    }

    #[test]
    #[ignore]
    fn a_pawn_dropped_over_a_water_volume_falls_swims_and_follows_its_destination() {
        use killer_assets::collision::{resources::Resources, transform, world::View};
        let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = Client::open(&root).unwrap();
        let mut resources = Resources::default();
        let mut found = None;
        let mut deepest = 0.0;
        for x in 15..=26 {
            for y in 10..=26 {
                let name = format!("{x}_{y}");
                let Ok(loaded) = client.load_level(&name) else { continue };
                let level = Arc::new(loaded.level);
                let tile = resources.tile(&mut client, &name, &level).unwrap();
                for volume in tile.volumes.iter().filter(|volume| volume.props.l2_water) {
                    let model = resources.model(&mut client, &volume.brush).unwrap();
                    let actor = &level.actors[volume.index];
                    let [lo, hi] = model.bounding_box;
                    let world = transform::actor(actor.placement, actor.scale, [0.0; 3]);
                    let corners: Vec<Vec3> = (0..8).map(|corner| world.transform_point3(Vec3::new(if corner & 1 == 0 { lo[0] } else { hi[0] }, if corner & 2 == 0 { lo[1] } else { hi[1] }, if corner & 4 == 0 { lo[2] } else { hi[2] }))).collect();
                    let center = corners.iter().sum::<Vec3>() / 8.0;
                    let top = corners.iter().map(|c| c.z).fold(f32::MIN, f32::max);
                    let depth = top - corners.iter().map(|c| c.z).fold(f32::MAX, f32::min);
                    if depth > 60.0 && depth < 400.0 && corners.iter().map(|c| (c.x - center.x).abs()).fold(0.0_f32, f32::max) > 80.0 && depth > deepest {
                        deepest = depth;
                        found = Some((name.clone(), level.clone(), center, top));
                    }
                }
            }
        }
        let (name, level, center, top) = found.expect("nenhum volume de água");
        let views = vec![View { name: name.clone(), level, poses: Default::default() }];
        let mut world = characters::demo_world(center, 0, 0);
        let me = world.me.as_mut().unwrap();
        me.position = Vec3::new(center.x, center.y, top - 100.0);
        me.collision_radius = 9.0;
        me.collision_height = 23.0;
        let mut state = crate::walk_world::WalkState::default();
        let mut swam = 0;
        for frame in 0..900 {
            let before = state.snapshot(&world);
            if frame == 400 && let Some(me) = world.me.as_mut() {
                me.dest = Some(me.position + Vec3::new(120.0, 0.0, 0.0));
                me.running = true;
            }
            world.tick(1.0 / 60.0);
            let (_, error) = crate::walk_world::walk_entities(&mut client, &mut resources, &mut state, &mut world, crate::walk_world::Frame { before: &before, views: &views, spawned: &[], dt: 1.0 / 60.0, options: &ClientOptions::default() });
            assert!(error.is_none(), "{error:?}");
            if state.is_swimming(world.me.as_ref().unwrap().object_id) { swam += 1; }
            if frame % 100 == 0 { eprintln!("quadro {frame}: {:?} nadando {}", world.me.as_ref().unwrap().position, state.is_swimming(world.me.as_ref().unwrap().object_id)); }
        }
        let me = world.me.as_ref().unwrap();
        eprintln!("água em {name}: topo {top:.0}, pawn terminou em {:?}, {swam} quadros nadando", me.position);
        assert!(swam > 100, "nunca nadou");
    }

    #[test]
    #[ignore]
    fn movement_smoothness_probe_in_giran() {
        let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let center = Vec3::new(81920.0, 147456.0, -3380.0);
        let loaded = load_tile(&root, "22_22", Some((center, 0)), false, None).unwrap();
        let mut client = Client::open(&root).unwrap();
        let mut resources = killer_assets::collision::resources::Resources::default();
        let views = vec![killer_assets::collision::world::View { name: "22_22".into(), level: loaded.level.clone(), poses: Default::default() }];
        {
            let mut fresh = killer_assets::collision::resources::Resources::default();
            let started = Instant::now();
            fresh.tile(&mut client, "22_22", &loaded.level).unwrap();
            eprintln!("custo Resources::tile 22_22 (frio): {:.1?}", started.elapsed());
            let started = Instant::now();
            let field = killer_assets::scene::decoration_points(&mut client, &loaded.level, &ClientOptions::default());
            eprintln!("custo decoration_points 22_22: {:.1?} ({} dentro de 100 u do centro)", started.elapsed(), field.count_within(81920.0, 147456.0, 100.0));
            let started = Instant::now();
            resources.absorb(fresh);
            eprintln!("custo absorb: {:.1?}, tile presente {}", started.elapsed(), resources.has_tile(&loaded.level));
        }
        let legs = [Vec3::new(0.0, 160.0, 0.0), Vec3::new(160.0, 0.0, 0.0), Vec3::new(0.0, -160.0, 0.0), Vec3::new(-160.0, 0.0, 0.0)];
        for (label, jitter, crowd) in [("dt fixo 16,7 ms", false, 0usize), ("dt 10 a 30 ms", true, 0), ("dt 10 a 30 ms com 40 pawns", true, 40)] {
            let mut world = characters::demo_world(center, 0, crowd);
            let mut state = crate::walk_world::WalkState::default();
            let mut seed = 12345u32;
            let mut leg = 0usize;
            let mut rows: Vec<(f32, Vec3, Vec3, bool)> = Vec::new();
            let mut costs: Vec<f32> = Vec::new();
            let mut previous = world.me.as_ref().unwrap().position;
            for frame in 0..1500 {
                let dt = if jitter {
                    seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                    0.010 + 0.020 * ((seed >> 8) as f32 / (1u32 << 24) as f32)
                } else {
                    1.0 / 60.0
                };
                if frame > 120 {
                    for entity in world.me.iter_mut().chain(world.entities.values_mut()) {
                        if entity.dest.is_none() || entity.blocked {
                            entity.dest = Some(entity.position + legs[leg % legs.len()]);
                            entity.blocked = false;
                            entity.running = true;
                            leg += 1;
                        }
                    }
                }
                let before = state.snapshot(&world);
                let started = Instant::now();
                world.tick(dt);
                let (_, error) = crate::walk_world::walk_entities(&mut client, &mut resources, &mut state, &mut world, crate::walk_world::Frame { before: &before, views: &views, spawned: &[], dt, options: &ClientOptions::default() });
                costs.push(started.elapsed().as_secs_f32() * 1000.0);
                assert!(error.is_none(), "{error:?}");
                let me = world.me.as_ref().unwrap();
                if frame > 125 {
                    let planar = (me.position - previous).truncate().length();
                    let ratio = planar / (me.speed() * dt);
                    if !me.blocked && planar > 0.0 && (ratio - 1.0).abs() > 0.25 || (me.position - previous).z.abs() > 3.0 {
                        eprintln!("  quadro {frame}: dt {dt:.4} razao XY {ratio:.2} dZ {:.2} bloqueado {} no chao {} caindo {} dest {:?}", (me.position - previous).z, me.blocked, state.is_grounded(me.object_id), state.is_falling(me.object_id), me.dest.map(|d| d - me.position));
                    }
                }
                rows.push((dt, me.position, me.position - previous, me.blocked));
                previous = me.position;
            }
            let speed = world.me.as_ref().unwrap().speed();
            let moving: Vec<_> = rows.iter().skip(125).filter(|row| !row.3 && row.2.truncate().length() > 0.0).collect();
            let ratios: Vec<f32> = moving.iter().map(|row| row.2.truncate().length() / (speed * row.0)).collect();
            let worst_xy = ratios.iter().fold(0.0f32, |acc, r| acc.max((r - 1.0).abs()));
            let mut zs: Vec<f32> = moving.iter().map(|row| row.2.z.abs()).collect();
            zs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let jumps = zs.iter().filter(|z| **z > 3.0).count();
            let mut sorted = costs[125..].to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            eprintln!("{label}: {} quadros andando livres, pior desvio XY {:.4}, dZ p50 {:.3} p99 {:.3} max {:.3}, quadros com |dZ|>3: {jumps}", moving.len(), worst_xy, zs[zs.len() / 2], zs[zs.len() * 99 / 100], zs[zs.len() - 1]);
            eprintln!("  custo de tick+fisica ms: p50 {:.3} p99 {:.3} max {:.3}", sorted[sorted.len() / 2], sorted[sorted.len() * 99 / 100], sorted[sorted.len() - 1]);
            for (index, row) in rows.iter().enumerate().skip(125).filter(|(_, row)| row.2.z.abs() > 3.0).take(6) {
                eprintln!("  dZ {:.2} no quadro {index} dt {:.4} em {:?}", row.2.z, row.0, row.1);
            }
            if crowd == 0 {
                assert!(worst_xy < 0.05, "{label}: passo XY fora de velocidade*dt ({worst_xy})");
            }
        }
    }

    #[test]
    #[ignore]
    fn walking_physics_is_stopped_by_another_pawn_cylinder() {
        use killer_assets::collision::world::Target;
        use killer_world::walk::{Walker, phys_walking};
        let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let center = Vec3::new(83400.0,148000.0,-3430.0);
        let loaded = load_tile(&root,"22_22",Some((center,0)),false,None).unwrap();
        let mut client = Client::open(&root).unwrap();
        let mut resources = killer_assets::collision::resources::Resources::default();
        let views = vec![killer_assets::collision::world::View { name:"22_22".into(),level:loaded.level.clone(),poses:Default::default() }];
        let mut sweep = crate::walk_world::WorldSweep::new(&mut client,&mut resources,&views);
        let start = center + Vec3::Z * 25.0;
        sweep.targets = vec![Target { object_id: 77, spawned: 0, position: start + Vec3::X * 100.0, radius: 20.0, height: 40.0 }];
        let mut walker = Walker::new(start, 9.0, 23.0);
        walker.on_world = true;
        for _ in 0..120 {
            walker.velocity = Vec3::X * 120.0;
            phys_walking(&mut sweep, &mut walker, 1.0 / 60.0);
        }
        assert!(sweep.error.is_none(), "{:?}", sweep.error);
        eprintln!("parou em x={:.1} (alvo em {:.1}, raios 9+20)", walker.location.x, start.x + 100.0);
        assert!(walker.location.x + 9.0 <= start.x + 100.0 - 20.0 + 0.5, "x={}", walker.location.x);
        assert!(walker.location.x > start.x + 40.0);
    }

    #[test]
    #[ignore]
    fn in_giran_the_walker_and_the_engine_camera_stay_inside_the_walls() {
        use killer_world::walk::{Outcome, Walker, phys_walking};
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let mut client = Client::open(&root).unwrap();
        let loaded = client.load_level("22_22").unwrap();
        let scene = build_scene_with(&mut client, &loaded.level, &ClientOptions::default(), Neighbours::default());
        let trace = TraceMesh::from_scene(&scene);
        let ground = GroundGrid::build(&scene);
        let settings = camera_settings(&mut client);
        let mut camera = initial_orbit(&mut client, &loaded.level, "22_22", &scene, &trace, settings, None);
        let views = vec![killer_assets::collision::world::View { name:"22_22".into(),level:Arc::new(loaded.level.clone()),poses:Default::default() }];
        let mut resources = killer_assets::collision::resources::Resources::default();
        let (target, yaw) = (camera.target, camera.yaw);
        let mut crossings = 0;
        for step in 0..36 {
            camera.yaw = yaw + (step as f32) * 10f32.to_radians();
            let engine = std::cell::RefCell::new((&mut client, &mut resources));
            camera.collide(|from, to| { let mut guard = engine.borrow_mut(); let (client, resources) = &mut *guard; crate::walk_world::camera_hit(client, resources, &views, &[], from, to, 0.1).unwrap() });
            let eye = camera.camera().eye;
            if trace.nearest_hit(target, eye).is_some() { crossings += 1; }
        }
        eprintln!("câmera do engine: {crossings}/36 olhos atrás de parede de render");
        let mut demo = characters::demo_world(target,0,0);
        let start = demo.me.as_mut().unwrap();
        start.position = target;
        start.collision_height = 23.0;
        start.collision_radius = 9.0;
        assert!(ground.place(start));
        let mut sweep = crate::walk_world::WorldSweep::new(&mut client,&mut resources,&views);
        let (mut steps, mut passes, mut falls) = (0, 0, 0);
        for direction in 0..16 {
            let angle = direction as f32 * std::f32::consts::TAU / 16.0;
            let mut walker = Walker::new(start.position, 9.0, 23.0);
            walker.on_world = true;
            for _ in 0..180 {
                walker.velocity = Vec3::new(angle.cos(), angle.sin(), 0.0) * 120.0;
                let before = walker.location;
                match phys_walking(&mut sweep, &mut walker, 1.0 / 60.0) {
                    Outcome::Walking => {}
                    _ => { falls += 1; walker.location = before; }
                }
                steps += 1;
                if trace.nearest_hit(before + Vec3::Z * 10.0, walker.location + Vec3::Z * 10.0).is_some() { passes += 1; }
            }
        }
        assert!(sweep.error.is_none(), "{:?}", sweep.error);
        eprintln!("giran: {steps} passos, {passes} atravessam parede de render, {falls} quedas");
    }

    #[test]
    #[ignore]
    fn in_giran_the_player_start_camera_views_230_and_never_crosses_a_wall_turning_360() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let mut client = Client::open(&root).unwrap();
        let loaded = client.load_level("22_22").unwrap();
        let scene = build_scene_with(&mut client, &loaded.level, &ClientOptions::default(), Neighbours::default());
        let trace = TraceMesh::from_scene(&scene);
        let settings = camera_settings(&mut client);
        assert!(settings.hit_check);
        let mut camera = initial_orbit(&mut client, &loaded.level, "22_22", &scene, &trace, settings, None);
        assert_eq!(camera.view_distance(), 230.0);
        assert!(camera.distance <= 200.0, "{}", camera.distance);
        let (target, yaw) = (camera.target, camera.yaw);
        let started = Instant::now();
        let mut placed = Vec::new();
        for step in 0..36 {
            camera.yaw = yaw + (step as f32) * 10f32.to_radians();
            camera.collide(|from, to| trace.nearest(from, to));
            let eye = camera.camera().eye;
            assert_eq!(trace.nearest_hit(target, eye), None, "passo {step}: olho {eye:?} a {:.1}", camera.distance);
            placed.push(camera.distance);
        }
        let per_trace = started.elapsed() / 72;
        println!("giran: {} triângulos, {per_trace:?} por traço, distâncias {placed:.0?}", trace.triangles());
        if !cfg!(debug_assertions) {
            assert!(per_trace < Duration::from_micros(300), "{per_trace:?}");
        }
        assert!(placed.iter().any(|&d| d < 200.0) && placed.contains(&200.0), "{placed:?}");
        camera.settings = CameraSettings { hit_check: false, ..settings };
        let mut looped = Vec::new();
        for step in 0..36 {
            camera.yaw = yaw + (step as f32) * 10f32.to_radians();
            camera.collide(|from, to| trace.nearest(from, to));
            let eye = camera.camera().eye;
            assert_eq!(trace.nearest_hit(target, eye), None, "passo {step} sem HitCheck: olho {eye:?} a {:.1}", camera.distance);
            looped.push(camera.distance);
        }
        println!("giran sem HitCheck (mínimo {}): distâncias {looped:.0?}", settings.hit_check_min_dist);
        assert!(looped.iter().zip(&placed).all(|(l, p)| l <= p), "{looped:?}");
    }

    #[test]
    fn the_capture_goes_to_pictures_killer_with_the_unix_time_and_the_version_comes_from_the_build() {
        let now = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let expected = PathBuf::from("/home/x/Pictures/killer/killer-1700000000.png");
        assert_eq!(capture_path(Some(PathBuf::from("/home/x")), now), expected);
        assert_eq!(capture_path(None, UNIX_EPOCH), PathBuf::from("Pictures/killer/killer-0.png"));
        assert!(!version().is_empty());
    }

    const BENCH_FRAMES: u32 = 360;
    const BENCH_CROWD: usize = 40;

    fn logged_field(line: &str, name: &str) -> i32 {
        let start = line.find(&format!("{name}: ")).map(|at| at + name.len() + 2).unwrap();
        let digits: String = line[start..].chars().take_while(|c| c.is_ascii_digit() || *c == '-').collect();
        digits.parse().unwrap()
    }

    fn logged_kinds(log: &str) -> Vec<EntityKind> {
        let mut kinds = Vec::new();
        for line in log.lines().filter(|line| line.contains("] personagem ") && line.ends_with("carregado")) {
            let mut equipped = [0; VISIBLE_EQUIP_SLOTS];
            for (slot, name) in [(1, "head"), (2, "rhand"), (3, "lhand"), (4, "gloves"), (5, "chest"), (6, "legs"), (7, "feet"), (8, "back"), (9, "lrhand"), (10, "hair")] {
                equipped[slot] = logged_field(line, name);
            }
            if line.contains("personagem Npc") {
                kinds.push(EntityKind::Npc { npc_id: logged_field(line, "npc_id"), rhand: equipped[2], chest: equipped[5], lhand: equipped[3], attackable: true });
            } else {
                let row = logged_field(line, "row") as usize;
                let (race, class_id) = match row / 2 * 2 {
                    0 => (0, 0),
                    2 => (2, 0),
                    4 => (4, 0),
                    6 => (1, 0),
                    8 => (0, 10),
                    10 => (3, 44),
                    _ => (3, 49),
                };
                kinds.push(EntityKind::Player {
                    class_id,
                    race,
                    sex: (row % 2) as i32,
                    equipped,
                    hair_style: logged_field(line, "hair_style"),
                    hair_color: logged_field(line, "hair_color"),
                    face: logged_field(line, "face"),
                    enchant_effect: 0,
                    hero: false,
                });
            }
        }
        kinds
    }

    #[test]
    #[ignore]
    fn ground_items_follow_packets_render_and_release_original_resources() {
        let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let size = (1280,720);
        let center = Vec3::new(83400.0,148000.0,-3430.0);
        let mut loaded = load_tile(&root,"22_22",Some((center,0)),true,None).unwrap();
        let mut headless = killer_render::headless::Headless::new(size.0,size.1).unwrap();
        let mut client = Client::open(&root).unwrap();
        headless.upload(&loaded.scene,&mut BankSource { bank:&mut loaded.bank,catalog:client.catalog() });
        let tables = loaded.tables.take().unwrap().unwrap();
        let mut view = WorldView {
            moving_trace:crate::trace::MovingTrace::default(),tile:loaded.tile.clone(),trace:TraceMesh::from_scene(&loaded.scene),
            display:loaded.display,zones:loaded.scene.zones.len(),lighting:LightMap::new(&loaded.level,&loaded.scene.lights),
            camera:loaded.camera,free:FreeCamera::from_orbit(&loaded.camera),use_free:false,timeenv:loaded.weather.base().clone(),weather:loaded.weather,
            bsp_tree:loaded.bsp_tree,ground:loaded.ground,shakes:Default::default(),fade:Default::default(),fade_shown:false,draws:loaded.scene.draws.len(),triangles:0,characters:0,instances:0,
            neighbours:NeighbourSet::new(TileId { x:22,y:22 },loaded.level.clone()),options:loaded.options,
        };
        view.camera.target = center;
        view.camera.distance = 320.0;
        view.camera.pitch = 0.65;
        let mut world = characters::demo_world(center+Vec3::new(-90.0,0.0,30.0),0,0);
        let source = world.me.as_ref().unwrap().object_id;
        let ids = [57,1,18,1299,21,10026];
        for (index,item_id) in ids.into_iter().enumerate() {
            let x = center.x as i32+(index as i32%3-1)*65;
            let y = center.y as i32+(index as i32/3)*65;
            world.apply_world(&WorldEvent::DropItem(l2_net::game::DropItem {
                player_id:source,object_id:9000+index as i32,item_id,x,y,z:center.z as i32,stackable:true,count:1,heading:0,
            }));
        }
        let mut items = crate::items::Items::default();
        let mut shared = crate::loader::Shared::default();
        let (sender,requests) = std::sync::mpsc::channel();
        for _ in 0..ids.len()*2 {
            items.sync(&world,headless.renderer_mut());
            items.request(&tables,&view,headless.renderer_mut(),&sender);
            let batch: Vec<_> = requests.try_iter().collect();
            if batch.is_empty() { break; }
            assert!(batch.len() <= 2);
            for request in batch {
                match crate::loader::handle(&mut client,&mut shared,request) {
                    LoadResult::Item { item_id,asset } => items.install_asset(item_id,asset,headless.renderer_mut(),&mut client),
                    LoadResult::PlaceItem { ticket,pieces } => items.install_placement(ticket,pieces),
                    _ => panic!("resultado não relacionado ao item"),
                }
            }
            client.catalog().release_files();
        }
        let environment = Environment::from_time(&view.timeenv,12.0);
        let mut rendered = Vec::new();
        let mut sounds = Vec::new();
        for frame in 0..=120 {
            rendered = items.instances(1.0/60.0,&view,headless.renderer_mut(),Some(&mut client));
            sounds.extend(items.take_sounds());
            headless.renderer_mut().set_time(frame as f32/60.0);
            headless.renderer_mut().set_instances(&rendered);
            if [0,30,120].contains(&frame) {
                let rgba = headless.render(&view.current(),&environment).unwrap();
                let path = PathBuf::from(format!("/tmp/killer-ground-items-{frame}.png"));
                write_png(&path,size.0,size.1,&rgba).unwrap();
                eprintln!("captura {}: {} instâncias",path.display(),rendered.len());
            }
        }
        let mut events = std::collections::HashSet::new();
        for sound in &sounds {
            assert!(events.insert((sound.actor,sound.sound.name.clone())),"som repetido sem novo pacote");
            assert!(items.audio_positions(&world).contains_key(&sound.actor));
            assert_eq!(sound.volume,if sound.sound.name == killer_assets::pickup::drop::THROW_SOUND { 0.5 } else { 1.0 });
        }
        assert_eq!(sounds.iter().filter(|sound| sound.volume == 1.0).count(),ids.len());
        assert!(sounds.iter().any(|sound| sound.volume == 0.5));
        eprintln!("{} sons de lançamento/queda sem repetição",sounds.len());
        world.me.as_mut().unwrap().position += Vec3::Y*300.0;
        for (index,item_id) in ids.into_iter().enumerate() {
            assert!(rendered.iter().any(|instance| instance.actor_id == 9000+index as i32),"item {item_id} não apareceu");
            let actor = rendered.iter().find(|instance| instance.actor_id == 9000+index as i32).unwrap();
            let point = actor.actor_position+Vec3::Z*actor.collision_height;
            let pixel = characters::project(&view.current(),(size.0 as f32,size.1 as f32),point).unwrap();
            let (origin,dir) = input::pixel_ray(&view.current(),size,(pixel[0],pixel[1])).unwrap();
            let request = crate::loader::PickRequest {
                views:crate::items::views(&view,Some(headless.renderer_mut()),origin),
                query:killer_assets::pickup::TraceQuery { start:origin,end:origin+dir*10000.0,extent:Vec3::splat(0.1),flags:0x200bf },
                targets:items.targets(&world),
                spawned:Vec::new(),
            };
            let LoadResult::Pick { selection,.. } = crate::loader::handle(&mut client,&mut shared,LoadRequest::Pick { ticket:index as u64,request }) else { panic!("resultado de seleção"); };
            let selected = selection.unwrap().unwrap();
            assert_eq!(selected.target.map(|target| target.object_id),Some(actor.actor_id),"item {item_id}: {selected:?}");
            assert!(matches!(target_action(&mut world,actor.actor_id,false,false),Some(NetCommand::Action { object_id,.. }) if object_id == actor.actor_id));
            let pieces = items.targets_with_pieces(&world);
            let request = crate::loader::PickRequest {
                views:crate::items::views(&view,Some(headless.renderer_mut()),origin),
                query:killer_assets::pickup::TraceQuery { start:origin,end:origin+dir*10000.0,extent:Vec3::splat(0.1),flags:0x200bf },
                targets:pieces.iter().map(|(target,_)| *target).collect(),
                spawned:Vec::new(),
            };
            let LoadResult::Hover { selection,.. } = crate::loader::handle(&mut client,&mut shared,LoadRequest::Hover { ticket:index as u64,request }) else { panic!("resultado do mouse"); };
            let hovered = selection.unwrap().unwrap().target.unwrap();
            let slot = pieces.iter().find(|(piece,_)| same_target(piece,&hovered)).and_then(|(_,slot)| *slot).unwrap();
            let (location,text) = items.label(&world,&tables,hovered.object_id,hovered.spawned,slot).unwrap();
            let name = &tables.itemname[&(item_id as u32)].name;
            assert_eq!(text,format!("{name}(1)"));
            assert_eq!([location.x,location.y],[hovered.position.x,hovered.position.y]);
            assert!(characters::project(&view.current(),(size.0 as f32,size.1 as f32),location).is_some());
            eprintln!("rótulo do item {item_id}: {text} em {location}");
            world.apply_world(&WorldEvent::GetItem(l2_net::game::GetItem { player_id:source,object_id:9000+index as i32,x:0,y:0,z:0 }));
            items.sync(&world,headless.renderer_mut());
            assert!(!items.targets(&world).iter().any(|target| target.object_id == actor.actor_id));
        }
        let buried = killer_assets::collision::world::Target { object_id:9100,spawned:0,position:center-Vec3::Z*80.0,radius:5.0,height:10.0 };
        let pixel = characters::project(&view.current(),(size.0 as f32,size.1 as f32),buried.position).unwrap();
        let (origin,dir) = input::pixel_ray(&view.current(),size,(pixel[0],pixel[1])).unwrap();
        let request = crate::loader::PickRequest {
            views:crate::items::views(&view,Some(headless.renderer_mut()),origin),
            query:killer_assets::pickup::TraceQuery { start:origin,end:origin+dir*10000.0,extent:Vec3::splat(0.1),flags:0x200bf },
            targets:vec![buried],
            spawned:Vec::new(),
        };
        let LoadResult::Pick { selection,.. } = crate::loader::handle(&mut client,&mut shared,LoadRequest::Pick { ticket:99,request }) else { panic!("resultado de seleção"); };
        let blocked = selection.unwrap().expect("o raio precisa parar no cenário");
        assert!(blocked.target.is_none(),"alvo atrás do cenário selecionado: {blocked:?}");
        assert!(blocked.hit.position.z > buried.position.z+buried.height,"{blocked:?}");
        items.sync(&world,headless.renderer_mut());
        assert!(items.instances(0.0,&view,headless.renderer_mut(),Some(&mut client)).is_empty());
        assert!(items.audio_positions(&world).is_empty());
        assert!(items.take_sounds().is_empty());
        drop(sounds);
        assert!(shared.sounds.values().all(|sound| sound.strong_count() == 0));
        assert!(shared.collision.cached_bytes() <= 64*1024*1024);
    }

    #[test]
    #[ignore]
    fn crowd_frame_stages_headless() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let crowd = std::env::var("KILLER_CROWD").ok().and_then(|n| n.parse().ok()).unwrap_or(BENCH_CROWD);
        let size = (1280, 720);
        let mut loaded = load_tile(&root, "23_24", None, true, Some((size.0 as i32, size.1 as i32))).unwrap();
        let mut headless = killer_render::headless::Headless::new(size.0, size.1).unwrap();
        let mut client = Client::open(&root).unwrap();
        headless.upload(&loaded.scene, &mut BankSource { bank: &mut loaded.bank, catalog: client.catalog() });
        let tables = loaded.tables.take().unwrap().unwrap();
        let mut ui = loaded.ui.take().unwrap().unwrap();
        ui.resized((size.0 as i32, size.1 as i32));
        ui.set_tab_names(&tables);
        let mut world = characters::demo_world(loaded.camera.target, loaded.camera.facing(), crowd);
        if let Ok(log) = std::env::var("KILLER_CROWD_LOG") {
            let kinds = logged_kinds(&std::fs::read_to_string(log).unwrap());
            let count = kinds.len();
            for (index, kind) in kinds.into_iter().enumerate() {
                let angle = std::f32::consts::TAU * index as f32 / count as f32;
                let radius = 60.0 + 1400.0 * (index % 7) as f32 / 7.0;
                let position = loaded.camera.target + Vec3::new(angle.cos(), angle.sin(), 0.0) * radius;
                let mut entity = world.me.clone().unwrap();
                entity.object_id = 1000 + index as i32;
                entity.kind = kind;
                entity.name = format!("Entidade{index}");
                entity.position = position;
                world.entities.insert(entity.object_id, entity);
            }
        }
        for entity in world.me.iter_mut().chain(world.entities.values_mut()) {
            loaded.ground.place(entity);
        }
        let mut view = WorldView {
            moving_trace: crate::trace::MovingTrace::default(),
            tile: loaded.tile.clone(),
            trace: TraceMesh::from_scene(&loaded.scene),
            display: loaded.display,
            zones: loaded.scene.zones.len(),
            lighting: LightMap::new(&loaded.level, &loaded.scene.lights),
            camera: loaded.camera,
            free: FreeCamera::from_orbit(&loaded.camera),
            use_free: false,
            timeenv: loaded.weather.base().clone(),
            weather: loaded.weather,
            bsp_tree: loaded.bsp_tree,
            ground: loaded.ground,
            shakes: Default::default(),
            fade: Default::default(),
            fade_shown: false,
            draws: loaded.scene.draws.len(),
            triangles: 0,
            characters: 0,
            instances: 0,
            neighbours: NeighbourSet::new(TileId { x: 22, y: 22 }, loaded.level.clone()),
            options: loaded.options.clone(),
        };
        let mut cache = CharacterCache::default();
        let mut shared = crate::loader::Shared::default();
        let display = view.display;
        let frame = |world: &World, view: &WorldView, dt: f32| Frame { display, viewer: viewer_position(world, &view.camera), dt };
        let first = frame(&world, &view, 0.0);
        characters::instances(&mut world, &mut cache, &tables, &first, |p| view.zone_at(p), |query| view.actor_lighting(query), |_, _| true);
        for key in cache.take_wanted(usize::MAX) {
            let look = characters::look(key, &tables).unwrap();
            match crate::loader::load_character(&mut client, &mut shared, key, &look.appearance, look.draw_scale) {
                LoadResult::Character { parts, animation, draw_scale, weapon_type, mut bank, .. } => {
                    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
                    let parts = parts.into_iter().map(|mesh| (headless.renderer_mut().upload_character(&mesh, &mut source), Arc::new(mesh))).collect();
                    cache.insert(key, LoadedCharacter::new(parts, animation, draw_scale, weapon_type).unwrap());
                }
                LoadResult::Failed { reason, .. } => {
                    eprintln!("{key:?}: {reason}");
                    cache.fail(key);
                }
                _ => panic!("resultado não relacionado ao personagem"),
            }
        }
        client.catalog().release_files();
        eprintln!("{} personagens carregados, {} entidades, texturas na GPU {}", cache.loaded_count(), world.entities.len() + 1, headless.renderer_mut().texture_count());
        let mut chat = ChatLog::default();
        for line in 0..20 {
            chat.push(ChatLine { kind: CHAT_ALL, sender: format!("Killer{line}"), text: "olá mundo, esta é uma linha de conversa".to_string(), group: 0, color: None, system: None, tables: None });
        }
        let mut profile = FrameProfile::new(Instant::now());
        let mut report = None;
        let mut frames = 0;
        while frames < BENCH_FRAMES {
            profile.begin(Instant::now());
            pace::run(&mut world, Duration::from_millis(16));
            
            view.camera.follow(world.me.as_ref().unwrap().position);
            profile.lap("tick", Instant::now());
            let frustum = view.current().frustum(size.0 as f32 / size.1 as f32);
            let current = frame(&world, &view, 0.016);
            let instances = characters::instances(&mut world, &mut cache, &tables, &current, |p| view.zone_at(p), |query| view.actor_lighting(query), |c, r| frustum.contains_sphere(c, r));
            profile.lap("instâncias", Instant::now());
            headless.renderer_mut().set_instances(&instances);
            profile.lap("set_instances", Instant::now());
            let camera = view.current();
            let labels = characters::labels(&world, &tables, &cache, &current, &camera, (size.0 as f32, size.1 as f32), None);
            let lines: Vec<ChatEntry> = chat.iter().map(ui2d::chat_entry).collect();
            let target = world.targeted().map(|entity| ui2d::target_view(&world, entity, characters::display_name(entity, &tables), Some(&tables))).or_else(|| ui2d::static_target_view(&world, Some(&tables)));
            profile.lap("rótulos", Instant::now());
            let views = ui2d::world_views(&world, Some(&tables), Instant::now());
            ui.desktop.raise_newly_shown();
            let list = ui.frame(&world, &labels, &lines, target.as_ref(), &views);
            profile.lap("ui", Instant::now());
            headless.renderer_mut().set_ui(&list, &mut BankSource { bank: &mut loaded.bank, catalog: client.catalog() });
            profile.lap("set_ui", Instant::now());
            let environment = Environment::from_time(&view.timeenv, DEFAULT_HOUR);
            let (color, depth) = headless.targets();
            let renderer = headless.renderer_mut();
            let mut encoder = renderer.device().create_command_encoder(&Default::default());
            renderer.render(&mut encoder, &color, &depth, size, &camera, &environment);
            profile.lap("render", Instant::now());
            renderer.queue().submit([encoder.finish()]);
            profile.lap("submit", Instant::now());
            renderer.device().poll(wgpu::PollType::wait_indefinitely()).unwrap();
            profile.lap("gpu", Instant::now());
            frames += 1;
            if let Some(line) = profile.end(Instant::now()) {
                let ms = |line: &str| line.split(' ').next().and_then(|ms| ms.parse::<f32>().ok()).unwrap_or(f32::MAX);
                if report.as_deref().is_none_or(|best| ms(&line) < ms(best)) {
                    report = Some(line);
                }
            }
        }
        let report = report.unwrap();
        let rss = std::process::Command::new("ps").args(["-o", "rss=", "-p", &std::process::id().to_string()]).output().unwrap();
        let visible = headless.renderer_mut().visible_draw_count(&view.current(), size.0 as f32 / size.1 as f32);
        eprintln!("{visible} draws, {} instâncias: {report} | rss {} MB", headless.renderer_mut().instance_count(), String::from_utf8_lossy(&rss.stdout).trim().parse::<u64>().unwrap_or(0) / 1024);
    }

    fn process_rss_mb() -> u64 {
        let pid = std::process::id().to_string();
        let output = std::process::Command::new("vmmap").args(["-summary", &pid]).output().unwrap();
        let text = String::from_utf8_lossy(&output.stdout);
        let line = text.lines().find(|line| line.starts_with("Physical footprint:")).unwrap_or("");
        let value = line.trim_start_matches("Physical footprint:").trim();
        let number: f64 = value.trim_end_matches(|c: char| c.is_ascii_alphabetic()).parse().unwrap_or(0.0);
        match value.chars().last() {
            Some('G') => (number * 1024.0) as u64,
            Some('M') => number as u64,
            Some('K') => (number / 1024.0) as u64,
            _ => 0,
        }
    }

    #[test]
    #[ignore]
    fn memory_churn_headless() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let cycles: u32 = std::env::var("KILLER_MEM_CYCLES").ok().and_then(|n| n.parse().ok()).unwrap_or(30);
        let frames_per_cycle: u32 = std::env::var("KILLER_MEM_FRAMES").ok().and_then(|n| n.parse().ok()).unwrap_or(40);
        let crowd: usize = std::env::var("KILLER_MEM_CROWD").ok().and_then(|n| n.parse().ok()).unwrap_or(40);
        let size = (1280, 720);
        eprintln!("rss inicial {} MB", process_rss_mb());
        let mut loaded = load_tile(&root, "23_24", None, true, Some((size.0 as i32, size.1 as i32))).unwrap();
        eprintln!("rss após load_tile {} MB", process_rss_mb());
        let mut headless = killer_render::headless::Headless::new(size.0, size.1).unwrap();
        let mut client = Client::open(&root).unwrap();
        headless.upload(&loaded.scene, &mut BankSource { bank: &mut loaded.bank, catalog: client.catalog() });
        eprintln!("rss após upload da cena {} MB", process_rss_mb());
        let tables = loaded.tables.take().unwrap().unwrap();
        let mut ui = loaded.ui.take().unwrap().unwrap();
        ui.resized((size.0 as i32, size.1 as i32));
        ui.set_tab_names(&tables);
        let mut world = characters::demo_world(loaded.camera.target, loaded.camera.facing(), crowd);
        for entity in world.me.iter_mut().chain(world.entities.values_mut()) {
            loaded.ground.place(entity);
        }
        let templates: Vec<killer_world::Entity> = world.entities.values().cloned().collect();
        let mut view = WorldView {
            moving_trace: crate::trace::MovingTrace::default(),
            tile: loaded.tile.clone(),
            trace: TraceMesh::from_scene(&loaded.scene),
            display: loaded.display,
            zones: loaded.scene.zones.len(),
            lighting: LightMap::new(&loaded.level, &loaded.scene.lights),
            camera: loaded.camera,
            free: FreeCamera::from_orbit(&loaded.camera),
            use_free: false,
            timeenv: loaded.weather.base().clone(),
            weather: loaded.weather,
            bsp_tree: loaded.bsp_tree,
            ground: loaded.ground,
            shakes: Default::default(),
            fade: Default::default(),
            fade_shown: false,
            draws: loaded.scene.draws.len(),
            triangles: 0,
            characters: 0,
            instances: 0,
            neighbours: NeighbourSet::new(TileId { x: 22, y: 22 }, loaded.level.clone()),
            options: loaded.options.clone(),
        };
        let mut cache = CharacterCache::default();
        let mut shared = crate::loader::Shared::default();
        let display = view.display;
        let mut chat = ChatLog::default();
        let mut previous_tile: Option<String> = None;
        let mut serial = 10_000;
        eprintln!("rss antes dos ciclos {} MB", process_rss_mb());
        for cycle in 0..cycles {
            world.entities.clear();
            for (index, template) in templates.iter().enumerate() {
                let mut entity = template.clone();
                serial += 1;
                entity.object_id = serial;
                if let EntityKind::Player { hair_style, hair_color, face, .. } = &mut entity.kind {
                    *hair_style = ((cycle as usize + index) % 5) as i32;
                    *hair_color = ((cycle as usize * 3 + index) % 4) as i32;
                    *face = ((cycle as usize + index * 2) % 3) as i32;
                }
                let angle = std::f32::consts::TAU * ((index as f32 + cycle as f32 * 0.37) / templates.len().max(1) as f32);
                entity.position = loaded.camera.target + Vec3::new(angle.cos(), angle.sin(), 0.0) * (200.0 + 40.0 * (index % 9) as f32);
                entity.dest = Some(entity.position + Vec3::new(angle.sin(), angle.cos(), 0.0) * 300.0);
                view.ground.place(&mut entity);
                world.entities.insert(entity.object_id, entity);
            }
            let name = format!("memtile{}", cycle % 2);
            let mut source = BankSource { bank: &mut loaded.bank, catalog: client.catalog() };
            headless.renderer_mut().add_tile(&name, &loaded.scene, &mut source);
            if let Some(old) = previous_tile.replace(name) {
                headless.renderer_mut().remove_tile(&old);
            }
            for frame_index in 0..frames_per_cycle {
                pace::run(&mut world, Duration::from_millis(16));
                view.camera.follow(world.me.as_ref().unwrap().position);
                let frustum = view.current().frustum(size.0 as f32 / size.1 as f32);
                let current = Frame { display, viewer: viewer_position(&world, &view.camera), dt: 0.016 };
                let drawn: std::collections::HashSet<i32> = characters::drawn_pawns(&world, &tables, &current).into_iter().collect();
                world.rendered.clone_from(&drawn);
                for character in cache.evict_drawn(&world, &drawn) {
                    headless.renderer_mut().remove_character(character);
                }
                let instances = characters::instances_drawn(&mut world, &mut cache, &tables, &current, |p| view.zone_at(p), |query| view.actor_lighting(query), |c, r| frustum.contains_sphere(c, r), |_| None, &drawn);
                for key in cache.take_wanted(4) {
                    let look = characters::look(key, &tables).unwrap();
                    match crate::loader::load_character(&mut client, &mut shared, key, &look.appearance, look.draw_scale) {
                        LoadResult::Character { parts, animation, draw_scale, weapon_type, mut bank, .. } => {
                            let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
                            let parts = parts.into_iter().map(|mesh| (headless.renderer_mut().upload_character(&mesh, &mut source), Arc::new(mesh))).collect();
                            cache.insert(key, LoadedCharacter::new(parts, animation, draw_scale, weapon_type).unwrap());
                        }
                        LoadResult::Failed { .. } => cache.fail(key),
                        _ => panic!("resultado não relacionado ao personagem"),
                    }
                }
                client.catalog().release_files();
                headless.renderer_mut().set_instances(&instances);
                chat.push(ChatLine { kind: CHAT_ALL, sender: format!("Killer{frame_index}"), text: "linha de conversa".to_string(), group: 0, color: None, system: None, tables: None });
                let camera = view.current();
                let labels = characters::labels(&world, &tables, &cache, &current, &camera, (size.0 as f32, size.1 as f32), None);
                let lines: Vec<ChatEntry> = chat.iter().map(ui2d::chat_entry).collect();
                let views = ui2d::world_views(&world, Some(&tables), Instant::now());
                ui.desktop.raise_newly_shown();
                let list = ui.frame(&world, &labels, &lines, None, &views);
                headless.renderer_mut().set_ui(&list, &mut BankSource { bank: &mut loaded.bank, catalog: client.catalog() });
                let environment = Environment::from_time(&view.timeenv, DEFAULT_HOUR);
                let (color, depth) = headless.targets();
                let renderer = headless.renderer_mut();
                let mut encoder = renderer.device().create_command_encoder(&Default::default());
                renderer.render(&mut encoder, &color, &depth, size, &camera, &environment);
                renderer.queue().submit([encoder.finish()]);
                renderer.device().poll(wgpu::PollType::wait_indefinitely()).unwrap();
            }
            eprintln!("ciclo {cycle}: rss {} MB, texturas {}, entidades {}, bancos {} MB | {}", process_rss_mb(), headless.renderer_mut().texture_count(), world.entities.len(), loaded.bank.cached_bytes() >> 20, cache.retained_sizes());
        }
    }

    #[test]
    #[ignore]
    fn memory_tile_cycle_headless() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let steps: usize = std::env::var("KILLER_MEM_CYCLES").ok().and_then(|n| n.parse().ok()).unwrap_or(16);
        let mut headless = killer_render::headless::Headless::new(640, 360).unwrap();
        let mut uploader = Client::open(&root).unwrap();
        let options = ClientOptions::load(&mut uploader).unwrap_or_default();
        let ring: Vec<(i32, i32)> = match std::env::var("KILLER_MEM_RING") { Ok(spec) => spec.split(',').filter_map(|part| part.split_once('_')).filter_map(|(x, y)| Some((x.parse().ok()?, y.parse().ok()?))).collect(), Err(_) => vec![(22, 22), (23, 22), (23, 23), (22, 23), (21, 23), (21, 22)] };
        let mut held: std::collections::VecDeque<crate::tiles::NeighbourTile> = Default::default();
        eprintln!("tiles: inicial {} MB", process_rss_mb());
        for step in 0..steps {
            let (x, y) = ring[step % ring.len()];
            let id = TileId { x, y };
            match crate::tiles::load_neighbour(&root, id, &options, [None, None, None]) {
                Ok(mut loaded) => {
                    let mode = std::env::var("KILLER_MEM_MODE").unwrap_or_default();
                    if mode != "load" {
                        let mut source = BankSource { bank: &mut loaded.bank, catalog: uploader.catalog() };
                        headless.renderer_mut().add_tile(&loaded.tile.name, &loaded.scene, &mut source);
                        uploader.catalog().release_files();
                        {
                            let renderer = headless.renderer_mut();
                            let encoder = renderer.device().create_command_encoder(&Default::default());
                            renderer.queue().submit([encoder.finish()]);
                        }
                        if mode == "gpu" {
                            headless.renderer_mut().remove_tile(&loaded.tile.name);
                            headless.renderer_mut().device().poll(wgpu::PollType::wait_indefinitely()).unwrap();
                        } else {
                            let crate::tiles::LoadedNeighbour { tile, scene, bank, .. } = loaded;
                            let with_scene = process_rss_mb();
                            drop(scene);
                            let without_scene = process_rss_mb();
                            drop(bank);
                            let without_bank = process_rss_mb();
                            eprintln!("tile {}: com cena {with_scene} MB, sem cena {without_scene} MB, sem banco {without_bank} MB", tile.name);
                            if mode == "probe" {
                                let crate::tiles::NeighbourTile { name, trace, ground, bsp_tree, zones: _, lighting, level, .. } = tile;
                                eprintln!("sonda {name}: inteiro {} MB", process_rss_mb());
                                drop(trace);
                                eprintln!("sonda: sem trace {} MB", process_rss_mb());
                                drop(ground);
                                eprintln!("sonda: sem ground {} MB", process_rss_mb());
                                drop(lighting);
                                eprintln!("sonda: sem lighting {} MB", process_rss_mb());
                                drop(bsp_tree);
                                eprintln!("sonda: sem bsp {} MB", process_rss_mb());
                                drop(level);
                                eprintln!("sonda: sem level {} MB", process_rss_mb());
                                continue;
                            }
                            held.push_back(tile);
                        }
                        let renderer = headless.renderer_mut();
                        let encoder = renderer.device().create_command_encoder(&Default::default());
                        renderer.queue().submit([encoder.finish()]);
                        renderer.device().poll(wgpu::PollType::wait_indefinitely()).unwrap();
                    }
                }
                Err(error) => eprintln!("tile {} falhou: {error}", id.name()),
            }
            while held.len() > 3 {
                if let Some(old) = held.pop_front() {
                    headless.renderer_mut().remove_tile(&old.name);
                }
            }
            let gpu = std::process::Command::new("vmmap").args(["-summary", &std::process::id().to_string()]).output().unwrap();
            let gpu = String::from_utf8_lossy(&gpu.stdout).lines().filter(|line| line.starts_with("IOAccelerator (graphics)")).map(|line| line.split_whitespace().skip(2).take(2).collect::<Vec<_>>().join(" ")).collect::<String>();
            eprintln!("tiles passo {step} ({}): {} MB, retidos {}, texturas gpu {}, IOAccelerator(graphics) {gpu}", id.name(), process_rss_mb(), held.len(), headless.renderer_mut().texture_count());
        }
        if std::env::var_os("KILLER_MEM_VMMAP").is_some() {
            let output = std::process::Command::new("vmmap").args(["-summary", &std::process::id().to_string()]).output().unwrap();
            eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }

    #[test]
    #[ignore]
    fn walking_physics_runs_on_the_real_collision_of_a_tile() {
        use killer_world::walk::{Outcome, Walker, phys_walking};
        let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let center = Vec3::new(83400.0,148000.0,-3430.0);
        let loaded = load_tile(&root,"22_22",Some((center,0)),false,None).unwrap();
        let mut client = Client::open(&root).unwrap();
        let mut resources = killer_assets::collision::resources::Resources::default();
        let views = vec![killer_assets::collision::world::View { name:"22_22".into(),level:loaded.level.clone(),poses:Default::default() }];
        let mut demo = characters::demo_world(center,0,0);
        let mover = demo.me.as_mut().unwrap();
        mover.position = center + Vec3::Z * 25.0;
        mover.collision_height = 23.0;
        mover.collision_radius = 9.0;
        assert!(loaded.ground.place(mover));
        let mut walker = Walker::new(mover.position,mover.collision_radius,mover.collision_height);
        walker.on_world = true;
        let mut sweep = crate::walk_world::WorldSweep::new(&mut client,&mut resources,&views);
        let mut falls = 0;
        let mut travelled = 0.0;
        for step in 0..600 {
            let angle = (step / 150) as f32 * std::f32::consts::FRAC_PI_2;
            walker.velocity = Vec3::new(angle.cos(),angle.sin(),0.0) * 120.0;
            let before = walker.location;
            match phys_walking(&mut sweep,&mut walker,1.0/60.0) {
                Outcome::Walking => travelled += (walker.location - before).truncate().length(),
                Outcome::Landed { .. } | Outcome::Swimming { .. } | Outcome::Surfaced { .. } => {}
                Outcome::Falling { .. } => { falls += 1; if falls < 6 { eprintln!("queda passo {step} em {:?} piso {:?}",before,walker.floor_normal); } walker.location = before; walker.on_world = true; }
                Outcome::Blocked | Outcome::Halted => {}
            }
            let ground = loaded.ground.ground_at(walker.location.x,walker.location.y,walker.location.z,walker.location.z-200.0);
            if let Some(ground) = ground {
                let gap = walker.location.z - walker.extent.z - ground;
                assert!(gap > -3.0 && gap < 14.0,"passo {step}: vão {gap}");
            }
        }
        assert!(sweep.error.is_none(),"{:?}",sweep.error);
        eprintln!("andou {travelled:.0} unidades, {falls} quedas, pos {:?}",walker.location);
        assert!(travelled > 100.0);
        drop(sweep);
        let target = walker.location + Vec3::Z * 30.0;
        let mut hits = 0;
        for turn in 0..36 {
            let yaw = turn as f32 * 10.0_f32.to_radians();
            let end = target + Vec3::new(yaw.cos(),yaw.sin(),0.35).normalize() * 230.0;
            let hit = crate::walk_world::camera_hit(&mut client,&mut resources,&views,&[],target,end,0.1).unwrap();
            if let Some(hit) = hit { assert!(hit.distance >= 0.0 && hit.distance <= 230.5); hits += 1; }
        }
        eprintln!("câmera: {hits}/36 direções com colisão");
    }

    #[test]
    fn controller_calls_of_the_local_pawn_become_the_packets_of_can_not_move_anymore_and_the_stop() {
        use killer_world::walk::ControllerCall;
        let mut world = characters::demo_world(Vec3::ZERO, 0, 0);
        let me = world.me.as_mut().unwrap();
        me.heading = 1234;
        me.dest = Some(Vec3::new(500.0, 0.0, 0.0));
        let id = me.object_id;
        let at = Vec3::new(10.4, -20.6, 30.0);
        let calls = [
            (id, ControllerCall::CannotMoveAnymore { location: at, from_hit_wall: true }),
            (id, ControllerCall::CannotMoveAnymore { location: at, from_hit_wall: false }),
            (id, ControllerCall::Stop { location: at }),
            (id + 1, ControllerCall::Stop { location: at }),
        ];
        let (commands, stopped) = crate::walk_world::controller_commands(&mut world, &calls);
        assert!(stopped && world.me.as_ref().unwrap().dest.is_none());
        assert_eq!(commands, vec![
            NetCommand::CannotMoveAnymore { position: [10, -21, 30], heading: 1234 },
            NetCommand::Validate { position: [10, -21, 30], heading: 1234 },
        ]);
        world.vehicles.insert(50, killer_world::state::Vehicle { x: 1000, y: 0, z: 0, heading: 16384, dest: None, move_speed: 0, rotation_speed: 0, state: 0 });
        world.me.as_mut().unwrap().vehicle = Some(50);
        let ship = Vec3::new(1000.0, 100.0, 10.0);
        let calls = [
            (id, ControllerCall::CannotMoveAnymore { location: ship, from_hit_wall: true }),
            (id, ControllerCall::Stop { location: ship }),
        ];
        let (commands, _) = crate::walk_world::controller_commands(&mut world, &calls);
        assert_eq!(commands, vec![
            NetCommand::CannotMoveAnymoreInVehicle { vehicle_id: 50, position: [100, 0, 10], heading: 1234 },
            NetCommand::ValidateInVehicle { vehicle_id: 50, position: [100, 0, 10], heading: 1234 },
        ]);
        world.vehicles.clear();
        let (commands, _) = crate::walk_world::controller_commands(&mut world, &calls);
        assert!(commands.is_empty());
    }
}
