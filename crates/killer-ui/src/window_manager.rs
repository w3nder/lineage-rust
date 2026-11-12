use crate::window_records::{RecordKey, WindowRecords};
use crate::window_unregistration::UnregistrationList;

#[derive(Debug, PartialEq, Eq)]
pub enum ManagerError<E> {
    InvalidRecord(RecordKey),
    InvalidRemovalIndex(usize),
    Callback(E),
}

#[derive(Debug)]
pub struct WindowManager<W> {
    pub records: WindowRecords<W>,
    pub count: u32,
    pub all: Vec<RecordKey>,
    pub flag1000: Vec<RecordKey>,
    pub roots: Vec<RecordKey>,
    pub at60: Vec<RecordKey>,
}

impl<W> Default for WindowManager<W> {
    fn default() -> Self {
        Self { records: WindowRecords::default(), count: 0, all: Vec::new(), flag1000: Vec::new(), roots: Vec::new(), at60: Vec::new() }
    }
}

impl<W: Clone> WindowManager<W> {
    pub fn register<E>(
        &mut self,
        window: W,
        flags: u32,
        parent: Option<RecordKey>,
        set_flags: impl FnOnce(&mut Self, W, u32) -> Result<(), E>,
        get_flags: impl FnOnce(&mut Self, W) -> Result<u32, E>,
    ) -> Result<Option<RecordKey>, ManagerError<E>> {
        if self.count >= 3000 { return Ok(None); }
        let record = self.records.allocate(1, window.clone());
        set_flags(self, window, flags).map_err(ManagerError::Callback)?;
        let stored = self.records.get_mut(record).ok_or(ManagerError::InvalidRecord(record))?;
        stored.manager_bound = true;
        stored.parent = parent;
        self.all.push(record);
        self.count = self.count.wrapping_add(1);
        if let Some(window) = self.records.get(record).ok_or(ManagerError::InvalidRecord(record))?.window.clone()
            && get_flags(self, window).map_err(ManagerError::Callback)? & 0x1000 != 0
        { self.flag1000.push(record); }
        if self.records.get(record).ok_or(ManagerError::InvalidRecord(record))?.parent.is_none() {
            self.roots.push(record);
        }
        Ok(Some(record))
    }

    pub fn list(&self, list: UnregistrationList) -> &[RecordKey] {
        match list {
            UnregistrationList::Roots => &self.roots,
            UnregistrationList::Flag1000 => &self.flag1000,
            UnregistrationList::At60 => &self.at60,
            UnregistrationList::All => &self.all,
        }
    }

    pub fn list_mut(&mut self, list: UnregistrationList) -> &mut Vec<RecordKey> {
        match list {
            UnregistrationList::Roots => &mut self.roots,
            UnregistrationList::Flag1000 => &mut self.flag1000,
            UnregistrationList::At60 => &mut self.at60,
            UnregistrationList::All => &mut self.all,
        }
    }

    pub fn unregister<E>(
        &mut self,
        record: RecordKey,
        notify: impl FnOnce(&mut Self, RecordKey, u32, u32, u32) -> Result<(), E>,
    ) -> Result<(), ManagerError<E>> {
        for list in [UnregistrationList::Roots, UnregistrationList::Flag1000, UnregistrationList::At60] {
            if list == UnregistrationList::Roots
                && self.records.get(record).ok_or(ManagerError::InvalidRecord(record))?.parent.is_some()
            { continue; }
            if let Some(index) = self.list(list).iter().position(|&candidate| candidate == record) {
                self.list_mut(list).remove(index);
            }
        }
        let Some(index) = self.all.iter().position(|&candidate| candidate == record) else { return Ok(()); };
        notify(self, record, 2, 0, 0).map_err(ManagerError::Callback)?;
        if index >= self.all.len() { return Err(ManagerError::InvalidRemovalIndex(index)); }
        self.all.remove(index);
        self.records.free(record).ok_or(ManagerError::InvalidRecord(record))?;
        self.count = self.count.wrapping_sub(1);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::Infallible;

    fn register(manager: &mut WindowManager<u32>, window: u32, flags: u32) -> RecordKey {
        manager.register(window, flags, None, |_, _, _| Ok::<_, Infallible>(()), |_, _| Ok(flags)).unwrap().unwrap()
    }

    #[test]
    fn concrete_register_preserves_duplicates_limits_and_independent_lists() {
        for count in [0, 2999, 3000, 3001, 0x80000000, u32::MAX] {
            for flags in [0, 0x1000, !0x1000, u32::MAX] {
                let mut manager = WindowManager { count, ..WindowManager::default() };
                let record = manager.register(12, flags, None, |m, w, f| {
                    assert_eq!((m.count, w, f), (count, 12, flags));
                    assert!(m.all.is_empty());
                    Ok::<_, Infallible>(())
                }, |m, w| {
                    assert_eq!(m.count, count + 1);
                    assert_eq!(w, 12);
                    assert_eq!(m.records.get(m.all[0]).unwrap().state, 1);
                    Ok(flags)
                }).unwrap();
                if let Some(record) = record {
                    assert_eq!(manager.all, [record]);
                    assert_eq!(manager.roots, [record]);
                    assert_eq!(manager.flag1000, if flags & 0x1000 != 0 { vec![record] } else { vec![] });
                } else {
                    assert!(count >= 3000);
                    assert_eq!(manager.count, count);
                    assert!(manager.all.is_empty());
                }
            }
        }
        let mut manager = WindowManager::default();
        let a = register(&mut manager, 12, 0x1000);
        let b = register(&mut manager, 12, 0);
        assert_ne!(a, b);
        assert_eq!(manager.all, [a, b]);
        assert_eq!(manager.flag1000, [a]);
    }

    #[test]
    fn flag_callback_mutation_is_observed_before_root_insertion() {
        let mut manager = WindowManager::default();
        let parent = register(&mut manager, 1, 0);
        let child = manager.register(12, 0, Some(parent), |_, _, _| Ok::<_, Infallible>(()), |m, _| {
            let record = *m.all.last().unwrap();
            m.records.get_mut(record).unwrap().parent = None;
            Ok(0x1000)
        }).unwrap().unwrap();
        assert_eq!(manager.roots, [parent, child]);
        assert_eq!(manager.flag1000, [child]);
    }

    #[test]
    fn unregister_notifies_after_auxiliary_removals_and_keeps_captured_index() {
        let mut manager = WindowManager::default();
        let a = register(&mut manager, 1, 0x1000);
        let b = register(&mut manager, 2, 0);
        manager.at60 = vec![a, b, a];
        manager.unregister(a, |m, key, message, first, second| {
            assert_eq!((key, message, first, second), (a, 2, 0, 0));
            assert_eq!(m.all, [a, b]);
            assert_eq!(m.roots, [b]);
            assert!(m.flag1000.is_empty());
            assert_eq!(m.at60, [b, a]);
            m.all.swap(0, 1);
            Ok::<_, Infallible>(())
        }).unwrap();
        assert_eq!(manager.all, [a]);
        assert_eq!(manager.records.get(a), None);
        assert_eq!(manager.records.get(b).unwrap().window, Some(2));
        assert_eq!(manager.count, 1);
    }

    #[test]
    fn callback_errors_and_invalidated_index_preserve_prior_effects() {
        let mut manager = WindowManager::default();
        let record = register(&mut manager, 1, 0x1000);
        assert_eq!(manager.unregister(record, |_, _, _, _, _| Err("notify")), Err(ManagerError::Callback("notify")));
        assert!(manager.roots.is_empty());
        assert!(manager.flag1000.is_empty());
        assert_eq!(manager.all, [record]);
        assert_eq!(manager.count, 1);
        assert_eq!(manager.unregister(record, |m, _, _, _, _| {
            m.all.clear();
            Ok::<_, Infallible>(())
        }), Err(ManagerError::InvalidRemovalIndex(0)));
        assert!(manager.records.get(record).is_some());
        assert_eq!(manager.count, 1);
    }
}
