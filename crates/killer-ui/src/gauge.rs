use crate::draw::DrawList;

pub const GAUGE_FILL: [&str; 4] = [
    "L2UI_CH3.etc.Minibar_water",
    "L2UI_CH3.Etc.Minibar_Arrow",
    "L2UI_CH3.Etc.Minibar_Magic",
    "L2UI_CH3.Etc.Minibar_Food",
];
pub const GAUGE_BACK: [&str; 3] =
    ["L2UI_CH3.etc.Minibar_Back21", "L2UI_CH3.etc.Minibar_Back22", "L2UI_CH3.etc.Minibar_Back23"];
pub const GAUGE_RISE: i32 = 44;
pub const GAUGE_INSET: i32 = 2;
/// O desenho da barra no `engine.dll` (`0x10529d80`–`0x1052a0e0`, o bloco que carrega as sete
/// texturas `Minibar_*` em `0x10529a9f` e seguintes) **não tem medida constante**: para cada peça
/// ele chama os slots `0x74` (`USize`) e `0x78` (`VSize`) da textura e usa o que vier
/// (`0x10529f20`, `0x10529f37`, `0x10529fcf`, `0x10529fe6`, `0x1052a07b`, `0x1052a092`). No cliente
/// as peças medem: fundo 8 × 8 (as três) e preenchimento 8 × 4 — conferido no teste
/// `minibar_sizes` de `crates/killer-assets/src/textures.rs`. Por isso a ponta tem **8** de largura
/// e a barra **8** de altura, e cada peça é desenhada com o retângulo de origem do tamanho da
/// textura, não recortada.
pub const GAUGE_BACK_TEXELS: (i32, i32) = (8, 8);
pub const GAUGE_FILL_TEXELS: (i32, i32) = (8, 4);
pub const GAUGE_END_WIDTH: i32 = GAUGE_BACK_TEXELS.0;
pub const GAUGE_HEIGHT: i32 = GAUGE_BACK_TEXELS.1;
pub const GAUGE_FULL_FILL: i32 = 96;
pub const GAUGE_WIDTH: i32 = GAUGE_FULL_FILL + 2 * GAUGE_INSET;
pub const STACK_DISTANCE_SCALE: f32 = 0.001;
pub const STACK_FACTOR: f32 = 3.0;

pub fn stack_step(focal: f32) -> i32 {
    (STACK_FACTOR * STACK_DISTANCE_SCALE * focal) as i32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GaugeView {
    pub kind: i32,
    pub remaining_ms: i32,
    pub max_ms: i32,
}

impl GaugeView {
    pub fn fill_texture(&self) -> &'static str {
        GAUGE_FILL[self.kind.clamp(0, 3) as usize]
    }

    pub fn remaining_width(&self, inner: i32) -> i32 {
        if self.max_ms <= 0 {
            return 0;
        }
        (i64::from(self.remaining_ms.clamp(0, self.max_ms)) * i64::from(inner) / i64::from(self.max_ms)) as i32
    }
}

pub fn paint_gauges(list: &mut DrawList, anchor_x: i32, anchor_y: i32, step: i32, gauges: &[GaugeView]) {
    let w = GAUGE_WIDTH;
    let h = GAUGE_HEIGHT;
    let (bu, bv) = GAUGE_BACK_TEXELS;
    let (fu, fv) = GAUGE_FILL_TEXELS;
    let x = anchor_x - w / 2;
    for (index, gauge) in gauges.iter().enumerate() {
        let y = anchor_y - GAUGE_RISE - index as i32 * step;
        let end = GAUGE_END_WIDTH;
        list.draw_texture(x, y, end, h, 0, 0, bu, bv, GAUGE_BACK[0], 1.0);
        list.draw_texture(x + end, y, w - 2 * end, h, 0, 0, bu, bv, GAUGE_BACK[1], 1.0);
        list.draw_texture(x + w - end, y, end, h, 0, 0, bu, bv, GAUGE_BACK[2], 1.0);
        let inner = w - 2 * GAUGE_INSET;
        let fill = gauge.remaining_width(inner);
        if fill > 0 {
            list.draw_texture(x + GAUGE_INSET, y + GAUGE_INSET, fill, fv, 0, 0, fu, fv, gauge.fill_texture(), 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Quad;

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    /// As peças saem no tamanho da própria textura, como o engine faz chamando `USize`/`VSize`
    /// (`0x10529f20`, `0x10529f37`, …): fundo 8 × 8 e preenchimento 8 × 4, sem recorte. Antes o
    /// porte desenhava 4 × 6 recortados de uma textura de 8 × 8.
    #[test]
    fn gauges_stack_44_pixels_above_the_anchor_with_the_fill_by_the_remaining_time() {
        let gauges = [
            GaugeView { kind: 2, remaining_ms: 2500, max_ms: 5000 },
            GaugeView { kind: 0, remaining_ms: 0, max_ms: 5000 },
        ];
        let mut list = DrawList::default();
        paint_gauges(&mut list, 400, 300, 8, &gauges);
        let left = with_texture(&list.quads, GAUGE_BACK[0]);
        assert_eq!(left.len(), 2);
        assert_eq!((left[0].x, left[0].y, left[0].w, left[0].h), (350.0, 256.0, 8.0, 8.0));
        assert_eq!((left[0].uw, left[0].vh), (8.0, 8.0), "a peça de fundo é desenhada inteira");
        assert_eq!(left[1].y, 248.0);
        let mid = with_texture(&list.quads, GAUGE_BACK[1]);
        assert_eq!((mid[0].x, mid[0].w), (358.0, 84.0));
        assert_eq!(with_texture(&list.quads, GAUGE_BACK[2])[0].x, 442.0);
        let magic = with_texture(&list.quads, GAUGE_FILL[2]);
        assert_eq!((magic[0].x, magic[0].y, magic[0].w, magic[0].h), (352.0, 258.0, 48.0, 4.0));
        assert_eq!((magic[0].uw, magic[0].vh), (8.0, 4.0), "o preenchimento também sai inteiro");
        assert!(with_texture(&list.quads, GAUGE_FILL[0]).is_empty());
        assert_eq!(GaugeView { kind: 9, remaining_ms: 1, max_ms: 1 }.fill_texture(), GAUGE_FILL[3]);
        assert_eq!(GaugeView { kind: 1, remaining_ms: 7000, max_ms: 5000 }.remaining_width(96), 96);
        assert_eq!(GaugeView { kind: 1, remaining_ms: 10, max_ms: 0 }.remaining_width(96), 0);
        assert_eq!(GAUGE_WIDTH, 100);
        assert_eq!(stack_step(886.8), 2);
    }
}
