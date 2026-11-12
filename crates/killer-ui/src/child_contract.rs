use crate::window::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChildKind {
    EditBox,
    CheckBox,
    PushButton,
    ComboBox,
    TabControl,
    Icon,
    SliderBar,
    ListCtrl,
}

pub const EDIT_KIND: u16 = 0x214;
pub const EDIT_MAX_LEN: u16 = 0x254;
pub const EDIT_KIND_PASSWORD: u32 = 2;
pub const EDIT_KIND_TINTED: u32 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChildContract {
    pub kind: ChildKind,
    pub rect: Rect,
    pub id: u32,
    pub vtable: u32,
    pub registration: (u32, u32, u32),
    pub handles: &'static [u16],
    pub state: &'static [(u16, u32)],
    pub textures: &'static [(u16, &'static str)],
}

impl ChildContract {
    pub const fn rect_at(&self, origin: (i32, i32)) -> Rect {
        Rect::new(self.rect.x + origin.0, self.rect.y + origin.1, self.rect.w, self.rect.h)
    }

    pub fn field(&self, offset: u16) -> u32 {
        self.state.iter().find(|(at, _)| *at == offset).map_or(0, |(_, value)| *value)
    }

    pub fn holds_handle(&self, offset: u16) -> bool {
        self.handles.contains(&offset)
    }

    pub fn texture(&self, offset: u16) -> Option<&'static str> {
        self.textures.iter().find(|(at, _)| *at == offset).map(|(_, name)| *name)
    }

    pub const fn texture_str(&self, offset: u16) -> &'static str {
        let mut index = 0;
        while index < self.textures.len() {
            if self.textures[index].0 == offset {
                return self.textures[index].1;
            }
            index += 1;
        }
        ""
    }

    pub fn edit_kind(&self) -> Option<u32> {
        (self.kind == ChildKind::EditBox).then(|| self.field(EDIT_KIND))
    }

    pub fn masks_input(&self) -> bool {
        self.edit_kind() == Some(EDIT_KIND_PASSWORD)
    }

    pub fn edit_max_len(&self) -> Option<u32> {
        (self.kind == ChildKind::EditBox).then(|| self.field(EDIT_MAX_LEN))
    }

    pub fn hit(&self, origin: (i32, i32), x: i32, y: i32) -> Option<u32> {
        let rect = self.rect_at(origin);
        (x >= rect.x && y >= rect.y && x < rect.x + rect.w && y < rect.y + rect.h).then_some(self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EDIT: ChildContract = ChildContract {
        kind: ChildKind::EditBox,
        rect: Rect::new(165, 374, 63, 17),
        id: 0xf0e404,
        vtable: 0x101a2730,
        registration: (2, 0, 0),
        handles: &[0x230],
        state: &[(0x214, 2), (0x254, 16)],
        textures: &[(0x230, "NWindow.ChatBack")],
    };

    #[test]
    fn state_defaults_to_zero_inside_the_extracted_range() {
        assert_eq!(EDIT.field(0x214), 2);
        assert_eq!(EDIT.field(0x218), 0);
        assert_eq!(EDIT.texture(0x230), Some("NWindow.ChatBack"));
        assert_eq!(EDIT.texture(0x234), None);
        assert_eq!(EDIT.texture_str(0x230), "NWindow.ChatBack");
        assert_eq!(EDIT.texture_str(0x234), "");
    }

    #[test]
    fn edit_accessors_only_apply_to_edit_boxes() {
        assert!(EDIT.masks_input());
        assert_eq!(EDIT.edit_max_len(), Some(16));
        let check = ChildContract { kind: ChildKind::CheckBox, ..EDIT };
        assert_eq!(check.edit_kind(), None);
        assert!(!check.masks_input());
        assert_eq!(check.edit_max_len(), None);
    }

    #[test]
    fn hit_uses_the_origin_and_half_open_bounds() {
        assert_eq!(EDIT.hit((10, 20), 175, 394), Some(0xf0e404));
        assert_eq!(EDIT.hit((10, 20), 174, 394), None);
        assert_eq!(EDIT.hit((10, 20), 238, 394), None);
        assert_eq!(EDIT.hit((10, 20), 175, 411), None);
    }
}
