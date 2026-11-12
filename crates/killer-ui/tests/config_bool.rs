use killer_ui::ini_value::config_bool;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore]
fn get_bool_1090f7d0_matches_executable_with_original_core_and_crt() {
    let mut cases: Vec<Option<Vec<u16>>> = vec![None];
    for text in ["", "True", "TRUE", "tRuE", "False", "FALSE", "TrUeX", " True", "True ", "1", "2", "-1",
        "+01", "1tail", "1.5", "4294967297", "-4294967295", "\t\r\n 1", "\u{a0}1", "True\0False", "1\0True"] {
        cases.push(Some(text.encode_utf16().collect()));
    }
    for unit in 0..=u16::MAX {
        cases.push(Some(vec![0x2b, unit, 0x31]));
    }
    for length in 0..=100 {
        for prefix in [0x30, 0x20] {
            cases.push(Some(std::iter::repeat_n(prefix, length).chain([0x31]).collect()));
        }
        cases.push(Some(std::iter::once(0x2d).chain(std::iter::repeat_n(0x39, length)).collect()));
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/config_bool_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for case in &cases {
        if let Some(units) = case {
            for unit in units { write!(input, "{unit:x} ").unwrap(); }
            writeln!(input).unwrap();
        } else {
            writeln!(input, "missing").unwrap();
        }
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{:?}", output.status);
    assert_eq!(output.stdout.len(), cases.len() * 4);
    for (case, bytes) in cases.iter().zip(output.stdout.as_chunks::<4>().0) {
        let native = i32::from_le_bytes(*bytes);
        let actual = config_bool(case.as_deref(), |value| {
            value.iter().copied().map(|unit| if (65..=90).contains(&unit) { unit + 32 } else { unit })
                .eq("true".encode_utf16())
        }, |unit| matches!(unit, 9..=13 | 32 | 160));
        assert_eq!(actual.map(i32::from).unwrap_or(-1), native, "{case:x?}");
    }
}
