use killer_assets::client::Client;
use l2_catalog::{Catalog, MissingAsset};
use std::collections::BTreeMap;
use std::path::PathBuf;

const PACKAGE_OBJECTS_WITHOUT_DATA: usize = 4;

#[derive(Default)]
struct Report {
    total: BTreeMap<String, usize>,
    causes: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    classes: BTreeMap<String, usize>,
    sequences: Vec<String>,
}

impl Report {
    fn add(&mut self, source: &str, path: &str, cause: Option<String>) {
        *self.total.entry(source.to_owned()).or_default() += 1;
        if let Some(cause) = cause {
            self.causes.entry(source.to_owned()).or_default().entry(cause).or_default().push(path.to_owned());
        }
    }

    fn failures(&self) -> usize {
        self.causes.values().flat_map(|causes| causes.values()).map(Vec::len).sum()
    }

    fn print(&self) {
        for (source, total) in &self.total {
            let failed: usize = self.causes.get(source).map_or(0, |causes| causes.values().map(Vec::len).sum());
            eprintln!("{source}: {total} referências, {failed} em xadrez/ausentes");
            for (cause, paths) in self.causes.get(source).into_iter().flatten() {
                let mut unique = paths.clone();
                unique.sort();
                unique.dedup();
                eprintln!("    {cause}: {} ({} distintos) ex: {:?}", paths.len(), unique.len(), &unique[..unique.len().min(if std::env::var_os("AUDIT_ALL").is_some() { usize::MAX } else { 3 })]);
            }
        }
        for (class, count) in &self.classes {
            eprintln!("classe {class}: {count}");
        }
        for line in &self.sequences {
            eprintln!("sequência/brilho {line}");
        }
        eprintln!("TOTAL {} referências, {} falhas", self.total.values().sum::<usize>(), self.failures());
    }
}

fn texture_cause(catalog: &mut Catalog, path: &str) -> Option<String> {
    match catalog.try_texture_raw(path) {
        Ok(raw) => {
            let usable = raw.mips.iter().any(|mip| mip.width > 0 && mip.height > 0 && !mip.data.is_empty());
            (!usable).then(|| "sem mips utilizáveis".to_owned())
        }
        Err(MissingAsset::Package) => Some(format!("pacote ausente: {}", path.split('.').next().unwrap_or(""))),
        Err(MissingAsset::WrongClass(class)) => Some(format!("objeto é {class}")),
        Err(other) => Some(other.to_string()),
    }
}

fn client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

#[test]
#[ignore]
fn probe_package_objects() {
    let (Ok(spec), Ok(root)) = (std::env::var("PROBE"), std::env::var("KILLER_CLIENT_DIR")) else { return };
    let (package, filter) = spec.split_once(':').unwrap_or((&spec, ""));
    let mut client = Client::open(&PathBuf::from(root)).unwrap();
    let catalog = client.catalog();
    let paths: Vec<PathBuf> = catalog.source_paths().filter(|p| p.file_stem().is_some_and(|s| s.eq_ignore_ascii_case(package))).map(|p| p.to_owned()).collect();
    for path in paths {
        let raw = std::fs::read(&path).unwrap();
        let bytes = l2_crypto::decrypt(&raw, &path).unwrap().into_owned();
        let package = ue2_package::Package::parse(&bytes).unwrap();
        for (index, export) in package.exports().iter().enumerate() {
            let name = package.object_path(ue2_package::ObjectRef::Export(index));
            if name.to_lowercase().contains(&filter.to_lowercase()) {
                eprintln!("{} [{}]", name, package.class_name(export));
                if std::env::var_os("PROBE_HEX").is_some() {
                    let start = export.serial_offset;
                    let end = start + export.serial_size;
                    let object = &bytes[start..end];
                    eprintln!("    {} bytes: head {:02x?} tail {:02x?}", object.len(), &object[..object.len().min(120)], &object[object.len().saturating_sub(40)..]);
                }
                if std::env::var_os("PROBE_PROPS").is_some()
                    && let Ok(properties) = ue2_assets::object_properties(&package, &bytes[..], export)
                {
                    for property in properties {
                        eprintln!("    {}[{}] obj={:?}", property.name, property.index, property.object(&package).map(|o| package.object_path(o)));
                    }
                }
            }
        }
    }
}

fn without_source(catalog: &mut Catalog, path: &str, class: &str) -> bool {
    let sources: &[&str] = if class.eq_ignore_ascii_case("Shader") { &["Diffuse", "SelfIllumination"] } else { &["Material"] };
    catalog
        .inspect_object(path, class, |package, file, export| {
            let properties = ue2_assets::object_properties(package, file, export).ok()?;
            let has = |name: &str| {
                ue2_assets::find(&properties, name).and_then(|p| p.object(package)).is_some_and(|o| !matches!(o, ue2_package::ObjectRef::Null))
            };
            Some(!sources.iter().any(|name| has(name)))
        })
        .unwrap_or(false)
}

const MATERIAL_CLASSES: [&str; 15] = [
    "Shader", "Combiner", "FinalBlend", "ColorModifier", "TexPanner", "TexRotator", "TexScaler", "TexOscillator",
    "TexEnvMap", "Cubemap", "VertexColor", "ConstantColor", "TexPannerTriggered", "TexOscillatorTriggered", "TexMatrix",
];

#[test]
#[ignore]
fn every_texture_and_material_object_of_every_package_resolves() {
    let Some(mut client) = client() else { return };
    let paths: Vec<PathBuf> = client.catalog().source_paths().map(|p| p.to_owned()).collect();
    let mut report = Report::default();
    for path in paths {
        let Some(stem) = path.file_stem().map(|s| s.to_string_lossy().into_owned()) else { continue };
        let Ok(raw) = std::fs::read(&path) else { continue };
        if raw.starts_with(&[0xCA, 0xFE, 0xF0, 0x0D]) {
            report.add("pacotes", &stem, Some("pacote CAFEF00D".to_owned()));
            continue;
        }
        let Ok(bytes) = l2_crypto::decrypt(&raw, &path).map(|b| b.into_owned()) else {
            report.add("pacotes", &stem, Some("descriptografia falhou".to_owned()));
            continue;
        };
        drop(raw);
        let Ok(package) = ue2_package::Package::parse(&bytes) else {
            report.add("pacotes", &stem, Some("índice ilegível".to_owned()));
            continue;
        };
        report.add("pacotes", &stem, None);
        let mut objects = Vec::new();
        for (index, export) in package.exports().iter().enumerate() {
            let class = package.class_name(export).to_owned();
            let full = package.object_path(ue2_package::ObjectRef::Export(index));
            if class.eq_ignore_ascii_case("Texture") || MATERIAL_CLASSES.iter().any(|c| c.eq_ignore_ascii_case(&class)) {
                objects.push((class, format!("{stem}.{full}#{index}")));
            }
        }
        drop(bytes);
        for (class, object) in objects {
            let source = format!("objeto {class}");
            let cause = if class.eq_ignore_ascii_case("Texture") {
                texture_cause(client.catalog(), &object)
            } else {
                match client.catalog().material(&object) {
                    None if without_source(client.catalog(), &object, &class) => None,
                    None => Some("material não resolvido".to_owned()),
                    Some(material) => {
                        let paths: Vec<String> = material.texture_paths().into_iter().map(str::to_owned).collect();
                        paths.iter().find_map(|inner| texture_cause(client.catalog(), inner))
                    }
                }
            };
            report.add(&source, &object, cause);
        }
        client.catalog().release_files();
    }
    report.print();
    assert_eq!(report.failures(), PACKAGE_OBJECTS_WITHOUT_DATA);
}

#[test]
#[ignore]
fn every_texture_of_every_tile_scene_resolves() {
    use killer_assets::options::ClientOptions;
    use killer_assets::scene::build_scene_with;
    let Some(mut client) = client() else { return };
    let options = ClientOptions::load(&mut client).unwrap();
    let mut report = Report::default();
    let mut seen = std::collections::HashSet::new();
    let mut tiles = 0;
    let mut nulls = 0;
    for x in 10..=30 {
        for y in 10..=30 {
            let name = format!("{x}_{y}");
            let Ok(loaded) = client.load_level(&name) else { continue };
            tiles += 1;
            let scene = build_scene_with(&mut client, &loaded.level, &options, Default::default());
            let mut paths: Vec<(&str, String)> = Vec::new();
            for draw in scene.material_draws() {
                for path in draw.material.texture_paths() {
                    paths.push(("cena: material", path.to_owned()));
                }
                if let Some(mask) = &draw.mask {
                    paths.push(("cena: máscara alfa", mask.clone()));
                }
            }
            for body in &scene.sky_bodies {
                for path in body.material.texture_paths() {
                    paths.push(("cena: corpo do céu", path.to_owned()));
                }
            }
            for flare in &scene.flares {
                paths.push(("cena: flare", flare.texture.clone()));
            }
            for corona in &scene.coronas {
                for path in corona.material.texture_paths() {
                    paths.push(("cena: corona", path.to_owned()));
                }
            }
            for (source, path) in paths {
                if !seen.insert((source, path.to_lowercase())) {
                    continue;
                }
                if path.is_empty() {
                    nulls += 1;
                    continue;
                }
                let cause = if !path.contains('.') {
                    Some("caminho sem pacote".to_owned())
                } else {
                    texture_cause(client.catalog(), &path)
                };
                report.add(source, &format!("{path} (tile {name})"), cause);
            }
            client.catalog().release_files();
        }
    }
    if let Some(sky) = killer_assets::scene::sky::load_shared(&mut client, &options) {
        for body in &sky.sky_bodies {
            for path in body.material.texture_paths() {
                report.add("céu compartilhado: corpo", path, texture_cause(client.catalog(), path));
            }
        }
        for flare in &sky.flares {
            report.add("céu compartilhado: flare", &flare.texture, texture_cause(client.catalog(), &flare.texture));
        }
        for draw in sky.material_draws() {
            for path in draw.material.texture_paths() {
                report.add("céu compartilhado: material", path, texture_cause(client.catalog(), path));
            }
        }
    }
    eprintln!("{tiles} tiles, {nulls} referências nulas (textura branca, como o material sem textura)");
    report.print();
    assert_eq!(report.failures(), 0);
}
