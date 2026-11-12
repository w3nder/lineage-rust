use super::AudioError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicController {
    Base,
    Lineage { field_a34: u32, flags_a38: u32 },
}

impl MusicController {
    pub fn reset_music_controll(&mut self) {
        if let Self::Lineage { field_a34, flags_a38 } = self {
            *flags_a38 |= 1;
            *field_a34 = 0;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LobbyMusicRequest<'a> {
    pub name: &'a str,
    pub fade: f32,
    pub looping: i32,
    pub voice: i32,
    pub volume: f32,
    pub force: i32,
}

pub fn restore_lobby_music<'a>(
    state_38: i32,
    selected_48: i32,
    names: &'a [String; 8],
    handle_40: &mut i32,
    mut play: impl FnMut(LobbyMusicRequest<'a>) -> Result<i32, AudioError>,
) -> Result<(), AudioError> {
    if state_38 == 6 || selected_48 < 0 {
        return Ok(());
    }
    let name = names.get(selected_48 as usize).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "índice de música do lobby fora da tabela original")
    })?;
    *handle_40 = play(LobbyMusicRequest { name, fade: 1.5, looping: 1, voice: 0, volume: 1.0, force: 0 })?;
    Ok(())
}
