use crate::button::CheckBox;
use crate::color::TEXT;
use crate::combo::ComboBox;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

pub const CONTROL_IDS: [u32; 13] = [0x1f08, 0x1f14, 0x1f09, 0x1f0a, 0x1f18, 0x1f19, 0x1f0d, 0x1f0e, 0x1f17, 0x1f11, 0x1f20, 0x1f24, 0x1f25];
pub const LABEL_IDS: [u32; 13] = [209, 351, 210, 211, 594, 607, 214, 215, 540, 352, 868, 979, 1229];
pub const COMBO_Y: [i32; 13] = [11, 30, 49, 68, 87, 106, 139, 158, 177, 229, 248, 317, 336];
pub const CHECK_IDS: [u32; 4] = [0xf0f01465, 0xf0f01467, 0xf0f01468, 0xf0f01469];
pub const CHECK_LABEL_IDS: [u32; 4] = [370, 212, 213, 464];
pub const CHECK_RECTS: [Rect; 4] = [Rect::new(25, 199, 100, 17), Rect::new(25, 270, 109, 17), Rect::new(135, 270, 109, 17), Rect::new(135, 287, 100, 17)];

pub struct VideoChoices<'a> {
    pub modes: &'a [(i32, i32, i32)],
    pub refresh_rates: &'a [i32],
    pub capability_10: bool,
    pub capability_cc: i32,
    pub screenshot_quality: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VideoSettings {
    pub mode: (i32, i32, i32),
    pub refresh_rate: i32,
    pub client_brightness: Option<f32>,
    pub pawn_clipping: f32,
    pub terrain_clipping: f32,
    pub render_deco: i32,
    pub post_process_type: i32,
    pub projector: i32,
    pub texture_detail: i32,
    pub reduce_pawn_lod: i32,
    pub skip_anim: i32,
    pub water_effect: i32,
    pub water_effect_type: i32,
    pub trilinear: i32,
    pub actor_limit: i32,
    pub keep_min_frame_rate: i32,
    pub antialiasing: i32,
}

pub struct VideoCapabilities<'a> {
    pub modes: &'a [(i32, i32, i32)],
    pub refresh_rates: &'a [i32],
    pub water: i32,
    pub shaders: (i32, i32),
    pub antialiasing: i32,
}

pub fn applied_fog_ranges(terrain: f32, keep_min_frame_rate: i32, mut read: impl FnMut(&str, &str, &str) -> Option<f32>) -> (f32, f32) {
    let (start_key, end_key, start_default, end_default) = if keep_min_frame_rate != 0 || terrain < 5.0 {
        ("StartRange1", "EndRange1", 0.5, 3.5)
    } else if terrain < 6.0 {
        ("StartRange2", "EndRange2", 1.0, 4.5)
    } else if terrain < 7.0 {
        ("StartRange3", "EndRange3", 1.5, 5.5)
    } else if terrain < 8.0 {
        ("StartRange4", "EndRange4", 2.0, 6.5)
    } else if terrain >= 8.0 {
        ("StartRange5", "EndRange5", 2.5, 7.5)
    } else {
        return (1.5, 5.5);
    };
    let start = read("env.int", "FOG", start_key).unwrap_or(start_default);
    let end = read("env.int", "FOG", end_key).unwrap_or(end_default);
    (start, end)
}

#[derive(Debug, Clone)]
pub struct VideoPage {
    pub labels: [String; 13],
    pub combos: [ComboBox; 13],
    pub checks: [CheckBox; 4],
}

impl VideoPage {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self {
            labels: LABEL_IDS.map(|id| lookup(id).unwrap_or_default()),
            combos: COMBO_Y.map(|y| ComboBox::new(Rect::new(107, y, 125, 17), Vec::new(), 0)),
            checks: std::array::from_fn(|index| CheckBox::new(CHECK_RECTS[index], &lookup(CHECK_LABEL_IDS[index]).unwrap_or_default())),
        }
    }

    pub fn clear(&mut self) {
        for index in [0, 1, 2, 3, 4, 5] {
            self.combos[index].clear_entries();
        }
        self.checks[1].checked = false;
        self.checks[2].checked = false;
        for index in [12, 6, 7, 8, 9] {
            self.combos[index].clear_entries();
        }
        self.checks[0].checked = false;
        self.checks[3].checked = false;
        self.combos[10].clear_entries();
        self.combos[11].clear_entries();
    }

    pub fn append_choices(&mut self, values: &VideoChoices<'_>, lookup: impl Fn(u32) -> Option<String>) {
        self.combos[0].entries.extend(values.modes.iter().take(100).map(|(w, h, bits)| format!("{w}*{h} {bits}bit")));
        self.combos[1].entries.extend(values.refresh_rates.iter().take(100).map(|hz| format!("{hz}Hz")));
        for (index, ids) in [
            (2, &[1233, 1234, 1235, 1236, 1237][..]),
            (3, &[224, 225, 226]),
            (4, &[224, 226]),
            (5, &[224, 226]),
            (12, &[869, 867, 1246]),
            (6, &[1238, 1239, 1240, 1241, 1242]),
            (7, &[1238, 1239, 1240, 1241, 1242]),
            (8, &[27, 1233, 1234, 1235, 1236, 1237]),
            (9, &[27, 353, 354]),
        ] {
            self.combos[index].entries.extend(ids.iter().map(|id| lookup(*id).unwrap_or_default()));
        }
        if values.capability_10 {
            self.combos[12].entries.extend([1230, 1231, 1232].map(|id| lookup(id).unwrap_or_default()));
        }
        if (0..=2).contains(&values.capability_cc) {
            self.combos[10].enabled = values.capability_cc != 0;
            self.combos[10].entries.extend((869..=869 + values.capability_cc as u32).map(|id| lookup(id).unwrap_or_default()));
        }
        self.combos[11].entries.extend((980..=983).map(|id| lookup(id).unwrap_or_default()));
        self.combos[11].selected = values.screenshot_quality;
    }

    pub fn set_font(&mut self, font: &BitmapFont) {
        let height = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
        for combo in &mut self.combos {
            combo.text_height = height;
        }
    }

    pub fn refresh_resolution(&mut self, settings: &VideoSettings, modes: impl FnOnce() -> Vec<(i32, i32, i32)>, rates: impl FnOnce((i32, i32, i32)) -> Vec<i32>) {
        self.combos[0].clear_entries();
        self.combos[1].clear_entries();
        self.fill_resolution_modes(settings, modes());
        self.fill_resolution_rates(settings, rates(settings.mode));
    }

    pub(crate) fn fill_resolution_modes(&mut self, settings: &VideoSettings, modes: Vec<(i32, i32, i32)>) {
        for (index, mode) in modes.into_iter().take(100).enumerate() {
            self.combos[0].entries.push(format!("{}*{} {}bit", mode.0, mode.1, mode.2));
            if mode == settings.mode {
                self.combos[0].selected = index as i32;
            }
        }
    }

    pub(crate) fn fill_resolution_rates(&mut self, settings: &VideoSettings, rates: Vec<i32>) {
        for (index, rate) in rates.into_iter().take(100).enumerate() {
            self.combos[1].entries.push(format!("{rate}Hz"));
            if rate == settings.refresh_rate {
                self.combos[1].selected = index as i32;
            }
        }
    }

    pub fn read_settings(&mut self, settings: &mut VideoSettings, caps: &VideoCapabilities<'_>) {
        for (index, mode) in caps.modes.iter().take(100).enumerate() {
            if *mode == settings.mode {
                self.combos[0].selected = index as i32;
            }
        }
        for (index, rate) in caps.refresh_rates.iter().take(100).enumerate() {
            if *rate == settings.refresh_rate {
                self.combos[1].selected = index as i32;
            }
        }
        for (index, value, bounds) in [
            (2, settings.client_brightness, [0.6, 0.8, 1.0, 1.2]),
            (7, Some(settings.pawn_clipping), [1.5, 2.0, 2.5, 3.0]),
            (6, Some(settings.terrain_clipping), [5.0, 6.0, 7.0, 8.0]),
        ] {
            if let Some(selected) = value.and_then(|value| threshold_selection(value, bounds)) {
                self.combos[index].selected = selected;
            }
        }
        self.checks[2].checked = settings.render_deco != 0;
        if !(0..=5).contains(&settings.post_process_type) {
            settings.post_process_type = 0;
        }
        self.combos[12].selected = settings.post_process_type;
        self.checks[1].checked = settings.projector != 0;
        self.combos[3].selected = settings.texture_detail.clamp(0, 2);
        self.combos[4].selected = i32::from(settings.reduce_pawn_lod != 0);
        self.combos[5].selected = i32::from(settings.skip_anim != 0);
        self.combos[9].selected = if settings.water_effect != 0 && (1..=2).contains(&settings.water_effect_type) {
            settings.water_effect_type
        } else {
            0
        };
        self.checks[3].checked = settings.trilinear != 0;
        self.combos[9].enabled = caps.water >= 12;
        if !self.combos[9].enabled {
            self.combos[9].selected = 0;
        }
        self.combos[8].selected = match settings.actor_limit {
            6.. => 0,
            2..=5 => 6 - settings.actor_limit,
            _ => 5,
        };
        let keep_min = settings.keep_min_frame_rate != 0;
        self.checks[0].checked = keep_min;
        for index in [3, 4, 5, 6, 7] {
            self.combos[index].enabled = !keep_min;
        }
        for index in [1, 2] {
            self.checks[index].enabled = !keep_min;
        }
        self.combos[9].enabled = !keep_min && caps.water >= 12;
        if !self.combos[9].enabled {
            self.combos[9].selected = 0;
        }
        self.combos[12].enabled = !keep_min && caps.shaders.0 >= 20 && caps.shaders.1 >= 20;
        if !self.combos[12].enabled {
            self.combos[12].selected = 0;
        }
        self.combos[10].enabled = caps.antialiasing > 0 && !(3..=5).contains(&settings.post_process_type);
        self.combos[10].selected = if self.combos[10].enabled { settings.antialiasing } else { 0 };
    }

    pub fn paint_labels(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        for (y, label) in COMBO_Y.into_iter().zip(&self.labels) {
            list.draw_text(origin.0 + 98 - font.measure(label) as i32, origin.1 + y + 3, TEXT, label, font);
        }
    }

    pub fn keep_min_frame_changed(&mut self, water_capability: i32, shader_capabilities: (i32, i32)) {
        let keep_min = self.checks[0].checked;
        for index in [3, 4, 5, 6, 7] {
            self.combos[index].enabled = !keep_min;
        }
        for index in [1, 2] {
            self.checks[index].enabled = !keep_min;
        }
        if keep_min {
            self.combos[12].enabled = false;
            self.combos[12].selected = 0;
            self.combos[9].enabled = false;
            self.combos[9].selected = 0;
        } else {
            self.combos[9].enabled = water_capability >= 12;
            if !self.combos[9].enabled {
                self.combos[9].selected = 0;
            }
            self.combos[12].enabled = shader_capabilities.0 >= 20 && shader_capabilities.1 >= 20;
            if !self.combos[12].enabled {
                self.combos[9].selected = 0;
            }
        }
    }

    pub fn post_process_changed(&mut self, selection: i32) {
        let antialiasing = &mut self.combos[10];
        antialiasing.enabled = !(3..=5).contains(&selection);
        if !antialiasing.enabled {
            antialiasing.selected = 0;
        }
    }

    pub fn resolution_changed(&mut self, selection: i32, modes: &[(i32, i32, i32)], current_refresh_rate: i32, rates_for_mode: impl FnOnce((i32, i32, i32)) -> Vec<i32>) {
        let Some(mode) = usize::try_from(selection).ok().filter(|index| *index < 100).and_then(|index| modes.get(index)).copied() else { return };
        let mut rates = rates_for_mode(mode);
        rates.truncate(100);
        let combo = &mut self.combos[1];
        let preferred = usize::try_from(combo.selected).ok().and_then(|index| rates.get(index)).copied().unwrap_or(current_refresh_rate);
        combo.clear_entries();
        combo.entries.extend(rates.iter().map(|rate| format!("{rate}Hz")));
        combo.selected = rates.iter().position(|rate| *rate == preferred).map_or(rates.len() as i32 - 1, |index| index as i32);
    }
}

fn threshold_selection(value: f32, bounds: [f32; 4]) -> Option<i32> {
    if value.is_nan() {
        None
    } else {
        Some(bounds.iter().position(|bound| value < *bound).map_or(0, |index| 4 - index as i32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn lookup(id: u32) -> Option<String> { Some(id.to_string()) }

    #[test]
    fn video_apply_100c5da0_fog_reads_each_native_interval_and_boundary() {
        for (terrain, range, expected) in [
            (f32::NEG_INFINITY, 1, (0.5, 3.5)),
            (5.0_f32.next_down(), 1, (0.5, 3.5)),
            (5.0, 2, (1.0, 4.5)),
            (6.0_f32.next_down(), 2, (1.0, 4.5)),
            (6.0, 3, (1.5, 5.5)),
            (7.0_f32.next_down(), 3, (1.5, 5.5)),
            (7.0, 4, (2.0, 6.5)),
            (8.0_f32.next_down(), 4, (2.0, 6.5)),
            (8.0, 5, (2.5, 7.5)),
            (f32::INFINITY, 5, (2.5, 7.5)),
        ] {
            let mut calls = Vec::new();
            let actual = applied_fog_ranges(terrain, 0, |file, section, key| {
                calls.push((file.to_owned(), section.to_owned(), key.to_owned()));
                None
            });
            assert_eq!(actual, expected);
            assert_eq!(calls, vec![
                ("env.int".into(), "FOG".into(), format!("StartRange{range}")),
                ("env.int".into(), "FOG".into(), format!("EndRange{range}")),
            ]);
        }
    }

    #[test]
    fn video_apply_100c5da0_fog_preserves_independent_reads_and_nan_branch() {
        assert_eq!(applied_fog_ranges(f32::NAN, 0, |_, _, _| panic!("NaN must not read config")), (1.5, 5.5));
        for keep in [i32::MIN, -1, 1, i32::MAX] {
            for terrain in [f32::NAN, 4.0, 8.0, f32::INFINITY] {
                let mut keys = Vec::new();
                assert_eq!(applied_fog_ranges(terrain, keep, |_, _, key| {
                    keys.push(key.to_owned());
                    None
                }), (0.5, 3.5));
                assert_eq!(keys, ["StartRange1", "EndRange1"]);
            }
        }
        assert_eq!(applied_fog_ranges(7.0, 0, |_, _, key| (key == "StartRange4").then_some(-12.0)), (-12.0, 6.5));
        assert_eq!(applied_fog_ranges(7.0, 0, |_, _, key| (key == "EndRange4").then_some(99.0)), (2.0, 99.0));
        let values = applied_fog_ranges(7.0, 0, |_, _, _| Some(f32::NAN));
        assert!(values.0.is_nan() && values.1.is_nan());
    }

    fn settings() -> VideoSettings {
        VideoSettings {
            mode: (800, 600, 32), refresh_rate: 60, client_brightness: Some(1.0),
            pawn_clipping: 2.0, terrain_clipping: 6.0, render_deco: 1,
            post_process_type: 2, projector: 1, texture_detail: 1,
            reduce_pawn_lod: 0, skip_anim: 1, water_effect: 1, water_effect_type: 2,
            trilinear: 1, actor_limit: 4, keep_min_frame_rate: 0, antialiasing: 2,
        }
    }

    #[test]
    fn video_clear_100c54b0_clears_all_controls_without_changing_enable_or_geometry() {
        let mut page = VideoPage::new(lookup);
        for combo in &mut page.combos {
            combo.entries = vec!["first".into(), "second".into()];
            combo.selected = 1;
            combo.hovered = 1;
            combo.text_height = 11;
            combo.open_list();
            combo.enabled = false;
        }
        for check in &mut page.checks {
            check.checked = true;
            check.enabled = false;
        }
        let previous = page.clone();
        page.clear();
        assert_eq!(page.labels, previous.labels);
        for (actual, old) in page.combos.iter().zip(previous.combos) {
            let mut expected = old.clone();
            expected.entries.clear();
            expected.selected = -1;
            expected.hovered = -1;
            assert_eq!(*actual, expected);
            assert_eq!(actual.expanded_rect(), old.expanded_rect());
        }
        for (actual, old) in page.checks.iter().zip(previous.checks) {
            assert!(!actual.checked);
            assert_eq!(actual.enabled, old.enabled);
            assert_eq!(actual.rect, old.rect);
            assert_eq!(actual.label, old.label);
        }
    }

    #[test]
    fn video_clear_fill_read_100c98b0_reopens_without_duplicate_choices() {
        let mut page = VideoPage::new(lookup);
        for keep_min in [0, 1, 0] {
            let mut state = settings();
            state.keep_min_frame_rate = keep_min;
            page.clear();
            page.append_choices(&VideoChoices {
                modes: &[(800, 600, 32)], refresh_rates: &[60, 75],
                capability_10: true, capability_cc: 2, screenshot_quality: 3,
            }, lookup);
            page.read_settings(&mut state, &caps());
            assert_eq!(page.combos.each_ref().map(|combo| combo.entries.len()), [1, 2, 5, 3, 2, 2, 5, 5, 6, 3, 3, 4, 6]);
            assert_eq!(page.combos[0].selected, 0);
            assert_eq!(page.combos[1].selected, 0);
            assert_eq!(page.combos[11].selected, 3);
            assert_eq!(page.checks[0].checked, keep_min != 0);
            assert_eq!(page.combos[3].enabled, keep_min == 0);
            page.combos[0].selected = -1;
            page.combos[1].selected = 1;
        }
    }

    fn caps() -> VideoCapabilities<'static> {
        VideoCapabilities { modes: &[(800, 600, 32)], refresh_rates: &[60], water: 12, shaders: (20, 20), antialiasing: 2 }
    }

    #[test]
    fn video_100c3fd0_refreshes_both_lists_and_selects_last_exact_matches() {
        let mut page = VideoPage::new(lookup);
        for combo in &mut page.combos {
            combo.entries = vec!["old".into()];
            combo.selected = 0;
            combo.hovered = 0;
        }
        let state = settings();
        page.refresh_resolution(&state,
            || vec![(1024, 768, 32), state.mode, (800, 600, 16), state.mode],
            |mode| {
                assert_eq!(mode, state.mode);
                vec![60, 75, 60]
            });
        assert_eq!(page.combos[0].entries, ["1024*768 32bit", "800*600 32bit", "800*600 16bit", "800*600 32bit"]);
        assert_eq!(page.combos[1].entries, ["60Hz", "75Hz", "60Hz"]);
        assert_eq!((page.combos[0].selected, page.combos[1].selected), (3, 2));
        assert_eq!((page.combos[0].hovered, page.combos[1].hovered), (-1, -1));
        for combo in &page.combos[2..] {
            assert_eq!(combo.entries, ["old"]);
            assert_eq!((combo.selected, combo.hovered), (0, 0));
        }
    }

    #[test]
    fn video_100c3fd0_queries_global_mode_even_when_missing_and_limits_enumeration() {
        let mut page = VideoPage::new(lookup);
        let state = settings();
        let mut modes = vec![(1024, 768, 16); 100];
        modes.push(state.mode);
        let mut rates = vec![75; 100];
        rates.push(state.refresh_rate);
        page.refresh_resolution(&state, || modes, |mode| {
            assert_eq!(mode, state.mode);
            rates
        });
        assert_eq!((page.combos[0].entries.len(), page.combos[1].entries.len()), (100, 100));
        assert_eq!((page.combos[0].selected, page.combos[1].selected), (-1, -1));
        page.refresh_resolution(&state, Vec::new, |mode| {
            assert_eq!(mode, state.mode);
            Vec::new()
        });
        assert!(page.combos[0].entries.is_empty() && page.combos[1].entries.is_empty());
        assert_eq!((page.combos[0].selected, page.combos[1].selected), (-1, -1));
    }

    #[test]
    fn video_100c5580_rebuilds_hz_using_the_old_index_in_the_new_modes_rates() {
        let mut page = VideoPage::new(lookup);
        page.combos[0].selected = 1;
        page.combos[1].entries = vec!["60Hz".into(), "75Hz".into()];
        page.combos[1].selected = 1;
        page.combos[1].hovered = 0;
        page.resolution_changed(1, &[(800, 600, 16), (1920, 1080, 32)], 60, |mode| {
            assert_eq!(mode, (1920, 1080, 32));
            vec![120, 144, 165]
        });
        assert_eq!(page.combos[1].entries, ["120Hz", "144Hz", "165Hz"]);
        assert_eq!(page.combos[1].selected, 1);
        assert_eq!(page.combos[1].hovered, -1);
        assert_eq!(page.combos[0].selected, 1);
        page.resolution_changed(0, &[(800, 600, 32)], 60, |_| vec![144, 144, 165]);
        assert_eq!(page.combos[1].selected, 0);
    }

    #[test]
    fn video_100c5580_uses_global_rate_then_last_entry_for_invalid_old_index() {
        for (rates, global, expected) in [(vec![60, 75, 60], 60, 0), (vec![60, 75], 144, 1), (vec![], 60, -1)] {
            let mut page = VideoPage::new(lookup);
            page.combos[1].selected = 99;
            page.resolution_changed(0, &[(800, 600, 32)], global, |_| rates);
            assert_eq!(page.combos[1].selected, expected);
        }
        let mut page = VideoPage::new(lookup);
        page.combos[1].selected = 2;
        for invalid in [-1, 1, i32::MAX] {
            page.resolution_changed(invalid, &[(800, 600, 32)], 60, |_| panic!("invalid mode must not query rates"));
            assert_eq!(page.combos[1].selected, 2);
        }
    }

    #[test]
    fn video_100c6c90_uses_checkbox_and_preserves_all_unaffected_controls() {
        let mut page = VideoPage::new(lookup);
        for (index, combo) in page.combos.iter_mut().enumerate() {
            combo.selected = index as i32 + 10;
        }
        for check in &mut page.checks { check.checked = true; }
        page.keep_min_frame_changed(12, (20, 20));
        for (index, combo) in page.combos.iter().enumerate() {
            assert_eq!(combo.enabled, ![3, 4, 5, 6, 7, 9, 12].contains(&index));
            assert_eq!(combo.selected, if [9, 12].contains(&index) { 0 } else { index as i32 + 10 });
        }
        assert!(page.checks.iter().all(|c| c.checked));
        assert_eq!(page.checks.each_ref().map(|c| c.enabled), [true, false, false, true]);
        page.checks[0].checked = false;
        page.keep_min_frame_changed(12, (20, 20));
        assert!(page.combos.iter().all(|c| c.enabled));
        assert!(page.checks.iter().all(|c| c.enabled));
        assert_eq!((page.combos[9].selected, page.combos[12].selected), (0, 0));
    }

    #[test]
    fn video_100c6c90_low_shader_branch_clears_water_not_postprocess_or_aa() {
        for water in [i32::MIN, 11, 12, i32::MAX] {
            for shaders in [(19, 20), (20, 19), (20, 20), (21, 21)] {
                let mut page = VideoPage::new(lookup);
                page.combos[9].selected = 2;
                page.combos[12].selected = 4;
                page.combos[10].selected = 2;
                page.combos[10].enabled = false;
                page.keep_min_frame_changed(water, shaders);
                let supported = shaders.0 >= 20 && shaders.1 >= 20;
                assert_eq!(page.combos[9].enabled, water >= 12);
                assert_eq!(page.combos[9].selected, if water >= 12 && supported { 2 } else { 0 });
                assert_eq!(page.combos[12].enabled, supported);
                assert_eq!(page.combos[12].selected, 4);
                assert_eq!(page.combos[10].selected, 2);
                assert!(!page.combos[10].enabled);
            }
        }
    }

    #[test]
    fn video_100c3f80_only_blocks_aa_for_indices_three_through_five() {
        for selection in [i32::MIN, -1, 0, 1, 2, 3, 4, 5, 6, i32::MAX] {
            let mut page = VideoPage::new(lookup);
            page.combos[10].selected = 99;
            page.combos[10].enabled = false;
            page.combos[12].selected = 2;
            page.post_process_changed(selection);
            assert_eq!(page.combos[10].enabled, !(3..=5).contains(&selection));
            assert_eq!(page.combos[10].selected, if (3..=5).contains(&selection) { 0 } else { 99 });
            assert_eq!(page.combos[12].selected, 2);
        }
    }

    #[test]
    fn video_read_100c56d0_matches_last_mode_and_rate_and_preserves_missing_matches() {
        let mut page = VideoPage::new(lookup);
        let mut settings = settings();
        page.combos[11].selected = 3;
        let caps = VideoCapabilities { modes: &[(800, 600, 32), (800, 600, 16), (800, 600, 32)], refresh_rates: &[60, 75, 60], ..caps() };
        page.read_settings(&mut settings, &caps);
        assert_eq!(page.combos.each_ref().map(|c| c.selected), [2, 2, 1, 1, 0, 1, 2, 2, 2, 2, 2, 3, 2]);
        assert!(page.combos.iter().all(|c| c.enabled));
        assert_eq!(page.checks.each_ref().map(|c| c.checked), [false, true, true, true]);
        settings.mode = (1024, 768, 32);
        settings.refresh_rate = 144;
        page.read_settings(&mut settings, &caps);
        assert_eq!((page.combos[0].selected, page.combos[1].selected), (2, 2));
    }

    #[test]
    fn video_read_100c56d0_float_thresholds_include_exact_lower_bounds() {
        for bounds in [[0.6f32, 0.8, 1.0, 1.2], [1.5, 2.0, 2.5, 3.0], [5.0, 6.0, 7.0, 8.0]] {
            for (index, value) in bounds.into_iter().enumerate() {
                assert_eq!(threshold_selection(f32::from_bits(value.to_bits() - 1), bounds), Some(4 - index as i32));
                assert_eq!(threshold_selection(value, bounds), Some(3 - index as i32));
                assert_eq!(threshold_selection(f32::from_bits(value.to_bits() + 1), bounds), Some(3 - index as i32));
            }
            assert_eq!(threshold_selection(f32::NAN, bounds), None);
            assert_eq!(threshold_selection(f32::NEG_INFINITY, bounds), Some(4));
            assert_eq!(threshold_selection(f32::INFINITY, bounds), Some(0));
        }
        let mut page = VideoPage::new(lookup);
        let mut settings = settings();
        settings.client_brightness = None;
        settings.pawn_clipping = f32::NAN;
        settings.terrain_clipping = f32::NAN;
        for index in [2, 6, 7] { page.combos[index].selected = 9; }
        page.read_settings(&mut settings, &caps());
        assert!([2, 6, 7].into_iter().all(|index| page.combos[index].selected == 9));
    }

    #[test]
    fn video_read_100c56d0_keep_min_frame_preserves_choices_and_reenables_on_exit() {
        let mut page = VideoPage::new(lookup);
        let mut settings = settings();
        settings.keep_min_frame_rate = -1;
        page.read_settings(&mut settings, &caps());
        assert!([3, 4, 5, 6, 7, 9, 12].into_iter().all(|i| !page.combos[i].enabled));
        assert!([0, 1, 2, 8, 10, 11].into_iter().all(|i| page.combos[i].enabled));
        assert_eq!(page.checks.each_ref().map(|c| c.enabled), [true, false, false, true]);
        assert_eq!(page.checks.each_ref().map(|c| c.checked), [true, true, true, true]);
        assert_eq!(page.combos[9].selected, 0);
        assert_eq!(page.combos[12].selected, 0);
        assert_eq!(settings.post_process_type, 2);
        settings.keep_min_frame_rate = 0;
        page.read_settings(&mut settings, &caps());
        assert!(page.combos.iter().all(|c| c.enabled));
        assert!(page.checks.iter().all(|c| c.enabled));
        assert_eq!((page.combos[9].selected, page.combos[12].selected), (2, 2));
    }

    #[test]
    fn video_read_100c56d0_capabilities_and_postprocess_global_gate_antialiasing() {
        for water in [11, 12] {
            for shaders in [(19, 20), (20, 19), (20, 20)] {
                for aa in [-1, 0, 1, 2] {
                    for post in [-1, 0, 1, 2, 3, 4, 5, 6] {
                        for keep_min in [0, 1] {
                            let mut settings = settings();
                            settings.post_process_type = post;
                            settings.keep_min_frame_rate = keep_min;
                            settings.antialiasing = 99;
                            let mut page = VideoPage::new(lookup);
                            page.read_settings(&mut settings, &VideoCapabilities { water, shaders, antialiasing: aa, ..caps() });
                            let post = if (0..=5).contains(&post) { post } else { 0 };
                            assert_eq!(settings.post_process_type, post);
                            assert_eq!(page.combos[9].enabled, water >= 12 && keep_min == 0);
                            assert_eq!(page.combos[12].enabled, shaders == (20, 20) && keep_min == 0);
                            assert_eq!(page.combos[12].selected, if page.combos[12].enabled { post } else { 0 });
                            let allowed = aa > 0 && !(3..=5).contains(&post);
                            assert_eq!(page.combos[10].enabled, allowed);
                            assert_eq!(page.combos[10].selected, if allowed { 99 } else { 0 });
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn video_read_100c56d0_maps_integer_settings_without_changing_the_globals() {
        for value in [i32::MIN, -1, 0, 1, 2, 3, 4, 5, 6, i32::MAX] {
            let mut settings = settings();
            settings.actor_limit = value;
            settings.texture_detail = value;
            settings.reduce_pawn_lod = value;
            settings.skip_anim = value;
            settings.render_deco = value;
            settings.projector = value;
            settings.trilinear = value;
            let mut page = VideoPage::new(lookup);
            page.read_settings(&mut settings, &caps());
            assert_eq!(page.combos[8].selected, if value >= 6 { 0 } else if value <= 1 { 5 } else { 6 - value });
            assert_eq!(page.combos[3].selected, value.clamp(0, 2));
            assert_eq!(page.combos[4].selected, i32::from(value != 0));
            assert_eq!(page.combos[5].selected, i32::from(value != 0));
            assert!([1, 2, 3].into_iter().all(|i| page.checks[i].checked == (value != 0)));
            assert_eq!(settings.actor_limit, value);
            assert_eq!(settings.texture_detail, value);
        }
        for active in [0, -1, 1] {
            for kind in [-1, 0, 1, 2, 3] {
                let mut settings = settings();
                settings.water_effect = active;
                settings.water_effect_type = kind;
                let mut page = VideoPage::new(lookup);
                page.read_settings(&mut settings, &caps());
                assert_eq!(page.combos[9].selected, if active != 0 && (1..=2).contains(&kind) { kind } else { 0 });
            }
        }
    }

    #[test]
    fn video_fill_100c8ff0_preserves_every_choice_order_and_native_display_format() {
        let mut page = VideoPage::new(lookup);
        page.append_choices(&VideoChoices { modes: &[(800, 600, 16), (1920, 1080, 32)], refresh_rates: &[60, 75], capability_10: true, capability_cc: 2, screenshot_quality: 3 }, lookup);
        let expected: [&[&str]; 13] = [
            &["800*600 16bit", "1920*1080 32bit"], &["60Hz", "75Hz"],
            &["1233", "1234", "1235", "1236", "1237"], &["224", "225", "226"],
            &["224", "226"], &["224", "226"],
            &["1238", "1239", "1240", "1241", "1242"], &["1238", "1239", "1240", "1241", "1242"],
            &["27", "1233", "1234", "1235", "1236", "1237"], &["27", "353", "354"],
            &["869", "870", "871"], &["980", "981", "982", "983"],
            &["869", "867", "1246", "1230", "1231", "1232"],
        ];
        for (combo, expected) in page.combos.iter().zip(expected) {
            assert_eq!(combo.entries, expected);
        }
        assert_eq!(page.combos[11].selected, 3);
    }

    #[test]
    fn video_fill_100c8ff0_handles_all_capability_branches_without_default_modes() {
        for (capability, count, enabled) in [(0, 1, false), (1, 2, true), (2, 3, true), (-1, 0, true), (3, 0, true)] {
            let mut page = VideoPage::new(lookup);
            page.append_choices(&VideoChoices { modes: &[], refresh_rates: &[], capability_10: false, capability_cc: capability, screenshot_quality: -7 }, lookup);
            assert!(page.combos[0].entries.is_empty());
            assert!(page.combos[1].entries.is_empty());
            assert_eq!(page.combos[10].entries.len(), count);
            assert_eq!(page.combos[10].enabled, enabled);
            assert_eq!(page.combos[12].entries, ["869", "867", "1246"]);
            assert_eq!(page.combos[11].selected, -7);
            page.combos[10].enabled = false;
            let previous = page.combos[10].entries.clone();
            page.append_choices(&VideoChoices { modes: &[], refresh_rates: &[], capability_10: false, capability_cc: 99, screenshot_quality: 99 }, lookup);
            assert!(!page.combos[10].enabled);
            assert_eq!(page.combos[10].entries, previous);
            assert_eq!(page.combos[11].entries.len(), 8);
            assert_eq!(page.combos[11].selected, 99);
        }
    }

    #[test]
    fn video_onpaint_100c4a70_places_every_label_and_measures_combo_font() {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        let font = BitmapFont::new("font".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap());
        let mut page = VideoPage::new(|_| Some("A".into()));
        page.set_font(&font);
        assert!(page.combos.iter().all(|combo| combo.text_height == 13));
        let mut list = DrawList::default();
        page.paint_labels(&mut list, (10, 48), &font);
        assert_eq!(list.quads.len(), 13);
        for (quad, y) in list.quads.iter().zip([14, 33, 52, 71, 90, 109, 142, 161, 180, 232, 251, 320, 339]) {
            assert_eq!((quad.x, quad.y), (102.0, (48 + y) as f32));
            assert_eq!(quad.color, TEXT.rgba());
        }
    }
}
