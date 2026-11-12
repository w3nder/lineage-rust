use l2_dat::armorgrp::{TAG_ARMOR, TAG_WEAPON, body_part};
use l2_dat::equipment::{item_class_id, weapon_anim_type};
use l2_dat::hairgrp::STYLES;
use l2_dat::weapongrp::anim_type;
use l2_dat::{Armor, CharTemplate, Equipped, HairAccessoryLoc, HairRow, Item, Items, Npc, SubMesh, Weapon};
use std::collections::HashMap;

pub const SLOT_BOOTS: u8 = 0;
pub const SLOT_LEGS: u8 = 1;
pub const SLOT_BODY: u8 = 2;
pub const SLOT_GLOVES: u8 = 3;
pub const SLOT_HAIR_B: u8 = 4;
pub const SLOT_HAIR_A: u8 = 5;
pub const SLOT_HELM: u8 = 6;
pub const SLOT_FACE: u8 = 7;
pub const SLOT_RIGHT_HAND: u8 = 8;
pub const SLOT_LEFT_HAND: u8 = 9;
pub const SLOT_RIGHT_ARM: u8 = 10;
pub const SLOT_LEFT_ARM: u8 = 11;
pub const SLOT_CLOAK: u8 = 12;
pub const SLOT_CLOAK_COVER: u8 = 13;
pub const SLOTS: u8 = 14;

pub const RIGHT_HAND_BONE: &str = "Weapon_R_Bone";
pub const LEFT_HAND_BONE: &str = "Weapon_L_Bone";
pub const RIGHT_ARM_BONE: &str = "Shield_R_Bone";
pub const LEFT_ARM_BONE: &str = "Shield_L_Bone";
pub const CAPE_BONE: &str = "Cape_Bone";
pub const LOWBODY_BONE: &str = "bip01_spine";
pub const HEAD_BONE: &str = "Bip01_head";
pub const HAND_ALIAS: &str = "e_hand";
pub const HEAD_GEAR_MESH: &str = "LineageWeapons.HeadGear_m00";
pub const DUAL_FIST_LEFT_ROTATION: [i32; 3] = [0x8000, 0, 0];
pub const DUAL_FIST_SCALE_DIVISOR: f32 = 2.9;
pub const WEAPON_SCALE_REFERENCE: f32 = 207.0;

const PKG: [&str; 14] = [
    "Fighter", "Fighter", "DarkElf", "DarkElf", "Dwarf", "Dwarf", "Elf", "Elf", "Magic", "Magic", "Orc", "Orc",
    "Shaman", "Shaman",
];
const CLS: [&str; 14] = [
    "MFighter", "FFighter", "MDarkElf", "FDarkElf", "MDwarf", "FDwarf", "MElf", "FElf", "MMagic", "FMagic", "MOrc",
    "FOrc", "MShaman", "FShaman",
];

pub enum ClassSource<'a> {
    Player { row: usize },
    Npc { record: Option<&'a Npc> },
}

impl ClassSource<'_> {
    pub fn name(&self) -> Option<String> {
        match self {
            Self::Player { row } => CLS.get(*row).map(|class| format!("LineageWarrior.{class}")),
            Self::Npc { record } => record
                .filter(|npc| !npc.class.eq_ignore_ascii_case("None"))
                .map(|npc| npc.class.clone()),
        }
    }
}
const FDARKELF_ROW: usize = 3;
const FDARKELF_HAIR_U00_BODIES: [&str; 3] = ["FDarkElf_m003_u", "FDarkElf_m006_u", "FDarkElf_m009_u"];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Placement {
    Master,
    Skinned,
    Helm { offset: [f32; 3], rotation: [i32; 3] },
    Attached { bone: &'static str },
    Enchanted { bone: &'static str, scale: [f32; 3], offset: [f32; 3], hero: bool },
    Extra { bone: Option<&'static str> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub slot: u8,
    pub mesh: String,
    pub textures: Vec<Option<String>>,
    pub placement: Placement,
}

impl Part {
    pub fn new(mesh: impl Into<String>, textures: Vec<String>) -> Self {
        Self {
            slot: SLOT_FACE,
            mesh: mesh.into(),
            textures: textures.iter().map(|name| template_string(name)).collect(),
            placement: Placement::Master,
        }
    }

    pub fn is_attached(&self) -> bool {
        matches!(
            self.placement,
            Placement::Helm { .. } | Placement::Attached { .. } | Placement::Enchanted { .. } | Placement::Extra { bone: Some(_) }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeaponEffect {
    pub class: String,
    pub bone: &'static str,
    pub right: bool,
    pub offset: [f32; 3],
    pub size_scale: f32,
    pub velocity_scale: f32,
    pub hero: bool,
}

pub const HERO_AURA_CLASS: &str = "LineageEffect.e_u091_aura";
pub const NPC_DAMAGE_EFFECT: &str = "LineageEffect.p_u002_a";

pub fn damage_effect_name(name: &str) -> Option<String> {
    (!name.is_empty() && !name.eq_ignore_ascii_case("None")).then(|| name.to_owned())
}

#[derive(Debug, Clone, PartialEq)]
pub enum PawnEffect {
    Weapon(WeaponEffect),
    HeroAura,
}

impl PawnEffect {
    pub fn class(&self) -> &str {
        match self {
            Self::Weapon(effect) => &effect.class,
            Self::HeroAura => HERO_AURA_CLASS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Appearance {
    pub class_name: Option<String>,
    pub parts: Vec<Part>,
    pub weapon_type: u32,
    pub cloak: Option<Part>,
    pub enchant_effect: u8,
    pub weapon_effects: Vec<WeaponEffect>,
    pub hero_aura: bool,
    pub damage_effect: Option<String>,
    pub arrow: Option<ArrowItem>,
}

pub const DEFAULT_ARROW_ITEM: i32 = 0x11;
pub const ARROW_ITEM_BY_GRADE: [i32; 6] = [0x11, 0x53d, 0x53e, 0x53f, 0x540, 0x541];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowOwner {
    Npc,
    Pc,
    Own { left_hand_item: Option<i32> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ArrowItem {
    pub bow_item: i32,
    pub arrow_item: i32,
}

pub fn arrow_item_id(items: &Items<'_>, equipped: &Equipped, owner: ArrowOwner) -> ArrowItem {
    let bow_item = if owner == ArrowOwner::Npc { equipped.rhand } else { equipped.lrhand };
    let arrow_item = match owner {
        ArrowOwner::Own { left_hand_item } => left_hand_item.unwrap_or(DEFAULT_ARROW_ITEM),
        ArrowOwner::Npc | ArrowOwner::Pc => match items.find(bow_item) {
            Some(Item::Weapon(weapon)) => {
                usize::try_from(weapon.attributes_dc[0]).ok().and_then(|grade| ARROW_ITEM_BY_GRADE.get(grade)).copied().unwrap_or(DEFAULT_ARROW_ITEM)
            }
            Some(Item::Armor(_)) | None => DEFAULT_ARROW_ITEM,
        },
    };
    ArrowItem { bow_item, arrow_item }
}

impl Appearance {
    pub fn part(&self, slot: u8) -> Option<&Part> {
        self.parts.iter().find(|p| p.slot == slot && !matches!(p.placement, Placement::Extra { .. }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClanCrest {
    pub server_id: i32,
    pub ally_crest_id: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct Tables<'a> {
    pub items: Items<'a>,
    pub hairgrp: &'a [HairRow],
    pub helmetgrp: &'a HashMap<u32, Vec<HairRow>>,
    pub hairaccessarygrp: &'a HashMap<u32, Vec<HairRow>>,
    pub accessory_locs: &'a HashMap<String, HairAccessoryLoc>,
}

#[derive(Debug, Clone, Copy)]
pub struct Player<'a> {
    pub template: &'a CharTemplate,
    pub row: usize,
    pub face: i32,
    pub hair_style: i32,
    pub hair_color: i32,
    pub equipped: Equipped,
    pub tables: Tables<'a>,
    pub mounted: Option<u8>,
    pub korean: bool,
    pub korean_flag: bool,
    pub clan_crest: Option<ClanCrest>,
}

impl<'a> Player<'a> {
    fn items(&self) -> &Items<'a> {
        &self.tables.items
    }

    fn item_class_id(&self, slot: u8) -> i32 {
        sub_mesh(slot).map_or(0, |s| item_class_id(self.items(), &self.equipped, s))
    }

    fn record(&self, slot: u8) -> Option<Item<'a>> {
        let id = self.item_class_id(slot);
        if id < 1 { None } else { self.items().find(id) }
    }

    pub fn have_accessary(&self) -> bool {
        self.equipped.hair > 0
    }

    pub fn helm_mesh_type(&self) -> i32 {
        helm_mesh_type(self.pc_mesh_name(SLOT_HELM).as_deref())
    }

    fn hair_table(&self) -> Option<&'a HairRow> {
        match self.helm_mesh_type() {
            0 => self.tables.hairgrp.get(self.row),
            t if t < 0 => None,
            t => {
                let table = if self.have_accessary() { self.tables.hairaccessarygrp } else { self.tables.helmetgrp };
                table.get(&(t as u32)).and_then(|rows| rows.get(self.row))
            }
        }
    }

    fn hair_index(&self, slot: u8) -> Option<i32> {
        let row = self.hair_table()?;
        let style = usize::try_from(self.hair_style).ok().filter(|&s| s < STYLES)?;
        let value = if slot == SLOT_HAIR_A { row.ah[style] } else { row.bh[style] };
        (value != -1).then_some(value)
    }

    fn body_is_fdarkelf_u00(&self) -> bool {
        self.row == FDARKELF_ROW
            && self
                .pc_mesh_name(SLOT_BODY)
                .is_some_and(|body| FDARKELF_HAIR_U00_BODIES.iter().any(|b| contains_ignore_case(&body, b)))
    }

    pub fn pc_mesh_name(&self, slot: u8) -> Option<String> {
        let row = self.row;
        let mut name: Option<String> = None;
        match self.record(slot) {
            None => match slot {
                SLOT_BOOTS => name = template_string(&self.template.body_meshes[3]),
                SLOT_LEGS => name = template_string(&self.template.body_meshes[1]),
                SLOT_BODY => name = template_string(&self.template.body_meshes[0]),
                SLOT_GLOVES => name = template_string(&self.template.body_meshes[2]),
                SLOT_HAIR_B | SLOT_HAIR_A => {
                    let suffix = if slot == SLOT_HAIR_A { "ah" } else { "bh" };
                    if let (Some(index), Some(pkg), Some(cls)) = (self.hair_index(slot), PKG.get(row), CLS.get(row)) {
                        name = Some(format!("{pkg}.{cls}_m00{index}_m00_{suffix}"));
                    }
                    if row == FDARKELF_ROW
                        && name
                            .as_deref()
                            .is_some_and(|n| contains_ignore_case(n, &format!("FDarkElf_m000_m00_{suffix}")))
                        && self.body_is_fdarkelf_u00()
                    {
                        return Some(format!("DarkElf.FDarkElf_m000_m00_{suffix}_u00"));
                    }
                }
                SLOT_FACE => name = self.template.face_meshes.first().and_then(|m| template_string(m)),
                _ => {}
            },
            Some(Item::Weapon(weapon)) => {
                if !weapon.sub_mesh_skipped {
                    let index = usize::from(weapon.is_dual() && slot == SLOT_LEFT_HAND);
                    name = weapon.mesh(index).map(str::to_string);
                }
            }
            Some(Item::Armor(armor)) => {
                name = armor.mesh(row, armor_mesh_index(armor, slot)).map(str::to_string);
            }
        }
        if slot == SLOT_GLOVES
            && let Some(gloves) = self.elf_bare_gloves()
        {
            return Some(gloves.to_string());
        }
        if self.korean
            && let Some(texture) = self.pc_tex_name(slot, 0)
        {
            for (set, needle) in [("m001", "FDarkElf_m001"), ("m005", "FDarkElf_m005")] {
                if contains_ignore_case(&texture, needle) {
                    match slot {
                        SLOT_LEGS => return Some(format!("DarkElf.FDarkElf_{set}_l")),
                        SLOT_BODY => return Some(format!("DarkElf.FDarkElf_{set}_u")),
                        _ => {}
                    }
                }
            }
        }
        name
    }

    fn elf_bare_gloves(&self) -> Option<&'static str> {
        let body = self.pc_mesh_name(SLOT_BODY)?;
        let gloves_texture = self.pc_tex_name(SLOT_GLOVES, 0).unwrap_or_default();
        for (needle, prefix, gloves) in
            [("MElf_m009_u", "MElf_m009_t", "Elf.MElf_m000_g"), ("FElf_m006_u", "FElf_m006_t", "Elf.FElf_m000_g")]
        {
            if contains_ignore_case(&body, needle) && !self.set_textures_match(prefix, &gloves_texture) {
                return Some(gloves);
            }
        }
        None
    }

    fn set_textures_match(&self, prefix: &str, gloves: &str) -> bool {
        let body = self.pc_tex_name(SLOT_BODY, 0).unwrap_or_default();
        match (find_ignore_case(&body, prefix), find_ignore_case(gloves, prefix)) {
            (Some(a), Some(b)) => {
                let a = &body[a + prefix.len()..];
                let b = &gloves[b + prefix.len()..];
                a.len() >= 2 && b.len() >= 2 && a.as_bytes()[..2] == b.as_bytes()[..2]
            }
            _ => false,
        }
    }

    pub fn pc_tex_num(&self, slot: u8) -> usize {
        if slot == SLOT_BODY || slot == SLOT_LEGS {
            return 1;
        }
        match self.record(slot) {
            Some(Item::Armor(armor)) => armor.rows.get(self.row).map_or(0, |r| r.textures.len()),
            Some(Item::Weapon(weapon)) => weapon.textures.len(),
            None => 1,
        }
    }

    pub fn pc_tex_name(&self, slot: u8, index: usize) -> Option<String> {
        let row = self.row;
        let mut name: Option<String> = None;
        let record = self.record(slot);
        let count = match record {
            Some(Item::Armor(armor)) => armor.rows.get(row).map_or(0, |r| r.textures.len()),
            Some(Item::Weapon(weapon)) => weapon.textures.len(),
            None => 0,
        };
        match record {
            None | Some(_) if record.is_none() || count <= index => match slot {
                _ if index != 0 => {}
                SLOT_BOOTS => name = template_string(&self.template.body_textures[3]),
                SLOT_LEGS => name = template_string(&self.template.body_textures[1]),
                SLOT_BODY => name = template_string(&self.template.body_textures[0]),
                SLOT_GLOVES => name = template_string(&self.template.body_textures[2]),
                SLOT_HAIR_B | SLOT_HAIR_A => {
                    let suffix = if slot == SLOT_HAIR_A { "ah" } else { "bh" };
                    if let (Some(hair), Some(cls)) = (self.hair_index(slot), CLS.get(row)) {
                        name = Some(format!("{cls}.{cls}_m00{hair}_t0{}_m00_{suffix}", self.hair_color));
                    }
                    if row == FDARKELF_ROW && self.body_is_fdarkelf_u00() {
                        for color in 0..4 {
                            let needle = format!("FDarkElf_m000_t0{color}_m00_{suffix}");
                            if name.as_deref().is_some_and(|n| contains_ignore_case(n, &needle)) {
                                return Some(format!("FDarkElf.FDarkElf_m000_t0{color}_m00_{suffix}_u00"));
                            }
                        }
                    }
                }
                SLOT_FACE => {
                    name = usize::try_from(self.face)
                        .ok()
                        .and_then(|f| self.template.face_textures.get(f))
                        .and_then(|t| template_string(t));
                }
                _ => {}
            },
            Some(Item::Weapon(weapon)) => {
                if weapon.is_dual() {
                    name = weapon.texture(usize::from(slot == SLOT_LEFT_HAND)).and_then(template_string);
                } else {
                    name = weapon.texture(index).and_then(template_string);
                    if let Some(crest) = self.clan_crest {
                        let crest_slot = weapon.crest_texture as usize;
                        if crest.server_id > 0 && crest.ally_crest_id > 0 && crest_slot > 0 && index == crest_slot - 1 {
                            name = Some(format!("crest.large.crest_{}_{}_l", crest.server_id, crest.ally_crest_id));
                        }
                    }
                }
            }
            Some(Item::Armor(armor)) => {
                let fixed = armor_mesh_index(armor, slot);
                name = armor.texture(row, if fixed != 0 { fixed } else { index }).and_then(template_string);
            }
            None => {}
        }
        if slot == SLOT_GLOVES
            && let Some(gloves) = self.elf_bare_gloves_texture(name.as_deref().unwrap_or_default())
        {
            return Some(gloves.to_string());
        }
        if self.korean
            && let Some(n) = name.as_deref()
        {
            let rules = [
                ("FDarkElf_m001_t01_l", "FDarkElf.FDarkElf_m005_t32_l"),
                ("FDarkElf_m001_t02_l", "FDarkElf.FDarkElf_m005_t12_l"),
                ("FDarkElf_m001_t01_u", "FDarkElf.FDarkElf_m005_t32_u"),
                ("FDarkElf_m001_t02_u", "FDarkElf.FDarkElf_m005_t12_u"),
            ];
            if let Some((_, replacement)) = rules.iter().find(|(needle, _)| contains_ignore_case(n, needle)) {
                return Some((*replacement).to_string());
            }
            if self.korean_flag
                && let Some(needle) =
                    rules.iter().map(|(needle, _)| needle).find(|needle| contains_ignore_case(n, needle))
            {
                return Some(format!("FDarkElf.{needle}_"));
            }
        }
        name
    }

    fn elf_bare_gloves_texture(&self, gloves_texture: &str) -> Option<&'static str> {
        let body = self.pc_mesh_name(SLOT_BODY)?;
        for (needle, prefix, texture) in [
            ("MElf_m009_u", "MElf_m009_t", "MElf.MElf_m000_t1000_g"),
            ("FElf_m006_u", "FElf_m006_t", "FElf.FElf_m000_t1000_g"),
        ] {
            if contains_ignore_case(&body, needle) && !self.set_textures_match(prefix, gloves_texture) {
                return Some(texture);
            }
        }
        None
    }

    pub fn pc_ex_mesh_name(&self, slot: u8) -> Option<String> {
        match self.record(slot)? {
            Item::Weapon(weapon) if weapon.sub_mesh_skipped => {
                weapon.mesh(usize::from(weapon.is_dual() && slot == SLOT_LEFT_HAND)).map(str::to_string)
            }
            Item::Weapon(_) => None,
            Item::Armor(armor) => {
                armor.rows.get(self.row)?.ex_meshes.get(armor_mesh_index(armor, slot)).and_then(|m| template_string(m))
            }
        }
    }

    pub fn pc_ex_tex_name(&self, slot: u8) -> Option<String> {
        match self.record(slot)? {
            Item::Weapon(weapon) if weapon.sub_mesh_skipped => {
                weapon.texture(usize::from(weapon.is_dual() && slot == SLOT_LEFT_HAND)).and_then(template_string)
            }
            Item::Weapon(_) => None,
            Item::Armor(armor) => armor
                .rows
                .get(self.row)?
                .ex_textures
                .get(armor_mesh_index(armor, slot))
                .and_then(|m| template_string(m)),
        }
    }

    pub fn accessory_placement(&self) -> Placement {
        let mut placement = Placement::Helm { offset: [0.0; 3], rotation: [0; 3] };
        if self.equipped.hair > 0
            && let Some(mesh) = self.pc_mesh_name(SLOT_HELM)
            && let Some(loc) = l2_dat::hairaccessorylocgrp::find(self.tables.accessory_locs, &mesh)
        {
            placement =
                Placement::Helm { offset: loc.offset(self.hair_style), rotation: loc.rotation(self.hair_style) };
        }
        placement
    }

    pub fn set_pawn_resource(&self) -> Appearance {
        let mut appearance = Appearance { class_name: ClassSource::Player { row: self.row }.name(), ..Appearance::default() };
        let accessory = self.accessory_placement();
        for slot in 0..SLOTS {
            if self.mounted.is_some() && (SLOT_RIGHT_HAND..=SLOT_LEFT_ARM).contains(&slot) {
                continue;
            }
            let mesh = if slot == SLOT_HELM && self.mounted == Some(1) {
                Some(HEAD_GEAR_MESH.to_string())
            } else {
                self.pc_mesh_name(slot)
            };
            if let Some(mesh) = mesh {
                let textures = (0..self.pc_tex_num(slot)).map(|i| self.pc_tex_name(slot, i)).collect();
                push_part(&mut appearance, slot, mesh, textures, accessory);
            }
            if let Some(ex_mesh) = self.pc_ex_mesh_name(slot) {
                let textures = vec![self.pc_ex_tex_name(slot)];
                let part =
                    Part { slot, mesh: ex_mesh, textures, placement: Placement::Extra { bone: extra_bone(slot) } };
                appearance.parts.push(part);
            }
        }
        appearance.weapon_type = weapon_anim_type(self.items(), &self.equipped);
        if appearance.weapon_type == anim_type::BOW {
            appearance.arrow = Some(arrow_item_id(self.items(), &self.equipped, ArrowOwner::Pc));
        }
        appearance.damage_effect = self.damage_effect();
        appearance
    }

    pub fn damage_effect(&self) -> Option<String> {
        let name = self.items().armor(self.item_class_id(SLOT_BODY)).map_or(self.template.damage_effect.as_str(), |armor| armor.damage_effect.as_str());
        damage_effect_name(name)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NpcPawn<'a> {
    pub npc: &'a Npc,
    pub equipped: Equipped,
    pub items: Items<'a>,
    pub mounted: bool,
    pub petrified: bool,
}

impl<'a> NpcPawn<'a> {
    pub fn npc_tex_names(&self) -> Vec<Option<String>> {
        let count = self.npc.textures.len();
        (0..count)
            .map(|i| {
                if self.petrified
                    && let Some(t) = self.npc.textures2.get(i).filter(|_| i + 1 < self.npc.textures2.len())
                {
                    return template_string(t);
                }
                self.npc.textures.get(i).and_then(|t| template_string(t))
            })
            .collect()
    }

    fn weapon(&self, slot: u8) -> Option<&'a Weapon> {
        let id = sub_mesh(slot).map_or(0, |s| item_class_id(&self.items, &self.equipped, s));
        if id < 1 {
            return None;
        }
        self.items.weapon(id)
    }

    pub fn set_pawn_resource(&self) -> Appearance {
        let mut appearance = Appearance { class_name: ClassSource::Npc { record: Some(self.npc) }.name(), ..Appearance::default() };
        if let Some(mesh) = template_string(&self.npc.mesh) {
            appearance.parts.push(Part {
                slot: SLOT_FACE,
                mesh,
                textures: self.npc_tex_names(),
                placement: Placement::Master,
            });
        }
        if !self.mounted {
            for slot in SLOT_RIGHT_HAND..=SLOT_LEFT_ARM {
                let Some(weapon) = self.weapon(slot) else { continue };
                let (mesh, textures) = if weapon.is_dual() && slot <= SLOT_LEFT_HAND {
                    let index = usize::from(slot == SLOT_LEFT_HAND);
                    (weapon.mesh(index), vec![weapon.texture(index).and_then(template_string)])
                } else {
                    (weapon.mesh(0), weapon.textures.iter().map(|t| template_string(t)).collect())
                };
                if let Some(mesh) = mesh {
                    push_part(&mut appearance, slot, mesh.to_string(), textures, Placement::Master);
                }
            }
        }
        let chest = item_class_id(&self.items, &self.equipped, SubMesh::Body);
        if chest > 0
            && let Some(armor) = self.items.armor(chest)
            && let Some(mesh) = armor.npc_meshes.first().and_then(|m| template_string(m))
        {
            let textures = armor.npc_textures.iter().map(|t| template_string(t)).collect();
            push_part(&mut appearance, SLOT_BODY, mesh, textures, Placement::Master);
        }
        appearance.weapon_type = weapon_anim_type(&self.items, &self.equipped);
        if appearance.weapon_type == anim_type::BOW {
            appearance.arrow = Some(arrow_item_id(&self.items, &self.equipped, ArrowOwner::Npc));
        }
        appearance.damage_effect = self.damage_effect();
        appearance
    }

    pub fn damage_effect(&self) -> Option<String> {
        let chest = item_class_id(&self.items, &self.equipped, SubMesh::Body);
        let name = self.items.armor(chest).map_or(self.npc.effect.as_str(), |armor| armor.damage_effect.as_str());
        damage_effect_name(name)
    }
}

fn push_part(appearance: &mut Appearance, slot: u8, mesh: String, textures: Vec<Option<String>>, accessory: Placement) {
    let placement = match slot {
        SLOT_FACE => Placement::Master,
        SLOT_HELM => accessory,
        SLOT_RIGHT_HAND => Placement::Attached { bone: RIGHT_HAND_BONE },
        SLOT_LEFT_HAND => Placement::Attached { bone: LEFT_HAND_BONE },
        SLOT_RIGHT_ARM => Placement::Attached { bone: RIGHT_ARM_BONE },
        SLOT_LEFT_ARM => Placement::Attached { bone: LEFT_ARM_BONE },
        SLOT_CLOAK | SLOT_CLOAK_COVER => {
            appearance.cloak = Some(Part { slot, mesh, textures, placement: Placement::Attached { bone: CAPE_BONE } });
            return;
        }
        _ => Placement::Skinned,
    };
    appearance.parts.push(Part { slot, mesh, textures, placement });
}

fn extra_bone(slot: u8) -> Option<&'static str> {
    match slot {
        SLOT_LEGS => Some(LOWBODY_BONE),
        SLOT_RIGHT_HAND => Some(RIGHT_HAND_BONE),
        SLOT_LEFT_HAND => Some(LEFT_HAND_BONE),
        SLOT_RIGHT_ARM => Some(RIGHT_ARM_BONE),
        SLOT_LEFT_ARM => Some(""),
        _ => None,
    }
}

pub fn armor_mesh_index(armor: &Armor, slot: u8) -> usize {
    match (armor.body_part, slot) {
        (body_part::ONEPIECE, SLOT_LEGS) | (body_part::ALLDRESS, SLOT_LEGS) => 1,
        (body_part::ALLDRESS, SLOT_GLOVES) => 2,
        (body_part::ALLDRESS, SLOT_BOOTS) => 3,
        _ => 0,
    }
}

pub fn helm_mesh_type(mesh: Option<&str>) -> i32 {
    let Some(mesh) = mesh else { return 0 };
    mesh.find("_m00")
        .and_then(|at| mesh[at + 4..].chars().next())
        .and_then(|digit| digit.to_digit(10))
        .map_or(0, |digit| digit as i32)
}

pub fn weapon_scale(collision_radius: f32, collision_height: f32, weapon_scale: f32) -> f32 {
    if weapon_scale == 0.0 {
        (collision_height * collision_radius).sqrt() / WEAPON_SCALE_REFERENCE.sqrt()
    } else {
        weapon_scale
    }
}

pub fn sub_mesh(slot: u8) -> Option<SubMesh> {
    Some(match slot {
        SLOT_BOOTS => SubMesh::Boots,
        SLOT_LEGS => SubMesh::Legs,
        SLOT_BODY => SubMesh::Body,
        SLOT_GLOVES => SubMesh::Gloves,
        SLOT_HAIR_B => SubMesh::HairB,
        SLOT_HAIR_A => SubMesh::HairA,
        SLOT_HELM => SubMesh::Helm,
        SLOT_FACE => SubMesh::Face,
        SLOT_RIGHT_HAND => SubMesh::RightHand,
        SLOT_LEFT_HAND => SubMesh::LeftHand,
        SLOT_RIGHT_ARM => SubMesh::RightArm,
        SLOT_LEFT_ARM => SubMesh::LeftArm,
        SLOT_CLOAK => SubMesh::Cloak,
        _ => return None,
    })
}

pub fn tag_of(item: &Item<'_>) -> u32 {
    match item {
        Item::Weapon(_) => TAG_WEAPON,
        Item::Armor(_) => TAG_ARMOR,
    }
}

pub fn is_dual_fist(weapon_type: u32) -> bool {
    weapon_type == anim_type::DUAL_FIST
}

fn template_string(value: &str) -> Option<String> {
    (!value.is_empty() && !value.eq_ignore_ascii_case("None")).then(|| value.to_string())
}

fn find_ignore_case(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let (h, n) = (haystack.as_bytes(), needle.as_bytes());
    (0..=h.len() - n.len()).find(|&at| h[at..at + n.len()].eq_ignore_ascii_case(n))
}

pub fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    find_ignore_case(haystack, needle).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2_dat::armorgrp::ArmorRow;
    use l2_dat::hairgrp::HairRow;
    use l2_dat::{CharTemplate, HairAccessoryLoc, Npc};

    struct Fixture {
        armors: HashMap<u32, Armor>,
        weapons: HashMap<u32, Weapon>,
        hairgrp: Vec<HairRow>,
        helmets: HashMap<u32, Vec<HairRow>>,
        accessories: HashMap<u32, Vec<HairRow>>,
        locs: HashMap<String, HairAccessoryLoc>,
        template: CharTemplate,
    }

    impl Fixture {
        fn tables(&self) -> Tables<'_> {
            Tables {
                items: Items { armors: &self.armors, weapons: &self.weapons },
                hairgrp: &self.hairgrp,
                helmetgrp: &self.helmets,
                hairaccessarygrp: &self.accessories,
                accessory_locs: &self.locs,
            }
        }

        fn player(&self, row: usize, equipped: Equipped) -> Player<'_> {
            Player {
                template: &self.template,
                row,
                face: 2,
                hair_style: 1,
                hair_color: 1,
                equipped,
                tables: self.tables(),
                mounted: None,
                korean: false,
                korean_flag: false,
                clan_crest: None,
            }
        }
    }

    fn armor(id: u32, body_part: u32, meshes: &[&str], textures: &[&str]) -> Armor {
        let mut rows = vec![ArmorRow::default(); 15];
        for (row, class) in [(0, "MFighter"), (1, "FFighter"), (3, "FDarkElf"), (6, "MElf")] {
            rows[row] = ArmorRow {
                meshes: meshes.iter().map(|m| m.replace("MFighter", class)).collect(),
                textures: textures.iter().map(|t| t.replace("MFighter", class)).collect(),
                ex_meshes: vec![String::new(); meshes.len()],
                ex_textures: vec![String::new(); textures.len()],
            };
        }
        Armor {
            drop_sound:String::new(),
            equip_sound: String::new(),
            boots_sounds:vec![],
            id,
            body_part,
            rows,
            npc_meshes: vec!["LineageNPCs.guard_armor".into()],
            npc_textures: vec!["LineageNPCsTex.guard_armor_t00".into()],
            damage_effect: "LineageEffect.p_u000_a".into(),
            attributes_58c: [0; 6],
            crystallizable: false,
        }
    }

    fn weapon(id: u32, body_part: u32, handness: u32, meshes: &[&str], textures: &[&str]) -> Weapon {
        Weapon {
            drop_sound:String::new(),
            equip_sound: String::new(),
            attack_sounds:vec![],
            id,
            body_part,
            handness,
            meshes: meshes.iter().map(|m| m.to_string()).collect(),
            textures: textures.iter().map(|t| t.to_string()).collect(),
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

    #[test]
    fn get_arrow_item_id_10466660_takes_every_branch() {
        let mut weapons = HashMap::new();
        for (id, grade) in [(100, 0), (101, 1), (102, 2), (103, 3), (104, 4), (105, 5), (106, 6), (107, -1)] {
            let mut bow = weapon(id, 0, anim_type::BOW, &[], &[]);
            bow.attributes_dc[0] = grade;
            weapons.insert(id, bow);
        }
        let mut armors = HashMap::new();
        armors.insert(200, armor(200, 0, &[], &[]));
        let items = Items { armors: &armors, weapons: &weapons };
        let expected = [0x11, 0x53d, 0x53e, 0x53f, 0x540, 0x541, 0x11, 0x11];
        for (index, want) in expected.iter().enumerate() {
            let id = 100 + index as i32;
            let pc = arrow_item_id(&items, &Equipped { lrhand: id, rhand: 7, ..Default::default() }, ArrowOwner::Pc);
            assert_eq!(pc, ArrowItem { bow_item: id, arrow_item: *want });
            let npc = arrow_item_id(&items, &Equipped { rhand: id, lrhand: 7, ..Default::default() }, ArrowOwner::Npc);
            assert_eq!(npc, ArrowItem { bow_item: id, arrow_item: *want });
        }
        assert_eq!(arrow_item_id(&items, &Equipped { lrhand: 200, ..Default::default() }, ArrowOwner::Pc).arrow_item, DEFAULT_ARROW_ITEM);
        assert_eq!(arrow_item_id(&items, &Equipped::default(), ArrowOwner::Pc), ArrowItem { bow_item: 0, arrow_item: DEFAULT_ARROW_ITEM });
        let own = Equipped { lrhand: 105, ..Default::default() };
        assert_eq!(arrow_item_id(&items, &own, ArrowOwner::Own { left_hand_item: Some(1344) }), ArrowItem { bow_item: 105, arrow_item: 1344 });
        assert_eq!(arrow_item_id(&items, &own, ArrowOwner::Own { left_hand_item: None }).arrow_item, DEFAULT_ARROW_ITEM);
    }

    fn fighter_hair() -> HairRow {
        HairRow {
            ah: [0, 1, 2, -1, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
            bh: [0, 1, 2, 3, 4, -1, -1, -1, -1, 9, -1, -1, -1, -1, -1],
        }
    }

    fn fixture() -> Fixture {
        let armors = [
            armor(
                21,
                body_part::CHEST,
                &["Fighter.MFighter_m001_u"],
                &["MFighter.MFighter_m001_t02_u", "MFighter.MFighter_m001_t02_u2"],
            ),
            armor(
                28,
                body_part::LEGS,
                &["Fighter.MFighter_m001_l"],
                &["MFighter.MFighter_m001_t02_l", "MFighter.MFighter_m001_t02_l2"],
            ),
            armor(
                48,
                body_part::GLOVES,
                &["Fighter.MFighter_m001_g"],
                &["MFighter.MFighter_m001_t02_g", "MFighter.MFighter_m001_t02_g2"],
            ),
            armor(35, body_part::FEET, &["Fighter.MFighter_m001_b"], &["MFighter.MFighter_m001_t02_b"]),
            armor(41, body_part::HEAD, &[""], &[""]),
            armor(
                60,
                body_part::ONEPIECE,
                &["Fighter.MFighter_m003_u", "Fighter.MFighter_m003_l"],
                &["MFighter.MFighter_m003_t61_u", "MFighter.MFighter_m003_t61_l"],
            ),
            armor(
                6408,
                body_part::ALLDRESS,
                &[
                    "Fighter.MFighter_m013_u",
                    "Fighter.MFighter_m013_l",
                    "Fighter.MFighter_m013_g",
                    "Fighter.MFighter_m013_b",
                ],
                &[
                    "MFighter.MFighter_m013_t2000_u",
                    "MFighter.MFighter_m013_t2000_l",
                    "MFighter.MFighter_m013_t2000_g",
                    "MFighter.MFighter_m013_t2000_b",
                ],
            ),
            armor(
                7836,
                body_part::HAIR,
                &["LineageAccessory.Mfighter_santas_cap_m002_a"],
                &["LineageAccessoryTex.santas_cap_m002_a_t00"],
            ),
            armor(
                7837,
                body_part::HAIR,
                &["LineageAccessory.Mfighter_hairpin_m000_a"],
                &["LineageAccessoryTex.hairpin_t00"],
            ),
            armor(70, body_part::CHEST, &["Elf.MElf_m009_u"], &["MElf.MElf_m009_t01_u"]),
            armor(71, body_part::GLOVES, &["Elf.MElf_m009_g"], &["MElf.MElf_m009_t01_g"]),
            armor(72, body_part::GLOVES, &["Elf.MElf_m009_g"], &["MElf.MElf_m009_t02_g"]),
            armor(80, body_part::CHEST, &["DarkElf.FDarkElf_m003_u"], &["FDarkElf.FDarkElf_m003_t00_u"]),
            armor(81, body_part::CHEST, &["DarkElf.FDarkElf_m001_u"], &["FDarkElf.FDarkElf_m001_t01_u"]),
            armor(
                90,
                body_part::BACK,
                &["LineageWeapons.cotton_cloak_m00_mt"],
                &["LineageWeaponsTex.cotton_cloak_t00_mt"],
            ),
        ];
        let mut rod = weapon(
            6529,
            body_part::LRHAND,
            anim_type::POLE,
            &["LineageWeapons.fishing_rod_m00_wp"],
            &["LineageWeaponsTex.fishing_rod_t00_wp"],
        );
        rod.sub_mesh_skipped = true;
        let weapons = [
            weapon(
                1,
                body_part::RHAND,
                anim_type::ONE_HAND_SWORD,
                &["LineageWeapons.small_sword_m00_wp"],
                &["LineageWeaponsTex.small_sword_t00_wp"],
            ),
            weapon(
                18,
                body_part::LHAND,
                anim_type::HAND,
                &["LineageWeapons.leather_shield_m00_sh"],
                &["LineageWeaponsTex.leather_shield_t00_sh", "LineageWeaponsTex.leather_shield_t00_sh2"],
            ),
            weapon(
                13,
                body_part::LRHAND,
                anim_type::BOW,
                &["LineageWeapons.short_bow_m00_wp"],
                &["LineageWeaponsTex.short_bow_t00_wp"],
            ),
            weapon(
                1299,
                body_part::LRHAND,
                anim_type::DUAL,
                &["LineageWeapons.sword_of_delusion_m00_wp", "LineageWeapons.sword_of_magic_m00_wp"],
                &["LineageWeaponsTex.sword_of_delusion_t00_wp", "LineageWeaponsTex.sword_of_magic_t00_wp"],
            ),
            weapon(
                43,
                body_part::LRHAND,
                anim_type::DUAL_FIST,
                &["LineageWeapons.bagh_nakh_m00_wp", "LineageWeapons.bagh_nakh_m00_wp2"],
                &["LineageWeaponsTex.bagh_nakh_t00_wp", "LineageWeaponsTex.bagh_nakh_t00_wp2"],
            ),
            rod,
        ];
        let mut helmets = HashMap::new();
        let mut capped = fighter_hair();
        capped.ah = [-1; 15];
        helmets.insert(2, vec![capped; 15]);
        let mut accessories = HashMap::new();
        let mut pinned = fighter_hair();
        pinned.bh[1] = 7;
        accessories.insert(2, vec![pinned; 15]);
        let mut locs = HashMap::new();
        let mut offsets = [[0.0; 3]; 15];
        offsets[1] = [3.4, 0.0, 1.8];
        let mut rotations = [[0; 3]; 15];
        rotations[1] = [16384, 32768, 0];
        let loc = HairAccessoryLoc { mesh: "LineageAccessory.Mfighter_hairpin_m000_a".into(), offsets, rotations };
        locs.insert(loc.mesh.to_ascii_lowercase(), loc);
        let template = CharTemplate {
            face_icon: String::new(),
            hair_meshes: vec![],
            hair_textures: vec![],
            face_meshes: vec!["Fighter.MFighter_m000_f".into()],
            face_textures: vec!["a".into(), "b".into(), "c".into()],
            body_meshes: ["u".into(), "l".into(), "g".into(), "b".into()],
            body_textures: ["tu".into(), "tl".into(), "tg".into(), "tb".into()],
            damage_effect: "LineageEffect.p_u002_a".into(),
            walk_anim_frame: 12,
            attack_sounds: vec![],
            defense_sounds: vec![],
            damage_sounds: vec![],
            voice_sounds: Default::default(),
        };
        Fixture {
            armors: armors.into_iter().map(|a| (a.id, a)).collect(),
            weapons: weapons.into_iter().map(|w| (w.id, w)).collect(),
            hairgrp: vec![fighter_hair(); 15],
            helmets,
            accessories,
            locs,
            template,
        }
    }

    fn meshes(a: &Appearance) -> Vec<(u8, &str)> {
        a.parts.iter().map(|p| (p.slot, p.mesh.as_str())).collect()
    }

    #[test]
    fn damage_effect_follows_get_damage_effect_1046a3c0() {
        let mut fixture = fixture();
        assert_eq!(fixture.player(0, Equipped::default()).set_pawn_resource().damage_effect.as_deref(), Some("LineageEffect.p_u002_a"));
        let chest = Equipped { chest: 21, ..Default::default() };
        assert_eq!(fixture.player(0, chest).set_pawn_resource().damage_effect.as_deref(), Some("LineageEffect.p_u000_a"));
        assert_eq!(fixture.player(0, Equipped { chest: 9999, ..Default::default() }).damage_effect().as_deref(), Some("LineageEffect.p_u002_a"));
        let mut record = npc("Mesh");
        record.effect = "LineageEffect.p_u110_a".into();
        let pawn = |equipped| NpcPawn { npc: &record, equipped, items: Items { armors: &fixture.armors, weapons: &fixture.weapons }, mounted: false, petrified: false };
        assert_eq!(pawn(Equipped::default()).set_pawn_resource().damage_effect.as_deref(), Some("LineageEffect.p_u110_a"));
        assert_eq!(pawn(chest).damage_effect().as_deref(), Some("LineageEffect.p_u000_a"));
        fixture.template.damage_effect = "NONE".into();
        assert_eq!(fixture.player(0, Equipped::default()).damage_effect(), None);
        assert_eq!(damage_effect_name(""), None);
    }

    #[test]
    fn part_keeps_empty_texture_positions_without_inventing_names() {
        let part = Part::new("Mesh", vec!["None".into(), "A".into(), "".into(), "B".into(), "nOnE".into()]);
        assert_eq!(part.textures, [None, Some("A".into()), None, Some("B".into()), None]);
        assert!(Part::new("Mesh", vec![]).textures.is_empty());
    }

    #[test]
    fn player_keeps_each_texture_call_including_missing_names() {
        let mut fixture = fixture();
        fixture.armors.get_mut(&48).unwrap().rows[0].textures = vec!["None".into(), "A".into(), "".into(), "B".into()];
        let player = fixture.player(0, Equipped { gloves: 48, ..Default::default() });
        assert_eq!(player.pc_tex_num(SLOT_GLOVES), 4);
        let appearance = player.set_pawn_resource();
        assert_eq!(appearance.part(SLOT_GLOVES).unwrap().textures, [None, Some("A".into()), None, Some("B".into())]);
    }

    #[test]
    fn npc_keeps_texture_indices_in_normal_and_petrified_lists() {
        let fixture = fixture();
        let mut record = npc("Mesh");
        record.textures = vec!["None".into(), "A".into(), "".into(), "B".into(), "None".into()];
        record.textures2 = vec!["Stone".into(), "None".into(), "UnusedLast".into()];
        let pawn = NpcPawn {
            npc: &record,
            equipped: Equipped::default(),
            items: Items { armors: &fixture.armors, weapons: &fixture.weapons },
            mounted: false,
            petrified: false,
        };
        let expected = vec![None, Some("A".into()), None, Some("B".into()), None];
        assert_eq!(pawn.npc_tex_names(), expected);
        assert_eq!(pawn.set_pawn_resource().part(SLOT_FACE).unwrap().textures, expected);
        let stone = NpcPawn { petrified: true, ..pawn };
        let expected = vec![Some("Stone".into()), None, None, Some("B".into()), None];
        assert_eq!(stone.npc_tex_names(), expected);
        assert_eq!(stone.set_pawn_resource().part(SLOT_FACE).unwrap().textures, expected);
    }

    #[test]
    fn a_naked_fighter_has_body_hair_and_face_in_slot_order() {
        let fixture = fixture();
        let a = fixture.player(0, Equipped::default()).set_pawn_resource();
        assert_eq!(a.weapon_type, anim_type::HAND);
        assert_eq!(
            meshes(&a),
            [
                (0, "b"),
                (1, "l"),
                (2, "u"),
                (3, "g"),
                (4, "Fighter.MFighter_m001_m00_bh"),
                (5, "Fighter.MFighter_m001_m00_ah"),
                (7, "Fighter.MFighter_m000_f")
            ]
        );
        assert_eq!(a.parts[0].textures, [Some("tb".into())]);
        assert_eq!(a.parts[2].textures, [Some("tu".into())]);
        assert_eq!(a.parts[4].textures, [Some("MFighter.MFighter_m001_t01_m00_bh".into())]);
        assert_eq!(a.parts[5].textures, [Some("MFighter.MFighter_m001_t01_m00_ah".into())]);
        assert_eq!(a.parts[6].textures, [Some("c".into())]);
        assert_eq!(a.parts[6].placement, Placement::Master);
        assert!(a.parts[..6].iter().all(|p| p.placement == Placement::Skinned));
        assert_eq!(a.part(7).map(|p| p.mesh.as_str()), Some("Fighter.MFighter_m000_f"));
        let mut player = fixture.player(0, Equipped::default());
        player.face = 9;
        assert_eq!(player.pc_tex_name(SLOT_FACE, 0), None);
        player.face = -1;
        assert_eq!(player.pc_tex_name(SLOT_FACE, 0), None);
        player.hair_style = 3;
        assert_eq!(player.pc_mesh_name(SLOT_HAIR_A), None);
        assert_eq!(player.pc_mesh_name(SLOT_HAIR_B).as_deref(), Some("Fighter.MFighter_m003_m00_bh"));
        player.hair_style = 15;
        assert_eq!(player.pc_mesh_name(SLOT_HAIR_B), None);
        player.row = 14;
        assert_eq!(player.pc_mesh_name(SLOT_HAIR_B), None);
    }

    #[test]
    fn armor_replaces_the_template_meshes_and_body_and_legs_take_one_texture() {
        let fixture = fixture();
        let set = Equipped { chest: 21, legs: 28, gloves: 48, feet: 35, head: 41, ..Default::default() };
        let player = fixture.player(0, set);
        let a = player.set_pawn_resource();
        assert_eq!(
            meshes(&a)[..4],
            [
                (0, "Fighter.MFighter_m001_b"),
                (1, "Fighter.MFighter_m001_l"),
                (2, "Fighter.MFighter_m001_u"),
                (3, "Fighter.MFighter_m001_g")
            ]
        );
        assert_eq!(player.pc_tex_num(SLOT_BODY), 1);
        assert_eq!(player.pc_tex_num(SLOT_LEGS), 1);
        assert_eq!(player.pc_tex_num(SLOT_GLOVES), 2);
        assert_eq!(player.pc_tex_num(SLOT_HELM), 1);
        assert_eq!(a.parts[2].textures, [Some("MFighter.MFighter_m001_t02_u".into())]);
        assert_eq!(a.parts[1].textures, [Some("MFighter.MFighter_m001_t02_l".into())]);
        assert_eq!(a.parts[3].textures, [Some("MFighter.MFighter_m001_t02_g".into()), Some("MFighter.MFighter_m001_t02_g2".into())]);
        assert!(a.part(6).is_none());
        assert_eq!(player.helm_mesh_type(), 0);
        let female = fixture.player(1, set).set_pawn_resource();
        assert_eq!(female.parts[2].mesh, "Fighter.FFighter_m001_u");
        let unknown = fixture.player(0, Equipped { chest: 9999, legs: 28, ..Default::default() }).set_pawn_resource();
        assert_eq!(meshes(&unknown)[..4], [(0, "b"), (1, "Fighter.MFighter_m001_l"), (2, "u"), (3, "g")]);
        let dwarf = fixture.player(5, set).set_pawn_resource();
        assert_eq!(
            meshes(&dwarf),
            [(4, "Dwarf.FDwarf_m001_m00_bh"), (5, "Dwarf.FDwarf_m001_m00_ah"), (7, "Fighter.MFighter_m000_f")]
        );
    }

    #[test]
    fn onepiece_and_alldress_take_their_pieces_by_index() {
        let fixture = fixture();
        let a = fixture
            .player(0, Equipped { chest: 60, legs: 28, gloves: 48, feet: 35, ..Default::default() })
            .set_pawn_resource();
        assert_eq!(
            meshes(&a)[..4],
            [
                (0, "Fighter.MFighter_m001_b"),
                (1, "Fighter.MFighter_m003_l"),
                (2, "Fighter.MFighter_m003_u"),
                (3, "Fighter.MFighter_m001_g")
            ]
        );
        assert_eq!(a.parts[1].textures, [Some("MFighter.MFighter_m003_t61_l".into())]);
        assert_eq!(a.parts[2].textures, [Some("MFighter.MFighter_m003_t61_u".into())]);
        let a = fixture
            .player(0, Equipped { chest: 6408, legs: 28, gloves: 48, feet: 35, ..Default::default() })
            .set_pawn_resource();
        assert_eq!(
            meshes(&a)[..4],
            [
                (0, "Fighter.MFighter_m013_b"),
                (1, "Fighter.MFighter_m013_l"),
                (2, "Fighter.MFighter_m013_u"),
                (3, "Fighter.MFighter_m013_g")
            ]
        );
        assert_eq!(a.parts[0].textures, vec![Some("MFighter.MFighter_m013_t2000_b".into()); 4]);
        assert_eq!(a.parts[3].textures, vec![Some("MFighter.MFighter_m013_t2000_g".into()); 4]);
        assert_eq!(a.parts[1].textures, [Some("MFighter.MFighter_m013_t2000_l".into())]);
    }

    #[test]
    fn weapons_and_shields_attach_to_the_hand_bones_and_rods_become_extra_meshes() {
        let fixture = fixture();
        let a = fixture.player(0, Equipped { rhand: 1, lhand: 18, ..Default::default() }).set_pawn_resource();
        assert_eq!(a.weapon_type, anim_type::ONE_HAND_SWORD);
        let sword = a.part(SLOT_RIGHT_HAND).unwrap();
        assert_eq!(sword.mesh, "LineageWeapons.small_sword_m00_wp");
        assert_eq!(sword.textures, [Some("LineageWeaponsTex.small_sword_t00_wp".into())]);
        assert_eq!(sword.placement, Placement::Attached { bone: RIGHT_HAND_BONE });
        let shield = a.part(SLOT_LEFT_ARM).unwrap();
        assert_eq!(shield.mesh, "LineageWeapons.leather_shield_m00_sh");
        assert_eq!(shield.textures.len(), 2);
        assert_eq!(shield.placement, Placement::Attached { bone: LEFT_ARM_BONE });
        assert_eq!(a.parts.len(), 9);
        let bow = fixture.player(0, Equipped { lrhand: 13, ..Default::default() }).set_pawn_resource();
        assert_eq!(bow.weapon_type, anim_type::BOW);
        assert_eq!(bow.part(SLOT_LEFT_HAND).map(|p| p.mesh.as_str()), Some("LineageWeapons.short_bow_m00_wp"));
        assert!(bow.part(SLOT_RIGHT_HAND).is_none());
        assert_eq!(bow.arrow, Some(ArrowItem { bow_item: 13, arrow_item: DEFAULT_ARROW_ITEM }));
        assert_eq!(a.arrow, None);
        let dual = fixture.player(0, Equipped { lrhand: 1299, ..Default::default() }).set_pawn_resource();
        assert_eq!(dual.weapon_type, anim_type::DUAL);
        assert_eq!(
            dual.part(SLOT_RIGHT_HAND).map(|p| (p.mesh.as_str(), p.textures.clone())),
            Some((
                "LineageWeapons.sword_of_delusion_m00_wp",
                vec![Some("LineageWeaponsTex.sword_of_delusion_t00_wp".to_string()); 2]
            ))
        );
        assert_eq!(
            dual.part(SLOT_LEFT_HAND).map(|p| (p.mesh.as_str(), p.textures.clone())),
            Some((
                "LineageWeapons.sword_of_magic_m00_wp",
                vec![Some("LineageWeaponsTex.sword_of_magic_t00_wp".to_string()); 2]
            ))
        );
        let fists = fixture.player(0, Equipped { lrhand: 43, ..Default::default() }).set_pawn_resource();
        assert_eq!(fists.weapon_type, anim_type::DUAL_FIST);
        assert!(is_dual_fist(fists.weapon_type));
        assert_eq!(fists.part(SLOT_LEFT_HAND).map(|p| p.mesh.as_str()), Some("LineageWeapons.bagh_nakh_m00_wp2"));
        let rod = fixture.player(0, Equipped { lrhand: 6529, ..Default::default() }).set_pawn_resource();
        assert_eq!(rod.weapon_type, anim_type::POLE);
        assert!(rod.part(SLOT_RIGHT_HAND).is_none());
        let extra = rod.parts.iter().find(|p| p.slot == SLOT_RIGHT_HAND).unwrap();
        assert_eq!(extra.mesh, "LineageWeapons.fishing_rod_m00_wp");
        assert_eq!(extra.textures, [Some("LineageWeaponsTex.fishing_rod_t00_wp".into())]);
        assert_eq!(extra.placement, Placement::Extra { bone: Some(RIGHT_HAND_BONE) });
        assert!(extra.is_attached());
        let mounted =
            Player { mounted: Some(1), ..fixture.player(0, Equipped { rhand: 1, lhand: 18, ..Default::default() }) }
                .set_pawn_resource();
        assert!(mounted.part(SLOT_RIGHT_HAND).is_none() && mounted.part(SLOT_LEFT_ARM).is_none());
        assert_eq!(mounted.part(SLOT_HELM).map(|p| p.mesh.as_str()), Some(HEAD_GEAR_MESH));
        let wyvern = Player { mounted: Some(2), ..fixture.player(0, Equipped::default()) }.set_pawn_resource();
        assert!(wyvern.part(SLOT_HELM).is_none());
    }

    #[test]
    fn hair_accessories_switch_tables_and_take_the_accessory_offset() {
        let fixture = fixture();
        let cap = fixture.player(0, Equipped { hair: 7836, ..Default::default() });
        assert_eq!(cap.helm_mesh_type(), 2);
        assert!(cap.have_accessary());
        let a = cap.set_pawn_resource();
        assert_eq!(
            meshes(&a)[4..],
            [
                (4, "Fighter.MFighter_m007_m00_bh"),
                (5, "Fighter.MFighter_m001_m00_ah"),
                (6, "LineageAccessory.Mfighter_santas_cap_m002_a"),
                (7, "Fighter.MFighter_m000_f")
            ]
        );
        assert_eq!(
            a.part(SLOT_HELM).map(|p| p.placement),
            Some(Placement::Helm { offset: [0.0; 3], rotation: [0; 3] })
        );
        let pin = fixture.player(0, Equipped { hair: 7837, ..Default::default() });
        assert_eq!(pin.helm_mesh_type(), 0);
        let a = pin.set_pawn_resource();
        assert_eq!(a.part(SLOT_HAIR_B).map(|p| p.mesh.as_str()), Some("Fighter.MFighter_m001_m00_bh"));
        assert_eq!(
            a.part(SLOT_HELM).map(|p| p.placement),
            Some(Placement::Helm { offset: [3.4, 0.0, 1.8], rotation: [16384, 32768, 0] })
        );
        let mut helmet_only = self::fixture();
        helmet_only.accessories.clear();
        let a = helmet_only.player(0, Equipped { hair: 7836, ..Default::default() }).set_pawn_resource();
        assert_eq!(
            meshes(&a)[4..],
            [(6, "LineageAccessory.Mfighter_santas_cap_m002_a"), (7, "Fighter.MFighter_m000_f")]
        );
        let mut helm_from_helmetgrp = self::fixture();
        helm_from_helmetgrp
            .armors
            .insert(42, armor(42, body_part::HEAD, &["LineageAccessory.Mfighter_santas_cap_m002_a"], &["x"]));
        let a = helm_from_helmetgrp.player(0, Equipped { head: 42, ..Default::default() }).set_pawn_resource();
        assert_eq!(
            meshes(&a)[4..],
            [
                (4, "Fighter.MFighter_m001_m00_bh"),
                (6, "LineageAccessory.Mfighter_santas_cap_m002_a"),
                (7, "Fighter.MFighter_m000_f")
            ]
        );
        assert_eq!(helm_mesh_type(Some("LineageAccessory.Mfighter_partymask_m000_a")), 0);
        assert_eq!(helm_mesh_type(Some("Fighter.MFighter_m001_u")), 1);
        assert_eq!(helm_mesh_type(None), 0);
    }

    #[test]
    fn elf_sets_drop_the_gloves_when_the_textures_do_not_match() {
        let fixture = fixture();
        let matched = fixture.player(6, Equipped { chest: 70, gloves: 71, ..Default::default() });
        assert_eq!(matched.pc_mesh_name(SLOT_GLOVES).as_deref(), Some("Elf.MElf_m009_g"));
        assert_eq!(matched.pc_tex_name(SLOT_GLOVES, 0).as_deref(), Some("MElf.MElf_m009_t01_g"));
        let mismatched = fixture.player(6, Equipped { chest: 70, gloves: 72, ..Default::default() });
        assert_eq!(mismatched.pc_mesh_name(SLOT_GLOVES).as_deref(), Some("Elf.MElf_m000_g"));
        assert_eq!(mismatched.pc_tex_name(SLOT_GLOVES, 0).as_deref(), Some("MElf.MElf_m000_t1000_g"));
        let bare = fixture.player(6, Equipped { chest: 70, ..Default::default() });
        assert_eq!(bare.pc_mesh_name(SLOT_GLOVES).as_deref(), Some("Elf.MElf_m000_g"));
    }

    #[test]
    fn fdarkelf_hair_takes_the_u00_variant_over_the_special_bodies() {
        let fixture = fixture();
        let mut plain = fixture.player(3, Equipped { chest: 80, ..Default::default() });
        plain.hair_style = 0;
        assert_eq!(plain.pc_mesh_name(SLOT_HAIR_B).as_deref(), Some("DarkElf.FDarkElf_m000_m00_bh_u00"));
        assert_eq!(plain.pc_mesh_name(SLOT_HAIR_A).as_deref(), Some("DarkElf.FDarkElf_m000_m00_ah_u00"));
        assert_eq!(plain.pc_tex_name(SLOT_HAIR_B, 0).as_deref(), Some("FDarkElf.FDarkElf_m000_t01_m00_bh_u00"));
        plain.hair_style = 1;
        assert_eq!(plain.pc_mesh_name(SLOT_HAIR_B).as_deref(), Some("DarkElf.FDarkElf_m001_m00_bh"));
        let other_body = fixture.player(3, Equipped { chest: 81, ..Default::default() });
        let mut other_body = other_body;
        other_body.hair_style = 0;
        assert_eq!(other_body.pc_mesh_name(SLOT_HAIR_B).as_deref(), Some("DarkElf.FDarkElf_m000_m00_bh"));
    }

    #[test]
    fn korean_clients_swap_the_fdarkelf_m001_set() {
        let fixture = fixture();
        let mut korean = fixture.player(3, Equipped { chest: 81, ..Default::default() });
        assert_eq!(korean.pc_mesh_name(SLOT_BODY).as_deref(), Some("DarkElf.FDarkElf_m001_u"));
        assert_eq!(korean.pc_tex_name(SLOT_BODY, 0).as_deref(), Some("FDarkElf.FDarkElf_m001_t01_u"));
        korean.korean = true;
        assert_eq!(korean.pc_mesh_name(SLOT_BODY).as_deref(), Some("DarkElf.FDarkElf_m005_u"));
        assert_eq!(korean.pc_tex_name(SLOT_BODY, 0).as_deref(), Some("FDarkElf.FDarkElf_m005_t32_u"));
        korean.korean_flag = true;
        assert_eq!(korean.pc_tex_name(SLOT_BODY, 0).as_deref(), Some("FDarkElf.FDarkElf_m005_t32_u"));
    }

    #[test]
    fn cloaks_are_kept_apart_because_the_engine_never_draws_them() {
        let fixture = fixture();
        let a = fixture.player(0, Equipped { back: 90, ..Default::default() }).set_pawn_resource();
        assert!(a.parts.iter().all(|p| p.slot != SLOT_CLOAK));
        assert_eq!(
            a.cloak.as_ref().map(|c| (c.slot, c.mesh.as_str(), c.placement)),
            Some((SLOT_CLOAK, "LineageWeapons.cotton_cloak_m00_mt", Placement::Attached { bone: CAPE_BONE }))
        );
    }

    #[test]
    fn clan_crest_replaces_the_last_texture_only_when_the_record_asks() {
        let mut fixture = fixture();
        let mut cloak =
            weapon(91, body_part::BACK, anim_type::HAND, &["LineageWeapons.pledge_cloak_m00_mt"], &["a", "b"]);
        cloak.crest_texture = 2;
        fixture.weapons.insert(91, cloak);
        let mut player = fixture.player(0, Equipped { lhand: 91, ..Default::default() });
        assert_eq!(player.pc_tex_name(SLOT_LEFT_ARM, 1).as_deref(), Some("b"));
        player.clan_crest = Some(ClanCrest { server_id: 1, ally_crest_id: 77 });
        assert_eq!(player.pc_tex_name(SLOT_LEFT_ARM, 1).as_deref(), Some("crest.large.crest_1_77_l"));
        assert_eq!(player.pc_tex_name(SLOT_LEFT_ARM, 0).as_deref(), Some("a"));
    }

    fn npc(mesh: &str) -> Npc {
        Npc {
            id: 1,
            class: "LineageNpc.guard".into(),
            mesh: mesh.into(),
            textures: vec!["LineageNPCsTex.guard_t00".into(), "LineageNPCsTex.guard_t01".into()],
            textures2: vec!["LineageNPCsTex.guard_stone".into(), String::new()],
            speed: 1.0,
            effect: String::new(),
            level_min: 0,
            level_max: 0,
            attack_sounds: vec![],
            defense_sounds: vec![],
            damage_sounds: vec![],
            sound_radius: 0.0,
            sound_volume: 0.0,
            sound_random: 0.0, skills: vec![], shows_bars: 0,
        }
    }

    #[test]
    fn npcs_take_the_record_mesh_hand_items_and_the_npc_body_of_a_chest_armor() {
        let fixture = fixture();
        let guard = npc("LineageNPCs.guard_m00");
        let items = Items { armors: &fixture.armors, weapons: &fixture.weapons };
        let pawn = NpcPawn {
            npc: &guard,
            equipped: Equipped { rhand: 1, lhand: 18, chest: 21, ..Default::default() },
            items,
            mounted: false,
            petrified: false,
        };
        let a = pawn.set_pawn_resource();
        assert_eq!(
            meshes(&a),
            [
                (7, "LineageNPCs.guard_m00"),
                (8, "LineageWeapons.small_sword_m00_wp"),
                (11, "LineageWeapons.leather_shield_m00_sh"),
                (2, "LineageNPCs.guard_armor")
            ]
        );
        assert_eq!(a.parts[0].textures, [Some("LineageNPCsTex.guard_t00".into()), Some("LineageNPCsTex.guard_t01".into())]);
        assert_eq!(a.parts[3].textures, [Some("LineageNPCsTex.guard_armor_t00".into())]);
        assert_eq!(a.parts[3].placement, Placement::Skinned);
        assert_eq!(a.weapon_type, anim_type::ONE_HAND_SWORD);
        let stone = NpcPawn { petrified: true, ..pawn }.npc_tex_names();
        assert_eq!(stone, [Some("LineageNPCsTex.guard_stone".into()), Some("LineageNPCsTex.guard_t01".into())]);
        let mounted = NpcPawn { mounted: true, ..pawn }.set_pawn_resource();
        assert_eq!(mounted.parts.len(), 2);
        let dual = NpcPawn { equipped: Equipped { lrhand: 1299, ..Default::default() }, ..pawn }.set_pawn_resource();
        assert_eq!(
            dual.part(SLOT_LEFT_HAND).map(|p| p.textures.clone()),
            Some(vec![Some("LineageWeaponsTex.sword_of_magic_t00_wp".to_string())])
        );
        let bare = NpcPawn { npc: &npc(""), equipped: Equipped::default(), ..pawn }.set_pawn_resource();
        assert!(bare.parts.is_empty());
    }

    #[test]
    fn weapon_scale_follows_the_collision_cylinder_unless_the_pawn_fixes_it() {
        assert!((weapon_scale(9.0, 23.0, 0.0) - 1.0).abs() < 1e-6);
        assert!((weapon_scale(9.0, 18.0, 0.0) - (162.0f32 / 207.0).sqrt()).abs() < 1e-6);
        assert_eq!(weapon_scale(9.0, 23.0, 1.5), 1.5);
        assert_eq!(armor_mesh_index(&armor(1, body_part::ALLDRESS, &[""], &[""]), SLOT_BOOTS), 3);
        assert_eq!(armor_mesh_index(&armor(1, body_part::ONEPIECE, &[""], &[""]), SLOT_GLOVES), 0);
        assert!(contains_ignore_case("Elf.MELF_m009_u", "melf_M009_U"));
        assert!(!contains_ignore_case("Elf", "Elfo"));
        assert_eq!(sub_mesh(SLOT_CLOAK_COVER), None);
        assert_eq!(tag_of(&Item::Weapon(&weapon(1, 0, 0, &[], &[]))), TAG_WEAPON);
        assert_eq!(tag_of(&Item::Armor(&armor(1, 0, &[""], &[""]))), TAG_ARMOR);
    }
}
