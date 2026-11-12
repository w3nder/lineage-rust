use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use l2_dat::symbolname::Symbols;

pub fn grade_symbol_1006b5a0(grade: i32) -> Option<&'static str> {
    match grade {
        1 => Some("graded"),
        2 => Some("gradec"),
        3 => Some("gradeb"),
        4 => Some("gradea"),
        5 => Some("grades"),
        _ => None,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemNameLabel {
    pub name: String,
    pub additional: Option<String>,
    pub type2: i32,
    pub enchant: i32,
    pub grade: i32,
    pub count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameLayout {
    pub name: String,
    pub additional: Option<(i32, String)>,
    pub symbol: Option<(i32, &'static str)>,
    pub count: Option<(i32, String)>,
}

impl ItemNameLabel {
    pub fn layout(&self, width: i32, font: &BitmapFont) -> NameLayout {
        let mut name = if self.enchant > 0 && (0..=3).contains(&self.type2) {
            format!("+{} {}", self.enchant, self.name)
        } else { self.name.clone() };
        let original = font.measure(&name) as i32;
        let additional = self.additional.as_ref().filter(|name| name.as_str() != "NAME_None");
        let after_add = original + additional.map_or(0, |name| font.measure(name) as i32 + 2);
        let symbol = (self.grade < 5).then(|| grade_symbol_1006b5a0(self.grade)).flatten();
        let after_symbol = after_add + if symbol.is_some() { 14 } else { 0 };
        let count = (self.count > 0).then(|| format!("x{}", self.count));
        let total = after_symbol + count.as_ref().map_or(0, |count| font.measure(count) as i32 + 2);
        let excess = total - width;
        let mut reduction = 0;
        if excess > 0 {
            let dots = font.measure("..") as i32;
            while name.pop().is_some() {
                reduction = original - dots - font.measure(&name) as i32;
                if reduction > excess { break; }
            }
            name.push_str("..");
        }
        NameLayout {
            name,
            additional: additional.map(|name| (original - reduction + 2, name.clone())),
            symbol: symbol.map(|alias| (after_add - reduction + 2, alias)),
            count: count.map(|count| (after_symbol - reduction + 2, count)),
        }
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), width: i32, font: &BitmapFont, symbols: &Symbols) {
        let layout = self.layout(width, font);
        let (x, y) = origin;
        let color = Argb(0xffdcdcdc);
        list.draw_text(x, y, color, &layout.name, font);
        if let Some((offset, name)) = layout.additional {
            list.draw_text(x + offset, y, Argb(0xffffd969), &name, font);
        }
        if let Some((offset, alias)) = layout.symbol
            && let Some(symbol) = symbols.by_alias(alias) {
            list.draw_texture(x + offset, y, 12, 12, 0, 0, 12, 12, &symbol.texture, 1.0);
            if let Some(quad) = list.quads.last_mut() { quad.color = color.rgba(); }
        }
        if let Some((offset, count)) = layout.count {
            list.draw_text(x + offset, y, color, &count, font);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font() -> BitmapFont {
        let glyphs = (32u8..=126).map(|c| (char::from(c), crate::font::GlyphRect { x: 0, width: 6, y: 0, height: 12 })).collect();
        BitmapFont::new("font".into(), crate::font::Glyphs { tex_width: 16, tex_height: 16, glyphs })
    }

    #[test]
    fn paint_item_1002c800_keeps_native_suffix_order_and_grade_bound() {
        let mut item = ItemNameLabel { name: "Sword".into(), additional: Some("SA".into()), type2: 0, enchant: 4, grade: 4, count: 2 };
        let layout = item.layout(200, &font());
        assert_eq!(layout.name, "+4 Sword");
        assert_eq!(layout.additional, Some((50, "SA".into())));
        assert_eq!(layout.symbol, Some((64, "gradea")));
        assert_eq!(layout.count, Some((78, "x2".into())));
        item.grade = 5;
        item.type2 = 4;
        item.additional = Some("NAME_None".into());
        let layout = item.layout(200, &font());
        assert_eq!(layout.name, "Sword");
        assert!(layout.additional.is_none() && layout.symbol.is_none());
        assert_eq!(layout.count, Some((32, "x2".into())));
    }

    #[test]
    fn paint_item_1002c800_truncates_strictly_beyond_the_excess() {
        let item = ItemNameLabel { name: "abcdefghij".into(), ..Default::default() };
        assert_eq!(item.layout(60, &font()).name, "abcdefghij");
        assert_eq!(item.layout(54, &font()).name, "abcdef..");
        let mut item = item;
        item.additional = Some(String::new());
        assert!(item.layout(100, &font()).additional.is_some());
    }

    #[test]
    fn draw_normal_text_1030f448_paints_symbols_as_twelve_pixel_textures() {
        let symbols = [l2_dat::symbolname::Symbol { id: 105, alias: "graded".into(), texture: "symbol.grade_d".into(), active: 1 }].into_iter().collect();
        let label = ItemNameLabel { name: "A".into(), grade: 1, ..Default::default() };
        let mut list = DrawList::default();
        label.paint(&mut list, (10, 20), 200, &font(), &symbols);
        let quad = list.quads.last().unwrap();
        assert_eq!(quad.texture, "symbol.grade_d");
        assert_eq!((quad.x, quad.y, quad.w, quad.h, quad.uw, quad.vh), (18.0, 20.0, 12.0, 12.0, 12.0, 12.0));
        assert_eq!(quad.color, Argb(0xffdcdcdc).rgba());
    }
}
