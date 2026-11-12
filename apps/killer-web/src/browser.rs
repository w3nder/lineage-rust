use glam::Vec3;
use killer_assets::cache::{SourceStamp, decode};
use killer_assets::textures::{TextureData, TextureSource};
use killer_assets::weather::Weather;
use killer_render::camera::Camera;
use killer_render::environment::Environment;
use killer_render::pipeline::DEPTH_FORMAT;
use killer_render::renderer::{Renderer, Viewer};
use std::collections::HashMap;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use crate::camera::{FreeCamera, Movement};

struct PackedTextures {
    textures: HashMap<String, Arc<TextureData>>,
    alpha_maps: HashMap<String, Arc<TextureData>>,
}

impl TextureSource for PackedTextures {
    fn texture(&mut self, path: &str) -> Arc<TextureData> {
        self.textures.get(&path.to_lowercase()).cloned().unwrap_or_else(|| {
            let image = killer_assets::textures::checker_image();
            Arc::new(TextureData { width: image.width, height: image.height,
                format: killer_assets::textures::TextureFormatKind::Rgba8, mips: vec![image.rgba] })
        })
    }

    fn alpha_map(&mut self, path: &str) -> Arc<TextureData> {
        self.alpha_maps.get(&path.to_lowercase()).cloned().unwrap_or_else(|| self.texture(path))
    }
}

#[wasm_bindgen]
pub struct WebClient {
    renderer: Renderer,
    adapter: wgpu::Adapter,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    depth: wgpu::Texture,
    environment: Environment,
    timeenv: Option<Weather>,
    camera: Camera,
    free_camera: FreeCamera,
    movement: Movement,
    runtime: Option<crate::runtime::Runtime>,
    lobby: Option<crate::web_lobby::WebLobby>,
    lobby_ui: crate::lobby_ui::LobbyUi,
    boot_lobby: crate::lobby::Lobby,
    lobby_ui_enabled: bool,
    lobby_shift: bool,
    lobby_actions: Vec<serde_json::Value>,
    table_cache: crate::table_cache::SessionTables,
    audio_enabled: bool,
    parked_audio: Option<crate::audio::Audio>,
    entry_events: crate::entry_events::EntryEvents,
    loading_stats: String,
    background_loading: bool,
    pending_world: Option<(String,Vec3,i32)>,
    loading_art: bool,
    progress: Option<js_sys::Function>,
    pending_events: Vec<killer_world::session::SessionEvent>,
    render_ms: f64,
    hud_ms: f64,
    render_frames: u32,
    scale: f32,
    click: crate::input::ClickTracker,
    ui_drag: bool,
    right_pressed_at: Option<killer_platform::time::Instant>,
    network: crate::network::Network,
    validate: crate::pace::ValidatePacer,
    previous_time: Option<f32>,
}

fn error(value: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&value.to_string())
}

fn depth(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("web-depth"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT, usage: wgpu::TextureUsages::RENDER_ATTACHMENT, view_formats: &[],
    })
}

impl Drop for WebClient {
    fn drop(&mut self) {
        self.renderer.device().destroy();
    }
}

#[wasm_bindgen]
impl WebClient {
    pub fn audio_available(&mut self,available: bool) {
        self.audio_enabled=available;
        if let Some(lobby)=&mut self.lobby {lobby.audio.set_available(available);}
        if let Some(audio)=&mut self.parked_audio {audio.set_available(available);}
        if let Some(runtime)=&mut self.runtime {
            let changed=runtime.audio.available()!=available;
            runtime.audio.set_available(available);
            runtime.ui.audio_volumes=available.then(||runtime.audio.config.volumes.ui_values());
            if changed {
                for combo in &mut runtime.ui.keys.option.audio_page.combos {combo.enabled=available;}
                runtime.ui.keys.option.audio_page.read_volumes(runtime.ui.audio_volumes);
            }
        }
    }

    pub fn audio_ended(&mut self,id: u32) {
        if let Some(audio)=self.active_audio() {audio.ended(id);}
    }

    pub fn audio_commands(&mut self) -> js_sys::Array {
        let commands=js_sys::Array::new();
        if let Some(audio)=self.active_audio() {
            for command in audio.drain() {
                let packet=js_sys::Array::new();
                let (metadata,bytes)=match command {
                    crate::audio::Command::Play {id,encoded,gain,spatial,looping,stream}=>{
                        let spatial=spatial.map(|value|serde_json::json!({"position":value.position,"reference":value.reference,"rolloff":value.rolloff,"pitch":value.pitch}));
                        (serde_json::json!({"kind":"play","id":id,"gain":gain,"spatial":spatial,"looping":looping,"stream":stream}),Some(encoded))
                    },
                    crate::audio::Command::Listener {position,front,up}=>(serde_json::json!({"kind":"listener","position":position,"front":front,"up":up}),None),
                    crate::audio::Command::Position {id,position,pitch}=>(serde_json::json!({"kind":"position","id":id,"position":position,"pitch":pitch}),None),
                    crate::audio::Command::Stop(id)=>(serde_json::json!({"kind":"stop","id":id}),None),
                    crate::audio::Command::Gain {id,value}=>(serde_json::json!({"kind":"gain","id":id,"value":value}),None),
                };
                packet.push(&JsValue::from_str(&metadata.to_string()));
                if let Some(bytes)=bytes {packet.push(&js_sys::Uint8Array::from(bytes.as_slice()));}
                commands.push(&packet);
            }
        }
        commands
    }

    pub fn performance_stats(&mut self) -> String {
        let frames=self.render_frames.max(1) as f64;
        let (stages,entities)=self.runtime.as_mut().map_or(([0.0;4],0),|runtime| {
            let count=runtime.profile_frames.max(1) as f64;
            runtime.profile_frames=0;
            let profile=std::mem::replace(&mut runtime.profile,[0.0;4]);
            (profile.map(|value|value/count),runtime.world.entities.len())
        });
        let ground=self.runtime.as_ref().and_then(crate::runtime::Runtime::ground_diagnostic);
        let panels:Vec<_>=self.runtime.as_ref().map(|runtime|runtime.ui.desktop.windows.iter().filter(|window|window.visible).map(|window|[window.rect.x,window.rect.y,window.rect.w,window.rect.h]).collect()).unwrap_or_default();
        let result=serde_json::json!({"panels":panels,"ground":ground,"stages":stages,"render":self.render_ms/frames,"hud":self.hud_ms/frames,"entities":entities,"pieces":self.renderer.instance_count(),"simd":cfg!(target_feature="simd128")});
        self.render_ms=0.0;self.hud_ms=0.0;self.render_frames=0;
        result.to_string()
    }

    pub fn remote_stats(&self) -> String {
        let (requests, bytes, ms) = killer_platform::fs::remote_stats();
        serde_json::json!({"requests": requests, "bytes": bytes, "ms": ms}).to_string()
    }

    pub fn replace_file(&mut self, path: &str, file: wasm_bindgen::JsValue) -> bool { killer_platform::fs::replace(path, &file) }

    pub fn stale_paths(&self) -> String { serde_json::json!(killer_platform::fs::take_stale_paths()).to_string() }

    pub fn refresh_file(&mut self, path: &str, file: wasm_bindgen::JsValue) -> bool {
        let refreshed = killer_platform::fs::refresh(path, &file);
        if refreshed { self.table_cache.clear(); }
        refreshed
    }

    pub fn opened_paths(&self) -> String {serde_json::json!(killer_platform::fs::opened_paths()).to_string()}

    pub fn settings_revision(&self) -> u32 {killer_platform::settings_revision()}

    pub fn settings(&self) -> String {serde_json::json!(killer_platform::settings()).to_string()}

    pub fn restore_settings(&mut self,text: &str) -> Result<(),JsValue> {
        let files=serde_json::from_str(text).map_err(error)?;
        killer_platform::restore_settings(files).map_err(error)?;
        self.table_cache.clear();
        Ok(())
    }

    pub fn preferences(&self) -> String { killer_platform::hud_preferences() }

    pub fn disconnect(&mut self) {
        let loading = self.pending_world.take().is_some();
        self.loading_art=false;
        self.pending_events.clear();
        if let Some(runtime)=&mut self.runtime {runtime.audio.stop();}
        self.network=Default::default();
        self.entry_events.clear();
        if let Some(runtime)=&mut self.runtime {runtime.online=false;}
        self.boot_lobby.back_to_login(String::new());
        if let Some(lobby)=&mut self.lobby {lobby.lobby.back_to_login(String::new());}
        else if (self.runtime.is_some() || loading) && let Err(reason)=self.load_lobby() {log::error!("retorno ao lobby: {reason:?}");}
    }

    pub fn restore_preferences(&mut self, text: String) { killer_platform::set_hud_preferences(text); }

    pub fn login_start(&mut self, account: String, mut password: String) -> Result<(),JsValue> {
        if self.runtime.is_some() {self.load_lobby()?;}
        self.entry_events.clear();
        let result=self.network.start(account,&password);
        password.clear();
        result.map_err(error)
    }

    pub fn login_receive(&mut self, bytes: &[u8]) -> Result<String,JsValue> {
        let mut result=self.network.login_receive(bytes).map_err(error)?;
        if result["notice"] == true && let Some(notice)=self.network.notice.take() {
            if self.c4_lobby() {
                self.lobby_state().show_notice(notice);
            } else {
                let frame=self.network.accept_notice().map_err(error)?;
                if let Some(list)=result["outgoing"].as_array_mut() {list.push(serde_json::json!(frame));}
            }
        }
        if let Some(card)=self.network.card_event.take() {self.lobby_state().pending_card=Some(card);}
        if result["servers"] == true {
            let servers=self.network.servers.clone();
            let lobby=self.lobby_state();
            lobby.login.connecting=false;
            lobby.set_servers(&servers);
        }
        Ok(result.to_string())
    }

    pub fn choose_server(&mut self, id: u8) -> Result<Vec<u8>,JsValue> {self.network.choose_server(id).map_err(error)}

    pub fn game_start(&mut self) -> Result<Vec<u8>,JsValue> {
        if let Some(runtime)=&mut self.runtime {runtime.begin_game().map_err(error)?;}
        self.click.cancel();
        self.ui_drag=false;
        self.validate=Default::default();
        self.network.start_game().map_err(error)
    }

    pub fn preview_character(&mut self,slot: usize) {
        if let Some(lobby)=&mut self.lobby {lobby.lobby.select(slot);}
    }

    pub fn choose_character(&mut self,slot: i32) -> Result<(),JsValue> {self.network.command(killer_world::NetCommand::Select(slot)).map_err(error)}

    pub fn game_receive(&mut self,bytes: &[u8]) -> Result<String,JsValue> {
        let result=self.network.receive_game(bytes).map_err(error)?;
        self.apply_network_events(result)
    }

    fn apply_network_events(&mut self, mut result: serde_json::Value) -> Result<String,JsValue> {
        let mut events = std::mem::take(&mut self.network.events).into_iter();
        while let Some(event) = events.next() {
            match &event {
                killer_world::session::SessionEvent::Characters(characters) => {
                    self.entry_events.clear();
                    if self.runtime.is_some() {self.load_lobby()?;}
                    self.lobby_state().set_characters(characters.clone());
                }
                killer_world::session::SessionEvent::RestartResponse(response) if response.result != 0 => {
                    self.entry_events.clear();
                    self.load_lobby()?;
                    if let Some(lobby)=&mut self.lobby {lobby.lobby.enter(crate::lobby::LobbyScreen::Characters);}
                }
                killer_world::session::SessionEvent::CharTemplates(templates) => {
                    let lobby = self.lobby_state();
                    lobby.set_templates(templates);
                    lobby.new_character_result(true);
                }
                killer_world::session::SessionEvent::World(killer_world::WorldEvent::NewCharacterFailNoop(_)) => self.lobby_state().new_character_result(false),
                killer_world::session::SessionEvent::CharCreateFailed(reason) => {
                    self.lobby_state().create_failed(*reason);
                }
                killer_world::session::SessionEvent::CharDeleteFailed(reason) => {
                    self.lobby_state().delete_failed(*reason);
                }
                killer_world::session::SessionEvent::CharCreated | killer_world::session::SessionEvent::CharDeleted => {
                    let lobby=self.lobby_state();
                    lobby.enter(crate::lobby::LobbyScreen::Characters);
                    lobby.status=if matches!(event,killer_world::session::SessionEvent::CharCreated) {"personagem criado"} else {"personagem excluído"}.into();
                }
                _ => {}
            }
            if self.runtime.is_none() && let killer_world::session::SessionEvent::UserInfo(info)=&event {
                let spawn=Vec3::new(info.x as f32,info.y as f32,info.z as f32);
                let tile = killer_assets::tiles::tile_name(spawn.x,spawn.y);
                if self.background_loading {
                    self.pending_world = Some((tile.clone(),spawn,info.heading));
                    self.pending_events = std::iter::once(event).chain(events).collect();
                    if let Some(mut lobby) = self.lobby.take() { lobby.release(&mut self.renderer); self.parked_audio=Some(lobby.audio); }
                    self.renderer.clear();
                    self.renderer.clear_ui();
                    self.show_loading_art();
                    result["loadWorld"] = serde_json::json!(tile);
                    result["entered"] = serde_json::json!(false);
                    break;
                }
                self.load_world(&tile,Some((spawn,info.heading)),None,None)?;
            }
            if let Some(runtime)=&mut self.runtime {
                for pending in self.entry_events.take() {runtime.apply(&pending,&mut self.renderer).map_err(error)?;}
                let entering=runtime.world.me.is_none();
                let teleported=matches!(&event,killer_world::session::SessionEvent::World(killer_world::WorldEvent::TeleportToLocation(value)) if runtime.world.me.as_ref().is_some_and(|me|me.object_id==value.target_id));
                runtime.apply(&event,&mut self.renderer).map_err(error)?;
                if matches!(event,killer_world::session::SessionEvent::ServerClosed) {runtime.world.dialogs.push(killer_world::Dialog::Disconnected {reason:2});}
                if let (killer_world::session::SessionEvent::World(killer_world::WorldEvent::SsqInfo(info)),Some(weather),Some(clock))=(&event,&mut self.timeenv,&runtime.world.clock) {
                    let hour=clock.hour(killer_platform::time::Instant::now());
                    if let Err(failure)=weather.change_env(info.env(),info.immediate(),0.02,0.02,1.0,hour) {log::warn!("SSQInfo: ChangeEnv recusado: {failure:?}");}
                }
                if teleported && !runtime.world.teleport_announce {runtime.teleported();}
                if entering && matches!(event,killer_world::session::SessionEvent::UserInfo(_)) {
                    if let Some(me)=&runtime.world.me {
                        self.network.command(killer_world::NetCommand::Validate {position:me.position.to_array().map(|v|v as i32),heading:me.heading}).map_err(error)?;
                    }
                    self.network.command(killer_world::NetCommand::Appearing).map_err(error)?;
                }
                if let Some(action)=runtime.world.pending_client_action.take() && let Some(command)=runtime.world.client_action(action,runtime.ui.keys.shift) {runtime.world.requests.push(command);}
                for command in runtime.world.take_requests() {self.network.command(command).map_err(error)?;}
            } else {self.entry_events.push(event);}
        }
        Ok(result.to_string())
    }

    fn lobby_state(&mut self) -> &mut crate::lobby::Lobby {
        self.lobby.as_mut().map_or(&mut self.boot_lobby, |lobby| &mut lobby.lobby)
    }

    pub fn lobby_actions(&mut self) -> String { serde_json::json!(std::mem::take(&mut self.lobby_actions)).to_string() }

    pub fn lobby_error(&mut self, message: String) {
        if self.network.disconnect_pending {return;}
        let refusal=self.network.take_refusal();
        let blocked=self.network.take_blocked();
        let text=(!self.modern_lobby()).then_some(()).and(refusal).and_then(|code|self.lobby.as_ref().and_then(|lobby|lobby.refusal_text(code)))
            .or_else(||(!self.modern_lobby()).then_some(()).and(blocked).and_then(|(flags,messages)|self.lobby.as_ref().and_then(|lobby|lobby.blocked_text(flags,&messages))));
        self.lobby_state().back_to_login(text.unwrap_or(message));
    }

    pub fn focus(&mut self, focused: bool) { self.lobby_ui.input.focus(focused); }

    pub fn paste(&mut self, text: String) {
        if self.runtime.is_none() { self.lobby_ui.input.paste(text); }
    }

    fn show_loading_art(&mut self) {
        if self.modern_lobby() {return;}
        let Ok(mut client)=killer_assets::client::Client::open(std::path::Path::new("/client")) else {return};
        let Ok(text)=client.read_system_text("localization.ini") else {return};
        let Some(texture)=killer_ui::loading::loading_texture(&killer_ui::ini::parse_ini(&text)) else {return};
        let mut list=killer_ui::draw::DrawList::default();
        killer_ui::loading::paint_loading(&mut list,0,0,self.config.width as i32,self.config.height as i32,&texture);
        let mut bank=killer_assets::textures::TextureBank::new();
        self.renderer.set_ui_size(None);
        self.renderer.set_ui(&list,&mut killer_assets::textures::BankSource {bank:&mut bank,catalog:client.catalog()});
        self.loading_art=true;
    }

    fn modern_lobby(&self) -> bool {
        if killer_platform::hud_preferences().contains("modern=") {crate::ui_modern::HudPreferences::load().enabled} else {false}
    }

    fn c4_lobby(&self) -> bool {
        self.runtime.is_none() && self.lobby_ui_enabled && !self.modern_lobby() && self.lobby.as_ref().is_some_and(|lobby|lobby.lobby.view.is_some())
    }

    fn lobby_command(&mut self, command: crate::lobby::LobbyCommand) -> Result<(), JsValue> {
        use crate::lobby::LobbyCommand;
        match command {
            LobbyCommand::Connect {account,password,..} => {
                self.login_start(account,password)?;
                self.lobby_actions.push(serde_json::json!({"kind":"connect"}));
            }
            LobbyCommand::CardAnswer(value) => {
                if let Some(card_id)=self.network.card {
                    let bytes=self.network.card_answer(card_id,value).map_err(error)?;
                    self.lobby_actions.push(serde_json::json!({"kind":"send","bytes":bytes}));
                }
            }
            LobbyCommand::NoticeAnswer(agreed) => {
                if agreed {
                    let bytes=self.network.accept_notice().map_err(error)?;
                    self.lobby_actions.push(serde_json::json!({"kind":"send","bytes":bytes}));
                }
            }
            LobbyCommand::ChooseServer(id) => {
                let bytes=self.choose_server(id)?;
                self.lobby_actions.push(serde_json::json!({"kind":"send","bytes":bytes}));
            }
            LobbyCommand::Select(slot) => {
                if self.lobby.is_some() { self.choose_character(slot as i32)?; }
                else { self.boot_lobby.busy=false; self.boot_lobby.status="aguardando carregamento…".into(); }
            }
            LobbyCommand::NewCharacter => self.network.command(killer_world::NetCommand::NewCharacter).map_err(error)?,
            LobbyCommand::Create(character) => self.network.command(killer_world::NetCommand::CharacterCreate(character)).map_err(error)?,
            LobbyCommand::Delete(slot) => self.network.command(killer_world::NetCommand::CharacterDelete {slot:slot as i32}).map_err(error)?,
            LobbyCommand::Restore(slot) => self.network.command(killer_world::NetCommand::CharacterRestore {slot:slot as i32}).map_err(error)?,
            LobbyCommand::Back | LobbyCommand::Exit => {
                self.network.command(killer_world::NetCommand::Logout).map_err(error)?;
                let bytes=self.network.drain();
                self.disconnect();
                self.lobby_actions.push(serde_json::json!({"kind":if matches!(command,LobbyCommand::Exit) {"exit"} else {"back"},"packets":bytes}));
            }
        }
        Ok(())
    }

    pub fn outgoing(&mut self) -> String {serde_json::json!(self.network.drain()).to_string()}
    pub async fn create(canvas: web_sys::OffscreenCanvas, scale: f32) -> Result<WebClient, JsValue> {
        console_error_panic_hook::set_once();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas.clone())).map_err(error)?;
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface), ..Default::default()
        }).await.map_err(error)?;
        let required_features = adapter.features() & wgpu::Features::TEXTURE_COMPRESSION_BC;
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            required_features, ..Default::default()
        }).await.map_err(error)?;
        let mut config = surface.get_default_config(&adapter, canvas.width().max(1), canvas.height().max(1))
            .ok_or_else(|| error("WebGPU não oferece formato para o canvas"))?;
        config.format = surface.get_capabilities(&adapter).formats.into_iter()
            .find(|format| format.is_srgb()).unwrap_or(config.format);
        surface.configure(&device, &config);
        let depth = depth(&device, config.width, config.height);
        let renderer = Renderer::new(device, queue, config.format);
        let lobby_ui = crate::lobby_ui::LobbyUi::new(&renderer, config.format);
        let direction = Vec3::new(2400.0, 2000.0, -1200.0).normalize();
        Ok(Self { renderer, adapter, surface, config, depth, environment: Environment::default(),timeenv:None,
            free_camera: FreeCamera { position: Vec3::new(81000.0,146000.0,-2200.0), yaw: direction.y.atan2(direction.x), pitch: direction.z.asin() },
            movement: Movement::default(),
            runtime: None, lobby:None, lobby_ui, boot_lobby:crate::lobby::Lobby::new(Default::default()), lobby_ui_enabled:true, lobby_shift:false, lobby_actions:Vec::new(), table_cache:Default::default(), audio_enabled:false, parked_audio:None, entry_events:Default::default(), loading_stats:String::new(), background_loading:false, pending_world:None, loading_art:false, progress:None, pending_events:Vec::new(), render_ms:0.0,hud_ms:0.0,render_frames:0,scale: scale.max(0.1), click: Default::default(), ui_drag: false, right_pressed_at: None,
            network: Default::default(),
            validate: Default::default(),
            camera: Camera { eye: Vec3::new(81000.0,146000.0,-2200.0), target: Vec3::new(83400.0,148000.0,-3400.0),
                fov_x_deg: 60.0, near: 10.0, far: 60000.0 }, previous_time: None })
    }

    pub fn load_scene(&mut self, name: &str, bytes: &[u8], size: u64, hash: u64) -> Result<(), JsValue> {
        let cached = decode(bytes, &SourceStamp { size, hash }).map_err(|e| error(format!("cena: {e:?}")))?;
        let mut textures = PackedTextures { textures: cached.textures.into_iter().collect(), alpha_maps: cached.alpha_maps.into_iter().collect() };
        self.renderer.add_tile(name, &cached.scene, &mut textures);
        Ok(())
    }

    pub fn game_cursors(&self) -> Result<js_sys::Array, JsValue> {
        let mut client = killer_assets::client::Client::open(std::path::Path::new("/client")).map_err(error)?;
        let bytes = client.read_system_file("nwindow.dll").map_err(error)?;
        let cursors = killer_assets::cursor::load(&bytes).ok_or_else(|| error("Cursores originais indisponíveis"))?;
        Ok(cursors.into_iter().map(|cursor| js_sys::Uint8Array::from(cursor.encode().as_slice())).collect())
    }

    pub fn game_cursor_request(&self) -> String {
        self.runtime.as_ref().map_or_else(|| "0".into(), |runtime| if runtime.cursor.is_none() { String::new() } else { crate::cursor::select(&runtime.ui, &runtime.world, runtime.hovered).key() })
    }

    pub fn game_cursor_texture(&mut self, path: &str) -> Result<Vec<u8>, JsValue> {
        let runtime = self.runtime.as_mut().ok_or_else(|| error("Mundo indisponível"))?;
        runtime.client.catalog().texture_raw(path).and_then(|raw| killer_assets::cursor::from_texture(&raw))
            .map(|cursor| cursor.encode()).ok_or_else(|| error("Textura de cursor indisponível"))
    }

    pub fn mount_files(&mut self, files: js_sys::Array) -> Result<usize, JsValue> {
        let count = killer_platform::fs::mount(&files)?;
        self.table_cache.clear();
        Ok(count)
    }

    pub fn set_progress(&mut self, callback: js_sys::Function) { self.progress = Some(callback); }

    pub fn loading_image(&self) -> Result<Vec<u8>, JsValue> {
        let mut client = killer_assets::client::Client::open(std::path::Path::new("/client")).map_err(error)?;
        let localization = killer_ui::ini::parse_ini(&client.read_system_text("localization.ini").map_err(error)?);
        let texture = killer_ui::loading::loading_texture(&localization).ok_or_else(|| error("textura de loading não definida no localization.ini"))?;
        let raw = client.catalog().texture_raw(&texture).ok_or_else(|| error(format!("textura de loading ausente: {texture}")))?;
        killer_assets::loading_image::from_texture(&raw).map(|image| image.encode()).ok_or_else(|| error("textura de loading inválida"))
    }

    pub fn load_lobby(&mut self) -> Result<(),JsValue> { self.install_lobby(None) }

    pub fn load_prepared_lobby(&mut self, scene: web_sys::Blob) -> Result<(),JsValue> { self.install_lobby(Some(scene)) }

    fn install_lobby(&mut self, prepared: Option<web_sys::Blob>) -> Result<(),JsValue> {
        self.entry_events.clear();
        if let Some(mut runtime)=self.runtime.take() {
            runtime.audio.stop();
            self.parked_audio=Some(runtime.audio);
            self.renderer=Renderer::new(self.renderer.device().clone(),self.renderer.queue().clone(),self.config.format);
        }
        self.click.cancel();
        self.ui_drag=false;
        self.validate=Default::default();
        if let Some(mut lobby)=self.lobby.take() {lobby.release(&mut self.renderer);self.parked_audio=Some(lobby.audio);}
        let progress=self.progress.clone();
        let report=move |stage: &str, ms: f64| if let Some(callback)=&progress {let _=callback.call2(&JsValue::NULL,&JsValue::from_str(stage),&JsValue::from_f64(ms));};
        let mut lobby=crate::web_lobby::WebLobby::load(std::mem::replace(&mut self.boot_lobby,crate::lobby::Lobby::new(Default::default())),&mut self.renderer,prepared,&mut self.table_cache,&report).map_err(error)?;
        if let Some(audio)=self.parked_audio.take() {lobby.audio.inherit(audio,true);}
        lobby.audio.set_available(self.audio_enabled);
        if lobby.lobby.screen==crate::lobby::LobbyScreen::Servers {lobby.lobby.set_servers(&self.network.servers);}
        self.loading_stats=lobby.loading_stats.clone();
        self.lobby=Some(lobby);
        self.movement=Default::default();
        Ok(())
    }

    pub fn loading_stats(&self) -> String {self.loading_stats.clone()}

    pub fn load_tile(&mut self, name: &str) -> Result<(), JsValue> {self.load_world(name,None,None,None)}

    pub fn enable_background_loading(&mut self) { self.background_loading = true; }

    pub fn finish_world(&mut self, scene: Option<web_sys::Blob>, trace: Option<web_sys::Blob>) -> Result<String,JsValue> {
        let (tile,spawn,heading) = self.pending_world.clone().ok_or_else(|| error("nenhum carregamento pendente"))?;
        self.load_world(&tile,Some((spawn,heading)),scene,trace)?;
        self.pending_world = None;
        self.loading_art=false;
        self.network.events = std::mem::take(&mut self.pending_events);
        self.apply_network_events(serde_json::json!({"entered":true}))
    }

    pub fn load_prepared_tile(&mut self, name: &str, scene: web_sys::Blob, trace: web_sys::Blob) -> Result<(),JsValue> {
        self.load_world(name,None,Some(scene),Some(trace))
    }
}

impl WebClient {
    fn active_audio(&mut self) -> Option<&mut crate::audio::Audio> {
        if let Some(runtime)=&mut self.runtime {Some(&mut runtime.audio)}
        else if let Some(lobby)=&mut self.lobby {Some(&mut lobby.audio)}
        else {self.parked_audio.as_mut()}
    }

    fn load_world(&mut self, name: &str, spawn: Option<(Vec3,i32)>, prepared: Option<web_sys::Blob>, trace: Option<web_sys::Blob>) -> Result<(), JsValue> {
        let started = killer_platform::time::Instant::now();
        let progress = self.progress.clone();
        let report = move |stage: &str, ms: f64| if let Some(callback) = &progress { let _ = callback.call2(&JsValue::NULL, &JsValue::from_str(&format!("world-{stage}")), &JsValue::from_f64(ms)); };
        use killer_assets::client::Client;
        use killer_assets::options::ClientOptions;
        use killer_assets::scene::{Neighbours, build_scene_with, sky};
        use killer_assets::textures::{BankSource, TextureBank};
        if let Some(mut lobby)=self.lobby.take() {lobby.release(&mut self.renderer);self.parked_audio=Some(lobby.audio);}
        if let Some(runtime)=self.runtime.take() {self.parked_audio=Some(runtime.audio);}
        let mut client = Client::open(std::path::Path::new("/client")).map_err(error)?;
        let loaded = client.load_level(name).map_err(error)?;
        let options = ClientOptions::load(&mut client).map_err(error)?;
        let environment = Weather::load(&mut client).map_err(error)?;
        self.environment = Environment::from_time(environment.base(), 12.0);
        self.timeenv=Some(environment);
        let video = crate::runtime::options::Video::load(&mut client,&options,&self.renderer,&self.adapter,(self.config.width,self.config.height)).map_err(error)?;
        let read_ms = started.elapsed().as_secs_f64() * 1000.0;
        report("read", read_ms);
        let mut scene = if let Some(file) = prepared {
            let reader = std::io::BufReader::with_capacity(1 << 20, killer_platform::fs::File::from_blob(file));
            let mut scene = decode(lz4_flex::frame::FrameDecoder::new(reader), &crate::preparation::STAMP).map_err(|e| error(format!("cache de cena: {e:?}")))?.scene;
            scene.emitters = killer_assets::scene::emitters::build(&mut client,&loaded.level,&options);
            scene
        } else { build_scene_with(&mut client, &loaded.level, &options, Neighbours::default()) };
        let scene_ms = started.elapsed().as_secs_f64() * 1000.0;
        report("scene", scene_ms);
        let mut bank = TextureBank::new();
        self.renderer.upload_with_texture_detail(&scene, &mut BankSource { bank: &mut bank, catalog: client.catalog() }, video.texture_detail());
        if let Some(scene) = options.shared_sky.then(|| sky::load_shared(&mut client, &options)).flatten() {
            self.renderer.add_tile("sky", &scene, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        }
        drop(bank);
        scene.lightmaps = Vec::new();
        client.catalog().release_files();
        let upload_ms = started.elapsed().as_secs_f64() * 1000.0;
        report("upload", upload_ms);
        let mut runtime = crate::runtime::Runtime::new(client,loaded.level,scene,&self.renderer,self.config.format,(self.config.width,self.config.height),self.scale,spawn,trace,&mut self.table_cache).map_err(error)?;
        let runtime_ms = started.elapsed().as_secs_f64() * 1000.0;
        report("runtime", runtime_ms);
        runtime.initialize_video(&mut self.renderer,&self.adapter,video).map_err(error)?;
        if let Some(audio)=self.parked_audio.take() {runtime.audio.inherit(audio,false);}
        if let Some(console)=&mut runtime.audio.console {console.state_38=6;}
        runtime.audio.set_available(self.audio_enabled);
        self.loading_stats=serde_json::json!({"tile":name,"read_ms":read_ms,"scene_ms":scene_ms-read_ms,"upload_ms":upload_ms-scene_ms,"runtime":runtime.loading_profile,"video_ms":started.elapsed().as_secs_f64()*1000.0-runtime_ms,"total_ms":started.elapsed().as_secs_f64()*1000.0}).to_string();
        self.runtime=Some(runtime);
        Ok(())
    }

}

#[wasm_bindgen]
impl WebClient {
    pub fn set_environment(&mut self, text: &str, hour: f32) {
        self.lobby_ui_enabled=false;
        self.environment = Environment::from_time(&killer_assets::timeenv::TimeEnv::parse(text), hour);
    }

    pub fn look(&mut self, dx: f32, dy: f32) {
        if self.lobby.is_some() {return;}
        if let Some(runtime) = &mut self.runtime { runtime.orbit.drag(dx,dy); }
        else { self.free_camera.look(dx, dy); }
    }

    pub fn movement(&mut self, forward: bool, back: bool, left: bool, right: bool, fast: bool) {
        self.movement = Movement { forward, back, left, right, fast };
        if let Some(runtime)=&mut self.runtime {runtime.keyboard_moving=(forward || back) && !runtime.ui.chat_focused();}
    }

    #[allow(clippy::too_many_arguments)]
    pub fn pointer(&mut self, kind: &str, x: f32, y: f32, dx: f32, dy: f32, buttons: u32, button: i32) {
        if self.c4_lobby() {
            if let Some(lobby)=&mut self.lobby {lobby.c4_pointer(kind,x,y,button);}
            return;
        }
        if self.runtime.is_none() && self.lobby_ui_enabled {self.lobby_ui.input.pointer(kind,x,y,button);return;}
        let Some(runtime) = &mut self.runtime else {
            if kind == "move" && buttons != 0 {self.free_camera.look(dx,dy);}
            return;
        };
        if kind=="leave" {runtime.cursor=None;return;}
        runtime.cursor=Some((x,y));
        match kind {
            "down" => {
                self.ui_drag = button == 0 && runtime.ui.mouse_down_at(x as i32,y as i32,killer_platform::time::Instant::now(),&runtime.world);
                if button==2 {
                    let handled=runtime.ui.right_click(x as i32,y as i32,&runtime.world);
                    self.ui_drag=handled || runtime.ui.window_under(x as i32,y as i32).is_some();
                    self.right_pressed_at=(!self.ui_drag).then(killer_platform::time::Instant::now);
                }
                if button==1 && runtime.orbit.settings.turn_back_on_middle && runtime.ui.window_under(x as i32,y as i32).is_none()
                    && let Some(me)=&runtime.world.me && runtime.orbit.turn_back(me.heading)
                    && let Some(command)=runtime.ui.tutorial.fire(killer_ui::tutorial::EVENT_TURN_BACK) {runtime.ui.keys.send(command);}

                if let Some(action)=runtime.ui.take_board_command() {runtime.action(action);}
                if std::mem::take(&mut runtime.ui.target_clicked) && let Some(target)=runtime.world.target {runtime.action(crate::ui2d::KeyAction::Select(target));}
                if let Some(action)=runtime.ui.take_status_click(&runtime.world) {runtime.action(action);}
                runtime.party_press();
                if !self.ui_drag && button == 0 {self.click.press(Some((x,y)));}
            }
            "move" => {
                self.click.moved((x,y));
                runtime.ui.mouse_move(x as i32,y as i32);
                if buttons & 3 != 0 && !self.ui_drag && !(runtime.world.camera_fixed && runtime.world.pending_click_target.is_some()) {
                    runtime.orbit.drag(dx,dy);
                    let (yaw,pitch)=runtime.orbit.settings.drag_radians(dx,dy);
                    for delta in [yaw,pitch] {
                        if let Some(command)=runtime.ui.tutorial.camera_rotated(delta) {runtime.ui.keys.send(command);}
                    }
                }
            }
            "up" => {
                if button==2 {runtime.party_assist(x as i32,y as i32);}
                if button==2 && let Some(since)=self.right_pressed_at.take()
                    && runtime.orbit.settings.fixed_on_right_release_ms.is_some_and(|max|since.elapsed().as_secs_f32()*1000.0<=max)
                    && let Some(me)=&runtime.world.me && runtime.orbit.fixed_default_camera(me.heading,0)
                    && let Some(command)=runtime.ui.tutorial.fire(killer_ui::tutorial::EVENT_DEFAULT_CAMERA) {runtime.ui.keys.send(command);}
                if button==2 {runtime.ui.right_released(x as i32,y as i32,&runtime.world);}
                runtime.release();
                if !self.ui_drag && button == 0 && let Some(pixel) = self.click.release()
                    && let Err(reason)=runtime.click(&self.camera,pixel,&self.renderer) {log::error!("{reason}");}
                self.ui_drag = false;
            }
            "cancel" => {self.right_pressed_at=None;self.click.cancel();self.ui_drag=false;runtime.ui.mouse_up_in(&runtime.world);}
            _ => {}
        }
    }

    pub fn wheel(&mut self, x: i32, y: i32, ticks: f32) {
        if self.c4_lobby() {
            if let Some(lobby)=&mut self.lobby && !lobby.c4_wheel_at(x as f32,y as f32,ticks) {lobby.c4_wheel(ticks);}
            return;
        }
        if self.runtime.is_none() && self.lobby_ui_enabled {self.lobby_ui.input.wheel(ticks);return;}
        if let Some(runtime) = &mut self.runtime && !runtime.ui.wheel_over(x,y,ticks,&runtime.chat) {
            runtime.orbit.wheel(ticks,runtime.world.me.as_ref().map(|me|me.collision_radius));
            if let Some(command)=runtime.ui.tutorial.fire(killer_ui::tutorial::EVENT_ZOOM) {runtime.ui.keys.send(command);}
        }
    }

    pub fn keyboard(&mut self, text: &str, vk: u32, alt: bool, pressed: bool, repeat: bool) {
        if self.c4_lobby() {
            if vk==0x10 {self.lobby_shift=pressed;}
            let shift=self.lobby_shift;
            if pressed && !alt && let Some(lobby)=&mut self.lobby {lobby.c4_key(text,shift);}
            return;
        }
        if self.runtime.is_none() && self.lobby_ui_enabled {self.lobby_ui.input.keyboard(text,vk,alt,pressed,repeat);return;}
        let Some(runtime) = &mut self.runtime else {return};
        runtime.ui.keys.alt=alt;
        if vk==0x11 {runtime.ui.keys.ctrl=pressed;}
        if vk==0x10 {runtime.ui.keys.shift=pressed;runtime.ui.shift_changed(pressed);}
        let msg = match (alt,pressed) {(true,true)=>0x104,(true,false)=>0x105,(false,true)=>0x100,(false,false)=>0x101};
        let mut lparam = if vk == 0x79 {0x44 << 16} else {0};
        if alt {lparam |= 1 << 29;}
        if repeat || !pressed {lparam |= 1 << 30;}
        if !pressed {lparam |= 1 << 31;}
        if runtime.ui.hook_key(msg,vk,lparam,&runtime.world) {return;}
        if !pressed {
            if !runtime.ui.chat_focused() {runtime.key_bind(vk,false);}
            return;
        }
        if alt {runtime.key_bind(vk,true);return;}
        use killer_ui::chat::InputKey;
        let key = match text {
            "Enter"=>Some(InputKey::Enter),"Escape"=>Some(InputKey::Escape),"Backspace"=>Some(InputKey::Backspace),
            "ArrowLeft"=>Some(InputKey::Left),"ArrowRight"=>Some(InputKey::Right),
            "Home"=>Some(InputKey::Home),"End"=>Some(InputKey::End),"Delete"=>Some(InputKey::Delete),
            "ArrowUp"=>Some(InputKey::Up),"ArrowDown"=>Some(InputKey::Down),
            _=>text.chars().next().filter(|_|text.chars().count()==1).map(|c|if matches!(c,'\n'|'\r') {InputKey::Enter} else {InputKey::Char(c)}),
        };
        let action=key.map_or(crate::ui2d::KeyAction::Ignored,|key|runtime.ui.key(key));
        if action==crate::ui2d::KeyAction::Ignored {runtime.key_bind(vk,true);}
        runtime.action(action);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(self.renderer.device(), &self.config);
        self.depth = depth(self.renderer.device(), self.config.width, self.config.height);
        if let Some(runtime) = &mut self.runtime {runtime.resize((self.config.width,self.config.height),self.scale);}
    }

    pub fn frame(&mut self, seconds: f32) -> Result<(), JsValue> {
        let dt = self.previous_time.map_or(0.0, |previous| crate::pace::engine_delta(std::time::Duration::from_secs_f32((seconds-previous).max(0.0))));
        self.previous_time = Some(seconds);
        self.free_camera.advance(&self.movement, dt);
        self.camera = self.free_camera.camera();
        self.renderer.set_time(seconds);
        self.renderer.set_viewer(Viewer { zone: 0, position: self.camera.eye, dt });
        if let Some(runtime) = &mut self.runtime {
            if let (Some(weather),Some(clock))=(&mut self.timeenv,&runtime.world.clock) {
                let hour=clock.hour(killer_platform::time::Instant::now());
                weather.state.update(hour);
                self.environment=Environment::from_weather(weather,hour);
            }
            let me_before=runtime.world.me.as_ref().map(|me|me.position);
            self.camera = runtime.tick(&mut self.renderer,dt,&self.environment);
            let commands=runtime.commands(&mut self.renderer,&self.adapter,&self.camera);
            if runtime.online && runtime.world.me.is_some() {
                for command in commands {self.network.command(command).map_err(error)?;}
            }
            if runtime.online && let Some(me)=&runtime.world.me && self.validate.poll(dt,crate::pace::moved(me_before,Some(me.position))) {
                self.network.command(killer_world::NetCommand::Validate {position:me.position.to_array().map(|v|v as i32),heading:me.heading}).map_err(error)?;
            }
        }
        if let Some(lobby)=&mut self.lobby {
            lobby.tick(&mut self.renderer,dt,(self.config.width,self.config.height));
            self.camera=lobby.lobby.camera();
            if let Some(view)=&lobby.lobby.view {self.environment=Environment::from_time(&view.timeenv,lobby.lobby.hour());}
        }
        let c4=self.c4_lobby();
        let mut lobby_commands=Vec::new();
        if c4 && let Some(lobby)=&mut self.lobby {
            let ui_size=((self.config.width as f32/self.scale) as i32,(self.config.height as f32/self.scale) as i32);
            lobby.c4_frame(&mut self.renderer,&self.adapter,ui_size,(dt*1000.0) as u32,(self.config.width,self.config.height));
            lobby_commands=lobby.c4_commands();
        }
        let prepare_started=killer_platform::time::Instant::now();
        let hud=self.runtime.as_mut().map(|runtime|runtime.prepare_hud(&mut self.renderer,(self.config.width,self.config.height),f64::from(seconds)));
        if hud.is_none() {self.renderer.clear_world_text();}
        self.hud_ms+=prepare_started.elapsed().as_secs_f64()*1000.0;
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(output) | wgpu::CurrentSurfaceTexture::Suboptimal(output) => output,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(self.renderer.device(), &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return Ok(()),
            wgpu::CurrentSurfaceTexture::Validation => return Err(error("superfície WebGPU inválida")),
        };
        let view = output.texture.create_view(&Default::default());
        let depth = self.depth.create_view(&Default::default());
        let mut encoder = self.renderer.device().create_command_encoder(&Default::default());
        let render_started=killer_platform::time::Instant::now();
        if let Some(runtime)=self.runtime.as_mut() {runtime.render_fishing(&mut self.renderer,&self.camera,&self.environment);}
        self.renderer.render(&mut encoder, &view, &depth, (self.config.width, self.config.height), &self.camera, &self.environment);
        self.render_ms+=render_started.elapsed().as_secs_f64()*1000.0;
        let hud_started=killer_platform::time::Instant::now();
        let mut commands = self.runtime.as_mut().zip(hud).map(|(runtime,hud)|runtime.paint(&self.renderer,&mut encoder,&view,(self.config.width,self.config.height),hud)).unwrap_or_default();
        if self.runtime.is_none() && self.lobby_ui_enabled && !c4 {
            let size=(self.config.width,self.config.height);
            self.lobby_ui.begin(size,self.scale,f64::from(seconds));
            if let Some(lobby)=&mut self.lobby {
                lobby_commands=lobby.lobby.panels(&self.lobby_ui.context,env!("CARGO_PKG_VERSION"),(size.0 as f32,size.1 as f32),1.0/self.scale);
            } else if let Some((tile,_,_))=&self.pending_world {
                if !self.loading_art {crate::lobby::panels::loading_world(&self.lobby_ui.context,tile);}
            } else {
                if self.modern_lobby() {
                    lobby_commands=self.boot_lobby.panels(&self.lobby_ui.context,env!("CARGO_PKG_VERSION"),(size.0 as f32,size.1 as f32),1.0/self.scale);
                }
                crate::lobby::panels::loading(&self.lobby_ui.context,"Loading");
            }
            commands.extend(self.lobby_ui.paint(&self.renderer,&mut encoder,&view,size));
        }
        self.hud_ms+=hud_started.elapsed().as_secs_f64()*1000.0;
        self.render_frames+=1;
        self.renderer.queue().submit(commands.into_iter().chain([encoder.finish()]));
        self.renderer.queue().present(output);
        if let Some(runtime)=&mut self.runtime {
            let exit=std::mem::take(&mut runtime.exit_requested);
            if (std::mem::take(&mut runtime.leave_requested) || exit)
                && let Err(reason)=self.lobby_command(if exit {crate::lobby::LobbyCommand::Exit} else {crate::lobby::LobbyCommand::Back}) {
                self.lobby_error(reason.as_string().unwrap_or_else(||"Falha na conexão".into()));
            }
        }
        if self.runtime.as_ref().is_some_and(|runtime|runtime.disconnect_accepted) {
            let text=self.runtime.as_ref().and_then(|runtime|runtime.connection_lost_text());
            self.network.disconnect_pending=false;
            self.lobby_error(text.unwrap_or_else(||"conexão encerrada pelo servidor".into()));
        }
        for command in lobby_commands {
            if let Err(reason)=self.lobby_command(command) {
                self.lobby_error(reason.as_string().unwrap_or_else(||"Falha na conexão".into()));
            }
        }
        Ok(())
    }
}
