use crate::hairgrp::STYLES;
use crate::{DatReader, Error};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct HairAccessoryLoc {
    pub mesh: String,
    pub offsets: [[f32; 3]; STYLES],
    pub rotations: [[i32; 3]; STYLES],
}

impl HairAccessoryLoc {
    pub fn offset(&self, style: i32) -> [f32; 3] {
        usize::try_from(style).ok().and_then(|s| self.offsets.get(s).copied()).unwrap_or([0.0; 3])
    }

    pub fn rotation(&self, style: i32) -> [i32; 3] {
        usize::try_from(style).ok().and_then(|s| self.rotations.get(s).copied()).unwrap_or([0; 3])
    }
}

pub fn read(body: &[u8]) -> Result<Vec<HairAccessoryLoc>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| record(&mut reader)).collect()
}

fn record(r: &mut DatReader<'_>) -> Result<HairAccessoryLoc, Error> {
    let mesh = r.unicode()?;
    let mut offsets = [[0.0; 3]; STYLES];
    let mut rotations = [[0; 3]; STYLES];
    for style in 0..STYLES {
        offsets[style] = [r.f32()?, r.f32()?, r.f32()?];
        rotations[style] = [r.i32()?, r.i32()?, r.i32()?];
    }
    Ok(HairAccessoryLoc { mesh, offsets, rotations })
}

pub fn by_mesh(rows: Vec<HairAccessoryLoc>) -> HashMap<String, HairAccessoryLoc> {
    rows.into_iter().map(|row| (row.mesh.to_ascii_lowercase(), row)).collect()
}

pub fn find<'a>(table: &'a HashMap<String, HairAccessoryLoc>, mesh: &str) -> Option<&'a HairAccessoryLoc> {
    table.get(&mesh.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_carry_an_offset_and_a_rotator_per_hair_style() {
        let rows = read(include_bytes!("../tests/fixtures/hairaccessorylocgrp-head.bin")).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].mesh, "LineageAccessory.Fdwarf_hairpin_m000_a");
        assert_eq!(rows[0].offsets[0], [3.0, -0.6, 2.15]);
        assert_eq!(rows[0].rotations[0], [16384, 32768, 0]);
        assert_eq!(rows[0].offsets[1], [3.5, -0.6, 2.1]);
        assert_eq!(rows[1].mesh, "LineageAccessory.Ffighter_hairpin_m000_a");
        assert_eq!(rows[1].offsets[0], [3.4, 0.0, 1.7]);
        let table = by_mesh(rows);
        let found = find(&table, "lineageaccessory.FDWARF_hairpin_m000_a").unwrap();
        assert_eq!(found.offset(1), [3.5, -0.6, 2.1]);
        assert_eq!(found.rotation(1), [16384, 32768, 0]);
        assert_eq!(found.offset(15), [0.0; 3]);
        assert_eq!(found.rotation(-1), [0; 3]);
        assert!(find(&table, "LineageAccessory.Felf_hairpin_m000_a").is_none());
    }

    #[test]
    fn truncated_table_is_an_error() {
        let body = include_bytes!("../tests/fixtures/hairaccessorylocgrp-head.bin");
        assert!(read(&body[..body.len() - 5]).is_err());
    }
}
