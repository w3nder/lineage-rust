use crate::abnormal::SKILL_ICON_TEXELS;
use crate::color::TEXT;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::progress::ProgressCtrl;
use crate::status::{ellipsize, fill_width};
use crate::target::{
    BODY_END_WIDTH, BODY_RIGHT_INSET, BODY_RIGHT_WIDTH, BODY_UW, BODY_X, SMALLWINDOW_BACK1, SMALLWINDOW_BACK2,
    SMALLWINDOW_BACK3_EX,
};
use crate::window::Rect;

pub const PET_STATUS_SIZE: (i32, i32) = (172, 46);
pub const PET_STATUS_DEFAULT_POS: (i32, i32) = (0, 76);
pub const PET_STATUS_INITIAL_POSITION: crate::initial_position::InitialPosition = crate::initial_position::InitialPosition {
    enabled: true, horizontal_mode: 0, vertical_mode: 3, x: PET_STATUS_DEFAULT_POS.0, y: PET_STATUS_DEFAULT_POS.1,
};
pub const HP_BAR: &str = "L2UI_CH3.SmallWnd.HPBAR";
pub const HP_BAR_BACK: &str = "L2UI_CH3.SmallWnd.HPBAR_BACK";
pub const MP_BAR: &str = "L2UI_CH3.SmallWnd.MPBAR";
pub const MP_BAR_BACK: &str = "L2UI_CH3.SmallWnd.MPBAR_BACK";
pub const FATIGUE_BAR: &str = "L2UI_CH3.SmallWnd.FATIGUEBAR";
pub const FATIGUE_BAR_BACK: &str = "L2UI_CH3.SmallWnd.FATIGUEBAR_BACK";
pub const PET_BUTTON: &str = "L2UI_CH3.Etc.pet_button";
pub const PET_BUTTON_DOWN: &str = "L2UI_CH3.Etc.pet_button_down";
pub const PET_BUTTON_TOOLTIP_SYSSTRING: u32 = 881;
pub const PET_BUTTON_RECT: Rect = Rect::new(1, 6, 11, 13);
pub const BODY_MID_INSET: i32 = 44;
pub const BAR_X: i32 = 16;
pub const BAR_RIGHT_INSET: i32 = 22;
pub const BAR_BACK_HEIGHT: i32 = 5;
pub const BAR_FILL_HEIGHT: i32 = 6;
pub const SUMMON_FILL_HEIGHT: i32 = 5;
pub const BAR_UW: i32 = 8;
pub const BAR_VH: i32 = 6;
pub const HP_BAR_Y: i32 = 19;
pub const MP_BAR_Y: i32 = 26;
pub const THIRD_BAR_Y: i32 = 33;
pub const SUMMON_TIME_BAR: ProgressCtrl = ProgressCtrl { back: FATIGUE_BAR_BACK, fill: FATIGUE_BAR, inset: 0 };
pub const NAME_RECT: Rect = Rect::new(12, 6, 160, 14);
pub const BUFF_ICON: i32 = 16;
pub const BUFF_ROW_YS: [i32; 2] = [5, 21];
pub const BUFFS_PER_ROW: usize = 10;
pub const MAX_BUFF_ICONS: usize = 20;
pub const SELECT_X: i32 = 14;
pub const GRIP_Y: i32 = 17;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThirdBar {
    Fed { fed: i32, max_fed: i32 },
    Time { remaining: i32, max: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PetStatusView {
    pub name: String,
    pub hp: (i32, i32),
    pub mp: (i32, i32),
    pub third: ThirdBar,
    pub buffs: Vec<Option<String>>,
    pub show_buffs: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetHit {
    Grip,
    BuffButton,
    Select,
}

pub fn grip() -> Rect {
    let full = FrameCtrl::new(FrameMode::VerticalGrip).grip(PET_STATUS_SIZE.0, PET_STATUS_SIZE.1);
    Rect::new(full.x, GRIP_Y, full.w, full.h - GRIP_Y)
}

pub const BUFF_BUTTON_TOOLTIP_SYSSTRING: u32 = 881;

pub fn hit(x: i32, y: i32) -> Option<PetHit> {
    hit_sized(x, y, PET_STATUS_SIZE.0)
}

pub fn hit_sized(x: i32, y: i32, w: i32) -> Option<PetHit> {
    let h = PET_STATUS_SIZE.1;
    if PET_BUTTON_RECT.contains(x, y) {
        Some(PetHit::BuffButton)
    } else if x >= SELECT_X && x < w && y >= 0 && y < h {
        Some(PetHit::Select)
    } else if grip().contains(x, y) {
        Some(PetHit::Grip)
    } else {
        None
    }
}

pub fn buff_icon_pos(index: usize) -> Option<(i32, i32)> {
    buff_icon_pos_sized(index, PET_STATUS_SIZE.0)
}

pub fn buff_icon_pos_sized(index: usize, width: i32) -> Option<(i32, i32)> {
    if index >= MAX_BUFF_ICONS {
        return None;
    }
    let (row, col) = (index / BUFFS_PER_ROW, index % BUFFS_PER_ROW);
    Some((width + 1 + col as i32 * BUFF_ICON, BUFF_ROW_YS[row]))
}

pub fn paint_pet_status(list: &mut DrawList, x: i32, y: i32, view: &PetStatusView, font: &BitmapFont, alpha: f32) {
    paint_pet_status_sized(list, x, y, PET_STATUS_SIZE.0, view, font, alpha);
}

#[allow(clippy::too_many_arguments)]
pub fn paint_pet_status_sized(list: &mut DrawList, x: i32, y: i32, w: i32, view: &PetStatusView, font: &BitmapFont, alpha: f32) {
    let h = PET_STATUS_SIZE.1;
    FrameCtrl::new(FrameMode::VerticalGrip).paint(list, x, y, w, h, font, alpha);
    list.draw_texture(x + BODY_X, y, BODY_END_WIDTH, h, 0, 0, BODY_UW, h, SMALLWINDOW_BACK1, alpha);
    list.draw_texture(
        x + BODY_X + BODY_END_WIDTH,
        y,
        w - BODY_MID_INSET,
        h,
        0,
        0,
        BODY_UW,
        h,
        SMALLWINDOW_BACK2,
        alpha,
    );
    list.draw_texture(
        x + w - BODY_RIGHT_INSET,
        y,
        BODY_RIGHT_WIDTH,
        h,
        0,
        0,
        BODY_RIGHT_WIDTH,
        h,
        SMALLWINDOW_BACK3_EX,
        alpha,
    );
    let button = if view.show_buffs { PET_BUTTON_DOWN } else { PET_BUTTON };
    let Rect { x: bx, y: by, w: bw, h: bh } = PET_BUTTON_RECT;
    list.draw_texture(x + bx, y + by, bw, bh, 0, 0, bw, bh, button, 1.0);

    let bar_w = w - BAR_RIGHT_INSET;
    if !view.name.is_empty() {
        let name = ellipsize(&view.name, NAME_RECT.w, font);
        let text_x = x + NAME_RECT.x + NAME_RECT.w / 2 - font.measure(&name) as i32 / 2;
        list.draw_text(text_x, y + NAME_RECT.y, TEXT, &name, font);
    }
    let fill_height = if matches!(view.third, ThirdBar::Time { .. }) { SUMMON_FILL_HEIGHT } else { BAR_FILL_HEIGHT };
    paint_bar(list, x + BAR_X, y + HP_BAR_Y, bar_w, HP_BAR_BACK, HP_BAR, fill_width(view.hp, bar_w), fill_height);
    paint_bar(list, x + BAR_X, y + MP_BAR_Y, bar_w, MP_BAR_BACK, MP_BAR, fill_width(view.mp, bar_w), fill_height);
    match view.third {
        ThirdBar::Fed { fed, max_fed } => {
            paint_bar(
                list,
                x + BAR_X,
                y + THIRD_BAR_Y,
                bar_w,
                FATIGUE_BAR_BACK,
                FATIGUE_BAR,
                fill_width((fed, max_fed), bar_w),
                BAR_FILL_HEIGHT,
            );
        }
        ThirdBar::Time { remaining, max } => {
            SUMMON_TIME_BAR.paint(list, x + BAR_X, y + THIRD_BAR_Y, bar_w, BAR_VH, max - remaining, max);
        }
    }
    if view.show_buffs {
        for (index, icon) in view.buffs.iter().enumerate().take(MAX_BUFF_ICONS) {
            let (Some(icon), Some((ix, iy))) = (icon, buff_icon_pos_sized(index, w)) else { continue };
            list.draw_texture(
                x + ix,
                y + iy,
                BUFF_ICON,
                BUFF_ICON,
                0,
                0,
                SKILL_ICON_TEXELS,
                SKILL_ICON_TEXELS,
                icon,
                1.0,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_bar(list: &mut DrawList, x: i32, y: i32, w: i32, back: &str, bar: &str, fill: i32, fill_height: i32) {
    list.draw_texture(x, y, w, BAR_BACK_HEIGHT, 0, 0, BAR_UW, BAR_VH, back, 1.0);
    if fill > 0 {
        list.draw_texture(x, y, fill, fill_height, 0, 0, BAR_UW, BAR_VH, bar, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Quad;
    use crate::font::{Glyphs, gly};

    const TEXTURES: &str = include_str!("../tests/fixtures/nwindow-texturas-pet.txt");

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn pet() -> PetStatusView {
        PetStatusView {
            name: "Wolf".into(),
            hp: (50, 100),
            mp: (30, 60),
            third: ThirdBar::Fed { fed: 25, max_fed: 100 },
            buffs: vec![Some("Icon.skill1204".into()), None, Some("Icon.skill1040".into())],
            show_buffs: false,
        }
    }

    fn painted(view: &PetStatusView) -> Vec<Quad> {
        let mut list = DrawList::default();
        paint_pet_status(&mut list, 0, 0, view, &font(), 1.0);
        list.quads
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    #[test]
    fn body_is_the_target_window_body_with_the_pet_button_at_one_six() {
        let quads = painted(&pet());
        let grip = with_texture(&quads, crate::frame::SMALLBAR_2);
        assert_eq!((grip[0].x, grip[0].y, grip[0].w, grip[0].h), (0.0, 8.0, 12.0, 30.0));
        let back1 = with_texture(&quads, SMALLWINDOW_BACK1);
        assert_eq!((back1[0].x, back1[0].w, back1[0].h, back1[0].uw, back1[0].vh), (12.0, 16.0, 46.0, 16.0, 46.0));
        let back2 = with_texture(&quads, SMALLWINDOW_BACK2);
        assert_eq!((back2[0].x, back2[0].w, back2[0].uw), (28.0, 128.0, 16.0));
        let back3 = with_texture(&quads, SMALLWINDOW_BACK3_EX);
        assert_eq!((back3[0].x, back3[0].w, back3[0].uw), (156.0, 19.0, 19.0));
        let button = with_texture(&quads, PET_BUTTON);
        assert_eq!((button[0].x, button[0].y, button[0].w, button[0].h), (1.0, 6.0, 11.0, 13.0));
        assert!(with_texture(&quads, PET_BUTTON_DOWN).is_empty());
        assert_eq!(super::grip(), Rect::new(0, 17, 12, 29));
    }

    #[test]
    fn bars_are_150_wide_with_five_pixel_backs_and_six_pixel_fills_at_19_26_and_33() {
        let quads = painted(&pet());
        let hp_back = with_texture(&quads, HP_BAR_BACK);
        assert_eq!(
            (hp_back[0].x, hp_back[0].y, hp_back[0].w, hp_back[0].h, hp_back[0].uw, hp_back[0].vh),
            (16.0, 19.0, 150.0, 5.0, 8.0, 6.0)
        );
        let hp = with_texture(&quads, HP_BAR);
        assert_eq!((hp[0].y, hp[0].w, hp[0].h), (19.0, 75.0, 6.0));
        assert_eq!((with_texture(&quads, MP_BAR_BACK)[0].y, with_texture(&quads, MP_BAR)[0].w), (26.0, 75.0));
        let fed_back = with_texture(&quads, FATIGUE_BAR_BACK);
        assert_eq!((fed_back[0].y, fed_back[0].h), (33.0, 5.0));
        let fed = with_texture(&quads, FATIGUE_BAR);
        assert_eq!((fed[0].y, fed[0].w, fed[0].h), (33.0, 37.0, 6.0));
        let empty = PetStatusView { hp: (0, 0), mp: (0, 0), third: ThirdBar::Fed { fed: 0, max_fed: 0 }, ..pet() };
        let quads = painted(&empty);
        assert!(with_texture(&quads, HP_BAR).is_empty() && with_texture(&quads, FATIGUE_BAR).is_empty());
        assert_eq!(with_texture(&quads, HP_BAR_BACK).len(), 1);
    }

    #[test]
    fn a_summon_shows_the_remaining_time_as_a_progress_control_without_inset() {
        let summon = PetStatusView { third: ThirdBar::Time { remaining: 900, max: 1200 }, ..pet() };
        let quads = painted(&summon);
        let back = with_texture(&quads, FATIGUE_BAR_BACK);
        assert_eq!(
            (back[0].x, back[0].y, back[0].w, back[0].h, back[0].uw, back[0].vh),
            (16.0, 33.0, 150.0, 6.0, 150.0, 6.0)
        );
        let fill = with_texture(&quads, FATIGUE_BAR);
        assert_eq!((fill[0].x, fill[0].y, fill[0].w, fill[0].h, fill[0].uw), (16.0, 33.0, 113.0, 6.0, 150.0));
        let hp = with_texture(&quads, HP_BAR);
        assert_eq!((hp[0].h, with_texture(&quads, MP_BAR)[0].h), (5.0, 5.0));
        let over = PetStatusView { third: ThirdBar::Time { remaining: 0, max: 1200 }, ..pet() };
        assert!(with_texture(&painted(&over), FATIGUE_BAR).is_empty());
    }

    #[test]
    fn the_name_is_centered_in_the_name_control_and_ellipsized() {
        let font = font();
        let quads = painted(&pet());
        let name: Vec<&Quad> = quads.iter().filter(|q| q.texture == font.atlas).collect();
        assert_eq!(name.len(), 4);
        assert_eq!((name[0].x, name[0].y, name[0].color), ((12 + 80 - 12) as f32, 6.0, TEXT.rgba()));
        let long = PetStatusView { name: "a".repeat(40), ..pet() };
        let quads = painted(&long);
        let name: Vec<&Quad> = quads.iter().filter(|q| q.texture == font.atlas).collect();
        assert!(name.iter().all(|q| q.x >= 12.0 && q.x + q.w <= 172.0));
        assert!(painted(&PetStatusView { name: String::new(), ..pet() }).iter().all(|q| q.texture != font.atlas));
    }

    #[test]
    fn expanded_buffs_go_right_of_the_window_in_two_rows_of_ten() {
        let mut view = pet();
        view.show_buffs = true;
        view.buffs = (0..25).map(|i| Some(format!("Icon.skill{i}"))).collect();
        let quads = painted(&view);
        assert_eq!(with_texture(&quads, PET_BUTTON_DOWN).len(), 1);
        let first = with_texture(&quads, "Icon.skill0");
        assert_eq!(
            (first[0].x, first[0].y, first[0].w, first[0].h, first[0].uw, first[0].vh),
            (173.0, 5.0, 16.0, 16.0, 32.0, 32.0)
        );
        assert_eq!(
            (with_texture(&quads, "Icon.skill9")[0].x, with_texture(&quads, "Icon.skill10")[0].x),
            (173.0 + 144.0, 173.0)
        );
        assert_eq!(with_texture(&quads, "Icon.skill10")[0].y, 21.0);
        assert!(with_texture(&quads, "Icon.skill20").is_empty());
        assert_eq!(buff_icon_pos(19), Some((173 + 144, 21)));
        assert_eq!(buff_icon_pos(20), None);
        assert!(painted(&pet()).iter().all(|q| !q.texture.starts_with("Icon.")));
    }

    #[test]
    fn clicks_split_into_grip_buff_button_and_select() {
        assert_eq!(hit(5, 20), Some(PetHit::Grip));
        assert_eq!(hit(5, 10), Some(PetHit::BuffButton));
        assert_eq!(hit(11, 18), Some(PetHit::BuffButton));
        assert_eq!(hit(5, 3), None);
        assert_eq!(hit(13, 3), None);
        assert_eq!(hit(14, 3), Some(PetHit::Select));
        assert_eq!(hit(100, 40), Some(PetHit::Select));
        assert_eq!(hit(172, 40), None);
        assert_eq!(hit(100, 46), None);
    }

    #[test]
    fn widened_window_selects_up_to_its_own_width() {
        assert_eq!(hit_sized(300, 20, 372), Some(PetHit::Select));
        assert_eq!(hit_sized(372, 20, 372), None);
        assert_eq!(hit(300, 20), None);
    }

    #[test]
    fn frame_and_body_carry_the_window_alpha() {
        let mut list = DrawList::default();
        paint_pet_status(&mut list, 5, 7, &pet(), &font(), 0.5);
        assert_eq!(with_texture(&list.quads, SMALLWINDOW_BACK2)[0].color[3], 0.5);
        assert_eq!(with_texture(&list.quads, crate::frame::SMALLBAR_1)[0].color[3], 0.5);
        assert_eq!(with_texture(&list.quads, HP_BAR_BACK)[0].color[3], 1.0);
        assert_eq!(
            (with_texture(&list.quads, PET_BUTTON)[0].x, with_texture(&list.quads, PET_BUTTON)[0].y),
            (6.0, 13.0)
        );
    }

    #[test]
    fn widened_window_1010f770_stretches_the_back_and_the_bars_with_the_width() {
        let mut list = DrawList::default();
        paint_pet_status_sized(&mut list, 0, 0, 372, &pet(), &font(), 1.0);
        let back2 = with_texture(&list.quads, SMALLWINDOW_BACK2);
        assert_eq!((back2[0].x, back2[0].w, back2[0].uw), (28.0, 328.0, 16.0));
        let back3 = with_texture(&list.quads, SMALLWINDOW_BACK3_EX);
        assert_eq!((back3[0].x, back3[0].w), (356.0, 19.0));
        let hp_back = with_texture(&list.quads, HP_BAR_BACK);
        assert_eq!((hp_back[0].x, hp_back[0].w), (16.0, 350.0));
        let hp = with_texture(&list.quads, HP_BAR);
        assert_eq!(hp[0].w, 175.0);
        let mut buffs = pet();
        buffs.show_buffs = true;
        let mut list = DrawList::default();
        paint_pet_status_sized(&mut list, 0, 0, 372, &buffs, &font(), 1.0);
        assert_eq!(with_texture(&list.quads, "Icon.skill1204")[0].x, 373.0);
    }

    #[test]
    fn every_texture_exists_in_the_client_list() {
        let known: Vec<String> = TEXTURES.lines().map(|l| l.trim().to_lowercase()).collect();
        let mut view = pet();
        view.show_buffs = true;
        let mut quads = painted(&view);
        quads.extend(painted(&PetStatusView { third: ThirdBar::Time { remaining: 1, max: 2 }, ..pet() }));
        for texture in
            quads.iter().map(|q| q.texture.as_str()).filter(|t| !t.starts_with("Icon.") && *t != "L2Font-e.SmallFont-e")
        {
            assert!(known.contains(&texture.to_lowercase()), "textura desconhecida {texture}");
        }
    }
}
