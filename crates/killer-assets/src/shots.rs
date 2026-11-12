use crate::options::Ini;

const GRADES: [&str; 6] = ["None", "D", "C", "B", "A", "S"];
const WEAPON_TYPES: [&str; 8] = ["HAND", "1HS", "2HS", "DUAL", "POLE", "BOW", "THROW", "DUALFIST"];
const STICK_MIN_LENGTH: f32 = 12.0;
const STICK_DEFAULT_VELOCITY: f32 = 30.0;
const LEFT_BOOK_YAW: i32 = 0x4000;
const HAND_NONE: i32 = -1;
const HAND_STICK: i32 = 1;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShotGrade {
    pub on_sticks: String,
    pub on_books: String,
    pub atk: String,
    pub atk_dark_cr: String,
    pub atk_cr: String,
    pub spirit_on_sticks: String,
    pub spirit_on_books: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SoulShotData {
    pub grades: [ShotGrade; 6],
    pub hands: [[i32; 2]; 8],
}

impl SoulShotData {
    pub fn parse(text: &str) -> Self {
        let ini = Ini::parse(text);
        Self {
            grades: GRADES.map(|grade| ShotGrade {
                on_sticks: ini.text(grade, "OnSticks"),
                on_books: ini.text(grade, "OnBooks"),
                atk: ini.text(grade, "Atk"),
                atk_dark_cr: ini.text(grade, "AtkDarkCr"),
                atk_cr: ini.text(grade, "AtkCr"),
                spirit_on_sticks: ini.text(grade, "SpiritOnSticks"),
                spirit_on_books: ini.text(grade, "SpiritOnBooks"),
            }),
            hands: WEAPON_TYPES.map(|kind| [ini.int(kind, "LH", 0), ini.int(kind, "RH", 0)]),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotKind {
    Soul,
    Spirit,
}

pub fn shot_skill(skill_id: i32) -> Option<(ShotKind, usize)> {
    match skill_id {
        2039 => Some((ShotKind::Soul, 0)),
        2150..=2154 => Some((ShotKind::Soul, ((skill_id - 2150) % 5 + 1) as usize)),
        2047 | 2061 => Some((ShotKind::Spirit, 0)),
        2155..=2164 => Some((ShotKind::Spirit, ((skill_id - 2155) % 5 + 1) as usize)),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShotEffect {
    pub class: String,
    pub right: bool,
    pub stick_velocity: Option<f32>,
    pub yaw: i32,
}

pub fn activation_effects(data: &SoulShotData, kind: ShotKind, grade: usize, weapon_type: usize, left: Option<&[f32]>, right: Option<&[f32]>) -> Vec<ShotEffect> {
    let (Some(grades), Some(hands)) = (data.grades.get(grade), data.hands.get(weapon_type)) else { return Vec::new() };
    let (sticks, books) = match kind {
        ShotKind::Soul => (&grades.on_sticks, &grades.on_books),
        ShotKind::Spirit => (&grades.spirit_on_sticks, &grades.spirit_on_books),
    };
    [(false, hands[0], left), (true, hands[1], right)]
        .into_iter()
        .filter_map(|(is_right, hand, lengths)| {
            let lengths = lengths?;
            let stick = hand == HAND_STICK && lengths.first().is_none_or(|length| *length > STICK_MIN_LENGTH);
            if stick {
                let velocity = lengths.first().copied().unwrap_or(STICK_DEFAULT_VELOCITY);
                return Some(ShotEffect { class: sticks.clone(), right: is_right, stick_velocity: Some(velocity), yaw: 0 });
            }
            (hand != HAND_NONE).then(|| ShotEffect { class: books.clone(), right: is_right, stick_velocity: None, yaw: if is_right { 0 } else { LEFT_BOOK_YAW } })
        })
        .filter(|effect| !effect.class.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INT: &str = "[None]\nOnSticks=LineageEffect.e_u505_a\nOnBooks=LineageEffect.e_u505_b\nSpiritOnSticks=LineageEffect.e_u512_a\nSpiritOnBooks=LineageEffect.e_u512_b\n[D]\nOnSticks=LineageEffect.e_u506_a\nOnBooks=LineageEffect.e_u506_b\n[HAND]\nLH=0\nRH=0\n[1HS]\nLH=-1\nRH=1\n[DUAL]\nLH=1\nRH=1\n[BOW]\nLH=0\nRH=-1\n";

    #[test]
    fn soulshot_data_load_10433f10_reads_grades_and_hands() {
        let data = SoulShotData::parse(INT);
        assert_eq!(data.grades[1].on_sticks, "LineageEffect.e_u506_a");
        assert_eq!(data.grades[0].spirit_on_books, "LineageEffect.e_u512_b");
        assert_eq!(data.hands[1], [-1, 1]);
        assert_eq!(data.hands[5], [0, -1]);
        assert_eq!(data.hands[2], [0, 0]);
    }

    #[test]
    fn spawn_n_transient_effect_104c84d0_picks_hand_stick_and_book() {
        let data = SoulShotData::parse(INT);
        assert_eq!(shot_skill(2039), Some((ShotKind::Soul, 0)));
        assert_eq!(shot_skill(2154), Some((ShotKind::Soul, 5)));
        assert_eq!(shot_skill(2061), Some((ShotKind::Spirit, 0)));
        assert_eq!(shot_skill(2160), Some((ShotKind::Spirit, 1)));
        assert_eq!(shot_skill(2164), Some((ShotKind::Spirit, 5)));
        assert_eq!(shot_skill(2181), None);
        let sword = [24.3, 7.0];
        let one_hand = activation_effects(&data, ShotKind::Soul, 1, 1, None, Some(&sword));
        assert_eq!(one_hand, vec![ShotEffect { class: "LineageEffect.e_u506_a".into(), right: true, stick_velocity: Some(24.3), yaw: 0 }]);
        let fist = [2.0, 2.0];
        let dual = activation_effects(&data, ShotKind::Soul, 1, 3, Some(&fist), Some(&[]));
        assert_eq!(dual, vec![
            ShotEffect { class: "LineageEffect.e_u506_b".into(), right: false, stick_velocity: None, yaw: 0x4000 },
            ShotEffect { class: "LineageEffect.e_u506_a".into(), right: true, stick_velocity: Some(30.0), yaw: 0 },
        ]);
        let hand = activation_effects(&data, ShotKind::Spirit, 0, 0, Some(&fist), Some(&fist));
        assert_eq!(hand.iter().map(|e| (e.class.as_str(), e.right, e.yaw)).collect::<Vec<_>>(), [("LineageEffect.e_u512_b", false, 0x4000), ("LineageEffect.e_u512_b", true, 0)]);
        assert!(activation_effects(&data, ShotKind::Soul, 1, 1, Some(&sword), None).is_empty());
        assert!(activation_effects(&data, ShotKind::Soul, 2, 1, None, Some(&sword)).is_empty());
        assert!(activation_effects(&data, ShotKind::Soul, 1, 9, None, Some(&sword)).is_empty());
    }
}
