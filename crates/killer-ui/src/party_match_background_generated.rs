use crate::background_contract::{BackgroundContract, ClipProof};

pub const MATCH_BACK: BackgroundContract = BackgroundContract {
    rect: [0, 20, 600, 411],
    uv_bits: [0, 0, 1142292480, 1137541120],
    texture: "L2UI_CH3.PartyMatchWnd.PartyMatch1_Back",
    alpha: 255,
    clip: 1,
    clip_proof: ClipProof::Contained([0, 0, 600, 431]),
    on_paint: 269165488,
    operation_index: 1,
    helper_return: 268513260,
};
