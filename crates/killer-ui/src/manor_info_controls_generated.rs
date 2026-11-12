use crate::window::Rect;

use crate::frame::FrameMode;
use crate::frame_contract::{FrameContract, FrameTitle};

pub const FRAME_7C: FrameContract = FrameContract {
    rect: Rect::new(0, 0, 600, 20),
    id: 0xfff00001,
    mode: FrameMode::Title,
    title: FrameTitle::Sysstring(738),
    close_button: true,
    minimize_button: false,
};
