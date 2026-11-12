pub trait WindowHideBackend {
    type ChildNode: Copy;
    type Record: Copy + PartialEq;
    fn flags(&mut self) -> u32;
    fn first_child(&mut self) -> Option<Self::ChildNode>;
    fn set_flags(&mut self, flags: u32);
    fn hide_child(&mut self, node: Self::ChildNode);
    fn next_child(&mut self, node: Self::ChildNode) -> Option<Self::ChildNode>;
    fn has_focus(&mut self) -> bool;
    fn focused_record(&mut self) -> Option<Self::Record>;
    fn record(&mut self) -> Option<Self::Record>;
    fn clear_focused_record(&mut self);
    fn clear_own_focus(&mut self);
    fn select_focus_after_hide(&mut self, record: Option<Self::Record>);
}

pub fn hide_window(backend: &mut impl WindowHideBackend) {
    let flags = backend.flags();
    let mut node = backend.first_child();
    backend.set_flags(flags & !2);
    while let Some(child) = node {
        backend.hide_child(child);
        node = backend.next_child(child);
    }
    if backend.has_focus() {
        if backend.focused_record() == backend.record() {
            backend.clear_focused_record();
        }
        let record = backend.record();
        backend.clear_own_focus();
        backend.select_focus_after_hide(record);
    }
}
