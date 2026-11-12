#[derive(Debug, Clone, Default)]
pub struct HelpHtmlWnd {
    pub m_b_show: bool,
}
pub trait HelpHtmlWndHost {
    fn parse_string(&mut self, arg0: &str, arg1: &str) -> String;
    fn uiapi_htmlctrl_load_html(&mut self, arg0: &str, arg1: &str);
    fn uiapi_htmlctrl_load_html_from_string(&mut self, arg0: &str, arg1: &str);
    fn uiapi_window_hide_window(&mut self, arg0: &str);
    fn uiapi_window_set_focus(&mut self, arg0: &str);
    fn uiapi_window_show_window(&mut self, arg0: &str);
}
impl HelpHtmlWnd {
    pub fn on_show(&mut self) {
        self.m_b_show = true;
    }
    pub fn on_hide(&mut self) {
        self.m_b_show = false;
    }
    pub fn handle_show_help(&mut self, host: &mut impl HelpHtmlWndHost, param: String) {
        #[derive(Default)]
        struct Locals {
            param: String,
            str_path: String,
        }
        let mut vars = Locals { param, ..Default::default() };
        if self.m_b_show {
            host.uiapi_window_hide_window("HelpHtmlWnd");
        } else {
            vars.str_path = host.parse_string(&vars.param, "FilePath");
            if ((vars.str_path).encode_utf16().count() as i32) > 0 {
                host.uiapi_htmlctrl_load_html("HelpHtmlWnd.HtmlViewer", &vars.str_path);
                host.uiapi_window_show_window("HelpHtmlWnd");
                host.uiapi_window_set_focus("HelpHtmlWnd");
            }
        }
    }
    pub fn handle_load_help_html(&mut self, host: &mut impl HelpHtmlWndHost, param: String) {
        #[derive(Default)]
        struct Locals {
            param: String,
            str_html: String,
        }
        let mut vars = Locals { param, ..Default::default() };
        vars.str_html = host.parse_string(&vars.param, "HtmlString");
        if ((vars.str_html).encode_utf16().count() as i32) > 0 {
            host.uiapi_htmlctrl_load_html_from_string("HelpHtmlWnd.HtmlViewer", &vars.str_html);
        }
    }
}
