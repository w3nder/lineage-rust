use crate::hairgrp::{HairRow, ROWS};
use crate::{DatReader, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelmetHair {
    pub helmet_index: u32,
    pub rows: Vec<HairRow>,
}

pub fn read(body: &[u8]) -> Result<Vec<HelmetHair>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count)
        .map(|_| {
            let helmet_index = reader.u32()?;
            let rows = (0..ROWS).map(|_| crate::hairgrp::record(&mut reader)).collect::<Result<_, _>>()?;
            Ok(HelmetHair { helmet_index, rows })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helmetgrp_is_keyed_by_helmet_index_with_a_hairgrp_table_each() {
        let helmets = read(include_bytes!("../tests/fixtures/helmetgrp-head.bin")).unwrap();
        assert_eq!(helmets.len(), 2);
        assert_eq!(helmets[0].helmet_index, 1);
        assert_eq!(helmets[1].helmet_index, 2);
        assert_eq!(helmets[0].rows.len(), ROWS);
        assert_eq!(helmets[0].rows[0].ah, [0, 1, 2, -1, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]);
        assert_eq!(helmets[0].rows[0].bh, [0, 1, 2, 3, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]);
        assert_eq!(helmets[0].rows[10].ah, [-1; 15]);
        assert_eq!(helmets[1].rows[3].bh, [0, 1, 2, 3, 4, 5, 6, -1, -1, -1, -1, -1, -1, -1, -1]);
    }

    #[test]
    fn hairaccessarygrp_shares_the_layout_and_starts_at_index_zero() {
        let accessories = read(include_bytes!("../tests/fixtures/hairaccessarygrp-head.bin")).unwrap();
        assert_eq!(accessories.iter().map(|a| a.helmet_index).collect::<Vec<_>>(), [0, 1, 2]);
        assert_eq!(accessories[0].rows[0].ah, [0, 1, 2, -1, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]);
        assert_eq!(accessories[0].rows[0].bh, [0, 1, 2, 3, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]);
    }

    #[test]
    fn truncated_table_is_an_error() {
        let body = include_bytes!("../tests/fixtures/helmetgrp-head.bin");
        assert!(read(&body[..body.len() - 3]).is_err());
        assert!(read(&[0, 0, 0, 0]).unwrap().is_empty());
    }
}
