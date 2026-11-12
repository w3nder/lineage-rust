use std::collections::HashMap;

#[derive(Default)]
pub struct Speech {
    elapsed: f64,
    entries: HashMap<i32, (String, u64)>,
}

impl Speech {
    pub fn receive(&mut self, id: i32, kind: i32, store: Option<u8>, text: &str) {
        if matches!(kind, 6 | 7) || store.is_some_and(|kind| kind != 0) { return; }
        let units: Vec<_> = text.encode_utf16().take_while(|unit| *unit != 0).collect();
        let text = if units.len() < 19 { String::from_utf16_lossy(&units) }
            else { format!("{}...", String::from_utf16_lossy(&units[..18])) };
        self.entries.insert(id, (text, self.elapsed.floor() as u64));
    }

    pub fn tick(&mut self, dt: f32) {
        if dt.is_finite() && dt > 0.0 { self.elapsed += f64::from(dt); }
        let now = self.elapsed.floor() as u64;
        self.entries.retain(|_, (_, stamp)| stamp.saturating_add(5) >= now);
    }

    pub fn text(&self, id: i32) -> Option<&str> {
        self.entries.get(&id).map(|(text, _)| text.as_str()).filter(|text| !text.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_chat_msg_1005d620_channels_store_types_and_utf16_limit() {
        for kind in 0..=255 {
            for store in std::iter::once(None).chain((0..=8).map(Some)) {
                let mut speech = Speech::default();
                speech.receive(1, kind, store, "Olá mundo");
                assert_eq!(speech.text(1).is_some(), !matches!(kind, 6 | 7) && store.is_none_or(|kind| kind == 0));
            }
        }
        let mut speech = Speech::default();
        for length in 0..=1023 {
            speech.receive(1, 0, Some(0), &"a".repeat(length));
            let expected = if length < 19 { "a".repeat(length) } else { format!("{}...", "a".repeat(18)) };
            assert_eq!(speech.text(1).unwrap_or(""), expected);
        }
        speech.receive(1, 0, None, "á漢😀á漢😀á漢😀á漢😀xyz");
        assert_eq!(speech.text(1), Some("á漢😀á漢😀á漢😀á漢😀xy..."));
    }

    #[test]
    fn timer_1005f140_expires_after_five_strict_ticks_and_replaces_by_id() {
        let mut speech = Speech::default();
        speech.receive(1, 0, None, "primeira");
        speech.tick(5.0);
        assert_eq!(speech.text(1), Some("primeira"));
        speech.receive(2, 0, None, "vizinho");
        speech.receive(1, 0, None, "segunda");
        speech.tick(5.0);
        assert_eq!(speech.text(1), Some("segunda"));
        assert_eq!(speech.text(2), Some("vizinho"));
        speech.tick(1.0);
        assert_eq!(speech.text(1), None);
        assert_eq!(speech.text(2), None);
    }
}
