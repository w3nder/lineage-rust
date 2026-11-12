use std::io;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;
use std::thread::{self, JoinHandle};
use killer_platform::time::Duration;

use crate::replay::{PlayEvent, ReplayPlayer, Recorder, ReplayError};

use l2_net::game::packets::{
    BuyOffer, ItemOrder, Macro, SellToStoreItem, StoreBuyOrder, StoreOffer, multi_sell_choose, request_add_trade_item,
    request_ally_crest, request_ally_dismiss, request_ally_info, request_ally_leave, request_answer_friend_invite,
    request_bbs_write, request_buy_item, request_crystallize_item, request_delete_macro, request_dismiss_ally,
    request_enchant_item, request_friend_del, request_friend_invite, request_friend_list, request_give_nick_name,
    request_henna_equip, request_henna_item_info, request_henna_list, request_henna_unequip, request_henna_unequip_info,
    request_henna_unequip_list, request_join_ally, request_join_party_room,
    request_join_pledge, request_make_macro, request_manage_party_room, request_oust_pledge_member,
    request_package_send, request_package_sendable_item_list, request_party_match_list, request_pet_get_item,
    request_pledge_crest, request_pledge_member_list, request_pledge_power, request_preview_item,
    request_private_store_buy, request_private_store_list_buy, request_private_store_list_sell,
    request_private_store_manage_buy, request_private_store_manage_cancel_buy, request_private_store_manage_cancel_sell,
    request_private_store_manage_sell, request_private_store_quit_buy, request_private_store_quit_sell,
    request_private_store_sell, request_procure_crop,
    request_quest_abort, request_restart_point, request_sell_item, request_send_friend_msg, request_set_ally_crest,
    request_set_pledge_crest, request_show_board, request_show_minimap, request_trade, request_trade_done,
    request_tutorial_client_event, request_tutorial_link_html, request_tutorial_pass_cmd,
    request_tutorial_question_mark, request_user_command, request_withdrawal_pledge, send_warehouse_deposit_list,
    send_warehouse_withdraw_list, set_private_store_list_buy, set_private_store_list_sell, set_private_store_msg_buy,
    set_private_store_msg_sell,
};
use l2_net::game::packets::{
    CropProcure, CropSetting, request_buy_seed, request_castle_siege_attacker_list, request_castle_siege_defender_list,
    request_castle_siege_info, request_confirm_castle_siege_waiting_list, request_dismiss_party_room,
    request_ex_accept_join_mpcc, request_ex_ask_join_mpcc, request_ex_enchant_skill, request_ex_enchant_skill_info,
    request_ex_oust_from_mpcc, request_ex_pledge_crest_large, request_ex_set_pledge_crest_large,
    request_join_castle_siege, request_magic_skill_list, request_manor_list, request_oust_from_party_room, request_petition,
    request_petition_cancel, request_gm_command, request_gm_snoop_end, request_observer_end, request_olympiad_match_list, request_olympiad_observer_end, request_petition_vote, send_bypass_build_cmd, request_procure_crop_list, request_recipe_book_open, request_recipe_item_delete,
    request_recipe_item_make_info, request_recipe_item_make_self, request_recipe_shop_list_set,
    request_recipe_shop_make_do, request_recipe_shop_make_info, request_recipe_shop_manage_cancel,
    request_recipe_shop_manage_list, request_recipe_shop_manage_quit, request_recipe_shop_message_set,
    request_recipe_shop_sell_list, request_set_castle_siege_time, request_set_crop, request_set_seed, request_siege_info, request_ssq_status,
    request_start_pledge_war, request_stop_pledge_war, request_withdraw_party_room, request_write_hero_words,
    vote_sociality,
};
use l2_net::game::packets::{
    NewCharacter, action, answer_trade_request, appearing, attack_request, cannot_move_anymore,
    cannot_move_anymore_in_vehicle, character_create, character_delete, character_restore, dlg_answer, finish_rotating,
    game_guard_reply, logout, move_backward_to_location, new_character, protocol_version, request_acquire_skill,
    request_acquire_skill_info, request_action_use, request_answer_join_ally, request_answer_join_party,
    request_answer_join_pledge, request_auto_soul_shot, request_block, request_bypass_to_server,
    request_change_move_type, request_change_party_leader, request_change_pet_name, request_change_wait_type,
    request_destroy_item, request_dismiss_party, request_drop_item, request_get_item_from_pet, request_give_item_to_pet, request_gm_list, request_item_list,
    request_join_party, request_link_html, request_magic_skill_use, request_oust_party_member, request_pet_use_item,
    request_pledge_info, request_quest_list, request_reload, request_restart, request_shortcut_del,
    request_remain_time, request_shortcut_reg, request_shortcut_use, request_skill_cool_time, request_skill_list, request_social_action, request_target_cancel, request_target_cancel_with,
    request_unequip_item, request_withdrawal_party, net_ping, revive_reply, say2, send_time_check, start_rotating, use_item, validate_position, validate_position_in_vehicle,
};
use l2_net::login::GameServer;
use l2_net::transport::{GameConnection, authenticate_choosing};

use crate::session::{Session, SessionEvent};
use l2_net::game::packets::{request_get_off_vehicle, request_get_on_vehicle, request_move_to_location_in_vehicle};

#[derive(Debug, Clone, PartialEq)]
pub enum NetCommand {
    Select(i32),
    SelectTransfer { slot: i32, position: [i32; 3] },
    Restart,
    MoveTo { target: [i32; 3], origin: [i32; 3] },
    GetOnVehicle { vehicle_id: i32, position: [i32; 3] },
    GetOffVehicle { vehicle_id: i32, position: [i32; 3] },
    MoveToInVehicle { vehicle_id: i32, local_target: [i32; 3], local_origin: [i32; 3] },
    Validate { position: [i32; 3], heading: i32 },
    ValidateInVehicle { vehicle_id: i32, position: [i32; 3], heading: i32 },
    Say { text: String, kind: i32, target: String },
    Action { object_id: i32, origin: [i32; 3], shift: bool },
    RequestMagicSkillList { first: i32, second: i32, third: i32 },
    Appearing,
    RequestItemList,
    UseItem { object_id: i32, ctrl: bool },
    Unequip { body_part: i32 },
    Bypass { command: String },
    LinkHtml { link: String },
    UseSkill { skill_id: i32, ctrl: bool, shift: bool },
    UseAction { id: i32, ctrl: bool, shift: bool },
    ShortcutReg { kind: i32, slot: i32, id: i32, character_type: i32 },
    ShortcutUse { slot: i32, page: i32, ctrl: u8 },
    ShortcutDel { slot: i32 },
    DestroyItem { object_id: i32, count: i32 },
    TargetCancel,
    TargetUnselect,
    RequestSkillCoolTime,
    AttackRequest { object_id: i32, origin: [i32; 3], shift: bool },
    DropItem { object_id: i32, count: i32, position: [i32; 3] },
    SocialAction { action: i32 },
    ChangeMoveType { running: bool },
    ChangeWaitType { standing: bool },
    AnswerJoinPledge { accept: bool },
    JoinParty { name: String, item_distribution: i32 },
    AnswerJoinParty { accept: bool },
    WithdrawParty,
    OustPartyMember { name: String },
    CannotMoveAnymore { position: [i32; 3], heading: i32 },
    CannotMoveAnymoreInVehicle { vehicle_id: i32, position: [i32; 3], heading: i32 },
    RequestSkillList,
    DismissParty,
    RequestGmList,
    ReviveReply { reply: i32 },
    SendTimeCheck { first: i32, second: i32 },
    RequestRemainTime,
    NetPing { first: i32, second: i32, third: i32 },
    AnswerTrade { accept: bool },
    StartRotating { degree: i32, side: i32 },
    FinishRotating { degree: i32, side: i32 },
    RequestQuestList,
    RequestPledgeInfo { pledge_id: i32 },
    AcquireSkillInfo { skill_id: i32, level: i32, mode: i32 },
    AcquireSkill { skill_id: i32, level: i32, mode: i32 },
    AnswerJoinAlly { accept: bool },
    ChangePetName { name: String },
    PetUseItem { object_id: i32 },
    GiveItemToPet { object_id: i32, count: i32 },
    GetItemFromPet { object_id: i32, count: i32 },
    Block { kind: i32, name: String },
    DlgAnswer { message_id: i32, answer: i32 },
    Reload,
    ChangePartyLeader { name: String },
    AutoSoulShot { item_id: i32, enable: bool },
    CharacterCreate(Box<NewCharacter>),
    CharacterDelete { slot: i32 },
    NewCharacter,
    CharacterRestore { slot: i32 },
    GameGuardReply { keys: [i32; 4] },
    Trade { object_id: i32 },
    AddTradeItem { trade_id: i32, object_id: i32, count: i32 },
    TradeDone { confirm: bool },
    SellItems { list_id: i32, orders: Vec<ItemOrder> },
    BuyItems { list_id: i32, orders: Vec<(i32, i32)> },
    JoinPledge { object_id: i32 },
    WithdrawPledge,
    OustPledgeMember { name: String },
    WarehouseDeposit { orders: Vec<(i32, i32)> },
    WarehouseWithdraw { orders: Vec<(i32, i32)> },
    RequestPledgeMemberList,
    SetPledgeCrest { data: Vec<u8> },
    SetAllyCrest { data: Vec<u8> },
    EnchantItem { object_id: i32 },
    QuestAbort { quest_id: i32 },
    RequestPledgeCrest { crest_id: i32 },
    RequestAllyCrest { crest_id: i32 },
    RestartPoint { point: i32 },
    CrystallizeItem { object_id: i32, count: i32 },
    PrivateStoreManageSell,
    PrivateStoreManageSellCancel,
    PrivateStoreListSell { store_id: i32 },
    SetPrivateStoreListSell { package_sale: bool, offers: Vec<StoreOffer> },
    PrivateStoreQuitSell,
    SetPrivateStoreMsgSell { message: String },
    PrivateStoreBuy { store_id: i32, orders: Vec<StoreBuyOrder> },
    TutorialLinkHtml { link: String },
    TutorialPassCmd { command: String },
    TutorialQuestionMark { mark_id: i32 },
    TutorialClientEvent { event_id: i32 },
    JoinAlly { object_id: i32 },
    AllyLeave,
    AllyDismiss { pledge_name: String },
    DismissAlly,
    RequestAllyInfo,
    PetGetItem { object_id: i32 },
    PrivateStoreManageBuy,
    PrivateStoreManageBuyCancel,
    PrivateStoreListBuy { store_id: i32 },
    SetPrivateStoreListBuy { offers: Vec<BuyOffer> },
    PrivateStoreQuitBuy,
    SetPrivateStoreMsgBuy { message: String },
    PrivateStoreSell { store_id: i32, items: Vec<SellToStoreItem> },
    PackageSendableItemList { target_id: i32 },
    PackageSend { target_id: i32, orders: Vec<(i32, i32)> },
    MultiSellChoose { list_id: i32, entry_id: i32, amount: i32 },
    UserCommand { command: i32 },
    HennaList,
    HennaItemInfo { symbol_id: i32 },
    HennaEquip { symbol_id: i32 },
    HennaUnequipList,
    HennaUnequipInfo { symbol_id: i32 },
    HennaUnequip { symbol_id: i32 },
    PledgePower { member_id: i32, action: i32, powers: [u8; 32] },
    MakeMacro(Box<Macro>),
    DeleteMacro { id: i32 },
    ProcureCrop { manor_id: i32, orders: Vec<ItemOrder> },
    PreviewItem { header: i32, list_id: i32, items: Vec<i32> },
    SendFriendMsg { message: String, receiver: String },
    ShowMinimap,
    FriendInvite { name: String },
    AnswerFriendInvite { accept: bool },
    FriendList,
    FriendDel { name: String },
    GiveNickName { target: String, title: String },
    ShowBoard,
    BbsWrite { fields: Box<[String; 6]> },
    PartyMatchList { level: i32, location: i32, mode: i32 },
    ManagePartyRoom { room_id: i32, max_members: i32, min_level: i32, max_level: i32, loot_type: i32, title: String },
    JoinPartyRoom { room_id: i32 },
    JoinPartyRoomAt { room_id: i32, location: i32 },
    RequestSiegeInfo,
    StartPledgeWar { pledge_name: String },
    StopPledgeWar { pledge_name: String },
    Petition { content: String, kind: i32 },
    PetitionCancel { content: String },
    PetitionVote { first: i32, index: i32, text: String },
    BypassBuildCmd { command: String },
    ObserverEnd,
    GmCommand { name: String, kind: i32 },
    GmSnoopEnd { snoop_id: i32 },
    OlympiadObserverEnd,
    OlympiadMatchList,
    CastleSiegeInfo { castle_id: i32 },
    CastleSiegeAttackerList { castle_id: i32 },
    CastleSiegeDefenderList { castle_id: i32 },
    JoinCastleSiege { castle_id: i32, attacker: bool, join: bool },
    ConfirmCastleSiegeWaitingList { castle_id: i32, pledge_id: i32, approved: bool },
    SetCastleSiegeTime { castle_id: i32, time: i32 },
    RecipeBookOpen { common: bool },
    RecipeItemDelete { recipe_id: i32 },
    RecipeItemMakeInfo { recipe_id: i32 },
    RecipeItemMakeSelf { recipe_id: i32 },
    RecipeShopManageList,
    RecipeShopMessageSet { message: String },
    RecipeShopListSet { items: Vec<(i32, i32)> },
    RecipeShopManageQuit,
    RecipeShopManageCancel,
    RecipeShopMakeInfo { manufacturer_id: i32, recipe_id: i32 },
    RecipeShopMakeDo { manufacturer_id: i32, recipe_id: i32, cost: i32 },
    RecipeShopSellList { manufacturer_id: i32 },
    VoteSociality { target_id: i32 },
    BuySeed { manor_id: i32, orders: Vec<(i32, i32)> },
    SsqStatus { page: u8 },
    OustFromPartyRoom { object_id: i32 },
    DismissPartyRoom { room_id: i32 },
    WithdrawPartyRoom { room_id: i32 },
    ExEnchantSkillInfo { skill_id: i32, level: i32 },
    ExEnchantSkill { skill_id: i32, level: i32 },
    ManorList,
    ProcureCropList { orders: Vec<CropProcure> },
    SetSeed { manor_id: i32, seeds: Vec<(i32, i32, i32)> },
    SetCrop { manor_id: i32, crops: Vec<CropSetting> },
    WriteHeroWords { words: String },
    ExAskJoinMpcc { name: String },
    ExAcceptJoinMpcc { accept: bool },
    ExOustFromMpcc { name: String },
    ExPledgeCrestLarge { crest_id: i32 },
    ExSetPledgeCrestLarge { data: Vec<u8> },
    Logout,
    Shutdown,
    Raw(Vec<u8>),
    ChooseServer(u8),
    NoticeAnswer(bool),
    CardAnswer(i32),
}

pub fn encode(command: &NetCommand) -> Option<Vec<u8>> {
    match command {
        NetCommand::Restart => Some(request_restart()),
        NetCommand::MoveTo { target, origin } => Some(move_backward_to_location(*target, *origin)),
        NetCommand::GetOnVehicle { vehicle_id, position } => Some(request_get_on_vehicle(*vehicle_id, *position)),
        NetCommand::GetOffVehicle { vehicle_id, position } => Some(request_get_off_vehicle(*vehicle_id, *position)),
        NetCommand::MoveToInVehicle { vehicle_id, local_target, local_origin } => {
            Some(request_move_to_location_in_vehicle(*vehicle_id, *local_target, *local_origin))
        }
        NetCommand::Validate { position, heading } => Some(validate_position(*position, *heading)),
        NetCommand::ValidateInVehicle { vehicle_id, position, heading } => Some(validate_position_in_vehicle(*vehicle_id, *position, *heading)),
        NetCommand::Say { text, kind, target } => Some(say2(text, *kind, target)),
        NetCommand::Action { object_id, origin, shift } => Some(action(*object_id, *origin, *shift)),
        NetCommand::RequestMagicSkillList { first, second, third } => Some(request_magic_skill_list(*first, *second, *third)),
        NetCommand::Appearing => Some(appearing()),
        NetCommand::RequestItemList => Some(request_item_list()),
        NetCommand::UseItem { object_id, ctrl } => Some(use_item(*object_id, *ctrl)),
        NetCommand::Unequip { body_part } => Some(request_unequip_item(*body_part)),
        NetCommand::Bypass { command } => Some(request_bypass_to_server(command)),
        NetCommand::LinkHtml { link } => Some(request_link_html(link)),
        NetCommand::UseSkill { skill_id, ctrl, shift } => Some(request_magic_skill_use(*skill_id, *ctrl, *shift)),
        NetCommand::UseAction { id, ctrl, shift } => Some(request_action_use(*id, *ctrl, *shift)),
        NetCommand::ShortcutReg { kind, slot, id, character_type } => {
            Some(request_shortcut_reg(*kind, *slot, *id, *character_type))
        }
        NetCommand::ShortcutUse { slot, page, ctrl } => Some(request_shortcut_use(*slot, *page, *ctrl)),
        NetCommand::ShortcutDel { slot } => Some(request_shortcut_del(*slot)),
        NetCommand::DestroyItem { object_id, count } => Some(request_destroy_item(*object_id, *count)),
        NetCommand::TargetCancel => Some(request_target_cancel()),
        NetCommand::TargetUnselect => Some(request_target_cancel_with(1)),
        NetCommand::RequestSkillCoolTime => Some(request_skill_cool_time()),
        NetCommand::AttackRequest { object_id, origin, shift } => Some(attack_request(*object_id, *origin, *shift)),
        NetCommand::DropItem { object_id, count, position } => Some(request_drop_item(*object_id, *count, *position)),
        NetCommand::SocialAction { action } => Some(request_social_action(*action)),
        NetCommand::ChangeMoveType { running } => Some(request_change_move_type(*running)),
        NetCommand::ChangeWaitType { standing } => Some(request_change_wait_type(*standing)),
        NetCommand::AnswerJoinPledge { accept } => Some(request_answer_join_pledge(*accept)),
        NetCommand::JoinParty { name, item_distribution } => Some(request_join_party(name, *item_distribution)),
        NetCommand::AnswerJoinParty { accept } => Some(request_answer_join_party(*accept)),
        NetCommand::WithdrawParty => Some(request_withdrawal_party()),
        NetCommand::OustPartyMember { name } => Some(request_oust_party_member(name)),
        NetCommand::CannotMoveAnymore { position, heading } => Some(cannot_move_anymore(*position, *heading)),
        NetCommand::CannotMoveAnymoreInVehicle { vehicle_id, position, heading } => {
            Some(cannot_move_anymore_in_vehicle(*vehicle_id, *position, *heading))
        }
        NetCommand::RequestSkillList => Some(request_skill_list()),
        NetCommand::DismissParty => Some(request_dismiss_party()),
        NetCommand::RequestGmList => Some(request_gm_list()),
        NetCommand::ReviveReply { reply } => Some(revive_reply(*reply)),
        NetCommand::SendTimeCheck { first, second } => Some(send_time_check(*first, *second)),
        NetCommand::RequestRemainTime => Some(request_remain_time()),
        NetCommand::NetPing { first, second, third } => Some(net_ping(*first, *second, *third)),
        NetCommand::AnswerTrade { accept } => Some(answer_trade_request(*accept)),
        NetCommand::StartRotating { degree, side } => Some(start_rotating(*degree, *side)),
        NetCommand::FinishRotating { degree, side } => Some(finish_rotating(*degree, *side)),
        NetCommand::RequestQuestList => Some(request_quest_list()),
        NetCommand::RequestPledgeInfo { pledge_id } => Some(request_pledge_info(*pledge_id)),
        NetCommand::AcquireSkillInfo { skill_id, level, mode } => {
            Some(request_acquire_skill_info(*skill_id, *level, *mode))
        }
        NetCommand::AcquireSkill { skill_id, level, mode } => {
            Some(request_acquire_skill(*skill_id, *level, *mode))
        }
        NetCommand::AnswerJoinAlly { accept } => Some(request_answer_join_ally(*accept)),
        NetCommand::ChangePetName { name } => Some(request_change_pet_name(name)),
        NetCommand::PetUseItem { object_id } => Some(request_pet_use_item(*object_id)),
        NetCommand::GiveItemToPet { object_id, count } => Some(request_give_item_to_pet(*object_id, *count)),
        NetCommand::GetItemFromPet { object_id, count } => Some(request_get_item_from_pet(*object_id, *count, 0)),
        NetCommand::Block { kind, name } => Some(request_block(*kind, name)),
        NetCommand::DlgAnswer { message_id, answer } => Some(dlg_answer(*message_id, *answer)),
        NetCommand::Reload => Some(request_reload()),
        NetCommand::ChangePartyLeader { name } => Some(request_change_party_leader(name)),
        NetCommand::AutoSoulShot { item_id, enable } => Some(request_auto_soul_shot(*item_id, *enable)),
        NetCommand::CharacterCreate(character) => Some(character_create(character)),
        NetCommand::CharacterDelete { slot } => Some(character_delete(*slot)),
        NetCommand::NewCharacter => Some(new_character()),
        NetCommand::CharacterRestore { slot } => Some(character_restore(*slot)),
        NetCommand::GameGuardReply { keys } => Some(game_guard_reply(*keys)),
        NetCommand::Trade { object_id } => Some(request_trade(*object_id)),
        NetCommand::AddTradeItem { trade_id, object_id, count } => {
            Some(request_add_trade_item(*trade_id, *object_id, *count))
        }
        NetCommand::TradeDone { confirm } => Some(request_trade_done(*confirm)),
        NetCommand::SellItems { list_id, orders } => Some(request_sell_item(*list_id, orders)),
        NetCommand::BuyItems { list_id, orders } => Some(request_buy_item(*list_id, orders)),
        NetCommand::JoinPledge { object_id } => Some(request_join_pledge(*object_id)),
        NetCommand::WithdrawPledge => Some(request_withdrawal_pledge()),
        NetCommand::OustPledgeMember { name } => Some(request_oust_pledge_member(name)),
        NetCommand::WarehouseDeposit { orders } => Some(send_warehouse_deposit_list(orders)),
        NetCommand::WarehouseWithdraw { orders } => Some(send_warehouse_withdraw_list(orders)),
        NetCommand::RequestPledgeMemberList => Some(request_pledge_member_list()),
        NetCommand::SetPledgeCrest { data } => Some(request_set_pledge_crest(data)),
        NetCommand::SetAllyCrest { data } => Some(request_set_ally_crest(data)),
        NetCommand::EnchantItem { object_id } => Some(request_enchant_item(*object_id)),
        NetCommand::QuestAbort { quest_id } => Some(request_quest_abort(*quest_id)),
        NetCommand::RequestPledgeCrest { crest_id } => Some(request_pledge_crest(*crest_id)),
        NetCommand::RequestAllyCrest { crest_id } => Some(request_ally_crest(*crest_id)),
        NetCommand::RestartPoint { point } => Some(request_restart_point(*point)),
        NetCommand::CrystallizeItem { object_id, count } => Some(request_crystallize_item(*object_id, *count)),
        NetCommand::PrivateStoreManageSell => Some(request_private_store_manage_sell()),
        NetCommand::PrivateStoreManageSellCancel => Some(request_private_store_manage_cancel_sell()),
        NetCommand::PrivateStoreListSell { store_id } => Some(request_private_store_list_sell(*store_id)),
        NetCommand::SetPrivateStoreListSell { package_sale, offers } => {
            Some(set_private_store_list_sell(*package_sale, offers))
        }
        NetCommand::PrivateStoreQuitSell => Some(request_private_store_quit_sell()),
        NetCommand::SetPrivateStoreMsgSell { message } => Some(set_private_store_msg_sell(message)),
        NetCommand::PrivateStoreBuy { store_id, orders } => Some(request_private_store_buy(*store_id, orders)),
        NetCommand::TutorialLinkHtml { link } => Some(request_tutorial_link_html(link)),
        NetCommand::TutorialPassCmd { command } => Some(request_tutorial_pass_cmd(command)),
        NetCommand::TutorialQuestionMark { mark_id } => Some(request_tutorial_question_mark(*mark_id)),
        NetCommand::TutorialClientEvent { event_id } => Some(request_tutorial_client_event(*event_id)),
        NetCommand::JoinAlly { object_id } => Some(request_join_ally(*object_id)),
        NetCommand::AllyLeave => Some(request_ally_leave()),
        NetCommand::AllyDismiss { pledge_name } => Some(request_ally_dismiss(pledge_name)),
        NetCommand::DismissAlly => Some(request_dismiss_ally()),
        NetCommand::RequestAllyInfo => Some(request_ally_info()),
        NetCommand::PetGetItem { object_id } => Some(request_pet_get_item(*object_id)),
        NetCommand::PrivateStoreManageBuy => Some(request_private_store_manage_buy()),
        NetCommand::PrivateStoreManageBuyCancel => Some(request_private_store_manage_cancel_buy()),
        NetCommand::PrivateStoreListBuy { store_id } => Some(request_private_store_list_buy(*store_id)),
        NetCommand::SetPrivateStoreListBuy { offers } => Some(set_private_store_list_buy(offers)),
        NetCommand::PrivateStoreQuitBuy => Some(request_private_store_quit_buy()),
        NetCommand::SetPrivateStoreMsgBuy { message } => Some(set_private_store_msg_buy(message)),
        NetCommand::PrivateStoreSell { store_id, items } => Some(request_private_store_sell(*store_id, items)),
        NetCommand::PackageSendableItemList { target_id } => Some(request_package_sendable_item_list(*target_id)),
        NetCommand::PackageSend { target_id, orders } => Some(request_package_send(*target_id, orders)),
        NetCommand::MultiSellChoose { list_id, entry_id, amount } => {
            Some(multi_sell_choose(*list_id, *entry_id, *amount))
        }
        NetCommand::UserCommand { command } => Some(request_user_command(*command)),
        NetCommand::HennaList => Some(request_henna_list(0)),
        NetCommand::HennaItemInfo { symbol_id } => Some(request_henna_item_info(*symbol_id)),
        NetCommand::HennaEquip { symbol_id } => Some(request_henna_equip(*symbol_id)),
        NetCommand::HennaUnequipList => Some(request_henna_unequip_list(0)),
        NetCommand::HennaUnequipInfo { symbol_id } => Some(request_henna_unequip_info(*symbol_id)),
        NetCommand::HennaUnequip { symbol_id } => Some(request_henna_unequip(*symbol_id)),
        NetCommand::PledgePower { member_id, action, powers } => Some(request_pledge_power(*member_id, *action, powers)),
        NetCommand::MakeMacro(m) => Some(request_make_macro(m)),
        NetCommand::DeleteMacro { id } => Some(request_delete_macro(*id)),
        NetCommand::ProcureCrop { manor_id, orders } => Some(request_procure_crop(*manor_id, orders)),
        NetCommand::PreviewItem { header, list_id, items } => Some(request_preview_item(*header, *list_id, items)),
        NetCommand::SendFriendMsg { message, receiver } => Some(request_send_friend_msg(message, receiver)),
        NetCommand::ShowMinimap => Some(request_show_minimap()),
        NetCommand::FriendInvite { name } => Some(request_friend_invite(name)),
        NetCommand::AnswerFriendInvite { accept } => Some(request_answer_friend_invite(*accept)),
        NetCommand::FriendList => Some(request_friend_list()),
        NetCommand::FriendDel { name } => Some(request_friend_del(name)),
        NetCommand::GiveNickName { target, title } => Some(request_give_nick_name(target, title)),
        NetCommand::ShowBoard => Some(request_show_board(0)),
        NetCommand::BbsWrite { fields } => {
            let [a, b, c, d, e, f] = &**fields;
            Some(request_bbs_write(&[a, b, c, d, e, f]))
        }
        NetCommand::PartyMatchList { level, location, mode } => {
            Some(request_party_match_list(*level, *location, *mode))
        }
        NetCommand::ManagePartyRoom { room_id, max_members, min_level, max_level, loot_type, title } => {
            Some(request_manage_party_room(*room_id, *max_members, *min_level, *max_level, *loot_type, title))
        }
        NetCommand::JoinPartyRoom { room_id } => Some(request_join_party_room(*room_id, 0)),
        NetCommand::JoinPartyRoomAt { room_id, location } => Some(request_join_party_room(*room_id, *location)),
        NetCommand::RequestSiegeInfo => Some(request_siege_info()),
        NetCommand::StartPledgeWar { pledge_name } => Some(request_start_pledge_war(pledge_name)),
        NetCommand::StopPledgeWar { pledge_name } => Some(request_stop_pledge_war(pledge_name)),
        NetCommand::Petition { content, kind } => Some(request_petition(content, *kind)),
        NetCommand::PetitionCancel { content } => Some(request_petition_cancel(content)),
        NetCommand::PetitionVote { first, index, text } => Some(request_petition_vote(*first, *index, text)),
        NetCommand::BypassBuildCmd { command } => Some(send_bypass_build_cmd(command)),
        NetCommand::ObserverEnd => Some(request_observer_end()),
        NetCommand::GmCommand { name, kind } => Some(request_gm_command(name, *kind)),
        NetCommand::GmSnoopEnd { snoop_id } => Some(request_gm_snoop_end(*snoop_id)),
        NetCommand::OlympiadObserverEnd => Some(request_olympiad_observer_end()),
        NetCommand::OlympiadMatchList => Some(request_olympiad_match_list()),
        NetCommand::CastleSiegeInfo { castle_id } => Some(request_castle_siege_info(*castle_id)),
        NetCommand::CastleSiegeAttackerList { castle_id } => Some(request_castle_siege_attacker_list(*castle_id)),
        NetCommand::CastleSiegeDefenderList { castle_id } => Some(request_castle_siege_defender_list(*castle_id)),
        NetCommand::JoinCastleSiege { castle_id, attacker, join } => {
            Some(request_join_castle_siege(*castle_id, *attacker, *join))
        }
        NetCommand::ConfirmCastleSiegeWaitingList { castle_id, pledge_id, approved } => {
            Some(request_confirm_castle_siege_waiting_list(*castle_id, *pledge_id, *approved))
        }
        NetCommand::SetCastleSiegeTime { castle_id, time } => Some(request_set_castle_siege_time(*castle_id, *time)),
        NetCommand::RecipeBookOpen { common } => Some(request_recipe_book_open(*common)),
        NetCommand::RecipeItemDelete { recipe_id } => Some(request_recipe_item_delete(*recipe_id)),
        NetCommand::RecipeItemMakeInfo { recipe_id } => Some(request_recipe_item_make_info(*recipe_id)),
        NetCommand::RecipeItemMakeSelf { recipe_id } => Some(request_recipe_item_make_self(*recipe_id)),
        NetCommand::RecipeShopManageList => Some(request_recipe_shop_manage_list()),
        NetCommand::RecipeShopMessageSet { message } => Some(request_recipe_shop_message_set(message)),
        NetCommand::RecipeShopListSet { items } => Some(request_recipe_shop_list_set(items)),
        NetCommand::RecipeShopManageQuit => Some(request_recipe_shop_manage_quit()),
        NetCommand::RecipeShopManageCancel => Some(request_recipe_shop_manage_cancel()),
        NetCommand::RecipeShopMakeInfo { manufacturer_id, recipe_id } => {
            Some(request_recipe_shop_make_info(*manufacturer_id, *recipe_id))
        }
        NetCommand::RecipeShopMakeDo { manufacturer_id, recipe_id, cost } => {
            Some(request_recipe_shop_make_do(*manufacturer_id, *recipe_id, *cost))
        }
        NetCommand::RecipeShopSellList { manufacturer_id } => Some(request_recipe_shop_sell_list(*manufacturer_id)),
        NetCommand::VoteSociality { target_id } => Some(vote_sociality(*target_id)),
        NetCommand::BuySeed { manor_id, orders } => Some(request_buy_seed(*manor_id, orders)),
        NetCommand::SsqStatus { page } => Some(request_ssq_status(*page)),
        NetCommand::OustFromPartyRoom { object_id } => Some(request_oust_from_party_room(*object_id)),
        NetCommand::DismissPartyRoom { room_id } => Some(request_dismiss_party_room(*room_id, 0)),
        NetCommand::WithdrawPartyRoom { room_id } => Some(request_withdraw_party_room(*room_id, 0)),
        NetCommand::ExEnchantSkillInfo { skill_id, level } => Some(request_ex_enchant_skill_info(*skill_id, *level)),
        NetCommand::ExEnchantSkill { skill_id, level } => Some(request_ex_enchant_skill(*skill_id, *level)),
        NetCommand::ManorList => Some(request_manor_list()),
        NetCommand::ProcureCropList { orders } => Some(request_procure_crop_list(orders)),
        NetCommand::SetSeed { manor_id, seeds } => Some(request_set_seed(*manor_id, seeds)),
        NetCommand::SetCrop { manor_id, crops } => Some(request_set_crop(*manor_id, crops)),
        NetCommand::WriteHeroWords { words } => Some(request_write_hero_words(words)),
        NetCommand::ExAskJoinMpcc { name } => Some(request_ex_ask_join_mpcc(name)),
        NetCommand::ExAcceptJoinMpcc { accept } => Some(request_ex_accept_join_mpcc(*accept)),
        NetCommand::ExOustFromMpcc { name } => Some(request_ex_oust_from_mpcc(name)),
        NetCommand::ExPledgeCrestLarge { crest_id } => Some(request_ex_pledge_crest_large(*crest_id)),
        NetCommand::ExSetPledgeCrestLarge { data } => Some(request_ex_set_pledge_crest_large(data)),
        NetCommand::Raw(body) => Some(body.clone()),
        NetCommand::Select(_) | NetCommand::SelectTransfer { .. } | NetCommand::Logout | NetCommand::Shutdown | NetCommand::ChooseServer(_) | NetCommand::NoticeAnswer(_) | NetCommand::CardAnswer(_) => None,
    }
}

#[derive(Debug)]
pub enum NetEvent {
    Connecting,
    Notice(l2_net::login::LoginNotice),
    CardCheck { card_id: i32, position: i8, retry: bool },
    ServerList(Vec<GameServer>),
    LoginOk(GameServer),
    Session(SessionEvent),
    Error(String),
    LoginRefused { code: u8, text: String },
    LoginBlocked { flags: Option<u32>, messages: Vec<String>, text: String },
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketRecord {
    pub outgoing: bool,
    pub body: Vec<u8>,
}

pub struct NetHandle {
    pub commands: Sender<NetCommand>,
    pub events: Receiver<NetEvent>,
    pub packets: Option<Receiver<PacketRecord>>,
    pub recorder: Arc<Mutex<Recorder>>,
    pub replay: Option<ReplayLink>,
    join: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReplayCommand {
    Ready(bool),
    Input { key: u32, action: u32, level_loaded: bool },
}

pub struct ReplayLink {
    pub events: Receiver<PlayEvent>,
    pub control: Sender<ReplayCommand>,
}

pub fn now_seconds() -> f64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f64() + 1.0
}

impl Drop for NetHandle {
    fn drop(&mut self) {
        let _ = self.commands.send(NetCommand::Shutdown);
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
    }
}

const IDLE_POLL: Duration = Duration::from_millis(20);
const NET_TIMEOUT: Duration = Duration::from_secs(10);

pub fn spawn(host: String, port: u16, account: String, password: String) -> NetHandle {
    spawn_choosing(host, port, account, password, false)
}

pub fn spawn_choosing(host: String, port: u16, account: String, password: String, choosing: bool) -> NetHandle {
    spawn_inner(host, port, account, password, choosing, false)
}

pub fn spawn_tapped(host: String, port: u16, account: String, password: String) -> NetHandle {
    spawn_inner(host, port, account, password, true, true)
}

fn spawn_inner(host: String, port: u16, account: String, mut password: String, choosing: bool, tapped: bool) -> NetHandle {
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (ev_tx, ev_rx) = mpsc::channel();
    let (tap_tx, tap_rx) = mpsc::channel();
    let recorder = Arc::new(Mutex::new(Recorder::new()));
    let packet_recorder = Arc::clone(&recorder);
    let tap = move |outgoing: bool, body: &[u8]| {
        if tapped {
            let _ = tap_tx.send(PacketRecord { outgoing, body: body.to_vec() });
        }
    };
    let join = thread::Builder::new().name("killer-net".into()).spawn(move || {
        let _ = ev_tx.send(NetEvent::Connecting);
        let choose = |servers: &[GameServer]| {
            if !choosing {
                return servers.first().map(|server| server.id);
            }
            let _ = ev_tx.send(NetEvent::ServerList(servers.to_vec()));
            loop {
                match cmd_rx.recv() {
                    Ok(NetCommand::ChooseServer(id)) => return Some(id),
                    Ok(NetCommand::Logout | NetCommand::Shutdown) | Err(_) => return None,
                    Ok(_) => {}
                }
            }
        };
        let notice = |info: &l2_net::login::LoginNotice| {
            if !choosing {
                return true;
            }
            let _ = ev_tx.send(NetEvent::Notice(*info));
            loop {
                match cmd_rx.recv() {
                    Ok(NetCommand::NoticeAnswer(agreed)) => return agreed,
                    Ok(NetCommand::Logout | NetCommand::Shutdown) | Err(_) => return false,
                    Ok(_) => {}
                }
            }
        };
        let card = |card_id: i32, position: i8, retry: bool| {
            if !choosing {
                return None;
            }
            let _ = ev_tx.send(NetEvent::CardCheck { card_id, position, retry });
            loop {
                match cmd_rx.recv() {
                    Ok(NetCommand::CardAnswer(value)) => return Some(value),
                    Ok(NetCommand::Logout | NetCommand::Shutdown) | Err(_) => return None,
                    Ok(_) => {}
                }
            }
        };
        let auth = match authenticate_choosing(&host, port, &account, &password, NET_TIMEOUT, notice, choose, card) {
            Ok(a) => a,
            Err(e) => {
                let _ = ev_tx.send(match &e {
                    l2_net::transport::TransportError::Login(l2_net::login::LoginError::Refused { code, .. }) => NetEvent::LoginRefused { code: *code, text: e.to_string() },
                    l2_net::transport::TransportError::Login(l2_net::login::LoginError::Blocked { flags }) => NetEvent::LoginBlocked { flags: Some(*flags), messages: Vec::new(), text: e.to_string() },
                    l2_net::transport::TransportError::Login(l2_net::login::LoginError::BlockedWithMessages { messages }) => NetEvent::LoginBlocked { flags: None, messages: messages.clone(), text: e.to_string() },
                    _ => NetEvent::Error(e.to_string()),
                });
                let _ = ev_tx.send(NetEvent::Closed);
                return;
            }
        };
        password.clear();
        password.shrink_to_fit();
        let _ = ev_tx.send(NetEvent::LoginOk(auth.server.clone()));
        let mut game = match GameConnection::connect(&auth.server.address(), NET_TIMEOUT) {
            Ok(g) => g,
            Err(e) => {
                let _ = ev_tx.send(NetEvent::Error(e.to_string()));
                let _ = ev_tx.send(NetEvent::Closed);
                return;
            }
        };
        tap(true, &protocol_version());
        if let Err(e) = game.send(protocol_version()) {
            let _ = ev_tx.send(NetEvent::Error(e.to_string()));
            let _ = ev_tx.send(NetEvent::Closed);
            return;
        }
        let mut session = Session::new(account, auth);
        loop {
            while let Ok(cmd) = cmd_rx.try_recv() {
                let body = match cmd {
                    NetCommand::Select(slot) => session.select(slot),
                    NetCommand::SelectTransfer { slot, position } => session.select_transfer(slot, position),
                    NetCommand::Logout | NetCommand::Shutdown => {
                        let _ = game.send(logout());
                        let _ = ev_tx.send(NetEvent::Closed);
                        return;
                    }
                    NetCommand::ChooseServer(_) | NetCommand::NoticeAnswer(_) | NetCommand::CardAnswer(_) => continue,
                    other => match encode(&other) {
                        Some(body) => body,
                        None => continue,
                    },
                };
                tap(true, &body);
                if let Err(e) = game.send(body) {
                    let _ = ev_tx.send(NetEvent::Error(e.to_string()));
                    let _ = ev_tx.send(NetEvent::Closed);
                    return;
                }
            }
            let body = match game.receive_timeout(IDLE_POLL) {
                Ok(Some(b)) => b,
                Ok(None) => continue,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    let _ = ev_tx.send(NetEvent::Closed);
                    return;
                }
                Err(e) => {
                    let _ = ev_tx.send(NetEvent::Error(e.to_string()));
                    let _ = ev_tx.send(NetEvent::Closed);
                    return;
                }
            };
            tap(false, &body);
            if let Ok(mut recording) = packet_recorder.lock() {
                recording.add_packet(now_seconds(), &body);
            }
            let (event, outgoing, crypt) = session.on_body(&body);
            if let Some(c) = crypt {
                game.enable_crypt(c);
            }
            for out in outgoing {
                tap(true, &out);
                if let Err(e) = game.send(out) {
                    let _ = ev_tx.send(NetEvent::Error(e.to_string()));
                    let _ = ev_tx.send(NetEvent::Closed);
                    return;
                }
            }
            if let Some(ev) = event {
                let _ = ev_tx.send(NetEvent::Session(ev));
            }
        }
    });
    NetHandle { commands: cmd_tx, events: ev_rx, packets: tapped.then_some(tap_rx), recorder, replay: None, join: join.ok() }
}

const REPLAY_POLL: Duration = Duration::from_millis(16);

pub fn spawn_replay(bytes: &[u8]) -> Result<NetHandle, ReplayError> {
    let mut player = ReplayPlayer::open(bytes)?;
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (ev_tx, ev_rx) = mpsc::channel();
    let (play_tx, play_rx) = mpsc::channel();
    let (control_tx, control_rx) = mpsc::channel();
    let join = thread::Builder::new()
        .name("killer-replay".into())
        .spawn(move || {
            let mut session = Session::replaying();
            let mut ready = false;
            let mut last = Instant::now();
            loop {
                while let Ok(command) = cmd_rx.try_recv() {
                    if matches!(command, NetCommand::Logout | NetCommand::Shutdown) {
                        let _ = ev_tx.send(NetEvent::Closed);
                        return;
                    }
                }
                while let Ok(command) = control_rx.try_recv() {
                    match command {
                        ReplayCommand::Ready(value) => ready = value,
                        ReplayCommand::Input { key, action, level_loaded } => {
                            player.input(key, action, level_loaded);
                        }
                    }
                }
                let now = Instant::now();
                let dt = now.duration_since(last).as_secs_f32();
                last = now;
                for event in player.tick(dt, ready) {
                    match event {
                        PlayEvent::Packet(body) => {
                            let (session_event, _, _) = session.on_body(&body);
                            if let Some(session_event) = session_event {
                                let _ = ev_tx.send(NetEvent::Session(session_event));
                            }
                        }
                        PlayEvent::Ended => {
                            let _ = play_tx.send(PlayEvent::Ended);
                            let _ = ev_tx.send(NetEvent::Closed);
                            return;
                        }
                        other => {
                            let _ = play_tx.send(other);
                        }
                    }
                }
                thread::sleep(REPLAY_POLL);
            }
        })
        .ok();
    Ok(NetHandle {
        commands: cmd_tx,
        events: ev_rx,
        packets: None,
        recorder: Arc::new(Mutex::new(Recorder::new())),
        replay: Some(ReplayLink { events: play_rx, control: control_tx }),
        join,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpListener;

    use l2_net::game::GameCrypt;
    use l2_net::game::packets::CHAT_TELL;
    use l2_net::login::LoginCipher;
    use l2_net::transport::frame;

    #[test]
    fn spawn_runs_login_and_reports_events_in_order() {
        let game_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let game_addr = game_listener.local_addr().unwrap();

        let game_thread = thread::spawn(move || {
            let (mut socket, _) = game_listener.accept().unwrap();
            let _protocol_version = frame::read(&mut socket).unwrap();
            let key_packet = vec![0u8, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0];
            socket.write_all(&frame::encode(&key_packet)).unwrap();
            let mut auth_login = frame::read(&mut socket).unwrap();
            GameCrypt::from_key_packet(&key_packet).unwrap().decrypt(&mut auth_login);
            assert_eq!(auth_login[0], 0x08);
            let mut chars = vec![0x13u8];
            chars.extend_from_slice(&0i32.to_le_bytes());
            let mut crypt = GameCrypt::from_key_packet(&key_packet).unwrap();
            crypt.decrypt(&mut vec![0u8; auth_login.len()]);
            crypt.encrypt(&mut chars);
            socket.write_all(&frame::encode(&chars)).unwrap();
            let mut moved = frame::read(&mut socket).unwrap();
            crypt.decrypt(&mut moved);
            assert_eq!(moved, move_backward_to_location([100, 200, -30], [10, 20, -3]));
        });

        let login_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let login_addr = login_listener.local_addr().unwrap();
        let login_thread = thread::spawn(move || {
            let cipher = LoginCipher::c4();
            let (mut socket, _) = login_listener.accept().unwrap();
            let mut init = vec![0u8];
            init.extend_from_slice(&[0u8; 8]);
            socket.write_all(&frame::encode(&init)).unwrap();
            let _credentials = cipher.open(&frame::read(&mut socket).unwrap()).unwrap();
            let mut ok = vec![3u8];
            ok.extend_from_slice(&[1, 0, 0, 0, 2, 0, 0, 0]);
            socket.write_all(&frame::encode(&cipher.seal(&ok))).unwrap();
            let _list_req = cipher.open(&frame::read(&mut socket).unwrap()).unwrap();
            let mut list = vec![4u8, 1, 1, 1, 127, 0, 0, 1];
            list.extend_from_slice(&(game_addr.port() as u32).to_le_bytes());
            list.extend_from_slice(&[0, 1, 3, 0, 9, 0, 1, 0, 0, 0, 0, 0]);
            socket.write_all(&frame::encode(&cipher.seal(&list))).unwrap();
            let _play_req = cipher.open(&frame::read(&mut socket).unwrap()).unwrap();
            let mut playok = vec![7u8];
            playok.extend_from_slice(&[3, 0, 0, 0, 4, 0, 0, 0]);
            socket.write_all(&frame::encode(&cipher.seal(&playok))).unwrap();
        });

        let handle = spawn("127.0.0.1".into(), login_addr.port(), "conta".into(), "senha".into());

        match handle.events.recv_timeout(Duration::from_secs(5)).unwrap() {
            NetEvent::Connecting => {}
            other => panic!("esperava Connecting, veio {other:?}"),
        }
        match handle.events.recv_timeout(Duration::from_secs(5)).unwrap() {
            NetEvent::LoginOk(server) => assert_eq!(server.port, game_addr.port()),
            other => panic!("esperava LoginOk, veio {other:?}"),
        }
        match handle.events.recv_timeout(Duration::from_secs(5)).unwrap() {
            NetEvent::Session(SessionEvent::Characters(chars)) => assert!(chars.is_empty()),
            other => panic!("esperava Session(Characters), veio {other:?}"),
        }
        handle.commands.send(NetCommand::MoveTo { target: [100, 200, -30], origin: [10, 20, -3] }).unwrap();
        match handle.events.recv_timeout(Duration::from_secs(5)).unwrap() {
            NetEvent::Closed => {}
            other => panic!("esperava Closed, veio {other:?}"),
        }

        login_thread.join().unwrap();
        game_thread.join().unwrap();
    }

    #[test]
    fn commands_encode_the_client_packets() {
        assert_eq!(
            encode(&NetCommand::MoveTo { target: [1, 2, 3], origin: [4, 5, 6] }),
            Some(move_backward_to_location([1, 2, 3], [4, 5, 6]))
        );
        assert_eq!(
            encode(&NetCommand::Validate { position: [1, 2, 3], heading: 9 }),
            Some(validate_position([1, 2, 3], 9))
        );
        assert_eq!(
            encode(&NetCommand::ValidateInVehicle { vehicle_id: 5, position: [1, 2, 3], heading: 9 }),
            Some(validate_position_in_vehicle(5, [1, 2, 3], 9))
        );
        assert_eq!(
            encode(&NetCommand::Say { text: "oi".into(), kind: CHAT_TELL, target: "Luzal".into() }),
            Some(say2("oi", CHAT_TELL, "Luzal"))
        );
        assert_eq!(
            encode(&NetCommand::Action { object_id: 7, origin: [1, 2, 3], shift: true }),
            Some(action(7, [1, 2, 3], true))
        );
        assert_eq!(encode(&NetCommand::RequestMagicSkillList { first: 2, second: 1, third: 3 }), Some(request_magic_skill_list(2, 1, 3)));
        assert_eq!(encode(&NetCommand::Appearing), Some(appearing()));
        assert_eq!(encode(&NetCommand::Restart), Some(request_restart()));
        assert_eq!(encode(&NetCommand::Logout), None);
        assert_eq!(encode(&NetCommand::Select(1)), None);
    }

    #[test]
    fn frente_g_commands_encode_the_inventory_npc_and_skill_packets() {
        assert_eq!(encode(&NetCommand::RequestItemList), Some(request_item_list()));
        assert_eq!(encode(&NetCommand::UseItem { object_id: 1001, ctrl: false }), Some(use_item(1001, false)));
        assert_eq!(encode(&NetCommand::Unequip { body_part: 0x80 }), Some(request_unequip_item(0x80)));
        assert_eq!(
            encode(&NetCommand::Bypass { command: "npc_1_Chat 1".into() }),
            Some(request_bypass_to_server("npc_1_Chat 1"))
        );
        assert_eq!(encode(&NetCommand::LinkHtml { link: "x.htm".into() }), Some(request_link_html("x.htm")));
        assert_eq!(
            encode(&NetCommand::UseSkill { skill_id: 1177, ctrl: true, shift: false }),
            Some(request_magic_skill_use(1177, true, false))
        );
        assert_eq!(
            encode(&NetCommand::UseAction { id: 2, ctrl: false, shift: true }),
            Some(request_action_use(2, false, true))
        );
        assert_eq!(
            encode(&NetCommand::ShortcutReg { kind: 2, slot: 13, id: 1177, character_type: 3 }),
            Some(request_shortcut_reg(2, 13, 1177, 3))
        );
        assert_eq!(encode(&NetCommand::ShortcutDel { slot: 13 }), Some(request_shortcut_del(13)));
        assert_eq!(encode(&NetCommand::DestroyItem { object_id: 1001, count: 2 }), Some(request_destroy_item(1001, 2)));
        assert_eq!(encode(&NetCommand::TargetCancel), Some(request_target_cancel()));
        assert_eq!(encode(&NetCommand::TargetUnselect), Some(vec![0x37, 1, 0]));
        assert_eq!(encode(&NetCommand::RequestSkillCoolTime), Some(request_skill_cool_time()));
    }

    #[test]
    fn frente_p_commands_encode_the_world_and_lobby_packets() {
        assert_eq!(
            encode(&NetCommand::GetOnVehicle { vehicle_id: 500, position: [1, -2, 3] }),
            Some(request_get_on_vehicle(500, [1, -2, 3]))
        );
        assert_eq!(
            encode(&NetCommand::GetOffVehicle { vehicle_id: 500, position: [1, -2, 3] }),
            Some(request_get_off_vehicle(500, [1, -2, 3]))
        );
        assert_eq!(
            encode(&NetCommand::MoveToInVehicle { vehicle_id: 500, local_target: [1, -2, 3], local_origin: [4, 5, -6] }),
            Some(request_move_to_location_in_vehicle(500, [1, -2, 3], [4, 5, -6]))
        );
        assert_eq!(
            encode(&NetCommand::AttackRequest { object_id: 7, origin: [1, 2, 3], shift: false }),
            Some(attack_request(7, [1, 2, 3], false))
        );
        assert_eq!(
            encode(&NetCommand::DropItem { object_id: 1001, count: 2, position: [1, 2, 3] }),
            Some(request_drop_item(1001, 2, [1, 2, 3]))
        );
        assert_eq!(encode(&NetCommand::SocialAction { action: 3 }), Some(request_social_action(3)));
        assert_eq!(encode(&NetCommand::ChangeMoveType { running: false }), Some(request_change_move_type(false)));
        assert_eq!(encode(&NetCommand::ChangeWaitType { standing: true }), Some(request_change_wait_type(true)));
        assert_eq!(encode(&NetCommand::AnswerJoinPledge { accept: true }), Some(request_answer_join_pledge(true)));
        assert_eq!(
            encode(&NetCommand::JoinParty { name: "L".into(), item_distribution: 1 }),
            Some(request_join_party("L", 1))
        );
        assert_eq!(encode(&NetCommand::AnswerJoinParty { accept: false }), Some(request_answer_join_party(false)));
        assert_eq!(encode(&NetCommand::WithdrawParty), Some(request_withdrawal_party()));
        assert_eq!(encode(&NetCommand::OustPartyMember { name: "L".into() }), Some(request_oust_party_member("L")));
        assert_eq!(
            encode(&NetCommand::CannotMoveAnymore { position: [1, 2, 3], heading: 4 }),
            Some(cannot_move_anymore([1, 2, 3], 4))
        );
        assert_eq!(
            encode(&NetCommand::CannotMoveAnymoreInVehicle { vehicle_id: 5, position: [1, 2, 3], heading: 4 }),
            Some(cannot_move_anymore_in_vehicle(5, [1, 2, 3], 4))
        );
        assert_eq!(encode(&NetCommand::RequestSkillList), Some(request_skill_list()));
        assert_eq!(encode(&NetCommand::DismissParty), Some(request_dismiss_party()));
        assert_eq!(encode(&NetCommand::RequestGmList), Some(request_gm_list()));
        assert_eq!(encode(&NetCommand::ReviveReply { reply: 1 }), Some(revive_reply(1)));
        assert_eq!(encode(&NetCommand::SendTimeCheck { first: 1, second: 2 }), Some(send_time_check(1, 2)));
        assert_eq!(encode(&NetCommand::RequestRemainTime), Some(request_remain_time()));
        assert_eq!(encode(&NetCommand::NetPing { first: 1, second: 2, third: 3 }), Some(net_ping(1, 2, 3)));
        assert_eq!(
            encode(&NetCommand::ShortcutUse { slot: 13, page: 2, ctrl: 1 }),
            Some(request_shortcut_use(13, 2, 1))
        );
        assert_eq!(encode(&NetCommand::AnswerTrade { accept: true }), Some(answer_trade_request(true)));
        assert_eq!(encode(&NetCommand::StartRotating { degree: 90, side: 1 }), Some(start_rotating(90, 1)));
        assert_eq!(encode(&NetCommand::FinishRotating { degree: 90, side: 0 }), Some(finish_rotating(90, 0)));
        assert_eq!(encode(&NetCommand::RequestQuestList), Some(request_quest_list()));
        assert_eq!(encode(&NetCommand::RequestPledgeInfo { pledge_id: 9 }), Some(request_pledge_info(9)));
        assert_eq!(
            encode(&NetCommand::AcquireSkillInfo { skill_id: 3, level: 1, mode: 0 }),
            Some(request_acquire_skill_info(3, 1, 0))
        );
        assert_eq!(
            encode(&NetCommand::AcquireSkill { skill_id: 3, level: 1, mode: 0 }),
            Some(request_acquire_skill(3, 1, 0))
        );
        assert_eq!(encode(&NetCommand::AnswerJoinAlly { accept: true }), Some(request_answer_join_ally(true)));
        assert_eq!(encode(&NetCommand::ChangePetName { name: "W".into() }), Some(request_change_pet_name("W")));
        assert_eq!(encode(&NetCommand::PetUseItem { object_id: 1 }), Some(request_pet_use_item(1)));
        assert_eq!(encode(&NetCommand::GiveItemToPet { object_id: 1, count: 2 }), Some(request_give_item_to_pet(1, 2)));
        assert_eq!(
            encode(&NetCommand::GetItemFromPet { object_id: 1, count: 2 }),
            Some(request_get_item_from_pet(1, 2, 0))
        );
        assert_eq!(encode(&NetCommand::Block { kind: 0, name: "L".into() }), Some(request_block(0, "L")));
        assert_eq!(encode(&NetCommand::DlgAnswer { message_id: 1510, answer: 1 }), Some(dlg_answer(1510, 1)));
        assert_eq!(encode(&NetCommand::Reload), Some(request_reload()));
        assert_eq!(encode(&NetCommand::ChangePartyLeader { name: "L".into() }), Some(request_change_party_leader("L")));
        assert_eq!(
            encode(&NetCommand::AutoSoulShot { item_id: 1835, enable: true }),
            Some(request_auto_soul_shot(1835, true))
        );
        let character = NewCharacter {
            name: "L".into(),
            race: 0,
            sex: 0,
            class_id: 0,
            int_: 1,
            str_: 2,
            con: 3,
            men: 4,
            dex: 5,
            wit: 6,
            hair_style: 0,
            hair_color: 0,
            face: 0,
        };
        assert_eq!(
            encode(&NetCommand::CharacterCreate(Box::new(character.clone()))),
            Some(character_create(&character))
        );
        assert_eq!(encode(&NetCommand::CharacterDelete { slot: 1 }), Some(character_delete(1)));
        assert_eq!(encode(&NetCommand::NewCharacter), Some(new_character()));
        assert_eq!(encode(&NetCommand::CharacterRestore { slot: 1 }), Some(character_restore(1)));
        assert_eq!(encode(&NetCommand::GameGuardReply { keys: [1, 2, 3, 4] }), Some(game_guard_reply([1, 2, 3, 4])));
    }

    #[test]
    fn frente_p_commands_encode_the_window_packets() {
        let orders = vec![ItemOrder { object_id: 1001, item_id: 57, count: 1 }];
        assert_eq!(encode(&NetCommand::Trade { object_id: 7 }), Some(request_trade(7)));
        assert_eq!(
            encode(&NetCommand::AddTradeItem { trade_id: 1, object_id: 1001, count: 1 }),
            Some(request_add_trade_item(1, 1001, 1))
        );
        assert_eq!(encode(&NetCommand::TradeDone { confirm: true }), Some(request_trade_done(true)));
        assert_eq!(
            encode(&NetCommand::SellItems { list_id: 9, orders: orders.clone() }),
            Some(request_sell_item(9, &orders))
        );
        assert_eq!(
            encode(&NetCommand::BuyItems { list_id: 9, orders: vec![(57, 1)] }),
            Some(request_buy_item(9, &[(57, 1)]))
        );
        assert_eq!(encode(&NetCommand::JoinPledge { object_id: 7 }), Some(request_join_pledge(7)));
        assert_eq!(encode(&NetCommand::WithdrawPledge), Some(request_withdrawal_pledge()));
        assert_eq!(encode(&NetCommand::OustPledgeMember { name: "L".into() }), Some(request_oust_pledge_member("L")));
        assert_eq!(
            encode(&NetCommand::WarehouseDeposit { orders: vec![(1, 2)] }),
            Some(send_warehouse_deposit_list(&[(1, 2)]))
        );
        assert_eq!(
            encode(&NetCommand::WarehouseWithdraw { orders: vec![(1, 2)] }),
            Some(send_warehouse_withdraw_list(&[(1, 2)]))
        );
        assert_eq!(encode(&NetCommand::RequestPledgeMemberList), Some(request_pledge_member_list()));
        assert_eq!(encode(&NetCommand::SetPledgeCrest { data: vec![1] }), Some(request_set_pledge_crest(&[1])));
        assert_eq!(encode(&NetCommand::SetAllyCrest { data: vec![1] }), Some(request_set_ally_crest(&[1])));
        assert_eq!(encode(&NetCommand::EnchantItem { object_id: 1 }), Some(request_enchant_item(1)));
        assert_eq!(encode(&NetCommand::QuestAbort { quest_id: 1 }), Some(request_quest_abort(1)));
        assert_eq!(encode(&NetCommand::RequestPledgeCrest { crest_id: 5 }), Some(request_pledge_crest(5)));
        assert_eq!(encode(&NetCommand::RequestAllyCrest { crest_id: 6 }), Some(request_ally_crest(6)));
        assert_eq!(encode(&NetCommand::RestartPoint { point: 1 }), Some(request_restart_point(1)));
        assert_eq!(
            encode(&NetCommand::CrystallizeItem { object_id: 1, count: 1 }),
            Some(request_crystallize_item(1, 1))
        );
        assert_eq!(encode(&NetCommand::PrivateStoreManageSell), Some(request_private_store_manage_sell()));
        assert_eq!(encode(&NetCommand::PrivateStoreManageSellCancel), Some(request_private_store_manage_cancel_sell()));
        assert_eq!(encode(&NetCommand::PrivateStoreListSell { store_id: 7 }), Some(request_private_store_list_sell(7)));
        let offers = vec![StoreOffer { object_id: 1, count: 1, price: 1 }];
        assert_eq!(
            encode(&NetCommand::SetPrivateStoreListSell { package_sale: false, offers: offers.clone() }),
            Some(set_private_store_list_sell(false, &offers))
        );
        assert_eq!(encode(&NetCommand::PrivateStoreQuitSell), Some(request_private_store_quit_sell()));
        assert_eq!(
            encode(&NetCommand::SetPrivateStoreMsgSell { message: "m".into() }),
            Some(set_private_store_msg_sell("m"))
        );
        let store_orders = vec![StoreBuyOrder { object_id: 1001, count: 1, price: 5 }];
        assert_eq!(
            encode(&NetCommand::PrivateStoreBuy { store_id: 7, orders: store_orders.clone() }),
            Some(request_private_store_buy(7, &store_orders))
        );
        assert_eq!(encode(&NetCommand::TutorialLinkHtml { link: "l".into() }), Some(request_tutorial_link_html("l")));
        assert_eq!(encode(&NetCommand::TutorialPassCmd { command: "c".into() }), Some(request_tutorial_pass_cmd("c")));
        assert_eq!(encode(&NetCommand::TutorialQuestionMark { mark_id: 1 }), Some(request_tutorial_question_mark(1)));
        assert_eq!(encode(&NetCommand::TutorialClientEvent { event_id: 1 }), Some(request_tutorial_client_event(1)));
        assert_eq!(encode(&NetCommand::JoinAlly { object_id: 7 }), Some(request_join_ally(7)));
        assert_eq!(encode(&NetCommand::AllyLeave), Some(request_ally_leave()));
        assert_eq!(encode(&NetCommand::AllyDismiss { pledge_name: "C".into() }), Some(request_ally_dismiss("C")));
        assert_eq!(encode(&NetCommand::DismissAlly), Some(request_dismiss_ally()));
        assert_eq!(encode(&NetCommand::RequestAllyInfo), Some(request_ally_info()));
        assert_eq!(encode(&NetCommand::PetGetItem { object_id: 9 }), Some(request_pet_get_item(9)));
        assert_eq!(encode(&NetCommand::PrivateStoreManageBuy), Some(request_private_store_manage_buy()));
        assert_eq!(encode(&NetCommand::PrivateStoreManageBuyCancel), Some(request_private_store_manage_cancel_buy()));
        assert_eq!(encode(&NetCommand::PrivateStoreListBuy { store_id: 7 }), Some(request_private_store_list_buy(7)));
        let buy = vec![BuyOffer { item_id: 57, enchant: 0, custom_type2: 0, count: 1, price: 1 }];
        assert_eq!(
            encode(&NetCommand::SetPrivateStoreListBuy { offers: buy.clone() }),
            Some(set_private_store_list_buy(&buy))
        );
        assert_eq!(encode(&NetCommand::PrivateStoreQuitBuy), Some(request_private_store_quit_buy()));
        assert_eq!(
            encode(&NetCommand::SetPrivateStoreMsgBuy { message: "m".into() }),
            Some(set_private_store_msg_buy("m"))
        );
        let sell = vec![SellToStoreItem { object_id: 1, item_id: 57, enchant: 0, custom_type2: 0, count: 1, price: 1 }];
        assert_eq!(
            encode(&NetCommand::PrivateStoreSell { store_id: 7, items: sell.clone() }),
            Some(request_private_store_sell(7, &sell))
        );
        assert_eq!(
            encode(&NetCommand::PackageSendableItemList { target_id: 9 }),
            Some(request_package_sendable_item_list(9))
        );
        assert_eq!(
            encode(&NetCommand::PackageSend { target_id: 9, orders: vec![(1, 1)] }),
            Some(request_package_send(9, &[(1, 1)]))
        );
        assert_eq!(
            encode(&NetCommand::MultiSellChoose { list_id: 1, entry_id: 2, amount: 3 }),
            Some(multi_sell_choose(1, 2, 3))
        );
        assert_eq!(encode(&NetCommand::UserCommand { command: 3 }), Some(request_user_command(3)));
        assert_eq!(encode(&NetCommand::HennaList), Some(request_henna_list(0)));
        assert_eq!(encode(&NetCommand::HennaItemInfo { symbol_id: 1 }), Some(request_henna_item_info(1)));
        assert_eq!(encode(&NetCommand::HennaEquip { symbol_id: 1 }), Some(request_henna_equip(1)));
        assert_eq!(encode(&NetCommand::HennaUnequipList), Some(request_henna_unequip_list(0)));
        assert_eq!(encode(&NetCommand::HennaUnequipInfo { symbol_id: 1 }), Some(request_henna_unequip_info(1)));
        assert_eq!(encode(&NetCommand::HennaUnequip { symbol_id: 1 }), Some(request_henna_unequip(1)));
        assert_eq!(encode(&NetCommand::PledgePower { member_id: 7, action: 3, powers: [9; 32] }), Some(request_pledge_power(7, 3, &[9; 32])));
        let m = Macro {
            id: 1,
            name: "n".into(),
            description: String::new(),
            acronym: "a".into(),
            icon: 0,
            commands: Vec::new(),
        };
        assert_eq!(encode(&NetCommand::MakeMacro(Box::new(m.clone()))), Some(request_make_macro(&m)));
        assert_eq!(encode(&NetCommand::DeleteMacro { id: 1 }), Some(request_delete_macro(1)));
        assert_eq!(
            encode(&NetCommand::ProcureCrop { manor_id: 1, orders: orders.clone() }),
            Some(request_procure_crop(1, &orders))
        );
        assert_eq!(
            encode(&NetCommand::PreviewItem { header: 9, list_id: 3, items: vec![2] }),
            Some(request_preview_item(9, 3, &[2]))
        );
        assert_eq!(
            encode(&NetCommand::SendFriendMsg { message: "m".into(), receiver: "L".into() }),
            Some(request_send_friend_msg("m", "L"))
        );
        assert_eq!(encode(&NetCommand::ShowMinimap), Some(request_show_minimap()));
        assert_eq!(encode(&NetCommand::FriendInvite { name: "L".into() }), Some(request_friend_invite("L")));
        assert_eq!(encode(&NetCommand::AnswerFriendInvite { accept: true }), Some(request_answer_friend_invite(true)));
        assert_eq!(encode(&NetCommand::FriendList), Some(request_friend_list()));
        assert_eq!(encode(&NetCommand::FriendDel { name: "L".into() }), Some(request_friend_del("L")));
        assert_eq!(
            encode(&NetCommand::GiveNickName { target: "L".into(), title: "T".into() }),
            Some(request_give_nick_name("L", "T"))
        );
        assert_eq!(encode(&NetCommand::ShowBoard), Some(request_show_board(0)));
        assert_eq!(
            encode(&NetCommand::BbsWrite {
                fields: Box::new([
                    "a".into(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new()
                ])
            }),
            Some(request_bbs_write(&["a", "", "", "", "", ""]))
        );
        assert_eq!(
            encode(&NetCommand::PartyMatchList { level: 40, location: 0, mode: 1 }),
            Some(request_party_match_list(40, 0, 1))
        );
        assert_eq!(
            encode(&NetCommand::ManagePartyRoom {
                room_id: 1,
                max_members: 9,
                min_level: 1,
                max_level: 80,
                loot_type: 0,
                title: "S".into()
            }),
            Some(request_manage_party_room(1, 9, 1, 80, 0, "S"))
        );
        assert_eq!(encode(&NetCommand::JoinPartyRoom { room_id: 1 }), Some(request_join_party_room(1, 0)));
    }

    #[test]
    fn frente_p_commands_encode_the_rare_packets() {
        assert_eq!(encode(&NetCommand::RequestSiegeInfo), Some(request_siege_info()));
        assert_eq!(
            encode(&NetCommand::StartPledgeWar { pledge_name: "C".into() }),
            Some(request_start_pledge_war("C"))
        );
        assert_eq!(encode(&NetCommand::StopPledgeWar { pledge_name: "C".into() }), Some(request_stop_pledge_war("C")));
        assert_eq!(encode(&NetCommand::Petition { content: "p".into(), kind: 1 }), Some(request_petition("p", 1)));
        assert_eq!(encode(&NetCommand::PetitionCancel { content: "p".into() }), Some(request_petition_cancel("p")));
        assert_eq!(encode(&NetCommand::PetitionVote { first: 1, index: 2, text: "t".into() }), Some(request_petition_vote(1, 2, "t")));
        assert_eq!(encode(&NetCommand::ObserverEnd), Some(request_observer_end()));
        assert_eq!(encode(&NetCommand::GmCommand { name: "a".into(), kind: 5 }), Some(request_gm_command("a", 5)));
        assert_eq!(encode(&NetCommand::GmSnoopEnd { snoop_id: 7 }), Some(request_gm_snoop_end(7)));
        assert_eq!(encode(&NetCommand::OlympiadObserverEnd), Some(request_olympiad_observer_end()));
        assert_eq!(encode(&NetCommand::OlympiadMatchList), Some(request_olympiad_match_list()));
        assert_eq!(encode(&NetCommand::BypassBuildCmd { command: "//a".into() }), Some(send_bypass_build_cmd("//a")));
        assert_eq!(encode(&NetCommand::CastleSiegeInfo { castle_id: 3 }), Some(request_castle_siege_info(3)));
        assert_eq!(
            encode(&NetCommand::CastleSiegeAttackerList { castle_id: 3 }),
            Some(request_castle_siege_attacker_list(3))
        );
        assert_eq!(
            encode(&NetCommand::CastleSiegeDefenderList { castle_id: 3 }),
            Some(request_castle_siege_defender_list(3))
        );
        assert_eq!(
            encode(&NetCommand::JoinCastleSiege { castle_id: 3, attacker: true, join: true }),
            Some(request_join_castle_siege(3, true, true))
        );
        assert_eq!(
            encode(&NetCommand::ConfirmCastleSiegeWaitingList { castle_id: 3, pledge_id: 33, approved: true }),
            Some(request_confirm_castle_siege_waiting_list(3, 33, true))
        );
        assert_eq!(encode(&NetCommand::RecipeBookOpen { common: true }), Some(request_recipe_book_open(true)));
        assert_eq!(encode(&NetCommand::RecipeItemDelete { recipe_id: 10 }), Some(request_recipe_item_delete(10)));
        assert_eq!(encode(&NetCommand::RecipeItemMakeInfo { recipe_id: 10 }), Some(request_recipe_item_make_info(10)));
        assert_eq!(encode(&NetCommand::RecipeItemMakeSelf { recipe_id: 10 }), Some(request_recipe_item_make_self(10)));
        assert_eq!(encode(&NetCommand::RecipeShopManageList), Some(request_recipe_shop_manage_list()));
        assert_eq!(
            encode(&NetCommand::RecipeShopMessageSet { message: "m".into() }),
            Some(request_recipe_shop_message_set("m"))
        );
        assert_eq!(
            encode(&NetCommand::RecipeShopListSet { items: vec![(10, 300)] }),
            Some(request_recipe_shop_list_set(&[(10, 300)]))
        );
        assert_eq!(encode(&NetCommand::RecipeShopManageQuit), Some(request_recipe_shop_manage_quit()));
        assert_eq!(encode(&NetCommand::RecipeShopManageCancel), Some(request_recipe_shop_manage_cancel()));
        assert_eq!(
            encode(&NetCommand::RecipeShopMakeInfo { manufacturer_id: 7, recipe_id: 10 }),
            Some(request_recipe_shop_make_info(7, 10))
        );
        assert_eq!(
            encode(&NetCommand::RecipeShopMakeDo { manufacturer_id: 7, recipe_id: 10, cost: 300 }),
            Some(request_recipe_shop_make_do(7, 10, 300))
        );
        assert_eq!(
            encode(&NetCommand::RecipeShopSellList { manufacturer_id: 7 }),
            Some(request_recipe_shop_sell_list(7))
        );
        assert_eq!(encode(&NetCommand::VoteSociality { target_id: 7 }), Some(vote_sociality(7)));
        assert_eq!(
            encode(&NetCommand::BuySeed { manor_id: 1, orders: vec![(5016, 10)] }),
            Some(request_buy_seed(1, &[(5016, 10)]))
        );
        assert_eq!(encode(&NetCommand::SsqStatus { page: 2 }), Some(request_ssq_status(2)));
        assert_eq!(encode(&NetCommand::OustFromPartyRoom { object_id: 7 }), Some(request_oust_from_party_room(7)));
        assert_eq!(encode(&NetCommand::DismissPartyRoom { room_id: 1 }), Some(request_dismiss_party_room(1, 0)));
        assert_eq!(encode(&NetCommand::WithdrawPartyRoom { room_id: 1 }), Some(request_withdraw_party_room(1, 0)));
        assert_eq!(
            encode(&NetCommand::ExEnchantSkillInfo { skill_id: 3, level: 101 }),
            Some(request_ex_enchant_skill_info(3, 101))
        );
        assert_eq!(
            encode(&NetCommand::ExEnchantSkill { skill_id: 3, level: 101 }),
            Some(request_ex_enchant_skill(3, 101))
        );
        assert_eq!(encode(&NetCommand::ManorList), Some(request_manor_list()));
        let procure = vec![CropProcure { object_id: 1, crop_id: 2, manor_id: 3, count: 4 }];
        assert_eq!(
            encode(&NetCommand::ProcureCropList { orders: procure.clone() }),
            Some(request_procure_crop_list(&procure))
        );
        assert_eq!(
            encode(&NetCommand::SetSeed { manor_id: 1, seeds: vec![(5016, 10, 5)] }),
            Some(request_set_seed(1, &[(5016, 10, 5)]))
        );
        let crops = vec![CropSetting { crop_id: 2, amount: 3, price: 4, reward: 1 }];
        assert_eq!(
            encode(&NetCommand::SetCrop { manor_id: 1, crops: crops.clone() }),
            Some(request_set_crop(1, &crops))
        );
        assert_eq!(encode(&NetCommand::WriteHeroWords { words: "w".into() }), Some(request_write_hero_words("w")));
        assert_eq!(encode(&NetCommand::ExAskJoinMpcc { name: "L".into() }), Some(request_ex_ask_join_mpcc("L")));
        assert_eq!(encode(&NetCommand::ExAcceptJoinMpcc { accept: true }), Some(request_ex_accept_join_mpcc(true)));
        assert_eq!(encode(&NetCommand::ExOustFromMpcc { name: "L".into() }), Some(request_ex_oust_from_mpcc("L")));
        assert_eq!(encode(&NetCommand::ExPledgeCrestLarge { crest_id: 5 }), Some(request_ex_pledge_crest_large(5)));
        assert_eq!(
            encode(&NetCommand::ExSetPledgeCrestLarge { data: vec![9] }),
            Some(request_ex_set_pledge_crest_large(&[9]))
        );
    }

    #[test]
    fn replay_source_feeds_session_events_and_ends_closed() {
        use crate::replay::{ReplayFile, ReplayHeader, ReplayPacket};
        let file = ReplayFile {
            marker: 1,
            header: ReplayHeader { start: [1.0, 2.0, 3.0], running_time: 0.2, ..ReplayHeader::default() },
            packets: vec![
                ReplayPacket::from_body(0.05, &[0x62, 1, 2, 3]).unwrap(),
                ReplayPacket::from_body(0.1, &[0x63, 4, 5]).unwrap(),
            ],
            cameras: Vec::new(),
            user_ids: Vec::new(),
        };
        let handle = spawn_replay(&file.to_bytes()).unwrap();
        let link = handle.replay.as_ref().unwrap();
        link.control.send(ReplayCommand::Ready(true)).unwrap();
        let mut sizes = Vec::new();
        loop {
            match handle.events.recv_timeout(Duration::from_secs(10)).unwrap() {
                NetEvent::Session(SessionEvent::Malformed { opcode, size }) | NetEvent::Session(SessionEvent::Unknown { opcode, size }) => sizes.push((opcode, size)),
                NetEvent::Session(_) => sizes.push((0, 0)),
                NetEvent::Closed => break,
                other => panic!("evento inesperado {other:?}"),
            }
        }
        assert_eq!(sizes.len(), 2);
        let mut played = Vec::new();
        while let Ok(event) = link.events.try_recv() {
            played.push(event);
        }
        assert!(matches!(played.first(), Some(PlayEvent::LoadMap(_))));
        assert!(matches!(played.last(), Some(PlayEvent::Ended)));
        assert!(played.iter().any(|event| matches!(event, PlayEvent::Teleport(_))));
    }
}
