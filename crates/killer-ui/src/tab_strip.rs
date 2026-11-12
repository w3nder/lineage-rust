use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::tab_button::TabButton;
use crate::tab_control::{TabControl, TabEntry};
use crate::window::Rect;

#[derive(Debug, Clone)]
pub struct TabSpec {
    pub label: Option<String>,
    pub page: Option<usize>,
    pub size: (i32, i32),
    pub textures: [Option<String>; 2],
    pub spacing: i32,
    pub tooltip: i32,
    pub special_attachment: bool,
}

#[derive(Debug, Clone)]
pub struct StripButton {
    pub paint: TabButton,
    pub internal_id: u32,
    pub index: usize,
    pub attachment_flags: u32,
    pub tooltip: Option<i32>,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct TabStrip {
    pub control: TabControl,
    pub buttons: Vec<StripButton>,
    pub id: u32,
    pub internal_id: u32,
    pub next_x: i32,
}

impl TabStrip {
    pub fn new(id: u32, internal_id: u32) -> Self {
        Self { control: TabControl::default(), buttons: Vec::new(), id, internal_id, next_x: 0 }
    }

    pub fn add(&mut self, spec: TabSpec, font: &BitmapFont) -> Option<usize> {
        let label = spec.label.as_ref()?;
        let (width, height) = if spec.size.0 != 0 && spec.size.1 != 0 {
            spec.size
        } else {
            ((font.measure(label) as i32).wrapping_add(10), (font.line_height as i32).wrapping_add(5))
        };
        self.add_sized(spec, (width, height))
    }

    pub fn add_fixed(&mut self, spec: TabSpec) -> Option<usize> {
        if spec.size.0 == 0 || spec.size.1 == 0 { return None; }
        let size = spec.size;
        self.add_sized(spec, size)
    }

    fn add_sized(&mut self, spec: TabSpec, (width, height): (i32, i32)) -> Option<usize> {
        let label = spec.label?;
        let index = self.buttons.len();
        let first = index == 0;
        if first { self.control.current = spec.page; }
        let textures = if spec.textures.iter().all(Option::is_some) {
            [spec.textures[0].clone(), spec.textures[1].clone(), None]
        } else {
            [Some("NWindow.BlackTexture".into()), Some("NWindow.Darkyellow".into()), None]
        };
        self.buttons.push(StripButton {
            paint: TabButton { rect: Rect::new(self.next_x, 0, width, height), textures,
                source: (0, 0), state: i32::from(first), enabled: true, label, text_offset: (2, 1) },
            internal_id: self.internal_id, index,
            attachment_flags: if spec.special_attachment { 0x400032 } else { 0x22 },
            tooltip: (spec.tooltip >= 0).then_some(spec.tooltip), visible: first,
        });
        self.control.entries.push(TabEntry { page: spec.page, enabled: true, detached: false, pressed: first, org: (self.next_x, 0) });
        self.next_x = self.next_x.wrapping_add(width).wrapping_add(spec.spacing);
        Some(index)
    }

    pub fn move_tabs(&mut self, dx: i32, dy: i32) {
        self.control.move_tabs(dx, dy);
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        for (button, entry) in self.buttons.iter().zip(&self.control.entries) {
            if button.visible {
                let mut paint = button.paint.clone();
                paint.state = i32::from(entry.pressed);
                paint.enabled = entry.enabled;
                paint.paint(list, origin, font);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn spec(page: usize) -> TabSpec {
        TabSpec { label: Some("Video".into()), page: Some(page), size: (74, 23),
            textures: [Some("l2ui_ch3.tabbutton.normal_tab_on".into()), Some("l2ui_ch3.tabbutton.normal_tab".into())],
            spacing: 1, tooltip: -1, special_attachment: false }
    }

    #[test]
    fn tab_add_1002df20_composes_settings_geometry_state_and_button_identity() {
        let mut strip = TabStrip::new(0xf006, 91);
        for (index, label) in ["Video", "Audio", "Game"].into_iter().enumerate() {
            let mut spec = spec(index);
            spec.label = Some(label.into());
            spec.spacing = i32::from(index < 2);
            assert_eq!(strip.add(spec, &font()), Some(index));
        }
        assert_eq!(strip.control.current, Some(0));
        assert_eq!(strip.next_x, 224);
        for (index, button) in strip.buttons.iter().enumerate() {
            assert_eq!(button.paint.rect, Rect::new(75 * index as i32, 0, 74, 23));
            assert_eq!(button.paint.text_offset, (2, 1));
            assert_eq!(button.paint.state, i32::from(index == 0));
            assert_eq!((button.internal_id, button.index, button.attachment_flags), (91, index, 0x22));
            assert_eq!(button.tooltip, None);
            assert_eq!(button.visible, index == 0);
        }
        let mut list = DrawList::default();
        strip.paint(&mut list, (12, 25), &font());
        assert_eq!(list.quads[0].texture, "l2ui_ch3.tabbutton.normal_tab");
        assert_eq!((list.quads[0].x, list.quads[0].y), (12.0, 25.0));
    }

    #[test]
    fn tab_add_1002df20_rejects_missing_label_and_preserves_explicit_or_measured_size() {
        let mut strip = TabStrip::new(1, 2);
        let mut missing = spec(0);
        missing.label = None;
        assert_eq!(strip.add(missing, &font()), None);
        assert!(strip.buttons.is_empty() && strip.control.entries.is_empty());
        assert_eq!((strip.control.current, strip.next_x), (None, 0));
        for size in [(0, 23), (74, 0), (0, 0), (-1, 23), (74, 23)] {
            let mut spec = spec(7);
            spec.size = size;
            spec.special_attachment = true;
            spec.tooltip = 4;
            let index = strip.add(spec, &font()).unwrap();
            let button = &strip.buttons[index];
            assert_eq!((button.paint.rect.w, button.paint.rect.h), if size.0 == 0 || size.1 == 0 { (40, 18) } else { size });
            assert_eq!((button.attachment_flags, button.tooltip), (0x400032, Some(4)));
        }
        assert_eq!(strip.control.current, Some(7));
    }

    #[test]
    fn tab_add_1002df20_uses_both_default_textures_when_either_argument_is_absent() {
        for mask in 0..4 {
            let mut strip = TabStrip::new(1, 2);
            let mut spec = spec(0);
            spec.textures = std::array::from_fn(|index| if mask & (1 << index) != 0 { Some(index.to_string()) } else { None });
            strip.add(spec, &font());
            let textures = &strip.buttons[0].paint.textures;
            assert_eq!(textures[0].as_deref(), Some(if mask == 3 { "0" } else { "NWindow.BlackTexture" }));
            assert_eq!(textures[1].as_deref(), Some(if mask == 3 { "1" } else { "NWindow.Darkyellow" }));
            assert_eq!(textures[2], None);
        }
    }
}
