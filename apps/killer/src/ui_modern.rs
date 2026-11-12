#[cfg(not(target_arch = "wasm32"))]
use crate::lobby::config::ServerConfig;
use crate::ui2d::{GameUi, status_view};
use egui::{Align2, Color32, Context, FontId, Id, LayerId, Order, Painter, Pos2, Rect, Stroke, StrokeKind, TextureHandle, TextureOptions, Vec2};
use killer_assets::client::Client;
use killer_ui::chat::{self, ChatEntry};
use killer_ui::modern_shortcut::{AUTO_MAX, AUTO_MIN, AutoHit, ModernBarButton, ModernShortcuts, ShotKind, Vital};
use killer_ui::shortcut::{SLOTS, SlotKind};
use killer_ui::window::{Rect as NativeRect, WindowId};
use killer_world::World;
use std::collections::HashMap;

pub const STATUS_SIZE: (i32,i32) = (256,76);

use crate::ui_theme as theme;
pub(crate) use crate::ui_theme::{GOLD, MUTED, TEXT};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HudPreferences {
    pub enabled: bool,
    pub bars: ModernShortcuts,
    pub primary: i32,
    pub potions: [i32; 3],
    pub percents: [i32; 3],
    pub potions_off: [bool; 3],
    pub shots: [i32; 2],
}

impl HudPreferences {
    pub fn parse(text: &str) -> Self {
        let mut value = Self::default();
        for line in text.lines() {
            let Some((key, raw)) = line.split_once('=') else { continue };
            let raw = raw.trim();
            match key.trim() {
                "modern" => value.enabled = raw == "true",
                "locked" => value.bars.locked = raw == "true",
                "bars" => if let Ok(count) = raw.parse() { value.bars.count = count; },
                "primary" => if let Ok(page) = raw.parse::<i32>() { value.primary = page.clamp(0, 9); },
                "potions" => for (id, raw) in value.potions.iter_mut().zip(raw.split(',')) {
                    if let Ok(number) = raw.trim().parse::<i32>() { *id = number.max(0); }
                },
                "potions_off" => for (on, raw) in value.potions_off.iter_mut().zip(raw.split(',')) { *on = raw.trim() == "1"; },
                "potion_percents" => for (percent, raw) in value.percents.iter_mut().zip(raw.split(',')) {
                    if let Ok(number) = raw.trim().parse::<i32>() { *percent = number.clamp(AUTO_MIN, AUTO_MAX); }
                },
                "shots" => for (id, raw) in value.shots.iter_mut().zip(raw.split(',')) {
                    if let Ok(number) = raw.trim().parse::<i32>() { *id = number.max(0); }
                },
                "pages" => for (page, raw) in value.bars.pages.iter_mut().zip(raw.split(',')) {
                    if let Ok(number) = raw.trim().parse() { *page = number; }
                },
                _ => {}
            }
        }
        value.bars = value.bars.normalized();
        value
    }

    pub fn text(self) -> String {
        format!("modern={}\nbars={}\nlocked={}\nprimary={}\npages={}\npotions={}\npotions_off={}\npotion_percents={}\nshots={}\n", self.enabled, self.bars.count, self.bars.locked, self.primary,
            self.bars.pages.map(|v| v.to_string()).join(","), self.potions.map(|v| v.to_string()).join(","), self.potions_off.map(|v| if v {"1"} else {"0"}).join(","), self.percents.map(|v| v.to_string()).join(","), self.shots.map(|v| v.to_string()).join(","))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn path() -> Option<std::path::PathBuf> { ServerConfig::path().map(|path| path.with_file_name("hud.ini")) }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> Self {
        Self::path().and_then(|path| std::fs::read_to_string(path).ok()).map(|text| Self::parse(&text)).unwrap_or_default()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save(self) -> std::io::Result<()> {
        let path = Self::path().ok_or_else(|| std::io::Error::other("pasta de configuração indisponível"))?;
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
        let temporary = path.with_extension("ini.tmp");
        std::fs::write(&temporary, self.text())?;
        std::fs::rename(temporary, path)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load() -> Self { Self::parse(&killer_platform::hud_preferences()) }

    #[cfg(target_arch = "wasm32")]
    fn save(self) -> std::io::Result<()> {
        killer_platform::set_hud_preferences(self.text());
        Ok(())
    }
}

#[derive(Default)]
pub struct ModernHud {
    pub preferences: HudPreferences,
    saved: Option<HudPreferences>,
    cast_visual: Option<(String,String)>,
    name_mesh: std::cell::RefCell<killer_render::ui::UiMesh>,
    name_shapes: std::cell::RefCell<Vec<(f32,Vec<egui::epaint::ClippedShape>)>>,
    icons: HashMap<String, Option<TextureHandle>>,
    windows: HashMap<WindowId,killer_ui::draw::DrawList>,
    overlays: Vec<(NativeRect,killer_ui::draw::DrawList)>,
    minimized: killer_ui::draw::DrawList,
}

impl ModernHud {
    pub fn configure(&mut self, style: Option<&str>) {
        self.preferences = HudPreferences::load();
        self.saved = Some(self.preferences);
        match style {
            Some("modern") => self.preferences.enabled = true,
            Some("classic") => self.preferences.enabled = false,
            _ => {}
        }
        if self.saved != Some(self.preferences) {
            if let Err(error) = self.preferences.save() { log::warn!("preferências do HUD: {error}"); }
            self.saved = Some(self.preferences);
        }
    }

    pub fn sync(&mut self, ui: &mut GameUi) {
        if !self.preferences.enabled { return; }
        if ui.shortcut.modern.is_none() {
            ui.shortcut.modern = Some(self.preferences.bars);
            ui.shortcut.vertical = false;
            ui.shortcut.set_page(self.preferences.primary);
            for (slot, shot) in ui.shortcut.auto.shots.iter_mut().enumerate() { shot.item_id = self.preferences.shots[slot]; }
            for (slot, potion) in ui.shortcut.auto.potions.iter_mut().enumerate() {
                potion.item_id = self.preferences.potions[slot];
                potion.on = !self.preferences.potions_off[slot];
                if self.preferences.percents[slot] > 0 { potion.percent = self.preferences.percents[slot]; }
            }
            let (width, height) = ui.shortcut.size();
            let size = ui.desktop.size;
            if let Some(window) = ui.desktop.window_mut(WindowId::SHORTCUT) {
                window.rect = NativeRect::new(((size.0 - width) / 2).max(0), (size.1 - height - 16).max(0), width, height);
                window.grip = ui.shortcut.grip();
                window.initial_position = Some(killer_ui::initial_position::InitialPosition { enabled:true,horizontal_mode:2,vertical_mode:4,x:width,y:height+16 });
            }
            if let Some(window) = ui.desktop.window_mut(WindowId::STATUS) {
                window.rect = NativeRect::new(16,16,STATUS_SIZE.0,STATUS_SIZE.1);
                window.grip = NativeRect::new(0,0,STATUS_SIZE.0,23);
                window.initial_position = Some(killer_ui::initial_position::InitialPosition { enabled:true,horizontal_mode:0,vertical_mode:3,x:16,y:16 });
            }
            ui.chat.height = 262;
            if let Some(window) = ui.desktop.window_mut(WindowId::CHAT) {
                window.rect = NativeRect::new(12,(size.1 - 278).max(0),chat::CHAT_WIDTH,262);
                window.initial_position = Some(killer_ui::initial_position::InitialPosition { enabled:true,horizontal_mode:0,vertical_mode:4,x:12,y:278 });
            }
            if let Some(window)=ui.desktop.window_mut(WindowId::ABNORMAL_STATUS) {
                window.rect.x=288;window.rect.y=16;
                window.initial_position=Some(killer_ui::initial_position::InitialPosition {enabled:true,horizontal_mode:0,vertical_mode:3,x:288,y:16});
            }
            ui.follow_chat();
        }
        ui.layout_modern_menu();
        if let Some(bars) = ui.shortcut.modern { self.preferences.bars = bars; }
        self.preferences.primary = ui.shortcut.page;
        for (slot, shot) in ui.shortcut.auto.shots.iter().enumerate() { self.preferences.shots[slot] = shot.item_id; }
        for (slot, potion) in ui.shortcut.auto.potions.iter().enumerate() {
            self.preferences.potions[slot] = potion.item_id;
            self.preferences.potions_off[slot] = !potion.on;
            self.preferences.percents[slot] = potion.percent;
        }
        if self.saved != Some(self.preferences) {
            if let Err(error) = self.preferences.save() { log::warn!("preferências do HUD: {error}"); }
            self.saved = Some(self.preferences);
        }
    }

    pub fn capture_windows(&mut self,ui:&GameUi,world:&World,lines:&[ChatEntry],target:Option<&killer_ui::target::TargetView>,views:&crate::ui2d::WorldViews) {
        self.windows.clear();
        self.overlays.clear();
        self.minimized = killer_ui::draw::DrawList { capture_text: true, ..Default::default() };
        if ui.shortcut.modern.is_none() || ui.keys.hidden {return;}
        for window in ui.desktop.windows.iter().filter(|window|window.visible && crate::ui_modern_windows::supports(window.id)) {
            let mut list=killer_ui::draw::DrawList {capture_text:true,..Default::default()};
            ui.paint_window(&mut list,window,world,lines,target,views);
            self.windows.insert(window.id,list);
        }
        ui.keys.minimized.paint(&mut self.minimized, &ui.small, ui.desktop.size.0);
        if ui.chat.filter_open && let Some(window)=ui.desktop.window(WindowId::CHAT).filter(|w|w.visible) {
            let mut list=killer_ui::draw::DrawList {capture_text:true,..Default::default()};
            ui.chat.paint_filter(&mut list,window.rect.x,window.rect.y,&ui.small);
            let (x,y)=ui.chat.filter_origin();
            let (w,h)=chat::FILTER_PANEL_SIZE;
            self.overlays.push((NativeRect::new(window.rect.x+x,window.rect.y+y,w,h),list));
        }
        if let Some(tooltip)=&ui.keys.tooltip && !ui.window_under(ui.cursor.0,ui.cursor.1).is_some_and(|(id,_,_)|id==WindowId::SHORTCUT) {
            let size=tooltip.size(&ui.small);
            let (x,y)=killer_ui::tooltip::place(ui.cursor,size,ui.desktop.size);
            let mut list=killer_ui::draw::DrawList {capture_text:true,..Default::default()};
            tooltip.paint(&mut list,x,y,&ui.small);
            self.overlays.push((NativeRect::new(x,y,size.0,size.1),list));
        }
    }

    pub fn prepare_icons(&mut self, ctx: &Context, ui: &GameUi, client: &mut Client) {
        let Some(bars) = ui.shortcut.modern else { return };
        for name in Self::menu_icons().into_iter().chain(killer_ui::sysmenu::SysMenuButton::ALL.map(|button|button.textures().0)) {
            if self.icons.contains_key(&name) {continue;}
            let texture=client.catalog().texture(&name).map(|image|ctx.load_texture(&name,egui::ColorImage::from_rgba_unmultiplied([image.width as usize,image.height as usize],&image.rgba),TextureOptions::LINEAR));
            self.icons.insert(name,texture);
        }
        for row in 0..bars.count {
            for column in 0..SLOTS {
                let index = bars.page(row, ui.shortcut.page) as usize * SLOTS + column;
                let Some(slot) = ui.shortcut.slot(index) else { continue };
                let name = slot.icon();
                if self.icons.contains_key(name) { continue; }
                let texture = client.catalog().texture(name).map(|image| {
                    let image = egui::ColorImage::from_rgba_unmultiplied([image.width as usize,image.height as usize], &image.rgba);
                    ctx.load_texture(name, image, TextureOptions::LINEAR)
                });
                self.icons.insert(name.to_owned(), texture);
            }
        }
        for name in ui.shortcut.auto.potions.iter().map(|p| p.icon.as_str()).chain(ui.shortcut.auto.shots.iter().map(|s| s.icon.as_str())) {
            if name.is_empty() || self.icons.contains_key(name) { continue; }
            let texture = client.catalog().texture(name).map(|image| {
                let image = egui::ColorImage::from_rgba_unmultiplied([image.width as usize,image.height as usize], &image.rgba);
                ctx.load_texture(name, image, TextureOptions::LINEAR)
            });
            self.icons.insert(name.to_owned(), texture);
        }
        for list in self.windows.values().chain(self.overlays.iter().map(|(_,list)|list)).chain(std::iter::once(&self.minimized)) {
            for quad in &list.quads {
                let name=quad.texture.as_str();
                if self.icons.contains_key(name) || crate::ui_modern_windows::surface(name)!=crate::ui_modern_windows::Surface::Image {continue;}
                let texture=client.catalog().texture(name).map(|image| {
                    let image=egui::ColorImage::from_rgba_unmultiplied([image.width as usize,image.height as usize],&image.rgba);
                    ctx.load_texture(name,image,TextureOptions::LINEAR)
                });
                self.icons.insert(name.to_owned(),texture);
            }
        }
    }

    fn menu_icons() -> [String;6] {
        let icons=killer_ui::menu::BUTTON_TEXTURES;
        [icons[0].into(),"icon.skill0001".into(),icons[1].into(),icons[2].into(),icons[2].into(),icons[3].into()]
    }

    fn menu(&self,p:&Painter,bounds:Rect,open:bool,system:bool,scale:f32) {
        let button=Rect::from_min_size(Pos2::new(bounds.right()-112.0*scale,bounds.bottom()-36.0*scale),Vec2::new(112.0,36.0)*scale);
        let active=open || p.ctx().pointer_hover_pos().is_some_and(|pos|button.contains(pos));
        let accent=if active {GOLD}else{TEXT};
        theme::button(p,button,scale,active);
        if open {
            let panel=Rect::from_min_max(bounds.min,Pos2::new(bounds.right(),button.top()-4.0*scale));
            theme::plate(p,panel,scale,false);
            let icons=Self::menu_icons();
            let mut rows:Vec<_>=[("Personagem","Alt+T"),("Skills","Alt+K"),("Inventário","Alt+V"),("Mapa","Alt+M"),("Minimapa",""),("Sistema","")].into_iter().enumerate().map(|(index,(name,key))|(name,key,icons[index].clone(),4.0+index as f32*32.0,32.0,false)).collect();
            if system {
                for (index,name) in ["Comunidade","Macros","Ajuda","Petição","Opções","Trocar personagem"].into_iter().enumerate() {
                    rows.push((name,"",killer_ui::sysmenu::SysMenuButton::ALL[index].textures().0,196.0+index as f32*28.0,28.0,true));
                }
            }
            rows.push(("Sair do jogo","",killer_ui::sysmenu::SysMenuButton::ExitGame.textures().0,196.0+if system {168.0}else{0.0},32.0,false));
            for (name,key,icon,y,height,nested) in rows {
                let row=Rect::from_min_size(bounds.min+Vec2::new(4.0,y)*scale,Vec2::new(216.0,height)*scale);
                if nested {p.rect_filled(row,0.0,Color32::from_rgba_unmultiplied(15,10,7,150));}
                if name=="Sair do jogo" {p.line_segment([row.left_top()+Vec2::new(8.0,0.0)*scale,row.right_top()-Vec2::new(8.0,0.0)*scale],Stroke::new(scale,theme::BRONZE_DIM));}
                if p.ctx().pointer_hover_pos().is_some_and(|point|row.contains(point)) {p.rect_filled(row,4.0*scale,Color32::from_rgba_unmultiplied(226,188,108,34));p.rect_stroke(row,4.0*scale,Stroke::new(scale,theme::BRONZE),StrokeKind::Inside);}
                let indent=if nested {18.0}else{8.0};
                let image=Rect::from_center_size(row.left_center()+Vec2::new(indent+10.0,0.0)*scale,Vec2::splat(20.0*scale));
                if let Some(Some(texture))=self.icons.get(&icon) {
                    let size=texture.size();
                    let texels=if nested || name=="Sair do jogo" {killer_ui::sysmenu::BUTTON_SIZE}else{killer_ui::menu::BUTTON_SIZE} as f32;
                    let uv=Pos2::new((texels/size[0] as f32).min(1.0),(texels/size[1] as f32).min(1.0));
                    p.image(texture.id(),image,Rect::from_min_max(Pos2::ZERO,uv),Color32::WHITE);
                }
                if name=="Sistema" {menu_chevron(p,row.right_center()-Vec2::new(12.0,0.0)*scale,system,scale);}
                p.text(row.left_center()+Vec2::new(indent+28.0,0.0)*scale,Align2::LEFT_CENTER,name,FontId::proportional(12.0*scale),TEXT);
                p.text(row.right_center()-Vec2::new(8.0,0.0)*scale,Align2::RIGHT_CENTER,key,FontId::proportional(10.0*scale),MUTED);
            }
        }
        let icon=button.left_center()+Vec2::new(19.0,0.0)*scale;
        for (x,y) in [(-5.0,-5.0),(1.0,-5.0),(-5.0,1.0),(1.0,1.0)] {
            p.rect_filled(Rect::from_min_size(icon+Vec2::new(x,y)*scale,Vec2::splat(4.0*scale)),scale,accent);
        }
        p.text(button.center()+Vec2::new(4.0,0.0)*scale,Align2::CENTER_CENTER,"Menu",FontId::proportional(12.0*scale),accent);
        menu_chevron(p,button.right_center()-Vec2::new(20.0,0.0)*scale,open,scale);
    }

    pub fn prepare_combat(&mut self, ctx: &Context, world: &World, tables: Option<&killer_assets::client::DatTables>, client: &mut Client) {
        self.cast_visual=None;
        if !self.preferences.enabled {return;}
        let Some(cast)=world.me.as_ref().and_then(|me|me.casting.as_ref()) else {return;};
        let name=tables.and_then(|tables|tables.skillname.get(&(cast.skill_id as u32,cast.skill_level as u32)))
            .map_or_else(||format!("Skill {}",cast.skill_id),|skill|skill.name.clone());
        let icon=crate::ui2d::skill_icon(tables,cast.skill_id,cast.skill_level).unwrap_or_default();
        if !icon.is_empty() && !self.icons.contains_key(&icon) {
            let texture=client.catalog().texture(&icon).map(|image| {
                let image=egui::ColorImage::from_rgba_unmultiplied([image.width as usize,image.height as usize],&image.rgba);
                ctx.load_texture(&icon,image,TextureOptions::LINEAR)
            });
            self.icons.insert(icon.clone(),texture);
        }
        self.cast_visual=Some((name,icon));
    }

    pub fn upload_names(&self, ctx: &Context, renderer: &mut killer_render::renderer::Renderer, painter: &egui_wgpu::Renderer) {
        self.tessellate_names(ctx);
        let mesh=self.name_mesh.borrow();
        if mesh.indices.is_empty() {renderer.clear_world_text();return;}
        if let Some(texture)=painter.texture(&egui::TextureId::Managed(0)).and_then(|texture|texture.texture.as_ref()) {
            renderer.update_world_text(&mesh,&texture.create_view(&Default::default()));
        } else {renderer.clear_world_text();}
    }

    fn tessellate_names(&self, ctx: &Context) {
        let mut mesh=self.name_mesh.borrow_mut();
        mesh.vertices.clear();mesh.indices.clear();mesh.clips.clear();
        let atlas=ctx.fonts(|fonts|fonts.font_image_size());
        for (depth,shapes) in self.name_shapes.borrow().iter() {
            let ppp=ctx.pixels_per_point();
            let viewport=ctx.content_rect().max.to_vec2()*ppp;
            for primitive in ctx.tessellate(shapes.clone(),ppp) {
                let egui::epaint::Primitive::Mesh(text)=primitive.primitive else {continue;};
                let base=mesh.vertices.len() as u32;
                let start=mesh.indices.len() as u32;
                mesh.vertices.extend(text.vertices.iter().map(|vertex| {
                    let rgba=vertex.color.to_array();
                    killer_render::ui::UiVertex {position:[vertex.pos.x*ppp,vertex.pos.y*ppp,*depth],uv:if vertex.uv==egui::epaint::WHITE_UV {[0.5/atlas[0] as f32,0.5/atlas[1] as f32]}else{[vertex.uv.x,vertex.uv.y]},color:rgba.map(|c|f32::from(c)/255.0)}
                }));
                mesh.indices.extend(text.indices.iter().map(|index|base+index));
                let end=mesh.indices.len() as u32;
                let min=(primitive.clip_rect.min.to_vec2()*ppp).clamp(Vec2::ZERO,viewport).round();
                let max=(primitive.clip_rect.max.to_vec2()*ppp).clamp(min,viewport).round();
                mesh.clips.push((start..end,[min.x as u32,min.y as u32,(max.x-min.x) as u32,(max.y-min.y) as u32]));
            }
        }
    }

    pub fn measure_labels(ctx: &Context, ui: &GameUi, labels: &mut [crate::characters::Label], scale: f32) {
        if ui.shortcut.modern.is_none() {return;}
        let scale=scale/ctx.pixels_per_point();
        for label in labels {
            label.modern_text_size=Some([(&label.name,14.0),(&label.title,13.0)].map(|(text,size)| {
                let galley=ctx.fonts_mut(|fonts|fonts.layout_job(label_job(text,size,Color32::WHITE,scale)));
                [galley.size().x/scale,galley.size().y/scale]
            }));
        }
    }

    pub fn paint_combat(&self, ctx: &Context, ui: &GameUi, world: &World, labels: &[crate::characters::Label], scale: f32) {
        self.name_shapes.borrow_mut().clear();
        if ui.shortcut.modern.is_none() || ui.keys.hidden {return;}
        let scale=scale/ctx.pixels_per_point();
        let names=ctx.layer_painter(LayerId::new(Order::Background,Id::new("modern-names")));
        let mut name_clips=vec![ctx.content_rect()];
        for window in ui.desktop.windows.iter().filter(|window|window.visible) {
            let cover=rect(window.rect,scale);
            name_clips=name_clips.into_iter().flat_map(|clip|subtract(clip,cover)).collect();
        }
        let mut speech_tops=HashMap::new();
        for label in labels {
            let center=Pos2::new(label.screen[0],label.screen[1])*scale;
            let galley=names.layout_job(label_job(&label.name,14.0,Color32::from_rgb(label.name_color[0],label.name_color[1],label.name_color[2]),scale));
            let title=names.layout_job(label_job(&label.title,13.0,Color32::from_rgb(label.title_color[0],label.title_color[1],label.title_color[2]),scale));
            let name_y=center.y-crate::ui2d::ENGINE_NAME_RISE as f32*scale;
            let position=Pos2::new(center.x+(label.crests_width() as f32*scale-galley.size().x)*0.5,name_y);
            let title_position=Pos2::new(center.x-title.size().x*0.5,name_y-title.size().y-2.0*scale);
            if label.name_visible && !label.name.is_empty() {
                let extent=Rect::from_min_size(position,galley.size()).union(Rect::from_min_size(title_position,title.size())).expand(scale);
                for clip in name_clips.iter().filter(|clip|clip.intersects(extent)) {
                    let names=names.with_clip_rect(*clip);
                    outlined_label(&names,position,galley.clone(),scale);
                    if !label.title.is_empty() {outlined_label(&names,title_position,title.clone(),scale);}
                }
            }
            if let Some(speech)=&label.speech {
                let text=names.layout_job(label_job(&speech.text,13.0,TEXT,scale));
                let bottom=(speech.screen[1]*scale).min(if label.title.is_empty(){name_y}else{title_position.y}-8.0*scale);
                let bounds=Rect::from_min_size(Pos2::new(speech.screen[0]*scale-text.size().x*0.5-7.0*scale,bottom-text.size().y-8.0*scale),text.size()+Vec2::new(14.0,8.0)*scale);
                speech_tops.insert(label.object_id,bounds.top());
                for clip in name_clips.iter().filter(|clip|clip.intersects(bounds.expand(5.0*scale))) {
                    let names=names.with_clip_rect(*clip);
                    names.rect_filled(bounds,4.0*scale,Color32::from_rgba_unmultiplied(14,18,22,232));
                    names.add(egui::Shape::convex_polygon(vec![Pos2::new(bounds.center().x-4.0*scale,bounds.bottom()),Pos2::new(bounds.center().x+4.0*scale,bounds.bottom()),Pos2::new(bounds.center().x,bounds.bottom()+5.0*scale)],Color32::from_rgba_unmultiplied(14,18,22,232),Stroke::NONE));
                    names.galley(bounds.min+Vec2::new(7.0,4.0)*scale,text.clone(),TEXT);
                }
            }
            let shapes=ctx.graphics_mut(|graphics| {
                let list=graphics.entry(names.layer_id());
                let shapes=list.all_entries().cloned().collect();
                *list=Default::default();
                shapes
            });
            self.name_shapes.borrow_mut().push((label.depth,shapes));
        }
        let Some(me)=world.me.as_ref() else {return;};
        let now=killer_platform::time::Instant::now();
        let painter=ctx.layer_painter(LayerId::new(Order::Background,Id::new("modern-combat")));
        let viewport=Vec2::new(ui.desktop.size.0 as f32,ui.desktop.size.1 as f32)*scale;
        let anchor=|id|labels.iter().find(|label|label.object_id==id).map(|label|Pos2::new(label.screen[0],label.screen[1])*scale);
        let cast=me.casting.as_ref().filter(|cast|cast.hit_time>0 && now.saturating_duration_since(cast.started).as_millis()<(cast.hit_time as u128));
        let gauge=world.gauge.as_ref().filter(|gauge|gauge.remaining(now)>0);
        if (cast.is_some() || gauge.is_some())
            && let Some(label)=labels.iter().find(|label|label.object_id==me.object_id) {
            let (remaining,total)=cast.map(|cast|((cast.hit_time as f32-now.saturating_duration_since(cast.started).as_secs_f32()*1000.0).max(0.0),cast.hit_time as f32))
                .unwrap_or_else(||gauge.map_or((0.0,1.0),|gauge|(gauge.remaining(now) as f32,gauge.setup.time as f32)));
            let center=Pos2::new(label.screen[0],label.screen[1])*scale;
            let rise=if label.title.is_empty() {crate::ui2d::ENGINE_NAME_RISE as f32} else {label.modern_text_size.map_or(ui.large.line_height as f32*2.0,|size|crate::ui2d::ENGINE_NAME_RISE as f32+size[1][1]+2.0)};
            let width=176.0*scale;
            let position=Pos2::new((center.x-width*0.5).clamp(0.0,(viewport.x-width).max(0.0)),(center.y-(rise+15.0)*scale).max(24.0*scale));
            let track=Rect::from_min_size(position,Vec2::new(width,6.0*scale));
            painter.rect_filled(track.expand(scale),3.0*scale,Color32::from_black_alpha(150));
            painter.rect_filled(track,2.0*scale,Color32::from_rgba_unmultiplied(20,57,82,170));
            let progress=(1.0-remaining/total.max(1.0)).clamp(0.0,1.0);
            let fill=Rect::from_min_size(track.min,Vec2::new(track.width()*progress,track.height()));
            if fill.width()>0.0 {
                painter.rect_filled(fill.expand(2.0*scale),4.0*scale,Color32::from_rgba_unmultiplied(47,167,255,35));
                painter.rect_filled(fill,2.0*scale,Color32::from_rgb(44,172,250));
                painter.line_segment([fill.left_top()+Vec2::new(0.0,scale),fill.right_top()+Vec2::new(0.0,scale)],Stroke::new(scale,Color32::from_rgb(167,238,255)));
            }
            if let Some((name,icon))=cast.and(self.cast_visual.as_ref()) {
                let texture=self.icons.get(icon).and_then(Option::as_ref);
                let icon_width=if texture.is_some() {22.0*scale} else {0.0};
                let mut job=egui::text::LayoutJob::simple_singleline(name.clone(),FontId::proportional(12.0*scale),TEXT);
                job.wrap.max_width=width-icon_width;
                job.wrap.max_rows=1;
                let galley=painter.layout_job(job);
                let left=track.center().x-(galley.size().x+icon_width)*0.5;
                let top=track.top()-23.0*scale;
                if let Some(texture)=texture {
                    let icon_rect=Rect::from_min_size(Pos2::new(left,top),Vec2::splat(18.0*scale));
                    painter.image(texture.id(),icon_rect,Rect::from_min_max(Pos2::ZERO,Pos2::new(1.0,1.0)),Color32::WHITE);
                }
                let text_position=Pos2::new(left+icon_width,top+2.0*scale);
                painter.galley_with_override_text_color(text_position+Vec2::splat(scale),galley.clone(),Color32::from_black_alpha(230));
                painter.galley(text_position,galley,TEXT);
            }
        }
        let mut stacks=HashMap::<i32,usize>::new();
        for event in world.combat.hits.iter().rev().filter(|event|event.attacker_id==me.object_id || event.hit.target_id==me.object_id).take(32) {
            let age=now.saturating_duration_since(event.at).as_secs_f32();
            if age>=1.4 {continue;}
            let Some(mut base)=anchor(event.hit.target_id) else {continue;};
            if let Some(top)=speech_tops.get(&event.hit.target_id) {base.y=base.y.min(top+29.0*scale);}
            let stack=stacks.entry(event.hit.target_id).or_default();
            if *stack>=5 {continue;}
            let incoming=event.hit.target_id==me.object_id;
            let critical=event.hit.critical();
            let text=if event.hit.miss() {"ERROU".to_owned()} else if critical {format!("{} !",event.hit.damage)} else {event.hit.damage.to_string()};
            let color=if incoming {Color32::from_rgb(255,112,116)} else if critical {Color32::from_rgb(255,210,110)} else {Color32::from_rgb(240,245,255)};
            let alpha=((1.4-age)/0.45).clamp(0.0,1.0);
            let position=base+Vec2::new(if incoming {-116.0} else {48.0},-35.0-age*35.0-*stack as f32*25.0)*scale;
            let font=FontId::proportional((if critical {27.0} else {22.0})*scale);
            painter.text(position+Vec2::splat(scale),Align2::CENTER_BOTTOM,&text,font.clone(),Color32::from_black_alpha((210.0*alpha) as u8));
            painter.text(position,Align2::CENTER_BOTTOM,text,font,color.gamma_multiply(alpha));
            *stack+=1;
        }
    }

    pub fn paint(&self, ctx: &Context, ui: &GameUi, world: &World, lines: &[ChatEntry], scale: f32) {
        if ui.shortcut.modern.is_none() || ui.keys.hidden { return; }
        let painter = ctx.layer_painter(LayerId::new(Order::Background, Id::new("modern-hud")));
        let scale = scale / ctx.pixels_per_point();
        for (index, window) in ui.desktop.windows.iter().enumerate() {
            if window.id == WindowId::PARTY_MATCH {
                crate::ui_modern_windows::paint_content(&painter.with_clip_rect(ctx.content_rect()), &self.minimized, &self.icons, scale);
            }
            if !window.visible { continue; }
            if !matches!(window.id, WindowId::STATUS | WindowId::CHAT | WindowId::SYSTEM_MSG | WindowId::SHORTCUT) && !crate::ui_modern_windows::supports(window.id) { continue; }
            let bounds = rect(window.rect, scale);
            let extent=self.windows.get(&window.id).map_or(bounds,|list|crate::ui_modern_windows::extent(bounds,list,scale));
            let mut clips = vec![extent];
            for other in ui.desktop.windows.iter().skip(index+1).filter(|other| other.visible) {
                let cover = rect(other.rect, scale);
                clips = clips.into_iter().flat_map(|clip| subtract(clip, cover)).collect();
            }
            if ui.chat.filter_open && let Some(chat_window)=ui.desktop.window(WindowId::CHAT).filter(|window|window.visible) {
                let (x,y)=ui.chat.filter_origin();
                let (w,h)=chat::FILTER_PANEL_SIZE;
                let cover=rect(NativeRect::new(chat_window.rect.x+x,chat_window.rect.y+y,w,h),scale);
                clips=clips.into_iter().flat_map(|clip|subtract(clip,cover)).collect();
            }
            if let Some(position)=ctx.pointer_hover_pos() {
                let cursor=((position.x/scale) as i32,(position.y/scale) as i32);
                if let Some(tooltip)=&ui.keys.tooltip
                    && !ui.window_under(cursor.0,cursor.1).is_some_and(|(id,_,_)|id==WindowId::SHORTCUT) {
                    let size=tooltip.size(&ui.small);
                    let (x,y)=killer_ui::tooltip::place(cursor,size,ui.desktop.size);
                    let cover=rect(NativeRect::new(x,y,size.0,size.1),scale);
                    clips=clips.into_iter().flat_map(|clip|subtract(clip,cover)).collect();
                }

            }
            for clip in clips {
                let painter = painter.with_clip_rect(clip);
                match window.id {
                    WindowId::STATUS => if ui.status_visible && let Some(me) = &world.me {
                        self.status(&painter,bounds,&status_view(me),scale);
                    },
                    WindowId::CHAT => self.chat(&painter,bounds,ui,lines,scale),
                    WindowId::SYSTEM_MSG => self.system_chat(&painter,bounds,ui,lines,scale),
                    WindowId::SHORTCUT => self.shortcuts(&painter,bounds,ui,scale),
                    WindowId::MENU => self.menu(&painter,bounds,ui.modern_menu_open,ui.modern_system_open,scale),
                    WindowId::ABNORMAL_STATUS => crate::ui_modern_buffs::paint(&painter,bounds,&ui.keys.abnormal,&self.icons,scale),
                    id => if let Some(list)=self.windows.get(&id) {crate::ui_modern_windows::paint(&painter,bounds,list,&self.icons,scale,id);}
                }
            }
        }
        for (bounds,list) in &self.overlays {
            let bounds=rect(*bounds,scale);
            crate::ui_modern_windows::paint(&painter.with_clip_rect(ctx.content_rect()),bounds,list,&self.icons,scale,WindowId::CHAT);
        }
        if let Some(position) = ctx.pointer_hover_pos() {
            let hit=ui.window_under((position.x/scale) as i32,(position.y/scale) as i32);
            if let Some((WindowId::SHORTCUT,x,y))=hit && let Some(bars)=ui.shortcut.modern {
                let name=bars.button_at(x,y).map(|button| match button {
                    ModernBarButton::Expand => "Adicionar barra (até 5)",
                    ModernBarButton::Collapse => "Recolher barra",
                    ModernBarButton::Lock => if bars.locked {"Destravar atalhos"}else{"Travar atalhos"},
                    ModernBarButton::Next(_) => "Próxima página",
                    ModernBarButton::Prev(_) => "Página anterior",
                }).or_else(||ui.keys.tooltip.as_ref().map(|tooltip|tooltip.name.as_str()));
                if let Some(name)=name.filter(|name|!name.is_empty()) {
                    let galley=painter.layout_no_wrap(name.to_owned(),FontId::proportional(13.0*scale),TEXT);
                    let size=galley.size()+Vec2::new(20.0,14.0)*scale;
                    let area=ctx.content_rect();
                    let pos=Pos2::new((position.x+14.0).min(area.max.x-size.x).max(0.0),(position.y-size.y-10.0).max(0.0));
                    let painter=ctx.layer_painter(LayerId::new(Order::Tooltip,Id::new("modern-name")));
                    panel(&painter,Rect::from_min_size(pos,size));
                    painter.galley(pos+Vec2::new(10.0,7.0)*scale,galley,TEXT);
                }
            }
        }
    }

    fn status(&self, painter: &Painter, bounds: Rect, view: &killer_ui::status::StatusView, scale: f32) {
        let origin=bounds.min;
        let point=|x:f32,y:f32|origin+Vec2::new(x,y)*scale;
        let plate=Rect::from_min_max(point(0.0,0.0),point(252.0,72.0));
        theme::plate(painter,plate,scale,false);
        let exp=view.exp_percent().clamp(0.0,100.0);
        let center=point(36.0,36.0);
        painter.circle_filled(center,28.0*scale,Color32::from_black_alpha(70));
        painter.circle_filled(center,26.0*scale,Color32::from_rgb(18,17,20));
        painter.circle_filled(center,21.0*scale,Color32::from_rgb(32,28,24));
        painter.circle_stroke(center,26.0*scale,Stroke::new(2.0*scale,theme::GOLD));
        painter.circle_stroke(center,27.5*scale,Stroke::new(scale,Color32::from_white_alpha(46)));
        painter.circle_stroke(center,20.5*scale,Stroke::new(scale,Color32::from_black_alpha(160)));
        if exp>0.0 {
            for (width,color) in [(7.0,Color32::from_rgba_unmultiplied(158,206,120,48)),(3.0,theme::XP.0)] {
                let points=(0..=72).map(|index| {
                    let angle=-std::f32::consts::FRAC_PI_2+std::f32::consts::TAU*(exp/100.0)*(index as f32/72.0);
                    center+Vec2::new(angle.cos(),angle.sin())*(23.5*scale)
                }).collect();
                painter.add(egui::Shape::line(points,Stroke::new(width*scale,color)));
            }
        }
        text(painter,center+Vec2::new(0.0,-9.0)*scale,Align2::CENTER_CENTER,"LV",8.0*scale,MUTED);
        text(painter,center+Vec2::new(0.0,4.0)*scale,Align2::CENTER_CENTER,&view.level.to_string(),17.0*scale,theme::GOLD_HI);
        let name_painter=painter.with_clip_rect(painter.clip_rect().intersect(Rect::from_min_max(point(70.0,4.0),point(180.0,24.0))));
        text(&name_painter,point(70.0,6.0),Align2::LEFT_TOP,&view.name,13.0*scale,TEXT);
        text(painter,point(240.0,8.0),Align2::RIGHT_TOP,&format!("{exp:.2}%"),10.0*scale,MUTED);
        for (index,(label,value,colors)) in [("CP",view.cp,theme::CP),("HP",view.hp,theme::HP),("MP",view.mp,theme::MP)].into_iter().enumerate() {
            let y=26.0+index as f32*15.0;
            let track=Rect::from_min_max(point(70.0,y),point(242.0,y+12.0));
            let ratio=if value.1>0 {(value.0 as f32/value.1 as f32).clamp(0.0,1.0)}else{0.0};
            theme::gauge(painter,track,ratio,colors);
            theme::outlined(painter,point(78.0,y+6.0),Align2::LEFT_CENTER,label,9.0*scale,theme::GOLD_HI,scale);
            let value=format!("{} / {}",value.0,value.1);
            let at=point(235.0,y+6.0);
            theme::outlined(painter,at,Align2::RIGHT_CENTER,&value,10.0*scale,Color32::WHITE,scale);
        }
    }

    fn system_chat(&self,painter:&Painter,bounds:Rect,ui:&GameUi,_lines:&[ChatEntry],scale:f32) {
        chat_panel(painter,Rect::from_min_max(bounds.min,bounds.max-Vec2::new(0.0,4.0)*scale),scale);
        text(painter,bounds.min+Vec2::new(14.0,8.0)*scale,Align2::LEFT_TOP,"Sistema",12.0*scale,MUTED);
        let rows=modern_message_rows(painter,ui.system_msg.list.iter(),bounds.width()-28.0*scale,scale);
        let mut counts=ui.modern_chat_rows.get();counts[1]=rows.len();ui.modern_chat_rows.set(counts);
        let range=chat_range(rows.len(),ui.system_msg.scroll,killer_ui::sysmsg::SYSTEM_MSG_SIZE.1,true);
        if rows.is_empty() {
            text(painter,bounds.min+Vec2::new(12.0,32.0)*scale,Align2::LEFT_TOP,"Mensagens do sistema aparecerão aqui.",12.0*scale,MUTED);
        }
        let messages=painter.with_clip_rect(painter.clip_rect().intersect(bounds.shrink(8.0*scale)));
        for (index,(indent,line,color)) in rows[range].iter().enumerate() {
            let color=Color32::from_rgb((color.0>>16) as u8,(color.0>>8) as u8,color.0 as u8);
            text(&messages,bounds.min+Vec2::new(12.0+*indent as f32,30.0+index as f32*chat::CHAT_LINE as f32)*scale,Align2::LEFT_TOP,line,12.0*scale,color);
        }
    }

    fn chat(&self, painter: &Painter, bounds: Rect, ui: &GameUi, _lines: &[ChatEntry], scale: f32) {
        chat_panel(painter,bounds,scale);
        let origin = bounds.min;
        painter.text(origin+Vec2::new(14.0,9.0)*scale,Align2::LEFT_TOP,"Conversas",FontId::proportional(13.0*scale),theme::GOLD_HI);
        for dot in [-6.0,0.0,6.0] {painter.circle_filled(origin+Vec2::new(bounds.width()/scale*0.5+dot,4.0)*scale,1.2*scale,Color32::from_white_alpha(90));}
        let rows=modern_message_rows(painter,ui.chat.active_list().iter(),bounds.width()-28.0*scale,scale);
        let mut counts=ui.modern_chat_rows.get();counts[0]=rows.len();ui.modern_chat_rows.set(counts);
        let range=chat_range(rows.len(),ui.chat.scroll,ui.chat.height,false);
        if rows.is_empty() {
            text(painter,origin+Vec2::new(14.0,75.0)*scale,Align2::LEFT_TOP,"Nenhuma mensagem por aqui.",12.0*scale,MUTED);
        }
        let clip = Rect::from_min_max(origin+Vec2::new(14.0,68.0)*scale,Pos2::new(bounds.right()-14.0*scale,bounds.bottom()-48.0*scale));
        let messages = painter.with_clip_rect(painter.clip_rect().intersect(clip));
        for (index,(indent,line,color)) in rows[range].iter().enumerate() {
            let color = Color32::from_rgb((color.0>>16) as u8,(color.0>>8) as u8,color.0 as u8);
            text(&messages,origin+Vec2::new(14.0+*indent as f32,68.0+index as f32*chat::CHAT_LINE as f32)*scale,Align2::LEFT_TOP,line,12.0*scale,color);
        }
        let strip=Rect::from_min_max(origin+Vec2::new(14.0,33.0)*scale,Pos2::new(bounds.right()-14.0*scale,origin.y+61.0*scale));
        painter.rect_filled(strip,6.0*scale,Color32::from_black_alpha(90));
        painter.rect_stroke(strip,6.0*scale,Stroke::new(scale,Color32::from_white_alpha(22)),StrokeKind::Inside);
        let tab_width=strip.width()/5.0;
        for (index,label) in ["Geral","Comércio","Grupo","Clã","Aliança"].into_iter().enumerate() {
            let selected=index==ui.chat.active_tab;
            let tab=Rect::from_min_size(strip.min+Vec2::new(index as f32*tab_width,0.0),Vec2::new(tab_width,strip.height()));
            let hovered=painter.ctx().pointer_hover_pos().is_some_and(|position|tab.contains(position));
            let surface=tab.shrink(3.0*scale);
            if selected {
                painter.rect_filled(surface,4.0*scale,Color32::from_rgba_unmultiplied(214,178,106,46));
                painter.rect_stroke(surface,4.0*scale,Stroke::new(scale,Color32::from_rgba_unmultiplied(214,178,106,150)),StrokeKind::Inside);
            }else if hovered {painter.rect_filled(surface,4.0*scale,Color32::from_white_alpha(18));}
            painter.text(tab.center(),Align2::CENTER_CENTER,label,FontId::proportional(11.5*scale),if selected {theme::GOLD_HI}else{Color32::from_rgb(176,180,192)});
        }
        let input = Rect::from_min_max(origin+Vec2::new(14.0,(ui.chat.height-44) as f32)*scale,bounds.max-Vec2::new(14.0,8.0)*scale);
        let footer=Rect::from_min_max(Pos2::new(bounds.left()+scale,input.top()-5.0*scale),bounds.max-Vec2::splat(scale));
        painter.line_segment([footer.left_top()+Vec2::new(14.0,0.0)*scale,footer.right_top()-Vec2::new(14.0,0.0)*scale],Stroke::new(scale,theme::BRONZE_DIM));
        let hovered=painter.ctx().pointer_hover_pos().is_some_and(|pos|input.contains(pos));
        theme::inset(painter,input,scale,ui.chat.focused||hovered);
        if ui.chat.focused {
            painter.line_segment([input.left_top()+Vec2::new(1.0,8.0)*scale,input.left_bottom()+Vec2::new(1.0,-8.0)*scale],Stroke::new(2.0*scale,GOLD));
        }
        let value = if ui.chat.input.text.is_empty() && !ui.chat.focused {"Enter para escrever…"}else{&ui.chat.input.text};
        let content=Rect::from_min_max(input.min+Vec2::new(34.0,4.0)*scale,input.max-Vec2::new(10.0,4.0)*scale);
        let input_painter = painter.with_clip_rect(painter.clip_rect().intersect(content));
        let before = painter.layout_no_wrap(ui.chat.input.before_cursor().to_owned(),FontId::proportional(12.0*scale),TEXT);
        let offset=if ui.chat.focused {chat_input_offset(before.size().x,content.width(),scale)}else{0.0};
        text(&input_painter,content.left_center()-Vec2::new(offset,0.0),Align2::LEFT_CENTER,value,12.0*scale,if ui.chat.focused {TEXT}else{MUTED});
        if ui.chat.focused {
            let x=content.left()+before.size().x-offset;
            input_painter.line_segment([Pos2::new(x,content.top()+4.0*scale),Pos2::new(x,content.bottom()-4.0*scale)],Stroke::new(scale,GOLD));
        }
        let option=chat_option_rect(ui.chat.height);
        let center=origin+Vec2::new((option.x+option.w/2) as f32,(option.y+option.h/2) as f32)*scale;
        for (y,x) in [(-5.0,-3.0),(0.0,3.0),(5.0,-1.0)] {
            painter.line_segment([center+Vec2::new(-7.0,y)*scale,center+Vec2::new(7.0,y)*scale],Stroke::new(scale,theme::BRONZE_HI));
            painter.circle_filled(center+Vec2::new(x,y)*scale,2.0*scale,GOLD);
        }
    }

    fn auto_strip(&self, painter: &Painter, origin: Pos2, bars: &ModernShortcuts, ui: &GameUi, scale: f32) {
        let auto = &ui.shortcut.auto;
        let hover = painter.ctx().pointer_hover_pos();
        let pointed = |area: Rect| hover.is_some_and(|position| area.contains(position) && ui.window_under((position.x/scale) as i32,(position.y/scale) as i32).is_some_and(|(id,_,_)|id==WindowId::SHORTCUT));
        let at = |hit: AutoHit| rect(bars.auto_rect(hit),scale).translate(origin.to_vec2());
        let tip = |area: Rect, label: String| {
            if !pointed(area) { return; }
            painter.rect_stroke(area,3.0*scale,Stroke::new(1.5*scale,theme::GOLD_HI),StrokeKind::Inside);
            let galley = painter.layout_no_wrap(label,FontId::proportional(10.5*scale),TEXT);
            let plate = Rect::from_center_size(area.center_top()-Vec2::new(0.0,12.0)*scale,galley.size()+Vec2::new(12.0,5.0)*scale);
            let plate = plate.translate(Vec2::new((origin.x+4.0*scale-plate.min.x).max(0.0),0.0));
            painter.rect_filled(plate,3.0*scale,Color32::from_rgba_unmultiplied(12,9,7,235));
            painter.rect_stroke(plate,3.0*scale,Stroke::new(scale,theme::BRONZE_DIM),StrokeKind::Inside);
            painter.galley(plate.min+Vec2::new(6.0,2.5)*scale,galley,TEXT);
        };
        let plate = Rect::from_min_max(origin,origin+Vec2::new(killer_ui::modern_shortcut::WIDTH as f32,killer_ui::modern_shortcut::AUTO as f32-2.0)*scale);
        theme::plate(painter,plate,scale,pointed(plate));
        let pulse = (painter.ctx().input(|input| input.time) * 5.0).sin() as f32 * 0.5 + 0.5;
        let carried = auto.carried;
        let cell = |area: Rect, name: &str, count: i32, item_id: i32, on: bool, accepts: Option<bool>, flash: u32| {
            painter.rect_filled(area,3.0*scale,Color32::from_rgba_unmultiplied(8,6,5,200));
            if item_id != 0 && let Some(Some(texture)) = self.icons.get(name) {
                let empty = count < 1;
                let tint = if on && !empty {Color32::WHITE}else{Color32::from_gray(if empty {60}else{95})};
                painter.image(texture.id(),area.shrink(1.5*scale),Rect::from_min_max(Pos2::ZERO,Pos2::new(1.0,1.0)),tint);
            }
            let border = if item_id != 0 && count < 1 {Color32::from_rgb(190,64,56)} else if on {GOLD} else {theme::BRONZE_DIM};
            painter.rect_stroke(area,3.0*scale,Stroke::new(if on {1.5}else{1.0}*scale,border),StrokeKind::Inside);
            if flash > 0 {
                let strength = flash as f32 / killer_ui::shortcut::AUTO_FLASH_FRAMES as f32;
                painter.rect_filled(area,3.0*scale,Color32::from_rgba_unmultiplied(255,236,170,(200.0*strength) as u8));
            }
            if item_id != 0 && count > 0 {
                let pos = area.max-Vec2::new(1.5,0.5)*scale;
                text(painter,pos+Vec2::splat(scale),Align2::RIGHT_BOTTOM,&count.to_string(),8.5*scale,Color32::BLACK);
                text(painter,pos,Align2::RIGHT_BOTTOM,&count.to_string(),8.5*scale,TEXT);
            }
            match accepts {
                Some(true) => {
                    painter.rect_filled(area,3.0*scale,Color32::from_rgba_unmultiplied(226,188,108,(30.0+50.0*pulse) as u8));
                    painter.rect_stroke(area.expand(1.0*scale),4.0*scale,Stroke::new(2.0*scale,theme::GOLD_HI),StrokeKind::Outside);
                }
                Some(false) => { painter.rect_filled(area,3.0*scale,Color32::from_black_alpha(150)); }
                None => {}
            }
        };
        let short = |name: &str| {
            let mut chars = name.chars();
            let head: String = chars.by_ref().take(11).collect();
            if chars.next().is_some() { format!("{head}…") } else { head }
        };
        for vital in Vital::ALL {
            let potion = &auto.potions[vital.index()];
            let icon = at(AutoHit::Potion(vital));
            cell(icon,&potion.icon,potion.count,potion.item_id,potion.on,(carried != 0).then_some(true),potion.flash);
            if potion.item_id == 0 {
                text(painter,icon.center(),Align2::CENTER_CENTER,vital.label(),9.0*scale,MUTED);
                tip(icon,format!("Arraste do inventário a poção de {} · botão direito esvazia",vital.label()));
            } else if potion.count < 1 {
                tip(icon,format!("Auto {}: {} · acabou o estoque",vital.label(),potion.name));
            } else {
                tip(icon,format!("Auto {}: {} · {} · clique liga/desliga · botão direito esvazia",vital.label(),potion.name,if potion.on {"ligada"}else{"desligada"}));
            }
            let level = at(AutoHit::Level(vital));
            painter.rect_filled(level,3.0*scale,Color32::from_rgba_unmultiplied(8,6,5,200));
            let tint = match vital {
                Vital::Hp => Color32::from_rgb(176,48,44),
                Vital::Cp => Color32::from_rgb(196,150,52),
                Vital::Mp => Color32::from_rgb(52,104,190),
            };
            let fill = Rect::from_min_size(level.min,Vec2::new(level.width()*potion.percent as f32/100.0,level.height()));
            painter.rect_filled(fill.shrink(1.0*scale),2.0*scale,tint.gamma_multiply(if potion.on {0.75}else{0.28}));
            let edge = level.min.x+level.width()*potion.percent as f32/100.0;
            painter.line_segment([Pos2::new(edge,level.min.y+1.0*scale),Pos2::new(edge,level.max.y-1.0*scale)],Stroke::new(1.5*scale,Color32::WHITE));
            painter.rect_stroke(level,3.0*scale,Stroke::new(scale,theme::BRONZE_DIM),StrokeKind::Inside);
            let color = if potion.on {Color32::WHITE}else{MUTED};
            text(painter,level.center()-Vec2::new(0.0,5.0)*scale,Align2::CENTER_CENTER,&format!("{} {}%",vital.label(),potion.percent),10.0*scale,color);
            let (line,line_color) = if potion.item_id == 0 {("sem poção".to_string(),MUTED)} else if potion.count < 1 {("acabou".to_string(),Color32::from_rgb(230,110,100))} else {(short(&potion.name),color)};
            text(painter,level.center()+Vec2::new(0.0,6.5)*scale,Align2::CENTER_CENTER,&line,7.5*scale,line_color);
            tip(level,format!("Usa {} quando o {} estiver abaixo de {}% · arraste a barra, clique ou use a roda (1% a 99%)",if potion.name.is_empty() {"a poção"}else{&potion.name},vital.label(),potion.percent));
            if auto_dragging(ui) == Some(vital) {
                let galley = painter.layout_no_wrap(format!("{} < {}%",vital.label(),potion.percent),FontId::proportional(15.0*scale),Color32::WHITE);
                let bubble = Rect::from_center_size(level.center_top()-Vec2::new(0.0,22.0)*scale,galley.size()+Vec2::new(18.0,8.0)*scale);
                painter.rect_filled(bubble,5.0*scale,Color32::from_rgba_unmultiplied(12,9,7,240));
                painter.rect_stroke(bubble,5.0*scale,Stroke::new(1.5*scale,tint),StrokeKind::Inside);
                painter.galley(bubble.min+Vec2::new(9.0,4.0)*scale,galley,Color32::WHITE);
            }
        }
        for kind in ShotKind::ALL {
            let shot = &auto.shots[kind.index()];
            let area = at(AutoHit::Shot(kind));
            cell(area,&shot.icon,shot.count,shot.item_id,shot.on,(carried != 0).then(|| kind.accepts(carried)),0);
            tip(area,format!("{}: {} · {}",kind.label(),shot.name,if shot.on {"ligado"}else{"desligado"}));
        }
    }

    fn shortcuts(&self, painter: &Painter, bounds: Rect, ui: &GameUi, scale: f32) {
        let Some(bars) = ui.shortcut.modern else { return };
        let origin=bounds.min;
        let body=Rect::from_min_max(origin+Vec2::new(0.0,killer_ui::modern_shortcut::AUTO as f32+4.0)*scale,origin+Vec2::new(556.0,bars.height() as f32)*scale);
        let engaged=painter.ctx().pointer_hover_pos().is_some_and(|pos|body.contains(pos));
        theme::plate(painter,body,scale,engaged);
        painter.line_segment([body.left_top()+Vec2::new(30.0,12.0)*scale,body.left_bottom()+Vec2::new(30.0,-8.0)*scale],Stroke::new(scale,theme::BRONZE_DIM));
        let controls=Rect::from_min_max(origin+Vec2::new(522.0,bars.row_y(0) as f32)*scale,body.max-Vec2::new(5.0,5.0)*scale);
        painter.rect_filled(controls,4.0*scale,Color32::from_rgba_unmultiplied(15,10,7,120));
        painter.line_segment([controls.left_top(),controls.left_bottom()],Stroke::new(scale,theme::BRONZE_DIM));
        self.auto_strip(painter,origin,&bars,ui,scale);
        for row in 0..bars.count {
            let y=bars.row_y(row);
            if row+1<bars.count {
                painter.line_segment([origin+Vec2::new(38.0,y as f32-2.0)*scale,origin+Vec2::new(508.0,y as f32-2.0)*scale],Stroke::new(scale,Color32::from_rgba_unmultiplied(206,166,92,30)));
            }
            text(painter,origin+Vec2::new(17.0,y as f32+24.0)*scale,Align2::CENTER_CENTER,&format!("{}",bars.page(row,ui.shortcut.page)+1),12.0*scale,theme::GOLD);
            for column in 0..SLOTS {
                let native=bars.slot_rect(row,column);
                let area=rect(native,scale).translate(origin.to_vec2());
                theme::slot_back(painter,area,scale);
                let index=bars.page(row,ui.shortcut.page) as usize*SLOTS+column;
                theme::slot_front(painter,area,scale,false);
                if let Some(slot)=ui.shortcut.slot(index) {
                    if let Some(Some(texture))=self.icons.get(slot.icon()) {
                        painter.image(texture.id(),area.shrink(2.0*scale),Rect::from_min_max(Pos2::ZERO,Pos2::new(1.0,1.0)),if slot.disabled {Color32::from_gray(100)}else{Color32::WHITE});
                        theme::slot_front(painter,area,scale,true);
                    }
                    if let Some(cooldown)=slot.cooldown {
                        painter.rect_filled(Rect::from_min_size(area.min,Vec2::new(area.width(),area.height()*cooldown.clamp(0.0,1.0))),0.0,Color32::from_black_alpha(165));
                    }
                    if slot.toggled || slot.disabled && slot.soulshot { painter.rect_stroke(area,1.0,Stroke::new(2.0*scale,GOLD),StrokeKind::Inside); }
                    if let Some(SlotKind::Macro{..})=slot.kind { text(painter,area.center(),Align2::CENTER_CENTER,&slot.macro_acronym,9.0*scale,TEXT); }
                }
                if row==0 {
                    theme::outlined(painter,area.center_bottom()+Vec2::new(0.0,5.0)*scale,Align2::CENTER_CENTER,&format!("F{}",column+1),9.5*scale,theme::GOLD,scale);
                }
                if let Some(position)=painter.ctx().pointer_hover_pos() && area.contains(position)
                    && ui.window_under((position.x/scale) as i32,(position.y/scale) as i32).is_some_and(|(id,_,_)|id==WindowId::SHORTCUT) {
                    painter.rect_filled(area,2.0,Color32::from_rgba_unmultiplied(226,188,108,40));
                    painter.rect_stroke(area,2.0,Stroke::new(2.0*scale,theme::GOLD_HI),StrokeKind::Inside);
                }
                if let Some(slot)=ui.shortcut.slot(index)
                    && let Some(SlotKind::Item { object_id })=slot.kind
                    && let Some(item)=ui.keys.inventory.items.iter().find(|item|item.object_id==object_id)
                    && item.count>1 {
                    let value=item.count.to_string();
                    let pos=area.max-Vec2::new(2.0,1.0)*scale;
                    text(painter,pos+Vec2::splat(scale),Align2::RIGHT_BOTTOM,&value,10.0*scale,Color32::BLACK);
                    text(painter,pos,Align2::RIGHT_BOTTOM,&value,10.0*scale,TEXT);
                }
            }
            for button in [ModernBarButton::Prev(row),ModernBarButton::Next(row)] {
                let area=rect(bars.button_rect(button),scale).translate(origin.to_vec2());
                control_hover(painter,area,scale);
                chevron(painter,area,matches!(button,ModernBarButton::Prev(_)),TEXT,scale);
            }
        }
        for button in [ModernBarButton::Expand,ModernBarButton::Lock,ModernBarButton::Collapse] {
            let area=rect(bars.button_rect(button),scale).translate(origin.to_vec2());
            control_hover(painter,area,scale);
            if button==ModernBarButton::Lock {
                let center=area.center();
                let color=if bars.locked {GOLD}else{TEXT};
                let body=Rect::from_center_size(center+Vec2::new(0.0,2.0)*scale,Vec2::new(9.0,7.0)*scale);
                painter.rect_stroke(body,1.0,Stroke::new(scale,color),StrokeKind::Inside);
                let top=center+Vec2::new(if bars.locked {0.0}else{2.0},-3.0)*scale;
                painter.line_segment([top+Vec2::new(-3.0,3.0)*scale,top+Vec2::new(-3.0,-2.0)*scale],Stroke::new(scale,color));
                painter.line_segment([top+Vec2::new(-3.0,-2.0)*scale,top+Vec2::new(3.0,-2.0)*scale],Stroke::new(scale,color));
                painter.line_segment([top+Vec2::new(3.0,-2.0)*scale,top+Vec2::new(3.0,3.0)*scale],Stroke::new(scale,color));
            } else {
                let disabled=button==ModernBarButton::Expand && bars.count==5 || button==ModernBarButton::Collapse && bars.count==1;
                chevron(painter,area,button==ModernBarButton::Expand,if disabled {Color32::from_gray(90)}else{TEXT},scale);
            }
        }
        let grip=Rect::from_min_size(origin+Vec2::new(0.0,killer_ui::modern_shortcut::AUTO as f32+4.0)*scale,Vec2::new(518.0,10.0)*scale);
        for x in [-6.0,0.0,6.0] {painter.circle_filled(grip.center()+Vec2::new(x,0.0)*scale,1.2*scale,theme::BRONZE_HI);}
    }
}

fn auto_dragging(ui: &GameUi) -> Option<Vital> { ui.shortcut.auto_dragging() }

pub fn chat_option_rect(height:i32)->NativeRect {NativeRect::new(16,height-40,26,28)}

fn modern_message_rows<'a>(p:&Painter,entries:impl Iterator<Item=&'a ChatEntry>,width:f32,scale:f32)->Vec<(i32,String,killer_ui::color::Argb)> {
    entries.flat_map(|entry| {
        let color=entry.color();
        let galley=p.layout(entry.line(),FontId::proportional(12.0*scale),TEXT,width);
        galley.rows.iter().map(|row|(0,row.glyphs.iter().map(|glyph|glyph.chr).collect(),color)).collect::<Vec<_>>()
    }).collect()
}

fn chat_input_offset(cursor_width:f32,available:f32,scale:f32)->f32 {
    (cursor_width-(available-2.0*scale).max(0.0)).max(0.0)
}

fn control_hover(painter:&Painter,area:Rect,scale:f32) {
    if painter.ctx().pointer_hover_pos().is_some_and(|pos|area.contains(pos)) {
        painter.rect_filled(area.shrink(scale),3.0,Color32::from_rgba_unmultiplied(226,188,108,34));
        painter.rect_stroke(area,3.0,Stroke::new(scale,GOLD),StrokeKind::Inside);
    }
}

pub fn chat_tab_rect(index:usize)->NativeRect { NativeRect::new(14+index as i32*64,32,64,30) }

pub fn chat_tab_at(x:i32,y:i32)->Option<usize> { (0..5).find(|&index|chat_tab_rect(index).contains(x,y)) }

pub fn chat_scroll_limit(rows:usize,height:i32,system:bool)->usize {
    let inset=if system {32}else{116};
    let visible=((height-inset)/chat::CHAT_LINE).max(0) as usize;
    rows.saturating_sub(visible)
}

fn chat_range(rows:usize,scroll:usize,height:i32,system:bool)->std::ops::Range<usize> {
    let max=chat_scroll_limit(rows,height,system);
    let end=rows.saturating_sub(scroll.min(max));
    let visible=rows-max;
    end.saturating_sub(visible)..end
}

fn rect(rect: NativeRect, scale: f32) -> Rect {
    Rect::from_min_size(Pos2::new(rect.x as f32,rect.y as f32)*scale,Vec2::new(rect.w as f32,rect.h as f32)*scale)
}

fn text(painter: &Painter, pos: Pos2, align: Align2, value: &str, size: f32, color: Color32) {
    painter.text(pos,align,value,FontId::proportional(size),color);
}

pub(crate) fn chevron(painter: &Painter, rect: Rect, up: bool, color: Color32, scale: f32) {
    let center=rect.center();
    let sign=if up {-1.0}else{1.0};
    let tip=center+Vec2::new(0.0,sign*2.0)*scale;
    for x in [-3.5,3.5] {
        painter.line_segment([center+Vec2::new(x,-sign*1.5)*scale,tip],Stroke::new(scale,color));
    }
}

pub(crate) fn panel(painter: &Painter, rect: Rect) {
    let focused=painter.ctx().pointer_hover_pos().is_some_and(|pos|rect.contains(pos));
    theme::plate(painter,rect,1.0,focused);
}

fn chat_panel(p:&Painter,bounds:Rect,scale:f32) {
    let focused=p.ctx().pointer_hover_pos().is_some_and(|pos|bounds.contains(pos));
    theme::plate(p,bounds,scale,focused);
}

fn menu_chevron(p:&Painter,center:Pos2,down:bool,scale:f32) {
    let direction=if down {1.0}else{-1.0};
    p.add(egui::Shape::line(vec![center+Vec2::new(-3.0,-direction*1.5)*scale,center+Vec2::new(0.0,direction*1.5)*scale,center+Vec2::new(3.0,-direction*1.5)*scale],Stroke::new(scale,MUTED)));
}

fn subtract(rect: Rect, cover: Rect) -> Vec<Rect> {
    let cut=rect.intersect(cover);
    if !cut.is_positive() { return vec![rect]; }
    [Rect::from_min_max(rect.min,Pos2::new(rect.max.x,cut.min.y)),
     Rect::from_min_max(Pos2::new(rect.min.x,cut.max.y),rect.max),
     Rect::from_min_max(Pos2::new(rect.min.x,cut.min.y),Pos2::new(cut.min.x,cut.max.y)),
     Rect::from_min_max(Pos2::new(cut.max.x,cut.min.y),Pos2::new(rect.max.x,cut.max.y))]
        .into_iter().filter(|r|r.is_positive()).collect()
}

fn label_job(text: &str, size: f32, color: Color32, scale: f32) -> egui::text::LayoutJob {
    let mut job=egui::text::LayoutJob::simple_singleline(text.into(),FontId::proportional(size*scale),color);
    for section in &mut job.sections {section.format.extra_letter_spacing=0.35*scale;}
    job
}

fn outlined_label(painter: &Painter, position: Pos2, galley: std::sync::Arc<egui::Galley>, scale: f32) {
    for offset in [Vec2::new(-1.0,0.0),Vec2::new(1.0,0.0),Vec2::new(0.0,-1.0),Vec2::new(0.0,1.0)] {
        painter.galley_with_override_text_color(position+offset*scale,galley.clone(),Color32::from_black_alpha(180));
    }
    painter.galley(position,galley,TEXT);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn combat_fixture() -> (ModernHud,GameUi,World,Vec<crate::characters::Label>) {
        let now=killer_platform::time::Instant::now();
        let mut ui=crate::ui2d::tests::game_ui((800,450));
        ui.shortcut.modern=Some(Default::default());
        let mut me=crate::ui2d::tests::me();
        me.casting=Some(killer_world::world::Casting {skill_id:1204,skill_level:2,hit_time:4000,target_id:me.object_id,started:now-killer_platform::time::Duration::from_millis(1700)});
        let id=me.object_id;
        let mut world=World {me:Some(me),..Default::default()};
        for (attacker,target,damage,flags,age) in [(id,55,1482,0x20,300),(id,55,386,0,100),(55,id,127,0,200)] {
            world.combat.record(attacker,&[l2_net::game::packets::Hit {target_id:target,damage,flags}],now-killer_platform::time::Duration::from_millis(age));
        }
        let labels=[(id,[310.0,240.0]),(55,[535.0,180.0])].into_iter().map(|(object_id,screen)|crate::characters::Label {
            object_id,screen,depth:0.5,name:String::new(),title:String::new(),store_sign:None,speech:None,modern_text_size:None,name_visible:true,name_color:[255;3],title_color:[255;3],clan_crest:None,ally_crest:None,mark:None,relation_icons:Default::default(),broadcast_icon:None,
        }).collect();
        let hud=ModernHud {preferences:HudPreferences {enabled:true,..Default::default()},cast_visual:Some(("Wind Walk".into(),String::new())),..Default::default()};
        (hud,ui,world,labels)
    }

    fn combat_frame(hud:&ModernHud,ui:&GameUi,world:&World,labels:&[crate::characters::Label],ctx:&Context) -> egui::FullOutput {
        ctx.begin_pass(egui::RawInput {screen_rect:Some(Rect::from_min_size(Pos2::ZERO,Vec2::new(800.0,450.0))),..Default::default()});
        hud.paint_combat(ctx,ui,world,labels,1.0);
        ctx.end_pass()
    }

    fn empty_combat_frame(hud:&ModernHud,ui:&GameUi,world:&World,labels:&[crate::characters::Label],ctx:&Context) -> bool {
        let mut output=combat_frame(hud,ui,world,labels,ctx);
        output.textures_delta.clear();
        output.shapes.is_empty()
    }

    #[test]
    fn modern_names_are_clipped_behind_visible_windows() {
        let (hud,mut ui,world,mut labels)=combat_fixture();
        for window in &mut ui.desktop.windows {window.visible=false;}
        let window=ui.desktop.window_mut(WindowId::MAIN).unwrap();
        window.visible=true;
        window.rect=NativeRect::new(280,215,40,40);
        labels[0].name="Covered name".into();
        let ctx=Context::default();
        let mut output=combat_frame(&hud,&ui,&world,&labels,&ctx);
        output.textures_delta.clear();
        let cover=Rect::from_min_size(Pos2::new(280.0,215.0),Vec2::new(40.0,40.0));
        hud.tessellate_names(&ctx);
        let mesh=hud.name_mesh.borrow();
        assert!(!mesh.indices.is_empty());
        assert!(mesh.vertices.iter().all(|vertex|vertex.position[2]==0.5));
        assert!(mesh.clips.iter().all(|(_,clip)| {
            let [x,y,w,h]=*clip;
            !Rect::from_min_size(Pos2::new(x as f32,y as f32),Vec2::new(w as f32,h as f32)).intersect(cover).is_positive()
        }));
    }

    #[test]
    fn combat_overlay_expires_hides_and_preserves_server_hits() {
        let (hud,mut ui,mut world,labels)=combat_fixture();
        let ctx=Context::default();
        let mut output=combat_frame(&hud,&ui,&world,&labels,&ctx);
        output.textures_delta.clear();
        let text:Vec<_>=output.shapes.iter().filter_map(|shape|match &shape.shape {egui::Shape::Text(text)=>Some(text.galley.text()),_=>None}).collect();
        assert!(text.contains(&"Wind Walk"));
        assert!(text.contains(&"1482 !"));
        assert!(text.contains(&"127"));
        assert_eq!(world.combat.hits.len(),3);
        world.me.as_mut().unwrap().casting=None;
        for hit in &mut world.combat.hits {hit.at-=killer_platform::time::Duration::from_secs(2);}
        assert!(empty_combat_frame(&hud,&ui,&world,&labels,&ctx));
        world.gauge=Some(killer_world::Gauge {setup:l2_net::game::packets::SetupGauge {color:0,time:5000,max_time:5000},started:killer_platform::time::Instant::now()});
        assert!(!empty_combat_frame(&hud,&ui,&world,&labels,&ctx));
        ui.keys.hidden=true;
        assert!(empty_combat_frame(&hud,&ui,&world,&labels,&ctx));
        ui.keys.hidden=false;
        ui.shortcut.modern=None;
        assert!(empty_combat_frame(&hud,&ui,&world,&labels,&ctx));
    }

    #[test]
    #[ignore]
    fn capture_modern_combat_with_real_skill_icon() {
        let (mut hud,ui,mut world,mut labels)=combat_fixture();
        let mut client=Client::open(std::path::Path::new(&std::env::var("KILLER_CLIENT_DIR").unwrap())).unwrap();
        let tables=client.tables().unwrap();
        let ctx=Context::default();
        hud.prepare_combat(&ctx,&world,Some(&tables),&mut client);
        hud.prepare_icons(&ctx,&ui,&mut client);
        for name in ModernHud::menu_icons().into_iter().chain(killer_ui::sysmenu::SysMenuButton::ALL.map(|button|button.textures().0)) {
            assert!(hud.icons.get(&name).is_some_and(Option::is_some),"{name}");
        }
        assert!(hud.cast_visual.as_ref().is_some_and(|(_,icon)|hud.icons.get(icon).is_some_and(Option::is_some)));
        let mut buff_icons=Vec::new();
        let candidates:std::collections::BTreeSet<_>=tables.skillgrp.icons().filter(|name|name.to_ascii_lowercase().starts_with("icon.skill")).map(str::to_owned).collect();
        for name in candidates {
            let Some(image)=client.catalog().texture(&name) else {continue;};
            let texture=ctx.load_texture(&name,egui::ColorImage::from_rgba_unmultiplied([image.width as usize,image.height as usize],&image.rgba),TextureOptions::LINEAR);
            hud.icons.insert(name.clone(),Some(texture));
            buff_icons.push(name);
            if buff_icons.len()==17 {break;}
        }
        assert_eq!(buff_icons.len(),17);
        let now=killer_platform::time::Instant::now();
        world.me.as_mut().unwrap().casting.as_mut().unwrap().started=now-killer_platform::time::Duration::from_millis(1700);
        for hit in &mut world.combat.hits {hit.at=now-killer_platform::time::Duration::from_millis(200);}
        labels[0].name="Wender".into();
        labels[0].title="Lineage II".into();
        labels[1].name="Álvaro Guerreiro".into();
        labels[1].title="Cavaleiro de Aden".into();
        labels[1].speech=Some(killer_ui::store_sign::StoreSign {text:"Vamos para Giran?".into(),color:killer_ui::color::Argb(0xffffffff),background:killer_ui::color::Argb(0xff000000),screen:[labels[1].screen[0],labels[1].screen[1]-40.0]});
        ctx.begin_pass(egui::RawInput {screen_rect:Some(Rect::from_min_size(Pos2::ZERO,Vec2::new(1200.0,720.0))),..Default::default()});
        ModernHud::measure_labels(&ctx,&ui,&mut labels,1.0);
        hud.paint_combat(&ctx,&ui,&world,&labels,1.0);
        let painter=ctx.layer_painter(LayerId::new(Order::Background,Id::new("preview-menu")));
        hud.menu(&painter,Rect::from_min_size(Pos2::new(960.0,268.0),Vec2::new(224.0,436.0)),true,true,1.0);
        let view=killer_ui::abnormal::AbnormalView {buffs:buff_icons.clone().into_iter().enumerate().map(|(index,icon)|killer_ui::abnormal::BuffIcon {icon:Some(icon),remaining:Some(if index==16 {8}else{3600}),debuff:index==16,..Default::default()}).collect(),..Default::default()};
        crate::ui_modern_buffs::paint(&painter,Rect::from_min_size(Pos2::new(310.0,20.0),Vec2::new(100.0,110.0)),&view,&hud.icons,1.0);
        let mut chat_ui=crate::ui2d::tests::game_ui((1200,720));
        chat_ui.chat.height=262;
        hud.chat(&painter,Rect::from_min_size(Pos2::new(14.0,442.0),Vec2::new(344.0,262.0)),&chat_ui,&[ChatEntry::new(0,"Wender","Vamos para Giran?"),ChatEntry::new(0,"Grupo","Todos prontos. Podemos seguir! Esta mensagem comprida precisa quebrar corretamente antes de alcançar a borda da janela.")],1.0);
        hud.status(&painter,Rect::from_min_size(Pos2::new(16.0,16.0),Vec2::new(276.0,80.0)),&status_view(world.me.as_ref().unwrap()),1.0);
        chat_ui.shortcut.modern=Some(ModernShortcuts {count:2,..Default::default()});
        for index in 0..24 {chat_ui.shortcut.set(index,killer_ui::shortcut::ShortcutSlot::new(SlotKind::Item {object_id:index as i32},&buff_icons[index%buff_icons.len()]));}
        hud.shortcuts(&painter,Rect::from_min_size(Pos2::new(380.0,586.0),Vec2::new(556.0,118.0)),&chat_ui,1.0);
        let mut output=ctx.end_pass();
        let jobs=ctx.tessellate(output.shapes,output.pixels_per_point);
        assert!(!jobs.is_empty());
        let mut gpu=killer_render::headless::Headless::new(1200,720).unwrap();
        let renderer=gpu.renderer_mut();
        let device=renderer.device();let queue=renderer.queue();
        let target=device.create_texture(&wgpu::TextureDescriptor {label:None,size:wgpu::Extent3d {width:1200,height:720,depth_or_array_layers:1},mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,format:wgpu::TextureFormat::Rgba8Unorm,usage:wgpu::TextureUsages::RENDER_ATTACHMENT|wgpu::TextureUsages::COPY_SRC,view_formats:&[]});
        let view=target.create_view(&Default::default());
        let mut painter=egui_wgpu::Renderer::new(device,wgpu::TextureFormat::Rgba8Unorm,Default::default());
        for (id,deltas) in &output.textures_delta.set {for delta in deltas {painter.update_texture(device,queue,*id,delta);}}
        output.textures_delta.clear();
        let mut encoder=device.create_command_encoder(&Default::default());
        let screen=egui_wgpu::ScreenDescriptor {size_in_pixels:[1200,720],pixels_per_point:output.pixels_per_point};
        let extra=painter.update_buffers(device,queue,&mut encoder,&jobs,&screen);
        {
            let _pass=encoder.begin_render_pass(&wgpu::RenderPassDescriptor {label:None,color_attachments:&[Some(wgpu::RenderPassColorAttachment {view:&view,resolve_target:None,depth_slice:None,ops:wgpu::Operations {load:wgpu::LoadOp::Clear(wgpu::Color {r:0.65,g:0.62,b:0.48,a:1.0}),store:wgpu::StoreOp::Store}})],..Default::default()}).forget_lifetime();

        }
        hud.tessellate_names(&ctx);
        let mut names=killer_render::ui::UiRenderer::new_premultiplied(device,wgpu::TextureFormat::Rgba8Unorm);
        let font=painter.texture(&egui::TextureId::Managed(0)).unwrap().texture.as_ref().unwrap();
        names.upload_mesh(device,queue,&hud.name_mesh.borrow(),&font.create_view(&Default::default()));
        names.render(&mut encoder,queue,&view,(1200,720));
        {
            let mut pass=encoder.begin_render_pass(&wgpu::RenderPassDescriptor {label:None,color_attachments:&[Some(wgpu::RenderPassColorAttachment {view:&view,resolve_target:None,depth_slice:None,ops:wgpu::Operations {load:wgpu::LoadOp::Load,store:wgpu::StoreOp::Store}})],..Default::default()}).forget_lifetime();
            painter.render(&mut pass,&jobs,&screen);
        }
        let stride=4864;
        let buffer=device.create_buffer(&wgpu::BufferDescriptor {label:None,size:stride*720,usage:wgpu::BufferUsages::COPY_DST|wgpu::BufferUsages::MAP_READ,mapped_at_creation:false});
        encoder.copy_texture_to_buffer(target.as_image_copy(),wgpu::TexelCopyBufferInfo {buffer:&buffer,layout:wgpu::TexelCopyBufferLayout {offset:0,bytes_per_row:Some(stride as u32),rows_per_image:None}},wgpu::Extent3d {width:1200,height:720,depth_or_array_layers:1});
        queue.submit(extra.into_iter().chain([encoder.finish()]));
        let (tx,rx)=std::sync::mpsc::channel();
        buffer.slice(..).map_async(wgpu::MapMode::Read,move |value| {tx.send(value).unwrap();});
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();rx.recv().unwrap().unwrap();
        let bytes=buffer.slice(..).get_mapped_range().unwrap();
        let bubble=hud.name_shapes.borrow().iter().flat_map(|(_,shapes)|shapes).find_map(|shape|match &shape.shape {egui::Shape::Rect(rect)=>Some(rect.rect.shrink(2.0)),_=>None}).unwrap();
        let mut dark=0;
        let mut sampled=0;
        for y in bubble.min.y.ceil() as usize..bubble.max.y.floor() as usize {
            for x in bubble.min.x.ceil() as usize..bubble.max.x.floor() as usize {
                let pixel=&bytes[y*stride as usize+x*4..][..3];
                dark+=usize::from(pixel.iter().all(|value|*value<80));
                sampled+=1;
            }
        }
        assert!(sampled>0 && dark*2>sampled,"fundo de fala sem contraste: {dark}/{sampled}");
        let mut ppm=b"P6\n1200 720\n255\n".to_vec();
        for row in bytes.chunks_exact(stride as usize) {for pixel in row[..4800].as_chunks::<4>().0 {ppm.extend_from_slice(&pixel[..3]);}}
        std::fs::write("/tmp/killer-combat-preview.ppm",ppm).unwrap();
    }

    #[test]
    fn proportional_chat_wrap_preserves_unicode_and_fits_each_row() {
        let ctx=Context::default();
        ctx.begin_pass(Default::default());
        let p=ctx.layer_painter(LayerId::background());
        let entry=ChatEntry::new(0,"Wender","Mensagem comprida com ação, clã e WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW");
        for scale in [1.0,2.0] {
            let rows=modern_message_rows(&p,std::iter::once(&entry),320.0*scale,scale);
            assert!(rows.len()>1);
            let joined:String=rows.iter().map(|(_,text,_)|text.as_str()).collect();
            assert_eq!(joined.split_whitespace().collect::<String>(),entry.line().split_whitespace().collect::<String>());
            for (_,text,_) in rows {assert!(p.layout_no_wrap(text,FontId::proportional(12.0*scale),TEXT).size().x<=320.0*scale+1.0);}
        }
        let mut output=ctx.end_pass();
        output.textures_delta.clear();
    }

    #[test]
    fn modern_label_metrics_match_paint_and_speech_survives_hidden_names() {
        let (hud,mut ui,world,mut labels)=combat_fixture();
        for window in &mut ui.desktop.windows {window.visible=false;}
        labels[0].name="Wender Águila".into();
        labels[0].title="Cavaleiro de Aden".into();
        labels[0].speech=Some(killer_ui::store_sign::StoreSign {text:"Olá, pessoal!".into(),color:killer_ui::color::Argb(0xffffffff),background:killer_ui::color::Argb(0xff000000),screen:[310.0,200.0]});
        for scale in [1.0,2.0] {
            let ctx=Context::default();
            ctx.begin_pass(Default::default());
            ModernHud::measure_labels(&ctx,&ui,&mut labels,scale);
            let measured=labels[0].modern_text_size.unwrap();
            assert!(measured[0][1]>=14.0 && measured[1][1]>=13.0);
            let galley=ctx.fonts_mut(|fonts|fonts.layout_job(label_job(&labels[0].name,14.0,TEXT,scale)));
            assert!((galley.size().x/scale-measured[0][0]).abs()<0.01);
            labels[0].name_visible=false;
            hud.paint_combat(&ctx,&ui,&world,&labels,scale);
            let shapes=hud.name_shapes.borrow();
            let text:Vec<_>=shapes[0].1.iter().filter_map(|shape|match &shape.shape {egui::Shape::Text(text)=>Some(text.galley.text()),_=>None}).collect();
            assert!(text.contains(&"Olá, pessoal!"));
            assert!(!text.contains(&"Wender Águila"));
            drop(shapes);
            let mut output=ctx.end_pass();output.textures_delta.clear();
        }
    }

    #[test]
    fn overlapping_names_survive_crossing_and_reordering_between_frames() {
        let (hud,mut ui,world,mut labels)=combat_fixture();
        for window in &mut ui.desktop.windows {window.visible=false;}
        labels[0].name="Wender".into();
        labels[1].name="Vizinho".into();
        labels[0].depth=0.25;
        labels[1].depth=0.75;
        let ctx=Context::default();
        for frame in 0..6 {
            for label in &mut labels {label.screen=[310.0,240.0+(frame as f32-3.0)*0.1];}
            labels.reverse();
            let mut output=combat_frame(&hud,&ui,&world,&labels,&ctx);
            output.textures_delta.clear();
            let shapes=hud.name_shapes.borrow();
            assert_eq!(shapes.len(),2);
            assert!(shapes.iter().all(|(_,shapes)|!shapes.is_empty()));
            drop(shapes);
            hud.tessellate_names(&ctx);
            let mesh=hud.name_mesh.borrow();
            for depth in [0.25,0.75] {
                assert!(mesh.vertices.iter().any(|vertex|vertex.position[2]==depth));
            }
        }
    }

    #[test]
    fn long_chat_input_keeps_the_cursor_inside_at_both_display_scales() {
        for scale in [1.0,2.0] {
            let available=240.0*scale;
            assert_eq!(chat_input_offset(0.0,available,scale),0.0);
            assert_eq!(chat_input_offset(180.0*scale,available,scale),0.0);
            for cursor in [240.0,600.0,1200.0] {
                let cursor=cursor*scale;
                let x=cursor-chat_input_offset(cursor,available,scale);
                assert!(x>=0.0 && x<=available-2.0*scale);
            }
        }
    }

    #[test]
    fn preferences_round_trip_all_five_rows_and_reject_out_of_range_pages() {
        let value=HudPreferences { enabled:true,bars:ModernShortcuts { count:5,pages:[9,8,2,7],locked:true },primary:6,potions:[1060,5592,728],percents:[45,60,35],potions_off:[false,true,false],shots:[1463,3947] };
        assert_eq!(HudPreferences::parse(&value.text()),value);
        let value=HudPreferences::parse("modern=true\nbars=100\nprimary=-100\npages=999,-4,2,3\nlocked=true\n");
        assert_eq!(value.bars.count,5);
        assert_eq!(value.bars.pages,[9,0,2,3]);
        assert_eq!(value.primary,0);
        assert!(value.bars.locked);
    }

    #[test]
    fn modern_chat_scrolling_reaches_both_ends_with_header_space() {
        for (height,system) in [(262,false),(125,true)] {
            let newest=chat_range(80,0,height,system);
            let oldest=chat_range(80,usize::MAX,height,system);
            assert_eq!(newest.end,80);
            assert_eq!(oldest.start,0);
            assert_eq!(newest.len(),oldest.len());
            assert_eq!(chat_range(0,0,height,system),0..0);
            assert_eq!(chat_range(2,999,height,system),0..2);
        }
    }

    #[test]
    fn native_windows_occlude_the_modern_hud_without_overlapping_paint() {
        let bounds=Rect::from_min_max(Pos2::ZERO,Pos2::new(100.0,100.0));
        let cover=Rect::from_min_max(Pos2::new(20.0,30.0),Pos2::new(70.0,60.0));
        let pieces=subtract(bounds,cover);
        assert_eq!(pieces.len(),4);
        assert_eq!(pieces.iter().map(Rect::area).sum::<f32>(),bounds.area()-cover.area());
        assert!(pieces.iter().all(|piece|!piece.intersect(cover).is_positive()));
    }

    #[test]
    fn command_channel_produces_modern_shapes_after_server_open() {
        use killer_ui::command_channel as channel;
        let mut ui = crate::ui2d::tests::game_ui((1024, 768));
        for window in &mut ui.desktop.windows { window.visible = false; }
        ui.shortcut.modern = Some(Default::default());
        let world = World::default();
        ui.world_event(&killer_world::WorldEvent::OpenMpcc, &world);
        let mut hud = ModernHud::default();
        hud.capture_windows(&ui, &world, &[], None, &crate::ui2d::WorldViews::default());
        assert_eq!(hud.windows.len(), 1);
        assert!(hud.windows[&channel::WINDOW_ID].quads.iter().any(|quad| quad.texture == channel::BACK));
        let ctx = Context::default();
        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1024.0, 768.0))),
            ..Default::default()
        });
        hud.paint(&ctx, &ui, &world, &[], 1.0);
        let mut output = ctx.end_pass();
        output.textures_delta.clear();
        assert!(!ctx.tessellate(output.shapes, output.pixels_per_point).is_empty());
    }
}
