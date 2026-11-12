use killer_ui::window_unregistration::{UnregistrationBackend, UnregistrationList, unregister_window};
use killer_ui::window_manager::WindowManager;
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    parent: bool,
    lists: [Vec<u32>; 4],
    events: Vec<u32>,
    mutation: bool,
    prefix: usize,
    count: u32,
}

impl UnregistrationBackend for Backend {
    type Record = u32;
    fn record_has_parent(&mut self, record: u32) -> bool {
        assert_eq!(record, 7);
        self.parent
    }
    fn records(&self, list: UnregistrationList) -> &[u32] {
        &self.lists[list as usize]
    }
    fn remove(&mut self, list: UnregistrationList, index: usize) {
        self.events.extend([10 + list as u32, index as u32]);
        self.lists[list as usize].remove(index);
    }
    fn notify_window(&mut self, record: u32, message: u32, first: u32, second: u32) {
        assert_eq!((record, message, first, second), (7, 2, 0, 0));
        self.events.push(20);
        if self.mutation {
            self.lists[3][self.prefix] = 6;
        }
    }
    fn free_record(&mut self, record: u32) {
        assert_eq!(record, 7);
        self.events.push(21);
    }
    fn decrement_count(&mut self) {
        self.count = self.count.wrapping_sub(1);
    }
}

#[test]
#[ignore]
fn unregister_10039ce0_matches_native_lists_duplicates_order_and_mutations() {
    let mut cases = Vec::new();
    for mask in 0..16 {
        for parent in [false, true] {
            for prefix in 0..4 {
                for mutation in [false, true] {
                    for count in [0, 1, u32::MAX] {
                        cases.push((mask, parent, prefix, mutation, count));
                    }
                }
            }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_unregistration_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for (mask, parent, prefix, mutation, count) in &cases {
        writeln!(input, "{mask} {} {prefix} {} {count}", u8::from(*parent), u8::from(*mutation)).unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for ((mask, parent, prefix, mutation, count), line) in cases.into_iter().zip(output.lines()) {
        let lists = std::array::from_fn(|index| {
            let mut values: Vec<u32> = (1..=prefix as u32).collect();
            if mask & (1 << index) != 0 {
                values.extend([7, 5, 7]);
            } else {
                values.push(5);
            }
            values
        });
        let initial_lists = lists.clone();
        let mut b = Backend { parent, lists, events: Vec::new(), mutation, prefix, count };
        unregister_window(&mut b, 7).unwrap();
        let mut actual = vec![b.count];
        for values in b.lists {
            actual.extend(values);
            actual.push(0);
        }
        actual.extend(b.events);
        let expected: Vec<u32> = line.split_whitespace().map(|v| v.parse().unwrap()).collect();
        assert_eq!(actual, expected, "{mask} {parent} {prefix} {mutation} {count}");
        let mut manager = WindowManager { count, ..WindowManager::default() };
        let keys: Vec<_> = (0..8).map(|window| manager.records.allocate(1, window)).collect();
        let record = keys[7];
        manager.records.get_mut(record).unwrap().parent = parent.then_some(keys[0]);
        let lists = [UnregistrationList::Roots, UnregistrationList::Flag1000, UnregistrationList::At60, UnregistrationList::All];
        for (list, values) in lists.into_iter().zip(initial_lists) {
            *manager.list_mut(list) = values.into_iter().map(|value| keys[value as usize]).collect();
        }
        let mut notified = false;
        manager.unregister(record, |m, key, message, first, second| {
            assert_eq!((key, message, first, second), (record, 2, 0, 0));
            assert!(m.records.get(record).is_some());
            notified = true;
            if mutation { m.all[prefix] = keys[6]; }
            Ok::<_, std::convert::Infallible>(())
        }).unwrap();
        let mut concrete = vec![manager.count];
        for list in lists {
            concrete.extend(manager.list(list).iter().map(|key| keys.iter().position(|value| value == key).unwrap() as u32));
            concrete.push(0);
        }
        assert_eq!(concrete, expected[..concrete.len()], "concrete {mask} {parent} {prefix} {mutation} {count}");
        assert_eq!(notified, mask & 8 != 0);
        assert_eq!(manager.records.get(record).is_none(), notified);
    }
}
