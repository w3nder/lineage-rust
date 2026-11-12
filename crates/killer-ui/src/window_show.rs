pub trait WindowShowBackend {
    type ChildNode: Copy;
    type Error;
    fn flags(&mut self) -> u32;
    fn hide_minimized(&mut self) -> Result<(), Self::Error>;
    fn set_flags(&mut self, flags: u32);
    fn first_child(&mut self) -> Option<Self::ChildNode>;
    fn show_child(&mut self, node: Self::ChildNode) -> Result<(), Self::Error>;
    fn next_child(&mut self, node: Self::ChildNode) -> Option<Self::ChildNode>;
}

pub fn show_window<B: WindowShowBackend>(backend: &mut B) -> Result<(), B::Error> {
    if backend.flags() & 0x2000 != 0 { backend.hide_minimized()?; }
    let flags = backend.flags();
    backend.set_flags(flags | 2);
    let mut node = backend.first_child();
    while let Some(child) = node {
        backend.show_child(child)?;
        node = backend.next_child(child);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Backend {
        flags: u32,
        children: usize,
        mutate: bool,
        missing_minimized: bool,
        fail_child: Option<usize>,
        calls: Vec<usize>,
    }

    impl WindowShowBackend for Backend {
        type ChildNode = usize;
        type Error = &'static str;
        fn flags(&mut self) -> u32 { self.flags }
        fn hide_minimized(&mut self) -> Result<(), Self::Error> {
            self.calls.push(10);
            if self.missing_minimized { return Err("missing minimized"); }
            if self.mutate { self.flags = 0x4000; self.children = 2; }
            Ok(())
        }
        fn set_flags(&mut self, flags: u32) { self.calls.push(11); self.flags = flags; }
        fn first_child(&mut self) -> Option<usize> { self.calls.push(12); (self.children > 0).then_some(0) }
        fn show_child(&mut self, node: usize) -> Result<(), Self::Error> {
            self.calls.push(node);
            if self.fail_child == Some(node) { return Err("child"); }
            if self.mutate { self.children = 3; }
            Ok(())
        }
        fn next_child(&mut self, node: usize) -> Option<usize> { (node + 1 < self.children).then_some(node + 1) }
    }

    #[test]
    fn show_rereads_flags_and_head_after_minimized_hide_and_next_after_child_show() {
        let mut b = Backend { flags: 0x2000, children: 0, mutate: true, missing_minimized: false, fail_child: None, calls: Vec::new() };
        assert_eq!(show_window(&mut b), Ok(()));
        assert_eq!(b.flags, 0x4002);
        assert_eq!(b.calls, [10, 11, 12, 0, 1, 2]);
    }

    #[test]
    fn show_does_not_skip_children_when_already_visible_and_missing_minimized_is_not_ignored() {
        for flags in [0, 2, 0x80000, 0x80002] {
            let mut b = Backend { flags, children: 2, mutate: false, missing_minimized: true, fail_child: None, calls: Vec::new() };
            assert_eq!(show_window(&mut b), Ok(()));
            assert_eq!(b.flags, flags | 2);
            assert_eq!(b.calls, [11, 12, 0, 1]);
        }
        let mut b = Backend { flags: 0x2000, children: 2, mutate: false, missing_minimized: true, fail_child: None, calls: Vec::new() };
        assert_eq!(show_window(&mut b), Err("missing minimized"));
        assert_eq!(b.flags, 0x2000);
        assert_eq!(b.calls, [10]);
        b.missing_minimized = false;
        b.fail_child = Some(0);
        b.calls.clear();
        assert_eq!(show_window(&mut b), Err("child"));
        assert_eq!(b.flags, 0x2002);
        assert_eq!(b.calls, [10, 11, 12, 0]);
    }
}
