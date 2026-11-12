use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, TextureHandle, Vec2};
use killer_ui::draw::{DrawList, Quad};
use killer_ui::window::WindowId;
use std::collections::HashMap;
use crate::ui_modern::{GOLD, TEXT, MUTED, panel};
use crate::ui_theme as theme;

pub fn supports(id:WindowId)->bool {
    !matches!(id,WindowId::STATUS|WindowId::CHAT|WindowId::SYSTEM_MSG|WindowId::SHORTCUT|WindowId::ZONE_TITLE|WindowId::RADAR|WindowId::LOADING|WindowId::TUTORIAL_BUTTON)
}

#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Surface { Image, Hidden, Close, Grip, Tab(bool), Button(bool), Slot(bool), Check(bool,bool), Input, Bar(Color32), Track, Line, Arrow(bool), MainTab(usize,bool) }

pub fn surface(name:&str)->Surface {
    let name=name.to_ascii_lowercase();
    let leaf=name.rsplit('.').next().unwrap_or_default();
    if name=="l2ui.squarewhite" {return Surface::Line;}
    if !name.starts_with("l2ui.") && !name.starts_with("l2ui_ch3.") && !name.starts_with("nwindow.") {return Surface::Image;}
    if leaf.contains("checkbox") {return Surface::Check(leaf.contains("checked"),leaf.contains("unable"));}
    if leaf.starts_with("textbox") || leaf.starts_with("inputbox") {return Surface::Input;}
    if leaf=="textbackline" {return Surface::Image;}
    if leaf=="blacktexture" {return Surface::Track;}
    if leaf.starts_with("downbutton") {return Surface::Arrow(false);}
    if leaf.starts_with("upbutton") {return Surface::Arrow(true);}
    if leaf.starts_with("sliderbar") {return Surface::Button(false);}
    if leaf=="frameclosebtn" {return Surface::Close;}
    if leaf.starts_with("frameback") || leaf.starts_with("smallbar") {return Surface::Grip;}
    if let Some(tab)=leaf.strip_prefix("mainwndtabbtn") {
        let index=tab.chars().next().and_then(|c|c.to_digit(10)).unwrap_or(0) as usize;
        return Surface::MainTab(index.saturating_sub(1),tab.ends_with("on"));
    }
    if leaf=="mainwndtabback" {return Surface::Hidden;}
    if leaf.contains("hpbar") && !leaf.contains("back") {return Surface::Bar(Color32::from_rgb(165,57,54));}
    if leaf.contains("mpbar") && !leaf.contains("back") {return Surface::Bar(Color32::from_rgb(58,119,181));}
    if (leaf.contains("cpbar") || leaf.contains("weightbar") || leaf.contains("expbar")) && !leaf.contains("back") {return Surface::Bar(GOLD);}
    if leaf.contains("bar_back") {return Surface::Track;}
    if leaf=="icon_back" || leaf=="inventory_outline" || leaf=="inventory_outline_down" {return Surface::Slot(false);}
    if leaf=="item_click" || leaf=="icon_click" {return Surface::Slot(true);}
    if leaf.contains("scroll") {
        if leaf.contains("up") {return Surface::Arrow(true);}
        if leaf.contains("down") {return Surface::Arrow(false);}
        return if leaf.contains("back") {Surface::Track}else{Surface::Button(false)};
    }
    if name.contains(".listctrl.") && leaf.starts_with("tab") {return Surface::Button(leaf.ends_with("down"));}
    if leaf.contains("tab") && !leaf.contains("icon") {return Surface::Tab(leaf.ends_with('1') || leaf.ends_with("down"));}
    if name.contains(".button.") || leaf.starts_with("btn") {return Surface::Button(leaf.contains("down") || leaf.ends_with("on"));}
    if leaf.contains("textselect") {return Surface::Tab(true);}
    Surface::Image
}

fn rgba(color:[f32;4])->Color32 {Color32::from_rgba_unmultiplied((color[0]*255.0) as u8,(color[1]*255.0) as u8,(color[2]*255.0) as u8,(color[3]*255.0) as u8)}
fn bounds(q:&Quad,scale:f32)->Rect {Rect::from_min_size(Pos2::new(q.x,q.y)*scale,Vec2::new(q.w,q.h)*scale)}
fn control(p:&Painter,r:Rect,selected:bool,scale:f32) {
    let hovered=p.ctx().pointer_hover_pos().is_some_and(|pos|r.contains(pos));
    let radius=5.0*scale;
    p.rect_filled(r,radius,if selected {Color32::from_rgba_unmultiplied(214,178,106,40)}else if hovered {Color32::from_white_alpha(30)}else{Color32::from_white_alpha(14)});
    theme::vertical(p,Rect::from_min_max(r.min+Vec2::new(3.0*scale,scale),Pos2::new(r.max.x-3.0*scale,r.min.y+r.height()*0.5)),Color32::from_white_alpha(if selected {26}else{14}),Color32::from_white_alpha(0));
    p.rect_stroke(r,radius,Stroke::new(scale,if selected {GOLD}else{Color32::from_white_alpha(if hovered {90}else{44})}),StrokeKind::Inside);
}

fn tab_surface(p:&Painter,r:Rect,selected:bool,scale:f32) {
    let hovered=p.ctx().pointer_hover_pos().is_some_and(|pos|r.contains(pos));
    if selected || hovered {p.rect_filled(r,5.0*scale,if selected {Color32::from_rgba_unmultiplied(214,178,106,38)}else{Color32::from_white_alpha(14)});}
    p.line_segment([r.left_bottom(),r.right_bottom()],Stroke::new(scale,Color32::from_white_alpha(26)));
    if selected {p.line_segment([r.left_bottom()+Vec2::new(6.0,-1.0)*scale,r.right_bottom()-Vec2::new(6.0,1.0)*scale],Stroke::new(2.0*scale,GOLD));}
}

pub fn extent(area:Rect,list:&DrawList,scale:f32)->Rect {
    list.quads.iter().fold(area,|area,q|area.union(bounds(q,scale)))
}

pub fn paint(p:&Painter,area:Rect,list:&DrawList,icons:&HashMap<String,Option<TextureHandle>>,scale:f32,_id:WindowId) {
    if list.quads.is_empty() {return;}
    panel(p,area);
    paint_content(p,list,icons,scale);
}

pub fn paint_content(p:&Painter,list:&DrawList,icons:&HashMap<String,Option<TextureHandle>>,scale:f32) {
    let mut index=0;
    let mut runs=list.text_runs.iter().peekable();
    while index<list.quads.len() {
        if let Some(run)=runs.peek().filter(|run|run.quads.start==index) {
            let [x,y,w,h]=run.rect;
            let color=rgba(run.color);
            let size=h.clamp(11.0,13.0)*scale;
            let galley=p.layout_no_wrap(run.text.clone(),FontId::proportional(size),color);
            let size=if galley.size().x>w*scale && w>0.0 {size*(w*scale/galley.size().x)}else{size};
            let text_painter=if let Some(clip)=run.clip {
                p.with_clip_rect(p.clip_rect().intersect(Rect::from_min_size(Pos2::new(clip.x as f32,clip.y as f32)*scale,Vec2::new(clip.w as f32,clip.h as f32)*scale)))
            }else{p.clone()};
            text_painter.text(Pos2::new(x,y)*scale,Align2::LEFT_TOP,&run.text,FontId::proportional(size),color);
            index=run.quads.end;
            runs.next();
            continue;
        }
        let q=&list.quads[index];
        let r=bounds(q,scale);
        match surface(&q.texture) {
            Surface::Hidden => {},
            Surface::Input => {
                theme::inset(p,r,scale,false);
            },
            Surface::Check(checked,disabled) => {
                let r=r.shrink(2.0*scale);
                control(p,r,false,scale);
                if checked {
                    let color=if disabled {MUTED}else{GOLD};
                    let mid=r.center()+Vec2::new(-1.0,2.0)*scale;
                    p.line_segment([r.left_center()+Vec2::new(3.0,0.0)*scale,mid],Stroke::new(1.5*scale,color));
                    p.line_segment([mid,r.right_top()+Vec2::new(-2.0,3.0)*scale],Stroke::new(1.5*scale,color));
                }
            },
            Surface::Grip => {
                theme::vertical(p,r,Color32::from_white_alpha(22),Color32::from_white_alpha(4));
                p.line_segment([r.left_bottom(),r.right_bottom()],Stroke::new(scale,Color32::from_white_alpha(30)));
            },
            Surface::Close => {
                control(p,r,false,scale);
                let hovered=p.ctx().pointer_hover_pos().is_some_and(|pos|r.contains(pos));
                if hovered {p.rect_filled(r.shrink(scale),4.0*scale,Color32::from_rgba_unmultiplied(214,72,72,150));}
                let inner=r.shrink(4.0*scale);
                let color=if hovered {TEXT}else{MUTED};
                p.line_segment([inner.left_top(),inner.right_bottom()],Stroke::new(scale,color));
                p.line_segment([inner.right_top(),inner.left_bottom()],Stroke::new(scale,color));
            },
            Surface::MainTab(tab,selected) => {
                let r=r.shrink2(Vec2::new(scale,4.0*scale));
                tab_surface(p,r,selected,scale);
                if let Some(label)=["Status","Skills","Ações","Clã","Quests"].get(tab) {
                    p.text(r.center(),Align2::CENTER_CENTER,label,FontId::proportional(10.0*scale),if selected {TEXT}else{MUTED});
                }
            },
            Surface::Tab(selected) => tab_surface(p,r.shrink(scale),selected,scale),
            Surface::Button(selected) => control(p,r.shrink(scale),selected,scale),
            Surface::Slot(selected) => {
                if selected {p.rect_filled(r,4.0*scale,Color32::from_rgba_unmultiplied(214,178,106,34));p.rect_stroke(r,4.0*scale,Stroke::new(2.0*scale,GOLD),StrokeKind::Inside);}else{theme::slot_front(p,r,scale,false);}
            },
            Surface::Bar(color) => theme::bar_fill(p,r,(color,color.gamma_multiply(0.7))),
            Surface::Track => theme::bar_rail(p,r),
            Surface::Line => {p.rect_filled(r,0.0,rgba(q.color));},
            Surface::Arrow(up) => {
                control(p,r,false,scale);
                super::ui_modern::chevron(p,r,up,TEXT,scale);
            },
            Surface::Image => if let Some(Some(image))=icons.get(q.texture.as_str()) {
                let [w,h]=image.size();
                let uv=Rect::from_min_max(Pos2::new(q.u/w as f32,q.v/h as f32),Pos2::new((q.u+q.uw)/w as f32,(q.v+q.vh)/h as f32));
                let mut mesh=egui::Mesh::with_texture(image.id());
                mesh.add_rect_with_uv(r,uv,rgba(q.color));
                if q.rotation!=0.0 {mesh.rotate(egui::emath::Rot2::from_angle(q.rotation),r.center());}
                p.add(egui::Shape::mesh(mesh));
            },
        }
        index+=1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_background_art_is_not_discarded_by_the_modern_theme() {
        for name in [
            "L2UI_CH3.InventoryWnd.itemenchant_back",
            killer_ui::detail_status::MYINFO_BACK,
            "L2UI_CH3.InventoryWnd.Inventory_Back",
            "L2UI_CH3.WarehouseWnd.Warehouse_Back",
            killer_ui::command_channel::BACK,
            killer_ui::ssq::STRIPE,
            "L2UI_CH3.Etc.textbackline",
        ] {
            assert_eq!(surface(name), Surface::Image, "{name}");
        }
    }

    #[test]
    fn decorative_overlays_remain_outlines_and_assets_keep_their_images() {
        for texture in [killer_ui::inventory::OUTLINE,killer_ui::inventory::OUTLINE_DOWN,killer_ui::skills::ICON_BACK] {
            assert_eq!(surface(texture),Surface::Slot(false));
        }
        assert_eq!(surface(killer_ui::inventory::ITEM_CLICK),Surface::Slot(true));
        for texture in ["icon.skill0001","Icon.weapon_short_sword_i00","town_map.giran","L2UI.WindowIcon.Weight"] {
            assert_eq!(surface(texture),Surface::Image);
        }
    }

    #[test]
    fn all_main_tabs_keep_the_native_content_and_capture_their_text() {
        use killer_ui::keys::MainTab;
        let mut ui=crate::ui2d::tests::game_ui((1558,933));
        let mut world=crate::characters::demo_world(glam::Vec3::ZERO,0,0);
        crate::ui_keys::demo_fill(&mut world);
        let views=crate::ui2d::world_views(&world,None,killer_platform::time::Instant::now());
        for window in &ui.desktop.windows {
            if !supports(window.id) {continue;}
            let mut original=DrawList::default();
            let mut modern=DrawList {capture_text:true,..Default::default()};
            ui.paint_window(&mut original,window,&world,&[],None,&views);
            ui.paint_window(&mut modern,window,&world,&[],None,&views);
            assert_eq!(original.quads,modern.quads,"{:?}",window.id);
            assert!(modern.text_runs.iter().all(|run|run.quads.start<run.quads.end && run.quads.end<=modern.quads.len()));
        }
        for tab in MainTab::ALL {
            ui.keys.main.tab=tab;
            let window=ui.desktop.window(WindowId::MAIN).unwrap();
            let mut original=DrawList::default();
            let mut modern=DrawList {capture_text:true,..Default::default()};
            ui.paint_window(&mut original,window,&world,&[],None,&views);
            ui.paint_window(&mut modern,window,&world,&[],None,&views);
            assert_eq!(original.quads,modern.quads);
            assert!(!modern.text_runs.is_empty());
            assert!(modern.text_runs.iter().all(|run|run.quads.end<=modern.quads.len()));
            assert_eq!(modern.quads.iter().filter(|q|matches!(surface(&q.texture),Surface::MainTab(_,true))).count(),1);
        }
    }
}
