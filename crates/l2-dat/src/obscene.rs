use crate::{DatReader, Error};

pub fn read(body: &[u8]) -> Result<Vec<String>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| reader.ascf()).collect()
}

const FOOTER: &str = "SafePackage";

pub fn read_as(body: &[u8], chronicle: crate::Chronicle) -> Result<Vec<String>, Error> {
    match chronicle {
        crate::Chronicle::C4 => read(body),
        crate::Chronicle::Interlude => Ok(read_interlude(body)?.into_iter().map(|(_, word)| word).collect()),
    }
}

pub fn read_interlude(body: &[u8]) -> Result<Vec<(u32, String)>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    let words = (0..count).map(|_| Ok((reader.u32()?, reader.ascf()?))).collect::<Result<Vec<_>, Error>>()?;
    let at = body.len() - reader.remaining();
    if reader.ascf()? != FOOTER || reader.remaining() != 0 {
        return Err(Error::InvalidString(at));
    }
    Ok(words)
}


pub fn mask(words: &[String], text: &str, replacement: &str) -> (String, bool) {
    let mut text: Vec<char> = text.chars().collect();
    let mut lowered: Vec<char> = text.iter().map(|c| c.to_ascii_lowercase()).collect();
    let replacement: Vec<char> = replacement.chars().collect();
    let mut found = false;
    for word in words.iter().filter(|word| !word.is_empty()) {
        let word: Vec<char> = word.chars().collect();
        while let Some(at) = lowered.windows(word.len()).position(|window| window == word.as_slice()) {
            found = true;
            let end = at + word.len();
            text.splice(at..end, replacement.iter().copied());
            lowered.splice(at..end, replacement.iter().copied());
            if replacement.windows(word.len()).any(|window| window == word.as_slice()) {
                break;
            }
        }
    }
    (text.into_iter().collect(), found)
}

pub fn contains(words: &[String], name: &str) -> bool {
    let lowered: String = name.chars().map(|c| c.to_ascii_lowercase()).collect();
    words.iter().any(|word| lowered.contains(word.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ascii(s: &str) -> Vec<u8> {
        let mut out = vec![s.len() as u8 + 1];
        out.extend(s.bytes());
        out.push(0);
        out
    }

    #[test]
    fn the_list_is_a_counted_run_of_strings_and_matches_inside_the_lowercased_name() {
        let mut body = 2u32.to_le_bytes().to_vec();
        body.extend(ascii("bad"));
        body.extend(ascii("worse"));
        let words = read(&body).unwrap();
        assert_eq!(words, ["bad", "worse"]);
        assert!(contains(&words, "xBADx"));
        assert!(contains(&words, "WORSE"));
        assert!(!contains(&words, "Good"));
        assert!(!contains(&[], "Anything"));
    }

    #[test]
    fn mask_replaces_every_occurrence_case_insensitively_like_hasbadword() {
        let words = vec!["bad".to_string(), String::new(), "ugly".to_string()];
        let (text, found) = mask(&words, "A BAD Day, badly UGLY", "***");
        assert!(found);
        assert_eq!(text, "A *** Day, ***ly ***");
        assert_eq!(mask(&words, "fine", "***"), ("fine".to_string(), false));
    }

    #[test]
    fn interlude_entries_are_id_then_string_and_end_with_the_footer() {
        let mut body = 2u32.to_le_bytes().to_vec();
        for (id, word) in [(1u32, "923"), (2, "bitch")] {
            body.extend(id.to_le_bytes());
            body.extend(ascii(word));
        }
        body.extend(ascii(FOOTER));
        assert_eq!(read_interlude(&body).unwrap(), [(1, "923".to_string()), (2, "bitch".to_string())]);
        assert_eq!(read_as(&body, crate::Chronicle::Interlude).unwrap(), ["923", "bitch"]);
        assert!(read_interlude(&body[..body.len() - 1]).is_err());
        body.push(0);
        assert!(read_interlude(&body).is_err());
    }
}
