use crate::options::Ini;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct AutoLogOn {
    pub id: String,
    pub password: String,
}

impl std::fmt::Debug for AutoLogOn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutoLogOn").field("id", &"<oculto>").field("password", &"<oculto>").finish()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoginIni {
    pub use_key_crypt: bool,
    pub replay_mode: bool,
    pub test_server: bool,
    pub debug_log: bool,
    pub auto_logon: Option<AutoLogOn>,
}

impl LoginIni {
    pub fn parse(lineage2_ini: &str, auto_log: &str) -> LoginIni {
        let lineage2 = Ini::parse(lineage2_ini);
        let auto = Ini::parse(auto_log);
        let use_key_crypt = lineage2.bool("URL", "L2UseKeyCrypt", false) && lineage2.bool("URL", "IsL2NetLogin", false);
        let replay_mode = lineage2.bool("URL", "L2ReplayMode", false);
        let test_server = lineage2.bool("URL", "L2TestServer", false);
        let debug_log = lineage2.bool("URL", "L2Debug", false);
        let auto_logon = match (auto.bool("AutoLogOn", "IsL2AutoLogOn", false), auto.get("AutoLogOn", "L2ID"), auto.get("AutoLogOn", "L2Passwd")) {
            (true, Some(id), Some(password)) => Some(AutoLogOn { id: id.to_string(), password: password.to_string() }),
            _ => None,
        };
        LoginIni { use_key_crypt, replay_mode, test_server, debug_log, auto_logon }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_crypt_needs_both_flags_of_the_url_section() {
        assert!(LoginIni::parse("[URL]\nL2UseKeyCrypt=True\nIsL2NetLogin=True\n", "").use_key_crypt);
        assert!(!LoginIni::parse("[URL]\nL2UseKeyCrypt=True\nIsL2NetLogin=False\n", "").use_key_crypt);
        assert!(!LoginIni::parse("[URL]\nL2UseKeyCrypt=false\nIsL2NetLogin=True\n", "").use_key_crypt);
        assert!(!LoginIni::parse("", "").use_key_crypt);
        assert!(LoginIni::parse("[URL]\nL2ReplayMode=True\n", "").replay_mode);
        assert!(!LoginIni::parse("[URL]\nL2UseReplayManager=True\n", "").replay_mode);
        assert!(LoginIni::parse("[URL]\nL2Debug=True\n", "").debug_log);
        assert!(!LoginIni::parse("", "").debug_log);
        assert!(LoginIni::parse("[URL]\nL2TestServer=True\n", "").test_server);
        assert!(!LoginIni::parse("", "").test_server);
    }

    #[test]
    fn auto_logon_comes_only_from_the_auto_log_file_when_the_flag_is_on() {
        let on = "[AutoLogOn]\nIsL2AutoLogOn=True\nL2ID=conta\nL2Passwd=senha\n";
        assert_eq!(LoginIni::parse("", on).auto_logon, Some(AutoLogOn { id: "conta".into(), password: "senha".into() }));
        assert_eq!(LoginIni::parse(on, "").auto_logon, None);
        assert_eq!(LoginIni::parse("", "[AutoLogOn]\nIsL2AutoLogOn=Ture\nL2ID=a\nL2Passwd=b\n").auto_logon, None);
        assert_eq!(LoginIni::parse("", "[AutoLogOn]\nIsL2AutoLogOn=True\nL2ID=a\n").auto_logon, None);
    }

    #[test]
    fn the_debug_output_never_carries_the_credentials() {
        let text = format!("{:?}", AutoLogOn { id: "conta".into(), password: "senha".into() });
        assert!(!text.contains("conta") && !text.contains("senha"));
    }
}
