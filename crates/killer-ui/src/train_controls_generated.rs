use crate::button::TextButton;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

pub struct TextButtonContract {
    pub rect: Rect,
    pub id: u32,
    pub sysstring: u32,
    pub normal: &'static str,
    pub down: &'static str,
}

impl TextButtonContract {
    pub const fn rect_at(&self, origin: (i32, i32)) -> Rect {
        Rect::new(self.rect.x + origin.0, self.rect.y + origin.1, self.rect.w, self.rect.h)
    }

    pub fn build(&self, origin: (i32, i32), text: impl Fn(u32) -> String) -> TextButton {
        TextButton::new(self.rect_at(origin), self.normal, self.down, &text(self.sysstring))
    }

    pub fn hit(&self, button: &TextButton, x: i32, y: i32) -> Option<u32> {
        button.contains(x, y).then_some(self.id)
    }

    pub fn paint(&self, button: &TextButton, list: &mut DrawList, origin: (i32, i32), pressed: bool, font: &BitmapFont) {
        button.paint(list, origin.0, origin.1, pressed, font);
    }
}

pub const BUTTON_5C0: TextButtonContract = TextButtonContract {
    rect: Rect::new(51, 352, 76, 23),
    id: 0x2,
    sysstring: 368,
    normal: "L2UI_CH3.Button.Btn1_Normal",
    down: "L2UI_CH3.Button.Btn1_NormalOn",
};

pub const BUTTON_5C4: TextButtonContract = TextButtonContract {
    rect: Rect::new(131, 352, 76, 23),
    id: 0x3,
    sysstring: 369,
    normal: "L2UI_CH3.Button.Btn1_Normal",
    down: "L2UI_CH3.Button.Btn1_NormalOn",
};
