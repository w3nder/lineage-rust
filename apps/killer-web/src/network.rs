use crate::protocol::{Game, Login, LoginUpdate};
use killer_world::session::SessionEvent;
use killer_world::NetCommand;
use serde_json::{Value, json};

#[derive(Default)]
pub struct Network {
    login: Option<Login>,
    game: Option<Game>,
    account: String,
    pub events: Vec<SessionEvent>,
    pub servers: Vec<l2_net::login::GameServer>,
    pub notice: Option<l2_net::login::LoginNotice>,
    pub card: Option<i32>,
    pub card_event: Option<(i32,i8,bool)>,
    pub outgoing: Vec<Vec<u8>>,
    pub entered: bool,
    pub disconnect_pending: bool,
}

impl Network {
    pub fn start(&mut self,account: String,password: &str) -> Result<(),String> {
        let login=Login::new(&account,password)?;
        *self=Self {login:Some(login),account,..Default::default()};
        Ok(())
    }

    pub fn login_receive(&mut self,bytes: &[u8]) -> Result<Value,String> {
        let updates=self.login.as_mut().ok_or("login não iniciado")?.receive(bytes)?;
        let mut result=json!({"outgoing":[],"servers":null,"connectGame":false,"notice":false});
        for update in updates {
            match update {
                LoginUpdate::Send(bytes)=>self.outgoing.push(bytes),
                LoginUpdate::Notice(notice)=>{result["notice"]=json!(true);self.notice=Some(notice);},
                LoginUpdate::Card {card_id,position,retry}=>{self.card=Some(card_id);self.card_event=Some((card_id,position,retry));}
                LoginUpdate::Servers(servers)=>{result["servers"]=json!(true);self.servers=servers;},
                LoginUpdate::Authorized(auth)=>{
                    self.game=Some(Game::new(std::mem::take(&mut self.account),auth));
                    self.login=None;
                    result["connectGame"]=json!(true);
                }
            }
        }
        result["outgoing"]=json!(std::mem::take(&mut self.outgoing));
        Ok(result)
    }

    pub fn accept_notice(&mut self) -> Result<Vec<u8>,String> {
        self.login.as_mut().ok_or("login não iniciado")?.accept_notice()
    }

    pub fn card_answer(&mut self,card_id: i32,value: i32) -> Result<Vec<u8>,String> {
        self.login.as_mut().ok_or("login não iniciado")?.card_answer(card_id,value)
    }

    pub fn choose_server(&mut self,id: u8) -> Result<Vec<u8>,String> {
        self.login.as_mut().ok_or("login não iniciado")?.choose(id)
    }

    pub fn start_game(&mut self) -> Result<Vec<u8>,String> {
        self.game.as_mut().ok_or("jogo não autorizado")?.start()
    }

    pub fn receive_game(&mut self,bytes: &[u8]) -> Result<Value,String> {
        let update=self.game.as_mut().ok_or("jogo não autorizado")?.receive(bytes)?;
        self.outgoing.extend(update.outgoing);
        let mut result=json!({"characters":null,"entered":false});
        for event in update.events {
            match &event {
                SessionEvent::Characters(characters)=>result["characters"]=json!(characters.iter().enumerate().map(|(slot,character)|json!({"slot":slot,"name":character.name,"level":character.level,"class":character.class_id,"deleted":character.delete_days!=0})).collect::<Vec<_>>()),
                SessionEvent::UserInfo(_)=>{self.entered=true;result["entered"]=json!(true)},
                SessionEvent::Refused(reason)=>return Err(reason.clone()),
                SessionEvent::ServerClosed if self.entered=>self.disconnect_pending=true,
                SessionEvent::ServerClosed | SessionEvent::LeftWorld=>return Err("conexão encerrada pelo servidor".into()),
                _=>{},
            }
            self.events.push(event);
        }
        Ok(result)
    }

    pub fn command(&mut self,command: NetCommand) -> Result<(),String> {
        if let Some(game)=&mut self.game && let Some(bytes)=game.command(command)? {self.outgoing.push(bytes);}
        Ok(())
    }

    pub fn take_refusal(&mut self) -> Option<u8> {self.login.as_mut().and_then(|login|login.refusal.take())}

    pub fn take_blocked(&mut self) -> Option<(Option<u32>,Vec<String>)> {self.login.as_mut().and_then(|login|login.blocked.take())}

    pub fn drain(&mut self) -> Vec<Vec<u8>> {std::mem::take(&mut self.outgoing)}
}
