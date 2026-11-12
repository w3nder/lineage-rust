use egui::{Color32,FontId,Painter,Pos2,Rect,Stroke,StrokeKind,TextureHandle,Vec2};
use killer_ui::abnormal::{AbnormalView,AbnormalHit,MAX_BUFFS};
use killer_ui::window::Rect as NativeRect;
use std::collections::HashMap;
use crate::ui_modern::{GOLD,TEXT};

pub fn cells(view:&AbnormalView)->Vec<(AbnormalHit,NativeRect)> {
    let mut entries:Vec<_>=(0..view.buffs.len().min(MAX_BUFFS)).map(AbnormalHit::Buff).collect();
    if view.short_buff.is_some() {entries.push(AbnormalHit::ShortBuff);}
    entries.extend((0..view.etc.len()).map(AbnormalHit::Etc));
    entries.into_iter().enumerate().map(|(index,hit)|(hit,cell(index%10,index/10))).collect()
}

fn cell(column:usize,row:usize)->NativeRect {NativeRect::new(12+column as i32*30,4+row as i32*30,28,28)}

pub fn size(view:&AbnormalView)->(i32,i32) {
    cells(view).iter().fold((12,0),|(w,h),(_,r)|(w.max(r.x+r.w+4),h.max(r.y+r.h+4)))
}

pub fn hit(view:&AbnormalView,x:i32,y:i32)->Option<AbnormalHit> {
    cells(view).into_iter().find(|(_,rect)|rect.contains(x,y)).map(|(hit,_)|hit)
}

pub fn paint(p:&Painter,area:Rect,view:&AbnormalView,icons:&HashMap<String,Option<TextureHandle>>,scale:f32) {
    for (kind,cell) in cells(view) {
        let (icon,remaining,debuff)=match kind {
            AbnormalHit::Buff(index)=>{let buff=&view.buffs[index];(buff.icon.as_ref(),buff.remaining,buff.debuff)},
            AbnormalHit::ShortBuff=>{let Some(buff)=&view.short_buff else {continue;};(buff.icon.as_ref(),buff.remaining,buff.debuff)},
            AbnormalHit::Etc(index)=>(view.etc[index].as_ref(),None,false),
        };
        let rect=Rect::from_min_size(area.min+Vec2::new(cell.x as f32,cell.y as f32)*scale,Vec2::splat(28.0*scale));
        let hovered=p.ctx().pointer_hover_pos().is_some_and(|position|rect.contains(position));
        p.rect_filled(rect,4.0,Color32::from_rgb(14,18,22));
        if let Some(Some(image))=icon.and_then(|name|icons.get(name)) {
            p.image(image.id(),rect.shrink(scale),Rect::from_min_max(Pos2::ZERO,Pos2::new(1.0,1.0)),Color32::WHITE);
        }
        let border=if debuff {Color32::from_rgb(211,83,78)}else if hovered {GOLD}else{Color32::from_white_alpha(35)};
        if hovered {p.rect_filled(rect.shrink(scale),3.0,Color32::from_white_alpha(22));}
        p.rect_stroke(rect,4.0,Stroke::new(if debuff {1.5}else{1.0}*scale,border),StrokeKind::Inside);
        if let Some(seconds)=remaining.filter(|seconds|*seconds>=0) {
            let label=if seconds>=3600 {format!("{}h",seconds/3600)}else if seconds>=60 {format!("{}m",seconds/60)}else{seconds.to_string()};
            let color=if debuff {Color32::from_rgb(255,153,144)}else if seconds<10 {GOLD}else{TEXT};
            let galley=p.layout_no_wrap(label,FontId::proportional(10.0*scale),color);
            let badge=Rect::from_min_size(Pos2::new(rect.right()-galley.size().x-5.0*scale,rect.bottom()-12.0*scale),Vec2::new(galley.size().x+4.0*scale,11.0*scale));
            p.rect_filled(badge,2.0*scale,Color32::from_black_alpha(225));
            p.galley(badge.center()-galley.size()*0.5,galley,color);
        }
    }
    if !view.is_empty() {
        let center=area.left_center()+Vec2::new(5.0,0.0)*scale;
        for y in [-4.0,0.0,4.0] {p.circle_filled(center+Vec2::new(0.0,y)*scale,scale,Color32::from_rgb(102,108,114));}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::abnormal::BuffIcon;
    #[test]
    fn expanded_effect_slots_match_hover_and_preserve_short_and_extra_effects() {
        for count in 0..=31 {
            let view=AbnormalView {buffs:(0..count).map(|index|BuffIcon {debuff:index%3==1,..Default::default()}).collect(),short_buff:Some(BuffIcon::default()),etc:vec![None;12],etc_skills:Vec::new()};
            let (w,h)=size(&view);
            let cells=cells(&view);
            assert_eq!(cells.len(),count.min(MAX_BUFFS)+13);
            for (expected,r) in cells {
                assert_eq!(hit(&view,r.x+16,r.y+16),Some(expected));
                assert_eq!(hit(&view,r.x+r.w+1,r.y+16),None);
                assert!(r.x+r.w<=w && r.y+r.h<=h);
            }
        }
    }
}
