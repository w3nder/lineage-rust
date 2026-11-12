use super::{InventoryItem, counted, read_d_or_zero, read_h_or_zero, read_inventory_item};
use crate::wire::{Reader, Result, WireError, Writer};

pub const OP_SELL_LIST: u8 = 0x10;
pub const OP_BUY_LIST: u8 = 0x11;
pub const OP_NEW_CHARACTER_FAIL: u8 = 0x18;
pub const OP_ATTACK_OUT_OF_RANGE: u8 = 0x08;
pub const OP_ATTACK_IN_COOL_TIME: u8 = 0x09;
pub const OP_ATTACK_DEAD_TARGET: u8 = 0x0a;
pub const OP_VALIDATE_LOCATION_IN_VEHICLE: u8 = 0x73;
pub const OP_GM_HIDE: u8 = 0x8d;
pub const OP_DUMMY: u8 = 0x70;
pub const OP_TRADE_START: u8 = 0x1e;
pub const OP_TRADE_START_OK: u8 = 0x1f;
pub const OP_TRADE_OWN_ADD: u8 = 0x20;
pub const OP_TRADE_OTHER_ADD: u8 = 0x21;
pub const OP_TRADE_DONE: u8 = 0x22;
pub const OP_START_PLEDGE_WAR: u8 = 0x65;
pub const OP_REPLY_START_PLEDGE_WAR: u8 = 0x66;
pub const OP_STOP_PLEDGE_WAR: u8 = 0x67;
pub const OP_REPLY_STOP_PLEDGE_WAR: u8 = 0x68;
pub const OP_SURRENDER_PLEDGE_WAR: u8 = 0x69;
pub const OP_REPLY_SURRENDER_PLEDGE_WAR: u8 = 0x6a;
pub const OP_SET_PLEDGE_CREST: u8 = 0x6b;
pub const OP_TRADE_PRESS_OWN_OK: u8 = 0x75;
pub const OP_TRADE_PRESS_OTHER_OK: u8 = 0x7c;
pub const OP_MAGIC_LIST: u8 = 0x57;
pub const OP_MAGIC_AND_SKILL_LIST: u8 = 0x40;
pub const OP_MANAGE_PLEDGE_POWER: u8 = 0x30;
pub const OP_CREATE_PLEDGE: u8 = 0x31;
pub const OP_JOIN_PLEDGE: u8 = 0x33;
pub const OP_WITHDRAWAL_PLEDGE: u8 = 0x34;
pub const OP_OUST_PLEDGE_MEMBER: u8 = 0x35;
pub const OP_SET_OUST_PLEDGE_MEMBER: u8 = 0x36;
pub const OP_DISMISS_PLEDGE: u8 = 0x37;
pub const OP_SET_DISMISS_PLEDGE: u8 = 0x38;
pub const OP_JOIN_PARTY: u8 = 0x3a;
pub const OP_WITHDRAWAL_PARTY: u8 = 0x3b;
pub const OP_OUST_PARTY_MEMBER: u8 = 0x3c;
pub const OP_SET_OUST_PARTY_MEMBER: u8 = 0x3d;
pub const OP_DISMISS_PARTY: u8 = 0x3e;
pub const OP_SET_DISMISS_PARTY: u8 = 0x3f;
pub const OP_WAREHOUSE_DEPOSIT_LIST: u8 = 0x41;
pub const OP_WAREHOUSE_WITHDRAW_LIST: u8 = 0x42;
pub const OP_WAREHOUSE_DONE: u8 = 0x43;
pub const OP_PLEDGE_SHOW_MEMBER_LIST_ALL: u8 = 0x53;
pub const OP_PLEDGE_SHOW_MEMBER_LIST_UPDATE: u8 = 0x54;
pub const OP_PLEDGE_SHOW_MEMBER_LIST_ADD: u8 = 0x55;
pub const OP_PLEDGE_SHOW_MEMBER_LIST_DELETE: u8 = 0x56;
pub const OP_TRADE_REQUEST: u8 = 0x5e;
pub const OP_VEHICLE_CHECK_LOCATION: u8 = 0x5b;
pub const OP_PLEDGE_CREST: u8 = 0x6c;
pub const OP_SHOW_BOARD: u8 = 0x6e;
pub const OP_CHOOSE_INVENTORY_ITEM: u8 = 0x6f;
pub const OP_TRADE_UPDATE: u8 = 0x74;
pub const OP_FRIEND_ADD_REQUEST: u8 = 0x7d;
pub const OP_FRIEND_ADD_REQUEST_RESULT: u8 = 0x77;
pub const OP_FRIEND_ADD: u8 = 0x78;
pub const OP_FRIEND_REMOVE: u8 = 0x79;
pub const OP_FRIEND_LIST_LEGACY: u8 = 0x7a;
pub const OP_FRIEND_STATUS: u8 = 0x7b;
pub const OP_REVIVE_REQUEST: u8 = 0x9e;
pub const OP_QUEST_LIST: u8 = 0x80;
pub const OP_ENCHANT_RESULT: u8 = 0x81;
pub const OP_PLEDGE_SHOW_MEMBER_LIST_DELETE_ALL: u8 = 0x82;
pub const OP_PLEDGE_INFO: u8 = 0x83;
pub const OP_PLEDGE_EXTENDED_INFO: u8 = 0x84;
pub const OP_SURRENDER_PERSONALLY: u8 = 0x85;
pub const OP_CLIENT_ACTION: u8 = 0x89;
pub const OP_ABNORMAL_VISUAL_EFFECT: u8 = 0x9f;
pub const OP_PLEDGE_SHOW_INFO_UPDATE: u8 = 0x88;
pub const OP_ACQUIRE_SKILL_LIST: u8 = 0x8a;
pub const OP_ACQUIRE_SKILL_INFO: u8 = 0x8b;
pub const OP_ACQUIRE_SKILL_DONE: u8 = 0x8e;
pub const OP_PARTY_MATCH_LIST: u8 = 0x96;
pub const OP_PARTY_MATCH_DETAIL: u8 = 0x97;
pub const OP_PRIVATE_STORE_MANAGE_LIST_SELL: u8 = 0x9a;
pub const OP_PRIVATE_STORE_LIST_SELL: u8 = 0x9b;
pub const OP_PRIVATE_STORE_MSG_SELL: u8 = 0x9c;
pub const OP_SHOW_MINIMAP: u8 = 0x9d;
pub const OP_TUTORIAL_SHOW_HTML: u8 = 0xa0;
pub const OP_TUTORIAL_SHOW_QUESTION_MARK: u8 = 0xa1;
pub const OP_TUTORIAL_ENABLE_CLIENT_EVENT: u8 = 0xa2;
pub const OP_TUTORIAL_CLOSE_HTML: u8 = 0xa3;
pub const OP_ALLIANCE_CREST: u8 = 0xae;
pub const OP_JOIN_ALLIANCE: u8 = 0xa9;
pub const OP_WITHDRAW_ALLIANCE: u8 = 0xaa;
pub const OP_OUST_ALLIANCE_MEMBER_PLEDGE: u8 = 0xab;
pub const OP_DISMISS_ALLIANCE: u8 = 0xac;
pub const OP_SET_ALLIANCE_CREST: u8 = 0xad;
pub const OP_ALLIANCE_INFO: u8 = 0xb4;
pub const OP_REQUEST_TIME_CHECK: u8 = 0xbb;
pub const OP_SERVER_CLOSE_SOCKET: u8 = 0xaf;
pub const OP_REPLY_START_ALLIANCE_WAR: u8 = 0xbd;
pub const OP_START_ALLIANCE_WAR: u8 = 0xbc;
pub const OP_STOP_ALLIANCE_WAR: u8 = 0xbe;
pub const OP_REPLY_STOP_ALLIANCE_WAR: u8 = 0xbf;
pub const OP_SURRENDER_ALLIANCE_WAR: u8 = 0xc0;
pub const OP_OBSERVER_START: u8 = 0xdf;
pub const OP_OBSERVER_END: u8 = 0xe0;
pub const OP_NORMAL_CAMERA: u8 = 0xc8;
pub const OP_EVENT_TRIGGER: u8 = 0xcf;
pub const OP_NET_PING_RESPONSE: u8 = 0xd3;
pub const OP_SNOOP: u8 = 0xd5;
pub const OP_CAMERA_MODE: u8 = 0xf1;
pub const OP_PETITION_VOTE: u8 = 0xf6;
pub const OP_PET_ITEM_LIST: u8 = 0xb2;
pub const OP_PET_INVENTORY_UPDATE: u8 = 0xb3;
pub const OP_PRIVATE_STORE_MANAGE_LIST_BUY: u8 = 0xb7;
pub const OP_PRIVATE_STORE_LIST_BUY: u8 = 0xb8;
pub const OP_PRIVATE_STORE_MSG_BUY: u8 = 0xb9;
pub const OP_SHOW_RADAR: u8 = 0xa4;
pub const OP_DELETE_RADAR: u8 = 0xa5;
pub const OP_PACKAGE_TO_LIST: u8 = 0xc2;
pub const OP_PACKAGE_SENDABLE_LIST: u8 = 0xc3;
pub const OP_MULTI_SELL_LIST: u8 = 0xd0;
pub const OP_SKILL_REMAIN_SEC: u8 = 0xd2;
pub const OP_SHOW_TOWN_MAP: u8 = 0xde;
pub const OP_HENNA_EQUIP_LIST: u8 = 0xe2;
pub const OP_HENNA_ITEM_INFO: u8 = 0xe3;
pub const OP_HENNA_INFO: u8 = 0xe4;
pub const OP_HENNA_UNEQUIP_LIST: u8 = 0xe5;
pub const OP_HENNA_UNEQUIP_INFO: u8 = 0xe6;
pub const OP_GM_VIEW_SKILL_INFO: u8 = 0x91;
pub const OP_GM_VIEW_MAGIC_INFO: u8 = 0x92;
pub const OP_GM_VIEW_QUEST_INFO: u8 = 0x93;
pub const OP_GM_VIEW_ITEM_LIST: u8 = 0x94;
pub const OP_GM_VIEW_WAREHOUSE_WITHDRAW_LIST: u8 = 0x95;
pub const OP_BLOCK_LIST: u8 = 0xc6;
pub const OP_SELL_LIST_PROCURE: u8 = 0xe9;
pub const OP_GM_HENNA_INFO: u8 = 0xea;
pub const OP_BUY_LIST_SEED: u8 = 0xe8;
pub const OP_RADAR_CONTROL: u8 = 0xeb;
pub const OP_SHOP_PREVIEW_LIST: u8 = 0xef;
pub const OP_SHOP_PREVIEW_INFO: u8 = 0xf0;
pub const OP_FRIEND_LIST: u8 = 0xfa;
pub const OP_L2_FRIEND: u8 = 0xfb;
pub const OP_L2_FRIEND_STATUS: u8 = 0xfc;
pub const OP_GM_VIEW_PLEDGE_INFO: u8 = 0x90;
pub const OP_GM_VIEW_CHARACTER_INFO: u8 = 0x8f;
pub const OP_FRIEND_RECV_MSG: u8 = 0xfd;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradePressOk;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradeStartOk;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PledgeWarNoop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetPledgeCrestNoop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurrenderPersonallyNoop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerCloseSocketNoop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllianceWarNoop;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllianceWar {
    pub first: String,
    pub second: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MagicAndSkillList {
    pub first: i32,
    pub second: i32,
    pub third: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillRemainSec {
    pub values: [i32; 7],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VehicleCheckLocation {
    pub values: [i32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DummyNoop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockListNoop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbnormalVisualEffectNoop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PledgeIdResult {
    pub pledge_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartyMemberResult {
    pub member_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PledgePartyConfirmationNoop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackFeedbackNoop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidateLocationInVehicle {
    pub values: [i32; 6],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GmHide {
    pub object_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllianceIdResult {
    pub alliance_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllianceNoop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestTimeCheck {
    pub value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverStart {
    pub values: [i32; 5],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverEnd {
    pub values: [i32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalCameraNoop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventTrigger {
    pub event_id: i32,
    pub on: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetPingResponse {
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snoop {
    pub first: i32,
    pub first_text: String,
    pub second: i32,
    pub third: i32,
    pub second_text: String,
    pub third_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraMode {
    pub mode: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PetitionVoteNoop;

fn read_opcode_only(body: &[u8]) -> Result<()> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(())
}

pub fn read_start_pledge_war(body: &[u8]) -> Result<PledgeWarNoop> {
    read_opcode_only(body)?;
    Ok(PledgeWarNoop)
}

pub fn read_reply_start_pledge_war(body: &[u8]) -> Result<PledgeWarNoop> {
    read_start_pledge_war(body)
}

pub fn read_stop_pledge_war(body: &[u8]) -> Result<PledgeWarNoop> {
    read_start_pledge_war(body)
}

pub fn read_reply_stop_pledge_war(body: &[u8]) -> Result<PledgeWarNoop> {
    read_start_pledge_war(body)
}

pub fn read_surrender_pledge_war(body: &[u8]) -> Result<PledgeWarNoop> {
    read_start_pledge_war(body)
}

pub fn read_reply_surrender_pledge_war(body: &[u8]) -> Result<PledgeWarNoop> {
    read_start_pledge_war(body)
}

pub fn read_set_pledge_crest(body: &[u8]) -> Result<SetPledgeCrestNoop> {
    read_opcode_only(body)?;
    Ok(SetPledgeCrestNoop)
}

pub fn read_surrender_personally(body: &[u8]) -> Result<SurrenderPersonallyNoop> {
    read_opcode_only(body)?;
    Ok(SurrenderPersonallyNoop)
}

pub fn read_server_close_socket(body: &[u8]) -> Result<ServerCloseSocketNoop> {
    read_opcode_only(body)?;
    Ok(ServerCloseSocketNoop)
}

pub fn read_reply_start_alliance_war(body: &[u8]) -> Result<AllianceWarNoop> {
    read_opcode_only(body)?;
    Ok(AllianceWarNoop)
}

pub fn read_start_alliance_war(body: &[u8]) -> Result<AllianceWar> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let first = r.read_s()?;
    let second = r.read_s()?;
    Ok(AllianceWar { first, second })
}

pub fn read_stop_alliance_war(body: &[u8]) -> Result<AllianceWar> {
    read_start_alliance_war(body)
}

pub fn read_reply_stop_alliance_war(body: &[u8]) -> Result<AllianceWarNoop> {
    read_reply_start_alliance_war(body)
}

pub fn read_surrender_alliance_war(body: &[u8]) -> Result<AllianceWarNoop> {
    read_reply_start_alliance_war(body)
}

pub fn read_magic_and_skill_list(body: &[u8]) -> Result<MagicAndSkillList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let first = r.read_d()?;
    let second = r.read_d()?;
    let third = r.read_d()?;
    Ok(MagicAndSkillList { first, second, third })
}

pub fn read_skill_remain_sec(body: &[u8]) -> Result<SkillRemainSec> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let mut values = [0; 7];
    for value in &mut values {
        *value = r.read_d()?;
    }
    Ok(SkillRemainSec { values })
}

pub fn read_vehicle_check_location(body: &[u8]) -> Result<VehicleCheckLocation> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let mut values = [0; 4];
    for value in &mut values {
        *value = r.read_d()?;
    }
    Ok(VehicleCheckLocation { values })
}

pub fn read_dummy(body: &[u8]) -> Result<DummyNoop> {
    read_opcode_only(body)?;
    Ok(DummyNoop)
}

pub fn read_block_list(body: &[u8]) -> Result<BlockListNoop> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(BlockListNoop)
}

pub fn read_abnormal_visual_effect(body: &[u8]) -> Result<AbnormalVisualEffectNoop> {
    read_opcode_only(body)?;
    Ok(AbnormalVisualEffectNoop)
}

fn read_pledge_id_result(body: &[u8]) -> Result<PledgeIdResult> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let pledge_id = r.read_d()?;
    Ok(PledgeIdResult { pledge_id })
}

pub fn read_create_pledge(body: &[u8]) -> Result<PledgeIdResult> {
    read_pledge_id_result(body)
}

pub fn read_withdrawal_pledge(body: &[u8]) -> Result<PledgeIdResult> {
    read_pledge_id_result(body)
}

pub fn read_oust_pledge_member(body: &[u8]) -> Result<PledgeIdResult> {
    read_pledge_id_result(body)
}

pub fn read_dismiss_pledge(body: &[u8]) -> Result<PledgeIdResult> {
    read_pledge_id_result(body)
}

fn read_party_member_result(body: &[u8]) -> Result<PartyMemberResult> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let member_id = r.read_d()?;
    Ok(PartyMemberResult { member_id })
}

pub fn read_withdrawal_party(body: &[u8]) -> Result<PartyMemberResult> {
    read_party_member_result(body)
}

pub fn read_oust_party_member(body: &[u8]) -> Result<PartyMemberResult> {
    read_party_member_result(body)
}

pub fn read_dismiss_party(body: &[u8]) -> Result<PartyMemberResult> {
    read_party_member_result(body)
}

fn read_pledge_party_confirmation(body: &[u8]) -> Result<PledgePartyConfirmationNoop> {
    read_opcode_only(body)?;
    Ok(PledgePartyConfirmationNoop)
}

pub fn read_set_oust_pledge_member(body: &[u8]) -> Result<PledgePartyConfirmationNoop> {
    read_pledge_party_confirmation(body)
}

pub fn read_set_dismiss_pledge(body: &[u8]) -> Result<PledgePartyConfirmationNoop> {
    read_pledge_party_confirmation(body)
}

pub fn read_set_oust_party_member(body: &[u8]) -> Result<PledgePartyConfirmationNoop> {
    read_pledge_party_confirmation(body)
}

pub fn read_set_dismiss_party(body: &[u8]) -> Result<PledgePartyConfirmationNoop> {
    read_pledge_party_confirmation(body)
}

pub fn read_attack_out_of_range(body: &[u8]) -> Result<AttackFeedbackNoop> {
    read_opcode_only(body)?;
    Ok(AttackFeedbackNoop)
}

pub fn read_attack_in_cool_time(body: &[u8]) -> Result<AttackFeedbackNoop> {
    read_attack_out_of_range(body)
}

pub fn read_attack_dead_target(body: &[u8]) -> Result<AttackFeedbackNoop> {
    read_attack_out_of_range(body)
}

pub fn read_validate_location_in_vehicle(body: &[u8]) -> Result<ValidateLocationInVehicle> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let mut values = [0; 6];
    for value in &mut values {
        *value = r.read_d()?;
    }
    Ok(ValidateLocationInVehicle { values })
}

pub fn read_gm_hide(body: &[u8]) -> Result<GmHide> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    Ok(GmHide { object_id })
}

fn read_alliance_id_result(body: &[u8]) -> Result<AllianceIdResult> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let alliance_id = r.read_d()?;
    Ok(AllianceIdResult { alliance_id })
}

pub fn read_join_alliance(body: &[u8]) -> Result<AllianceIdResult> {
    read_alliance_id_result(body)
}

pub fn read_withdraw_alliance(body: &[u8]) -> Result<AllianceIdResult> {
    read_alliance_id_result(body)
}

pub fn read_oust_alliance_member_pledge(body: &[u8]) -> Result<AllianceIdResult> {
    read_alliance_id_result(body)
}

pub fn read_dismiss_alliance(body: &[u8]) -> Result<AllianceIdResult> {
    read_alliance_id_result(body)
}

pub fn read_set_alliance_crest(body: &[u8]) -> Result<AllianceNoop> {
    read_opcode_only(body)?;
    Ok(AllianceNoop)
}

pub fn read_alliance_info(body: &[u8]) -> Result<AllianceNoop> {
    read_set_alliance_crest(body)
}

pub fn read_request_time_check(body: &[u8]) -> Result<RequestTimeCheck> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let value = r.read_d()?;
    Ok(RequestTimeCheck { value })
}

pub fn read_observer_start(body: &[u8]) -> Result<ObserverStart> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let mut values = [0; 5];
    for value in &mut values {
        *value = r.read_d()?;
    }
    Ok(ObserverStart { values })
}

pub fn read_observer_end(body: &[u8]) -> Result<ObserverEnd> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let mut values = [0; 3];
    for value in &mut values {
        *value = r.read_d()?;
    }
    Ok(ObserverEnd { values })
}

pub fn read_normal_camera(body: &[u8]) -> Result<NormalCameraNoop> {
    read_opcode_only(body)?;
    Ok(NormalCameraNoop)
}

pub fn read_event_trigger(body: &[u8]) -> Result<EventTrigger> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let event_id = r.read_d()?;
    let on = r.read_c()?;
    Ok(EventTrigger { event_id, on })
}

pub fn read_net_ping_response(body: &[u8]) -> Result<NetPingResponse> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let value = r.read_d()?;
    Ok(NetPingResponse { value })
}

pub fn read_snoop(body: &[u8]) -> Result<Snoop> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let first = r.read_d()?;
    let first_text = r.read_s()?;
    let second = r.read_d()?;
    let third = r.read_d()?;
    let second_text = r.read_s()?;
    let third_text = r.read_s()?;
    Ok(Snoop { first, first_text, second, third, second_text, third_text })
}

pub fn read_camera_mode(body: &[u8]) -> Result<CameraMode> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let mode = r.read_d()?;
    Ok(CameraMode { mode })
}

pub fn read_petition_vote(body: &[u8]) -> Result<PetitionVoteNoop> {
    read_opcode_only(body)?;
    Ok(PetitionVoteNoop)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FriendNoop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FriendAddRequestResult {
    pub result: i32,
}

pub fn read_friend_add_request_result(body: &[u8]) -> Result<FriendAddRequestResult> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let result = r.read_d()?;
    Ok(FriendAddRequestResult { result })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MagicListNoop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewCharacterFailNoop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReviveRequestNoop;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PledgeExtendedInfo {
    pub leading_text: String,
    pub leading_values: [i32; 2],
    pub middle_text: String,
    pub middle_values: [i32; 8],
    pub trailing_text: String,
    pub trailing_value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientAction {
    pub action: i32,
}

pub fn read_client_action(body: &[u8]) -> Result<ClientAction> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let action = r.read_d()?;
    Ok(ClientAction { action })
}

pub fn read_pledge_extended_info(body: &[u8]) -> Result<PledgeExtendedInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let leading_text = r.read_s()?;
    let leading_values = [r.read_d()?, r.read_d()?];
    let middle_text = r.read_s()?;
    let mut middle_values = [0; 8];
    for value in &mut middle_values {
        *value = r.read_d()?;
    }
    let trailing_text = r.read_s()?;
    let trailing_value = r.read_d()?;
    Ok(PledgeExtendedInfo { leading_text, leading_values, middle_text, middle_values, trailing_text, trailing_value })
}

pub fn read_new_character_fail(body: &[u8]) -> Result<NewCharacterFailNoop> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(NewCharacterFailNoop)
}

pub fn read_revive_request(body: &[u8]) -> Result<ReviveRequestNoop> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(ReviveRequestNoop)
}

pub fn read_friend_add(body: &[u8]) -> Result<FriendNoop> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(FriendNoop)
}

pub fn read_friend_remove(body: &[u8]) -> Result<FriendNoop> {
    read_friend_add(body)
}

pub fn read_friend_list_noop(body: &[u8]) -> Result<FriendNoop> {
    read_friend_add(body)
}

pub fn read_friend_status_noop(body: &[u8]) -> Result<FriendNoop> {
    read_friend_add(body)
}

pub fn read_magic_list(body: &[u8]) -> Result<MagicListNoop> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(MagicListNoop)
}

fn read_trade_press_ok(body: &[u8]) -> Result<TradePressOk> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(TradePressOk)
}

pub fn read_trade_press_own_ok(body: &[u8]) -> Result<TradePressOk> {
    read_trade_press_ok(body)
}

pub fn read_trade_press_other_ok(body: &[u8]) -> Result<TradePressOk> {
    read_trade_press_ok(body)
}

pub fn read_trade_start_ok(body: &[u8]) -> Result<TradeStartOk> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(TradeStartOk)
}

pub const OP_REQUEST_TRADE: u8 = 0x15;
pub const OP_REQUEST_ADD_TRADE_ITEM: u8 = 0x16;
pub const OP_REQUEST_TRADE_DONE: u8 = 0x17;
pub const OP_REQUEST_SELL_ITEM: u8 = 0x1e;
pub const OP_REQUEST_BUY_ITEM: u8 = 0x1f;
pub const OP_REQUEST_JOIN_PLEDGE: u8 = 0x24;
pub const OP_REQUEST_WITHDRAWAL_PLEDGE: u8 = 0x26;
pub const OP_REQUEST_OUST_PLEDGE_MEMBER: u8 = 0x27;
pub const OP_SEND_WAREHOUSE_DEPOSIT_LIST: u8 = 0x31;
pub const OP_SEND_WAREHOUSE_WITHDRAW_LIST: u8 = 0x32;
pub const OP_REQUEST_PLEDGE_MEMBER_LIST: u8 = 0x3c;
pub const OP_REQUEST_SET_PLEDGE_CREST: u8 = 0x53;
pub const OP_REQUEST_ENCHANT_ITEM: u8 = 0x58;
pub const OP_REQUEST_QUEST_ABORT: u8 = 0x64;
pub const OP_REQUEST_PLEDGE_CREST: u8 = 0x68;
pub const OP_REQUEST_RESTART_POINT: u8 = 0x6d;
pub const OP_REQUEST_CRYSTALLIZE_ITEM: u8 = 0x72;
pub const OP_REQUEST_PRIVATE_STORE_MANAGE_SELL: u8 = 0x73;
pub const OP_SET_PRIVATE_STORE_LIST_SELL: u8 = 0x74;
pub const OP_REQUEST_PRIVATE_STORE_MANAGE_CANCEL_SELL: u8 = 0x75;
pub const OP_REQUEST_PRIVATE_STORE_QUIT_SELL: u8 = 0x76;
pub const OP_SET_PRIVATE_STORE_MSG_SELL: u8 = 0x77;
pub const OP_REQUEST_PRIVATE_STORE_LIST_SELL: u8 = 0x78;
pub const OP_REQUEST_PRIVATE_STORE_BUY: u8 = 0x79;
pub const OP_REQUEST_TUTORIAL_LINK_HTML: u8 = 0x7b;
pub const OP_REQUEST_TUTORIAL_PASS_CMD: u8 = 0x7c;
pub const OP_REQUEST_TUTORIAL_QUESTION_MARK: u8 = 0x7d;
pub const OP_REQUEST_TUTORIAL_CLIENT_EVENT: u8 = 0x7e;
pub const OP_REQUEST_JOIN_ALLY: u8 = 0x82;
pub const OP_REQUEST_ALLY_LEAVE: u8 = 0x84;
pub const OP_REQUEST_ALLY_DISMISS: u8 = 0x85;
pub const OP_REQUEST_DISMISS_ALLY: u8 = 0x86;
pub const OP_REQUEST_SET_ALLY_CREST: u8 = 0x87;
pub const OP_REQUEST_ALLY_CREST: u8 = 0x88;
pub const OP_REQUEST_ALLY_INFO: u8 = 0x8e;
pub const OP_REQUEST_PET_GET_ITEM: u8 = 0x8f;
pub const OP_REQUEST_PRIVATE_STORE_MANAGE_BUY: u8 = 0x90;
pub const OP_SET_PRIVATE_STORE_LIST_BUY: u8 = 0x91;
pub const OP_REQUEST_PRIVATE_STORE_MANAGE_CANCEL_BUY: u8 = 0x92;
pub const OP_REQUEST_PRIVATE_STORE_QUIT_BUY: u8 = 0x93;
pub const OP_SET_PRIVATE_STORE_MSG_BUY: u8 = 0x94;
pub const OP_REQUEST_PRIVATE_STORE_LIST_BUY: u8 = 0x95;
pub const OP_REQUEST_PRIVATE_STORE_SELL: u8 = 0x96;
pub const OP_REQUEST_PACKAGE_SENDABLE_ITEM_LIST: u8 = 0x9e;
pub const OP_REQUEST_PACKAGE_SEND: u8 = 0x9f;
pub const OP_MULTI_SELL_CHOOSE: u8 = 0xa7;
pub const OP_REQUEST_USER_COMMAND: u8 = 0xaa;
pub const OP_REQUEST_HENNA_LIST: u8 = 0xba;
pub const OP_REQUEST_HENNA_ITEM_INFO: u8 = 0xbb;
pub const OP_REQUEST_HENNA_EQUIP: u8 = 0xbc;
pub const OP_REQUEST_HENNA_UNEQUIP_LIST: u8 = 0xbd;
pub const OP_REQUEST_HENNA_UNEQUIP_INFO: u8 = 0xbe;
pub const OP_REQUEST_HENNA_UNEQUIP: u8 = 0xbf;
pub const OP_REQUEST_PLEDGE_POWER: u8 = 0xc0;
pub const OP_REQUEST_MAKE_MACRO: u8 = 0xc1;
pub const OP_REQUEST_DELETE_MACRO: u8 = 0xc2;
pub const OP_REQUEST_PROCURE_CROP: u8 = 0xc3;
pub const OP_REQUEST_PREVIEW_ITEM: u8 = 0xc6;
pub const OP_REQUEST_SEND_FRIEND_MSG: u8 = 0xcc;
pub const OP_REQUEST_SHOW_MINIMAP: u8 = 0xcd;
pub const OP_REQUEST_FRIEND_INVITE: u8 = 0x5e;
pub const OP_REQUEST_ANSWER_FRIEND_INVITE: u8 = 0x5f;
pub const OP_REQUEST_FRIEND_LIST: u8 = 0x60;
pub const OP_REQUEST_FRIEND_DEL: u8 = 0x61;
pub const OP_REQUEST_GIVE_NICK_NAME: u8 = 0x55;
pub const OP_REQUEST_SHOW_BOARD: u8 = 0x57;
pub const OP_REQUEST_BBS_WRITE: u8 = 0x22;
pub const OP_REQUEST_PARTY_MATCH_LIST: u8 = 0x6f;
pub const OP_REQUEST_MANAGE_PARTY_ROOM: u8 = 0x70;
pub const OP_REQUEST_JOIN_PARTY_ROOM: u8 = 0x71;

pub const BOARD_STRINGS: usize = 9;
pub const BOARD_LINK_MAX_CHARS: usize = 0x100 / 2 - 1;
pub const BOARD_HTML_MAX_CHARS: usize = 0x4000 / 2 - 1;
pub const TUTORIAL_HTML_MAX_CHARS: usize = 0x4000 / 2 - 1;
pub const SHOP_PREVIEW_SLOTS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradeItem {
    pub type1: i16,
    pub object_id: i32,
    pub item_id: i32,
    pub count: i32,
    pub type2: i16,
    pub custom_type1: i16,
    pub body_part: i32,
    pub enchant: i16,
    pub custom_type2: i16,
}

fn read_trade_item_body(r: &mut Reader, type1: i16) -> Result<TradeItem> {
    let object_id = r.read_d()?;
    let item_id = r.read_d()?;
    let count = r.read_d()?;
    let type2 = r.read_h()? as i16;
    let custom_type1 = r.read_h()? as i16;
    let body_part = r.read_d()?;
    let enchant = r.read_h()? as i16;
    r.read_h()?;
    let custom_type2 = r.read_h()? as i16;
    Ok(TradeItem { type1, object_id, item_id, count, type2, custom_type1, body_part, enchant, custom_type2 })
}

fn read_trade_item(r: &mut Reader) -> Result<TradeItem> {
    let type1 = r.read_h()? as i16;
    read_trade_item_body(r, type1)
}

const TRADE_ITEM_SIZE: usize = 2 + 4 + 4 + 4 + 2 + 2 + 4 + 2 + 2 + 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PricedItem {
    pub item: TradeItem,
    pub price: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopList {
    pub money: i32,
    pub list_id: i32,
    pub items: Vec<PricedItem>,
}

fn read_shop_list(body: &[u8]) -> Result<ShopList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let money = r.read_d()?;
    let list_id = r.read_d()?;
    let declared = r.read_h()?;
    let count = counted(&r, declared as i32, TRADE_ITEM_SIZE + 4)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let item = read_trade_item(&mut r)?;
        let price = r.read_d()?;
        items.push(PricedItem { item, price });
    }
    Ok(ShopList { money, list_id, items })
}

pub fn read_sell_list(body: &[u8]) -> Result<ShopList> {
    read_shop_list(body)
}

pub fn read_buy_list(body: &[u8]) -> Result<ShopList> {
    read_shop_list(body)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeStart {
    pub partner_id: i32,
    pub items: Vec<TradeItem>,
}

pub fn read_trade_start(body: &[u8]) -> Result<TradeStart> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let partner_id = r.read_d()?;
    let declared = r.read_h()?;
    let count = counted(&r, declared as i32, TRADE_ITEM_SIZE)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        items.push(read_trade_item(&mut r)?);
    }
    Ok(TradeStart { partner_id, items })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeAdd {
    pub items: Vec<TradeItem>,
}

fn read_trade_add(body: &[u8]) -> Result<TradeAdd> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let declared = r.read_h()?;
    let count = counted(&r, declared as i32, TRADE_ITEM_SIZE)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        items.push(read_trade_item(&mut r)?);
    }
    Ok(TradeAdd { items })
}

pub fn read_trade_own_add(body: &[u8]) -> Result<TradeAdd> {
    read_trade_add(body)
}

pub fn read_trade_other_add(body: &[u8]) -> Result<TradeAdd> {
    read_trade_add(body)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradeDone {
    pub success: bool,
}

pub fn read_trade_done(body: &[u8]) -> Result<TradeDone> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let success = r.read_d()? != 0;
    Ok(TradeDone { success })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradeUpdateItem {
    pub update_type: i16,
    pub item: TradeItem,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradeUpdate {
    pub items: Vec<TradeUpdateItem>,
}

pub fn read_trade_update(body: &[u8]) -> Result<TradeUpdate> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let declared = r.read_h()?;
    let count = counted(&r, declared as i32, TRADE_ITEM_SIZE + 2)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let update_type = r.read_h()? as i16;
        let type1 = r.read_h()? as i16;
        let item = read_trade_item_body(&mut r, type1)?;
        items.push(TradeUpdateItem { update_type, item });
    }
    Ok(TradeUpdate { items })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradeRequest {
    pub sender_id: i32,
}

pub fn read_trade_request(body: &[u8]) -> Result<TradeRequest> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let sender_id = r.read_d()?;
    Ok(TradeRequest { sender_id })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManagePledgePower {
    pub first: i32,
    pub second: i32,
    pub powers: [u8; PLEDGE_POWER_BYTES],
}

pub const PLEDGE_POWER_BYTES: usize = 0x20;

pub fn read_manage_pledge_power(body: &[u8]) -> Result<ManagePledgePower> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let first = r.read_d()?;
    let second = r.read_d()?;
    let mut powers = [0u8; PLEDGE_POWER_BYTES];
    let tail = r.read_b(r.remaining().min(PLEDGE_POWER_BYTES))?;
    powers[..tail.len()].copy_from_slice(tail);
    Ok(ManagePledgePower { first, second, powers })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JoinPledge {
    pub pledge_id: i32,
}

pub fn read_join_pledge(body: &[u8]) -> Result<JoinPledge> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let pledge_id = r.read_d()?;
    Ok(JoinPledge { pledge_id })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JoinParty {
    pub response: i32,
}

pub fn read_join_party(body: &[u8]) -> Result<JoinParty> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let response = r.read_d()?;
    Ok(JoinParty { response })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarehouseItem {
    pub item: TradeItem,
    pub warehouse_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarehouseList {
    pub warehouse_type: i16,
    pub adena: i32,
    pub items: Vec<WarehouseItem>,
}

fn read_warehouse_list(body: &[u8]) -> Result<WarehouseList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let warehouse_type = r.read_h()? as i16;
    let adena = r.read_d()?;
    let declared = r.read_h()?;
    let count = counted(&r, declared as i32, TRADE_ITEM_SIZE + 4)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let item = read_trade_item(&mut r)?;
        let warehouse_id = r.read_d()?;
        items.push(WarehouseItem { item, warehouse_id });
    }
    Ok(WarehouseList { warehouse_type, adena, items })
}

pub fn read_warehouse_deposit_list(body: &[u8]) -> Result<WarehouseList> {
    read_warehouse_list(body)
}

pub fn read_warehouse_withdraw_list(body: &[u8]) -> Result<WarehouseList> {
    read_warehouse_list(body)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarehouseDone {
    pub success: bool,
}

pub fn read_warehouse_done(body: &[u8]) -> Result<WarehouseDone> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let success = r.read_d()? != 0;
    Ok(WarehouseDone { success })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PledgeMember {
    pub name: String,
    pub level: i32,
    pub class_id: i32,
    pub unknown: i32,
    pub flag: i32,
    pub online_object_id: i32,
}

fn read_pledge_member(r: &mut Reader) -> Result<PledgeMember> {
    let name = r.read_s()?;
    let level = r.read_d()?;
    let class_id = r.read_d()?;
    let unknown = r.read_d()?;
    let flag = r.read_d()?;
    let online_object_id = r.read_d()?;
    Ok(PledgeMember { name, level, class_id, unknown, flag, online_object_id })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PledgeShowMemberListAll {
    pub pledge_id: i32,
    pub name: String,
    pub leader_name: String,
    pub crest_id: i32,
    pub level: i32,
    pub castle_id: i32,
    pub hideout_id: i32,
    pub my_level: i32,
    pub dissolving: i32,
    pub ally_id: i32,
    pub ally_name: String,
    pub ally_crest_id: i32,
    pub at_war: bool,
    pub members: Vec<PledgeMember>,
}

pub fn read_pledge_show_member_list_all(body: &[u8]) -> Result<PledgeShowMemberListAll> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let pledge_id = r.read_d()?;
    let name = r.read_s()?;
    let leader_name = r.read_s()?;
    let crest_id = r.read_d()?;
    let level = r.read_d()?;
    let castle_id = r.read_d()?;
    let hideout_id = r.read_d()?;
    r.read_d()?;
    let my_level = r.read_d()?;
    let dissolving = r.read_d()?;
    r.read_d()?;
    let ally_id = r.read_d()?;
    let ally_name = r.read_s()?;
    let ally_crest_id = r.read_d()?;
    let at_war = r.read_d()? != 0;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 2 + 5 * 4)?;
    let mut members = Vec::with_capacity(count);
    for _ in 0..count {
        members.push(read_pledge_member(&mut r)?);
    }
    Ok(PledgeShowMemberListAll {
        pledge_id,
        name,
        leader_name,
        crest_id,
        level,
        castle_id,
        hideout_id,
        my_level,
        dissolving,
        ally_id,
        ally_name,
        ally_crest_id,
        at_war,
        members,
    })
}

fn read_single_pledge_member(body: &[u8]) -> Result<PledgeMember> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let member = read_pledge_member(&mut r)?;
    Ok(member)
}

pub fn read_pledge_show_member_list_update(body: &[u8]) -> Result<PledgeMember> {
    read_single_pledge_member(body)
}

pub fn read_pledge_show_member_list_add(body: &[u8]) -> Result<PledgeMember> {
    read_single_pledge_member(body)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PledgeShowMemberListDelete {
    pub name: String,
}

pub fn read_pledge_show_member_list_delete(body: &[u8]) -> Result<PledgeShowMemberListDelete> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let name = r.read_s()?;
    Ok(PledgeShowMemberListDelete { name })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PledgeShowMemberListDeleteAll;

pub fn read_pledge_show_member_list_delete_all(body: &[u8]) -> Result<PledgeShowMemberListDeleteAll> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(PledgeShowMemberListDeleteAll)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Crest {
    pub crest_id: i32,
    pub data: Vec<u8>,
}

fn read_crest(body: &[u8]) -> Result<Crest> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let crest_id = r.read_d()?;
    let size = r.read_d()?;
    let size = usize::try_from(size).map_err(|_| WireError::Short)?;
    let data = r.read_b(size)?.to_vec();
    Ok(Crest { crest_id, data })
}

pub fn read_pledge_crest(body: &[u8]) -> Result<Crest> {
    read_crest(body)
}

pub fn read_alliance_crest(body: &[u8]) -> Result<Crest> {
    read_crest(body)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowBoard {
    pub links: Vec<String>,
    pub html: String,
}

pub fn read_show_board(body: &[u8]) -> Result<Option<ShowBoard>> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let show = r.read_c()?;
    if show != 1 {
        return Ok(None);
    }
    let mut links = Vec::with_capacity(BOARD_STRINGS - 1);
    for _ in 0..BOARD_STRINGS - 1 {
        let link = r.read_s()?;
        if link.encode_utf16().count() > BOARD_LINK_MAX_CHARS {
            return Err(WireError::TooLong);
        }
        links.push(link);
    }
    let html = r.read_s()?;
    if html.encode_utf16().count() > BOARD_HTML_MAX_CHARS {
        return Err(WireError::TooLong);
    }
    Ok(Some(ShowBoard { links, html }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChooseInventoryItem {
    pub item_id: i32,
}

pub fn read_choose_inventory_item(body: &[u8]) -> Result<ChooseInventoryItem> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let item_id = r.read_d()?;
    Ok(ChooseInventoryItem { item_id })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FriendAddRequest {
    pub requestor_name: String,
}

pub fn read_friend_add_request(body: &[u8]) -> Result<FriendAddRequest> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let requestor_name = r.read_s()?;
    Ok(FriendAddRequest { requestor_name })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuestState {
    pub quest_id: i32,
    pub cond: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuestExtra {
    pub values: [i32; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestList {
    pub quests: Vec<QuestState>,
    pub extras: Vec<QuestExtra>,
}

pub fn read_quest_list(body: &[u8]) -> Result<QuestList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let declared = r.read_h()?;
    let count = counted(&r, declared as i32, 8)?;
    let mut quests = Vec::with_capacity(count);
    for _ in 0..count {
        let quest_id = r.read_d()?;
        let cond = r.read_d()?;
        quests.push(QuestState { quest_id, cond });
    }
    let declared = read_h_or_zero(&mut r)?;
    let count = counted(&r, declared as i32, 16)?;
    let mut extras = Vec::with_capacity(count);
    for _ in 0..count {
        let mut values = [0i32; 4];
        for value in &mut values {
            *value = r.read_d()?;
        }
        extras.push(QuestExtra { values });
    }
    Ok(QuestList { quests, extras })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnchantResult {
    pub result: i32,
}

pub fn read_enchant_result(body: &[u8]) -> Result<EnchantResult> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let result = r.read_d()?;
    Ok(EnchantResult { result })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PledgeInfo {
    pub pledge_id: i32,
    pub name: String,
    pub ally_name: String,
}

pub fn read_pledge_info(body: &[u8]) -> Result<PledgeInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let pledge_id = r.read_d()?;
    let name = r.read_s()?;
    let ally_name = r.read_s()?;
    Ok(PledgeInfo { pledge_id, name, ally_name })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PledgeShowInfoUpdate {
    pub pledge_id: i32,
    pub crest_id: i32,
    pub level: i32,
    pub castle_id: i32,
    pub hideout_id: i32,
    pub leader_level: i32,
    pub dissolving: i32,
    pub ally_id: i32,
    pub ally_name: String,
    pub ally_crest_id: i32,
    pub at_war: bool,
}

pub fn read_pledge_show_info_update(body: &[u8]) -> Result<PledgeShowInfoUpdate> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let pledge_id = r.read_d()?;
    let crest_id = r.read_d()?;
    let level = r.read_d()?;
    let castle_id = r.read_d()?;
    let hideout_id = r.read_d()?;
    r.read_d()?;
    let leader_level = r.read_d()?;
    let dissolving = r.read_d()?;
    r.read_d()?;
    let ally_id = r.read_d()?;
    let ally_name = r.read_s()?;
    let ally_crest_id = r.read_d()?;
    let at_war = r.read_d()? != 0;
    Ok(PledgeShowInfoUpdate {
        pledge_id,
        crest_id,
        level,
        castle_id,
        hideout_id,
        leader_level,
        dissolving,
        ally_id,
        ally_name,
        ally_crest_id,
        at_war,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LearnableSkill {
    pub id: i32,
    pub next_level: i32,
    pub max_level: i32,
    pub sp_cost: i32,
    pub requirements: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcquireSkillList {
    pub mode: i32,
    pub skills: Vec<LearnableSkill>,
}

pub fn read_acquire_skill_list(body: &[u8]) -> Result<AcquireSkillList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let mode = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 20)?;
    let mut skills = Vec::with_capacity(count);
    for _ in 0..count {
        let id = r.read_d()?;
        let next_level = r.read_d()?;
        let max_level = r.read_d()?;
        let sp_cost = r.read_d()?;
        let requirements = r.read_d()?;
        skills.push(LearnableSkill { id, next_level, max_level, sp_cost, requirements });
    }
    Ok(AcquireSkillList { mode, skills })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillRequirement {
    pub kind: i32,
    pub item_id: i32,
    pub count: i32,
    pub unknown: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcquireSkillInfo {
    pub id: i32,
    pub level: i32,
    pub sp_cost: i32,
    pub mode: i32,
    pub requirements: Vec<SkillRequirement>,
}

pub fn read_acquire_skill_info(body: &[u8]) -> Result<AcquireSkillInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let id = r.read_d()?;
    let level = r.read_d()?;
    let sp_cost = r.read_d()?;
    let mode = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 16)?;
    let mut requirements = Vec::with_capacity(count);
    for _ in 0..count {
        let kind = r.read_d()?;
        let item_id = r.read_d()?;
        let count = r.read_d()?;
        let unknown = r.read_d()?;
        requirements.push(SkillRequirement { kind, item_id, count, unknown });
    }
    Ok(AcquireSkillInfo { id, level, sp_cost, mode, requirements })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcquireSkillDone;

pub fn read_acquire_skill_done(body: &[u8]) -> Result<AcquireSkillDone> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(AcquireSkillDone)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyRoom {
    pub id: i32,
    pub title: String,
    pub location: i32,
    pub min_level: i32,
    pub max_level: i32,
    pub members: i32,
    pub max_members: i32,
    pub owner_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyMatchList {
    pub flag: i32,
    pub rooms: Vec<PartyRoom>,
}

pub fn read_party_match_list(body: &[u8]) -> Result<PartyMatchList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let flag = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 4 + 2 + 5 * 4 + 2)?;
    let mut rooms = Vec::with_capacity(count);
    for _ in 0..count {
        let id = r.read_d()?;
        let title = r.read_s()?;
        let location = r.read_d()?;
        let min_level = r.read_d()?;
        let max_level = r.read_d()?;
        let members = r.read_d()?;
        let max_members = r.read_d()?;
        let owner_name = r.read_s()?;
        rooms.push(PartyRoom { id, title, location, min_level, max_level, members, max_members, owner_name });
    }
    Ok(PartyMatchList { flag, rooms })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyMatchDetail {
    pub id: i32,
    pub max_members: i32,
    pub min_level: i32,
    pub max_level: i32,
    pub loot_type: i32,
    pub location: i32,
    pub title: String,
}

pub fn read_party_match_detail(body: &[u8]) -> Result<PartyMatchDetail> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let id = r.read_d()?;
    let max_members = r.read_d()?;
    let min_level = r.read_d()?;
    let max_level = r.read_d()?;
    let loot_type = r.read_d()?;
    let location = r.read_d()?;
    let title = r.read_s()?;
    Ok(PartyMatchDetail { id, max_members, min_level, max_level, loot_type, location, title })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreSellItem {
    pub type2: i32,
    pub object_id: i32,
    pub item_id: i32,
    pub count: i32,
    pub enchant: i16,
    pub custom_type2: i16,
    pub body_part: i32,
    pub price: i32,
    pub store_price: i32,
}

fn read_store_sell_item(r: &mut Reader, with_store_price: bool) -> Result<StoreSellItem> {
    let type2 = r.read_d()?;
    let object_id = r.read_d()?;
    let item_id = r.read_d()?;
    let count = r.read_d()?;
    r.read_h()?;
    let enchant = r.read_h()? as i16;
    let custom_type2 = r.read_h()? as i16;
    let body_part = r.read_d()?;
    let price = r.read_d()?;
    let store_price = if with_store_price { r.read_d()? } else { 0 };
    Ok(StoreSellItem { type2, object_id, item_id, count, enchant, custom_type2, body_part, price, store_price })
}

const STORE_SELL_ITEM_SIZE: usize = 4 * 4 + 3 * 2 + 4 + 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateStoreManageListSell {
    pub object_id: i32,
    pub package_sale: bool,
    pub adena: i32,
    pub available: Vec<StoreSellItem>,
    pub selling: Vec<StoreSellItem>,
}

pub fn read_private_store_manage_list_sell(body: &[u8]) -> Result<PrivateStoreManageListSell> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let package_sale = r.read_d()? != 0;
    let adena = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, STORE_SELL_ITEM_SIZE)?;
    let mut available = Vec::with_capacity(count);
    for _ in 0..count {
        available.push(read_store_sell_item(&mut r, false)?);
    }
    let declared = r.read_d()?;
    let count = counted(&r, declared, STORE_SELL_ITEM_SIZE + 4)?;
    let mut selling = Vec::with_capacity(count);
    for _ in 0..count {
        selling.push(read_store_sell_item(&mut r, true)?);
    }
    Ok(PrivateStoreManageListSell { object_id, package_sale, adena, available, selling })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateStoreListSell {
    pub object_id: i32,
    pub package_sale: bool,
    pub adena: i32,
    pub items: Vec<StoreSellItem>,
}

pub fn read_private_store_list_sell(body: &[u8]) -> Result<PrivateStoreListSell> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let package_sale = r.read_d()? != 0;
    let adena = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, STORE_SELL_ITEM_SIZE + 4)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        items.push(read_store_sell_item(&mut r, true)?);
    }
    Ok(PrivateStoreListSell { object_id, package_sale, adena, items })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateStoreMsg {
    pub object_id: i32,
    pub message: String,
}

fn read_private_store_msg(body: &[u8]) -> Result<PrivateStoreMsg> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let message = r.read_s()?;
    Ok(PrivateStoreMsg { object_id, message })
}

pub fn read_private_store_msg_sell(body: &[u8]) -> Result<PrivateStoreMsg> {
    read_private_store_msg(body)
}

pub fn read_private_store_msg_buy(body: &[u8]) -> Result<PrivateStoreMsg> {
    read_private_store_msg(body)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreBuyItem {
    pub object_id: i32,
    pub item_id: i32,
    pub enchant: i16,
    pub count: i32,
    pub store_price: i32,
    pub body_part: i32,
    pub type2: i16,
    pub price: i32,
    pub max_count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateStoreManageListBuy {
    pub object_id: i32,
    pub adena: i32,
    pub available: Vec<StoreBuyItem>,
    pub buying: Vec<StoreBuyItem>,
}

fn read_buy_manage_item(r: &mut Reader, wanted: bool) -> Result<StoreBuyItem> {
    let item_id = r.read_d()?;
    let enchant = r.read_h()? as i16;
    let count = r.read_d()?;
    let store_price = r.read_d()?;
    r.read_h()?;
    let body_part = r.read_d()?;
    let type2 = r.read_h()? as i16;
    let (price, max_count) = if wanted { (r.read_d()?, r.read_d()?) } else { (0, 0) };
    Ok(StoreBuyItem { object_id: 0, item_id, enchant, count, store_price, body_part, type2, price, max_count })
}

const BUY_MANAGE_ITEM_SIZE: usize = 4 + 2 + 4 + 4 + 2 + 4 + 2;

pub fn read_private_store_manage_list_buy(body: &[u8]) -> Result<PrivateStoreManageListBuy> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let adena = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, BUY_MANAGE_ITEM_SIZE)?;
    let mut available = Vec::with_capacity(count);
    for _ in 0..count {
        available.push(read_buy_manage_item(&mut r, false)?);
    }
    let declared = r.read_d()?;
    let count = counted(&r, declared, BUY_MANAGE_ITEM_SIZE + 8)?;
    let mut buying = Vec::with_capacity(count);
    for _ in 0..count {
        buying.push(read_buy_manage_item(&mut r, true)?);
    }
    Ok(PrivateStoreManageListBuy { object_id, adena, available, buying })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateStoreListBuy {
    pub object_id: i32,
    pub adena: i32,
    pub items: Vec<StoreBuyItem>,
}

pub fn read_private_store_list_buy(body: &[u8]) -> Result<PrivateStoreListBuy> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let adena = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 4 + BUY_MANAGE_ITEM_SIZE + 8)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let object_id = r.read_d()?;
        let item_id = r.read_d()?;
        let enchant = r.read_h()? as i16;
        let count = r.read_d()?;
        let store_price = r.read_d()?;
        r.read_h()?;
        let body_part = r.read_d()?;
        let type2 = r.read_h()? as i16;
        let price = r.read_d()?;
        let max_count = r.read_d()?;
        items.push(StoreBuyItem {
            object_id,
            item_id,
            enchant,
            count,
            store_price,
            body_part,
            type2,
            price,
            max_count,
        });
    }
    Ok(PrivateStoreListBuy { object_id, adena, items })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShowMinimap {
    pub map_id: i32,
    pub flag: u8,
}

pub fn read_show_minimap(body: &[u8]) -> Result<ShowMinimap> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let map_id = r.read_d()?;
    let flag = if r.remaining() >= 1 { r.read_c()? } else { 0 };
    Ok(ShowMinimap { map_id, flag })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TutorialShowHtml {
    pub html: String,
}

pub fn read_tutorial_show_html(body: &[u8]) -> Result<TutorialShowHtml> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let html = r.read_s()?;
    if html.encode_utf16().count() > TUTORIAL_HTML_MAX_CHARS {
        return Err(WireError::TooLong);
    }
    Ok(TutorialShowHtml { html })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TutorialShowQuestionMark {
    pub mark_id: i32,
}

pub fn read_tutorial_show_question_mark(body: &[u8]) -> Result<TutorialShowQuestionMark> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let mark_id = r.read_d()?;
    Ok(TutorialShowQuestionMark { mark_id })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TutorialEnableClientEvent {
    pub event_id: i32,
}

pub fn read_tutorial_enable_client_event(body: &[u8]) -> Result<TutorialEnableClientEvent> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let event_id = r.read_d()?;
    Ok(TutorialEnableClientEvent { event_id })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TutorialCloseHtml;

pub fn read_tutorial_close_html(body: &[u8]) -> Result<TutorialCloseHtml> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(TutorialCloseHtml)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PetItemList {
    pub items: Vec<InventoryItem>,
}

pub fn read_pet_item_list(body: &[u8]) -> Result<PetItemList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let declared = r.read_h()?;
    let count = counted(&r, declared as i32, super::INVENTORY_ITEM_SIZE)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        items.push(read_inventory_item(&mut r)?);
    }
    Ok(PetItemList { items })
}

pub fn read_pet_inventory_update(body: &[u8]) -> Result<super::InventoryUpdate> {
    super::read_inventory_update(body)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageTarget {
    pub object_id: i32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageToList {
    pub targets: Vec<PackageTarget>,
}

pub fn read_package_to_list(body: &[u8]) -> Result<PackageToList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 6)?;
    let mut targets = Vec::with_capacity(count);
    for _ in 0..count {
        let object_id = r.read_d()?;
        let name = r.read_s()?;
        targets.push(PackageTarget { object_id, name });
    }
    Ok(PackageToList { targets })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSendableList {
    pub target_id: i32,
    pub adena: i32,
    pub items: Vec<WarehouseItem>,
}

pub fn read_package_sendable_list(body: &[u8]) -> Result<PackageSendableList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let target_id = r.read_d()?;
    let adena = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, TRADE_ITEM_SIZE + 4)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let item = read_trade_item(&mut r)?;
        let warehouse_id = r.read_d()?;
        items.push(WarehouseItem { item, warehouse_id });
    }
    Ok(PackageSendableList { target_id, adena, items })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiSellProduct {
    pub item_id: i16,
    pub body_part: i32,
    pub type2: i16,
    pub count: i32,
    pub enchant: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiSellIngredient {
    pub item_id: i16,
    pub type2: i16,
    pub count: i32,
    pub enchant: i16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiSellEntry {
    pub entry_id: i32,
    pub stackable: u8,
    pub products: Vec<MultiSellProduct>,
    pub ingredients: Vec<MultiSellIngredient>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiSellList {
    pub list_id: i32,
    pub page: i32,
    pub finished: bool,
    pub page_size: i32,
    pub entries: Vec<MultiSellEntry>,
}

pub fn read_multi_sell_list(body: &[u8]) -> Result<MultiSellList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let list_id = r.read_d()?;
    let page = r.read_d()?;
    let finished = r.read_d()? == 1;
    let page_size = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 9)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let entry_id = r.read_d()?;
        let stackable = r.read_c()?;
        let product_count = r.read_h()?;
        let ingredient_count = r.read_h()?;
        let products_len = counted(&r, product_count as i32, 14)?;
        let mut products = Vec::with_capacity(products_len);
        for _ in 0..products_len {
            let item_id = r.read_h()? as i16;
            let body_part = r.read_d()?;
            let type2 = r.read_h()? as i16;
            let count = r.read_d()?;
            let enchant = r.read_h()? as i16;
            products.push(MultiSellProduct { item_id, body_part, type2, count, enchant });
        }
        let ingredients_len = counted(&r, ingredient_count as i32, 10)?;
        let mut ingredients = Vec::with_capacity(ingredients_len);
        for _ in 0..ingredients_len {
            let item_id = r.read_h()? as i16;
            let type2 = r.read_h()? as i16;
            let count = r.read_d()?;
            let enchant = r.read_h()? as i16;
            ingredients.push(MultiSellIngredient { item_id, type2, count, enchant });
        }
        entries.push(MultiSellEntry { entry_id, stackable, products, ingredients });
    }
    Ok(MultiSellList { list_id, page, finished, page_size, entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowTownMap {
    pub texture: String,
    pub x: i32,
    pub y: i32,
}

pub fn read_show_town_map(body: &[u8]) -> Result<ShowTownMap> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let texture = r.read_s()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    Ok(ShowTownMap { texture, x, y })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HennaOffer {
    pub symbol_id: i32,
    pub dye_item_id: i32,
    pub dye_amount: i32,
    pub price: i32,
    pub available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HennaEquipList {
    pub adena: i32,
    pub slots: i32,
    pub offers: Vec<HennaOffer>,
}

pub fn read_henna_equip_list(body: &[u8]) -> Result<HennaEquipList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let adena = r.read_d()?;
    let slots = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 20)?;
    let mut offers = Vec::with_capacity(count);
    for _ in 0..count {
        let symbol_id = r.read_d()?;
        let dye_item_id = r.read_d()?;
        let dye_amount = r.read_d()?;
        let price = r.read_d()?;
        let available = r.read_d()? != 0;
        offers.push(HennaOffer { symbol_id, dye_item_id, dye_amount, price, available });
    }
    Ok(HennaEquipList { adena, slots, offers })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HennaItemInfo {
    pub symbol_id: i32,
    pub dye_item_id: i32,
    pub dye_amount: i32,
    pub price: i32,
    pub available: bool,
    pub adena: i32,
    pub int_: (i32, u8),
    pub str_: (i32, u8),
    pub con: (i32, u8),
    pub men: (i32, u8),
    pub dex: (i32, u8),
    pub wit: (i32, u8),
}

pub fn read_henna_item_info(body: &[u8]) -> Result<HennaItemInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let symbol_id = r.read_d()?;
    let dye_item_id = r.read_d()?;
    let dye_amount = r.read_d()?;
    let price = r.read_d()?;
    let available = r.read_d()? != 0;
    let adena = r.read_d()?;
    let mut stats = [(0i32, 0u8); 6];
    for stat in &mut stats {
        *stat = (r.read_d()?, r.read_c()?);
    }
    let [int_, str_, con, men, dex, wit] = stats;
    Ok(HennaItemInfo { symbol_id, dye_item_id, dye_amount, price, available, adena, int_, str_, con, men, dex, wit })
}

/// Os seis `c` são os **modificadores** que os símbolos dão, e o
/// `ReceiveHennaInfo @ 0x10408cc0` os estende **com sinal** (`movsbl` em `0x10408d43` e seguintes)
/// antes de passá-los ao console — ao contrário do `HennaItemInfo`, onde o leitor escreve o byte
/// num `dword` já zerado e o valor é o atributo resultante, sempre positivo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HennaInfo {
    pub int_: i8,
    pub str_: i8,
    pub con: i8,
    pub men: i8,
    pub dex: i8,
    pub wit: i8,
    pub slots: i32,
    pub symbols: Vec<(i32, i32)>,
}

pub fn read_henna_info(body: &[u8]) -> Result<HennaInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let int_ = r.read_c()? as i8;
    let str_ = r.read_c()? as i8;
    let con = r.read_c()? as i8;
    let men = r.read_c()? as i8;
    let dex = r.read_c()? as i8;
    let wit = r.read_c()? as i8;
    let slots = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 8)?;
    let mut symbols = Vec::with_capacity(count);
    for _ in 0..count {
        symbols.push((r.read_d()?, r.read_d()?));
    }
    Ok(HennaInfo { int_, str_, con, men, dex, wit, slots, symbols })
}

/// `0xe5` tem exatamente a forma do `0xe2`: `ReceiveHennaUnequipList @ 0x10409080` lê o mesmo
/// `"ddd"` (adena, espaços, contagem) e o mesmo `"ddddd"` por linha, só trocando os slots do
/// console (`0x4a0`/`0x4a4` no lugar de `0x48c`/`0x490`).
pub fn read_henna_unequip_list(body: &[u8]) -> Result<HennaEquipList> {
    read_henna_equip_list(body)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmViewSkillInfo {
    pub name: String,
    pub entries: Vec<[i32; 3]>,
}

pub fn read_gm_view_skill_info(body: &[u8]) -> Result<GmViewSkillInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let name = r.read_s()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 12)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push([r.read_d()?, r.read_d()?, r.read_d()?]);
    }
    Ok(GmViewSkillInfo { name, entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmViewMagicInfo {
    pub name: String,
    pub entries: Vec<[i32; 2]>,
}

pub fn read_gm_view_magic_info(body: &[u8]) -> Result<GmViewMagicInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let name = r.read_s()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 12)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        r.read_d()?;
        entries.push([r.read_d()?, r.read_d()?]);
    }
    Ok(GmViewMagicInfo { name, entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmViewQuestInfo {
    pub name: String,
    pub first: Vec<(i32, i32)>,
    pub second: Vec<[i32; 4]>,
}

pub fn read_gm_view_quest_info(body: &[u8]) -> Result<GmViewQuestInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let name = r.read_s()?;
    let first_count = r.read_h()?;
    let first_len = counted(&r, first_count as i32, 8)?;
    let mut first = Vec::with_capacity(first_len);
    for _ in 0..first_len {
        first.push((r.read_d()?, r.read_d()?));
    }
    let second_count = r.read_h()?;
    let second_len = counted(&r, second_count as i32, 16)?;
    let mut second = Vec::with_capacity(second_len);
    for _ in 0..second_len {
        second.push([r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?]);
    }
    Ok(GmViewQuestInfo { name, first, second })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GmViewItem {
    pub kind: i16,
    pub values: [i32; 3],
    pub shorts: [i16; 3],
    pub value: i32,
    pub tail: [i16; 2],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmViewItemList {
    pub name: String,
    pub header_value: i32,
    pub header: [i16; 2],
    pub items: Vec<GmViewItem>,
}

pub fn read_gm_view_item_list(body: &[u8]) -> Result<GmViewItemList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let name = r.read_s()?;
    let header_value = r.read_d()?;
    let show = r.read_h()?;
    let declared = r.read_h()?;
    let header = [show as i16, declared as i16];
    let count = counted(&r, declared as i32, 28)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let kind = r.read_h()? as i16;
        let values = [r.read_d()?, r.read_d()?, r.read_d()?];
        let shorts = [r.read_h()? as i16, r.read_h()? as i16, r.read_h()? as i16];
        let value = r.read_d()?;
        let tail = [r.read_h()? as i16, r.read_h()? as i16];
        items.push(GmViewItem { kind, values, shorts, value, tail });
    }
    Ok(GmViewItemList { name, header_value, header, items })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GmViewWarehouseItem {
    pub kind: i16,
    pub values: [i32; 3],
    pub shorts: [i16; 2],
    pub details: Option<(i32, [i16; 3])>,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmViewWarehouseWithdrawList {
    pub name: String,
    pub header_value: i32,
    pub header: i16,
    pub items: Vec<GmViewWarehouseItem>,
}

pub fn read_gm_view_warehouse_withdraw_list(body: &[u8]) -> Result<GmViewWarehouseWithdrawList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let name = r.read_s()?;
    let header_value = r.read_d()?;
    let declared = r.read_h()?;
    let header = declared as i16;
    let count = counted(&r, declared as i32, 22)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let kind = r.read_h()? as i16;
        let values = [r.read_d()?, r.read_d()?, r.read_d()?];
        let shorts = [r.read_h()? as i16, r.read_h()? as i16];
        let details = if (0..=3).contains(&kind) {
            Some((r.read_d()?, [r.read_h()? as i16, r.read_h()? as i16, r.read_h()? as i16]))
        } else {
            None
        };
        let value = r.read_d()?;
        items.push(GmViewWarehouseItem { kind, values, shorts, details, value });
    }
    Ok(GmViewWarehouseWithdrawList { name, header_value, header, items })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SellListProcureItem {
    pub kind: i16,
    pub values: [i32; 3],
    pub shorts: [i16; 2],
    pub details: Option<(i32, [i16; 3])>,
    pub price: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SellListProcure {
    pub header: [i32; 2],
    pub header_short: i16,
    pub items: Vec<SellListProcureItem>,
}

pub fn read_sell_list_procure(body: &[u8]) -> Result<SellListProcure> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let header = [r.read_d()?, r.read_d()?];
    let header_short = r.read_h()? as i16;
    let declared = r.read_h()?;
    let count = counted(&r, declared as i32, 22)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let kind = r.read_h()? as i16;
        let values = [r.read_d()?, r.read_d()?, r.read_d()?];
        let shorts = [r.read_h()? as i16, r.read_h()? as i16];
        let details = if (0..4).contains(&kind) {
            Some((r.read_d()?, [r.read_h()? as i16, r.read_h()? as i16, r.read_h()? as i16]))
        } else {
            None
        };
        let price = r.read_d()?;
        items.push(SellListProcureItem { kind, values, shorts, details, price });
    }
    Ok(SellListProcure { header, header_short, items })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmHennaInfo {
    pub stats: [u8; 6],
    pub header: [i32; 2],
    pub entries: Vec<(i32, i32)>,
}

pub fn read_gm_henna_info(body: &[u8]) -> Result<GmHennaInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let stats = [r.read_c()?, r.read_c()?, r.read_c()?, r.read_c()?, r.read_c()?, r.read_c()?];
    let header = [r.read_d()?, r.read_d()?];
    let count = counted(&r, header[1], 8)?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push((r.read_d()?, r.read_d()?));
    }
    Ok(GmHennaInfo { stats, header, entries })
}

/// `0xe6` tem exatamente a forma do `0xe3` (`"dddddddcdcdcdcdcdc"`), e o handler do console
/// (`0x10071390`, slot `0x4a8`) recebe os mesmos dezoito argumentos que o de equipar
/// (`0x10070fa0`, slot `0x494`): os cinco campos do símbolo, a adena e os seis pares `(d, c)`.
pub fn read_henna_unequip_info(body: &[u8]) -> Result<HennaItemInfo> {
    read_henna_item_info(body)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeedItem {
    pub type1: i16,
    pub object_id: i32,
    pub item_id: i32,
    pub count: i32,
    pub type2: i16,
    pub custom_type1: i16,
    pub body_part: i32,
    pub enchant: i16,
    pub price: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuyListSeed {
    pub money: i32,
    pub manor_id: i32,
    pub items: Vec<SeedItem>,
}

pub fn read_buy_list_seed(body: &[u8]) -> Result<BuyListSeed> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let money = r.read_d()?;
    let manor_id = r.read_d()?;
    let declared = r.read_h()?;
    let count = counted(&r, declared as i32, 2 + 4 * 3 + 2 * 2 + 4)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let type1 = r.read_h()? as i16;
        let object_id = r.read_d()?;
        let item_id = r.read_d()?;
        let count = r.read_d()?;
        let type2 = r.read_h()? as i16;
        let custom_type1 = r.read_h()? as i16;
        let (body_part, enchant) = if (0..4).contains(&type1) {
            let body_part = r.read_d()?;
            let enchant = r.read_h()? as i16;
            r.read_h()?;
            r.read_h()?;
            (body_part, enchant)
        } else {
            (0, 0)
        };
        let price = r.read_d()?;
        items.push(SeedItem { type1, object_id, item_id, count, type2, custom_type1, body_part, enchant, price });
    }
    Ok(BuyListSeed { money, manor_id, items })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RadarControl {
    pub show: i32,
    pub kind: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShowRadar {
    pub first: i32,
    pub second: i32,
    pub third: i32,
    pub fourth: i32,
    pub fifth: i32,
}

pub fn read_show_radar(body: &[u8]) -> Result<ShowRadar> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let first = r.read_d()?;
    let second = r.read_d()?;
    let third = r.read_d()?;
    let fourth = r.read_d()?;
    let fifth = r.read_d()?;
    Ok(ShowRadar { first, second, third, fourth, fifth })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeleteRadar {
    pub first: i32,
    pub second: i32,
    pub third: i32,
}

pub fn read_delete_radar(body: &[u8]) -> Result<DeleteRadar> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let first = r.read_d()?;
    let second = r.read_d()?;
    let third = r.read_d()?;
    Ok(DeleteRadar { first, second, third })
}

pub fn read_radar_control(body: &[u8]) -> Result<RadarControl> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let show = r.read_d()?;
    let kind = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    Ok(RadarControl { show, kind, x, y, z })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreviewItem {
    pub item_id: i32,
    pub type2: i16,
    pub body_part: i16,
    pub price: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShopPreviewList {
    pub header: i32,
    pub money: i32,
    pub list_id: i32,
    pub items: Vec<PreviewItem>,
}

pub fn read_shop_preview_list(body: &[u8]) -> Result<ShopPreviewList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let header = r.read_d()?;
    let money = r.read_d()?;
    let list_id = r.read_d()?;
    let declared = r.read_h()?;
    let count = counted(&r, declared as i32, 12)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let item_id = r.read_d()?;
        let type2 = r.read_h()? as i16;
        let body_part = r.read_h()? as i16;
        let price = r.read_d()?;
        items.push(PreviewItem { item_id, type2, body_part, price });
    }
    Ok(ShopPreviewList { header, money, list_id, items })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShopPreviewInfo {
    pub slots: [i32; SHOP_PREVIEW_SLOTS],
}

pub fn read_shop_preview_info(body: &[u8]) -> Result<ShopPreviewInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let mut slots = [0i32; SHOP_PREVIEW_SLOTS];
    for slot in &mut slots {
        *slot = r.read_d()?;
    }
    Ok(ShopPreviewInfo { slots })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Friend {
    pub object_id: i32,
    pub name: String,
    pub online: bool,
    pub tail: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FriendList {
    pub friends: Vec<Friend>,
}

pub fn read_friend_list(body: &[u8]) -> Result<FriendList> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 4 + 2 + 4)?;
    let mut friends = Vec::with_capacity(count);
    for _ in 0..count {
        let object_id = r.read_d()?;
        let name = r.read_s()?;
        let online = r.read_d()? != 0;
        let tail = read_d_or_zero(&mut r)?;
        friends.push(Friend { object_id, name, online, tail });
    }
    Ok(FriendList { friends })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FriendRecvMsg {
    pub unknown: i32,
    pub receiver: String,
    pub sender: String,
    pub message: String,
}

pub fn read_friend_recv_msg(body: &[u8]) -> Result<FriendRecvMsg> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let unknown = r.read_d()?;
    let receiver = r.read_s()?;
    let sender = r.read_s()?;
    let message = r.read_s()?;
    Ok(FriendRecvMsg { unknown, receiver, sender, message })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L2Friend {
    pub values: [i32; 2],
    pub name: String,
    pub tail: [i32; 2],
}

pub fn read_l2_friend(body: &[u8]) -> Result<L2Friend> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let values = [r.read_d()?, r.read_d()?];
    let name = r.read_s()?;
    let tail = [r.read_d()?, r.read_d()?];
    Ok(L2Friend { values, name, tail })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct L2FriendStatus {
    pub value: i32,
    pub name: String,
    pub status: i32,
}

pub fn read_l2_friend_status(body: &[u8]) -> Result<L2FriendStatus> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let value = r.read_d()?;
    let name = r.read_s()?;
    let status = r.read_d()?;
    Ok(L2FriendStatus { value, name, status })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmPledgeDetails {
    pub values: [i32; 9],
    pub name: String,
    pub tail: [i32; 2],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmViewPledgeInfo {
    pub header: (String, i32, String, String),
    pub details: Option<GmPledgeDetails>,
    pub members: Vec<(String, [i32; 5])>,
}

pub fn read_gm_view_pledge_info(body: &[u8]) -> Result<GmViewPledgeInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let header = (r.read_s()?, r.read_d()?, r.read_s()?, r.read_s()?);
    let details = if header.1 != 0 && !header.2.is_empty() {
        let values = [
            r.read_d()?,
            r.read_d()?,
            r.read_d()?,
            r.read_d()?,
            r.read_d()?,
            r.read_d()?,
            r.read_d()?,
            r.read_d()?,
            r.read_d()?,
        ];
        let name = r.read_s()?;
        let tail = [r.read_d()?, r.read_d()?];
        Some(GmPledgeDetails { values, name, tail })
    } else {
        None
    };
    let declared = r.read_d()?;
    let count = counted(&r, declared, 4 + 5 * 4)?;
    let mut members = Vec::with_capacity(count);
    for _ in 0..count {
        members.push((r.read_s()?, [r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?]));
    }
    Ok(GmViewPledgeInfo { header, details, members })
}

#[derive(Debug, Clone, PartialEq)]
pub struct GmViewCharacterInfo {
    pub header: [i32; 5],
    pub name: String,
    pub values: [i32; 71],
    pub floats: [f64; 4],
    pub post_values: [i32; 4],
    pub title: String,
    pub tail_values: [i32; 3],
    pub flags: [u8; 3],
    pub kills: [i32; 2],
    pub short_values: [u16; 2],
    pub final_values: [i32; 3],
}

pub fn read_gm_view_character_info(body: &[u8]) -> Result<GmViewCharacterInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let header = [r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?];
    let name = r.read_s()?;
    let mut values = [0i32; 71];
    for value in &mut values {
        *value = r.read_d()?;
    }
    let floats = [r.read_f()?, r.read_f()?, r.read_f()?, r.read_f()?];
    let post_values = [r.read_d()?, r.read_d()?, r.read_d()?, r.read_d()?];
    let title = r.read_s()?;
    let tail_values = [r.read_d()?, r.read_d()?, r.read_d()?];
    let flags = [r.read_c()?, r.read_c()?, r.read_c()?];
    let kills = [r.read_d()?, r.read_d()?];
    let short_values = [r.read_h()?, r.read_h()?];
    let final_values = [r.read_d()?, r.read_d()?, r.read_d()?];
    Ok(GmViewCharacterInfo {
        header,
        name,
        values,
        floats,
        post_values,
        title,
        tail_values,
        flags,
        kills,
        short_values,
        final_values,
    })
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

fn opcode_two_ints(opcode: u8, a: i32, b: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(opcode);
    w.write_d(a);
    w.write_d(b);
    w.into_bytes()
}

pub fn request_trade(object_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_TRADE, object_id)
}

pub fn request_add_trade_item(trade_id: i32, object_id: i32, count: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_ADD_TRADE_ITEM);
    w.write_d(trade_id);
    w.write_d(object_id);
    w.write_d(count);
    w.into_bytes()
}

pub fn request_trade_done(confirm: bool) -> Vec<u8> {
    opcode_int(OP_REQUEST_TRADE_DONE, confirm as i32)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemOrder {
    pub object_id: i32,
    pub item_id: i32,
    pub count: i32,
}

fn item_orders(w: &mut Writer, orders: &[ItemOrder]) {
    for order in orders {
        w.write_d(order.object_id);
        w.write_d(order.item_id);
        w.write_d(order.count);
    }
}

pub fn request_sell_item(list_id: i32, orders: &[ItemOrder]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_SELL_ITEM);
    w.write_d(list_id);
    w.write_d(orders.len() as i32);
    item_orders(&mut w, orders);
    w.into_bytes()
}

pub fn request_buy_item(list_id: i32, orders: &[(i32, i32)]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_BUY_ITEM);
    w.write_d(list_id);
    w.write_d(orders.len() as i32);
    for (item_id, count) in orders {
        w.write_d(*item_id);
        w.write_d(*count);
    }
    w.into_bytes()
}

pub fn request_join_pledge(object_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_JOIN_PLEDGE, object_id)
}

pub fn request_withdrawal_pledge() -> Vec<u8> {
    opcode_only(OP_REQUEST_WITHDRAWAL_PLEDGE)
}

pub fn request_oust_pledge_member(name: &str) -> Vec<u8> {
    opcode_string(OP_REQUEST_OUST_PLEDGE_MEMBER, name)
}

fn warehouse_orders(opcode: u8, orders: &[(i32, i32)]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(opcode);
    w.write_d(orders.len() as i32);
    for (object_id, count) in orders {
        w.write_d(*object_id);
        w.write_d(*count);
    }
    w.into_bytes()
}

pub fn send_warehouse_deposit_list(orders: &[(i32, i32)]) -> Vec<u8> {
    warehouse_orders(OP_SEND_WAREHOUSE_DEPOSIT_LIST, orders)
}

pub fn send_warehouse_withdraw_list(orders: &[(i32, i32)]) -> Vec<u8> {
    warehouse_orders(OP_SEND_WAREHOUSE_WITHDRAW_LIST, orders)
}

pub fn request_pledge_member_list() -> Vec<u8> {
    opcode_only(OP_REQUEST_PLEDGE_MEMBER_LIST)
}

fn crest_upload(opcode: u8, data: &[u8]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(opcode);
    w.write_d(data.len() as i32);
    w.write_b(data);
    w.into_bytes()
}

pub fn request_set_pledge_crest(data: &[u8]) -> Vec<u8> {
    crest_upload(OP_REQUEST_SET_PLEDGE_CREST, data)
}

pub fn request_set_ally_crest(data: &[u8]) -> Vec<u8> {
    crest_upload(OP_REQUEST_SET_ALLY_CREST, data)
}

pub fn request_enchant_item(object_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_ENCHANT_ITEM, object_id)
}

pub fn request_quest_abort(quest_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_QUEST_ABORT, quest_id)
}

pub fn request_pledge_crest(crest_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_PLEDGE_CREST, crest_id)
}

pub fn request_ally_crest(crest_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_ALLY_CREST, crest_id)
}

pub fn request_restart_point(point: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_RESTART_POINT, point)
}

pub fn request_crystallize_item(object_id: i32, count: i32) -> Vec<u8> {
    opcode_two_ints(OP_REQUEST_CRYSTALLIZE_ITEM, object_id, count)
}

pub fn request_private_store_manage_sell() -> Vec<u8> {
    opcode_only(OP_REQUEST_PRIVATE_STORE_MANAGE_SELL)
}

pub fn request_private_store_manage_cancel_sell() -> Vec<u8> {
    opcode_only(OP_REQUEST_PRIVATE_STORE_MANAGE_CANCEL_SELL)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreOffer {
    pub object_id: i32,
    pub count: i32,
    pub price: i32,
}

pub fn set_private_store_list_sell(package_sale: bool, offers: &[StoreOffer]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_SET_PRIVATE_STORE_LIST_SELL);
    w.write_d(package_sale as i32);
    w.write_d(offers.len() as i32);
    for offer in offers {
        w.write_d(offer.object_id);
        w.write_d(offer.count);
        w.write_d(offer.price);
    }
    w.into_bytes()
}

pub fn request_private_store_quit_sell() -> Vec<u8> {
    opcode_only(OP_REQUEST_PRIVATE_STORE_QUIT_SELL)
}

pub fn request_private_store_list_sell(store_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_PRIVATE_STORE_LIST_SELL, store_id)
}

pub fn set_private_store_msg_sell(message: &str) -> Vec<u8> {
    opcode_string(OP_SET_PRIVATE_STORE_MSG_SELL, message)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreBuyOrder {
    pub object_id: i32,
    pub count: i32,
    pub price: i32,
}

pub fn request_private_store_buy(store_id: i32, orders: &[StoreBuyOrder]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_PRIVATE_STORE_BUY);
    w.write_d(store_id);
    w.write_d(orders.len() as i32);
    for order in orders {
        w.write_d(order.object_id);
        w.write_d(order.count);
        w.write_d(order.price);
    }
    w.into_bytes()
}

pub fn request_tutorial_link_html(link: &str) -> Vec<u8> {
    opcode_string(OP_REQUEST_TUTORIAL_LINK_HTML, link)
}

pub fn request_tutorial_pass_cmd(command: &str) -> Vec<u8> {
    opcode_string(OP_REQUEST_TUTORIAL_PASS_CMD, command)
}

pub fn request_tutorial_question_mark(mark_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_TUTORIAL_QUESTION_MARK, mark_id)
}

pub fn request_tutorial_client_event(event_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_TUTORIAL_CLIENT_EVENT, event_id)
}

pub fn request_join_ally(object_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_JOIN_ALLY, object_id)
}

pub fn request_ally_leave() -> Vec<u8> {
    opcode_only(OP_REQUEST_ALLY_LEAVE)
}

pub fn request_ally_dismiss(pledge_name: &str) -> Vec<u8> {
    opcode_string(OP_REQUEST_ALLY_DISMISS, pledge_name)
}

pub fn request_dismiss_ally() -> Vec<u8> {
    opcode_only(OP_REQUEST_DISMISS_ALLY)
}

pub fn request_ally_info() -> Vec<u8> {
    opcode_only(OP_REQUEST_ALLY_INFO)
}

pub fn request_pet_get_item(object_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_PET_GET_ITEM, object_id)
}

pub fn request_private_store_manage_buy() -> Vec<u8> {
    opcode_only(OP_REQUEST_PRIVATE_STORE_MANAGE_BUY)
}

pub fn request_private_store_manage_cancel_buy() -> Vec<u8> {
    opcode_only(OP_REQUEST_PRIVATE_STORE_MANAGE_CANCEL_BUY)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuyOffer {
    pub item_id: i32,
    pub enchant: i16,
    pub custom_type2: i16,
    pub count: i32,
    pub price: i32,
}

pub fn set_private_store_list_buy(offers: &[BuyOffer]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_SET_PRIVATE_STORE_LIST_BUY);
    w.write_d(offers.len() as i32);
    for offer in offers {
        w.write_d(offer.item_id);
        w.write_h(offer.enchant as u16);
        w.write_h(offer.custom_type2 as u16);
        w.write_d(offer.count);
        w.write_d(offer.price);
    }
    w.into_bytes()
}

pub fn request_private_store_quit_buy() -> Vec<u8> {
    opcode_only(OP_REQUEST_PRIVATE_STORE_QUIT_BUY)
}

pub fn request_private_store_list_buy(store_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_PRIVATE_STORE_LIST_BUY, store_id)
}

pub fn set_private_store_msg_buy(message: &str) -> Vec<u8> {
    opcode_string(OP_SET_PRIVATE_STORE_MSG_BUY, message)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SellToStoreItem {
    pub object_id: i32,
    pub item_id: i32,
    pub enchant: i16,
    pub custom_type2: i16,
    pub count: i32,
    pub price: i32,
}

pub fn request_private_store_sell(store_id: i32, items: &[SellToStoreItem]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_PRIVATE_STORE_SELL);
    w.write_d(store_id);
    w.write_d(items.len() as i32);
    for item in items {
        w.write_d(item.object_id);
        w.write_d(item.item_id);
        w.write_h(item.enchant as u16);
        w.write_h(item.custom_type2 as u16);
        w.write_d(item.count);
        w.write_d(item.price);
    }
    w.into_bytes()
}

pub fn request_package_sendable_item_list(target_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_PACKAGE_SENDABLE_ITEM_LIST, target_id)
}

pub fn request_package_send(target_id: i32, orders: &[(i32, i32)]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_PACKAGE_SEND);
    w.write_d(target_id);
    w.write_d(orders.len() as i32);
    for (object_id, count) in orders {
        w.write_d(*object_id);
        w.write_d(*count);
    }
    w.into_bytes()
}

pub fn multi_sell_choose(list_id: i32, entry_id: i32, amount: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_MULTI_SELL_CHOOSE);
    w.write_d(list_id);
    w.write_d(entry_id);
    w.write_d(amount);
    w.into_bytes()
}

pub fn request_user_command(command: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_USER_COMMAND, command)
}

pub fn request_henna_list(unknown: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_HENNA_LIST, unknown)
}

pub fn request_henna_item_info(symbol_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_HENNA_ITEM_INFO, symbol_id)
}

pub fn request_henna_equip(symbol_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_HENNA_EQUIP, symbol_id)
}

/// `UNetworkHandler::RequestHennaUnequipList @ 0x103fa890` declara `"cd"` mas chama o escritor
/// variádico só com o opcode, do mesmo jeito que o `RequestHennaItemList @ 0x103fa840`: o `d` sai
/// com o que estivesse na pilha. Mandamos zero, como no `request_henna_list`.
pub fn request_henna_unequip_list(unknown: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_HENNA_UNEQUIP_LIST, unknown)
}

pub fn request_henna_unequip_info(symbol_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_HENNA_UNEQUIP_INFO, symbol_id)
}

pub fn request_henna_unequip(symbol_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_HENNA_UNEQUIP, symbol_id)
}

pub fn request_pledge_power(member_id: i32, action: i32, powers: &[u8; PLEDGE_POWER_BYTES]) -> Vec<u8> {
    if matches!(action, 1 | 2) {
        return opcode_two_ints(OP_REQUEST_PLEDGE_POWER, member_id, action);
    }
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_PLEDGE_POWER);
    w.write_d(member_id);
    w.write_d(action);
    w.write_b(powers);
    w.into_bytes()
}

pub fn request_make_macro(m: &super::Macro) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_MAKE_MACRO);
    w.write_d(m.id);
    w.write_s(&m.name);
    w.write_s(&m.description);
    w.write_s(&m.acronym);
    w.write_c(m.icon);
    w.write_c(m.commands.len() as u8);
    for command in &m.commands {
        w.write_c(command.index);
        w.write_c(command.kind);
        w.write_d(command.skill_id);
        w.write_c(command.shortcut_id);
        w.write_s(&command.command);
    }
    w.into_bytes()
}

pub fn request_delete_macro(id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_DELETE_MACRO, id)
}

pub fn request_procure_crop(manor_id: i32, orders: &[ItemOrder]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_PROCURE_CROP);
    w.write_d(manor_id);
    w.write_d(orders.len() as i32);
    item_orders(&mut w, orders);
    w.into_bytes()
}

pub fn request_preview_item(unknown: i32, list_id: i32, items: &[i32]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_PREVIEW_ITEM);
    w.write_d(unknown);
    w.write_d(list_id);
    w.write_d(items.len() as i32);
    for item_id in items {
        w.write_d(*item_id);
    }
    w.into_bytes()
}

pub fn request_send_friend_msg(message: &str, receiver: &str) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_SEND_FRIEND_MSG);
    w.write_s(message);
    w.write_s(receiver);
    w.into_bytes()
}

pub fn request_show_minimap() -> Vec<u8> {
    opcode_only(OP_REQUEST_SHOW_MINIMAP)
}

pub fn request_friend_invite(name: &str) -> Vec<u8> {
    opcode_string(OP_REQUEST_FRIEND_INVITE, name)
}

pub fn request_answer_friend_invite(accept: bool) -> Vec<u8> {
    opcode_int(OP_REQUEST_ANSWER_FRIEND_INVITE, accept as i32)
}

pub fn request_friend_list() -> Vec<u8> {
    opcode_only(OP_REQUEST_FRIEND_LIST)
}

pub fn request_friend_del(name: &str) -> Vec<u8> {
    opcode_string(OP_REQUEST_FRIEND_DEL, name)
}

pub fn request_give_nick_name(target: &str, title: &str) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_GIVE_NICK_NAME);
    w.write_s(target);
    w.write_s(title);
    w.into_bytes()
}

pub fn request_show_board(unknown: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_SHOW_BOARD, unknown)
}

pub fn request_bbs_write(fields: &[&str; 6]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_BBS_WRITE);
    for field in fields {
        w.write_s(field);
    }
    w.into_bytes()
}

pub fn request_party_match_list(level: i32, location: i32, mode: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_PARTY_MATCH_LIST);
    w.write_d(level);
    w.write_d(location);
    w.write_d(mode);
    w.into_bytes()
}

pub fn request_manage_party_room(
    room_id: i32,
    max_members: i32,
    min_level: i32,
    max_level: i32,
    loot_type: i32,
    title: &str,
) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_MANAGE_PARTY_ROOM);
    w.write_d(room_id);
    w.write_d(max_members);
    w.write_d(min_level);
    w.write_d(max_level);
    w.write_d(loot_type);
    w.write_s(title);
    w.into_bytes()
}

pub fn request_join_party_room(room_id: i32, unknown: i32) -> Vec<u8> {
    opcode_two_ints(OP_REQUEST_JOIN_PARTY_ROOM, room_id, unknown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gm_view_item_list_reads_two_header_shorts_and_a_leading_short_per_item() {
        let mut w = Writer::new();
        w.write_c(OP_GM_VIEW_ITEM_LIST);
        w.write_s("Ana");
        w.write_d(80);
        w.write_h(1);
        w.write_h(1);
        w.write_h(7);
        for v in [10, 20, 30] {
            w.write_d(v);
        }
        for v in [3, 4, 1] {
            w.write_h(v);
        }
        w.write_d(64);
        w.write_h(5);
        w.write_h(6);
        let list = read_gm_view_item_list(&w.into_bytes()).unwrap();
        assert_eq!((list.name.as_str(), list.header_value, list.header), ("Ana", 80, [1, 1]));
        assert_eq!(list.items.len(), 1);
        assert_eq!((list.items[0].kind, list.items[0].values, list.items[0].shorts, list.items[0].value, list.items[0].tail), (7, [10, 20, 30], [3, 4, 1], 64, [5, 6]));
    }

    #[test]
    fn gm_view_magic_info_discards_the_first_of_three_values_per_entry() {
        let mut w = Writer::new();
        w.write_c(OP_GM_VIEW_MAGIC_INFO);
        w.write_s("Ana");
        w.write_d(2);
        for v in [9, 1, 2, 9, 3, 4] {
            w.write_d(v);
        }
        let info = read_gm_view_magic_info(&w.into_bytes()).unwrap();
        assert_eq!(info.entries, vec![[1, 2], [3, 4]]);
    }

    #[test]
    fn gm_view_warehouse_reads_the_conditional_details_only_for_kinds_zero_to_three() {
        let mut w = Writer::new();
        w.write_c(OP_GM_VIEW_WAREHOUSE_WITHDRAW_LIST);
        w.write_s("Ana");
        w.write_d(500);
        w.write_h(2);
        for kind in [2u16, 9] {
            w.write_h(kind);
            for v in [1, 2, 3] {
                w.write_d(v);
            }
            w.write_h(4);
            w.write_h(5);
            if kind <= 3 {
                w.write_d(6);
                for v in [7, 8, 9] {
                    w.write_h(v);
                }
            }
            w.write_d(10);
        }
        let list = read_gm_view_warehouse_withdraw_list(&w.into_bytes()).unwrap();
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.items[0].details, Some((6, [7, 8, 9])));
        assert_eq!((list.items[1].details, list.items[1].value), (None, 10));
    }

    fn ds(op: u8, values: &[i32]) -> Vec<u8> {
        let mut w = Writer::new();
        w.write_c(op);
        for v in values {
            w.write_d(*v);
        }
        w.into_bytes()
    }

    fn trade_item(w: &mut Writer, object_id: i32, item_id: i32, count: i32) {
        w.write_h(1);
        w.write_d(object_id);
        w.write_d(item_id);
        w.write_d(count);
        w.write_h(2);
        w.write_h(0);
        w.write_d(0x80);
        w.write_h(3);
        w.write_h(0);
        w.write_h(4);
    }

    fn expected_item(object_id: i32, item_id: i32, count: i32) -> TradeItem {
        TradeItem {
            type1: 1,
            object_id,
            item_id,
            count,
            type2: 2,
            custom_type1: 0,
            body_part: 0x80,
            enchant: 3,
            custom_type2: 4,
        }
    }

    #[test]
    fn sell_and_buy_lists_read_money_list_id_and_priced_items() {
        let mut w = Writer::new();
        w.write_c(OP_SELL_LIST);
        w.write_d(5000);
        w.write_d(0);
        w.write_h(2);
        trade_item(&mut w, 1001, 57, 1);
        w.write_d(10);
        trade_item(&mut w, 1002, 58, 2);
        w.write_d(20);
        let body = w.into_bytes();
        let list = read_sell_list(&body).unwrap();
        assert_eq!((list.money, list.list_id), (5000, 0));
        assert_eq!(
            list.items,
            vec![
                PricedItem { item: expected_item(1001, 57, 1), price: 10 },
                PricedItem { item: expected_item(1002, 58, 2), price: 20 }
            ]
        );
        assert!(read_sell_list(&body[..body.len() - 1]).is_err());
        let mut buy = body.clone();
        buy[0] = OP_BUY_LIST;
        assert_eq!(read_buy_list(&buy).unwrap().items.len(), 2);
        assert!(read_buy_list(&[OP_BUY_LIST, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0]).is_err());
    }

    #[test]
    fn trade_start_add_update_done_and_request() {
        let mut w = Writer::new();
        w.write_c(OP_TRADE_START);
        w.write_d(7);
        w.write_h(1);
        trade_item(&mut w, 1001, 57, 1);
        let body = w.into_bytes();
        let start = read_trade_start(&body).unwrap();
        assert_eq!(start.partner_id, 7);
        assert_eq!(start.items, vec![expected_item(1001, 57, 1)]);
        assert!(read_trade_start(&body[..body.len() - 2]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_TRADE_OWN_ADD);
        w.write_h(1);
        trade_item(&mut w, 1001, 57, 3);
        let body = w.into_bytes();
        assert_eq!(read_trade_own_add(&body).unwrap().items[0].count, 3);
        assert_eq!(read_trade_other_add(&body).unwrap().items.len(), 1);
        assert!(read_trade_own_add(&body[..5]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_TRADE_UPDATE);
        w.write_h(1);
        w.write_h(3);
        trade_item(&mut w, 1001, 57, 9);
        let body = w.into_bytes();
        let update = read_trade_update(&body).unwrap();
        assert_eq!(update.items, vec![TradeUpdateItem { update_type: 3, item: expected_item(1001, 57, 9) }]);
        assert!(read_trade_update(&body[..body.len() - 1]).is_err());
        assert_eq!(read_trade_done(&ds(OP_TRADE_DONE, &[1])).unwrap(), TradeDone { success: true });
        assert!(read_trade_done(&[OP_TRADE_DONE]).is_err());
        assert_eq!(read_trade_request(&ds(OP_TRADE_REQUEST, &[7])).unwrap(), TradeRequest { sender_id: 7 });
        assert_eq!(read_trade_request(&ds(OP_TRADE_REQUEST, &[7, 0])).unwrap(), TradeRequest { sender_id: 7 });
    }

    #[test]
    fn manage_pledge_power_reads_two_ints_and_the_bitmap_zero_filled_past_the_end() {
        let mut expected = [0u8; PLEDGE_POWER_BYTES];
        expected[..4].copy_from_slice(&0x1234i32.to_le_bytes());
        assert_eq!(
            read_manage_pledge_power(&ds(OP_MANAGE_PLEDGE_POWER, &[0, 0, 0x1234])).unwrap(),
            ManagePledgePower { first: 0, second: 0, powers: expected }
        );
        let mut body = ds(OP_MANAGE_PLEDGE_POWER, &[5, 6]);
        body.extend((0..PLEDGE_POWER_BYTES as u8).map(|b| b + 1));
        body.extend([0xee; 4]);
        let read = read_manage_pledge_power(&body).unwrap();
        assert_eq!((read.first, read.second, read.powers[0], read.powers[31]), (5, 6, 1, 32));
        assert!(read_manage_pledge_power(&ds(OP_MANAGE_PLEDGE_POWER, &[0])).is_err());
        assert_eq!(read_join_pledge(&ds(OP_JOIN_PLEDGE, &[33])).unwrap(), JoinPledge { pledge_id: 33 });
        assert_eq!(read_join_party(&ds(OP_JOIN_PARTY, &[1])).unwrap(), JoinParty { response: 1 });
        assert!(read_join_party(&[OP_JOIN_PARTY, 1]).is_err());
    }

    #[test]
    fn warehouse_lists_read_type_adena_and_items_with_the_warehouse_id() {
        let mut w = Writer::new();
        w.write_c(OP_WAREHOUSE_DEPOSIT_LIST);
        w.write_h(1);
        w.write_d(9000);
        w.write_h(1);
        trade_item(&mut w, 1001, 57, 1);
        w.write_d(1001);
        let body = w.into_bytes();
        let list = read_warehouse_deposit_list(&body).unwrap();
        assert_eq!((list.warehouse_type, list.adena), (1, 9000));
        assert_eq!(list.items, vec![WarehouseItem { item: expected_item(1001, 57, 1), warehouse_id: 1001 }]);
        assert_eq!(read_warehouse_withdraw_list(&body).unwrap(), list);
        assert!(read_warehouse_withdraw_list(&body[..body.len() - 4]).is_err());
    }

    #[test]
    fn trade_press_ok_handlers_are_opcode_only() {
        assert_eq!(read_trade_start_ok(&[OP_TRADE_START_OK]).unwrap(), TradeStartOk);
        assert_eq!(read_trade_press_own_ok(&[OP_TRADE_PRESS_OWN_OK]).unwrap(), TradePressOk);
        assert_eq!(read_trade_press_other_ok(&[OP_TRADE_PRESS_OTHER_OK]).unwrap(), TradePressOk);
        assert_eq!(read_trade_start_ok(&[OP_TRADE_START_OK, 1]).unwrap(), TradeStartOk);
        assert_eq!(read_trade_press_own_ok(&[OP_TRADE_PRESS_OWN_OK, 1]).unwrap(), TradePressOk);
        assert!(read_trade_press_other_ok(&[]).is_err());
    }

    fn pledge_member(w: &mut Writer, name: &str, level: i32, online: i32) {
        w.write_s(name);
        w.write_d(level);
        w.write_d(88);
        w.write_d(0);
        w.write_d(1);
        w.write_d(online);
    }

    #[test]
    fn pledge_member_lists() {
        let mut w = Writer::new();
        w.write_c(OP_PLEDGE_SHOW_MEMBER_LIST_ALL);
        w.write_d(33);
        w.write_s("Canastra");
        w.write_s("Luzal");
        for v in [5, 3, 0, 0, 0, 52, 0, 0, 44] {
            w.write_d(v);
        }
        w.write_s("Aliados");
        w.write_d(6);
        w.write_d(1);
        w.write_d(2);
        pledge_member(&mut w, "A", 40, 100);
        pledge_member(&mut w, "B", 41, 0);
        let body = w.into_bytes();
        let all = read_pledge_show_member_list_all(&body).unwrap();
        assert_eq!((all.pledge_id, all.name.as_str(), all.leader_name.as_str()), (33, "Canastra", "Luzal"));
        assert_eq!(
            (all.crest_id, all.level, all.castle_id, all.hideout_id, all.my_level, all.dissolving),
            (5, 3, 0, 0, 52, 0)
        );
        assert_eq!((all.ally_id, all.ally_name.as_str(), all.ally_crest_id, all.at_war), (44, "Aliados", 6, true));
        assert_eq!(all.members.len(), 2);
        assert_eq!(
            all.members[0],
            PledgeMember { name: "A".into(), level: 40, class_id: 88, unknown: 0, flag: 1, online_object_id: 100 }
        );
        assert!(read_pledge_show_member_list_all(&body[..body.len() - 1]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_PLEDGE_SHOW_MEMBER_LIST_UPDATE);
        pledge_member(&mut w, "A", 42, 0);
        let body = w.into_bytes();
        assert_eq!(read_pledge_show_member_list_update(&body).unwrap().level, 42);
        assert_eq!(read_pledge_show_member_list_add(&body).unwrap().name, "A");
        assert!(read_pledge_show_member_list_add(&body[..body.len() - 2]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_PLEDGE_SHOW_MEMBER_LIST_DELETE);
        w.write_s("A");
        let body = w.into_bytes();
        assert_eq!(
            read_pledge_show_member_list_delete(&body).unwrap(),
            PledgeShowMemberListDelete { name: "A".into() }
        );
        assert!(read_pledge_show_member_list_delete(&body[..2]).is_err());
        assert_eq!(
            read_pledge_show_member_list_delete_all(&[OP_PLEDGE_SHOW_MEMBER_LIST_DELETE_ALL]).unwrap(),
            PledgeShowMemberListDeleteAll
        );
    }

    #[test]
    fn pledge_info_and_show_info_update() {
        let mut w = Writer::new();
        w.write_c(OP_PLEDGE_INFO);
        w.write_d(33);
        w.write_s("Canastra");
        w.write_s("");
        let body = w.into_bytes();
        assert_eq!(
            read_pledge_info(&body).unwrap(),
            PledgeInfo { pledge_id: 33, name: "Canastra".into(), ally_name: String::new() }
        );
        assert!(read_pledge_info(&body[..body.len() - 2]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_PLEDGE_SHOW_INFO_UPDATE);
        for v in [33, 5, 3, 1, 0, 0, 52, 3, 0, 44] {
            w.write_d(v);
        }
        w.write_s("Aliados");
        w.write_d(6);
        w.write_d(0);
        let body = w.into_bytes();
        let u = read_pledge_show_info_update(&body).unwrap();
        assert_eq!(
            (u.pledge_id, u.crest_id, u.level, u.castle_id, u.hideout_id, u.leader_level, u.dissolving),
            (33, 5, 3, 1, 0, 52, 3)
        );
        assert_eq!((u.ally_id, u.ally_name.as_str(), u.ally_crest_id, u.at_war), (44, "Aliados", 6, false));
        assert!(read_pledge_show_info_update(&body[..body.len() - 4]).is_err());
    }

    #[test]
    fn crests_carry_the_raw_bytes_after_id_and_size() {
        let mut body = ds(OP_PLEDGE_CREST, &[5, 3]);
        body.extend_from_slice(&[9, 8, 7]);
        assert_eq!(read_pledge_crest(&body).unwrap(), Crest { crest_id: 5, data: vec![9, 8, 7] });
        body[0] = OP_ALLIANCE_CREST;
        assert_eq!(read_alliance_crest(&body).unwrap().data, vec![9, 8, 7]);
        assert!(read_pledge_crest(&body[..body.len() - 1]).is_err());
        assert!(read_pledge_crest(&ds(OP_PLEDGE_CREST, &[5, -1])).is_err());
    }

    #[test]
    fn show_board_reads_the_flag_then_eight_links_and_the_html() {
        let mut w = Writer::new();
        w.write_c(OP_SHOW_BOARD);
        w.write_c(1);
        for link in [
            "bypass _bbshome",
            "bypass _bbsgetfav",
            "bypass _bbsloc",
            "bypass _bbsclan",
            "bypass _bbsmemo",
            "bypass _bbsmail",
            "bypass _bbsfriends",
            "bypass bbs_add_fav",
        ] {
            w.write_s(link);
        }
        w.write_s("<html><body>oi</body></html>");
        let body = w.into_bytes();
        let board = read_show_board(&body).unwrap().unwrap();
        assert_eq!(board.links.len(), 8);
        assert_eq!(board.links[2], "bypass _bbsloc");
        assert_eq!(board.html, "<html><body>oi</body></html>");
        assert!(read_show_board(&body[..body.len() - 2]).is_err());
        assert_eq!(read_show_board(&[OP_SHOW_BOARD, 0]).unwrap(), None);
        let mut w = Writer::new();
        w.write_c(OP_SHOW_BOARD);
        w.write_c(1);
        w.write_s(&"l".repeat(BOARD_LINK_MAX_CHARS + 1));
        for _ in 0..8 {
            w.write_s("");
        }
        assert_eq!(read_show_board(&w.into_bytes()), Err(WireError::TooLong));
    }

    #[test]
    fn quests_enchant_friend_request_and_choose_item() {
        let mut w = Writer::new();
        w.write_c(OP_QUEST_LIST);
        w.write_h(2);
        w.write_d(101);
        w.write_d(3);
        w.write_d(257);
        w.write_d(1);
        let lisvus = w.into_bytes();
        let list = read_quest_list(&lisvus).unwrap();
        assert_eq!(list.quests, vec![QuestState { quest_id: 101, cond: 3 }, QuestState { quest_id: 257, cond: 1 }]);
        assert!(list.extras.is_empty());
        let mut full = lisvus.clone();
        full.extend_from_slice(&1u16.to_le_bytes());
        full.extend(ds(0, &[1, 2, 3, 4])[1..].iter());
        assert_eq!(read_quest_list(&full).unwrap().extras, vec![QuestExtra { values: [1, 2, 3, 4] }]);
        assert!(read_quest_list(&lisvus[..lisvus.len() - 1]).is_err());
        assert_eq!(read_enchant_result(&ds(OP_ENCHANT_RESULT, &[0])).unwrap(), EnchantResult { result: 0 });
        assert!(read_enchant_result(&[OP_ENCHANT_RESULT]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_FRIEND_ADD_REQUEST);
        w.write_s("Luzal");
        w.write_d(0);
        assert_eq!(
            read_friend_add_request(&w.into_bytes()).unwrap(),
            FriendAddRequest { requestor_name: "Luzal".into() }
        );
        assert!(read_friend_add_request(&[OP_FRIEND_ADD_REQUEST, b'L']).is_err());
        assert_eq!(
            read_choose_inventory_item(&ds(OP_CHOOSE_INVENTORY_ITEM, &[57])).unwrap(),
            ChooseInventoryItem { item_id: 57 }
        );
        assert_eq!(read_choose_inventory_item(&ds(OP_CHOOSE_INVENTORY_ITEM, &[57, 0])).unwrap(), ChooseInventoryItem { item_id: 57 });
    }

    #[test]
    fn receive_acquire_skill_list_10405450_preserves_every_mode_bit() {
        for mode in [i32::MIN, -1, 0, 1, 2, i32::MAX] {
            let list = read_acquire_skill_list(&ds(OP_ACQUIRE_SKILL_LIST, &[mode, 1, 3, 2, 5, 1000, 0])).unwrap();
            assert_eq!(list.mode, mode);
            assert_eq!(super::super::request_acquire_skill_info(3, 2, mode), ds(super::super::OP_REQUEST_ACQUIRE_SKILL_INFO, &[3, 2, mode]));
            assert_eq!(super::super::request_acquire_skill(3, 2, mode), ds(super::super::OP_REQUEST_ACQUIRE_SKILL, &[3, 2, mode]));
        }
    }

    #[test]
    fn acquire_skill_list_info_and_done() {
        let list = read_acquire_skill_list(&ds(OP_ACQUIRE_SKILL_LIST, &[0, 1, 3, 2, 5, 1000, 0])).unwrap();
        assert_eq!(list.mode, 0);
        assert_eq!(
            list.skills,
            vec![LearnableSkill { id: 3, next_level: 2, max_level: 5, sp_cost: 1000, requirements: 0 }]
        );
        assert!(read_acquire_skill_list(&ds(OP_ACQUIRE_SKILL_LIST, &[0, 1, 3, 2])).is_err());
        let info = read_acquire_skill_info(&ds(OP_ACQUIRE_SKILL_INFO, &[3, 2, 1000, 0, 1, 99, 1830, 3, 0])).unwrap();
        assert_eq!((info.id, info.level, info.sp_cost, info.mode), (3, 2, 1000, 0));
        assert_eq!(info.requirements, vec![SkillRequirement { kind: 99, item_id: 1830, count: 3, unknown: 0 }]);
        assert!(read_acquire_skill_info(&ds(OP_ACQUIRE_SKILL_INFO, &[3, 2, 1000, 0, 1, 99])).is_err());
        assert_eq!(read_acquire_skill_done(&[OP_ACQUIRE_SKILL_DONE]).unwrap(), AcquireSkillDone);
    }

    #[test]
    fn party_match_list_and_detail() {
        let mut w = Writer::new();
        w.write_c(OP_PARTY_MATCH_LIST);
        w.write_d(1);
        w.write_d(1);
        w.write_d(7);
        w.write_s("Sala");
        for v in [3, 20, 40, 2, 9] {
            w.write_d(v);
        }
        w.write_s("Luzal");
        let body = w.into_bytes();
        let list = read_party_match_list(&body).unwrap();
        assert_eq!(list.flag, 1);
        assert_eq!(
            list.rooms,
            vec![PartyRoom {
                id: 7,
                title: "Sala".into(),
                location: 3,
                min_level: 20,
                max_level: 40,
                members: 2,
                max_members: 9,
                owner_name: "Luzal".into()
            }]
        );
        assert!(read_party_match_list(&body[..body.len() - 2]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_PARTY_MATCH_DETAIL);
        for v in [7, 9, 20, 40, 1, 3] {
            w.write_d(v);
        }
        w.write_s("Sala");
        let body = w.into_bytes();
        assert_eq!(
            read_party_match_detail(&body).unwrap(),
            PartyMatchDetail {
                id: 7,
                max_members: 9,
                min_level: 20,
                max_level: 40,
                loot_type: 1,
                location: 3,
                title: "Sala".into()
            }
        );
        assert!(read_party_match_detail(&body[..body.len() - 2]).is_err());
    }

    fn store_sell_item(w: &mut Writer, object_id: i32, price: i32, store_price: Option<i32>) {
        w.write_d(2);
        w.write_d(object_id);
        w.write_d(57);
        w.write_d(1);
        w.write_h(0);
        w.write_h(3);
        w.write_h(0);
        w.write_d(0x80);
        w.write_d(price);
        if let Some(store_price) = store_price {
            w.write_d(store_price);
        }
    }

    #[test]
    fn private_store_sell_lists_and_messages() {
        let mut w = Writer::new();
        w.write_c(OP_PRIVATE_STORE_MANAGE_LIST_SELL);
        w.write_d(7);
        w.write_d(0);
        w.write_d(5000);
        w.write_d(1);
        store_sell_item(&mut w, 1001, 100, None);
        w.write_d(1);
        store_sell_item(&mut w, 1002, 200, Some(150));
        let body = w.into_bytes();
        let manage = read_private_store_manage_list_sell(&body).unwrap();
        assert_eq!((manage.object_id, manage.package_sale, manage.adena), (7, false, 5000));
        assert_eq!(
            manage.available[0],
            StoreSellItem {
                type2: 2,
                object_id: 1001,
                item_id: 57,
                count: 1,
                enchant: 3,
                custom_type2: 0,
                body_part: 0x80,
                price: 100,
                store_price: 0
            }
        );
        assert_eq!(manage.selling[0].store_price, 150);
        assert!(read_private_store_manage_list_sell(&body[..body.len() - 4]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_PRIVATE_STORE_LIST_SELL);
        w.write_d(7);
        w.write_d(1);
        w.write_d(5000);
        w.write_d(1);
        store_sell_item(&mut w, 1002, 200, Some(150));
        let body = w.into_bytes();
        let list = read_private_store_list_sell(&body).unwrap();
        assert!(list.package_sale);
        assert_eq!(list.items[0].price, 200);
        assert!(read_private_store_list_sell(&body[..body.len() - 4]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_PRIVATE_STORE_MSG_SELL);
        w.write_d(7);
        w.write_s("barato");
        let body = w.into_bytes();
        assert_eq!(
            read_private_store_msg_sell(&body).unwrap(),
            PrivateStoreMsg { object_id: 7, message: "barato".into() }
        );
        assert_eq!(read_private_store_msg_buy(&body).unwrap().message, "barato");
        assert!(read_private_store_msg_buy(&body[..body.len() - 2]).is_err());
    }

    #[test]
    fn private_store_buy_lists() {
        let mut w = Writer::new();
        w.write_c(OP_PRIVATE_STORE_MANAGE_LIST_BUY);
        w.write_d(7);
        w.write_d(5000);
        w.write_d(1);
        w.write_d(57);
        w.write_h(0);
        w.write_d(10);
        w.write_d(5);
        w.write_h(0);
        w.write_d(0);
        w.write_h(4);
        w.write_d(1);
        w.write_d(58);
        w.write_h(1);
        w.write_d(2);
        w.write_d(6);
        w.write_h(0);
        w.write_d(0x80);
        w.write_h(1);
        w.write_d(300);
        w.write_d(2);
        let body = w.into_bytes();
        let manage = read_private_store_manage_list_buy(&body).unwrap();
        assert_eq!((manage.object_id, manage.adena), (7, 5000));
        assert_eq!(
            manage.available[0],
            StoreBuyItem {
                object_id: 0,
                item_id: 57,
                enchant: 0,
                count: 10,
                store_price: 5,
                body_part: 0,
                type2: 4,
                price: 0,
                max_count: 0
            }
        );
        assert_eq!(
            manage.buying[0],
            StoreBuyItem {
                object_id: 0,
                item_id: 58,
                enchant: 1,
                count: 2,
                store_price: 6,
                body_part: 0x80,
                type2: 1,
                price: 300,
                max_count: 2
            }
        );
        assert!(read_private_store_manage_list_buy(&body[..body.len() - 1]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_PRIVATE_STORE_LIST_BUY);
        w.write_d(7);
        w.write_d(5000);
        w.write_d(1);
        w.write_d(1001);
        w.write_d(58);
        w.write_h(1);
        w.write_d(2);
        w.write_d(6);
        w.write_h(0);
        w.write_d(0x80);
        w.write_h(1);
        w.write_d(300);
        w.write_d(2);
        let body = w.into_bytes();
        let list = read_private_store_list_buy(&body).unwrap();
        assert_eq!(list.items[0].object_id, 1001);
        assert_eq!(list.items[0].price, 300);
        assert!(read_private_store_list_buy(&body[..body.len() - 3]).is_err());
    }

    #[test]
    fn minimap_tutorial_and_town_map() {
        assert_eq!(read_show_minimap(&ds(OP_SHOW_MINIMAP, &[1665])).unwrap(), ShowMinimap { map_id: 1665, flag: 0 });
        assert_eq!(read_show_minimap(&[OP_SHOW_MINIMAP, 0, 0, 0, 0, 1]).unwrap().flag, 1);
        assert!(read_show_minimap(&[OP_SHOW_MINIMAP, 0]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_TUTORIAL_SHOW_HTML);
        w.write_s("<html>oi</html>");
        let body = w.into_bytes();
        assert_eq!(read_tutorial_show_html(&body).unwrap().html, "<html>oi</html>");
        assert!(read_tutorial_show_html(&body[..body.len() - 1]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_TUTORIAL_SHOW_HTML);
        w.write_s(&"x".repeat(TUTORIAL_HTML_MAX_CHARS + 1));
        assert_eq!(read_tutorial_show_html(&w.into_bytes()), Err(WireError::TooLong));
        assert_eq!(
            read_tutorial_show_question_mark(&ds(OP_TUTORIAL_SHOW_QUESTION_MARK, &[1])).unwrap(),
            TutorialShowQuestionMark { mark_id: 1 }
        );
        assert_eq!(
            read_tutorial_enable_client_event(&ds(OP_TUTORIAL_ENABLE_CLIENT_EVENT, &[8])).unwrap(),
            TutorialEnableClientEvent { event_id: 8 }
        );
        assert!(read_tutorial_enable_client_event(&[OP_TUTORIAL_ENABLE_CLIENT_EVENT]).is_err());
        assert_eq!(read_tutorial_close_html(&[OP_TUTORIAL_CLOSE_HTML]).unwrap(), TutorialCloseHtml);
        let mut w = Writer::new();
        w.write_c(OP_SHOW_TOWN_MAP);
        w.write_s("town_map.giran");
        w.write_d(1);
        w.write_d(2);
        let body = w.into_bytes();
        assert_eq!(read_show_town_map(&body).unwrap(), ShowTownMap { texture: "town_map.giran".into(), x: 1, y: 2 });
        assert!(read_show_town_map(&body[..body.len() - 4]).is_err());
    }

    #[test]
    fn pet_item_list_and_inventory_update_share_the_inventory_item_shape() {
        let mut w = Writer::new();
        w.write_c(OP_PET_ITEM_LIST);
        w.write_h(1);
        w.write_h(1);
        w.write_d(1001);
        w.write_d(57);
        w.write_d(5);
        w.write_h(2);
        w.write_h(0xff);
        w.write_h(0);
        w.write_d(0);
        w.write_h(0);
        w.write_h(0);
        let body = w.into_bytes();
        let list = read_pet_item_list(&body).unwrap();
        assert_eq!((list.items[0].object_id, list.items[0].item_id, list.items[0].count), (1001, 57, 5));
        assert!(read_pet_item_list(&body[..body.len() - 1]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_PET_INVENTORY_UPDATE);
        w.write_h(1);
        w.write_h(3);
        w.write_h(1);
        w.write_d(1001);
        w.write_d(57);
        w.write_d(0);
        w.write_h(2);
        w.write_h(0);
        w.write_h(0);
        w.write_d(0);
        w.write_h(0);
        w.write_h(0);
        let body = w.into_bytes();
        let update = read_pet_inventory_update(&body).unwrap();
        assert_eq!(update.changes[0].change, 3);
        assert!(read_pet_inventory_update(&body[..body.len() - 2]).is_err());
    }

    #[test]
    fn package_lists() {
        let mut w = Writer::new();
        w.write_c(OP_PACKAGE_TO_LIST);
        w.write_d(1);
        w.write_d(9);
        w.write_s("Bravo");
        let body = w.into_bytes();
        assert_eq!(
            read_package_to_list(&body).unwrap().targets,
            vec![PackageTarget { object_id: 9, name: "Bravo".into() }]
        );
        assert!(read_package_to_list(&body[..body.len() - 2]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_PACKAGE_SENDABLE_LIST);
        w.write_d(9);
        w.write_d(5000);
        w.write_d(1);
        trade_item(&mut w, 1001, 57, 1);
        w.write_d(1001);
        let body = w.into_bytes();
        let list = read_package_sendable_list(&body).unwrap();
        assert_eq!((list.target_id, list.adena), (9, 5000));
        assert_eq!(list.items[0].warehouse_id, 1001);
        assert!(read_package_sendable_list(&body[..body.len() - 1]).is_err());
    }

    #[test]
    fn multi_sell_list_reads_entries_products_and_ingredients() {
        let mut w = Writer::new();
        w.write_c(OP_MULTI_SELL_LIST);
        w.write_d(1);
        w.write_d(1);
        w.write_d(1);
        w.write_d(0x28);
        w.write_d(1);
        w.write_d(1);
        w.write_c(1);
        w.write_h(1);
        w.write_h(2);
        w.write_h(57);
        w.write_d(0);
        w.write_h(4);
        w.write_d(100);
        w.write_h(0);
        w.write_h(1458);
        w.write_h(4);
        w.write_d(5);
        w.write_h(0);
        w.write_h(57);
        w.write_h(4);
        w.write_d(1000);
        w.write_h(0);
        let body = w.into_bytes();
        let list = read_multi_sell_list(&body).unwrap();
        assert_eq!((list.list_id, list.page, list.finished, list.page_size), (1, 1, true, 0x28));
        assert_eq!(list.entries.len(), 1);
        assert_eq!(
            list.entries[0].products,
            vec![MultiSellProduct { item_id: 57, body_part: 0, type2: 4, count: 100, enchant: 0 }]
        );
        assert_eq!(list.entries[0].ingredients.len(), 2);
        assert_eq!(
            list.entries[0].ingredients[1],
            MultiSellIngredient { item_id: 57, type2: 4, count: 1000, enchant: 0 }
        );
        assert!(read_multi_sell_list(&body[..body.len() - 1]).is_err());
        assert!(read_multi_sell_list(&ds(OP_MULTI_SELL_LIST, &[1, 1, 1, 0x28, 0])).unwrap().entries.is_empty());
    }

    #[test]
    fn receive_multi_sell_10407960_finishes_only_on_one() {
        for marker in [i32::MIN, -1, 0, 1, 2, i32::MAX] {
            let packet = ds(OP_MULTI_SELL_LIST, &[7, 1, marker, 40, 0]);
            assert_eq!(read_multi_sell_list(&packet).unwrap().finished, marker == 1);
        }
    }

    #[test]
    fn henna_lists_and_info() {
        let list = read_henna_equip_list(&ds(OP_HENNA_EQUIP_LIST, &[5000, 3, 1, 1, 4445, 10, 900, 1])).unwrap();
        assert_eq!((list.adena, list.slots), (5000, 3));
        assert_eq!(
            list.offers,
            vec![HennaOffer { symbol_id: 1, dye_item_id: 4445, dye_amount: 10, price: 900, available: true }]
        );
        assert!(read_henna_equip_list(&ds(OP_HENNA_EQUIP_LIST, &[5000, 3, 1, 1])).is_err());
        let mut w = Writer::new();
        w.write_c(OP_HENNA_ITEM_INFO);
        for v in [1, 4445, 10, 900, 1, 5000] {
            w.write_d(v);
        }
        for (current, equipped) in [(21, 22), (40, 39), (43, 43), (25, 25), (30, 31), (11, 11)] {
            w.write_d(current);
            w.write_c(equipped);
        }
        let body = w.into_bytes();
        let info = read_henna_item_info(&body).unwrap();
        assert_eq!(
            (info.symbol_id, info.adena, info.int_, info.str_, info.wit),
            (1, 5000, (21, 22), (40, 39), (11, 11))
        );
        assert!(read_henna_item_info(&body[..body.len() - 1]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_HENNA_INFO);
        for v in [1u8, 0, 0, 0, 0xfe, 0] {
            w.write_c(v);
        }
        w.write_d(3);
        w.write_d(1);
        w.write_d(7);
        w.write_d(7);
        let body = w.into_bytes();
        let info = read_henna_info(&body).unwrap();
        assert_eq!((info.int_, info.slots), (1, 3));
        assert_eq!(info.dex, -2, "os seis modificadores têm sinal (`movsbl` em 0x10408d43)");
        assert_eq!(info.symbols, vec![(7, 7)]);
        assert!(read_henna_info(&body[..body.len() - 4]).is_err());
    }

    /// `0xe5` e `0xe6` têm a mesma forma do `0xe2` e do `0xe3`: o `ReceiveHennaUnequipList @
    /// 0x10409080` lê o mesmo `"ddd"` (adena, espaços, contagem) e o mesmo `"ddddd"` por linha, e o
    /// `ReceiveHennaUnequipInfo @ 0x10409500` o mesmo `"dddddddcdcdcdcdcdc"`. A contagem do
    /// cabeçalho é o **terceiro** campo, não o primeiro.
    #[test]
    fn henna_unequip_lists_and_info_read_like_the_equip_ones() {
        let body = ds(OP_HENNA_UNEQUIP_LIST, &[5000, 3, 1, 145, 4589, 10, 33000, 0]);
        let list = read_henna_unequip_list(&body).unwrap();
        assert_eq!((list.adena, list.slots), (5000, 3));
        assert_eq!(
            list.offers,
            vec![HennaOffer { symbol_id: 145, dye_item_id: 4589, dye_amount: 10, price: 33000, available: false }]
        );
        assert_eq!(read_henna_unequip_list(&ds(OP_HENNA_UNEQUIP_LIST, &[5000, 3, 0])).unwrap().offers, vec![]);
        assert!(read_henna_unequip_list(&body[..body.len() - 1]).is_err());

        let mut w = Writer::new();
        w.write_c(OP_HENNA_UNEQUIP_INFO);
        for v in [145, 4589, 10, 33000, 1, 5000] {
            w.write_d(v);
        }
        for (current, unequipped) in [(21, 19), (40, 38), (43, 43), (25, 25), (30, 30), (11, 11)] {
            w.write_d(current);
            w.write_c(unequipped);
        }
        let body = w.into_bytes();
        let info = read_henna_unequip_info(&body).unwrap();
        assert_eq!((info.symbol_id, info.dye_item_id, info.price, info.adena), (145, 4589, 33000, 5000));
        assert_eq!((info.int_, info.str_, info.con, info.men, info.dex, info.wit),
            ((21, 19), (40, 38), (43, 43), (25, 25), (30, 30), (11, 11)));
        assert!(read_henna_unequip_info(&body[..body.len() - 1]).is_err());
    }

    #[test]
    fn buy_list_seed_reads_the_body_part_block_only_for_equipment_types() {
        let mut w = Writer::new();
        w.write_c(OP_BUY_LIST_SEED);
        w.write_d(5000);
        w.write_d(1);
        w.write_h(2);
        w.write_h(4);
        w.write_d(0);
        w.write_d(5016);
        w.write_d(100);
        w.write_h(4);
        w.write_h(0);
        w.write_d(30);
        w.write_h(1);
        w.write_d(0);
        w.write_d(10);
        w.write_d(1);
        w.write_h(1);
        w.write_h(0);
        w.write_d(0x80);
        w.write_h(2);
        w.write_h(0);
        w.write_h(0);
        w.write_d(700);
        let body = w.into_bytes();
        let list = read_buy_list_seed(&body).unwrap();
        assert_eq!((list.money, list.manor_id), (5000, 1));
        assert_eq!(
            list.items[0],
            SeedItem {
                type1: 4,
                object_id: 0,
                item_id: 5016,
                count: 100,
                type2: 4,
                custom_type1: 0,
                body_part: 0,
                enchant: 0,
                price: 30
            }
        );
        assert_eq!(
            list.items[1],
            SeedItem {
                type1: 1,
                object_id: 0,
                item_id: 10,
                count: 1,
                type2: 1,
                custom_type1: 0,
                body_part: 0x80,
                enchant: 2,
                price: 700
            }
        );
        assert!(read_buy_list_seed(&body[..body.len() - 2]).is_err());
    }

    #[test]
    fn radar_preview_and_friends() {
        assert_eq!(read_start_pledge_war(&[OP_START_PLEDGE_WAR]).unwrap(), PledgeWarNoop);
        assert_eq!(read_reply_start_pledge_war(&[OP_REPLY_START_PLEDGE_WAR]).unwrap(), PledgeWarNoop);
        assert_eq!(read_stop_pledge_war(&[OP_STOP_PLEDGE_WAR]).unwrap(), PledgeWarNoop);
        assert_eq!(read_reply_stop_pledge_war(&[OP_REPLY_STOP_PLEDGE_WAR]).unwrap(), PledgeWarNoop);
        assert_eq!(read_surrender_pledge_war(&[OP_SURRENDER_PLEDGE_WAR]).unwrap(), PledgeWarNoop);
        assert_eq!(read_reply_surrender_pledge_war(&[OP_REPLY_SURRENDER_PLEDGE_WAR]).unwrap(), PledgeWarNoop);
        assert_eq!(read_set_pledge_crest(&[OP_SET_PLEDGE_CREST]).unwrap(), SetPledgeCrestNoop);
        assert_eq!(read_surrender_personally(&[OP_SURRENDER_PERSONALLY]).unwrap(), SurrenderPersonallyNoop);
        assert_eq!(read_server_close_socket(&[OP_SERVER_CLOSE_SOCKET]).unwrap(), ServerCloseSocketNoop);
        assert_eq!(read_surrender_personally(&[OP_SURRENDER_PERSONALLY, 1]).unwrap(), SurrenderPersonallyNoop);
        assert_eq!(read_server_close_socket(&[OP_SERVER_CLOSE_SOCKET, 1]).unwrap(), ServerCloseSocketNoop);
        assert_eq!(read_reply_start_alliance_war(&[OP_REPLY_START_ALLIANCE_WAR]).unwrap(), AllianceWarNoop);
        assert_eq!(read_reply_stop_alliance_war(&[OP_REPLY_STOP_ALLIANCE_WAR]).unwrap(), AllianceWarNoop);
        assert_eq!(read_surrender_alliance_war(&[OP_SURRENDER_ALLIANCE_WAR]).unwrap(), AllianceWarNoop);
        assert_eq!(read_magic_and_skill_list(&[OP_MAGIC_AND_SKILL_LIST, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0]).unwrap(), MagicAndSkillList { first: 1, second: 2, third: 3 });
        assert!(read_magic_and_skill_list(&[OP_MAGIC_AND_SKILL_LIST, 1]).is_err());
        assert_eq!(read_skill_remain_sec(&[OP_SKILL_REMAIN_SEC, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0]).unwrap().values, [1, 2, 3, 4, 5, 6, 7]);
        assert!(read_skill_remain_sec(&[OP_SKILL_REMAIN_SEC]).is_err());
        assert_eq!(read_vehicle_check_location(&[OP_VEHICLE_CHECK_LOCATION, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]).unwrap().values, [1, 2, 3, 4]);
        assert!(read_vehicle_check_location(&[OP_VEHICLE_CHECK_LOCATION, 1]).is_err());
        assert_eq!(read_dummy(&[OP_DUMMY]).unwrap(), DummyNoop);
        assert_eq!(read_abnormal_visual_effect(&[OP_ABNORMAL_VISUAL_EFFECT]).unwrap(), AbnormalVisualEffectNoop);
        assert_eq!(read_dummy(&[OP_DUMMY, 1]).unwrap(), DummyNoop);
        assert_eq!(read_abnormal_visual_effect(&[OP_ABNORMAL_VISUAL_EFFECT, 1]).unwrap(), AbnormalVisualEffectNoop);
        let pledge = [OP_CREATE_PLEDGE, 9, 0, 0, 0];
        assert_eq!(read_create_pledge(&pledge).unwrap(), PledgeIdResult { pledge_id: 9 });
        assert_eq!(read_withdrawal_pledge(&[OP_WITHDRAWAL_PLEDGE, 9, 0, 0, 0]).unwrap(), PledgeIdResult { pledge_id: 9 });
        assert_eq!(read_oust_pledge_member(&[OP_OUST_PLEDGE_MEMBER, 9, 0, 0, 0]).unwrap(), PledgeIdResult { pledge_id: 9 });
        assert_eq!(read_dismiss_pledge(&[OP_DISMISS_PLEDGE, 9, 0, 0, 0]).unwrap(), PledgeIdResult { pledge_id: 9 });
        assert_eq!(read_withdrawal_party(&[OP_WITHDRAWAL_PARTY, 7, 0, 0, 0]).unwrap(), PartyMemberResult { member_id: 7 });
        assert_eq!(read_oust_party_member(&[OP_OUST_PARTY_MEMBER, 7, 0, 0, 0]).unwrap(), PartyMemberResult { member_id: 7 });
        assert_eq!(read_dismiss_party(&[OP_DISMISS_PARTY, 7, 0, 0, 0]).unwrap(), PartyMemberResult { member_id: 7 });
        assert_eq!(read_set_oust_pledge_member(&[OP_SET_OUST_PLEDGE_MEMBER]).unwrap(), PledgePartyConfirmationNoop);
        assert_eq!(read_set_dismiss_pledge(&[OP_SET_DISMISS_PLEDGE]).unwrap(), PledgePartyConfirmationNoop);
        assert_eq!(read_set_oust_party_member(&[OP_SET_OUST_PARTY_MEMBER]).unwrap(), PledgePartyConfirmationNoop);
        assert_eq!(read_set_dismiss_party(&[OP_SET_DISMISS_PARTY]).unwrap(), PledgePartyConfirmationNoop);
        assert_eq!(read_attack_out_of_range(&[OP_ATTACK_OUT_OF_RANGE]).unwrap(), AttackFeedbackNoop);
        assert_eq!(read_attack_in_cool_time(&[OP_ATTACK_IN_COOL_TIME]).unwrap(), AttackFeedbackNoop);
        assert_eq!(read_attack_dead_target(&[OP_ATTACK_DEAD_TARGET]).unwrap(), AttackFeedbackNoop);
        assert_eq!(read_validate_location_in_vehicle(&[OP_VALIDATE_LOCATION_IN_VEHICLE, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0]).unwrap().values, [1, 2, 3, 4, 5, 6]);
        assert!(read_validate_location_in_vehicle(&[OP_VALIDATE_LOCATION_IN_VEHICLE]).is_err());
        assert_eq!(read_gm_hide(&[OP_GM_HIDE, 7, 0, 0, 0]).unwrap(), GmHide { object_id: 7 });
        assert!(read_gm_hide(&[OP_GM_HIDE]).is_err());
        assert_eq!(read_join_alliance(&[OP_JOIN_ALLIANCE, 4, 0, 0, 0]).unwrap(), AllianceIdResult { alliance_id: 4 });
        assert_eq!(read_withdraw_alliance(&[OP_WITHDRAW_ALLIANCE, 4, 0, 0, 0]).unwrap(), AllianceIdResult { alliance_id: 4 });
        assert_eq!(read_oust_alliance_member_pledge(&[OP_OUST_ALLIANCE_MEMBER_PLEDGE, 4, 0, 0, 0]).unwrap(), AllianceIdResult { alliance_id: 4 });
        assert_eq!(read_dismiss_alliance(&[OP_DISMISS_ALLIANCE, 4, 0, 0, 0]).unwrap(), AllianceIdResult { alliance_id: 4 });
        assert_eq!(read_set_alliance_crest(&[OP_SET_ALLIANCE_CREST]).unwrap(), AllianceNoop);
        assert_eq!(read_alliance_info(&[OP_ALLIANCE_INFO]).unwrap(), AllianceNoop);
        assert_eq!(read_request_time_check(&[OP_REQUEST_TIME_CHECK, 9, 0, 0, 0]).unwrap(), RequestTimeCheck { value: 9 });
        assert!(read_request_time_check(&[OP_REQUEST_TIME_CHECK]).is_err());
        let start = [OP_START_ALLIANCE_WAR, b'A', 0, b'l', 0, b'l', 0, b'y', 0, 0, 0, b'E', 0, b'n', 0, b'e', 0, b'm', 0, b'y', 0, 0, 0];
        assert_eq!(read_start_alliance_war(&start).unwrap(), AllianceWar { first: "Ally".into(), second: "Enemy".into() });
        assert_eq!(read_stop_alliance_war(&[OP_STOP_ALLIANCE_WAR, b'A', 0, 0, 0, b'E', 0, 0, 0]).unwrap().first, "A");
        assert_eq!(read_client_action(&ds(OP_CLIENT_ACTION, &[17])).unwrap(), ClientAction { action: 17 });
        assert!(read_client_action(&[OP_CLIENT_ACTION]).is_err());
        let mut pledge = Writer::new();
        pledge.write_c(OP_PLEDGE_EXTENDED_INFO);
        pledge.write_s("lead");
        pledge.write_d(1);
        pledge.write_d(2);
        pledge.write_s("middle");
        for value in 3..11 {
            pledge.write_d(value);
        }
        pledge.write_s("tail");
        pledge.write_d(11);
        let info = read_pledge_extended_info(&pledge.into_bytes()).unwrap();
        assert_eq!(info.leading_text, "lead");
        assert_eq!(info.leading_values, [1, 2]);
        assert_eq!(info.middle_text, "middle");
        assert_eq!(info.middle_values, [3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(info.trailing_text, "tail");
        assert_eq!(info.trailing_value, 11);
        assert_eq!(read_new_character_fail(&[OP_NEW_CHARACTER_FAIL]).unwrap(), NewCharacterFailNoop);
        assert_eq!(read_revive_request(&[OP_REVIVE_REQUEST]).unwrap(), ReviveRequestNoop);
        assert_eq!(read_new_character_fail(&[OP_NEW_CHARACTER_FAIL, 1]).unwrap(), NewCharacterFailNoop);
        assert_eq!(read_revive_request(&[OP_REVIVE_REQUEST, 1]).unwrap(), ReviveRequestNoop);
        assert_eq!(
            read_friend_add_request_result(&ds(OP_FRIEND_ADD_REQUEST_RESULT, &[7])).unwrap(),
            FriendAddRequestResult { result: 7 }
        );
        assert!(read_friend_add_request_result(&[OP_FRIEND_ADD_REQUEST_RESULT]).is_err());
        assert_eq!(read_magic_list(&[OP_MAGIC_LIST]).unwrap(), MagicListNoop);
        assert_eq!(read_friend_add(&[OP_FRIEND_ADD]).unwrap(), FriendNoop);
        assert_eq!(read_friend_remove(&[OP_FRIEND_REMOVE]).unwrap(), FriendNoop);
        assert_eq!(read_friend_list_noop(&[OP_FRIEND_LIST_LEGACY]).unwrap(), FriendNoop);
        assert_eq!(read_friend_status_noop(&[OP_FRIEND_STATUS]).unwrap(), FriendNoop);
        assert_eq!(read_magic_list(&[OP_MAGIC_LIST, 1]).unwrap(), MagicListNoop);
        assert_eq!(read_friend_status_noop(&[OP_FRIEND_STATUS, 1]).unwrap(), FriendNoop);
        assert_eq!(
            read_radar_control(&ds(OP_RADAR_CONTROL, &[0, 1, 2, 3, 4])).unwrap(),
            RadarControl { show: 0, kind: 1, x: 2, y: 3, z: 4 }
        );
        assert!(read_radar_control(&ds(OP_RADAR_CONTROL, &[0, 1, 2, 3])).is_err());
        assert_eq!(
            read_show_radar(&ds(OP_SHOW_RADAR, &[1, 2, 3, 4, 5])).unwrap(),
            ShowRadar { first: 1, second: 2, third: 3, fourth: 4, fifth: 5 }
        );
        assert_eq!(
            read_delete_radar(&ds(OP_DELETE_RADAR, &[6, 7, 8])).unwrap(),
            DeleteRadar { first: 6, second: 7, third: 8 }
        );
        assert!(read_show_radar(&ds(OP_SHOW_RADAR, &[1, 2, 3, 4])).is_err());
        assert_eq!(read_delete_radar(&ds(OP_DELETE_RADAR, &[1, 2, 3, 4])).unwrap(), DeleteRadar { first: 1, second: 2, third: 3 });
        let mut w = Writer::new();
        w.write_c(OP_SHOP_PREVIEW_LIST);
        w.write_c(0xc0);
        w.write_c(0x13);
        w.write_c(0);
        w.write_c(0);
        w.write_d(5000);
        w.write_d(3);
        w.write_h(1);
        w.write_d(2);
        w.write_h(1);
        w.write_h(0x80);
        w.write_d(10);
        let body = w.into_bytes();
        let list = read_shop_preview_list(&body).unwrap();
        assert_eq!((list.header, list.money, list.list_id), (0x13c0, 5000, 3));
        assert_eq!(list.items, vec![PreviewItem { item_id: 2, type2: 1, body_part: 0x80, price: 10 }]);
        assert!(read_shop_preview_list(&body[..body.len() - 1]).is_err());
        let info =
            read_shop_preview_info(&ds(OP_SHOP_PREVIEW_INFO, &[16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]))
                .unwrap();
        assert_eq!(info.slots[0], 16);
        assert_eq!(info.slots[15], 15);
        assert!(read_shop_preview_info(&ds(OP_SHOP_PREVIEW_INFO, &[16, 1])).is_err());
        let mut w = Writer::new();
        w.write_c(OP_FRIEND_LIST);
        w.write_h(2);
        w.write_h(0);
        w.write_d(11);
        w.write_s("A");
        w.write_d(1);
        w.write_h(0);
        w.write_h(0);
        w.write_d(12);
        w.write_s("B");
        w.write_d(0);
        w.write_h(0);
        let lisvus = w.into_bytes();
        let list = read_friend_list(&lisvus).unwrap();
        assert_eq!(list.friends.len(), 2);
        assert_eq!(list.friends[0], Friend { object_id: 11, name: "A".into(), online: true, tail: 0 });
        assert_eq!(list.friends[1], Friend { object_id: 12, name: "B".into(), online: false, tail: 0 });
        assert!(read_friend_list(&lisvus[..lisvus.len() - 8]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_FRIEND_RECV_MSG);
        w.write_d(0);
        w.write_s("Eu");
        w.write_s("Luzal");
        w.write_s("oi");
        let body = w.into_bytes();
        assert_eq!(
            read_friend_recv_msg(&body).unwrap(),
            FriendRecvMsg { unknown: 0, receiver: "Eu".into(), sender: "Luzal".into(), message: "oi".into() }
        );
        assert!(read_friend_recv_msg(&body[..body.len() - 2]).is_err());
    }

    #[test]
    fn window_encoders_follow_the_engine_formats() {
        assert_eq!(request_trade(7), ds(OP_REQUEST_TRADE, &[7]));
        assert_eq!(request_add_trade_item(1, 1001, 2), ds(OP_REQUEST_ADD_TRADE_ITEM, &[1, 1001, 2]));
        assert_eq!(request_trade_done(true), ds(OP_REQUEST_TRADE_DONE, &[1]));
        let orders = [ItemOrder { object_id: 1001, item_id: 57, count: 2 }];
        assert_eq!(request_sell_item(9, &orders), ds(OP_REQUEST_SELL_ITEM, &[9, 1, 1001, 57, 2]));
        assert_eq!(request_buy_item(9, &[(57, 2)]), ds(OP_REQUEST_BUY_ITEM, &[9, 1, 57, 2]));
        assert_eq!(request_join_pledge(7), ds(OP_REQUEST_JOIN_PLEDGE, &[7]));
        assert_eq!(request_withdrawal_pledge(), [OP_REQUEST_WITHDRAWAL_PLEDGE]);
        assert_eq!(request_oust_pledge_member("L"), [OP_REQUEST_OUST_PLEDGE_MEMBER, b'L', 0, 0, 0]);
        assert_eq!(send_warehouse_deposit_list(&[(1001, 2)]), ds(OP_SEND_WAREHOUSE_DEPOSIT_LIST, &[1, 1001, 2]));
        assert_eq!(send_warehouse_withdraw_list(&[(1001, 2)]), ds(OP_SEND_WAREHOUSE_WITHDRAW_LIST, &[1, 1001, 2]));
        assert_eq!(request_pledge_member_list(), [OP_REQUEST_PLEDGE_MEMBER_LIST]);
        assert_eq!(request_set_pledge_crest(&[1, 2]), [OP_REQUEST_SET_PLEDGE_CREST, 2, 0, 0, 0, 1, 2]);
        assert_eq!(request_set_ally_crest(&[1]), [OP_REQUEST_SET_ALLY_CREST, 1, 0, 0, 0, 1]);
        assert_eq!(request_enchant_item(1001), ds(OP_REQUEST_ENCHANT_ITEM, &[1001]));
        assert_eq!(request_quest_abort(101), ds(OP_REQUEST_QUEST_ABORT, &[101]));
        assert_eq!(request_pledge_crest(5), ds(OP_REQUEST_PLEDGE_CREST, &[5]));
        assert_eq!(request_ally_crest(6), ds(OP_REQUEST_ALLY_CREST, &[6]));
        assert_eq!(request_restart_point(1), ds(OP_REQUEST_RESTART_POINT, &[1]));
        assert_eq!(request_crystallize_item(1001, 1), ds(OP_REQUEST_CRYSTALLIZE_ITEM, &[1001, 1]));
        assert_eq!(request_private_store_manage_sell(), [OP_REQUEST_PRIVATE_STORE_MANAGE_SELL]);
        assert_eq!(request_private_store_manage_cancel_sell(), [OP_REQUEST_PRIVATE_STORE_MANAGE_CANCEL_SELL]);
        assert_eq!(
            set_private_store_list_sell(true, &[StoreOffer { object_id: 1001, count: 1, price: 100 }]),
            ds(OP_SET_PRIVATE_STORE_LIST_SELL, &[1, 1, 1001, 1, 100])
        );
        assert_eq!(request_private_store_quit_sell(), [OP_REQUEST_PRIVATE_STORE_QUIT_SELL]);
        assert_eq!(request_private_store_list_sell(7), ds(OP_REQUEST_PRIVATE_STORE_LIST_SELL, &[7]));
        assert_eq!(set_private_store_msg_sell("x"), [OP_SET_PRIVATE_STORE_MSG_SELL, b'x', 0, 0, 0]);
        let store_orders = [StoreBuyOrder { object_id: 1001, count: 2, price: 500 }];
        assert_eq!(request_private_store_buy(7, &store_orders), ds(OP_REQUEST_PRIVATE_STORE_BUY, &[7, 1, 1001, 2, 500]));
        assert_eq!(request_tutorial_link_html("l"), [OP_REQUEST_TUTORIAL_LINK_HTML, b'l', 0, 0, 0]);
        assert_eq!(request_tutorial_pass_cmd("c"), [OP_REQUEST_TUTORIAL_PASS_CMD, b'c', 0, 0, 0]);
        assert_eq!(request_tutorial_question_mark(1), ds(OP_REQUEST_TUTORIAL_QUESTION_MARK, &[1]));
        assert_eq!(request_tutorial_client_event(8), ds(OP_REQUEST_TUTORIAL_CLIENT_EVENT, &[8]));
        assert_eq!(request_join_ally(7), ds(OP_REQUEST_JOIN_ALLY, &[7]));
        assert_eq!(request_ally_leave(), [OP_REQUEST_ALLY_LEAVE]);
        assert_eq!(request_ally_dismiss("C"), [OP_REQUEST_ALLY_DISMISS, b'C', 0, 0, 0]);
        assert_eq!(request_dismiss_ally(), [OP_REQUEST_DISMISS_ALLY]);
        assert_eq!(request_ally_info(), [OP_REQUEST_ALLY_INFO]);
        assert_eq!(request_pet_get_item(900), ds(OP_REQUEST_PET_GET_ITEM, &[900]));
        assert_eq!(request_private_store_manage_buy(), [OP_REQUEST_PRIVATE_STORE_MANAGE_BUY]);
        assert_eq!(request_private_store_manage_cancel_buy(), [OP_REQUEST_PRIVATE_STORE_MANAGE_CANCEL_BUY]);
        let buy =
            set_private_store_list_buy(&[BuyOffer { item_id: 57, enchant: 1, custom_type2: 0, count: 2, price: 3 }]);
        assert_eq!(
            buy,
            [vec![OP_SET_PRIVATE_STORE_LIST_BUY, 1, 0, 0, 0, 57, 0, 0, 0, 1, 0, 0, 0], ds(0, &[2, 3])[1..].to_vec()]
                .concat()
        );
        assert_eq!(request_private_store_quit_buy(), [OP_REQUEST_PRIVATE_STORE_QUIT_BUY]);
        assert_eq!(request_private_store_list_buy(7), ds(OP_REQUEST_PRIVATE_STORE_LIST_BUY, &[7]));
        assert_eq!(set_private_store_msg_buy("x"), [OP_SET_PRIVATE_STORE_MSG_BUY, b'x', 0, 0, 0]);
        let sell = request_private_store_sell(
            7,
            &[SellToStoreItem { object_id: 1001, item_id: 57, enchant: 0, custom_type2: 0, count: 1, price: 5 }],
        );
        assert_eq!(sell.len(), 1 + 4 + 4 + 4 + 4 + 2 + 2 + 4 + 4);
        assert_eq!(sell[0], OP_REQUEST_PRIVATE_STORE_SELL);
        assert_eq!(request_package_sendable_item_list(9), ds(OP_REQUEST_PACKAGE_SENDABLE_ITEM_LIST, &[9]));
        assert_eq!(request_package_send(9, &[(1001, 1)]), ds(OP_REQUEST_PACKAGE_SEND, &[9, 1, 1001, 1]));
        assert_eq!(multi_sell_choose(1, 2, 3), ds(OP_MULTI_SELL_CHOOSE, &[1, 2, 3]));
        assert_eq!(request_user_command(3), ds(OP_REQUEST_USER_COMMAND, &[3]));
        assert_eq!(request_henna_list(0), ds(OP_REQUEST_HENNA_LIST, &[0]));
        assert_eq!(request_henna_item_info(1), ds(OP_REQUEST_HENNA_ITEM_INFO, &[1]));
        assert_eq!(request_henna_equip(1), ds(OP_REQUEST_HENNA_EQUIP, &[1]));
        assert_eq!(request_henna_unequip_list(0), ds(OP_REQUEST_HENNA_UNEQUIP_LIST, &[0]));
        assert_eq!(request_henna_unequip_info(145), ds(OP_REQUEST_HENNA_UNEQUIP_INFO, &[145]));
        assert_eq!(request_henna_unequip(145), ds(OP_REQUEST_HENNA_UNEQUIP, &[145]));
        let powers = [0x5a; PLEDGE_POWER_BYTES];
        for action in [1, 2] {
            assert_eq!(request_pledge_power(7, action, &powers), ds(OP_REQUEST_PLEDGE_POWER, &[7, action]));
        }
        let mut write = ds(OP_REQUEST_PLEDGE_POWER, &[7, 3]);
        write.extend(powers);
        assert_eq!(request_pledge_power(7, 3, &powers), write);
        let m = super::super::Macro {
            id: 1000,
            name: "Buff".into(),
            description: String::new(),
            acronym: "BF".into(),
            icon: 1,
            commands: vec![super::super::MacroCommand {
                index: 1,
                kind: 3,
                skill_id: 0,
                shortcut_id: 5,
                command: "/sit".into(),
            }],
        };
        let body = request_make_macro(&m);
        assert_eq!(body[0], OP_REQUEST_MAKE_MACRO);
        assert_eq!(body.len(), 1 + 4 + 10 + 2 + 6 + 1 + 1 + (1 + 1 + 4 + 1 + 10));
        assert_eq!(request_delete_macro(1000), ds(OP_REQUEST_DELETE_MACRO, &[1000]));
        assert_eq!(request_procure_crop(1, &orders), ds(OP_REQUEST_PROCURE_CROP, &[1, 1, 1001, 57, 2]));
        assert_eq!(request_preview_item(0, 3, &[2]), ds(OP_REQUEST_PREVIEW_ITEM, &[0, 3, 1, 2]));
        assert_eq!(request_send_friend_msg("m", "L"), [OP_REQUEST_SEND_FRIEND_MSG, b'm', 0, 0, 0, b'L', 0, 0, 0]);
        assert_eq!(request_show_minimap(), [OP_REQUEST_SHOW_MINIMAP]);
        assert_eq!(request_friend_invite("L"), [OP_REQUEST_FRIEND_INVITE, b'L', 0, 0, 0]);
        assert_eq!(request_answer_friend_invite(true), ds(OP_REQUEST_ANSWER_FRIEND_INVITE, &[1]));
        assert_eq!(request_friend_list(), [OP_REQUEST_FRIEND_LIST]);
        assert_eq!(request_friend_del("L"), [OP_REQUEST_FRIEND_DEL, b'L', 0, 0, 0]);
        assert_eq!(request_give_nick_name("L", "T"), [OP_REQUEST_GIVE_NICK_NAME, b'L', 0, 0, 0, b'T', 0, 0, 0]);
        assert_eq!(request_show_board(0), ds(OP_REQUEST_SHOW_BOARD, &[0]));
        assert_eq!(
            request_bbs_write(&["a", "", "", "", "", ""]),
            [OP_REQUEST_BBS_WRITE, b'a', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(request_party_match_list(40, 0, 1), ds(OP_REQUEST_PARTY_MATCH_LIST, &[40, 0, 1]));
        assert_eq!(
            request_manage_party_room(1, 9, 20, 40, 1, "S"),
            [ds(OP_REQUEST_MANAGE_PARTY_ROOM, &[1, 9, 20, 40, 1]), vec![b'S', 0, 0, 0]].concat()
        );
        assert_eq!(request_join_party_room(1, 0), ds(OP_REQUEST_JOIN_PARTY_ROOM, &[1, 0]));
    }
}
