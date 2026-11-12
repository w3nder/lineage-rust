use killer_ui::notification_layout::{NotificationButton, NotificationLayoutBackend, reposition_notifications};
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    y: i32,
    visible: bool,
    moves: Vec<(NotificationButton, i32, i32)>,
}

impl NotificationLayoutBackend for Backend {
    type RootRect = ();
    fn root_rect(&mut self) {}
    fn chat_y(&mut self) -> i32 { self.y }
    fn chat_child_visible(&mut self) -> bool { self.visible }
    fn set_button_position(&mut self, button: NotificationButton, x: i32, y: i32, (): ()) {
        self.moves.push((button, x, y));
    }
}

#[test]
#[ignore]
fn notification_layout_1005cfe0_matches_original_instructions() {
    let cases: Vec<_> = [i32::MIN, -1, 0, 1, 36, 37, 166, 167, 746, 933, i32::MAX]
        .into_iter().flat_map(|y| [false, true].map(|visible| (y, visible))).collect();
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/notification_layout_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for (y, visible) in &cases { writeln!(input, "{y} {}", u8::from(*visible)).unwrap(); }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{:?}", output.status);
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for ((y, visible), native) in cases.into_iter().zip(output.lines()) {
        let mut backend = Backend { y, visible, moves: Vec::new() };
        reposition_notifications(&mut backend);
        assert_eq!(backend.moves.iter().map(|item| item.0).collect::<Vec<_>>(),
            [NotificationButton::Tutorial, NotificationButton::Quest, NotificationButton::Mail]);
        let actual = backend.moves.iter().flat_map(|(_, x, y)| [x.to_string(), y.to_string()]).collect::<Vec<_>>().join(" ");
        assert_eq!(actual, native, "y={y} visible={visible}");
    }
}
