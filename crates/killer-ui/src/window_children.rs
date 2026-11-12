#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChildNodeId {
    index: usize,
    generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChildRecord(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromotionError {
    EmptyList,
    InvalidNode,
    MissingOwnerRecord,
}

pub trait WindowPromotionBackend {
    type Window: Clone;
    type Record: Copy;
    fn promote_local(&mut self, window: Self::Window, record: Self::Record) -> Result<(), PromotionError>;
    fn window_record(&mut self, window: Self::Window) -> Option<Self::Record>;
    fn parent_window(&mut self, record: Self::Record) -> Option<Self::Window>;
}

pub fn promote_to_ancestors<B: WindowPromotionBackend>(backend: &mut B, mut window: B::Window, mut record: B::Record) -> Result<(), PromotionError> {
    loop {
        backend.promote_local(window.clone(), record)?;
        record = backend.window_record(window).ok_or(PromotionError::MissingOwnerRecord)?;
        let Some(parent) = backend.parent_window(record) else { return Ok(()) };
        window = parent;
    }
}

#[derive(Debug)]
struct ChildNode<R> {
    record: R,
    previous: Option<ChildNodeId>,
    next: Option<ChildNodeId>,
}

#[derive(Debug)]
struct ChildSlot<R> {
    generation: u64,
    node: Option<ChildNode<R>>,
}

#[derive(Debug)]
pub struct WindowChildren<R = ChildRecord> {
    nodes: Vec<ChildSlot<R>>,
    free: Vec<usize>,
    head: Option<ChildNodeId>,
    tail: Option<ChildNodeId>,
}

impl<R> Default for WindowChildren<R> {
    fn default() -> Self {
        Self { nodes: Vec::new(), free: Vec::new(), head: None, tail: None }
    }
}

impl<R: Copy + Eq> WindowChildren<R> {
    pub fn promote(
        &mut self,
        record: R,
        mut flags: impl FnMut(&mut Self, R) -> u32,
    ) -> Result<bool, PromotionError> {
        let head = self.head.ok_or(PromotionError::EmptyList)?;
        if self.record(head).ok_or(PromotionError::InvalidNode)? == record { return Ok(false); }
        let mut current = self.next(head);
        let target = loop {
            let Some(node) = current else { return Ok(false) };
            if self.record(node).ok_or(PromotionError::InvalidNode)? == record { break node; }
            current = self.next(node);
        };
        let previous = self.previous(target).ok_or(PromotionError::InvalidNode)?;
        let next = self.next(target);
        self.node_mut(previous).ok_or(PromotionError::InvalidNode)?.next = next;
        if let Some(next) = next {
            self.node_mut(next).ok_or(PromotionError::InvalidNode)?.previous = Some(previous);
        } else {
            self.tail = Some(previous);
        }
        let head = self.head.ok_or(PromotionError::EmptyList)?;
        let head_record = self.record(head).ok_or(PromotionError::InvalidNode)?;
        let before = if flags(self, head_record) & 0x800000 == 0 {
            Some(self.head.ok_or(PromotionError::EmptyList)?)
        } else {
            let head = self.head.ok_or(PromotionError::EmptyList)?;
            let mut node = self.next(head);
            while let Some(candidate) = node {
                let record = self.record(candidate).ok_or(PromotionError::InvalidNode)?;
                if flags(self, record) & 0x800000 == 0 { break; }
                self.node(candidate).ok_or(PromotionError::InvalidNode)?;
                node = self.next(candidate);
            }
            node
        };
        let previous = if let Some(before) = before {
            self.node(before).ok_or(PromotionError::InvalidNode)?.previous
        } else {
            Some(self.tail.ok_or(PromotionError::EmptyList)?)
        };
        if let Some(previous) = previous {
            self.node_mut(previous).ok_or(PromotionError::InvalidNode)?.next = Some(target);
        } else {
            self.head = Some(target);
        }
        if let Some(before) = before {
            self.node_mut(before).ok_or(PromotionError::InvalidNode)?.previous = Some(target);
        } else {
            self.tail = Some(target);
        }
        let target = self.node_mut(target).ok_or(PromotionError::InvalidNode)?;
        target.previous = previous;
        target.next = before;
        Ok(true)
    }

    pub fn first(&self) -> Option<ChildNodeId> {
        self.head
    }
    pub fn last(&self) -> Option<ChildNodeId> {
        self.tail
    }
    pub fn record(&self, id: ChildNodeId) -> Option<R> {
        self.node(id).map(|n| n.record)
    }
    pub fn next(&self, id: ChildNodeId) -> Option<ChildNodeId> {
        self.node(id).and_then(|n| n.next)
    }
    pub fn previous(&self, id: ChildNodeId) -> Option<ChildNodeId> {
        self.node(id).and_then(|n| n.previous)
    }
    fn node(&self, id: ChildNodeId) -> Option<&ChildNode<R>> {
        self.nodes.get(id.index).filter(|slot| slot.generation == id.generation).and_then(|slot| slot.node.as_ref())
    }
    fn node_mut(&mut self, id: ChildNodeId) -> Option<&mut ChildNode<R>> {
        self.nodes.get_mut(id.index).filter(|slot| slot.generation == id.generation).and_then(|slot| slot.node.as_mut())
    }
    pub fn insert(&mut self, record: R, flags: impl FnOnce() -> u32) -> ChildNodeId {
        let front = self.head.is_some() && flags() & 0x800000 != 0;
        let (previous, next) = if front { (None, self.head) } else { (self.tail, None) };
        let node = Some(ChildNode { record, previous, next });
        let id = if let Some(index) = self.free.pop() {
            let slot = &mut self.nodes[index];
            slot.node = node;
            ChildNodeId { index, generation: slot.generation }
        } else {
            let index = self.nodes.len();
            self.nodes.push(ChildSlot { generation: 0, node });
            ChildNodeId { index, generation: 0 }
        };
        if let Some(previous) = previous.and_then(|n| self.node_mut(n)) {
            previous.next = Some(id);
        } else {
            self.head = Some(id);
        }
        if let Some(next) = next.and_then(|n| self.node_mut(n)) {
            next.previous = Some(id);
        } else {
            self.tail = Some(id);
        }
        id
    }
    pub fn remove(&mut self, record: R) -> Option<ChildNodeId> {
        let mut current = self.head;
        while let Some(id) = current {
            let node = self.node(id)?;
            let previous = node.previous;
            let next = node.next;
            if node.record == record {
                if let Some(previous) = previous.and_then(|n| self.node_mut(n)) {
                    previous.next = next;
                } else {
                    self.head = next;
                }
                if let Some(next) = next.and_then(|n| self.node_mut(n)) {
                    next.previous = previous;
                } else {
                    self.tail = previous;
                }
                let slot = &mut self.nodes[id.index];
                slot.node = None;
                if let Some(generation) = slot.generation.checked_add(1) {
                    slot.generation = generation;
                    self.free.push(id.index);
                }
                return Some(id);
            }
            current = next;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ancestor_promotion_uses_each_owners_record_even_without_local_change() {
        struct Backend { calls: Vec<(u32, ChildRecord)> }
        impl WindowPromotionBackend for Backend {
            type Window = u32;
            type Record = ChildRecord;
            fn promote_local(&mut self, window: u32, record: ChildRecord) -> Result<(), PromotionError> { self.calls.push((window, record)); Ok(()) }
            fn window_record(&mut self, window: u32) -> Option<ChildRecord> { Some(ChildRecord(window + 10)) }
            fn parent_window(&mut self, record: ChildRecord) -> Option<u32> { (record.0 < 12).then_some(record.0 - 9) }
        }
        let mut backend = Backend { calls: Vec::new() };
        promote_to_ancestors(&mut backend, 0, ChildRecord(99)).unwrap();
        assert_eq!(backend.calls, [(0, ChildRecord(99)), (1, ChildRecord(10)), (2, ChildRecord(11))]);
    }

    fn ordered(list: &WindowChildren) -> Vec<ChildNodeId> {
        let mut out = Vec::new();
        let mut node = list.first();
        while let Some(current) = node {
            assert!(!out.contains(&current));
            assert_eq!(list.previous(current), out.last().copied());
            out.push(current);
            node = list.next(current);
        }
        assert_eq!(out.last().copied(), list.last());
        out
    }

    #[test]
    fn promotion_retains_node_identity_and_obeys_remaining_head_flag_prefix() {
        for mask in 0..32 {
            for target in 0..6 {
                let mut list = WindowChildren::default();
                let ids: Vec<_> = (0..5).map(|i| list.insert(ChildRecord(i), || 0)).collect();
                let mut expected = ids.clone();
                let mut calls = Vec::new();
                let changed = list.promote(ChildRecord(target), |_, record| {
                    calls.push(record.0);
                    if mask & (1 << record.0) != 0 { 0x800000 } else { 0 }
                }).unwrap();
                if target != 0 && target < 5 {
                    let id = expected.remove(target as usize);
                    let at = (0..expected.len()).find(|i| {
                        let record = list.record(expected[*i]).unwrap();
                        mask & (1 << record.0) == 0
                    }).unwrap_or(expected.len());
                    expected.insert(at, id);
                    assert!(changed);
                    assert!(!calls.contains(&target));
                } else {
                    assert!(!changed);
                    assert!(calls.is_empty());
                }
                assert_eq!(ordered(&list), expected, "mask={mask} target={target}");
                assert_eq!(list.nodes.len(), 5);
                assert!(list.free.is_empty());
                for (i, id) in ids.iter().enumerate() { assert_eq!(list.record(*id), Some(ChildRecord(i as u32))); }
            }
        }
    }

    #[test]
    fn promotion_rereads_live_head_and_handles_first_duplicate() {
        let mut list = WindowChildren::default();
        assert_eq!(list.promote(ChildRecord(1), |_, _| 0), Err(PromotionError::EmptyList));
        let a = list.insert(ChildRecord(0), || 0);
        let b = list.insert(ChildRecord(1), || 0);
        let c = list.insert(ChildRecord(1), || 0);
        let mut inserted = None;
        list.promote(ChildRecord(1), |list, record| {
            assert_eq!(record, ChildRecord(0));
            inserted = Some(list.insert(ChildRecord(2), || 0x800000));
            0
        }).unwrap();
        assert_eq!(ordered(&list), [b, inserted.unwrap(), a, c]);
    }

    #[test]
    fn insertion_preserves_duplicates_and_tests_only_native_flag() {
        let mut list = WindowChildren::default();
        let a = list.insert(ChildRecord(1), || panic!("empty list must not query flags"));
        let b = list.insert(ChildRecord(2), || 0x800000);
        let c = list.insert(ChildRecord(1), || !0x800000);
        assert_eq!(list.first(), Some(b));
        assert_eq!(list.next(b), Some(a));
        assert_eq!(list.next(a), Some(c));
        assert_eq!(list.previous(c), Some(a));
        assert_eq!(list.previous(a), Some(b));
        assert_eq!(list.last(), Some(c));
        assert_eq!(list.remove(ChildRecord(1)), Some(a));
        assert_eq!(list.next(b), Some(c));
        assert_eq!(list.previous(c), Some(b));
        assert_eq!(list.remove(ChildRecord(1)), Some(c));
        assert_eq!(list.last(), Some(b));
        assert_eq!(list.remove(ChildRecord(7)), None);
        assert_eq!(list.remove(ChildRecord(2)), Some(b));
        assert_eq!((list.first(), list.last()), (None, None));
        assert_eq!(list.record(a), None);
        let d = list.insert(ChildRecord(1), || 0);
        assert_ne!(d, a);
    }

    #[test]
    fn live_next_reflects_appends_and_removal_of_following_nodes() {
        let mut list = WindowChildren::default();
        let a = list.insert(ChildRecord(1), || 0);
        assert_eq!(list.next(a), None);
        let b = list.insert(ChildRecord(2), || 0);
        let c = list.insert(ChildRecord(3), || 0);
        assert_eq!(list.next(a), Some(b));
        list.remove(ChildRecord(2));
        assert_eq!(list.next(a), Some(c));
        list.remove(ChildRecord(1));
        assert_eq!(list.first(), Some(c));
        assert_eq!(list.previous(c), None);
    }

    #[test]
    fn repeated_removal_reuses_storage_without_reviving_old_handles() {
        let mut list = WindowChildren::default();
        let first = list.insert(ChildRecord(1), || 0);
        list.remove(ChildRecord(1));
        for _ in 0..100_000 {
            let current = list.insert(ChildRecord(2), || 0);
            assert_eq!(current.index, first.index);
            assert_ne!(current.generation, first.generation);
            assert_eq!(list.record(first), None);
            assert_eq!(list.next(first), None);
            assert_eq!(list.previous(first), None);
            assert!(list.node_mut(first).is_none());
            assert_eq!(list.remove(ChildRecord(2)), Some(current));
            assert_eq!(list.record(current), None);
        }
        assert_eq!(list.nodes.len(), 1);
        assert_eq!(list.free, [0]);
    }

    #[test]
    fn generation_exhaustion_retires_slot_instead_of_wrapping() {
        let mut list = WindowChildren::default();
        let old = list.insert(ChildRecord(1), || 0);
        let last = ChildNodeId { index: old.index, generation: u64::MAX };
        list.nodes[old.index].generation = u64::MAX;
        list.head = Some(last);
        list.tail = Some(last);
        assert_eq!(list.remove(ChildRecord(1)), Some(last));
        assert!(list.free.is_empty());
        let new = list.insert(ChildRecord(1), || 0);
        assert_ne!(new.index, old.index);
        assert_eq!(list.record(old), None);
        assert_eq!(list.record(last), None);
    }
}
