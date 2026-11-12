use crate::assets_cmd::client_dir;
use clap::Args as ClapArgs;
use killer_assets::client::Client;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use ue2_assets::{Property, find, object_data, object_properties};
use ue2_core::Reader;
use ue2_level::Level;
use ue2_package::{ObjectRef, Package};

const PACKAGE_FOLDERS: [&str; 4] = ["textures", "systextures", "staticmeshes", "animations"];
const CACHE_BUDGET: usize = 1_500_000_000;
const MAX_CHAIN: usize = 64;

const TEXTURE_FORMATS: [&str; 12] =
    ["P8", "RGBA7", "RGB16", "DXT1", "RGB8", "RGBA8", "NODATA", "DXT3", "DXT5", "L8", "G16", "RRRGGGBBB"];

const POLY_FLAGS: [(u32, &str); 23] = [
    (ue2_level::PF_L2WATER, "L2Water"),
    (0x1, "Invisible"),
    (0x2, "Masked"),
    (0x4, "Translucent"),
    (0x8, "NotSolid"),
    (0x10, "Environment"),
    (0x20, "Semisolid"),
    (0x40, "Modulated"),
    (0x80, "FakeBackdrop"),
    (0x100, "TwoSided"),
    (0x200, "AutoUPan"),
    (0x400, "AutoVPan"),
    (0x800, "NoSmooth"),
    (0x1000, "BigWavy"),
    (0x2000, "SpecialPoly"),
    (0x4000, "Flat"),
    (0x10000, "NoMerge"),
    (0x20000, "AlphaTexture"),
    (0x100000, "SpecialLit"),
    (0x400000, "Unlit"),
    (0x4000000, "Portal"),
    (0x8000000, "Mirrored"),
    (0x40000000, "FlatShaded"),
];

const EDITOR_FLAGS: u32 = 0x0080_0000
    | 0x0020_0000
    | 0x0004_0000
    | 0x0100_0000
    | 0x0200_0000
    | 0x1000_0000
    | 0x8000_0000
    | 0x8000
    | 0x10000;

const MATERIAL_REFS: [&str; 17] = [
    "SourceTexture",
    "Material",
    "Material1",
    "Material2",
    "Diffuse",
    "Specular",
    "SpecularityMask",
    "Opacity",
    "SelfIllumination",
    "SelfIlluminationMask",
    "Detail",
    "FallbackMaterial",
    "Mask",
    "Faces",
    "ProjTexture",
    "EnvMap",
    "Palette",
];

#[derive(ClapArgs)]
pub struct CensusArgs {
    #[arg(long)]
    client: Option<PathBuf>,
    #[arg(long, default_value = "10..31")]
    x: String,
    #[arg(long, default_value = "5..31")]
    y: String,
    #[arg(long, value_delimiter = ',')]
    tiles: Option<Vec<String>>,
    #[arg(long)]
    materials: bool,
}

struct Loaded {
    name: String,
    file: Vec<u8>,
    package: Package,
    objects: HashMap<String, usize>,
}

struct Packages {
    files: HashMap<String, PathBuf>,
    loaded: HashMap<String, Option<Rc<Loaded>>>,
    bytes: usize,
}

impl Packages {
    fn index(root: &Path) -> Self {
        let mut files = HashMap::new();
        for entry in fs::read_dir(root).into_iter().flatten().flatten() {
            let folder = entry.path();
            let name = folder.file_name().map(|name| name.to_string_lossy().to_lowercase()).unwrap_or_default();
            if !PACKAGE_FOLDERS.contains(&name.as_str()) {
                continue;
            }
            for file in fs::read_dir(&folder).into_iter().flatten().flatten() {
                let path = file.path();
                if let Some(stem) = path.file_stem() {
                    files.insert(stem.to_string_lossy().to_lowercase(), path);
                }
            }
        }
        Self { files, loaded: HashMap::new(), bytes: 0 }
    }

    fn get(&mut self, name: &str) -> Option<Rc<Loaded>> {
        let key = name.to_lowercase();
        if let Some(found) = self.loaded.get(&key) {
            return found.clone();
        }
        if self.bytes > CACHE_BUDGET {
            self.loaded.clear();
            self.bytes = 0;
        }
        let loaded = self.files.get(&key).and_then(|path| load(&key, path)).map(Rc::new);
        self.bytes += loaded.as_ref().map_or(0, |loaded| loaded.file.len());
        self.loaded.insert(key, loaded.clone());
        loaded
    }
}

fn load(name: &str, path: &Path) -> Option<Loaded> {
    let raw = fs::read(path).ok()?;
    let file = l2_crypto::decrypt_owned(raw, path).ok()?;
    let package = Package::parse(&file).ok()?;
    Some(Loaded { name: name.to_owned(), objects: object_index(&package), file, package })
}

fn object_index(package: &Package) -> HashMap<String, usize> {
    let mut objects = HashMap::new();
    for (index, export) in package.exports().iter().enumerate() {
        objects.insert(package.object_path(ObjectRef::Export(index)).to_lowercase(), index);
        objects.entry(package.name(export.name).to_lowercase()).or_insert(index);
    }
    objects
}

#[derive(Clone)]
struct Located {
    package: Rc<Loaded>,
    index: usize,
}

struct Census {
    tile: String,
    counts: BTreeMap<(String, String), usize>,
    notes: Vec<String>,
}

impl Census {
    fn add(&mut self, kind: &str, key: impl Into<String>) {
        self.bump(kind, key, 1);
    }

    fn bump(&mut self, kind: &str, key: impl Into<String>, n: usize) {
        *self.counts.entry((kind.to_owned(), key.into())).or_insert(0) += n;
    }

    fn note(&mut self, line: impl Into<String>) {
        self.notes.push(line.into());
    }

    fn print(&self) {
        for ((kind, key), n) in &self.counts {
            println!("{}\t{kind}\t{key}\t{n}", self.tile);
        }
        for note in &self.notes {
            println!("{}\tnote\t{note}", self.tile);
        }
    }
}

fn parse_range(text: &str) -> Result<std::ops::Range<i32>, Box<dyn Error>> {
    let (from, to) = text.split_once("..").ok_or("intervalo no formato a..b")?;
    Ok(from.trim().parse()?..to.trim().parse()?)
}

pub fn run(args: CensusArgs) -> Result<(), Box<dyn Error>> {
    let root = client_dir(args.client);
    let maps = fs::read_dir(&root)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.file_name().is_some_and(|name| name.eq_ignore_ascii_case("maps")))
        .ok_or("pasta maps não encontrada")?;
    let tiles: Vec<String> = match args.tiles {
        Some(tiles) => tiles,
        None => {
            let xs = parse_range(&args.x)?;
            let ys = parse_range(&args.y)?;
            xs.flat_map(|x| ys.clone().map(move |y| format!("{x}_{y}"))).collect()
        }
    };
    let mut packages = Packages::index(&root);
    let mut client = Client::open(&root)?;
    for tile in tiles {
        let path = maps.join(format!("{tile}.unr"));
        if !path.exists() {
            println!("{tile}\tstatus\tmissing\t0");
            continue;
        }
        let raw = match fs::read(&path) {
            Ok(raw) => raw,
            Err(err) => {
                println!("{tile}\tstatus\tunreadable: {err}\t0");
                continue;
            }
        };
        let file = match l2_crypto::decrypt_owned(raw, &path) {
            Ok(file) => file,
            Err(err) => {
                println!("{tile}\tstatus\tundecryptable: {err:?}\t0");
                continue;
            }
        };
        let package = match Package::parse(&file) {
            Ok(package) => package,
            Err(err) => {
                println!("{tile}\tstatus\tunparsable: {err:?}\t0");
                continue;
            }
        };
        let level = match ue2_level::read_level(&package, &file) {
            Ok(level) => level,
            Err(err) => {
                println!("{tile}\tstatus\tlevel error: {err:?}\t0");
                continue;
            }
        };
        let mut census = Census { tile: tile.clone(), counts: BTreeMap::new(), notes: Vec::new() };
        census.bump("header", "bytes", file.len());
        census.bump("header", format!("version {} licensee {}", package.version, package.licensee), 1);
        census.bump("header", "exports", package.exports().len());
        census.bump("header", "imports", package.imports().len());
        let map = Rc::new(Loaded { name: tile.to_lowercase(), objects: object_index(&package), package, file });
        tile_census(&mut census, &map, &level, &mut packages, &mut client, args.materials);
        client.catalog().release_files();
        census.print();
        println!("{tile}\tstatus\tok\t1");
    }
    Ok(())
}

fn tile_census(
    census: &mut Census,
    map: &Rc<Loaded>,
    level: &Level,
    packages: &mut Packages,
    client: &mut Client,
    materials: bool,
) {
    let package = &map.package;
    let file = &map.file;
    let mut referenced: BTreeMap<String, Located> = BTreeMap::new();
    let colored: HashSet<&str> =
        level.actors.iter().filter(|a| !a.lighting.is_empty()).map(|a| a.name.as_str()).collect();
    for (index, export) in package.exports().iter().enumerate() {
        let class = package.class_name(export).to_owned();
        census.add("export_class", &class);
        if export.serial_size == 0 || matches!(export.class, ObjectRef::Null) {
            continue;
        }
        let Ok(properties) = object_properties(package, file, export) else {
            census.add("export_unreadable", &class);
            continue;
        };
        if class.ends_with("Emitter") && !class.eq_ignore_ascii_case("Emitter") && find(&properties, "Level").is_none()
        {
            census.add("subemitter", &class);
            for property in &properties {
                census.add("subemitter_prop", format!("{class}.{}", property.name));
            }
            for name in ["Texture", "StaticMesh"] {
                if let Some(object) = find(&properties, name).and_then(|p| p.object(package)) {
                    refer(census, map, packages, &mut referenced, &format!("emitter.{name}"), object);
                }
            }
            if let Some(list) = find(&properties, "CustomMaterials").and_then(|p| p.objects(package)) {
                for object in list {
                    refer(census, map, packages, &mut referenced, "emitter.CustomMaterials", object);
                }
            }
        }
        if find(&properties, "Level").is_none() {
            continue;
        }
        census.add("actor_class", &class);
        for property in &properties {
            census.add("actor_prop", format!("{class}.{}", property.name));
        }
        let flag = |name: &str| find(&properties, name).and_then(Property::bool).unwrap_or(false);
        if let Some(mesh) = find(&properties, "StaticMesh").and_then(|p| p.object(package))
            && !matches!(mesh, ObjectRef::Null)
        {
            census.add("smesh", "actors");
            census.add("smesh", format!("class {class}"));
            let has_colors = colored.contains(package.object_name(ObjectRef::Export(index)));
            census.add("smesh", if has_colors { "with color stream" } else { "without color stream" });
            if flag("bUnlit") {
                census.add("smesh", "bUnlit");
            }
            if flag("bSunAffect") {
                census.add("smesh", "bSunAffect");
            }
            if flag("bShadowCast") {
                census.add("smesh", "bShadowCast");
            }
            if flag("bStaticLighting") {
                census.add("smesh", "bStaticLighting");
            }
            if flag("bHidden") {
                census.add("smesh", "bHidden");
            }
            if let Some(skins) = find(&properties, "Skins").and_then(|p| p.objects(package)) {
                if skins.iter().any(|skin| !matches!(skin, ObjectRef::Null)) {
                    census.add("smesh", "with Skins");
                }
                for skin in skins {
                    refer(census, map, packages, &mut referenced, "skin", skin);
                }
            }
            let path = package.object_path(mesh);
            census.add("smesh_distinct", path.clone());
            if let Some(located) = locate(map, packages, mesh) {
                census.add("smesh", "mesh found");
                for material in mesh_materials(&located) {
                    refer(census, &located.package, packages, &mut referenced, "mesh", material);
                }
            } else {
                census.add("smesh", "mesh missing");
                census.add("smesh_missing", path);
            }
        }
        if let Some(light_type) = find(&properties, "LightType")
            .and_then(Property::byte)
            .or_else(|| class.eq_ignore_ascii_case("Light").then_some(1))
        {
            let effect = find(&properties, "LightEffect").and_then(Property::byte).unwrap_or(0);
            census.add("light", format!("{class} type={light_type} effect={effect}"));
            if flag("bDynamicLight") {
                census.add("light", format!("{class} bDynamicLight"));
            }
        }
        let lower = class.to_ascii_lowercase();
        if lower.ends_with("zoneinfo") || lower.ends_with("foginfo") || lower == "levelinfo" {
            let mut flags = Vec::new();
            for property in &properties {
                if let Some(value) = property.bool() {
                    flags.push(format!("{}={}", property.name, u8::from(value)));
                } else if let Some(value) = property.byte() {
                    flags.push(format!("{}={value}", property.name));
                } else if let Some(value) = property.float() {
                    flags.push(format!("{}={value}", property.name));
                } else if let Some(value) = property.int() {
                    flags.push(format!("{}={value}", property.name));
                } else if let Some(value) = property.color() {
                    flags.push(format!("{}={value:?}", property.name));
                } else if let Some(object) = property.object(package) {
                    flags.push(format!("{}={}", property.name, package.object_path(object)));
                    if property.name.eq_ignore_ascii_case("SkyBox")
                        || property.name.eq_ignore_ascii_case("EnvironmentMap")
                    {
                        refer(census, map, packages, &mut referenced, &format!("zone.{}", property.name), object);
                    }
                }
            }
            census.note(format!("zone {} {class} {}", package.object_name(ObjectRef::Export(index)), flags.join(" ")));
        }
        if class.eq_ignore_ascii_case("TerrainInfo") {
            for layer in properties.iter().filter(|p| p.name.eq_ignore_ascii_case("Layers")) {
                let Some(fields) = layer.fields(package) else { continue };
                if let Some(object) = find(&fields, "Texture").and_then(|p| p.object(package)) {
                    refer(census, map, packages, &mut referenced, "terrain.layer", object);
                }
                if let Some(object) = find(&fields, "AlphaMap").and_then(|p| p.object(package)) {
                    refer(census, map, packages, &mut referenced, "terrain.alphamap", object);
                }
            }
            for deco in find(&properties, "DecoLayers").and_then(|p| p.structs(package)).unwrap_or_default() {
                let shown =
                    find(&deco, "ShowOnTerrain").and_then(|p| p.int().or_else(|| p.byte().map(i32::from))).unwrap_or(1)
                        != 0;
                census.add("deco", if shown { "shown" } else { "hidden" });
                if let Some(object) = find(&deco, "StaticMesh").and_then(|p| p.object(package)) {
                    census.add("deco_mesh", package.object_path(object));
                    if shown && let Some(located) = locate(map, packages, object) {
                        for material in mesh_materials(&located) {
                            refer(census, &located.package, packages, &mut referenced, "deco.mesh", material);
                        }
                    }
                }
                if let Some(object) = find(&deco, "DensityMap").and_then(|p| p.object(package)) {
                    refer(census, map, packages, &mut referenced, "deco.densitymap", object);
                }
            }
            if let Some(object) = find(&properties, "TerrainMap").and_then(|p| p.object(package)) {
                refer(census, map, packages, &mut referenced, "terrain.heightmap", object);
            }
        }
        if class.eq_ignore_ascii_case("Projector")
            && let Some(object) = find(&properties, "ProjTexture").and_then(|p| p.object(package))
        {
            refer(census, map, packages, &mut referenced, "projector", object);
        }
        if let Some(object) = find(&properties, "Texture").and_then(|p| p.object(package))
            && !matches!(object, ObjectRef::Null)
            && find(&properties, "StaticMesh").is_none()
        {
            refer(census, map, packages, &mut referenced, &format!("{class}.Texture"), object);
        }
        if let Some(object) = find(&properties, "AmbientSound").and_then(|p| p.object(package))
            && !matches!(object, ObjectRef::Null)
        {
            census.add("sound", package.object_path(object));
        }
        if class.eq_ignore_ascii_case("FluidSurfaceInfo") || class.starts_with("L2Water") || lower.contains("fluid") {
            let mut flags = Vec::new();
            for property in &properties {
                if let Some(v) = property.int() {
                    flags.push(format!("{}={v}", property.name));
                } else if let Some(v) = property.float() {
                    flags.push(format!("{}={v}", property.name));
                } else if let Some(v) = property.byte() {
                    flags.push(format!("{}={v}", property.name));
                } else if let Some(v) = property.bool() {
                    flags.push(format!("{}={}", property.name, u8::from(v)));
                } else if let Some(object) = property.object(package) {
                    flags.push(format!("{}={}", property.name, package.object_path(object)));
                } else if let Some(objects) = property.objects(package) {
                    flags.push(format!(
                        "{}=[{}]",
                        property.name,
                        objects.iter().map(|o| package.object_path(*o)).collect::<Vec<_>>().join(",")
                    ));
                }
            }
            census.note(format!("fluid {} {class} {}", package.object_name(ObjectRef::Export(index)), flags.join(" ")));
        }
    }
    for polygon in &level.bsp {
        if let Some(material) = &polygon.material {
            census.add("bsp_material", material.clone());
        }
    }
    for polygon in level.bsp.iter().filter(|p| p.material.is_some()) {
        let path = polygon.material.clone().unwrap_or_default();
        if !referenced.contains_key(&path) {
            let object = find_object(map, &path);
            if let Some(object) = object {
                refer(census, map, packages, &mut referenced, "bsp", object);
            }
        }
    }
    bsp_census(census, map);
    census.bump("bsp", "polygons drawn", level.bsp.len());
    census.bump("bsp", "zones", level.zones.len());
    census.bump("bsp", "zone infos", level.zone_infos.len());
    census.bump("terrain", "actors", level.terrains.len());
    for terrain in &level.terrains {
        census.bump("terrain", "layers", terrain.layers.len());
        census.bump("terrain", "deco layers shown", terrain.deco_layers.len());
        census.bump("terrain", "sectors", terrain.sectors.len());
        let states: BTreeSet<usize> = terrain.sectors.iter().map(|s| s.intensities.len()).collect();
        for state in states {
            census.add("terrain", format!("intensity maps per sector = {state}"));
        }
        census.note(format!(
            "terrain {} scale {:?} layers {} deco {} visible_words {}",
            terrain.heightmap,
            terrain.scale,
            terrain.layers.len(),
            terrain.deco_layers.len(),
            terrain.visible_quads.len()
        ));
    }
    census.bump("emitter", "actors", level.emitters.len());
    census.bump("emitter", "sprite emitters enabled", level.emitters.iter().flat_map(|e| &e.sprites).filter(|s| !s.disabled && s.class.eq_ignore_ascii_case("SpriteEmitter")).count());
    census.bump(
        "emitter",
        "mesh emitters enabled",
        level.emitters.iter().flat_map(|e| &e.sprites).filter(|s| !s.disabled && s.mesh.is_some()).count(),
    );
    census.bump("emitter", "beam emitters enabled", level.emitters.iter().flat_map(|e| &e.sprites).filter(|s| !s.disabled && s.beam.is_some()).count());
    census.bump("lights", "read", level.lights.len());
    census.bump("actors", "movable", level.actors.iter().filter(|a| a.movement.is_some()).count());
    if materials {
        material_census(census, packages, client, referenced);
    }
}

fn refer(
    census: &mut Census,
    map: &Rc<Loaded>,
    packages: &mut Packages,
    referenced: &mut BTreeMap<String, Located>,
    kind: &str,
    object: ObjectRef,
) {
    if let Some(located) = locate(map, packages, object) {
        let path = full_path(&located);
        census.add("material_use", format!("{kind} {}", class_of(&located)));
        referenced.entry(path).or_insert(located);
    }
}

fn class_of(located: &Located) -> String {
    let package = &located.package.package;
    package.exports().get(located.index).map(|export| package.class_name(export).to_owned()).unwrap_or_default()
}

fn full_path(located: &Located) -> String {
    let package = &located.package.package;
    format!("{}.{}", located.package.name, package.object_path(ObjectRef::Export(located.index))).to_lowercase()
}

fn locate(map: &Rc<Loaded>, packages: &mut Packages, object: ObjectRef) -> Option<Located> {
    match object {
        ObjectRef::Null => None,
        ObjectRef::Export(index) => Some(Located { package: map.clone(), index }),
        ObjectRef::Import(_) => {
            let path = map.package.object_path(object);
            find_in_packages(map, packages, &path)
        }
    }
}

fn find_in_packages(map: &Rc<Loaded>, packages: &mut Packages, path: &str) -> Option<Located> {
    let lower = path.to_lowercase();
    let (package_name, rest) = lower.split_once('.')?;
    if let Some(loaded) = packages.get(package_name) {
        if let Some(&index) =
            loaded.objects.get(rest).or_else(|| loaded.objects.get(rest.rsplit('.').next().unwrap_or(rest)))
        {
            return Some(Located { package: loaded, index });
        }
        return None;
    }
    let index = *map.objects.get(&lower)?;
    Some(Located { package: map.clone(), index })
}

fn find_object(map: &Rc<Loaded>, path: &str) -> Option<ObjectRef> {
    let lower = path.to_lowercase();
    if let Some(&index) = map.objects.get(&lower) {
        return Some(ObjectRef::Export(index));
    }
    let package = &map.package;
    (0..package.imports().len())
        .map(ObjectRef::Import)
        .find(|import| package.object_path(*import).to_lowercase() == lower)
}

fn mesh_materials(located: &Located) -> Vec<ObjectRef> {
    let package = &located.package.package;
    let Some(export) = package.exports().get(located.index) else { return Vec::new() };
    let Ok(properties) = object_properties(package, &located.package.file, export) else { return Vec::new() };
    find(&properties, "Materials")
        .and_then(|p| p.structs(package))
        .unwrap_or_default()
        .iter()
        .filter_map(|fields| find(fields, "Material").and_then(|p| p.object(package)))
        .collect()
}

fn material_census(
    census: &mut Census,
    packages: &mut Packages,
    client: &mut Client,
    referenced: BTreeMap<String, Located>,
) {
    let mut seen: HashSet<String> = HashSet::new();
    let mut queue: Vec<(Located, usize)> = referenced.into_values().map(|located| (located, 0)).collect();
    let mut root_classes: BTreeMap<String, usize> = BTreeMap::new();
    for (located, _) in &queue {
        let class = class_of(located);
        *root_classes.entry(class.clone()).or_insert(0) += 1;
        if class.eq_ignore_ascii_case("StaticMesh") || class.eq_ignore_ascii_case("Package") {
            continue;
        }
        let path = full_path(located);
        match client.catalog().material(&path) {
            Some(material) => census.add("material_resolved", format!("{class} {}", material.summary())),
            None => {
                census.add("material_unresolved", class.clone());
                census.note(format!("unresolved {class} {path}"));
            }
        }
    }
    for (class, n) in root_classes {
        census.bump("material_root", class, n);
    }
    while let Some((located, depth)) = queue.pop() {
        let path = full_path(&located);
        if !seen.insert(path.clone()) || depth > MAX_CHAIN {
            continue;
        }
        let class = class_of(&located);
        census.add("material_class", &class);
        let package = &located.package.package;
        let Some(export) = package.exports().get(located.index) else { continue };
        let Ok(properties) = object_properties(package, &located.package.file, export) else {
            census.add("material_unreadable", &class);
            continue;
        };
        if class.eq_ignore_ascii_case("Texture") {
            let format = find(&properties, "Format").and_then(Property::byte).unwrap_or(0);
            let name = TEXTURE_FORMATS.get(usize::from(format)).copied().unwrap_or("?");
            census.add("texture_format", name);
            let size = |name| find(&properties, name).and_then(Property::int).unwrap_or(0);
            census.add("texture_size", format!("{}x{}", size("USize"), size("VSize")));
            for property in &properties {
                if property.bool() == Some(true) {
                    census.add("texture_flag", property.name);
                }
                if (property.name.eq_ignore_ascii_case("UClampMode")
                    || property.name.eq_ignore_ascii_case("VClampMode"))
                    && let Some(mode) = property.byte()
                {
                    census.add("texture_flag", format!("{}={mode}", property.name));
                }
                if property.name.eq_ignore_ascii_case("LODSet")
                    && let Some(set) = property.byte()
                {
                    census.add("texture_flag", format!("LODSet={set}"));
                }
            }
            if find(&properties, "AnimNext")
                .and_then(|p| p.object(package))
                .is_some_and(|o| !matches!(o, ObjectRef::Null))
            {
                census.add("texture_flag", "AnimNext");
            }
            if find(&properties, "Palette")
                .and_then(|p| p.object(package))
                .is_some_and(|o| !matches!(o, ObjectRef::Null))
            {
                census.add("texture_flag", "Palette");
            }
            for name in ["Detail", "DetailTexture"] {
                if let Some(detail) = find(&properties, name).and_then(|p| p.object(package))
                    && !matches!(detail, ObjectRef::Null)
                {
                    census.add("texture_flag", name);
                    census.note(format!("{name} {path} -> {}", package.object_path(detail)));
                }
            }
            continue;
        }
        let mut detail = Vec::new();
        for property in &properties {
            if let Some(byte) = property.byte() {
                detail.push(format!("{}={byte}", property.name));
            } else if let Some(value) = property.bool() {
                detail.push(format!("{}={}", property.name, u8::from(value)));
            }
        }
        census.add("material_detail", format!("{class} {}", detail.join(" ")));
        for property in &properties {
            if !MATERIAL_REFS.iter().any(|name| property.name.eq_ignore_ascii_case(name)) {
                continue;
            }
            let mut targets = Vec::new();
            if let Some(object) = property.object(package) {
                targets.push(object);
            } else if let Some(objects) = property.objects(package) {
                targets.extend(objects);
            }
            for object in targets {
                let Some(next) = locate(&located.package, packages, object) else {
                    census.add("material_missing_ref", format!("{class}.{}", property.name));
                    continue;
                };
                census.add("material_ref", format!("{class}.{} -> {}", property.name, class_of(&next)));
                queue.push((next, depth + 1));
            }
        }
    }
}

struct ModelStats {
    nodes: usize,
    lightmapped: usize,
    surfaces: usize,
    flags: BTreeMap<u32, usize>,
    points: usize,
}

fn bsp_census(census: &mut Census, map: &Rc<Loaded>) {
    let package = &map.package;
    let mut best: Option<ModelStats> = None;
    for export in package.exports() {
        if !package.class_name(export).eq_ignore_ascii_case("Model") {
            continue;
        }
        let Some(model) = read_model(package, &map.file, export) else { continue };
        if best.as_ref().is_none_or(|b| model.nodes > b.nodes) {
            best = Some(model);
        }
    }
    let Some(ModelStats { nodes, lightmapped, surfaces, flags, points }) = best else { return };
    census.bump("bsp", "nodes", nodes);
    census.bump("bsp", "nodes with lightmap", lightmapped);
    census.bump("bsp", "surfaces", surfaces);
    census.bump("bsp", "points", points);
    for (bits, n) in flags {
        let names: Vec<&str> = POLY_FLAGS.iter().filter(|(bit, _)| bits & bit != 0).map(|(_, name)| *name).collect();
        let unknown = bits & !POLY_FLAGS.iter().fold(0, |acc, (bit, _)| acc | bit);
        let mut label = if names.is_empty() { "none".to_owned() } else { names.join("|") };
        if unknown != 0 {
            label.push_str(&format!("|{unknown:#x}"));
        }
        census.bump("bsp_surface_flags", label, n);
    }
}

fn read_model(package: &Package, file: &[u8], export: &ue2_package::Export) -> Option<ModelStats> {
    let (_, data) = object_data(package, file, export).ok()?;
    let mut reader = Reader::at(data, 0);
    reader.bytes(25 + 16).ok()?;
    let vectors = usize::try_from(reader.compact().ok()?).ok()?;
    reader.bytes(vectors.checked_mul(12)?).ok()?;
    let points = usize::try_from(reader.compact().ok()?).ok()?;
    reader.bytes(points.checked_mul(12)?).ok()?;
    let nodes = usize::try_from(reader.compact().ok()?).ok()?;
    if nodes == 0 {
        return None;
    }
    let mut lightmapped = 0;
    for _ in 0..nodes {
        reader.bytes(16 + 8 + 1).ok()?;
        for _ in 0..7 {
            reader.compact().ok()?;
        }
        reader.bytes(32 + 3).ok()?;
        reader.bytes(4 * 4).ok()?;
        if reader.i32().ok()? >= 0 {
            lightmapped += 1;
        }
    }
    let lineage = package.licensee >= 23;
    let surfaces = usize::try_from(reader.compact().ok()?).ok()?;
    let mut flags = BTreeMap::new();
    for _ in 0..surfaces {
        reader.compact().ok()?;
        let bits = reader.u32().ok()?;
        *flags.entry(bits & !EDITOR_FLAGS).or_insert(0) += 1;
        for _ in 0..6 {
            reader.compact().ok()?;
        }
        reader.bytes(20).ok()?;
        if lineage {
            reader.i32().ok()?;
        }
    }
    Some(ModelStats { nodes, lightmapped, surfaces, flags, points })
}
