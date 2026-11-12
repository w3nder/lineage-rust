#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnregistrationList {
    Roots,
    Flag1000,
    At60,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidRemovalIndex(pub usize);

pub trait UnregistrationBackend {
    type Record: Copy + Eq;
    fn record_has_parent(&mut self, record: Self::Record) -> bool;
    fn records(&self, list: UnregistrationList) -> &[Self::Record];
    fn remove(&mut self, list: UnregistrationList, index: usize);
    fn notify_window(&mut self, record: Self::Record, message: u32, first: u32, second: u32);
    fn free_record(&mut self, record: Self::Record);
    fn decrement_count(&mut self);
}

pub fn unregister_window<B: UnregistrationBackend>(
    backend: &mut B,
    record: B::Record,
) -> Result<(), InvalidRemovalIndex> {
    for list in [UnregistrationList::Roots, UnregistrationList::Flag1000, UnregistrationList::At60] {
        if list == UnregistrationList::Roots && backend.record_has_parent(record) {
            continue;
        }
        if let Some(index) = backend.records(list).iter().position(|&candidate| candidate == record) {
            backend.remove(list, index);
        }
    }
    let Some(index) = backend.records(UnregistrationList::All).iter().position(|&candidate| candidate == record) else {
        return Ok(());
    };
    backend.notify_window(record, 2, 0, 0);
    if index >= backend.records(UnregistrationList::All).len() {
        return Err(InvalidRemovalIndex(index));
    }
    backend.remove(UnregistrationList::All, index);
    backend.free_record(record);
    backend.decrement_count();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Backend {
        parent: bool,
        lists: [Vec<u32>; 4],
        calls: Vec<String>,
        mutation: u8,
        count: u32,
    }

    impl UnregistrationBackend for Backend {
        type Record = u32;
        fn record_has_parent(&mut self, _: u32) -> bool {
            self.parent
        }
        fn records(&self, list: UnregistrationList) -> &[u32] {
            &self.lists[list as usize]
        }
        fn remove(&mut self, list: UnregistrationList, index: usize) {
            self.calls.push(format!("remove {list:?} {index}"));
            self.lists[list as usize].remove(index);
        }
        fn notify_window(&mut self, record: u32, message: u32, first: u32, second: u32) {
            assert_eq!((record, message, first, second), (7, 2, 0, 0));
            self.calls.push("notify".into());
            match self.mutation {
                1 => self.lists[3].clear(),
                2 => self.lists[3][1] = 99,
                _ => {}
            }
        }
        fn free_record(&mut self, record: u32) {
            assert_eq!(record, 7);
            self.calls.push("free".into());
        }
        fn decrement_count(&mut self) {
            self.count = self.count.wrapping_sub(1);
            self.calls.push("decrement".into());
        }
    }

    #[test]
    fn removal_order_first_duplicates_and_absence_in_all() {
        for mask in 0..16 {
            for parent in [false, true] {
                let lists =
                    std::array::from_fn(|index| if mask & (1 << index) != 0 { vec![1, 7, 2, 7] } else { vec![1, 2] });
                let mut b = Backend { parent, lists, calls: Vec::new(), mutation: 0, count: 0 };
                unregister_window(&mut b, 7).unwrap();
                let mut expected = Vec::new();
                for (index, name) in ["Roots", "Flag1000", "At60", "All"].into_iter().enumerate() {
                    let removed = mask & (1 << index) != 0 && !(index == 0 && parent);
                    if removed {
                        if index == 3 {
                            expected.push("notify".into());
                        }
                        expected.push(format!("remove {name} 1"));
                        assert_eq!(b.lists[index], [1, 2, 7]);
                    }
                }
                if mask & 8 != 0 {
                    expected.extend(["free".into(), "decrement".into()]);
                    assert_eq!(b.count, u32::MAX);
                } else {
                    assert_eq!(b.count, 0);
                }
                assert_eq!(b.calls, expected);
            }
        }
    }

    #[test]
    fn callback_keeps_captured_index_and_invalid_index_is_not_silently_skipped() {
        for mutation in [1, 2] {
            let mut b = Backend {
                parent: false,
                lists: [vec![], vec![], vec![], vec![1, 7, 2, 7]],
                calls: Vec::new(),
                mutation,
                count: 4,
            };
            let result = unregister_window(&mut b, 7);
            if mutation == 1 {
                assert_eq!(result, Err(InvalidRemovalIndex(1)));
                assert_eq!(b.calls, ["notify"]);
                assert_eq!(b.count, 4);
            } else {
                assert_eq!(result, Ok(()));
                assert_eq!(b.lists[3], [1, 2, 7]);
                assert_eq!(b.calls, ["notify", "remove All 1", "free", "decrement"]);
            }
        }
    }
}
