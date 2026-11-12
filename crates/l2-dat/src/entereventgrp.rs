use crate::{DatReader, Error};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct EnterEvent {
    pub id: u32,
    pub reserved: String,
    pub sound_name: String,
    pub sound_vol: f32,
    pub sound_rad: f32,
    pub is_rise: u32,
    pub spawn_type: u32,
    pub effect_name: String,
    pub anim_name: String,
}

impl EnterEvent {
    pub fn is_enter(&self) -> bool {
        self.spawn_type != 0
    }

    pub fn is_exit(&self) -> bool {
        self.spawn_type == 0
    }
}

pub fn by_npc(rows: Vec<EnterEvent>) -> HashMap<u32, EnterEvent> {
    rows.into_iter().map(|row| (row.id, row)).collect()
}

pub fn enter_of(table: &HashMap<u32, EnterEvent>, npc_id: u32) -> Option<&EnterEvent> {
    table.get(&npc_id).filter(|row| row.is_enter())
}

pub fn exit_of(table: &HashMap<u32, EnterEvent>, npc_id: u32) -> Option<&EnterEvent> {
    table.get(&npc_id).filter(|row| row.is_exit())
}

pub fn read(body: &[u8]) -> Result<Vec<EnterEvent>, Error> {
    read_prefix(body).map(|(rows, _)| rows)
}

pub fn read_prefix(body: &[u8]) -> Result<(Vec<EnterEvent>, usize), Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    let rows = (0..count).map(|_| record(&mut reader)).collect::<Result<Vec<_>, _>>()?;
    Ok((rows, body.len() - reader.remaining()))
}

fn record(r: &mut DatReader<'_>) -> Result<EnterEvent, Error> {
    let id = r.u32()?;
    let reserved = r.ascf()?;
    let sound_name = r.ascf()?;
    let sound_vol = r.f32()?;
    let sound_rad = r.f32()?;
    let is_rise = r.u32()?;
    let spawn_type = r.u32()?;
    let effect_name = r.unicode()?;
    let anim_name = r.unicode()?;
    Ok(EnterEvent { id, reserved, sound_name, sound_vol, sound_rad, is_rise, spawn_type, effect_name, anim_name })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unicode(s: &str) -> Vec<u8> {
        let mut out = (s.len() as u32 * 2).to_le_bytes().to_vec();
        out.extend(s.encode_utf16().flat_map(u16::to_le_bytes));
        out
    }

    #[test]
    fn record_follows_the_serializer_field_order() {
        let mut body = 1u32.to_le_bytes().to_vec();
        body.extend(212u32.to_le_bytes());
        body.push(0);
        body.extend(b"\x18SkillSound4.summon_shot\x00");
        body.extend(250.0f32.to_le_bytes());
        body.extend(100.0f32.to_le_bytes());
        body.extend(1u32.to_le_bytes());
        body.extend(1u32.to_le_bytes());
        body.extend(unicode("LineageEffect.e_u011_b"));
        body.extend(unicode("atkwait"));
        let (rows, used) = read_prefix(&body).unwrap();
        assert_eq!(used, body.len());
        assert_eq!(
            rows,
            [EnterEvent {
                id: 212,
                reserved: String::new(),
                sound_name: "SkillSound4.summon_shot".into(),
                sound_vol: 250.0,
                sound_rad: 100.0,
                is_rise: 1,
                spawn_type: 1,
                effect_name: "LineageEffect.e_u011_b".into(),
                anim_name: "atkwait".into(),
            }]
        );
        assert!(read(&body[..body.len() - 1]).is_err());
    }

    #[test]
    fn spawn_type_selects_enter_rows_and_exit_rows_and_the_last_row_of_an_id_wins() {
        let row = |id: u32, spawn_type: u32, anim: &str| EnterEvent {
            id,
            reserved: String::new(),
            sound_name: String::new(),
            sound_vol: 0.0,
            sound_rad: 0.0,
            is_rise: 0,
            spawn_type,
            effect_name: String::new(),
            anim_name: anim.into(),
        };
        let table = by_npc(vec![row(1, 1, "a"), row(2, 0, "b"), row(1, 1, "c"), row(3, 7, "d")]);
        assert_eq!(table.len(), 3);
        assert_eq!(enter_of(&table, 1).map(|r| r.anim_name.as_str()), Some("c"));
        assert!(enter_of(&table, 2).is_none() && exit_of(&table, 2).is_some());
        assert!(enter_of(&table, 3).is_some() && exit_of(&table, 3).is_none());
        assert!(enter_of(&table, 9).is_none() && exit_of(&table, 9).is_none());
    }
}
