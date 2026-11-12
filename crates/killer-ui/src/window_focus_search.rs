use crate::window_focus::FOCUS_CLASS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissingRegisteredFocusWindow;

pub trait FocusSearchBackend {
    type Window: Clone;
    type ChildNode: Copy;
    fn registered_window(&mut self, window: Self::Window) -> Option<Self::Window>;
    fn focus_mode(&mut self, window: Self::Window) -> bool;
    fn visible(&mut self, window: Self::Window) -> bool;
    fn inherits(&mut self, window: Self::Window, class: u32) -> bool;
    fn registered_parent(&mut self, window: Self::Window) -> Option<Self::Window>;
    fn set_focus(&mut self, window: Self::Window);
    fn first_child(&mut self, window: Self::Window) -> Option<Self::ChildNode>;
    fn child_window(&mut self, node: Self::ChildNode) -> Self::Window;
    fn next_child(&mut self, node: Self::ChildNode) -> Option<Self::ChildNode>;
}

pub fn has_direct_focus_class<B: FocusSearchBackend>(backend: &mut B, window: B::Window) -> Result<bool, MissingRegisteredFocusWindow> {
    let owner = backend.registered_window(window).ok_or(MissingRegisteredFocusWindow)?;
    let mut node = backend.first_child(owner);
    while let Some(child) = node {
        let window = backend.child_window(child);
        if backend.inherits(window, FOCUS_CLASS) { return Ok(true); }
        node = backend.next_child(child);
    }
    Ok(false)
}

pub fn focus_descendant<B: FocusSearchBackend>(backend: &mut B, window: B::Window) -> Result<bool, MissingRegisteredFocusWindow> {
    search(backend, window, false)
}

pub fn focus_parent_of_class<B: FocusSearchBackend>(backend: &mut B, window: B::Window) -> Result<bool, MissingRegisteredFocusWindow> {
    search(backend, window, true)
}

fn search<B: FocusSearchBackend>(backend: &mut B, mut window: B::Window, by_class: bool) -> Result<bool, MissingRegisteredFocusWindow> {
    let mut pending = Vec::new();
    loop {
        let owner = backend.registered_window(window.clone()).ok_or(MissingRegisteredFocusWindow)?;
        if by_class {
            if backend.inherits(owner.clone(), FOCUS_CLASS) {
                if let Some(parent) = backend.registered_parent(window) { backend.set_focus(parent); }
                return Ok(true);
            }
        } else if backend.focus_mode(owner.clone()) && backend.visible(owner.clone()) {
            backend.set_focus(owner);
            return Ok(true);
        }
        let mut child = backend.first_child(owner);
        while child.is_none() {
            let Some(previous) = pending.pop() else { return Ok(false) };
            child = backend.next_child(previous);
        }
        if let Some(child) = child {
            pending.push(child);
            window = backend.child_window(child);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Backend {
        children: Vec<Vec<usize>>,
        classes: u32,
        modes: u32,
        visible: u32,
        mutate: bool,
        events: Vec<(u32, usize)>,
    }

    impl FocusSearchBackend for Backend {
        type Window = usize;
        type ChildNode = (usize, usize);
        fn registered_window(&mut self, window: usize) -> Option<usize> { (window < self.children.len()).then_some(window) }
        fn focus_mode(&mut self, window: usize) -> bool { self.events.push((1, window)); self.modes & (1 << window) != 0 }
        fn visible(&mut self, window: usize) -> bool {
            self.events.push((2, window));
            if self.mutate && window == 1 { self.children[0].push(4); }
            self.visible & (1 << window) != 0
        }
        fn inherits(&mut self, window: usize, class: u32) -> bool {
            assert_eq!(class, FOCUS_CLASS);
            self.events.push((3, window));
            self.classes & (1 << window) != 0
        }
        fn registered_parent(&mut self, window: usize) -> Option<usize> { self.children.iter().position(|children| children.contains(&window)) }
        fn set_focus(&mut self, window: usize) { self.events.push((4, window)); }
        fn first_child(&mut self, window: usize) -> Option<(usize, usize)> { (!self.children[window].is_empty()).then_some((window, 0)) }
        fn child_window(&mut self, node: (usize, usize)) -> usize { self.children[node.0][node.1] }
        fn next_child(&mut self, node: (usize, usize)) -> Option<(usize, usize)> { (node.1 + 1 < self.children[node.0].len()).then_some((node.0, node.1 + 1)) }
    }

    fn backend() -> Backend {
        Backend { children: vec![vec![1, 2], vec![3], vec![], vec![], vec![]], classes: 0, modes: 0, visible: 0, mutate: false, events: Vec::new() }
    }

    #[test]
    fn focus_search_descends_through_hidden_nodes_and_checks_mode_before_visibility() {
        let mut b = backend();
        b.modes = 0b1011;
        b.visible = 0b1000;
        assert_eq!(focus_descendant(&mut b, 0), Ok(true));
        assert_eq!(b.events, [(1, 0), (2, 0), (1, 1), (2, 1), (1, 3), (2, 3), (4, 3)]);
    }

    #[test]
    fn direct_search_excludes_self_and_grandchildren_recursive_search_focuses_parent() {
        let mut b = backend();
        b.classes = 1 << 3;
        assert_eq!(has_direct_focus_class(&mut b, 0), Ok(false));
        assert_eq!(b.events, [(3, 1), (3, 2)]);
        b.events.clear();
        assert_eq!(focus_parent_of_class(&mut b, 0), Ok(true));
        assert_eq!(b.events, [(3, 0), (3, 1), (3, 3), (4, 1)]);
        b.events.clear();
        b.classes = 1;
        assert_eq!(focus_parent_of_class(&mut b, 0), Ok(true));
        assert_eq!(b.events, [(3, 0)]);
    }

    #[test]
    fn next_node_is_read_after_recursive_callbacks_and_missing_record_is_an_error() {
        let mut b = backend();
        b.mutate = true;
        b.modes = (1 << 1) | (1 << 4);
        b.visible = 1 << 4;
        assert_eq!(focus_descendant(&mut b, 0), Ok(true));
        assert_eq!(b.events, [(1, 0), (1, 1), (2, 1), (1, 3), (1, 2), (1, 4), (2, 4), (4, 4)]);
        assert_eq!(focus_descendant(&mut b, 9), Err(MissingRegisteredFocusWindow));
    }

    #[test]
    #[ignore]
    fn three_searches_match_native_dfs_callbacks_and_class_inheritance() {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut cases = Vec::new();
        for graph in 0..4 {
            for modes in 0..32 {
                for visible in 0..32 {
                    for mutation in 0..2 { cases.push([0, graph, modes, visible, 0, mutation, 0]); }
                }
            }
            for operation in 1..3 {
                for classes in 0..32 {
                    for depth in 0..2 { cases.push([operation, graph, 0, 0, classes, 0, depth]); }
                }
            }
        }
        let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
            .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_focus_search_reference.py"))
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
            let mut b = backend();
            b.children = match v[1] {
                0 => vec![vec![1, 2], vec![3], vec![], vec![], vec![]],
                1 => vec![vec![1], vec![2], vec![3], vec![], vec![]],
                2 => vec![vec![]; 5],
                _ => vec![vec![1, 2, 3], vec![], vec![], vec![], vec![]],
            };
            b.modes = v[2]; b.visible = v[3]; b.classes = v[4]; b.mutate = v[5] != 0;
            let found = match v[0] {
                0 => focus_descendant(&mut b, 0),
                1 => has_direct_focus_class(&mut b, 0),
                _ => focus_parent_of_class(&mut b, 0),
            }.unwrap();
            let expected: Vec<_> = [u32::from(found)].into_iter().chain(b.events.into_iter().flat_map(|(event, window)| [event, window as u32])).collect();
            let actual: Vec<u32> = line.split_whitespace().map(|value| value.parse().unwrap()).collect();
            assert_eq!(actual, expected, "case={v:?}");
        }
    }
}
