// Gerado por scripts/skill_effects_rust_gen.py --explosion a partir de scripts/skill_effects_oracle_gen.py (execução de
// ANSkillProjectile::SkillEffectExplosion 0x104be0a0 do engine.dll C4 no emulador) e scripts/asm_lift.py. Não editar à mão.

pub const EXPLOSION_HIT_NULL: u8 = 1;
pub const EXPLOSION_OWNER_NULL: u8 = 2;
pub const EXPLOSION_DAMAGE_EFFECT: u8 = 4;
pub const EXPLOSION_AREA: u8 = 8;
pub const EXPLOSION_SELF_HIT: u8 = 16;
pub const EXPLOSION_HIT_MOVER: u8 = 32;

static NODES: &[Node] = &[
    Node::Slot { party: Party::Projectile, offset: 0x4b0 },
    Node::Slot { party: Party::Hit, offset: 0x2f0 },
    Node::Int(1067030938),
    Node::Int(0),
    Node::Normal(0),
    Node::Normal(1),
    Node::Normal(2),
    Node::Rotation { axis: 0, vector: [4, 5, 6] },
    Node::Rotation { axis: 1, vector: [4, 5, 6] },
    Node::Rotation { axis: 2, vector: [4, 5, 6] },
    Node::Tvb { axis: 0, vector: [2, 3, 3], rotator: [7, 8, 9] },
    Node::Op(Op::Mul, 1, 10),
    Node::ToF32(11),
    Node::Op(Op::Sub, 0, 12),
    Node::ToF32(13),
    Node::Slot { party: Party::Projectile, offset: 0x4b4 },
    Node::Tvb { axis: 1, vector: [2, 3, 3], rotator: [7, 8, 9] },
    Node::Op(Op::Mul, 16, 1),
    Node::ToF32(17),
    Node::Op(Op::Sub, 15, 18),
    Node::ToF32(19),
    Node::Slot { party: Party::Projectile, offset: 0x4b8 },
    Node::Tvb { axis: 2, vector: [2, 3, 3], rotator: [7, 8, 9] },
    Node::Op(Op::Mul, 22, 1),
    Node::ToF32(23),
    Node::Op(Op::Sub, 21, 24),
    Node::ToF32(25),
    Node::Float(1.0),
    Node::Slot { party: Party::Hit, offset: 0x1bc },
    Node::Float(-2.0),
    Node::Op(Op::Mul, 1, 29),
    Node::ToF32(30),
    Node::Tvb { axis: 0, vector: [31, 3, 3], rotator: [7, 8, 9] },
    Node::Op(Op::Add, 28, 32),
    Node::ToF32(33),
    Node::Tvb { axis: 1, vector: [31, 3, 3], rotator: [7, 8, 9] },
    Node::Slot { party: Party::Hit, offset: 0x1c0 },
    Node::Op(Op::Add, 35, 36),
    Node::ToF32(37),
    Node::Tvb { axis: 2, vector: [31, 3, 3], rotator: [7, 8, 9] },
    Node::Slot { party: Party::Hit, offset: 0x1c4 },
    Node::Op(Op::Add, 39, 40),
    Node::ToF32(41),
    Node::Int(1065353216),
    Node::Slot { party: Party::Area, offset: 0x1bc },
    Node::Slot { party: Party::Area, offset: 0x2f0 },
    Node::Op(Op::Mul, 45, 29),
    Node::ToF32(46),
    Node::Slot { party: Party::Caster, offset: 0x1bc },
    Node::Op(Op::Sub, 44, 48),
    Node::ToF32(49),
    Node::Slot { party: Party::Area, offset: 0x1c0 },
    Node::Slot { party: Party::Caster, offset: 0x1c0 },
    Node::Op(Op::Sub, 51, 52),
    Node::ToF32(53),
    Node::Slot { party: Party::Area, offset: 0x1c4 },
    Node::Slot { party: Party::Caster, offset: 0x1c4 },
    Node::Op(Op::Sub, 55, 56),
    Node::ToF32(57),
    Node::Rotation { axis: 0, vector: [50, 54, 58] },
    Node::Rotation { axis: 1, vector: [50, 54, 58] },
    Node::Rotation { axis: 2, vector: [50, 54, 58] },
    Node::Tvb { axis: 0, vector: [47, 3, 3], rotator: [59, 60, 61] },
    Node::Op(Op::Add, 44, 62),
    Node::ToF32(63),
    Node::Tvb { axis: 1, vector: [47, 3, 3], rotator: [59, 60, 61] },
    Node::Op(Op::Add, 51, 65),
    Node::ToF32(66),
    Node::Tvb { axis: 2, vector: [47, 3, 3], rotator: [59, 60, 61] },
    Node::Op(Op::Add, 55, 68),
    Node::ToF32(69),
    Node::Op(Op::Add, 12, 12),
    Node::ToF32(71),
    Node::Op(Op::Sub, 28, 72),
    Node::ToF32(73),
    Node::Op(Op::Mul, 1, 16),
    Node::ToF32(75),
    Node::Op(Op::Add, 76, 76),
    Node::ToF32(77),
    Node::Op(Op::Sub, 36, 78),
    Node::ToF32(79),
    Node::Op(Op::Add, 23, 23),
    Node::Op(Op::Sub, 40, 81),
    Node::ToF32(82),
    Node::Int(1050253722),
    Node::Int(1106247680),
    Node::Op(Op::Sub, 0, 72),
    Node::ToF32(86),
    Node::Op(Op::Sub, 15, 78),
    Node::ToF32(88),
    Node::Op(Op::Sub, 21, 81),
    Node::ToF32(90),
    Node::Slot { party: Party::Caster, offset: 0x600 },
    Node::Op(Op::Mul, 10, 1),
    Node::ToF32(93),
    Node::Op(Op::Add, 94, 94),
    Node::ToF32(95),
    Node::Op(Op::Sub, 0, 96),
    Node::ToF32(97),
    Node::Op(Op::Add, 18, 18),
    Node::ToF32(99),
    Node::Op(Op::Sub, 15, 100),
    Node::ToF32(101),
    Node::Slot { party: Party::Target, offset: 0x600 },
    Node::Slot { party: Party::Target, offset: 0x1bc },
    Node::Slot { party: Party::Target, offset: 0x1c0 },
    Node::Slot { party: Party::Target, offset: 0x1c4 },
    Node::Slot { party: Party::Target, offset: 0x2f4 },
    Node::Op(Op::Sub, 106, 107),
    Node::ToF32(108),
    Node::Int(1086324736),
    Node::Int(1176256512),
    Node::Int(1123024896),
    Node::Random(1),
    Node::ToF32(113),
    Node::Random(0),
    Node::ToF32(115),
    Node::Op(Op::Mul, 116, 116),
    Node::Op(Op::Mul, 114, 114),
    Node::Op(Op::Add, 117, 118),
    Node::Sqrt(119),
    Node::Op(Op::Div, 27, 120),
    Node::Op(Op::Mul, 114, 121),
    Node::ToF32(122),
    Node::Int(1120403456),
    Node::Op(Op::Div, 123, 124),
    Node::Op(Op::Mul, 116, 121),
    Node::ToF32(126),
    Node::Op(Op::Div, 127, 124),
    Node::Op(Op::Div, 3, 124),
    Node::Int(1128792064),
    Node::Op(Op::Mul, 130, 123),
    Node::Op(Op::Add, 131, 131),
    Node::ToF32(132),
    Node::Op(Op::Mul, 130, 127),
    Node::Op(Op::Add, 134, 134),
    Node::ToF32(135),
    Node::Op(Op::Mul, 130, 3),
    Node::Op(Op::Add, 137, 137),
    Node::ToF32(138),
    Node::Int(1157234688),
    Node::Tvb { axis: 0, vector: [43, 3, 3], rotator: [7, 8, 9] },
    Node::Op(Op::Sub, 0, 141),
    Node::ToF32(142),
    Node::Tvb { axis: 1, vector: [43, 3, 3], rotator: [7, 8, 9] },
    Node::Op(Op::Sub, 15, 144),
    Node::ToF32(145),
    Node::Tvb { axis: 2, vector: [43, 3, 3], rotator: [7, 8, 9] },
    Node::Op(Op::Sub, 21, 147),
    Node::ToF32(148),
    Node::Op(Op::Mul, 121, 114),
    Node::ToF32(150),
    Node::Op(Op::Div, 151, 124),
    Node::Op(Op::Mul, 130, 151),
    Node::Op(Op::Add, 153, 153),
    Node::ToF32(154),
    Node::ToF32(103),
    Node::Neg(4),
    Node::ToF32(157),
    Node::Neg(5),
    Node::ToF32(159),
    Node::Rotation { axis: 0, vector: [158, 160, 3] },
    Node::Rotation { axis: 1, vector: [158, 160, 3] },
    Node::Rotation { axis: 2, vector: [158, 160, 3] },
    Node::Slot { party: Party::Hit, offset: 0x410 },
    Node::Slot { party: Party::Hit, offset: 0x414 },
    Node::Slot { party: Party::Hit, offset: 0x418 },
    Node::Rotation { axis: 0, vector: [164, 165, 166] },
    Node::Rotation { axis: 1, vector: [164, 165, 166] },
    Node::Rotation { axis: 2, vector: [164, 165, 166] },
];

pub static SITES: &[Site] = &[
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Hit, mode: 0, vector: [34, 38, 42], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Area, mode: 0, vector: [64, 67, 70], rotator: [59, 60, 61], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [74, 80, 83], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [74, 80, 83], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [74, 80, 83], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [74, 80, 83], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [87, 89, 91], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [87, 89, 91], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 1, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [87, 89, 91], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [87, 89, 91], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [87, 89, 91], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [87, 89, 91], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 1, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [87, 89, 91], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [87, 89, 91], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [74, 80, 83], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [74, 80, 83], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [74, 80, 83], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [74, 80, 83], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [44, 51, 55], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [44, 51, 55], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [44, 51, 55], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [74, 80, 83], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [74, 80, 83], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [74, 80, 83], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [74, 80, 83], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [87, 89, 91], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [87, 89, 91], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 1, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [87, 89, 91], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [87, 89, 91], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [74, 80, 83], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [74, 80, 83], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [74, 80, 83], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [74, 80, 83], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 92, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [87, 89, 91], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [87, 89, 91], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 92, flags: [0, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [87, 89, 91], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [87, 89, 91], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 92, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [98, 102, 91], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [98, 102, 91], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 43, 43, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 92, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 103, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [74, 80, 83], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [74, 80, 83], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [74, 80, 83], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [74, 80, 83], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 1, vector: [104, 105, 109], rotator: [3, 3, 3], scale: 92, flags: [0, 1, 0, 0] },
    Site::ViewShake { kind: 0, values: [110, 43, 111, 112, 125, 128, 129, 133, 136, 139, 112, 112, 3, 104, 105, 109, 130, 140] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 1, vector: [143, 146, 149], rotator: [3, 3, 3], scale: 92, flags: [0, 1, 0, 0] },
    Site::ViewShake { kind: 0, values: [110, 43, 111, 112, 152, 128, 129, 155, 136, 139, 112, 112, 3, 143, 146, 149, 130, 140] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 1, vector: [104, 105, 109], rotator: [3, 3, 3], scale: 156, flags: [0, 1, 0, 0] },
    Site::ViewShake { kind: 0, values: [110, 43, 111, 112, 125, 128, 129, 133, 136, 139, 112, 112, 3, 104, 105, 109, 130, 140] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Hit, mode: 0, vector: [0, 15, 21], rotator: [161, 162, 163], scale: 43, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [161, 162, 163], scale: 43, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Hit, mode: 0, vector: [0, 15, 21], rotator: [167, 168, 169], scale: 43, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 92, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [98, 102, 91], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [98, 102, 91], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 43, 43, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 92, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 103, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Area, instigator: Party::Area, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [74, 80, 83], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [74, 80, 83], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 43, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [74, 80, 83], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [74, 80, 83], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 3, 3, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 92, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [98, 102, 91], rotator: [7, 8, 9], kind: 0 },
    Site::LightInit { kind: 0, vector: [98, 102, 91], rotator: [7, 8, 9], byte_a: 1, byte_b: 12, plane: [43, 43, 43, 43], float_a: 84, float_b: 85, byte_c: 0 },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 92, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 103, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Hit, instigator: Party::Hit, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 43, flags: [1, 1, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 15, 21], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::PawnLight { pawn: Party::Hit, vector: [14, 20, 26], rotator: [7, 8, 9], kind: 0 },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Hit, instigator: Party::Null, mode: 0, vector: [14, 20, 26], rotator: [7, 8, 9], scale: 27, flags: [0, 0, 0, 0] },
];

const L0: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 0 },
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 1 },
    Step::Sound { mode: 3 },
];
const L1: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 2 },
    Step::Sound { mode: 3 },
];
const L2: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 0 },
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 1 },
];
const L3: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 2 },
];
const L4: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 3 },
    Step::Sound { mode: 3 },
];
const L5: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 3 },
];
const L6: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 4 },
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 5 },
    Step::Sound { mode: 3 },
];
const L7: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 6 },
    Step::Sound { mode: 3 },
];
const L8: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 7 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 8 },
];
const L9: &[Step] = &[
];
const L10: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 4 },
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 5 },
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 9 },
    Step::Sound { mode: 3 },
];
const L11: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 6 },
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 10 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L12: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 7 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 8 },
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 11 },
    Step::Attach(Bone::Index(0)),
];
const L13: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 12 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 13 },
    Step::Sound { mode: 3 },
];
const L14: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 12 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 13 },
];
const L15: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 12 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 13 },
    Step::DamageEffect { site: 14 },
    Step::Sound { mode: 3 },
];
const L16: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 12 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 13 },
    Step::DamageEffect { site: 14 },
];
const L17: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 15 },
    Step::Sound { mode: 3 },
];
const L18: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 15 },
];
const L19: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 16 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 17 },
    Step::Sound { mode: 3 },
];
const L20: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 18 },
    Step::Sound { mode: 3 },
];
const L21: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 19 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 20 },
];
const L22: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 16 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 17 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 21 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L23: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 18 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 22 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L24: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 19 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 20 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 23 },
    Step::Attach(Bone::Index(0)),
];
const L25: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 24 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 25 },
    Step::Sound { mode: 3 },
];
const L26: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 26 },
    Step::Sound { mode: 3 },
];
const L27: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 27 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 28 },
];
const L28: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 24 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 25 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 29 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L29: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 26 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 30 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L30: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 27 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 28 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 31 },
    Step::Attach(Bone::Index(0)),
];
const L31: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 32 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 33 },
    Step::Sound { mode: 3 },
];
const L32: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 34 },
];
const L33: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 32 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 33 },
];
const L34: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 35 },
    Step::Sound { mode: 3 },
];
const L35: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 35 },
];
const L36: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 36 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 37 },
    Step::Sound { mode: 3 },
];
const L37: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 36 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 37 },
];
const L38: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 36 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 37 },
    Step::DamageEffect { site: 38 },
    Step::Sound { mode: 3 },
];
const L39: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 36 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 37 },
    Step::DamageEffect { site: 38 },
];
const L40: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 39 },
    Step::Sound { mode: 3 },
];
const L41: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 39 },
];
const L42: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 40 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 41 },
    Step::Sound { mode: 3 },
];
const L43: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 40 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 41 },
];
const L44: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 40 },
    Step::DamageEffect { site: 42 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 43 },
    Step::Sound { mode: 3 },
];
const L45: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 40 },
    Step::DamageEffect { site: 42 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 43 },
];
const L46: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 44 },
    Step::Sound { mode: 3 },
];
const L47: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 44 },
];
const L48: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 45 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 46 },
    Step::LightInit { site: 47 },
    Step::Sound { mode: 3 },
];
const L49: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 48 },
    Step::Sound { mode: 3 },
];
const L50: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 49 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 50 },
    Step::LightInit { site: 51 },
];
const L51: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 45 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L52: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 49 },
    Step::Attach(Bone::Index(0)),
];
const L53: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 52 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 53 },
    Step::Sound { mode: 3 },
];
const L54: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 52 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 53 },
];
const L55: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 52 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 53 },
    Step::DamageEffect { site: 54 },
    Step::Sound { mode: 3 },
];
const L56: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 52 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 53 },
    Step::DamageEffect { site: 54 },
];
const L57: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 55 },
    Step::Sound { mode: 3 },
];
const L58: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 55 },
];
const L59: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 56 },
    Step::Spawn { class: "LineageEffect.s_u509_e", site: 57 },
    Step::Sound { mode: 3 },
];
const L60: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 56 },
    Step::Spawn { class: "LineageEffect.s_u509_e", site: 57 },
];
const L61: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 56 },
    Step::Spawn { class: "LineageEffect.s_u509_e", site: 57 },
    Step::DamageEffect { site: 58 },
    Step::Sound { mode: 3 },
];
const L62: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 56 },
    Step::Spawn { class: "LineageEffect.s_u509_e", site: 57 },
    Step::DamageEffect { site: 58 },
];
const L63: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.s_u509_e", site: 59 },
    Step::Sound { mode: 3 },
];
const L64: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.s_u509_e", site: 59 },
];
const L65: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 60 },
    Step::Spawn { class: "LineageEffect.s_u509_e", site: 61 },
    Step::Sound { mode: 3 },
];
const L66: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 60 },
    Step::Spawn { class: "LineageEffect.s_u509_e", site: 61 },
];
const L67: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 60 },
    Step::Spawn { class: "LineageEffect.s_u509_e", site: 61 },
    Step::DamageEffect { site: 62 },
    Step::Sound { mode: 3 },
];
const L68: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 60 },
    Step::Spawn { class: "LineageEffect.s_u509_e", site: 61 },
    Step::DamageEffect { site: 62 },
];
const L69: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.s_u509_e", site: 63 },
    Step::Sound { mode: 3 },
];
const L70: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.s_u509_e", site: 63 },
];
const L71: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 64 },
    Step::Spawn { class: "LineageEffect.sp_agility_ta", site: 65 },
    Step::TrailerPhysics { physics: 10, pre_pivot: true, same_rotation: false },
    Step::Sound { mode: 3 },
];
const L72: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 64 },
    Step::Spawn { class: "LineageEffect.sp_agility_ta", site: 65 },
    Step::TrailerPhysics { physics: 10, pre_pivot: true, same_rotation: false },
];
const L73: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 64 },
    Step::Spawn { class: "LineageEffect.sp_agility_ta", site: 65 },
    Step::TrailerPhysics { physics: 10, pre_pivot: true, same_rotation: false },
    Step::DamageEffect { site: 66 },
    Step::Sound { mode: 3 },
];
const L74: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 64 },
    Step::Spawn { class: "LineageEffect.sp_agility_ta", site: 65 },
    Step::TrailerPhysics { physics: 10, pre_pivot: true, same_rotation: false },
    Step::DamageEffect { site: 66 },
];
const L75: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.sp_agility_ta", site: 67 },
    Step::TrailerPhysics { physics: 10, pre_pivot: true, same_rotation: false },
    Step::Sound { mode: 3 },
];
const L76: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.sp_agility_ta", site: 67 },
    Step::TrailerPhysics { physics: 10, pre_pivot: true, same_rotation: false },
];
const L77: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_c", site: 68 },
    Step::PawnLight { site: 69 },
    Step::LightInit { site: 70 },
    Step::Sound { mode: 3 },
];
const L78: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_c", site: 71 },
    Step::Sound { mode: 3 },
];
const L79: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_c", site: 72 },
    Step::PawnLight { site: 73 },
    Step::LightInit { site: 74 },
];
const L80: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_c", site: 68 },
    Step::Sound { mode: 3 },
];
const L81: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_c", site: 72 },
];
const L82: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_c", site: 75 },
    Step::PawnLight { site: 76 },
    Step::LightInit { site: 77 },
    Step::Sound { mode: 3 },
];
const L83: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_c", site: 78 },
    Step::Sound { mode: 3 },
];
const L84: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_c", site: 79 },
    Step::PawnLight { site: 80 },
    Step::LightInit { site: 81 },
];
const L85: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_c", site: 75 },
    Step::Sound { mode: 3 },
];
const L86: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_c", site: 79 },
];
const L87: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 82 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 83 },
    Step::Sound { mode: 3 },
];
const L88: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 84 },
];
const L89: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 82 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 83 },
];
const L90: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 85 },
    Step::Sound { mode: 3 },
];
const L91: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 85 },
];
const L92: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 86 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 87 },
    Step::LightInit { site: 88 },
    Step::Sound { mode: 3 },
];
const L93: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 89 },
    Step::Sound { mode: 3 },
];
const L94: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 90 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 91 },
    Step::LightInit { site: 92 },
];
const L95: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 86 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L96: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 90 },
    Step::Attach(Bone::Index(0)),
];
const L97: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 93 },
    Step::Spawn { class: "LineageEffect.m_u039_e", site: 94 },
    Step::Sound { mode: 3 },
];
const L98: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u039_e", site: 95 },
    Step::Sound { mode: 3 },
];
const L99: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 93 },
    Step::Spawn { class: "LineageEffect.m_u039_e", site: 94 },
];
const L100: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u039_e", site: 95 },
];
const L101: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u039_e", site: 96 },
    Step::Sound { mode: 3 },
];
const L102: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u039_e", site: 96 },
];
const L103: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 97 },
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 98 },
    Step::Sound { mode: 3 },
];
const L104: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 99 },
    Step::Sound { mode: 3 },
];
const L105: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 97 },
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 98 },
];
const L106: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 99 },
];
const L107: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 97 },
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 100 },
    Step::Sound { mode: 3 },
];
const L108: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 101 },
    Step::Sound { mode: 3 },
];
const L109: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 97 },
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 100 },
];
const L110: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 101 },
];
const L111: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 102 },
    Step::Sound { mode: 3 },
];
const L112: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 102 },
];
const L113: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 103 },
    Step::Sound { mode: 3 },
];
const L114: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 103 },
];
const L115: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 104 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 105 },
    Step::Sound { mode: 3 },
];
const L116: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 106 },
];
const L117: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 104 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 105 },
];
const L118: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 107 },
    Step::Sound { mode: 3 },
];
const L119: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 107 },
];
const L120: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 108 },
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 109 },
    Step::Sound { mode: 3 },
];
const L121: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 110 },
    Step::Sound { mode: 3 },
];
const L122: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 108 },
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 109 },
];
const L123: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 110 },
];
const L124: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 111 },
    Step::Sound { mode: 3 },
];
const L125: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 111 },
];
const L126: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 112 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 113 },
    Step::Sound { mode: 3 },
];
const L127: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 114 },
    Step::Sound { mode: 3 },
];
const L128: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 115 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 116 },
];
const L129: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 112 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 113 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 117 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L130: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 114 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 118 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L131: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 115 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 116 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 119 },
    Step::Attach(Bone::Index(0)),
];
const L132: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 120 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 121 },
    Step::LightInit { site: 122 },
    Step::Sound { mode: 3 },
];
const L133: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 123 },
    Step::Sound { mode: 3 },
];
const L134: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 124 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 125 },
    Step::LightInit { site: 126 },
];
const L135: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 120 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L136: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 124 },
    Step::Attach(Bone::Index(0)),
];
const L137: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 127 },
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 128 },
    Step::Sound { mode: 3 },
];
const L138: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 129 },
    Step::Sound { mode: 3 },
];
const L139: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 127 },
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 128 },
];
const L140: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 129 },
];
const L141: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 130 },
    Step::Sound { mode: 3 },
];
const L142: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 130 },
];
const L143: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 131 },
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 132 },
    Step::Sound { mode: 3 },
];
const L144: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 133 },
    Step::Sound { mode: 3 },
];
const L145: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 131 },
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 132 },
];
const L146: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 133 },
];
const L147: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 134 },
    Step::Sound { mode: 3 },
];
const L148: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 134 },
];
const L149: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_c", site: 135 },
    Step::PawnLight { site: 136 },
    Step::LightInit { site: 137 },
    Step::Sound { mode: 3 },
];
const L150: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_c", site: 138 },
    Step::Sound { mode: 3 },
];
const L151: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_c", site: 139 },
    Step::PawnLight { site: 140 },
    Step::LightInit { site: 141 },
];
const L152: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_c", site: 135 },
    Step::Sound { mode: 3 },
];
const L153: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_c", site: 139 },
];
const L154: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 142 },
    Step::Spawn { class: "LineageEffect.m_u039_e", site: 143 },
    Step::Sound { mode: 3 },
];
const L155: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u039_e", site: 144 },
    Step::Sound { mode: 3 },
];
const L156: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 142 },
    Step::Spawn { class: "LineageEffect.m_u039_e", site: 143 },
];
const L157: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u039_e", site: 144 },
];
const L158: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u039_e", site: 145 },
    Step::Sound { mode: 3 },
];
const L159: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u039_e", site: 145 },
];
const L160: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 146 },
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 147 },
    Step::Sound { mode: 3 },
];
const L161: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 148 },
    Step::Sound { mode: 3 },
];
const L162: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 146 },
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 147 },
];
const L163: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 148 },
];
const L164: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 149 },
    Step::Sound { mode: 3 },
];
const L165: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 149 },
];
const L166: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 150 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 151 },
    Step::Sound { mode: 3 },
];
const L167: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 152 },
];
const L168: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 150 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 151 },
];
const L169: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 153 },
    Step::Sound { mode: 3 },
];
const L170: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 153 },
];
const L171: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 154 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 155 },
    Step::Sound { mode: 3 },
];
const L172: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 156 },
];
const L173: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 154 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 155 },
];
const L174: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 157 },
    Step::Sound { mode: 3 },
];
const L175: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 157 },
];
const L176: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 158 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 159 },
    Step::Sound { mode: 3 },
];
const L177: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 160 },
    Step::Sound { mode: 3 },
];
const L178: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 161 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 162 },
];
const L179: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 158 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 159 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 163 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L180: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 160 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 164 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L181: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 161 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 162 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 165 },
    Step::Attach(Bone::Index(0)),
];
const L182: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 166 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 167 },
    Step::Sound { mode: 3 },
];
const L183: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 166 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 167 },
];
const L184: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 166 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 167 },
    Step::DamageEffect { site: 168 },
    Step::Sound { mode: 3 },
];
const L185: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 166 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 167 },
    Step::DamageEffect { site: 168 },
];
const L186: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 169 },
    Step::Sound { mode: 3 },
];
const L187: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 169 },
];
const L188: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 170 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 171 },
    Step::LightInit { site: 172 },
    Step::Sound { mode: 3 },
];
const L189: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 173 },
    Step::Sound { mode: 3 },
];
const L190: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 174 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 175 },
    Step::LightInit { site: 176 },
];
const L191: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 170 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L192: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 174 },
    Step::Attach(Bone::Index(0)),
];
const L193: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 177 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 178 },
    Step::Sound { mode: 3 },
];
const L194: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 179 },
    Step::Sound { mode: 3 },
];
const L195: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 180 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 181 },
];
const L196: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 177 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 178 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 182 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L197: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 179 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 183 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L198: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 180 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 181 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 184 },
    Step::Attach(Bone::Index(0)),
];
const L199: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 185 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 186 },
    Step::Sound { mode: 3 },
];
const L200: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 187 },
];
const L201: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 185 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 186 },
];
const L202: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 188 },
    Step::Sound { mode: 3 },
];
const L203: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 188 },
];
const L204: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 189 },
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 190 },
    Step::Sound { mode: 3 },
];
const L205: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 191 },
    Step::Sound { mode: 3 },
];
const L206: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 189 },
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 190 },
];
const L207: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 191 },
];
const L208: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 192 },
    Step::Sound { mode: 3 },
];
const L209: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 192 },
];
const L210: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u033_c", site: 193 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 194 },
    Step::LightInit { site: 195 },
    Step::Sound { mode: 3 },
];
const L211: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u033_c", site: 196 },
    Step::Sound { mode: 3 },
];
const L212: &[Step] = &[
    Step::PawnLight { site: 197 },
    Step::LightInit { site: 198 },
];
const L213: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u033_c", site: 193 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L214: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_c", site: 199 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 200 },
    Step::LightInit { site: 201 },
    Step::Sound { mode: 3 },
];
const L215: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_c", site: 202 },
    Step::Sound { mode: 3 },
];
const L216: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_c", site: 203 },
    Step::Attach(Bone::Index(0)),
];
const L217: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_c", site: 199 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L218: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_c", site: 204 },
    Step::Attach(Bone::Index(0)),
];
const L219: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 205 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 206 },
    Step::Sound { mode: 3 },
];
const L220: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 207 },
    Step::Sound { mode: 3 },
];
const L221: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 208 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 209 },
];
const L222: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 205 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 206 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 210 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L223: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 207 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 211 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L224: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 208 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 209 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 212 },
    Step::Attach(Bone::Index(0)),
];
const L225: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 213 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 214 },
    Step::LightInit { site: 215 },
    Step::Sound { mode: 3 },
];
const L226: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 216 },
    Step::Sound { mode: 3 },
];
const L227: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 217 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 218 },
    Step::LightInit { site: 219 },
];
const L228: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 213 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L229: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 217 },
    Step::Attach(Bone::Index(0)),
];
const L230: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u046_c", site: 220 },
    Step::ViewShake { site: 221 },
    Step::Sound { mode: 3 },
];
const L231: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u046_c", site: 222 },
    Step::ViewShake { site: 223 },
    Step::Sound { mode: 3 },
];
const L232: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u046_c", site: 224 },
    Step::ViewShake { site: 225 },
];
const L233: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 226 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 227 },
    Step::Sound { mode: 3 },
];
const L234: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 226 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 227 },
];
const L235: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 226 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 227 },
    Step::DamageEffect { site: 228 },
    Step::Sound { mode: 3 },
];
const L236: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 226 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 227 },
    Step::DamageEffect { site: 228 },
];
const L237: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 229 },
    Step::Sound { mode: 3 },
];
const L238: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 229 },
];
const L239: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 230 },
    Step::Spawn { class: "LineageEffect.e_u058_b", site: 231 },
    Step::Sound { mode: 3 },
];
const L240: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u058_b", site: 232 },
    Step::Sound { mode: 3 },
];
const L241: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 230 },
    Step::Spawn { class: "LineageEffect.e_u058_b", site: 231 },
];
const L242: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u058_b", site: 232 },
];
const L243: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.e_u058_b", site: 233 },
    Step::Sound { mode: 3 },
];
const L244: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.e_u058_b", site: 233 },
];
const L245: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 234 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 235 },
    Step::Sound { mode: 3 },
];
const L246: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 236 },
];
const L247: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 234 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 235 },
];
const L248: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 237 },
    Step::Sound { mode: 3 },
];
const L249: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 237 },
];
const L250: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 238 },
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 239 },
    Step::Sound { mode: 3 },
];
const L251: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 240 },
    Step::Sound { mode: 3 },
];
const L252: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 238 },
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 239 },
];
const L253: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 240 },
];
const L254: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 241 },
    Step::Sound { mode: 3 },
];
const L255: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 241 },
];
const L256: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 242 },
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 243 },
    Step::Sound { mode: 3 },
];
const L257: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 244 },
    Step::Sound { mode: 3 },
];
const L258: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 242 },
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 243 },
];
const L259: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 244 },
];
const L260: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 245 },
    Step::Sound { mode: 3 },
];
const L261: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 245 },
];
const L262: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 246 },
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 247 },
    Step::Sound { mode: 3 },
];
const L263: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 248 },
    Step::Sound { mode: 3 },
];
const L264: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 246 },
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 247 },
];
const L265: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 248 },
];
const L266: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 249 },
    Step::Sound { mode: 3 },
];
const L267: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u037_e", site: 249 },
];
const L268: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 250 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 251 },
    Step::Sound { mode: 3 },
];
const L269: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 250 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 251 },
];
const L270: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 250 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 251 },
    Step::DamageEffect { site: 252 },
    Step::Sound { mode: 3 },
];
const L271: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 250 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 251 },
    Step::DamageEffect { site: 252 },
];
const L272: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 253 },
    Step::Sound { mode: 3 },
];
const L273: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 253 },
];
const L274: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 254 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 255 },
    Step::Sound { mode: 3 },
];
const L275: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 254 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 255 },
];
const L276: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 254 },
    Step::DamageEffect { site: 256 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 257 },
    Step::Sound { mode: 3 },
];
const L277: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 254 },
    Step::DamageEffect { site: 256 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 257 },
];
const L278: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 258 },
    Step::Sound { mode: 3 },
];
const L279: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 258 },
];
const L280: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 259 },
    Step::Spawn { class: "LineageEffect.e_u058_b", site: 260 },
    Step::Sound { mode: 3 },
];
const L281: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u058_b", site: 261 },
    Step::Sound { mode: 3 },
];
const L282: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 259 },
    Step::Spawn { class: "LineageEffect.e_u058_b", site: 260 },
];
const L283: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u058_b", site: 261 },
];
const L284: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.e_u058_b", site: 262 },
    Step::Sound { mode: 3 },
];
const L285: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.e_u058_b", site: 262 },
];
const L286: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u073_ground", site: 263 },
    Step::Sound { mode: 3 },
];
const L287: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u073_ground", site: 264 },
    Step::Sound { mode: 3 },
];
const L288: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u073_ground", site: 263 },
];
const L289: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u073_ground", site: 264 },
];
const L290: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u073_door", site: 265 },
    Step::Sound { mode: 3 },
];
const L291: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u073_door", site: 265 },
];
const L292: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u072_b", site: 266 },
    Step::Sound { mode: 3 },
];
const L293: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u072_b", site: 267 },
    Step::Sound { mode: 3 },
];
const L294: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u072_b", site: 266 },
];
const L295: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u072_b", site: 267 },
];
const L296: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 268 },
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 269 },
    Step::Sound { mode: 3 },
];
const L297: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 270 },
    Step::Sound { mode: 3 },
];
const L298: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 268 },
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 269 },
];
const L299: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 270 },
];
const L300: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 271 },
    Step::Sound { mode: 3 },
];
const L301: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u038_d", site: 271 },
];
const L302: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 272 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 273 },
    Step::Sound { mode: 3 },
];
const L303: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 274 },
];
const L304: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 272 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 273 },
];
const L305: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 275 },
    Step::Sound { mode: 3 },
];
const L306: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 275 },
];
const L307: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 276 },
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 277 },
    Step::Sound { mode: 3 },
];
const L308: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 278 },
    Step::Sound { mode: 3 },
];
const L309: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 276 },
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 277 },
];
const L310: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 278 },
];
const L311: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 279 },
    Step::Sound { mode: 3 },
];
const L312: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u026_d", site: 279 },
];
const L313: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_c", site: 280 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 281 },
    Step::LightInit { site: 282 },
    Step::Sound { mode: 3 },
];
const L314: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_c", site: 283 },
    Step::Sound { mode: 3 },
];
const L315: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_c", site: 284 },
    Step::Attach(Bone::Index(0)),
];
const L316: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_c", site: 280 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L317: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_c", site: 285 },
    Step::Attach(Bone::Index(0)),
];
const L318: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 286 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 287 },
    Step::Sound { mode: 3 },
];
const L319: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 288 },
    Step::Sound { mode: 3 },
];
const L320: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 289 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 290 },
];
const L321: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 286 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 287 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 291 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L322: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 288 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 292 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L323: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 289 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 290 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 293 },
    Step::Attach(Bone::Index(0)),
];
const L324: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 294 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 295 },
    Step::LightInit { site: 296 },
    Step::Sound { mode: 3 },
];
const L325: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 297 },
    Step::Sound { mode: 3 },
];
const L326: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 298 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 299 },
    Step::LightInit { site: 300 },
];
const L327: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 294 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L328: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 298 },
    Step::Attach(Bone::Index(0)),
];
const L329: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_c", site: 301 },
    Step::Attach(Bone::Index(0)),
    Step::PawnLight { site: 302 },
    Step::LightInit { site: 303 },
    Step::Sound { mode: 3 },
];
const L330: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_c", site: 304 },
    Step::Sound { mode: 3 },
];
const L331: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_c", site: 305 },
    Step::Attach(Bone::Index(0)),
];
const L332: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_c", site: 301 },
    Step::Attach(Bone::Index(0)),
    Step::Sound { mode: 3 },
];
const L333: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_c", site: 306 },
    Step::Attach(Bone::Index(0)),
];
const L334: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 307 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 308 },
    Step::Sound { mode: 3 },
];
const L335: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 309 },
];
const L336: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 307 },
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 308 },
];
const L337: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 310 },
    Step::Sound { mode: 3 },
];
const L338: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.m_u000_d", site: 310 },
];
const L339: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 311 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 312 },
    Step::Sound { mode: 3 },
];
const L340: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 311 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 312 },
];
const L341: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 311 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 312 },
    Step::DamageEffect { site: 313 },
    Step::Sound { mode: 3 },
];
const L342: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 311 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 312 },
    Step::DamageEffect { site: 313 },
];
const L343: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 314 },
    Step::Sound { mode: 3 },
];
const L344: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 314 },
];
const L345: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 315 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 316 },
    Step::Sound { mode: 3 },
];
const L346: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 315 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 316 },
];
const L347: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 315 },
    Step::DamageEffect { site: 317 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 318 },
    Step::Sound { mode: 3 },
];
const L348: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 315 },
    Step::DamageEffect { site: 317 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 318 },
];
const L349: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 319 },
    Step::Sound { mode: 3 },
];
const L350: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 319 },
];
const L351: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 320 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 321 },
    Step::Sound { mode: 3 },
];
const L352: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 320 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 321 },
];
const L353: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 320 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 321 },
    Step::DamageEffect { site: 322 },
    Step::Sound { mode: 3 },
];
const L354: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 320 },
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 321 },
    Step::DamageEffect { site: 322 },
];
const L355: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 323 },
    Step::Sound { mode: 3 },
];
const L356: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 323 },
];
const L357: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 324 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 325 },
    Step::Sound { mode: 3 },
];
const L358: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 324 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 325 },
];
const L359: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 324 },
    Step::DamageEffect { site: 326 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 327 },
    Step::Sound { mode: 3 },
];
const L360: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::PawnLight { site: 324 },
    Step::DamageEffect { site: 326 },
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 327 },
];
const L361: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 328 },
    Step::Sound { mode: 3 },
];
const L362: &[Step] = &[
    Step::DamageAct(Party::Hit),
    Step::Spawn { class: "LineageEffect.s_u505_c", site: 328 },
];

const V_EXPLOSION_6: Variants = &[(0, L0), (1, L1), (2, L2), (3, L3), (32, L4), (33, L1), (34, L5), (35, L3)];
const V_EXPLOSION_7: Variants = &[(0, L6), (1, L7), (2, L8), (3, L9), (8, L10), (9, L11), (10, L12), (11, L9)];
const V_EXPLOSION_13: Variants = &[(0, L13), (1, L9), (2, L14), (3, L9), (4, L15), (5, L9), (6, L16), (7, L9), (32, L17), (33, L9), (34, L18), (35, L9), (36, L17), (37, L9), (38, L18), (39, L9)];
const V_EXPLOSION_18: Variants = &[(0, L19), (1, L20), (2, L21), (3, L9), (8, L22), (9, L23), (10, L24), (11, L9)];
const V_EXPLOSION_23: Variants = &[(0, L25), (1, L26), (2, L27), (3, L9), (8, L28), (9, L29), (10, L30), (11, L9)];
const V_EXPLOSION_36: Variants = &[(0, L31), (1, L32), (2, L33), (3, L32), (32, L34), (33, L32), (34, L35), (35, L32)];
const V_EXPLOSION_38: Variants = &[(0, L36), (1, L9), (2, L37), (3, L9), (4, L38), (5, L9), (6, L39), (7, L9), (32, L40), (33, L9), (34, L41), (35, L9), (36, L40), (37, L9), (38, L41), (39, L9)];
const V_EXPLOSION_65: Variants = &[(0, L42), (1, L9), (2, L43), (3, L9), (4, L44), (5, L9), (6, L45), (7, L9), (32, L46), (33, L9), (34, L47), (35, L9), (36, L46), (37, L9), (38, L47), (39, L9)];
const V_EXPLOSION_118: Variants = &[(0, L48), (1, L49), (2, L50), (3, L9), (32, L51), (33, L49), (34, L52), (35, L9)];
const V_EXPLOSION_13A: Variants = &[(0, L53), (1, L9), (2, L54), (3, L9), (4, L55), (5, L9), (6, L56), (7, L9), (32, L57), (33, L9), (34, L58), (35, L9), (36, L57), (37, L9), (38, L58), (39, L9)];
const V_EXPLOSION_157: Variants = &[(0, L59), (1, L9), (2, L60), (3, L9), (4, L61), (5, L9), (6, L62), (7, L9), (32, L63), (33, L9), (34, L64), (35, L9), (36, L63), (37, L9), (38, L64), (39, L9)];
const V_EXPLOSION_162: Variants = &[(0, L65), (1, L9), (2, L66), (3, L9), (4, L67), (5, L9), (6, L68), (7, L9), (32, L69), (33, L9), (34, L70), (35, L9), (36, L69), (37, L9), (38, L70), (39, L9)];
const V_EXPLOSION_171: Variants = &[(0, L71), (1, L9), (2, L72), (3, L9), (4, L73), (5, L9), (6, L74), (7, L9), (32, L75), (33, L9), (34, L76), (35, L9), (36, L75), (37, L9), (38, L76), (39, L9)];
const V_EXPLOSION_44C: Variants = &[(0, L77), (1, L78), (2, L79), (3, L9), (32, L80), (33, L78), (34, L81), (35, L9)];
const V_EXPLOSION_453: Variants = &[(0, L82), (1, L83), (2, L84), (3, L9), (32, L85), (33, L83), (34, L86), (35, L9)];
const V_EXPLOSION_47C: Variants = &[(0, L87), (1, L88), (2, L89), (3, L88), (32, L90), (33, L88), (34, L91), (35, L88)];
const V_EXPLOSION_487: Variants = &[(0, L92), (1, L93), (2, L94), (3, L9), (32, L95), (33, L93), (34, L96), (35, L9)];
const V_EXPLOSION_497: Variants = &[(0, L97), (1, L98), (2, L99), (3, L100), (32, L101), (33, L98), (34, L102), (35, L100)];
const V_EXPLOSION_498: Variants = &[(0, L103), (1, L104), (2, L105), (3, L106), (8, L107), (9, L108), (10, L109), (11, L110), (32, L111), (33, L104), (34, L112), (35, L106), (40, L113), (41, L108), (42, L114), (43, L110)];
const V_EXPLOSION_499: Variants = &[(0, L115), (1, L116), (2, L117), (3, L116), (32, L118), (33, L116), (34, L119), (35, L116)];
const V_EXPLOSION_49A: Variants = &[(0, L120), (1, L121), (2, L122), (3, L123), (32, L124), (33, L121), (34, L125), (35, L123)];
const V_EXPLOSION_49D: Variants = &[(0, L126), (1, L127), (2, L128), (3, L9), (8, L129), (9, L130), (10, L131), (11, L9)];
const V_EXPLOSION_4C4: Variants = &[(0, L132), (1, L133), (2, L134), (3, L9), (32, L135), (33, L133), (34, L136), (35, L9)];
const V_EXPLOSION_4CE: Variants = &[(0, L132), (1, L133), (2, L134), (3, L9), (32, L135), (33, L133), (34, L136), (35, L9)];
const V_EXPLOSION_4D5: Variants = &[(0, L137), (1, L138), (2, L139), (3, L140), (32, L141), (33, L138), (34, L142), (35, L140)];
const V_EXPLOSION_4D7: Variants = &[(0, L143), (1, L144), (2, L145), (3, L146), (32, L147), (33, L144), (34, L148), (35, L146)];
const V_EXPLOSION_4DC: Variants = &[(0, L149), (1, L150), (2, L151), (3, L9), (32, L152), (33, L150), (34, L153), (35, L9)];
const V_EXPLOSION_4F2: Variants = &[(0, L132), (1, L133), (2, L134), (3, L9), (32, L135), (33, L133), (34, L136), (35, L9)];
const V_EXPLOSION_4F3: Variants = &[(0, L132), (1, L133), (2, L134), (3, L9), (32, L135), (33, L133), (34, L136), (35, L9)];
const V_EXPLOSION_509: Variants = &[(0, L132), (1, L133), (2, L134), (3, L9), (32, L135), (33, L133), (34, L136), (35, L9)];
const V_EXPLOSION_50A: Variants = &[(0, L154), (1, L155), (2, L156), (3, L157), (32, L158), (33, L155), (34, L159), (35, L157)];
const V_EXPLOSION_50B: Variants = &[(0, L160), (1, L161), (2, L162), (3, L163), (32, L164), (33, L161), (34, L165), (35, L163)];
const V_EXPLOSION_FA1: Variants = &[(0, L166), (1, L167), (2, L168), (3, L167), (32, L169), (33, L167), (34, L170), (35, L167)];
const V_EXPLOSION_FB2: Variants = &[(0, L171), (1, L172), (2, L173), (3, L172), (32, L174), (33, L172), (34, L175), (35, L172)];
const V_EXPLOSION_FBA: Variants = &[(0, L176), (1, L177), (2, L178), (3, L9), (8, L179), (9, L180), (10, L181), (11, L9)];
const V_EXPLOSION_FC8: Variants = &[(0, L182), (1, L9), (2, L183), (3, L9), (4, L184), (5, L9), (6, L185), (7, L9), (32, L186), (33, L9), (34, L187), (35, L9), (36, L186), (37, L9), (38, L187), (39, L9)];
const V_EXPLOSION_FCA: Variants = &[(0, L188), (1, L189), (2, L190), (3, L9), (32, L191), (33, L189), (34, L192), (35, L9)];
const V_EXPLOSION_FCB: Variants = &[(0, L193), (1, L194), (2, L195), (3, L9), (8, L196), (9, L197), (10, L198), (11, L9)];
const V_EXPLOSION_FD1: Variants = &[(0, L199), (1, L200), (2, L201), (3, L200), (32, L202), (33, L200), (34, L203), (35, L200)];
const V_EXPLOSION_FE2: Variants = &[(0, L204), (1, L205), (2, L206), (3, L207), (32, L208), (33, L205), (34, L209), (35, L207)];
const V_EXPLOSION_FE4: Variants = &[(0, L210), (1, L211), (2, L212), (3, L9), (32, L213), (33, L211), (34, L9), (35, L9)];
const V_EXPLOSION_FE5: Variants = &[(0, L214), (1, L215), (2, L216), (3, L9), (32, L217), (33, L215), (34, L218), (35, L9)];
const V_EXPLOSION_FEE: Variants = &[(0, L219), (1, L220), (2, L221), (3, L9), (8, L222), (9, L223), (10, L224), (11, L9)];
const V_EXPLOSION_FF7: Variants = &[(0, L225), (1, L226), (2, L227), (3, L9), (32, L228), (33, L226), (34, L229), (35, L9)];
const V_EXPLOSION_1004: Variants = &[(0, L225), (1, L226), (2, L227), (3, L9), (32, L228), (33, L226), (34, L229), (35, L9)];
const V_EXPLOSION_1009: Variants = &[(0, L214), (1, L215), (2, L216), (3, L9), (32, L217), (33, L215), (34, L218), (35, L9)];
const V_EXPLOSION_100F: Variants = &[(0, L230), (1, L231), (2, L232), (3, L9)];
const V_EXPLOSION_1012: Variants = &[(0, L219), (1, L220), (2, L221), (3, L9), (8, L222), (9, L223), (10, L224), (11, L9)];
const V_EXPLOSION_1018: Variants = &[(0, L233), (1, L9), (2, L234), (3, L9), (4, L235), (5, L9), (6, L236), (7, L9), (32, L237), (33, L9), (34, L238), (35, L9), (36, L237), (37, L9), (38, L238), (39, L9)];
const V_EXPLOSION_101C: Variants = &[(0, L233), (1, L9), (2, L234), (3, L9), (4, L235), (5, L9), (6, L236), (7, L9), (32, L237), (33, L9), (34, L238), (35, L9), (36, L237), (37, L9), (38, L238), (39, L9)];
const V_EXPLOSION_102D: Variants = &[(0, L239), (1, L240), (2, L241), (3, L242), (32, L243), (33, L240), (34, L244), (35, L242)];
const V_EXPLOSION_102E: Variants = &[(0, L239), (1, L240), (2, L241), (3, L242), (32, L243), (33, L240), (34, L244), (35, L242)];
const V_EXPLOSION_1037: Variants = &[(0, L245), (1, L246), (2, L247), (3, L246), (32, L248), (33, L246), (34, L249), (35, L246)];
const V_EXPLOSION_103B: Variants = &[(0, L250), (1, L251), (2, L252), (3, L253), (32, L254), (33, L251), (34, L255), (35, L253)];
const V_EXPLOSION_103C: Variants = &[(0, L214), (1, L215), (2, L216), (3, L9), (32, L217), (33, L215), (34, L218), (35, L9)];
const V_EXPLOSION_103D: Variants = &[(0, L225), (1, L226), (2, L227), (3, L9), (32, L228), (33, L226), (34, L229), (35, L9)];
const V_EXPLOSION_103E: Variants = &[(0, L225), (1, L226), (2, L227), (3, L9), (32, L228), (33, L226), (34, L229), (35, L9)];
const V_EXPLOSION_103F: Variants = &[(0, L214), (1, L215), (2, L216), (3, L9), (32, L217), (33, L215), (34, L218), (35, L9)];
const V_EXPLOSION_1045: Variants = &[(0, L256), (1, L257), (2, L258), (3, L259), (32, L260), (33, L257), (34, L261), (35, L259)];
const V_EXPLOSION_1052: Variants = &[(0, L219), (1, L220), (2, L221), (3, L9), (8, L222), (9, L223), (10, L224), (11, L9)];
const V_EXPLOSION_105F: Variants = &[(0, L245), (1, L246), (2, L247), (3, L246), (32, L248), (33, L246), (34, L249), (35, L246)];
const V_EXPLOSION_1063: Variants = &[(0, L250), (1, L251), (2, L252), (3, L253), (32, L254), (33, L251), (34, L255), (35, L253)];
const V_EXPLOSION_106C: Variants = &[(0, L262), (1, L263), (2, L264), (3, L265), (32, L266), (33, L263), (34, L267), (35, L265)];
const V_EXPLOSION_106F: Variants = &[(0, L268), (1, L9), (2, L269), (3, L9), (4, L270), (5, L9), (6, L271), (7, L9), (32, L272), (33, L9), (34, L273), (35, L9), (36, L272), (37, L9), (38, L273), (39, L9)];
const V_EXPLOSION_1070: Variants = &[(0, L274), (1, L9), (2, L275), (3, L9), (4, L276), (5, L9), (6, L277), (7, L9), (32, L278), (33, L9), (34, L279), (35, L9), (36, L278), (37, L9), (38, L279), (39, L9)];
const V_EXPLOSION_1084: Variants = &[(0, L268), (1, L9), (2, L269), (3, L9), (4, L270), (5, L9), (6, L271), (7, L9), (32, L272), (33, L9), (34, L273), (35, L9), (36, L272), (37, L9), (38, L273), (39, L9)];
const V_EXPLOSION_1085: Variants = &[(0, L280), (1, L281), (2, L282), (3, L283), (32, L284), (33, L281), (34, L285), (35, L283)];
const V_EXPLOSION_1086: Variants = &[(0, L286), (1, L287), (2, L288), (3, L289), (32, L290), (33, L287), (34, L291), (35, L289)];
const V_EXPLOSION_1087: Variants = &[(0, L292), (1, L293), (2, L294), (3, L295)];
const V_EXPLOSION_108C: Variants = &[(0, L296), (1, L297), (2, L298), (3, L299), (32, L300), (33, L297), (34, L301), (35, L299)];
const V_EXPLOSION_1097: Variants = &[(0, L302), (1, L303), (2, L304), (3, L303), (32, L305), (33, L303), (34, L306), (35, L303)];
const V_EXPLOSION_109A: Variants = &[(0, L307), (1, L308), (2, L309), (3, L310), (32, L311), (33, L308), (34, L312), (35, L310)];
const V_EXPLOSION_109B: Variants = &[(0, L313), (1, L314), (2, L315), (3, L9), (32, L316), (33, L314), (34, L317), (35, L9)];
const V_EXPLOSION_109C: Variants = &[(0, L318), (1, L319), (2, L320), (3, L9), (8, L321), (9, L322), (10, L323), (11, L9)];
const V_EXPLOSION_109D: Variants = &[(0, L324), (1, L325), (2, L326), (3, L9), (32, L327), (33, L325), (34, L328), (35, L9)];
const V_EXPLOSION_109E: Variants = &[(0, L324), (1, L325), (2, L326), (3, L9), (32, L327), (33, L325), (34, L328), (35, L9)];
const V_EXPLOSION_109F: Variants = &[(0, L329), (1, L330), (2, L331), (3, L9), (32, L332), (33, L330), (34, L333), (35, L9)];
const V_EXPLOSION_10A5: Variants = &[(0, L334), (1, L335), (2, L336), (3, L335), (32, L337), (33, L335), (34, L338), (35, L335)];
const V_EXPLOSION_118F: Variants = &[(0, L339), (1, L9), (2, L340), (3, L9), (4, L341), (5, L9), (6, L342), (7, L9), (32, L343), (33, L9), (34, L344), (35, L9), (36, L343), (37, L9), (38, L344), (39, L9)];
const V_EXPLOSION_1190: Variants = &[(0, L345), (1, L9), (2, L346), (3, L9), (4, L347), (5, L9), (6, L348), (7, L9), (32, L349), (33, L9), (34, L350), (35, L9), (36, L349), (37, L9), (38, L350), (39, L9)];
const V_EXPLOSION_1292: Variants = &[(0, L351), (1, L9), (2, L352), (3, L9), (4, L353), (5, L9), (6, L354), (7, L9), (32, L355), (33, L9), (34, L356), (35, L9), (36, L355), (37, L9), (38, L356), (39, L9)];
const V_EXPLOSION_1293: Variants = &[(0, L351), (1, L9), (2, L352), (3, L9), (4, L353), (5, L9), (6, L354), (7, L9), (32, L355), (33, L9), (34, L356), (35, L9), (36, L355), (37, L9), (38, L356), (39, L9)];
const V_EXPLOSION_1294: Variants = &[(0, L351), (1, L9), (2, L352), (3, L9), (4, L353), (5, L9), (6, L354), (7, L9), (32, L355), (33, L9), (34, L356), (35, L9), (36, L355), (37, L9), (38, L356), (39, L9)];
const V_EXPLOSION_1295: Variants = &[(0, L351), (1, L9), (2, L352), (3, L9), (4, L353), (5, L9), (6, L354), (7, L9), (32, L355), (33, L9), (34, L356), (35, L9), (36, L355), (37, L9), (38, L356), (39, L9)];
const V_EXPLOSION_1298: Variants = &[(0, L357), (1, L9), (2, L358), (3, L9), (4, L359), (5, L9), (6, L360), (7, L9), (32, L361), (33, L9), (34, L362), (35, L9), (36, L361), (37, L9), (38, L362), (39, L9)];
const V_EXPLOSION_1299: Variants = &[(0, L357), (1, L9), (2, L358), (3, L9), (4, L359), (5, L9), (6, L360), (7, L9), (32, L361), (33, L9), (34, L362), (35, L9), (36, L361), (37, L9), (38, L362), (39, L9)];

static TABLE: &[(Phase, i32, u8, Variants)] = &[
    (Phase::Explosion, 0x6, 35, V_EXPLOSION_6),
    (Phase::Explosion, 0x7, 11, V_EXPLOSION_7),
    (Phase::Explosion, 0x13, 39, V_EXPLOSION_13),
    (Phase::Explosion, 0x18, 11, V_EXPLOSION_18),
    (Phase::Explosion, 0x23, 11, V_EXPLOSION_23),
    (Phase::Explosion, 0x36, 35, V_EXPLOSION_36),
    (Phase::Explosion, 0x38, 39, V_EXPLOSION_38),
    (Phase::Explosion, 0x65, 39, V_EXPLOSION_65),
    (Phase::Explosion, 0x118, 35, V_EXPLOSION_118),
    (Phase::Explosion, 0x13a, 39, V_EXPLOSION_13A),
    (Phase::Explosion, 0x157, 39, V_EXPLOSION_157),
    (Phase::Explosion, 0x162, 39, V_EXPLOSION_162),
    (Phase::Explosion, 0x171, 39, V_EXPLOSION_171),
    (Phase::Explosion, 0x44c, 35, V_EXPLOSION_44C),
    (Phase::Explosion, 0x453, 35, V_EXPLOSION_453),
    (Phase::Explosion, 0x47c, 35, V_EXPLOSION_47C),
    (Phase::Explosion, 0x487, 35, V_EXPLOSION_487),
    (Phase::Explosion, 0x497, 35, V_EXPLOSION_497),
    (Phase::Explosion, 0x498, 43, V_EXPLOSION_498),
    (Phase::Explosion, 0x499, 35, V_EXPLOSION_499),
    (Phase::Explosion, 0x49a, 35, V_EXPLOSION_49A),
    (Phase::Explosion, 0x49d, 11, V_EXPLOSION_49D),
    (Phase::Explosion, 0x4c4, 35, V_EXPLOSION_4C4),
    (Phase::Explosion, 0x4ce, 35, V_EXPLOSION_4CE),
    (Phase::Explosion, 0x4d5, 35, V_EXPLOSION_4D5),
    (Phase::Explosion, 0x4d7, 35, V_EXPLOSION_4D7),
    (Phase::Explosion, 0x4dc, 35, V_EXPLOSION_4DC),
    (Phase::Explosion, 0x4f2, 35, V_EXPLOSION_4F2),
    (Phase::Explosion, 0x4f3, 35, V_EXPLOSION_4F3),
    (Phase::Explosion, 0x509, 35, V_EXPLOSION_509),
    (Phase::Explosion, 0x50a, 35, V_EXPLOSION_50A),
    (Phase::Explosion, 0x50b, 35, V_EXPLOSION_50B),
    (Phase::Explosion, 0xfa1, 35, V_EXPLOSION_FA1),
    (Phase::Explosion, 0xfb2, 35, V_EXPLOSION_FB2),
    (Phase::Explosion, 0xfba, 11, V_EXPLOSION_FBA),
    (Phase::Explosion, 0xfc8, 39, V_EXPLOSION_FC8),
    (Phase::Explosion, 0xfca, 35, V_EXPLOSION_FCA),
    (Phase::Explosion, 0xfcb, 11, V_EXPLOSION_FCB),
    (Phase::Explosion, 0xfd1, 35, V_EXPLOSION_FD1),
    (Phase::Explosion, 0xfe2, 35, V_EXPLOSION_FE2),
    (Phase::Explosion, 0xfe4, 35, V_EXPLOSION_FE4),
    (Phase::Explosion, 0xfe5, 35, V_EXPLOSION_FE5),
    (Phase::Explosion, 0xfee, 11, V_EXPLOSION_FEE),
    (Phase::Explosion, 0xff7, 35, V_EXPLOSION_FF7),
    (Phase::Explosion, 0x1004, 35, V_EXPLOSION_1004),
    (Phase::Explosion, 0x1009, 35, V_EXPLOSION_1009),
    (Phase::Explosion, 0x100f, 3, V_EXPLOSION_100F),
    (Phase::Explosion, 0x1012, 11, V_EXPLOSION_1012),
    (Phase::Explosion, 0x1018, 39, V_EXPLOSION_1018),
    (Phase::Explosion, 0x101c, 39, V_EXPLOSION_101C),
    (Phase::Explosion, 0x102d, 35, V_EXPLOSION_102D),
    (Phase::Explosion, 0x102e, 35, V_EXPLOSION_102E),
    (Phase::Explosion, 0x1037, 35, V_EXPLOSION_1037),
    (Phase::Explosion, 0x103b, 35, V_EXPLOSION_103B),
    (Phase::Explosion, 0x103c, 35, V_EXPLOSION_103C),
    (Phase::Explosion, 0x103d, 35, V_EXPLOSION_103D),
    (Phase::Explosion, 0x103e, 35, V_EXPLOSION_103E),
    (Phase::Explosion, 0x103f, 35, V_EXPLOSION_103F),
    (Phase::Explosion, 0x1045, 35, V_EXPLOSION_1045),
    (Phase::Explosion, 0x1052, 11, V_EXPLOSION_1052),
    (Phase::Explosion, 0x105f, 35, V_EXPLOSION_105F),
    (Phase::Explosion, 0x1063, 35, V_EXPLOSION_1063),
    (Phase::Explosion, 0x106c, 35, V_EXPLOSION_106C),
    (Phase::Explosion, 0x106f, 39, V_EXPLOSION_106F),
    (Phase::Explosion, 0x1070, 39, V_EXPLOSION_1070),
    (Phase::Explosion, 0x1084, 39, V_EXPLOSION_1084),
    (Phase::Explosion, 0x1085, 35, V_EXPLOSION_1085),
    (Phase::Explosion, 0x1086, 35, V_EXPLOSION_1086),
    (Phase::Explosion, 0x1087, 3, V_EXPLOSION_1087),
    (Phase::Explosion, 0x108c, 35, V_EXPLOSION_108C),
    (Phase::Explosion, 0x1097, 35, V_EXPLOSION_1097),
    (Phase::Explosion, 0x109a, 35, V_EXPLOSION_109A),
    (Phase::Explosion, 0x109b, 35, V_EXPLOSION_109B),
    (Phase::Explosion, 0x109c, 11, V_EXPLOSION_109C),
    (Phase::Explosion, 0x109d, 35, V_EXPLOSION_109D),
    (Phase::Explosion, 0x109e, 35, V_EXPLOSION_109E),
    (Phase::Explosion, 0x109f, 35, V_EXPLOSION_109F),
    (Phase::Explosion, 0x10a5, 35, V_EXPLOSION_10A5),
    (Phase::Explosion, 0x118f, 39, V_EXPLOSION_118F),
    (Phase::Explosion, 0x1190, 39, V_EXPLOSION_1190),
    (Phase::Explosion, 0x1292, 39, V_EXPLOSION_1292),
    (Phase::Explosion, 0x1293, 39, V_EXPLOSION_1293),
    (Phase::Explosion, 0x1294, 39, V_EXPLOSION_1294),
    (Phase::Explosion, 0x1295, 39, V_EXPLOSION_1295),
    (Phase::Explosion, 0x1298, 39, V_EXPLOSION_1298),
    (Phase::Explosion, 0x1299, 39, V_EXPLOSION_1299),
];
