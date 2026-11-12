use crate::color::TEXT;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::inventory::{BLACK_TEXTURE, OUTLINE, OUTLINE_DOWN, OUTLINE_SIZE, SLOT, grid_slot_at, grid_slot_pos_cols};
use crate::window::Rect;

pub const PAGE_SIZE: (i32, i32) = (256, 335);
pub const ACTION_BACK: &str = "L2UI_CH3.ActionWnd.Action_Back";
pub const CATEGORY_BASIC: u32 = 1;
pub const CATEGORY_PARTY: u32 = 2;
pub const CATEGORY_SOCIAL: u32 = 3;
pub const HEADER_X: i32 = 22;
pub const HEADER_SYSSTRINGS: [u32; 3] = [129, 130, 204];
pub const HEADER_Y: [i32; 3] = [11, 141, 201];
pub const LIST_RECTS: [Rect; 3] = [Rect::new(18, 25, 219, 104), Rect::new(18, 155, 219, 34), Rect::new(18, 215, 219, 104)];
pub const LIST_COLS: usize = 6;
pub const LIST_ROWS: [usize; 3] = [3, 1, 3];
pub const LIST_CATEGORIES: [u32; 3] = [CATEGORY_BASIC, CATEGORY_PARTY, CATEGORY_SOCIAL];
pub const DWARVEN_MANUFACTURE_ID: i32 = 0x25;
pub const DWARVEN_CLASSES: [i32; 7] = [0x35, 0x36, 0x37, 0x38, 0x39, 0x75, 0x76];
pub const CATEGORY2_RIDE: i32 = -1;
pub const SHORTCUT_TYPE_LOW_LIMIT: i32 = 14;
pub const SHORTCUT_TYPE_HIGH_LIMIT: i32 = 27;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActionEntry {
    pub id: i32,
    pub category: u32,
    pub icon: String,
    pub name: String,
    pub desc: String,
    pub ride: bool,
}

impl ActionEntry {
    pub fn tooltip(&self) -> crate::tooltip::Tooltip {
        crate::tooltip::Tooltip { name: self.name.clone(), description: self.desc.clone(), ..Default::default() }
    }

    pub fn icon(&self) -> &str {
        if self.icon.is_empty() { BLACK_TEXTURE } else { &self.icon }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActionContext {
    pub class_id: Option<i32>,
    pub in_world: bool,
    pub riding: bool,
    pub pet: Option<ActionPet>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActionPet {
    pub object_id: i32,
    pub mountable: i32,
}

impl ActionContext {
    pub fn shows(&self, action: &ActionEntry) -> bool {
        if action.id == DWARVEN_MANUFACTURE_ID && !self.class_id.is_some_and(|class| DWARVEN_CLASSES.contains(&class)) {
            return false;
        }
        if action.ride {
            return self.in_world && (self.riding || self.pet.is_some_and(|pet| pet.mountable != 0 && pet.object_id >= 0));
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActionsView {
    pub context: ActionContext,
    pub headers: [String; 3],
    pub actions: Vec<ActionEntry>,
    pub selected: Option<i32>,
    pub pressed: Option<i32>,
}

pub fn shortcut_character_type(action_id: i32) -> i32 {
    if (0..=SHORTCUT_TYPE_LOW_LIMIT).contains(&action_id) {
        1
    } else if (SHORTCUT_TYPE_LOW_LIMIT + 1..=SHORTCUT_TYPE_HIGH_LIMIT).contains(&action_id) {
        2
    } else {
        0
    }
}

impl ActionsView {
    pub fn with_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> ActionsView {
        let headers = HEADER_SYSSTRINGS.map(|id| lookup(id).unwrap_or_else(|| format!("sysstring {id}")));
        ActionsView { headers, ..Default::default() }
    }

    pub fn list(&self, index: usize) -> Vec<&ActionEntry> {
        let category = LIST_CATEGORIES[index];
        self.actions.iter().filter(|a| a.category == category && self.context.shows(a)).take(LIST_COLS * LIST_ROWS[index]).collect()
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<&ActionEntry> {
        for (index, rect) in LIST_RECTS.iter().enumerate() {
            if !rect.contains(x, y) {
                continue;
            }
            let slot = grid_slot_at(x - rect.x, y - rect.y, LIST_COLS, LIST_ROWS[index], 0)?;
            return self.list(index).get(slot).copied();
        }
        None
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let (w, h) = PAGE_SIZE;
        list.draw_texture(x, y, w, h, 0, 0, w, h, ACTION_BACK, 1.0);
        for (index, rect) in LIST_RECTS.iter().enumerate() {
            list.draw_text(x + HEADER_X, y + HEADER_Y[index], TEXT, &self.headers[index], font);
            for (slot, action) in self.list(index).into_iter().enumerate() {
                let (sx, sy) = grid_slot_pos_cols(slot, LIST_COLS);
                let (sx, sy) = (x + rect.x + sx, y + rect.y + sy);
                list.draw_texture(sx, sy, SLOT, SLOT, 0, 0, SLOT, SLOT, action.icon(), 1.0);
                list.draw_texture(sx - 1, sy - 1, OUTLINE_SIZE, OUTLINE_SIZE, 0, 0, OUTLINE_SIZE, OUTLINE_SIZE, OUTLINE, 1.0);
                if self.selected == Some(action.id) && self.pressed == Some(action.id) {
                    list.draw_texture(sx - 1, sy - 1, OUTLINE_SIZE, OUTLINE_SIZE, 0, 0, OUTLINE_SIZE, OUTLINE_SIZE, OUTLINE_DOWN, 1.0);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        let bytes = gly(&[(0x20, &glyphs)], (1024, 128));
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&bytes).unwrap())
    }

    fn action(id: i32, category: u32) -> ActionEntry {
        ActionEntry { id, category, icon: format!("icon.action{id:03}"), name: format!("a{id}"), desc: String::new(), ride: false }
    }

    fn view() -> ActionsView {
        let mut view = ActionsView::with_sysstrings(|id| Some(format!("s{id}")));
        view.actions = vec![action(0, 1), action(1, 1), action(2, 1), action(7, 2), action(8, 2), action(12, 3), action(13, 3), action(15, 4)];
        view
    }

    #[test]
    fn three_lists_by_category_at_the_positions_of_on_create() {
        let view = view();
        assert_eq!(view.headers, ["s129", "s130", "s204"]);
        assert_eq!(LIST_RECTS[1], Rect::new(18, 155, 219, 34));
        assert_eq!(LIST_ROWS, [3, 1, 3]);
        assert_eq!(view.list(0).iter().map(|a| a.id).collect::<Vec<_>>(), vec![0, 1, 2]);
        assert_eq!(view.list(1).iter().map(|a| a.id).collect::<Vec<_>>(), vec![7, 8]);
        assert_eq!(view.list(2).iter().map(|a| a.id).collect::<Vec<_>>(), vec![12, 13]);
        assert_eq!(view.hit(18 + 1 + 5, 25 + 1 + 5).map(|a| a.id), Some(0));
        assert_eq!(view.hit(18 + 38, 26).map(|a| a.id), Some(1));
        assert_eq!(view.hit(18 + 38 * 3, 26), None);
        assert_eq!(view.hit(19, 156).map(|a| a.id), Some(7));
        assert_eq!(view.hit(19 + 37, 216).map(|a| a.id), Some(13));
        assert_eq!(view.hit(5, 5), None);
        assert_eq!(shortcut_character_type(0), 1);
        assert_eq!(shortcut_character_type(14), 1);
        assert_eq!(shortcut_character_type(15), 2);
        assert_eq!(shortcut_character_type(27), 2);
        assert_eq!(shortcut_character_type(28), 0);
        assert_eq!(shortcut_character_type(1000), 0);
        assert_eq!(shortcut_character_type(-1), 0);
    }

    #[test]
    fn paint_draws_the_back_headers_icons_and_outlines() {
        let mut view = view();
        view.selected = Some(1);
        view.pressed = Some(1);
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 66, &font());
        assert_eq!((list.quads[0].texture.as_str(), list.quads[0].y, list.quads[0].h), (ACTION_BACK, 66.0, 335.0));
        let icon0: Vec<_> = list.quads.iter().filter(|q| q.texture == "icon.action000").collect();
        assert_eq!((icon0[0].x, icon0[0].y, icon0[0].w), (19.0, 66.0 + 26.0, 32.0));
        let icon7: Vec<_> = list.quads.iter().filter(|q| q.texture == "icon.action007").collect();
        assert_eq!((icon7[0].x, icon7[0].y), (19.0, 66.0 + 156.0));
        let icon13: Vec<_> = list.quads.iter().filter(|q| q.texture == "icon.action013").collect();
        assert_eq!((icon13[0].x, icon13[0].y), (19.0 + 37.0, 66.0 + 216.0));
        assert!(!list.quads.iter().any(|q| q.texture == "icon.action015"));
        assert_eq!(list.quads.iter().filter(|q| q.texture == OUTLINE).count(), 7);
        let down: Vec<_> = list.quads.iter().filter(|q| q.texture == OUTLINE_DOWN).collect();
        assert_eq!((down.len(), down[0].x, down[0].y), (1, 18.0 + 37.0, 66.0 + 25.0));
        let headers: Vec<(f32, f32)> = list.quads.iter().filter(|q| q.texture == "L2Font-e.SmallFont-e" && q.x == 22.0).map(|q| (q.x, q.y)).collect();
        assert_eq!(headers, vec![(22.0, 77.0), (22.0, 207.0), (22.0, 267.0)]);
    }

    fn view_with(context: ActionContext) -> ActionsView {
        let mount = ActionEntry { ride: true, ..action(38, CATEGORY_BASIC) };
        ActionsView { actions: vec![action(0, CATEGORY_BASIC), action(DWARVEN_MANUFACTURE_ID, CATEGORY_BASIC), mount], context, ..Default::default() }
    }

    fn ids(view: &ActionsView) -> Vec<i32> {
        view.list(0).iter().map(|a| a.id).collect()
    }

    #[test]
    fn mount_needs_a_ride_or_a_summoned_mountable_pet() {
        let world = ActionContext { in_world: true, class_id: Some(0), ..Default::default() };
        assert_eq!(ids(&view_with(world)), [0]);
        let riding = ActionContext { riding: true, ..world };
        assert_eq!(ids(&view_with(riding)), [0, 38]);
        let strider = ActionContext { pet: Some(ActionPet { object_id: 5, mountable: 1 }), ..world };
        assert_eq!(ids(&view_with(strider)), [0, 38]);
        let wolf = ActionContext { pet: Some(ActionPet { object_id: 5, mountable: 0 }), ..world };
        assert_eq!(ids(&view_with(wolf)), [0]);
        let outside = ActionContext { riding: true, ..Default::default() };
        assert_eq!(ids(&view_with(outside)), [0]);
    }

    #[test]
    fn dwarven_manufacture_is_only_for_the_dwarven_crafting_classes() {
        for class in DWARVEN_CLASSES {
            let context = ActionContext { in_world: true, class_id: Some(class), ..Default::default() };
            assert_eq!(ids(&view_with(context)), [0, DWARVEN_MANUFACTURE_ID]);
        }
        let human = ActionContext { in_world: true, class_id: Some(0x34), ..Default::default() };
        assert_eq!(ids(&view_with(human)), [0]);
        assert_eq!(ids(&view_with(ActionContext::default())), [0]);
    }

    #[test]
    fn the_tooltip_is_the_name_with_the_description_of_the_action_table() {
        let entry = ActionEntry { desc: "texto".into(), ..action(3, 1) };
        let tooltip = entry.tooltip();
        assert_eq!((tooltip.name.as_str(), tooltip.description.as_str()), ("a3", "texto"));
        assert!(tooltip.lines.is_empty() && tooltip.level.is_none());
    }
}
