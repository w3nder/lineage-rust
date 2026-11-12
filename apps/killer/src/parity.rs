use crate::characters::AppearanceKey;
use crate::characters::{Collision, LoadedCharacter, PartKind};
use crate::loader::{LoadResult, Shared, load_character};
use glam::{Mat4, Vec3};
use killer_assets::MissingAsset;
use killer_assets::characters::{CharacterMesh, load_part};
use killer_assets::client::{Client, DatTables};
use killer_assets::pawn::{
    Appearance, Part, Placement, Player, SLOT_BODY, SLOT_BOOTS, SLOT_FACE, SLOT_GLOVES, SLOT_HAIR_A, SLOT_HAIR_B,
    SLOT_HELM, SLOT_LEFT_ARM, SLOT_LEFT_HAND, SLOT_LEGS, SLOT_RIGHT_HAND,
};
use killer_assets::textures::TextureBank;
use killer_render::skinned::CharacterId;
use l2_dat::animnames::AnimKind;
use l2_dat::armorgrp::body_part;
use l2_dat::weapongrp::anim_type;
use l2_dat::{Equipped, Weapon};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;

const CLASSES: [(usize, &str, f32, f32); 14] = [
    (0, "mfighter", 9.0, 23.0),
    (1, "ffighter", 8.0, 23.5),
    (2, "mdarkelf", 7.5, 24.0),
    (3, "fdarkelf", 7.0, 23.5),
    (4, "mdwarf", 9.0, 18.0),
    (5, "fdwarf", 5.0, 19.0),
    (6, "melf", 7.5, 24.0),
    (7, "felf", 7.5, 23.0),
    (8, "mmagic", 7.5, 22.8),
    (9, "fmagic", 6.5, 22.5),
    (10, "morc", 11.0, 28.0),
    (11, "forc", 7.0, 27.0),
    (12, "mshaman", 7.0, 27.0),
    (13, "fshaman", 8.0, 25.5),
];
const ENCRYPTED_PACKAGES: [&str; 2] = ["l2killer_headgear", "l2killer_assassin_light"];
const GRIP_RADIUS: f32 = 3.0;
const CLUSTER_DEGREES: f32 = 35.0;
const HEAD_RADIUS: f32 = 30.0;

fn client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

#[derive(Default)]
struct Report {
    data: BTreeMap<String, Vec<String>>,
    code: Vec<String>,
    counts: BTreeMap<String, usize>,
}

impl Report {
    fn data(&mut self, kind: &str, what: String) {
        self.data.entry(kind.to_string()).or_default().push(what);
    }

    fn count(&mut self, what: &str) {
        *self.counts.entry(what.to_string()).or_default() += 1;
    }

    fn code(&mut self, what: String) {
        self.code.push(what);
    }

    fn print(&self, title: &str) {
        let mut out = String::new();
        writeln!(out, "== {title}").unwrap();
        for (what, count) in &self.counts {
            writeln!(out, "   {what}: {count}").unwrap();
        }
        for (kind, items) in &self.data {
            let mut unique: Vec<&String> = items.iter().collect::<BTreeSet<_>>().into_iter().collect();
            unique.sort();
            writeln!(out, "   por dado [{kind}]: {} ({} únicos)", items.len(), unique.len()).unwrap();
            for item in unique.iter().take(40) {
                writeln!(out, "      {item}").unwrap();
            }
        }
        writeln!(out, "   por código: {}", self.code.len()).unwrap();
        for item in &self.code {
            writeln!(out, "      {item}").unwrap();
        }
        eprint!("{out}");
    }
}

fn player<'a>(tables: &'a DatTables, row: usize, hair_style: i32, equipped: Equipped) -> Player<'a> {
    Player {
        template: &tables.chargrp[row],
        row,
        face: 0,
        hair_style,
        hair_color: 0,
        equipped,
        tables: tables.pawn_tables(),
        mounted: None,
        korean: false,
        korean_flag: false,
        clan_crest: None,
    }
}

fn mesh_failure(client: &mut Client, mesh: &str) -> (String, String) {
    let package = mesh.split('.').next().unwrap_or(mesh).to_ascii_lowercase();
    if ENCRYPTED_PACKAGES.contains(&package.as_str()) {
        return ("CAFEF00D".to_string(), mesh.to_string());
    }
    match client.catalog().try_skeletal_mesh(mesh) {
        Ok(_) => ("carregou".to_string(), mesh.to_string()),
        Err(MissingAsset::Undecodable { version, licensee, error }) => {
            (format!("indecodificável v{version} licensee {licensee}"), format!("{mesh}: {error}"))
        }
        Err(why) => (why.to_string(), mesh.to_string()),
    }
}

fn load(client: &mut Client, part: &Part, report: &mut Report) -> Option<CharacterMesh> {
    match load_part(client, part) {
        Some(mesh) => Some(mesh),
        None => {
            let (kind, what) = mesh_failure(client, &part.mesh);
            report.data(&kind, what);
            None
        }
    }
}

fn character(
    client: &mut Client,
    shared: &mut Shared,
    key: AppearanceKey,
    appearance: &Appearance,
) -> Option<LoadedCharacter> {
    match load_character(client, shared, key, appearance, 1.0) {
        LoadResult::Character { parts, animation, draw_scale, weapon_type, .. } => {
            let parts = parts.into_iter().enumerate().map(|(i, m)| (CharacterId(i), Arc::new(m))).collect();
            LoadedCharacter::new(parts, animation, draw_scale, weapon_type)
        }
        _ => None,
    }
}

fn master_of(
    client: &mut Client,
    tables: &DatTables,
    row: usize,
) -> Option<(CharacterMesh, Arc<ue2_assets::MeshAnimation>)> {
    let appearance = player(tables, row, 0, Equipped::default()).set_pawn_resource();
    let face = appearance.part(SLOT_FACE)?;
    let mesh = load_part(client, face)?;
    let animation = Arc::new(client.catalog().mesh_animation(mesh.animation.as_deref()?)?);
    Some((mesh, animation))
}

fn wait_sequence(
    character: &mut LoadedCharacter,
    tables: &DatTables,
    section: &str,
    weapon_type: usize,
) -> Option<usize> {
    let name = tables.anim_names.get(section)?.name(AnimKind::Wait, weapon_type)?;
    character.sequence(name)
}

fn bone_names_match(a: &str, b: &str) -> bool {
    a.chars()
        .map(|c| if c == ' ' { '_' } else { c.to_ascii_lowercase() })
        .eq(b.chars().map(|c| if c == ' ' { '_' } else { c.to_ascii_lowercase() }))
}

fn skinning_parity(mesh: &CharacterMesh, master: &CharacterMesh, label: &str, report: &mut Report) -> bool {
    let mut used: BTreeSet<usize> = BTreeSet::new();
    let mut root = 0usize;
    for vertex in &mesh.vertices {
        for (bone, weight) in vertex.bones.iter().zip(vertex.weights) {
            if weight > 0.0 {
                used.insert(usize::from(*bone));
                if *bone == 0 {
                    root += 1;
                }
            }
        }
    }
    let mut ok = true;
    for bone in used {
        match (mesh.bones.get(bone), master.bones.get(bone)) {
            (Some(own), Some(masters)) if bone_names_match(&own.name, &masters.name) => {}
            (Some(own), Some(masters))
                if own.parent == masters.parent
                    && Vec3::from(own.position).abs_diff_eq(Vec3::from(masters.position), 1e-3) =>
            {
                report.data(
                    "osso com outro nome na mesma posição",
                    format!("{label}: osso {bone} {} no mestre é {}", own.name, masters.name),
                );
            }
            (Some(own), Some(masters)) => {
                report.data("osso trocado", format!("{label}: osso {bone} {} no mestre é {}", own.name, masters.name));
                ok = false;
            }
            _ => {
                report.data("osso fora do mestre", format!("{label}: osso {bone} de {}", mesh.bones.len()));
                ok = false;
            }
        }
    }
    if root > 0 && mesh.bones.len() > 1 {
        report.data("peso na raiz", format!("{label}: {root} vértices no osso 0"));
    }
    ok
}

fn attached_world(character: &LoadedCharacter, part: usize, pose: &[Mat4], collision: Collision) -> Option<Mat4> {
    let palette = character.palette(part, pose, collision)?;
    Some(character.world(part, Vec3::ZERO, 0) * palette[0])
}

fn bone_world(character: &LoadedCharacter, pose: &[Mat4], bone: usize) -> Vec3 {
    (character.world(character.master(), Vec3::ZERO, 0) * pose[bone]).transform_point3(Vec3::ZERO)
}

fn slot_for(weapon: &Weapon, index: usize) -> (u8, Equipped) {
    let id = weapon.id as i32;
    match weapon.body_part {
        body_part::LHAND => (SLOT_LEFT_ARM, Equipped { lhand: id, ..Default::default() }),
        body_part::LRHAND if weapon.handness == anim_type::BOW => {
            (SLOT_LEFT_HAND, Equipped { lrhand: id, ..Default::default() })
        }
        body_part::LRHAND if index == 1 => (SLOT_LEFT_HAND, Equipped { lrhand: id, ..Default::default() }),
        body_part::LRHAND => (SLOT_RIGHT_HAND, Equipped { lrhand: id, ..Default::default() }),
        _ => (SLOT_RIGHT_HAND, Equipped { rhand: id, ..Default::default() }),
    }
}

fn long_axis(mesh: &CharacterMesh) -> (Vec3, f32) {
    let (mut low, mut high) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for v in &mesh.vertices {
        low = low.min(Vec3::from(v.position));
        high = high.max(Vec3::from(v.position));
    }
    let extent = high - low;
    let axis = if extent.x >= extent.y && extent.x >= extent.z {
        Vec3::X
    } else if extent.y >= extent.z {
        Vec3::Y
    } else {
        Vec3::Z
    };
    let signed = if (high * axis).length() >= (low * axis).length() { axis } else { -axis };
    (signed, extent.max_element())
}

type Direction = (String, Vec3, f32);

fn handness_name(handness: u32) -> &'static str {
    match handness {
        anim_type::HAND => "Hand",
        anim_type::ONE_HAND_SWORD => "1HS",
        anim_type::TWO_HAND_SWORD => "2HS",
        anim_type::DUAL => "Dual",
        anim_type::POLE => "Pole",
        anim_type::BOW => "Bow",
        anim_type::THROW => "Throw",
        anim_type::DUAL_FIST => "DualFist",
        _ => "?",
    }
}

#[test]
#[ignore]
fn every_weapon_mesh_attaches_to_its_bone_with_a_coherent_orientation() {
    let Some(mut client) = client() else { return };
    let tables = client.tables().unwrap();
    let mut shared = Shared::default();
    let mut report = Report::default();
    let mut directions: BTreeMap<(u32, u8), Vec<Direction>> = BTreeMap::new();
    let mut weapons: Vec<&Weapon> = tables.weapongrp.values().collect();
    weapons.sort_by_key(|w| w.id);
    let collision = Collision { radius: 9.0, height: 23.0 };
    let mut meshes_seen: BTreeSet<String> = BTreeSet::new();
    for weapon in weapons {
        for index in 0..weapon.meshes.len() {
            if weapon.meshes[index].is_empty() || !meshes_seen.insert(weapon.meshes[index].to_ascii_lowercase()) {
                continue;
            }
            report.count("malhas do weapongrp");
            let (_, equipped) = slot_for(weapon, index);
            let appearance = player(&tables, 0, 0, equipped).set_pawn_resource();
            let placed = appearance
                .parts
                .iter()
                .find(|p| p.slot >= SLOT_RIGHT_HAND && p.mesh.eq_ignore_ascii_case(&weapon.meshes[index]));
            let Some(part) = placed else {
                report.code(format!(
                    "{} não entrou em nenhum slot ({:?})",
                    weapon.meshes[index],
                    appearance.parts.iter().map(|p| (p.slot, p.mesh.clone())).collect::<Vec<_>>()
                ));
                continue;
            };
            let slot = part.slot;
            if matches!(part.placement, Placement::Extra { .. }) {
                report.count("ex-meshes (varas/draconic)");
            }
            let Some(mesh) = load(&mut client, part, &mut report) else { continue };
            let key = AppearanceKey::Player { row: 0, face: 0, hair_style: 0, hair_color: 0, equipped, enchant_effect: 0, hero: false, mounted: 0 };
            let Some(character) = character(&mut client, &mut shared, key, &appearance) else {
                report.code(format!("{} não montou o personagem", part.mesh));
                continue;
            };
            let part_index = character
                .parts()
                .iter()
                .position(|(_, m)| m.slot == slot && m.placement != Placement::Skinned)
                .unwrap();
            let (bone, dual_fist) = match character.kind(part_index) {
                Some(PartKind::Attached { bone: Some(bone), dual_fist, .. }) => (bone, dual_fist),
                other => {
                    report.code(format!("{}: sem osso ({other:?})", part.mesh));
                    continue;
                }
            };
            let mut character = character;
            let weapon_type = appearance.weapon_type as usize;
            if tables.anim_names.get("mfighter").and_then(|names| names.name(AnimKind::Wait, weapon_type)).is_none() {
                report.data(
                    "tabela de nomes sem Wait para o tipo",
                    format!("{}: tipo {weapon_type} sem WaitAnimName em MFighter", part.mesh),
                );
                continue;
            }
            let Some(wait) = wait_sequence(&mut character, &tables, "mfighter", weapon_type) else {
                report.code(format!("{}: sem Wait para o tipo {}", part.mesh, appearance.weapon_type));
                continue;
            };
            let pose = character.frame_pose(wait, 0.0);
            let Some(world) = attached_world(&character, part_index, &pose, collision) else {
                report.code(format!("{}: sem paleta", part.mesh));
                continue;
            };
            let grip = world.transform_point3(mesh.origin);
            let bone_at = bone_world(&character, &pose, bone);
            let expected_offset = dual_fist.map_or(0.0, |d| d.along.abs() * character.scale());
            let offset = (grip - bone_at).length();
            if (offset - expected_offset).abs() > 1e-3 {
                report.code(format!("{}: origem a {offset:.3} do osso (esperado {expected_offset:.3})", part.mesh));
            }
            if dual_fist.is_some() {
                report.count("punhos duplos (deslocamento e_hand)");
            }
            let nearest = mesh.vertices.iter().map(|v| Vec3::from(v.position).length()).fold(f32::MAX, f32::min);
            if nearest > GRIP_RADIUS {
                report.data("punho longe da origem", format!("{}: vértice mais próximo a {nearest:.1}", part.mesh));
            }
            let (axis, length) = long_axis(&mesh);
            let direction = world.transform_vector3(axis).normalize();
            directions.entry((weapon.handness, slot)).or_default().push((part.mesh.clone(), direction, length));
            report.count("malhas anexadas");
        }
    }
    for ((handness, slot), entries) in &directions {
        let mean = entries.iter().map(|(_, d, _)| *d).sum::<Vec3>().normalize_or_zero();
        let mut outliers = 0;
        for (mesh, direction, _) in entries {
            let degrees = direction.dot(mean).clamp(-1.0, 1.0).acos().to_degrees();
            if degrees > CLUSTER_DEGREES {
                outliers += 1;
                report.data(
                    "orientação fora do grupo",
                    format!("{mesh} ({}, slot {slot}): {degrees:.0}° do grupo", handness_name(*handness)),
                );
            }
        }
        let elevation = mean.z.clamp(-1.0, 1.0).asin().to_degrees();
        eprintln!(
            "   Wait_{} slot {slot}: {} malhas, eixo longo médio {:?} (elevação {elevation:.0}°), {outliers} fora de {CLUSTER_DEGREES}°",
            handness_name(*handness),
            entries.len(),
            mean
        );
    }
    report.print("weapongrp");
    assert!(report.code.is_empty(), "{:?}", report.code);
    assert!(report.counts["malhas anexadas"] >= 300);
}

#[test]
#[ignore]
fn every_armor_row_skins_on_its_class_skeleton() {
    let Some(mut client) = client() else { return };
    let tables = client.tables().unwrap();
    let mut report = Report::default();
    let mut masters = Vec::new();
    for (row, _, _, _) in CLASSES {
        masters.push(master_of(&mut client, &tables, row).map(|(m, _)| m));
    }
    let mut armors: Vec<_> = tables.armorgrp.values().collect();
    armors.sort_by_key(|a| a.id);
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for armor in armors {
        for (row, _, _, _) in CLASSES {
            let Some(master) = &masters[row] else { continue };
            let Some(armor_row) = armor.rows.get(row) else { continue };
            for (index, mesh) in armor_row.meshes.iter().enumerate() {
                if mesh.is_empty() || !seen.insert(mesh.to_ascii_lowercase()) {
                    continue;
                }
                report.count("malhas do armorgrp");
                if armor.body_part == body_part::BACK {
                    report.count("mantos (o engine C4 não os desenha)");
                    continue;
                }
                let accessory = mesh.to_ascii_lowercase().ends_with("_a");
                let slot = match (armor.body_part, index) {
                    (body_part::HAIR | body_part::HEAD, _) => SLOT_HELM,
                    (part, _) if part > body_part::ALLDRESS || (part == body_part::ALLDRESS && accessory) => SLOT_HELM,
                    (body_part::GLOVES, _) | (body_part::ALLDRESS, 2) => SLOT_GLOVES,
                    (body_part::FEET, _) | (body_part::ALLDRESS, 3) => SLOT_BOOTS,
                    (body_part::LEGS, _) | (body_part::ONEPIECE | body_part::ALLDRESS, 1) => SLOT_LEGS,
                    _ => SLOT_BODY,
                };
                let part = Part {
                    slot,
                    mesh: mesh.clone(),
                    textures: Part::new(mesh.clone(), armor_row.textures.clone()).textures,
                    placement: if slot == SLOT_HELM {
                        Placement::Helm { offset: [0.0; 3], rotation: [0; 3] }
                    } else {
                        Placement::Skinned
                    },
                };
                let Some(loaded) = load(&mut client, &part, &mut report) else { continue };
                report.count("malhas carregadas");
                if slot == SLOT_HELM {
                    report.count("elmos/acessórios (anexados à cabeça)");
                    continue;
                }
                if skinning_parity(
                    &loaded,
                    master,
                    &format!("{mesh} (linha {row}, item {} parte {})", armor.id, armor.body_part),
                    &mut report,
                ) {
                    report.count("malhas com pesos só em ossos do mestre");
                }
            }
        }
    }
    report.print("armorgrp");
    assert!(report.code.is_empty(), "{:?}", report.code);
    assert!(report.counts["malhas com pesos só em ossos do mestre"] > 600);
}

#[test]
#[ignore]
fn every_hair_style_and_head_accessory_lands_on_the_head() {
    let Some(mut client) = client() else { return };
    let tables = client.tables().unwrap();
    let mut shared = Shared::default();
    let mut report = Report::default();
    let mut accessories: Vec<_> =
        tables.armorgrp.values().filter(|a| a.body_part == body_part::HAIR || a.body_part == body_part::HEAD).collect();
    accessories.sort_by_key(|a| a.id);
    let mut hair_seen: BTreeSet<String> = BTreeSet::new();
    let mut accessory_seen: BTreeSet<String> = BTreeSet::new();
    for (row, section, radius, height) in CLASSES {
        let Some((master, _)) = master_of(&mut client, &tables, row) else { continue };
        let collision = Collision { radius, height };
        let mut looks: Vec<(Equipped, i32)> = (0..15).map(|style| (Equipped::default(), style)).collect();
        for accessory in &accessories {
            if accessory.rows.get(row).is_some_and(|r| r.meshes.iter().any(|m| !m.is_empty())) {
                let equipped = if accessory.body_part == body_part::HAIR {
                    Equipped { hair: accessory.id as i32, ..Default::default() }
                } else {
                    Equipped { head: accessory.id as i32, ..Default::default() }
                };
                looks.extend((0..15).map(|style| (equipped, style)));
            }
        }
        for (equipped, style) in looks {
            let appearance = player(&tables, row, style, equipped).set_pawn_resource();
            for part in &appearance.parts {
                let hair = part.slot == SLOT_HAIR_A || part.slot == SLOT_HAIR_B;
                if hair && !hair_seen.insert(part.mesh.to_ascii_lowercase()) {
                    continue;
                }
                if part.slot == SLOT_HELM
                    && !accessory_seen.insert(format!("{}#{style}", part.mesh.to_ascii_lowercase()))
                {
                    continue;
                }
                if !hair && part.slot != SLOT_HELM {
                    continue;
                }
                report.count(if hair { "cabelos" } else { "acessórios × estilos" });
                let Some(loaded) = load(&mut client, part, &mut report) else { continue };
                if hair {
                    if loaded.hair_kind >= 1 && loaded.hair_kind != 4 {
                        report.count("cabelos dinâmicos (esqueleto próprio)");
                        if master.match_ref_bone("Bip01_head").is_none() {
                            report.code(format!("{}: mestre sem Bip01_head", part.mesh));
                        }
                    } else if skinning_parity(&loaded, &master, &format!("{} (linha {row})", part.mesh), &mut report) {
                        report.count("cabelos com pesos só em ossos do mestre");
                    }
                    continue;
                }
                let key = AppearanceKey::Player { row, face: 0, hair_style: style, hair_color: 0, equipped, enchant_effect: 0, hero: false, mounted: 0 };
                let Some(mut character) = character(&mut client, &mut shared, key, &appearance) else {
                    report.code(format!("{}: não montou", part.mesh));
                    continue;
                };
                let part_index = character.parts().iter().position(|(_, m)| m.slot == SLOT_HELM).unwrap();
                let Some(PartKind::Attached { bone: Some(head), .. }) = character.kind(part_index) else {
                    report.code(format!("{}: sem osso da cabeça", part.mesh));
                    continue;
                };
                let Some(wait) = wait_sequence(&mut character, &tables, section, 0) else { continue };
                let pose = character.frame_pose(wait, 0.0);
                let Some(world) = attached_world(&character, part_index, &pose, collision) else {
                    report.code(format!("{}: sem paleta", part.mesh));
                    continue;
                };
                let head_at = bone_world(&character, &pose, head);
                let center =
                    loaded.vertices.iter().map(|v| world.transform_point3(Vec3::from(v.position))).sum::<Vec3>()
                        / loaded.vertices.len().max(1) as f32;
                let distance = (center - head_at).length();
                if distance > HEAD_RADIUS {
                    report.data(
                        "acessório longe da cabeça",
                        format!("{} estilo {style} linha {row}: centro a {distance:.1} da cabeça", part.mesh),
                    );
                } else {
                    report.count("acessórios a menos de 30 u da cabeça");
                }
                if center.z < head_at.z - 5.0 {
                    report.data(
                        "acessório abaixo da cabeça",
                        format!("{} estilo {style} linha {row}: {:.1} abaixo", part.mesh, head_at.z - center.z),
                    );
                }
            }
        }
    }
    report.print("cabelos e acessórios");
    assert!(report.code.is_empty(), "{:?}", report.code);
}

#[test]
#[ignore]
fn the_beret_sits_on_the_dwarf_head() {
    let Some(mut client) = client() else { return };
    let tables = client.tables().unwrap();
    let mut shared = Shared::default();
    let equipped = Equipped { hair: 13081, ..Default::default() };
    let appearance = player(&tables, 4, 0, equipped).set_pawn_resource();
    let helm = appearance.part(SLOT_HELM).expect("boina no slot 6");
    assert_eq!(helm.mesh, "L2killer_Helmet_UN.MW_viRUS_Beret_1_m002_a");
    assert_eq!(helm.placement, Placement::Helm { offset: [0.0; 3], rotation: [0; 3] });
    let key = AppearanceKey::Player { row: 4, face: 0, hair_style: 0, hair_color: 0, equipped, enchant_effect: 0, hero: false, mounted: 0 };
    let mut character = character(&mut client, &mut shared, key, &appearance).unwrap();
    let helm_index = character.parts().iter().position(|(_, m)| m.slot == SLOT_HELM).unwrap();
    let Some(PartKind::Attached { bone: Some(head), .. }) = character.kind(helm_index) else { panic!("sem cabeça") };
    let wait = wait_sequence(&mut character, &tables, "mdwarf", 0).unwrap();
    let pose = character.frame_pose(wait, 0.0);
    let collision = Collision { radius: 9.0, height: 18.0 };
    let world = attached_world(&character, helm_index, &pose, collision).unwrap();
    let head_at = bone_world(&character, &pose, head);
    let beret = &character.parts()[helm_index].1;
    let center = beret.vertices.iter().map(|v| world.transform_point3(Vec3::from(v.position))).sum::<Vec3>()
        / beret.vertices.len() as f32;
    let body = character.parts().iter().position(|(_, m)| m.slot == SLOT_BODY).unwrap();
    let body_palette = character.palette(body, &pose, collision).unwrap();
    let body_mesh = character.parts()[body].1.clone();
    let body_top = body_mesh
        .vertices
        .iter()
        .map(|v| {
            let skinned: Vec3 = v
                .bones
                .iter()
                .zip(v.weights)
                .filter(|(_, w)| *w > 0.0)
                .map(|(b, w)| body_palette[usize::from(*b)].transform_point3(Vec3::from(v.position)) * w)
                .sum();
            character.world(body, Vec3::ZERO, 0).transform_point3(skinned).z
        })
        .fold(f32::MIN, f32::max);
    eprintln!("boina: centro {center:?}, cabeça {head_at:?}, topo do corpo {body_top:.1}");
    assert!(center.z > head_at.z, "boina {center:?} abaixo da cabeça {head_at:?}");
    assert!(center.z > body_top - 2.0, "boina {center:?} abaixo do topo do corpo {body_top:.1}");
    assert!((center - head_at).length() < HEAD_RADIUS);
    for part in 0..character.parts().len() {
        if part != helm_index {
            assert_eq!(character.kind(part), Some(PartKind::Skinned), "peça {part}");
        }
    }
}

#[test]
#[ignore]
fn every_npc_mesh_loads_and_matches_its_weapon_bones() {
    let Some(mut client) = client() else { return };
    let tables = client.tables().unwrap();
    let mut report = Report::default();
    let mut npcs: Vec<_> = tables.npcgrp.iter().collect();
    npcs.sort_by_key(|n| n.id);
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for npc in npcs {
        if npc.mesh.is_empty() || !seen.insert(npc.mesh.to_ascii_lowercase()) {
            continue;
        }
        report.count("malhas do npcgrp");
        let part = Part {
            slot: SLOT_FACE,
            mesh: npc.mesh.clone(),
            textures: Part::new(npc.mesh.clone(), npc.textures.clone()).textures,
            placement: Placement::Master,
        };
        let Some(mesh) = load(&mut client, &part, &mut report) else { continue };
        report.count("malhas carregadas");
        let hands = [
            mesh.match_ref_bone("Weapon_R_Bone"),
            mesh.match_ref_bone("Weapon_L_Bone"),
            mesh.match_ref_bone("Shield_R_Bone"),
            mesh.match_ref_bone("Shield_L_Bone"),
        ];
        if hands.iter().all(Option::is_some) {
            report.count("com os quatro ossos de mão/escudo");
        } else if hands.iter().any(Option::is_some) {
            report.count("com alguns ossos de mão/escudo");
        } else {
            report.count("sem ossos de arma (armas não desenhadas, como no engine)");
        }
    }
    report.print("npcgrp");
    assert!(report.code.is_empty(), "{:?}", report.code);
    assert!(report.counts["malhas carregadas"] > 400);
}

#[test]
#[ignore]
fn every_table_texture_resolves_or_says_why() {
    let Some(mut client) = client() else { return };
    let tables = client.tables().unwrap();
    let mut report = Report::default();
    let mut names: BTreeMap<String, &'static str> = BTreeMap::new();
    for npc in tables.npcgrp.iter() {
        for texture in npc.textures.iter().chain(&npc.textures2) {
            names.entry(texture.to_ascii_lowercase()).or_insert("npcgrp");
        }
    }
    for armor in tables.armorgrp.values() {
        for row in &armor.rows {
            for texture in row.textures.iter().chain(&row.ex_textures) {
                names.entry(texture.to_ascii_lowercase()).or_insert("armorgrp");
            }
        }
        for texture in &armor.npc_textures {
            names.entry(texture.to_ascii_lowercase()).or_insert("armorgrp (npc)");
        }
    }
    for weapon in tables.weapongrp.values() {
        for texture in &weapon.textures {
            names.entry(texture.to_ascii_lowercase()).or_insert("weapongrp");
        }
    }
    for (name, table) in &names {
        if name.is_empty() || name == "none" {
            continue;
        }
        report.count(&format!("texturas do {table}"));
        if client.catalog().material(name).is_some() {
            report.count("materiais resolvidos");
            continue;
        }
        match client.catalog().try_texture_raw(name) {
            Ok(_) => report.count("texturas cruas resolvidas"),
            Err(MissingAsset::Encrypted) => report.data("CAFEF00D", format!("{name} ({table})")),
            Err(MissingAsset::Undecodable { version, licensee, error }) => report
                .data(&format!("indecodificável v{version} licensee {licensee}"), format!("{name} ({table}): {error}")),
            Err(why) => report.data(&why.to_string(), format!("{name} ({table})")),
        }
    }
    report.print("texturas das tabelas");
    assert!(report.code.is_empty());
}

#[test]
#[ignore]
fn npc_named_cat_queen_gets_its_textures_or_a_reason() {
    let Some(mut client) = client() else { return };
    let tables = client.tables().unwrap();
    let mut report = Report::default();
    let mut bank = TextureBank::new();
    let cats: Vec<u32> =
        tables.npcname.values().filter(|n| n.name.to_ascii_lowercase().contains("cat queen")).map(|n| n.id).collect();
    for id in &cats {
        let Some(npc) = tables.npcgrp.get(*id) else {
            report.data("sem npcgrp", format!("{id}"));
            continue;
        };
        let part = Part {
            slot: SLOT_FACE,
            mesh: npc.mesh.clone(),
            textures: Part::new(npc.mesh.clone(), npc.textures.clone()).textures,
            placement: Placement::Master,
        };
        let Some(mesh) = load(&mut client, &part, &mut report) else { continue };
        for (index, section) in mesh.sections.iter().enumerate() {
            if section.texture().is_empty() {
                report.data(
                    "seção sem textura",
                    format!("npc {id} {} seção {index}: tabela {:?}", npc.mesh, npc.textures),
                );
            } else if client.catalog().try_texture_raw(section.texture()).is_err() {
                report.data(
                    "textura da seção ausente",
                    format!("npc {id} {} seção {index}: {}", npc.mesh, section.texture()),
                );
            } else {
                let data = bank.texture(client.catalog(), section.texture());
                if data.width == 8 && data.height == 8 {
                    report.data("xadrez", format!("npc {id} {} seção {index}: {}", npc.mesh, section.texture()));
                } else {
                    report.count("seções com textura decodificada");
                }
            }
        }
    }
    eprintln!("Cat Queen: ids {cats:?}");
    report.print("Cat Queen");
    assert!(report.data.is_empty(), "{:?}", report.data);
}
