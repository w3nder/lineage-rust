use super::{OP_EXTENDED_CLIENT, counted, read_d_or_zero};
use crate::wire::{Reader, Result, WireError, Writer};

pub const OP_CASTLE_SIEGE_INFO: u8 = 0xc9;
pub const OP_CASTLE_SIEGE_ATTACKER_LIST: u8 = 0xca;
pub const OP_CASTLE_SIEGE_DEFENDER_LIST: u8 = 0xcb;
pub const OP_RECIPE_BOOK_ITEM_LIST: u8 = 0xd6;
pub const OP_RECIPE_ITEM_MAKE_INFO: u8 = 0xd7;
pub const OP_RECIPE_SHOP_MANAGE_LIST: u8 = 0xd8;
pub const OP_RECIPE_SHOP_SELL_LIST: u8 = 0xd9;
pub const OP_RECIPE_SHOP_ITEM_INFO: u8 = 0xda;
pub const OP_RECIPE_SHOP_MSG: u8 = 0xdb;
pub const OP_SHOW_CALCULATOR: u8 = 0xdc;
pub const OP_MON_RACE_INFO: u8 = 0xdd;
pub const OP_SHOW_XMAS_SEAL: u8 = 0xf2;
pub const OP_SSQ_STATUS: u8 = 0xf5;
pub const OP_AGIT_DECO_INFO: u8 = 0xf7;
pub const OP_SSQ_INFO: u8 = 0xf8;

pub const OP_REQUEST_SIEGE_INFO: u8 = 0x47;
pub const OP_REQUEST_START_PLEDGE_WAR: u8 = 0x4d;
pub const OP_REQUEST_STOP_PLEDGE_WAR: u8 = 0x4f;
pub const OP_REQUEST_PETITION: u8 = 0x7f;
pub const OP_REQUEST_PETITION_CANCEL: u8 = 0x80;
pub const OP_SEND_BYPASS_BUILD_CMD: u8 = 0x5b;
pub const OP_PETITION_VOTE_CLIENT: u8 = 0xc8;
pub const OP_REQUEST_OBSERVER_END: u8 = 0xb8;
pub const OP_REQUEST_GM_COMMAND: u8 = 0x6e;
pub const OP_GM_SNOOP_END: u8 = 0xab;
pub const OP_REQUEST_CASTLE_SIEGE_INFO: u8 = 0xa1;
pub const OP_REQUEST_CASTLE_SIEGE_ATTACKER_LIST: u8 = 0xa2;
pub const OP_REQUEST_CASTLE_SIEGE_DEFENDER_LIST: u8 = 0xa3;
pub const OP_REQUEST_JOIN_CASTLE_SIEGE: u8 = 0xa4;
pub const OP_REQUEST_CONFIRM_CASTLE_SIEGE_WAITING_LIST: u8 = 0xa5;
pub const OP_REQUEST_SET_CASTLE_SIEGE_TIME: u8 = 0xa6;
pub const OP_REQUEST_RECIPE_BOOK_OPEN: u8 = 0xac;
pub const OP_REQUEST_RECIPE_ITEM_DELETE: u8 = 0xad;
pub const OP_REQUEST_RECIPE_ITEM_MAKE_INFO: u8 = 0xae;
pub const OP_REQUEST_RECIPE_ITEM_MAKE_SELF: u8 = 0xaf;
pub const OP_REQUEST_RECIPE_SHOP_MANAGE_LIST: u8 = 0xb0;
pub const OP_REQUEST_RECIPE_SHOP_MESSAGE_SET: u8 = 0xb1;
pub const OP_REQUEST_RECIPE_SHOP_LIST_SET: u8 = 0xb2;
pub const OP_REQUEST_RECIPE_SHOP_MANAGE_QUIT: u8 = 0xb3;
pub const OP_REQUEST_RECIPE_SHOP_MANAGE_CANCEL: u8 = 0xb4;
pub const OP_REQUEST_RECIPE_SHOP_MAKE_INFO: u8 = 0xb5;
pub const OP_REQUEST_RECIPE_SHOP_MAKE_DO: u8 = 0xb6;
pub const OP_REQUEST_RECIPE_SHOP_SELL_LIST: u8 = 0xb7;
pub const OP_VOTE_SOCIALITY: u8 = 0xb9;
pub const OP_REQUEST_BUY_SEED: u8 = 0xc4;
pub const OP_REQUEST_SSQ_STATUS: u8 = 0xc7;
pub const EX_OUST_FROM_PARTY_ROOM: u16 = 0x01;
pub const EX_DISMISS_PARTY_ROOM: u16 = 0x02;
pub const EX_WITHDRAW_PARTY_ROOM: u16 = 0x03;
pub const EX_ENCHANT_SKILL_INFO: u16 = 0x06;
pub const EX_ENCHANT_SKILL: u16 = 0x07;
pub const EX_MANOR_LIST: u16 = 0x08;
pub const EX_PROCURE_CROP_LIST: u16 = 0x09;
pub const EX_SET_SEED: u16 = 0x0a;
pub const EX_SET_CROP: u16 = 0x0b;
pub const EX_WRITE_HERO_WORDS: u16 = 0x0c;
pub const EX_ASK_JOIN_MPCC: u16 = 0x0d;
pub const EX_ACCEPT_JOIN_MPCC: u16 = 0x0e;
pub const EX_OUST_FROM_MPCC: u16 = 0x0f;
pub const EX_PLEDGE_CREST_LARGE: u16 = 0x10;
pub const EX_SET_PLEDGE_CREST_LARGE: u16 = 0x11;

pub const SIEGE_NAME_MAX_CHARS: usize = 0x30 / 2 - 1;
pub const SSQ_NAME_MAX_CHARS: usize = 0x200 / 2 - 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CastleSiegeInfo {
    pub castle_id: i32,
    pub owner_flags: i32,
    pub owner_id: i32,
    pub pledge_name: String,
    pub leader_name: String,
    pub ally_id: i32,
    pub ally_name: String,
    pub now: i32,
    pub siege_time: i32,
    pub choices: Vec<i32>,
}

pub fn read_castle_siege_info(body: &[u8]) -> Result<CastleSiegeInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let castle_id = r.read_d()?;
    let owner_flags = r.read_d()?;
    let owner_id = r.read_d()?;
    let pledge_name = r.read_s()?;
    let leader_name = r.read_s()?;
    let ally_id = r.read_d()?;
    let ally_name = r.read_s()?;
    let now = r.read_d()?;
    let siege_time = r.read_d()?;
    if [&pledge_name, &leader_name, &ally_name].iter().any(|s| s.encode_utf16().count() > SIEGE_NAME_MAX_CHARS) {
        return Err(WireError::TooLong);
    }
    let declared = read_d_or_zero(&mut r)?;
    let count = counted(&r, declared, 4)?;
    let mut choices = Vec::with_capacity(count);
    for _ in 0..count {
        choices.push(r.read_d()?);
    }
    Ok(CastleSiegeInfo {
        castle_id,
        owner_flags,
        owner_id,
        pledge_name,
        leader_name,
        ally_id,
        ally_name,
        now,
        siege_time,
        choices,
    })
}

pub const SIEGE_CLAN_OWNER: i32 = 1;
pub const SIEGE_CLAN_DEFENDER_PENDING: i32 = 2;
pub const SIEGE_CLAN_DEFENDER: i32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiegeClan {
    pub pledge_id: i32,
    pub name: String,
    pub leader_name: String,
    pub crest_id: i32,
    pub signed_time: i32,
    pub kind: i32,
    pub ally_id: i32,
    pub ally_name: String,
    pub ally_leader_name: String,
    pub ally_crest_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiegeClanList {
    pub castle_id: i32,
    pub header: [i32; 3],
    pub total: i32,
    pub clans: Vec<SiegeClan>,
}

fn read_siege_clan_list(body: &[u8], with_kind: bool) -> Result<SiegeClanList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let castle_id = r.read_d()?;
    let mut header = [0i32; 3];
    for value in &mut header {
        *value = r.read_d()?;
    }
    let total = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 4 + 2 + 2 + 4 + 4 + 4 + 2 + 2 + 4)?;
    let mut clans = Vec::with_capacity(count);
    for _ in 0..count {
        let pledge_id = r.read_d()?;
        let name = r.read_s()?;
        let leader_name = r.read_s()?;
        let crest_id = r.read_d()?;
        let signed_time = r.read_d()?;
        let kind = if with_kind { r.read_d()? } else { 0 };
        let ally_id = r.read_d()?;
        let ally_name = r.read_s()?;
        let ally_leader_name = r.read_s()?;
        let ally_crest_id = r.read_d()?;
        clans.push(SiegeClan {
            pledge_id,
            name,
            leader_name,
            crest_id,
            signed_time,
            kind,
            ally_id,
            ally_name,
            ally_leader_name,
            ally_crest_id,
        });
    }
    Ok(SiegeClanList { castle_id, header, total, clans })
}

pub fn read_castle_siege_attacker_list(body: &[u8]) -> Result<SiegeClanList> {
    read_siege_clan_list(body, false)
}

pub fn read_castle_siege_defender_list(body: &[u8]) -> Result<SiegeClanList> {
    read_siege_clan_list(body, true)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeBookItemList {
    pub common: bool,
    pub max_mp: i32,
    pub recipes: Vec<(i32, i32)>,
}

pub fn read_recipe_book_item_list(body: &[u8]) -> Result<RecipeBookItemList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let common = r.read_d()? != 0;
    let max_mp = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 8)?;
    let mut recipes = Vec::with_capacity(count);
    for _ in 0..count {
        recipes.push((r.read_d()?, r.read_d()?));
    }
    Ok(RecipeBookItemList { common, max_mp, recipes })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipeItemMakeInfo {
    pub recipe_id: i32,
    pub common: bool,
    pub mp: i32,
    pub max_mp: i32,
    pub status: i32,
}

pub fn read_recipe_item_make_info(body: &[u8]) -> Result<RecipeItemMakeInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let recipe_id = r.read_d()?;
    let common = r.read_d()? != 0;
    let mp = r.read_d()?;
    let max_mp = r.read_d()?;
    let status = r.read_d()?;
    Ok(RecipeItemMakeInfo { recipe_id, common, mp, max_mp, status })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManufactureItem {
    pub recipe_id: i32,
    pub unknown: i32,
    pub cost: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeShopManageList {
    pub common: bool,
    pub mp: i32,
    pub max_mp: i32,
    pub recipes: Vec<(i32, i32)>,
    pub items: Vec<ManufactureItem>,
}

pub fn read_recipe_shop_manage_list(body: &[u8]) -> Result<RecipeShopManageList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let common = r.read_d()? != 0;
    let mp = r.read_d()?;
    let max_mp = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 8)?;
    let mut recipes = Vec::with_capacity(count);
    for _ in 0..count {
        recipes.push((r.read_d()?, r.read_d()?));
    }
    let declared = r.read_d()?;
    let count = counted(&r, declared, 12)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let recipe_id = r.read_d()?;
        let unknown = r.read_d()?;
        let cost = r.read_d()?;
        items.push(ManufactureItem { recipe_id, unknown, cost });
    }
    Ok(RecipeShopManageList { common, mp, max_mp, recipes, items })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeShopSellList {
    pub manufacturer_id: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub adena: i32,
    pub items: Vec<ManufactureItem>,
}

pub fn read_recipe_shop_sell_list(body: &[u8]) -> Result<RecipeShopSellList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let manufacturer_id = r.read_d()?;
    let mp = r.read_d()?;
    let max_mp = r.read_d()?;
    let adena = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 12)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let recipe_id = r.read_d()?;
        let unknown = r.read_d()?;
        let cost = r.read_d()?;
        items.push(ManufactureItem { recipe_id, unknown, cost });
    }
    Ok(RecipeShopSellList { manufacturer_id, mp, max_mp, adena, items })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipeShopItemInfo {
    pub manufacturer_id: i32,
    pub recipe_id: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub status: i32,
    pub cost: i32,
}

pub fn read_recipe_shop_item_info(body: &[u8]) -> Result<RecipeShopItemInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let manufacturer_id = r.read_d()?;
    let recipe_id = r.read_d()?;
    let mp = r.read_d()?;
    let max_mp = r.read_d()?;
    let status = r.read_d()?;
    let cost = r.read_d()?;
    Ok(RecipeShopItemInfo { manufacturer_id, recipe_id, mp, max_mp, status, cost })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeShopMsg {
    pub object_id: i32,
    pub message: String,
}

pub fn read_recipe_shop_msg(body: &[u8]) -> Result<RecipeShopMsg> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let message = r.read_s()?;
    Ok(RecipeShopMsg { object_id, message })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShowCalculator {
    pub calculator_id: i32,
}

pub fn read_show_calculator(body: &[u8]) -> Result<ShowCalculator> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let calculator_id = r.read_d()?;
    Ok(ShowCalculator { calculator_id })
}

pub const RACE_SPEED_STEPS: usize = 20;

#[derive(Debug, Clone, PartialEq)]
pub struct RaceMonster {
    pub object_id: i32,
    pub npc_id: i32,
    pub origin: [i32; 3],
    pub end: [i32; 3],
    pub collision_height: f64,
    pub collision_radius: f64,
    pub unknown: i32,
    pub speeds: [u8; RACE_SPEED_STEPS],
    pub tail: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonRaceInfo {
    pub state: i32,
    pub second: i32,
    pub monsters: Vec<RaceMonster>,
}

pub fn read_mon_race_info(body: &[u8]) -> Result<MonRaceInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let state = r.read_d()?;
    let second = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 8 * 4 + 16 + 4 + RACE_SPEED_STEPS + 4)?;
    let mut monsters = Vec::with_capacity(count);
    for _ in 0..count {
        let object_id = r.read_d()?;
        let npc_id = r.read_d()?.wrapping_sub(super::NPC_TYPE_OFFSET);
        let origin = [r.read_d()?, r.read_d()?, r.read_d()?];
        let end = [r.read_d()?, r.read_d()?, r.read_d()?];
        let collision_height = r.read_f()?;
        let collision_radius = r.read_f()?;
        let unknown = r.read_d()?;
        let mut speeds = [0u8; RACE_SPEED_STEPS];
        for speed in &mut speeds {
            *speed = r.read_c()?;
        }
        let tail = r.read_d()?;
        monsters.push(RaceMonster {
            object_id,
            npc_id,
            origin,
            end,
            collision_height,
            collision_radius,
            unknown,
            speeds,
            tail,
        });
    }
    Ok(MonRaceInfo { state, second, monsters })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShowXMasSeal {
    pub item_id: i32,
}

pub fn read_show_xmas_seal(body: &[u8]) -> Result<ShowXMasSeal> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let item_id = r.read_d()?;
    Ok(ShowXMasSeal { item_id })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SsqSummary {
    pub cycle: i32,
    pub period_message: i32,
    pub until_message: i32,
    pub player_cabal: u8,
    pub player_seal: u8,
    pub stones: i32,
    pub adena: i32,
    pub dusk_stone_score: i32,
    pub dusk_festival_score: i32,
    pub dusk_total_score: i32,
    pub dusk_percent: u8,
    pub dawn_stone_score: i32,
    pub dawn_festival_score: i32,
    pub dawn_total_score: i32,
    pub dawn_percent: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsqFestival {
    pub id: u8,
    pub level_score: i32,
    pub dusk_score: i32,
    pub dusk_members: Vec<String>,
    pub dawn_score: i32,
    pub dawn_members: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsqFestivalGroup {
    pub flag: u8,
    pub festivals: Vec<SsqFestival>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SsqSeal {
    pub id: u8,
    pub owner: u8,
    pub dawn_percent: u8,
    pub dusk_percent: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SsqSealPrediction {
    pub id: u8,
    pub owner: u8,
    pub message: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SsqPage {
    Summary(SsqSummary),
    Festivals(Vec<SsqFestivalGroup>),
    Seals { retain_limit: u8, claim_limit: u8, seals: Vec<SsqSeal> },
    Predictions { winning_cabal: u8, seals: Vec<SsqSealPrediction> },
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsqStatus {
    pub page: u8,
    pub period: u8,
    pub body: SsqPage,
}

fn read_ssq_names(r: &mut Reader) -> Result<Vec<String>> {
    let count = r.read_c()? as usize;
    let mut names = Vec::with_capacity(count);
    for _ in 0..count {
        let name = r.read_s()?;
        if name.encode_utf16().count() > SSQ_NAME_MAX_CHARS {
            return Err(WireError::TooLong);
        }
        names.push(name);
    }
    Ok(names)
}

pub fn read_ssq_status(body: &[u8]) -> Result<SsqStatus> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let page = r.read_c()?;
    let period = r.read_c()?;
    let body = match page {
        1 => {
            let cycle = r.read_d()?;
            let period_message = r.read_d()?;
            let until_message = r.read_d()?;
            let player_cabal = r.read_c()?;
            let player_seal = r.read_c()?;
            let stones = r.read_d()?;
            let adena = r.read_d()?;
            let dusk_stone_score = r.read_d()?;
            let dusk_festival_score = r.read_d()?;
            let dusk_total_score = r.read_d()?;
            let dusk_percent = r.read_c()?;
            let dawn_stone_score = r.read_d()?;
            let dawn_festival_score = r.read_d()?;
            let dawn_total_score = r.read_d()?;
            let dawn_percent = r.read_c()?;
            SsqPage::Summary(SsqSummary {
                cycle,
                period_message,
                until_message,
                player_cabal,
                player_seal,
                stones,
                adena,
                dusk_stone_score,
                dusk_festival_score,
                dusk_total_score,
                dusk_percent,
                dawn_stone_score,
                dawn_festival_score,
                dawn_total_score,
                dawn_percent,
            })
        }
        2 => {
            let groups_count = r.read_c()? as usize;
            let mut groups = Vec::with_capacity(groups_count);
            for _ in 0..groups_count {
                let flag = r.read_c()?;
                let festival_count = r.read_c()? as usize;
                let mut festivals = Vec::with_capacity(festival_count);
                for _ in 0..festival_count {
                    let id = r.read_c()?;
                    let level_score = r.read_d()?;
                    let dusk_score = r.read_d()?;
                    let dusk_members = read_ssq_names(&mut r)?;
                    let dawn_score = r.read_d()?;
                    let dawn_members = read_ssq_names(&mut r)?;
                    festivals.push(SsqFestival { id, level_score, dusk_score, dusk_members, dawn_score, dawn_members });
                }
                groups.push(SsqFestivalGroup { flag, festivals });
            }
            SsqPage::Festivals(groups)
        }
        3 => {
            let retain_limit = r.read_c()?;
            let claim_limit = r.read_c()?;
            let count = r.read_c()? as usize;
            let mut seals = Vec::with_capacity(count);
            for _ in 0..count {
                let id = r.read_c()?;
                let owner = r.read_c()?;
                let dawn_percent = r.read_c()?;
                let dusk_percent = r.read_c()?;
                seals.push(SsqSeal { id, owner, dawn_percent, dusk_percent });
            }
            SsqPage::Seals { retain_limit, claim_limit, seals }
        }
        4 => {
            let winning_cabal = r.read_c()?;
            let count = r.read_c()? as usize;
            let mut seals = Vec::with_capacity(count);
            for _ in 0..count {
                let id = r.read_c()?;
                let owner = r.read_c()?;
                let message = r.read_d()?;
                seals.push(SsqSealPrediction { id, owner, message });
            }
            SsqPage::Predictions { winning_cabal, seals }
        }
        _ => SsqPage::Unknown,
    };
    Ok(SsqStatus { page, period, body })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgitDecoInfo {
    pub agit_id: i32,
}

pub fn read_agit_deco_info(body: &[u8]) -> Result<AgitDecoInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let agit_id = r.read_d()?;
    Ok(AgitDecoInfo { agit_id })
}

pub const SSQ_SKY_NONE: u16 = 256;
pub const SSQ_SKY_DAWN: u16 = 257;
pub const SSQ_SKY_DUSK: u16 = 258;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SsqInfo {
    pub sky: u16,
}

impl SsqInfo {
    pub fn env(&self) -> u8 {
        self.sky.to_le_bytes()[0]
    }

    pub fn immediate(&self) -> bool {
        self.sky.to_le_bytes()[1] == 1
    }
}

pub fn read_ssq_info(body: &[u8]) -> Result<SsqInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let low = r.read_c()?;
    let high = r.read_c()?;
    Ok(SsqInfo { sky: u16::from_le_bytes([low, high]) })
}

fn opcode_only(opcode: u8) -> Vec<u8> {
    vec![opcode]
}

fn opcode_int(opcode: u8, value: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(opcode);
    w.write_d(value);
    w.into_bytes()
}

fn opcode_string(opcode: u8, text: &str) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(opcode);
    w.write_s(text);
    w.into_bytes()
}

fn extended(sub: u16) -> Writer {
    let mut w = Writer::new();
    w.write_c(OP_EXTENDED_CLIENT);
    w.write_h(sub);
    w
}

pub fn request_siege_info() -> Vec<u8> {
    opcode_only(OP_REQUEST_SIEGE_INFO)
}

pub fn request_start_pledge_war(pledge_name: &str) -> Vec<u8> {
    opcode_string(OP_REQUEST_START_PLEDGE_WAR, pledge_name)
}

pub fn request_stop_pledge_war(pledge_name: &str) -> Vec<u8> {
    opcode_string(OP_REQUEST_STOP_PLEDGE_WAR, pledge_name)
}

pub fn request_petition(content: &str, kind: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_PETITION);
    w.write_s(content);
    w.write_d(kind);
    w.into_bytes()
}

pub fn request_petition_cancel(content: &str) -> Vec<u8> {
    opcode_string(OP_REQUEST_PETITION_CANCEL, content)
}

pub fn request_petition_vote(first: i32, index: i32, text: &str) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_PETITION_VOTE_CLIENT);
    w.write_d(first);
    w.write_d(index);
    w.write_s(text);
    w.into_bytes()
}

pub fn request_olympiad_observer_end() -> Vec<u8> {
    extended(0x12).into_bytes()
}

pub fn request_olympiad_match_list() -> Vec<u8> {
    extended(0x13).into_bytes()
}

pub fn request_gm_command(name: &str, kind: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_GM_COMMAND);
    w.write_s(name);
    w.write_d(kind);
    w.write_d(0);
    w.into_bytes()
}

pub fn request_gm_snoop_end(snoop_id: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_GM_SNOOP_END);
    w.write_d(snoop_id);
    w.into_bytes()
}

pub fn request_observer_end() -> Vec<u8> {
    vec![OP_REQUEST_OBSERVER_END]
}

pub fn send_bypass_build_cmd(command: &str) -> Vec<u8> {
    opcode_string(OP_SEND_BYPASS_BUILD_CMD, command)
}

pub fn request_castle_siege_info(castle_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_CASTLE_SIEGE_INFO, castle_id)
}

pub fn request_castle_siege_attacker_list(castle_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_CASTLE_SIEGE_ATTACKER_LIST, castle_id)
}

pub fn request_castle_siege_defender_list(castle_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_CASTLE_SIEGE_DEFENDER_LIST, castle_id)
}

fn opcode_three_ints(opcode: u8, a: i32, b: i32, c: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(opcode);
    w.write_d(a);
    w.write_d(b);
    w.write_d(c);
    w.into_bytes()
}

pub fn request_join_castle_siege(castle_id: i32, attacker: bool, join: bool) -> Vec<u8> {
    opcode_three_ints(OP_REQUEST_JOIN_CASTLE_SIEGE, castle_id, attacker as i32, join as i32)
}

pub fn request_confirm_castle_siege_waiting_list(castle_id: i32, pledge_id: i32, approved: bool) -> Vec<u8> {
    opcode_three_ints(OP_REQUEST_CONFIRM_CASTLE_SIEGE_WAITING_LIST, castle_id, pledge_id, approved as i32)
}

pub fn request_set_castle_siege_time(castle_id: i32, time: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_SET_CASTLE_SIEGE_TIME);
    w.write_d(castle_id);
    w.write_d(time);
    w.into_bytes()
}

pub fn request_recipe_book_open(common: bool) -> Vec<u8> {
    opcode_int(OP_REQUEST_RECIPE_BOOK_OPEN, common as i32)
}

pub fn request_recipe_item_delete(recipe_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_RECIPE_ITEM_DELETE, recipe_id)
}

pub fn request_recipe_item_make_info(recipe_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_RECIPE_ITEM_MAKE_INFO, recipe_id)
}

pub fn request_recipe_item_make_self(recipe_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_RECIPE_ITEM_MAKE_SELF, recipe_id)
}

pub fn request_recipe_shop_manage_list() -> Vec<u8> {
    opcode_only(OP_REQUEST_RECIPE_SHOP_MANAGE_LIST)
}

pub fn request_recipe_shop_message_set(message: &str) -> Vec<u8> {
    opcode_string(OP_REQUEST_RECIPE_SHOP_MESSAGE_SET, message)
}

pub fn request_recipe_shop_list_set(items: &[(i32, i32)]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_RECIPE_SHOP_LIST_SET);
    w.write_d(items.len() as i32);
    for (recipe_id, cost) in items {
        w.write_d(*recipe_id);
        w.write_d(*cost);
    }
    w.into_bytes()
}

pub fn request_recipe_shop_manage_quit() -> Vec<u8> {
    opcode_only(OP_REQUEST_RECIPE_SHOP_MANAGE_QUIT)
}

pub fn request_recipe_shop_manage_cancel() -> Vec<u8> {
    opcode_only(OP_REQUEST_RECIPE_SHOP_MANAGE_CANCEL)
}

pub fn request_recipe_shop_make_info(manufacturer_id: i32, recipe_id: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_RECIPE_SHOP_MAKE_INFO);
    w.write_d(manufacturer_id);
    w.write_d(recipe_id);
    w.into_bytes()
}

pub fn request_recipe_shop_make_do(manufacturer_id: i32, recipe_id: i32, cost: i32) -> Vec<u8> {
    opcode_three_ints(OP_REQUEST_RECIPE_SHOP_MAKE_DO, manufacturer_id, recipe_id, cost)
}

pub fn request_recipe_shop_sell_list(manufacturer_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_RECIPE_SHOP_SELL_LIST, manufacturer_id)
}

pub fn vote_sociality(target_id: i32) -> Vec<u8> {
    opcode_int(OP_VOTE_SOCIALITY, target_id)
}

pub fn request_buy_seed(manor_id: i32, orders: &[(i32, i32)]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_BUY_SEED);
    w.write_d(manor_id);
    w.write_d(orders.len() as i32);
    for (item_id, count) in orders {
        w.write_d(*item_id);
        w.write_d(*count);
    }
    w.into_bytes()
}

pub fn request_ssq_status(page: u8) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_SSQ_STATUS);
    w.write_c(page);
    w.into_bytes()
}

pub fn request_oust_from_party_room(object_id: i32) -> Vec<u8> {
    let mut w = extended(EX_OUST_FROM_PARTY_ROOM);
    w.write_d(object_id);
    w.into_bytes()
}

pub fn request_dismiss_party_room(room_id: i32, unknown: i32) -> Vec<u8> {
    let mut w = extended(EX_DISMISS_PARTY_ROOM);
    w.write_d(room_id);
    w.write_d(unknown);
    w.into_bytes()
}

pub fn request_withdraw_party_room(room_id: i32, unknown: i32) -> Vec<u8> {
    let mut w = extended(EX_WITHDRAW_PARTY_ROOM);
    w.write_d(room_id);
    w.write_d(unknown);
    w.into_bytes()
}

pub fn request_ex_enchant_skill_info(skill_id: i32, level: i32) -> Vec<u8> {
    let mut w = extended(EX_ENCHANT_SKILL_INFO);
    w.write_d(skill_id);
    w.write_d(level);
    w.into_bytes()
}

pub fn request_ex_enchant_skill(skill_id: i32, level: i32) -> Vec<u8> {
    let mut w = extended(EX_ENCHANT_SKILL);
    w.write_d(skill_id);
    w.write_d(level);
    w.into_bytes()
}

pub fn request_manor_list() -> Vec<u8> {
    extended(EX_MANOR_LIST).into_bytes()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CropProcure {
    pub object_id: i32,
    pub crop_id: i32,
    pub manor_id: i32,
    pub count: i32,
}

pub fn request_procure_crop_list(orders: &[CropProcure]) -> Vec<u8> {
    let mut w = extended(EX_PROCURE_CROP_LIST);
    w.write_d(orders.len() as i32);
    for order in orders {
        w.write_d(order.object_id);
        w.write_d(order.crop_id);
        w.write_d(order.manor_id);
        w.write_d(order.count);
    }
    w.into_bytes()
}

pub fn request_set_seed(manor_id: i32, seeds: &[(i32, i32, i32)]) -> Vec<u8> {
    let mut w = extended(EX_SET_SEED);
    w.write_d(manor_id);
    w.write_d(seeds.len() as i32);
    for (seed_id, amount, price) in seeds {
        w.write_d(*seed_id);
        w.write_d(*amount);
        w.write_d(*price);
    }
    w.into_bytes()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CropSetting {
    pub crop_id: i32,
    pub amount: i32,
    pub price: i32,
    pub reward: u8,
}

pub fn request_set_crop(manor_id: i32, crops: &[CropSetting]) -> Vec<u8> {
    let mut w = extended(EX_SET_CROP);
    w.write_d(manor_id);
    w.write_d(crops.len() as i32);
    for crop in crops {
        w.write_d(crop.crop_id);
        w.write_d(crop.amount);
        w.write_d(crop.price);
        w.write_c(crop.reward);
    }
    w.into_bytes()
}

pub fn request_write_hero_words(words: &str) -> Vec<u8> {
    let mut w = extended(EX_WRITE_HERO_WORDS);
    w.write_s(words);
    w.into_bytes()
}

pub fn request_ex_ask_join_mpcc(name: &str) -> Vec<u8> {
    let mut w = extended(EX_ASK_JOIN_MPCC);
    w.write_s(name);
    w.into_bytes()
}

pub fn request_ex_accept_join_mpcc(accept: bool) -> Vec<u8> {
    let mut w = extended(EX_ACCEPT_JOIN_MPCC);
    w.write_d(accept as i32);
    w.into_bytes()
}

pub fn request_ex_oust_from_mpcc(name: &str) -> Vec<u8> {
    let mut w = extended(EX_OUST_FROM_MPCC);
    w.write_s(name);
    w.into_bytes()
}

pub fn request_ex_pledge_crest_large(crest_id: i32) -> Vec<u8> {
    let mut w = extended(EX_PLEDGE_CREST_LARGE);
    w.write_d(crest_id);
    w.into_bytes()
}

pub fn request_ex_set_pledge_crest_large(data: &[u8]) -> Vec<u8> {
    let mut w = extended(EX_SET_PLEDGE_CREST_LARGE);
    w.write_d(data.len() as i32);
    w.write_b(data);
    w.into_bytes()
}

#[cfg(test)]
mod tests {
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
    fn castle_siege_info_reads_the_header_and_the_optional_choices() {
        let mut w = Writer::new();
        w.write_c(OP_CASTLE_SIEGE_INFO);
        w.write_d(3);
        w.write_d(7);
        w.write_d(33);
        w.write_s("Canastra");
        w.write_s("Luzal");
        w.write_d(44);
        w.write_s("Aliados");
        w.write_d(1000);
        w.write_d(2000);
        let lisvus_prefix = w.into_bytes();
        let mut lisvus = lisvus_prefix.clone();
        lisvus.extend_from_slice(&0i32.to_le_bytes());
        let info = read_castle_siege_info(&lisvus).unwrap();
        assert_eq!((info.castle_id, info.owner_flags, info.owner_id), (3, 7, 33));
        assert_eq!(
            (info.pledge_name.as_str(), info.leader_name.as_str(), info.ally_id, info.ally_name.as_str()),
            ("Canastra", "Luzal", 44, "Aliados")
        );
        assert_eq!((info.now, info.siege_time), (1000, 2000));
        assert!(info.choices.is_empty());
        assert!(read_castle_siege_info(&lisvus_prefix).unwrap().choices.is_empty());
        let mut with_choices = lisvus_prefix.clone();
        with_choices.extend(ds(0, &[2, 5, 6])[1..].iter());
        assert_eq!(read_castle_siege_info(&with_choices).unwrap().choices, vec![5, 6]);
        assert!(read_castle_siege_info(&lisvus_prefix[..lisvus_prefix.len() - 2]).is_err());
    }

    fn siege_clan(w: &mut Writer, pledge_id: i32, kind: Option<i32>) {
        w.write_d(pledge_id);
        w.write_s("C");
        w.write_s("L");
        w.write_d(5);
        w.write_d(0);
        if let Some(kind) = kind {
            w.write_d(kind);
        }
        w.write_d(44);
        w.write_s("A");
        w.write_s("");
        w.write_d(6);
    }

    #[test]
    fn siege_attacker_and_defender_lists() {
        let mut w = Writer::new();
        w.write_c(OP_CASTLE_SIEGE_ATTACKER_LIST);
        for v in [3, 0, 1, 0, 1, 1] {
            w.write_d(v);
        }
        siege_clan(&mut w, 33, None);
        let body = w.into_bytes();
        let list = read_castle_siege_attacker_list(&body).unwrap();
        assert_eq!((list.castle_id, list.header, list.total), (3, [0, 1, 0], 1));
        assert_eq!(list.clans[0].pledge_id, 33);
        assert_eq!(list.clans[0].kind, 0);
        assert_eq!(list.clans[0].ally_crest_id, 6);
        assert!(read_castle_siege_attacker_list(&body[..body.len() - 1]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_CASTLE_SIEGE_DEFENDER_LIST);
        for v in [3, 0, 1, 0, 1, 1] {
            w.write_d(v);
        }
        siege_clan(&mut w, 33, Some(SIEGE_CLAN_OWNER));
        let body = w.into_bytes();
        assert_eq!(read_castle_siege_defender_list(&body).unwrap().clans[0].kind, SIEGE_CLAN_OWNER);
        assert!(read_castle_siege_defender_list(&body[..body.len() - 4]).is_err());
        assert!(
            read_castle_siege_defender_list(&ds(OP_CASTLE_SIEGE_DEFENDER_LIST, &[3, 0, 1, 0, 0, 0]))
                .unwrap()
                .clans
                .is_empty()
        );
    }

    #[test]
    fn recipes() {
        let book = read_recipe_book_item_list(&ds(OP_RECIPE_BOOK_ITEM_LIST, &[1, 500, 2, 10, 1, 11, 2])).unwrap();
        assert!(book.common);
        assert_eq!((book.max_mp, book.recipes.clone()), (500, vec![(10, 1), (11, 2)]));
        assert!(read_recipe_book_item_list(&ds(OP_RECIPE_BOOK_ITEM_LIST, &[1, 500, 2, 10])).is_err());
        let info = read_recipe_item_make_info(&ds(OP_RECIPE_ITEM_MAKE_INFO, &[10, 0, 100, 500, 1])).unwrap();
        assert_eq!(info, RecipeItemMakeInfo { recipe_id: 10, common: false, mp: 100, max_mp: 500, status: 1 });
        assert!(read_recipe_item_make_info(&ds(OP_RECIPE_ITEM_MAKE_INFO, &[10, 0, 100, 500])).is_err());
        let manage =
            read_recipe_shop_manage_list(&ds(OP_RECIPE_SHOP_MANAGE_LIST, &[0, 100, 500, 1, 10, 1, 1, 10, 0, 300]))
                .unwrap();
        assert_eq!(manage.recipes, vec![(10, 1)]);
        assert_eq!(manage.items, vec![ManufactureItem { recipe_id: 10, unknown: 0, cost: 300 }]);
        assert!(
            read_recipe_shop_manage_list(&ds(OP_RECIPE_SHOP_MANAGE_LIST, &[0, 100, 500, 1, 10, 1, 1, 10])).is_err()
        );
        let sell =
            read_recipe_shop_sell_list(&ds(OP_RECIPE_SHOP_SELL_LIST, &[7, 100, 500, 9000, 1, 10, 0, 300])).unwrap();
        assert_eq!((sell.manufacturer_id, sell.adena, sell.items.len()), (7, 9000, 1));
        assert!(read_recipe_shop_sell_list(&ds(OP_RECIPE_SHOP_SELL_LIST, &[7, 100, 500, 9000, 1])).is_err());
        let item = read_recipe_shop_item_info(&ds(OP_RECIPE_SHOP_ITEM_INFO, &[7, 10, 100, 500, 1, 0])).unwrap();
        assert_eq!(
            item,
            RecipeShopItemInfo { manufacturer_id: 7, recipe_id: 10, mp: 100, max_mp: 500, status: 1, cost: 0 }
        );
        assert_eq!(
            read_recipe_shop_item_info(&ds(OP_RECIPE_SHOP_ITEM_INFO, &[7, 10, 100, 500, 1, 9])).unwrap().cost,
            9
        );
        assert!(read_recipe_shop_item_info(&ds(OP_RECIPE_SHOP_ITEM_INFO, &[7, 10, 100, 500, 1])).is_err());
        let mut w = Writer::new();
        w.write_c(OP_RECIPE_SHOP_MSG);
        w.write_d(7);
        w.write_s("crafto");
        let body = w.into_bytes();
        assert_eq!(read_recipe_shop_msg(&body).unwrap(), RecipeShopMsg { object_id: 7, message: "crafto".into() });
        assert!(read_recipe_shop_msg(&body[..body.len() - 2]).is_err());
    }

    #[test]
    fn calculator_xmas_agit_and_sky() {
        assert_eq!(
            read_show_calculator(&ds(OP_SHOW_CALCULATOR, &[4393])).unwrap(),
            ShowCalculator { calculator_id: 4393 }
        );
        assert!(read_show_calculator(&[OP_SHOW_CALCULATOR]).is_err());
        assert_eq!(read_show_xmas_seal(&ds(OP_SHOW_XMAS_SEAL, &[5555])).unwrap(), ShowXMasSeal { item_id: 5555 });
        assert_eq!(read_show_xmas_seal(&ds(OP_SHOW_XMAS_SEAL, &[5555, 0])).unwrap(), ShowXMasSeal { item_id: 5555 });
        assert_eq!(read_agit_deco_info(&ds(OP_AGIT_DECO_INFO, &[21, 0, 0])).unwrap(), AgitDecoInfo { agit_id: 21 });
        assert!(read_agit_deco_info(&[OP_AGIT_DECO_INFO, 1]).is_err());
        assert_eq!(read_ssq_info(&[OP_SSQ_INFO, 1, 1]).unwrap(), SsqInfo { sky: SSQ_SKY_DAWN });
        assert_eq!(read_ssq_info(&[OP_SSQ_INFO, 2, 1]).unwrap().sky, SSQ_SKY_DUSK);
        assert!(read_ssq_info(&[OP_SSQ_INFO, 1]).is_err());
        assert_eq!(read_ssq_info(&[OP_SSQ_INFO, 1, 1, 0]).unwrap(), SsqInfo { sky: SSQ_SKY_DAWN });
    }

    #[test]
    fn mon_race_info_reads_eight_runners_with_twenty_speed_steps() {
        let mut w = Writer::new();
        w.write_c(OP_MON_RACE_INFO);
        w.write_d(0);
        w.write_d(0);
        w.write_d(1);
        w.write_d(900);
        w.write_d(1_031_003);
        for v in [14107, 181875, -3566, 12080, 181875, -3566] {
            w.write_d(v);
        }
        w.write_f(10.0);
        w.write_f(5.0);
        w.write_d(120);
        for step in 0..RACE_SPEED_STEPS {
            w.write_c(step as u8);
        }
        w.write_d(0);
        let body = w.into_bytes();
        let race = read_mon_race_info(&body).unwrap();
        assert_eq!(race.monsters.len(), 1);
        let m = &race.monsters[0];
        assert_eq!((m.object_id, m.npc_id), (900, 31003));
        assert_eq!(m.origin, [14107, 181875, -3566]);
        assert_eq!(m.end, [12080, 181875, -3566]);
        assert_eq!((m.collision_height, m.collision_radius, m.unknown), (10.0, 5.0, 120));
        assert_eq!(m.speeds[19], 19);
        assert!(read_mon_race_info(&body[..body.len() - 1]).is_err());
    }

    #[test]
    fn ssq_status_reads_each_page_shape() {
        let mut w = Writer::new();
        w.write_c(OP_SSQ_STATUS);
        w.write_c(1);
        w.write_c(2);
        w.write_d(5);
        w.write_d(1183);
        w.write_d(1186);
        w.write_c(1);
        w.write_c(2);
        for v in [10, 20, 30, 40, 70] {
            w.write_d(v);
        }
        w.write_c(45);
        for v in [31, 41, 72] {
            w.write_d(v);
        }
        w.write_c(55);
        let body = w.into_bytes();
        let s = read_ssq_status(&body).unwrap();
        assert_eq!((s.page, s.period), (1, 2));
        match s.body {
            SsqPage::Summary(summary) => {
                assert_eq!((summary.cycle, summary.period_message, summary.until_message), (5, 1183, 1186));
                assert_eq!((summary.player_cabal, summary.player_seal, summary.stones, summary.adena), (1, 2, 10, 20));
                assert_eq!(
                    (summary.dusk_total_score, summary.dusk_percent, summary.dawn_total_score, summary.dawn_percent),
                    (70, 45, 72, 55)
                );
            }
            other => panic!("esperava a página 1, veio {other:?}"),
        }
        assert!(read_ssq_status(&body[..body.len() - 1]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_SSQ_STATUS);
        w.write_c(2);
        w.write_c(2);
        w.write_h(1);
        w.write_c(1);
        w.write_c(1);
        w.write_d(200);
        w.write_d(300);
        w.write_c(2);
        w.write_s("A");
        w.write_s("B");
        w.write_d(400);
        w.write_c(0);
        let s = read_ssq_status(&w.into_bytes()).unwrap();
        match s.body {
            SsqPage::Festivals(groups) => {
                assert_eq!(groups.len(), 1);
                assert_eq!(groups[0].flag, 0);
                let festival = &groups[0].festivals[0];
                assert_eq!(
                    (festival.id, festival.level_score, festival.dusk_score, festival.dawn_score),
                    (1, 200, 300, 400)
                );
                assert_eq!(festival.dusk_members, vec!["A".to_string(), "B".to_string()]);
                assert!(festival.dawn_members.is_empty());
            }
            other => panic!("esperava a página 2, veio {other:?}"),
        }
        let s = read_ssq_status(&[OP_SSQ_STATUS, 3, 2, 10, 35, 1, 1, 2, 60, 40]).unwrap();
        assert_eq!(
            s.body,
            SsqPage::Seals {
                retain_limit: 10,
                claim_limit: 35,
                seals: vec![SsqSeal { id: 1, owner: 2, dawn_percent: 60, dusk_percent: 40 }]
            }
        );
        let s = read_ssq_status(&[OP_SSQ_STATUS, 4, 2, 2, 1, 1, 2, 0x1c, 0x05, 0, 0]).unwrap();
        assert_eq!(
            s.body,
            SsqPage::Predictions {
                winning_cabal: 2,
                seals: vec![SsqSealPrediction { id: 1, owner: 2, message: 0x051c }]
            }
        );
        assert_eq!(read_ssq_status(&[OP_SSQ_STATUS, 9, 2]).unwrap().body, SsqPage::Unknown);
        assert!(read_ssq_status(&[OP_SSQ_STATUS, 3, 2, 10, 35, 1, 1]).is_err());
    }

    #[test]
    fn rare_encoders_follow_the_engine_formats() {
        assert_eq!(request_set_castle_siege_time(3, 0x12345678), [0xa6, 3, 0, 0, 0, 0x78, 0x56, 0x34, 0x12]);
        assert_eq!(request_siege_info(), [OP_REQUEST_SIEGE_INFO]);
        assert_eq!(request_start_pledge_war("C"), [OP_REQUEST_START_PLEDGE_WAR, b'C', 0, 0, 0]);
        assert_eq!(request_stop_pledge_war("C"), [OP_REQUEST_STOP_PLEDGE_WAR, b'C', 0, 0, 0]);
        assert_eq!(request_petition("p", 1), [OP_REQUEST_PETITION, b'p', 0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(request_petition_cancel("p"), [OP_REQUEST_PETITION_CANCEL, b'p', 0, 0, 0]);
        assert_eq!(request_petition_vote(1, 2, "t"), [OP_PETITION_VOTE_CLIENT, 1, 0, 0, 0, 2, 0, 0, 0, b't', 0, 0, 0]);
        assert_eq!(request_observer_end(), [OP_REQUEST_OBSERVER_END]);
        assert_eq!(request_gm_command("a", 5), [OP_REQUEST_GM_COMMAND, b'a', 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(request_gm_snoop_end(7), [OP_GM_SNOOP_END, 7, 0, 0, 0]);
        assert_eq!(request_olympiad_observer_end(), [OP_EXTENDED_CLIENT, 0x12, 0]);
        assert_eq!(request_olympiad_match_list(), [OP_EXTENDED_CLIENT, 0x13, 0]);
        assert_eq!(send_bypass_build_cmd("//a"), [OP_SEND_BYPASS_BUILD_CMD, b'/', 0, b'/', 0, b'a', 0, 0, 0]);
        assert_eq!(request_castle_siege_info(3), ds(OP_REQUEST_CASTLE_SIEGE_INFO, &[3]));
        assert_eq!(request_castle_siege_attacker_list(3), ds(OP_REQUEST_CASTLE_SIEGE_ATTACKER_LIST, &[3]));
        assert_eq!(request_castle_siege_defender_list(3), ds(OP_REQUEST_CASTLE_SIEGE_DEFENDER_LIST, &[3]));
        assert_eq!(request_join_castle_siege(3, true, true), ds(OP_REQUEST_JOIN_CASTLE_SIEGE, &[3, 1, 1]));
        assert_eq!(
            request_confirm_castle_siege_waiting_list(3, 33, false),
            ds(OP_REQUEST_CONFIRM_CASTLE_SIEGE_WAITING_LIST, &[3, 33, 0])
        );
        assert_eq!(request_recipe_book_open(true), ds(OP_REQUEST_RECIPE_BOOK_OPEN, &[1]));
        assert_eq!(request_recipe_item_delete(10), ds(OP_REQUEST_RECIPE_ITEM_DELETE, &[10]));
        assert_eq!(request_recipe_item_make_info(10), ds(OP_REQUEST_RECIPE_ITEM_MAKE_INFO, &[10]));
        assert_eq!(request_recipe_item_make_self(10), ds(OP_REQUEST_RECIPE_ITEM_MAKE_SELF, &[10]));
        assert_eq!(request_recipe_shop_manage_list(), [OP_REQUEST_RECIPE_SHOP_MANAGE_LIST]);
        assert_eq!(request_recipe_shop_message_set("m"), [OP_REQUEST_RECIPE_SHOP_MESSAGE_SET, b'm', 0, 0, 0]);
        assert_eq!(request_recipe_shop_list_set(&[(10, 300)]), ds(OP_REQUEST_RECIPE_SHOP_LIST_SET, &[1, 10, 300]));
        assert_eq!(request_recipe_shop_manage_quit(), [OP_REQUEST_RECIPE_SHOP_MANAGE_QUIT]);
        assert_eq!(request_recipe_shop_manage_cancel(), [OP_REQUEST_RECIPE_SHOP_MANAGE_CANCEL]);
        assert_eq!(request_recipe_shop_make_info(7, 10), ds(OP_REQUEST_RECIPE_SHOP_MAKE_INFO, &[7, 10]));
        assert_eq!(request_recipe_shop_make_do(7, 10, 300), ds(OP_REQUEST_RECIPE_SHOP_MAKE_DO, &[7, 10, 300]));
        assert_eq!(request_recipe_shop_sell_list(7), ds(OP_REQUEST_RECIPE_SHOP_SELL_LIST, &[7]));
        assert_eq!(vote_sociality(7), ds(OP_VOTE_SOCIALITY, &[7]));
        assert_eq!(request_buy_seed(1, &[(5016, 10)]), ds(OP_REQUEST_BUY_SEED, &[1, 1, 5016, 10]));
        assert_eq!(request_ssq_status(2), [OP_REQUEST_SSQ_STATUS, 2]);
        assert_eq!(
            request_oust_from_party_room(7),
            [vec![OP_EXTENDED_CLIENT, 1, 0], ds(0, &[7])[1..].to_vec()].concat()
        );
        assert_eq!(
            request_dismiss_party_room(1, 0),
            [vec![OP_EXTENDED_CLIENT, 2, 0], ds(0, &[1, 0])[1..].to_vec()].concat()
        );
        assert_eq!(
            request_withdraw_party_room(1, 0),
            [vec![OP_EXTENDED_CLIENT, 3, 0], ds(0, &[1, 0])[1..].to_vec()].concat()
        );
        assert_eq!(
            request_ex_enchant_skill_info(3, 101),
            [vec![OP_EXTENDED_CLIENT, 6, 0], ds(0, &[3, 101])[1..].to_vec()].concat()
        );
        assert_eq!(
            request_ex_enchant_skill(3, 101),
            [vec![OP_EXTENDED_CLIENT, 7, 0], ds(0, &[3, 101])[1..].to_vec()].concat()
        );
        assert_eq!(request_manor_list(), [OP_EXTENDED_CLIENT, 8, 0]);
        let procure = request_procure_crop_list(&[CropProcure { object_id: 1, crop_id: 2, manor_id: 3, count: 4 }]);
        assert_eq!(procure, [vec![OP_EXTENDED_CLIENT, 9, 0], ds(0, &[1, 1, 2, 3, 4])[1..].to_vec()].concat());
        assert_eq!(
            request_set_seed(1, &[(5016, 10, 5)]),
            [vec![OP_EXTENDED_CLIENT, 10, 0], ds(0, &[1, 1, 5016, 10, 5])[1..].to_vec()].concat()
        );
        let crop = request_set_crop(1, &[CropSetting { crop_id: 2, amount: 3, price: 4, reward: 1 }]);
        assert_eq!(crop, [vec![OP_EXTENDED_CLIENT, 11, 0], ds(0, &[1, 1, 2, 3, 4])[1..].to_vec(), vec![1]].concat());
        assert_eq!(request_write_hero_words("w"), [OP_EXTENDED_CLIENT, 12, 0, b'w', 0, 0, 0]);
        assert_eq!(request_ex_ask_join_mpcc("L"), [OP_EXTENDED_CLIENT, 13, 0, b'L', 0, 0, 0]);
        assert_eq!(request_ex_accept_join_mpcc(true), [OP_EXTENDED_CLIENT, 14, 0, 1, 0, 0, 0]);
        assert_eq!(request_ex_oust_from_mpcc("L"), [OP_EXTENDED_CLIENT, 15, 0, b'L', 0, 0, 0]);
        assert_eq!(request_ex_pledge_crest_large(5), [OP_EXTENDED_CLIENT, 16, 0, 5, 0, 0, 0]);
        assert_eq!(request_ex_set_pledge_crest_large(&[9]), [OP_EXTENDED_CLIENT, 17, 0, 1, 0, 0, 0, 9]);
    }
}
