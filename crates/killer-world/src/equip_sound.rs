use l2_net::game::{PAPERDOLL_SLOTS, PaperdollSlot};
use std::collections::HashMap;

pub fn diff(
    old: &[PaperdollSlot; PAPERDOLL_SLOTS],
    new: &[PaperdollSlot; PAPERDOLL_SLOTS],
    entries: &mut HashMap<i32, i32>,
) -> Vec<i32> {
    let mut played = Vec::new();
    for (before, after) in old.iter().zip(new) {
        let (old_object, new_object) = (before.object_id, after.object_id);
        if old_object == 0 && new_object == 0 {
            continue;
        }
        let old_entry = entries.get(&old_object).copied();
        let new_entry = entries.get(&new_object).copied();
        if old_entry.is_some() && new_entry.is_some() && old_object == new_object {
            continue;
        }
        if new_object != 0 && new_entry.is_none() {
            entries.insert(new_object, after.item_id);
        }
        match old_entry {
            None => {
                if new_object > 0 && old_object != new_object {
                    played.push(after.item_id);
                }
            }
            Some(old_item) => {
                if new_object < 1 || old_object == new_object {
                    if new_object == 0 {
                        played.push(old_item);
                    }
                } else {
                    played.push(after.item_id);
                }
                entries.remove(&old_object);
            }
        }
    }
    played
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slots(items: &[(usize, i32, i32)]) -> [PaperdollSlot; PAPERDOLL_SLOTS] {
        let mut slots = [PaperdollSlot::default(); PAPERDOLL_SLOTS];
        for &(index, object_id, item_id) in items {
            slots[index] = PaperdollSlot { object_id, item_id };
        }
        slots
    }

    #[test]
    fn the_first_user_info_plays_every_equipped_item_once() {
        let mut entries = HashMap::new();
        let worn = slots(&[(7, 10, 2369), (10, 11, 1146)]);
        let played = diff(&slots(&[]), &worn, &mut entries);
        assert_eq!(played, vec![2369, 1146]);
        assert!(diff(&worn, &worn, &mut entries).is_empty());
    }

    #[test]
    fn swapping_plays_the_new_item_and_unequipping_plays_the_old_one() {
        let mut entries = HashMap::new();
        let first = slots(&[(7, 10, 2369)]);
        diff(&slots(&[]), &first, &mut entries);
        let swapped = slots(&[(7, 12, 5)]);
        assert_eq!(diff(&first, &swapped, &mut entries), vec![5]);
        assert_eq!(diff(&swapped, &slots(&[]), &mut entries), vec![5]);
        assert!(entries.is_empty());
    }

    #[test]
    fn a_removed_slot_without_a_registered_entry_is_silent() {
        let mut entries = HashMap::new();
        assert!(diff(&slots(&[(7, 10, 2369)]), &slots(&[]), &mut entries).is_empty());
    }
}
