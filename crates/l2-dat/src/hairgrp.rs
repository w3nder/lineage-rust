use crate::{DatReader, Error};

pub const ROWS: usize = 15;
pub const STYLES: usize = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HairRow {
    pub ah: [i32; STYLES],
    pub bh: [i32; STYLES],
}

impl HairRow {
    pub const NONE: Self = Self { ah: [-1; STYLES], bh: [-1; STYLES] };

    pub fn ah(&self, style: i32) -> Option<i32> {
        index(&self.ah, style)
    }

    pub fn bh(&self, style: i32) -> Option<i32> {
        index(&self.bh, style)
    }
}

fn index(table: &[i32; STYLES], style: i32) -> Option<i32> {
    let value = *table.get(usize::try_from(style).ok()?)?;
    (value >= 0).then_some(value)
}

pub fn read(body: &[u8]) -> Result<Vec<HairRow>, Error> {
    let mut reader = DatReader::new(body);
    let mut rows = Vec::with_capacity(ROWS);
    while rows.len() < ROWS && reader.remaining() > 0 {
        rows.push(record(&mut reader)?);
    }
    Ok(rows)
}

pub(crate) fn record(r: &mut DatReader<'_>) -> Result<HairRow, Error> {
    let mut row = HairRow::NONE;
    for style in 0..STYLES {
        row.ah[style] = r.i32()?;
        row.bh[style] = r.i32()?;
    }
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hairgrp_rows_interleave_ah_and_bh_per_style() {
        let rows = read(include_bytes!("../tests/fixtures/hairgrp-head.bin")).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].ah, [0, 1, 2, -1, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]);
        assert_eq!(rows[0].bh, [0, 1, 2, 3, 4, -1, -1, -1, -1, 9, -1, -1, -1, -1, -1]);
        assert_eq!(rows[1].ah, [0, 1, 2, 3, -1, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1]);
        assert_eq!(rows[1].bh, [0, 1, 2, 3, 4, 5, 6, -1, -1, 9, -1, -1, -1, -1, -1]);
    }

    #[test]
    fn style_lookup_hides_missing_and_out_of_range_indices() {
        let row = read(include_bytes!("../tests/fixtures/hairgrp-head.bin")).unwrap()[0];
        assert_eq!(row.ah(0), Some(0));
        assert_eq!(row.bh(3), Some(3));
        assert_eq!(row.ah(3), None);
        assert_eq!(row.ah(15), None);
        assert_eq!(row.ah(-1), None);
        assert_eq!(HairRow::NONE.ah(0), None);
        assert_eq!(HairRow::NONE.bh(14), None);
    }

    #[test]
    fn truncated_row_is_an_error() {
        let body = include_bytes!("../tests/fixtures/hairgrp-head.bin");
        assert!(read(&body[..body.len() - 3]).is_err());
        assert!(read(&[]).unwrap().is_empty());
    }
}
