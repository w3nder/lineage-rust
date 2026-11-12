use killer_bot_core::names::Names;
use killer_world::World;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub object_id: i32,
    pub item_id: i32,
    pub name: String,
    pub add_name: String,
    pub grade: Option<&'static str>,
    pub count: i32,
    pub enchant: i32,
    pub price: i32,
    pub store_price: i32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShopSide {
    pub list_id: i32,
    pub items: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShopDto {
    pub adena: i32,
    pub buy: Option<ShopSide>,
    pub sell: Option<ShopSide>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Piece {
    pub item_id: i32,
    pub name: String,
    pub add_name: String,
    pub grade: Option<&'static str>,
    pub count: i32,
    pub enchant: i32,
    pub have: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OfferDto {
    pub entry_id: i32,
    pub stackable: bool,
    pub products: Vec<Piece>,
    pub ingredients: Vec<Piece>,
    pub can: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MultisellDto {
    pub list_id: i32,
    pub finished: bool,
    pub adena: i32,
    pub offers: Vec<OfferDto>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseDto {
    pub deposit: bool,
    pub adena: i32,
    pub items: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LearnSkillDto {
    pub id: i32,
    pub name: String,
    pub next_level: i32,
    pub sp_cost: i32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LearnDto {
    pub mode: i32,
    pub sp: i32,
    pub skills: Vec<LearnSkillDto>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TradeDto {
    pub partner: String,
    pub mine: Vec<Entry>,
    pub theirs: Vec<Entry>,
    pub tradable: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QuestDto {
    pub quest_id: i32,
    pub name: String,
    pub step: Option<String>,
    pub cond: i32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StoreDto {
    pub kind: &'static str,
    pub adena: i32,
    pub message: String,
    pub package_sale: bool,
    pub items: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PetItemDto {
    pub id: i32,
    pub item_id: i32,
    pub name: String,
    pub add_name: String,
    pub count: i32,
    pub equipped: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PetDto {
    pub name: String,
    pub level: i32,
    pub hp: [i32; 2],
    pub mp: [i32; 2],
    pub fed: [i32; 2],
    pub exp: [i64; 2],
    pub load: [i32; 2],
    pub stats: Vec<(String, i32)>,
    pub items: Vec<PetItemDto>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WindowsDto {
    pub shop: Option<ShopDto>,
    pub multisell: Option<MultisellDto>,
    pub warehouse: Option<WarehouseDto>,
    pub learn: Option<LearnDto>,
    pub trade: Option<TradeDto>,
    pub quests: Vec<QuestDto>,
    pub store: Option<StoreDto>,
    pub pet: Option<PetDto>,
}

fn entry(names: &dyn Names, item_id: i32) -> Entry {
    Entry { object_id: 0, item_id, name: names.item(item_id), add_name: names.item_add_name(item_id), grade: names.item_grade(item_id), count: 0, enchant: 0, price: 0, store_price: 0 }
}

fn trade_entry(names: &dyn Names, item: &l2_net::game::TradeItem) -> Entry {
    Entry { object_id: item.object_id, count: item.count, enchant: i32::from(item.enchant), ..entry(names, item.item_id) }
}

fn owned(world: &World, item_id: i32) -> i64 {
    world.inventory.items.iter().filter(|item| item.item_id == item_id).map(|item| i64::from(item.count)).sum()
}

fn shop(world: &World, names: &dyn Names) -> Option<ShopDto> {
    let (buy, sell) = (world.windows.buy.as_ref(), world.windows.sell.as_ref());
    if buy.is_none() && sell.is_none() {
        return None;
    }
    let side = |list: &l2_net::game::ShopList| ShopSide { list_id: list.list_id, items: list.items.iter().map(|priced| Entry { price: priced.price, ..trade_entry(names, &priced.item) }).collect() };
    Some(ShopDto { adena: world.inventory.adena(), buy: buy.map(side), sell: sell.map(side) })
}

fn multisell(world: &World, names: &dyn Names) -> Option<MultisellDto> {
    let state = world.windows.multisell.as_ref()?;
    let piece = |item_id: i32, count: i32, enchant: i32| Piece { item_id, name: names.item(item_id), add_name: names.item_add_name(item_id), grade: names.item_grade(item_id), count, enchant, have: owned(world, item_id) };
    let offers = state
        .offers
        .iter()
        .map(|offer| {
            let ingredients: Vec<Piece> = offer.ingredients.iter().map(|ingredient| piece(i32::from(ingredient.item_id), ingredient.count, i32::from(ingredient.enchant))).collect();
            OfferDto {
                entry_id: offer.entry_id,
                stackable: offer.stackable != 0,
                products: offer.products.iter().flatten().map(|product| piece(i32::from(product.item_id), product.count, i32::from(product.enchant))).collect(),
                can: ingredients.iter().all(|ingredient| ingredient.have >= i64::from(ingredient.count)),
                ingredients,
            }
        })
        .collect();
    Some(MultisellDto { list_id: state.list_id, finished: state.finished, adena: world.inventory.adena(), offers })
}

fn warehouse(world: &World, names: &dyn Names) -> Option<WarehouseDto> {
    let list = world.windows.warehouse.as_ref()?;
    Some(WarehouseDto { deposit: world.windows.warehouse_deposit, adena: list.adena, items: list.items.iter().map(|entry| trade_entry(names, &entry.item)).collect() })
}

fn learn(world: &World, names: &dyn Names) -> Option<LearnDto> {
    let list = world.windows.learnable.as_ref()?;
    Some(LearnDto { mode: list.mode, sp: world.stats.sp, skills: list.skills.iter().map(|skill| LearnSkillDto { id: skill.id, name: names.skill(skill.id, skill.next_level), next_level: skill.next_level, sp_cost: skill.sp_cost }).collect() })
}

fn trade(world: &World, names: &dyn Names) -> Option<TradeDto> {
    let partner = world.trade.partner_id?;
    let name = world.entity(partner).map_or_else(|| format!("#{partner}"), |entity| entity.name.clone());
    let list = |items: &[l2_net::game::TradeItem]| items.iter().map(|item| trade_entry(names, item)).collect();
    Some(TradeDto { partner: name, mine: list(&world.trade.mine), theirs: list(&world.trade.theirs), tradable: list(&world.trade.tradable) })
}

fn store(world: &World, names: &dyn Names) -> Option<StoreDto> {
    let stores = &world.stores;
    let sell_item = |item: &l2_net::game::StoreSellItem| Entry { object_id: item.object_id, count: item.count, enchant: i32::from(item.enchant), price: item.price, store_price: item.store_price, ..entry(names, item.item_id) };
    let buy_item = |item: &l2_net::game::StoreBuyItem| Entry { object_id: item.object_id, count: item.count, enchant: i32::from(item.enchant), price: item.price, store_price: item.store_price, ..entry(names, item.item_id) };
    let adena = world.inventory.adena();
    if let Some(manage) = &stores.manage_sell {
        return Some(StoreDto { kind: "manageSell", adena, message: String::new(), package_sale: manage.package_sale, items: manage.available.iter().map(sell_item).collect() });
    }
    if let Some(manage) = &stores.manage_buy {
        return Some(StoreDto { kind: "manageBuy", adena, message: String::new(), package_sale: false, items: manage.available.iter().map(buy_item).collect() });
    }
    if let Some(store) = &stores.sell {
        return Some(StoreDto { kind: "sell", adena, message: stores.sell_messages.get(&store.object_id).cloned().unwrap_or_default(), package_sale: store.package_sale, items: store.items.iter().map(sell_item).collect() });
    }
    if let Some(store) = &stores.buy {
        return Some(StoreDto { kind: "buy", adena, message: stores.buy_messages.get(&store.object_id).cloned().unwrap_or_default(), package_sale: false, items: store.items.iter().map(buy_item).collect() });
    }
    None
}

fn pet(world: &World, names: &dyn Names) -> Option<PetDto> {
    let pet = world.pet.as_ref()?;
    let info = &pet.info;
    Some(PetDto {
        name: info.name.clone(),
        level: info.level,
        hp: [info.hp, info.max_hp],
        mp: [info.mp, info.max_mp],
        fed: [info.fed, info.max_fed],
        exp: [i64::from(info.exp - info.exp_this_level), i64::from(info.exp_next_level - info.exp_this_level)],
        load: [info.load, info.max_load],
        stats: [("P.Atk", info.p_atk), ("M.Atk", info.m_atk), ("P.Def", info.p_def), ("M.Def", info.m_def), ("Accuracy", info.accuracy), ("Evasion", info.evasion), ("Critical", info.critical), ("SP", info.sp)].iter().map(|(label, value)| ((*label).to_owned(), *value)).collect(),
        items: world.pet_inventory.items.iter().map(|item| PetItemDto { id: item.object_id, item_id: item.item_id, name: names.item(item.item_id), add_name: names.item_add_name(item.item_id), count: item.count, equipped: item.equipped }).collect(),
    })
}

pub fn windows(world: &World, names: &dyn Names) -> WindowsDto {
    WindowsDto {
        shop: shop(world, names),
        multisell: multisell(world, names),
        warehouse: warehouse(world, names),
        learn: learn(world, names),
        trade: trade(world, names),
        quests: world.windows.quests.iter().map(|quest| QuestDto { quest_id: quest.quest_id, name: names.quest(quest.quest_id).unwrap_or_else(|| format!("Quest #{}", quest.quest_id)), step: names.quest_step(quest.quest_id, quest.cond), cond: quest.cond }).collect(),
        store: store(world, names),
        pet: pet(world, names),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_bot_core::names::IdNames;

    #[test]
    fn an_empty_world_has_no_open_windows() {
        let windows = windows(&World::default(), &IdNames);
        assert!(windows.shop.is_none() && windows.multisell.is_none() && windows.warehouse.is_none());
        assert!(windows.learn.is_none() && windows.trade.is_none() && windows.store.is_none() && windows.pet.is_none());
        assert!(windows.quests.is_empty());
    }

    #[test]
    fn an_open_buy_list_becomes_a_shop_with_prices() {
        let mut world = World::default();
        let item = l2_net::game::TradeItem { type1: 0, object_id: 5, item_id: 57, count: 3, type2: 0, custom_type1: 0, body_part: 0, enchant: 0, custom_type2: 0 };
        world.windows.buy = Some(l2_net::game::ShopList { money: 10, list_id: 9, items: vec![l2_net::game::PricedItem { item, price: 250 }] });
        let shop = windows(&world, &IdNames).shop.unwrap();
        assert_eq!(shop.buy.as_ref().map(|side| side.list_id), Some(9));
        assert_eq!(shop.buy.unwrap().items[0].price, 250);
        assert!(shop.sell.is_none());
    }
}
