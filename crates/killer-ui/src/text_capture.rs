pub const CAPTURE_COLOR: u32 = 0xFFDCDCDC;
pub const END_LINE: &str = "=====[ Text Capture End ]=====";
pub const EXEC_COMMAND: &str = "TextCapture";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaptureState {
    #[default]
    Idle,
    StartAsked,
    Capturing,
    StopAsked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextCapture {
    pub state: CaptureState,
    pub file_name: String,
    content: Vec<u8>,
    dirty: bool,
}

pub fn file_name(stamp: Stamp) -> String {
    format!("L2_{:02}_{:02}_{:02}_{:02}.log", stamp.month, stamp.day, stamp.hour, stamp.minute)
}

pub fn header_line(stamp: Stamp) -> String {
    format!("[ L2 Text Capture] [ {:02}/{:02} {:02}:{:02}:{:02} ]", stamp.month, stamp.day, stamp.hour, stamp.minute, stamp.second)
}

pub fn start_line(file: &str) -> String {
    format!("[ Text Capture Start ][ FileName : {file} ]")
}

fn utf16(text: &str) -> Vec<u8> {
    text.encode_utf16().flat_map(u16::to_le_bytes).collect()
}

impl TextCapture {
    pub fn exec(&mut self, command: &str) -> bool {
        let mut words = command.split_whitespace();
        if !words.next().is_some_and(|word| word.eq_ignore_ascii_case(EXEC_COMMAND)) {
            return false;
        }
        match words.next() {
            Some(word) if word.eq_ignore_ascii_case("on") => {
                if self.state == CaptureState::Idle {
                    self.state = CaptureState::StartAsked;
                }
                true
            }
            Some(word) if word.eq_ignore_ascii_case("off") => {
                if self.state == CaptureState::Capturing {
                    self.state = CaptureState::StopAsked;
                }
                true
            }
            _ => false,
        }
    }

    fn write_line(&mut self, text: &str) {
        self.content.extend(utf16(text));
        self.content.extend(utf16("\r\n"));
        self.dirty = true;
    }

    pub fn update(&mut self, stamp: Stamp) -> Option<String> {
        match self.state {
            CaptureState::StartAsked => {
                self.file_name = file_name(stamp);
                self.content = vec![0xFF, 0xFE];
                self.write_line(&header_line(stamp));
                self.state = CaptureState::Capturing;
                Some(start_line(&self.file_name))
            }
            CaptureState::StopAsked => {
                self.state = CaptureState::Idle;
                Some(END_LINE.to_string())
            }
            _ => None,
        }
    }

    pub fn message(&mut self, text: &str, shown_by_current_tab: bool) {
        if self.state == CaptureState::Capturing && shown_by_current_tab {
            self.write_line(text);
        }
    }

    pub fn take_write(&mut self) -> Option<(String, Vec<u8>)> {
        std::mem::take(&mut self.dirty).then(|| (self.file_name.clone(), self.content.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STAMP: Stamp = Stamp { month: 3, day: 9, hour: 7, minute: 5, second: 2 };

    #[test]
    fn exec_on_starts_only_from_idle_and_off_stops_only_while_capturing() {
        let mut capture = TextCapture::default();
        assert!(!capture.exec("stat fps"));
        assert!(!capture.exec("TextCapture maybe"));
        assert!(capture.exec("textcapture OFF"));
        assert_eq!(capture.state, CaptureState::Idle);
        assert!(capture.exec("TEXTCAPTURE on"));
        assert_eq!(capture.state, CaptureState::StartAsked);
        assert_eq!(capture.update(STAMP), Some("[ Text Capture Start ][ FileName : L2_03_09_07_05.log ]".to_string()));
        assert_eq!(capture.state, CaptureState::Capturing);
        assert!(capture.exec("TextCapture ON"));
        assert_eq!(capture.state, CaptureState::Capturing);
        assert!(capture.exec("TextCapture OFF"));
        assert_eq!(capture.state, CaptureState::StopAsked);
        assert_eq!(capture.update(STAMP), Some(END_LINE.to_string()));
        assert_eq!(capture.state, CaptureState::Idle);
        assert_eq!(capture.update(STAMP), None);
    }

    #[test]
    fn the_file_is_utf16_with_bom_the_header_and_only_the_lines_the_tab_shows() {
        let mut capture = TextCapture::default();
        capture.message("ignored", true);
        capture.exec("TextCapture ON");
        capture.update(STAMP);
        capture.message("Bob: oi", true);
        capture.message("hidden", false);
        let (name, bytes) = capture.take_write().unwrap();
        assert_eq!(name, "L2_03_09_07_05.log");
        assert_eq!(&bytes[..2], &[0xFF, 0xFE]);
        let text: Vec<u16> = bytes[2..].chunks(2).map(|pair| u16::from_le_bytes([pair[0], pair[1]])).collect();
        assert_eq!(String::from_utf16(&text).unwrap(), "[ L2 Text Capture] [ 03/09 07:05:02 ]\r\nBob: oi\r\n");
        assert!(capture.take_write().is_none());
    }
}
