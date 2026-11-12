use crate::armorgrp::{Armor, body_part};
use crate::weapongrp::{Weapon, anim_type};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Equipped {
    pub head: i32,
    pub rhand: i32,
    pub lhand: i32,
    pub gloves: i32,
    pub chest: i32,
    pub legs: i32,
    pub feet: i32,
    pub back: i32,
    pub lrhand: i32,
    pub hair: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Item<'a> {
    Armor(&'a Armor),
    Weapon(&'a Weapon),
}

#[derive(Debug, Clone, Copy)]
pub struct Items<'a> {
    pub armors: &'a HashMap<u32, Armor>,
    pub weapons: &'a HashMap<u32, Weapon>,
}

impl<'a> Items<'a> {
    pub fn find(&self, id: i32) -> Option<Item<'a>> {
        let id = u32::try_from(id).ok().filter(|&id| id > 0)?;
        self.weapons.get(&id).map(Item::Weapon).or_else(|| self.armors.get(&id).map(Item::Armor))
    }

    pub fn armor(&self, id: i32) -> Option<&'a Armor> {
        match self.find(id)? {
            Item::Armor(armor) => Some(armor),
            Item::Weapon(_) => None,
        }
    }

    pub fn weapon(&self, id: i32) -> Option<&'a Weapon> {
        match self.find(id)? {
            Item::Weapon(weapon) => Some(weapon),
            Item::Armor(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubMesh {
    Boots,
    Legs,
    Body,
    Gloves,
    HairB,
    HairA,
    Helm,
    Face,
    RightHand,
    LeftHand,
    RightArm,
    LeftArm,
    Cloak,
}

pub fn item_class_id(items: &Items<'_>, equipped: &Equipped, slot: SubMesh) -> i32 {
    let chest_part = items.armor(equipped.chest).map(|a| a.body_part);
    let id = match slot {
        SubMesh::Boots if chest_part == Some(body_part::ALLDRESS) => equipped.chest,
        SubMesh::Boots => equipped.feet,
        SubMesh::Legs if matches!(chest_part, Some(body_part::ONEPIECE | body_part::ALLDRESS)) => equipped.chest,
        SubMesh::Legs => equipped.legs,
        SubMesh::Body => equipped.chest,
        SubMesh::Gloves if chest_part == Some(body_part::ALLDRESS) => equipped.chest,
        SubMesh::Gloves => equipped.gloves,
        SubMesh::Helm if equipped.hair > 0 => equipped.hair,
        SubMesh::Helm => equipped.head,
        SubMesh::HairB | SubMesh::HairA | SubMesh::Face => 0,
        SubMesh::RightHand => {
            let id = if equipped.rhand > 0 { equipped.rhand } else { equipped.lrhand };
            weapon_with(items, id, |w| w.handness != anim_type::BOW)
        }
        SubMesh::LeftHand => left_hand(items, equipped),
        SubMesh::RightArm => {
            let id = if equipped.lrhand > 0 { equipped.lrhand } else { equipped.rhand };
            weapon_with(items, id, |w| w.handness == anim_type::HAND)
        }
        SubMesh::LeftArm => weapon_with(items, equipped.lhand, |w| w.handness == anim_type::HAND),
        SubMesh::Cloak => equipped.back,
    };
    id.max(0)
}

fn left_hand(items: &Items<'_>, equipped: &Equipped) -> i32 {
    let Equipped { rhand, lhand, lrhand, .. } = *equipped;
    if lrhand > 0 && lhand > 0 {
        weapon_with(items, lrhand, |w| w.handness == anim_type::BOW)
    } else if lhand > 0 {
        weapon_with(items, lhand, |w| w.handness == anim_type::ONE_HAND_SWORD)
    } else if lrhand > 0 {
        weapon_with(items, lrhand, |w| matches!(w.handness, anim_type::BOW | anim_type::DUAL | anim_type::DUAL_FIST))
    } else {
        weapon_with(items, rhand, |w| w.handness == anim_type::BOW)
    }
}

fn weapon_with(items: &Items<'_>, id: i32, accept: impl Fn(&Weapon) -> bool) -> i32 {
    if id < 1 {
        return 0;
    }
    match items.weapon(id) {
        Some(weapon) if accept(weapon) => id,
        _ => 0,
    }
}

pub fn weapon_anim_type(items: &Items<'_>, equipped: &Equipped) -> u32 {
    let right = items.weapon(item_class_id(items, equipped, SubMesh::RightHand));
    let left = items.weapon(item_class_id(items, equipped, SubMesh::LeftHand));
    let shield = items.weapon(item_class_id(items, equipped, SubMesh::LeftArm));
    match (right, left) {
        (Some(right), Some(left)) => match (right.handness, left.handness) {
            (anim_type::HAND, _) => anim_type::HAND,
            (anim_type::DUAL, anim_type::DUAL) => anim_type::DUAL,
            (anim_type::DUAL_FIST, anim_type::DUAL_FIST) => anim_type::DUAL_FIST,
            _ => anim_type::ONE_HAND_SWORD,
        },
        (Some(right), None) => match right.handness {
            anim_type::HAND => anim_type::HAND,
            h @ (anim_type::TWO_HAND_SWORD | anim_type::POLE | anim_type::THROW) => h,
            _ => anim_type::ONE_HAND_SWORD,
        },
        (None, Some(left)) if left.handness == anim_type::BOW => anim_type::BOW,
        (None, Some(_)) => anim_type::ONE_HAND_SWORD,
        (None, None) if shield.is_some_and(|s| s.handness == anim_type::HAND) => anim_type::ONE_HAND_SWORD,
        (None, None) => anim_type::HAND,
    }
}

pub fn weapon_curvature(items: &Items<'_>, equipped: &Equipped) -> Option<u32> {
    let right = items.weapon(item_class_id(items, equipped, SubMesh::RightHand));
    let left = items.weapon(item_class_id(items, equipped, SubMesh::LeftHand));
    match (right, left) {
        (Some(right), None) if right.handness == anim_type::THROW => Some(right.curvature),
        (None, Some(left)) if left.handness == anim_type::BOW => Some(left.curvature),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::armorgrp::ArmorRow;

    fn armor(id: u32, body_part: u32) -> Armor {
        Armor { id, drop_sound:String::new(), equip_sound:String::new(), boots_sounds:vec![], body_part, rows: vec![ArmorRow::default(); 15], npc_meshes: vec![], npc_textures: vec![], damage_effect: String::new(), attributes_58c: [0; 6], crystallizable: false }
    }

    fn weapon(id: u32, body_part: u32, handness: u32) -> Weapon {
        Weapon {
            drop_sound:String::new(),
            equip_sound:String::new(),
            attack_sounds:vec![],
            id,
            body_part,
            handness,
            meshes: vec![format!("W.w{id}")],
            textures: vec![],
            weapon_type: 0,
            attributes_cc: [0; 3],
            attributes_dc: [0; 10],
            curvature: 1000,
            sub_mesh_skipped: false,
            crest_texture: 0,
            hero_weapon: -1,
            effects: vec![],
            enchant_meshes: vec![],
            crystallizable: false,
        }
    }

    struct Tables {
        armors: HashMap<u32, Armor>,
        weapons: HashMap<u32, Weapon>,
    }

    fn tables() -> Tables {
        let armors = [armor(21, body_part::CHEST), armor(60, body_part::ONEPIECE), armor(6408, body_part::ALLDRESS)];
        let armors = armors.into_iter().map(|a| (a.id, a)).collect();
        let weapons = [
            weapon(1, body_part::RHAND, anim_type::ONE_HAND_SWORD),
            weapon(18, body_part::LHAND, anim_type::HAND),
            weapon(13, body_part::LRHAND, anim_type::BOW),
            weapon(1299, body_part::LRHAND, anim_type::DUAL),
            weapon(70, body_part::LRHAND, anim_type::TWO_HAND_SWORD),
            weapon(15, body_part::LRHAND, anim_type::POLE),
            weapon(253, body_part::LRHAND, anim_type::DUAL_FIST),
            weapon(2507, body_part::RHAND, anim_type::THROW),
        ];
        let weapons = weapons.into_iter().map(|w| (w.id, w)).collect();
        Tables { armors, weapons }
    }

    impl Tables {
        fn items(&self) -> Items<'_> {
            Items { armors: &self.armors, weapons: &self.weapons }
        }
    }

    #[test]
    fn find_tells_armors_from_weapons_and_ignores_empty_ids() {
        let t = tables();
        let items = t.items();
        assert!(matches!(items.find(21), Some(Item::Armor(a)) if a.id == 21));
        assert!(matches!(items.find(1), Some(Item::Weapon(w)) if w.id == 1));
        assert_eq!(items.find(0), None);
        assert_eq!(items.find(-5), None);
        assert_eq!(items.find(9999), None);
        assert!(items.armor(1).is_none());
        assert!(items.weapon(21).is_none());
    }

    #[test]
    fn body_slots_take_the_chest_item_for_onepiece_and_alldress() {
        let t = tables();
        let items = t.items();
        let plain = Equipped { chest: 21, legs: 28, gloves: 48, feet: 35, ..Default::default() };
        assert_eq!(item_class_id(&items, &plain, SubMesh::Body), 21);
        assert_eq!(item_class_id(&items, &plain, SubMesh::Legs), 28);
        assert_eq!(item_class_id(&items, &plain, SubMesh::Gloves), 48);
        assert_eq!(item_class_id(&items, &plain, SubMesh::Boots), 35);
        let onepiece = Equipped { chest: 60, legs: 28, gloves: 48, feet: 35, ..Default::default() };
        assert_eq!(item_class_id(&items, &onepiece, SubMesh::Legs), 60);
        assert_eq!(item_class_id(&items, &onepiece, SubMesh::Gloves), 48);
        assert_eq!(item_class_id(&items, &onepiece, SubMesh::Boots), 35);
        let dress = Equipped { chest: 6408, legs: 28, gloves: 48, feet: 35, ..Default::default() };
        assert_eq!(item_class_id(&items, &dress, SubMesh::Legs), 6408);
        assert_eq!(item_class_id(&items, &dress, SubMesh::Gloves), 6408);
        assert_eq!(item_class_id(&items, &dress, SubMesh::Boots), 6408);
        let none = Equipped::default();
        assert_eq!(item_class_id(&items, &none, SubMesh::Body), 0);
        assert_eq!(item_class_id(&items, &Equipped { feet: -3, ..none }, SubMesh::Boots), 0);
    }

    #[test]
    fn helm_prefers_the_hair_accessory_and_hair_slots_have_no_item() {
        let items = tables();
        let items = items.items();
        assert_eq!(
            item_class_id(&items, &Equipped { head: 41, hair: 5808, ..Default::default() }, SubMesh::Helm),
            5808
        );
        assert_eq!(item_class_id(&items, &Equipped { head: 41, ..Default::default() }, SubMesh::Helm), 41);
        assert_eq!(item_class_id(&items, &Equipped { head: 41, ..Default::default() }, SubMesh::HairA), 0);
        assert_eq!(item_class_id(&items, &Equipped { head: 41, ..Default::default() }, SubMesh::Face), 0);
        assert_eq!(item_class_id(&items, &Equipped { back: 52, ..Default::default() }, SubMesh::Cloak), 52);
    }

    #[test]
    fn hands_follow_get_item_class_id() {
        let t = tables();
        let items = t.items();
        let sword = Equipped { rhand: 1, ..Default::default() };
        assert_eq!(item_class_id(&items, &sword, SubMesh::RightHand), 1);
        assert_eq!(item_class_id(&items, &sword, SubMesh::LeftHand), 0);
        assert_eq!(item_class_id(&items, &sword, SubMesh::LeftArm), 0);
        let sword_shield = Equipped { rhand: 1, lhand: 18, ..Default::default() };
        assert_eq!(item_class_id(&items, &sword_shield, SubMesh::RightHand), 1);
        assert_eq!(item_class_id(&items, &sword_shield, SubMesh::LeftHand), 0);
        assert_eq!(item_class_id(&items, &sword_shield, SubMesh::LeftArm), 18);
        assert_eq!(item_class_id(&items, &sword_shield, SubMesh::RightArm), 0);
        let bow = Equipped { lrhand: 13, ..Default::default() };
        assert_eq!(item_class_id(&items, &bow, SubMesh::RightHand), 0);
        assert_eq!(item_class_id(&items, &bow, SubMesh::LeftHand), 13);
        let bow_arrows = Equipped { lrhand: 13, lhand: 17, ..Default::default() };
        assert_eq!(item_class_id(&items, &bow_arrows, SubMesh::LeftHand), 13);
        let dual = Equipped { lrhand: 1299, ..Default::default() };
        assert_eq!(item_class_id(&items, &dual, SubMesh::RightHand), 1299);
        assert_eq!(item_class_id(&items, &dual, SubMesh::LeftHand), 1299);
        let two_hand = Equipped { lrhand: 70, ..Default::default() };
        assert_eq!(item_class_id(&items, &two_hand, SubMesh::RightHand), 70);
        assert_eq!(item_class_id(&items, &two_hand, SubMesh::LeftHand), 0);
        let armor_in_hand = Equipped { rhand: 21, ..Default::default() };
        assert_eq!(item_class_id(&items, &armor_in_hand, SubMesh::RightHand), 0);
        let shield_only = Equipped { lhand: 18, ..Default::default() };
        assert_eq!(item_class_id(&items, &shield_only, SubMesh::LeftHand), 0);
        assert_eq!(item_class_id(&items, &shield_only, SubMesh::LeftArm), 18);
        let sword_left = Equipped { lhand: 1, ..Default::default() };
        assert_eq!(item_class_id(&items, &sword_left, SubMesh::LeftHand), 1);
    }

    #[test]
    fn anim_type_follows_get_anim_type() {
        let t = tables();
        let items = t.items();
        let at = |equipped: Equipped| weapon_anim_type(&items, &equipped);
        assert_eq!(at(Equipped::default()), anim_type::HAND);
        assert_eq!(at(Equipped { rhand: 1, ..Default::default() }), anim_type::ONE_HAND_SWORD);
        assert_eq!(at(Equipped { rhand: 1, lhand: 18, ..Default::default() }), anim_type::ONE_HAND_SWORD);
        assert_eq!(at(Equipped { lhand: 18, ..Default::default() }), anim_type::ONE_HAND_SWORD);
        assert_eq!(at(Equipped { lrhand: 70, ..Default::default() }), anim_type::TWO_HAND_SWORD);
        assert_eq!(at(Equipped { lrhand: 15, ..Default::default() }), anim_type::POLE);
        assert_eq!(at(Equipped { rhand: 2507, ..Default::default() }), anim_type::THROW);
        assert_eq!(at(Equipped { lrhand: 13, ..Default::default() }), anim_type::BOW);
        assert_eq!(at(Equipped { lrhand: 13, lhand: 17, ..Default::default() }), anim_type::BOW);
        assert_eq!(at(Equipped { lrhand: 1299, ..Default::default() }), anim_type::DUAL);
        assert_eq!(at(Equipped { lrhand: 253, ..Default::default() }), anim_type::DUAL_FIST);
        assert_eq!(at(Equipped { rhand: 21, ..Default::default() }), anim_type::HAND);
        assert_eq!(at(Equipped { rhand: 9999, ..Default::default() }), anim_type::HAND);
    }
}
