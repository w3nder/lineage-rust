use std::path::PathBuf;

pub const DEFAULT_HOST: &str = "server.l2killer.com";
pub const DEFAULT_PORT: u16 = 2106;
const FILE_NAME: &str = "config.toml";
const APP_DIR: &str = "killer";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { host: DEFAULT_HOST.to_string(), port: DEFAULT_PORT }
    }
}

impl ServerConfig {
    pub fn path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        let base = if cfg!(target_os = "macos") {
            home?.join("Library/Application Support")
        } else if cfg!(target_os = "windows") {
            PathBuf::from(std::env::var_os("APPDATA")?)
        } else {
            match std::env::var_os("XDG_CONFIG_HOME") {
                Some(dir) => PathBuf::from(dir),
                None => home?.join(".config"),
            }
        };
        Some(base.join(APP_DIR).join(FILE_NAME))
    }

    pub fn parse(text: &str) -> Self {
        let mut config = Self::default();
        let mut in_login = true;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(section) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                in_login = section.trim() == "login";
                continue;
            }
            let Some((key, value)) = line.split_once('=') else { continue };
            if !in_login {
                continue;
            }
            let value = value.trim().trim_matches('"');
            match key.trim() {
                "host" if !value.is_empty() => config.host = value.to_string(),
                "port" => {
                    if let Ok(port) = value.parse() {
                        config.port = port;
                    }
                }
                _ => {}
            }
        }
        config
    }

    pub fn to_text(&self) -> String {
        format!("[login]\nhost = \"{}\"\nport = {}\n", self.host, self.port)
    }

    pub fn load() -> Self {
        Self::path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .map(|text| Self::parse(&text))
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "sem pasta de configuração"))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, self.to_text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_host_and_port_and_falls_back_to_the_defaults() {
        let parsed = ServerConfig::parse("# comentário\n[login]\nhost = \"127.0.0.1\"\nport = 2107\n");
        assert_eq!(parsed, ServerConfig { host: "127.0.0.1".into(), port: 2107 });
        assert_eq!(ServerConfig::parse(""), ServerConfig::default());
        assert_eq!(ServerConfig::parse("[login]\nport = abc\nhost = \"\"\n"), ServerConfig::default());
        assert_eq!(ServerConfig::parse("[outra]\nhost = \"x\"\n"), ServerConfig::default());
        assert_eq!(ServerConfig::parse("host = y\nport=1\n"), ServerConfig { host: "y".into(), port: 1 });
    }

    #[test]
    fn round_trips_through_the_text_form() {
        let config = ServerConfig { host: "server.l2killer.com".into(), port: 2106 };
        assert_eq!(config.to_text(), "[login]\nhost = \"server.l2killer.com\"\nport = 2106\n");
        assert_eq!(ServerConfig::parse(&config.to_text()), config);
        assert!(ServerConfig::path().is_some_and(|p| p.ends_with("killer/config.toml")));
    }
}
