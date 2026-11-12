use crate::candidate_box::CandidateBox;
use crate::composite_box::CompositeBox;
use crate::window::Rect;

pub const KIND_PASSWORD: u32 = 2;
pub const KIND_CHAT: u32 = 6;
pub const CANDIDATE_SIZE: (i32, i32) = (100, 0x87);
pub const CANDIDATE_PAGE: i32 = 9;
pub const CANDIDATE_ROW: i32 = 0xf;
pub const COMPOSITE_SIZE: (i32, i32) = (100, 0xf);
pub const JAPANESE_MODE: (u32, u32, u32) = (9, 8, 0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Korea,
    English,
    Japan,
    Taiwan,
    China,
    Thailand,
}

impl Language {
    pub fn from_code(code: i32) -> Option<Self> {
        Some(match code {
            0 => Self::Korea,
            1 => Self::English,
            2 => Self::Japan,
            3 => Self::Taiwan,
            4 => Self::China,
            5 => Self::Thailand,
            _ => return None,
        })
    }

    pub fn code(self) -> i32 {
        match self {
            Self::Korea => 0,
            Self::English => 1,
            Self::Japan => 2,
            Self::Taiwan => 3,
            Self::China => 4,
            Self::Thailand => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImeClass {
    Korean,
    Japanese,
    Taiwan,
    Chinese,
    Thai,
}

impl ImeClass {
    pub fn for_language(language: Language) -> Self {
        match language {
            Language::Korea | Language::English => Self::Korean,
            Language::Japan => Self::Japanese,
            Language::Taiwan => Self::Taiwan,
            Language::China => Self::Chinese,
            Language::Thailand => Self::Thai,
        }
    }

    pub fn vtable(self) -> u32 {
        match self {
            Self::Korean => 0x101a5b18,
            Self::Japanese => 0x101a5c70,
            Self::Taiwan => 0x101a5dd0,
            Self::Chinese => 0x101a5f30,
            Self::Thai => 0x101a6090,
        }
    }

    pub fn constructor(self) -> u32 {
        match self {
            Self::Korean => 0x1001d5a0,
            Self::Japanese => 0x1001d670,
            Self::Taiwan => 0x1001d740,
            Self::Chinese => 0x1001d810,
            Self::Thai => 0x1001d940,
        }
    }

    pub fn slot(self) -> usize {
        match self {
            Self::Korean => 0,
            Self::Japanese => 1,
            Self::Taiwan => 2,
            Self::Chinese => 3,
            Self::Thai => 4,
        }
    }

    pub fn has_candidate_box(self) -> bool {
        matches!(self, Self::Japanese | Self::Taiwan | Self::Chinese)
    }

    pub fn has_composite_box(self) -> bool {
        matches!(self, Self::Chinese)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ime {
    pub class: ImeClass,
    pub kind: u32,
    pub flags: u32,
    pub mode: u32,
    pub state: u32,
    pub candidate: Option<CandidateBox>,
    pub composite: Option<CompositeBox>,
}

impl Ime {
    fn base(class: ImeClass, kind: u32) -> Self {
        let mut ime = Self {
            class,
            kind,
            flags: 0,
            mode: 0,
            state: 0,
            candidate: None,
            composite: None,
        };
        if kind != KIND_CHAT && kind != KIND_PASSWORD {
            ime.flags |= 1;
            ime.state = 1;
        }
        ime
    }

    fn candidate_box() -> CandidateBox {
        let mut candidate = CandidateBox::new(Rect {
            x: 0,
            y: 0,
            w: CANDIDATE_SIZE.0,
            h: CANDIDATE_SIZE.1,
        });
        candidate.page = CANDIDATE_PAGE;
        candidate.row_height = CANDIDATE_ROW;
        candidate.on_create();
        candidate.visible = false;
        candidate
    }

    fn composite_box() -> CompositeBox {
        CompositeBox {
            rect: Rect {
                x: 0,
                y: 0,
                w: COMPOSITE_SIZE.0,
                h: COMPOSITE_SIZE.1,
            },
            background: None,
            text: Vec::new(),
            first: 0,
            caret: 0,
            caret_visible: false,
        }
    }

    pub fn on_create(class: ImeClass, kind: u32) -> Self {
        let mut ime = Self::base(class, kind);
        match class {
            ImeClass::Japanese => {
                ime.flags = JAPANESE_MODE.0;
                ime.mode = JAPANESE_MODE.1;
                ime.state = JAPANESE_MODE.2;
                ime.candidate = Some(Self::candidate_box());
            }
            ImeClass::Taiwan => ime.candidate = Some(Self::candidate_box()),
            ImeClass::Chinese => {
                ime.candidate = Some(Self::candidate_box());
                ime.composite = Some(Self::composite_box());
            }
            ImeClass::Korean | ImeClass::Thai => {}
        }
        ime
    }
}

pub fn thai_char_type(language: Language, ch: u32) -> i32 {
    if language != Language::Thailand {
        return 0;
    }
    match ch {
        0x20 => 1,
        0xe01..=0xe2f => 2,
        0xe33 => 3,
        0xe31 | 0xe34..=0xe3a | 0xe47 | 0xe4d => 4,
        0xe30 | 0xe32 | 0xe40..=0xe45 => 5,
        0xe3f | 0xe46 => 6,
        0xe50..=0xe59 => 7,
        0xe48..=0xe4b => 8,
        0xe4c => 9,
        _ => 0,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImeHolder {
    pub kind: u32,
    slots: [Option<Ime>; 5],
}

impl ImeHolder {
    pub fn create(language: Language, kind: u32) -> Self {
        let class = ImeClass::for_language(language);
        let mut slots: [Option<Ime>; 5] = [None, None, None, None, None];
        slots[class.slot()] = Some(Ime::on_create(class, kind));
        Self { kind, slots }
    }

    pub fn slot(&self, class: ImeClass) -> Option<&Ime> {
        self.slots[class.slot()].as_ref()
    }

    pub fn slot_mut(&mut self, class: ImeClass) -> Option<&mut Ime> {
        self.slots[class.slot()].as_mut()
    }

    pub fn state(&self) -> Option<i32> {
        if self.slots.iter().all(Option::is_none) {
            return None;
        }
        Some(i32::from(self.slots[0].is_some()))
    }

    pub fn paints_composition(&self) -> bool {
        self.state() == Some(0)
    }

    pub fn accepts_edit(&self) -> bool {
        self.state() != Some(0)
    }
}

pub fn edit_box_paints_composition(holder: Option<&ImeHolder>) -> bool {
    holder.is_some_and(ImeHolder::paints_composition)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Language; 6] = [
        Language::Korea,
        Language::English,
        Language::Japan,
        Language::Taiwan,
        Language::China,
        Language::Thailand,
    ];

    #[test]
    fn the_factory_of_1001da50_picks_the_class_and_slot_by_language() {
        let picked: Vec<(ImeClass, u32, u32)> = ALL
            .iter()
            .map(|l| {
                let c = ImeClass::for_language(*l);
                (c, c.constructor(), c.vtable())
            })
            .collect();
        assert_eq!(picked[0], (ImeClass::Korean, 0x1001d5a0, 0x101a5b18));
        assert_eq!(picked[1], picked[0]);
        assert_eq!(picked[2], (ImeClass::Japanese, 0x1001d670, 0x101a5c70));
        assert_eq!(picked[3], (ImeClass::Taiwan, 0x1001d740, 0x101a5dd0));
        assert_eq!(picked[4], (ImeClass::Chinese, 0x1001d810, 0x101a5f30));
        assert_eq!(picked[5], (ImeClass::Thai, 0x1001d940, 0x101a6090));
    }

    #[test]
    fn only_japan_taiwan_and_china_create_a_candidate_box_and_only_china_a_composite_box() {
        for language in ALL {
            let holder = ImeHolder::create(language, 0);
            let class = ImeClass::for_language(language);
            let ime = holder.slot(class).unwrap();
            assert_eq!(ime.candidate.is_some(), class.has_candidate_box());
            assert_eq!(ime.composite.is_some(), class.has_composite_box());
            assert_eq!(
                ime.candidate.is_some(),
                matches!(language, Language::Japan | Language::Taiwan | Language::China)
            );
        }
    }

    #[test]
    fn english_never_takes_the_composition_paint_branch_of_1000f080() {
        for language in [Language::Korea, Language::English] {
            let holder = ImeHolder::create(language, 0);
            assert_eq!(holder.state(), Some(1));
            assert!(!edit_box_paints_composition(Some(&holder)));
            assert!(holder.accepts_edit());
        }
        for language in [Language::Japan, Language::Taiwan, Language::China, Language::Thailand] {
            let holder = ImeHolder::create(language, 0);
            assert_eq!(holder.state(), Some(0));
            assert!(edit_box_paints_composition(Some(&holder)));
        }
        assert!(!edit_box_paints_composition(None));
    }

    #[test]
    fn an_untouched_holder_reports_minus_one_and_keeps_the_plain_path() {
        let holder = ImeHolder {
            kind: 0,
            slots: [None, None, None, None, None],
        };
        assert_eq!(holder.state(), None);
        assert!(!holder.paints_composition());
    }

    #[test]
    fn the_candidate_box_comes_with_nine_rows_of_fifteen_pixels() {
        let holder = ImeHolder::create(Language::China, 0);
        let ime = holder.slot(ImeClass::Chinese).unwrap();
        let candidate = ime.candidate.as_ref().unwrap();
        assert_eq!((candidate.page, candidate.row_height), (9, 0xf));
        assert_eq!((candidate.rect.w, candidate.rect.h), (100, 0x87));
        assert_eq!(candidate.textures.len(), 9);
        assert_eq!(ime.composite.as_ref().unwrap().rect.h, 0xf);
    }

    #[test]
    fn the_base_hides_the_system_ime_window_and_enables_conversion_except_for_chat_and_password() {
        assert_eq!(Ime::on_create(ImeClass::Korean, 0).flags, 1);
        assert_eq!(Ime::on_create(ImeClass::Korean, KIND_PASSWORD).flags, 0);
        assert_eq!(Ime::on_create(ImeClass::Korean, KIND_CHAT).state, 0);
        let japanese = Ime::on_create(ImeClass::Japanese, 0);
        assert_eq!((japanese.flags, japanese.mode, japanese.state), (9, 8, 0));
    }

    #[test]
    fn thai_char_type_10016120_classifies_only_in_the_thai_client() {
        let cases = [(0x20, 1), (0xe01, 2), (0xe2f, 2), (0xe33, 3), (0xe31, 4), (0xe3a, 4), (0xe47, 4), (0xe4d, 4), (0xe30, 5), (0xe45, 5), (0xe3f, 6), (0xe46, 6), (0xe55, 7), (0xe49, 8), (0xe4c, 9), (0x41, 0)];
        for (ch, kind) in cases {
            assert_eq!(thai_char_type(Language::Thailand, ch), kind, "{ch:#x}");
            assert_eq!(thai_char_type(Language::English, ch), 0);
        }
    }
}
