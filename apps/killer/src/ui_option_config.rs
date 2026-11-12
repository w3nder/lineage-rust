use std::io;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEdit {
    pub section: String,
    pub key: String,
    pub value: String,
}

impl ConfigEdit {
    pub fn new(section: &str, key: &str, value: impl Into<String>) -> Self {
        Self { section: section.into(), key: key.into(), value: value.into() }
    }

    pub fn boolean(section: &str, key: &str, value: i32) -> Self {
        Self::new(section, key, if value == 0 { "False" } else { "True" })
    }
}

pub fn edit_ini(text: &str, edits: &[ConfigEdit]) -> String {
    let ending = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    for edit in edits {
        let mut section = false;
        let mut insertion = None;
        let mut found = false;
        for (index, line) in lines.iter_mut().enumerate() {
            let trimmed = line.trim();
            if let Some(name) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                section = name.trim().eq_ignore_ascii_case(&edit.section);
                if section { insertion = Some(index + 1); }
            } else if section {
                insertion = Some(index + 1);
                if !trimmed.starts_with(';') && trimmed.split_once('=').is_some_and(|(key, _)| key.trim().eq_ignore_ascii_case(&edit.key)) {
                    *line = format!("{}={}", edit.key, edit.value);
                    found = true;
                }
            }
        }
        if !found {
            let line = format!("{}={}", edit.key, edit.value);
            if let Some(index) = insertion { lines.insert(index, line); }
            else { lines.extend([format!("[{}]", edit.section), line]); }
        }
    }
    let mut result = lines.join(ending);
    if !result.is_empty() { result.push_str(ending); }
    result
}

pub fn save(path: &Path, edits: &[ConfigEdit]) -> io::Result<String> {
    use std::io::Write;
    let original = std::fs::read(path)?;
    let (text, encoding) = if original.starts_with(&[0xff, 0xfe]) || original.starts_with(&[0xfe, 0xff]) {
        let little = original[0] == 0xff;
        if original.len() % 2 != 0 { return Err(io::Error::new(io::ErrorKind::InvalidData, "ini UTF-16 truncado")); }
        let units: Vec<_> = original[2..].as_chunks::<2>().0.iter().map(|b| if little { u16::from_le_bytes([b[0], b[1]]) } else { u16::from_be_bytes([b[0], b[1]]) }).collect();
        (String::from_utf16(&units).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?, if little { 1 } else { 2 })
    } else {
        let bom = original.starts_with(&[0xef, 0xbb, 0xbf]);
        (String::from_utf8(original[if bom { 3 } else { 0 }..].to_vec()).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?, if bom { 3 } else { 0 })
    };
    let result = edit_ini(&text, edits);
    let bytes = match encoding {
        1 => [vec![0xff, 0xfe], result.encode_utf16().flat_map(u16::to_le_bytes).collect()].concat(),
        2 => [vec![0xfe, 0xff], result.encode_utf16().flat_map(u16::to_be_bytes).collect()].concat(),
        3 => [vec![0xef, 0xbb, 0xbf], result.as_bytes().to_vec()].concat(),
        _ => result.as_bytes().to_vec(),
    };
    let stamp = killer_platform::time::SystemTime::now().duration_since(killer_platform::time::UNIX_EPOCH).map_err(io::Error::other)?.as_nanos();
    let temporary = path.with_extension(format!("killer-{}-{stamp}.tmp", std::process::id()));
    let mut output = std::fs::OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    let written = (|| {
        output.set_permissions(std::fs::metadata(path)?.permissions())?;
        output.write_all(&bytes)?;
        output.sync_all()?;
        std::fs::rename(&temporary, path)
    })();
    if written.is_err() { let _ = std::fs::remove_file(&temporary); }
    written?;
    Ok(result)
}

pub fn save_or_create(path: &Path, edits: &[ConfigEdit]) -> io::Result<()> {
    match save(path, edits) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => std::fs::write(path, edit_ini("", edits)),
        other => other.map(|_| ()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::ini::parse_ini;

    #[test]
    fn updates_only_the_named_section_and_inserts_missing_keys_inside_it() {
        let text = "; keep\r\n[Video]\r\nGamma=0.8\r\n[Game]\r\nMyName=True\r\n[Other]\r\nGamma=17\r\nMyName=keep\r\n";
        let edits = [ConfigEdit::new("Video", "Gamma", "1.000000"), ConfigEdit::boolean("Game", "MyName", 0), ConfigEdit::new("Game", "PartyLooting", "4"), ConfigEdit::new("Audio", "MusicVolume", "0.400000")];
        let updated = edit_ini(text, &edits);
        let ini = parse_ini(&updated);
        assert_eq!(ini.get("Video", "Gamma"), Some("1.000000"));
        assert_eq!(ini.bool("Game", "MyName"), Some(false));
        assert_eq!(ini.int("Game", "PartyLooting"), Some(4));
        assert_eq!(ini.get("Other", "MyName"), Some("keep"));
        assert_eq!(ini.int("Other", "Gamma"), Some(17));
        assert_eq!(ini.get("Audio", "MusicVolume"), Some("0.400000"));
        assert!(updated.starts_with("; keep\r\n"));
        assert_eq!(edit_ini(&updated, &edits), updated);
    }

    #[test]
    fn save_preserves_encoding_permissions_and_unrelated_options() {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let directory = std::env::temp_dir().join(format!("killer-option-config-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let text = "; opção\r\n[Game]\r\nMyName=True\r\n[Other]\r\nMyName=preservar\r\n";
        let edits = [ConfigEdit::boolean("Game", "MyName", 0)];
        let expected = edit_ini(text, &edits);
        let encode = |text: &str, encoding| match encoding {
            1 => [vec![0xff, 0xfe], text.encode_utf16().flat_map(u16::to_le_bytes).collect()].concat(),
            2 => [vec![0xfe, 0xff], text.encode_utf16().flat_map(u16::to_be_bytes).collect()].concat(),
            3 => [vec![0xef, 0xbb, 0xbf], text.as_bytes().to_vec()].concat(),
            _ => text.as_bytes().to_vec(),
        };
        for encoding in 0..4 {
            let path = directory.join(format!("option-{encoding}.ini"));
            std::fs::write(&path, encode(text, encoding)).unwrap();
            let permissions = std::fs::metadata(&path).unwrap().permissions();
            assert_eq!(save(&path, &edits).unwrap(), expected);
            assert_eq!(std::fs::read(&path).unwrap(), encode(&expected, encoding));
            assert_eq!(std::fs::metadata(&path).unwrap().permissions(), permissions);
            std::fs::remove_file(path).unwrap();
        }
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
        std::fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn save_rejects_invalid_encoding_without_overwriting_original() {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let directory = std::env::temp_dir().join(format!("killer-option-invalid-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("option.ini");
        let edits = [ConfigEdit::boolean("Game", "MyName", 1)];
        for bytes in [vec![0xff, 0xfe, 0x00], vec![0xff, 0xfe, 0x00, 0xd8], vec![0xff]] {
            std::fs::write(&path, &bytes).unwrap();
            assert_eq!(save(&path, &edits).unwrap_err().kind(), io::ErrorKind::InvalidData);
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
            assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
        }
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
