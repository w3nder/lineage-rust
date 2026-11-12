use crate::window::Rect;

use crate::child_contract::{ChildContract, ChildKind};

pub const CHILD_EC: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(35, 55, 112, 14),
    id: 0x1505,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[0x230],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x80), (0x2b8, 0x1)],
    textures: &[(0x230, "NWindow.ChatBack")],
};
