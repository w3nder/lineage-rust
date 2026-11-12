use killer_bot_core::profile::Profile;
use std::path::Path;

pub fn file_name(label: &str) -> String {
    let clean: String = label.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    format!("{clean}.toml")
}

pub fn save_profile(dir: &Path, label: &str, profile: &Profile) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join(file_name(label)), profile.to_toml())
}

pub fn load_profiles(dir: &Path) -> Vec<(String, Result<Profile, String>)> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut found: Vec<(String, Result<Profile, String>)> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "toml"))
        .map(|entry| {
            let label = entry.path().file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default();
            let parsed = std::fs::read_to_string(entry.path()).map_err(|error| error.to_string()).and_then(|text| Profile::parse(&text).map_err(|error| error.to_string()));
            (label, parsed)
        })
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "account = \"conta\"\ncharacter = \"Wender\"\n[server]\nhost = \"h\"\nport = 1\n";

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("killer-bot-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn saved_profile_loads_back_and_has_no_password_key() {
        let dir = scratch("roundtrip");
        let profile = Profile::parse(TEXT).unwrap();
        save_profile(&dir, "Meu perfil", &profile).unwrap();
        let loaded = load_profiles(&dir);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].1.as_ref().unwrap(), &profile);
        let raw = std::fs::read_to_string(dir.join(file_name("Meu perfil"))).unwrap();
        assert!(!raw.to_lowercase().contains("password"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unreadable_profile_file_is_reported_not_fatal() {
        let dir = scratch("broken");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("ruim.toml"), "isto não é um perfil").unwrap();
        std::fs::write(dir.join("bom.toml"), TEXT).unwrap();
        let loaded = load_profiles(&dir);
        assert_eq!(loaded.len(), 2);
        assert!(loaded.iter().any(|(label, result)| label == "ruim" && result.is_err()));
        assert!(loaded.iter().any(|(label, result)| label == "bom" && result.is_ok()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_names_never_escape_the_folder() {
        assert_eq!(file_name("../../etc/passwd"), "______etc_passwd.toml");
        assert!(!load_profiles(Path::new("/nao/existe/mesmo")).iter().any(|_| true));
    }
}
