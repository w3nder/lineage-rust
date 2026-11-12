use crate::Error;
use crate::reader::DatReader;

pub const SELECT_SLOTS: usize = 7;
pub const STAGE_SLOTS: usize = 18;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Spot {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Logongrp {
    pub slots: [Spot; SELECT_SLOTS],
    pub center: Spot,
    pub stage: [Spot; STAGE_SLOTS],
}

impl Logongrp {
    pub fn stage_index(race: i32, sex: i32, class_id: i32) -> Option<usize> {
        let pair = match (race, class_id) {
            (0, 0) => 0,
            (0, 10) => 2,
            (1, 18) => 4,
            (1, 25) => 6,
            (2, 31) => 8,
            (2, 38) => 10,
            (3, 44) => 12,
            (3, 49) => 14,
            (4, 53) => 16,
            _ => return None,
        };
        Some(pair + usize::from(sex != 0))
    }
}

pub const STAGE_TEMPLATES: [(i32, i32, i32); STAGE_SLOTS] = [
    (0, 0, 0),
    (0, 1, 0),
    (0, 0, 10),
    (0, 1, 10),
    (1, 0, 18),
    (1, 1, 18),
    (1, 0, 25),
    (1, 1, 25),
    (2, 0, 31),
    (2, 1, 31),
    (2, 0, 38),
    (2, 1, 38),
    (3, 0, 44),
    (3, 1, 44),
    (3, 0, 49),
    (3, 1, 49),
    (4, 0, 53),
    (4, 1, 53),
];

fn spot(r: &mut DatReader<'_>) -> Result<Spot, Error> {
    Ok(Spot { x: r.f32()?, y: r.f32()?, z: r.f32()?, yaw: r.f32()? })
}

pub fn read(body: &[u8]) -> Result<Logongrp, Error> {
    let mut r = DatReader::new(body);
    let mut slots = [Spot::default(); SELECT_SLOTS];
    for slot in &mut slots {
        *slot = spot(&mut r)?;
    }
    let center = spot(&mut r)?;
    let mut stage = [Spot::default(); STAGE_SLOTS];
    for slot in &mut stage {
        *slot = spot(&mut r)?;
    }
    Ok(Logongrp { slots, center, stage })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(records: usize) -> Vec<u8> {
        let mut body = Vec::new();
        for i in 0..records {
            for v in [-16622.0 - i as f32, 238671.0, -6256.0, -26624.0 + i as f32] {
                body.extend_from_slice(&v.to_le_bytes());
            }
        }
        body.extend_from_slice(&[12, b'S', b'a', b'f', b'e', b'P', b'a', b'c', b'k', b'a', b'g', b'e', 0]);
        body
    }

    #[test]
    fn reads_seven_slots_the_center_and_eighteen_stage_spots_in_the_dat_order() {
        let table = read(&body(26)).unwrap();
        assert_eq!(table.slots[0], Spot { x: -16622.0, y: 238671.0, z: -6256.0, yaw: -26624.0 });
        assert_eq!(table.slots[6].x, -16628.0);
        assert_eq!(table.center.x, -16629.0);
        assert_eq!(table.stage[0].x, -16630.0);
        assert_eq!(table.stage[17].yaw, -26624.0 + 25.0);
        assert!(read(&body(20)).is_err());
    }

    #[test]
    fn stage_index_follows_the_engine_table_of_default_characters() {
        for (index, (race, sex, class_id)) in STAGE_TEMPLATES.iter().enumerate() {
            assert_eq!(Logongrp::stage_index(*race, *sex, *class_id), Some(index));
        }
        assert_eq!(Logongrp::stage_index(4, 0, 49), None);
        assert_eq!(Logongrp::stage_index(0, 1, 1), None);
    }
}
