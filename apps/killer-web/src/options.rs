use super::Runtime;
use crate::ui_option_config::{ConfigEdit,edit_ini};
use crate::ui_video::{DisplayBackend,RendererVideoCapabilities,VideoOptionRuntime};
use killer_assets::textures::{BankSource,TextureBank};
use killer_render::renderer::Renderer;
use killer_render::video::{GammaSettings,TextureFiltering};
use killer_ui::ini::{IniFile,parse_ini};
use killer_ui::option_audio::{AudioApplyBackend,AudioChannel};
use killer_ui::option_video::{VideoPage,VideoSettings};
use killer_ui::option_video_apply::{ConfigValue,ResolutionParams,ResolutionRequest,VideoApplyBackend};
use killer_ui::window::WindowId;
use killer_world::World;

pub struct CanvasDisplay {
    pub params: ResolutionParams,
}

impl DisplayBackend for CanvasDisplay {
    fn params(&self) -> ResolutionParams {self.params}
    fn modes(&self) -> Vec<(i32,i32,i32)> {vec![(self.params.width,self.params.height,32)]}
    fn rates(&self,_mode: (i32,i32,i32)) -> Vec<i32> {Vec::new()}
}

pub struct Video {
    runtime: VideoOptionRuntime<CanvasDisplay>,
    env: IniFile,
    shown: bool,
    antialiasing: i32,
}

impl Video {
    pub fn load(client: &mut killer_assets::client::Client, options: &killer_assets::options::ClientOptions, renderer: &Renderer, adapter: &wgpu::Adapter, size: (u32,u32)) -> Result<Self,String> {
        let option=parse_ini(&client.read_system_text("option.ini").map_err(|e|e.to_string())?);
        let l2=parse_ini(&client.read_system_text("l2.ini").map_err(|e|e.to_string())?);
        let env=parse_ini(&client.read_system_text("env.int").map_err(|e|e.to_string())?);
        let display=CanvasDisplay {params:ResolutionParams {width:size.0 as i32,height:size.1 as i32,bytes_per_pixel:4,refresh_rate:0,fullscreen:0}};
        let mut runtime=VideoOptionRuntime::from_ini(&option,&l2,options,display,RendererVideoCapabilities::query(renderer))?;
        runtime.state.screenshot_quality=option.int("Game","ScreenShotQuality").unwrap_or(0);
        Ok(Self {runtime,env,shown:false,antialiasing:killer_render::video::multisample_capability(adapter,renderer.device())})
    }

    pub fn texture_detail(&self) -> i32 {
        let settings=self.runtime.settings();
        if settings.keep_min_frame_rate!=0 {2}else{settings.texture_detail}
    }

    fn show(&mut self,page: &mut VideoPage,lookup: impl Fn(u32)->Option<String>) {
        self.runtime.show(page,self.antialiasing,lookup);
        page.combos[0].enabled=false;
        page.combos[1].enabled=false;
        page.combos[1].entries=vec!["Pelo navegador".into()];
        page.combos[1].selected=0;
    }
    pub fn reset(&mut self) {self.shown=false;}
    pub fn resize(&mut self,size: (u32,u32)) {
        self.runtime.display.params.width=size.0 as i32;
        self.runtime.display.params.height=size.1 as i32;
    }
}

struct VideoBackend<'a> {
    renderer: &'a mut Renderer,
    adapter: &'a wgpu::Adapter,
    client: &'a mut killer_assets::client::Client,
    bank: &'a TextureBank,
    gamma: &'a mut GammaSettings,
    env: &'a IniFile,
    display: &'a CanvasDisplay,
    edits: Vec<ConfigEdit>,
    error: Option<String>,
}

impl VideoApplyBackend for VideoBackend<'_> {
    fn fullscreen(&mut self) -> i32 {self.display.params.fullscreen}
    fn uses_trilinear(&mut self) -> i32 {i32::from(self.renderer.texture_filtering().trilinear)}
    fn set_trilinear(&mut self,value: i32) {
        self.renderer.set_texture_filtering(TextureFiltering {trilinear:value!=0,..self.renderer.texture_filtering()});
    }
    fn delete_all_textures(&mut self,settings: &VideoSettings) {
        let detail=if settings.keep_min_frame_rate!=0 {2}else{settings.texture_detail};
        let mut bank=crate::ui_video::reload_bank(self.bank);
        self.renderer.reload_texture_detail(detail,&mut BankSource {bank:&mut bank,catalog:self.client.catalog()});
    }
    fn modes(&mut self) -> Vec<(i32,i32,i32)> {self.display.modes()}
    fn refresh_rates(&mut self,mode: (i32,i32,i32)) -> Vec<i32> {self.display.rates(mode)}
    fn set_resolution(&mut self,request: ResolutionRequest,settings: &mut VideoSettings) -> bool {
        let actual=self.display.params;
        if request.refresh_rate.is_some() || request.width!=actual.width || request.height!=actual.height || request.bytes_per_pixel!=4 {
            self.error=Some("A resolução física é controlada pelo navegador.".into());
            return false;
        }
        let post=killer_render::postprocess::PostProcessSettings::read(settings.post_process_type,settings.keep_min_frame_rate!=0,self.env);
        if !post.active() && let Err(error)=self.renderer.set_postprocess(post) {self.error=Some(error.into());return false;}
        if let Err(error)=self.renderer.set_antialiasing(self.adapter,settings.antialiasing) {self.error=Some(error.into());return false;}
        settings.mode=(actual.width,actual.height,32);
        true
    }
    fn resolution_params(&mut self) -> ResolutionParams {self.display.params}
    fn update_gamma(&mut self,value: f32) {self.gamma.gamma=value;self.renderer.set_gamma(*self.gamma);}
    fn set_option(&mut self,section: &str,key: &str,value: ConfigValue) {
        let value=match value {ConfigValue::Text(value)=>value,ConfigValue::Integer(value)=>value.to_string(),ConfigValue::Decimal(value)=>format!("{value:.6}")};
        self.edits.push(ConfigEdit::new(section,key,value));
    }
    fn read_float(&mut self,file: &str,section: &str,key: &str) -> Option<f32> {
        if file.eq_ignore_ascii_case("env.int") {self.env.get(section,key)?.parse().ok()}else{None}
    }
}

struct AudioBackend<'a> {
    audio: &'a mut crate::audio::Audio,
    controller_present: bool,
    edits: &'a mut Vec<ConfigEdit>,
}

impl AudioApplyBackend for AudioBackend<'_> {
    fn available(&mut self) -> bool {self.audio.available()}
    fn volume(&mut self,channel: AudioChannel) -> f32 {self.audio.config.volumes.get(channel)}
    fn set_volume(&mut self,channel: AudioChannel,value: f32) {self.audio.set_volume(channel,value);}
    fn stop(&mut self,channel: AudioChannel) {self.audio.stop_channel(channel);}
    fn has_music_controller(&mut self) -> bool {self.controller_present}
    fn reset_music_controller(&mut self) {self.audio.controller.reset_music_controll();}
    fn restore_lobby_music(&mut self) {
        if let Err(error)=crate::audio::music_state::restore_lobby_music(6,-1,&Default::default(),&mut -1,|_|Err(std::io::Error::other("lobby 3D indisponível no host web").into())) {log::warn!("RestoreLobbyMusic: {error}");}
    }
    fn write_volume(&mut self,file: &str,section: &str,key: &str,value: f32) {
        if file.eq_ignore_ascii_case("Option.ini") {self.edits.push(ConfigEdit::new(section,key,format!("{value:.6}")));}
        else {log::error!("áudio: arquivo inesperado {file}");}
    }
}

impl Runtime {
    pub fn initialize_video(&mut self,renderer: &mut Renderer,adapter: &wgpu::Adapter,mut video: Video) -> Result<(),String> {
        let settings=video.runtime.settings();
        let post=killer_render::postprocess::PostProcessSettings::read(settings.post_process_type,settings.keep_min_frame_rate!=0,&video.env);
        renderer.set_antialiasing(adapter,if post.active(){0}else{settings.antialiasing})?;
        renderer.set_postprocess(post)?;
        video.runtime.apply_render_state(renderer,&mut self.options);
        self.video=Some(video);
        Ok(())
    }

    pub fn sync_video_page(&mut self) {
        let open=self.ui.desktop.window(WindowId::OPTION).is_some_and(|window|window.visible);
        if let Some(video)=&mut self.video {
            if open && !video.shown {video.show(&mut self.ui.keys.option.video_page,|id|self.ui.keys.sysstrings.get(&id).cloned());}
            video.shown=open;
        }
    }

    pub fn option_resolution_changed(&mut self,selection: i32) {
        if let Some(video)=&self.video {video.runtime.resolution_changed(&mut self.ui.keys.option.video_page,selection);}
    }

    pub fn option_min_frame_changed(&mut self) {
        if let Some(video)=&self.video {
            let caps=video.runtime.capabilities;
            self.ui.keys.option.video_page.keep_min_frame_changed(caps.water,caps.shaders);
        }
    }

    fn apply_video(&mut self,renderer: &mut Renderer,adapter: &wgpu::Adapter) -> Result<Vec<ConfigEdit>,String> {
        let video=self.video.as_mut().ok_or("estado de vídeo indisponível")?;
        let previous=video.runtime.state.clone();
        let mut next=previous.clone();
        next.settings.reduce_pawn_lod=i32::from(self.ui.keys.option.video_page.combos[4].selected!=0);
        next.settings.skip_anim=i32::from(self.ui.keys.option.video_page.combos[5].selected!=0);
        crate::ui_video::validate_pawn_options(&previous,&next)?;
        let mut page=self.ui.keys.option.video_page.clone();
        let mut backend=VideoBackend {renderer,adapter,client:&mut self.client,bank:&self.bank,gamma:&mut video.runtime.gamma,env:&video.env,display:&video.runtime.display,edits:Vec::new(),error:None};
        page.apply(&mut next,&mut backend);
        if let Some(error)=backend.error {return Err(error);}
        let edits=backend.edits;
        if previous.settings.render_deco!=next.settings.render_deco || previous.settings.keep_min_frame_rate!=next.settings.keep_min_frame_rate {
            let mut options=self.options.clone();
            options.render_deco=next.settings.render_deco!=0;
            options.keep_min_frame_rate=next.settings.keep_min_frame_rate!=0;
            for (name,level) in std::iter::once(("",&self.neighbours.main_level)).chain(self.neighbours.tiles.iter().map(|(name,tile)|(name.as_str(),&tile.level))) {
                let zones=killer_assets::scene::zone_lighting(level);
                let draws=killer_assets::scene::decoration_draws(&mut self.client,level,&zones,&options);
                let mut bank=crate::ui_video::reload_bank(&self.bank);
                renderer.replace_decorations(name,&draws,&mut BankSource {bank:&mut bank,catalog:self.client.catalog()});
            }
        }
        self.display.pawn_clipping=if next.settings.keep_min_frame_rate!=0 {f32::from_bits(0x3f333333)}else{next.settings.pawn_clipping};
        self.display.render_limit=next.actor_count.max(0) as usize;
        let post=killer_render::postprocess::PostProcessSettings::read(next.settings.post_process_type,next.settings.keep_min_frame_rate!=0,&video.env);
        renderer.set_postprocess(post)?;
        video.runtime.state=next;
        video.runtime.apply_render_state(renderer,&mut self.options);
        self.ui.keys.option.video_page=page;
        Ok(edits)
    }

    pub fn apply_options(&mut self,renderer: &mut Renderer,adapter: &wgpu::Adapter,close: Option<bool>) -> Result<(),String> {
        let applied=self.apply_video(renderer,adapter);
        self.client.catalog().release_files();
        let mut edits=applied?;
        if let Some(close)=close {
            self.ui.keys.option.audio_page.apply(&mut AudioBackend {audio:&mut self.audio,controller_present:self.world.me.is_some(),edits:&mut edits});
            let mut effects=crate::ui_option_apply::GameApplyEffects::default();
            let mut native=None;
            if let Some(runtime)=&mut self.ui.option_game_runtime {
                let mut state=runtime.state.clone();
                self.ui.keys.option.game_page.apply(&mut state,&mut crate::ui_option_apply::GameApplyHost {runtime,shortcut:&mut self.ui.shortcut,edits:&mut edits,effects:&mut effects});
                native=Some(state.is_native!=0);
                runtime.state=state;
            }
            let (section,key,value)=crate::ui_option_apply::FIRST_RUN;
            edits.push(ConfigEdit::new(section,key,value));
            if let (true,Some(native))=(effects.clear_game_data,native) {
                self.tables=Default::default();
                let mut language=self.client.client_language();
                language.is_native=native;
                match self.client.tables_for_language(&language) {
                    Ok(tables)=>self.tables=tables.into(),
                    Err(error)=>self.push_chat(killer_ui::chat::ChatEntry::new(-1,"",format!("FL2GameData::Load: {error}"))),
                }
                self.client.catalog().release_files();
                if effects.refresh_localized_windows {crate::ui_option_apply::relocalize_windows(&mut self.ui.keys,&self.tables);}
            }
            crate::ui_option_apply::apply_system_message(&mut self.ui.desktop,&effects);
            self.save_options(edits)?;
            if close {
                self.ui.keys.option.lose_combo_focus();
                if let Some(window)=self.ui.desktop.window_mut(WindowId::OPTION) {window.visible=false;}
            }
        } else {self.save_options(edits)?;}
        Ok(())
    }

    pub fn save_windows_info(&mut self) {
        let current=self.client.read_system_text("WindowsInfo.ini").unwrap_or_default();
        let saved=crate::ui_option_config::edit_ini(&current,&crate::ui_windows_info::edits(&self.ui));
        if self.windows_saved.as_deref()==Some(saved.as_str()) || saved==current {
            self.windows_saved=Some(saved);
            return;
        }
        match self.client.write_system_text("WindowsInfo.ini",&saved) {
            Ok(())=>self.windows_saved=Some(saved),
            Err(error)=>log::warn!("WindowsInfo.ini: {error}"),
        }
    }

    fn save_options(&mut self,edits: Vec<ConfigEdit>) -> Result<(),String> {
        let text=self.client.read_system_text("option.ini").map_err(|e|e.to_string())?;
        let saved=edit_ini(&text,&edits);
        self.client.write_system_text("option.ini",&saved).map_err(|e|e.to_string())?;
        crate::ui_options::synchronize_game_options(&mut self.ui,parse_ini(&saved));
        crate::ui_options::synchronize_name_options(&mut self.display,&self.ui.options);
        Ok(())
    }

    pub fn toggle_min_frame(&mut self,renderer: &mut Renderer,adapter: &wgpu::Adapter) -> Result<(),String> {
        let video=self.video.as_mut().ok_or("estado de vídeo indisponível")?;
        let enabled=video.runtime.state.settings.keep_min_frame_rate!=0;
        video.show(&mut self.ui.keys.option.video_page,|id|self.ui.keys.sysstrings.get(&id).cloned());
        self.ui.keys.option.video_page.checks[0].checked = !enabled;
        self.option_min_frame_changed();
        let mut line=killer_world::ChatLine::new(killer_world::chat::CHAT_SYSTEM,String::new(),crate::ui2d::system_text(Some(&self.tables),if enabled {1284}else{1283},&[]));
        line.color=Some(0xffb09b79);
        self.push_chat(crate::ui2d::chat_entry(&line));
        self.apply_options(renderer,adapter,None)
    }
}

pub struct LobbyOptions {
    pub ui: crate::ui2d::GameUi,
    video: Option<Video>,
    options: killer_assets::options::ClientOptions,
    captured: bool,
    size: (i32,i32),
    pub native_language: Option<bool>,
}

impl LobbyOptions {
    pub fn load(client: &mut killer_assets::client::Client, tables: &killer_assets::client::DatTables, size: (i32,i32)) -> Result<Self,String> {
        let mut ui=crate::ui2d::GameUi::load(client,size).map_err(|e|e.to_string())?;
        ui.set_tab_names(tables);
        let options=killer_assets::options::ClientOptions::load(client).map_err(|e|e.to_string())?;
        Ok(Self {ui,video:None,options,captured:false,size,native_language:None})
    }

    pub fn open(&self) -> bool {self.ui.option_open()}

    pub fn resize(&mut self,size: (i32,i32)) {
        if self.size==size {return;}
        self.size=size;
        self.ui.resized(size);
        if let Some(video)=&mut self.video {video.resize((size.0.max(1) as u32,size.1.max(1) as u32));}
    }

    pub fn toggle(&mut self,client: &mut killer_assets::client::Client,renderer: &mut Renderer,adapter: &wgpu::Adapter,size: (u32,u32)) -> Result<(),String> {
        let show=!self.open();
        if let Some(window)=self.ui.desktop.window_mut(WindowId::OPTION) {window.visible=show;}
        if show && self.video.is_none() {
            let video=Video::load(client,&self.options,renderer,adapter,size)?;
            let settings=video.runtime.settings();
            let post=killer_render::postprocess::PostProcessSettings::read(settings.post_process_type,settings.keep_min_frame_rate!=0,&video.env);
            renderer.set_antialiasing(adapter,if post.active(){0}else{settings.antialiasing})?;
            renderer.set_postprocess(post)?;
            let detail=if settings.keep_min_frame_rate!=0 {2}else{settings.texture_detail};
            let mut bank=TextureBank::new();
            renderer.reload_texture_detail(detail,&mut BankSource {bank:&mut bank,catalog:client.catalog()});
            let mut video=video;
            video.runtime.apply_render_state(renderer,&mut self.options);
            self.video=Some(video);
        }
        Ok(())
    }

    pub fn sync(&mut self) {
        let open=self.open();
        if let Some(video)=&mut self.video {
            if open && !video.shown {video.show(&mut self.ui.keys.option.video_page,|id|self.ui.keys.sysstrings.get(&id).cloned());}
            video.shown=open;
        }
        self.ui.sync_option_window();
    }

    fn over(&self,x: i32,y: i32) -> bool {
        self.ui.desktop.window(WindowId::OPTION).is_some_and(|window|window.visible && window.rect.contains(x,y))
    }

    pub fn pointer(&mut self,kind: &str,x: i32,y: i32,button: i32) -> bool {
        if !self.open() {return false;}
        let world=World::default();
        match kind {
            "move"=>{let dragging=self.ui.mouse_move(x,y);dragging || self.captured},
            "down" if button==0 && self.over(x,y)=>{
                let taken=self.ui.mouse_down_at(x,y,killer_platform::time::Instant::now(),&world);
                self.captured=taken;
                taken
            }
            "up" if button==0=>{
                if !std::mem::take(&mut self.captured) {return false;}
                self.ui.mouse_up_in(&world);
                true
            }
            _=>false,
        }
    }

    pub fn wheel(&mut self,x: i32,y: i32,ticks: f32) -> bool {
        self.open() && self.over(x,y) && self.ui.wheel_over(x,y,ticks,&[])
    }

    pub fn paint(&self,list: &mut killer_ui::draw::DrawList) {
        if let Some(window)=self.ui.desktop.window(WindowId::OPTION).filter(|window|window.visible) {
            self.ui.paint_window(list,window,&World::default(),&[],None,&crate::ui2d::WorldViews::default());
        }
    }

    pub fn run_commands(&mut self,client: &mut killer_assets::client::Client,audio: &mut crate::audio::Audio,renderer: &mut Renderer,adapter: &wgpu::Adapter) {
        use crate::ui_keys::UiCommand;
        for command in self.ui.take_commands() {
            match command {
                UiCommand::ApplyOptions {close}=>if let Err(error)=self.apply(client,audio,renderer,adapter,close) {log::warn!("Settings: {error}");},
                UiCommand::OptionResolutionChanged(selection)=>if let Some(video)=&self.video {video.runtime.resolution_changed(&mut self.ui.keys.option.video_page,selection);},
                UiCommand::OptionKeepMinFrameRateChanged=>if let Some(video)=&self.video {
                    let caps=video.runtime.capabilities;
                    self.ui.keys.option.video_page.keep_min_frame_changed(caps.water,caps.shaders);
                },
                UiCommand::InitializeInterface=>if let Err(error)=crate::ui_initialize_run::initialize(&mut self.ui) {log::warn!("Initialize: {error}");},
                UiCommand::Log(message)=>log::info!("{message}"),
                _=>{}
            }
        }
    }

    fn apply_video(&mut self,client: &mut killer_assets::client::Client,renderer: &mut Renderer,adapter: &wgpu::Adapter) -> Result<Vec<ConfigEdit>,String> {
        let video=self.video.as_mut().ok_or("estado de vídeo indisponível")?;
        let previous=video.runtime.state.clone();
        let mut next=previous.clone();
        next.settings.reduce_pawn_lod=i32::from(self.ui.keys.option.video_page.combos[4].selected!=0);
        next.settings.skip_anim=i32::from(self.ui.keys.option.video_page.combos[5].selected!=0);
        crate::ui_video::validate_pawn_options(&previous,&next)?;
        let mut page=self.ui.keys.option.video_page.clone();
        let bank=TextureBank::new();
        let mut backend=VideoBackend {renderer,adapter,client,bank:&bank,gamma:&mut video.runtime.gamma,env:&video.env,display:&video.runtime.display,edits:Vec::new(),error:None};
        page.apply(&mut next,&mut backend);
        if let Some(error)=backend.error {return Err(error);}
        let edits=backend.edits;
        let post=killer_render::postprocess::PostProcessSettings::read(next.settings.post_process_type,next.settings.keep_min_frame_rate!=0,&video.env);
        renderer.set_postprocess(post)?;
        video.runtime.state=next;
        video.runtime.apply_render_state(renderer,&mut self.options);
        self.ui.keys.option.video_page=page;
        Ok(edits)
    }

    fn apply(&mut self,client: &mut killer_assets::client::Client,audio: &mut crate::audio::Audio,renderer: &mut Renderer,adapter: &wgpu::Adapter,close: bool) -> Result<(),String> {
        let applied=self.apply_video(client,renderer,adapter);
        client.catalog().release_files();
        let mut edits=applied?;
        self.ui.keys.option.audio_page.apply(&mut AudioBackend {audio,controller_present:false,edits:&mut edits});
        let mut effects=crate::ui_option_apply::GameApplyEffects::default();
        if let Some(runtime)=&mut self.ui.option_game_runtime {
            let mut state=runtime.state.clone();
            self.ui.keys.option.game_page.apply(&mut state,&mut crate::ui_option_apply::GameApplyHost {runtime,shortcut:&mut self.ui.shortcut,edits:&mut edits,effects:&mut effects});
            if effects.clear_game_data {self.native_language=Some(state.is_native!=0);}
            runtime.state=state;
        }
        let (section,key,value)=crate::ui_option_apply::FIRST_RUN;
        edits.push(ConfigEdit::new(section,key,value));
        let text=client.read_system_text("option.ini").map_err(|e|e.to_string())?;
        let saved=edit_ini(&text,&edits);
        client.write_system_text("option.ini",&saved).map_err(|e|e.to_string())?;
        crate::ui_options::synchronize_game_options(&mut self.ui,parse_ini(&saved));
        if close {
            self.ui.keys.option.lose_combo_focus();
            if let Some(window)=self.ui.desktop.window_mut(WindowId::OPTION) {window.visible=false;}
        }
        Ok(())
    }

    pub fn relocalize(&mut self,tables: &killer_assets::client::DatTables) {
        self.ui.set_tab_names(tables);
        crate::ui_option_apply::relocalize_windows(&mut self.ui.keys,tables);
    }
}

#[cfg(feature="platform-check")]
pub async fn check() -> Result<String,String> {
    fn require(value: bool,message: &str) -> Result<(),String> {if value {Ok(())}else{Err(message.into())}}
    let instance=wgpu::Instance::default();
    let adapter=instance.request_adapter(&Default::default()).await.map_err(|e|e.to_string())?;
    let (device,queue)=adapter.request_device(&Default::default()).await.map_err(|e|e.to_string())?;
    let scope=device.push_error_scope(wgpu::ErrorFilter::Validation);
    let mut renderer=Renderer::new(device,queue,wgpu::TextureFormat::Rgba8Unorm);
    let text="; preserve\n[Game]\nIsNative=False\nMyName=True\n[Audio]\nSoundVolume=0.5\nMusicVolume=0\n[Video]\nGamma=1\nAntiAliasing=0\nUseTrilinear=True\n";
    let option=parse_ini(text);
    let l2=parse_ini("[WinDrv.WindowsClient]\nGamma=1\nBrightness=0.5\nContrast=0.5\n");
    let mut options=killer_assets::options::ClientOptions::default();
    let env=parse_ini("");
    let display=CanvasDisplay {params:ResolutionParams {width:800,height:600,bytes_per_pixel:4,refresh_rate:0,fullscreen:0}};
    let runtime=VideoOptionRuntime::from_ini(&option,&l2,&options,display,RendererVideoCapabilities::query(&renderer))?;
    let mut video=Video {runtime,env,shown:false,antialiasing:killer_render::video::multisample_capability(&adapter,renderer.device())};
    let mut page=VideoPage::new(|id|Some(id.to_string()));
    video.show(&mut page,|id|Some(id.to_string()));
    require(!page.combos[0].enabled && !page.combos[1].enabled,"seletores físicos devem estar indisponíveis")?;
    page.checks[3].checked=false;
    page.combos[2].selected=2;
    page.combos[7].selected=3;
    let bank=TextureBank::new();
    let mut client=killer_assets::client::Client::open(std::path::Path::new("/client")).map_err(|e|e.to_string())?;
    let mut next=video.runtime.state.clone();
    let mut backend=VideoBackend {renderer:&mut renderer,adapter:&adapter,client:&mut client,bank:&bank,gamma:&mut video.runtime.gamma,env:&video.env,display:&video.runtime.display,edits:Vec::new(),error:None};
    page.apply(&mut next,&mut backend);
    if let Some(error)=backend.error {return Err(error);}
    let mut edits=backend.edits;
    require(!edits.iter().any(|edit|edit.key=="RefreshRate" || edit.key=="GamePlayViewportX"),"não gravar frequência ou modo físico inventados")?;
    video.runtime.state=next;
    video.runtime.apply_render_state(&mut renderer,&mut options);
    require(!renderer.texture_filtering().trilinear && video.runtime.gamma.gamma==0.8 && options.clipping.pawn==1.5,"aplicação do vídeo no renderer")?;
    let mut audio=crate::audio::Audio::new(crate::audio::InitConfig::read(&crate::audio::config::AudioIni {option:&option,alaudio:&parse_ini("")}),2);
    audio.set_available(true);
    let frame=crate::audio::AudioFrame {time:0.0,view_actor_location:[0.0;3],max_radius_multiplier:50.0,realtime:false,replay_recording:false};
    require(audio.interface(vec![1],Some(frame))==1,"voz inicial")?;
    audio.drain();
    let mut audio_page=killer_ui::option_audio::AudioPage::new(|id|Some(id.to_string()));
    for (combo,value) in audio_page.combos.iter_mut().zip([3,0,2,2]) {combo.selected=value;}
    audio_page.apply(&mut AudioBackend {audio:&mut audio,controller_present:true,edits:&mut edits});
    require(audio.config.volumes.ui_values()==[0.4,1.0,0.6,0.6],"aplicação dos quatro canais")?;
    require(audio.controller==crate::audio::music_state::MusicController::Lineage {field_a34:0,flags_a38:1},"reset do controller de música")?;
    require(matches!(&audio.drain()[..],[crate::audio::Command::Gain {value,..}] if (*value-0.2).abs()<1e-6),"ganho da voz existente")?;
    let mut game=crate::ui_options::GameOptionRuntime::new(option,parse_ini(""),&l2,false,None);
    let mut game_page=killer_ui::option_game::GamePage::new(false,|id|Some(id.to_string()));
    let mut shortcut=killer_ui::shortcut::ShortcutView::new(false);
    game.show(&mut game_page,&mut shortcut,|id|Some(id.to_string()));
    game_page.checks.iter_mut().find(|check|check.id==0xf0f01459).ok_or("controle MyName ausente")?.check.checked=false;
    let mut effects=crate::ui_option_apply::GameApplyEffects::default();
    let mut state=game.state.clone();
    game_page.apply(&mut state,&mut crate::ui_option_apply::GameApplyHost {runtime:&mut game,shortcut:&mut shortcut,edits:&mut edits,effects:&mut effects});
    require(state.my_name==0,"estado do nome do jogador")?;
    let saved=edit_ini(text,&edits);
    client.write_system_text("option.ini",&saved).map_err(|e|e.to_string())?;
    let restored=client.read_system_text("option.ini").map_err(|e|e.to_string())?;
    require(restored.starts_with("; preserve\n"),"preservação do restante do ini")?;
    let ini=parse_ini(&restored);
    require(ini.bool("Game","MyName")==Some(false) && ini.get("Audio","SoundVolume")==Some("0.400000"),"persistência de jogo e áudio")?;
    let texture=|format|renderer.device().create_texture(&wgpu::TextureDescriptor {label:None,size:wgpu::Extent3d {width:16,height:16,depth_or_array_layers:1},mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format,usage:wgpu::TextureUsages::RENDER_ATTACHMENT,view_formats:&[]});
    let output=texture(wgpu::TextureFormat::Rgba8Unorm).create_view(&Default::default());
    let depth=texture(killer_render::pipeline::DEPTH_FORMAT).create_view(&Default::default());
    let mut encoder=renderer.device().create_command_encoder(&Default::default());
    let camera=killer_render::camera::Camera {eye:glam::Vec3::ZERO,target:glam::Vec3::X,fov_x_deg:60.0,near:1.0,far:1000.0};
    renderer.render(&mut encoder,&output,&depth,(16,16),&camera,&Default::default());
    renderer.queue().submit([encoder.finish()]);
    if let Some(error)=scope.pop().await {return Err(error.to_string());}
    Ok("Opções passaram: vídeo aplicado e quadro WebGPU sem erros, canais/ganho de áudio, estado de jogo e persistência; sem frequência física inventada.".into())
}
