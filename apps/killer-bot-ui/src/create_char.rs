use l2_net::game::NewCharacter;

pub fn class_id(race: usize, mage: bool) -> Option<i32> {
    Some(match (race, mage) {
        (0, false) => 0,
        (0, true) => 10,
        (1, false) => 18,
        (1, true) => 25,
        (2, false) => 31,
        (2, true) => 38,
        (3, false) => 44,
        (3, true) => 49,
        (4, false) => 53,
        _ => return None,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Choice {
    pub race: usize,
    pub female: bool,
    pub mage: bool,
    pub hair_style: i32,
    pub hair_color: i32,
    pub face: i32,
}

pub fn build(name: &str, choice: Choice) -> Result<NewCharacter, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("digite o nome".to_owned());
    }
    if name.len() > 16 {
        return Err("nome com 16 caracteres ou menos".to_owned());
    }
    let class_id = class_id(choice.race, choice.mage).ok_or("anão só pode ser Fighter")?;
    if !choice.female && choice.hair_style >= 5 {
        return Err("os estilos F e G são só para personagens femininos".to_owned());
    }
    Ok(NewCharacter {
        name: name.to_owned(),
        race: choice.race as i32,
        sex: i32::from(choice.female),
        class_id,
        int_: 0,
        str_: 0,
        con: 0,
        men: 0,
        dex: 0,
        wit: 0,
        hair_style: choice.hair_style,
        hair_color: choice.hair_color,
        face: choice.face,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: Choice = Choice { race: 0, female: false, mage: false, hair_style: 0, hair_color: 0, face: 0 };

    #[test]
    fn starting_classes_follow_the_client_table() {
        assert_eq!([class_id(0, false), class_id(0, true), class_id(1, false), class_id(1, true)], [Some(0), Some(10), Some(18), Some(25)]);
        assert_eq!([class_id(2, false), class_id(2, true), class_id(3, false), class_id(3, true)], [Some(31), Some(38), Some(44), Some(49)]);
        assert_eq!([class_id(4, false), class_id(4, true), class_id(5, false)], [Some(53), None, None]);
    }

    #[test]
    fn building_a_character_checks_name_class_and_female_only_styles() {
        let ok = build(" Novo ", Choice { race: 2, female: true, mage: true, hair_style: 6, hair_color: 3, face: 2 }).unwrap();
        assert_eq!((ok.name.as_str(), ok.race, ok.sex, ok.class_id, ok.hair_style, ok.hair_color, ok.face), ("Novo", 2, 1, 38, 6, 3, 2));
        assert_eq!(build("", BASE).unwrap_err(), "digite o nome");
        assert!(build("DezesseisLetrasXX", BASE).unwrap_err().contains("16"));
        assert!(build("Anao", Choice { race: 4, mage: true, ..BASE }).unwrap_err().contains("anão"));
        assert!(build("Homem", Choice { hair_style: 5, ..BASE }).unwrap_err().contains("femininos"));
        assert!(build("Mulher", Choice { female: true, hair_style: 5, ..BASE }).is_ok());
    }
}
