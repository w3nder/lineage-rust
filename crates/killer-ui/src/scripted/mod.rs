#[path = "generated/board_wnd.rs"]
pub mod board_wnd;
#[path = "generated/help_html_wnd.rs"]
pub mod help_html_wnd;
#[path = "generated/tutorial_viewer_wnd.rs"]
pub mod tutorial_viewer_wnd;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptSound {
    Open,
    Close,
}

impl<F: FnMut(&str) -> i32> board_wnd::BoardWndHost for F {
    fn uiapi_htmlctrl_controller_execution(&mut self, _: &str, action: &str) -> i32 {
        self(action)
    }
}
