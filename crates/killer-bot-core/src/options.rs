use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Trait {
    #[default]
    Always,
    Cp,
    Hp,
    Mp,
    Dead,
    Charges,
    Souls,
    DeathPenalty,
    Bleeding,
    Poisoned,
    Iced,
    Shackled,
    Feared,
    Stunned,
    Slept,
    Muted,
    Rooted,
    Paralyzed,
    Petrified,
    UltimateDefense,
}

impl Trait {
    pub const BUFF_HEAL: [Trait; 20] = [
        Trait::Always,
        Trait::Cp,
        Trait::Hp,
        Trait::Mp,
        Trait::Dead,
        Trait::Charges,
        Trait::Souls,
        Trait::DeathPenalty,
        Trait::Bleeding,
        Trait::Poisoned,
        Trait::Iced,
        Trait::Shackled,
        Trait::Feared,
        Trait::Stunned,
        Trait::Slept,
        Trait::Muted,
        Trait::Rooted,
        Trait::Paralyzed,
        Trait::Petrified,
        Trait::UltimateDefense,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Trait::Always => "Always",
            Trait::Cp => "CP",
            Trait::Hp => "HP",
            Trait::Mp => "MP",
            Trait::Dead => "Dead",
            Trait::Charges => "Charges",
            Trait::Souls => "Souls",
            Trait::DeathPenalty => "Death Penalty",
            Trait::Bleeding => "Bleeding",
            Trait::Poisoned => "Poisoned",
            Trait::Iced => "Iced",
            Trait::Shackled => "Shackled",
            Trait::Feared => "Feared",
            Trait::Stunned => "Stunned",
            Trait::Slept => "Slept",
            Trait::Muted => "Muted",
            Trait::Rooted => "Rooted",
            Trait::Paralyzed => "Paralyzed",
            Trait::Petrified => "Petrified",
            Trait::UltimateDefense => "Ultimate Defense",
        }
    }

    pub fn index(self) -> usize {
        Self::BUFF_HEAL.iter().position(|candidate| *candidate == self).unwrap_or(0)
    }

    pub fn from_index(index: usize) -> Trait {
        Self::BUFF_HEAL.get(index).copied().unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BuffHeal {
    pub skill_id: i32,
    pub on: bool,
    pub trait_: Trait,
    pub names: String,
    pub below_percent: u8,
    pub delay_seconds: u32,
    pub mp_above: u32,
    pub need_target: bool,
}

impl Default for BuffHeal {
    fn default() -> Self {
        Self { skill_id: 0, on: true, trait_: Trait::Always, names: String::new(), below_percent: 60, delay_seconds: 1, mp_above: 100, need_target: false }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToggleTrait {
    #[default]
    Hp,
    Mp,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Toggle {
    pub skill_id: i32,
    pub on: bool,
    pub trait_: ToggleTrait,
    pub below_percent: u8,
    pub above_percent: u8,
}

impl Default for Toggle {
    fn default() -> Self {
        Self { skill_id: 0, on: true, trait_: ToggleTrait::Hp, below_percent: 100, above_percent: 100 }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ItemRule {
    pub item_id: i32,
    pub on: bool,
    pub trait_: Trait,
    pub below_percent: u8,
    pub delay_ms: u32,
}

impl Default for ItemRule {
    fn default() -> Self {
        Self { item_id: 0, on: true, trait_: Trait::Always, below_percent: 60, delay_ms: 100 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Comparison {
    #[default]
    AtLeast,
    AtMost,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CombatRule {
    pub on: bool,
    pub page: u8,
    pub slot: u8,
    pub trait_: Trait,
    pub comparison: Comparison,
    pub percent: u8,
    pub delay_ms: u32,
    pub mp_above: u32,
}

impl Default for CombatRule {
    fn default() -> Self {
        Self { on: true, page: 1, slot: 1, trait_: Trait::Always, comparison: Comparison::AtLeast, percent: 60, delay_ms: 1, mp_above: 100 }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Polygon {
    pub points: Vec<Point>,
    pub z_range: i32,
}

impl Default for Polygon {
    fn default() -> Self {
        Self { points: Vec::new(), z_range: 1000 }
    }
}

impl Polygon {
    pub fn contains(&self, x: f32, y: f32, z: f32, center_z: f32) -> bool {
        if self.points.len() < 3 {
            return true;
        }
        if (z - center_z).abs() > self.z_range as f32 {
            return false;
        }
        let mut inside = false;
        let mut previous = self.points.len() - 1;
        for index in 0..self.points.len() {
            let (a, b) = (&self.points[index], &self.points[previous]);
            let (ax, ay, bx, by) = (a.x as f32, a.y as f32, b.x as f32, b.y as f32);
            if (ay > y) != (by > y) && x < (bx - ax) * (y - ay) / (by - ay) + ax {
                inside = !inside;
            }
            previous = index;
        }
        inside
    }

    pub fn regular(center: (i32, i32), sides: u32, radius: f32, offset_degrees: f32) -> Vec<Point> {
        (0..sides.max(3))
            .map(|index| {
                let angle = (offset_degrees + 360.0 * index as f32 / sides.max(3) as f32).to_radians();
                Point { x: center.0 + (radius * angle.cos()).round() as i32, y: center.1 + (radius * angle.sin()).round() as i32 }
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LootType {
    #[default]
    FindersKeepers,
    Random,
    RandomIncludingSpoil,
    ByTurn,
    ByTurnIncludingSpoil,
}

impl LootType {
    pub const ALL: [LootType; 5] = [LootType::FindersKeepers, LootType::Random, LootType::RandomIncludingSpoil, LootType::ByTurn, LootType::ByTurnIncludingSpoil];

    pub fn distribution(self) -> i32 {
        Self::ALL.iter().position(|candidate| *candidate == self).unwrap_or(0) as i32
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartyOptions {
    pub accept_rez: bool,
    pub accept_rez_names: String,
    pub accept_rez_party: bool,
    pub accept_rez_alliance: bool,
    pub accept_rez_clan: bool,
    pub accept_party: bool,
    pub accept_party_names: String,
    pub accept_party_alliance: bool,
    pub accept_party_clan: bool,
    pub drop_party_if_leader: bool,
    pub auto_invite: bool,
    pub auto_invite_names: String,
    pub oop_members: bool,
    pub oop_names: String,
    pub loot_type: LootType,
    pub buff_range: f32,
    pub buff_with_control: bool,
    pub buff_with_shift: bool,
}

impl Default for PartyOptions {
    fn default() -> Self {
        Self {
            accept_rez: false,
            accept_rez_names: String::new(),
            accept_rez_party: false,
            accept_rez_alliance: false,
            accept_rez_clan: false,
            accept_party: false,
            accept_party_names: String::new(),
            accept_party_alliance: false,
            accept_party_clan: false,
            drop_party_if_leader: false,
            auto_invite: false,
            auto_invite_names: String::new(),
            oop_members: false,
            oop_names: String::new(),
            loot_type: LootType::FindersKeepers,
            buff_range: 550.0,
            buff_with_control: false,
            buff_with_shift: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FollowStyle {
    #[default]
    Walker,
    L2Net,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FollowOptions {
    pub active_follow: bool,
    pub name: String,
    pub distance: f32,
    pub style: FollowStyle,
    pub attack: bool,
    pub instant_attack: bool,
    pub target: bool,
}

impl Default for FollowOptions {
    fn default() -> Self {
        Self { active_follow: false, name: String::new(), distance: 150.0, style: FollowStyle::Walker, attack: false, instant_attack: false, target: false }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct PickupOptions {
    pub pickup: bool,
    pub after_attack: bool,
    pub range: f32,
    pub only_mine: bool,
}

impl Default for PickupOptions {
    fn default() -> Self {
        Self { pickup: false, after_attack: false, range: 250.0, only_mine: false }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NoMobsOptions {
    pub move_to_loc: bool,
    pub out_of_combat: bool,
    pub leash: f32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Default for NoMobsOptions {
    fn default() -> Self {
        Self { move_to_loc: false, out_of_combat: false, leash: 100.0, x: 0, y: 0, z: 0 }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct PetOptions {
    pub summon_instant_attack: bool,
    pub solo_attack: bool,
    pub summon_auto_assist: bool,
    pub auto_assist: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DeadOptions {
    pub toggle_botting: bool,
    pub return_enabled: bool,
    pub return_delay_seconds: u32,
    pub return_point: u8,
    pub logout_enabled: bool,
    pub logout_delay_seconds: u32,
}

impl Default for DeadOptions {
    fn default() -> Self {
        Self { toggle_botting: false, return_enabled: false, return_delay_seconds: 10, return_point: 0, logout_enabled: false, logout_delay_seconds: 30 }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AttackOptions {
    pub cancel_target_on_dead: bool,
    pub move_before_attack: bool,
    pub move_range: f32,
    pub active_attack: bool,
    pub smart_move_before_attack: bool,
    pub move_before_targeting: bool,
    pub active_target: bool,
    pub auto_unstuck: bool,
    pub auto_blacklist: bool,
}

impl Default for AttackOptions {
    fn default() -> Self {
        Self {
            cancel_target_on_dead: false,
            move_before_attack: false,
            move_range: 150.0,
            active_attack: false,
            smart_move_before_attack: false,
            move_before_targeting: false,
            active_target: false,
            auto_unstuck: false,
            auto_blacklist: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SpoilOptions {
    pub auto_spoil: bool,
    pub auto_sweep: bool,
    pub plunder: bool,
    pub spoil_crush: bool,
    pub until_success: bool,
    pub mp_above: u32,
}

impl Default for SpoilOptions {
    fn default() -> Self {
        Self { auto_spoil: false, auto_sweep: false, plunder: false, spoil_crush: false, until_success: false, mp_above: 50 }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AdvancedOptions {
    pub window_title: String,
    pub pickup_timeout: u32,
    pub anti_ks_delay: u32,
    pub autofollow_delay: u32,
    pub blacklist_tries: u32,
}

impl Default for AdvancedOptions {
    fn default() -> Self {
        Self { window_title: String::new(), pickup_timeout: 5, anti_ks_delay: 0, autofollow_delay: 0, blacklist_tries: 3 }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct RestOptions {
    pub follow_rest: bool,
    pub follow_rest_name: String,
    pub below_hp: Option<u8>,
    pub below_mp: Option<u8>,
    pub until_hp: Option<u8>,
    pub until_mp: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KillSteal {
    #[default]
    Dont,
    Only,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BoxFilter {
    #[default]
    In,
    NotIn,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AliveFilter {
    #[default]
    Alive,
    Dead,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AttackableFilter {
    #[default]
    Attackable,
    Invincible,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KindFilter {
    #[default]
    Npcs,
    Players,
    Both,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct TargetingOptions {
    pub kill_steal: KillSteal,
    pub in_box: BoxFilter,
    pub alive: AliveFilter,
    pub attackable: AttackableFilter,
    pub kind: KindFilter,
    pub protect_priority: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct DoNotOptions {
    pub attack_only_listed: bool,
    pub ignore_summons: bool,
    pub ignore_chests: bool,
    pub ignore_raidbosses: bool,
    pub npcs: Vec<i32>,
    pub pick_only_listed: bool,
    pub ignore_meshless: bool,
    pub ignore_unknown_items: bool,
    pub items: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct AlertOptions {
    pub one_way_war: bool,
    pub two_way_war: bool,
    pub minus_one_way_war: bool,
    pub player: bool,
    pub player_names: String,
    pub player_ignore_party: bool,
    pub clan: bool,
    pub clan_names: String,
    pub clan_ignore_party: bool,
    pub hp_below: Option<u8>,
    pub mp_below: Option<u8>,
    pub cp_below: Option<u8>,
    pub private_message: bool,
    pub white_chat: bool,
    pub friend_chat: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct LogoutOptions {
    pub one_way_war: bool,
    pub two_way_war: bool,
    pub minus_one_way_war: bool,
    pub player: bool,
    pub player_names: String,
    pub clan: bool,
    pub clan_names: String,
    pub hp_below: Option<u8>,
    pub mp_below: Option<u8>,
    pub cp_below: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct PlayerSortOptions {
    pub max_z_difference: i32,
    pub max_distance: i32,
    pub equidistant_threshold: i32,
    pub priorities: Vec<ClassPriority>,
}

impl Default for PlayerSortOptions {
    fn default() -> Self {
        Self { max_z_difference: 500, max_distance: 1000, equidistant_threshold: 50, priorities: Vec::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ClassPriority {
    pub class_id: i32,
    pub priority: i32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct AutoShots {
    pub item_id: i32,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct BotOptions {
    pub party: PartyOptions,
    pub follow: FollowOptions,
    pub pickup: PickupOptions,
    pub no_mobs: NoMobsOptions,
    pub pet: PetOptions,
    pub dead: DeadOptions,
    pub attack: AttackOptions,
    pub spoil: SpoilOptions,
    pub advanced: AdvancedOptions,
    pub rest: RestOptions,
    pub targeting: TargetingOptions,
    pub buffs_heals: Vec<BuffHeal>,
    pub toggles: Vec<Toggle>,
    pub auto_shots: AutoShots,
    pub items: Vec<ItemRule>,
    pub combat: Vec<CombatRule>,
    pub polygon: Polygon,
    pub do_not: DoNotOptions,
    pub alerts: AlertOptions,
    pub logout: LogoutOptions,
    pub player_sorting: PlayerSortOptions,
}

pub fn names_list(text: &str) -> Vec<String> {
    text.split([',', ';', '\n']).map(str::trim).filter(|name| !name.is_empty()).map(str::to_owned).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_original_option_screen() {
        let options = BotOptions::default();
        assert_eq!(options.party.buff_range, 550.0);
        assert_eq!(options.follow.distance, 150.0);
        assert_eq!(options.pickup.range, 250.0);
        assert_eq!(options.no_mobs.leash, 100.0);
        assert_eq!(options.attack.move_range, 150.0);
        assert_eq!(options.spoil.mp_above, 50);
        assert_eq!((options.dead.return_delay_seconds, options.dead.logout_delay_seconds), (10, 30));
        assert_eq!(options.polygon.z_range, 1000);
        let buff = BuffHeal::default();
        assert_eq!((buff.below_percent, buff.delay_seconds, buff.mp_above), (60, 1, 100));
        assert_eq!(ItemRule::default().delay_ms, 100);
    }

    #[test]
    fn options_round_trip_through_toml_with_every_table() {
        let mut options = BotOptions::default();
        options.buffs_heals.push(BuffHeal { skill_id: 1204, trait_: Trait::Hp, names: "Tank, Healer".into(), ..Default::default() });
        options.toggles.push(Toggle { skill_id: 1035, trait_: ToggleTrait::Mp, ..Default::default() });
        options.items.push(ItemRule { item_id: 1060, trait_: Trait::Hp, ..Default::default() });
        options.combat.push(CombatRule { page: 2, slot: 5, trait_: Trait::Mp, comparison: Comparison::AtMost, ..Default::default() });
        options.polygon.points = Polygon::regular((100, 100), 4, 500.0, 45.0);
        options.do_not.npcs = vec![20001, 20002];
        options.player_sorting.priorities.push(ClassPriority { class_id: 88, priority: 2 });
        let text = toml::to_string_pretty(&options).unwrap();
        let back: BotOptions = toml::from_str(&text).unwrap();
        assert_eq!(back, options);
        assert!(toml::from_str::<BotOptions>("campo_inexistente = 1").is_err());
    }

    #[test]
    fn traits_have_the_original_order_and_labels() {
        assert_eq!(Trait::BUFF_HEAL.len(), 20);
        assert_eq!(Trait::from_index(0).label(), "Always");
        assert_eq!(Trait::from_index(2).label(), "HP");
        assert_eq!(Trait::from_index(19).label(), "Ultimate Defense");
        assert_eq!(Trait::from_index(99), Trait::Always);
        for (index, item) in Trait::BUFF_HEAL.iter().enumerate() {
            assert_eq!(item.index(), index);
        }
    }

    #[test]
    fn polygon_containment_respects_z_range_and_open_polygons() {
        let square = Polygon { points: Polygon::regular((0, 0), 4, 100.0, 45.0), z_range: 50 };
        assert!(square.contains(0.0, 0.0, 10.0, 0.0));
        assert!(!square.contains(500.0, 0.0, 0.0, 0.0));
        assert!(!square.contains(0.0, 0.0, 200.0, 0.0));
        assert!(Polygon::default().contains(99999.0, 99999.0, 0.0, 0.0));
    }

    #[test]
    fn name_lists_split_on_commas_semicolons_and_lines() {
        assert_eq!(names_list(" Tank, Healer;Mage\nSupport ,, "), vec!["Tank", "Healer", "Mage", "Support"]);
        assert!(names_list("  ").is_empty());
    }

    #[test]
    fn loot_types_map_to_the_party_distribution_codes() {
        assert_eq!(LootType::FindersKeepers.distribution(), 0);
        assert_eq!(LootType::ByTurnIncludingSpoil.distribution(), 4);
    }
}
