use crate::{DatReader, Error};

pub const SLOTS: usize = 3;
pub const RACES: usize = 15;

#[derive(Debug, Clone, PartialEq)]
pub struct Sound {
    pub name: String,
    pub volume: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectSlot {
    pub spell: Sound,
    pub shot: Sound,
    pub explosion: Sound,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkillSound {
    pub id: i32,
    pub level: i32,
    pub slots: [EffectSlot; SLOTS],
    pub cast: [String; RACES],
    pub magic: [String; RACES],
    pub cast_volume: f32,
    pub cast_radius: f32,
}

pub fn find(rows: &[SkillSound], id: i32, level: i32) -> Option<&SkillSound> {
    let mut fallback = None;
    for row in rows.iter().rev().filter(|row| row.id == id) {
        if row.level == level {
            return Some(row);
        }
        if row.level == 1 {
            fallback = Some(row);
        }
    }
    fallback
}

pub fn read(body: &[u8]) -> Result<Vec<SkillSound>, Error> {
    read_with_tail(body).map(|(rows, _)| rows)
}

pub fn read_with_tail(body: &[u8]) -> Result<(Vec<SkillSound>, &[u8]), Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    let rows = (0..count).map(|_| record(&mut reader)).collect::<Result<Vec<_>, _>>()?;
    Ok((rows, &body[body.len() - reader.remaining()..]))
}

fn record(r: &mut DatReader<'_>) -> Result<SkillSound, Error> {
    let id = r.i32()?;
    let level = r.i32()?;
    let slots = [slot(r)?, slot(r)?, slot(r)?];
    let cast = names(r)?;
    let magic = names(r)?;
    let cast_volume = r.f32()?;
    let cast_radius = r.f32()?;
    Ok(SkillSound { id, level, slots, cast, magic, cast_volume, cast_radius })
}

fn slot(r: &mut DatReader<'_>) -> Result<EffectSlot, Error> {
    let spell = r.unicode()?;
    let shot = r.unicode()?;
    let explosion = r.unicode()?;
    let spell = Sound { name: spell, volume: r.f32()?, radius: r.f32()? };
    let shot = Sound { name: shot, volume: r.f32()?, radius: r.f32()? };
    let explosion = Sound { name: explosion, volume: r.f32()?, radius: r.f32()? };
    Ok(EffectSlot { spell, shot, explosion })
}

fn names(r: &mut DatReader<'_>) -> Result<[String; RACES], Error> {
    let mut out: [String; RACES] = Default::default();
    for name in &mut out {
        *name = r.unicode()?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unicode(s: &str) -> Vec<u8> {
        let mut out = (s.len() as u32 * 2).to_le_bytes().to_vec();
        out.extend(s.encode_utf16().flat_map(u16::to_le_bytes));
        out
    }

    fn floats(body: &mut Vec<u8>, values: [f32; 2]) {
        for v in values {
            body.extend(v.to_le_bytes());
        }
    }

    #[test]
    fn record_reads_slots_race_names_and_cast_volume_and_leaves_the_footer() {
        let mut body = 1u32.to_le_bytes().to_vec();
        body.extend(2i32.to_le_bytes());
        body.extend(1i32.to_le_bytes());
        body.extend(unicode("SkillSound.sleep_cast"));
        body.extend(unicode("SkillSound.sleep_shot"));
        body.extend(unicode(""));
        floats(&mut body, [250.0, 50.0]);
        floats(&mut body, [250.0, 100.0]);
        floats(&mut body, [0.0, 0.0]);
        for _ in 1..SLOTS {
            for _ in 0..3 {
                body.extend(unicode(""));
            }
            for _ in 0..6 {
                body.extend(0f32.to_le_bytes());
            }
        }
        for group in ["sub", "throw"] {
            for race in 0..RACES {
                if race == 0 {
                    body.extend(unicode(&format!("chrsound.m_hfighter_{group}")));
                } else {
                    body.extend(unicode(""));
                }
            }
        }
        floats(&mut body, [250.0, 50.0]);
        body.extend(b"\x0cSafePackage\x00");
        let (rows, tail) = read_with_tail(&body).unwrap();
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!((row.id, row.level), (2, 1));
        assert_eq!(row.slots[0].spell, Sound { name: "SkillSound.sleep_cast".into(), volume: 250.0, radius: 50.0 });
        assert_eq!(row.slots[0].shot.name, "SkillSound.sleep_shot");
        assert_eq!((row.slots[0].shot.volume, row.slots[0].shot.radius), (250.0, 100.0));
        assert!(row.slots[0].explosion.name.is_empty());
        assert!(row.slots[1].spell.name.is_empty());
        assert_eq!(row.cast[0], "chrsound.m_hfighter_sub");
        assert_eq!(row.magic[0], "chrsound.m_hfighter_throw");
        assert!(row.cast[1..].iter().all(String::is_empty));
        assert_eq!((row.cast_volume, row.cast_radius), (250.0, 50.0));
        assert_eq!(tail, b"\x0cSafePackage\x00");
        assert!(read(&body[..40]).is_err());
    }

    #[test]
    fn find_takes_the_last_exact_match_and_falls_back_to_the_first_level_one_of_the_file() {
        let row = |id: i32, level: i32, tag: &str| SkillSound {
            id,
            level,
            slots: std::array::from_fn(|_| EffectSlot {
                spell: Sound { name: tag.into(), volume: 250.0, radius: 50.0 },
                shot: Sound { name: String::new(), volume: 0.0, radius: 0.0 },
                explosion: Sound { name: String::new(), volume: 0.0, radius: 0.0 },
            }),
            cast: std::array::from_fn(|_| String::new()),
            magic: std::array::from_fn(|_| String::new()),
            cast_volume: 250.0,
            cast_radius: 50.0,
        };
        let rows = vec![row(4180, 1, "first"), row(7, 1, "seven"), row(4180, 1, "second"), row(9, 1, "nine1"), row(9, 3, "nine3")];
        assert_eq!(find(&rows, 4180, 1).unwrap().slots[0].spell.name, "second");
        assert_eq!(find(&rows, 4180, 5).unwrap().slots[0].spell.name, "first");
        assert_eq!(find(&rows, 9, 3).unwrap().slots[0].spell.name, "nine3");
        assert_eq!(find(&rows, 9, 2).unwrap().slots[0].spell.name, "nine1");
        assert!(find(&rows, 100, 1).is_none());
    }
}
