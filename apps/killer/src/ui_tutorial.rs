use killer_assets::client::DatTables;
use killer_platform::time::Instant;

use killer_ui::font::BitmapFont;
use killer_ui::html::{bypass_command, link_target};
use killer_ui::notification_layout::{NotificationButton, NotificationLayoutBackend, reposition_notifications};
use killer_ui::tutorial::{self, ClientEvents, EffectButton, TutorialViewer};
use killer_ui::window::{Desktop, Rect, Window, WindowId};
use killer_world::{NetCommand, WorldEvent};

#[derive(Default)]
pub struct TutorialWindows {
    pub viewer: TutorialViewer,
    pub button: EffectButton,
    pub events: ClientEvents,
    pub button_pressed: bool,
    pub quest_button: EffectButton,
    pub quest_pressed: bool,
    pub quest_click: Option<i32>,
    pub mail_button: EffectButton,
    pub mail_pressed: bool,
    pub mail_click: bool,
    link_pressed: Option<String>,
    pub link_question: Option<(String, String)>,
    last_tick: Option<Instant>,
}

pub fn create_windows(desktop: &mut Desktop, viewport: (i32, i32)) {
    let (x, y) = tutorial::viewer_position(viewport);
    let mut viewer = Window::new(WindowId::TUTORIAL_VIEWER, Rect::new(x, y, tutorial::VIEWER_WIDTH, tutorial::VIEWER_CREATE_HEIGHT), tutorial::viewer_grip())
        .with_initial_position(tutorial::VIEWER_INITIAL_POSITION);
    viewer.visible = false;
    desktop.windows.push(viewer);
    let size = tutorial::BUTTON_SIZE;
    let mut button = Window::new(WindowId::TUTORIAL_BUTTON, Rect::new(tutorial::BUTTON_X, viewport.1 - tutorial::BUTTON_BOTTOM, size, size), Rect::new(0, 0, 0, 0))
        .with_initial_position(tutorial::BUTTON_INITIAL_POSITION);
    button.visible = false;
    desktop.windows.push(button);
    let mut quest = Window::new(WindowId::QUEST_BUTTON, Rect::new(tutorial::QUEST_BUTTON_X, viewport.1 - tutorial::BUTTON_BOTTOM, size, size), Rect::new(0, 0, 0, 0))
        .with_initial_position(killer_ui::initial_position::InitialPosition { x: tutorial::QUEST_BUTTON_X, ..tutorial::BUTTON_INITIAL_POSITION });
    quest.visible = false;
    desktop.windows.push(quest);
    let mut mail = Window::new(WindowId::MAIL_BUTTON, Rect::new(tutorial::MAIL_BUTTON_X, viewport.1 - tutorial::BUTTON_BOTTOM, size, size), Rect::new(0, 0, 0, 0))
        .with_initial_position(killer_ui::initial_position::InitialPosition { x: tutorial::MAIL_BUTTON_X, ..tutorial::BUTTON_INITIAL_POSITION });
    mail.visible = false;
    desktop.windows.push(mail);
}

struct Layout<'a>(&'a mut Desktop);

impl NotificationLayoutBackend for Layout<'_> {
    type RootRect = ();
    fn root_rect(&mut self) {}
    fn chat_y(&mut self) -> i32 {
        self.0.window(WindowId::CHAT).map_or(0, |w| w.rect.y)
    }
    fn chat_child_visible(&mut self) -> bool {
        self.0.window(WindowId::SYSTEM_MSG).is_some_and(|w| w.visible)
    }
    fn set_button_position(&mut self, button: NotificationButton, x: i32, y: i32, _: ()) {
        let id = match button {
            NotificationButton::Tutorial => WindowId::TUTORIAL_BUTTON,
            NotificationButton::Quest => WindowId::QUEST_BUTTON,
            NotificationButton::Mail => WindowId::MAIL_BUTTON,
        };
        if let Some(window) = self.0.window_mut(id) {
            (window.rect.x, window.rect.y) = (x, y);
        }
    }
}

fn set_visible(desktop: &mut Desktop, id: WindowId, visible: bool) {
    if let Some(window) = desktop.window_mut(id) {
        window.visible = visible;
    }
    if visible
        && let Some(index) = desktop.windows.iter().position(|w| w.id == id)
    {
        let window = desktop.windows.remove(index);
        desktop.windows.push(window);
    }
}

struct TutorialHost<'a> {
    viewer: &'a mut TutorialViewer,
    desktop: &'a mut Desktop,
    font: &'a BitmapFont,
    html: &'a str,
    loaded: bool,
    tables: Option<&'a DatTables>,
}

impl killer_ui::scripted::tutorial_viewer_wnd::TutorialViewerWndHost for TutorialHost<'_> {
    fn parse_string(&mut self, _: &str, _: &str) -> String { self.html.to_owned() }
    fn hide_window(&mut self, _: &str) { set_visible(self.desktop, WindowId::TUTORIAL_VIEWER, false); }
    fn show_window_with_focus(&mut self, _: &str) { set_visible(self.desktop, WindowId::TUTORIAL_VIEWER, true); }
    fn uiapi_htmlctrl_load_html_from_string(&mut self, _: &str, html: &str) { self.loaded = self.viewer.load_with(html, self.font, &|marker, id| crate::ui2d::html_marker(self.tables, marker, id)); }
    fn uiapi_htmlctrl_get_frame_max_height(&mut self, _: &str) -> i32 { self.viewer.doc_height }
    fn uiapi_window_get_rect(&mut self, _: &str) -> Rect {
        self.desktop.window(WindowId::TUTORIAL_VIEWER).map_or(Rect::default(), |w| w.rect)
    }
    fn uiapi_window_set_window_size(&mut self, path: &str, width: i32, height: i32) {
        if !self.loaded { return; }
        match path {
            "TutorialViewerWnd" => {
                self.viewer.height = height;
                if let Some(window) = self.desktop.window_mut(WindowId::TUTORIAL_VIEWER) {
                    window.rect.w = width;
                    window.rect.h = height;
                }
            }
            "TutorialViewerWnd.texTutorialViewerBack2" => {
                self.viewer.body_bounds.w = width;
                self.viewer.body_bounds.h = height;
            }
            "TutorialViewerWnd.HtmlTutorialViewer" => {
                self.viewer.html_bounds.w = width;
                self.viewer.html_bounds.h = height;
            }
            _ => unreachable!("compiled tutorial path"),
        }
    }
    fn uiapi_window_move_to(&mut self, _: &str, _: i32, y: i32) {
        if self.loaded {
            let origin = self.desktop.window(WindowId::TUTORIAL_VIEWER).map_or(0, |w| w.rect.y);
            self.viewer.bottom_y = y - origin;
        }
    }
}

impl TutorialWindows {
    pub fn world_event(&mut self, event: &WorldEvent, desktop: &mut Desktop, font: &BitmapFont, tables: Option<&DatTables>) {
        match event {
            WorldEvent::TutorialShowHtml(packet) => {
                self.link_question = None;
                let mut host = TutorialHost { viewer: &mut self.viewer, desktop, font, html: &packet.html, loaded: false, tables };
                killer_ui::scripted::tutorial_viewer_wnd::TutorialViewerWnd::default().on_event(&mut host, 2430, String::new());
            }
            WorldEvent::TutorialCloseHtml(_) => {
                self.link_pressed = None;
                self.link_question = None;
                let mut host = TutorialHost { viewer: &mut self.viewer, desktop, font, html: "", loaded: false, tables };
                killer_ui::scripted::tutorial_viewer_wnd::TutorialViewerWnd::default().on_event(&mut host, 2440, String::new());
            }
            WorldEvent::TutorialShowQuestionMark(mark) => {
                self.button.begin_effect(mark.mark_id);
                set_visible(desktop, WindowId::TUTORIAL_BUTTON, true);
                reposition_notifications(&mut Layout(desktop));
            }
            WorldEvent::ShowQuestMark(mark) => {
                self.quest_button.begin_effect(mark.value);
                set_visible(desktop, WindowId::QUEST_BUTTON, true);
                reposition_notifications(&mut Layout(desktop));
            }
            WorldEvent::MailArrived => {
                set_visible(desktop, WindowId::MAIL_BUTTON, true);
                self.mail_button.begin_effect(0);
                reposition_notifications(&mut Layout(desktop));
            }
            WorldEvent::TutorialEnableClientEvent(event) => self.events.enable(event.event_id),
            _ => {}
        }
    }

    pub fn tick(&mut self, now: Instant, desktop: &mut Desktop) {
        let dt = self.last_tick.map_or(0, |last| now.saturating_duration_since(last).as_millis().min(1000) as u32);
        self.last_tick = Some(now);
        self.button.tick(dt);
        self.quest_button.tick(dt);
        self.mail_button.tick(dt);
        if [WindowId::TUTORIAL_BUTTON, WindowId::QUEST_BUTTON, WindowId::MAIL_BUTTON].into_iter().any(|id| desktop.window(id).is_some_and(|w| w.visible)) {
            reposition_notifications(&mut Layout(desktop));
        }
    }

    pub fn mouse_down(&mut self, id: WindowId, x: i32, y: i32, font: &BitmapFont) {
        match id {
            WindowId::TUTORIAL_VIEWER => self.link_pressed = self.viewer.link_at(x, y, font).map(str::to_string),
            WindowId::TUTORIAL_BUTTON => self.button_pressed = true,
            WindowId::QUEST_BUTTON => self.quest_pressed = true,
            WindowId::MAIL_BUTTON => self.mail_pressed = true,
            _ => {}
        }
    }

    pub fn mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop, font: &BitmapFont) -> Vec<NetCommand> {
        let mut commands = Vec::new();
        if std::mem::take(&mut self.button_pressed) && matches!(over, Some((WindowId::TUTORIAL_BUTTON, _, _))) {
            commands.push(NetCommand::TutorialQuestionMark { mark_id: self.button.mark_id });
            set_visible(desktop, WindowId::TUTORIAL_BUTTON, false);
        }
        if std::mem::take(&mut self.quest_pressed) && matches!(over, Some((WindowId::QUEST_BUTTON, _, _))) {
            self.quest_click = Some(self.quest_button.mark_id);
            set_visible(desktop, WindowId::QUEST_BUTTON, false);
        }
        if std::mem::take(&mut self.mail_pressed) && matches!(over, Some((WindowId::MAIL_BUTTON, _, _))) {
            self.mail_click = true;
            set_visible(desktop, WindowId::MAIL_BUTTON, false);
        }
        if let Some(pressed) = self.link_pressed.take()
            && let Some((WindowId::TUTORIAL_VIEWER, x, y)) = over
            && self.viewer.link_at(x, y, font) == Some(pressed.as_str())
        {
            match self.viewer.link_message_at(x, y).map(str::to_string) {
                Some(message) => self.link_question = Some((pressed, message)),
                None => commands.extend(Self::link_command(&pressed)),
            }
        }
        commands
    }

    pub fn link_command(action: &str) -> Option<NetCommand> {
        if let Some(link) = link_target(action) {
            Some(NetCommand::TutorialLinkHtml { link })
        } else {
            bypass_command(action).map(|command| NetCommand::TutorialPassCmd { command })
        }
    }

    pub fn fire(&mut self, event: i32) -> Option<NetCommand> {
        self.events.fire(event).map(|event_id| NetCommand::TutorialClientEvent { event_id })
    }

    pub fn camera_rotated(&mut self, radians: f32) -> Option<NetCommand> {
        if tutorial::camera_rotate_units(radians).abs() > tutorial::CAMERA_EVENT_MIN_UNITS { self.fire(tutorial::EVENT_CAMERA_ROTATE) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::font::Glyphs;
    use l2_net::game::{TutorialCloseHtml, TutorialEnableClientEvent, TutorialShowHtml, TutorialShowQuestionMark};

    fn font() -> BitmapFont {
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(include_bytes!("../../../crates/killer-ui/tests/fixtures/smallfont-e.gly")).unwrap())
    }

    fn desktop() -> Desktop {
        let mut desktop = Desktop::new((1024, 768));
        desktop.windows.push(Window::new(WindowId::CHAT, Rect::new(0, 600, 300, 168), Rect::new(0, 0, 0, 0)));
        let mut sysmsg = Window::new(WindowId::SYSTEM_MSG, Rect::new(0, 470, 300, 125), Rect::new(0, 0, 0, 0));
        sysmsg.visible = false;
        desktop.windows.push(sysmsg);
        create_windows(&mut desktop, (1024, 768));
        desktop
    }

    #[test]
    fn the_quest_mark_shows_the_pulsing_button_beside_the_tutorial_one_and_a_click_reports_the_quest() {
        let (font, mut desktop) = (font(), desktop());
        let mut windows = TutorialWindows::default();
        assert!(!desktop.window(WindowId::QUEST_BUTTON).unwrap().visible);
        windows.world_event(&WorldEvent::ShowQuestMark(l2_net::game::ShowQuestMark { value: 348 }), &mut desktop, &font, None);
        let button = desktop.window(WindowId::QUEST_BUTTON).unwrap();
        assert!(button.visible);
        assert_eq!((button.rect.x, button.rect.y), (42, 600 - 37));
        assert!(windows.quest_button.effect);
        windows.mouse_down(WindowId::QUEST_BUTTON, 3, 3, &font);
        assert!(windows.mouse_up(Some((WindowId::QUEST_BUTTON, 3, 3)), &mut desktop, &font).is_empty());
        assert_eq!(windows.quest_click.take(), Some(348));
        assert!(!desktop.window(WindowId::QUEST_BUTTON).unwrap().visible);
        windows.world_event(&WorldEvent::ShowQuestMark(l2_net::game::ShowQuestMark { value: 9 }), &mut desktop, &font, None);
        windows.mouse_down(WindowId::QUEST_BUTTON, 3, 3, &font);
        windows.mouse_up(Some((WindowId::TARGET, 3, 3)), &mut desktop, &font);
        assert_eq!(windows.quest_click, None);
    }

    #[test]
    fn a_tutorial_link_with_msg_defers_its_command_to_the_dialog_3c_1004baa0() {
        let (font, mut desktop) = (font(), desktop());
        let mut windows = TutorialWindows::default();
        windows.world_event(&WorldEvent::TutorialShowHtml(TutorialShowHtml { html: "<html><body><a action=\"bypass -h tutorial_close\" msg=\"570;Adena\">x</a></body></html>".into() }), &mut desktop, &font, None);
        let span = windows.viewer.lines.iter().flat_map(|l| l.spans.iter().filter(|s| s.action.is_some()).map(move |s| (l.y, s.x))).next().unwrap();
        let (x, y) = (tutorial::HTML_POS.0 + span.1 + 1, tutorial::HTML_POS.1 + span.0 + 1);
        windows.mouse_down(WindowId::TUTORIAL_VIEWER, x, y, &font);
        assert!(windows.mouse_up(Some((WindowId::TUTORIAL_VIEWER, x, y)), &mut desktop, &font).is_empty());
        assert_eq!(windows.link_question, Some(("bypass -h tutorial_close".into(), "570;Adena".into())));
        assert_eq!(TutorialWindows::link_command("bypass -h tutorial_close"), Some(NetCommand::TutorialPassCmd { command: "tutorial_close".into() }));
    }

    #[test]
    fn the_server_opens_resizes_and_closes_the_viewer() {
        let (font, mut desktop) = (font(), desktop());
        let mut windows = TutorialWindows::default();
        assert_eq!(desktop.window(WindowId::TUTORIAL_VIEWER).unwrap().rect, Rect::new(694, 20, 310, 401));
        windows.world_event(&WorldEvent::TutorialShowHtml(TutorialShowHtml { html: "<html><body><a action=\"link t2.htm\">go</a> <a action=\"bypass -h tutorial_close\">x</a></body></html>".into() }), &mut desktop, &font, None);
        let window = desktop.window(WindowId::TUTORIAL_VIEWER).unwrap();
        assert!(window.visible);
        assert_eq!(window.rect.h, 264);
        let spans: Vec<_> = windows.viewer.lines.iter().flat_map(|l| l.spans.iter().filter(|s| s.action.is_some()).map(move |s| (l.y, s.x))).collect();
        let at = |(ly, sx): (i32, i32)| (tutorial::HTML_POS.0 + sx + 1, tutorial::HTML_POS.1 + ly + 1);
        let (x, y) = at(spans[0]);
        windows.mouse_down(WindowId::TUTORIAL_VIEWER, x, y, &font);
        assert_eq!(windows.mouse_up(Some((WindowId::TUTORIAL_VIEWER, x, y)), &mut desktop, &font), vec![NetCommand::TutorialLinkHtml { link: "t2.htm".into() }]);
        let (x, y) = at(spans[1]);
        windows.mouse_down(WindowId::TUTORIAL_VIEWER, x, y, &font);
        assert_eq!(windows.mouse_up(Some((WindowId::TUTORIAL_VIEWER, x, y)), &mut desktop, &font), vec![NetCommand::TutorialPassCmd { command: "tutorial_close".into() }]);
        windows.world_event(&WorldEvent::TutorialCloseHtml(TutorialCloseHtml), &mut desktop, &font, None);
        assert!(!desktop.window(WindowId::TUTORIAL_VIEWER).unwrap().visible);
    }

    #[test]
    fn the_question_mark_glows_over_the_chat_and_reports_the_click() {
        let (font, mut desktop) = (font(), desktop());
        let mut windows = TutorialWindows::default();
        windows.world_event(&WorldEvent::TutorialShowQuestionMark(TutorialShowQuestionMark { mark_id: 9 }), &mut desktop, &font, None);
        let button = desktop.window(WindowId::TUTORIAL_BUTTON).unwrap();
        assert!(button.visible);
        assert_eq!(button.rect, Rect::new(5, 600 - 37, 32, 32));
        assert!(windows.button.effect);
        desktop.window_mut(WindowId::SYSTEM_MSG).unwrap().visible = true;
        let now = Instant::now();
        windows.tick(now, &mut desktop);
        assert_eq!(desktop.window(WindowId::TUTORIAL_BUTTON).unwrap().rect.y, 600 - 167);
        windows.mouse_down(WindowId::TUTORIAL_BUTTON, 3, 3, &font);
        assert_eq!(windows.mouse_up(Some((WindowId::TUTORIAL_BUTTON, 3, 3)), &mut desktop, &font), vec![NetCommand::TutorialQuestionMark { mark_id: 9 }]);
        assert!(!desktop.window(WindowId::TUTORIAL_BUTTON).unwrap().visible);
    }

    #[test]
    fn enabled_client_events_are_reported_once() {
        let (font, mut desktop) = (font(), desktop());
        let mut windows = TutorialWindows::default();
        assert_eq!(windows.fire(tutorial::EVENT_MOVE), None);
        windows.world_event(&WorldEvent::TutorialEnableClientEvent(TutorialEnableClientEvent { event_id: 7 }), &mut desktop, &font, None);
        assert_eq!(windows.fire(tutorial::EVENT_MOVE), Some(NetCommand::TutorialClientEvent { event_id: 1 }));
        assert_eq!(windows.fire(tutorial::EVENT_MOVE), None);
        assert_eq!(windows.camera_rotated(0.001), None);
        assert_eq!(windows.camera_rotated(0.05), Some(NetCommand::TutorialClientEvent { event_id: 2 }));
        assert_eq!(windows.fire(tutorial::EVENT_ZOOM), Some(NetCommand::TutorialClientEvent { event_id: 4 }));
    }
}
