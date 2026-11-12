#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissingInitialFocusRoot;

pub trait FocusSelectionBackend {
    type Record: Copy;
    type Window: Clone + PartialEq;
    type ChildNode: Copy;
    fn root(&mut self) -> Option<Self::Window>;
    fn parent_record(&mut self, record: Self::Record) -> Option<Self::Record>;
    fn record_window(&mut self, record: Self::Record) -> Self::Window;
    fn is_visible(&mut self, window: Self::Window) -> bool;
    fn window_record(&mut self, window: Self::Window) -> Option<Self::Record>;
    fn first_child(&mut self, window: Self::Window) -> Option<Self::ChildNode>;
    fn next_child(&mut self, node: Self::ChildNode) -> Option<Self::ChildNode>;
    fn child_window(&mut self, node: Self::ChildNode) -> Self::Window;
    fn flags(&mut self, window: Self::Window) -> u32;
    fn set_focus(&mut self, window: Self::Window);
}

pub fn select_focus_after_hide<B: FocusSelectionBackend>(
    backend: &mut B,
    record: Option<B::Record>,
) -> Result<(), MissingInitialFocusRoot> {
    let Some(mut record) = record else { return Ok(()) };
    let root = backend.root().ok_or(MissingInitialFocusRoot)?;
    let is_descendant = loop {
        let Some(parent) = backend.parent_record(record) else { break false };
        if backend.record_window(parent) == root { break true; }
        record = parent;
    };
    if !backend.is_visible(root) || !is_descendant { return Ok(()); }
    let Some(root) = backend.root() else { return Ok(()) };
    let Some(record) = backend.window_record(root) else { return Ok(()) };
    let owner = backend.record_window(record);
    let mut node = backend.first_child(owner);
    while let Some(current) = node {
        let Some(next) = backend.next_child(current) else { break };
        let window = backend.child_window(next);
        let eligible = backend.flags(window) & 0x80000 == 0;
        let window = backend.child_window(next);
        if backend.is_visible(window) && eligible {
            let window = backend.child_window(next);
            backend.set_focus(window);
            break;
        }
        node = Some(next);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Backend {
        root: Option<usize>,
        root_visible: bool,
        root_record: bool,
        descendant: bool,
        masks: (u32, u32),
        children: usize,
        mutate: bool,
        replacement: usize,
        events: Vec<(u32, usize)>,
    }

    impl FocusSelectionBackend for Backend {
        type Record = usize;
        type Window = usize;
        type ChildNode = usize;
        fn root(&mut self) -> Option<usize> { self.root }
        fn parent_record(&mut self, record: usize) -> Option<usize> { (self.descendant && record != 0).then_some(record - 1) }
        fn record_window(&mut self, record: usize) -> usize { record }
        fn is_visible(&mut self, window: usize) -> bool {
            self.events.push((1, window));
            if window == 0 { return self.root_visible; }
            if self.mutate { self.replacement = 7; self.children = 4; }
            self.masks.0 & (1 << window) != 0
        }
        fn window_record(&mut self, window: usize) -> Option<usize> { self.root_record.then_some(window) }
        fn first_child(&mut self, _: usize) -> Option<usize> { (self.children != 0).then_some(0) }
        fn next_child(&mut self, node: usize) -> Option<usize> { (node + 1 < self.children).then_some(node + 1) }
        fn child_window(&mut self, node: usize) -> usize { if self.mutate && node == 1 && self.replacement != 0 { self.replacement } else { node } }
        fn flags(&mut self, window: usize) -> u32 {
            self.events.push((2, window));
            if self.mutate { self.replacement = 6; }
            if self.masks.1 & (1 << window) != 0 { 0x80000 } else { 0 }
        }
        fn set_focus(&mut self, window: usize) { self.events.push((3, window)); }
    }

    fn backend() -> Backend {
        Backend { root: Some(0), root_visible: true, root_record: true, descendant: true, masks: (0xff, 0), children: 4, mutate: false, replacement: 0, events: Vec::new() }
    }

    #[test]
    fn all_visibility_and_exclusion_combinations_skip_head_and_stop_at_first_candidate() {
        for visible in 0..16 {
            for excluded in 0..16 {
                let mut b = backend();
                b.masks = (visible, excluded);
                select_focus_after_hide(&mut b, Some(2)).unwrap();
                let mut expected = vec![(1, 0)];
                for child in 1..4 {
                    expected.extend([(2, child), (1, child)]);
                    if visible & (1 << child) != 0 && excluded & (1 << child) == 0 { expected.push((3, child)); break; }
                }
                assert_eq!(b.events, expected);
            }
        }
    }

    #[test]
    fn child_identity_is_reread_after_both_callbacks() {
        let mut b = backend();
        b.mutate = true;
        select_focus_after_hide(&mut b, Some(2)).unwrap();
        assert_eq!(b.events, [(1, 0), (2, 1), (1, 6), (3, 7)]);
    }

    #[test]
    fn null_input_and_missing_root_are_distinct_and_unrelated_record_still_checks_visibility() {
        let mut b = backend();
        b.root = None;
        assert_eq!(select_focus_after_hide(&mut b, None), Ok(()));
        assert_eq!(select_focus_after_hide(&mut b, Some(2)), Err(MissingInitialFocusRoot));
        assert!(b.events.is_empty());
        b.root = Some(0);
        b.descendant = false;
        select_focus_after_hide(&mut b, Some(2)).unwrap();
        assert_eq!(b.events, [(1, 0)]);
    }
}
