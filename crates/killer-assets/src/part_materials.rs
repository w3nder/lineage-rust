use crate::pawn::{Part, Placement};
use crate::pawn_textures::{InvalidSkinIndex, PawnTextures, TextureResources};
use crate::skeletal_material::{
    AttachedSelectionContext, InvalidTextureIndex, MaterialSelectionContext, select_attached_weapon_texture,
    select_extra_sub_material, select_material, select_sub_material,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialBindingError {
    Skin(InvalidSkinIndex),
    Texture(InvalidTextureIndex),
    MissingSection(usize),
    MissingMaterial(u16),
    TooManyTextures,
}

struct Resource {
    material: Option<usize>,
}

impl TextureResources for Resource {
    fn load(&mut self, _: i32, target: &mut Option<usize>) -> bool {
        if let Some(material) = self.material {
            *target = Some(material);
        }
        true
    }

    fn enqueue(&mut self, _: i32, _: i32) {}
}

pub struct PartMaterials {
    pawn: PawnTextures,
    mesh: Vec<Option<usize>>,
    names: Vec<String>,
    extra: Option<usize>,
    placement: Placement,
    slot: u8,
    pub missing: Vec<(usize, String)>,
}

impl PartMaterials {
    pub fn load(
        part: &Part,
        mesh_textures: &[Option<String>],
        mut available: impl FnMut(&str) -> bool,
    ) -> Result<Self, MaterialBindingError> {
        let mut bindings = Self {
            pawn: PawnTextures::default(),
            mesh: Vec::with_capacity(mesh_textures.len()),
            names: Vec::new(),
            extra: None,
            placement: part.placement,
            slot: part.slot,
            missing: Vec::new(),
        };
        for name in mesh_textures {
            let material = name.as_deref().and_then(|name| bindings.register(name, &mut available));
            bindings.mesh.push(material);
        }
        for (index, name) in part.textures.iter().enumerate() {
            let material = name.as_deref().and_then(|name| bindings.register(name, &mut available));
            let mut resource = Resource { material };
            if matches!(part.placement, Placement::Extra { .. }) {
                if index == 0 {
                    bindings.extra = material;
                }
            } else {
                bindings
                    .pawn
                    .set_texes(
                        i32::from(part.slot),
                        name.is_some(),
                        i32::try_from(index).map_err(|_| MaterialBindingError::TooManyTextures)?,
                        &mut resource,
                    )
                    .map_err(MaterialBindingError::Skin)?;
            }
            if material.is_none()
                && let Some(name) = name
            {
                bindings.missing.push((index, name.clone()));
            }
        }
        Ok(bindings)
    }

    fn register(&mut self, name: &str, available: &mut impl FnMut(&str) -> bool) -> Option<usize> {
        if let Some(index) = self.names.iter().position(|existing| existing.eq_ignore_ascii_case(name)) {
            return Some(index);
        }
        if !available(name) {
            return None;
        }
        let index = self.names.len();
        self.names.push(name.to_owned());
        Some(index)
    }

    pub fn section(
        &mut self,
        section: usize,
        material_slots: &[u16],
        texture_indices: &[i32],
    ) -> Result<Option<&str>, MaterialBindingError> {
        let slot = *material_slots.get(section).ok_or(MaterialBindingError::MissingSection(section))?;
        let index = *texture_indices.get(usize::from(slot)).ok_or(MaterialBindingError::MissingMaterial(slot))?;
        let pawn_slot = i32::from(self.slot);
        let extra = self.extra;
        let selected = match self.placement {
            Placement::Master => select_material(self, index),
            Placement::Skinned | Placement::Helm { .. } => select_sub_material(self, index, pawn_slot),
            Placement::Attached { .. } | Placement::Enchanted { .. } => {
                select_attached_weapon_texture(self, index, pawn_slot)
            }
            Placement::Extra { .. } => select_extra_sub_material(self, index, extra),
        }
        .map_err(MaterialBindingError::Texture)?;
        Ok(selected.and_then(|id| self.names.get(id)).map(String::as_str))
    }
}

impl MaterialSelectionContext for PartMaterials {
    fn has_actor(&self) -> bool {
        true
    }
    fn skin(&mut self, index: usize) -> Option<usize> {
        self.pawn.skins.get(index).flatten()
    }
    fn first_skin(&self) -> Option<Option<usize>> {
        self.pawn.skins.get(0)
    }
    fn sub_skin(&mut self, index: i32) -> Option<usize> {
        crate::skeletal_material::sub_skin(Some(&self.pawn.sub_skins), index)
    }
    fn mesh_texture(&self, index: usize) -> Option<Option<usize>> {
        self.mesh.get(index).copied()
    }
}

impl AttachedSelectionContext for PartMaterials {
    fn is_pawn(&self) -> bool {
        true
    }
    fn attached_skin(&mut self, slot: i32, index: usize) -> Option<usize> {
        self.pawn.attached_skin(slot, index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Write;
    use std::process::{Command, Stdio};

    fn make_part(slot: u8, placement: Placement, names: &[&str]) -> Part {
        Part { slot, placement, ..Part::new("Mesh", names.iter().map(|name| (*name).into()).collect()) }
    }

    fn resolve(bindings: &mut PartMaterials, index: i32) -> Option<String> {
        bindings.section(0, &[0], &[index]).unwrap().map(str::to_owned)
    }

    #[test]
    fn section_material_slot_and_texture_index_are_both_indirections() {
        let part = make_part(7, Placement::Master, &["Actor0", "Actor1", "Actor2"]);
        let mut bindings = PartMaterials::load(&part, &[], |_| true).unwrap();
        let slots = [2, 0, 2, 1];
        let indices = [1, 2, 0];
        let paths: Vec<_> = (0..4).map(|i| bindings.section(i, &slots, &indices).unwrap().map(str::to_owned)).collect();
        assert_eq!(paths, [Some("Actor0".into()), Some("Actor1".into()), Some("Actor0".into()), Some("Actor2".into())]);
    }

    #[test]
    fn master_keeps_null_gaps_and_prefers_mesh_entry_before_first_skin() {
        let part = make_part(7, Placement::Master, &["None", "Actor1", "", "Actor3"]);
        let mut bindings =
            PartMaterials::load(&part, &[Some("Mesh0".into()), None, Some("Mesh2".into())], |_| true).unwrap();
        assert_eq!(resolve(&mut bindings, 0), None);
        assert_eq!(resolve(&mut bindings, 1).as_deref(), Some("Actor1"));
        assert_eq!(resolve(&mut bindings, 2).as_deref(), Some("Mesh2"));
        assert_eq!(resolve(&mut bindings, 3).as_deref(), Some("Actor3"));
        assert_eq!(resolve(&mut bindings, 4), None);
    }

    #[test]
    fn submesh_and_helm_use_last_set_texes_write_only_at_texture_zero() {
        for placement in [Placement::Skinned, Placement::Helm { offset: [0.0; 3], rotation: [0; 3] }] {
            let part = make_part(3, placement, &["First", "Last"]);
            let mut bindings =
                PartMaterials::load(&part, &[Some("Mesh0".into()), Some("Mesh1".into())], |_| true).unwrap();
            assert_eq!(resolve(&mut bindings, 0).as_deref(), Some("Last"));
            assert_eq!(resolve(&mut bindings, 1).as_deref(), Some("Mesh1"));
            assert_eq!(resolve(&mut bindings, 2), None);
            let part = make_part(3, placement, &["First", "None"]);
            let mut bindings = PartMaterials::load(&part, &[Some("Mesh0".into())], |_| true).unwrap();
            assert_eq!(resolve(&mut bindings, 0).as_deref(), Some("Mesh0"));
        }
    }

    #[test]
    fn attached_singletons_and_left_arm_array_do_not_repeat_last_name() {
        for slot in 8..=12 {
            let part = make_part(slot, Placement::Attached { bone: "Bone" }, &["First", "Last"]);
            let mut bindings = PartMaterials::load(&part, &[Some("Mesh0".into()), None], |_| true).unwrap();
            assert_eq!(resolve(&mut bindings, 0).as_deref(), Some(if slot == 11 { "First" } else { "Last" }));
            assert_eq!(resolve(&mut bindings, 1).as_deref(), if slot == 11 { Some("Last") } else { None });
            assert_eq!(resolve(&mut bindings, 2).as_deref(), Some(if slot == 11 { "Mesh0" } else { "Last" }));
        }
    }

    #[test]
    fn extra_material_is_not_written_into_pawn_subskins_or_repeated() {
        let part = make_part(3, Placement::Extra { bone: None }, &["Extra"]);
        let mut bindings = PartMaterials::load(&part, &[Some("Mesh0".into()), Some("Mesh1".into())], |_| true).unwrap();
        assert_eq!(bindings.pawn, PawnTextures::default());
        assert_eq!(resolve(&mut bindings, 0).as_deref(), Some("Extra"));
        assert_eq!(resolve(&mut bindings, 1).as_deref(), Some("Mesh1"));
        assert_eq!(resolve(&mut bindings, 2), None);
    }

    #[test]
    fn failed_synchronous_resource_preserves_previous_singleton_and_records_missing_name() {
        let part = make_part(2, Placement::Skinned, &["Ready", "Missing"]);
        let mut bindings = PartMaterials::load(&part, &[], |name| name != "Missing").unwrap();
        assert_eq!(bindings.missing, [(1, "Missing".into())]);
        assert_eq!(resolve(&mut bindings, 0).as_deref(), Some("Ready"));
    }

    #[test]
    fn invalid_section_mapping_does_not_invent_a_material() {
        let part = make_part(7, Placement::Master, &["Actor"]);
        let mut bindings = PartMaterials::load(&part, &[], |_| true).unwrap();
        assert_eq!(bindings.section(0, &[], &[]), Err(MaterialBindingError::MissingSection(0)));
        assert_eq!(bindings.section(0, &[3], &[0]), Err(MaterialBindingError::MissingMaterial(3)));
        assert_eq!(bindings.section(0, &[0], &[-1]), Err(MaterialBindingError::Texture(InvalidTextureIndex(-1))));
    }

    #[test]
    #[ignore]
    fn all_chargrp_lod_sections_match_native_material_selection() {
        let root = std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = crate::client::Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        assert!(tables.chargrp.len() >= 14);
        let mut cases = Vec::new();
        let mut expected = Vec::new();
        let mut labels = Vec::new();
        let mut pieces = 0;
        let mut lods = 0;
        for (row, template) in tables.chargrp.iter().enumerate() {
            let player = crate::pawn::Player {
                template,
                row,
                face: 0,
                hair_style: 0,
                hair_color: 0,
                equipped: l2_dat::Equipped::default(),
                tables: tables.pawn_tables(),
                mounted: None,
                korean: false,
                korean_flag: false,
                clan_crest: None,
            };
            for part in player.set_pawn_resource().parts {
                pieces += 1;
                let (_, levels) = client.catalog().try_skeletal_mesh_lod_with_info(&part.mesh, 0).expect(&part.mesh);
                for level in 0..levels.len() {
                    lods += 1;
                    let packed = client.catalog().try_skeletal_mesh_lod(&part.mesh, level).unwrap();
                    let mut bindings = PartMaterials::load(&part, &packed.textures, |name| {
                        client.catalog().object_class(name).is_some()
                    })
                    .unwrap();
                    assert!(bindings.names.len() < 128);
                    let skins: Vec<_> =
                        (0..bindings.pawn.skins.len()).map(|i| bindings.pawn.skins.get(i).unwrap()).collect();
                    for (section, slot) in packed.mesh.section_material_slots.iter().enumerate() {
                        let index = packed.mesh.material_texture_indices[usize::from(*slot)];
                        let selected = bindings
                            .section(
                                section,
                                &packed.mesh.section_material_slots,
                                &packed.mesh.material_texture_indices,
                            )
                            .unwrap()
                            .map(str::to_owned);
                        expected.push(
                            selected
                                .as_ref()
                                .map(|name| bindings.names.iter().position(|candidate| candidate == name).unwrap()),
                        );
                        cases.push(json!({
                            "resolver": if part.placement == Placement::Master { "normal" } else { "sub" },
                            "mesh": bindings.mesh, "skins": skins, "sub_skins": bindings.pawn.sub_skins,
                            "pawn": true, "index": index, "sub_index": part.slot,
                            "modifier": 0, "modified": false, "shader_flags": 0, "color_flags": 0,
                        }));
                        labels.push(format!("row {row} {} LOD {level} seção {section}", part.mesh));
                    }
                }
                client.catalog().release_files();
            }
        }
        assert!(pieces >= 90 && lods >= pieces && cases.len() >= lods);
        let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
            .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/skeletal_material_reference.py"))
            .arg(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&serde_json::to_vec(&cases).unwrap()).unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let actual: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(actual.len(), expected.len());
        for ((result, expected), label) in actual.iter().zip(expected).zip(labels) {
            assert_eq!(result[0], json!(expected), "{label}");
        }
        eprintln!(
            "{} linhas chargrp, {pieces} peças, {lods} LODs, {} seções comparadas com engine.dll",
            tables.chargrp.len(),
            cases.len()
        );
    }
}
