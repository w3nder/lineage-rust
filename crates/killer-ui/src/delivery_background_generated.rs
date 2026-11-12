use crate::background_contract::{BackgroundContract, ClipProof};

pub const BACKGROUND: BackgroundContract = BackgroundContract {
    rect: [0, 20, 256, 381],
    uv_bits: [0, 0, 1132462080, 1136558080],
    texture: "L2UI_CH3.StoreWnd.Store1_back",
    alpha: 255,
    clip: 1,
    clip_proof: ClipProof::Contained([0, 0, 256, 401]),
    on_paint: 268980144,
    operation_index: 1,
    helper_return: 268513260,
};
