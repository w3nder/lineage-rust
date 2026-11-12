use crate::{DatReader, Error};

#[derive(Debug, Clone, PartialEq)]
pub struct QuestName {
    pub index: usize,
    pub valid: i32,
    pub id: i32,
    pub state: i32,
    pub title: String,
    pub sub_name: String,
    pub description: String,
    pub item_ids: Vec<i32>,
    pub item_nums: Vec<i32>,
    pub target_loc: [f32; 3],
    pub level: i32,
    pub quest_level: i32,
    pub quest_type: i32,
    pub target_name: String,
    pub minimap_only: i32,
    pub journal_disp: i32,
    pub unknown: i32,
    pub start_npc_id: i32,
    pub start_npc_loc: [f32; 3],
    pub requirement: String,
    pub intro: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestExtra {
    pub classes: Vec<i32>,
    pub extra_ids: Vec<i32>,
    pub extra: [i32; 4],
}

pub fn read(body: &[u8]) -> Result<Vec<QuestName>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count)
        .map(|index| record(&mut reader).map(|row| QuestName { index: index as usize, ..row }))
        .collect()
}

pub fn read_as(body: &[u8], chronicle: crate::Chronicle) -> Result<Vec<QuestName>, Error> {
    match chronicle {
        crate::Chronicle::C4 => read(body),
        crate::Chronicle::Interlude => read_interlude(body).map(|rows| rows.into_iter().map(|(quest, _)| quest).collect()),
    }
}

pub fn read_interlude(body: &[u8]) -> Result<Vec<(QuestName, QuestExtra)>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count)
        .map(|index| {
            let row = record(&mut reader)?;
            let extra = QuestExtra {
                classes: int_array(&mut reader)?,
                extra_ids: int_array(&mut reader)?,
                extra: [reader.i32()?, reader.i32()?, reader.i32()?, reader.i32()?],
            };
            Ok((QuestName { index: index as usize, ..row }, extra))
        })
        .collect()
}

fn int_array(r: &mut DatReader<'_>) -> Result<Vec<i32>, Error> {
    let count = r.count()?;
    (0..count).map(|_| r.i32()).collect()
}

fn record(r: &mut DatReader<'_>) -> Result<QuestName, Error> {
    let valid = r.i32()?;
    let id = r.i32()?;
    let state = r.i32()?;
    let title = r.ascf()?;
    let sub_name = r.ascf()?;
    let description = r.ascf()?;
    let item_ids = int_array(r)?;
    let item_nums = int_array(r)?;
    let target_loc = [r.f32()?, r.f32()?, r.f32()?];
    let level = r.i32()?;
    let quest_level = r.i32()?;
    let quest_type = r.i32()?;
    let target_name = r.ascf()?;
    let minimap_only = r.i32()?;
    let journal_disp = r.i32()?;
    let unknown = r.i32()?;
    let start_npc_id = r.i32()?;
    let start_npc_loc = [r.f32()?, r.f32()?, r.f32()?];
    let requirement = r.ascf()?;
    let intro = r.ascf()?;
    Ok(QuestName {
        index: 0,
        valid,
        id,
        state,
        title,
        sub_name,
        description,
        item_ids,
        item_nums,
        target_loc,
        level,
        quest_level,
        quest_type,
        target_name,
        minimap_only,
        journal_disp,
        unknown,
        start_npc_id,
        start_npc_loc,
        requirement,
        intro,
    })
}

pub fn by_id_state(rows: Vec<QuestName>) -> std::collections::HashMap<(i32, i32), QuestName> {
    rows.into_iter().map(|q| ((q.id, q.state), q)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ascf(s: &str) -> Vec<u8> {
        let mut out = vec![(s.len() + 1) as u8];
        out.extend(s.bytes());
        out.push(0);
        out
    }

    #[test]
    fn records_carry_id_state_title_sub_name_description_and_the_locations() {
        let mut body = 1u32.to_le_bytes().to_vec();
        for v in [1i32, 1, 2] {
            body.extend(v.to_le_bytes());
        }
        body.extend(ascf("Letters of Love"));
        body.extend(ascf("Roxxy's Kerchief"));
        body.extend(ascf("Roxxy reads the letter."));
        body.push(1);
        body.extend(1234i32.to_le_bytes());
        body.push(1);
        body.extend(2i32.to_le_bytes());
        for v in [-84000.0f32, 243000.0, -3700.0] {
            body.extend(v.to_le_bytes());
        }
        for v in [2i32, 5, 3] {
            body.extend(v.to_le_bytes());
        }
        body.extend(ascf("Darin"));
        for v in [0i32, 1, 1, 7048] {
            body.extend(v.to_le_bytes());
        }
        for v in [-84100.0f32, 243100.0, -3700.0] {
            body.extend(v.to_le_bytes());
        }
        body.extend(ascf("No Requirements"));
        body.extend(ascf("Darin, a young man."));
        let rows = read(&body).unwrap();
        assert_eq!(rows.len(), 1);
        let q = &rows[0];
        assert_eq!((q.id, q.state, q.title.as_str(), q.sub_name.as_str()), (1, 2, "Letters of Love", "Roxxy's Kerchief"));
        assert_eq!((q.item_ids.clone(), q.item_nums.clone()), (vec![1234], vec![2]));
        assert_eq!((q.level, q.quest_level, q.quest_type), (2, 5, 3));
        assert_eq!((q.target_name.as_str(), q.start_npc_id), ("Darin", 7048));
        assert_eq!(q.target_loc, [-84000.0, 243000.0, -3700.0]);
        assert_eq!(q.intro, "Darin, a young man.");
        let map = by_id_state(rows);
        assert!(map.contains_key(&(1, 2)));
    }

    fn interlude_row(state: i32, classes: &[i32], extra_ids: &[i32], extra: [i32; 4]) -> Vec<u8> {
        let mut body = Vec::new();
        for v in [1i32, 1, state] {
            body.extend(v.to_le_bytes());
        }
        body.extend(ascf("Letters of Love"));
        body.extend(ascf("Delivery"));
        body.extend(ascf("Darin."));
        body.extend([0, 0]);
        for v in [0.0f32; 3] {
            body.extend(v.to_le_bytes());
        }
        for v in [2i32, 5, 3] {
            body.extend(v.to_le_bytes());
        }
        body.extend(ascf("Roxxy"));
        for v in [0i32, 1, 1, 30048] {
            body.extend(v.to_le_bytes());
        }
        for v in [0.0f32; 3] {
            body.extend(v.to_le_bytes());
        }
        body.extend(ascf("No Requirements"));
        body.extend(ascf("Darin."));
        for list in [classes, extra_ids] {
            body.push(list.len() as u8);
            for v in list {
                body.extend(v.to_le_bytes());
            }
        }
        for v in extra {
            body.extend(v.to_le_bytes());
        }
        body
    }

    #[test]
    fn interlude_rows_end_with_two_int_arrays_and_four_ints() {
        let mut body = 2u32.to_le_bytes().to_vec();
        body.extend(interlude_row(1, &[], &[], [0, 0, 0, 30]));
        body.extend(interlude_row(2, &[0, 1, 88], &[1508, 1509], [1, 121, 0, 67]));
        let rows = read_interlude(&body).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].1, QuestExtra { classes: vec![], extra_ids: vec![], extra: [0, 0, 0, 30] });
        assert_eq!(rows[1].1, QuestExtra { classes: vec![0, 1, 88], extra_ids: vec![1508, 1509], extra: [1, 121, 0, 67] });
        assert_eq!((rows[1].0.state, rows[1].0.start_npc_id, rows[1].0.index), (2, 30048, 1));
        assert!(read_interlude(&body[..body.len() - 1]).is_err());
        assert_eq!(read_as(&body, crate::Chronicle::Interlude).unwrap().len(), 2);
    }
}
