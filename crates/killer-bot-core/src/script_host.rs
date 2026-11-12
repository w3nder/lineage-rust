use crate::chat::Channel;
use crate::condition::percent;
use crate::env::Env;
use crate::options::Trait;
use crate::traits::{Subject, status_bit};
use killer_bot_script::{Host, ScriptError, Status, Value, Vm};
use killer_world::{Entity, EntityKind, NetCommand};
use std::collections::VecDeque;

pub const QUEUE_LIMIT: usize = 64;
pub const NEAREST_RANGE: f32 = 1500.0;
pub const PICKUP_RANGE: f32 = 250.0;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ScriptStatus {
    #[default]
    None,
    Running,
    Waiting,
    Finished,
    Failed(ScriptError),
}

impl ScriptStatus {
    pub fn of(status: &Status) -> ScriptStatus {
        match status {
            Status::Running => ScriptStatus::Running,
            Status::Waiting => ScriptStatus::Waiting,
            Status::Finished => ScriptStatus::Finished,
            Status::Failed(error) => ScriptStatus::Failed(error.clone()),
        }
    }

    pub fn text(&self, english: bool) -> String {
        match self {
            ScriptStatus::None => if english { "No script" } else { "Sem script" }.to_owned(),
            ScriptStatus::Running => if english { "Running" } else { "Rodando" }.to_owned(),
            ScriptStatus::Waiting => if english { "Waiting" } else { "Esperando" }.to_owned(),
            ScriptStatus::Finished => if english { "Finished" } else { "Terminou" }.to_owned(),
            ScriptStatus::Failed(error) => format!("{}: {error}", if english { "Error" } else { "Erro" }),
        }
    }
}

pub struct ScriptRun {
    pub vm: Vm,
    pub queue: VecDeque<NetCommand>,
    pub status: ScriptStatus,
}

pub struct SessionHost<'a, 'b> {
    pub env: &'a Env<'b>,
    pub out: &'a mut VecDeque<NetCommand>,
    pub log: &'a mut Vec<String>,
}

pub const QUERIES: &[&str] = &[
    "hp", "mp", "cp", "level", "adena", "x", "y", "z", "my_name", "uptime", "party_size", "item_count", "skill_level", "nearby_count", "npc_count", "distance_to", "buff_time", "has_target", "target_id", "target_name", "target_hp", "target_mp",
    "target_dead", "target_level", "distance_to_target", "in_combat", "sitting", "running", "is_dead", "dead", "has_buff", "skill_ready", "has_skill", "has_item", "in_party", "in_range", "is_player", "is_monster", "bleeding", "poisoned",
    "stunned", "asleep", "muted", "rooted", "paralyzed", "frozen", "afraid", "shackled", "petrified", "target_bleeding", "target_poisoned", "target_stunned", "target_asleep", "target_muted", "target_rooted", "target_paralyzed", "target_frozen",
    "target_afraid", "target_shackled", "target_petrified", "event_text", "can_attack",
];

pub fn describe(command: &NetCommand) -> String {
    match command {
        NetCommand::UseSkill { skill_id, .. } => format!("use_skill {skill_id}"),
        NetCommand::UseItem { object_id, .. } => format!("use_item (objeto {object_id})"),
        NetCommand::UseAction { id, .. } => format!("use_action {id}"),
        NetCommand::Say { text, kind, target } if target.is_empty() => format!("say (canal {kind}) {text}"),
        NetCommand::Say { text, target, .. } => format!("whisper {target} {text}"),
        NetCommand::Bypass { command } => format!("bypass {command}"),
        NetCommand::MoveTo { target, .. } => format!("move_to {} {} {}", target[0], target[1], target[2]),
        NetCommand::AttackRequest { object_id, .. } => format!("attack (objeto {object_id})"),
        NetCommand::Action { object_id, .. } => format!("select (objeto {object_id})"),
        NetCommand::ChangeWaitType { standing } => if *standing { "stand" } else { "sit" }.to_owned(),
        NetCommand::ChangeMoveType { running } => if *running { "run" } else { "walk" }.to_owned(),
        NetCommand::TargetCancel => "cancel_target".to_owned(),
        NetCommand::SocialAction { action } => format!("social {action}"),
        other => format!("{other:?}").split([' ', '{', '(']).next().unwrap_or_default().to_owned(),
    }
}

pub const COMMANDS: &[&str] = &[
    "attack", "target_nearest", "select", "use_skill", "use_item", "use_action", "move_to", "sit", "stand", "run", "walk", "say", "whisper", "bypass", "pickup", "cancel_target", "social", "print", "stop", "assist",
];

fn trait_of(name: &str) -> Option<Trait> {
    Some(match name {
        "bleeding" => Trait::Bleeding,
        "poisoned" => Trait::Poisoned,
        "stunned" => Trait::Stunned,
        "asleep" => Trait::Slept,
        "muted" => Trait::Muted,
        "rooted" => Trait::Rooted,
        "paralyzed" => Trait::Paralyzed,
        "frozen" => Trait::Iced,
        "afraid" => Trait::Feared,
        "shackled" => Trait::Shackled,
        "petrified" => Trait::Petrified,
        _ => return None,
    })
}

fn channel_named(name: &str) -> Option<Channel> {
    Some(match name.to_lowercase().as_str() {
        "all" => Channel::All,
        "shout" => Channel::Shout,
        "party" => Channel::Party,
        "clan" => Channel::Clan,
        "ally" => Channel::Ally,
        "trade" => Channel::Trade,
        "hero" => Channel::Hero,
        _ => return None,
    })
}

fn number(args: &[Value], index: usize, what: &str) -> Result<i64, String> {
    match args.get(index) {
        Some(value) => value.num().ok_or_else(|| format!("{what} precisa ser número")),
        None => Err(format!("falta {what}")),
    }
}

fn text(args: &[Value], index: usize, what: &str) -> Result<String, String> {
    match args.get(index) {
        Some(Value::Text(value)) => Ok(value.clone()),
        Some(_) => Err(format!("{what} precisa ser texto")),
        None => Err(format!("falta {what}")),
    }
}

fn bit(value: bool) -> Value {
    Value::Bool(value)
}

impl SessionHost<'_, '_> {
    fn me(&self) -> Result<&Entity, String> {
        self.env.me().ok_or_else(|| "personagem ainda não está no mundo".to_owned())
    }

    fn target(&self) -> Option<&Entity> {
        self.env.world.targeted()
    }

    fn push(&mut self, command: NetCommand) -> Result<(), String> {
        if self.out.len() >= QUEUE_LIMIT {
            return Err("fila de comandos cheia".to_owned());
        }
        self.out.push_back(command);
        Ok(())
    }

    fn trait_value(&self, name: &str) -> Result<Value, String> {
        let (target, base) = match name.strip_prefix("target_") {
            Some(rest) => (true, rest),
            None => (false, name),
        };
        let trait_ = trait_of(base).ok_or_else(|| format!("consulta desconhecida: {name}"))?;
        let entity = if target { self.target() } else { self.env.me() };
        let held = entity.is_some_and(|entity| status_bit(trait_).is_some_and(|mask| Subject::of_entity(entity).abnormal & mask != 0));
        Ok(bit(held))
    }

    fn nearest(&self, filter: Option<&str>) -> Option<i32> {
        let me = self.env.me()?;
        self.env
            .world
            .entities
            .values()
            .filter(|entity| matches!(entity.kind, EntityKind::Npc { attackable: true, .. }) && !entity.dead)
            .filter(|entity| filter.is_none_or(|name| entity.name.to_lowercase().contains(&name.to_lowercase())))
            .map(|entity| (entity.position.distance(me.position), entity.object_id))
            .filter(|(distance, _)| *distance <= NEAREST_RANGE)
            .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
            .map(|(_, id)| id)
    }
}

impl Host for SessionHost<'_, '_> {
    fn query(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        let world = self.env.world;
        let me = self.env.me();
        let count = |id: i64| {
            world.inventory.items.iter().filter(|item| i64::from(item.item_id) == id).map(|item| i64::from(item.count)).sum::<i64>()
        };
        Ok(match name {
            "hp" => Value::Num(me.and_then(|me| percent(me.hp)).map_or(0, i64::from)),
            "mp" => Value::Num(me.and_then(|me| percent(me.mp)).map_or(0, i64::from)),
            "cp" => Value::Num(me.and_then(|me| percent(me.cp)).map_or(0, i64::from)),
            "level" => Value::Num(me.map_or(0, |me| i64::from(me.level))),
            "adena" => Value::Num(i64::from(world.inventory.adena())),
            "x" => Value::Num(me.map_or(0, |me| me.position.x as i64)),
            "y" => Value::Num(me.map_or(0, |me| me.position.y as i64)),
            "z" => Value::Num(me.map_or(0, |me| me.position.z as i64)),
            "my_name" => Value::Text(me.map(|me| me.name.clone()).unwrap_or_default()),
            "uptime" => Value::Num(self.env.now.as_secs() as i64),
            "party_size" => Value::Num(world.party.members.len() as i64),
            "in_party" => bit(!world.party.members.is_empty()),
            "item_count" => Value::Num(count(number(args, 0, "o id do item")?)),
            "has_item" => bit(count(number(args, 0, "o id do item")?) > 0),
            "skill_level" => {
                let id = number(args, 0, "o id da skill")?;
                Value::Num(world.skills.iter().find(|skill| i64::from(skill.id) == id).map_or(0, |skill| i64::from(skill.level)))
            }
            "has_skill" => {
                let id = number(args, 0, "o id da skill")?;
                bit(world.skills.iter().any(|skill| i64::from(skill.id) == id))
            }
            "skill_ready" => bit(self.env.ready(number(args, 0, "o id da skill")? as i32)),
            "has_buff" => {
                let id = number(args, 0, "o id da skill")?;
                bit(world.buffs.iter().any(|buff| i64::from(buff.skill_id) == id))
            }
            "buff_time" => {
                let id = number(args, 0, "o id da skill")?;
                Value::Num(world.buffs.iter().find(|buff| i64::from(buff.skill_id) == id).map_or(0, |buff| i64::from(buff.duration)))
            }
            "nearby_count" => {
                let range = number(args, 0, "o raio")? as f32;
                let origin = me.map(|me| me.position);
                Value::Num(world.entities.values().filter(|entity| matches!(entity.kind, EntityKind::Npc { attackable: true, .. }) && !entity.dead && origin.is_some_and(|origin| entity.position.distance(origin) <= range)).count() as i64)
            }
            "npc_count" => {
                let id = number(args, 0, "o id do npc")?;
                Value::Num(world.entities.values().filter(|entity| matches!(entity.kind, EntityKind::Npc { npc_id, .. } if i64::from(npc_id) == id) && !entity.dead).count() as i64)
            }
            "distance_to" => {
                let (x, y, z) = (number(args, 0, "x")? as f32, number(args, 1, "y")? as f32, number(args, 2, "z")? as f32);
                Value::Num(me.map_or(0, |me| me.position.distance(glam::Vec3::new(x, y, z)) as i64))
            }
            "has_target" => bit(self.target().is_some()),
            "target_id" => Value::Num(self.target().map_or(0, |target| i64::from(target.object_id))),
            "target_name" => Value::Text(self.target().map(|target| target.name.clone()).unwrap_or_default()),
            "target_hp" => Value::Num(self.target().and_then(|target| percent(target.hp)).map_or(0, i64::from)),
            "target_mp" => Value::Num(self.target().and_then(|target| percent(target.mp)).map_or(0, i64::from)),
            "target_dead" => bit(self.target().is_some_and(|target| target.dead)),
            "target_level" => Value::Num(self.target().map_or(0, |target| i64::from(target.level))),
            "distance_to_target" => Value::Num(match (me, self.target()) {
                (Some(me), Some(target)) => me.position.distance(target.position) as i64,
                _ => 0,
            }),
            "in_range" => {
                let range = number(args, 0, "o alcance")? as f32;
                bit(matches!((me, self.target()), (Some(me), Some(target)) if me.position.distance(target.position) <= range))
            }
            "can_attack" => bit(self.target().is_some_and(|target| !target.dead && matches!(target.kind, EntityKind::Npc { attackable: true, .. } | EntityKind::Player { .. }))),
            "in_combat" => bit(me.is_some_and(|me| me.in_combat)),
            "sitting" => bit(me.is_some_and(|me| me.sitting)),
            "running" => bit(me.is_some_and(|me| me.running)),
            "is_dead" | "dead" => bit(me.is_some_and(|me| me.dead) || world.death.is_some()),
            "is_player" => {
                let id = number(args, 0, "o id do objeto")? as i32;
                bit(world.entity(id).is_some_and(|entity| matches!(entity.kind, EntityKind::Player { .. })))
            }
            "is_monster" => {
                let id = number(args, 0, "o id do objeto")? as i32;
                bit(world.entity(id).is_some_and(|entity| matches!(entity.kind, EntityKind::Npc { attackable: true, .. })))
            }
            other => return self.trait_value(other),
        })
    }

    fn act(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
        let dead = self.env.me().is_some_and(|me| me.dead) || self.env.world.death.is_some();
        if dead && !matches!(name, "say" | "whisper" | "cancel_target" | "bypass") {
            return Ok(());
        }
        let origin = || self.env.origin().ok_or_else(|| "personagem ainda não está no mundo".to_owned());
        match name {
            "attack" => {
                let target = self.target().ok_or("sem alvo para atacar")?.object_id;
                let command = NetCommand::AttackRequest { object_id: target, origin: origin()?, shift: false };
                self.push(command)
            }
            "assist" => {
                let me = self.me()?.object_id;
                let leader = self.env.world.party.members.iter().find(|member| member.object_id != me).map(|member| member.object_id).ok_or("sem grupo para assistir")?;
                let command = self.env.select(leader).ok_or("personagem ainda não está no mundo")?;
                self.push(command)
            }
            "target_nearest" => {
                let filter = match args.first() {
                    Some(Value::Text(name)) => Some(name.clone()),
                    Some(_) => return Err("o nome do alvo precisa ser texto".to_owned()),
                    None => None,
                };
                if let Some(id) = self.nearest(filter.as_deref())
                    && let Some(command) = self.env.select(id)
                {
                    self.push(command)?;
                }
                Ok(())
            }
            "select" => {
                let id = number(args, 0, "o id do objeto")? as i32;
                if self.env.world.entity(id).is_none() {
                    return Err(format!("objeto {id} não está por perto"));
                }
                let command = self.env.select(id).ok_or("personagem ainda não está no mundo")?;
                self.push(command)
            }
            "use_skill" => {
                let skill_id = match args.first() {
                    Some(Value::Text(name)) => self.env.skill_by_name(name).ok_or_else(|| format!("skill desconhecida: {name}"))?,
                    _ => number(args, 0, "o id da skill")? as i32,
                };
                if !self.env.knows(skill_id) {
                    return Err(format!("skill {skill_id} desconhecida ou passiva"));
                }
                self.push(NetCommand::UseSkill { skill_id, ctrl: false, shift: false })
            }
            "use_item" => {
                let item_id = number(args, 0, "o id do item")? as i32;
                let object_id = self.env.item_object(item_id).ok_or_else(|| format!("item {item_id} não está no inventário"))?;
                self.push(NetCommand::UseItem { object_id, ctrl: false })
            }
            "use_action" => {
                let id = number(args, 0, "o id da ação")? as i32;
                self.push(NetCommand::UseAction { id, ctrl: false, shift: false })
            }
            "social" => {
                let action = number(args, 0, "o id da ação social")? as i32;
                self.push(NetCommand::SocialAction { action })
            }
            "move_to" => {
                let target = [number(args, 0, "x")? as i32, number(args, 1, "y")? as i32, number(args, 2, "z")? as i32];
                let command = NetCommand::MoveTo { target, origin: origin()? };
                self.push(command)
            }
            "sit" => self.push(NetCommand::ChangeWaitType { standing: false }),
            "stand" => self.push(NetCommand::ChangeWaitType { standing: true }),
            "run" => self.push(NetCommand::ChangeMoveType { running: true }),
            "walk" => self.push(NetCommand::ChangeMoveType { running: false }),
            "say" => {
                let channel = channel_named(&text(args, 0, "o canal")?).ok_or("canal desconhecido (all, shout, party, clan, ally, trade, hero)")?;
                let body = text(args, 1, "o texto")?;
                if body.trim().is_empty() {
                    return Err("texto vazio".to_owned());
                }
                let kind = channel.kind().ok_or("canal sem envio")?;
                self.push(NetCommand::Say { text: body, kind, target: String::new() })
            }
            "whisper" => {
                let target = text(args, 0, "o nome")?;
                let body = text(args, 1, "o texto")?;
                if target.trim().is_empty() || body.trim().is_empty() {
                    return Err("nome ou texto vazio".to_owned());
                }
                let kind = Channel::Whisper.kind().ok_or("canal sem envio")?;
                self.push(NetCommand::Say { text: body, kind, target })
            }
            "bypass" => {
                let command = text(args, 0, "o comando")?;
                self.push(NetCommand::Bypass { command })
            }
            "cancel_target" => self.push(NetCommand::TargetCancel),
            "pickup" => {
                let range = match args.first() {
                    Some(value) => value.num().ok_or("o alcance precisa ser número")? as f32,
                    None => PICKUP_RANGE,
                };
                let me = self.me()?.position;
                let best = self
                    .env
                    .world
                    .entities
                    .values()
                    .filter(|entity| matches!(entity.kind, EntityKind::Item { .. }))
                    .map(|entity| (entity.position.distance(me), entity.object_id))
                    .filter(|(distance, _)| *distance <= range)
                    .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
                if let Some((_, id)) = best
                    && let Some(command) = self.env.select(id)
                {
                    self.push(command)?;
                }
                Ok(())
            }
            other => Err(format!("comando desconhecido: {other}")),
        }
    }

    fn print(&mut self, text: &str) {
        self.log.push(text.to_owned());
    }
}
