#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureResources {
    pub primary: bool,
    pub secondary: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureInput<T> {
    pub rect: [i32; 4],
    pub uv: [i32; 4],
    pub texture: T,
    pub alpha: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CanvasUv {
    Texture([f32; 4]),
    Stretched([i32; 4]),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasTexture<T> {
    pub rect: [i32; 4],
    pub uv: CanvasUv,
    pub texture: T,
    pub alpha: u32,
    pub clip: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiDevice {
    pub origin: [i32; 2],
    pub minimum_y: i32,
    pub maximum_y: i32,
    pub transparency_state: u32,
}

impl UiDevice {
    pub fn draw_texture<T, E>(
        &mut self,
        input: TextureInput<T>,
        resources: Option<TextureResources>,
        clip: u32,
        canvas: impl FnOnce(&mut Self, CanvasTexture<T>) -> Result<(), E>,
    ) -> Result<(), E> {
        self.draw(input, resources, Some(clip), canvas)
    }

    pub fn draw_stretched_texture<T, E>(
        &mut self,
        input: TextureInput<T>,
        resources: Option<TextureResources>,
        canvas: impl FnOnce(&mut Self, CanvasTexture<T>) -> Result<(), E>,
    ) -> Result<(), E> {
        self.draw(input, resources, None, canvas)
    }

    fn draw<T, E>(
        &mut self,
        input: TextureInput<T>,
        resources: Option<TextureResources>,
        clip: Option<u32>,
        canvas: impl FnOnce(&mut Self, CanvasTexture<T>) -> Result<(), E>,
    ) -> Result<(), E> {
        if !resources.is_some_and(|resources| resources.primary || resources.secondary) { return Ok(()); }
        let [x, y, width, height] = input.rect;
        let alpha = if input.alpha as u8 == 255 && self.transparency_state != 0 { 128 } else { input.alpha };
        let uv = if clip.is_some() { CanvasUv::Texture(input.uv.map(|value| value as f32)) } else { CanvasUv::Stretched(input.uv) };
        canvas(self, CanvasTexture {
            rect: [self.origin[0].wrapping_add(x), self.origin[1].wrapping_add(y), width, height],
            uv, texture: input.texture, alpha, clip,
        })?;
        let top = self.origin[1].wrapping_add(y);
        let bottom = top.wrapping_add(height);
        if bottom > self.maximum_y { self.maximum_y = bottom; }
        if top < self.minimum_y { self.minimum_y = top; }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_guards_alpha_byte_and_uv_conversion_cover_both_wrappers() {
        for stretched in [false, true] {
            for state in [0, 1, u32::MAX] {
                for alpha in (0..256).chain([0x100, 0x1ff, u32::MAX]) {
                    for mask in 0..5 {
                        let mut device = UiDevice { origin: [i32::MAX, 20], minimum_y: 50, maximum_y: -50, transparency_state: state };
                        let before = device;
                        let resources = (mask != 0).then_some(TextureResources { primary: mask & 1 != 0, secondary: mask & 2 != 0 });
                        let input = TextureInput { rect: [1, -10, 0, -5], uv: [i32::MIN, i32::MAX, 16777217, -16777217], texture: 7, alpha };
                        let mut calls = 0;
                        let callback = |device: &mut UiDevice, call: CanvasTexture<u32>| {
                            calls += 1;
                            assert_eq!(call.rect, [i32::MIN, 10, 0, -5]);
                            assert_eq!(call.alpha, if alpha as u8 == 255 && state != 0 { 128 } else { alpha });
                            assert_eq!(call.uv, if stretched { CanvasUv::Stretched(input.uv) } else { CanvasUv::Texture(input.uv.map(|v| v as f32)) });
                            assert_eq!(call.clip, (!stretched).then_some(123));
                            device.origin[1] = 40;
                            Ok::<_, ()>(())
                        };
                        if stretched { device.draw_stretched_texture(input, resources, callback).unwrap(); }
                        else { device.draw_texture(input, resources, 123, callback).unwrap(); }
                        if matches!(mask, 1..=3) {
                            assert_eq!(calls, 1); assert_eq!((device.minimum_y, device.maximum_y), (30, 25));
                        } else { assert_eq!(calls, 0); assert_eq!(device, before); }
                    }
                }
            }
        }
    }

    #[test]
    fn bounds_use_signed_wrapping_and_callback_error_does_not_update_them() {
        let mut device = UiDevice { origin: [0, i32::MAX], minimum_y: 0, maximum_y: 0, transparency_state: 0 };
        let input = TextureInput { rect: [0, 1, -1, -1], uv: [0; 4], texture: (), alpha: 255 };
        let resources = Some(TextureResources { primary: true, secondary: false });
        device.draw_texture(input, resources, 0, |_, _| Ok::<_, ()>(())).unwrap();
        assert_eq!((device.minimum_y, device.maximum_y), (i32::MIN, i32::MAX));
        let before = device;
        assert_eq!(device.draw_texture(input, resources, 0, |_, _| Err(9)), Err(9));
        assert_eq!(device, before);
    }
}
