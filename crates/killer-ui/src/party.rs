use crate::abnormal::{BuffIcon, DEBUFF_OVERLAY, SKILL_ICON_TEXELS};
use crate::color::TEXT;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::status::{ellipsize, fill_width};
use crate::target::{
    BODY_END_WIDTH, BODY_RIGHT_INSET, BODY_RIGHT_WIDTH, BODY_UW, SMALLWINDOW_BACK1, SMALLWINDOW_BACK2,
    SMALLWINDOW_BACK3_EX,
};
use crate::window::Rect;

pub const PARTY_WIDTH: i32 = 172;
pub const PARTY_MAX_WIDTH: i32 = PARTY_WIDTH + 202;
pub const PARTY_DEFAULT_POS: (i32, i32) = (0, 152);
pub const PARTY_INITIAL_POSITION: crate::initial_position::InitialPosition = crate::initial_position::InitialPosition {
    enabled: true, horizontal_mode: 0, vertical_mode: 3, x: PARTY_DEFAULT_POS.0, y: PARTY_DEFAULT_POS.1,
};
pub const CELL_HEIGHT: i32 = 46;
pub const CELL_X: i32 = 12;
pub const CELL_STEP: i32 = 46;
pub const MAX_MEMBERS: usize = 8;
pub const BUFF_BUTTON: &str = "L2UI_CH3.PartyWnd.party_buffbutton";
pub const BUFF_BUTTON_DOWN: &str = "L2UI_CH3.PartyWnd.party_buffbutton_down";
pub const BUFF_BUTTON_RECT: Rect = Rect::new(1, 7, 11, 13);
pub const BUFF_BUTTON_TOOLTIP_SYSSTRING: u32 = 881;
pub const LEADER_ICON: &str = "L2UI_CH3.PartyWnd.party_leadericon";
pub const STYLE_ICON: &str = "L2UI_CH3.PartyWnd.party_styleicon1";
pub const LEADER_ICON_RECT: Rect = Rect::new(4, 8, 11, 11);
pub const LEADER_ICON_NAME_GAP: f32 = 11.0;

pub fn leader_icon_x(cell_width: i32, name_width: i32) -> i32 {
    (cell_width as f32 * 0.5 - (name_width / 2) as f32 - LEADER_ICON_NAME_GAP) as i32
}
pub const CP_BAR: &str = "L2UI_CH3.SmallWnd.cpBar";
pub const CP_BAR_BACK: &str = "L2UI_CH3.SmallWnd.CpBar_back";
pub const HP_BAR: &str = "L2UI_CH3.SmallWnd.HpBar";
pub const HP_BAR_BACK: &str = "L2UI_CH3.SmallWnd.HpBar_back";
pub const MP_BAR: &str = "L2UI_CH3.SmallWnd.MpBar";
pub const MP_BAR_BACK: &str = "L2UI_CH3.SmallWnd.MpBar_back";
pub const BODY_MID_INSET: i32 = 32;
pub const BAR_X: i32 = 4;
pub const BAR_RIGHT_INSET: i32 = 10;
pub const BAR_BACK_HEIGHT: i32 = 6;
pub const BAR_FILL_HEIGHT: i32 = 5;
pub const BAR_UW: i32 = 8;
pub const BAR_VH: i32 = 6;
pub const CP_BAR_Y: i32 = 21;
pub const HP_BAR_Y: i32 = 28;
pub const MP_BAR_Y: i32 = 35;
pub const NAME_Y: i32 = 8;
pub const BUFF_ICON: i32 = 16;
pub const BUFF_ROW_YS: [i32; 2] = [5, 21];
pub const BUFFS_PER_ROW: usize = 10;
pub const MAX_BUFF_ICONS: usize = 20;
pub const CLICK_RIGHT_INSET: i32 = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyMemberView {
    pub object_id: i32,
    pub name: String,
    pub cp: (i32, i32),
    pub hp: (i32, i32),
    pub mp: (i32, i32),
    pub leader: bool,
    pub buffs: Vec<BuffIcon>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PartyView {
    pub members: Vec<PartyMemberView>,
    pub show_buffs: bool,
    pub loot_rule: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartyHit {
    Grip,
    BuffButton,
    Member(usize),
}

impl PartyView {
    pub fn height(&self) -> i32 {
        self.members.len().min(MAX_MEMBERS) as i32 * CELL_STEP
    }

    pub fn size(&self, width: i32) -> (i32, i32) {
        (width.clamp(PARTY_WIDTH, PARTY_MAX_WIDTH), self.height())
    }

    pub fn cell(&self, index: usize, width: i32) -> Option<Rect> {
        (index < self.members.len().min(MAX_MEMBERS))
            .then(|| Rect::new(CELL_X, index as i32 * CELL_STEP, width - CELL_X, CELL_HEIGHT))
    }

    pub fn hit(&self, x: i32, y: i32, width: i32) -> Option<PartyHit> {
        if self.members.is_empty() {
            return None;
        }
        if BUFF_BUTTON_RECT.contains(x, y) {
            return Some(PartyHit::BuffButton);
        }
        if grip(self.height()).contains(x, y) {
            return Some(PartyHit::Grip);
        }
        (0..self.members.len().min(MAX_MEMBERS))
            .find(|i| self.cell(*i, width).is_some_and(|c| c.contains(x, y) && x < c.x + c.w - CLICK_RIGHT_INSET))
            .map(PartyHit::Member)
    }
}

pub const LOOT_RULE_LABEL_SYSSTRING: u32 = 489;
pub const LOOT_RULE_SYSSTRINGS: [u32; 5] = [487, 488, 798, 799, 800];

pub fn class_group(class_id: i32) -> i32 {
    CLASS_GROUPS.iter().find(|(id, _)| *id == class_id).map_or(class_id, |(_, group)| *group)
}

pub const CLASS_GROUP_SYSSTRINGS: [u32; 16] = [874, 875, 876, 877, 878, 879, 880, 175, 176, 874, 875, 876, 877, 878, 879, 880];
pub const CLASS_GROUPS: [(i32, i32); 89] = [
    (0, 7), (1, 7), (2, 0), (3, 0), (4, 7), (5, 2), (6, 2), (7, 7), (8, 0), (9, 1), (10, 8), (11, 8), (12, 4),
    (13, 4), (14, 6), (15, 8), (16, 5), (17, 5), (18, 7), (19, 7), (20, 2), (21, 3), (22, 7), (23, 0),
    (24, 1), (25, 8), (26, 8), (27, 4), (28, 6), (29, 8), (30, 5), (31, 7), (32, 7), (33, 2), (34, 3),
    (35, 7), (36, 0), (37, 1), (38, 8), (39, 8), (40, 4), (41, 6), (42, 8), (43, 5), (44, 7), (45, 7),
    (46, 0), (47, 7), (48, 0), (49, 8), (50, 8), (51, 5), (52, 5), (53, 7), (54, 7), (55, 0), (56, 7),
    (57, 0), (88, 9), (89, 9), (90, 11), (91, 11), (92, 10), (93, 9), (94, 13), (95, 13), (96, 15), (97, 14),
    (98, 14), (99, 11), (100, 12), (101, 9), (102, 10), (103, 13), (104, 15), (105, 14), (106, 11), (107, 12),
    (108, 9), (109, 10), (110, 13), (111, 15), (112, 14), (113, 9), (114, 9), (115, 14), (116, 14), (117, 9),
    (118, 9),
];

impl PartyView {
    pub fn leader_icon_at(&self, x: i32, y: i32, width: i32, font: &BitmapFont) -> bool {
        let w = self.size(width).0;
        let Some(index) = self.members.iter().position(|member| member.leader && !member.name.is_empty()) else { return false };
        let Some(cell) = self.cell(index.min(MAX_MEMBERS - 1), w) else { return false };
        let name_width = font.measure(&ellipsize(&self.members[index].name, cell.w, font)) as i32;
        let lx = cell.x + leader_icon_x(cell.w, name_width);
        let Rect { y: ly, w: lw, h: lh, .. } = LEADER_ICON_RECT;
        Rect::new(lx, cell.y + ly, lw, lh).contains(x, y)
    }
}

pub const GRIP_TOP: i32 = 18;

pub fn grip(height: i32) -> Rect {
    let edge = FrameCtrl::new(FrameMode::VerticalGrip).grip(PARTY_WIDTH, height);
    Rect::new(edge.x, GRIP_TOP, edge.w, (height - GRIP_TOP).max(0))
}

pub fn buff_icon_pos(index: usize, cell_width: i32) -> Option<(i32, i32)> {
    if index >= MAX_BUFF_ICONS {
        return None;
    }
    let (row, col) = (index / BUFFS_PER_ROW, index % BUFFS_PER_ROW);
    Some((cell_width + 1 + col as i32 * BUFF_ICON, BUFF_ROW_YS[row]))
}

pub fn paint_party(list: &mut DrawList, x: i32, y: i32, width: i32, view: &PartyView, font: &BitmapFont, alpha: f32) {
    if view.members.is_empty() {
        return;
    }
    let (w, h) = view.size(width);
    FrameCtrl::new(FrameMode::VerticalGrip).paint(list, x, y, w, h, font, alpha);
    let button = if view.show_buffs { BUFF_BUTTON_DOWN } else { BUFF_BUTTON };
    let Rect { x: bx, y: by, w: bw, h: bh } = BUFF_BUTTON_RECT;
    list.draw_texture(x + bx, y + by, bw, bh, 0, 0, bw, bh, button, 1.0);
    for (index, member) in view.members.iter().enumerate().take(MAX_MEMBERS) {
        let Some(cell) = view.cell(index, w) else { continue };
        paint_member(list, x + cell.x, y + cell.y, cell.w, member, view.show_buffs, font, alpha);
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_member(
    list: &mut DrawList,
    x: i32,
    y: i32,
    w: i32,
    member: &PartyMemberView,
    show_buffs: bool,
    font: &BitmapFont,
    alpha: f32,
) {
    let h = CELL_HEIGHT;
    list.draw_texture(x, y, BODY_END_WIDTH, h, 0, 0, BODY_UW, h, SMALLWINDOW_BACK1, alpha);
    list.draw_texture(x + BODY_END_WIDTH, y, w - BODY_MID_INSET, h, 0, 0, BODY_UW, h, SMALLWINDOW_BACK2, alpha);
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
    if member.leader && !member.name.is_empty() {
        let Rect { y: ly, w: lw, h: lh, .. } = LEADER_ICON_RECT;
        let lx = leader_icon_x(w, font.measure(&ellipsize(&member.name, w, font)) as i32);
        list.draw_texture(x + lx, y + ly, lw, lh, 0, 0, lw, lh, LEADER_ICON, 1.0);
    }
    if member.name.is_empty() {
        return;
    }
    let name = ellipsize(&member.name, w, font);
    list.draw_text(x + w / 2 - font.measure(&name) as i32 / 2, y + NAME_Y, TEXT, &name, font);
    let bar_w = w - BAR_RIGHT_INSET;
    paint_bar(list, x + BAR_X, y + CP_BAR_Y, bar_w, CP_BAR_BACK, CP_BAR, fill_width(member.cp, bar_w));
    paint_bar(list, x + BAR_X, y + HP_BAR_Y, bar_w, HP_BAR_BACK, HP_BAR, fill_width(member.hp, bar_w));
    paint_bar(list, x + BAR_X, y + MP_BAR_Y, bar_w, MP_BAR_BACK, MP_BAR, fill_width(member.mp, bar_w));
    if !show_buffs {
        return;
    }
    let placed: Vec<(usize, (i32, i32))> =
        (0..member.buffs.len().min(MAX_BUFF_ICONS)).filter_map(|i| buff_icon_pos(i, w).map(|p| (i, p))).collect();
    for (index, (ix, iy)) in placed.iter().filter(|(i, _)| !member.buffs[*i].debuff) {
        if let Some(icon) = &member.buffs[*index].icon {
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
    for (index, (ix, iy)) in placed.iter().filter(|(i, _)| member.buffs[*i].debuff) {
        if let Some(icon) = &member.buffs[*index].icon {
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
        list.draw_texture(x + ix, y + iy, BUFF_ICON, BUFF_ICON, 0, 0, 26, 26, DEBUFF_OVERLAY, 1.0);
    }
}

fn paint_bar(list: &mut DrawList, x: i32, y: i32, w: i32, back: &str, bar: &str, fill: i32) {
    list.draw_texture(x, y, w, BAR_BACK_HEIGHT, 0, 0, BAR_UW, BAR_VH, back, 1.0);
    if fill > 0 {
        list.draw_texture(x, y, fill, BAR_FILL_HEIGHT, 0, 0, BAR_UW, BAR_VH, bar, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Quad;
    use crate::font::{Glyphs, gly};

    const TEXTURES: &str = include_str!("../tests/fixtures/nwindow-texturas-party.txt");

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn member(id: i32, name: &str, leader: bool) -> PartyMemberView {
        PartyMemberView {
            object_id: id,
            name: name.into(),
            cp: (50, 100),
            hp: (300, 600),
            mp: (45, 90),
            leader,
            buffs: vec![
                BuffIcon { icon: Some("Icon.skill1204".into()), remaining: Some(100), debuff: false, ..Default::default() },
                BuffIcon { icon: Some("Icon.skill1069".into()), remaining: Some(10), debuff: true, ..Default::default() },
            ],
        }
    }

    fn party(n: usize) -> PartyView {
        PartyView {
            members: (0..n).map(|i| member(10 + i as i32, &format!("M{i}"), i == 0)).collect(),
            show_buffs: false,
            loot_rule: 0,
        }
    }

    fn painted(view: &PartyView) -> Vec<Quad> {
        let mut list = DrawList::default();
        paint_party(&mut list, 0, 0, PARTY_WIDTH, view, &font(), 1.0);
        list.quads
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    #[test]
    fn cells_of_160_by_46_stack_every_46_pixels_right_of_the_grip() {
        let view = party(3);
        assert_eq!(view.size(PARTY_WIDTH), (172, 138));
        assert_eq!(view.size(250), (250, 138));
        assert_eq!(view.size(500), (374, 138));
        assert_eq!(view.cell(2, 172), Some(Rect::new(12, 92, 160, 46)));
        assert_eq!(view.cell(3, 172), None);
        assert_eq!(grip(138), Rect::new(0, 18, 12, 120));
        let quads = painted(&view);
        let grip = with_texture(&quads, crate::frame::SMALLBAR_2);
        assert_eq!((grip[0].x, grip[0].y, grip[0].w, grip[0].h), (0.0, 8.0, 12.0, 122.0));
        let back1 = with_texture(&quads, SMALLWINDOW_BACK1);
        assert_eq!(back1.len(), 3);
        assert_eq!(
            (back1[1].x, back1[1].y, back1[1].w, back1[1].h, back1[1].uw, back1[1].vh),
            (12.0, 46.0, 16.0, 46.0, 16.0, 46.0)
        );
        let back2 = with_texture(&quads, SMALLWINDOW_BACK2);
        assert_eq!((back2[0].x, back2[0].w, back2[0].uw), (28.0, 128.0, 16.0));
        let back3 = with_texture(&quads, SMALLWINDOW_BACK3_EX);
        assert_eq!((back3[0].x, back3[0].w, back3[0].uw), (156.0, 19.0, 19.0));
        let button = with_texture(&quads, BUFF_BUTTON);
        assert_eq!((button[0].x, button[0].y, button[0].w, button[0].h), (1.0, 7.0, 11.0, 13.0));
        let leader = with_texture(&quads, LEADER_ICON);
        assert_eq!(leader.len(), 1);
        assert_eq!((leader[0].x, leader[0].y, leader[0].w), (12.0 + leader_icon_x(160, font().measure("M0") as i32) as f32, 8.0, 11.0));
        assert_eq!(leader_icon_x(160, 40), 49);
        assert!(painted(&PartyView::default()).is_empty());
        assert_eq!(party(9).size(PARTY_WIDTH).1, 368);
    }

    #[test]
    fn bars_are_150_wide_at_4_with_six_pixel_backs_and_five_pixel_fills_at_21_28_and_35() {
        let quads = painted(&party(1));
        let cp_back = with_texture(&quads, CP_BAR_BACK);
        assert_eq!(
            (cp_back[0].x, cp_back[0].y, cp_back[0].w, cp_back[0].h, cp_back[0].uw, cp_back[0].vh),
            (16.0, 21.0, 150.0, 6.0, 8.0, 6.0)
        );
        let cp = with_texture(&quads, CP_BAR);
        assert_eq!((cp[0].w, cp[0].h), (75.0, 5.0));
        assert_eq!((with_texture(&quads, HP_BAR_BACK)[0].y, with_texture(&quads, HP_BAR)[0].w), (28.0, 75.0));
        assert_eq!((with_texture(&quads, MP_BAR_BACK)[0].y, with_texture(&quads, MP_BAR)[0].w), (35.0, 75.0));
        let font = font();
        let name: Vec<&Quad> = quads.iter().filter(|q| q.texture == font.atlas).collect();
        assert_eq!((name[0].x, name[0].y, name[0].color), ((12 + 80 - 6) as f32, 8.0, TEXT.rgba()));
        let mut wide = DrawList::default();
        paint_party(&mut wide, 0, 0, 202, &party(1), &font, 1.0);
        assert_eq!(with_texture(&wide.quads, CP_BAR_BACK)[0].w, 180.0);
        assert_eq!(with_texture(&wide.quads, SMALLWINDOW_BACK2)[0].w, 158.0);
    }

    #[test]
    fn expanded_buffs_sit_right_of_each_cell_with_debuffs_drawn_last_under_the_overlay() {
        let mut view = party(2);
        view.show_buffs = true;
        let quads = painted(&view);
        assert_eq!(with_texture(&quads, BUFF_BUTTON_DOWN).len(), 1);
        let buff = with_texture(&quads, "Icon.skill1204");
        assert_eq!((buff[0].x, buff[0].y, buff[0].w, buff[0].uw), (12.0 + 161.0, 5.0, 16.0, 32.0));
        assert_eq!(buff[1].y, 46.0 + 5.0);
        let debuff = with_texture(&quads, "Icon.skill1069");
        assert_eq!((debuff[0].x, debuff[0].y), (12.0 + 161.0 + 16.0, 5.0));
        let overlay = with_texture(&quads, DEBUFF_OVERLAY);
        assert_eq!((overlay[0].x, overlay[0].y, overlay[0].w, overlay[0].uw), (12.0 + 161.0 + 16.0, 5.0, 16.0, 26.0));
        let order: Vec<&str> = quads
            .iter()
            .map(|q| q.texture.as_str())
            .filter(|t| t.starts_with("Icon.") || *t == DEBUFF_OVERLAY)
            .take(3)
            .collect();
        assert_eq!(order, ["Icon.skill1204", "Icon.skill1069", DEBUFF_OVERLAY]);
        assert!(painted(&party(1)).iter().all(|q| !q.texture.starts_with("Icon.")));
        assert_eq!(buff_icon_pos(10, 160), Some((161, 21)));
        assert_eq!(buff_icon_pos(20, 160), None);
    }

    #[test]
    fn clicks_split_into_buff_button_grip_and_member_cells_minus_the_right_ten_pixels() {
        let view = party(2);
        assert_eq!(view.hit(5, 10, 172), Some(PartyHit::BuffButton));
        assert_eq!(view.hit(5, 50, 172), Some(PartyHit::Grip));
        assert_eq!(view.hit(100, 20, 172), Some(PartyHit::Member(0)));
        assert_eq!(view.hit(100, 60, 172), Some(PartyHit::Member(1)));
        assert_eq!(view.hit(161, 60, 172), Some(PartyHit::Member(1)));
        assert_eq!(view.hit(162, 60, 172), None);
        assert_eq!(view.hit(100, 100, 172), None);
        assert_eq!(PartyView::default().hit(5, 10, 172), None);
    }

    #[test]
    fn frame_and_body_carry_the_window_alpha() {
        let mut list = DrawList::default();
        paint_party(&mut list, 10, 20, 172, &party(1), &font(), 0.5);
        assert_eq!(with_texture(&list.quads, SMALLWINDOW_BACK2)[0].color[3], 0.5);
        assert_eq!(with_texture(&list.quads, crate::frame::SMALLBAR_1)[0].color[3], 0.5);
        assert_eq!(with_texture(&list.quads, HP_BAR_BACK)[0].color[3], 1.0);
        assert_eq!(
            (with_texture(&list.quads, SMALLWINDOW_BACK1)[0].x, with_texture(&list.quads, SMALLWINDOW_BACK1)[0].y),
            (22.0, 20.0)
        );
    }

    #[test]
    fn every_texture_exists_in_the_client_list() {
        let known: Vec<String> = TEXTURES.lines().map(|l| l.trim().to_lowercase()).collect();
        let mut view = party(2);
        view.show_buffs = true;
        for texture in painted(&view)
            .iter()
            .map(|q| q.texture.as_str())
            .filter(|t| !t.starts_with("Icon.") && *t != "L2Font-e.SmallFont-e")
        {
            assert!(known.contains(&texture.to_lowercase()), "textura desconhecida {texture}");
        }
        assert!(known.contains(&STYLE_ICON.to_lowercase()));
    }

    #[test]
    fn the_leader_icon_sits_left_of_the_centered_name_and_the_class_groups_follow_the_table() {
        assert_eq!(leader_icon_x(172, 40), (86.0f32 - 20.0 - 11.0) as i32);
        assert_eq!(class_group(2), 0);
        assert_eq!(class_group(10), 8);
        assert_eq!(class_group(111), 15);
        assert_eq!(class_group(60), 60);
        assert_eq!(CLASS_GROUP_SYSSTRINGS[class_group(5) as usize], 876);
        assert_eq!(LOOT_RULE_SYSSTRINGS[2], 798);
    }

    #[test]
    #[ignore]
    fn class_groups_match_the_nwindow_function_for_every_class_id() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR") else { return };
        let dll = std::fs::read(std::path::Path::new(&root).join("system/nwindow.dll")).unwrap();
        let u32_at = |at: usize| u32::from_le_bytes(dll[at..at + 4].try_into().unwrap());
        let pe = u32_at(0x3c) as usize;
        let sections = u16::from_le_bytes([dll[pe + 6], dll[pe + 7]]) as usize;
        let optional = pe + 24;
        let base = u32_at(optional + 28);
        let first = optional + u16::from_le_bytes([dll[pe + 20], dll[pe + 21]]) as usize;
        let offset = |va: u32| {
            (0..sections).find_map(|i| {
                let s = first + 40 * i;
                let (size, rva, raw, at) = (u32_at(s + 8), u32_at(s + 12), u32_at(s + 16), u32_at(s + 20));
                let r = va - base;
                (r >= rva && r < rva + size.max(raw)).then(|| (r - rva + at) as usize)
            })
        };
        let function = 0x1006bac0u32;
        let code = &dll[offset(function).unwrap()..][..0x360];
        let block = |target: u32| {
            let at = offset(target).unwrap();
            if dll[at] == 0xb8 { u32_at(at + 1) as i32 } else { 0 }
        };
        let mut cases: Vec<(i32, i32)> = Vec::new();
        let mut p = 4;
        loop {
            if code[p] == 0x83 && code[p + 1] == 0xf8 && code[p + 3] == 0x0f && code[p + 4] == 0x84 {
                let rel = i32::from_le_bytes(code[p + 5..p + 9].try_into().unwrap());
                cases.push((i32::from(code[p + 2]), block((function as i64 + p as i64 + 9 + i64::from(rel)) as u32)));
                p += 9;
            } else if code[p] == 0x83 && code[p + 1] == 0xf8 && code[p + 3] == 0x74 {
                cases.push((i32::from(code[p + 2]), block((function as i64 + p as i64 + 5 + i64::from(code[p + 4] as i8)) as u32)));
                p += 5;
            } else if code[p] == 0x85 && code[p + 1] == 0xc0 && code[p + 2] == 0x0f && code[p + 3] == 0x84 {
                let rel = i32::from_le_bytes(code[p + 4..p + 8].try_into().unwrap());
                cases.push((0, block((function as i64 + p as i64 + 8 + i64::from(rel)) as u32)));
                p += 8;
            } else {
                break;
            }
        }
        assert_eq!(&code[p..p + 3], [0x83, 0xf8, 0x6f]);
        cases.push((0x6f, u32_at(offset(function).unwrap() + p + 6) as i32));
        for id in 0..400 {
            let original = cases.iter().find(|(case, _)| *case == id).map_or(id, |(_, group)| *group);
            assert_eq!(class_group(id), original, "classe {id}");
        }
    }
}
