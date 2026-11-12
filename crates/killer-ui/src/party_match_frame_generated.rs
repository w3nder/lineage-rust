use crate::window::Rect;

use crate::frame::FrameMode;
use crate::frame_contract::{FrameContract, FrameTitle};

pub const FRAME_114: FrameContract = FrameContract {
    rect: Rect::new(0, 0, 600, 20),
    id: 0xf7,
    mode: FrameMode::Title,
    title: FrameTitle::Sysstring(389),
    close_button: true,
    minimize_button: true,
};
