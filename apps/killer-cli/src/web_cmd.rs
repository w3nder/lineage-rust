use clap::Args as ClapArgs;
use killer_assets::cache::{CachedScene, SourceStamp, encode};
use killer_assets::client::Client;
use killer_assets::options::ClientOptions;
use killer_assets::scene::{Neighbours, build_scene_with, sky};
use killer_assets::textures::{BankSource, TextureBank};
use killer_render::headless::Headless;
use std::error::Error;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;

#[derive(ClapArgs)]
pub struct Args {
    #[arg(default_value = "22_22")]
    tile: String,
    #[arg(long)]
    client: Option<PathBuf>,
    #[arg(long, default_value = "web/data")]
    out: PathBuf,
}

pub fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let mut client = Client::open(&crate::assets_cmd::client_dir(args.client))?;
    let loaded = client.load_level(&args.tile)?;
    let options = ClientOptions::load(&mut client)?;
    let scene = build_scene_with(&mut client, &loaded.level, &options, Neighbours::default());
    let sky = options.shared_sky.then(|| sky::load_shared(&mut client, &options)).flatten();
    fs::create_dir_all(&args.out)?;
    fs::write(args.out.join("timeenv.txt"), client.read_system_text("timeenv0.int")?)?;
    let mut gpu = Headless::new(1, 1)?;
    let mut files = Vec::new();
    for (name, scene) in [("world", Some(scene)), ("sky", sky)] {
        let Some(mut scene) = scene else { continue };
        let draws = std::mem::take(&mut scene.draws);
        let lightmaps = std::mem::take(&mut scene.lightmaps);
        let projectors = std::mem::take(&mut scene.projectors);
        let mut parts = vec![scene.clone()];
        scene.emitters.clear();
        scene.moving.clear();
        scene.coronas.clear();
        scene.sky_bodies.clear();
        scene.flares.clear();
        scene.projectors.clear();
        for (chunk_index, chunk) in draws.chunks(256).enumerate() {
            let mut part = scene.clone();
            part.draws = chunk.to_vec();
            let mut groups = std::collections::HashMap::new();
            for draw in &mut part.draws {
                if let Some(group) = draw.lightmap {
                    draw.lightmap = Some(*groups.entry(group).or_insert_with(|| {
                        let local = part.lightmaps.len();
                        part.lightmaps.push(lightmaps[group].clone());
                        local
                    }));
                }
            }
            let first = chunk_index * 256;
            for projector in &projectors {
                let indices: Vec<_> = projector.draws.iter().copied()
                    .filter(|index| (first..first + chunk.len()).contains(index))
                    .map(|index| index - first).collect();
                if !indices.is_empty() {
                    let mut projector = projector.clone();
                    projector.draws = indices;
                    part.projectors.push(projector);
                }
            }
            parts.push(part);
        }
        for (index, scene) in parts.into_iter().enumerate() {
        let mut bank = TextureBank::new();
        gpu.upload(&scene, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        let cached = CachedScene::from_bank(scene, &bank);
        let stamp = SourceStamp::of(name.as_bytes());
        let file = format!("{name}-{index}.scene");
        let path = args.out.join(&file);
        encode(BufWriter::new(File::create(&path)?), &stamp, &cached.scene, &cached.textures, &cached.alpha_maps)?;
        println!("{}: {} draws, {} texturas", path.display(), cached.scene.draws.len(), cached.textures.len());
        files.push(file);
        }
    }
    fs::write(args.out.join("manifest.json"), serde_json::to_vec(&files)?)?;
    Ok(())
}
