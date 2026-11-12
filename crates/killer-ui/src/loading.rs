use crate::draw::DrawList;
use crate::ini::IniFile;

pub const LOADING_TEXTURE_UV: (i32, i32) = (1024, 768);
pub const LOADING_TEXTURE_KOREAN: [&str; 3] = ["L2Font.loading02-k", "L2Font.loading03-k", "L2Font.loading04-k"];
pub const LOADING_TEXTURE_KEY: &str = "LoadingTexture";

pub fn loading_texture(localization: &IniFile) -> Option<String> {
    let language = localization.get("LanguageSet", "Language")?;
    if language == "0" {
        return Some(LOADING_TEXTURE_KOREAN[0].to_string());
    }
    let country = localization.get("LanguageSet", language)?;
    localization.get(country, &format!("{country}_{LOADING_TEXTURE_KEY}")).map(str::to_string)
}

pub fn paint_loading(list: &mut DrawList, x: i32, y: i32, w: i32, h: i32, texture: &str) {
    let (uw, vh) = LOADING_TEXTURE_UV;
    list.draw_texture(x, y, w, h, 0, 0, uw, vh, texture, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ini::parse_ini;

    const LOCALIZATION: &str = include_str!("../tests/fixtures/localization.ini");

    #[test]
    fn the_english_client_uses_loading02_e_and_korean_falls_back_to_the_first_screen() {
        assert_eq!(loading_texture(&parse_ini(LOCALIZATION)).as_deref(), Some("L2Font-e.loading02-e"));
        assert_eq!(loading_texture(&parse_ini("[LanguageSet]\nLanguage=0\n")).as_deref(), Some("L2Font.loading02-k"));
        assert_eq!(loading_texture(&parse_ini("")), None);
    }

    #[test]
    fn the_screen_is_the_texture_stretched_over_the_viewport_with_a_1024_by_768_uv() {
        let mut list = DrawList::default();
        paint_loading(&mut list, 0, 0, 1558, 933, "L2Font-e.loading02-e");
        let q = &list.quads[0];
        assert_eq!((q.x, q.y, q.w, q.h, q.u, q.v, q.uw, q.vh), (0.0, 0.0, 1558.0, 933.0, 0.0, 0.0, 1024.0, 768.0));
        assert_eq!(q.texture, "L2Font-e.loading02-e");
        assert_eq!(q.color[3], 1.0);
    }
}
