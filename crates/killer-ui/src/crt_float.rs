const INDEFINITE: u64 = 0xFFF8_0000_0000_0000;
const DECIMALS: usize = 2;
const SIGNIFICANT: usize = 17;

pub const INDEFINITE_F32: u32 = 0xFFC0_0000;

pub fn div_f32(numerator: f32, denominator: f32) -> f32 {
    let quotient = numerator / denominator;
    if quotient.is_nan() && !numerator.is_nan() && !denominator.is_nan() {
        return f32::from_bits(INDEFINITE_F32);
    }
    quotient
}

fn non_finite(value: f64) -> String {
    let bits = value.to_bits();
    let sign = if bits >> 63 != 0 { "-" } else { "" };
    let class = if value.is_infinite() || bits == INDEFINITE {
        'J'
    } else if bits & (1 << 51) != 0 {
        'R'
    } else {
        'T'
    };
    format!("{sign}1.#{class}")
}

pub fn format_f2(value: f64) -> String {
    if !value.is_finite() {
        return non_finite(value);
    }
    let sign = if value < 0.0 { "-" } else { "" };
    let scientific = format!("{:.*e}", SIGNIFICANT - 1, value.abs());
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let exponent: i32 = exponent.parse().unwrap_or(0);
    let mut digits: Vec<u8> = mantissa.bytes().filter(u8::is_ascii_digit).map(|d| d - b'0').collect();
    if value == 0.0 {
        digits.fill(0);
    }
    let kept = exponent + 1 + DECIMALS as i32;
    let mut number: Vec<u8> = if kept < 0 {
        Vec::new()
    } else {
        (0..kept as usize).map(|i| digits.get(i).copied().unwrap_or(0)).collect()
    };
    let next = if kept < 0 { 0 } else { digits.get(kept as usize).copied().unwrap_or(0) };
    if next >= 5 {
        let mut carry = true;
        for digit in number.iter_mut().rev() {
            if *digit == 9 {
                *digit = 0;
            } else {
                *digit += 1;
                carry = false;
                break;
            }
        }
        if carry {
            number.insert(0, 1);
        }
    }
    while number.len() < DECIMALS + 1 {
        number.insert(0, 0);
    }
    let split = number.len() - DECIMALS;
    let text = |part: &[u8]| part.iter().map(|d| char::from(b'0' + d)).collect::<String>();
    format!("{sign}{}.{}", text(&number[..split]), text(&number[split..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORACLE: &[(u64, &str)] = &[
        (0x7ff0000000000000, "1.#J"),
        (0xfff0000000000000, "-1.#J"),
        (0xfff8000000000000, "-1.#J"),
        (0x7ff8000000000000, "1.#R"),
        (0x7ffc000000000000, "1.#R"),
        (0xfffc000000000000, "-1.#R"),
        (0x7ff4000000000000, "1.#T"),
        (0xfff4000000000000, "-1.#T"),
        (0x3ff0000000000000, "1.00"),
        (0x4059000000000000, "100.00"),
        (0x0000000000000000, "0.00"),
        (0x8000000000000000, "0.00"),
        (0x800000000000000f, "-0.00"),
        (0x0000000000000001, "0.00"),
        (0x3fc0000000000000, "0.13"),
        (0x3fd8000000000000, "0.38"),
        (0x3fe4000000000000, "0.63"),
        (0x3fec000000000000, "0.88"),
        (0x4005666666666666, "2.67"),
        (0x3ff0147ae147ae14, "1.00"),
        (0x3f8eb851eb851eb8, "0.01"),
        (0x4023fd70a3d70a3d, "9.99"),
        (0x4053c6147ae147ae, "79.09"),
        (0x437b69b4ba630f35, "123456789012345680.00"),
        (0xbfb999999999999a, "-0.10"),
        (0xbf847ae147ae147b, "-0.01"),
        (0xc059000000000000, "-100.00"),
        (0x40091eb851eb851f, "3.14"),
    ];

    #[test]
    fn vswprintf_10171dec_with_percent_dot_2f_matches_the_recorded_crt_output() {
        for (bits, expected) in ORACLE {
            assert_eq!(format_f2(f64::from_bits(*bits)), *expected, "{bits:016x}");
        }
    }

    #[test]
    fn rounding_is_half_up_on_the_seventeen_significant_digits() {
        assert_eq!(format_f2(0.125), "0.13");
        assert_eq!(format_f2(0.995), "1.00");
        assert_eq!(format_f2(99.995), "100.00");
        assert_eq!(format_f2(0.004), "0.00");
        assert_eq!(format_f2(0.005), "0.01");
    }

    #[test]
    fn x87_division_by_zero_yields_infinity_or_the_indefinite_nan() {
        assert_eq!(format_f2(f64::from(div_f32(5.0, 0.0))), "1.#J");
        assert_eq!(format_f2(f64::from(div_f32(-5.0, 0.0))), "-1.#J");
        assert_eq!(div_f32(0.0, 0.0).to_bits(), INDEFINITE_F32);
        assert_eq!(format_f2(f64::from(div_f32(0.0, 0.0))), "-1.#J");
        assert_eq!(div_f32(6.0, 3.0), 2.0);
    }
}
