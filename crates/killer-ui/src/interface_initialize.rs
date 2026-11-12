#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootWindowSlot(pub u32);

pub const MINIMIZED_RESTORE_ORDER: [RootWindowSlot; 9] = [
    RootWindowSlot(0xe8),
    RootWindowSlot(0x164),
    RootWindowSlot(0x16c),
    RootWindowSlot(0x148),
    RootWindowSlot(0x174),
    RootWindowSlot(0x170),
    RootWindowSlot(0x188),
    RootWindowSlot(0x18c),
    RootWindowSlot(0x120),
];

pub trait InterfaceInitializeBackend {
    type ChildNode: Copy;
    fn has_root(&mut self) -> bool;
    fn minimized_visible(&mut self, slot: RootWindowSlot) -> bool;
    fn restore_minimized(&mut self, slot: RootWindowSlot);
    fn has_secondary_root(&mut self) -> bool;
    fn layout_secondary_root(&mut self);
    fn resize_root_to_screen(&mut self);
    fn reset_shortcut_orientation(&mut self);
    fn rearrange_abnormal_status(&mut self);
    fn request_width(&mut self, slot: RootWindowSlot, width: i32);
    fn first_child(&mut self) -> Option<Self::ChildNode>;
    fn child_is_chat(&mut self, node: Self::ChildNode) -> bool;
    fn restore_initial_position(&mut self, node: Self::ChildNode);
    fn next_child(&mut self, node: Self::ChildNode) -> Option<Self::ChildNode>;
    fn reset_chat_geometry(&mut self, height: i32);
    fn restore_chat_tabs(&mut self);
    fn reposition_notifications(&mut self);
}

pub fn initialize_interface(backend: &mut impl InterfaceInitializeBackend) {
    if !backend.has_root() {
        return;
    }
    for slot in MINIMIZED_RESTORE_ORDER {
        if backend.minimized_visible(slot) {
            backend.restore_minimized(slot);
        }
    }
    if backend.has_secondary_root() {
        backend.layout_secondary_root();
    }
    backend.resize_root_to_screen();
    backend.reset_shortcut_orientation();
    backend.rearrange_abnormal_status();
    backend.request_width(RootWindowSlot(0x128), 172);
    backend.request_width(RootWindowSlot(0x124), 172);
    let mut node = backend.first_child();
    while let Some(child) = node {
        if !backend.child_is_chat(child) {
            backend.restore_initial_position(child);
        }
        node = backend.next_child(child);
    }
    backend.reset_chat_geometry(187);
    backend.restore_chat_tabs();
    backend.reposition_notifications();
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Backend {
        root: bool,
        secondary: bool,
        mask: u16,
        nodes: usize,
        append_on_restore: bool,
        calls: Vec<String>,
    }

    impl InterfaceInitializeBackend for Backend {
        type ChildNode = usize;
        fn has_root(&mut self) -> bool {
            self.root
        }
        fn minimized_visible(&mut self, slot: RootWindowSlot) -> bool {
            self.calls.push(format!("visible:{}", slot.0));
            let index = MINIMIZED_RESTORE_ORDER.iter().position(|value| *value == slot).unwrap();
            self.mask & (1 << index) != 0
        }
        fn restore_minimized(&mut self, slot: RootWindowSlot) {
            self.calls.push(format!("restore:{}", slot.0));
        }
        fn has_secondary_root(&mut self) -> bool {
            self.secondary
        }
        fn layout_secondary_root(&mut self) {
            self.calls.push("secondary".into());
        }
        fn resize_root_to_screen(&mut self) {
            self.calls.push("root:size".into());
        }
        fn reset_shortcut_orientation(&mut self) {
            self.calls.push("shortcut:0".into());
        }
        fn rearrange_abnormal_status(&mut self) {
            self.calls.push("abnormal".into());
        }
        fn request_width(&mut self, slot: RootWindowSlot, width: i32) {
            self.calls.push(format!("width:{}:{width}", slot.0));
        }
        fn first_child(&mut self) -> Option<usize> {
            (self.nodes > 0).then_some(0)
        }
        fn child_is_chat(&mut self, node: usize) -> bool {
            node == 1 || node == 2
        }
        fn restore_initial_position(&mut self, node: usize) {
            self.calls.push(format!("child:{node}"));
            if self.append_on_restore {
                self.nodes += 1;
                self.append_on_restore = false;
            }
        }
        fn next_child(&mut self, node: usize) -> Option<usize> {
            (node + 1 < self.nodes).then_some(node + 1)
        }
        fn reset_chat_geometry(&mut self, height: i32) {
            self.calls.push(format!("chat:{height}"));
        }
        fn restore_chat_tabs(&mut self) {
            self.calls.push("tabs".into());
        }
        fn reposition_notifications(&mut self) {
            self.calls.push("notifications".into());
        }
    }

    #[test]
    fn initialize_10073b70_checks_all_minimized_windows_and_preserves_effect_order() {
        for mask in 0..512 {
            for secondary in [false, true] {
                let mut backend =
                    Backend { root: true, secondary, mask, nodes: 4, append_on_restore: false, calls: Vec::new() };
                initialize_interface(&mut backend);
                let mut expected = Vec::new();
                for (index, slot) in MINIMIZED_RESTORE_ORDER.iter().enumerate() {
                    expected.push(format!("visible:{}", slot.0));
                    if mask & (1 << index) != 0 {
                        expected.push(format!("restore:{}", slot.0));
                    }
                }
                if secondary {
                    expected.push("secondary".into());
                }
                expected.extend(
                    [
                        "root:size",
                        "shortcut:0",
                        "abnormal",
                        "width:296:172",
                        "width:292:172",
                        "child:0",
                        "child:3",
                        "chat:187",
                        "tabs",
                        "notifications",
                    ]
                    .map(str::to_owned),
                );
                assert_eq!(backend.calls, expected);
            }
        }
    }

    #[test]
    fn initialize_10073b70_missing_root_has_no_effect_and_empty_children_still_resets_chat() {
        let mut backend =
            Backend { root: false, secondary: true, mask: 511, nodes: 4, append_on_restore: false, calls: Vec::new() };
        initialize_interface(&mut backend);
        assert!(backend.calls.is_empty());
        backend.root = true;
        backend.nodes = 0;
        initialize_interface(&mut backend);
        assert!(!backend.calls.iter().any(|call| call.starts_with("child:")));
        assert_eq!(&backend.calls[backend.calls.len() - 3..], ["chat:187", "tabs", "notifications"]);
    }

    #[test]
    fn initialize_10073e53_reads_next_child_after_restore_changes_the_list() {
        let mut backend =
            Backend { root: true, secondary: false, mask: 0, nodes: 4, append_on_restore: true, calls: Vec::new() };
        initialize_interface(&mut backend);
        let children: Vec<_> =
            backend.calls.iter().filter(|call| call.starts_with("child:")).map(String::as_str).collect();
        assert_eq!(children, ["child:0", "child:3", "child:4"]);
        assert_eq!(&backend.calls[backend.calls.len() - 3..], ["chat:187", "tabs", "notifications"]);
    }
}
