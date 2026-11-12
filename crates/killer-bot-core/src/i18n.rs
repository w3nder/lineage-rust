#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Pt,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Text {
    Accounts,
    Status,
    Chat,
    Lists,
    Options,
    Npc,
    AddAccount,
    Connect,
    Disconnect,
    Start,
    Pause,
    Resume,
    Account,
    Character,
    Password,
    Server,
    Port,
    Hp,
    Mp,
    Cp,
    Target,
    NoTarget,
    Dead,
    Skills,
    Inventory,
    Buffs,
    Party,
    Nearby,
    Combat,
    Loot,
    Support,
    Follow,
    Send,
    Language,
    StateConnecting,
    StateChoosingServer,
    StateChoosingCharacter,
    StateEntering,
    StateInWorld,
    StateDisconnected,
}

impl Text {
    pub const ALL: [Text; 40] = [
        Text::Accounts,
        Text::Status,
        Text::Chat,
        Text::Lists,
        Text::Options,
        Text::Npc,
        Text::AddAccount,
        Text::Connect,
        Text::Disconnect,
        Text::Start,
        Text::Pause,
        Text::Resume,
        Text::Account,
        Text::Character,
        Text::Password,
        Text::Server,
        Text::Port,
        Text::Hp,
        Text::Mp,
        Text::Cp,
        Text::Target,
        Text::NoTarget,
        Text::Dead,
        Text::Skills,
        Text::Inventory,
        Text::Buffs,
        Text::Party,
        Text::Nearby,
        Text::Combat,
        Text::Loot,
        Text::Support,
        Text::Follow,
        Text::Send,
        Text::Language,
        Text::StateConnecting,
        Text::StateChoosingServer,
        Text::StateChoosingCharacter,
        Text::StateEntering,
        Text::StateInWorld,
        Text::StateDisconnected,
    ];
}

pub fn tr(lang: Lang, text: Text) -> &'static str {
    let (pt, en) = match text {
        Text::Accounts => ("Contas", "Accounts"),
        Text::Status => ("Estado", "Status"),
        Text::Chat => ("Chat", "Chat"),
        Text::Lists => ("Listas", "Lists"),
        Text::Options => ("Opções", "Options"),
        Text::Npc => ("NPC", "NPC"),
        Text::AddAccount => ("Adicionar conta", "Add account"),
        Text::Connect => ("Conectar", "Connect"),
        Text::Disconnect => ("Desconectar", "Disconnect"),
        Text::Start => ("Iniciar", "Start"),
        Text::Pause => ("Pausar", "Pause"),
        Text::Resume => ("Retomar", "Resume"),
        Text::Account => ("Conta", "Account"),
        Text::Character => ("Personagem", "Character"),
        Text::Password => ("Senha", "Password"),
        Text::Server => ("Servidor", "Server"),
        Text::Port => ("Porta", "Port"),
        Text::Hp => ("HP", "HP"),
        Text::Mp => ("MP", "MP"),
        Text::Cp => ("CP", "CP"),
        Text::Target => ("Alvo", "Target"),
        Text::NoTarget => ("Sem alvo", "No target"),
        Text::Dead => ("morto", "dead"),
        Text::Skills => ("Skills", "Skills"),
        Text::Inventory => ("Inventário", "Inventory"),
        Text::Buffs => ("Buffs", "Buffs"),
        Text::Party => ("Grupo", "Party"),
        Text::Nearby => ("Ao redor", "Nearby"),
        Text::Combat => ("Combate", "Combat"),
        Text::Loot => ("Coleta", "Loot"),
        Text::Support => ("Apoio", "Support"),
        Text::Follow => ("Seguir", "Follow"),
        Text::Send => ("Enviar", "Send"),
        Text::Language => ("Idioma", "Language"),
        Text::StateConnecting => ("Conectando", "Connecting"),
        Text::StateChoosingServer => ("Escolhendo servidor", "Choosing server"),
        Text::StateChoosingCharacter => ("Escolhendo personagem", "Choosing character"),
        Text::StateEntering => ("Entrando no mundo", "Entering the world"),
        Text::StateInWorld => ("No mundo", "In world"),
        Text::StateDisconnected => ("Desconectado", "Disconnected"),
    };
    match lang {
        Lang::Pt => pt,
        Lang::En => en,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_text_has_both_languages() {
        for text in Text::ALL {
            assert!(!tr(Lang::Pt, text).is_empty() && !tr(Lang::En, text).is_empty(), "{text:?}");
        }
        assert_ne!(tr(Lang::Pt, Text::Accounts), tr(Lang::En, Text::Accounts));
        assert_ne!(tr(Lang::Pt, Text::StateInWorld), tr(Lang::En, Text::StateInWorld));
    }
}
