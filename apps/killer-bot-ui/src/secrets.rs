#[cfg(test)]
use std::collections::HashMap;
use std::path::PathBuf;
#[cfg(test)]
use std::sync::Mutex;

const SERVICE: &str = "killer-bot";
const PASSWORD_VARIABLE: &str = "KILLER_BOT_PASSWORD";

pub trait SecretStore {
    fn get(&self, account: &str) -> Option<String>;
    fn set(&self, account: &str, password: &str) -> Result<(), String>;
    fn forget(&self, account: &str);
}

pub struct KeyringStore;

impl SecretStore for KeyringStore {
    fn get(&self, account: &str) -> Option<String> {
        keyring::Entry::new(SERVICE, account).ok()?.get_password().ok()
    }

    fn set(&self, account: &str, password: &str) -> Result<(), String> {
        keyring::Entry::new(SERVICE, account)
            .and_then(|entry| entry.set_password(password))
            .map_err(|error| error.to_string())
    }

    fn forget(&self, account: &str) {
        if let Ok(entry) = keyring::Entry::new(SERVICE, account) {
            let _ = entry.delete_credential();
        }
    }
}

pub struct EnvStore<F: Fn(&str) -> Option<String>>(pub F);

impl<F: Fn(&str) -> Option<String>> SecretStore for EnvStore<F> {
    fn get(&self, _account: &str) -> Option<String> {
        (self.0)(PASSWORD_VARIABLE).filter(|value| !value.is_empty())
    }

    fn set(&self, _account: &str, _password: &str) -> Result<(), String> {
        Err("sem chaveiro: a senha só pode vir da variável de ambiente".to_owned())
    }

    fn forget(&self, _account: &str) {}
}

pub struct ChainStore(pub Vec<Box<dyn SecretStore>>);

impl SecretStore for ChainStore {
    fn get(&self, account: &str) -> Option<String> {
        self.0.iter().find_map(|store| store.get(account))
    }

    fn set(&self, account: &str, password: &str) -> Result<(), String> {
        let mut last = "nenhum armazenamento de senha disponível".to_owned();
        for store in &self.0 {
            match store.set(account, password) {
                Ok(()) => return Ok(()),
                Err(error) => last = error,
            }
        }
        Err(last)
    }

    fn forget(&self, account: &str) {
        for store in &self.0 {
            store.forget(account);
        }
    }
}

#[cfg(test)]
#[derive(Default)]
pub struct MemoryStore(Mutex<HashMap<String, String>>);

#[cfg(test)]
impl SecretStore for MemoryStore {
    fn get(&self, account: &str) -> Option<String> {
        self.0.lock().ok()?.get(account).cloned()
    }

    fn set(&self, account: &str, password: &str) -> Result<(), String> {
        self.0.lock().map_err(|error| error.to_string())?.insert(account.to_owned(), password.to_owned());
        Ok(())
    }

    fn forget(&self, account: &str) {
        if let Ok(mut map) = self.0.lock() {
            map.remove(account);
        }
    }
}

pub fn secret_key(host: &str, port: u16, account: &str) -> String {
    format!("{host}:{port}/{account}")
}

pub fn config_dir(env: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    let base = if cfg!(target_os = "macos") {
        PathBuf::from(env("HOME")?).join("Library/Application Support")
    } else if cfg!(target_os = "windows") {
        PathBuf::from(env("APPDATA")?)
    } else {
        match env("XDG_CONFIG_HOME") {
            Some(dir) => PathBuf::from(dir),
            None => PathBuf::from(env("HOME")?).join(".config"),
        }
    };
    Some(base.join("killer-bot"))
}

pub fn profile_dir(env: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    config_dir(env).map(|dir| dir.join("perfis"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_bot_core::profile::Profile;

    #[test]
    fn memory_store_round_trips_and_forgets() {
        let store = MemoryStore::default();
        assert_eq!(store.get("a"), None);
        store.set("a", "segredo").unwrap();
        assert_eq!(store.get("a"), Some("segredo".to_owned()));
        store.forget("a");
        assert_eq!(store.get("a"), None);
    }

    #[test]
    fn profile_files_never_contain_the_password() {
        let store = MemoryStore::default();
        let password = "senha-que-nao-pode-vazar";
        store.set(&secret_key("h", 1, "conta"), password).unwrap();
        let profile = Profile::parse("account = \"conta\"\ncharacter = \"Wender\"\n[server]\nhost = \"h\"\nport = 1\n").unwrap();
        let text = profile.to_toml();
        assert!(!text.contains(password));
        assert!(!text.to_lowercase().contains("password"));
    }

    #[test]
    fn chain_store_falls_back_to_the_environment_when_the_keychain_is_empty() {
        let chain = ChainStore(vec![Box::new(MemoryStore::default()), Box::new(EnvStore(|name: &str| (name == PASSWORD_VARIABLE).then(|| "da-env".to_owned())))]);
        assert_eq!(chain.get("a"), Some("da-env".to_owned()));
        chain.set("a", "do-chaveiro").unwrap();
        assert_eq!(chain.get("a"), Some("do-chaveiro".to_owned()));
        chain.forget("a");
        assert_eq!(chain.get("a"), Some("da-env".to_owned()));
        let only_env = ChainStore(vec![Box::new(EnvStore(|_: &str| None))]);
        assert!(only_env.set("a", "b").is_err());
    }

    #[test]
    fn env_store_reads_the_variable_only() {
        let store = EnvStore(|name: &str| (name == PASSWORD_VARIABLE).then(|| "x".to_owned()));
        assert_eq!(store.get("qualquer"), Some("x".to_owned()));
        assert!(store.set("a", "b").is_err());
        let empty = EnvStore(|_: &str| Some(String::new()));
        assert_eq!(empty.get("a"), None);
    }

    #[test]
    fn profile_dir_is_inside_the_system_config_folder() {
        let dir = profile_dir(|name| match name {
            "HOME" => Some("/home/u".to_owned()),
            "APPDATA" => Some("C:/u".to_owned()),
            _ => None,
        })
        .unwrap();
        assert!(dir.ends_with("killer-bot/perfis"));
        assert!(profile_dir(|_| None).is_none());
    }
}
