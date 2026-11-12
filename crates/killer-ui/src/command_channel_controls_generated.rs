use crate::window::Rect;

use crate::child_contract::{ChildContract, ChildKind};

pub const CHILD_FC: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(18, 166, 15, 15),
    id: 0xf0a0c3,
    vtable: 0x101a0ce8,
    registration: (0x22, 0x0, 0x0),
    handles: &[0x274, 0x278],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0xffffffff), (0x2a4, 0xf), (0x2a8, 0xf)],
    textures: &[(0x274, "L2UI_ch3.PartyCommandWnd.Herochat_btn1"), (0x278, "L2UI_ch3.PartyCommandWnd.Herochat_btn2")],
};
