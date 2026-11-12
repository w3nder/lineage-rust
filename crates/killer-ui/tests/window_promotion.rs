use killer_ui::window_children::{ChildNodeId, PromotionError, WindowChildren, WindowPromotionBackend, promote_to_ancestors};
use killer_ui::window_instance::WindowInstance;
use killer_ui::window_records::{RecordKey, WindowRecords};
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    lists: Vec<WindowChildren<RecordKey>>,
    ids: Vec<Vec<ChildNodeId>>,
    records: WindowRecords<WindowInstance>,
    keys: Vec<RecordKey>,
    windows: Vec<WindowInstance>,
    mask: u32,
    mutate: bool,
    inserted: bool,
    events: Vec<u32>,
}

impl WindowPromotionBackend for Backend {
    type Window = WindowInstance;
    type Record = RecordKey;
    fn promote_local(&mut self, window: WindowInstance, record: RecordKey) -> Result<(), PromotionError> {
        let Self { lists, ids, keys, windows, mask, mutate, inserted, events, .. } = self;
        let index = windows.iter().position(|candidate| *candidate == window).unwrap() - 10;
        lists[index].promote(record, |list, queried| {
            let numeric = keys.iter().position(|key| *key == queried).unwrap() as u32;
            events.push(numeric);
            if *mutate && numeric == 0 && !*inserted {
                ids[index].push(list.insert(keys[7], || 0x800000));
                *inserted = true;
            }
            if *mask & (1 << (numeric % 5)) != 0 { 0x800000 } else { 0 }
        })?;
        Ok(())
    }
    fn window_record(&mut self, window: WindowInstance) -> Option<RecordKey> {
        self.windows.iter().position(|candidate| *candidate == window).map(|index| self.keys[index])
    }
    fn parent_window(&mut self, record: RecordKey) -> Option<WindowInstance> {
        let parent = self.records.get(record)?.parent?;
        self.records.get(parent)?.window.clone()
    }
}

#[test]
#[ignore]
fn promotion_matches_native_node_identity_flags_and_ancestor_propagation() {
    let mut cases = Vec::new();
    for mask in 0..32 {
        for target in 0..7 {
            for ancestors in 0..3 {
                for duplicate in 0..2 {
                    for mutation in 0..2 { cases.push([mask, target, ancestors, duplicate, mutation]); }
                }
            }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_promotion_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for v in &cases { writeln!(input, "{}", v.iter().map(u32::to_string).collect::<Vec<_>>().join(" ")).unwrap(); }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for (v, line) in cases.iter().zip(output.lines()) {
        let windows: Vec<_> = (0..13).map(|_| WindowInstance::default()).collect();
        let mut records = WindowRecords::default();
        let stale = records.allocate(1, WindowInstance::default());
        records.free(stale).unwrap();
        let keys: Vec<_> = windows.iter().map(|window| records.allocate(1, window.clone())).collect();
        assert_ne!(stale, keys[0]);
        assert!(records.get(stale).is_none());
        for index in 0..v[2] as usize {
            records.get_mut(keys[10 + index]).unwrap().parent = Some(keys[11 + index]);
        }
        let mut b = Backend { lists: Vec::new(), ids: Vec::new(), records, keys, windows, mask: v[0], mutate: v[4] != 0, inserted: false, events: Vec::new() };
        for entries in [vec![0, 1, 2, if v[3] == 0 { 3 } else { 1 }, 4], vec![6, 10, 5], vec![6, 11]] {
            let mut list = WindowChildren::default();
            let ids = entries.into_iter().map(|id| list.insert(b.keys[id], || 0)).collect();
            b.lists.push(list); b.ids.push(ids);
        }
        let owner = b.windows[10].clone();
        let record = b.keys[v[1] as usize];
        promote_to_ancestors(&mut b, owner, record).unwrap();
        let mut expected = Vec::new();
        for (list, ids) in b.lists.iter().zip(&b.ids) {
            let mut current = list.first();
            let mut previous = None;
            let mut order = Vec::new();
            while let Some(node) = current {
                assert_eq!(list.previous(node), previous);
                let index = ids.iter().position(|id| *id == node).unwrap() as u32;
                assert!(!order.contains(&index));
                order.push(index);
                previous = Some(node); current = list.next(node);
            }
            assert_eq!(list.last(), previous);
            expected.push(order.len() as u32); expected.extend(order);
        }
        expected.push(b.events.len() as u32); expected.extend(b.events);
        let actual: Vec<u32> = line.split_whitespace().map(|value| value.parse().unwrap()).collect();
        assert_eq!(actual, expected, "case={v:?}");
    }
}
