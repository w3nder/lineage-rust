use killer_ui::draw::DrawList;
use killer_ui::font::BitmapFont;
use killer_ui::frame::FrameCtrl;
use killer_ui::html::{self, Line};
use killer_ui::ini::WindowsInfo;
use killer_ui::initial_position::InitialPosition;
use killer_ui::npc_viewer::{HTML_WRAP_MARGIN, paint_html_viewer};
use killer_ui::scripted::help_html_wnd::{HelpHtmlWnd, HelpHtmlWndHost};
use killer_ui::window::{Desktop, Rect, Window, WindowId};

pub const SIZE: (i32, i32) = (310, 311);
pub const CONTENT: Rect = Rect::new(7, 30, 296, 274);
pub const BACK: &str = "L2UI_ch3.NpcWnd.Npc2_back";
pub const HELP_FILE: &str = "..\\L2text\\help.htm";
pub const MACRO_FILE: &str = "..\\L2text\\help_macro.htm";

#[derive(Default)]
pub struct HelpWindow {
    pub title: String,
    pub lines: Vec<Line>,
    pub height: i32,
    pub scroll: i32,
    script: HelpHtmlWnd,
    pressed: Option<String>,
}

pub fn create_window(desktop: &mut Desktop, info: &WindowsInfo, viewport: (i32, i32)) {
    let initial = InitialPosition { enabled: true, horizontal_mode: 0, vertical_mode: 5, x: 0, y: 252 };
    let (x, y) = info
        .position(WindowId::HELP)
        .unwrap_or((0, killer_ui::window_restore::x87_to_int(f64::from(viewport.1 as f32) * 0.5 - 252.0)));
    let mut window = Window::new(WindowId::HELP, Rect::new(x, y, SIZE.0, SIZE.1), Rect::new(0, 0, SIZE.0, 20))
        .with_initial_position(initial);
    window.visible = false;
    desktop.windows.push(window);
}

fn visible(desktop: &mut Desktop, show: bool) {
    if let Some(window) = desktop.window_mut(WindowId::HELP) {
        window.visible = show;
    }
}

struct Host<'a> {
    desktop: &'a mut Desktop,
    font: &'a BitmapFont,
    path: &'a str,
    html: &'a str,
    lines: &'a mut Vec<Line>,
    height: &'a mut i32,
    scroll: &'a mut i32,
    resolve: &'a dyn Fn(html::Marker, i32) -> Option<String>,
}

impl Host<'_> {
    fn load(&mut self, text: &str) {
        *self.lines = html::layout(&html::parse_with(text, self.resolve), CONTENT.w - HTML_WRAP_MARGIN, self.font);
        *self.height = html::height(self.lines, self.font);
        *self.scroll = 0;
    }
}

impl HelpHtmlWndHost for Host<'_> {
    fn parse_string(&mut self, _: &str, key: &str) -> String {
        match key {
            "FilePath" => self.path.to_owned(),
            "HtmlString" => self.html.to_owned(),
            _ => unreachable!("compiled help parameter"),
        }
    }
    fn uiapi_htmlctrl_load_html(&mut self, _: &str, _: &str) {
        self.load(self.html);
    }
    fn uiapi_htmlctrl_load_html_from_string(&mut self, _: &str, text: &str) {
        self.load(text);
    }
    fn uiapi_window_hide_window(&mut self, _: &str) {
        visible(self.desktop, false);
    }
    fn uiapi_window_show_window(&mut self, _: &str) {
        visible(self.desktop, true);
    }
    fn uiapi_window_set_focus(&mut self, _: &str) {
        if let Some(index) = self.desktop.windows.iter().position(|w| w.id == WindowId::HELP) {
            let window = self.desktop.windows.remove(index);
            self.desktop.windows.push(window);
        }
    }
}

impl HelpWindow {
    pub fn toggle(&mut self, desktop: &mut Desktop, font: &BitmapFont, path: &str, html: &str, resolve: &dyn Fn(html::Marker, i32) -> Option<String>) {
        self.script.m_b_show = desktop.window(WindowId::HELP).is_some_and(|w| w.visible);
        let mut host = Host {
            desktop,
            font,
            path,
            html,
            lines: &mut self.lines,
            height: &mut self.height,
            scroll: &mut self.scroll,
            resolve,
        };
        self.script.handle_show_help(&mut host, String::new());
        if host.desktop.window(WindowId::HELP).is_some_and(|w| w.visible) {
            self.script.on_show();
        } else {
            self.script.on_hide();
        }
        self.pressed = None;
    }

    pub fn load(&mut self, desktop: &mut Desktop, font: &BitmapFont, html: &str, resolve: &dyn Fn(html::Marker, i32) -> Option<String>) {
        let mut host = Host {
            desktop,
            font,
            path: "",
            html,
            lines: &mut self.lines,
            height: &mut self.height,
            scroll: &mut self.scroll,
            resolve,
        };
        self.script.handle_load_help_html(&mut host, String::new());
        self.pressed = None;
    }

    pub fn close(&mut self, desktop: &mut Desktop) {
        visible(desktop, false);
        self.script.on_hide();
        self.pressed = None;
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        FrameCtrl::titled(&self.title, true).paint(list, x, y, SIZE.0, SIZE.1, font, alpha);
        list.draw_texture(x, y + 20, SIZE.0, SIZE.1 - 20, 0, 0, SIZE.0, SIZE.1 - 20, BACK, 1.0);
        paint_html_viewer(
            list,
            x + CONTENT.x,
            y + CONTENT.y,
            (CONTENT.w, CONTENT.h),
            &self.lines,
            self.scroll,
            font,
            alpha,
        );
    }

    pub fn wheel(&mut self, dy: i32) {
        self.scroll = (self.scroll + dy).clamp(0, (self.height - CONTENT.h).max(0));
    }

    fn link_at(&self, x: i32, y: i32, font: &BitmapFont) -> Option<&str> {
        if !CONTENT.contains(x, y) {
            return None;
        }
        let action = html::link_at(&self.lines, x - CONTENT.x, y - CONTENT.y + self.scroll, font)?;
        html::local_link_target(action)
    }

    pub fn mouse_down(&mut self, desktop: &mut Desktop, x: i32, y: i32, font: &BitmapFont) {
        if killer_ui::npc_viewer::close_button().contains(x, y) {
            self.close(desktop);
            return;
        }
        self.pressed = self.link_at(x, y, font).map(str::to_owned);
    }

    pub fn mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, font: &BitmapFont) -> Option<String> {
        let pressed = self.pressed.take()?;
        let (WindowId::HELP, x, y) = over? else {
            return None;
        };
        (self.link_at(x, y, font) == Some(pressed.as_str())).then_some(pressed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::font::Glyphs;

    fn font() -> BitmapFont {
        BitmapFont::new(
            "L2Font-e.SmallFont-e".into(),
            Glyphs::parse(include_bytes!("../../../crates/killer-ui/tests/fixtures/smallfont-e.gly")).unwrap(),
        )
    }

    #[test]
    fn generated_help_toggle_local_navigation_and_close_share_the_real_window() {
        let font = font();
        let mut desktop = Desktop::new((1024, 768));
        create_window(&mut desktop, &WindowsInfo::default(), (1024, 768));
        let mut help = HelpWindow::default();
        assert_eq!(desktop.window(WindowId::HELP).unwrap().rect, Rect::new(0, 132, 310, 311));
        help.toggle(
            &mut desktop,
            &font,
            HELP_FILE,
            "<html><body><a link=\"..\\L2text\\help_general.htm\">General</a></body></html>",
            &|_, _| None,
        );
        assert!(desktop.window(WindowId::HELP).unwrap().visible);
        assert_eq!(desktop.windows.last().unwrap().id, WindowId::HELP);
        let (line, span) = help
            .lines
            .iter()
            .find_map(|line| line.spans.iter().find(|s| s.action.is_some()).map(|span| (line, span)))
            .unwrap();
        let (x, y) = (CONTENT.x + span.x + 1, CONTENT.y + line.y + 1);
        help.mouse_down(&mut desktop, x, y, &font);
        assert_eq!(help.mouse_up(None, &font), None);
        help.mouse_down(&mut desktop, x, y, &font);
        assert_eq!(help.mouse_up(Some((WindowId::HELP, x, y)), &font), Some("..\\L2text\\help_general.htm".into()));
        help.load(&mut desktop, &font, "<html><body>General help</body></html>", &|_, _| None);
        assert!(help.lines.iter().flat_map(|line| &line.spans).any(|span| span.text.contains("General")));
        help.toggle(&mut desktop, &font, MACRO_FILE, "", &|_, _| None);
        assert!(!desktop.window(WindowId::HELP).unwrap().visible);
        help.toggle(&mut desktop, &font, MACRO_FILE, "<html><body>Macro</body></html>", &|_, _| None);
        help.mouse_down(&mut desktop, 290, 5, &font);
        assert!(!desktop.window(WindowId::HELP).unwrap().visible);
    }
}
