use crate::ui_option_config::ConfigEdit;
use killer_ui::ini::IniFile;
use std::collections::BTreeMap;

pub const NO_PING: i32 = 9999;
pub const EXPIRY: i32 = 12;
const DATE_SECTION: &str = "Date";
const TIME_KEY: &str = "Time";

pub fn is_leap(year: i32) -> bool {
    if year % 4 == 0 { year % 100 != 0 || year % 400 == 0 } else { year % 400 == 0 }
}

fn days_before_month(year: i32, month: i32) -> i32 {
    let mut total = 0;
    let february = if is_leap(year) { 29 } else { 28 };
    for m in 1..month {
        total += match m {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            2 => february,
            _ => 30,
        };
    }
    total
}

fn days_before_year(year: i32) -> i32 {
    (1..=year).map(|y| if is_leap(y) { 366 } else { 365 }).sum()
}

fn day_hours(day: i32, hour: i32) -> i32 {
    if day <= 1 { hour } else { day * 24 - 24 + hour }
}

pub fn time_key(year: i32, month: i32, day: i32, hour: i32) -> i32 {
    days_before_month(year, month) + days_before_year(year - 1) + day_hours(day, hour)
}

pub fn civil_from_unix(seconds: i64) -> (i32, i32, i32, i32) {
    let days = seconds.div_euclid(86_400);
    let hour = (seconds.rem_euclid(86_400) / 3600) as i32;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as i32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as i32;
    let year = (yoe + era * 400) as i32 + i32::from(month <= 2);
    (year, month, day, hour)
}

pub fn now_key() -> i32 {
    let seconds = killer_platform::time::SystemTime::now()
        .duration_since(killer_platform::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() as i64);
    let (year, month, day, hour) = civil_from_unix(seconds);
    time_key(year, month, day, hour)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PingCache {
    times: BTreeMap<u8, i32>,
    pub expired: bool,
    pub dirty: bool,
}

impl PingCache {
    pub fn from_ini(ini: &IniFile, now: i32) -> PingCache {
        let expired = ini.int(DATE_SECTION, TIME_KEY).is_none_or(|stored| stored.wrapping_add(EXPIRY) < now);
        let mut times = BTreeMap::new();
        for section in ini.0.keys() {
            if let Ok(id) = section.parse::<u8>()
                && let Some(time) = ini.int(section, TIME_KEY)
            {
                times.insert(id, time);
            }
        }
        PingCache { times, expired, dirty: false }
    }

    pub fn cached(&self, id: u8) -> Option<i32> {
        if self.expired {
            return None;
        }
        self.times.get(&id).copied().filter(|time| *time != NO_PING)
    }

    pub fn record(&mut self, id: u8, ms: i32) {
        self.times.insert(id, ms);
        self.dirty = true;
    }

    pub fn finish(&mut self, now: i32) -> Vec<ConfigEdit> {
        let mut edits = Vec::new();
        if self.dirty {
            for (id, time) in &self.times {
                edits.push(ConfigEdit::new(&id.to_string(), TIME_KEY, time.to_string()));
            }
            self.dirty = false;
        }
        if self.expired {
            edits.push(ConfigEdit::new(DATE_SECTION, TIME_KEY, now.to_string()));
            self.expired = false;
        }
        edits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::ini::parse_ini;

    #[test]
    fn the_time_key_follows_101003c0_months_years_and_day_hours() {
        assert_eq!(time_key(2000, 1, 1, 0), days_before_year(1999));
        assert_eq!(time_key(2024, 3, 1, 5), 31 + 29 + days_before_year(2023) + 5);
        assert_eq!(time_key(2023, 3, 1, 5), 31 + 28 + days_before_year(2022) + 5);
        assert_eq!(time_key(2024, 1, 3, 7), days_before_year(2023) + 2 * 24 + 7);
        assert!(is_leap(2000) && !is_leap(1900) && is_leap(2024) && !is_leap(2023));
    }

    #[test]
    fn unix_seconds_become_the_calendar_fields() {
        assert_eq!(civil_from_unix(0), (1970, 1, 1, 0));
        assert_eq!(civil_from_unix(951_782_400 + 3 * 3600), (2000, 2, 29, 3));
        assert_eq!(civil_from_unix(1_700_000_000), (2023, 11, 14, 22));
    }

    #[test]
    fn the_cache_expires_when_the_date_is_missing_or_older_than_12_units() {
        let now = time_key(2025, 5, 10, 12);
        assert!(PingCache::from_ini(&parse_ini(""), now).expired);
        let fresh = format!("[Date]\nTime={}\n[1]\nTime=40\n[2]\nTime=9999\n", now - 12);
        let cache = PingCache::from_ini(&parse_ini(&fresh), now);
        assert!(!cache.expired);
        assert_eq!((cache.cached(1), cache.cached(2), cache.cached(3)), (Some(40), None, None));
        let stale = format!("[Date]\nTime={}\n[1]\nTime=40\n", now - 13);
        let cache = PingCache::from_ini(&parse_ini(&stale), now);
        assert!(cache.expired);
        assert_eq!(cache.cached(1), None);
    }

    #[test]
    fn recorded_pings_and_the_date_stamp_are_written_on_confirm_like_101004c0() {
        let now = time_key(2025, 5, 10, 12);
        let mut cache = PingCache::from_ini(&parse_ini(""), now);
        cache.record(2, 80);
        cache.record(1, 9999);
        let edits = cache.finish(now);
        let written = crate::ui_option_config::edit_ini("", &edits);
        let ini = parse_ini(&written);
        assert_eq!((ini.int("1", "Time"), ini.int("2", "Time")), (Some(9999), Some(80)));
        assert_eq!(ini.int("Date", "Time"), Some(now));
        assert!(!cache.expired && !cache.dirty);
        assert!(cache.finish(now).is_empty());
    }
}
