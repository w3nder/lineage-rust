use crate::{color::Argb, draw::DrawList, font::BitmapFont};

#[derive(Clone, Debug, PartialEq)]
pub struct StoreSign {
    pub text: String,
    pub color: Argb,
    pub background: Argb,
    pub screen: [f32; 2],
}

pub fn style(kind: u8, sitting: bool, search: u8) -> Option<(u32, Argb, Argb)> {
    let (label, color, selected, background, search_kind) = match kind {
        1 if sitting => (500, 0xffeaa5f5, 0xff3cff3c, 0xff00ff00, 1),
        3 if sitting => (499, 0xfffdfea5, 0xff3cffc8, 0xff00ffff, 2),
        5 if sitting => (665, 0xffffbb33, 0xffc0ff00, 0xffffff00, 3),
        8 if sitting => (1273, 0xfffc7575, 0xff1788f2, 0xff3cc8ff, 4),
        7 => (741, 0xff8041e9, 0xff8041e9, 0xff000000, 0),
        _ => return None,
    };
    Some((label, Argb(if search == search_kind { selected } else { color }), Argb(if search == search_kind { background } else { 0xff000000 })))
}

impl StoreSign {
    pub fn paint(&self, list: &mut DrawList, font: &BitmapFont) {self.paint_kind(list,font,false);}

    pub fn paint_speech(&self, list: &mut DrawList, font: &BitmapFont) {self.paint_kind(list,font,true);}

    fn paint_kind(&self, list: &mut DrawList, font: &BitmapFont, speech: bool) {
        let lines: Vec<_> = self.text.split('\n').collect();
        let width = lines.iter().map(|line| font.measure(line) as i32).max().unwrap_or(0);
        let x = self.screen[0] as i32 - width / 2;
        let y = self.screen[1] as i32 - lines.len() as i32 * font.line_height as i32;
        let middle = x + width / 2 - 5;
        let (body,tail,kind)=if speech {(19,25,1)}else{(32,38,2)};
        for (left, right, height, part) in [
            (middle, middle + 8, tail, 3),
            (x - 5, x + 3, body, 1),
            (x + width - 3, x + width + 5, body, 4),
            (x + 3, middle, body, 2),
            (middle + 8, x + width - 3, body, 2),
        ] {
            list.draw_texture(left, y - 5, right - left, height, 0, 0, 8, height, &format!("L2UI_ch3.ChatBack.balloon{kind}_{part}"), 1.0);
            if let Some(quad) = list.quads.last_mut() { quad.color = self.background.rgba(); }
        }
        for (row, line) in lines.into_iter().enumerate() {
            list.draw_text(x, y + row as i32 * font.line_height as i32, self.color, line, font);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_chat_106184a0_covers_every_store_type_wait_state_and_search_kind() {
        for kind in 0..=u8::MAX {
            for sitting in [false, true] {
                for search in 0..=4 {
                    let result = style(kind, sitting, search);
                    assert_eq!(result.is_some(), kind == 7 || (sitting && [1, 3, 5, 8].contains(&kind)));
                    if let Some((_, _, background)) = result {
                        let matching = match kind { 1 => search == 1, 3 => search == 2, 5 => search == 3, 8 => search == 4, _ => false };
                        assert_eq!(background.0 != 0xff000000, matching);
                    }
                }
            }
        }
        for (kind, id, color) in [(1, 500, 0xffeaa5f5), (3, 499, 0xfffdfea5), (5, 665, 0xffffbb33), (7, 741, 0xff8041e9), (8, 1273, 0xfffc7575)] {
            assert_eq!(style(kind, true, 0), Some((id, Argb(color), Argb(0xff000000))));
        }
    }
}
