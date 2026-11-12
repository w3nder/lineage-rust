use killer_assets::client::{Client, DatTables};
use killer_bot_core::names::Names;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const DEFAULT_CLIENT_URL: &str = "";
pub const NEEDED: [&str; 3] = ["system/", "systextures/icon.utx", "systextures/L2Font-e.utx"];

pub fn client_url(env: impl Fn(&str) -> Option<String>) -> String {
    env("KILLER_CLIENT_URL").filter(|value| !value.is_empty()).unwrap_or_else(|| DEFAULT_CLIENT_URL.to_owned())
}

pub fn data_dir(config: &Path, env: impl Fn(&str) -> Option<String>) -> PathBuf {
    env("KILLER_CLIENT_DIR").filter(|value| !value.is_empty()).map_or_else(|| config.join("dados"), PathBuf::from)
}

pub fn has_tables(root: &Path) -> bool {
    root.join("system").join("itemname-e.dat").exists() && root.join("systextures").join("icon.utx").exists()
}

pub enum DataState {
    Disabled,
    Downloading(killer_download::Download),
    Loading(std::sync::mpsc::Receiver<Result<GameData, String>>),
    Ready(Box<GameData>),
    Failed(String),
}

pub struct DataLoader {
    pub state: DataState,
    root: PathBuf,
    url: String,
    extra: Option<killer_download::Download>,
    reload: Option<std::sync::mpsc::Receiver<Result<GameData, String>>>,
    extras_checked: bool,
}

impl DataLoader {
    pub fn start(root: PathBuf, url: String) -> DataLoader {
        let mut loader = DataLoader { state: DataState::Disabled, root, url, extra: None, reload: None, extras_checked: false };
        loader.state = if has_tables(&loader.root) { loader.begin_loading() } else { loader.begin_download() };
        loader
    }

    fn begin_download(&self) -> DataState {
        DataState::Downloading(killer_download::Download::start_filtered(self.url.clone(), None, self.root.clone(), NEEDED.map(String::from).to_vec()))
    }

    fn begin_loading(&self) -> DataState {
        let (sender, receiver) = std::sync::mpsc::channel();
        let root = self.root.clone();
        std::thread::spawn(move || {
            let _ = sender.send(GameData::load(&root));
        });
        DataState::Loading(receiver)
    }

    pub fn poll(&mut self) {
        let next = match &self.state {
            DataState::Downloading(download) => match download.snapshot().phase {
                killer_download::Phase::Done => Some(self.begin_loading()),
                killer_download::Phase::Failed(message) => Some(DataState::Failed(message)),
                _ => None,
            },
            DataState::Loading(receiver) => match receiver.try_recv() {
                Ok(Ok(data)) => Some(DataState::Ready(Box::new(data))),
                Ok(Err(message)) => Some(DataState::Failed(message)),
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(DataState::Failed("a carga dos dados do jogo terminou sem resposta".into())),
            },
            _ => None,
        };
        if let Some(state) = next {
            self.state = state;
        }
        self.poll_extras();
    }

    fn poll_extras(&mut self) {
        if let Some(receiver) = &self.reload {
            match receiver.try_recv() {
                Ok(result) => {
                    self.reload = None;
                    if let Ok(data) = result {
                        self.state = DataState::Ready(Box::new(data));
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.reload = None,
            }
            return;
        }
        if let Some(download) = &self.extra {
            match download.snapshot().phase {
                killer_download::Phase::Done => {
                    self.extra = None;
                    let (sender, receiver) = std::sync::mpsc::channel();
                    let root = self.root.clone();
                    std::thread::spawn(move || {
                        let _ = sender.send(GameData::load(&root));
                    });
                    self.reload = Some(receiver);
                }
                killer_download::Phase::Failed(_) => self.extra = None,
                _ => {}
            }
            return;
        }
        if self.extras_checked {
            return;
        }
        let DataState::Ready(data) = &self.state else { return };
        self.extras_checked = true;
        let missing = data.missing_packages(&self.root);
        if !missing.is_empty() {
            let prefixes = missing.iter().flat_map(|package| [format!("systextures/{package}.utx"), format!("textures/{package}.utx")]).collect();
            self.extra = Some(killer_download::Download::start_filtered(self.url.clone(), None, self.root.clone(), prefixes));
        }
    }

    pub fn retry(&mut self) {
        if matches!(self.state, DataState::Failed(_)) {
            self.state = if has_tables(&self.root) { self.begin_loading() } else { self.begin_download() };
        }
    }

    pub fn ready(&mut self) -> Option<&mut GameData> {
        match &mut self.state {
            DataState::Ready(data) => Some(data),
            _ => None,
        }
    }

    pub fn status(&self, english: bool) -> String {
        match &self.state {
            DataState::Disabled => String::new(),
            DataState::Downloading(download) => {
                let snapshot = download.snapshot();
                let megabytes = |bytes: u64| bytes / (1024 * 1024);
                if english {
                    format!("Downloading game data: {}/{} MB", megabytes(snapshot.done_bytes), megabytes(snapshot.total_bytes))
                } else {
                    format!("Baixando dados do jogo: {}/{} MB", megabytes(snapshot.done_bytes), megabytes(snapshot.total_bytes))
                }
            }
            DataState::Loading(_) => (if english { "Loading game data..." } else { "Carregando dados do jogo..." }).to_owned(),
            DataState::Ready(_) => match &self.extra {
                Some(download) => {
                    let snapshot = download.snapshot();
                    let megabytes = |bytes: u64| bytes / (1024 * 1024);
                    if english {
                        format!("Downloading icons: {}/{} MB", megabytes(snapshot.done_bytes), megabytes(snapshot.total_bytes))
                    } else {
                        format!("Baixando ícones: {}/{} MB", megabytes(snapshot.done_bytes), megabytes(snapshot.total_bytes))
                    }
                }
                None if self.reload.is_some() => (if english { "Loading icons..." } else { "Carregando ícones..." }).to_owned(),
                None => String::new(),
            },
            DataState::Failed(message) => format!("{}: {message}", if english { "Game data failed" } else { "Falha nos dados do jogo" }),
        }
    }
}

pub struct TableNames {
    tables: Arc<DatTables>,
    servers: Arc<HashMap<u32, String>>,
}

pub const GRADES: [&str; 6] = ["", "D", "C", "B", "A", "S"];

impl Names for TableNames {
    fn item(&self, id: i32) -> String {
        u32::try_from(id).ok().and_then(|id| self.tables.itemname.get(&id)).filter(|row| !row.name.trim().is_empty()).map_or_else(|| format!("Item #{id}"), |row| row.name.clone())
    }

    fn item_add_name(&self, id: i32) -> String {
        u32::try_from(id).ok().and_then(|id| self.tables.itemname.get(&id)).map(|row| row.add_name.trim().to_owned()).unwrap_or_default()
    }

    fn item_grade(&self, id: i32) -> Option<&'static str> {
        let grade = self.tables.item_grade(u32::try_from(id).ok()?)?;
        GRADES.get(usize::try_from(grade).ok()?).copied().filter(|letter| !letter.is_empty())
    }

    fn is_known_item(&self, id: i32) -> bool {
        u32::try_from(id).ok().is_some_and(|id| self.tables.itemname.get(&id).is_some_and(|row| !row.name.trim().is_empty()))
    }

    fn skill(&self, id: i32, level: i32) -> String {
        let (Ok(skill), Ok(level)) = (u32::try_from(id), u32::try_from(level)) else { return format!("Skill #{id}") };
        self.tables
            .skillname
            .get(&(skill, level))
            .or_else(|| self.tables.skillname.get(&(skill, 1)))
            .or_else(|| self.tables.skillname.iter().filter(|((other, _), _)| *other == skill).min_by_key(|((_, other_level), _)| *other_level).map(|(_, row)| row))
            .map_or_else(|| format!("Skill #{id}"), |row| row.name.clone())
    }

    fn npc(&self, id: i32) -> String {
        u32::try_from(id).ok().and_then(|id| self.tables.npcname.get(&id)).filter(|row| !row.name.trim().is_empty()).map_or_else(|| format!("NPC #{id}"), |row| row.name.clone())
    }

    fn server(&self, id: u8, flags: u32) -> Option<String> {
        let index = if flags & 8 != 0 {
            126
        } else if flags & 4 != 0 {
            125
        } else {
            u32::from(id).saturating_sub(1)
        };
        self.servers.get(&index).cloned().filter(|name| !name.trim().is_empty())
    }

    fn quest_step(&self, id: i32, state: i32) -> Option<String> {
        let row = self.tables.questname.get(&(id, state))?;
        [row.sub_name.trim(), row.description.trim()].into_iter().find(|text| !text.is_empty()).map(str::to_owned)
    }

    fn castle(&self, id: i32) -> Option<String> {
        self.tables.castlename.get(&u32::try_from(id).ok()?).map(|castle| castle.name.clone()).filter(|name| !name.is_empty())
    }

    fn command_token(&self, verb: &str) -> Option<u32> {
        self.tables.commandname.get(verb).copied()
    }

    fn command_verb(&self, token: u32) -> Option<String> {
        self.tables.commandname.iter().find(|(_, entry)| **entry == token).map(|(name, _)| name.clone())
    }

    fn action_social(&self, id: i32) -> i32 {
        self.tables.actions.iter().find(|action| action.id as i32 == id).map_or(0, |action| action.kind)
    }

    fn action_name(&self, id: i32) -> Option<String> {
        self.tables.actions.iter().find(|action| action.id as i32 == id).map(|action| action.name.clone())
    }

    fn action_icon(&self, id: i32) -> Option<String> {
        self.tables.actions.iter().find(|action| action.id as i32 == id).map(|action| action.icon.clone()).filter(|icon| !icon.is_empty())
    }

    fn quest(&self, id: i32) -> Option<String> {
        self.tables.questname.iter().filter(|((quest, _), row)| *quest == id && !row.title.trim().is_empty()).min_by_key(|((_, state), _)| *state).map(|(_, row)| row.title.clone())
    }

    fn system_message(&self, id: i32) -> Option<killer_bot_core::names::SystemText> {
        u32::try_from(id).ok().and_then(|id| self.tables.systemmsg.get(&id)).map(|row| killer_bot_core::names::SystemText { text: row.text.clone(), color: row.color, group: row.group })
    }

    fn system_string(&self, id: i32) -> Option<String> {
        u32::try_from(id).ok().and_then(|id| self.tables.sysstring.get(&id)).cloned()
    }

    fn find_skill(&self, name: &str) -> Option<i32> {
        self.tables.skillname.iter().filter(|(_, row)| row.name.eq_ignore_ascii_case(name)).map(|((id, _), _)| *id as i32).min()
    }

    fn class(&self, id: i32) -> String {
        let Ok(class) = u32::try_from(id) else { return format!("Class #{id}") };
        self.tables.sysstring.get(&killer_ui::detail_status::class_name_sysstring(class)).cloned().unwrap_or_else(|| format!("Class #{id}"))
    }
}

pub struct GameData {
    pub tables: Arc<DatTables>,
    servers: Arc<HashMap<u32, String>>,
    client: Client,
    icons: HashMap<String, Option<Arc<Vec<u8>>>>,
}

impl GameData {
    pub fn load(root: &Path) -> Result<GameData, String> {
        let mut client = Client::open(root).map_err(|error| error.to_string())?;
        let tables = client.tables().map_err(|error| error.to_string())?;
        let servers: HashMap<u32, String> = client.server_names().map(|rows| rows.into_iter().map(|row| (row.id, row.name)).collect()).unwrap_or_default();
        Ok(GameData { tables: Arc::new(tables), servers: Arc::new(servers), client, icons: HashMap::new() })
    }

    pub fn missing_packages(&self, root: &Path) -> Vec<String> {
        let mut names: Vec<String> = vec![format!("{}1", killer_ui::macros::ICON_PREFIX)];
        names.extend(self.tables.item_icons.items.keys().filter_map(|id| self.tables.item_icons.texture(*id)));
        names.extend(self.tables.skillgrp.icons().map(str::to_owned));
        names.extend(self.tables.actions.iter().map(|action| action.icon.clone()));
        let mut packages: Vec<String> = names.iter().filter_map(|name| name.split_once('.').map(|(package, _)| package.to_owned())).filter(|package| !package.is_empty()).collect();
        packages.sort_by_key(|package| package.to_lowercase());
        packages.dedup_by_key(|package| package.to_lowercase());
        let present = |package: &str| ["systextures", "textures"].iter().any(|folder| [package.to_owned(), package.to_lowercase()].iter().any(|stem| root.join(folder).join(format!("{stem}.utx")).exists()));
        packages.into_iter().filter(|package| !present(package)).collect()
    }

    pub fn names(&self) -> TableNames {
        TableNames { tables: self.tables.clone(), servers: self.servers.clone() }
    }

    pub fn texture_png(&mut self, name: &str) -> Option<Arc<Vec<u8>>> {
        if let Some(cached) = self.icons.get(name) {
            return cached.clone();
        }
        let encoded = self.client.catalog().texture(name).and_then(|image| encode_png(image.width, image.height, &image.rgba)).map(Arc::new);
        self.icons.insert(name.to_owned(), encoded.clone());
        encoded
    }

    pub fn item_icon_name(&self, item_id: i32) -> Option<String> {
        self.tables.item_icons.texture(u32::try_from(item_id).ok()?)
    }

    pub fn skill_icon_name(&self, skill_id: i32, level: i32) -> Option<String> {
        let skill = self.tables.skillgrp.lookup(u32::try_from(skill_id).ok()?, u32::try_from(level).ok()?.max(1))?;
        (!skill.icon.is_empty()).then(|| skill.icon.clone())
    }
}

pub fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().ok()?;
    writer.write_image_data(rgba).ok()?;
    writer.finish().ok()?;
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn probe_icons() {
        let root = std::env::var("KILLER_CLIENT_DIR").unwrap();
        let mut data = GameData::load(Path::new(&root)).unwrap();
        let actions: Vec<(String, String)> = data.tables.actions.iter().map(|a| (a.name.clone(), a.icon.clone())).collect();
        let mut miss = 0;
        for (name, icon) in &actions {
            if data.texture_png(icon).is_none() {
                miss += 1;
                eprintln!("ACTION sem icone: {name} -> {icon:?}");
            }
        }
        eprintln!("ACTIONS {}/{} sem icone", miss, actions.len());
        for n in 1..=7 {
            let name = format!("L2UI.MacroWnd.Macro_Icon{n}");
            eprintln!("MACRO {name}: {}", data.texture_png(&name).is_some());
        }
        let ids: Vec<u32> = data.tables.itemname.keys().copied().collect();
        let (mut none_name, mut none_tex) = (0, 0);
        for id in &ids {
            match data.item_icon_name(*id as i32) {
                None => none_name += 1,
                Some(name) => {
                    if data.texture_png(&name).is_none() {
                        none_tex += 1;
                        if none_tex < 6 {
                            eprintln!("ITEM {id} textura {name:?} nao resolve");
                        }
                    }
                }
            }
        }
        eprintln!("ITEMS total {} sem nome de icone {} sem textura {}", ids.len(), none_name, none_tex);
        let skills: Vec<(u32, u32)> = data.tables.skillname.keys().copied().collect();
        let (mut s_none, mut s_tex) = (0, 0);
        for (id, level) in skills.iter().take(3000) {
            match data.skill_icon_name(*id as i32, *level as i32) {
                None => s_none += 1,
                Some(name) => {
                    if data.texture_png(&name).is_none() {
                        s_tex += 1;
                        if s_tex < 6 {
                            eprintln!("SKILL {id}/{level} textura {name:?} nao resolve");
                        }
                    }
                }
            }
        }
        eprintln!("PACOTES FALTANDO {:?}", data.missing_packages(Path::new(&root)));
        eprintln!("SKILLS amostra {} sem nome {} sem textura {}", skills.len().min(3000), s_none, s_tex);
    }

    #[test]
    fn the_client_url_defaults_to_the_public_bucket_and_the_environment_overrides_it() {
        assert_eq!(client_url(|_| None), DEFAULT_CLIENT_URL);
        assert_eq!(client_url(|_| Some(String::new())), DEFAULT_CLIENT_URL);
        assert_eq!(client_url(|name| (name == "KILLER_CLIENT_URL").then(|| "https://x.test/a".to_owned())), "https://x.test/a");
    }

    #[test]
    fn the_data_folder_is_inside_the_config_folder_unless_a_full_client_is_given() {
        let config = Path::new("/c/killer-bot");
        assert_eq!(data_dir(config, |_| None), config.join("dados"));
        assert_eq!(data_dir(config, |name| (name == "KILLER_CLIENT_DIR").then(|| "/jogo".to_owned())), PathBuf::from("/jogo"));
    }

    #[test]
    fn tables_are_detected_by_their_files_and_a_missing_folder_starts_a_download() {
        let dir = std::env::temp_dir().join(format!("killer-bot-dados-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(!has_tables(&dir));
        std::fs::create_dir_all(dir.join("system")).unwrap();
        std::fs::create_dir_all(dir.join("systextures")).unwrap();
        std::fs::write(dir.join("system/itemname-e.dat"), b"x").unwrap();
        assert!(!has_tables(&dir));
        std::fs::write(dir.join("systextures/icon.utx"), b"x").unwrap();
        assert!(has_tables(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_failed_load_reports_and_can_be_retried_without_losing_the_message() {
        let mut loader = DataLoader { state: DataState::Failed("sem rede".into()), root: std::env::temp_dir().join("killer-bot-inexistente-xyz"), url: "http://127.0.0.1:1/x".into(), extra: None, reload: None, extras_checked: false };
        assert!(loader.status(false).contains("sem rede"));
        assert!(loader.status(true).contains("failed"));
        loader.retry();
        assert!(matches!(loader.state, DataState::Downloading(_)));
        assert!(loader.status(false).starts_with("Baixando"));
    }

    #[test]
    fn only_tables_icons_and_map_textures_are_downloaded() {
        assert!(killer_download::wanted("system/itemname-e.dat", &NEEDED.map(String::from)));
        assert!(killer_download::wanted("systextures/Icon.utx", &NEEDED.map(String::from)));
        assert!(killer_download::wanted("systextures/L2Font-e.utx", &NEEDED.map(String::from)));
        assert!(!killer_download::wanted("systextures/L2UI_CH3.utx", &NEEDED.map(String::from)));
        assert!(!killer_download::wanted("maps/20_18.unr", &NEEDED.map(String::from)));
    }

    #[test]
    fn png_encoding_produces_a_png_signature() {
        let bytes = encode_png(1, 1, &[1, 2, 3, 255]).unwrap();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        assert!(encode_png(2, 2, &[0; 4]).is_none());
    }

    #[test]
    #[ignore]
    fn real_client_names_and_icons_resolve() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let mut data = GameData::load(&root).unwrap();
        let names = data.names();
        assert_ne!(names.item(57), "#57");
        assert_ne!(names.skill(1204, 1), "#1204");
        let npc = *data.tables.npcname.keys().next().unwrap();
        assert_ne!(names.npc(npc as i32), format!("#{npc}"));
        assert_ne!(names.class(0), "#0");
        let name = data.item_icon_name(57).unwrap();
        assert!(data.texture_png(&name).is_some());
        assert_eq!(names.item_add_name(10003), "Weapon Master");
        assert_eq!(names.item(9104), "Item #9104");
        assert!(!names.is_known_item(9104) && names.is_known_item(57));
        assert_eq!(names.server(1, 0).as_deref(), Some("Killer 70x"));
        assert_eq!(names.castle(1).as_deref(), Some("Gludio Castle"));
        assert_eq!(names.quest(1).as_deref(), Some("Letters of Love"));
        assert_eq!(names.quest_step(1, 1).as_deref(), Some("Delivery of Love Letters"));
        assert_eq!(names.skill(4408, 7), names.skill(4408, 1));
        let skill = data.skill_icon_name(1204, 1).unwrap();
        assert!(data.texture_png(&skill).is_some());
    }
}
