use l2_net::game::packets::{
    OP_ABNORMAL_VISUAL_EFFECT, OP_ACQUIRE_SKILL_DONE, OP_ACQUIRE_SKILL_INFO, OP_ACQUIRE_SKILL_LIST, OP_ALLIANCE_CREST, OP_ALLIANCE_INFO, OP_ATTACK_DEAD_TARGET, OP_ATTACK_IN_COOL_TIME, OP_ATTACK_OUT_OF_RANGE, OP_BUY_LIST, OP_CREATE_PLEDGE, OP_DISMISS_ALLIANCE, OP_DISMISS_PARTY, OP_DISMISS_PLEDGE,
    OP_BLOCK_LIST, OP_BUY_LIST_SEED, OP_CHOOSE_INVENTORY_ITEM, OP_CLIENT_ACTION, OP_ENCHANT_RESULT, OP_FRIEND_ADD_REQUEST,
    OP_FRIEND_LIST,
    OP_FRIEND_RECV_MSG, OP_GM_VIEW_ITEM_LIST, OP_GM_VIEW_MAGIC_INFO, OP_GM_VIEW_QUEST_INFO, OP_GM_VIEW_SKILL_INFO,
    OP_GM_VIEW_CHARACTER_INFO, OP_GM_VIEW_PLEDGE_INFO, OP_GM_VIEW_WAREHOUSE_WITHDRAW_LIST,
    OP_HENNA_EQUIP_LIST,
    OP_HENNA_INFO,
    OP_HENNA_ITEM_INFO, OP_HENNA_UNEQUIP_INFO,
    OP_HENNA_UNEQUIP_LIST, OP_JOIN_PARTY, OP_JOIN_PLEDGE, OP_L2_FRIEND, OP_L2_FRIEND_STATUS,
    OP_MANAGE_PLEDGE_POWER, OP_MULTI_SELL_LIST, OP_PACKAGE_SENDABLE_LIST, OP_PACKAGE_TO_LIST, OP_PARTY_MATCH_DETAIL,
    OP_PARTY_MATCH_LIST, OP_PET_INVENTORY_UPDATE, OP_PET_ITEM_LIST, OP_PLEDGE_CREST, OP_PLEDGE_INFO,
    OP_PLEDGE_EXTENDED_INFO, OP_PLEDGE_SHOW_INFO_UPDATE, OP_PLEDGE_SHOW_MEMBER_LIST_ADD, OP_PLEDGE_SHOW_MEMBER_LIST_ALL,
    OP_PLEDGE_SHOW_MEMBER_LIST_DELETE, OP_PLEDGE_SHOW_MEMBER_LIST_DELETE_ALL, OP_PLEDGE_SHOW_MEMBER_LIST_UPDATE,
    OP_DELETE_RADAR, OP_DUMMY, OP_FRIEND_ADD, OP_FRIEND_ADD_REQUEST_RESULT, OP_FRIEND_LIST_LEGACY, OP_FRIEND_REMOVE, OP_FRIEND_STATUS, OP_GM_HIDE, OP_JOIN_ALLIANCE, OP_MAGIC_LIST, OP_NEW_CHARACTER_FAIL, OP_OUST_ALLIANCE_MEMBER_PLEDGE, OP_OUST_PARTY_MEMBER, OP_OUST_PLEDGE_MEMBER, OP_PRIVATE_STORE_LIST_BUY, OP_PRIVATE_STORE_LIST_SELL, OP_PRIVATE_STORE_MANAGE_LIST_BUY, OP_REPLY_SURRENDER_PLEDGE_WAR, OP_REPLY_START_PLEDGE_WAR, OP_REPLY_STOP_PLEDGE_WAR, OP_REVIVE_REQUEST,
    OP_CAMERA_MODE, OP_EVENT_TRIGGER, OP_MAGIC_AND_SKILL_LIST, OP_NET_PING_RESPONSE, OP_NORMAL_CAMERA, OP_OBSERVER_END, OP_OBSERVER_START, OP_PETITION_VOTE, OP_PRIVATE_STORE_MANAGE_LIST_SELL, OP_PRIVATE_STORE_MSG_BUY, OP_PRIVATE_STORE_MSG_SELL, OP_QUEST_LIST, OP_REPLY_START_ALLIANCE_WAR, OP_REPLY_STOP_ALLIANCE_WAR, OP_REQUEST_TIME_CHECK, OP_SERVER_CLOSE_SOCKET, OP_SET_PLEDGE_CREST, OP_SNOOP, OP_START_ALLIANCE_WAR, OP_START_PLEDGE_WAR, OP_STOP_ALLIANCE_WAR, OP_STOP_PLEDGE_WAR, OP_SURRENDER_ALLIANCE_WAR, OP_SURRENDER_PERSONALLY, OP_SURRENDER_PLEDGE_WAR,
    OP_RADAR_CONTROL, OP_SELL_LIST, OP_SHOP_PREVIEW_INFO, OP_SHOP_PREVIEW_LIST, OP_SHOW_BOARD, OP_SHOW_MINIMAP,
    OP_GM_HENNA_INFO, OP_SELL_LIST_PROCURE, OP_SHOW_RADAR, OP_SKILL_REMAIN_SEC,
    OP_SHOW_TOWN_MAP, OP_TRADE_DONE, OP_TRADE_OTHER_ADD, OP_TRADE_OWN_ADD, OP_TRADE_PRESS_OTHER_OK,
    OP_SET_ALLIANCE_CREST, OP_TRADE_PRESS_OWN_OK, OP_TRADE_REQUEST, OP_TRADE_START, OP_TRADE_START_OK, OP_VALIDATE_LOCATION_IN_VEHICLE,
    OP_SET_DISMISS_PARTY, OP_SET_DISMISS_PLEDGE, OP_SET_OUST_PARTY_MEMBER, OP_SET_OUST_PLEDGE_MEMBER, OP_TRADE_UPDATE, OP_TUTORIAL_CLOSE_HTML, OP_TUTORIAL_ENABLE_CLIENT_EVENT, OP_TUTORIAL_SHOW_HTML, OP_VEHICLE_CHECK_LOCATION, OP_WITHDRAW_ALLIANCE, OP_WITHDRAWAL_PARTY, OP_WITHDRAWAL_PLEDGE,
    OP_TUTORIAL_SHOW_QUESTION_MARK, OP_WAREHOUSE_DEPOSIT_LIST, OP_WAREHOUSE_DONE, OP_WAREHOUSE_WITHDRAW_LIST,
    read_acquire_skill_done,
    read_acquire_skill_info, read_acquire_skill_list, read_alliance_crest, read_buy_list, read_buy_list_seed,
    read_choose_inventory_item, read_client_action, read_enchant_result, read_friend_add_request, read_friend_list, read_friend_recv_msg,
    read_gm_view_item_list, read_gm_view_magic_info, read_gm_view_quest_info, read_gm_view_skill_info,
    read_gm_view_character_info, read_gm_view_pledge_info, read_gm_view_warehouse_withdraw_list,
    read_henna_equip_list, read_henna_info,
    read_henna_item_info,
    read_henna_unequip_info, read_henna_unequip_list, read_l2_friend, read_l2_friend_status,
    read_join_party, read_join_pledge,
    read_manage_pledge_power, read_multi_sell_list, read_package_sendable_list, read_package_to_list,
    read_party_match_detail, read_party_match_list, read_pet_inventory_update, read_pet_item_list, read_pledge_crest,
    read_pledge_extended_info, read_pledge_info, read_pledge_show_info_update, read_pledge_show_member_list_add, read_pledge_show_member_list_all,
    read_pledge_show_member_list_delete, read_pledge_show_member_list_delete_all, read_pledge_show_member_list_update,
    read_private_store_list_buy, read_private_store_list_sell, read_private_store_manage_list_buy,
    read_delete_radar, read_friend_add, read_friend_add_request_result, read_friend_list_noop, read_friend_remove, read_friend_status_noop, read_magic_list, read_new_character_fail, read_private_store_manage_list_sell, read_private_store_msg_buy, read_private_store_msg_sell, read_reply_surrender_pledge_war, read_reply_start_pledge_war, read_reply_stop_pledge_war,
    read_abnormal_visual_effect, read_alliance_info, read_attack_dead_target, read_attack_in_cool_time, read_attack_out_of_range, read_block_list, read_create_pledge, read_dismiss_alliance, read_dismiss_party, read_dismiss_pledge, read_dummy, read_gm_hide, read_join_alliance, read_magic_and_skill_list, read_oust_alliance_member_pledge, read_oust_party_member, read_oust_pledge_member, read_quest_list, read_radar_control, read_reply_start_alliance_war, read_reply_stop_alliance_war, read_restart_response, read_sell_list, read_server_close_socket, read_set_alliance_crest, read_set_dismiss_party, read_set_dismiss_pledge, read_set_oust_party_member, read_set_oust_pledge_member, read_set_pledge_crest, read_shop_preview_info, read_shop_preview_list, read_show_board,
    read_gm_henna_info, read_sell_list_procure, read_show_radar, read_skill_remain_sec,
    read_show_minimap, read_show_town_map, read_trade_done, read_trade_other_add, read_trade_own_add,
    read_camera_mode, read_event_trigger, read_net_ping_response, read_normal_camera, read_observer_end, read_observer_start, read_petition_vote, read_request_time_check, read_snoop, read_start_alliance_war, read_start_pledge_war, read_stop_alliance_war, read_stop_pledge_war, read_surrender_alliance_war, read_surrender_personally, read_surrender_pledge_war, read_trade_press_other_ok, read_trade_press_own_ok, read_trade_request, read_trade_start, read_trade_start_ok, read_revive_request,
    read_trade_update, read_tutorial_close_html,
    read_tutorial_enable_client_event, read_tutorial_show_html, read_tutorial_show_question_mark,
    read_validate_location_in_vehicle, read_vehicle_check_location, read_warehouse_deposit_list, read_warehouse_done, read_warehouse_withdraw_list, read_withdraw_alliance, read_withdrawal_party, read_withdrawal_pledge,
};
use l2_net::game::packets::{
    OP_ACTION_FAILED, OP_AUTH_LOGIN_FAIL, OP_CHANGE_MOVE_TYPE, OP_CHANGE_WAIT_TYPE, OP_CHAR_INFO, OP_CHAR_SELECT_INFO,
    OP_CHAR_SELECTED, OP_CLIENT_SET_TIME, OP_DELETE_OBJECT, OP_DIE, OP_EQUIP_UPDATE, OP_ETC_STATUS_UPDATE,
    OP_INVENTORY_UPDATE, OP_ITEM_LIST, OP_KEY_PACKET, OP_MAGIC_EFFECT_ICONS, OP_MAGIC_SKILL_CANCELED,
    OP_MAGIC_SKILL_LAUNCHED, OP_MAGIC_SKILL_USE, OP_MOVE_TO_LOCATION, OP_MOVE_TO_PAWN, OP_MY_TARGET_SELECTED,
    OP_NPC_HTML_MESSAGE, OP_NPC_INFO, OP_NPC_SAY, OP_PLEDGE_STATUS_CHANGED, OP_RELATION_CHANGED, OP_RESTART_RESPONSE,
    OP_REVIVE, OP_SAY2, OP_SEND_MACRO_LIST, OP_SHORT_CUT_DELETE, OP_SHORT_CUT_INIT, OP_SHORT_CUT_REGISTER,
    OP_SKILL_COOL_TIME, OP_SKILL_LIST, OP_SOCIAL_ACTION, OP_SPAWN_ITEM, OP_STATUS_UPDATE, OP_STOP_MOVE,
    OP_SYSTEM_MESSAGE, OP_TARGET_SELECTED, OP_TARGET_UNSELECTED, OP_TELEPORT_TO_LOCATION, OP_USER_INFO,
    OP_VALIDATE_LOCATION, auth_login, character_selected, character_selected_transfer, enter_world, request_skill_list,
};
use l2_net::game::packets::request_manor_list;
use l2_net::game::packets::{
    OP_AGIT_DECO_INFO, OP_CASTLE_SIEGE_ATTACKER_LIST, OP_CASTLE_SIEGE_DEFENDER_LIST, OP_CASTLE_SIEGE_INFO,
    OP_MON_RACE_INFO, OP_RECIPE_BOOK_ITEM_LIST, OP_RECIPE_ITEM_MAKE_INFO, OP_RECIPE_SHOP_ITEM_INFO,
    OP_RECIPE_SHOP_MANAGE_LIST, OP_RECIPE_SHOP_MSG, OP_RECIPE_SHOP_SELL_LIST, OP_SHOW_CALCULATOR, OP_SHOW_XMAS_SEAL,
    OP_SSQ_INFO, OP_SSQ_STATUS, read_agit_deco_info, read_castle_siege_attacker_list, read_castle_siege_defender_list,
    read_castle_siege_info, read_mon_race_info, read_recipe_book_item_list, read_recipe_item_make_info,
    read_recipe_shop_item_info, read_recipe_shop_manage_list, read_recipe_shop_msg, read_recipe_shop_sell_list,
    read_show_calculator, read_show_xmas_seal, read_ssq_info, read_ssq_status,
};
use l2_net::game::packets::{
    OP_ASK_JOIN_ALLIANCE, OP_ASK_JOIN_PARTY, OP_ASK_JOIN_PLEDGE, OP_ATTACK, OP_AUTO_ATTACK_START, OP_AUTO_ATTACK_STOP,
    OP_CHAIR_SIT, OP_CHAR_CREATE_FAIL, OP_CHAR_CREATE_OK, OP_CHAR_DELETE_FAIL, OP_CHAR_DELETE_OK, OP_CHAR_TEMPLATES,
    OP_CONFIRM_DLG, OP_DICE, OP_DOOR_INFO, OP_DOOR_STATUS_UPDATE, OP_DROP_ITEM, OP_EARTHQUAKE, OP_FINISH_ROTATING,
    OP_FLY_TO_LOCATION, OP_GAME_GUARD_QUERY, OP_GET_ITEM, OP_GET_OFF_VEHICLE, OP_GET_ON_VEHICLE, OP_LEAVE_WORLD,
    OP_MOVE_TO_LOCATION_IN_VEHICLE, OP_NICKNAME_CHANGED, OP_PARTY_MEMBER_POSITION, OP_PARTY_SMALL_WINDOW_ADD,
    OP_PARTY_SMALL_WINDOW_ALL, OP_PARTY_SMALL_WINDOW_DELETE, OP_PARTY_SMALL_WINDOW_DELETE_ALL,
    OP_PARTY_SMALL_WINDOW_UPDATE, OP_PARTY_SPELLED, OP_PET_DELETE, OP_PET_INFO, OP_PET_STATUS_SHOW,
    OP_PET_STATUS_UPDATE, OP_PLAY_SOUND, OP_RIDE, OP_SERVER_CLOSE, OP_SERVER_OBJECT_INFO, OP_SET_SUMMON_REMAIN_TIME,
    OP_SETUP_GAUGE, OP_SHORT_BUFF_STATUS_UPDATE, OP_SPECIAL_CAMERA, OP_START_ROTATING, OP_STATIC_OBJECT,
    OP_STOP_MOVE_IN_VEHICLE, OP_SUN_RISE, OP_SUN_SET, OP_VEHICLE_DEPARTURE, OP_VEHICLE_INFO, OP_VEHICLE_START,
    game_guard_reply, read_ask_join_alliance, read_ask_join_party, read_ask_join_pledge, read_attack,
    read_auto_attack_start, read_auto_attack_stop, read_chair_sit, read_char_create_fail, read_char_create_ok,
    read_char_delete_fail, read_char_delete_ok, read_char_templates, read_confirm_dlg, read_dice, read_door_info,
    read_door_status_update, read_drop_item, read_earthquake, read_finish_rotating, read_fly_to_location,
    read_game_guard_query, read_get_item, read_get_off_vehicle, read_get_on_vehicle, read_leave_world,
    read_move_to_location_in_vehicle, read_nickname_changed, read_party_member_position, read_party_small_window_add,
    read_party_small_window_all, read_party_small_window_delete, read_party_small_window_delete_all,
    read_party_small_window_update, read_party_spelled, read_pet_delete, read_pet_info, read_pet_status_show,
    read_pet_status_update, read_play_sound, read_ride, read_server_close, read_server_object_info,
    read_set_summon_remain_time, read_setup_gauge, read_short_buff_status_update, read_special_camera,
    read_start_rotating, read_static_object, read_stop_move_in_vehicle, read_sun_rise, read_sun_set,
    read_vehicle_departure, read_vehicle_info, read_vehicle_start,
};
use l2_net::game::{
    CharSelectEntry, CharTemplates, ClientSetTime, GameCrypt, UserInfoHead, read_action_failed, read_change_move_type,
    read_change_wait_type, read_char_info, read_char_select_info, read_client_set_time, read_delete_object, read_die,
    read_equip_update, read_etc_status_update, read_inventory_update, read_item_list, read_magic_effect_icons,
    read_magic_skill_canceled, read_magic_skill_launched, read_magic_skill_use, read_move_to_location,
    read_move_to_pawn, read_my_target_selected, read_npc_html_message, read_npc_info, read_npc_say,
    read_pledge_status_changed, read_relation_changed, read_revive, read_say2, read_send_macro_list,
    read_short_cut_delete, read_short_cut_init, read_short_cut_register, read_skill_cool_time, read_skill_list,
    read_social_action, read_spawn_item, read_status_update, read_stop_move, read_system_message, read_target_selected,
    read_target_unselected, read_teleport_to_location, read_user_info_head, read_validate_location,
};
use l2_net::login::Authorization;
use l2_net::wire;

use crate::world::WorldEvent;

#[derive(Debug)]
pub enum SessionEvent {
    Characters(Vec<CharSelectEntry>),
    Selected,
    UserInfo(Box<UserInfoHead>),
    World(WorldEvent),
    Time(ClientSetTime),
    Refused(String),
    CharTemplates(CharTemplates),
    CharCreated,
    CharCreateFailed(i32),
    CharDeleted,
    CharDeleteFailed(i32),
    RestartResponse(l2_net::game::packets::RestartResponse),
    ServerClosed,
    LeftWorld,
    GameGuardQuery([i32; 4]),
    Unknown { opcode: u8, size: usize },
    Malformed { opcode: u8, size: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    AwaitingKey,
    AwaitingCharacters,
    Selecting,
    Entering,
    InWorld,
}

pub const PROTOCOL_MISMATCH: &str = "versão do protocolo diferente";

pub struct Session {
    account: String,
    auth: Option<Authorization>,
    state: SessionState,
}

impl Session {
    pub fn new(account: String, auth: Authorization) -> Session {
        Session { account, auth: Some(auth), state: SessionState::AwaitingKey }
    }

    pub fn replaying() -> Session {
        Session { account: String::new(), auth: None, state: SessionState::InWorld }
    }

    pub fn on_body(&mut self, body: &[u8]) -> (Option<SessionEvent>, Vec<Vec<u8>>, Option<GameCrypt>) {
        let opcode = match body.first() {
            Some(op) => *op,
            None => return (Some(SessionEvent::Unknown { opcode: 0, size: 0 }), Vec::new(), None),
        };
        match opcode {
            OP_KEY_PACKET => match GameCrypt::from_key_packet(body) {
                Ok(crypt) => match &self.auth {
                    Some(auth) => {
                        self.state = SessionState::AwaitingCharacters;
                        (None, vec![auth_login(&self.account, auth)], Some(crypt))
                    }
                    None => (Some(SessionEvent::Unknown { opcode, size: body.len() }), Vec::new(), None),
                },
                Err(_) if body.len() == 14 && body[1] == 0 && body[6..10] == [1, 0, 0, 0] => {
                    (Some(SessionEvent::Refused(PROTOCOL_MISMATCH.into())), vec![l2_net::game::packets::logout()], None)
                }
                Err(_) => (Some(SessionEvent::Refused(format!("pacote {opcode:#04x} malformado"))), Vec::new(), None),
            },
            OP_CHAR_SELECT_INFO => match read_char_select_info(body) {
                Ok(characters) => {
                    self.state = SessionState::Selecting;
                    (Some(SessionEvent::Characters(characters)), Vec::new(), None)
                }
                Err(_) => (Some(SessionEvent::Malformed { opcode, size: body.len() }), Vec::new(), None),
            },
            OP_AUTH_LOGIN_FAIL => {
                (Some(SessionEvent::Refused("game server recusou o AuthLogin".into())), Vec::new(), None)
            }
            OP_CHAR_SELECTED => (Some(SessionEvent::Selected), vec![enter_world()], None),
            OP_USER_INFO => match read_user_info_head(body) {
                Ok(head) => {
                    let out = if self.state == SessionState::Entering { vec![request_skill_list(), request_manor_list()] } else { Vec::new() };
                    self.state = SessionState::InWorld;
                    (Some(SessionEvent::UserInfo(Box::new(head))), out, None)
                }
                Err(_) => (Some(SessionEvent::Malformed { opcode, size: body.len() }), Vec::new(), None),
            },
            OP_RESTART_RESPONSE => match read_restart_response(body) {
                Ok(response) => {
                    self.state = SessionState::AwaitingCharacters;
                    (Some(SessionEvent::RestartResponse(response)), Vec::new(), None)
                }
                Err(_) => (Some(SessionEvent::Malformed { opcode, size: body.len() }), Vec::new(), None),
            }
            OP_CLIENT_SET_TIME => match read_client_set_time(body) {
                Ok(time) => (Some(SessionEvent::Time(time)), Vec::new(), None),
                Err(_) => (Some(SessionEvent::Malformed { opcode, size: body.len() }), Vec::new(), None),
            },
            OP_CHAR_TEMPLATES => match read_char_templates(body) {
                Ok(templates) => (Some(SessionEvent::CharTemplates(templates)), Vec::new(), None),
                Err(_) => (Some(SessionEvent::Malformed { opcode, size: body.len() }), Vec::new(), None),
            },
            OP_CHAR_CREATE_OK => match read_char_create_ok(body) {
                Ok(_) => (Some(SessionEvent::CharCreated), Vec::new(), None),
                Err(_) => (Some(SessionEvent::Malformed { opcode, size: body.len() }), Vec::new(), None),
            },
            OP_CHAR_CREATE_FAIL => match read_char_create_fail(body) {
                Ok(fail) => (Some(SessionEvent::CharCreateFailed(fail.reason)), Vec::new(), None),
                Err(_) => (Some(SessionEvent::Malformed { opcode, size: body.len() }), Vec::new(), None),
            },
            OP_CHAR_DELETE_OK => match read_char_delete_ok(body) {
                Ok(_) => (Some(SessionEvent::CharDeleted), Vec::new(), None),
                Err(_) => (Some(SessionEvent::Malformed { opcode, size: body.len() }), Vec::new(), None),
            },
            OP_CHAR_DELETE_FAIL => match read_char_delete_fail(body) {
                Ok(fail) => (Some(SessionEvent::CharDeleteFailed(fail.reason)), Vec::new(), None),
                Err(_) => (Some(SessionEvent::Malformed { opcode, size: body.len() }), Vec::new(), None),
            },
            OP_SERVER_CLOSE => match read_server_close(body) {
                Ok(_) => (Some(SessionEvent::ServerClosed), Vec::new(), None),
                Err(_) => (Some(SessionEvent::Malformed { opcode, size: body.len() }), Vec::new(), None),
            },
            OP_LEAVE_WORLD => match read_leave_world(body) {
                Ok(_) => {
                    self.state = SessionState::AwaitingCharacters;
                    (Some(SessionEvent::LeftWorld), Vec::new(), None)
                }
                Err(_) => (Some(SessionEvent::Malformed { opcode, size: body.len() }), Vec::new(), None),
            },
            OP_GAME_GUARD_QUERY => match read_game_guard_query(body) {
                Ok(query) => (Some(SessionEvent::GameGuardQuery(query.keys)), vec![game_guard_reply(query.keys)], None),
                Err(_) => (Some(SessionEvent::Malformed { opcode, size: body.len() }), Vec::new(), None),
            },
            _ if matches!(self.state, SessionState::Entering | SessionState::InWorld) => {
                let event = match decode_world(body) {
                    Some(Ok(event)) => SessionEvent::World(event),
                    Some(Err(why)) => {
                        log::warn!("pacote {opcode:#04x} ({} bytes) malformado: {why}; início {:02x?}", body.len(), &body[..body.len().min(64)]);
                        SessionEvent::Malformed { opcode, size: body.len() }
                    }
                    None => {
                        log::info!("opcode {opcode:#04x} sem leitor ({} bytes); início {:02x?}", body.len(), &body[..body.len().min(48)]);
                        SessionEvent::Unknown { opcode, size: body.len() }
                    }
                };
                (Some(event), Vec::new(), None)
            }
            _ => (Some(SessionEvent::Unknown { opcode, size: body.len() }), Vec::new(), None),
        }
    }

    pub fn select(&mut self, slot: i32) -> Vec<u8> {
        self.state = SessionState::Entering;
        character_selected(slot)
    }

    pub fn select_transfer(&mut self, slot: i32, position: [i32; 3]) -> Vec<u8> {
        self.state = SessionState::Entering;
        character_selected_transfer(slot, true, position)
    }

    pub fn state(&self) -> SessionState {
        self.state
    }
}

pub fn decode_world(body: &[u8]) -> Option<wire::Result<WorldEvent>> {
    use l2_net::game::EX_COLOSSEUM_FENCE_INFO;
    use l2_net::game::read_ex_colosseum_fence_info;
    use l2_net::game::EX_EVENT_MATCH_SPELLED_INFO;
    use l2_net::game::read_ex_event_match_spelled_info;
    use l2_net::game::EX_EVENT_MATCH_FIRECRACKER;
    use l2_net::game::read_ex_event_match_firecracker;
    use l2_net::game::EX_EVENT_MATCH_TEAM_UNLOCKED;
    use l2_net::game::read_ex_event_match_team_unlocked;
    use l2_net::game::EX_EVENT_MATCH_GM_TEST;
    use l2_net::game::read_ex_event_match_gm_test;
    use l2_net::game::EX_CLOSE_PARTY_ROOM;
    use l2_net::game::read_ex_close_party_room;
    use l2_net::game::EX_FISHING_START;
    use l2_net::game::read_ex_fishing_start;
    use l2_net::game::EX_OPEN_MPCC;
    use l2_net::game::read_ex_open_mpcc;
    use l2_net::game::EX_CLOSE_MPCC;
    use l2_net::game::read_ex_close_mpcc;
    use l2_net::game::EX_OLYMPIAD_MATCH_END;
    use l2_net::game::read_ex_olympiad_match_end;
    use l2_net::game::EX_MAIL_ARRIVED;
    use l2_net::game::read_ex_mail_arrived;
    use l2_net::game::EX_PARTY_ROOM_MEMBER;
    use l2_net::game::read_ex_party_room_member;
    use l2_net::game::EX_MANAGE_PARTY_ROOM_MEMBER;
    use l2_net::game::read_ex_manage_party_room_member;
    use l2_net::game::{EX_ENCHANT_SKILL_LIST, EX_EVENT_MATCH_CREATE, EX_EVENT_MATCH_MANAGE, EX_EVENT_MATCH_MESSAGE, EX_EVENT_MATCH_OBSERVER, EX_EVENT_MATCH_SCORE, EX_EVENT_MATCH_TEAM_INFO, EX_EVENT_MATCH_USER_INFO, EX_FISHING_END, EX_FISHING_HP_REGEN, EX_FISHING_START_COMBAT, EX_HERO_LIST, EX_MULTI_PARTY_COMMAND_CHANNEL_INFO, EX_RECEIVE_ASK_JOIN_MPCC, EX_RECEIVE_AUTO_SOUL_SHOT, EX_RECEIVE_ENCHANT_SKILL_INFO, EX_RECEIVE_OLYMPIAD_MODE, EX_RECEIVE_OLYMPIAD_SPELLED_INFO, EX_RECEIVE_OLYMPIAD_USER_INFO, EX_RECEIVE_PLEDGE_CREST_LARGE, EX_REGEN_MAX, EX_SEND_MANOR_LIST, EX_SERVER_PRIMITIVE, EX_SHOW_CROP_INFO, EX_SHOW_CROP_SETTING, EX_SHOW_MANOR_DEFAULT_INFO, EX_SHOW_PROCURE_CROP_DETAIL, EX_SHOW_QUEST_INFO, EX_SHOW_QUEST_MARK, EX_SHOW_SEED_INFO, EX_SHOW_SEED_SETTING, EX_SHOW_SELL_CROP_LIST, EX_STORAGE_MAX_COUNT, OP_EX_PACKET, read_ex_ask_join_mpcc, read_ex_auto_soul_shot, read_ex_enchant_skill_info, read_ex_enchant_skill_list, read_ex_event_match_create, read_ex_event_match_manage, read_ex_event_match_message, read_ex_event_match_observer, read_ex_event_match_score, read_ex_event_match_team_info, read_ex_event_match_user_info, read_ex_fishing_end, read_ex_fishing_hp_regen, read_ex_fishing_start_combat, read_ex_hero_list, read_ex_multi_party_command_channel_info, read_ex_olympiad_mode, read_ex_olympiad_spelled_info, read_ex_olympiad_user_info, read_ex_pledge_crest_large, read_ex_regen_max, read_ex_send_manor_list, read_ex_server_primitive, read_ex_show_crop_info, read_ex_show_crop_setting, read_ex_show_manor_default_info, read_ex_show_procure_crop_detail, read_ex_show_quest_info, read_ex_show_quest_mark, read_ex_show_seed_info, read_ex_show_seed_setting, read_ex_show_sell_crop_list, read_ex_storage_max_count, read_ex_subopcode};
    let event = match *body.first()? {
        OP_EX_PACKET => match read_ex_subopcode(body) {
            Ok(EX_RECEIVE_AUTO_SOUL_SHOT) => read_ex_auto_soul_shot(body).map(WorldEvent::AutoSoulShot),
            Ok(EX_FISHING_END) => read_ex_fishing_end(body).map(WorldEvent::FishingEnd),
            Ok(EX_FISHING_START_COMBAT) => read_ex_fishing_start_combat(body).map(WorldEvent::FishingStartCombat),
            Ok(EX_FISHING_HP_REGEN) => read_ex_fishing_hp_regen(body).map(WorldEvent::FishingHpRegen),
            Ok(EX_ENCHANT_SKILL_LIST) => read_ex_enchant_skill_list(body).map(WorldEvent::EnchantSkillList),
            Ok(EX_RECEIVE_ENCHANT_SKILL_INFO) => read_ex_enchant_skill_info(body).map(WorldEvent::EnchantSkillInfo),
            Ok(EX_SHOW_QUEST_INFO) => read_ex_show_quest_info(body).map(|()| WorldEvent::ShowQuestInfo),
            Ok(EX_SHOW_QUEST_MARK) => read_ex_show_quest_mark(body).map(WorldEvent::ShowQuestMark),
            Ok(EX_SEND_MANOR_LIST) => read_ex_send_manor_list(body).map(WorldEvent::SendManorList),
            Ok(EX_SHOW_SEED_INFO) => read_ex_show_seed_info(body).map(WorldEvent::ShowSeedInfo),
            Ok(EX_SHOW_CROP_INFO) => read_ex_show_crop_info(body).map(WorldEvent::ShowCropInfo),
            Ok(EX_SHOW_MANOR_DEFAULT_INFO) => read_ex_show_manor_default_info(body).map(WorldEvent::ShowManorDefaultInfo),
            Ok(EX_SHOW_SEED_SETTING) => read_ex_show_seed_setting(body).map(WorldEvent::ShowSeedSetting),
            Ok(EX_SHOW_CROP_SETTING) => read_ex_show_crop_setting(body).map(WorldEvent::ShowCropSetting),
            Ok(EX_SHOW_SELL_CROP_LIST) => read_ex_show_sell_crop_list(body).map(WorldEvent::ShowSellCropList),
            Ok(EX_SHOW_PROCURE_CROP_DETAIL) => read_ex_show_procure_crop_detail(body).map(WorldEvent::ShowProcureCropDetail),
            Ok(EX_HERO_LIST) => read_ex_hero_list(body).map(WorldEvent::HeroList),
            Ok(EX_SERVER_PRIMITIVE) => read_ex_server_primitive(body).map(WorldEvent::ServerPrimitive),
            Ok(EX_RECEIVE_ASK_JOIN_MPCC) => read_ex_ask_join_mpcc(body).map(WorldEvent::AskJoinMpcc),
            Ok(EX_RECEIVE_PLEDGE_CREST_LARGE) => read_ex_pledge_crest_large(body).map(WorldEvent::PledgeCrestLarge),
            Ok(EX_RECEIVE_OLYMPIAD_USER_INFO) => read_ex_olympiad_user_info(body).map(WorldEvent::OlympiadUserInfo),
            Ok(EX_RECEIVE_OLYMPIAD_SPELLED_INFO) => read_ex_olympiad_spelled_info(body).map(WorldEvent::OlympiadSpelledInfo),
            Ok(EX_RECEIVE_OLYMPIAD_MODE) => read_ex_olympiad_mode(body).map(WorldEvent::OlympiadMode),
            Ok(EX_EVENT_MATCH_MANAGE) => read_ex_event_match_manage(body).map(WorldEvent::EventMatchManage),
            Ok(EX_MULTI_PARTY_COMMAND_CHANNEL_INFO) => read_ex_multi_party_command_channel_info(body).map(WorldEvent::MultiPartyChannelInfo),
            Ok(EX_REGEN_MAX) => read_ex_regen_max(body).map(WorldEvent::RegenMax),
            Ok(EX_EVENT_MATCH_OBSERVER) => read_ex_event_match_observer(body).map(WorldEvent::EventMatchObserver),
            Ok(EX_EVENT_MATCH_MESSAGE) => read_ex_event_match_message(body).map(WorldEvent::EventMatchMessage),
            Ok(EX_EVENT_MATCH_SCORE) => read_ex_event_match_score(body).map(WorldEvent::EventMatchScore),
            Ok(EX_EVENT_MATCH_TEAM_INFO) => read_ex_event_match_team_info(body).map(WorldEvent::EventMatchTeamInfo),
            Ok(EX_EVENT_MATCH_CREATE) => read_ex_event_match_create(body).map(WorldEvent::EventMatchCreate),
            Ok(EX_EVENT_MATCH_USER_INFO) => read_ex_event_match_user_info(body).map(WorldEvent::EventMatchUserInfo),
            Ok(EX_COLOSSEUM_FENCE_INFO) => read_ex_colosseum_fence_info(body).map(WorldEvent::ColosseumFenceInfo),
            Ok(EX_EVENT_MATCH_SPELLED_INFO) => read_ex_event_match_spelled_info(body).map(WorldEvent::EventMatchSpelledInfo),
            Ok(EX_EVENT_MATCH_FIRECRACKER) => read_ex_event_match_firecracker(body).map(WorldEvent::EventMatchFirecracker),
            Ok(EX_EVENT_MATCH_TEAM_UNLOCKED) => read_ex_event_match_team_unlocked(body).map(WorldEvent::EventMatchTeamUnlocked),
            Ok(EX_EVENT_MATCH_GM_TEST) => read_ex_event_match_gm_test(body).map(|()| WorldEvent::EventMatchGmTest),
            Ok(EX_CLOSE_PARTY_ROOM) => read_ex_close_party_room(body).map(|()| WorldEvent::ClosePartyRoom),
            Ok(EX_FISHING_START) => read_ex_fishing_start(body).map(WorldEvent::FishingStart),
            Ok(EX_OPEN_MPCC) => read_ex_open_mpcc(body).map(|()| WorldEvent::OpenMpcc),
            Ok(EX_CLOSE_MPCC) => read_ex_close_mpcc(body).map(|()| WorldEvent::CloseMpcc),
            Ok(EX_OLYMPIAD_MATCH_END) => read_ex_olympiad_match_end(body).map(|()| WorldEvent::OlympiadMatchEnd),
            Ok(EX_MAIL_ARRIVED) => read_ex_mail_arrived(body).map(|()| WorldEvent::MailArrived),
            Ok(EX_PARTY_ROOM_MEMBER) => read_ex_party_room_member(body).map(WorldEvent::PartyRoomMember),
            Ok(EX_MANAGE_PARTY_ROOM_MEMBER) => read_ex_manage_party_room_member(body).map(WorldEvent::ManagePartyRoomMember),
            Ok(EX_STORAGE_MAX_COUNT) => read_ex_storage_max_count(body).map(WorldEvent::StorageMaxCount),
            Ok(_) => return None,
            Err(error) => Err(error),
        },
        OP_CHAR_INFO => read_char_info(body).map(|c| WorldEvent::CharInfo(Box::new(c))),
        OP_NPC_INFO => read_npc_info(body).map(WorldEvent::NpcInfo),
        OP_MOVE_TO_LOCATION => read_move_to_location(body).map(WorldEvent::MoveToLocation),
        OP_STOP_MOVE => read_stop_move(body).map(WorldEvent::StopMove),
        OP_VALIDATE_LOCATION => read_validate_location(body).map(WorldEvent::ValidateLocation),
        OP_TELEPORT_TO_LOCATION => read_teleport_to_location(body).map(WorldEvent::TeleportToLocation),
        OP_DELETE_OBJECT => read_delete_object(body).map(WorldEvent::DeleteObject),
        OP_STATUS_UPDATE => read_status_update(body).map(WorldEvent::StatusUpdate),
        OP_CHANGE_MOVE_TYPE => read_change_move_type(body).map(WorldEvent::ChangeMoveType),
        OP_CHANGE_WAIT_TYPE => read_change_wait_type(body).map(WorldEvent::ChangeWaitType),
        OP_DIE => read_die(body).map(WorldEvent::Die),
        OP_REVIVE => read_revive(body).map(WorldEvent::Revive),
        OP_SOCIAL_ACTION => read_social_action(body).map(WorldEvent::SocialAction),
        OP_SAY2 => read_say2(body).map(WorldEvent::Say2),
        OP_NPC_SAY => read_npc_say(body).map(WorldEvent::NpcSay),
        OP_SYSTEM_MESSAGE => read_system_message(body).map(WorldEvent::SystemMessage),
        OP_TARGET_SELECTED => read_target_selected(body).map(WorldEvent::TargetSelected),
        OP_TARGET_UNSELECTED => read_target_unselected(body).map(WorldEvent::TargetUnselected),
        OP_MOVE_TO_PAWN => read_move_to_pawn(body).map(WorldEvent::MoveToPawn),
        OP_RELATION_CHANGED => read_relation_changed(body).map(WorldEvent::RelationChanged),
        OP_MAGIC_SKILL_USE => read_magic_skill_use(body).map(WorldEvent::MagicSkillUse),
        OP_MAGIC_SKILL_LAUNCHED => read_magic_skill_launched(body).map(WorldEvent::MagicSkillLaunched),
        OP_MAGIC_SKILL_CANCELED => read_magic_skill_canceled(body).map(WorldEvent::MagicSkillCanceled),
        OP_SPAWN_ITEM => read_spawn_item(body).map(WorldEvent::SpawnItem),
        OP_ETC_STATUS_UPDATE => read_etc_status_update(body).map(WorldEvent::EtcStatusUpdate),
        OP_MAGIC_EFFECT_ICONS => read_magic_effect_icons(body).map(WorldEvent::MagicEffectIcons),
        OP_PLEDGE_STATUS_CHANGED => read_pledge_status_changed(body).map(WorldEvent::PledgeStatusChanged),
        OP_SEND_MACRO_LIST => read_send_macro_list(body).map(WorldEvent::SendMacroList),
        OP_ITEM_LIST => read_item_list(body).map(WorldEvent::ItemList),
        OP_INVENTORY_UPDATE => read_inventory_update(body).map(WorldEvent::InventoryUpdate),
        OP_EQUIP_UPDATE => read_equip_update(body).map(WorldEvent::EquipUpdate),
        OP_SHORT_CUT_INIT => read_short_cut_init(body).map(WorldEvent::ShortCutInit),
        OP_SHORT_CUT_REGISTER => read_short_cut_register(body).map(WorldEvent::ShortCutRegister),
        OP_SHORT_CUT_DELETE => read_short_cut_delete(body).map(WorldEvent::ShortCutDelete),
        OP_SKILL_LIST => read_skill_list(body).map(WorldEvent::SkillList),
        OP_NPC_HTML_MESSAGE => read_npc_html_message(body).map(WorldEvent::NpcHtmlMessage),
        OP_MY_TARGET_SELECTED => read_my_target_selected(body).map(WorldEvent::MyTargetSelected),
        OP_SKILL_COOL_TIME => read_skill_cool_time(body).map(WorldEvent::SkillCoolTime),
        OP_ACTION_FAILED => read_action_failed(body).map(WorldEvent::ActionFailed),
        OP_ATTACK => read_attack(body).map(WorldEvent::Attack),
        OP_DROP_ITEM => read_drop_item(body).map(WorldEvent::DropItem),
        OP_GET_ITEM => read_get_item(body).map(WorldEvent::GetItem),
        OP_SUN_RISE => read_sun_rise(body).map(WorldEvent::SunRise),
        OP_SUN_SET => read_sun_set(body).map(WorldEvent::SunSet),
        OP_AUTO_ATTACK_START => read_auto_attack_start(body).map(WorldEvent::AutoAttackStart),
        OP_AUTO_ATTACK_STOP => read_auto_attack_stop(body).map(WorldEvent::AutoAttackStop),
        OP_ASK_JOIN_PLEDGE => read_ask_join_pledge(body).map(WorldEvent::AskJoinPledge),
        OP_ASK_JOIN_PARTY => read_ask_join_party(body).map(WorldEvent::AskJoinParty),
        OP_DOOR_INFO => read_door_info(body).map(WorldEvent::DoorInfo),
        OP_DOOR_STATUS_UPDATE => read_door_status_update(body).map(WorldEvent::DoorStatusUpdate),
        OP_PARTY_SMALL_WINDOW_ALL => read_party_small_window_all(body).map(WorldEvent::PartySmallWindowAll),
        OP_PARTY_SMALL_WINDOW_ADD => read_party_small_window_add(body).map(WorldEvent::PartySmallWindowAdd),
        OP_PARTY_SMALL_WINDOW_DELETE_ALL => {
            read_party_small_window_delete_all(body).map(WorldEvent::PartySmallWindowDeleteAll)
        }
        OP_PARTY_SMALL_WINDOW_DELETE => read_party_small_window_delete(body).map(WorldEvent::PartySmallWindowDelete),
        OP_PARTY_SMALL_WINDOW_UPDATE => read_party_small_window_update(body).map(WorldEvent::PartySmallWindowUpdate),
        OP_VEHICLE_INFO => read_vehicle_info(body).map(WorldEvent::VehicleInfo),
        OP_VEHICLE_DEPARTURE => read_vehicle_departure(body).map(WorldEvent::VehicleDeparture),
        OP_GET_ON_VEHICLE => read_get_on_vehicle(body).map(WorldEvent::GetOnVehicle),
        OP_GET_OFF_VEHICLE => read_get_off_vehicle(body).map(WorldEvent::GetOffVehicle),
        OP_START_ROTATING => read_start_rotating(body).map(WorldEvent::StartRotating),
        OP_FINISH_ROTATING => read_finish_rotating(body).map(WorldEvent::FinishRotating),
        OP_SETUP_GAUGE => read_setup_gauge(body).map(WorldEvent::SetupGauge),
        OP_MOVE_TO_LOCATION_IN_VEHICLE => {
            read_move_to_location_in_vehicle(body).map(WorldEvent::MoveToLocationInVehicle)
        }
        OP_STOP_MOVE_IN_VEHICLE => read_stop_move_in_vehicle(body).map(WorldEvent::StopMoveInVehicle),
        OP_RIDE => read_ride(body).map(WorldEvent::Ride),
        OP_SERVER_OBJECT_INFO => read_server_object_info(body).map(WorldEvent::ServerObjectInfo),
        OP_PLAY_SOUND => read_play_sound(body).map(WorldEvent::PlaySound),
        OP_STATIC_OBJECT => read_static_object(body).map(WorldEvent::StaticObject),
        OP_PARTY_MEMBER_POSITION => read_party_member_position(body).map(WorldEvent::PartyMemberPosition),
        OP_ASK_JOIN_ALLIANCE => read_ask_join_alliance(body).map(WorldEvent::AskJoinAlliance),
        OP_PET_STATUS_SHOW => read_pet_status_show(body).map(WorldEvent::PetStatusShow),
        OP_PET_INFO => read_pet_info(body).map(|p| WorldEvent::PetInfo(Box::new(p))),
        OP_PET_STATUS_UPDATE => read_pet_status_update(body).map(WorldEvent::PetStatusUpdate),
        OP_PET_DELETE => read_pet_delete(body).map(WorldEvent::PetDelete),
        OP_VEHICLE_START => read_vehicle_start(body).map(WorldEvent::VehicleStart),
        OP_EARTHQUAKE => read_earthquake(body).map(WorldEvent::Earthquake),
        OP_FLY_TO_LOCATION => read_fly_to_location(body).map(WorldEvent::FlyToLocation),
        OP_SPECIAL_CAMERA => read_special_camera(body).map(WorldEvent::SpecialCamera),
        OP_NICKNAME_CHANGED => read_nickname_changed(body).map(WorldEvent::NicknameChanged),
        OP_SET_SUMMON_REMAIN_TIME => read_set_summon_remain_time(body).map(WorldEvent::SetSummonRemainTime),
        OP_DICE => read_dice(body).map(WorldEvent::Dice),
        OP_CHAIR_SIT => read_chair_sit(body).map(WorldEvent::ChairSit),
        OP_CONFIRM_DLG => read_confirm_dlg(body).map(WorldEvent::ConfirmDlg),
        OP_PARTY_SPELLED => read_party_spelled(body).map(WorldEvent::PartySpelled),
        OP_SHORT_BUFF_STATUS_UPDATE => read_short_buff_status_update(body).map(WorldEvent::ShortBuffStatusUpdate),
        OP_SELL_LIST => read_sell_list(body).map(WorldEvent::SellList),
        OP_BUY_LIST => read_buy_list(body).map(WorldEvent::BuyList),
        OP_TRADE_START => read_trade_start(body).map(WorldEvent::TradeStart),
        OP_TRADE_OWN_ADD => read_trade_own_add(body).map(WorldEvent::TradeOwnAdd),
        OP_TRADE_OTHER_ADD => read_trade_other_add(body).map(WorldEvent::TradeOtherAdd),
        OP_TRADE_DONE => read_trade_done(body).map(WorldEvent::TradeDone),
        OP_TRADE_START_OK => read_trade_start_ok(body).map(WorldEvent::TradeStartOk),
        OP_TRADE_PRESS_OWN_OK => read_trade_press_own_ok(body).map(WorldEvent::TradePressOwnOk),
        OP_TRADE_PRESS_OTHER_OK => read_trade_press_other_ok(body).map(WorldEvent::TradePressOtherOk),
        OP_MANAGE_PLEDGE_POWER => read_manage_pledge_power(body).map(WorldEvent::ManagePledgePower),
        OP_JOIN_PLEDGE => read_join_pledge(body).map(WorldEvent::JoinPledge),
        OP_JOIN_PARTY => read_join_party(body).map(WorldEvent::JoinParty),
        OP_WAREHOUSE_DEPOSIT_LIST => read_warehouse_deposit_list(body).map(WorldEvent::WarehouseDepositList),
        OP_WAREHOUSE_WITHDRAW_LIST => read_warehouse_withdraw_list(body).map(WorldEvent::WarehouseWithdrawList),
        OP_WAREHOUSE_DONE => read_warehouse_done(body).map(WorldEvent::WarehouseDone),
        OP_PLEDGE_SHOW_MEMBER_LIST_ALL => {
            read_pledge_show_member_list_all(body).map(WorldEvent::PledgeShowMemberListAll)
        }
        OP_PLEDGE_SHOW_MEMBER_LIST_UPDATE => {
            read_pledge_show_member_list_update(body).map(WorldEvent::PledgeShowMemberListUpdate)
        }
        OP_PLEDGE_SHOW_MEMBER_LIST_ADD => {
            read_pledge_show_member_list_add(body).map(WorldEvent::PledgeShowMemberListAdd)
        }
        OP_PLEDGE_SHOW_MEMBER_LIST_DELETE => {
            read_pledge_show_member_list_delete(body).map(WorldEvent::PledgeShowMemberListDelete)
        }
        OP_TRADE_REQUEST => read_trade_request(body).map(WorldEvent::TradeRequest),
        OP_PLEDGE_CREST => read_pledge_crest(body).map(WorldEvent::PledgeCrest),
        OP_SHOW_BOARD => read_show_board(body).map(WorldEvent::ShowBoard),
        OP_CHOOSE_INVENTORY_ITEM => read_choose_inventory_item(body).map(WorldEvent::ChooseInventoryItem),
        OP_CLIENT_ACTION => read_client_action(body).map(WorldEvent::ClientAction),
        OP_TRADE_UPDATE => read_trade_update(body).map(WorldEvent::TradeUpdate),
        OP_FRIEND_ADD_REQUEST => read_friend_add_request(body).map(WorldEvent::FriendAddRequest),
        OP_FRIEND_ADD_REQUEST_RESULT => read_friend_add_request_result(body).map(WorldEvent::FriendAddRequestResult),
        OP_FRIEND_ADD => read_friend_add(body).map(WorldEvent::FriendAdd),
        OP_FRIEND_REMOVE => read_friend_remove(body).map(WorldEvent::FriendRemove),
        OP_FRIEND_LIST_LEGACY => read_friend_list_noop(body).map(WorldEvent::FriendListNoop),
        OP_FRIEND_STATUS => read_friend_status_noop(body).map(WorldEvent::FriendStatusNoop),
        OP_MAGIC_LIST => read_magic_list(body).map(WorldEvent::MagicListNoop),
        OP_NEW_CHARACTER_FAIL => read_new_character_fail(body).map(WorldEvent::NewCharacterFailNoop),
        OP_QUEST_LIST => read_quest_list(body).map(WorldEvent::QuestList),
        OP_ENCHANT_RESULT => read_enchant_result(body).map(WorldEvent::EnchantResult),
        OP_PLEDGE_SHOW_MEMBER_LIST_DELETE_ALL => {
            read_pledge_show_member_list_delete_all(body).map(WorldEvent::PledgeShowMemberListDeleteAll)
        }
        OP_PLEDGE_INFO => read_pledge_info(body).map(WorldEvent::PledgeInfo),
        OP_PLEDGE_EXTENDED_INFO => read_pledge_extended_info(body).map(WorldEvent::PledgeExtendedInfo),
        OP_START_PLEDGE_WAR => read_start_pledge_war(body).map(WorldEvent::StartPledgeWarNoop),
        OP_REPLY_START_PLEDGE_WAR => read_reply_start_pledge_war(body).map(WorldEvent::ReplyStartPledgeWarNoop),
        OP_STOP_PLEDGE_WAR => read_stop_pledge_war(body).map(WorldEvent::StopPledgeWarNoop),
        OP_REPLY_STOP_PLEDGE_WAR => read_reply_stop_pledge_war(body).map(WorldEvent::ReplyStopPledgeWarNoop),
        OP_SURRENDER_PLEDGE_WAR => read_surrender_pledge_war(body).map(WorldEvent::SurrenderPledgeWarNoop),
        OP_REPLY_SURRENDER_PLEDGE_WAR => read_reply_surrender_pledge_war(body).map(WorldEvent::ReplySurrenderPledgeWarNoop),
        OP_SET_PLEDGE_CREST => read_set_pledge_crest(body).map(WorldEvent::SetPledgeCrestNoop),
        OP_SURRENDER_PERSONALLY => read_surrender_personally(body).map(WorldEvent::SurrenderPersonallyNoop),
        OP_SERVER_CLOSE_SOCKET => read_server_close_socket(body).map(WorldEvent::ServerCloseSocketNoop),
        OP_REPLY_START_ALLIANCE_WAR => read_reply_start_alliance_war(body).map(WorldEvent::ReplyStartAllianceWarNoop),
        OP_REPLY_STOP_ALLIANCE_WAR => read_reply_stop_alliance_war(body).map(WorldEvent::ReplyStopAllianceWarNoop),
        OP_SURRENDER_ALLIANCE_WAR => read_surrender_alliance_war(body).map(WorldEvent::SurrenderAllianceWarNoop),
        OP_START_ALLIANCE_WAR => read_start_alliance_war(body).map(WorldEvent::StartAllianceWar),
        OP_STOP_ALLIANCE_WAR => read_stop_alliance_war(body).map(WorldEvent::StopAllianceWar),
        OP_MAGIC_AND_SKILL_LIST => read_magic_and_skill_list(body).map(WorldEvent::MagicAndSkillList),
        OP_SKILL_REMAIN_SEC => read_skill_remain_sec(body).map(WorldEvent::SkillRemainSec),
        OP_VEHICLE_CHECK_LOCATION => read_vehicle_check_location(body).map(WorldEvent::VehicleCheckLocation),
        OP_DUMMY => read_dummy(body).map(WorldEvent::DummyNoop),
        OP_BLOCK_LIST => read_block_list(body).map(WorldEvent::BlockListNoop),
        OP_ABNORMAL_VISUAL_EFFECT => read_abnormal_visual_effect(body).map(WorldEvent::AbnormalVisualEffectNoop),
        OP_CREATE_PLEDGE => read_create_pledge(body).map(WorldEvent::CreatePledge),
        OP_WITHDRAWAL_PLEDGE => read_withdrawal_pledge(body).map(WorldEvent::WithdrawalPledge),
        OP_OUST_PLEDGE_MEMBER => read_oust_pledge_member(body).map(WorldEvent::OustPledgeMember),
        OP_DISMISS_PLEDGE => read_dismiss_pledge(body).map(WorldEvent::DismissPledge),
        OP_WITHDRAWAL_PARTY => read_withdrawal_party(body).map(WorldEvent::WithdrawalParty),
        OP_OUST_PARTY_MEMBER => read_oust_party_member(body).map(WorldEvent::OustPartyMember),
        OP_DISMISS_PARTY => read_dismiss_party(body).map(WorldEvent::DismissParty),
        OP_SET_OUST_PLEDGE_MEMBER => read_set_oust_pledge_member(body).map(WorldEvent::SetOustPledgeMemberNoop),
        OP_SET_DISMISS_PLEDGE => read_set_dismiss_pledge(body).map(WorldEvent::SetDismissPledgeNoop),
        OP_SET_OUST_PARTY_MEMBER => read_set_oust_party_member(body).map(WorldEvent::SetOustPartyMemberNoop),
        OP_SET_DISMISS_PARTY => read_set_dismiss_party(body).map(WorldEvent::SetDismissPartyNoop),
        OP_ATTACK_OUT_OF_RANGE => read_attack_out_of_range(body).map(WorldEvent::AttackOutOfRangeNoop),
        OP_ATTACK_IN_COOL_TIME => read_attack_in_cool_time(body).map(WorldEvent::AttackInCoolTimeNoop),
        OP_ATTACK_DEAD_TARGET => read_attack_dead_target(body).map(WorldEvent::AttackDeadTargetNoop),
        OP_VALIDATE_LOCATION_IN_VEHICLE => read_validate_location_in_vehicle(body).map(WorldEvent::ValidateLocationInVehicle),
        OP_GM_HIDE => read_gm_hide(body).map(WorldEvent::GmHide),
        OP_JOIN_ALLIANCE => read_join_alliance(body).map(WorldEvent::JoinAllianceResult),
        OP_WITHDRAW_ALLIANCE => read_withdraw_alliance(body).map(WorldEvent::WithdrawAllianceResult),
        OP_OUST_ALLIANCE_MEMBER_PLEDGE => read_oust_alliance_member_pledge(body).map(WorldEvent::OustAllianceMemberPledgeResult),
        OP_DISMISS_ALLIANCE => read_dismiss_alliance(body).map(WorldEvent::DismissAllianceResult),
        OP_SET_ALLIANCE_CREST => read_set_alliance_crest(body).map(WorldEvent::SetAllianceCrestNoop),
        OP_ALLIANCE_INFO => read_alliance_info(body).map(WorldEvent::AllianceInfoNoop),
        OP_REQUEST_TIME_CHECK => read_request_time_check(body).map(WorldEvent::RequestTimeCheck),
        OP_OBSERVER_START => read_observer_start(body).map(WorldEvent::ObserverStart),
        OP_OBSERVER_END => read_observer_end(body).map(WorldEvent::ObserverEnd),
        OP_NORMAL_CAMERA => read_normal_camera(body).map(WorldEvent::NormalCameraNoop),
        OP_EVENT_TRIGGER => read_event_trigger(body).map(WorldEvent::EventTrigger),
        OP_NET_PING_RESPONSE => read_net_ping_response(body).map(WorldEvent::NetPingResponse),
        OP_SNOOP => read_snoop(body).map(WorldEvent::Snoop),
        OP_CAMERA_MODE => read_camera_mode(body).map(WorldEvent::CameraMode),
        OP_PETITION_VOTE => read_petition_vote(body).map(WorldEvent::PetitionVoteNoop),
        OP_PLEDGE_SHOW_INFO_UPDATE => read_pledge_show_info_update(body).map(WorldEvent::PledgeShowInfoUpdate),
        OP_ACQUIRE_SKILL_LIST => read_acquire_skill_list(body).map(WorldEvent::AcquireSkillList),
        OP_ACQUIRE_SKILL_INFO => read_acquire_skill_info(body).map(WorldEvent::AcquireSkillInfo),
        OP_ACQUIRE_SKILL_DONE => read_acquire_skill_done(body).map(WorldEvent::AcquireSkillDone),
        OP_PARTY_MATCH_LIST => read_party_match_list(body).map(WorldEvent::PartyMatchList),
        OP_PARTY_MATCH_DETAIL => read_party_match_detail(body).map(WorldEvent::PartyMatchDetail),
        OP_PRIVATE_STORE_MANAGE_LIST_SELL => {
            read_private_store_manage_list_sell(body).map(WorldEvent::PrivateStoreManageListSell)
        }
        OP_PRIVATE_STORE_LIST_SELL => read_private_store_list_sell(body).map(WorldEvent::PrivateStoreListSell),
        OP_PRIVATE_STORE_MSG_SELL => read_private_store_msg_sell(body).map(WorldEvent::PrivateStoreMsgSell),
        OP_SHOW_MINIMAP => read_show_minimap(body).map(WorldEvent::ShowMinimap),
        OP_REVIVE_REQUEST => read_revive_request(body).map(WorldEvent::ReviveRequestNoop),
        OP_TUTORIAL_SHOW_HTML => read_tutorial_show_html(body).map(WorldEvent::TutorialShowHtml),
        OP_TUTORIAL_SHOW_QUESTION_MARK => {
            read_tutorial_show_question_mark(body).map(WorldEvent::TutorialShowQuestionMark)
        }
        OP_TUTORIAL_ENABLE_CLIENT_EVENT => {
            read_tutorial_enable_client_event(body).map(WorldEvent::TutorialEnableClientEvent)
        }
        OP_TUTORIAL_CLOSE_HTML => read_tutorial_close_html(body).map(WorldEvent::TutorialCloseHtml),
        OP_ALLIANCE_CREST => read_alliance_crest(body).map(WorldEvent::AllianceCrest),
        OP_PET_ITEM_LIST => read_pet_item_list(body).map(WorldEvent::PetItemList),
        OP_PET_INVENTORY_UPDATE => read_pet_inventory_update(body).map(WorldEvent::PetInventoryUpdate),
        OP_PRIVATE_STORE_MANAGE_LIST_BUY => {
            read_private_store_manage_list_buy(body).map(WorldEvent::PrivateStoreManageListBuy)
        }
        OP_PRIVATE_STORE_LIST_BUY => read_private_store_list_buy(body).map(WorldEvent::PrivateStoreListBuy),
        OP_PRIVATE_STORE_MSG_BUY => read_private_store_msg_buy(body).map(WorldEvent::PrivateStoreMsgBuy),
        OP_PACKAGE_TO_LIST => read_package_to_list(body).map(WorldEvent::PackageToList),
        OP_PACKAGE_SENDABLE_LIST => read_package_sendable_list(body).map(WorldEvent::PackageSendableList),
        OP_MULTI_SELL_LIST => read_multi_sell_list(body).map(WorldEvent::MultiSellList),
        OP_SHOW_TOWN_MAP => read_show_town_map(body).map(WorldEvent::ShowTownMap),
        OP_HENNA_EQUIP_LIST => read_henna_equip_list(body).map(WorldEvent::HennaEquipList),
        OP_HENNA_ITEM_INFO => read_henna_item_info(body).map(WorldEvent::HennaItemInfo),
        OP_HENNA_INFO => read_henna_info(body).map(WorldEvent::HennaInfo),
        OP_HENNA_UNEQUIP_LIST => read_henna_unequip_list(body).map(WorldEvent::HennaUnequipList),
        OP_HENNA_UNEQUIP_INFO => read_henna_unequip_info(body).map(WorldEvent::HennaUnequipInfo),
        OP_GM_VIEW_SKILL_INFO => read_gm_view_skill_info(body).map(WorldEvent::GmViewSkillInfo),
        OP_GM_VIEW_MAGIC_INFO => read_gm_view_magic_info(body).map(WorldEvent::GmViewMagicInfo),
        OP_GM_VIEW_QUEST_INFO => read_gm_view_quest_info(body).map(WorldEvent::GmViewQuestInfo),
        OP_GM_VIEW_ITEM_LIST => read_gm_view_item_list(body).map(WorldEvent::GmViewItemList),
        OP_GM_VIEW_WAREHOUSE_WITHDRAW_LIST => {
            read_gm_view_warehouse_withdraw_list(body).map(WorldEvent::GmViewWarehouseWithdrawList)
        }
        OP_SELL_LIST_PROCURE => read_sell_list_procure(body).map(WorldEvent::SellListProcure),
        OP_GM_HENNA_INFO => read_gm_henna_info(body).map(WorldEvent::GmHennaInfo),
        OP_L2_FRIEND => read_l2_friend(body).map(WorldEvent::L2Friend),
        OP_L2_FRIEND_STATUS => read_l2_friend_status(body).map(WorldEvent::L2FriendStatus),
        OP_GM_VIEW_PLEDGE_INFO => read_gm_view_pledge_info(body).map(WorldEvent::GmViewPledgeInfo),
        OP_GM_VIEW_CHARACTER_INFO => read_gm_view_character_info(body).map(|info| WorldEvent::GmViewCharacterInfo(Box::new(info))),
        OP_BUY_LIST_SEED => read_buy_list_seed(body).map(WorldEvent::BuyListSeed),
        OP_SHOW_RADAR => read_show_radar(body).map(WorldEvent::ShowRadar),
        OP_DELETE_RADAR => read_delete_radar(body).map(WorldEvent::DeleteRadar),
        OP_RADAR_CONTROL => read_radar_control(body).map(WorldEvent::RadarControl),
        OP_SHOP_PREVIEW_LIST => read_shop_preview_list(body).map(WorldEvent::ShopPreviewList),
        OP_SHOP_PREVIEW_INFO => read_shop_preview_info(body).map(WorldEvent::ShopPreviewInfo),
        OP_FRIEND_LIST => read_friend_list(body).map(WorldEvent::FriendList),
        OP_FRIEND_RECV_MSG => read_friend_recv_msg(body).map(WorldEvent::FriendRecvMsg),
        OP_CASTLE_SIEGE_INFO => read_castle_siege_info(body).map(WorldEvent::CastleSiegeInfo),
        OP_CASTLE_SIEGE_ATTACKER_LIST => read_castle_siege_attacker_list(body).map(WorldEvent::CastleSiegeAttackerList),
        OP_CASTLE_SIEGE_DEFENDER_LIST => read_castle_siege_defender_list(body).map(WorldEvent::CastleSiegeDefenderList),
        OP_RECIPE_BOOK_ITEM_LIST => read_recipe_book_item_list(body).map(WorldEvent::RecipeBookItemList),
        OP_RECIPE_ITEM_MAKE_INFO => read_recipe_item_make_info(body).map(WorldEvent::RecipeItemMakeInfo),
        OP_RECIPE_SHOP_MANAGE_LIST => read_recipe_shop_manage_list(body).map(WorldEvent::RecipeShopManageList),
        OP_RECIPE_SHOP_SELL_LIST => read_recipe_shop_sell_list(body).map(WorldEvent::RecipeShopSellList),
        OP_RECIPE_SHOP_ITEM_INFO => read_recipe_shop_item_info(body).map(WorldEvent::RecipeShopItemInfo),
        OP_RECIPE_SHOP_MSG => read_recipe_shop_msg(body).map(WorldEvent::RecipeShopMsg),
        OP_SHOW_CALCULATOR => read_show_calculator(body).map(WorldEvent::ShowCalculator),
        OP_MON_RACE_INFO => read_mon_race_info(body).map(WorldEvent::MonRaceInfo),
        OP_SHOW_XMAS_SEAL => read_show_xmas_seal(body).map(WorldEvent::ShowXMasSeal),
        OP_SSQ_STATUS => read_ssq_status(body).map(WorldEvent::SsqStatus),
        OP_AGIT_DECO_INFO => read_agit_deco_info(body).map(WorldEvent::AgitDecoInfo),
        OP_SSQ_INFO => read_ssq_info(body).map(WorldEvent::SsqInfo),
        _ => return None,
    };
    Some(event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2_net::login::GameServer;
    use l2_net::wire::Writer;

    #[test]
    fn a_key_packet_with_a_different_protocol_logs_out_and_reports_the_mismatch_10065c90() {
        let auth = Authorization {
            login_key: [1, 2],
            play_key: [3, 4],
            server: GameServer { id: 1, ip: [127, 0, 0, 1], port: 7777, players: 0, max_players: 0, online: true, extra: Default::default() },
        };
        let mut s = Session::new("conta".into(), auth);
        let (ev, out, crypt) = s.on_body(&[0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
        assert!(matches!(ev, Some(SessionEvent::Refused(ref text)) if text == PROTOCOL_MISMATCH) && crypt.is_none());
        assert_eq!(out, vec![l2_net::game::packets::logout()]);
        let (ev, out, _) = s.on_body(&[0, 1, 1]);
        assert!(matches!(ev, Some(SessionEvent::Refused(ref text)) if text != PROTOCOL_MISMATCH) && out.is_empty());
    }

    #[test]
    fn key_packet_then_auth_login_then_characters_then_select_then_world() {
        let auth = Authorization {
            login_key: [1, 2],
            play_key: [3, 4],
            server: GameServer { id: 1, ip: [127, 0, 0, 1], port: 7777, players: 0, max_players: 0, online: true, extra: Default::default() },
        };
        let mut s = Session::new("conta".into(), auth);
        assert!(matches!(s.state(), SessionState::AwaitingKey));
        let (ev, out, crypt) = s.on_body(&[0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
        assert!(ev.is_none() && crypt.is_some());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0][0], 0x08);
        let mut body = vec![0x13u8];
        body.extend_from_slice(&0i32.to_le_bytes());
        let (ev, out, _) = s.on_body(&body);
        assert!(matches!(ev, Some(SessionEvent::Characters(ref c)) if c.is_empty()) && out.is_empty());
        assert!(matches!(s.state(), SessionState::Selecting));
        assert_eq!(s.select(0)[0], 0x0d);
        assert!(matches!(s.state(), SessionState::Entering));
        let (ev, out, _) = s.on_body(&[0x15, 0, 0, 0, 0]);
        assert!(matches!(ev, Some(SessionEvent::Selected)));
        assert_eq!(out, vec![enter_world()]);
        let (ev, _, _) = s.on_body(&[0x45, 0, 0, 0, 0]);
        assert!(matches!(ev, Some(SessionEvent::World(WorldEvent::ShortCutInit(ref init))) if init.shortcuts.is_empty()));
        assert!(matches!(s.state(), SessionState::Entering));
        let mut w = Writer::new();
        w.write_c(0x04);
        w.write_d(1);
        w.write_d(2);
        w.write_d(3);
        w.write_d(4);
        w.write_d(5);
        w.write_s("W");
        w.write_d(0);
        w.write_d(0);
        w.write_d(0);
        w.write_d(1);
        w.write_b(&[0; 500]);
        let (ev, out, _) = s.on_body(&w.into_bytes());
        assert!(matches!(ev, Some(SessionEvent::UserInfo(ref h)) if h.name == "W"));
        assert_eq!(out, vec![request_skill_list(), request_manor_list()]);
        assert!(matches!(s.state(), SessionState::InWorld));
    }

    #[test]
    fn malformed_char_select_info_does_not_disconnect() {
        let auth = Authorization {
            login_key: [1, 2],
            play_key: [3, 4],
            server: GameServer { id: 1, ip: [127, 0, 0, 1], port: 7777, players: 0, max_players: 0, online: true, extra: Default::default() },
        };
        let mut s = Session::new("conta".into(), auth);
        s.on_body(&[0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
        let state_before = s.state();
        let mut body = vec![0x13u8];
        body.extend_from_slice(&1i32.to_le_bytes());
        let (ev, out, crypt) = s.on_body(&body);
        assert!(matches!(ev, Some(SessionEvent::Malformed { opcode: 0x13, .. })));
        assert!(out.is_empty() && crypt.is_none());
        assert_eq!(s.state(), state_before);
    }

    #[test]
    fn client_set_time_is_decoded_in_any_state() {
        let auth = Authorization {
            login_key: [1, 2],
            play_key: [3, 4],
            server: GameServer { id: 1, ip: [0; 4], port: 0, players: 0, max_players: 0, online: true, extra: Default::default() },
        };
        let mut s = Session::new("conta".into(), auth);
        let body = [0xec, 0x68, 0x01, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00];
        let (ev, out, crypt) = s.on_body(&body);
        assert!(matches!(ev, Some(SessionEvent::Time(t)) if t.minutes == 360 && t.ratio == 6));
        assert!(out.is_empty() && crypt.is_none());
        assert!(matches!(s.state(), SessionState::AwaitingKey));
    }

    #[test]
    fn auth_login_fail_is_refused_and_unknown_opcodes_are_reported() {
        let auth = Authorization {
            login_key: [1, 2],
            play_key: [3, 4],
            server: GameServer { id: 1, ip: [0; 4], port: 0, players: 0, max_players: 0, online: true, extra: Default::default() },
        };
        let mut s = Session::new("conta".into(), auth);
        s.on_body(&[0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
        let (ev, _, _) = s.on_body(&[0x14, 0, 0, 0, 0]);
        assert!(matches!(ev, Some(SessionEvent::Refused(_))));
        let (ev, _, _) = s.on_body(&[0x7f, 1, 2, 3]);
        assert!(matches!(ev, Some(SessionEvent::Unknown { opcode: 0x7f, size: 4 })));
    }

    fn in_world() -> Session {
        let auth = Authorization {
            login_key: [1, 2],
            play_key: [3, 4],
            server: GameServer { id: 1, ip: [0; 4], port: 0, players: 0, max_players: 0, online: true, extra: Default::default() },
        };
        let mut s = Session::new("conta".into(), auth);
        s.on_body(&[0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
        let mut w = Writer::new();
        w.write_c(0x04);
        for v in [1, 2, 3, 4, 5] {
            w.write_d(v);
        }
        w.write_s("W");
        w.write_b(&[0; 500]);
        s.on_body(&w.into_bytes());
        assert_eq!(s.state(), SessionState::InWorld);
        s
    }

    #[test]
    fn restart_response_requires_the_original_d_s_body() {
        let mut s = in_world();
        let mut w = Writer::new();
        w.write_c(OP_RESTART_RESPONSE);
        w.write_d(1);
        w.write_s("ok");
        let (event, out, crypt) = s.on_body(&w.into_bytes());
        assert!(matches!(event, Some(SessionEvent::RestartResponse(response)) if response.result == 1 && response.message == "ok"));
        assert!(out.is_empty() && crypt.is_none());
        assert_eq!(s.state(), SessionState::AwaitingCharacters);
        let (event, _, _) = s.on_body(&[OP_RESTART_RESPONSE, 1, 0, 0, 0]);
        assert!(matches!(event, Some(SessionEvent::Malformed { opcode: OP_RESTART_RESPONSE, .. })));
        assert_eq!(s.state(), SessionState::AwaitingCharacters);
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
    fn in_world_bodies_become_world_events() {
        let mut s = in_world();
        let (ev, out, crypt) = s.on_body(&ds(0x01, &[7, 100, 200, -30, 10, 20, -3]));
        assert!(out.is_empty() && crypt.is_none());
        match ev {
            Some(SessionEvent::World(WorldEvent::MoveToLocation(m))) => assert_eq!((m.object_id, m.to_x), (7, 100)),
            other => panic!("esperava MoveToLocation, veio {other:?}"),
        }
        let (ev, _, _) = s.on_body(&ds(0x12, &[7, 0]));
        assert!(matches!(ev, Some(SessionEvent::World(WorldEvent::DeleteObject(d))) if d.object_id == 7));
        let (ev, _, _) = s.on_body(&ds(0x0e, &[7, 1, 9, 50]));
        assert!(matches!(ev, Some(SessionEvent::World(WorldEvent::StatusUpdate(_)))));
        let mut w = Writer::new();
        w.write_c(0x4a);
        w.write_d(5);
        w.write_d(0);
        w.write_s("Luzal");
        w.write_s("oi");
        let (ev, _, _) = s.on_body(&w.into_bytes());
        assert!(matches!(ev, Some(SessionEvent::World(WorldEvent::Say2(ref say))) if say.name() == "Luzal"));
        assert_eq!(s.state(), SessionState::InWorld);
    }

    #[test]
    fn receive_warehouse_done_104037b0_dispatches_in_both_world_states() {
        for state in [SessionState::Entering, SessionState::InWorld] {
            let mut session = in_world();
            session.state = state;
            for value in [0, 1, -1] {
                let mut body = ds(0x43, &[value]);
                body.extend_from_slice(&[0xaa, 0xbb]);
                let (event, outgoing, crypt) = session.on_body(&body);
                assert!(matches!(event, Some(SessionEvent::World(WorldEvent::WarehouseDone(done))) if done.success == (value != 0)));
                assert!(outgoing.is_empty());
                assert!(crypt.is_none());
                assert_eq!(session.state(), state);
            }
            for len in 1..5 {
                let (event, outgoing, _) = session.on_body(&ds(0x43, &[0])[..len]);
                assert!(matches!(event, Some(SessionEvent::Malformed { opcode: 0x43, .. })));
                assert!(outgoing.is_empty());
            }
        }
    }

    #[test]
    fn malformed_world_bodies_are_reported_and_unknown_opcodes_kept() {
        let mut s = in_world();
        let (ev, _, _) = s.on_body(&ds(0x01, &[7, 100, 200]));
        assert!(matches!(ev, Some(SessionEvent::Malformed { opcode: 0x01, size: 13 })));
        let (ev, _, _) = s.on_body(&ds(0x01, &[7, 100, 200, -30, 10, 20, -3, 0]));
        assert!(matches!(ev, Some(SessionEvent::World(WorldEvent::MoveToLocation(_)))));
        let (ev, _, _) = s.on_body(&[0x8d, 1, 2, 3]);
        assert!(matches!(ev, Some(SessionEvent::Malformed { opcode: 0x8d, size: 4 })));
        assert_eq!(s.state(), SessionState::InWorld);
    }

    #[test]
    fn receive_trade_done_10402340_ignores_tail_without_hiding_truncation() {
        for state in [SessionState::Entering, SessionState::InWorld] {
            let mut session = in_world();
            session.state = state;
            let mut body = ds(OP_TRADE_DONE, &[1]);
            body.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
            let (event, outgoing, crypt) = session.on_body(&body);
            assert!(matches!(event, Some(SessionEvent::World(WorldEvent::TradeDone(done))) if done.success));
            assert!(outgoing.is_empty() && crypt.is_none());
            let (event, _, _) = session.on_body(&body[..4]);
            assert!(matches!(event, Some(SessionEvent::Malformed { opcode: OP_TRADE_DONE, size: 4 })));
        }
    }

    #[test]
    fn world_opcodes_before_entering_are_unknown() {
        let auth = Authorization {
            login_key: [1, 2],
            play_key: [3, 4],
            server: GameServer { id: 1, ip: [0; 4], port: 0, players: 0, max_players: 0, online: true, extra: Default::default() },
        };
        let mut s = Session::new("conta".into(), auth);
        s.on_body(&[0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
        let (ev, _, _) = s.on_body(&ds(0x01, &[7, 100, 200, -30, 10, 20, -3]));
        assert!(matches!(ev, Some(SessionEvent::Unknown { opcode: 0x01, size: 29 })));
    }

    #[test]
    fn extended_storage_dispatch_preserves_inventory_and_full_subopcode() {
        let values: [i32; 7] = [999, 120, 777, 4, 5, 6, 7];
        let mut body = vec![0xfe, 0x2e, 0];
        for value in values {
            body.extend_from_slice(&value.to_le_bytes());
        }
        for state in [SessionState::Entering, SessionState::InWorld] {
            let mut session = in_world();
            session.state = state;
            let mut world = crate::world::World { inventory_limit: 80, ..Default::default() };
            let (event, _, _) = session.on_body(&body);
            let event = event.expect("evento estendido");
            assert!(matches!(&event, SessionEvent::World(WorldEvent::StorageMaxCount(limits)) if limits.values == values));
            world.apply(&event);
            assert_eq!(world.inventory_limit, 80);
            assert_eq!(world.storage_limits.as_ref().map(|limits| limits.warehouse_limit()), Some(120));
            assert_eq!(session.state(), state);
        }
        for len in 1..body.len() {
            assert!(matches!(decode_world(&body[..len]), Some(Err(_))));
        }
        body[2] = 1;
        assert!(decode_world(&body).is_none());
        body[1] = 0xff;
        body[2] = 0xff;
        assert!(decode_world(&body).is_none());
    }

    #[test]
    fn lobby_results_become_session_events_and_game_guard_is_answered() {
        let auth = Authorization {
            login_key: [1, 2],
            play_key: [3, 4],
            server: GameServer { id: 1, ip: [0; 4], port: 0, players: 0, max_players: 0, online: true, extra: Default::default() },
        };
        let mut s = Session::new("conta".into(), auth);
        s.on_body(&[0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
        let (ev, _, _) = s.on_body(&ds(0x17, &[0]));
        assert!(matches!(ev, Some(SessionEvent::CharTemplates(t)) if t.templates.is_empty()));
        let (ev, _, _) = s.on_body(&[0x19]);
        assert!(matches!(ev, Some(SessionEvent::CharCreated)));
        let (ev, _, _) = s.on_body(&ds(0x1a, &[3]));
        assert!(matches!(ev, Some(SessionEvent::CharCreateFailed(3))));
        let (ev, _, _) = s.on_body(&[0x23]);
        assert!(matches!(ev, Some(SessionEvent::CharDeleted)));
        let (ev, _, _) = s.on_body(&ds(0x24, &[2]));
        assert!(matches!(ev, Some(SessionEvent::CharDeleteFailed(2))));
        let (ev, _, _) = s.on_body(&[0x26]);
        assert!(matches!(ev, Some(SessionEvent::ServerClosed)));
        let (ev, out, _) = s.on_body(&ds(0xf9, &[1, 2, 3, 4]));
        assert!(matches!(ev, Some(SessionEvent::GameGuardQuery([1, 2, 3, 4]))));
        assert_eq!(out, vec![game_guard_reply([1, 2, 3, 4])]);
        let (ev, _, _) = s.on_body(&ds(0x1a, &[]));
        assert!(matches!(ev, Some(SessionEvent::Malformed { opcode: 0x1a, .. })));
        let mut s = in_world();
        let (ev, _, _) = s.on_body(&[0x7e]);
        assert!(matches!(ev, Some(SessionEvent::LeftWorld)));
        assert_eq!(s.state(), SessionState::AwaitingCharacters);
    }

    #[test]
    fn decode_world_covers_the_frente_p_world_opcodes() {
        let mut party = vec![0x4eu8];
        party.extend_from_slice(&1i32.to_le_bytes());
        party.extend_from_slice(&2i32.to_le_bytes());
        party.extend_from_slice(&0i32.to_le_bytes());
        for (body, expect) in [
            (vec![0x05, 1, 0, 0, 0, 2, 0, 0, 0, 5, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0], "Attack"),
            (ds(0x0c, &[1, 2, 3, 4, 5, 6, 0, 1, 1]), "DropItem"),
            (ds(0x0d, &[1, 2, 3, 4, 5]), "GetItem"),
            (vec![0x1c], "SunRise"),
            (vec![0x1d], "SunSet"),
            (ds(0x2b, &[1]), "AutoAttackStart"),
            (ds(0x2c, &[1]), "AutoAttackStop"),
            (vec![0x32, 1, 0, 0, 0, 0, 0], "AskJoinPledge"),
            (vec![0x39, 0, 0, 1, 0, 0, 0], "AskJoinParty"),
            (ds(0x4c, &[1, 2]), "DoorInfo"),
            (ds(0x4d, &[1, 0, 0, 0, 2, 0, 0]), "DoorStatusUpdate"),
            (party, "PartySmallWindowAll"),
            (vec![0x50], "PartySmallWindowDeleteAll"),
            (vec![0x51, 1, 0, 0, 0, 0, 0], "PartySmallWindowDelete"),
            (ds(0x59, &[1, 0, 0, 0, 0]), "VehicleInfo"),
            (ds(0x5a, &[1, 0, 0, 0, 0, 0]), "VehicleDeparture"),
            (ds(0x5c, &[1, 2, 0, 0, 0]), "GetOnVehicle"),
            (ds(0x5d, &[1, 2, 0, 0, 0]), "GetOffVehicle"),
            (ds(0x62, &[1, 0, 0, 0]), "StartRotating"),
            (ds(0x63, &[1, 0, 0]), "FinishRotating"),
            (ds(0x6d, &[0, 1, 1]), "SetupGauge"),
            (ds(0x71, &[1, 2, 0, 0, 0, 0, 0, 0]), "MoveToLocationInVehicle"),
            (ds(0x72, &[1, 2, 0, 0, 0, 0]), "StopMoveInVehicle"),
            (ds(0x86, &[1, 1, 1, 1]), "Ride"),
            (ds(0x99, &[1, 2]), "StaticObject"),
            (ds(0xa7, &[0]), "PartyMemberPosition"),
            (vec![0xa8, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0], "AskJoinAlliance"),
            (ds(0xb0, &[2]), "PetStatusShow"),
            (ds(0xb6, &[2, 1]), "PetDelete"),
            (ds(0xba, &[1, 1]), "VehicleStart"),
            (ds(0xc4, &[1, 2, 3, 4, 5, 0]), "Earthquake"),
            (ds(0xc5, &[1, 2, 3, 4, 5, 6, 7]), "FlyToLocation"),
            (ds(0xc7, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0]), "SpecialCamera"),
            (vec![0xcc, 1, 0, 0, 0, 0, 0], "NicknameChanged"),
            (ds(0xd1, &[1, 2]), "SetSummonRemainTime"),
            (ds(0xd4, &[1, 2, 3, 4, 5, 6]), "Dice"),
            (ds(0xe1, &[1, 2]), "ChairSit"),
            (ds(0xed, &[1, 0]), "ConfirmDlg"),
            (ds(0xee, &[0, 1, 0]), "PartySpelled"),
            (ds(0xf4, &[1, 1, 1]), "ShortBuffStatusUpdate"),
        ] {
            let event = decode_world(&body).expect("opcode do mundo").expect("corpo valido");
            assert!(format!("{event:?}").starts_with(expect), "{expect}: {event:?}");
        }
        let mut w = Writer::new();
        w.write_c(0x98);
        w.write_d(0);
        w.write_s("x");
        for _ in 0..6 {
            w.write_d(0);
        }
        assert!(matches!(decode_world(&w.into_bytes()), Some(Ok(WorldEvent::PlaySound(_)))));
        assert!(matches!(decode_world(&[0x4f, 1]), Some(Err(_))));
        assert!(matches!(decode_world(&[0x52, 1]), Some(Err(_))));
        assert!(matches!(decode_world(&[0x8c, 1]), Some(Err(_))));
        assert!(matches!(decode_world(&[0xb1, 1]), Some(Err(_))));
        assert!(matches!(decode_world(&[0xb5, 1]), Some(Err(_))));
        assert!(matches!(decode_world(&[0x8f]), Some(Err(_))));
    }

    #[test]
    fn decode_world_covers_the_frente_p_window_opcodes() {
        let mut pledge_all = vec![0x53u8, 1, 0, 0, 0, 0, 0, 0, 0];
        for _ in 0..9 {
            pledge_all.extend_from_slice(&0i32.to_le_bytes());
        }
        pledge_all.extend_from_slice(&[0, 0]);
        pledge_all.extend_from_slice(&0i32.to_le_bytes());
        pledge_all.extend_from_slice(&0i32.to_le_bytes());
        pledge_all.extend_from_slice(&0i32.to_le_bytes());
        let mut pledge_extended = Writer::new();
        pledge_extended.write_c(0x84);
        pledge_extended.write_s("lead");
        pledge_extended.write_d(1);
        pledge_extended.write_d(2);
        pledge_extended.write_s("middle");
        for value in 3..11 {
            pledge_extended.write_d(value);
        }
        pledge_extended.write_s("tail");
        pledge_extended.write_d(11);
        let pledge_extended = pledge_extended.into_bytes();
        for (body, expect) in [
            (vec![0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "SellList"),
            (vec![0x11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "BuyList"),
            (vec![0x1e, 1, 0, 0, 0, 0, 0], "TradeStart"),
            (vec![0x20, 0, 0], "TradeOwnAdd"),
            (vec![0x21, 0, 0], "TradeOtherAdd"),
            (ds(0x22, &[1]), "TradeDone"),
            (vec![0x1f], "TradeStartOk"),
            (vec![0x75], "TradePressOwnOk"),
            (vec![0x7c], "TradePressOtherOk"),
            (ds(0x30, &[0, 0]), "ManagePledgePower"),
            (ds(0x33, &[1]), "JoinPledge"),
            (ds(0x3a, &[1]), "JoinParty"),
            (vec![0x41, 1, 0, 0, 0, 0, 0, 0, 0], "WarehouseDepositList"),
            (vec![0x42, 1, 0, 0, 0, 0, 0, 0, 0], "WarehouseWithdrawList"),
            (pledge_all, "PledgeShowMemberListAll"),
            (
                vec![0x54, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                "PledgeShowMemberListUpdate",
            ),
            (vec![0x55, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "PledgeShowMemberListAdd"),
            (vec![0x56, 0, 0], "PledgeShowMemberListDelete"),
            (ds(0x5e, &[7]), "TradeRequest"),
            (ds(0x6c, &[5, 0]), "PledgeCrest"),
            (vec![0x6e, 0], "ShowBoard"),
            (ds(0x6f, &[57]), "ChooseInventoryItem"),
            (ds(0x89, &[1]), "ClientAction"),
            (vec![0x74, 0, 0], "TradeUpdate"),
            (vec![0x7d, 0, 0], "FriendAddRequest"),
            (ds(0x77, &[1]), "FriendAddRequestResult"),
            (vec![0x78], "FriendAdd"),
            (vec![0x79], "FriendRemove"),
            (vec![0x7a], "FriendList"),
            (vec![0x7b], "FriendStatus"),
            (vec![0x57], "MagicList"),
            (vec![0x18], "NewCharacterFail"),
            (vec![0x9e], "ReviveRequest"),
            (vec![0x80, 0, 0], "QuestList"),
            (ds(0x81, &[0]), "EnchantResult"),
            (vec![0x82], "PledgeShowMemberListDeleteAll"),
            (vec![0x83, 1, 0, 0, 0, 0, 0, 0, 0], "PledgeInfo"),
            (pledge_extended, "PledgeExtendedInfo"),
            (vec![0x65], "StartPledgeWar"),
            (vec![0x66], "ReplyStartPledgeWar"),
            (vec![0x67], "StopPledgeWar"),
            (vec![0x68], "ReplyStopPledgeWar"),
            (vec![0x69], "SurrenderPledgeWar"),
            (vec![0x6a], "ReplySurrenderPledgeWar"),
            (vec![0x6b], "SetPledgeCrest"),
            (vec![0x85], "SurrenderPersonally"),
            (ds(0x8a, &[0, 0]), "AcquireSkillList"),
            (ds(0x8b, &[1, 1, 0, 0, 0]), "AcquireSkillInfo"),
            (vec![0x8e], "AcquireSkillDone"),
            (ds(0x96, &[0, 0]), "PartyMatchList"),
            (
                vec![0x97, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                "PartyMatchDetail",
            ),
            (ds(0x9a, &[7, 0, 0, 0, 0]), "PrivateStoreManageListSell"),
            (ds(0x9b, &[7, 0, 0, 0]), "PrivateStoreListSell"),
            (vec![0x9c, 7, 0, 0, 0, 0, 0], "PrivateStoreMsgSell"),
            (ds(0x9d, &[1665]), "ShowMinimap"),
            (vec![0xa0, 0, 0], "TutorialShowHtml"),
            (ds(0xa1, &[1]), "TutorialShowQuestionMark"),
            (ds(0xa2, &[1]), "TutorialEnableClientEvent"),
            (vec![0xa3], "TutorialCloseHtml"),
            (ds(0xa4, &[1, 2, 3, 4, 5]), "ShowRadar"),
            (ds(0xa5, &[1, 2, 3]), "DeleteRadar"),
            (ds(0xae, &[5, 0]), "AllianceCrest"),
            (vec![0xaf], "ServerCloseSocket"),
            (vec![0xbd], "ReplyStartAllianceWar"),
            (vec![0xbf], "ReplyStopAllianceWar"),
            (vec![0xc0], "SurrenderAllianceWar"),
            (vec![0xbc, b'A', 0, 0, 0, b'E', 0, 0, 0], "StartAllianceWar"),
            (vec![0xbe, b'A', 0, 0, 0, b'E', 0, 0, 0], "StopAllianceWar"),
            (vec![0x40, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0], "MagicAndSkillList"),
            (vec![0xd2, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0], "SkillRemainSec"),
            (vec![0x5b, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0], "VehicleCheckLocation"),
            (vec![0x70], "Dummy"),
            (vec![0x9f], "AbnormalVisualEffect"),
            (vec![0x31, 9, 0, 0, 0], "CreatePledge"),
            (vec![0x34, 9, 0, 0, 0], "WithdrawalPledge"),
            (vec![0x35, 9, 0, 0, 0], "OustPledgeMember"),
            (vec![0x37, 9, 0, 0, 0], "DismissPledge"),
            (vec![0x3b, 7, 0, 0, 0], "WithdrawalParty"),
            (vec![0x3c, 7, 0, 0, 0], "OustPartyMember"),
            (vec![0x3e, 7, 0, 0, 0], "DismissParty"),
            (vec![0x36], "SetOustPledgeMember"),
            (vec![0x38], "SetDismissPledge"),
            (vec![0x3d], "SetOustPartyMember"),
            (vec![0x3f], "SetDismissParty"),
            (vec![0x08], "AttackOutOfRange"),
            (vec![0x09], "AttackInCoolTime"),
            (vec![0x0a], "AttackDeadTarget"),
            (vec![0x73, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0], "ValidateLocationInVehicle"),
            (vec![0x8d, 7, 0, 0, 0], "GmHide"),
            (vec![0xa9, 4, 0, 0, 0], "JoinAllianceResult"),
            (vec![0xaa, 4, 0, 0, 0], "WithdrawAllianceResult"),
            (vec![0xab, 4, 0, 0, 0], "OustAllianceMemberPledgeResult"),
            (vec![0xac, 4, 0, 0, 0], "DismissAllianceResult"),
            (vec![0xad], "SetAllianceCrestNoop"),
            (vec![0xb4], "AllianceInfoNoop"),
            (vec![0xbb, 9, 0, 0, 0], "RequestTimeCheck"),
            (vec![0xdf, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0], "ObserverStart"),
            (vec![0xe0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0], "ObserverEnd"),
            (vec![0xc8], "NormalCameraNoop"),
            (vec![0xcf, 7, 0, 0, 0, 1], "EventTrigger"),
            (vec![0xd3, 9, 0, 0, 0], "NetPingResponse"),
            (vec![0xd5, 7, 0, 0, 0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0], "Snoop"),
            (vec![0xf1, 2, 0, 0, 0], "CameraMode"),
            (vec![0xf6], "PetitionVoteNoop"),
            (vec![0xb2, 0, 0], "PetItemList"),
            (vec![0xb3, 0, 0], "PetInventoryUpdate"),
            (ds(0xb7, &[7, 0, 0, 0]), "PrivateStoreManageListBuy"),
            (ds(0xb8, &[7, 0, 0]), "PrivateStoreListBuy"),
            (vec![0xb9, 7, 0, 0, 0, 0, 0], "PrivateStoreMsgBuy"),
            (ds(0xc2, &[0]), "PackageToList"),
            (ds(0xc3, &[9, 0, 0]), "PackageSendableList"),
            (ds(0xd0, &[1, 1, 1, 40, 0]), "MultiSellList"),
            (vec![0xde, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "ShowTownMap"),
            (ds(0xe2, &[0, 3, 0]), "HennaEquipList"),
            (vec![0xe4, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0], "HennaInfo"),
            (vec![0xe8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "BuyListSeed"),
            (ds(0xeb, &[0, 1, 2, 3, 4]), "RadarControl"),
            (vec![0xef, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "ShopPreviewList"),
            (ds(0xf0, &[0; 16]), "ShopPreviewInfo"),
            (ds(0xfa, &[0]), "FriendList"),
            (vec![0xfd, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "FriendRecvMsg"),
        ] {
            let event = decode_world(&body).expect("opcode do mundo").expect("corpo valido");
            assert!(format!("{event:?}").starts_with(expect), "{expect}: {event:?}");
        }
        assert!(matches!(decode_world(&[0xe3, 1]), Some(Err(_))));
        assert!(matches!(decode_world(&[0x88, 1]), Some(Err(_))));
    }

    #[test]
    fn decode_world_covers_the_frente_p_rare_opcodes() {
        for (body, expect) in [
            (
                vec![0xc9, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                "CastleSiegeInfo",
            ),
            (ds(0xca, &[3, 0, 1, 0, 0, 0]), "CastleSiegeAttackerList"),
            (ds(0xcb, &[3, 0, 1, 0, 0, 0]), "CastleSiegeDefenderList"),
            (ds(0xd6, &[1, 500, 0]), "RecipeBookItemList"),
            (ds(0xd7, &[10, 0, 1, 2, 1]), "RecipeItemMakeInfo"),
            (ds(0xd8, &[0, 1, 2, 0, 0]), "RecipeShopManageList"),
            (ds(0xd9, &[7, 1, 2, 3, 0]), "RecipeShopSellList"),
            (ds(0xda, &[7, 10, 1, 2, 1, 0]), "RecipeShopItemInfo"),
            (vec![0xdb, 7, 0, 0, 0, 0, 0], "RecipeShopMsg"),
            (ds(0xdc, &[4393]), "ShowCalculator"),
            (ds(0xdd, &[0, 0, 0]), "MonRaceInfo"),
            (ds(0xf2, &[5555]), "ShowXMasSeal"),
            (vec![0xf5, 9, 2], "SsqStatus"),
            (ds(0xf7, &[21]), "AgitDecoInfo"),
            (vec![0xf8, 1, 1], "SsqInfo"),
        ] {
            let event = decode_world(&body).expect("opcode do mundo").expect("corpo valido");
            assert!(format!("{event:?}").starts_with(expect), "{expect}: {event:?}");
        }
        assert!(matches!(decode_world(&[0xc9, 1]), Some(Err(_))));
    }

    #[test]
    fn decode_world_covers_every_world_opcode() {
        for (body, expect) in [
            (ds(0x01, &[1, 0, 0, 0, 0, 0, 0]), "MoveToLocation"),
            (ds(0x06, &[1, 0, 0, 0, 0, 0]), "Die"),
            (ds(0x07, &[1]), "Revive"),
            (ds(0x0e, &[1, 0]), "StatusUpdate"),
            (ds(0x12, &[1, 0]), "DeleteObject"),
            (ds(0x28, &[1, 0, 0, 0, 0]), "TeleportToLocation"),
            (ds(0x2d, &[1, 0]), "SocialAction"),
            (ds(0x2e, &[1, 0, 0]), "ChangeMoveType"),
            (ds(0x2f, &[1, 0, 0, 0, 0]), "ChangeWaitType"),
            (ds(0x47, &[1, 0, 0, 0, 0]), "StopMove"),
            (ds(0x61, &[1, 0, 0, 0, 0]), "ValidateLocation"),
            (ds(0x64, &[1, 0]), "SystemMessage"),
            (ds(0x29, &[1, 2, 0, 0, 0]), "TargetSelected"),
            (ds(0x2a, &[1, 0, 0, 0]), "TargetUnselected"),
            (ds(0x60, &[1, 2, 40, 0, 0, 0]), "MoveToPawn"),
            (ds(0xce, &[1, 0, 0, 0, 0]), "RelationChanged"),
            (ds(0x48, &[1, 2, 3, 1, 0, 0, 0, 0, 0]), "MagicSkillUse"),
            (ds(0x76, &[1, 3, 1, 0]), "MagicSkillLaunched"),
            (ds(0x49, &[1]), "MagicSkillCanceled"),
            (ds(0x0b, &[1, 57, 0, 0, 0, 1, 1, 0]), "SpawnItem"),
            (ds(0xf3, &[0, 0, 0, 0, 0]), "EtcStatusUpdate"),
            (vec![0x7f, 0, 0], "MagicEffectIcons"),
            (ds(0xcd, &[1, 2, 3, 4, 5, 6]), "PledgeStatusChanged"),
            (vec![0xe7, 0, 1, 0, 0, 0, 0, 0], "SendMacroList"),
            (vec![0x1b, 0, 0, 0, 0], "ItemList"),
            (vec![0x27, 0, 0], "InventoryUpdate"),
            (ds(0x4b, &[1, 2, 3]), "EquipUpdate"),
            (ds(0x45, &[1, 1, 0, 1001, 1]), "ShortCutInit"),
            (ds(0x44, &[1, 0, 1001, 1]), "ShortCutRegister"),
            (ds(0x46, &[0]), "ShortCutDelete"),
            (ds(0x58, &[1, 0, 1, 194]), "SkillList"),
            (vec![0x0f, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0], "NpcHtmlMessage"),
            (vec![0xa6, 1, 0, 0, 0, 0, 0], "MyTargetSelected"),
            (ds(0xc1, &[0]), "SkillCoolTime"),
            (vec![0x25], "ActionFailed"),
        ] {
            let event = decode_world(&body).expect("opcode do mundo").expect("corpo valido");
            assert!(format!("{event:?}").starts_with(expect), "{expect}: {event:?}");
        }
        let mut w = Writer::new();
        w.write_c(0x02);
        w.write_d(1);
        w.write_d(0);
        w.write_d(1_000_001);
        w.write_s("x");
        assert!(matches!(decode_world(&w.into_bytes()), Some(Ok(WorldEvent::NpcSay(_)))));
        assert!(matches!(decode_world(&[0x8d]), Some(Err(_))));
        assert!(decode_world(&[]).is_none());
        assert!(matches!(decode_world(&[0x16, 1]), Some(Err(_))));
        assert!(matches!(decode_world(&[0x03, 1]), Some(Err(_))));
        assert!(matches!(decode_world(&[0x7f, 1]), Some(Err(_))));
        assert!(matches!(decode_world(&ds(0x60, &[1, 2, 40, 0, 0])), Some(Err(_))));
        assert!(matches!(decode_world(&[0x1b, 0]), Some(Err(_))));
        assert!(matches!(decode_world(&[0x0f, 1, 0, 0, 0]), Some(Err(_))));
        assert!(matches!(decode_world(&[0x25, 0]), Some(Ok(WorldEvent::ActionFailed(_)))));
    }
}
