use l2_dat::animnames::{self, AnimKind};
use l2_dat::{
    Armor, HairRow, ItemIcons, ItemKind, NpcTable, Weapon, actionname, armorgrp, chargrp, commandname, etcitemgrp,
    hairgrp, helmetgrp, itemicons, itemname, mobskillanimgrp, npcgrp, npcname, questname, skillgrp, skillname,
    systemmsg, weapongrp, zonename, entereventgrp, musicinfo,
};
use std::collections::HashMap;
use std::path::PathBuf;

fn system_file(name: &str) -> Option<Vec<u8>> {
    let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else {
        eprintln!("KILLER_CLIENT_DIR not set; skipping");
        return None;
    };
    let path = root.join("system").join(name);
    let raw = std::fs::read(&path).unwrap();
    Some(l2_crypto::decrypt(&raw, &path).unwrap().into_owned())
}

#[test]
#[ignore]
fn item_drop_headers_10437e40_preserve_every_mesh_and_material_in_all_three_tables() {
    let mut modes = std::collections::BTreeMap::new();
    for (file, kind) in [("weapongrp.dat", ItemKind::Weapon), ("armorgrp.dat", ItemKind::Armor), ("etcitemgrp.dat", ItemKind::Etc)] {
        let bytes = system_file(file).expect("KILLER_CLIENT_DIR");
        let rows = itemicons::read(&bytes, kind).unwrap();
        assert_eq!(rows.len(), u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize);
        let mut with_mesh = 0;
        let mut unqualified = 0;
        for row in &rows {
            let drop = &row.drop;
            *modes.entry((drop.style, drop.parameter_14)).or_insert(0usize) += 1;
            with_mesh += usize::from(drop.meshes.iter().any(|m| !m.is_empty() && !m.eq_ignore_ascii_case("None")));
            for path in drop.meshes.iter().chain(&drop.materials) {
                unqualified += usize::from(!path.is_empty() && !path.eq_ignore_ascii_case("None") && !path.contains('.'));
            }
        }
        eprintln!("{file}: {} registros, {with_mesh} com mesh de drop, {unqualified} nomes sem pacote", rows.len());
        assert!(with_mesh > 0);
    }
    eprintln!("estilo/efeito de drop: {modes:?}");
}

#[test]
#[ignore]
fn symbol_info_load_10441130_matches_every_original_symbol() {
    let bytes = system_file("symbolname-e.dat").expect("KILLER_CLIENT_DIR");
    let rows = l2_dat::symbolname::read(&bytes).unwrap();
    let expected = [
        (1, "smile", "symbol.imoticon001"), (2, "-_-", "symbol.imoticon002"),
        (3, "crying", "symbol.imoticon003"), (4, "rage", "symbol.imoticon004"),
        (5, "heart", "symbol.imoticon005"), (100, "adena", "symbol.adena"),
        (101, "grades", "symbol.grade_s"), (102, "gradea", "symbol.grade_a"),
        (103, "gradeb", "symbol.grade_b"), (104, "gradec", "symbol.grade_c"),
        (105, "graded", "symbol.grade_d"),
    ];
    assert_eq!(rows.len(), expected.len());
    for (row, (id, alias, texture)) in rows.iter().zip(expected) {
        assert_eq!((row.id, row.alias.as_str(), row.texture.as_str(), row.active), (id, alias, texture, 1));
    }
    let symbols: l2_dat::symbolname::Symbols = rows.into_iter().collect();
    for (id, alias, texture) in expected {
        assert_eq!(symbols.by_id(id).unwrap().texture, texture);
        assert_eq!(symbols.by_alias(alias).unwrap().id, id);
    }
    assert!(symbols.by_alias("GradeA").is_none());
    assert!(bytes.ends_with(b"\x0cSafePackage\0"));
}

#[test]
#[ignore]
fn npcgrp_matches_the_c4_export() {
    let Some(body) = system_file("npcgrp.dat") else { return };
    let npcs = npcgrp::read(&body).unwrap();
    assert_eq!(npcs.len(), 6346);
    assert_eq!(npcs[0].id, 1);
    assert_eq!(npcs[0].class, "LineageMonster.gremlin");
    assert_eq!(npcs[3000].id, 7861);
    assert_eq!(npcs[3000].class, "LineageNPC.a_patriarch_MElf");
    assert_eq!(npcs[3000].mesh, "LineageNPCs.a_patriarch_MElf_m00");
    assert_eq!(npcs[3000].textures.len(), 4);
    assert_eq!(npcs[3000].textures[1], "LineageNPCsTex.a_patriarch_MElf_m00_t00_b01");
    assert_eq!(npcs[6345].id, 10128);
    assert_eq!(npcs[6345].mesh, "LineageNPCs_l2Killer02.viRUS_NPC01_m00");
    let table: NpcTable = npcs.into_iter().collect();
    assert_eq!(table.len(), 6340);
    assert_eq!(table.get(7).unwrap().mesh, "LineageMonsters.walking_fungus_m00");
    assert_eq!(table.get(9).unwrap().mesh, "LineageMonsters.orc_m00");
    let lector = table.get(7001).unwrap();
    assert_eq!(lector.mesh, "LineageNPCs.a_traderA_MHuman_m00");
    assert_eq!(lector.speed, 1.0);
    assert_eq!((lector.level_min, lector.level_max), (3, 3));
    assert_eq!(lector.effect, "LineageEffect.p_u002_a");
    assert_eq!(table.get(579).unwrap().mesh, "lineageMonsters_L2Killer.viRUS_Hunchback_m00");
    assert_eq!(table.get(580).unwrap().mesh, "lineageMonsters_L2Killer.viRUS_Dragon_m00");
    assert_eq!(table.get(581).unwrap().mesh, "lineageMonsters_L2Killer.viRUS_Sakum_m00");
    assert_eq!(table.get(10004).unwrap().mesh, "lineageMonsters_L2Killer.viRUS_Balok_m00");
    assert_eq!(table.get(10007).unwrap().mesh, "lineageMonsters_L2Killer.viRUS_Spezion_m00");
    assert_eq!(table.get(10013).unwrap().mesh, "lineageMonsters_L2Killer.viRUS_Trasken_m00");
    assert!(table.get(30001).is_none());
}

#[test]
#[ignore]
fn chargrp_has_fifteen_rows() {
    let Some(body) = system_file("chargrp.dat") else { return };
    let rows = chargrp::read(&body).unwrap();
    assert_eq!(rows.len(), 15);
    assert_eq!(rows[0].face_meshes, ["Fighter.MFighter_m000_f"]);
    assert_eq!(rows[7].face_icon, "SEK.cbui30");
    assert_eq!(rows[7].face_meshes, ["Elf.FElf_m000_f"]);
    assert_eq!(rows[7].body_textures[0], "FElf.FElf_m000_t1000_u");
    assert_eq!(rows[12].face_meshes, ["Shaman.MShaman_m000_f"]);
    assert_eq!(rows[12].body_textures[0], "MShaman.MShaman_m000_t1000_u");
    assert_eq!(rows[13].face_meshes, ["Shaman.FShaman_m000_f"]);
    assert!(rows.iter().take(14).all(|row| row.walk_anim_frame == 12));
    assert_eq!(rows[14].face_icon, "");
    assert!(rows[14].face_meshes.is_empty());
    assert_eq!(rows[14].body_meshes, [""; 4]);
    let index = chargrp::index_for(3, 0, 49).unwrap();
    assert_eq!(rows[index].face_meshes, ["Shaman.MShaman_m000_f"]);
    assert_eq!(rows[chargrp::index_for(0, 1, 97).unwrap()].face_meshes, ["Magic.FMagic_m000_f"]);
}

#[test]
#[ignore]
fn npcname_matches_the_c4_export() {
    let Some(body) = system_file("npcname-e.dat") else { return };
    let names = npcname::read(&body).unwrap();
    assert_eq!(names.len(), 6361);
    assert_eq!((names[0].id, names[0].name.as_str()), (28086, "Lavier"));
    assert_eq!((names[3000].id, names[3000].name.as_str(), names[3000].title.as_str()), (7850, "Aiken", "Master"));
    assert_eq!(names[3000].color, [0x9c, 0xe8, 0xa9]);
    assert_eq!((names[6360].id, names[6360].name.as_str()), (35711, "DM Maker G"));
    let green_fungus = names.iter().find(|n| n.id == 7).unwrap();
    assert_eq!((green_fungus.name.as_str(), green_fungus.title.as_str()), ("Green Fungus", ""));
    let roxxy = names.iter().find(|n| n.id == 7006).unwrap();
    assert_eq!((roxxy.name.as_str(), roxxy.title.as_str()), ("Roxxy", "Gatekeeper"));
    assert_eq!(names.iter().filter(|n| n.id == 28086).count(), 2);
    let by_id = l2_dat::by_id(names, |n| n.id);
    assert_eq!(by_id.len(), 6350);
    assert_eq!((by_id[&28086].name.as_str(), by_id[&28086].title.as_str()), ("Almir Revenger", ""));
    assert_eq!(by_id[&28086].color, [0x9c, 0xe8, 0xa9]);
}

#[test]
#[ignore]
fn anim_names_of_the_three_int_files() {
    let Some(warrior) = system_file("lineagewarrior.int") else { return };
    let warrior = animnames::parse(&animnames::decode_text(&warrior));
    assert_eq!(warrior.len(), 14);
    assert_eq!(warrior["mfighter"].name(AnimKind::Wait, 0), Some("wait_Hand_MFighter"));
    assert_eq!(warrior["fshaman"].name(AnimKind::Run, 5), Some("run_Bow_FShaman"));
    assert_eq!(warrior["fshaman"].name(AnimKind::DeathWait, 0), Some("DeathWait_FShaman"));
    let monster = animnames::parse(&animnames::decode_text(&system_file("lineagemonster.int").unwrap()));
    assert!(monster.len() >= 520, "{}", monster.len());
    assert_eq!(monster["gremlin"].name(AnimKind::Walk, 0), Some("walk"));
    assert_eq!(monster["gremlin"].name(AnimKind::AtkWait, 0), Some("atkwait"));
    assert!(monster.values().filter(|s| s.name(AnimKind::Wait, 0) == Some("wait")).count() >= 500);
    let npc = animnames::parse(&animnames::decode_text(&system_file("lineagenpc.int").unwrap()));
    assert!(npc.len() >= 100, "{}", npc.len());
    assert_eq!(npc["a_tradera_mhuman"].name(AnimKind::DeathWait, 0), Some("DeathWait_Hand"));
    assert_eq!(npc["a_tradera_mhuman"].name(AnimKind::Wait, 0), Some("wait_Hand"));
}

#[test]
#[ignore]
fn systemmsg_matches_the_c4_export() {
    let Some(body) = system_file("systemmsg-e.dat") else { return };
    let messages = systemmsg::read(&body).unwrap();
    assert_eq!(messages.len(), 1689);
    assert!(messages.iter().enumerate().all(|(i, m)| m.id as usize == i));
    assert_eq!(messages[0].text, "You have been disconnected from the server.");
    assert!(messages[1].text.starts_with("The server will be disconnected"));
    assert_eq!(messages[1].color, [0, 0, 0xee]);
    assert_eq!(messages[1].sound, "ItemSound3.sys_broadcast");
    assert_eq!(messages[13].sound, "ItemSound3.sys_impossible");
    assert_eq!(messages[13].second_sound, "");
    assert_eq!(messages[1000].text, "The Innadril pleasure boat will leave in one minute.");
    assert_eq!(messages[1000].color, [0x79, 0x9b, 0xb0]);
    assert_eq!(messages[1000].sound, "");
    assert!(messages[1688].text.starts_with("You cannot practice enchanting"));
}

#[test]
#[ignore]
fn hairgrp_matches_the_c4_export() {
    let Some(body) = system_file("hairgrp.dat") else { return };
    let rows = hairgrp::read(&body).unwrap();
    assert_eq!(rows.len(), 15);
    assert_eq!(rows[0].ah[0], 0);
    assert_eq!(rows[0].bh[3], 3);
    assert_eq!(rows[0].ah[3], -1);
    assert_eq!(rows[2].ah[0], -1);
    assert_eq!(rows[2].bh[0], 0);
    assert_eq!(rows[13].bh[6], 6);
    assert_eq!(rows[13].ah[6], -1);
    assert_eq!(rows[14], HairRow { ah: [0; 15], bh: [0; 15] });
    assert!(rows.iter().take(14).all(|row| row.bh[9] == 9 && row.ah[9] == -1));
}

#[test]
#[ignore]
fn armorgrp_matches_the_c4_export() {
    let Some(body) = system_file("armorgrp.dat") else { return };
    let armors = armorgrp::read(&body).unwrap();
    assert_eq!(armors.len(), 1652);
    let by_id: HashMap<u32, Armor> = l2_dat::by_id(armors, |a| a.id);
    let tunic = &by_id[&21];
    assert_eq!(tunic.body_part, armorgrp::body_part::CHEST);
    assert_eq!(tunic.mesh(0, 0), Some("Fighter.MFighter_m001_u"));
    assert_eq!(tunic.texture(0, 0), Some("MFighter.MFighter_m001_t02_u"));
    assert_eq!(tunic.mesh(9, 0), Some("Magic.FMagic_m003_u"));
    let boots = &by_id[&35];
    assert_eq!(boots.body_part, armorgrp::body_part::FEET);
    assert_eq!(boots.mesh(0, 0), Some("Fighter.MFighter_m001_b"));
    assert_eq!(boots.texture(0, 0), Some("MFighter.MFighter_m001_t02_b"));
    let cloak = &by_id[&52];
    assert_eq!(cloak.body_part, armorgrp::body_part::BACK);
    assert_eq!(cloak.mesh(0, 0), Some("LineageWeapons.cotton_cloak_m00_mt"));
    assert_eq!(cloak.npc_meshes, ["LineageWeapons.cotton_cloak_m00_mt"]);
    let helmet = &by_id[&41];
    assert_eq!(helmet.body_part, armorgrp::body_part::HEAD);
    assert_eq!(helmet.mesh(0, 0), None);
}

#[test]
#[ignore]
fn weapongrp_matches_the_c4_export() {
    let Some(body) = system_file("weapongrp.dat") else { return };
    let weapons = weapongrp::read(&body).unwrap();
    assert_eq!(weapons.len(), 1102);
    let by_id: HashMap<u32, Weapon> = l2_dat::by_id(weapons, |w| w.id);
    let sword = &by_id[&1];
    assert_eq!((sword.body_part, sword.handness, sword.weapon_type), (7, 1, 1));
    assert_eq!(sword.meshes, ["LineageWeapons.small_sword_m00_wp"]);
    assert_eq!(sword.textures, ["LineageWeaponsTex.small_sword_t00_wp"]);
    let shield = &by_id[&18];
    assert_eq!((shield.body_part, shield.handness, shield.weapon_type), (8, 0, 0));
    assert_eq!(shield.meshes, ["LineageWeapons.leather_shield_m00_sh"]);
    let dual = &by_id[&1299];
    assert_eq!((dual.body_part, dual.handness, dual.weapon_type), (14, 3, 8));
    assert_eq!(dual.meshes, ["LineageWeapons.sword_of_delusion_m00_wp", "LineageWeapons.sword_of_magic_m00_wp"]);
    assert_eq!(
        dual.textures,
        ["LineageWeaponsTex.sword_of_delusion_t00_wp", "LineageWeaponsTex.sword_of_magic_t00_wp"]
    );
    let rod = &by_id[&6529];
    assert!(rod.sub_mesh_skipped);
    let skipped: Vec<u32> = by_id.values().filter(|w| w.sub_mesh_skipped).map(|w| w.id).collect();
    assert_eq!(skipped.len(), 16);
    assert!(skipped.iter().all(|id| {
        let mesh = &by_id[id].meshes[0];
        mesh.contains("fishing_rod") || mesh.contains("draconic_bow")
    }));
}

#[test]
#[ignore]
fn item_attribute_slots_match_every_serialized_client_record() {
    let armor_bytes = system_file("armorgrp.dat").expect("KILLER_CLIENT_DIR");
    let weapon_bytes = system_file("weapongrp.dat").expect("KILLER_CLIENT_DIR");
    let mut armor_reader = l2_dat::DatReader::new(&armor_bytes);
    let armor_count = armor_reader.u32().unwrap();
    assert_eq!(armor_count, 1652);
    for _ in 0..armor_count {
        let head = armorgrp::item_head(&mut armor_reader).unwrap();
        let armor = armorgrp::body(&mut armor_reader, head.id).unwrap();
        let end = armor_bytes.len() - armor_reader.remaining();
        let raw: Vec<i32> = armor_bytes[end - 24..end].as_chunks::<4>().0.iter()
            .map(|word| i32::from_le_bytes(*word)).collect();
        assert_eq!(armor.attributes_58c.as_slice(), raw, "armor {}", head.id);
    }
    let mut weapon_reader = l2_dat::DatReader::new(&weapon_bytes);
    let weapon_count = weapon_reader.u32().unwrap();
    assert_eq!(weapon_count, 1102);
    for _ in 0..weapon_count {
        let head = armorgrp::item_head(&mut weapon_reader).unwrap();
        let start = weapon_bytes.len() - weapon_reader.remaining();
        let weapon = weapongrp::body(&mut weapon_reader, head.id).unwrap();
        let mut prefix = l2_dat::DatReader::new(&weapon_bytes[start..]);
        prefix.u32().unwrap();
        prefix.u32().unwrap();
        for _ in 0..3 {
            prefix.unicode_array().unwrap();
        }
        for _ in 0..3 {
            prefix.unicode().unwrap();
        }
        let attributes = weapon_bytes.len() - prefix.remaining();
        let words: Vec<i32> = weapon_bytes[attributes..attributes + 56].as_chunks::<4>().0.iter()
            .map(|word| i32::from_le_bytes(*word)).collect();
        assert_eq!(weapon.attributes_cc.as_slice(), &words[..3], "weapon {}", head.id);
        assert_eq!(weapon.weapon_type, words[3] as u32);
        assert_eq!(weapon.attributes_dc.as_slice(), &words[4..], "weapon {}", head.id);
    }
}

#[test]
#[ignore]
fn helmetgrp_has_two_helmet_tables() {
    let Some(body) = system_file("helmetgrp.dat") else { return };
    let helmets = helmetgrp::read(&body).unwrap();
    assert_eq!(helmets.iter().map(|h| h.helmet_index).collect::<Vec<_>>(), [1, 2]);
    assert_eq!(helmets[0].rows[0].bh(9), None);
    assert_eq!(helmets[0].rows[0].ah(4), Some(4));
}

#[test]
#[ignore]
fn etcitemgrp_matches_the_c4_export() {
    let Some(body) = system_file("etcitemgrp.dat") else { return };
    let items = etcitemgrp::read(&body).unwrap();
    assert_eq!(items.len(), 6372);
    assert_eq!((items[0].id, items[0].icons[0].as_str()), (17, "icon.etc_wooden_quiver_i00"));
    assert_eq!((items[0].weight, items[0].stackable, items[0].family), (6, 2, 2));
    assert_eq!(items[0].meshes, ["LineageWeapons.wooden_arrow_m00_et"]);
    assert_eq!(items[0].item_sound, "ItemSound.itemdrop_arrow");
    let adena = items.iter().find(|i| i.id == 57).unwrap();
    assert_eq!(adena.icons[0], "icon.etc_adena_i00");
    assert_eq!((adena.stackable, adena.family, adena.grade), (3, 0, 0));
    assert_eq!(items[6371].id, 25006);
    assert_eq!(items[6371].icons[0], "scotti.BRONZESOLDIERBADGE");
    let icons = itemicons::read(&body, ItemKind::Etc).unwrap();
    assert_eq!(icons.len(), items.len());
    assert!(icons.iter().zip(&items).all(|(a, b)| a.id == b.id && a.icons == b.icons));
}

#[test]
#[ignore]
fn item_icons_cover_the_three_groups() {
    let (Some(weapons), Some(armors), Some(etc)) =
        (system_file("weapongrp.dat"), system_file("armorgrp.dat"), system_file("etcitemgrp.dat"))
    else {
        return;
    };
    let mut table = ItemIcons::default();
    table.insert_all(itemicons::read(&weapons, ItemKind::Weapon).unwrap());
    table.insert_all(itemicons::read(&armors, ItemKind::Armor).unwrap());
    table.insert_all(itemicons::read(&etc, ItemKind::Etc).unwrap());
    assert!(table.len() >= 9000, "{}", table.len());
    assert_eq!(table.texture(1146).as_deref(), Some("icon.armor_t01_u_i00"));
    assert_eq!(table.texture(2369).as_deref(), Some("icon.weapon_squires_sword_i00"));
    assert_eq!(table.texture(57).as_deref(), Some("icon.etc_adena_i00"));
    assert_eq!(table.get(2369).unwrap().kind, ItemKind::Weapon);
    assert!(table.items.values().filter(|i| i.texture(0).is_some()).count() >= table.len() - 20);
    assert_eq!(table.texture(244), None);
}

#[test]
#[ignore]
fn itemname_matches_the_c4_export() {
    let Some(body) = system_file("itemname-e.dat") else { return };
    let names = itemname::read(&body).unwrap();
    assert_eq!(names.len(), 8729);
    assert_eq!((names[0].id, names[0].name.as_str(), names[0].popup), (17, "Wooden Arrow", -1));
    assert_eq!(names[0].description, "An arrow made of wood. It is an arrow used for a no grade bow.");
    assert_eq!((names[1].id, names[1].name.as_str()), (57, "Adena"));
    assert_eq!((names[8728].id, names[8728].name.as_str(), names[8728].add_name.as_str()), (25006, "Bronze", "Killer"));
}

#[test]
#[ignore]
fn skillgrp_matches_the_c4_export() {
    let Some(body) = system_file("skillgrp.dat") else { return };
    let skills = skillgrp::read(&body).unwrap();
    assert_eq!(skills.len(), 26351);
    let first = &skills[0];
    assert_eq!((first.id, first.level, first.mp_consume, first.cast_range, first.cast_style), (3, 1, 10, 40, 3));
    assert_eq!(first.hit_time, 1.08);
    assert_eq!((first.ani_char.as_str(), first.icon.as_str()), ("S", "icon.skill0003"));
    assert_eq!((first.rumble_self, first.rumble_target), (9, 11));
    assert_eq!(skills[3].mp_consume, 13);
    let last = &skills[26350];
    assert_eq!((last.id, last.level), (10615, 1));
    assert_eq!(last.icon, "viRUS_Share_Skill2.viRUS_Icon.l2_11023_skill_i00");
    let table = skillgrp::by_id_level(skills);
    assert_eq!(table.len(), 26263);
    assert_eq!(table[&(1177, 1)].icon, "icon.skill1177");
    assert_eq!(table[&(3, 9)].icon, "icon.skill0003");
    assert!(table.values().all(|s| s.icon.contains('.')));
    let wind_walk = table.lookup(1204, 2).unwrap();
    assert_eq!((wind_walk.icon.as_str(), wind_walk.is_debuff()), ("icon.skill1204", false));
    let sleep = table.lookup(1069, 1).unwrap();
    assert_eq!((sleep.extra_eff, sleep.is_debuff(), sleep.cast_range), (1, true, 600));
    assert_eq!(table.lookup(4, 2).unwrap().mp_consume, 41);
    assert_eq!(table.lookup(10615, 1).unwrap().icon, "viRUS_Share_Skill2.viRUS_Icon.l2_11023_skill_i00");
    assert!(table.icons().all(|icon| icon.contains('.')));
}

#[test]
#[ignore]
fn zonename_matches_the_c4_export() {
    let Some(body) = system_file("zonename-e.dat") else { return };
    let zones = zonename::read(&body).unwrap();
    assert_eq!(zones.len(), 315);
    assert_eq!((zones[0].id, zones[0].name.as_str()), (0, "Neutral Waterways"));
    assert_eq!(zones.iter().filter(|z| z.is_territory()).count(), 19);
    let giran: Vec<&str> = zones.iter().filter(|z| (z.grid_x, z.grid_y) == (22, 22)).map(|z| z.name.as_str()).collect();
    assert_eq!(giran, ["Arena", "Giran Castle Town", "Tanor Canyon", "Breka's Stronghold"]);
    let town = zones.iter().find(|z| z.name == "Giran Castle Town").unwrap();
    assert_eq!((town.id, town.color_id), (43, 255));
    assert!(zones.iter().all(|z| z.top_z >= z.bottom_z));
}

#[test]
#[ignore]
fn mobskillanimgrp_matches_the_c4_export() {
    let Some(body) = system_file("mobskillanimgrp.dat") else { return };
    let rows = mobskillanimgrp::read(&body).unwrap();
    assert_eq!(rows.len(), 4899);
    let first = &rows[0];
    assert_eq!((first.npc_id, first.skill_id, first.seq_name.as_str()), (12783, 122, "spatk02"));
    assert_eq!((first.skill_name.as_str(), first.npc_name.as_str()), ("s_hex10", "tamed_kukaburo_war"));
    assert_eq!(first.npc_class, "LineageMonster2.pet_ostrich");
    let last = &rows[4898];
    assert_eq!((last.npc_id, last.skill_id, last.seq_name.as_str()), (10483, 4788, "atk01"));
    assert_eq!(
        (last.skill_name.as_str(), last.npc_class.as_str()),
        ("s_life_chant_boss_a_1c_1", "LineageMonster.silhouette")
    );
    assert_eq!(rows.iter().filter(|r| r.seq_name == "atk01").count(), 2964);
    assert_eq!(rows.iter().filter(|r| r.seq_name == "spatk01").count(), 1334);
    let table = mobskillanimgrp::by_npc_skill(rows);
    assert_eq!(table.len(), 4899);
    assert_eq!(table[&(12787, 122)], "spatk01");
}

#[test]
#[ignore]
fn skillname_matches_the_c4_export() {
    let Some(body) = system_file("skillname-e.dat") else { return };
    let names = skillname::read(&body).unwrap();
    assert_eq!(names.len(), 25461);
    assert_eq!((names[0].id, names[0].level, names[0].name.as_str()), (3, 1, "Power Strike"));
    assert!(names[0].description.ends_with("Power 25."));
    assert_eq!(names[0].desc_add1, "none");
    assert_eq!((names[25460].id, names[25460].level, names[25460].name.as_str()), (4066, 13, "Respect"));
    let table = skillname::by_id_level(names);
    assert_eq!(table[&(1177, 1)].name, "Wind Strike");
}

#[test]
#[ignore]
fn actionname_matches_the_c4_export() {
    let Some(body) = system_file("actionname-e.dat") else { return };
    let actions = actionname::read(&body).unwrap();
    assert_eq!(actions.len(), 88);
    let sit = &actions[0];
    assert_eq!(
        (sit.id, sit.kind, sit.category, sit.category2.as_slice()),
        (0, -1, actionname::CATEGORY_BASIC, &[-2][..])
    );
    assert_eq!((sit.cmd.as_str(), sit.icon.as_str(), sit.name.as_str()), ("sitstand", "icon.action001", "Sit/Stand"));
    assert_eq!(sit.desc, "Toggle Sit/Stand mode. (/sit, /stand)");
    assert_eq!((actions[2].id, actions[2].name.as_str()), (2, "Attack"));
    assert_eq!((actions[7].id, actions[7].category), (7, actionname::CATEGORY_PARTY));
    assert_eq!((actions[12].id, actions[12].kind, actions[12].category), (12, 2, actionname::CATEGORY_SOCIAL));
    assert_eq!((actions[15].id, actions[15].category), (15, actionname::CATEGORY_PET));
    assert_eq!((actions[87].id, actions[87].category, actions[87].cmd.as_str()), (1030, 5, "skill7002"));
    assert_eq!(actions[87].icon, "icon.action108");
    assert!(actions.iter().all(|a| a.icon.starts_with("icon.")));
}

#[test]
#[ignore]
fn questname_matches_the_c4_export() {
    let Some(body) = system_file("questname-e.dat") else { return };
    let quests = questname::read(&body).unwrap();
    assert_eq!(quests.len(), 1816);
    assert_eq!((quests[0].id, quests[0].state, quests[0].title.as_str()), (1, 1, "Letters of Love"));
    assert_eq!(quests[1].sub_name, "Roxxy's Kerchief");
    assert_eq!(quests[0].target_name, "Gatekeeper Roxxy");
    assert_eq!((quests[0].level, quests[0].quest_level, quests[0].quest_type), (2, 5, 3));
    assert_eq!(quests[0].start_npc_id, 7048);
    assert_eq!(quests[0].requirement, "No Requirements");
    assert!(quests.iter().all(|q| q.valid == 1));
    let table = questname::by_id_state(quests);
    assert_eq!(table[&(215, 11)].title, "Trial of the Pilgrim");
    assert_eq!(table[&(215, 12)].target_name, "Andellia");
}

#[test]
#[ignore]
fn logongrp_matches_the_c4_export() {
    let Some(body) = system_file("logongrp.dat") else { return };
    let table = l2_dat::logongrp::read(&body).unwrap();
    assert_eq!(table.slots[0], l2_dat::Spot { x: -16622.0, y: 238671.0, z: -6256.0, yaw: -26624.0 });
    assert_eq!(table.slots[3], l2_dat::Spot { x: -16716.0, y: 238721.0, z: -6256.0, yaw: -16384.0 });
    assert_eq!(table.slots[6], l2_dat::Spot { x: -16820.0, y: 238668.0, z: -6256.0, yaw: -4096.0 });
    assert_eq!(table.center, l2_dat::Spot { x: -16709.0, y: 238583.0, z: -6256.0, yaw: -15360.0 });
    assert_eq!(table.stage[0], l2_dat::Spot { x: -14819.0, y: 243940.0, z: -6407.0, yaw: 47848.0 });
    assert_eq!(table.stage[4], l2_dat::Spot { x: -8797.0, y: 247045.0, z: -6431.0, yaw: -78728.0 });
    assert_eq!(table.stage[17], l2_dat::Spot { x: -21915.0, y: 240266.0, z: -6330.0, yaw: 21504.0 });
    assert_eq!(&body[body.len() - 13..], b"\x0cSafePackage\0");
}

#[test]
#[ignore]
fn servername_matches_the_c4_export() {
    let Some(body) = system_file("servername-e.dat") else { return };
    let rows = l2_dat::servername::read(&body).unwrap();
    assert_eq!(rows.len(), 127);
    assert_eq!((rows[0].id, rows[0].name.as_str()), (0, "Killer 70x"));
    assert_eq!((rows[1].id, rows[1].name.as_str()), (1, "Honor 25x"));
    assert_eq!((rows[2].id, rows[2].name.as_str()), (2, "Killer 150x"));
    assert_eq!((rows[3].id, rows[3].name.as_str()), (3, "New Killer"));
    assert_eq!(rows[126].name, "---");
}

#[test]
#[ignore]
fn classinfo_matches_the_c4_export() {
    let Some(body) = system_file("classinfo-e.dat") else { return };
    let rows = l2_dat::classinfo::read(&body).unwrap();
    assert_eq!(rows.len(), 10);
    assert_eq!(rows[0].text, "Select a race and occupation.");
    assert!(rows[1].text.starts_with("\"Even if my heart should cease to beat"));
    assert!(rows[1].text.contains("\n\nFighters possess"));
    assert!(rows[9].text.contains("Dwarv") || rows[9].text.contains("Dwarf"), "{}", rows[9].text);
}

#[test]
#[ignore]
fn commandname_matches_the_c4_export_and_the_macro_stepper_token_groups() {
    let Some(body) = system_file("commandname-e.dat") else { return };
    let rows = commandname::read(&body).unwrap();
    assert_eq!(rows.len(), 110);
    assert_eq!(rows[0], l2_dat::CommandName { id: 1, token: 0, name: "loc".into() });
    assert_eq!(rows[109], l2_dat::CommandName { id: 110, token: 109, name: "olympiadstat".into() });
    let map = commandname::by_name(rows);
    // Grupo "resolve nome" (FUN_100ab3b6, 0x100ab100 no nwindow.dll): target/targetnext/assist.
    assert_eq!(map.get("target"), Some(&0xc));
    assert_eq!(map.get("targetnext"), Some(&0xd));
    assert_eq!(map.get("assist"), Some(&0xf));
    // Grupo "sem espera extra" (mesmo tratamento de kind==1/skill): useskill*, pickup.
    assert_eq!(map.get("pickup"), Some(&0xe));
    assert_eq!(map.get("useskill"), Some(&0x18));
    assert_eq!(map.get("useskillforce"), Some(&0x19));
    assert_eq!(map.get("useskillstand"), Some(&0x1a));
    // Grupo "lê dois argumentos numéricos" (FUN_100ab236): useshortcutforce/useshortcutstand.
    assert_eq!(map.get("useshortcutforce"), Some(&0x1c));
    assert_eq!(map.get("useshortcutstand"), Some(&0x1d));
    assert_eq!(map.get("delay"), Some(&0x4c));
    assert_eq!(map.get("TARGET"), None, "a chave é sempre minúscula");
}

/// `hennagrp-e.dat` inteiro: 180 linhas, uma por `symbol_id` de 1 a 180, lidas na ordem do
/// serializador do registro (`0x104329f0`, chamado pelo `FL2GameData::HennaDataLoad @ 0x10458ab0`).
/// O arquivo é 15 blocos de 12 linhas; cada bloco fixa um par (bônus, penalidade) e repete os
/// mesmos 12 pares de atributos, com `dye_item_id = 4444 + symbol_id` e `symbol_description`
/// igual ao `symbol_add_name`.
#[test]
#[ignore]
fn henna_data_load_10458ab0_matches_every_original_row() {
    let Some(body) = system_file("hennagrp-e.dat") else { return };
    let rows = l2_dat::hennagrp::read(&body).unwrap();
    let stats = [
        ("Str", "str", "Strength"), ("Con", "con", "Constitution"), ("Dex", "dex", "Dexterity"),
        ("Int", "int", "Intelligence"), ("Men", "men", "Mental Strength"), ("Wit", "wit", "Wit"),
    ];
    let pairs = [(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1), (3, 4), (3, 5), (4, 3), (4, 5), (5, 3), (5, 4)];
    let blocks = [
        (1, 3, false), (1, 2, false), (1, 1, false), (1, 3, true), (1, 2, true), (2, 4, true),
        (2, 3, true), (3, 5, true), (3, 4, true), (1, 1, true), (4, 6, true), (4, 5, true),
        (2, 2, true), (3, 3, true), (4, 4, true),
    ];
    assert_eq!(rows.len(), blocks.len() * pairs.len());
    for (index, row) in rows.iter().enumerate() {
        let (plus, minus, greater) = blocks[index / pairs.len()];
        let (up, down) = pairs[index % pairs.len()];
        let (short, slug, long) = stats[up];
        let text = format!("{short}+{plus} {}-{minus}", stats[down].0);
        let prefix = if greater { "Greater " } else { "" };
        let suffix = if greater { "i01" } else { "i00" };
        assert_eq!(
            row,
            &l2_dat::HennaGrp {
                symbol_id: index as u32 + 1,
                dye_item_id: index as u32 + 4445,
                name: format!("{prefix}Symbol of {long}"),
                icon: format!("icon.etc_{slug}_symbol_{suffix}"),
                description: text.clone(),
                add_name: text,
            }
        );
    }
    assert!(body.ends_with(b"\x0cSafePackage\0"));
}

/// `Recipe-c.dat` inteiro pelo serializador `0x10449440`: o arquivo é consumido até o rodapé
/// `SafePackage` que o `CheckSafePackage` confere, cada `id` é único, e todo produto, material e item de receita existe no
/// `itemname-e.dat` do mesmo cliente. O item de receita (`recipe_id`) é um etcitem da família
/// receita.
#[test]
#[ignore]
fn recipe_data_load_10462870_reads_every_original_row() {
    let Some(body) = system_file("Recipe-c.dat") else { return };
    let rows = l2_dat::recipe::read(&body).unwrap();
    let mut reader = l2_dat::DatReader::new(&body);
    let count = reader.u32().unwrap() as usize;
    assert_eq!(rows.len(), count);
    let consumed: usize = 4 + rows.iter().map(|row| row.name.len() + 2 + 32 + 8 * row.materials.len()).sum::<usize>();
    let tail = &body[consumed..];
    assert_eq!(tail, b"\x0cSafePackage\0", "só o rodapé de CheckSafePackage sobra");
    let names: std::collections::HashMap<_, _> = l2_dat::itemname::read(&system_file("itemname-e.dat").unwrap()).unwrap()
        .into_iter().map(|row| (row.id as i32, row.name)).collect();
    let etc: std::collections::HashMap<_, _> = l2_dat::etcitemgrp::read(&system_file("etcitemgrp.dat").unwrap()).unwrap()
        .into_iter().map(|row| (row.id as i32, row.family)).collect();
    let mut ids = std::collections::HashSet::new();
    for row in &rows {
        assert!(ids.insert(row.id), "id repetido {}", row.id);
        assert!(names.contains_key(&row.product_id), "{} produto {}", row.name, row.product_id);
        assert_eq!(etc.get(&row.recipe_id), Some(&l2_dat::etcitemgrp::ETCITEM_TYPE_RECIPE), "{} receita {}", row.name, row.recipe_id);
        assert!(!row.materials.is_empty() && row.materials.len() <= 20, "{}", row.name);
        for material in &row.materials {
            assert!(names.contains_key(&material.item_id), "{} material {}", row.name, material.item_id);
            assert!(material.count > 0);
        }
        assert!(row.product_num > 0 && (0..=100).contains(&row.success_rate) && row.level > 0);
    }
    eprintln!("Recipe-c.dat: {} receitas", rows.len());
}

#[test]
#[ignore]
fn castlename_matches_the_c4_file() {
    let Some(body) = system_file("castlename-e.dat") else { return };
    let (castles, tail) = l2_dat::castlename::read_with_tail(&body).unwrap();
    assert_eq!(castles.len(), 38);
    assert_eq!(tail, b"\x0cSafePackage\0", "só o rodapé de CheckSafePackage sobra");
    assert!(castles.iter().enumerate().all(|(i, c)| c.index as usize == i && c.present == 1));
    let gludio = &castles[0];
    assert_eq!((gludio.id, gludio.name.as_str(), gludio.location.as_str(), gludio.description.as_str()), (1, "Gludio Castle", "Gludio", ""));
    let last = &castles[37];
    assert_eq!((last.id, last.name.as_str(), last.location.as_str()), (50, "Sapphire Hall", "Goddard"));
    assert_eq!(last.description, "Clan hall located in Goddard Castle Town");
    assert_eq!(castles[36].name, "Emerald Hall");
}

#[test]
#[ignore]
fn staticobject_matches_the_c4_file() {
    let Some(body) = system_file("staticobject-e.dat") else { return };
    let (objects, tail) = l2_dat::staticobject::read_with_tail(&body).unwrap();
    assert_eq!(objects.len(), 261);
    assert_eq!(tail, b"\x0cSafePackage\0", "só o rodapé de CheckSafePackage sobra");
    assert_eq!((objects[0].id, objects[0].name.as_str()), (17220001, "Clan Hall Door"));
    assert_eq!((objects[100].id, objects[100].name.as_str()), (22250011, "Water Wall"));
    assert_eq!((objects[260].id, objects[260].name.as_str()), (24160024, "Throne"));
    let ids: std::collections::HashSet<_> = objects.iter().map(|o| o.id).collect();
    assert_eq!(ids.len(), 261);
    assert_eq!(objects.iter().filter(|o| o.name == "Clan Hall Door").count(), 64);
}

#[test]
#[ignore]
fn creditgrp_matches_the_c4_file() {
    let bytes = system_file("creditgrp-e.dat").expect("KILLER_CLIENT_DIR");
    assert!(bytes.ends_with(b"\x0cSafePackage\0"));
    let credits = l2_dat::creditgrp::read(&bytes).unwrap();
    assert_eq!(credits.len(), 54);
    assert_eq!(u32::from_le_bytes(bytes[..4].try_into().unwrap()), 54);
    for (index, credit) in credits.iter().enumerate() {
        assert_eq!(credit.id as usize, index);
        assert_eq!(credit.time, 10);
        assert!(credit.html.starts_with("credit_") && credit.html.ends_with(".htm"), "{}", credit.html);
        assert!(credit.image.starts_with("L2UI_CH3.credit_ch3_"), "{}", credit.image);
        assert!([1, 3, 5, 7, 9].contains(&credit.align), "{}", credit.id);
    }
    let first = &credits[0];
    assert_eq!(
        (first.html.as_str(), first.image.as_str(), first.time, first.align),
        ("credit_ceo.htm", "L2UI_CH3.credit_ch3_01", 10, 5)
    );
    let third = &credits[2];
    assert_eq!(
        (third.html.as_str(), third.image.as_str(), third.align),
        ("credit_design.htm", "L2UI_CH3.credit_ch3_10", 9)
    );
    let last = &credits[53];
    assert_eq!(
        (last.html.as_str(), last.image.as_str(), last.align),
        ("credit_special_02.htm", "L2UI_CH3.credit_ch3_02", 5)
    );
}

#[test]
#[ignore]
fn eula_matches_the_c4_file() {
    let bytes = system_file("eula-e.dat").expect("KILLER_CLIENT_DIR");
    assert_eq!(bytes.len(), 51612);
    assert!(bytes.ends_with(b"\x0cSafePackage\0"));
    let eula = l2_dat::eula::read(&bytes).unwrap();
    assert_eq!(eula.eula.encode_utf16().count(), 25795);
    assert!(eula.eula.starts_with("\r\nLineage II USER AGREEMENT (the \u{201c} Agreement\u{201d}) \r\n"));
    assert!(eula.eula.ends_with("Attn.: Customer Support.\r\n"));
    assert_eq!(eula.test_server, "\r\n");
}

#[test]
#[ignore]
fn entereventgrp_matches_the_c4_file() {
    let Some(body) = system_file("entereventgrp.dat") else { return };
    let (rows, used) = entereventgrp::read_prefix(&body).unwrap();
    assert_eq!(&body[used..], b"\x0cSafePackage\0");
    assert_eq!(rows.len(), 1120);
    assert_eq!(rows.iter().map(|r| r.id).collect::<std::collections::HashSet<_>>().len(), 1120);
    assert!(rows.iter().all(|r| r.reserved.is_empty() && r.spawn_type == 1));
    assert_eq!(rows.iter().filter(|r| r.sound_vol == 250.0 && r.sound_rad == 100.0).count(), 1119);
    assert_eq!(rows.iter().filter(|r| r.sound_vol == 0.0 && r.sound_rad == 0.0).count(), 1);
    assert_eq!(rows.iter().filter(|r| r.is_rise == 1).count(), 1060);
    assert_eq!(rows.iter().filter(|r| r.is_rise == 0).count(), 36);
    assert_eq!(rows.iter().filter(|r| r.is_rise == 2).count(), 24);
    let first = &rows[0];
    assert_eq!(first.id, 212);
    assert_eq!(first.sound_name, "SkillSound4.summon_shot");
    assert_eq!((first.effect_name.as_str(), first.anim_name.as_str()), ("LineageEffect.e_u011_b", "atkwait"));
    let last = rows.last().unwrap();
    assert_eq!((last.id, last.anim_name.as_str()), (12782, "spwait01"));
    let walk = rows.iter().find(|r| r.id == 5257).unwrap();
    assert_eq!((walk.is_rise, walk.effect_name.as_str(), walk.anim_name.as_str()), (2, "LineageEffect.m_u022_b", "Walk"));
}

#[test]
#[ignore]
fn musicinfo_matches_the_c4_file() {
    let Some(body) = system_file("musicinfo.dat") else { return };
    let (rows, used) = musicinfo::read_prefix(&body).unwrap();
    assert_eq!(&body[used..], b"\x0cSafePackage\0");
    assert_eq!(rows.len(), 331);
    assert_eq!(rows.iter().map(|r| r.id).collect::<std::collections::HashSet<_>>().len(), 331);
    assert_eq!(rows[0].id, 0);
    assert_eq!(rows[0].titles, ["INTRO", "OUTRO", "S20_F"]);
    assert_eq!(rows[1].id, 1);
    assert_eq!(rows[1].titles, ["CC_01", "CC_02", "CC_03", "CC_04", "CC_05", "CC_06"]);
    assert_eq!(rows[2].id, 162401);
    assert_eq!(rows[2].titles, ["F07_F", "F07_S01"]);
    let last = rows.last().unwrap();
    assert_eq!((last.id, last.titles.as_slice()), (241404, &["NS09_F".to_string()][..]));
    assert_eq!(rows.iter().map(|r| r.titles.len()).sum::<usize>(), 121 * 2 + 89 + 33 * 3 + 29 * 4 + 16 * 5 + 12 * 6 + 11 * 7 + 12 * 8 + 8 * 10);
}

#[test]
#[ignore]
fn skillsoundgrp_matches_the_c4_file() {
    let Some(body) = system_file("skillsoundgrp.dat") else { return };
    let (rows, tail) = l2_dat::skillsoundgrp::read_with_tail(&body).unwrap();
    assert_eq!(tail, b"\x0cSafePackage\x00");
    assert_eq!(rows.len(), 962);
    assert_eq!(rows.len(), u32::from_le_bytes(body[..4].try_into().unwrap()) as usize);
    let first = &rows[0];
    assert_eq!((first.id, first.level), (1, 1));
    assert_eq!(first.slots[0].spell.name, "SkillSound4.fatal_strike_cast");
    assert_eq!(first.slots[0].shot.name, "SkillSound4.fatal_strike_shot");
    assert!(first.slots[0].explosion.name.is_empty());
    assert_eq!((first.slots[0].spell.volume, first.slots[0].spell.radius), (250.0, 50.0));
    assert_eq!((first.slots[0].shot.volume, first.slots[0].shot.radius), (250.0, 100.0));
    assert!(first.cast.iter().all(String::is_empty));
    let sleep = &rows[1];
    assert_eq!((sleep.id, sleep.level), (2, 1));
    assert_eq!(sleep.slots[0].spell.name, "SkillSound.sleep_cast");
    assert_eq!(sleep.cast[0], "chrsound.m_hfighter_sub");
    assert_eq!(sleep.cast[13], "chrsound.f_orc_magician_type_c");
    assert!(sleep.cast[14].is_empty());
    assert_eq!(sleep.magic[0], "chrsound.m_hfighter_throw");
    assert_eq!(sleep.magic[13], "chrsound.f_orc_magician_throw");
    let last = rows.last().unwrap();
    assert_eq!((last.id, last.level), (4066, 1));
    assert_eq!(last.slots[0].spell.name, "SkillSound.wind_strike_cast");
    assert_eq!(last.slots[0].shot.name, "SkillSound.wind_strike_shot");
    assert_eq!(last.slots[0].explosion.name, "SkillSound.wind_strike_explotion");
    assert_eq!((last.slots[0].explosion.volume, last.slots[0].explosion.radius), (250.0, 80.0));
    assert!(rows.iter().all(|r| (r.cast_volume, r.cast_radius) == (250.0, 50.0)));
    assert_eq!(rows.iter().filter(|r| r.cast.iter().any(|n| !n.is_empty())).count(), 216);
    assert_eq!(rows.iter().filter(|r| r.magic.iter().any(|n| !n.is_empty())).count(), 248);
    assert!(rows.iter().all(|r| r.cast[14].is_empty() && r.magic[14].is_empty()));
    assert_eq!(rows.iter().map(|r| r.id).max(), Some(49666));
}

#[test]
#[ignore]
fn hairaccessarygrp_matches_the_c4_file() {
    let Some(body) = system_file("hairaccessarygrp.dat") else { return };
    let (rows, tail) = l2_dat::hairaccessarygrp::read_with_tail(&body).unwrap();
    assert_eq!(tail, b"\x0cSafePackage\x00");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows.iter().map(|r| r.index).collect::<Vec<_>>(), [0, 1, 2]);
    assert_eq!(rows[0].a[0][..6], [0, 1, 2, -1, 4, -1]);
    assert_eq!(rows[0].b[0][..6], [0, 1, 2, 3, 4, -1]);
    assert_eq!(rows[0].a[1][..6], [0, 1, 2, 3, -1, 5]);
    assert!(rows[1].a[0].iter().all(|&v| v == -1));
    assert_eq!(rows[1].b[0][..6], [0, 1, 2, 3, 4, -1]);
    assert!(rows[2].b[0].iter().all(|&v| v == 9));
    assert!(rows[2].a[0].iter().all(|&v| v == -1));
    assert!(rows.iter().all(|r| r.a[14] == [0; 15] && r.b[14] == [0; 15]));
}

#[test]
#[ignore]
fn skillsound_lookup_follows_the_multimap_order_of_the_engine() {
    let Some(body) = system_file("skillsoundgrp.dat") else { return };
    let (rows, _) = l2_dat::skillsoundgrp::read_with_tail(&body).unwrap();
    let battle_roar = l2_dat::skillsoundgrp::find(&rows, 4180, 1).unwrap();
    assert_eq!(battle_roar.slots[0].spell.name, "SkillSound3.battle_roar_cast");
    assert_eq!(l2_dat::skillsoundgrp::find(&rows, 4180, 99).unwrap().slots[0].spell.name, "SkillSound4.fatal_strike_cast");
    assert_eq!(l2_dat::skillsoundgrp::find(&rows, 4032, 3).unwrap().level, 3);
    assert!(l2_dat::skillsoundgrp::find(&rows, 999_999, 1).is_none());
}

#[test]
#[ignore]
fn the_delete_failure_messages_exist_in_systemmsg() {
    let Some(body) = system_file("systemmsg-e.dat") else { return };
    let messages = systemmsg::read(&body).unwrap();
    for id in [306usize, 541, 540, 428, 429, 430, 431, 432, 433, 434, 435, 436, 437, 438, 439, 440, 441, 442, 443, 444, 445, 446, 447] {
        assert!(!messages[id].text.is_empty(), "{id}");
        eprintln!("{id}: {}", messages[id].text);
    }
}


#[test]
#[ignore]
fn skillgrp_fields_feed_the_skill_tooltip_the_enchant_level_and_the_visual_effect_gate() {
    let Some(body) = system_file("skillgrp.dat") else { return };
    let skills = skillgrp::read(&body).unwrap();
    assert_eq!(skills.len(), 26351);
    let sysstring = l2_dat::sysstring::read(&system_file("sysstring-e.dat").unwrap()).unwrap();
    let text = |id: u32| sysstring.iter().find(|row| row.id == id).map(|row| row.text.clone());
    for id in [
        skillgrp::TIP_TYPE_MAGIC_SYSSTRING,
        skillgrp::TIP_TYPE_OPER_2_SYSSTRING,
        skillgrp::TIP_TYPE_OTHER_SYSSTRING,
        skillgrp::TIP_HP_SYSSTRING,
        skillgrp::TIP_MP_SYSSTRING,
        skillgrp::TIP_RANGE_SYSSTRING,
    ] {
        assert!(text(id).is_some_and(|text| !text.is_empty()), "{id}");
    }
    let mut hp = 0;
    let mut enchanted = 0;
    let mut packages = std::collections::BTreeSet::new();
    for skill in &skills {
        let tip = skill.tip();
        assert_eq!(tip.enchant_level.is_some(), skill.is_ench != 0);
        assert_eq!(tip.display_level(skill.level), if skill.is_ench != 0 { skill.ench_skill_id } else { skill.level });
        let numbers = tip.numbers();
        assert_eq!(numbers.iter().any(|(label, _)| *label == skillgrp::TIP_HP_SYSSTRING), skill.hp_consume != 0);
        assert_eq!(numbers.iter().any(|(label, _)| *label == skillgrp::TIP_MP_SYSSTRING), skill.mp_consume != 0);
        assert_eq!(numbers.iter().any(|(label, _)| *label == skillgrp::TIP_RANGE_SYSSTRING), skill.cast_range >= 0);
        hp += usize::from(skill.hp_consume != 0);
        enchanted += usize::from(skill.is_ench != 0);
        if !skill.visual_effect.is_empty() {
            packages.insert(skill.visual_effect.split('.').next().unwrap().to_ascii_lowercase());
        }
    }
    assert_eq!((hp, enchanted), (426, 13202));
    assert_eq!(packages.iter().map(String::as_str).collect::<Vec<_>>(), ["0", "skill", "virus_share_skill2"]);
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").unwrap()).join("system");
    for package in &packages {
        assert!(!root.join(format!("{package}.u")).exists(), "{package}");
    }
    let mut rumble = std::collections::BTreeSet::new();
    for skill in &skills {
        rumble.insert((skill.rumble_self, skill.rumble_target));
    }
    assert_eq!(rumble.len(), 11);
}

#[test]
#[ignore]
fn actionname_every_row_carries_the_description_the_actions_tab_shows() {
    let Some(body) = system_file("actionname-e.dat") else { return };
    let rows = actionname::read(&body).unwrap();
    assert_eq!(rows.len(), 88);
    assert!(rows.iter().all(|row| !row.desc.is_empty()));
}

#[test]
#[ignore]
fn eula_test_server_text_is_the_alternate_agreement_text() {
    let Some(body) = system_file("eula-e.dat") else { return };
    let eula = l2_dat::eula::read(&body).unwrap();
    assert_eq!(eula.text(true), "\r\n");
    assert_eq!(eula.text(false), eula.eula);
    assert!(eula.eula.chars().count() > 1);
}

#[test]
#[ignore]
fn commandname_token_is_the_value_the_macro_lookup_returns_for_every_row() {
    let Some(body) = system_file("commandname-e.dat") else { return };
    let rows = commandname::read(&body).unwrap();
    assert_eq!(rows.len(), 110);
    let map = commandname::by_name(rows.clone());
    for row in &rows {
        assert!(map.contains_key(&row.name.to_ascii_lowercase()), "{}", row.name);
    }
    assert!(rows.iter().all(|row| row.token == row.id - 1));
}

#[test]
#[ignore]
fn action_commands_resolve_through_the_engine_token_table_into_the_command_verbs() {
    let Some(actions_body) = system_file("actionname-e.dat") else { return };
    let Some(commands_body) = system_file("commandname-e.dat") else { return };
    let actions = actionname::read(&actions_body).unwrap();
    let commands = commandname::read(&commands_body).unwrap();
    let missing: Vec<&str> = actionname::COMMAND_TOKENS
        .iter()
        .filter(|(_, token)| !commands.iter().any(|row| row.token == *token))
        .map(|(name, _)| *name)
        .collect();
    assert!(missing.is_empty(), "{missing:?}");
    let resolved: Vec<(u32, &str, u32)> = actions
        .iter()
        .filter_map(|a| actionname::command_token(&a.cmd).map(|t| (a.id, a.cmd.as_str(), t)))
        .collect();
    eprintln!("{resolved:?}");
    let unresolved: Vec<(u32, &str)> =
        actions.iter().filter(|a| actionname::command_token(&a.cmd).is_none()).map(|a| (a.id, a.cmd.as_str())).collect();
    eprintln!("{unresolved:?}");
    assert_eq!(resolved.len() + unresolved.len(), 88);
}
