use std::collections::HashMap;

use killer_assets::collision::world::MusicZone;

const REFRESH: u32 = 1;
const SERVER_MUSIC_PENDING: u32 = 2;
const VOICE_BUSY: u32 = 1;
const IDLE_RESTART_DEFAULT: f32 = 60.0;
const IDLE_INTERVAL_DEFAULT: f32 = 30.0;
const ZONE_STOP_FADE: f32 = 2.5;
const SERVER_MUSIC_STOP_FADE: f32 = 5.5;
const VOICE_STOP_FADE: f32 = 1.0;
const VOICE_DUCK_VOLUME: f32 = 0.3;
const VOICE_RESTORE_FADE: f32 = 2.0;
const SERVER_MUSIC_DUCKED_VOLUME: f32 = 0.5;
const CHRISTMAS_YEAR: i32 = 2005;
const CHRISTMAS_MONTH: i32 = 12;
const CHRISTMAS_IDS: [i32; 15] = [
    0x2a0a9, 0x2a1d5, 0x2c49a, 0x2ee65, 0x31449, 0x315d9, 0x3163d, 0x33bbd, 0x362ce, 0x363f9, 0x38721, 0x38a41, 0x38bd9, 0x3b089, 0x3afc3,
];
const CHRISTMAS_EXTRA_ID: i32 = 0x33a93;

pub trait MusicOut {
    fn music_playing(&self) -> bool;
    fn ogg_voice_playing(&self) -> bool;
    fn wav_voice_playing(&self) -> bool;
    fn voice_finished(&self) -> bool;
    fn play_music(&mut self, name: &str, volume: f32) -> i32;
    fn stop_music(&mut self, handle: i32, fade: f32);
    fn set_music_volume(&mut self, handle: i32, volume: f32, fade: f32);
    fn play_voice(&mut self, name: &str, ogg: bool) -> i32;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MusicTimes {
    pub restart_delay: f32,
    pub idle_interval: f32,
}

impl Default for MusicTimes {
    fn default() -> Self {
        Self { restart_delay: IDLE_RESTART_DEFAULT, idle_interval: IDLE_INTERVAL_DEFAULT }
    }
}

impl MusicTimes {
    pub fn read(read: impl Fn(&str, &str) -> Option<String>) -> Self {
        let float = |key: &str, default: f32| {
            read("MusicVolume", key).and_then(|text| text.trim().trim_end_matches(['f', 'F']).parse::<f32>().ok()).unwrap_or(default)
        };
        Self { restart_delay: float("MaxTime", IDLE_RESTART_DEFAULT), idle_interval: float("DefaultMusicTime", IDLE_INTERVAL_DEFAULT) }
    }
}

pub struct MusicNames<'a> {
    pub titles: &'a HashMap<u32, Vec<String>>,
    pub today: (i32, i32, i32),
}

impl MusicNames<'_> {
    pub fn get(&self, id: i32, mut rand: impl FnMut() -> i32) -> Option<String> {
        if id < 0 || self.titles.is_empty() {
            return None;
        }
        let (year, month, day) = self.today;
        let id = if year == CHRISTMAS_YEAR && month == CHRISTMAS_MONTH && day > 20 && day < 27 && (CHRISTMAS_IDS.contains(&id) || id == CHRISTMAS_EXTRA_ID) {
            1
        } else {
            id
        };
        let names = self.titles.get(&u32::try_from(id).ok()?)?;
        let count = i32::try_from(names.len()).ok().filter(|count| *count > 0)?;
        names.get(usize::try_from(rand().rem_euclid(count)).ok()?).cloned()
    }
}

pub fn civil_date(unix_seconds: u64) -> (i32, i32, i32) {
    let days = i64::try_from(unix_seconds / 86_400).unwrap_or(0) + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 { month_index + 3 } else { month_index - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year as i32, month as i32, day as i32)
}

fn truncate(value: f64) -> i32 {
    killer_assets::pawn_rotation::ftol(value)
}

pub fn idle_music_id(location: [f32; 3]) -> i32 {
    let tile_x = truncate((f64::from(location[0]) + 360_448.0) * 3.051_757_812_5e-5 + 9.0);
    let tile_y = truncate((f64::from(location[1]) + 294_912.0) * 3.051_757_812_5e-5 + 9.0);
    truncate(f64::from(tile_x) * 100.0) + tile_y
}

pub struct Pawn {
    pub location: [f32; 3],
    pub zone: Option<MusicZone>,
    pub on_vehicle: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ZoneMusic {
    last_zone: Option<MusicZone>,
    flags: u32,
    restart_timer: f32,
    idle_timer: f32,
    music_handle: i32,
    voice_handle: i32,
    vehicle_status: bool,
    voice_request: u8,
    server_music_delay: f32,
    voice_delay: f32,
    voice_busy: u32,
    server_music_pending: u32,
    server_music: String,
    voice_name: String,
}

impl ZoneMusic {
    pub fn reset(&mut self) {
        self.flags |= REFRESH;
        self.last_zone = None;
    }

    pub fn set_vehicle_status(&mut self, on: bool) {
        self.vehicle_status = on;
    }

    pub fn request_server_music(&mut self, name: &str, delay: f32) {
        self.server_music_pending |= SERVER_MUSIC_PENDING;
        self.server_music = name.to_owned();
        self.server_music_delay = delay;
    }

    #[cfg(test)]
    pub fn voice_handle(&self) -> i32 {
        self.voice_handle
    }

    pub fn request_server_voice(&mut self, name: &str, delay: f32, ogg: bool) {
        self.voice_request = u8::from(!ogg) + 1;
        self.voice_name = name.to_owned();
        self.voice_delay = delay;
    }

    pub fn tick(
        &mut self,
        dt: f32,
        times: MusicTimes,
        pawn: Option<&Pawn>,
        audio: Option<&mut dyn MusicOut>,
        mut name_of: impl FnMut(i32) -> Option<String>,
    ) {
        let on_vehicle = pawn.is_some_and(|pawn| pawn.on_vehicle) && self.vehicle_status;
        let Some(audio) = audio else {
            return;
        };
        if audio.voice_finished() && audio.music_playing() && self.music_handle > 0 {
            audio.set_music_volume(self.music_handle, 1.0, 2.0);
        }
        self.tick_server_voice(dt, audio);
        self.tick_server_music(dt, audio);
        if let Some(pawn) = pawn {
            if self.voice_busy & VOICE_BUSY == 0 && on_vehicle {
                if !audio.music_playing()
                    && let Some(name) = name_of(0)
                {
                    self.music_handle = audio.play_music(&name, 1.0);
                }
                self.flags |= REFRESH;
            }
            if !on_vehicle && self.voice_busy & VOICE_BUSY == 0 {
                self.tick_zone(dt, times, pawn, audio, &mut name_of);
            }
        }
        if self.voice_busy & VOICE_BUSY != 0 && !audio.music_playing() {
            self.voice_busy &= !VOICE_BUSY;
        }
    }

    fn tick_server_voice(&mut self, dt: f32, audio: &mut dyn MusicOut) {
        if self.voice_request == 0 {
            return;
        }
        let ogg = audio.ogg_voice_playing();
        let wav = audio.wav_voice_playing();
        let playing = audio.music_playing();
        let stop = if self.voice_request == 1 {
            (wav || ogg) && self.voice_handle > 0
        } else {
            wav && !ogg && self.voice_handle > 0
        };
        if stop {
            audio.stop_music(self.voice_handle, VOICE_STOP_FADE);
            self.voice_handle = -1;
        } else if self.voice_request == 2 && ogg {
            self.voice_delay = 0.0;
            self.voice_request = 0;
            self.voice_name.clear();
        }
        if self.voice_delay > 0.0 {
            self.voice_delay -= dt;
            return;
        }
        if self.voice_name.is_empty() {
            return;
        }
        self.voice_delay = 0.0;
        if self.voice_request == 1 || self.voice_request == 2 {
            self.voice_handle = audio.play_voice(&self.voice_name, self.voice_request == 1);
        }
        self.voice_request = 0;
        if self.voice_handle > 0 {
            if playing && self.music_handle > 0 {
                audio.set_music_volume(self.music_handle, VOICE_DUCK_VOLUME, VOICE_RESTORE_FADE);
            }
            self.voice_busy |= VOICE_BUSY;
        }
    }

    fn tick_server_music(&mut self, dt: f32, audio: &mut dyn MusicOut) {
        if self.server_music_pending & SERVER_MUSIC_PENDING == 0 {
            return;
        }
        let voice = audio.ogg_voice_playing() || audio.wav_voice_playing();
        if audio.music_playing() && self.music_handle > 0 {
            audio.stop_music(self.music_handle, SERVER_MUSIC_STOP_FADE);
            self.music_handle = -1;
        }
        if self.server_music_delay > 0.0 {
            self.server_music_delay -= dt;
            return;
        }
        self.server_music_delay = 0.0;
        let volume = if voice && self.voice_handle > 0 { SERVER_MUSIC_DUCKED_VOLUME } else { 1.0 };
        self.music_handle = audio.play_music(&self.server_music, volume);
        self.server_music_pending &= !SERVER_MUSIC_PENDING;
        if self.music_handle > 0 {
            self.voice_busy |= VOICE_BUSY;
        }
    }

    fn tick_zone(&mut self, dt: f32, times: MusicTimes, pawn: &Pawn, audio: &mut dyn MusicOut, name_of: &mut impl FnMut(i32) -> Option<String>) {
        let playing = audio.music_playing();
        let zone = pawn.zone.as_ref();
        if !playing {
            let restart = if zone.is_none_or(|zone| !zone.looped) || self.flags & REFRESH != 0 {
                if self.flags & REFRESH != 0 {
                    false
                } else {
                    self.restart_timer += dt;
                    self.restart_timer >= times.restart_delay
                }
            } else {
                true
            };
            if restart {
                self.flags |= REFRESH;
                self.restart_timer = 0.0;
            }
        }
        if let Some(zone) = zone {
            let skip = if self.last_zone.as_ref() == Some(zone) || !zone.force_play {
                self.flags & REFRESH == 0 || playing
            } else {
                if playing && self.music_handle > 0 {
                    audio.stop_music(self.music_handle, ZONE_STOP_FADE);
                    self.music_handle = -1;
                }
                false
            };
            if !skip && let Some(name) = name_of(zone.music_id) {
                self.music_handle = audio.play_music(&name, 1.0);
                self.flags &= !REFRESH;
            }
        }
        self.last_zone = pawn.zone.clone();
        if zone.is_some() {
            self.idle_timer = 0.0;
        } else if !playing {
            self.idle_timer += dt;
            if self.idle_timer >= times.idle_interval {
                self.idle_timer = 0.0;
                if let Some(name) = name_of(idle_music_id(pawn.location)) {
                    self.music_handle = audio.play_music(&name, 1.0);
                    self.flags &= !REFRESH;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Recorder {
        playing: bool,
        ogg: bool,
        wav: bool,
        finished: bool,
        next_handle: i32,
        log: Vec<String>,
    }

    impl MusicOut for Recorder {
        fn music_playing(&self) -> bool {
            self.playing
        }
        fn ogg_voice_playing(&self) -> bool {
            self.ogg
        }
        fn wav_voice_playing(&self) -> bool {
            self.wav
        }
        fn voice_finished(&self) -> bool {
            self.finished
        }
        fn play_music(&mut self, name: &str, volume: f32) -> i32 {
            self.next_handle += 1;
            self.log.push(format!("play {name} {volume}"));
            self.playing = true;
            self.next_handle
        }
        fn stop_music(&mut self, handle: i32, fade: f32) {
            self.log.push(format!("stop {handle} {fade}"));
            self.playing = false;
        }
        fn set_music_volume(&mut self, handle: i32, volume: f32, fade: f32) {
            self.log.push(format!("volume {handle} {volume} {fade}"));
        }
        fn play_voice(&mut self, name: &str, ogg: bool) -> i32 {
            self.next_handle += 1;
            self.log.push(format!("voice {name} {ogg}"));
            self.next_handle
        }
    }

    fn zone(index: usize, id: i32, force: bool, looped: bool) -> MusicZone {
        MusicZone { tile: "t".to_owned(), index, music_id: id, force_play: force, looped }
    }

    fn pawn(zone: Option<MusicZone>) -> Pawn {
        Pawn { location: [0.0; 3], zone, on_vehicle: false }
    }

    fn names(id: i32) -> Option<String> {
        Some(format!("m{id}"))
    }

    #[test]
    fn a_fresh_refresh_flag_plays_the_zone_music_once_and_clears_the_flag() {
        let mut music = ZoneMusic::default();
        music.reset();
        let mut audio = Recorder::default();
        let pawn = pawn(Some(zone(1, 7, false, false)));
        music.tick(0.1, MusicTimes::default(), Some(&pawn), Some(&mut audio), names);
        music.tick(0.1, MusicTimes::default(), Some(&pawn), Some(&mut audio), names);
        assert_eq!(audio.log, vec!["play m7 1"]);
    }

    #[test]
    fn a_forcing_zone_stops_the_current_music_with_the_zone_fade_and_starts_the_new_one() {
        let mut music = ZoneMusic::default();
        music.reset();
        let mut audio = Recorder::default();
        music.tick(0.1, MusicTimes::default(), Some(&pawn(Some(zone(1, 7, false, false)))), Some(&mut audio), names);
        music.tick(0.1, MusicTimes::default(), Some(&pawn(Some(zone(2, 8, true, false)))), Some(&mut audio), names);
        assert_eq!(audio.log, vec!["play m7 1", "stop 1 2.5", "play m8 1"]);
    }

    #[test]
    fn a_non_forcing_zone_waits_for_the_music_to_end_and_the_restart_delay() {
        let mut music = ZoneMusic::default();
        music.reset();
        let mut audio = Recorder::default();
        let times = MusicTimes { restart_delay: 1.0, idle_interval: 30.0 };
        music.tick(0.1, times, Some(&pawn(Some(zone(1, 7, false, false)))), Some(&mut audio), names);
        music.tick(0.1, times, Some(&pawn(Some(zone(2, 8, false, false)))), Some(&mut audio), names);
        assert_eq!(audio.log, vec!["play m7 1"]);
        audio.playing = false;
        music.tick(0.5, times, Some(&pawn(Some(zone(2, 8, false, false)))), Some(&mut audio), names);
        assert_eq!(audio.log.len(), 1);
        music.tick(0.6, times, Some(&pawn(Some(zone(2, 8, false, false)))), Some(&mut audio), names);
        assert_eq!(audio.log, vec!["play m7 1", "play m8 1"]);
    }

    #[test]
    fn a_looping_zone_restarts_as_soon_as_the_music_ends() {
        let mut music = ZoneMusic::default();
        music.reset();
        let mut audio = Recorder::default();
        let looped = pawn(Some(zone(1, 7, false, true)));
        music.tick(0.1, MusicTimes::default(), Some(&looped), Some(&mut audio), names);
        audio.playing = false;
        music.tick(0.1, MusicTimes::default(), Some(&looped), Some(&mut audio), names);
        assert_eq!(audio.log, vec!["play m7 1", "play m7 1"]);
    }

    #[test]
    fn outside_every_zone_the_idle_music_of_the_map_tile_plays_after_the_interval() {
        let mut music = ZoneMusic::default();
        let mut audio = Recorder::default();
        let times = MusicTimes { restart_delay: 30.0, idle_interval: 1.0 };
        let mut at = pawn(None);
        at.location = [0.0, 0.0, 0.0];
        music.tick(0.6, times, Some(&at), Some(&mut audio), names);
        assert!(audio.log.is_empty());
        music.tick(0.6, times, Some(&at), Some(&mut audio), names);
        assert_eq!(audio.log, vec![format!("play m{} 1", idle_music_id([0.0; 3]))]);
    }

    #[test]
    fn the_idle_id_is_the_map_tile_x_times_a_hundred_plus_the_tile_y() {
        assert_eq!(idle_music_id([0.0, 0.0, 0.0]), 20 * 100 + 18);
        assert_eq!(idle_music_id([32_768.0, -32_768.0, 0.0]), 21 * 100 + 17);
    }

    #[test]
    fn the_vehicle_plays_music_zero_and_skips_the_zones() {
        let mut music = ZoneMusic::default();
        music.set_vehicle_status(true);
        let mut audio = Recorder::default();
        let mut boat = pawn(Some(zone(1, 7, false, false)));
        boat.on_vehicle = true;
        music.tick(0.1, MusicTimes::default(), Some(&boat), Some(&mut audio), names);
        assert_eq!(audio.log, vec!["play m0 1"]);
    }

    #[test]
    fn the_server_music_request_stops_the_current_music_and_plays_after_its_delay() {
        let mut music = ZoneMusic::default();
        let mut audio = Recorder::default();
        music.reset();
        music.tick(0.1, MusicTimes::default(), Some(&pawn(Some(zone(1, 7, false, false)))), Some(&mut audio), names);
        music.request_server_music("boss", 1.0);
        music.tick(0.5, MusicTimes::default(), Some(&pawn(None)), Some(&mut audio), names);
        music.tick(0.6, MusicTimes::default(), Some(&pawn(None)), Some(&mut audio), names);
        assert_eq!(audio.log, vec!["play m7 1", "stop 1 5.5"]);
        music.tick(0.1, MusicTimes::default(), Some(&pawn(None)), Some(&mut audio), names);
        assert_eq!(audio.log, vec!["play m7 1", "stop 1 5.5", "play boss 1"]);
    }

    #[test]
    fn the_server_voice_ducks_the_music_to_thirty_percent() {
        let mut music = ZoneMusic::default();
        music.reset();
        let mut audio = Recorder::default();
        music.tick(0.1, MusicTimes::default(), Some(&pawn(Some(zone(1, 7, false, false)))), Some(&mut audio), names);
        music.request_server_voice("hello", 0.0, true);
        music.tick(0.1, MusicTimes::default(), Some(&pawn(None)), Some(&mut audio), names);
        assert_eq!(audio.log, vec!["play m7 1", "voice hello true", "volume 1 0.3 2"]);
    }

    #[test]
    fn music_names_pick_a_title_by_the_rand_modulo_and_swap_the_christmas_ids() {
        let mut titles = HashMap::new();
        titles.insert(5, vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]);
        titles.insert(1, vec!["xmas".to_owned()]);
        let plain = MusicNames { titles: &titles, today: (2026, 1, 1) };
        assert_eq!(plain.get(5, || 4).as_deref(), Some("b"));
        assert_eq!(plain.get(9, || 0), None);
        assert_eq!(plain.get(-1, || 0), None);
        let december = MusicNames { titles: &titles, today: (2005, 12, 24) };
        assert_eq!(december.get(0x2a0a9, || 0).as_deref(), Some("xmas"));
        assert_eq!(december.get(5, || 1).as_deref(), Some("b"));
    }

    #[test]
    fn civil_date_matches_known_days() {
        assert_eq!(civil_date(0), (1970, 1, 1));
        assert_eq!(civil_date(1_135_382_400), (2005, 12, 24));
        assert_eq!(civil_date(1_782_432_000), (2026, 6, 26));
    }

    #[test]
    fn times_read_the_music_volume_section_and_fall_back_to_the_engine_defaults() {
        let read = |_: &str, key: &str| (key == "MaxTime").then(|| "45.f".to_owned());
        assert_eq!(MusicTimes::read(read), MusicTimes { restart_delay: 45.0, idle_interval: 30.0 });
        assert_eq!(MusicTimes::read(|_, _| None), MusicTimes { restart_delay: 60.0, idle_interval: 30.0 });
    }
}
