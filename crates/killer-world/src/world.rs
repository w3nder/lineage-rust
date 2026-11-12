use std::collections::{HashMap, HashSet};
use killer_platform::time::Instant;

use glam::Vec3;
use l2_net::game::packets::{
    ATTRIBUTE_CURRENT_CP, ATTRIBUTE_CURRENT_HP, ATTRIBUTE_CURRENT_MP, ATTRIBUTE_EXP, ATTRIBUTE_LEVEL, ATTRIBUTE_MAX_CP,
    ATTRIBUTE_MAX_HP, ATTRIBUTE_MAX_MP, PAPERDOLL_SLOTS, VISIBLE_EQUIP_SLOTS,
};
use l2_net::game::{
    ActionFailed, AskJoinAlliance, AskJoinParty, AskJoinPledge, Attack, AutoAttackStart, AutoAttackStop, AutoSoulShot, Buff,
    AskJoinMpcc, CropInfo, CropSettingInfo, EnchantSkillInfo, EnchantSkillList, EventMatchCreate, EventMatchManage, EventMatchMessage, EventMatchObserver, EventMatchScore, EventMatchTeamInfo, EventMatchUserInfo, FishingEnd, FishingHpRegen, FishingStartCombat, HeroList, ManorDefaultInfo, ManorList, MultiPartyChannelInfo, OlympiadMode, OlympiadSpelledInfo, OlympiadUserInfo, PledgeCrestLarge, ProcureCropDetail, RegenMax, SeedInfo, SeedSetting, SellCropList, ServerPrimitive, ShowQuestMark,
    ChairSit, ChangeMoveType, ChangeWaitType, CharInfo, ClientAction, DeleteObject, Dice, Die, DoorInfo, DoorStatusUpdate, DropItem,
    Earthquake, EquipUpdate, EtcStatusUpdate, FinishRotating, FlyToLocation, GetItem, InventoryUpdate, ItemList, Macro,
    MagicEffectIcons, MagicSkillCanceled, MagicSkillLaunched, MagicSkillUse, MoveToLocation, MoveToLocationInVehicle,
    MoveToPawn, MyTargetSelected, NicknameChanged, NpcHtmlMessage, NpcInfo, NpcSay, PaperdollSlot, PartyMember,
    PartyMemberPosition, PartySmallWindowAdd, PartySmallWindowAll, PartySmallWindowDelete, PartySmallWindowDeleteAll,
    PartySpelled, PetDelete, PetInfo, PetStatusShow, PetStatusUpdate, PlaySound, PledgeStatusChanged, RelationChanged,
    Revive, Ride, Say2, SendMacroList, ServerObjectInfo, SetSummonRemainTime, SetupGauge, ShortBuffStatusUpdate,
    ShortCutDelete, ShortCutInit, Shortcut, SkillCoolTime, SkillEntry, SkillList, SkillReuse, SocialAction, SpawnItem,
    SpecialCamera, StartRotating, StaticObject, StatusUpdate, StopMove, StopMoveInVehicle, SunRise, SunSet,
    AbnormalVisualEffectNoop, AllianceIdResult, AllianceNoop, AllianceWar, AllianceWarNoop, AttackFeedbackNoop, BlockListNoop, CameraMode, DeleteRadar, DummyNoop, EventTrigger, FriendAddRequestResult, FriendNoop, GmHide, MagicAndSkillList, MagicListNoop, NetPingResponse, NewCharacterFailNoop, NormalCameraNoop, ObserverEnd, ObserverStart, PartyMemberResult, PetitionVoteNoop, PledgeIdResult, PledgePartyConfirmationNoop, PledgeWarNoop, RequestTimeCheck, ReviveRequestNoop, ServerCloseSocketNoop, SetPledgeCrestNoop, ShowRadar, SkillRemainSec, Snoop, SurrenderPersonallyNoop, SystemMessage, TargetSelected, TargetUnselected, TeleportToLocation, TradePressOk,
    TradeStartOk, UserInfoHead, ValidateLocation,
    ValidateLocationInVehicle, VehicleBoarding, VehicleCheckLocation, VehicleDeparture, VehicleInfo, VehicleStart,
};

pub use l2_net::game::UserSocial;
pub use l2_net::game::UserStats as Stats;

use crate::anim::{AnimEvent, AnimPlayer, Hit, MoveType, PawnView, Tables, Trigger, WaitChange, WaitType};
use crate::clock::{GameClock, SUN_RISE_MINUTES, SUN_SET_MINUTES};
use crate::inventory::Inventory;
use crate::magic::{self, AcceptedCast, CastContext, CastTarget, CastVariant, MagicInfo, MagicState};
use crate::movement;
use crate::session::SessionEvent;
use l2_dat::animnames::AnimSet;
use l2_net::game::{
    AcquireSkillDone, AcquireSkillInfo, AcquireSkillList, BuyListSeed, ChooseInventoryItem, Crest, EnchantResult,
    FriendAddRequest, FriendList, FriendRecvMsg, GmHennaInfo, GmViewItemList, GmViewMagicInfo, GmViewQuestInfo,
    GmViewCharacterInfo, GmViewPledgeInfo, GmViewSkillInfo, GmViewWarehouseWithdrawList, HennaEquipList, HennaInfo, HennaItemInfo,
    JoinParty, JoinPledge, L2Friend, L2FriendStatus,
    ManagePledgePower, MultiSellList, PackageSendableList, PackageToList, PartyMatchDetail, PartyMatchList,
    PetItemList, PledgeExtendedInfo, PledgeInfo, PledgeMember, PledgeShowInfoUpdate, PledgeShowMemberListAll, PledgeShowMemberListDelete,
    PledgeShowMemberListDeleteAll, PrivateStoreListBuy, PrivateStoreListSell, PrivateStoreManageListBuy,
    PrivateStoreManageListSell, PrivateStoreMsg, QuestList, RadarControl, SellListProcure, ShopList, ShopPreviewInfo,
    ShopPreviewList,
    ShowBoard, ShowMinimap, ShowTownMap, TradeAdd, TradeDone, TradeRequest, TradeStart, TradeUpdate, TutorialCloseHtml,
    TutorialEnableClientEvent, TutorialShowHtml, TutorialShowQuestionMark, WarehouseDone, WarehouseList,
};

use l2_net::game::{
    AgitDecoInfo, CastleSiegeInfo, MonRaceInfo, RecipeBookItemList, RecipeItemMakeInfo, RecipeShopItemInfo,
    RecipeShopManageList, RecipeShopMsg, RecipeShopSellList, ShowCalculator, ShowXMasSeal, SiegeClanList, SsqInfo,
    SsqStatus,
};

use crate::net::NetCommand;
use crate::state::{
    CameraCommand, Combat, Dialog, Dialogs, Door, Effects, Friends, Gauge, Henna, Party, Pet, Pledge, PrivateStores,
    PledgeRecord, Rare, Recipes, Siege, StaticTarget, Trade, Tutorial, Vehicle, Windows,
};

#[derive(Debug, Clone, PartialEq)]
pub enum EntityKind {
    Player {
        class_id: i32,
        race: i32,
        sex: i32,
        equipped: [i32; VISIBLE_EQUIP_SLOTS],
        hair_style: i32,
        hair_color: i32,
        face: i32,
        enchant_effect: u8,
        hero: bool,
    },
    Npc {
        npc_id: i32,
        rhand: i32,
        chest: i32,
        lhand: i32,
        attackable: bool,
    },
    Item {
        item_id: i32,
        count: i32,
        stackable: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mount {
    pub ride_type: i32,
    pub ride_class_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rotation {
    pub degree: i32,
    pub side: i32,
    pub speed: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Casting {
    pub skill_id: i32,
    pub skill_level: i32,
    pub hit_time: i32,
    pub target_id: i32,
    pub started: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemOrigin {
    Spawn,
    Drop { source_id: i32, source_position: Option<Vec3> },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Follow {
    pub target_id: i32,
    pub distance: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub item_origin: Option<ItemOrigin>,
    pub abnormal_effect: i32,
    pub object_id: i32,
    pub kind: EntityKind,
    pub name: String,
    pub title: String,
    pub position: Vec3,
    pub heading: i32,
    pub dest: Option<Vec3>,
    pub stop_heading: Option<i32>,
    pub desired_heading: i32,
    pub turn_rate: Option<i32>,
    pub run_speed: i32,
    pub walk_speed: i32,
    pub speed_multiplier: f64,
    pub attack_speed_multiplier: f64,
    pub m_atk_speed: i32,
    pub skill_aim: [i32; 3],
    pub p_atk_speed: i32,
    pub speed_bonus: f32,
    pub speed_bonus_left: f32,
    pub blocked: bool,
    pub follow: Option<Follow>,
    pub running: bool,
    pub sitting: bool,
    pub fake_dead: bool,
    pub dead: bool,
    pub walk_anim_frame_in_move: i32,
    pub hp: Option<(i32, i32)>,
    pub mp: Option<(i32, i32)>,
    pub cp: Option<(i32, i32)>,
    pub level: i32,
    pub exp: i32,
    pub collision_radius: f32,
    pub collision_height: f32,
    pub base_collision: [f32; 2],
    pub karma: i32,
    pub pvp_flag: i32,
    pub relation: i32,
    pub auto_attack: bool,
    pub surrender_war_id: i32,
    pub target: Option<i32>,
    pub casting: Option<Casting>,
    pub magic: MagicState,
    pub anim: AnimPlayer,
    pub spawned: u64,
    pub name_color: i32,
    pub recommendations: i32,
    pub name_blink: bool,
    pub name_blink_elapsed: f32,
    pub clan_id: i32,
    pub clan_crest_id: i32,
    pub ally_id: i32,
    pub ally_crest_id: i32,
    pub store_type: u8,
    pub in_combat: bool,
    pub fishing: u8,
    pub mount: Option<Mount>,
    pub vehicle: Option<i32>,
    pub chair: Option<i32>,
    pub rotation: Option<Rotation>,
}

pub type Player = Entity;

pub const SKILL_SPEED_PER_ATK_SPD: f32 = 1.0 / 333.0;
pub const WHITE_NAME_COLOR: i32 = 0xffffff;
pub const NAME_BLINK_PERIOD: f32 = 0.2;
pub const DEFAULT_WALK_ANIM_FRAME_IN_MOVE: i32 = 12;
const HISTORY_LIMIT: usize = 200;

fn trim_oldest<T>(list: &mut Vec<T>, limit: usize) {
    if list.len() > limit {
        list.drain(..list.len() - limit);
    }
}

impl Entity {
    pub fn assign_turn_rate(&mut self, rates: &killer_assets::pawn_rotation::RotationRates) {
        if self.turn_rate.is_some() {
            return;
        }
        self.turn_rate = match self.kind {
            EntityKind::Player { class_id, race, sex, .. } => l2_dat::chargrp::index_for(race, sex, class_id).and_then(|row| rates.players.get(&row)).copied(),
            EntityKind::Npc { npc_id, .. } => u32::try_from(npc_id).ok().and_then(|id| rates.npcs.get(&id)).copied(),
            EntityKind::Item { .. } => None,
        };
    }

    pub fn assign_walk_anim_frame(&mut self, frames: &std::collections::HashMap<usize, i32>) {
        if let EntityKind::Player { class_id, race, sex, .. } = self.kind
            && let Some(frame) = l2_dat::chargrp::index_for(race, sex, class_id).and_then(|row| frames.get(&row))
        {
            self.walk_anim_frame_in_move = *frame;
        }
    }

    pub fn set_heading(&mut self, heading: i32) {
        self.heading = heading;
        self.desired_heading = heading;
    }

    pub fn base_speed(&self) -> f32 {
        (if self.running { self.run_speed } else { self.walk_speed }) as f32
    }

    pub fn speed(&self) -> f32 {
        let multiplier = if self.speed_multiplier > 0.0 { self.speed_multiplier } else { 1.0 };
        ((self.base_speed() as f64 * multiplier) as f32 + self.speed_bonus).max(0.0)
    }

    pub fn wait_type(&self) -> WaitType {
        if self.chair.is_some() {
            WaitType::ChairSit
        } else if self.sitting {
            WaitType::Sit
        } else if self.fake_dead {
            WaitType::FakeDead
        } else {
            WaitType::Stand
        }
    }

    pub fn tick_anim(&mut self, dt: f32, weapon_type: usize, set: &AnimSet, tables: &dyn Tables) -> Vec<AnimEvent> {
        if let EntityKind::Npc { npc_id, .. } = self.kind {
            self.anim.npc_class_id = npc_id;
        }
        let view = self.pawn_view(weapon_type);
        let moving = self.dest.is_some() && !self.blocked;
        let events = self.anim.tick(dt, &view, set, tables, moving, self.heading as f32);
        if events.iter().any(|e| matches!(e, AnimEvent::CastStart { .. } | AnimEvent::MoveStop)) {
            movement::stop(self);
        }
        if events.iter().any(|e| matches!(e, AnimEvent::CastStart { .. })) {
            self.magic.set_magic_info();
        }
        events
    }

    pub fn pawn_view(&self, weapon_type: usize) -> PawnView {
        PawnView {
            weapon_type,
            npc: matches!(self.kind, EntityKind::Npc { .. }),
            ride_type: self.mount.and_then(|mount| usize::try_from(mount.ride_type).ok()).filter(|ride| *ride > 0),
            wait_type: self.wait_type(),
            move_type: if self.running { MoveType::Fast } else { MoveType::Slow },
            dead: self.dead,
            auto_attacking: self.in_combat,
            non_attack_speed_rate: self.speed_multiplier as f32,
            attack_speed_rate: self.attack_speed_multiplier as f32,
            skill_speed_rate: self.m_atk_speed as f32 * SKILL_SPEED_PER_ATK_SPD,
            walk_anim_frame_in_move: self.walk_anim_frame_in_move,
            npc_speed: self.anim.npc_speed,
            fishing: match self.fishing {
                1 => crate::anim::FishingType::Wait,
                2 => crate::anim::FishingType::Battle,
                _ => crate::anim::FishingType::None,
            },
            ..PawnView::default()
        }
    }

    fn from_user_info(head: &UserInfoHead) -> Entity {
        Entity {
            item_origin: None,
            abnormal_effect: head.abnormal_effect,
            object_id: head.object_id,
            kind: EntityKind::Player {
                class_id: head.class_id,
                race: head.race,
                sex: head.sex,
                equipped: head.equipped(),
                hair_style: head.hair_style,
                hair_color: head.hair_color,
                face: head.face,
                enchant_effect: head.enchant_effect,
                hero: head.social.hero,
            },
            name: head.name.clone(),
            title: head.title.clone(),
            position: Vec3::new(head.x as f32, head.y as f32, head.z as f32),
            heading: head.heading,
            dest: None,
            stop_heading: None,
            desired_heading: head.heading,
            turn_rate: None,
            run_speed: head.run_speed,
            walk_speed: head.walk_speed,
            speed_multiplier: head.speed_multiplier,
            attack_speed_multiplier: head.attack_speed_multiplier,
            m_atk_speed: head.stats.cast_spd,
            skill_aim: [0; 3],
            p_atk_speed: head.stats.atk_spd,
            speed_bonus: 0.0,
            speed_bonus_left: 0.0,
            blocked: false,
            follow: None,
            running: true,
            sitting: false,
            fake_dead: false,
            dead: false,
            walk_anim_frame_in_move: DEFAULT_WALK_ANIM_FRAME_IN_MOVE,
            hp: Some((head.hp, head.max_hp)),
            mp: Some((head.mp, head.max_mp)),
            cp: Some((head.cp, head.max_cp)),
            level: head.level,
            exp: head.exp,
            collision_radius: head.collision_radius as f32,
            collision_height: head.collision_height as f32,
            base_collision: [0.0; 2],
            karma: head.karma,
            pvp_flag: head.pvp_flag,
            relation: head.social.relation, auto_attack: false, surrender_war_id: 0,
            target: None,
            casting: None,
            magic: Default::default(),
            anim: AnimPlayer::default(),
            spawned: 0,
            name_color: head.name_color,
            recommendations: i32::from(head.social.recom_have),
            name_blink: false,
            name_blink_elapsed: 0.0,
            clan_id: head.social.clan_id,
            clan_crest_id: head.social.clan_crest_id,
            ally_id: head.social.ally_id,
            ally_crest_id: head.social.ally_crest_id,
            store_type: 0,
            in_combat: false,
            fishing: 0,
            mount: None,
            vehicle: None,
            chair: None,
            rotation: None,
        }
    }

    fn from_char_info(c: &CharInfo) -> Entity {
        Entity {
            item_origin: None,
            abnormal_effect: c.abnormal_effect,
            object_id: c.object_id,
            kind: EntityKind::Player {
                class_id: c.class_id,
                race: c.race,
                sex: c.sex,
                equipped: c.equipped,
                hair_style: c.hair_style,
                hair_color: c.hair_color,
                face: c.face,
                enchant_effect: c.enchant_effect,
                hero: c.hero != 0,
            },
            name: c.name.clone(),
            title: c.title.clone(),
            position: Vec3::new(c.x as f32, c.y as f32, c.z as f32),
            heading: c.heading,
            dest: None,
            stop_heading: None,
            desired_heading: c.heading,
            turn_rate: None,
            run_speed: c.run_speed,
            walk_speed: c.walk_speed,
            speed_multiplier: c.speed_multiplier,
            attack_speed_multiplier: c.attack_speed_multiplier,
            m_atk_speed: c.m_atk_speed,
            skill_aim: [0; 3],
            p_atk_speed: c.p_atk_speed,
            speed_bonus: 0.0,
            speed_bonus_left: 0.0,
            blocked: false,
            follow: None,
            running: c.running,
            sitting: c.sitting,
            fake_dead: false,
            dead: c.alike_dead,
            walk_anim_frame_in_move: DEFAULT_WALK_ANIM_FRAME_IN_MOVE,
            hp: None,
            mp: None,
            cp: None,
            level: 0,
            exp: 0,
            collision_radius: c.collision_radius as f32,
            collision_height: c.collision_height as f32,
            base_collision: [0.0; 2],
            karma: c.karma,
            pvp_flag: c.pvp_flag,
            relation: 0, auto_attack: false, surrender_war_id: 0,
            target: None,
            casting: None,
            magic: Default::default(),
            anim: AnimPlayer::default(),
            spawned: 0,
            name_color: c.name_color,
            recommendations: c.recommendations_received,
            name_blink: false,
            name_blink_elapsed: 0.0,
            clan_id: c.clan_id,
            clan_crest_id: c.clan_crest_id,
            ally_id: c.ally_id,
            ally_crest_id: c.ally_crest_id,
            store_type: c.store_type,
            fishing: u8::from(c.fishing != 0),
            in_combat: false,
            mount: matches!(c.mount_type, 1 | 2).then_some(Mount { ride_type: i32::from(c.mount_type), ride_class_id: c.mount_npc_id }),
            vehicle: None,
            chair: None,
            rotation: None,
        }
    }

    fn from_npc_info(n: &NpcInfo) -> Entity {
        Entity {
            item_origin: None,
            abnormal_effect: n.abnormal_effect,
            object_id: n.object_id,
            kind: EntityKind::Npc {
                npc_id: n.npc_id,
                rhand: n.rhand,
                chest: n.chest,
                lhand: n.lhand,
                attackable: n.attackable,
            },
            name: n.name.clone(),
            title: n.title.clone(),
            position: Vec3::new(n.x as f32, n.y as f32, n.z as f32),
            heading: n.heading,
            dest: None,
            stop_heading: None,
            desired_heading: n.heading,
            turn_rate: None,
            run_speed: n.run_speed,
            walk_speed: n.walk_speed,
            speed_multiplier: n.speed_multiplier,
            attack_speed_multiplier: n.attack_speed_multiplier,
            m_atk_speed: n.m_atk_speed,
            skill_aim: [0; 3],
            p_atk_speed: n.p_atk_speed,
            speed_bonus: 0.0,
            speed_bonus_left: 0.0,
            blocked: false,
            follow: None,
            running: n.running,
            sitting: false,
            fake_dead: false,
            dead: n.alike_dead,
            walk_anim_frame_in_move: DEFAULT_WALK_ANIM_FRAME_IN_MOVE,
            hp: None,
            mp: None,
            cp: None,
            level: 0,
            exp: 0,
            collision_radius: n.collision_radius as f32,
            collision_height: n.collision_height as f32,
            base_collision: [n.base_collision_radius as f32, n.base_collision_height as f32],
            karma: n.karma,
            pvp_flag: n.pvp_flag,
            relation: 0, auto_attack: n.attackable, surrender_war_id: 0,
            target: None,
            casting: None,
            magic: Default::default(),
            anim: AnimPlayer::default(),
            spawned: 0,
            name_color: WHITE_NAME_COLOR,
            recommendations: 0,
            name_blink: false,
            name_blink_elapsed: 0.0,
            clan_id: 0,
            clan_crest_id: 0,
            ally_id: 0,
            ally_crest_id: 0,
            store_type: 0,
            in_combat: false,
            fishing: 0,
            mount: None,
            vehicle: None,
            chair: None,
            rotation: None,
        }
    }

    fn from_spawn_item(s: &SpawnItem) -> Entity {
        let mut entity = Entity::item(s.object_id, s.item_id, s.count, s.stackable, Vec3::new(s.x as f32, s.y as f32, s.z as f32));
        entity.item_origin = Some(ItemOrigin::Spawn);
        entity.set_heading(s.heading);
        entity
    }

    fn from_drop_item(d: &DropItem, source_position: Option<Vec3>) -> Entity {
        let mut entity = Entity::item(d.object_id, d.item_id, d.count, d.stackable, Vec3::new(d.x as f32, d.y as f32, d.z as f32));
        entity.item_origin = Some(ItemOrigin::Drop { source_id:d.player_id,source_position });
        entity.set_heading(d.heading);
        entity
    }

    fn from_pet_info(p: &PetInfo) -> Entity {
        let mut entity = Entity::item(p.object_id, 0, 0, false, Vec3::new(p.x as f32, p.y as f32, p.z as f32));
        entity.kind = EntityKind::Npc {
            npc_id: p.npc_id,
            rhand: p.rhand,
            chest: p.chest,
            lhand: p.lhand,
            attackable: p.attackable,
        };
        entity.auto_attack = p.attackable;
        entity.name = p.name.clone();
        entity.title = p.title.clone();
        entity.set_heading(p.heading);
        entity.run_speed = p.run_speed;
        entity.walk_speed = p.walk_speed;
        entity.speed_multiplier = p.speed_multiplier;
        entity.attack_speed_multiplier = p.attack_speed_multiplier;
        entity.m_atk_speed = p.m_atk_speed;
        entity.p_atk_speed = p.p_atk_speed;
        entity.running = p.running;
        entity.dead = p.alike_dead;
        entity.in_combat = p.in_combat;
        entity.hp = Some((p.hp, p.max_hp));
        entity.mp = Some((p.mp, p.max_mp));
        entity.level = p.level;
        entity.exp = p.exp;
        entity.collision_radius = p.collision_radius as f32;
        entity.collision_height = p.collision_height as f32;
        entity.karma = p.karma;
        entity.pvp_flag = p.pvp_flag;
        entity
    }

    fn from_server_object_info(o: &ServerObjectInfo) -> Entity {
        let mut entity = Entity::item(o.object_id, 0, 0, false, Vec3::new(o.x as f32, o.y as f32, o.z as f32));
        entity.kind = EntityKind::Npc { npc_id: o.npc_id, rhand: 0, chest: 0, lhand: 0, attackable: o.attackable };
        entity.auto_attack = o.attackable;
        entity.name = o.name.clone();
        entity.set_heading(o.heading);
        entity.speed_multiplier = o.speed_multiplier;
        entity.hp = Some((o.hp, o.max_hp));
        entity.collision_radius = o.collision_radius as f32;
        entity.collision_height = o.collision_height as f32;
        entity
    }

    pub(crate) fn item(object_id: i32, item_id: i32, count: i32, stackable: bool, position: Vec3) -> Entity {
        Entity {
            item_origin: None,
            abnormal_effect: 0,
            object_id,
            kind: EntityKind::Item { item_id, count, stackable },
            name: String::new(),
            title: String::new(),
            position,
            heading: 0,
            dest: None,
            stop_heading: None,
            desired_heading: 0,
            turn_rate: None,
            run_speed: 0,
            walk_speed: 0,
            speed_multiplier: 1.0,
            attack_speed_multiplier: 1.0,
            m_atk_speed: 0,
            skill_aim: [0; 3],
            p_atk_speed: 0,
            speed_bonus: 0.0,
            speed_bonus_left: 0.0,
            blocked: false,
            follow: None,
            running: false,
            sitting: false,
            fake_dead: false,
            dead: false,
            walk_anim_frame_in_move: DEFAULT_WALK_ANIM_FRAME_IN_MOVE,
            hp: None,
            mp: None,
            cp: None,
            level: 0,
            exp: 0,
            collision_radius: 0.0,
            collision_height: 0.0,
            base_collision: [0.0; 2],
            karma: 0,
            pvp_flag: 0,
            relation: 0, auto_attack: false, surrender_war_id: 0,
            target: None,
            casting: None,
            magic: Default::default(),
            anim: AnimPlayer::default(),
            spawned: 0,
            name_color: WHITE_NAME_COLOR,
            recommendations: 0,
            name_blink: false,
            name_blink_elapsed: 0.0,
            clan_id: 0,
            clan_crest_id: 0,
            ally_id: 0,
            ally_crest_id: 0,
            store_type: 0,
            in_combat: false,
            fishing: 0,
            mount: None,
            vehicle: None,
            chair: None,
            rotation: None,
        }
    }

    fn refresh(&mut self, fresh: Entity) {
        self.abnormal_effect = fresh.abnormal_effect;
        self.run_speed = fresh.run_speed;
        self.walk_speed = fresh.walk_speed;
        self.speed_multiplier = fresh.speed_multiplier;
        self.attack_speed_multiplier = fresh.attack_speed_multiplier;
        self.m_atk_speed = fresh.m_atk_speed;
        self.p_atk_speed = fresh.p_atk_speed;
        self.hp = fresh.hp;
        self.mp = fresh.mp;
        self.cp = fresh.cp;
        self.level = fresh.level;
        self.exp = fresh.exp;
        self.name = fresh.name;
        self.kind = fresh.kind;
        self.collision_radius = fresh.collision_radius;
        self.collision_height = fresh.collision_height;
        self.base_collision = fresh.base_collision;
        self.karma = fresh.karma;
        self.pvp_flag = fresh.pvp_flag;
        self.name_color = fresh.name_color;
        self.recommendations = fresh.recommendations;
        self.clan_id = fresh.clan_id;
        self.clan_crest_id = fresh.clan_crest_id;
        self.ally_id = fresh.ally_id;
        self.ally_crest_id = fresh.ally_crest_id;
    }

    pub fn tick_name_blink(&mut self, dt: f32) {
        self.name_blink_elapsed += dt;
        if self.name_blink_elapsed > NAME_BLINK_PERIOD {
            self.name_blink = !self.name_blink;
            self.name_blink_elapsed = 0.0;
        }
    }

    fn set_hp(&mut self, current: Option<i32>, max: Option<i32>) {
        let (c, m) = self.hp.unwrap_or((0, 0));
        self.hp = Some((current.unwrap_or(c), max.unwrap_or(m)));
    }

    fn set_mp(&mut self, current: Option<i32>, max: Option<i32>) {
        let (c, m) = self.mp.unwrap_or((0, 0));
        self.mp = Some((current.unwrap_or(c), max.unwrap_or(m)));
    }

    fn set_cp(&mut self, current: Option<i32>, max: Option<i32>) {
        let (c, m) = self.cp.unwrap_or((0, 0));
        self.cp = Some((current.unwrap_or(c), max.unwrap_or(m)));
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorldEvent {
    EventMatchGmTest,
    ClosePartyRoom,
    FishingStart(l2_net::game::FishingStart),
    OpenMpcc,
    CloseMpcc,
    OlympiadMatchEnd,
    MailArrived,
    ColosseumFenceInfo(l2_net::game::ColosseumFenceInfo),
    EventMatchSpelledInfo(l2_net::game::EventMatchSpelledInfo),
    EventMatchFirecracker(l2_net::game::EventMatchFirecracker),
    EventMatchTeamUnlocked(l2_net::game::EventMatchTeamUnlocked),
    PartyRoomMember(l2_net::game::PartyRoomMember),
    ManagePartyRoomMember(l2_net::game::ManagePartyRoomMember),
    StorageMaxCount(l2_net::game::StorageMaxCount),
    AutoSoulShot(AutoSoulShot),
    FishingEnd(FishingEnd),
    FishingStartCombat(FishingStartCombat),
    FishingHpRegen(FishingHpRegen),
    EnchantSkillList(EnchantSkillList),
    EnchantSkillInfo(EnchantSkillInfo),
    ShowQuestInfo,
    ShowQuestMark(ShowQuestMark),
    SendManorList(ManorList),
    ShowSeedInfo(SeedInfo),
    ShowCropInfo(CropInfo),
    ShowManorDefaultInfo(ManorDefaultInfo),
    ShowSeedSetting(SeedSetting),
    ShowCropSetting(CropSettingInfo),
    ShowSellCropList(SellCropList),
    ShowProcureCropDetail(ProcureCropDetail),
    HeroList(HeroList),
    ServerPrimitive(ServerPrimitive),
    AskJoinMpcc(AskJoinMpcc),
    PledgeCrestLarge(PledgeCrestLarge),
    OlympiadUserInfo(OlympiadUserInfo),
    OlympiadSpelledInfo(OlympiadSpelledInfo),
    OlympiadMode(OlympiadMode),
    EventMatchManage(EventMatchManage),
    MultiPartyChannelInfo(MultiPartyChannelInfo),
    RegenMax(RegenMax),
    EventMatchObserver(EventMatchObserver),
    EventMatchMessage(EventMatchMessage),
    EventMatchScore(EventMatchScore),
    EventMatchTeamInfo(EventMatchTeamInfo),
    EventMatchCreate(EventMatchCreate),
    EventMatchUserInfo(EventMatchUserInfo),
    CharInfo(Box<CharInfo>),
    NpcInfo(NpcInfo),
    MoveToLocation(MoveToLocation),
    StopMove(StopMove),
    ValidateLocation(ValidateLocation),
    TeleportToLocation(TeleportToLocation),
    DeleteObject(DeleteObject),
    StatusUpdate(StatusUpdate),
    ChangeMoveType(ChangeMoveType),
    ChangeWaitType(ChangeWaitType),
    Die(Die),
    Revive(Revive),
    SocialAction(SocialAction),
    Say2(Say2),
    NpcSay(NpcSay),
    SystemMessage(SystemMessage),
    TargetSelected(TargetSelected),
    TargetUnselected(TargetUnselected),
    MoveToPawn(MoveToPawn),
    RelationChanged(RelationChanged),
    MagicSkillUse(MagicSkillUse),
    MagicSkillLaunched(MagicSkillLaunched),
    MagicSkillCanceled(MagicSkillCanceled),
    SpawnItem(SpawnItem),
    EtcStatusUpdate(EtcStatusUpdate),
    MagicEffectIcons(MagicEffectIcons),
    PledgeStatusChanged(PledgeStatusChanged),
    SendMacroList(SendMacroList),
    ItemList(ItemList),
    InventoryUpdate(InventoryUpdate),
    EquipUpdate(EquipUpdate),
    ShortCutInit(ShortCutInit),
    ShortCutRegister(Shortcut),
    ShortCutDelete(ShortCutDelete),
    SkillList(SkillList),
    NpcHtmlMessage(NpcHtmlMessage),
    MyTargetSelected(MyTargetSelected),
    SkillCoolTime(SkillCoolTime),
    ActionFailed(ActionFailed),
    Attack(Attack),
    DropItem(DropItem),
    GetItem(GetItem),
    SunRise(SunRise),
    SunSet(SunSet),
    AutoAttackStart(AutoAttackStart),
    AutoAttackStop(AutoAttackStop),
    AskJoinPledge(AskJoinPledge),
    AskJoinParty(AskJoinParty),
    DoorInfo(DoorInfo),
    DoorStatusUpdate(DoorStatusUpdate),
    PartySmallWindowAll(PartySmallWindowAll),
    PartySmallWindowAdd(PartySmallWindowAdd),
    PartySmallWindowDeleteAll(PartySmallWindowDeleteAll),
    PartySmallWindowDelete(PartySmallWindowDelete),
    PartySmallWindowUpdate(PartyMember),
    VehicleInfo(VehicleInfo),
    VehicleDeparture(VehicleDeparture),
    GetOnVehicle(VehicleBoarding),
    GetOffVehicle(VehicleBoarding),
    StartRotating(StartRotating),
    FinishRotating(FinishRotating),
    SetupGauge(SetupGauge),
    MoveToLocationInVehicle(MoveToLocationInVehicle),
    StopMoveInVehicle(StopMoveInVehicle),
    Ride(Ride),
    ServerObjectInfo(ServerObjectInfo),
    PlaySound(PlaySound),
    StaticObject(StaticObject),
    PartyMemberPosition(PartyMemberPosition),
    AskJoinAlliance(AskJoinAlliance),
    PetStatusShow(PetStatusShow),
    PetInfo(Box<PetInfo>),
    PetStatusUpdate(PetStatusUpdate),
    PetDelete(PetDelete),
    VehicleStart(VehicleStart),
    Earthquake(Earthquake),
    FlyToLocation(FlyToLocation),
    SpecialCamera(SpecialCamera),
    NicknameChanged(NicknameChanged),
    SetSummonRemainTime(SetSummonRemainTime),
    Dice(Dice),
    ChairSit(ChairSit),
    ConfirmDlg(SystemMessage),
    PartySpelled(PartySpelled),
    ShortBuffStatusUpdate(ShortBuffStatusUpdate),
    SellList(ShopList),
    BuyList(ShopList),
    TradeStart(TradeStart),
    TradeOwnAdd(TradeAdd),
    TradeOtherAdd(TradeAdd),
    TradeDone(TradeDone),
    TradeStartOk(TradeStartOk),
    TradePressOwnOk(TradePressOk),
    TradePressOtherOk(TradePressOk),
    ShowRadar(ShowRadar),
    DeleteRadar(DeleteRadar),
    ManagePledgePower(ManagePledgePower),
    JoinPledge(JoinPledge),
    JoinParty(JoinParty),
    WarehouseDepositList(WarehouseList),
    WarehouseWithdrawList(WarehouseList),
    WarehouseDone(WarehouseDone),
    PledgeShowMemberListAll(PledgeShowMemberListAll),
    PledgeShowMemberListUpdate(PledgeMember),
    PledgeShowMemberListAdd(PledgeMember),
    PledgeShowMemberListDelete(PledgeShowMemberListDelete),
    TradeRequest(TradeRequest),
    PledgeCrest(Crest),
    ShowBoard(Option<ShowBoard>),
    ChooseInventoryItem(ChooseInventoryItem),
    ClientAction(ClientAction),
    TradeUpdate(TradeUpdate),
    FriendAddRequest(FriendAddRequest),
    FriendAddRequestResult(FriendAddRequestResult),
    NewCharacterFailNoop(NewCharacterFailNoop),
    ReviveRequestNoop(ReviveRequestNoop),
    FriendAdd(FriendNoop),
    FriendRemove(FriendNoop),
    FriendListNoop(FriendNoop),
    FriendStatusNoop(FriendNoop),
    MagicListNoop(MagicListNoop),
    QuestList(QuestList),
    EnchantResult(EnchantResult),
    PledgeShowMemberListDeleteAll(PledgeShowMemberListDeleteAll),
    PledgeInfo(PledgeInfo),
    PledgeExtendedInfo(PledgeExtendedInfo),
    StartPledgeWarNoop(PledgeWarNoop),
    ReplyStartPledgeWarNoop(PledgeWarNoop),
    StopPledgeWarNoop(PledgeWarNoop),
    ReplyStopPledgeWarNoop(PledgeWarNoop),
    SurrenderPledgeWarNoop(PledgeWarNoop),
    ReplySurrenderPledgeWarNoop(PledgeWarNoop),
    SetPledgeCrestNoop(SetPledgeCrestNoop),
    SurrenderPersonallyNoop(SurrenderPersonallyNoop),
    ServerCloseSocketNoop(ServerCloseSocketNoop),
    ReplyStartAllianceWarNoop(AllianceWarNoop),
    ReplyStopAllianceWarNoop(AllianceWarNoop),
    SurrenderAllianceWarNoop(AllianceWarNoop),
    StartAllianceWar(AllianceWar),
    StopAllianceWar(AllianceWar),
    MagicAndSkillList(MagicAndSkillList),
    SkillRemainSec(SkillRemainSec),
    VehicleCheckLocation(VehicleCheckLocation),
    DummyNoop(DummyNoop),
    BlockListNoop(BlockListNoop),
    AbnormalVisualEffectNoop(AbnormalVisualEffectNoop),
    CreatePledge(PledgeIdResult),
    WithdrawalPledge(PledgeIdResult),
    OustPledgeMember(PledgeIdResult),
    DismissPledge(PledgeIdResult),
    WithdrawalParty(PartyMemberResult),
    OustPartyMember(PartyMemberResult),
    DismissParty(PartyMemberResult),
    SetOustPledgeMemberNoop(PledgePartyConfirmationNoop),
    SetDismissPledgeNoop(PledgePartyConfirmationNoop),
    SetOustPartyMemberNoop(PledgePartyConfirmationNoop),
    SetDismissPartyNoop(PledgePartyConfirmationNoop),
    AttackOutOfRangeNoop(AttackFeedbackNoop),
    AttackInCoolTimeNoop(AttackFeedbackNoop),
    AttackDeadTargetNoop(AttackFeedbackNoop),
    ValidateLocationInVehicle(ValidateLocationInVehicle),
    GmHide(GmHide),
    JoinAllianceResult(AllianceIdResult),
    WithdrawAllianceResult(AllianceIdResult),
    OustAllianceMemberPledgeResult(AllianceIdResult),
    DismissAllianceResult(AllianceIdResult),
    SetAllianceCrestNoop(AllianceNoop),
    AllianceInfoNoop(AllianceNoop),
    RequestTimeCheck(RequestTimeCheck),
    ObserverStart(ObserverStart),
    ObserverEnd(ObserverEnd),
    NormalCameraNoop(NormalCameraNoop),
    EventTrigger(EventTrigger),
    NetPingResponse(NetPingResponse),
    Snoop(Snoop),
    CameraMode(CameraMode),
    PetitionVoteNoop(PetitionVoteNoop),
    PledgeShowInfoUpdate(PledgeShowInfoUpdate),
    AcquireSkillList(AcquireSkillList),
    AcquireSkillInfo(AcquireSkillInfo),
    AcquireSkillDone(AcquireSkillDone),
    PartyMatchList(PartyMatchList),
    PartyMatchDetail(PartyMatchDetail),
    PrivateStoreManageListSell(PrivateStoreManageListSell),
    PrivateStoreListSell(PrivateStoreListSell),
    PrivateStoreMsgSell(PrivateStoreMsg),
    ShowMinimap(ShowMinimap),
    TutorialShowHtml(TutorialShowHtml),
    TutorialShowQuestionMark(TutorialShowQuestionMark),
    TutorialEnableClientEvent(TutorialEnableClientEvent),
    TutorialCloseHtml(TutorialCloseHtml),
    AllianceCrest(Crest),
    PetItemList(PetItemList),
    PetInventoryUpdate(InventoryUpdate),
    PrivateStoreManageListBuy(PrivateStoreManageListBuy),
    PrivateStoreListBuy(PrivateStoreListBuy),
    PrivateStoreMsgBuy(PrivateStoreMsg),
    PackageToList(PackageToList),
    PackageSendableList(PackageSendableList),
    MultiSellList(MultiSellList),
    ShowTownMap(ShowTownMap),
    HennaEquipList(HennaEquipList),
    HennaItemInfo(HennaItemInfo),
    HennaInfo(HennaInfo),
    HennaUnequipList(HennaEquipList),
    HennaUnequipInfo(HennaItemInfo),
    GmViewSkillInfo(GmViewSkillInfo),
    GmViewMagicInfo(GmViewMagicInfo),
    GmViewQuestInfo(GmViewQuestInfo),
    GmViewItemList(GmViewItemList),
    GmViewWarehouseWithdrawList(GmViewWarehouseWithdrawList),
    SellListProcure(SellListProcure),
    GmHennaInfo(GmHennaInfo),
    L2Friend(L2Friend),
    L2FriendStatus(L2FriendStatus),
    GmViewPledgeInfo(GmViewPledgeInfo),
    GmViewCharacterInfo(Box<GmViewCharacterInfo>),
    BuyListSeed(BuyListSeed),
    RadarControl(RadarControl),
    ShopPreviewList(ShopPreviewList),
    ShopPreviewInfo(ShopPreviewInfo),
    FriendList(FriendList),
    FriendRecvMsg(FriendRecvMsg),
    CastleSiegeInfo(CastleSiegeInfo),
    CastleSiegeAttackerList(SiegeClanList),
    CastleSiegeDefenderList(SiegeClanList),
    RecipeBookItemList(RecipeBookItemList),
    RecipeItemMakeInfo(RecipeItemMakeInfo),
    RecipeShopManageList(RecipeShopManageList),
    RecipeShopSellList(RecipeShopSellList),
    RecipeShopItemInfo(RecipeShopItemInfo),
    RecipeShopMsg(RecipeShopMsg),
    ShowCalculator(ShowCalculator),
    MonRaceInfo(MonRaceInfo),
    ShowXMasSeal(ShowXMasSeal),
    SsqStatus(SsqStatus),
    AgitDecoInfo(AgitDecoInfo),
    SsqInfo(SsqInfo),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RequestedPledges(HashMap<i32, i32>);

impl RequestedPledges {
    pub fn find_requested_pledge(&self, pledge_id: i32) -> i32 {
        self.0.get(&pledge_id).copied().unwrap_or(0)
    }

    pub fn note_pledge_requested(&mut self, pledge_id: i32, value: i32) {
        self.0.insert(pledge_id, value);
    }

    pub fn get_requested_pledge_id(&self, pledge_id: i32) -> bool {
        pledge_id > 0 && self.find_requested_pledge(pledge_id) > 0
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MacroList {
    pub revision: i32,
    pub total_count: u8,
    pub macros: Vec<Macro>,
}

impl MacroList {
    /// O servidor manda um macro por pacote e sobe a revisão a cada mudança, então um pacote de
    /// revisão nova costuma trazer só o macro que mudou — apagar a lista inteira nesse caso
    /// perdia todos os outros. Quem diz quantos existem é o `total_count` do próprio pacote: só
    /// quando ele é zero a lista fica vazia. Fora isso o macro é mesclado pelo id.
    ///
    /// Em aberto: o campo `flag` do pacote, que é lido mas ainda não usado — ele deve dizer se o
    /// pacote é uma inclusão, uma alteração ou uma remoção, e é o que falta pra tratar a remoção
    /// de um macro sem depender do `total_count`.
    fn apply(&mut self, list: &SendMacroList) {
        self.revision = list.revision;
        self.total_count = list.total_count;
        if list.total_count == 0 {
            self.macros.clear();
            return;
        }
        for m in &list.macros {
            match self.macros.iter_mut().find(|known| known.id == m.id) {
                Some(known) => *known = m.clone(),
                None => self.macros.push(m.clone()),
            }
        }
    }
}

impl WorldEvent {
    pub fn placed_object(&self) -> Option<i32> {
        match self {
            WorldEvent::CharInfo(c) => Some(c.object_id),
            WorldEvent::NpcInfo(n) => Some(n.object_id),
            WorldEvent::MoveToLocation(m) => Some(m.object_id),
            WorldEvent::StopMove(s) => Some(s.object_id),
            WorldEvent::ValidateLocation(v) => Some(v.object_id),
            WorldEvent::TeleportToLocation(t) => Some(t.target_id),
            WorldEvent::ChangeWaitType(c) => Some(c.object_id),
            WorldEvent::DropItem(d) => Some(d.object_id),
            WorldEvent::PetInfo(p) => Some(p.object_id),
            WorldEvent::ServerObjectInfo(o) => Some(o.object_id),
            WorldEvent::FlyToLocation(f) => Some(f.object_id),
            _ => None,
        }
    }
}

pub fn event_trigger(trigger: &EventTrigger) -> (i32, i32) {
    (trigger.event_id, i32::from(trigger.on as i8))
}

pub const CLIENT_ACTION_RADIUS: f32 = 200.0;
pub const VEHICLE_REPOSITION_HEIGHT: f32 = 100.0;
pub const VEHICLE_SNAP_DISTANCE: f32 = 15.0;

fn vehicle_angle(vehicle: &Vehicle) -> f32 {
    (vehicle.heading & 0xffff) as f32 * std::f32::consts::TAU / 65536.0
}

pub fn vehicle_world(vehicle: &Vehicle, local: Vec3) -> Vec3 {
    let (sin, cos) = vehicle_angle(vehicle).sin_cos();
    Vec3::new(local.x * cos - local.y * sin, local.x * sin + local.y * cos, local.z) + Vec3::new(vehicle.x as f32, vehicle.y as f32, vehicle.z as f32)
}

pub fn vehicle_local(vehicle: &Vehicle, world: Vec3) -> Vec3 {
    let offset = world - Vec3::new(vehicle.x as f32, vehicle.y as f32, vehicle.z as f32);
    let (sin, cos) = (-vehicle_angle(vehicle)).sin_cos();
    Vec3::new(offset.x * cos - offset.y * sin, offset.x * sin + offset.y * cos, offset.z)
}
pub const CLIENT_ACTION_NEAREST_PAWN: i32 = 4;
pub const CLIENT_ACTION_NEAREST_ITEM: i32 = 5;
pub const CLIENT_ACTION_TARGET: i32 = 6;

#[derive(Default)]
pub struct World {
    pub object_cycle: std::cell::Cell<crate::object_scan::ObjectCycle>,
    pub store_matches: std::collections::HashMap<i32, u8>,
    pub teleport_fades: bool,
    pub teleport_announce: bool,
    pub rotation_rates: killer_assets::pawn_rotation::RotationRates,
    pub enter_events: std::collections::HashMap<u32, l2_dat::entereventgrp::EnterEvent>,
    pub rising: std::collections::HashMap<i32, crate::enter_event::Rise>,
    pub walk_anim_frames: std::collections::HashMap<usize, i32>,
    pub speech: crate::speech::Speech,
    pub me: Option<Entity>,
    pub observer: crate::observer::ObserverControl,
    pub appendix: crate::appendix::Appendix,
    pub music_stop: bool,
    pub music_reset: bool,
    pub attack_intend: bool,
    pub entities: rustc_hash::FxHashMap<i32, Entity>,
    pub spawn_sequence: u64,
    pub clock: Option<GameClock>,
    pub target: Option<i32>,
    pub etc_status: Option<EtcStatusUpdate>,
    pub buffs: Vec<Buff>,
    pub buffs_at: Option<Instant>,
    pub pledge: Option<PledgeStatusChanged>,
    pub macros: MacroList,
    pub inventory: Inventory,
    pub shortcuts: Vec<Shortcut>,
    pub skills: Vec<SkillEntry>,
    pub cooldowns: Vec<SkillReuse>,
    pub npc_html: Option<NpcHtmlMessage>,
    pub stats: Stats,
    pub paperdoll: [PaperdollSlot; PAPERDOLL_SLOTS],
    pub equip_entries: std::collections::HashMap<i32, i32>,
    pub inventory_limit: i32,
    pub builder_level: i32,
    pub storage_limits: Option<l2_net::game::StorageMaxCount>,
    pub social: UserSocial,
    pub target_color: Option<i16>,
    pub pending_click_target: Option<i32>,
    pub camera_fixed: bool,
    pub pledges: HashMap<i32, PledgeRecord>,
    pub requested_pledges: RequestedPledges,
    pub crests_requested: HashSet<i32>,
    pub ally_crests_requested: HashSet<i32>,
    pub requests: Vec<NetCommand>,
    pub party: Party,
    pub dialogs: Dialogs,
    pub pending_client_action: Option<i32>,
    pub time_check: Option<(i32, i32)>,
    pub regen_fill: crate::regen::RegenFill,
    pub seconds: f64,
    pub regen_hp: i32,
    pub frame_time: f32,
    pub pawn_clipping: f32,
    pub keep_min_frame_rate: bool,
    pub rendered: HashSet<i32>,
    pub last_cast: Option<AcceptedCast>,
    pub fishing_window: crate::fishing::FishWindow,
    pub fishing_view: crate::fishing_view::FishingView,
    pub combat: Combat,
    pub gauge: Option<Gauge>,
    pub doors: HashMap<i32, Door>,
    pub event_trigger: Option<(i32, i32)>,
    pub vehicles: HashMap<i32, Vehicle>,
    pub pet: Option<Pet>,
    pub pet_window: Option<i32>,
    pub effects: Effects,
    pub daylight: Option<bool>,
    pub static_objects: HashMap<i32, i32>,
    pub short_buff: Option<ShortBuffStatusUpdate>,
    pub short_buff_at: Option<Instant>,
    pub death: Option<Die>,
    pub dice: Vec<Dice>,
    pub windows: Windows,
    pub trade: Trade,
    pub pledge_window: Pledge,
    pub stores: PrivateStores,
    pub tutorial: Tutorial,
    pub henna: Henna,
    pub friends: Friends,
    pub pet_inventory: Inventory,
    pub siege: Siege,
    pub recipes: Recipes,
    pub rare: Rare,
    pub server_objects: crate::server_objects::ServerObjects,
}

impl World {
    pub fn apply(&mut self, event: &SessionEvent) {
        match event {
            SessionEvent::UserInfo(head) => {
                self.builder_level = head.builder_level;
                let fresh = Entity::from_user_info(head);
                match self.me.as_mut().filter(|me| me.object_id == fresh.object_id) {
                    Some(me) => me.refresh(fresh),
                    None => {
                        let mut fresh = fresh;
                        fresh.anim.push(Trigger::Spawn);
                        self.me = Some(fresh);
                    }
                }
                if let Some(me) = self.me.as_mut() {
                    me.relation = head.social.relation;
                }
                self.stats = head.stats;
                let played = crate::equip_sound::diff(&self.paperdoll, &head.paperdoll, &mut self.equip_entries);
                self.effects.equip_sounds.extend(played);
                self.paperdoll = head.paperdoll;
                self.inventory_limit = head.inventory_limit;
                self.builder_level = head.builder_level;
                self.social = head.social.clone();
                let social = &head.social;
                if social.clan_id > 0 {
                    self.register_pledge(social.clan_id, social.clan_crest_id, social.ally_id, social.ally_crest_id, 0);
                }
            }
            SessionEvent::World(event) => self.apply_world(event),
            SessionEvent::Time(time) => {
                self.clock = Some(GameClock::new(time.minutes, time.ratio, Instant::now()));
            }
            _ => {}
        }
    }

    fn set_clock_minutes(&mut self, minutes: i32) {
        if let Some(clock) = self.clock {
            self.clock = Some(GameClock::new(minutes, clock.ratio, Instant::now()));
        }
    }

    fn track_attack_intend(&mut self, event: &WorldEvent) {
        let Some((me, dead)) = self.me.as_ref().map(|me| (me.object_id, me.dead)) else { return };
        match event {
            WorldEvent::Attack(a) if a.attacker_id == me && !dead
                && a.hits.first().is_some_and(|hit| hit.target_id != me && self.find(hit.target_id).is_some()) => {
                self.attack_intend = true;
            }
            WorldEvent::MoveToLocation(m) if m.object_id == me && !dead => self.attack_intend = false,
            WorldEvent::MoveToPawn(m) if m.object_id == me && !dead => self.attack_intend = false,
            WorldEvent::FlyToLocation(f) if f.object_id == me && !dead => self.attack_intend = false,
            WorldEvent::TargetSelected(t) if t.object_id == me => self.attack_intend = false,
            WorldEvent::TargetUnselected(t) if t.object_id == me => self.attack_intend = false,
            WorldEvent::TeleportToLocation(t) if t.target_id == me => self.attack_intend = false,
            WorldEvent::MyTargetSelected(_) => self.attack_intend = false,
            WorldEvent::AttackDeadTargetNoop(_)
            | WorldEvent::AttackOutOfRangeNoop(_)
            | WorldEvent::AttackInCoolTimeNoop(_)
            | WorldEvent::ActionFailed(_) => self.attack_intend = false,
            _ => {}
        }
    }

    pub fn receive_speech(&mut self, id: i32, kind: i32, text: &str) {
        let store=self.entity(id).map(|entity|entity.store_type);
        self.speech.receive(id,kind,store,text);
    }

    pub fn apply_appendix(&mut self, op: crate::appendix::AppendixOp) {
        let object_id = op.object_id();
        if self.is_me(object_id) || self.entities.contains_key(&object_id) {
            self.appendix.apply(op);
        }
    }

    pub fn apply_world(&mut self, event: &WorldEvent) {
        self.track_attack_intend(event);
        match event {
            WorldEvent::CharInfo(c) => {
                self.see(Entity::from_char_info(c));
                self.appendix.set_team(c.object_id, i32::from(c.team));
                if c.clan_id > 0 {
                    self.register_pledge(c.clan_id, c.clan_crest_id, c.ally_id, c.ally_crest_id, c.large_crest_id);
                }
            }
            WorldEvent::NpcInfo(n) => {
                let fresh = !self.is_me(n.object_id) && !self.entities.contains_key(&n.object_id);
                self.see(Entity::from_npc_info(n));
                self.appendix.set_team(n.object_id, i32::from(n.team));
                if fresh && n.spawn_animation == crate::enter_event::SPAWN_ANIMATION_ENTER_EVENT {
                    self.spawn_enter_event(n.object_id);
                }
            }
            WorldEvent::MoveToLocation(m) => {
                let origin = Vec3::new(m.x as f32, m.y as f32, m.z as f32);
                let dest = Vec3::new(m.to_x as f32, m.to_y as f32, m.to_z as f32);
                let me = self.is_me(m.object_id);
                self.update(m.object_id, |e| {
                    if !me {
                        movement::adjust_pawn_location(e, origin);
                    }
                    e.sitting = false;
                    movement::add_move_to(e, origin, dest);
                });
            }
            WorldEvent::StopMove(s) => {
                let position = Vec3::new(s.x as f32, s.y as f32, s.z as f32);
                let me = self.is_me(s.object_id);
                self.update(s.object_id, |e| {
                    if !me && !e.dead {
                        movement::adjust_pawn_location(e, position);
                    }
                    movement::stop(e);
                    e.set_heading(e.heading);
                });
            }
            WorldEvent::ValidateLocation(v) => {
                let position = Vec3::new(v.x as f32, v.y as f32, v.z as f32);
                if self.is_me(v.object_id) {
                    if let Some(me) = self.me.as_mut()
                        && let Some(delta) = movement::validate_me(me, position, v.heading)
                    {
                        log::debug!("ValidateLocation corrige o personagem em {delta:.1} u (xy)");
                    }
                } else {
                    self.update(v.object_id, |e| movement::validate_other(e, position));
                }
            }
            WorldEvent::TeleportToLocation(t) if self.teleport_fades && t.instant == 0 && self.me.as_ref().is_some_and(|me| me.object_id == t.target_id) => {
                let height = self.me.as_ref().map_or(0.0, |me| me.collision_height);
                self.observer.teleport(t.x as f32, t.y as f32, t.z as f32, height);
                self.teleport_announce = true;
                self.teleport_start(t.target_id);
            }
            WorldEvent::TeleportToLocation(t) => {
                let mine = self.me.as_ref().is_some_and(|me| me.object_id == t.target_id);
                if mine {
                    self.teleport_start(t.target_id);
                    self.entities.clear();
                    self.target = None;
                }
                self.update(t.target_id, |e| {
                    e.position = Vec3::new(t.x as f32, t.y as f32, t.z as f32);
                    e.dest = None;
                });
                if mine {
                    self.teleport_arrival(t.target_id);
                }
            }
            WorldEvent::DeleteObject(d) => {
                self.server_objects.delete_object(d.object_id);
                self.entities.remove(&d.object_id);
                self.appendix.forget(d.object_id);
                self.stores.sell_messages.remove(&d.object_id);
                self.stores.buy_messages.remove(&d.object_id);
                if self.target == Some(d.object_id) {
                    self.target = None;
                }
            }
            WorldEvent::StatusUpdate(s) => {
                if self.is_me(s.object_id) {
                    for (id, value) in &s.attributes {
                        self.stats.apply(*id, *value);
                    }
                }
                self.update(s.object_id, |e| {
                    for (id, value) in &s.attributes {
                        match *id {
                            ATTRIBUTE_LEVEL => e.level = *value,
                            ATTRIBUTE_EXP => e.exp = *value,
                            ATTRIBUTE_CURRENT_HP => e.set_hp(Some(*value), None),
                            ATTRIBUTE_MAX_HP => e.set_hp(None, Some(*value)),
                            ATTRIBUTE_CURRENT_MP => e.set_mp(Some(*value), None),
                            ATTRIBUTE_MAX_MP => e.set_mp(None, Some(*value)),
                            ATTRIBUTE_CURRENT_CP => e.set_cp(Some(*value), None),
                            ATTRIBUTE_MAX_CP => e.set_cp(None, Some(*value)),
                            _ => {}
                        }
                    }
                })
            }
            WorldEvent::ChangeMoveType(c) => self.update(c.object_id, |e| e.running = c.running),
            WorldEvent::ChangeWaitType(c) => {
                let position = Vec3::new(c.x as f32, c.y as f32, c.z as f32);
                let me = self.is_me(c.object_id);
                let change = WaitChange::from_packet(c.kind);
                self.update(c.object_id, |e| {
                    if !me {
                        movement::adjust_pawn_location(e, position);
                    }
                    if e.sitting != c.sitting {
                        movement::stop(e);
                    }
                    if let Some(change) = change {
                        e.anim.push(Trigger::ChangeWaitType { change, from: e.wait_type() });
                    }
                    e.sitting = c.sitting;
                    e.fake_dead = change == Some(WaitChange::FakeDeath);
                    if !c.sitting {
                        e.chair = None;
                    }
                });
            }
            WorldEvent::Die(d) => {
                self.appendix.delete_all(d.object_id);
                if self.is_me(d.object_id) {
                    self.death = Some(*d);
                    self.requests.push(NetCommand::TargetCancel);
                    self.buffs.clear();
                    self.short_buff = None;
                }
                if d.sweepable && self.entity(d.object_id).is_some() {
                    self.effects.sweepable_deaths.push_back(d.object_id);
                }
                self.update(d.object_id, |e| {
                    e.anim.push(Trigger::Die);
                    e.dead = true;
                    e.in_combat = false;
                    e.dest = None;
                    e.set_hp(Some(0), None);
                });
            }
            WorldEvent::Revive(r) => {
                if self.is_me(r.object_id) {
                    self.death = None;
                }
                self.update(r.object_id, |e| {
                    e.anim.push(Trigger::Revive);
                    e.dead = false;
                });
            }
            WorldEvent::TargetSelected(t) => {
                if self.is_me(t.object_id) {
                    self.target = Some(t.target_id);
                } else {
                    self.update(t.object_id, |e| e.target = Some(t.target_id));
                }
            }
            WorldEvent::TargetUnselected(t) => {
                if self.is_me(t.object_id) {
                    self.target = None;
                    self.target_color = None;
                } else {
                    self.update(t.object_id, |e| e.target = None);
                }
            }
            WorldEvent::MoveToPawn(m) => {
                let origin = Vec3::new(m.x as f32, m.y as f32, m.z as f32);
                let target = self.find(m.target_id).map(|t| t.position);
                let me = self.is_me(m.object_id);
                self.update(m.object_id, |e| {
                    if !me {
                        movement::adjust_pawn_location(e, origin);
                    }
                    match target {
                        Some(position) => movement::follow(e, origin, m.target_id, position, m.distance as f32),
                        None => movement::stop(e),
                    }
                });
            }
            WorldEvent::RelationChanged(r) => self.update(r.object_id, |e| {
                e.relation = r.relation;
                e.auto_attack = r.auto_attack != 0;
                e.karma = r.karma;
                e.pvp_flag = r.pvp_flag;
            }),
            WorldEvent::MagicSkillUse(m) => self.magic_skill_use(m),
            WorldEvent::MagicSkillLaunched(m) => {
                let targets: Vec<i32> = m.targets.iter().copied().filter(|id| self.entity(*id).is_some()).collect();
                self.update(m.caster_id, |e| {
                    e.casting = None;
                    if !targets.is_empty() {
                        e.anim.push(Trigger::MagicSkillLaunched { skill_id: m.skill_id, targets });
                    }
                });
            }
            WorldEvent::MagicSkillCanceled(m) => self.update(m.object_id, |e| {
                e.casting = None;
                e.anim.push(Trigger::MagicCancel);
            }),
            WorldEvent::SpawnItem(s) => self.see(Entity::from_spawn_item(s)),
            WorldEvent::EtcStatusUpdate(e) => self.etc_status = Some(*e),
            WorldEvent::MagicEffectIcons(m) => {
                self.buffs = m.effects.clone();
                self.buffs_at = Some(Instant::now());
            }
            WorldEvent::PledgeStatusChanged(p) => {
                self.pledge = Some(*p);
                if p.clan_id > 0 {
                    self.update(p.leader_id, |e| {
                        e.clan_id = p.clan_id;
                        e.clan_crest_id = p.crest_id;
                        e.ally_id = p.ally_id;
                        e.ally_crest_id = p.ally_crest_id;
                    });
                    self.register_pledge(p.clan_id, p.crest_id, p.ally_id, p.ally_crest_id, 0);
                }
            }
            WorldEvent::SendMacroList(list) => self.macros.apply(list),
            WorldEvent::ItemList(list) => self.inventory.apply_list(list),
            WorldEvent::InventoryUpdate(update) => self.inventory.apply_update(update),
            WorldEvent::EquipUpdate(update) => self.inventory.apply_equip(update),
            WorldEvent::ShortCutInit(init) => {
                self.shortcuts = init.shortcuts.clone();
                self.requests.push(NetCommand::RequestSkillCoolTime);
            }
            WorldEvent::ShortCutRegister(shortcut) => {
                match self.shortcuts.iter_mut().find(|s| s.slot == shortcut.slot) {
                    Some(known) => *known = *shortcut,
                    None => self.shortcuts.push(*shortcut),
                }
            }
            WorldEvent::ShortCutDelete(d) => self.shortcuts.retain(|s| s.slot != d.slot),
            WorldEvent::SkillList(list) => {
                self.skills = list.skills.clone();
                self.requests.push(NetCommand::RequestSkillCoolTime);
            }
            WorldEvent::SkillCoolTime(cool) => self.cooldowns = cool.entries.clone(),
            WorldEvent::NpcHtmlMessage(html) => self.npc_html = Some(html.clone()),
            WorldEvent::MyTargetSelected(t) => {
                self.target = Some(t.object_id);
                self.target_color = Some(t.color);
            }
            WorldEvent::Attack(a) => {
                log::debug!("pacote de ataque: {} -> {:?}", a.attacker_id, a.hits.iter().map(|h| (h.target_id, h.damage, h.flags)).collect::<Vec<_>>());
                let at = Instant::now();
                self.combat.record(a.attacker_id, &a.hits, at);
                let first = a.hits.first().map(|h| h.target_id);
                let target_position = first.and_then(|target_id| self.find(target_id).map(|e| e.position));
                let hit = a.hits.first().map(|h| Hit {
                    target: h.target_id,
                    damage: h.damage,
                    miss: h.miss(),
                    critical: h.critical(),
                    shield: h.shield(),
                    spirit: h.soulshot(),
                    soulshot_grade: (h.flags & 0x0f) as i32,
                });
                self.update(a.attacker_id, |e| {
                    movement::adjust_pawn_location(e, Vec3::new(a.x as f32, a.y as f32, a.z as f32));
                    movement::stop(e);
                    if let Some(position) = target_position {
                        let delta = position - e.position;
                        if delta.x != 0.0 || delta.y != 0.0 {
                            e.set_heading(movement::heading_of(delta));
                        }
                    }
                    if !e.dead {
                        e.in_combat = true;
                        if let Some(hit) = hit {
                            e.anim.push(Trigger::Attack(hit));
                        }
                    }
                    e.target = first;
                });
            }
            WorldEvent::DropItem(d) => {
                let source_position = self.entity(d.player_id).filter(|e| !matches!(e.kind,EntityKind::Item { .. })).map(|e| e.position);
                self.see(Entity::from_drop_item(d,source_position));
            }
            WorldEvent::GetItem(g) => {
                self.entities.remove(&g.object_id);
                if self.target == Some(g.object_id) {
                    self.target = None;
                }
                self.update(g.player_id, |e| {
                    e.dest = None;
                    e.anim.push(Trigger::PicItem);
                });
            }
            WorldEvent::SunRise(_) => {
                self.daylight = Some(true);
                self.set_clock_minutes(SUN_RISE_MINUTES);
            }
            WorldEvent::SunSet(_) => {
                self.daylight = Some(false);
                self.set_clock_minutes(SUN_SET_MINUTES);
            }
            WorldEvent::AutoAttackStart(a) => {
                if self.is_me(a.target_id) {
                    self.combat.auto_attack_target = self.target;
                }
                self.combat.auto_attacking.insert(a.target_id, self.target.unwrap_or(0));
                self.update(a.target_id, |e| e.in_combat = true);
            }
            WorldEvent::AutoAttackStop(a) => {
                if self.is_me(a.target_id) {
                    self.combat.auto_attack_target = None;
                }
                self.combat.auto_attacking.remove(&a.target_id);
                self.update(a.target_id, |e| e.in_combat = false);
            }
            WorldEvent::AskJoinPledge(ask) => {
                if self.is_user(ask.requestor_id) {
                    self.dialogs.push(Dialog::JoinPledge(ask.clone()));
                } else {
                    self.requests.push(NetCommand::AnswerJoinPledge { accept: false });
                }
            }
            WorldEvent::AskJoinParty(ask) => {
                if ask.requestor_name.is_empty() {
                    self.requests.push(NetCommand::AnswerJoinParty { accept: false });
                } else {
                    self.dialogs.push(Dialog::JoinParty(ask.clone()));
                }
            }
            WorldEvent::AskJoinAlliance(ask) => {
                if self.is_user(ask.requestor_id) {
                    self.dialogs.push(Dialog::JoinAlliance(ask.clone()));
                } else {
                    self.requests.push(NetCommand::AnswerJoinAlly { accept: false });
                }
            }
            WorldEvent::ConfirmDlg(dlg) => self.dialogs.push(Dialog::Confirm(dlg.clone())),
            WorldEvent::DoorInfo(d) => {
                let door = self.doors.entry(d.object_id).or_default();
                door.door_id = d.door_id;
                door.info_door_id = Some(d.door_id);
            }
            WorldEvent::DoorStatusUpdate(d) => self.doors.entry(d.object_id).or_default().apply(d),
            WorldEvent::PartySmallWindowAll(p) => self.party.apply_all(p),
            WorldEvent::PartySmallWindowAdd(p) => self.party.apply_add(p),
            WorldEvent::PartySmallWindowDeleteAll(_) => self.party.clear(),
            WorldEvent::PartySmallWindowDelete(p) => self.party.remove(p.object_id),
            WorldEvent::PartySmallWindowUpdate(m) => self.party.update_existing(m),
            WorldEvent::PartyMemberPosition(p) => {
                for m in &p.positions {
                    self.party.positions.insert(m.object_id, [m.x, m.y, m.z]);
                }
            }
            WorldEvent::PartySpelled(p) => {
                self.party.spelled.insert(p.object_id, p.effects.clone());
            }
            WorldEvent::VehicleInfo(v) => {
                self.vehicles.insert(v.object_id, Vehicle::from_info(v));
            }
            WorldEvent::VehicleDeparture(v) => {
                if let Some(vehicle) = self.vehicles.get_mut(&v.object_id) {
                    vehicle.depart(v);
                }
            }
            WorldEvent::VehicleStart(v) => {
                if let Some(vehicle) = self.vehicles.get_mut(&v.object_id) {
                    vehicle.state = v.state;
                }
            }
            WorldEvent::GetOnVehicle(b) => {
                let base = self.vehicles.get(&b.vehicle_id).map(|v| Vec3::new(v.x as f32, v.y as f32, v.z as f32));
                self.update(b.object_id, |e| {
                    e.vehicle = Some(b.vehicle_id);
                    e.dest = None;
                    if let Some(base) = base {
                        e.position = base + Vec3::new(b.x as f32, b.y as f32, b.z as f32);
                    }
                });
            }
            WorldEvent::GetOffVehicle(b) => self.update(b.object_id, |e| {
                e.vehicle = None;
                e.dest = None;
                e.position = Vec3::new(b.x as f32, b.y as f32, b.z as f32);
            }),
            WorldEvent::MoveToLocationInVehicle(m) => {
                let base = self.vehicles.get(&m.vehicle_id).map(|v| Vec3::new(v.x as f32, v.y as f32, v.z as f32));
                self.update(m.object_id, |e| {
                    e.vehicle = Some(m.vehicle_id);
                    if let Some(base) = base {
                        let origin = base + Vec3::new(m.x as f32, m.y as f32, m.z as f32);
                        let dest = base + Vec3::new(m.to_x as f32, m.to_y as f32, m.to_z as f32);
                        movement::add_move_to(e, origin, dest);
                    }
                });
            }
            WorldEvent::StopMoveInVehicle(m) => {
                let base = self.vehicles.get(&m.vehicle_id).map(|v| Vec3::new(v.x as f32, v.y as f32, v.z as f32));
                self.update(m.object_id, |e| {
                    e.vehicle = Some(m.vehicle_id);
                    match base {
                        Some(base) => {
                            movement::stop_at(e, base + Vec3::new(m.x as f32, m.y as f32, m.z as f32), m.heading)
                        }
                        None => movement::stop(e),
                    }
                });
            }
            WorldEvent::StartRotating(r) => self.update(r.object_id, |e| {
                e.rotation = Some(Rotation { degree: r.degree, side: r.side, speed: r.speed });
            }),
            WorldEvent::FinishRotating(r) => self.update(r.object_id, |e| {
                e.rotation = None;
                e.set_heading(r.degree);
            }),
            WorldEvent::SetupGauge(g) => {
                self.gauge = if g.time > 0 { Some(Gauge { setup: *g, started: Instant::now() }) } else { None };
            }
            WorldEvent::Ride(r) => self.update(r.object_id, |e| {
                e.mount = r.mounted.then_some(Mount { ride_type: r.ride_type, ride_class_id: r.ride_class_id });
            }),
            WorldEvent::ServerObjectInfo(o) => self.see(Entity::from_server_object_info(o)),
            WorldEvent::StaticObject(s) => {
                self.static_objects.insert(s.object_id, s.static_object_id);
            }
            WorldEvent::PlaySound(s) => self.effects.push_sound(s.clone()),
            WorldEvent::PetStatusShow(p) => self.pet_window = Some(p.summon_type),
            WorldEvent::PetInfo(p) => {
                self.see(Entity::from_pet_info(p));
                self.appendix.set_team(p.object_id, i32::from(p.team));
                let mine = self.me.as_ref().is_some_and(|me| p.title == me.name);
                if mine || self.pet.as_ref().is_some_and(|pet| pet.info.object_id == p.object_id) {
                    let window_shown = self.pet.as_ref().is_some_and(|pet| pet.info.summon_type == p.summon_type);
                    if !window_shown {
                        self.party.spelled.remove(&p.object_id);
                    }
                    let (max_time, remaining_time) =
                        self.pet.as_ref().map(|pet| (pet.max_time, pet.remaining_time)).unwrap_or((0, 0));
                    self.pet = Some(Pet { info: (**p).clone(), max_time, remaining_time });
                }
            }
            WorldEvent::PetStatusUpdate(p) => {
                if let Some(pet) = self.pet.as_mut().filter(|pet| pet.info.object_id == p.object_id) {
                    pet.apply_status(p);
                }
                self.update(p.object_id, |e| {
                    e.hp = Some((p.hp, p.max_hp));
                    e.mp = Some((p.mp, p.max_mp));
                    e.level = p.level;
                    e.exp = p.exp;
                });
            }
            WorldEvent::PetDelete(p) => {
                if self.pet.as_ref().is_some_and(|pet| pet.info.object_id == p.object_id) {
                    self.pet = None;
                    self.pet_window = None;
                }
            }
            WorldEvent::SetSummonRemainTime(t) => {
                if let Some(pet) = self.pet.as_mut() {
                    pet.max_time = t.max_time;
                    pet.remaining_time = t.remaining_time;
                }
            }
            WorldEvent::Earthquake(q) => {
                if self.effects.earthquakes.len() == crate::state::SOUND_QUEUE {
                    self.effects.earthquakes.pop_front();
                }
                self.effects.earthquakes.push_back(*q);
            }
            WorldEvent::SpecialCamera(c) => {
                self.effects.camera_commands.push_back(CameraCommand::Special(*c));
            }
            WorldEvent::FlyToLocation(f) => self.update(f.object_id, |e| {
                e.position = Vec3::new(f.to_x as f32, f.to_y as f32, f.to_z as f32);
                e.dest = None;
            }),
            WorldEvent::NicknameChanged(n) => self.update(n.object_id, |e| e.title = n.title.clone()),
            WorldEvent::Dice(d) => {
                self.dice.push(*d);
                trim_oldest(&mut self.dice, HISTORY_LIMIT);
            }
            WorldEvent::SocialAction(s) => {
                if s.action > 0 {
                    let action = s.action as u32;
                    self.update(s.object_id, |e| e.anim.push(Trigger::Social(action)));
                }
            }
            WorldEvent::ChairSit(c) => self.update(c.object_id, |e| {
                e.anim.push(Trigger::ChairSit);
                e.chair = Some(c.static_object_id);
                e.sitting = true;
                movement::stop(e);
            }),
            WorldEvent::ShortBuffStatusUpdate(b) => {
                self.short_buff = if b.duration > 0 { Some(*b) } else { None };
                self.short_buff_at = Some(Instant::now());
            }
            WorldEvent::SellList(list) => self.windows.sell = Some(list.clone()),
            WorldEvent::BuyList(list) => self.windows.buy = Some(list.clone()),
            WorldEvent::TradeStart(start) => self.trade.start(start),
            WorldEvent::TradeOwnAdd(add) => self.trade.mine.extend(add.items.iter().copied()),
            WorldEvent::TradeOtherAdd(add) => self.trade.theirs.extend(add.items.iter().copied()),
            WorldEvent::TradeUpdate(update) => {
                for entry in &update.items {
                    let index = self.trade.tradable.iter().position(|t| t.object_id == entry.item.object_id);
                    match (entry.update_type, index) {
                        (1, _) => self.trade.tradable.push(entry.item),
                        (2, Some(index)) => { self.trade.tradable.remove(index); }
                        (3, Some(index)) => self.trade.tradable[index] = entry.item,
                        _ => {}
                    }
                }
            }
            WorldEvent::TradeDone(done) => self.trade.done(done.success),
            WorldEvent::TradeStartOk(_) => {}
            WorldEvent::TradePressOwnOk(_) | WorldEvent::TradePressOtherOk(_) => {}
            WorldEvent::ShowRadar(radar) => {
                self.windows.radar.apply_control(radar.first, radar.second, [radar.third, radar.fourth, radar.fifth]);
            }
            WorldEvent::DeleteRadar(radar) => self.windows.radar.delete([radar.first, radar.second, radar.third]),
            WorldEvent::FriendAdd(_) | WorldEvent::FriendRemove(_) => {}
            WorldEvent::FriendListNoop(_) | WorldEvent::FriendStatusNoop(_) | WorldEvent::MagicListNoop(_) => {}
            WorldEvent::TradeRequest(request) => {
                if self.is_user(request.sender_id) {
                    self.dialogs.push(Dialog::TradeRequest { requestor_id: request.sender_id });
                } else {
                    self.requests.push(NetCommand::AnswerTrade { accept: false });
                }
            }
            WorldEvent::ManagePledgePower(power) => self.pledge_window.power = Some(*power),
            WorldEvent::JoinPledge(join) => self.pledge_window.joined_pledge_id = Some(join.pledge_id),
            WorldEvent::JoinParty(join) => self.windows.party_join_result = Some(join.response),
            WorldEvent::StorageMaxCount(limits) => self.storage_limits = Some(limits.clone()),
            WorldEvent::AutoSoulShot(s) => {
                let [item_id, enabled] = s.values;
                if item_id == 0 && enabled == 0 {
                    self.combat.auto_soulshots.values_mut().for_each(|on| *on = false);
                }
                self.combat.auto_soulshots.insert(item_id, enabled != 0);
            }
            WorldEvent::FishingEnd(f) => self.fishing_end(f.value, f.flag),
            WorldEvent::FishingStartCombat(f) => self.fishing_combat(f.values[0], f.values[1], f.values[2], f.flags[0], f.flags[1] != 0),
            WorldEvent::FishingHpRegen(f) => self.fishing_regen(f),
            WorldEvent::EnchantSkillList(_) => {}
            WorldEvent::EnchantSkillInfo(_) => {}
            WorldEvent::ShowQuestInfo => {}
            WorldEvent::ShowQuestMark(_) => {}
            WorldEvent::SendManorList(list) => self.windows.manor.castles = list.entries.clone(),
            WorldEvent::ShowSeedInfo(info) => self.windows.manor.seed_info = Some(info.clone()),
            WorldEvent::ShowCropInfo(info) => self.windows.manor.crop_info = Some(info.clone()),
            WorldEvent::ShowManorDefaultInfo(info) => self.windows.manor.default_info = Some(info.clone()),
            WorldEvent::ShowSeedSetting(setting) => self.windows.manor.seed_setting = Some(setting.clone()),
            WorldEvent::ShowCropSetting(setting) => self.windows.manor.crop_setting = Some(setting.clone()),
            WorldEvent::ShowSellCropList(list) => self.windows.manor.sell_crop_list = Some(list.clone()),
            WorldEvent::ShowProcureCropDetail(detail) => self.windows.manor.procure_crop_detail = Some(detail.clone()),
            WorldEvent::HeroList(list) => self.windows.hero_tower.list = Some(list.clone()),
            WorldEvent::ServerPrimitive(primitive) => self.server_objects.apply_primitive(primitive),
            WorldEvent::AskJoinMpcc(ask) => self.dialogs.push(Dialog::JoinMpcc { name: ask.name.clone() }),
            WorldEvent::PledgeCrestLarge(crest) => {
                if crest.values[1] > 0 && crest.values[2] > 0 {
                    self.pledge_window.large_crests.entry(crest.values[1]).or_insert_with(|| crest.data.clone());
                }
            }
            WorldEvent::OlympiadUserInfo(_) => {}
            WorldEvent::OlympiadSpelledInfo(_) => {}
            WorldEvent::OlympiadMode(mode) => self.observer.olympiad_mode(mode.mode),
            WorldEvent::EventMatchManage(_) => {}
            WorldEvent::MultiPartyChannelInfo(_) => {}
            WorldEvent::RegenMax(r) => self.regen_fill = crate::regen::RegenFill::start(r.values[1], r.values[2], r.amount, self.seconds),
            WorldEvent::EventMatchObserver(_) => {}
            WorldEvent::EventMatchMessage(_) => {}
            WorldEvent::EventMatchScore(_) => {}
            WorldEvent::EventMatchTeamInfo(_) => {}
            WorldEvent::EventMatchCreate(_) => {}
            WorldEvent::EventMatchUserInfo(_) => {}
            WorldEvent::ColosseumFenceInfo(info) => self.server_objects.apply_fence(info),
            WorldEvent::EventMatchGmTest => {}
            WorldEvent::ClosePartyRoom => {}
            WorldEvent::FishingStart(f) => self.fishing_start(f.values),
            WorldEvent::OpenMpcc => {}
            WorldEvent::CloseMpcc => {}
            WorldEvent::OlympiadMatchEnd => {}
            WorldEvent::MailArrived => {}
            WorldEvent::EventMatchSpelledInfo(_) => {}
            WorldEvent::EventMatchFirecracker(f) => self.effects.firecrackers.push_back(f.value),
            WorldEvent::EventMatchTeamUnlocked(_) => {}
            WorldEvent::PartyRoomMember(_) => {}
            WorldEvent::ManagePartyRoomMember(_) => {}
            WorldEvent::WarehouseDepositList(list) => {
                self.windows.warehouse = Some(list.clone());
                self.windows.warehouse_deposit = true;
            }
            WorldEvent::WarehouseWithdrawList(list) => {
                self.windows.warehouse = Some(list.clone());
                self.windows.warehouse_deposit = false;
            }
            WorldEvent::WarehouseDone(_) => {}
            WorldEvent::PledgeShowMemberListAll(all) => {
                if all.pledge_id != 0 && !all.name.is_empty() {
                    self.register_pledge(all.pledge_id, all.crest_id, all.ally_id, all.ally_crest_id, 0);
                    self.pledge_window.members.clear();
                    self.pledge_window.listed.clear();
                    self.pledge_window.list_reset = self.pledge_window.list_reset.wrapping_add(1);
                    for member in &all.members {
                        self.pledge_window.upsert(member);
                    }
                    self.pledge_window.info = Some(all.clone());
                }
            }
            WorldEvent::PledgeShowMemberListUpdate(member) => self.pledge_window.update_existing(member),
            WorldEvent::PledgeShowMemberListAdd(member) => self.pledge_window.upsert(member),
            WorldEvent::PledgeShowMemberListDelete(delete) => self.pledge_window.remove(&delete.name),
            WorldEvent::PledgeShowMemberListDeleteAll(_) => self.pledge_window.clear_header(),
            WorldEvent::PledgeCrest(crest) => {
                self.pledge_window.crests.insert(crest.crest_id, crest.data.clone());
            }
            WorldEvent::AllianceCrest(crest) => {
                self.pledge_window.ally_crests.insert(crest.crest_id, crest.data.clone());
            }
            WorldEvent::PledgeInfo(info) => {
                self.pledge_window.summary = Some(info.clone());
                if info.pledge_id > 0
                    && let Some(record) = self.pledges.get_mut(&info.pledge_id)
                {
                    record.name = info.name.clone();
                    record.ally_name = info.ally_name.clone();
                }
            }
            WorldEvent::PledgeExtendedInfo(_) => {}
            WorldEvent::StartPledgeWarNoop(_)
            | WorldEvent::ReplyStartPledgeWarNoop(_)
            | WorldEvent::StopPledgeWarNoop(_)
            | WorldEvent::ReplyStopPledgeWarNoop(_)
            | WorldEvent::SurrenderPledgeWarNoop(_)
            | WorldEvent::ReplySurrenderPledgeWarNoop(_)
            | WorldEvent::SetPledgeCrestNoop(_) => {}
            WorldEvent::SurrenderPersonallyNoop(_) | WorldEvent::ServerCloseSocketNoop(_) => {}
            WorldEvent::ReplyStartAllianceWarNoop(_)
            | WorldEvent::ReplyStopAllianceWarNoop(_)
            | WorldEvent::SurrenderAllianceWarNoop(_) => {}
            WorldEvent::StartAllianceWar(w) | WorldEvent::StopAllianceWar(w) => {
                if !w.first.is_empty() && !w.second.is_empty() {
                    let start = matches!(event, WorldEvent::StartAllianceWar(_));
                    self.dialogs.push(Dialog::AllianceWar { start, first: w.first.clone(), second: w.second.clone() });
                }
            }
            WorldEvent::MagicAndSkillList(m) => {
                self.requests.push(NetCommand::RequestMagicSkillList { first: m.second, second: m.first, third: m.third });
            }
            WorldEvent::SkillRemainSec(r) => {
                let [caster, target, skill_id, level, _, _, remain] = r.values;
                let target = if self.find(target).is_some() { target } else { caster };
                let started = Instant::now();
                self.update(caster, |e| {
                    if e.dead {
                        return;
                    }
                    e.magic.pending = None;
                    e.anim.push(Trigger::ActionStop);
                    e.casting = Some(Casting { skill_id, skill_level: level, hit_time: remain * 1000, target_id: target, started });
                    e.anim.push(Trigger::MagicSkillUse { skill_id, level, hit_time: remain * 1000, target });
                });
            }
            WorldEvent::VehicleCheckLocation(_) => {}
            WorldEvent::DummyNoop(_) | WorldEvent::AbnormalVisualEffectNoop(_) => {}
            WorldEvent::BlockListNoop(_) => {}
            WorldEvent::CreatePledge(_)
            | WorldEvent::WithdrawalPledge(_)
            | WorldEvent::OustPledgeMember(_)
            | WorldEvent::DismissPledge(_) => {}
            WorldEvent::WithdrawalParty(_)
            | WorldEvent::OustPartyMember(_)
            | WorldEvent::DismissParty(_) => {}
            WorldEvent::SetOustPledgeMemberNoop(_)
            | WorldEvent::SetDismissPledgeNoop(_)
            | WorldEvent::SetOustPartyMemberNoop(_)
            | WorldEvent::SetDismissPartyNoop(_) => {}
            WorldEvent::AttackOutOfRangeNoop(_)
            | WorldEvent::AttackInCoolTimeNoop(_)
            | WorldEvent::AttackDeadTargetNoop(_) => {}
            WorldEvent::ValidateLocationInVehicle(v) => {
                let [object_id, vehicle_id, x, y, z, yaw] = v.values;
                if let Some(vehicle) = self.vehicles.get(&vehicle_id).copied() {
                let local = Vec3::new(x as f32, y as f32, z as f32);
                let world = vehicle_world(&vehicle, local);
                let mine = self.is_me(object_id);
                self.update(object_id, |e| {
                    if e.dead {
                        return;
                    }
                    e.vehicle = Some(vehicle_id);
                    if mine {
                        e.position = world;
                        e.dest = None;
                        e.set_heading(yaw);
                        return;
                    }
                    let current = vehicle_local(&vehicle, e.position);
                    if (current.z - local.z).abs() > VEHICLE_REPOSITION_HEIGHT {
                        e.position = world;
                        e.dest = None;
                    } else if matches!(e.anim.action, crate::anim::Action::None) && current.distance(local) > VEHICLE_SNAP_DISTANCE {
                        movement::add_move_to(e, e.position, world);
                    }
                });
                }
            }
            WorldEvent::GmHide(_) => {}
            WorldEvent::JoinAllianceResult(_)
            | WorldEvent::WithdrawAllianceResult(_)
            | WorldEvent::OustAllianceMemberPledgeResult(_)
            | WorldEvent::DismissAllianceResult(_)
            | WorldEvent::SetAllianceCrestNoop(_)
            | WorldEvent::AllianceInfoNoop(_) => {}
            WorldEvent::RequestTimeCheck(t) => self.time_check = Some((t.value, 0)),
            WorldEvent::ObserverStart(start) => {
                if let Some((id, height)) = self.me.as_ref().map(|me| (me.object_id, me.collision_height)) {
                    let yaw = self.observer.start(start.values, height);
                    self.update(id, |entity| {
                        entity.set_heading(yaw);
                        entity.dest = None;
                    });
                }
            }
            WorldEvent::ObserverEnd(end) => {
                if let Some(height) = self.me.as_ref().map(|me| me.collision_height) {
                    self.observer.end(end.values, height);
                }
            }
            WorldEvent::NormalCameraNoop(_) => self.effects.camera_commands.push_back(CameraCommand::Normal),
            WorldEvent::EventTrigger(trigger) => self.event_trigger = Some(event_trigger(trigger)),
            WorldEvent::NetPingResponse(ping) => {
                let fps = if self.frame_time == 0.0 { 0 } else { (1.0 / f64::from(self.frame_time)) as i32 };
                let range = (f64::from(self.pawn_clipping) * 32768.0 * 0.0625) as i32;
                self.requests.push(NetCommand::NetPing { first: ping.value, second: fps, third: range });
                self.frame_time = 0.0;
            }
            WorldEvent::Snoop(_) => {}
            WorldEvent::CameraMode(m) => self.effects.camera_commands.push_back(CameraCommand::Mode(m.mode)),
            WorldEvent::PetitionVoteNoop(_) => {}
            WorldEvent::PledgeShowInfoUpdate(update) => {
                self.register_pledge(update.pledge_id, update.crest_id, 0, 0, 0);
                if let Some(info) = self.pledge_window.info.as_mut() {
                    info.crest_id = update.crest_id;
                    info.level = update.level;
                    info.castle_id = update.castle_id;
                    info.hideout_id = update.hideout_id;
                    info.my_level = update.leader_level;
                    info.dissolving = update.dissolving;
                }
                self.pledge_window.last_update = Some(update.clone());
            }
            WorldEvent::ShowBoard(board) => self.windows.board = board.clone(),
            WorldEvent::ChooseInventoryItem(choose) => self.windows.chosen_item_id = Some(choose.item_id),
            WorldEvent::ClientAction(a) => self.pending_client_action = Some(a.action),
            WorldEvent::FriendAddRequest(request) => {
                self.dialogs.push(Dialog::FriendRequest { requestor_name: request.requestor_name.clone() })
            }
            WorldEvent::FriendAddRequestResult(_) => {}
            WorldEvent::NewCharacterFailNoop(_) | WorldEvent::ReviveRequestNoop(_) => {}
            WorldEvent::QuestList(list) => self.windows.quests = list.quests.clone(),
            WorldEvent::EnchantResult(result) => self.windows.enchant_result = Some(result.result),
            WorldEvent::AcquireSkillList(list) => self.windows.learnable = Some(list.clone()),
            WorldEvent::AcquireSkillInfo(info) => self.windows.skill_info = Some(info.clone()),
            WorldEvent::AcquireSkillDone(_) => self.windows.skill_info = None,
            WorldEvent::PartyMatchList(list) => self.windows.party_rooms = Some(list.clone()),
            WorldEvent::PartyMatchDetail(detail) => self.windows.party_room = Some(detail.clone()),
            WorldEvent::PrivateStoreManageListSell(list) => self.stores.manage_sell = Some(list.clone()),
            WorldEvent::PrivateStoreListSell(list) => self.stores.sell = Some(list.clone()),
            WorldEvent::PrivateStoreMsgSell(msg) => {
                self.stores.sell_messages.insert(msg.object_id, msg.message.clone());
            }
            WorldEvent::PrivateStoreManageListBuy(list) => self.stores.manage_buy = Some(list.clone()),
            WorldEvent::PrivateStoreListBuy(list) => self.stores.buy = Some(list.clone()),
            WorldEvent::PrivateStoreMsgBuy(msg) => {
                self.stores.buy_messages.insert(msg.object_id, msg.message.clone());
            }
            WorldEvent::ShowMinimap(map) => self.windows.minimap = Some(*map),
            WorldEvent::TutorialShowHtml(html) => self.tutorial.html = Some(html.html.clone()),
            WorldEvent::TutorialShowQuestionMark(mark) => self.tutorial.question_mark = Some(mark.mark_id),
            WorldEvent::TutorialEnableClientEvent(event) => self.tutorial.client_event = Some(event.event_id),
            WorldEvent::TutorialCloseHtml(_) => self.tutorial.html = None,
            WorldEvent::PetItemList(list) => {
                self.pet_inventory.apply_list(&ItemList { show_window: true, items: list.items.clone() });
            }
            WorldEvent::PetInventoryUpdate(update) => self.pet_inventory.apply_update(update),
            WorldEvent::PackageToList(list) => self.windows.package_targets = Some(list.clone()),
            WorldEvent::PackageSendableList(list) => self.windows.package_items = Some(list.clone()),
            WorldEvent::MultiSellList(list) => {
                if let Err(error) = self.windows.multisell.get_or_insert_default().receive(list) {
                    log::warn!("multisell: oferta {} excede os 50 slots de produto do C4", error.entry_id);
                }
            }
            WorldEvent::ShowTownMap(map) => self.windows.town_map = Some(map.clone()),
            WorldEvent::HennaEquipList(list) => self.henna.equip_list = Some(list.clone()),
            WorldEvent::HennaItemInfo(info) => self.henna.item_info = Some(*info),
            WorldEvent::HennaInfo(info) => self.henna.info = Some(info.clone()),
            WorldEvent::HennaUnequipList(list) => self.henna.unequip_list = Some(list.clone()),
            WorldEvent::HennaUnequipInfo(info) => self.henna.unequip_info = Some(*info),
            WorldEvent::GmViewSkillInfo(_) | WorldEvent::GmViewMagicInfo(_) => {}
            WorldEvent::GmViewQuestInfo(_) => {}
            WorldEvent::GmViewItemList(_) => {}
            WorldEvent::GmViewWarehouseWithdrawList(_) => {}
            WorldEvent::SellListProcure(_) => {}
            WorldEvent::GmHennaInfo(_) => {}
            WorldEvent::L2Friend(friend) => {
                self.friends.apply_packet(friend);
            }
            WorldEvent::L2FriendStatus(status) => {
                self.friends.apply_status(status);
            }
            WorldEvent::GmViewPledgeInfo(_) => {}
            WorldEvent::GmViewCharacterInfo(_) => {}
            WorldEvent::BuyListSeed(list) => self.windows.seeds = Some(list.clone()),
            WorldEvent::RadarControl(radar) => self.windows.radar.apply_control(radar.show, radar.kind, [radar.x, radar.y, radar.z]),
            WorldEvent::ShopPreviewList(list) => self.windows.preview_list = Some(list.clone()),
            WorldEvent::ShopPreviewInfo(info) => self.windows.preview = Some(*info),
            WorldEvent::FriendList(list) => {
                self.friends.list = list.friends.clone();
                self.friends.sort();
            }
            WorldEvent::FriendRecvMsg(msg) => {
                self.friends.messages.push(msg.clone());
                trim_oldest(&mut self.friends.messages, HISTORY_LIMIT);
            }
            WorldEvent::CastleSiegeInfo(info) => self.siege.info = Some(info.clone()),
            WorldEvent::CastleSiegeAttackerList(list) => self.siege.attackers = Some(list.clone()),
            WorldEvent::CastleSiegeDefenderList(list) => self.siege.defenders = Some(list.clone()),
            WorldEvent::RecipeBookItemList(book) => self.recipes.book = Some(book.clone()),
            WorldEvent::RecipeItemMakeInfo(info) => self.recipes.make_info = Some(*info),
            WorldEvent::RecipeShopManageList(list) => self.recipes.shop_manage = Some(list.clone()),
            WorldEvent::RecipeShopSellList(list) => self.recipes.shop_sell = Some(list.clone()),
            WorldEvent::RecipeShopItemInfo(info) => self.recipes.shop_item = Some(*info),
            WorldEvent::RecipeShopMsg(msg) => {
                self.recipes.shop_messages.insert(msg.object_id, msg.message.clone());
            }
            WorldEvent::ShowCalculator(calc) => self.rare.calculator = Some(calc.calculator_id),
            WorldEvent::MonRaceInfo(race) => self.rare.race = Some(race.clone()),
            WorldEvent::ShowXMasSeal(seal) => self.rare.xmas_seal = Some(seal.item_id),
            WorldEvent::SsqStatus(status) => self.rare.ssq_status = Some(status.clone()),
            WorldEvent::AgitDecoInfo(deco) => self.rare.agit_deco = Some(deco.agit_id),
            WorldEvent::SsqInfo(info) => self.rare.ssq_sky = Some(info.sky),
            WorldEvent::ActionFailed(_)
            | WorldEvent::Say2(_)
            | WorldEvent::NpcSay(_)
            | WorldEvent::SystemMessage(_) => {}
        }
    }

    pub fn shortcut(&self, slot: i32) -> Option<&Shortcut> {
        self.shortcuts.iter().find(|s| s.slot == slot)
    }

    pub fn target_entity(&self) -> Option<&Entity> {
        self.find(self.target?)
    }

    pub fn find(&self, object_id: i32) -> Option<&Entity> {
        self.me.as_ref().filter(|me| me.object_id == object_id).or_else(|| self.entities.get(&object_id))
    }

    fn magic_skill_use(&mut self, m: &MagicSkillUse) {
        self.last_cast = None;
        let Some(caster) = self.find(m.caster_id) else { return };
        let caster_is_me = self.is_me(m.caster_id);
        let target = match self.find(m.target_id) {
            Some(target) if !matches!(target.kind, EntityKind::Item { .. }) => CastTarget::Pawn { is_me: self.is_me(m.target_id) },
            _ if self.doors.contains_key(&m.target_id) => CastTarget::Mover,
            _ => CastTarget::Missing,
        };
        let context = CastContext {
            keep_min_frame_rate: self.keep_min_frame_rate,
            pawn_clipping: self.pawn_clipping,
            me_position: self.me.as_ref().map(|me| me.position),
            caster_position: caster.position,
            caster_rendered: self.rendered.contains(&m.caster_id),
            caster_is_me,
            caster_dead: caster.dead,
            target,
        };
        let Some(variant) = magic::variant_of(target).filter(|_| magic::admitted(&context)) else { return };
        let position = Vec3::new(m.x as f32, m.y as f32, m.z as f32);
        if !caster_is_me {
            self.update(m.caster_id, |e| {
                movement::adjust_pawn_location(e, position);
            });
        }
        if m.hit_time < 0 {
            return;
        }
        self.last_cast = Some(AcceptedCast {
            caster_id: m.caster_id,
            target_id: m.target_id,
            skill_id: m.skill_id,
            skill_level: m.skill_level,
            hit_time: m.hit_time,
            reuse_delay: m.reuse_delay,
            caster_is_me,
            variant,
        });
        let info = MagicInfo {
            effect_actor: m.extra.iter().map(|value| i32::from(*value as i16)).collect(),
            ground_targets: m.positions.iter().map(|p| Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32)).collect(),
        };
        let started = Instant::now();
        let trigger = Trigger::MagicSkillUse { skill_id: m.skill_id, level: m.skill_level, hit_time: m.hit_time, target: m.target_id };
        self.update(m.caster_id, |e| {
            e.casting =
                Some(Casting { skill_id: m.skill_id, skill_level: m.skill_level, hit_time: m.hit_time, target_id: m.target_id, started });
            e.magic.pending = Some(info);
            e.anim.push(match variant {
                CastVariant::Pawn => trigger,
                CastVariant::Mover => Trigger::MagicSkillUseOnMover { skill_id: m.skill_id, level: m.skill_level, hit_time: m.hit_time, target: m.target_id },
            });
        });
    }

    pub fn cast_accepted(&self, m: &MagicSkillUse) -> Option<&AcceptedCast> {
        self.last_cast.as_ref().filter(|cast| {
            cast.caster_id == m.caster_id
                && cast.target_id == m.target_id
                && cast.skill_id == m.skill_id
                && cast.skill_level == m.skill_level
                && cast.hit_time == m.hit_time
        })
    }

    pub fn is_me(&self, object_id: i32) -> bool {
        self.me.as_ref().is_some_and(|me| me.object_id == object_id)
    }

    pub fn is_user(&self, object_id: i32) -> bool {
        self.find(object_id).is_some_and(|entity| !matches!(entity.kind, EntityKind::Item { .. }))
    }

    pub fn register_pledge(&mut self, clan_id: i32, crest_id: i32, ally_id: i32, ally_crest_id: i32, large_crest_id: i32) {
        match self.pledges.get_mut(&clan_id) {
            None => {
                self.pledges.insert(
                    clan_id,
                    PledgeRecord { pledge_id: clan_id, crest_id, ally_id, ally_crest_id, large_crest_id, ..Default::default() },
                );
                self.request_pledge_info(clan_id);
            }
            Some(record) => {
                record.crest_id = crest_id;
                record.large_crest_id = large_crest_id;
                let ally_changed = record.ally_id != ally_id;
                record.ally_id = ally_id;
                record.ally_crest_id = ally_crest_id;
                if ally_changed {
                    self.request_pledge_info(clan_id);
                }
            }
        }
        if crest_id > 0 && !self.pledge_window.crests.contains_key(&crest_id) && self.crests_requested.insert(crest_id) {
            self.requests.push(NetCommand::RequestPledgeCrest { crest_id });
        }
        if ally_id > 0 && ally_crest_id > 0 && !self.pledge_window.ally_crests.contains_key(&ally_crest_id) && self.ally_crests_requested.insert(ally_crest_id) {
            self.requests.push(NetCommand::RequestAllyCrest { crest_id: ally_crest_id });
        }
        if clan_id > 0 && large_crest_id > 0 && !self.pledge_window.large_crests.contains_key(&large_crest_id) {
            self.requests.push(NetCommand::ExPledgeCrestLarge { crest_id: large_crest_id });
        }
    }

    pub fn request_pledge_info(&mut self, pledge_id: i32) {
        self.requests.push(NetCommand::RequestPledgeInfo { pledge_id });
        self.requested_pledges.note_pledge_requested(pledge_id, pledge_id);
    }

    pub fn pledge_name(&self, clan_id: i32) -> Option<&str> {
        self.pledges.get(&clan_id).map(|p| p.name.as_str()).filter(|n| !n.is_empty())
    }

    pub fn ally_name(&self, clan_id: i32) -> Option<&str> {
        self.pledges.get(&clan_id).map(|p| p.ally_name.as_str()).filter(|n| !n.is_empty())
    }

    pub fn search_stores(&mut self, word: &str) {
        self.store_matches.clear();
        if word.is_empty() {
            return;
        }
        let word = word.to_uppercase();
        let matches: Vec<(i32, u8)> = self
            .me
            .iter()
            .chain(self.entities.values())
            .filter_map(|entity| {
                let (message, kind) = match entity.store_type {
                    1 | 8 => (self.stores.sell_messages.get(&entity.object_id), if entity.store_type == 1 { 1 } else { 4 }),
                    3 => (self.stores.buy_messages.get(&entity.object_id), 2),
                    5 => (self.recipes.shop_messages.get(&entity.object_id), 3),
                    _ => return None,
                };
                let message = message.filter(|text| !text.is_empty())?;
                message.to_uppercase().contains(&word).then_some((entity.object_id, kind))
            })
            .collect();
        self.store_matches.extend(matches);
    }

    pub fn take_requests(&mut self) -> Vec<NetCommand> {
        std::mem::take(&mut self.requests)
    }

    pub fn tick(&mut self, dt: f32) {
        self.speech.tick(dt);
        self.server_objects.tick(dt);
        self.seconds += f64::from(dt);
        if let Some(hp) = self.me.as_ref().and_then(|me| me.hp).map(|(current, _)| current) {
            if hp != self.regen_hp {
                self.regen_fill.decay(self.seconds);
            }
            self.regen_hp = hp;
            if hp < 1 {
                self.regen_fill.clear();
            }
        }
        self.frame_time = if self.frame_time != 0.0 { (f64::from(dt) * 0.02 + f64::from(self.frame_time) * 0.98) as f32 } else { dt };
        if let Some((target, elapsed)) = &mut self.time_check {
            *elapsed += (f64::from(dt) * 1000.0) as i32;
            if *elapsed >= *target {
                let (target, elapsed) = (*target, *elapsed);
                if self.me.is_some() {
                    self.requests.push(NetCommand::SendTimeCheck { first: target, second: elapsed });
                }
                self.time_check = None;
            }
        }
        self.fishing_window.tick(dt);
        self.tick_rising(dt);
        let followed: Vec<i32> = self.me.iter().chain(self.entities.values()).filter_map(|e| e.follow).map(|f| f.target_id).collect();
        let positions: std::collections::HashMap<i32, Vec3> =
            followed.into_iter().filter_map(|id| self.find(id).map(|t| (id, t.position))).collect();
        if let Some(me) = &mut self.me {
            movement::track(me, &positions);
        }
        for entity in self.entities.values_mut() {
            movement::track(entity, &positions);
        }
        if let Some(me) = &mut self.me {
            me.assign_turn_rate(&self.rotation_rates);
            me.assign_walk_anim_frame(&self.walk_anim_frames);
            movement::step(me, dt);
            me.tick_name_blink(dt);
        }
        for entity in self.entities.values_mut() {
            entity.assign_turn_rate(&self.rotation_rates);
            entity.assign_walk_anim_frame(&self.walk_anim_frames);
            movement::step(entity, dt);
            entity.tick_name_blink(dt);
        }
    }

    pub fn observer_fade_done(&mut self) -> bool {
        let Some(location) = self.observer.fade_done() else { return false };
        let Some(id) = self.me.as_ref().map(|me| me.object_id) else { return false };
        self.entities.clear();
        self.target = None;
        self.update(id, |entity| {
            entity.position = Vec3::new(location[0], location[1], location[2]);
            entity.dest = None;
        });
        self.teleport_arrival(id);
        true
    }

    fn teleport_start(&mut self, object_id: i32) {
        self.target = None;
        self.attack_intend = false;
        self.music_stop = true;
        self.update(object_id, |entity| {
            entity.dest = None;
            entity.anim.push(Trigger::ActionStop);
        });
    }

    fn teleport_arrival(&mut self, object_id: i32) {
        self.music_reset = true;
        self.update(object_id, |entity| entity.chair = None);
    }

    pub fn entity(&self, object_id: i32) -> Option<&Entity> {
        self.me.as_ref().filter(|me| me.object_id == object_id).or_else(|| self.entities.get(&object_id))
    }

    pub fn select(&mut self, object_id: i32) -> bool {
        if self.entity(object_id).is_none() && self.static_target(object_id).is_none() {
            return false;
        }
        self.target = Some(object_id);
        true
    }

    pub fn static_target(&self, object_id: i32) -> Option<StaticTarget> {
        if let Some(door) = self.doors.get(&object_id)
            && let Some(static_object_id) = door.info_door_id
        {
            return Some(StaticTarget { object_id, static_object_id, hp: Some((door.hp, door.max_hp)) });
        }
        self.static_objects
            .get(&object_id)
            .map(|&static_object_id| StaticTarget { object_id, static_object_id, hp: None })
    }

    pub fn targeted_static(&self) -> Option<StaticTarget> {
        let object_id = self.target?;
        if self.entity(object_id).is_some() {
            return None;
        }
        self.static_target(object_id)
    }

    pub fn object_for_server_id(&self, server_id: i32) -> Option<i32> {
        let door = self.doors.iter().filter(|(_, door)| door.info_door_id == Some(server_id)).map(|(&id, _)| id);
        let object = self.static_objects.iter().filter(|(_, id)| **id == server_id).map(|(&id, _)| id);
        door.chain(object).min()
    }

    pub fn assist_target(&self) -> Option<i32> {
        let wanted = self.targeted()?.target?;
        match self.entity(wanted) {
            Some(entity) => Some(entity.object_id),
            None => self.static_target(wanted).map(|target| target.static_object_id),
        }
    }

    pub fn action_command(&self, object_id: i32, shift: bool) -> Option<NetCommand> {
        let me = self.me.as_ref()?;
        let origin = [me.position.x as i32, me.position.y as i32, me.position.z as i32];
        Some(NetCommand::Action { object_id, origin, shift })
    }

    pub fn client_action(&self, action: i32, shift: bool) -> Option<NetCommand> {
        let me = self.me.as_ref()?;
        let origin = [me.position.x as i32, me.position.y as i32, me.position.z as i32];
        let object_id = match action {
            CLIENT_ACTION_NEAREST_PAWN => self.next_enemy(CLIENT_ACTION_RADIUS, self.targeted().map_or(-1, |t| t.object_id))?,
            CLIENT_ACTION_NEAREST_ITEM => self.nearest_item(CLIENT_ACTION_RADIUS, -1)?,
            CLIENT_ACTION_TARGET => self.assist_target()?,
            _ => return None,
        };
        (object_id > 0).then_some(NetCommand::Action { object_id, origin, shift })
    }

    fn fishing_start(&mut self, values: [i32; 5]) {
        let object_id = values[0];
        let Some(entity) = self.entity_mut(object_id).filter(|entity| entity.fishing == 0) else { return };
        entity.fishing = 1;
        entity.anim.push(Trigger::FishingStart);
        let pose = (entity.position, entity.heading);
        if self.is_me(object_id) {
            self.fishing_window.reset();
            self.fishing_window.visible = true;
            let float = crate::fishing_view::float_location(Vec3::new(values[2] as f32, values[3] as f32, values[4] as f32), None);
            self.fishing_view.cast(pose.0, [0, pose.1, 0], float, values[1]);
        }
    }

    fn fishing_combat(&mut self, object_id: i32, time: i32, hp: i32, mode: u8, deceptive: bool) {
        let Some(entity) = self.entity_mut(object_id).filter(|entity| entity.fishing != 0 && entity.fishing != 2) else { return };
        entity.fishing = 2;
        if self.is_me(object_id) {
            self.fishing_view.combat(mode);
            self.fishing_window.combat(hp, time, deceptive);
            self.fishing_window.update(hp, time, false, mode, crate::fishing::ACTION_BAIT, 0);
        }
    }

    fn fishing_regen(&mut self, regen: &FishingHpRegen) {
        let [object_id, time, hp] = regen.values;
        let [mode, hit, action] = regen.flags;
        let Some(entity) = self.entity_mut(object_id).filter(|entity| entity.fishing == 2) else { return };
        if action > 0 {
            entity.anim.push(Trigger::FishingPull);
        }
        if self.is_me(object_id) {
            self.fishing_window.update(hp, time, hit != 0, mode, action, regen.amount);
            self.fishing_view.regen(mode, action);
        }
    }

    fn fishing_end(&mut self, object_id: i32, success: u8) {
        let Some(entity) = self.entity_mut(object_id).filter(|entity| entity.fishing != 0) else { return };
        entity.anim.push(Trigger::FishingEnd);
        entity.fishing = 0;
        if self.is_me(object_id) {
            self.fishing_view.end();
            if success == 1 {
                self.fishing_window.success();
                self.fishing_window.update(0, 0, false, 0, crate::fishing::ACTION_SUCCESS, 0);
            } else {
                self.fishing_window.visible = false;
            }
        }
    }

    pub fn targeted(&self) -> Option<&Entity> {
        self.entity(self.target?)
    }

    pub fn nearest(&self, count: usize) -> Vec<(&Entity, f32)> {
        let Some(me) = &self.me else { return Vec::new() };
        let mut list: Vec<(&Entity, f32)> =
            self.entities.values().map(|e| (e, e.position.distance(me.position))).collect();
        list.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.object_id.cmp(&b.0.object_id)));
        list.truncate(count);
        list
    }

    fn spawn_enter_event(&mut self, object_id: i32) {
        use crate::enter_event::{EnterEffect, EnterSound, EFFECT_SCALE_PER_RADIUS};
        let Some(entity) = self.entities.get_mut(&object_id) else { return };
        let EntityKind::Npc { npc_id, .. } = entity.kind else { return };
        let Some(row) = l2_dat::entereventgrp::enter_of(&self.enter_events, npc_id as u32).cloned() else { return };
        if let Some(class) = crate::enter_event::effect_of(&row) {
            self.effects.enter_effects.push_back(EnterEffect {
                object_id,
                class: class.to_owned(),
                location: [entity.position.x, entity.position.y, entity.position.z - entity.collision_height],
                yaw: entity.heading,
                scale: entity.collision_radius * EFFECT_SCALE_PER_RADIUS,
            });
        }
        if let Some(rise) = crate::enter_event::begin(row.is_rise, entity.position.z, entity.collision_height) {
            entity.position.z = rise.start;
            self.rising.insert(object_id, rise);
        }
        if let Some(name) = crate::enter_event::anim_of(&row) {
            entity.anim.push(Trigger::PlayAnim(name.to_owned()));
        }
        if let Some(name) = crate::enter_event::sound_of(&row) {
            self.effects.enter_sounds.push_back(EnterSound {
                object_id,
                name: name.to_owned(),
                position: [entity.position.x, entity.position.y, entity.position.z],
                volume: row.sound_vol,
                radius: row.sound_rad,
            });
        }
    }

    fn tick_rising(&mut self, dt: f32) {
        let ids: Vec<i32> = self.rising.keys().copied().collect();
        for id in ids {
            let Some(rise) = self.rising.get(&id).copied() else { continue };
            let Some(entity) = self.entities.get_mut(&id) else {
                self.rising.remove(&id);
                continue;
            };
            let (z, done) = crate::enter_event::step(&rise, entity.position.z, dt);
            if let Some(z) = z {
                entity.position.z = z;
            }
            if done {
                self.rising.remove(&id);
            }
        }
    }

    fn see(&mut self, fresh: Entity) {
        if self.me.as_ref().is_some_and(|me| me.object_id == fresh.object_id) {
            return;
        }
        if matches!(fresh.kind,EntityKind::Item { .. }) {
            self.spawn_sequence += 1;
            let mut fresh = fresh;
            fresh.spawned = self.spawn_sequence;
            self.entities.insert(fresh.object_id,fresh);
            return;
        }
        match self.entities.get_mut(&fresh.object_id) {
            Some(existing) => {
                let hp = existing.hp;
                let mp = existing.mp;
                let cp = existing.cp;
                let position = existing.position;
                let heading = existing.heading;
                let dest = existing.dest;
                let stop_heading = existing.stop_heading;
                let speed_bonus = existing.speed_bonus;
                let speed_bonus_left = existing.speed_bonus_left;
                let relation = existing.relation;
                let interaction = (existing.auto_attack, existing.surrender_war_id);
                let target = existing.target;
                let casting = existing.casting;
                let spawned = existing.spawned;
                let name_blink = (existing.name_blink, existing.name_blink_elapsed);
                let anim = std::mem::take(&mut existing.anim);
                *existing = fresh;
                existing.anim = anim;
                existing.spawned = spawned;
                (existing.name_blink, existing.name_blink_elapsed) = name_blink;
                existing.hp = hp;
                existing.mp = mp;
                existing.cp = cp;
                existing.position = position;
                existing.set_heading(heading);
                existing.dest = dest;
                existing.stop_heading = stop_heading;
                existing.speed_bonus = speed_bonus;
                existing.speed_bonus_left = speed_bonus_left;
                existing.relation = relation;
                if matches!(existing.kind, EntityKind::Player { .. }) {
                    (existing.auto_attack, existing.surrender_war_id) = interaction;
                }
                existing.target = target;
                existing.casting = casting;
            }
            None => {
                let mut fresh = fresh;
                self.spawn_sequence += 1;
                fresh.spawned = self.spawn_sequence;
                if !matches!(fresh.kind, EntityKind::Item { .. }) {
                    fresh.anim.push(Trigger::Spawn);
                }
                self.entities.insert(fresh.object_id, fresh);
            }
        }
    }

    pub fn apply_shop_preview_info(&mut self, info: &ShopPreviewInfo, body_part: impl Fn(i32) -> Option<u32>) {
        use l2_dat::armorgrp::body_part::{ALLDRESS, ONEPIECE};
        use l2_net::game::{EQUIP_BACK, EQUIP_CHEST, EQUIP_FEET, EQUIP_GLOVES, EQUIP_HAIR, EQUIP_HEAD, EQUIP_LEGS, EQUIP_LHAND, EQUIP_LRHAND, EQUIP_RHAND};
        let Some(me) = self.me.as_mut() else { return };
        let EntityKind::Player { equipped, enchant_effect, .. } = &mut me.kind else { return };
        let slot = |index: usize| info.slots[index];
        let chest_part = |equipped: &[i32; VISIBLE_EQUIP_SLOTS]| {
            let chest = equipped[EQUIP_CHEST];
            if chest > 0 { body_part(chest) } else { None }
        };
        if slot(6) != 0 {
            equipped[EQUIP_HEAD] = slot(6);
        }
        if slot(7) != 0 {
            if equipped[EQUIP_LRHAND] != 0 {
                equipped[EQUIP_LRHAND] = 0;
                equipped[EQUIP_LHAND] = 0;
            }
            equipped[EQUIP_RHAND] = slot(7);
            *enchant_effect = 0;
        }
        if slot(8) != 0 {
            if equipped[EQUIP_LRHAND] != 0 {
                equipped[EQUIP_LRHAND] = 0;
                equipped[EQUIP_RHAND] = 0;
            }
            equipped[EQUIP_LHAND] = slot(8);
        }
        for (index, target) in [(9, EQUIP_GLOVES), (12, EQUIP_FEET)] {
            if slot(index) != 0 {
                if chest_part(equipped) == Some(ALLDRESS) {
                    equipped[EQUIP_CHEST] = 0;
                    equipped[EQUIP_GLOVES] = 0;
                    equipped[EQUIP_FEET] = 0;
                }
                equipped[target] = slot(index);
            }
        }
        if slot(11) != 0 {
            match chest_part(equipped) {
                Some(ONEPIECE) => equipped[EQUIP_CHEST] = 0,
                Some(ALLDRESS) => {
                    equipped[EQUIP_GLOVES] = 0;
                    equipped[EQUIP_FEET] = 0;
                }
                _ => {}
            }
            equipped[EQUIP_LEGS] = slot(11);
        }
        if slot(10) != 0 {
            equipped[EQUIP_CHEST] = slot(10);
        }
        if slot(13) != 0 {
            equipped[EQUIP_BACK] = slot(13);
        }
        if slot(14) != 0 {
            equipped[EQUIP_LHAND] = 0;
            equipped[EQUIP_RHAND] = 0;
            equipped[EQUIP_LRHAND] = slot(14);
            *enchant_effect = 0;
        }
        if slot(15) != 0 {
            equipped[EQUIP_HAIR] = slot(15);
        }
    }

    pub fn entity_mut(&mut self, object_id: i32) -> Option<&mut Entity> {
        match self.me.as_mut() {
            Some(me) if me.object_id == object_id => Some(me),
            _ => self.entities.get_mut(&object_id),
        }
    }

    fn update(&mut self, object_id: i32, change: impl FnOnce(&mut Entity)) {
        if let Some(entity) = self.entity_mut(object_id) {
            change(entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2_net::game::packets::PAPERDOLL_SLOTS;
    use l2_net::game::packets::{
        ATTRIBUTE_CURRENT_CP, ATTRIBUTE_CURRENT_HP, ATTRIBUTE_CURRENT_MP, ATTRIBUTE_MAX_HP, ATTRIBUTE_MAX_MP,
    };
    use l2_net::game::{
        Buff, EtcStatusUpdate, Macro, MagicEffectIcons, MagicSkillCanceled, MagicSkillLaunched, MagicSkillUse,
        MoveToPawn, PledgeStatusChanged, RelationChanged, SendMacroList, SpawnItem, TargetSelected, TargetUnselected,
    };
    use l2_net::game::{
        ChangeMoveType, ChangeWaitType, CharInfo, ClientSetTime, DeleteObject, Die, MoveToLocation, NpcInfo, Revive,
        StatusUpdate, StopMove, TeleportToLocation, UserInfoHead, ValidateLocation,
    };
    use l2_net::game::{PaperdollSlot, UserStats};

    fn head() -> UserInfoHead {
        UserInfoHead {
            x: 83400,
            abnormal_effect: 0,
            y: 148000,
            z: -3400,
            heading: 16384,
            object_id: 268481001,
            name: "Wender".into(),
            race: 0,
            sex: 1,
            class_id: 109,
            level: 52,
            exp: 12345,
            max_hp: 900,
            hp: 400,
            max_mp: 300,
            mp: 200,
            run_speed: 117,
            walk_speed: 80,
            speed_multiplier: 1.5,
            attack_speed_multiplier: 1.0,
            pvp_flag: 0,
            karma: 0,
            collision_radius: 9.0,
            collision_height: 23.0,
            hair_style: 2,
            hair_color: 1,
            face: 3,
            builder_level: 0,
            title: "Titulo".into(),
            max_cp: 500,
            cp: 450,
            stats: UserStats::default(),
            paperdoll: [PaperdollSlot::default(); PAPERDOLL_SLOTS],
            inventory_limit: 80,
            name_color: 0xffffff,
            social: Default::default(),
            enchant_effect: 0,
        }
    }

    pub(super) fn npc(object_id: i32) -> NpcInfo {
        NpcInfo {
            object_id,
            npc_id: 20001,
            attackable: true,
            x: 10,
            y: 20,
            z: 30,
            heading: 32768,
            m_atk_speed: 333,
            p_atk_speed: 253,
            run_speed: 120,
            walk_speed: 60,
            speed_multiplier: 1.25,
            attack_speed_multiplier: 1.0,
            collision_radius: 8.0,
            collision_height: 24.0,
            rhand: 5,
            chest: 21,
            lhand: 6,
            name_above: true,
            running: true,
            in_combat: false,
            alike_dead: false,
            spawn_animation: 1,
            name: "Gremlin".into(),
            title: String::new(),
            playable_flag: 0,
            pvp_flag: 0,
            karma: 0,
            abnormal_effect: 0,
            clan_id: 0,
            clan_crest_id: 0,
            ally_id: 0,
            ally_crest_id: 0,
            team: 0,
            base_collision_radius: 7.0,
            base_collision_height: 20.0,
        }
    }

    fn enter_event_row(is_rise: u32, spawn_type: u32) -> l2_dat::entereventgrp::EnterEvent {
        l2_dat::entereventgrp::EnterEvent {
            id: 20001,
            reserved: String::new(),
            sound_name: "SkillSound4.summon_shot".into(),
            sound_vol: 250.0,
            sound_rad: 100.0,
            is_rise,
            spawn_type,
            effect_name: "LineageEffect.e_u011_b".into(),
            anim_name: "spatk01".into(),
        }
    }

    #[test]
    fn search_stores_marks_the_stores_whose_message_contains_the_word_by_kind_like_is_matched_store_msg() {
        let mut world = World::default();
        let mut seller = Entity::from_user_info(&head());
        seller.object_id = 11;
        seller.store_type = 1;
        let mut buyer = seller.clone();
        buyer.object_id = 12;
        buyer.store_type = 3;
        let mut idle = seller.clone();
        idle.object_id = 13;
        idle.store_type = 0;
        for entity in [seller, buyer, idle] {
            world.entities.insert(entity.object_id, entity);
        }
        world.stores.sell_messages.insert(11, "Selling Sword of Limit".into());
        world.stores.buy_messages.insert(12, "Buying SWORD".into());
        world.stores.sell_messages.insert(13, "sword".into());
        world.search_stores("sword");
        assert_eq!(world.store_matches.get(&11), Some(&1));
        assert_eq!(world.store_matches.get(&12), Some(&2));
        assert_eq!(world.store_matches.get(&13), None);
        world.search_stores("bow");
        assert!(world.store_matches.is_empty());
        world.search_stores("sword");
        world.search_stores("");
        assert!(world.store_matches.is_empty());
    }

    #[test]
    fn the_dword_after_the_face_of_user_info_is_the_builder_level() {
        let mut world = World::default();
        assert_eq!(world.builder_level, 0);
        let mut gm = head();
        gm.builder_level = 3;
        world.apply(&SessionEvent::UserInfo(Box::new(gm)));
        assert_eq!(world.builder_level, 3);
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        assert_eq!(world.builder_level, 0);
    }

    #[test]
    fn an_npc_info_with_spawn_animation_two_runs_the_enter_event_row_of_the_npc() {
        let mut world = World { enter_events: l2_dat::entereventgrp::by_npc(vec![enter_event_row(1, 1)]), ..World::default() };
        let mut info = npc(7);
        info.spawn_animation = 2;
        world.apply_world(&WorldEvent::NpcInfo(info.clone()));
        let effects = world.effects.take_enter_effects();
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].class, "LineageEffect.e_u011_b");
        assert_eq!(effects[0].location, [10.0, 20.0, 30.0 - 24.0]);
        assert_eq!(effects[0].yaw, 32768);
        assert_eq!(effects[0].scale, 8.0 * crate::enter_event::EFFECT_SCALE_PER_RADIUS);
        let sounds = world.effects.take_enter_sounds();
        assert_eq!(sounds.len(), 1);
        assert_eq!((sounds[0].name.as_str(), sounds[0].volume, sounds[0].radius), ("SkillSound4.summon_shot", 250.0, 100.0));
        assert_eq!(sounds[0].position, [10.0, 20.0, 30.0 - 48.0]);
        assert_eq!(world.entities[&7].position.z, 30.0 - 48.0);
        assert!(world.entities[&7].anim.pending.contains(&Trigger::PlayAnim("spatk01".into())));
        assert!(world.rising.contains_key(&7));
        for _ in 0..600 {
            world.tick(1.0 / 60.0);
        }
        assert_eq!(world.entities[&7].position.z, 30.0);
        assert!(world.rising.is_empty());
    }

    #[test]
    fn the_enter_event_only_runs_for_a_new_npc_with_an_enter_row_and_spawn_animation_two() {
        let mut world = World { enter_events: l2_dat::entereventgrp::by_npc(vec![enter_event_row(0, 1)]), ..World::default() };
        let mut info = npc(7);
        world.apply_world(&WorldEvent::NpcInfo(info.clone()));
        info.spawn_animation = 2;
        world.apply_world(&WorldEvent::NpcInfo(info.clone()));
        assert!(world.effects.take_enter_effects().is_empty());
        info.object_id = 8;
        world.apply_world(&WorldEvent::NpcInfo(info.clone()));
        assert_eq!(world.effects.take_enter_effects().len(), 1);
        assert_eq!(world.effects.take_enter_sounds().len(), 1);
        assert!(world.rising.is_empty());
        assert_eq!(world.entities[&8].position.z, 30.0);
        world.enter_events = l2_dat::entereventgrp::by_npc(vec![enter_event_row(1, 0)]);
        info.object_id = 9;
        world.apply_world(&WorldEvent::NpcInfo(info.clone()));
        assert!(world.effects.take_enter_effects().is_empty() && world.rising.is_empty());
        world.enter_events.clear();
        info.object_id = 10;
        world.apply_world(&WorldEvent::NpcInfo(info));
        assert!(world.effects.take_enter_effects().is_empty() && world.effects.take_enter_sounds().is_empty());
    }

    #[test]
    fn char_info_with_a_strider_or_wyvern_spawns_the_character_mounted() {
        let mut info = char_info(5);
        assert_eq!(Entity::from_char_info(&info).mount, None);
        info.mount_type = 1;
        info.mount_npc_id = 1_012_526;
        assert_eq!(Entity::from_char_info(&info).mount, Some(Mount { ride_type: 1, ride_class_id: 1_012_526 }));
        info.mount_type = 2;
        assert_eq!(Entity::from_char_info(&info).mount.map(|mount| mount.ride_type), Some(2));
        info.mount_type = 3;
        assert_eq!(Entity::from_char_info(&info).mount, None);
    }

    pub(super) fn char_info(object_id: i32) -> CharInfo {
        CharInfo {
            x: 1,
            y: 2,
            z: 3,
            heading: 4,
            object_id,
            name: "Luzal".into(),
            race: 1,
            sex: 0,
            class_id: 20,
            equipped: [7; 11],
            pvp_flag: 1,
            karma: 30,
            m_atk_speed: 0,
            p_atk_speed: 0,
            run_speed: 130,
            walk_speed: 70,
            swim_run_speed: 0,
            swim_walk_speed: 0,
            ground_run_speed: 0,
            ground_walk_speed: 0,
            fly_run_speed: 0,
            fly_walk_speed: 0,
            speed_multiplier: 1.0,
            attack_speed_multiplier: 1.0,
            collision_radius: 8.0,
            collision_height: 23.0,
            hair_style: 2,
            hair_color: 1,
            face: 3,
            title: "Titulo".into(),
            clan_id: 0,
            clan_crest_id: 0,
            ally_id: 0,
            ally_crest_id: 0,
            sitting: true,
            running: false,
            in_combat: false,
            alike_dead: false,
            invisible: false,
            mount_type: 0,
            store_type: 0,
            cubics: Vec::new(),
            finding_party: 0,
            abnormal_effect: 0,
            recommendations_left: 0,
            recommendations_received: 0,
            mount_npc_id: 0,
            active_class_id: 20,
            enchant_effect: 0,
            team: 0,
            large_crest_id: 0,
            noble: 0,
            hero: 0,
            fishing: 0,
            fish_x: 0,
            fish_y: 0,
            fish_z: 0,
            name_color: 0xffffff,
        }
    }

    #[test]
    fn the_team_byte_of_char_npc_and_pet_info_picks_the_team_circle_and_death_or_removal_drops_it() {
        use crate::appendix::{IDENTIFIER_BLUE, IDENTIFIER_RED};
        let mut world = World::default();
        let mut blue = char_info(2);
        blue.team = 1;
        world.apply_world(&WorldEvent::CharInfo(Box::new(blue)));
        assert_eq!(world.appendix.effects(2), IDENTIFIER_BLUE);
        let mut red = npc(3);
        red.team = 2;
        world.apply_world(&WorldEvent::NpcInfo(red));
        assert_eq!(world.appendix.effects(3), IDENTIFIER_RED);
        let mut none = char_info(2);
        none.team = 0;
        world.apply_world(&WorldEvent::CharInfo(Box::new(none)));
        assert_eq!(world.appendix.effects(2), 0);
        world.apply_world(&WorldEvent::Die(l2_net::game::Die { object_id: 3, to_village: false, to_hideout: false, to_castle: false, to_siege_hq: false, sweepable: false, to_fixed_spot: false }));
        assert_eq!(world.appendix.effects(3), 0);
        world.apply_world(&WorldEvent::DeleteObject(l2_net::game::DeleteObject { object_id: 3 }));
        assert_eq!(world.appendix.ids().collect::<Vec<_>>(), vec![2]);
    }

    #[test]
    fn appendix_ops_only_reach_pawns_that_exist() {
        use crate::appendix::AppendixOp;
        let mut world = World::default();
        world.apply_world(&WorldEvent::CharInfo(Box::new(char_info(2))));
        world.apply_appendix(AppendixOp::Add { object_id: 2, bit: 0x10 });
        world.apply_appendix(AppendixOp::Add { object_id: 9, bit: 0x10 });
        assert_eq!(world.appendix.effects(2), 0x10);
        assert_eq!(world.appendix.ids().collect::<Vec<_>>(), vec![2]);
    }

    #[test]
    fn a_char_info_with_a_clan_registers_the_pledge_and_asks_for_its_info_and_crests_once() {
        let mut world = World::default();
        let mut info = char_info(2);
        info.clan_id = 33;
        info.clan_crest_id = 5;
        info.ally_id = 44;
        info.ally_crest_id = 6;
        info.store_type = 2;
        world.apply_world(&WorldEvent::CharInfo(Box::new(info.clone())));
        assert_eq!(
            world.take_requests(),
            vec![
                NetCommand::RequestPledgeInfo { pledge_id: 33 },
                NetCommand::RequestPledgeCrest { crest_id: 5 },
                NetCommand::RequestAllyCrest { crest_id: 6 },
            ]
        );
        assert_eq!(world.entities[&info.object_id].clan_id, 33);
        assert_eq!(world.entities[&info.object_id].store_type, 2);
        assert_eq!(world.pledge_name(33), None);
        world.apply_world(&WorldEvent::CharInfo(Box::new(info.clone())));
        assert!(world.take_requests().is_empty());
        world.apply_world(&WorldEvent::PledgeInfo(PledgeInfo { pledge_id: 33, name: "Canastra".into(), ally_name: "Baralho".into() }));
        assert_eq!(world.pledge_name(33), Some("Canastra"));
        assert_eq!(world.ally_name(33), Some("Baralho"));
        info.ally_id = 45;
        world.apply_world(&WorldEvent::CharInfo(Box::new(info.clone())));
        assert_eq!(world.take_requests(), vec![NetCommand::RequestPledgeInfo { pledge_id: 33 }]);
        world.apply_world(&WorldEvent::PledgeCrest(Crest { crest_id: 7, data: vec![1] }));
        let mut other = info.clone();
        other.object_id = 99;
        other.clan_id = 34;
        other.clan_crest_id = 7;
        other.ally_id = 0;
        other.ally_crest_id = 0;
        world.apply_world(&WorldEvent::CharInfo(Box::new(other)));
        assert_eq!(world.take_requests(), vec![NetCommand::RequestPledgeInfo { pledge_id: 34 }]);
        let mut head = head();
        head.social.clan_id = 35;
        head.social.clan_crest_id = 8;
        world.apply(&SessionEvent::UserInfo(Box::new(head)));
        assert_eq!(world.me.as_ref().unwrap().clan_id, 35);
        assert_eq!(world.take_requests(), vec![NetCommand::RequestPledgeInfo { pledge_id: 35 }, NetCommand::RequestPledgeCrest { crest_id: 8 }]);
        assert!(world.requested_pledges.get_requested_pledge_id(33));
        assert_eq!(world.requested_pledges.find_requested_pledge(34), 34);
        assert!(!world.requested_pledges.get_requested_pledge_id(36));
    }

    #[test]
    fn requested_pledges_follow_the_tmap_find_104114d0_and_set_1041b120() {
        let mut pledges = RequestedPledges::default();
        assert_eq!(pledges.find_requested_pledge(5), 0);
        pledges.note_pledge_requested(5, 5);
        assert_eq!(pledges.find_requested_pledge(5), 5);
        pledges.note_pledge_requested(5, -2);
        assert_eq!(pledges.find_requested_pledge(5), -2);
        assert!(!pledges.get_requested_pledge_id(5));
        pledges.note_pledge_requested(0, 9);
        assert!(!pledges.get_requested_pledge_id(0));
        pledges.note_pledge_requested(7, 7);
        assert!(pledges.get_requested_pledge_id(7));
    }

    #[test]
    fn char_info_and_user_info_keep_the_hero_bit_of_pawn_1710() {
        let mut world = World::default();
        let mut info = char_info(77);
        info.hero = 1;
        world.apply_world(&WorldEvent::CharInfo(Box::new(info)));
        assert!(matches!(world.entities[&77].kind, EntityKind::Player { hero: true, .. }));
        let mut me = head();
        me.social.hero = true;
        world.apply(&SessionEvent::UserInfo(Box::new(me)));
        assert!(matches!(world.me.as_ref().unwrap().kind, EntityKind::Player { hero: true, .. }));
    }

    #[test]
    fn char_info_keeps_the_original_weapon_enchant_effect() {
        let mut world = World::default();
        let mut info = char_info(77);
        info.enchant_effect = 20;
        world.apply_world(&WorldEvent::CharInfo(Box::new(info)));
        assert!(matches!(world.entities[&77].kind, EntityKind::Player { enchant_effect: 20, .. }));
    }

    #[test]
    fn receive_info_handlers_replace_the_pawn_16c0_mask_without_merging_or_losing_bits() {
        let mut world = World::default();
        let mut me = head();
        let mut player = char_info(77);
        let mut monster = npc(88);
        for mask in [0, 0x40, 0x10000, i32::MIN, -1, 0] {
            me.abnormal_effect = mask;
            player.abnormal_effect = mask;
            monster.abnormal_effect = mask;
            world.apply(&SessionEvent::UserInfo(Box::new(me.clone())));
            world.apply_world(&WorldEvent::CharInfo(Box::new(player.clone())));
            world.apply_world(&WorldEvent::NpcInfo(monster.clone()));
            assert_eq!(world.me.as_ref().unwrap().abnormal_effect, mask);
            assert_eq!(world.entities[&77].abnormal_effect, mask);
            assert_eq!(world.entities[&88].abnormal_effect, mask);
        }
    }

    #[test]
    fn user_info_keeps_and_updates_the_local_weapon_enchant() {
        let mut world = World::default();
        let mut info = head();
        info.enchant_effect = 20;
        world.apply(&SessionEvent::UserInfo(Box::new(info.clone())));
        assert!(matches!(world.me.as_ref().unwrap().kind, EntityKind::Player { enchant_effect: 20, .. }));
        info.enchant_effect = 0;
        world.apply(&SessionEvent::UserInfo(Box::new(info)));
        assert!(matches!(world.me.as_ref().unwrap().kind, EntityKind::Player { enchant_effect: 0, .. }));
    }

    #[test]
    fn entering_world_fills_me_with_position_vitals_and_speed() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        let me = world.me.as_ref().expect("me deveria estar preenchido");
        assert_eq!(me.object_id, 268481001);
        assert_eq!(me.name, "Wender");
        assert_eq!(me.heading, 16384);
        assert_eq!(me.position, Vec3::new(83400.0, 148000.0, -3400.0));
        assert_eq!(me.hp, Some((400, 900)));
        assert_eq!(me.mp, Some((200, 300)));
        assert_eq!(me.cp, Some((450, 500)));
        assert_eq!(me.level, 52);
        assert_eq!((me.run_speed, me.walk_speed, me.speed_multiplier), (117, 80, 1.5));
        assert!(matches!(
            me.kind,
            EntityKind::Player { class_id: 109, race: 0, sex: 1, hair_style: 2, hair_color: 1, face: 3, .. }
        ));
        assert_eq!(me.collision_height, 23.0);
        assert_eq!((me.karma, me.pvp_flag), (0, 0));
        assert_eq!(me.title, "Titulo");
        assert!(world.entities.is_empty());
    }

    #[test]
    fn npc_info_inserts_an_npc_and_char_info_a_player() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::NpcInfo(npc(1234)));
        let e = &world.entities[&1234];
        assert!(matches!(e.kind, EntityKind::Npc { npc_id: 20001, rhand: 5, chest: 21, lhand: 6, attackable: true }));
        assert_eq!(e.name, "Gremlin");
        assert_eq!(e.position, Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(e.heading, 32768);
        assert_eq!((e.run_speed, e.walk_speed, e.speed_multiplier), (120, 60, 1.25));
        assert_eq!(e.collision_height, 24.0);
        assert_eq!(e.level, 0);
        assert!(e.running && !e.sitting && !e.dead && e.dest.is_none());
        assert_eq!((e.karma, e.pvp_flag), (0, 0));
        world.apply_world(&WorldEvent::CharInfo(Box::new(char_info(77))));
        let p = &world.entities[&77];
        assert!(
            matches!(p.kind, EntityKind::Player { class_id: 20, race: 1, sex: 0, equipped, hair_style: 2, hair_color: 1, face: 3, .. } if equipped == [7; 11])
        );
        assert_eq!(p.title, "Titulo");
        assert_eq!((p.karma, p.pvp_flag), (30, 1));
        assert_eq!(p.collision_height, 23.0);
        assert!(p.sitting && !p.running);
        assert_eq!(world.entities.len(), 2);
    }

    #[test]
    fn a_second_npc_info_updates_in_place_and_keeps_hp_position_and_heading() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::StatusUpdate(StatusUpdate {
            object_id: 1,
            attributes: vec![(ATTRIBUTE_CURRENT_HP, 50)],
        }));
        let mut again = npc(1);
        again.x = 99;
        again.heading = 7;
        again.run_speed = 99;
        again.collision_height = 30.0;
        world.apply_world(&WorldEvent::NpcInfo(again));
        assert_eq!(world.entities.len(), 1);
        let e = &world.entities[&1];
        assert_eq!(e.position, Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(e.heading, 32768);
        assert_eq!(e.run_speed, 99);
        assert_eq!(e.collision_height, 30.0);
        assert_eq!(e.hp, Some((50, 0)));
    }

    #[test]
    fn a_second_npc_info_keeps_dest_while_walking() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: 1,
            to_x: 100,
            to_y: 200,
            to_z: 30,
            x: 10,
            y: 20,
            z: 30,
        }));
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        assert_eq!(world.entities[&1].dest, Some(Vec3::new(100.0, 200.0, 30.0)));
        world.apply_world(&WorldEvent::CharInfo(Box::new(char_info(2))));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: 2,
            to_x: 7,
            to_y: 8,
            to_z: 9,
            x: 1,
            y: 2,
            z: 3,
        }));
        world.apply_world(&WorldEvent::CharInfo(Box::new(char_info(2))));
        assert_eq!(world.entities[&2].dest, Some(Vec3::new(7.0, 8.0, 9.0)));
    }

    #[test]
    fn a_second_user_info_updates_me_and_keeps_movement_state() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: 268481001,
            to_x: 100,
            to_y: 200,
            to_z: 30,
            x: 83400,
            y: 148000,
            z: -3400,
        }));
        world.apply_world(&WorldEvent::ChangeMoveType(ChangeMoveType { object_id: 268481001, running: false }));
        let mut again = head();
        again.x = 83500;
        again.heading = 1;
        again.hp = 10;
        again.run_speed = 200;
        world.apply(&SessionEvent::UserInfo(Box::new(again)));
        let me = world.me.as_ref().unwrap();
        assert_eq!(me.position, Vec3::new(83400.0, 148000.0, -3400.0));
        assert_eq!(me.heading, 16384);
        assert_eq!(me.hp, Some((10, 900)));
        assert_eq!(me.run_speed, 200);
        assert_eq!(me.dest, Some(Vec3::new(100.0, 200.0, 30.0)));
        assert!(!me.running && !me.sitting);
        world.apply_world(&WorldEvent::ChangeWaitType(ChangeWaitType {
            object_id: 268481001,
            kind: 0,
            sitting: true,
            x: 83400,
            y: 148000,
            z: -3400,
        }));
        let me = world.me.as_ref().unwrap();
        assert!(me.sitting && me.dest.is_none());
        assert_eq!(me.position, Vec3::new(83400.0, 148000.0, -3400.0));
        let mut other = head();
        other.object_id = 5;
        world.apply(&SessionEvent::UserInfo(Box::new(other)));
        let me = world.me.as_ref().unwrap();
        assert_eq!(me.object_id, 5);
        assert!(me.dest.is_none() && me.running && !me.sitting);
    }

    #[test]
    fn positional_events_name_the_object_they_place() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        assert_eq!(WorldEvent::NpcInfo(npc(1)).placed_object(), Some(1));
        assert_eq!(WorldEvent::CharInfo(Box::new(char_info(2))).placed_object(), Some(2));
        assert_eq!(
            WorldEvent::MoveToLocation(MoveToLocation { object_id: 3, to_x: 0, to_y: 0, to_z: 0, x: 0, y: 0, z: 0 })
                .placed_object(),
            Some(3)
        );
        assert_eq!(
            WorldEvent::StopMove(StopMove { object_id: 4, x: 0, y: 0, z: 0, heading: 0 }).placed_object(),
            Some(4)
        );
        assert_eq!(
            WorldEvent::ValidateLocation(ValidateLocation { object_id: 5, x: 0, y: 0, z: 0, heading: 0 })
                .placed_object(),
            Some(5)
        );
        assert_eq!(
            WorldEvent::TeleportToLocation(TeleportToLocation { target_id: 6, x: 0, y: 0, z: 0, instant: 0 })
                .placed_object(),
            Some(6)
        );
        assert_eq!(
            WorldEvent::ChangeWaitType(ChangeWaitType { object_id: 7, kind: 0, sitting: true, x: 0, y: 0, z: 0 })
                .placed_object(),
            Some(7)
        );
        assert_eq!(WorldEvent::Revive(Revive { object_id: 8 }).placed_object(), None);
        assert_eq!(world.entity_mut(268481001).map(|e| e.object_id), Some(268481001));
        assert_eq!(world.entity_mut(1).map(|e| e.object_id), Some(1));
        assert!(world.entity_mut(9).is_none());
    }

    #[test]
    fn my_teleport_forgets_the_neighbourhood() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::TeleportToLocation(TeleportToLocation {
            target_id: 1,
            x: 0,
            y: 0,
            z: 0,
            instant: 0,
        }));
        assert_eq!(world.entities.len(), 1);
        world.apply_world(&WorldEvent::TeleportToLocation(TeleportToLocation {
            target_id: 268481001,
            x: 9,
            y: 9,
            z: 9,
            instant: 0,
        }));
        assert!(world.entities.is_empty());
        assert_eq!(world.me.as_ref().unwrap().position, Vec3::new(9.0, 9.0, 9.0));
    }

    #[test]
    fn my_teleport_runs_the_on_teleport_to_location_preamble_and_the_arrival_resets() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        assert!(world.select(1));
        world.me.as_mut().unwrap().chair = Some(5);
        world.apply_world(&WorldEvent::TeleportToLocation(TeleportToLocation { target_id: 1, x: 7, y: 7, z: 7, instant: 0 }));
        assert!(!world.music_stop && !world.music_reset && world.target == Some(1));
        world.apply_world(&WorldEvent::TeleportToLocation(TeleportToLocation { target_id: 268481001, x: 9, y: 9, z: 9, instant: 0 }));
        assert!(world.music_stop && world.music_reset);
        assert_eq!(world.target, None);
        assert_eq!(world.me.as_ref().unwrap().chair, None);
    }

    #[test]
    fn move_to_location_keeps_the_local_position_and_sets_dest() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: 1,
            to_x: 100,
            to_y: 200,
            to_z: 30,
            x: 11,
            y: 21,
            z: 30,
        }));
        let e = &world.entities[&1];
        assert_eq!(e.position, Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(e.dest, Some(Vec3::new(100.0, 200.0, 30.0)));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: 1,
            to_x: 1000,
            to_y: 20,
            to_z: 30,
            x: 500,
            y: 20,
            z: 30,
        }));
        let e = &world.entities[&1];
        assert_eq!(e.position, Vec3::new(500.0, 20.0, 30.0));
        assert_eq!(e.dest, Some(Vec3::new(1000.0, 20.0, 30.0)));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: 2,
            to_x: 1,
            to_y: 2,
            to_z: 3,
            x: 0,
            y: 0,
            z: 0,
        }));
        assert!(!world.entities.contains_key(&2));
    }

    #[test]
    fn my_move_to_location_never_moves_me_and_stands_me_up() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::ChangeWaitType(ChangeWaitType {
            object_id: ME,
            kind: 0,
            sitting: true,
            x: 83400,
            y: 148000,
            z: -3400,
        }));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: ME,
            to_x: 84400,
            to_y: 148000,
            to_z: -3400,
            x: 83000,
            y: 147000,
            z: -3400,
        }));
        let me = world.me.as_ref().unwrap();
        assert_eq!(me.position, Vec3::new(83400.0, 148000.0, -3400.0));
        assert_eq!(me.dest, Some(Vec3::new(84400.0, 148000.0, -3400.0)));
        assert!(!me.sitting);
    }

    #[test]
    fn stop_move_of_another_pawn_stops_it_where_it_is_unless_it_is_over_200_units_off() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: 1,
            to_x: 1010,
            to_y: 20,
            to_z: 30,
            x: 10,
            y: 20,
            z: 30,
        }));
        world.tick(0.5);
        let walked = world.entities[&1].position;
        let facing = world.entities[&1].heading;
        assert!(walked.x > 10.0 && walked.x < 1010.0);
        world.apply_world(&WorldEvent::StopMove(StopMove { object_id: 1, x: 140, y: 20, z: 30, heading: 999 }));
        let e = &world.entities[&1];
        assert_eq!(e.position, walked);
        assert_eq!(e.heading, facing);
        assert!(e.dest.is_none() && e.stop_heading.is_none());
        world.apply_world(&WorldEvent::StopMove(StopMove { object_id: 1, x: 900, y: 20, z: 30, heading: 999 }));
        assert_eq!(world.entities[&1].position, Vec3::new(900.0, 20.0, 30.0));
        assert!(world.entities[&1].dest.is_none());
    }

    #[test]
    fn my_stop_move_stops_me_where_i_am() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: ME,
            to_x: 84400,
            to_y: 148000,
            to_z: -3400,
            x: 83400,
            y: 148000,
            z: -3400,
        }));
        world.tick(1.0);
        world.apply_world(&WorldEvent::StopMove(StopMove {
            object_id: ME,
            x: 83500,
            y: 148000,
            z: -3400,
            heading: 999,
        }));
        let me = world.me.as_ref().unwrap();
        assert!((me.position.x - 83575.5).abs() < 0.01, "{}", me.position.x);
        assert!(me.dest.is_none());
        assert_eq!(me.heading, 0);
    }

    #[test]
    fn validate_location_of_another_pawn_only_repositions_beyond_200_units() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: 1,
            to_x: 100,
            to_y: 200,
            to_z: 30,
            x: 10,
            y: 20,
            z: 30,
        }));
        world.apply_world(&WorldEvent::ValidateLocation(ValidateLocation {
            object_id: 1,
            x: 70,
            y: 90,
            z: 30,
            heading: 1000,
        }));
        let e = &world.entities[&1];
        assert_eq!(e.position, Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(e.heading, 32768);
        assert_eq!(e.dest, Some(Vec3::new(100.0, 200.0, 30.0)));
        world.apply_world(&WorldEvent::ValidateLocation(ValidateLocation {
            object_id: 1,
            x: 700,
            y: 90,
            z: 30,
            heading: 1000,
        }));
        let e = &world.entities[&1];
        assert_eq!(e.position, Vec3::new(700.0, 90.0, 30.0));
        assert_eq!(e.dest, Some(Vec3::new(100.0, 200.0, 30.0)));
        world.apply_world(&WorldEvent::TeleportToLocation(TeleportToLocation {
            target_id: 1,
            x: -1,
            y: -2,
            z: -3,
            instant: 0,
        }));
        let e = &world.entities[&1];
        assert_eq!(e.position, Vec3::new(-1.0, -2.0, -3.0));
        assert!(e.dest.is_none());
    }

    #[test]
    fn a_non_instant_teleport_of_me_waits_for_the_black_fade_and_lands_ch_plus_10_above() {
        let mut world = World { teleport_fades: true, ..World::default() };
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        let me = world.me.as_ref().unwrap();
        let (id, start, height) = (me.object_id, me.position, me.collision_height);
        world.apply_world(&WorldEvent::TeleportToLocation(TeleportToLocation { target_id: id, x: 500, y: 600, z: -700, instant: 0 }));
        assert_eq!(world.me.as_ref().unwrap().position, start);
        assert!(world.teleport_announce && world.observer.fade_out_pending);
        assert!(world.observer_fade_done());
        assert_eq!(world.me.as_ref().unwrap().position, Vec3::new(500.0, 600.0, -700.0 + height + 10.0));
        let mut instant = World { teleport_fades: true, ..World::default() };
        instant.apply(&SessionEvent::UserInfo(Box::new(head())));
        instant.apply_world(&WorldEvent::TeleportToLocation(TeleportToLocation { target_id: id, x: 1, y: 2, z: 3, instant: 1 }));
        assert!(!instant.teleport_announce);
        assert_eq!(instant.me.as_ref().unwrap().position, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn my_validate_location_teleports_me_only_while_standing() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: ME,
            to_x: 84400,
            to_y: 148000,
            to_z: -3400,
            x: 83400,
            y: 148000,
            z: -3400,
        }));
        world.apply_world(&WorldEvent::ValidateLocation(ValidateLocation {
            object_id: ME,
            x: 83410,
            y: 148010,
            z: -3400,
            heading: 5,
        }));
        let me = world.me.as_ref().unwrap();
        assert_eq!(me.position, Vec3::new(83410.0, 148010.0, -3400.0));
        assert_eq!(me.heading, 5);
        assert_eq!(me.dest, Some(Vec3::new(84400.0, 148000.0, -3400.0)));
        world.apply_world(&WorldEvent::ChangeWaitType(ChangeWaitType {
            object_id: ME,
            kind: 0,
            sitting: true,
            x: 83410,
            y: 148010,
            z: -3400,
        }));
        world.apply_world(&WorldEvent::ValidateLocation(ValidateLocation {
            object_id: ME,
            x: 83000,
            y: 148000,
            z: -3400,
            heading: 6,
        }));
        let me = world.me.as_ref().unwrap();
        assert_eq!(me.position, Vec3::new(83410.0, 148010.0, -3400.0));
        assert_eq!(me.heading, 5);
    }

    #[test]
    fn delete_object_removes() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::DeleteObject(DeleteObject { object_id: 1 }));
        assert!(world.entities.is_empty());
        world.apply_world(&WorldEvent::DeleteObject(DeleteObject { object_id: 1 }));
    }

    #[test]
    fn target_follows_the_selected_entity_until_it_leaves() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        assert!(world.targeted().is_none());
        assert!(!world.select(99));
        assert!(world.select(1));
        assert_eq!(world.targeted().map(|e| e.name.as_str()), Some("Gremlin"));
        world.apply_world(&WorldEvent::DeleteObject(DeleteObject { object_id: 7 }));
        assert_eq!(world.target, Some(1));
        world.apply_world(&WorldEvent::DeleteObject(DeleteObject { object_id: 1 }));
        assert_eq!(world.target, None);
        assert!(world.select(268481001));
        assert_eq!(world.targeted().map(|e| e.object_id), Some(268481001));
        world.apply_world(&WorldEvent::NpcInfo(npc(2)));
        world.select(2);
        world.apply_world(&WorldEvent::TeleportToLocation(TeleportToLocation {
            target_id: 268481001,
            x: 9,
            y: 9,
            z: 9,
            instant: 0,
        }));
        assert_eq!(world.target, None);
    }

    #[test]
    fn status_update_fills_hp_and_mp() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        assert_eq!(world.entities[&1].hp, None);
        world.apply_world(&WorldEvent::StatusUpdate(StatusUpdate {
            object_id: 1,
            attributes: vec![
                (ATTRIBUTE_CURRENT_HP, 400),
                (ATTRIBUTE_MAX_HP, 900),
                (ATTRIBUTE_MAX_MP, 30),
                (ATTRIBUTE_CURRENT_MP, 10),
                (ATTRIBUTE_CURRENT_CP, 5),
            ],
        }));
        assert_eq!(world.entities[&1].hp, Some((400, 900)));
        assert_eq!(world.entities[&1].mp, Some((10, 30)));
        assert_eq!(world.entities[&1].cp, Some((5, 0)));
    }

    #[test]
    fn posture_death_and_revive() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::ChangeMoveType(ChangeMoveType { object_id: 1, running: false }));
        assert!(!world.entities[&1].running);
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: 1,
            to_x: 100,
            to_y: 200,
            to_z: 30,
            x: 10,
            y: 20,
            z: 30,
        }));
        world.apply_world(&WorldEvent::ChangeWaitType(ChangeWaitType {
            object_id: 1,
            kind: 0,
            sitting: true,
            x: 5,
            y: 6,
            z: 7,
        }));
        assert!(world.entities[&1].sitting);
        assert!(world.entities[&1].dest.is_none());
        assert_eq!(world.entities[&1].position, Vec3::new(10.0, 20.0, 30.0));
        world.apply_world(&WorldEvent::ChangeWaitType(ChangeWaitType {
            object_id: 1,
            kind: 0,
            sitting: true,
            x: 500,
            y: 6,
            z: 7,
        }));
        assert_eq!(world.entities[&1].position, Vec3::new(500.0, 6.0, 7.0));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: 1,
            to_x: 100,
            to_y: 200,
            to_z: 30,
            x: 500,
            y: 6,
            z: 7,
        }));
        assert!(!world.entities[&1].sitting);
        world.apply_world(&WorldEvent::Die(Die {
            object_id: 1,
            to_village: false,
            to_hideout: false,
            to_castle: false,
            to_siege_hq: false,
            sweepable: true,
            to_fixed_spot: false,
        }));
        let e = &world.entities[&1];
        assert!(e.dead && e.dest.is_none());
        assert!(world.death.is_none());
        assert_eq!(world.effects.take_sweepable_deaths(), [1]);
        world.apply_world(&WorldEvent::Revive(Revive { object_id: 1 }));
        assert!(!world.entities[&1].dead);
    }

    #[test]
    fn my_own_death_keeps_the_restart_options_until_the_revive() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        let me = world.me.as_ref().unwrap().object_id;
        let die = Die {
            object_id: me,
            to_village: true,
            to_hideout: false,
            to_castle: true,
            to_siege_hq: false,
            sweepable: false,
            to_fixed_spot: false,
        };
        world.apply_world(&WorldEvent::Die(die));
        assert_eq!(world.death, Some(die));
        assert!(world.me.as_ref().unwrap().dead);
        world.apply_world(&WorldEvent::Revive(Revive { object_id: me }));
        assert!(world.death.is_none());
        assert!(!world.me.as_ref().unwrap().dead);
    }

    #[test]
    fn me_follows_events_with_my_object_id_and_is_never_in_entities() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        let me_id = 268481001;
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: me_id,
            to_x: 100,
            to_y: 200,
            to_z: 30,
            x: 83400,
            y: 148000,
            z: -3400,
        }));
        assert_eq!(world.me.as_ref().unwrap().dest, Some(Vec3::new(100.0, 200.0, 30.0)));
        world.apply_world(&WorldEvent::ValidateLocation(ValidateLocation {
            object_id: me_id,
            x: 83410,
            y: 148010,
            z: -3400,
            heading: 5,
        }));
        assert_eq!(world.me.as_ref().unwrap().position, Vec3::new(83410.0, 148010.0, -3400.0));
        world.apply_world(&WorldEvent::StatusUpdate(StatusUpdate {
            object_id: me_id,
            attributes: vec![(ATTRIBUTE_CURRENT_HP, 1)],
        }));
        assert_eq!(world.me.as_ref().unwrap().hp, Some((1, 900)));
        world.apply_world(&WorldEvent::ChangeMoveType(ChangeMoveType { object_id: me_id, running: false }));
        assert!(!world.me.as_ref().unwrap().running);
        world.apply_world(&WorldEvent::Die(Die {
            object_id: me_id,
            to_village: true,
            to_hideout: false,
            to_castle: false,
            to_siege_hq: false,
            sweepable: false,
            to_fixed_spot: false,
        }));
        assert!(world.me.as_ref().unwrap().dead);
        world.apply_world(&WorldEvent::TeleportToLocation(TeleportToLocation {
            target_id: me_id,
            x: 1,
            y: 2,
            z: 3,
            instant: 1,
        }));
        assert_eq!(world.me.as_ref().unwrap().position, Vec3::new(1.0, 2.0, 3.0));
        world.apply_world(&WorldEvent::DeleteObject(DeleteObject { object_id: me_id }));
        assert!(world.me.is_some());
        assert!(world.entities.is_empty());
    }

    #[test]
    fn tick_moves_entities_with_dest_at_their_speed() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: 1,
            to_x: 1010,
            to_y: 20,
            to_z: 30,
            x: 10,
            y: 20,
            z: 30,
        }));
        world.tick(1.0);
        let e = &world.entities[&1];
        assert!((e.position.x - 160.0).abs() < 0.01, "{}", e.position.x);
        assert_eq!(e.position.y, 20.0);
        assert!(e.dest.is_some());
    }

    #[test]
    fn nearest_lists_entities_by_distance_from_me() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        let mut far = npc(1);
        far.x = 83400 + 1000;
        far.y = 148000;
        far.z = -3400;
        let mut near = npc(2);
        near.x = 83400 + 10;
        near.y = 148000;
        near.z = -3400;
        world.apply_world(&WorldEvent::NpcInfo(far));
        world.apply_world(&WorldEvent::NpcInfo(near));
        let list: Vec<(i32, f32)> = world.nearest(10).into_iter().map(|(e, d)| (e.object_id, d)).collect();
        assert_eq!(list, vec![(2, 10.0), (1, 1000.0)]);
        assert_eq!(world.nearest(1).len(), 1);
    }

    #[test]
    fn client_set_time_fills_the_clock() {
        let mut world = World::default();
        assert!(world.clock.is_none());
        world.apply(&SessionEvent::Time(ClientSetTime { minutes: 360, ratio: 6 }));
        let clock = world.clock.expect("clock deveria estar preenchido");
        assert_eq!(clock.minutes_at_sync, 360);
        assert_eq!(clock.ratio, 6);
    }

    const ME: i32 = 268481001;

    #[test]
    fn my_target_selected_and_unselected_drive_world_target() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        assert!(world.target.is_none());
        world.apply_world(&WorldEvent::TargetSelected(TargetSelected {
            object_id: ME,
            target_id: 1,
            x: 0,
            y: 0,
            z: 0,
        }));
        assert_eq!(world.target, Some(1));
        assert_eq!(world.target_entity().map(|e| e.name.as_str()), Some("Gremlin"));
        assert!(world.me.as_ref().unwrap().target.is_none());
        world.apply_world(&WorldEvent::TargetUnselected(TargetUnselected { object_id: ME, x: 0, y: 0, z: 0 }));
        assert!(world.target.is_none());
        assert!(world.target_entity().is_none());
    }

    #[test]
    fn other_targets_live_on_the_entity_not_on_the_world() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::CharInfo(Box::new(char_info(2))));
        world.apply_world(&WorldEvent::TargetSelected(TargetSelected { object_id: 2, target_id: 1, x: 0, y: 0, z: 0 }));
        assert!(world.target.is_none());
        assert_eq!(world.entities[&2].target, Some(1));
        world.apply_world(&WorldEvent::TargetSelected(TargetSelected {
            object_id: 2,
            target_id: ME,
            x: 0,
            y: 0,
            z: 0,
        }));
        assert_eq!(world.entities[&2].target, Some(ME));
        world.apply_world(&WorldEvent::CharInfo(Box::new(char_info(2))));
        assert_eq!(world.entities[&2].target, Some(ME));
        world.apply_world(&WorldEvent::TargetUnselected(TargetUnselected { object_id: 2, x: 0, y: 0, z: 0 }));
        assert!(world.entities[&2].target.is_none());
        world.apply_world(&WorldEvent::TargetSelected(TargetSelected {
            object_id: 99,
            target_id: 1,
            x: 0,
            y: 0,
            z: 0,
        }));
        assert!(world.target.is_none() && !world.entities.contains_key(&99));
    }

    #[test]
    fn move_to_pawn_walks_towards_the_target_and_stops_the_distance_before_it() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        let mut lector = npc(1);
        lector.x = 83400 + 100;
        lector.y = 148000;
        lector.z = -3400;
        world.apply_world(&WorldEvent::NpcInfo(lector));
        world.apply_world(&WorldEvent::MoveToPawn(MoveToPawn {
            object_id: ME,
            target_id: 1,
            distance: 40,
            x: 83400,
            y: 148000,
            z: -3400,
        }));
        let me = world.me.as_ref().unwrap();
        assert_eq!(me.position, Vec3::new(83400.0, 148000.0, -3400.0));
        assert_eq!(me.dest, Some(Vec3::new(83500.0, 148000.0, -3400.0)));
        assert_eq!(me.follow, Some(Follow { target_id: 1, distance: 40.0 }));
        world.apply_world(&WorldEvent::MoveToPawn(MoveToPawn {
            object_id: ME,
            target_id: 1,
            distance: 200,
            x: 83400,
            y: 148000,
            z: -3400,
        }));
        assert!(world.me.as_ref().unwrap().dest.is_none());
    }

    #[test]
    fn move_to_pawn_of_others_and_of_unknown_targets() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::CharInfo(Box::new(char_info(2))));
        world.apply_world(&WorldEvent::MoveToPawn(MoveToPawn {
            object_id: 2,
            target_id: ME,
            distance: 0,
            x: 83400 + 300,
            y: 148000,
            z: -3400,
        }));
        let walker = &world.entities[&2];
        assert_eq!(walker.position, Vec3::new(83700.0, 148000.0, -3400.0));
        assert_eq!(walker.dest, Some(Vec3::new(83400.0, 148000.0, -3400.0)));
        assert!(walker.follow.is_some());
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: 1,
            to_x: 500,
            to_y: 0,
            to_z: 30,
            x: 10,
            y: 20,
            z: 30,
        }));
        world.apply_world(&WorldEvent::MoveToPawn(MoveToPawn {
            object_id: 1,
            target_id: 777,
            distance: 10,
            x: 11,
            y: 21,
            z: 30,
        }));
        let npc = &world.entities[&1];
        assert_eq!(npc.position, Vec3::new(10.0, 20.0, 30.0));
        assert!(npc.dest.is_none());
        world.apply_world(&WorldEvent::MoveToPawn(MoveToPawn {
            object_id: 555,
            target_id: 1,
            distance: 10,
            x: 0,
            y: 0,
            z: 0,
        }));
        assert!(!world.entities.contains_key(&555));
    }

    #[test]
    fn relation_changed_updates_relation_karma_and_pvp_flag() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::CharInfo(Box::new(char_info(2))));
        world.apply_world(&WorldEvent::RelationChanged(RelationChanged {
            object_id: 2,
            relation: 0x8000,
            auto_attack: 1,
            karma: 99,
            pvp_flag: 1,
        }));
        let p = &world.entities[&2];
        assert_eq!((p.relation, p.karma, p.pvp_flag), (0x8000, 99, 1));
        world.apply_world(&WorldEvent::CharInfo(Box::new(char_info(2))));
        assert_eq!(world.entities[&2].relation, 0x8000);
        assert!(world.entities[&2].auto_attack);
        world.apply_world(&WorldEvent::RelationChanged(RelationChanged {
            object_id: ME,
            relation: 2,
            auto_attack: 0,
            karma: 5,
            pvp_flag: 0,
        }));
        let me = world.me.as_ref().unwrap();
        assert_eq!((me.relation, me.karma, me.pvp_flag), (2, 5, 0));
        assert!(!me.auto_attack);
    }

    pub(super) fn cast(caster_id: i32, target_id: i32) -> MagicSkillUse {
        MagicSkillUse {
            caster_id,
            target_id,
            skill_id: 1204,
            skill_level: 2,
            hit_time: 4000,
            reuse_delay: 10000,
            x: 7,
            y: 8,
            z: 9,
            extra: Vec::new(),
            positions: Vec::new(),
        }
    }

    #[test]
    fn magic_skill_use_starts_a_cast_at_the_caster_position_and_launch_ends_it() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: 1,
            to_x: 500,
            to_y: 0,
            to_z: 30,
            x: 10,
            y: 20,
            z: 30,
        }));
        let before = Instant::now();
        world.apply_world(&WorldEvent::MagicSkillUse(cast(1, ME)));
        let e = &world.entities[&1];
        let casting = e.casting.expect("casting");
        assert_eq!((casting.skill_id, casting.hit_time, casting.target_id), (1204, 4000, ME));
        assert!(casting.started >= before);
        assert_eq!(e.position, Vec3::new(10.0, 20.0, 30.0));
        assert!(e.dest.is_some(), "0x10480220 só chama AdjustPawnLocation; a parada vem do lançamento");
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        assert!(world.entities[&1].casting.is_some());
        world.apply_world(&WorldEvent::MagicSkillLaunched(MagicSkillLaunched {
            caster_id: 1,
            skill_id: 1204,
            skill_level: 2,
            targets: vec![ME],
        }));
        assert!(world.entities[&1].casting.is_none());
        assert_eq!(world.entities[&1].anim.pending.last(), Some(&Trigger::MagicSkillLaunched { skill_id: 1204, targets: vec![ME] }));
        world.apply_world(&WorldEvent::MagicSkillLaunched(MagicSkillLaunched { caster_id: 1, skill_id: 1204, skill_level: 2, targets: vec![404, 405] }));
        assert_eq!(world.entities[&1].anim.pending.iter().filter(|trigger| matches!(trigger, Trigger::MagicSkillLaunched { .. })).count(), 1);
        world.apply_world(&WorldEvent::MagicSkillUse(cast(ME, 1)));
        assert_eq!(world.me.as_ref().unwrap().casting.map(|c| c.skill_id), Some(1204));
        world.apply_world(&WorldEvent::MagicSkillCanceled(MagicSkillCanceled { object_id: ME }));
        assert!(world.me.as_ref().unwrap().casting.is_none());
        world.apply_world(&WorldEvent::MagicSkillUse(cast(404, 1)));
        assert!(!world.entities.contains_key(&404));
    }

    #[test]
    fn magic_skill_use_needs_a_known_target_and_a_living_caster() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::MagicSkillUse(cast(1, 404)));
        let e = &world.entities[&1];
        assert!(e.casting.is_none() && world.last_cast.is_none());
        assert_eq!(e.position, Vec3::new(10.0, 20.0, 30.0), "sem alvo o AdjustPawnLocation nem roda");
        world.entities.get_mut(&1).unwrap().dead = true;
        world.apply_world(&WorldEvent::MagicSkillUse(cast(1, ME)));
        assert!(world.entities[&1].casting.is_none() && world.last_cast.is_none());
        world.entities.get_mut(&1).unwrap().dead = false;
        world.apply_world(&WorldEvent::MagicSkillUse(cast(1, ME)));
        let accepted = world.cast_accepted(&cast(1, ME)).copied().unwrap();
        assert_eq!((accepted.caster_is_me, accepted.variant, accepted.reuse_delay), (false, CastVariant::Pawn, 10000));
        world.apply_world(&WorldEvent::MagicSkillUse(cast(ME, 1)));
        assert!(world.cast_accepted(&cast(ME, 1)).is_some_and(|cast| cast.caster_is_me));
        assert!(world.cast_accepted(&cast(1, ME)).is_none(), "só vale o último");
    }

    #[test]
    fn a_negative_hit_time_adjusts_the_position_and_stops_there() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::MagicSkillUse(MagicSkillUse { hit_time: -1, x: 900, ..cast(1, ME) }));
        let e = &world.entities[&1];
        assert_eq!(e.position.x, 900.0);
        assert!(e.casting.is_none() && world.last_cast.is_none());
        assert!(!e.anim.pending.iter().any(|trigger| matches!(trigger, Trigger::MagicSkillUse { .. })));
    }

    #[test]
    fn the_caster_of_the_local_player_is_never_repositioned() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        let before = world.me.as_ref().unwrap().position;
        world.apply_world(&WorldEvent::MagicSkillUse(MagicSkillUse { x: before.x as i32 + 5000, ..cast(ME, 1) }));
        assert_eq!(world.me.as_ref().unwrap().position, before);
        assert!(world.cast_accepted(&MagicSkillUse { x: before.x as i32 + 5000, ..cast(ME, 1) }).is_some());
    }

    #[test]
    fn a_door_target_takes_the_mover_variant_and_extras_wait_for_the_cast_start() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::DoorInfo(DoorInfo { object_id: 100, door_id: 24190001 }));
        let event = MagicSkillUse { extra: vec![0, 0xfffe], positions: vec![[1, 2, 3]], ..cast(1, 100) };
        world.apply_world(&WorldEvent::MagicSkillUse(event.clone()));
        assert_eq!(world.cast_accepted(&event).unwrap().variant, CastVariant::Mover);
        let e = &world.entities[&1];
        assert!(matches!(e.anim.pending.last(), Some(Trigger::MagicSkillUseOnMover { target: 100, .. })));
        let pending = e.magic.pending.clone().unwrap();
        assert_eq!(pending.effect_actor, [0, -2]);
        assert_eq!(pending.ground_targets, [Vec3::new(1.0, 2.0, 3.0)]);
        assert!(e.magic.current.effect_actor.is_empty());
        let mut magic = e.magic.clone();
        magic.set_magic_info();
        assert!(magic.current.effect_list_head_empty() && magic.pending.is_none());
        magic.set_magic_info();
        assert_eq!(magic.current, MagicInfo::default());
    }

    #[test]
    fn the_frame_rate_cull_drops_a_far_stranger_before_anything_happens() {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        let me = world.me.as_ref().unwrap().position;
        let place = |world: &mut World, distance: f32| world.entities.get_mut(&1).unwrap().position = me + Vec3::new(distance, 0.0, 0.0);
        world.keep_min_frame_rate = true;
        world.pawn_clipping = 0.7;
        place(&mut world, 100.0);
        world.apply_world(&WorldEvent::MagicSkillUse(cast(1, 1)));
        assert!(world.last_cast.is_none(), "fora da tela e o alvo não é o jogador");
        world.apply_world(&WorldEvent::MagicSkillUse(cast(1, ME)));
        assert!(world.last_cast.is_some());
        world.rendered.insert(1);
        place(&mut world, 100.0);
        world.apply_world(&WorldEvent::MagicSkillUse(cast(1, 1)));
        assert!(world.last_cast.is_some());
        place(&mut world, 2000.0);
        world.apply_world(&WorldEvent::MagicSkillUse(cast(1, 1)));
        assert!(world.last_cast.is_none(), "além de 0,7 × 2048 unidades");
    }

    #[test]
    fn spawn_item_adds_an_item_entity_that_delete_object_removes() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::SpawnItem(SpawnItem {
            object_id: 50,
            item_id: 57,
            x: 1,
            y: 2,
            z: 3,
            stackable: true,
            count: 250,
            heading: 16384,
        }));
        let item = &world.entities[&50];
        assert_eq!(item.kind, EntityKind::Item { item_id: 57, count: 250, stackable: true });
        assert_eq!(item.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(item.heading, 16384);
        assert!(item.name.is_empty() && item.dest.is_none() && !item.running);
        assert_eq!(item.speed(), 0.0);
        world.apply_world(&WorldEvent::SpawnItem(SpawnItem {
            object_id: 50,
            item_id: 57,
            x: 1,
            y: 2,
            z: 3,
            stackable: true,
            count: 300,
            heading: 16384,
        }));
        assert_eq!(world.entities.len(), 1);
        assert!(matches!(world.entities[&50].kind, EntityKind::Item { count: 300, .. }));
        world.apply_world(&WorldEvent::DeleteObject(DeleteObject { object_id: 50 }));
        assert!(world.entities.is_empty());
    }

    #[test]
    fn etc_status_buffs_and_pledge_are_kept_on_the_world() {
        let mut world = World::default();
        assert!(world.etc_status.is_none() && world.buffs.is_empty() && world.pledge.is_none());
        world.apply_world(&WorldEvent::EtcStatusUpdate(EtcStatusUpdate { values: [1, 2, 3, 4, 5] }));
        assert_eq!(world.etc_status.map(|e| e.values), Some([1, 2, 3, 4, 5]));
        let buffs =
            vec![Buff { skill_id: 1204, level: 2, duration: 1200 }, Buff { skill_id: 1040, level: 3, duration: -1 }];
        assert!(world.buffs_at.is_none());
        world.apply_world(&WorldEvent::MagicEffectIcons(MagicEffectIcons { effects: buffs.clone() }));
        assert_eq!(world.buffs, buffs);
        assert!(world.buffs_at.is_some());
        world.apply_world(&WorldEvent::MagicEffectIcons(MagicEffectIcons { effects: Vec::new() }));
        assert!(world.buffs.is_empty());
        let pledge = PledgeStatusChanged { leader_id: 1, clan_id: 2, crest_id: 3, ally_id: 4, ally_crest_id: 5 };
        world.apply_world(&WorldEvent::PledgeStatusChanged(pledge));
        assert_eq!(world.pledge, Some(pledge));
    }

    fn macro_packet(revision: i32, id: i32, name: &str) -> SendMacroList {
        SendMacroList {
            flag: 0,
            revision,
            total_count: 2,
            macros: vec![Macro {
                id,
                name: name.into(),
                description: String::new(),
                acronym: String::new(),
                icon: 0,
                commands: Vec::new(),
            }],
        }
    }

    #[test]
    fn send_macro_list_accumulates_one_macro_per_packet_and_keeps_the_others_on_a_new_revision() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::SendMacroList(macro_packet(1, 10, "A")));
        world.apply_world(&WorldEvent::SendMacroList(macro_packet(1, 11, "B")));
        assert_eq!((world.macros.revision, world.macros.total_count), (1, 2));
        assert_eq!(world.macros.macros.iter().map(|m| m.id).collect::<Vec<_>>(), vec![10, 11]);
        world.apply_world(&WorldEvent::SendMacroList(macro_packet(1, 10, "A2")));
        assert_eq!(world.macros.macros.len(), 2);
        assert_eq!(world.macros.macros[0].name, "A2");
        // revisão nova com um macro só é uma alteração: os outros continuam na lista
        world.apply_world(&WorldEvent::SendMacroList(macro_packet(2, 11, "B2")));
        assert_eq!(world.macros.macros.iter().map(|m| m.id).collect::<Vec<_>>(), vec![10, 11]);
        assert_eq!(world.macros.macros[1].name, "B2");
        assert_eq!(world.macros.revision, 2);
        // só o total zerado esvazia
        world.apply_world(&WorldEvent::SendMacroList(SendMacroList {
            flag: 0,
            revision: 3,
            total_count: 0,
            macros: Vec::new(),
        }));
        assert!(world.macros.macros.is_empty());
        assert_eq!((world.macros.revision, world.macros.total_count), (3, 0));
    }

    #[test]
    fn shop_preview_info_dresses_the_players_pawn_with_the_rules_of_1047f010() {
        use l2_net::game::{EQUIP_CHEST, EQUIP_FEET, EQUIP_GLOVES, EQUIP_LHAND, EQUIP_LRHAND, EQUIP_RHAND};
        let body_part = |id: i32| match id {
            60 => Some(l2_dat::armorgrp::body_part::ONEPIECE),
            6408 => Some(l2_dat::armorgrp::body_part::ALLDRESS),
            _ => Some(l2_dat::armorgrp::body_part::CHEST),
        };
        let mut world = World { me: Some(me_with_equipment([0; 11], 7)), ..Default::default() };
        let mut slots = [0; 16];
        slots[7] = 100;
        world.apply_shop_preview_info(&ShopPreviewInfo { slots }, body_part);
        let (equipped, enchant) = player_equipment(&world);
        assert_eq!((equipped[EQUIP_RHAND], enchant), (100, 0));

        let mut start = [0; 11];
        start[EQUIP_CHEST] = 6408;
        start[EQUIP_GLOVES] = 5;
        start[EQUIP_FEET] = 6;
        let mut world = World { me: Some(me_with_equipment(start, 0)), ..Default::default() };
        let mut slots = [0; 16];
        slots[9] = 200;
        world.apply_shop_preview_info(&ShopPreviewInfo { slots }, body_part);
        let (equipped, _) = player_equipment(&world);
        assert_eq!((equipped[EQUIP_CHEST], equipped[EQUIP_GLOVES], equipped[EQUIP_FEET]), (0, 200, 0));

        let mut start = [0; 11];
        start[EQUIP_LRHAND] = 9;
        start[EQUIP_LHAND] = 3;
        let mut world = World { me: Some(me_with_equipment(start, 3)), ..Default::default() };
        let mut slots = [0; 16];
        slots[8] = 300;
        world.apply_shop_preview_info(&ShopPreviewInfo { slots }, body_part);
        let (equipped, enchant) = player_equipment(&world);
        assert_eq!((equipped[EQUIP_LRHAND], equipped[EQUIP_RHAND], equipped[EQUIP_LHAND], enchant), (0, 0, 300, 3));

        let mut slots = [0; 16];
        slots[14] = 400;
        slots[0] = 9;
        slots[1] = 9;
        world.apply_shop_preview_info(&ShopPreviewInfo { slots }, body_part);
        let (equipped, enchant) = player_equipment(&world);
        assert_eq!((equipped[EQUIP_LRHAND], equipped[EQUIP_LHAND], equipped[0], enchant), (400, 0, 0, 0));

        let mut start = [0; 11];
        start[EQUIP_CHEST] = 60;
        let mut world = World { me: Some(me_with_equipment(start, 0)), ..Default::default() };
        let mut slots = [0; 16];
        slots[11] = 500;
        world.apply_shop_preview_info(&ShopPreviewInfo { slots }, body_part);
        assert_eq!(player_equipment(&world).0[EQUIP_CHEST], 0);
    }

    fn me_with_equipment(equipped: [i32; 11], enchant_effect: u8) -> Entity {
        let mut me = Entity::from_user_info(&head());
        me.kind = EntityKind::Player { class_id: 0, race: 0, sex: 0, equipped, hair_style: 0, hair_color: 0, face: 0, enchant_effect, hero: false };
        me
    }

    fn player_equipment(world: &World) -> ([i32; 11], u8) {
        match &world.me.as_ref().unwrap().kind {
            EntityKind::Player { equipped, enchant_effect, .. } => (*equipped, *enchant_effect),
            _ => panic!("not a player"),
        }
    }
}

#[cfg(test)]
mod frente_g_tests {
    use super::*;
    use l2_net::game::packets::{
        ATTRIBUTE_CAST_SPD, ATTRIBUTE_CURRENT_CP, ATTRIBUTE_MAX_CP, ATTRIBUTE_P_ATK, ATTRIBUTE_STR, BODY_PART_R_HAND,
        ITEM_ADDED, PAPERDOLL_R_HAND, SHORTCUT_ITEM,
    };
    use l2_net::game::{
        ActionFailed, EquipUpdate, InventoryItem, InventoryUpdate, ItemChange, ItemList, MyTargetSelected,
        NpcHtmlMessage, PaperdollSlot, ShortCutDelete, ShortCutInit, Shortcut, ShortcutKind, SkillCoolTime, SkillEntry,
        SkillList, SkillReuse, StatusUpdate, UserInfoHead, UserStats,
    };

    fn head() -> UserInfoHead {
        let mut paperdoll = [PaperdollSlot::default(); 16];
        paperdoll[PAPERDOLL_R_HAND] = PaperdollSlot { object_id: 2, item_id: 2369 };
        UserInfoHead {
            x: 0,
            abnormal_effect: 0,
            y: 0,
            z: 0,
            heading: 0,
            object_id: 1,
            name: "Wender".into(),
            race: 0,
            sex: 1,
            class_id: 109,
            level: 52,
            exp: 12345,
            max_hp: 900,
            hp: 400,
            max_mp: 300,
            mp: 200,
            run_speed: 117,
            walk_speed: 80,
            speed_multiplier: 1.5,
            attack_speed_multiplier: 1.0,
            pvp_flag: 0,
            karma: 0,
            collision_radius: 9.0,
            collision_height: 23.0,
            hair_style: 2,
            hair_color: 1,
            face: 3,
            builder_level: 0,
            title: "Titulo".into(),
            max_cp: 500,
            cp: 450,
            stats: UserStats {
                str_: 40,
                dex: 30,
                con: 43,
                int_: 21,
                wit: 11,
                men: 25,
                sp: 777,
                load: 1500,
                max_load: 62_000,
                attack_range: 40,
                p_atk: 150,
                atk_spd: 300,
                p_def: 120,
                evasion: 30,
                accuracy: 33,
                critical: 44,
                m_atk: 60,
                cast_spd: 213,
                m_def: 80,
            },
            paperdoll,
            inventory_limit: 80,
            name_color: 0xffffff,
            social: Default::default(),
            enchant_effect: 0,
        }
    }

    fn item(object_id: i32, item_id: i32, count: i32, equipped: bool, body_part: i32) -> InventoryItem {
        InventoryItem {
            type1: 0,
            object_id,
            item_id,
            count,
            type2: 0,
            custom_type1: 0,
            equipped,
            body_part,
            enchant: 0,
            custom_type2: 0,
        }
    }

    fn entered() -> World {
        let mut world = World::default();
        world.apply(&SessionEvent::UserInfo(Box::new(head())));
        world
    }

    #[test]
    fn user_info_fills_stats_paperdoll_and_inventory_limit() {
        let world = entered();
        assert_eq!(world.stats, head().stats);
        assert_eq!(world.paperdoll[PAPERDOLL_R_HAND].item_id, 2369);
        assert_eq!(world.inventory_limit, 80);
    }

    #[test]
    fn my_status_update_changes_stats_max_cp_level_and_exp_but_not_others() {
        let mut world = entered();
        world.apply_world(&WorldEvent::StatusUpdate(StatusUpdate {
            object_id: 1,
            attributes: vec![
                (ATTRIBUTE_STR, 41),
                (ATTRIBUTE_P_ATK, 151),
                (ATTRIBUTE_CAST_SPD, 333),
                (ATTRIBUTE_MAX_CP, 600),
                (ATTRIBUTE_CURRENT_CP, 100),
                (ATTRIBUTE_LEVEL, 53),
                (ATTRIBUTE_EXP, 99),
            ],
        }));
        assert_eq!((world.stats.str_, world.stats.p_atk, world.stats.cast_spd), (41, 151, 333));
        let me = world.me.as_ref().expect("me");
        assert_eq!(me.cp, Some((100, 600)));
        assert_eq!((me.level, me.exp), (53, 99));
        world.apply_world(&WorldEvent::StatusUpdate(StatusUpdate {
            object_id: 7,
            attributes: vec![(ATTRIBUTE_STR, 1)],
        }));
        assert_eq!(world.stats.str_, 41);
    }

    #[test]
    fn item_list_inventory_update_and_equip_update_reach_the_inventory() {
        let mut world = entered();
        world.apply_world(&WorldEvent::ItemList(ItemList {
            show_window: true,
            items: vec![item(10, 57, 300, false, 0), item(2, 2369, 1, true, BODY_PART_R_HAND)],
        }));
        assert_eq!(world.inventory.adena(), 300);
        world.apply_world(&WorldEvent::InventoryUpdate(InventoryUpdate {
            changes: vec![ItemChange { change: ITEM_ADDED, item: item(11, 1060, 2, false, 0) }],
        }));
        assert_eq!(world.inventory.items.len(), 3);
        world.apply_world(&WorldEvent::EquipUpdate(EquipUpdate { change: 0, object_id: 2, body_part: 7 }));
        assert_eq!(world.inventory.equipped_in(BODY_PART_R_HAND), None);
    }

    #[test]
    fn shortcuts_init_register_and_delete() {
        let mut world = entered();
        let sc = |slot, object_id| Shortcut { slot, kind: ShortcutKind::Item { object_id }, character_type: 1 };
        world.apply_world(&WorldEvent::ShortCutInit(ShortCutInit { shortcuts: vec![sc(0, 10), sc(1, 11)] }));
        assert_eq!(world.shortcuts.len(), 2);
        world.apply_world(&WorldEvent::ShortCutRegister(sc(1, 12)));
        world.apply_world(&WorldEvent::ShortCutRegister(sc(13, 13)));
        assert_eq!(world.shortcuts.len(), 3);
        assert_eq!(world.shortcut(1), Some(&sc(1, 12)));
        world.apply_world(&WorldEvent::ShortCutDelete(ShortCutDelete { slot: 0 }));
        assert_eq!(world.shortcut(0), None);
        assert_eq!(world.shortcuts.len(), 2);
        assert_eq!(SHORTCUT_ITEM, 1);
    }

    #[test]
    fn skills_cooldowns_npc_html_and_my_target() {
        let mut world = entered();
        world.apply_world(&WorldEvent::SkillList(SkillList {
            skills: vec![SkillEntry { passive: false, level: 3, id: 1177 }],
        }));
        assert_eq!(world.skills.len(), 1);
        world.apply_world(&WorldEvent::SkillCoolTime(SkillCoolTime {
            entries: vec![SkillReuse { skill_id: 1177, level: 3, reuse: 20, remaining: 15 }],
        }));
        assert_eq!(world.cooldowns[0].remaining, 15);
        world.apply_world(&WorldEvent::NpcHtmlMessage(NpcHtmlMessage {
            npc_object_id: 5,
            html: "<html></html>".into(),
            item_id: 0,
        }));
        assert_eq!(world.npc_html.as_ref().map(|h| h.npc_object_id), Some(5));
        world.apply_world(&WorldEvent::NpcInfo(super::tests::npc(7)));
        world.apply_world(&WorldEvent::MyTargetSelected(MyTargetSelected { object_id: 7, color: -2 }));
        assert_eq!(world.target, Some(7));
        assert_eq!(world.target_color, Some(-2));
        world.apply_world(&WorldEvent::ActionFailed(ActionFailed));
        assert_eq!(world.target, Some(7));
    }
}

#[cfg(test)]
mod frente_p_tests {
    use super::tests::{char_info, npc};
    use super::*;
    use crate::state::Dialog;
    use l2_net::game::packets::{GAUGE_BLUE, HIT_CRITICAL, HIT_MISS, SUMMON_PET};
    use l2_net::game::{
        Hit, MemberPosition, PartyBuff, PartyMember, SystemMessageArg, VehicleBoarding, VehicleDeparture, VehicleInfo,
    };

    const ME: i32 = 268481001;

    fn world_with(ids: &[i32]) -> World {
        let mut world = World::default();
        let mut me = npc(ME);
        me.name = "Wender".into();
        world.apply_world(&WorldEvent::NpcInfo(me));
        world.me = world.entities.remove(&ME);
        for id in ids {
            world.apply_world(&WorldEvent::NpcInfo(npc(*id)));
        }
        world
    }

    fn member(object_id: i32, name: &str) -> PartyMember {
        PartyMember {
            object_id,
            name: name.into(),
            cp: 1,
            max_cp: 2,
            hp: 3,
            max_hp: 4,
            mp: 5,
            max_mp: 6,
            level: 7,
            class_id: 8,
        }
    }

    #[test]
    fn attack_records_the_hits_moves_the_attacker_and_marks_combat() {
        let mut world = world_with(&[1, 2]);
        world.entities.get_mut(&2).unwrap().position = Vec3::new(30.0, 20.0, 30.0);
        let hits = vec![
            Hit { target_id: 2, damage: 90, flags: HIT_CRITICAL },
            Hit { target_id: ME, damage: 0, flags: HIT_MISS },
        ];
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation {
            object_id: 1,
            to_x: 500,
            to_y: 0,
            to_z: 0,
            x: 10,
            y: 20,
            z: 30,
        }));
        world.apply_world(&WorldEvent::Attack(Attack { attacker_id: 1, x: 11, y: 21, z: 31, hits }));
        let attacker = &world.entities[&1];
        assert_eq!(attacker.position, Vec3::new(10.0, 20.0, 30.0));
        assert!(attacker.dest.is_none() && attacker.in_combat);
        assert_eq!(attacker.target, Some(2));
        assert_eq!(attacker.heading, 0);
        assert_eq!(world.combat.hits.len(), 2);
        assert_eq!(world.combat.hits[0].attacker_id, 1);
        assert!(world.combat.hits[0].hit.critical());
        assert!(world.combat.hits[1].hit.miss());
        for _ in 0..crate::state::HIT_HISTORY {
            world.apply_world(&WorldEvent::Attack(Attack {
                attacker_id: 2,
                x: 0,
                y: 0,
                z: 0,
                hits: vec![Hit { target_id: 1, damage: 1, flags: 0 }],
            }));
        }
        assert_eq!(world.combat.hits.len(), crate::state::HIT_HISTORY);
        assert_eq!(world.combat.hits[0].attacker_id, 2);
    }

    #[test]
    fn auto_attack_start_and_stop_track_who_is_fighting() {
        let mut world = world_with(&[1]);
        world.target = Some(1);
        world.apply_world(&WorldEvent::AutoAttackStart(AutoAttackStart { target_id: ME }));
        assert_eq!(world.combat.auto_attack_target, Some(1));
        assert!(world.me.as_ref().unwrap().in_combat);
        world.apply_world(&WorldEvent::AutoAttackStart(AutoAttackStart { target_id: 1 }));
        assert!(world.entities[&1].in_combat);
        assert!(world.combat.auto_attacking.contains_key(&1));
        world.apply_world(&WorldEvent::AutoAttackStop(AutoAttackStop { target_id: 1 }));
        assert!(!world.entities[&1].in_combat && !world.combat.auto_attacking.contains_key(&1));
        world.apply_world(&WorldEvent::AutoAttackStop(AutoAttackStop { target_id: ME }));
        assert_eq!(world.combat.auto_attack_target, None);
        assert!(!world.me.as_ref().unwrap().in_combat);
    }

    #[test]
    fn auto_soulshot_updates_each_item_state_from_the_server() {
        let mut world = world_with(&[]);
        world.apply_world(&WorldEvent::AutoSoulShot(AutoSoulShot { values: [1463, 1] }));
        assert_eq!(world.combat.auto_soulshots.get(&1463), Some(&true));
        world.apply_world(&WorldEvent::AutoSoulShot(AutoSoulShot { values: [1463, 0] }));
        assert_eq!(world.combat.auto_soulshots.get(&1463), Some(&false));
        world.apply_world(&WorldEvent::AutoSoulShot(AutoSoulShot { values: [1463, 1] }));
        world.apply_world(&WorldEvent::AutoSoulShot(AutoSoulShot { values: [2509, 1] }));
        world.apply_world(&WorldEvent::AutoSoulShot(AutoSoulShot { values: [0, 0] }));
        assert!(world.combat.auto_soulshots.values().all(|on| !on), "0x1006e2d0: (0, 0) desliga todos");
    }

    #[test]
    fn drop_item_spawns_the_item_and_get_item_removes_it() {
        let mut world = world_with(&[1]);
        let source_position = world.entity(1).map(|e| e.position);
        world.apply_world(&WorldEvent::DropItem(DropItem {
            player_id: 1,
            object_id: 900,
            item_id: 57,
            x: 1,
            y: 2,
            z: 3,
            stackable: true,
            count: 25,
            heading: 16384,
        }));
        assert!(matches!(world.entities[&900].kind, EntityKind::Item { item_id: 57, count: 25, stackable: true }));
        assert_eq!(world.entities[&900].position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(world.entities[&900].heading, 16384);
        assert_eq!(world.entities[&900].item_origin,Some(ItemOrigin::Drop { source_id:1,source_position }));
        world.update(1,|e| e.position = Vec3::new(999.0,999.0,999.0));
        assert_eq!(world.entities[&900].item_origin,Some(ItemOrigin::Drop { source_id:1,source_position }));
        world.target = Some(900);
        world.apply_world(&WorldEvent::GetItem(GetItem { player_id: 1, object_id: 900, x: 1, y: 2, z: 3 }));
        assert!(!world.entities.contains_key(&900));
        assert_eq!(world.target, None);
    }

    #[test]
    fn spawn_replaces_drop_origin_and_absent_source_remains_absent() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::DropItem(DropItem { player_id:7,object_id:9,item_id:57,x:1,y:2,z:3,stackable:true,count:1,heading:0 }));
        assert_eq!(world.entities[&9].item_origin,Some(ItemOrigin::Drop { source_id:7,source_position:None }));
        let previous = world.entities[&9].spawned;
        world.apply_world(&WorldEvent::SpawnItem(SpawnItem { object_id:9,item_id:57,x:4,y:5,z:6,stackable:true,count:1,heading:16384 }));
        assert_eq!(world.entities[&9].item_origin,Some(ItemOrigin::Spawn));
        assert_eq!(world.entities[&9].position,Vec3::new(4.0,5.0,6.0));
        assert_eq!(world.entities[&9].heading,16384);
        assert!(world.entities[&9].spawned > previous);
        world.apply_world(&WorldEvent::DeleteObject(DeleteObject { object_id:9 }));
        assert!(!world.entities.contains_key(&9));
    }

    #[test]
    fn sun_rise_and_sun_set_flip_the_daylight() {
        let mut world = World::default();
        assert_eq!(world.daylight, None);
        world.apply_world(&WorldEvent::SunSet(SunSet));
        assert_eq!(world.daylight, Some(false));
        world.apply_world(&WorldEvent::SunRise(SunRise));
        assert_eq!(world.daylight, Some(true));
    }

    #[test]
    fn sun_rise_and_sun_set_reset_the_game_clock_keeping_its_ratio() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::SunRise(SunRise));
        assert!(world.clock.is_none());
        world.apply(&SessionEvent::Time(l2_net::game::ClientSetTime { minutes: 100, ratio: 6 }));
        world.apply_world(&WorldEvent::SunRise(SunRise));
        let clock = world.clock.unwrap();
        assert_eq!((clock.minutes_at_sync, clock.ratio), (420, 6));
        world.apply_world(&WorldEvent::SunSet(SunSet));
        let clock = world.clock.unwrap();
        assert_eq!((clock.minutes_at_sync, clock.ratio), (60, 6));
    }

    #[test]
    fn asks_and_confirm_dlg_queue_dialogs_in_order() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::CharInfo(Box::new(char_info(7))));
        world.apply_world(&WorldEvent::AskJoinParty(AskJoinParty {
            requestor_name: "Luzal".into(),
            item_distribution: 2,
        }));
        world
            .apply_world(&WorldEvent::AskJoinPledge(AskJoinPledge { requestor_id: 7, pledge_name: "Canastra".into() }));
        world.apply_world(&WorldEvent::AskJoinAlliance(AskJoinAlliance {
            requestor_id: 7,
            requestor_name: "L".into(),
            requestor_pledge_name: String::new(),
            requestor_ally_name: "A".into(),
        }));
        world.apply_world(&WorldEvent::ConfirmDlg(SystemMessage {
            id: 1510,
            args: vec![SystemMessageArg::Text("Luzal".into())],
        }));
        assert_eq!(world.dialogs.pending.len(), 4);
        assert!(matches!(world.dialogs.take(), Some(Dialog::JoinParty(ask)) if ask.requestor_name == "Luzal"));
        assert!(matches!(world.dialogs.take(), Some(Dialog::JoinPledge(ask)) if ask.pledge_name == "Canastra"));
        assert!(matches!(world.dialogs.take(), Some(Dialog::JoinAlliance(_))));
        assert!(matches!(world.dialogs.take(), Some(Dialog::Confirm(m)) if m.id == 1510));
        assert!(world.dialogs.take().is_none());
    }

    #[test]
    fn invitations_from_unknown_users_are_refused_at_once_like_the_console_handlers() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::AskJoinPledge(AskJoinPledge { requestor_id: 9, pledge_name: "C".into() }));
        world.apply_world(&WorldEvent::AskJoinAlliance(AskJoinAlliance {
            requestor_id: 9,
            requestor_name: "L".into(),
            requestor_pledge_name: String::new(),
            requestor_ally_name: "A".into(),
        }));
        world.apply_world(&WorldEvent::AskJoinParty(AskJoinParty { requestor_name: String::new(), item_distribution: 0 }));
        world.apply_world(&WorldEvent::TradeRequest(TradeRequest { sender_id: 9 }));
        assert!(world.dialogs.pending.is_empty());
        assert_eq!(
            world.take_requests(),
            vec![
                NetCommand::AnswerJoinPledge { accept: false },
                NetCommand::AnswerJoinAlly { accept: false },
                NetCommand::AnswerJoinParty { accept: false },
                NetCommand::AnswerTrade { accept: false },
            ]
        );
    }

    #[test]
    fn showing_the_pet_status_window_clears_its_buffs_but_a_refresh_keeps_them() {
        let mut world = World { me: Some(Entity::from_char_info(&char_info(1))), ..Default::default() };
        let pet = |object_id: i32| pet_info(object_id, "Luzal");
        world.party.spelled.insert(5, vec![l2_net::game::packets::PartyBuff { skill_id: 1, level: 1, duration: 10 }]);
        world.apply_world(&WorldEvent::PetInfo(Box::new(pet(5))));
        assert!(!world.party.spelled.contains_key(&5));
        world.party.spelled.insert(5, vec![l2_net::game::packets::PartyBuff { skill_id: 1, level: 1, duration: 10 }]);
        world.apply_world(&WorldEvent::PetInfo(Box::new(pet(5))));
        assert!(world.party.spelled.contains_key(&5));
    }

    #[test]
    fn dying_cancels_the_target_and_clears_the_buffs_like_the_restart_menu_handler() {
        let mut world = World {
            me: Some(Entity::from_char_info(&char_info(1))),
            buffs: vec![Buff { skill_id: 1, level: 1, duration: 10 }],
            short_buff: Some(ShortBuffStatusUpdate { skill_id: 1, level: 1, duration: 10 }),
            ..Default::default()
        };
        world.apply_world(&WorldEvent::Die(Die {
            object_id: 2,
            to_village: false,
            to_hideout: false,
            to_castle: false,
            to_siege_hq: false,
            sweepable: false,
            to_fixed_spot: false,
        }));
        assert!(world.take_requests().is_empty() && world.buffs.len() == 1);
        world.apply_world(&WorldEvent::Die(Die {
            object_id: 1,
            to_village: true,
            to_hideout: false,
            to_castle: false,
            to_siege_hq: false,
            sweepable: false,
            to_fixed_spot: false,
        }));
        assert_eq!(world.take_requests(), vec![NetCommand::TargetCancel]);
        assert!(world.buffs.is_empty() && world.short_buff.is_none());
    }

    #[test]
    fn event_trigger_keeps_the_id_and_the_sign_extended_on() {
        let mut world = World::default();
        assert_eq!(world.event_trigger, None);
        world.apply_world(&WorldEvent::EventTrigger(EventTrigger { event_id: 2_418_001, on: 1 }));
        assert_eq!(world.event_trigger, Some((2_418_001, 1)));
        world.apply_world(&WorldEvent::EventTrigger(EventTrigger { event_id: 2_418_002, on: 0xff }));
        assert_eq!(world.event_trigger, Some((2_418_002, -1)));
        assert_eq!(event_trigger(&EventTrigger { event_id: 5, on: 0 }), (5, 0));
    }

    #[test]
    fn doors_and_static_objects_are_selectable_and_carry_the_bar_only_for_doors() {
        let mut world = World::default();
        assert!(!world.select(100));
        world.apply_world(&WorldEvent::DoorStatusUpdate(DoorStatusUpdate {
            object_id: 100, closed: true, damage: 0, show_hp: true, door_id: 24190001, max_hp: 500, hp: 400,
        }));
        assert!(!world.select(100));
        world.apply_world(&WorldEvent::DoorInfo(DoorInfo { object_id: 100, door_id: 24190001 }));
        assert!(world.select(100));
        assert!(world.targeted().is_none());
        assert_eq!(world.targeted_static(), Some(StaticTarget { object_id: 100, static_object_id: 24190001, hp: Some((400, 500)) }));
        world.apply_world(&WorldEvent::StaticObject(l2_net::game::StaticObject { static_object_id: 17220001, object_id: 900 }));
        assert!(world.select(900));
        assert_eq!(world.targeted_static(), Some(StaticTarget { object_id: 900, static_object_id: 17220001, hp: None }));
        assert_eq!(world.object_for_server_id(24190001), Some(100));
        assert_eq!(world.object_for_server_id(17220001), Some(900));
        assert_eq!(world.object_for_server_id(1), None);
    }

    #[test]
    fn doors_keep_id_state_and_hp() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::DoorInfo(DoorInfo { object_id: 100, door_id: 24190001 }));
        assert_eq!(world.doors[&100].door_id, 24190001);
        assert_eq!(world.doors[&100].info_door_id, Some(24190001));
        assert!(!world.doors[&100].status_received);
        world.apply_world(&WorldEvent::DoorStatusUpdate(DoorStatusUpdate {
            object_id: 100,
            closed: false,
            damage: 1,
            show_hp: true,
            door_id: 24190001,
            max_hp: 500,
            hp: 400,
        }));
        let door = world.doors[&100];
        assert!(!door.closed && door.show_hp);
        assert!(door.status_received);
        assert_eq!((door.hp, door.max_hp, door.damage), (400, 500, 1));
        world.apply_world(&WorldEvent::DoorStatusUpdate(DoorStatusUpdate {
            object_id: 101,
            closed: true,
            damage: 0,
            show_hp: false,
            door_id: 5,
            max_hp: 0,
            hp: 0,
        }));
        assert!(world.doors[&101].closed);
        assert_eq!(world.doors[&101].info_door_id, None);
        assert!(!world.doors[&101].status_received);
        world.apply_world(&WorldEvent::DoorInfo(DoorInfo { object_id: 101, door_id: 5 }));
        assert!(!world.doors[&101].status_received);
    }

    #[test]
    fn party_window_all_add_update_delete_positions_and_spelled() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::PartySmallWindowAll(PartySmallWindowAll {
            leader_id: 1,
            loot_distribution: 2,
            members: vec![member(11, "A"), member(12, "B")],
        }));
        assert!(world.party.in_party());
        assert_eq!((world.party.leader_id, world.party.loot_distribution, world.party.members.len()), (1, 2, 2));
        world.apply_world(&WorldEvent::PartySmallWindowAdd(PartySmallWindowAdd {
            leader_id: 1,
            loot_distribution: 2,
            member: member(13, "C"),
        }));
        assert_eq!(world.party.members.len(), 3);
        let mut c = member(13, "C");
        c.hp = 99;
        world.apply_world(&WorldEvent::PartySmallWindowUpdate(c));
        assert_eq!(world.party.member(13).unwrap().hp, 99);
        assert_eq!(world.party.members.len(), 3);
        world.apply_world(&WorldEvent::PartyMemberPosition(PartyMemberPosition {
            positions: vec![MemberPosition { object_id: 11, x: 1, y: 2, z: 3 }],
        }));
        assert_eq!(world.party.positions.get(&11), Some(&[1, 2, 3]));
        world.apply_world(&WorldEvent::PartySpelled(PartySpelled {
            summon_type: 0,
            object_id: 11,
            effects: vec![PartyBuff { skill_id: 1204, level: 2, duration: 30 }],
        }));
        assert_eq!(world.party.spelled[&11].len(), 1);
        world.apply_world(&WorldEvent::PartySmallWindowDelete(PartySmallWindowDelete {
            object_id: 11,
            name: "A".into(),
        }));
        assert!(
            world.party.member(11).is_none()
                && !world.party.positions.contains_key(&11)
                && !world.party.spelled.contains_key(&11)
        );
        world.apply_world(&WorldEvent::PartySmallWindowDeleteAll(PartySmallWindowDeleteAll));
        assert!(!world.party.in_party());
        assert_eq!(world.party, Party::default());
    }

    #[test]
    fn vehicles_board_move_and_unboard_relative_to_the_boat() {
        let mut world = world_with(&[1]);
        world.apply_world(&WorldEvent::VehicleInfo(VehicleInfo {
            object_id: 500,
            x: 1000,
            y: 2000,
            z: -30,
            heading: 0,
        }));
        assert_eq!(world.vehicles[&500].x, 1000);
        world.apply_world(&WorldEvent::VehicleDeparture(VehicleDeparture {
            object_id: 500,
            move_speed: 300,
            rotation_speed: 100,
            to_x: 5,
            to_y: 6,
            to_z: 7,
        }));
        assert_eq!(world.vehicles[&500].dest, Some([5, 6, 7]));
        assert_eq!(world.vehicles[&500].move_speed, 300);
        world.apply_world(&WorldEvent::VehicleStart(VehicleStart { object_id: 500, state: 1 }));
        assert_eq!(world.vehicles[&500].state, 1);
        world.apply_world(&WorldEvent::GetOnVehicle(VehicleBoarding {
            object_id: 1,
            vehicle_id: 500,
            x: 10,
            y: 0,
            z: 5,
        }));
        assert_eq!(world.entities[&1].vehicle, Some(500));
        assert_eq!(world.entities[&1].position, Vec3::new(1010.0, 2000.0, -25.0));
        world.apply_world(&WorldEvent::MoveToLocationInVehicle(MoveToLocationInVehicle {
            object_id: 1,
            vehicle_id: 500,
            to_x: 20,
            to_y: 0,
            to_z: 5,
            x: 10,
            y: 0,
            z: 5,
        }));
        assert_eq!(world.entities[&1].dest, Some(Vec3::new(1020.0, 2000.0, -25.0)));
        world.apply_world(&WorldEvent::StopMoveInVehicle(StopMoveInVehicle {
            object_id: 1,
            vehicle_id: 500,
            x: 20,
            y: 0,
            z: 5,
            heading: 9,
        }));
        assert_eq!(world.entities[&1].dest, Some(Vec3::new(1020.0, 2000.0, -25.0)));
        assert_eq!(world.entities[&1].stop_heading, Some(9));
        world.apply_world(&WorldEvent::GetOffVehicle(VehicleBoarding {
            object_id: 1,
            vehicle_id: 500,
            x: 7,
            y: 8,
            z: 9,
        }));
        assert_eq!(world.entities[&1].vehicle, None);
        assert_eq!(world.entities[&1].position, Vec3::new(7.0, 8.0, 9.0));
        world.apply_world(&WorldEvent::VehicleDeparture(VehicleDeparture {
            object_id: 501,
            move_speed: 0,
            rotation_speed: 0,
            to_x: 0,
            to_y: 0,
            to_z: 0,
        }));
        assert!(!world.vehicles.contains_key(&501));
    }

    #[test]
    fn rotating_keeps_the_turn_until_finish_sets_the_heading() {
        let mut world = world_with(&[1]);
        world.apply_world(&WorldEvent::StartRotating(StartRotating { object_id: 1, degree: 90, side: 1, speed: 0 }));
        assert_eq!(world.entities[&1].rotation, Some(Rotation { degree: 90, side: 1, speed: 0 }));
        world.apply_world(&WorldEvent::FinishRotating(FinishRotating { object_id: 1, degree: 16384, speed: 0 }));
        assert_eq!(world.entities[&1].rotation, None);
        assert_eq!(world.entities[&1].heading, 16384);
    }

    #[test]
    fn setup_gauge_ride_chair_nickname_and_fly() {
        let mut world = world_with(&[1]);
        world.apply_world(&WorldEvent::SetupGauge(SetupGauge { color: GAUGE_BLUE, time: 5000, max_time: 5000 }));
        let gauge = world.gauge.expect("gauge ligada");
        assert_eq!(gauge.setup.time, 5000);
        assert!(gauge.remaining(gauge.started) <= 5000);
        assert_eq!(gauge.remaining(gauge.started + killer_platform::time::Duration::from_secs(9)), 0);
        world.apply_world(&WorldEvent::SetupGauge(SetupGauge { color: GAUGE_BLUE, time: 0, max_time: 0 }));
        assert!(world.gauge.is_none());
        world.apply_world(&WorldEvent::Ride(Ride { object_id: 1, mounted: true, ride_type: 1, ride_class_id: 12526 }));
        assert_eq!(world.entities[&1].mount, Some(Mount { ride_type: 1, ride_class_id: 12526 }));
        world.apply_world(&WorldEvent::Ride(Ride { object_id: 1, mounted: false, ride_type: 0, ride_class_id: 0 }));
        assert_eq!(world.entities[&1].mount, None);
        world.apply_world(&WorldEvent::ChairSit(ChairSit { object_id: 1, static_object_id: 24 }));
        assert!(world.entities[&1].sitting);
        assert_eq!(world.entities[&1].chair, Some(24));
        world.apply_world(&WorldEvent::ChangeWaitType(ChangeWaitType {
            object_id: 1,
            kind: 1,
            sitting: false,
            x: 10,
            y: 20,
            z: 30,
        }));
        assert!(!world.entities[&1].sitting && world.entities[&1].chair.is_none());
        world.apply_world(&WorldEvent::NicknameChanged(NicknameChanged { object_id: 1, title: "Herói".into() }));
        assert_eq!(world.entities[&1].title, "Herói");
        world.apply_world(&WorldEvent::FlyToLocation(FlyToLocation {
            object_id: 1,
            to_x: 1,
            to_y: 2,
            to_z: 3,
            x: 0,
            y: 0,
            z: 0,
        }));
        assert_eq!(world.entities[&1].position, Vec3::new(1.0, 2.0, 3.0));
        assert!(world.entities[&1].dest.is_none());
    }

    #[test]
    fn server_object_info_and_static_object_spawn_the_static_npc() {
        let mut world = World::default();
        let info = ServerObjectInfo {
            object_id: 300,
            npc_id: 35000,
            name: String::new(),
            attackable: true,
            x: 1,
            y: 2,
            z: 3,
            heading: 4,
            speed_multiplier: 1.0,
            attack_speed_multiplier: 1.0,
            collision_radius: 12.0,
            collision_height: 30.0,
            hp: 900,
            max_hp: 1000,
            object_type: 1,
            special_effects: 0,
        };
        world.apply_world(&WorldEvent::ServerObjectInfo(info));
        let e = &world.entities[&300];
        assert!(matches!(e.kind, EntityKind::Npc { npc_id: 35000, attackable: true, .. }));
        assert_eq!(e.hp, Some((900, 1000)));
        assert_eq!(e.collision_height, 30.0);
        world.apply_world(&WorldEvent::StaticObject(StaticObject { static_object_id: 24, object_id: 900 }));
        assert_eq!(world.static_objects[&900], 24);
    }

    #[test]
    fn play_sound_queues_and_the_app_drains() {
        let mut world = World::default();
        let sound = PlaySound {
            kind: 0,
            sound_file: "ItemSound.quest_accept".into(),
            bind_to_object: false,
            object_id: 0,
            x: 0,
            y: 0,
            z: 0,
            delay: 0,
        };
        for _ in 0..crate::state::SOUND_QUEUE + 1 {
            world.apply_world(&WorldEvent::PlaySound(sound.clone()));
        }
        assert_eq!(world.effects.sounds.len(), crate::state::SOUND_QUEUE);
        assert_eq!(world.effects.take_sounds().len(), crate::state::SOUND_QUEUE);
        assert!(world.effects.sounds.is_empty());
    }

    #[test]
    fn client_action_1007a3a0_targets_the_nearest_pawn_item_or_current_target() {
        let mut world = World::default();
        for (id, x) in [(10, 0.0), (11, 150.0), (12, 190.0), (13, 120.0), (14, 260.0)] {
            world.apply_world(&WorldEvent::NpcInfo(npc(id)));
            let entity = world.entities.get_mut(&id).unwrap();
            entity.position = Vec3::new(x, 0.0, 0.0);
        }
        world.entities.get_mut(&13).unwrap().dead = true;
        world.me = world.entities.remove(&10);
        world.entities.insert(20, Entity::item(20, 57, 1, true, Vec3::new(180.0, 0.0, 0.0)));
        let action = |world: &World, d: i32| world.client_action(d, true);
        assert_eq!(action(&world, 4), Some(NetCommand::Action { object_id: 11, origin: [0, 0, 0], shift: true }));
        world.target = Some(11);
        assert_eq!(action(&world, 4), Some(NetCommand::Action { object_id: 12, origin: [0, 0, 0], shift: true }));
        assert_eq!(action(&world, 5), Some(NetCommand::Action { object_id: 20, origin: [0, 0, 0], shift: true }));
        assert_eq!(action(&world, 6), None, "o alvo sem alvo próprio não dá assist");
        world.entities.get_mut(&11).unwrap().target = Some(12);
        assert_eq!(action(&world, 6), Some(NetCommand::Action { object_id: 12, origin: [0, 0, 0], shift: true }));
        world.entities.get_mut(&11).unwrap().target = Some(99);
        assert_eq!(action(&world, 6), None);
        world.static_objects.insert(99, 7);
        assert_eq!(action(&world, 6), Some(NetCommand::Action { object_id: 7, origin: [0, 0, 0], shift: true }));
        assert_eq!(action(&world, 7), None);
        world.target = None;
        world.entities.retain(|id, _| *id == 14);
        assert_eq!(action(&world, 4), None);
        assert_eq!(action(&world, 5), None);
        assert_eq!(action(&world, 6), None);
        world.apply_world(&WorldEvent::ClientAction(ClientAction { action: 5 }));
        assert_eq!(world.pending_client_action, Some(5));
    }

    #[test]
    fn alliance_war_10007a880_opens_a_dialog_only_with_both_names() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::StartAllianceWar(AllianceWar { first: "A".into(), second: "B".into() }));
        world.apply_world(&WorldEvent::StopAllianceWar(AllianceWar { first: "A".into(), second: String::new() }));
        world.apply_world(&WorldEvent::StopAllianceWar(AllianceWar { first: "C".into(), second: "D".into() }));
        assert_eq!(world.dialogs.take(), Some(Dialog::AllianceWar { start: true, first: "A".into(), second: "B".into() }));
        assert_eq!(world.dialogs.take(), Some(Dialog::AllianceWar { start: false, first: "C".into(), second: "D".into() }));
        assert_eq!(world.dialogs.take(), None);
    }

    #[test]
    fn fishing_packets_drive_the_pawn_and_the_local_window() {
        use l2_net::game::{FishingEnd, FishingHpRegen, FishingStart, FishingStartCombat};
        let mut world = World::default();
        world.apply_world(&WorldEvent::NpcInfo(npc(7)));
        world.me = world.entities.remove(&7);
        world.apply_world(&WorldEvent::FishingStartCombat(FishingStartCombat { values: [7, 30, 100], flags: [0, 0] }));
        assert_eq!(world.me.as_ref().unwrap().fishing, 0);
        world.apply_world(&WorldEvent::FishingStart(FishingStart { values: [7, 1, 11, 22, 33] }));
        assert_eq!(world.me.as_ref().unwrap().fishing, 1);
        assert!(world.fishing_window.visible);
        assert!(world.fishing_view.active);
        assert_eq!(world.fishing_view.float, Some(Vec3::new(11.0, 22.0, 33.0)));
        assert_eq!(world.fishing_view.calc_step, crate::fishing_view::CALC_STEP_CAST);
        world.apply_world(&WorldEvent::FishingStartCombat(FishingStartCombat { values: [7, 30, 100], flags: [1, 0] }));
        assert_eq!((world.me.as_ref().unwrap().fishing, world.fishing_window.start_hp, world.fishing_window.mode), (2, 100, 1));
        assert_eq!(world.fishing_view.calc_step, crate::fishing_view::CALC_STEP_COMBAT);
        world.apply_world(&WorldEvent::FishingHpRegen(FishingHpRegen { values: [7, 27, 90], flags: [1, 1, 1], amount: 0 }));
        assert!(world.fishing_window.started);
        assert_eq!(world.fishing_window.hp, 90);
        world.apply_world(&WorldEvent::FishingEnd(FishingEnd { value: 7, flag: 0 }));
        assert_eq!(world.me.as_ref().unwrap().fishing, 0);
        assert!(!world.fishing_window.visible);
        assert!(!world.fishing_view.active);
    }

    #[test]
    fn time_check_and_net_ping_answer_like_the_engine() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::NpcInfo(npc(7)));
        world.me = world.entities.remove(&7);
        world.apply_world(&WorldEvent::RequestTimeCheck(RequestTimeCheck { value: 50 }));
        world.tick(0.0255);
        assert!(world.requests.is_empty());
        world.tick(0.0255);
        assert_eq!(world.take_requests(), [NetCommand::SendTimeCheck { first: 50, second: 50 }]);
        assert_eq!(world.time_check, None);
        world.pawn_clipping = 1.5;
        world.frame_time = 0.0;
        world.tick(0.02);
        world.apply_world(&WorldEvent::NetPingResponse(NetPingResponse { value: 9 }));
        assert_eq!(world.take_requests(), [NetCommand::NetPing { first: 9, second: 50, third: 3072 }]);
        assert_eq!(world.frame_time, 0.0);
    }

    #[test]
    fn skill_remain_sec_10470560_resumes_the_cast_with_the_remaining_seconds() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::NpcInfo(npc(7)));
        world.apply_world(&WorldEvent::SkillRemainSec(SkillRemainSec { values: [7, 99, 1177, 3, 0, 0, 2] }));
        let casting = world.entities[&7].casting.unwrap();
        assert_eq!((casting.skill_id, casting.skill_level, casting.hit_time, casting.target_id), (1177, 3, 2000, 7));
    }

    #[test]
    fn validate_location_in_vehicle_104766d0_teleports_me_and_moves_others() {
        let mut world = World::default();
        world.vehicles.insert(50, Vehicle { x: 1000, y: 0, z: 0, heading: 16384, dest: None, move_speed: 0, rotation_speed: 0, state: 0 });
        world.apply_world(&WorldEvent::NpcInfo(npc(7)));
        world.apply_world(&WorldEvent::NpcInfo(npc(8)));
        world.me = world.entities.remove(&7);
        world.apply_world(&WorldEvent::ValidateLocationInVehicle(ValidateLocationInVehicle { values: [7, 50, 100, 0, 10, 4000] }));
        let me = world.me.as_ref().unwrap();
        assert!((me.position - Vec3::new(1000.0, 100.0, 10.0)).length() < 1e-3);
        assert_eq!((me.heading, me.vehicle), (4000, Some(50)));
        world.entities.get_mut(&8).unwrap().position = Vec3::new(1000.0, 0.0, 500.0);
        world.apply_world(&WorldEvent::ValidateLocationInVehicle(ValidateLocationInVehicle { values: [8, 50, 0, 50, 0, 0] }));
        assert!((world.entities[&8].position - Vec3::new(950.0, 0.0, 0.0)).length() < 1e-3);
    }

    #[test]
    fn earthquakes_and_camera_commands_queue_up() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::Earthquake(Earthquake { x: 1, y: 2, z: 3, intensity: 20, duration: 10 }));
        world.apply_world(&WorldEvent::Earthquake(Earthquake { x: 1, y: 2, z: 3, intensity: 30, duration: 5 }));
        assert_eq!(world.effects.take_earthquakes().iter().map(|q| q.intensity).collect::<Vec<_>>(), [20, 30]);
        assert!(world.effects.take_earthquakes().is_empty());
        let camera = SpecialCamera {
            target_id: 7,
            distance: 300,
            yaw: 90,
            pitch: 10,
            time: 2000,
            duration: 5000,
            turn: 1,
            rise: 0,
            widescreen: 1,
            unknown: 0,
        };
        world.apply_world(&WorldEvent::SpecialCamera(camera));
        world.apply_world(&WorldEvent::NormalCameraNoop(NormalCameraNoop));
        world.apply_world(&WorldEvent::CameraMode(CameraMode { mode: 1 }));
        assert_eq!(world.effects.take_camera_commands(), [CameraCommand::Special(camera), CameraCommand::Normal, CameraCommand::Mode(1)]);
    }

    fn pet_info(object_id: i32, owner: &str) -> PetInfo {
        let n = npc(object_id);
        PetInfo {
            summon_type: SUMMON_PET,
            object_id,
            npc_id: 12077,
            attackable: false,
            x: n.x,
            y: n.y,
            z: n.z,
            heading: n.heading,
            m_atk_speed: n.m_atk_speed,
            p_atk_speed: n.p_atk_speed,
            run_speed: n.run_speed,
            walk_speed: n.walk_speed,
            speed_multiplier: n.speed_multiplier,
            attack_speed_multiplier: n.attack_speed_multiplier,
            collision_radius: n.collision_radius,
            collision_height: n.collision_height,
            rhand: 0,
            chest: 0,
            lhand: 0,
            has_owner: true,
            running: true,
            in_combat: false,
            alike_dead: false,
            spawn_animation: 2,
            name: "Wolf".into(),
            title: owner.into(),
            pvp_flag: 0,
            karma: 0,
            fed: 80,
            max_fed: 100,
            hp: 500,
            max_hp: 600,
            mp: 70,
            max_mp: 90,
            sp: 0,
            level: 25,
            exp: 1000,
            exp_this_level: 900,
            exp_next_level: 1500,
            load: 0,
            max_load: 0,
            p_atk: 0,
            p_def: 0,
            m_atk: 0,
            m_def: 0,
            accuracy: 0,
            evasion: 0,
            critical: 0,
            speed: 0,
            atk_spd: 0,
            cast_spd: 0,
            abnormal_effect: 0,
            mountable: 1,
            team: 0,
            soulshots_per_hit: 1,
            spiritshots_per_hit: 1,
        }
    }

    #[test]
    fn my_pet_is_kept_and_updated_and_other_pets_are_only_entities() {
        let mut world = world_with(&[]);
        world.apply_world(&WorldEvent::PetInfo(Box::new(pet_info(700, "Alguem"))));
        assert!(world.pet.is_none());
        assert!(matches!(world.entities[&700].kind, EntityKind::Npc { npc_id: 12077, .. }));
        world.apply_world(&WorldEvent::PetStatusShow(PetStatusShow { summon_type: SUMMON_PET }));
        assert_eq!(world.pet_window, Some(SUMMON_PET));
        world.apply_world(&WorldEvent::PetInfo(Box::new(pet_info(600, "Wender"))));
        let pet = world.pet.as_ref().expect("pet meu");
        assert_eq!((pet.info.object_id, pet.info.hp, pet.info.max_hp), (600, 500, 600));
        assert_eq!(world.entities[&600].hp, Some((500, 600)));
        world
            .apply_world(&WorldEvent::SetSummonRemainTime(SetSummonRemainTime { max_time: 1200, remaining_time: 600 }));
        assert_eq!((world.pet.as_ref().unwrap().max_time, world.pet.as_ref().unwrap().remaining_time), (1200, 600));
        world.apply_world(&WorldEvent::PetStatusUpdate(PetStatusUpdate {
            summon_type: SUMMON_PET,
            object_id: 600,
            x: 1,
            y: 2,
            z: 3,
            name: String::new(),
            fed: 70,
            max_fed: 100,
            hp: 450,
            max_hp: 600,
            mp: 60,
            max_mp: 90,
            level: 26,
            exp: 1100,
            exp_this_level: 1000,
            exp_next_level: 1600,
        }));
        let pet = world.pet.as_ref().unwrap();
        assert_eq!((pet.info.fed, pet.info.hp, pet.info.level, pet.info.x), (70, 450, 26, 1));
        assert_eq!(world.entities[&600].hp, Some((450, 600)));
        assert_eq!(world.entities[&600].level, 26);
        world.apply_world(&WorldEvent::PetInfo(Box::new(pet_info(600, "Wender"))));
        assert_eq!(world.pet.as_ref().unwrap().remaining_time, 600);
        world.apply_world(&WorldEvent::PetDelete(PetDelete { summon_type: SUMMON_PET, object_id: 700 }));
        assert!(world.pet.is_some());
        world.apply_world(&WorldEvent::PetDelete(PetDelete { summon_type: SUMMON_PET, object_id: 600 }));
        assert!(world.pet.is_none() && world.pet_window.is_none());
    }

    #[test]
    fn dice_history_keeps_only_the_newest_entries() {
        let mut world = World::default();
        for number in 0..(HISTORY_LIMIT as i32 + 50) {
            world.apply_world(&WorldEvent::Dice(Dice { player_id: 7, item_id: 4625, number, x: 0, y: 0, z: 0 }));
        }
        assert_eq!(world.dice.len(), HISTORY_LIMIT);
        assert_eq!(world.dice[0].number, 50);
        assert_eq!(world.dice.last().map(|d| d.number), Some(HISTORY_LIMIT as i32 + 49));
    }

    #[test]
    fn dice_and_short_buff_are_kept() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::Dice(Dice { player_id: 7, item_id: 4625, number: 6, x: 0, y: 0, z: 0 }));
        assert_eq!(world.dice.len(), 1);
        world.apply_world(&WorldEvent::ShortBuffStatusUpdate(ShortBuffStatusUpdate {
            skill_id: 2274,
            level: 1,
            duration: 20,
        }));
        assert_eq!(world.short_buff.map(|b| b.skill_id), Some(2274));
        assert!(world.short_buff_at.is_some());
        world.apply_world(&WorldEvent::ShortBuffStatusUpdate(ShortBuffStatusUpdate {
            skill_id: 2274,
            level: 1,
            duration: 0,
        }));
        assert!(world.short_buff.is_none());
    }
}

#[cfg(test)]
mod frente_p_window_tests {
    use super::*;
    use super::tests::char_info;
    use crate::state::Dialog;
    use l2_net::game::{
        Friend, InventoryItem, ItemChange, LearnableSkill, PricedItem, QuestState, TradeItem, TradeUpdateItem,
    };

    fn item(object_id: i32) -> TradeItem {
        TradeItem {
            type1: 1,
            object_id,
            item_id: 57,
            count: 1,
            type2: 2,
            custom_type1: 0,
            body_part: 0,
            enchant: 0,
            custom_type2: 0,
        }
    }

    fn member(name: &str, level: i32) -> PledgeMember {
        PledgeMember { name: name.into(), level, class_id: 88, unknown: 0, flag: 1, online_object_id: 0 }
    }

    #[test]
    fn shop_and_warehouse_lists_are_kept_for_the_windows() {
        let mut world = World::default();
        let list = ShopList { money: 5000, list_id: 1, items: vec![PricedItem { item: item(1001), price: 10 }] };
        world.apply_world(&WorldEvent::SellList(list.clone()));
        world.apply_world(&WorldEvent::BuyList(list.clone()));
        assert_eq!(world.windows.sell.as_ref().unwrap().money, 5000);
        assert_eq!(world.windows.buy.as_ref().unwrap().items.len(), 1);
        let warehouse = WarehouseList { warehouse_type: 1, adena: 9, items: Vec::new() };
        world.apply_world(&WorldEvent::WarehouseDepositList(warehouse.clone()));
        assert_eq!(world.windows.warehouse.as_ref().unwrap().adena, 9);
        world.apply_world(&WorldEvent::WarehouseWithdrawList(WarehouseList { warehouse_type: 2, ..warehouse }));
        assert_eq!(world.windows.warehouse.as_ref().unwrap().warehouse_type, 2);
    }

    #[test]
    fn trade_flow_start_add_update_done_and_request_dialog() {
        let mut world = World { me: Some(Entity::from_char_info(&char_info(1))), ..Default::default() };
        world.apply_world(&WorldEvent::CharInfo(Box::new(char_info(7))));
        world.apply_world(&WorldEvent::TradeRequest(TradeRequest { sender_id: 7 }));
        assert!(matches!(world.dialogs.take(), Some(Dialog::TradeRequest { requestor_id: 7 })));
        world.apply_world(&WorldEvent::TradeStart(TradeStart { partner_id: 7, items: vec![item(1001)] }));
        assert_eq!(world.trade.partner_id, Some(7));
        assert_eq!(world.trade.tradable.len(), 1);
        world.apply_world(&WorldEvent::TradeOwnAdd(TradeAdd { items: vec![item(1001)] }));
        world.apply_world(&WorldEvent::TradeOtherAdd(TradeAdd { items: vec![item(2001)] }));
        assert_eq!((world.trade.mine.len(), world.trade.theirs.len()), (1, 1));
        let mut changed = item(1001);
        changed.count = 3;
        world.apply_world(&WorldEvent::TradeUpdate(TradeUpdate {
            items: vec![TradeUpdateItem { update_type: 3, item: changed }],
        }));
        assert_eq!(world.trade.tradable[0].count, 3);
        world.apply_world(&WorldEvent::TradeDone(TradeDone { success: true }));
        assert_eq!(world.trade.partner_id, None);
        assert!(world.trade.mine.is_empty() && world.trade.theirs.is_empty());
        assert_eq!(world.trade.last_result, Some(true));
    }

    #[test]
    fn trade_update_1011af40_distinguishes_add_remove_replace_and_unknown() {
        let mut world = World::default();
        for update_type in [3, 2, 0, -1, 4] {
            world.apply_world(&WorldEvent::TradeUpdate(TradeUpdate {
                items: vec![TradeUpdateItem { update_type, item: item(1001) }],
            }));
            assert!(world.trade.tradable.is_empty());
        }
        for _ in 0..2 {
            world.apply_world(&WorldEvent::TradeUpdate(TradeUpdate {
                items: vec![TradeUpdateItem { update_type: 1, item: item(1001) }],
            }));
        }
        assert_eq!(world.trade.tradable.len(), 2);
        world.apply_world(&WorldEvent::TradeUpdate(TradeUpdate {
            items: vec![TradeUpdateItem { update_type: 2, item: item(1001) }],
        }));
        assert_eq!(world.trade.tradable.len(), 1);
    }

    #[test]
    fn pledge_window_members_info_crests_power_and_join() {
        let mut world = World::default();
        let all = PledgeShowMemberListAll {
            pledge_id: 33,
            name: "Canastra".into(),
            leader_name: "Luzal".into(),
            crest_id: 5,
            level: 3,
            castle_id: 0,
            hideout_id: 0,
            my_level: 52,
            dissolving: 0,
            ally_id: 0,
            ally_name: String::new(),
            ally_crest_id: 0,
            at_war: false,
            members: vec![member("A", 40)],
        };
        world.apply_world(&WorldEvent::PledgeShowMemberListAll(all));
        assert_eq!(world.pledge_window.members.len(), 1);
        assert_eq!(world.pledge_window.info.as_ref().unwrap().name, "Canastra");
        world.apply_world(&WorldEvent::PledgeShowMemberListAdd(member("B", 41)));
        world.apply_world(&WorldEvent::PledgeShowMemberListUpdate(member("A", 42)));
        assert_eq!(world.pledge_window.members.len(), 2);
        assert_eq!(world.pledge_window.members[0].level, 42);
        world.apply_world(&WorldEvent::PledgeShowMemberListDelete(PledgeShowMemberListDelete { name: "A".into() }));
        assert_eq!(world.pledge_window.members.len(), 1);
        world.apply_world(&WorldEvent::PledgeShowMemberListDeleteAll(PledgeShowMemberListDeleteAll));
        assert!(world.pledge_window.members.is_empty());
        world.apply_world(&WorldEvent::PledgeCrest(Crest { crest_id: 5, data: vec![1, 2] }));
        world.apply_world(&WorldEvent::AllianceCrest(Crest { crest_id: 6, data: vec![3] }));
        assert_eq!(world.pledge_window.crests[&5], vec![1, 2]);
        assert_eq!(world.pledge_window.ally_crests[&6], vec![3]);
        world.apply_world(&WorldEvent::PledgeInfo(PledgeInfo {
            pledge_id: 33,
            name: "Canastra".into(),
            ally_name: String::new(),
        }));
        assert_eq!(world.pledge_window.summary.as_ref().unwrap().pledge_id, 33);
        world.apply_world(&WorldEvent::ManagePledgePower(ManagePledgePower { first: 0, second: 0, powers: [0; 32] }));
        assert!(world.pledge_window.power.is_some());
        world.apply_world(&WorldEvent::JoinPledge(JoinPledge { pledge_id: 33 }));
        assert_eq!(world.pledge_window.joined_pledge_id, Some(33));
        world.apply_world(&WorldEvent::JoinParty(JoinParty { response: 1 }));
        assert_eq!(world.windows.party_join_result, Some(1));
    }

    #[test]
    fn party_updates_only_touch_known_members_and_the_pledge_list_ignores_unknown_rows() {
        let mut world = World::default();
        let party_member = |hp: i32| PartyMember {
            object_id: 9,
            name: "P".into(),
            cp: 1,
            max_cp: 2,
            hp,
            max_hp: 4,
            mp: 5,
            max_mp: 6,
            level: 7,
            class_id: 8,
        };
        world.apply_world(&WorldEvent::PartySmallWindowUpdate(party_member(5)));
        assert!(world.party.members.is_empty());
        world.party.upsert(&party_member(1));
        world.apply_world(&WorldEvent::PartySmallWindowUpdate(party_member(77)));
        assert_eq!(world.party.members.iter().map(|m| m.hp).collect::<Vec<_>>(), [77]);
        world.apply_world(&WorldEvent::PledgeShowMemberListUpdate(member("Ghost", 10)));
        assert!(world.pledge_window.members.is_empty());
        world.pledge_window.upsert(&member("A", 10));
        world.apply_world(&WorldEvent::PledgeShowMemberListUpdate(member("", 99)));
        world.apply_world(&WorldEvent::PledgeShowMemberListUpdate(member("A", 11)));
        assert_eq!(world.pledge_window.members.iter().map(|m| m.level).collect::<Vec<_>>(), [11]);
    }

    fn pledge_all(pledge_id: i32, name: &str, ally_id: i32) -> PledgeShowMemberListAll {
        PledgeShowMemberListAll {
            pledge_id,
            name: name.into(),
            leader_name: "Luzal".into(),
            crest_id: 5,
            level: 3,
            castle_id: 2,
            hideout_id: 4,
            my_level: 52,
            dissolving: 1,
            ally_id,
            ally_name: "Aliança".into(),
            ally_crest_id: 8,
            at_war: false,
            members: vec![member("A", 40), member("A", 41), member("B", 30)],
        }
    }

    #[test]
    fn the_member_list_needs_a_clan_and_a_name_registers_the_clan_and_merges_rows_by_name() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::PledgeShowMemberListAll(pledge_all(0, "Canastra", 0)));
        world.apply_world(&WorldEvent::PledgeShowMemberListAll(pledge_all(33, "", 0)));
        assert!(world.pledge_window.info.is_none() && world.pledge_window.members.is_empty() && world.pledges.is_empty());
        world.apply_world(&WorldEvent::PledgeShowMemberListAll(pledge_all(33, "Canastra", 7)));
        assert_eq!(world.pledge_window.members.iter().map(|m| (m.name.as_str(), m.level)).collect::<Vec<_>>(), [("A", 41), ("B", 30)]);
        assert_eq!((world.pledges[&33].crest_id, world.pledges[&33].ally_id, world.pledges[&33].ally_crest_id), (5, 7, 8));
        assert!(world.requests.contains(&NetCommand::RequestPledgeInfo { pledge_id: 33 }));
        assert!(world.requests.contains(&NetCommand::RequestPledgeCrest { crest_id: 5 }));
    }

    #[test]
    fn the_info_update_zeroes_the_ally_of_the_clan_record_and_refreshes_the_window_header() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::PledgeShowMemberListAll(pledge_all(33, "Canastra", 7)));
        world.requests.clear();
        world.apply_world(&WorldEvent::PledgeShowInfoUpdate(PledgeShowInfoUpdate {
            pledge_id: 33,
            crest_id: 6,
            level: 4,
            castle_id: 0,
            hideout_id: 9,
            leader_level: 60,
            dissolving: 0,
            ally_id: 7,
            ally_name: String::new(),
            ally_crest_id: 8,
            at_war: false,
        }));
        let record = &world.pledges[&33];
        assert_eq!((record.crest_id, record.ally_id, record.ally_crest_id, record.large_crest_id), (6, 0, 0, 0));
        assert!(world.requests.contains(&NetCommand::RequestPledgeInfo { pledge_id: 33 }));
        let info = world.pledge_window.info.as_ref().unwrap();
        assert_eq!((info.crest_id, info.level, info.castle_id, info.hideout_id, info.my_level, info.dissolving), (6, 4, 0, 9, 60, 0));
        assert_eq!((info.name.as_str(), info.leader_name.as_str(), info.pledge_id), ("Canastra", "Luzal", 33));
    }

    #[test]
    fn delete_all_clears_the_rows_and_the_window_header() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::PledgeShowMemberListAll(pledge_all(33, "Canastra", 0)));
        world.apply_world(&WorldEvent::PledgeShowMemberListDeleteAll(PledgeShowMemberListDeleteAll));
        let info = world.pledge_window.info.as_ref().unwrap();
        assert!(world.pledge_window.members.is_empty());
        assert_eq!((info.pledge_id, info.name.as_str(), info.leader_name.as_str(), info.crest_id), (0, "", "", 0));
        assert_eq!((info.level, info.my_level, info.dissolving, info.castle_id, info.hideout_id), (0, 0, 0, 0, 0));
    }

    #[test]
    fn the_skill_list_echo_resends_the_request_with_the_first_two_values_swapped() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::MagicAndSkillList(MagicAndSkillList { first: 1, second: 2, third: 3 }));
        assert_eq!(world.requests, [NetCommand::RequestMagicSkillList { first: 2, second: 1, third: 3 }]);
    }

    #[test]
    fn radar_control_adds_removes_and_clears_the_radar_and_minimap_lists_by_kind() {
        let mut world = World::default();
        let control = |show, kind, x| WorldEvent::RadarControl(RadarControl { show, kind, x, y: 20, z: 30 });
        world.apply_world(&control(0, 0, 1));
        world.apply_world(&control(0, 1, 2));
        world.apply_world(&control(0, 2, 3));
        let radar = &world.windows.radar;
        assert_eq!((radar.enabled, radar.target), (true, Some([3.0, 20.0, 30.0])));
        assert_eq!(radar.points, [[1, 20, 30], [3, 20, 30]]);
        assert_eq!(radar.map_points, [[2, 20, 30], [3, 20, 30]]);
        world.apply_world(&control(1, 2, 3));
        assert_eq!(world.windows.radar.points, [[1, 20, 30]]);
        assert_eq!(world.windows.radar.map_points, [[2, 20, 30]]);
        world.apply_world(&control(0, 5, 9));
        assert_eq!(world.windows.radar.points.len() + world.windows.radar.map_points.len(), 2);
        world.apply_world(&control(2, 0, 0));
        let radar = &world.windows.radar;
        assert_eq!((radar.enabled, radar.target), (false, Some([9.0, 20.0, 30.0])));
        assert!(radar.points.is_empty() && radar.map_points.is_empty());
    }

    #[test]
    fn removing_a_duplicated_point_steps_past_the_row_that_slid_into_its_place() {
        let mut world = World::default();
        let show = |x| WorldEvent::ShowRadar(ShowRadar { first: 0, second: 0, third: x, fourth: 0, fifth: 0 });
        for x in [7, 7, 7, 8] {
            world.apply_world(&show(x));
        }
        world.apply_world(&WorldEvent::DeleteRadar(DeleteRadar { first: 7, second: 0, third: 0 }));
        assert_eq!(world.windows.radar.points, [[7, 0, 0], [8, 0, 0]]);
        world.apply_world(&WorldEvent::ShowRadar(ShowRadar { first: 1, second: 0, third: 8, fourth: 0, fifth: 0 }));
        assert_eq!(world.windows.radar.points, [[7, 0, 0]]);
    }

    #[test]
    fn board_quests_skills_tutorial_and_friends() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::ShowBoard(Some(ShowBoard { links: Vec::new(), html: "<html/>".into() })));
        assert_eq!(world.windows.board.as_ref().unwrap().html, "<html/>");
        world.apply_world(&WorldEvent::ShowBoard(None));
        assert!(world.windows.board.is_none());
        world.apply_world(&WorldEvent::QuestList(QuestList {
            quests: vec![QuestState { quest_id: 101, cond: 3 }],
            extras: Vec::new(),
        }));
        assert_eq!(world.windows.quests[0].quest_id, 101);
        world.apply_world(&WorldEvent::EnchantResult(EnchantResult { result: 0 }));
        assert_eq!(world.windows.enchant_result, Some(0));
        world.apply_world(&WorldEvent::ChooseInventoryItem(ChooseInventoryItem { item_id: 57 }));
        assert_eq!(world.windows.chosen_item_id, Some(57));
        let learnable = AcquireSkillList {
            mode: 0,
            skills: vec![LearnableSkill { id: 3, next_level: 2, max_level: 5, sp_cost: 1, requirements: 0 }],
        };
        world.apply_world(&WorldEvent::AcquireSkillList(learnable));
        assert_eq!(world.windows.learnable.as_ref().unwrap().skills.len(), 1);
        world.apply_world(&WorldEvent::AcquireSkillInfo(AcquireSkillInfo {
            id: 3,
            level: 2,
            sp_cost: 1,
            mode: 0,
            requirements: Vec::new(),
        }));
        assert!(world.windows.skill_info.is_some());
        world.apply_world(&WorldEvent::AcquireSkillDone(AcquireSkillDone));
        assert!(world.windows.skill_info.is_none());
        world.apply_world(&WorldEvent::TutorialShowHtml(TutorialShowHtml { html: "oi".into() }));
        world.apply_world(&WorldEvent::TutorialShowQuestionMark(TutorialShowQuestionMark { mark_id: 1 }));
        world.apply_world(&WorldEvent::TutorialEnableClientEvent(TutorialEnableClientEvent { event_id: 8 }));
        assert_eq!(
            (world.tutorial.html.as_deref(), world.tutorial.question_mark, world.tutorial.client_event),
            (Some("oi"), Some(1), Some(8))
        );
        world.apply_world(&WorldEvent::TutorialCloseHtml(TutorialCloseHtml));
        assert!(world.tutorial.html.is_none());
        world.apply_world(&WorldEvent::FriendAddRequest(FriendAddRequest { requestor_name: "Luzal".into() }));
        assert!(
            matches!(world.dialogs.take(), Some(Dialog::FriendRequest { requestor_name }) if requestor_name == "Luzal")
        );
        world.apply_world(&WorldEvent::FriendList(FriendList {
            friends: vec![Friend { object_id: 11, name: "A".into(), online: true, tail: 0 }],
        }));
        assert_eq!(world.friends.list.len(), 1);
        world.apply_world(&WorldEvent::FriendRecvMsg(FriendRecvMsg {
            unknown: 0,
            receiver: "Eu".into(),
            sender: "A".into(),
            message: "oi".into(),
        }));
        assert_eq!(world.friends.messages[0].message, "oi");
    }

    #[test]
    fn stores_party_rooms_maps_henna_and_pet_inventory() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::PrivateStoreMsgSell(PrivateStoreMsg { object_id: 7, message: "vendo".into() }));
        world.apply_world(&WorldEvent::PrivateStoreMsgBuy(PrivateStoreMsg { object_id: 8, message: "compro".into() }));
        assert_eq!(world.stores.sell_messages[&7], "vendo");
        assert_eq!(world.stores.buy_messages[&8], "compro");
        world.apply_world(&WorldEvent::PrivateStoreListSell(PrivateStoreListSell {
            object_id: 7,
            package_sale: false,
            adena: 1,
            items: Vec::new(),
        }));
        world.apply_world(&WorldEvent::PrivateStoreManageListSell(PrivateStoreManageListSell {
            object_id: 7,
            package_sale: false,
            adena: 1,
            available: Vec::new(),
            selling: Vec::new(),
        }));
        world.apply_world(&WorldEvent::PrivateStoreListBuy(PrivateStoreListBuy {
            object_id: 8,
            adena: 1,
            items: Vec::new(),
        }));
        world.apply_world(&WorldEvent::PrivateStoreManageListBuy(PrivateStoreManageListBuy {
            object_id: 8,
            adena: 1,
            available: Vec::new(),
            buying: Vec::new(),
        }));
        assert!(
            world.stores.sell.is_some()
                && world.stores.manage_sell.is_some()
                && world.stores.buy.is_some()
                && world.stores.manage_buy.is_some()
        );
        world.apply_world(&WorldEvent::PartyMatchList(PartyMatchList { flag: 1, rooms: Vec::new() }));
        world.apply_world(&WorldEvent::PartyMatchDetail(PartyMatchDetail {
            id: 7,
            max_members: 9,
            min_level: 1,
            max_level: 80,
            loot_type: 0,
            location: 0,
            title: "S".into(),
        }));
        assert_eq!(world.windows.party_room.as_ref().unwrap().id, 7);
        assert!(world.windows.party_rooms.is_some());
        world.apply_world(&WorldEvent::ShowMinimap(ShowMinimap { map_id: 1665, flag: 0 }));
        world.apply_world(&WorldEvent::ShowTownMap(ShowTownMap { texture: "t".into(), x: 1, y: 2 }));
        world.apply_world(&WorldEvent::RadarControl(RadarControl { show: 0, kind: 1, x: 1, y: 2, z: 3 }));
        assert_eq!(world.windows.minimap.unwrap().map_id, 1665);
        assert_eq!(world.windows.town_map.as_ref().unwrap().x, 1);
        assert_eq!(world.windows.radar.map_points, [[1, 2, 3]]);
        world.apply_world(&WorldEvent::HennaInfo(HennaInfo {
            int_: 1,
            str_: 0,
            con: 0,
            men: 0,
            dex: 0,
            wit: 0,
            slots: 3,
            symbols: Vec::new(),
        }));
        world.apply_world(&WorldEvent::HennaEquipList(HennaEquipList { adena: 1, slots: 3, offers: Vec::new() }));
        assert_eq!(world.henna.info.as_ref().unwrap().int_, 1);
        assert!(world.henna.equip_list.is_some() && world.henna.item_info.is_none());
        world.apply_world(&WorldEvent::PackageToList(PackageToList { targets: Vec::new() }));
        world.apply_world(&WorldEvent::PackageSendableList(PackageSendableList {
            target_id: 9,
            adena: 0,
            items: Vec::new(),
        }));
        world.apply_world(&WorldEvent::MultiSellList(MultiSellList {
            list_id: 1,
            page: 1,
            finished: true,
            page_size: 40,
            entries: Vec::new(),
        }));
        world.apply_world(&WorldEvent::MultiSellList(MultiSellList {
            list_id: 999,
            page: 2,
            finished: true,
            page_size: 1,
            entries: Vec::new(),
        }));
        assert_eq!(world.windows.multisell.as_ref().unwrap().list_id, 1);
        world.apply_world(&WorldEvent::BuyListSeed(BuyListSeed { money: 1, manor_id: 1, items: Vec::new() }));
        world.apply_world(&WorldEvent::ShopPreviewList(ShopPreviewList {
            header: 0,
            money: 1,
            list_id: 1,
            items: Vec::new(),
        }));
        world.apply_world(&WorldEvent::ShopPreviewInfo(ShopPreviewInfo { slots: [0; 16] }));
        assert!(world.windows.package_targets.is_some() && world.windows.package_items.is_some());
        assert!(world.windows.multisell.is_some() && world.windows.seeds.is_some());
        assert!(world.windows.preview_list.is_some() && world.windows.preview.is_some());
        let pet_item = InventoryItem {
            type1: 1,
            object_id: 1001,
            item_id: 57,
            count: 5,
            type2: 2,
            custom_type1: 0,
            equipped: false,
            body_part: 0,
            enchant: 0,
            custom_type2: 0,
        };
        world.apply_world(&WorldEvent::PetItemList(PetItemList { items: vec![pet_item] }));
        assert_eq!(world.pet_inventory.items.len(), 1);
        world.apply_world(&WorldEvent::PetInventoryUpdate(InventoryUpdate {
            changes: vec![ItemChange { change: l2_net::game::packets::ITEM_REMOVED, item: pet_item }],
        }));
        assert!(world.pet_inventory.items.is_empty());
    }
}

#[cfg(test)]
mod frente_p_rare_tests {
    use super::*;
    use l2_net::game::SsqPage;
    use l2_net::game::packets::SSQ_SKY_DAWN;

    #[test]
    fn siege_recipes_and_rare_windows_are_kept() {
        let mut world = World::default();
        let info = CastleSiegeInfo {
            castle_id: 3,
            owner_flags: 0,
            owner_id: 0,
            pledge_name: "NPC".into(),
            leader_name: String::new(),
            ally_id: 0,
            ally_name: String::new(),
            now: 1,
            siege_time: 2,
            choices: Vec::new(),
        };
        world.apply_world(&WorldEvent::CastleSiegeInfo(info));
        let list = SiegeClanList { castle_id: 3, header: [0, 1, 0], total: 0, clans: Vec::new() };
        world.apply_world(&WorldEvent::CastleSiegeAttackerList(list.clone()));
        world.apply_world(&WorldEvent::CastleSiegeDefenderList(list));
        assert_eq!(world.siege.info.as_ref().unwrap().castle_id, 3);
        assert!(world.siege.attackers.is_some() && world.siege.defenders.is_some());
        world.apply_world(&WorldEvent::RecipeBookItemList(RecipeBookItemList {
            common: true,
            max_mp: 500,
            recipes: vec![(10, 1)],
        }));
        world.apply_world(&WorldEvent::RecipeItemMakeInfo(RecipeItemMakeInfo {
            recipe_id: 10,
            common: true,
            mp: 1,
            max_mp: 2,
            status: 1,
        }));
        world.apply_world(&WorldEvent::RecipeShopManageList(RecipeShopManageList {
            common: true,
            mp: 1,
            max_mp: 2,
            recipes: Vec::new(),
            items: Vec::new(),
        }));
        world.apply_world(&WorldEvent::RecipeShopSellList(RecipeShopSellList {
            manufacturer_id: 7,
            mp: 1,
            max_mp: 2,
            adena: 3,
            items: Vec::new(),
        }));
        world.apply_world(&WorldEvent::RecipeShopItemInfo(RecipeShopItemInfo {
            manufacturer_id: 7,
            recipe_id: 10,
            mp: 1,
            max_mp: 2,
            status: 1,
            cost: 0,
        }));
        world.apply_world(&WorldEvent::RecipeShopMsg(RecipeShopMsg { object_id: 7, message: "crafto".into() }));
        assert_eq!(world.recipes.book.as_ref().unwrap().recipes, vec![(10, 1)]);
        assert_eq!(world.recipes.make_info.unwrap().recipe_id, 10);
        assert!(world.recipes.shop_manage.is_some() && world.recipes.shop_sell.is_some());
        assert_eq!(world.recipes.shop_item.unwrap().manufacturer_id, 7);
        assert_eq!(world.recipes.shop_messages[&7], "crafto");
        world.apply_world(&WorldEvent::ShowCalculator(ShowCalculator { calculator_id: 4393 }));
        world.apply_world(&WorldEvent::MonRaceInfo(MonRaceInfo { state: 0, second: 0, monsters: Vec::new() }));
        world.apply_world(&WorldEvent::ShowXMasSeal(ShowXMasSeal { item_id: 5555 }));
        world.apply_world(&WorldEvent::SsqStatus(SsqStatus { page: 9, period: 2, body: SsqPage::Unknown }));
        world.apply_world(&WorldEvent::AgitDecoInfo(AgitDecoInfo { agit_id: 21 }));
        world.apply_world(&WorldEvent::SsqInfo(SsqInfo { sky: SSQ_SKY_DAWN }));
        assert_eq!(
            (world.rare.calculator, world.rare.xmas_seal, world.rare.agit_deco, world.rare.ssq_sky),
            (Some(4393), Some(5555), Some(21), Some(SSQ_SKY_DAWN))
        );
        assert!(world.rare.race.is_some());
        assert_eq!(world.rare.ssq_status.as_ref().unwrap().page, 9);
    }
}

#[cfg(test)]
mod anim_trigger_tests {
    use super::tests::npc;
    use super::*;
    use crate::anim::{AnimEvent, Hit as AnimHit, SeqInfo, Sequences, SkillData, Tables, Trigger, WaitChange};
    use l2_dat::animnames::AnimKind;
    use l2_net::game::packets::Hit;
    use l2_net::game::{
        Attack, ChairSit, ChangeWaitType, Die, GetItem, MagicSkillCanceled, MagicSkillUse, Revive, SocialAction,
    };

    fn world() -> World {
        let mut world = World::default();
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        world.apply_world(&WorldEvent::NpcInfo(npc(2)));
        world
    }

    fn triggers(world: &World, id: i32) -> &[Trigger] {
        &world.entities[&id].anim.pending
    }

    #[test]
    fn spawning_a_pawn_queues_the_wait_and_reseeing_it_keeps_the_player() {
        let mut world = world();
        assert_eq!(triggers(&world, 1), [Trigger::Spawn]);
        world.entities.get_mut(&1).unwrap().anim.pending.clear();
        world.entities.get_mut(&1).unwrap().anim.move_timer = 0.7;
        world.apply_world(&WorldEvent::NpcInfo(npc(1)));
        assert!(triggers(&world, 1).is_empty());
        assert_eq!(world.entities[&1].anim.move_timer, 0.7);
        world.apply_world(&WorldEvent::SpawnItem(l2_net::game::SpawnItem {
            object_id: 50,
            item_id: 57,
            x: 0,
            y: 0,
            z: 0,
            stackable: true,
            count: 1,
            heading: 16384,
        }));
        assert!(triggers(&world, 50).is_empty());
    }

    #[test]
    fn my_attack_sets_the_attack_intend_until_a_move_or_a_new_target() {
        let mut world = world();
        world.me = world.entities.remove(&1);
        let attack = |target_id| WorldEvent::Attack(Attack { attacker_id: 1, x: 0, y: 0, z: 0, hits: vec![Hit { target_id, damage: 1, flags: 0 }] });
        world.apply_world(&attack(3));
        assert!(!world.attack_intend);
        world.apply_world(&attack(2));
        assert!(world.attack_intend);
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation { object_id: 2, to_x: 0, to_y: 0, to_z: 0, x: 0, y: 0, z: 0 }));
        assert!(world.attack_intend);
        world.apply_world(&WorldEvent::MoveToLocation(MoveToLocation { object_id: 1, to_x: 0, to_y: 0, to_z: 0, x: 0, y: 0, z: 0 }));
        assert!(!world.attack_intend);
        world.apply_world(&attack(2));
        world.apply_world(&WorldEvent::MyTargetSelected(l2_net::game::MyTargetSelected { object_id: 2, color: 0 }));
        assert!(!world.attack_intend);
        world.apply_world(&attack(2));
        assert!(world.attack_intend);
        world.apply_world(&WorldEvent::AttackDeadTargetNoop(AttackFeedbackNoop));
        assert!(!world.attack_intend);
        for event in [
            WorldEvent::AttackOutOfRangeNoop(AttackFeedbackNoop),
            WorldEvent::AttackInCoolTimeNoop(AttackFeedbackNoop),
            WorldEvent::ActionFailed(ActionFailed),
        ] {
            world.apply_world(&attack(2));
            assert!(world.attack_intend);
            world.apply_world(&event);
            assert!(!world.attack_intend, "{event:?}");
        }
        world.me.as_mut().unwrap().dead = true;
        world.apply_world(&attack(2));
        assert!(!world.attack_intend);
    }

    #[test]
    fn attack_queues_the_first_hit_for_a_living_attacker_only() {
        let mut world = world();
        let hits = vec![
            Hit { target_id: 2, damage: 40, flags: 0x20 | 0x10 | 0x03 },
            Hit { target_id: 3, damage: 0, flags: 0x80 },
        ];
        world.apply_world(&WorldEvent::Attack(Attack { attacker_id: 1, x: 0, y: 0, z: 0, hits: hits.clone() }));
        let expected = AnimHit {
            target: 2,
            damage: 40,
            miss: false,
            critical: true,
            shield: false,
            spirit: true,
            soulshot_grade: 3,
        };
        assert_eq!(triggers(&world, 1), [Trigger::Spawn, Trigger::Attack(expected)]);
        world.apply_world(&WorldEvent::Die(Die {
            object_id: 1,
            to_village: false,
            to_hideout: false,
            to_castle: false,
            to_siege_hq: false,
            sweepable: false,
            to_fixed_spot: false,
        }));
        world.apply_world(&WorldEvent::Attack(Attack { attacker_id: 1, x: 0, y: 0, z: 0, hits }));
        assert_eq!(triggers(&world, 1).last(), Some(&Trigger::Die));
        assert!(!world.entities[&1].in_combat);
    }

    #[test]
    fn wait_type_changes_queue_sit_stand_and_fake_death() {
        let mut world = world();
        let change = |kind: i32| ChangeWaitType { object_id: 1, kind, sitting: kind == 0, x: 0, y: 0, z: 0 };
        world.apply_world(&WorldEvent::ChangeWaitType(change(0)));
        assert_eq!(world.entities[&1].wait_type(), WaitType::Sit);
        world.apply_world(&WorldEvent::ChangeWaitType(change(1)));
        assert_eq!(world.entities[&1].wait_type(), WaitType::Stand);
        world.apply_world(&WorldEvent::ChangeWaitType(change(2)));
        assert_eq!(world.entities[&1].wait_type(), WaitType::FakeDead);
        world.apply_world(&WorldEvent::ChangeWaitType(change(3)));
        assert_eq!(world.entities[&1].wait_type(), WaitType::Stand);
        world.apply_world(&WorldEvent::ChangeWaitType(change(9)));
        assert_eq!(
            triggers(&world, 1),
            [
                Trigger::Spawn,
                Trigger::ChangeWaitType { change: WaitChange::Sit, from: WaitType::Stand },
                Trigger::ChangeWaitType { change: WaitChange::Stand, from: WaitType::Sit },
                Trigger::ChangeWaitType { change: WaitChange::FakeDeath, from: WaitType::Stand },
                Trigger::ChangeWaitType { change: WaitChange::FakeDeathStand, from: WaitType::FakeDead },
            ]
        );
        world.apply_world(&WorldEvent::ChairSit(ChairSit { object_id: 1, static_object_id: 77 }));
        assert_eq!(world.entities[&1].wait_type(), WaitType::ChairSit);
        assert_eq!(triggers(&world, 1).last(), Some(&Trigger::ChairSit));
    }

    #[test]
    fn die_revive_social_pick_and_magic_queue_their_triggers() {
        let mut world = world();
        world.apply_world(&WorldEvent::Revive(Revive { object_id: 1 }));
        world.apply_world(&WorldEvent::SocialAction(SocialAction { object_id: 1, action: 2 }));
        world.apply_world(&WorldEvent::SocialAction(SocialAction { object_id: 1, action: 0 }));
        world.apply_world(&WorldEvent::GetItem(GetItem { player_id: 1, object_id: 2, x: 0, y: 0, z: 0 }));
        world.apply_world(&WorldEvent::NpcInfo(npc(2)));
        world.apply_world(&WorldEvent::MagicSkillUse(MagicSkillUse {
            caster_id: 1,
            target_id: 2,
            skill_id: 1204,
            skill_level: 2,
            hit_time: 4000,
            reuse_delay: 0,
            x: 0,
            y: 0,
            z: 0,
            extra: Vec::new(),
            positions: Vec::new(),
        }));
        assert_eq!(world.entities[&1].casting.as_ref().unwrap().skill_level, 2);
        world.apply_world(&WorldEvent::MagicSkillCanceled(MagicSkillCanceled { object_id: 1 }));
        assert_eq!(
            triggers(&world, 1),
            [
                Trigger::Spawn,
                Trigger::Revive,
                Trigger::Social(2),
                Trigger::PicItem,
                Trigger::MagicSkillUse { skill_id: 1204, level: 2, hit_time: 4000, target: 2 },
                Trigger::MagicCancel,
            ]
        );
    }

    struct Gremlin;

    impl Sequences for Gremlin {
        fn seq(&self, name: &str) -> Option<SeqInfo> {
            match name.to_ascii_lowercase().as_str() {
                "wait" => Some(SeqInfo::new(61, 30.0)),
                "run" => Some(SeqInfo::new(17, 30.0)),
                "walk" => Some(SeqInfo::new(23, 30.0)),
                "atkwait" => Some(SeqInfo::new(41, 30.0)),
                "atk01" => Some(SeqInfo::with_shots(55, 30.0, &[0.6])),
                "death" => Some(SeqInfo::new(64, 30.0)),
                "deathwait" => Some(SeqInfo::new(1, 30.0)),
                _ => None,
            }
        }
    }

    impl Tables for Gremlin {
        fn skill(&self, _skill_id: i32, _level: i32) -> Option<SkillData> {
            None
        }

        fn npc_skill_anim(&self, _npc_class_id: i32, _skill_id: i32) -> Option<String> {
            None
        }
    }

    fn gremlin_set() -> AnimSet {
        let mut set = AnimSet::default();
        for (kind, name) in [
            (AnimKind::Wait, "wait"),
            (AnimKind::Run, "run"),
            (AnimKind::Walk, "walk"),
            (AnimKind::AtkWait, "atkwait"),
            (AnimKind::Atk01, "atk01"),
            (AnimKind::Atk02, "atk01"),
            (AnimKind::Atk03, "atk01"),
            (AnimKind::Death, "death"),
            (AnimKind::DeathWait, "deathwait"),
        ] {
            set.names_mut(kind)[0] = Some(name.into());
        }
        set
    }

    struct Shots;

    impl Sequences for Shots {
        fn seq(&self, name: &str) -> Option<SeqInfo> {
            Gremlin.seq(name)
        }
    }

    impl Tables for Shots {
        fn skill(&self, skill_id: i32, _level: i32) -> Option<SkillData> {
            match skill_id {
                2039 => Some(SkillData { animation: "L".into(), cast_style: 0, visual_effect_lead: None }),
                1204 => Some(SkillData { animation: "A".into(), cast_style: 1, visual_effect_lead: None }),
                _ => None,
            }
        }

        fn npc_skill_anim(&self, _npc_class_id: i32, _skill_id: i32) -> Option<String> {
            None
        }
    }

    #[test]
    fn a_soulshot_magic_skill_use_keeps_moving_and_a_cast_stops() {
        let mut world = world();
        let set = gremlin_set();
        world.entities.get_mut(&1).unwrap().tick_anim(0.0, 0, &set, &Shots);
        let walk = MoveToLocation { object_id: 1, to_x: 500, to_y: 0, to_z: 0, x: 0, y: 0, z: 0 };
        world.apply_world(&WorldEvent::MoveToLocation(walk));
        let shot = MagicSkillUse { skill_id: 2039, skill_level: 1, hit_time: 0, target_id: 1, ..super::tests::cast(1, 1) };
        world.apply_world(&WorldEvent::MagicSkillUse(shot));
        let events = world.entities.get_mut(&1).unwrap().tick_anim(0.0, 0, &set, &Shots);
        assert_eq!(events, [AnimEvent::TransientEffect { skill_id: 2039, level: 1, target: 1 }]);
        assert!(world.entities[&1].dest.is_some());
        world.apply_world(&WorldEvent::MagicSkillUse(super::tests::cast(1, 1)));
        world.entities.get_mut(&1).unwrap().tick_anim(0.0, 0, &set, &Shots);
        assert!(world.entities[&1].dest.is_none());
    }

    #[test]
    fn entity_tick_anim_drives_the_pawn_from_the_queued_packets() {
        let mut world = world();
        let set = gremlin_set();
        let events = world.entities.get_mut(&1).unwrap().tick_anim(0.0, 0, &set, &Gremlin);
        assert!(events.is_empty());
        assert_eq!(world.entities[&1].anim.current(), Some("wait"));
        world.apply_world(&WorldEvent::MoveToLocation(l2_net::game::MoveToLocation {
            object_id: 1,
            to_x: 500,
            to_y: 0,
            to_z: 0,
            x: 0,
            y: 0,
            z: 0,
        }));
        world.entities.get_mut(&1).unwrap().tick_anim(0.1, 0, &set, &Gremlin);
        assert_eq!(world.entities[&1].anim.current(), Some("run"));
        let hit = Hit { target_id: 2, damage: 10, flags: 0 };
        world.apply_world(&WorldEvent::Attack(Attack { attacker_id: 1, x: 0, y: 0, z: 0, hits: vec![hit] }));
        world.entities.get_mut(&1).unwrap().tick_anim(0.0, 0, &set, &Gremlin);
        assert_eq!(world.entities[&1].anim.current(), Some("atk01"));
        let mut fired = Vec::new();
        for _ in 0..40 {
            fired.extend(world.entities.get_mut(&1).unwrap().tick_anim(0.05, 0, &set, &Gremlin));
        }
        assert_eq!(fired, [AnimEvent::AttackShot(AnimHit { target: 2, damage: 10, ..AnimHit::default() })]);
        assert_eq!(world.entities[&1].anim.current(), Some("atkwait"));
        world.apply_world(&WorldEvent::Die(Die {
            object_id: 1,
            to_village: false,
            to_hideout: false,
            to_castle: false,
            to_siege_hq: false,
            sweepable: false,
            to_fixed_spot: false,
        }));
        world.entities.get_mut(&1).unwrap().tick_anim(0.0, 0, &set, &Gremlin);
        assert_eq!(world.entities[&1].anim.current(), Some("death"));
        for _ in 0..60 {
            world.entities.get_mut(&1).unwrap().tick_anim(0.05, 0, &set, &Gremlin);
        }
        assert_eq!(world.entities[&1].anim.current(), Some("deathwait"));
    }

    #[test]
    fn hero_list_is_kept_for_the_monument_window() {
        let mut world = World::default();
        let list = l2_net::game::HeroList { entries: vec![l2_net::game::HeroEntry { names: ["a".into(), "b".into(), "c".into()], values: [1, 2, 3, 4] }] };
        world.apply_world(&WorldEvent::HeroList(list.clone()));
        assert_eq!(world.windows.hero_tower.list, Some(list));
    }

    #[test]
    fn manor_events_store_the_castle_list_and_the_three_information_pages() {
        use l2_net::game::{CropInfo, ManorDefaultInfo, ManorEntry, ManorList, SeedInfo};
        let mut world = World::default();
        let entries = vec![ManorEntry { id: 1, name: "Gludio".into() }, ManorEntry { id: 2, name: "Dion".into() }];
        world.apply_world(&WorldEvent::SendManorList(ManorList { entries: entries.clone() }));
        assert_eq!(world.windows.manor.castles, entries);
        let seed = SeedInfo { flag: 1, values: [2, 0], entries: Vec::new() };
        world.apply_world(&WorldEvent::ShowSeedInfo(seed.clone()));
        let crop = CropInfo { flag: 0, values: [1, 0], entries: Vec::new() };
        world.apply_world(&WorldEvent::ShowCropInfo(crop.clone()));
        let default = ManorDefaultInfo { flag: 1, entries: Vec::new() };
        world.apply_world(&WorldEvent::ShowManorDefaultInfo(default.clone()));
        assert_eq!(world.windows.manor.seed_info, Some(seed));
        assert_eq!(world.windows.manor.crop_info, Some(crop));
        assert_eq!(world.windows.manor.default_info, Some(default));
        let setting = l2_net::game::SeedSetting { value: 3, entries: Vec::new() };
        world.apply_world(&WorldEvent::ShowSeedSetting(setting.clone()));
        assert_eq!(world.windows.manor.seed_setting, Some(setting));
        let crop_setting = l2_net::game::CropSettingInfo { value: 4, entries: Vec::new() };
        world.apply_world(&WorldEvent::ShowCropSetting(crop_setting.clone()));
        assert_eq!(world.windows.manor.crop_setting, Some(crop_setting));
        let sell = l2_net::game::SellCropList { value: 1, entries: Vec::new() };
        world.apply_world(&WorldEvent::ShowSellCropList(sell.clone()));
        assert_eq!(world.windows.manor.sell_crop_list, Some(sell));
        let detail = l2_net::game::ProcureCropDetail { value: 5, entries: Vec::new() };
        world.apply_world(&WorldEvent::ShowProcureCropDetail(detail.clone()));
        assert_eq!(world.windows.manor.procure_crop_detail, Some(detail));
        world.apply_world(&WorldEvent::SendManorList(ManorList { entries: Vec::new() }));
        assert!(world.windows.manor.castles.is_empty());
    }


    #[test]
    fn skill_list_shortcut_init_mpcc_and_large_crest_follow_the_engine() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::SkillList(SkillList { skills: Vec::new() }));
        world.apply_world(&WorldEvent::ShortCutInit(ShortCutInit { shortcuts: Vec::new() }));
        assert_eq!(world.take_requests(), vec![NetCommand::RequestSkillCoolTime, NetCommand::RequestSkillCoolTime]);
        world.apply_world(&WorldEvent::AskJoinMpcc(AskJoinMpcc { name: "Leader".into() }));
        assert_eq!(world.dialogs.take(), Some(Dialog::JoinMpcc { name: "Leader".into() }));
        world.register_pledge(5, 0, 0, 0, 9);
        world.take_requests();
        world.register_pledge(5, 0, 0, 0, 9);
        assert_eq!(world.take_requests(), vec![NetCommand::ExPledgeCrestLarge { crest_id: 9 }]);
        world.apply_world(&WorldEvent::PledgeCrestLarge(PledgeCrestLarge { values: [0, 9, 2], data: vec![1, 2] }));
        world.register_pledge(5, 0, 0, 0, 9);
        assert!(world.take_requests().is_empty());
        world.register_pledge(0, 0, 0, 0, 10);
        assert!(!world.take_requests().contains(&NetCommand::ExPledgeCrestLarge { crest_id: 10 }));
    }
}
