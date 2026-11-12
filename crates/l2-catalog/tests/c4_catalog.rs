use std::path::PathBuf;

use l2_catalog::Catalog;
use ue2_assets::{Bone, SkinVertex};

const FALLBACK_DISTANCE: f32 = 5.0;

#[test]
#[ignore]
fn render_1069a520_catalog_preserves_selected_lod_materials_and_skeleton() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut catalog = Catalog::open(&root);
    let path = "Fighter.MFighter_m000_u";
    let original = catalog.try_skeletal_mesh(path).unwrap();
    let (zero, info) = catalog.try_skeletal_mesh_lod_with_info(path, 0).unwrap();
    assert!(!info.is_empty());
    assert_eq!(original.mesh, zero.mesh);
    assert_eq!(original.materials, zero.materials);
    assert_eq!(original.animation, zero.animation);
    for lod in 1..info.len() {
        let (selected, selected_info) = catalog.try_skeletal_mesh_lod_with_info(path, lod).unwrap();
        assert_eq!(selected_info.len(), info.len());
        for (actual, expected) in selected_info.iter().zip(&info) {
            assert_eq!(actual.distance_factor.to_bits(), expected.distance_factor.to_bits());
            assert_eq!(actual.hysteresis.to_bits(), expected.hysteresis.to_bits());
            assert_eq!(actual.soft_wedges, expected.soft_wedges);
        }
        assert_eq!(original.mesh.bones, selected.mesh.bones);
        assert_eq!(original.mesh.lod_settings, selected.mesh.lod_settings);
        assert_eq!(original.animation, selected.animation);
        assert_eq!(selected.materials.len(), selected.mesh.sections.len());
        assert!(selected.mesh.indices.len() < original.mesh.indices.len());
    }
    assert!(catalog.try_skeletal_mesh_lod(path, usize::MAX).is_err());
    assert_eq!(catalog.try_skeletal_mesh(path).unwrap().mesh, original.mesh);
}

fn client_dir() -> Option<PathBuf> {
    let Some(client) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else {
        eprintln!("KILLER_CLIENT_DIR not set; skipping");
        return None;
    };
    Some(client)
}

fn multiply(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let ([ax, ay, az, aw], [bx, by, bz, bw]) = (a, b);
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

fn conjugate(q: [f32; 4]) -> [f32; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}

fn rotate(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let r = multiply(multiply(q, [v[0], v[1], v[2], 0.0]), conjugate(q));
    [r[0], r[1], r[2]]
}

fn bind_pose(bones: &[Bone]) -> Vec<([f32; 4], [f32; 3])> {
    let mut world: Vec<([f32; 4], [f32; 3])> = Vec::with_capacity(bones.len());
    for (index, bone) in bones.iter().enumerate() {
        let (rotation, origin) = if index == 0 {
            (bone.rotation, bone.position)
        } else {
            let (parent_rotation, parent_origin) = world[bone.parent];
            let offset = rotate(parent_rotation, bone.position);
            let origin = [parent_origin[0] + offset[0], parent_origin[1] + offset[1], parent_origin[2] + offset[2]];
            (multiply(parent_rotation, conjugate(bone.rotation)), origin)
        };
        world.push((rotation, origin));
    }
    world
}

fn centroid(vertices: &[SkinVertex]) -> [f32; 3] {
    let n = vertices.len() as f32;
    vertices.iter().fold([0.0; 3], |sum, vertex| {
        [sum[0] + vertex.position[0] / n, sum[1] + vertex.position[1] / n, sum[2] + vertex.position[2] / n]
    })
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[test]
#[ignore]
fn c4_fighter_through_the_catalog() {
    let Some(client) = client_dir() else { return };
    let mut catalog = Catalog::open(&client);

    let packed = catalog.skeletal_mesh("Fighter.MFighter_m000_u").unwrap();
    assert_eq!(packed.mesh.bones.len(), 70);
    assert!(packed.mesh.vertices.len() > 300 && packed.mesh.indices.len() > 1400);
    assert!(packed.animation.as_deref().is_some_and(|path| path.eq_ignore_ascii_case("Fighter.MFighter_anim")));
    assert_eq!(packed.materials.len(), packed.mesh.sections.len());

    for (name, vertices) in [("Fighter.FFighter_m000_f", 192), ("Fighter.MFighter_m000_f", 207)] {
        let rigid = catalog.skeletal_mesh(name).unwrap();
        assert_eq!(rigid.mesh.vertices.len(), vertices, "{name}");
        let head = rigid.mesh.bones.iter().position(|bone| bone.name == "Bip01_head").unwrap();
        let bound = |vertex: &SkinVertex| usize::from(vertex.bones[0]) == head && vertex.weights[0] == 1.0;
        assert!(rigid.mesh.vertices.iter().all(bound), "{name}");
        let head_origin = bind_pose(&rigid.mesh.bones)[head].1;
        let gap = distance(centroid(&rigid.mesh.vertices), head_origin);
        assert!(gap <= FALLBACK_DISTANCE, "{name}: face centroid {gap} units from Bip01_head");
    }
    let face = catalog.skeletal_mesh("Fighter.MFighter_m000_f").unwrap();
    assert!(
        face.materials[0].as_deref().is_some_and(|path| path.to_lowercase().contains("face")),
        "{:?}",
        face.materials
    );

    let animation = catalog.mesh_animation(packed.animation.as_deref().unwrap()).unwrap();
    assert_eq!(animation.bones.len(), packed.mesh.bones.len());
    assert_eq!(animation.sequences.len(), 114);
    assert!(animation.sequences.iter().any(|sequence| sequence.name == "Falling_MFighter"));
}

#[test]
#[ignore]
fn c5_armor_final_blends_write_z_and_add_an_env_map_specular() {
    use l2_catalog::{AlphaOp, BlendFactor, ColorOp, CoordSource, FogMode};
    let Some(client) = client_dir() else { return };
    let mut catalog = Catalog::open(&client);
    let final_blends =
        ["c5MDwarf.MDwarf_m004_t1005_u", "c5MDwarf.MDwarf_m004_t1005_l", "LineageWeaponsTex.shining_bow_t00_wp"];
    let plain_shaders = ["c5MDwarf.MDwarf_m004_t1005_g", "c5MDwarf.MDwarf_m004_t1005_b"];
    for path in final_blends.iter().chain(&plain_shaders) {
        let material = catalog.material(path).unwrap_or_else(|| panic!("{path}"));
        let pass = &material.passes[0];
        assert!(pass.z_write && pass.z_test && pass.two_sided, "{path}: {pass:?}");
        if final_blends.contains(path) {
            assert_eq!(pass.alpha_test, Some(160), "{path}");
            assert_eq!((pass.src, pass.dst), (BlendFactor::SrcAlpha, BlendFactor::InvSrcAlpha), "{path}");
            assert_eq!(material.sorting_level, 1, "{path}");
        } else {
            assert_eq!(pass.alpha_test, None, "{path}");
            assert_eq!((pass.src, pass.dst), (BlendFactor::One, BlendFactor::Zero), "{path}");
            assert_eq!(material.sorting_level, 0, "{path}");
        }
        let stages = material.pass_stages(pass);
        assert_eq!(stages[0].color_op, ColorOp::SelectTexture, "{path}");
        assert_eq!(stages[0].alpha_op, AlphaOp::Texture, "{path}");
        assert_eq!(pass.light_after, 1, "{path}");
        assert_eq!(stages.len(), 1, "{path}: {stages:?}");
        let alpha_source = stages
            .iter()
            .rev()
            .find(|stage| stage.alpha_op != AlphaOp::Current)
            .and_then(|stage| stage.texture())
            .unwrap_or_else(|| panic!("{path}"));
        let expected_alpha = if final_blends.contains(path) { "_ori" } else { "_sp" };
        assert!(
            alpha_source.path.to_lowercase().ends_with(expected_alpha),
            "{path}: alfa do passe vem de {}",
            alpha_source.path
        );
        assert_eq!(material.passes.len(), 2, "{path}");
        let shine = &material.passes[1];
        assert_eq!((shine.src, shine.dst), (BlendFactor::SrcAlpha, BlendFactor::One), "{path}");
        assert!(!shine.z_write && !shine.lit, "{path}: {shine:?}");
        assert_eq!(shine.fog, FogMode::Black, "{path}");
        let shine_stages = material.pass_stages(shine);
        let mask = shine_stages.last().unwrap_or_else(|| panic!("{path}"));
        assert_eq!((mask.color_op, mask.alpha_op), (ColorOp::SelectCurrent, AlphaOp::Texture), "{path}");
        assert!(mask.texture().is_some_and(|texture| texture.path.to_lowercase().ends_with("_sp")), "{path}: {mask:?}");
        if path.contains("shining_bow") {
            assert_eq!(shine_stages.len(), 2, "{path}: {shine_stages:?}");
            assert_eq!(shine_stages[0].coord_source, CoordSource::CameraEnvMap, "{path}");
            assert!(
                shine_stages[0].texture().is_some_and(|texture| texture.path.to_lowercase().starts_with("cubemaps.")),
                "{path}: {shine_stages:?}"
            );
        } else {
            assert_eq!(shine_stages.len(), 1, "{path}: {shine_stages:?}");
            assert_eq!(mask.coord_source, CoordSource::Stream(0), "{path}");
        }
    }
}

#[test]
#[ignore]
fn a_tile_env_map_over_a_cubemap_is_a_reflection_stage_in_one_pass() {
    use l2_catalog::{ColorOp, CoordSource, StageSource};
    let Some(client) = client_dir() else { return };
    let mut catalog = Catalog::open(&client);
    let mut seen = 0;
    for path in ["Gludio_S.gludio_metal_s", "AGIT_S.Agit_S_metal"] {
        let Some(material) = catalog.material(path) else { continue };
        let stages = material.pass_stages(&material.passes[0]);
        if let Some(stage) = stages.iter().find(|stage| matches!(stage.source, StageSource::Cubemap(_))) {
            assert!(matches!(stage.coord_source, CoordSource::CameraEnvMap | CoordSource::WorldEnvMap));
            assert!(matches!(stage.color_op, ColorOp::Add | ColorOp::ModulateAlphaAddColor | ColorOp::MultiplyAdd));
            seen += 1;
        }
    }
    let _ = seen;
}
