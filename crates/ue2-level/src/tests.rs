//! A hand-built map package: a mesh actor and a login scene that warps the camera.

use super::*;

const NAMES: &[&str] = &[
    "None",
    "Core",
    "Engine",
    "Package",
    "Class",
    "StaticMesh",
    "LevelInfo",
    "StaticMeshActor",
    "SceneManager",
    "ActionWarp",
    "InterpolationPoint",
    "L2_Lobby",
    "BTree_A",
    "Level",
    "Location",
    "Rotation",
    "DrawScale",
    "DrawScale3D",
    "Tag",
    "Group",
    "Actions",
    "IntPoint",
    "Vector",
    "Rotator",
    "Logon_Warp",
    "None,7thLobby",
    "LevelInfo0",
    "Tree",
    "Scene",
    "Warp",
    "Point",
    "bSunAffect",
    "Light",
    "Lamp",
    "LightRadius",
    "LightBrightness",
    "LightHue",
    "EmitterLightType",
    "bTimeLight",
    "L2GameEvent",
    "EventID",
    "EventCmd",
];

fn name(text: &str) -> i32 {
    i32::try_from(NAMES.iter().position(|name| *name == text).unwrap()).unwrap()
}

fn compact(value: i32, out: &mut Vec<u8>) {
    let mut rest = value.unsigned_abs();
    let mut first = u8::try_from(rest & 0x3F).unwrap() | if value < 0 { 0x80 } else { 0 };
    rest >>= 6;
    if rest > 0 {
        first |= 0x40;
    }
    out.push(first);
    while rest > 0 {
        let mut byte = u8::try_from(rest & 0x7F).unwrap();
        rest >>= 7;
        if rest > 0 {
            byte |= 0x80;
        }
        out.push(byte);
    }
}

fn compacts(values: &[i32]) -> Vec<u8> {
    let mut out = Vec::new();
    for &value in values {
        compact(value, &mut out);
    }
    out
}

/// A property with a one-byte size, which every value here fits.
fn property(out: &mut Vec<u8>, property: &str, kind: u8, struct_name: Option<&str>, value: &[u8]) {
    compact(name(property), out);
    out.push(kind | 5 << 4);
    if let Some(struct_name) = struct_name {
        compact(name(struct_name), out);
    }
    out.push(u8::try_from(value.len()).unwrap());
    out.extend_from_slice(value);
}

fn flag(out: &mut Vec<u8>, property: &str, value: bool) {
    compact(name(property), out);
    out.push(3 | if value { 0x80 } else { 0 });
}

fn floats(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

fn ints(values: &[i32]) -> Vec<u8> {
    values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

/// Object references: exports count up from 1, imports down from -1.
const LEVEL_INFO: i32 = 1;
const WARP: i32 = 4;
const POINT: i32 = 6;
const TREE_MESH: i32 = -8;

fn map() -> Vec<u8> {
    map_with_mesh(TREE_MESH)
}

fn map_with_mesh(mesh: i32) -> Vec<u8> {
    let level = |out: &mut Vec<u8>| property(out, "Level", 5, None, &compacts(&[LEVEL_INFO]));
    let mut objects: Vec<(i32, &str, Vec<u8>)> = Vec::new();

    let mut info = Vec::new();
    level(&mut info);
    objects.push((-6, "LevelInfo0", info));

    let mut tree = Vec::new();
    level(&mut tree);
    property(&mut tree, "Location", 10, Some("Vector"), &floats(&[1.0, 2.0, 3.0]));
    property(&mut tree, "DrawScale", 4, None, &floats(&[2.0]));
    property(&mut tree, "DrawScale3D", 10, Some("Vector"), &floats(&[1.0, 2.0, 3.0]));
    property(&mut tree, "StaticMesh", 5, None, &compacts(&[mesh]));
    property(&mut tree, "Group", 6, None, &compacts(&[name("None,7thLobby")]));
    flag(&mut tree, "bSunAffect", true);
    let mut events = Vec::new();
    compact(3, &mut events);
    property(&mut events, "EventID", 2, None, &ints(&[2_418_001]));
    property(&mut events, "EventCmd", 1, None, &[1]);
    compact(name("None"), &mut events);
    property(&mut events, "EventCmd", 1, None, &[2]);
    compact(name("None"), &mut events);
    property(&mut events, "EventID", 2, None, &ints(&[2_418_001]));
    compact(name("None"), &mut events);
    property(&mut tree, "L2GameEvent", 9, None, &events);
    objects.push((-9, "Tree", tree));

    let mut scene = Vec::new();
    level(&mut scene);
    property(&mut scene, "Tag", 6, None, &compacts(&[name("Logon_Warp")]));
    property(&mut scene, "Actions", 9, None, &compacts(&[1, WARP]));
    objects.push((-10, "Scene", scene));

    let mut warp = Vec::new();
    property(&mut warp, "IntPoint", 5, None, &compacts(&[POINT]));
    objects.push((-11, "Warp", warp));

    let mut lamp = Vec::new();
    level(&mut lamp);
    property(&mut lamp, "Location", 10, Some("Vector"), &floats(&[10.0, 20.0, 30.0]));
    property(&mut lamp, "LightRadius", 4, None, &floats(&[30.0]));
    property(&mut lamp, "LightBrightness", 4, None, &floats(&[120.0]));
    property(&mut lamp, "LightHue", 1, None, &[26]);
    property(&mut lamp, "EmitterLightType", 1, None, &[2]);
    flag(&mut lamp, "bTimeLight", true);
    objects.push((-13, "Lamp", lamp));

    let mut point = Vec::new();
    level(&mut point);
    property(&mut point, "Location", 10, Some("Vector"), &floats(&[148_959.1, -252_862.6, -5817.1]));
    property(&mut point, "Rotation", 10, Some("Rotator"), &ints(&[2200, -42971, 0]));
    objects.push((-12, "Point", point));

    // Imports: (class package, class, outer, name).
    let imports = [
        ("Core", "Package", 0, "Engine"),
        ("Core", "Package", 0, "L2_Lobby"),
        ("Core", "Class", -1, "Class"),
        ("Core", "Class", -1, "Package"),
        ("Core", "Class", -1, "StaticMesh"),
        ("Core", "Class", -1, "LevelInfo"),
        ("Core", "Class", -1, "Level"),
        ("Engine", "StaticMesh", -2, "BTree_A"),
        ("Core", "Class", -1, "StaticMeshActor"),
        ("Core", "Class", -1, "SceneManager"),
        ("Core", "Class", -1, "ActionWarp"),
        ("Core", "Class", -1, "InterpolationPoint"),
        ("Core", "Class", -1, "Light"),
    ];

    let mut file = vec![0; 36];
    let name_offset = file.len();
    for text in NAMES {
        compact(i32::try_from(text.len() + 1).unwrap(), &mut file);
        file.extend_from_slice(text.as_bytes());
        file.extend_from_slice(&[0, 0, 0, 0, 0]);
    }
    let import_offset = file.len();
    for (class_package, class, outer, object) in imports {
        compact(name(class_package), &mut file);
        compact(name(class), &mut file);
        file.extend_from_slice(&i32::to_le_bytes(outer));
        compact(name(object), &mut file);
    }
    let mut serials = Vec::new();
    for (_, _, data) in &objects {
        let mut data = data.clone();
        compact(name("None"), &mut data);
        serials.push((file.len(), data.len()));
        file.extend_from_slice(&data);
    }
    let export_offset = file.len();
    for ((class, object, _), (offset, size)) in objects.iter().zip(serials) {
        compact(*class, &mut file);
        compact(0, &mut file);
        file.extend_from_slice(&0i32.to_le_bytes());
        compact(name(object), &mut file);
        file.extend_from_slice(&0u32.to_le_bytes());
        compact(i32::try_from(size).unwrap(), &mut file);
        compact(i32::try_from(offset).unwrap(), &mut file);
    }

    let header: Vec<u32> = [0x9E2A_83C1, 123, 0]
        .into_iter()
        .chain(
            [NAMES.len(), name_offset, objects.len(), export_offset, imports.len(), import_offset]
                .map(|value| u32::try_from(value).unwrap()),
        )
        .collect();
    file.splice(0..36, header.iter().flat_map(|value| value.to_le_bytes()));
    file
}

#[test]
fn level_context_qualifies_exports_and_preserves_imports_and_null() {
    for (mesh, expected) in [(LEVEL_INFO, Some("Lobby.LevelInfo0#0")), (TREE_MESH, Some("L2_Lobby.BTree_A")), (0, None)] {
        let file = map_with_mesh(mesh);
        let package = Package::parse(&file).unwrap();
        let level = read_level_in(&package, &file, Some("Lobby")).unwrap();
        let tree = level.actors.iter().find(|actor| actor.name == "Tree").unwrap();
        assert_eq!(tree.static_mesh.as_deref(), expected);
    }
}

#[test]
#[expect(clippy::float_cmp, reason = "values are written and read back as the same bits")]
fn reads_actors_and_the_scene_camera() {
    let file = map();
    let package = Package::parse(&file).unwrap();
    let level = read_level(&package, &file).unwrap();

    let names: Vec<_> = level.actors.iter().map(|actor| (actor.class.as_str(), actor.name.as_str())).collect();
    assert_eq!(
        names,
        [
            ("LevelInfo", "LevelInfo0"),
            ("StaticMeshActor", "Tree"),
            ("SceneManager", "Scene"),
            ("Light", "Lamp"),
            ("InterpolationPoint", "Point")
        ]
    );
    assert_eq!(
        level.lights,
        [Light {
            name: "Lamp".into(),
            location: [10.0, 20.0, 30.0],
            radius: 775.0,
            brightness: 120.0,
            hue: 26,
            saturation: 255,
            light_type: 1,
            effect: 0,
            emitter_light_type: 2,
            time_light: true,
            period: 32,
            phase: 0,
            cone: 128,
            direction: [1.0, 0.0, 0.0],
            rotation: [0; 3],
            special_lit: false,
            corona: None,
        }]
    );

    let tree = &level.actors[1];
    assert_eq!(tree.static_mesh.as_deref(), Some("L2_Lobby.BTree_A"));
    assert_eq!(tree.groups, ["7thLobby"]);
    assert_eq!(tree.scale, [2.0, 4.0, 6.0]);
    assert_eq!(tree.placement, Placement { location: [1.0, 2.0, 3.0], rotation: [0; 3] });
    assert!(tree.sun_affect);
    assert!(!level.actors[4].sun_affect);
    assert_eq!(tree.l2_events, [L2Event { id: 2_418_001, cmd: LEC_SHOW }, L2Event { id: 0, cmd: LEC_PLAY }, L2Event { id: 2_418_001, cmd: 0 }]);
    assert!(level.actors[4].l2_events.is_empty());

    let camera = Placement { location: [148_959.1, -252_862.6, -5817.1], rotation: [2200, -42971, 0] };
    assert_eq!(
        level.warps.get("Logon_Warp"),
        Some(&Warp { placement: camera, fog: None, zone: None, zone_state: None })
    );
    let warp = Shot { placement: camera, seconds: 0.0, curved: false, handles: [[0.0; 3]; 2] };
    assert_eq!(level.shots.get("Logon_Warp").map(Vec::as_slice), Some(&[warp][..]));
    assert_eq!(level.actors[2].tag.as_deref(), Some("Logon_Warp"));
}

#[test]
fn l2_event_trigger_walks_every_entry_with_a_command_and_the_id() {
    let events = [
        L2Event { id: 7, cmd: LEC_SHOW },
        L2Event { id: 8, cmd: LEC_SHOW },
        L2Event { id: 7, cmd: 0 },
        L2Event { id: 7, cmd: LEC_PLAY },
        L2Event { id: 7, cmd: 3 },
        L2Event { id: 7, cmd: LEC_SHOW },
    ];
    assert_eq!(l2_event_commands(&events, 7).collect::<Vec<_>>(), [LEC_SHOW, LEC_PLAY, 3, LEC_SHOW]);
    assert_eq!(l2_event_commands(&events, 8).collect::<Vec<_>>(), [LEC_SHOW]);
    assert_eq!(l2_event_commands(&events, 0).count(), 0);
    assert_eq!(l2_event_commands(&[], 7).count(), 0);
}
