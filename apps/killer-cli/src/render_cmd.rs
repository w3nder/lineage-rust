use crate::assets_cmd::client_dir;
use clap::Args as ClapArgs;
use glam::{Vec3, vec3};
use killer_assets::client::Client;
use killer_assets::options::ClientOptions;
use killer_assets::scene::{Neighbours, build_scene_with, sky};
use killer_assets::textures::{BankSource, TextureBank, write_png};
use killer_assets::weather::Weather;
use killer_render::camera::Camera;
use killer_render::environment::Environment;
use killer_render::headless::Headless;
use killer_render::renderer::Viewer;
use std::error::Error;
use std::path::PathBuf;
use std::time::Instant;

/// Two frames this long settle every fog transition (`FogSpeed` and `DistanceFogBlendTime` are 1 s).
const FOG_SETTLE_SECONDS: f32 = 5.0;

#[derive(ClapArgs)]
pub struct Args {
    tile: String,
    #[arg(long, default_value = "81000,146000,-2200")]
    eye: String,
    #[arg(long, default_value = "83400,148000,-3400")]
    target: String,
    #[arg(long = "fov", default_value_t = 60.0)]
    fov_x: f32,
    #[arg(long, default_value_t = 10.0)]
    near: f32,
    #[arg(long, default_value = "1280x720")]
    size: String,
    #[arg(long, default_value = "giran.png")]
    out: PathBuf,
    #[arg(long)]
    client: Option<PathBuf>,
    #[arg(long, default_value_t = 12.0)]
    hour: f32,
    #[arg(long, default_value_t = 0)]
    env: u8,
    #[arg(long, default_value_t = 0.0)]
    time: f32,
    #[arg(long)]
    without_emitters: bool,
    #[arg(long)]
    without_projectors: bool,
}

fn parse_vec3(text: &str) -> Result<Vec3, Box<dyn Error>> {
    let parts: Vec<f32> = text.split(',').map(|p| p.trim().parse::<f32>()).collect::<Result<_, _>>()?;
    if parts.len() != 3 {
        return Err(format!("esperava x,y,z: {text}").into());
    }
    Ok(vec3(parts[0], parts[1], parts[2]))
}

fn parse_size(text: &str) -> Result<(u32, u32), Box<dyn Error>> {
    let (w, h) = text.split_once('x').ok_or("esperava LARGURAxALTURA")?;
    let width: u32 = w.parse()?;
    let height: u32 = h.parse()?;
    if width == 0 || height == 0 {
        return Err(format!("--size não pode ter dimensão zero: {text}").into());
    }
    Ok((width, height))
}

pub fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let started = Instant::now();
    let mut client = Client::open(&client_dir(args.client))?;
    let loaded = client.load_level(&args.tile)?;
    println!("mapa {} lido em {:.1?}", loaded.name, started.elapsed());
    let mut weather = Weather::load(&mut client)?;
    weather.change_env(args.env, true, 0.02, 0.02, 1.0, args.hour)?;
    weather.state.update(args.hour);
    let mut bank = TextureBank::new();
    let options = ClientOptions::load(&mut client).unwrap_or_default();
    let options = if args.tile == "lobby" { ClientOptions { shared_sky: false, ..options } } else { options };
    let mut scene = build_scene_with(&mut client, &loaded.level, &options, Neighbours::default());
    println!("emitters: {} actors, {} sub-emitters", scene.emitters.len(), scene.emitters.iter().map(|a| a.definition.sprites.len()).sum::<usize>());
    if args.without_emitters { scene.emitters.clear(); }
    let sky = options.shared_sky.then(|| sky::load_shared(&mut client, &options)).flatten();
    if let Some(sky) = &sky {
        println!("céu compartilhado (skylevel.unr): {} draws, {} astros, {} lens flares", sky.draws.len(), sky.sky_bodies.len(), sky.flares.len());
    }
    let triangles: usize = scene.draws.iter().map(|d| d.world_index_count() / 3).sum();
    println!("cena: {} draws, {} triângulos, limites {:?}..{:?} em {:.1?}", scene.draws.len(), triangles, scene.bounds_min, scene.bounds_max, started.elapsed());
    let (width, height) = parse_size(&args.size)?;
    let mut headless = Headless::new(width, height)?;
    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
    headless.renderer_mut().set_time(args.time);
    headless.renderer_mut().set_no_fractal_anim(options.no_fractal_anim);
    headless.renderer_mut().set_projectors(options.projectors && !args.without_projectors);
    headless.upload(&scene, &mut source);
    if let Some(sky) = &sky {
        headless.renderer_mut().add_tile("sky", sky, &mut source);
    }
    println!("GPU pronta em {:.1?}", started.elapsed());
    let camera = Camera { eye: parse_vec3(&args.eye)?, target: parse_vec3(&args.target)?, fov_x_deg: args.fov_x, near: args.near, far: 60000.0 };
    let env = Environment::from_weather(&weather, args.hour);
    println!("hora {:.2}: sol {:?} × {:.3} dir {:?}, ambiente static mesh {:?}; tempo dos materiais {:.2} s", args.hour, env.sun_color, env.sun_brightness, env.sun_dir, env.static_mesh_ambient, args.time);
    let animated = scene.draws.iter().filter(|draw| draw.material.animated()).count();
    println!("materiais animados: {animated} draws");
    let zone = loaded.level.bsp_tree.zone_at(camera.eye.to_array());
    headless.renderer_mut().set_particle_delta(Some(0.0));
    let ticks = (args.time.max(0.0) * 30.0).floor() as u32;
    for _ in 0..ticks { headless.renderer_mut().tick_particles(1.0 / 30.0, camera.eye); }
    headless.renderer_mut().tick_particles(args.time.max(0.0) - ticks as f32 / 30.0, camera.eye);
    headless.renderer_mut().set_viewer(Viewer { zone, position: camera.eye, dt: FOG_SETTLE_SECONDS });
    headless.renderer_mut().update_coronas(&camera, width as f32 / height as f32, FOG_SETTLE_SECONDS, |a, b| killer_render::coronas::scene_clear(&scene, a, b));
    headless.render(&camera, &env)?;
    let rgba = headless.render(&camera, &env)?;
    if let Some(fog) = headless.renderer_mut().fog() {
        println!("névoa: zona {zone}, {} {:.0}..{:.0} cor {:?}{}", if fog.enabled { "ligada" } else { "desligada" }, fog.values.start, fog.values.end, fog.values.color.map(|c| (c * 255.0).round() as u8), if fog.sky.is_some() { " (L2FogInfo)" } else { "" });
    }
    write_png(&args.out, width, height, &rgba)?;
    println!("{} escrito em {:.1?}", args.out.display(), started.elapsed());
    Ok(())
}
