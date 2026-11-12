pub const KEY: [u8; 2] = [0xF5, 0xA1];
pub const SERVER_VALUE: i32 = 1;

pub fn ansi_name(name: &str) -> Vec<u8> {
    name.chars().map(|c| u8::try_from(u32::from(c)).unwrap_or(b'?')).collect()
}

pub fn coded_name(name: &str) -> String {
    ansi_name(name).iter().enumerate().map(|(i, byte)| (byte ^ KEY[i & 1]).to_string()).collect()
}

pub fn file_name(name: &str) -> String {
    format!("{}_{}.dat", coded_name(name), SERVER_VALUE)
}

pub fn encode(ids: &[i32]) -> Vec<u8> {
    ids.iter().flat_map(|id| id.to_le_bytes()).collect()
}

pub fn decode(bytes: &[u8]) -> Vec<i32> {
    bytes.chunks_exact(4).map(|chunk| i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])).collect()
}

pub fn restore<T: Clone>(items: &mut [T], id: impl Fn(&T) -> i32, order: &[i32]) {
    let count = items.len();
    let mut first = items.iter().take_while(|item| !order.contains(&id(item))).count();
    for (k, wanted) in order.iter().enumerate() {
        if k >= count {
            continue;
        }
        let found = (k..count).find(|&at| *wanted == id(&items[at]) && first < count);
        if let Some(found) = found {
            items.swap(first, found);
            first += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_name_is_the_decimal_xor_of_the_ansi_bytes_and_the_server_value() {
        assert_eq!(coded_name("Ab"), format!("{}{}", 0x41 ^ 0xF5, 0x62 ^ 0xA1));
        assert_eq!(coded_name("A"), "180");
        assert_eq!(file_name("A"), "180_1.dat");
        assert_eq!(coded_name(""), "");
        assert_eq!(ansi_name("Ação"), vec![0x41, 0xE7, 0xE3, 0x6F]);
    }

    #[test]
    fn the_file_is_the_item_ids_as_little_endian_integers_and_round_trips() {
        let ids = [57, 1060, -1, 0x1794];
        let bytes = encode(&ids);
        assert_eq!(bytes[..8], [57, 0, 0, 0, 0x24, 4, 0, 0]);
        assert_eq!(decode(&bytes), ids);
        assert_eq!(decode(&bytes[..bytes.len() - 1]), ids[..3]);
    }

    #[test]
    fn restoring_swaps_the_saved_ids_to_the_front_of_the_known_ones_and_leaves_new_ones_first() {
        let mut items = vec![9, 3, 1, 2, 7];
        restore(&mut items, |v| *v, &[1, 2, 3]);
        assert_eq!(items, [9, 1, 2, 3, 7]);
        let mut same = vec![5, 6, 7];
        restore(&mut same, |v| *v, &[5, 6, 7]);
        assert_eq!(same, [5, 6, 7]);
        let mut unknown = vec![5, 6];
        restore(&mut unknown, |v| *v, &[8, 9]);
        assert_eq!(unknown, [5, 6]);
        let mut reverse = vec![1, 2, 3];
        restore(&mut reverse, |v| *v, &[3, 2, 1]);
        assert_eq!(reverse, [3, 2, 1]);
    }
}
