use l2_dat::animnames::{AnimKind, AnimSet};
use l2_dat::chargrp;
use l2_dat::equipment::weapon_anim_type;
use l2_dat::{Equipped, Items};
use l2_net::game::packets::{
    EQUIP_BACK, EQUIP_CHEST, EQUIP_FEET, EQUIP_GLOVES, EQUIP_HAIR, EQUIP_HEAD, EQUIP_LEGS, EQUIP_LHAND, EQUIP_LRHAND,
    EQUIP_RHAND, VISIBLE_EQUIP_SLOTS,
};

use crate::world::EntityKind;

pub mod channel;
pub mod names;
pub mod notify;
pub mod player;
pub mod ride;
pub mod skill;

pub use channel::{Channel, SeqInfo, Sequences, Step};
pub use notify::{AttackVoice, EffectNotify, NotifyKind, ScreenFade, SeqNotify, SoundNotify, ViewShake};
pub use names::{FishingType, MoveType, PawnView, Physics, WaitType};
pub use player::{
    Action, AnimEvent, AnimPlayer, Attack, AttackKind, Hit, Magic, SkillData, Tables, Trigger, UpdateMovementAnim,
    WaitChange,
};
pub use skill::{SkillAnim, SkillTimeline, StageSeq, cast_lead, skill_anim, skill_timeline};

const PLAYER_SUFFIXES: [&str; 14] = [
    "MFighter", "FFighter", "MDarkElf", "FDarkElf", "MDwarf", "FDwarf", "MElf", "FElf", "MMagic", "FMagic", "MOrc",
    "FOrc", "MShaman", "FShaman",
];

pub fn player_suffix(class_id: i32, race: i32, sex: i32) -> &'static str {
    chargrp::index_for(race, sex, class_id)
        .and_then(|row| PLAYER_SUFFIXES.get(row))
        .copied()
        .unwrap_or(PLAYER_SUFFIXES[0])
}

pub fn anim_section(kind: &EntityKind, npc_class: Option<&str>) -> Option<String> {
    match kind {
        EntityKind::Player { class_id, race, sex, .. } => {
            Some(player_suffix(*class_id, *race, *sex).to_ascii_lowercase())
        }
        EntityKind::Npc { .. } => npc_class.map(|class| class.rsplit('.').next().unwrap_or(class).to_ascii_lowercase()),
        EntityKind::Item { .. } => None,
    }
}

pub fn sequence_name(set: &AnimSet, kind: AnimKind, weapon: usize) -> Option<&str> {
    set.name(kind, weapon)
}

pub fn equipped_of(visible: &[i32; VISIBLE_EQUIP_SLOTS]) -> Equipped {
    Equipped {
        head: visible[EQUIP_HEAD],
        rhand: visible[EQUIP_RHAND],
        lhand: visible[EQUIP_LHAND],
        gloves: visible[EQUIP_GLOVES],
        chest: visible[EQUIP_CHEST],
        legs: visible[EQUIP_LEGS],
        feet: visible[EQUIP_FEET],
        back: visible[EQUIP_BACK],
        lrhand: visible[EQUIP_LRHAND],
        hair: visible[EQUIP_HAIR],
    }
}

pub fn equipped(kind: &EntityKind) -> Equipped {
    match kind {
        EntityKind::Player { equipped, .. } => equipped_of(equipped),
        EntityKind::Npc { rhand, chest, lhand, .. } => {
            Equipped { rhand: *rhand, chest: *chest, lhand: *lhand, ..Default::default() }
        }
        EntityKind::Item { .. } => Equipped::default(),
    }
}

pub fn weapon_type(kind: &EntityKind, items: &Items<'_>) -> usize {
    match kind {
        EntityKind::Player { .. } => weapon_anim_type(items, &equipped(kind)) as usize,
        EntityKind::Npc { .. } | EntityKind::Item { .. } => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::EntityKind;
    use l2_net::game::packets::VISIBLE_EQUIP_SLOTS;

    fn player(class_id: i32, race: i32, sex: i32) -> EntityKind {
        EntityKind::Player {
            class_id,
            race,
            sex,
            equipped: [0; VISIBLE_EQUIP_SLOTS],
            hair_style: 0,
            hair_color: 0,
            face: 0,
            enchant_effect: 0,
            hero: false,
        }
    }

    #[test]
    fn anim_section_is_the_chargrp_suffix_or_the_npc_class_name() {
        let npc = EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: false };
        assert_eq!(anim_section(&npc, Some("LineageMonster.gremlin")).as_deref(), Some("gremlin"));
        assert_eq!(anim_section(&npc, Some("LineageNPC.a_traderA_MHuman")).as_deref(), Some("a_tradera_mhuman"));
        assert_eq!(anim_section(&npc, Some("gremlin")).as_deref(), Some("gremlin"));
        assert_eq!(anim_section(&npc, None), None);
        assert_eq!(anim_section(&player(0, 0, 0), None).as_deref(), Some("mfighter"));
        assert_eq!(anim_section(&player(97, 0, 1), None).as_deref(), Some("fmagic"));
        assert_eq!(anim_section(&player(18, 1, 1), None).as_deref(), Some("felf"));
        assert_eq!(anim_section(&player(115, 3, 0), None).as_deref(), Some("mshaman"));
        assert_eq!(anim_section(&player(44, 3, 1), None).as_deref(), Some("forc"));
        assert_eq!(anim_section(&player(53, 4, 0), None).as_deref(), Some("mdwarf"));
        assert_eq!(anim_section(&player(31, 2, 0), None).as_deref(), Some("mdarkelf"));
        assert_eq!(player_suffix(0, 9, 0), "MFighter");
    }

    #[test]
    fn equipped_follows_the_char_info_slot_order_and_npcs_only_carry_hands() {
        let visible = [100, 700, 800, 900, 1000, 1100, 1200, 1300, 1400, 1500, 1600];
        let e = equipped_of(&visible);
        assert_eq!(
            e,
            Equipped {
                head: 700,
                rhand: 800,
                lhand: 900,
                gloves: 1000,
                chest: 1100,
                legs: 1200,
                feet: 1300,
                back: 1400,
                lrhand: 1500,
                hair: 1600,
            }
        );
        let mut kind = player(0, 0, 0);
        if let EntityKind::Player { equipped, .. } = &mut kind {
            *equipped = visible;
        }
        assert_eq!(equipped(&kind), e);
        let npc = EntityKind::Npc { npc_id: 1, rhand: 5, chest: 0, lhand: 6, attackable: false };
        assert_eq!(equipped(&npc), Equipped { rhand: 5, lhand: 6, ..Default::default() });
        assert_eq!(equipped(&EntityKind::Item { item_id: 57, count: 1, stackable: true }), Equipped::default());
    }

    #[test]
    fn weapon_type_comes_from_the_weapongrp_for_players_and_stays_hand_for_npcs() {
        use l2_dat::Weapon;
        use l2_dat::weapongrp::anim_type;
        let sword = Weapon {
            id: 1,
            drop_sound: String::new(),
            equip_sound: String::new(),
            attack_sounds: vec![],
            body_part: 7,
            handness: anim_type::ONE_HAND_SWORD,
            meshes: vec![],
            textures: vec![],
            weapon_type: 1,
            attributes_cc: [0; 3],
            attributes_dc: [0; 10],
            curvature: 1000,
            sub_mesh_skipped: false,
            crest_texture: 0,
            hero_weapon: -1,
            effects: vec![],
            enchant_meshes: vec![],
            crystallizable: false,
        };
        let bow = Weapon { id: 13, body_part: 14, handness: anim_type::BOW, ..sword.clone() };
        let armors = std::collections::HashMap::new();
        let weapons = [(1, sword), (13, bow)].into_iter().collect();
        let items = Items { armors: &armors, weapons: &weapons };
        let mut visible = [0; VISIBLE_EQUIP_SLOTS];
        visible[EQUIP_RHAND] = 1;
        let mut kind = player(0, 0, 0);
        if let EntityKind::Player { equipped, .. } = &mut kind {
            *equipped = visible;
        }
        assert_eq!(weapon_type(&kind, &items), 1);
        visible[EQUIP_RHAND] = 0;
        visible[EQUIP_LRHAND] = 13;
        if let EntityKind::Player { equipped, .. } = &mut kind {
            *equipped = visible;
        }
        assert_eq!(weapon_type(&kind, &items), 5);
        assert_eq!(weapon_type(&player(0, 0, 0), &items), 0);
        assert_eq!(
            weapon_type(&EntityKind::Npc { npc_id: 1, rhand: 1, chest: 0, lhand: 0, attackable: false }, &items),
            0
        );
    }

    #[test]
    fn sequence_name_reads_the_table_by_kind_and_weapon_without_fallback() {
        let mut set = AnimSet::default();
        set.names_mut(AnimKind::Wait)[0] = Some("wait_Hand_X".into());
        set.names_mut(AnimKind::Wait)[1] = Some("wait_1HS_X".into());
        set.names_mut(AnimKind::Sit)[0] = Some("Sit_X".into());
        assert_eq!(sequence_name(&set, AnimKind::Wait, 0), Some("wait_Hand_X"));
        assert_eq!(sequence_name(&set, AnimKind::Wait, 1), Some("wait_1HS_X"));
        assert_eq!(sequence_name(&set, AnimKind::Wait, 7), None);
        assert_eq!(sequence_name(&set, AnimKind::Sit, 0), Some("Sit_X"));
        assert_eq!(sequence_name(&set, AnimKind::Sit, 5), None);
        assert_eq!(sequence_name(&AnimSet::default(), AnimKind::Wait, 0), None);
    }
}
