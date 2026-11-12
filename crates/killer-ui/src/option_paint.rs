use crate::ui_device::TextureInput;
use crate::window_restore::x87_to_int;

pub trait OptionPaintBackend {
    type Texture;
    type Error;
    fn position(&mut self) -> [i32; 2];
    fn size(&mut self) -> [f32; 2];
    fn texture(&mut self, slot: u32) -> Option<Self::Texture>;
    fn set_clip(&mut self, rect: [i32; 4]) -> Result<(), Self::Error>;
    fn begin_origin(&mut self, position: [i32; 2]);
    fn draw_texture(&mut self, input: TextureInput<Self::Texture>, clip: u32) -> Result<(), Self::Error>;
    fn clear_origin(&mut self);
    fn release_clip(&mut self) -> Result<(), Self::Error>;
}

pub fn paint_option<B: OptionPaintBackend>(backend: &mut B) -> Result<bool, B::Error> {
    let [width, height] = backend.size();
    let height = x87_to_int(f64::from(height));
    let width = x87_to_int(f64::from(width));
    let [x, y] = backend.position();
    backend.set_clip([x, y, width, height])?;
    let position = backend.position();
    backend.begin_origin(position);
    if let Some(texture) = backend.texture(0xe8) {
        backend.draw_texture(TextureInput { rect: [0, 20, 256, 28], uv: [0, 0, 256, 28], texture, alpha: 255 }, 1)?;
    }
    if let Some(texture) = backend.texture(0xec) {
        let [width, height] = backend.size();
        let height = x87_to_int(f64::from(height) - 48.0);
        let width = x87_to_int(f64::from(width));
        backend.draw_texture(TextureInput { rect: [0, 48, width, height], uv: [0, 0, width, height], texture, alpha: 255 }, 1)?;
    }
    backend.clear_origin();
    backend.release_clip()?;
    Ok(true)
}

pub struct OptionPaintWindow<'a> {
    pub position: [i32; 2],
    pub size: [f32; 2],
    pub header: Option<&'a str>,
    pub background: Option<&'a str>,
}

pub struct OptionDrawBackend<'a, 't, F> {
    pub window: &'a OptionPaintWindow<'t>,
    pub device: &'a mut crate::ui_device::UiDevice,
    pub canvas: &'a mut crate::canvas_clip::CanvasClip,
    pub base_x: &'a mut i32,
    pub depth: f32,
    pub list: &'a mut crate::draw::DrawList,
    pub resources: F,
}

impl<'t, F: FnMut(&str) -> Option<crate::ui_device::TextureResources>> OptionPaintBackend for OptionDrawBackend<'_, 't, F> {
    type Texture = &'t str;
    type Error = std::convert::Infallible;
    fn position(&mut self) -> [i32; 2] { self.window.position }
    fn size(&mut self) -> [f32; 2] { self.window.size }
    fn texture(&mut self, slot: u32) -> Option<&'t str> {
        match slot { 0xe8 => self.window.header, 0xec => self.window.background, _ => None }
    }
    fn set_clip(&mut self, rect: [i32; 4]) -> Result<(), Self::Error> { self.canvas.set_clip(rect); Ok(()) }
    fn begin_origin(&mut self, position: [i32; 2]) {
        self.device.origin = position;
        *self.base_x = position[0];
        self.device.maximum_y = 0;
        self.device.minimum_y = position[1];
    }
    fn draw_texture(&mut self, input: TextureInput<&'t str>, clip: u32) -> Result<(), Self::Error> {
        let resources = (self.resources)(input.texture);
        let Self { device, canvas, list, depth, .. } = self;
        device.draw_texture(input, resources, clip, |_, call| {
            list.draw_canvas_texture(canvas, *depth, crate::canvas_texture::CanvasTextureInput {
                texture: call.texture, rect: call.rect, uv: input.uv.map(|value| value as f32), alpha: call.alpha, clip,
            });
            Ok(())
        })
    }
    fn clear_origin(&mut self) { self.device.origin = [0, 0]; }
    fn release_clip(&mut self) -> Result<(), Self::Error> { self.canvas.release_clip(); Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Backend { position: [i32; 2], size: [f32; 2], textures: [Option<u32>; 2], calls: Vec<Vec<u32>>, mutate: bool }
    impl OptionPaintBackend for Backend {
        type Texture = u32;
        type Error = ();
        fn position(&mut self) -> [i32; 2] { self.position }
        fn size(&mut self) -> [f32; 2] { self.size }
        fn texture(&mut self, slot: u32) -> Option<u32> { self.textures[usize::from(slot == 0xec)] }
        fn set_clip(&mut self, rect: [i32; 4]) -> Result<(), ()> {
            self.calls.push(vec![0, rect[0] as u32, rect[1] as u32, rect[2] as u32, rect[3] as u32]);
            if self.mutate { self.position = [17, 31]; }
            Ok(())
        }
        fn begin_origin(&mut self, position: [i32; 2]) { self.calls.push(vec![1, position[0] as u32, position[1] as u32]); }
        fn draw_texture(&mut self, input: TextureInput<u32>, clip: u32) -> Result<(), ()> {
            let mut call = vec![2, input.texture, input.alpha, clip]; call.extend(input.rect.map(|v| v as u32)); call.extend(input.uv.map(|v| v as u32)); self.calls.push(call);
            if self.mutate && input.texture == 7 { self.size = [100.0, 48.5]; self.textures[1] = Some(9); }
            Ok(())
        }
        fn clear_origin(&mut self) { self.calls.push(vec![3]); }
        fn release_clip(&mut self) -> Result<(), ()> { self.calls.push(vec![4]); Ok(()) }
    }

    #[test]
    fn container_rereads_state_after_callbacks_and_releases_after_zeroing_origin() {
        let mut b = Backend { position: [3, 5], size: [256.0, 480.0], textures: [Some(7), Some(8)], calls: Vec::new(), mutate: true };
        assert!(paint_option(&mut b).unwrap());
        assert_eq!(b.calls, [vec![0, 3, 5, 256, 480], vec![1, 17, 31], vec![2, 7, 255, 1, 0, 20, 256, 28, 0, 0, 256, 28], vec![2, 9, 255, 1, 0, 48, 100, 0, 0, 0, 100, 0], vec![3], vec![4]]);
    }

    #[test]
    fn missing_textures_skip_draws_but_preserve_clip_and_origin_lifecycle() {
        for mask in 0..4 {
            let mut b = Backend { position: [-3, -5], size: [256.0, 480.0], textures: [(mask & 1 != 0).then_some(7), (mask & 2 != 0).then_some(8)], calls: Vec::new(), mutate: false };
            assert!(paint_option(&mut b).unwrap());
            assert_eq!(b.calls.iter().filter(|c| c[0] == 2).count(), (mask as u32).count_ones() as usize);
            assert_eq!(&b.calls[b.calls.len() - 2..], [vec![3], vec![4]]);
        }
    }

    #[test]
    fn concrete_paint_uses_native_device_canvas_and_draw_list_without_painting_children() {
        use crate::canvas_clip::CanvasClip;
        use crate::draw::DrawList;
        use crate::ui_device::{TextureResources, UiDevice};
        for state in [0, 1] {
            for missing_resource in [false, true] {
                let window = OptionPaintWindow { position: [3, 5], size: [256.0, 480.0], header: Some(crate::option::HEADER), background: Some(crate::option::GAME_BACK) };
                let mut device = UiDevice { origin: [100, 100], minimum_y: -9, maximum_y: -9, transparency_state: state };
                let mut canvas = CanvasClip { flags: 0x100, origin: [9.0; 2], size: [9.0; 2], cursor: [9.0; 2], saved_cursor: [9.0; 2], saved_origin: [9.0; 2], saved_size: [9.0; 2] };
                let mut list = DrawList::default();
                let mut base_x = 999;
                let mut backend = OptionDrawBackend { window: &window, device: &mut device, canvas: &mut canvas, base_x: &mut base_x, depth: 0.0, list: &mut list, resources: |_: &str| (!missing_resource).then_some(TextureResources { primary: true, secondary: false }) };
                assert!(paint_option(&mut backend).unwrap());
                assert_eq!(list.quads.len(), if missing_resource { 0 } else { 2 });
                if !missing_resource {
                    assert_eq!([list.quads[0].x, list.quads[0].y, list.quads[0].w, list.quads[0].h], [3.0, 25.0, 256.0, 28.0]);
                    assert_eq!([list.quads[1].x, list.quads[1].y, list.quads[1].w, list.quads[1].h], [3.0, 53.0, 256.0, 432.0]);
                    assert_eq!(list.quads[1].color[3], if state == 0 { 1.0 } else { 128.0 / 255.0 });
                }
                assert_eq!((device.origin, device.minimum_y, device.maximum_y, base_x), ([0, 0], 5, if missing_resource { 0 } else { 485 }, 3));
                assert_eq!((canvas.flags, canvas.origin, canvas.size, canvas.cursor), (0x100, [3.0, 5.0], [256.0, 480.0], [0.0; 2]));
            }
        }
    }
}
