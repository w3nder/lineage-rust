use crate::color::Argb;
use crate::font::BitmapFont;
use std::sync::Arc;

pub const DIGIT_TEXTURE: &str = "L2UI.NWindow.Number";
pub const DIGIT_CELL: i32 = 8;
pub const DIGIT_CELLS: &str = "0123456789HMP/%.C:";
pub const LINE_TEXTURE: &str = "L2UI.SquareWhite";
pub const XLARGE_DIGIT_TEXTURE: &str = "L2UI_CH3.Etc.XLargeNumber";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DigitStrip {
    pub texture: &'static str,
    pub cell_width: i32,
    pub cell_height: i32,
}

pub const NUMBER_STRIP: DigitStrip =
    DigitStrip { texture: DIGIT_TEXTURE, cell_width: DIGIT_CELL, cell_height: DIGIT_CELL };
pub const XLARGE_NUMBER_STRIP: DigitStrip =
    DigitStrip { texture: XLARGE_DIGIT_TEXTURE, cell_width: 11, cell_height: 14 };

pub fn digits_width(text: &str) -> i32 {
    digits_width_of(NUMBER_STRIP, text)
}

pub fn digits_width_of(strip: DigitStrip, text: &str) -> i32 {
    text.chars().filter(|c| DIGIT_CELLS.contains(*c)).count() as i32 * strip.cell_width
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Texture(Arc<str>);

impl Texture {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for Texture {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&*self.0, f)
    }
}

impl std::fmt::Display for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for Texture {
    fn from(value: &str) -> Self {
        Self(Arc::from(value))
    }
}

impl From<String> for Texture {
    fn from(value: String) -> Self {
        Self(Arc::from(value))
    }
}

impl PartialEq<str> for Texture {
    fn eq(&self, other: &str) -> bool {
        &*self.0 == other
    }
}

impl PartialEq<&str> for Texture {
    fn eq(&self, other: &&str) -> bool {
        &*self.0 == *other
    }
}

impl PartialEq<String> for Texture {
    fn eq(&self, other: &String) -> bool {
        &*self.0 == other.as_str()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Quad {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub u: f32,
    pub v: f32,
    pub uw: f32,
    pub vh: f32,
    pub texture: Texture,
    pub color: [f32; 4],
    pub rotation: f32,
    pub depth: f32,
    pub additive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextRun {
    pub quads: std::ops::Range<usize>,
    pub text: String,
    pub rect: [f32; 4],
    pub color: [f32; 4],
    pub clip: Option<crate::window::Rect>,
}

const INTERNED_TEXTURES: usize = 16;

pub fn is_frame_close(texture: &str) -> bool {
    let name = texture.rsplit('.').next().unwrap_or(texture).to_ascii_lowercase();
    name == "frameclosebtn" || name == "framecloseonbtn"
}

pub const SILENT_PRESS_TEXTURES: &[&str] = &["l2ui_ch3.menuicon.menubutton1"];

pub fn is_silent_press(texture: &str) -> bool {
    SILENT_PRESS_TEXTURES.iter().any(|silent| texture.eq_ignore_ascii_case(silent))
}

pub fn button_face(texture: &str) -> Option<bool> {
    let name = texture.rsplit('.').next().unwrap_or(texture).to_ascii_lowercase();
    let face = (name.contains("btn") || name.contains("button")) && !["back", "glow", "radio"].iter().any(|skip| name.contains(skip));
    face.then(|| !name.contains("disable"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonRegion {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub enabled: bool,
    pub quad: usize,
    pub close: bool,
    pub silent: bool,
}

impl ButtonRegion {
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextAnim {
    pub per_glyph: f32,
    pub elapsed: f32,
    pub count: u32,
    pub finished: bool,
}

impl TextAnim {
    pub fn new(per_glyph: f32, elapsed: f32) -> TextAnim {
        TextAnim { per_glyph, elapsed, count: 0, finished: false }
    }

    pub fn begin_string(&mut self) {
        self.finished = false;
    }

    pub fn glyph_blocked(&mut self) -> bool {
        let due = f64::from(self.count) * f64::from(self.per_glyph);
        self.count += 1;
        due > f64::from(self.elapsed)
    }
}

#[derive(Default, Debug, Clone)]
pub struct DrawList {
    pub text_anim: Option<TextAnim>,
    pub quads: Vec<Quad>,
    pub text_runs: Vec<TextRun>,
    pub capture_text: bool,
    pub interned: Vec<Texture>,
    pub buttons: Vec<ButtonRegion>,
    pub radar_at: Option<usize>,
    pub transparent: bool,
}

impl PartialEq for DrawList {
    fn eq(&self, other: &Self) -> bool {
        self.quads == other.quads && self.text_runs == other.text_runs && self.capture_text == other.capture_text
    }
}

pub const TRANSPARENT_ALPHA: f32 = 128.0 / 255.0;

impl DrawList {
    fn window_alpha(&self, alpha: f32) -> f32 {
        if self.transparent && alpha == 1.0 { TRANSPARENT_ALPHA } else { alpha }
    }

    pub fn intern(&mut self, name: &str) -> Texture {
        if let Some(found) = self.interned.iter().find(|texture| texture.as_str() == name) {
            return found.clone();
        }
        let texture = Texture::from(name);
        if self.interned.len() < INTERNED_TEXTURES {
            self.interned.push(texture.clone());
        } else {
            let slot = self.quads.len() % INTERNED_TEXTURES;
            self.interned[slot] = texture.clone();
        }
        texture
    }

    pub fn register_button(&mut self, x: i32, y: i32, w: i32, h: i32, enabled: bool) {
        self.register((x, y, w, h), enabled, false, false);
    }

    fn register(&mut self, (x, y, w, h): (i32, i32, i32, i32), enabled: bool, close: bool, silent: bool) {
        let len = self.quads.len();
        if let Some(last) = self.buttons.last_mut().filter(|last| (last.x, last.y, last.w, last.h) == (x, y, w, h) && last.quad + 1 >= len) {
            last.close |= close;
            last.silent |= silent;
            return;
        }
        self.buttons.push(ButtonRegion { x, y, w, h, enabled, quad: len, close, silent });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_button_texture(&mut self, x: i32, y: i32, w: i32, h: i32, u: i32, v: i32, uw: i32, vh: i32, texture: &str, alpha: f32, enabled: bool) {
        self.register_button(x, y, w, h, enabled);
        self.draw_texture(x, y, w, h, u, v, uw, vh, texture, alpha);
    }

    pub fn append_clipped(&mut self, body:Self, clip:crate::window::Rect) {
        let base=self.quads.len();
        let mut mapping=Vec::with_capacity(body.quads.len()+1);
        for mut q in body.quads {
            mapping.push(self.quads.len()-base);
            let left=q.x.max(clip.x as f32);
            let top=q.y.max(clip.y as f32);
            let right=(q.x+q.w).min((clip.x+clip.w) as f32);
            let bottom=(q.y+q.h).min((clip.y+clip.h) as f32);
            if right<=left || bottom<=top {continue;}
            if left!=q.x || top!=q.y || right!=q.x+q.w || bottom!=q.y+q.h {
                let (us,vs)=(q.uw/q.w,q.vh/q.h);
                q.u+=(left-q.x)*us;q.v+=(top-q.y)*vs;
                (q.x,q.y,q.w,q.h)=(left,top,right-left,bottom-top);
                (q.uw,q.vh)=(q.w*us,q.h*vs);
            }
            self.quads.push(q);
        }
        mapping.push(self.quads.len()-base);
        for button in &body.buttons {
            let left=button.x.max(clip.x);
            let top=button.y.max(clip.y);
            let right=(button.x+button.w).min(clip.x+clip.w);
            let bottom=(button.y+button.h).min(clip.y+clip.h);
            if right<=left || bottom<=top {continue;}
            let quad=base+mapping.get(button.quad).copied().unwrap_or(self.quads.len()-base);
            self.buttons.push(ButtonRegion{x:left,y:top,w:right-left,h:bottom-top,enabled:button.enabled,quad,close:button.close,silent:button.silent});
        }
        for mut run in body.text_runs {
            let Some((&start,&end))=mapping.get(run.quads.start).zip(mapping.get(run.quads.end)) else {continue;};
            if start==end {continue;}
            run.quads=(base+start)..(base+end);
            run.clip=Some(if let Some(previous)=run.clip {
                let (x,y)=(clip.x.max(previous.x),clip.y.max(previous.y));
                crate::window::Rect::new(x,y,(clip.x+clip.w).min(previous.x+previous.w)-x,(clip.y+clip.h).min(previous.y+previous.h)-y)
            }else{clip});
            self.text_runs.push(run);
        }
    }

    pub fn clip_from(&mut self,start:usize,clip:crate::window::Rect) {
        let split=self.text_runs.partition_point(|run|run.quads.start<start);
        let moved=self.buttons.partition_point(|button|button.quad<start);
        let mut buttons=self.buttons.split_off(moved);
        for button in &mut buttons {button.quad-=start;}
        let mut body=Self {quads:self.quads.split_off(start),text_runs:self.text_runs.split_off(split),capture_text:self.capture_text,interned:Vec::new(),buttons,radar_at:None,transparent:self.transparent,text_anim:None};
        for run in &mut body.text_runs {run.quads=(run.quads.start-start)..(run.quads.end-start);}
        self.append_clipped(body,clip);
    }

    pub fn draw_canvas_texture(&mut self, canvas: &crate::canvas_clip::CanvasClip, depth: f32, input: crate::canvas_texture::CanvasTextureInput<&str>) {
        let Some(tile) = crate::canvas_texture::texture_tile(canvas, depth, input) else { return };
        let [x, y, w, h] = tile.rect;
        let [u, v, uw, vh] = tile.uv;
        let texture = self.intern(tile.texture);
        self.quads.push(Quad {
            x, y, w, h, u, v, uw, vh, texture,
            color: {
                let mut color = tile.color.map(|channel| channel / 255.0);
                color[3] = self.window_alpha(color[3]);
                color
            },
            rotation: 0.0, depth: 0.0, additive: false,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_texture(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        u: i32,
        v: i32,
        uw: i32,
        vh: i32,
        texture: &str,
        alpha: f32,
    ) {
        if let Some(enabled) = button_face(texture).or_else(|| crate::button_pushed::pushed_face(texture).map(|_| true)) {
            self.register((x, y, w, h), enabled, is_frame_close(texture), is_silent_press(texture));
        }
        let alpha = self.window_alpha(alpha);
        self.push(x, y, w, h, u, v, uw, vh, texture, [1.0, 1.0, 1.0, alpha]);
    }

    pub fn draw_brighten_texture(&mut self, x: i32, y: i32, size_w: i32, size_h: i32, texture: &str, tint: Argb) {
        let [r, g, b, a] = tint.rgba();
        let alpha = self.window_alpha(a);
        self.push(x, y, size_w, size_h, 0, 0, size_w, size_h, texture, [r, g, b, alpha]);
        if let Some(quad) = self.quads.last_mut() {
            quad.additive = true;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_texture_rotated(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        uw: i32,
        vh: i32,
        texture: &str,
        rotation: f32,
        alpha: f32,
    ) {
        let alpha = self.window_alpha(alpha);
        self.push(x, y, w, h, 0, 0, uw, vh, texture, [1.0, 1.0, 1.0, alpha]);
        if let Some(quad) = self.quads.last_mut() {
            quad.rotation = rotation;
        }
    }

    pub fn draw_true_text(&mut self, x: i32, y: i32, color: Argb, text: &str, font: &crate::true_font::TrueFont) -> i32 {
        let rgba = color.rgba();
        let mut cursor_y = y;
        let mut widest = 0;
        for line in text.lines() {
            let (width, height) = font.face.bitmap_size(font.style, line);
            if font.face.text_width(font.style, line) > 0 {
                self.push(x, cursor_y, width, height, 0, 0, width, height, &killer_font::texture_name(font.slot, line), rgba);
                if self.capture_text {
                    let quad = self.quads.len() - 1;
                    self.text_runs.push(TextRun { quads: quad..quad + 1, text: line.to_string(), rect: [x as f32, cursor_y as f32, width as f32, height as f32], color: rgba, clip: None });
                }
            }
            widest = widest.max(width);
            cursor_y += font.face.line_height(font.style);
        }
        x + widest
    }

    pub fn draw_text(&mut self, x: i32, y: i32, color: Argb, text: &str, font: &BitmapFont) -> i32 {
        self.draw_text_clipped(x, y, i32::MAX, color, text, font)
    }

    pub fn draw_text_clipped(
        &mut self,
        x: i32,
        y: i32,
        max_width: i32,
        color: Argb,
        text: &str,
        font: &BitmapFont,
    ) -> i32 {
        let start = self.quads.len();
        let mut visible = String::new();
        let rgba = color.rgba();
        let limit = x.saturating_add(max_width);
        let mut cursor_x = x;
        let mut cursor_y = y;
        let mut blocked = false;
        if let Some(anim) = self.text_anim.as_mut() {
            anim.begin_string();
        }
        for c in text.chars() {
            if self.text_anim.as_mut().is_some_and(TextAnim::glyph_blocked) {
                blocked = true;
                break;
            }
            if c == '\n' {
                if self.capture_text { visible.push(c); }
                cursor_x = x;
                cursor_y += font.line_height as i32;
                continue;
            }
            let Some(glyph) = font.glyph(c) else { continue };
            let (w, h) = (glyph.width as i32, glyph.height as i32);
            if cursor_x + w > limit {
                continue;
            }
            if self.capture_text { visible.push(c); }
            if c != ' ' {
                self.push(cursor_x, cursor_y, w, h, glyph.x, glyph.y as i32, w, h, &font.atlas, rgba);
            }
            cursor_x += w;
        }
        if !blocked && let Some(anim) = self.text_anim.as_mut() {
            anim.finished = true;
        }
        if self.capture_text && start < self.quads.len() {
            self.text_runs.push(TextRun { quads:start..self.quads.len(),text:visible,rect:[x as f32,y as f32,(cursor_x-x) as f32,(cursor_y-y+font.line_height as i32) as f32],color:rgba,clip:None });
        }
        cursor_x
    }

    pub fn draw_hline(&mut self, x: i32, y: i32, w: i32, color: Argb) {
        if w > 0 {
            self.push(x, y, w, 1, 0, 0, 1, 1, LINE_TEXTURE, color.rgba());
        }
    }

    pub fn draw_fill(&mut self, x: i32, y: i32, w: i32, h: i32, color: Argb) {
        if w > 0 && h > 0 {
            self.push(x, y, w, h, 0, 0, 1, 1, LINE_TEXTURE, color.rgba());
        }
    }

    pub fn draw_vline(&mut self, x: i32, y: i32, h: i32, color: Argb) {
        if h > 0 {
            self.push(x, y, 1, h, 0, 0, 1, 1, LINE_TEXTURE, color.rgba());
        }
    }

    pub fn draw_digits(&mut self, x: i32, y: i32, text: &str, color: Argb) {
        self.draw_digits_with(NUMBER_STRIP, x, y, text, color);
    }

    pub fn draw_digits_with(&mut self, strip: DigitStrip, x: i32, y: i32, text: &str, color: Argb) {
        let start=self.quads.len();
        let mut visible=String::new();
        let rgba = color.rgba();
        let (cw, ch) = (strip.cell_width, strip.cell_height);
        let mut cursor_x = x;
        for c in text.chars() {
            let Some(cell) = DIGIT_CELLS.find(c) else { continue };
            if self.capture_text { visible.push(c); }
            self.push(cursor_x, y, cw, ch, cell as i32 * cw, 0, cw, ch, strip.texture, rgba);
            cursor_x += cw;
        }
        if self.capture_text && start < self.quads.len() {
            self.text_runs.push(TextRun { quads:start..self.quads.len(),text:visible,rect:[x as f32,y as f32,(cursor_x-x) as f32,ch as f32],color:rgba,clip:None });
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn slices3_h(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        left: (&str, i32),
        mid: &str,
        right: (&str, i32),
        vh: i32,
        alpha: f32,
    ) {
        let (left_tex, left_w) = left;
        let (right_tex, right_w) = right;
        let mid_w = (w - left_w - right_w).max(0);
        self.draw_texture(x, y, left_w, h, 0, 0, left_w, vh, left_tex, alpha);
        if mid_w > 0 {
            self.draw_texture(x + left_w, y, mid_w, h, 0, 0, left_w, vh, mid, alpha);
        }
        self.draw_texture(x + w - right_w, y, right_w, h, 0, 0, right_w, vh, right_tex, alpha);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn slices3_v(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        top: (&str, i32),
        mid: &str,
        bottom: (&str, i32),
        uw: i32,
        alpha: f32,
    ) {
        let (top_tex, top_h) = top;
        let (bottom_tex, bottom_h) = bottom;
        let mid_h = (h - top_h - bottom_h).max(0);
        self.draw_texture(x, y, w, top_h, 0, 0, uw, top_h, top_tex, alpha);
        if mid_h > 0 {
            self.draw_texture(x, y + top_h, w, mid_h, 0, 0, uw, top_h, mid, alpha);
        }
        self.draw_texture(x, y + h - bottom_h, w, bottom_h, 0, 0, uw, bottom_h, bottom_tex, alpha);
    }

    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        u: i32,
        v: i32,
        uw: i32,
        vh: i32,
        texture: &str,
        color: [f32; 4],
    ) {
        let texture = self.intern(texture);
        self.quads.push(Quad {
            x: x as f32,
            y: y as f32,
            w: w as f32,
            h: h as f32,
            u: u as f32,
            v: v as f32,
            uw: uw as f32,
            vh: vh as f32,
            texture,
            color,
            rotation: 0.0, depth: 0.0, additive: false,
        });
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_the_character_button_of_the_menu_bar_presses_without_the_click_sound() {
        let mut list = DrawList::default();
        for (index, name) in ["MenuButton1", "MenuButton2", "MenuButton3", "MenuButton4"].iter().enumerate() {
            list.draw_texture(index as i32 * 37, 6, 34, 34, 0, 0, 34, 34, &format!("L2UI_ch3.MenuIcon.{name}"), 1.0);
        }
        assert_eq!(list.buttons.iter().map(|button| button.silent).collect::<Vec<_>>(), [true, false, false, false]);
    }

    #[test]
    fn button_faces_are_recognised_by_the_texture_name_of_the_nwindow_list() {
        assert_eq!(button_face("L2UI_CH3.Button.BigButton_down"), Some(true));
        assert_eq!(button_face("L2UI_CH3.FrameCtrl.FrameCloseBtn"), Some(true));
        assert_eq!(button_face("L2UI_CH3.Button.btn1_normalDisable"), Some(false));
        assert_eq!(button_face("L2UI_CH3.LobbyWnd.lobby_startbtnback"), None);
        assert_eq!(button_face("l2ui_ch3.etc.glowbtn1"), None);
        assert_eq!(button_face("L2UI_CH3.RadioCtrl.RadioButton1"), None);
        assert_eq!(button_face("L2UI_CH3.ScrollBar.ScrollBarUpBtn"), Some(true));
        let mut list = DrawList::default();
        list.draw_texture(10, 20, 30, 40, 0, 0, 30, 40, "L2UI_CH3.Button.BigButton", 1.0);
        list.draw_button_texture(10, 20, 30, 40, 0, 0, 30, 40, "L2UI_CH3.Button.BigButton_down", 1.0, true);
        assert_eq!(list.buttons.len(), 1);
        list.draw_texture(60, 0, 15, 15, 0, 0, 15, 15, "L2UI_CH3.FrameCtrl.FrameCloseBtn", 1.0);
        assert!(list.buttons[1].close && !list.buttons[0].close);
        assert!(list.buttons[0].contains(10, 20) && !list.buttons[0].contains(40, 20));
    }

    use super::*;
    use crate::color::TEXT;
    use crate::font::{Glyphs, gly};

    #[test]
    fn captured_text_keeps_ranges_and_clip_when_scrolling_discards_glyphs() {
        let font=BitmapFont {atlas:"font".into(),glyphs:Glyphs::parse(&gly(&[(0x20,&[[0,3,0,13],[4,6,0,13]])],(1024,128))).unwrap(),line_height:13};
        let mut list=DrawList {capture_text:true,..Default::default()};
        list.draw_text(0,0,TEXT,"!!!",&font);
        list.draw_text(0,20,TEXT,"!!",&font);
        list.clip_from(0,crate::window::Rect::new(7,0,9,14));
        assert_eq!(list.quads.len(),2);
        assert_eq!(list.text_runs.len(),1);
        assert_eq!(list.text_runs[0].quads,0..2);
        assert_eq!(list.text_runs[0].text,"!!!");
        assert_eq!(list.text_runs[0].rect,[0.0,0.0,18.0,13.0]);
        assert_eq!(list.text_runs[0].clip,Some(crate::window::Rect::new(7,0,9,14)));
        assert_eq!((list.quads[0].x,list.quads[0].w),(7.0,5.0));
        list.draw_text(0,30,TEXT,"!",&font);
        assert_eq!(list.text_runs[1].quads,2..3);
    }

    #[test]
    fn native_device_and_canvas_path_keeps_clipped_uv_and_applies_transparency_once() {
        use crate::canvas_clip::CanvasClip;
        use crate::canvas_texture::CanvasTextureInput;
        use crate::ui_device::{CanvasUv, TextureInput, TextureResources, UiDevice};
        for transparency_state in [0, 1] {
            for alpha in [0, 64, 128, 255] {
                let mut list = DrawList::default();
                let mut canvas = CanvasClip { flags: 0, origin: [0.0; 2], size: [0.0; 2], cursor: [0.0; 2], saved_cursor: [0.0; 2], saved_origin: [0.0; 2], saved_size: [0.0; 2] };
                canvas.set_clip([10, 20, 30, 40]);
                let mut device = UiDevice { origin: [0, 0], minimum_y: 0, maximum_y: 0, transparency_state };
                device.draw_texture(TextureInput { rect: [0, 0, 100, 100], uv: [99, 98, 200, 300], texture: "original.texture", alpha }, Some(TextureResources { primary: true, secondary: false }), 1, |_, call| {
                    let CanvasUv::Texture(uv) = call.uv else { panic!("wrong canvas slot") };
                    list.draw_canvas_texture(&canvas, 0.0, CanvasTextureInput { texture: call.texture, rect: call.rect, uv, alpha: call.alpha, clip: call.clip.unwrap() });
                    Ok::<_, ()>(())
                }).unwrap();
                assert_eq!(list.quads.len(), 1);
                let quad = &list.quads[0];
                assert_eq!([quad.x, quad.y, quad.w, quad.h], [10.0, 20.0, 30.0, 40.0]);
                assert_eq!([quad.u, quad.v, quad.uw, quad.vh], [20.0, 60.0, 60.0, 120.0]);
                assert_eq!(quad.color, [1.0, 1.0, 1.0, if transparency_state != 0 && alpha == 255 { 128.0 / 255.0 } else { alpha as f32 / 255.0 }]);
                assert_eq!((device.minimum_y, device.maximum_y), (0, 100));
            }
        }
    }

    #[test]
    fn rejected_canvas_texture_does_not_insert_a_quad() {
        let canvas = crate::canvas_clip::CanvasClip { flags: 0, origin: [10.0, 20.0], size: [30.0, 40.0], cursor: [0.0; 2], saved_cursor: [0.0; 2], saved_origin: [0.0; 2], saved_size: [0.0; 2] };
        let mut list = DrawList::default();
        list.draw_canvas_texture(&canvas, 0.0, crate::canvas_texture::CanvasTextureInput { texture: "original.texture", rect: [100, 100, 20, 20], uv: [0.0, 0.0, 20.0, 20.0], alpha: 255, clip: 1 });
        assert!(list.quads.is_empty());
    }

    #[test]
    fn draw_text_emits_one_quad_per_glyph_with_texel_uvs() {
        let bytes = gly(&[(0x20, &[[0, 3, 0, 13], [4, 6, 0, 13]])], (1024, 128));
        let font = BitmapFont { atlas: "a".into(), glyphs: Glyphs::parse(&bytes).unwrap(), line_height: 13 };
        let mut list = DrawList::default();
        let end = list.draw_text(10, 20, TEXT, "! !", &font);
        assert_eq!(end, 10 + 15);
        assert_eq!(list.quads.len(), 2);
        let q = &list.quads[1];
        assert_eq!((q.x, q.y, q.w, q.h, q.u, q.v, q.uw, q.vh), (19.0, 20.0, 6.0, 13.0, 4.0, 0.0, 6.0, 13.0));
        assert_eq!(q.color, TEXT.rgba());
    }

    #[test]
    fn slices3_h_stretches_the_middle_over_the_tile_uv() {
        let mut list = DrawList::default();
        list.slices3_h(0, 0, 100, 20, ("L", 16), "M", ("R", 16), 20, 1.0);
        assert_eq!(list.quads.len(), 3);
        assert_eq!((list.quads[0].x, list.quads[0].w), (0.0, 16.0));
        assert_eq!((list.quads[1].x, list.quads[1].w, list.quads[1].uw), (16.0, 68.0, 16.0));
        assert_eq!((list.quads[2].x, list.quads[2].w), (84.0, 16.0));
    }

    #[test]
    fn slices3_skip_the_middle_when_it_has_no_width() {
        let mut list = DrawList::default();
        list.slices3_h(0, 0, 32, 20, ("L", 16), "M", ("R", 16), 20, 1.0);
        assert_eq!(list.quads.len(), 2);
        assert_eq!((list.quads[1].x, list.quads[1].texture.as_str()), (16.0, "R"));
        list.slices3_v(0, 0, 12, 16, ("T", 8), "M", ("B", 8), 12, 1.0);
        assert_eq!(list.quads.len(), 4);
        assert_eq!((list.quads[3].y, list.quads[3].texture.as_str()), (8.0, "B"));
        list.slices3_v(0, 0, 12, 40, ("T", 8), "M", ("B", 8), 12, 1.0);
        assert_eq!(list.quads.len(), 7);
        assert_eq!((list.quads[5].y, list.quads[5].h, list.quads[5].vh), (8.0, 24.0, 8.0));
    }

    #[test]
    fn clipped_text_stops_before_the_limit() {
        let bytes = gly(&[(0x20, &[[0, 3, 0, 13], [4, 6, 0, 13]])], (1024, 128));
        let font = BitmapFont { atlas: "a".into(), glyphs: Glyphs::parse(&bytes).unwrap(), line_height: 13 };
        let mut list = DrawList::default();
        let end = list.draw_text_clipped(10, 0, 15, TEXT, "!!!", &font);
        assert_eq!(list.quads.len(), 2);
        assert_eq!(end, 22);
        let mut wide = DrawList::default();
        assert_eq!(wide.draw_text_clipped(10, 0, 100, TEXT, "! !", &font), 25);
        assert_eq!(wide.quads.len(), 2);
        let mut none = DrawList::default();
        assert_eq!(none.draw_text_clipped(10, 0, 5, TEXT, "!", &font), 10);
        assert!(none.quads.is_empty());
    }

    #[test]
    fn hline_is_a_one_pixel_quad_of_the_square_white_texture() {
        let mut list = DrawList::default();
        list.draw_hline(5, 18, 40, TEXT);
        assert_eq!(list.quads.len(), 1);
        let q = &list.quads[0];
        assert_eq!((q.x, q.y, q.w, q.h, q.uw, q.vh), (5.0, 18.0, 40.0, 1.0, 1.0, 1.0));
        assert_eq!((q.texture.as_str(), q.color), (LINE_TEXTURE, TEXT.rgba()));
        list.draw_hline(0, 0, 0, TEXT);
        assert_eq!(list.quads.len(), 1);
    }

    #[test]
    fn digits_width_counts_eight_pixels_per_strip_character() {
        assert_eq!(digits_width("52"), 16);
        assert_eq!(digits_width("400/900"), 56);
        assert_eq!(digits_width("1x2"), 16);
        assert_eq!(digits_width(""), 0);
    }

    #[test]
    fn rotated_textures_keep_the_angle_and_plain_quads_have_none() {
        let mut list = DrawList::default();
        list.draw_texture(0, 0, 8, 8, 0, 0, 8, 8, "a", 1.0);
        list.draw_texture_rotated(10, 20, 16, 16, 64, 64, "b", 1.5, 0.5);
        assert_eq!(list.quads[0].rotation, 0.0);
        let q = &list.quads[1];
        assert_eq!(
            (q.x, q.y, q.w, q.h, q.uw, q.vh, q.rotation, q.color[3]),
            (10.0, 20.0, 16.0, 16.0, 64.0, 64.0, 1.5, 0.5)
        );
    }

    #[test]
    fn the_xlarge_strip_uses_eleven_by_fourteen_cells_and_keeps_the_alpha() {
        let mut list = DrawList::default();
        list.draw_digits_with(XLARGE_NUMBER_STRIP, 3, 4, "7x:", Argb(0xCCDCDCDC));
        assert_eq!(list.quads.len(), 2);
        let q = &list.quads[0];
        assert_eq!((q.x, q.y, q.w, q.h, q.u, q.v, q.uw, q.vh), (3.0, 4.0, 11.0, 14.0, 77.0, 0.0, 11.0, 14.0));
        assert_eq!(q.texture, XLARGE_DIGIT_TEXTURE);
        assert_eq!(q.color[3], 0xCC as f32 / 255.0);
        assert_eq!((list.quads[1].x, list.quads[1].u), (14.0, 17.0 * 11.0));
        assert_eq!(digits_width_of(XLARGE_NUMBER_STRIP, "12"), 22);
    }

    #[test]
    fn digits_use_eight_pixel_cells_of_the_number_strip() {
        let mut list = DrawList::default();
        list.draw_digits(5, 5, "12/", TEXT);
        assert_eq!(list.quads.len(), 3);
        assert_eq!((list.quads[0].u, list.quads[0].uw, list.quads[0].w), (8.0, 8.0, 8.0));
        assert_eq!(list.quads[2].u, 13.0 * 8.0);
        assert_eq!(list.quads[2].texture, "L2UI.NWindow.Number");
    }

    #[test]
    fn transparent_list_draws_opaque_textures_at_half_alpha_and_leaves_text_and_fills() {
        let mut list = DrawList { transparent: true, ..Default::default() };
        list.draw_texture(0, 0, 8, 8, 0, 0, 8, 8, "a.b", 1.0);
        list.draw_texture(0, 0, 8, 8, 0, 0, 8, 8, "a.c", 0.25);
        list.draw_fill(0, 0, 4, 4, Argb(0xFFFFFFFF));
        list.draw_digits(0, 0, "1", Argb(0xFFFFFFFF));
        assert_eq!(list.quads[0].color[3], TRANSPARENT_ALPHA);
        assert_eq!(list.quads[1].color[3], 0.25);
        assert_eq!(list.quads[2].color[3], 1.0);
        assert_eq!(list.quads[3].color[3], 1.0);
    }
}
