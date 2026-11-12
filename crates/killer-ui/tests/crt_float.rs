use killer_ui::crt_float::format_f2;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore]
fn vswprintf_10171dec_matches_the_executable_for_percent_dot_2f() {
    let mut cases: Vec<u64> = vec![
        0x7ff0000000000000, 0xfff0000000000000, 0xfff8000000000000, 0x7ff8000000000000, 0x7ffc000000000000,
        0xfffc000000000000, 0x7ff4000000000000, 0xfff4000000000000, 0, 0x8000000000000000, 1, 0x800000000000000f,
    ];
    let mut state = 0x2545_f491_4f6c_dd1du64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for _ in 0..4000 {
        let raw = next();
        cases.push(match raw % 4 {
            0 => f64::from((next() % 10_000_000) as f32 / 8.0).to_bits(),
            1 => f64::from_bits(raw).to_bits(),
            2 => (f64::from((next() % 100_000) as u32) / 1000.0).to_bits(),
            _ => f64::from(((next() % 1000) as f32) * 100.0 / ((next() % 977 + 1) as f32)).to_bits(),
        });
    }
    for value in [0.125, 0.375, 0.995, 99.995, 2.675, 1.005, 0.005, 1e15, 1e16] {
        cases.push(f64::to_bits(value));
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/crt_float_reference.py"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for bits in &cases {
        writeln!(input, "{bits:016x}").unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{:?}", output.status);
    let text = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), cases.len());
    for (bits, line) in cases.iter().zip(lines) {
        assert_eq!(format!("{}%", format_f2(f64::from_bits(*bits))), line, "{bits:016x}");
    }
}
