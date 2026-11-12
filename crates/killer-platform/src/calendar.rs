#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalDate {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    pub minute: i32,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn from_seconds(seconds: i32) -> Option<LocalDate> {
    let zone = jiff::tz::TimeZone::try_system().ok()?;
    let date = jiff::Timestamp::from_second(i64::from(seconds)).ok()?.to_zoned(zone);
    Some(LocalDate { year: i32::from(date.year()), month: i32::from(date.month()), day: i32::from(date.day()), hour: i32::from(date.hour()), minute: i32::from(date.minute()) })
}

#[cfg(target_arch = "wasm32")]
pub fn from_seconds(seconds: i32) -> Option<LocalDate> {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(f64::from(seconds) * 1000.0));
    Some(LocalDate { year: date.get_full_year() as i32, month: date.get_month() as i32 + 1, day: date.get_date() as i32, hour: date.get_hours() as i32, minute: date.get_minutes() as i32 })
}
