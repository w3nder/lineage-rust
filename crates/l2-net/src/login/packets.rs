use crate::wire::{Reader, Result, Writer};

pub const OP_INIT: u8 = 0x00;
pub const OP_LOGIN_FAIL: u8 = 0x01;
pub const OP_ACCOUNT_KICKED: u8 = 0x02;
pub const OP_LOGIN_OK: u8 = 0x03;
pub const OP_SERVER_LIST: u8 = 0x04;
pub const OP_SERVER_FAIL: u8 = 0x05;
pub const OP_PLAY_FAIL: u8 = 0x06;
pub const OP_ACCOUNT_KICKED_REASON: u8 = 0x08;
pub const OP_BLOCKED_WITH_MSG: u8 = 0x09;
pub const OP_PLAY_OK: u8 = 0x07;
pub const OP_SC_CHECK_START: u8 = 0x0a;
pub const SC_CHECK_WRONG: u8 = 31;

pub const OP_REQUEST_AUTH_LOGIN: u8 = 0x00;
pub const OP_REQUEST_SERVER_LOGIN: u8 = 0x02;
pub const OP_REQUEST_SERVER_LIST: u8 = 0x05;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameServer {
    pub id: u8,
    pub ip: [u8; 4],
    pub port: u16,
    pub players: u16,
    pub max_players: u16,
    pub online: bool,
    pub extra: ServerExtra,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ServerExtra {
    pub age: u8,
    pub pvp: u8,
    pub status: u8,
    pub flags: u32,
    pub brackets: u8,
    pub last: bool,
}

impl GameServer {
    pub fn address(&self) -> String {
        let [a, b, c, d] = self.ip;
        format!("{a}.{b}.{c}.{d}:{}", self.port)
    }
}

pub fn init_is_plaintext(body: &[u8]) -> bool {
    body.len() == 9 && body[0] == OP_INIT
}

pub fn read_key_pair(body: &[u8]) -> Result<[u32; 2]> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let a = r.read_d()? as u32;
    let b = r.read_d()? as u32;
    Ok([a, b])
}

pub fn read_login_notice(body: &[u8]) -> super::flow::LoginNotice {
    let mut r = Reader::new(body);
    let mut notice = super::flow::LoginNotice::default();
    if r.read_c().is_err() {
        return notice;
    }
    for index in 0..5 {
        let Ok(value) = r.read_d() else { return notice };
        if index == 4 {
            notice.flags = value as u32;
        }
    }
    for _ in 0..3 {
        if r.read_d().is_err() {
            return notice;
        }
    }
    if let Ok(bytes) = r.read_b(16) {
        notice.forbidden.copy_from_slice(&bytes[..16]);
    }
    notice
}

pub fn read_server_list(body: &[u8]) -> Result<Vec<GameServer>> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let count = r.read_c()? as usize;
    let last = r.read_c()?;
    let mut servers = Vec::with_capacity(count);
    for _ in 0..count {
        let id = r.read_c()?;
        let ip_bytes = r.read_b(4)?;
        let ip = [ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]];
        let port = r.read_d()? as u16;
        let age = r.read_c()?;
        let pvp = r.read_c()?;
        let players = r.read_h()?;
        let max_players = r.read_h()?;
        let status = r.read_c()?;
        let flags = r.read_d()? as u32;
        let brackets = r.read_c()?;
        let extra = ServerExtra { age, pvp, status, flags, brackets, last: id == last };
        servers.push(GameServer { id, ip, port, players, max_players, online: status != 0, extra });
    }
    Ok(servers)
}

pub fn request_auth_login(user: &[u8], password: &[u8]) -> Vec<u8> {
    let mut body = vec![0u8; 32];
    body[0] = OP_REQUEST_AUTH_LOGIN;
    body[1..1 + user.len()].copy_from_slice(user);
    body[15..15 + password.len()].copy_from_slice(password);
    body[31] = 8;
    body
}

pub fn read_sc_check_start(body: &[u8]) -> Result<(i32, i8)> {
    let mut r = Reader::new(body);
    r.read_c()?;
    let card_id = r.read_d()?;
    let position = r.read_c()? as i8;
    Ok((card_id, position))
}

pub fn request_server_list(login_key: [u32; 2]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_SERVER_LIST);
    w.write_d(login_key[0] as i32);
    w.write_d(login_key[1] as i32);
    w.write_c(4);
    w.into_bytes()
}

pub fn request_server_login(login_key: [u32; 2], server_id: u8) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_c(OP_REQUEST_SERVER_LOGIN);
    w.write_d(login_key[0] as i32);
    w.write_d(login_key[1] as i32);
    w.write_c(server_id);
    w.into_bytes()
}

pub fn describe_refusal(opcode: u8, code: u8) -> (&'static str, String) {
    let (stage, reason) = match (opcode, code) {
        (OP_LOGIN_FAIL, 0x01) => ("credenciais", "erro do sistema"),
        (OP_LOGIN_FAIL, 0x02) => ("credenciais", "senha incorreta"),
        (OP_LOGIN_FAIL, 0x03) => ("credenciais", "usuário ou senha incorretos"),
        (OP_LOGIN_FAIL, 0x04) => ("credenciais", "acesso negado"),
        (OP_LOGIN_FAIL, 0x07) => ("credenciais", "conta já em uso"),
        (OP_LOGIN_FAIL, _) => ("credenciais", "recusado"),
        (OP_ACCOUNT_KICKED, 0x20) => ("conta", "banida permanentemente"),
        (OP_ACCOUNT_KICKED, 0x10) => ("conta", "suspensa por 7 dias"),
        (OP_ACCOUNT_KICKED, _) => ("conta", "expulsa"),
        (OP_ACCOUNT_KICKED_REASON, _) => ("conta", "expulsa"),
        (OP_SERVER_FAIL, _) => ("servidor", "falha do servidor"),
        (OP_PLAY_FAIL, 0x0f) => ("servidor", "servidor cheio"),
        (OP_PLAY_FAIL, 0x04) => ("servidor", "acesso negado"),
        (OP_PLAY_FAIL, _) => ("servidor", "entrada recusada"),
        _ => ("login", "opcode inesperado"),
    };
    (stage, format!("{reason} (código {code})"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_list_keeps_offline_rows_and_exact_population_across_records() {
        let body = [
            4, 2, 9, 9, 127, 0, 0, 1, 0x61, 0x1e, 0, 0, 0, 1, 0x34, 0x12, 0x78, 0x56, 0, 1, 2, 3, 4, 1, 3, 10, 20, 30,
            40, 0x62, 0x1e, 0, 0, 18, 0, 0x45, 0x03, 0x88, 0x13, 1, 5, 6, 7, 8, 0,
        ];
        assert_eq!(
            read_server_list(&body).unwrap(),
            vec![
                GameServer { id: 9, ip: [127, 0, 0, 1], port: 7777, players: 4660, max_players: 22136, online: false, extra: ServerExtra { age: 0, pvp: 1, status: 0, flags: 0x0403_0201, brackets: 1, last: true } },
                GameServer { id: 3, ip: [10, 20, 30, 40], port: 7778, players: 837, max_players: 5000, online: true, extra: ServerExtra { age: 18, pvp: 0, status: 1, flags: 0x0807_0605, brackets: 0, last: false } },
            ]
        );
        for end in 0..body.len() {
            assert!(read_server_list(&body[..end]).is_err());
        }
    }

    #[test]
    fn login_ok_notice_reads_the_fifth_dword_and_the_sixteen_forbidden_bytes_and_tolerates_short_bodies() {
        let mut body = vec![3u8];
        for value in [11u32, 22, 0, 0, 0x3ea, 0, 0, 2] {
            body.extend_from_slice(&value.to_le_bytes());
        }
        body.extend((0..16u8).map(|b| b + 1));
        let notice = read_login_notice(&body);
        assert_eq!(notice.flags, 0x3ea);
        assert!(!notice.shows_warning());
        assert_eq!(notice.forbidden[0], 1);
        assert_eq!(notice.forbidden[15], 16);
        body[17] = 0x03;
        assert!(read_login_notice(&body).shows_warning());
        assert_eq!(read_login_notice(&[3, 1, 0, 0, 0, 2, 0, 0, 0]), super::super::flow::LoginNotice::default());
        assert_eq!(read_login_notice(&[]), super::super::flow::LoginNotice::default());
    }

    #[test]
    fn read_key_pair_rejects_empty_body() {
        assert!(read_key_pair(&[]).is_err());
    }

    #[test]
    fn read_server_list_rejects_empty_body() {
        assert!(read_server_list(&[]).is_err());
    }
}
