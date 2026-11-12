use killer_ui::window::WindowId;
use killer_ui::window_manager::WindowManager;
use killer_ui::window_registration::{RegistrationBackend, RegistrationList, register_window};
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    count: u32,
    flags: u32,
    parent: Option<u32>,
    lengths: [u32; 3],
    events: Vec<u32>,
}

impl RegistrationBackend for Backend {
    type Record = u32;
    type Window = WindowId;
    fn count(&mut self) -> u32 {
        self.count
    }
    fn allocate_record(&mut self, state: u32, window: WindowId) -> u32 {
        assert_eq!(state, 1);
        assert_eq!(window, WindowId(12));
        self.events.push(1);
        7
    }
    fn set_window_flags(&mut self, window: WindowId, flags: u32) {
        assert_eq!(window, WindowId(12));
        self.flags = flags;
        self.events.push(2);
    }
    fn bind_record(&mut self, record: u32, parent: Option<u32>) {
        assert_eq!(record, 7);
        self.parent = parent;
    }
    fn append(&mut self, list: RegistrationList, record: u32) {
        assert_eq!(record, 7);
        let index = match list {
            RegistrationList::All => 0,
            RegistrationList::Flag1000 => 1,
            RegistrationList::Roots => 2,
        };
        self.lengths[index] += 1;
        self.events.push([3, 5, 6][index]);
    }
    fn increment_count(&mut self) {
        self.count += 1;
    }
    fn record_window(&mut self, _: u32) -> Option<WindowId> {
        Some(WindowId(12))
    }
    fn window_flags(&mut self, _: WindowId) -> u32 {
        self.events.push(4);
        self.flags
    }
    fn record_has_parent(&mut self, _: u32) -> bool {
        self.parent.is_some()
    }
}

#[test]
#[ignore]
fn registration_10039ba0_matches_original_limit_order_flags_and_lists() {
    let mut cases = Vec::new();
    let mut seed = 0xdea91023_u32;
    let mut flags = vec![0, 0x1000, !0x1000, u32::MAX];
    for _ in 0..128 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        flags.push(seed);
    }
    for count in [0, 1, 2998, 2999, 3000, 3001, 0x80000000, u32::MAX] {
        for &flags in &flags {
            for parent in [false, true] {
                cases.push((count, flags, parent));
            }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_registration_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for (count, flags, parent) in &cases {
        writeln!(input, "{count} {flags} {}", u8::from(*parent)).unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for ((count, flags, parent), line) in cases.into_iter().zip(output.lines()) {
        let mut b = Backend { count, flags: 0xa5a5a5a5, parent: None, lengths: [2; 3], events: Vec::new() };
        let record = register_window(&mut b, WindowId(12), flags, parent.then_some(42));
        let mut actual = vec![u32::from(record.is_some()), b.count, b.flags];
        actual.extend(b.lengths);
        actual.extend(b.events);
        let expected: Vec<u32> = line.split_whitespace().map(|v| v.parse().unwrap()).collect();
        assert_eq!(actual, expected, "{count} {flags} {parent}");
        let mut manager = WindowManager { count, ..WindowManager::default() };
        let prefix = [manager.records.allocate(1, WindowId(1)), manager.records.allocate(1, WindowId(2))];
        manager.all = prefix.to_vec();
        manager.flag1000 = prefix.to_vec();
        manager.roots = prefix.to_vec();
        let actual_flags = std::cell::Cell::new(0xa5a5a5a5);
        let record = manager.register(WindowId(12), flags, parent.then_some(prefix[0]), |m, w, value| {
            assert_eq!(m.count, count);
            assert_eq!(w, WindowId(12));
            actual_flags.set(value);
            Ok::<_, std::convert::Infallible>(())
        }, |m, w| {
            assert_eq!(m.count, count + 1);
            assert_eq!(w, WindowId(12));
            assert!(m.records.get(*m.all.last().unwrap()).unwrap().manager_bound);
            Ok(actual_flags.get())
        }).unwrap();
        let concrete = [u32::from(record.is_some()), manager.count, actual_flags.get(), manager.all.len() as u32, manager.flag1000.len() as u32, manager.roots.len() as u32];
        assert_eq!(concrete, expected[..6], "concrete {count} {flags} {parent}");
        if let Some(record) = record {
            let stored = manager.records.get(record).unwrap();
            assert!(stored.manager_bound);
            assert_eq!((stored.state, stored.window, stored.parent), (1, Some(WindowId(12)), parent.then_some(prefix[0])));
        }
        for list in [&manager.all, &manager.flag1000, &manager.roots] {
            assert_eq!(list[..2], prefix);
            assert!(list.len() == 2 || list[2] == record.unwrap());
        }
    }
}
