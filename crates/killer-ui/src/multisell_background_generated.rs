use crate::background_contract::{BackgroundContract, ClipProof};

pub const BACKGROUND: BackgroundContract = BackgroundContract {
    rect: [0, 20, 512, 381],
    uv_bits: [0, 0, 1140850688, 1136558080],
    texture: "L2UI_CH3.MultiSellWnd.multicell_back",
    alpha: 255,
    clip: 1,
    clip_proof: ClipProof::Contained([0, 0, 512, 401]),
    on_paint: 269099360,
    operation_index: 1,
    helper_return: 268513260,
};
