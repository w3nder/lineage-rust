use std::fmt::Debug;

use l2_net::game::packets;
use l2_net::wire::Result;

fn accepts_tail<T: Debug + PartialEq>(read: impl Fn(&[u8]) -> Result<T>, body: Vec<u8>) {
    let expected = read(&body).unwrap();
    for tail in [&[0xff][..], &[0xde, 0xad, 0xbe, 0xef][..], &[0x7f; 17][..]] {
        let mut padded = body.clone();
        padded.extend_from_slice(tail);
        assert_eq!(read(&padded).unwrap(), expected);
    }
}

macro_rules! fixture {
    ($reader:ident, $size:expr) => {
        #[test]
        fn $reader() {
            accepts_tail(packets::$reader, vec![0; $size]);
        }
    };
}

fixture!(read_char_templates, 5);
fixture!(read_char_create_fail, 5);
fixture!(read_char_delete_fail, 5);
fixture!(read_game_guard_query, 17);
fixture!(read_castle_siege_info, 35);
fixture!(read_castle_siege_attacker_list, 25);
fixture!(read_castle_siege_defender_list, 25);
fixture!(read_recipe_book_item_list, 13);
fixture!(read_recipe_item_make_info, 21);
fixture!(read_recipe_shop_manage_list, 21);
fixture!(read_recipe_shop_sell_list, 21);
fixture!(read_recipe_shop_item_info, 25);
fixture!(read_recipe_shop_msg, 7);
fixture!(read_show_calculator, 5);
fixture!(read_mon_race_info, 13);
fixture!(read_show_xmas_seal, 5);
fixture!(read_ssq_status, 3);
fixture!(read_ssq_info, 3);
fixture!(read_sell_list, 11);
fixture!(read_buy_list, 11);
fixture!(read_trade_start, 7);
fixture!(read_trade_own_add, 3);
fixture!(read_trade_other_add, 3);
fixture!(read_trade_done, 5);
fixture!(read_trade_update, 3);
fixture!(read_trade_request, 5);
fixture!(read_join_pledge, 5);
fixture!(read_join_party, 5);
fixture!(read_warehouse_deposit_list, 9);
fixture!(read_warehouse_withdraw_list, 9);
fixture!(read_pledge_show_member_list_all, 59);
fixture!(read_pledge_show_member_list_update, 23);
fixture!(read_pledge_show_member_list_add, 23);
fixture!(read_pledge_show_member_list_delete, 3);
fixture!(read_choose_inventory_item, 5);
fixture!(read_quest_list, 5);
fixture!(read_enchant_result, 5);
fixture!(read_pledge_show_info_update, 51);
fixture!(read_acquire_skill_list, 9);
fixture!(read_acquire_skill_info, 21);
fixture!(read_party_match_list, 9);
fixture!(read_party_match_detail, 27);
fixture!(read_private_store_manage_list_sell, 21);
fixture!(read_private_store_list_sell, 17);
fixture!(read_private_store_msg_sell, 7);
fixture!(read_private_store_msg_buy, 7);
fixture!(read_private_store_manage_list_buy, 17);
fixture!(read_private_store_list_buy, 13);
fixture!(read_show_minimap, 6);
fixture!(read_tutorial_show_html, 3);
fixture!(read_tutorial_show_question_mark, 5);
fixture!(read_tutorial_enable_client_event, 5);
fixture!(read_pet_item_list, 3);
fixture!(read_package_to_list, 5);
fixture!(read_package_sendable_list, 13);
fixture!(read_multi_sell_list, 21);
fixture!(read_show_town_map, 11);
fixture!(read_henna_equip_list, 13);
fixture!(read_henna_item_info, 55);
fixture!(read_henna_info, 15);
fixture!(read_henna_unequip_list, 13);
fixture!(read_henna_unequip_info, 55);
fixture!(read_buy_list_seed, 11);
fixture!(read_radar_control, 21);
fixture!(read_shop_preview_list, 15);
fixture!(read_shop_preview_info, 65);
fixture!(read_friend_recv_msg, 11);
fixture!(read_attack, 28);
fixture!(read_drop_item, 37);
fixture!(read_get_item, 21);
fixture!(read_auto_attack_start, 5);
fixture!(read_auto_attack_stop, 5);
fixture!(read_ask_join_pledge, 7);
fixture!(read_ask_join_party, 7);
fixture!(read_door_info, 9);
fixture!(read_door_status_update, 29);
fixture!(read_party_small_window_all, 13);
fixture!(read_party_small_window_add, 55);
fixture!(read_party_small_window_delete, 7);
fixture!(read_party_small_window_update, 39);
fixture!(read_vehicle_info, 21);
fixture!(read_vehicle_departure, 25);
fixture!(read_get_on_vehicle, 21);
fixture!(read_get_off_vehicle, 21);
fixture!(read_start_rotating, 17);
fixture!(read_setup_gauge, 13);
fixture!(read_move_to_location_in_vehicle, 33);
fixture!(read_stop_move_in_vehicle, 25);
fixture!(read_ride, 17);
fixture!(read_server_object_info, 79);
fixture!(read_play_sound, 31);
fixture!(read_static_object, 9);
fixture!(read_party_member_position, 5);
fixture!(read_ask_join_alliance, 11);
fixture!(read_pet_status_show, 5);
fixture!(read_pet_info, 252);
fixture!(read_pet_status_update, 63);
fixture!(read_pet_delete, 9);
fixture!(read_vehicle_start, 9);
fixture!(read_earthquake, 25);
fixture!(read_special_camera, 41);
fixture!(read_nickname_changed, 7);
fixture!(read_set_summon_remain_time, 9);
fixture!(read_dice, 25);
fixture!(read_chair_sit, 9);
fixture!(read_party_spelled, 13);
fixture!(read_short_buff_status_update, 13);

#[test]
fn read_show_board() {
    let mut body = vec![0; 20];
    body[1] = 1;
    accepts_tail(packets::read_show_board, body);
}

#[test]
fn tails_do_not_replace_missing_fields_or_bypass_string_and_count_checks() {
    use l2_net::wire::WireError;

    for length in 0..5 {
        assert_eq!(packets::read_trade_done(&vec![0; length]), Err(WireError::Short));
    }
    let mut buy = vec![0; 11];
    buy[0] = packets::OP_BUY_LIST;
    buy[9] = 1;
    buy.extend_from_slice(&[0xff; 3]);
    assert_eq!(packets::read_buy_list(&buy), Err(WireError::Short));
    let invalid_utf16 = [packets::OP_TUTORIAL_SHOW_HTML, 0, 0xd8, 0, 0, 0xde, 0xad];
    assert_eq!(packets::read_tutorial_show_html(&invalid_utf16), Err(WireError::Utf16));
    let missing_terminator = [packets::OP_TUTORIAL_SHOW_HTML, b'A', 0];
    assert_eq!(packets::read_tutorial_show_html(&missing_terminator), Err(WireError::Short));
}

#[test]
fn game_packet_readers_do_not_require_end_of_transport_frame() {
    for source in [
        include_str!("../src/game/packets.rs"),
        include_str!("../src/game/packets/lobby.rs"),
        include_str!("../src/game/packets/rare.rs"),
        include_str!("../src/game/packets/windows.rs"),
        include_str!("../src/game/packets/world.rs"),
    ] {
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(!production.contains(".finish()"));
    }
}
