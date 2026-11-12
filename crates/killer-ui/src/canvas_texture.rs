use crate::canvas_clip::CanvasClip;
use crate::window_restore::x87_to_int;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasTile<T> {
    pub texture: T,
    pub rect: [f32; 4],
    pub uv: [f32; 4],
    pub depth: f32,
    pub color: [f32; 4],
    pub secondary_color: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasTextureInput<T> {
    pub texture: T,
    pub rect: [i32; 4],
    pub uv: [f32; 4],
    pub alpha: u32,
    pub clip: u32,
}

fn x87_result(left: f64, right: f64, result: f64) -> f64 {
    if result.is_nan() && !left.is_nan() && !right.is_nan() { f64::from_bits(0xfff8000000000000) } else { result }
}

fn cropped_uv(extent: f32, size: i32, cut: i32) -> f64 {
    let (extent, size, cut) = (f64::from(extent), f64::from(size), f64::from(cut));
    let ratio = x87_result(extent, size, extent / size);
    x87_result(ratio, cut, ratio * cut)
}

fn remaining_uv(extent: f32, cropped: f64) -> f32 {
    let extent = f64::from(extent);
    x87_result(extent, cropped, extent - cropped) as f32
}

pub fn texture_tile<T>(canvas: &CanvasClip, depth: f32, input: CanvasTextureInput<T>) -> Option<CanvasTile<T>> {
    let [mut x, mut y, mut width, mut height] = input.rect;
    let [mut u, mut v, mut uw, mut vh] = input.uv;
    if width <= 0 || height <= 0 { return None; }
    if input.clip as u8 != 0 {
        let [ox, oy] = canvas.origin;
        let right = (f64::from(ox) + f64::from(canvas.size[0])) as f32;
        let bottom = (f64::from(oy) + f64::from(canvas.size[1])) as f32;
        let xf = x as f32;
        let yf = y as f32;
        if xf > right || yf > bottom { return None; }
        if xf < ox {
            let cut = x87_to_int(f64::from(ox) - f64::from(xf));
            let cropped = cropped_uv(uw, width, cut);
            width = width.wrapping_sub(cut);
            u = cropped as f32;
            uw = remaining_uv(uw, cropped);
            x = x87_to_int(f64::from(ox));
        }
        if f64::from(x.wrapping_add(width)) > f64::from(right) {
            let cut = x87_to_int(f64::from(x.wrapping_add(width)) - f64::from(right));
            let cropped = cropped_uv(uw, width, cut);
            width = width.wrapping_sub(cut);
            uw = remaining_uv(uw, cropped);
        }
        if f64::from(y.wrapping_add(height)) < f64::from(oy) { return None; }
        if yf < oy {
            let cut = x87_to_int(f64::from(oy) - f64::from(yf));
            let cropped = cropped_uv(vh, height, cut);
            height = height.wrapping_sub(cut);
            v = cropped as f32;
            vh = remaining_uv(vh, cropped);
            y = x87_to_int(f64::from(oy));
        }
        if f64::from(y.wrapping_add(height)) > f64::from(bottom) {
            let cut = x87_to_int(f64::from(y.wrapping_add(height)) - f64::from(bottom));
            let cropped = cropped_uv(vh, height, cut);
            height = height.wrapping_sub(cut);
            vh = remaining_uv(vh, cropped);
        }
        if width <= 0 || height <= 0 || uw <= 0.0 || vh <= 0.0 || u < 0.0 || v < 0.0 { return None; }
    }
    Some(CanvasTile {
        texture: input.texture, rect: [x as f32, y as f32, width as f32, height as f32], uv: [u, v, uw, vh],
        depth, color: [255.0, 255.0, 255.0, f32::from(input.alpha as u8)], secondary_color: [0.0; 4],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn canvas() -> CanvasClip { CanvasClip { flags: 0, origin: [10.0, 20.0], size: [30.0, 40.0], cursor: [0.0; 2], saved_cursor: [0.0; 2], saved_origin: [0.0; 2], saved_size: [0.0; 2] } }

    #[test]
    fn clip_replaces_uv_origin_and_trims_all_edges_without_using_active_flag() {
        let tile = texture_tile(&canvas(), 0.5, CanvasTextureInput { texture: 7, rect: [0, 0, 100, 100], uv: [99.0, 98.0, 200.0, 300.0], alpha: 0x180, clip: 1 }).unwrap();
        assert_eq!(tile.rect, [10.0, 20.0, 30.0, 40.0]);
        assert_eq!(tile.uv, [20.0, 60.0, 60.0, 120.0]);
        assert_eq!(tile.color, [255.0, 255.0, 255.0, 128.0]);
    }

    #[test]
    fn clip_byte_controls_uv_validation_but_dimensions_always_reject() {
        let input = CanvasTextureInput { texture: (), rect: [10, 20, 5, 5], uv: [-1.0, -1.0, -2.0, -2.0], alpha: 255, clip: 0x100 };
        assert!(texture_tile(&canvas(), 0.0, input).is_some());
        assert!(texture_tile(&canvas(), 0.0, CanvasTextureInput { clip: 1, ..input }).is_none());
        assert!(texture_tile(&canvas(), 0.0, CanvasTextureInput { rect: [10, 20, 0, 5], ..input }).is_none());
        let nan = f32::from_bits(0x7fc12345);
        assert!(texture_tile(&canvas(), 0.0, CanvasTextureInput { uv: [nan; 4], clip: 1, ..input }).is_some());
    }

    #[test]
    fn invalid_uv_operations_keep_x87_negative_indefinite_bits() {
        let tile = texture_tile(&canvas(), 0.5, CanvasTextureInput { texture: (), rect: [0, 0, 100, 100], uv: [0.0, 0.0, f32::INFINITY, f32::INFINITY], alpha: 255, clip: 1 }).unwrap();
        assert_eq!(tile.uv.map(f32::to_bits), [0x7f800000, 0x7f800000, 0xffc00000, 0xffc00000]);
        assert_eq!(cropped_uv(f32::INFINITY, 100, 0).to_bits(), 0xfff8000000000000);
        assert_eq!(cropped_uv(0.0, 0, 1).to_bits(), 0xfff8000000000000);
    }
}
