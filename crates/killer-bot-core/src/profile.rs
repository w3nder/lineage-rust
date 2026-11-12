use crate::options::BotOptions;
use serde::{Deserialize, Serialize};
use std::fmt;

pub const ORIGINAL_CLIENT_GAP_MS: u64 = 100;

#[derive(Debug, Clone, PartialEq)]
pub enum ProfileError {
    Parse(String),
    Invalid(String),
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfileError::Parse(text) => write!(f, "perfil ilegível: {text}"),
            ProfileError::Invalid(text) => write!(f, "perfil inválido: {text}"),
        }
    }
}

impl std::error::Error for ProfileError {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub server: ServerSection,
    pub account: String,
    #[serde(default)]
    pub character: String,
    #[serde(default)]
    pub server_id: Option<u8>,
    #[serde(default = "default_pacing")]
    pub pacing_ms: u64,
    #[serde(default = "default_true")]
    pub accept_notice: bool,
    #[serde(default)]
    pub options: BotOptions,
}

fn default_true() -> bool {
    true
}

fn default_pacing() -> u64 {
    ORIGINAL_CLIENT_GAP_MS
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServerSection {
    pub host: String,
    pub port: u16,
}

impl Profile {
    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
    }

    pub fn parse(text: &str) -> Result<Profile, ProfileError> {
        let profile: Profile = toml::from_str(text).map_err(|error| ProfileError::Parse(error.to_string()))?;
        profile.check()?;
        Ok(profile)
    }

    pub fn check(&self) -> Result<(), ProfileError> {
        let invalid = |text: &str| Err(ProfileError::Invalid(text.to_owned()));
        if self.account.trim().is_empty() {
            return invalid("a conta não pode ser vazia");
        }
        let options = &self.options;
        let percents = [
            options.rest.below_hp,
            options.rest.below_mp,
            options.rest.until_hp,
            options.rest.until_mp,
            options.alerts.hp_below,
            options.alerts.mp_below,
            options.alerts.cp_below,
            options.logout.hp_below,
            options.logout.mp_below,
            options.logout.cp_below,
        ];
        if percents.into_iter().flatten().any(|value| value > 100) {
            return invalid("porcentagem acima de 100");
        }
        if options.buffs_heals.iter().any(|row| row.below_percent > 100) || options.items.iter().any(|row| row.below_percent > 100) || options.toggles.iter().any(|row| row.below_percent > 100 || row.above_percent > 100) || options.combat.iter().any(|row| row.percent > 100) {
            return invalid("porcentagem acima de 100 em uma tabela de regras");
        }
        if options.combat.iter().any(|row| row.page == 0 || row.slot == 0 || row.slot > 12) {
            return invalid("atalho fora da barra (página a partir de 1, posição de 1 a 12)");
        }
        if options.dead.return_point > 4 {
            return invalid("ponto de retorno desconhecido");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{BuffHeal, CombatRule, Trait};

    const MINIMAL: &str = "account = \"a\"\n[server]\nhost = \"h\"\nport = 1\n";

    #[test]
    fn a_minimal_profile_parses_with_defaults() {
        let profile = Profile::parse(MINIMAL).unwrap();
        assert_eq!(profile.pacing_ms, 100);
        assert!(profile.accept_notice && profile.character.is_empty() && profile.server_id.is_none());
        assert_eq!(profile.options, BotOptions::default());
    }

    #[test]
    fn a_full_profile_round_trips_with_every_option_table() {
        let mut profile = Profile::parse(MINIMAL).unwrap();
        profile.character = "Wender".into();
        profile.server_id = Some(2);
        profile.options.attack.active_attack = true;
        profile.options.buffs_heals.push(BuffHeal { skill_id: 1204, trait_: Trait::Hp, ..Default::default() });
        profile.options.combat.push(CombatRule { page: 2, slot: 4, ..Default::default() });
        profile.options.do_not.npcs = vec![1, 2];
        let text = profile.to_toml();
        assert_eq!(Profile::parse(&text).unwrap(), profile);
        assert!(!text.to_lowercase().contains("password"));
    }

    #[test]
    fn rejects_a_password_field_and_unknown_option_keys() {
        assert!(matches!(Profile::parse(&format!("password = \"segredo\"\n{MINIMAL}")), Err(ProfileError::Parse(_))));
        assert!(matches!(Profile::parse(&format!("{MINIMAL}[options.attack]\nnao_existe = true\n")), Err(ProfileError::Parse(_))));
    }

    #[test]
    fn rejects_percentages_slots_and_return_points_out_of_range() {
        let with = |extra: &str| Profile::parse(&format!("{MINIMAL}{extra}"));
        assert!(matches!(with("[options.rest]\nbelow_hp = 101\n"), Err(ProfileError::Invalid(_))));
        assert!(matches!(with("[[options.combat]]\npercent = 120\n"), Err(ProfileError::Invalid(_))));
        assert!(matches!(with("[[options.combat]]\nslot = 13\n"), Err(ProfileError::Invalid(_))));
        assert!(matches!(with("[[options.combat]]\npage = 0\n"), Err(ProfileError::Invalid(_))));
        assert!(matches!(with("[options.dead]\nreturn_point = 9\n"), Err(ProfileError::Invalid(_))));
        assert!(matches!(Profile::parse("account = \"  \"\n[server]\nhost = \"h\"\nport = 1\n"), Err(ProfileError::Invalid(_))));
    }
}
