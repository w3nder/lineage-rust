use crate::characters::{self, CharacterCache, LoadedCharacter};
use crate::loader::{LoadResult, Shared, load_character};
use crate::lobby::{Lobby, LoadedLobby, scene::SceneBook};
use killer_assets::client::{Client, DatTables};
use killer_assets::scene::actor_lighting::LightMap;
use killer_assets::textures::{BankSource, TextureBank};
use killer_assets::timeenv::TimeEnv;
use killer_render::renderer::{Renderer, Viewer};
use std::sync::Arc;

pub struct WebLobby {
    pub lobby: Lobby,
    pub audio: crate::audio::Audio,
    audio_time: f64,
    audio_frame: Option<crate::audio::AudioFrame>,
    pub loading_stats: String,
    client: Client,
    tables: Arc<DatTables>,
    pub cache: CharacterCache,
    shared: Shared,
    effects: crate::pawn_effects::Effects,
    font: killer_ui::font::BitmapFont,
    credit_font: Option<killer_ui::font::BitmapFont>,
    c4: Option<crate::lobby_c4::LobbyC4>,
    ui_bank: TextureBank,
    options: Option<crate::runtime::options::LobbyOptions>,
}

impl WebLobby {
    pub fn load(mut lobby: Lobby, renderer: &mut Renderer, prepared: Option<web_sys::Blob>, table_cache: &mut crate::table_cache::SessionTables, report: &dyn Fn(&str, f64)) -> Result<Self,String> {
        let started=killer_platform::time::Instant::now();
        let mut client=Client::open(std::path::Path::new("/client")).map_err(|e|e.to_string())?;
        let option=killer_ui::ini::parse_ini(&client.read_system_text("option.ini").map_err(|e|e.to_string())?);
        let alaudio=killer_ui::ini::parse_ini(&client.read_system_text("alaudio.int").map_err(|e|e.to_string())?);
        let system=killer_ui::ini::parse_ini(&client.read_system_text("l2.ini").map_err(|e|e.to_string())?);
        let mut audio=crate::audio::Audio::new(crate::audio::InitConfig::read(&crate::audio::config::AudioIni {option:&option,alaudio:&alaudio}),system.int("ALAudio.ALAudioSubsystem","Channels").unwrap_or(0));
        audio.configure_lobby(&alaudio);
        let open_ms=started.elapsed().as_secs_f64()*1000.0;
        report("open",open_ms);
        let loaded=client.load_level(crate::lobby::LOBBY_MAP).map_err(|e|e.to_string())?;
        let level_ms=started.elapsed().as_secs_f64()*1000.0;
        report("level",level_ms);
        let book=SceneBook::from_level(&loaded.level);
        if book.warp_placement(crate::lobby::scene::TAG_LOGON).is_none() {return Err("lobby sem cena LogOn_Warp".into());}
        let logongrp=client.logongrp().map_err(|e|e.to_string())?;
        let idc_names=client.idc_names().map(|rows|rows.into_iter().map(|row|(row.id,row.name)).collect()).unwrap_or_default();
        let server_names=client.server_names().map_err(|e|e.to_string())?.into_iter().map(|row|(row.id,row.name)).collect();
        let class_info=client.class_info().map_err(|e|e.to_string())?;
        let timeenv=TimeEnv::load(&mut client,0).map_err(|e|e.to_string())?;
        let config_ms=started.elapsed().as_secs_f64()*1000.0;
        report("config",config_ms);
        let tables=table_cache.load(&mut client).map_err(|e|e.to_string())?;
        let read_ms=started.elapsed().as_secs_f64()*1000.0;
        report("tables",read_ms);
        let options=killer_assets::options::ClientOptions::load(&mut client).map_err(|e|e.to_string())?;
        renderer.set_no_fractal_anim(options.no_fractal_anim);
        renderer.set_projectors(options.projectors);
        renderer.set_pawn_shadow(options.pawn_shadow,options.keep_min_frame_rate);
        let scene=Arc::new(if let Some(file)=prepared {
            let reader=std::io::BufReader::with_capacity(1 << 20,killer_platform::fs::File::from_blob(file));
            let mut scene=killer_assets::cache::decode(lz4_flex::frame::FrameDecoder::new(reader),&crate::preparation::STAMP).map_err(|e|format!("cache do lobby: {e:?}"))?.scene;
            scene.emitters=killer_assets::scene::emitters::build(&mut client,&loaded.level,&options);
            scene
        } else {killer_assets::scene::build_scene_with(&mut client,&loaded.level,&killer_assets::options::ClientOptions {shared_sky:false,..options.clone()},Default::default())});
        let scene_ms=started.elapsed().as_secs_f64()*1000.0;
        report("scene",scene_ms);
        let ground=crate::ground::GroundGrid::build(&scene);
        let ground_ms=started.elapsed().as_secs_f64()*1000.0;
        report("ground",ground_ms);
        let mut bank=TextureBank::new();
        renderer.upload(&scene,&mut BankSource {bank:&mut bank,catalog:client.catalog()});
        let upload_ms=started.elapsed().as_secs_f64()*1000.0;
        report("upload",upload_ms);
        renderer.set_instances(&[]);
        renderer.clear_ui();
        let mut data=LoadedLobby {lighting:LightMap::new(&loaded.level,&scene.lights),scene,bank,timeenv,bsp_tree:loaded.level.bsp_tree,bsp_render:loaded.level.bsp_render,book,logongrp,server_names,idc_names,class_info,rotation_defaults:crate::lobby::load_rotation_defaults(&mut client),elapsed:Default::default()};
        lobby.install(&mut data);
        lobby.install_ground(ground);
        let font=crate::ui2d::load_lobby_font(&mut client).map_err(|e|e.to_string())?;
        let credit_font=crate::ui2d::load_named_font(&mut client,killer_ui::lobby_credit::FONT_TEXTURE,killer_ui::lobby_credit::FONT_FILE).map_err(|e|log::warn!("fonte dos créditos: {e}")).ok();
        client.catalog().release_files();
        let total_ms=started.elapsed().as_secs_f64()*1000.0;
        let loading_stats=serde_json::json!({"tile":"lobby","read_tables_ms":read_ms,"open_ms":open_ms,"level_ms":level_ms-open_ms,"config_ms":config_ms-level_ms,"tables_ms":read_ms-config_ms,"ground_ms":ground_ms-scene_ms,"upload_ms":upload_ms-ground_ms,"scene_ms":scene_ms-read_ms,"ground_upload_ms":upload_ms-scene_ms,"install_ms":total_ms-upload_ms,"total_ms":total_ms}).to_string();
        Ok(Self {audio,audio_time:0.0,audio_frame:None,lobby,loading_stats,client,tables,cache:Default::default(),shared:Default::default(),effects:Default::default(),font,credit_font,c4:None,ui_bank:TextureBank::new(),options:None})
    }

    fn interface_sound(&mut self,name: &str) {
        if self.audio.available() && self.audio_frame.is_some() && !name.is_empty() && name!="NAME_None"
            && let Some((format,encoded))=self.client.sound(name) {
            if format.eq_ignore_ascii_case("WAV") {self.audio.interface(encoded,self.audio_frame);}
            else {log::warn!("USound {name}: formato {format} indisponível");}
        }
    }

    pub fn release(&mut self,renderer: &mut Renderer) {
        self.effects.clear(renderer);
        renderer.set_instances(&[]);
        for id in self.cache.drain() {renderer.remove_character(id);}
    }

    pub fn tick(&mut self,renderer: &mut Renderer,dt: f32,size: (u32,u32)) {
        self.audio_time+=f64::from(dt);
        let state=match self.lobby.screen {
            crate::lobby::LobbyScreen::Login | crate::lobby::LobbyScreen::Servers=>2,
            crate::lobby::LobbyScreen::Characters=>4,
            crate::lobby::LobbyScreen::Create=>3,
        };
        let frame=crate::audio::AudioFrame {time:self.audio_time,view_actor_location:self.lobby.camera().eye.to_array(),max_radius_multiplier:50.0,realtime:false,replay_recording:false};
        self.audio_frame=Some(frame);
        if let Err(error)=self.audio.update_lobby(std::path::Path::new("/client"),frame,state,self.lobby.create.race.map(|race|race as i32)) {log::warn!("áudio do lobby: {error}");}
        let credits=self.c4.as_ref().is_some_and(|c4|c4.credits_visible());
        if let Err(error)=self.audio.set_credits(std::path::Path::new("/client"),frame,credits,(self.audio_time*1000.0) as u32) {log::warn!("música dos créditos: {error}");}
        let dt=self.lobby.tick(std::time::Duration::from_secs_f32(dt));
        let instances=self.lobby.instances(&mut self.cache,&self.tables,dt,size);
        renderer.set_instances(&instances);
        let camera=self.lobby.camera();
        renderer.set_viewer(Viewer {zone:self.lobby.view.as_ref().map_or(0,|view|view.zone_at(camera.eye)),position:camera.eye,dt});
        for key in self.cache.take_wanted(1) {
            let Some(look)=characters::look(key,&self.tables) else {self.cache.fail(key);continue;};
            match load_character(&mut self.client,&mut self.shared,key,&look.appearance,look.draw_scale) {
                LoadResult::Character {parts,animation,draw_scale,weapon_type,mut bank,render_defaults,effects,..}=>{
                    let parts=parts.into_iter().map(|mesh| {
                        let id=renderer.upload_character(&mesh,&mut BankSource {bank:&mut bank,catalog:self.client.catalog()});
                        (id,Arc::new(mesh))
                    }).collect();
                    if let Some(mut character)=LoadedCharacter::new(parts,animation,draw_scale,weapon_type) {
                        character.render_defaults=render_defaults;
                        self.cache.insert(key,character.with_effects(effects.effects,effects.bank));
                    } else {self.cache.fail(key);}
                }
                _=>self.cache.fail(key),
            }
            self.client.catalog().release_files();
        }
        self.effects.sync(renderer,&mut self.cache,&mut self.client,&self.lobby.world,&Default::default());
    }

    pub fn c4_frame(&mut self,renderer: &mut Renderer,adapter: &wgpu::Adapter,ui_size: (i32,i32),dt_ms: u32,frame_size: (u32,u32)) {
        let class_info: &[l2_dat::ClassInfo]=self.lobby.view.as_ref().map_or(&[],|view|view.class_info.as_slice());
        let c4=self.c4.get_or_insert_with(||crate::lobby_c4::LobbyC4::new(ui_size,&self.tables,class_info));
        c4.resize(ui_size);
        c4.update(&mut self.lobby,dt_ms);
        self.lobby.voices.clear();
        self.lobby.stop_voices=false;
        if c4.needs_credit_font() {c4.set_credit_font(self.credit_font.take());}
        if let Some(link)=c4.take_credit_request() {
            match self.client.read_help_text(&link) {
                Ok(text)=>c4.set_credit_html(&text,&self.font,&|marker,id|crate::ui2d::html_marker(Some(&self.tables),marker,id)),
                Err(error)=>log::warn!("créditos {link}: {error}"),
            }
        }
        let _=c4.take_ping_requests();
        if let Some(link)=c4.take_help_request() {
            match self.client.read_help_text(&link) {
                Ok(text)=>c4.set_help(&text,&self.font,&|marker,id|crate::ui2d::html_marker(Some(&self.tables),marker,id)),
                Err(error)=>log::warn!("ajuda do servidor {link}: {error}"),
            }
        }
        let slots=self.lobby.slots((ui_size.0 as f32,ui_size.1 as f32),1.0);
        let tags=c4.name_tags(&self.lobby,&slots);
        if c4.take_options_close() && let Some(options)=&mut self.options && options.open()
            && let Err(error)=options.toggle(&mut self.client,renderer,adapter,frame_size) {log::warn!("opções do lobby: {error}");}
        let Some(c4)=self.c4.as_mut() else {return};
        if c4.take_options_request() {
            let show=!self.options.as_ref().is_some_and(|options|options.open());
            self.interface_sound(if show {"itemsound.window_open"} else {"itemsound.window_close"});
            if self.options.is_none() {
                match crate::runtime::options::LobbyOptions::load(&mut self.client,&self.tables,ui_size) {
                    Ok(options)=>self.options=Some(options),
                    Err(error)=>log::warn!("opções do lobby: {error}"),
                }
            }
            if let Some(options)=&mut self.options && let Err(error)=options.toggle(&mut self.client,renderer,adapter,frame_size) {log::warn!("opções do lobby: {error}");}
        }
        if let Some(options)=&mut self.options {
            options.resize(ui_size);
            options.sync();
            options.run_commands(&mut self.client,&mut self.audio,renderer,adapter);
            if let Some(native)=options.native_language.take() {
                let mut language=self.client.client_language();
                language.is_native=native;
                match self.client.tables_for_language(&language) {
                    Ok(tables)=>{
                        self.tables=std::sync::Arc::new(tables);
                        options.relocalize(&self.tables);
                        self.c4=None;
                    }
                    Err(error)=>log::warn!("FL2GameData::Load: {error}"),
                }
                self.client.catalog().release_files();
            }
        }
        let Some(c4)=self.c4.as_mut() else {return};
        let mut list=c4.frame(&self.lobby,&self.font,&tags);
        if let Some(options)=&self.options {options.paint(&mut list);}
        renderer.set_ui_size(Some((ui_size.0 as u32,ui_size.1 as u32)));
        renderer.set_ui(&list,&mut BankSource {bank:&mut self.ui_bank,catalog:self.client.catalog()});
    }

    pub fn c4_pointer(&mut self,kind: &str,x: f32,y: f32,button: i32) {
        if let Some(options)=&mut self.options && options.pointer(kind,x as i32,y as i32,button) {return;}
        let Some(c4)=self.c4.as_mut() else {return};
        let (x,y)=(x as i32,y as i32);
        match (kind,button) {
            ("move",_)=>c4.mouse_move(x,y,&self.lobby),
            ("down",0)=>{
                c4.mouse_down(x,y,&mut self.lobby,&self.font);
                for name in std::mem::take(&mut c4.sounds) {self.interface_sound(name);}
            }
            ("up",0)=>c4.mouse_up(x,y,&mut self.lobby,&self.font),
            _=>{}
        }
    }

    pub fn c4_wheel(&mut self,ticks: f32) {
        if let Some(c4)=self.c4.as_mut() {c4.wheel(ticks,&self.lobby,&self.font);}
    }

    pub fn c4_wheel_at(&mut self,x: f32,y: f32,ticks: f32) -> bool {
        self.options.as_mut().is_some_and(|options|options.wheel(x as i32,y as i32,ticks))
    }

    pub fn c4_key(&mut self,text: &str,shift: bool) {
        use killer_ui::chat::InputKey;
        let Some(c4)=self.c4.as_mut() else {return};
        let key=match text {
            "Enter"=>Some(InputKey::Enter),"Escape"=>Some(InputKey::Escape),"Backspace"=>Some(InputKey::Backspace),
            "ArrowLeft"=>Some(InputKey::Left),"ArrowRight"=>Some(InputKey::Right),"Tab"=>Some(InputKey::Char('\t')),
            _=>text.chars().next().filter(|_|text.chars().count()==1).map(InputKey::Char),
        };
        if let Some(key)=key {c4.key(key,shift,&mut self.lobby,&self.font);}
    }

    pub fn refusal_text(&self,code: u8) -> Option<String> {
        killer_ui::lobby_login::refusal_text(code,|id|self.tables.systemmsg.get(&id).map(|message|message.text.clone()))
    }

    pub fn blocked_text(&self,flags: Option<u32>,messages: &[String]) -> Option<String> {
        let lookup=|id: u32|self.tables.systemmsg.get(&id).map(|message|message.text.clone());
        match (flags,messages.last()) {
            (_,Some(message))=>killer_ui::lobby_login::blocked_message_text(message,lookup),
            (Some(flags),None)=>killer_ui::lobby_login::blocked_text(flags,lookup),
            (None,None)=>None,
        }
    }

    pub fn c4_commands(&mut self) -> Vec<crate::lobby::LobbyCommand> {
        self.c4.as_mut().map_or(Vec::new(),|c4|c4.take_commands())
    }
}
