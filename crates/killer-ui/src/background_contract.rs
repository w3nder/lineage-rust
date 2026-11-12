use crate::draw::{DrawList, Quad};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipProof {
    Unknown,
    Disabled,
    Contained([i32; 4]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundError {
    NonFinite,
    Opacity,
    Geometry,
    Clip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackgroundContract {
    pub rect: [i32; 4],
    pub uv_bits: [u32; 4],
    pub texture: &'static str,
    pub alpha: u32,
    pub clip: u32,
    pub clip_proof: ClipProof,
    pub on_paint: u32,
    pub operation_index: usize,
    pub helper_return: u32,
}

fn exact_rect(rect: [i32; 4]) -> bool {
    let [x, y, w, h] = rect;
    w > 0
        && h > 0
        && [Some(x), Some(y), Some(w), Some(h), x.checked_add(w), y.checked_add(h)]
            .into_iter()
            .all(|v| v.is_some_and(|v| (-16_777_216..=16_777_216).contains(&v)))
}

impl BackgroundContract {
    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), opacity: f32) -> Result<(), BackgroundError> {
        let uv = self.uv_bits.map(f32::from_bits);
        if !opacity.is_finite() || uv.iter().any(|v| !v.is_finite()) {
            return Err(BackgroundError::NonFinite);
        }
        if !(0.0..=1.0).contains(&opacity) {
            return Err(BackgroundError::Opacity);
        }
        let [x, y, w, h] = self.rect;
        if !exact_rect(self.rect) {
            return Err(BackgroundError::Geometry);
        }
        match self.clip_proof {
            ClipProof::Disabled if self.clip as u8 == 0 => {}
            ClipProof::Contained(rect) if self.clip as u8 != 0 => {
                let [cx, cy, cw, ch] = rect;
                if !exact_rect(rect)
                    || x < cx
                    || y < cy
                    || i64::from(x) + i64::from(w) > i64::from(cx) + i64::from(cw)
                    || i64::from(y) + i64::from(h) > i64::from(cy) + i64::from(ch)
                    || uv[0] < 0.0
                    || uv[1] < 0.0
                    || uv[2] <= 0.0
                    || uv[3] <= 0.0
                {
                    return Err(BackgroundError::Clip);
                }
                let cx = cx.checked_add(origin.0).ok_or(BackgroundError::Geometry)?;
                let cy = cy.checked_add(origin.1).ok_or(BackgroundError::Geometry)?;
                if !exact_rect([cx, cy, cw, ch]) {
                    return Err(BackgroundError::Geometry);
                }
            }
            _ => return Err(BackgroundError::Clip),
        }
        let x = x.checked_add(origin.0).ok_or(BackgroundError::Geometry)?;
        let y = y.checked_add(origin.1).ok_or(BackgroundError::Geometry)?;
        if !exact_rect([x, y, w, h]) {
            return Err(BackgroundError::Geometry);
        }
        let [u, v, uw, vh] = uv;
        let texture = list.intern(self.texture);
        let alpha = if self.alpha as u8 == 255 && list.transparent { 128 } else { self.alpha as u8 };
        list.quads.push(Quad {
            x: x as f32,
            y: y as f32,
            w: w as f32,
            h: h as f32,
            u,
            v,
            uw,
            vh,
            texture,
            color: [1.0, 1.0, 1.0, f32::from(alpha) / 255.0 * opacity],
            rotation: 0.0,
            depth: 0.0,
            additive: false,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTRACT: BackgroundContract = BackgroundContract {
        rect: [0, 20, 600, 411],
        uv_bits: [0, 0, 600.0f32.to_bits(), 411.0f32.to_bits()],
        texture: "L2UI_CH3.PartyMatchWnd.PartyMatch1_Back",
        alpha: 255,
        clip: 1,
        clip_proof: ClipProof::Contained([0, 0, 600, 431]),
        on_paint: 0x100b23b0,
        operation_index: 1,
        helper_return: 0x10012fec,
    };

    #[test]
    fn observed_match_background_preserves_existing_draw_for_origins_and_opacity() {
        for origin in [(0, 0), (100, 50), (-20, -40)] {
            for alpha in [0.0, 0.25, 0.75, 1.0] {
                let mut actual = DrawList::default();
                let mut expected = DrawList::default();
                CONTRACT.paint(&mut actual, origin, alpha).unwrap();
                expected.draw_texture(origin.0, origin.1 + 20, 600, 411, 0, 0, 600, 411, CONTRACT.texture, alpha);
                assert_eq!(actual, expected);
            }
        }
    }

    #[test]
    fn rejects_unproven_clipping_nonfinite_values_and_overflow_without_drawing() {
        let mut invalid = vec![
            BackgroundContract { clip_proof: ClipProof::Unknown, ..CONTRACT },
            BackgroundContract { clip_proof: ClipProof::Disabled, ..CONTRACT },
            BackgroundContract { clip_proof: ClipProof::Contained([0, 0, 599, 431]), ..CONTRACT },
            BackgroundContract { rect: [0, 20, 0, 411], ..CONTRACT },
        ];
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            invalid.push(BackgroundContract { uv_bits: [0, 0, value.to_bits(), 411.0f32.to_bits()], ..CONTRACT });
        }
        let mut list = DrawList::default();
        for contract in invalid {
            assert!(contract.paint(&mut list, (0, 0), 1.0).is_err());
        }
        for opacity in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
            assert!(CONTRACT.paint(&mut list, (0, 0), opacity).is_err());
        }
        assert!(CONTRACT.paint(&mut list, (i32::MAX, 0), 1.0).is_err());
        assert!(list.quads.is_empty());
    }

    #[test]
    fn preserves_fractional_uv_alpha_low_byte_and_disabled_clip() {
        let contract = BackgroundContract {
            uv_bits: [0.25f32.to_bits(), 0.5f32.to_bits(), 600.125f32.to_bits(), 411.25f32.to_bits()],
            alpha: 0x180,
            clip: 0x100,
            clip_proof: ClipProof::Disabled,
            ..CONTRACT
        };
        let mut list = DrawList::default();
        contract.paint(&mut list, (0, 0), 0.75).unwrap();
        let q = &list.quads[0];
        assert_eq!([q.u.to_bits(), q.v.to_bits(), q.uw.to_bits(), q.vh.to_bits()], contract.uv_bits);
        assert_eq!(q.color[3], 128.0 / 255.0 * 0.75);
    }
}
