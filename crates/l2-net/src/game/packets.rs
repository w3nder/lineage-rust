mod extended;
mod lobby;
mod rare;
mod windows;
mod world;

pub use extended::*;
pub use lobby::*;
pub use rare::*;
pub use windows::*;
pub use world::*;

use crate::login::Authorization;
use crate::wire::{Reader, Result, WireError, Writer};

pub const OP_PROTOCOL_VERSION: u8 = 0x00;
pub const OP_AUTH_LOGIN: u8 = 0x08;
pub const OP_CHARACTER_SELECTED: u8 = 0x0d;
pub const OP_ENTER_WORLD: u8 = 0x03;

pub const OP_KEY_PACKET: u8 = 0x00;
pub const OP_CHAR_SELECT_INFO: u8 = 0x13;
pub const OP_AUTH_LOGIN_FAIL: u8 = 0x14;
pub const OP_CHAR_SELECTED: u8 = 0x15;

pub const OP_USER_INFO: u8 = 0x04;
pub const OP_RESTART_RESPONSE: u8 = 0x5f;
pub const OP_REQUEST_RESTART: u8 = 0x46;
pub const OP_LOGOUT: u8 = 0x09;
pub const OP_CLIENT_SET_TIME: u8 = 0xec;

pub const PROTOCOL_VERSION: i32 = 656;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestartResponse {
    pub result: i32,
    pub message: String,
}

pub fn read_restart_response(body: &[u8]) -> Result<RestartResponse> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let result = r.read_d()?;
    let message = r.read_s()?;
    Ok(RestartResponse { result, message })
}

const CLIENT_BLOCK: [u8; 256] = [
    0x09, 0x07, 0x54, 0x56, 0x03, 0x09, 0x0b, 0x01, 0x07, 0x02, 0x54, 0x54, 0x56, 0x07, 0x00, 0x02, 0x55, 0x56, 0x00,
    0x51, 0x00, 0x53, 0x57, 0x04, 0x07, 0x55, 0x08, 0x54, 0x01, 0x07, 0x01, 0x53, 0x00, 0x56, 0x55, 0x56, 0x01, 0x06,
    0x05, 0x04, 0x51, 0x03, 0x08, 0x51, 0x08, 0x51, 0x56, 0x04, 0x54, 0x06, 0x55, 0x08, 0x02, 0x09, 0x51, 0x56, 0x01,
    0x53, 0x06, 0x55, 0x04, 0x53, 0x00, 0x56, 0x56, 0x53, 0x01, 0x09, 0x02, 0x09, 0x01, 0x51, 0x54, 0x51, 0x09, 0x55,
    0x56, 0x09, 0x03, 0x04, 0x07, 0x05, 0x55, 0x04, 0x06, 0x55, 0x04, 0x06, 0x09, 0x04, 0x51, 0x01, 0x08, 0x08, 0x06,
    0x05, 0x52, 0x06, 0x04, 0x01, 0x07, 0x54, 0x03, 0x06, 0x52, 0x55, 0x06, 0x55, 0x55, 0x51, 0x01, 0x02, 0x04, 0x54,
    0x03, 0x55, 0x54, 0x01, 0x57, 0x51, 0x55, 0x05, 0x52, 0x05, 0x54, 0x07, 0x51, 0x51, 0x55, 0x07, 0x02, 0x53, 0x53,
    0x00, 0x52, 0x05, 0x52, 0x07, 0x01, 0x54, 0x00, 0x03, 0x05, 0x05, 0x08, 0x06, 0x05, 0x05, 0x06, 0x03, 0x00, 0x0d,
    0x08, 0x01, 0x07, 0x09, 0x03, 0x51, 0x03, 0x07, 0x53, 0x09, 0x51, 0x06, 0x07, 0x54, 0x0a, 0x50, 0x56, 0x02, 0x52,
    0x04, 0x05, 0x55, 0x51, 0x02, 0x53, 0x00, 0x08, 0x54, 0x04, 0x52, 0x56, 0x06, 0x02, 0x09, 0x00, 0x08, 0x03, 0x53,
    0x56, 0x01, 0x05, 0x00, 0x55, 0x06, 0x08, 0x56, 0x04, 0x0d, 0x06, 0x07, 0x52, 0x06, 0x07, 0x04, 0x0a, 0x06, 0x01,
    0x04, 0x54, 0x04, 0x00, 0x05, 0x02, 0x04, 0x54, 0x00, 0x09, 0x52, 0x53, 0x05, 0x04, 0x01, 0x04, 0x05, 0x05, 0x01,
    0x52, 0x51, 0x52, 0x0d, 0x06, 0x51, 0x08, 0x09, 0x54, 0x53, 0x00, 0x0d, 0x01, 0x02, 0x03, 0x54, 0x53, 0x01, 0x05,
    0x03, 0x08, 0x56, 0x54, 0x07, 0x02, 0x54, 0x0b, 0x06,
];

pub fn protocol_version() -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_PROTOCOL_VERSION);
    w.write_d(PROTOCOL_VERSION);
    w.write_b(&CLIENT_BLOCK);
    w.into_bytes()
}

pub fn auth_login(name: &str, auth: &Authorization) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_AUTH_LOGIN);
    w.write_s(name);
    w.write_d(auth.play_key[1] as i32);
    w.write_d(auth.play_key[0] as i32);
    w.write_d(auth.login_key[0] as i32);
    w.write_d(auth.login_key[1] as i32);
    w.into_bytes()
}

pub fn character_selected(slot: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_CHARACTER_SELECTED);
    w.write_d(slot);
    w.write_h(0);
    w.write_d(0);
    w.write_d(0);
    w.write_d(0);
    w.into_bytes()
}

pub fn character_selected_transfer(slot: i32, transfer: bool, position: [i32; 3]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_CHARACTER_SELECTED);
    w.write_d(slot);
    w.write_h(u16::from(transfer));
    w.write_d(position[0]);
    w.write_d(position[1]);
    w.write_d(position[2]);
    w.into_bytes()
}

pub const ENTER_WORLD_TAG: &str = "narcasse";

pub fn enter_world() -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_ENTER_WORLD);
    w.write_s(ENTER_WORLD_TAG);
    w.into_bytes()
}

pub fn request_restart() -> Vec<u8> {
    vec![OP_REQUEST_RESTART]
}

pub fn logout() -> Vec<u8> {
    vec![OP_LOGOUT]
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserInfoHead {
    pub abnormal_effect: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub heading: i32,
    pub object_id: i32,
    pub name: String,
    pub race: i32,
    pub sex: i32,
    pub class_id: i32,
    pub level: i32,
    pub exp: i32,
    pub max_hp: i32,
    pub hp: i32,
    pub max_mp: i32,
    pub mp: i32,
    pub pvp_flag: i32,
    pub karma: i32,
    pub run_speed: i32,
    pub walk_speed: i32,
    pub speed_multiplier: f64,
    pub attack_speed_multiplier: f64,
    pub collision_radius: f64,
    pub collision_height: f64,
    pub hair_style: i32,
    pub hair_color: i32,
    pub face: i32,
    pub builder_level: i32,
    pub title: String,
    pub max_cp: i32,
    pub cp: i32,
    pub stats: UserStats,
    pub paperdoll: [PaperdollSlot; PAPERDOLL_SLOTS],
    pub inventory_limit: i32,
    pub name_color: i32,
    pub social: UserSocial,
    pub enchant_effect: u8,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UserSocial {
    pub clan_id: i32,
    pub clan_crest_id: i32,
    pub ally_id: i32,
    pub ally_crest_id: i32,
    pub mount_type: u8,
    pub private_store: u8,
    pub dwarven_craft: bool,
    pub pk_kills: i32,
    pub pvp_kills: i32,
    pub cubic_ids: Vec<u16>,
    pub recom_left: u16,
    pub recom_have: u16,
    pub noble: bool,
    pub hero: bool,
    pub relation: i32,
    pub pledge_rights: [bool; 4],
}

impl UserInfoHead {
    pub fn equipped(&self) -> [i32; VISIBLE_EQUIP_SLOTS] {
        visible_equipment(&self.paperdoll)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UserStats {
    pub str_: i32,
    pub dex: i32,
    pub con: i32,
    pub int_: i32,
    pub wit: i32,
    pub men: i32,
    pub sp: i32,
    pub load: i32,
    pub max_load: i32,
    pub attack_range: i32,
    pub p_atk: i32,
    pub atk_spd: i32,
    pub p_def: i32,
    pub evasion: i32,
    pub accuracy: i32,
    pub critical: i32,
    pub m_atk: i32,
    pub cast_spd: i32,
    pub m_def: i32,
}

impl UserStats {
    pub fn apply(&mut self, attribute: i32, value: i32) -> bool {
        let field = match attribute {
            ATTRIBUTE_STR => &mut self.str_,
            ATTRIBUTE_DEX => &mut self.dex,
            ATTRIBUTE_CON => &mut self.con,
            ATTRIBUTE_INT => &mut self.int_,
            ATTRIBUTE_WIT => &mut self.wit,
            ATTRIBUTE_MEN => &mut self.men,
            ATTRIBUTE_SP => &mut self.sp,
            ATTRIBUTE_CUR_LOAD => &mut self.load,
            ATTRIBUTE_MAX_LOAD => &mut self.max_load,
            ATTRIBUTE_ATTACK_RANGE => &mut self.attack_range,
            ATTRIBUTE_P_ATK => &mut self.p_atk,
            ATTRIBUTE_P_ATK_SPD => &mut self.atk_spd,
            ATTRIBUTE_P_DEF => &mut self.p_def,
            ATTRIBUTE_EVASION => &mut self.evasion,
            ATTRIBUTE_ACCURACY => &mut self.accuracy,
            ATTRIBUTE_CRITICAL => &mut self.critical,
            ATTRIBUTE_M_ATK => &mut self.m_atk,
            ATTRIBUTE_CAST_SPD => &mut self.cast_spd,
            ATTRIBUTE_M_DEF => &mut self.m_def,
            _ => return false,
        };
        *field = value;
        true
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PaperdollSlot {
    pub object_id: i32,
    pub item_id: i32,
}

pub const PAPERDOLL_SLOTS: usize = 16;
pub const PAPERDOLL_UNDER: usize = 0;
pub const PAPERDOLL_R_EAR: usize = 1;
pub const PAPERDOLL_L_EAR: usize = 2;
pub const PAPERDOLL_NECK: usize = 3;
pub const PAPERDOLL_R_FINGER: usize = 4;
pub const PAPERDOLL_L_FINGER: usize = 5;
pub const PAPERDOLL_HEAD: usize = 6;
pub const PAPERDOLL_R_HAND: usize = 7;
pub const PAPERDOLL_L_HAND: usize = 8;
pub const PAPERDOLL_GLOVES: usize = 9;
pub const PAPERDOLL_CHEST: usize = 10;
pub const PAPERDOLL_LEGS: usize = 11;
pub const PAPERDOLL_FEET: usize = 12;
pub const PAPERDOLL_BACK: usize = 13;
pub const PAPERDOLL_LR_HAND: usize = 14;
pub const PAPERDOLL_HAIR: usize = 15;

pub const BODY_PART_UNDERWEAR: i32 = 0x0001;
pub const BODY_PART_R_EAR: i32 = 0x0002;
pub const BODY_PART_L_EAR: i32 = 0x0004;
pub const BODY_PART_NECK: i32 = 0x0008;
pub const BODY_PART_R_FINGER: i32 = 0x0010;
pub const BODY_PART_L_FINGER: i32 = 0x0020;
pub const BODY_PART_HEAD: i32 = 0x0040;
pub const BODY_PART_R_HAND: i32 = 0x0080;
pub const BODY_PART_L_HAND: i32 = 0x0100;
pub const BODY_PART_GLOVES: i32 = 0x0200;
pub const BODY_PART_CHEST: i32 = 0x0400;
pub const BODY_PART_LEGS: i32 = 0x0800;
pub const BODY_PART_FEET: i32 = 0x1000;
pub const BODY_PART_BACK: i32 = 0x2000;
pub const BODY_PART_LR_HAND: i32 = 0x4000;
pub const BODY_PART_FULL_ARMOR: i32 = 0x8000;
pub const BODY_PART_HAIR: i32 = 0x10000;

fn pledge_rights_from_block(block: &[u8; 32]) -> [bool; 4] {
    [block[0] & 1 != 0, block[0] >> 1 & 1 != 0, block[0] >> 3 & 1 != 0, block[1] >> 2 & 1 != 0]
}

pub fn read_user_info_head(body: &[u8]) -> Result<UserInfoHead> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let heading = r.read_d()?;
    let object_id = r.read_d()?;
    let name = r.read_s()?;
    let race = r.read_d()?;
    let sex = r.read_d()?;
    let class_id = r.read_d()?;
    let level = r.read_d()?;
    let exp = r.read_d()?;
    let str_ = r.read_d()?;
    let dex = r.read_d()?;
    let con = r.read_d()?;
    let int_ = r.read_d()?;
    let wit = r.read_d()?;
    let men = r.read_d()?;
    let max_hp = r.read_d()?;
    let hp = r.read_d()?;
    let max_mp = r.read_d()?;
    let mp = r.read_d()?;
    let sp = r.read_d()?;
    let load = r.read_d()?;
    let max_load = r.read_d()?;
    let attack_range = r.read_d()?;
    let mut paperdoll = [PaperdollSlot::default(); PAPERDOLL_SLOTS];
    for slot in &mut paperdoll {
        slot.object_id = r.read_d()?;
    }
    for slot in &mut paperdoll {
        slot.item_id = r.read_d()?;
    }
    let p_atk = r.read_d()?;
    let atk_spd = r.read_d()?;
    let p_def = r.read_d()?;
    let evasion = r.read_d()?;
    let accuracy = r.read_d()?;
    let critical = r.read_d()?;
    let m_atk = r.read_d()?;
    let cast_spd = r.read_d()?;
    r.read_d()?;
    let m_def = r.read_d()?;
    let stats = UserStats {
        str_,
        dex,
        con,
        int_,
        wit,
        men,
        sp,
        load,
        max_load,
        attack_range,
        p_atk,
        atk_spd,
        p_def,
        evasion,
        accuracy,
        critical,
        m_atk,
        cast_spd,
        m_def,
    };
    let pvp_flag = r.read_d()?;
    let karma = r.read_d()?;
    let run_speed = r.read_d()?;
    let walk_speed = r.read_d()?;
    for _ in 0..6 {
        r.read_d()?;
    }
    let speed_multiplier = r.read_f()?;
    let attack_speed_multiplier = r.read_f()?;
    let collision_radius = r.read_f()?;
    let collision_height = r.read_f()?;
    let hair_style = r.read_d()?;
    let hair_color = r.read_d()?;
    let face = r.read_d()?;
    let builder_level = r.read_d()?;
    let title = r.read_s()?;
    let clan_id = r.read_d()?;
    let clan_crest_id = r.read_d()?;
    let ally_id = r.read_d()?;
    let ally_crest_id = r.read_d()?;
    let relation = r.read_d()?;
    let mount_type = r.read_c()?;
    let private_store = r.read_c()?;
    let dwarven_craft = r.read_c()? != 0;
    let pk_kills = r.read_d()?;
    let pvp_kills = r.read_d()?;
    let cubics = r.read_h()?;
    let mut cubic_ids = Vec::new();
    for _ in 0..cubics {
        cubic_ids.push(r.read_h()?);
    }
    r.read_c()?;
    let abnormal_effect = r.read_d()?;
    r.read_c()?;
    let mut privileges = [0u8; 32];
    for chunk in privileges.chunks_exact_mut(4) {
        chunk.copy_from_slice(&r.read_d()?.to_le_bytes());
    }
    let recom_left = r.read_h()?;
    let recom_have = r.read_h()?;
    r.read_d()?;
    let inventory_limit = r.read_h()? as i32;
    r.read_d()?;
    r.read_d()?;
    let max_cp = r.read_d()?;
    let cp = r.read_d()?;
    let enchant_effect = r.read_c()?;
    r.read_c()?;
    r.read_d()?;
    let noble = r.read_c()? != 0;
    let hero = r.read_c()? != 0;
    r.read_c()?;
    r.read_d()?;
    r.read_d()?;
    r.read_d()?;
    let name_color = r.read_d()?;
    let social = UserSocial {
        clan_id,
        clan_crest_id,
        ally_id,
        ally_crest_id,
        mount_type,
        private_store,
        dwarven_craft,
        pk_kills,
        pvp_kills,
        cubic_ids,
        recom_left,
        recom_have,
        noble,
        hero,
        relation,
        pledge_rights: if clan_id > 0 { pledge_rights_from_block(&privileges) } else { [false; 4] },
    };
    Ok(UserInfoHead {
        abnormal_effect,
        x,
        y,
        z,
        heading,
        object_id,
        name,
        race,
        sex,
        class_id,
        level,
        exp,
        max_hp,
        hp,
        max_mp,
        mp,
        pvp_flag,
        karma,
        run_speed,
        walk_speed,
        speed_multiplier,
        attack_speed_multiplier,
        collision_radius,
        collision_height,
        hair_style,
        hair_color,
        face,
        builder_level,
        title,
        max_cp,
        cp,
        stats,
        paperdoll,
        inventory_limit,
        name_color,
        social,
        enchant_effect,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharacterSelected {
    pub name: String,
    pub char_id: i32,
    pub title: String,
    pub session_id: i32,
    pub clan_id: i32,
    pub sex: i32,
    pub race: i32,
    pub class_id: i32,
    pub active: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub hp: f64,
    pub mp: f64,
    pub sp: i32,
    pub exp: i32,
    pub level: i32,
    pub karma: i32,
    pub middle: [i32; 40],
    pub inspector_bin: Vec<u8>,
    pub inspector_params: Vec<[i32; 4]>,
    pub tail_head: i32,
    pub tail: [i32; 4],
}

pub fn read_character_selected(body: &[u8]) -> Result<CharacterSelected> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let name = r.read_s()?;
    let char_id = r.read_d()?;
    let title = r.read_s()?;
    let session_id = r.read_d()?;
    let clan_id = r.read_d()?;
    r.read_d()?;
    let sex = r.read_d()?;
    let race = r.read_d()?;
    let class_id = r.read_d()?;
    let active = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let hp = r.read_f()?;
    let mp = r.read_f()?;
    let sp = r.read_d()?;
    let exp = r.read_d()?;
    let level = r.read_d()?;
    let karma = r.read_d()?;
    let mut middle = [0i32; 40];
    for slot in &mut middle {
        *slot = r.read_d()?;
    }
    let count = r.read_d()?;
    let mut inspector_bin = Vec::new();
    let mut inspector_params = Vec::new();
    if count > 0 {
        let size = r.read_d()?;
        if size > 0 {
            inspector_bin = r.read_b(size as usize)?.to_vec();
        }
        for _ in 0..count {
            inspector_params.push([r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?]);
        }
    }
    let tail_head = r.read_d()?;
    let tail = [r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?];
    Ok(CharacterSelected {
        name, char_id, title, session_id, clan_id, sex, race, class_id, active, x, y, z, hp, mp, sp, exp, level, karma,
        middle, inspector_bin, inspector_params, tail_head, tail,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharSelectEntry {
    pub name: String,
    pub char_id: i32,
    pub login: String,
    pub clan_id: i32,
    pub sex: i32,
    pub race: i32,
    pub base_class_id: i32,
    pub active: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub hp: f64,
    pub mp: f64,
    pub sp: i32,
    pub exp: i32,
    pub level: i32,
    pub karma: i32,
    pub paperdoll_objects: [i32; 16],
    pub paperdoll_items: [i32; 16],
    pub hair_style: i32,
    pub hair_color: i32,
    pub face: i32,
    pub max_hp: f64,
    pub max_mp: f64,
    pub delete_days: i32,
    pub class_id: i32,
    pub last_used: i32,
    pub enchant_effect: u8,
}

pub fn read_char_select_info(body: &[u8]) -> Result<Vec<CharSelectEntry>> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let count = r.read_d()?.clamp(0, 32) as usize;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let name = r.read_s()?;
        let char_id = r.read_d()?;
        let login = r.read_s()?;
        r.read_d()?;
        let clan_id = r.read_d()?;
        r.read_d()?;
        let sex = r.read_d()?;
        let race = r.read_d()?;
        let base_class_id = r.read_d()?;
        let active = r.read_d()?;
        let x = r.read_d()?;
        let y = r.read_d()?;
        let z = r.read_d()?;
        let hp = r.read_f()?;
        let mp = r.read_f()?;
        let sp = r.read_d()?;
        let exp = r.read_d()?;
        let level = r.read_d()?;
        let karma = r.read_d()?;
        for _ in 0..9 {
            r.read_d()?;
        }
        let mut paperdoll_objects = [0i32; 16];
        for slot in &mut paperdoll_objects {
            *slot = r.read_d()?;
        }
        let mut paperdoll_items = [0i32; 16];
        for slot in &mut paperdoll_items {
            *slot = r.read_d()?;
        }
        let hair_style = r.read_d()?;
        let hair_color = r.read_d()?;
        let face = r.read_d()?;
        let max_hp = r.read_f()?;
        let max_mp = r.read_f()?;
        let delete_days = r.read_d()?;
        let class_id = r.read_d()?;
        let last_used = r.read_d()?;
        let enchant_effect = r.read_c()?;
        entries.push(CharSelectEntry {
            name,
            char_id,
            login,
            clan_id,
            sex,
            race,
            base_class_id,
            active,
            x,
            y,
            z,
            hp,
            mp,
            sp,
            exp,
            level,
            karma,
            paperdoll_objects,
            paperdoll_items,
            hair_style,
            hair_color,
            face,
            max_hp,
            max_mp,
            delete_days,
            class_id,
            last_used,
            enchant_effect,
        });
    }
    Ok(entries)
}

pub const OP_MOVE_TO_LOCATION: u8 = 0x01;
pub const OP_NPC_SAY: u8 = 0x02;
pub const OP_CHAR_INFO: u8 = 0x03;
pub const OP_DIE: u8 = 0x06;
pub const OP_REVIVE: u8 = 0x07;
pub const OP_STATUS_UPDATE: u8 = 0x0e;
pub const OP_DELETE_OBJECT: u8 = 0x12;
pub const OP_NPC_INFO: u8 = 0x16;
pub const OP_TELEPORT_TO_LOCATION: u8 = 0x28;
pub const OP_SOCIAL_ACTION: u8 = 0x2d;
pub const OP_CHANGE_MOVE_TYPE: u8 = 0x2e;
pub const OP_CHANGE_WAIT_TYPE: u8 = 0x2f;
pub const OP_STOP_MOVE: u8 = 0x47;
pub const OP_SAY2: u8 = 0x4a;
pub const OP_VALIDATE_LOCATION: u8 = 0x61;
pub const OP_SYSTEM_MESSAGE: u8 = 0x64;
pub const OP_SPAWN_ITEM: u8 = 0x0b;
pub const OP_TARGET_SELECTED: u8 = 0x29;
pub const OP_TARGET_UNSELECTED: u8 = 0x2a;
pub const OP_MAGIC_SKILL_USE: u8 = 0x48;
pub const OP_MAGIC_SKILL_CANCELED: u8 = 0x49;
pub const OP_MOVE_TO_PAWN: u8 = 0x60;
pub const OP_MAGIC_SKILL_LAUNCHED: u8 = 0x76;
pub const OP_MAGIC_EFFECT_ICONS: u8 = 0x7f;
pub const OP_PLEDGE_STATUS_CHANGED: u8 = 0xcd;
pub const OP_RELATION_CHANGED: u8 = 0xce;
pub const OP_SEND_MACRO_LIST: u8 = 0xe7;
pub const OP_ETC_STATUS_UPDATE: u8 = 0xf3;
pub const OP_NPC_HTML_MESSAGE: u8 = 0x0f;
pub const OP_ITEM_LIST: u8 = 0x1b;
pub const OP_ACTION_FAILED: u8 = 0x25;
pub const OP_INVENTORY_UPDATE: u8 = 0x27;
pub const OP_SHORT_CUT_REGISTER: u8 = 0x44;
pub const OP_SHORT_CUT_INIT: u8 = 0x45;
pub const OP_SHORT_CUT_DELETE: u8 = 0x46;
pub const OP_EQUIP_UPDATE: u8 = 0x4b;
pub const OP_SKILL_LIST: u8 = 0x58;
pub const OP_MY_TARGET_SELECTED: u8 = 0xa6;
pub const OP_SKILL_COOL_TIME: u8 = 0xc1;

pub const OP_MOVE_BACKWARD_TO_LOCATION: u8 = 0x01;
pub const OP_ACTION: u8 = 0x04;
pub const OP_REQUEST_MAGIC_SKILL_LIST: u8 = 0x2e;
pub const OP_APPEARING: u8 = 0x30;
pub const OP_SAY2_CLIENT: u8 = 0x38;
pub const OP_VALIDATE_POSITION: u8 = 0x48;
pub const OP_REQUEST_ITEM_LIST: u8 = 0x0f;
pub const OP_REQUEST_UNEQUIP_ITEM: u8 = 0x11;
pub const OP_USE_ITEM: u8 = 0x14;
pub const OP_REQUEST_LINK_HTML: u8 = 0x20;
pub const OP_REQUEST_BYPASS_TO_SERVER: u8 = 0x21;
pub const OP_REQUEST_MAGIC_SKILL_USE: u8 = 0x2f;
pub const OP_REQUEST_SHORT_CUT_REG: u8 = 0x33;
pub const OP_REQUEST_SHORT_CUT_USE: u8 = 0x34;
pub const OP_REQUEST_SHORT_CUT_DEL: u8 = 0x35;
pub const OP_REQUEST_TARGET_CANCEL: u8 = 0x37;
pub const OP_REQUEST_DISMISS_PARTY: u8 = 0x2d;
pub const OP_REQUEST_GM_LIST: u8 = 0x81;
pub const OP_REVIVE_REPLY: u8 = 0x7a;
pub const OP_SEND_TIME_CHECK: u8 = 0x97;
pub const OP_REQUEST_REMAIN_TIME: u8 = 0xa9;
pub const OP_NET_PING: u8 = 0xa8;
pub const OP_REQUEST_ACTION_USE: u8 = 0x45;
pub const OP_REQUEST_DESTROY_ITEM: u8 = 0x59;
pub const OP_REQUEST_SKILL_COOL_TIME: u8 = 0x9d;

pub const ITEM_ADDED: u16 = 1;
pub const ITEM_MODIFIED: u16 = 2;
pub const ITEM_REMOVED: u16 = 3;
pub const ADENA_ITEM_ID: i32 = 57;

pub const SHORTCUT_ITEM: i32 = 1;
pub const SHORTCUT_SKILL: i32 = 2;
pub const SHORTCUT_ACTION: i32 = 3;
pub const SHORTCUT_MACRO: i32 = 4;
pub const SHORTCUT_RECIPE: i32 = 5;
pub const SHORTCUT_SLOTS_PER_PAGE: i32 = 12;

pub const NPC_HTML_MAX_CHARS: usize = 0x4000 - 1;

pub const MOVEMENT_MODE_MOUSE: i32 = 1;
pub const NPC_TYPE_OFFSET: i32 = 1_000_000;
pub const WAIT_TYPE_SIT: u8 = 0;
pub const VISIBLE_EQUIP_SLOTS: usize = 11;
pub const EQUIP_DHAIR: usize = 0;
pub const EQUIP_HEAD: usize = 1;
pub const EQUIP_RHAND: usize = 2;
pub const EQUIP_LHAND: usize = 3;
pub const EQUIP_GLOVES: usize = 4;
pub const EQUIP_CHEST: usize = 5;
pub const EQUIP_LEGS: usize = 6;
pub const EQUIP_FEET: usize = 7;
pub const EQUIP_BACK: usize = 8;
pub const EQUIP_LRHAND: usize = 9;
pub const EQUIP_HAIR: usize = 10;
pub const PAPERDOLL_VISIBLE: [usize; VISIBLE_EQUIP_SLOTS] = [0, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

pub fn visible_equipment(paperdoll: &[PaperdollSlot; PAPERDOLL_SLOTS]) -> [i32; VISIBLE_EQUIP_SLOTS] {
    PAPERDOLL_VISIBLE.map(|slot| paperdoll[slot].item_id)
}

pub const ATTRIBUTE_LEVEL: i32 = 0x01;
pub const ATTRIBUTE_EXP: i32 = 0x02;
pub const ATTRIBUTE_CURRENT_HP: i32 = 0x09;
pub const ATTRIBUTE_MAX_HP: i32 = 0x0a;
pub const ATTRIBUTE_CURRENT_MP: i32 = 0x0b;
pub const ATTRIBUTE_MAX_MP: i32 = 0x0c;
pub const ATTRIBUTE_STR: i32 = 0x03;
pub const ATTRIBUTE_DEX: i32 = 0x04;
pub const ATTRIBUTE_CON: i32 = 0x05;
pub const ATTRIBUTE_INT: i32 = 0x06;
pub const ATTRIBUTE_WIT: i32 = 0x07;
pub const ATTRIBUTE_MEN: i32 = 0x08;
pub const ATTRIBUTE_SP: i32 = 0x0d;
pub const ATTRIBUTE_CUR_LOAD: i32 = 0x0e;
pub const ATTRIBUTE_MAX_LOAD: i32 = 0x0f;
pub const ATTRIBUTE_ATTACK_RANGE: i32 = 0x10;
pub const ATTRIBUTE_P_ATK: i32 = 0x11;
pub const ATTRIBUTE_P_ATK_SPD: i32 = 0x12;
pub const ATTRIBUTE_P_DEF: i32 = 0x13;
pub const ATTRIBUTE_EVASION: i32 = 0x14;
pub const ATTRIBUTE_ACCURACY: i32 = 0x15;
pub const ATTRIBUTE_CRITICAL: i32 = 0x16;
pub const ATTRIBUTE_M_ATK: i32 = 0x17;
pub const ATTRIBUTE_CAST_SPD: i32 = 0x18;
pub const ATTRIBUTE_M_DEF: i32 = 0x19;
pub const ATTRIBUTE_PVP_FLAG: i32 = 0x1a;
pub const ATTRIBUTE_KARMA: i32 = 0x1b;
pub const ATTRIBUTE_CURRENT_CP: i32 = 0x21;
pub const ATTRIBUTE_MAX_CP: i32 = 0x22;

pub const CHAT_ALL: i32 = 0;
pub const CHAT_SHOUT: i32 = 1;
pub const CHAT_TELL: i32 = 2;
pub const CHAT_PARTY: i32 = 3;
pub const CHAT_CLAN: i32 = 4;
pub const CHAT_GM: i32 = 5;
pub const CHAT_PETITION_PLAYER: i32 = 6;
pub const CHAT_PETITION_GM: i32 = 7;
pub const CHAT_TRADE: i32 = 8;
pub const CHAT_ALLIANCE: i32 = 9;
pub const CHAT_ANNOUNCEMENT: i32 = 10;
pub const CHAT_BOAT: i32 = 11;
pub const CHAT_FRIEND: i32 = 12;
pub const CHAT_MSN: i32 = 13;
pub const CHAT_PARTY_ROOM: i32 = 14;
pub const CHAT_CHANNEL_LEADER: i32 = 15;
pub const CHAT_CHANNEL_ALL: i32 = 16;
pub const CHAT_HERO_VOICE: i32 = 17;
pub const CHAT_CRITICAL_ANNOUNCE: i32 = 18;

pub const ARGUMENT_TEXT: i32 = 0;
pub const ARGUMENT_NUMBER: i32 = 1;
pub const ARGUMENT_NPC_NAME: i32 = 2;
pub const ARGUMENT_ITEM_NAME: i32 = 3;
pub const ARGUMENT_SKILL_NAME: i32 = 4;
pub const ARGUMENT_CASTLE_NAME: i32 = 5;
pub const SYSTEM_MESSAGE_TEXT_MAX_CHARS: usize = 0x400 / 2 - 1;

#[derive(Debug, Clone, PartialEq)]
pub struct CharInfo {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub heading: i32,
    pub object_id: i32,
    pub name: String,
    pub race: i32,
    pub sex: i32,
    pub class_id: i32,
    pub equipped: [i32; VISIBLE_EQUIP_SLOTS],
    pub pvp_flag: i32,
    pub karma: i32,
    pub m_atk_speed: i32,
    pub p_atk_speed: i32,
    pub run_speed: i32,
    pub walk_speed: i32,
    pub swim_run_speed: i32,
    pub swim_walk_speed: i32,
    pub ground_run_speed: i32,
    pub ground_walk_speed: i32,
    pub fly_run_speed: i32,
    pub fly_walk_speed: i32,
    pub speed_multiplier: f64,
    pub attack_speed_multiplier: f64,
    pub collision_radius: f64,
    pub collision_height: f64,
    pub hair_style: i32,
    pub hair_color: i32,
    pub face: i32,
    pub title: String,
    pub clan_id: i32,
    pub clan_crest_id: i32,
    pub ally_id: i32,
    pub ally_crest_id: i32,
    pub sitting: bool,
    pub running: bool,
    pub in_combat: bool,
    pub alike_dead: bool,
    pub invisible: bool,
    pub mount_type: u8,
    pub store_type: u8,
    pub cubics: Vec<u16>,
    pub finding_party: u8,
    pub abnormal_effect: i32,
    pub recommendations_left: i32,
    pub recommendations_received: i32,
    pub mount_npc_id: i32,
    pub active_class_id: i32,
    pub enchant_effect: u8,
    pub team: u8,
    pub large_crest_id: i32,
    pub noble: u8,
    pub hero: u8,
    pub fishing: u8,
    pub fish_x: i32,
    pub fish_y: i32,
    pub fish_z: i32,
    pub name_color: i32,
}

pub fn read_char_info(body: &[u8]) -> Result<CharInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let heading = r.read_d()?;
    let object_id = r.read_d()?;
    let name = r.read_s()?;
    let race = r.read_d()?;
    let sex = r.read_d()?;
    let class_id = r.read_d()?;
    let mut equipped = [0i32; VISIBLE_EQUIP_SLOTS];
    for slot in &mut equipped {
        *slot = r.read_d()?;
    }
    let pvp_flag = r.read_d()?;
    let karma = r.read_d()?;
    let m_atk_speed = r.read_d()?;
    let p_atk_speed = r.read_d()?;
    r.read_d()?;
    r.read_d()?;
    let run_speed = r.read_d()?;
    let walk_speed = r.read_d()?;
    let swim_run_speed = r.read_d()?;
    let swim_walk_speed = r.read_d()?;
    let ground_run_speed = r.read_d()?;
    let ground_walk_speed = r.read_d()?;
    let fly_run_speed = r.read_d()?;
    let fly_walk_speed = r.read_d()?;
    let speed_multiplier = r.read_f()?;
    let attack_speed_multiplier = r.read_f()?;
    let collision_radius = r.read_f()?;
    let collision_height = r.read_f()?;
    let hair_style = r.read_d()?;
    let hair_color = r.read_d()?;
    let face = r.read_d()?;
    let title = r.read_s()?;
    let clan_id = r.read_d()?;
    let clan_crest_id = r.read_d()?;
    let ally_id = r.read_d()?;
    let ally_crest_id = r.read_d()?;
    r.read_d()?;
    let sitting = r.read_c()? == WAIT_TYPE_SIT;
    let running = r.read_c()? == 1;
    let in_combat = r.read_c()? == 1;
    let alike_dead = r.read_c()? == 1;
    let invisible = r.read_c()? == 1;
    let mount_type = r.read_c()?;
    let store_type = r.read_c()?;
    let count = r.read_h()? as usize;
    let mut cubics = Vec::with_capacity(count.min(16));
    for _ in 0..count {
        cubics.push(r.read_h()?);
    }
    let finding_party = r.read_c()?;
    let abnormal_effect = r.read_d()?;
    let recommendations_left = r.read_c()? as i32;
    let recommendations_received = r.read_h()? as i32;
    let mount_npc_id = r.read_d()?;
    let active_class_id = r.read_d()?;
    r.read_d()?;
    let enchant_effect = r.read_c()?;
    let team = r.read_c()?;
    let large_crest_id = r.read_d()?;
    let noble = r.read_c()?;
    let hero = r.read_c()?;
    let fishing = r.read_c()?;
    let fish_x = r.read_d()?;
    let fish_y = r.read_d()?;
    let fish_z = r.read_d()?;
    let name_color = r.read_d()?;
    Ok(CharInfo {
        x,
        y,
        z,
        heading,
        object_id,
        name,
        race,
        sex,
        class_id,
        equipped,
        pvp_flag,
        karma,
        m_atk_speed,
        p_atk_speed,
        run_speed,
        walk_speed,
        swim_run_speed,
        swim_walk_speed,
        ground_run_speed,
        ground_walk_speed,
        fly_run_speed,
        fly_walk_speed,
        speed_multiplier,
        attack_speed_multiplier,
        collision_radius,
        collision_height,
        hair_style,
        hair_color,
        face,
        title,
        clan_id,
        clan_crest_id,
        ally_id,
        ally_crest_id,
        sitting,
        running,
        in_combat,
        alike_dead,
        invisible,
        mount_type,
        store_type,
        cubics,
        finding_party,
        abnormal_effect,
        recommendations_left,
        recommendations_received,
        mount_npc_id,
        active_class_id,
        enchant_effect,
        team,
        large_crest_id,
        noble,
        hero,
        fishing,
        fish_x,
        fish_y,
        fish_z,
        name_color,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct NpcInfo {
    pub object_id: i32,
    pub npc_id: i32,
    pub attackable: bool,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub heading: i32,
    pub m_atk_speed: i32,
    pub p_atk_speed: i32,
    pub run_speed: i32,
    pub walk_speed: i32,
    pub speed_multiplier: f64,
    pub attack_speed_multiplier: f64,
    pub collision_radius: f64,
    pub collision_height: f64,
    pub rhand: i32,
    pub chest: i32,
    pub lhand: i32,
    pub name_above: bool,
    pub running: bool,
    pub in_combat: bool,
    pub alike_dead: bool,
    pub spawn_animation: u8,
    pub name: String,
    pub title: String,
    pub playable_flag: i32,
    pub pvp_flag: i32,
    pub karma: i32,
    pub abnormal_effect: i32,
    pub clan_id: i32,
    pub clan_crest_id: i32,
    pub ally_id: i32,
    pub ally_crest_id: i32,
    pub team: u8,
    pub base_collision_radius: f64,
    pub base_collision_height: f64,
}

pub fn read_npc_info(body: &[u8]) -> Result<NpcInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let npc_id = r.read_d()?.wrapping_sub(NPC_TYPE_OFFSET);
    let attackable = r.read_d()? == 1;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let heading = r.read_d()?;
    r.read_d()?;
    let m_atk_speed = r.read_d()?;
    let p_atk_speed = r.read_d()?;
    let run_speed = r.read_d()?;
    let walk_speed = r.read_d()?;
    for _ in 0..6 {
        r.read_d()?;
    }
    let speed_multiplier = r.read_f()?;
    let attack_speed_multiplier = r.read_f()?;
    let collision_radius = r.read_f()?;
    let collision_height = r.read_f()?;
    let rhand = r.read_d()?;
    let chest = r.read_d()?;
    let lhand = r.read_d()?;
    let name_above = r.read_c()? == 1;
    let running = r.read_c()? == 1;
    let in_combat = r.read_c()? == 1;
    let alike_dead = r.read_c()? == 1;
    let spawn_animation = r.read_c()?;
    let name = r.read_s()?;
    let title = r.read_s()?;
    let playable_flag = r.read_d()?;
    let pvp_flag = r.read_d()?;
    let karma = r.read_d()?;
    let abnormal_effect = r.read_d()?;
    let clan_id = r.read_d()?;
    let clan_crest_id = r.read_d()?;
    let ally_id = r.read_d()?;
    let ally_crest_id = r.read_d()?;
    r.read_c()?;
    let team = r.read_c()?;
    let base_collision_radius = r.read_f()?;
    let base_collision_height = r.read_f()?;
    r.read_d()?;
    Ok(NpcInfo {
        object_id,
        npc_id,
        attackable,
        x,
        y,
        z,
        heading,
        m_atk_speed,
        p_atk_speed,
        run_speed,
        walk_speed,
        speed_multiplier,
        attack_speed_multiplier,
        collision_radius,
        collision_height,
        rhand,
        chest,
        lhand,
        name_above,
        running,
        in_combat,
        alike_dead,
        spawn_animation,
        name,
        title,
        playable_flag,
        pvp_flag,
        karma,
        abnormal_effect,
        clan_id,
        clan_crest_id,
        ally_id,
        ally_crest_id,
        team,
        base_collision_radius,
        base_collision_height,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveToLocation {
    pub object_id: i32,
    pub to_x: i32,
    pub to_y: i32,
    pub to_z: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub fn read_move_to_location(body: &[u8]) -> Result<MoveToLocation> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let to_x = r.read_d()?;
    let to_y = r.read_d()?;
    let to_z = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    Ok(MoveToLocation { object_id, to_x, to_y, to_z, x, y, z })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopMove {
    pub object_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub heading: i32,
}

pub fn read_stop_move(body: &[u8]) -> Result<StopMove> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let heading = r.read_d()?;
    Ok(StopMove { object_id, x, y, z, heading })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidateLocation {
    pub object_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub heading: i32,
}

pub fn read_validate_location(body: &[u8]) -> Result<ValidateLocation> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let heading = r.read_d()?;
    Ok(ValidateLocation { object_id, x, y, z, heading })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeleportToLocation {
    pub target_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub instant: i32,
}

pub fn read_teleport_to_location(body: &[u8]) -> Result<TeleportToLocation> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let target_id = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let instant = r.read_d()?;
    Ok(TeleportToLocation { target_id, x, y, z, instant })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeleteObject {
    pub object_id: i32,
}

pub fn read_delete_object(body: &[u8]) -> Result<DeleteObject> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    r.read_d()?;
    Ok(DeleteObject { object_id })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusUpdate {
    pub object_id: i32,
    pub attributes: Vec<(i32, i32)>,
}

pub fn read_status_update(body: &[u8]) -> Result<StatusUpdate> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 8)?;
    let mut attributes = Vec::with_capacity(count);
    for _ in 0..count {
        let id = r.read_d()?;
        let value = r.read_d()?;
        attributes.push((id, value));
    }
    Ok(StatusUpdate { object_id, attributes })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChangeMoveType {
    pub object_id: i32,
    pub running: bool,
}

pub fn read_change_move_type(body: &[u8]) -> Result<ChangeMoveType> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let running = r.read_d()? != 0;
    r.read_d()?;
    Ok(ChangeMoveType { object_id, running })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChangeWaitType {
    pub object_id: i32,
    pub kind: i32,
    pub sitting: bool,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub fn read_change_wait_type(body: &[u8]) -> Result<ChangeWaitType> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let kind = r.read_d()?;
    let sitting = kind == WAIT_TYPE_SIT as i32;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    Ok(ChangeWaitType { object_id, kind, sitting, x, y, z })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Die {
    pub object_id: i32,
    pub to_village: bool,
    pub to_hideout: bool,
    pub to_castle: bool,
    pub to_siege_hq: bool,
    pub sweepable: bool,
    pub to_fixed_spot: bool,
}

pub fn read_die(body: &[u8]) -> Result<Die> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let to_village = r.read_d()? != 0;
    let to_hideout = r.read_d()? != 0;
    let to_castle = r.read_d()? != 0;
    let to_siege_hq = r.read_d()? != 0;
    let sweepable = r.read_d()? != 0;
    let to_fixed_spot = if r.remaining() >= 4 { r.read_d()? != 0 } else { false };
    Ok(Die { object_id, to_village, to_hideout, to_castle, to_siege_hq, sweepable, to_fixed_spot })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Revive {
    pub object_id: i32,
}

pub fn read_revive(body: &[u8]) -> Result<Revive> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    Ok(Revive { object_id })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SocialAction {
    pub object_id: i32,
    pub action: i32,
}

pub fn read_social_action(body: &[u8]) -> Result<SocialAction> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let action = r.read_d()?;
    Ok(SocialAction { object_id, action })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Say2Body {
    Text { name: String, text: String },
    Tables { sys_string_id: i32, sys_msg_id: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Say2 {
    pub object_id: i32,
    pub kind: i32,
    pub body: Say2Body,
}

impl Say2 {
    pub fn text(object_id: i32, kind: i32, name: &str, text: &str) -> Say2 {
        Say2 { object_id, kind, body: Say2Body::Text { name: name.into(), text: text.into() } }
    }

    pub fn name(&self) -> &str {
        match &self.body {
            Say2Body::Text { name, .. } => name,
            Say2Body::Tables { .. } => "",
        }
    }
}

pub const SAY2_NAME_MAX_CHARS: usize = 0x60 / 2 - 1;
pub const SAY2_TEXT_MAX_CHARS: usize = 0x800 / 2 - 1;

pub fn read_say2(body: &[u8]) -> Result<Say2> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let kind = r.read_d()?;
    let body = if kind == CHAT_BOAT {
        let sys_string_id = r.read_d()?;
        let sys_msg_id = r.read_d()?;
        Say2Body::Tables { sys_string_id, sys_msg_id }
    } else {
        let name = r.read_s()?;
        let text = r.read_s()?;
        if name.encode_utf16().count() > SAY2_NAME_MAX_CHARS || text.encode_utf16().count() > SAY2_TEXT_MAX_CHARS {
            return Err(WireError::TooLong);
        }
        Say2Body::Text { name, text }
    };
    Ok(Say2 { object_id, kind, body })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcSay {
    pub object_id: i32,
    pub kind: i32,
    pub npc_id: i32,
    pub text: String,
}

pub fn read_npc_say(body: &[u8]) -> Result<NpcSay> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let kind = r.read_d()?;
    let npc_id = r.read_d()?.wrapping_sub(NPC_TYPE_OFFSET);
    let text = r.read_s()?;
    Ok(NpcSay { object_id, kind, npc_id, text })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemMessageArg {
    Text(String),
    Number(i32),
    NpcName(i32),
    ItemName(i32),
    SkillName { id: i32, level: i32 },
    CastleName(i32),
    Unknown { kind: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemMessage {
    pub id: i32,
    pub args: Vec<SystemMessageArg>,
}

pub fn read_system_message(body: &[u8]) -> Result<SystemMessage> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let id = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 6)?;
    let mut args = Vec::with_capacity(count);
    for _ in 0..count {
        let kind = r.read_d()?;
        let arg = match kind {
            ARGUMENT_TEXT => {
                let text = r.read_s()?;
                if text.encode_utf16().count() > SYSTEM_MESSAGE_TEXT_MAX_CHARS {
                    return Err(WireError::TooLong);
                }
                SystemMessageArg::Text(text)
            }
            ARGUMENT_NUMBER | ARGUMENT_NPC_NAME | ARGUMENT_ITEM_NAME | ARGUMENT_CASTLE_NAME => {
                let value = r.read_d()?;
                match kind {
                    ARGUMENT_NUMBER => SystemMessageArg::Number(value),
                    ARGUMENT_NPC_NAME => SystemMessageArg::NpcName(value),
                    ARGUMENT_ITEM_NAME => SystemMessageArg::ItemName(value),
                    _ => SystemMessageArg::CastleName(value),
                }
            }
            ARGUMENT_SKILL_NAME => {
                let id = r.read_d()?;
                let level = r.read_d()?;
                SystemMessageArg::SkillName { id, level }
            }
            _ => SystemMessageArg::Unknown { kind },
        };
        args.push(arg);
    }
    Ok(SystemMessage { id, args })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetSelected {
    pub object_id: i32,
    pub target_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub fn read_target_selected(body: &[u8]) -> Result<TargetSelected> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let target_id = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    Ok(TargetSelected { object_id, target_id, x, y, z })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetUnselected {
    pub object_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub fn read_target_unselected(body: &[u8]) -> Result<TargetUnselected> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    Ok(TargetUnselected { object_id, x, y, z })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveToPawn {
    pub object_id: i32,
    pub target_id: i32,
    pub distance: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub fn read_move_to_pawn(body: &[u8]) -> Result<MoveToPawn> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let target_id = r.read_d()?;
    let distance = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    Ok(MoveToPawn { object_id, target_id, distance, x, y, z })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationChanged {
    pub object_id: i32,
    pub relation: i32,
    pub auto_attack: i32,
    pub karma: i32,
    pub pvp_flag: i32,
}

pub fn read_relation_changed(body: &[u8]) -> Result<RelationChanged> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let relation = r.read_d()?;
    let auto_attack = r.read_d()?;
    let karma = r.read_d()?;
    let pvp_flag = r.read_d()?;
    Ok(RelationChanged { object_id, relation, auto_attack, karma, pvp_flag })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MagicSkillUse {
    pub caster_id: i32,
    pub target_id: i32,
    pub skill_id: i32,
    pub skill_level: i32,
    pub hit_time: i32,
    pub reuse_delay: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub extra: Vec<u16>,
    pub positions: Vec<[i32; 3]>,
}

pub fn read_magic_skill_use(body: &[u8]) -> Result<MagicSkillUse> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let caster_id = r.read_d()?;
    let target_id = r.read_d()?;
    let skill_id = r.read_d()?;
    let skill_level = r.read_d()?;
    let hit_time = r.read_d()?;
    let reuse_delay = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let count = read_h_or_zero(&mut r)? as usize;
    let mut extra = Vec::with_capacity(count.min(16));
    for _ in 0..count {
        extra.push(read_h_or_zero(&mut r)?);
    }
    let count = read_h_or_zero(&mut r)? as usize;
    let mut positions = Vec::with_capacity(count.min(16));
    for _ in 0..count {
        positions.push([read_d_or_zero(&mut r)?, read_d_or_zero(&mut r)?, read_d_or_zero(&mut r)?]);
    }
    Ok(MagicSkillUse { caster_id, target_id, skill_id, skill_level, hit_time, reuse_delay, x, y, z, extra, positions })
}

pub(super) fn read_h_or_zero(r: &mut Reader) -> Result<u16> {
    if r.remaining() >= 2 { r.read_h() } else { Ok(0) }
}

pub(super) fn read_d_or_zero(r: &mut Reader) -> Result<i32> {
    if r.remaining() >= 4 { r.read_d() } else { Ok(0) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MagicSkillLaunched {
    pub caster_id: i32,
    pub skill_id: i32,
    pub skill_level: i32,
    pub targets: Vec<i32>,
}

pub fn read_magic_skill_launched(body: &[u8]) -> Result<MagicSkillLaunched> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let caster_id = r.read_d()?;
    let skill_id = r.read_d()?;
    let skill_level = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 4)?;
    let mut targets = Vec::with_capacity(count);
    for _ in 0..count {
        targets.push(r.read_d()?);
    }
    Ok(MagicSkillLaunched { caster_id, skill_id, skill_level, targets })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MagicSkillCanceled {
    pub object_id: i32,
}

pub fn read_magic_skill_canceled(body: &[u8]) -> Result<MagicSkillCanceled> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    Ok(MagicSkillCanceled { object_id })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnItem {
    pub object_id: i32,
    pub item_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub stackable: bool,
    pub count: i32,
    pub heading: i32,
}

pub fn read_spawn_item(body: &[u8]) -> Result<SpawnItem> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let item_id = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let stackable = r.read_d()? != 0;
    let count = r.read_d()?;
    let heading = r.read_d()?;
    Ok(SpawnItem { object_id, item_id, x, y, z, stackable, count, heading })
}

pub const ETC_STATUS_VALUES: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EtcStatusUpdate {
    pub values: [i32; ETC_STATUS_VALUES],
}

pub fn read_etc_status_update(body: &[u8]) -> Result<EtcStatusUpdate> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let mut values = [0i32; ETC_STATUS_VALUES];
    for value in &mut values {
        *value = r.read_d()?;
    }
    Ok(EtcStatusUpdate { values })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Buff {
    pub skill_id: i32,
    pub level: u16,
    pub duration: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MagicEffectIcons {
    pub effects: Vec<Buff>,
}

pub fn read_magic_effect_icons(body: &[u8]) -> Result<MagicEffectIcons> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let declared = r.read_h()?;
    let count = counted(&r, declared as i32, 10)?;
    let mut effects = Vec::with_capacity(count);
    for _ in 0..count {
        let skill_id = r.read_d()?;
        let level = r.read_h()?;
        let duration = r.read_d()?;
        effects.push(Buff { skill_id, level, duration });
    }
    Ok(MagicEffectIcons { effects })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PledgeStatusChanged {
    pub leader_id: i32,
    pub clan_id: i32,
    pub crest_id: i32,
    pub ally_id: i32,
    pub ally_crest_id: i32,
}

pub fn read_pledge_status_changed(body: &[u8]) -> Result<PledgeStatusChanged> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let leader_id = r.read_d()?;
    let clan_id = r.read_d()?;
    let crest_id = r.read_d()?;
    let ally_id = r.read_d()?;
    let ally_crest_id = r.read_d()?;
    r.read_d()?;
    Ok(PledgeStatusChanged { leader_id, clan_id, crest_id, ally_id, ally_crest_id })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroCommand {
    pub index: u8,
    pub kind: u8,
    pub skill_id: i32,
    pub shortcut_id: u8,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Macro {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub acronym: String,
    pub icon: u8,
    pub commands: Vec<MacroCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendMacroList {
    pub flag: u8,
    pub revision: i32,
    pub total_count: u8,
    pub macros: Vec<Macro>,
}

pub const MACRO_NAME_MAX_CHARS: usize = 0x1a / 2 - 1;
pub const MACRO_DESCRIPTION_MAX_CHARS: usize = 0x42 / 2 - 1;
pub const MACRO_ACRONYM_MAX_CHARS: usize = 0xa / 2 - 1;
pub const MACRO_COMMANDS_MAX: usize = 12;
pub const MACROS_PER_PACKET_MAX: usize = 100;

pub fn read_send_macro_list(body: &[u8]) -> Result<SendMacroList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let flag = r.read_c()?;
    let revision = r.read_d()?;
    let total_count = r.read_c()?;
    let count = (r.read_c()? as usize).min(MACROS_PER_PACKET_MAX);
    let mut macros = Vec::with_capacity(count);
    for _ in 0..count {
        let id = r.read_d()?;
        let name = r.read_s()?;
        let description = r.read_s()?;
        let acronym = r.read_s()?;
        let icon = r.read_c()?;
        let commands_count = (r.read_c()? as usize).min(MACRO_COMMANDS_MAX);
        if name.encode_utf16().count() > MACRO_NAME_MAX_CHARS
            || description.encode_utf16().count() > MACRO_DESCRIPTION_MAX_CHARS
            || acronym.encode_utf16().count() > MACRO_ACRONYM_MAX_CHARS
        {
            return Err(WireError::TooLong);
        }
        let mut commands = Vec::with_capacity(commands_count);
        for _ in 0..commands_count {
            let index = r.read_c()?;
            let kind = r.read_c()?;
            let skill_id = r.read_d()?;
            let shortcut_id = r.read_c()?;
            let command = r.read_s()?;
            commands.push(MacroCommand { index, kind, skill_id, shortcut_id, command });
        }
        macros.push(Macro { id, name, description, acronym, icon, commands });
    }
    Ok(SendMacroList { flag, revision, total_count, macros })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InventoryItem {
    pub type1: i16,
    pub object_id: i32,
    pub item_id: i32,
    pub count: i32,
    pub type2: i16,
    pub custom_type1: i16,
    pub equipped: bool,
    pub body_part: i32,
    pub enchant: i16,
    pub custom_type2: i16,
}

pub(super) fn read_inventory_item(r: &mut Reader) -> Result<InventoryItem> {
    let type1 = r.read_h()? as i16;
    let object_id = r.read_d()?;
    let item_id = r.read_d()?;
    let count = r.read_d()?;
    let type2 = r.read_h()? as i16;
    let custom_type1 = r.read_h()? as i16;
    let equipped = r.read_h()? != 0;
    let body_part = r.read_d()?;
    let enchant = r.read_h()? as i16;
    let custom_type2 = r.read_h()? as i16;
    Ok(InventoryItem {
        type1,
        object_id,
        item_id,
        count,
        type2,
        custom_type1,
        equipped,
        body_part,
        enchant,
        custom_type2,
    })
}

pub(super) const INVENTORY_ITEM_SIZE: usize = 2 + 4 + 4 + 4 + 2 + 2 + 2 + 4 + 2 + 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemList {
    pub show_window: bool,
    pub items: Vec<InventoryItem>,
}

pub fn read_item_list(body: &[u8]) -> Result<ItemList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let show_window = r.read_h()? != 0;
    let declared = r.read_h()?;
    let count = counted(&r, declared as i32, INVENTORY_ITEM_SIZE)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        items.push(read_inventory_item(&mut r)?);
    }
    Ok(ItemList { show_window, items })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemChange {
    pub change: u16,
    pub item: InventoryItem,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryUpdate {
    pub changes: Vec<ItemChange>,
}

pub fn read_inventory_update(body: &[u8]) -> Result<InventoryUpdate> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let declared = r.read_h()?;
    let count = counted(&r, declared as i32, 2 + INVENTORY_ITEM_SIZE)?;
    let mut changes = Vec::with_capacity(count);
    for _ in 0..count {
        let change = r.read_h()?;
        let item = read_inventory_item(&mut r)?;
        changes.push(ItemChange { change, item });
    }
    Ok(InventoryUpdate { changes })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EquipUpdate {
    pub change: i32,
    pub object_id: i32,
    pub body_part: i32,
}

pub fn read_equip_update(body: &[u8]) -> Result<EquipUpdate> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let change = r.read_d()?;
    let object_id = r.read_d()?;
    let body_part = r.read_d()?;
    Ok(EquipUpdate { change, object_id, body_part })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutKind {
    Item { object_id: i32 },
    Skill { skill_id: i32, level: i32 },
    Action { id: i32 },
    Macro { id: i32 },
    Recipe { id: i32 },
    Other { kind: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shortcut {
    pub slot: i32,
    pub kind: ShortcutKind,
    pub character_type: i32,
}

impl Shortcut {
    pub fn page(&self) -> i32 {
        self.slot / SHORTCUT_SLOTS_PER_PAGE
    }

    pub fn slot_in_page(&self) -> i32 {
        self.slot % SHORTCUT_SLOTS_PER_PAGE
    }
}

fn read_shortcut(r: &mut Reader) -> Result<Shortcut> {
    let kind = r.read_d()?;
    let slot = r.read_d()?;
    let kind = match kind {
        SHORTCUT_ITEM => ShortcutKind::Item { object_id: r.read_d()? },
        SHORTCUT_SKILL => {
            let skill_id = r.read_d()?;
            let level = r.read_d()?;
            ShortcutKind::Skill { skill_id, level }
        }
        SHORTCUT_ACTION => ShortcutKind::Action { id: r.read_d()? },
        SHORTCUT_MACRO => ShortcutKind::Macro { id: r.read_d()? },
        SHORTCUT_RECIPE => ShortcutKind::Recipe { id: r.read_d()? },
        kind => ShortcutKind::Other { kind },
    };
    let character_type = r.read_d()?;
    Ok(Shortcut { slot, kind, character_type })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortCutInit {
    pub shortcuts: Vec<Shortcut>,
}

pub fn read_short_cut_init(body: &[u8]) -> Result<ShortCutInit> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 12)?;
    let mut shortcuts = Vec::with_capacity(count);
    for _ in 0..count {
        shortcuts.push(read_shortcut(&mut r)?);
    }
    Ok(ShortCutInit { shortcuts })
}

pub fn read_short_cut_register(body: &[u8]) -> Result<Shortcut> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let shortcut = read_shortcut(&mut r)?;
    Ok(shortcut)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortCutDelete {
    pub slot: i32,
}

pub fn read_short_cut_delete(body: &[u8]) -> Result<ShortCutDelete> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let slot = r.read_d()?;
    if r.remaining() == 4 {
        r.read_d()?;
    }
    Ok(ShortCutDelete { slot })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillEntry {
    pub passive: bool,
    pub level: i32,
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillList {
    pub skills: Vec<SkillEntry>,
}

pub fn read_skill_list(body: &[u8]) -> Result<SkillList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 12)?;
    let mut skills = Vec::with_capacity(count);
    for _ in 0..count {
        let passive = r.read_d()? != 0;
        let level = r.read_d()?;
        let id = r.read_d()?;
        skills.push(SkillEntry { passive, level, id });
    }
    Ok(SkillList { skills })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcHtmlMessage {
    pub npc_object_id: i32,
    pub html: String,
    pub item_id: i32,
}

pub fn read_npc_html_message(body: &[u8]) -> Result<NpcHtmlMessage> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let npc_object_id = r.read_d()?;
    let html = r.read_s()?;
    if html.encode_utf16().count() > NPC_HTML_MAX_CHARS {
        return Err(WireError::TooLong);
    }
    let item_id = r.read_d()?;
    Ok(NpcHtmlMessage { npc_object_id, html, item_id })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MyTargetSelected {
    pub object_id: i32,
    pub color: i16,
}

pub fn read_my_target_selected(body: &[u8]) -> Result<MyTargetSelected> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let color = r.read_h()? as i16;
    Ok(MyTargetSelected { object_id, color })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillReuse {
    pub skill_id: i32,
    pub level: i32,
    pub reuse: i32,
    pub remaining: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillCoolTime {
    pub entries: Vec<SkillReuse>,
}

pub fn read_skill_cool_time(body: &[u8]) -> Result<SkillCoolTime> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 16)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let skill_id = r.read_d()?;
        let level = r.read_d()?;
        let reuse = r.read_d()?;
        let remaining = r.read_d()?;
        entries.push(SkillReuse { skill_id, level, reuse, remaining });
    }
    Ok(SkillCoolTime { entries })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionFailed;

pub fn read_action_failed(body: &[u8]) -> Result<ActionFailed> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(ActionFailed)
}

pub fn request_item_list() -> Vec<u8> {
    vec![OP_REQUEST_ITEM_LIST]
}

pub fn use_item(object_id: i32, ctrl: bool) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_USE_ITEM);
    w.write_d(object_id);
    w.write_d(ctrl as i32);
    w.into_bytes()
}

pub fn request_unequip_item(body_part: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_UNEQUIP_ITEM);
    w.write_d(body_part);
    w.into_bytes()
}

pub fn request_bypass_to_server(command: &str) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_BYPASS_TO_SERVER);
    w.write_s(command);
    w.into_bytes()
}

pub fn request_link_html(link: &str) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_LINK_HTML);
    w.write_s(link);
    w.into_bytes()
}

fn id_ctrl_shift(opcode: u8, id: i32, ctrl: bool, shift: bool) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(opcode);
    w.write_d(id);
    w.write_d(ctrl as i32);
    w.write_c(shift as u8);
    w.into_bytes()
}

pub fn request_magic_skill_use(skill_id: i32, ctrl: bool, shift: bool) -> Vec<u8> {
    id_ctrl_shift(OP_REQUEST_MAGIC_SKILL_USE, skill_id, ctrl, shift)
}

pub fn request_action_use(action_id: i32, ctrl: bool, shift: bool) -> Vec<u8> {
    id_ctrl_shift(OP_REQUEST_ACTION_USE, action_id, ctrl, shift)
}

pub fn request_shortcut_reg(kind: i32, slot: i32, id: i32, character_type_or_level: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_SHORT_CUT_REG);
    w.write_d(kind);
    w.write_d(slot);
    w.write_d(id);
    w.write_d(character_type_or_level);
    w.into_bytes()
}

pub fn request_shortcut_del(slot: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_SHORT_CUT_DEL);
    w.write_d(slot);
    w.into_bytes()
}

pub fn request_shortcut_use(slot: i32, page: i32, ctrl: u8) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_SHORT_CUT_USE);
    w.write_d(slot);
    w.write_d(page);
    w.write_c(ctrl);
    w.into_bytes()
}

pub fn request_dismiss_party() -> Vec<u8> {
    vec![OP_REQUEST_DISMISS_PARTY]
}

pub fn request_gm_list() -> Vec<u8> {
    vec![OP_REQUEST_GM_LIST]
}

pub fn revive_reply(reply: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REVIVE_REPLY);
    w.write_d(reply);
    w.into_bytes()
}

pub fn send_time_check(first: i32, second: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_SEND_TIME_CHECK);
    w.write_d(first);
    w.write_d(second);
    w.into_bytes()
}

pub fn request_remain_time() -> Vec<u8> {
    vec![OP_REQUEST_REMAIN_TIME]
}

pub fn net_ping(first: i32, second: i32, third: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_NET_PING);
    w.write_d(first);
    w.write_d(second);
    w.write_d(third);
    w.into_bytes()
}

pub fn request_destroy_item(object_id: i32, count: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_DESTROY_ITEM);
    w.write_d(object_id);
    w.write_d(count);
    w.into_bytes()
}

pub fn request_target_cancel() -> Vec<u8> {
    request_target_cancel_with(0)
}

pub fn request_target_cancel_with(unselect: u16) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_TARGET_CANCEL);
    w.write_h(unselect);
    w.into_bytes()
}

pub fn request_skill_cool_time() -> Vec<u8> {
    vec![OP_REQUEST_SKILL_COOL_TIME]
}

pub(super) fn counted(r: &Reader, count: i32, min_size: usize) -> Result<usize> {
    usize::try_from(count).ok().filter(|n| n * min_size <= r.remaining()).ok_or(WireError::Short)
}

pub fn appearing() -> Vec<u8> {
    vec![OP_APPEARING]
}

pub fn move_backward_to_location(target: [i32; 3], origin: [i32; 3]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_MOVE_BACKWARD_TO_LOCATION);
    for v in target.into_iter().chain(origin) {
        w.write_d(v);
    }
    w.write_d(MOVEMENT_MODE_MOUSE);
    w.into_bytes()
}

pub fn validate_position(pos: [i32; 3], heading: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_VALIDATE_POSITION);
    for v in pos {
        w.write_d(v);
    }
    w.write_d(heading);
    w.write_d(0);
    w.into_bytes()
}

pub fn validate_position_in_vehicle(vehicle_id: i32, pos: [i32; 3], heading: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_VALIDATE_POSITION);
    for v in pos {
        w.write_d(v);
    }
    w.write_d(heading);
    w.write_d(vehicle_id);
    w.into_bytes()
}

pub fn action(object_id: i32, origin: [i32; 3], shift: bool) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_ACTION);
    w.write_d(object_id);
    for v in origin {
        w.write_d(v);
    }
    w.write_c(shift as u8);
    w.into_bytes()
}

pub fn request_magic_skill_list(first: i32, second: i32, third: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_MAGIC_SKILL_LIST);
    w.write_d(first);
    w.write_d(second);
    w.write_d(third);
    w.into_bytes()
}

pub fn say2(text: &str, kind: i32, target: &str) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_SAY2_CLIENT);
    w.write_s(text);
    w.write_d(kind);
    if kind == CHAT_TELL {
        w.write_s(target);
    }
    w.into_bytes()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientSetTime {
    pub minutes: i32,
    pub ratio: i32,
}

pub fn read_client_set_time(body: &[u8]) -> Result<ClientSetTime> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let minutes = r.read_d()?;
    let ratio = r.read_d()?;
    Ok(ClientSetTime { minutes, ratio })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::login::{Authorization, GameServer};

    fn entry(name: &str, level: i32) -> Vec<u8> {
        let mut w = Writer::new();
        w.write_s(name);
        w.write_d(268481001);
        w.write_s("conta");
        for i in 0..10 {
            w.write_d(i);
        }
        w.write_f(7620.0);
        w.write_f(2100.0);
        w.write_d(5);
        w.write_d(6);
        w.write_d(level);
        w.write_d(0);
        for i in 0..44 {
            w.write_d(100 + i);
        }
        w.write_f(8000.0);
        w.write_f(2500.0);
        w.write_d(0);
        w.write_d(88);
        w.write_d(1);
        w.write_c(0);
        w.into_bytes()
    }

    #[test]
    fn protocol_version_is_261_bytes_of_656() {
        let body = protocol_version();
        assert_eq!(body.len(), 261);
        assert_eq!(body[0], 0);
        assert_eq!(&body[1..5], &656i32.to_le_bytes());
        assert_eq!(body[5], 0x09);
        assert_eq!(body[260], 0x06);
    }

    #[test]
    fn auth_login_orders_keys_play2_play1_login1_login2() {
        let auth = Authorization {
            login_key: [11, 22],
            play_key: [33, 44],
            server: GameServer { id: 1, ip: [0; 4], port: 0, players: 0, max_players: 0, online: true, extra: Default::default() },
        };
        let body = auth_login("ab", &auth);
        assert_eq!(body[0], 0x08);
        assert_eq!(&body[1..7], &[b'a', 0, b'b', 0, 0, 0]);
        assert_eq!(&body[7..11], &44i32.to_le_bytes());
        assert_eq!(&body[11..15], &33i32.to_le_bytes());
        assert_eq!(&body[15..19], &11i32.to_le_bytes());
        assert_eq!(&body[19..23], &22i32.to_le_bytes());
    }

    #[test]
    fn request_game_start_1030c4b9_writes_cdhddd_with_the_transfer_flag_and_position() {
        let body = character_selected_transfer(2, true, [10, -20, 30]);
        let mut expected = vec![OP_CHARACTER_SELECTED, 2, 0, 0, 0, 1, 0];
        for value in [10i32, -20, 30] {
            expected.extend(value.to_le_bytes());
        }
        assert_eq!(body, expected);
        assert_eq!(character_selected_transfer(1, false, [0; 3]), character_selected(1));
    }

    #[test]
    fn character_selected_and_enter_world_shapes() {
        assert_eq!(character_selected(2), [0x0d, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let mut narcasse = vec![0x03u8];
        narcasse.extend("narcasse".encode_utf16().flat_map(|u| u.to_le_bytes()));
        narcasse.extend_from_slice(&[0, 0]);
        assert_eq!(enter_world(), narcasse);
    }

    #[test]
    fn char_select_info_reads_two_entries() {
        let mut body = vec![0x13u8];
        body.extend_from_slice(&2i32.to_le_bytes());
        body.extend(entry("Luzal", 40));
        body.extend(entry("Bravo", 7));
        let chars = read_char_select_info(&body).unwrap();
        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0].name, "Luzal");
        assert_eq!(chars[0].level, 40);
        assert_eq!(chars[0].hp, 7620.0);
        assert_eq!(chars[0].max_hp, 8000.0);
        assert_eq!(chars[0].class_id, 88);
        assert_eq!(chars[0].active, 6);
        assert_eq!(chars[0].x, 7);
        assert_eq!(chars[0].karma, 0);
        assert_eq!(chars[0].paperdoll_objects[0], 100 + 9);
        assert_eq!(chars[0].paperdoll_items[0], 100 + 25);
        assert_eq!(chars[0].paperdoll_items[15], 100 + 40);
        assert_eq!(chars[0].hair_style, 100 + 41);
        assert_eq!(chars[0].hair_color, 100 + 42);
        assert_eq!(chars[0].face, 100 + 43);
        assert_eq!(chars[1].name, "Bravo");
        assert_eq!(chars[1].level, 7);
    }

    fn character_selected_body(count: i32, bin: &[u8]) -> Vec<u8> {
        let mut w = Writer::new();
        w.write_c(OP_CHAR_SELECTED);
        w.write_s("Luzal");
        w.write_d(7);
        w.write_s("Title");
        for v in [11, 22, 0, 1, 2, 18, 1, 100, 200, 300] {
            w.write_d(v);
        }
        w.write_f(55.5);
        w.write_f(44.5);
        for v in [5, 6, 40, -3] {
            w.write_d(v);
        }
        for i in 0..40 {
            w.write_d(1000 + i);
        }
        w.write_d(count);
        if count > 0 {
            w.write_d(bin.len() as i32);
            w.write_b(bin);
            for i in 0..count {
                for k in 0..4 {
                    w.write_d(i * 10 + k);
                }
            }
        }
        w.write_d(99);
        for v in [1, 2, 3, 4] {
            w.write_d(v);
        }
        w.into_bytes()
    }

    #[test]
    fn character_selected_103ffcc0_reads_the_format_the_inspector_loop_and_the_tail() {
        let selected = read_character_selected(&character_selected_body(0, &[])).unwrap();
        assert_eq!((selected.name.as_str(), selected.char_id, selected.title.as_str()), ("Luzal", 7, "Title"));
        assert_eq!((selected.session_id, selected.clan_id, selected.sex, selected.race, selected.class_id), (11, 22, 1, 2, 18));
        assert_eq!((selected.x, selected.y, selected.z, selected.hp, selected.mp), (100, 200, 300, 55.5, 44.5));
        assert_eq!((selected.sp, selected.exp, selected.level, selected.karma), (5, 6, 40, -3));
        assert_eq!((selected.middle[0], selected.middle[39]), (1000, 1039));
        assert!(selected.inspector_bin.is_empty() && selected.inspector_params.is_empty());
        assert_eq!((selected.tail_head, selected.tail), (99, [1, 2, 3, 4]));
        let with_loop = read_character_selected(&character_selected_body(2, b"abc")).unwrap();
        assert_eq!(with_loop.inspector_bin, b"abc");
        assert_eq!(with_loop.inspector_params, vec![[0, 1, 2, 3], [10, 11, 12, 13]]);
        assert_eq!((with_loop.tail_head, with_loop.tail), (99, [1, 2, 3, 4]));
        assert!(read_character_selected(&character_selected_body(2, b"abc")[..60]).is_err());
    }

    #[test]
    fn empty_char_select_info() {
        assert!(read_char_select_info(&[0x13, 0, 0, 0, 0]).unwrap().is_empty());
    }

    #[test]
    fn truncated_char_select_info_is_an_error() {
        let mut body = vec![0x13u8];
        body.extend_from_slice(&1i32.to_le_bytes());
        body.extend(&entry("Luzal", 40)[..30]);
        assert!(read_char_select_info(&body).is_err());
    }

    #[test]
    fn empty_body_is_an_error_not_a_panic() {
        assert!(read_char_select_info(&[]).is_err());
    }

    #[test]
    fn user_info_head_reads_position_and_identity_and_ignores_the_rest() {
        let mut w = Writer::new();
        w.write_c(0x04);
        w.write_d(83400);
        w.write_d(148000);
        w.write_d(-3400);
        w.write_d(16384);
        w.write_d(268481001);
        w.write_s("Wender");
        w.write_d(0);
        w.write_d(1);
        w.write_d(109);
        w.write_d(52);
        w.write_b(&[0u8; 500]);
        let head = read_user_info_head(&w.into_bytes()).unwrap();
        assert_eq!((head.x, head.y, head.z, head.heading), (83400, 148000, -3400, 16384));
        assert_eq!(head.object_id, 268481001);
        assert_eq!(head.name, "Wender");
        assert_eq!((head.race, head.sex, head.class_id, head.level), (0, 1, 109, 52));
    }

    fn user_info_clan_body(clan_id: i32, relation: i32, privileges: [u8; 32]) -> Vec<u8> {
        let mut w = Writer::new();
        w.write_c(0x04);
        for v in [1, 2, 3, 4, 5] {
            w.write_d(v);
        }
        w.write_s("W");
        for _ in 0..5 + 6 + 5 + 19 + 16 + 10 + 4 + 6 {
            w.write_d(0);
        }
        for _ in 0..4 {
            w.write_f(1.0);
        }
        for _ in 0..4 {
            w.write_d(0);
        }
        w.write_s("T");
        for v in [clan_id, 11, 12, 13, relation] {
            w.write_d(v);
        }
        for _ in 0..3 {
            w.write_c(0);
        }
        w.write_d(0);
        w.write_d(0);
        w.write_h(0);
        w.write_c(0);
        w.write_d(0);
        w.write_c(0);
        w.write_b(&privileges);
        w.write_b(&[0u8; 300]);
        w.into_bytes()
    }

    #[test]
    fn user_info_head_reads_the_relation_and_the_clan_rights_block_the_way_receive_user_info_does() {
        let mut block = [0u8; 32];
        block[0] = 0b0000_1011;
        block[1] = 0b0000_0100;
        let head = read_user_info_head(&user_info_clan_body(7, 0x40, block)).unwrap();
        assert_eq!((head.social.clan_id, head.social.relation), (7, 0x40));
        assert_eq!(head.social.pledge_rights, [true, true, true, true]);
        block[0] = 0b0000_0100;
        block[1] = 0b0000_0011;
        let head = read_user_info_head(&user_info_clan_body(7, 0, block)).unwrap();
        assert_eq!(head.social.pledge_rights, [false, false, false, false]);
        block[0] = 0b0000_0010;
        let head = read_user_info_head(&user_info_clan_body(7, 0, block)).unwrap();
        assert_eq!(head.social.pledge_rights, [false, true, false, false]);
        let head = read_user_info_head(&user_info_clan_body(0, 0, [0xff; 32])).unwrap();
        assert_eq!(head.social.pledge_rights, [false; 4]);
    }

    #[test]
    fn user_info_head_reads_vitals_speeds_flags_collision_and_looks() {
        let mut w = Writer::new();
        w.write_c(0x04);
        for v in [1, 2, 3, 4, 5] {
            w.write_d(v);
        }
        w.write_s("W");
        w.write_d(0);
        w.write_d(0);
        w.write_d(0);
        w.write_d(1);
        w.write_d(12345);
        for _ in 0..6 {
            w.write_d(0);
        }
        w.write_d(900);
        w.write_d(400);
        w.write_d(300);
        w.write_d(200);
        w.write_d(777);
        for _ in 0..3 + 16 {
            w.write_d(0);
        }
        for item in 1..=16 {
            w.write_d(item * 100);
        }
        for _ in 0..10 {
            w.write_d(0);
        }
        w.write_d(1);
        w.write_d(30);
        w.write_d(117);
        w.write_d(80);
        for _ in 0..6 {
            w.write_d(0);
        }
        w.write_f(1.5);
        w.write_f(1.1);
        w.write_f(9.0);
        w.write_f(23.0);
        w.write_d(2);
        w.write_d(1);
        w.write_d(3);
        w.write_d(0);
        w.write_s("Titulo");
        for _ in 0..5 {
            w.write_d(0);
        }
        w.write_c(0);
        w.write_c(0);
        w.write_c(0);
        w.write_d(0);
        w.write_d(0);
        w.write_h(2);
        w.write_h(1);
        w.write_h(2);
        w.write_c(0);
        w.write_d(i32::MIN | 0x10040);
        w.write_c(0);
        for _ in 0..8 {
            w.write_d(0);
        }
        w.write_h(0);
        w.write_h(0);
        w.write_d(0);
        w.write_h(0);
        w.write_d(0);
        w.write_d(0);
        w.write_d(500);
        w.write_d(450);
        w.write_c(20);
        w.write_b(&[0u8; 300]);
        let head = read_user_info_head(&w.into_bytes()).unwrap();
        assert_eq!((head.level, head.exp, head.stats.sp), (1, 12345, 777));
        assert_eq!((head.max_hp, head.hp, head.max_mp, head.mp), (900, 400, 300, 200));
        assert_eq!((head.max_cp, head.cp), (500, 450));
        assert_eq!(head.enchant_effect, 20);
        assert_eq!(head.abnormal_effect, i32::MIN | 0x10040);
        assert_eq!((head.pvp_flag, head.karma), (1, 30));
        assert_eq!((head.run_speed, head.walk_speed), (117, 80));
        assert_eq!(head.speed_multiplier, 1.5);
        assert_eq!((head.collision_radius, head.collision_height), (9.0, 23.0));
        assert_eq!((head.hair_style, head.hair_color, head.face), (2, 1, 3));
        assert_eq!(head.title, "Titulo");
        assert_eq!(head.paperdoll[6].item_id, 700);
        let equipped = head.equipped();
        assert_eq!(equipped, [100, 700, 800, 900, 1000, 1100, 1200, 1300, 1400, 1500, 1600]);
        assert_eq!(equipped[EQUIP_DHAIR], 100);
        assert_eq!(equipped[EQUIP_HEAD], 700);
        assert_eq!(equipped[EQUIP_RHAND], 800);
        assert_eq!(equipped[EQUIP_LHAND], 900);
        assert_eq!(equipped[EQUIP_GLOVES], 1000);
        assert_eq!(equipped[EQUIP_CHEST], 1100);
        assert_eq!(equipped[EQUIP_LEGS], 1200);
        assert_eq!(equipped[EQUIP_FEET], 1300);
        assert_eq!(equipped[EQUIP_BACK], 1400);
        assert_eq!(equipped[EQUIP_LRHAND], 1500);
        assert_eq!(equipped[EQUIP_HAIR], 1600);
    }

    #[test]
    fn user_info_head_truncated_before_the_cp_is_an_error() {
        let mut w = Writer::new();
        w.write_c(0x04);
        for _ in 0..5 {
            w.write_d(0);
        }
        w.write_s("W");
        w.write_b(&[0u8; 4 * (4 + 7 + 4 + 4 + 32 + 10 + 2 + 2 + 6) + 8 * 4 + 4 * 4]);
        w.write_s("T");
        w.write_b(&[0u8; 4 * 5 + 3 + 8 + 2 + 1 + 4 + 1 + 32 + 2 + 2 + 4 + 2 + 4 + 4 + 4]);
        assert!(read_user_info_head(&w.into_bytes()).is_err());
    }

    #[test]
    fn user_info_head_truncated_before_the_title_is_an_error() {
        let mut w = Writer::new();
        w.write_c(0x04);
        for _ in 0..5 {
            w.write_d(0);
        }
        w.write_s("W");
        w.write_b(&[0u8; 4 * (4 + 7 + 4 + 4 + 32 + 10 + 2 + 2 + 6) + 8 * 4 + 4 * 4]);
        assert!(read_user_info_head(&w.into_bytes()).is_err());
    }

    #[test]
    fn restart_and_logout_shapes() {
        assert_eq!(request_restart(), [0x46]);
        assert_eq!(logout(), [0x09]);
    }

    #[test]
    fn restart_response_reads_result_and_message() {
        let mut w = Writer::new();
        w.write_c(OP_RESTART_RESPONSE);
        w.write_d(1);
        w.write_s("reiniciando");
        assert_eq!(read_restart_response(&w.into_bytes()).unwrap(), RestartResponse { result: 1, message: "reiniciando".into() });
        assert!(read_restart_response(&[OP_RESTART_RESPONSE, 1, 0, 0, 0]).is_err());
    }

    #[test]
    fn client_set_time_reads_minutes_and_ratio() {
        let body = [0xec, 0x68, 0x01, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00];
        let t = read_client_set_time(&body).unwrap();
        assert_eq!(t.minutes, 360);
        assert_eq!(t.ratio, 6);
    }

    #[test]
    fn client_set_time_round_trips_through_writer() {
        let mut w = Writer::new();
        w.write_c(OP_CLIENT_SET_TIME);
        w.write_d(420);
        w.write_d(6);
        let t = read_client_set_time(&w.into_bytes()).unwrap();
        assert_eq!(t, ClientSetTime { minutes: 420, ratio: 6 });
    }

    #[test]
    fn client_set_time_ignores_trailing_bytes_like_the_engine() {
        let body = [0xec, 0, 0, 0, 0, 0, 0, 0, 0, 0xff];
        assert!(read_client_set_time(&body).is_ok());
    }
}

#[cfg(test)]
mod world_tests {
    use super::*;

    fn char_info_body() -> Vec<u8> {
        char_info_body_with([1, 1, 0, 0, 0])
    }

    fn char_info_body_with(state: [u8; 5]) -> Vec<u8> {
        let mut w = Writer::new();
        w.write_c(OP_CHAR_INFO);
        w.write_d(83400);
        w.write_d(148000);
        w.write_d(-3400);
        w.write_d(16384);
        w.write_d(268481001);
        w.write_s("Luzal");
        w.write_d(1);
        w.write_d(0);
        w.write_d(20);
        for i in 0..11 {
            w.write_d(1000 + i);
        }
        w.write_d(0);
        w.write_d(-5);
        w.write_d(333);
        w.write_d(300);
        w.write_d(7);
        w.write_d(8);
        w.write_d(117);
        w.write_d(80);
        w.write_d(50);
        w.write_d(50);
        w.write_d(117);
        w.write_d(80);
        w.write_d(0);
        w.write_d(0);
        w.write_f(1.5);
        w.write_f(1.1);
        w.write_f(8.0);
        w.write_f(23.0);
        w.write_d(2);
        w.write_d(1);
        w.write_d(3);
        w.write_s("Titulo");
        w.write_d(100);
        w.write_d(101);
        w.write_d(102);
        w.write_d(103);
        w.write_d(9);
        for byte in state {
            w.write_c(byte);
        }
        w.write_c(0);
        w.write_c(0);
        w.write_h(2);
        w.write_h(11);
        w.write_h(12);
        w.write_c(1);
        w.write_d(0x40);
        w.write_c(5);
        w.write_h(6);
        w.write_d(0);
        w.write_d(20);
        w.write_d(9);
        w.write_c(2);
        w.write_c(1);
        w.write_d(0);
        w.write_c(1);
        w.write_c(0);
        w.write_c(0);
        w.write_d(0);
        w.write_d(0);
        w.write_d(0);
        w.write_d(0xffffff);
        w.into_bytes()
    }

    fn npc_info_body() -> Vec<u8> {
        let mut w = Writer::new();
        w.write_c(OP_NPC_INFO);
        w.write_d(1234);
        w.write_d(1_020_001);
        w.write_d(1);
        w.write_d(10);
        w.write_d(20);
        w.write_d(30);
        w.write_d(32768);
        w.write_d(0);
        w.write_d(333);
        w.write_d(253);
        w.write_d(120);
        w.write_d(60);
        for _ in 0..6 {
            w.write_d(0);
        }
        w.write_f(1.25);
        w.write_f(1.0);
        w.write_f(8.0);
        w.write_f(24.0);
        w.write_d(5);
        w.write_d(21);
        w.write_d(6);
        w.write_c(1);
        w.write_c(1);
        w.write_c(0);
        w.write_c(0);
        w.write_c(2);
        w.write_s("Gremlin");
        w.write_s("");
        w.write_d(0);
        w.write_d(1);
        w.write_d(-5);
        w.write_d(0x40);
        w.write_d(11);
        w.write_d(12);
        w.write_d(13);
        w.write_d(14);
        w.write_c(0);
        w.write_c(2);
        w.write_f(6.5);
        w.write_f(20.0);
        w.write_d(0);
        w.into_bytes()
    }

    fn ds(op: u8, values: &[i32]) -> Vec<u8> {
        let mut w = Writer::new();
        w.write_c(op);
        for v in values {
            w.write_d(*v);
        }
        w.into_bytes()
    }

    #[test]
    fn char_info_reads_every_field_in_l2g_order() {
        let c = read_char_info(&char_info_body()).unwrap();
        assert_eq!((c.x, c.y, c.z, c.heading), (83400, 148000, -3400, 16384));
        assert_eq!(c.object_id, 268481001);
        assert_eq!(c.name, "Luzal");
        assert_eq!((c.race, c.sex, c.class_id), (1, 0, 20));
        assert_eq!(c.equipped[0], 1000);
        assert_eq!(c.equipped[10], 1010);
        assert_eq!((c.pvp_flag, c.karma, c.m_atk_speed, c.p_atk_speed), (0, -5, 333, 300));
        assert_eq!((c.run_speed, c.walk_speed), (117, 80));
        assert_eq!((c.swim_run_speed, c.swim_walk_speed), (50, 50));
        assert_eq!((c.ground_run_speed, c.ground_walk_speed), (117, 80));
        assert_eq!((c.fly_run_speed, c.fly_walk_speed), (0, 0));
        assert_eq!(c.speed_multiplier, 1.5);
        assert_eq!(c.attack_speed_multiplier, 1.1);
        assert_eq!((c.collision_radius, c.collision_height), (8.0, 23.0));
        assert_eq!((c.hair_style, c.hair_color, c.face), (2, 1, 3));
        assert_eq!(c.title, "Titulo");
        assert_eq!((c.clan_id, c.clan_crest_id, c.ally_id, c.ally_crest_id), (100, 101, 102, 103));
        assert!(!c.sitting && c.running && !c.in_combat && !c.alike_dead && !c.invisible);
        assert_eq!((c.mount_type, c.store_type), (0, 0));
        assert_eq!(c.cubics, vec![11, 12]);
        assert_eq!(c.finding_party, 1);
        assert_eq!(c.abnormal_effect, 0x40);
        assert_eq!((c.recommendations_left, c.recommendations_received), (5, 6));
        assert_eq!((c.mount_npc_id, c.active_class_id), (0, 20));
        assert_eq!((c.enchant_effect, c.team, c.large_crest_id), (2, 1, 0));
        assert_eq!((c.noble, c.hero, c.fishing), (1, 0, 0));
        assert_eq!((c.fish_x, c.fish_y, c.fish_z), (0, 0, 0));
        assert_eq!(c.name_color, 0xffffff);
    }

    #[test]
    fn char_info_state_bytes_follow_the_engine_wait_type_and_flags() {
        let c = read_char_info(&char_info_body_with([0, 0, 1, 1, 1])).unwrap();
        assert!(c.sitting && !c.running && c.in_combat && c.alike_dead && c.invisible);
        let c = read_char_info(&char_info_body_with([1, 1, 0, 0, 0])).unwrap();
        assert!(!c.sitting && c.running && !c.in_combat && !c.alike_dead && !c.invisible);
        let c = read_char_info(&char_info_body_with([2, 0, 0, 0, 0])).unwrap();
        assert!(!c.sitting && !c.running);
    }

    #[test]
    fn truncated_char_info_is_an_error() {
        let body = char_info_body();
        assert!(read_char_info(&body[..body.len() - 3]).is_err());
        assert!(read_char_info(&body[..40]).is_err());
    }

    #[test]
    fn npc_info_strips_the_template_offset_and_reads_speeds() {
        let n = read_npc_info(&npc_info_body()).unwrap();
        assert_eq!(n.object_id, 1234);
        assert_eq!(n.npc_id, 20001);
        assert!(n.attackable);
        assert_eq!((n.x, n.y, n.z, n.heading), (10, 20, 30, 32768));
        assert_eq!((n.m_atk_speed, n.p_atk_speed), (333, 253));
        assert_eq!((n.run_speed, n.walk_speed), (120, 60));
        assert_eq!(n.speed_multiplier, 1.25);
        assert_eq!(n.attack_speed_multiplier, 1.0);
        assert_eq!((n.collision_radius, n.collision_height), (8.0, 24.0));
        assert_eq!((n.rhand, n.chest, n.lhand), (5, 21, 6));
        assert!(n.name_above && n.running && !n.in_combat && !n.alike_dead);
        assert_eq!(n.spawn_animation, 2);
        assert_eq!(n.name, "Gremlin");
        assert_eq!(n.title, "");
        assert_eq!((n.playable_flag, n.pvp_flag, n.karma, n.abnormal_effect), (0, 1, -5, 0x40));
        assert_eq!((n.clan_id, n.clan_crest_id, n.ally_id, n.ally_crest_id), (11, 12, 13, 14));
        assert_eq!(n.team, 2);
        assert_eq!((n.base_collision_radius, n.base_collision_height), (6.5, 20.0));
    }

    #[test]
    fn truncated_or_padded_npc_info_is_an_error() {
        let body = npc_info_body();
        assert!(read_npc_info(&body[..body.len() - 1]).is_err());
        assert!(read_npc_info(&body[..20]).is_err());
        let mut padded = body.clone();
        padded.push(0);
        assert!(read_npc_info(&padded).is_ok());
    }

    #[test]
    fn move_to_location_reads_destination_then_origin() {
        let m = read_move_to_location(&ds(OP_MOVE_TO_LOCATION, &[7, 100, 200, -30, 10, 20, -3])).unwrap();
        assert_eq!(m.object_id, 7);
        assert_eq!((m.to_x, m.to_y, m.to_z), (100, 200, -30));
        assert_eq!((m.x, m.y, m.z), (10, 20, -3));
        assert!(read_move_to_location(&ds(OP_MOVE_TO_LOCATION, &[7, 100, 200, -30, 10, 20])).is_err());
        assert!(read_move_to_location(&ds(OP_MOVE_TO_LOCATION, &[7, 100, 200, -30, 10, 20, -3, 0])).is_ok());
    }

    #[test]
    fn stop_move_and_validate_location_read_position_and_heading() {
        let s = read_stop_move(&ds(OP_STOP_MOVE, &[7, 1, 2, 3, 4])).unwrap();
        assert_eq!((s.object_id, s.x, s.y, s.z, s.heading), (7, 1, 2, 3, 4));
        let v = read_validate_location(&ds(OP_VALIDATE_LOCATION, &[8, 5, 6, 7, 8])).unwrap();
        assert_eq!((v.object_id, v.x, v.y, v.z, v.heading), (8, 5, 6, 7, 8));
        assert!(read_stop_move(&ds(OP_STOP_MOVE, &[7, 1, 2, 3])).is_err());
        assert!(read_validate_location(&ds(OP_VALIDATE_LOCATION, &[8, 5, 6, 7])).is_err());
    }

    #[test]
    fn teleport_and_delete_object() {
        let t = read_teleport_to_location(&ds(OP_TELEPORT_TO_LOCATION, &[9, 1, 2, 3, 0])).unwrap();
        assert_eq!((t.target_id, t.x, t.y, t.z, t.instant), (9, 1, 2, 3, 0));
        assert!(read_teleport_to_location(&ds(OP_TELEPORT_TO_LOCATION, &[9, 1, 2, 3])).is_err());
        let d = read_delete_object(&ds(OP_DELETE_OBJECT, &[9, 0])).unwrap();
        assert_eq!(d.object_id, 9);
        assert!(read_delete_object(&ds(OP_DELETE_OBJECT, &[9])).is_err());
    }

    #[test]
    fn status_update_reads_attribute_pairs() {
        let s = read_status_update(&ds(OP_STATUS_UPDATE, &[5, 2, ATTRIBUTE_CURRENT_HP, 400, ATTRIBUTE_MAX_HP, 900]))
            .unwrap();
        assert_eq!(s.object_id, 5);
        assert_eq!(s.attributes, vec![(ATTRIBUTE_CURRENT_HP, 400), (ATTRIBUTE_MAX_HP, 900)]);
        assert_eq!((ATTRIBUTE_CURRENT_HP, ATTRIBUTE_MAX_HP, ATTRIBUTE_CURRENT_MP, ATTRIBUTE_MAX_MP), (9, 10, 11, 12));
        assert_eq!(ATTRIBUTE_CURRENT_CP, 0x21);
        assert!(read_status_update(&ds(OP_STATUS_UPDATE, &[5, 2, 9, 400, 10])).is_err());
        assert!(read_status_update(&ds(OP_STATUS_UPDATE, &[5])).is_err());
    }

    #[test]
    fn move_and_wait_type() {
        let m = read_change_move_type(&ds(OP_CHANGE_MOVE_TYPE, &[5, 1, 0])).unwrap();
        assert_eq!(m.object_id, 5);
        assert!(m.running);
        assert!(!read_change_move_type(&ds(OP_CHANGE_MOVE_TYPE, &[5, 0, 0])).unwrap().running);
        assert!(read_change_move_type(&ds(OP_CHANGE_MOVE_TYPE, &[5, 1])).is_err());
        let w = read_change_wait_type(&ds(OP_CHANGE_WAIT_TYPE, &[5, 0, 1, 2, 3])).unwrap();
        assert_eq!((w.object_id, w.x, w.y, w.z), (5, 1, 2, 3));
        assert!(w.sitting);
        assert!(!read_change_wait_type(&ds(OP_CHANGE_WAIT_TYPE, &[5, 1, 1, 2, 3])).unwrap().sitting);
        assert!(read_change_wait_type(&ds(OP_CHANGE_WAIT_TYPE, &[5, 1, 1, 2])).is_err());
    }

    #[test]
    fn die_accepts_five_or_six_fields_and_revive_one() {
        let d = read_die(&ds(OP_DIE, &[5, 1, 0, 0, 0, 1])).unwrap();
        assert_eq!(d.object_id, 5);
        assert!(d.to_village && !d.to_hideout && d.sweepable && !d.to_fixed_spot);
        let d = read_die(&ds(OP_DIE, &[5, 1, 0, 0, 0, 0, 1])).unwrap();
        assert!(d.to_fixed_spot && !d.sweepable);
        assert!(read_die(&ds(OP_DIE, &[5, 1, 0, 0, 0])).is_err());
        assert!(read_die(&ds(OP_DIE, &[5, 1, 0, 0, 0, 0, 1, 1])).is_ok());
        assert_eq!(read_revive(&ds(OP_REVIVE, &[5])).unwrap().object_id, 5);
        assert!(read_revive(&[OP_REVIVE]).is_err());
    }

    #[test]
    fn social_action() {
        let s = read_social_action(&ds(OP_SOCIAL_ACTION, &[5, 3])).unwrap();
        assert_eq!((s.object_id, s.action), (5, 3));
        assert!(read_social_action(&ds(OP_SOCIAL_ACTION, &[5])).is_err());
    }

    #[test]
    fn say2_and_npc_say() {
        let mut w = Writer::new();
        w.write_c(OP_SAY2);
        w.write_d(5);
        w.write_d(CHAT_SHOUT);
        w.write_s("Luzal");
        w.write_s("oi");
        let body = w.into_bytes();
        let s = read_say2(&body).unwrap();
        assert_eq!((s.object_id, s.kind), (5, CHAT_SHOUT));
        assert_eq!(s.body, Say2Body::Text { name: "Luzal".into(), text: "oi".into() });
        assert_eq!(s.name(), "Luzal");
        assert!(read_say2(&body[..body.len() - 2]).is_err());
        let boat = read_say2(&ds(OP_SAY2, &[7, CHAT_BOAT, 1500, 1200])).unwrap();
        assert_eq!(boat.body, Say2Body::Tables { sys_string_id: 1500, sys_msg_id: 1200 });
        assert_eq!(boat.name(), "");
        assert!(read_say2(&ds(OP_SAY2, &[7, CHAT_BOAT, 1500])).is_err());
        assert!(read_say2(&ds(OP_SAY2, &[7, CHAT_BOAT, 1500, 1200, 0])).is_ok());
        let mut w = Writer::new();
        w.write_c(OP_SAY2);
        w.write_d(5);
        w.write_d(CHAT_ALL);
        w.write_s(&"n".repeat(SAY2_NAME_MAX_CHARS + 1));
        w.write_s("oi");
        assert_eq!(read_say2(&w.into_bytes()), Err(WireError::TooLong));
        let mut w = Writer::new();
        w.write_c(OP_SAY2);
        w.write_d(5);
        w.write_d(CHAT_ALL);
        w.write_s("n");
        w.write_s(&"t".repeat(SAY2_TEXT_MAX_CHARS + 1));
        assert_eq!(read_say2(&w.into_bytes()), Err(WireError::TooLong));
        let mut w = Writer::new();
        w.write_c(OP_NPC_SAY);
        w.write_d(6);
        w.write_d(CHAT_ALL);
        w.write_d(1_030_001);
        w.write_s("bem-vindo");
        let body = w.into_bytes();
        let n = read_npc_say(&body).unwrap();
        assert_eq!((n.object_id, n.kind, n.npc_id), (6, CHAT_ALL, 30001));
        assert_eq!(n.text, "bem-vindo");
        assert!(read_npc_say(&body[..9]).is_err());
    }

    #[test]
    fn system_message_reads_typed_arguments() {
        let mut w = Writer::new();
        w.write_c(OP_SYSTEM_MESSAGE);
        w.write_d(30);
        w.write_d(7);
        w.write_d(ARGUMENT_TEXT);
        w.write_s("Luzal");
        w.write_d(ARGUMENT_NUMBER);
        w.write_d(42);
        w.write_d(ARGUMENT_NPC_NAME);
        w.write_d(1_020_001);
        w.write_d(ARGUMENT_SKILL_NAME);
        w.write_d(3);
        w.write_d(2);
        w.write_d(ARGUMENT_CASTLE_NAME);
        w.write_d(1);
        w.write_d(ARGUMENT_ITEM_NAME);
        w.write_d(57);
        w.write_d(9);
        let body = w.into_bytes();
        let m = read_system_message(&body).unwrap();
        assert_eq!(m.id, 30);
        assert_eq!(
            m.args,
            vec![
                SystemMessageArg::Text("Luzal".into()),
                SystemMessageArg::Number(42),
                SystemMessageArg::NpcName(1_020_001),
                SystemMessageArg::SkillName { id: 3, level: 2 },
                SystemMessageArg::CastleName(1),
                SystemMessageArg::ItemName(57),
                SystemMessageArg::Unknown { kind: 9 },
            ]
        );
        assert!(read_system_message(&body[..body.len() - 8]).is_err());
        let empty = read_system_message(&ds(OP_SYSTEM_MESSAGE, &[30, 0])).unwrap();
        assert!(empty.args.is_empty());
        let mut w = Writer::new();
        w.write_c(OP_SYSTEM_MESSAGE);
        w.write_d(30);
        w.write_d(1);
        w.write_d(ARGUMENT_TEXT);
        w.write_s(&"x".repeat(SYSTEM_MESSAGE_TEXT_MAX_CHARS + 1));
        assert_eq!(read_system_message(&w.into_bytes()), Err(WireError::TooLong));
    }

    #[test]
    fn huge_counts_fail_on_the_body_not_on_a_cap() {
        let mut w = Writer::new();
        w.write_c(OP_STATUS_UPDATE);
        w.write_d(5);
        w.write_d(100);
        for i in 0..100 {
            w.write_d(i);
            w.write_d(i * 2);
        }
        let s = read_status_update(&w.into_bytes()).unwrap();
        assert_eq!(s.attributes.len(), 100);
        assert!(read_status_update(&ds(OP_STATUS_UPDATE, &[5, i32::MAX])).is_err());
        assert!(read_status_update(&ds(OP_STATUS_UPDATE, &[5, -1])).is_err());
        let mut w = Writer::new();
        w.write_c(OP_SYSTEM_MESSAGE);
        w.write_d(30);
        w.write_d(70);
        for _ in 0..70 {
            w.write_d(ARGUMENT_NUMBER);
            w.write_d(1);
        }
        assert_eq!(read_system_message(&w.into_bytes()).unwrap().args.len(), 70);
        assert!(read_system_message(&ds(OP_SYSTEM_MESSAGE, &[30, i32::MAX])).is_err());
        assert!(read_system_message(&ds(OP_SYSTEM_MESSAGE, &[30, -1])).is_err());
    }

    #[test]
    fn appearing_is_a_bare_opcode() {
        assert_eq!(appearing(), [OP_APPEARING]);
        assert_eq!(OP_APPEARING, 0x30);
    }

    #[test]
    fn move_backward_to_location_ends_with_mouse_mode() {
        let body = move_backward_to_location([100, 200, -30], [10, 20, -3]);
        assert_eq!(body, ds(OP_MOVE_BACKWARD_TO_LOCATION, &[100, 200, -30, 10, 20, -3, MOVEMENT_MODE_MOUSE]));
        assert_eq!(body.len(), 29);
    }

    #[test]
    fn validate_position_ends_with_a_zero_int() {
        assert_eq!(validate_position([1, 2, 3], 16384), ds(OP_VALIDATE_POSITION, &[1, 2, 3, 16384, 0]));
        assert_eq!(validate_position_in_vehicle(77, [1, 2, 3], 16384), ds(OP_VALIDATE_POSITION, &[1, 2, 3, 16384, 77]));
    }

    #[test]
    fn action_carries_origin_and_shift_byte() {
        let mut expected = ds(OP_ACTION, &[77, 1, 2, 3]);
        expected.push(1);
        assert_eq!(action(77, [1, 2, 3], true), expected);
        assert_eq!(request_magic_skill_list(2, 1, 3), [0x2e, 2, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0]);
        assert_eq!(*action(77, [1, 2, 3], false).last().unwrap(), 0);
    }

    #[test]
    fn say2_writes_target_only_for_tell() {
        let mut w = Writer::new();
        w.write_c(OP_SAY2_CLIENT);
        w.write_s("oi");
        w.write_d(CHAT_ALL);
        assert_eq!(say2("oi", CHAT_ALL, "Luzal"), w.into_bytes());
        let mut w = Writer::new();
        w.write_c(OP_SAY2_CLIENT);
        w.write_s("oi");
        w.write_d(CHAT_TELL);
        w.write_s("Luzal");
        assert_eq!(say2("oi", CHAT_TELL, "Luzal"), w.into_bytes());
    }
}

#[cfg(test)]
mod live_tests {
    use super::*;

    fn ds(op: u8, values: &[i32]) -> Vec<u8> {
        let mut w = Writer::new();
        w.write_c(op);
        for v in values {
            w.write_d(*v);
        }
        w.into_bytes()
    }

    #[test]
    fn target_selected_reads_selector_target_and_position() {
        let t = read_target_selected(&ds(OP_TARGET_SELECTED, &[7, 9, 1, 2, 3])).unwrap();
        assert_eq!(t, TargetSelected { object_id: 7, target_id: 9, x: 1, y: 2, z: 3 });
        assert!(read_target_selected(&ds(OP_TARGET_SELECTED, &[7, 9, 1, 2])).is_err());
        assert!(read_target_selected(&ds(OP_TARGET_SELECTED, &[7, 9, 1, 2, 3, 0])).is_ok());
    }

    #[test]
    fn target_unselected_is_four_ints() {
        let t = read_target_unselected(&ds(OP_TARGET_UNSELECTED, &[7, 1, 2, 3])).unwrap();
        assert_eq!(t, TargetUnselected { object_id: 7, x: 1, y: 2, z: 3 });
        assert!(read_target_unselected(&ds(OP_TARGET_UNSELECTED, &[7, 1, 2])).is_err());
        assert!(read_target_unselected(&ds(OP_TARGET_UNSELECTED, &[7, 1, 2, 3, 0])).is_ok());
    }

    #[test]
    fn move_to_pawn_reads_target_distance_and_origin() {
        let m = read_move_to_pawn(&ds(OP_MOVE_TO_PAWN, &[7, 9, 40, 1, 2, 3])).unwrap();
        assert_eq!(m, MoveToPawn { object_id: 7, target_id: 9, distance: 40, x: 1, y: 2, z: 3 });
        assert!(read_move_to_pawn(&ds(OP_MOVE_TO_PAWN, &[7, 9, 40, 1, 2])).is_err());
        assert!(read_move_to_pawn(&ds(OP_MOVE_TO_PAWN, &[7, 9, 40, 1, 2, 3, 4])).is_ok());
    }

    #[test]
    fn relation_changed_reads_five_ints() {
        let r = read_relation_changed(&ds(OP_RELATION_CHANGED, &[7, 2, 1, 30, 1])).unwrap();
        assert_eq!(r, RelationChanged { object_id: 7, relation: 2, auto_attack: 1, karma: 30, pvp_flag: 1 });
        assert!(read_relation_changed(&ds(OP_RELATION_CHANGED, &[7, 2, 1, 30])).is_err());
        assert!(read_relation_changed(&ds(OP_RELATION_CHANGED, &[7, 2, 1, 30, 1, 0])).is_ok());
    }

    #[test]
    fn magic_skill_use_reads_nine_ints_then_the_two_lists_like_the_engine() {
        let body = ds(OP_MAGIC_SKILL_USE, &[7, 9, 1204, 1, 4000, 10000, 1, 2, 3]);
        let m = read_magic_skill_use(&body).unwrap();
        assert_eq!(
            m,
            MagicSkillUse {
                caster_id: 7,
                target_id: 9,
                skill_id: 1204,
                skill_level: 1,
                hit_time: 4000,
                reuse_delay: 10000,
                x: 1,
                y: 2,
                z: 3,
                extra: Vec::new(),
                positions: Vec::new(),
            }
        );
        let mut lisvus_crit = body.clone();
        lisvus_crit.extend_from_slice(&1i32.to_le_bytes());
        let crit = read_magic_skill_use(&lisvus_crit).unwrap();
        assert_eq!(crit.extra, vec![0]);
        assert!(crit.positions.is_empty());
        let mut full = body.clone();
        let mut w = Writer::new();
        w.write_h(2);
        w.write_h(5);
        w.write_h(6);
        w.write_h(1);
        w.write_d(10);
        w.write_d(20);
        w.write_d(30);
        full.extend(w.into_bytes());
        let m = read_magic_skill_use(&full).unwrap();
        assert_eq!(m.extra, vec![5, 6]);
        assert_eq!(m.positions, vec![[10, 20, 30]]);
        assert!(read_magic_skill_use(&body[..body.len() - 1]).is_err());
        full.push(0);
        assert!(read_magic_skill_use(&full).is_ok());
    }

    #[test]
    fn magic_skill_launched_reads_the_target_list() {
        let m = read_magic_skill_launched(&ds(OP_MAGIC_SKILL_LAUNCHED, &[7, 1204, 1, 2, 9, 10])).unwrap();
        assert_eq!(m, MagicSkillLaunched { caster_id: 7, skill_id: 1204, skill_level: 1, targets: vec![9, 10] });
        let empty = read_magic_skill_launched(&ds(OP_MAGIC_SKILL_LAUNCHED, &[7, 1204, 1, 0])).unwrap();
        assert!(empty.targets.is_empty());
        assert!(read_magic_skill_launched(&ds(OP_MAGIC_SKILL_LAUNCHED, &[7, 1204, 1, 2, 9])).is_err());
        assert!(read_magic_skill_launched(&ds(OP_MAGIC_SKILL_LAUNCHED, &[7, 1204, 1, -1])).is_err());
        assert!(read_magic_skill_launched(&ds(OP_MAGIC_SKILL_LAUNCHED, &[7, 1204, 1, 1, 9, 10])).is_ok());
    }

    #[test]
    fn magic_skill_canceled_reads_the_caster() {
        assert_eq!(read_magic_skill_canceled(&ds(OP_MAGIC_SKILL_CANCELED, &[7])).unwrap().object_id, 7);
        assert!(read_magic_skill_canceled(&[OP_MAGIC_SKILL_CANCELED, 1, 2]).is_err());
        assert!(read_magic_skill_canceled(&ds(OP_MAGIC_SKILL_CANCELED, &[7, 0])).is_ok());
    }

    #[test]
    fn spawn_item_reads_item_position_stackable_count_and_heading() {
        let s = read_spawn_item(&ds(OP_SPAWN_ITEM, &[7, 57, 1, 2, 3, 1, 250, 1])).unwrap();
        assert_eq!(s, SpawnItem { object_id: 7, item_id: 57, x: 1, y: 2, z: 3, stackable: true, count: 250, heading: 1 });
        assert!(!read_spawn_item(&ds(OP_SPAWN_ITEM, &[7, 57, 1, 2, 3, 0, 1, 0])).unwrap().stackable);
        assert!(read_spawn_item(&ds(OP_SPAWN_ITEM, &[7, 57, 1, 2, 3, 1, 250])).is_err());
        assert!(read_spawn_item(&ds(OP_SPAWN_ITEM, &[7, 57, 1, 2, 3, 1, 250, 0, 0])).is_ok());
    }

    #[test]
    fn etc_status_update_reads_five_values() {
        let e = read_etc_status_update(&ds(OP_ETC_STATUS_UPDATE, &[1, 2, 3, 4, 5])).unwrap();
        assert_eq!(e.values, [1, 2, 3, 4, 5]);
        assert!(read_etc_status_update(&ds(OP_ETC_STATUS_UPDATE, &[1, 2, 3, 4])).is_err());
        assert!(read_etc_status_update(&ds(OP_ETC_STATUS_UPDATE, &[1, 2, 3, 4, 5, 6])).is_ok());
    }

    #[test]
    fn magic_effect_icons_reads_a_short_count_of_buffs() {
        let mut w = Writer::new();
        w.write_c(OP_MAGIC_EFFECT_ICONS);
        w.write_h(2);
        w.write_d(1204);
        w.write_h(2);
        w.write_d(1200);
        w.write_d(1040);
        w.write_h(3);
        w.write_d(-1);
        let body = w.into_bytes();
        let e = read_magic_effect_icons(&body).unwrap();
        assert_eq!(
            e.effects,
            vec![Buff { skill_id: 1204, level: 2, duration: 1200 }, Buff { skill_id: 1040, level: 3, duration: -1 }]
        );
        assert!(read_magic_effect_icons(&body[..body.len() - 2]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_MAGIC_EFFECT_ICONS);
        w.write_h(0);
        assert!(read_magic_effect_icons(&w.into_bytes()).unwrap().effects.is_empty());
        assert!(read_magic_effect_icons(&[OP_MAGIC_EFFECT_ICONS, 1]).is_err());
    }

    #[test]
    fn pledge_status_changed_is_six_ints_as_measured_by_l2g() {
        let raw = [
            0xcd, 0x8a, 0xd9, 0xf0, 0x48, 0x4b, 0xc6, 0x00, 0x00, 0x68, 0x89, 0x03, 0x00, 0x1e, 0x1f, 0x00, 0x00, 0x66,
            0x89, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let p = read_pledge_status_changed(&raw).unwrap();
        assert_eq!(p.leader_id, 1223743882);
        assert_eq!(p.clan_id, 50763);
        assert_eq!(p.ally_id, 7966);
        assert_eq!((p.crest_id, p.ally_crest_id), (231784, 231782));
        assert!(read_pledge_status_changed(&raw[..raw.len() - 4]).is_err());
        assert!(read_pledge_status_changed(&ds(OP_PLEDGE_STATUS_CHANGED, &[1, 2, 3, 4, 5, 6, 7])).is_ok());
    }

    fn macro_list_capture() -> Vec<u8> {
        let mut w = Writer::new();
        w.write_c(OP_SEND_MACRO_LIST);
        w.write_d(1);
        w.write_c(0);
        w.write_c(0x13);
        w.write_c(1);
        w.write_d(0x07a5fc);
        w.write_s("AIIOX 1");
        w.write_s("");
        w.write_s("AI1");
        w.write_c(4);
        w.write_c(0);
        w.into_bytes()
    }

    #[test]
    fn send_macro_list_reads_the_l2g_capture_with_one_macro() {
        let m = read_send_macro_list(&macro_list_capture()).unwrap();
        assert_eq!((m.flag, m.revision, m.total_count, m.macros.len()), (1, 0, 19, 1));
        let mac = &m.macros[0];
        assert_eq!(mac.id, 0x07a5fc);
        assert_eq!(mac.name, "AIIOX 1");
        assert_eq!(mac.description, "");
        assert_eq!(mac.acronym, "AI1");
        assert_eq!(mac.icon, 4);
        assert!(mac.commands.is_empty());
    }

    #[test]
    fn send_macro_list_reads_commands_and_the_absent_form() {
        let mut w = Writer::new();
        w.write_c(OP_SEND_MACRO_LIST);
        w.write_d(2);
        w.write_c(0);
        w.write_c(1);
        w.write_c(1);
        w.write_d(1000);
        w.write_s("Buff");
        w.write_s("desc");
        w.write_s("BF");
        w.write_c(1);
        w.write_c(2);
        w.write_c(1);
        w.write_c(1);
        w.write_d(1204);
        w.write_c(0);
        w.write_s("/useskill Wind Walk");
        w.write_c(2);
        w.write_c(3);
        w.write_d(0);
        w.write_c(5);
        w.write_s("/sit");
        let body = w.into_bytes();
        let m = read_send_macro_list(&body).unwrap();
        assert_eq!((m.flag, m.revision, m.total_count), (2, 0, 1));
        let mac = &m.macros[0];
        assert_eq!(mac.commands.len(), 2);
        assert_eq!(
            mac.commands[0],
            MacroCommand { index: 1, kind: 1, skill_id: 1204, shortcut_id: 0, command: "/useskill Wind Walk".into() }
        );
        assert_eq!(
            mac.commands[1],
            MacroCommand { index: 2, kind: 3, skill_id: 0, shortcut_id: 5, command: "/sit".into() }
        );
        assert!(read_send_macro_list(&body[..body.len() - 2]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_SEND_MACRO_LIST);
        w.write_d(3);
        w.write_c(0);
        w.write_c(0);
        w.write_c(0);
        let absent = read_send_macro_list(&w.into_bytes()).unwrap();
        assert_eq!(absent, SendMacroList { flag: 3, revision: 0, total_count: 0, macros: Vec::new() });
        assert!(read_send_macro_list(&[OP_SEND_MACRO_LIST, 3, 0, 0, 0, 0, 0]).is_err());
        assert!(read_send_macro_list(&[OP_SEND_MACRO_LIST, 3, 0, 0, 0, 0, 0, 0, 1]).is_ok());
    }

    #[test]
    fn send_macro_list_reads_the_engine_header_byte_int_count_count_and_several_macros() {
        let mut w = Writer::new();
        w.write_c(OP_SEND_MACRO_LIST);
        w.write_c(9);
        w.write_d(77);
        w.write_c(2);
        w.write_c(2);
        for id in [1, 2] {
            w.write_d(id);
            w.write_s("m");
            w.write_s("");
            w.write_s("M");
            w.write_c(3);
            w.write_c(0);
        }
        let m = read_send_macro_list(&w.into_bytes()).unwrap();
        assert_eq!((m.flag, m.revision, m.total_count), (9, 77, 2));
        assert_eq!(m.macros.iter().map(|m| m.id).collect::<Vec<_>>(), vec![1, 2]);
        let mut w = Writer::new();
        w.write_c(OP_SEND_MACRO_LIST);
        w.write_c(0);
        w.write_d(1);
        w.write_c(1);
        w.write_c(1);
        w.write_d(5);
        w.write_s(&"n".repeat(MACRO_NAME_MAX_CHARS + 1));
        w.write_s("");
        w.write_s("");
        w.write_c(0);
        w.write_c(0);
        assert_eq!(read_send_macro_list(&w.into_bytes()), Err(WireError::TooLong));
    }

    #[test]
    fn every_live_decoder_rejects_an_empty_body() {
        assert!(read_target_selected(&[]).is_err());
        assert!(read_target_unselected(&[]).is_err());
        assert!(read_move_to_pawn(&[]).is_err());
        assert!(read_relation_changed(&[]).is_err());
        assert!(read_magic_skill_use(&[]).is_err());
        assert!(read_magic_skill_launched(&[]).is_err());
        assert!(read_magic_skill_canceled(&[]).is_err());
        assert!(read_spawn_item(&[]).is_err());
        assert!(read_etc_status_update(&[]).is_err());
        assert!(read_magic_effect_icons(&[]).is_err());
        assert!(read_pledge_status_changed(&[]).is_err());
        assert!(read_send_macro_list(&[]).is_err());
    }
}

#[cfg(test)]
mod inventory_tests {
    use super::*;

    fn item(w: &mut Writer, object_id: i32, item_id: i32, count: i32, equipped: bool, body_part: i32) {
        w.write_h(0);
        w.write_d(object_id);
        w.write_d(item_id);
        w.write_d(count);
        w.write_h(5);
        w.write_h(0);
        w.write_h(equipped as u16);
        w.write_d(body_part);
        w.write_h(3);
        w.write_h(0);
    }

    #[test]
    fn item_list_reads_show_window_and_every_item_field() {
        let mut w = Writer::new();
        w.write_c(OP_ITEM_LIST);
        w.write_h(1);
        w.write_h(2);
        item(&mut w, 1001, 57, 2500, false, 0);
        item(&mut w, 1002, 2369, 1, true, BODY_PART_R_HAND);
        let list = read_item_list(&w.into_bytes()).unwrap();
        assert!(list.show_window);
        assert_eq!(list.items.len(), 2);
        assert_eq!(
            list.items[0],
            InventoryItem {
                type1: 0,
                object_id: 1001,
                item_id: 57,
                count: 2500,
                type2: 5,
                custom_type1: 0,
                equipped: false,
                body_part: 0,
                enchant: 3,
                custom_type2: 0,
            }
        );
        assert!(list.items[1].equipped);
        assert_eq!(list.items[1].body_part, BODY_PART_R_HAND);
    }

    #[test]
    fn item_list_hidden_empty_truncated_and_trailing() {
        assert_eq!(
            read_item_list(&[OP_ITEM_LIST, 0, 0, 0, 0]).unwrap(),
            ItemList { show_window: false, items: Vec::new() }
        );
        let mut w = Writer::new();
        w.write_c(OP_ITEM_LIST);
        w.write_h(0);
        w.write_h(1);
        item(&mut w, 1, 2, 3, false, 0);
        let body = w.into_bytes();
        assert!(read_item_list(&body[..body.len() - 1]).is_err());
        let mut trailing = body.clone();
        trailing.push(0);
        assert_eq!(read_item_list(&trailing).unwrap().items.len(), 1);
        assert!(read_item_list(&[OP_ITEM_LIST, 0, 0, 9, 0]).is_err());
    }

    #[test]
    fn inventory_update_reads_change_kind_per_item() {
        let mut w = Writer::new();
        w.write_c(OP_INVENTORY_UPDATE);
        w.write_h(3);
        w.write_h(ITEM_ADDED);
        item(&mut w, 11, 57, 10, false, 0);
        w.write_h(ITEM_MODIFIED);
        item(&mut w, 12, 1060, 4, false, 0);
        w.write_h(ITEM_REMOVED);
        item(&mut w, 13, 2369, 1, false, BODY_PART_R_HAND);
        let update = read_inventory_update(&w.into_bytes()).unwrap();
        assert_eq!(update.changes.len(), 3);
        assert_eq!((update.changes[0].change, update.changes[0].item.object_id), (ITEM_ADDED, 11));
        assert_eq!((update.changes[1].change, update.changes[1].item.count), (ITEM_MODIFIED, 4));
        assert_eq!((update.changes[2].change, update.changes[2].item.item_id), (ITEM_REMOVED, 2369));
        assert!(read_inventory_update(&[OP_INVENTORY_UPDATE, 1, 0, 1, 0]).is_err());
        assert!(read_inventory_update(&[OP_INVENTORY_UPDATE, 0, 0, 0]).is_ok());
    }

    #[test]
    fn equip_update_is_three_ints() {
        let mut w = Writer::new();
        w.write_c(OP_EQUIP_UPDATE);
        w.write_d(1);
        w.write_d(1002);
        w.write_d(7);
        assert_eq!(
            read_equip_update(&w.into_bytes()).unwrap(),
            EquipUpdate { change: 1, object_id: 1002, body_part: 7 }
        );
        assert!(read_equip_update(&[OP_EQUIP_UPDATE, 1, 0, 0, 0]).is_err());
    }

    #[test]
    fn short_cut_init_reads_every_kind_with_its_body_and_the_trailing_int() {
        let mut w = Writer::new();
        w.write_c(OP_SHORT_CUT_INIT);
        w.write_d(5);
        w.write_d(SHORTCUT_ITEM);
        w.write_d(0);
        w.write_d(1001);
        w.write_d(1);
        w.write_d(SHORTCUT_SKILL);
        w.write_d(13);
        w.write_d(1177);
        w.write_d(3);
        w.write_d(1);
        w.write_d(SHORTCUT_ACTION);
        w.write_d(2);
        w.write_d(2);
        w.write_d(1);
        w.write_d(SHORTCUT_MACRO);
        w.write_d(3);
        w.write_d(1000);
        w.write_d(1);
        w.write_d(SHORTCUT_RECIPE);
        w.write_d(4);
        w.write_d(20);
        w.write_d(1);
        let init = read_short_cut_init(&w.into_bytes()).unwrap();
        assert_eq!(
            init.shortcuts,
            vec![
                Shortcut { slot: 0, kind: ShortcutKind::Item { object_id: 1001 }, character_type: 1 },
                Shortcut { slot: 13, kind: ShortcutKind::Skill { skill_id: 1177, level: 3 }, character_type: 1 },
                Shortcut { slot: 2, kind: ShortcutKind::Action { id: 2 }, character_type: 1 },
                Shortcut { slot: 3, kind: ShortcutKind::Macro { id: 1000 }, character_type: 1 },
                Shortcut { slot: 4, kind: ShortcutKind::Recipe { id: 20 }, character_type: 1 },
            ]
        );
        assert_eq!(init.shortcuts[1].page(), 1);
        assert_eq!(init.shortcuts[1].slot_in_page(), 1);
    }

    #[test]
    fn short_cut_init_skips_the_body_of_unknown_kinds_like_the_engine_and_rejects_truncation_and_trailing() {
        let mut w = Writer::new();
        w.write_c(OP_SHORT_CUT_INIT);
        w.write_d(1);
        w.write_d(9);
        w.write_d(0);
        w.write_d(1);
        assert_eq!(
            read_short_cut_init(&w.into_bytes()).unwrap().shortcuts,
            vec![Shortcut { slot: 0, kind: ShortcutKind::Other { kind: 9 }, character_type: 1 }]
        );
        let mut w = Writer::new();
        w.write_c(OP_SHORT_CUT_INIT);
        w.write_d(1);
        w.write_d(SHORTCUT_SKILL);
        w.write_d(0);
        w.write_d(1177);
        assert!(read_short_cut_init(&w.into_bytes()).is_err());
        let mut w = Writer::new();
        w.write_c(OP_SHORT_CUT_INIT);
        w.write_d(0);
        w.write_d(0);
        assert_eq!(read_short_cut_init(&w.into_bytes()).unwrap().shortcuts, Vec::new());
        assert_eq!(read_short_cut_init(&[OP_SHORT_CUT_INIT, 0, 0, 0, 0]).unwrap().shortcuts, Vec::new());
    }

    #[test]
    fn short_cut_register_is_one_entry_and_delete_tolerates_the_lisvus_extra_int() {
        let mut w = Writer::new();
        w.write_c(OP_SHORT_CUT_REGISTER);
        w.write_d(SHORTCUT_ITEM);
        w.write_d(7);
        w.write_d(1001);
        w.write_d(1);
        assert_eq!(
            read_short_cut_register(&w.into_bytes()).unwrap(),
            Shortcut { slot: 7, kind: ShortcutKind::Item { object_id: 1001 }, character_type: 1 }
        );
        let mut w = Writer::new();
        w.write_c(OP_SHORT_CUT_DELETE);
        w.write_d(7);
        assert_eq!(read_short_cut_delete(&w.into_bytes()).unwrap(), ShortCutDelete { slot: 7 });
        let mut w = Writer::new();
        w.write_c(OP_SHORT_CUT_DELETE);
        w.write_d(7);
        w.write_d(0);
        assert_eq!(read_short_cut_delete(&w.into_bytes()).unwrap(), ShortCutDelete { slot: 7 });
        assert!(read_short_cut_delete(&[OP_SHORT_CUT_DELETE, 7, 0]).is_err());
        assert!(read_short_cut_delete(&[OP_SHORT_CUT_DELETE, 7, 0, 0, 0, 0]).is_ok());
    }

    #[test]
    fn skill_list_reads_passive_level_id_in_the_engine_order() {
        let mut w = Writer::new();
        w.write_c(OP_SKILL_LIST);
        w.write_d(2);
        w.write_d(0);
        w.write_d(3);
        w.write_d(1177);
        w.write_d(1);
        w.write_d(1);
        w.write_d(194);
        let list = read_skill_list(&w.into_bytes()).unwrap();
        assert_eq!(
            list.skills,
            vec![SkillEntry { passive: false, level: 3, id: 1177 }, SkillEntry { passive: true, level: 1, id: 194 }]
        );
        assert!(read_skill_list(&[OP_SKILL_LIST, 1, 0, 0, 0, 0]).is_err());
        assert!(read_skill_list(&[OP_SKILL_LIST, 0, 0, 0, 0, 0]).is_ok());
    }

    #[test]
    fn npc_html_message_reads_npc_html_and_item() {
        let mut w = Writer::new();
        w.write_c(OP_NPC_HTML_MESSAGE);
        w.write_d(268_500_000);
        w.write_s("<html><body>Oi</body></html>");
        w.write_d(0);
        let msg = read_npc_html_message(&w.into_bytes()).unwrap();
        assert_eq!(msg.npc_object_id, 268_500_000);
        assert_eq!(msg.html, "<html><body>Oi</body></html>");
        assert_eq!(msg.item_id, 0);
        let mut w = Writer::new();
        w.write_c(OP_NPC_HTML_MESSAGE);
        w.write_d(1);
        w.write_s("abc");
        assert!(read_npc_html_message(&w.into_bytes()).is_err());
    }

    #[test]
    fn npc_html_message_is_bounded_by_the_engine_buffer() {
        let fits = "a".repeat(NPC_HTML_MAX_CHARS);
        let mut w = Writer::new();
        w.write_c(OP_NPC_HTML_MESSAGE);
        w.write_d(1);
        w.write_s(&fits);
        w.write_d(0);
        assert_eq!(read_npc_html_message(&w.into_bytes()).unwrap().html.len(), NPC_HTML_MAX_CHARS);
        let too_long = "a".repeat(16_385);
        let mut w = Writer::new();
        w.write_c(OP_NPC_HTML_MESSAGE);
        w.write_d(1);
        w.write_s(&too_long);
        w.write_d(0);
        assert!(matches!(read_npc_html_message(&w.into_bytes()), Err(WireError::TooLong)));
    }

    #[test]
    fn my_target_selected_skill_cool_time_and_action_failed() {
        let mut w = Writer::new();
        w.write_c(OP_MY_TARGET_SELECTED);
        w.write_d(77);
        w.write_h((-3i16) as u16);
        assert_eq!(read_my_target_selected(&w.into_bytes()).unwrap(), MyTargetSelected { object_id: 77, color: -3 });
        assert!(read_my_target_selected(&[OP_MY_TARGET_SELECTED, 1, 0, 0, 0]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_SKILL_COOL_TIME);
        w.write_d(1);
        w.write_d(1177);
        w.write_d(3);
        w.write_d(20);
        w.write_d(15);
        assert_eq!(
            read_skill_cool_time(&w.into_bytes()).unwrap().entries,
            vec![SkillReuse { skill_id: 1177, level: 3, reuse: 20, remaining: 15 }]
        );
        assert!(read_skill_cool_time(&[OP_SKILL_COOL_TIME, 1, 0, 0, 0]).is_err());
        assert!(read_action_failed(&[OP_ACTION_FAILED]).is_ok());
        assert!(read_action_failed(&[OP_ACTION_FAILED, 0]).is_ok());
        assert!(read_action_failed(&[]).is_err());
    }

    #[test]
    fn user_info_reads_attributes_load_paperdoll_and_inventory_limit() {
        let mut w = Writer::new();
        w.write_c(OP_USER_INFO);
        for v in [1, 2, 3, 4, 5] {
            w.write_d(v);
        }
        w.write_s("W");
        w.write_d(0);
        w.write_d(0);
        w.write_d(0);
        w.write_d(1);
        w.write_d(0);
        for v in [40, 30, 43, 21, 11, 25] {
            w.write_d(v);
        }
        w.write_d(900);
        w.write_d(400);
        w.write_d(300);
        w.write_d(200);
        w.write_d(777);
        w.write_d(1500);
        w.write_d(62_000);
        w.write_d(40);
        for i in 0..16 {
            w.write_d(5000 + i);
        }
        for i in 0..16 {
            w.write_d(100 + i);
        }
        for v in [150, 300, 120, 30, 33, 44, 60, 213, 0, 80] {
            w.write_d(v);
        }
        w.write_d(0);
        w.write_d(0);
        for _ in 0..8 {
            w.write_d(0);
        }
        w.write_f(1.0);
        w.write_f(1.0);
        w.write_f(9.0);
        w.write_f(23.0);
        w.write_d(0);
        w.write_d(0);
        w.write_d(0);
        w.write_d(0);
        w.write_s("T");
        for _ in 0..5 {
            w.write_d(0);
        }
        w.write_c(0);
        w.write_c(0);
        w.write_c(0);
        w.write_d(0);
        w.write_d(0);
        w.write_h(0);
        w.write_c(0);
        w.write_d(0);
        w.write_c(0);
        for _ in 0..8 {
            w.write_d(0);
        }
        w.write_h(0);
        w.write_h(0);
        w.write_d(0);
        w.write_h(80);
        w.write_d(0);
        w.write_d(0);
        w.write_d(500);
        w.write_d(450);
        w.write_c(0);
        w.write_c(0);
        w.write_d(0);
        w.write_c(0);
        w.write_c(0);
        w.write_c(0);
        w.write_d(0);
        w.write_d(0);
        w.write_d(0);
        w.write_d(0xffffff);
        let head = read_user_info_head(&w.into_bytes()).unwrap();
        let s = head.stats;
        assert_eq!((s.str_, s.dex, s.con, s.int_, s.wit, s.men), (40, 30, 43, 21, 11, 25));
        assert_eq!((s.sp, s.load, s.max_load, s.attack_range), (777, 1500, 62_000, 40));
        assert_eq!((s.p_atk, s.atk_spd, s.p_def, s.evasion, s.accuracy, s.critical), (150, 300, 120, 30, 33, 44));
        assert_eq!((s.m_atk, s.cast_spd, s.m_def), (60, 213, 80));
        assert_eq!(head.inventory_limit, 80);
        assert_eq!(head.paperdoll[PAPERDOLL_UNDER], PaperdollSlot { object_id: 5000, item_id: 100 });
        assert_eq!(head.paperdoll[PAPERDOLL_R_HAND], PaperdollSlot { object_id: 5007, item_id: 107 });
        assert_eq!(head.paperdoll[PAPERDOLL_HAIR], PaperdollSlot { object_id: 5015, item_id: 115 });
        assert_eq!((head.max_cp, head.cp), (500, 450));
        assert_eq!(head.name_color, 0xffffff);
    }

    #[test]
    fn status_update_attribute_ids_follow_the_engine_switch() {
        assert_eq!(
            (ATTRIBUTE_STR, ATTRIBUTE_DEX, ATTRIBUTE_CON, ATTRIBUTE_INT, ATTRIBUTE_WIT, ATTRIBUTE_MEN),
            (3, 4, 5, 6, 7, 8)
        );
        assert_eq!(
            (ATTRIBUTE_SP, ATTRIBUTE_CUR_LOAD, ATTRIBUTE_MAX_LOAD, ATTRIBUTE_ATTACK_RANGE),
            (0x0d, 0x0e, 0x0f, 0x10)
        );
        assert_eq!(
            (ATTRIBUTE_P_ATK, ATTRIBUTE_P_ATK_SPD, ATTRIBUTE_P_DEF, ATTRIBUTE_EVASION),
            (0x11, 0x12, 0x13, 0x14)
        );
        assert_eq!(
            (ATTRIBUTE_ACCURACY, ATTRIBUTE_CRITICAL, ATTRIBUTE_M_ATK, ATTRIBUTE_CAST_SPD, ATTRIBUTE_M_DEF),
            (0x15, 0x16, 0x17, 0x18, 0x19)
        );
        assert_eq!(ATTRIBUTE_MAX_CP, 0x22);
        let mut s = UserStats::default();
        assert!(s.apply(ATTRIBUTE_STR, 41));
        assert!(s.apply(ATTRIBUTE_P_ATK, 151));
        assert!(s.apply(ATTRIBUTE_CAST_SPD, 333));
        assert!(s.apply(ATTRIBUTE_CUR_LOAD, 1600));
        assert!(s.apply(ATTRIBUTE_SP, 1));
        assert!(!s.apply(ATTRIBUTE_CURRENT_HP, 1));
        assert!(!s.apply(0x1c, 1));
        assert_eq!((s.str_, s.p_atk, s.cast_spd, s.load, s.sp), (41, 151, 333, 1600, 1));
    }

    #[test]
    fn body_part_masks_and_paperdoll_indices() {
        assert_eq!(BODY_PART_UNDERWEAR, 0x0001);
        assert_eq!(BODY_PART_R_EAR, 0x0002);
        assert_eq!(BODY_PART_L_EAR, 0x0004);
        assert_eq!(BODY_PART_NECK, 0x0008);
        assert_eq!(BODY_PART_R_FINGER, 0x0010);
        assert_eq!(BODY_PART_L_FINGER, 0x0020);
        assert_eq!(BODY_PART_HEAD, 0x0040);
        assert_eq!(BODY_PART_R_HAND, 0x0080);
        assert_eq!(BODY_PART_L_HAND, 0x0100);
        assert_eq!(BODY_PART_GLOVES, 0x0200);
        assert_eq!(BODY_PART_CHEST, 0x0400);
        assert_eq!(BODY_PART_LEGS, 0x0800);
        assert_eq!(BODY_PART_FEET, 0x1000);
        assert_eq!(BODY_PART_BACK, 0x2000);
        assert_eq!(BODY_PART_LR_HAND, 0x4000);
        assert_eq!(BODY_PART_FULL_ARMOR, 0x8000);
        assert_eq!(BODY_PART_HAIR, 0x10000);
        assert_eq!(PAPERDOLL_SLOTS, 16);
        assert_eq!(
            [
                PAPERDOLL_UNDER,
                PAPERDOLL_R_EAR,
                PAPERDOLL_L_EAR,
                PAPERDOLL_NECK,
                PAPERDOLL_R_FINGER,
                PAPERDOLL_L_FINGER,
                PAPERDOLL_HEAD,
                PAPERDOLL_R_HAND
            ],
            [0, 1, 2, 3, 4, 5, 6, 7]
        );
        assert_eq!(
            [
                PAPERDOLL_L_HAND,
                PAPERDOLL_GLOVES,
                PAPERDOLL_CHEST,
                PAPERDOLL_LEGS,
                PAPERDOLL_FEET,
                PAPERDOLL_BACK,
                PAPERDOLL_LR_HAND,
                PAPERDOLL_HAIR
            ],
            [8, 9, 10, 11, 12, 13, 14, 15]
        );
    }

    #[test]
    fn client_encoders_follow_the_engine_formats() {
        assert_eq!(request_item_list(), [0x0f]);
        assert_eq!(use_item(1001, false), [0x14, 0xe9, 3, 0, 0, 0, 0, 0, 0]);
        assert_eq!(use_item(1001, true), [0x14, 0xe9, 3, 0, 0, 1, 0, 0, 0]);
        assert_eq!(request_unequip_item(BODY_PART_R_HAND), [0x11, 0x80, 0, 0, 0]);
        let mut expect = vec![0x21u8];
        for unit in "npc_1234_Chat 1".encode_utf16() {
            expect.extend_from_slice(&unit.to_le_bytes());
        }
        expect.extend_from_slice(&[0, 0]);
        assert_eq!(request_bypass_to_server("npc_1234_Chat 1"), expect);
        assert_eq!(request_link_html("x"), [0x20, b'x', 0, 0, 0]);
        assert_eq!(request_magic_skill_use(1177, false, true), [0x2f, 0x99, 4, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(request_magic_skill_use(1177, true, false), [0x2f, 0x99, 4, 0, 0, 1, 0, 0, 0, 0]);
        assert_eq!(request_action_use(2, false, false), [0x45, 2, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            request_shortcut_reg(SHORTCUT_SKILL, 13, 1177, 3),
            [0x33, 2, 0, 0, 0, 13, 0, 0, 0, 0x99, 4, 0, 0, 3, 0, 0, 0]
        );
        assert_eq!(request_shortcut_del(13), [0x35, 13, 0, 0, 0]);
        assert_eq!(request_shortcut_use(13, 2, 1), [0x34, 13, 0, 0, 0, 2, 0, 0, 0, 1]);
        assert_eq!(request_dismiss_party(), [0x2d]);
        assert_eq!(request_gm_list(), [0x81]);
        assert_eq!(revive_reply(1), [0x7a, 1, 0, 0, 0]);
        assert_eq!(send_time_check(1, 2), [0x97, 1, 0, 0, 0, 2, 0, 0, 0]);
        assert_eq!(request_remain_time(), [0xa9]);
        assert_eq!(net_ping(1, 2, 3), [0xa8, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0]);
        assert_eq!(request_destroy_item(1001, 5), [0x59, 0xe9, 3, 0, 0, 5, 0, 0, 0]);
        assert_eq!(request_target_cancel(), [0x37, 0, 0]);
        assert_eq!(request_target_cancel_with(1), [0x37, 1, 0]);
        assert_eq!(request_skill_cool_time(), [0x9d]);
    }
}
