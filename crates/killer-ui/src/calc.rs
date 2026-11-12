use crate::draw::DrawList;
use crate::window::Rect;

pub const SIZE: (i32, i32) = (256, 211);
pub const CALCULATE2_BACK: &str = "l2ui_ch3.calculate.calculate2_back";
pub const FRAME_CTRL_7C_RECT: Rect = Rect::new(0, 0, 256, 20);
pub const FRAME_CTRL_7C_TEXTURES: [&str; 9] = ["L2UI_CH3.FrameCtrl.FrameBackLeft", "L2UI_CH3.FrameCtrl.FrameBackMid", "L2UI_CH3.FrameCtrl.FrameBackRight", "L2UI_CH3.FrameCtrl.smallbar_h1", "L2UI_CH3.FrameCtrl.smallbar_h2", "L2UI_CH3.FrameCtrl.smallbar_h3", "L2UI_CH3.FrameCtrl.smallbar1", "L2UI_CH3.FrameCtrl.smallbar2", "L2UI_CH3.FrameCtrl.smallbar3"];
pub const CALC_EDIT_CTRL_EC_RECT: Rect = Rect::new(34, 40, 188, 17);
pub const CALC_EDIT_CTRL_EC_ID: u32 = 0xff01ff02;
pub const BUTTON_F0S: [(Rect, u32, [&str; 3]); 19] = [
    (Rect::new(35, 66, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_7", "L2UI_CH3.Calculate.Calculate2_7_down"]),
    (Rect::new(74, 66, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_8", "L2UI_CH3.Calculate.Calculate2_8_down"]),
    (Rect::new(113, 66, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_9", "L2UI_CH3.Calculate.Calculate2_9_down"]),
    (Rect::new(152, 66, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_add", "L2UI_CH3.Calculate.Calculate2_add_down"]),
    (Rect::new(191, 66, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_ce", "L2UI_CH3.Calculate.Calculate2_ce_down"]),
    (Rect::new(35, 93, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_4", "L2UI_CH3.Calculate.Calculate2_4_down"]),
    (Rect::new(74, 93, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_5", "L2UI_CH3.Calculate.Calculate2_5_down"]),
    (Rect::new(113, 93, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_6", "L2UI_CH3.Calculate.Calculate2_6_down"]),
    (Rect::new(152, 93, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_sub", "L2UI_CH3.Calculate.Calculate2_sub_down"]),
    (Rect::new(191, 93, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_c", "L2UI_CH3.Calculate.Calculate2_c_down"]),
    (Rect::new(35, 120, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_1", "L2UI_CH3.Calculate.Calculate2_1_down"]),
    (Rect::new(74, 120, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_2", "L2UI_CH3.Calculate.Calculate2_2_down"]),
    (Rect::new(113, 120, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_3", "L2UI_CH3.Calculate.Calculate2_3_down"]),
    (Rect::new(152, 120, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_mul", "L2UI_CH3.Calculate.Calculate2_mul_down"]),
    (Rect::new(191, 120, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_bs", "L2UI_CH3.Calculate.Calculate2_bs_down"]),
    (Rect::new(35, 147, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_0", "L2UI_CH3.Calculate.Calculate2_0_down"]),
    (Rect::new(74, 147, 73, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_00", "L2UI_CH3.Calculate.Calculate2_00_down"]),
    (Rect::new(151, 147, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_div", "L2UI_CH3.Calculate.Calculate2_div_down"]),
    (Rect::new(190, 147, 35, 24), 0xff01ff01, ["L2UI.tooltip.mini-tool01", "L2UI_CH3.Calculate.Calculate2_eq", "L2UI_CH3.Calculate.Calculate2_eq_down"]),
];
pub const BUTTON_13C_RECT: Rect = Rect::new(91, 182, 76, 23);
pub const BUTTON_13C_ID: u32 = 0xff01ff00;
pub const BUTTON_13C_TEXTURES: [&str; 3] = ["L2UI.tooltip.mini-tool01", "L2UI_ch3.button.btn1_normal", "L2UI_ch3.button.btn1_normalon"];
pub const BUTTON_13C_SYSSTRING: u32 = 646;

pub fn paint_calc(list: &mut DrawList, x: i32, y: i32) {
    list.draw_texture(x, y + 20, 256, 191, 0, 0, 256, 191, CALCULATE2_BACK, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_follows_the_original_on_paint_1011f2f0() {
        let mut list = DrawList::default();
        paint_calc(&mut list, 100, 50);
        let quads: Vec<_> = list.quads.iter().map(|q| (q.x, q.y, q.w, q.h, q.u, q.v, q.uw, q.vh, q.texture.as_str())).collect();
        assert_eq!(quads, [
            (100.0, 70.0, 256.0, 191.0, 0.0, 0.0, 256.0, 191.0, "l2ui_ch3.calculate.calculate2_back"),
        ]);
    }
}
