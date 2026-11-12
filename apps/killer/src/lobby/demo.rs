use crate::app::Screen;
use crate::lobby::{Lobby, LobbyScreen};
use l2_net::game::CharSelectEntry;
use l2_net::login::GameServer;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LobbyDemo {
    Login,
    Servers,
    Characters,
    Create,
}

pub fn parse(name: &str) -> Option<LobbyDemo> {
    match name.to_ascii_lowercase().as_str() {
        "login" => Some(LobbyDemo::Login),
        "servers" | "servidores" => Some(LobbyDemo::Servers),
        "characters" | "personagens" => Some(LobbyDemo::Characters),
        "create" | "criacao" | "criação" => Some(LobbyDemo::Create),
        _ => None,
    }
}

pub fn entry(
    name: &str,
    race: i32,
    sex: i32,
    class_id: i32,
    level: i32,
    delete_days: i32,
    equipped: [i32; 16],
) -> CharSelectEntry {
    CharSelectEntry {
        name: name.to_string(),
        char_id: level,
        login: String::new(),
        clan_id: 0,
        sex,
        race,
        base_class_id: class_id,
        active: 0,
        x: 0,
        y: 0,
        z: 0,
        hp: 100.0,
        mp: 50.0,
        sp: 0,
        exp: 0,
        level,
        karma: 0,
        paperdoll_objects: [0; 16],
        paperdoll_items: equipped,
        hair_style: level % 3,
        hair_color: level % 4,
        face: level % 3,
        max_hp: 100.0,
        max_mp: 50.0,
        delete_days,
        class_id,
        last_used: 0,
        enchant_effect: 0,
    }
}

pub fn characters() -> Vec<CharSelectEntry> {
    let mut armed = [0; 16];
    armed[7] = crate::characters::DEMO_RHAND;
    armed[10] = crate::characters::DEMO_CHEST;
    armed[11] = crate::characters::DEMO_LEGS;
    armed[12] = crate::characters::DEMO_FEET;
    armed[9] = crate::characters::DEMO_GLOVES;
    vec![
        entry("Canastra", 0, 0, 0, 52, 0, armed),
        entry("Sombra", 2, 1, 31, 20, 0, [0; 16]),
        entry("Noite", 1, 1, 25, 7, 3, [0; 16]),
    ]
}

pub fn servers() -> Vec<GameServer> {
    vec![
        GameServer { id: 1, ip: [127, 0, 0, 1], port: 7777, players: 12, max_players: 1000, online: true, extra: Default::default() },
        GameServer { id: 2, ip: [127, 0, 0, 1], port: 7778, players: 0, max_players: 500, online: false, extra: Default::default() },
    ]
}

pub fn apply(lobby: &mut Lobby, demo: LobbyDemo) -> Screen {
    match demo {
        LobbyDemo::Login => {
            lobby.enter(LobbyScreen::Login);
            Screen::Login
        }
        LobbyDemo::Servers => {
            lobby.set_servers(&servers());
            Screen::Servers
        }
        LobbyDemo::Characters => {
            lobby.set_characters(characters());
            lobby.select(0);
            Screen::Characters
        }
        LobbyDemo::Create => {
            lobby.set_characters(characters());
            lobby.enter(LobbyScreen::Create);
            lobby.create.race = Some(1);
            lobby.choose();
            Screen::Create
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_four_screens_in_both_languages() {
        assert_eq!(parse("Login"), Some(LobbyDemo::Login));
        assert_eq!(parse("servidores"), Some(LobbyDemo::Servers));
        assert_eq!(parse("characters"), Some(LobbyDemo::Characters));
        assert_eq!(parse("criação"), Some(LobbyDemo::Create));
        assert_eq!(parse("mundo"), None);
        assert_eq!(characters().len(), 3);
        assert_eq!(characters()[2].delete_days, 3);
    }
}
