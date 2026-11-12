use std::collections::HashMap;

pub const WEAPON_TYPES: usize = 8;
pub const RIDE_TYPES: usize = 3;
pub const PC_SOCIALS: usize = 20;
pub const NPC_SOCIALS: usize = 5;

macro_rules! anim_kinds {
    ($($variant:ident = $key:literal),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum AnimKind {
            $($variant),*
        }

        impl AnimKind {
            pub const ALL: [AnimKind; anim_kinds!(@count $($variant)*)] = [$(AnimKind::$variant),*];

            pub fn key(self) -> &'static str {
                match self {
                    $(AnimKind::$variant => $key),*
                }
            }
        }
    };
    (@count) => { 0 };
    (@count $head:ident $($tail:ident)*) => { 1 + anim_kinds!(@count $($tail)*) };
}

anim_kinds! {
    Turn = "Turn",
    Walk = "Walk",
    Run = "Run",
    Fall = "Fall",
    DamageFly = "DamageFly",
    Jump = "Jump",
    BJump = "BJump",
    Wait = "Wait",
    AtkWait = "AtkWait",
    Sit = "Sit",
    SitWait = "SitWait",
    Stand = "Stand",
    DeathStand = "DeathStand",
    ChairSit = "ChairSit",
    ChairWait = "ChairWait",
    ChairStand = "ChairStand",
    Stun = "Stun",
    Atk01 = "Atk01",
    Atk02 = "Atk02",
    Atk03 = "Atk03",
    AtkUp = "AtkUp",
    AtkDown = "AtkDown",
    SpAtk01 = "SpAtk01",
    SpAtk02 = "SpAtk02",
    SpAtk03 = "SpAtk03",
    SpAtk04 = "SpAtk04",
    SpAtk05 = "SpAtk05",
    SpAtk06 = "SpAtk06",
    SpAtk07 = "SpAtk07",
    SpAtk08 = "SpAtk08",
    SpAtk09 = "SpAtk09",
    SpAtk10 = "SpAtk10",
    SpAtk11 = "SpAtk11",
    SpAtk12 = "SpAtk12",
    SpAtk13 = "SpAtk13",
    SpAtk14 = "SpAtk14",
    SpAtk15 = "SpAtk15",
    SpAtk16 = "SpAtk16",
    SpAtk17 = "SpAtk17",
    SpAtk18 = "SpAtk18",
    SpAtk19 = "SpAtk19",
    SpAtk20 = "SpAtk20",
    SpAtk21 = "SpAtk21",
    SpAtk22 = "SpAtk22",
    SpAtk23 = "SpAtk23",
    SpAtk24 = "SpAtk24",
    SpAtk25 = "SpAtk25",
    SpAtk26 = "SpAtk26",
    SpAtk27 = "SpAtk27",
    ShieldAtk = "ShieldAtk",
    Defence = "Defence",
    Dodge = "Dodge",
    Death = "Death",
    DeathWait = "DeathWait",
    SitDeath = "SitDeath",
    SitDeathWait = "SitDeathWait",
    Damage = "Damage",
    CastShort = "CastShort",
    CastMid = "CastMid",
    CastLong = "CastLong",
    CastEnd = "CastEnd",
    MagicThrow = "MagicThrow",
    MagicShot = "MagicShot",
    MagicNoTarget = "MagicNoTarget",
    MagicFriend = "MagicFriend",
    PicItem = "PicItem",
    Throw = "Throw",
    SwimDeath = "SwimDeath",
    Swim = "Swim",
    SwimWait = "SwimWait",
    SwimDeathWait = "SwimDeathWait",
    SwimAttackWait = "SwimAttackWait",
}

impl AnimKind {
    pub fn sp_atk(index: usize) -> Option<AnimKind> {
        (1..=27).contains(&index).then(|| AnimKind::ALL[AnimKind::SpAtk01 as usize + index - 1])
    }

    fn rate_key(key: &str) -> Option<AnimKind> {
        if key.eq_ignore_ascii_case("CastShortEnd") {
            return Some(AnimKind::CastEnd);
        }
        AnimKind::ALL.into_iter().find(|kind| kind.key().eq_ignore_ascii_case(key))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RiderKind {
    Wait,
    Atk,
    Run,
    DeathWait,
    Death,
}

impl RiderKind {
    const ALL: [RiderKind; 5] =
        [RiderKind::Wait, RiderKind::Atk, RiderKind::Run, RiderKind::DeathWait, RiderKind::Death];

    fn key(self) -> &'static str {
        match self {
            RiderKind::Wait => "RiderWait",
            RiderKind::Atk => "RiderAtk",
            RiderKind::Run => "RiderRun",
            RiderKind::DeathWait => "RiderDeathWait",
            RiderKind::Death => "RiderDeath",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FishingKind {
    Start,
    Wait,
    Control,
    Pull,
    End,
}

impl FishingKind {
    const ALL: [FishingKind; 5] =
        [FishingKind::Start, FishingKind::Wait, FishingKind::Control, FishingKind::Pull, FishingKind::End];

    fn key(self) -> &'static str {
        match self {
            FishingKind::Start => "FishStart",
            FishingKind::Wait => "FishWait",
            FishingKind::Control => "FishControl",
            FishingKind::Pull => "FishPull",
            FishingKind::End => "FishEnd",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HideWeapons {
    pub right: bool,
    pub left: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AnimSet {
    names: HashMap<AnimKind, [Option<String>; WEAPON_TYPES]>,
    rates: HashMap<AnimKind, [f32; WEAPON_TYPES]>,
    pub pc_social: [Option<String>; PC_SOCIALS],
    pub npc_social: [Option<String>; NPC_SOCIALS],
    pub pc_social_hide: [HideWeapons; PC_SOCIALS],
    pub npc_social_hide: [HideWeapons; NPC_SOCIALS],
    pub cast_hide: HideWeapons,
    pub rider: [[Option<String>; RIDE_TYPES]; 5],
    pub fishing: [Option<String>; 5],
    pub sit_rate: f32,
    pub stand_rate: f32,
}

const EMPTY_NAMES: [Option<String>; WEAPON_TYPES] = [None, None, None, None, None, None, None, None];

impl AnimSet {
    pub fn names(&self, kind: AnimKind) -> &[Option<String>; WEAPON_TYPES] {
        self.names.get(&kind).unwrap_or(&EMPTY_NAMES)
    }

    pub fn names_mut(&mut self, kind: AnimKind) -> &mut [Option<String>; WEAPON_TYPES] {
        self.names.entry(kind).or_insert(EMPTY_NAMES)
    }

    pub fn name(&self, kind: AnimKind, weapon: usize) -> Option<&str> {
        self.names(kind).get(weapon).and_then(Option::as_deref)
    }

    pub fn rate(&self, kind: AnimKind, weapon: usize) -> f32 {
        let rate = self.rates.get(&kind).and_then(|rates| rates.get(weapon)).copied().unwrap_or(0.0);
        if rate == 0.0 { 1.0 } else { rate }
    }

    pub fn set_rate(&mut self, kind: AnimKind, weapon: usize, rate: f32) {
        if let Some(slot) = self.rates.entry(kind).or_insert([0.0; WEAPON_TYPES]).get_mut(weapon) {
            *slot = rate;
        }
    }

    pub fn sit_rate(&self) -> f32 {
        if self.sit_rate == 0.0 { 1.0 } else { self.sit_rate }
    }

    pub fn stand_rate(&self) -> f32 {
        if self.stand_rate == 0.0 { 1.0 } else { self.stand_rate }
    }

    pub fn rider(&self, kind: RiderKind, ride_type: usize) -> Option<&str> {
        self.rider[kind as usize].get(ride_type).and_then(Option::as_deref)
    }

    pub fn fishing(&self, kind: FishingKind) -> Option<&str> {
        self.fishing[kind as usize].as_deref()
    }

    pub fn all_names(&self) -> impl Iterator<Item = &str> {
        let by_weapon = AnimKind::ALL.iter().flat_map(|kind| self.names(*kind).iter());
        by_weapon
            .chain(self.pc_social.iter())
            .chain(self.npc_social.iter())
            .chain(self.rider.iter().flatten())
            .chain(self.fishing.iter())
            .filter_map(Option::as_deref)
    }
}

pub fn decode_text(bytes: &[u8]) -> String {
    match bytes {
        [0xff, 0xfe, rest @ ..] => {
            let units: Vec<u16> = rest.as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).collect();
            String::from_utf16_lossy(&units)
        }
        _ => bytes.iter().map(|&b| b as char).collect(),
    }
}

pub fn parse(text: &str) -> HashMap<String, AnimSet> {
    let mut sets = HashMap::new();
    let mut section: Option<String> = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = Some(name.trim().to_ascii_lowercase());
            continue;
        }
        let (Some(section), Some((key, value))) = (&section, line.split_once('=')) else { continue };
        let Some(entry) = Entry::parse(key.trim()) else { continue };
        let set: &mut AnimSet = sets.entry(section.clone()).or_default();
        entry.apply(set, value.trim());
    }
    sets
}

enum Entry {
    Name(AnimKind, usize),
    Rate(AnimKind, usize),
    PcSocial(usize),
    NpcSocial(usize),
    PcSocialHide(usize, bool),
    NpcSocialHide(usize, bool),
    CastHide(bool),
    Rider(RiderKind, usize),
    Fishing(FishingKind),
    SitRate,
    StandRate,
}

fn strip_suffix_ignore_case<'a>(name: &'a str, suffix: &str) -> Option<&'a str> {
    let cut = name.len().checked_sub(suffix.len())?;
    (cut > 0 && name.is_char_boundary(cut) && name[cut..].eq_ignore_ascii_case(suffix)).then(|| &name[..cut])
}

impl Entry {
    fn parse(key: &str) -> Option<Entry> {
        let (name, index) = match key.strip_suffix(']').and_then(|k| k.split_once('[')) {
            Some((name, index)) => (name, Some(index.trim().parse::<usize>().ok()?)),
            None => (key, None),
        };
        match index {
            None => Entry::parse_scalar(name),
            Some(index) => Entry::parse_indexed(name, index),
        }
    }

    fn parse_scalar(name: &str) -> Option<Entry> {
        if name.eq_ignore_ascii_case("SitAnimRate") {
            return Some(Entry::SitRate);
        }
        if name.eq_ignore_ascii_case("StandAnimRate") {
            return Some(Entry::StandRate);
        }
        if name.eq_ignore_ascii_case("PcCastHideRightWeapon") {
            return Some(Entry::CastHide(true));
        }
        if name.eq_ignore_ascii_case("PcCastHideLeftWeapon") {
            return Some(Entry::CastHide(false));
        }
        let stem = strip_suffix_ignore_case(name, "AnimName")?;
        FishingKind::ALL.into_iter().find(|kind| kind.key().eq_ignore_ascii_case(stem)).map(Entry::Fishing)
    }

    fn parse_indexed(name: &str, index: usize) -> Option<Entry> {
        if let Some(stem) = strip_suffix_ignore_case(name, "AnimName") {
            if stem.eq_ignore_ascii_case("PcSocial") {
                return (index < PC_SOCIALS).then_some(Entry::PcSocial(index));
            }
            if stem.eq_ignore_ascii_case("NpcSocial") {
                return (index < NPC_SOCIALS).then_some(Entry::NpcSocial(index));
            }
            if let Some(kind) = RiderKind::ALL.into_iter().find(|kind| kind.key().eq_ignore_ascii_case(stem)) {
                return (index < RIDE_TYPES).then_some(Entry::Rider(kind, index));
            }
            let kind = AnimKind::ALL.into_iter().find(|kind| kind.key().eq_ignore_ascii_case(stem))?;
            return (index < WEAPON_TYPES).then_some(Entry::Name(kind, index));
        }
        if let Some(stem) = strip_suffix_ignore_case(name, "AnimRate") {
            let kind = AnimKind::rate_key(stem)?;
            return (index < WEAPON_TYPES).then_some(Entry::Rate(kind, index));
        }
        for (suffix, right) in [("HideRightWeapon", true), ("HideLeftWeapon", false)] {
            let Some(stem) = strip_suffix_ignore_case(name, suffix) else { continue };
            if stem.eq_ignore_ascii_case("PcSocial") {
                return (index < PC_SOCIALS).then_some(Entry::PcSocialHide(index, right));
            }
            if stem.eq_ignore_ascii_case("NpcSocial") {
                return (index < NPC_SOCIALS).then_some(Entry::NpcSocialHide(index, right));
            }
        }
        None
    }

    fn apply(self, set: &mut AnimSet, value: &str) {
        let name = || (!value.is_empty() && !value.eq_ignore_ascii_case("None")).then(|| value.to_string());
        let flag = || value.parse::<i32>().unwrap_or(0) != 0;
        let rate = || value.parse::<f32>().unwrap_or(0.0);
        match self {
            Entry::Name(kind, index) => set.names_mut(kind)[index] = name(),
            Entry::Rate(kind, index) => set.set_rate(kind, index, rate()),
            Entry::PcSocial(index) => set.pc_social[index] = name(),
            Entry::NpcSocial(index) => set.npc_social[index] = name(),
            Entry::PcSocialHide(index, true) => set.pc_social_hide[index].right = flag(),
            Entry::PcSocialHide(index, false) => set.pc_social_hide[index].left = flag(),
            Entry::NpcSocialHide(index, true) => set.npc_social_hide[index].right = flag(),
            Entry::NpcSocialHide(index, false) => set.npc_social_hide[index].left = flag(),
            Entry::CastHide(true) => set.cast_hide.right = flag(),
            Entry::CastHide(false) => set.cast_hide.left = flag(),
            Entry::Rider(kind, index) => set.rider[kind as usize][index] = name(),
            Entry::Fishing(kind) => set.fishing[kind as usize] = name(),
            Entry::SitRate => set.sit_rate = rate(),
            Entry::StandRate => set.stand_rate = rate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WARRIOR: &str = include_str!("../tests/fixtures/lineagewarrior.int.txt");
    const MONSTER: &str = include_str!("../tests/fixtures/lineagemonster.int.txt");
    const NPC: &str = include_str!("../tests/fixtures/lineagenpc.int.txt");

    #[test]
    fn fighter_names_come_by_weapon_type_and_dualfist_maps_to_hand() {
        let sets = parse(WARRIOR);
        assert_eq!(sets.len(), 1);
        let fighter = &sets["mfighter"];
        assert_eq!(fighter.name(AnimKind::Wait, 0), Some("wait_Hand_MFighter"));
        assert_eq!(fighter.name(AnimKind::Wait, 1), Some("wait_1HS_MFighter"));
        assert_eq!(fighter.name(AnimKind::Wait, 5), Some("wait_Bow_MFighter"));
        assert_eq!(fighter.name(AnimKind::Wait, 7), Some("wait_Hand_MFighter"));
        assert_eq!(fighter.name(AnimKind::AtkWait, 3), Some("atkWait_Dual_MFighter"));
        assert_eq!(fighter.name(AnimKind::Walk, 4), Some("walk_Pole_MFighter"));
        assert_eq!(fighter.name(AnimKind::Run, 2), Some("run_2HS_MFighter"));
        assert_eq!(fighter.name(AnimKind::Sit, 3), Some("Sit_MFighter"));
        assert_eq!(fighter.name(AnimKind::SitWait, 0), Some("SitWait_MFighter"));
        assert_eq!(fighter.name(AnimKind::Stand, 1), Some("Stand_MFighter"));
        assert_eq!(fighter.name(AnimKind::Death, 7), Some("Death_MFighter"));
        assert_eq!(fighter.name(AnimKind::DeathWait, 5), Some("DeathWait_MFighter"));
        assert!(fighter.names(AnimKind::Wait)[6].is_none());
    }

    #[test]
    fn empty_index_is_none_like_the_client_localized_array() {
        let sets = parse(WARRIOR);
        let fighter = &sets["mfighter"];
        assert_eq!(fighter.name(AnimKind::Wait, 6), None);
        assert_eq!(fighter.name(AnimKind::Run, 6), None);
        assert_eq!(fighter.name(AnimKind::Walk, 99), None);
        assert_eq!(AnimSet::default().name(AnimKind::Wait, 0), None);
    }

    #[test]
    fn fighter_actions_casts_socials_riders_and_fishing_are_read() {
        let sets = parse(WARRIOR);
        let fighter = &sets["mfighter"];
        assert_eq!(fighter.name(AnimKind::Turn, 1), Some("walk_1HS_MFighter"));
        assert_eq!(fighter.name(AnimKind::Atk01, 1), Some("Atk01_1HS_MFighter"));
        assert_eq!(fighter.name(AnimKind::Atk02, 0), Some("Atk01_Hand_MFighter"));
        assert_eq!(fighter.name(AnimKind::Atk03, 4), Some("Atk03_Pole_MFighter"));
        assert_eq!(fighter.name(AnimKind::PicItem, 0), Some("PicItem_MFighter"));
        assert_eq!(fighter.name(AnimKind::CastShort, 0), Some("CastShort_MFighter"));
        assert_eq!(fighter.name(AnimKind::CastMid, 0), Some("CastMid_MFighter"));
        assert_eq!(fighter.name(AnimKind::CastLong, 0), Some("CastLong_MFighter"));
        assert_eq!(fighter.name(AnimKind::CastEnd, 0), Some("CastEnd_MFighter"));
        assert_eq!(fighter.name(AnimKind::MagicShot, 0), Some("MagicShot_MFighter"));
        assert_eq!(fighter.name(AnimKind::MagicThrow, 0), Some("MagicThrow_MFighter"));
        assert_eq!(fighter.name(AnimKind::MagicNoTarget, 0), Some("MagicNoTarget_MFighter"));
        assert_eq!(fighter.name(AnimKind::MagicFriend, 0), None);
        assert_eq!(fighter.name(AnimKind::SpAtk01, 1), Some("SpAtk01_1HS_MFighter"));
        assert_eq!(fighter.name(AnimKind::SpAtk05, 0), Some("Social_dance_MFighter"));
        assert_eq!(fighter.name(AnimKind::SpAtk06, 0), Some("spatk06_Hand_MFighter"));
        assert_eq!(fighter.name(AnimKind::SpAtk27, 3), Some("Social_dance_MFighter"));
        assert_eq!(fighter.name(AnimKind::ShieldAtk, 0), Some("shieldatk_1HS_MFighter"));
        assert_eq!(fighter.name(AnimKind::Fall, 0), Some("Falling_MFighter"));
        assert_eq!(fighter.name(AnimKind::DeathStand, 0), Some("Rebirth_MFighter"));
        assert_eq!(fighter.name(AnimKind::DamageFly, 0), Some("DamageFly_MFighter"));
        assert_eq!(fighter.name(AnimKind::Damage, 0), None);
        assert_eq!(fighter.name(AnimKind::Swim, 0), Some("Swim_MFighter"));
        assert_eq!(fighter.name(AnimKind::SwimWait, 0), Some("SwimWait_MFighter"));
        assert_eq!(fighter.name(AnimKind::SwimAttackWait, 0), None);
        assert_eq!(fighter.name(AnimKind::SwimDeath, 0), Some("SwimDeath_MFighter"));
        assert_eq!(fighter.name(AnimKind::SwimDeathWait, 0), Some("SwimDeathWait_MFighter"));
        assert_eq!(fighter.name(AnimKind::ChairSit, 0), Some("ChairSit_MFighter"));
        assert_eq!(fighter.name(AnimKind::ChairWait, 0), Some("ChairWait_MFighter"));
        assert_eq!(fighter.name(AnimKind::ChairStand, 0), Some("ChairStand_MFighter"));
        assert_eq!(fighter.rider(RiderKind::Wait, 1), Some("Strider_MFighter"));
        assert_eq!(fighter.rider(RiderKind::Run, 2), Some("wyvern_MFighter"));
        assert_eq!(fighter.rider(RiderKind::Death, 0), None);
        assert_eq!(fighter.pc_social[2].as_deref(), Some("Social_Nod_MFighter"));
        assert_eq!(fighter.pc_social[13].as_deref(), Some("Social_sad_MFighter"));
        assert_eq!(fighter.pc_social[0], None);
        assert_eq!(fighter.pc_social[14], None);
        assert_eq!(fighter.pc_social_hide[2], HideWeapons { right: true, left: false });
        assert_eq!(fighter.pc_social_hide[3], HideWeapons { right: false, left: false });
        assert_eq!(fighter.pc_social_hide[12], HideWeapons { right: true, left: true });
        assert_eq!(fighter.fishing(FishingKind::Start), Some("Fishing_fly_MFighter"));
        assert_eq!(fighter.fishing(FishingKind::Wait), Some("Fishing_wait_MFighter"));
        assert_eq!(fighter.fishing(FishingKind::Control), Some("Fishing_control_MFighter"));
        assert_eq!(fighter.fishing(FishingKind::Pull), Some("Fishing_pull_MFighter"));
        assert_eq!(fighter.fishing(FishingKind::End), Some("Fishing_end_MFighter"));
    }

    #[test]
    fn fighter_rates_default_to_one_when_absent_or_zero() {
        let sets = parse(WARRIOR);
        let fighter = &sets["mfighter"];
        assert!((fighter.rate(AnimKind::Atk01, 0) - 0.8333).abs() < 1e-6);
        assert!((fighter.rate(AnimKind::Atk03, 5) - 1.1111).abs() < 1e-6);
        assert_eq!(fighter.rate(AnimKind::Atk01, 6), 1.0);
        assert_eq!(fighter.rate(AnimKind::Walk, 0), 1.0);
        assert_eq!(fighter.rate(AnimKind::CastShort, 0), 1.0);
        assert!((fighter.sit_rate() - 1.27451).abs() < 1e-6);
        assert!((fighter.stand_rate() - 1.01307).abs() < 1e-6);
        assert_eq!(AnimSet::default().sit_rate(), 1.0);
        assert_eq!(AnimSet::default().stand_rate(), 1.0);
        let mut set = AnimSet::default();
        set.set_rate(AnimKind::Run, 0, 1.42857);
        set.set_rate(AnimKind::Run, 9, 3.0);
        assert!((set.rate(AnimKind::Run, 0) - 1.42857).abs() < 1e-6);
        assert_eq!(set.rate(AnimKind::Run, 1), 1.0);
    }

    #[test]
    fn monster_values_lose_trailing_tabs() {
        let sets = parse(MONSTER);
        let gremlin = &sets["gremlin"];
        assert_eq!(gremlin.name(AnimKind::Walk, 0), Some("walk"));
        assert_eq!(gremlin.name(AnimKind::Run, 0), Some("run"));
        assert_eq!(gremlin.name(AnimKind::Wait, 0), Some("wait"));
        assert_eq!(gremlin.name(AnimKind::AtkWait, 0), Some("atkwait"));
        assert_eq!(gremlin.name(AnimKind::Death, 0), Some("death"));
        assert_eq!(gremlin.name(AnimKind::DeathWait, 0), Some("deathwait"));
        assert_eq!(gremlin.name(AnimKind::Atk01, 0), Some("atk01"));
        assert_eq!(gremlin.name(AnimKind::CastShort, 0), Some("atkwait"));
        assert_eq!(gremlin.name(AnimKind::MagicShot, 0), Some("atk01"));
        assert_eq!(gremlin.name(AnimKind::Sit, 0), None);
        assert_eq!(gremlin.name(AnimKind::Wait, 3), None);
    }

    #[test]
    fn npc_section_key_is_lowercase_and_carries_npc_socials() {
        let sets = parse(NPC);
        let trader = &sets["a_tradera_mhuman"];
        assert_eq!(trader.name(AnimKind::DeathWait, 0), Some("DeathWait_Hand"));
        assert_eq!(trader.name(AnimKind::Wait, 1), Some("wait_1HS"));
        assert_eq!(trader.name(AnimKind::Wait, 2), Some("wait_1HS"));
        assert_eq!(trader.name(AnimKind::Wait, 3), None);
        assert_eq!(trader.name(AnimKind::Death, 2), Some("death_Hand"));
        assert_eq!(trader.npc_social[0].as_deref(), Some("social01"));
        assert_eq!(trader.npc_social[2].as_deref(), Some("spwait02"));
        assert_eq!(trader.npc_social[3], None);
        assert_eq!(trader.npc_social_hide[1], HideWeapons { right: true, left: true });
        assert_eq!(trader.npc_social_hide[3], HideWeapons::default());
        assert!(!sets.contains_key("a_traderA_MHuman"));
    }

    #[test]
    fn sections_without_anim_names_are_skipped_and_repeated_sections_merge() {
        let text = "[Config]\nNUM=3\n\n[Gremlin]\nWaitAnimName[0]=a\n[Other]\nRunAnimName[0]=x\n[gremlin]\nRunAnimName[0]=b\nWaitAnimName[0]=c\nWaitAnimName[9]=z\nWaitAnimName[x]=z\nFoo[0]=y\nSpatk01AnimName[0]=s\nSwimAtkWaitAnimName[0]=w\nWalkAnimName[1]=None\nPcCastHideRightWeapon=1\nAnimName[0]=q\n";
        let sets = parse(text);
        assert_eq!(sets.len(), 2);
        let gremlin = &sets["gremlin"];
        assert_eq!(gremlin.name(AnimKind::Wait, 0), Some("c"));
        assert_eq!(gremlin.name(AnimKind::Run, 0), Some("b"));
        assert_eq!(gremlin.name(AnimKind::SpAtk01, 0), Some("s"));
        assert_eq!(gremlin.name(AnimKind::SwimAttackWait, 0), None);
        assert_eq!(gremlin.name(AnimKind::Walk, 1), None);
        assert_eq!(gremlin.cast_hide, HideWeapons { right: true, left: false });
        assert_eq!(sets["other"].name(AnimKind::Run, 0), Some("x"));
        assert_eq!(sets["other"].name(AnimKind::Wait, 0), None);
    }

    #[test]
    fn sp_atk_index_maps_to_the_kind_and_all_names_lists_every_sequence() {
        assert_eq!(AnimKind::sp_atk(1), Some(AnimKind::SpAtk01));
        assert_eq!(AnimKind::sp_atk(27), Some(AnimKind::SpAtk27));
        assert_eq!(AnimKind::sp_atk(0), None);
        assert_eq!(AnimKind::sp_atk(28), None);
        assert_eq!(AnimKind::ALL.len(), 72);
        let sets = parse(NPC);
        let names: Vec<&str> = sets["a_tradera_mhuman"].all_names().collect();
        assert!(names.contains(&"wait_Hand"));
        assert!(names.contains(&"spwait02"));
        assert_eq!(names.len(), 43);
    }

    #[test]
    fn decode_text_reads_one_byte_chars_and_utf16le_with_bom() {
        let ascii: Vec<u8> = MONSTER.bytes().collect();
        let mut utf16: Vec<u8> = vec![0xff, 0xfe];
        utf16.extend(MONSTER.encode_utf16().flat_map(u16::to_le_bytes));
        assert_eq!(parse(&decode_text(&ascii)), parse(&decode_text(&utf16)));
        assert_eq!(decode_text(b"[a]\r\nWaitAnimName[0]=\xe9\r\n"), "[a]\r\nWaitAnimName[0]=é\r\n");
    }
}
