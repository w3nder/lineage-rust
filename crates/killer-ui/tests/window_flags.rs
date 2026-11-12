use killer_ui::window::WindowId;
use killer_ui::window_dispatch::WindowDispatch;
use killer_ui::window_owner::{WindowParent, original_registration, original_root_registration};
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore]
fn every_installed_window_uses_original_base_flag_accessors() {
    let mut ids: Vec<_> = (0..84).map(WindowId).collect();
    ids.extend([WindowId::SYSTEM_MSG, WindowId::RADAR, WindowId::ZONE_TITLE, WindowId::RESTART_MENU, WindowId::OPTION, WindowId::SYSTEM_MENU]);
    let registrations: Vec<_> = ids.into_iter().filter_map(|id| original_registration(id).map(|registration| (id, registration))).collect();
    assert_eq!(registrations.len(), 88);
    let roots = [WindowParent::PrimaryRoot, WindowParent::SecondaryRoot].map(|parent| original_root_registration(parent).unwrap());
    assert!(roots.iter().all(|root| root.flags_argument == 0xa));
    assert_eq!(original_root_registration(WindowParent::Window(WindowId::CHAT)), None);
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_flags_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for (_, registration) in &registrations { writeln!(input, "{}", registration.vtable).unwrap(); }
    for root in roots { writeln!(input, "{}", root.vtable).unwrap(); }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), registrations.len() + 2);
    for (root, line) in roots.into_iter().zip(output.lines().skip(registrations.len())) {
        let methods: Vec<u32> = line.split_whitespace().map(|word| word.parse().unwrap()).collect();
        let mut expected = vec![0x100347b0, 0x10003070];
        expected.extend(WindowDispatch::from_vtable(root.vtable).unwrap().methods());
        assert_eq!(methods, expected, "root {:08x}", root.vtable);
    }
    for ((id, registration), line) in registrations.into_iter().zip(output.lines()) {
        let methods: Vec<u32> = line.split_whitespace().map(|word| word.parse().unwrap()).collect();
        let mut expected = vec![0x100347b0, 0x10003070];
        expected.extend(WindowDispatch::from_vtable(registration.vtable).unwrap().methods());
        assert_eq!(methods, expected, "{id:?} {:08x}", registration.vtable);
    }
}
