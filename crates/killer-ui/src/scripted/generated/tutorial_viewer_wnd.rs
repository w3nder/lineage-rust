use crate::window::Rect;
#[derive(Debug, Clone, Default)]
pub struct TutorialViewerWnd {
}
pub trait TutorialViewerWndHost {
    fn hide_window(&mut self, arg0: &str);
    fn parse_string(&mut self, arg0: &str, arg1: &str) -> String;
    fn show_window_with_focus(&mut self, arg0: &str);
    fn uiapi_htmlctrl_get_frame_max_height(&mut self, arg0: &str) -> i32;
    fn uiapi_htmlctrl_load_html_from_string(&mut self, arg0: &str, arg1: &str);
    fn uiapi_window_get_rect(&mut self, arg0: &str) -> Rect;
    fn uiapi_window_move_to(&mut self, arg0: &str, arg1: i32, arg2: i32);
    fn uiapi_window_set_window_size(&mut self, arg0: &str, arg1: i32, arg2: i32);
}
impl TutorialViewerWnd {
    pub fn on_event(&mut self, host: &mut impl TutorialViewerWndHost, event_id: i32, param: String) {
        #[derive(Default)]
        struct Locals {
            event_id: i32,
            param: String,
            html_string: String,
            rect: Rect,
            html_height: i32,
        }
        let mut vars = Locals { event_id, param, ..Default::default() };
        match vars.event_id {
            2430 => {
                vars.html_string = host.parse_string(&vars.param, "HtmlString");
                host.uiapi_htmlctrl_load_html_from_string("TutorialViewerWnd.HtmlTutorialViewer", &vars.html_string);
                vars.rect = host.uiapi_window_get_rect("TutorialViewerWnd");
                vars.html_height = host.uiapi_htmlctrl_get_frame_max_height("TutorialViewerWnd.HtmlTutorialViewer");
                vars.html_height = vars.html_height.clamp(256, 680 - 8);
                vars.rect.h = vars.html_height + 8;
                host.uiapi_window_set_window_size("TutorialViewerWnd", vars.rect.w, vars.rect.h);
                host.uiapi_window_set_window_size("TutorialViewerWnd.texTutorialViewerBack2", vars.rect.w, (vars.rect.h - 32) - 9);
                host.uiapi_window_move_to("TutorialViewerWnd.texTutorialViewerBack3", vars.rect.x, (vars.rect.y + vars.rect.h) - 9);
                host.uiapi_window_set_window_size("TutorialViewerWnd.HtmlTutorialViewer", vars.rect.w - 15, (vars.rect.h - 32) - 9);
                host.show_window_with_focus("TutorialViewerWnd");
            }
            2440 => {
                host.hide_window("TutorialViewerWnd");
            }
            _ => {
            }
        }
    }
}
