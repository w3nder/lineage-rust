use killer_ui::initial_position::{InitialPosition, InitialPositionBackend};
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Default)]
struct Backend(Option<(i32, i32)>);

impl InitialPositionBackend for Backend {
    type RootRect = ();
    fn root_rect(&mut self) {}
    fn set_position(&mut self, x: i32, y: i32, (): ()) { self.0 = Some((x, y)); }
}

#[test]
#[ignore]
fn restore_initial_position_10036750_matches_all_defined_modes() {
    let mut cases = Vec::new();
    for horizontal_mode in [0, 1, 2, 5] {
        for vertical_mode in [2, 3, 4, 5] {
            for screen in [(1558, 933), (1559, 934), (0, 0), (-3, -5), (i32::MIN, i32::MAX)] {
                for value in [0, 1, -1, 3, -3, 172, 187, i32::MIN, i32::MAX] {
                    for enabled in [false, true] {
                        cases.push((InitialPosition { enabled, horizontal_mode, vertical_mode, x: value, y: value }, screen));
                    }
                }
            }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/initial_position_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for (position, (width, height)) in &cases {
        writeln!(input, "{} {} {} {} {} {} {}", u8::from(position.enabled), position.horizontal_mode,
            position.vertical_mode, position.x, position.y, width, height).unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{:?}", output.status);
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for ((position, screen), line) in cases.iter().zip(output.lines()) {
        let mut backend = Backend::default();
        position.restore(*screen, &mut backend).unwrap();
        let actual = backend.0.map_or_else(|| "none".into(), |(x, y)| format!("{x} {y}"));
        assert_eq!(actual, line, "{position:?} {screen:?}");
    }
}
