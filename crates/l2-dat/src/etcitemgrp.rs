use crate::armorgrp::{ICONS, ItemHead, item_head};
use crate::{DatReader, Error};

pub const TAG_ETC: u32 = 2;
pub const ETCITEM_TYPE_RECIPE: u32 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EtcItem {
    pub id: u32,
    pub icons: [String; ICONS],
    pub weight: u32,
    pub type1: u32,
    pub meshes: Vec<String>,
    pub textures: Vec<String>,
    pub item_sound: String,
    pub equip_sound: String,
    pub stackable: u32,
    pub family: u32,
    pub grade: u32,
    pub crystallizable: bool,
}

pub fn read(body: &[u8]) -> Result<Vec<EtcItem>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count)
        .map(|_| {
            let head = item_head(&mut reader)?;
            record(&mut reader, head)
        })
        .collect()
}

pub fn record(r: &mut DatReader<'_>, head: ItemHead) -> Result<EtcItem, Error> {
    let meshes = r.unicode_array()?;
    let textures = r.unicode_array()?;
    let item_sound = r.unicode()?;
    let equip_sound = r.unicode()?;
    let stackable = r.u32()?;
    let family = r.u32()?;
    let grade = r.u32()?;
    Ok(EtcItem {
        id: head.id,
        icons: head.icons,
        weight: head.weight,
        type1: head.type1,
        meshes,
        textures,
        item_sound,
        equip_sound,
        stackable,
        family,
        grade,
        crystallizable: head.crystallizable,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    fn unicode(s: &str) -> Vec<u8> {
        let mut out = (s.len() as u32 * 2).to_le_bytes().to_vec();
        out.extend(s.encode_utf16().flat_map(u16::to_le_bytes));
        out
    }

    fn u32s(values: &[u32]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    pub(crate) fn adena_record() -> Vec<u8> {
        let mut out = u32s(&[2, 57, 0, 5, 4, 1, 0]);
        for name in ["DropItems.coin_m00", "", "", "DropItemsTex.coin_t00", "DropItemsTex.coin_t01", ""] {
            out.extend(unicode(name));
        }
        for name in ["icon.etc_adena_i00", "", "", "", ""] {
            out.extend(unicode(name));
        }
        out.extend(u32s(&[0, 0, 3, 0, 0]));
        out.extend(u32s(&[1]));
        out.extend(unicode(""));
        out.extend(u32s(&[1]));
        out.extend(unicode(""));
        out.extend(unicode("ItemSound2.itemdrop_etc_money_2"));
        out.extend(unicode(""));
        out.extend(u32s(&[3, 0, 0]));
        out
    }

    #[test]
    fn adena_has_icon_zero_and_is_stackable_family_three() {
        let mut body = u32s(&[1]);
        body.extend(adena_record());
        let items = read(&body).unwrap();
        assert_eq!(items.len(), 1);
        let adena = &items[0];
        assert_eq!(adena.id, 57);
        assert_eq!(adena.icons[0], "icon.etc_adena_i00");
        assert_eq!(adena.icons[1], "");
        assert_eq!((adena.weight, adena.type1), (0, 0));
        assert_eq!(adena.meshes, [""]);
        assert_eq!(adena.textures, [""]);
        assert_eq!(adena.item_sound, "ItemSound2.itemdrop_etc_money_2");
        assert_eq!((adena.stackable, adena.family, adena.grade), (3, 0, 0));
        assert!(read(&body[..body.len() - 1]).is_err());
    }
}
