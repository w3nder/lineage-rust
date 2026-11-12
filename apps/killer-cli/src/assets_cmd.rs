use clap::Args as ClapArgs;
use killer_assets::client::Client;
use killer_assets::textures::write_png;
use std::error::Error;
use std::path::PathBuf;
use ue2_assets::{Property, object_properties, read_skeletal_mesh, read_static_mesh, read_texture};
use ue2_package::{ObjectRef, Package};

pub(crate) fn client_dir(explicit: Option<PathBuf>) -> PathBuf {
    explicit.or_else(|| std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)).unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("L2Killer"))
}

#[derive(ClapArgs)]
pub struct PackageArgs {
    file: PathBuf,
}

pub fn package(args: PackageArgs) -> Result<(), Box<dyn Error>> {
    let raw = std::fs::read(&args.file)?;
    let file = l2_crypto::decrypt(&raw, &args.file).map_err(|e| format!("{e:?}"))?;
    let package = Package::parse(&file).map_err(|e| format!("{e:?}"))?;
    println!("v{} licensee {} | {} names, {} imports, {} exports", package.version, package.licensee, package.names().len(), package.imports().len(), package.exports().len());
    for (i, export) in package.exports().iter().enumerate() {
        println!("{i:6} {:<22} {:<48} {:>9} @ {:#x}", package.class_name(export), package.name(export.name), export.serial_size, export.serial_offset);
    }
    Ok(())
}

#[derive(ClapArgs)]
pub struct TextureArgs {
    path: String,
    out: PathBuf,
    #[arg(long)]
    client: Option<PathBuf>,
}

pub fn texture(args: TextureArgs) -> Result<(), Box<dyn Error>> {
    let mut client = Client::open(&client_dir(args.client))?;
    let image = client.catalog().texture(&args.path).ok_or_else(|| format!("textura não encontrada: {}", args.path))?;
    write_png(&args.out, image.width, image.height, &image.rgba)?;
    println!("{} → {} ({}x{})", args.path, args.out.display(), image.width, image.height);
    Ok(())
}

#[derive(ClapArgs)]
pub struct LevelArgs {
    tile: String,
    #[arg(long)]
    client: Option<PathBuf>,
    #[arg(long, value_delimiter = ',')]
    near: Option<Vec<f32>>,
    #[arg(long, default_value_t = 1000.0)]
    radius: f32,
    #[arg(long)]
    draws: bool,
    #[arg(long)]
    scenes: bool,
}

pub fn level(args: LevelArgs) -> Result<(), Box<dyn Error>> {
    let mut client = Client::open(&client_dir(args.client))?;
    let loaded = client.load_level(&args.tile)?;
    let level = &loaded.level;
    let mut classes = std::collections::BTreeMap::new();
    for actor in &level.actors {
        *classes.entry(actor.class.as_str()).or_insert(0usize) += 1;
    }
    let meshes: std::collections::BTreeSet<_> = level.actors.iter().filter_map(|a| a.static_mesh.as_deref()).collect();
    println!("{}: {} actors, {} static meshes distintos, {} polígonos BSP, {} terrenos", loaded.name, level.actors.len(), meshes.len(), level.bsp.len(), level.terrains.len());
    for (class, n) in classes {
        println!("{n:6}  {class}");
    }
    let mut bsp_materials = std::collections::BTreeMap::new();
    for polygon in &level.bsp {
        *bsp_materials.entry((polygon.material.as_deref().unwrap_or(""), polygon.flags)).or_insert(0usize) += 1;
    }
    for ((material, flags), n) in bsp_materials {
        println!("{n:6}  BSP {material} flags {flags:#010x}");
    }
    if let Some(center) = &args.near {
        let center = glam::Vec3::from_slice(center);
        let zones = killer_assets::scene::zone_lighting(level);
        for actor in &level.actors {
            let location = glam::Vec3::from(actor.placement.location);
            if location.distance(center) <= args.radius && actor.static_mesh.is_some() {
                let zone = level.bsp_tree.zone_at(actor.placement.location);
                println!(
                    "{:<24} {:<40} {:?} rot {:?} escala {:?} skins {:?} zona {} sol {} cores {} unlit {}",
                    actor.name,
                    actor.static_mesh.as_deref().unwrap_or(""),
                    actor.placement.location,
                    actor.placement.rotation,
                    actor.scale,
                    actor.skins,
                    zone,
                    killer_assets::scene::sun_affect(&zones, zone),
                    actor.lighting.len(),
                    actor.unlit
                );
            }
        }
    }
    if args.draws {
        print_draws(&mut client, &loaded, args.near.as_deref().map(glam::Vec3::from_slice), args.radius);
    }
    if args.scenes {
        print_scenes(level);
    }
    for terrain in &level.terrains {
        println!("terreno {} em {:?} escala {:?}, {} camadas", terrain.heightmap, terrain.location, terrain.scale, terrain.layers.len());
        for layer in &terrain.layers {
            println!("   camada {} máscara {:?} escala {:?}", layer.material, layer.alpha_map, layer.scale);
        }
    }
    Ok(())
}

fn print_draws(client: &mut Client, loaded: &killer_assets::client::LoadedLevel, near: Option<glam::Vec3>, radius: f32) {
    let scene = killer_assets::scene::build_scene(client, &loaded.level);
    let mut passes = std::collections::BTreeMap::new();
    for draw in &scene.draws {
        let blend = draw.material.passes.first().map_or("invisible".to_owned(), |pass| format!("{:?}/{:?}", pass.src, pass.dst));
        let entry = passes.entry(format!("{:?} {:?} {blend}", draw.pass, draw.kind)).or_insert((0usize, 0usize));
        entry.0 += 1;
        entry.1 += draw.world_index_count() / 3;
    }
    for (key, (draws, triangles)) in &passes {
        println!("{draws:6} draws {triangles:9} triângulos  {key}");
    }
    let mut translucent: Vec<(&killer_assets::scene::MeshDraw, glam::Vec3, glam::Vec3)> = scene
        .draws
        .iter()
        .filter(|draw| draw.pass == killer_assets::scene::Pass::Translucent)
        .map(|draw| {
            let (mut lo, mut hi) = (glam::Vec3::splat(f32::MAX), glam::Vec3::splat(f32::MIN));
            for v in draw.world_vertices() {
                lo = lo.min(glam::Vec3::from(v.position));
                hi = hi.max(glam::Vec3::from(v.position));
            }
            (draw, lo, hi)
        })
        .filter(|(_, lo, hi)| near.is_none_or(|center| center.clamp(*lo, *hi).distance(center) <= radius))
        .collect();
    translucent.sort_by(|a, b| {
        let area = |(lo, hi): (glam::Vec3, glam::Vec3)| (hi.x - lo.x) * (hi.y - lo.y);
        area((b.1, b.2)).total_cmp(&area((a.1, a.2)))
    });
    for (draw, lo, hi) in translucent.iter().take(20) {
        println!(
            "translúcido {:?} {:<44} máscara {:?} {:5} triângulos x {:.0}..{:.0} y {:.0}..{:.0} z {:.0}..{:.0} zona {} {}",
            draw.kind,
            killer_assets::scene::first_texture_path(&draw.material),
            draw.mask,
            draw.world_index_count() / 3,
            lo.x,
            hi.x,
            lo.y,
            hi.y,
            lo.z,
            hi.z,
            draw.zone,
            draw.material.summary()
        );
    }
}

#[derive(ClapArgs)]
pub struct ObjectArgs {
    file: PathBuf,
    name: String,
}

pub fn object(args: ObjectArgs) -> Result<(), Box<dyn Error>> {
    let raw = std::fs::read(&args.file)?;
    let file = l2_crypto::decrypt(&raw, &args.file).map_err(|e| format!("{e:?}"))?;
    let package = Package::parse(&file).map_err(|e| format!("{e:?}"))?;
    let exports = package.exports();
    let wanted = args.name.to_lowercase();
    let found = exports.iter().enumerate().filter(|(index, export)| {
        package.name(export.name).eq_ignore_ascii_case(&args.name) || package.object_path(ObjectRef::Export(*index)).to_lowercase() == wanted
    });
    let mut any = false;
    for (index, export) in found {
        any = true;
        println!("{index} {} {} ({} bytes)", package.class_name(export), package.object_path(ObjectRef::Export(index)), export.serial_size);
        let class = package.class_name(export);
        if ["Class", "Struct", "State", "Function", "Enum", "Const"].iter().any(|kind| class.eq_ignore_ascii_case(kind)) {
            println!("   (UStruct: o serial começa pelo cabeçalho de classe/função e não por uma lista de propriedades)");
            continue;
        }
        let properties = object_properties(&package, &file, export).map_err(|e| format!("{e:?}"))?;
        for property in &properties {
            println!("   {} = {}", property.name, describe(&package, property));
        }
        if package.class_name(export).eq_ignore_ascii_case("Texture") {
            let texture = read_texture(&package, &file, export).map_err(|e| format!("{e:?}"))?;
            let mips: Vec<String> = texture.mips.iter().map(|mip| format!("{}x{}", mip.width, mip.height)).collect();
            println!("   formato {:?}, paleta {:?}, mips {}", texture.format, texture.palette, mips.join(" "));
        }
        if package.class_name(export).eq_ignore_ascii_case("SkeletalMesh") {
            let mesh = read_skeletal_mesh(&package, &file, export).map_err(|e| format!("{e:?}"))?;
            println!("   escala {:?} origem {:?} rotação {:?}", mesh.scale, mesh.origin, mesh.rotation);
            println!("   tipo de cabelo {} apelidos {:?}", mesh.hair_kind, mesh.aliases);
            println!("   ossos {}", mesh.bones.len());
            for (index, bone) in mesh.bones.iter().enumerate() {
                println!("     {index} {} (pai {}) pos {:?} rot {:?}", bone.name, bone.parent, bone.position, bone.rotation);
            }
            let mut weighted: Vec<usize> = mesh
                .vertices
                .iter()
                .flat_map(|v| v.bones.iter().zip(v.weights).filter(|(_, w)| *w > 0.0).map(|(b, _)| usize::from(*b)))
                .collect();
            weighted.sort_unstable();
            weighted.dedup();
            println!("   ossos com peso {:?}", weighted);
            if let Some((low, high)) = vertex_bounds(mesh.vertices.iter().map(|v| v.position)) {
                println!("   vértices {} de {low:?} a {high:?}", mesh.vertices.len());
            }
            for (section, material) in mesh.sections.iter().zip(&mesh.materials) {
                println!("   seção @{} {} primitivas{} material {}", section.first_index, section.primitives, if section.strip { " (strip)" } else { "" }, package.object_path(*material));
            }
        }
        if package.class_name(export).eq_ignore_ascii_case("StaticMesh") {
            let mesh = read_static_mesh(&package, &file, export).map_err(|e| format!("{e:?}"))?;
            for (section, material) in mesh.sections.iter().zip(&mesh.materials) {
                println!("   seção @{} {} primitivas{} material {}", section.first_index, section.primitives, if section.strip { " (strip)" } else { "" }, package.object_path(*material));
            }
        }
    }
    if !any {
        return Err(format!("objeto não encontrado: {}", args.name).into());
    }
    Ok(())
}

fn print_scenes(level: &ue2_level::Level) {
    for actor in level.actors.iter().filter(|a| a.class.eq_ignore_ascii_case("PlayerStart")) {
        println!("PlayerStart {} {:?} rot {:?}", actor.name, actor.placement.location, actor.placement.rotation);
    }
    for (tag, shots) in &level.shots {
        let warp =
            level.warps.get(tag).map(|w| format!(" warp zona {:?} névoa {:?}", w.zone, w.fog)).unwrap_or_default();
        println!("cena {tag}: {} tomadas{warp}", shots.len());
        for shot in shots {
            println!(
                "   {:?} rot {:?} {:.2} s {} alças {:?}",
                shot.placement.location,
                shot.placement.rotation,
                shot.seconds,
                if shot.curved { "bezier" } else { "reta" },
                shot.handles
            );
        }
    }
}

fn vertex_bounds(positions: impl Iterator<Item = [f32; 3]>) -> Option<([f32; 3], [f32; 3])> {
    positions.fold(None, |bounds, p| {
        let (low, high) = bounds.unwrap_or((p, p));
        Some((std::array::from_fn(|i| low[i].min(p[i])), std::array::from_fn(|i| high[i].max(p[i]))))
    })
}

fn describe(package: &Package, property: &Property<'_>) -> String {
    if let Some(value) = property.bool() {
        return value.to_string();
    }
    if let Some(value) = property.byte() {
        return value.to_string();
    }
    if let Some(value) = property.int() {
        return value.to_string();
    }
    if let Some(value) = property.float() {
        return value.to_string();
    }
    if let Some(value) = property.object(package) {
        return package.object_path(value);
    }
    if let Some(value) = property.name_value(package) {
        return value.to_owned();
    }
    if let Some(value) = property.color() {
        return format!("rgba{value:?}");
    }
    if let Some(value) = property.vector() {
        return format!("{value:?}");
    }
    if let Some(value) = property.rotator() {
        return format!("{value:?}");
    }
    format!("{property:?}")
}

#[derive(ClapArgs)]
pub struct SystemArgs {
    name: String,
    #[arg(long)]
    client: Option<PathBuf>,
    #[arg(long)]
    raw: bool,
}

pub fn system(args: SystemArgs) -> Result<(), Box<dyn Error>> {
    let mut client = Client::open(&client_dir(args.client))?;
    let bytes = client.read_system_file(&args.name)?;
    if args.raw {
        std::io::Write::write_all(&mut std::io::stdout().lock(), &bytes)?;
    } else {
        print!("{}", l2_dat::animnames::decode_text(&bytes));
    }
    Ok(())
}
