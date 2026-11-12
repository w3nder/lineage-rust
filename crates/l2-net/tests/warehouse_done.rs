use l2_net::game::packets::{OP_WAREHOUSE_DONE, WarehouseDone, read_warehouse_done};

#[test]
fn receive_warehouse_done_104037b0_reads_one_d_and_ignores_the_tail() {
    assert_eq!(OP_WAREHOUSE_DONE, 0x43);
    for value in [0i32, 1, 2, -1, i32::MIN, i32::MAX] {
        let mut body = vec![0x43];
        body.extend_from_slice(&value.to_le_bytes());
        let expected = WarehouseDone { success: value != 0 };
        assert_eq!(read_warehouse_done(&body).unwrap(), expected);
        for len in 0..body.len() {
            assert!(read_warehouse_done(&body[..len]).is_err());
        }
        body.extend_from_slice(&[0xff; 17]);
        assert_eq!(read_warehouse_done(&body).unwrap(), expected);
    }
}
