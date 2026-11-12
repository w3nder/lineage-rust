use l2_net::game::packets::{ADENA_ITEM_ID, ITEM_ADDED, ITEM_MODIFIED, ITEM_REMOVED};
use l2_net::game::{EquipUpdate, InventoryItem, InventoryUpdate, ItemList};

pub const EQUIP_SLOT_NONE: i32 = 0;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Inventory {
    pub items: Vec<InventoryItem>,
    pub loads: u32,
}

impl Inventory {
    pub fn apply_list(&mut self, list: &ItemList) {
        self.items = list.items.clone();
        self.loads += 1;
    }

    pub fn apply_update(&mut self, update: &InventoryUpdate) {
        for change in &update.changes {
            match change.change {
                ITEM_ADDED | ITEM_MODIFIED => self.put(change.item),
                ITEM_REMOVED => self.items.retain(|i| i.object_id != change.item.object_id),
                _ => {}
            }
        }
    }

    pub fn apply_equip(&mut self, update: &EquipUpdate) {
        if let Some(item) = self.items.iter_mut().find(|i| i.object_id == update.object_id) {
            item.equipped = update.change != EQUIP_SLOT_NONE;
        }
    }

    pub fn find(&self, object_id: i32) -> Option<&InventoryItem> {
        self.items.iter().find(|i| i.object_id == object_id)
    }

    pub fn adena(&self) -> i32 {
        self.items.iter().filter(|i| i.item_id == ADENA_ITEM_ID).map(|i| i.count).sum()
    }

    pub fn equipped(&self) -> impl Iterator<Item = &InventoryItem> {
        self.items.iter().filter(|i| i.equipped)
    }

    pub fn equipped_in(&self, body_part: i32) -> Option<&InventoryItem> {
        self.equipped().find(|i| i.body_part == body_part)
    }

    fn put(&mut self, item: InventoryItem) {
        match self.items.iter_mut().find(|i| i.object_id == item.object_id) {
            Some(known) => *known = item,
            None => self.items.push(item),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2_net::game::packets::{BODY_PART_CHEST, BODY_PART_L_HAND, BODY_PART_R_HAND, ITEM_ADDED, ITEM_MODIFIED, ITEM_REMOVED};
    use l2_net::game::{EquipUpdate, InventoryUpdate, ItemChange, ItemList};

    fn item(object_id: i32, item_id: i32, count: i32, equipped: bool, body_part: i32) -> InventoryItem {
        InventoryItem {
            type1: 0,
            object_id,
            item_id,
            count,
            type2: 0,
            custom_type1: 0,
            equipped,
            body_part,
            enchant: 0,
            custom_type2: 0,
        }
    }

    fn stocked() -> Inventory {
        let mut inventory = Inventory::default();
        inventory.apply_list(&ItemList {
            show_window: false,
            items: vec![
                item(1, 57, 2500, false, 0),
                item(2, 2369, 1, true, BODY_PART_R_HAND),
                item(3, 1146, 1, false, BODY_PART_CHEST),
            ],
        });
        inventory
    }

    #[test]
    fn item_list_replaces_everything_and_adena_is_item_57() {
        let mut inventory = stocked();
        assert_eq!(inventory.items.len(), 3);
        assert_eq!(inventory.adena(), 2500);
        inventory.apply_list(&ItemList { show_window: true, items: vec![item(9, 1060, 3, false, 0)] });
        assert_eq!(inventory.items.len(), 1);
        assert_eq!(inventory.adena(), 0);
    }

    #[test]
    fn inventory_update_adds_modifies_and_removes_by_object_id() {
        let mut inventory = stocked();
        inventory.apply_update(&InventoryUpdate {
            changes: vec![
                ItemChange { change: ITEM_ADDED, item: item(4, 1060, 5, false, 0) },
                ItemChange { change: ITEM_MODIFIED, item: item(1, 57, 2000, false, 0) },
                ItemChange { change: ITEM_REMOVED, item: item(3, 1146, 1, false, BODY_PART_CHEST) },
                ItemChange { change: 9, item: item(5, 1, 1, false, 0) },
            ],
        });
        assert_eq!(inventory.items.iter().map(|i| i.object_id).collect::<Vec<_>>(), vec![1, 2, 4]);
        assert_eq!(inventory.adena(), 2000);
        assert_eq!(inventory.find(4).map(|i| i.count), Some(5));
        assert!(inventory.find(3).is_none());
    }

    #[test]
    fn modifying_an_unknown_item_adds_it_and_adding_a_known_one_replaces_it() {
        let mut inventory = stocked();
        inventory.apply_update(&InventoryUpdate {
            changes: vec![
                ItemChange { change: ITEM_MODIFIED, item: item(7, 1060, 1, false, 0) },
                ItemChange { change: ITEM_ADDED, item: item(1, 57, 1, false, 0) },
            ],
        });
        assert_eq!(inventory.items.len(), 4);
        assert_eq!(inventory.adena(), 1);
    }

    #[test]
    fn equipped_in_finds_the_item_by_body_part_mask_only_when_equipped() {
        let mut inventory = stocked();
        assert_eq!(inventory.equipped_in(BODY_PART_R_HAND).map(|i| i.object_id), Some(2));
        assert_eq!(inventory.equipped_in(BODY_PART_CHEST), None);
        inventory.apply_update(&InventoryUpdate {
            changes: vec![ItemChange { change: ITEM_MODIFIED, item: item(3, 1146, 1, true, BODY_PART_CHEST) }],
        });
        assert_eq!(inventory.equipped_in(BODY_PART_CHEST).map(|i| i.item_id), Some(1146));
        assert_eq!(inventory.equipped().count(), 2);
    }

    #[test]
    fn an_equipped_arrow_stack_is_updated_by_inventory_update_modified() {
        let mut inventory = stocked();
        inventory.apply_update(&InventoryUpdate {
            changes: vec![ItemChange { change: ITEM_ADDED, item: item(8, 1341, 100, true, BODY_PART_L_HAND) }],
        });
        for shot in 1..=3 {
            inventory.apply_update(&InventoryUpdate {
                changes: vec![ItemChange { change: ITEM_MODIFIED, item: item(8, 1341, 100 - shot, true, BODY_PART_L_HAND) }],
            });
        }
        let arrows = inventory.find(8).unwrap();
        assert_eq!((arrows.count, arrows.equipped), (97, true));
        assert_eq!(inventory.equipped_in(BODY_PART_L_HAND).map(|i| i.count), Some(97));
        assert_eq!(inventory.items.len(), 4);
        inventory.apply_update(&InventoryUpdate {
            changes: vec![ItemChange { change: ITEM_REMOVED, item: item(8, 1341, 0, true, BODY_PART_L_HAND) }],
        });
        assert!(inventory.find(8).is_none());
    }

    #[test]
    fn equip_update_toggles_the_equipped_flag_of_the_object() {
        let mut inventory = stocked();
        inventory.apply_equip(&EquipUpdate { change: EQUIP_SLOT_NONE, object_id: 2, body_part: 7 });
        assert!(!inventory.find(2).map(|i| i.equipped).unwrap_or(true));
        inventory.apply_equip(&EquipUpdate { change: 1, object_id: 3, body_part: 10 });
        assert!(inventory.find(3).map(|i| i.equipped).unwrap_or(false));
        inventory.apply_equip(&EquipUpdate { change: 1, object_id: 99, body_part: 10 });
        assert_eq!(inventory.items.len(), 3);
    }
}
