use crate::window::Rect;

use crate::child_contract::{ChildContract, ChildKind};

pub const CHILD_3C: ChildContract = ChildContract {
    kind: ChildKind::Icon,
    rect: Rect::new(0, 0, 32, 32),
    id: 0xf,
    vtable: 0x101a62d0,
    registration: (0x81022, 0x0, 0x30000000),
    handles: &[0x218],
    state: &[(0x210, 0xffffffff)],
    textures: &[(0x218, "NWindow.ChatBack")],
};

pub const CHILD_EC: ChildContract = ChildContract {
    kind: ChildKind::ComboBox,
    rect: Rect::new(84, 52, 90, 17),
    id: 0xf8,
    vtable: 0x101a1120,
    registration: (0x12, 0x0, 0x0),
    handles: &[],
    state: &[(0x1f4, 0x5), (0x240, 0x1), (0x244, 0xffffffff), (0x248, 0x11)],
    textures: &[],
};

pub const CHILD_F0: ChildContract = ChildContract {
    kind: ChildKind::ComboBox,
    rect: Rect::new(264, 52, 90, 17),
    id: 0xf9,
    vtable: 0x101a1120,
    registration: (0x12, 0x0, 0x0),
    handles: &[],
    state: &[(0x1f4, 0x5), (0x240, 0x1), (0x244, 0xffffffff), (0x248, 0x11)],
    textures: &[],
};
