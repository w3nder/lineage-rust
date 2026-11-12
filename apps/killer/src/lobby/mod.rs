pub mod classes;
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod demo;
pub mod panels;
pub mod scene;
pub mod turn;

use crate::characters::{self, CharacterCache, DisplaySettings, Frame, demo_entity, project};
use crate::ground::GroundGrid;
use crate::pace;
use crate::ui2d::GameUi;
use config::ServerConfig;
use glam::Vec3;
#[cfg(not(target_arch = "wasm32"))]
use killer_assets::cache::SceneCache;
#[cfg(test)]
use killer_assets::cache::SourceStamp;
use killer_assets::client::{Client, DatTables};
use killer_assets::scene::actor_lighting::LightMap;
use killer_assets::scene::Scene;
#[cfg(not(target_arch = "wasm32"))]
use killer_assets::scene::{Neighbours, build_scene_with};
use killer_assets::textures::TextureBank;
use killer_assets::timeenv::TimeEnv;
use killer_render::camera::Camera;
use killer_render::skinned::CharacterInstance;
use killer_world::{Entity, EntityKind, World};
use killer_assets::pawn::ClassSource;
use killer_assets::pawn_rotation::RotationDefaults;
use l2_dat::ClassInfo;
use l2_dat::logongrp::{Logongrp, Spot};
use l2_net::game::CharSelectEntry;
use l2_net::game::packets::{NewCharacter, PAPERDOLL_VISIBLE, VISIBLE_EQUIP_SLOTS};
use l2_net::login::GameServer;
use panels::{
    CharacterSlot, CharactersAction, CreateAction, CreateForm, LoginAction, LoginForm, ServerRow, ServersAction,
};
use scene::{
    CameraMove, CreateChoice, LobbyPose, SceneBook, StageModel, chest_tag, class_collision, create_transition,
    heading_of, lobby_hour, lobby_pose, placement_camera, spot_position, stage_models, stage_visible,
};
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc;
#[cfg(not(target_arch = "wasm32"))]
use std::thread;
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
use ue2_level::BspTree;

pub const LOBBY_MAP: &str = "lobby";
pub const NEW_CHARACTER_FAIL_SYSMSG: u32 = 128;
pub const DELETE_FAIL_SYSMSGS: [u32; 3] = [306, 541, 540];
pub const VOICE_CHAR_START: &str = "SystemMsg_e.char_start";
pub const VOICE_CHAR_CREATION: &str = "SystemMsg_e.char_creation";
pub const VOICE_CHAR_LOBBY: &str = "SystemMsg_e.char_lobby";
pub const SELECT_FIRST_ID: i32 = 1;
pub const STAGE_FIRST_ID: i32 = 100;
pub const LOBBY_WALK_SPEED: i32 = 80;
pub const LOBBY_RUN_SPEED: i32 = 120;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LobbyScreen {
    Login,
    Servers,
    Characters,
    Create,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LobbyCommand {
    Connect { config: ServerConfig, account: String, password: String },
    ChooseServer(u8),
    NoticeAnswer(bool),
    CardAnswer(i32),
    Select(usize),
    NewCharacter,
    Create(Box<NewCharacter>),
    Delete(usize),
    Restore(usize),
    Back,
    Exit,
}

pub struct LoadedLobby {
    pub lighting: LightMap,
    pub scene: Arc<Scene>,
    pub bank: TextureBank,
    pub timeenv: TimeEnv,
    pub bsp_tree: BspTree,
    pub bsp_render: ue2_level::BspRender,
    pub book: SceneBook,
    pub logongrp: Logongrp,
    pub server_names: HashMap<u32, String>,
    pub idc_names: HashMap<u32, String>,
    pub class_info: Vec<ClassInfo>,
    pub rotation_defaults: HashMap<usize, RotationDefaults>,
    pub elapsed: Duration,
}

pub const DELETE_LINE_LIFT: f32 = 9.0;

pub fn select_label_height(collision_height: f32, mesh_type: i32) -> f32 {
    collision_height * 0.5 + 7.0 + if mesh_type == 0 { 4.0 } else { 0.0 }
}

pub struct LobbyExtras {
    pub tables: Option<Result<DatTables, String>>,
    pub ui: Option<Result<GameUi, String>>,
    pub elapsed: Duration,
}

pub enum LobbyPart {
    Scene(Box<LoadedLobby>),
    Ground(Box<GroundGrid>),
    Extras(Box<LobbyExtras>),
}

pub struct LobbyView {
    pub zones: usize,
    pub lighting: LightMap,
    pub bsp_tree: BspTree,
    pub bsp_render: ue2_level::BspRender,
    pub ground: GroundGrid,
    pub timeenv: TimeEnv,
    pub book: SceneBook,
    pub logongrp: Logongrp,
    pub server_names: HashMap<u32, String>,
    pub idc_names: HashMap<u32, String>,
    pub class_info: Vec<ClassInfo>,
    pub rotation_defaults: HashMap<usize, RotationDefaults>,
}

impl LobbyView {
    pub fn zone_at(&self, position: Vec3) -> u8 {
        let zone = self.bsp_tree.zone_at(position.to_array());
        if usize::from(zone) < self.zones { zone } else { 0 }
    }
}

pub struct Lobby {
    pub screen: LobbyScreen,
    pub login: LoginForm,
    pub servers: Vec<ServerRow>,
    pub characters: Vec<CharSelectEntry>,
    pub selected: Option<usize>,
    pub create: CreateForm,
    pub template: l2_net::game::CharTemplate,
    pub status: String,
    pub busy: bool,
    pub revision: u32,
    pub raw_servers: Vec<GameServer>,
    pub servers_revision: u32,
    pub pending_notice: Option<l2_net::login::LoginNotice>,
    pub pending_card: Option<(i32, i8, bool)>,
    pub forbidden: [u8; 16],
    pub create_failure: Option<i32>,
    pub system_message: Option<u32>,
    pub voices: Vec<&'static str>,
    pub stop_voices: bool,
    pub turns: HashMap<i32, turn::PawnTurn>,
    pub turn_model: Option<usize>,
    pub view: Option<LobbyView>,
    pub pending: Option<Receiver<Result<LobbyPart, String>>>,
    extras_pending: bool,
    pub load_error: Option<String>,
    pub world: World,
    camera: Option<CameraMove>,
    choice: CreateChoice,
    zoomed: bool,
    lobby_waits: HashMap<i32, &'static str>,
    socials: HashMap<i32, &'static str>,
    stage: Vec<StageModel>,
    elapsed: f32,
    social_timer: f32,
    rng: u32,
}

#[cfg(not(target_arch = "wasm32"))]
fn send_part(tx: &mpsc::Sender<Result<LobbyPart, String>>, part: Result<LobbyPart, String>) -> bool {
    tx.send(part).is_ok()
}

pub fn load_rotation_defaults(client: &mut Client) -> HashMap<usize, RotationDefaults> {
    (0..)
        .map_while(|row| ClassSource::Player { row }.name().map(|name| (row, name)))
        .filter_map(|(row, name)| match client.pawn_rotation_defaults(&name) {
            Ok(defaults) => Some((row, defaults)),
            Err(error) => {
                log::warn!("rotação de {name}: {error}");
                None
            }
        })
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_lobby(
    root: &Path,
    want_tables: bool,
    ui_size: Option<(i32, i32)>,
    tx: &mpsc::Sender<Result<LobbyPart, String>>,
) {
    let started = Instant::now();
    let mut phase = Instant::now();
    let mut lap = |name: &str| {
        let took = phase.elapsed();
        phase = Instant::now();
        log::info!("lobby: {name} em {took:.1?}");
    };
    let mut client = match Client::open(root) {
        Ok(client) => client,
        Err(e) => {
            send_part(tx, Err(e.to_string()));
            return;
        }
    };
    let logongrp = match client.logongrp() {
        Ok(table) => table,
        Err(e) => {
            send_part(tx, Err(e.to_string()));
            return;
        }
    };
    let server_names = client
        .server_names()
        .map(|rows| rows.into_iter().map(|row| (row.id, row.name)).collect())
        .unwrap_or_else(|e| {
            log::warn!("servername-e.dat: {e}");
            HashMap::new()
        });
    let idc_names = client
        .idc_names()
        .map(|rows| rows.into_iter().map(|row| (row.id, row.name)).collect())
        .unwrap_or_else(|e| {
            log::warn!("idcname-e.dat: {e}");
            HashMap::new()
        });
    let class_info = client.class_info().unwrap_or_else(|e| {
        log::warn!("classinfo-e.dat: {e}");
        Vec::new()
    });
    lap("tabelas do lobby");
    let path = match client.level_path(LOBBY_MAP) {
        Ok(path) => path,
        Err(e) => {
            send_part(tx, Err(e.to_string()));
            return;
        }
    };
    let raw = match std::fs::read(&path) {
        Ok(raw) => raw,
        Err(e) => {
            send_part(tx, Err(e.to_string()));
            return;
        }
    };
    let stamp = crate::scene_cache::stamp(&mut client, &raw);
    let cache = SceneCache::default_dir();
    let cached = cache.as_ref().and_then(|cache| cache.read(&path, &stamp));
    lap(&format!("cache da cena ({})", if cached.is_some() { "acertou" } else { "ausente" }));
    let loaded = match client.parse_level(LOBBY_MAP, &path, &raw) {
        Ok(loaded) => loaded,
        Err(e) => {
            send_part(tx, Err(e.to_string()));
            return;
        }
    };
    drop(raw);
    lap("leitura do lobby.unr");
    let timeenv = match TimeEnv::load(&mut client, 0) {
        Ok(timeenv) => timeenv,
        Err(e) => {
            send_part(tx, Err(e.to_string()));
            return;
        }
    };
    let book = SceneBook::from_level(&loaded.level);
    let (scene, bank, fresh) = match cached {
        Some(cached) => {
            let (mut scene, mut bank) = cached.into_bank();
            let options = killer_assets::options::ClientOptions::load(&mut client).unwrap_or_default();
            scene.emitters = killer_assets::scene::emitters::build(&mut client, &loaded.level, &options);
            prefetch_textures(&mut client, &scene, &mut bank);
            (scene, bank, false)
        }
        None => {
            let options = killer_assets::options::ClientOptions::load(&mut client).unwrap_or_default();
            let options = killer_assets::options::ClientOptions { shared_sky: false, ..options };
            let scene = build_scene_with(&mut client, &loaded.level, &options, Neighbours::default());
            lap(&format!("cena ({} draws)", scene.draws.len()));
            let mut bank = TextureBank::new();
            let textures = prefetch_textures(&mut client, &scene, &mut bank);
            client.catalog().release_files();
            lap(&format!("texturas ({textures} referências)"));
            (scene, bank, true)
        }
    };
    let scene = Arc::new(scene);
    let cache_write = (fresh && cache.is_some()).then(|| (scene.clone(), bank.textures().collect::<Vec<_>>(), bank.alpha_maps().collect::<Vec<_>>()));
    let part = LoadedLobby {
        lighting: LightMap::new(&loaded.level, &scene.lights),
        scene: scene.clone(),
        bank,
        timeenv,
        bsp_tree: loaded.level.bsp_tree,
        bsp_render: loaded.level.bsp_render,
        book,
        logongrp,
        server_names,
        idc_names,
        class_info,
        rotation_defaults: load_rotation_defaults(&mut client),
        elapsed: started.elapsed(),
    };
    log::info!("lobby: cena pronta com {} cenas em {:.1?}", loaded.level.shots.len(), started.elapsed());
    if !send_part(tx, Ok(LobbyPart::Scene(Box::new(part)))) {
        return;
    }
    let ground = GroundGrid::build(&scene);
    lap(&format!("chão ({} triângulos)", ground.triangle_count()));
    if !send_part(tx, Ok(LobbyPart::Ground(Box::new(ground)))) {
        return;
    }
    if let (Some(cache), Some((scene, textures, alpha_maps))) = (cache, cache_write) {
        let map = path.clone();
        thread::spawn(move || {
            let started = Instant::now();
            match cache.write_parts(&map, &stamp, &scene, &textures, &alpha_maps) {
                Ok(file) => log::info!("lobby: cache da cena gravado em {} ({:.1?})", file.display(), started.elapsed()),
                Err(e) => log::warn!("lobby: cache da cena não gravado: {e}"),
            }
        });
    }
    let tables = want_tables.then(|| client.tables().map_err(|e| e.to_string()));
    lap("tabelas .dat");
    let ui = ui_size.map(|size| GameUi::load(&mut client, size).map_err(|e| e.to_string()));
    lap("interface");
    log::info!("lobby: extras prontos em {:.1?}", started.elapsed());
    send_part(tx, Ok(LobbyPart::Extras(Box::new(LobbyExtras { tables, ui, elapsed: started.elapsed() }))));
}

pub fn prefetch_textures(client: &mut Client, scene: &Scene, bank: &mut TextureBank) -> usize {
    let mut textures = 0usize;
    let materials = scene.material_draws().map(|draw| &draw.material).chain(scene.sky_bodies.iter().map(|body| &body.material));
    for material in materials {
        for path in material.texture_paths() {
            bank.texture(client.catalog(), path);
            textures += 1;
        }
    }
    for draw in &scene.draws {
        if let Some(mask) = &draw.mask {
            bank.alpha_map(client.catalog(), mask);
            textures += 1;
        }
    }
    for flare in &scene.flares {
        bank.texture(client.catalog(), &flare.texture);
        textures += 1;
    }
    textures
}

pub fn visible_equipment(entry: &CharSelectEntry) -> [i32; VISIBLE_EQUIP_SLOTS] {
    PAPERDOLL_VISIBLE.map(|slot| entry.paperdoll_items[slot])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PawnLook {
    pub race: i32,
    pub sex: i32,
    pub class_id: i32,
    pub face: i32,
    pub hair_style: i32,
    pub hair_color: i32,
    pub equipped: [i32; VISIBLE_EQUIP_SLOTS],
    pub enchant_effect: u8,
}

pub fn lobby_entity(object_id: i32, look: PawnLook, spot: Spot) -> Entity {
    let (radius, height) = class_collision(look.race, look.sex, look.class_id);
    let kind = EntityKind::Player {
        class_id: look.class_id,
        race: look.race,
        sex: look.sex,
        equipped: look.equipped,
        hair_style: look.hair_style,
        hair_color: look.hair_color,
        face: look.face,
        enchant_effect: look.enchant_effect,
        hero: false,
    };
    let mut entity =
        demo_entity(object_id, kind, spot_position(spot) + Vec3::new(0.0, 0.0, height), heading_of(spot.yaw), height);
    entity.collision_radius = radius;
    entity.walk_speed = LOBBY_WALK_SPEED;
    entity.run_speed = LOBBY_RUN_SPEED;
    entity.running = false;
    entity
}

pub fn select_entity(slot: usize, entry: &CharSelectEntry, spot: Spot) -> Entity {
    let object_id = SELECT_FIRST_ID + slot as i32;
    let look = PawnLook {
        race: entry.race,
        sex: entry.sex,
        class_id: entry.class_id,
        face: entry.face,
        hair_style: entry.hair_style,
        hair_color: entry.hair_color,
        equipped: visible_equipment(entry),
        enchant_effect: entry.enchant_effect,
    };
    let mut entity = lobby_entity(object_id, look, spot);
    entity.name = entry.name.clone();
    entity.level = entry.level;
    match lobby_pose(entry.delete_days) {
        LobbyPose::Standing => {}
        LobbyPose::Sitting => entity.sitting = true,
        LobbyPose::Dead => entity.dead = true,
    }
    entity
}

pub fn stage_entity(model: &StageModel) -> Entity {
    let look = PawnLook {
        race: model.race,
        sex: model.sex,
        class_id: model.class_id,
        face: 0,
        hair_style: 0,
        hair_color: 0,
        equipped: [0; VISIBLE_EQUIP_SLOTS],
        enchant_effect: 0,
    };
    let mut entity = lobby_entity(STAGE_FIRST_ID + model.index as i32, look, model.spot);
    entity.position.z += scene::ENGINE_STAGE_SPAWN_LIFT;
    entity
}

fn next_random(state: &mut u32) -> u32 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    *state
}

impl Lobby {
    pub fn new(config: ServerConfig) -> Self {
        let login = LoginForm::from_env(&config);
        Self {
            screen: LobbyScreen::Login,
            login,
            servers: Vec::new(),
            characters: Vec::new(),
            selected: None,
            create: CreateForm::default(),
            status: String::new(),
            busy: false,
            revision: 0,
            raw_servers: Vec::new(),
            servers_revision: 0,
            pending_notice: None,
            pending_card: None,
            forbidden: [0; 16],
            create_failure: None,
            system_message: None,
            voices: Vec::new(),
            stop_voices: false,
            template: l2_net::game::CharTemplate { race: 0, class_id: 0, str_: 0, dex: 0, con: 0, int_: 0, wit: 0, men: 0 },
            turns: HashMap::new(),
            turn_model: None,
            view: None,
            pending: None,
            extras_pending: false,
            load_error: None,
            world: World::default(),
            camera: None,
            choice: CreateChoice::default(),
            zoomed: false,
            lobby_waits: HashMap::new(),
            socials: HashMap::new(),
            stage: Vec::new(),
            elapsed: 0.0,
            social_timer: 0.0,
            rng: 0x9e37_79b9,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn start_loading(&mut self, root: PathBuf, want_tables: bool, ui_size: Option<(i32, i32)>) {
        let (tx, rx) = mpsc::channel();
        self.load_error = None;
        self.extras_pending = true;
        thread::spawn(move || load_lobby(&root, want_tables, ui_size, &tx));
        self.pending = Some(rx);
    }

    pub fn loading(&self) -> bool {
        self.pending.is_some()
    }

    pub fn ready(&self) -> bool {
        self.view.is_some()
    }

    pub fn settled(&self) -> bool {
        self.view.is_some() && !self.extras_pending
    }

    pub fn poll_loaded(&mut self) -> Option<LobbyPart> {
        let rx = self.pending.as_ref()?;
        match rx.try_recv() {
            Ok(Ok(part)) => {
                if matches!(part, LobbyPart::Extras(_)) {
                    self.pending = None;
                    self.extras_pending = false;
                }
                Some(part)
            }
            Ok(Err(e)) => {
                self.pending = None;
                self.extras_pending = false;
                log::error!("lobby: {e}");
                self.load_error = Some(e);
                None
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
                self.extras_pending = false;
                if self.view.is_none() {
                    self.load_error = Some("carga do lobby terminou sem resposta".to_string());
                }
                None
            }
        }
    }

    pub fn install_ground(&mut self, ground: GroundGrid) {
        let Some(view) = self.view.as_mut() else { return };
        view.ground = ground;
        for entity in self.world.entities.values_mut() {
            view.ground.place(entity);
        }
    }

    pub fn install(&mut self, loaded: &mut LoadedLobby) {
        let scene = &loaded.scene;
        self.stage = stage_models(&loaded.logongrp);
        self.view = Some(LobbyView {
            zones: scene.zones.len(),
            lighting: std::mem::take(&mut loaded.lighting),
            bsp_tree: std::mem::take(&mut loaded.bsp_tree),
            bsp_render: std::mem::take(&mut loaded.bsp_render),
            ground: GroundGrid::default(),
            timeenv: loaded.timeenv.clone(),
            book: loaded.book.clone(),
            logongrp: loaded.logongrp.clone(),
            server_names: std::mem::take(&mut loaded.server_names),
            idc_names: std::mem::take(&mut loaded.idc_names),
            class_info: std::mem::take(&mut loaded.class_info),
            rotation_defaults: std::mem::take(&mut loaded.rotation_defaults),
        });
        self.enter(self.screen);
    }

    pub fn drop_view(&mut self) {
        self.view = None;
        self.pending = None;
        self.extras_pending = false;
        self.camera = None;
        self.world = World::default();
        self.turns.clear();
        self.lobby_waits.clear();
    }

    pub fn enter(&mut self, screen: LobbyScreen) {
        log::debug!("lobby: tela {screen:?}");
        self.screen = screen;
        self.busy = false;
        self.revision = self.revision.wrapping_add(1);
        let Some(view) = &self.view else { return };
        let state = match screen {
            LobbyScreen::Login | LobbyScreen::Servers => killer_world::lobby_camera::STATE_LOGON,
            LobbyScreen::Characters => killer_world::lobby_camera::STATE_CHAR_SELECT,
            LobbyScreen::Create => killer_world::lobby_camera::STATE_CHAR_CREATE,
        };
        self.camera = killer_world::lobby_camera::move_camera_by_state(state).and_then(|tag| view.book.play(tag, false));
        self.world = World::default();
        self.turns.clear();
        self.lobby_waits.clear();
        match screen {
            LobbyScreen::Characters => self.place_characters(),
            LobbyScreen::Create => {
                self.choice = CreateChoice::default();
                self.create = CreateForm::default();
                self.social_timer = 0.0;
                self.zoomed = false;
                self.place_stage();
            }
            LobbyScreen::Login | LobbyScreen::Servers => {}
        }
    }

    pub fn set_characters(&mut self, list: Vec<CharSelectEntry>) {
        self.characters = list;
        self.selected = None;
        self.status.clear();
        self.voices.push(VOICE_CHAR_START);
        self.revision = self.revision.wrapping_add(1);
        self.enter(LobbyScreen::Characters);
    }

    fn place_characters(&mut self) {
        let Some(view) = &self.view else { return };
        for (slot, entry) in self.characters.iter().enumerate().take(view.logongrp.slots.len()) {
            let mut entity = select_entity(slot, entry, view.logongrp.slots[slot]);
            view.ground.place(&mut entity);
            self.world.entities.insert(entity.object_id, entity);
        }
    }

    pub fn set_servers(&mut self, servers: &[GameServer]) {
        self.raw_servers = servers.to_vec();
        self.servers_revision = self.servers_revision.wrapping_add(1);
        let names = self.view.as_ref().map(|view| &view.server_names);
        self.servers = servers
            .iter()
            .map(|server| ServerRow {
                id: server.id,
                name: names
                    .and_then(|names| u32::from(server.id).checked_sub(1).and_then(|id| names.get(&id)).cloned())
                    .unwrap_or_else(|| format!("Server {}", server.id)),
                players: server.players,
                max_players: server.max_players,
                online: server.online,
            })
            .collect();
        self.enter(LobbyScreen::Servers);
    }

    pub fn replay_selected(&mut self, slot: usize) {
        if self.selected == Some(slot)
            && let Some(entity) = self.world.entities.get_mut(&(SELECT_FIRST_ID + slot as i32))
        {
            entity.anim.push(killer_world::anim::Trigger::Replay);
        }
    }

    pub fn select(&mut self, slot: usize) {
        let Some(view) = &self.view else { return };
        if self.selected == Some(slot) || slot >= self.characters.len() {
            return;
        }
        if self.characters.get(slot).is_some_and(|entry| lobby_pose(entry.delete_days) == LobbyPose::Dead) {
            return;
        }
        if let Some(previous) = self.selected.take()
            && let Some(entity) = self.world.entities.get_mut(&(SELECT_FIRST_ID + previous as i32))
        {
            walk_to(entity, view.logongrp.slots[previous]);
        }
        if let Some(entity) = self.world.entities.get_mut(&(SELECT_FIRST_ID + slot as i32)) {
            walk_to(entity, view.logongrp.center);
        }
        self.selected = Some(slot);
    }

    pub fn deselect(&mut self) {
        let Some(view) = &self.view else { return };
        if let Some(previous) = self.selected.take()
            && let Some(entity) = self.world.entities.get_mut(&(SELECT_FIRST_ID + previous as i32))
        {
            walk_to(entity, view.logongrp.slots[previous]);
        }
    }

    pub fn choose(&mut self) {
        self.apply_choice();
    }

    pub fn look_changed(&mut self) {
        if !self.zoomed {
            self.zoom(true);
        }
    }

    pub fn zoom(&mut self, zoom_in: bool) {
        let Some(view) = &self.view else { return };
        if self.zoomed == zoom_in || self.camera.as_ref().is_some_and(|camera| !camera.done()) {
            return;
        }
        if let Some(tag) = chest_tag(self.choice)
            && let Some(camera) = view.book.play(&tag, !zoom_in)
        {
            self.camera = Some(camera);
            self.zoomed = zoom_in;
            self.create.zoomed = zoom_in;
        }
    }

    pub fn apply_choice(&mut self) {
        let next = CreateChoice {
            race: self.create.race,
            mage: self.create.class.map(|class| class == 1),
            female: self.create.female,
        };
        if next == self.choice {
            return;
        }
        let event = create_transition(self.choice, next);
        self.apply_choice_with(event, true);
    }

    pub fn apply_choice_with(&mut self, event: Option<(String, bool)>, fallback: bool) {
        let next = CreateChoice {
            race: self.create.race,
            mage: self.create.class.map(|class| class == 1),
            female: self.create.female,
        };
        let Some(view) = &self.view else {
            self.choice = next;
            return;
        };
        if let Some((tag, reverse)) = event {
            log::debug!("lobby: cena {tag} (reverso {reverse})");
            let played = view.book.play(&tag, reverse);
            self.camera = if fallback { played.or_else(|| next.race.and_then(|race| view.book.play(scene::RACE_TAGS[race], false))) } else { played };
        }
        self.choice = next;
        self.zoomed = false;
        self.create.zoomed = false;
        self.place_stage();
    }

    fn place_stage(&mut self) {
        let Some(view) = &self.view else { return };
        let choice = self.choice;
        self.world = World::default();
        self.turns.clear();
        self.lobby_waits.clear();
        for model in self.stage.iter().filter(|model| stage_visible(model, choice)) {
            let mut entity = stage_entity(model);
            view.ground.place(&mut entity);
            self.lobby_waits.insert(entity.object_id, model.wait_anim);
            self.world.entities.insert(entity.object_id, entity);
        }
    }

    pub fn tick(&mut self, elapsed: Duration) -> f32 {
        let dt = pace::run(&mut self.world, elapsed);
        self.elapsed += dt;
        if let Some(camera) = self.camera.as_mut() {
            camera.tick(dt);
        }
        if let Some(view) = self.view.as_ref() {
            for (id, pawn) in &mut self.turns {
                let Some(entity) = self.world.entities.get_mut(id) else { continue };
                let EntityKind::Player { race, sex, class_id, .. } = entity.kind else { continue };
                let Some(defaults) = l2_dat::chargrp::index_for(race, sex, class_id).and_then(|row| view.rotation_defaults.get(&row)) else { continue };
                turn::tick_pawn(pawn, &mut entity.heading, entity.dest.is_some(), defaults, dt);
            }
        }
        if self.screen == LobbyScreen::Create && !self.world.entities.is_empty() {
            self.social_timer += dt;
            if self.social_timer >= scene::ENGINE_STAGE_SOCIAL_PERIOD {
                self.social_timer = 0.0;
                self.stage_socials();
            }
        }
        dt
    }

    fn stage_socials(&mut self) {
        for model in &self.stage {
            let object_id = STAGE_FIRST_ID + model.index as i32;
            let roll = next_random(&mut self.rng) % 100;
            if roll < 100 - scene::ENGINE_STAGE_SOCIAL_CHANCE {
                continue;
            }
            if self.world.entities.contains_key(&object_id) {
                self.socials.insert(object_id, model.social_anim);
            }
        }
    }

    pub fn camera(&self) -> Camera {
        match (&self.camera, &self.view) {
            (Some(camera), _) => camera.camera(),
            (None, Some(view)) => {
                view.book.warp_placement(scene::TAG_LOGON).map(placement_camera).unwrap_or_else(default_camera)
            }
            (None, None) => default_camera(),
        }
    }

    pub fn pick_slot(&self, size: (u32, u32), pixel: (f32, f32)) -> Option<usize> {
        let (origin, dir) = crate::input::pixel_ray(&self.camera(), size, pixel)?;
        let query = crate::input::pick_query(origin, dir, false);
        let pawn = (0..self.characters.len())
            .filter_map(|slot| {
                let entity = self.world.entities.get(&(SELECT_FIRST_ID + slot as i32))?;
                if entity.collision_radius <= 0.0 || entity.collision_height <= 0.0 {
                    return None;
                }
                let hit = killer_assets::collision::cylinder::trace(
                    entity.position.to_array(),
                    entity.collision_radius,
                    entity.collision_height,
                    query.start.to_array(),
                    query.end.to_array(),
                    query.extent.to_array(),
                )?;
                Some((slot, hit.time))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))?;
        if let Some(view) = &self.view {
            let model = killer_assets::collision::model::Model { tree: &view.bsp_tree, data: &view.bsp_render, transform: None, unscaled: true };
            let level = model.trace_line(query.start.to_array(), query.end.to_array(), query.flags | 0x400);
            if level.is_some_and(|hit| hit.time < pawn.1) {
                return None;
            }
        }
        Some(pawn.0)
    }

    pub fn hour(&self) -> f32 {
        lobby_hour(self.elapsed)
    }

    pub fn instances(
        &mut self,
        cache: &mut CharacterCache,
        tables: &DatTables,
        dt: f32,
        size: (u32, u32),
    ) -> Vec<CharacterInstance> {
        let Some(view) = &self.view else { return Vec::new() };
        let camera = self.camera();
        let frustum = camera.frustum(size.0 as f32 / size.1.max(1) as f32);
        let frame = Frame { display: DisplaySettings::default(), viewer: camera.eye, dt };
        let waits = &self.lobby_waits;
        let socials = std::mem::take(&mut self.socials);
        characters::instances_with(
            &mut self.world,
            cache,
            tables,
            &frame,
            |p| view.zone_at(p),
            |query| view.lighting.sample(query, 0, self.elapsed),
            |center, radius| frustum.contains_sphere(center, radius),
            |object_id| waits.get(&object_id).map(|wait| (*wait, socials.get(&object_id).copied())),
        )
    }

    pub fn slots(&self, size: (f32, f32), points_per_pixel: f32) -> Vec<CharacterSlot> {
        let camera = self.camera();
        self.characters
            .iter()
            .enumerate()
            .map(|(slot, entry)| {
                let entity = self.world.entities.get(&(SELECT_FIRST_ID + slot as i32));
                let project_at = |lift: f32| {
                    entity.and_then(|entity| {
                        let EntityKind::Player { race, sex, class_id, .. } = entity.kind else { return None };
                        let mesh_type = l2_dat::chargrp::index_for(race, sex, class_id).map_or(-1, |row| row as i32);
                        let top = entity.position + Vec3::new(0.0, 0.0, select_label_height(entity.collision_height, mesh_type) + lift);
                        project(&camera, size, top).map(|[x, y]| [x * points_per_pixel, y * points_per_pixel])
                    })
                };
                let label = project_at(0.0);
                let delete_label = project_at(DELETE_LINE_LIFT);
                CharacterSlot {
                    name: entry.name.clone(),
                    class_name: classes::class_name(entry.class_id),
                    level: entry.level,
                    deleting: entry.delete_days != 0,
                    label,
                    delete_label,
                }
            })
            .collect()
    }

    pub fn description(&self) -> String {
        let id = self.create.class_info_id();
        self.view
            .as_ref()
            .and_then(|view| view.class_info.iter().find(|row| row.id == id))
            .map(|row| row.text.clone())
            .unwrap_or_else(|| if id == 0 { "Select a race and occupation.".to_string() } else { String::new() })
    }

    pub fn panels(
        &mut self,
        ctx: &egui::Context,
        version: &str,
        size: (f32, f32),
        points_per_pixel: f32,
    ) -> Vec<LobbyCommand> {
        let mut commands = Vec::new();
        panels::version_tag(ctx, version);
        if self.view.is_none() && self.pending.is_some() && self.load_error.is_none() {
            panels::loading(ctx, "Loading");
        }
        match self.screen {
            LobbyScreen::Login => match panels::login(ctx, &mut self.login) {
                Some(LoginAction::Connect) => {
                    if let Some(config) = self.login.config() {
                        let password = std::mem::take(&mut self.login.password);
                        self.login.connecting = true;
                        self.login.status = "conectando…".to_string();
                        commands.push(LobbyCommand::Connect { config, account: self.login.account.clone(), password });
                    }
                }
                Some(LoginAction::Exit) => commands.push(LobbyCommand::Exit),
                None => {}
            },
            LobbyScreen::Servers => match panels::servers(ctx, &self.servers, &self.status) {
                Some(ServersAction::Choose(id)) => {
                    self.status = "entrando…".to_string();
                    commands.push(LobbyCommand::ChooseServer(id));
                }
                Some(ServersAction::Back) => commands.push(LobbyCommand::Back),
                None => {}
            },
            LobbyScreen::Characters => {
                let slots = self.slots(size, points_per_pixel);
                match panels::characters(ctx, &slots, self.selected, &self.status, self.busy) {
                    Some(CharactersAction::Select(slot)) => self.select(slot),
                    Some(CharactersAction::Enter) => {
                        if let Some(slot) = self.selected {
                            self.busy = true;
                            self.status = "entrando…".to_string();
                            commands.push(LobbyCommand::Select(slot));
                        }
                    }
                    Some(CharactersAction::Back) => commands.push(LobbyCommand::Back),
                    Some(CharactersAction::Delete) => {
                        if let Some(slot) = self.selected {
                            commands.push(LobbyCommand::Delete(slot));
                        }
                    }
                    Some(CharactersAction::Create) => {
                        self.deselect();
                        self.enter(LobbyScreen::Create);
                    }
                    None => {}
                }
            }
            LobbyScreen::Create => {
                let description = self.description();
                match panels::create(ctx, &mut self.create, &description) {
                    Some(CreateAction::Changed) => self.apply_choice(),
                    Some(CreateAction::Look) => self.look_changed(),
                    Some(CreateAction::Zoom) => self.zoom(!self.zoomed),
                    Some(CreateAction::Create) => {
                        if let Some(class_id) = self.create.class_id() {
                            self.create.sending = true;
                            self.create.status = "criando…".to_string();
                            commands.push(LobbyCommand::Create(Box::new(NewCharacter {
                                name: self.create.name.trim().to_string(),
                                race: self.create.race.unwrap_or_default() as i32,
                                sex: i32::from(self.create.female.unwrap_or_default()),
                                class_id,
                                int_: self.template.int_,
                                str_: self.template.str_,
                                con: self.template.con,
                                men: self.template.men,
                                dex: self.template.dex,
                                wit: self.template.wit,
                                hair_style: self.create.hair_style,
                                hair_color: self.create.hair_color,
                                face: self.create.face,
                            })));
                        }
                    }
                    Some(CreateAction::Cancel) => self.enter(LobbyScreen::Characters),
                    None => {}
                }
            }
        }
        commands
    }

    pub fn model_entity(&self, model: Option<usize>) -> Option<i32> {
        let index = model?;
        self.stage.iter().find(|stage| Logongrp::stage_index(stage.race, stage.sex, stage.class_id) == Some(index)).map(|stage| STAGE_FIRST_ID + stage.index as i32)
    }

    fn pawn_turn(&mut self, id: i32) -> &mut turn::PawnTurn {
        let heading = self.world.entities.get(&id).map_or(0, |entity| entity.heading);
        self.turns.entry(id).or_insert_with(|| turn::PawnTurn { desired: [0, heading, 0], ..turn::PawnTurn::default() })
    }

    pub fn turn(&mut self, model: Option<usize>, rate: f32) {
        if rate != 0.0 {
            self.turn_model = model;
            let pawn = self.model_entity(model).filter(|id| self.world.entities.contains_key(id));
            turn::default_character_turn(pawn.map(|id| self.pawn_turn(id)), rate);
        } else {
            let pawn = self.model_entity(self.turn_model).filter(|id| self.world.entities.contains_key(id));
            turn::default_character_stop(pawn.and_then(|id| self.turns.get_mut(&id)));
        }
    }

    pub fn apply_look(&mut self, model: Option<usize>, style: i32, color: i32, face: i32) {
        let chosen = self.model_entity(model);
        for (id, entity) in self.world.entities.iter_mut().filter(|(id, _)| **id >= STAGE_FIRST_ID) {
            if let EntityKind::Player { hair_style, hair_color, face: look_face, .. } = &mut entity.kind {
                let on = chosen == Some(*id);
                *hair_style = if on { style } else { 0 };
                *hair_color = if on { color } else { 0 };
                *look_face = if on { face } else { 0 };
            }
        }
        for model in &mut self.stage {
            let id = STAGE_FIRST_ID + model.index as i32;
            let Some(heading) = self.world.entities.get(&id).map(|entity| entity.heading) else { continue };
            let pawn = self.turns.entry(id).or_insert_with(|| turn::PawnTurn { desired: [0, heading, 0], ..turn::PawnTurn::default() });
            turn::reset_default_character_yaw(Some(pawn), &mut model.spot.yaw, heading);
        }
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn set_templates(&mut self, templates: &l2_net::game::CharTemplates) {
        if let Some(last) = templates.templates.last() {
            self.template = *last;
        }
    }

    pub fn new_character_result(&mut self, success: bool) {
        if success {
            self.deselect();
            self.voices.push(VOICE_CHAR_CREATION);
            self.enter(LobbyScreen::Create);
            return;
        }
        if self.screen != LobbyScreen::Characters {
            self.enter(LobbyScreen::Characters);
        }
        self.system_message = Some(NEW_CHARACTER_FAIL_SYSMSG);
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn character_created(&mut self) {
        self.voices.push(VOICE_CHAR_LOBBY);
        self.status = "personagem criado".to_string();
    }

    pub fn create_failed(&mut self, reason: i32) {
        self.create_failure = Some(reason);
        self.revision = self.revision.wrapping_add(1);
        self.create.sending = false;
        self.create.status = format!("criação recusada ({})", create_failure(reason));
    }

    pub fn delete_failed(&mut self, reason: i32) {
        self.system_message = usize::try_from(reason - 1).ok().and_then(|index| DELETE_FAIL_SYSMSGS.get(index)).copied();
        self.status = format!("exclusão recusada ({reason})");
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn show_notice(&mut self, notice: l2_net::login::LoginNotice) {
        self.forbidden = notice.forbidden;
        self.pending_notice = Some(notice);
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn back_to_login(&mut self, status: String) {
        self.pending_notice = None;
        self.pending_card = None;
        self.characters.clear();
        self.servers.clear();
        self.selected = None;
        self.login.connecting = false;
        self.login.status = status;
        self.status.clear();
        self.enter(LobbyScreen::Login);
    }
}

fn walk_to(entity: &mut Entity, spot: Spot) {
    let target = spot_position(spot).with_z(entity.position.z);
    entity.dest = Some(target);
    entity.stop_heading = Some(heading_of(spot.yaw));
    entity.running = false;
}

fn default_camera() -> Camera {
    Camera {
        eye: Vec3::ZERO,
        target: Vec3::X,
        fov_x_deg: scene::ENGINE_LOBBY_FOV_X_DEG,
        near: scene::NEAR,
        far: scene::FAR,
    }
}

pub fn create_failure(reason: i32) -> &'static str {
    match reason {
        0 => "criação falhou",
        1 => "muitos personagens",
        2 => "nome já existe",
        3 => "nome com 16 caracteres ou menos",
        4 => "nome incorreto",
        5 => "não pode criar personagem",
        _ => "recusado",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lobby::panels::tests::{click_on, rect_of, settle};
    use egui::pos2;
    use l2_dat::logongrp::{STAGE_SLOTS, STAGE_TEMPLATES};

    fn entry(name: &str, race: i32, sex: i32, class_id: i32, delete_days: i32) -> CharSelectEntry {
        CharSelectEntry {
            name: name.into(),
            char_id: 1,
            login: "conta".into(),
            clan_id: 0,
            sex,
            race,
            base_class_id: class_id,
            active: 0,
            x: 0,
            y: 0,
            z: 0,
            hp: 100.0,
            mp: 50.0,
            sp: 0,
            exp: 0,
            level: 5,
            karma: 0,
            paperdoll_objects: [0; 16],
            paperdoll_items: [0; 16],
            hair_style: 1,
            hair_color: 2,
            face: 1,
            max_hp: 100.0,
            max_mp: 50.0,
            delete_days,
            class_id,
            last_used: 0,
            enchant_effect: 0,
        }
    }

    fn table() -> Logongrp {
        Logongrp {
            slots: std::array::from_fn(|i| Spot {
                x: -16622.0 - 30.0 * i as f32,
                y: 238671.0,
                z: -6256.0,
                yaw: -26624.0 + 4096.0 * i as f32,
            }),
            center: Spot { x: -16709.0, y: 238583.0, z: -6256.0, yaw: -15360.0 },
            stage: std::array::from_fn(|i| Spot { x: 1000.0 * i as f32, y: 0.0, z: -6400.0, yaw: 0.0 }),
        }
    }

    fn view() -> LobbyView {
        let mut level = ue2_level::Level::default();
        let mut shot = |tag: &str, location: [f32; 3]| {
            level.shots.insert(
                tag.into(),
                vec![ue2_level::Shot {
                    placement: ue2_level::Placement { location, rotation: [0, 16384, 0] },
                    seconds: 0.0,
                    curved: false,
                    handles: [[0.0; 3]; 2],
                }],
            );
        };
        shot(scene::TAG_LOGON, [1.0, 0.0, 0.0]);
        shot(killer_world::lobby_camera::move_camera_by_state(killer_world::lobby_camera::STATE_CHAR_SELECT).unwrap(), [2.0, 0.0, 0.0]);
        shot(killer_world::lobby_camera::move_camera_by_state(killer_world::lobby_camera::STATE_CHAR_CREATE).unwrap(), [3.0, 0.0, 0.0]);
        shot("Human", [4.0, 0.0, 0.0]);
        shot("Elf", [5.0, 0.0, 0.0]);
        let book = SceneBook::from_level(&level);
        LobbyView {
            zones: 0,
            lighting: LightMap::default(),
            bsp_tree: BspTree::default(),
            bsp_render: ue2_level::BspRender::default(),
            ground: GroundGrid::default(),
            timeenv: TimeEnv::parse(""),
            book,
            logongrp: table(),
            server_names: HashMap::from([(0, "Killer 70x".to_string()), (1, "Honor 25x".to_string())]),
            idc_names: HashMap::new(),
            class_info: vec![
                ClassInfo { id: 0, text: "Select a race and occupation.".into() },
                ClassInfo { id: 3, text: "Elven Fighter text".into() },
            ],
            rotation_defaults: HashMap::new(),
        }
    }

    fn lobby() -> Lobby {
        let mut lobby = Lobby::new(ServerConfig::default());
        lobby.view = Some(view());
        lobby.stage = stage_models(&table());
        lobby
    }

    #[test]
    fn a_selected_character_carries_the_weapon_enchant_of_the_character_list() {
        let mut character = entry("A", 0, 0, 0, 0);
        character.enchant_effect = 16;
        let spot = Spot { x: 0.0, y: 0.0, z: 0.0, yaw: 0.0 };
        let entity = select_entity(0, &character, spot);
        assert!(matches!(entity.kind, EntityKind::Player { enchant_effect: 16, .. }));
    }

    #[test]
    fn three_characters_stand_on_their_slots_and_the_selected_one_walks_to_the_center() {
        let mut lobby = lobby();
        lobby.set_characters(vec![
            entry("Canastra", 0, 0, 0, 0),
            entry("Sombra", 2, 1, 31, 3),
            entry("Noite", 1, 1, 25, -1),
        ]);
        assert_eq!(lobby.screen, LobbyScreen::Characters);
        assert_eq!(lobby.world.entities.len(), 3);
        let canastra = &lobby.world.entities[&SELECT_FIRST_ID];
        assert_eq!((canastra.position.x, canastra.position.y), (-16622.0, 238671.0));
        assert_eq!(canastra.position.z, -6256.0 + 23.0);
        assert_eq!(canastra.heading, heading_of(-26624.0));
        assert_eq!((canastra.collision_radius, canastra.collision_height), (9.0, 23.0));
        assert!(!canastra.sitting && !canastra.dead && !canastra.running);
        let sombra = &lobby.world.entities[&(SELECT_FIRST_ID + 1)];
        assert!(sombra.sitting && !sombra.dead);
        assert_eq!(sombra.position.x, -16652.0);
        assert!(matches!(
            sombra.kind,
            EntityKind::Player { race: 2, sex: 1, class_id: 31, face: 1, hair_style: 1, hair_color: 2, .. }
        ));
        let noite = &lobby.world.entities[&(SELECT_FIRST_ID + 2)];
        assert!(noite.dead);
        assert_eq!(lobby.camera().eye, Vec3::new(2.0, 0.0, 0.0));
        lobby.select(0);
        assert_eq!(lobby.selected, Some(0));
        let canastra = &lobby.world.entities[&SELECT_FIRST_ID];
        assert_eq!(canastra.dest, Some(Vec3::new(-16709.0, 238583.0, -6256.0 + 23.0)));
        assert_eq!(canastra.stop_heading, Some(heading_of(-15360.0)));
        lobby.select(2);
        assert_eq!(lobby.selected, Some(0));
        lobby.select(1);
        assert_eq!(lobby.selected, Some(1));
        let canastra = &lobby.world.entities[&SELECT_FIRST_ID];
        assert_eq!(canastra.dest.map(|d| (d.x, d.y)), Some((-16622.0, 238671.0)));
        assert_eq!(lobby.world.entities[&(SELECT_FIRST_ID + 1)].dest.map(|d| d.x), Some(-16709.0));
        for _ in 0..600 {
            lobby.tick(Duration::from_millis(16));
        }
        let sombra = &lobby.world.entities[&(SELECT_FIRST_ID + 1)];
        assert_eq!(sombra.dest, None);
        let slot = glam::Vec2::new(-16709.0, 238583.0);
        assert!(sombra.position.truncate().distance(slot) <= sombra.collision_radius);
        assert_eq!(sombra.heading, heading_of(-15360.0));
        assert!((lobby.hour() - (12.0 + 9.6 * 6.0 / 3600.0)).abs() < 1e-3);
    }

    #[test]
    fn a_click_picks_the_nearest_pawn_cylinder_along_the_camera_ray() {
        let mut lobby = lobby();
        lobby.set_characters(vec![entry("A", 0, 0, 0, 0), entry("B", 0, 0, 0, 0)]);
        let camera = lobby.camera();
        let forward = (camera.target - camera.eye).normalize();
        let size = (1280, 720);
        let center = (640.0, 360.0);
        let near = camera.eye + forward * 400.0;
        let far = camera.eye + forward * 900.0;
        lobby.world.entities.get_mut(&SELECT_FIRST_ID).unwrap().position = far;
        lobby.world.entities.get_mut(&(SELECT_FIRST_ID + 1)).unwrap().position = near;
        assert_eq!(lobby.pick_slot(size, center), Some(1));
        lobby.world.entities.get_mut(&(SELECT_FIRST_ID + 1)).unwrap().position = near + Vec3::new(0.0, 0.0, 5000.0);
        assert_eq!(lobby.pick_slot(size, center), Some(0));
        assert_eq!(lobby.pick_slot(size, (5.0, 5.0)), None);
    }

    #[test]
    fn the_create_screen_shows_the_race_models_and_moves_the_camera_by_the_scene_tags() {
        let mut lobby = lobby();
        lobby.enter(LobbyScreen::Create);
        assert_eq!(lobby.camera().eye, Vec3::new(3.0, 0.0, 0.0));
        assert_eq!(lobby.world.entities.len(), 18);
        lobby.create.race = Some(1);
        lobby.apply_choice();
        assert_eq!(lobby.camera().eye, Vec3::new(5.0, 0.0, 0.0));
        let mut ids: Vec<i32> = lobby.world.entities.keys().copied().collect();
        ids.sort_unstable();
        assert_eq!(ids, (100..118).collect::<Vec<i32>>());
        assert_eq!(lobby.lobby_waits[&104], "Social_SpWait01_MElf");
        assert_eq!(lobby.world.entities[&104].position, Vec3::new(4000.0, 0.0, -6400.0 + 24.0 + 60.0));
        lobby.create.class = Some(1);
        lobby.create.female = Some(true);
        lobby.apply_choice();
        let ids: Vec<i32> = lobby.world.entities.keys().copied().collect();
        assert_eq!(ids, [107]);
        assert_eq!(lobby.camera().eye, Vec3::new(5.0, 0.0, 0.0));
        lobby.create.race = Some(0);
        lobby.apply_choice();
        assert_eq!(lobby.camera().eye, Vec3::new(4.0, 0.0, 0.0));
        assert_eq!(lobby.world.entities.keys().copied().collect::<Vec<_>>(), [103]);
        assert_eq!(lobby.description(), "");
        lobby.create.race = Some(1);
        lobby.create.class = Some(0);
        lobby.apply_choice();
        assert_eq!(lobby.create.class_info_id(), 3);
        assert_eq!(lobby.description(), "Elven Fighter text");
        lobby.create.female = Some(false);
        lobby.apply_choice();
        lobby.look_changed();
        assert!(!lobby.zoomed);
        let mut level = ue2_level::Level::default();
        level.shots.insert(
            "Elf_Kman_Chest".into(),
            vec![
                ue2_level::Shot {
                    placement: ue2_level::Placement { location: [5.0, 0.0, 0.0], rotation: [0, 0, 0] },
                    seconds: 0.0,
                    curved: false,
                    handles: [[0.0; 3]; 2],
                },
                ue2_level::Shot {
                    placement: ue2_level::Placement { location: [6.0, 0.0, 0.0], rotation: [0, 0, 0] },
                    seconds: 1.0,
                    curved: false,
                    handles: [[0.0; 3]; 2],
                },
            ],
        );
        lobby.view.as_mut().unwrap().book = SceneBook::from_level(&level);
        lobby.look_changed();
        assert!(lobby.zoomed);
        for _ in 0..10 {
            lobby.tick(Duration::from_millis(200));
        }
        assert_eq!(lobby.camera().eye, Vec3::new(6.0, 0.0, 0.0));
        lobby.zoom(false);
        assert!(!lobby.zoomed && !lobby.create.zoomed);
        assert_eq!(lobby.camera().eye, Vec3::new(6.0, 0.0, 0.0));
        for _ in 0..10 {
            lobby.tick(Duration::from_millis(200));
        }
        assert_eq!(lobby.camera().eye, Vec3::new(5.0, 0.0, 0.0));
        lobby.rng = 1;
        lobby.social_timer = 0.0;
        let mut fired = 0;
        for _ in 0..(5.5 * 60.0) as usize {
            lobby.tick(Duration::from_millis(16));
            fired += lobby.socials.len();
            lobby.socials.clear();
        }
        assert!(lobby.social_timer < 1.0 && fired <= 4, "{fired}");
        assert_eq!(STAGE_TEMPLATES.len(), STAGE_SLOTS);
    }

    #[test]
    fn panels_turn_into_commands_and_the_server_list_takes_the_names_of_the_dat() {
        let mut lobby = lobby();
        let ctx = egui::Context::default();
        lobby.login.account = "conta".into();
        lobby.login.password = "senha".into();
        settle(&ctx, |ctx| lobby.panels(ctx, "x", (1280.0, 720.0), 1.0));
        let rect = rect_of(&ctx, "login");
        let log_in = pos2(
            rect.left() + 18.0 + 96.0 + panels::BUTTON_SIZE.x / 2.0,
            rect.bottom() - 18.0 - panels::BUTTON_SIZE.y / 2.0,
        );
        let commands =
            click_on(&ctx, log_in, |ctx| lobby.panels(ctx, "x", (1280.0, 720.0), 1.0));
        assert_eq!(
            commands,
            [LobbyCommand::Connect {
                config: ServerConfig::default(),
                account: "conta".into(),
                password: "senha".into()
            }]
        );
        assert!(lobby.login.password.is_empty() && lobby.login.connecting);
        lobby.set_servers(&[
            GameServer { id: 1, ip: [127, 0, 0, 1], port: 7777, players: 3, max_players: 1000, online: true, extra: Default::default() },
            GameServer { id: 9, ip: [127, 0, 0, 1], port: 7778, players: 0, max_players: 10, online: false, extra: Default::default() },
        ]);
        assert_eq!(lobby.screen, LobbyScreen::Servers);
        assert_eq!(lobby.servers[0].name, "Killer 70x");
        assert_eq!(lobby.servers[1].name, "Server 9");
        assert_eq!((lobby.servers[0].players, lobby.servers[0].max_players, lobby.servers[0].online), (3, 1000, true));
        assert_eq!((lobby.servers[1].players, lobby.servers[1].max_players, lobby.servers[1].online), (0, 10, false));
        assert_eq!(lobby.camera().eye, Vec3::new(1.0, 0.0, 0.0));
        lobby.back_to_login("recusado".into());
        assert_eq!(lobby.screen, LobbyScreen::Login);
        assert!(!lobby.login.connecting && lobby.servers.is_empty());
        assert_eq!(lobby.login.status, "recusado");
    }

    #[test]
    fn server_packet_population_and_status_stay_with_the_correct_world_name() {
        let bytes = [
            4, 2, 2,
            2, 127, 0, 0, 1, 0x61, 0x1e, 0, 0, 0, 1, 0, 0, 0xf4, 1, 0, 0, 0, 0, 0, 0,
            1, 127, 0, 0, 1, 0x62, 0x1e, 0, 0, 0, 1, 0x39, 0x05, 0x88, 0x13, 1, 0, 0, 0, 0, 0,
        ];
        let servers = l2_net::login::packets::read_server_list(&bytes).unwrap();
        let mut lobby = Lobby::new(ServerConfig::default());
        lobby.view = Some(view());
        lobby.set_servers(&servers);
        assert_eq!(lobby.servers, vec![
            ServerRow { id: 2, name: "Honor 25x".into(), players: 0, max_players: 500, online: false },
            ServerRow { id: 1, name: "Killer 70x".into(), players: 1337, max_players: 5000, online: true },
        ]);
        lobby.set_servers(&[GameServer { id: 0, ip: [0; 4], port: 7777, players: 0, max_players: 0, online: false, extra: Default::default() }]);
        assert_eq!(lobby.servers[0].name, "Server 0");
    }

    #[test]
    #[ignore]
    fn every_original_server_name_matches_its_wire_id_without_shifting_population() {
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let rows = client.server_names().unwrap();
        assert!(!rows.is_empty());
        let mut lobby = Lobby::new(ServerConfig::default());
        let mut lobby_view = view();
        lobby_view.server_names = rows.iter().map(|row| (row.id, row.name.clone())).collect();
        lobby.view = Some(lobby_view);
        for row in &rows {
            let id = u8::try_from(row.id + 1).expect("ID do servidor representável no pacote C4");
            lobby.set_servers(&[GameServer { id, ip: [127, 0, 0, 1], port: 7777, players: 321, max_players: 5000, online: id % 2 == 1, extra: Default::default() }]);
            assert_eq!(lobby.servers[0], ServerRow { id, name: row.name.clone(), players: 321, max_players: 5000, online: id % 2 == 1 });
        }
        println!("{} nomes originais conferidos", rows.len());
    }

    #[test]
    fn the_entry_equipment_follows_the_visible_paperdoll_slots() {
        let mut entry = entry("x", 0, 0, 0, 0);
        entry.paperdoll_items = std::array::from_fn(|i| i as i32 * 10);
        assert_eq!(visible_equipment(&entry), [0, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150]);
        assert_eq!(create_failure(2), "nome já existe");
    }
}

#[cfg(test)]
mod client_tests {
    use super::*;
    use crate::characters::{AppearanceKey, look};
    use crate::loader::{LoadResult, Shared, load_character};
    use killer_assets::animation::sequence_index;
    use killer_assets::cache::CachedScene;
    use killer_assets::scene::Vertex;
    use l2_dat::chargrp::index_for;
    use std::path::PathBuf;

    #[test]
    #[ignore]
    fn the_lobby_scene_survives_the_disk_cache_and_loads_faster_from_it() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let mut client = Client::open(&root).unwrap();
        let path = client.level_path(LOBBY_MAP).unwrap();
        let raw = std::fs::read(&path).unwrap();
        let stamp = crate::scene_cache::stamp(&mut client, &raw);
        let started = Instant::now();
        let loaded = client.parse_level(LOBBY_MAP, &path, &raw).unwrap();
        let scene = killer_assets::scene::build_scene(&mut client, &loaded.level);
        let mut bank = TextureBank::new();
        let referenced = prefetch_textures(&mut client, &scene, &mut bank);
        let built = started.elapsed();
        let dir = std::env::temp_dir().join(format!("killer-lobby-cache-{}", std::process::id()));
        let cache = SceneCache::at(dir.clone());
        let snapshot = CachedScene::from_bank(scene.clone(), &bank);
        let started = Instant::now();
        let file = cache.write(&path, &stamp, &snapshot).unwrap();
        let written = started.elapsed();
        let size = std::fs::metadata(&file).unwrap().len();
        let started = Instant::now();
        let back = cache.read(&path, &stamp).expect("cache legível");
        let read = started.elapsed();
        assert_eq!(back.scene.draws.len(), scene.draws.len());
        for (index, (a, b)) in back.scene.draws.iter().zip(&snapshot.scene.draws).enumerate() {
            let same_vertices = a.vertices.len() == b.vertices.len() && a.vertices.iter().zip(&b.vertices).all(|(x, y)| x.position.map(f32::to_bits) == y.position.map(f32::to_bits) && x.normal.map(f32::to_bits) == y.normal.map(f32::to_bits) && x.uv.map(f32::to_bits) == y.uv.map(f32::to_bits) && x.mask_uv.map(f32::to_bits) == y.mask_uv.map(f32::to_bits) && x.color.map(f32::to_bits) == y.color.map(f32::to_bits));
            assert!(same_vertices, "draw {index}: vértices diferem");
            assert_eq!(a.indices, b.indices, "draw {index}: índices");
            assert_eq!(a.material, b.material, "draw {index}: material");
            assert!(
                (a.mask == b.mask) && a.kind == b.kind && a.sun_affect == b.sun_affect && a.pass == b.pass && a.zone == b.zone && a.lightmap == b.lightmap && a.modulate_2x == b.modulate_2x && a.sector == b.sector && a.fade == b.fade && a.tint == b.tint && a.actor == b.actor,
                "draw {index}: {:?} {:?} {:?} {:?} {:?} {:?}",
                a.mask,
                a.kind,
                a.pass,
                a.sector,
                a.fade,
                a.actor
            );
        }
        assert_eq!(back.scene.lightmaps, snapshot.scene.lightmaps);
        assert_eq!(back.scene.zones, snapshot.scene.zones);
        assert_eq!(back.scene.lights, snapshot.scene.lights);
        assert_eq!((back.scene.bounds_min, back.scene.bounds_max, back.scene.terrain_range), (snapshot.scene.bounds_min, snapshot.scene.bounds_max, snapshot.scene.terrain_range));
        assert_eq!(back.scene.sky_bodies, snapshot.scene.sky_bodies);
        assert_eq!(back.scene.flares, snapshot.scene.flares);
        assert_eq!(back.textures.len(), snapshot.textures.len());
        assert_eq!(back.alpha_maps.len(), snapshot.alpha_maps.len());
        for ((name, a), (other, b)) in back.textures.iter().chain(&back.alpha_maps).zip(snapshot.textures.iter().chain(&snapshot.alpha_maps)) {
            assert_eq!(name, other);
            assert!((a.width, a.height, a.format) == (b.width, b.height, b.format) && a.mips == b.mips, "textura {name}");
        }
        assert!(back.textures.len() + back.alpha_maps.len() <= referenced);
        assert!(back.textures.iter().all(|(name, _)| bank.has_texture(name)));
        let stale = SourceStamp { size: stamp.size, hash: stamp.hash ^ 1 };
        assert_eq!(cache.read(&path, &stale), None);
        let _ = std::fs::remove_dir_all(dir);
        let vertex_bytes: usize = scene.draws.iter().map(|d| d.vertices.len() * std::mem::size_of::<Vertex>() + d.indices.len() * 4).sum();
        let mut by_format = std::collections::BTreeMap::new();
        for (_, data) in bank.textures().chain(bank.alpha_maps()) {
            let entry = by_format.entry(format!("{:?}", data.format)).or_insert((0usize, 0usize));
            entry.0 += 1;
            entry.1 += data.mips.iter().map(Vec::len).sum::<usize>();
        }
        let lightmap_bytes: usize = scene.lightmaps.iter().flat_map(|g| g.slices.iter()).flat_map(|s| s.iter()).map(Vec::len).sum();
        eprintln!(
            "lobby: construir {built:.1?} ({} draws, {referenced} referências de textura), gravar {written:.1?} ({} MB: geometria {} MB, lightmaps {} MB, texturas {:?}), ler {read:.1?}",
            scene.draws.len(),
            size / (1024 * 1024),
            vertex_bytes / (1024 * 1024),
            lightmap_bytes / (1024 * 1024),
            by_format.iter().map(|(f, (n, b))| format!("{f} {n}× {} MB", b / (1024 * 1024))).collect::<Vec<_>>()
        );
    }

    #[test]
    #[ignore]
    fn every_stage_model_has_its_lobby_wait_and_social_sequences() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let mut client = Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        let table = client.logongrp().unwrap();
        let mut shared = Shared::default();
        for model in stage_models(&table) {
            let row = index_for(model.race, model.sex, model.class_id).unwrap();
            let key = AppearanceKey::Player {
                row,
                face: 0,
                hair_style: 0,
                hair_color: 0,
                equipped: l2_dat::Equipped::default(),
                enchant_effect: 0,
                hero: false,
                mounted: 0,
            };
            let look = look(key, &tables).unwrap();
            let LoadResult::Character { animation, .. } =
                load_character(&mut client, &mut shared, key, &look.appearance, look.draw_scale)
            else {
                panic!("modelo {} não carregou", model.index);
            };
            let animation = animation.expect("sem animação");
            assert!(sequence_index(&animation, model.wait_anim).is_some(), "{} sem {}", model.index, model.wait_anim);
            assert!(
                sequence_index(&animation, model.social_anim).is_some(),
                "{} sem {}",
                model.index,
                model.social_anim
            );
        }
    }
}
