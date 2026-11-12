pub fn wide_digit(unit: u16) -> Option<i32> {
    [0x0030_u16, 0x0660, 0x06f0, 0x0966, 0x09e6, 0x0a66, 0x0ae6, 0x0b66, 0x0c66,
        0x0ce6, 0x0d66, 0x0e50, 0x0ed0, 0x0f20, 0x1040, 0x17e0, 0x1810, 0xff10]
        .into_iter()
        .find_map(|base| unit.checked_sub(base).filter(|digit| *digit < 10).map(i32::from))
}

pub fn wide_atoi(value: &[u16], mut is_space: impl FnMut(u16) -> bool) -> i32 {
    let mut units = value.iter().copied().chain(std::iter::once(0));
    let mut current = units.next().unwrap_or(0);
    while current != 0 && is_space(current) {
        current = units.next().unwrap_or(0);
    }
    let negative = current == 0x2d;
    if negative || current == 0x2b {
        current = units.next().unwrap_or(0);
    }
    let mut number = 0_i32;
    while let Some(digit) = wide_digit(current) {
        number = number.wrapping_mul(10).wrapping_add(digit);
        current = units.next().unwrap_or(0);
    }
    if negative { number.wrapping_neg() } else { number }
}

pub fn config_bool(
    value: Option<&[u16]>,
    mut equals_true: impl FnMut(&[u16]) -> bool,
    is_space: impl FnMut(u16) -> bool,
) -> Option<bool> {
    let value = value?;
    let length = value.iter().take(79).position(|unit| *unit == 0).unwrap_or(value.len().min(79));
    let value = &value[..length];
    Some(equals_true(value) || wide_atoi(value, is_space) == 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn space(unit: u16) -> bool {
        matches!(unit, 9..=13 | 32 | 160)
    }

    fn boolean(value: Option<&str>) -> Option<bool> {
        let value = value.map(|value| value.encode_utf16().collect::<Vec<_>>());
        config_bool(value.as_deref(), |value| {
            value.iter().copied().map(|unit| if (65..=90).contains(&unit) { unit + 32 } else { unit })
                .eq("true".encode_utf16())
        }, space)
    }

    #[test]
    fn get_bool_1090f7d0_preserves_absence_and_converts_present_values() {
        assert_eq!(boolean(None), None);
        for value in ["True", "TRUE", "tRuE", "1", "+01", "1tail", "1.5", "4294967297", "-4294967295", "\u{a0}1", "١"] {
            assert_eq!(boolean(Some(value)), Some(true), "{value:?}");
        }
        for value in ["", "False", "2", "-1", "TrueX", " True", "True ", "garbage", "+", "--1", "௧"] {
            assert_eq!(boolean(Some(value)), Some(false), "{value:?}");
        }
        assert_eq!(boolean(Some("True\0ignored")), Some(true));
        assert_eq!(boolean(Some(&format!("{}1", "0".repeat(78)))), Some(true));
        assert_eq!(boolean(Some(&format!("{}1", "0".repeat(79)))), Some(false));
    }

    #[test]
    fn wtol_7c001adc_wraps_and_stops_at_the_first_invalid_unit() {
        for (text, expected) in [("2147483648", i32::MIN), ("-2147483648", i32::MIN),
            ("4294967295", -1), ("4294967296", 0), ("  -１２3tail", -123), ("12\0３", 12)] {
            assert_eq!(wide_atoi(&text.encode_utf16().collect::<Vec<_>>(), space), expected);
        }
        assert_eq!(wide_atoi(&[0xd800, 49], space), 0);
        assert_eq!(wide_atoi(&[0x2000, 49], |unit| unit == 0x2000), 1);
        assert_eq!(wide_atoi(&[0x2000, 49], |_| false), 0);
    }

    #[test]
    fn get_bool_does_not_call_conversion_on_missing_or_true_values() {
        assert_eq!(config_bool(None, |_| panic!("missing comparison"), |_| panic!("missing conversion")), None);
        assert_eq!(config_bool(Some(&[84, 114, 117, 101]), |_| true, |_| panic!("true conversion")), Some(true));
    }
}
