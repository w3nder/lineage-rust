use super::{counted, read_d_or_zero};
use crate::wire::{Reader, Result, Writer};

pub const OP_CHAR_TEMPLATES: u8 = 0x17;
pub const OP_CHAR_CREATE_OK: u8 = 0x19;
pub const OP_CHAR_CREATE_FAIL: u8 = 0x1a;
pub const OP_CHAR_DELETE_OK: u8 = 0x23;
pub const OP_CHAR_DELETE_FAIL: u8 = 0x24;
pub const OP_SERVER_CLOSE: u8 = 0x26;
pub const OP_LEAVE_WORLD: u8 = 0x7e;
pub const OP_GAME_GUARD_QUERY: u8 = 0xf9;

pub const OP_CHARACTER_CREATE: u8 = 0x0b;
pub const OP_CHARACTER_DELETE: u8 = 0x0c;
pub const OP_NEW_CHARACTER: u8 = 0x0e;
pub const OP_CHARACTER_RESTORE: u8 = 0x62;
pub const OP_GAME_GUARD_REPLY: u8 = 0xca;

pub const CHAR_CREATE_FAIL_INVALID_NAME: i32 = 0x02;
pub const CHAR_CREATE_FAIL_NAME_EXISTS: i32 = 0x03;
pub const CHAR_CREATE_FAIL_TOO_MANY: i32 = 0x04;
pub const CHAR_DELETE_FAIL_PLEDGE_MEMBER: i32 = 0x02;
pub const CHAR_DELETE_FAIL_PLEDGE_MASTER: i32 = 0x03;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharTemplate {
    pub race: i32,
    pub class_id: i32,
    pub str_: i32,
    pub dex: i32,
    pub con: i32,
    pub int_: i32,
    pub wit: i32,
    pub men: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharTemplates {
    pub templates: Vec<CharTemplate>,
}

pub fn read_char_templates(body: &[u8]) -> Result<CharTemplates> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let declared = r.read_d()?;
    let count = counted(&r, declared, 20 * 4)?;
    let mut templates = Vec::with_capacity(count);
    for _ in 0..count {
        let race = r.read_d()?;
        let class_id = r.read_d()?;
        let mut stats = [0i32; 6];
        for stat in &mut stats {
            r.read_d()?;
            *stat = r.read_d()?;
            r.read_d()?;
        }
        let [str_, dex, con, int_, wit, men] = stats;
        templates.push(CharTemplate { race, class_id, str_, dex, con, int_, wit, men });
    }
    Ok(CharTemplates { templates })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharCreateOk;

pub fn read_char_create_ok(body: &[u8]) -> Result<CharCreateOk> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(CharCreateOk)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharCreateFail {
    pub reason: i32,
}

pub fn read_char_create_fail(body: &[u8]) -> Result<CharCreateFail> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let reason = r.read_d()?;
    Ok(CharCreateFail { reason })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharDeleteOk;

pub fn read_char_delete_ok(body: &[u8]) -> Result<CharDeleteOk> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(CharDeleteOk)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharDeleteFail {
    pub reason: i32,
}

pub fn read_char_delete_fail(body: &[u8]) -> Result<CharDeleteFail> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let reason = r.read_d()?;
    Ok(CharDeleteFail { reason })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerClose;

pub fn read_server_close(body: &[u8]) -> Result<ServerClose> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(ServerClose)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveWorld;

pub fn read_leave_world(body: &[u8]) -> Result<LeaveWorld> {
    let mut r = Reader::new(body);
    r.read_c()?;
    Ok(LeaveWorld)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameGuardQuery {
    pub keys: [i32; 4],
}

pub fn read_game_guard_query(body: &[u8]) -> Result<GameGuardQuery> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let mut keys = [0i32; 4];
    for key in &mut keys {
        *key = read_d_or_zero(&mut r)?;
    }
    Ok(GameGuardQuery { keys })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCharacter {
    pub name: String,
    pub race: i32,
    pub sex: i32,
    pub class_id: i32,
    pub int_: i32,
    pub str_: i32,
    pub con: i32,
    pub men: i32,
    pub dex: i32,
    pub wit: i32,
    pub hair_style: i32,
    pub hair_color: i32,
    pub face: i32,
}

pub fn character_create(c: &NewCharacter) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_CHARACTER_CREATE);
    w.write_s(&c.name);
    for v in [c.race, c.sex, c.class_id, c.int_, c.str_, c.con, c.men, c.dex, c.wit, c.hair_style, c.hair_color, c.face]
    {
        w.write_d(v);
    }
    w.into_bytes()
}

pub fn character_delete(slot: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_CHARACTER_DELETE);
    w.write_d(slot);
    w.into_bytes()
}

pub fn new_character() -> Vec<u8> {
    vec![OP_NEW_CHARACTER]
}

pub fn character_restore(slot: i32) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_CHARACTER_RESTORE);
    w.write_d(slot);
    w.into_bytes()
}

pub fn game_guard_reply(keys: [i32; 4]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_GAME_GUARD_REPLY);
    for key in keys {
        w.write_d(key);
    }
    w.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ds(op: u8, values: &[i32]) -> Vec<u8> {
        let mut w = Writer::new();
        w.write_c(op);
        for v in values {
            w.write_d(*v);
        }
        w.into_bytes()
    }

    #[test]
    fn char_templates_reads_race_class_and_the_six_stats_between_the_min_max_bounds() {
        let mut values = vec![2];
        for (race, class_id, stats) in [(0, 0, [40, 30, 43, 21, 11, 25]), (1, 18, [36, 35, 36, 23, 14, 26])] {
            values.push(race);
            values.push(class_id);
            for stat in stats {
                values.extend([0x46, stat, 0x0a]);
            }
        }
        let t = read_char_templates(&ds(OP_CHAR_TEMPLATES, &values)).unwrap();
        assert_eq!(t.templates.len(), 2);
        assert_eq!(
            t.templates[1],
            CharTemplate { race: 1, class_id: 18, str_: 36, dex: 35, con: 36, int_: 23, wit: 14, men: 26 }
        );
        assert!(read_char_templates(&ds(OP_CHAR_TEMPLATES, &values[..30])).is_err());
        assert!(read_char_templates(&ds(OP_CHAR_TEMPLATES, &[0])).unwrap().templates.is_empty());
    }

    #[test]
    fn create_and_delete_results() {
        assert_eq!(read_char_create_ok(&[OP_CHAR_CREATE_OK]).unwrap(), CharCreateOk);
        assert_eq!(
            read_char_create_fail(&ds(OP_CHAR_CREATE_FAIL, &[CHAR_CREATE_FAIL_NAME_EXISTS])).unwrap(),
            CharCreateFail { reason: 3 }
        );
        assert!(read_char_create_fail(&[OP_CHAR_CREATE_FAIL, 1]).is_err());
        assert_eq!(read_char_delete_ok(&[OP_CHAR_DELETE_OK]).unwrap(), CharDeleteOk);
        assert_eq!(
            read_char_delete_fail(&ds(OP_CHAR_DELETE_FAIL, &[CHAR_DELETE_FAIL_PLEDGE_MASTER])).unwrap(),
            CharDeleteFail { reason: 3 }
        );
        assert_eq!(read_char_delete_fail(&ds(OP_CHAR_DELETE_FAIL, &[3, 0])).unwrap(), CharDeleteFail { reason: 3 });
        assert_eq!(read_server_close(&[OP_SERVER_CLOSE]).unwrap(), ServerClose);
        assert_eq!(read_leave_world(&[OP_LEAVE_WORLD]).unwrap(), LeaveWorld);
        assert!(read_leave_world(&[]).is_err());
    }

    #[test]
    fn game_guard_query_zero_fills_the_keys_lisvus_leaves_out() {
        assert_eq!(read_game_guard_query(&[OP_GAME_GUARD_QUERY]).unwrap(), GameGuardQuery { keys: [0; 4] });
        assert_eq!(read_game_guard_query(&ds(OP_GAME_GUARD_QUERY, &[1, 2, 3, 4])).unwrap().keys, [1, 2, 3, 4]);
        assert_eq!(read_game_guard_query(&ds(OP_GAME_GUARD_QUERY, &[1, 2, 3, 4, 5])).unwrap().keys, [1, 2, 3, 4]);
    }

    #[test]
    fn lobby_encoders_follow_the_engine_formats() {
        let c = NewCharacter {
            name: "Luzal".into(),
            race: 1,
            sex: 0,
            class_id: 18,
            int_: 23,
            str_: 36,
            con: 36,
            men: 26,
            dex: 35,
            wit: 14,
            hair_style: 2,
            hair_color: 1,
            face: 0,
        };
        let body = character_create(&c);
        assert_eq!(body[0], OP_CHARACTER_CREATE);
        assert_eq!(body.len(), 1 + 12 + 12 * 4);
        let ints: Vec<i32> = body[13..].chunks(4).map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
        assert_eq!(ints, vec![1, 0, 18, 23, 36, 36, 26, 35, 14, 2, 1, 0]);
        assert_eq!(character_delete(2), [OP_CHARACTER_DELETE, 2, 0, 0, 0]);
        assert_eq!(character_restore(2), [OP_CHARACTER_RESTORE, 2, 0, 0, 0]);
        assert_eq!(new_character(), [OP_NEW_CHARACTER]);
        assert_eq!(game_guard_reply([1, 2, 3, 4]), ds(OP_GAME_GUARD_REPLY, &[1, 2, 3, 4]));
    }
}
