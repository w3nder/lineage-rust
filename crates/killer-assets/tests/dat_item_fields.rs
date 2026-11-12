use killer_assets::client::Client;
use std::path::PathBuf;

fn test_client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

#[test]
#[ignore]
fn equip_sound_follows_the_item_tag_for_every_weapon_armor_and_etc_item() {
    let Some(mut client) = test_client() else { return };
    let tables = client.tables().unwrap();
    let (mut weapons, mut armors, mut etcs) = (0, 0, 0);
    for (id, weapon) in &tables.weapongrp {
        let expected = Some(weapon.equip_sound.as_str()).filter(|n| !n.is_empty() && !n.eq_ignore_ascii_case("None"));
        assert_eq!(tables.equip_sound(*id as i32), expected, "weapon {id}");
        weapons += usize::from(expected.is_some());
    }
    for (id, armor) in &tables.armorgrp {
        if tables.weapongrp.contains_key(id) {
            continue;
        }
        let expected = Some(armor.equip_sound.as_str()).filter(|n| !n.is_empty() && !n.eq_ignore_ascii_case("None"));
        assert_eq!(tables.equip_sound(*id as i32), expected, "armor {id}");
        armors += usize::from(expected.is_some());
    }
    for (id, etc) in &tables.etcitemgrp {
        if tables.weapongrp.contains_key(id) || tables.armorgrp.contains_key(id) {
            continue;
        }
        let expected = Some(etc.equip_sound.as_str()).filter(|n| !n.is_empty() && !n.eq_ignore_ascii_case("None"));
        assert_eq!(tables.equip_sound(*id as i32), expected, "etc {id}");
        etcs += usize::from(expected.is_some());
    }
    eprintln!("equip_sound: weapons {weapons} armors {armors} etc {etcs}");
    assert!(weapons + armors + etcs > 0);
    assert_eq!(tables.equip_sound(0), None);
    assert_eq!(tables.equip_sound(-5), None);
}

#[test]
#[ignore]
fn walk_anim_frames_cover_every_chargrp_row_and_reach_every_player_pawn() {
    let Some(mut client) = test_client() else { return };
    let tables = client.tables().unwrap();
    let frames = tables.walk_anim_frames();
    assert_eq!(frames.len(), tables.chargrp.len());
    for (row, template) in tables.chargrp.iter().enumerate() {
        assert_eq!(frames[&row], template.walk_anim_frame as i32, "row {row}");
    }
    for race in 0..=4 {
        for sex in 0..=1 {
            for class_id in 0..=118 {
                let Some(row) = l2_dat::chargrp::index_for(race, sex, class_id) else { continue };
                assert!(frames.contains_key(&row), "race {race} sex {sex} class {class_id} row {row}");
            }
        }
    }
}

#[test]
#[ignore]
fn weapon_curvature_follows_get_anim_type_for_every_bow_and_thrown_weapon() {
    use l2_dat::Equipped;
    use l2_dat::equipment::{weapon_anim_type, weapon_curvature};
    use l2_dat::weapongrp::anim_type;
    let Some(mut client) = test_client() else { return };
    let tables = client.tables().unwrap();
    let items = tables.items();
    let mut seen = std::collections::BTreeMap::new();
    for (id, weapon) in &tables.weapongrp {
        if tables.armorgrp.contains_key(id) {
            continue;
        }
        let id = *id as i32;
        match weapon.handness {
            anim_type::BOW => {
                let equipped = Equipped { lrhand: id, ..Equipped::default() };
                assert_eq!(weapon_anim_type(&items, &equipped), anim_type::BOW, "{id}");
                assert_eq!(weapon_curvature(&items, &equipped), Some(weapon.curvature), "{id}");
                *seen.entry(("bow", weapon.curvature)).or_insert(0) += 1;
            }
            anim_type::THROW => {
                let equipped = Equipped { rhand: id, ..Equipped::default() };
                assert_eq!(weapon_anim_type(&items, &equipped), anim_type::THROW, "{id}");
                assert_eq!(weapon_curvature(&items, &equipped), Some(weapon.curvature), "{id}");
                *seen.entry(("throw", weapon.curvature)).or_insert(0) += 1;
            }
            anim_type::ONE_HAND_SWORD | anim_type::TWO_HAND_SWORD | anim_type::POLE => {
                let equipped = Equipped { rhand: id, ..Equipped::default() };
                assert_eq!(weapon_curvature(&items, &equipped), None, "{id}");
            }
            _ => {}
        }
    }
    eprintln!("curvature por tipo: {seen:?}");
    assert!(!seen.is_empty());
}
