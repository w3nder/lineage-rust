use std::collections::{HashMap, VecDeque};
use killer_platform::time::Instant;

use l2_net::game::{
    AcquireSkillInfo, AcquireSkillList, AskJoinAlliance, AskJoinParty, AskJoinPledge, BuyListSeed, DoorStatusUpdate,
    Earthquake, Friend, FriendRecvMsg, HennaEquipList, HennaInfo, HennaItemInfo, Hit, InventoryItem, ManagePledgePower,
    PackageSendableList, PackageToList, PartyBuff, PartyMatchDetail, PartyMatchList, PartyMember,
    PartySmallWindowAdd, PartySmallWindowAll, PetInfo, PetStatusUpdate, PlaySound, PledgeInfo, PledgeMember,
    PledgeShowInfoUpdate, PledgeShowMemberListAll, PrivateStoreListBuy, PrivateStoreListSell,
    PrivateStoreManageListBuy, PrivateStoreManageListSell, QuestState, SetupGauge, ShopList,
    ShopPreviewInfo, ShopPreviewList, ShowBoard, ShowMinimap, ShowTownMap, SpecialCamera, SystemMessage, TradeItem,
    TradeStart, VehicleDeparture, VehicleInfo, WarehouseList,
};
use l2_net::game::{
    CastleSiegeInfo, CropInfo, ManorDefaultInfo, ManorEntry, MonRaceInfo, SeedInfo, SeedSetting, CropSettingInfo, SellCropList, ProcureCropDetail, HeroList, RecipeBookItemList, RecipeItemMakeInfo, RecipeShopItemInfo, RecipeShopManageList,
    RecipeShopSellList, SiegeClanList, SsqStatus,
};

pub const HIT_HISTORY: usize = 64;
pub const SOUND_QUEUE: usize = 32;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Party {
    pub leader_id: i32,
    pub loot_distribution: i32,
    pub members: Vec<PartyMember>,
    pub positions: PositionMap,
    pub spelled: HashMap<i32, Vec<PartyBuff>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PositionMap {
    pairs: Vec<(i32, [i32; 3])>,
}

impl PositionMap {
    pub fn insert(&mut self, key: i32, value: [i32; 3]) -> Option<[i32; 3]> {
        match self.pairs.iter_mut().find(|pair| pair.0 == key) {
            Some(pair) => Some(std::mem::replace(&mut pair.1, value)),
            None => {
                self.pairs.push((key, value));
                None
            }
        }
    }

    pub fn remove(&mut self, key: &i32) -> Option<[i32; 3]> {
        let index = self.pairs.iter().position(|pair| pair.0 == *key)?;
        Some(self.pairs.remove(index).1)
    }

    pub fn get(&self, key: &i32) -> Option<&[i32; 3]> {
        self.pairs.iter().find(|pair| pair.0 == *key).map(|pair| &pair.1)
    }

    pub fn contains_key(&self, key: &i32) -> bool {
        self.get(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&i32, &[i32; 3])> {
        self.pairs.iter().map(|pair| (&pair.0, &pair.1))
    }
}

impl Party {
    pub fn apply_all(&mut self, all: &PartySmallWindowAll) {
        self.leader_id = all.leader_id;
        self.loot_distribution = all.loot_distribution;
        self.members = all.members.clone();
    }

    pub fn apply_add(&mut self, add: &PartySmallWindowAdd) {
        self.leader_id = add.leader_id;
        self.loot_distribution = add.loot_distribution;
        self.upsert(&add.member);
    }

    pub fn update_existing(&mut self, member: &PartyMember) {
        if let Some(known) = self.members.iter_mut().find(|m| m.object_id == member.object_id) {
            *known = member.clone();
        }
    }

    pub fn upsert(&mut self, member: &PartyMember) {
        match self.members.iter_mut().find(|m| m.object_id == member.object_id) {
            Some(known) => *known = member.clone(),
            None => self.members.push(member.clone()),
        }
    }

    pub fn remove(&mut self, object_id: i32) {
        self.members.retain(|m| m.object_id != object_id);
        self.positions.remove(&object_id);
        self.spelled.remove(&object_id);
    }

    pub fn clear(&mut self) {
        *self = Party::default();
    }

    pub fn in_party(&self) -> bool {
        !self.members.is_empty()
    }

    pub fn member(&self, object_id: i32) -> Option<&PartyMember> {
        self.members.iter().find(|m| m.object_id == object_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemDropKind {
    Drop,
    Destroy,
    Crystallize,
    GiveToPet,
    GetFromPet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SiegeDecision {
    Join { castle_id: i32, attacker: bool, join: bool },
    Approve { castle_id: i32, pledge_id: i32, approved: bool },
    SetTime { castle_id: i32, time: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dialog {
    Siege { message_id: u32, dialog_type: i32, parameter: String, decision: SiegeDecision },
    JoinPledge(AskJoinPledge),
    JoinParty(AskJoinParty),
    JoinAlliance(AskJoinAlliance),
    Confirm(SystemMessage),
    Notice(SystemMessage),
    TradeRequest { requestor_id: i32 },
    FriendRequest { requestor_name: String },
    UseItemPopup { object_id: i32, message_id: i32, item_name: String },
    RecipeRegister { object_id: i32, item_name: String },
    OustPledgeMember { name: String },
    AbortQuest { quest_id: i32 },
    SystemMenu { restart: bool },
    GmConfirm { dialog_type: i32, message_id: i32, name: String, command: String },
    GmText { text: String },
    GmNotice { message_id: i32 },
    WithdrawPledge { pledge_name: String },
    /// Confirmação dos comandos de macro `/allyleave` e `/allydissolve`: o original só manda o
    /// pacote depois de o jogador aceitar o diálogo (`FUN_10007b70(0x200)` e `(0x202)` em
    /// `FUN_10058330`, cujas mensagens são "Do you really wish to withdraw from the alliance?" e
    /// "Do you really wish to dissolve the alliance?").
    AllyConfirm { message_id: u32, dissolve: bool },
    AllyDismissName,
    AllyDismissConfirm { pledge_name: String },
    DeleteMacro { id: i32, name: String },
    DeleteRecipe { recipe_id: i32, name: String },
    PetName,
    DropItem { object_id: i32, name: String, message_id: u32, count: Option<i32>, position: [i32; 3], kind: ItemDropKind },
    PledgeWarConfirm { message_id: u32, pledge_name: String, start: bool },
    AllianceWar { start: bool, first: String, second: String },
    GiveTitle { target: String },
    DeleteTitle { target: String },
    PledgeWarName { start: bool },
    JoinMpcc { name: String },
    DeleteCrest,
    DeleteInsignia,
    CrestNotice { large: bool },
    CrestFile { large: bool },
    Disconnected { reason: i32 },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Dialogs {
    pub pending: Vec<Dialog>,
}

impl Dialogs {
    pub fn push(&mut self, dialog: Dialog) {
        self.pending.push(dialog);
    }

    pub fn take(&mut self) -> Option<Dialog> {
        if self.pending.is_empty() { None } else { Some(self.pending.remove(0)) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitEvent {
    pub attacker_id: i32,
    pub hit: Hit,
    pub at: Instant,
}

#[derive(Debug, Clone, Default)]
pub struct Combat {
    pub auto_attack_target: Option<i32>,
    pub auto_attacking: HashMap<i32, i32>,
    pub auto_soulshots: HashMap<i32, bool>,
    pub hits: VecDeque<HitEvent>,
}

impl Combat {
    pub fn record(&mut self, attacker_id: i32, hits: &[Hit], at: Instant) {
        for hit in hits {
            if self.hits.len() == HIT_HISTORY {
                self.hits.pop_front();
            }
            self.hits.push_back(HitEvent { attacker_id, hit: *hit, at });
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gauge {
    pub setup: SetupGauge,
    pub started: Instant,
}

impl Gauge {
    pub fn remaining(&self, now: Instant) -> i32 {
        let elapsed = now.saturating_duration_since(self.started).as_millis() as i32;
        (self.setup.time - elapsed).max(0)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Door {
    pub info_door_id: Option<i32>,
    pub status_received: bool,
    pub door_id: i32,
    pub closed: bool,
    pub damage: i32,
    pub show_hp: bool,
    pub hp: i32,
    pub max_hp: i32,
}

impl Door {
    pub fn apply(&mut self, update: &DoorStatusUpdate) {
        self.status_received = self.info_door_id.is_some();
        self.door_id = update.door_id;
        self.closed = update.closed;
        self.damage = update.damage;
        self.show_hp = update.show_hp;
        self.hp = update.hp;
        self.max_hp = update.max_hp;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticTarget {
    pub object_id: i32,
    pub static_object_id: i32,
    pub hp: Option<(i32, i32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vehicle {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub heading: i32,
    pub dest: Option<[i32; 3]>,
    pub move_speed: i32,
    pub rotation_speed: i32,
    pub state: i32,
}

impl Vehicle {
    pub fn from_info(info: &VehicleInfo) -> Vehicle {
        Vehicle {
            x: info.x,
            y: info.y,
            z: info.z,
            heading: info.heading,
            dest: None,
            move_speed: 0,
            rotation_speed: 0,
            state: 0,
        }
    }

    pub fn depart(&mut self, departure: &VehicleDeparture) {
        self.move_speed = departure.move_speed;
        self.rotation_speed = departure.rotation_speed;
        self.dest = Some([departure.to_x, departure.to_y, departure.to_z]);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pet {
    pub info: PetInfo,
    pub max_time: i32,
    pub remaining_time: i32,
}

impl Pet {
    pub fn apply_status(&mut self, status: &PetStatusUpdate) {
        let info = &mut self.info;
        info.x = status.x;
        info.y = status.y;
        info.z = status.z;
        info.fed = status.fed;
        info.max_fed = status.max_fed;
        info.hp = status.hp;
        info.max_hp = status.max_hp;
        info.mp = status.mp;
        info.max_mp = status.max_mp;
        info.level = status.level;
        info.exp = status.exp;
        info.exp_this_level = status.exp_this_level;
        info.exp_next_level = status.exp_next_level;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraCommand {
    Special(SpecialCamera),
    Normal,
    Mode(i32),
}

#[derive(Debug, Clone, Default)]
pub struct Effects {
    pub sounds: VecDeque<PlaySound>,
    pub camera_commands: VecDeque<CameraCommand>,
    pub earthquakes: VecDeque<Earthquake>,
    pub firecrackers: VecDeque<i32>,
    pub sweepable_deaths: VecDeque<i32>,
    pub equip_sounds: VecDeque<i32>,
    pub enter_effects: VecDeque<crate::enter_event::EnterEffect>,
    pub enter_sounds: VecDeque<crate::enter_event::EnterSound>,
}

impl Effects {
    pub fn take_camera_commands(&mut self) -> Vec<CameraCommand> {
        self.camera_commands.drain(..).collect()
    }

    pub fn take_firecrackers(&mut self) -> Vec<i32> {
        self.firecrackers.drain(..).collect()
    }

    pub fn take_equip_sounds(&mut self) -> Vec<i32> {
        self.equip_sounds.drain(..).collect()
    }

    pub fn take_enter_effects(&mut self) -> Vec<crate::enter_event::EnterEffect> {
        self.enter_effects.drain(..).collect()
    }

    pub fn take_enter_sounds(&mut self) -> Vec<crate::enter_event::EnterSound> {
        self.enter_sounds.drain(..).collect()
    }

    pub fn take_sweepable_deaths(&mut self) -> Vec<i32> {
        self.sweepable_deaths.drain(..).collect()
    }

    pub fn take_earthquakes(&mut self) -> Vec<Earthquake> {
        self.earthquakes.drain(..).collect()
    }

    pub fn push_sound(&mut self, sound: PlaySound) {
        if self.sounds.len() == SOUND_QUEUE {
            self.sounds.pop_front();
        }
        self.sounds.push_back(sound);
    }

    pub fn take_sounds(&mut self) -> Vec<PlaySound> {
        self.sounds.drain(..).collect()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Trade {
    pub partner_id: Option<i32>,
    pub mine: Vec<TradeItem>,
    pub theirs: Vec<TradeItem>,
    pub tradable: Vec<TradeItem>,
    pub last_result: Option<bool>,
}

impl Trade {
    pub fn start(&mut self, start: &TradeStart) {
        self.partner_id = Some(start.partner_id);
        self.tradable = start.items.clone();
        self.mine.clear();
        self.theirs.clear();
        self.last_result = None;
    }

    pub fn done(&mut self, success: bool) {
        self.partner_id = None;
        self.mine.clear();
        self.theirs.clear();
        self.tradable.clear();
        self.last_result = Some(success);
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PledgeRecord {
    pub enemy_pledge_id: i32,
    pub surrender_war_id: i32,
    pub pledge_id: i32,
    pub name: String,
    pub ally_name: String,
    pub crest_id: i32,
    pub ally_id: i32,
    pub ally_crest_id: i32,
    pub large_crest_id: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pledge {
    pub info: Option<PledgeShowMemberListAll>,
    pub members: Vec<PledgeMember>,
    pub summary: Option<PledgeInfo>,
    pub last_update: Option<PledgeShowInfoUpdate>,
    pub crests: HashMap<i32, Vec<u8>>,
    pub ally_crests: HashMap<i32, Vec<u8>>,
    pub large_crests: HashMap<i32, Vec<u8>>,
    pub joined_pledge_id: Option<i32>,
    pub power: Option<ManagePledgePower>,
    pub list_reset: u32,
    pub listed: Vec<bool>,
}

impl Pledge {
    fn align_listed(&mut self) {
        self.listed.truncate(self.members.len());
        while self.listed.len() < self.members.len() {
            self.listed.push(self.members[self.listed.len()].online_object_id > 0);
        }
    }

    fn find(&self, name: &str) -> Option<usize> {
        if name.is_empty() {
            return None;
        }
        self.members.iter().position(|m| !m.name.is_empty() && m.name.eq_ignore_ascii_case(name))
    }

    pub fn listed_online(&self, index: usize) -> bool {
        self.listed.get(index).copied().unwrap_or_else(|| self.members.get(index).is_some_and(|m| m.online_object_id > 0))
    }

    pub fn update_existing(&mut self, member: &PledgeMember) {
        self.align_listed();
        let Some(index) = self.find(&member.name) else { return };
        let known = &mut self.members[index];
        known.level = member.level;
        if member.online_object_id > 0 {
            if member.class_id > -1 {
                known.class_id = member.class_id;
            }
            known.online_object_id = member.online_object_id;
            self.listed[index] = true;
        } else {
            self.listed[index] = false;
        }
    }

    pub fn clear_header(&mut self) {
        self.list_reset = self.list_reset.wrapping_add(1);
        self.members.clear();
        self.listed.clear();
        if let Some(info) = self.info.as_mut() {
            info.pledge_id = 0;
            info.name.clear();
            info.leader_name.clear();
            info.crest_id = 0;
            info.level = 0;
            info.castle_id = 0;
            info.hideout_id = 0;
            info.my_level = 0;
            info.dissolving = 0;
        }
    }

    pub fn upsert(&mut self, member: &PledgeMember) {
        self.align_listed();
        match self.find(&member.name) {
            Some(index) => {
                let known = &mut self.members[index];
                known.level = member.level;
                if member.class_id > -1 {
                    known.class_id = member.class_id;
                }
                self.listed[index] = member.online_object_id > 0;
            }
            None => {
                self.members.push(member.clone());
                self.listed.push(member.online_object_id > 0);
            }
        }
    }

    pub fn remove(&mut self, name: &str) {
        self.align_listed();
        if let Some(index) = self.members.iter().position(|m| m.name.eq_ignore_ascii_case(name)) {
            self.members.remove(index);
            self.listed.remove(index);
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RadarState {
    pub enabled: bool,
    pub target: Option<[f32; 3]>,
    pub points: Vec<[i32; 3]>,
    pub map_points: Vec<[i32; 3]>,
    pub show_time: bool,
}

fn remove_stepping_past(list: &mut Vec<[i32; 3]>, item: [i32; 3]) {
    let mut index = 0;
    while index < list.len() {
        if list[index] == item {
            list.remove(index);
        }
        index += 1;
    }
}

impl RadarState {
    pub fn apply_control(&mut self, mode: i32, kind: i32, point: [i32; 3]) {
        let radar = matches!(kind, 0 | 2);
        let map = matches!(kind, 1 | 2);
        match mode {
            0 => {
                self.enabled = true;
                self.target = Some(point.map(|v| v as f32));
                if radar {
                    self.points.push(point);
                }
                if map {
                    self.map_points.push(point);
                }
            }
            1 => {
                if radar {
                    remove_stepping_past(&mut self.points, point);
                }
                if map {
                    remove_stepping_past(&mut self.map_points, point);
                }
            }
            2 => {
                self.enabled = false;
                self.points.clear();
                self.map_points.clear();
            }
            _ => {}
        }
    }

    pub fn delete(&mut self, point: [i32; 3]) {
        self.apply_control(1, 2, point);
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrivateStores {
    pub manage_sell: Option<PrivateStoreManageListSell>,
    pub sell: Option<PrivateStoreListSell>,
    pub manage_buy: Option<PrivateStoreManageListBuy>,
    pub buy: Option<PrivateStoreListBuy>,
    pub sell_messages: HashMap<i32, String>,
    pub buy_messages: HashMap<i32, String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tutorial {
    pub html: Option<String>,
    pub question_mark: Option<i32>,
    pub client_event: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Henna {
    pub equip_list: Option<HennaEquipList>,
    pub item_info: Option<HennaItemInfo>,
    pub info: Option<HennaInfo>,
    pub unequip_list: Option<HennaEquipList>,
    pub unequip_info: Option<HennaItemInfo>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Friends {
    pub list: Vec<Friend>,
    pub messages: Vec<FriendRecvMsg>,
}

pub const FRIEND_ACTION_ADD: i32 = 1;
pub const FRIEND_ACTION_UPDATE: i32 = 2;
pub const FRIEND_ACTION_REMOVE: i32 = 3;

impl Friends {
    pub fn sort(&mut self) {
        self.list.sort_by(|a, b| {
            b.online.cmp(&a.online).then_with(|| a.name.encode_utf16().cmp(b.name.encode_utf16()))
        });
    }

    pub fn apply_packet(&mut self, packet: &l2_net::game::L2Friend) {
        let friend = Friend {
            object_id: packet.values[1],
            name: packet.name.clone(),
            online: packet.tail[0] != 0,
            tail: packet.tail[1],
        };
        let position = self.list.iter().position(|known| known.name == friend.name);
        match (packet.values[0], position) {
            (FRIEND_ACTION_ADD, Some(index)) | (FRIEND_ACTION_UPDATE, Some(index)) => self.list[index] = friend,
            (FRIEND_ACTION_ADD, None) => self.list.push(friend),
            (FRIEND_ACTION_REMOVE, Some(index)) => {
                self.list.remove(index);
            }
            _ => return,
        }
        self.sort();
    }

    pub fn apply_status(&mut self, status: &l2_net::game::L2FriendStatus) {
        if let Some(friend) = self.list.iter_mut().find(|known| known.name == status.name) {
            friend.online = status.value != 0;
            friend.object_id = status.status;
            self.sort();
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Windows {
    pub sell: Option<ShopList>,
    pub buy: Option<ShopList>,
    pub warehouse: Option<WarehouseList>,
    pub warehouse_deposit: bool,
    pub board: Option<ShowBoard>,
    pub chosen_item_id: Option<i32>,
    pub quests: Vec<QuestState>,
    pub enchant_result: Option<i32>,
    pub learnable: Option<AcquireSkillList>,
    pub skill_info: Option<AcquireSkillInfo>,
    pub party_rooms: Option<PartyMatchList>,
    pub party_room: Option<PartyMatchDetail>,
    pub minimap: Option<ShowMinimap>,
    pub pet_items: Vec<InventoryItem>,
    pub package_targets: Option<PackageToList>,
    pub package_items: Option<PackageSendableList>,
    pub multisell: Option<crate::multisell::MultiSellState>,
    pub town_map: Option<ShowTownMap>,
    pub seeds: Option<BuyListSeed>,
    pub manor: Manor,
    pub hero_tower: HeroTower,
    pub radar: RadarState,
    pub preview_list: Option<ShopPreviewList>,
    pub preview: Option<ShopPreviewInfo>,
    pub party_join_result: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Manor {
    pub castles: Vec<ManorEntry>,
    pub seed_info: Option<SeedInfo>,
    pub crop_info: Option<CropInfo>,
    pub default_info: Option<ManorDefaultInfo>,
    pub seed_setting: Option<SeedSetting>,
    pub crop_setting: Option<CropSettingInfo>,
    pub sell_crop_list: Option<SellCropList>,
    pub procure_crop_detail: Option<ProcureCropDetail>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeroTower {
    pub list: Option<HeroList>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Siege {
    pub info: Option<CastleSiegeInfo>,
    pub attackers: Option<SiegeClanList>,
    pub defenders: Option<SiegeClanList>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Recipes {
    pub book: Option<RecipeBookItemList>,
    pub make_info: Option<RecipeItemMakeInfo>,
    pub shop_manage: Option<RecipeShopManageList>,
    pub shop_sell: Option<RecipeShopSellList>,
    pub shop_item: Option<RecipeShopItemInfo>,
    pub shop_messages: HashMap<i32, String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rare {
    pub calculator: Option<i32>,
    pub race: Option<MonRaceInfo>,
    pub xmas_seal: Option<i32>,
    pub ssq_status: Option<SsqStatus>,
    pub ssq_sky: Option<u16>,
    pub agit_deco: Option<i32>,
}

#[cfg(test)]
mod friend_tests {
    use super::*;
    use l2_net::game::{L2Friend, L2FriendStatus};

    fn packet(action: i32, id: i32, name: &str, online: i32, tail: i32) -> L2Friend {
        L2Friend { values: [action, id], name: name.into(), tail: [online, tail] }
    }

    fn names(friends: &Friends) -> Vec<&str> {
        friends.list.iter().map(|friend| friend.name.as_str()).collect()
    }

    #[test]
    fn actions_add_update_and_remove_and_the_list_stays_online_first_then_by_name() {
        let mut friends = Friends::default();
        friends.apply_packet(&packet(FRIEND_ACTION_ADD, 1, "Zed", 0, 0));
        friends.apply_packet(&packet(FRIEND_ACTION_ADD, 2, "Ana", 0, 0));
        friends.apply_packet(&packet(FRIEND_ACTION_ADD, 3, "Bo", 1, 9));
        assert_eq!(names(&friends), vec!["Bo", "Ana", "Zed"]);
        friends.apply_packet(&packet(FRIEND_ACTION_UPDATE, 4, "Zed", 1, 0));
        assert_eq!(names(&friends), vec!["Bo", "Zed", "Ana"]);
        assert_eq!(friends.list[1].object_id, 4);
        friends.apply_packet(&packet(FRIEND_ACTION_UPDATE, 5, "Nobody", 1, 0));
        assert_eq!(friends.list.len(), 3);
        friends.apply_packet(&packet(FRIEND_ACTION_REMOVE, 0, "Bo", 0, 0));
        assert_eq!(names(&friends), vec!["Zed", "Ana"]);
    }

    #[test]
    fn the_status_packet_rewrites_online_and_id_by_name_only() {
        let mut friends = Friends::default();
        friends.apply_packet(&packet(FRIEND_ACTION_ADD, 1, "Ana", 0, 7));
        friends.apply_status(&L2FriendStatus { value: 1, name: "Ana".into(), status: 44 });
        assert_eq!((friends.list[0].online, friends.list[0].object_id, friends.list[0].tail), (true, 44, 7));
        friends.apply_status(&L2FriendStatus { value: 0, name: "Other".into(), status: 1 });
        assert_eq!(friends.list.len(), 1);
    }

    #[test]
    fn the_name_order_is_case_sensitive() {
        let mut friends = Friends::default();
        friends.apply_packet(&packet(FRIEND_ACTION_ADD, 1, "bob", 0, 0));
        friends.apply_packet(&packet(FRIEND_ACTION_ADD, 2, "Zoe", 0, 0));
        assert_eq!(names(&friends), vec!["Zoe", "bob"]);
    }

    #[test]
    fn member_positions_keep_insertion_order_like_the_pairs_of_a_tmap() {
        let mut positions = PositionMap::default();
        for key in [9, 3, 7] {
            positions.insert(key, [key, 0, 0]);
        }
        assert_eq!(positions.insert(3, [30, 0, 0]), Some([3, 0, 0]));
        assert_eq!(positions.iter().map(|(k, v)| (*k, v[0])).collect::<Vec<_>>(), vec![(9, 9), (3, 30), (7, 7)]);
        assert_eq!(positions.remove(&9), Some([9, 0, 0]));
        positions.insert(9, [1, 1, 1]);
        assert_eq!(positions.iter().map(|(k, _)| *k).collect::<Vec<_>>(), vec![3, 7, 9]);
        assert!(positions.contains_key(&7) && !positions.contains_key(&8));
        assert_eq!(positions.len(), 3);
    }
}

#[cfg(test)]
mod pledge_rows_tests {
    use super::*;

    fn member(name: &str, level: i32, class_id: i32, online: i32) -> PledgeMember {
        PledgeMember { name: name.into(), level, class_id, unknown: 0, flag: 0, online_object_id: online }
    }

    #[test]
    fn add_member_1004dc50_on_an_existing_row_keeps_param1_and_only_moves_level_class_and_icon() {
        let mut pledge = Pledge::default();
        pledge.upsert(&member("Luzal", 10, 3, 77));
        assert_eq!((pledge.members[0].online_object_id, pledge.listed_online(0)), (77, true));
        pledge.upsert(&member("LUZAL", 11, -1, 0));
        assert_eq!(pledge.members.len(), 1);
        assert_eq!((pledge.members[0].level, pledge.members[0].class_id), (11, 3));
        assert_eq!((pledge.members[0].online_object_id, pledge.listed_online(0)), (77, false));
        pledge.upsert(&member("luzal", 12, 5, 90));
        assert_eq!((pledge.members[0].class_id, pledge.members[0].online_object_id, pledge.listed_online(0)), (5, 77, true));
    }

    #[test]
    fn update_member_1004df30_offline_keeps_class_and_params_online_replaces_them() {
        let mut pledge = Pledge::default();
        pledge.upsert(&member("Luzal", 10, 3, 77));
        pledge.update_existing(&member("luzal", 20, 9, 0));
        assert_eq!((pledge.members[0].level, pledge.members[0].class_id, pledge.members[0].online_object_id), (20, 3, 77));
        assert!(!pledge.listed_online(0));
        pledge.update_existing(&member("Luzal", 21, -1, 91));
        assert_eq!((pledge.members[0].class_id, pledge.members[0].online_object_id, pledge.listed_online(0)), (3, 91, true));
        pledge.update_existing(&member("Outro", 5, 1, 4));
        assert_eq!(pledge.members.len(), 1);
        pledge.upsert(&member("Amigo", 5, 1, 0));
        pledge.remove("LUZAL");
        assert_eq!(pledge.members.len(), 1);
        assert!(!pledge.listed_online(0));
    }
}
