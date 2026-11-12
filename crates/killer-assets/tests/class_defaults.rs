use killer_assets::client::{ClassRenderDefaults, Client};
use std::path::PathBuf;

#[test]
fn get_class_name_10313bf6_uses_all_player_rows_and_npc_record() {
    use killer_assets::pawn::ClassSource;
    for (row, expected) in [
        "MFighter", "FFighter", "MDarkElf", "FDarkElf", "MDwarf", "FDwarf", "MElf", "FElf",
        "MMagic", "FMagic", "MOrc", "FOrc", "MShaman", "FShaman",
    ].into_iter().enumerate() {
        assert_eq!(ClassSource::Player { row }.name(), Some(format!("LineageWarrior.{expected}")));
    }
    for row in [14, 15, usize::MAX] {
        assert_eq!(ClassSource::Player { row }.name(), None);
    }
    assert_eq!(ClassSource::Npc { record: None }.name(), None);
    let mut npc = l2_dat::npcgrp::read(include_bytes!("../../l2-dat/tests/fixtures/npcgrp-head.bin")).unwrap().remove(0);
    assert_eq!(ClassSource::Npc { record: Some(&npc) }.name(), Some(npc.class.clone()));
    npc.class = "nOnE".into();
    assert_eq!(ClassSource::Npc { record: Some(&npc) }.name(), None);
}

#[test]
#[ignore]
fn init_properties_10161ac0_preserves_overrides_in_all_c4_pawn_packages() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    for name in ["Engine", "LineageWarrior", "LineageNpc", "LineageMonster", "LineageMonster2"] {
        let file = client.read_system_file(&format!("{name}.u")).unwrap();
        let package = ue2_package::Package::parse(&file).unwrap();
        let mut count = 0;
        for export in package.exports().iter().filter(|export| package.class_name(export) == "Class") {
            let path = format!("{name}.{}", package.name(export.name));
            let resolved = client.class_render_defaults(&path).unwrap_or_else(|error| panic!("{path}: {error}"));
            let properties = ue2_assets::class_default_properties(&package, &file, export).unwrap();
            for (property, actual) in [("LODBias", resolved.lod_bias), ("DrawScale", resolved.draw_scale)] {
                if let Some(value) = ue2_assets::find(&properties, property) {
                    assert_eq!(actual.map(f32::to_bits), value.float().map(f32::to_bits), "{path}.{property}");
                }
            }
            if let Some(value) = ue2_assets::find(&properties, "DrawScale3D") {
                assert_eq!(resolved.draw_scale_3d.map(|v| v.map(f32::to_bits)), value.vector().map(|v| v.map(f32::to_bits)), "{path}.DrawScale3D");
            }
            count += 1;
        }
        assert!(count > 0);
        eprintln!("{name}: herança resolvida para {count} classes");
    }
}

#[test]
#[ignore]
fn init_properties_10161ac0_inherits_render_fields_for_every_player_class() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let actor = client.class_render_defaults("Engine.Actor").unwrap();
    assert_eq!(actor, ClassRenderDefaults {
        lod_bias: Some(1.0), draw_scale: Some(1.0), draw_scale_3d: Some([1.0; 3]),
    });
    for class in [
        "LineagePawn", "MFighter", "FFighter", "MMagic", "FMagic", "MElf", "FElf",
        "MDarkElf", "FDarkElf", "MOrc", "FOrc", "MShaman", "FShaman", "MDwarf", "FDwarf",
    ] {
        assert_eq!(client.class_render_defaults(&format!("LineageWarrior.{class}")).unwrap(), actor, "{class}");
    }
    assert_eq!(client.class_render_defaults("Core.Object").unwrap(), ClassRenderDefaults::default());
    assert_eq!(client.class_render_defaults("lineagewarrior.mfighter").unwrap(), actor);
    for invalid in ["Actor", "Engine.", ".Actor", "Engine.NoSuchClass", "NoSuchPackage.Actor"] {
        assert!(client.class_render_defaults(invalid).is_err(), "{invalid}");
    }
}
