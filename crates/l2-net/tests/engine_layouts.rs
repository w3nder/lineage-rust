use l2_net::game::{
    read_gm_view_pledge_info, read_gm_view_quest_info, read_gm_view_skill_info, read_net_ping_response, read_petition_vote,
    read_sell_list_procure,
    read_camera_mode, read_event_trigger, read_gm_henna_info, read_l2_friend, read_l2_friend_status,
    read_move_to_location_in_vehicle, read_normal_camera, read_observer_end, read_observer_start,
    read_party_small_window_delete_all, read_pledge_show_member_list_add, read_pledge_show_member_list_delete,
    read_pledge_show_member_list_delete_all, read_pledge_show_member_list_update, read_snoop,
    read_tutorial_enable_client_event, read_tutorial_show_question_mark,
};
use l2_net::wire::Writer;

enum Field {
    C(u8),
    D(i32),
    H(u16),
    S(&'static str),
}

fn body(opcode: u8, fields: &[Field]) -> Vec<u8> {
    let mut writer = Writer::new();
    writer.write_c(opcode);
    for field in fields {
        match field {
            Field::C(value) => writer.write_c(*value),
            Field::D(value) => writer.write_d(*value),
            Field::H(value) => writer.write_h(*value),
            Field::S(value) => writer.write_s(value),
        }
    }
    writer.into_bytes()
}

fn follows<T, E: std::fmt::Debug>(read: impl Fn(&[u8]) -> Result<T, E>, opcode: u8, fields: &[Field]) {
    let bytes = body(opcode, fields);
    read(&bytes).expect("o leitor aceita o corpo completo do formato do engine");
    for cut in 1..bytes.len() {
        assert!(read(&bytes[..cut]).is_err(), "o leitor aceitou {cut} de {} bytes", bytes.len());
    }
}

fn opcode_only<T, E: std::fmt::Debug>(read: impl Fn(&[u8]) -> Result<T, E>, opcode: u8) {
    read(&[opcode]).expect("handler sem campos");
}

fn ds(count: usize) -> Vec<Field> {
    (0..count).map(|index| Field::D(index as i32 + 1)).collect()
}

#[test]
fn receive_tutorial_show_question_mark_103fe500_reads_d() {
    follows(read_tutorial_show_question_mark, 0xa1, &ds(1));
}

#[test]
fn receive_tutorial_enable_client_event_103fe590_reads_d() {
    follows(read_tutorial_enable_client_event, 0xa2, &ds(1));
}

#[test]
fn receive_event_trigger_104078a0_reads_dc() {
    follows(read_event_trigger, 0xcf, &[Field::D(5), Field::C(1)]);
}

#[test]
fn receive_snoop_10407ce0_reads_dsddss() {
    follows(
        read_snoop,
        0xd5,
        &[Field::D(1), Field::S("a"), Field::D(2), Field::D(3), Field::S("b"), Field::S("c")],
    );
}

#[test]
fn receive_observer_start_104085f0_reads_five_d() {
    follows(read_observer_start, 0xdf, &ds(5));
}

#[test]
fn receive_observer_end_10408750_reads_three_d() {
    follows(read_observer_end, 0xe0, &ds(3));
}

#[test]
fn receive_gm_henna_info_10408ea0_reads_cccccc_dd_then_dd_per_symbol() {
    let mut fields: Vec<Field> = (1..=6).map(Field::C).collect();
    fields.extend([Field::D(1), Field::D(2), Field::D(10), Field::D(11), Field::D(12), Field::D(13)]);
    follows(read_gm_henna_info, 0xea, &fields);
}

#[test]
fn receive_camera_mode_10409ec0_reads_d() {
    follows(read_camera_mode, 0xf1, &ds(1));
}

#[test]
fn receive_l2_friend_1040ae90_reads_ddsdd() {
    follows(read_l2_friend, 0xfb, &[Field::D(1), Field::D(2), Field::S("Luzal"), Field::D(3), Field::D(4)]);
}

#[test]
fn receive_l2_friend_status_1040af50_reads_dsd() {
    follows(read_l2_friend_status, 0xfc, &[Field::D(1), Field::S("Luzal"), Field::D(2)]);
}

#[test]
fn receive_pledge_show_member_list_delete_10404640_reads_s() {
    follows(read_pledge_show_member_list_delete, 0x56, &[Field::S("Luzal")]);
}

#[test]
fn receive_pledge_show_member_list_update_104043e0_and_add_10404510_read_sddddd() {
    let fields = [Field::S("Luzal"), Field::D(1), Field::D(2), Field::D(3), Field::D(4), Field::D(5)];
    follows(read_pledge_show_member_list_update, 0x54, &fields);
    follows(read_pledge_show_member_list_add, 0x55, &fields);
}

#[test]
fn receive_move_to_location_in_vehicle_10418260_reads_eight_d() {
    follows(read_move_to_location_in_vehicle, 0x71, &ds(8));
}

#[test]
fn handlers_without_fields_read_only_the_opcode() {
    opcode_only(read_party_small_window_delete_all, 0x50);
    opcode_only(read_pledge_show_member_list_delete_all, 0x82);
    opcode_only(read_normal_camera, 0xc8);
}

#[test]
fn receive_net_ping_10407c50_reads_d() {
    follows(read_net_ping_response, 0xd3, &ds(1));
}

#[test]
fn receive_petition_vote_103f1ef0_has_no_fields() {
    opcode_only(read_petition_vote, 0xf6);
}

#[test]
fn receive_gm_view_skill_info_10400b50_reads_sd_then_ddd_per_skill() {
    let mut fields = vec![Field::S("gm"), Field::D(2)];
    fields.extend(ds(6));
    follows(read_gm_view_skill_info, 0x91, &fields);
    let parsed = read_gm_view_skill_info(&body(0x91, &fields)).unwrap();
    assert_eq!(parsed.entries, vec![[1, 2, 3], [4, 5, 6]]);
}

#[test]
fn receive_gm_view_quest_info_10400e60_reads_two_counted_lists() {
    let mut fields = vec![Field::S("gm"), Field::H(1)];
    fields.extend(ds(2));
    fields.push(Field::H(1));
    fields.extend(ds(4));
    follows(read_gm_view_quest_info, 0x93, &fields);
    let parsed = read_gm_view_quest_info(&body(0x93, &fields)).unwrap();
    assert_eq!((parsed.first, parsed.second), (vec![(1, 2)], vec![[1, 2, 3, 4]]));
}

#[test]
fn receive_gm_view_pledge_info_10400880_reads_details_only_with_a_pledge() {
    let mut fields = vec![Field::S("char"), Field::D(7), Field::S("clan"), Field::S("leader")];
    fields.extend(ds(9));
    fields.extend([Field::S("ally"), Field::D(1), Field::D(2), Field::D(1), Field::S("member")]);
    fields.extend(ds(5));
    follows(read_gm_view_pledge_info, 0x90, &fields);
    let parsed = read_gm_view_pledge_info(&body(0x90, &fields)).unwrap();
    assert_eq!(parsed.details.map(|d| d.name), Some("ally".to_owned()));
    assert_eq!(parsed.members.len(), 1);
    let mut empty = vec![Field::S("char"), Field::D(0), Field::S(""), Field::S("")];
    empty.push(Field::D(0));
    assert_eq!(read_gm_view_pledge_info(&body(0x90, &empty)).unwrap().details, None);
}

#[test]
fn receive_sell_list_procure_103ff600_reads_details_only_for_equipment() {
    let fields = vec![
        Field::D(1), Field::D(2), Field::H(3), Field::H(2),
        Field::H(1), Field::D(4), Field::D(5), Field::D(6), Field::H(7), Field::H(8), Field::D(9), Field::H(1), Field::H(2), Field::H(3), Field::D(100),
        Field::H(5), Field::D(4), Field::D(5), Field::D(6), Field::H(7), Field::H(8), Field::D(200),
    ];
    follows(read_sell_list_procure, 0xe9, &fields);
    let parsed = read_sell_list_procure(&body(0xe9, &fields)).unwrap();
    assert_eq!(parsed.items.iter().map(|item| (item.details.is_some(), item.price)).collect::<Vec<_>>(), [(true, 100), (false, 200)]);
}
