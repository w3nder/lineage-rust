use super::{NPC_TYPE_OFFSET, counted};
use crate::wire::{Reader, Result, WireError, Writer};

pub const OP_ATTACK: u8 = 0x05;
pub const OP_DROP_ITEM: u8 = 0x0c;
pub const OP_GET_ITEM: u8 = 0x0d;
pub const OP_SUN_RISE: u8 = 0x1c;
pub const OP_SUN_SET: u8 = 0x1d;
pub const OP_AUTO_ATTACK_START: u8 = 0x2b;
pub const OP_AUTO_ATTACK_STOP: u8 = 0x2c;
pub const OP_ASK_JOIN_PLEDGE: u8 = 0x32;
pub const OP_ASK_JOIN_PARTY: u8 = 0x39;
pub const OP_DOOR_INFO: u8 = 0x4c;
pub const OP_DOOR_STATUS_UPDATE: u8 = 0x4d;
pub const OP_PARTY_SMALL_WINDOW_ALL: u8 = 0x4e;
pub const OP_PARTY_SMALL_WINDOW_ADD: u8 = 0x4f;
pub const OP_PARTY_SMALL_WINDOW_DELETE_ALL: u8 = 0x50;
pub const OP_PARTY_SMALL_WINDOW_DELETE: u8 = 0x51;
pub const OP_PARTY_SMALL_WINDOW_UPDATE: u8 = 0x52;
pub const OP_VEHICLE_INFO: u8 = 0x59;
pub const OP_VEHICLE_DEPARTURE: u8 = 0x5a;
pub const OP_GET_ON_VEHICLE: u8 = 0x5c;
pub const OP_GET_OFF_VEHICLE: u8 = 0x5d;
pub const OP_START_ROTATING: u8 = 0x62;
pub const OP_FINISH_ROTATING: u8 = 0x63;
pub const OP_SETUP_GAUGE: u8 = 0x6d;
pub const OP_MOVE_TO_LOCATION_IN_VEHICLE: u8 = 0x71;
pub const OP_STOP_MOVE_IN_VEHICLE: u8 = 0x72;
pub const OP_RIDE: u8 = 0x86;
pub const OP_SERVER_OBJECT_INFO: u8 = 0x8c;
pub const OP_PLAY_SOUND: u8 = 0x98;
pub const OP_STATIC_OBJECT: u8 = 0x99;
pub const OP_PARTY_MEMBER_POSITION: u8 = 0xa7;
pub const OP_ASK_JOIN_ALLIANCE: u8 = 0xa8;
pub const OP_PET_STATUS_SHOW: u8 = 0xb0;
pub const OP_PET_INFO: u8 = 0xb1;
pub const OP_PET_STATUS_UPDATE: u8 = 0xb5;
pub const OP_PET_DELETE: u8 = 0xb6;
pub const OP_VEHICLE_START: u8 = 0xba;
pub const OP_EARTHQUAKE: u8 = 0xc4;
pub const OP_FLY_TO_LOCATION: u8 = 0xc5;
pub const OP_SPECIAL_CAMERA: u8 = 0xc7;
pub const OP_NICKNAME_CHANGED: u8 = 0xcc;
pub const OP_SET_SUMMON_REMAIN_TIME: u8 = 0xd1;
pub const OP_DICE: u8 = 0xd4;
pub const OP_CHAIR_SIT: u8 = 0xe1;
pub const OP_CONFIRM_DLG: u8 = 0xed;
pub const OP_PARTY_SPELLED: u8 = 0xee;
pub const OP_SHORT_BUFF_STATUS_UPDATE: u8 = 0xf4;

pub const OP_ATTACK_REQUEST: u8 = 0x0a;
pub const OP_REQUEST_DROP_ITEM: u8 = 0x12;
pub const OP_REQUEST_SOCIAL_ACTION: u8 = 0x1b;
pub const OP_REQUEST_CHANGE_MOVE_TYPE: u8 = 0x1c;
pub const OP_REQUEST_CHANGE_WAIT_TYPE: u8 = 0x1d;
pub const OP_REQUEST_ANSWER_JOIN_PLEDGE: u8 = 0x25;
pub const OP_REQUEST_JOIN_PARTY: u8 = 0x29;
pub const OP_REQUEST_ANSWER_JOIN_PARTY: u8 = 0x2a;
pub const OP_REQUEST_WITHDRAWAL_PARTY: u8 = 0x2b;
pub const OP_REQUEST_OUST_PARTY_MEMBER: u8 = 0x2c;
pub const OP_CANNOT_MOVE_ANYMORE: u8 = 0x36;
pub const OP_REQUEST_SKILL_LIST: u8 = 0x3f;
pub const OP_REQUEST_GET_ON_VEHICLE: u8 = 0x42;
pub const OP_REQUEST_GET_OFF_VEHICLE: u8 = 0x43;
pub const OP_ANSWER_TRADE_REQUEST: u8 = 0x44;
pub const OP_START_ROTATING_CLIENT: u8 = 0x4a;
pub const OP_FINISH_ROTATING_CLIENT: u8 = 0x4b;
pub const OP_CANNOT_MOVE_ANYMORE_IN_VEHICLE: u8 = 0x5d;
pub const OP_REQUEST_MOVE_TO_LOCATION_IN_VEHICLE: u8 = 0x5c;
pub const OP_REQUEST_QUEST_LIST: u8 = 0x63;
pub const OP_REQUEST_PLEDGE_INFO: u8 = 0x66;
pub const OP_REQUEST_ACQUIRE_SKILL_INFO: u8 = 0x6b;
pub const OP_REQUEST_ACQUIRE_SKILL: u8 = 0x6c;
pub const OP_REQUEST_ANSWER_JOIN_ALLY: u8 = 0x83;
pub const OP_REQUEST_CHANGE_PET_NAME: u8 = 0x89;
pub const OP_REQUEST_PET_USE_ITEM: u8 = 0x8a;
pub const OP_REQUEST_GIVE_ITEM_TO_PET: u8 = 0x8b;
pub const OP_REQUEST_GET_ITEM_FROM_PET: u8 = 0x8c;
pub const OP_REQUEST_BLOCK: u8 = 0xa0;
pub const OP_DLG_ANSWER: u8 = 0xc5;
pub const OP_REQUEST_RELOAD: u8 = 0xcf;
pub const OP_EXTENDED_CLIENT: u8 = 0xd0;
pub const EX_CHANGE_PARTY_LEADER: u16 = 0x04;
pub const EX_AUTO_SOUL_SHOT: u16 = 0x05;

pub const BLOCK_ADD: i32 = 0;
pub const BLOCK_REMOVE: i32 = 1;
pub const BLOCK_LIST: i32 = 2;
pub const BLOCK_ALL: i32 = 3;
pub const BLOCK_NONE: i32 = 4;

pub const HIT_SOULSHOT: u8 = 0x10;
pub const HIT_CRITICAL: u8 = 0x20;
pub const HIT_SHIELD: u8 = 0x40;
pub const HIT_MISS: u8 = 0x80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    pub target_id: i32,
    pub damage: i32,
    pub flags: u8,
}

impl Hit {
    pub fn miss(&self) -> bool {
        self.flags & HIT_MISS != 0
    }

    pub fn critical(&self) -> bool {
        self.flags & HIT_CRITICAL != 0
    }

    pub fn shield(&self) -> bool {
        self.flags & HIT_SHIELD != 0
    }

    pub fn soulshot(&self) -> bool {
        self.flags & HIT_SOULSHOT != 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attack {
    pub attacker_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub hits: Vec<Hit>,
}

pub fn read_attack(body: &[u8]) -> Result<Attack> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let attacker_id = r.read_d()?;
    let target_id = r.read_d()?;
    let damage = r.read_d()?;
    let flags = r.read_c()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let declared = r.read_h()?;
    let count = counted(&r, declared as i32, 9)?;
    let mut hits = Vec::with_capacity(count + 1);
    hits.push(Hit { target_id, damage, flags });
    for _ in 0..count {
        let target_id = r.read_d()?;
        let damage = r.read_d()?;
        let flags = r.read_c()?;
        hits.push(Hit { target_id, damage, flags });
    }
    Ok(Attack { attacker_id, x, y, z, hits })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropItem {
    pub player_id: i32,
    pub object_id: i32,
    pub item_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub stackable: bool,
    pub count: i32,
    pub heading: i32,
}

pub fn read_drop_item(body: &[u8]) -> Result<DropItem> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let player_id = r.read_d()?;
    let object_id = r.read_d()?;
    let item_id = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let stackable = r.read_d()? != 0;
    let count = r.read_d()?;
    let heading = r.read_d()?;
    Ok(DropItem { player_id, object_id, item_id, x, y, z, stackable, count, heading })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetItem {
    pub player_id: i32,
    pub object_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub fn read_get_item(body: &[u8]) -> Result<GetItem> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let player_id = r.read_d()?;
    let object_id = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    Ok(GetItem { player_id, object_id, x, y, z })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SunRise;

pub fn read_sun_rise(body: &[u8]) -> Result<SunRise> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(SunRise)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SunSet;

pub fn read_sun_set(body: &[u8]) -> Result<SunSet> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(SunSet)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoAttackStart {
    pub target_id: i32,
}

pub fn read_auto_attack_start(body: &[u8]) -> Result<AutoAttackStart> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let target_id = r.read_d()?;
    Ok(AutoAttackStart { target_id })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoAttackStop {
    pub target_id: i32,
}

pub fn read_auto_attack_stop(body: &[u8]) -> Result<AutoAttackStop> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let target_id = r.read_d()?;
    Ok(AutoAttackStop { target_id })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskJoinPledge {
    pub requestor_id: i32,
    pub pledge_name: String,
}

pub fn read_ask_join_pledge(body: &[u8]) -> Result<AskJoinPledge> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let requestor_id = r.read_d()?;
    let pledge_name = r.read_s()?;
    Ok(AskJoinPledge { requestor_id, pledge_name })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskJoinParty {
    pub requestor_name: String,
    pub item_distribution: i32,
}

pub fn read_ask_join_party(body: &[u8]) -> Result<AskJoinParty> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let requestor_name = r.read_s()?;
    let item_distribution = r.read_d()?;
    Ok(AskJoinParty { requestor_name, item_distribution })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorInfo {
    pub object_id: i32,
    pub door_id: i32,
}

pub fn read_door_info(body: &[u8]) -> Result<DoorInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let door_id = r.read_d()?;
    Ok(DoorInfo { object_id, door_id })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorStatusUpdate {
    pub object_id: i32,
    pub closed: bool,
    pub damage: i32,
    pub show_hp: bool,
    pub door_id: i32,
    pub max_hp: i32,
    pub hp: i32,
}

pub fn read_door_status_update(body: &[u8]) -> Result<DoorStatusUpdate> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let closed = r.read_d()? != 0;
    let damage = r.read_d()?;
    let show_hp = r.read_d()? != 0;
    let door_id = r.read_d()?;
    let max_hp = r.read_d()?;
    let hp = r.read_d()?;
    Ok(DoorStatusUpdate { object_id, closed, damage, show_hp, door_id, max_hp, hp })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyMember {
    pub object_id: i32,
    pub name: String,
    pub cp: i32,
    pub max_cp: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub level: i32,
    pub class_id: i32,
}

fn read_party_member(r: &mut Reader) -> Result<PartyMember> {
    let object_id = r.read_d()?;
    let name = r.read_s()?;
    let cp = r.read_d()?;
    let max_cp = r.read_d()?;
    let hp = r.read_d()?;
    let max_hp = r.read_d()?;
    let mp = r.read_d()?;
    let max_mp = r.read_d()?;
    let level = r.read_d()?;
    let class_id = r.read_d()?;
    Ok(PartyMember { object_id, name, cp, max_cp, hp, max_hp, mp, max_mp, level, class_id })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartySmallWindowAll {
    pub leader_id: i32,
    pub loot_distribution: i32,
    pub members: Vec<PartyMember>,
}

pub fn read_party_small_window_all(body: &[u8]) -> Result<PartySmallWindowAll> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let leader_id = r.read_d()?;
    let loot_distribution = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 4 + 2 + 8 * 4 + 8)?;
    let mut members = Vec::with_capacity(count);
    for _ in 0..count {
        let member = read_party_member(&mut r)?;
        r.read_d()?;
        r.read_d()?;
        members.push(member);
    }
    Ok(PartySmallWindowAll { leader_id, loot_distribution, members })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartySmallWindowAdd {
    pub leader_id: i32,
    pub loot_distribution: i32,
    pub member: PartyMember,
}

pub fn read_party_small_window_add(body: &[u8]) -> Result<PartySmallWindowAdd> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let leader_id = r.read_d()?;
    let loot_distribution = r.read_d()?;
    let member = read_party_member(&mut r)?;
    r.read_d()?;
    r.read_d()?;
    Ok(PartySmallWindowAdd { leader_id, loot_distribution, member })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartySmallWindowDeleteAll;

pub fn read_party_small_window_delete_all(body: &[u8]) -> Result<PartySmallWindowDeleteAll> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(PartySmallWindowDeleteAll)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartySmallWindowDelete {
    pub object_id: i32,
    pub name: String,
}

pub fn read_party_small_window_delete(body: &[u8]) -> Result<PartySmallWindowDelete> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let name = r.read_s()?;
    Ok(PartySmallWindowDelete { object_id, name })
}

pub fn read_party_small_window_update(body: &[u8]) -> Result<PartyMember> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let member = read_party_member(&mut r)?;
    Ok(member)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VehicleInfo {
    pub object_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub heading: i32,
}

pub fn read_vehicle_info(body: &[u8]) -> Result<VehicleInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let heading = r.read_d()?;
    Ok(VehicleInfo { object_id, x, y, z, heading })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VehicleDeparture {
    pub object_id: i32,
    pub move_speed: i32,
    pub rotation_speed: i32,
    pub to_x: i32,
    pub to_y: i32,
    pub to_z: i32,
}

pub fn read_vehicle_departure(body: &[u8]) -> Result<VehicleDeparture> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let move_speed = r.read_d()?;
    let rotation_speed = r.read_d()?;
    let to_x = r.read_d()?;
    let to_y = r.read_d()?;
    let to_z = r.read_d()?;
    Ok(VehicleDeparture { object_id, move_speed, rotation_speed, to_x, to_y, to_z })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VehicleBoarding {
    pub object_id: i32,
    pub vehicle_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

fn read_vehicle_boarding(body: &[u8]) -> Result<VehicleBoarding> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let vehicle_id = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    Ok(VehicleBoarding { object_id, vehicle_id, x, y, z })
}

pub fn read_get_on_vehicle(body: &[u8]) -> Result<VehicleBoarding> {
    read_vehicle_boarding(body)
}

pub fn read_get_off_vehicle(body: &[u8]) -> Result<VehicleBoarding> {
    read_vehicle_boarding(body)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartRotating {
    pub object_id: i32,
    pub degree: i32,
    pub side: i32,
    pub speed: i32,
}

pub fn read_start_rotating(body: &[u8]) -> Result<StartRotating> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let degree = r.read_d()?;
    let side = r.read_d()?;
    let speed = r.read_d()?;
    Ok(StartRotating { object_id, degree, side, speed })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinishRotating {
    pub object_id: i32,
    pub degree: i32,
    pub speed: i32,
}

pub fn read_finish_rotating(body: &[u8]) -> Result<FinishRotating> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let degree = r.read_d()?;
    let speed = r.read_d()?;
    Ok(FinishRotating { object_id, degree, speed })
}

pub const GAUGE_BLUE: i32 = 0;
pub const GAUGE_RED: i32 = 1;
pub const GAUGE_CYAN: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetupGauge {
    pub color: i32,
    pub time: i32,
    pub max_time: i32,
}

pub fn read_setup_gauge(body: &[u8]) -> Result<SetupGauge> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let color = r.read_d()?;
    let time = r.read_d()?;
    let max_time = r.read_d()?;
    Ok(SetupGauge { color, time, max_time })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveToLocationInVehicle {
    pub object_id: i32,
    pub vehicle_id: i32,
    pub to_x: i32,
    pub to_y: i32,
    pub to_z: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub fn read_move_to_location_in_vehicle(body: &[u8]) -> Result<MoveToLocationInVehicle> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let vehicle_id = r.read_d()?;
    let to_x = r.read_d()?;
    let to_y = r.read_d()?;
    let to_z = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    Ok(MoveToLocationInVehicle { object_id, vehicle_id, to_x, to_y, to_z, x, y, z })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopMoveInVehicle {
    pub object_id: i32,
    pub vehicle_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub heading: i32,
}

pub fn read_stop_move_in_vehicle(body: &[u8]) -> Result<StopMoveInVehicle> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let vehicle_id = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let heading = r.read_d()?;
    Ok(StopMoveInVehicle { object_id, vehicle_id, x, y, z, heading })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ride {
    pub object_id: i32,
    pub mounted: bool,
    pub ride_type: i32,
    pub ride_class_id: i32,
}

pub fn read_ride(body: &[u8]) -> Result<Ride> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let mounted = r.read_d()? != 0;
    let ride_type = r.read_d()?;
    let ride_class_id = r.read_d()?;
    Ok(Ride { object_id, mounted, ride_type, ride_class_id })
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerObjectInfo {
    pub object_id: i32,
    pub npc_id: i32,
    pub name: String,
    pub attackable: bool,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub heading: i32,
    pub speed_multiplier: f64,
    pub attack_speed_multiplier: f64,
    pub collision_radius: f64,
    pub collision_height: f64,
    pub hp: i32,
    pub max_hp: i32,
    pub object_type: i32,
    pub special_effects: i32,
}

pub fn read_server_object_info(body: &[u8]) -> Result<ServerObjectInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let npc_id = r.read_d()?.wrapping_sub(NPC_TYPE_OFFSET);
    let name = r.read_s()?;
    let attackable = r.read_d()? != 0;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let heading = r.read_d()?;
    let speed_multiplier = r.read_f()?;
    let attack_speed_multiplier = r.read_f()?;
    let collision_radius = r.read_f()?;
    let collision_height = r.read_f()?;
    let hp = r.read_d()?;
    let max_hp = r.read_d()?;
    let object_type = r.read_d()?;
    let special_effects = r.read_d()?;
    Ok(ServerObjectInfo {
        object_id,
        npc_id,
        name,
        attackable,
        x,
        y,
        z,
        heading,
        speed_multiplier,
        attack_speed_multiplier,
        collision_radius,
        collision_height,
        hp,
        max_hp,
        object_type,
        special_effects,
    })
}

pub const SOUND_MAX_CHARS: usize = 0x80 / 2 - 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaySound {
    pub kind: i32,
    pub sound_file: String,
    pub bind_to_object: bool,
    pub object_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub delay: i32,
}

pub fn read_play_sound(body: &[u8]) -> Result<PlaySound> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let kind = r.read_d()?;
    let sound_file = r.read_s()?;
    if sound_file.encode_utf16().count() > SOUND_MAX_CHARS {
        return Err(WireError::TooLong);
    }
    let bind_to_object = r.read_d()? != 0;
    let object_id = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let delay = r.read_d()?;
    Ok(PlaySound { kind, sound_file, bind_to_object, object_id, x, y, z, delay })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticObject {
    pub static_object_id: i32,
    pub object_id: i32,
}

pub fn read_static_object(body: &[u8]) -> Result<StaticObject> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let static_object_id = r.read_d()?;
    let object_id = r.read_d()?;
    Ok(StaticObject { static_object_id, object_id })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemberPosition {
    pub object_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyMemberPosition {
    pub positions: Vec<MemberPosition>,
}

pub fn read_party_member_position(body: &[u8]) -> Result<PartyMemberPosition> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 16)?;
    let mut positions = Vec::with_capacity(count);
    for _ in 0..count {
        let object_id = r.read_d()?;
        let x = r.read_d()?;
        let y = r.read_d()?;
        let z = r.read_d()?;
        positions.push(MemberPosition { object_id, x, y, z });
    }
    Ok(PartyMemberPosition { positions })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskJoinAlliance {
    pub requestor_id: i32,
    pub requestor_name: String,
    pub requestor_pledge_name: String,
    pub requestor_ally_name: String,
}

pub fn read_ask_join_alliance(body: &[u8]) -> Result<AskJoinAlliance> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let requestor_id = r.read_d()?;
    let requestor_name = r.read_s()?;
    let requestor_pledge_name = r.read_s()?;
    let requestor_ally_name = r.read_s()?;
    Ok(AskJoinAlliance { requestor_id, requestor_name, requestor_pledge_name, requestor_ally_name })
}

pub const SUMMON_PET: i32 = 2;
pub const SUMMON_SERVITOR: i32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PetStatusShow {
    pub summon_type: i32,
}

pub fn read_pet_status_show(body: &[u8]) -> Result<PetStatusShow> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let summon_type = r.read_d()?;
    Ok(PetStatusShow { summon_type })
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PetInfo {
    pub summon_type: i32,
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
    pub has_owner: bool,
    pub running: bool,
    pub in_combat: bool,
    pub alike_dead: bool,
    pub spawn_animation: u8,
    pub name: String,
    pub title: String,
    pub pvp_flag: i32,
    pub karma: i32,
    pub fed: i32,
    pub max_fed: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub sp: i32,
    pub level: i32,
    pub exp: i32,
    pub exp_this_level: i32,
    pub exp_next_level: i32,
    pub load: i32,
    pub max_load: i32,
    pub p_atk: i32,
    pub p_def: i32,
    pub m_atk: i32,
    pub m_def: i32,
    pub accuracy: i32,
    pub evasion: i32,
    pub critical: i32,
    pub speed: i32,
    pub atk_spd: i32,
    pub cast_spd: i32,
    pub abnormal_effect: i32,
    pub mountable: i32,
    pub team: u8,
    pub soulshots_per_hit: i32,
    pub spiritshots_per_hit: i32,
}

pub fn read_pet_info(body: &[u8]) -> Result<PetInfo> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let summon_type = r.read_d()?;
    let object_id = r.read_d()?;
    let npc_id = r.read_d()?.wrapping_sub(NPC_TYPE_OFFSET);
    let attackable = r.read_d()? != 0;
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
    let has_owner = r.read_c()? != 0;
    let running = r.read_c()? != 0;
    let in_combat = r.read_c()? != 0;
    let alike_dead = r.read_c()? != 0;
    let spawn_animation = r.read_c()?;
    let name = r.read_s()?;
    let title = r.read_s()?;
    r.read_d()?;
    let pvp_flag = r.read_d()?;
    let karma = r.read_d()?;
    let fed = r.read_d()?;
    let max_fed = r.read_d()?;
    let hp = r.read_d()?;
    let max_hp = r.read_d()?;
    let mp = r.read_d()?;
    let max_mp = r.read_d()?;
    let sp = r.read_d()?;
    let level = r.read_d()?;
    let exp = r.read_d()?;
    let exp_this_level = r.read_d()?;
    let exp_next_level = r.read_d()?;
    let load = r.read_d()?;
    let max_load = r.read_d()?;
    let p_atk = r.read_d()?;
    let p_def = r.read_d()?;
    let m_atk = r.read_d()?;
    let m_def = r.read_d()?;
    let accuracy = r.read_d()?;
    let evasion = r.read_d()?;
    let critical = r.read_d()?;
    let speed = r.read_d()?;
    let atk_spd = r.read_d()?;
    let cast_spd = r.read_d()?;
    let abnormal_effect = r.read_d()?;
    let mountable = r.read_d()? & 0xffff;
    r.read_c()?;
    let team = r.read_c()?;
    let soulshots_per_hit = r.read_d()?;
    let spiritshots_per_hit = r.read_d()?;
    Ok(PetInfo {
        summon_type,
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
        has_owner,
        running,
        in_combat,
        alike_dead,
        spawn_animation,
        name,
        title,
        pvp_flag,
        karma,
        fed,
        max_fed,
        hp,
        max_hp,
        mp,
        max_mp,
        sp,
        level,
        exp,
        exp_this_level,
        exp_next_level,
        load,
        max_load,
        p_atk,
        p_def,
        m_atk,
        m_def,
        accuracy,
        evasion,
        critical,
        speed,
        atk_spd,
        cast_spd,
        abnormal_effect,
        mountable,
        team,
        soulshots_per_hit,
        spiritshots_per_hit,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PetStatusUpdate {
    pub summon_type: i32,
    pub object_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub name: String,
    pub fed: i32,
    pub max_fed: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub level: i32,
    pub exp: i32,
    pub exp_this_level: i32,
    pub exp_next_level: i32,
}

pub fn read_pet_status_update(body: &[u8]) -> Result<PetStatusUpdate> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let summon_type = r.read_d()?;
    let object_id = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let name = r.read_s()?;
    let fed = r.read_d()?;
    let max_fed = r.read_d()?;
    let hp = r.read_d()?;
    let max_hp = r.read_d()?;
    let mp = r.read_d()?;
    let max_mp = r.read_d()?;
    let level = r.read_d()?;
    let exp = r.read_d()?;
    let exp_this_level = r.read_d()?;
    let exp_next_level = r.read_d()?;
    Ok(PetStatusUpdate {
        summon_type,
        object_id,
        x,
        y,
        z,
        name,
        fed,
        max_fed,
        hp,
        max_hp,
        mp,
        max_mp,
        level,
        exp,
        exp_this_level,
        exp_next_level,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PetDelete {
    pub summon_type: i32,
    pub object_id: i32,
}

pub fn read_pet_delete(body: &[u8]) -> Result<PetDelete> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let summon_type = r.read_d()?;
    let object_id = r.read_d()?;
    Ok(PetDelete { summon_type, object_id })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VehicleStart {
    pub object_id: i32,
    pub state: i32,
}

pub fn read_vehicle_start(body: &[u8]) -> Result<VehicleStart> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let state = r.read_d()?;
    Ok(VehicleStart { object_id, state })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Earthquake {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub intensity: i32,
    pub duration: i32,
}

pub fn read_earthquake(body: &[u8]) -> Result<Earthquake> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    let intensity = r.read_d()?;
    let duration = r.read_d()?;
    r.read_d()?;
    Ok(Earthquake { x, y, z, intensity, duration })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlyToLocation {
    pub object_id: i32,
    pub to_x: i32,
    pub to_y: i32,
    pub to_z: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub fn read_fly_to_location(body: &[u8]) -> Result<FlyToLocation> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let to_x = r.read_d()?;
    let to_y = r.read_d()?;
    let to_z = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    Ok(FlyToLocation { object_id, to_x, to_y, to_z, x, y, z })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecialCamera {
    pub target_id: i32,
    pub distance: i32,
    pub yaw: i32,
    pub pitch: i32,
    pub time: i32,
    pub duration: i32,
    pub turn: i32,
    pub rise: i32,
    pub widescreen: i32,
    pub unknown: i32,
}

pub fn read_special_camera(body: &[u8]) -> Result<SpecialCamera> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let target_id = r.read_d()?;
    let distance = r.read_d()?;
    let yaw = r.read_d()?;
    let pitch = r.read_d()?;
    let time = r.read_d()?;
    let duration = r.read_d()?;
    let turn = r.read_d()?;
    let rise = r.read_d()?;
    let widescreen = r.read_d()?;
    let unknown = r.read_d()?;
    Ok(SpecialCamera { target_id, distance, yaw, pitch, time, duration, turn, rise, widescreen, unknown })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NicknameChanged {
    pub object_id: i32,
    pub title: String,
}

pub fn read_nickname_changed(body: &[u8]) -> Result<NicknameChanged> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let title = r.read_s()?;
    Ok(NicknameChanged { object_id, title })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetSummonRemainTime {
    pub max_time: i32,
    pub remaining_time: i32,
}

pub fn read_set_summon_remain_time(body: &[u8]) -> Result<SetSummonRemainTime> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let max_time = r.read_d()?;
    let remaining_time = r.read_d()?;
    Ok(SetSummonRemainTime { max_time, remaining_time })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dice {
    pub player_id: i32,
    pub item_id: i32,
    pub number: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub fn read_dice(body: &[u8]) -> Result<Dice> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let player_id = r.read_d()?;
    let item_id = r.read_d()?;
    let number = r.read_d()?;
    let x = r.read_d()?;
    let y = r.read_d()?;
    let z = r.read_d()?;
    Ok(Dice { player_id, item_id, number, x, y, z })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChairSit {
    pub object_id: i32,
    pub static_object_id: i32,
}

pub fn read_chair_sit(body: &[u8]) -> Result<ChairSit> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let object_id = r.read_d()?;
    let static_object_id = r.read_d()?;
    Ok(ChairSit { object_id, static_object_id })
}

pub fn read_confirm_dlg(body: &[u8]) -> Result<super::SystemMessage> {
    super::read_system_message(body)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartyBuff {
    pub skill_id: i32,
    pub level: u16,
    pub duration: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartySpelled {
    pub summon_type: i32,
    pub object_id: i32,
    pub effects: Vec<PartyBuff>,
}

pub fn read_party_spelled(body: &[u8]) -> Result<PartySpelled> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let summon_type = r.read_d()?;
    let object_id = r.read_d()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 10)?;
    let mut effects = Vec::with_capacity(count);
    for _ in 0..count {
        let skill_id = r.read_d()?;
        let level = r.read_h()?;
        let duration = r.read_d()?;
        effects.push(PartyBuff { skill_id, level, duration });
    }
    Ok(PartySpelled { summon_type, object_id, effects })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortBuffStatusUpdate {
    pub skill_id: i32,
    pub level: i32,
    pub duration: i32,
}

pub fn read_short_buff_status_update(body: &[u8]) -> Result<ShortBuffStatusUpdate> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let skill_id = r.read_d()?;
    let level = r.read_d()?;
    let duration = r.read_d()?;
    Ok(ShortBuffStatusUpdate { skill_id, level, duration })
}

pub fn attack_request(object_id: i32, origin: [i32; 3], shift: bool) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_ATTACK_REQUEST);
    w.write_d(object_id);
    for v in origin {
        w.write_d(v);
    }
    w.write_c(shift as u8);
    w.into_bytes()
}

pub fn request_drop_item(object_id: i32, count: i32, position: [i32; 3]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_DROP_ITEM);
    w.write_d(object_id);
    w.write_d(count);
    for v in position {
        w.write_d(v);
    }
    w.into_bytes()
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

pub fn request_social_action(action: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_SOCIAL_ACTION, action)
}

pub fn request_change_move_type(running: bool) -> Vec<u8> {
    opcode_int(OP_REQUEST_CHANGE_MOVE_TYPE, running as i32)
}

pub fn request_change_wait_type(standing: bool) -> Vec<u8> {
    opcode_int(OP_REQUEST_CHANGE_WAIT_TYPE, standing as i32)
}

pub fn request_answer_join_pledge(accept: bool) -> Vec<u8> {
    opcode_int(OP_REQUEST_ANSWER_JOIN_PLEDGE, accept as i32)
}

pub fn request_join_party(name: &str, item_distribution: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_JOIN_PARTY);
    w.write_s(name);
    w.write_d(item_distribution);
    w.into_bytes()
}

pub fn request_answer_join_party(accept: bool) -> Vec<u8> {
    opcode_int(OP_REQUEST_ANSWER_JOIN_PARTY, accept as i32)
}

pub fn request_withdrawal_party() -> Vec<u8> {
    vec![OP_REQUEST_WITHDRAWAL_PARTY]
}

pub fn request_oust_party_member(name: &str) -> Vec<u8> {
    opcode_string(OP_REQUEST_OUST_PARTY_MEMBER, name)
}

pub fn cannot_move_anymore(position: [i32; 3], heading: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_CANNOT_MOVE_ANYMORE);
    for v in position {
        w.write_d(v);
    }
    w.write_d(heading);
    w.into_bytes()
}

pub fn cannot_move_anymore_in_vehicle(vehicle_id: i32, position: [i32; 3], heading: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_CANNOT_MOVE_ANYMORE_IN_VEHICLE);
    w.write_d(vehicle_id);
    for v in position {
        w.write_d(v);
    }
    w.write_d(heading);
    w.into_bytes()
}

pub fn request_get_on_vehicle(vehicle_id: i32, position: [i32; 3]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_GET_ON_VEHICLE);
    w.write_d(vehicle_id);
    for value in position {
        w.write_d(value);
    }
    w.into_bytes()
}

pub fn request_get_off_vehicle(vehicle_id: i32, position: [i32; 3]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_GET_OFF_VEHICLE);
    w.write_d(vehicle_id);
    for value in position {
        w.write_d(value);
    }
    w.into_bytes()
}

pub fn request_move_to_location_in_vehicle(
    vehicle_id: i32,
    local_target: [i32; 3],
    local_origin: [i32; 3],
) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_MOVE_TO_LOCATION_IN_VEHICLE);
    w.write_d(vehicle_id);
    for value in local_target {
        w.write_d(value);
    }
    for value in local_origin {
        w.write_d(value);
    }
    w.into_bytes()
}

pub fn request_skill_list() -> Vec<u8> {
    vec![OP_REQUEST_SKILL_LIST]
}

pub fn answer_trade_request(accept: bool) -> Vec<u8> {
    opcode_int(OP_ANSWER_TRADE_REQUEST, accept as i32)
}

pub fn start_rotating(degree: i32, side: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_START_ROTATING_CLIENT);
    w.write_d(degree);
    w.write_d(side);
    w.into_bytes()
}

pub fn finish_rotating(degree: i32, side: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_FINISH_ROTATING_CLIENT);
    w.write_d(degree);
    w.write_d(side);
    w.into_bytes()
}

pub fn request_quest_list() -> Vec<u8> {
    vec![OP_REQUEST_QUEST_LIST]
}

pub fn request_pledge_info(pledge_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_PLEDGE_INFO, pledge_id)
}

fn opcode_three_ints(opcode: u8, a: i32, b: i32, c: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(opcode);
    w.write_d(a);
    w.write_d(b);
    w.write_d(c);
    w.into_bytes()
}

pub fn request_acquire_skill_info(skill_id: i32, level: i32, mode: i32) -> Vec<u8> {
    opcode_three_ints(OP_REQUEST_ACQUIRE_SKILL_INFO, skill_id, level, mode)
}

pub fn request_acquire_skill(skill_id: i32, level: i32, mode: i32) -> Vec<u8> {
    opcode_three_ints(OP_REQUEST_ACQUIRE_SKILL, skill_id, level, mode)
}

pub fn request_answer_join_ally(accept: bool) -> Vec<u8> {
    opcode_int(OP_REQUEST_ANSWER_JOIN_ALLY, accept as i32)
}

pub fn request_change_pet_name(name: &str) -> Vec<u8> {
    opcode_string(OP_REQUEST_CHANGE_PET_NAME, name)
}

pub fn request_pet_use_item(object_id: i32) -> Vec<u8> {
    opcode_int(OP_REQUEST_PET_USE_ITEM, object_id)
}

pub fn request_give_item_to_pet(object_id: i32, count: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_GIVE_ITEM_TO_PET);
    w.write_d(object_id);
    w.write_d(count);
    w.into_bytes()
}

pub fn request_get_item_from_pet(object_id: i32, count: i32, unknown: i32) -> Vec<u8> {
    opcode_three_ints(OP_REQUEST_GET_ITEM_FROM_PET, object_id, count, unknown)
}

pub fn request_block(kind: i32, name: &str) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_BLOCK);
    w.write_d(kind);
    if kind == BLOCK_ADD || kind == BLOCK_REMOVE {
        w.write_s(name);
    }
    w.into_bytes()
}

pub fn dlg_answer(message_id: i32, answer: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_DLG_ANSWER);
    w.write_d(message_id);
    w.write_d(answer);
    w.into_bytes()
}

pub fn request_reload() -> Vec<u8> {
    vec![OP_REQUEST_RELOAD]
}

pub fn request_change_party_leader(name: &str) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_EXTENDED_CLIENT);
    w.write_h(EX_CHANGE_PARTY_LEADER);
    w.write_s(name);
    w.into_bytes()
}

pub fn request_auto_soul_shot(item_id: i32, enable: bool) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_EXTENDED_CLIENT);
    w.write_h(EX_AUTO_SOUL_SHOT);
    w.write_d(item_id);
    w.write_d(enable as i32);
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

    fn party_member(w: &mut Writer, object_id: i32, name: &str, level: i32) {
        w.write_d(object_id);
        w.write_s(name);
        for v in [10, 20, 300, 400, 50, 60] {
            w.write_d(v);
        }
        w.write_d(level);
        w.write_d(88);
    }

    #[test]
    fn attack_reads_the_first_hit_inline_and_the_extra_hits_after_the_short_count() {
        let mut w = Writer::new();
        w.write_c(OP_ATTACK);
        w.write_d(1);
        w.write_d(2);
        w.write_d(150);
        w.write_c(HIT_CRITICAL | HIT_SOULSHOT);
        w.write_d(10);
        w.write_d(20);
        w.write_d(30);
        w.write_h(2);
        w.write_d(3);
        w.write_d(0);
        w.write_c(HIT_MISS);
        w.write_d(4);
        w.write_d(7);
        w.write_c(HIT_SHIELD);
        let body = w.into_bytes();
        let a = read_attack(&body).unwrap();
        assert_eq!((a.attacker_id, a.x, a.y, a.z), (1, 10, 20, 30));
        assert_eq!(a.hits.len(), 3);
        assert_eq!(a.hits[0], Hit { target_id: 2, damage: 150, flags: HIT_CRITICAL | HIT_SOULSHOT });
        assert!(a.hits[0].critical() && a.hits[0].soulshot() && !a.hits[0].miss() && !a.hits[0].shield());
        assert!(a.hits[1].miss());
        assert!(a.hits[2].shield());
        assert!(read_attack(&body[..body.len() - 1]).is_err());
        assert!(read_attack(&body[..1 + 3 * 4 + 1 + 3 * 4 + 2]).is_err());
        let mut short = body[..1 + 3 * 4 + 1 + 3 * 4].to_vec();
        short.extend_from_slice(&0u16.to_le_bytes());
        assert_eq!(read_attack(&short).unwrap().hits.len(), 1);
        assert!(read_attack(&[OP_ATTACK, 1]).is_err());
    }

    #[test]
    fn drop_item_and_get_item() {
        let d = read_drop_item(&ds(OP_DROP_ITEM, &[9, 1001, 57, 1, 2, 3, 1, 250, 1])).unwrap();
        assert_eq!(
            d,
            DropItem { player_id: 9, object_id: 1001, item_id: 57, x: 1, y: 2, z: 3, stackable: true, count: 250, heading: 1 }
        );
        assert!(read_drop_item(&ds(OP_DROP_ITEM, &[9, 1001, 57, 1, 2, 3, 1, 250])).is_err());
        assert_eq!(read_drop_item(&ds(OP_DROP_ITEM, &[9, 1001, 57, 1, 2, 3, 1, 250, 1, 0])).unwrap(), d);
        let g = read_get_item(&ds(OP_GET_ITEM, &[9, 1001, 1, 2, 3])).unwrap();
        assert_eq!(g, GetItem { player_id: 9, object_id: 1001, x: 1, y: 2, z: 3 });
        assert!(read_get_item(&ds(OP_GET_ITEM, &[9, 1001, 1, 2])).is_err());
    }

    #[test]
    fn sun_rise_and_sun_set_are_bare_opcodes_and_ignore_a_tail_like_the_engine() {
        assert_eq!(read_sun_rise(&[OP_SUN_RISE]).unwrap(), SunRise);
        assert_eq!(read_sun_set(&[OP_SUN_SET, 1]).unwrap(), SunSet);
        assert!(read_sun_rise(&[]).is_err());
    }

    #[test]
    fn auto_attack_start_and_stop_carry_the_target() {
        assert_eq!(read_auto_attack_start(&ds(OP_AUTO_ATTACK_START, &[5])).unwrap(), AutoAttackStart { target_id: 5 });
        assert_eq!(read_auto_attack_stop(&ds(OP_AUTO_ATTACK_STOP, &[5])).unwrap(), AutoAttackStop { target_id: 5 });
        assert!(read_auto_attack_start(&[OP_AUTO_ATTACK_START, 1, 2]).is_err());
        assert_eq!(read_auto_attack_stop(&ds(OP_AUTO_ATTACK_STOP, &[5, 0])).unwrap(), AutoAttackStop { target_id: 5 });
    }

    #[test]
    fn ask_join_pledge_party_and_alliance() {
        let mut w = Writer::new();
        w.write_c(OP_ASK_JOIN_PLEDGE);
        w.write_d(7);
        w.write_s("Canastra");
        let body = w.into_bytes();
        assert_eq!(
            read_ask_join_pledge(&body).unwrap(),
            AskJoinPledge { requestor_id: 7, pledge_name: "Canastra".into() }
        );
        assert!(read_ask_join_pledge(&body[..body.len() - 2]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_ASK_JOIN_PARTY);
        w.write_s("Luzal");
        w.write_d(2);
        let body = w.into_bytes();
        assert_eq!(
            read_ask_join_party(&body).unwrap(),
            AskJoinParty { requestor_name: "Luzal".into(), item_distribution: 2 }
        );
        assert!(read_ask_join_party(&body[..body.len() - 1]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_ASK_JOIN_ALLIANCE);
        w.write_d(7);
        w.write_s("Luzal");
        w.write_s("");
        w.write_s("Aliados");
        let body = w.into_bytes();
        let a = read_ask_join_alliance(&body).unwrap();
        assert_eq!(
            (a.requestor_id, a.requestor_name.as_str(), a.requestor_ally_name.as_str()),
            (7, "Luzal", "Aliados")
        );
        assert!(read_ask_join_alliance(&body[..body.len() - 2]).is_err());
    }

    #[test]
    fn door_info_and_door_status_update() {
        assert_eq!(
            read_door_info(&ds(OP_DOOR_INFO, &[100, 24190001])).unwrap(),
            DoorInfo { object_id: 100, door_id: 24190001 }
        );
        assert!(read_door_info(&ds(OP_DOOR_INFO, &[100])).is_err());
        let d = read_door_status_update(&ds(OP_DOOR_STATUS_UPDATE, &[100, 1, 0, 1, 24190001, 5000, 4000])).unwrap();
        assert_eq!(
            d,
            DoorStatusUpdate {
                object_id: 100,
                closed: true,
                damage: 0,
                show_hp: true,
                door_id: 24190001,
                max_hp: 5000,
                hp: 4000
            }
        );
        assert!(read_door_status_update(&ds(OP_DOOR_STATUS_UPDATE, &[100, 1, 0, 1, 24190001, 5000])).is_err());
    }

    #[test]
    fn party_small_window_all_add_update_delete() {
        let mut w = Writer::new();
        w.write_c(OP_PARTY_SMALL_WINDOW_ALL);
        w.write_d(1);
        w.write_d(2);
        w.write_d(2);
        party_member(&mut w, 11, "A", 40);
        w.write_d(0);
        w.write_d(0);
        party_member(&mut w, 12, "B", 41);
        w.write_d(0);
        w.write_d(0);
        let body = w.into_bytes();
        let all = read_party_small_window_all(&body).unwrap();
        assert_eq!((all.leader_id, all.loot_distribution), (1, 2));
        assert_eq!(all.members.len(), 2);
        assert_eq!(
            all.members[1],
            PartyMember {
                object_id: 12,
                name: "B".into(),
                cp: 10,
                max_cp: 20,
                hp: 300,
                max_hp: 400,
                mp: 50,
                max_mp: 60,
                level: 41,
                class_id: 88
            }
        );
        assert!(read_party_small_window_all(&body[..body.len() - 4]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_PARTY_SMALL_WINDOW_ADD);
        w.write_d(1);
        w.write_d(2);
        party_member(&mut w, 13, "C", 42);
        w.write_d(0);
        w.write_d(0);
        let body = w.into_bytes();
        let add = read_party_small_window_add(&body).unwrap();
        assert_eq!((add.leader_id, add.member.object_id, add.member.level), (1, 13, 42));
        assert!(read_party_small_window_add(&body[..body.len() - 1]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_PARTY_SMALL_WINDOW_UPDATE);
        party_member(&mut w, 13, "C", 43);
        let body = w.into_bytes();
        assert_eq!(read_party_small_window_update(&body).unwrap().level, 43);
        assert!(read_party_small_window_update(&body[..body.len() - 3]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_PARTY_SMALL_WINDOW_DELETE);
        w.write_d(13);
        w.write_s("C");
        let body = w.into_bytes();
        assert_eq!(
            read_party_small_window_delete(&body).unwrap(),
            PartySmallWindowDelete { object_id: 13, name: "C".into() }
        );
        assert!(read_party_small_window_delete(&body[..body.len() - 2]).is_err());
        assert_eq!(
            read_party_small_window_delete_all(&[OP_PARTY_SMALL_WINDOW_DELETE_ALL]).unwrap(),
            PartySmallWindowDeleteAll
        );
    }

    #[test]
    fn vehicles() {
        let v = read_vehicle_info(&ds(OP_VEHICLE_INFO, &[500, 1, 2, 3, 4])).unwrap();
        assert_eq!(v, VehicleInfo { object_id: 500, x: 1, y: 2, z: 3, heading: 4 });
        assert!(read_vehicle_info(&ds(OP_VEHICLE_INFO, &[500, 1, 2, 3])).is_err());
        let d = read_vehicle_departure(&ds(OP_VEHICLE_DEPARTURE, &[500, 300, 100, 1, 2, 3])).unwrap();
        assert_eq!(
            d,
            VehicleDeparture { object_id: 500, move_speed: 300, rotation_speed: 100, to_x: 1, to_y: 2, to_z: 3 }
        );
        assert!(read_vehicle_departure(&ds(OP_VEHICLE_DEPARTURE, &[500, 300, 100, 1, 2])).is_err());
        let on = read_get_on_vehicle(&ds(OP_GET_ON_VEHICLE, &[7, 500, 1, 2, 3])).unwrap();
        assert_eq!(on, VehicleBoarding { object_id: 7, vehicle_id: 500, x: 1, y: 2, z: 3 });
        assert_eq!(read_get_off_vehicle(&ds(OP_GET_OFF_VEHICLE, &[7, 500, 1, 2, 3])).unwrap(), on);
        assert!(read_get_off_vehicle(&ds(OP_GET_OFF_VEHICLE, &[7, 500, 1, 2])).is_err());
        assert_eq!(
            read_vehicle_start(&ds(OP_VEHICLE_START, &[500, 1])).unwrap(),
            VehicleStart { object_id: 500, state: 1 }
        );
        assert!(read_vehicle_start(&ds(OP_VEHICLE_START, &[500])).is_err());
        let m =
            read_move_to_location_in_vehicle(&ds(OP_MOVE_TO_LOCATION_IN_VEHICLE, &[7, 500, 1, 2, 3, 4, 5, 6])).unwrap();
        assert_eq!(
            m,
            MoveToLocationInVehicle { object_id: 7, vehicle_id: 500, to_x: 1, to_y: 2, to_z: 3, x: 4, y: 5, z: 6 }
        );
        assert!(
            read_move_to_location_in_vehicle(&ds(OP_MOVE_TO_LOCATION_IN_VEHICLE, &[7, 500, 1, 2, 3, 4, 5])).is_err()
        );
        let s = read_stop_move_in_vehicle(&ds(OP_STOP_MOVE_IN_VEHICLE, &[7, 500, 1, 2, 3, 9])).unwrap();
        assert_eq!(s, StopMoveInVehicle { object_id: 7, vehicle_id: 500, x: 1, y: 2, z: 3, heading: 9 });
        assert!(read_stop_move_in_vehicle(&ds(OP_STOP_MOVE_IN_VEHICLE, &[7, 500, 1, 2, 3])).is_err());
    }

    #[test]
    fn rotating_reads_id_yaw_dir_speed_and_finish_ignores_the_lisvus_tail_byte() {
        let s = read_start_rotating(&ds(OP_START_ROTATING, &[7, 90, 1, 0])).unwrap();
        assert_eq!(s, StartRotating { object_id: 7, degree: 90, side: 1, speed: 0 });
        assert!(read_start_rotating(&ds(OP_START_ROTATING, &[7, 90, 1])).is_err());
        let mut lisvus = ds(OP_FINISH_ROTATING, &[7, 180, 65535]);
        lisvus.push(0);
        assert_eq!(read_finish_rotating(&lisvus).unwrap(), FinishRotating { object_id: 7, degree: 180, speed: 65535 });
        assert!(read_finish_rotating(&ds(OP_FINISH_ROTATING, &[7, 180])).is_err());
    }

    #[test]
    fn setup_gauge_ride_and_short_buff() {
        assert_eq!(
            read_setup_gauge(&ds(OP_SETUP_GAUGE, &[GAUGE_BLUE, 5000, 5000])).unwrap(),
            SetupGauge { color: 0, time: 5000, max_time: 5000 }
        );
        assert!(read_setup_gauge(&ds(OP_SETUP_GAUGE, &[0, 5000])).is_err());
        assert_eq!(
            read_ride(&ds(OP_RIDE, &[7, 1, 1, 12526])).unwrap(),
            Ride { object_id: 7, mounted: true, ride_type: 1, ride_class_id: 12526 }
        );
        assert!(read_ride(&ds(OP_RIDE, &[7, 1, 1])).is_err());
        let b = read_short_buff_status_update(&ds(OP_SHORT_BUFF_STATUS_UPDATE, &[2274, 1, 20])).unwrap();
        assert_eq!(b, ShortBuffStatusUpdate { skill_id: 2274, level: 1, duration: 20 });
        assert!(read_short_buff_status_update(&ds(OP_SHORT_BUFF_STATUS_UPDATE, &[2274, 1])).is_err());
    }

    #[test]
    fn server_object_info_reads_the_static_npc_shape() {
        let mut w = Writer::new();
        w.write_c(OP_SERVER_OBJECT_INFO);
        w.write_d(300);
        w.write_d(1_035_000);
        w.write_s("");
        w.write_d(1);
        w.write_d(1);
        w.write_d(2);
        w.write_d(3);
        w.write_d(4);
        w.write_f(1.0);
        w.write_f(1.0);
        w.write_f(12.0);
        w.write_f(30.0);
        w.write_d(900);
        w.write_d(1000);
        w.write_d(1);
        w.write_d(0);
        let body = w.into_bytes();
        let s = read_server_object_info(&body).unwrap();
        assert_eq!((s.object_id, s.npc_id, s.attackable), (300, 35000, true));
        assert_eq!((s.x, s.y, s.z, s.heading), (1, 2, 3, 4));
        assert_eq!((s.collision_radius, s.collision_height), (12.0, 30.0));
        assert_eq!((s.hp, s.max_hp, s.object_type, s.special_effects), (900, 1000, 1, 0));
        assert!(read_server_object_info(&body[..body.len() - 4]).is_err());
    }

    #[test]
    fn play_sound_reads_type_file_binding_object_position_and_delay() {
        let mut w = Writer::new();
        w.write_c(OP_PLAY_SOUND);
        w.write_d(0);
        w.write_s("ItemSound.quest_accept");
        w.write_d(0);
        w.write_d(0);
        w.write_d(0);
        w.write_d(0);
        w.write_d(0);
        w.write_d(0);
        let body = w.into_bytes();
        let s = read_play_sound(&body).unwrap();
        assert_eq!(s.sound_file, "ItemSound.quest_accept");
        assert!(!s.bind_to_object);
        assert!(read_play_sound(&body[..body.len() - 1]).is_err());
        let mut w = Writer::new();
        w.write_c(OP_PLAY_SOUND);
        w.write_d(1);
        w.write_s(&"s".repeat(SOUND_MAX_CHARS + 1));
        for _ in 0..6 {
            w.write_d(0);
        }
        assert_eq!(read_play_sound(&w.into_bytes()), Err(WireError::TooLong));
    }

    #[test]
    fn static_object_party_positions_and_chair() {
        assert_eq!(
            read_static_object(&ds(OP_STATIC_OBJECT, &[24, 900])).unwrap(),
            StaticObject { static_object_id: 24, object_id: 900 }
        );
        assert!(read_static_object(&ds(OP_STATIC_OBJECT, &[24])).is_err());
        let p = read_party_member_position(&ds(OP_PARTY_MEMBER_POSITION, &[2, 11, 1, 2, 3, 12, 4, 5, 6])).unwrap();
        assert_eq!(
            p.positions,
            vec![
                MemberPosition { object_id: 11, x: 1, y: 2, z: 3 },
                MemberPosition { object_id: 12, x: 4, y: 5, z: 6 }
            ]
        );
        assert!(read_party_member_position(&ds(OP_PARTY_MEMBER_POSITION, &[2, 11, 1, 2, 3])).is_err());
        assert_eq!(
            read_chair_sit(&ds(OP_CHAIR_SIT, &[7, 24])).unwrap(),
            ChairSit { object_id: 7, static_object_id: 24 }
        );
        assert!(read_chair_sit(&ds(OP_CHAIR_SIT, &[7])).is_err());
    }

    fn pet_info_body() -> Vec<u8> {
        let mut w = Writer::new();
        w.write_c(OP_PET_INFO);
        w.write_d(SUMMON_PET);
        w.write_d(600);
        w.write_d(1_012_077);
        w.write_d(0);
        w.write_d(1);
        w.write_d(2);
        w.write_d(3);
        w.write_d(4);
        w.write_d(0);
        w.write_d(333);
        w.write_d(253);
        w.write_d(120);
        w.write_d(60);
        for _ in 0..6 {
            w.write_d(0);
        }
        w.write_f(1.0);
        w.write_f(1.0);
        w.write_f(9.0);
        w.write_f(15.0);
        w.write_d(0);
        w.write_d(0);
        w.write_d(0);
        w.write_c(1);
        w.write_c(1);
        w.write_c(0);
        w.write_c(0);
        w.write_c(2);
        w.write_s("Wolf");
        w.write_s("Luzal");
        w.write_d(1);
        w.write_d(0);
        w.write_d(0);
        w.write_d(80);
        w.write_d(100);
        w.write_d(500);
        w.write_d(600);
        w.write_d(70);
        w.write_d(90);
        w.write_d(11);
        w.write_d(25);
        w.write_d(1000);
        w.write_d(900);
        w.write_d(1500);
        w.write_d(30);
        w.write_d(200);
        for v in [51, 52, 53, 54, 55, 56, 57, 58, 59, 60] {
            w.write_d(v);
        }
        w.write_d(0);
        w.write_h(1);
        w.write_c(0);
        w.write_h(0);
        w.write_c(2);
        w.write_d(1);
        w.write_d(1);
        w.into_bytes()
    }

    #[test]
    fn pet_info_reads_the_lisvus_shape_with_the_engine_letters() {
        let p = read_pet_info(&pet_info_body()).unwrap();
        assert_eq!((p.summon_type, p.object_id, p.npc_id, p.attackable), (SUMMON_PET, 600, 12077, false));
        assert_eq!((p.x, p.y, p.z, p.heading), (1, 2, 3, 4));
        assert_eq!((p.run_speed, p.walk_speed), (120, 60));
        assert!(p.has_owner && p.running && !p.in_combat && !p.alike_dead);
        assert_eq!(p.spawn_animation, 2);
        assert_eq!((p.name.as_str(), p.title.as_str()), ("Wolf", "Luzal"));
        assert_eq!((p.fed, p.max_fed, p.hp, p.max_hp, p.mp, p.max_mp), (80, 100, 500, 600, 70, 90));
        assert_eq!((p.sp, p.level, p.exp, p.exp_this_level, p.exp_next_level), (11, 25, 1000, 900, 1500));
        assert_eq!((p.load, p.max_load, p.p_atk, p.cast_spd, p.abnormal_effect), (30, 200, 51, 60, 0));
        assert_eq!((p.mountable, p.team, p.soulshots_per_hit, p.spiritshots_per_hit), (1, 2, 1, 1));
        let body = pet_info_body();
        assert!(read_pet_info(&body[..body.len() - 1]).is_err());
    }

    #[test]
    fn pet_status_show_update_delete_and_summon_time() {
        assert_eq!(read_pet_status_show(&ds(OP_PET_STATUS_SHOW, &[2])).unwrap(), PetStatusShow { summon_type: 2 });
        let mut w = Writer::new();
        w.write_c(OP_PET_STATUS_UPDATE);
        w.write_d(2);
        w.write_d(600);
        w.write_d(1);
        w.write_d(2);
        w.write_d(3);
        w.write_s("");
        for v in [80, 100, 500, 600, 70, 90, 25, 1000, 900, 1500] {
            w.write_d(v);
        }
        let body = w.into_bytes();
        let u = read_pet_status_update(&body).unwrap();
        assert_eq!((u.summon_type, u.object_id, u.x, u.y, u.z), (2, 600, 1, 2, 3));
        assert_eq!((u.fed, u.max_fed, u.hp, u.max_hp, u.mp, u.max_mp, u.level), (80, 100, 500, 600, 70, 90, 25));
        assert_eq!((u.exp, u.exp_this_level, u.exp_next_level), (1000, 900, 1500));
        assert!(read_pet_status_update(&body[..body.len() - 4]).is_err());
        assert_eq!(
            read_pet_delete(&ds(OP_PET_DELETE, &[2, 600])).unwrap(),
            PetDelete { summon_type: 2, object_id: 600 }
        );
        assert!(read_pet_delete(&ds(OP_PET_DELETE, &[2])).is_err());
        let t = read_set_summon_remain_time(&ds(OP_SET_SUMMON_REMAIN_TIME, &[1200, 600])).unwrap();
        assert_eq!(t, SetSummonRemainTime { max_time: 1200, remaining_time: 600 });
        assert!(read_set_summon_remain_time(&ds(OP_SET_SUMMON_REMAIN_TIME, &[1200])).is_err());
    }

    #[test]
    fn earthquake_fly_camera_dice_nickname() {
        let e = read_earthquake(&ds(OP_EARTHQUAKE, &[1, 2, 3, 20, 10, 0])).unwrap();
        assert_eq!(e, Earthquake { x: 1, y: 2, z: 3, intensity: 20, duration: 10 });
        assert!(read_earthquake(&ds(OP_EARTHQUAKE, &[1, 2, 3, 20, 10])).is_err());
        let f = read_fly_to_location(&ds(OP_FLY_TO_LOCATION, &[7, 1, 2, 3, 4, 5, 6, 1])).unwrap();
        assert_eq!(f, FlyToLocation { object_id: 7, to_x: 1, to_y: 2, to_z: 3, x: 4, y: 5, z: 6 });
        assert_eq!(read_fly_to_location(&ds(OP_FLY_TO_LOCATION, &[7, 1, 2, 3, 4, 5, 6])).unwrap(), f);
        assert!(read_fly_to_location(&ds(OP_FLY_TO_LOCATION, &[7, 1, 2, 3, 4, 5])).is_err());
        let c = read_special_camera(&ds(OP_SPECIAL_CAMERA, &[7, 300, 90, 10, 2000, 5000, 1, 0, 1, 0])).unwrap();
        assert_eq!(
            c,
            SpecialCamera {
                target_id: 7,
                distance: 300,
                yaw: 90,
                pitch: 10,
                time: 2000,
                duration: 5000,
                turn: 1,
                rise: 0,
                widescreen: 1,
                unknown: 0
            }
        );
        assert!(read_special_camera(&ds(OP_SPECIAL_CAMERA, &[7, 300, 90, 10, 2000, 5000, 1, 0, 1])).is_err());
        let d = read_dice(&ds(OP_DICE, &[7, 4625, 6, 1, 2, 3])).unwrap();
        assert_eq!(d, Dice { player_id: 7, item_id: 4625, number: 6, x: 1, y: 2, z: 3 });
        assert!(read_dice(&ds(OP_DICE, &[7, 4625, 6, 1, 2])).is_err());
        let mut w = Writer::new();
        w.write_c(OP_NICKNAME_CHANGED);
        w.write_d(7);
        w.write_s("Herói");
        let body = w.into_bytes();
        assert_eq!(read_nickname_changed(&body).unwrap(), NicknameChanged { object_id: 7, title: "Herói".into() });
        assert!(read_nickname_changed(&body[..body.len() - 2]).is_err());
    }

    #[test]
    fn confirm_dlg_shares_the_system_message_layout() {
        let mut w = Writer::new();
        w.write_c(OP_CONFIRM_DLG);
        w.write_d(1510);
        w.write_d(1);
        w.write_d(0);
        w.write_s("Luzal");
        let m = read_confirm_dlg(&w.into_bytes()).unwrap();
        assert_eq!(m.id, 1510);
        assert_eq!(m.args, vec![super::super::SystemMessageArg::Text("Luzal".into())]);
        assert!(read_confirm_dlg(&[OP_CONFIRM_DLG, 1]).is_err());
    }

    #[test]
    fn party_spelled_reads_the_buff_list() {
        let mut w = Writer::new();
        w.write_c(OP_PARTY_SPELLED);
        w.write_d(0);
        w.write_d(7);
        w.write_d(2);
        w.write_d(1204);
        w.write_h(2);
        w.write_d(1200);
        w.write_d(1040);
        w.write_h(1);
        w.write_d(30);
        let body = w.into_bytes();
        let p = read_party_spelled(&body).unwrap();
        assert_eq!((p.summon_type, p.object_id), (0, 7));
        assert_eq!(
            p.effects,
            vec![
                PartyBuff { skill_id: 1204, level: 2, duration: 1200 },
                PartyBuff { skill_id: 1040, level: 1, duration: 30 }
            ]
        );
        assert!(read_party_spelled(&body[..body.len() - 1]).is_err());
        assert!(read_party_spelled(&ds(OP_PARTY_SPELLED, &[0, 7, 0])).unwrap().effects.is_empty());
    }

    #[test]
    fn unetworkhandler_get_on_vehicle_103f4dc0_and_get_off_vehicle_103f4e50() {
        for (encode, opcode) in [
            (request_get_on_vehicle as fn(i32, [i32; 3]) -> Vec<u8>, 0x42),
            (request_get_off_vehicle as fn(i32, [i32; 3]) -> Vec<u8>, 0x43),
        ] {
            for vehicle_id in [i32::MIN, -1, 0, 500, i32::MAX] {
                for position in [[0, 0, 0], [1, -2, 3], [i32::MIN, i32::MAX, -1]] {
                    let bytes = encode(vehicle_id, position);
                    assert_eq!(bytes.len(), 17);
                    assert_eq!(bytes[0], opcode);
                    for (index, value) in [vehicle_id, position[0], position[1], position[2]].iter().enumerate() {
                        assert_eq!(&bytes[1 + index * 4..5 + index * 4], &value.to_le_bytes());
                    }
                }
            }
        }
    }

    #[test]
    fn unetworkhandler_move_to_location_10410da0_vehicle_wire_10410fc6() {
        for vehicle_id in [i32::MIN, -1, 0, 500, i32::MAX] {
            for local_target in [[0, 0, 0], [1, -2, 3], [i32::MIN, i32::MAX, -1]] {
                for local_origin in [[4, 5, -6], [0, 0, 0], [i32::MAX, i32::MIN, 1]] {
                    let bytes = request_move_to_location_in_vehicle(vehicle_id, local_target, local_origin);
                    assert_eq!(bytes.len(), 29);
                    assert_eq!(bytes[0], 0x5c);
                    let fields = [
                        vehicle_id, local_target[0], local_target[1], local_target[2],
                        local_origin[0], local_origin[1], local_origin[2],
                    ];
                    for (index, value) in fields.iter().enumerate() {
                        assert_eq!(&bytes[1 + index * 4..5 + index * 4], &value.to_le_bytes());
                    }
                }
            }
        }
    }

    #[test]
    fn world_encoders_follow_the_engine_formats() {
        assert_eq!(
            attack_request(7, [1, 2, 3], true),
            [vec![OP_ATTACK_REQUEST], ds(0, &[7, 1, 2, 3])[1..].to_vec(), vec![1]].concat()
        );
        assert_eq!(request_drop_item(1001, 5, [1, 2, 3]), ds(OP_REQUEST_DROP_ITEM, &[1001, 5, 1, 2, 3]));
        assert_eq!(request_social_action(2), ds(OP_REQUEST_SOCIAL_ACTION, &[2]));
        assert_eq!(request_change_move_type(true), ds(OP_REQUEST_CHANGE_MOVE_TYPE, &[1]));
        assert_eq!(request_change_wait_type(false), ds(OP_REQUEST_CHANGE_WAIT_TYPE, &[0]));
        assert_eq!(request_answer_join_pledge(true), ds(OP_REQUEST_ANSWER_JOIN_PLEDGE, &[1]));
        let mut w = Writer::new();
        w.write_c(OP_REQUEST_JOIN_PARTY);
        w.write_s("Luzal");
        w.write_d(2);
        assert_eq!(request_join_party("Luzal", 2), w.into_bytes());
        assert_eq!(request_answer_join_party(false), ds(OP_REQUEST_ANSWER_JOIN_PARTY, &[0]));
        assert_eq!(request_withdrawal_party(), [OP_REQUEST_WITHDRAWAL_PARTY]);
        assert_eq!(request_oust_party_member("L"), [OP_REQUEST_OUST_PARTY_MEMBER, b'L', 0, 0, 0]);
        assert_eq!(cannot_move_anymore([1, 2, 3], 9), ds(OP_CANNOT_MOVE_ANYMORE, &[1, 2, 3, 9]));
        assert_eq!(
            cannot_move_anymore_in_vehicle(500, [1, 2, 3], 9),
            ds(OP_CANNOT_MOVE_ANYMORE_IN_VEHICLE, &[500, 1, 2, 3, 9])
        );
        assert_eq!(request_skill_list(), [OP_REQUEST_SKILL_LIST]);
        assert_eq!(answer_trade_request(true), ds(OP_ANSWER_TRADE_REQUEST, &[1]));
        assert_eq!(start_rotating(90, 1), ds(OP_START_ROTATING_CLIENT, &[90, 1]));
        assert_eq!(finish_rotating(90, 0), ds(OP_FINISH_ROTATING_CLIENT, &[90, 0]));
        assert_eq!(request_quest_list(), [OP_REQUEST_QUEST_LIST]);
        assert_eq!(request_pledge_info(33), ds(OP_REQUEST_PLEDGE_INFO, &[33]));
        assert_eq!(request_acquire_skill_info(3, 2, 0), ds(OP_REQUEST_ACQUIRE_SKILL_INFO, &[3, 2, 0]));
        assert_eq!(request_acquire_skill(3, 2, 1), ds(OP_REQUEST_ACQUIRE_SKILL, &[3, 2, 1]));
        assert_eq!(request_answer_join_ally(true), ds(OP_REQUEST_ANSWER_JOIN_ALLY, &[1]));
        assert_eq!(request_change_pet_name("W"), [OP_REQUEST_CHANGE_PET_NAME, b'W', 0, 0, 0]);
        assert_eq!(request_pet_use_item(1001), ds(OP_REQUEST_PET_USE_ITEM, &[1001]));
        assert_eq!(request_give_item_to_pet(1001, 3), ds(OP_REQUEST_GIVE_ITEM_TO_PET, &[1001, 3]));
        assert_eq!(request_get_item_from_pet(1001, 3, 0), ds(OP_REQUEST_GET_ITEM_FROM_PET, &[1001, 3, 0]));
        assert_eq!(request_block(BLOCK_ADD, "L"), [OP_REQUEST_BLOCK, 0, 0, 0, 0, b'L', 0, 0, 0]);
        assert_eq!(request_block(BLOCK_LIST, "L"), ds(OP_REQUEST_BLOCK, &[BLOCK_LIST]));
        assert_eq!(dlg_answer(1510, 1), ds(OP_DLG_ANSWER, &[1510, 1]));
        assert_eq!(request_reload(), [OP_REQUEST_RELOAD]);
        assert_eq!(request_change_party_leader("L"), [OP_EXTENDED_CLIENT, 4, 0, b'L', 0, 0, 0]);
        assert_eq!(
            request_auto_soul_shot(1835, true),
            [vec![OP_EXTENDED_CLIENT, 5, 0], ds(0, &[1835, 1])[1..].to_vec()].concat()
        );
    }
}
