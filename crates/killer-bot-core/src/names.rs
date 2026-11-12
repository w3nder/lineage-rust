#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemText {
    pub text: String,
    pub color: [u8; 3],
    pub group: u32,
}

pub trait Names {
    fn item(&self, id: i32) -> String;
    fn skill(&self, id: i32, level: i32) -> String;
    fn npc(&self, id: i32) -> String;
    fn class(&self, id: i32) -> String;
    fn find_skill(&self, name: &str) -> Option<i32>;
    fn system_message(&self, id: i32) -> Option<SystemText> {
        let _ = id;
        None
    }
    fn system_string(&self, id: i32) -> Option<String> {
        let _ = id;
        None
    }
    fn item_add_name(&self, id: i32) -> String {
        let _ = id;
        String::new()
    }
    fn item_grade(&self, id: i32) -> Option<&'static str> {
        let _ = id;
        None
    }
    fn castle(&self, id: i32) -> Option<String> {
        let _ = id;
        None
    }
    fn quest(&self, id: i32) -> Option<String> {
        let _ = id;
        None
    }
    fn server(&self, id: u8, flags: u32) -> Option<String> {
        let _ = (id, flags);
        None
    }
    fn quest_step(&self, id: i32, state: i32) -> Option<String> {
        let _ = (id, state);
        None
    }
    fn is_known_item(&self, id: i32) -> bool {
        let _ = id;
        true
    }
    fn command_token(&self, verb: &str) -> Option<u32> {
        let _ = verb;
        None
    }
    fn command_verb(&self, token: u32) -> Option<String> {
        let _ = token;
        None
    }
    fn action_social(&self, id: i32) -> i32 {
        let _ = id;
        0
    }
    fn action_name(&self, id: i32) -> Option<String> {
        let _ = id;
        None
    }
    fn action_icon(&self, id: i32) -> Option<String> {
        let _ = id;
        None
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IdNames;

impl Names for IdNames {
    fn item(&self, id: i32) -> String {
        format!("#{id}")
    }

    fn skill(&self, id: i32, _level: i32) -> String {
        format!("#{id}")
    }

    fn npc(&self, id: i32) -> String {
        format!("#{id}")
    }

    fn class(&self, id: i32) -> String {
        format!("#{id}")
    }

    fn find_skill(&self, _name: &str) -> Option<i32> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_data_the_names_are_the_ids() {
        let names = IdNames;
        assert_eq!(names.item(57), "#57");
        assert_eq!(names.skill(56, 3), "#56");
        assert_eq!(names.npc(20001), "#20001");
        assert_eq!(names.class(88), "#88");
        assert_eq!(names.find_skill("Spoil"), None);
    }
}
