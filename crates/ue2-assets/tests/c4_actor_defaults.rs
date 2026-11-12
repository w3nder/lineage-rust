use std::path::PathBuf;
use ue2_assets::{class_default_properties, find, object_data};
use ue2_core::Reader;
use ue2_package::Package;

#[test]
#[ignore]
fn l2_color_modifier_default_properties_are_read_from_engine_package() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let path = root.join("system/Engine.u");
    let raw = std::fs::read(&path).unwrap();
    let file = l2_crypto::decrypt(&raw, &path).unwrap().into_owned();
    let package = Package::parse(&file).unwrap();
    let mut sources_checked = 0;
    for class in ["Material", "Modifier", "ColorModifier", "L2ColorModifier"] {
        let export = package
            .exports()
            .iter()
            .find(|export| package.class_name(export) == "Class" && package.name(export.name) == class)
            .unwrap_or_else(|| panic!("classe {class} ausente"));
        let properties = class_default_properties(&package, &file, export).unwrap();
        eprintln!(
            "Engine.{class} super={} export@{:x}: {properties:?}",
            package.object_path(export.super_class),
            export.serial_offset
        );
        if class == "Material" {
            let reference = find(&properties, "DefaultMaterial").and_then(|property| property.object(&package));
            assert_eq!(reference.map(|object| package.object_path(object)), Some("DefaultTexture".into()));
        }
        if matches!(class, "ColorModifier" | "L2ColorModifier") {
            assert_eq!(package.object_path(export.super_class), "Modifier");
            assert_eq!(find(&properties, "Color").and_then(|p| p.color()), Some([255; 4]));
            assert_eq!(find(&properties, "RenderTwoSided").and_then(|p| p.bool()), Some(true));
            assert_eq!(find(&properties, "AlphaBlend").and_then(|p| p.bool()), Some(true));
            assert!(find(&properties, "AlphaOp").is_none());
            assert!(find(&properties, "ColorOp").is_none());
        }
        if class == "L2ColorModifier" {
            let fields: Vec<_> = package
                .exports()
                .iter()
                .filter(|field| {
                    package.object_path(field.outer) == class && package.class_name(field).ends_with("Property")
                })
                .map(|field| (package.name(field.name), package.class_name(field)))
                .collect();
            assert_eq!(
                fields,
                [
                    ("Color", "StructProperty"),
                    ("RenderTwoSided", "BoolProperty"),
                    ("AlphaBlend", "BoolProperty"),
                    ("AlphaOp", "ByteProperty"),
                    ("ColorOp", "ByteProperty")
                ]
            );
        }
        for field in package.exports().iter().filter(|field| package.object_path(field.outer) == class) {
            eprintln!(
                "  {} {} @{:x} size={}",
                package.class_name(field),
                package.name(field.name),
                field.serial_offset,
                field.serial_size
            );
            if class == "L2ColorModifier" && package.class_name(field) == "TextBuffer" {
                let (_, data) = object_data(&package, &file, field).unwrap();
                let mut reader = Reader::at(data, 0);
                let editor_state = [reader.i32().unwrap(), reader.i32().unwrap()];
                let source = reader.string().unwrap();
                assert!(reader.remaining().is_empty());
                sources_checked += 1;
                let enumeration = source.split_once("enum EL2TextureOp").unwrap().1;
                let enumeration = enumeration.split_once('{').unwrap().1.split_once('}').unwrap().0;
                let names: Vec<_> = enumeration.split(',').map(str::trim).filter(|name| !name.is_empty()).collect();
                assert_eq!(
                    names,
                    [
                        "P_DISABLE",
                        "P_SELECTARG1",
                        "P_SELECTARG2",
                        "P_MODULATE",
                        "P_MODULATE2X",
                        "P_MODULATE4X",
                        "P_ADD",
                        "P_ADDSIGNED",
                        "P_ADDSIGNED2X",
                        "P_SUBTRACT",
                        "P_ADDSMOOTH",
                        "P_BLENDDIFFUSEALPHA",
                        "P_BLENDTEXTUREALPHA",
                        "P_BLENDFACTORALPHA",
                        "P_BLENDTEXTUREALPHAPM",
                        "P_BLENDCURRENTALPHA",
                        "P_PREMODULATE",
                        "P_MODULATEALPHA_ADDCOLOR",
                        "P_MODULATECOLOR_ADDALPHA",
                        "P_MODULATEINVALPHA_ADDCOLOR",
                        "P_MODULATEINVCOLOR_ADDALPHA",
                        "P_BUMPENVMAP",
                        "P_BUMPENVMAPLUMINANCE",
                        "P_DOTPRODUCT3",
                        "P_MULTIPLYADD"
                    ]
                );
                eprintln!(
                    "TextBuffer: editor={editor_state:?}, {} bytes de fonte, {} operações",
                    source.len(),
                    names.len()
                );
            }
        }
    }
    assert_eq!(sources_checked, 1);
}

#[test]
#[ignore]
fn serialize_expr_10132200_reads_defaults_of_every_engine_and_warrior_class() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    for name in ["Engine", "LineageWarrior"] {
        let path = root.join("system").join(format!("{name}.u"));
        let raw = std::fs::read(&path).unwrap();
        let file = l2_crypto::decrypt(&raw, &path).unwrap().into_owned();
        let package = Package::parse(&file).unwrap();
        let mut count = 0;
        for export in package.exports().iter().filter(|export| package.class_name(export) == "Class") {
            class_default_properties(&package, &file, export)
                .unwrap_or_else(|error| panic!("{name}.{}: {error:?}", package.name(export.name)));
            count += 1;
        }
        assert!(count > 0);
        eprintln!("{name}: {count} classes com defaults lidos");
    }
}

#[test]
#[ignore]
fn render_1069a520_reads_actor_lod_defaults_including_scripted_classes() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut inspected = 0;
    for (name, classes) in [
        ("Engine", &["Actor", "Pawn", "PlayerController", "LineagePlayerController"][..]),
        (
            "LineageWarrior",
            &[
                "LineagePawn",
                "MFighter",
                "FFighter",
                "MMagic",
                "FMagic",
                "MElf",
                "FElf",
                "MDarkElf",
                "FDarkElf",
                "MOrc",
                "FOrc",
                "MShaman",
                "FShaman",
                "MDwarf",
                "FDwarf",
            ][..],
        ),
    ] {
        let path = root.join("system").join(format!("{name}.u"));
        let raw = std::fs::read(&path).unwrap();
        let file = l2_crypto::decrypt(&raw, &path).unwrap().into_owned();
        let package = Package::parse(&file).unwrap();
        for class in classes {
            let export = package.exports().iter().find(|export| {
                package.class_name(export) == "Class" && package.name(export.name).eq_ignore_ascii_case(class)
            });
            let export = export.unwrap_or_else(|| panic!("{name}.{class} ausente"));
            eprintln!("{name}.{class}: super={}", package.object_path(export.super_class));
            let properties = class_default_properties(&package, &file, export)
                .unwrap_or_else(|error| panic!("{name}.{class}: {error:?}"));
            inspected += 1;
            if name == "Engine" && *class == "Actor" {
                assert_eq!(
                    find(&properties, "LODBias").and_then(|value| value.float()).map(f32::to_bits),
                    Some(1.0f32.to_bits())
                );
            }
            eprintln!(
                "{name}.{class}: export@{:x}, LODBias={:?}, DrawScale={:?}, DrawScale3D={:?}, FovAngle={:?}, DesiredFOV={:?}, DefaultFOV={:?}",
                export.serial_offset,
                find(&properties, "LODBias").and_then(|value| value.float()),
                find(&properties, "DrawScale").and_then(|value| value.float()),
                find(&properties, "DrawScale3D").and_then(|value| value.vector()),
                find(&properties, "FovAngle").and_then(|value| value.float()),
                find(&properties, "DesiredFOV").and_then(|value| value.float()),
                find(&properties, "DefaultFOV").and_then(|value| value.float())
            );
        }
    }
    assert_eq!(inspected, 19);
    eprintln!("{inspected} classes lidas incluindo Pawn e PlayerController");
}
