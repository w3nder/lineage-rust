pub mod actionname;
pub mod animnames;
pub mod armorgrp;
pub mod castlename;
pub mod chronicle;
pub mod chargrp;
pub mod classinfo;
pub mod commandname;
pub mod creditgrp;
pub mod entereventgrp;
pub mod equipment;
pub mod etcitemgrp;
pub mod eula;
pub mod obscene;
pub mod warningnotice;
pub mod hairaccessarygrp;
pub mod hairaccessorylocgrp;
pub mod hairgrp;
pub mod helmetgrp;
pub mod hennagrp;
pub mod itemicons;
pub mod itemname;
pub mod logongrp;
pub mod mobskillanimgrp;
pub mod musicinfo;
pub mod npcgrp;
pub mod npcname;
pub mod questname;
pub mod reader;
pub mod recipe;
pub mod servername;
pub mod skillgrp;
pub mod skillname;
pub mod skillsoundgrp;
pub mod staticobject;
pub mod symbolname;
pub mod sysstring;
pub mod systemmsg;
pub mod weapongrp;
pub mod zonename;

use std::collections::HashMap;
use thiserror::Error;

pub use actionname::Action;
pub use animnames::{AnimKind, AnimSet};
pub use armorgrp::Armor;
pub use chargrp::CharTemplate;
pub use chronicle::Chronicle;
pub use classinfo::ClassInfo;
pub use commandname::CommandName;
pub use equipment::{Equipped, Item, Items, SubMesh};
pub use etcitemgrp::EtcItem;
pub use hairaccessorylocgrp::HairAccessoryLoc;
pub use hairgrp::HairRow;
pub use helmetgrp::HelmetHair;
pub use hennagrp::HennaGrp;
pub use itemicons::{ItemIcon, ItemIcons, ItemKind};
pub use itemname::ItemName;
pub use logongrp::{Logongrp, Spot};
pub use mobskillanimgrp::NpcSkillAnim;
pub use npcgrp::{Npc, NpcTable};
pub use npcname::NpcName;
pub use questname::QuestName;
pub use reader::DatReader;
pub use servername::ServerName;
pub use skillgrp::{Skill, SkillTable};
pub use skillname::SkillName;
pub use sysstring::SysString;
pub use systemmsg::SystemMessage;
pub use weapongrp::Weapon;
pub use zonename::ZoneName;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    #[error("dados truncados em {0:#x}")]
    Truncated(usize),
    #[error("índice compacto inválido em {0:#x}")]
    BadCompactIndex(usize),
    #[error("texto inválido em {0:#x}")]
    InvalidString(usize),
    #[error("contagem negativa em {0:#x}")]
    NegativeCount(usize),
}

impl From<ue2_core::ReadError> for Error {
    fn from(error: ue2_core::ReadError) -> Self {
        match error {
            ue2_core::ReadError::UnexpectedEof(at) => Self::Truncated(at),
            ue2_core::ReadError::BadCompactIndex(at) => Self::BadCompactIndex(at),
            ue2_core::ReadError::InvalidString(at) => Self::InvalidString(at),
        }
    }
}

pub fn by_id<T>(rows: impl IntoIterator<Item = T>, id: impl Fn(&T) -> u32) -> HashMap<u32, T> {
    rows.into_iter().map(|row| (id(&row), row)).collect()
}
