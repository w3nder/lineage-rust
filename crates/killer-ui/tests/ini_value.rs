use killer_ui::ini_value::{wide_atoi, wide_digit};
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore]
fn msvcr70_digits_and_integer_conversion_match_original_instructions() {
    let mut cases: Vec<Vec<u16>> = ["", "1", "2", "True", "-1", "+01", "1tail", "1.5", "2147483648",
        "-2147483648", "4294967295", "4294967296", "4294967297", "-4294967295", "\t\r\n 1", "\u{a0}1",
        "+١٢3", "-１２３", "12\0ignored", "+௧", "+😀", "--1"]
        .into_iter().map(|text| text.encode_utf16().collect()).collect();
    for unit in 0..=u16::MAX {
        cases.push(vec![0x2b, unit, 0x31]);
    }
    for count in 1..=100 {
        cases.push(vec![0x39; count]);
        cases.push(std::iter::once(0x2d).chain(std::iter::repeat_n(0x39, count)).collect());
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/ini_value_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for case in &cases {
        for unit in case { write!(input, "{unit:x} ").unwrap(); }
        writeln!(input).unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{:?}", output.status);
    assert_eq!(output.stdout.len(), (65536 + cases.len()) * 4);
    let values: Vec<_> = output.stdout.as_chunks::<4>().0.iter().map(|bytes| i32::from_le_bytes(*bytes)).collect();
    for unit in 0..=u16::MAX {
        assert_eq!(wide_digit(unit).unwrap_or(-1), values[usize::from(unit)], "unit {unit:04x}");
    }
    for (case, native) in cases.iter().zip(&values[65536..]) {
        assert_eq!(wide_atoi(case, |unit| matches!(unit, 9..=13 | 32 | 160)), *native, "{case:x?}");
    }
}
