#[derive(Debug, Clone, Default)]
pub struct BoardWnd {
    pub m_b_show: bool,
    pub m_b_btn_lock: bool,
    pub m_command: [String; 8],
}
pub trait BoardWndPacketHost {
    fn parse_int(&mut self, arg0: &str, arg1: &str) -> i32;
    fn parse_string(&mut self, arg0: &str, arg1: &str) -> String;
    fn uiapi_htmlctrl_set_html_buff_data(&mut self, arg0: &str, arg1: &str);
    fn uiapi_window_hide_window(&mut self, arg0: &str);
    fn uiapi_window_set_focus(&mut self, arg0: &str);
    fn uiapi_window_show_window(&mut self, arg0: &str);
}
pub trait BoardWndHost {
    fn uiapi_htmlctrl_controller_execution(&mut self, arg0: &str, arg1: &str) -> i32;
}
impl BoardWnd {
    pub fn handle_show_board_packet(&mut self, host: &mut impl BoardWndPacketHost, param: String) {
        #[derive(Default)]
        struct Locals {
            param: String,
            idx: i32,
            ok: i32,
            address: String,
        }
        let mut vars = Locals { param, ..Default::default() };
        vars.ok = host.parse_int(&vars.param, "OK");
        if vars.ok < 1 {
            host.uiapi_window_hide_window("BoardWnd");
            return;
        }
        vars.address = host.parse_string(&vars.param, "Address");
        host.uiapi_htmlctrl_set_html_buff_data("BoardWnd.HtmlViewer", &vars.address);
        vars.idx = 0;
        self.m_command[0] = "".to_owned();
        self.m_command[1] = "".to_owned();
        self.m_command[2] = "".to_owned();
        self.m_command[3] = "".to_owned();
        self.m_command[4] = "".to_owned();
        self.m_command[5] = "".to_owned();
        self.m_command[6] = "".to_owned();
        self.m_command[7] = "".to_owned();
        vars.idx = 8;
        self.m_command[0] = host.parse_string(&vars.param, "Command1");
        self.m_command[1] = host.parse_string(&vars.param, "Command2");
        self.m_command[2] = host.parse_string(&vars.param, "Command3");
        self.m_command[3] = host.parse_string(&vars.param, "Command4");
        self.m_command[4] = host.parse_string(&vars.param, "Command5");
        self.m_command[5] = host.parse_string(&vars.param, "Command6");
        self.m_command[6] = host.parse_string(&vars.param, "Command7");
        self.m_command[7] = host.parse_string(&vars.param, "Command8");
        self.m_b_btn_lock = false;
        host.uiapi_window_show_window("BoardWnd");
        host.uiapi_window_set_focus("BoardWnd");
    }
    pub fn show_bbs_tab(&mut self, host: &mut impl BoardWndHost, index: i32) {
        #[derive(Default)]
        struct Locals {
            index: i32,
            str_bypass: String,
            ret: i32,
        }
        let mut vars = Locals { index, ..Default::default() };
        match vars.index {
            0 => {
                vars.str_bypass = "bypass _bbshome".to_owned();
            }
            1 => {
                vars.str_bypass = "bypass _bbsgetfav".to_owned();
            }
            2 => {
                vars.str_bypass = "bypass _bbsloc".to_owned();
            }
            3 => {
                vars.str_bypass = "bypass _bbsclan".to_owned();
            }
            4 => {
                vars.str_bypass = "bypass _bbsmemo".to_owned();
            }
            5 => {
                vars.str_bypass = "bypass _maillist_0_1_0_".to_owned();
            }
            6 => {
                vars.str_bypass = "bypass _friendlist_0_".to_owned();
            }
            _ => {
            }
        }
        if ((vars.str_bypass).encode_utf16().count() as i32) > 0 {
            vars.ret = host.uiapi_htmlctrl_controller_execution("BoardWnd.HtmlViewer", &vars.str_bypass);
            if vars.ret == 1 {
                self.m_b_btn_lock = true;
            }
        }
    }
    pub fn on_click_bookmark(&mut self, host: &mut impl BoardWndHost) {
        #[derive(Default)]
        struct Locals {
            ret: i32,
        }
        let mut vars = Locals { ..Default::default() };
        if (((self.m_command[7]).encode_utf16().count() as i32) > 0) && !(self.m_b_btn_lock) {
            vars.ret = host.uiapi_htmlctrl_controller_execution("BoardWnd.HtmlViewer", &self.m_command[7]);
            if vars.ret == 1 {
                self.m_b_btn_lock = true;
            }
        }
    }
}
