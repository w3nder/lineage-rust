use l2_dat::animnames::{AnimKind, AnimSet, FishingKind, HideWeapons};

use super::channel::{Channel, Sequences};
use super::notify::NotifyKind;
use super::names::{
    PawnView, Physics, WaitType, cur_atk_anim_name, cur_turn_anim_name, cur_wait_anim_name, death_anim_name,
    is_cast_anim, random_atk_anim_kind, script_cur_wait_anim_name, social_anim_name,
};
use super::skill::{SkillTimeline, StageSeq, cast_lead, skill_anim, skill_timeline};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpdateMovementAnim {
    #[default]
    None,
    Once,
    Force,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hit {
    pub target: i32,
    pub damage: i32,
    pub miss: bool,
    pub critical: bool,
    pub shield: bool,
    pub spirit: bool,
    pub soulshot_grade: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackKind {
    Melee,
    Bow,
    Throw,
    Shot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attack {
    pub kind: AttackKind,
    pub status: u8,
    pub hit: Hit,
    pub shots: usize,
    pub fired: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Magic {
    pub skill_id: i32,
    pub level: i32,
    pub target: i32,
    pub stages: Vec<Option<String>>,
    pub timeline: SkillTimeline,
    pub elapsed: f32,
    pub stage: Option<usize>,
    pub associated: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Action {
    #[default]
    None,
    Attack(Attack),
    Magic(Magic),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitChange {
    Sit,
    Stand,
    FakeDeath,
    FakeDeathStand,
}

impl WaitChange {
    pub fn code(self) -> u8 {
        match self {
            WaitChange::Sit => 0,
            WaitChange::Stand => 1,
            WaitChange::FakeDeath => 2,
            WaitChange::FakeDeathStand => 3,
        }
    }

    pub fn from_packet(kind: i32) -> Option<WaitChange> {
        match kind {
            0 => Some(WaitChange::Sit),
            1 => Some(WaitChange::Stand),
            2 => Some(WaitChange::FakeDeath),
            3 => Some(WaitChange::FakeDeathStand),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Trigger {
    Spawn,
    Attack(Hit),
    ChangeWaitType { change: WaitChange, from: WaitType },
    Die,
    Revive,
    Replay,
    Social(u32),
    PicItem,
    ChairSit,
    PlayAnim(String),
    MagicSkillUse { skill_id: i32, level: i32, hit_time: i32, target: i32 },
    MagicSkillUseOnMover { skill_id: i32, level: i32, hit_time: i32, target: i32 },
    MagicSkillLaunched { skill_id: i32, targets: Vec<i32> },
    MagicCancel,
    ActionStop,
    FishingStart,
    FishingPull,
    FishingEnd,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkillData {
    pub animation: String,
    pub cast_style: i32,
    pub visual_effect_lead: Option<f32>,
}

pub trait Tables: Sequences {
    fn skill(&self, skill_id: i32, level: i32) -> Option<SkillData>;
    fn npc_skill_anim(&self, npc_class_id: i32, skill_id: i32) -> Option<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnimEvent {
    AttackShot(Hit),
    ArrowReleased(Hit),
    MagicShot { skill_id: i32, level: i32, target: i32, shot_time: f32, associated: Vec<i32> },
    MagicPreShot { skill_id: i32, level: i32, target: i32, shot_time: f32 },
    Social { id: u32, duration: f32 },
    TransientEffect { skill_id: i32, level: i32, target: i32 },
    CastStart { skill_id: i32, level: i32, target: i32, shot_time: f32 },
    MagicStop,
    MoveStop,
    Notify(NotifyKind),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimPlayer {
    pub channel: Channel,
    pub update_movement_anim: UpdateMovementAnim,
    pub action: Action,
    pub move_timer: f32,
    pub last_yaw: f32,
    pub hide_weapons: HideWeapons,
    pub pending: Vec<Trigger>,
    pub npc_class_id: i32,
    pub npc_speed: f32,
    pub seed: u32,
    pub ride_attack: Option<AnimKind>,
}

impl Default for AnimPlayer {
    fn default() -> Self {
        AnimPlayer {
            channel: Channel::default(),
            update_movement_anim: UpdateMovementAnim::None,
            action: Action::None,
            move_timer: 0.0,
            last_yaw: 0.0,
            hide_weapons: HideWeapons::default(),
            pending: Vec::new(),
            npc_class_id: 0,
            npc_speed: 0.0,
            seed: 0x2545_f491,
            ride_attack: None,
        }
    }
}

pub const LOBBY_CLICK_SOCIAL: usize = 3;
pub const ENTER_EVENT_TWEEN: f32 = f32::from_bits(0x3dcc_cccd);
pub const SWORD_RATE_BONUS: f32 = 1.1;
pub const TURN_THRESHOLD_UNITS_PER_SECOND: f32 = 1000.0;
pub const WALK_FRAME_SECONDS: f32 = 1.0 / 30.0;
pub const ATTACK_EFF_TOLERANCE: f32 = 0.1;
pub const ATTACK_EFF_MAX: f32 = 0.9;

impl AnimPlayer {
    pub fn with_seed(seed: u32) -> Self {
        AnimPlayer { seed, ..Default::default() }
    }

    pub fn current(&self) -> Option<&str> {
        self.channel.sequence.as_deref()
    }

    fn is_current(&self, name: Option<&str>) -> bool {
        match (self.current(), name) {
            (Some(current), Some(name)) => current.eq_ignore_ascii_case(name),
            (None, None) => true,
            _ => false,
        }
    }

    fn random(&mut self) -> u32 {
        let mut x = self.seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.seed = x;
        x
    }

    pub fn push(&mut self, trigger: Trigger) {
        self.pending.push(trigger);
    }

    pub fn play_anim(
        &mut self,
        name: Option<&str>,
        rate: f32,
        tween: f32,
        looping: bool,
        update: UpdateMovementAnim,
        seqs: &dyn Sequences,
    ) -> bool {
        let Some(name) = name else { return false };
        self.update_movement_anim = update;
        self.hide_weapons = HideWeapons::default();
        let played = seqs.seq(name).is_some_and(|seq| self.channel.play(name, &seq, rate, tween, looping));
        if !played {
            self.update_movement_anim = UpdateMovementAnim::None;
        }
        played
    }

    fn fix_rate(&mut self, rate: f32) {
        if self.channel.sequence.is_some() && self.channel.rate != rate {
            self.channel.set_rate(rate);
        }
    }

    pub fn tick(
        &mut self,
        dt: f32,
        view: &PawnView,
        set: &AnimSet,
        tables: &dyn Tables,
        moving: bool,
        yaw: f32,
    ) -> Vec<AnimEvent> {
        let mut events = Vec::new();
        let pending = std::mem::take(&mut self.pending);
        for trigger in pending {
            self.apply(trigger, view, set, tables, &mut events);
        }
        self.tick_special(dt, view, set, tables, moving, yaw, &mut events);
        let step = self.channel.advance(dt);
        for notify in step.fired {
            match notify.kind {
                NotifyKind::AttackShot => self.on_attack_shot(&mut events),
                NotifyKind::AttackPreShot => self.on_attack_pre_shot(&mut events),
                kind => events.push(AnimEvent::Notify(kind)),
            }
        }
        if step.ended {
            self.anim_end(view, set, tables);
        }
        events
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tick_special(
        &mut self,
        dt: f32,
        view: &PawnView,
        set: &AnimSet,
        tables: &dyn Tables,
        moving: bool,
        yaw: f32,
        events: &mut Vec<AnimEvent>,
    ) {
        self.update_movement_animation(dt, view, set, tables, moving, yaw);
        self.naction_process(dt, view, set, tables, events);
    }

    pub fn update_movement_animation(
        &mut self,
        dt: f32,
        view: &PawnView,
        set: &AnimSet,
        seqs: &dyn Sequences,
        moving: bool,
        yaw: f32,
    ) {
        let wyvern = view.ride_pawn_type == Some(2);
        if (!wyvern && view.dead) || view.same_anim {
            return;
        }
        let rate = view.non_attack_speed_rate;
        let name = |kind: AnimKind| set.name(kind, view.weapon_type);
        if moving {
            match view.physics {
                Physics::Other | Physics::Falling => {
                    let fall = name(AnimKind::Fall).or_else(|| name(AnimKind::Wait));
                    self.play_anim(fall, rate, 0.1, true, UpdateMovementAnim::None, seqs);
                    self.fix_rate(rate);
                }
                Physics::Swimming if !view.hit_ground_in_water && name(AnimKind::Swim).is_some() => {
                    let swim = name(AnimKind::Swim);
                    if !self.is_current(swim) {
                        self.play_anim(swim, rate, 0.5, true, UpdateMovementAnim::None, seqs);
                    }
                    self.fix_rate(rate);
                }
                _ => {
                    let tween = if self.is_current(name(AnimKind::Swim)) { 0.4 } else { 0.1 };
                    self.move_timer += dt;
                    let walk = name(AnimKind::Walk);
                    let run = view.move_type == super::names::MoveType::Fast
                        && (view.npc
                            || walk.and_then(|walk| seqs.seq(walk)).is_none()
                            || view.walk_anim_frame_in_move as f32 * WALK_FRAME_SECONDS < self.move_timer);
                    let npc_speed = if view.npc && view.npc_speed != 0.0 { view.npc_speed } else { 1.0 };
                    let (kind, multiplier) = if run {
                        (AnimKind::Run, set.rate(AnimKind::Run, view.weapon_type) * npc_speed * rate)
                    } else {
                        (AnimKind::Walk, set.rate(AnimKind::Walk, view.weapon_type) * npc_speed * rate)
                    };
                    let target = name(kind);
                    if !self.is_current(target) {
                        self.play_anim(target, multiplier, tween, true, UpdateMovementAnim::None, seqs);
                    }
                    self.fix_rate(multiplier);
                }
            }
            self.last_yaw = yaw;
            return;
        }
        if self.update_movement_anim == UpdateMovementAnim::None {
            if view.physics == Physics::Other {
                let fall = name(AnimKind::DamageFly).or_else(|| name(AnimKind::Fall)).or_else(|| name(AnimKind::Wait));
                if fall.is_some() {
                    self.play_anim(fall, rate, 0.1, true, UpdateMovementAnim::None, seqs);
                }
            } else if view.physics == Physics::Swimming && !view.hit_ground_in_water && !view.dead {
                let swim_wait = name(AnimKind::SwimWait);
                if swim_wait.is_some() {
                    if !self.is_current(swim_wait) {
                        self.play_anim(swim_wait, rate, 0.2, true, UpdateMovementAnim::None, seqs);
                    }
                    self.fix_rate(rate);
                }
            } else {
                let delta = yaw - self.last_yaw;
                let turn = if dt != 0.0 { delta / (dt * TURN_THRESHOLD_UNITS_PER_SECOND) } else { delta };
                if !(-1.0..=1.0).contains(&turn) {
                    let turn_name = cur_turn_anim_name(set, view);
                    if turn_name.is_some() && !self.is_current(turn_name) {
                        self.play_anim(turn_name, rate, 0.1, true, UpdateMovementAnim::None, seqs);
                    }
                } else {
                    let wait = cur_wait_anim_name(set, view);
                    if !(self.move_timer == 0.0 && self.is_current(wait)) {
                        self.play_anim(wait, rate, 0.2, true, UpdateMovementAnim::None, seqs);
                    }
                }
                self.fix_rate(rate);
            }
        }
        self.move_timer = 0.0;
        self.last_yaw = yaw;
    }

    pub fn naction_process(&mut self, dt: f32, view: &PawnView, set: &AnimSet, seqs: &dyn Sequences, events: &mut Vec<AnimEvent>) {
        match std::mem::take(&mut self.action) {
            Action::None => {}
            Action::Attack(mut attack) => {
                let keep = match attack.kind {
                    AttackKind::Melee => self.sword_attack_process(&mut attack, view, set, seqs),
                    AttackKind::Bow | AttackKind::Throw | AttackKind::Shot => {
                        self.ranged_attack_process(&mut attack, view, set, seqs, events)
                    }
                };
                if keep {
                    self.action = Action::Attack(attack);
                }
            }
            Action::Magic(mut magic) => {
                if self.magic_process(&mut magic, dt, view, set, seqs) {
                    self.action = Action::Magic(magic);
                }
            }
        }
    }

    fn sword_attack_process(
        &mut self,
        attack: &mut Attack,
        view: &PawnView,
        set: &AnimSet,
        seqs: &dyn Sequences,
    ) -> bool {
        match attack.status {
            1 => {
                let kind = random_atk_anim_kind(self.random());
                let name = set.name(kind, view.weapon_type).map(str::to_string);
                let rate = set.rate(kind, view.weapon_type) * view.attack_speed_rate;
                attack.shots = name.as_deref().and_then(|name| seqs.seq(name)).map_or(0, |seq| seq.shots.len());
                if view.ride_type.is_some() {
                    self.ride_attack = Some(kind);
                }
                let played = self.play_anim(
                    name.as_deref(),
                    rate * SWORD_RATE_BONUS,
                    0.1 / rate,
                    false,
                    UpdateMovementAnim::Once,
                    seqs,
                );
                if played {
                    attack.status = 3;
                    true
                } else {
                    self.attack_stop(view, set, seqs);
                    false
                }
            }
            3 => {
                if !self.current_is_attack(view, set) {
                    attack.status = 5;
                }
                true
            }
            4 => {
                let eff = (self.channel.attack_eff_frame + ATTACK_EFF_TOLERANCE).min(ATTACK_EFF_MAX);
                if self.channel.frame > eff {
                    attack.status = 5;
                }
                true
            }
            5 => {
                if self.current_is_attack(view, set) {
                    true
                } else {
                    self.attack_stop(view, set, seqs);
                    false
                }
            }
            _ => true,
        }
    }

    fn ranged_attack_process(
        &mut self,
        attack: &mut Attack,
        view: &PawnView,
        set: &AnimSet,
        seqs: &dyn Sequences,
        events: &mut Vec<AnimEvent>,
    ) -> bool {
        match attack.status {
            1 | 2 => {
                let name = set.name(AnimKind::Atk01, view.weapon_type).map(str::to_string);
                let rate = set.rate(AnimKind::Atk01, view.weapon_type) * view.attack_speed_rate;
                attack.shots = name.as_deref().and_then(|name| seqs.seq(name)).map_or(0, |seq| seq.shots.len());
                self.play_anim(name.as_deref(), rate, 0.2 / rate, false, UpdateMovementAnim::Once, seqs);
                attack.status = if attack.kind == AttackKind::Shot { 3 } else { 4 };
                true
            }
            3 | 4 => {
                let eff = if self.channel.attack_eff_frame != 0.0 { self.channel.attack_eff_frame } else { self.channel.pre_shot_time };
                if self.channel.frame > eff {
                    attack.status = 5;
                }
                if !self.current_is_attack(view, set) {
                    attack.status = 6;
                }
                true
            }
            5 => {
                let end = if self.channel.attack_end_eff_frame != 0.0 { self.channel.attack_end_eff_frame } else { self.channel.shot_time };
                if self.channel.frame > end {
                    events.push(AnimEvent::ArrowReleased(attack.hit));
                    attack.status = 6;
                    attack.fired = attack.shots;
                }
                if !self.current_is_attack(view, set) {
                    attack.status = 6;
                }
                true
            }
            _ => self.current_is_attack(view, set),
        }
    }

    fn current_is_attack(&mut self, view: &PawnView, set: &AnimSet) -> bool {
        let random = self.random();
        let atk = cur_atk_anim_name(set, view, self.current(), random).map(str::to_string);
        self.is_current(atk.as_deref())
    }

    fn attack_stop(&mut self, view: &PawnView, set: &AnimSet, seqs: &dyn Sequences) {
        if self.current_is_attack(view, set) {
            let wait = cur_wait_anim_name(set, view).map(str::to_string);
            self.play_anim(wait.as_deref(), view.non_attack_speed_rate, 0.1, true, UpdateMovementAnim::None, seqs);
        }
        self.update_movement_anim = UpdateMovementAnim::None;
    }

    fn magic_stop(&mut self, view: &PawnView, set: &AnimSet, seqs: &dyn Sequences, events: &mut Vec<AnimEvent>) {
        if self.current().is_some_and(|current| is_cast_anim(set, view, current)) {
            let wait = cur_wait_anim_name(set, view).map(str::to_string);
            self.play_anim(wait.as_deref(), view.non_attack_speed_rate, 0.0, true, UpdateMovementAnim::None, seqs);
        }
        events.push(AnimEvent::MagicStop);
        self.update_movement_anim = UpdateMovementAnim::None;
    }

    fn magic_process(
        &mut self,
        magic: &mut Magic,
        dt: f32,
        _view: &PawnView,
        _set: &AnimSet,
        seqs: &dyn Sequences,
    ) -> bool {
        let next = magic.timeline.next_stage(magic.elapsed, magic.stage);
        let mut keep = true;
        if next != magic.stage || magic.stage.is_none() {
            match next {
                None => keep = false,
                Some(stage) => {
                    let tween = if magic.elapsed == 0.0 { magic.timeline.first_tween } else { 0.0 };
                    let name = magic.stages.get(stage).cloned().flatten();
                    self.play_anim(
                        name.as_deref(),
                        magic.timeline.rate,
                        tween,
                        magic.timeline.is_hold(stage),
                        UpdateMovementAnim::Once,
                        seqs,
                    );
                    magic.stage = Some(stage);
                }
            }
        }
        magic.elapsed += dt;
        keep
    }

    fn naction_stop(&mut self, view: &PawnView, set: &AnimSet, seqs: &dyn Sequences, events: &mut Vec<AnimEvent>) {
        match std::mem::take(&mut self.action) {
            Action::Attack(attack) if matches!(attack.kind, AttackKind::Melee | AttackKind::Bow | AttackKind::Shot) => {
                self.attack_stop(view, set, seqs)
            }
            Action::Magic(_) => self.magic_stop(view, set, seqs, events),
            _ => {}
        }
    }

    fn on_attack_pre_shot(&mut self, events: &mut Vec<AnimEvent>) {
        if let Action::Magic(magic) = &self.action
            && magic.target != 0
        {
            events.push(AnimEvent::MagicPreShot { skill_id: magic.skill_id, level: magic.level, target: magic.target, shot_time: magic.timeline.shot_time });
        }
    }

    fn on_attack_shot(&mut self, events: &mut Vec<AnimEvent>) {
        match &mut self.action {
            Action::Attack(attack) if attack.kind == AttackKind::Melee && attack.status == 3 => {
                events.push(AnimEvent::AttackShot(attack.hit));
                attack.fired += 1;
                if attack.fired >= attack.shots {
                    attack.status = 4;
                }
            }
            Action::Magic(magic) => {
                let associated = std::mem::take(&mut magic.associated);
                events.push(AnimEvent::MagicShot { skill_id: magic.skill_id, level: magic.level, target: magic.target, shot_time: magic.timeline.shot_time, associated });
            }
            _ => {}
        }
    }

    pub fn anim_end(&mut self, view: &PawnView, set: &AnimSet, seqs: &dyn Sequences) {
        self.hide_weapons = HideWeapons::default();
        if self.update_movement_anim == UpdateMovementAnim::Once {
            self.update_movement_anim = UpdateMovementAnim::None;
        }
        if view.physics == Physics::Swimming || view.physics == Physics::Falling || view.physics == Physics::Flying {
            return;
        }
        let wait = script_cur_wait_anim_name(set, view).map(str::to_string);
        let Some(wait) = wait else { return };
        let Some(seq) = seqs.seq(&wait) else { return };
        self.channel.play(&wait, &seq, 1.0, 0.2, true);
    }

    fn apply(
        &mut self,
        trigger: Trigger,
        view: &PawnView,
        set: &AnimSet,
        tables: &dyn Tables,
        events: &mut Vec<AnimEvent>,
    ) {
        let rate = view.non_attack_speed_rate;
        let name = |kind: AnimKind| set.name(kind, view.weapon_type).map(str::to_string);
        match trigger {
            Trigger::Replay => {
                let social = set.pc_social.get(LOBBY_CLICK_SOCIAL).and_then(Option::as_deref).map(str::to_string);
                self.play_anim(social.as_deref(), rate, 0.3, false, UpdateMovementAnim::Once, tables);
            }
            Trigger::Spawn => {
                let wait = cur_wait_anim_name(set, view).map(str::to_string);
                self.play_anim(wait.as_deref(), rate, 0.0, true, UpdateMovementAnim::None, tables);
            }
            Trigger::Attack(hit) => {
                if view.dead {
                    return;
                }
                self.naction_stop(view, set, tables, events);
                let kind = match (view.weapon_type, view.ride_type) {
                    (_, Some(2)) => AttackKind::Shot,
                    (5, _) => AttackKind::Bow,
                    (6, _) => AttackKind::Throw,
                    _ => AttackKind::Melee,
                };
                log::debug!("ataque: arma tipo {} montaria {:?} -> {:?}, Atk01 {:?}", view.weapon_type, view.ride_type, kind, set.name(AnimKind::Atk01, view.weapon_type));
                log::debug!("ataque: velocidade {} duração {:?}", view.attack_speed_rate, set.name(AnimKind::Atk01, view.weapon_type).and_then(|name| tables.seq(name)).map(|seq| (seq.seconds(), seq.attack_eff_frame(), seq.attack_end_eff_frame(), seq.notify_time(true), seq.notify_time(false))));
                self.action = Action::Attack(Attack { kind, status: 1, hit, shots: 0, fired: 0 });
            }
            Trigger::ChangeWaitType { change, from } => {
                if from.code() == change.code() {
                    return;
                }
                self.naction_stop(view, set, tables, events);
                match change {
                    WaitChange::Sit => {
                        self.play_anim(
                            name(AnimKind::Sit).as_deref(),
                            set.sit_rate(),
                            0.1,
                            false,
                            UpdateMovementAnim::Once,
                            tables,
                        );
                    }
                    WaitChange::Stand => match from {
                        WaitType::Sit => {
                            self.play_anim(
                                name(AnimKind::Stand).as_deref(),
                                set.stand_rate(),
                                0.1,
                                false,
                                UpdateMovementAnim::Once,
                                tables,
                            );
                        }
                        WaitType::ChairSit => {
                            self.play_anim(
                                name(AnimKind::ChairStand).as_deref(),
                                rate,
                                0.1,
                                false,
                                UpdateMovementAnim::Once,
                                tables,
                            );
                        }
                        _ => {}
                    },
                    WaitChange::FakeDeath => {
                        let death = death_anim_name(set, view).map(str::to_string);
                        self.play_anim(death.as_deref(), rate, 0.1, false, UpdateMovementAnim::Once, tables);
                    }
                    WaitChange::FakeDeathStand => {
                        self.play_anim(
                            name(AnimKind::DeathStand).as_deref(),
                            rate,
                            0.1,
                            false,
                            UpdateMovementAnim::Once,
                            tables,
                        );
                    }
                }
            }
            Trigger::Die => {
                self.naction_stop(view, set, tables, events);
                let death = death_anim_name(set, view).map(str::to_string);
                self.play_anim(death.as_deref(), 1.0, 0.1, false, UpdateMovementAnim::Once, tables);
            }
            Trigger::Revive => {
                self.play_anim(
                    name(AnimKind::DeathStand).as_deref(),
                    rate,
                    0.1,
                    false,
                    UpdateMovementAnim::Once,
                    tables,
                );
            }
            Trigger::Social(id) => {
                let social = social_anim_name(set, view.npc, id as usize).map(str::to_string);
                let tween = if view.npc { 0.1 } else { 0.3 };
                if social.is_some()
                    && self.play_anim(social.as_deref(), rate, tween, false, UpdateMovementAnim::Once, tables)
                {
                    self.hide_weapons = if view.npc {
                        set.npc_social_hide.get(id as usize - 1).copied().unwrap_or_default()
                    } else {
                        set.pc_social_hide.get(id as usize).copied().unwrap_or_default()
                    };
                    let seconds = |name: &str| tables.seq(name).map(|seq| seq.seconds());
                    let duration = super::names::social_anim_duration(seconds, set, view, id as usize);
                    events.push(AnimEvent::Social { id, duration });
                }
            }
            Trigger::PlayAnim(sequence) => {
                self.play_anim(Some(&sequence), 1.0, ENTER_EVENT_TWEEN, false, UpdateMovementAnim::Once, tables);
            }
            Trigger::PicItem => {
                self.play_anim(name(AnimKind::PicItem).as_deref(), rate, 0.1, false, UpdateMovementAnim::Once, tables);
            }
            Trigger::ChairSit => {
                self.play_anim(name(AnimKind::ChairSit).as_deref(), rate, 0.1, false, UpdateMovementAnim::Once, tables);
            }
            Trigger::MagicSkillUse { skill_id, level, hit_time, target }
            | Trigger::MagicSkillUseOnMover { skill_id, level, hit_time, target } => {
                let on_mover = matches!(trigger, Trigger::MagicSkillUseOnMover { .. });
                let Some(skill) = tables.skill(skill_id, level) else { return };
                if skill.cast_style == 0 && !on_mover {
                    events.push(AnimEvent::TransientEffect { skill_id, level, target });
                    if !crate::magic::spawn_n_transient_effect(skill_id) {
                        events.push(AnimEvent::MoveStop);
                        self.naction_stop(view, set, tables, events);
                    }
                    return;
                }
                if view.dead {
                    return;
                }
                self.naction_stop(view, set, tables, events);
                let npc_seq = view.npc.then(|| tables.npc_skill_anim(self.npc_class_id, skill_id)).flatten();
                let Some(anim) = skill_anim(&skill.animation, set, view.weapon_type, npc_seq.as_deref()) else {
                    events.push(AnimEvent::CastStart { skill_id, level, target, shot_time: 0.0 });
                    return;
                };
                let stages: Vec<Option<StageSeq>> = anim
                    .stages
                    .iter()
                    .map(|stage| {
                        stage
                            .as_deref()
                            .and_then(|name| tables.seq(name))
                            .map(|seq| StageSeq { seconds: seq.seconds(), attack_shot: seq.first_shot() })
                    })
                    .collect();
                let lead = cast_lead(skill.cast_style, skill.visual_effect_lead);
                let hit_seconds = hit_time as f32 * 0.001;
                let Some(timeline) = skill_timeline(&stages, anim.hold, hit_seconds, view.skill_speed_rate, lead)
                else {
                    events.push(AnimEvent::CastStart { skill_id, level, target, shot_time: 0.0 });
                    return;
                };
                events.push(AnimEvent::CastStart { skill_id, level, target, shot_time: timeline.shot_time });
                self.action =
                    Action::Magic(Magic { skill_id, level, target, stages: anim.stages, timeline, elapsed: 0.0, stage: None, associated: Vec::new() });
            }
            Trigger::MagicSkillLaunched { skill_id, targets } => {
                if view.dead {
                    return;
                }
                if let Action::Magic(magic) = &mut self.action
                    && magic.skill_id == skill_id
                    && magic.target != 0
                {
                    magic.associated.extend(targets);
                }
            }
            Trigger::MagicCancel => {
                self.action = Action::None;
                self.magic_stop(view, set, tables, events);
            }
            Trigger::ActionStop => self.naction_stop(view, set, tables, events),
            Trigger::FishingStart | Trigger::FishingPull | Trigger::FishingEnd => {
                let kind = match trigger {
                    Trigger::FishingStart => FishingKind::Start,
                    Trigger::FishingPull => FishingKind::Pull,
                    _ => FishingKind::End,
                };
                let name = set.fishing(kind).map(str::to_string);
                self.play_anim(name.as_deref(), rate, 0.1, false, UpdateMovementAnim::Once, tables);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::channel::SeqInfo;
    use crate::anim::names::MoveType;
    use l2_dat::animnames::parse;

    const WARRIOR: &str = include_str!("../../../l2-dat/tests/fixtures/lineagewarrior.int.txt");

    struct Mock;

    impl Sequences for Mock {
        fn seq(&self, name: &str) -> Option<SeqInfo> {
            let lower = name.to_ascii_lowercase();
            let stem = lower.strip_suffix("_mfighter").unwrap_or(&lower);
            Some(match stem {
                "wait_hand" | "wait_1hs" | "wait_2hs" | "wait_bow" => SeqInfo::new(10, 3.0),
                "atkwait_hand" | "atkwait_1hs" | "atkwait_bow" => SeqInfo::new(10, 6.0),
                "walk_hand" | "walk_1hs" | "walk_bow" => SeqInfo::new(12, 15.0),
                "run_hand" | "run_1hs" | "run_bow" => SeqInfo::new(13, 15.0),
                "sit" => SeqInfo::new(41, 12.0),
                "sitwait" => SeqInfo::new(7, 3.0),
                "stand" => SeqInfo::new(41, 15.0),
                "death" => SeqInfo::new(51, 15.0),
                "deathwait" => SeqInfo::new(2, 30.0),
                "rebirth" => SeqInfo::new(30, 15.0),
                "chairsit" | "chairwait" | "chairstand" => SeqInfo::new(20, 10.0),
                "picitem" => SeqInfo::new(20, 10.0),
                "social_nod" | "social_victory" => SeqInfo::new(30, 15.0),
                "atk01_1hs" | "atk02_1hs" | "atk03_1hs" | "atk01_hand" => SeqInfo::with_shots(20, 20.0, &[0.5]),
                "atk01_bow" => SeqInfo::with_shots(30, 15.0, &[0.4, 0.7]),
                "castshort" => SeqInfo::new(10, 10.0),
                "castend" => SeqInfo::new(10, 10.0),
                "magicnotarget" => SeqInfo::with_shots(20, 10.0, &[0.5]),
                "magicthrow" => SeqInfo::with_shots(20, 10.0, &[0.5]),
                _ => return None,
            })
        }
    }

    impl Tables for Mock {
        fn skill(&self, skill_id: i32, _level: i32) -> Option<SkillData> {
            match skill_id {
                1 => Some(SkillData { animation: "A".into(), cast_style: 1, visual_effect_lead: None }),
                2 => Some(SkillData { animation: "L".into(), cast_style: 0, visual_effect_lead: None }),
                3 => Some(SkillData { animation: "l".into(), cast_style: 3, visual_effect_lead: None }),
                2039 => Some(SkillData { animation: String::new(), cast_style: 0, visual_effect_lead: None }),
                _ => None,
            }
        }

        fn npc_skill_anim(&self, _npc_class_id: i32, _skill_id: i32) -> Option<String> {
            None
        }
    }

    fn fighter() -> AnimSet {
        parse(WARRIOR).remove("mfighter").unwrap()
    }

    fn spawned(view: &PawnView, set: &AnimSet) -> AnimPlayer {
        let mut player = AnimPlayer::with_seed(7);
        player.push(Trigger::Spawn);
        player.tick(0.0, view, set, &Mock, false, 0.0);
        player
    }

    fn run_until_end(player: &mut AnimPlayer, view: &PawnView, set: &AnimSet, moving: bool) -> Vec<AnimEvent> {
        let mut events = Vec::new();
        for _ in 0..600 {
            let before = player.current().map(str::to_string);
            events.extend(player.tick(0.05, view, set, &Mock, moving, 0.0));
            if player.current().map(str::to_string) != before {
                break;
            }
        }
        events
    }

    #[test]
    fn spawn_loops_the_wait_at_the_move_multiplier_without_tween() {
        let set = fighter();
        let mut view = PawnView::standing(1);
        view.non_attack_speed_rate = 1.5;
        let player = spawned(&view, &set);
        assert_eq!(player.current(), Some("wait_1HS_MFighter"));
        assert!(player.channel.looping && !player.channel.is_tweening());
        assert!((player.channel.rate - 1.5 * 0.3).abs() < 1e-6);
        assert_eq!(player.update_movement_anim, UpdateMovementAnim::None);
    }

    #[test]
    fn a_player_walks_for_walk_anim_frame_in_move_frames_then_runs_and_stops_into_wait() {
        let set = fighter();
        let mut view = PawnView::standing(1);
        view.non_attack_speed_rate = 1.2;
        let mut player = spawned(&view, &set);
        player.tick(0.1, &view, &set, &Mock, true, 0.0);
        assert_eq!(player.current(), Some("walk_1HS_MFighter"));
        assert!((player.channel.tween_rate - 1.0 / (0.1 * 12.0)).abs() < 1e-5);
        assert!((player.channel.rate - 1.2 * 15.0 / 12.0).abs() < 1e-5);
        for _ in 0..3 {
            player.tick(0.1, &view, &set, &Mock, true, 0.0);
        }
        assert_eq!(player.current(), Some("walk_1HS_MFighter"));
        player.tick(0.1, &view, &set, &Mock, true, 0.0);
        assert_eq!(player.current(), Some("run_1HS_MFighter"));
        assert!((player.channel.rate - 1.2 * 15.0 / 13.0).abs() < 1e-5);
        player.tick(0.1, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("wait_1HS_MFighter"));
        assert!((player.channel.tween_rate - 1.0 / (0.2 * 10.0)).abs() < 1e-5);
        assert_eq!(player.move_timer, 0.0);
        let frame = player.channel.frame;
        player.tick(0.1, &view, &set, &Mock, false, 0.0);
        assert!(player.channel.frame > frame);
    }

    #[test]
    fn npcs_and_slow_movers_skip_the_walk_lead_in() {
        let set = fighter();
        let mut npc = PawnView::standing(0);
        npc.npc = true;
        let mut player = spawned(&npc, &set);
        player.tick(0.1, &npc, &set, &Mock, true, 0.0);
        assert_eq!(player.current(), Some("run_Hand_MFighter"));
        let mut slow = PawnView::standing(0);
        slow.move_type = MoveType::Slow;
        let mut walker = spawned(&slow, &set);
        for _ in 0..20 {
            walker.tick(0.1, &slow, &set, &Mock, true, 0.0);
        }
        assert_eq!(walker.current(), Some("walk_Hand_MFighter"));
    }

    #[test]
    fn npc_speed_from_the_npcgrp_scales_walk_and_run_and_zero_means_one() {
        let set = fighter();
        let mut npc = PawnView::standing(0);
        npc.npc = true;
        npc.npc_speed = 1.5;
        let mut player = spawned(&npc, &set);
        player.tick(0.1, &npc, &set, &Mock, true, 0.0);
        assert_eq!(player.current(), Some("run_Hand_MFighter"));
        assert!((player.channel.rate - 1.5 * 15.0 / 13.0).abs() < 1e-5, "{}", player.channel.rate);
        npc.npc_speed = 0.0;
        let mut plain = spawned(&npc, &set);
        plain.tick(0.1, &npc, &set, &Mock, true, 0.0);
        assert!((plain.channel.rate - 15.0 / 13.0).abs() < 1e-5);
        let mut pc = PawnView::standing(0);
        pc.npc_speed = 1.5;
        let mut player = spawned(&pc, &set);
        for _ in 0..20 {
            player.tick(0.1, &pc, &set, &Mock, true, 0.0);
        }
        assert_eq!(player.current(), Some("run_Hand_MFighter"));
        assert!((player.channel.rate - 15.0 / 13.0).abs() < 1e-5);
    }

    #[test]
    fn turning_in_place_plays_the_turn_animation() {
        let set = fighter();
        let view = PawnView::standing(0);
        let mut player = spawned(&view, &set);
        player.tick(0.1, &view, &set, &Mock, false, 0.0);
        player.tick(0.1, &view, &set, &Mock, false, 5000.0);
        assert_eq!(player.current(), Some("walk_Hand_MFighter"));
        assert!((player.channel.tween_rate - 1.0 / (0.1 * 12.0)).abs() < 1e-5);
        player.tick(0.1, &view, &set, &Mock, false, 5000.0);
        assert_eq!(player.current(), Some("wait_Hand_MFighter"));
    }

    #[test]
    fn auto_attacking_switches_the_wait_to_atk_wait() {
        let set = fighter();
        let mut view = PawnView::standing(1);
        let mut player = spawned(&view, &set);
        view.auto_attacking = true;
        player.tick(0.1, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("atkWait_1HS_MFighter"));
        assert!((player.channel.tween_rate - 1.0 / (0.2 * 10.0)).abs() < 1e-5);
        view.auto_attacking = false;
        player.tick(0.1, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("wait_1HS_MFighter"));
    }

    #[test]
    fn a_melee_attack_plays_one_random_swing_fires_the_shot_and_returns_to_atk_wait() {
        let set = fighter();
        let mut view = PawnView::standing(1);
        view.attack_speed_rate = 1.2;
        let mut player = spawned(&view, &set);
        let hit = Hit { target: 9, damage: 30, ..Hit::default() };
        player.push(Trigger::Attack(hit));
        view.auto_attacking = true;
        let events = player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert!(events.is_empty());
        let current = player.current().unwrap().to_string();
        assert!(current.starts_with("Atk0") && current.ends_with("_1HS_MFighter"), "{current}");
        let rate = 0.8333 * 1.2;
        assert!((player.channel.rate - rate * SWORD_RATE_BONUS).abs() < 1e-5);
        assert!((player.channel.tween_rate - 1.0 / (0.1 / rate * 20.0)).abs() < 1e-4);
        assert_eq!(player.update_movement_anim, UpdateMovementAnim::Once);
        assert!(matches!(&player.action, Action::Attack(a) if a.status == 3 && a.shots == 1));
        let mut fired = Vec::new();
        for _ in 0..40 {
            fired.extend(player.tick(0.05, &view, &set, &Mock, false, 0.0));
            if !fired.is_empty() {
                break;
            }
        }
        assert_eq!(fired, [AnimEvent::AttackShot(hit)]);
        assert!(matches!(&player.action, Action::Attack(a) if a.status == 4));
        assert_eq!(player.current(), Some(current.as_str()));
        run_until_end(&mut player, &view, &set, false);
        assert_eq!(player.current(), Some("atkWait_1HS_MFighter"));
        player.tick(0.05, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.action, Action::None);
        assert_eq!(player.update_movement_anim, UpdateMovementAnim::None);
    }

    #[test]
    fn attack_animation_is_not_overridden_by_the_wait_but_movement_interrupts_it() {
        let set = fighter();
        let mut view = PawnView::standing(1);
        view.auto_attacking = true;
        let mut player = spawned(&view, &set);
        player.push(Trigger::Attack(Hit::default()));
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        let swing = player.current().unwrap().to_string();
        player.tick(0.1, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some(swing.as_str()));
        player.tick(0.1, &view, &set, &Mock, true, 0.0);
        assert_eq!(player.current(), Some("walk_1HS_MFighter"));
        assert_eq!(player.update_movement_anim, UpdateMovementAnim::None);
    }

    #[test]
    fn bow_attacks_always_use_atk01_and_release_the_arrow_at_the_last_shot() {
        let set = fighter();
        let mut view = PawnView::standing(5);
        view.attack_speed_rate = 1.0;
        let mut player = spawned(&view, &set);
        let hit = Hit { target: 3, ..Hit::default() };
        player.push(Trigger::Attack(hit));
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("Atk01_Bow_MFighter"));
        assert!((player.channel.rate - 1.1111 * 15.0 / 30.0).abs() < 1e-5);
        assert!((player.channel.tween_rate - 1.0 / (0.2 / 1.1111 * 30.0)).abs() < 1e-4);
        assert!(matches!(&player.action, Action::Attack(a) if a.kind == AttackKind::Bow && a.status == 4));
        let mut events = Vec::new();
        for _ in 0..60 {
            events.extend(player.tick(0.05, &view, &set, &Mock, false, 0.0));
        }
        assert_eq!(events, [AnimEvent::ArrowReleased(hit)]);
        assert_eq!(player.action, Action::None);
    }

    #[test]
    fn sit_sit_wait_stand_and_chair_follow_change_wait_type() {
        let set = fighter();
        let mut view = PawnView::standing(0);
        let mut player = spawned(&view, &set);
        player.push(Trigger::ChangeWaitType { change: WaitChange::Sit, from: WaitType::Stand });
        view.wait_type = WaitType::Sit;
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("Sit_MFighter"));
        assert!((player.channel.rate - 1.27451 * 12.0 / 41.0).abs() < 1e-5);
        assert_eq!(player.update_movement_anim, UpdateMovementAnim::Once);
        run_until_end(&mut player, &view, &set, false);
        assert_eq!(player.current(), Some("SitWait_MFighter"));
        assert!(player.channel.looping);
        assert_eq!(player.update_movement_anim, UpdateMovementAnim::None);
        player.push(Trigger::ChangeWaitType { change: WaitChange::Sit, from: WaitType::Sit });
        player.tick(0.1, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("SitWait_MFighter"));
        player.push(Trigger::ChangeWaitType { change: WaitChange::Stand, from: WaitType::Sit });
        view.wait_type = WaitType::Stand;
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("Stand_MFighter"));
        assert!((player.channel.rate - 1.01307 * 15.0 / 41.0).abs() < 1e-5);
        run_until_end(&mut player, &view, &set, false);
        assert_eq!(player.current(), Some("wait_Hand_MFighter"));
        player.push(Trigger::ChairSit);
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("ChairSit_MFighter"));
        view.wait_type = WaitType::ChairSit;
        run_until_end(&mut player, &view, &set, false);
        assert_eq!(player.current(), Some("wait_Hand_MFighter"));
        player.tick(0.05, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("ChairWait_MFighter"));
        player.push(Trigger::ChangeWaitType { change: WaitChange::Stand, from: WaitType::ChairSit });
        view.wait_type = WaitType::Stand;
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("ChairStand_MFighter"));
    }

    #[test]
    fn fake_death_plays_death_then_death_wait_and_rebirth_on_stand() {
        let set = fighter();
        let mut view = PawnView::standing(0);
        let mut player = spawned(&view, &set);
        player.push(Trigger::ChangeWaitType { change: WaitChange::FakeDeath, from: WaitType::Stand });
        view.wait_type = WaitType::FakeDead;
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("Death_MFighter"));
        run_until_end(&mut player, &view, &set, false);
        assert_eq!(player.current(), Some("DeathWait_MFighter"));
        player.push(Trigger::ChangeWaitType { change: WaitChange::FakeDeathStand, from: WaitType::FakeDead });
        view.wait_type = WaitType::Stand;
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("Rebirth_MFighter"));
    }

    #[test]
    fn dying_plays_death_at_rate_one_holds_death_wait_and_revives_with_rebirth() {
        let set = fighter();
        let mut view = PawnView::standing(2);
        view.non_attack_speed_rate = 1.3;
        let mut player = spawned(&view, &set);
        player.push(Trigger::Attack(Hit::default()));
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        player.push(Trigger::Die);
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("Death_MFighter"));
        assert!((player.channel.rate - 15.0 / 51.0).abs() < 1e-5);
        assert_eq!(player.action, Action::None);
        view.dead = true;
        run_until_end(&mut player, &view, &set, false);
        assert_eq!(player.current(), Some("DeathWait_MFighter"));
        for _ in 0..10 {
            player.tick(0.1, &view, &set, &Mock, true, 100.0);
        }
        assert_eq!(player.current(), Some("DeathWait_MFighter"));
        player.push(Trigger::Revive);
        view.dead = false;
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("Rebirth_MFighter"));
        assert!((player.channel.rate - 1.3 * 15.0 / 30.0).abs() < 1e-5);
        run_until_end(&mut player, &view, &set, false);
        assert_eq!(player.current(), Some("wait_2HS_MFighter"));
    }

    #[test]
    fn social_actions_hide_weapons_until_the_animation_ends() {
        let set = fighter();
        let view = PawnView::standing(1);
        let mut player = spawned(&view, &set);
        player.push(Trigger::Social(2));
        let events = player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("Social_Nod_MFighter"));
        assert!((player.channel.tween_rate - 1.0 / (0.3 * 30.0)).abs() < 1e-5);
        assert_eq!(player.hide_weapons, HideWeapons { right: true, left: false });
        assert_eq!(events, [AnimEvent::Social { id: 2, duration: 2.0 + 0.3 }]);
        run_until_end(&mut player, &view, &set, false);
        assert_eq!(player.current(), Some("wait_1HS_MFighter"));
        assert_eq!(player.hide_weapons, HideWeapons::default());
        player.push(Trigger::Social(1));
        player.tick(0.1, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("wait_1HS_MFighter"));
    }

    #[test]
    fn clicking_the_selected_lobby_pawn_plays_pc_social_three_once_and_returns_to_wait() {
        let set = fighter();
        let view = PawnView::standing(1);
        let mut player = spawned(&view, &set);
        player.push(Trigger::Replay);
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), social_anim_name(&set, false, LOBBY_CLICK_SOCIAL));
        assert_eq!(player.hide_weapons, HideWeapons::default());
        assert!((player.channel.tween_rate - 1.0 / (0.3 * 30.0)).abs() < 1e-5);
        run_until_end(&mut player, &view, &set, false);
        assert_eq!(player.current(), Some("wait_1HS_MFighter"));
    }

    #[test]
    fn picking_an_item_plays_pic_item_once() {
        let set = fighter();
        let view = PawnView::standing(0);
        let mut player = spawned(&view, &set);
        player.push(Trigger::PicItem);
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("PicItem_MFighter"));
        assert_eq!(player.update_movement_anim, UpdateMovementAnim::Once);
        run_until_end(&mut player, &view, &set, false);
        assert_eq!(player.current(), Some("wait_Hand_MFighter"));
    }

    #[test]
    fn a_cast_plays_intro_hold_and_release_with_the_shot_at_hit_time() {
        let set = fighter();
        let mut view = PawnView::standing(0);
        view.skill_speed_rate = 1.0;
        let mut player = spawned(&view, &set);
        player.push(Trigger::MagicSkillUse { skill_id: 1, level: 1, hit_time: 3000, target: 5 });
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert_eq!(player.current(), Some("CastShort_MFighter"));
        assert!(!player.channel.looping);
        assert!((player.channel.tween_rate - 1.0 / (0.2 * 10.0)).abs() < 1e-5);
        let Action::Magic(magic) = &player.action else { panic!() };
        assert_eq!(magic.stage, Some(0));
        let available = 3.0 - 0.15;
        let intro_end = 1.2 - 1.2 / available * 0.15;
        assert!((magic.timeline.ends[0] - intro_end).abs() < 1e-4);
        assert!((magic.timeline.shot_time - (available - (1.2 - intro_end))).abs() < 1e-4);
        let mut elapsed = 0.0;
        let mut events = Vec::new();
        while elapsed < 1.5 {
            events.extend(player.tick(0.05, &view, &set, &Mock, false, 0.0));
            elapsed += 0.05;
        }
        assert_eq!(player.current(), Some("CastEnd_MFighter"));
        assert!(player.channel.looping);
        while elapsed < 2.2 {
            events.extend(player.tick(0.05, &view, &set, &Mock, false, 0.0));
            elapsed += 0.05;
        }
        assert_eq!(player.current(), Some("MagicNoTarget_MFighter"));
        assert!(!player.channel.looping);
        while elapsed < 4.5 {
            events.extend(player.tick(0.05, &view, &set, &Mock, false, 0.0));
            elapsed += 0.05;
        }
        assert!(matches!(events[..], [AnimEvent::MagicShot { skill_id: 1, target: 5, .. }]));
        assert_eq!(player.action, Action::None);
        assert_eq!(player.current(), Some("wait_Hand_MFighter"));
    }

    #[test]
    fn magic_skill_launched_104ca770_feeds_the_list_that_the_final_shot_hands_over_once() {
        let set = fighter();
        let mut view = PawnView::standing(0);
        view.skill_speed_rate = 1.0;
        let mut player = spawned(&view, &set);
        player.push(Trigger::MagicSkillLaunched { skill_id: 1, targets: vec![9] });
        player.push(Trigger::MagicSkillUse { skill_id: 1, level: 1, hit_time: 3000, target: 5 });
        player.push(Trigger::MagicSkillLaunched { skill_id: 2, targets: vec![7] });
        player.push(Trigger::MagicSkillLaunched { skill_id: 1, targets: vec![5, 6] });
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        let Action::Magic(magic) = &player.action else { panic!() };
        assert_eq!(magic.associated, vec![5, 6]);
        let mut events = Vec::new();
        let dead = PawnView { dead: true, ..view };
        player.apply(Trigger::MagicSkillLaunched { skill_id: 1, targets: vec![8] }, &dead, &set, &Mock, &mut events);
        player.on_attack_shot(&mut events);
        player.on_attack_shot(&mut events);
        assert!(matches!(&events[..], [AnimEvent::MagicShot { associated: first, .. }, AnimEvent::MagicShot { associated: second, .. }] if *first == vec![5, 6] && second.is_empty()));
    }

    #[test]
    fn attack_pre_shot_notify_1067ca50_raises_the_pre_shot_only_for_a_magic_action_with_a_target() {
        let set = fighter();
        let mut view = PawnView::standing(0);
        view.skill_speed_rate = 1.0;
        let mut player = spawned(&view, &set);
        let mut events = Vec::new();
        player.on_attack_pre_shot(&mut events);
        assert!(events.is_empty());
        player.push(Trigger::MagicSkillUse { skill_id: 1, level: 1, hit_time: 3000, target: 5 });
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        player.on_attack_pre_shot(&mut events);
        assert!(matches!(events[..], [AnimEvent::MagicPreShot { skill_id: 1, target: 5, .. }]));
        if let Action::Magic(magic) = &mut player.action {
            magic.target = 0;
        }
        events.clear();
        player.on_attack_pre_shot(&mut events);
        assert!(events.is_empty());
    }

    #[test]
    fn on_receive_magic_skill_use_10480220_spawns_the_transient_effect_even_when_dead() {
        let set = fighter();
        let mut view = PawnView::standing(0);
        let mut player = spawned(&view, &set);
        view.dead = true;
        player.push(Trigger::MagicSkillUse { skill_id: 2, level: 1, hit_time: 0, target: 9 });
        assert_eq!(player.tick(0.0, &view, &set, &Mock, false, 0.0), [AnimEvent::TransientEffect { skill_id: 2, level: 1, target: 9 }, AnimEvent::MoveStop]);
        player.push(Trigger::MagicSkillUse { skill_id: 1, level: 1, hit_time: 1000, target: 9 });
        assert!(player.tick(0.0, &view, &set, &Mock, false, 0.0).is_empty());
    }

    #[test]
    fn on_receive_magic_skill_use_10480880_always_sets_the_magic_info_for_a_mover_target() {
        let set = fighter();
        let view = PawnView::standing(0);
        let mut player = spawned(&view, &set);
        player.push(Trigger::MagicSkillUseOnMover { skill_id: 2, level: 1, hit_time: 1000, target: 100 });
        let events = player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert!(!matches!(events[..], [AnimEvent::TransientEffect { .. }]));
        assert!(matches!(events[..], [AnimEvent::CastStart { skill_id: 2, target: 100, .. }]));
        player.push(Trigger::MagicSkillUseOnMover { skill_id: 99, level: 1, hit_time: 1000, target: 100 });
        assert!(player.tick(0.0, &view, &set, &Mock, false, 0.0).is_empty(), "sem GetMSData nada acontece");
    }

    #[test]
    fn on_receive_magic_skill_use_10480220_keeps_the_pawn_moving_when_spawn_n_transient_effect_returns_nonzero() {
        let set = fighter();
        let view = PawnView::standing(0);
        let mut player = spawned(&view, &set);
        player.push(Trigger::MagicSkillUse { skill_id: 2039, level: 1, hit_time: 0, target: 9 });
        assert_eq!(player.tick(0.0, &view, &set, &Mock, false, 0.0), [AnimEvent::TransientEffect { skill_id: 2039, level: 1, target: 9 }]);
    }

    #[test]
    fn instant_skills_without_cast_style_do_not_animate_and_cancel_returns_to_wait() {
        let set = fighter();
        let view = PawnView::standing(0);
        let mut player = spawned(&view, &set);
        player.push(Trigger::MagicSkillUse { skill_id: 2, level: 1, hit_time: 0, target: 5 });
        let events = player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert_eq!(events, [AnimEvent::TransientEffect { skill_id: 2, level: 1, target: 5 }, AnimEvent::MoveStop]);
        assert_eq!(player.action, Action::None);
        assert_eq!(player.current(), Some("wait_Hand_MFighter"));
        player.push(Trigger::MagicSkillUse { skill_id: 3, level: 1, hit_time: 1000, target: 5 });
        let events = player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert!(matches!(events[..], [AnimEvent::CastStart { skill_id: 3, target: 5, .. }]));
        assert_eq!(player.current(), Some("CastEnd_MFighter"));
        player.push(Trigger::MagicCancel);
        let events = player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert_eq!(events, [AnimEvent::MagicStop]);
        assert_eq!(player.current(), Some("wait_Hand_MFighter"));
        assert!(!player.channel.is_tweening());
        assert_eq!(player.action, Action::None);
    }

    #[test]
    fn magic_stop_104e5490_and_magic_cancel_104e5b20_raise_the_light_removal_event() {
        let set = fighter();
        let mut view = PawnView::standing(0);
        view.skill_speed_rate = 1.0;
        let mut player = spawned(&view, &set);
        player.push(Trigger::ActionStop);
        assert!(player.tick(0.0, &view, &set, &Mock, false, 0.0).is_empty());
        player.push(Trigger::MagicSkillUse { skill_id: 1, level: 1, hit_time: 3000, target: 5 });
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert!(matches!(player.action, Action::Magic(_)));
        player.push(Trigger::ActionStop);
        assert_eq!(player.tick(0.0, &view, &set, &Mock, false, 0.0), [AnimEvent::MagicStop]);
        player.push(Trigger::MagicSkillUse { skill_id: 1, level: 1, hit_time: 3000, target: 5 });
        player.tick(0.0, &view, &set, &Mock, false, 0.0);
        player.push(Trigger::MagicSkillUse { skill_id: 1, level: 1, hit_time: 3000, target: 6 });
        let events = player.tick(0.0, &view, &set, &Mock, false, 0.0);
        assert!(matches!(events[..], [AnimEvent::MagicStop, AnimEvent::CastStart { target: 6, .. }]));
        player.action = Action::None;
        player.push(Trigger::MagicCancel);
        assert_eq!(player.tick(0.0, &view, &set, &Mock, false, 0.0), [AnimEvent::MagicStop]);
        let mut events = Vec::new();
        player.magic_stop(&view, &set, &Mock, &mut events);
        assert_eq!(events, [AnimEvent::MagicStop]);
    }

    #[test]
    fn play_anim_with_no_name_or_unknown_sequence_changes_nothing() {
        let set = fighter();
        let view = PawnView::standing(6);
        let mut player = spawned(&view, &set);
        assert_eq!(player.current(), None);
        player.tick(0.1, &view, &set, &Mock, true, 0.0);
        assert_eq!(player.current(), None);
        let mut other = AnimPlayer::default();
        assert!(!other.play_anim(Some("Missing"), 1.0, 0.0, false, UpdateMovementAnim::Once, &Mock));
        assert_eq!(other.update_movement_anim, UpdateMovementAnim::None);
        assert!(!other.play_anim(None, 1.0, 0.0, false, UpdateMovementAnim::Once, &Mock));
    }
}
