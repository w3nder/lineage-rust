// Gerado por scripts/skill_effects_rust_gen.py a partir de scripts/skill_effects_oracle_gen.py (execução do
// engine.dll C4 no emulador) e scripts/asm_lift.py (expressões lidas do asm). Não editar à mão.

static NODES: &[Node] = &[
    Node::Field { target: false, offset: 0x1bc },
    Node::Field { target: false, offset: 0x1c0 },
    Node::Field { target: false, offset: 0x1c4 },
    Node::Int(0),
    Node::Int(1065353216),
    Node::Field { target: false, offset: 0x2f0 },
    Node::Float(0.6666667),
    Node::Op(Op::Mul, 5, 6),
    Node::Int(-1082130432),
    Node::Field { target: false, offset: 0x320 },
    Node::Field { target: false, offset: 0x324 },
    Node::Field { target: false, offset: 0x328 },
    Node::Tvb { axis: 0, vector: [8, 3, 3], rotator: [9, 10, 11] },
    Node::Op(Op::Mul, 7, 12),
    Node::ToF32(13),
    Node::ToF32(7),
    Node::Tvb { axis: 1, vector: [8, 3, 3], rotator: [9, 10, 11] },
    Node::Op(Op::Mul, 15, 16),
    Node::ToF32(17),
    Node::Field { target: false, offset: 0x1c8 },
    Node::Field { target: false, offset: 0x1cc },
    Node::Field { target: false, offset: 0x1d0 },
    Node::Float(1.0),
    Node::Field { target: true, offset: 0x1bc },
    Node::Op(Op::Sub, 23, 0),
    Node::ToF32(24),
    Node::Field { target: true, offset: 0x1c0 },
    Node::Op(Op::Sub, 26, 1),
    Node::ToF32(27),
    Node::Field { target: true, offset: 0x1c4 },
    Node::Op(Op::Sub, 29, 2),
    Node::ToF32(30),
    Node::Rotation { axis: 0, vector: [25, 28, 31] },
    Node::Rotation { axis: 1, vector: [25, 28, 31] },
    Node::Rotation { axis: 2, vector: [25, 28, 31] },
    Node::Tvb { axis: 0, vector: [4, 3, 3], rotator: [32, 33, 34] },
    Node::Op(Op::Mul, 5, 35),
    Node::ToF32(36),
    Node::Op(Op::Add, 37, 37),
    Node::ToF32(38),
    Node::Tvb { axis: 1, vector: [4, 3, 3], rotator: [32, 33, 34] },
    Node::Op(Op::Mul, 40, 5),
    Node::ToF32(41),
    Node::Op(Op::Add, 42, 42),
    Node::ToF32(43),
    Node::Tvb { axis: 2, vector: [4, 3, 3], rotator: [32, 33, 34] },
    Node::Op(Op::Mul, 45, 5),
    Node::ToF32(46),
    Node::Op(Op::Add, 47, 47),
    Node::ToF32(48),
    Node::Tvb { axis: 0, vector: [4, 3, 3], rotator: [19, 20, 21] },
    Node::Op(Op::Mul, 5, 50),
    Node::ToF32(51),
    Node::Op(Op::Add, 52, 52),
    Node::ToF32(53),
    Node::Tvb { axis: 1, vector: [4, 3, 3], rotator: [19, 20, 21] },
    Node::Op(Op::Mul, 55, 5),
    Node::ToF32(56),
    Node::Op(Op::Add, 57, 57),
    Node::ToF32(58),
    Node::Tvb { axis: 2, vector: [4, 3, 3], rotator: [19, 20, 21] },
    Node::Op(Op::Mul, 60, 5),
    Node::ToF32(61),
    Node::Op(Op::Add, 62, 62),
    Node::ToF32(63),
    Node::Field { target: false, offset: 0x2f4 },
    Node::Op(Op::Sub, 2, 65),
    Node::ToF32(66),
    Node::Op(Op::Add, 37, 0),
    Node::ToF32(68),
    Node::Op(Op::Add, 42, 1),
    Node::ToF32(70),
    Node::Op(Op::Add, 47, 2),
    Node::ToF32(72),
    Node::Op(Op::Add, 52, 0),
    Node::ToF32(74),
    Node::Op(Op::Add, 57, 1),
    Node::ToF32(76),
    Node::Op(Op::Add, 62, 2),
    Node::ToF32(78),
    Node::Tvb { axis: 0, vector: [4, 3, 3], rotator: [9, 10, 11] },
    Node::Op(Op::Mul, 5, 80),
    Node::ToF32(81),
    Node::Op(Op::Add, 82, 0),
    Node::ToF32(83),
    Node::Tvb { axis: 1, vector: [4, 3, 3], rotator: [9, 10, 11] },
    Node::Op(Op::Mul, 5, 85),
    Node::ToF32(86),
    Node::Op(Op::Add, 87, 1),
    Node::ToF32(88),
    Node::Float(0.33333334),
    Node::Op(Op::Mul, 65, 90),
    Node::Tvb { axis: 2, vector: [4, 3, 3], rotator: [9, 10, 11] },
    Node::Op(Op::Mul, 92, 5),
    Node::Op(Op::Add, 93, 2),
    Node::Op(Op::Add, 91, 94),
    Node::ToF32(95),
    Node::Op(Op::Add, 2, 65),
    Node::ToF32(97),
    Node::Field { target: true, offset: 0x2f0 },
    Node::Op(Op::Mul, 99, 35),
    Node::ToF32(100),
    Node::Int(1073741824),
    Node::Op(Op::Div, 101, 102),
    Node::Op(Op::Sub, 23, 103),
    Node::ToF32(104),
    Node::Op(Op::Mul, 99, 40),
    Node::ToF32(106),
    Node::Op(Op::Div, 107, 102),
    Node::Op(Op::Sub, 26, 108),
    Node::ToF32(109),
    Node::Op(Op::Mul, 99, 45),
    Node::ToF32(111),
    Node::Op(Op::Div, 112, 102),
    Node::Op(Op::Sub, 29, 113),
    Node::ToF32(114),
    Node::Op(Op::Mul, 35, 99),
    Node::ToF32(116),
    Node::Op(Op::Div, 117, 102),
    Node::Op(Op::Sub, 23, 118),
    Node::ToF32(119),
    Node::Op(Op::Mul, 40, 99),
    Node::ToF32(121),
    Node::Op(Op::Div, 122, 102),
    Node::Op(Op::Sub, 26, 123),
    Node::ToF32(124),
    Node::Op(Op::Add, 91, 2),
    Node::ToF32(126),
    Node::Neg(65),
    Node::ToF32(128),
    Node::Field { target: false, offset: 0x600 },
    Node::ToF32(26),
    Node::Field { target: true, offset: 0x2f4 },
    Node::Op(Op::Sub, 29, 132),
    Node::ToF32(133),
    Node::Op(Op::Mul, 35, 5),
    Node::ToF32(135),
    Node::Op(Op::Add, 136, 136),
    Node::ToF32(137),
    Node::Op(Op::Add, 138, 0),
    Node::ToF32(139),
    Node::Op(Op::Add, 44, 1),
    Node::ToF32(141),
    Node::Op(Op::Add, 46, 46),
    Node::Op(Op::Add, 143, 2),
    Node::ToF32(144),
    Node::Op(Op::Mul, 50, 5),
    Node::ToF32(146),
    Node::Op(Op::Add, 147, 147),
    Node::ToF32(148),
    Node::Op(Op::Add, 149, 0),
    Node::ToF32(150),
    Node::Op(Op::Add, 59, 1),
    Node::ToF32(152),
    Node::Op(Op::Add, 61, 61),
    Node::Op(Op::Add, 154, 2),
    Node::ToF32(155),
    Node::Float(5.0),
    Node::Op(Op::Mul, 37, 157),
    Node::ToF32(158),
    Node::Op(Op::Add, 159, 0),
    Node::ToF32(160),
    Node::Op(Op::Mul, 5, 40),
    Node::ToF32(162),
    Node::Op(Op::Mul, 163, 157),
    Node::ToF32(164),
    Node::Op(Op::Add, 165, 1),
    Node::ToF32(166),
    Node::Op(Op::Mul, 46, 157),
    Node::Op(Op::Add, 168, 2),
    Node::ToF32(169),
    Node::Tvb { axis: 0, vector: [5, 3, 3], rotator: [19, 20, 21] },
    Node::Op(Op::Add, 0, 171),
    Node::ToF32(172),
    Node::Tvb { axis: 1, vector: [5, 3, 3], rotator: [19, 20, 21] },
    Node::Op(Op::Add, 174, 1),
    Node::ToF32(175),
    Node::Tvb { axis: 2, vector: [5, 3, 3], rotator: [19, 20, 21] },
    Node::Op(Op::Add, 177, 2),
    Node::ToF32(178),
    Node::Op(Op::Mul, 45, 99),
    Node::ToF32(180),
    Node::Op(Op::Div, 181, 102),
    Node::Op(Op::Sub, 29, 182),
    Node::ToF32(183),
    Node::Op(Op::Add, 171, 0),
    Node::ToF32(185),
    Node::Tvb { axis: 0, vector: [15, 3, 3], rotator: [9, 10, 11] },
    Node::Op(Op::Add, 0, 187),
    Node::ToF32(188),
    Node::Tvb { axis: 1, vector: [15, 3, 3], rotator: [9, 10, 11] },
    Node::Op(Op::Add, 190, 1),
    Node::ToF32(191),
    Node::Tvb { axis: 2, vector: [15, 3, 3], rotator: [9, 10, 11] },
    Node::Op(Op::Add, 193, 2),
    Node::ToF32(194),
    Node::Int(1069547520),
    Node::Op(Op::Div, 82, 196),
    Node::Op(Op::Add, 197, 0),
    Node::ToF32(198),
    Node::Op(Op::Div, 87, 196),
    Node::Op(Op::Add, 200, 1),
    Node::ToF32(201),
    Node::Op(Op::Mul, 5, 92),
    Node::ToF32(203),
    Node::Op(Op::Div, 204, 196),
    Node::Op(Op::Add, 205, 2),
    Node::ToF32(206),
    Node::Op(Op::Div, 136, 102),
    Node::Op(Op::Add, 0, 208),
    Node::ToF32(209),
    Node::Op(Op::Div, 42, 102),
    Node::Op(Op::Add, 211, 1),
    Node::ToF32(212),
    Node::Op(Op::Mul, 5, 45),
    Node::ToF32(214),
    Node::Op(Op::Div, 215, 102),
    Node::Op(Op::Add, 216, 2),
    Node::ToF32(217),
    Node::Float(300.0),
    Node::Op(Op::Mul, 50, 219),
    Node::ToF32(220),
    Node::Op(Op::Add, 221, 0),
    Node::ToF32(222),
    Node::Op(Op::Mul, 55, 219),
    Node::ToF32(224),
    Node::Op(Op::Add, 225, 1),
    Node::ToF32(226),
    Node::Float(600.0),
    Node::Op(Op::Mul, 50, 228),
    Node::ToF32(229),
    Node::Op(Op::Sub, 0, 230),
    Node::ToF32(231),
    Node::Op(Op::Mul, 55, 228),
    Node::ToF32(233),
    Node::Op(Op::Sub, 1, 234),
    Node::ToF32(235),
    Node::Float(425.0),
    Node::Op(Op::Mul, 50, 237),
    Node::ToF32(238),
    Node::Op(Op::Add, 239, 0),
    Node::ToF32(240),
    Node::Op(Op::Mul, 55, 237),
    Node::ToF32(242),
    Node::Op(Op::Add, 243, 1),
    Node::ToF32(244),
    Node::Float(950.0),
    Node::Op(Op::Mul, 35, 246),
    Node::ToF32(247),
    Node::Op(Op::Add, 248, 0),
    Node::ToF32(249),
    Node::Op(Op::Mul, 40, 246),
    Node::ToF32(251),
    Node::Op(Op::Add, 252, 1),
    Node::ToF32(253),
    Node::Float(0.5),
    Node::Op(Op::Mul, 65, 255),
    Node::Op(Op::Add, 256, 2),
    Node::ToF32(257),
    Node::Rotation { axis: 0, vector: [23, 26, 3] },
    Node::Rotation { axis: 1, vector: [23, 26, 3] },
    Node::Rotation { axis: 2, vector: [23, 26, 3] },
    Node::Float(200.0),
    Node::Op(Op::Add, 23, 262),
    Node::ToF32(263),
    Node::Float(400.0),
    Node::Op(Op::Sub, 26, 265),
    Node::ToF32(266),
    Node::Op(Op::Add, 267, 262),
    Node::Op(Op::Add, 268, 228),
    Node::ToF32(269),
    Node::Float(800.0),
    Node::Op(Op::Sub, 264, 271),
    Node::ToF32(272),
    Node::Op(Op::Sub, 270, 265),
    Node::ToF32(274),
    Node::Op(Op::Add, 273, 262),
    Node::ToF32(276),
    Node::Op(Op::Add, 277, 228),
    Node::ToF32(278),
    Node::Op(Op::Sub, 279, 262),
    Node::ToF32(280),
    Node::Op(Op::Add, 275, 265),
    Node::ToF32(282),
    Node::Op(Op::Sub, 283, 265),
    Node::ToF32(284),
    Node::Float(350.0),
    Node::Op(Op::Sub, 281, 286),
    Node::ToF32(287),
    Node::Op(Op::Add, 285, 228),
    Node::ToF32(289),
    Node::Float(250.0),
    Node::Op(Op::Sub, 288, 291),
    Node::ToF32(292),
    Node::Op(Op::Sub, 290, 262),
    Node::ToF32(294),
    Node::Op(Op::Add, 293, 262),
    Node::ToF32(296),
    Node::Op(Op::Sub, 295, 228),
    Node::ToF32(298),
    Node::Op(Op::Add, 297, 265),
    Node::ToF32(300),
    Node::Op(Op::Add, 299, 271),
    Node::ToF32(302),
    Node::Op(Op::Sub, 301, 265),
    Node::ToF32(304),
    Node::Op(Op::Sub, 303, 262),
    Node::ToF32(306),
    Node::Op(Op::Add, 187, 0),
    Node::ToF32(308),
    Node::Tvb { axis: 0, vector: [5, 3, 3], rotator: [32, 33, 34] },
    Node::Op(Op::Add, 0, 310),
    Node::ToF32(311),
    Node::Tvb { axis: 1, vector: [5, 3, 3], rotator: [32, 33, 34] },
    Node::Op(Op::Add, 1, 313),
    Node::ToF32(314),
    Node::Tvb { axis: 2, vector: [5, 3, 3], rotator: [32, 33, 34] },
    Node::Op(Op::Add, 2, 316),
    Node::ToF32(317),
    Node::Field { target: true, offset: 0x1c8 },
    Node::Field { target: true, offset: 0x1cc },
    Node::Field { target: true, offset: 0x1d0 },
];

pub static SITES: &[Site] = &[
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 67], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [69, 71, 73], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [75, 77, 79], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [69, 71, 73], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [75, 77, 79], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [84, 89, 96], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [69, 71, 73], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [75, 77, 79], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 98], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 67], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [69, 71, 73], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [75, 77, 79], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 67], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 67], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [84, 89, 96], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 98], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [69, 71, 73], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [75, 77, 79], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 98], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [3, 3, 3], rotator: [9, 10, 11], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 98], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [84, 89, 96], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [69, 71, 73], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [75, 77, 79], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 98], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 67], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [69, 71, 73], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [75, 77, 79], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [69, 71, 73], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [75, 77, 79], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [84, 89, 96], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [69, 71, 73], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [75, 77, 79], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 98], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [39, 44, 49], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [54, 59, 64], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [14, 18, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 1] },
    Site::Unsupported("leitura de memória ('add', ('ld', ('add', ('sym', 'this'), 1348), 4), ('mul', ('mul', 0, 2"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 1] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [120, 125, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Unsupported("nó frand"),
    Site::Unsupported("nó frand"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 1, vector: [3, 3, 129], rotator: [3, 3, 3], scale: 130, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 130, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [32, 33, 34], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [0, 1, 98], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 130, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 1, vector: [3, 3, 129], rotator: [3, 3, 3], scale: 130, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [32, 33, 34], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [105, 110, 115], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [23, 131, 134], rotator: [3, 3, 3], scale: 4, flags: [0, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [23, 26, 29], rotator: [32, 33, 34], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [120, 125, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('mul', 0, 4), ('ld', ('add', ('sym', 'this'), 1312), 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [0, 1, 98], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [23, 26, 29], rotator: [32, 33, 34], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("nó undef"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [120, 125, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273543650)"),
    Site::Unsupported("ponteiro ('ret', 273543650)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273513810)"),
    Site::Unsupported("ponteiro ('ret', 273513810)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273543650)"),
    Site::Unsupported("ponteiro ('ret', 273543650)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273513810)"),
    Site::Unsupported("ponteiro ('ret', 273513810)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::Level, parent: Party::Target, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273544402)"),
    Site::Unsupported("ponteiro ('ret', 273544402)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [161, 167, 170], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Target, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 130, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 130, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [0, 1, 98], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [23, 131, 134], rotator: [3, 3, 3], scale: 4, flags: [0, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 1, vector: [3, 3, 129], rotator: [3, 3, 3], scale: 130, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [173, 176, 179], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [173, 176, 179], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [32, 33, 34], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273544402)"),
    Site::Unsupported("ponteiro ('ret', 273544402)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273543650)"),
    Site::Unsupported("ponteiro ('ret', 273543650)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [120, 125, 184], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [186, 176, 179], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [161, 167, 170], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Target, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273544402)"),
    Site::Unsupported("ponteiro ('ret', 273544402)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273543650)"),
    Site::Unsupported("ponteiro ('ret', 273543650)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273513810)"),
    Site::Unsupported("ponteiro ('ret', 273513810)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Unsupported("leitura de memória ('add', ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('mul', 0, 4), ('ld', ('add', ('sym', 'this'), 1312), 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [0, 1, 98], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [23, 131, 134], rotator: [3, 3, 3], scale: 4, flags: [0, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [120, 125, 184], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [120, 125, 184], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [120, 125, 184], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [173, 176, 179], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [173, 176, 179], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273550042)"),
    Site::Unsupported("ponteiro ('ret', 273550042)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273543650)"),
    Site::Unsupported("ponteiro ('ret', 273543650)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273544402)"),
    Site::Unsupported("ponteiro ('ret', 273544402)"),
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::Level, parent: Party::Target, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [23, 26, 29], rotator: [32, 33, 34], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Unsupported("leitura de memória ('add', ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul"),
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Target, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [23, 131, 134], rotator: [3, 3, 3], scale: 4, flags: [0, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [0, 1, 98], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [23, 131, 134], rotator: [3, 3, 3], scale: 4, flags: [0, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::Level, parent: Party::Target, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [0, 1, 98], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [189, 192, 195], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [199, 202, 207], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [199, 202, 207], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [210, 213, 218], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Target, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [0, 1, 98], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273543650)"),
    Site::Unsupported("ponteiro ('ret', 273543650)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [199, 202, 207], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [199, 202, 207], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Unsupported("leitura de memória ('add', ('ret', 273521999), 152)"),
    Site::Unsupported("leitura de memória ('add', ('ret', 273522959), 152)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [23, 26, 134], rotator: [3, 3, 3], scale: 3, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [241, 245, 134], rotator: [3, 3, 3], scale: 3, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [250, 254, 134], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [23, 131, 134], rotator: [3, 3, 3], scale: 4, flags: [0, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [0, 1, 258], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [32, 33, 34], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [23, 26, 134], rotator: [259, 260, 261], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [23, 26, 134], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [264, 267, 134], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [264, 270, 134], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [273, 275, 134], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [277, 275, 134], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [279, 275, 134], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [281, 283, 134], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [281, 285, 134], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [288, 290, 134], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [293, 295, 134], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [297, 299, 134], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [301, 303, 134], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [305, 307, 134], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [23, 26, 134], rotator: [259, 260, 261], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 130, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 130, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [32, 33, 34], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 130, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Target, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::Level, parent: Party::Target, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [309, 192, 195], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [309, 192, 195], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [0, 1, 98], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [199, 202, 207], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [199, 202, 207], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [0, 1, 98], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [23, 131, 134], rotator: [3, 3, 3], scale: 4, flags: [0, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Target, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [186, 176, 179], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273543650)"),
    Site::Unsupported("ponteiro ('ret', 273543650)"),
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('shl', 0, 2)"),
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [312, 315, 318], rotator: [32, 33, 34], scale: 4, flags: [0, 0, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 0, 0, 0] },
    Site::Unsupported("leitura de memória ('add', ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [173, 176, 179], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 127], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273544402)"),
    Site::Unsupported("ponteiro ('ret', 273544402)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [309, 192, 195], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [309, 192, 195], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [199, 202, 207], rotator: [19, 20, 21], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [199, 202, 207], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [32, 33, 34], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [23, 26, 134], rotator: [319, 320, 321], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [23, 26, 134], rotator: [319, 320, 321], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Caster, instigator: Party::Caster, mode: 0, vector: [0, 1, 258], rotator: [19, 20, 21], scale: 4, flags: [0, 1, 1, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Target, instigator: Party::Null, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Unsupported("leitura de memória ('add', ('ld', ('add', ('mul', 0, 4), ('ld', ('add', ('sym', 'this'), 1"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Target, mode: 0, vector: [105, 110, 115], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [23, 26, 29], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [140, 142, 145], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [151, 153, 156], rotator: [3, 3, 3], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 1, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ret', 273550042)"),
    Site::Unsupported("ponteiro ('ret', 273550042)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Target, instigator: Party::Target, mode: 2, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [1, 1, 0, 0] },
    Site::Unsupported("ponteiro ('ld', ('add', ('ld', ('add', ('sym', 'this'), 1312), 4), ('mul', 0, 4)"),
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [3, 3, 3], rotator: [3, 3, 3], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [0, 1, 67], rotator: [3, 3, 3], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Caster, mode: 0, vector: [0, 1, 2], rotator: [3, 3, 3], scale: 4, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 2], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 2], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 2], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 2], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 2], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 2], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [105, 125, 184], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 2], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::SkillEffect, parent: Party::Null, instigator: Party::Null, mode: 0, vector: [105, 125, 184], rotator: [32, 33, 34], scale: 4, flags: [0, 1, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 2], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 2], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 2], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
    Site::Known { kind: SiteKind::Level, parent: Party::Caster, instigator: Party::Null, mode: 0, vector: [0, 1, 2], rotator: [19, 20, 21], scale: 22, flags: [0, 0, 0, 0] },
];

const L0: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_b", site: 0 },
    Step::Attach(Bone::RightHand),
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 1 },
];
const L1: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_a", site: 2 },
];
const L2: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u002_a", site: 3 },
    Step::Attach(Bone::RightHand),
    Step::Trailer { points: 0, time: Scalar::Const(0.8), flag: 1 },
];
const L3: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u002_a", site: 3 },
    Step::Attach(Bone::RightHand),
];
const L4: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u016_a", site: 4 },
];
const L5: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 5 },
];
const L6: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u011_a", site: 6 },
];
const L7: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u001_a", site: 7 },
    Step::Attach(Bone::RightHand),
    Step::Trailer { points: 0, time: Scalar::Const(0.8), flag: 1 },
];
const L8: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u001_a", site: 7 },
    Step::Attach(Bone::RightHand),
];
const L9: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 8 },
    Step::Spawn { class: "LineageEffect.s_u003_a", site: 9 },
    Step::Attach(Bone::LeftHand),
];
const L10: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u022_a", site: 10 },
];
const L11: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u003_a", site: 11 },
    Step::Attach(Bone::LeftHand),
];
const L12: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_a", site: 12 },
];
const L13: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u006_a", site: 13 },
    Step::Attach(Bone::RightHand),
];
const L14: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_b", site: 14 },
    Step::Attach(Bone::RightHand),
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 15 },
    Step::Trailer { points: 0, time: Scalar::Const(0.8), flag: 1 },
];
const L15: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_b", site: 14 },
    Step::Attach(Bone::RightHand),
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 15 },
];
const L16: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_a", site: 16 },
];
const L17: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u020_a", site: 17 },
];
const L18: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u017_a", site: 18 },
];
const L19: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_a", site: 19 },
];
const L20: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_a", site: 20 },
];
const L21: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u019_a", site: 21 },
];
const L22: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u017_a", site: 22 },
];
const L23: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u030_a", site: 23 },
];
const L24: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u013_a", site: 24 },
];
const L25: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u505_a", site: 25 },
    Step::Attach(Bone::LeftHand),
];
const L26: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_a", site: 26 },
];
const L27: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_a", site: 27 },
];
const L28: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_a", site: 28 },
];
const L29: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_a", site: 29 },
];
const L30: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_b", site: 30 },
    Step::Attach(Bone::RightHand),
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 31 },
    Step::Trailer { points: 0, time: Scalar::Const(0.8), flag: 1 },
];
const L31: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_b", site: 30 },
    Step::Attach(Bone::RightHand),
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 31 },
];
const L32: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u002_a", site: 32 },
    Step::Attach(Bone::RightHand),
    Step::Trailer { points: 0, time: Scalar::Const(0.8), flag: 1 },
];
const L33: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u002_a", site: 32 },
    Step::Attach(Bone::RightHand),
];
const L34: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u016_a", site: 33 },
];
const L35: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_a", site: 34 },
];
const L36: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u011_a", site: 35 },
];
const L37: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u007_a", site: 36 },
    Step::Attach(Bone::RightHand),
];
const L38: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_b", site: 37 },
    Step::Attach(Bone::RightHand),
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 38 },
];
const L39: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_a", site: 39 },
];
const L40: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 40 },
];
const L41: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_a", site: 41 },
];
const L42: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u017_a", site: 42 },
];
const L43: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u020_a", site: 43 },
];
const L44: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_a", site: 44 },
];
const L45: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u016_a", site: 45 },
];
const L46: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u011_a", site: 46 },
];
const L47: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u007_a", site: 47 },
    Step::Attach(Bone::RightHand),
];
const L48: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 48 },
];
const L49: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u509_a", site: 49 },
    Step::Attach(Bone::LeftHand),
];
const L50: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u513_a", site: 50 },
    Step::Attach(Bone::RightHand),
];
const L51: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 51 },
];
const L52: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_a", site: 52 },
];
const L53: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_a", site: 53 },
];
const L54: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_a", site: 54 },
];
const L55: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_a", site: 55 },
];
const L56: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_a", site: 56 },
];
const L57: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_a", site: 57 },
];
const L58: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_a", site: 58 },
];
const L59: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_a", site: 59 },
];
const L60: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_a", site: 60 },
];
const L61: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u022_a", site: 61 },
];
const L62: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_a", site: 62 },
];
const L63: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u023_a", site: 63 },
];
const L64: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u032_a", site: 64 },
];
const L65: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_a", site: 65 },
];
const L66: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u017_a", site: 66 },
];
const L67: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_a", site: 67 },
];
const L68: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u030_a", site: 68 },
];
const L69: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u014_a", site: 69 },
];
const L70: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u031_a", site: 70 },
];
const L71: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_a", site: 71 },
];
const L72: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_a", site: 72 },
];
const L73: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_a", site: 73 },
];
const L74: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_a", site: 74 },
];
const L75: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_a", site: 75 },
];
const L76: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_a", site: 76 },
];
const L77: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_a", site: 77 },
];
const L78: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_a", site: 78 },
];
const L79: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_a", site: 79 },
];
const L80: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u027_a", site: 80 },
];
const L81: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u011_a", site: 81 },
];
const L82: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_a", site: 82 },
];
const L83: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_a", site: 83 },
];
const L84: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_a", site: 84 },
];
const L85: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u011_a", site: 85 },
];
const L86: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_a", site: 86 },
];
const L87: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_a", site: 87 },
];
const L88: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_a", site: 88 },
];
const L89: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_a", site: 89 },
];
const L90: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_a", site: 90 },
];
const L91: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_a", site: 91 },
];
const L92: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u005_a", site: 92 },
];
const L93: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u039_a", site: 93 },
    Step::Spawn { class: "LineageEffect.m_u039_b", site: 94 },
];
const L94: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u039_a", site: 93 },
    Step::Spawn { class: "LineageEffect.m_u039_b", site: 95 },
];
const L95: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_a", site: 96 },
    Step::Spawn { class: "LineageEffect.m_u038_b", site: 97 },
];
const L96: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_a", site: 96 },
    Step::Spawn { class: "LineageEffect.m_u038_b", site: 98 },
];
const L97: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_a", site: 99 },
    Step::Spawn { class: "LineageEffect.m_u000_b", site: 100 },
];
const L98: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_a", site: 101 },
    Step::Spawn { class: "LineageEffect.m_u026_b", site: 102 },
];
const L99: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_a", site: 101 },
    Step::Spawn { class: "LineageEffect.m_u026_b", site: 103 },
];
const L100: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_a", site: 104 },
    Step::Spawn { class: "LineageEffect.m_u006_b", site: 105 },
    Step::Life(Scalar::CastTime),
];
const L101: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u033_a", site: 106 },
];
const L102: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u010_a", site: 107 },
];
const L103: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u013_a", site: 108 },
];
const L104: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u009_a", site: 109 },
];
const L105: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_a", site: 110 },
];
const L106: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_a", site: 111 },
];
const L107: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_a", site: 112 },
];
const L108: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u032_a", site: 113 },
];
const L109: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_a", site: 114 },
];
const L110: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_a", site: 115 },
];
const L111: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_a", site: 116 },
];
const L112: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u031_a", site: 117 },
];
const L113: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_a", site: 118 },
];
const L114: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u037_a", site: 119 },
    Step::Spawn { class: "LineageEffect.m_u037_b", site: 120 },
];
const L115: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u037_a", site: 119 },
    Step::Spawn { class: "LineageEffect.m_u037_b", site: 121 },
];
const L116: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_a", site: 122 },
];
const L117: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u044_a", site: 123 },
];
const L118: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_a", site: 124 },
];
const L119: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_a", site: 125 },
];
const L120: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_a", site: 126 },
];
const L121: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_a", site: 127 },
];
const L122: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u023_a", site: 128 },
];
const L123: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u031_a", site: 129 },
];
const L124: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_a", site: 130 },
];
const L125: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 131 },
];
const L126: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u039_a", site: 132 },
];
const L127: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_a", site: 133 },
];
const L128: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u031_a", site: 134 },
];
const L129: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u044_a", site: 135 },
];
const L130: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u800_a", site: 136 },
];
const L131: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u800_a", site: 137 },
];
const L132: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_a", site: 138 },
];
const L133: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_a", site: 139 },
];
const L134: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_a", site: 140 },
];
const L135: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_a", site: 141 },
];
const L136: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u031_a", site: 142 },
];
const L137: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u023_a", site: 143 },
];
const L138: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u031_a", site: 144 },
];
const L139: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u011_a", site: 145 },
];
const L140: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u023_a", site: 146 },
];
const L141: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_a", site: 147 },
];
const L142: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_a", site: 148 },
];
const L143: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_a", site: 149 },
];
const L144: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_a", site: 150 },
];
const L145: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_a", site: 151 },
];
const L146: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u010_a", site: 152 },
];
const L147: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_a", site: 153 },
];
const L148: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u023_a", site: 154 },
];
const L149: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u014_a", site: 155 },
];
const L150: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_a", site: 156 },
];
const L151: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u007_a", site: 157 },
    Step::Attach(Bone::RightHand),
];
const L152: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u031_a", site: 158 },
];
const L153: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_a", site: 159 },
    Step::Spawn { class: "LineageEffect.m_u000_b", site: 160 },
];
const L154: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_a", site: 161 },
];
const L155: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 162 },
];
const L156: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_a", site: 163 },
];
const L157: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u022_a", site: 164 },
];
const L158: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_a", site: 165 },
    Step::Spawn { class: "LineageEffect.m_u006_b", site: 166 },
    Step::Life(Scalar::CastTime),
];
const L159: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_a", site: 167 },
];
const L160: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_a", site: 168 },
];
const L161: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_a", site: 169 },
];
const L162: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_a", site: 170 },
];
const L163: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u002_a", site: 171 },
    Step::Attach(Bone::RightHand),
    Step::Trailer { points: 0, time: Scalar::Const(0.8), flag: 1 },
];
const L164: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u002_a", site: 171 },
    Step::Attach(Bone::RightHand),
];
const L165: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_a", site: 172 },
];
const L166: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u005_a", site: 173 },
];
const L167: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_a", site: 174 },
];
const L168: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_a", site: 175 },
];
const L169: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_a", site: 176 },
];
const L170: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u009_a", site: 177 },
];
const L171: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u003_a", site: 178 },
    Step::Attach(Bone::LeftHand),
];
const L172: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_a", site: 179 },
];
const L173: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_a", site: 180 },
];
const L174: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u013_a", site: 181 },
];
const L175: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 182 },
];
const L176: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_a", site: 183 },
];
const L177: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_a", site: 184 },
    Step::Spawn { class: "LineageEffect.m_u026_b", site: 185 },
];
const L178: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_a", site: 184 },
    Step::Spawn { class: "LineageEffect.m_u026_b", site: 186 },
];
const L179: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_a", site: 187 },
];
const L180: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_a", site: 188 },
];
const L181: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_a", site: 189 },
];
const L182: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_a", site: 190 },
    Step::Spawn { class: "LineageEffect.m_u006_b", site: 191 },
    Step::Life(Scalar::CastTime),
];
const L183: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_a", site: 192 },
];
const L184: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_a", site: 193 },
];
const L185: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_a", site: 194 },
];
const L186: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_a", site: 195 },
];
const L187: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_a", site: 196 },
];
const L188: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u027_a", site: 197 },
];
const L189: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_a", site: 198 },
];
const L190: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u003_a", site: 199 },
    Step::Attach(Bone::LeftHand),
];
const L191: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u065_a", site: 200 },
    Step::Delayed { time: Scalar::Const(0.0), a: 1, b: 0 },
    Step::SizeScale(Scalar::OverRadius(9.0)),
];
const L192: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u064_cloud", site: 201 },
];
const L193: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u033_a", site: 202 },
];
const L194: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u060_a", site: 203 },
    Step::Life(Scalar::CastTime),
    Step::Delayed { time: Scalar::Const(0.0), a: 1, b: 0 },
    Step::Attach(Bone::Named("Bone04")),
];
const L195: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u058_r", site: 204 },
    Step::Attach(Bone::RightHand),
];
const L196: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 205 },
];
const L197: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_a", site: 206 },
    Step::Spawn { class: "LineageEffect.m_u000_b", site: 207 },
];
const L198: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_a", site: 208 },
];
const L199: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u005_a", site: 209 },
];
const L200: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_a", site: 210 },
    Step::Spawn { class: "LineageEffect.m_u026_b", site: 211 },
];
const L201: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_a", site: 210 },
    Step::Spawn { class: "LineageEffect.m_u026_b", site: 212 },
];
const L202: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_a", site: 213 },
    Step::Spawn { class: "LineageEffect.m_u006_b", site: 214 },
    Step::Life(Scalar::CastTime),
];
const L203: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_a", site: 215 },
];
const L204: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u031_a", site: 216 },
];
const L205: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_a", site: 217 },
];
const L206: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_a", site: 218 },
];
const L207: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_a", site: 219 },
];
const L208: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u016_a", site: 220 },
];
const L209: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_a", site: 221 },
];
const L210: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_a", site: 222 },
];
const L211: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u009_a", site: 223 },
];
const L212: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_a", site: 224 },
];
const L213: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u013_a", site: 225 },
];
const L214: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u027_a", site: 226 },
];
const L215: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_a", site: 227 },
];
const L216: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u037_a", site: 228 },
    Step::Spawn { class: "LineageEffect.m_u037_b", site: 229 },
];
const L217: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u037_a", site: 228 },
    Step::Spawn { class: "LineageEffect.m_u037_b", site: 230 },
];
const L218: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u003_a", site: 231 },
    Step::Attach(Bone::LeftHand),
];
const L219: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u505_a", site: 232 },
    Step::Attach(Bone::LeftHand),
];
const L220: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_a", site: 233 },
];
const L221: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_a", site: 234 },
];
const L222: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_a", site: 235 },
];
const L223: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_a", site: 236 },
];
const L224: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_a", site: 237 },
];
const L225: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_a", site: 238 },
];
const L226: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u071_a", site: 239 },
];
const L227: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_a", site: 240 },
];
const L228: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u013_a", site: 241 },
];
const L229: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 242 },
];
const L230: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u031_a", site: 243 },
];
const L231: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u058_r", site: 244 },
    Step::Attach(Bone::RightHand),
    Step::Spawn { class: "LineageEffect.e_u058_l", site: 245 },
    Step::Attach(Bone::LeftHand),
];
const L232: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u057_a", site: 246 },
    Step::Attach(Bone::RightHand),
];
const L233: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_a", site: 247 },
    Step::Spawn { class: "LineageEffect.m_u038_b", site: 248 },
];
const L234: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_a", site: 247 },
    Step::Spawn { class: "LineageEffect.m_u038_b", site: 249 },
];
const L235: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u005_a", site: 250 },
];
const L236: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_a", site: 251 },
];
const L237: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_a", site: 252 },
];
const L238: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_a", site: 253 },
    Step::Spawn { class: "LineageEffect.m_u000_b", site: 254 },
];
const L239: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_a", site: 255 },
    Step::Spawn { class: "LineageEffect.m_u026_b", site: 256 },
];
const L240: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_a", site: 255 },
    Step::Spawn { class: "LineageEffect.m_u026_b", site: 257 },
];
const L241: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_a", site: 258 },
    Step::Spawn { class: "LineageEffect.m_u006_b", site: 259 },
    Step::Life(Scalar::CastTime),
];
const L242: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u033_a", site: 260 },
];
const L243: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_a", site: 261 },
];
const L244: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u082_rainbow", site: 262 },
    Step::Attach(Bone::Named("dummy04")),
    Step::Spawn { class: "LineageEffect.e_u082_core", site: 263 },
    Step::Attach(Bone::Named("dummy05")),
];
const L245: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u504_a", site: 264 },
    Step::SizeScale(Scalar::OverRadius(9.0)),
    Step::Spawn { class: "LineageEffect.e_u504_rh", site: 265 },
    Step::Attach(Bone::RightHand),
    Step::Spawn { class: "LineageEffect.e_u504_rh", site: 266 },
    Step::Attach(Bone::LeftHand),
];
const L246: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u010_a", site: 267 },
];
const L247: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_a", site: 268 },
];
const L248: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u019_a", site: 269 },
];
const L249: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u800_a", site: 270 },
];
const L250: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u800_a", site: 271 },
];
const L251: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_a", site: 272 },
];
const L252: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_a", site: 273 },
];
const L253: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_a", site: 274 },
];
const L254: &[Step] = &[
];
const L255: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u524_meteor", site: 275 },
    Step::Delayed { time: Scalar::CastOffset(-0.75), a: 1, b: 0 },
];
const L256: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u003_a", site: 276 },
    Step::Attach(Bone::LeftHand),
];
const L257: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u505_a", site: 277 },
    Step::Attach(Bone::LeftHand),
];
const L258: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 278 },
];
const L259: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 279 },
];
const L260: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 280 },
];
const L261: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 281 },
];
const L262: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u016_b", site: 282 },
];
const L263: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u016_b", site: 283 },
];
const L264: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 284 },
];
const L265: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 285 },
];
const L266: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u015_a", site: 286 },
];
const L267: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u015_a", site: 287 },
];
const L268: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 288 },
];
const L269: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 288 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 289 },
    Step::Attach(Bone::Index(0)),
];
const L270: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 290 },
    Step::Attach(Bone::Index(0)),
];
const L271: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u022_b", site: 291 },
];
const L272: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 292 },
];
const L273: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 293 },
];
const L274: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u006_b", site: 294 },
];
const L275: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 295 },
];
const L276: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 296 },
];
const L277: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_c", site: 297 },
];
const L278: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u020_b", site: 298 },
    Step::Spawn { class: "LineageEffect.s_u020_c", site: 299 },
    Step::Spawn { class: "LineageEffect.s_u020_c", site: 300 },
];
const L279: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u017_b", site: 301 },
];
const L280: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u017_b", site: 302 },
];
const L281: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_b", site: 303 },
];
const L282: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_b", site: 304 },
];
const L283: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_c", site: 305 },
];
const L284: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 306 },
];
const L285: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 307 },
];
const L286: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 308 },
];
const L287: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 309 },
];
const L288: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 310 },
];
const L289: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u019_b", site: 311 },
    Step::Spawn { class: "LineageEffect.s_u019_c", site: 312 },
];
const L290: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u017_b", site: 313 },
];
const L291: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u017_b", site: 314 },
];
const L292: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u016_b", site: 315 },
];
const L293: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u016_b", site: 316 },
];
const L294: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_b", site: 317 },
];
const L295: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_b", site: 318 },
];
const L296: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 319 },
    Step::Attach(Bone::Index(0)),
];
const L297: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 320 },
];
const L298: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 320 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 321 },
    Step::Attach(Bone::Index(0)),
];
const L299: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 322 },
];
const L300: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 323 },
];
const L301: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u006_b", site: 324 },
];
const L302: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 325 },
];
const L303: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u030_b", site: 326 },
];
const L304: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u013_c", site: 327 },
];
const L305: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u013_c", site: 328 },
];
const L306: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 329 },
];
const L307: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 330 },
];
const L308: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u505_b", site: 331 },
    Step::Attach(Bone::LeftHand),
];
const L309: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 332 },
];
const L310: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 333 },
];
const L311: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_b", site: 334 },
];
const L312: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_b", site: 335 },
];
const L313: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u005_b", site: 336 },
    Step::Attach(Bone::Index(0)),
];
const L314: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u005_b", site: 337 },
    Step::Attach(Bone::Index(0)),
];
const L315: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 338 },
    Step::Attach(Bone::Index(0)),
];
const L316: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 339 },
    Step::Attach(Bone::Index(0)),
];
const L317: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_b", site: 340 },
];
const L318: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_b", site: 341 },
];
const L319: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_b", site: 342 },
];
const L320: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_b", site: 343 },
];
const L321: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 344 },
];
const L322: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 345 },
];
const L323: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u016_b", site: 346 },
];
const L324: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u016_b", site: 347 },
];
const L325: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 348 },
];
const L326: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 349 },
];
const L327: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u007_b", site: 350 },
];
const L328: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u007_b", site: 351 },
];
const L329: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u007_b", site: 352 },
];
const L330: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 353 },
];
const L331: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u015_b", site: 354 },
];
const L332: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 355 },
];
const L333: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 356 },
];
const L334: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u009_a", site: 357 },
];
const L335: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u012_a", site: 358 },
];
const L336: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u011_a", site: 359 },
];
const L337: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u011_a", site: 360 },
];
const L338: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_c", site: 361 },
];
const L339: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 362 },
];
const L340: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 363 },
];
const L341: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 364 },
];
const L342: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 365 },
];
const L343: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u017_b", site: 366 },
];
const L344: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u017_b", site: 367 },
];
const L345: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u020_b", site: 368 },
    Step::Spawn { class: "LineageEffect.s_u020_c", site: 369 },
    Step::Spawn { class: "LineageEffect.s_u020_c", site: 370 },
];
const L346: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_b", site: 371 },
];
const L347: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_b", site: 372 },
];
const L348: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u016_b", site: 373 },
];
const L349: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u016_b", site: 374 },
];
const L350: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u007_b", site: 375 },
];
const L351: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u007_b", site: 376 },
];
const L352: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u007_b", site: 377 },
];
const L353: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u009_a", site: 378 },
];
const L354: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u012_a", site: 379 },
];
const L355: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u509_b", site: 380 },
    Step::Attach(Bone::LeftHand),
];
const L356: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u509_b", site: 381 },
    Step::Attach(Bone::LeftHand),
];
const L357: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u513_b", site: 382 },
    Step::Attach(Bone::RightHand),
    Step::Spawn { class: "LineageEffect.s_u513_c", site: 383 },
];
const L358: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u513_b", site: 382 },
    Step::Attach(Bone::RightHand),
];
const L359: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 384 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 385 },
];
const L360: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 386 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 387 },
];
const L361: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 384 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 388 },
];
const L362: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 386 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 389 },
];
const L363: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_b", site: 390 },
    Step::Spawn { class: "LineageEffect.m_u034_c", site: 391 },
];
const L364: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_b", site: 392 },
    Step::Spawn { class: "LineageEffect.m_u034_c", site: 393 },
];
const L365: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_b", site: 390 },
    Step::Spawn { class: "LineageEffect.m_u034_c", site: 394 },
];
const L366: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_b", site: 392 },
    Step::Spawn { class: "LineageEffect.m_u034_c", site: 395 },
];
const L367: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 396 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 397 },
];
const L368: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 398 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 399 },
];
const L369: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 396 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 400 },
];
const L370: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 398 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 401 },
];
const L371: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_b", site: 402 },
    Step::Spawn { class: "LineageEffect.m_u034_c", site: 403 },
];
const L372: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_b", site: 404 },
    Step::Spawn { class: "LineageEffect.m_u034_c", site: 405 },
];
const L373: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_b", site: 402 },
    Step::Spawn { class: "LineageEffect.m_u034_c", site: 406 },
];
const L374: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_b", site: 404 },
    Step::Spawn { class: "LineageEffect.m_u034_c", site: 407 },
];
const L375: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 408 },
];
const L376: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 409 },
];
const L377: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u022_b", site: 410 },
];
const L378: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 411 },
];
const L379: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u023_b", site: 412 },
];
const L380: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u023_b", site: 413 },
];
const L381: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u022_b", site: 414 },
];
const L382: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u032_b", site: 415 },
];
const L383: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u032_b", site: 416 },
];
const L384: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 417 },
];
const L385: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u017_b", site: 418 },
];
const L386: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u017_b", site: 419 },
];
const L387: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_b", site: 420 },
];
const L388: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_b", site: 421 },
];
const L389: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u030_b", site: 422 },
];
const L390: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u014_b", site: 423 },
];
const L391: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u014_b", site: 424 },
];
const L392: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u005_a", site: 425 },
];
const L393: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_b", site: 426 },
];
const L394: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_b", site: 427 },
];
const L395: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 428 },
];
const L396: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 429 },
];
const L397: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 430 },
];
const L398: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 431 },
];
const L399: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_b", site: 432 },
];
const L400: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_b", site: 433 },
];
const L401: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_c", site: 434 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 435 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 436 },
];
const L402: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 437 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 438 },
];
const L403: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 439 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 440 },
];
const L404: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 437 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 441 },
];
const L405: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 439 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 442 },
];
const L406: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_b", site: 443 },
];
const L407: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u027_b", site: 444 },
];
const L408: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u027_b", site: 445 },
];
const L409: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 446 },
];
const L410: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 447 },
];
const L411: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_c", site: 448 },
];
const L412: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_c", site: 449 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 450 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 451 },
];
const L413: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 452 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 453 },
];
const L414: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 452 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 453 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 454 },
    Step::Attach(Bone::Index(0)),
];
const L415: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 455 },
    Step::Attach(Bone::Index(0)),
];
const L416: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 456 },
    Step::Attach(Bone::Index(0)),
];
const L417: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 457 },
];
const L418: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 458 },
];
const L419: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_c", site: 459 },
];
const L420: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 460 },
    Step::Attach(Bone::Index(0)),
];
const L421: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 461 },
    Step::Attach(Bone::Index(0)),
];
const L422: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_b", site: 462 },
];
const L423: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_b", site: 463 },
];
const L424: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 464 },
];
const L425: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 464 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 465 },
    Step::Attach(Bone::Index(0)),
];
const L426: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_b", site: 466 },
];
const L427: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_b", site: 467 },
];
const L428: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u005_b", site: 468 },
    Step::Attach(Bone::Index(0)),
];
const L429: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u005_b", site: 469 },
    Step::Attach(Bone::Index(0)),
];
const L430: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u039_c", site: 470 },
    Step::Spawn { class: "LineageEffect.m_u039_d", site: 471 },
];
const L431: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_c", site: 472 },
    Step::Spawn { class: "LineageEffect.m_u038_c", site: 473 },
];
const L432: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_c", site: 474 },
];
const L433: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_c", site: 475 },
];
const L434: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u033_b", site: 476 },
];
const L435: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u033_b", site: 477 },
];
const L436: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u010_c", site: 478 },
];
const L437: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u010_c", site: 479 },
];
const L438: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u013_c", site: 480 },
];
const L439: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u013_c", site: 481 },
];
const L440: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u009_c", site: 482 },
];
const L441: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u009_c", site: 483 },
];
const L442: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 484 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 485 },
];
const L443: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 486 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 487 },
];
const L444: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 484 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 488 },
];
const L445: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 486 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 489 },
];
const L446: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 490 },
];
const L447: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 491 },
];
const L448: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u032_b", site: 492 },
];
const L449: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u032_b", site: 493 },
];
const L450: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 494 },
];
const L451: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 495 },
];
const L452: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 496 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 497 },
];
const L453: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 498 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 499 },
];
const L454: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 496 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 500 },
];
const L455: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 498 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 501 },
];
const L456: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u031_b", site: 502 },
];
const L457: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u031_b", site: 503 },
];
const L458: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_b", site: 504 },
];
const L459: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_b", site: 505 },
];
const L460: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u037_c", site: 506 },
    Step::Spawn { class: "LineageEffect.m_u037_d", site: 507 },
];
const L461: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_b", site: 508 },
];
const L462: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_b", site: 509 },
];
const L463: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 510 },
];
const L464: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 511 },
];
const L465: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_b", site: 512 },
];
const L466: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_b", site: 513 },
];
const L467: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u028_b", site: 514 },
];
const L468: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_c", site: 515 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 516 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 517 },
];
const L469: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 518 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 519 },
];
const L470: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 520 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 521 },
];
const L471: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 518 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 522 },
];
const L472: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 520 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 523 },
];
const L473: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 524 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 525 },
];
const L474: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 526 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 527 },
];
const L475: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 524 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 528 },
];
const L476: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 526 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 529 },
];
const L477: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_b", site: 530 },
    Step::Spawn { class: "LineageEffect.m_u034_c", site: 531 },
];
const L478: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_b", site: 532 },
    Step::Spawn { class: "LineageEffect.m_u034_c", site: 533 },
];
const L479: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_b", site: 530 },
    Step::Spawn { class: "LineageEffect.m_u034_c", site: 534 },
];
const L480: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u034_b", site: 532 },
    Step::Spawn { class: "LineageEffect.m_u034_c", site: 535 },
];
const L481: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u023_b", site: 536 },
];
const L482: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u023_b", site: 537 },
];
const L483: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u005_a", site: 538 },
];
const L484: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_b", site: 539 },
];
const L485: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_b", site: 540 },
];
const L486: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u032_b", site: 541 },
];
const L487: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u032_b", site: 542 },
];
const L488: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_b", site: 543 },
];
const L489: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_b", site: 544 },
];
const L490: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 545 },
    Step::Attach(Bone::Index(0)),
];
const L491: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 546 },
    Step::Attach(Bone::Index(0)),
];
const L492: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u011_a", site: 547 },
];
const L493: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u011_a", site: 548 },
];
const L494: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_c", site: 549 },
];
const L495: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_b", site: 550 },
];
const L496: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_b", site: 551 },
];
const L497: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 552 },
];
const L498: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 553 },
];
const L499: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_b", site: 554 },
];
const L500: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_b", site: 555 },
];
const L501: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u031_b", site: 556 },
];
const L502: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u031_b", site: 557 },
];
const L503: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u022_b", site: 558 },
];
const L504: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u031_b", site: 559 },
    Step::Spawn { class: "LineageEffect.m_u031_b", site: 560 },
    Step::Delayed { time: Scalar::Const(0.25), a: 1, b: 0 },
];
const L505: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u031_b", site: 561 },
    Step::Spawn { class: "LineageEffect.m_u031_b", site: 562 },
    Step::Delayed { time: Scalar::Const(0.2), a: 1, b: 0 },
];
const L506: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u039_c", site: 563 },
    Step::Spawn { class: "LineageEffect.m_u039_d", site: 564 },
];
const L507: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_c", site: 565 },
    Step::Spawn { class: "LineageEffect.m_u038_c", site: 566 },
];
const L508: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 567 },
    Step::Spawn { class: "LineageEffect.m_u800_b", site: 568 },
];
const L509: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 569 },
    Step::Spawn { class: "LineageEffect.m_u800_b", site: 570 },
];
const L510: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 567 },
    Step::Spawn { class: "LineageEffect.m_u800_b", site: 571 },
];
const L511: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 569 },
    Step::Spawn { class: "LineageEffect.m_u800_b", site: 572 },
];
const L512: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 573 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 574 },
];
const L513: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 575 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 576 },
];
const L514: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 573 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 577 },
];
const L515: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 575 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 578 },
];
const L516: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 579 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 580 },
];
const L517: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 581 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 582 },
];
const L518: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 579 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 583 },
];
const L519: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 581 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 584 },
];
const L520: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u005_a", site: 585 },
];
const L521: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u023_b", site: 586 },
];
const L522: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u023_b", site: 587 },
];
const L523: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u005_a", site: 588 },
];
const L524: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_b", site: 589 },
];
const L525: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_b", site: 590 },
];
const L526: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_b", site: 591 },
];
const L527: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_b", site: 592 },
];
const L528: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_b", site: 593 },
];
const L529: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_b", site: 594 },
];
const L530: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 595 },
];
const L531: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 596 },
];
const L532: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u010_c", site: 597 },
];
const L533: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u010_c", site: 598 },
];
const L534: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_b", site: 599 },
];
const L535: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_b", site: 600 },
];
const L536: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u023_b", site: 601 },
];
const L537: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u023_b", site: 602 },
];
const L538: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u014_b", site: 603 },
];
const L539: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u014_b", site: 604 },
];
const L540: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_c", site: 605 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 606 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 607 },
];
const L541: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u007_b", site: 608 },
];
const L542: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u007_b", site: 609 },
];
const L543: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u007_b", site: 610 },
];
const L544: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u005_a", site: 611 },
];
const L545: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_c", site: 612 },
];
const L546: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_c", site: 613 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 614 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 615 },
];
const L547: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_c", site: 616 },
    Step::SizeScale(Scalar::Const(2.0)),
];
const L548: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_b", site: 617 },
];
const L549: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_b", site: 618 },
];
const L550: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 619 },
];
const L551: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u022_b", site: 620 },
];
const L552: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_c", site: 621 },
];
const L553: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 622 },
];
const L554: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 623 },
];
const L555: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_b", site: 624 },
];
const L556: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_b", site: 625 },
];
const L557: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 626 },
];
const L558: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 627 },
];
const L559: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_b", site: 628 },
];
const L560: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_b", site: 629 },
];
const L561: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 630 },
];
const L562: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 631 },
];
const L563: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_b", site: 632 },
];
const L564: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_b", site: 633 },
];
const L565: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u005_b", site: 634 },
    Step::Attach(Bone::Index(0)),
];
const L566: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u005_b", site: 635 },
    Step::Attach(Bone::Index(0)),
];
const L567: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 636 },
    Step::Attach(Bone::Index(0)),
];
const L568: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 637 },
    Step::Attach(Bone::Index(0)),
];
const L569: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_b", site: 638 },
];
const L570: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_b", site: 639 },
];
const L571: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 640 },
    Step::Attach(Bone::Index(0)),
];
const L572: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 641 },
    Step::Attach(Bone::Index(0)),
];
const L573: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u009_c", site: 642 },
];
const L574: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u009_c", site: 643 },
];
const L575: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_c", site: 644 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 645 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 646 },
];
const L576: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_c", site: 647 },
];
const L577: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 648 },
];
const L578: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 649 },
];
const L579: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 650 },
];
const L580: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 651 },
];
const L581: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u013_c", site: 652 },
];
const L582: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u013_c", site: 653 },
];
const L583: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 654 },
];
const L584: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 655 },
];
const L585: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_c", site: 656 },
];
const L586: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u033_a", site: 657 },
    Step::Spawn { class: "LineageEffect.e_u033_b", site: 658 },
];
const L587: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_a", site: 659 },
    Step::Spawn { class: "LineageEffect.e_u034_b", site: 660 },
];
const L588: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 661 },
];
const L589: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 662 },
];
const L590: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 663 },
];
const L591: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 664 },
];
const L592: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 665 },
    Step::Attach(Bone::Index(0)),
];
const L593: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 666 },
    Step::Attach(Bone::Index(0)),
];
const L594: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_b", site: 667 },
];
const L595: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_b", site: 668 },
];
const L596: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 669 },
];
const L597: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_c", site: 670 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 671 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 672 },
];
const L598: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 673 },
];
const L599: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 674 },
];
const L600: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 675 },
];
const L601: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 676 },
];
const L602: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_c", site: 677 },
];
const L603: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_b", site: 678 },
];
const L604: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u005_b", site: 679 },
];
const L605: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_b", site: 680 },
];
const L606: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_b", site: 681 },
];
const L607: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 682 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 683 },
];
const L608: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 684 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 685 },
];
const L609: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 682 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 686 },
];
const L610: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 684 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 687 },
];
const L611: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 688 },
];
const L612: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 689 },
];
const L613: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u027_b", site: 690 },
];
const L614: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u027_b", site: 691 },
];
const L615: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_a", site: 692 },
    Step::Spawn { class: "LineageEffect.e_u034_b", site: 693 },
];
const L616: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u043_a", site: 694 },
];
const L617: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u051_a", site: 695 },
];
const L618: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u050_b", site: 696 },
];
const L619: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u050_b", site: 697 },
];
const L620: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u044_b", site: 698 },
];
const L621: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 699 },
];
const L622: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 700 },
];
const L623: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 701 },
    Step::Attach(Bone::Index(0)),
];
const L624: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 702 },
    Step::Attach(Bone::Index(0)),
];
const L625: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_b", site: 703 },
];
const L626: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_b", site: 704 },
];
const L627: &[Step] = &[
    Step::Spawn { class: "LineageEffect.NSpear_sp", site: 705 },
];
const L628: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u063_a", site: 706 },
];
const L629: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u066_a", site: 707 },
];
const L630: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u064_a", site: 708 },
    Step::Spawn { class: "LineageEffect.e_u064_a", site: 709 },
    Step::Delayed { time: Scalar::Const(0.5), a: 1, b: 0 },
    Step::Spawn { class: "LineageEffect.e_u064_a", site: 710 },
    Step::Delayed { time: Scalar::Const(0.63), a: 1, b: 0 },
    Step::Spawn { class: "LineageEffect.e_u064_a", site: 711 },
    Step::Delayed { time: Scalar::Const(1.0), a: 1, b: 0 },
    Step::Spawn { class: "LineageEffect.e_u064_a", site: 712 },
    Step::Delayed { time: Scalar::Const(1.1), a: 1, b: 0 },
    Step::Spawn { class: "LineageEffect.e_u064_a", site: 713 },
    Step::Delayed { time: Scalar::Const(1.14), a: 1, b: 0 },
    Step::Spawn { class: "LineageEffect.e_u064_a", site: 714 },
    Step::Delayed { time: Scalar::Const(1.2), a: 1, b: 0 },
    Step::Spawn { class: "LineageEffect.e_u064_a", site: 715 },
    Step::Delayed { time: Scalar::Const(1.22), a: 1, b: 0 },
    Step::Spawn { class: "LineageEffect.e_u064_a", site: 716 },
    Step::Delayed { time: Scalar::Const(1.39), a: 1, b: 0 },
    Step::Spawn { class: "LineageEffect.e_u064_a", site: 717 },
    Step::Delayed { time: Scalar::Const(1.42), a: 1, b: 0 },
    Step::Spawn { class: "LineageEffect.e_u064_a", site: 718 },
    Step::Delayed { time: Scalar::Const(1.48), a: 1, b: 0 },
    Step::Spawn { class: "LineageEffect.e_u064_a", site: 719 },
    Step::Delayed { time: Scalar::Const(1.6), a: 1, b: 0 },
    Step::Spawn { class: "LineageEffect.e_u064_a", site: 720 },
    Step::Delayed { time: Scalar::Const(1.85), a: 1, b: 0 },
];
const L631: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u067_hand", site: 721 },
    Step::Attach(Bone::RightHand),
    Step::Spawn { class: "LineageEffect.e_u067_a", site: 722 },
];
const L632: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 723 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 724 },
];
const L633: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 723 },
    Step::Attach(Bone::Index(0)),
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 724 },
    Step::Spawn { class: "LineageEffect.m_u006_e", site: 725 },
    Step::Attach(Bone::Index(0)),
];
const L634: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u033_b", site: 726 },
];
const L635: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u033_b", site: 727 },
];
const L636: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 728 },
    Step::Spawn { class: "LineageEffect.e_u060_b", site: 729 },
    Step::Spawn { class: "LineageEffect.e_u060_b", site: 730 },
];
const L637: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_d", site: 728 },
    Step::Spawn { class: "LineageEffect.e_u060_b", site: 729 },
    Step::Spawn { class: "LineageEffect.e_u060_b", site: 731 },
];
const L638: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_c", site: 732 },
];
const L639: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_c", site: 733 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 734 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 735 },
];
const L640: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u005_b", site: 736 },
    Step::Attach(Bone::Index(0)),
];
const L641: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u005_b", site: 737 },
    Step::Attach(Bone::Index(0)),
];
const L642: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_c", site: 738 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 739 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 740 },
];
const L643: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_c", site: 741 },
];
const L644: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_a", site: 742 },
    Step::Spawn { class: "LineageEffect.e_u034_b", site: 743 },
];
const L645: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_c", site: 744 },
];
const L646: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_a", site: 745 },
    Step::Spawn { class: "LineageEffect.e_u034_b", site: 746 },
];
const L647: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_b", site: 747 },
];
const L648: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_b", site: 748 },
];
const L649: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_b", site: 749 },
];
const L650: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_b", site: 750 },
];
const L651: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 751 },
];
const L652: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 752 },
];
const L653: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 753 },
];
const L654: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u004_b", site: 754 },
];
const L655: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_b", site: 755 },
];
const L656: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u008_b", site: 756 },
];
const L657: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_b", site: 757 },
];
const L658: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u036_b", site: 758 },
];
const L659: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u016_b", site: 759 },
];
const L660: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u016_b", site: 760 },
];
const L661: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_b", site: 761 },
];
const L662: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u016_b", site: 762 },
];
const L663: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u006_c", site: 763 },
];
const L664: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 764 },
];
const L665: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 765 },
];
const L666: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_b", site: 766 },
];
const L667: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u021_b", site: 767 },
];
const L668: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u009_c", site: 768 },
];
const L669: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u009_c", site: 769 },
];
const L670: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 770 },
];
const L671: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 771 },
];
const L672: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u013_c", site: 772 },
];
const L673: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u013_c", site: 773 },
];
const L674: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u027_b", site: 774 },
];
const L675: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u027_b", site: 775 },
];
const L676: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u000_c", site: 776 },
];
const L677: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_c", site: 777 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 778 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 779 },
];
const L678: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_b", site: 780 },
];
const L679: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u019_b", site: 781 },
];
const L680: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u026_c", site: 782 },
];
const L681: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u005_b", site: 783 },
    Step::Attach(Bone::Index(0)),
];
const L682: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u005_b", site: 784 },
    Step::Attach(Bone::Index(0)),
];
const L683: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 785 },
    Step::Attach(Bone::Index(0)),
];
const L684: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u007_b", site: 786 },
    Step::Attach(Bone::Index(0)),
];
const L685: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u037_c", site: 787 },
    Step::Spawn { class: "LineageEffect.m_u037_d", site: 788 },
];
const L686: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u505_b", site: 789 },
    Step::Attach(Bone::LeftHand),
];
const L687: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 790 },
];
const L688: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u001_b", site: 791 },
];
const L689: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 792 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 793 },
];
const L690: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 794 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 795 },
];
const L691: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 792 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 796 },
];
const L692: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 794 },
    Step::Spawn { class: "LineageEffect.m_u024_c", site: 797 },
];
const L693: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u071_b", site: 798 },
];
const L694: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 799 },
];
const L695: &[Step] = &[
    Step::Spawn { class: "LineageEffect.p_u004_a", site: 800 },
];
const L696: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u005_a", site: 801 },
];
const L697: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u072_a", site: 802 },
];
const L698: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u057_b", site: 803 },
];
const L699: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u005_a", site: 804 },
];
const L700: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u038_c", site: 805 },
    Step::Spawn { class: "LineageEffect.m_u038_c", site: 806 },
];
const L701: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 807 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 808 },
];
const L702: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 809 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 810 },
];
const L703: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 807 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 811 },
];
const L704: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u025_b", site: 809 },
    Step::Spawn { class: "LineageEffect.m_u025_c", site: 812 },
];
const L705: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_a", site: 813 },
    Step::Spawn { class: "LineageEffect.e_u034_b", site: 814 },
];
const L706: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u034_a", site: 815 },
    Step::Spawn { class: "LineageEffect.e_u034_b", site: 816 },
];
const L707: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u033_b", site: 817 },
];
const L708: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u033_b", site: 818 },
];
const L709: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u082_a", site: 819 },
];
const L710: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u504_b", site: 820 },
];
const L711: &[Step] = &[
    Step::Spawn { class: "LineageEffect.NSpear_sp", site: 821 },
];
const L712: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u003_c", site: 822 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 823 },
    Step::Spawn { class: "LineageEffect.m_u003_b", site: 824 },
];
const L713: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u019_b", site: 825 },
    Step::Spawn { class: "LineageEffect.s_u019_c", site: 826 },
];
const L714: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u019_b", site: 827 },
    Step::Spawn { class: "LineageEffect.s_u019_c", site: 828 },
    Step::Spawn { class: "LineageEffect.s_u019_b", site: 829 },
    Step::Spawn { class: "LineageEffect.s_u019_c", site: 830 },
];
const L715: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 831 },
    Step::Spawn { class: "LineageEffect.m_u800_b", site: 832 },
];
const L716: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 833 },
    Step::Spawn { class: "LineageEffect.m_u800_b", site: 834 },
];
const L717: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 831 },
    Step::Spawn { class: "LineageEffect.m_u800_b", site: 835 },
];
const L718: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u024_b", site: 833 },
    Step::Spawn { class: "LineageEffect.m_u800_b", site: 836 },
];
const L719: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u013_c", site: 837 },
];
const L720: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u013_c", site: 838 },
];
const L721: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 839 },
];
const L722: &[Step] = &[
    Step::Spawn { class: "LineageEffect.m_u018_b", site: 840 },
];
const L723: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u802_fallback", site: 841 },
];
const L724: &[Step] = &[
    Step::Spawn { class: "LineageEffect.e_u524_a", site: 842 },
];
const L725: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u505_b", site: 843 },
    Step::Attach(Bone::LeftHand),
];
const L726: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u003_d", site: 844 },
    Step::Attach(Bone::RightHand),
];
const L727: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u003_d", site: 845 },
    Step::Attach(Bone::RightHand),
];
const L728: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u003_d", site: 846 },
    Step::Attach(Bone::RightHand),
];
const L729: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u003_d", site: 847 },
    Step::Attach(Bone::RightHand),
];
const L730: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u003_d", site: 848 },
    Step::Attach(Bone::RightHand),
];
const L731: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u003_d", site: 849 },
    Step::Attach(Bone::RightHand),
    Step::Spawn { class: "LineageEffect.s_u509_d", site: 850 },
];
const L732: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u003_d", site: 851 },
    Step::Attach(Bone::RightHand),
    Step::Spawn { class: "LineageEffect.s_u509_d", site: 852 },
];
const L733: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u003_d", site: 853 },
    Step::Attach(Bone::RightHand),
];
const L734: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u003_d", site: 854 },
    Step::Attach(Bone::RightHand),
];
const L735: &[Step] = &[
    Step::Spawn { class: "LineageEffect.NSpear_sp", site: 855 },
    Step::Attach(Bone::LeftHand),
];
const L736: &[Step] = &[
    Step::Spawn { class: "LineageEffect.s_u003_d", site: 856 },
    Step::Attach(Bone::RightHand),
];

const V_CAST_1: Variants = &[(0, L0)];
const V_CAST_2: Variants = &[(0, L1)];
const V_CAST_3: Variants = &[(0, L2), (4, L3)];
const V_CAST_4: Variants = &[(0, L4)];
const V_CAST_5: Variants = &[(0, L0)];
const V_CAST_6: Variants = &[(0, L5)];
const V_CAST_7: Variants = &[(0, L5)];
const V_CAST_8: Variants = &[(0, L4)];
const V_CAST_9: Variants = &[(0, L5)];
const V_CAST_A: Variants = &[(0, L6)];
const V_CAST_B: Variants = &[(0, L1)];
const V_CAST_C: Variants = &[(0, L1)];
const V_CAST_D: Variants = &[(0, L6)];
const V_CAST_F: Variants = &[(0, L1)];
const V_CAST_10: Variants = &[(0, L7), (4, L8)];
const V_CAST_11: Variants = &[(0, L5)];
const V_CAST_12: Variants = &[(0, L1)];
const V_CAST_13: Variants = &[(0, L9)];
const V_CAST_15: Variants = &[(0, L10)];
const V_CAST_16: Variants = &[(0, L6)];
const V_CAST_18: Variants = &[(0, L11)];
const V_CAST_19: Variants = &[(0, L6)];
const V_CAST_1B: Variants = &[(0, L12)];
const V_CAST_1C: Variants = &[(0, L1)];
const V_CAST_1D: Variants = &[(0, L13)];
const V_CAST_1E: Variants = &[(0, L14), (4, L15)];
const V_CAST_21: Variants = &[(0, L6)];
const V_CAST_22: Variants = &[(0, L16)];
const V_CAST_23: Variants = &[(0, L5)];
const V_CAST_24: Variants = &[(0, L14), (4, L15)];
const V_CAST_2C: Variants = &[(0, L16)];
const V_CAST_2D: Variants = &[(0, L16)];
const V_CAST_2E: Variants = &[(0, L17)];
const V_CAST_30: Variants = &[(0, L14), (4, L15)];
const V_CAST_31: Variants = &[(0, L18)];
const V_CAST_32: Variants = &[(0, L19)];
const V_CAST_33: Variants = &[(0, L1)];
const V_CAST_36: Variants = &[(0, L5)];
const V_CAST_38: Variants = &[(0, L11)];
const V_CAST_3A: Variants = &[(0, L20)];
const V_CAST_3D: Variants = &[(0, L10)];
const V_CAST_41: Variants = &[(0, L1)];
const V_CAST_43: Variants = &[(0, L6)];
const V_CAST_45: Variants = &[(0, L16)];
const V_CAST_46: Variants = &[(0, L21)];
const V_CAST_48: Variants = &[(0, L22)];
const V_CAST_4B: Variants = &[(0, L4)];
const V_CAST_4C: Variants = &[(0, L19)];
const V_CAST_4D: Variants = &[(0, L19)];
const V_CAST_4E: Variants = &[(0, L19)];
const V_CAST_50: Variants = &[(0, L4)];
const V_CAST_51: Variants = &[(0, L5)];
const V_CAST_52: Variants = &[(0, L19)];
const V_CAST_53: Variants = &[(0, L19)];
const V_CAST_54: Variants = &[(0, L5)];
const V_CAST_56: Variants = &[(0, L4)];
const V_CAST_57: Variants = &[(0, L4)];
const V_CAST_58: Variants = &[(0, L4)];
const V_CAST_5B: Variants = &[(0, L19)];
const V_CAST_5C: Variants = &[(0, L5)];
const V_CAST_5E: Variants = &[(0, L19)];
const V_CAST_5F: Variants = &[(0, L13)];
const V_CAST_60: Variants = &[(0, L7), (4, L8)];
const V_CAST_61: Variants = &[(0, L23)];
const V_CAST_62: Variants = &[(0, L24)];
const V_CAST_63: Variants = &[(0, L4)];
const V_CAST_64: Variants = &[(0, L2), (4, L3)];
const V_CAST_65: Variants = &[(0, L25)];
const V_CAST_66: Variants = &[(0, L1)];
const V_CAST_67: Variants = &[(0, L26)];
const V_CAST_68: Variants = &[(0, L4)];
const V_CAST_69: Variants = &[(0, L5)];
const V_CAST_6A: Variants = &[(0, L1)];
const V_CAST_6B: Variants = &[(0, L1)];
const V_CAST_6D: Variants = &[(0, L19)];
const V_CAST_6E: Variants = &[(0, L22)];
const V_CAST_6F: Variants = &[(0, L22)];
const V_CAST_70: Variants = &[(0, L19)];
const V_CAST_73: Variants = &[(0, L1)];
const V_CAST_74: Variants = &[(0, L23)];
const V_CAST_78: Variants = &[(0, L5)];
const V_CAST_79: Variants = &[(0, L19)];
const V_CAST_7A: Variants = &[(0, L1)];
const V_CAST_7B: Variants = &[(0, L22)];
const V_CAST_7F: Variants = &[(0, L1)];
const V_CAST_81: Variants = &[(0, L27)];
const V_CAST_82: Variants = &[(0, L4)];
const V_CAST_83: Variants = &[(0, L4)];
const V_CAST_8B: Variants = &[(0, L28)];
const V_CAST_B0: Variants = &[(0, L29)];
const V_CAST_B5: Variants = &[(0, L29)];
const V_CAST_BE: Variants = &[(0, L30), (4, L31)];
const V_CAST_DF: Variants = &[(0, L32), (4, L33)];
const V_CAST_E6: Variants = &[(0, L34)];
const V_CAST_F5: Variants = &[(0, L32), (4, L33)];
const V_CAST_F6: Variants = &[(0, L35)];
const V_CAST_F7: Variants = &[(0, L36)];
const V_CAST_FE: Variants = &[(0, L37)];
const V_CAST_FF: Variants = &[(0, L30), (4, L31)];
const V_CAST_104: Variants = &[(0, L30), (4, L31)];
const V_CAST_105: Variants = &[(0, L38)];
const V_CAST_106: Variants = &[(0, L39)];
const V_CAST_107: Variants = &[(0, L40)];
const V_CAST_116: Variants = &[(0, L36)];
const V_CAST_117: Variants = &[(0, L40)];
const V_CAST_118: Variants = &[(0, L40)];
const V_CAST_119: Variants = &[(0, L40)];
const V_CAST_11A: Variants = &[(0, L29)];
const V_CAST_11B: Variants = &[(0, L36)];
const V_CAST_11C: Variants = &[(0, L40)];
const V_CAST_11E: Variants = &[(0, L41)];
const V_CAST_11F: Variants = &[(0, L42)];
const V_CAST_121: Variants = &[(0, L43)];
const V_CAST_124: Variants = &[(0, L44)];
const V_CAST_129: Variants = &[(0, L45)];
const V_CAST_12A: Variants = &[(0, L44)];
const V_CAST_12B: Variants = &[(0, L46)];
const V_CAST_12D: Variants = &[(0, L46)];
const V_CAST_12E: Variants = &[(0, L47)];
const V_CAST_12F: Variants = &[(0, L44)];
const V_CAST_13A: Variants = &[(0, L48)];
const V_CAST_157: Variants = &[(0, L49)];
const V_CAST_162: Variants = &[(0, L49)];
const V_CAST_16A: Variants = &[(0, L50)];
const V_CAST_171: Variants = &[(0, L51)];
const V_CAST_3EA: Variants = &[(0, L52), (2, L53)];
const V_CAST_3EB: Variants = &[(0, L52), (2, L53)];
const V_CAST_3EC: Variants = &[(0, L52), (2, L53)];
const V_CAST_3ED: Variants = &[(0, L54), (2, L55)];
const V_CAST_3EE: Variants = &[(0, L54), (2, L55)];
const V_CAST_3EF: Variants = &[(0, L56), (2, L57)];
const V_CAST_3F0: Variants = &[(0, L58), (2, L59)];
const V_CAST_3F1: Variants = &[(0, L58), (2, L59)];
const V_CAST_3F2: Variants = &[(0, L58), (2, L59)];
const V_CAST_3F3: Variants = &[(0, L60)];
const V_CAST_3F4: Variants = &[(0, L61)];
const V_CAST_3F5: Variants = &[(0, L61)];
const V_CAST_3F7: Variants = &[(0, L62)];
const V_CAST_3F8: Variants = &[(0, L63)];
const V_CAST_3FA: Variants = &[(0, L61)];
const V_CAST_3FC: Variants = &[(0, L64)];
const V_CAST_403: Variants = &[(0, L65)];
const V_CAST_404: Variants = &[(0, L66)];
const V_CAST_407: Variants = &[(0, L66)];
const V_CAST_408: Variants = &[(0, L67)];
const V_CAST_409: Variants = &[(0, L67)];
const V_CAST_40A: Variants = &[(0, L68)];
const V_CAST_40B: Variants = &[(0, L67)];
const V_CAST_40C: Variants = &[(0, L67)];
const V_CAST_410: Variants = &[(0, L67)];
const V_CAST_412: Variants = &[(0, L66)];
const V_CAST_413: Variants = &[(0, L69)];
const V_CAST_414: Variants = &[(0, L69)];
const V_CAST_415: Variants = &[(0, L69)];
const V_CAST_417: Variants = &[(0, L69)];
const V_CAST_418: Variants = &[(0, L69)];
const V_CAST_419: Variants = &[(0, L68)];
const V_CAST_41A: Variants = &[(0, L70)];
const V_CAST_420: Variants = &[(0, L71)];
const V_CAST_423: Variants = &[(0, L71)];
const V_CAST_426: Variants = &[(0, L72)];
const V_CAST_428: Variants = &[(0, L72)];
const V_CAST_429: Variants = &[(0, L71)];
const V_CAST_42C: Variants = &[(0, L73)];
const V_CAST_42D: Variants = &[(0, L72)];
const V_CAST_42F: Variants = &[(0, L72)];
const V_CAST_430: Variants = &[(0, L72)];
const V_CAST_431: Variants = &[(0, L71)];
const V_CAST_432: Variants = &[(0, L72)];
const V_CAST_433: Variants = &[(0, L72)];
const V_CAST_435: Variants = &[(0, L74)];
const V_CAST_436: Variants = &[(0, L73)];
const V_CAST_43B: Variants = &[(0, L72)];
const V_CAST_43D: Variants = &[(0, L71)];
const V_CAST_43E: Variants = &[(0, L74)];
const V_CAST_43F: Variants = &[(0, L73)];
const V_CAST_442: Variants = &[(0, L75)];
const V_CAST_444: Variants = &[(0, L76), (2, L77)];
const V_CAST_447: Variants = &[(0, L76), (2, L77)];
const V_CAST_448: Variants = &[(0, L76), (2, L77)];
const V_CAST_449: Variants = &[(0, L76), (2, L77)];
const V_CAST_44B: Variants = &[(0, L76), (2, L77)];
const V_CAST_44C: Variants = &[(0, L78), (2, L79)];
const V_CAST_44D: Variants = &[(0, L80)];
const V_CAST_44E: Variants = &[(0, L76), (2, L77)];
const V_CAST_450: Variants = &[(0, L76), (2, L77)];
const V_CAST_451: Variants = &[(0, L76), (2, L77)];
const V_CAST_453: Variants = &[(0, L78), (2, L79)];
const V_CAST_454: Variants = &[(0, L80)];
const V_CAST_457: Variants = &[(0, L81)];
const V_CAST_466: Variants = &[(0, L82)];
const V_CAST_467: Variants = &[(0, L82)];
const V_CAST_468: Variants = &[(0, L81)];
const V_CAST_469: Variants = &[(0, L81)];
const V_CAST_472: Variants = &[(0, L71)];
const V_CAST_473: Variants = &[(0, L71)];
const V_CAST_474: Variants = &[(0, L71)];
const V_CAST_475: Variants = &[(0, L71)];
const V_CAST_478: Variants = &[(0, L83)];
const V_CAST_479: Variants = &[(0, L83)];
const V_CAST_47A: Variants = &[(0, L83)];
const V_CAST_47B: Variants = &[(0, L84)];
const V_CAST_47C: Variants = &[(0, L84)];
const V_CAST_47F: Variants = &[(0, L84)];
const V_CAST_482: Variants = &[(0, L85)];
const V_CAST_483: Variants = &[(0, L84)];
const V_CAST_484: Variants = &[(0, L86)];
const V_CAST_485: Variants = &[(0, L87)];
const V_CAST_487: Variants = &[(0, L84)];
const V_CAST_488: Variants = &[(0, L86)];
const V_CAST_48B: Variants = &[(0, L86)];
const V_CAST_48C: Variants = &[(0, L88)];
const V_CAST_48F: Variants = &[(0, L89)];
const V_CAST_490: Variants = &[(0, L86)];
const V_CAST_491: Variants = &[(0, L86)];
const V_CAST_492: Variants = &[(0, L86)];
const V_CAST_493: Variants = &[(0, L90)];
const V_CAST_494: Variants = &[(0, L91)];
const V_CAST_496: Variants = &[(0, L92)];
const V_CAST_497: Variants = &[(0, L93), (2, L94)];
const V_CAST_498: Variants = &[(0, L95), (2, L96)];
const V_CAST_499: Variants = &[(0, L97)];
const V_CAST_49A: Variants = &[(0, L98), (2, L99)];
const V_CAST_49D: Variants = &[(0, L100)];
const V_CAST_49E: Variants = &[(0, L87)];
const V_CAST_49F: Variants = &[(0, L101)];
const V_CAST_4A0: Variants = &[(0, L92)];
const V_CAST_4A5: Variants = &[(0, L102)];
const V_CAST_4A7: Variants = &[(0, L87)];
const V_CAST_4B1: Variants = &[(0, L103)];
const V_CAST_4B4: Variants = &[(0, L102)];
const V_CAST_4B6: Variants = &[(0, L104)];
const V_CAST_4B8: Variants = &[(0, L105), (2, L106)];
const V_CAST_4B9: Variants = &[(0, L105), (2, L106)];
const V_CAST_4BA: Variants = &[(0, L105), (2, L106)];
const V_CAST_4BD: Variants = &[(0, L105), (2, L106)];
const V_CAST_4C0: Variants = &[(0, L107)];
const V_CAST_4C1: Variants = &[(0, L108)];
const V_CAST_4C2: Variants = &[(0, L108)];
const V_CAST_4C3: Variants = &[(0, L108)];
const V_CAST_4C4: Variants = &[(0, L100)];
const V_CAST_4C6: Variants = &[(0, L88)];
const V_CAST_4C7: Variants = &[(0, L109)];
const V_CAST_4C8: Variants = &[(0, L109)];
const V_CAST_4C9: Variants = &[(0, L85)];
const V_CAST_4CA: Variants = &[(0, L85)];
const V_CAST_4CB: Variants = &[(0, L85)];
const V_CAST_4CC: Variants = &[(0, L85)];
const V_CAST_4CD: Variants = &[(0, L110), (2, L111)];
const V_CAST_4CE: Variants = &[(0, L100)];
const V_CAST_4CF: Variants = &[(0, L112)];
const V_CAST_4D0: Variants = &[(0, L113)];
const V_CAST_4D1: Variants = &[(0, L103)];
const V_CAST_4D2: Variants = &[(0, L84)];
const V_CAST_4D3: Variants = &[(0, L101)];
const V_CAST_4D4: Variants = &[(0, L92)];
const V_CAST_4D5: Variants = &[(0, L114), (2, L115)];
const V_CAST_4D6: Variants = &[(0, L113)];
const V_CAST_4D7: Variants = &[(0, L95), (2, L96)];
const V_CAST_4D8: Variants = &[(0, L116)];
const V_CAST_4D9: Variants = &[(0, L87)];
const V_CAST_4DA: Variants = &[(0, L116)];
const V_CAST_4DB: Variants = &[(0, L117)];
const V_CAST_4DC: Variants = &[(0, L118), (2, L119)];
const V_CAST_4DD: Variants = &[(0, L84)];
const V_CAST_4DE: Variants = &[(0, L105), (2, L106)];
const V_CAST_4DF: Variants = &[(0, L105), (2, L106)];
const V_CAST_4E0: Variants = &[(0, L105), (2, L106)];
const V_CAST_4E1: Variants = &[(0, L110), (2, L111)];
const V_CAST_4E2: Variants = &[(0, L120), (2, L121)];
const V_CAST_4E3: Variants = &[(0, L110), (2, L111)];
const V_CAST_4E4: Variants = &[(0, L120), (2, L121)];
const V_CAST_4E5: Variants = &[(0, L110), (2, L111)];
const V_CAST_4E6: Variants = &[(0, L122)];
const V_CAST_4E7: Variants = &[(0, L123)];
const V_CAST_4E8: Variants = &[(0, L110), (2, L111)];
const V_CAST_4E9: Variants = &[(0, L124)];
const V_CAST_4EA: Variants = &[(0, L108)];
const V_CAST_4EB: Variants = &[(0, L113)];
const V_CAST_4EC: Variants = &[(0, L120), (2, L121)];
const V_CAST_4ED: Variants = &[(0, L120), (2, L121)];
const V_CAST_4EF: Variants = &[(0, L86)];
const V_CAST_4F0: Variants = &[(0, L125)];
const V_CAST_4F1: Variants = &[(0, L125)];
const V_CAST_4F2: Variants = &[(0, L84)];
const V_CAST_4F3: Variants = &[(0, L84)];
const V_CAST_4F4: Variants = &[(0, L116)];
const V_CAST_4F5: Variants = &[(0, L86)];
const V_CAST_4F7: Variants = &[(0, L108)];
const V_CAST_4F8: Variants = &[(0, L89)];
const V_CAST_4F9: Variants = &[(0, L109)];
const V_CAST_4FA: Variants = &[(0, L91)];
const V_CAST_4FB: Variants = &[(0, L112)];
const V_CAST_4FC: Variants = &[(0, L85)];
const V_CAST_4FD: Variants = &[(0, L85)];
const V_CAST_4FE: Variants = &[(0, L85)];
const V_CAST_4FF: Variants = &[(0, L85)];
const V_CAST_500: Variants = &[(0, L85)];
const V_CAST_501: Variants = &[(0, L85)];
const V_CAST_502: Variants = &[(0, L110), (2, L111)];
const V_CAST_504: Variants = &[(0, L120), (2, L121)];
const V_CAST_505: Variants = &[(0, L90)];
const V_CAST_506: Variants = &[(0, L126)];
const V_CAST_507: Variants = &[(0, L127)];
const V_CAST_508: Variants = &[(0, L112)];
const V_CAST_509: Variants = &[(0, L100)];
const V_CAST_50A: Variants = &[(0, L93), (2, L94)];
const V_CAST_50B: Variants = &[(0, L95), (2, L96)];
const V_CAST_50C: Variants = &[(0, L128)];
const V_CAST_50D: Variants = &[(0, L128)];
const V_CAST_50E: Variants = &[(0, L128)];
const V_CAST_518: Variants = &[(0, L129)];
const V_CAST_552: Variants = &[(0, L130), (2, L131)];
const V_CAST_553: Variants = &[(0, L130), (2, L131)];
const V_CAST_554: Variants = &[(0, L132), (2, L133)];
const V_CAST_555: Variants = &[(0, L132), (2, L133)];
const V_CAST_556: Variants = &[(0, L134), (2, L135)];
const V_CAST_557: Variants = &[(0, L134), (2, L135)];
const V_CAST_7DD: Variants = &[(0, L136)];
const V_CAST_7DE: Variants = &[(0, L137)];
const V_CAST_7F8: Variants = &[(0, L138)];
const V_CAST_7F9: Variants = &[(0, L138)];
const V_CAST_7FE: Variants = &[(0, L139)];
const V_CAST_801: Variants = &[(0, L140)];
const V_CAST_802: Variants = &[(0, L141)];
const V_CAST_803: Variants = &[(0, L141)];
const V_CAST_804: Variants = &[(0, L141)];
const V_CAST_805: Variants = &[(0, L142)];
const V_CAST_806: Variants = &[(0, L143)];
const V_CAST_807: Variants = &[(0, L144)];
const V_CAST_808: Variants = &[(0, L145)];
const V_CAST_809: Variants = &[(0, L144)];
const V_CAST_80A: Variants = &[(0, L146)];
const V_CAST_80B: Variants = &[(0, L147)];
const V_CAST_80E: Variants = &[(0, L148)];
const V_CAST_810: Variants = &[(0, L149)];
const V_CAST_830: Variants = &[(0, L150)];
const V_CAST_831: Variants = &[(0, L151)];
const V_CAST_833: Variants = &[(0, L152)];
const V_CAST_834: Variants = &[(0, L152)];
const V_CAST_FA1: Variants = &[(0, L153)];
const V_CAST_FA2: Variants = &[(0, L154)];
const V_CAST_FB1: Variants = &[(0, L155)];
const V_CAST_FB4: Variants = &[(0, L156)];
const V_CAST_FB8: Variants = &[(0, L156)];
const V_CAST_FB9: Variants = &[(0, L157)];
const V_CAST_FBA: Variants = &[(0, L158)];
const V_CAST_FBB: Variants = &[(0, L159)];
const V_CAST_FBC: Variants = &[(0, L160)];
const V_CAST_FBD: Variants = &[(0, L160)];
const V_CAST_FBE: Variants = &[(0, L161)];
const V_CAST_FBF: Variants = &[(0, L162)];
const V_CAST_FC0: Variants = &[(0, L163), (4, L164)];
const V_CAST_FC1: Variants = &[(0, L165)];
const V_CAST_FC2: Variants = &[(0, L166)];
const V_CAST_FC3: Variants = &[(0, L167)];
const V_CAST_FC4: Variants = &[(0, L168)];
const V_CAST_FC5: Variants = &[(0, L169)];
const V_CAST_FC6: Variants = &[(0, L170)];
const V_CAST_FC7: Variants = &[(0, L154)];
const V_CAST_FC8: Variants = &[(0, L171)];
const V_CAST_FCA: Variants = &[(0, L158)];
const V_CAST_FCB: Variants = &[(0, L158)];
const V_CAST_FCC: Variants = &[(0, L172)];
const V_CAST_FCE: Variants = &[(0, L173)];
const V_CAST_FCF: Variants = &[(0, L174)];
const V_CAST_FDF: Variants = &[(0, L175)];
const V_CAST_FE0: Variants = &[(0, L175)];
const V_CAST_FE1: Variants = &[(0, L176)];
const V_CAST_FE2: Variants = &[(0, L177), (2, L178)];
const V_CAST_FE3: Variants = &[(0, L175)];
const V_CAST_FE8: Variants = &[(0, L175)];
const V_CAST_FE9: Variants = &[(0, L175)];
const V_CAST_FEA: Variants = &[(0, L179)];
const V_CAST_FEB: Variants = &[(0, L175)];
const V_CAST_FEC: Variants = &[(0, L180)];
const V_CAST_FED: Variants = &[(0, L181)];
const V_CAST_FEE: Variants = &[(0, L182)];
const V_CAST_FF0: Variants = &[(0, L183)];
const V_CAST_FF1: Variants = &[(0, L183)];
const V_CAST_FF7: Variants = &[(0, L182)];
const V_CAST_FF8: Variants = &[(0, L175)];
const V_CAST_FF9: Variants = &[(0, L184)];
const V_CAST_FFA: Variants = &[(0, L184)];
const V_CAST_FFB: Variants = &[(0, L184)];
const V_CAST_FFC: Variants = &[(0, L184)];
const V_CAST_FFE: Variants = &[(0, L185)];
const V_CAST_1000: Variants = &[(0, L184)];
const V_CAST_1001: Variants = &[(0, L186), (2, L187)];
const V_CAST_1002: Variants = &[(0, L173)];
const V_CAST_1003: Variants = &[(0, L173)];
const V_CAST_1004: Variants = &[(0, L182)];
const V_CAST_1005: Variants = &[(0, L175)];
const V_CAST_1006: Variants = &[(0, L173)];
const V_CAST_1007: Variants = &[(0, L184)];
const V_CAST_1008: Variants = &[(0, L188)];
const V_CAST_1009: Variants = &[(0, L175)];
const V_CAST_1012: Variants = &[(0, L182)];
const V_CAST_1013: Variants = &[(0, L176)];
const V_CAST_1015: Variants = &[(0, L180)];
const V_CAST_1016: Variants = &[(0, L189)];
const V_CAST_1017: Variants = &[(0, L180)];
const V_CAST_1018: Variants = &[(0, L190)];
const V_CAST_101B: Variants = &[(0, L173)];
const V_CAST_101C: Variants = &[(0, L175)];
const V_CAST_1020: Variants = &[(0, L191)];
const V_CAST_1022: Variants = &[(0, L192), (2, L192)];
const V_CAST_1024: Variants = &[(0, L175)];
const V_CAST_1025: Variants = &[(0, L176)];
const V_CAST_1029: Variants = &[(0, L193)];
const V_CAST_102A: Variants = &[(0, L183)];
const V_CAST_102B: Variants = &[(0, L194)];
const V_CAST_102D: Variants = &[(0, L195)];
const V_CAST_102E: Variants = &[(0, L195)];
const V_CAST_1030: Variants = &[(0, L196)];
const V_CAST_1037: Variants = &[(0, L197)];
const V_CAST_1038: Variants = &[(0, L198)];
const V_CAST_1039: Variants = &[(0, L199)];
const V_CAST_103A: Variants = &[(0, L198)];
const V_CAST_103B: Variants = &[(0, L200), (2, L201)];
const V_CAST_103D: Variants = &[(0, L202)];
const V_CAST_103E: Variants = &[(0, L202)];
const V_CAST_1040: Variants = &[(0, L203)];
const V_CAST_1041: Variants = &[(0, L204), (2, L204)];
const V_CAST_1043: Variants = &[(0, L205)];
const V_CAST_1048: Variants = &[(0, L196)];
const V_CAST_1049: Variants = &[(0, L196)];
const V_CAST_104A: Variants = &[(0, L196)];
const V_CAST_104B: Variants = &[(0, L196)];
const V_CAST_104C: Variants = &[(0, L196)];
const V_CAST_104D: Variants = &[(0, L206)];
const V_CAST_104E: Variants = &[(0, L205)];
const V_CAST_104F: Variants = &[(0, L207)];
const V_CAST_1050: Variants = &[(0, L208)];
const V_CAST_1051: Variants = &[(0, L209)];
const V_CAST_1052: Variants = &[(0, L202)];
const V_CAST_1053: Variants = &[(0, L196)];
const V_CAST_1054: Variants = &[(0, L196)];
const V_CAST_1055: Variants = &[(0, L196)];
const V_CAST_1056: Variants = &[(0, L210)];
const V_CAST_1057: Variants = &[(0, L210)];
const V_CAST_1058: Variants = &[(0, L211)];
const V_CAST_1059: Variants = &[(0, L212)];
const V_CAST_105A: Variants = &[(0, L213)];
const V_CAST_105B: Variants = &[(0, L210)];
const V_CAST_105C: Variants = &[(0, L214)];
const V_CAST_105D: Variants = &[(0, L210)];
const V_CAST_105E: Variants = &[(0, L210)];
const V_CAST_105F: Variants = &[(0, L197)];
const V_CAST_1060: Variants = &[(0, L198)];
const V_CAST_1061: Variants = &[(0, L198)];
const V_CAST_1062: Variants = &[(0, L203)];
const V_CAST_1063: Variants = &[(0, L200), (2, L201)];
const V_CAST_1064: Variants = &[(0, L199)];
const V_CAST_1065: Variants = &[(0, L213)];
const V_CAST_1066: Variants = &[(0, L215)];
const V_CAST_1067: Variants = &[(0, L215)];
const V_CAST_1068: Variants = &[(0, L211)];
const V_CAST_1069: Variants = &[(0, L212)];
const V_CAST_106A: Variants = &[(0, L213)];
const V_CAST_106B: Variants = &[(0, L215)];
const V_CAST_106C: Variants = &[(0, L216), (2, L217)];
const V_CAST_106D: Variants = &[(0, L215)];
const V_CAST_106E: Variants = &[(0, L215)];
const V_CAST_106F: Variants = &[(0, L218)];
const V_CAST_1070: Variants = &[(0, L219)];
const V_CAST_1071: Variants = &[(0, L220)];
const V_CAST_1072: Variants = &[(0, L221), (2, L222)];
const V_CAST_1073: Variants = &[(0, L223)];
const V_CAST_1074: Variants = &[(0, L224)];
const V_CAST_1075: Variants = &[(0, L225)];
const V_CAST_1076: Variants = &[(0, L225)];
const V_CAST_1078: Variants = &[(0, L226)];
const V_CAST_1079: Variants = &[(0, L226)];
const V_CAST_107A: Variants = &[(0, L227)];
const V_CAST_107B: Variants = &[(0, L228)];
const V_CAST_107C: Variants = &[(0, L229)];
const V_CAST_107D: Variants = &[(0, L229)];
const V_CAST_107E: Variants = &[(0, L230)];
const V_CAST_1084: Variants = &[(0, L229)];
const V_CAST_1085: Variants = &[(0, L231)];
const V_CAST_1088: Variants = &[(0, L232)];
const V_CAST_108B: Variants = &[(0, L230)];
const V_CAST_108C: Variants = &[(0, L233), (2, L234)];
const V_CAST_108D: Variants = &[(0, L235)];
const V_CAST_108E: Variants = &[(0, L236), (2, L237)];
const V_CAST_1094: Variants = &[(0, L229)];
const V_CAST_1097: Variants = &[(0, L238)];
const V_CAST_1098: Variants = &[(0, L227)];
const V_CAST_1099: Variants = &[(0, L235)];
const V_CAST_109A: Variants = &[(0, L239), (2, L240)];
const V_CAST_109C: Variants = &[(0, L241)];
const V_CAST_109D: Variants = &[(0, L241)];
const V_CAST_109E: Variants = &[(0, L241)];
const V_CAST_109F: Variants = &[(0, L229)];
const V_CAST_10A1: Variants = &[(0, L242)];
const V_CAST_10A3: Variants = &[(0, L243)];
const V_CAST_10A4: Variants = &[(0, L227)];
const V_CAST_10A5: Variants = &[(0, L238)];
const V_CAST_10DA: Variants = &[(0, L244)];
const V_CAST_10DB: Variants = &[(0, L245)];
const V_CAST_118F: Variants = &[(0, L246)];
const V_CAST_1190: Variants = &[(0, L246)];
const V_CAST_11F4: Variants = &[(0, L247)];
const V_CAST_11F5: Variants = &[(0, L247)];
const V_CAST_120D: Variants = &[(0, L248)];
const V_CAST_120E: Variants = &[(0, L248)];
const V_CAST_1219: Variants = &[(0, L249), (2, L250)];
const V_CAST_121F: Variants = &[(0, L249), (2, L250)];
const V_CAST_1231: Variants = &[(0, L251)];
const V_CAST_1232: Variants = &[(0, L251)];
const V_CAST_1234: Variants = &[(0, L252)];
const V_CAST_1235: Variants = &[(0, L252)];
const V_CAST_1238: Variants = &[(0, L253)];
const V_CAST_1239: Variants = &[(0, L253)];
const V_CAST_1252: Variants = &[(0, L254), (8, L255)];
const V_CAST_1292: Variants = &[(0, L256)];
const V_CAST_1293: Variants = &[(0, L256)];
const V_CAST_1294: Variants = &[(0, L256)];
const V_CAST_1295: Variants = &[(0, L256)];
const V_CAST_1298: Variants = &[(0, L257)];
const V_CAST_1299: Variants = &[(0, L257)];
const V_PRESHOT_13: Variants = &[(0, L726)];
const V_PRESHOT_18: Variants = &[(0, L727)];
const V_PRESHOT_38: Variants = &[(0, L728)];
const V_PRESHOT_65: Variants = &[(0, L729)];
const V_PRESHOT_13A: Variants = &[(0, L730)];
const V_PRESHOT_157: Variants = &[(0, L731), (2, L731)];
const V_PRESHOT_162: Variants = &[(0, L732), (2, L732)];
const V_PRESHOT_171: Variants = &[(0, L733)];
const V_PRESHOT_FC8: Variants = &[(0, L734)];
const V_PRESHOT_1018: Variants = &[(0, L734)];
const V_PRESHOT_106F: Variants = &[(0, L734)];
const V_PRESHOT_1070: Variants = &[(0, L734)];
const V_PRESHOT_1084: Variants = &[(0, L735)];
const V_PRESHOT_1292: Variants = &[(0, L736)];
const V_PRESHOT_1293: Variants = &[(0, L736)];
const V_PRESHOT_1294: Variants = &[(0, L736)];
const V_PRESHOT_1295: Variants = &[(0, L736)];
const V_PRESHOT_1298: Variants = &[(0, L736)];
const V_PRESHOT_1299: Variants = &[(0, L736)];
const V_SHOT_2: Variants = &[(0, L258), (2, L258), (16, L259), (18, L259)];
const V_SHOT_3: Variants = &[(0, L254), (1, L260), (2, L254), (3, L260), (16, L254), (17, L261), (18, L254), (19, L261)];
const V_SHOT_4: Variants = &[(0, L262), (2, L262), (16, L263), (18, L263)];
const V_SHOT_5: Variants = &[(0, L254), (1, L264), (2, L254), (3, L264), (16, L254), (17, L265), (18, L254), (19, L265)];
const V_SHOT_6: Variants = &[(0, L254), (1, L266)];
const V_SHOT_7: Variants = &[(0, L254), (1, L267)];
const V_SHOT_8: Variants = &[(0, L262), (2, L262), (16, L263), (18, L263)];
const V_SHOT_9: Variants = &[(0, L268), (16, L269)];
const V_SHOT_B: Variants = &[(0, L258), (2, L258), (16, L259), (18, L259)];
const V_SHOT_C: Variants = &[(0, L258), (2, L258), (16, L259), (18, L259)];
const V_SHOT_F: Variants = &[(0, L258), (2, L258), (16, L259), (18, L259)];
const V_SHOT_11: Variants = &[(0, L254), (1, L270), (2, L254), (3, L270)];
const V_SHOT_12: Variants = &[(0, L258), (2, L258), (16, L259), (18, L259)];
const V_SHOT_15: Variants = &[(0, L271), (2, L271)];
const V_SHOT_1B: Variants = &[(0, L272), (2, L272), (16, L273), (18, L273)];
const V_SHOT_1C: Variants = &[(0, L258), (2, L258), (16, L259), (18, L259)];
const V_SHOT_1D: Variants = &[(0, L254), (1, L274), (2, L254), (3, L274)];
const V_SHOT_1E: Variants = &[(0, L254), (1, L260), (2, L254), (3, L260), (16, L254), (17, L261), (18, L254), (19, L261)];
const V_SHOT_22: Variants = &[(0, L275), (2, L275), (16, L276), (18, L276)];
const V_SHOT_23: Variants = &[(0, L254), (1, L277)];
const V_SHOT_24: Variants = &[(0, L254), (1, L260), (2, L254), (3, L260), (16, L254), (17, L261), (18, L254), (19, L261)];
const V_SHOT_2C: Variants = &[(0, L275), (2, L275), (16, L276), (18, L276)];
const V_SHOT_2D: Variants = &[(0, L275), (2, L275), (16, L276), (18, L276)];
const V_SHOT_2E: Variants = &[(0, L278), (2, L278)];
const V_SHOT_30: Variants = &[(0, L254), (1, L260), (2, L254), (3, L260), (16, L254), (17, L261), (18, L254), (19, L261)];
const V_SHOT_31: Variants = &[(0, L279), (2, L279), (16, L280), (18, L280)];
const V_SHOT_32: Variants = &[(0, L281), (2, L281), (16, L282), (18, L282)];
const V_SHOT_33: Variants = &[(0, L258), (2, L258), (16, L259), (18, L259)];
const V_SHOT_36: Variants = &[(0, L254), (1, L283)];
const V_SHOT_3A: Variants = &[(0, L284), (2, L284)];
const V_SHOT_3D: Variants = &[(0, L271), (2, L271)];
const V_SHOT_41: Variants = &[(0, L285), (2, L285), (16, L286), (18, L286)];
const V_SHOT_45: Variants = &[(0, L287), (2, L287), (16, L288), (18, L288)];
const V_SHOT_46: Variants = &[(0, L289), (2, L289)];
const V_SHOT_48: Variants = &[(0, L290), (2, L290), (16, L291), (18, L291)];
const V_SHOT_4B: Variants = &[(0, L292), (2, L292), (16, L293), (18, L293)];
const V_SHOT_4C: Variants = &[(0, L294), (2, L294), (16, L295), (18, L295)];
const V_SHOT_4D: Variants = &[(0, L294), (2, L294), (16, L295), (18, L295)];
const V_SHOT_4E: Variants = &[(0, L294), (2, L294), (16, L295), (18, L295)];
const V_SHOT_50: Variants = &[(0, L292), (2, L292), (16, L293), (18, L293)];
const V_SHOT_51: Variants = &[(0, L254), (1, L296), (2, L254), (3, L296)];
const V_SHOT_52: Variants = &[(0, L294), (2, L294), (16, L295), (18, L295)];
const V_SHOT_53: Variants = &[(0, L294), (2, L294), (16, L295), (18, L295)];
const V_SHOT_54: Variants = &[(0, L254), (1, L297), (16, L254), (17, L298)];
const V_SHOT_56: Variants = &[(0, L292), (2, L292), (16, L293), (18, L293)];
const V_SHOT_57: Variants = &[(0, L292), (2, L292), (16, L293), (18, L293)];
const V_SHOT_58: Variants = &[(0, L292), (2, L292), (16, L293), (18, L293)];
const V_SHOT_5B: Variants = &[(0, L294), (2, L294), (16, L295), (18, L295)];
const V_SHOT_5C: Variants = &[(0, L299), (2, L299), (16, L300), (18, L300)];
const V_SHOT_5D: Variants = &[(0, L299), (2, L299), (16, L300), (18, L300)];
const V_SHOT_5E: Variants = &[(0, L294), (2, L294), (16, L295), (18, L295)];
const V_SHOT_5F: Variants = &[(0, L254), (1, L301), (2, L254), (3, L301)];
const V_SHOT_60: Variants = &[(0, L302), (2, L302)];
const V_SHOT_61: Variants = &[(0, L303)];
const V_SHOT_62: Variants = &[(0, L304), (2, L304), (16, L305), (18, L305)];
const V_SHOT_63: Variants = &[(0, L292), (2, L292), (16, L293), (18, L293)];
const V_SHOT_64: Variants = &[(0, L254), (1, L306), (2, L254), (3, L306), (16, L254), (17, L307), (18, L254), (19, L307)];
const V_SHOT_65: Variants = &[(0, L308)];
const V_SHOT_66: Variants = &[(0, L309), (2, L309), (16, L310), (18, L310)];
const V_SHOT_67: Variants = &[(0, L311), (2, L311), (16, L312), (18, L312)];
const V_SHOT_68: Variants = &[(0, L292), (2, L292), (16, L293), (18, L293)];
const V_SHOT_69: Variants = &[(0, L313), (2, L313), (16, L314), (18, L314)];
const V_SHOT_6A: Variants = &[(0, L309), (2, L309), (16, L310), (18, L310)];
const V_SHOT_6B: Variants = &[(0, L309), (2, L309), (16, L310), (18, L310)];
const V_SHOT_6D: Variants = &[(0, L294), (2, L294), (16, L295), (18, L295)];
const V_SHOT_6E: Variants = &[(0, L290), (2, L290), (16, L291), (18, L291)];
const V_SHOT_6F: Variants = &[(0, L290), (2, L290), (16, L291), (18, L291)];
const V_SHOT_70: Variants = &[(0, L294), (2, L294), (16, L295), (18, L295)];
const V_SHOT_73: Variants = &[(0, L309), (2, L309), (16, L310), (18, L310)];
const V_SHOT_74: Variants = &[(0, L303)];
const V_SHOT_78: Variants = &[(0, L254), (1, L306), (2, L254), (3, L306), (16, L254), (17, L307), (18, L254), (19, L307)];
const V_SHOT_79: Variants = &[(0, L294), (2, L294), (16, L295), (18, L295)];
const V_SHOT_7A: Variants = &[(0, L309), (2, L309), (16, L310), (18, L310)];
const V_SHOT_7B: Variants = &[(0, L290), (2, L290), (16, L291), (18, L291)];
const V_SHOT_7F: Variants = &[(0, L309), (2, L309), (16, L310), (18, L310)];
const V_SHOT_81: Variants = &[(0, L315), (2, L315), (16, L316), (18, L316)];
const V_SHOT_82: Variants = &[(0, L292), (2, L292), (16, L293), (18, L293)];
const V_SHOT_83: Variants = &[(0, L292), (2, L292), (16, L293), (18, L293)];
const V_SHOT_8B: Variants = &[(0, L317), (2, L317), (16, L318), (18, L318)];
const V_SHOT_B0: Variants = &[(0, L319), (2, L319), (16, L320), (18, L320)];
const V_SHOT_B5: Variants = &[(0, L319), (2, L319), (16, L320), (18, L320)];
const V_SHOT_BE: Variants = &[(0, L254), (1, L321), (2, L254), (3, L321), (16, L254), (17, L322), (18, L254), (19, L322)];
const V_SHOT_DF: Variants = &[(0, L254), (1, L321), (2, L254), (3, L321), (16, L254), (17, L322), (18, L254), (19, L322)];
const V_SHOT_E6: Variants = &[(0, L323), (2, L323), (16, L324), (18, L324)];
const V_SHOT_F5: Variants = &[(0, L254), (1, L321), (2, L254), (3, L321), (16, L254), (17, L322), (18, L254), (19, L322)];
const V_SHOT_F6: Variants = &[(0, L325), (2, L325), (16, L326), (18, L326)];
const V_SHOT_FE: Variants = &[(0, L327), (2, L328), (16, L329), (18, L329)];
const V_SHOT_FF: Variants = &[(0, L254), (1, L321), (2, L254), (3, L321), (16, L254), (17, L322), (18, L254), (19, L322)];
const V_SHOT_104: Variants = &[(0, L254), (1, L321), (2, L254), (3, L321), (16, L254), (17, L322), (18, L254), (19, L322)];
const V_SHOT_105: Variants = &[(0, L254), (1, L330), (2, L254), (3, L330), (16, L254), (17, L331), (18, L254), (19, L331)];
const V_SHOT_106: Variants = &[(0, L332), (2, L332), (16, L333), (18, L333)];
const V_SHOT_108: Variants = &[(0, L334)];
const V_SHOT_109: Variants = &[(0, L334)];
const V_SHOT_10A: Variants = &[(0, L334)];
const V_SHOT_10B: Variants = &[(0, L334)];
const V_SHOT_10C: Variants = &[(0, L334)];
const V_SHOT_10D: Variants = &[(0, L334)];
const V_SHOT_10E: Variants = &[(0, L334)];
const V_SHOT_10F: Variants = &[(0, L335)];
const V_SHOT_110: Variants = &[(0, L335)];
const V_SHOT_111: Variants = &[(0, L335)];
const V_SHOT_112: Variants = &[(0, L335)];
const V_SHOT_113: Variants = &[(0, L335)];
const V_SHOT_114: Variants = &[(0, L335)];
const V_SHOT_115: Variants = &[(0, L335)];
const V_SHOT_117: Variants = &[(0, L336), (2, L336), (16, L337), (18, L337)];
const V_SHOT_118: Variants = &[(0, L254), (1, L338)];
const V_SHOT_119: Variants = &[(0, L254), (1, L321), (2, L254), (3, L321), (16, L254), (17, L322), (18, L254), (19, L322)];
const V_SHOT_11A: Variants = &[(0, L319), (2, L319), (16, L320), (18, L320)];
const V_SHOT_11C: Variants = &[(0, L339), (2, L339), (16, L340), (18, L340)];
const V_SHOT_11E: Variants = &[(0, L341), (2, L341), (16, L342), (18, L342)];
const V_SHOT_11F: Variants = &[(0, L343), (2, L343), (16, L344), (18, L344)];
const V_SHOT_121: Variants = &[(0, L345), (2, L345)];
const V_SHOT_124: Variants = &[(0, L346), (2, L346), (16, L347), (18, L347)];
const V_SHOT_129: Variants = &[(0, L348), (2, L348), (16, L349), (18, L349)];
const V_SHOT_12A: Variants = &[(0, L346), (2, L346), (16, L347), (18, L347)];
const V_SHOT_12E: Variants = &[(0, L350), (2, L351), (16, L352), (18, L352)];
const V_SHOT_12F: Variants = &[(0, L346), (2, L346), (16, L347), (18, L347)];
const V_SHOT_130: Variants = &[(0, L353)];
const V_SHOT_131: Variants = &[(0, L353)];
const V_SHOT_132: Variants = &[(0, L353)];
const V_SHOT_133: Variants = &[(0, L354)];
const V_SHOT_134: Variants = &[(0, L353)];
const V_SHOT_135: Variants = &[(0, L354)];
const V_SHOT_136: Variants = &[(0, L354)];
const V_SHOT_137: Variants = &[(0, L354)];
const V_SHOT_157: Variants = &[(0, L355)];
const V_SHOT_162: Variants = &[(0, L356)];
const V_SHOT_16A: Variants = &[(0, L357), (1, L357), (2, L358), (3, L358), (4, L357), (5, L357), (6, L358), (7, L358), (8, L357), (9, L357), (10, L358), (11, L358), (12, L357), (13, L357), (14, L358), (15, L358), (16, L357), (17, L357), (18, L358), (19, L358), (20, L357), (21, L357), (22, L358), (23, L358), (24, L357), (25, L357), (26, L358), (27, L358), (28, L357), (29, L357), (30, L358), (31, L358)];
const V_SHOT_3EA: Variants = &[(0, L359), (2, L360), (16, L361), (18, L362)];
const V_SHOT_3EB: Variants = &[(0, L359), (2, L360), (16, L361), (18, L362)];
const V_SHOT_3EC: Variants = &[(0, L359), (2, L360), (16, L361), (18, L362)];
const V_SHOT_3ED: Variants = &[(0, L363), (2, L364), (16, L365), (18, L366)];
const V_SHOT_3EE: Variants = &[(0, L363), (2, L364), (16, L365), (18, L366)];
const V_SHOT_3EF: Variants = &[(0, L367), (2, L368), (16, L369), (18, L370)];
const V_SHOT_3F0: Variants = &[(0, L371), (2, L372), (16, L373), (18, L374)];
const V_SHOT_3F1: Variants = &[(0, L371), (2, L372), (16, L373), (18, L374)];
const V_SHOT_3F2: Variants = &[(0, L371), (2, L372), (16, L373), (18, L374)];
const V_SHOT_3F3: Variants = &[(0, L375), (2, L375), (16, L376), (18, L376)];
const V_SHOT_3F4: Variants = &[(0, L377), (2, L377)];
const V_SHOT_3F5: Variants = &[(0, L377), (2, L377)];
const V_SHOT_3F7: Variants = &[(0, L378), (2, L378)];
const V_SHOT_3F8: Variants = &[(0, L379), (2, L379), (16, L380), (18, L380)];
const V_SHOT_3FA: Variants = &[(0, L381), (2, L381)];
const V_SHOT_3FC: Variants = &[(0, L382), (2, L382), (16, L383), (18, L383)];
const V_SHOT_403: Variants = &[(0, L254), (16, L384)];
const V_SHOT_404: Variants = &[(0, L385), (2, L385), (16, L386), (18, L386)];
const V_SHOT_407: Variants = &[(0, L385), (2, L385), (16, L386), (18, L386)];
const V_SHOT_408: Variants = &[(0, L387), (2, L387), (16, L388), (18, L388)];
const V_SHOT_409: Variants = &[(0, L387), (2, L387), (16, L388), (18, L388)];
const V_SHOT_40A: Variants = &[(0, L389)];
const V_SHOT_40B: Variants = &[(0, L387), (2, L387), (16, L388), (18, L388)];
const V_SHOT_40C: Variants = &[(0, L387), (2, L387), (16, L388), (18, L388)];
const V_SHOT_410: Variants = &[(0, L387), (2, L387), (16, L388), (18, L388)];
const V_SHOT_412: Variants = &[(0, L385), (2, L385), (16, L386), (18, L386)];
const V_SHOT_413: Variants = &[(0, L390), (2, L390), (16, L391), (18, L391)];
const V_SHOT_414: Variants = &[(0, L390), (2, L390), (16, L391), (18, L391)];
const V_SHOT_415: Variants = &[(0, L390), (2, L390), (16, L391), (18, L391)];
const V_SHOT_417: Variants = &[(0, L390), (2, L390), (16, L391), (18, L391)];
const V_SHOT_418: Variants = &[(0, L390), (2, L390), (16, L391), (18, L391)];
const V_SHOT_419: Variants = &[(0, L389)];
const V_SHOT_41A: Variants = &[(0, L392)];
const V_SHOT_420: Variants = &[(0, L393), (2, L393), (16, L394), (18, L394)];
const V_SHOT_423: Variants = &[(0, L393), (2, L393), (16, L394), (18, L394)];
const V_SHOT_426: Variants = &[(0, L395), (2, L395), (16, L396), (18, L396)];
const V_SHOT_428: Variants = &[(0, L395), (2, L395), (16, L396), (18, L396)];
const V_SHOT_429: Variants = &[(0, L393), (2, L393), (16, L394), (18, L394)];
const V_SHOT_42C: Variants = &[(0, L397), (2, L397), (16, L398), (18, L398)];
const V_SHOT_42D: Variants = &[(0, L395), (2, L395), (16, L396), (18, L396)];
const V_SHOT_42F: Variants = &[(0, L395), (2, L395), (16, L396), (18, L396)];
const V_SHOT_430: Variants = &[(0, L395), (2, L395), (16, L396), (18, L396)];
const V_SHOT_431: Variants = &[(0, L393), (2, L393), (16, L394), (18, L394)];
const V_SHOT_432: Variants = &[(0, L395), (2, L395), (16, L396), (18, L396)];
const V_SHOT_433: Variants = &[(0, L395), (2, L395), (16, L396), (18, L396)];
const V_SHOT_435: Variants = &[(0, L399), (2, L399), (16, L400), (18, L400)];
const V_SHOT_436: Variants = &[(0, L397), (2, L397), (16, L398), (18, L398)];
const V_SHOT_43B: Variants = &[(0, L395), (2, L395), (16, L396), (18, L396)];
const V_SHOT_43D: Variants = &[(0, L393), (2, L393), (16, L394), (18, L394)];
const V_SHOT_43E: Variants = &[(0, L399), (2, L399), (16, L400), (18, L400)];
const V_SHOT_43F: Variants = &[(0, L397), (2, L397), (16, L398), (18, L398)];
const V_SHOT_442: Variants = &[(0, L401), (2, L401)];
const V_SHOT_444: Variants = &[(0, L402), (2, L403), (16, L404), (18, L405)];
const V_SHOT_447: Variants = &[(0, L402), (2, L403), (16, L404), (18, L405)];
const V_SHOT_448: Variants = &[(0, L402), (2, L403), (16, L404), (18, L405)];
const V_SHOT_449: Variants = &[(0, L402), (2, L403), (16, L404), (18, L405)];
const V_SHOT_44B: Variants = &[(0, L402), (2, L403), (16, L404), (18, L405)];
const V_SHOT_44C: Variants = &[(0, L406), (2, L406)];
const V_SHOT_44D: Variants = &[(0, L407), (2, L407), (16, L408), (18, L408)];
const V_SHOT_44E: Variants = &[(0, L402), (2, L403), (16, L404), (18, L405)];
const V_SHOT_450: Variants = &[(0, L402), (2, L403), (16, L404), (18, L405)];
const V_SHOT_451: Variants = &[(0, L402), (2, L403), (16, L404), (18, L405)];
const V_SHOT_453: Variants = &[(0, L406), (2, L406)];
const V_SHOT_454: Variants = &[(0, L407), (2, L407), (16, L408), (18, L408)];
const V_SHOT_466: Variants = &[(0, L409), (2, L409), (16, L410), (18, L410)];
const V_SHOT_467: Variants = &[(0, L409), (2, L409), (16, L410), (18, L410)];
const V_SHOT_472: Variants = &[(0, L393), (2, L393), (16, L394), (18, L394)];
const V_SHOT_473: Variants = &[(0, L393), (2, L393), (16, L394), (18, L394)];
const V_SHOT_474: Variants = &[(0, L393), (2, L393), (16, L394), (18, L394)];
const V_SHOT_475: Variants = &[(0, L393), (2, L393), (16, L394), (18, L394)];
const V_SHOT_478: Variants = &[(0, L393), (2, L393), (16, L394), (18, L394)];
const V_SHOT_479: Variants = &[(0, L393), (2, L393), (16, L394), (18, L394)];
const V_SHOT_47A: Variants = &[(0, L393), (2, L393), (16, L394), (18, L394)];
const V_SHOT_47B: Variants = &[(0, L401), (2, L401)];
const V_SHOT_47C: Variants = &[(0, L254), (1, L411)];
const V_SHOT_47F: Variants = &[(0, L412), (2, L412)];
const V_SHOT_483: Variants = &[(0, L413), (2, L413), (16, L413), (18, L414)];
const V_SHOT_484: Variants = &[(0, L415), (2, L415), (16, L416), (18, L416)];
const V_SHOT_485: Variants = &[(0, L417), (2, L417), (16, L418), (18, L418)];
const V_SHOT_487: Variants = &[(0, L254), (1, L419)];
const V_SHOT_488: Variants = &[(0, L415), (2, L415), (16, L416), (18, L416)];
const V_SHOT_48B: Variants = &[(0, L415), (2, L415), (16, L416), (18, L416)];
const V_SHOT_48C: Variants = &[(0, L420), (2, L420), (16, L421), (18, L421)];
const V_SHOT_48F: Variants = &[(0, L422), (2, L422), (16, L423), (18, L423)];
const V_SHOT_490: Variants = &[(0, L415), (2, L415), (16, L416), (18, L416)];
const V_SHOT_491: Variants = &[(0, L415), (2, L415), (16, L416), (18, L416)];
const V_SHOT_492: Variants = &[(0, L415), (2, L415), (16, L416), (18, L416)];
const V_SHOT_493: Variants = &[(0, L424), (16, L425)];
const V_SHOT_494: Variants = &[(0, L426), (2, L426), (16, L427), (18, L427)];
const V_SHOT_496: Variants = &[(0, L428), (2, L428), (16, L429), (18, L429)];
const V_SHOT_497: Variants = &[(0, L430)];
const V_SHOT_498: Variants = &[(0, L431)];
const V_SHOT_499: Variants = &[(0, L254), (1, L432)];
const V_SHOT_49A: Variants = &[(0, L433)];
const V_SHOT_49D: Variants = &[(0, L254), (1, L419)];
const V_SHOT_49E: Variants = &[(0, L417), (2, L417), (16, L418), (18, L418)];
const V_SHOT_49F: Variants = &[(0, L434), (2, L434), (16, L435), (18, L435)];
const V_SHOT_4A0: Variants = &[(0, L428), (2, L428), (16, L429), (18, L429)];
const V_SHOT_4A5: Variants = &[(0, L436), (2, L436), (16, L437), (18, L437)];
const V_SHOT_4A7: Variants = &[(0, L417), (2, L417), (16, L418), (18, L418)];
const V_SHOT_4B1: Variants = &[(0, L438), (2, L438), (16, L439), (18, L439)];
const V_SHOT_4B4: Variants = &[(0, L436), (2, L436), (16, L437), (18, L437)];
const V_SHOT_4B6: Variants = &[(0, L440), (2, L440), (16, L441), (18, L441)];
const V_SHOT_4B8: Variants = &[(0, L442), (2, L443), (16, L444), (18, L445)];
const V_SHOT_4B9: Variants = &[(0, L442), (2, L443), (16, L444), (18, L445)];
const V_SHOT_4BA: Variants = &[(0, L442), (2, L443), (16, L444), (18, L445)];
const V_SHOT_4BD: Variants = &[(0, L442), (2, L443), (16, L444), (18, L445)];
const V_SHOT_4C0: Variants = &[(0, L446), (2, L446), (16, L447), (18, L447)];
const V_SHOT_4C1: Variants = &[(0, L448), (2, L448), (16, L449), (18, L449)];
const V_SHOT_4C2: Variants = &[(0, L448), (2, L448), (16, L449), (18, L449)];
const V_SHOT_4C3: Variants = &[(0, L448), (2, L448), (16, L449), (18, L449)];
const V_SHOT_4C4: Variants = &[(0, L254), (1, L419)];
const V_SHOT_4C6: Variants = &[(0, L420), (2, L420), (16, L421), (18, L421)];
const V_SHOT_4C7: Variants = &[(0, L450), (2, L450), (16, L451), (18, L451)];
const V_SHOT_4C8: Variants = &[(0, L450), (2, L450), (16, L451), (18, L451)];
const V_SHOT_4CD: Variants = &[(0, L452), (2, L453), (16, L454), (18, L455)];
const V_SHOT_4CE: Variants = &[(0, L254), (1, L419)];
const V_SHOT_4CF: Variants = &[(0, L456), (2, L456), (16, L457), (18, L457)];
const V_SHOT_4D0: Variants = &[(0, L458), (2, L458), (16, L459), (18, L459)];
const V_SHOT_4D1: Variants = &[(0, L438), (2, L438), (16, L439), (18, L439)];
const V_SHOT_4D2: Variants = &[(0, L412), (2, L412)];
const V_SHOT_4D3: Variants = &[(0, L434), (2, L434), (16, L435), (18, L435)];
const V_SHOT_4D4: Variants = &[(0, L428), (2, L428), (16, L429), (18, L429)];
const V_SHOT_4D5: Variants = &[(0, L460)];
const V_SHOT_4D6: Variants = &[(0, L458), (2, L458), (16, L459), (18, L459)];
const V_SHOT_4D7: Variants = &[(0, L431)];
const V_SHOT_4D8: Variants = &[(0, L461), (2, L461), (16, L462), (18, L462)];
const V_SHOT_4D9: Variants = &[(0, L463), (2, L463), (16, L464), (18, L464)];
const V_SHOT_4DA: Variants = &[(0, L465), (2, L465), (16, L466), (18, L466)];
const V_SHOT_4DC: Variants = &[(0, L467), (2, L467)];
const V_SHOT_4DD: Variants = &[(0, L468), (2, L468)];
const V_SHOT_4DE: Variants = &[(0, L469), (2, L470), (16, L471), (18, L472)];
const V_SHOT_4DF: Variants = &[(0, L469), (2, L470), (16, L471), (18, L472)];
const V_SHOT_4E0: Variants = &[(0, L469), (2, L470), (16, L471), (18, L472)];
const V_SHOT_4E1: Variants = &[(0, L473), (2, L474), (16, L475), (18, L476)];
const V_SHOT_4E2: Variants = &[(0, L477), (2, L478), (16, L479), (18, L480)];
const V_SHOT_4E3: Variants = &[(0, L473), (2, L474), (16, L475), (18, L476)];
const V_SHOT_4E4: Variants = &[(0, L477), (2, L478), (16, L479), (18, L480)];
const V_SHOT_4E5: Variants = &[(0, L473), (2, L474), (16, L475), (18, L476)];
const V_SHOT_4E6: Variants = &[(0, L481), (2, L481), (16, L482), (18, L482)];
const V_SHOT_4E7: Variants = &[(0, L254), (16, L483)];
const V_SHOT_4E8: Variants = &[(0, L473), (2, L474), (16, L475), (18, L476)];
const V_SHOT_4E9: Variants = &[(0, L484), (2, L484), (16, L485), (18, L485)];
const V_SHOT_4EA: Variants = &[(0, L486), (2, L486), (16, L487), (18, L487)];
const V_SHOT_4EB: Variants = &[(0, L488), (2, L488), (16, L489), (18, L489)];
const V_SHOT_4EC: Variants = &[(0, L477), (2, L478), (16, L479), (18, L480)];
const V_SHOT_4ED: Variants = &[(0, L477), (2, L478), (16, L479), (18, L480)];
const V_SHOT_4EF: Variants = &[(0, L490), (2, L490), (16, L491), (18, L491)];
const V_SHOT_4F0: Variants = &[(0, L492), (2, L492), (16, L493), (18, L493)];
const V_SHOT_4F1: Variants = &[(0, L492), (2, L492), (16, L493), (18, L493)];
const V_SHOT_4F2: Variants = &[(0, L254), (1, L494)];
const V_SHOT_4F3: Variants = &[(0, L254), (1, L494)];
const V_SHOT_4F4: Variants = &[(0, L465), (2, L465), (16, L466), (18, L466)];
const V_SHOT_4F5: Variants = &[(0, L490), (2, L490), (16, L491), (18, L491)];
const V_SHOT_4F7: Variants = &[(0, L486), (2, L486), (16, L487), (18, L487)];
const V_SHOT_4F8: Variants = &[(0, L495), (2, L495), (16, L496), (18, L496)];
const V_SHOT_4F9: Variants = &[(0, L497), (2, L497), (16, L498), (18, L498)];
const V_SHOT_4FA: Variants = &[(0, L499), (2, L499), (16, L500), (18, L500)];
const V_SHOT_4FB: Variants = &[(0, L501), (2, L501), (16, L502), (18, L502)];
const V_SHOT_502: Variants = &[(0, L473), (2, L474), (16, L475), (18, L476)];
const V_SHOT_504: Variants = &[(0, L477), (2, L478), (16, L479), (18, L480)];
const V_SHOT_505: Variants = &[(0, L503), (2, L503)];
const V_SHOT_506: Variants = &[(0, L503), (2, L503)];
const V_SHOT_507: Variants = &[(0, L503), (2, L503)];
const V_SHOT_508: Variants = &[(0, L504), (2, L504), (16, L505), (18, L505)];
const V_SHOT_509: Variants = &[(0, L254), (1, L494)];
const V_SHOT_50A: Variants = &[(0, L506)];
const V_SHOT_50B: Variants = &[(0, L507)];
const V_SHOT_50C: Variants = &[(0, L504), (2, L504), (16, L505), (18, L505)];
const V_SHOT_50D: Variants = &[(0, L504), (2, L504), (16, L505), (18, L505)];
const V_SHOT_50E: Variants = &[(0, L504), (2, L504), (16, L505), (18, L505)];
const V_SHOT_552: Variants = &[(0, L508), (2, L509), (16, L510), (18, L511)];
const V_SHOT_553: Variants = &[(0, L508), (2, L509), (16, L510), (18, L511)];
const V_SHOT_554: Variants = &[(0, L512), (2, L513), (16, L514), (18, L515)];
const V_SHOT_555: Variants = &[(0, L512), (2, L513), (16, L514), (18, L515)];
const V_SHOT_556: Variants = &[(0, L516), (2, L517), (16, L518), (18, L519)];
const V_SHOT_557: Variants = &[(0, L516), (2, L517), (16, L518), (18, L519)];
const V_SHOT_7DD: Variants = &[(0, L520)];
const V_SHOT_7DE: Variants = &[(0, L521), (2, L521), (16, L522), (18, L522)];
const V_SHOT_7F8: Variants = &[(0, L523)];
const V_SHOT_7F9: Variants = &[(0, L523)];
const V_SHOT_801: Variants = &[(0, L521), (2, L521), (16, L522), (18, L522)];
const V_SHOT_802: Variants = &[(0, L524), (2, L524), (16, L525), (18, L525)];
const V_SHOT_803: Variants = &[(0, L524), (2, L524), (16, L525), (18, L525)];
const V_SHOT_804: Variants = &[(0, L524), (2, L524), (16, L525), (18, L525)];
const V_SHOT_805: Variants = &[(0, L526), (2, L526), (16, L527), (18, L527)];
const V_SHOT_806: Variants = &[(0, L528), (2, L528), (16, L529), (18, L529)];
const V_SHOT_807: Variants = &[(0, L530), (2, L530), (16, L531), (18, L531)];
const V_SHOT_808: Variants = &[(0, L526), (2, L526), (16, L527), (18, L527)];
const V_SHOT_809: Variants = &[(0, L530), (2, L530), (16, L531), (18, L531)];
const V_SHOT_80A: Variants = &[(0, L532), (2, L532), (16, L533), (18, L533)];
const V_SHOT_80B: Variants = &[(0, L534), (2, L534), (16, L535), (18, L535)];
const V_SHOT_80E: Variants = &[(0, L536), (2, L536), (16, L537), (18, L537)];
const V_SHOT_810: Variants = &[(0, L538), (2, L538), (16, L539), (18, L539)];
const V_SHOT_830: Variants = &[(0, L540), (2, L540)];
const V_SHOT_831: Variants = &[(0, L541), (2, L542), (16, L543), (18, L543)];
const V_SHOT_833: Variants = &[(0, L254), (16, L544)];
const V_SHOT_834: Variants = &[(0, L254), (16, L544)];
const V_SHOT_FA1: Variants = &[(0, L254), (1, L545)];
const V_SHOT_FA2: Variants = &[(0, L546), (2, L546)];
const V_SHOT_FB2: Variants = &[(0, L547)];
const V_SHOT_FB3: Variants = &[(0, L548), (2, L548), (16, L549), (18, L549)];
const V_SHOT_FB4: Variants = &[(0, L254), (16, L550)];
const V_SHOT_FB8: Variants = &[(0, L254), (16, L550)];
const V_SHOT_FB9: Variants = &[(0, L551), (2, L551)];
const V_SHOT_FBA: Variants = &[(0, L254), (1, L552)];
const V_SHOT_FBB: Variants = &[(0, L553), (2, L553), (16, L554), (18, L554)];
const V_SHOT_FBC: Variants = &[(0, L555), (2, L555), (16, L556), (18, L556)];
const V_SHOT_FBD: Variants = &[(0, L555), (2, L555), (16, L556), (18, L556)];
const V_SHOT_FBE: Variants = &[(0, L557), (2, L557), (16, L558), (18, L558)];
const V_SHOT_FBF: Variants = &[(0, L559), (2, L559), (16, L560), (18, L560)];
const V_SHOT_FC0: Variants = &[(0, L254), (1, L561), (2, L254), (3, L561), (16, L254), (17, L562), (18, L254), (19, L562)];
const V_SHOT_FC1: Variants = &[(0, L563), (2, L563), (16, L564), (18, L564)];
const V_SHOT_FC2: Variants = &[(0, L565), (2, L565), (16, L566), (18, L566)];
const V_SHOT_FC3: Variants = &[(0, L567), (2, L567), (16, L568), (18, L568)];
const V_SHOT_FC4: Variants = &[(0, L569), (2, L569), (16, L570), (18, L570)];
const V_SHOT_FC5: Variants = &[(0, L571), (2, L571), (16, L572), (18, L572)];
const V_SHOT_FC6: Variants = &[(0, L573), (2, L573), (16, L574), (18, L574)];
const V_SHOT_FC7: Variants = &[(0, L575), (2, L575)];
const V_SHOT_FCA: Variants = &[(0, L254), (1, L576)];
const V_SHOT_FCB: Variants = &[(0, L254), (1, L576)];
const V_SHOT_FCC: Variants = &[(0, L577), (2, L577), (16, L578), (18, L578)];
const V_SHOT_FCE: Variants = &[(0, L579), (2, L579), (16, L580), (18, L580)];
const V_SHOT_FCF: Variants = &[(0, L581), (2, L581), (16, L582), (18, L582)];
const V_SHOT_FDF: Variants = &[(0, L254), (1, L583), (2, L254), (3, L583), (16, L254), (17, L584), (18, L254), (19, L584)];
const V_SHOT_FE0: Variants = &[(0, L254), (1, L583), (2, L254), (3, L583), (16, L254), (17, L584), (18, L254), (19, L584)];
const V_SHOT_FE1: Variants = &[(0, L577), (2, L577), (16, L578), (18, L578)];
const V_SHOT_FE2: Variants = &[(0, L585)];
const V_SHOT_FE3: Variants = &[(0, L254), (1, L583), (2, L254), (3, L583), (16, L254), (17, L584), (18, L254), (19, L584)];
const V_SHOT_FE4: Variants = &[(0, L586)];
const V_SHOT_FE5: Variants = &[(0, L587)];
const V_SHOT_FE8: Variants = &[(0, L588), (2, L588), (16, L589), (18, L589)];
const V_SHOT_FE9: Variants = &[(0, L588), (2, L588), (16, L589), (18, L589)];
const V_SHOT_FEA: Variants = &[(0, L590), (2, L590), (16, L591), (18, L591)];
const V_SHOT_FEB: Variants = &[(0, L588), (2, L588), (16, L589), (18, L589)];
const V_SHOT_FEC: Variants = &[(0, L592), (2, L592), (16, L593), (18, L593)];
const V_SHOT_FED: Variants = &[(0, L594), (2, L594), (16, L595), (18, L595)];
const V_SHOT_FEE: Variants = &[(0, L254), (1, L576)];
const V_SHOT_FEF: Variants = &[(0, L596), (2, L596)];
const V_SHOT_FF1: Variants = &[(0, L597), (2, L597)];
const V_SHOT_FF2: Variants = &[(0, L598), (2, L598), (16, L599), (18, L599)];
const V_SHOT_FF3: Variants = &[(0, L254), (1, L600), (2, L254), (3, L600), (16, L254), (17, L601), (18, L254), (19, L601)];
const V_SHOT_FF7: Variants = &[(0, L254), (1, L602)];
const V_SHOT_FF9: Variants = &[(0, L603), (2, L603), (16, L604), (18, L604)];
const V_SHOT_FFA: Variants = &[(0, L603), (2, L603), (16, L604), (18, L604)];
const V_SHOT_FFB: Variants = &[(0, L603), (2, L603), (16, L604), (18, L604)];
const V_SHOT_FFC: Variants = &[(0, L603), (2, L603), (16, L604), (18, L604)];
const V_SHOT_FFE: Variants = &[(0, L605), (2, L605), (16, L606), (18, L606)];
const V_SHOT_1000: Variants = &[(0, L603), (2, L603), (16, L604), (18, L604)];
const V_SHOT_1001: Variants = &[(0, L607), (2, L608), (16, L609), (18, L610)];
const V_SHOT_1002: Variants = &[(0, L611), (2, L611), (16, L612), (18, L612)];
const V_SHOT_1003: Variants = &[(0, L611), (2, L611), (16, L612), (18, L612)];
const V_SHOT_1004: Variants = &[(0, L254), (1, L602)];
const V_SHOT_1006: Variants = &[(0, L611), (2, L611), (16, L612), (18, L612)];
const V_SHOT_1007: Variants = &[(0, L603), (2, L603), (16, L604), (18, L604)];
const V_SHOT_1008: Variants = &[(0, L613), (2, L613), (16, L614), (18, L614)];
const V_SHOT_1009: Variants = &[(0, L615)];
const V_SHOT_100A: Variants = &[(0, L616), (2, L616)];
const V_SHOT_100B: Variants = &[(0, L617), (2, L617)];
const V_SHOT_100D: Variants = &[(0, L618), (2, L619)];
const V_SHOT_100E: Variants = &[(0, L620), (2, L620)];
const V_SHOT_1012: Variants = &[(0, L254), (1, L602)];
const V_SHOT_1013: Variants = &[(0, L621), (2, L621), (16, L622), (18, L622)];
const V_SHOT_1015: Variants = &[(0, L623), (2, L623), (16, L624), (18, L624)];
const V_SHOT_1016: Variants = &[(0, L625), (2, L625), (16, L626), (18, L626)];
const V_SHOT_1017: Variants = &[(0, L623), (2, L623), (16, L624), (18, L624)];
const V_SHOT_101B: Variants = &[(0, L611), (2, L611), (16, L612), (18, L612)];
const V_SHOT_101C: Variants = &[(0, L627)];
const V_SHOT_101F: Variants = &[(0, L628), (2, L628)];
const V_SHOT_1021: Variants = &[(0, L629), (2, L629)];
const V_SHOT_1022: Variants = &[(0, L630), (2, L630)];
const V_SHOT_1023: Variants = &[(0, L631), (2, L631)];
const V_SHOT_1024: Variants = &[(0, L632), (2, L632), (16, L632), (18, L633)];
const V_SHOT_1025: Variants = &[(0, L621), (2, L621), (16, L622), (18, L622)];
const V_SHOT_1029: Variants = &[(0, L634), (2, L634), (16, L635), (18, L635)];
const V_SHOT_102A: Variants = &[(0, L632), (2, L632), (16, L632), (18, L633)];
const V_SHOT_102B: Variants = &[(0, L636), (2, L636), (16, L637), (18, L637)];
const V_SHOT_1030: Variants = &[(0, L254), (1, L600), (2, L254), (3, L600), (16, L254), (17, L601), (18, L254), (19, L601)];
const V_SHOT_1037: Variants = &[(0, L254), (1, L638)];
const V_SHOT_1038: Variants = &[(0, L639), (2, L639)];
const V_SHOT_1039: Variants = &[(0, L640), (2, L640), (16, L641), (18, L641)];
const V_SHOT_103A: Variants = &[(0, L642), (2, L642)];
const V_SHOT_103B: Variants = &[(0, L643)];
const V_SHOT_103C: Variants = &[(0, L644)];
const V_SHOT_103D: Variants = &[(0, L254), (1, L645)];
const V_SHOT_103E: Variants = &[(0, L254), (1, L645)];
const V_SHOT_103F: Variants = &[(0, L646)];
const V_SHOT_1040: Variants = &[(0, L647), (2, L647), (16, L648), (18, L648)];
const V_SHOT_1043: Variants = &[(0, L649), (2, L649), (16, L650), (18, L650)];
const V_SHOT_1048: Variants = &[(0, L254), (1, L651), (2, L254), (3, L651), (16, L254), (17, L652), (18, L254), (19, L652)];
const V_SHOT_1049: Variants = &[(0, L254), (1, L651), (2, L254), (3, L651), (16, L254), (17, L652), (18, L254), (19, L652)];
const V_SHOT_104A: Variants = &[(0, L254), (1, L651), (2, L254), (3, L651), (16, L254), (17, L652), (18, L254), (19, L652)];
const V_SHOT_104B: Variants = &[(0, L254), (1, L651), (2, L254), (3, L651), (16, L254), (17, L652), (18, L254), (19, L652)];
const V_SHOT_104C: Variants = &[(0, L254), (1, L651), (2, L254), (3, L651), (16, L254), (17, L652), (18, L254), (19, L652)];
const V_SHOT_104D: Variants = &[(0, L653), (2, L653), (16, L654), (18, L654)];
const V_SHOT_104E: Variants = &[(0, L655), (2, L655), (16, L656), (18, L656)];
const V_SHOT_104F: Variants = &[(0, L657), (2, L657), (16, L658), (18, L658)];
const V_SHOT_1050: Variants = &[(0, L659), (2, L659), (16, L660), (18, L660)];
const V_SHOT_1051: Variants = &[(0, L661), (2, L661), (16, L662), (18, L662)];
const V_SHOT_1052: Variants = &[(0, L254), (1, L663)];
const V_SHOT_1053: Variants = &[(0, L254), (1, L664), (2, L254), (3, L664), (16, L254), (17, L665), (18, L254), (19, L665)];
const V_SHOT_1054: Variants = &[(0, L254), (1, L664), (2, L254), (3, L664), (16, L254), (17, L665), (18, L254), (19, L665)];
const V_SHOT_1055: Variants = &[(0, L254), (1, L664), (2, L254), (3, L664), (16, L254), (17, L665), (18, L254), (19, L665)];
const V_SHOT_1056: Variants = &[(0, L666), (2, L666), (16, L667), (18, L667)];
const V_SHOT_1057: Variants = &[(0, L666), (2, L666), (16, L667), (18, L667)];
const V_SHOT_1058: Variants = &[(0, L668), (2, L668), (16, L669), (18, L669)];
const V_SHOT_1059: Variants = &[(0, L670), (2, L670), (16, L671), (18, L671)];
const V_SHOT_105A: Variants = &[(0, L672), (2, L672), (16, L673), (18, L673)];
const V_SHOT_105B: Variants = &[(0, L666), (2, L666), (16, L667), (18, L667)];
const V_SHOT_105C: Variants = &[(0, L674), (2, L674), (16, L675), (18, L675)];
const V_SHOT_105D: Variants = &[(0, L666), (2, L666), (16, L667), (18, L667)];
const V_SHOT_105E: Variants = &[(0, L666), (2, L666), (16, L667), (18, L667)];
const V_SHOT_105F: Variants = &[(0, L254), (1, L676)];
const V_SHOT_1060: Variants = &[(0, L677), (2, L677)];
const V_SHOT_1061: Variants = &[(0, L677), (2, L677)];
const V_SHOT_1062: Variants = &[(0, L678), (2, L678), (16, L679), (18, L679)];
const V_SHOT_1063: Variants = &[(0, L680)];
const V_SHOT_1064: Variants = &[(0, L681), (2, L681), (16, L682), (18, L682)];
const V_SHOT_1065: Variants = &[(0, L672), (2, L672), (16, L673), (18, L673)];
const V_SHOT_1066: Variants = &[(0, L683), (2, L683), (16, L684), (18, L684)];
const V_SHOT_1067: Variants = &[(0, L683), (2, L683), (16, L684), (18, L684)];
const V_SHOT_1068: Variants = &[(0, L668), (2, L668), (16, L669), (18, L669)];
const V_SHOT_1069: Variants = &[(0, L670), (2, L670), (16, L671), (18, L671)];
const V_SHOT_106A: Variants = &[(0, L672), (2, L672), (16, L673), (18, L673)];
const V_SHOT_106B: Variants = &[(0, L683), (2, L683), (16, L684), (18, L684)];
const V_SHOT_106C: Variants = &[(0, L685)];
const V_SHOT_106D: Variants = &[(0, L683), (2, L683), (16, L684), (18, L684)];
const V_SHOT_106E: Variants = &[(0, L683), (2, L683), (16, L684), (18, L684)];
const V_SHOT_1070: Variants = &[(0, L686)];
const V_SHOT_1071: Variants = &[(0, L687), (2, L687), (16, L688), (18, L688)];
const V_SHOT_1072: Variants = &[(0, L689), (2, L690), (16, L691), (18, L692)];
const V_SHOT_1073: Variants = &[(0, L653), (2, L653), (16, L654), (18, L654)];
const V_SHOT_1074: Variants = &[(0, L655), (2, L655), (16, L656), (18, L656)];
const V_SHOT_1075: Variants = &[(0, L657), (2, L657), (16, L658), (18, L658)];
const V_SHOT_1076: Variants = &[(0, L657), (2, L657), (16, L658), (18, L658)];
const V_SHOT_1078: Variants = &[(0, L254), (16, L693)];
const V_SHOT_1079: Variants = &[(0, L254), (16, L693)];
const V_SHOT_107A: Variants = &[(0, L677), (2, L677)];
const V_SHOT_107B: Variants = &[(0, L672), (2, L672), (16, L673), (18, L673)];
const V_SHOT_107C: Variants = &[(0, L694), (2, L694), (16, L695), (18, L695)];
const V_SHOT_107E: Variants = &[(0, L696)];
const V_SHOT_1087: Variants = &[(0, L697), (2, L697)];
const V_SHOT_1088: Variants = &[(0, L698), (2, L698)];
const V_SHOT_108B: Variants = &[(0, L254), (16, L699)];
const V_SHOT_108C: Variants = &[(0, L700)];
const V_SHOT_108D: Variants = &[(0, L681), (2, L681), (16, L682), (18, L682)];
const V_SHOT_108E: Variants = &[(0, L701), (2, L702), (16, L703), (18, L704)];
const V_SHOT_1094: Variants = &[(0, L254), (1, L664), (2, L254), (3, L664), (16, L254), (17, L665), (18, L254), (19, L665)];
const V_SHOT_1097: Variants = &[(0, L254), (1, L676)];
const V_SHOT_1098: Variants = &[(0, L677), (2, L677)];
const V_SHOT_1099: Variants = &[(0, L681), (2, L681), (16, L682), (18, L682)];
const V_SHOT_109A: Variants = &[(0, L680)];
const V_SHOT_109B: Variants = &[(0, L705)];
const V_SHOT_109C: Variants = &[(0, L254), (1, L663)];
const V_SHOT_109D: Variants = &[(0, L254), (1, L663)];
const V_SHOT_109E: Variants = &[(0, L254), (1, L663)];
const V_SHOT_109F: Variants = &[(0, L706)];
const V_SHOT_10A1: Variants = &[(0, L707), (2, L707), (16, L708), (18, L708)];
const V_SHOT_10A3: Variants = &[(0, L666), (2, L666), (16, L667), (18, L667)];
const V_SHOT_10A4: Variants = &[(0, L677), (2, L677)];
const V_SHOT_10A5: Variants = &[(0, L254), (1, L676)];
const V_SHOT_10DA: Variants = &[(0, L709), (2, L709)];
const V_SHOT_10DB: Variants = &[(0, L710), (2, L710)];
const V_SHOT_118F: Variants = &[(0, L711)];
const V_SHOT_1190: Variants = &[(0, L711)];
const V_SHOT_11F4: Variants = &[(0, L712), (2, L712)];
const V_SHOT_11F5: Variants = &[(0, L712), (2, L712)];
const V_SHOT_120D: Variants = &[(0, L713), (2, L713), (16, L714), (18, L714)];
const V_SHOT_120E: Variants = &[(0, L713), (2, L713), (16, L714), (18, L714)];
const V_SHOT_1219: Variants = &[(0, L715), (2, L716), (16, L717), (18, L718)];
const V_SHOT_121F: Variants = &[(0, L715), (2, L716), (16, L717), (18, L718)];
const V_SHOT_1231: Variants = &[(0, L719), (2, L719), (16, L720), (18, L720)];
const V_SHOT_1232: Variants = &[(0, L719), (2, L719), (16, L720), (18, L720)];
const V_SHOT_1234: Variants = &[(0, L721), (2, L721), (16, L722), (18, L722)];
const V_SHOT_1235: Variants = &[(0, L721), (2, L721), (16, L722), (18, L722)];
const V_SHOT_1238: Variants = &[(0, L712), (2, L712)];
const V_SHOT_1239: Variants = &[(0, L712), (2, L712)];
const V_SHOT_1251: Variants = &[(0, L723)];
const V_SHOT_1252: Variants = &[(0, L724)];
const V_SHOT_1298: Variants = &[(0, L725)];
const V_SHOT_1299: Variants = &[(0, L725)];

static TABLE: &[(Phase, i32, u8, Variants)] = &[
    (Phase::Cast, 0x1, 0, V_CAST_1),
    (Phase::Cast, 0x2, 0, V_CAST_2),
    (Phase::Cast, 0x3, 4, V_CAST_3),
    (Phase::Cast, 0x4, 0, V_CAST_4),
    (Phase::Cast, 0x5, 0, V_CAST_5),
    (Phase::Cast, 0x6, 0, V_CAST_6),
    (Phase::Cast, 0x7, 0, V_CAST_7),
    (Phase::Cast, 0x8, 0, V_CAST_8),
    (Phase::Cast, 0x9, 0, V_CAST_9),
    (Phase::Cast, 0xa, 0, V_CAST_A),
    (Phase::Cast, 0xb, 0, V_CAST_B),
    (Phase::Cast, 0xc, 0, V_CAST_C),
    (Phase::Cast, 0xd, 0, V_CAST_D),
    (Phase::Cast, 0xf, 0, V_CAST_F),
    (Phase::Cast, 0x10, 4, V_CAST_10),
    (Phase::Cast, 0x11, 0, V_CAST_11),
    (Phase::Cast, 0x12, 0, V_CAST_12),
    (Phase::Cast, 0x13, 0, V_CAST_13),
    (Phase::Cast, 0x15, 0, V_CAST_15),
    (Phase::Cast, 0x16, 0, V_CAST_16),
    (Phase::Cast, 0x18, 0, V_CAST_18),
    (Phase::Cast, 0x19, 0, V_CAST_19),
    (Phase::Cast, 0x1b, 0, V_CAST_1B),
    (Phase::Cast, 0x1c, 0, V_CAST_1C),
    (Phase::Cast, 0x1d, 0, V_CAST_1D),
    (Phase::Cast, 0x1e, 4, V_CAST_1E),
    (Phase::Cast, 0x21, 0, V_CAST_21),
    (Phase::Cast, 0x22, 0, V_CAST_22),
    (Phase::Cast, 0x23, 0, V_CAST_23),
    (Phase::Cast, 0x24, 4, V_CAST_24),
    (Phase::Cast, 0x2c, 0, V_CAST_2C),
    (Phase::Cast, 0x2d, 0, V_CAST_2D),
    (Phase::Cast, 0x2e, 0, V_CAST_2E),
    (Phase::Cast, 0x30, 4, V_CAST_30),
    (Phase::Cast, 0x31, 0, V_CAST_31),
    (Phase::Cast, 0x32, 0, V_CAST_32),
    (Phase::Cast, 0x33, 0, V_CAST_33),
    (Phase::Cast, 0x36, 0, V_CAST_36),
    (Phase::Cast, 0x38, 0, V_CAST_38),
    (Phase::Cast, 0x3a, 0, V_CAST_3A),
    (Phase::Cast, 0x3d, 0, V_CAST_3D),
    (Phase::Cast, 0x41, 0, V_CAST_41),
    (Phase::Cast, 0x43, 0, V_CAST_43),
    (Phase::Cast, 0x45, 0, V_CAST_45),
    (Phase::Cast, 0x46, 0, V_CAST_46),
    (Phase::Cast, 0x48, 0, V_CAST_48),
    (Phase::Cast, 0x4b, 0, V_CAST_4B),
    (Phase::Cast, 0x4c, 0, V_CAST_4C),
    (Phase::Cast, 0x4d, 0, V_CAST_4D),
    (Phase::Cast, 0x4e, 0, V_CAST_4E),
    (Phase::Cast, 0x50, 0, V_CAST_50),
    (Phase::Cast, 0x51, 0, V_CAST_51),
    (Phase::Cast, 0x52, 0, V_CAST_52),
    (Phase::Cast, 0x53, 0, V_CAST_53),
    (Phase::Cast, 0x54, 0, V_CAST_54),
    (Phase::Cast, 0x56, 0, V_CAST_56),
    (Phase::Cast, 0x57, 0, V_CAST_57),
    (Phase::Cast, 0x58, 0, V_CAST_58),
    (Phase::Cast, 0x5b, 0, V_CAST_5B),
    (Phase::Cast, 0x5c, 0, V_CAST_5C),
    (Phase::Cast, 0x5e, 0, V_CAST_5E),
    (Phase::Cast, 0x5f, 0, V_CAST_5F),
    (Phase::Cast, 0x60, 4, V_CAST_60),
    (Phase::Cast, 0x61, 0, V_CAST_61),
    (Phase::Cast, 0x62, 0, V_CAST_62),
    (Phase::Cast, 0x63, 0, V_CAST_63),
    (Phase::Cast, 0x64, 4, V_CAST_64),
    (Phase::Cast, 0x65, 0, V_CAST_65),
    (Phase::Cast, 0x66, 0, V_CAST_66),
    (Phase::Cast, 0x67, 0, V_CAST_67),
    (Phase::Cast, 0x68, 0, V_CAST_68),
    (Phase::Cast, 0x69, 0, V_CAST_69),
    (Phase::Cast, 0x6a, 0, V_CAST_6A),
    (Phase::Cast, 0x6b, 0, V_CAST_6B),
    (Phase::Cast, 0x6d, 0, V_CAST_6D),
    (Phase::Cast, 0x6e, 0, V_CAST_6E),
    (Phase::Cast, 0x6f, 0, V_CAST_6F),
    (Phase::Cast, 0x70, 0, V_CAST_70),
    (Phase::Cast, 0x73, 0, V_CAST_73),
    (Phase::Cast, 0x74, 0, V_CAST_74),
    (Phase::Cast, 0x78, 0, V_CAST_78),
    (Phase::Cast, 0x79, 0, V_CAST_79),
    (Phase::Cast, 0x7a, 0, V_CAST_7A),
    (Phase::Cast, 0x7b, 0, V_CAST_7B),
    (Phase::Cast, 0x7f, 0, V_CAST_7F),
    (Phase::Cast, 0x81, 0, V_CAST_81),
    (Phase::Cast, 0x82, 0, V_CAST_82),
    (Phase::Cast, 0x83, 0, V_CAST_83),
    (Phase::Cast, 0x8b, 0, V_CAST_8B),
    (Phase::Cast, 0xb0, 0, V_CAST_B0),
    (Phase::Cast, 0xb5, 0, V_CAST_B5),
    (Phase::Cast, 0xbe, 4, V_CAST_BE),
    (Phase::Cast, 0xdf, 4, V_CAST_DF),
    (Phase::Cast, 0xe6, 0, V_CAST_E6),
    (Phase::Cast, 0xf5, 4, V_CAST_F5),
    (Phase::Cast, 0xf6, 0, V_CAST_F6),
    (Phase::Cast, 0xf7, 0, V_CAST_F7),
    (Phase::Cast, 0xfe, 0, V_CAST_FE),
    (Phase::Cast, 0xff, 4, V_CAST_FF),
    (Phase::Cast, 0x104, 4, V_CAST_104),
    (Phase::Cast, 0x105, 0, V_CAST_105),
    (Phase::Cast, 0x106, 0, V_CAST_106),
    (Phase::Cast, 0x107, 0, V_CAST_107),
    (Phase::Cast, 0x116, 0, V_CAST_116),
    (Phase::Cast, 0x117, 0, V_CAST_117),
    (Phase::Cast, 0x118, 0, V_CAST_118),
    (Phase::Cast, 0x119, 0, V_CAST_119),
    (Phase::Cast, 0x11a, 0, V_CAST_11A),
    (Phase::Cast, 0x11b, 0, V_CAST_11B),
    (Phase::Cast, 0x11c, 0, V_CAST_11C),
    (Phase::Cast, 0x11e, 0, V_CAST_11E),
    (Phase::Cast, 0x11f, 0, V_CAST_11F),
    (Phase::Cast, 0x121, 0, V_CAST_121),
    (Phase::Cast, 0x124, 0, V_CAST_124),
    (Phase::Cast, 0x129, 0, V_CAST_129),
    (Phase::Cast, 0x12a, 0, V_CAST_12A),
    (Phase::Cast, 0x12b, 0, V_CAST_12B),
    (Phase::Cast, 0x12d, 0, V_CAST_12D),
    (Phase::Cast, 0x12e, 0, V_CAST_12E),
    (Phase::Cast, 0x12f, 0, V_CAST_12F),
    (Phase::Cast, 0x13a, 0, V_CAST_13A),
    (Phase::Cast, 0x157, 0, V_CAST_157),
    (Phase::Cast, 0x162, 0, V_CAST_162),
    (Phase::Cast, 0x16a, 0, V_CAST_16A),
    (Phase::Cast, 0x171, 0, V_CAST_171),
    (Phase::Cast, 0x3ea, 2, V_CAST_3EA),
    (Phase::Cast, 0x3eb, 2, V_CAST_3EB),
    (Phase::Cast, 0x3ec, 2, V_CAST_3EC),
    (Phase::Cast, 0x3ed, 2, V_CAST_3ED),
    (Phase::Cast, 0x3ee, 2, V_CAST_3EE),
    (Phase::Cast, 0x3ef, 2, V_CAST_3EF),
    (Phase::Cast, 0x3f0, 2, V_CAST_3F0),
    (Phase::Cast, 0x3f1, 2, V_CAST_3F1),
    (Phase::Cast, 0x3f2, 2, V_CAST_3F2),
    (Phase::Cast, 0x3f3, 0, V_CAST_3F3),
    (Phase::Cast, 0x3f4, 0, V_CAST_3F4),
    (Phase::Cast, 0x3f5, 0, V_CAST_3F5),
    (Phase::Cast, 0x3f7, 0, V_CAST_3F7),
    (Phase::Cast, 0x3f8, 0, V_CAST_3F8),
    (Phase::Cast, 0x3fa, 0, V_CAST_3FA),
    (Phase::Cast, 0x3fc, 0, V_CAST_3FC),
    (Phase::Cast, 0x403, 0, V_CAST_403),
    (Phase::Cast, 0x404, 0, V_CAST_404),
    (Phase::Cast, 0x407, 0, V_CAST_407),
    (Phase::Cast, 0x408, 0, V_CAST_408),
    (Phase::Cast, 0x409, 0, V_CAST_409),
    (Phase::Cast, 0x40a, 0, V_CAST_40A),
    (Phase::Cast, 0x40b, 0, V_CAST_40B),
    (Phase::Cast, 0x40c, 0, V_CAST_40C),
    (Phase::Cast, 0x410, 0, V_CAST_410),
    (Phase::Cast, 0x412, 0, V_CAST_412),
    (Phase::Cast, 0x413, 0, V_CAST_413),
    (Phase::Cast, 0x414, 0, V_CAST_414),
    (Phase::Cast, 0x415, 0, V_CAST_415),
    (Phase::Cast, 0x417, 0, V_CAST_417),
    (Phase::Cast, 0x418, 0, V_CAST_418),
    (Phase::Cast, 0x419, 0, V_CAST_419),
    (Phase::Cast, 0x41a, 0, V_CAST_41A),
    (Phase::Cast, 0x420, 0, V_CAST_420),
    (Phase::Cast, 0x423, 0, V_CAST_423),
    (Phase::Cast, 0x426, 0, V_CAST_426),
    (Phase::Cast, 0x428, 0, V_CAST_428),
    (Phase::Cast, 0x429, 0, V_CAST_429),
    (Phase::Cast, 0x42c, 0, V_CAST_42C),
    (Phase::Cast, 0x42d, 0, V_CAST_42D),
    (Phase::Cast, 0x42f, 0, V_CAST_42F),
    (Phase::Cast, 0x430, 0, V_CAST_430),
    (Phase::Cast, 0x431, 0, V_CAST_431),
    (Phase::Cast, 0x432, 0, V_CAST_432),
    (Phase::Cast, 0x433, 0, V_CAST_433),
    (Phase::Cast, 0x435, 0, V_CAST_435),
    (Phase::Cast, 0x436, 0, V_CAST_436),
    (Phase::Cast, 0x43b, 0, V_CAST_43B),
    (Phase::Cast, 0x43d, 0, V_CAST_43D),
    (Phase::Cast, 0x43e, 0, V_CAST_43E),
    (Phase::Cast, 0x43f, 0, V_CAST_43F),
    (Phase::Cast, 0x442, 0, V_CAST_442),
    (Phase::Cast, 0x444, 2, V_CAST_444),
    (Phase::Cast, 0x447, 2, V_CAST_447),
    (Phase::Cast, 0x448, 2, V_CAST_448),
    (Phase::Cast, 0x449, 2, V_CAST_449),
    (Phase::Cast, 0x44b, 2, V_CAST_44B),
    (Phase::Cast, 0x44c, 2, V_CAST_44C),
    (Phase::Cast, 0x44d, 0, V_CAST_44D),
    (Phase::Cast, 0x44e, 2, V_CAST_44E),
    (Phase::Cast, 0x450, 2, V_CAST_450),
    (Phase::Cast, 0x451, 2, V_CAST_451),
    (Phase::Cast, 0x453, 2, V_CAST_453),
    (Phase::Cast, 0x454, 0, V_CAST_454),
    (Phase::Cast, 0x457, 0, V_CAST_457),
    (Phase::Cast, 0x466, 0, V_CAST_466),
    (Phase::Cast, 0x467, 0, V_CAST_467),
    (Phase::Cast, 0x468, 0, V_CAST_468),
    (Phase::Cast, 0x469, 0, V_CAST_469),
    (Phase::Cast, 0x472, 0, V_CAST_472),
    (Phase::Cast, 0x473, 0, V_CAST_473),
    (Phase::Cast, 0x474, 0, V_CAST_474),
    (Phase::Cast, 0x475, 0, V_CAST_475),
    (Phase::Cast, 0x478, 0, V_CAST_478),
    (Phase::Cast, 0x479, 0, V_CAST_479),
    (Phase::Cast, 0x47a, 0, V_CAST_47A),
    (Phase::Cast, 0x47b, 0, V_CAST_47B),
    (Phase::Cast, 0x47c, 0, V_CAST_47C),
    (Phase::Cast, 0x47f, 0, V_CAST_47F),
    (Phase::Cast, 0x482, 0, V_CAST_482),
    (Phase::Cast, 0x483, 0, V_CAST_483),
    (Phase::Cast, 0x484, 0, V_CAST_484),
    (Phase::Cast, 0x485, 0, V_CAST_485),
    (Phase::Cast, 0x487, 0, V_CAST_487),
    (Phase::Cast, 0x488, 0, V_CAST_488),
    (Phase::Cast, 0x48b, 0, V_CAST_48B),
    (Phase::Cast, 0x48c, 0, V_CAST_48C),
    (Phase::Cast, 0x48f, 0, V_CAST_48F),
    (Phase::Cast, 0x490, 0, V_CAST_490),
    (Phase::Cast, 0x491, 0, V_CAST_491),
    (Phase::Cast, 0x492, 0, V_CAST_492),
    (Phase::Cast, 0x493, 0, V_CAST_493),
    (Phase::Cast, 0x494, 0, V_CAST_494),
    (Phase::Cast, 0x496, 0, V_CAST_496),
    (Phase::Cast, 0x497, 2, V_CAST_497),
    (Phase::Cast, 0x498, 2, V_CAST_498),
    (Phase::Cast, 0x499, 0, V_CAST_499),
    (Phase::Cast, 0x49a, 2, V_CAST_49A),
    (Phase::Cast, 0x49d, 0, V_CAST_49D),
    (Phase::Cast, 0x49e, 0, V_CAST_49E),
    (Phase::Cast, 0x49f, 0, V_CAST_49F),
    (Phase::Cast, 0x4a0, 0, V_CAST_4A0),
    (Phase::Cast, 0x4a5, 0, V_CAST_4A5),
    (Phase::Cast, 0x4a7, 0, V_CAST_4A7),
    (Phase::Cast, 0x4b1, 0, V_CAST_4B1),
    (Phase::Cast, 0x4b4, 0, V_CAST_4B4),
    (Phase::Cast, 0x4b6, 0, V_CAST_4B6),
    (Phase::Cast, 0x4b8, 2, V_CAST_4B8),
    (Phase::Cast, 0x4b9, 2, V_CAST_4B9),
    (Phase::Cast, 0x4ba, 2, V_CAST_4BA),
    (Phase::Cast, 0x4bd, 2, V_CAST_4BD),
    (Phase::Cast, 0x4c0, 0, V_CAST_4C0),
    (Phase::Cast, 0x4c1, 0, V_CAST_4C1),
    (Phase::Cast, 0x4c2, 0, V_CAST_4C2),
    (Phase::Cast, 0x4c3, 0, V_CAST_4C3),
    (Phase::Cast, 0x4c4, 0, V_CAST_4C4),
    (Phase::Cast, 0x4c6, 0, V_CAST_4C6),
    (Phase::Cast, 0x4c7, 0, V_CAST_4C7),
    (Phase::Cast, 0x4c8, 0, V_CAST_4C8),
    (Phase::Cast, 0x4c9, 0, V_CAST_4C9),
    (Phase::Cast, 0x4ca, 0, V_CAST_4CA),
    (Phase::Cast, 0x4cb, 0, V_CAST_4CB),
    (Phase::Cast, 0x4cc, 0, V_CAST_4CC),
    (Phase::Cast, 0x4cd, 2, V_CAST_4CD),
    (Phase::Cast, 0x4ce, 0, V_CAST_4CE),
    (Phase::Cast, 0x4cf, 0, V_CAST_4CF),
    (Phase::Cast, 0x4d0, 0, V_CAST_4D0),
    (Phase::Cast, 0x4d1, 0, V_CAST_4D1),
    (Phase::Cast, 0x4d2, 0, V_CAST_4D2),
    (Phase::Cast, 0x4d3, 0, V_CAST_4D3),
    (Phase::Cast, 0x4d4, 0, V_CAST_4D4),
    (Phase::Cast, 0x4d5, 2, V_CAST_4D5),
    (Phase::Cast, 0x4d6, 0, V_CAST_4D6),
    (Phase::Cast, 0x4d7, 2, V_CAST_4D7),
    (Phase::Cast, 0x4d8, 0, V_CAST_4D8),
    (Phase::Cast, 0x4d9, 0, V_CAST_4D9),
    (Phase::Cast, 0x4da, 0, V_CAST_4DA),
    (Phase::Cast, 0x4db, 0, V_CAST_4DB),
    (Phase::Cast, 0x4dc, 2, V_CAST_4DC),
    (Phase::Cast, 0x4dd, 0, V_CAST_4DD),
    (Phase::Cast, 0x4de, 2, V_CAST_4DE),
    (Phase::Cast, 0x4df, 2, V_CAST_4DF),
    (Phase::Cast, 0x4e0, 2, V_CAST_4E0),
    (Phase::Cast, 0x4e1, 2, V_CAST_4E1),
    (Phase::Cast, 0x4e2, 2, V_CAST_4E2),
    (Phase::Cast, 0x4e3, 2, V_CAST_4E3),
    (Phase::Cast, 0x4e4, 2, V_CAST_4E4),
    (Phase::Cast, 0x4e5, 2, V_CAST_4E5),
    (Phase::Cast, 0x4e6, 0, V_CAST_4E6),
    (Phase::Cast, 0x4e7, 0, V_CAST_4E7),
    (Phase::Cast, 0x4e8, 2, V_CAST_4E8),
    (Phase::Cast, 0x4e9, 0, V_CAST_4E9),
    (Phase::Cast, 0x4ea, 0, V_CAST_4EA),
    (Phase::Cast, 0x4eb, 0, V_CAST_4EB),
    (Phase::Cast, 0x4ec, 2, V_CAST_4EC),
    (Phase::Cast, 0x4ed, 2, V_CAST_4ED),
    (Phase::Cast, 0x4ef, 0, V_CAST_4EF),
    (Phase::Cast, 0x4f0, 0, V_CAST_4F0),
    (Phase::Cast, 0x4f1, 0, V_CAST_4F1),
    (Phase::Cast, 0x4f2, 0, V_CAST_4F2),
    (Phase::Cast, 0x4f3, 0, V_CAST_4F3),
    (Phase::Cast, 0x4f4, 0, V_CAST_4F4),
    (Phase::Cast, 0x4f5, 0, V_CAST_4F5),
    (Phase::Cast, 0x4f7, 0, V_CAST_4F7),
    (Phase::Cast, 0x4f8, 0, V_CAST_4F8),
    (Phase::Cast, 0x4f9, 0, V_CAST_4F9),
    (Phase::Cast, 0x4fa, 0, V_CAST_4FA),
    (Phase::Cast, 0x4fb, 0, V_CAST_4FB),
    (Phase::Cast, 0x4fc, 0, V_CAST_4FC),
    (Phase::Cast, 0x4fd, 0, V_CAST_4FD),
    (Phase::Cast, 0x4fe, 0, V_CAST_4FE),
    (Phase::Cast, 0x4ff, 0, V_CAST_4FF),
    (Phase::Cast, 0x500, 0, V_CAST_500),
    (Phase::Cast, 0x501, 0, V_CAST_501),
    (Phase::Cast, 0x502, 2, V_CAST_502),
    (Phase::Cast, 0x504, 2, V_CAST_504),
    (Phase::Cast, 0x505, 0, V_CAST_505),
    (Phase::Cast, 0x506, 0, V_CAST_506),
    (Phase::Cast, 0x507, 0, V_CAST_507),
    (Phase::Cast, 0x508, 0, V_CAST_508),
    (Phase::Cast, 0x509, 0, V_CAST_509),
    (Phase::Cast, 0x50a, 2, V_CAST_50A),
    (Phase::Cast, 0x50b, 2, V_CAST_50B),
    (Phase::Cast, 0x50c, 0, V_CAST_50C),
    (Phase::Cast, 0x50d, 0, V_CAST_50D),
    (Phase::Cast, 0x50e, 0, V_CAST_50E),
    (Phase::Cast, 0x518, 0, V_CAST_518),
    (Phase::Cast, 0x552, 2, V_CAST_552),
    (Phase::Cast, 0x553, 2, V_CAST_553),
    (Phase::Cast, 0x554, 2, V_CAST_554),
    (Phase::Cast, 0x555, 2, V_CAST_555),
    (Phase::Cast, 0x556, 2, V_CAST_556),
    (Phase::Cast, 0x557, 2, V_CAST_557),
    (Phase::Cast, 0x7dd, 0, V_CAST_7DD),
    (Phase::Cast, 0x7de, 0, V_CAST_7DE),
    (Phase::Cast, 0x7f8, 0, V_CAST_7F8),
    (Phase::Cast, 0x7f9, 0, V_CAST_7F9),
    (Phase::Cast, 0x7fe, 0, V_CAST_7FE),
    (Phase::Cast, 0x801, 0, V_CAST_801),
    (Phase::Cast, 0x802, 0, V_CAST_802),
    (Phase::Cast, 0x803, 0, V_CAST_803),
    (Phase::Cast, 0x804, 0, V_CAST_804),
    (Phase::Cast, 0x805, 0, V_CAST_805),
    (Phase::Cast, 0x806, 0, V_CAST_806),
    (Phase::Cast, 0x807, 0, V_CAST_807),
    (Phase::Cast, 0x808, 0, V_CAST_808),
    (Phase::Cast, 0x809, 0, V_CAST_809),
    (Phase::Cast, 0x80a, 0, V_CAST_80A),
    (Phase::Cast, 0x80b, 0, V_CAST_80B),
    (Phase::Cast, 0x80e, 0, V_CAST_80E),
    (Phase::Cast, 0x810, 0, V_CAST_810),
    (Phase::Cast, 0x830, 0, V_CAST_830),
    (Phase::Cast, 0x831, 0, V_CAST_831),
    (Phase::Cast, 0x833, 0, V_CAST_833),
    (Phase::Cast, 0x834, 0, V_CAST_834),
    (Phase::Cast, 0xfa1, 0, V_CAST_FA1),
    (Phase::Cast, 0xfa2, 0, V_CAST_FA2),
    (Phase::Cast, 0xfb1, 0, V_CAST_FB1),
    (Phase::Cast, 0xfb4, 0, V_CAST_FB4),
    (Phase::Cast, 0xfb8, 0, V_CAST_FB8),
    (Phase::Cast, 0xfb9, 0, V_CAST_FB9),
    (Phase::Cast, 0xfba, 0, V_CAST_FBA),
    (Phase::Cast, 0xfbb, 0, V_CAST_FBB),
    (Phase::Cast, 0xfbc, 0, V_CAST_FBC),
    (Phase::Cast, 0xfbd, 0, V_CAST_FBD),
    (Phase::Cast, 0xfbe, 0, V_CAST_FBE),
    (Phase::Cast, 0xfbf, 0, V_CAST_FBF),
    (Phase::Cast, 0xfc0, 4, V_CAST_FC0),
    (Phase::Cast, 0xfc1, 0, V_CAST_FC1),
    (Phase::Cast, 0xfc2, 0, V_CAST_FC2),
    (Phase::Cast, 0xfc3, 0, V_CAST_FC3),
    (Phase::Cast, 0xfc4, 0, V_CAST_FC4),
    (Phase::Cast, 0xfc5, 0, V_CAST_FC5),
    (Phase::Cast, 0xfc6, 0, V_CAST_FC6),
    (Phase::Cast, 0xfc7, 0, V_CAST_FC7),
    (Phase::Cast, 0xfc8, 0, V_CAST_FC8),
    (Phase::Cast, 0xfca, 0, V_CAST_FCA),
    (Phase::Cast, 0xfcb, 0, V_CAST_FCB),
    (Phase::Cast, 0xfcc, 0, V_CAST_FCC),
    (Phase::Cast, 0xfce, 0, V_CAST_FCE),
    (Phase::Cast, 0xfcf, 0, V_CAST_FCF),
    (Phase::Cast, 0xfdf, 0, V_CAST_FDF),
    (Phase::Cast, 0xfe0, 0, V_CAST_FE0),
    (Phase::Cast, 0xfe1, 0, V_CAST_FE1),
    (Phase::Cast, 0xfe2, 2, V_CAST_FE2),
    (Phase::Cast, 0xfe3, 0, V_CAST_FE3),
    (Phase::Cast, 0xfe8, 0, V_CAST_FE8),
    (Phase::Cast, 0xfe9, 0, V_CAST_FE9),
    (Phase::Cast, 0xfea, 0, V_CAST_FEA),
    (Phase::Cast, 0xfeb, 0, V_CAST_FEB),
    (Phase::Cast, 0xfec, 0, V_CAST_FEC),
    (Phase::Cast, 0xfed, 0, V_CAST_FED),
    (Phase::Cast, 0xfee, 0, V_CAST_FEE),
    (Phase::Cast, 0xff0, 0, V_CAST_FF0),
    (Phase::Cast, 0xff1, 0, V_CAST_FF1),
    (Phase::Cast, 0xff7, 0, V_CAST_FF7),
    (Phase::Cast, 0xff8, 0, V_CAST_FF8),
    (Phase::Cast, 0xff9, 0, V_CAST_FF9),
    (Phase::Cast, 0xffa, 0, V_CAST_FFA),
    (Phase::Cast, 0xffb, 0, V_CAST_FFB),
    (Phase::Cast, 0xffc, 0, V_CAST_FFC),
    (Phase::Cast, 0xffe, 0, V_CAST_FFE),
    (Phase::Cast, 0x1000, 0, V_CAST_1000),
    (Phase::Cast, 0x1001, 2, V_CAST_1001),
    (Phase::Cast, 0x1002, 0, V_CAST_1002),
    (Phase::Cast, 0x1003, 0, V_CAST_1003),
    (Phase::Cast, 0x1004, 0, V_CAST_1004),
    (Phase::Cast, 0x1005, 0, V_CAST_1005),
    (Phase::Cast, 0x1006, 0, V_CAST_1006),
    (Phase::Cast, 0x1007, 0, V_CAST_1007),
    (Phase::Cast, 0x1008, 0, V_CAST_1008),
    (Phase::Cast, 0x1009, 0, V_CAST_1009),
    (Phase::Cast, 0x1012, 0, V_CAST_1012),
    (Phase::Cast, 0x1013, 0, V_CAST_1013),
    (Phase::Cast, 0x1015, 0, V_CAST_1015),
    (Phase::Cast, 0x1016, 0, V_CAST_1016),
    (Phase::Cast, 0x1017, 0, V_CAST_1017),
    (Phase::Cast, 0x1018, 0, V_CAST_1018),
    (Phase::Cast, 0x101b, 0, V_CAST_101B),
    (Phase::Cast, 0x101c, 0, V_CAST_101C),
    (Phase::Cast, 0x1020, 0, V_CAST_1020),
    (Phase::Cast, 0x1022, 2, V_CAST_1022),
    (Phase::Cast, 0x1024, 0, V_CAST_1024),
    (Phase::Cast, 0x1025, 0, V_CAST_1025),
    (Phase::Cast, 0x1029, 0, V_CAST_1029),
    (Phase::Cast, 0x102a, 0, V_CAST_102A),
    (Phase::Cast, 0x102b, 0, V_CAST_102B),
    (Phase::Cast, 0x102d, 0, V_CAST_102D),
    (Phase::Cast, 0x102e, 0, V_CAST_102E),
    (Phase::Cast, 0x1030, 0, V_CAST_1030),
    (Phase::Cast, 0x1037, 0, V_CAST_1037),
    (Phase::Cast, 0x1038, 0, V_CAST_1038),
    (Phase::Cast, 0x1039, 0, V_CAST_1039),
    (Phase::Cast, 0x103a, 0, V_CAST_103A),
    (Phase::Cast, 0x103b, 2, V_CAST_103B),
    (Phase::Cast, 0x103d, 0, V_CAST_103D),
    (Phase::Cast, 0x103e, 0, V_CAST_103E),
    (Phase::Cast, 0x1040, 0, V_CAST_1040),
    (Phase::Cast, 0x1041, 2, V_CAST_1041),
    (Phase::Cast, 0x1043, 0, V_CAST_1043),
    (Phase::Cast, 0x1048, 0, V_CAST_1048),
    (Phase::Cast, 0x1049, 0, V_CAST_1049),
    (Phase::Cast, 0x104a, 0, V_CAST_104A),
    (Phase::Cast, 0x104b, 0, V_CAST_104B),
    (Phase::Cast, 0x104c, 0, V_CAST_104C),
    (Phase::Cast, 0x104d, 0, V_CAST_104D),
    (Phase::Cast, 0x104e, 0, V_CAST_104E),
    (Phase::Cast, 0x104f, 0, V_CAST_104F),
    (Phase::Cast, 0x1050, 0, V_CAST_1050),
    (Phase::Cast, 0x1051, 0, V_CAST_1051),
    (Phase::Cast, 0x1052, 0, V_CAST_1052),
    (Phase::Cast, 0x1053, 0, V_CAST_1053),
    (Phase::Cast, 0x1054, 0, V_CAST_1054),
    (Phase::Cast, 0x1055, 0, V_CAST_1055),
    (Phase::Cast, 0x1056, 0, V_CAST_1056),
    (Phase::Cast, 0x1057, 0, V_CAST_1057),
    (Phase::Cast, 0x1058, 0, V_CAST_1058),
    (Phase::Cast, 0x1059, 0, V_CAST_1059),
    (Phase::Cast, 0x105a, 0, V_CAST_105A),
    (Phase::Cast, 0x105b, 0, V_CAST_105B),
    (Phase::Cast, 0x105c, 0, V_CAST_105C),
    (Phase::Cast, 0x105d, 0, V_CAST_105D),
    (Phase::Cast, 0x105e, 0, V_CAST_105E),
    (Phase::Cast, 0x105f, 0, V_CAST_105F),
    (Phase::Cast, 0x1060, 0, V_CAST_1060),
    (Phase::Cast, 0x1061, 0, V_CAST_1061),
    (Phase::Cast, 0x1062, 0, V_CAST_1062),
    (Phase::Cast, 0x1063, 2, V_CAST_1063),
    (Phase::Cast, 0x1064, 0, V_CAST_1064),
    (Phase::Cast, 0x1065, 0, V_CAST_1065),
    (Phase::Cast, 0x1066, 0, V_CAST_1066),
    (Phase::Cast, 0x1067, 0, V_CAST_1067),
    (Phase::Cast, 0x1068, 0, V_CAST_1068),
    (Phase::Cast, 0x1069, 0, V_CAST_1069),
    (Phase::Cast, 0x106a, 0, V_CAST_106A),
    (Phase::Cast, 0x106b, 0, V_CAST_106B),
    (Phase::Cast, 0x106c, 2, V_CAST_106C),
    (Phase::Cast, 0x106d, 0, V_CAST_106D),
    (Phase::Cast, 0x106e, 0, V_CAST_106E),
    (Phase::Cast, 0x106f, 0, V_CAST_106F),
    (Phase::Cast, 0x1070, 0, V_CAST_1070),
    (Phase::Cast, 0x1071, 0, V_CAST_1071),
    (Phase::Cast, 0x1072, 2, V_CAST_1072),
    (Phase::Cast, 0x1073, 0, V_CAST_1073),
    (Phase::Cast, 0x1074, 0, V_CAST_1074),
    (Phase::Cast, 0x1075, 0, V_CAST_1075),
    (Phase::Cast, 0x1076, 0, V_CAST_1076),
    (Phase::Cast, 0x1078, 0, V_CAST_1078),
    (Phase::Cast, 0x1079, 0, V_CAST_1079),
    (Phase::Cast, 0x107a, 0, V_CAST_107A),
    (Phase::Cast, 0x107b, 0, V_CAST_107B),
    (Phase::Cast, 0x107c, 0, V_CAST_107C),
    (Phase::Cast, 0x107d, 0, V_CAST_107D),
    (Phase::Cast, 0x107e, 0, V_CAST_107E),
    (Phase::Cast, 0x1084, 0, V_CAST_1084),
    (Phase::Cast, 0x1085, 0, V_CAST_1085),
    (Phase::Cast, 0x1088, 0, V_CAST_1088),
    (Phase::Cast, 0x108b, 0, V_CAST_108B),
    (Phase::Cast, 0x108c, 2, V_CAST_108C),
    (Phase::Cast, 0x108d, 0, V_CAST_108D),
    (Phase::Cast, 0x108e, 2, V_CAST_108E),
    (Phase::Cast, 0x1094, 0, V_CAST_1094),
    (Phase::Cast, 0x1097, 0, V_CAST_1097),
    (Phase::Cast, 0x1098, 0, V_CAST_1098),
    (Phase::Cast, 0x1099, 0, V_CAST_1099),
    (Phase::Cast, 0x109a, 2, V_CAST_109A),
    (Phase::Cast, 0x109c, 0, V_CAST_109C),
    (Phase::Cast, 0x109d, 0, V_CAST_109D),
    (Phase::Cast, 0x109e, 0, V_CAST_109E),
    (Phase::Cast, 0x109f, 0, V_CAST_109F),
    (Phase::Cast, 0x10a1, 0, V_CAST_10A1),
    (Phase::Cast, 0x10a3, 0, V_CAST_10A3),
    (Phase::Cast, 0x10a4, 0, V_CAST_10A4),
    (Phase::Cast, 0x10a5, 0, V_CAST_10A5),
    (Phase::Cast, 0x10da, 0, V_CAST_10DA),
    (Phase::Cast, 0x10db, 0, V_CAST_10DB),
    (Phase::Cast, 0x118f, 0, V_CAST_118F),
    (Phase::Cast, 0x1190, 0, V_CAST_1190),
    (Phase::Cast, 0x11f4, 0, V_CAST_11F4),
    (Phase::Cast, 0x11f5, 0, V_CAST_11F5),
    (Phase::Cast, 0x120d, 0, V_CAST_120D),
    (Phase::Cast, 0x120e, 0, V_CAST_120E),
    (Phase::Cast, 0x1219, 2, V_CAST_1219),
    (Phase::Cast, 0x121f, 2, V_CAST_121F),
    (Phase::Cast, 0x1231, 0, V_CAST_1231),
    (Phase::Cast, 0x1232, 0, V_CAST_1232),
    (Phase::Cast, 0x1234, 0, V_CAST_1234),
    (Phase::Cast, 0x1235, 0, V_CAST_1235),
    (Phase::Cast, 0x1238, 0, V_CAST_1238),
    (Phase::Cast, 0x1239, 0, V_CAST_1239),
    (Phase::Cast, 0x1252, 8, V_CAST_1252),
    (Phase::Cast, 0x1292, 0, V_CAST_1292),
    (Phase::Cast, 0x1293, 0, V_CAST_1293),
    (Phase::Cast, 0x1294, 0, V_CAST_1294),
    (Phase::Cast, 0x1295, 0, V_CAST_1295),
    (Phase::Cast, 0x1298, 0, V_CAST_1298),
    (Phase::Cast, 0x1299, 0, V_CAST_1299),
    (Phase::PreShot, 0x13, 0, V_PRESHOT_13),
    (Phase::PreShot, 0x18, 0, V_PRESHOT_18),
    (Phase::PreShot, 0x38, 0, V_PRESHOT_38),
    (Phase::PreShot, 0x65, 0, V_PRESHOT_65),
    (Phase::PreShot, 0x13a, 0, V_PRESHOT_13A),
    (Phase::PreShot, 0x157, 2, V_PRESHOT_157),
    (Phase::PreShot, 0x162, 2, V_PRESHOT_162),
    (Phase::PreShot, 0x171, 0, V_PRESHOT_171),
    (Phase::PreShot, 0xfc8, 0, V_PRESHOT_FC8),
    (Phase::PreShot, 0x1018, 0, V_PRESHOT_1018),
    (Phase::PreShot, 0x106f, 0, V_PRESHOT_106F),
    (Phase::PreShot, 0x1070, 0, V_PRESHOT_1070),
    (Phase::PreShot, 0x1084, 0, V_PRESHOT_1084),
    (Phase::PreShot, 0x1292, 0, V_PRESHOT_1292),
    (Phase::PreShot, 0x1293, 0, V_PRESHOT_1293),
    (Phase::PreShot, 0x1294, 0, V_PRESHOT_1294),
    (Phase::PreShot, 0x1295, 0, V_PRESHOT_1295),
    (Phase::PreShot, 0x1298, 0, V_PRESHOT_1298),
    (Phase::PreShot, 0x1299, 0, V_PRESHOT_1299),
    (Phase::Shot, 0x2, 18, V_SHOT_2),
    (Phase::Shot, 0x3, 19, V_SHOT_3),
    (Phase::Shot, 0x4, 18, V_SHOT_4),
    (Phase::Shot, 0x5, 19, V_SHOT_5),
    (Phase::Shot, 0x6, 1, V_SHOT_6),
    (Phase::Shot, 0x7, 1, V_SHOT_7),
    (Phase::Shot, 0x8, 18, V_SHOT_8),
    (Phase::Shot, 0x9, 16, V_SHOT_9),
    (Phase::Shot, 0xb, 18, V_SHOT_B),
    (Phase::Shot, 0xc, 18, V_SHOT_C),
    (Phase::Shot, 0xf, 18, V_SHOT_F),
    (Phase::Shot, 0x11, 3, V_SHOT_11),
    (Phase::Shot, 0x12, 18, V_SHOT_12),
    (Phase::Shot, 0x15, 2, V_SHOT_15),
    (Phase::Shot, 0x1b, 18, V_SHOT_1B),
    (Phase::Shot, 0x1c, 18, V_SHOT_1C),
    (Phase::Shot, 0x1d, 3, V_SHOT_1D),
    (Phase::Shot, 0x1e, 19, V_SHOT_1E),
    (Phase::Shot, 0x22, 18, V_SHOT_22),
    (Phase::Shot, 0x23, 1, V_SHOT_23),
    (Phase::Shot, 0x24, 19, V_SHOT_24),
    (Phase::Shot, 0x2c, 18, V_SHOT_2C),
    (Phase::Shot, 0x2d, 18, V_SHOT_2D),
    (Phase::Shot, 0x2e, 2, V_SHOT_2E),
    (Phase::Shot, 0x30, 19, V_SHOT_30),
    (Phase::Shot, 0x31, 18, V_SHOT_31),
    (Phase::Shot, 0x32, 18, V_SHOT_32),
    (Phase::Shot, 0x33, 18, V_SHOT_33),
    (Phase::Shot, 0x36, 1, V_SHOT_36),
    (Phase::Shot, 0x3a, 2, V_SHOT_3A),
    (Phase::Shot, 0x3d, 2, V_SHOT_3D),
    (Phase::Shot, 0x41, 18, V_SHOT_41),
    (Phase::Shot, 0x45, 18, V_SHOT_45),
    (Phase::Shot, 0x46, 2, V_SHOT_46),
    (Phase::Shot, 0x48, 18, V_SHOT_48),
    (Phase::Shot, 0x4b, 18, V_SHOT_4B),
    (Phase::Shot, 0x4c, 18, V_SHOT_4C),
    (Phase::Shot, 0x4d, 18, V_SHOT_4D),
    (Phase::Shot, 0x4e, 18, V_SHOT_4E),
    (Phase::Shot, 0x50, 18, V_SHOT_50),
    (Phase::Shot, 0x51, 3, V_SHOT_51),
    (Phase::Shot, 0x52, 18, V_SHOT_52),
    (Phase::Shot, 0x53, 18, V_SHOT_53),
    (Phase::Shot, 0x54, 17, V_SHOT_54),
    (Phase::Shot, 0x56, 18, V_SHOT_56),
    (Phase::Shot, 0x57, 18, V_SHOT_57),
    (Phase::Shot, 0x58, 18, V_SHOT_58),
    (Phase::Shot, 0x5b, 18, V_SHOT_5B),
    (Phase::Shot, 0x5c, 18, V_SHOT_5C),
    (Phase::Shot, 0x5d, 18, V_SHOT_5D),
    (Phase::Shot, 0x5e, 18, V_SHOT_5E),
    (Phase::Shot, 0x5f, 3, V_SHOT_5F),
    (Phase::Shot, 0x60, 2, V_SHOT_60),
    (Phase::Shot, 0x61, 0, V_SHOT_61),
    (Phase::Shot, 0x62, 18, V_SHOT_62),
    (Phase::Shot, 0x63, 18, V_SHOT_63),
    (Phase::Shot, 0x64, 19, V_SHOT_64),
    (Phase::Shot, 0x65, 0, V_SHOT_65),
    (Phase::Shot, 0x66, 18, V_SHOT_66),
    (Phase::Shot, 0x67, 18, V_SHOT_67),
    (Phase::Shot, 0x68, 18, V_SHOT_68),
    (Phase::Shot, 0x69, 18, V_SHOT_69),
    (Phase::Shot, 0x6a, 18, V_SHOT_6A),
    (Phase::Shot, 0x6b, 18, V_SHOT_6B),
    (Phase::Shot, 0x6d, 18, V_SHOT_6D),
    (Phase::Shot, 0x6e, 18, V_SHOT_6E),
    (Phase::Shot, 0x6f, 18, V_SHOT_6F),
    (Phase::Shot, 0x70, 18, V_SHOT_70),
    (Phase::Shot, 0x73, 18, V_SHOT_73),
    (Phase::Shot, 0x74, 0, V_SHOT_74),
    (Phase::Shot, 0x78, 19, V_SHOT_78),
    (Phase::Shot, 0x79, 18, V_SHOT_79),
    (Phase::Shot, 0x7a, 18, V_SHOT_7A),
    (Phase::Shot, 0x7b, 18, V_SHOT_7B),
    (Phase::Shot, 0x7f, 18, V_SHOT_7F),
    (Phase::Shot, 0x81, 18, V_SHOT_81),
    (Phase::Shot, 0x82, 18, V_SHOT_82),
    (Phase::Shot, 0x83, 18, V_SHOT_83),
    (Phase::Shot, 0x8b, 18, V_SHOT_8B),
    (Phase::Shot, 0xb0, 18, V_SHOT_B0),
    (Phase::Shot, 0xb5, 18, V_SHOT_B5),
    (Phase::Shot, 0xbe, 19, V_SHOT_BE),
    (Phase::Shot, 0xdf, 19, V_SHOT_DF),
    (Phase::Shot, 0xe6, 18, V_SHOT_E6),
    (Phase::Shot, 0xf5, 19, V_SHOT_F5),
    (Phase::Shot, 0xf6, 18, V_SHOT_F6),
    (Phase::Shot, 0xfe, 18, V_SHOT_FE),
    (Phase::Shot, 0xff, 19, V_SHOT_FF),
    (Phase::Shot, 0x104, 19, V_SHOT_104),
    (Phase::Shot, 0x105, 19, V_SHOT_105),
    (Phase::Shot, 0x106, 18, V_SHOT_106),
    (Phase::Shot, 0x108, 0, V_SHOT_108),
    (Phase::Shot, 0x109, 0, V_SHOT_109),
    (Phase::Shot, 0x10a, 0, V_SHOT_10A),
    (Phase::Shot, 0x10b, 0, V_SHOT_10B),
    (Phase::Shot, 0x10c, 0, V_SHOT_10C),
    (Phase::Shot, 0x10d, 0, V_SHOT_10D),
    (Phase::Shot, 0x10e, 0, V_SHOT_10E),
    (Phase::Shot, 0x10f, 0, V_SHOT_10F),
    (Phase::Shot, 0x110, 0, V_SHOT_110),
    (Phase::Shot, 0x111, 0, V_SHOT_111),
    (Phase::Shot, 0x112, 0, V_SHOT_112),
    (Phase::Shot, 0x113, 0, V_SHOT_113),
    (Phase::Shot, 0x114, 0, V_SHOT_114),
    (Phase::Shot, 0x115, 0, V_SHOT_115),
    (Phase::Shot, 0x117, 18, V_SHOT_117),
    (Phase::Shot, 0x118, 1, V_SHOT_118),
    (Phase::Shot, 0x119, 19, V_SHOT_119),
    (Phase::Shot, 0x11a, 18, V_SHOT_11A),
    (Phase::Shot, 0x11c, 18, V_SHOT_11C),
    (Phase::Shot, 0x11e, 18, V_SHOT_11E),
    (Phase::Shot, 0x11f, 18, V_SHOT_11F),
    (Phase::Shot, 0x121, 2, V_SHOT_121),
    (Phase::Shot, 0x124, 18, V_SHOT_124),
    (Phase::Shot, 0x129, 18, V_SHOT_129),
    (Phase::Shot, 0x12a, 18, V_SHOT_12A),
    (Phase::Shot, 0x12e, 18, V_SHOT_12E),
    (Phase::Shot, 0x12f, 18, V_SHOT_12F),
    (Phase::Shot, 0x130, 0, V_SHOT_130),
    (Phase::Shot, 0x131, 0, V_SHOT_131),
    (Phase::Shot, 0x132, 0, V_SHOT_132),
    (Phase::Shot, 0x133, 0, V_SHOT_133),
    (Phase::Shot, 0x134, 0, V_SHOT_134),
    (Phase::Shot, 0x135, 0, V_SHOT_135),
    (Phase::Shot, 0x136, 0, V_SHOT_136),
    (Phase::Shot, 0x137, 0, V_SHOT_137),
    (Phase::Shot, 0x157, 0, V_SHOT_157),
    (Phase::Shot, 0x162, 0, V_SHOT_162),
    (Phase::Shot, 0x16a, 31, V_SHOT_16A),
    (Phase::Shot, 0x3ea, 18, V_SHOT_3EA),
    (Phase::Shot, 0x3eb, 18, V_SHOT_3EB),
    (Phase::Shot, 0x3ec, 18, V_SHOT_3EC),
    (Phase::Shot, 0x3ed, 18, V_SHOT_3ED),
    (Phase::Shot, 0x3ee, 18, V_SHOT_3EE),
    (Phase::Shot, 0x3ef, 18, V_SHOT_3EF),
    (Phase::Shot, 0x3f0, 18, V_SHOT_3F0),
    (Phase::Shot, 0x3f1, 18, V_SHOT_3F1),
    (Phase::Shot, 0x3f2, 18, V_SHOT_3F2),
    (Phase::Shot, 0x3f3, 18, V_SHOT_3F3),
    (Phase::Shot, 0x3f4, 2, V_SHOT_3F4),
    (Phase::Shot, 0x3f5, 2, V_SHOT_3F5),
    (Phase::Shot, 0x3f7, 2, V_SHOT_3F7),
    (Phase::Shot, 0x3f8, 18, V_SHOT_3F8),
    (Phase::Shot, 0x3fa, 2, V_SHOT_3FA),
    (Phase::Shot, 0x3fc, 18, V_SHOT_3FC),
    (Phase::Shot, 0x403, 16, V_SHOT_403),
    (Phase::Shot, 0x404, 18, V_SHOT_404),
    (Phase::Shot, 0x407, 18, V_SHOT_407),
    (Phase::Shot, 0x408, 18, V_SHOT_408),
    (Phase::Shot, 0x409, 18, V_SHOT_409),
    (Phase::Shot, 0x40a, 0, V_SHOT_40A),
    (Phase::Shot, 0x40b, 18, V_SHOT_40B),
    (Phase::Shot, 0x40c, 18, V_SHOT_40C),
    (Phase::Shot, 0x410, 18, V_SHOT_410),
    (Phase::Shot, 0x412, 18, V_SHOT_412),
    (Phase::Shot, 0x413, 18, V_SHOT_413),
    (Phase::Shot, 0x414, 18, V_SHOT_414),
    (Phase::Shot, 0x415, 18, V_SHOT_415),
    (Phase::Shot, 0x417, 18, V_SHOT_417),
    (Phase::Shot, 0x418, 18, V_SHOT_418),
    (Phase::Shot, 0x419, 0, V_SHOT_419),
    (Phase::Shot, 0x41a, 0, V_SHOT_41A),
    (Phase::Shot, 0x420, 18, V_SHOT_420),
    (Phase::Shot, 0x423, 18, V_SHOT_423),
    (Phase::Shot, 0x426, 18, V_SHOT_426),
    (Phase::Shot, 0x428, 18, V_SHOT_428),
    (Phase::Shot, 0x429, 18, V_SHOT_429),
    (Phase::Shot, 0x42c, 18, V_SHOT_42C),
    (Phase::Shot, 0x42d, 18, V_SHOT_42D),
    (Phase::Shot, 0x42f, 18, V_SHOT_42F),
    (Phase::Shot, 0x430, 18, V_SHOT_430),
    (Phase::Shot, 0x431, 18, V_SHOT_431),
    (Phase::Shot, 0x432, 18, V_SHOT_432),
    (Phase::Shot, 0x433, 18, V_SHOT_433),
    (Phase::Shot, 0x435, 18, V_SHOT_435),
    (Phase::Shot, 0x436, 18, V_SHOT_436),
    (Phase::Shot, 0x43b, 18, V_SHOT_43B),
    (Phase::Shot, 0x43d, 18, V_SHOT_43D),
    (Phase::Shot, 0x43e, 18, V_SHOT_43E),
    (Phase::Shot, 0x43f, 18, V_SHOT_43F),
    (Phase::Shot, 0x442, 2, V_SHOT_442),
    (Phase::Shot, 0x444, 18, V_SHOT_444),
    (Phase::Shot, 0x447, 18, V_SHOT_447),
    (Phase::Shot, 0x448, 18, V_SHOT_448),
    (Phase::Shot, 0x449, 18, V_SHOT_449),
    (Phase::Shot, 0x44b, 18, V_SHOT_44B),
    (Phase::Shot, 0x44c, 2, V_SHOT_44C),
    (Phase::Shot, 0x44d, 18, V_SHOT_44D),
    (Phase::Shot, 0x44e, 18, V_SHOT_44E),
    (Phase::Shot, 0x450, 18, V_SHOT_450),
    (Phase::Shot, 0x451, 18, V_SHOT_451),
    (Phase::Shot, 0x453, 2, V_SHOT_453),
    (Phase::Shot, 0x454, 18, V_SHOT_454),
    (Phase::Shot, 0x466, 18, V_SHOT_466),
    (Phase::Shot, 0x467, 18, V_SHOT_467),
    (Phase::Shot, 0x472, 18, V_SHOT_472),
    (Phase::Shot, 0x473, 18, V_SHOT_473),
    (Phase::Shot, 0x474, 18, V_SHOT_474),
    (Phase::Shot, 0x475, 18, V_SHOT_475),
    (Phase::Shot, 0x478, 18, V_SHOT_478),
    (Phase::Shot, 0x479, 18, V_SHOT_479),
    (Phase::Shot, 0x47a, 18, V_SHOT_47A),
    (Phase::Shot, 0x47b, 2, V_SHOT_47B),
    (Phase::Shot, 0x47c, 1, V_SHOT_47C),
    (Phase::Shot, 0x47f, 2, V_SHOT_47F),
    (Phase::Shot, 0x483, 18, V_SHOT_483),
    (Phase::Shot, 0x484, 18, V_SHOT_484),
    (Phase::Shot, 0x485, 18, V_SHOT_485),
    (Phase::Shot, 0x487, 1, V_SHOT_487),
    (Phase::Shot, 0x488, 18, V_SHOT_488),
    (Phase::Shot, 0x48b, 18, V_SHOT_48B),
    (Phase::Shot, 0x48c, 18, V_SHOT_48C),
    (Phase::Shot, 0x48f, 18, V_SHOT_48F),
    (Phase::Shot, 0x490, 18, V_SHOT_490),
    (Phase::Shot, 0x491, 18, V_SHOT_491),
    (Phase::Shot, 0x492, 18, V_SHOT_492),
    (Phase::Shot, 0x493, 16, V_SHOT_493),
    (Phase::Shot, 0x494, 18, V_SHOT_494),
    (Phase::Shot, 0x496, 18, V_SHOT_496),
    (Phase::Shot, 0x497, 0, V_SHOT_497),
    (Phase::Shot, 0x498, 0, V_SHOT_498),
    (Phase::Shot, 0x499, 1, V_SHOT_499),
    (Phase::Shot, 0x49a, 0, V_SHOT_49A),
    (Phase::Shot, 0x49d, 1, V_SHOT_49D),
    (Phase::Shot, 0x49e, 18, V_SHOT_49E),
    (Phase::Shot, 0x49f, 18, V_SHOT_49F),
    (Phase::Shot, 0x4a0, 18, V_SHOT_4A0),
    (Phase::Shot, 0x4a5, 18, V_SHOT_4A5),
    (Phase::Shot, 0x4a7, 18, V_SHOT_4A7),
    (Phase::Shot, 0x4b1, 18, V_SHOT_4B1),
    (Phase::Shot, 0x4b4, 18, V_SHOT_4B4),
    (Phase::Shot, 0x4b6, 18, V_SHOT_4B6),
    (Phase::Shot, 0x4b8, 18, V_SHOT_4B8),
    (Phase::Shot, 0x4b9, 18, V_SHOT_4B9),
    (Phase::Shot, 0x4ba, 18, V_SHOT_4BA),
    (Phase::Shot, 0x4bd, 18, V_SHOT_4BD),
    (Phase::Shot, 0x4c0, 18, V_SHOT_4C0),
    (Phase::Shot, 0x4c1, 18, V_SHOT_4C1),
    (Phase::Shot, 0x4c2, 18, V_SHOT_4C2),
    (Phase::Shot, 0x4c3, 18, V_SHOT_4C3),
    (Phase::Shot, 0x4c4, 1, V_SHOT_4C4),
    (Phase::Shot, 0x4c6, 18, V_SHOT_4C6),
    (Phase::Shot, 0x4c7, 18, V_SHOT_4C7),
    (Phase::Shot, 0x4c8, 18, V_SHOT_4C8),
    (Phase::Shot, 0x4cd, 18, V_SHOT_4CD),
    (Phase::Shot, 0x4ce, 1, V_SHOT_4CE),
    (Phase::Shot, 0x4cf, 18, V_SHOT_4CF),
    (Phase::Shot, 0x4d0, 18, V_SHOT_4D0),
    (Phase::Shot, 0x4d1, 18, V_SHOT_4D1),
    (Phase::Shot, 0x4d2, 2, V_SHOT_4D2),
    (Phase::Shot, 0x4d3, 18, V_SHOT_4D3),
    (Phase::Shot, 0x4d4, 18, V_SHOT_4D4),
    (Phase::Shot, 0x4d5, 0, V_SHOT_4D5),
    (Phase::Shot, 0x4d6, 18, V_SHOT_4D6),
    (Phase::Shot, 0x4d7, 0, V_SHOT_4D7),
    (Phase::Shot, 0x4d8, 18, V_SHOT_4D8),
    (Phase::Shot, 0x4d9, 18, V_SHOT_4D9),
    (Phase::Shot, 0x4da, 18, V_SHOT_4DA),
    (Phase::Shot, 0x4dc, 2, V_SHOT_4DC),
    (Phase::Shot, 0x4dd, 2, V_SHOT_4DD),
    (Phase::Shot, 0x4de, 18, V_SHOT_4DE),
    (Phase::Shot, 0x4df, 18, V_SHOT_4DF),
    (Phase::Shot, 0x4e0, 18, V_SHOT_4E0),
    (Phase::Shot, 0x4e1, 18, V_SHOT_4E1),
    (Phase::Shot, 0x4e2, 18, V_SHOT_4E2),
    (Phase::Shot, 0x4e3, 18, V_SHOT_4E3),
    (Phase::Shot, 0x4e4, 18, V_SHOT_4E4),
    (Phase::Shot, 0x4e5, 18, V_SHOT_4E5),
    (Phase::Shot, 0x4e6, 18, V_SHOT_4E6),
    (Phase::Shot, 0x4e7, 16, V_SHOT_4E7),
    (Phase::Shot, 0x4e8, 18, V_SHOT_4E8),
    (Phase::Shot, 0x4e9, 18, V_SHOT_4E9),
    (Phase::Shot, 0x4ea, 18, V_SHOT_4EA),
    (Phase::Shot, 0x4eb, 18, V_SHOT_4EB),
    (Phase::Shot, 0x4ec, 18, V_SHOT_4EC),
    (Phase::Shot, 0x4ed, 18, V_SHOT_4ED),
    (Phase::Shot, 0x4ef, 18, V_SHOT_4EF),
    (Phase::Shot, 0x4f0, 18, V_SHOT_4F0),
    (Phase::Shot, 0x4f1, 18, V_SHOT_4F1),
    (Phase::Shot, 0x4f2, 1, V_SHOT_4F2),
    (Phase::Shot, 0x4f3, 1, V_SHOT_4F3),
    (Phase::Shot, 0x4f4, 18, V_SHOT_4F4),
    (Phase::Shot, 0x4f5, 18, V_SHOT_4F5),
    (Phase::Shot, 0x4f7, 18, V_SHOT_4F7),
    (Phase::Shot, 0x4f8, 18, V_SHOT_4F8),
    (Phase::Shot, 0x4f9, 18, V_SHOT_4F9),
    (Phase::Shot, 0x4fa, 18, V_SHOT_4FA),
    (Phase::Shot, 0x4fb, 18, V_SHOT_4FB),
    (Phase::Shot, 0x502, 18, V_SHOT_502),
    (Phase::Shot, 0x504, 18, V_SHOT_504),
    (Phase::Shot, 0x505, 2, V_SHOT_505),
    (Phase::Shot, 0x506, 2, V_SHOT_506),
    (Phase::Shot, 0x507, 2, V_SHOT_507),
    (Phase::Shot, 0x508, 18, V_SHOT_508),
    (Phase::Shot, 0x509, 1, V_SHOT_509),
    (Phase::Shot, 0x50a, 0, V_SHOT_50A),
    (Phase::Shot, 0x50b, 0, V_SHOT_50B),
    (Phase::Shot, 0x50c, 18, V_SHOT_50C),
    (Phase::Shot, 0x50d, 18, V_SHOT_50D),
    (Phase::Shot, 0x50e, 18, V_SHOT_50E),
    (Phase::Shot, 0x552, 18, V_SHOT_552),
    (Phase::Shot, 0x553, 18, V_SHOT_553),
    (Phase::Shot, 0x554, 18, V_SHOT_554),
    (Phase::Shot, 0x555, 18, V_SHOT_555),
    (Phase::Shot, 0x556, 18, V_SHOT_556),
    (Phase::Shot, 0x557, 18, V_SHOT_557),
    (Phase::Shot, 0x7dd, 0, V_SHOT_7DD),
    (Phase::Shot, 0x7de, 18, V_SHOT_7DE),
    (Phase::Shot, 0x7f8, 0, V_SHOT_7F8),
    (Phase::Shot, 0x7f9, 0, V_SHOT_7F9),
    (Phase::Shot, 0x801, 18, V_SHOT_801),
    (Phase::Shot, 0x802, 18, V_SHOT_802),
    (Phase::Shot, 0x803, 18, V_SHOT_803),
    (Phase::Shot, 0x804, 18, V_SHOT_804),
    (Phase::Shot, 0x805, 18, V_SHOT_805),
    (Phase::Shot, 0x806, 18, V_SHOT_806),
    (Phase::Shot, 0x807, 18, V_SHOT_807),
    (Phase::Shot, 0x808, 18, V_SHOT_808),
    (Phase::Shot, 0x809, 18, V_SHOT_809),
    (Phase::Shot, 0x80a, 18, V_SHOT_80A),
    (Phase::Shot, 0x80b, 18, V_SHOT_80B),
    (Phase::Shot, 0x80e, 18, V_SHOT_80E),
    (Phase::Shot, 0x810, 18, V_SHOT_810),
    (Phase::Shot, 0x830, 2, V_SHOT_830),
    (Phase::Shot, 0x831, 18, V_SHOT_831),
    (Phase::Shot, 0x833, 16, V_SHOT_833),
    (Phase::Shot, 0x834, 16, V_SHOT_834),
    (Phase::Shot, 0xfa1, 1, V_SHOT_FA1),
    (Phase::Shot, 0xfa2, 2, V_SHOT_FA2),
    (Phase::Shot, 0xfb2, 0, V_SHOT_FB2),
    (Phase::Shot, 0xfb3, 18, V_SHOT_FB3),
    (Phase::Shot, 0xfb4, 16, V_SHOT_FB4),
    (Phase::Shot, 0xfb8, 16, V_SHOT_FB8),
    (Phase::Shot, 0xfb9, 2, V_SHOT_FB9),
    (Phase::Shot, 0xfba, 1, V_SHOT_FBA),
    (Phase::Shot, 0xfbb, 18, V_SHOT_FBB),
    (Phase::Shot, 0xfbc, 18, V_SHOT_FBC),
    (Phase::Shot, 0xfbd, 18, V_SHOT_FBD),
    (Phase::Shot, 0xfbe, 18, V_SHOT_FBE),
    (Phase::Shot, 0xfbf, 18, V_SHOT_FBF),
    (Phase::Shot, 0xfc0, 19, V_SHOT_FC0),
    (Phase::Shot, 0xfc1, 18, V_SHOT_FC1),
    (Phase::Shot, 0xfc2, 18, V_SHOT_FC2),
    (Phase::Shot, 0xfc3, 18, V_SHOT_FC3),
    (Phase::Shot, 0xfc4, 18, V_SHOT_FC4),
    (Phase::Shot, 0xfc5, 18, V_SHOT_FC5),
    (Phase::Shot, 0xfc6, 18, V_SHOT_FC6),
    (Phase::Shot, 0xfc7, 2, V_SHOT_FC7),
    (Phase::Shot, 0xfca, 1, V_SHOT_FCA),
    (Phase::Shot, 0xfcb, 1, V_SHOT_FCB),
    (Phase::Shot, 0xfcc, 18, V_SHOT_FCC),
    (Phase::Shot, 0xfce, 18, V_SHOT_FCE),
    (Phase::Shot, 0xfcf, 18, V_SHOT_FCF),
    (Phase::Shot, 0xfdf, 19, V_SHOT_FDF),
    (Phase::Shot, 0xfe0, 19, V_SHOT_FE0),
    (Phase::Shot, 0xfe1, 18, V_SHOT_FE1),
    (Phase::Shot, 0xfe2, 0, V_SHOT_FE2),
    (Phase::Shot, 0xfe3, 19, V_SHOT_FE3),
    (Phase::Shot, 0xfe4, 0, V_SHOT_FE4),
    (Phase::Shot, 0xfe5, 0, V_SHOT_FE5),
    (Phase::Shot, 0xfe8, 18, V_SHOT_FE8),
    (Phase::Shot, 0xfe9, 18, V_SHOT_FE9),
    (Phase::Shot, 0xfea, 18, V_SHOT_FEA),
    (Phase::Shot, 0xfeb, 18, V_SHOT_FEB),
    (Phase::Shot, 0xfec, 18, V_SHOT_FEC),
    (Phase::Shot, 0xfed, 18, V_SHOT_FED),
    (Phase::Shot, 0xfee, 1, V_SHOT_FEE),
    (Phase::Shot, 0xfef, 2, V_SHOT_FEF),
    (Phase::Shot, 0xff1, 2, V_SHOT_FF1),
    (Phase::Shot, 0xff2, 18, V_SHOT_FF2),
    (Phase::Shot, 0xff3, 19, V_SHOT_FF3),
    (Phase::Shot, 0xff7, 1, V_SHOT_FF7),
    (Phase::Shot, 0xff9, 18, V_SHOT_FF9),
    (Phase::Shot, 0xffa, 18, V_SHOT_FFA),
    (Phase::Shot, 0xffb, 18, V_SHOT_FFB),
    (Phase::Shot, 0xffc, 18, V_SHOT_FFC),
    (Phase::Shot, 0xffe, 18, V_SHOT_FFE),
    (Phase::Shot, 0x1000, 18, V_SHOT_1000),
    (Phase::Shot, 0x1001, 18, V_SHOT_1001),
    (Phase::Shot, 0x1002, 18, V_SHOT_1002),
    (Phase::Shot, 0x1003, 18, V_SHOT_1003),
    (Phase::Shot, 0x1004, 1, V_SHOT_1004),
    (Phase::Shot, 0x1006, 18, V_SHOT_1006),
    (Phase::Shot, 0x1007, 18, V_SHOT_1007),
    (Phase::Shot, 0x1008, 18, V_SHOT_1008),
    (Phase::Shot, 0x1009, 0, V_SHOT_1009),
    (Phase::Shot, 0x100a, 2, V_SHOT_100A),
    (Phase::Shot, 0x100b, 2, V_SHOT_100B),
    (Phase::Shot, 0x100d, 2, V_SHOT_100D),
    (Phase::Shot, 0x100e, 2, V_SHOT_100E),
    (Phase::Shot, 0x1012, 1, V_SHOT_1012),
    (Phase::Shot, 0x1013, 18, V_SHOT_1013),
    (Phase::Shot, 0x1015, 18, V_SHOT_1015),
    (Phase::Shot, 0x1016, 18, V_SHOT_1016),
    (Phase::Shot, 0x1017, 18, V_SHOT_1017),
    (Phase::Shot, 0x101b, 18, V_SHOT_101B),
    (Phase::Shot, 0x101c, 0, V_SHOT_101C),
    (Phase::Shot, 0x101f, 2, V_SHOT_101F),
    (Phase::Shot, 0x1021, 2, V_SHOT_1021),
    (Phase::Shot, 0x1022, 2, V_SHOT_1022),
    (Phase::Shot, 0x1023, 2, V_SHOT_1023),
    (Phase::Shot, 0x1024, 18, V_SHOT_1024),
    (Phase::Shot, 0x1025, 18, V_SHOT_1025),
    (Phase::Shot, 0x1029, 18, V_SHOT_1029),
    (Phase::Shot, 0x102a, 18, V_SHOT_102A),
    (Phase::Shot, 0x102b, 18, V_SHOT_102B),
    (Phase::Shot, 0x1030, 19, V_SHOT_1030),
    (Phase::Shot, 0x1037, 1, V_SHOT_1037),
    (Phase::Shot, 0x1038, 2, V_SHOT_1038),
    (Phase::Shot, 0x1039, 18, V_SHOT_1039),
    (Phase::Shot, 0x103a, 2, V_SHOT_103A),
    (Phase::Shot, 0x103b, 0, V_SHOT_103B),
    (Phase::Shot, 0x103c, 0, V_SHOT_103C),
    (Phase::Shot, 0x103d, 1, V_SHOT_103D),
    (Phase::Shot, 0x103e, 1, V_SHOT_103E),
    (Phase::Shot, 0x103f, 0, V_SHOT_103F),
    (Phase::Shot, 0x1040, 18, V_SHOT_1040),
    (Phase::Shot, 0x1043, 18, V_SHOT_1043),
    (Phase::Shot, 0x1048, 19, V_SHOT_1048),
    (Phase::Shot, 0x1049, 19, V_SHOT_1049),
    (Phase::Shot, 0x104a, 19, V_SHOT_104A),
    (Phase::Shot, 0x104b, 19, V_SHOT_104B),
    (Phase::Shot, 0x104c, 19, V_SHOT_104C),
    (Phase::Shot, 0x104d, 18, V_SHOT_104D),
    (Phase::Shot, 0x104e, 18, V_SHOT_104E),
    (Phase::Shot, 0x104f, 18, V_SHOT_104F),
    (Phase::Shot, 0x1050, 18, V_SHOT_1050),
    (Phase::Shot, 0x1051, 18, V_SHOT_1051),
    (Phase::Shot, 0x1052, 1, V_SHOT_1052),
    (Phase::Shot, 0x1053, 19, V_SHOT_1053),
    (Phase::Shot, 0x1054, 19, V_SHOT_1054),
    (Phase::Shot, 0x1055, 19, V_SHOT_1055),
    (Phase::Shot, 0x1056, 18, V_SHOT_1056),
    (Phase::Shot, 0x1057, 18, V_SHOT_1057),
    (Phase::Shot, 0x1058, 18, V_SHOT_1058),
    (Phase::Shot, 0x1059, 18, V_SHOT_1059),
    (Phase::Shot, 0x105a, 18, V_SHOT_105A),
    (Phase::Shot, 0x105b, 18, V_SHOT_105B),
    (Phase::Shot, 0x105c, 18, V_SHOT_105C),
    (Phase::Shot, 0x105d, 18, V_SHOT_105D),
    (Phase::Shot, 0x105e, 18, V_SHOT_105E),
    (Phase::Shot, 0x105f, 1, V_SHOT_105F),
    (Phase::Shot, 0x1060, 2, V_SHOT_1060),
    (Phase::Shot, 0x1061, 2, V_SHOT_1061),
    (Phase::Shot, 0x1062, 18, V_SHOT_1062),
    (Phase::Shot, 0x1063, 0, V_SHOT_1063),
    (Phase::Shot, 0x1064, 18, V_SHOT_1064),
    (Phase::Shot, 0x1065, 18, V_SHOT_1065),
    (Phase::Shot, 0x1066, 18, V_SHOT_1066),
    (Phase::Shot, 0x1067, 18, V_SHOT_1067),
    (Phase::Shot, 0x1068, 18, V_SHOT_1068),
    (Phase::Shot, 0x1069, 18, V_SHOT_1069),
    (Phase::Shot, 0x106a, 18, V_SHOT_106A),
    (Phase::Shot, 0x106b, 18, V_SHOT_106B),
    (Phase::Shot, 0x106c, 0, V_SHOT_106C),
    (Phase::Shot, 0x106d, 18, V_SHOT_106D),
    (Phase::Shot, 0x106e, 18, V_SHOT_106E),
    (Phase::Shot, 0x1070, 0, V_SHOT_1070),
    (Phase::Shot, 0x1071, 18, V_SHOT_1071),
    (Phase::Shot, 0x1072, 18, V_SHOT_1072),
    (Phase::Shot, 0x1073, 18, V_SHOT_1073),
    (Phase::Shot, 0x1074, 18, V_SHOT_1074),
    (Phase::Shot, 0x1075, 18, V_SHOT_1075),
    (Phase::Shot, 0x1076, 18, V_SHOT_1076),
    (Phase::Shot, 0x1078, 16, V_SHOT_1078),
    (Phase::Shot, 0x1079, 16, V_SHOT_1079),
    (Phase::Shot, 0x107a, 2, V_SHOT_107A),
    (Phase::Shot, 0x107b, 18, V_SHOT_107B),
    (Phase::Shot, 0x107c, 18, V_SHOT_107C),
    (Phase::Shot, 0x107e, 0, V_SHOT_107E),
    (Phase::Shot, 0x1087, 2, V_SHOT_1087),
    (Phase::Shot, 0x1088, 2, V_SHOT_1088),
    (Phase::Shot, 0x108b, 16, V_SHOT_108B),
    (Phase::Shot, 0x108c, 0, V_SHOT_108C),
    (Phase::Shot, 0x108d, 18, V_SHOT_108D),
    (Phase::Shot, 0x108e, 18, V_SHOT_108E),
    (Phase::Shot, 0x1094, 19, V_SHOT_1094),
    (Phase::Shot, 0x1097, 1, V_SHOT_1097),
    (Phase::Shot, 0x1098, 2, V_SHOT_1098),
    (Phase::Shot, 0x1099, 18, V_SHOT_1099),
    (Phase::Shot, 0x109a, 0, V_SHOT_109A),
    (Phase::Shot, 0x109b, 0, V_SHOT_109B),
    (Phase::Shot, 0x109c, 1, V_SHOT_109C),
    (Phase::Shot, 0x109d, 1, V_SHOT_109D),
    (Phase::Shot, 0x109e, 1, V_SHOT_109E),
    (Phase::Shot, 0x109f, 0, V_SHOT_109F),
    (Phase::Shot, 0x10a1, 18, V_SHOT_10A1),
    (Phase::Shot, 0x10a3, 18, V_SHOT_10A3),
    (Phase::Shot, 0x10a4, 2, V_SHOT_10A4),
    (Phase::Shot, 0x10a5, 1, V_SHOT_10A5),
    (Phase::Shot, 0x10da, 2, V_SHOT_10DA),
    (Phase::Shot, 0x10db, 2, V_SHOT_10DB),
    (Phase::Shot, 0x118f, 0, V_SHOT_118F),
    (Phase::Shot, 0x1190, 0, V_SHOT_1190),
    (Phase::Shot, 0x11f4, 2, V_SHOT_11F4),
    (Phase::Shot, 0x11f5, 2, V_SHOT_11F5),
    (Phase::Shot, 0x120d, 18, V_SHOT_120D),
    (Phase::Shot, 0x120e, 18, V_SHOT_120E),
    (Phase::Shot, 0x1219, 18, V_SHOT_1219),
    (Phase::Shot, 0x121f, 18, V_SHOT_121F),
    (Phase::Shot, 0x1231, 18, V_SHOT_1231),
    (Phase::Shot, 0x1232, 18, V_SHOT_1232),
    (Phase::Shot, 0x1234, 18, V_SHOT_1234),
    (Phase::Shot, 0x1235, 18, V_SHOT_1235),
    (Phase::Shot, 0x1238, 2, V_SHOT_1238),
    (Phase::Shot, 0x1239, 2, V_SHOT_1239),
    (Phase::Shot, 0x1251, 0, V_SHOT_1251),
    (Phase::Shot, 0x1252, 0, V_SHOT_1252),
    (Phase::Shot, 0x1298, 0, V_SHOT_1298),
    (Phase::Shot, 0x1299, 0, V_SHOT_1299),
];
