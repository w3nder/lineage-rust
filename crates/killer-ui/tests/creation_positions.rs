use std::io::Write;
use std::process::{Command, Stdio};
use killer_ui::initial_position::InitialPositionBackend;

struct DisabledBackend;

impl InitialPositionBackend for DisabledBackend {
    type RootRect = ();
    fn root_rect(&mut self) { panic!("disabled restore must not read root"); }
    fn set_position(&mut self, _: i32, _: i32, _: ()) { panic!("disabled restore must not move"); }
}

#[test]
#[ignore]
fn creation_positions_and_descriptors_match_original_calls_and_set_window_size() {
    let edges = [i32::MIN, -16_777_217, -503, -1, 0, 1, 301, 501, 503, 504, 505, 933, 1558, 16_777_217, i32::MAX];
    let mut cases = Vec::new();
    for kind in 0..34 {
        for width in edges {
            for height in edges {
                cases.push((kind, (width, height)));
            }
        }
    }
    let mut random = 0x93c5e71au32;
    for _ in 0..512 {
        random = random.wrapping_mul(1664525).wrapping_add(1013904223);
        let width = random as i32;
        random = random.wrapping_mul(1664525).wrapping_add(1013904223);
        for kind in 0..34 {
            cases.push((kind, (width, random as i32)));
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/creation_positions_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for (kind, (width, height)) in &cases {
        writeln!(input, "{kind} {width} {height}").unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{:?}", output.status);
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for ((kind, viewport), line) in cases.iter().zip(output.lines()) {
        if *kind == 33 {
            killer_ui::sysmsg::INITIAL_POSITION.restore(*viewport, &mut DisabledBackend).unwrap();
            assert_eq!(line, "0 -130 disabled", "viewport={viewport:?}");
            continue;
        }
        if *kind == 32 {
            killer_ui::option::INITIAL_POSITION.restore(*viewport, &mut DisabledBackend).unwrap();
            let (x, y) = killer_ui::option::creation_position(*viewport);
            assert_eq!(line, format!("{x} {y} disabled"), "viewport={viewport:?}");
            continue;
        }
        if *kind == 30 {
            killer_ui::initial_position::DISABLED_INITIAL_POSITION.restore(*viewport, &mut DisabledBackend).unwrap();
            assert_eq!(line, "disabled");
            continue;
        }
        let ((x, y), descriptor) = match kind {
            0 => (killer_ui::mainwnd::default_position(*viewport), killer_ui::mainwnd::initial_position(*viewport)),
            1 => (killer_ui::inventory::default_position(*viewport), killer_ui::inventory::initial_position(*viewport)),
            2 => (killer_ui::shop::default_position(*viewport), killer_ui::shop::initial_position(*viewport)),
            3 => (killer_ui::shop::WAREHOUSE_DEFAULT_POS, killer_ui::shop::WAREHOUSE_INITIAL_POSITION),
            4 => (killer_ui::trade::default_position(*viewport), killer_ui::trade::initial_position(*viewport)),
            5 => (killer_ui::shop::private_default_position(*viewport), killer_ui::shop::private_initial_position(*viewport)),
            6 => (killer_ui::multisell::default_position(*viewport), killer_ui::multisell::initial_position(*viewport)),
            7 => (killer_ui::train::default_position(*viewport), killer_ui::train::initial_position(*viewport)),
            8 => (killer_ui::enchant_wnd::default_position(*viewport), killer_ui::enchant_wnd::initial_position(*viewport)),
            9 => (killer_ui::macros::list_default_position(*viewport), killer_ui::macros::list_initial_position(*viewport)),
            10 => (killer_ui::macros::info_default_position(*viewport), killer_ui::macros::info_initial_position(*viewport)),
            11 => (killer_ui::macros::edit_default_position(*viewport), killer_ui::macros::edit_initial_position(*viewport)),
            12 | 13 => (killer_ui::henna::default_position(*viewport), killer_ui::henna::initial_position(*viewport)),
            14 => (killer_ui::party_room::making_default_position(*viewport), killer_ui::party_room::MAKING_INITIAL_POSITION),
            15 => (killer_ui::party_room::ROOM_POS, killer_ui::party_room::ROOM_INITIAL_POSITION),
            16 => (killer_ui::abnormal::ABNORMAL_DEFAULT_POS, killer_ui::abnormal::ABNORMAL_INITIAL_POSITION),
            17 => (killer_ui::party::PARTY_DEFAULT_POS, killer_ui::party::PARTY_INITIAL_POSITION),
            18 | 19 => (killer_ui::pet_status::PET_STATUS_DEFAULT_POS, killer_ui::pet_status::PET_STATUS_INITIAL_POSITION),
            20 => (killer_ui::target::default_position(*viewport), killer_ui::initial_position::TARGET_INITIAL_POSITION),
            21 => (killer_ui::npc_viewer::default_position(*viewport), killer_ui::npc_viewer::initial_position(*viewport)),
            22 => (killer_ui::menu::default_position(*viewport), killer_ui::menu::initial_position(*viewport)),
            23 => (killer_ui::sysmenu::default_position(*viewport), killer_ui::sysmenu::initial_position(*viewport)),
            24 => (killer_ui::minimap::default_position(*viewport), killer_ui::minimap::initial_position(*viewport)),
            25 => (killer_ui::shortcut::creation_position(*viewport), killer_ui::shortcut::initial_position(*viewport)),
            26 | 27 => (killer_ui::dialog::default_position(*kind == 27, *viewport), killer_ui::dialog::initial_position(*kind == 27, *viewport)),
            28 => (killer_ui::radar::default_position(*viewport), killer_ui::radar::initial_position(*viewport)),
            29 => (killer_ui::restart::default_position(*viewport), killer_ui::restart::RESTART_INITIAL_POSITION),
            31 => (killer_ui::chat::creation_position(*viewport), killer_ui::chat::initial_position(*viewport)),
            _ => unreachable!(),
        };
        assert_eq!(line, format!("{x} {y} {} {} {} {}", descriptor.horizontal_mode, descriptor.vertical_mode, descriptor.x, descriptor.y), "kind={kind} viewport={viewport:?}");
    }
}
