use glam::{Mat4, Vec3};
use killer_assets::animation::{find_sequence, palette, pose};
use killer_assets::characters::{Part, Placement, WEAPON_R_BONE, bind_pose, load_part};
use killer_assets::client::{Client, DatTables};
use killer_assets::pawn::{Player, SLOT_HAIR_A, SLOT_HAIR_B};
use l2_dat::Equipped;
use std::path::PathBuf;

fn test_client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

fn bone_index(mesh: &killer_assets::characters::CharacterMesh, name: &str) -> usize {
    mesh.bones.iter().position(|b| b.name.eq_ignore_ascii_case(name)).unwrap()
}

#[test]
#[ignore]
fn every_chargrp_base_part_preserves_pose_contract_across_all_lods() {
    let mut client = test_client().expect("KILLER_CLIENT_DIR obrigatório");
    let tables = client.tables().unwrap();
    assert!(tables.chargrp.len() >= 14);
    let mut parts = 0;
    let mut levels = 0;
    let mut changed_sections = 0;
    for row in 0..tables.chargrp.len() {
        for part in player(&tables, row, 0, 0, Equipped::default()).set_pawn_resource().parts {
            let base = load_part(&mut client, &part).expect(&part.mesh);
            let info = base.lod.as_ref().unwrap();
            for index in 0..info.levels.len() {
                let selected = killer_assets::characters::load_part_lod(&mut client, &part, index).expect(&part.mesh);
                let metadata = selected.lod.as_ref().unwrap();
                assert_eq!(metadata.source, part, "{} LOD {index}", part.mesh);
                assert_eq!(metadata.index, index);
                let packed = client.catalog().try_skeletal_mesh_lod(&part.mesh, index).unwrap();
                assert_eq!(metadata.section_material_slots, packed.mesh.section_material_slots);
                assert_eq!(metadata.material_texture_indices, packed.mesh.material_texture_indices);
                assert_eq!(metadata.textures, packed.textures);
                assert_eq!(metadata.material_texture_indices, info.material_texture_indices);
                assert_eq!(metadata.textures, info.textures);
                for (section, slot) in metadata.section_material_slots.iter().enumerate() {
                    let texture = metadata.material_texture_indices.get(usize::from(*slot))
                        .and_then(|&index| usize::try_from(index).ok())
                        .and_then(|index| metadata.textures.get(index)).cloned().flatten();
                    assert_eq!(texture, packed.materials[section]);
                }
                assert_eq!(metadata.section_material_slots.len(), selected.sections.len());
                assert_eq!(metadata.levels, info.levels);
                assert_eq!(metadata.settings, info.settings);
                assert_eq!(selected.bones, base.bones, "{} LOD {index}", part.mesh);
                assert_eq!(selected.bind, base.bind);
                assert_eq!(selected.inverse_bind, base.inverse_bind);
                assert_eq!(selected.local, base.local);
                assert_eq!(selected.origin, base.origin);
                assert_eq!(selected.rotation, base.rotation);
                assert_eq!(selected.scale, base.scale);
                assert_eq!(selected.aliases, base.aliases);
                assert_eq!(selected.animation, base.animation);
                assert_eq!(selected.placement, base.placement);
                assert_eq!(selected.slot, base.slot);
                assert_eq!(selected.hair_kind, base.hair_kind);
                assert_eq!(selected.floats_150, base.floats_150);
                for section in &selected.sections {
                    assert!((section.first_index as usize + section.index_count as usize) <= selected.indices.len());
                }
                assert!(selected.indices.iter().all(|index| (*index as usize) < selected.vertices.len()));
                changed_sections += usize::from(selected.sections.len() != base.sections.len());
                levels += 1;
            }
            parts += 1;
        }
        client.catalog().release_files();
    }
    assert!(parts >= tables.chargrp.len());
    eprintln!("{} linhas chargrp, {parts} partes básicas, {levels} LODs; {changed_sections} com contagem de seções diferente", tables.chargrp.len());
}

#[test]
#[ignore]
fn render_1069a520_selected_lod_keeps_character_placement_and_bind_pose() {
    let mut client = test_client().expect("KILLER_CLIENT_DIR obrigatório");
    let part = Part::new("Fighter.MFighter_m000_u", vec!["MFighter.MFighter_m000_t1000_u".into()]);
    let original = load_part(&mut client, &part).unwrap();
    let original_lod = original.lod.as_ref().unwrap();
    assert_eq!(original_lod.source, part);
    assert_eq!(original_lod.index, 0);
    let (_, info) = client.catalog().try_skeletal_mesh_lod_with_info(&part.mesh, 0).unwrap();
    assert!(!info.is_empty());
    for lod in 0..info.len() {
        let (selected, selected_info) = killer_assets::characters::load_part_lod_with_info(&mut client, &part, lod).unwrap();
        let selected_lod = selected.lod.as_ref().unwrap();
        assert_eq!(selected_lod.source, part);
        assert_eq!(selected_lod.index, lod);
        assert_eq!(selected_lod.settings, original_lod.settings);
        assert_eq!(selected_lod.levels, selected_info);
        assert_eq!(selected_info.len(), info.len());
        for (actual, expected) in selected_info.iter().zip(&info) {
            assert_eq!(actual.distance_factor.to_bits(), expected.distance_factor.to_bits());
            assert_eq!(actual.hysteresis.to_bits(), expected.hysteresis.to_bits());
            assert_eq!(actual.soft_wedges, expected.soft_wedges);
        }
        assert_eq!(selected.slot, original.slot);
        assert_eq!(selected.placement, original.placement);
        assert_eq!(selected.bones, original.bones);
        assert_eq!(selected.bind, original.bind);
        assert_eq!(selected.inverse_bind, original.inverse_bind);
        assert_eq!(selected.local, original.local);
        assert_eq!(selected.aliases, original.aliases);
        assert_eq!(selected.animation, original.animation);
        assert!(selected.sections.iter().all(|section| section.texture() == "MFighter.MFighter_m000_t1000_u"));
        if lod == 0 {
            assert_eq!(selected.vertices, original.vertices);
            assert_eq!(selected.indices, original.indices);
        } else {
            assert!(selected.indices.len() < original.indices.len());
        }
    }
    assert!(killer_assets::characters::load_part_lod(&mut client, &part, usize::MAX).is_none());
}

#[test]
#[ignore]
fn get_render_bounding_box_1066fb50_entire_chargrp() {
    let mut client = test_client().expect("KILLER_CLIENT_DIR obrigatório");
    let tables = client.tables().unwrap();
    assert!(tables.chargrp.len() >= 14);
    let mut checked = 0;
    for row in 0..tables.chargrp.len() {
        for part in player(&tables, row, 0, 0, Equipped::default()).set_pawn_resource().parts {
            let packed = client.catalog().skeletal_mesh(&part.mesh).expect(&part.mesh);
            let mesh = load_part(&mut client, &part).expect(&part.mesh);
            assert_eq!(mesh.bounding_box, packed.mesh.bounding_box);
            assert_eq!(mesh.bounding_sphere, packed.mesh.bounding_sphere);
            let bounds = killer_assets::scene::actor_lighting::Bounds::transformed(mesh.bounding_box, mesh.local);
            assert!(bounds.min.is_finite() && bounds.max.is_finite(), "{}", part.mesh);
            assert!(bounds.extent().cmpge(Vec3::ZERO).all(), "{}", part.mesh);
            for x in 0..2 {
                for y in 0..2 {
                    for z in 0..2 {
                        let point = mesh.local.transform_point3(Vec3::new(mesh.bounding_box[x][0], mesh.bounding_box[y][1], mesh.bounding_box[z][2]));
                        assert!(point.cmpge(bounds.min).all() && point.cmple(bounds.max).all(), "{}", part.mesh);
                    }
                }
            }
            checked += 1;
        }
    }
    assert!(checked >= 14);
    eprintln!("{} registros de chargrp; {checked} partes com FBox e FSphere serializadas preservadas e oito cantos transformados", tables.chargrp.len());
}

#[test]
#[ignore]
fn fighter_upper_body_loads_with_bind_pose_and_textures() {
    let Some(mut client) = test_client() else { return };
    let part = Part::new("Fighter.MFighter_m000_u", vec!["MFighter.MFighter_m000_t1000_u".into()]);
    let mesh = load_part(&mut client, &part).unwrap();
    let packed = client.catalog().skeletal_mesh(&part.mesh).unwrap();
    assert_eq!(mesh.bones.len(), 70);
    assert_eq!(mesh.sections.len(), packed.mesh.sections.len());
    assert_eq!(mesh.bind.len(), 70);
    assert_eq!(mesh.inverse_bind.len(), 70);
    assert_eq!(mesh.vertices.len(), packed.mesh.vertices.len());
    assert_eq!(mesh.indices.len(), packed.mesh.indices.len());
    assert!(mesh.animation.as_deref().is_some_and(|a| a.eq_ignore_ascii_case("Fighter.MFighter_anim")));
    for section in &mesh.sections {
        assert_eq!(section.texture(), "MFighter.MFighter_m000_t1000_u");
        assert!(client.catalog().material(section.texture()).is_some(), "{}", section.texture());
        assert!(section.index_count > 0 && section.index_count % 3 == 0);
        assert!((section.first_index + section.index_count) as usize <= mesh.indices.len());
    }
    assert_eq!(bind_pose(&mesh.bones), mesh.bind);
    let head = bone_index(&mesh, "Bip01_head");
    let pelvis = bone_index(&mesh, "Bip01_Pelvis");
    let z = |i: usize| mesh.bind[i].transform_point3(Vec3::ZERO).z;
    assert!(z(head) > z(pelvis));
}

#[test]
#[ignore]
fn mesh_to_world_of_the_fighter_upper_body_follows_the_client() {
    let Some(mut client) = test_client() else { return };
    let part = Part::new("Fighter.MFighter_m000_u", vec![]);
    let mesh = load_part(&mut client, &part).unwrap();
    let packed = client.catalog().skeletal_mesh(&part.mesh).unwrap();
    assert_eq!(packed.mesh.origin, [0.0, 0.0, 25.2]);
    assert_eq!(packed.mesh.rotation, [0, 49152, 0]);
    assert_eq!(packed.mesh.scale, [1.03; 3]);
    let p = mesh.local.transform_point3(Vec3::new(1.0, 2.0, 3.0));
    assert!((p - Vec3::new(2.06, -1.03, -22.866)).length() < 1e-3, "{p}");
    let front = mesh.local.transform_vector3(Vec3::Y);
    assert!((front - Vec3::new(1.03, 0.0, 0.0)).length() < 1e-4, "{front}");
    let boots = load_part(&mut client, &Part::new("Fighter.MFighter_m000_b", vec![])).unwrap();
    assert_eq!(boots.local, mesh.local);
    let feet = boots.vertices.iter().map(|v| v.position[2]).fold(f32::MAX, f32::min);
    let feet = boots.local.transform_point3(Vec3::new(0.0, 0.0, feet)).z;
    assert!((-27.0..-25.0).contains(&feet), "{feet}");
}

#[test]
#[ignore]
fn npc_part_without_textures_uses_the_mesh_materials() {
    let Some(mut client) = test_client() else { return };
    let part = Part::new("LineageMonsters.gremlin_m00", vec![]);
    let mesh = load_part(&mut client, &part).unwrap();
    assert!(!mesh.sections.is_empty());
    assert!(mesh.sections.iter().all(|s| !s.texture().is_empty()));
    assert!(load_part(&mut client, &Part::new("Nao.Existe", vec![])).is_none());
}

#[test]
#[ignore]
fn wait_pose_keeps_the_fighter_standing() {
    let Some(mut client) = test_client() else { return };
    let part = Part::new("Fighter.MFighter_m000_u", vec!["MFighter.MFighter_m000_t1000_u".into()]);
    let mesh = load_part(&mut client, &part).unwrap();
    let anim = client.catalog().mesh_animation(mesh.animation.as_deref().unwrap()).unwrap();
    let sequence = find_sequence(&anim, "wait_hand_mfighter").unwrap();
    assert_eq!(sequence.name, "Wait_Hand_MFighter");
    let posed = pose(&anim, sequence, &mesh.bones, 0.0);
    assert_eq!(posed.len(), 70);
    let head = bone_index(&mesh, "Bip01_head");
    let pelvis = bone_index(&mesh, "Bip01_Pelvis");
    let z = |i: usize| posed[i].transform_point3(Vec3::ZERO).z;
    assert!(z(head) > z(pelvis));
    let palette = palette(&posed, &mesh.inverse_bind);
    assert_eq!(palette.len(), 70);
    let root = mesh.bind[0].transform_point3(Vec3::ZERO);
    let moved = palette[0].transform_point3(root);
    assert!((moved - posed[0].transform_point3(Vec3::ZERO)).length() < 1e-3);
}

fn player<'a>(tables: &'a DatTables, row: usize, hair_style: i32, hair_color: i32, equipped: Equipped) -> Player<'a> {
    Player {
        template: &tables.chargrp[row],
        row,
        face: 0,
        hair_style,
        hair_color,
        equipped,
        tables: tables.pawn_tables(),
        mounted: None,
        korean: false,
        korean_flag: false,
        clan_crest: None,
    }
}

#[test]
#[ignore]
fn fighter_hair_meshes_and_textures_exist_for_styles_zero_to_four() {
    let Some(mut client) = test_client() else { return };
    let tables = client.tables().unwrap();
    let mut missing = Vec::new();
    let mut checked = 0;
    for style in 0..5 {
        for color in 0..3 {
            let fighter = player(&tables, 0, style, color, Equipped::default());
            let parts: Vec<_> = fighter
                .set_pawn_resource()
                .parts
                .into_iter()
                .filter(|p| p.slot == SLOT_HAIR_A || p.slot == SLOT_HAIR_B)
                .collect();
            assert!(!parts.is_empty(), "style {style}");
            for part in parts {
                checked += 1;
                if client.catalog().skeletal_mesh(&part.mesh).is_none() {
                    missing.push(part.mesh.clone());
                }
                for texture in part.textures.iter().flatten() {
                    if client.catalog().material(texture).is_none() && client.catalog().texture(texture).is_none() {
                        missing.push(texture.clone());
                    }
                }
            }
        }
    }
    assert_eq!(checked, 27);
    assert!(missing.is_empty(), "faltam no catálogo: {missing:?}");
    let a = player(&tables, 0, 1, 1, Equipped::default()).set_pawn_resource();
    assert_eq!(a.parts.len(), 7);
    assert_eq!(a.parts[4].mesh, "Fighter.MFighter_m001_m00_bh");
    assert_eq!(a.parts[5].mesh, "Fighter.MFighter_m001_m00_ah");
    let ah = load_part(&mut client, &a.parts[5]).unwrap();
    assert_eq!(ah.bones.len(), 70);
    assert_eq!(ah.hair_kind, 4);
    assert!(ah.sections.iter().all(|s| !s.texture().is_empty()));
}

#[test]
#[ignore]
fn fighter_in_a_leather_tunic_with_a_sword_and_shield_loads() {
    let Some(mut client) = test_client() else { return };
    let tables = client.tables().unwrap();
    let equipped = Equipped { chest: 21, legs: 28, rhand: 1, lhand: 18, ..Default::default() };
    let a = player(&tables, 0, 1, 1, equipped).set_pawn_resource();
    assert_eq!(a.weapon_type, 1);
    let names: Vec<(u8, &str)> = a.parts.iter().map(|p| (p.slot, p.mesh.as_str())).collect();
    assert_eq!(names[2], (2, "Fighter.MFighter_m001_u"));
    assert_eq!(a.parts[2].textures, [Some("MFighter.MFighter_m001_t02_u".into())]);
    assert_eq!(names[1], (1, "Fighter.MFighter_m001_l"));
    assert_eq!(names[7], (8, "LineageWeapons.small_sword_m00_wp"));
    assert_eq!(names[8], (11, "LineageWeapons.leather_shield_m00_sh"));
    let body = load_part(&mut client, &a.parts[2]).unwrap();
    assert_eq!(body.bones.len(), 70);
    assert!(body.sections.iter().all(|s| s.texture() == "MFighter.MFighter_m001_t02_u"));
    assert!(body.bones.iter().any(|b| b.name == WEAPON_R_BONE));
    let sword = load_part(&mut client, &a.parts[7]).unwrap();
    assert_eq!(sword.placement, Placement::Attached { bone: WEAPON_R_BONE });
    assert_eq!(sword.bones.len(), 1);
    assert_eq!(sword.sections.len(), 1);
    assert_eq!(sword.sections[0].texture(), "LineageWeaponsTex.small_sword_t00_wp");
    assert!(client.catalog().material(sword.sections[0].texture()).is_some());
    assert!(sword.vertices.iter().all(|v| v.bones[0] == 0 && v.weights[0] == 1.0));
    assert_eq!(sword.local, Mat4::IDENTITY);
    assert_eq!((sword.origin, sword.scale), (Vec3::ZERO, Vec3::ONE));
    let shield = load_part(&mut client, &a.parts[8]).unwrap();
    assert_eq!(shield.bones.len(), 1);
    assert_eq!(shield.sections.len(), 2);
    assert!(shield.sections.iter().all(|s| !s.texture().is_empty()));
}

#[test]
#[ignore]
fn c5_dwarf_set_and_shining_bow_keep_z_alpha_test_and_specular() {
    use l2_catalog::{BlendFactor, StageSource};
    let Some(mut client) = test_client() else { return };
    let tables = client.tables().unwrap();
    let mut parts = Vec::new();
    for armor in tables.armorgrp.values() {
        for row in &armor.rows {
            for (index, texture) in row.textures.iter().enumerate() {
                if texture.to_lowercase().starts_with("c5mdwarf.mdwarf_m004_t1005_")
                    && let Some(mesh) = row.meshes.get(index).filter(|mesh| !mesh.is_empty())
                {
                    parts.push(Part::new(mesh.clone(), vec![texture.clone()]));
                }
            }
        }
    }
    let bow = tables.weapongrp.values().find(|weapon| weapon.meshes.iter().any(|mesh| mesh.eq_ignore_ascii_case("LineageWeapons.shining_bow_m00_wp"))).unwrap();
    parts.push(Part::new(bow.meshes[0].clone(), bow.textures.clone()));
    parts.sort_by(|a, b| a.mesh.cmp(&b.mesh));
    parts.dedup_by(|a, b| a.mesh == b.mesh && a.textures == b.textures);
    assert!(parts.len() >= 5, "{parts:?}");
    let default_path = client.catalog().default_material_path().expect("Engine.Material.DefaultMaterial");
    assert!(default_path.eq_ignore_ascii_case("Engine.DefaultTexture"));
    let default_material = client.catalog().material(&default_path).expect(&default_path);
    let mut default_sections = 0;
    let mut clothing_sections = 0;
    for part in &parts {
        let mesh = load_part(&mut client, part).unwrap_or_else(|| panic!("{}", part.mesh));
        assert!(!mesh.sections.is_empty(), "{}", part.mesh);
        let lod = mesh.lod.as_ref().unwrap();
        let mut bindings = killer_assets::part_materials::PartMaterials::load(part, &lod.textures, |name| {
            client.catalog().object_class(name).is_some()
        }).unwrap();
        for (index, section) in mesh.sections.iter().enumerate() {
            let material = &section.material;
            if bindings.section(index, &lod.section_material_slots, &lod.material_texture_indices).unwrap().is_none() {
                assert_eq!(material, &default_material, "{} seção {index}", part.mesh);
                default_sections += 1;
                continue;
            }
            clothing_sections += 1;
            let pass = &material.passes[0];
            assert!(pass.z_write && pass.z_test && pass.two_sided, "{part:?} {}: {pass:?}, LOD {:?}", section.texture(), mesh.lod);
            let final_blend = pass.alpha_test == Some(160);
            if final_blend {
                assert_eq!((pass.src, pass.dst), (BlendFactor::SrcAlpha, BlendFactor::InvSrcAlpha), "{}", part.mesh);
                assert_eq!(section.pass, killer_assets::scene::Pass::Translucent, "{}", part.mesh);
                assert_eq!(material.sorting_level, 1, "{}", part.mesh);
            } else {
                assert_eq!((pass.src, pass.dst), (BlendFactor::One, BlendFactor::Zero), "{}", part.mesh);
                assert_eq!(section.pass, killer_assets::scene::Pass::Opaque, "{}", part.mesh);
            }
            let env_map_stage = material.pass_stages(pass).iter().any(|stage| stage.coord_source == l2_catalog::CoordSource::CameraEnvMap);
            let shine_pass = material.passes.get(1).is_some_and(|shine| (shine.src, shine.dst) == (BlendFactor::SrcAlpha, BlendFactor::One) && !shine.z_write);
            assert!(env_map_stage || shine_pass, "{} {}: {material:?}", part.mesh, section.texture());
            assert!(material.stages.iter().any(|stage| matches!(&stage.source, StageSource::Texture(texture) if texture.path.to_lowercase().ends_with("_sp"))), "{}", part.mesh);
        }
    }
    assert!(default_sections > 0);
    assert!(clothing_sections >= parts.len());
}

#[test]
#[ignore]
fn every_item_material_keeps_its_opacity_alpha_in_the_first_pass() {
    use l2_catalog::AlphaOp;
    use std::collections::BTreeMap;
    let Some(mut client) = test_client() else { return };
    let tables = client.tables().unwrap();
    let mut paths: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for armor in tables.armorgrp.values() {
        for row in &armor.rows {
            paths.extend(row.textures.iter().filter(|t| !t.is_empty()).cloned());
        }
    }
    for weapon in tables.weapongrp.values() {
        paths.extend(weapon.textures.iter().filter(|t| !t.is_empty()).cloned());
    }
    let mut shapes: BTreeMap<String, usize> = BTreeMap::new();
    let mut with_opacity_and_mask = 0;
    let mut unresolved = Vec::new();
    let mut wrong_alpha = Vec::new();
    for path in &paths {
        let Some(material) = client.catalog().material(path) else {
            unresolved.push(path.clone());
            continue;
        };
        *shapes.entry(material.summary()).or_default() += 1;
        let Some(pass) = material.passes.first() else { continue };
        let stages = material.pass_stages(pass);
        let opacity_blend = pass.src == l2_catalog::BlendFactor::SrcAlpha && pass.dst == l2_catalog::BlendFactor::InvSrcAlpha;
        let mask_pass = material.passes.get(1).is_some_and(|shine| shine.src == l2_catalog::BlendFactor::SrcAlpha && shine.dst == l2_catalog::BlendFactor::One);
        if opacity_blend && mask_pass {
            with_opacity_and_mask += 1;
        }
        let alpha_from = stages.iter().rev().find(|stage| stage.alpha_op != AlphaOp::Current);
        if (pass.alpha_blend || pass.alpha_test.is_some())
            && let Some(stage) = alpha_from
            && stage.texture().is_some_and(|texture| texture.path.to_lowercase().ends_with("_sp"))
            && stages.iter().any(|other| other.texture().is_some_and(|texture| texture.path.to_lowercase().ends_with("_ori")))
        {
            wrong_alpha.push(path.clone());
        }
    }
    eprintln!("{} materiais de itens: {} não resolvem, {} com Opacity + SpecularityMask em passe próprio", paths.len(), unresolved.len(), with_opacity_and_mask);
    for (shape, count) in &shapes {
        eprintln!("{count:6}  {shape}");
    }
    eprintln!("não resolvidos: {unresolved:?}");
    assert!(wrong_alpha.is_empty(), "alfa do passe 0 vindo do _sp: {wrong_alpha:?}");
    assert!(unresolved.len() < paths.len() / 100, "{unresolved:?}");
}

#[test]
#[ignore]
fn skeletal_mesh_triangles_wind_against_their_stored_normals() {
    let Some(mut client) = test_client() else { return };
    let (mut along, mut against) = (0usize, 0usize);
    for mesh in ["Fighter.MFighter_m000_u", "Fighter.MFighter_m000_b", "LineageMonsters.gremlin_m00", "Dwarf.MDwarf_m004_u", "LineageWeapons.shining_bow_m00_wp"] {
        let Some(packed) = client.catalog().skeletal_mesh(mesh) else { continue };
        let mesh = packed.mesh;
        for tri in mesh.indices.as_chunks::<3>().0 {
            let v = |i: u16| &mesh.vertices[usize::from(i)];
            let (a, b, c) = (v(tri[0]), v(tri[1]), v(tri[2]));
            let winding = (Vec3::from(b.position) - Vec3::from(a.position)).cross(Vec3::from(c.position) - Vec3::from(a.position));
            let stored = Vec3::from(a.normal) + Vec3::from(b.normal) + Vec3::from(c.normal);
            let dot = winding.dot(stored);
            if dot > 1e-6 {
                along += 1;
            } else if dot < -1e-6 {
                against += 1;
            }
        }
    }
    eprintln!("skeletal: {along} along, {against} against");
    assert!(against > 20 * along, "{along} along vs {against} against");
}
