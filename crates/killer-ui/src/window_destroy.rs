use crate::window::WindowId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnedWindow {
    Frame,
    Minimized,
}

pub trait WindowDestroyBackend {
    type Auxiliary: Copy;
    fn owned_window(&mut self, kind: OwnedWindow) -> Option<WindowId>;
    fn unregister_owned(&mut self, window: WindowId);
    fn delete_window(&mut self, window: WindowId, mode: u32);
    fn clear_owned(&mut self, kind: OwnedWindow);
    fn auxiliary(&mut self) -> Option<Self::Auxiliary>;
    fn destroy_auxiliary(&mut self, auxiliary: Self::Auxiliary);
    fn free_auxiliary(&mut self, auxiliary: Self::Auxiliary);
    fn clear_auxiliary(&mut self);
}

fn destroy_owned(backend: &mut impl WindowDestroyBackend, kind: OwnedWindow) {
    if let Some(window) = backend.owned_window(kind) {
        backend.unregister_owned(window);
        if let Some(window) = backend.owned_window(kind) {
            backend.delete_window(window, 1);
        }
        backend.clear_owned(kind);
    }
}

pub fn destroy_window(backend: &mut impl WindowDestroyBackend) -> bool {
    destroy_owned(backend, OwnedWindow::Frame);
    if let Some(auxiliary) = backend.auxiliary() {
        backend.destroy_auxiliary(auxiliary);
        backend.free_auxiliary(auxiliary);
        backend.clear_auxiliary();
    }
    destroy_owned(backend, OwnedWindow::Minimized);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Backend {
        windows: [Option<WindowId>; 2],
        auxiliary: Option<u32>,
        mutation: u8,
        calls: Vec<String>,
    }

    impl WindowDestroyBackend for Backend {
        type Auxiliary = u32;
        fn owned_window(&mut self, kind: OwnedWindow) -> Option<WindowId> {
            self.windows[kind as usize]
        }
        fn unregister_owned(&mut self, window: WindowId) {
            self.calls.push(format!("unregister {}", window.0));
            let index = if window.0 == 1 { 0 } else { 1 };
            match self.mutation {
                1 => self.windows[index] = None,
                2 => self.windows[index] = Some(WindowId(window.0 + 10)),
                _ => {}
            }
        }
        fn delete_window(&mut self, window: WindowId, mode: u32) {
            assert_eq!(mode, 1);
            self.calls.push(format!("delete {}", window.0));
        }
        fn clear_owned(&mut self, kind: OwnedWindow) {
            self.calls.push(format!("clear {kind:?}"));
            self.windows[kind as usize] = None;
        }
        fn auxiliary(&mut self) -> Option<u32> {
            self.auxiliary
        }
        fn destroy_auxiliary(&mut self, auxiliary: u32) {
            assert_eq!(auxiliary, 3);
            self.calls.push("destroy auxiliary".into());
            self.auxiliary = Some(99);
        }
        fn free_auxiliary(&mut self, auxiliary: u32) {
            assert_eq!(auxiliary, 3);
            self.calls.push("free auxiliary".into());
        }
        fn clear_auxiliary(&mut self) {
            self.calls.push("clear auxiliary".into());
            self.auxiliary = None;
        }
    }

    #[test]
    fn destruction_orders_all_presence_combinations_and_reloads_owned_windows() {
        for mask in 0..8 {
            for mutation in 0..3 {
                let mut b = Backend {
                    windows: [(mask & 1 != 0).then_some(WindowId(1)), (mask & 4 != 0).then_some(WindowId(2))],
                    auxiliary: (mask & 2 != 0).then_some(3),
                    mutation,
                    calls: Vec::new(),
                };
                assert!(destroy_window(&mut b));
                let mut expected = Vec::new();
                for (bit, id, name) in [(1, 1, "Frame"), (2, 3, "auxiliary"), (4, 2, "Minimized")] {
                    if mask & bit == 0 {
                        continue;
                    }
                    if bit == 2 {
                        expected.extend([
                            "destroy auxiliary".into(),
                            "free auxiliary".into(),
                            "clear auxiliary".into(),
                        ]);
                    } else {
                        expected.push(format!("unregister {id}"));
                        if mutation != 1 {
                            expected.push(format!("delete {}", if mutation == 2 { id + 10 } else { id }));
                        }
                        expected.push(format!("clear {name}"));
                    }
                }
                assert_eq!(b.calls, expected);
                assert_eq!(b.windows, [None, None]);
                assert_eq!(b.auxiliary, None);
            }
        }
    }
}
