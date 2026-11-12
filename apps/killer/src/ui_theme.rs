use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2};

pub(crate) const BACK: Color32 = Color32::from_rgba_premultiplied(12, 13, 16, 214);
pub(crate) const BRONZE: Color32 = Color32::from_rgba_premultiplied(40, 40, 40, 40);
pub(crate) const BRONZE_HI: Color32 = Color32::from_rgb(214, 178, 106);
pub(crate) const BRONZE_DIM: Color32 = Color32::from_rgba_premultiplied(34, 34, 34, 34);
pub(crate) const GOLD: Color32 = Color32::from_rgb(214, 178, 106);
pub(crate) const GOLD_HI: Color32 = Color32::from_rgb(248, 244, 236);
pub(crate) const TEXT: Color32 = Color32::from_rgb(236, 236, 238);
pub(crate) const MUTED: Color32 = Color32::from_rgb(142, 146, 156);

pub(crate) const CP: (Color32, Color32) = (Color32::from_rgb(232, 188, 84), Color32::from_rgb(206, 156, 52));
pub(crate) const HP: (Color32, Color32) = (Color32::from_rgb(226, 84, 84), Color32::from_rgb(190, 58, 62));
pub(crate) const MP: (Color32, Color32) = (Color32::from_rgb(84, 150, 238), Color32::from_rgb(56, 112, 206));
pub(crate) const XP: (Color32, Color32) = (Color32::from_rgb(158, 206, 120), Color32::from_rgb(130, 180, 96));

pub(crate) fn vertical(p: &Painter, rect: Rect, top: Color32, bottom: Color32) {
    gradient_quad(p, [rect.left_top(), rect.right_top(), rect.right_bottom(), rect.left_bottom()], top, bottom);
}

/// Painel: vidro escuro com degradê, borda fina, brilho no topo e um acento dourado no canto.
pub(crate) fn plate(p: &Painter, rect: Rect, scale: f32, focused: bool) {
    let radius = 8.0 * scale;
    p.rect_filled(rect.expand(2.0 * scale), radius + 2.0 * scale, Color32::from_black_alpha(46));
    p.rect_filled(rect, radius, BACK);
    let body = rect.shrink(scale);
    vertical(p, Rect::from_min_max(body.min + Vec2::new(3.0 * scale, 2.0 * scale), Pos2::new(body.max.x - 3.0 * scale, body.min.y + body.height() * 0.5)), Color32::from_white_alpha(if focused { 26 } else { 16 }), Color32::from_white_alpha(0));
    p.rect_stroke(rect, radius, Stroke::new(scale, Color32::from_white_alpha(if focused { 80 } else { 34 })), StrokeKind::Inside);
    let top = rect.left_top() + Vec2::new(radius, 0.5 * scale);
    p.line_segment([top, top + Vec2::new(34.0 * scale, 0.0)], Stroke::new(2.0 * scale, GOLD));
    p.line_segment([top + Vec2::new(34.0 * scale, 0.0), rect.right_top() + Vec2::new(-radius, 0.5 * scale)], Stroke::new(scale, Color32::from_white_alpha(40)));
}

/// Campo ou casa rebaixada (caixa de texto).
pub(crate) fn inset(p: &Painter, rect: Rect, scale: f32, active: bool) {
    let radius = 5.0 * scale;
    p.rect_filled(rect, radius, Color32::from_black_alpha(130));
    p.rect_stroke(rect, radius, Stroke::new(scale, if active { GOLD } else { Color32::from_white_alpha(24) }), StrokeKind::Inside);
}

/// Casa de atalho: fundo fundo, aro duplo e, depois do ícone, acabamento com brilho e sombra interna.
pub(crate) fn slot_back(p: &Painter, rect: Rect, scale: f32) {
    let radius = 4.0 * scale;
    p.rect_filled(rect.expand(scale), radius + scale, Color32::from_black_alpha(90));
    p.rect_filled(rect, radius, Color32::from_rgb(10, 10, 13));
}

pub(crate) fn slot_front(p: &Painter, rect: Rect, scale: f32, filled: bool) {
    let radius = 4.0 * scale;
    if filled {
        vertical(p, Rect::from_min_max(rect.min + Vec2::splat(scale), Pos2::new(rect.max.x - scale, rect.min.y + rect.height() * 0.45)), Color32::from_white_alpha(34), Color32::from_white_alpha(0));
        p.rect_stroke(rect.shrink(scale), radius - scale, Stroke::new(scale, Color32::from_black_alpha(150)), StrokeKind::Inside);
    }
    p.rect_stroke(rect, radius, Stroke::new(scale, Color32::from_white_alpha(if filled { 84 } else { 40 })), StrokeKind::Inside);
}

/// Texto com contorno escuro, legível sobre barras claras e sobre o cenário.
pub(crate) fn outlined(p: &Painter, pos: Pos2, align: egui::Align2, value: &str, size: f32, color: Color32, scale: f32) {
    for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0), (1.0, 1.0)] {
        p.text(pos + Vec2::new(dx, dy) * scale, align, value, egui::FontId::proportional(size), Color32::from_black_alpha(210));
    }
    p.text(pos, align, value, egui::FontId::proportional(size), color);
}

fn slanted(rect: Rect, skew: f32) -> [Pos2; 4] {
    [Pos2::new(rect.left() + skew, rect.top()), Pos2::new(rect.right() + skew, rect.top()), Pos2::new(rect.right(), rect.bottom()), Pos2::new(rect.left(), rect.bottom())]
}

fn gradient_quad(p: &Painter, points: [Pos2; 4], top: Color32, bottom: Color32) {
    let mut mesh = egui::Mesh::default();
    for (index, point) in points.into_iter().enumerate() {
        mesh.colored_vertex(point, if index < 2 { top } else { bottom });
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    p.add(Shape::mesh(mesh));
}

pub(crate) fn bar_rail(p: &Painter, rect: Rect) {
    let skew = rect.height() * 0.45;
    let rail = Rect::from_min_max(rect.min, Pos2::new(rect.max.x - skew, rect.max.y));
    p.add(Shape::convex_polygon(slanted(rail.expand(1.0), skew).to_vec(), Color32::from_black_alpha(150), Stroke::NONE));
    gradient_quad(p, slanted(rail, skew), Color32::from_rgba_unmultiplied(0, 0, 0, 200), Color32::from_rgba_unmultiplied(40, 40, 46, 190));
    p.add(Shape::closed_line(slanted(rail, skew).to_vec(), Stroke::new(1.0, Color32::from_white_alpha(70))));
}

pub(crate) fn bar_fill(p: &Painter, rect: Rect, colors: (Color32, Color32)) {
    let skew = rect.height() * 0.45;
    let fill = Rect::from_min_max(rect.min, Pos2::new((rect.max.x - skew).max(rect.min.x + skew), rect.max.y));
    let glow = Color32::from_rgba_unmultiplied(colors.0.r(), colors.0.g(), colors.0.b(), 46);
    p.add(Shape::convex_polygon(slanted(fill.expand(2.0), skew).to_vec(), glow, Stroke::NONE));
    gradient_quad(p, slanted(fill, skew), colors.0, colors.1);
    let upper = Rect::from_min_size(fill.min, Vec2::new(fill.width(), fill.height() * 0.45));
    gradient_quad(p, slanted(upper, skew * 0.45).map(|point| point + Vec2::new(skew * 0.55, 0.0)), Color32::from_white_alpha(96), Color32::from_white_alpha(8));
}

/// Barra de energia inclinada: trilho com borda, preenchimento em degradê com brilho, aura da cor e segmentos.
pub(crate) fn gauge(p: &Painter, rect: Rect, ratio: f32, colors: (Color32, Color32)) {
    let skew = rect.height() * 0.45;
    bar_rail(p, rect);
    let ratio = ratio.clamp(0.0, 1.0);
    if ratio > 0.0 {
        let width = ((rect.width() - skew) * ratio + skew).max(2.0 * skew);
        bar_fill(p, Rect::from_min_size(rect.min, Vec2::new(width, rect.height())), colors);
        let rail = Rect::from_min_max(rect.min, Pos2::new(rect.max.x - skew, rect.max.y));
        for step in 1..4 {
            let x = rail.left() + rail.width() * step as f32 / 4.0;
            if x < rect.left() + width - skew {
                p.line_segment([Pos2::new(x + skew, rail.top()), Pos2::new(x, rail.bottom())], Stroke::new(1.0, Color32::from_black_alpha(80)));
            }
        }
    }
}

/// Botão: placa simples com realce quando ativo.
pub(crate) fn button(p: &Painter, rect: Rect, scale: f32, active: bool) {
    let radius = 6.0 * scale;
    p.rect_filled(rect, radius, if active { Color32::from_white_alpha(30) } else { Color32::from_rgba_unmultiplied(12, 13, 16, 214) });
    p.rect_stroke(rect, radius, Stroke::new(scale, if active { GOLD } else { Color32::from_white_alpha(30) }), StrokeKind::Inside);
}
