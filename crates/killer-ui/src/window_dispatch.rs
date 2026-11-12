#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowDispatch {
    pub focus: u32,
    pub focus_mode: u32,
    pub movement: u32,
    pub resize: u32,
    pub show: u32,
    pub hide: u32,
    pub move_dispatch: u32,
}

include!("window_dispatch_generated.rs");

impl WindowDispatch {
    pub fn from_vtable(vtable: u32) -> Option<Self> {
        WINDOW_DISPATCH
            .binary_search_by_key(&vtable, |(address, _)| *address)
            .ok()
            .map(|index| WINDOW_DISPATCH[index].1)
    }

    pub fn methods(self) -> [u32; 7] {
        [self.focus, self.focus_mode, self.movement, self.resize, self.show, self.hide, self.move_dispatch]
    }
}

