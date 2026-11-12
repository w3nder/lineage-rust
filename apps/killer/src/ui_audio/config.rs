use killer_ui::ini::IniFile;
use super::AudioConfigReader;

pub struct AudioIni<'a> {
    pub option: &'a IniFile,
    pub alaudio: &'a IniFile,
}

pub fn native_atof(text: &str) -> f32 {
    let text = text.trim_start();
    let mut end = 0;
    let bytes = text.as_bytes();
    if end < bytes.len() && matches!(bytes[end], b'+' | b'-') { end += 1; }
    while end < bytes.len() && bytes[end].is_ascii_digit() { end += 1; }
    if end < bytes.len() && bytes[end] == b'.' {
        end += 1;
        while end < bytes.len() && bytes[end].is_ascii_digit() { end += 1; }
    }
    if end < bytes.len() && matches!(bytes[end], b'e' | b'E') {
        let mut exponent = end + 1;
        if exponent < bytes.len() && matches!(bytes[exponent], b'+' | b'-') { exponent += 1; }
        if exponent < bytes.len() && bytes[exponent].is_ascii_digit() {
            while exponent < bytes.len() && bytes[exponent].is_ascii_digit() { exponent += 1; }
            end = exponent;
        }
    }
    text[..end].parse::<f64>().map_or(0.0, |value| value as f32)
}

impl AudioIni<'_> {
    fn file(&self, name: &str) -> Option<&IniFile> {
        if name.eq_ignore_ascii_case("Option.ini") { Some(self.option) }
        else if name.eq_ignore_ascii_case("Alaudio.int") { Some(self.alaudio) }
        else { None }
    }
}

impl AudioConfigReader for AudioIni<'_> {
    fn float(&self, file: &str, section: &str, key: &str) -> Option<f32> {
        self.file(file)?.get(section, key).map(native_atof)
    }
    fn integer(&self, file: &str, section: &str, key: &str) -> Option<i32> {
        self.file(file)?.int(section, key)
    }
    fn boolean(&self, file: &str, section: &str, key: &str) -> Option<bool> {
        self.file(file)?.bool(section, key)
    }
}
