use crate::window::Rect;

use crate::frame::FrameMode;
use crate::frame_contract::{FrameContract, FrameTitle};

pub const FRAME_7C: FrameContract = FrameContract {
    rect: Rect::new(0, 0, 256, 20),
    id: 0x4,
    mode: FrameMode::Title,
    title: FrameTitle::Sysstring(477),
    close_button: true,
    minimize_button: false,
};
