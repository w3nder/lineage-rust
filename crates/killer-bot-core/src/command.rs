use crate::chat::Channel;
use crate::session::{BotSession, BotState};
use killer_world::NetCommand;

#[derive(Debug, Clone, PartialEq)]
pub enum UserCommand {
    Say { channel: Channel, target: String, text: String },
    UseItem(i32),
    UseSkill(i32),
    MoveTo([i32; 3]),
    SelectTarget(i32),
    CancelTarget,
    Sit,
    Stand,
    Action(i32),
    DropItem { object_id: i32, count: i32 },
    DestroyItem { object_id: i32, count: i32 },
    Crystallize { object_id: i32, count: i32 },
    ClearStats,
    RequestClan,
    IgnoreItem(i32),
    IgnoreNpc(i32),
    Blacklist(i32),
    Buy { list_id: i32, orders: Vec<(i32, i32)> },
    Sell { list_id: i32, orders: Vec<(i32, i32, i32)> },
    StoreBuy { orders: Vec<(i32, i32)> },
    StoreSell { orders: Vec<(i32, i32)> },
    StoreManage { buy: bool },
    StoreSetSell { package_sale: bool, offers: Vec<(i32, i32, i32)> },
    StoreSetBuy { offers: Vec<(i32, i32, i32)> },
    StoreMessage { buy: bool, text: String },
    StoreQuit { buy: bool },
    PetGive { object_id: i32, count: i32 },
    PetTake { object_id: i32, count: i32 },
    Pause,
    Resume,
    SetAutomation(bool),
    Bypass(String),
    LinkHtml(String),
    MultiSellChoose { list_id: i32, entry_id: i32, amount: i32 },
    Warehouse { deposit: bool, orders: Vec<(i32, i32)> },
    LearnSkill { skill_id: i32, level: i32, mode: i32 },
    TradeRequest(i32),
    TradeAdd { object_id: i32, count: i32 },
    TradeConfirm,
    TradeCancel,
    PartyInvite(String),
    PartyLeave,
    PartyKick(String),
    PartyLeader(String),
    PartyDismiss,
    ClanInvite(i32),
    FriendInvite(String),
    FriendList,
    FriendDelete(String),
    RequestQuests,
    AbortQuest(i32),
    Net(NetCommand),
    RestartPoint(i32),
    Answer { index: usize, accept: bool },
    ChooseServer(u8),
    ChooseCharacter(usize),
    DeleteCharacter(usize),
    RequestNewCharacter,
    CreateCharacter(Box<l2_net::game::NewCharacter>),
    Restart,
    Logout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    NotInWorld,
    Dead,
    NoTarget,
    UnknownItem,
    UnknownSkill,
    ChannelNotSendable,
    EmptyText,
    NoSuchQuestion,
    RestartNotAllowed,
    WrongStage,
    NoSuchChoice,
    NoPet,
    BadCount,
    NoShop,
    NotEnoughAdena,
    BadName,
}

impl BotSession {
    fn origin(&self) -> Option<[i32; 3]> {
        self.world().me.as_ref().map(|me| me.position.to_array().map(|value| value as i32))
    }

    pub fn command(&mut self, command: UserCommand) -> Result<(), Refusal> {
        match &command {
            UserCommand::Pause => {
                self.manual_pause = true;
                return Ok(());
            }
            UserCommand::Resume => {
                self.manual_pause = false;
                return Ok(());
            }
            UserCommand::SetAutomation(on) => {
                self.automation = *on;
                return Ok(());
            }
            _ => {}
        }
        match &command {
            UserCommand::IgnoreItem(item_id) => {
                let item_id = *item_id;
                self.update_options(|options| {
                    if !options.do_not.items.contains(&item_id) {
                        options.do_not.items.push(item_id);
                    }
                });
                return Ok(());
            }
            UserCommand::IgnoreNpc(npc_id) => {
                let npc_id = *npc_id;
                self.update_options(|options| {
                    if !options.do_not.npcs.contains(&npc_id) {
                        options.do_not.npcs.push(npc_id);
                    }
                });
                return Ok(());
            }
            UserCommand::Blacklist(object_id) => {
                self.blacklist(*object_id);
                return Ok(());
            }
            UserCommand::ClearStats => {
                self.clear_stats();
                return Ok(());
            }
            UserCommand::ChooseServer(id) => {
                if self.state() != BotState::ChoosingServer {
                    return Err(Refusal::WrongStage);
                }
                if !self.servers().iter().any(|server| server.id == *id) {
                    return Err(Refusal::NoSuchChoice);
                }
                self.send(NetCommand::ChooseServer(*id));
                return Ok(());
            }
            UserCommand::ChooseCharacter(slot) => {
                if self.state() != BotState::ChoosingCharacter {
                    return Err(Refusal::WrongStage);
                }
                if *slot >= self.characters().len() {
                    return Err(Refusal::NoSuchChoice);
                }
                self.send(NetCommand::Select(*slot as i32));
                return Ok(());
            }
            UserCommand::RequestNewCharacter => {
                if self.state() != BotState::ChoosingCharacter {
                    return Err(Refusal::WrongStage);
                }
                self.send(NetCommand::NewCharacter);
                return Ok(());
            }
            UserCommand::CreateCharacter(new) => {
                if self.state() != BotState::ChoosingCharacter {
                    return Err(Refusal::WrongStage);
                }
                let name = new.name.trim();
                if name.is_empty() || name.len() > 16 {
                    return Err(Refusal::BadName);
                }
                let mut new = new.clone();
                new.name = name.to_owned();
                self.send(NetCommand::CharacterCreate(new));
                return Ok(());
            }
            UserCommand::DeleteCharacter(slot) => {
                if self.state() != BotState::ChoosingCharacter {
                    return Err(Refusal::WrongStage);
                }
                if *slot >= self.characters().len() {
                    return Err(Refusal::NoSuchChoice);
                }
                self.send(NetCommand::CharacterDelete { slot: *slot as i32 });
                return Ok(());
            }
            _ => {}
        }
        if self.state() != BotState::InWorld {
            return Err(Refusal::NotInWorld);
        }
        let dead = self.world().me.as_ref().is_some_and(|me| me.dead) || self.world().death.is_some();
        let allowed_when_dead = matches!(command, UserCommand::Say { .. } | UserCommand::CancelTarget | UserCommand::Bypass(_) | UserCommand::LinkHtml(_) | UserCommand::RestartPoint(_) | UserCommand::Answer { .. } | UserCommand::Restart | UserCommand::Logout);
        if dead && !allowed_when_dead {
            return Err(Refusal::Dead);
        }
        let net = match command {
            UserCommand::Say { channel, target, text } => {
                let kind = channel.kind().ok_or(Refusal::ChannelNotSendable)?;
                if text.trim().is_empty() {
                    return Err(Refusal::EmptyText);
                }
                if channel == Channel::Whisper && target.trim().is_empty() {
                    return Err(Refusal::NoTarget);
                }
                NetCommand::Say { text, kind, target }
            }
            UserCommand::UseItem(object_id) => {
                if self.world().inventory.find(object_id).is_none() {
                    return Err(Refusal::UnknownItem);
                }
                NetCommand::UseItem { object_id, ctrl: false }
            }
            UserCommand::UseSkill(skill_id) => {
                if !self.world().skills.iter().any(|skill| skill.id == skill_id && !skill.passive) {
                    return Err(Refusal::UnknownSkill);
                }
                NetCommand::UseSkill { skill_id, ctrl: false, shift: false }
            }
            UserCommand::MoveTo(target) => NetCommand::MoveTo { target, origin: self.origin().ok_or(Refusal::NotInWorld)? },
            UserCommand::SelectTarget(object_id) => {
                if self.world().entity(object_id).is_none() {
                    return Err(Refusal::NoTarget);
                }
                NetCommand::Action { object_id, origin: self.origin().ok_or(Refusal::NotInWorld)?, shift: false }
            }
            UserCommand::CancelTarget => NetCommand::TargetCancel,
            UserCommand::Sit => NetCommand::ChangeWaitType { standing: false },
            UserCommand::Stand => NetCommand::ChangeWaitType { standing: true },
            UserCommand::Buy { list_id, orders } => {
                let shop = self.world().windows.buy.as_ref().filter(|shop| shop.list_id == list_id).ok_or(Refusal::NoShop)?;
                if orders.is_empty() {
                    return Err(Refusal::BadCount);
                }
                let mut total = 0i64;
                for (item_id, count) in &orders {
                    let priced = shop.items.iter().find(|priced| priced.item.item_id == *item_id).ok_or(Refusal::UnknownItem)?;
                    if *count <= 0 || (priced.item.count > 0 && *count > priced.item.count) {
                        return Err(Refusal::BadCount);
                    }
                    total += i64::from(priced.price) * i64::from(*count);
                }
                if total > i64::from(self.world().inventory.adena()) {
                    return Err(Refusal::NotEnoughAdena);
                }
                NetCommand::BuyItems { list_id, orders }
            }
            UserCommand::Sell { list_id, orders } => {
                let shop = self.world().windows.sell.as_ref().filter(|shop| shop.list_id == list_id).ok_or(Refusal::NoShop)?;
                if orders.is_empty() {
                    return Err(Refusal::BadCount);
                }
                let mut sold = Vec::new();
                for (object_id, item_id, count) in &orders {
                    let listed = shop.items.iter().find(|priced| priced.item.object_id == *object_id).ok_or(Refusal::UnknownItem)?;
                    if *count <= 0 || *count > listed.item.count {
                        return Err(Refusal::BadCount);
                    }
                    sold.push(l2_net::game::ItemOrder { object_id: *object_id, item_id: *item_id, count: *count });
                }
                NetCommand::SellItems { list_id, orders: sold }
            }
            UserCommand::StoreBuy { orders } => {
                let store = self.world().stores.sell.as_ref().ok_or(Refusal::NoShop)?;
                if orders.is_empty() {
                    return Err(Refusal::BadCount);
                }
                let mut total = 0i64;
                let mut picked = Vec::new();
                for (object_id, count) in &orders {
                    let item = store.items.iter().find(|item| item.object_id == *object_id).ok_or(Refusal::UnknownItem)?;
                    if *count <= 0 || *count > item.count {
                        return Err(Refusal::BadCount);
                    }
                    total += i64::from(item.price) * i64::from(*count);
                    picked.push(l2_net::game::StoreBuyOrder { object_id: *object_id, count: *count, price: item.price });
                }
                if store.package_sale && !store.items.iter().all(|item| orders.iter().any(|(id, count)| *id == item.object_id && *count == item.count)) {
                    return Err(Refusal::BadCount);
                }
                if total > i64::from(self.world().inventory.adena()) {
                    return Err(Refusal::NotEnoughAdena);
                }
                NetCommand::PrivateStoreBuy { store_id: store.object_id, orders: picked }
            }
            UserCommand::StoreSell { orders } => {
                let store = self.world().stores.buy.as_ref().ok_or(Refusal::NoShop)?;
                if orders.is_empty() {
                    return Err(Refusal::BadCount);
                }
                let mut sold = Vec::new();
                for (object_id, count) in &orders {
                    let wanted = store.items.iter().find(|item| item.object_id == *object_id).ok_or(Refusal::UnknownItem)?;
                    let mine = self.world().inventory.find(*object_id).ok_or(Refusal::UnknownItem)?;
                    if *count <= 0 || *count > wanted.count || *count > mine.count {
                        return Err(Refusal::BadCount);
                    }
                    sold.push(l2_net::game::SellToStoreItem { object_id: *object_id, item_id: wanted.item_id, enchant: wanted.enchant, custom_type2: mine.custom_type2, count: *count, price: wanted.price });
                }
                NetCommand::PrivateStoreSell { store_id: store.object_id, items: sold }
            }
            UserCommand::StoreManage { buy } => if buy { NetCommand::PrivateStoreManageBuy } else { NetCommand::PrivateStoreManageSell },
            UserCommand::StoreSetSell { package_sale, offers } => {
                let manage = self.world().stores.manage_sell.as_ref().ok_or(Refusal::NoShop)?;
                if offers.is_empty() {
                    return Err(Refusal::BadCount);
                }
                let mut list = Vec::new();
                for (object_id, count, price) in offers {
                    let item = manage.available.iter().find(|item| item.object_id == object_id).ok_or(Refusal::UnknownItem)?;
                    if count <= 0 || count > item.count || price < 0 {
                        return Err(Refusal::BadCount);
                    }
                    list.push(l2_net::game::StoreOffer { object_id, count, price });
                }
                NetCommand::SetPrivateStoreListSell { package_sale, offers: list }
            }
            UserCommand::StoreSetBuy { offers } => {
                let manage = self.world().stores.manage_buy.as_ref().ok_or(Refusal::NoShop)?;
                if offers.is_empty() {
                    return Err(Refusal::BadCount);
                }
                let mut list = Vec::new();
                for (item_id, count, price) in offers {
                    let item = manage.available.iter().find(|item| item.item_id == item_id).ok_or(Refusal::UnknownItem)?;
                    if count <= 0 || price < 0 {
                        return Err(Refusal::BadCount);
                    }
                    list.push(l2_net::game::BuyOffer { item_id, enchant: item.enchant, custom_type2: 0, count, price });
                }
                NetCommand::SetPrivateStoreListBuy { offers: list }
            }
            UserCommand::StoreMessage { buy, text } => {
                if buy { NetCommand::SetPrivateStoreMsgBuy { message: text } } else { NetCommand::SetPrivateStoreMsgSell { message: text } }
            }
            UserCommand::StoreQuit { buy } => if buy { NetCommand::PrivateStoreQuitBuy } else { NetCommand::PrivateStoreQuitSell },
            UserCommand::PetGive { object_id, count } => {
                if self.world().pet.is_none() {
                    return Err(Refusal::NoPet);
                }
                let item = self.world().inventory.find(object_id).ok_or(Refusal::UnknownItem)?;
                if count <= 0 || count > item.count {
                    return Err(Refusal::BadCount);
                }
                NetCommand::GiveItemToPet { object_id, count }
            }
            UserCommand::PetTake { object_id, count } => {
                if self.world().pet.is_none() {
                    return Err(Refusal::NoPet);
                }
                let item = self.world().pet_inventory.find(object_id).ok_or(Refusal::UnknownItem)?;
                if count <= 0 || count > item.count {
                    return Err(Refusal::BadCount);
                }
                NetCommand::GetItemFromPet { object_id, count }
            }
            UserCommand::DropItem { object_id, count } | UserCommand::DestroyItem { object_id, count } | UserCommand::Crystallize { object_id, count } => {
                let item = self.world().inventory.find(object_id).ok_or(Refusal::UnknownItem)?;
                if count <= 0 || count > item.count {
                    return Err(Refusal::BadCount);
                }
                match command {
                    UserCommand::DropItem { .. } => NetCommand::DropItem { object_id, count, position: self.origin().ok_or(Refusal::NotInWorld)? },
                    UserCommand::DestroyItem { .. } => NetCommand::DestroyItem { object_id, count },
                    _ => NetCommand::CrystallizeItem { object_id, count },
                }
            }
            UserCommand::RequestClan => NetCommand::RequestPledgeMemberList,
            UserCommand::Action(id) => NetCommand::UseAction { id, ctrl: false, shift: false },
            UserCommand::Bypass(command) => NetCommand::Bypass { command },
            UserCommand::LinkHtml(link) => NetCommand::LinkHtml { link },
            UserCommand::PartyInvite(name) => NetCommand::JoinParty { name, item_distribution: 0 },
            UserCommand::PartyLeave => NetCommand::WithdrawParty,
            UserCommand::PartyKick(name) => NetCommand::OustPartyMember { name },
            UserCommand::PartyLeader(name) => NetCommand::ChangePartyLeader { name },
            UserCommand::PartyDismiss => NetCommand::DismissParty,
            UserCommand::ClanInvite(object_id) => {
                if self.world().entity(object_id).is_none() {
                    return Err(Refusal::NoTarget);
                }
                NetCommand::JoinPledge { object_id }
            }
            UserCommand::FriendInvite(name) => NetCommand::FriendInvite { name },
            UserCommand::FriendList => NetCommand::FriendList,
            UserCommand::FriendDelete(name) => NetCommand::FriendDel { name },
            UserCommand::Net(command) => command,
            UserCommand::RequestQuests => NetCommand::RequestQuestList,
            UserCommand::AbortQuest(quest_id) => NetCommand::QuestAbort { quest_id },
            UserCommand::Warehouse { deposit, orders } => {
                if self.world().windows.warehouse.is_none() || self.world().windows.warehouse_deposit != deposit {
                    return Err(Refusal::NoShop);
                }
                if orders.is_empty() || orders.iter().any(|(_, count)| *count <= 0) {
                    return Err(Refusal::BadCount);
                }
                if deposit { NetCommand::WarehouseDeposit { orders } } else { NetCommand::WarehouseWithdraw { orders } }
            }
            UserCommand::LearnSkill { skill_id, level, mode } => {
                let list = self.world().windows.learnable.as_ref().ok_or(Refusal::NoShop)?;
                if !list.skills.iter().any(|skill| skill.id == skill_id && skill.next_level == level) {
                    return Err(Refusal::UnknownSkill);
                }
                NetCommand::AcquireSkill { skill_id, level, mode }
            }
            UserCommand::TradeRequest(object_id) => {
                if self.world().entity(object_id).is_none() {
                    return Err(Refusal::NoTarget);
                }
                NetCommand::Trade { object_id }
            }
            UserCommand::TradeAdd { object_id, count } => {
                if self.world().trade.partner_id.is_none() {
                    return Err(Refusal::NoShop);
                }
                if count <= 0 {
                    return Err(Refusal::BadCount);
                }
                if !self.world().trade.tradable.iter().any(|item| item.object_id == object_id) {
                    return Err(Refusal::UnknownItem);
                }
                NetCommand::AddTradeItem { trade_id: 1, object_id, count }
            }
            UserCommand::TradeConfirm | UserCommand::TradeCancel => {
                if self.world().trade.partner_id.is_none() {
                    return Err(Refusal::NoShop);
                }
                NetCommand::TradeDone { confirm: command == UserCommand::TradeConfirm }
            }
            UserCommand::MultiSellChoose { list_id, entry_id, amount } => {
                let state = self.world().windows.multisell.as_ref().filter(|state| state.list_id == list_id).ok_or(Refusal::NoShop)?;
                if !state.offers.iter().any(|offer| offer.entry_id == entry_id) {
                    return Err(Refusal::UnknownItem);
                }
                if amount <= 0 {
                    return Err(Refusal::BadCount);
                }
                NetCommand::MultiSellChoose { list_id, entry_id, amount }
            }
            UserCommand::Restart => NetCommand::Restart,
            UserCommand::Logout => NetCommand::Logout,
            UserCommand::RestartPoint(point) => {
                let allowed = self.world().death.as_ref().is_some_and(|die| match point {
                    0 => die.to_village,
                    1 => die.to_hideout,
                    2 => die.to_castle,
                    3 => die.to_siege_hq,
                    4 => die.to_fixed_spot,
                    _ => false,
                });
                if !allowed {
                    return Err(Refusal::RestartNotAllowed);
                }
                NetCommand::RestartPoint { point }
            }
            UserCommand::Answer { index, accept } => self.take_question(index, accept).ok_or(Refusal::NoSuchQuestion)?,
            UserCommand::Pause | UserCommand::Resume | UserCommand::SetAutomation(_) | UserCommand::ChooseServer(_) | UserCommand::ChooseCharacter(_) | UserCommand::DeleteCharacter(_) | UserCommand::RequestNewCharacter | UserCommand::CreateCharacter(_) | UserCommand::IgnoreItem(_) | UserCommand::IgnoreNpc(_) | UserCommand::Blacklist(_) | UserCommand::ClearStats => return Ok(()),
        };
        self.send(net);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::ManualClock;
    use crate::profile::Profile;
    use crate::testkit::{Vitals, char_entry, npc_info, user_info};
    use crate::transport::{FakeTransport, Transport};
    use killer_world::{NetEvent, SessionEvent, WorldEvent};
    use l2_net::game::SkillEntry;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[derive(Clone, Default)]
    struct Wire(Arc<Mutex<FakeTransport>>);

    impl Transport for Wire {
        fn send(&mut self, command: NetCommand) {
            self.0.lock().unwrap().send(command);
        }

        fn poll(&mut self) -> Vec<NetEvent> {
            self.0.lock().unwrap().poll()
        }
    }

    const PROFILE: &str = "account = \"a\"\ncharacter = \"Wender\"\n[server]\nhost = \"h\"\nport = 1\n[options.attack]\nactive_target = true\nactive_attack = true\n";

    fn in_world() -> (BotSession, Wire, ManualClock) {
        let wire = Wire::default();
        let clock = ManualClock::new();
        let mut session = BotSession::new(Profile::parse(PROFILE).unwrap(), Box::new(wire.clone()), Arc::new(clock.clone()));
        let mut inbox = wire.0.lock().unwrap();
        inbox.push(NetEvent::Session(SessionEvent::Characters(vec![char_entry("Wender")])));
        inbox.push(NetEvent::Session(user_info(1, "Wender", &Vitals::full(), [0, 0, 0])));
        inbox.push(NetEvent::Session(SessionEvent::World(WorldEvent::SkillList(l2_net::game::SkillList {
            skills: vec![SkillEntry { passive: false, level: 1, id: 56 }, SkillEntry { passive: true, level: 1, id: 7 }],
        }))));
        drop(inbox);
        session.tick();
        wire.0.lock().unwrap().sent.clear();
        (session, wire, clock)
    }

    fn sent(wire: &Wire) -> Vec<NetCommand> {
        wire.0.lock().unwrap().sent.clone()
    }

    #[test]
    fn commands_are_refused_when_not_in_world() {
        let wire = Wire::default();
        let mut session = BotSession::new(Profile::parse(PROFILE).unwrap(), Box::new(wire.clone()), Arc::new(ManualClock::new()));
        assert_eq!(session.command(UserCommand::Sit), Err(Refusal::NotInWorld));
        assert_eq!(session.command(UserCommand::Pause), Ok(()));
        assert!(sent(&wire).is_empty());
    }

    #[test]
    fn an_action_button_sends_the_client_action_id() {
        let (mut session, wire, _) = in_world();
        assert_eq!(session.command(UserCommand::Action(4)), Ok(()));
        assert_eq!(sent(&wire), vec![NetCommand::UseAction { id: 4, ctrl: false, shift: false }]);
    }

    fn trade_item(object_id: i32, item_id: i32, count: i32) -> l2_net::game::TradeItem {
        l2_net::game::TradeItem { type1: 0, object_id, item_id, count, type2: 0, custom_type1: 0, body_part: 0, enchant: 0, custom_type2: 0 }
    }

    #[test]
    fn buy_and_sell_are_checked_against_the_open_shop_and_the_wallet() {
        use l2_net::game::{PricedItem, ShopList};
        let (mut session, wire, _) = in_world();
        let order = vec![(1060, 5)];
        assert_eq!(session.command(UserCommand::Buy { list_id: 7, orders: order.clone() }), Err(Refusal::NoShop));
        wire.0.lock().unwrap().push(NetEvent::Session(SessionEvent::World(WorldEvent::BuyList(ShopList { money: 0, list_id: 7, items: vec![PricedItem { item: trade_item(0, 1060, 0), price: 100 }] }))));
        session.tick();
        assert_eq!(session.command(UserCommand::Buy { list_id: 8, orders: order.clone() }), Err(Refusal::NoShop));
        assert_eq!(session.command(UserCommand::Buy { list_id: 7, orders: vec![(999, 1)] }), Err(Refusal::UnknownItem));
        assert_eq!(session.command(UserCommand::Buy { list_id: 7, orders: vec![(1060, 0)] }), Err(Refusal::BadCount));
        assert_eq!(session.command(UserCommand::Buy { list_id: 7, orders: order }), Err(Refusal::NotEnoughAdena));
        assert!(sent(&wire).is_empty());
        wire.0.lock().unwrap().push(NetEvent::Session(SessionEvent::World(WorldEvent::SellList(ShopList { money: 0, list_id: 9, items: vec![PricedItem { item: trade_item(500, 57, 3), price: 1 }] }))));
        session.tick();
        assert_eq!(session.command(UserCommand::Sell { list_id: 9, orders: vec![(500, 57, 4)] }), Err(Refusal::BadCount));
        assert_eq!(session.command(UserCommand::Sell { list_id: 9, orders: vec![(501, 57, 1)] }), Err(Refusal::UnknownItem));
        assert_eq!(session.command(UserCommand::Sell { list_id: 9, orders: vec![(500, 57, 2)] }), Ok(()));
        assert_eq!(sent(&wire), vec![NetCommand::SellItems { list_id: 9, orders: vec![l2_net::game::ItemOrder { object_id: 500, item_id: 57, count: 2 }] }]);
        session.close_shop();
        assert_eq!(session.command(UserCommand::Sell { list_id: 9, orders: vec![(500, 57, 1)] }), Err(Refusal::NoShop));
    }

    #[test]
    fn character_creation_only_works_while_choosing_and_checks_the_name_length() {
        let wire = Wire::default();
        let profile = Profile::parse("account = \"a\"\n[server]\nhost = \"h\"\nport = 1\n").unwrap();
        let mut session = BotSession::new(profile, Box::new(wire.clone()), Arc::new(ManualClock::new()));
        let new = |name: &str| Box::new(l2_net::game::NewCharacter { name: name.to_owned(), race: 0, sex: 0, class_id: 0, int_: 0, str_: 0, con: 0, men: 0, dex: 0, wit: 0, hair_style: 0, hair_color: 0, face: 0 });
        assert_eq!(session.command(UserCommand::CreateCharacter(new("Novo"))), Err(Refusal::WrongStage));
        assert_eq!(session.command(UserCommand::RequestNewCharacter), Err(Refusal::WrongStage));
        wire.0.lock().unwrap().push(NetEvent::Session(SessionEvent::Characters(vec![char_entry("Wender")])));
        session.tick();
        assert_eq!(session.state(), BotState::ChoosingCharacter);
        assert_eq!(session.command(UserCommand::CreateCharacter(new("  "))), Err(Refusal::BadName));
        assert_eq!(session.command(UserCommand::CreateCharacter(new("DezesseisLetrasXX"))), Err(Refusal::BadName));
        assert_eq!(session.command(UserCommand::RequestNewCharacter), Ok(()));
        assert_eq!(session.command(UserCommand::CreateCharacter(new(" Novo "))), Ok(()));
        let sent = sent(&wire);
        assert_eq!(sent[0], NetCommand::NewCharacter);
        assert!(matches!(&sent[1], NetCommand::CharacterCreate(created) if created.name == "Novo"));
    }

    #[test]
    fn private_store_purchases_are_checked_against_the_visited_store() {
        use l2_net::game::{PrivateStoreListSell, StoreSellItem};
        let (mut session, wire, _) = in_world();
        let item = |object_id: i32, count: i32, price: i32| StoreSellItem { type2: 0, object_id, item_id: 1060, count, enchant: 0, custom_type2: 0, body_part: 0, price, store_price: 0 };
        assert_eq!(session.command(UserCommand::StoreBuy { orders: vec![(5, 1)] }), Err(Refusal::NoShop));
        let list = |package_sale: bool| SessionEvent::World(WorldEvent::PrivateStoreListSell(PrivateStoreListSell { object_id: 77, package_sale, adena: 0, items: vec![item(5, 10, 100), item(6, 2, 50)] }));
        wire.0.lock().unwrap().push(NetEvent::Session(list(false)));
        session.tick();
        assert_eq!(session.command(UserCommand::StoreBuy { orders: vec![(9, 1)] }), Err(Refusal::UnknownItem));
        assert_eq!(session.command(UserCommand::StoreBuy { orders: vec![(5, 11)] }), Err(Refusal::BadCount));
        assert_eq!(session.command(UserCommand::StoreBuy { orders: vec![(5, 1)] }), Err(Refusal::NotEnoughAdena));
        wire.0.lock().unwrap().push(NetEvent::Session(list(true)));
        session.tick();
        assert_eq!(session.command(UserCommand::StoreBuy { orders: vec![(6, 2)] }), Err(Refusal::BadCount));
        assert_eq!(session.command(UserCommand::StoreSell { orders: vec![(5, 1)] }), Err(Refusal::NoShop));
        assert_eq!(session.command(UserCommand::StoreMessage { buy: false, text: "vendo".into() }), Ok(()));
        assert_eq!(session.command(UserCommand::StoreSetSell { package_sale: false, offers: vec![(1, 1, 1)] }), Err(Refusal::NoShop));
        session.close_store();
        assert_eq!(session.command(UserCommand::StoreBuy { orders: vec![(5, 1)] }), Err(Refusal::NoShop));
        assert!(matches!(sent(&wire).last(), Some(NetCommand::SetPrivateStoreMsgSell { message }) if message == "vendo"));
    }

    #[test]
    fn inventory_commands_check_the_stack_and_ignore_lists_edit_the_options() {
        let (mut session, wire, _) = in_world();
        assert_eq!(session.command(UserCommand::DestroyItem { object_id: 1, count: 1 }), Err(Refusal::UnknownItem));
        session.command(UserCommand::IgnoreItem(57)).unwrap();
        session.command(UserCommand::IgnoreItem(57)).unwrap();
        session.command(UserCommand::IgnoreNpc(20001)).unwrap();
        assert_eq!(session.profile().options.do_not.items, vec![57]);
        assert_eq!(session.profile().options.do_not.npcs, vec![20001]);
        session.command(UserCommand::Blacklist(9)).unwrap();
        assert!(sent(&wire).is_empty());
    }

    #[test]
    fn pet_item_commands_need_a_pet_and_valid_counts() {
        let (mut session, wire, _) = in_world();
        assert_eq!(session.command(UserCommand::PetGive { object_id: 900, count: 1 }), Err(Refusal::NoPet));
        assert_eq!(session.command(UserCommand::PetTake { object_id: 900, count: 1 }), Err(Refusal::NoPet));
        assert!(sent(&wire).is_empty());
    }

    #[test]
    fn use_skill_requires_a_known_active_skill() {
        let (mut session, wire, _) = in_world();
        assert_eq!(session.command(UserCommand::UseSkill(99)), Err(Refusal::UnknownSkill));
        assert_eq!(session.command(UserCommand::UseSkill(7)), Err(Refusal::UnknownSkill));
        assert_eq!(session.command(UserCommand::UseSkill(56)), Ok(()));
        assert_eq!(sent(&wire), vec![NetCommand::UseSkill { skill_id: 56, ctrl: false, shift: false }]);
    }

    #[test]
    fn dead_character_refuses_movement_and_skills_but_can_still_talk() {
        let (mut session, wire, _) = in_world();
        wire.0.lock().unwrap().push(NetEvent::Session(SessionEvent::World(WorldEvent::Die(l2_net::game::Die {
            object_id: 1,
            to_village: true,
            to_hideout: false,
            to_castle: false,
            to_siege_hq: false,
            sweepable: false,
            to_fixed_spot: false,
        }))));
        session.tick();
        assert_eq!(session.command(UserCommand::MoveTo([1, 1, 1])), Err(Refusal::Dead));
        assert_eq!(session.command(UserCommand::UseSkill(56)), Err(Refusal::Dead));
        assert_eq!(session.command(UserCommand::Say { channel: Channel::All, target: String::new(), text: "oi".into() }), Ok(()));
    }

    #[test]
    fn pause_stops_automation_and_resume_restarts_it() {
        let (mut session, wire, clock) = in_world();
        wire.0.lock().unwrap().push(NetEvent::Session(npc_info(9, 1, "Wolf", true, [100, 0, 0])));
        session.command(UserCommand::Pause).unwrap();
        clock.advance(Duration::from_secs(1));
        session.tick();
        assert!(sent(&wire).is_empty());
        session.command(UserCommand::Resume).unwrap();
        clock.advance(Duration::from_secs(1));
        session.tick();
        assert!(!sent(&wire).is_empty());
        wire.0.lock().unwrap().sent.clear();
        session.command(UserCommand::SetAutomation(false)).unwrap();
        clock.advance(Duration::from_secs(1));
        session.tick();
        assert!(sent(&wire).is_empty());
    }

    #[test]
    fn say_maps_each_channel_to_the_net_command_and_validates_the_text() {
        let (mut session, wire, _) = in_world();
        session.command(UserCommand::Say { channel: Channel::Whisper, target: "Luzal".into(), text: "oi".into() }).unwrap();
        assert_eq!(sent(&wire), vec![NetCommand::Say { text: "oi".into(), kind: l2_net::game::packets::CHAT_TELL, target: "Luzal".into() }]);
        assert_eq!(session.command(UserCommand::Say { channel: Channel::System, target: String::new(), text: "x".into() }), Err(Refusal::ChannelNotSendable));
        assert_eq!(session.command(UserCommand::Say { channel: Channel::All, target: String::new(), text: "  ".into() }), Err(Refusal::EmptyText));
        assert_eq!(session.command(UserCommand::Say { channel: Channel::Whisper, target: String::new(), text: "oi".into() }), Err(Refusal::NoTarget));
    }

    #[test]
    fn restart_only_to_the_points_the_server_offered_and_questions_are_answered_once() {
        let (mut session, wire, _) = in_world();
        assert_eq!(session.command(UserCommand::RestartPoint(0)), Err(Refusal::RestartNotAllowed));
        {
            let mut inbox = wire.0.lock().unwrap();
            inbox.push(NetEvent::Session(SessionEvent::World(WorldEvent::Die(l2_net::game::Die {
                object_id: 1,
                to_village: true,
                to_hideout: false,
                to_castle: false,
                to_siege_hq: false,
                sweepable: false,
                to_fixed_spot: false,
            }))));
            inbox.push(NetEvent::Session(SessionEvent::World(WorldEvent::AskJoinParty(l2_net::game::AskJoinParty { requestor_name: "Tank".into(), item_distribution: 0 }))));
        }
        session.tick();
        wire.0.lock().unwrap().sent.clear();
        assert_eq!(session.command(UserCommand::RestartPoint(2)), Err(Refusal::RestartNotAllowed));
        assert_eq!(session.command(UserCommand::RestartPoint(0)), Ok(()));
        assert_eq!(session.command(UserCommand::Answer { index: 0, accept: true }), Ok(()));
        assert_eq!(session.command(UserCommand::Answer { index: 0, accept: true }), Err(Refusal::NoSuchQuestion));
        assert_eq!(sent(&wire), vec![NetCommand::RestartPoint { point: 0 }, NetCommand::AnswerJoinParty { accept: true }]);
    }

    #[test]
    fn use_item_and_select_target_need_something_that_exists() {
        let (mut session, wire, _) = in_world();
        assert_eq!(session.command(UserCommand::UseItem(500)), Err(Refusal::UnknownItem));
        assert_eq!(session.command(UserCommand::SelectTarget(9)), Err(Refusal::NoTarget));
        {
            let mut inbox = wire.0.lock().unwrap();
            inbox.push(NetEvent::Session(npc_info(9, 1, "Wolf", true, [100, 0, 0])));
        }
        session.tick();
        wire.0.lock().unwrap().sent.clear();
        assert_eq!(session.command(UserCommand::SelectTarget(9)), Ok(()));
        assert_eq!(sent(&wire), vec![NetCommand::Action { object_id: 9, origin: [0, 0, 0], shift: false }]);
    }

    #[test]
    fn the_character_walks_toward_the_server_destination_as_time_passes() {
        let (mut session, wire, clock) = in_world();
        wire.0.lock().unwrap().push(NetEvent::Session(SessionEvent::World(WorldEvent::MoveToLocation(l2_net::game::MoveToLocation {
            object_id: 1,
            to_x: 1000,
            to_y: 0,
            to_z: 0,
            x: 0,
            y: 0,
            z: 0,
        }))));
        session.tick();
        clock.advance(std::time::Duration::from_secs(1));
        session.tick();
        let me = session.world().me.as_ref().unwrap();
        assert!(me.position.x > 50.0 && me.dest.is_some(), "{:?}", me.position);
        clock.advance(std::time::Duration::from_secs(1));
        for _ in 0..30 {
            clock.advance(std::time::Duration::from_secs(1));
            session.tick();
        }
        let me = session.world().me.as_ref().unwrap();
        assert!(me.dest.is_none() && me.position.x == 1000.0);
    }

    #[test]
    fn a_link_html_action_sends_the_link_request() {
        let (mut session, wire, _) = in_world();
        assert_eq!(session.command(UserCommand::LinkHtml("teleporter/1.htm".into())), Ok(()));
        assert_eq!(sent(&wire), vec![NetCommand::LinkHtml { link: "teleporter/1.htm".into() }]);
    }

    #[test]
    fn teleporting_clears_the_surroundings_and_announces_the_new_position() {
        let (mut session, wire, _) = in_world();
        wire.0.lock().unwrap().push(NetEvent::Session(npc_info(9, 20001, "Wolf", true, [100, 0, 0])));
        session.tick();
        assert_eq!(session.world().entities.len(), 1);
        wire.0.lock().unwrap().push(NetEvent::Session(SessionEvent::World(WorldEvent::TeleportToLocation(l2_net::game::TeleportToLocation { target_id: 1, x: 5000, y: 6000, z: -100, instant: 0 }))));
        session.tick();
        assert!(session.world().entities.is_empty());
        let sent = sent(&wire);
        assert!(sent.iter().any(|command| matches!(command, NetCommand::Validate { position, .. } if *position == [5000, 6000, -100])));
        assert!(sent.contains(&NetCommand::Appearing));
    }

    #[test]
    fn an_exchange_is_checked_against_the_open_multisell_list() {
        use l2_net::game::{MultiSellEntry, MultiSellIngredient, MultiSellList, MultiSellProduct};
        let (mut session, wire, _) = in_world();
        let choose = |list_id, entry_id, amount| UserCommand::MultiSellChoose { list_id, entry_id, amount };
        assert_eq!(session.command(choose(10, 1, 1)), Err(Refusal::NoShop));
        let entry = MultiSellEntry {
            entry_id: 1,
            stackable: 1,
            products: vec![MultiSellProduct { item_id: 1, body_part: 0, type2: 0, count: 1, enchant: 0 }],
            ingredients: vec![MultiSellIngredient { item_id: 57, type2: 4, count: 100, enchant: 0 }],
        };
        wire.0.lock().unwrap().push(NetEvent::Session(SessionEvent::World(WorldEvent::MultiSellList(MultiSellList { list_id: 10, page: 1, finished: true, page_size: 40, entries: vec![entry] }))));
        session.tick();
        assert_eq!(session.command(choose(11, 1, 1)), Err(Refusal::NoShop));
        assert_eq!(session.command(choose(10, 2, 1)), Err(Refusal::UnknownItem));
        assert_eq!(session.command(choose(10, 1, 0)), Err(Refusal::BadCount));
        assert_eq!(session.command(choose(10, 1, 3)), Ok(()));
        assert_eq!(sent(&wire), vec![NetCommand::MultiSellChoose { list_id: 10, entry_id: 1, amount: 3 }]);
        session.close_multisell();
        assert_eq!(session.command(choose(10, 1, 1)), Err(Refusal::NoShop));
    }

    #[test]
    fn warehouse_learning_and_trade_commands_are_checked_against_the_open_windows() {
        use l2_net::game::{AcquireSkillList, LearnableSkill, TradeStart, WarehouseItem, WarehouseList};
        let (mut session, wire, _) = in_world();
        assert_eq!(session.command(UserCommand::Warehouse { deposit: true, orders: vec![(5, 1)] }), Err(Refusal::NoShop));
        assert_eq!(session.command(UserCommand::LearnSkill { skill_id: 1, level: 1, mode: 0 }), Err(Refusal::NoShop));
        assert_eq!(session.command(UserCommand::TradeConfirm), Err(Refusal::NoShop));
        let item = trade_item(5, 57, 10);
        let events = [
            WorldEvent::WarehouseDepositList(WarehouseList { warehouse_type: 1, adena: 0, items: vec![WarehouseItem { item, warehouse_id: 0 }] }),
            WorldEvent::AcquireSkillList(AcquireSkillList { mode: 0, skills: vec![LearnableSkill { id: 56, next_level: 2, max_level: 5, sp_cost: 100, requirements: 0 }] }),
            WorldEvent::TradeStart(TradeStart { partner_id: 77, items: vec![item] }),
        ];
        for event in events {
            wire.0.lock().unwrap().push(NetEvent::Session(SessionEvent::World(event)));
        }
        session.tick();
        assert_eq!(session.command(UserCommand::Warehouse { deposit: false, orders: vec![(5, 1)] }), Err(Refusal::NoShop));
        assert_eq!(session.command(UserCommand::Warehouse { deposit: true, orders: vec![(5, 0)] }), Err(Refusal::BadCount));
        assert_eq!(session.command(UserCommand::Warehouse { deposit: true, orders: vec![(5, 4)] }), Ok(()));
        assert_eq!(session.command(UserCommand::LearnSkill { skill_id: 56, level: 3, mode: 0 }), Err(Refusal::UnknownSkill));
        assert_eq!(session.command(UserCommand::LearnSkill { skill_id: 56, level: 2, mode: 0 }), Ok(()));
        assert_eq!(session.command(UserCommand::TradeAdd { object_id: 999, count: 1 }), Err(Refusal::UnknownItem));
        assert_eq!(session.command(UserCommand::TradeAdd { object_id: 5, count: 3 }), Ok(()));
        assert_eq!(session.command(UserCommand::TradeConfirm), Ok(()));
        assert_eq!(session.command(UserCommand::TradeCancel), Ok(()));
        assert_eq!(
            sent(&wire),
            vec![
                NetCommand::WarehouseDeposit { orders: vec![(5, 4)] },
                NetCommand::AcquireSkill { skill_id: 56, level: 2, mode: 0 },
                NetCommand::AddTradeItem { trade_id: 1, object_id: 5, count: 3 },
                NetCommand::TradeDone { confirm: true },
                NetCommand::TradeDone { confirm: false },
            ]
        );
    }

    #[test]
    fn party_clan_friend_and_quest_commands_map_to_the_wire() {
        let (mut session, wire, _) = in_world();
        wire.0.lock().unwrap().push(NetEvent::Session(npc_info(9, 1, "Wolf", true, [100, 0, 0])));
        session.tick();
        wire.0.lock().unwrap().sent.clear();
        assert_eq!(session.command(UserCommand::ClanInvite(404)), Err(Refusal::NoTarget));
        for command in [
            UserCommand::PartyInvite("Tank".into()),
            UserCommand::PartyLeave,
            UserCommand::PartyKick("Tank".into()),
            UserCommand::PartyLeader("Tank".into()),
            UserCommand::PartyDismiss,
            UserCommand::ClanInvite(9),
            UserCommand::FriendInvite("Tank".into()),
            UserCommand::FriendList,
            UserCommand::FriendDelete("Tank".into()),
            UserCommand::RequestQuests,
            UserCommand::AbortQuest(5),
        ] {
            assert_eq!(session.command(command), Ok(()));
        }
        assert_eq!(
            sent(&wire),
            vec![
                NetCommand::JoinParty { name: "Tank".into(), item_distribution: 0 },
                NetCommand::WithdrawParty,
                NetCommand::OustPartyMember { name: "Tank".into() },
                NetCommand::ChangePartyLeader { name: "Tank".into() },
                NetCommand::DismissParty,
                NetCommand::JoinPledge { object_id: 9 },
                NetCommand::FriendInvite { name: "Tank".into() },
                NetCommand::FriendList,
                NetCommand::FriendDel { name: "Tank".into() },
                NetCommand::RequestQuestList,
                NetCommand::QuestAbort { quest_id: 5 },
            ]
        );
    }
}
