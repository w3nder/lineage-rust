use crate::lighting::SceneLighting;
use l2_catalog::Catalog;
use l2_dat::animnames::{self, AnimSet, decode_text};
use l2_dat::{
    Action, Armor, CharTemplate, EtcItem, HairAccessoryLoc, HairRow, ItemIcons, ItemKind, ItemName, Items, NpcName,
    HennaGrp, NpcTable, QuestName, SkillName, SkillTable, SystemMessage, Weapon, ZoneName, actionname, armorgrp, by_id,
    chargrp, commandname, etcitemgrp, hairaccessorylocgrp, hairgrp, helmetgrp, hennagrp, itemicons, itemname,
    mobskillanimgrp, npcgrp,
    npcname, questname, skillgrp, skillname, sysstring, symbolname, systemmsg, weapongrp, zonename,
};
use std::collections::{HashMap, HashSet};
use killer_platform::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;
use ue2_level::Level;
use ue2_package::{ObjectRef, Package};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TrailerClass {
    pub physics: u8,
    pub pre_pivot: bool,
    pub same_rotation: bool,
    pub relative_trail: bool,
    pub no_owner_destroy: bool,
}

pub const PHYS_TRAILER: u8 = 10;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClassRenderDefaults {
    pub lod_bias: Option<f32>,
    pub draw_scale: Option<f32>,
    pub draw_scale_3d: Option<[f32; 3]>,
}

#[derive(Debug, Error)]
pub enum AssetError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("decifra falhou: {0}")]
    Crypto(String),
    #[error("pacote inválido: {0}")]
    Package(String),
    #[error("não encontrado: {0}")]
    NotFound(String),
    #[error("tabela {0}: {1}")]
    Table(String, l2_dat::Error),
}

pub const ITEM_ICON_LOAD_FAILED: &str = "NWindow.BlackTexture";

#[derive(Default)]
pub struct DatTables {
    pub npcgrp: NpcTable,
    pub rotation_rates: crate::pawn_rotation::RotationRates,
    pub chargrp: Vec<CharTemplate>,
    pub hairgrp: Vec<HairRow>,
    pub helmetgrp: HashMap<u32, Vec<HairRow>>,
    pub hairaccessarygrp: HashMap<u32, Vec<HairRow>>,
    pub accessory_locs: HashMap<String, HairAccessoryLoc>,
    pub armorgrp: HashMap<u32, Armor>,
    pub weapongrp: HashMap<u32, Weapon>,
    pub etcitemgrp: HashMap<u32, EtcItem>,
    pub skillsound: Vec<l2_dat::skillsoundgrp::SkillSound>,
    pub eula: Option<l2_dat::eula::Eula>,
    pub warning_notice: Option<String>,
    pub credits: Vec<l2_dat::creditgrp::Credit>,
    pub questname: HashMap<(i32, i32), QuestName>,
    pub npcname: HashMap<u32, NpcName>,
    pub systemmsg: HashMap<u32, SystemMessage>,
    pub sysstring: HashMap<u32, String>,
    pub symbolname: symbolname::Symbols,
    pub hennagrp: HashMap<u32, HennaGrp>,
    pub recipes: HashMap<i32, l2_dat::recipe::Recipe>,
    pub recipe_order: Vec<i32>,
    pub skillgrp: SkillTable,
    pub zonename: Vec<ZoneName>,
    pub castlename: HashMap<u32, l2_dat::castlename::Castle>,
    pub staticobject: HashMap<u32, String>,
    pub anim_names: HashMap<String, AnimSet>,
    pub language_type: i32,
    pub logo_textures: (String, String),
    pub login_ini: crate::login_ini::LoginIni,
    pub item_icons: ItemIcons,
    pub itemname: HashMap<u32, ItemName>,
    pub skillname: HashMap<(u32, u32), SkillName>,
    pub mobskillanimgrp: HashMap<(u32, u32), String>,
    pub actions: Vec<Action>,
    pub commandname: HashMap<String, u32>,
    pub soulshot: crate::shots::SoulShotData,
    pub musicinfo: HashMap<u32, Vec<String>>,
    pub entereventgrp: HashMap<u32, l2_dat::entereventgrp::EnterEvent>,
    pub obscene: Vec<String>,
    pub search_order: SearchOrder,
    pub debug_ini: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SearchOrder {
    pub npc: HashMap<u32, u32>,
    pub item: HashMap<u32, u32>,
    pub skill: HashMap<(u32, u32), u32>,
}

impl DatTables {
    pub fn search_npc(&self, name: &str) -> Option<u32> {
        let first = |(key, npc): (&u32, &NpcName)| (npc.name == name).then(|| (self.search_order.npc.get(key).copied().unwrap_or(u32::MAX), npc.id));
        self.npcname.iter().filter_map(first).min().map(|(_, id)| id)
    }

    pub fn search_item(&self, name: &str) -> Option<u32> {
        let first = |(key, item): (&u32, &ItemName)| (item.name == name).then(|| (self.search_order.item.get(key).copied().unwrap_or(u32::MAX), item.id));
        self.itemname.iter().filter_map(first).min().map(|(_, id)| id)
    }

    pub fn search_skill(&self, name: &str) -> Option<u32> {
        let first = |(key, skill): (&(u32, u32), &SkillName)| (skill.name == name).then(|| (self.search_order.skill.get(key).copied().unwrap_or(u32::MAX), skill.id));
        self.skillname.iter().filter_map(first).min().map(|(_, id)| id)
    }

    pub fn item_grade(&self, id: u32) -> Option<i32> {
        if let Some(weapon) = self.weapongrp.get(&id) {
            Some(weapon.attributes_dc[0])
        } else if let Some(armor) = self.armorgrp.get(&id) {
            Some(armor.attributes_58c[1])
        } else {
            self.etcitemgrp.get(&id).map(|item| item.grade as i32)
        }
    }

    pub fn item_crystallizable(&self, id: u32) -> bool {
        if let Some(weapon) = self.weapongrp.get(&id) {
            weapon.crystallizable
        } else if let Some(armor) = self.armorgrp.get(&id) {
            armor.crystallizable
        } else {
            self.etcitemgrp.get(&id).is_some_and(|item| item.crystallizable)
        }
    }

    pub fn walk_anim_frames(&self) -> HashMap<usize, i32> {
        self.chargrp.iter().enumerate().map(|(row, template)| (row, template.walk_anim_frame as i32)).collect()
    }

    pub fn equip_sound(&self, id: i32) -> Option<&str> {
        let id = u32::try_from(id).ok()?;
        let name = if let Some(weapon) = self.weapongrp.get(&id) {
            &weapon.equip_sound
        } else if let Some(armor) = self.armorgrp.get(&id) {
            &armor.equip_sound
        } else {
            &self.etcitemgrp.get(&id)?.equip_sound
        };
        (!name.is_empty() && !name.eq_ignore_ascii_case("None")).then_some(name.as_str())
    }

    pub fn items(&self) -> Items<'_> {
        Items { armors: &self.armorgrp, weapons: &self.weapongrp }
    }

    pub fn pawn_tables(&self) -> crate::pawn::Tables<'_> {
        crate::pawn::Tables {
            items: self.items(),
            hairgrp: &self.hairgrp,
            helmetgrp: &self.helmetgrp,
            hairaccessarygrp: &self.hairaccessarygrp,
            accessory_locs: &self.accessory_locs,
        }
    }
}

const ANIM_NAME_FILES: [&str; 4] =
    ["lineagewarrior.int", "lineagemonster.int", "lineagemonster2.int", "lineagenpc.int"];
const LOCALIZATION_INI: &str = "localization.ini";
pub const LANGUAGE_KOREA: i32 = 0;

pub struct LoadedLevel {
    pub name: String,
    pub level: Level,
    pub lighting: SceneLighting,
}

static BASE_ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

pub fn set_base_root(root: PathBuf) {
    let _ = BASE_ROOT.set(root);
}

fn detect_chronicle(root: &Path) -> l2_dat::Chronicle {
    if find_file_by_name(&root.join("system"), "interface.xdat").is_some() {
        l2_dat::Chronicle::Interlude
    } else {
        l2_dat::Chronicle::C4
    }
}

pub struct Client {
    root: PathBuf,
    catalog: Catalog,
    script_packages: HashMap<String, Option<(Vec<u8>, Package)>>,
    chronicle: l2_dat::Chronicle,
    base: Option<(PathBuf, l2_dat::Chronicle)>,
}

impl Client {
    pub fn open(root: &Path) -> io::Result<Self> {
        if !killer_platform::is_dir(root) {
            return Err(io::Error::new(io::ErrorKind::NotFound, root.display().to_string()));
        }
        let base = BASE_ROOT.get().filter(|base| killer_platform::is_dir(base) && base.as_path() != root).map(|base| (base.clone(), detect_chronicle(base)));
        let mut catalog = match &base {
            Some((base, _)) => Catalog::open_with_fallback(root, base),
            None => Catalog::open(root),
        };
        if let Some(path) = find_file_by_name(&root.join("system"), "Engine.u")
            .or_else(|| base.as_ref().and_then(|(base, _)| find_file_by_name(&base.join("system"), "Engine.u")))
        {
            catalog.register_package("Engine", &path);
        }
        let chronicle = l2_dat::Chronicle::forced().unwrap_or_else(|| detect_chronicle(root));
        Ok(Self { root: root.to_path_buf(), catalog, script_packages: HashMap::new(), chronicle, base })
    }

    pub fn chronicle(&self) -> l2_dat::Chronicle {
        self.chronicle
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn catalog(&mut self) -> &mut Catalog {
        &mut self.catalog
    }

    pub fn level_path(&self, tile: &str) -> Result<PathBuf, AssetError> {
        find_file(&self.root.join("maps"), tile, "unr").ok_or_else(|| AssetError::NotFound(format!("maps/{tile}.unr")))
    }

    pub fn load_level(&mut self, tile: &str) -> Result<LoadedLevel, AssetError> {
        let path = self.level_path(tile)?;
        let raw = fs::read(&path)?;
        self.parse_level(tile, &path, &raw)
    }

    pub fn parse_level(&mut self, tile: &str, path: &Path, raw: &[u8]) -> Result<LoadedLevel, AssetError> {
        let file = l2_crypto::decrypt(raw, path).map_err(|e| AssetError::Crypto(format!("{e:?}")))?;
        let package = Package::parse(&file).map_err(|e| AssetError::Package(format!("{e:?}")))?;
        let level = ue2_level::read_level_in(&package, &file, Some(tile)).map_err(|e| AssetError::Package(format!("{e:?}")))?;
        self.catalog.register_package(tile, path);
        let lighting = SceneLighting::read(&package, &file);
        Ok(LoadedLevel { name: tile.to_string(), level, lighting })
    }

    pub fn sound(&mut self, name: &str) -> Option<(String, Vec<u8>)> {
        let (package, _) = name.split_once('.')?;
        if !package.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') { return None; }
        if self.catalog.package_status(package) == crate::PackageStatus::Missing {
            let file = format!("{package}.uax");
            let sounds = self.root.join("sounds").join(&file);
            let path = if sounds.exists() { sounds } else { self.root.join("voice").join(&file) };
            self.catalog.register_package(package, &path);
        }
        self.catalog.sound(name)
    }

    fn data_root(&self, name: &str) -> Option<&(PathBuf, l2_dat::Chronicle)> {
        let base = self.base.as_ref()?;
        (name.to_ascii_lowercase().ends_with(".dat") && find_file_by_name(&base.0.join("system"), name).is_some()).then_some(base)
    }

    fn chronicle_of(&self, name: &str) -> l2_dat::Chronicle {
        if let Some(base) = self.data_root(name) {
            return base.1;
        }
        match &self.base {
            Some(base) if find_file_by_name(&self.root.join("system"), name).is_none() => base.1,
            _ => self.chronicle,
        }
    }

    pub fn read_system_file(&mut self, name: &str) -> Result<Vec<u8>, AssetError> {
        let root = self.data_root(name).map_or_else(|| self.root.clone(), |base| base.0.clone());
        let path = find_file_by_name(&root.join("system"), name)
            .or_else(|| self.base.as_ref().and_then(|base| find_file_by_name(&base.0.join("system"), name)))
            .ok_or_else(|| AssetError::NotFound(format!("system/{name}")))?;
        let raw = fs::read(&path)?;
        let file = l2_crypto::decrypt(&raw, &path).map_err(|e| AssetError::Crypto(format!("{e:?}")))?;
        Ok(file.into_owned())
    }

    pub fn emitter_class(&mut self, path: &str) -> Option<ue2_level::Emitter> {
        let (package_name, class) = path.split_once('.')?;
        let key = package_name.to_ascii_lowercase();
        if !self.script_packages.contains_key(&key) {
            let loaded = self.read_system_file(&format!("{package_name}.u")).ok().and_then(|file| Package::parse(&file).ok().map(|package| (file, package)));
            self.script_packages.insert(key.clone(), loaded);
        }
        let (file, package) = self.script_packages.get(&key)?.as_ref()?;
        let export = package.exports().iter().find(|export| package.class_name(export) == "Class" && package.name(export.name).eq_ignore_ascii_case(class))?;
        ue2_level::class_emitter(package, file, export, Some(package_name)).ok()
    }

    pub fn class_render_defaults(&mut self, path: &str) -> Result<ClassRenderDefaults, AssetError> {
        let mut current = path.to_owned();
        let mut visited = HashSet::new();
        let mut result = ClassRenderDefaults::default();
        loop {
            if !visited.insert(current.to_ascii_lowercase()) {
                return Err(AssetError::Package(format!("herança cíclica: {current}")));
            }
            let (package_name, class) = current.split_once('.')
                .filter(|(package, class)| !package.is_empty() && !class.is_empty())
                .ok_or_else(|| AssetError::Package(format!("classe sem pacote: {current}")))?;
            let key = package_name.to_ascii_lowercase();
            if !matches!(self.script_packages.get(&key), Some(Some(_))) {
                let file = self.read_system_file(&format!("{package_name}.u"))?;
                let package = Package::parse(&file)
                    .map_err(|error| AssetError::Package(format!("{package_name}: {error:?}")))?;
                self.script_packages.insert(key.clone(), Some((file, package)));
            }
            let (file, package) = self.script_packages.get(&key).and_then(Option::as_ref)
                .ok_or_else(|| AssetError::NotFound(current.clone()))?;
            let export = package.exports().iter().find(|export| {
                package.class_name(export) == "Class" && package.name(export.name).eq_ignore_ascii_case(class)
            }).ok_or_else(|| AssetError::NotFound(current.clone()))?;
            let properties = ue2_assets::class_default_properties(package, file, export)
                .map_err(|error| AssetError::Package(format!("{current}: {error:?}")))?;
            for (name, target) in [("LODBias", &mut result.lod_bias), ("DrawScale", &mut result.draw_scale)] {
                if let Some(property) = ue2_assets::find(&properties, name) {
                    let value = property.float().ok_or_else(|| AssetError::Package(format!("{current}.{name}: float inválido")))?;
                    target.get_or_insert(value);
                }
            }
            if let Some(property) = ue2_assets::find(&properties, "DrawScale3D") {
                let value = property.vector().ok_or_else(|| AssetError::Package(format!("{current}.DrawScale3D: vetor inválido")))?;
                result.draw_scale_3d.get_or_insert(value);
            }
            current = match export.super_class {
                ObjectRef::Null => return Ok(result),
                ObjectRef::Export(_) => format!("{package_name}.{}", package.object_path(export.super_class)),
                ObjectRef::Import(_) => package.object_path(export.super_class),
            };
        }
    }

    pub fn read_help_text(&self, link: &str) -> Result<String, AssetError> {
        let normalized = link.replace('\\', "/");
        let prefix = "../l2text/";
        let name = match normalized.get(..prefix.len()) {
            Some(head) if head.eq_ignore_ascii_case(prefix) => &normalized[prefix.len()..],
            _ => &normalized,
        };
        if name.is_empty() || !name.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'.'))
            || !name.to_ascii_lowercase().ends_with(".htm")
        {
            return Err(AssetError::NotFound("caminho HTML fora de L2text".into()));
        }
        let path = self.root.join("L2text").join(name);
        let raw = fs::read(&path)?;
        let file = l2_crypto::decrypt(&raw, &path).map_err(|e| AssetError::Crypto(e.to_string()))?;
        Ok(decode_text(&file))
    }

    pub fn read_system_text(&mut self, name: &str) -> Result<String, AssetError> {
        #[cfg(target_arch = "wasm32")]
        if let Some(text)=killer_platform::setting(name) {return Ok(text);}
        Ok(decode_text(&self.read_system_file(name)?))
    }

    pub fn class_boolean_defaults(&mut self, path: &str, names: &[&str]) -> Result<HashMap<String,bool>,AssetError> {
        self.class_property_defaults(path,names,|property,_| property.bool())
    }

    pub fn class_byte_defaults(&mut self, path: &str, names: &[&str]) -> Result<HashMap<String,u8>,AssetError> {
        self.class_property_defaults(path,names,|property,_| property.byte())
    }

    /// Propriedades de `AActor::physTrailer` (0x105fe4c0) lidas dos padrões da classe: `Physics` (10 = trailer, o ramo
    /// `cmp cl, 0xa`) e os bits `bTrailerPrePivot` (0x40000), `bTrailerSameRotation` (0x20000), `bRelativeTrail`
    /// (0x100000) e `bTrailerNoOwnerDestroy` (0x80000) testados nesse ramo.
    pub fn trailer_class(&mut self, path: &str) -> Result<TrailerClass,AssetError> {
        let flags = self.class_boolean_defaults(path,&["bTrailerPrePivot","bTrailerSameRotation","bRelativeTrail","bTrailerNoOwnerDestroy"])?;
        let physics = self.class_byte_defaults(path,&["Physics"])?;
        let flag = |name: &str| flags.get(name).copied().unwrap_or(false);
        Ok(TrailerClass {
            physics: physics.get("Physics").copied().unwrap_or(0),
            pre_pivot: flag("bTrailerPrePivot"),
            same_rotation: flag("bTrailerSameRotation"),
            relative_trail: flag("bRelativeTrail"),
            no_owner_destroy: flag("bTrailerNoOwnerDestroy"),
        })
    }

    pub fn class_int_defaults(&mut self, path: &str, names: &[&str]) -> Result<HashMap<String,i32>,AssetError> {
        self.class_property_defaults(path,names,|property,_| property.int())
    }

    pub fn class_vector_defaults(&mut self, path: &str, names: &[&str]) -> Result<HashMap<String,[f32;3]>,AssetError> {
        self.class_property_defaults(path,names,|property,_| property.vector())
    }

    pub fn class_rotator_defaults(&mut self, path: &str, names: &[&str]) -> Result<HashMap<String,[i32;3]>,AssetError> {
        self.class_property_defaults(path,names,|property,_| property.rotator())
    }

    pub fn class_float_defaults(&mut self, path: &str, names: &[&str]) -> Result<HashMap<String,f32>,AssetError> {
        self.class_property_defaults(path,names,|property,_| property.float())
    }

    pub fn class_name_defaults(&mut self, path: &str, names: &[&str]) -> Result<HashMap<String,String>,AssetError> {
        self.class_property_defaults(path,names,|property,package| property.name_value(package).map(str::to_owned))
    }

    pub fn pawn_class_defaults(&mut self, path: &str) -> Result<crate::damage_act::PawnDefaults,AssetError> {
        let flags = self.class_boolean_defaults(path,&["bSpineRotation"])?;
        let mut names = self.class_name_defaults(path,&["SpineBone","HeadBone"])?;
        Ok(crate::damage_act::PawnDefaults {
            spine_rotation: flags.get("bSpineRotation").copied().unwrap_or(false),
            spine_bone: names.remove("SpineBone"),
            head_bone: names.remove("HeadBone"),
        })
    }

    pub fn pawn_rotation_defaults(&mut self, path: &str) -> Result<crate::pawn_rotation::RotationDefaults,AssetError> {
        let rotators = self.class_rotator_defaults(path,&["RotationRate","KayboardRotationRate"])?;
        let flags = self.class_boolean_defaults(path,&["bRotateToDesired"])?;
        let floats = self.class_float_defaults(path,&["AccelRate"])?;
        Ok(crate::pawn_rotation::RotationDefaults {
            accel_rate: floats.get("AccelRate").copied().unwrap_or_default(),
            rate: rotators.get("RotationRate").copied().unwrap_or_default(),
            keyboard_rate: rotators.get("KayboardRotationRate").copied().unwrap_or_default(),
            rotate_to_desired: flags.get("bRotateToDesired").copied().unwrap_or(false),
        })
    }

    pub fn rotation_rates(&mut self, npcs: &NpcTable) -> crate::pawn_rotation::RotationRates {
        let mut classes: HashMap<String,Option<i32>> = HashMap::new();
        let mut yaw_rate = |client: &mut Self, name: &str| {
            *classes.entry(name.to_owned()).or_insert_with(|| match client.pawn_rotation_defaults(name) {
                Ok(defaults) => Some(defaults.rate[1]),
                Err(error) => {
                    log::warn!("RotationRate de {name}: {error}");
                    None
                }
            })
        };
        let players = (0..).map_while(|row| crate::pawn::ClassSource::Player { row }.name().map(|name| (row,name)))
            .filter_map(|(row,name)| yaw_rate(self,&name).map(|rate| (row,rate))).collect();
        let npcs = npcs.iter().filter(|npc| !npc.class.eq_ignore_ascii_case("None"))
            .filter_map(|npc| yaw_rate(self,&npc.class).map(|rate| (npc.id,rate))).collect();
        crate::pawn_rotation::RotationRates { players, npcs }
    }

    fn class_property_defaults<T>(&mut self, path: &str, names: &[&str], read: impl for<'p> Fn(&ue2_assets::Property<'p>, &'p Package) -> Option<T>) -> Result<HashMap<String,T>,AssetError> {
        let mut current = path.to_owned();
        let mut visited = HashSet::new();
        let mut result = HashMap::new();
        loop {
            if !visited.insert(current.to_ascii_lowercase()) {
                return Err(AssetError::Package(format!("herança cíclica: {current}")));
            }
            let (package_name,class) = current.split_once('.')
                .filter(|(package,class)| !package.is_empty() && !class.is_empty())
                .ok_or_else(|| AssetError::Package(format!("classe sem pacote: {current}")))?;
            let key = package_name.to_ascii_lowercase();
            if !matches!(self.script_packages.get(&key),Some(Some(_))) {
                let file = self.read_system_file(&format!("{package_name}.u"))?;
                let package = Package::parse(&file).map_err(|e| AssetError::Package(format!("{package_name}: {e:?}")))?;
                self.script_packages.insert(key.clone(),Some((file,package)));
            }
            let (file,package) = self.script_packages.get(&key).and_then(Option::as_ref)
                .ok_or_else(|| AssetError::NotFound(current.clone()))?;
            let export = package.exports().iter().find(|export| package.class_name(export)=="Class" && package.name(export.name).eq_ignore_ascii_case(class))
                .ok_or_else(|| AssetError::NotFound(current.clone()))?;
            let properties = ue2_assets::class_default_properties(package,file,export)
                .map_err(|e| AssetError::Package(format!("{current}: {e:?}")))?;
            for &name in names {
                if let Some(property) = ue2_assets::find(&properties,name) {
                    let value = read(property,package).ok_or_else(|| AssetError::Package(format!("{current}.{name}: tipo inválido")))?;
                    result.entry(name.to_owned()).or_insert(value);
                }
            }
            current = match export.super_class {
                ObjectRef::Null => return Ok(result),
                ObjectRef::Export(_) => format!("{package_name}.{}",package.object_path(export.super_class)),
                ObjectRef::Import(_) => package.object_path(export.super_class),
            };
        }
    }

    pub fn class_is_a(&mut self, path: &str, base: &str) -> Result<bool,AssetError> {
        let mut current = path.to_owned();
        let mut visited = HashSet::new();
        loop {
            if current.eq_ignore_ascii_case(base) { return Ok(true); }
            if !visited.insert(current.to_ascii_lowercase()) {
                return Err(AssetError::Package(format!("herança cíclica: {current}")));
            }
            let (package_name,class) = current.split_once('.')
                .filter(|(package,class)| !package.is_empty() && !class.is_empty())
                .ok_or_else(|| AssetError::Package(format!("classe sem pacote: {current}")))?;
            let key = package_name.to_ascii_lowercase();
            if !matches!(self.script_packages.get(&key),Some(Some(_))) {
                let file = self.read_system_file(&format!("{package_name}.u"))?;
                let package = Package::parse(&file).map_err(|e| AssetError::Package(format!("{package_name}: {e:?}")))?;
                self.script_packages.insert(key.clone(),Some((file,package)));
            }
            let (_,package) = self.script_packages.get(&key).and_then(Option::as_ref)
                .ok_or_else(|| AssetError::NotFound(current.clone()))?;
            let export = package.exports().iter().find(|export| package.class_name(export)=="Class" && package.name(export.name).eq_ignore_ascii_case(class))
                .ok_or_else(|| AssetError::NotFound(current.clone()))?;
            current = match export.super_class {
                ObjectRef::Null => return Ok(false),
                ObjectRef::Export(_) => format!("{package_name}.{}",package.object_path(export.super_class)),
                ObjectRef::Import(_) => package.object_path(export.super_class),
            };
        }
    }

    pub fn write_system_text(&mut self, name: &str, text: &str) -> Result<(), AssetError> {
        #[cfg(target_arch = "wasm32")]
        { killer_platform::set_setting(name,text.into()).map_err(|e|std::io::Error::new(std::io::ErrorKind::PermissionDenied,e))?; Ok(()) }
        #[cfg(not(target_arch = "wasm32"))]
        {
        let system = self.root.join("system");
        let path = find_file_by_name(&system, name).unwrap_or_else(|| system.join(name));
        fs::write(&path, text.as_bytes())?;
        Ok(())
        }
    }

    pub fn class_info(&mut self) -> Result<Vec<l2_dat::ClassInfo>, AssetError> {
        self.table("classinfo-e.dat", l2_dat::classinfo::read)
    }

    pub fn server_names(&mut self) -> Result<Vec<l2_dat::ServerName>, AssetError> {
        self.table("servername-e.dat", l2_dat::servername::read)
    }

    pub fn idc_names(&mut self) -> Result<Vec<l2_dat::ServerName>, AssetError> {
        self.table("idcname-e.dat", l2_dat::servername::read)
    }

    pub fn logongrp(&mut self) -> Result<l2_dat::Logongrp, AssetError> {
        self.table("logongrp.dat", l2_dat::logongrp::read)
    }

    pub fn tables(&mut self) -> Result<DatTables, AssetError> {
        let language = self.client_language();
        self.tables_for_language(&language)
    }

    pub fn client_language(&mut self) -> crate::language::ClientLanguage {
        let localization = self.base.as_ref().and_then(|base| find_file_by_name(&base.0.join("system"), LOCALIZATION_INI)).and_then(|path| {
            let raw = fs::read(&path).ok()?;
            Some(decode_text(&l2_crypto::decrypt(&raw, &path).ok()?))
        });
        let localization = match localization {
            Some(text) => text,
            None => self.read_system_text(LOCALIZATION_INI).unwrap_or_default(),
        };
        crate::language::ClientLanguage::parse(
            &localization,
            &self.read_system_text("option.ini").unwrap_or_default(),
        )
    }

    fn black_texture_for_unloadable_icons(&mut self, icons: &mut ItemIcons) {
        for icon in icons.items.values_mut() {
            if let Some(name) = icon.texture(0)
                && self.catalog().object_class(&name).is_none()
            {
                icon.icons[0] = ITEM_ICON_LOAD_FAILED.to_owned();
            }
        }
    }

    pub fn tables_for_language(&mut self, language: &crate::language::ClientLanguage) -> Result<DatTables, AssetError> {
        let recipes = self.table("Recipe-c.dat", l2_dat::recipe::read)?;
        let (weapons, weapon_icons) = self.table_with_icons_as("weapongrp.dat", weapongrp::read_as, ItemKind::Weapon)?;
        let (armors, armor_icons) = self.table_with_icons("armorgrp.dat", armorgrp::read, ItemKind::Armor)?;
        let (etc_items, etc_icons) = self.table_with_icons("etcitemgrp.dat", etcitemgrp::read, ItemKind::Etc)?;
        let mut item_icons = ItemIcons::default();
        for icons in [etc_icons, armor_icons, weapon_icons] { item_icons.insert_all(icons); }
        self.black_texture_for_unloadable_icons(&mut item_icons);
        let weapongrp = by_id(weapons, |w| w.id);
        let mut armorgrp = by_id(armors, |a| a.id);
        armorgrp.retain(|id, _| !weapongrp.contains_key(id));
        let mut etcitemgrp = by_id(etc_items, |e| e.id);
        etcitemgrp.retain(|id, _| !weapongrp.contains_key(id) && !armorgrp.contains_key(id));
        let npc_rows = self.table_as("npcgrp.dat", npcgrp::read_as)?;
        let mut search_order = SearchOrder::default();
        for (index, npc) in npc_rows.iter().enumerate() {
            search_order.npc.entry(npc.id).or_insert(index as u32);
        }
        let npcs: NpcTable = npc_rows.into_iter().collect();
        let item_names = self.table_as(&language.table_name("itemname"), itemname::read_as)?;
        for (index, item) in item_names.iter().enumerate() {
            search_order.item.entry(item.id).or_insert(index as u32);
        }
        let skill_names = self.table(&language.table_name("skillname"), skillname::read)?;
        for (index, skill) in skill_names.iter().enumerate() {
            search_order.skill.entry((skill.id, skill.level)).or_insert(index as u32);
        }
        let npc_names = npcname::npc_records(npcs.iter().map(|npc| npc.id), self.table(&language.table_name("npcname"), npcname::read)?);
        let rotation_rates = self.rotation_rates(&npcs);
        Ok(DatTables {
            rotation_rates,
            npcgrp: npcs,
            chargrp: self.table("chargrp.dat", chargrp::read)?,
            hairgrp: self.table("hairgrp.dat", hairgrp::read)?,
            helmetgrp: self
                .table("helmetgrp.dat", helmetgrp::read)?
                .into_iter()
                .map(|h| (h.helmet_index, h.rows))
                .collect(),
            hairaccessarygrp: self
                .table("hairaccessarygrp.dat", helmetgrp::read)?
                .into_iter()
                .map(|h| (h.helmet_index, h.rows))
                .collect(),
            accessory_locs: hairaccessorylocgrp::by_mesh(
                self.table("hairaccessorylocgrp.dat", hairaccessorylocgrp::read)?,
            ),
            armorgrp,
            weapongrp,
            etcitemgrp,
            skillsound: self.table("skillsoundgrp.dat", l2_dat::skillsoundgrp::read)?,
            eula: self.table_as(&language.table_name("eula"), l2_dat::eula::read_as).ok(),
            warning_notice: self.table(&language.table_name("warningnotice"), l2_dat::warningnotice::read).ok(),
            credits: self.table(&language.table_name("creditgrp"), l2_dat::creditgrp::read).unwrap_or_default(),
            questname: questname::by_id_state(self.table_as(&language.table_name("questname"), questname::read_as)?),
            npcname: npc_names,
            systemmsg: by_id(self.table_as(&language.table_name("systemmsg"), systemmsg::read_as)?, |m| m.id),
            sysstring: self.table(&language.table_name("sysstring"), sysstring::read)?.into_iter().map(|s| (s.id, s.text)).collect(),
            symbolname: self.table(&language.table_name("symbolname"), symbolname::read)?.into_iter().collect(),
            hennagrp: by_id(self.table(&language.table_name("hennagrp"), hennagrp::read)?, |h| h.symbol_id),
            recipe_order: recipes.iter().map(|r| r.id).collect(),
            recipes: recipes.into_iter().map(|r| (r.id, r)).collect(),
            skillgrp: skillgrp::by_id_level(self.table("skillgrp.dat", skillgrp::read)?),
            zonename: self.table_as(&language.table_name("zonename"), zonename::read_as)?,
            castlename: by_id(self.table(&language.table_name("castlename"), l2_dat::castlename::read)?, |castle| castle.id),
            staticobject: self.table(&language.table_name("staticobject"), l2_dat::staticobject::read)?.into_iter().map(|object| (object.id, object.name)).collect(),
            anim_names: self.anim_names()?,
            language_type: language.language_type,
            logo_textures: crate::language::logo_textures(&self.read_system_text(LOCALIZATION_INI).unwrap_or_default(), &language.country),
            login_ini: crate::login_ini::LoginIni::parse(&self.read_system_text("lineage2.ini").unwrap_or_default(), &self.read_system_text("AutoLog.txt").unwrap_or_default()),
            item_icons,
            itemname: by_id(item_names, |n| n.id),
            skillname: skillname::by_id_level(skill_names),
            search_order,
            debug_ini: self.read_system_text("L2Debug.ini").unwrap_or_default(),
            mobskillanimgrp: mobskillanimgrp::by_npc_skill(self.table("mobskillanimgrp.dat", mobskillanimgrp::read)?),
            actions: self.table(&language.table_name("actionname"), actionname::read)?,
            commandname: commandname::by_name(self.table(&language.table_name("commandname"), commandname::read)?),
            soulshot: crate::shots::SoulShotData::parse(&self.read_system_text("soulshot.int").unwrap_or_default()),
            obscene: self.table_as(&language.table_name("obscene"), l2_dat::obscene::read_as).unwrap_or_default(),
            musicinfo: self.table("musicinfo.dat", l2_dat::musicinfo::read).map(|rows| rows.into_iter().map(|row| (row.id, row.titles)).collect()).unwrap_or_default(),
            entereventgrp: self.table("entereventgrp.dat", l2_dat::entereventgrp::read).map(l2_dat::entereventgrp::by_npc).unwrap_or_default(),
        })
    }

    fn table_with_icons_as<T>(&mut self, name: &str, read: fn(&[u8], l2_dat::Chronicle) -> Result<Vec<T>, l2_dat::Error>, kind: ItemKind) -> Result<(Vec<T>, Vec<itemicons::ItemIcon>), AssetError> {
        let body = self.read_system_file(name)?;
        let records = read(&body, self.chronicle_of(name)).map_err(|e| AssetError::Table(name.to_string(), e))?;
        let icons = itemicons::read_as(&body, kind, self.chronicle_of(name)).map_err(|e| AssetError::Table(name.to_string(), e))?;
        Ok((records, icons))
    }

    fn table_with_icons<T>(&mut self, name: &str, read: fn(&[u8]) -> Result<Vec<T>, l2_dat::Error>, kind: ItemKind) -> Result<(Vec<T>, Vec<itemicons::ItemIcon>), AssetError> {
        let body = self.read_system_file(name)?;
        let records = read(&body).map_err(|e| AssetError::Table(name.to_string(), e))?;
        let icons = itemicons::read(&body, kind).map_err(|e| AssetError::Table(name.to_string(), e))?;
        Ok((records, icons))
    }

    fn anim_names(&mut self) -> Result<HashMap<String, AnimSet>, AssetError> {
        let mut sets = HashMap::new();
        for name in ANIM_NAME_FILES {
            sets.extend(animnames::parse(&decode_text(&self.read_system_file(name)?)));
        }
        Ok(sets)
    }

    fn table_as<T>(&mut self, name: &str, read: fn(&[u8], l2_dat::Chronicle) -> Result<T, l2_dat::Error>) -> Result<T, AssetError> {
        let body = self.read_system_file(name)?;
        read(&body, self.chronicle_of(name)).map_err(|e| AssetError::Table(name.to_string(), e))
    }

    fn table<T>(&mut self, name: &str, read: fn(&[u8]) -> Result<T, l2_dat::Error>) -> Result<T, AssetError> {
        let body = self.read_system_file(name)?;
        read(&body).map_err(|e| AssetError::Table(name.to_string(), e))
    }
}

fn find_file(dir: &Path, stem: &str, extension: &str) -> Option<PathBuf> {
    fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).find(|p| {
        p.file_stem().is_some_and(|s| s.eq_ignore_ascii_case(stem))
            && p.extension().is_some_and(|e| e.eq_ignore_ascii_case(extension))
    })
}

fn find_file_by_name(dir: &Path, name: &str) -> Option<PathBuf> {
    fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.file_name().is_some_and(|n| n.eq_ignore_ascii_case(name)))
}

#[cfg(test)]
mod search_tests {
    use super::*;

    #[test]
    fn the_gm_searches_return_the_first_record_of_the_file_with_that_name() {
        let mut tables = DatTables::default();
        let npc = |id: u32, name: &str| NpcName { id, name: name.into(), title: String::new(), color: [0; 3] };
        for (index, (id, name)) in [(300, "Dup"), (100, "Dup"), (200, "Other")].into_iter().enumerate() {
            tables.npcname.insert(id, npc(id, name));
            tables.search_order.npc.insert(id, index as u32);
        }
        assert_eq!(tables.search_npc("Dup"), Some(300));
        assert_eq!(tables.search_npc("Missing"), None);
        let item = |id: u32, name: &str| ItemName { id, name: name.into(), add_name: String::new(), description: String::new(), popup: 0 };
        for (index, (id, name)) in [(9, "Sword"), (4, "Sword")].into_iter().enumerate() {
            tables.itemname.insert(id, item(id, name));
            tables.search_order.item.insert(id, index as u32);
        }
        assert_eq!(tables.search_item("Sword"), Some(9));
        let skill = |id: u32, level: u32, name: &str| SkillName { id, level, name: name.into(), description: String::new(), desc_add1: String::new(), desc_add2: String::new() };
        for (index, (id, level)) in [(50, 2), (50, 1), (20, 1)].into_iter().enumerate() {
            tables.skillname.insert((id, level), skill(id, level, "Strike"));
            tables.search_order.skill.insert((id, level), index as u32);
        }
        assert_eq!(tables.search_skill("Strike"), Some(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn shared_item_bytes_preserve_every_original_record_and_icon() {
        fn verify<T: PartialEq>(client: &mut Client, name: &str, read: fn(&[u8]) -> Result<Vec<T>, l2_dat::Error>, kind: ItemKind) {
            let (records, icons) = client.table_with_icons(name, read, kind).unwrap();
            let original = client.read_system_file(name).unwrap();
            let expected = read(&original).unwrap();
            assert_eq!(records.len(), expected.len(), "{name}");
            for (index, (actual, expected)) in records.iter().zip(&expected).enumerate() {
                assert!(actual == expected, "{name}: registro {index}");
            }
            let expected_icons = itemicons::read(&original, kind).unwrap();
            assert_eq!(icons.len(), expected_icons.len(), "{name}");
            for (index, (actual, expected)) in icons.iter().zip(&expected_icons).enumerate() {
                assert!(actual == expected, "{name}: ícone {index}");
            }
        }
        let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = Client::open(&root).unwrap();
        verify(&mut client, "weapongrp.dat", weapongrp::read, ItemKind::Weapon);
        verify(&mut client, "armorgrp.dat", armorgrp::read, ItemKind::Armor);
        verify(&mut client, "etcitemgrp.dat", etcitemgrp::read, ItemKind::Etc);
    }

    #[test]
    #[ignore]
    fn symbol_info_load_10441130_audits_every_texture_from_the_client() {
        let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        assert_eq!(tables.symbolname.entries.len(), 11);
        let mut bank = crate::textures::TextureBank::new();
        let mut missing = Vec::new();
        let mut resolved = Vec::new();
        for symbol in tables.symbolname.entries.values() {
            let Some(texture) = client.catalog().texture_raw(&symbol.texture) else {
                missing.push(symbol.id);
                continue;
            };
            resolved.push(symbol.id);
            assert!(!texture.mips.is_empty(), "{}", symbol.texture);
            assert!(texture.mips[0].width >= 12 && texture.mips[0].height >= 12, "{}", symbol.texture);
            let decoded = bank.texture(client.catalog(), &symbol.texture);
            assert!(!decoded.mips[0].is_empty(), "{}", symbol.texture);
        }
        assert_eq!(missing, [1, 2, 3, 4, 5]);
        assert_eq!(resolved, [100, 101, 102, 103, 104, 105]);
    }
}
