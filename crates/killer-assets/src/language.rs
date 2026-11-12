use crate::options::Ini;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientLanguage {
    pub language_type: i32,
    pub country: String,
    pub code: String,
    pub is_native: bool,
}

impl ClientLanguage {
    pub fn parse(localization: &str, option: &str) -> Self {
        let localization = Ini::parse(localization);
        let option = Ini::parse(option);
        let language_type = localization.int("LanguageSet", "Language", 0);
        let country = native_buffer(localization.get("LanguageSet", &language_type.to_string()).unwrap_or("Korea"));
        let code = native_buffer(localization.get(&country, &format!("{country}_Code")).unwrap_or("k"));
        Self { language_type, country, code, is_native: option.bool("Game", "IsNative", true) }
    }

    pub fn voice_suffix(&self) -> String {
        if !self.is_native {
            "-e".to_owned()
        } else if self.language_type != 0 {
            format!("-{}", self.code)
        } else {
            String::new()
        }
    }

    pub fn table_name(&self, base: &str) -> String {
        format!("{base}-{}.dat", if self.is_native { &self.code } else { "e" })
    }
}

pub const DEFAULT_LOGO: &str = "L2Font.start_logo-k";
pub const DEFAULT_MINI_LOGO: &str = "L2Font.start_minilogo-k";

pub fn logo_textures(localization: &str, country: &str) -> (String, String) {
    let ini = Ini::parse(localization);
    let logo = ini.get(country, &format!("{country}_LogoTexture")).unwrap_or(DEFAULT_LOGO);
    let mini = ini.get(country, &format!("{country}_MiniLogoTexture")).unwrap_or(DEFAULT_MINI_LOGO);
    (logo.to_owned(), mini.to_owned())
}

fn native_buffer(text: &str) -> String {
    String::from_utf16_lossy(&text.encode_utf16().take(255).collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_logos_come_from_the_country_keys_and_fall_back_to_the_korean_names_like_1009f7a0() {
        let ini = "[English]\nEnglish_LogoTexture=L2Font-e.start_logo-e\nEnglish_MiniLogoTexture=L2Font-e.mini_logo-e\n[Custom]\nCustom_LogoTexture=a.b\n";
        assert_eq!(logo_textures(ini, "English"), ("L2Font-e.start_logo-e".into(), "L2Font-e.mini_logo-e".into()));
        assert_eq!(logo_textures(ini, "Custom"), ("a.b".into(), DEFAULT_MINI_LOGO.into()));
        assert_eq!(logo_textures(ini, "Korea"), (DEFAULT_LOGO.into(), DEFAULT_MINI_LOGO.into()));
    }

    #[test]
    fn voice_files_take_the_english_suffix_unless_native_and_then_the_country_code_off_the_base_language() {
        let language = |native, language_type| ClientLanguage { language_type, country: "x".to_owned(), code: "cn".to_owned(), is_native: native };
        assert_eq!(language(false, 4).voice_suffix(), "-e");
        assert_eq!(language(true, 4).voice_suffix(), "-cn");
        assert_eq!(language(true, 0).voice_suffix(), "");
    }

    #[test]
    fn engine_init_10545ee2_reads_country_and_code_instead_of_hardcoding_language() {
        for (index, country, code) in [(0, "Korea", "k"), (1, "English", "e"), (2, "Japan", "j"),
            (3, "Taiwan", "tw"), (4, "China", "cn"), (5, "Thailand", "th"), (99, "Custom", "custom")] {
            let ini = format!("[LanguageSet]\nLanguage={index}\n{index}={country}\n[{country}]\n{country}_Code={code}\n");
            for native in [false, true] {
                let language = ClientLanguage::parse(&ini, &format!("[Game]\nIsNative={native}\n"));
                assert_eq!(language.language_type, index);
                assert_eq!(language.country, country);
                assert_eq!(language.code, code);
                for table in ["sysstring", "itemname", "npcname", "systemmsg", "questname", "zonename", "commandname"] {
                    assert_eq!(language.table_name(table), format!("{table}-{}.dat", if native { code } else { "e" }));
                }
            }
        }
    }

    #[test]
    fn engine_init_10545ee2_defaults_only_when_keys_are_absent() {
        let language = ClientLanguage::parse("", "");
        assert_eq!(language, ClientLanguage { language_type: 0, country: "Korea".into(), code: "k".into(), is_native: true });
        assert_eq!(ClientLanguage::parse("[LanguageSet]\nLanguage=-3\n", "").code, "k");
        let empty = ClientLanguage::parse("[LanguageSet]\n0=\n[]\n_Code=\n", "");
        assert_eq!(empty.country, "");
        assert_eq!(empty.table_name("sysstring"), "sysstring-.dat");
    }

    #[test]
    fn engine_init_10545f56_bounds_country_and_code_to_native_buffer() {
        let country = "x".repeat(256);
        let bounded = "x".repeat(255);
        let code = "a".repeat(256);
        let language = ClientLanguage::parse(&format!("[LanguageSet]\n0={country}\n[{bounded}]\n{bounded}_Code={code}\n"), "");
        assert_eq!(language.country, bounded);
        assert_eq!(language.code, "a".repeat(255));
    }
}
