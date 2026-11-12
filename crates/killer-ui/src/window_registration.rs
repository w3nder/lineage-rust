#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistrationList {
    All,
    Flag1000,
    Roots,
}

pub trait RegistrationBackend {
    type Record: Copy;
    type Window: Clone;
    fn count(&mut self) -> u32;
    fn allocate_record(&mut self, state: u32, window: Self::Window) -> Self::Record;
    fn set_window_flags(&mut self, window: Self::Window, flags: u32);
    fn bind_record(&mut self, record: Self::Record, parent: Option<Self::Record>);
    fn append(&mut self, list: RegistrationList, record: Self::Record);
    fn increment_count(&mut self);
    fn record_window(&mut self, record: Self::Record) -> Option<Self::Window>;
    fn window_flags(&mut self, window: Self::Window) -> u32;
    fn record_has_parent(&mut self, record: Self::Record) -> bool;
}

pub fn register_window<B: RegistrationBackend>(
    backend: &mut B,
    window: B::Window,
    flags: u32,
    parent: Option<B::Record>,
) -> Option<B::Record> {
    if backend.count() >= 3000 {
        return None;
    }
    let record = backend.allocate_record(1, window.clone());
    backend.set_window_flags(window, flags);
    backend.bind_record(record, parent);
    backend.append(RegistrationList::All, record);
    backend.increment_count();
    if let Some(window) = backend.record_window(record)
        && backend.window_flags(window) & 0x1000 != 0
    {
        backend.append(RegistrationList::Flag1000, record);
    }
    if !backend.record_has_parent(record) {
        backend.append(RegistrationList::Roots, record);
    }
    Some(record)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChildRegistrationFailed;

pub trait ChildRegistrationBackend {
    type Record: Copy;
    type Window: Clone;
    fn parent_record(&mut self) -> Option<Self::Record>;
    fn register_in_manager(&mut self, child: Self::Window, flags: u32, parent: Self::Record) -> Option<Self::Record>;
    fn set_record_parent(&mut self, record: Self::Record, parent: Option<Self::Record>);
    fn append_child(&mut self, record: Self::Record);
}

pub fn register_child<B: ChildRegistrationBackend>(
    backend: &mut B,
    child: B::Window,
    flags: u32,
) -> Result<Option<B::Record>, ChildRegistrationFailed> {
    let Some(parent) = backend.parent_record() else { return Ok(None) };
    let record = backend.register_in_manager(child, flags, parent).ok_or(ChildRegistrationFailed)?;
    let parent = backend.parent_record();
    backend.set_record_parent(record, parent);
    backend.append_child(record);
    Ok(Some(record))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window::WindowId;

    struct Backend {
        count: u32,
        flags: u32,
        parent: Option<u32>,
        window: Option<WindowId>,
        calls: Vec<String>,
    }

    impl RegistrationBackend for Backend {
        type Record = u32;
        type Window = WindowId;
        fn count(&mut self) -> u32 {
            self.calls.push("count".into());
            self.count
        }
        fn allocate_record(&mut self, state: u32, window: WindowId) -> u32 {
            self.calls.push(format!("allocate {state} {}", window.0));
            7
        }
        fn set_window_flags(&mut self, window: WindowId, flags: u32) {
            self.calls.push(format!("set {} {flags}", window.0));
            self.flags = flags;
        }
        fn bind_record(&mut self, record: u32, parent: Option<u32>) {
            assert_eq!(record, 7);
            self.calls.push("bind".into());
            self.parent = parent;
        }
        fn append(&mut self, list: RegistrationList, record: u32) {
            assert_eq!(record, 7);
            self.calls.push(format!("append {list:?}"));
        }
        fn increment_count(&mut self) {
            self.calls.push("increment".into());
            self.count += 1;
        }
        fn record_window(&mut self, _: u32) -> Option<WindowId> {
            self.calls.push("window".into());
            self.window
        }
        fn window_flags(&mut self, window: WindowId) -> u32 {
            self.calls.push(format!("flags {}", window.0));
            self.flags
        }
        fn record_has_parent(&mut self, _: u32) -> bool {
            self.calls.push("parent".into());
            self.parent.is_some()
        }
    }

    #[test]
    fn registration_orders_effects_and_uses_only_flag_1000() {
        for flags in [0, 0x1000, !0x1000, u32::MAX] {
            for parent in [None, Some(42)] {
                for window in [None, Some(WindowId(23))] {
                    let mut b = Backend { count: 2999, flags: !flags, parent: None, window, calls: Vec::new() };
                    assert_eq!(register_window(&mut b, WindowId(12), flags, parent), Some(7));
                    let mut expected = vec![
                        "count".into(),
                        "allocate 1 12".into(),
                        format!("set 12 {flags}"),
                        "bind".into(),
                        "append All".into(),
                        "increment".into(),
                        "window".into(),
                    ];
                    if window.is_some() {
                        expected.push("flags 23".into());
                        if flags & 0x1000 != 0 {
                            expected.push("append Flag1000".into());
                        }
                    }
                    expected.push("parent".into());
                    if parent.is_none() {
                        expected.push("append Roots".into());
                    }
                    assert_eq!(b.calls, expected);
                    assert_eq!(b.count, 3000);
                }
            }
        }
    }

    #[test]
    fn unsigned_limit_has_no_effects_after_count_query() {
        for count in [3000, 3001, i32::MAX as u32, 0x80000000, u32::MAX] {
            let mut b = Backend { count, flags: 0, parent: None, window: None, calls: Vec::new() };
            assert_eq!(register_window(&mut b, WindowId(12), u32::MAX, Some(9)), None);
            assert_eq!(b.calls, ["count"]);
            assert_eq!(b.count, count);
        }
    }

    struct ChildBackend {
        parent: Option<u32>,
        next_parent: Option<u32>,
        success: bool,
        calls: Vec<String>,
    }

    impl ChildRegistrationBackend for ChildBackend {
        type Record = u32;
        type Window = WindowId;
        fn parent_record(&mut self) -> Option<u32> {
            self.calls.push("parent".into());
            self.parent
        }
        fn register_in_manager(&mut self, child: WindowId, flags: u32, parent: u32) -> Option<u32> {
            assert_eq!(child, WindowId(12));
            assert_eq!(flags, 0x22);
            assert_eq!(parent, 7);
            self.calls.push("register".into());
            self.parent = self.next_parent;
            self.success.then_some(9)
        }
        fn set_record_parent(&mut self, record: u32, parent: Option<u32>) {
            assert_eq!(record, 9);
            assert_eq!(parent, self.next_parent);
            self.calls.push("bind".into());
        }
        fn append_child(&mut self, record: u32) {
            assert_eq!(record, 9);
            self.calls.push("append".into());
        }
    }

    #[test]
    fn child_registration_reloads_parent_and_exposes_null_result() {
        for parent in [None, Some(7)] {
            for next_parent in [None, Some(8)] {
                for success in [false, true] {
                    let mut b = ChildBackend { parent, next_parent, success, calls: Vec::new() };
                    let result = register_child(&mut b, WindowId(12), 0x22);
                    if parent.is_none() {
                        assert_eq!(result, Ok(None));
                        assert_eq!(b.calls, ["parent"]);
                    } else if success {
                        assert_eq!(result, Ok(Some(9)));
                        assert_eq!(b.calls, ["parent", "register", "parent", "bind", "append"]);
                    } else {
                        assert_eq!(result, Err(ChildRegistrationFailed));
                        assert_eq!(b.calls, ["parent", "register"]);
                    }
                }
            }
        }
    }
}
