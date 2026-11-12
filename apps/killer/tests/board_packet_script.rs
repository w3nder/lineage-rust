use killer_ui::board::BoardPages;
use killer_ui::scripted::board_wnd::{BoardWnd, BoardWndPacketHost};
use serde_json::{Value, json};
use std::process::Command;

struct Host {
    packet: Value,
    calls: Vec<Value>,
    visible: bool,
    pages: BoardPages,
    document: String,
}

impl BoardWndPacketHost for Host {
    fn parse_int(&mut self, _: &str, key: &str) -> i32 {
        assert_eq!(key, "OK");
        self.packet["flag"].as_i64().unwrap() as i32
    }
    fn parse_string(&mut self, _: &str, key: &str) -> String {
        if key == "Address" {
            self.packet["html"].as_str().unwrap().to_owned()
        } else {
            let index = key.strip_prefix("Command").unwrap().parse::<usize>().unwrap() - 1;
            self.packet["links"][index].as_str().unwrap().to_owned()
        }
    }
    fn uiapi_htmlctrl_set_html_buff_data(&mut self, path: &str, html: &str) {
        assert_eq!(path, "BoardWnd.HtmlViewer");
        self.calls.push(json!(["html", html]));
        if let Some(page) = self.pages.receive(html) {
            self.document = page.text;
        }
    }
    fn uiapi_window_hide_window(&mut self, path: &str) {
        assert_eq!(path, "BoardWnd");
        self.visible = false;
        self.calls.push(json!(["hide"]));
    }
    fn uiapi_window_show_window(&mut self, path: &str) {
        assert_eq!(path, "BoardWnd");
        self.visible = true;
        self.calls.push(json!(["show"]));
    }
    fn uiapi_window_set_focus(&mut self, path: &str) {
        assert_eq!(path, "BoardWnd");
        self.calls.push(json!(["focus"]));
    }
}

#[test]
#[ignore]
fn generated_board_packet_matches_native_receive_set_links_and_html_buffers() {
    let output = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/board_packet_reference.py"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let mut scenarios = 0;
    let mut events = 0;
    for line in String::from_utf8(output.stdout).unwrap().lines() {
        let case: Value = serde_json::from_str(line).unwrap();
        let mut script = BoardWnd {
            m_b_show: case["visible"].as_bool().unwrap(),
            m_b_btn_lock: case["locked"].as_bool().unwrap(),
            m_command: std::array::from_fn(|i| format!("previous-{i}")),
        };
        let mut host = Host {
            packet: Value::Null,
            calls: Vec::new(),
            visible: script.m_b_show,
            pages: BoardPages::default(),
            document: "<html>previous</html>".into(),
        };
        for packet in case["events"].as_array().unwrap() {
            host.packet = packet.clone();
            host.calls.clear();
            script.handle_show_board_packet(&mut host, String::new());
            assert_eq!(json!(host.calls), packet["calls"], "scenario {scenarios}, event {events}");
            assert_eq!(json!(script.m_command), packet["commands"]);
            assert_eq!(script.m_b_btn_lock, packet["locked"].as_bool().unwrap());
            assert_eq!(host.visible, packet["visible"].as_bool().unwrap());
            assert_eq!(host.document, packet["document"].as_str().unwrap(), "scenario {scenarios}, event {events}");
            assert_eq!(
                json!([host.pages.template_count, host.pages.form_count, host.pages.list_count]),
                packet["counts"]
            );
            assert_eq!(json!(host.pages.template), packet["template"]);
            assert_eq!(json!(host.pages.form), packet["form"]);
            assert_eq!(json!(host.pages.list), packet["list"]);
            events += 1;
        }
        scenarios += 1;
    }
    assert_eq!((scenarios, events), (320, 800));
}
