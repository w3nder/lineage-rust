use killer_world::session::{Session, SessionEvent};
use killer_world::NetCommand;
use l2_net::game::GameCrypt;
use l2_net::login::{Authorization, GameServer, LoginCipher, LoginFlow, LoginStep, init_is_plaintext};

#[derive(Default)]
pub struct Frames { bytes: Vec<u8> }

impl Frames {
    pub fn feed(&mut self, bytes: &[u8]) -> Result<Vec<Vec<u8>>, String> {
        if self.bytes.len().saturating_add(bytes.len()) > 1024 * 1024 {
            return Err("fila de rede excedeu 1 MiB".into());
        }
        self.bytes.extend_from_slice(bytes);
        let mut offset = 0;
        let mut output = Vec::new();
        while self.bytes.len() - offset >= 2 {
            let length = u16::from_le_bytes([self.bytes[offset],self.bytes[offset+1]]) as usize;
            if length < 2 {return Err("quadro C4 menor que o cabeçalho".into());}
            if self.bytes.len() - offset < length {break;}
            output.push(self.bytes[offset+2..offset+length].to_vec());
            offset += length;
        }
        self.bytes.drain(..offset);
        Ok(output)
    }
}

pub enum LoginUpdate {
    Send(Vec<u8>),
    Notice(l2_net::login::LoginNotice),
    Servers(Vec<GameServer>),
    Card { card_id: i32, position: i8, retry: bool },
    Authorized(Authorization),
}

pub struct Login {
    flow: LoginFlow,
    cipher: LoginCipher,
    frames: Frames,
    first: bool,
    servers: Vec<GameServer>,
    pub refusal: Option<u8>,
    pub blocked: Option<(Option<u32>,Vec<String>)>,
}

impl Login {
    pub fn new(account: &str, password: &str) -> Result<Self,String> {
        Ok(Self {flow:LoginFlow::choosing(account,password).map_err(|e|e.to_string())?,cipher:LoginCipher::c4(),frames:Frames::default(),first:true,servers:Vec::new(),refusal:None,blocked:None})
    }

    pub fn receive(&mut self, bytes: &[u8]) -> Result<Vec<LoginUpdate>,String> {
        let mut updates = Vec::new();
        for raw in self.frames.feed(bytes)? {
            let body = if self.first && init_is_plaintext(&raw) {raw} else {self.cipher.open(&raw).map_err(|e|e.to_string())?};
            self.first = false;
            let step=self.flow.step(&body).map_err(|e|{
                match &e {
                    l2_net::login::LoginError::Refused {code,..}=>self.refusal=Some(*code),
                    l2_net::login::LoginError::Blocked {flags}=>self.blocked=Some((Some(*flags),Vec::new())),
                    l2_net::login::LoginError::BlockedWithMessages {messages}=>self.blocked=Some((None,messages.clone())),
                    _=>{}
                }
                e.to_string()
            })?;
            match step {
                LoginStep::Send(mut body) => {
                    updates.push(LoginUpdate::Send(l2_net::transport::frame::encode(&self.cipher.seal(&body))));
                    body.fill(0);
                }
                LoginStep::Notice(notice) => updates.push(LoginUpdate::Notice(notice)),
                LoginStep::Choose(servers) => {self.servers=servers.clone();updates.push(LoginUpdate::Servers(servers));}
                LoginStep::CardCheck { card_id, position, retry } => updates.push(LoginUpdate::Card { card_id, position, retry }),
                LoginStep::Done(auth) => updates.push(LoginUpdate::Authorized(auth)),
            }
        }
        Ok(updates)
    }

    pub fn accept_notice(&mut self) -> Result<Vec<u8>,String> {
        let body=self.flow.accept_notice().map_err(|e|e.to_string())?;
        Ok(l2_net::transport::frame::encode(&self.cipher.seal(&body)))
    }

    pub fn card_answer(&mut self, card_id: i32, value: i32) -> Result<Vec<u8>,String> {
        let body=self.flow.card_answer(card_id,value).map_err(|e|e.to_string())?;
        Ok(l2_net::transport::frame::encode(&self.cipher.seal(&body)))
    }

    pub fn choose(&mut self, server: u8) -> Result<Vec<u8>,String> {
        let body = self.flow.choose(std::mem::take(&mut self.servers),server).map_err(|e|e.to_string())?;
        Ok(l2_net::transport::frame::encode(&self.cipher.seal(&body)))
    }
}

pub struct Game {
    session: Session,
    crypt: Option<GameCrypt>,
    frames: Frames,
}

pub struct GameUpdate {
    pub events: Vec<SessionEvent>,
    pub outgoing: Vec<Vec<u8>>,
}

impl Game {
    pub fn new(account: String, auth: Authorization) -> Self {
        Self {session:Session::new(account,auth),crypt:None,frames:Frames::default()}
    }

    fn send(&mut self, mut body: Vec<u8>) -> Result<Vec<u8>,String> {
        if body.len()>65533 {return Err("pacote C4 excede 65533 bytes".into());}
        if let Some(crypt) = &mut self.crypt {crypt.encrypt(&mut body);}
        Ok(l2_net::transport::frame::encode(&body))
    }

    pub fn start(&mut self) -> Result<Vec<u8>,String> {self.send(l2_net::game::packets::protocol_version())}

    pub fn command(&mut self, command: NetCommand) -> Result<Option<Vec<u8>>,String> {
        let body = match command {
            NetCommand::Select(slot)=>Some(self.session.select(slot)),
            NetCommand::Logout | NetCommand::Shutdown=>Some(l2_net::game::packets::logout()),
            other=>killer_world::net::encode(&other),
        };
        body.map(|body|self.send(body)).transpose()
    }

    pub fn receive(&mut self, bytes: &[u8]) -> Result<GameUpdate,String> {
        let mut update = GameUpdate {events:Vec::new(),outgoing:Vec::new()};
        for mut body in self.frames.feed(bytes)? {
            if let Some(crypt)=&mut self.crypt {crypt.decrypt(&mut body);}
            let (event,outgoing,crypt)=self.session.on_body(&body);
            if let Some(crypt)=crypt {self.crypt=Some(crypt);}
            for body in outgoing {update.outgoing.push(self.send(body)?);}
            if let Some(event)=event {update.events.push(event);}
        }
        Ok(update)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tcp_fragmentation_and_coalescing_preserve_whole_packets() {
        let expected=vec![vec![0,1,2],vec![7;65533],vec![9]];
        let encoded:Vec<_>=expected.iter().flat_map(|packet|l2_net::transport::frame::encode(packet)).collect();
        for split in [1,2,7,127,65535] {
            let mut frames=Frames::default();
            let output:Vec<_>=encoded.chunks(split).flat_map(|chunk|frames.feed(chunk).unwrap()).collect();
            assert_eq!(output,expected);
            assert!(frames.bytes.is_empty());
        }
        assert!(Frames::default().feed(&[1,0]).is_err());
        assert!(Frames::default().feed(&vec![0;1024*1024+1]).is_err());
    }

    #[test]
    fn browser_login_emits_the_same_encrypted_credentials_as_native() {
        let mut login=Login::new("test","secret").unwrap();
        let init=l2_net::transport::frame::encode(&[0,1,2,3,4,5,6,7,8]);
        let update=login.receive(&init).unwrap();
        let LoginUpdate::Send(bytes)=&update[0] else {panic!("auth")};
        let body=LoginCipher::c4().open(&bytes[2..]).unwrap();
        assert_eq!(&body[..32],l2_net::login::packets::request_auth_login(b"test",b"secret"));
    }

    #[test]
    fn browser_selects_server_and_reuses_game_cipher_in_both_directions() {
        let frame=l2_net::transport::frame::encode;
        let cipher=LoginCipher::c4();
        let mut login=Login::new("test","secret").unwrap();
        login.receive(&frame(&[0;9])).unwrap();
        let ok=login.receive(&frame(&cipher.seal(&[3,1,0,0,0,2,0,0,0]))).unwrap();
        assert!(matches!(ok.as_slice(),[LoginUpdate::Notice(notice)] if *notice==l2_net::login::LoginNotice::default()));
        let request=login.accept_notice().unwrap();
        assert_eq!(cipher.open(&request[2..]).unwrap()[0],5);
        assert!(login.accept_notice().is_err());
        let mut list=vec![4,1,1,1,127,0,0,1];
        list.extend_from_slice(&7777u32.to_le_bytes());
        list.extend_from_slice(&[0,1,3,0,9,0,1,0,0,0,0,0]);
        let servers=login.receive(&frame(&cipher.seal(&list))).unwrap();
        assert!(matches!(&servers[0],LoginUpdate::Servers(servers) if servers[0].port==7777));
        let chosen=login.choose(1).unwrap();
        assert_eq!(cipher.open(&chosen[2..]).unwrap()[0],2);
        let auth=login.receive(&frame(&cipher.seal(&[7,3,0,0,0,4,0,0,0]))).unwrap().pop().unwrap();
        let LoginUpdate::Authorized(auth)=auth else {panic!("autorização")};
        let mut game=Game::new("test".into(),auth);
        assert_eq!(&game.start().unwrap()[2..],l2_net::game::packets::protocol_version());
        let key=[0,1,1,0,0,0,1,0,0,0,0,0,0,0];
        let update=game.receive(&frame(&key)).unwrap();
        let mut peer=GameCrypt::from_key_packet(&key).unwrap();
        let mut auth=update.outgoing[0][2..].to_vec();
        peer.decrypt(&mut auth);
        assert_eq!(auth[0],8);
        let mut characters=vec![0x13,0,0,0,0];
        peer.encrypt(&mut characters);
        let update=game.receive(&frame(&characters)).unwrap();
        assert!(matches!(&update.events[0],SessionEvent::Characters(characters) if characters.is_empty()));
        for command in [
            NetCommand::MoveTo {target:[100,200,-30],origin:[10,20,-3]},
            NetCommand::Trade {object_id:42},
            NetCommand::JoinParty {name:"Parceiro".into(),item_distribution:3},
            NetCommand::OustPartyMember {name:"Parceiro".into()},
            NetCommand::ChangePartyLeader {name:"Parceiro".into()},
            NetCommand::SocialAction {action:2},
            NetCommand::CharacterCreate(Box::new(l2_net::game::packets::NewCharacter {
                name:"Teste".into(), race:1, sex:1, class_id:25,
                int_:0, str_:0, con:0, men:0, dex:0, wit:0,
                hair_style:2, hair_color:1, face:1,
            })),
            NetCommand::CharacterDelete {slot:2},
        ] {
            let mut outgoing=game.command(command.clone()).unwrap().unwrap()[2..].to_vec();
            peer.decrypt(&mut outgoing);
            assert_eq!(outgoing,killer_world::net::encode(&command).unwrap());
        }
        let mut second=vec![0x13,0,0,0,0];
        peer.encrypt(&mut second);
        assert!(matches!(&game.receive(&frame(&second)).unwrap().events[0],SessionEvent::Characters(_)));
    }
}
