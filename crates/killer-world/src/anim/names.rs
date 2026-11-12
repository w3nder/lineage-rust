use l2_dat::animnames::{AnimKind, AnimSet, FishingKind, RiderKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WaitType {
    Sit,
    #[default]
    Stand,
    FakeDead,
    FakeDeadStand,
    ChairSit,
}

impl WaitType {
    pub fn code(self) -> u8 {
        match self {
            WaitType::Sit => 0,
            WaitType::Stand => 1,
            WaitType::FakeDead => 2,
            WaitType::FakeDeadStand => 3,
            WaitType::ChairSit => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MoveType {
    Slow,
    #[default]
    Fast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Physics {
    #[default]
    Walking,
    Falling,
    Swimming,
    Flying,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FishingType {
    #[default]
    None,
    Wait,
    Battle,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PawnView {
    pub weapon_type: usize,
    pub npc: bool,
    pub ride_type: Option<usize>,
    pub ride_pawn_type: Option<usize>,
    pub physics: Physics,
    pub hit_ground_in_water: bool,
    pub wait_type: WaitType,
    pub move_type: MoveType,
    pub dead: bool,
    pub auto_attacking: bool,
    pub fishing: FishingType,
    pub non_attack_speed_rate: f32,
    pub attack_speed_rate: f32,
    pub skill_speed_rate: f32,
    pub walk_anim_frame_in_move: i32,
    pub same_anim: bool,
    pub npc_speed: f32,
}

impl PawnView {
    pub fn standing(weapon_type: usize) -> Self {
        PawnView {
            weapon_type,
            non_attack_speed_rate: 1.0,
            attack_speed_rate: 1.0,
            skill_speed_rate: 1.0,
            walk_anim_frame_in_move: 12,
            ..Default::default()
        }
    }

    fn name<'a>(&self, set: &'a AnimSet, kind: AnimKind) -> Option<&'a str> {
        set.name(kind, self.weapon_type)
    }

    fn rider<'a>(&self, set: &'a AnimSet, kind: RiderKind) -> Option<&'a str> {
        set.rider(kind, self.ride_type?)
    }
}

pub fn wait_anim_name<'a>(set: &'a AnimSet, view: &PawnView) -> Option<&'a str> {
    view.name(set, AnimKind::Wait)
}

pub fn walk_anim_name<'a>(set: &'a AnimSet, view: &PawnView) -> Option<&'a str> {
    view.name(set, AnimKind::Walk)
}

pub fn run_anim_name<'a>(set: &'a AnimSet, view: &PawnView) -> Option<&'a str> {
    view.name(set, AnimKind::Run)
}

pub fn death_anim_name<'a>(set: &'a AnimSet, view: &PawnView) -> Option<&'a str> {
    if view.physics == Physics::Swimming
        && let Some(name) = view.name(set, AnimKind::SwimDeath)
    {
        return Some(name);
    }
    if view.wait_type == WaitType::Sit
        && let Some(name) = view.name(set, AnimKind::SitDeath)
    {
        return Some(name);
    }
    view.name(set, AnimKind::Death)
}

pub fn death_wait_anim_name<'a>(set: &'a AnimSet, view: &PawnView) -> Option<&'a str> {
    if view.physics == Physics::Swimming
        && let Some(name) = view.name(set, AnimKind::SwimDeathWait)
    {
        return Some(name);
    }
    if view.wait_type == WaitType::Sit
        && let Some(name) = view.name(set, AnimKind::SitDeathWait)
    {
        return Some(name);
    }
    view.name(set, AnimKind::DeathWait)
}

pub fn cur_wait_anim_name<'a>(set: &'a AnimSet, view: &PawnView) -> Option<&'a str> {
    if view.dead {
        return if view.ride_type.is_some() {
            view.rider(set, RiderKind::DeathWait)
        } else {
            death_wait_anim_name(set, view)
        };
    }
    if view.ride_type.is_some() {
        return view.rider(set, RiderKind::Wait);
    }
    match view.fishing {
        FishingType::Wait => return set.fishing(FishingKind::Wait),
        FishingType::Battle => return set.fishing(FishingKind::Control),
        FishingType::None => {}
    }
    match view.wait_type {
        WaitType::Sit => return view.name(set, AnimKind::SitWait),
        WaitType::FakeDead => return death_wait_anim_name(set, view),
        WaitType::ChairSit => return view.name(set, AnimKind::ChairWait),
        WaitType::Stand | WaitType::FakeDeadStand => {}
    }
    let swimming = view.physics == Physics::Swimming && !view.hit_ground_in_water;
    if view.auto_attacking {
        if swimming && let Some(name) = view.name(set, AnimKind::SwimAttackWait) {
            return Some(name);
        }
        return view.name(set, AnimKind::AtkWait);
    }
    if view.wait_type == WaitType::Stand
        && swimming
        && let Some(name) = view.name(set, AnimKind::SwimWait)
    {
        return Some(name);
    }
    view.name(set, AnimKind::Wait)
}

pub fn script_cur_wait_anim_name<'a>(set: &'a AnimSet, view: &PawnView) -> Option<&'a str> {
    if view.dead {
        return death_wait_anim_name(set, view);
    }
    match view.wait_type {
        WaitType::Sit => return view.name(set, AnimKind::SitWait),
        WaitType::FakeDead => return death_wait_anim_name(set, view),
        _ => {}
    }
    if view.auto_attacking {
        if view.physics == Physics::Swimming
            && let Some(name) = view.name(set, AnimKind::SwimAttackWait)
        {
            return Some(name);
        }
        return view.name(set, AnimKind::AtkWait);
    }
    if view.wait_type == WaitType::Stand
        && view.physics == Physics::Swimming
        && let Some(name) = view.name(set, AnimKind::SwimWait)
    {
        return Some(name);
    }
    view.name(set, AnimKind::Wait)
}

pub fn cur_move_anim_name<'a>(set: &'a AnimSet, view: &PawnView) -> Option<&'a str> {
    if view.ride_type.is_some() {
        return view.rider(set, RiderKind::Run);
    }
    match view.move_type {
        MoveType::Slow => walk_anim_name(set, view),
        MoveType::Fast => run_anim_name(set, view),
    }
}

pub fn cur_turn_anim_name<'a>(set: &'a AnimSet, view: &PawnView) -> Option<&'a str> {
    if view.ride_type.is_some() {
        return view.rider(set, RiderKind::Wait);
    }
    view.name(set, AnimKind::Turn)
}

pub const ATTACK_KINDS: [AnimKind; 3] = [AnimKind::Atk01, AnimKind::Atk02, AnimKind::Atk03];

pub fn random_atk_anim_kind(random: u32) -> AnimKind {
    ATTACK_KINDS[(random % 3) as usize]
}

pub fn random_atk_anim_name<'a>(set: &'a AnimSet, view: &PawnView, random: u32) -> Option<&'a str> {
    if view.ride_type.is_some() {
        return view.rider(set, RiderKind::Atk);
    }
    view.name(set, random_atk_anim_kind(random))
}

pub fn cur_atk_anim_name<'a>(set: &'a AnimSet, view: &PawnView, current: Option<&str>, random: u32) -> Option<&'a str> {
    if view.ride_type.is_some() {
        return view.rider(set, RiderKind::Atk);
    }
    let is_attack = current.is_some_and(|current| {
        ATTACK_KINDS.iter().any(|kind| view.name(set, *kind).is_some_and(|name| name.eq_ignore_ascii_case(current)))
    });
    if is_attack {
        return current.and_then(|current| {
            ATTACK_KINDS.iter().find_map(|kind| view.name(set, *kind).filter(|name| name.eq_ignore_ascii_case(current)))
        });
    }
    view.name(set, random_atk_anim_kind(random))
}

pub fn is_cast_anim(set: &AnimSet, view: &PawnView, current: &str) -> bool {
    [AnimKind::CastShort, AnimKind::CastMid, AnimKind::CastLong, AnimKind::CastEnd, AnimKind::MagicShot]
        .iter()
        .any(|kind| view.name(set, *kind).is_some_and(|name| name.eq_ignore_ascii_case(current)))
}

pub fn social_anim_name(set: &AnimSet, npc: bool, id: usize) -> Option<&str> {
    if npc {
        (1..=set.npc_social.len()).contains(&id).then(|| set.npc_social[id - 1].as_deref()).flatten()
    } else {
        set.pc_social.get(id).and_then(Option::as_deref)
    }
}

pub fn social_anim_duration(
    seq_seconds: impl Fn(&str) -> Option<f32>,
    set: &AnimSet,
    view: &PawnView,
    id: usize,
) -> f32 {
    if view.npc {
        return -1.0;
    }
    let (name, extra) = match id {
        0 => (view.name(set, AnimKind::Sit), 0.0),
        1 => (view.name(set, AnimKind::Stand), 0.0),
        2..=19 => (set.pc_social.get(id).and_then(Option::as_deref), 0.3),
        _ => return -1.0,
    };
    match name.and_then(&seq_seconds) {
        Some(seconds) => (seconds + extra) / view.non_attack_speed_rate,
        None => -1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2_dat::animnames::parse;

    const WARRIOR: &str = include_str!("../../../l2-dat/tests/fixtures/lineagewarrior.int.txt");
    const NPC: &str = include_str!("../../../l2-dat/tests/fixtures/lineagenpc.int.txt");

    fn fighter() -> AnimSet {
        parse(WARRIOR).remove("mfighter").unwrap()
    }

    #[test]
    fn wait_follows_the_native_get_cur_wait_anim_name() {
        let set = fighter();
        let mut view = PawnView::standing(1);
        assert_eq!(cur_wait_anim_name(&set, &view), Some("wait_1HS_MFighter"));
        view.auto_attacking = true;
        assert_eq!(cur_wait_anim_name(&set, &view), Some("atkWait_1HS_MFighter"));
        view.physics = Physics::Swimming;
        assert_eq!(cur_wait_anim_name(&set, &view), Some("atkWait_1HS_MFighter"));
        view.auto_attacking = false;
        assert_eq!(cur_wait_anim_name(&set, &view), Some("SwimWait_MFighter"));
        view.hit_ground_in_water = true;
        assert_eq!(cur_wait_anim_name(&set, &view), Some("wait_1HS_MFighter"));
        view.physics = Physics::Walking;
        view.wait_type = WaitType::Sit;
        assert_eq!(cur_wait_anim_name(&set, &view), Some("SitWait_MFighter"));
        view.wait_type = WaitType::ChairSit;
        assert_eq!(cur_wait_anim_name(&set, &view), Some("ChairWait_MFighter"));
        view.wait_type = WaitType::FakeDead;
        assert_eq!(cur_wait_anim_name(&set, &view), Some("DeathWait_MFighter"));
        view.wait_type = WaitType::FakeDeadStand;
        assert_eq!(cur_wait_anim_name(&set, &view), Some("wait_1HS_MFighter"));
        view.fishing = FishingType::Wait;
        assert_eq!(cur_wait_anim_name(&set, &view), Some("Fishing_wait_MFighter"));
        view.fishing = FishingType::Battle;
        assert_eq!(cur_wait_anim_name(&set, &view), Some("Fishing_control_MFighter"));
        view.fishing = FishingType::None;
        view.ride_type = Some(1);
        assert_eq!(cur_wait_anim_name(&set, &view), Some("Strider_MFighter"));
        view.dead = true;
        assert_eq!(cur_wait_anim_name(&set, &view), Some("Strider_MFighter"));
        view.ride_type = Some(0);
        assert_eq!(cur_wait_anim_name(&set, &view), None);
        view.ride_type = None;
        assert_eq!(cur_wait_anim_name(&set, &view), Some("DeathWait_MFighter"));
        view.physics = Physics::Swimming;
        assert_eq!(cur_wait_anim_name(&set, &view), Some("SwimDeathWait_MFighter"));
    }

    #[test]
    fn script_wait_ignores_chair_fishing_and_riding() {
        let set = fighter();
        let mut view = PawnView::standing(0);
        view.wait_type = WaitType::ChairSit;
        assert_eq!(script_cur_wait_anim_name(&set, &view), Some("wait_Hand_MFighter"));
        view.fishing = FishingType::Wait;
        assert_eq!(script_cur_wait_anim_name(&set, &view), Some("wait_Hand_MFighter"));
        view.wait_type = WaitType::Sit;
        assert_eq!(script_cur_wait_anim_name(&set, &view), Some("SitWait_MFighter"));
        view.wait_type = WaitType::Stand;
        view.auto_attacking = true;
        assert_eq!(script_cur_wait_anim_name(&set, &view), Some("atkWait_Hand_MFighter"));
    }

    #[test]
    fn death_names_prefer_swimming_then_sitting() {
        let set = fighter();
        let mut view = PawnView::standing(2);
        assert_eq!(death_anim_name(&set, &view), Some("Death_MFighter"));
        view.wait_type = WaitType::Sit;
        assert_eq!(death_anim_name(&set, &view), Some("Death_MFighter"));
        view.physics = Physics::Swimming;
        assert_eq!(death_anim_name(&set, &view), Some("SwimDeath_MFighter"));
        assert_eq!(death_wait_anim_name(&set, &view), Some("SwimDeathWait_MFighter"));
    }

    #[test]
    fn move_and_turn_names_follow_move_type_and_riding() {
        let set = fighter();
        let mut view = PawnView::standing(5);
        assert_eq!(cur_move_anim_name(&set, &view), Some("run_Bow_MFighter"));
        view.move_type = MoveType::Slow;
        assert_eq!(cur_move_anim_name(&set, &view), Some("walk_Bow_MFighter"));
        assert_eq!(cur_turn_anim_name(&set, &view), Some("walk_Bow_MFighter"));
        view.ride_type = Some(2);
        assert_eq!(cur_move_anim_name(&set, &view), Some("wyvern_MFighter"));
        assert_eq!(cur_turn_anim_name(&set, &view), Some("wyvern_MFighter"));
        view.ride_type = Some(0);
        assert_eq!(cur_move_anim_name(&set, &view), None);
    }

    #[test]
    fn attack_names_are_random_unless_one_is_already_playing() {
        let set = fighter();
        let view = PawnView::standing(1);
        assert_eq!(random_atk_anim_name(&set, &view, 0), Some("Atk01_1HS_MFighter"));
        assert_eq!(random_atk_anim_name(&set, &view, 4), Some("Atk02_1HS_MFighter"));
        assert_eq!(random_atk_anim_name(&set, &view, 5), Some("Atk03_1HS_MFighter"));
        assert_eq!(cur_atk_anim_name(&set, &view, Some("atk02_1hs_mfighter"), 0), Some("Atk02_1HS_MFighter"));
        assert_eq!(cur_atk_anim_name(&set, &view, Some("wait_1HS_MFighter"), 2), Some("Atk03_1HS_MFighter"));
        assert_eq!(cur_atk_anim_name(&set, &view, None, 1), Some("Atk02_1HS_MFighter"));
        let mut rider = view;
        rider.ride_type = Some(1);
        assert_eq!(cur_atk_anim_name(&set, &rider, None, 1), Some("Strider_MFighter"));
        assert!(is_cast_anim(&set, &view, "castend_mfighter"));
        assert!(is_cast_anim(&set, &view, "MagicShot_MFighter"));
        assert!(!is_cast_anim(&set, &view, "MagicThrow_MFighter"));
    }

    #[test]
    fn social_names_index_pc_directly_and_npc_from_one() {
        let set = fighter();
        assert_eq!(social_anim_name(&set, false, 2), Some("Social_Nod_MFighter"));
        assert_eq!(social_anim_name(&set, false, 13), Some("Social_sad_MFighter"));
        assert_eq!(social_anim_name(&set, false, 1), None);
        assert_eq!(social_anim_name(&set, false, 20), None);
        let npc = parse(NPC).remove("a_tradera_mhuman").unwrap();
        assert_eq!(social_anim_name(&npc, true, 1), Some("social01"));
        assert_eq!(social_anim_name(&npc, true, 3), Some("spwait02"));
        assert_eq!(social_anim_name(&npc, true, 0), None);
        assert_eq!(social_anim_name(&npc, true, 6), None);
    }

    #[test]
    fn social_duration_adds_the_lead_and_divides_by_the_move_multiplier() {
        let set = fighter();
        let mut view = PawnView::standing(0);
        view.non_attack_speed_rate = 2.0;
        let seconds = |name: &str| match name {
            "Sit_MFighter" => Some(41.0 / 12.0),
            "Social_Nod_MFighter" => Some(1.5),
            _ => None,
        };
        assert!((social_anim_duration(seconds, &set, &view, 0) - 41.0 / 12.0 / 2.0).abs() < 1e-6);
        assert!((social_anim_duration(seconds, &set, &view, 2) - 0.9).abs() < 1e-6);
        assert_eq!(social_anim_duration(seconds, &set, &view, 3), -1.0);
        assert_eq!(social_anim_duration(seconds, &set, &view, 20), -1.0);
        view.npc = true;
        assert_eq!(social_anim_duration(seconds, &set, &view, 2), -1.0);
    }
}
