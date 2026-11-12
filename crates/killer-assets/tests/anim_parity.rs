use killer_assets::animation::{find_sequence, sequence_timing};
use killer_assets::client::Client;
use l2_dat::animnames::{AnimKind, AnimSet, RiderKind};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use ue2_assets::MeshAnimation;

fn test_client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

const PLAYER_SECTIONS: [&str; 14] = [
    "mfighter", "ffighter", "mdarkelf", "fdarkelf", "mdwarf", "fdwarf", "melf", "felf", "mmagic", "fmagic", "morc",
    "forc", "mshaman", "fshaman",
];

const BASIC_KINDS: [AnimKind; 9] = [
    AnimKind::Wait,
    AnimKind::AtkWait,
    AnimKind::Walk,
    AnimKind::Run,
    AnimKind::Sit,
    AnimKind::SitWait,
    AnimKind::Stand,
    AnimKind::Death,
    AnimKind::DeathWait,
];

#[derive(Default)]
struct Report {
    sections: usize,
    names: usize,
    missing: BTreeMap<String, BTreeSet<String>>,
    without_animation: BTreeSet<String>,
    shots: usize,
    shot_sequences: usize,
    max_notify_time: f32,
    bad_shot_times: BTreeSet<String>,
    classes: BTreeMap<String, usize>,
    properties: BTreeMap<String, BTreeMap<String, usize>>,
    chains: BTreeSet<String>,
}

impl Report {
    fn check(&mut self, section: &str, set: &AnimSet, anim: &MeshAnimation) {
        self.sections += 1;
        for name in set.all_names() {
            self.names += 1;
            if find_sequence(anim, name).is_none() {
                self.missing.entry(section.to_string()).or_default().insert(name.to_string());
            }
        }
        for sequence in &anim.sequences {
            let timing = sequence_timing(sequence);
            if !timing.attack_shots.is_empty() {
                self.shot_sequences += 1;
                self.shots += timing.attack_shots.len();
            }
            for notify in &sequence.notifies {
                if notify.class.eq_ignore_ascii_case("AnimNotify_Effect") {
                    let class = notify.properties.iter().find(|p| p.name == "EffectClass").map(|p| format!("{:?}", p.value)).unwrap_or_default();
                    let bone = notify.properties.iter().find(|p| p.name == "Bone").map(|p| format!("{:?}", p.value)).unwrap_or_default();
                    eprintln!("EFEITO {class} {bone}");
                }
                self.chains.insert(notify.classes.iter().map(|class| class.name.as_str()).collect::<Vec<_>>().join(">"));
                for property in &notify.properties {
                    let kind = format!("{}:{}", property.name, format!("{:?}", property.value).split('(').next().unwrap_or_default());
                    *self.properties.entry(notify.class.to_ascii_lowercase()).or_default().entry(kind).or_default() += 1;
                }
                *self.classes.entry(notify.class.to_ascii_lowercase()).or_default() += 1;
                self.max_notify_time = self.max_notify_time.max(notify.time);
                if !(0.0..=1.0).contains(&notify.time) {
                    self.bad_shot_times.insert(format!("{section}:{}", sequence.name));
                }
            }
        }
    }

    fn print(&self, title: &str) {
        eprintln!("== {title}: {} secoes, {} nomes conferidos", self.sections, self.names);
        let missing: usize = self.missing.values().map(BTreeSet::len).sum();
        eprintln!("   faltando na MeshAnimation: {missing} nomes em {} secoes", self.missing.len());
        for (section, names) in &self.missing {
            eprintln!("   {section}: {}", names.iter().cloned().collect::<Vec<_>>().join(", "));
        }
        if !self.without_animation.is_empty() {
            eprintln!(
                "   sem MeshAnimation carregavel: {}",
                self.without_animation.iter().cloned().collect::<Vec<_>>().join(", ")
            );
        }
        eprintln!("   classes de notify: {:?}", self.classes);
        eprintln!("   cadeias: {:?}", self.chains.iter().collect::<Vec<_>>());
        for (class, kinds) in self.properties.iter().filter(|(class, _)| !class.starts_with("animnotify_attackshot")) {
            eprintln!("   {class}: {kinds:?}");
        }
        eprintln!(
            "   AttackShot: {} notifies em {} sequencias; maior tempo de notify {}; fora de [0, 1]: {}",
            self.shots,
            self.shot_sequences,
            self.max_notify_time,
            self.bad_shot_times.len()
        );
    }
}

fn animation_of(client: &mut Client, meshes: &[&str]) -> Option<MeshAnimation> {
    let name = meshes.iter().find_map(|mesh| client.catalog().skeletal_mesh(mesh)?.animation)?;
    client.catalog().mesh_animation(&name)
}

#[test]
#[ignore]
fn every_player_sequence_the_port_can_request_exists_in_the_class_animation() {
    let Some(mut client) = test_client() else { return };
    let tables = client.tables().unwrap();
    let mut report = Report::default();
    for (row, section) in PLAYER_SECTIONS.iter().enumerate() {
        let set = &tables.anim_names[*section];
        let template = &tables.chargrp[row];
        let meshes: Vec<&str> = template
            .body_meshes
            .iter()
            .map(String::as_str)
            .chain(template.face_meshes.iter().map(String::as_str))
            .collect();
        let anim = animation_of(&mut client, &meshes).unwrap_or_else(|| panic!("{section}: {meshes:?}"));
        report.check(section, set, &anim);
        for kind in BASIC_KINDS {
            for weapon in [0, 1, 2, 3, 4, 5, 7] {
                let name = set.name(kind, weapon).unwrap_or_else(|| panic!("{section} {kind:?}[{weapon}]"));
                assert!(find_sequence(&anim, name).is_some(), "{section}: {name}");
            }
        }
        for weapon in [0, 1, 2, 3, 4, 5, 7] {
            for kind in [AnimKind::Atk01, AnimKind::Atk02, AnimKind::Atk03] {
                let name = set.name(kind, weapon).unwrap();
                let sequence = find_sequence(&anim, name).unwrap_or_else(|| panic!("{section}: {name}"));
                assert!(!sequence_timing(sequence).attack_shots.is_empty(), "{section}: {name} sem AttackShot");
            }
        }
        for id in 2..=13 {
            assert!(set.pc_social[id].is_some(), "{section}: PcSocialAnimName[{id}]");
        }
    }
    report.print("jogadores");
    let known: BTreeMap<String, BTreeSet<String>> = [
        ("fshaman".to_string(), BTreeSet::from(["DamageFly_FShaman".to_string()])),
        ("melf".to_string(), BTreeSet::from(["shieldatk1HS_MElf".to_string()])),
    ]
    .into_iter()
    .collect();
    assert_eq!(report.missing, known);
    assert!(report.bad_shot_times.is_empty());
    assert!(report.shot_sequences >= 400);
}

#[test]
#[ignore]
fn npc_sequences_named_by_the_int_exist_in_the_npc_animations() {
    let Some(mut client) = test_client() else { return };
    let tables = client.tables().unwrap();
    let mut report = Report::default();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut without_section = BTreeSet::new();
    for npc in tables.npcgrp.iter() {
        let section = npc.class.rsplit('.').next().unwrap_or(&npc.class).to_ascii_lowercase();
        if !seen.insert((section.clone(), npc.mesh.to_ascii_lowercase())) {
            continue;
        }
        let Some(set) = tables.anim_names.get(&section) else {
            without_section.insert(section);
            continue;
        };
        match animation_of(&mut client, &[npc.mesh.as_str()]) {
            Some(anim) => report.check(&section, set, &anim),
            None => {
                report.without_animation.insert(npc.mesh.clone());
            }
        }
    }
    report.print("npcs");
    eprintln!(
        "   classes sem secao no .int: {}: {}",
        without_section.len(),
        without_section.iter().cloned().collect::<Vec<_>>().join(", ")
    );
    let basic_missing: Vec<&String> = report
        .missing
        .iter()
        .filter(|(_, names)| {
            names.iter().any(|n| {
                ["wait", "run", "walk", "death", "deathwait", "atkwait"].contains(&n.to_ascii_lowercase().as_str())
            })
        })
        .map(|(section, _)| section)
        .collect();
    eprintln!("   secoes sem algum estado basico: {}", basic_missing.len());
    assert!(report.sections > 500, "{}", report.sections);
    assert!(report.without_animation.is_empty(), "{:?}", report.without_animation);
    let known: BTreeMap<String, BTreeSet<String>> =
        [("dire_wolf".to_string(), BTreeSet::from(["spwait01".to_string()]))].into_iter().collect();
    assert_eq!(report.missing, known);
    assert!(report.bad_shot_times.is_empty());
}

#[test]
#[ignore]
fn anim_notify_class_defaults_come_from_engine_u() {
    let Some(mut client) = test_client() else { return };
    let file = client.read_system_file("Engine.u").unwrap();
    let package = ue2_package::Package::parse(&file).unwrap();
    for export in package.exports().iter().filter(|export| package.class_name(export) == "Class") {
        let name = package.name(export.name);
        if name.starts_with("AnimNotify") {
            let defaults = ue2_assets::read_class_notify_defaults(&package, &file, export);
            let value = |property: &str| defaults.iter().find(|candidate| candidate.name == property).map(|candidate| candidate.value.clone());
            match name {
                "AnimNotify_Sound" => {
                    assert_eq!(value("Random"), Some(ue2_assets::NotifyValue::Int(100)));
                    assert_eq!(value("Volume"), Some(ue2_assets::NotifyValue::Float(1.0)));
                }
                "AnimNotify_Effect" => {
                    assert_eq!(value("EffectScale"), Some(ue2_assets::NotifyValue::Float(1.0)));
                    assert_eq!(value("DrawScale3D"), Some(ue2_assets::NotifyValue::Vector([1.0; 3])));
                }
                "AnimNotify_ScreenFade" => assert_eq!(value("FadeInDuration"), Some(ue2_assets::NotifyValue::Float(1000.0))),
                _ => {}
            }
        }
    }
}

#[test]
#[ignore]
fn lineage_effect_class_defaults_for_tag() {
    let Some(mut client) = test_client() else { return };
    let file = client.read_system_file("LineageEffect.u").unwrap();
    let package = ue2_package::Package::parse(&file).unwrap();
    let mut tags = 0;
    let mut total = 0;
    for export in package.exports().iter().filter(|export| package.class_name(export) == "Class") {
        let name = package.name(export.name);
        let defaults = ue2_assets::read_class_notify_defaults(&package, &file, export);
        total += 1;
        if let Some(tag) = defaults.iter().find(|candidate| candidate.name == "Tag") {
            tags += 1;
            eprintln!("TAGV {name}: {:?}", tag.value);
        }
    }
    eprintln!("TAG classes {total} com Tag {tags}");
}

#[test]
#[ignore]
fn alaudio_int_sections_for_footsteps_and_swimming() {
    let Some(mut client) = test_client() else { return };
    let text = client.read_system_text("alaudio.int").unwrap();
    let mut section = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            section = line.to_owned();
        }
        if section.eq_ignore_ascii_case("[L2FootStepSound]") || section.eq_ignore_ascii_case("[SwimSound]") || line.starts_with('[') {
            eprintln!("ALAUDIO {line}");
        }
    }
}

#[test]
#[ignore]
fn pawn_sound_columns_of_the_dats_are_read() {
    let Some(mut client) = test_client() else { return };
    let tables = client.tables().unwrap();
    for (row, template) in tables.chargrp.iter().enumerate() {
        let voices: Vec<usize> = template.voice_sounds.iter().map(Vec::len).collect();
        eprintln!("CHAR {row}: ataque {} defesa {} dano {} vozes {:?} primeira voz {:?}", template.attack_sounds.len(), template.defense_sounds.len(), template.damage_sounds.len(), voices, template.voice_sounds.iter().find_map(|group| group.first()));
        assert!(template.attack_sounds.len() <= 3 && template.defense_sounds.len() <= 5 && template.damage_sounds.len() <= 3);
        assert!(voices.iter().all(|&count| count <= 5));
    }
    let with_attack = tables.npcgrp.iter().filter(|npc| !npc.attack_sounds.is_empty()).count();
    eprintln!("NPC {} com som de ataque de {}", with_attack, tables.npcgrp.len());
    let weapons = tables.weapongrp.values().filter(|weapon| !weapon.attack_sounds.is_empty()).count();
    eprintln!("ARMAS {} com som de ataque de {}", weapons, tables.weapongrp.len());
    assert!(with_attack > 100 && weapons > 100);
}

#[test]
#[ignore]
fn game_engine_tick_rate_in_the_ini_files() {
    let Some(mut client) = test_client() else { return };
    for name in ["l2.ini", "option.ini", "user.ini"] {
        let text = client.read_system_text(name).unwrap();
        for (index, line) in text.lines().enumerate() {
            if line.to_ascii_lowercase().contains("tickrate") || line.to_ascii_lowercase().contains("framerate") {
                eprintln!("INI {name}:{index} {line}");
            }
        }
    }
}

const MOUNT_KINDS: [AnimKind; 8] = [
    AnimKind::Wait,
    AnimKind::AtkWait,
    AnimKind::Walk,
    AnimKind::Run,
    AnimKind::Atk01,
    AnimKind::Atk02,
    AnimKind::Atk03,
    AnimKind::Death,
];

#[test]
#[ignore]
fn riders_and_mounts_have_every_sequence_and_the_attach_bone() {
    let Some(mut client) = test_client() else { return };
    let tables = client.tables().unwrap();
    for (row, section) in PLAYER_SECTIONS.iter().enumerate() {
        let set = &tables.anim_names[*section];
        let template = &tables.chargrp[row];
        let meshes: Vec<&str> = template.body_meshes.iter().map(String::as_str).chain(template.face_meshes.iter().map(String::as_str)).collect();
        let anim = animation_of(&mut client, &meshes).unwrap();
        for ride in [1, 2] {
            for kind in [RiderKind::Wait, RiderKind::Run, RiderKind::Atk, RiderKind::Death, RiderKind::DeathWait] {
                let name = set.rider(kind, ride).unwrap_or_else(|| panic!("{section} {kind:?}[{ride}]"));
                assert!(find_sequence(&anim, name).is_some(), "{section}: {name}");
            }
        }
    }
    let mut mounts = 0;
    for npc in tables.npcgrp.iter() {
        let class = npc.class.rsplit('.').next().unwrap_or(&npc.class).to_ascii_lowercase();
        if class != "strider" && class != "wyvern" {
            continue;
        }
        mounts += 1;
        let set = tables.anim_names.get(&class).unwrap_or_else(|| panic!("{class}: sem secao"));
        let skinned = client.catalog().skeletal_mesh(&npc.mesh).unwrap_or_else(|| panic!("{}: malha", npc.mesh));
        assert!(skinned.mesh.match_ref_bone("Bone15").is_some(), "{}: sem Bone15", npc.mesh);
        let anim = animation_of(&mut client, &[npc.mesh.as_str()]).unwrap();
        for kind in MOUNT_KINDS {
            let name = set.name(kind, 0).unwrap_or_else(|| panic!("{class} {kind:?}"));
            assert!(find_sequence(&anim, name).is_some(), "{class}: {name}");
        }
    }
    eprintln!("   montarias strider/wyvern no npcgrp: {mounts}");
    assert!(mounts > 0);
}
