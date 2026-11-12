use super::counted;
use crate::wire::{Reader, Result};

pub const OP_EX_PACKET: u8 = 0xfe;
pub const EX_STORAGE_MAX_COUNT: u16 = 0x002e;
pub const EX_RECEIVE_AUTO_SOUL_SHOT: u16 = 0x0012;
pub const EX_FISHING_END: u16 = 0x0014;
pub const EX_FISHING_START_COMBAT: u16 = 0x0015;
pub const EX_FISHING_HP_REGEN: u16 = 0x0016;
pub const EX_ENCHANT_SKILL_LIST: u16 = 0x0017;
pub const EX_RECEIVE_ENCHANT_SKILL_INFO: u16 = 0x0018;
pub const EX_SHOW_QUEST_INFO: u16 = 0x0019;
pub const EX_SHOW_QUEST_MARK: u16 = 0x001a;
pub const EX_SEND_MANOR_LIST: u16 = 0x001b;
pub const EX_SHOW_SEED_INFO: u16 = 0x001c;
pub const EX_SHOW_CROP_INFO: u16 = 0x001d;
pub const EX_SHOW_MANOR_DEFAULT_INFO: u16 = 0x001e;
pub const EX_SHOW_SEED_SETTING: u16 = 0x001f;
pub const EX_SHOW_CROP_SETTING: u16 = 0x0020;
pub const EX_SHOW_SELL_CROP_LIST: u16 = 0x0021;
pub const EX_SHOW_PROCURE_CROP_DETAIL: u16 = 0x0022;
pub const EX_HERO_LIST: u16 = 0x0023;
pub const EX_SERVER_PRIMITIVE: u16 = 0x0024;
pub const EX_RECEIVE_ASK_JOIN_MPCC: u16 = 0x0027;
pub const EX_RECEIVE_PLEDGE_CREST_LARGE: u16 = 0x0028;
pub const EX_RECEIVE_OLYMPIAD_USER_INFO: u16 = 0x0029;
pub const EX_RECEIVE_OLYMPIAD_SPELLED_INFO: u16 = 0x002a;
pub const EX_RECEIVE_OLYMPIAD_MODE: u16 = 0x002b;
pub const EX_EVENT_MATCH_MANAGE: u16 = 0x002f;
pub const EX_MULTI_PARTY_COMMAND_CHANNEL_INFO: u16 = 0x0030;
pub const EX_REGEN_MAX: u16 = 0x0001;
pub const EX_EVENT_MATCH_OBSERVER: u16 = 0x0003;
pub const EX_EVENT_MATCH_MESSAGE: u16 = 0x0004;
pub const EX_EVENT_MATCH_SCORE: u16 = 0x0005;
pub const EX_EVENT_MATCH_TEAM_INFO: u16 = 0x0006;
pub const EX_EVENT_MATCH_CREATE: u16 = 0x0007;
pub const EX_EVENT_MATCH_USER_INFO: u16 = 0x0008;
pub const EX_COLOSSEUM_FENCE_INFO: u16 = 0x0009;
pub const EX_EVENT_MATCH_SPELLED_INFO: u16 = 0x000a;
pub const EX_EVENT_MATCH_FIRECRACKER: u16 = 0x000b;
pub const EX_EVENT_MATCH_TEAM_UNLOCKED: u16 = 0x000c;
pub const EX_EVENT_MATCH_GM_TEST: u16 = 0x000d;
pub const EX_CLOSE_PARTY_ROOM: u16 = 0x000f;
pub const EX_FISHING_START: u16 = 0x0013;
pub const EX_OPEN_MPCC: u16 = 0x0025;
pub const EX_CLOSE_MPCC: u16 = 0x0026;
pub const EX_OLYMPIAD_MATCH_END: u16 = 0x002c;
pub const EX_MAIL_ARRIVED: u16 = 0x002d;
pub const EX_PARTY_ROOM_MEMBER: u16 = 0x000e;
pub const EX_MANAGE_PARTY_ROOM_MEMBER: u16 = 0x0010;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantSkillList {
    pub entries: Vec<[i32; 4]>,
}

pub fn read_ex_enchant_skill_list(body: &[u8]) -> Result<EnchantSkillList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 16)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push([r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?]);
    }
    Ok(EnchantSkillList { entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantSkillInfo {
    pub header: [i32; 5],
    pub entries: Vec<[i32; 4]>,
}

pub fn read_ex_enchant_skill_info(body: &[u8]) -> Result<EnchantSkillInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let header = [r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?];
    let declared = r.read_d()?;
    let count = counted(&r, declared, 16)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push([r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?]);
    }
    Ok(EnchantSkillInfo { header, entries })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FishingHpRegen {
    pub values: [i32; 3],
    pub flags: [u8; 3],
    pub amount: i32,
}

pub fn read_ex_fishing_hp_regen(body: &[u8]) -> Result<FishingHpRegen> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let values = [r.read_d()?, r.read_d()?, r.read_d()?];
    let flags = [r.read_c()?, r.read_c()?, r.read_c()?];
    let amount = r.read_d()?;
    Ok(FishingHpRegen { values, flags, amount })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FishingStartCombat {
    pub values: [i32; 3],
    pub flags: [u8; 2],
}

pub fn read_ex_fishing_start_combat(body: &[u8]) -> Result<FishingStartCombat> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let values = [r.read_d()?, r.read_d()?, r.read_d()?];
    let flags = [r.read_c()?, r.read_c()?];
    Ok(FishingStartCombat { values, flags })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FishingEnd {
    pub value: i32,
    pub flag: u8,
}

pub fn read_ex_fishing_end(body: &[u8]) -> Result<FishingEnd> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let value = r.read_d()?;
    let flag = r.read_c()?;
    Ok(FishingEnd { value, flag })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoSoulShot {
    pub values: [i32; 2],
}

pub fn read_ex_auto_soul_shot(body: &[u8]) -> Result<AutoSoulShot> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let values = [r.read_d()?, r.read_d()?];
    Ok(AutoSoulShot { values })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageMaxCount {
    pub values: [i32; 7],
}

impl StorageMaxCount {
    pub fn warehouse_limit(&self) -> i32 {
        self.values[1]
    }

    pub fn private_sell_limit(&self) -> i32 {
        self.values[3]
    }

    pub fn private_buy_limit(&self) -> i32 {
        self.values[4]
    }
}

pub fn read_ex_subopcode(body: &[u8]) -> Result<u16> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()
}

pub fn read_ex_show_quest_info(body: &[u8]) -> Result<()> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShowQuestMark {
    pub value: i32,
}

pub fn read_ex_show_quest_mark(body: &[u8]) -> Result<ShowQuestMark> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(ShowQuestMark { value: r.read_d()? })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManorEntry {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManorList {
    pub entries: Vec<ManorEntry>,
}

pub fn read_ex_send_manor_list(body: &[u8]) -> Result<ManorList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 6)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push(ManorEntry { id: r.read_d()?, name: r.read_s()? });
    }
    Ok(ManorList { entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedInfoEntry {
    pub values: [i32; 5],
    pub first: Vec<i32>,
    pub second: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedInfo {
    pub flag: u8,
    pub values: [i32; 2],
    pub entries: Vec<SeedInfoEntry>,
}

pub fn read_ex_show_seed_info(body: &[u8]) -> Result<SeedInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let flag = r.read_c()?;
    let values = [r.read_d()?, r.read_d()?];
    let declared = r.read_d()?;
    let count = counted(&r, declared, 22)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let values = [r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?];
        let first_count = i32::from(r.read_c()?);
        let first_len = counted(&r, first_count, 4)?;
        let mut first = Vec::with_capacity(first_len);
        for _ in 0..first_len {
            first.push(r.read_d()?);
        }
        let second_count = i32::from(r.read_c()?);
        let second_len = counted(&r, second_count, 4)?;
        let mut second = Vec::with_capacity(second_len);
        for _ in 0..second_len {
            second.push(r.read_d()?);
        }
        entries.push(SeedInfoEntry { values, first, second });
    }
    Ok(SeedInfo { flag, values, entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CropInfoEntry {
    pub values: [i32; 4],
    pub flag: u8,
    pub value: i32,
    pub first: Vec<i32>,
    pub second: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CropInfo {
    pub flag: u8,
    pub values: [i32; 2],
    pub entries: Vec<CropInfoEntry>,
}

pub fn read_ex_show_crop_info(body: &[u8]) -> Result<CropInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let flag = r.read_c()?;
    let values = [r.read_d()?, r.read_d()?];
    let declared = r.read_d()?;
    let count = counted(&r, declared, 23)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let values = [r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?];
        let flag = r.read_c()?;
        let value = r.read_d()?;
        let first_count = i32::from(r.read_c()?);
        let first_len = counted(&r, first_count, 4)?;
        let mut first = Vec::with_capacity(first_len);
        for _ in 0..first_len {
            first.push(r.read_d()?);
        }
        let second_count = i32::from(r.read_c()?);
        let second_len = counted(&r, second_count, 4)?;
        let mut second = Vec::with_capacity(second_len);
        for _ in 0..second_len {
            second.push(r.read_d()?);
        }
        entries.push(CropInfoEntry { values, flag, value, first, second });
    }
    Ok(CropInfo { flag, values, entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManorDefaultEntry {
    pub values: [i32; 4],
    pub first: Vec<i32>,
    pub second: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManorDefaultInfo {
    pub flag: u8,
    pub entries: Vec<ManorDefaultEntry>,
}

pub fn read_ex_show_manor_default_info(body: &[u8]) -> Result<ManorDefaultInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let flag = r.read_c()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 22)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let values = [r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?];
        let first_count = i32::from(r.read_c()?);
        let first_len = counted(&r, first_count, 4)?;
        let mut first = Vec::with_capacity(first_len);
        for _ in 0..first_len {
            first.push(r.read_d()?);
        }
        let second_count = i32::from(r.read_c()?);
        let second_len = counted(&r, second_count, 4)?;
        let mut second = Vec::with_capacity(second_len);
        for _ in 0..second_len {
            second.push(r.read_d()?);
        }
        entries.push(ManorDefaultEntry { values, first, second });
    }
    Ok(ManorDefaultInfo { flag, entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedSettingEntry {
    pub values: [i32; 2],
    pub first: Vec<i32>,
    pub second: Vec<i32>,
    pub tail: [i32; 8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedSetting {
    pub value: i32,
    pub entries: Vec<SeedSettingEntry>,
}

fn signed_list(r: &mut Reader) -> Result<Vec<i32>> {
    let declared = i32::from(r.read_c()? as i8).max(0);
    let len = counted(r, declared, 4)?;
    let mut list = Vec::with_capacity(len);
    for _ in 0..len {
        list.push(r.read_d()?);
    }
    Ok(list)
}

pub fn read_ex_show_seed_setting(body: &[u8]) -> Result<SeedSetting> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let value = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 42)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let values = [r.read_d()?, r.read_d()?];
        let first = signed_list(&mut r)?;
        let second = signed_list(&mut r)?;
        let mut tail = [0; 8];
        for item in &mut tail {
            *item = r.read_d()?;
        }
        entries.push(SeedSettingEntry { values, first, second, tail });
    }
    Ok(SeedSetting { value, entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CropSettingInfoEntry {
    pub values: [i32; 2],
    pub first: Vec<i32>,
    pub second: Vec<i32>,
    pub settings: [i32; 6],
    pub flag: u8,
    pub tail: [i32; 2],
    pub last: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CropSettingInfo {
    pub value: i32,
    pub entries: Vec<CropSettingInfoEntry>,
}

pub fn read_ex_show_crop_setting(body: &[u8]) -> Result<CropSettingInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let value = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 46)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let values = [r.read_d()?, r.read_d()?];
        let first = signed_list(&mut r)?;
        let second = signed_list(&mut r)?;
        let settings = [r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?];
        let flag = r.read_c()?;
        let tail = [r.read_d()?, r.read_d()?];
        let last = r.read_c()?;
        entries.push(CropSettingInfoEntry { values, first, second, settings, flag, tail, last });
    }
    Ok(CropSettingInfo { value, entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SellCropEntry {
    pub values: [i32; 3],
    pub first: Vec<i32>,
    pub second: Vec<i32>,
    pub tail: [i32; 3],
    pub flag: u8,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SellCropList {
    pub value: i32,
    pub entries: Vec<SellCropEntry>,
}

pub fn read_ex_show_sell_crop_list(body: &[u8]) -> Result<SellCropList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let header = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 22)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let values = [r.read_d()?, r.read_d()?, r.read_d()?];
        let first = signed_list(&mut r)?;
        let second = signed_list(&mut r)?;
        let tail = [r.read_d()?, r.read_d()?, r.read_d()?];
        let flag = r.read_c()?;
        let value = r.read_d()?;
        entries.push(SellCropEntry { values, first, second, tail, flag, value });
    }
    Ok(SellCropList { value: header, entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcureCropDetailEntry {
    pub values: [i32; 3],
    pub flag: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcureCropDetail {
    pub value: i32,
    pub entries: Vec<ProcureCropDetailEntry>,
}

pub fn read_ex_show_procure_crop_detail(body: &[u8]) -> Result<ProcureCropDetail> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let value = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 13)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let values = [r.read_d()?, r.read_d()?, r.read_d()?];
        let flag = r.read_c()?;
        entries.push(ProcureCropDetailEntry { values, flag });
    }
    Ok(ProcureCropDetail { value, entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroEntry {
    pub names: [String; 3],
    pub values: [i32; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroList {
    pub entries: Vec<HeroEntry>,
}

pub fn read_ex_hero_list(body: &[u8]) -> Result<HeroList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 18)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let name0 = r.read_s()?;
        let value0 = r.read_d()?;
        let name1 = r.read_s()?;
        let value1 = r.read_d()?;
        let name2 = r.read_s()?;
        let values = [value0, value1, r.read_d()?, r.read_d()?];
        let names = [name0, name1, name2];
        entries.push(HeroEntry { names, values });
    }
    Ok(HeroList { entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPrimitiveEntry {
    pub kind: u8,
    pub text: Option<String>,
    pub values: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPrimitive {
    pub name: String,
    pub values: [i32; 6],
    pub entries: Vec<ServerPrimitiveEntry>,
}

pub fn read_ex_server_primitive(body: &[u8]) -> Result<ServerPrimitive> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let name = r.read_s()?;
    let mut values = [0; 6];
    for value in &mut values {
        *value = r.read_d()?;
    }
    let count = counted(&r, values[5], 1)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let kind = r.read_c()?;
        let fields = match kind {
            1 => 7,
            2 => 10,
            _ => 0,
        };
        let text = if fields > 0 { Some(r.read_s()?) } else { None };
        let mut values = Vec::with_capacity(fields);
        for _ in 0..fields {
            values.push(r.read_d()?);
        }
        entries.push(ServerPrimitiveEntry { kind, text, values });
    }
    Ok(ServerPrimitive { name, values, entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskJoinMpcc {
    pub name: String,
}

pub fn read_ex_ask_join_mpcc(body: &[u8]) -> Result<AskJoinMpcc> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(AskJoinMpcc { name: r.read_s()? })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PledgeCrestLarge {
    pub values: [i32; 3],
    pub data: Vec<u8>,
}

pub fn read_ex_pledge_crest_large(body: &[u8]) -> Result<PledgeCrestLarge> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let values = [r.read_d()?, r.read_d()?, r.read_d()?];
    let data = if values[2] > 0 { r.read_b(values[2] as usize)?.to_vec() } else { Vec::new() };
    Ok(PledgeCrestLarge { values, data })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OlympiadUserInfo {
    pub flag: u8,
    pub value: i32,
    pub name: String,
    pub values: [i32; 5],
}

pub fn read_ex_olympiad_user_info(body: &[u8]) -> Result<OlympiadUserInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let flag = r.read_c()?;
    let value = r.read_d()?;
    let name = r.read_s()?;
    let values = [r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?];
    Ok(OlympiadUserInfo { flag, value, name, values })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OlympiadSpelledEntry {
    pub value: i32,
    pub level: u16,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OlympiadSpelledInfo {
    pub values: [i32; 2],
    pub entries: Vec<OlympiadSpelledEntry>,
}

pub fn read_ex_olympiad_spelled_info(body: &[u8]) -> Result<OlympiadSpelledInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let values = [r.read_d()?, r.read_d()?];
    let count = counted(&r, values[1], 10)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push(OlympiadSpelledEntry { value: r.read_d()?, level: r.read_h()?, amount: r.read_d()? });
    }
    Ok(OlympiadSpelledInfo { values, entries })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OlympiadMode {
    pub mode: u8,
}

pub fn read_ex_olympiad_mode(body: &[u8]) -> Result<OlympiadMode> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(OlympiadMode { mode: r.read_c()? })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventMatchManageEntry {
    pub flags: [u8; 2],
    pub value: i32,
    pub name: String,
    pub values: [i32; 2],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventMatchManage {
    pub value: i32,
    pub flags: [u8; 2],
    pub first_name: String,
    pub middle_flag: u8,
    pub second_name: String,
    pub last_flag: u8,
    pub entries: Vec<EventMatchManageEntry>,
}

pub fn read_ex_event_match_manage(body: &[u8]) -> Result<EventMatchManage> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let value = r.read_d()?;
    let flags = [r.read_c()?, r.read_c()?];
    let first_name = r.read_s()?;
    let middle_flag = r.read_c()?;
    let second_name = r.read_s()?;
    let last_flag = r.read_c()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 18)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let flags = [r.read_c()?, r.read_c()?];
        let value = r.read_d()?;
        let name = r.read_s()?;
        let values = [r.read_d()?, r.read_d()?];
        entries.push(EventMatchManageEntry { flags, value, name, values });
    }
    Ok(EventMatchManage { value, flags, first_name, middle_flag, second_name, last_flag, entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiPartyChannelMember {
    pub name: String,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiPartyChannelInfo {
    pub name: String,
    pub values: [i32; 2],
    pub members: Vec<MultiPartyChannelMember>,
}

pub fn read_ex_multi_party_command_channel_info(body: &[u8]) -> Result<MultiPartyChannelInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let name = r.read_s()?;
    let values = [r.read_d()?, r.read_d()?];
    let count = counted(&r, values[1], 6)?;
    let mut members = Vec::with_capacity(count);
    for _ in 0..count {
        members.push(MultiPartyChannelMember { name: r.read_s()?, value: r.read_d()? });
    }
    Ok(MultiPartyChannelInfo { name, values, members })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegenMax {
    pub values: [i32; 3],
    pub amount: f64,
}

pub fn read_ex_regen_max(body: &[u8]) -> Result<RegenMax> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(RegenMax { values: [r.read_d()?, r.read_d()?, r.read_d()?], amount: r.read_f()? })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventMatchObserver {
    pub value: i32,
    pub flags: [u8; 2],
    pub names: [String; 2],
    pub values: [i32; 3],
}

pub fn read_ex_event_match_observer(body: &[u8]) -> Result<EventMatchObserver> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let value = r.read_d()?;
    let flags = [r.read_c()?, r.read_c()?];
    let names = [r.read_s()?, r.read_s()?];
    let values = [r.read_d()?, r.read_d()?, r.read_d()?];
    Ok(EventMatchObserver { value, flags, names, values })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventMatchMessage {
    pub flag: u8,
    pub message: String,
}

pub fn read_ex_event_match_message(body: &[u8]) -> Result<EventMatchMessage> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(EventMatchMessage { flag: r.read_c()?, message: r.read_s()? })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventMatchScore {
    pub values: [i32; 3],
}

pub fn read_ex_event_match_score(body: &[u8]) -> Result<EventMatchScore> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(EventMatchScore { values: [r.read_d()?, r.read_d()?, r.read_d()?] })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventMatchTeamEntry {
    pub id: i32,
    pub name: String,
    pub values: [i32; 10],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventMatchTeamInfo {
    pub header: [i32; 3],
    pub entries: Vec<EventMatchTeamEntry>,
}

pub fn read_ex_event_match_team_info(body: &[u8]) -> Result<EventMatchTeamInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let header = [r.read_d()?, r.read_d()?, r.read_d()?];
    let count = counted(&r, header[2], 16)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let id = r.read_d()?;
        let name = r.read_s()?;
        let mut values = [0; 10];
        for value in &mut values {
            *value = r.read_d()?;
        }
        entries.push(EventMatchTeamEntry { id, name, values });
    }
    Ok(EventMatchTeamInfo { header, entries })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventMatchCreate {
    pub value: i32,
}

pub fn read_ex_event_match_create(body: &[u8]) -> Result<EventMatchCreate> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(EventMatchCreate { value: r.read_d()? })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventMatchUserInfo {
    pub id: i32,
    pub name: String,
    pub values: [i32; 8],
}

pub fn read_ex_event_match_user_info(body: &[u8]) -> Result<EventMatchUserInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let id = r.read_d()?;
    let name = r.read_s()?;
    let mut values = [0; 8];
    for value in &mut values {
        *value = r.read_d()?;
    }
    Ok(EventMatchUserInfo { id, name, values })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColosseumFenceInfo {
    pub values: [i32; 7],
}

pub fn read_ex_colosseum_fence_info(body: &[u8]) -> Result<ColosseumFenceInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let mut values = [0; 7];
    for value in &mut values {
        *value = r.read_d()?;
    }
    Ok(ColosseumFenceInfo { values })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventMatchSpelledEntry {
    pub value: i32,
    pub level: u16,
    pub amount: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventMatchSpelledInfo {
    pub values: [i32; 3],
    pub entries: Vec<EventMatchSpelledEntry>,
}

pub fn read_ex_event_match_spelled_info(body: &[u8]) -> Result<EventMatchSpelledInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let values = [r.read_d()?, r.read_d()?, r.read_d()?];
    let count = counted(&r, values[2], 10)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push(EventMatchSpelledEntry { value: r.read_d()?, level: r.read_h()?, amount: r.read_d()? });
    }
    Ok(EventMatchSpelledInfo { values, entries })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventMatchFirecracker {
    pub value: i32,
}

pub fn read_ex_event_match_firecracker(body: &[u8]) -> Result<EventMatchFirecracker> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(EventMatchFirecracker { value: r.read_d()? })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventMatchTeamUnlocked {
    pub value: i32,
    pub flag: u8,
}

pub fn read_ex_event_match_team_unlocked(body: &[u8]) -> Result<EventMatchTeamUnlocked> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(EventMatchTeamUnlocked { value: r.read_d()?, flag: r.read_c()? })
}

pub fn read_ex_event_match_gm_test(body: &[u8]) -> Result<()> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(())
}

pub fn read_ex_close_party_room(body: &[u8]) -> Result<()> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FishingStart {
    pub values: [i32; 5],
}

pub fn read_ex_fishing_start(body: &[u8]) -> Result<FishingStart> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let values = [r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?];
    Ok(FishingStart { values })
}

pub fn read_ex_open_mpcc(body: &[u8]) -> Result<()> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(())
}

pub fn read_ex_close_mpcc(body: &[u8]) -> Result<()> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(())
}

pub fn read_ex_olympiad_match_end(body: &[u8]) -> Result<()> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(())
}

pub fn read_ex_mail_arrived(body: &[u8]) -> Result<()> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyRoomMemberEntry {
    pub id: i32,
    pub name: String,
    pub values: [i32; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyRoomMember {
    pub header: [i32; 2],
    pub entries: Vec<PartyRoomMemberEntry>,
}

pub fn read_ex_party_room_member(body: &[u8]) -> Result<PartyRoomMember> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let header = [r.read_d()?, r.read_d()?];
    let count = counted(&r, header[1], 25)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let id = r.read_d()?;
        let name = r.read_s()?;
        let values = [r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?];
        entries.push(PartyRoomMemberEntry { id, name, values });
    }
    Ok(PartyRoomMember { header, entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagePartyRoomMember {
    pub header: [i32; 2],
    pub name: String,
    pub values: [i32; 4],
}

pub fn read_ex_manage_party_room_member(body: &[u8]) -> Result<ManagePartyRoomMember> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let header = [r.read_d()?, r.read_d()?];
    let name = r.read_s()?;
    let values = [r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?];
    Ok(ManagePartyRoomMember { header, name, values })
}

pub fn read_ex_storage_max_count(body: &[u8]) -> Result<StorageMaxCount> {
    let mut r = Reader::new(body);
    r.read_c()?;
    r.read_h()?;
    let mut values = [0; 7];
    for value in &mut values {
        *value = r.read_d()?;
    }
    Ok(StorageMaxCount { values })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::{WireError, Writer};

    #[test]
    fn receive_ex_storage_max_count_1040eb40_reads_seven_dwords() {
        for values in [[0; 7], [i32::MIN, -1, 0, 1, 80, 200, i32::MAX]] {
            let mut body = vec![0xfe, 0x2e, 0];
            for value in values {
                body.extend_from_slice(&value.to_le_bytes());
            }
            for len in 0..body.len() {
                assert_eq!(read_ex_storage_max_count(&body[..len]), Err(WireError::Short));
            }
            let expected = StorageMaxCount { values };
            assert_eq!(read_ex_storage_max_count(&body), Ok(expected.clone()));
            assert_eq!(expected.warehouse_limit(), values[1]);
            body.extend_from_slice(&[0xff, 0x80, 0]);
            assert_eq!(read_ex_storage_max_count(&body), Ok(expected));
        }
    }

    #[test]
    fn tick_10410600_reads_full_subopcode() {
        assert_eq!(read_ex_subopcode(&[0xfe, 0x2e, 1]), Ok(0x012e));
        assert_eq!(read_ex_subopcode(&[0xfe, 0xff, 0xff]), Ok(0xffff));
        assert_eq!(read_ex_subopcode(&[0xfe, 0x2e]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_enchant_skill_info_1040c2a0_reads_header_count_and_entries() {
        let header: [i32; 5] = [1, -2, 3, 4, 5];
        let entries: [[i32; 4]; 2] = [[10, 11, 12, 13], [20, 21, 22, 23]];
        let mut body = vec![0xfe, 0x18, 0];
        for value in header {
            body.extend_from_slice(&value.to_le_bytes());
        }
        body.extend_from_slice(&(entries.len() as i32).to_le_bytes());
        for entry in entries {
            for value in entry {
                body.extend_from_slice(&value.to_le_bytes());
            }
        }
        for len in 0..body.len() {
            assert_eq!(read_ex_enchant_skill_info(&body[..len]), Err(WireError::Short));
        }
        assert_eq!(read_ex_enchant_skill_info(&body), Ok(EnchantSkillInfo { header, entries: entries.to_vec() }));
    }

    #[test]
    fn receive_ex_show_quest_info_103f2130_reads_only_the_extended_envelope() {
        assert_eq!(read_ex_show_quest_info(&[0xfe, 0x19, 0]), Ok(()));
        assert_eq!(read_ex_show_quest_info(&[0xfe, 0x19]), Err(WireError::Short));
        assert_eq!(read_ex_show_quest_info(&[0xfe]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_show_quest_mark_1040c560_reads_the_dword() {
        let mut body = vec![0xfe, 0x1a, 0];
        body.extend_from_slice(&(-77_i32).to_le_bytes());
        assert_eq!(read_ex_show_quest_mark(&body), Ok(ShowQuestMark { value: -77 }));
        assert_eq!(read_ex_show_quest_mark(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_send_manor_list_1040c670_reads_count_and_named_entries() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x001b);
        writer.write_d(2);
        writer.write_d(7);
        writer.write_s("Gludio");
        writer.write_d(9);
        writer.write_s("Giran");
        let body = writer.into_bytes();
        assert_eq!(read_ex_send_manor_list(&body), Ok(ManorList {
            entries: vec![
                ManorEntry { id: 7, name: "Gludio".into() },
                ManorEntry { id: 9, name: "Giran".into() },
            ],
        }));
        assert_eq!(read_ex_send_manor_list(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_show_seed_info_1040c800_reads_nested_seed_blocks() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x001c);
        writer.write_c(1);
        writer.write_d(20);
        writer.write_d(30);
        writer.write_d(1);
        writer.write_d(100);
        writer.write_d(101);
        writer.write_d(102);
        writer.write_d(103);
        writer.write_d(104);
        writer.write_c(2);
        writer.write_d(200);
        writer.write_d(201);
        writer.write_c(1);
        writer.write_d(300);
        let body = writer.into_bytes();
        assert_eq!(read_ex_show_seed_info(&body), Ok(SeedInfo {
            flag: 1,
            values: [20, 30],
            entries: vec![SeedInfoEntry { values: [100, 101, 102, 103, 104], first: vec![200, 201], second: vec![300] }],
        }));
        assert_eq!(read_ex_show_seed_info(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_show_crop_info_1040cad0_reads_nested_crop_blocks() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x001d);
        writer.write_c(2);
        writer.write_d(40);
        writer.write_d(50);
        writer.write_d(1);
        writer.write_d(110);
        writer.write_d(111);
        writer.write_d(112);
        writer.write_d(113);
        writer.write_c(3);
        writer.write_d(114);
        writer.write_c(1);
        writer.write_d(210);
        writer.write_c(1);
        writer.write_d(310);
        let body = writer.into_bytes();
        assert_eq!(read_ex_show_crop_info(&body), Ok(CropInfo {
            flag: 2,
            values: [40, 50],
            entries: vec![CropInfoEntry {
                values: [110, 111, 112, 113],
                flag: 3,
                value: 114,
                first: vec![210],
                second: vec![310],
            }],
        }));
        assert_eq!(read_ex_show_crop_info(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_show_manor_default_info_1040cdc0_reads_nested_blocks() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x001e);
        writer.write_c(4);
        writer.write_d(1);
        writer.write_d(120);
        writer.write_d(121);
        writer.write_d(122);
        writer.write_d(123);
        writer.write_c(1);
        writer.write_d(220);
        writer.write_c(2);
        writer.write_d(320);
        writer.write_d(321);
        let body = writer.into_bytes();
        assert_eq!(read_ex_show_manor_default_info(&body), Ok(ManorDefaultInfo {
            flag: 4,
            entries: vec![ManorDefaultEntry { values: [120, 121, 122, 123], first: vec![220], second: vec![320, 321] }],
        }));
        assert_eq!(read_ex_show_manor_default_info(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_show_seed_setting_1040d070_reads_the_eight_values_inside_each_entry() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x001f);
        writer.write_d(10);
        writer.write_d(2);
        for base in [30, 130] {
            writer.write_d(base);
            writer.write_d(base + 1);
            writer.write_c(1);
            writer.write_d(base + 10);
            writer.write_c(2);
            writer.write_d(base + 20);
            writer.write_d(base + 21);
            for value in 0..8 {
                writer.write_d(base + 30 + value);
            }
        }
        let body = writer.into_bytes();
        let entry = |base: i32| SeedSettingEntry {
            values: [base, base + 1],
            first: vec![base + 10],
            second: vec![base + 20, base + 21],
            tail: std::array::from_fn(|k| base + 30 + k as i32),
        };
        assert_eq!(read_ex_show_seed_setting(&body), Ok(SeedSetting { value: 10, entries: vec![entry(30), entry(130)] }));
        assert_eq!(read_ex_show_seed_setting(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_show_crop_setting_1040d3b0_reads_ddc_lists_then_ddddddcddc() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0020);
        writer.write_d(10);
        writer.write_d(1);
        writer.write_d(30);
        writer.write_d(31);
        writer.write_c(1);
        writer.write_d(40);
        writer.write_c(0xff);
        for value in 50..56 {
            writer.write_d(value);
        }
        writer.write_c(3);
        writer.write_d(60);
        writer.write_d(61);
        writer.write_c(4);
        let body = writer.into_bytes();
        assert_eq!(read_ex_show_crop_setting(&body), Ok(CropSettingInfo {
            value: 10,
            entries: vec![CropSettingInfoEntry {
                values: [30, 31],
                first: vec![40],
                second: vec![],
                settings: [50, 51, 52, 53, 54, 55],
                flag: 3,
                tail: [60, 61],
                last: 4,
            }],
        }));
        assert_eq!(read_ex_show_crop_setting(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_show_sell_crop_list_1040d710_reads_nested_blocks() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0021);
        writer.write_d(10);
        writer.write_d(1);
        writer.write_d(30);
        writer.write_d(31);
        writer.write_d(32);
        writer.write_c(1);
        writer.write_d(40);
        writer.write_c(2);
        writer.write_d(50);
        writer.write_d(51);
        writer.write_d(60);
        writer.write_d(61);
        writer.write_d(62);
        writer.write_c(3);
        writer.write_d(63);
        let body = writer.into_bytes();
        assert_eq!(read_ex_show_sell_crop_list(&body), Ok(SellCropList {
            value: 10,
            entries: vec![SellCropEntry {
                values: [30, 31, 32],
                first: vec![40],
                second: vec![50, 51],
                tail: [60, 61, 62],
                flag: 3,
                value: 63,
            }],
        }));
        assert_eq!(read_ex_show_sell_crop_list(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_show_procure_crop_detail_1040da20_reads_entries() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0022);
        writer.write_d(10);
        writer.write_d(2);
        writer.write_d(30);
        writer.write_d(31);
        writer.write_d(32);
        writer.write_c(3);
        writer.write_d(40);
        writer.write_d(41);
        writer.write_d(42);
        writer.write_c(4);
        let body = writer.into_bytes();
        assert_eq!(read_ex_show_procure_crop_detail(&body), Ok(ProcureCropDetail {
            value: 10,
            entries: vec![
                ProcureCropDetailEntry { values: [30, 31, 32], flag: 3 },
                ProcureCropDetailEntry { values: [40, 41, 42], flag: 4 },
            ],
        }));
        assert_eq!(read_ex_show_procure_crop_detail(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_hero_list_1040dbe0_reads_seven_fields_per_hero() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0023);
        writer.write_d(2);
        for (name, base) in [("Hero", 10), ("Other", 50)] {
            writer.write_s(name);
            writer.write_d(base);
            writer.write_s("Clan");
            writer.write_d(base + 10);
            writer.write_s("Ally");
            writer.write_d(base + 20);
            writer.write_d(base + 30);
        }
        let body = writer.into_bytes();
        let hero = |name: &str, base: i32| HeroEntry {
            names: [name.into(), "Clan".into(), "Ally".into()],
            values: [base, base + 10, base + 20, base + 30],
        };
        assert_eq!(read_ex_hero_list(&body), Ok(HeroList { entries: vec![hero("Hero", 10), hero("Other", 50)] }));
        assert_eq!(read_ex_hero_list(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_server_primitive_1040de40_reads_typed_entries() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0024);
        writer.write_s("Primitive");
        writer.write_d(10);
        writer.write_d(11);
        writer.write_d(12);
        writer.write_d(13);
        writer.write_d(14);
        writer.write_d(2);
        writer.write_c(1);
        writer.write_s("Point");
        for value in 21..28 {
            writer.write_d(value);
        }
        writer.write_c(2);
        writer.write_s("Line");
        for value in 31..41 {
            writer.write_d(value);
        }
        let body = writer.into_bytes();
        assert_eq!(read_ex_server_primitive(&body), Ok(ServerPrimitive {
            name: "Primitive".into(),
            values: [10, 11, 12, 13, 14, 2],
            entries: vec![
                ServerPrimitiveEntry { kind: 1, text: Some("Point".into()), values: (21..28).collect() },
                ServerPrimitiveEntry { kind: 2, text: Some("Line".into()), values: (31..41).collect() },
            ],
        }));
        assert_eq!(read_ex_server_primitive(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_ask_join_mpcc_1040e480_reads_name() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0027);
        writer.write_s("Alliance");
        let body = writer.into_bytes();
        assert_eq!(read_ex_ask_join_mpcc(&body), Ok(AskJoinMpcc { name: "Alliance".into() }));
        assert_eq!(read_ex_ask_join_mpcc(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_pledge_crest_large_1040e560_reads_three_dwords_and_the_crest_bytes() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0028);
        writer.write_d(10);
        writer.write_d(20);
        writer.write_d(3);
        writer.write_b(&[7, 8, 9]);
        let body = writer.into_bytes();
        assert_eq!(read_ex_pledge_crest_large(&body), Ok(PledgeCrestLarge { values: [10, 20, 3], data: vec![7, 8, 9] }));
        assert_eq!(read_ex_pledge_crest_large(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_olympiad_user_info_1040e740_reads_cd_s_ddddd() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0029);
        writer.write_c(1);
        writer.write_d(10);
        writer.write_s("Hero");
        for value in 20..25 {
            writer.write_d(value);
        }
        let body = writer.into_bytes();
        assert_eq!(read_ex_olympiad_user_info(&body), Ok(OlympiadUserInfo {
            flag: 1,
            value: 10,
            name: "Hero".into(),
            values: [20, 21, 22, 23, 24],
        }));
        assert_eq!(read_ex_olympiad_user_info(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_olympiad_spelled_info_1040e8f0_reads_dhd_entries() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x002a);
        writer.write_d(7);
        writer.write_d(1);
        writer.write_d(100);
        writer.write_h(3);
        writer.write_d(200);
        let body = writer.into_bytes();
        assert_eq!(read_ex_olympiad_spelled_info(&body), Ok(OlympiadSpelledInfo {
            values: [7, 1],
            entries: vec![OlympiadSpelledEntry { value: 100, level: 3, amount: 200 }],
        }));
        assert_eq!(read_ex_olympiad_spelled_info(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_olympiad_mode_1040ea70_reads_mode_byte() {
        let body = [0xfe, 0x2b, 0, 7];
        assert_eq!(read_ex_olympiad_mode(&body), Ok(OlympiadMode { mode: 7 }));
        assert_eq!(read_ex_olympiad_mode(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_event_match_manage_1040ecc0_reads_header_and_entries() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x002f);
        writer.write_d(10);
        writer.write_c(1);
        writer.write_c(2);
        writer.write_s("First");
        writer.write_c(4);
        writer.write_s("Second");
        writer.write_c(5);
        writer.write_d(1);
        writer.write_c(6);
        writer.write_c(7);
        writer.write_d(20);
        writer.write_s("Entry");
        writer.write_d(30);
        writer.write_d(31);
        let body = writer.into_bytes();
        assert_eq!(read_ex_event_match_manage(&body), Ok(EventMatchManage {
            value: 10,
            flags: [1, 2],
            first_name: "First".into(),
            middle_flag: 4,
            second_name: "Second".into(),
            last_flag: 5,
            entries: vec![EventMatchManageEntry { flags: [6, 7], value: 20, name: "Entry".into(), values: [30, 31] }],
        }));
        assert_eq!(read_ex_event_match_manage(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_multi_party_command_channel_info_1040ef80_reads_members() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0030);
        writer.write_s("Channel");
        writer.write_d(11);
        writer.write_d(1);
        writer.write_s("Leader");
        writer.write_d(22);
        let body = writer.into_bytes();
        assert_eq!(read_ex_multi_party_command_channel_info(&body), Ok(MultiPartyChannelInfo {
            name: "Channel".into(),
            values: [11, 1],
            members: vec![MultiPartyChannelMember { name: "Leader".into(), value: 22 }],
        }));
        assert_eq!(read_ex_multi_party_command_channel_info(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_regen_max_1040b0c0_reads_dddf() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0001);
        writer.write_d(10);
        writer.write_d(20);
        writer.write_d(30);
        writer.write_f(1.5);
        let body = writer.into_bytes();
        assert_eq!(read_ex_regen_max(&body), Ok(RegenMax { values: [10, 20, 30], amount: 1.5 }));
        assert_eq!(read_ex_regen_max(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_event_match_observer_1040b180_reads_dcc_ss_ddd() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0003);
        writer.write_d(10);
        writer.write_c(1);
        writer.write_c(2);
        writer.write_s("Left");
        writer.write_s("Right");
        writer.write_d(20);
        writer.write_d(21);
        writer.write_d(22);
        let body = writer.into_bytes();
        assert_eq!(read_ex_event_match_observer(&body), Ok(EventMatchObserver {
            value: 10,
            flags: [1, 2],
            names: ["Left".into(), "Right".into()],
            values: [20, 21, 22],
        }));
        assert_eq!(read_ex_event_match_observer(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_event_match_message_1040b2f0_reads_flag_and_message() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0004);
        writer.write_c(2);
        writer.write_s("Match started");
        let body = writer.into_bytes();
        assert_eq!(read_ex_event_match_message(&body), Ok(EventMatchMessage { flag: 2, message: "Match started".into() }));
        assert_eq!(read_ex_event_match_message(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_event_match_score_1040b3e0_reads_three_dwords() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0005);
        writer.write_d(10);
        writer.write_d(20);
        writer.write_d(30);
        let body = writer.into_bytes();
        assert_eq!(read_ex_event_match_score(&body), Ok(EventMatchScore { values: [10, 20, 30] }));
        assert_eq!(read_ex_event_match_score(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_event_match_team_info_1040b4c0_reads_header_and_team() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0006);
        writer.write_d(1);
        writer.write_d(11);
        writer.write_d(1);
        writer.write_d(20);
        writer.write_s("Blue");
        for value in 30..40 {
            writer.write_d(value);
        }
        let body = writer.into_bytes();
        assert_eq!(read_ex_event_match_team_info(&body), Ok(EventMatchTeamInfo {
            header: [1, 11, 1],
            entries: vec![EventMatchTeamEntry { id: 20, name: "Blue".into(), values: [30, 31, 32, 33, 34, 35, 36, 37, 38, 39] }],
        }));
        assert_eq!(read_ex_event_match_team_info(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_event_match_create_1040b6d0_reads_dword() {
        let body = [0xfe, 0x07, 0, 42, 0, 0, 0];
        assert_eq!(read_ex_event_match_create(&body), Ok(EventMatchCreate { value: 42 }));
        assert_eq!(read_ex_event_match_create(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_event_match_user_info_1040b7e0_reads_name_and_values() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0008);
        writer.write_d(10);
        writer.write_s("Player");
        for value in 20..28 {
            writer.write_d(value);
        }
        let body = writer.into_bytes();
        assert_eq!(read_ex_event_match_user_info(&body), Ok(EventMatchUserInfo {
            id: 10,
            name: "Player".into(),
            values: [20, 21, 22, 23, 24, 25, 26, 27],
        }));
        assert_eq!(read_ex_event_match_user_info(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_colosseum_fence_info_1041fb10_reads_seven_dwords() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0009);
        for value in 10..17 {
            writer.write_d(value);
        }
        let body = writer.into_bytes();
        assert_eq!(read_ex_colosseum_fence_info(&body), Ok(ColosseumFenceInfo { values: [10, 11, 12, 13, 14, 15, 16] }));
        assert_eq!(read_ex_colosseum_fence_info(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_event_match_spelled_info_1040b920_reads_dd_and_dhd() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x000a);
        writer.write_d(10);
        writer.write_d(20);
        writer.write_d(1);
        writer.write_d(30);
        writer.write_h(4);
        writer.write_d(40);
        let body = writer.into_bytes();
        assert_eq!(read_ex_event_match_spelled_info(&body), Ok(EventMatchSpelledInfo {
            values: [10, 20, 1],
            entries: vec![EventMatchSpelledEntry { value: 30, level: 4, amount: 40 }],
        }));
        assert_eq!(read_ex_event_match_spelled_info(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_event_match_firecracker_10419a00_reads_dword() {
        let body = [0xfe, 0x0b, 0, 42, 0, 0, 0];
        assert_eq!(read_ex_event_match_firecracker(&body), Ok(EventMatchFirecracker { value: 42 }));
        assert_eq!(read_ex_event_match_firecracker(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_event_match_team_unlocked_1040bab0_reads_dc() {
        let body = [0xfe, 0x0c, 0, 42, 0, 0, 0, 1];
        assert_eq!(read_ex_event_match_team_unlocked(&body), Ok(EventMatchTeamUnlocked { value: 42, flag: 1 }));
        assert_eq!(read_ex_event_match_team_unlocked(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_event_match_gm_test_103f1f60_reads_envelope_only() {
        let body = [0xfe, 0x0d, 0];
        assert_eq!(read_ex_event_match_gm_test(&body), Ok(()));
        assert_eq!(read_ex_event_match_gm_test(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_close_party_room_103f2000_reads_envelope_only() {
        let body = [0xfe, 0x0f, 0];
        assert_eq!(read_ex_close_party_room(&body), Ok(()));
        assert_eq!(read_ex_close_party_room(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_fishing_start_10419b40_reads_five_dwords() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0013);
        for value in 10..15 {
            writer.write_d(value);
        }
        let body = writer.into_bytes();
        assert_eq!(read_ex_fishing_start(&body), Ok(FishingStart { values: [10, 11, 12, 13, 14] }));
        assert_eq!(read_ex_fishing_start(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_open_mpcc_103f2280_reads_envelope_only() {
        let body = [0xfe, 0x25, 0];
        assert_eq!(read_ex_open_mpcc(&body), Ok(()));
        assert_eq!(read_ex_open_mpcc(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_close_mpcc_103f2310_reads_envelope_only() {
        let body = [0xfe, 0x26, 0];
        assert_eq!(read_ex_close_mpcc(&body), Ok(()));
        assert_eq!(read_ex_close_mpcc(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_olympiad_match_end_103f23a0_reads_envelope_only() {
        let body = [0xfe, 0x2c, 0];
        assert_eq!(read_ex_olympiad_match_end(&body), Ok(()));
        assert_eq!(read_ex_olympiad_match_end(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_mail_arrived_103f2440_reads_envelope_only() {
        let body = [0xfe, 0x2d, 0];
        assert_eq!(read_ex_mail_arrived(&body), Ok(()));
        assert_eq!(read_ex_mail_arrived(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_party_room_member_1040bb90_reads_header_and_entries() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x000e);
        writer.write_d(10);
        writer.write_d(1);
        writer.write_d(20);
        writer.write_s("Member");
        writer.write_d(30);
        writer.write_d(31);
        writer.write_d(32);
        writer.write_d(33);
        let body = writer.into_bytes();
        assert_eq!(read_ex_party_room_member(&body), Ok(PartyRoomMember {
            header: [10, 1],
            entries: vec![PartyRoomMemberEntry { id: 20, name: "Member".into(), values: [30, 31, 32, 33] }],
        }));
        assert_eq!(read_ex_party_room_member(&body[..body.len() - 1]), Err(WireError::Short));
    }

    #[test]
    fn receive_ex_manage_party_room_member_1040bda0_reads_dd_s_dddd() {
        let mut writer = Writer::new();
        writer.write_c(0xfe);
        writer.write_h(0x0010);
        writer.write_d(10);
        writer.write_d(11);
        writer.write_s("Member");
        writer.write_d(20);
        writer.write_d(21);
        writer.write_d(22);
        writer.write_d(23);
        let body = writer.into_bytes();
        assert_eq!(read_ex_manage_party_room_member(&body), Ok(ManagePartyRoomMember {
            header: [10, 11],
            name: "Member".into(),
            values: [20, 21, 22, 23],
        }));
        assert_eq!(read_ex_manage_party_room_member(&body[..body.len() - 1]), Err(WireError::Short));
    }
}
