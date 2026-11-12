use killer_ui::window::WindowId;
use killer_ui::window_children::WindowChildren;
use killer_ui::window_records::{RecordKey, WindowRecord, WindowRecords};
use killer_ui::window_registration::{ChildRegistrationBackend, RegistrationBackend, RegistrationList, register_child, register_window};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Instance(usize);

#[derive(Default)]
struct Manager {
    windows: Vec<(WindowId, u32)>,
    records: WindowRecords<Instance>,
    lists: [Vec<RecordKey>; 3],
    count: u32,
    parent: Option<RecordKey>,
    children: WindowChildren<RecordKey>,
}

impl RegistrationBackend for Manager {
    type Window = Instance;
    type Record = RecordKey;

    fn count(&mut self) -> u32 { self.count }
    fn allocate_record(&mut self, state: u32, window: Instance) -> RecordKey {
        assert_eq!(state, 1);
        self.records.allocate(state, window)
    }
    fn set_window_flags(&mut self, window: Instance, flags: u32) {
        self.windows[window.0].1 = flags;
    }
    fn bind_record(&mut self, record: RecordKey, parent: Option<RecordKey>) {
        let stored = self.records.get_mut(record).unwrap();
        stored.manager_bound = true;
        stored.parent = parent;
    }
    fn append(&mut self, list: RegistrationList, record: RecordKey) {
        let index = match list {
            RegistrationList::All => 0,
            RegistrationList::Flag1000 => 1,
            RegistrationList::Roots => 2,
        };
        self.lists[index].push(record);
    }
    fn increment_count(&mut self) { self.count += 1; }
    fn record_window(&mut self, record: RecordKey) -> Option<Instance> {
        self.records.get(record).unwrap().window
    }
    fn window_flags(&mut self, window: Instance) -> u32 { self.windows[window.0].1 }
    fn record_has_parent(&mut self, record: RecordKey) -> bool { self.records.get(record).unwrap().parent.is_some() }
}

impl ChildRegistrationBackend for Manager {
    type Window = Instance;
    type Record = RecordKey;

    fn parent_record(&mut self) -> Option<RecordKey> { self.parent }
    fn register_in_manager(&mut self, child: Instance, flags: u32, parent: RecordKey) -> Option<RecordKey> {
        register_window(self, child, flags, Some(parent))
    }
    fn set_record_parent(&mut self, record: RecordKey, parent: Option<RecordKey>) {
        self.records.get_mut(record).unwrap().parent = parent;
    }
    fn append_child(&mut self, record: RecordKey) {
        let records = &self.records;
        let windows = &self.windows;
        self.children.insert(record, || windows[records.get(record).unwrap().window.unwrap().0].1);
    }
}

fn child_records(manager: &Manager) -> Vec<RecordKey> {
    let mut output = Vec::new();
    let mut node = manager.children.first();
    while let Some(current) = node {
        output.push(manager.children.record(current).unwrap());
        node = manager.children.next(current);
    }
    output
}

#[test]
fn composed_registration_distinguishes_instances_with_equal_ids_and_repeated_records() {
    let mut manager = Manager {
        windows: vec![(WindowId(0), 0), (WindowId(12), 0), (WindowId(12), 0)],
        ..Manager::default()
    };
    let root = register_window(&mut manager, Instance(0), 2, None).unwrap();
    manager.parent = Some(root);
    let first = register_child(&mut manager, Instance(1), 0x1002).unwrap().unwrap();
    let second = register_child(&mut manager, Instance(2), 2).unwrap().unwrap();
    let repeated = register_child(&mut manager, Instance(1), 8).unwrap().unwrap();
    assert_ne!(first, second);
    assert_ne!(first, repeated);
    assert_eq!(manager.windows, [(WindowId(0), 2), (WindowId(12), 8), (WindowId(12), 2)]);
    for (record, instance, parent) in [(root, 0, None), (first, 1, Some(root)), (second, 2, Some(root)), (repeated, 1, Some(root))] {
        assert_eq!(manager.records.get(record), Some(&WindowRecord { state: 1, window: Some(Instance(instance)), manager_bound: true, parent }));
    }
    assert_eq!(manager.lists, [vec![root, first, second, repeated], vec![first], vec![root]]);
    assert_eq!(child_records(&manager), [first, second, repeated]);
    assert_eq!(manager.count, 4);
}

#[test]
fn composed_registration_limit_has_no_partial_child_or_instance_mutation() {
    let mut manager = Manager {
        windows: vec![(WindowId(12), 0x22)],
        count: 3000,
        ..Manager::default()
    };
    let parent = manager.records.allocate(1, Instance(0));
    manager.parent = Some(parent);
    assert!(register_child(&mut manager, Instance(0), 0x1000).is_err());
    assert_eq!(manager.windows, [(WindowId(12), 0x22)]);
    assert_eq!(manager.count, 3000);
    assert_eq!(manager.records.get(parent).unwrap().window, Some(Instance(0)));
    assert_eq!(manager.children.first(), None);
    assert!(manager.lists.iter().all(Vec::is_empty));
}

#[test]
fn concrete_child_list_uses_prefix_flag_without_reordering_manager_arrays() {
    for mask in 0..8 {
        let mut manager = Manager { windows: vec![(WindowId(12), 0); 4], ..Manager::default() };
        let root = register_window(&mut manager, Instance(0), 2, None).unwrap();
        manager.parent = Some(root);
        let mut creation = Vec::new();
        let mut expected = Vec::new();
        for index in 1..4 {
            let prefix = mask & (1 << (index - 1)) != 0;
            let flags = 0x1002 | if prefix { 0x800000 } else { 0 };
            let record = register_child(&mut manager, Instance(index), flags).unwrap().unwrap();
            creation.push(record);
            if prefix { expected.insert(0, record); } else { expected.push(record); }
        }
        assert_eq!(child_records(&manager), expected);
        assert_eq!(&manager.lists[0][1..], creation);
        assert_eq!(manager.lists[1], creation);
        assert_eq!(manager.lists[2], [root]);
    }
}
