use crate::window::Rect;

use crate::frame::FrameMode;
use crate::frame_contract::{FrameContract, FrameTitle};

pub const FRAME_7C: FrameContract = FrameContract {
    rect: Rect::new(0, 0, 12, 46),
    id: 0xafddcea0,
    mode: FrameMode::VerticalGrip,
    title: FrameTitle::None,
    close_button: false,
    minimize_button: false,
};
