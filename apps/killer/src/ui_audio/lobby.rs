use killer_ui::ini::IniFile;
use super::music_state::LobbyMusicRequest;
use super::AudioError;

pub const LOBBY_SOUND_KEYS: [&str; 8] = [
    "LogInSound", "CharacterSelect", "CharacterCreate", "CharacterHuman",
    "CharacterElf", "CharacterDarkElf", "CharacterOrc", "CharacterDwarf",
];
pub fn lobby_sound_names(alaudio: &IniFile) -> [String; 8] {
    LOBBY_SOUND_KEYS.map(|key| match alaudio.get("LobbySound", key) {
        Some(name) => String::from_utf16_lossy(&name.encode_utf16().take(255).collect::<Vec<_>>()),
        None => "intro".to_owned(),
    })
}

pub const CREDIT_SOUND_KEYS: [&str; 5] = ["CreditSound1", "CreditSound2", "CreditSound3", "CreditSound4", "CreditSound5"];
pub fn credit_sound_names(alaudio: &IniFile) -> [String; 5] {
    CREDIT_SOUND_KEYS.map(|key| match alaudio.get("Credit", key) {
        Some(name) => String::from_utf16_lossy(&name.encode_utf16().take(255).collect::<Vec<_>>()),
        None => "intro".to_owned(),
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct LobbyConsole {
    pub state_38: i32,
    pub previous_3c: i32,
    pub handle_40: i32,
    pub create_44: i32,
    pub selected_48: i32,
    pub names: [String; 8],
    pub credit_names: [String; 5],
    pub credit_handle: i32,
    pub credits_active: bool,
    pub credit_sound_volume: f32,
}

impl LobbyConsole {
    pub fn new(names: [String; 8]) -> Self {
        Self { state_38: 2, previous_3c: 0, handle_40: -1, create_44: -1, selected_48: -1, names, credit_names: std::array::from_fn(|_| "intro".to_owned()), credit_handle: 0, credits_active: false, credit_sound_volume: 0.0 }
    }

    pub fn set_state(&mut self, state: i32, audio: Option<&mut impl LobbyMusic>) -> Result<(), AudioError> {
        let mut result = Ok(());
        let music = match state {
            2 if self.state_38 != 2 => Some(0),
            4 if self.state_38 != 4 => Some(1),
            3 if self.state_38 != 3 => {
                self.create_44 = -1;
                (self.state_38 != 1).then_some(2)
            }
            7 => {
                self.previous_3c = self.state_38;
                self.state_38 = 7;
                return Ok(());
            }
            _ => None,
        };
        if let (Some(index), Some(audio)) = (music, audio) {
            if self.handle_40 > 0 {
                audio.stop_music(self.handle_40, 0.0);
            }
            let name = self.names[index].clone();
            match audio.play_music(LobbyMusicRequest { name: &name, fade: 1.5, looping: 1, voice: 0, volume: 1.0, force: 0 }) {
                Ok(handle) => self.handle_40 = handle,
                Err(error) => result = Err(error),
            }
            self.selected_48 = index as i32;
        }
        if self.state_38 == 7 { self.previous_3c = state; } else { self.state_38 = state; }
        result
    }
}

impl LobbyConsole {
    pub fn with_credit_names(mut self, names: [String; 5]) -> Self {
        self.credit_names = names;
        self
    }

    pub fn set_credits(&mut self, visible: bool, roll: u32, audio: Option<&mut impl LobbyMusic>) -> Result<(), AudioError> {
        let Some(audio) = audio else {
            self.credits_active = visible;
            return Ok(());
        };
        if visible && !self.credits_active {
            self.credits_active = true;
            if self.handle_40 > 0 {
                audio.stop_music(self.handle_40, 0.0);
            }
            self.handle_40 = -1;
            self.credit_sound_volume = audio.sound_volume();
            audio.set_sound_volume(0.0);
        }
        if visible {
            if audio.music_playing() {
                return Ok(());
            }
            if self.credit_handle != 0 {
                audio.stop_music(self.credit_handle, 0.0);
            }
            let name = self.credit_names[(roll % 5) as usize].clone();
            self.credit_handle = audio.play_music(LobbyMusicRequest { name: &name, fade: 1.5, looping: 0, voice: 0, volume: 1.0, force: 0 })?;
            return Ok(());
        }
        if self.credits_active {
            self.credits_active = false;
            if self.credit_handle != 0 {
                audio.stop_music(self.credit_handle, CREDIT_EXIT_FADE);
                let result = self.play_intro_music(Some(&mut *audio));
                audio.set_sound_volume(self.credit_sound_volume);
                return result;
            }
        }
        Ok(())
    }

    pub fn play_intro_music(&mut self, audio: Option<&mut impl LobbyMusic>) -> Result<(), AudioError> {
        let Some(audio) = audio else { return Ok(()) };
        if self.handle_40 >= 0 {
            return Ok(());
        }
        let name = self.names[0].clone();
        self.handle_40 = audio.play_music(LobbyMusicRequest { name: &name, fade: 1.5, looping: 1, voice: 0, volume: 1.0, force: 0 })?;
        self.selected_48 = 0;
        Ok(())
    }

    pub fn set_current_making_race(&mut self, race: i32, audio: Option<&mut impl LobbyMusic>) -> Result<(), AudioError> {
        if self.state_38 != 3 {
            return Ok(());
        }
        let choice = match race {
            0 => Some((3, 3)),
            1 => Some((4, 4)),
            2 => Some((5, 2)),
            3 => Some((6, 3)),
            4 => Some((7, 4)),
            _ => None,
        };
        let mut result = Ok(());
        if let (Some((index, selected)), Some(audio)) = (choice, audio)
            && self.create_44 != race
        {
            if self.handle_40 > 0 {
                audio.stop_music(self.handle_40, 0.0);
            }
            let name = self.names[index].clone();
            match audio.play_music(LobbyMusicRequest { name: &name, fade: 1.5, looping: 1, voice: 0, volume: 1.0, force: 0 }) {
                Ok(handle) => {
                    self.selected_48 = selected;
                    self.handle_40 = handle;
                }
                Err(error) => result = Err(error),
            }
        }
        self.create_44 = race;
        result
    }
}

pub const CREDIT_EXIT_FADE: f32 = 2.0;

pub trait LobbyMusic {
    fn music_playing(&self) -> bool;
    fn sound_volume(&self) -> f32;
    fn set_sound_volume(&mut self, volume: f32);
    fn stop_music(&mut self, handle: i32, fade: f32);
    fn play_music(&mut self, request: LobbyMusicRequest<'_>) -> Result<i32, AudioError>;
}

