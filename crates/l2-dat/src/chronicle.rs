use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Chronicle {
    #[default]
    C4,
    Interlude,
}

static OVERRIDE: OnceLock<Chronicle> = OnceLock::new();

impl Chronicle {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "c4" => Some(Self::C4),
            "interlude" | "c6" => Some(Self::Interlude),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::C4 => "c4",
            Self::Interlude => "interlude",
        }
    }

    pub fn set_override(chronicle: Self) {
        let _ = OVERRIDE.set(chronicle);
    }

    pub fn forced() -> Option<Self> {
        OVERRIDE.get().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_command_line_names() {
        assert_eq!(Chronicle::parse("c4"), Some(Chronicle::C4));
        assert_eq!(Chronicle::parse(" Interlude "), Some(Chronicle::Interlude));
        assert_eq!(Chronicle::parse("h5"), None);
        assert_eq!(Chronicle::default(), Chronicle::C4);
    }
}
