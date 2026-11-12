use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::ini::IniFile;
use crate::list_box::{ListBox, ListBoxItem};
use crate::scroll::{ScrollWnd, SliderBar};
use crate::window::{Rect, WindowId};

pub const INI_NAME: &str = ".\\L2Debug.ini";
pub const SECTION: &str = "DebugInfo";
pub const COMMAND_LOG: &str = "L2Debug";
pub const COMMAND_WINDOW: &str = "L2DebugWindow";
pub const BACK_TEXTURE: &str = "NWindow.ChatBack";
pub const MAX_LINES: i32 = 150;
pub const VISIBLE_LINES: i32 = 20;
pub const TEXT_X: i32 = 30;
pub const TEXT_Y: i32 = 3;
pub const PLAIN_COLOR: u32 = 0xFFDCDCDC;
pub const HEIGHT: i32 = 246;
pub const ID: WindowId = WindowId(0x10000 + 44);
pub const REGISTRATION_CALL: u32 = 0x10082e5b;
pub const REGISTRATION_FLAGS: u32 = 0x2;
pub const VTABLE: u32 = 0x101b3f30;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DebugCategory {
    pub name: String,
    pub color: u32,
    pub enabled: bool,
}

fn parse_channel(value: &str, key: &str) -> u8 {
    let Some(at) = value.find(key) else { return 0xFF };
    let digits: String = value[at + key.len()..].trim_start().chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() { 0xFF } else { digits.parse::<i64>().unwrap_or(0) as u8 }
}

pub fn command(verb: &str) -> Option<bool> {
    if verb.eq_ignore_ascii_case(COMMAND_LOG) {
        Some(false)
    } else if verb.eq_ignore_ascii_case(COMMAND_WINDOW) {
        Some(true)
    } else {
        None
    }
}

pub fn load_categories(ini: &IniFile) -> Vec<DebugCategory> {
    let count = ini.int(SECTION, "count").unwrap_or(0).max(0);
    (0..count)
        .map(|index| {
            let section = format!("Library{index}");
            let color = ini.get(&section, &format!("color{index}")).map_or(0, |value| {
                let (r, g, b) = (parse_channel(value, "R="), parse_channel(value, "G="), parse_channel(value, "B="));
                0xFF00_0000 | u32::from(r) << 16 | u32::from(g) << 8 | u32::from(b)
            });
            DebugCategory {
                name: ini.get(&section, &format!("name{index}")).unwrap_or_default().to_string(),
                color,
                enabled: ini.bool(&section, &format!("IsEnable{index}")).unwrap_or(false),
            }
        })
        .collect()
}

pub fn log_line(categories: &[DebugCategory], category: i32, text: &str) -> Option<(String, u32)> {
    if category < 0 || (!categories.is_empty() && category as usize > categories.len()) {
        return None;
    }
    let entry = categories.get(category as usize)?;
    entry.enabled.then(|| (format!("[{}]{}", entry.name, text), entry.color))
}

#[derive(Debug, Clone)]
pub struct DebugWnd {
    pub categories: Vec<DebugCategory>,
    pub list: ListBox,
    pub num_lines: i32,
    scroll: ScrollWnd,
    pub log_enabled: bool,
}

impl DebugWnd {
    pub fn new(width: i32, categories: Vec<DebugCategory>, log_enabled: bool) -> Self {
        let rect = Rect::new(0, 0, width, HEIGHT);
        Self {
            categories,
            list: ListBox {
                rect,
                background: Some(BACK_TEXTURE.to_string()),
                items: Vec::new(),
                visible_rows: VISIBLE_LINES,
                first: 0,
                last_allowed: 0,
                selected: -1,
                text_origin: (TEXT_X, TEXT_Y),
                text_height: 0,
                max_lines: MAX_LINES,
                auto_scroll: true,
            },
            num_lines: 0,
            scroll: ScrollWnd { slider: SliderBar::new(HEIGHT - 30), up_pressed: false, down_pressed: false },
            log_enabled,
        }
    }

    pub fn resize(&mut self, width: i32) {
        self.list.rect.w = width;
    }

    fn update_scroll_bar(&mut self) {
        if !self.list.items.is_empty() {
            self.num_lines = self.list.items.len() as i32;
        }
        self.list.last_allowed = self.num_lines;
        self.scroll.slider.set_range(self.num_lines, VISIBLE_LINES);
        self.scroll.slider.set_pos(self.list.first);
    }

    pub fn add_string(&mut self, text: &str, color: Argb) {
        if self.num_lines < MAX_LINES {
            self.num_lines += 1;
            if self.num_lines > VISIBLE_LINES {
                self.list.first += 1;
            }
        } else if !self.list.items.is_empty() {
            self.list.items.remove(0);
        }
        self.list.items.push(ListBoxItem { text: text.to_string(), color, param: 0 });
        self.update_scroll_bar();
    }

    pub fn add_log(&mut self, category: i32, text: &str) {
        if let Some((line, color)) = log_line(&self.categories, category, text) {
            self.add_string(&line, Argb(color));
        }
    }

    pub fn debug_logf(&mut self, category: i32, text: &str) {
        if self.log_enabled {
            self.add_log(category, text);
        }
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        self.list.paint(list, origin, font);
        let rect = self.list.rect;
        self.scroll.paint(list, origin.0 + rect.x, origin.1 + rect.y, rect.w, rect.h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ini::parse_ini;

    const INI: &str = "[DebugInfo]\ncount=2\n[Library0]\nname0=Net\ncolor0=R=10,G=20,B=30\nIsEnable0=True\n[Library1]\nname1=Gfx\nIsEnable1=False\n";

    #[test]
    fn the_categories_come_from_l2debug_ini_like_0x10084910() {
        let categories = load_categories(&parse_ini(INI));
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0], DebugCategory { name: "Net".into(), color: 0xFF0A141E, enabled: true });
        assert_eq!((categories[1].name.as_str(), categories[1].enabled), ("Gfx", false));
        assert!(load_categories(&parse_ini("")).is_empty());
    }

    #[test]
    fn a_log_line_needs_its_category_enabled_and_carries_the_color_0x100847d0() {
        let categories = load_categories(&parse_ini(INI));
        assert_eq!(log_line(&categories, 0, "up"), Some(("[Net]up".into(), 0xFF0A141E)));
        assert_eq!(log_line(&categories, 1, "up"), None);
        assert_eq!(log_line(&categories, 2, "up"), None);
        assert_eq!(log_line(&categories, -1, "up"), None);
        assert_eq!(log_line(&[], 0, "up"), None);
    }

    fn font() -> BitmapFont {
        use crate::font::{Glyphs, gly};
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    #[test]
    fn the_window_keeps_150_lines_follows_the_last_one_and_paints_them_at_the_pitch_of_12() {
        let categories = load_categories(&parse_ini(INI));
        let mut window = DebugWnd::new(800, categories, true);
        for index in 0..160 {
            window.debug_logf(0, &format!("l{index}"));
        }
        window.debug_logf(1, "off");
        assert_eq!(window.list.items.len(), 150);
        assert_eq!(window.list.items[0].text, "[Net]l10");
        assert_eq!(window.list.first, 130);
        let mut draws = DrawList::default();
        window.paint(&mut draws, (0, 0), &font());
        assert_eq!(draws.quads[0].texture, BACK_TEXTURE);
        assert_eq!((draws.quads[0].w, draws.quads[0].h), (800.0, 246.0));
        window.log_enabled = false;
        window.debug_logf(0, "x");
        assert_eq!(window.list.items.len(), 150);
    }
}
