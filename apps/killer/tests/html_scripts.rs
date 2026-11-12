use killer_ui::scripted::board_wnd::BoardWnd;
use killer_ui::scripted::help_html_wnd::{HelpHtmlWnd, HelpHtmlWndHost};
use killer_ui::scripted::tutorial_viewer_wnd::{TutorialViewerWnd, TutorialViewerWndHost};
use killer_ui::window::Rect;
use serde_json::{Value, json};
use std::process::Command;

#[derive(Default)]
struct Host {
    trace: Vec<Value>,
    height: i32,
    rect: Rect,
    path: String,
    html: String,
    body: (i32, i32),
    bottom: (i32, i32),
}

impl HelpHtmlWndHost for Host {
    fn parse_string(&mut self, _: &str, key: &str) -> String {
        match key {
            "FilePath" => self.path.clone(),
            "HtmlString" => self.html.clone(),
            _ => panic!("{key}"),
        }
    }
    fn uiapi_htmlctrl_load_html(&mut self, path: &str, file: &str) {
        assert_eq!(path, "HelpHtmlWnd.HtmlViewer");
        self.trace.push(json!(["load", file]));
    }
    fn uiapi_htmlctrl_load_html_from_string(&mut self, path: &str, html: &str) {
        assert_eq!(path, "HelpHtmlWnd.HtmlViewer");
        self.trace.push(json!(["html", html]));
    }
    fn uiapi_window_hide_window(&mut self, path: &str) {
        assert_eq!(path, "HelpHtmlWnd");
        self.trace.push(json!(["hide"]));
    }
    fn uiapi_window_show_window(&mut self, path: &str) {
        assert_eq!(path, "HelpHtmlWnd");
        self.trace.push(json!(["show"]));
    }
    fn uiapi_window_set_focus(&mut self, path: &str) {
        assert_eq!(path, "HelpHtmlWnd");
        self.trace.push(json!(["focus"]));
    }
}

impl TutorialViewerWndHost for Host {
    fn parse_string(&mut self, _: &str, key: &str) -> String {
        assert_eq!(key, "HtmlString");
        self.html.clone()
    }
    fn hide_window(&mut self, path: &str) {
        assert_eq!(path, "TutorialViewerWnd");
        self.trace.push(json!(["hide"]));
    }
    fn show_window_with_focus(&mut self, path: &str) {
        assert_eq!(path, "TutorialViewerWnd");
    }
    fn uiapi_htmlctrl_get_frame_max_height(&mut self, path: &str) -> i32 {
        assert_eq!(path, "TutorialViewerWnd.HtmlTutorialViewer");
        self.height
    }
    fn uiapi_htmlctrl_load_html_from_string(&mut self, path: &str, _: &str) {
        assert_eq!(path, "TutorialViewerWnd.HtmlTutorialViewer");
    }
    fn uiapi_window_get_rect(&mut self, path: &str) -> Rect {
        assert_eq!(path, "TutorialViewerWnd");
        self.rect
    }
    fn uiapi_window_move_to(&mut self, path: &str, x: i32, y: i32) {
        assert_eq!(path, "TutorialViewerWnd.texTutorialViewerBack3");
        self.bottom = (x, y);
    }
    fn uiapi_window_set_window_size(&mut self, path: &str, w: i32, h: i32) {
        match path {
            "TutorialViewerWnd" => {
                self.rect.w = w;
                self.rect.h = h;
                self.trace.push(json!(["resize", w, h]));
            }
            "TutorialViewerWnd.HtmlTutorialViewer" => self.trace.push(json!(["html_size", w, h])),
            "TutorialViewerWnd.texTutorialViewerBack2" => self.body = (w, h),
            _ => panic!("{path}"),
        }
    }
}

#[test]
fn generated_board_ignores_indices_outside_the_seven_tabs() {
    let mut script = BoardWnd::default();
    for index in [i32::MIN, -1, 7, i32::MAX] {
        script.show_bbs_tab(&mut |_: &str| panic!("invalid tab sent a command"), index);
    }
    assert!(!script.m_b_btn_lock);
}

#[test]
fn generated_help_ignores_empty_inputs_and_tracks_visibility_callbacks() {
    let mut host = Host::default();
    let mut script = HelpHtmlWnd::default();
    script.handle_show_help(&mut host, String::new());
    script.handle_load_help_html(&mut host, String::new());
    assert!(host.trace.is_empty());
    script.on_show();
    script.handle_show_help(&mut host, String::new());
    assert_eq!(host.trace, [json!(["hide"])]);
    script.on_hide();
    host.html = "<html>á & \"text\"</html>".into();
    script.handle_load_help_html(&mut host, String::new());
    assert_eq!(host.trace.last(), Some(&json!(["html", host.html])));
}

#[test]
fn generated_tutorial_dispatch_does_not_handle_unknown_events() {
    let mut host = Host::default();
    let mut script = TutorialViewerWnd::default();
    script.on_event(&mut host, -1, String::new());
    assert!(host.trace.is_empty());
    script.on_event(&mut host, 2440, String::new());
    assert_eq!(host.trace, [json!(["hide"])]);
}

#[test]
#[ignore]
fn generated_handlers_match_complete_native_board_help_and_tutorial_branches() {
    let output = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/html_scripts_reference.py"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let mut counts = [0; 4];
    for line in String::from_utf8(output.stdout).unwrap().lines() {
        let case: Value = serde_json::from_str(line).unwrap();
        let mut host = Host { rect: Rect::new(694, 20, 310, 401), ..Default::default() };
        match case["kind"].as_str().unwrap() {
            kind @ ("tab" | "favorite") => {
                let mut script = BoardWnd { m_b_btn_lock: case["before"].as_bool().unwrap(), ..Default::default() };
                let result = case["result"].as_i64().unwrap() as i32;
                let mut execute = |action: &str| {
                    host.trace.push(json!(["action", action]));
                    result
                };
                if kind == "tab" {
                    counts[0] += 1;
                    script.show_bbs_tab(&mut execute, case["index"].as_i64().unwrap() as i32);
                } else {
                    counts[1] += 1;
                    script.m_command[7] = case["link"].as_str().unwrap().into();
                    script.on_click_bookmark(&mut execute);
                }
                assert_eq!(script.m_b_btn_lock, case["locked"].as_bool().unwrap(), "{case}");
            }
            "help" => {
                counts[2] += 1;
                let mut script = HelpHtmlWnd { m_b_show: case["visible"].as_bool().unwrap() };
                host.path = if case["macro"].as_bool().unwrap() {
                    "..\\L2text\\help_macro.htm"
                } else {
                    "..\\L2text\\help.htm"
                }
                .into();
                script.handle_show_help(&mut host, String::new());
            }
            "tutorial" => {
                counts[3] += 1;
                host.height = case["height"].as_i64().unwrap() as i32;
                TutorialViewerWnd::default().on_event(&mut host, 2430, String::new());
                assert_eq!(host.body, (310, host.rect.h - 41));
                assert_eq!(host.bottom, (694, 20 + host.rect.h - 9));
            }
            _ => panic!("{case}"),
        }
        assert_eq!(json!(host.trace), case["calls"], "{case}");
    }
    assert_eq!(counts, [42, 18, 4, 1028]);
}
