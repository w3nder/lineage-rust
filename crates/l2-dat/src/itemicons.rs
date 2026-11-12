use crate::armorgrp::{self, ICONS, item_head};
use crate::etcitemgrp;
use crate::weapongrp;
use crate::{DatReader, Error};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Weapon,
    Armor,
    Etc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemIcon {
    pub id: u32,
    pub kind: ItemKind,
    pub drop: armorgrp::ItemDrop,
    pub icons: [String; ICONS],
    pub weight: u32,
    pub type1: u32,
}

impl ItemIcon {
    pub fn texture(&self, index: usize) -> Option<String> {
        texture_name(self.icons.get(index)?)
    }
}

pub fn texture_name(icon: &str) -> Option<String> {
    let name = icon.trim();
    name.contains('.').then(|| name.to_string())
}

pub fn read_as(body: &[u8], kind: ItemKind, chronicle: crate::Chronicle) -> Result<Vec<ItemIcon>, Error> {
    read_with(body, kind, chronicle)
}

pub fn read(body: &[u8], kind: ItemKind) -> Result<Vec<ItemIcon>, Error> {
    read_with(body, kind, crate::Chronicle::C4)
}

fn read_with(body: &[u8], kind: ItemKind, chronicle: crate::Chronicle) -> Result<Vec<ItemIcon>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count)
        .map(|_| {
            let head = item_head(&mut reader)?;
            let icon =
                ItemIcon { id: head.id, kind, drop: head.drop.clone(), icons: head.icons.clone(), weight: head.weight, type1: head.type1 };
            match kind {
                ItemKind::Weapon => {
                    weapongrp::body_as(&mut reader, head.id, chronicle)?;
                }
                ItemKind::Armor => {
                    armorgrp::body(&mut reader, head.id)?;
                }
                ItemKind::Etc => {
                    etcitemgrp::record(&mut reader, head)?;
                }
            }
            Ok(icon)
        })
        .collect()
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemIcons {
    pub items: HashMap<u32, ItemIcon>,
}

impl ItemIcons {
    pub fn insert_all(&mut self, icons: Vec<ItemIcon>) {
        for icon in icons {
            self.items.insert(icon.id, icon);
        }
    }

    pub fn get(&self, id: u32) -> Option<&ItemIcon> {
        self.items.get(&id)
    }

    pub fn texture(&self, id: u32) -> Option<String> {
        self.get(id)?.texture(0)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_names_are_used_as_texture_paths_when_they_name_a_package() {
        assert_eq!(texture_name("icon.weapon_small_sword_i00"), Some("icon.weapon_small_sword_i00".into()));
        assert_eq!(texture_name(" scotti.BRONZESOLDIERBADGE "), Some("scotti.BRONZESOLDIERBADGE".into()));
        assert_eq!(
            texture_name("viRUS_Share_Skill2.viRUS_Icon.l2_11023_skill_i00"),
            Some("viRUS_Share_Skill2.viRUS_Icon.l2_11023_skill_i00".into())
        );
        assert_eq!(texture_name(""), None);
        assert_eq!(texture_name("skill0003"), None);
    }

    #[test]
    fn armor_and_weapon_fixtures_yield_their_first_icon() {
        let armors = read(include_bytes!("../tests/fixtures/armorgrp-head.bin"), ItemKind::Armor).unwrap();
        assert_eq!(armors.len(), 3);
        assert_eq!((armors[0].id, armors[0].kind), (21, ItemKind::Armor));
        assert!(!armors[0].icons[0].is_empty());
        assert_eq!(armors[0].texture(0), texture_name(&armors[0].icons[0]));
        let weapons = read(include_bytes!("../tests/fixtures/weapongrp-head.bin"), ItemKind::Weapon).unwrap();
        assert!(!weapons.is_empty());
        assert_eq!(weapons[0].kind, ItemKind::Weapon);
        assert!(weapons[0].icons[0].to_lowercase().starts_with("icon."));
        let mut table = ItemIcons::default();
        table.insert_all(armors);
        table.insert_all(weapons);
        assert_eq!(table.len(), table.items.len());
        assert!(table.texture(21).is_some());
        assert_eq!(table.texture(999_999), None);
    }

    #[test]
    fn etc_items_are_read_through_the_same_head() {
        let mut body = 1u32.to_le_bytes().to_vec();
        body.extend(crate::etcitemgrp::tests::adena_record());
        let icons = read(&body, ItemKind::Etc).unwrap();
        assert_eq!(icons[0].id, 57);
        assert_eq!(icons[0].texture(0), Some("icon.etc_adena_i00".into()));
        assert_eq!(icons[0].texture(1), None);
        assert!(read(&body[..body.len() - 2], ItemKind::Etc).is_err());
    }
}
