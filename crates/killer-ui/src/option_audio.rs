use crate::color::TEXT;
use crate::combo::ComboBox;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

pub const CONTROL_IDS: [u32; 4] = [0x1f12, 0x1f16, 0x1f22, 0x1f23];
pub const LABEL_IDS: [u32; 4] = [216, 217, 977, 978];
pub const CHOICE_IDS: [u32; 6] = [1233, 1234, 1235, 1236, 1237, 228];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChannel {
    Sound,
    Music,
    WavVoice,
    OggVoice,
}

pub trait AudioApplyBackend {
    fn available(&mut self) -> bool;
    fn volume(&mut self, channel: AudioChannel) -> f32;
    fn set_volume(&mut self, channel: AudioChannel, value: f32);
    fn stop(&mut self, channel: AudioChannel);
    fn has_music_controller(&mut self) -> bool;
    fn reset_music_controller(&mut self);
    fn restore_lobby_music(&mut self);
    fn write_volume(&mut self, file: &str, section: &str, key: &str, value: f32);
}

#[derive(Debug, Clone)]
pub struct AudioPage {
    pub labels: [String; 4],
    pub combos: [ComboBox; 4],
}

impl AudioPage {
    pub fn clear(&mut self) {
        for combo in &mut self.combos { combo.clear_entries(); }
    }

    pub fn fill(&mut self, volumes: Option<[f32; 4]>, lookup: impl Fn(u32) -> Option<String>) {
        for combo in &mut self.combos {
            combo.entries.extend(CHOICE_IDS.map(|id| lookup(id).unwrap_or_default()));
        }
        self.read_volumes(volumes);
    }

    pub fn apply(&self, backend: &mut impl AudioApplyBackend) {
        for (index, (channel, key)) in [
            (AudioChannel::Sound, "SoundVolume"),
            (AudioChannel::Music, "MusicVolume"),
            (AudioChannel::WavVoice, "WavVoiceVolume"),
            (AudioChannel::OggVoice, "OggVoiceVolume"),
        ].into_iter().enumerate() {
            if !backend.available() {
                continue;
            }
            let old_music = (channel == AudioChannel::Music).then(|| backend.volume(channel));
            let selected = self.combos[index].selected;
            if let Some(value) = [1.0, 0.8, 0.6, 0.4, 0.2, 0.0].get(selected as usize) {
                backend.set_volume(channel, *value);
                if selected == 5 {
                    backend.stop(channel);
                }
            }
            if let Some(previous) = old_music {
                let current = backend.volume(channel);
                if current > 0.0 && previous == 0.0 {
                    if backend.has_music_controller() {
                        backend.reset_music_controller();
                    }
                    backend.restore_lobby_music();
                }
            }
            let value = backend.volume(channel);
            backend.write_volume("Option.ini", "Audio", key, value);
        }
    }

    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let entries: Vec<_> = CHOICE_IDS.iter().map(|id| lookup(*id).unwrap_or_default()).collect();
        Self {
            labels: LABEL_IDS.map(|id| lookup(id).unwrap_or_default()),
            combos: std::array::from_fn(|index| ComboBox::new(Rect::new(107, 11 + 19 * index as i32, 125, 17), entries.clone(), 0)),
        }
    }

    pub fn set_font(&mut self, font: &BitmapFont) {
        let height = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
        for combo in &mut self.combos {
            combo.text_height = height;
        }
    }

    pub fn read_volumes(&mut self, volumes: Option<[f32; 4]>) {
        for (index, combo) in self.combos.iter_mut().enumerate() {
            match volumes {
                None => combo.enabled = false,
                Some(values) => {
                    if let Some(selected) = volume_selection(values[index]) {
                        combo.selected = selected;
                    }
                }
            }
        }
    }

    pub fn paint_labels(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        for (index, label) in self.labels.iter().enumerate() {
            list.draw_text(origin.0 + 98 - font.measure(label) as i32, origin.1 + 14 + 19 * index as i32, TEXT, label, font);
        }
    }
}

pub fn volume_selection(volume: f32) -> Option<i32> {
    if volume >= 1.0 {
        Some(0)
    } else {
        [0.0, 0.2, 0.4, 0.6, 0.8, 1.0].windows(2).enumerate()
            .find(|(_, bounds)| volume >= bounds[0] && volume < bounds[1])
            .map(|(index, _)| 5 - index as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_clear_100c4170_fill_100c46e0_preserve_native_lifecycle() {
        let mut page = AudioPage::new(|id| Some(id.to_string()));
        page.combos[0].open_list();
        page.combos[0].hovered = 4;
        page.combos[0].selected = 3;
        page.combos[1].enabled = false;
        let expanded = page.combos[0].expanded_rect();
        page.clear();
        for combo in &page.combos {
            assert!(combo.entries.is_empty());
            assert_eq!((combo.selected, combo.hovered), (-1, -1));
        }
        assert!(page.combos[0].open);
        assert_eq!(page.combos[0].expanded_rect(), expanded);
        assert!(!page.combos[1].enabled);
        for _ in 0..3 {
            page.clear();
            page.fill(Some([1.0, 0.8, 0.4, 0.0]), |id| Some(id.to_string()));
            for (combo, expected) in page.combos.iter().zip([0, 1, 3, 5]) {
                assert_eq!(combo.entries, CHOICE_IDS.map(|id| id.to_string()));
                assert_eq!(combo.selected, expected);
            }
            assert!(!page.combos[1].enabled);
        }
        page.clear();
        page.fill(None, |id| Some(id.to_string()));
        assert!(page.combos.iter().all(|combo| !combo.enabled && combo.selected == -1 && combo.entries.len() == 6));
        page.fill(None, |id| Some(id.to_string()));
        assert!(page.combos.iter().all(|combo| combo.entries.len() == 12));
    }
    use crate::font::{Glyphs, gly};
    use std::collections::VecDeque;

    #[derive(Debug, PartialEq)]
    enum Event {
        Available(bool),
        Read(AudioChannel),
        Set(AudioChannel, f32),
        Stop(AudioChannel),
        Controller,
        Reset,
        Lobby,
        Write(String, String, String, f32),
    }

    struct Backend {
        available: VecDeque<bool>,
        volumes: [f32; 4],
        music_readings: VecDeque<f32>,
        has_controller: bool,
        events: Vec<Event>,
        setter_scale: f32,
    }

    impl Backend {
        fn new() -> Self {
            Self { available: [true; 4].into(), volumes: [0.5; 4], music_readings: VecDeque::new(), has_controller: true, events: Vec::new(), setter_scale: 1.0 }
        }
    }

    impl AudioApplyBackend for Backend {
        fn available(&mut self) -> bool {
            let value = self.available.pop_front().expect("unexpected availability query");
            self.events.push(Event::Available(value));
            value
        }
        fn volume(&mut self, channel: AudioChannel) -> f32 {
            self.events.push(Event::Read(channel));
            if channel == AudioChannel::Music && let Some(value) = self.music_readings.pop_front() { return value; }
            self.volumes[channel as usize]
        }
        fn set_volume(&mut self, channel: AudioChannel, value: f32) {
            self.events.push(Event::Set(channel, value));
            self.volumes[channel as usize] = value * self.setter_scale;
        }
        fn stop(&mut self, channel: AudioChannel) { self.events.push(Event::Stop(channel)); }
        fn has_music_controller(&mut self) -> bool { self.events.push(Event::Controller); self.has_controller }
        fn reset_music_controller(&mut self) { self.events.push(Event::Reset); }
        fn restore_lobby_music(&mut self) { self.events.push(Event::Lobby); }
        fn write_volume(&mut self, file: &str, section: &str, key: &str, value: f32) { self.events.push(Event::Write(file.into(), section.into(), key.into(), value)); }
    }

    #[test]
    fn audio_apply_100c7250_all_indices_set_stop_read_and_persist_in_native_order() {
        for selected in [i32::MIN, -1, 0, 1, 2, 3, 4, 5, 6, i32::MAX] {
            let mut page = AudioPage::new(|_| None);
            for combo in &mut page.combos { combo.selected = selected; combo.enabled = false; }
            let mut backend = Backend::new();
            backend.setter_scale = 0.5;
            page.apply(&mut backend);
            let mut expected = Vec::new();
            for (channel, key) in [(AudioChannel::Sound, "SoundVolume"), (AudioChannel::Music, "MusicVolume"), (AudioChannel::WavVoice, "WavVoiceVolume"), (AudioChannel::OggVoice, "OggVoiceVolume")] {
                expected.push(Event::Available(true));
                if channel == AudioChannel::Music { expected.push(Event::Read(channel)); }
                let setting = [1.0, 0.8, 0.6, 0.4, 0.2, 0.0].get(selected as usize).copied();
                if let Some(value) = setting {
                    expected.push(Event::Set(channel, value));
                    if selected == 5 { expected.push(Event::Stop(channel)); }
                }
                if channel == AudioChannel::Music { expected.push(Event::Read(channel)); }
                expected.push(Event::Read(channel));
                expected.push(Event::Write("Option.ini".into(), "Audio".into(), key.into(), setting.map_or(0.5, |value| value * 0.5)));
            }
            assert_eq!(backend.events, expected, "selection {selected}");
            assert!(backend.available.is_empty());
        }
    }

    #[test]
    fn audio_apply_100c7250_rechecks_backend_presence_for_each_channel() {
        for mask in 0..16 {
            let page = AudioPage::new(|_| None);
            let mut backend = Backend::new();
            backend.available = (0..4).map(|index| mask & (1 << index) != 0).collect();
            page.apply(&mut backend);
            let keys: Vec<_> = backend.events.iter().filter_map(|event| match event { Event::Write(_, _, key, _) => Some(key.as_str()), _ => None }).collect();
            let expected: Vec<_> = ["SoundVolume", "MusicVolume", "WavVoiceVolume", "OggVoiceVolume"].into_iter().enumerate().filter_map(|(index, key)| (mask & (1 << index) != 0).then_some(key)).collect();
            assert_eq!(keys, expected);
            assert_eq!(backend.events.iter().filter(|event| matches!(event, Event::Available(_))).count(), 4);
            if mask == 0 { assert!(backend.events.iter().all(|event| *event == Event::Available(false))); }
        }
    }

    #[test]
    fn audio_apply_100c7250_music_resumes_only_zero_to_positive_then_reads_again() {
        for old in [0.0, -0.0, -1.0, 0.2, f32::NAN, f32::INFINITY] {
            for current in [0.0, -1.0, 0.2, f32::NAN, f32::INFINITY] {
                for controller in [false, true] {
                    let page = AudioPage::new(|_| None);
                    let mut backend = Backend::new();
                    backend.available = [false, true, false, false].into();
                    backend.music_readings = [old, current, 0.37].into();
                    backend.has_controller = controller;
                    page.apply(&mut backend);
                    let resumes = old == 0.0 && current > 0.0;
                    let mut expected = vec![Event::Available(false), Event::Available(true), Event::Read(AudioChannel::Music), Event::Read(AudioChannel::Music)];
                    if resumes {
                        expected.push(Event::Controller);
                        if controller { expected.push(Event::Reset); }
                        expected.push(Event::Lobby);
                    }
                    expected.extend([
                        Event::Read(AudioChannel::Music),
                        Event::Write("Option.ini".into(), "Audio".into(), "MusicVolume".into(), 0.37),
                        Event::Available(false), Event::Available(false),
                    ]);
                    assert_eq!(backend.events, expected);
                    assert!(backend.music_readings.is_empty());
                }
            }
        }
    }

    #[test]
    fn audio_create_100c9460_and_fill_100c46e0_define_every_control_and_choice() {
        let page = AudioPage::new(|id| Some(id.to_string()));
        assert_eq!(page.labels, ["216", "217", "977", "978"]);
        for (index, combo) in page.combos.iter().enumerate() {
            assert_eq!(combo.rect, Rect::new(107, 11 + 19 * index as i32, 125, 17));
            assert_eq!(combo.entries, ["1233", "1234", "1235", "1236", "1237", "228"]);
        }
    }

    #[test]
    fn audio_read_100c41a0_uses_lower_inclusive_float_intervals_not_rounding() {
        assert_eq!(volume_selection(-0.0), Some(5));
        for (index, value) in [0.2f32, 0.4, 0.6, 0.8, 1.0].into_iter().enumerate() {
            assert_eq!(volume_selection(f32::from_bits(value.to_bits() - 1)), Some(5 - index as i32));
            assert_eq!(volume_selection(value), Some(4 - index as i32));
            assert_eq!(volume_selection(f32::from_bits(value.to_bits() + 1)), Some(4 - index as i32));
        }
        assert_eq!(volume_selection(-1.0), None);
        assert_eq!(volume_selection(f32::NAN), None);
        assert_eq!(volume_selection(f32::NEG_INFINITY), None);
        assert_eq!(volume_selection(f32::INFINITY), Some(0));
    }

    #[test]
    fn audio_read_100c41a0_preserves_invalid_values_and_disables_without_backend() {
        let mut page = AudioPage::new(|_| None);
        page.read_volumes(Some([1.0, 0.4, 0.0, 0.8]));
        assert_eq!(page.combos.each_ref().map(|c| c.selected), [0, 3, 5, 1]);
        page.read_volumes(Some([f32::NAN, -1.0, -0.1, f32::NEG_INFINITY]));
        assert_eq!(page.combos.each_ref().map(|c| c.selected), [0, 3, 5, 1]);
        page.read_volumes(None);
        assert!(page.combos.iter().all(|c| !c.enabled));
        assert_eq!(page.combos.each_ref().map(|c| c.selected), [0, 3, 5, 1]);
        page.read_volumes(Some([0.0; 4]));
        assert!(page.combos.iter().all(|c| !c.enabled));
    }

    #[test]
    fn audio_onpaint_100c6e70_right_aligns_all_labels_and_measures_a_for_the_combos() {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        let font = BitmapFont::new("font".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap());
        let mut page = AudioPage::new(|id| Some(id.to_string()));
        page.set_font(&font);
        assert!(page.combos.iter().all(|c| c.text_height == 13));
        let mut list = DrawList::default();
        page.paint_labels(&mut list, (20, 48), &font);
        assert_eq!(list.quads.len(), 12);
        for (index, row) in list.quads.chunks(3).enumerate() {
            assert_eq!((row[0].x, row[0].y), (100.0, (62 + index * 19) as f32));
            assert_eq!(row[2].x + row[2].w, 118.0);
            assert!(row.iter().all(|q| q.color == TEXT.rgba()));
        }
    }
}
