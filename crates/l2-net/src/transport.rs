use crate::game::GameCrypt;
use crate::login::{
    Authorization, CipherError, GameServer, LoginCipher, LoginError, LoginFlow, LoginNotice, LoginPhase, LoginStep,
    init_is_plaintext,
};
use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use thiserror::Error;

pub mod frame {
    use super::*;

    pub fn encode(body: &[u8]) -> Vec<u8> {
        let len = (body.len() + 2) as u16;
        let mut out = Vec::with_capacity(body.len() + 2);
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(body);
        out
    }

    pub fn read(reader: &mut impl Read) -> io::Result<Vec<u8>> {
        let mut header = [0u8; 2];
        reader.read_exact(&mut header)?;
        let len = u16::from_le_bytes(header) as usize;
        if len < 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "quadro menor que o cabeçalho"));
        }
        let mut body = vec![0u8; len - 2];
        reader.read_exact(&mut body)?;
        Ok(body)
    }
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Login(#[from] LoginError),
    #[error(transparent)]
    Cipher(#[from] CipherError),
    #[error("conexão encerrada: {0}")]
    Closed(&'static str),
}

fn checked_frame(body: &[u8]) -> io::Result<Vec<u8>> {
    if body.len() > 65533 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "corpo maior que 65533 bytes"));
    }
    Ok(frame::encode(body))
}

fn connect(addr: &str, timeout: Duration) -> io::Result<TcpStream> {
    let resolved = addr
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "endereço não resolvido"))?;
    let stream = TcpStream::connect_timeout(&resolved, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    stream.set_nodelay(true)?;
    Ok(stream)
}

pub fn authenticate(
    host: &str,
    port: u16,
    user: &str,
    password: &str,
    timeout: Duration,
) -> Result<Authorization, TransportError> {
    run_login(host, port, LoginFlow::new(user, password)?, timeout, |_| true, |_| None, |_, _, _| None)
}

#[allow(clippy::too_many_arguments)]
pub fn authenticate_choosing(
    host: &str,
    port: u16,
    user: &str,
    password: &str,
    timeout: Duration,
    notice: impl FnMut(&LoginNotice) -> bool,
    choose: impl FnMut(&[GameServer]) -> Option<u8>,
    card: impl FnMut(i32, i8, bool) -> Option<i32>,
) -> Result<Authorization, TransportError> {
    run_login(host, port, LoginFlow::choosing(user, password)?, timeout, notice, choose, card)
}

fn run_login(
    host: &str,
    port: u16,
    mut flow: LoginFlow,
    timeout: Duration,
    mut notice: impl FnMut(&LoginNotice) -> bool,
    mut choose: impl FnMut(&[GameServer]) -> Option<u8>,
    mut card: impl FnMut(i32, i8, bool) -> Option<i32>,
) -> Result<Authorization, TransportError> {
    let cipher = LoginCipher::c4();
    let mut stream = connect(&format!("{host}:{port}"), timeout)?;
    let mut init_pending = true;
    loop {
        let raw = frame::read(&mut stream).map_err(|e| match e.kind() {
            io::ErrorKind::UnexpectedEof => TransportError::Closed(match flow.phase() {
                LoginPhase::Init => "o login server fechou a conexão antes do Init",
                LoginPhase::LoginOk => {
                    "o login server fechou a conexão ao receber as credenciais; conta já online ou pedido malformado"
                }
                LoginPhase::Notice | LoginPhase::ServerList => "o login server fechou a conexão antes da lista de servidores",
                LoginPhase::PlayOk | LoginPhase::Complete => {
                    "o login server fechou a conexão antes de liberar o game server"
                }
            }),
            _ => TransportError::Io(e),
        })?;
        let body = if init_pending && init_is_plaintext(&raw) {
            init_pending = false;
            raw
        } else {
            cipher.open(&raw)?
        };
        match flow.step(&body)? {
            LoginStep::Send(mut reply) => {
                stream.write_all(&checked_frame(&cipher.seal(&reply))?)?;
                reply.fill(0);
            }
            LoginStep::Notice(info) => {
                if !notice(&info) {
                    return Err(TransportError::Closed("termos recusados"));
                }
                let reply = flow.accept_notice()?;
                stream.write_all(&checked_frame(&cipher.seal(&reply))?)?;
            }
            LoginStep::Choose(servers) => {
                let Some(id) = choose(&servers) else {
                    return Err(TransportError::Closed("escolha de servidor cancelada"));
                };
                let reply = flow.choose(servers, id)?;
                stream.write_all(&checked_frame(&cipher.seal(&reply))?)?;
            }
            LoginStep::CardCheck { card_id, position, retry } => {
                let Some(value) = card(card_id, position, retry) else {
                    return Err(TransportError::Closed("cartão de segurança cancelado"));
                };
                let reply = flow.card_answer(card_id, value)?;
                stream.write_all(&checked_frame(&cipher.seal(&reply))?)?;
            }
            LoginStep::Done(auth) => return Ok(auth),
        }
    }
}

pub struct GameConnection {
    stream: TcpStream,
    crypt: Option<GameCrypt>,
}

impl GameConnection {
    pub fn connect(addr: &str, timeout: Duration) -> io::Result<Self> {
        Ok(Self { stream: connect(addr, timeout)?, crypt: None })
    }

    pub fn enable_crypt(&mut self, crypt: GameCrypt) {
        self.crypt = Some(crypt);
    }

    pub fn send(&mut self, mut body: Vec<u8>) -> io::Result<()> {
        if let Some(crypt) = &mut self.crypt {
            crypt.encrypt(&mut body);
        }
        self.stream.write_all(&checked_frame(&body)?)
    }

    pub fn receive(&mut self) -> io::Result<Vec<u8>> {
        let mut body = frame::read(&mut self.stream)?;
        if let Some(crypt) = &mut self.crypt {
            crypt.decrypt(&mut body);
        }
        Ok(body)
    }

    pub fn set_read_timeout(&mut self, d: Option<Duration>) -> io::Result<()> {
        self.stream.set_read_timeout(d)
    }

    pub fn receive_timeout(&mut self, d: Duration) -> io::Result<Option<Vec<u8>>> {
        self.stream.set_read_timeout(Some(d))?;
        let mut probe = [0u8; 1];
        let peeked = self.stream.peek(&mut probe);
        self.stream.set_read_timeout(None)?;
        match peeked {
            Ok(_) => Ok(Some(self.receive()?)),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn frame_roundtrip() {
        let framed = frame::encode(&[9, 8, 7]);
        assert_eq!(framed, [5, 0, 9, 8, 7]);
        let mut cursor = Cursor::new(framed);
        assert_eq!(frame::read(&mut cursor).unwrap(), [9, 8, 7]);
    }

    #[test]
    fn frame_rejects_lengths_below_header() {
        let mut cursor = Cursor::new(vec![1u8, 0]);
        assert!(frame::read(&mut cursor).is_err());
    }

    #[test]
    fn game_connection_encrypts_after_key() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let first = frame::read(&mut socket).unwrap();
            let key_packet = vec![0u8, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0];
            socket.write_all(&frame::encode(&key_packet)).unwrap();
            let mut second = frame::read(&mut socket).unwrap();
            GameCrypt::from_key_packet(&key_packet).unwrap().decrypt(&mut second);
            (first, second)
        });
        let mut conn = GameConnection::connect(&addr.to_string(), Duration::from_secs(2)).unwrap();
        conn.send(vec![0, 1, 2]).unwrap();
        let key_packet = conn.receive().unwrap();
        conn.enable_crypt(GameCrypt::from_key_packet(&key_packet).unwrap());
        conn.send(vec![8, 9]).unwrap();
        let (first, second) = server.join().unwrap();
        assert_eq!(first, [0, 1, 2]);
        assert_eq!(second, [8, 9]);
    }

    #[test]
    fn authenticate_runs_the_full_flow_against_a_fake_login_server() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let cipher = LoginCipher::c4();
            let (mut socket, _) = listener.accept().unwrap();
            let mut init = vec![0u8];
            init.extend_from_slice(&[0u8; 8]);
            socket.write_all(&frame::encode(&init)).unwrap();
            let auth = cipher.open(&frame::read(&mut socket).unwrap()).unwrap();
            assert_eq!(auth[0], 0);
            let mut ok = vec![3u8];
            ok.extend_from_slice(&[1, 0, 0, 0, 2, 0, 0, 0]);
            socket.write_all(&frame::encode(&cipher.seal(&ok))).unwrap();
            let list_req = cipher.open(&frame::read(&mut socket).unwrap()).unwrap();
            assert_eq!(list_req[0], 5);
            let mut list = vec![4u8, 1, 1, 1, 127, 0, 0, 1];
            list.extend_from_slice(&7777u32.to_le_bytes());
            list.extend_from_slice(&[0, 1, 3, 0, 9, 0, 1, 0, 0, 0, 0, 0]);
            socket.write_all(&frame::encode(&cipher.seal(&list))).unwrap();
            let play_req = cipher.open(&frame::read(&mut socket).unwrap()).unwrap();
            assert_eq!(play_req[0], 2);
            let mut playok = vec![7u8];
            playok.extend_from_slice(&[3, 0, 0, 0, 4, 0, 0, 0]);
            socket.write_all(&frame::encode(&cipher.seal(&playok))).unwrap();
        });
        let auth = authenticate("127.0.0.1", addr.port(), "conta", "senha", Duration::from_secs(2)).unwrap();
        assert_eq!(auth.login_key, [1, 2]);
        assert_eq!(auth.play_key, [3, 4]);
        assert_eq!(auth.server.address(), "127.0.0.1:7777");
    }

    #[test]
    fn authenticate_choosing_hands_the_list_to_the_caller_and_logs_into_its_pick() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let cipher = LoginCipher::c4();
            let (mut socket, _) = listener.accept().unwrap();
            let mut init = vec![0u8];
            init.extend_from_slice(&[0u8; 8]);
            socket.write_all(&frame::encode(&init)).unwrap();
            cipher.open(&frame::read(&mut socket).unwrap()).unwrap();
            let mut ok = vec![3u8];
            ok.extend_from_slice(&[1, 0, 0, 0, 2, 0, 0, 0]);
            socket.write_all(&frame::encode(&cipher.seal(&ok))).unwrap();
            cipher.open(&frame::read(&mut socket).unwrap()).unwrap();
            let mut list = vec![4u8, 2, 1];
            for id in [1u8, 2] {
                list.extend_from_slice(&[id, 127, 0, 0, id]);
                list.extend_from_slice(&7777u32.to_le_bytes());
                list.extend_from_slice(&[0, 1, 3, 0, 9, 0, 1, 0, 0, 0, 0, 0]);
            }
            socket.write_all(&frame::encode(&cipher.seal(&list))).unwrap();
            let play_req = cipher.open(&frame::read(&mut socket).unwrap()).unwrap();
            assert_eq!(play_req[0], 2);
            assert_eq!(play_req[9], 2);
            let mut playok = vec![7u8];
            playok.extend_from_slice(&[3, 0, 0, 0, 4, 0, 0, 0]);
            socket.write_all(&frame::encode(&cipher.seal(&playok))).unwrap();
        });
        let mut offered = Vec::new();
        let mut notices = 0;
        let auth =
            authenticate_choosing(
                "127.0.0.1",
                addr.port(),
                "conta",
                "senha",
                Duration::from_secs(2),
                |_| {
                    notices += 1;
                    true
                },
                |servers| {
                    offered = servers.iter().map(|s| s.id).collect();
                    Some(2)
                },
                |_, _, _| None,
            )
            .unwrap();
        assert_eq!(offered, [1, 2]);
        assert_eq!(notices, 1);
        assert_eq!(auth.server.address(), "127.0.0.2:7777");
    }

    #[test]
    fn send_rejects_body_over_the_frame_limit() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let _ = listener.accept();
        });
        let mut conn = GameConnection::connect(&addr.to_string(), Duration::from_secs(2)).unwrap();
        let err = conn.send(vec![0u8; 65534]).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn receive_timeout_returns_none_then_the_whole_frame() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            thread::sleep(Duration::from_millis(300));
            let framed = frame::encode(&[1, 2, 3]);
            socket.write_all(&framed[..2]).unwrap();
            thread::sleep(Duration::from_millis(300));
            socket.write_all(&framed[2..]).unwrap();
        });
        let mut conn = GameConnection::connect(&addr.to_string(), Duration::from_secs(2)).unwrap();
        assert!(conn.receive_timeout(Duration::from_millis(200)).unwrap().is_none());
        let body = conn.receive_timeout(Duration::from_millis(200)).unwrap().unwrap();
        assert_eq!(body, [1, 2, 3]);
        server.join().unwrap();
    }

    #[test]
    fn eof_while_sending_credentials_names_the_login_phase() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut init = vec![0u8];
            init.extend_from_slice(&[0u8; 8]);
            socket.write_all(&frame::encode(&init)).unwrap();
            let _ = frame::read(&mut socket).unwrap();
        });
        let err = authenticate("127.0.0.1", addr.port(), "conta", "senha", Duration::from_secs(2)).unwrap_err();
        match err {
            TransportError::Closed(msg) => assert!(msg.contains("credenciais")),
            other => panic!("esperava TransportError::Closed, veio {other}"),
        }
    }
}
