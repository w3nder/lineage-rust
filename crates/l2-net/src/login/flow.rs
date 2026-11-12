use super::packets::*;
use crate::wire::WireError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("login recusado ({stage}): {reason}")]
    Refused { stage: &'static str, code: u8, reason: String },
    #[error("conta bloqueada (marcas {flags:#x})")]
    Blocked { flags: u32 },
    #[error("conta bloqueada: {}", messages.join(" / "))]
    BlockedWithMessages { messages: Vec<String> },
    #[error("pedido do cartão de segurança desligado: o original cifra o bloco de 128 bytes com RSA (expoente 10001 em 0x107bd4d8, módulo entregue pelo Init) e o Init do L2Killer tem 9 bytes sem módulo")]
    CardUnavailable,
    #[error("pacote de login malformado: {0}")]
    Malformed(&'static str),
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error("pacote fora de fase")]
    Phase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authorization {
    pub login_key: [u32; 2],
    pub play_key: [u32; 2],
    pub server: GameServer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LoginNotice {
    pub flags: u32,
    pub forbidden: [u8; 16],
}

impl LoginNotice {
    pub fn shows_warning(&self) -> bool {
        self.flags & 1 != 0
    }
}

pub enum LoginStep {
    Send(Vec<u8>),
    Notice(LoginNotice),
    Choose(Vec<GameServer>),
    CardCheck { card_id: i32, position: i8, retry: bool },
    Done(Authorization),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginPhase {
    Init,
    LoginOk,
    Notice,
    ServerList,
    PlayOk,
    Complete,
}

pub struct LoginFlow {
    phase: LoginPhase,
    user: Vec<u8>,
    password: Vec<u8>,
    login_key: [u32; 2],
    server: Option<GameServer>,
    choosing: bool,
    card: Option<(i32, i8)>,
}

impl LoginFlow {
    pub fn new(user: &str, password: &str) -> Result<Self, LoginError> {
        let user = user.as_bytes().to_vec();
        let password = password.as_bytes().to_vec();
        let bounded = |v: &[u8], max: usize| !v.is_empty() && v.len() <= max && !v.contains(&0);
        if !bounded(&user, 14) || !bounded(&password, 16) {
            return Err(LoginError::Malformed("conta até 14 bytes e senha até 16, sem NUL"));
        }
        Ok(Self { phase: LoginPhase::Init, user, password, login_key: [0; 2], server: None, choosing: false, card: None })
    }

    pub fn card_answer(&self, card_id: i32, value: i32) -> Result<Vec<u8>, LoginError> {
        if self.phase != LoginPhase::LoginOk {
            return Err(LoginError::Phase);
        }
        let _ = (card_id, value);
        Err(LoginError::CardUnavailable)
    }

    pub fn choosing(user: &str, password: &str) -> Result<Self, LoginError> {
        let mut flow = Self::new(user, password)?;
        flow.choosing = true;
        Ok(flow)
    }

    pub fn choose(&mut self, servers: Vec<GameServer>, id: u8) -> Result<Vec<u8>, LoginError> {
        if self.phase != LoginPhase::ServerList || !self.choosing {
            return Err(LoginError::Phase);
        }
        let server = servers
            .into_iter()
            .find(|server| server.id == id)
            .ok_or(LoginError::Malformed("servidor escolhido fora da lista"))?;
        let reply = request_server_login(self.login_key, server.id);
        self.server = Some(server);
        self.phase = LoginPhase::PlayOk;
        Ok(reply)
    }

    pub fn accept_notice(&mut self) -> Result<Vec<u8>, LoginError> {
        if self.phase != LoginPhase::Notice {
            return Err(LoginError::Phase);
        }
        self.phase = LoginPhase::ServerList;
        Ok(request_server_list(self.login_key))
    }

    pub fn phase(&self) -> LoginPhase {
        self.phase
    }

    pub fn step(&mut self, body: &[u8]) -> Result<LoginStep, LoginError> {
        let opcode = *body.first().ok_or(LoginError::Malformed("corpo vazio"))?;
        if opcode == OP_LOGIN_FAIL
            && self.phase == LoginPhase::LoginOk
            && body.get(1) == Some(&SC_CHECK_WRONG)
            && let Some((card_id, position)) = self.card
        {
            return Ok(LoginStep::CardCheck { card_id, position, retry: true });
        }
        if self.phase != LoginPhase::Init {
            match opcode {
                OP_LOGIN_FAIL | OP_SERVER_FAIL | OP_PLAY_FAIL | OP_ACCOUNT_KICKED_REASON => {
                    let code = body.get(1).copied().unwrap_or(0);
                    let (stage, reason) = describe_refusal(opcode, code);
                    self.phase = LoginPhase::Complete;
                    return Err(LoginError::Refused { stage, code, reason });
                }
                OP_ACCOUNT_KICKED => {
                    let mut raw = [0u8; 4];
                    for (slot, byte) in raw.iter_mut().zip(body.iter().skip(1)) {
                        *slot = *byte;
                    }
                    self.phase = LoginPhase::Complete;
                    return Err(LoginError::Blocked { flags: u32::from_le_bytes(raw) });
                }
                OP_BLOCKED_WITH_MSG => {
                    let mut reader = crate::wire::Reader::new(&body[1..]);
                    let count = reader.read_c()?;
                    let mut messages = Vec::new();
                    for _ in 0..count {
                        reader.read_h()?;
                        messages.push(reader.read_s()?);
                    }
                    self.phase = LoginPhase::Complete;
                    return Err(LoginError::BlockedWithMessages { messages });
                }
                _ => {}
            }
        }
        if opcode == OP_SC_CHECK_START && self.phase == LoginPhase::LoginOk {
            let (card_id, position) = read_sc_check_start(body)?;
            self.card = Some((card_id, position));
            return Ok(LoginStep::CardCheck { card_id, position, retry: false });
        }
        match (self.phase, opcode) {
            (LoginPhase::Init, OP_INIT) => {
                if body.len() != 9 {
                    return Err(LoginError::Malformed("Init deve ter 9 bytes"));
                }
                let reply = request_auth_login(&self.user, &self.password);
                self.password.fill(0);
                self.phase = LoginPhase::LoginOk;
                Ok(LoginStep::Send(reply))
            }
            (LoginPhase::LoginOk, OP_LOGIN_OK) => {
                self.login_key = read_key_pair(body)?;
                if self.choosing {
                    self.phase = LoginPhase::Notice;
                    return Ok(LoginStep::Notice(read_login_notice(body)));
                }
                self.phase = LoginPhase::ServerList;
                Ok(LoginStep::Send(request_server_list(self.login_key)))
            }
            (LoginPhase::ServerList, OP_SERVER_LIST) => {
                let servers = read_server_list(body)?;
                if servers.is_empty() {
                    return Err(LoginError::Malformed("lista de servidores vazia"));
                }
                if self.choosing {
                    return Ok(LoginStep::Choose(servers));
                }
                let server = servers.into_iter().next().ok_or(LoginError::Malformed("lista de servidores vazia"))?;
                let reply = request_server_login(self.login_key, server.id);
                self.server = Some(server);
                self.phase = LoginPhase::PlayOk;
                Ok(LoginStep::Send(reply))
            }
            (LoginPhase::PlayOk, OP_PLAY_OK) => {
                let play_key = read_key_pair(body)?;
                let server = self.server.take().ok_or(LoginError::Phase)?;
                self.phase = LoginPhase::Complete;
                Ok(LoginStep::Done(Authorization { login_key: self.login_key, play_key, server }))
            }
            _ => Err(LoginError::Phase),
        }
    }
}

impl Drop for LoginFlow {
    fn drop(&mut self) {
        self.password.fill(0);
        self.user.fill(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server_list(count: u8, status: u8) -> Vec<u8> {
        let mut body = vec![4, count, 1];
        for id in 1..=count {
            body.extend_from_slice(&[id, 192, 99, 44, 99]);
            body.extend_from_slice(&7777u32.to_le_bytes());
            body.extend_from_slice(&[0, 1]);
            body.extend_from_slice(&12u16.to_le_bytes());
            body.extend_from_slice(&500u16.to_le_bytes());
            body.push(status);
            body.extend_from_slice(&[0, 0, 0, 0, 0]);
        }
        body
    }

    #[test]
    fn choosing_flow_stops_at_the_server_list_and_logs_into_the_chosen_server() {
        let mut flow = LoginFlow::choosing("conta", "senha").unwrap();
        let mut init = vec![0u8];
        init.extend_from_slice(&[0u8; 8]);
        let LoginStep::Send(_) = flow.step(&init).unwrap() else { panic!() };
        let mut ok = vec![3u8];
        ok.extend_from_slice(&11u32.to_le_bytes());
        ok.extend_from_slice(&22u32.to_le_bytes());
        ok.extend_from_slice(&[0u8; 8]);
        let LoginStep::Notice(notice) = flow.step(&ok).unwrap() else { panic!() };
        assert_eq!(notice, LoginNotice::default());
        assert_eq!(flow.phase(), LoginPhase::Notice);
        assert!(flow.choose(Vec::new(), 2).is_err());
        assert_eq!(flow.accept_notice().unwrap(), [5, 11, 0, 0, 0, 22, 0, 0, 0, 4]);
        assert!(flow.accept_notice().is_err());
        assert!(flow.choose(Vec::new(), 2).is_err());
        let LoginStep::Choose(servers) = flow.step(&server_list(3, 1)).unwrap() else { panic!() };
        assert_eq!(servers.iter().map(|s| s.id).collect::<Vec<_>>(), [1, 2, 3]);
        assert!(flow.choose(servers.clone(), 9).is_err());
        assert_eq!(flow.phase(), LoginPhase::ServerList);
        let play = flow.choose(servers, 2).unwrap();
        assert_eq!(play, [2, 11, 0, 0, 0, 22, 0, 0, 0, 2]);
        assert_eq!(flow.phase(), LoginPhase::PlayOk);
        let mut playok = vec![7u8];
        playok.extend_from_slice(&33u32.to_le_bytes());
        playok.extend_from_slice(&44u32.to_le_bytes());
        let LoginStep::Done(auth) = flow.step(&playok).unwrap() else { panic!() };
        assert_eq!(auth.server.id, 2);
        let mut plain = LoginFlow::new("conta", "senha").unwrap();
        assert!(plain.choose(Vec::new(), 1).is_err());
    }

    #[test]
    fn the_security_card_start_0x0a_carries_the_card_id_and_the_position_and_the_answer_stays_off_until_the_rsa_block_can_be_built() {
        let mut flow = LoginFlow::new("conta", "senha").unwrap();
        let mut init = vec![0u8];
        init.extend_from_slice(&[0u8; 8]);
        let LoginStep::Send(_) = flow.step(&init).unwrap() else { panic!() };
        let mut start = vec![0x0au8];
        start.extend_from_slice(&77i32.to_le_bytes());
        start.push(3);
        let LoginStep::CardCheck { card_id, position, retry } = flow.step(&start).unwrap() else { panic!() };
        assert_eq!((card_id, position, retry), (77, 3, false));
        let LoginStep::CardCheck { retry, .. } = flow.step(&[1, 31]).unwrap() else { panic!() };
        assert!(retry);
        assert_eq!(flow.phase(), LoginPhase::LoginOk);
        assert!(matches!(flow.card_answer(77, 1234), Err(LoginError::CardUnavailable)));
        assert!(matches!(flow.step(&[1, 2]), Err(LoginError::Refused { .. })));
        let mut early = LoginFlow::new("conta", "senha").unwrap();
        assert!(matches!(early.step(&start), Err(LoginError::Phase)));
    }

    #[test]
    fn full_handshake() {
        let mut flow = LoginFlow::new("conta", "senha").unwrap();
        let mut init = vec![0u8];
        init.extend_from_slice(&0x1234u32.to_le_bytes());
        init.extend_from_slice(&0xc621u32.to_le_bytes());
        let LoginStep::Send(auth) = flow.step(&init).unwrap() else { panic!() };
        assert_eq!(auth.len(), 32);
        assert_eq!(auth[0], 0);
        assert_eq!(&auth[1..6], b"conta");
        assert_eq!(&auth[15..20], b"senha");
        assert_eq!(auth[31], 8);

        let mut ok = vec![3u8];
        ok.extend_from_slice(&11u32.to_le_bytes());
        ok.extend_from_slice(&22u32.to_le_bytes());
        ok.extend_from_slice(&[0u8; 8]);
        let LoginStep::Send(list) = flow.step(&ok).unwrap() else { panic!() };
        assert_eq!(list, [5, 11, 0, 0, 0, 22, 0, 0, 0, 4]);

        let LoginStep::Send(play) = flow.step(&server_list(2, 1)).unwrap() else { panic!() };
        assert_eq!(play, [2, 11, 0, 0, 0, 22, 0, 0, 0, 1]);

        let mut playok = vec![7u8];
        playok.extend_from_slice(&33u32.to_le_bytes());
        playok.extend_from_slice(&44u32.to_le_bytes());
        let LoginStep::Done(auth) = flow.step(&playok).unwrap() else { panic!() };
        assert_eq!(auth.login_key, [11, 22]);
        assert_eq!(auth.play_key, [33, 44]);
        assert_eq!(auth.server.address(), "192.99.44.99:7777");
        assert!(auth.server.online);
        assert_eq!(auth.server.players, 12);
    }

    #[test]
    fn refusal_after_init_is_reported() {
        let mut flow = LoginFlow::new("conta", "senha").unwrap();
        let mut init = vec![0u8];
        init.extend_from_slice(&[0u8; 8]);
        flow.step(&init).unwrap();
        let Err(err) = flow.step(&[1, 7]) else { panic!() };
        assert!(matches!(err, LoginError::Refused { code: 7, .. }));
    }

    #[test]
    fn blocked_account_packets_follow_the_engine_handlers() {
        let mut flow = LoginFlow::new("conta", "senha").unwrap();
        let mut init = vec![0u8];
        init.extend_from_slice(&[0u8; 8]);
        flow.step(&init).unwrap();
        let Err(err) = flow.step(&[2, 0x10, 0x01, 0, 0]) else { panic!() };
        assert!(matches!(err, LoginError::Blocked { flags: 0x110 }));
        let mut flow = LoginFlow::new("conta", "senha").unwrap();
        flow.step(&init).unwrap();
        let mut body = vec![9u8, 2];
        for text in ["ab", "c"] {
            body.extend_from_slice(&0u16.to_le_bytes());
            for unit in text.encode_utf16().chain([0]) {
                body.extend_from_slice(&unit.to_le_bytes());
            }
        }
        let Err(LoginError::BlockedWithMessages { messages }) = flow.step(&body) else { panic!() };
        assert_eq!(messages, ["ab", "c"]);
        let mut flow = LoginFlow::new("conta", "senha").unwrap();
        flow.step(&init).unwrap();
        assert!(matches!(flow.step(&[8, 3]), Err(LoginError::Refused { code: 3, .. })));
    }

    #[test]
    fn credentials_are_bounded() {
        assert!(LoginFlow::new("", "x").is_err());
        assert!(LoginFlow::new("quinze_letras_x", "x").is_err());
        assert!(LoginFlow::new("ok", "dezessete_letras_").is_err());
    }

    #[test]
    fn empty_server_list_is_malformed() {
        let mut flow = LoginFlow::new("conta", "senha").unwrap();
        let mut init = vec![0u8];
        init.extend_from_slice(&[0u8; 8]);
        flow.step(&init).unwrap();
        let mut ok = vec![3u8];
        ok.extend_from_slice(&[0u8; 8]);
        flow.step(&ok).unwrap();
        assert!(matches!(flow.step(&[4, 0, 0]), Err(LoginError::Malformed(_))));
    }

    #[test]
    fn password_is_zeroed_after_init_step() {
        let mut flow = LoginFlow::new("conta", "senha").unwrap();
        let mut init = vec![0u8];
        init.extend_from_slice(&[0u8; 8]);
        flow.step(&init).unwrap();
        assert!(flow.password.iter().all(|b| *b == 0));
    }
}
