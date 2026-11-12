#[path = "options.rs"]
pub(crate) mod options;
use crate::characters::{self, CharacterCache, DisplaySettings, Frame, LoadedCharacter};
use crate::loader::{LoadResult, Shared, load_character};
use crate::ui2d::{self, GameUi};
use crate::ui_modern::ModernHud;
use glam::Vec3;
use killer_assets::client::{Client, DatTables};
use killer_assets::scene::actor_lighting::LightMap;
use killer_assets::textures::{BankSource, TextureBank};
use killer_platform::time::Instant;
use killer_render::camera::Camera;
use killer_render::renderer::Renderer;
use killer_ui::chat::ChatEntry;
use killer_world::World;
use std::sync::Arc;

#[derive(Default)]
struct Marks {
    state: crate::landmark::LandMark,
    active: std::collections::HashMap<crate::landmark::Slot,killer_render::renderer::EffectId>,
    loose: Vec<(killer_render::renderer::EffectId,f32)>,
    spans: std::collections::HashMap<&'static str,f32>,
}

pub struct Runtime {
    pub client: Client,
    pub audio: crate::audio::Audio,
    audio_frame: Option<crate::audio::AudioFrame>,
    audio_time: f64,
    pub world: World,
    pub ui: GameUi,
    pub orbit: crate::camera::OrbitCamera,
    pub ground: crate::ground::GroundGrid,
    pub trace: crate::trace::TraceMesh,
    tables: Arc<DatTables>,
    cache: CharacterCache,
    items: crate::items::Items,
    fences: crate::server_objects::Fences,
    sound_bank: crate::world_sound::SoundBank,
    shakes: killer_world::camera_shake::ShakeSet,
    fade: killer_world::screen_fade::ScreenFade,
    fade_shown: bool,
    effects: crate::pawn_effects::Effects,
    abnormal: crate::abnormal_host::AbnormalStates,
    shared: Shared,
    walk_warned: std::collections::HashSet<String>,
    walk_state: crate::walk_world::WalkState,
    marks: Marks,
    level: Arc<ue2_level::Level>,
    zones: usize,
    neighbours: crate::tiles::NeighbourSet,
    moving_trace: crate::trace::MovingTrace,
    options: killer_assets::options::ClientOptions,
    video: Option<options::Video>,
    labels: Vec<characters::Label>,
    lights: LightMap,
    display: DisplaySettings,
    bank: TextureBank,
    radar_toggle: bool,
    binds: killer_world::engine_input::KeyBinds,
    modern: ModernHud,
    context: egui::Context,
    painter: egui_wgpu::Renderer,
    pub chat: Vec<ChatEntry>,
    crest_inbox: crate::crest_file::Inbox,
    scale: f32,
    tile: String,
    banks_age: f32,
    zone_music: crate::zone_music::ZoneMusic,
    music_zone: Option<killer_assets::collision::world::MusicZone>,
    music_times: crate::zone_music::MusicTimes,
    fishing_host: crate::fishing_host::FishingHost,
    fishing_target: Option<killer_render::viewport::ViewportTarget>,
    appendix: crate::appendix_effects::AppendixEffects,
    windows_age: f32,
    windows_saved: Option<String>,
    console_mark: crate::ui_olympiad::ConsoleMark,
    pub online: bool,
    pub disconnect_accepted: bool,
    pub cursor: Option<(f32,f32)>,
    pub hovered: Option<(i32,u64)>,
    pub loading_profile: [f64;4],
    pub profile: [f64;4],
    pub profile_frames: u32,
}

impl Runtime {
    #[allow(clippy::too_many_arguments)]
    pub fn new(mut client: Client, level: ue2_level::Level, scene: killer_assets::scene::Scene, renderer: &Renderer, format: wgpu::TextureFormat, size: (u32,u32), scale: f32, spawn: Option<(Vec3,i32)>, trace_file: Option<web_sys::Blob>, table_cache: &mut crate::table_cache::SessionTables) -> Result<Self, String> {
        let started = Instant::now();
        let tables = table_cache.load(&mut client).map_err(|e| e.to_string())?;
        let user = killer_ui::ini::parse_ini(&client.read_system_text("user.ini").map_err(|e| e.to_string())?);
        let option = killer_ui::ini::parse_ini(&client.read_system_text("option.ini").map_err(|e| e.to_string())?);
        let system = killer_ui::ini::parse_ini(&client.read_system_text("l2.ini").map_err(|e| e.to_string())?);
        let alaudio=killer_ui::ini::parse_ini(&client.read_system_text("alaudio.int").map_err(|e|e.to_string())?);
        let sound_config=crate::world_sound::SoundConfig::read(&alaudio);
        let mut audio=crate::audio::Audio::new(crate::audio::InitConfig::read(&crate::audio::config::AudioIni {option:&option,alaudio:&alaudio}),system.int("ALAudio.ALAudioSubsystem","Channels").unwrap_or(0));
        if let Err(error)=crate::audio::spatial_state::Config::read(&system).and_then(|config|audio.configure_spatial(config)) {log::warn!("áudio espacial: {error}");}
        let display = DisplaySettings::from_ini(&system, &option);
        let (anchor,heading) = spawn.unwrap_or((Vec3::new(83400.0,148000.0,-3400.0),16384));
        let orbit = crate::camera::OrbitCamera::behind(anchor,heading,crate::camera::CameraSettings::from_ini(&user));
        let binds = killer_world::engine_input::KeyBinds::from_ini(|name| user.get(killer_world::engine_input::INPUT_SECTION,name));
        let mut world = if spawn.is_some() {World::default()} else {characters::demo_world(anchor,heading,0)};
        world.rotation_rates=tables.rotation_rates.clone();
        world.enter_events=tables.entereventgrp.clone();
        world.walk_anim_frames=tables.walk_anim_frames();
        if spawn.is_none() {crate::ui_keys::demo_fill(&mut world);}
        let setup_ms = started.elapsed().as_secs_f64()*1000.0;
        let ground = crate::ground::GroundGrid::build(&scene);
        let lights = LightMap::new(&level,&scene.lights);
        let zones = scene.zones.len();
        let ground_ms = started.elapsed().as_secs_f64()*1000.0;
        let trace = if let Some(file) = trace_file {
            drop(scene);
            let size = file.size() as u64;
            crate::trace::TraceMesh::read_cache(std::io::BufReader::with_capacity(1 << 20, killer_platform::fs::File::from_blob(file)), size).map_err(|e| e.to_string())?
        } else { crate::trace::TraceMesh::from_owned_scene(scene) };
        let trace_ms = started.elapsed().as_secs_f64()*1000.0;
        if let Some(me) = world.me.as_mut() { ground.place(me); }
        let mut ui = GameUi::load(&mut client, ((size.0 as f32 / scale) as i32,(size.1 as f32 / scale) as i32)).map_err(|e| e.to_string())?;
        ui.set_tab_names(&tables);
        let mut modern = ModernHud::default();
        modern.configure(if killer_platform::hud_preferences().contains("modern=") {None}else{Some("classic")});
        modern.sync(&mut ui);
        let options=killer_assets::options::ClientOptions::load(&mut client).map_err(|e|e.to_string())?;
        let level=Arc::new(level);
        let neighbours=crate::tiles::NeighbourSet::new(killer_assets::tiles::TileId::of(anchor.x,anchor.y),level.clone());
        let mut shared=Shared::default();
        let music_times=client.read_system_text("alaudio.int").map(|text|{let ini=killer_ui::ini::parse_ini(&text);crate::zone_music::MusicTimes::read(|section,key|ini.get(section,key).map(str::to_owned))}).unwrap_or_default();
        if let Err(error)=shared.collision.tile(&mut client,&killer_assets::tiles::tile_name(anchor.x,anchor.y),&level) {log::warn!("colisão do tile: {error}");}
        Ok(Self { client,audio,audio_frame:None,audio_time:0.0,world,ui,orbit,ground,trace,tables,cache:{let mut cache=CharacterCache::default();cache.set_sound_config(sound_config);cache},items:Default::default(),fences:Default::default(),sound_bank:Default::default(),shakes:Default::default(),fade:Default::default(),fade_shown:false,effects:Default::default(),abnormal:Default::default(),shared,walk_warned:Default::default(),walk_state:Default::default(),marks:Default::default(),level,zones,neighbours,moving_trace:Default::default(),options,video:None,labels:Vec::new(),lights,display,
            bank:TextureBank::new(),radar_toggle:false,binds,modern,context:egui::Context::default(),painter:egui_wgpu::Renderer::new(renderer.device(),format,Default::default()),chat:Vec::new(),crest_inbox:Default::default(),scale,tile:killer_assets::tiles::tile_name(anchor.x,anchor.y),online:spawn.is_some(),disconnect_accepted:false,cursor:None,hovered:None,loading_profile:[setup_ms,ground_ms-setup_ms,trace_ms-ground_ms,started.elapsed().as_secs_f64()*1000.0-trace_ms],profile:[0.0;4],profile_frames:0,banks_age:0.0,zone_music:Default::default(),music_zone:None,music_times,fishing_host:Default::default(),fishing_target:None,appendix:Default::default(),windows_age:0.0,windows_saved:None,console_mark:Default::default() })
    }

    pub fn ground_diagnostic(&self) -> Option<serde_json::Value> {
        let me=self.world.me.as_ref()?;
        let grid=self.neighbours.ground_at(me.position,&self.ground);
        let feet=me.position.z-me.collision_height;
        let below=feet-crate::ground::ENGINE_PLACE_BELOW-me.collision_height;
        let above=feet+crate::ground::ENGINE_PLACE_ABOVE;
        let floor=grid.placement_floor(me.position.x,me.position.y,feet,me.collision_height).into_iter()
            .chain(self.moving_trace.ground_at(me.position.x,me.position.y,above,below)).max_by(f32::total_cmp);
        Some(serde_json::json!({
            "tile":killer_assets::tiles::tile_name(me.position.x,me.position.y),
            "position":me.position.to_array(),"feet":feet,"floor":floor,
            "triangles":grid.triangle_count(),
            "loaded":self.neighbours.is_loaded(killer_assets::tiles::TileId::of(me.position.x,me.position.y)),
        }))
    }

    pub fn apply(&mut self,event: &killer_world::session::SessionEvent,renderer: &mut Renderer) -> Result<(),String> {
        let entering=self.world.me.is_none() && matches!(event,killer_world::session::SessionEvent::UserInfo(_));
        self.world.apply(event);
        if let killer_world::session::SessionEvent::World(killer_world::WorldEvent::ShopPreviewInfo(info))=event {
            self.world.apply_shop_preview_info(info,|id|u32::try_from(id).ok().and_then(|id|self.tables.armorgrp.get(&id)).map(|armor|armor.body_part));
        }
        if matches!(event,killer_world::session::SessionEvent::UserInfo(_))
            && let Some(me)=&mut self.world.me {
            self.neighbours.ground_at(me.position,&self.ground).place(me);
        }
        if let killer_world::session::SessionEvent::World(event)=event {
            if let Some(object_id)=event.placed_object()
                && let Some(entity)=self.world.entity_mut(object_id) {
                self.neighbours.ground_at(entity.position,&self.ground).place(entity);
            }
            let resolved=ui2d::channel_say_text(Some(&self.tables),event);
            self.ui.html_tables=Some(self.tables.clone());
            self.ui.world_event(resolved.as_ref().unwrap_or(event),&self.world);
            self.ui.keys.special_maps.world_event(event,Some(&self.tables),&mut self.ui.desktop);
            match event {
                killer_world::WorldEvent::PledgeCrest(crest)=>{
                    if let Err(error)=self.bank.insert_dds(&ui2d::crest_texture(crest.crest_id),&crest.data) {log::warn!("brasão: {error}");}
                }
                killer_world::WorldEvent::AllianceCrest(crest)=>{
                    if let Err(error)=self.bank.insert_dds(&ui2d::ally_crest_texture(crest.crest_id),&crest.data) {log::warn!("brasão: {error}");}
                }
                killer_world::WorldEvent::DoorInfo(door)=>renderer.door_info(door.object_id,door.door_id),
                killer_world::WorldEvent::DoorStatusUpdate(door)=>renderer.door_status(door.object_id,door.closed,door.damage,door.show_hp),
                killer_world::WorldEvent::EventTrigger(trigger)=>{let (id,on)=killer_world::world::event_trigger(trigger);renderer.l2_event_trigger(id,on);}
                killer_world::WorldEvent::SystemMessage(message) if ui2d::is_notice(&self.tables,message)=>self.world.dialogs.push(killer_world::Dialog::Notice(message.clone())),
                _=>{},
            }
            self.ui.shop_event(event,Some(&self.tables));
            self.ui.private_shop_event(event,Some(&self.tables));
            self.ui.multisell_event(event,&self.world,Some(&self.tables));
            self.ui.manor_event(event,Some(&self.tables));
            self.ui.hero_event(event,&self.world);
            self.ui.empower_event(event);
            self.ui.observer_event(event);
            self.ui.calc_event(event);
            self.ui.gm_event_world(event);
            self.ui.gm_inventory_event(event,Some(&self.tables));
            self.ui.gm_status_event(event,&self.world);
            self.ui.gm_skills_event(event,Some(&self.tables));
            self.ui.gm_quest_event(event,Some(&self.tables));
            self.ui.gm_clan_event(event);
            self.ui.broadcast_event(event,Some(&self.tables),self.world.target);
            self.ui.cast_event(event,&self.world,Some(&self.tables));
            self.ui.olympiad_event(event,&self.world,Some(&self.tables));
            self.ui.quest_info_event(event,Some(&self.tables));
            self.ui.petition_event(event);
            self.ui.snoop_event(event);
            self.ui.train_event(event,&self.world,Some(&self.tables));
            self.ui.henna_event(event,Some(&self.tables));
            self.ui.macros_event(event,&self.world);
            self.ui.trade_event(event,&self.world,Some(&self.tables));
            if let killer_world::WorldEvent::NpcHtmlMessage(html)=event {self.ui.open_npc_html_with(html,Some(self.tables.as_ref()));}
            let mut sound=None;
            if let killer_world::WorldEvent::SystemMessage(message)=event && !ui2d::is_notice(&self.tables,message) {sound=ui2d::system_sound(&self.tables,message);}
            let notice_line=killer_world::notice_of(event).and_then(|notice|match notice.color {
                None=>{
                    let message=l2_net::game::SystemMessage {id:notice.id,args:Vec::new()};
                    if ui2d::is_notice(&self.tables,&message) {self.world.dialogs.push(killer_world::Dialog::Notice(message.clone()));}
                    else {sound=ui2d::system_sound(&self.tables,&message);}
                    ui2d::system_line(&self.tables,&message)
                }
                Some(color)=>{
                    let mut line=killer_world::ChatLine::new(killer_world::chat::CHAT_SYSTEM,String::new(),ui2d::system_text(Some(&self.tables),notice.id as u32,&[]));
                    line.color=Some(color);
                    Some(line)
                }
            });
            let line=match event {
                killer_world::WorldEvent::SystemMessage(message)=>ui2d::system_line(&self.tables,message),
                killer_world::WorldEvent::Say2(message)=>ui2d::say_line(&self.tables,message),
                killer_world::WorldEvent::NpcSay(say)=>ui2d::npc_say_line(&self.tables,say),
                _=>notice_line.or_else(||killer_world::ChatLine::from_event(event)),
            };
            if let Some(line)=line {
                let mut to_channel=false;
                if let killer_world::WorldEvent::Say2(message)=event {
                    self.world.receive_speech(message.object_id,message.kind,&line.text);
                    to_channel=matches!(message.kind,killer_ui::command_channel::SAY_ALL_MEMBERS|killer_ui::command_channel::SAY_LEADER|killer_ui::party_room::SAY_PARTY_ROOM);
                }
                if let killer_world::WorldEvent::NpcSay(say)=event {self.world.receive_speech(say.object_id,line.kind,&line.text);}
                if matches!(event,killer_world::WorldEvent::Say2(_)|killer_world::WorldEvent::NpcSay(_)) && line.kind==2 {self.ui.chat.tells.add(&line.sender);}
                if !to_channel && let Some(line)=self.ui.route_system_line(line) {self.push_chat(ui2d::chat_entry(&line));}
            }
            if let Some(name)=sound {self.interface_sound(&name);}
            if self.chat.len()>killer_world::chat::CHAT_HISTORY {self.chat.remove(0);}
        }
        if let Some((position,heading))=self.world.me.as_ref().map(|me|(me.position,me.heading)) {
            let tile=killer_assets::tiles::tile_name(position.x,position.y);
            if !killer_assets::tiles::TileId::parse(&tile).is_some_and(|id|self.neighbours.is_loaded(id)) {
                let loaded=self.client.load_level(&tile).map_err(|e|e.to_string())?;
                let options=killer_assets::options::ClientOptions::load(&mut self.client).map_err(|e|e.to_string())?;
                let mut scene=killer_assets::scene::build_scene_with(&mut self.client,&loaded.level,&options,Default::default());
                let mut bank=TextureBank::new();
                self.effects.clear(renderer);
                renderer.upload(&scene,&mut BankSource {bank:&mut bank,catalog:self.client.catalog()});
                if let Some(sky)=options.shared_sky.then(||killer_assets::scene::sky::load_shared(&mut self.client,&options)).flatten() {
                    renderer.add_tile("sky",&sky,&mut BankSource {bank:&mut bank,catalog:self.client.catalog()});
                }
                drop(bank);
                scene.lightmaps=Vec::new();
                self.client.catalog().release_files();
                self.lights=LightMap::new(&loaded.level,&scene.lights);
                self.level=Arc::new(loaded.level);
                if let Err(error)=self.shared.collision.tile(&mut self.client,&tile,&self.level) {log::warn!("colisão do tile {tile}: {error}");}
                self.zones=scene.zones.len();
                self.neighbours=crate::tiles::NeighbourSet::new(killer_assets::tiles::TileId::of(position.x,position.y),self.level.clone());
                self.options=options;
                self.moving_trace=Default::default();
                self.cache=CharacterCache::default();
                self.appendix.clear();
                self.trace=crate::trace::TraceMesh::from_scene(&killer_assets::scene::Scene::default());
                self.ground=crate::ground::GroundGrid::build(&scene);
                self.trace=crate::trace::TraceMesh::from_owned_scene(scene);
                self.items.clear();
                self.fences.clear();
                self.clear_marks(renderer);
                self.tile=tile;
                for entity in self.world.me.iter_mut().chain(self.world.entities.values_mut()) {
                    self.neighbours.ground_at(entity.position,&self.ground).place(entity);
                }
                let placed=self.world.me.as_ref().map_or(position,|me|me.position);
                self.ui.update_zone(&self.tables,(placed.x,placed.y,placed.z),Instant::now());
                for (&object_id,door) in &self.world.doors {
                    let Some(door_id)=door.info_door_id else {continue;};
                    renderer.door_info(object_id,door_id);
                    if door.status_received {renderer.door_status(object_id,door.closed,door.damage,door.show_hp);}
                }
            }
            let position=self.world.me.as_ref().map_or(position,|me|me.position);
            if entering {self.ui.update_zone(&self.tables,(position.x,position.y,position.z),Instant::now());}
            if entering {self.orbit=crate::camera::OrbitCamera::behind(position,heading,self.orbit.settings);}
            else {self.orbit.follow(position);}
        }
        Ok(())
    }

    fn interface_sound(&mut self,name: &str) {
        if self.audio.available() && self.audio_frame.is_some() && !name.is_empty() && name!="NAME_None"
            && let Some((format,encoded))=self.client.sound(name) {
            if format.eq_ignore_ascii_case("WAV") {self.audio.interface(encoded,self.audio_frame);}
            else {log::warn!("USound {name}: formato {format} indisponível");}
        }
    }

    pub fn resize(&mut self, size: (u32,u32), scale: f32) {
        if let Some(video)=&mut self.video {video.resize(size);}
        self.scale = scale;
        self.ui.resized(((size.0 as f32 / scale) as i32,(size.1 as f32 / scale) as i32));
    }

    pub fn begin_game(&mut self) -> Result<(),String> {
        if let Some(video)=&mut self.video {video.reset();}
        self.audio.stop();
        self.audio_frame=None;
        let mut ui=GameUi::load(&mut self.client,self.ui.desktop.size).map_err(|e|e.to_string())?;
        ui.set_tab_names(&self.tables);
        self.modern.sync(&mut ui);
        self.ui=ui;
        self.world=Default::default();
        self.chat.clear();
        self.ui.chat.clear_lists();
        self.ui.system_msg.list.clear();
        self.labels.clear();
        self.online=true;
        Ok(())
    }

    pub fn key_bind(&mut self, vk: u32, pressed: bool) {
        let (commands,action) = if pressed {(self.binds.press(vk),killer_world::engine_input::InputAction::Press)} else {(self.binds.release(vk),killer_world::engine_input::InputAction::Release)};
        self.exec_engine_commands(commands,action,0.0);
    }

    fn exec_engine_commands(&mut self, commands: Vec<String>, action: killer_world::engine_input::InputAction, delta: f32) {
        let has_pawn = self.world.me.is_some();
        for command in commands {
            killer_world::engine_input::exec_radar(&mut self.world.windows.radar,&command,has_pawn);
            self.orbit.exec_input(&command,action,delta);
        }
    }

    pub fn party_press(&mut self) {
        if let Some(action)=self.ui.take_status_click(&self.world) {self.action(action);}
    }

    pub fn party_assist(&mut self, x: i32, y: i32) {
        let views=ui2d::world_views(&self.world,Some(&self.tables),Instant::now());
        if let Some(action)=self.ui.party_assist_at(x,y,&views) {self.action(action);}
    }

    pub fn connection_lost_text(&self) -> Option<String> {self.tables.systemmsg.get(&0).map(|message|message.text.clone())}

    fn say(&mut self, typed: String) {
        use killer_world::NetCommand;
        let effects=self.ui.chat_line(&typed,&self.world);
        for effect in effects {
            match effect {
                killer_ui::chat_line::LineEffect::Chat {kind,text,target}=>self.ui.keys.send(NetCommand::Say {text,kind,target}),
                killer_ui::chat_line::LineEffect::Command(line)=>self.ui.keys.chat_command(&self.world,&line),
                killer_ui::chat_line::LineEffect::StoreSearch(word)=>self.world.search_stores(&word),
                killer_ui::chat_line::LineEffect::Builder(command)=>self.ui.keys.send(NetCommand::BypassBuildCmd {command}),
                killer_ui::chat_line::LineEffect::Console(command)=>{
                    if !self.ui.chat.capture.exec(&command) {log::info!("console: {command}");}
                }
                killer_ui::chat_line::LineEffect::Text(text)=>{
                    let mut line=killer_world::ChatLine::new(killer_world::chat::CHAT_SYSTEM,String::new(),text);
                    line.color=Some(killer_ui::chat_line::CONSOLE_TEXT_COLOR);
                    self.push_chat(ui2d::chat_entry(&line));
                }
                killer_ui::chat_line::LineEffect::System(id)=>self.ui.keys.chat_system(id),
            }
        }
    }

    fn push_chat(&mut self, entry: ChatEntry) {
        let shown=self.ui.chat.shows_in(self.ui.chat.active_tab,&entry);
        self.ui.chat.capture.message(&entry.line(),shown);
        self.ui.chat.ingest(&entry);
        self.ui.system_msg.ingest(&entry);
        self.chat.push(entry);
    }

    pub fn action(&mut self, action: ui2d::KeyAction) {
        use killer_world::NetCommand;
        let command=match action {
            ui2d::KeyAction::Say(text)=>{
                if !self.online {self.push_chat(ChatEntry::new(0,"Prévia local",text));return;}
                self.say(text);
                None
            }
            ui2d::KeyAction::Bypass(command)=>Some(NetCommand::Bypass {command}),
            ui2d::KeyAction::LinkHtml(link)=>Some(NetCommand::LinkHtml {link}),
            ui2d::KeyAction::Dialog(ui2d::DialogAnswer::Disconnected,accept)=>{self.disconnect_accepted|=accept;None}
            ui2d::KeyAction::Dialog(answer,accept)=>ui2d::dialog_command(answer,accept),
            ui2d::KeyAction::RestartPoint(point)=>Some(NetCommand::RestartPoint {point}),
            ui2d::KeyAction::Select(object_id)=>crate::input::target_action(&mut self.world,object_id,self.ui.keys.ctrl,self.ui.keys.shift),
            ui2d::KeyAction::Assist(member)=>{
                let target=self.world.entity(member).and_then(|entity|entity.target)
                    .filter(|target|self.world.entity(*target).is_some() || self.world.static_target(*target).is_some());
                target.zip(self.world.me.as_ref()).map(|(object_id,me)|NetCommand::Action {object_id,origin:me.position.to_array().map(|v|v as i32),shift:self.ui.keys.shift})
            }
            ui2d::KeyAction::Ignored | ui2d::KeyAction::Consumed=>None,
        };
        if let Some(command)=command {self.ui.keys.send(command);}
    }

    pub fn click(&mut self, camera: &Camera, pixel: (f32,f32), renderer: &Renderer) -> Result<(),String> {
        if self.world.me.is_none() {return Ok(());}
        if let Some(id)=characters::label_at(&self.labels,&self.ui.large,pixel) {
            self.action(ui2d::KeyAction::Select(id));
            return Ok(());
        }
        let Some(hit)=self.pick(camera,pixel,renderer)? else {return Ok(());};
        if self.options.l2_mark==2 {
            let life=self.mark_life(crate::landmark::MARK_CLASS);
            self.marks.state.add(hit.hit.position,hit.hit.normal,hit.surface,life,crate::landmark::sine());
        }
        if let Some(target)=hit.target {
            self.action(ui2d::KeyAction::Select(target.object_id));
            self.marks.state.delete(false);
        } else if let Some(me)=self.world.me.as_mut() {
            let target=hit.hit.position.to_array().map(|v|v as i32);
            let command=killer_world::NetCommand::MoveTo {target,origin:me.position.to_array().map(|v|v as i32)};
            me.dest=Some(Vec3::new(target[0] as f32,target[1] as f32,target[2] as f32));
            self.ui.keys.send(command);
            if let Some(command)=self.ui.tutorial.fire(killer_ui::tutorial::EVENT_MOVE) {self.ui.keys.send(command);}
        }
        Ok(())
    }

    fn pick(&mut self,camera: &Camera,pixel: (f32,f32),renderer: &Renderer) -> Result<Option<killer_assets::collision::world::Selection>,String> {
        let size=(self.ui.desktop.size.0 as u32,self.ui.desktop.size.1 as u32);
        let Some((origin,direction))=crate::input::pixel_ray(camera,size,pixel) else {return Ok(None);};
        let view=crate::items::GeometryView {neighbours:&self.neighbours,lighting:&self.lights,bsp_tree:&self.level.bsp_tree,zones:self.zones};
        let views=crate::items::views(&view,Some(renderer),origin);
        let targets=self.items.targets(&self.world);
        let query=killer_assets::pickup::TraceQuery {start:origin,end:origin+direction*10000.0,extent:Vec3::splat(0.1),flags:0x200bf};
        let spawned=self.fences.colliders(&self.world);
        killer_assets::collision::world::trace_targets(&mut self.client,&mut self.shared.collision,&views,query,&targets,&spawned)
    }

    pub fn release(&mut self) {
        let clicked=self.ui.mouse_up_in(&self.world).and_then(|action|ui2d::html_action(&action));
        if let Some(text)=self.ui.chat_filter_released()
            && let Err(error)=self.client.write_system_text("chatfilter.ini",&text) {
            self.push_chat(ChatEntry::new(-1,"",format!("Não foi possível salvar o filtro: {error}")));
        }
        let menu=self.ui.menu_released(&self.world);
        let warning=self.ui.warning_released();
        let dialog=self.ui.dialog_released();
        let views=ui2d::world_views(&self.world,Some(&self.tables),Instant::now());
        let pet=self.ui.pet_released(&views);
        let restart=self.ui.restart_released(&views);
        let party=self.ui.party_released(&views);
        let board=self.ui.board_released();
        if let Some(action)=warning.or(clicked).or(menu).or(dialog).or(pet).or(restart).or(party).or(board) {self.action(action);}
    }

    fn item_drop(&mut self,item: (i32,i32,i32,String,crate::ui_keys::ItemDropTarget),renderer: &Renderer,camera: &Camera) -> Result<(),String> {
        use crate::ui_keys::ItemDropTarget;
        use killer_world::{NetCommand,ItemDropKind,Dialog};
        let (object_id,item_id,count,name,target)=item;
        let etc=u32::try_from(item_id).ok().and_then(|id|self.tables.etcitemgrp.get(&id));
        let stackable=etc.is_some_and(|etc|etc.stackable!=0) && count>=2;
        let (kind,message_id,count,position)=match target {
            ItemDropTarget::World=>{
                let Some(pixel)=self.cursor else {return Ok(());};
                if self.world.target.is_some_and(|id|self.world.entity(id).is_some_and(|entity|matches!(entity.kind,killer_world::EntityKind::Player { .. }))) {return Ok(());}
                let (message_id,count)=if etc.is_some_and(|etc|etc.family==ui2d::DROP_PLACE_FAMILY) {(ui2d::DROP_PLACE_SYSMSG,None)}
                    else if stackable {(ui2d::DROP_COUNT_SYSMSG,Some(count))} else {(ui2d::DROP_CONFIRM_SYSMSG,None)};
                let Some(hit)=self.pick(camera,pixel,renderer)? else {return Ok(());};
                if self.world.inventory.find(object_id).is_none() {return Ok(());}
                (ItemDropKind::Drop,message_id,count,hit.hit.position.to_array().map(|v|v as i32))
            }
            ItemDropTarget::Trash if stackable=>(ItemDropKind::Destroy,ui2d::DESTROY_COUNT_SYSMSG,Some(count),[0;3]),
            ItemDropTarget::Trash=>(ItemDropKind::Destroy,ui2d::DESTROY_CONFIRM_SYSMSG,None,[0;3]),
            ItemDropTarget::Crystallize=>(ItemDropKind::Crystallize,ui2d::CRYSTALLIZE_SYSMSG,None,[0;3]),
            ItemDropTarget::Pet | ItemDropTarget::PetEquipped | ItemDropTarget::FromPet=>{
                let give=target!=ItemDropTarget::FromPet;
                if !stackable {
                    self.ui.keys.send(if give {NetCommand::GiveItemToPet {object_id,count:1}} else {NetCommand::GetItemFromPet {object_id,count:1}});
                    return Ok(());
                }
                (if give {ItemDropKind::GiveToPet}else{ItemDropKind::GetFromPet},if target==ItemDropTarget::PetEquipped {ui2d::PET_EQUIPPED_COUNT_SYSMSG}else{ui2d::PET_MOVE_COUNT_SYSMSG},Some(count),[0;3])
            }
        };
        self.world.dialogs.push(Dialog::DropItem {object_id,name,message_id,count,position,kind});
        Ok(())
    }

    pub fn commands(&mut self,renderer: &mut Renderer,adapter: &wgpu::Adapter,camera: &Camera) -> Vec<killer_world::NetCommand> {
        use crate::ui_keys::UiCommand;
        let mut outgoing=Vec::new();
        let arrived: Vec<_>=self.crest_inbox.lock().map(|mut inbox|std::mem::take(&mut *inbox)).unwrap_or_default();
        for (kind,bytes) in arrived {
            self.ui.keys.crest_file(kind,bytes.as_deref());
        }
        for command in self.ui.take_commands() {
            match command {
                UiCommand::Net(command)=>outgoing.push(command),
                UiCommand::ServerTransfer {..}=>outgoing.push(killer_world::NetCommand::Logout),
                UiCommand::ReplayRecord(_)=>{},
                UiCommand::DebugCommand {window}=>self.ui.keys.debug_command(window,&mut self.ui.desktop),
                UiCommand::DeleteLandMark=>self.marks.state.delete(false),
                UiCommand::ConfirmDeleteMacro {id,name}=>self.world.dialogs.push(killer_world::Dialog::DeleteMacro {id,name}),
                UiCommand::ConfirmDeleteRecipe {recipe_id,name}=>self.world.dialogs.push(killer_world::Dialog::DeleteRecipe {recipe_id,name}),
                UiCommand::ItemDrop {object_id,item_id,count,name,target}=>{
                    if let Err(error)=self.item_drop((object_id,item_id,count,name,target),renderer,camera) {self.push_chat(ChatEntry::new(-1,"",error));}
                }
                UiCommand::OpenPetNameDialog=>self.world.dialogs.push(killer_world::Dialog::PetName),
                UiCommand::SystemMessage(id)=>self.push_chat(ChatEntry::new(-1,"",ui2d::system_text(Some(&self.tables),id,&[]))),
                UiCommand::ColoredSystemMessage {id,color}=>{
                    let mut line=killer_world::ChatLine::new(killer_world::chat::CHAT_SYSTEM,String::new(),ui2d::system_text(Some(&self.tables),id,&[]));
                    line.color=Some(color);
                    self.push_chat(ui2d::chat_entry(&line));
                }
                UiCommand::ApplyOptions {close}=>if let Err(error)=self.apply_options(renderer,adapter,Some(close)) {self.push_chat(ChatEntry::new(-1,"",format!("Settings: {error}")));},
                UiCommand::OptionResolutionChanged(selection)=>self.option_resolution_changed(selection),
                UiCommand::OptionKeepMinFrameRateChanged=>self.option_min_frame_changed(),
                UiCommand::ToggleMinFrameRate=>if let Err(error)=self.toggle_min_frame(renderer,adapter) {self.push_chat(ChatEntry::new(-1,"",format!("Settings: {error}")));},
                UiCommand::InitializeInterface=>if let Err(error)=crate::ui_initialize_run::initialize(&mut self.ui) {self.push_chat(ChatEntry::new(-1,"",format!("Initialize: {error}")));},
                UiCommand::ToggleHelp {macro_help}=>{
                    let path=if macro_help {crate::ui_help::MACRO_FILE}else{crate::ui_help::HELP_FILE};
                    if let Ok(text)=self.client.read_help_text(path) {{let tables=self.tables.clone();self.ui.help.toggle(&mut self.ui.desktop,&self.ui.small,path,&text,&|marker,id|ui2d::html_marker(Some(&tables),marker,id));}}
                }
                UiCommand::LoadHelp(path)=>if let Ok(text)=self.client.read_help_text(&path) {{let tables=self.tables.clone();self.ui.help.load(&mut self.ui.desktop,&self.ui.small,&text,&|marker,id|ui2d::html_marker(Some(&tables),marker,id));}},
                UiCommand::Log(message)=>log::info!("{message}"),
                UiCommand::WarningMessage {id,numbers}=>{
                    let args: Vec<_>=numbers.into_iter().map(l2_net::game::SystemMessageArg::Number).collect();
                    self.ui.show_warning(&ui2d::system_text(Some(&self.tables),id,&args),false);
                }
                UiCommand::OpenPetition(text)=>{
                    let preset=text.as_deref().map(|text|(text,&self.ui.small));
                    self.ui.keys.petition.open_begin(preset,&mut self.ui.desktop);
                }
                UiCommand::LoadPetitionPage(path)=>if let Ok(text)=self.client.read_help_text(&path) {{let tables=self.tables.clone();self.ui.keys.petition.begin.set_page_with(&text,&self.ui.small,&|marker,id|ui2d::html_marker(Some(&tables),marker,id));}},
                UiCommand::GmEventFile {file,commands}=>if let Ok(text)=self.client.read_system_text(&file) {
                    let ini=killer_ui::ini::parse_ini(&text);
                    if commands {
                        let html=killer_ui::gm_event::command_html(&ini);
                        self.ui.show_html("",&html);
                    } else {
                        let actions=match killer_ui::gm_event::read_options(&ini) {
                            Ok(options)=>self.ui.keys.gm_event.start_race(options),
                            Err(error)=>vec![error],
                        };
                        self.ui.gm_event_actions(actions,&self.world);
                    }
                },
                UiCommand::CrestFile {kind,..}=>crate::crest_file::pick(kind,&self.crest_inbox),
                UiCommand::InterfaceSound(name)=>self.interface_sound(name),
            }
        }
        outgoing
    }

    pub fn tick(&mut self, renderer: &mut Renderer, dt: f32, environment: &killer_render::environment::Environment) -> Camera {
        let started=Instant::now();
        renderer.set_no_fractal_anim(self.options.no_fractal_anim);
        renderer.set_projectors(self.options.projectors);
        renderer.set_pawn_shadow(self.options.pawn_shadow,self.options.keep_min_frame_rate);
        self.cache.set_keep_min_frame_rate(self.options.keep_min_frame_rate);
        self.windows_age+=dt;
        if self.windows_age>=5.0 {
            self.windows_age=0.0;
            self.save_windows_info();
        }
        self.banks_age+=dt;
        if self.banks_age>=30.0 {
            self.banks_age=0.0;
            let dropped=self.bank.trim_over(crate::pawn_effects::BANK_TRIM_BYTES)+self.effects.trim_banks();
            if dropped>0 {log::info!("bancos de textura: {dropped} cópias de CPU soltas");}
        }
        self.advance_neighbours(renderer);
        renderer.set_movers_auto_tick(false);
        renderer.tick_movers(dt);
        self.moving_trace.sync(renderer);
        let before=self.walk_state.snapshot(&self.world);
        self.world.pawn_clipping=self.display.pawn_clipping;
        let held=self.binds.hold(dt);
        self.exec_engine_commands(held,killer_world::engine_input::InputAction::Hold,dt);
        self.world.tick(dt);
        if !before.is_empty() {
            let origin=self.world.me.as_ref().map_or(Vec3::ZERO,|me|me.position);
            let view=crate::items::GeometryView {neighbours:&self.neighbours,lighting:&self.lights,bsp_tree:&self.level.bsp_tree,zones:self.zones};
            let views=crate::items::views(&view,Some(renderer),origin);
            let spawned=self.fences.colliders(&self.world);
            self.music_zone=killer_assets::collision::world::music_volume_at(&mut self.client,&mut self.shared.collision,&views,origin).unwrap_or_else(|error|{if self.walk_warned.insert(error.clone()) {log::warn!("zona de música: {error}");}None});
            let (_,error)=crate::walk_world::walk_entities(&mut self.client,&mut self.shared.collision,&mut self.walk_state,&mut self.world,crate::walk_world::Frame {before:&before,views:&views,spawned:&spawned,dt,options:&self.options});
            let calls=std::mem::take(&mut self.walk_state.calls);
            let (commands,stopped)=crate::walk_world::controller_commands(&mut self.world,&calls);
            if stopped {self.marks.state.delete(false);}
            for command in commands {self.ui.keys.send(command);}
            self.cache.set_foot_volumes(&self.walk_state.foot);
            self.cache.set_swim_ticks(&self.walk_state.swim);
            if let Some(error)=error && self.walk_warned.insert(error.clone()) {log::warn!("física de andar: {error}");}
        }
        if let Some(action)=self.ui.tick_dialog(&mut self.world,Some(&self.tables),Instant::now()) {self.action(action);}
        self.ui.tick_tutorial(Instant::now());
        self.ui.tick_command_channel(Instant::now());
        self.ui.tick_zone_title(Instant::now());
        if let Some(me) = self.world.me.as_mut() {
            self.orbit.follow(me.position);
        }
        self.orbit.player_tick(dt);
        self.orbit.camera_commands(&mut self.world,dt);
        let origin=self.world.me.as_ref().map_or(Vec3::ZERO,|me|me.position);
        let camera_views={
            let view=crate::items::GeometryView {neighbours:&self.neighbours,lighting:&self.lights,bsp_tree:&self.level.bsp_tree,zones:self.zones};
            crate::items::views(&view,Some(renderer),origin)
        };
        let extent=if self.orbit.settings.hit_check && !self.orbit.settings.volume_camera {0.1} else {0.0};
        let camera_spawned=self.fences.colliders(&self.world);
        let engine=std::cell::RefCell::new((&mut self.client,&mut self.shared.collision,&mut self.walk_warned));
        self.orbit.collide(|from,to|{
            let mut guard=engine.borrow_mut();
            let engine=&mut *guard;
            match crate::walk_world::camera_hit(&mut *engine.0,&mut *engine.1,&camera_views,&camera_spawned,from,to,extent) {
                Ok(hit)=>hit,
                Err(error)=>{if engine.2.insert(error.clone()) {log::warn!("colisão da câmera: {error}");}None}
            }
        });
        let unshaken = self.orbit.camera();
        self.shakes.measure(dt);
        for quake in self.world.effects.take_earthquakes() {
            let (shake,sound)=killer_world::camera_shake::earthquake([quake.x,quake.y,quake.z],quake.intensity,quake.duration);
            self.shakes.add(&shake,unshaken.eye.to_array());
            if let Some(me)=self.world.me.as_ref() {
                self.cache.push_sound_plan(crate::world_sound::SoundPlan {actor:me.object_id,name:sound.to_owned(),position:unshaken.eye,volume:killer_world::camera_shake::EARTHQUAKE_SOUND_VOLUME*crate::world_sound::VOLUME_SCALE,radius:killer_world::camera_shake::EARTHQUAKE_SOUND_RADIUS,skip_if_playing:false});
            }
        }
        for item_id in self.world.effects.take_equip_sounds() {
            if let Some(name)=self.tables.equip_sound(item_id).map(str::to_owned) {self.interface_sound(&name);}
        }
        for sound in self.world.effects.take_sounds() {
            if (sound.kind==1 || sound.kind==2) && !sound.sound_file.is_empty() && self.world.me.is_some() {
                let delay=(sound.delay/1000) as f32;
                if sound.kind==1 {self.zone_music.request_server_music(&sound.sound_file,delay);} else {self.zone_music.request_server_voice(&sound.sound_file,delay,true);}
                continue;
            }
            if sound.kind==0 && !sound.sound_file.is_empty() && let Some(me)=self.world.me.as_ref() {
                self.cache.push_sound_plan(crate::world_sound::SoundPlan {actor:me.object_id,name:sound.sound_file,position:me.position,volume:1.0,radius:crate::world_sound::DEFAULT_RADIUS,skip_if_playing:false});
            }
        }
        for plan in self.cache.take_view_shakes() {
            self.shakes.add_notify(&plan.notify,plan.location.to_array(),unshaken.eye.to_array(),plan.randoms);
            if let Some(range)=plan.smoke {renderer.l2_event_smoke(plan.location.to_array(),range);}
        }
        for shake in self.cache.take_explosion_shakes() {self.shakes.add(&shake,unshaken.eye.to_array());}
        for (location,radius) in self.cache.take_event_smokes() {renderer.l2_event_smoke(location.to_array(),radius);}
        self.shakes.update(dt);
        for fade in self.cache.take_screen_fades() {self.fade.blink(fade.fade_out_duration,fade.fade_out_color,fade.fade_in_duration,fade.black_out_duration);}
        if std::mem::take(&mut self.world.observer.fade_out_pending) {
            self.fade.fade_out(killer_world::observer::FADE_MS,killer_world::observer::FADE_COLOR,killer_world::observer::TELEPORT_FADE_ID);
        }
        if std::mem::take(&mut self.world.observer.camera_default_pending) {self.orbit.observer_default_camera();}
        for op in std::mem::take(&mut self.ui.keys.olympiad.mark_ops) {self.console_mark.apply(op,&mut self.options.l2_mark);}
        for on in std::mem::take(&mut self.ui.keys.broadcast.controller) {self.world.observer.ctrl_broadcast=on;}
        for op in std::mem::take(&mut self.ui.keys.broadcast.canvas_ops).into_iter().chain(std::mem::take(&mut self.ui.keys.olympiad.canvas_ops)) {self.world.appendix.canvas_apply(op);}
        for op in std::mem::take(&mut self.ui.keys.broadcast.marks) {self.world.apply_appendix(op);}
        if let Some(done)=self.fade.update(dt) && done.id==killer_world::observer::TELEPORT_FADE_ID {
            if self.world.observer_fade_done() && let Some(me)=self.world.me.as_mut() {
                self.neighbours.ground_at(me.position,&self.ground).place(me);
            }
            self.fade.fade_in(killer_world::observer::FADE_MS,0);
        }
        if self.fade.active() || self.fade_shown {
            let (scale,fog)=self.fade.flash();
            renderer.set_flash(scale,fog);
            self.fade_shown=self.fade.active();
        }
        let camera = crate::camera::apply_shake(unshaken,self.shakes.rotation_offset());
        self.audio_time+=f64::from(dt);
        self.audio_frame=self.world.me.as_ref().map(|me|crate::audio::AudioFrame {time:self.audio_time,view_actor_location:me.position.to_array(),max_radius_multiplier:50.0,realtime:false,replay_recording:false});
        if let Err(error)=self.audio.update_spatial(self.audio_frame,camera.view().inverse(),&{let mut positions=self.items.audio_positions(&self.world);positions.extend(crate::world_sound::pawn_positions(&self.world));positions}) {log::warn!("áudio espacial: {error}");}
        self.audio.update(self.audio_frame);
        if let Some(frame)=self.audio_frame {
            if std::mem::take(&mut self.world.music_stop) {self.audio.stop_channel(killer_ui::option_audio::AudioChannel::Music);}
            if std::mem::take(&mut self.world.music_reset) {self.audio.controller.reset_music_controll();self.zone_music.reset();}
            let pawn=self.world.me.as_ref().map(|me|crate::zone_music::Pawn {location:[me.position.x,me.position.y,me.position.z],zone:self.music_zone.clone(),on_vehicle:me.vehicle.is_some()});
            let today=crate::zone_music::civil_date(killer_platform::time::SystemTime::now().duration_since(killer_platform::time::UNIX_EPOCH).map_or(0,|d|d.as_secs()));
            self.zone_music.set_vehicle_status(pawn.as_ref().is_some_and(|pawn|pawn.on_vehicle));
            let cache=&mut self.cache;
            let tables=&self.tables;
            let mut out=crate::audio::WorldMusic {audio:&mut self.audio,root:std::path::Path::new("/client"),frame};
            self.zone_music.tick(dt,self.music_times,pawn.as_ref(),Some(&mut out),|id|crate::zone_music::MusicNames {titles:&tables.musicinfo,today}.get(id,||cache.crt_rand()));
        }
        self.ui.audio_volumes=self.audio.available().then(||self.audio.config.volumes.ui_values());
        self.profile[0]+=started.elapsed().as_secs_f64()*1000.0;
        let started=Instant::now();
        let frame = Frame { display:self.display,viewer:self.world.me.as_ref().map_or(camera.eye,|me|me.position),dt };
        let drawn=characters::drawn_pawns(&self.world,&self.tables,&frame).into_iter().collect();
        for character in self.cache.evict_drawn(&self.world,&drawn) {renderer.remove_character(character);}
        let frustum=camera.frustum(self.ui.desktop.size.0 as f32/self.ui.desktop.size.1.max(1) as f32);
        let zone = |position: Vec3| self.neighbours.zone_at(position).unwrap_or_else(||{
            let zone=self.level.bsp_tree.zone_at(position.to_array());
            if usize::from(zone)<self.zones {zone} else {0}
        });
        let mut instances = characters::instances_drawn(&mut self.world,&mut self.cache,&self.tables,&frame,zone,|query|self.neighbours.tile_at(query.position).map_or(&self.lights,|tile|&tile.lighting).sample(query,0,0.0),|center,radius|frustum.contains_sphere(center,radius),|_|None,&drawn);
        self.appendix.sync(&mut self.cache,&mut self.world,&mut self.client);
        self.profile[1]+=started.elapsed().as_secs_f64()*1000.0;
        let started=Instant::now();
        self.items.sync(&self.world,renderer);
        let view=crate::items::GeometryView {neighbours:&self.neighbours,lighting:&self.lights,bsp_tree:&self.level.bsp_tree,zones:self.zones};
        let mut requests=Vec::new();
        self.items.request_with(&self.tables,&view,renderer,|request|{requests.push(request);true});
        for request in requests {
            match crate::loader::handle(&mut self.client,&mut self.shared,request) {
                LoadResult::Item {item_id,asset}=>self.items.install_asset(item_id,asset,renderer,&mut self.client),
                LoadResult::PlaceItem {ticket,pieces}=>self.items.install_placement(ticket,pieces),
                _=>{},
            }
        }
        self.fences.sync(&self.world,renderer);
        let mut fence_requests=Vec::new();
        self.fences.request_with(&self.world,&view,renderer,self.options.clipping,|request|{fence_requests.push(request);true});
        for request in fence_requests {
            if let LoadResult::Fence {batch}=crate::loader::handle(&mut self.client,&mut self.shared,request) {self.fences.install(batch,renderer,&mut self.client);}
        }
        let mut lines=crate::server_objects::lines(&self.world);
        if let Some(me)=self.world.me.as_ref().map(|me|me.object_id) {
            let eye=camera.eye;
            let zone=self.neighbours.zone_at(eye).unwrap_or_else(||{let zone=self.level.bsp_tree.zone_at(eye.to_array());if usize::from(zone)<self.zones {zone}else{0}});
            let fishing=self.fishing_host.update(&mut self.world.fishing_view,&mut self.client,renderer,&mut self.cache,crate::fishing_host::FishingContext {me,camera:eye,zone,dt});
            instances.extend(fishing.instance);
            lines.extend(fishing.lines);
        }
        renderer.set_lines(&lines);
        instances.extend(self.items.instances(dt,&view,renderer,Some(&mut self.client)));
        instances.extend(self.effects.arrow_instances(&view));
        for request in self.items.take_sounds() {
            if let Err(error)=self.audio.pickup(request.actor,&request.sound.encoded,request.position,request.volume,self.audio_frame) {log::warn!("som do item {}: {error}",request.sound.name);}
        }
        for plan in self.cache.take_sound_plans() {
            let Some(sound)=self.sound_bank.resolve(&mut self.client,&plan) else {continue;};
            if !(sound.skip_if_playing && self.audio.is_playing(sound.actor,sound.key)) && let Err(error)=self.audio.world(&sound,self.audio_frame) {log::warn!("som {}: {error}",sound.name);}
        }
        let clear=|a,b|self.neighbours.nearest(a,b,&self.trace).is_none() && self.moving_trace.nearest(a,b).is_none();
        renderer.update_sun_visibility(frame.viewer,environment.sun_dir,&instances,clear);
        renderer.update_coronas(&camera,self.ui.desktop.size.0 as f32/self.ui.desktop.size.1.max(1) as f32,dt,clear);
        if let Some(me)=&self.world.me {
            let end=me.position+environment.sky.sun_trace*killer_assets::options::range_units(self.options.clipping.static_mesh);
            renderer.set_lens_flare_clear(clear(me.position,end));
        }
        renderer.set_instances(&instances);
        self.ui.keys.social_durations.clone_from(&self.cache.social_durations);
        for key in self.cache.take_wanted(1) {
            let Some(look) = characters::look(key,&self.tables) else { self.cache.fail(key); continue };
            match load_character(&mut self.client,&mut self.shared,key,&look.appearance,look.draw_scale) {
                LoadResult::Character {parts,animation,draw_scale,weapon_type,mut bank,render_defaults,effects,..} => {
                    let parts = parts.into_iter().map(|mesh| {
                        let id = renderer.upload_character(&mesh,&mut BankSource {bank:&mut bank,catalog:self.client.catalog()});
                        (id,Arc::new(mesh))
                    }).collect();
                    if let Some(mut character) = LoadedCharacter::new(parts,animation,draw_scale,weapon_type) {
                        character.render_defaults = render_defaults;
                        self.cache.insert(key,character.with_effects(effects.effects,effects.bank));
                    } else { self.cache.fail(key); }
                }
                _ => self.cache.fail(key),
            }
        }
        self.effects.set_viewer(camera.eye,killer_assets::scene::transform::vector_rotation(camera.target-camera.eye),self.neighbours.zone_at(camera.eye).unwrap_or_else(||{let zone=self.level.bsp_tree.zone_at(camera.eye.to_array());if usize::from(zone)<self.zones {zone}else{0}}));
        self.abnormal.sync(&mut self.cache,&mut self.world,&drawn,&mut self.client,&self.tables,dt);
        self.effects.sync(renderer,&mut self.cache,&mut self.client,&self.world,&self.options);
        self.effects.fly(renderer,&mut self.cache,&self.world,dt);
        self.effects.firecrackers(renderer,&mut self.cache,&mut self.client,&mut self.world,&self.options,dt);
        let arrow_option=self.ui.keys.option.checked(crate::quest_arrow::ARROW_CHECK);
        let zone=self.world.me.as_ref().map_or(0,|me|self.neighbours.zone_at(me.position).unwrap_or_else(||{let zone=self.level.bsp_tree.zone_at(me.position.to_array());if usize::from(zone)<self.zones {zone}else{0}}));
        self.effects.quest_arrow(renderer,&mut self.cache,&mut self.client,&mut self.world,&self.options,arrow_option,zone);
        self.sync_marks(renderer,dt);
        self.profile[2]+=started.elapsed().as_secs_f64()*1000.0;
        let started=Instant::now();
        self.modern.sync(&mut self.ui);
        self.sync_video_page();
        self.ui.sync_windows(&self.world,Some(&self.tables),Instant::now());
        let (hovered,item_label)=self.hover_feedback(&camera,renderer);
        let mut views = ui2d::world_views(&self.world,Some(&self.tables),Instant::now());
        views.item_label=item_label;
        let screen=(self.ui.desktop.size.0 as f32,self.ui.desktop.size.1 as f32);
        views.primitives=crate::server_objects::marks(&self.world,|point|characters::project(&camera,screen,point));
        views.radar.camera_yaw=self.orbit.facing();
        views.radar.camera_pitch=-(self.orbit.pitch/std::f32::consts::TAU*crate::camera::UE_UNITS_PER_TURN) as i32;
        self.ui.show_pet_window(&views);
        let input = egui::RawInput { screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(self.ui.desktop.size.0 as f32*self.scale,self.ui.desktop.size.1 as f32*self.scale))),time:Some(self.audio_time),events:vec![self.cursor.map_or(egui::Event::PointerGone,|(x,y)|egui::Event::PointerMoved(egui::pos2(x*self.scale,y*self.scale)))],..Default::default() };
        self.context.begin_pass(input);
        self.labels = characters::labels_drawn(&self.world,&self.tables,&self.cache,&frame,&camera,(self.ui.desktop.size.0 as f32,self.ui.desktop.size.1 as f32),hovered,&drawn);
        let target = self.world.targeted().map(|entity|ui2d::target_view(&self.world,entity,characters::display_name(entity,&self.tables),Some(&self.tables))).or_else(||ui2d::static_target_view(&self.world,Some(&self.tables)));
        self.modern.capture_windows(&self.ui,&self.world,&self.chat,target.as_ref(),&views);
        self.modern.prepare_icons(&self.context,&self.ui,&mut self.client);
        self.modern.prepare_combat(&self.context,&self.world,Some(&self.tables),&mut self.client);
        crate::ui_modern::ModernHud::measure_labels(&self.context,&self.ui,&mut self.labels,self.scale);
        self.world.fishing_view.refresh(self.ui.desktop.size.0);
        if self.world.fishing_view.param.is_some() {
            if self.fishing_target.is_none() {
                let (width,height)=killer_world::fishing_view::VIEWPORT_SIZE;
                let target=renderer.viewport_target((width as u32,height as u32));
                renderer.set_ui_external_texture(killer_ui::fishing::VIEWPORT_TEXTURE,target.view(),[width as f32,height as f32]);
                self.fishing_target=Some(target);
            }
        } else if self.fishing_target.take().is_some() {
            renderer.clear_ui_external_texture(killer_ui::fishing::VIEWPORT_TEXTURE);
        }
        let list = self.ui.frame(&self.world,&self.labels,&self.chat,target.as_ref(),&views);
        renderer.set_ui_size(Some((self.ui.desktop.size.0 as u32,self.ui.desktop.size.1 as u32)));
        renderer.set_ui(&list,&mut BankSource {bank:&mut self.bank,catalog:self.client.catalog()});
        if !renderer.radar_meshes_loaded() {
            let meshes=killer_render::radar::load_meshes(&mut self.client);
            if meshes.len()<killer_render::radar::RadarMesh::ALL.len() {log::warn!("radar: {} de {} malhas LineageDecos carregadas",meshes.len(),killer_render::radar::RadarMesh::ALL.len());}
            let mut bank=TextureBank::new();
            renderer.set_radar_meshes(&meshes,&mut BankSource {bank:&mut bank,catalog:self.client.catalog()});
        }
        let steps=if self.ui.radar_visible(&self.world) {killer_render::radar::radar_steps(&views.radar,|mesh|renderer.radar_mesh_ready(mesh),&mut self.radar_toggle)} else {Vec::new()};
        renderer.set_radar(steps);
        renderer.set_view_target(self.world.me.as_ref().map(|me|me.position));
        renderer.set_viewer(killer_render::renderer::Viewer {zone:self.neighbours.zone_at(camera.eye).unwrap_or_else(||{let zone=self.level.bsp_tree.zone_at(camera.eye.to_array());if usize::from(zone)<self.zones {zone}else{0}}),position:camera.eye,dt});
        self.profile[3]+=started.elapsed().as_secs_f64()*1000.0;
        self.profile_frames+=1;
        camera
    }

    fn advance_neighbours(&mut self,renderer: &mut Renderer) {
        use crate::tiles::Job;
        let Some(position)=self.world.me.as_ref().map(|me|me.position) else {return;};
        for name in self.neighbours.follow(position) {renderer.remove_tile(&name);}
        if let Some(job)=self.neighbours.jobs.pop_front() {
            let root=std::path::Path::new("/client");
            match job {
                Job::Load(tile)=>{
                    let seam=self.neighbours.seam_levels(tile);
                    match crate::tiles::load_neighbour_uploaded(root,tile,&self.options,seam,renderer) {
                        Ok(loaded)=>{
                            if let Err(error)=self.shared.collision.tile(&mut self.client,&loaded.name,&loaded.level) {log::warn!("colisão do vizinho {}: {error}",loaded.name);}
                            for name in self.neighbours.attach(loaded,position) {renderer.remove_tile(&name);}
                        }
                        Err(error)=>{self.neighbours.fail(tile);log::error!("vizinho: {error}");}
                    }
                }
                Job::Reseam(name)=>{
                    if let Some(id)=killer_assets::tiles::TileId::parse(&name)
                        && let Some(level)=self.neighbours.level_of(id) {
                        match crate::tiles::reseam(root,&name,level,self.neighbours.seam_levels(id)) {
                            Ok(mut seamed)=>{
                                let tile=if name==self.tile {""} else {&name};
                                renderer.replace_draws(tile,killer_assets::scene::DrawKind::Terrain,&seamed.draws,&mut BankSource {bank:&mut seamed.bank,catalog:self.client.catalog()});
                                self.client.catalog().release_files();
                            }
                            Err(error)=>log::error!("costura: {error}"),
                        }
                    }
                }
            }
        }
        let drawn=self.neighbours.drawn(position,self.options.clipping.actor);
        for name in self.neighbours.tiles.keys().chain(std::iter::once(&self.tile)) {
            renderer.set_tile_visible(if name==&self.tile {""}else{name},drawn.contains(name));
        }
    }

    fn hover_feedback(&mut self,camera:&Camera,renderer:&Renderer)->(Option<i32>,Option<ui2d::ItemLabel>) {
        self.hovered=None;
        let Some(pixel)=self.cursor else {return (None,None);};
        let size=(self.ui.desktop.size.0 as f32,self.ui.desktop.size.1 as f32);
        if !(0.0..=size.0).contains(&pixel.0) || !(0.0..=size.1).contains(&pixel.1) {return (None,None);}
        let hit=match self.pick(camera,pixel,renderer) {
            Ok(hit)=>hit,
            Err(error)=>{log::warn!("ator sob o mouse: {error}");return (None,None);}
        };
        let Some(target)=hit.and_then(|hit|hit.target) else {return (None,None);};
        self.hovered=Some((target.object_id,target.spawned));
        let pieces=self.items.targets_with_pieces(&self.world);
        let slot=pieces.iter().find(|(piece,_)|piece.object_id==target.object_id && piece.spawned==target.spawned && piece.position==target.position && piece.radius==target.radius && piece.height==target.height).and_then(|(_,slot)|*slot);
        let Some(slot)=slot else {return (Some(target.object_id),None);};
        if self.ui.window_under(pixel.0 as i32,pixel.1 as i32).is_some() {return (None,None);}
        let label=self.items.label(&self.world,&self.tables,target.object_id,target.spawned,slot).and_then(|(location,text)|characters::project(camera,size,location).map(|screen|ui2d::ItemLabel {screen,text}));
        (None,label)
    }

    fn mark_life(&mut self,class:&'static str)->f32 {
        *self.marks.spans.entry(class).or_insert_with(||self.client.class_float_defaults(class,&["LifeSpan"]).ok().and_then(|values|values.get("LifeSpan").copied()).unwrap_or(0.0))
    }

    fn clear_marks(&mut self,renderer:&mut Renderer) {
        for (_,id) in self.marks.active.drain() {renderer.remove_effect(id);}
        for (id,_) in self.marks.loose.drain(..) {renderer.remove_effect(id);}
        self.marks.state=Default::default();
    }

    fn sync_marks(&mut self,renderer:&mut Renderer,dt:f32) {
        use crate::landmark::{Request,Pawn};
        use killer_render::renderer::EffectPlacement;
        let arrival_life=self.mark_life(crate::landmark::ARRIVAL_CLASS);
        let pawn=self.world.me.as_ref().map(|me|Pawn {location:me.position,waiting:self.cache.me_waiting});
        self.marks.state.update(dt,pawn.as_ref(),arrival_life);
        for request in self.marks.state.take_requests() {
            match request {
                Request::Spawn {slot,class,location,rotation}=>{
                    if let Some(old)=self.marks.active.remove(&slot) {renderer.remove_effect(old);}
                    let asset=self.effects.assets.entry(class.to_owned()).or_insert_with(||self.client.emitter_class(class).map(|emitter|killer_assets::scene::emitters::effect_asset(&mut self.client,&emitter,&self.options)));
                    let Some(asset)=asset.as_ref() else {continue;};
                    let id=renderer.add_effect(asset,&mut BankSource {bank:&mut self.effects.bank,catalog:self.client.catalog()});
                    let zone=self.neighbours.zone_at(location).unwrap_or_else(||{let zone=self.level.bsp_tree.zone_at(location.to_array());if usize::from(zone)<self.zones {zone}else{0}});
                    renderer.place_effect(id,EffectPlacement {location,rotation,zone,visible:true});
                    self.marks.active.insert(slot,id);
                }
                Request::Remove(slot)=>{if let Some(id)=self.marks.active.remove(&slot) {renderer.remove_effect(id);}}
                Request::Release {slot,life}=>{if let Some(id)=self.marks.active.remove(&slot) {self.marks.loose.push((id,life));}}
            }
        }
        self.marks.active.retain(|_,id|{let finished=renderer.effect_finished(*id);if finished {renderer.remove_effect(*id);}!finished});
        self.marks.loose.retain_mut(|(id,life)|{
            *life=(f64::from(*life)-f64::from(dt)) as f32;
            let finished=*life<=0.0001 || renderer.effect_finished(*id);
            if finished {renderer.remove_effect(*id);}
            !finished
        });
    }

    pub fn render_fishing(&mut self, renderer: &mut Renderer, camera: &Camera, environment: &killer_render::environment::Environment) {
        if let (Some(target),Some(param))=(self.fishing_target.as_mut(),self.world.fishing_view.param) {
            let forward=killer_assets::engine_math::rotator_forward(param.rotation);
            let fishing=Camera {eye:param.location,target:param.location+forward,fov_x_deg:param.fov,near:camera.near,far:camera.far};
            renderer.render_viewport(target,&fishing,environment);
        }
    }

    pub fn prepare_hud(&mut self, renderer: &mut Renderer, _size: (u32,u32), _time: f64) -> egui::FullOutput {
        self.modern.paint(&self.context,&self.ui,&self.world,&self.chat,self.scale);
        self.modern.paint_combat(&self.context,&self.ui,&self.world,&self.labels,self.scale);
        let output_ui = self.context.end_pass();
        for (id,deltas) in &output_ui.textures_delta.set {for delta in deltas {self.painter.update_texture(renderer.device(),renderer.queue(),*id,delta);}}
        self.modern.upload_names(&self.context,renderer,&self.painter);
        output_ui
    }

    pub fn paint(&mut self, renderer: &Renderer, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView, size: (u32,u32), output_ui: egui::FullOutput) -> Vec<wgpu::CommandBuffer> {
        let jobs = self.context.tessellate(output_ui.shapes,output_ui.pixels_per_point);
        let screen = egui_wgpu::ScreenDescriptor {size_in_pixels:[size.0,size.1],pixels_per_point:output_ui.pixels_per_point};
        let commands = self.painter.update_buffers(renderer.device(),renderer.queue(),encoder,&jobs,&screen);
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label:Some("web-hud"),
                color_attachments:&[Some(wgpu::RenderPassColorAttachment {view:output,resolve_target:None,ops:wgpu::Operations {load:wgpu::LoadOp::Load,store:wgpu::StoreOp::Store},depth_slice:None})],..Default::default()
            }).forget_lifetime();
            self.painter.render(&mut pass,&jobs,&screen);
        }
        for id in &output_ui.textures_delta.free {self.painter.free_texture(id);}
        commands
    }
}
