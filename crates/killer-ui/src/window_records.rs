#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RecordKey {
    index: usize,
    generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowRecord<W> {
    pub state: u32,
    pub window: Option<W>,
    pub manager_bound: bool,
    pub parent: Option<RecordKey>,
}

#[derive(Debug)]
struct RecordSlot<W> {
    generation: u64,
    record: Option<WindowRecord<W>>,
}

#[derive(Debug)]
pub struct WindowRecords<W> {
    slots: Vec<RecordSlot<W>>,
    free: Vec<usize>,
}

impl<W> Default for WindowRecords<W> {
    fn default() -> Self {
        Self { slots: Vec::new(), free: Vec::new() }
    }
}

impl<W> WindowRecords<W> {
    pub fn allocate(&mut self, state: u32, window: W) -> RecordKey {
        let record = Some(WindowRecord { state, window: Some(window), manager_bound: false, parent: None });
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index];
            slot.record = record;
            RecordKey { index, generation: slot.generation }
        } else {
            let index = self.slots.len();
            self.slots.push(RecordSlot { generation: 0, record });
            RecordKey { index, generation: 0 }
        }
    }

    pub fn get(&self, key: RecordKey) -> Option<&WindowRecord<W>> {
        self.slots.get(key.index).filter(|slot| slot.generation == key.generation)?.record.as_ref()
    }

    pub fn get_mut(&mut self, key: RecordKey) -> Option<&mut WindowRecord<W>> {
        self.slots.get_mut(key.index).filter(|slot| slot.generation == key.generation)?.record.as_mut()
    }

    pub fn free(&mut self, key: RecordKey) -> Option<WindowRecord<W>> {
        let slot = self.slots.get_mut(key.index).filter(|slot| slot.generation == key.generation)?;
        let record = slot.record.take()?;
        if let Some(generation) = slot.generation.checked_add(1) {
            slot.generation = generation;
            self.free.push(key.index);
        }
        Some(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window_children::WindowChildren;

    #[test]
    fn record_reuse_invalidates_old_keys_without_merging_equal_windows() {
        let mut records = WindowRecords::default();
        let root = records.allocate(1, 7);
        assert!(!records.get(root).unwrap().manager_bound);
        let first = records.allocate(1, 12);
        let second = records.allocate(1, 12);
        records.get_mut(first).unwrap().parent = Some(root);
        assert_ne!(first, second);
        assert_eq!(records.get(second).unwrap().parent, None);
        assert_eq!(records.free(first).unwrap().parent, Some(root));
        assert_eq!(records.free(first), None);
        let replacement = records.allocate(1, 33);
        assert_eq!(first.index, replacement.index);
        assert_ne!(first, replacement);
        assert_eq!(records.get(first), None);
        assert_eq!(records.get_mut(first), None);
        assert_eq!(records.free(first), None);
        assert_eq!(records.get(replacement).unwrap().window, Some(33));
        assert_eq!(records.get(second).unwrap().window, Some(12));
    }

    #[test]
    fn lists_use_record_identity_and_do_not_silently_rebind_freed_entries() {
        let mut records = WindowRecords::default();
        let first = records.allocate(1, 12);
        let second = records.allocate(1, 12);
        let mut children = WindowChildren::default();
        let first_node = children.insert(first, || 0);
        let second_node = children.insert(second, || 0);
        let duplicate = children.insert(first, || 0);
        assert_eq!(children.promote(second, |_, _| 0), Ok(true));
        assert_eq!(children.first(), Some(second_node));
        assert_eq!(children.next(second_node), Some(first_node));
        assert_eq!(children.next(first_node), Some(duplicate));
        records.free(first).unwrap();
        let replacement = records.allocate(1, 19);
        assert_eq!(children.record(first_node), Some(first));
        assert_eq!(records.get(children.record(first_node).unwrap()), None);
        assert_ne!(children.record(first_node), Some(replacement));
        assert_eq!(children.remove(first), Some(first_node));
        assert_eq!(children.record(duplicate), Some(first));
        assert_eq!(children.record(second_node), Some(second));
    }

    #[test]
    fn repeated_lifecycle_reuses_storage_and_generation_overflow_retires_slot() {
        let mut records = WindowRecords::default();
        for value in 0..100_000 {
            let key = records.allocate(1, value);
            assert_eq!(records.free(key).unwrap().window, Some(value));
            assert_eq!(records.get(key), None);
        }
        assert_eq!(records.slots.len(), 1);
        assert_eq!(records.free.len(), 1);
        records.slots[0].generation = u64::MAX;
        let last = records.allocate(1, 42);
        records.free(last).unwrap();
        assert!(records.free.is_empty());
        let next = records.allocate(1, 43);
        assert_ne!(last.index, next.index);
        assert_eq!(records.get(last), None);
    }
}
