mod app;
mod camera;
mod cursor;
mod download;
mod characters;
mod gpu;
mod splash;
mod ground;
mod ignored;
mod joystick;
mod touch_hud;
mod input;
mod loader;
mod notifies;
mod world_sound;
mod skill_sound;
mod lobby_c4;
#[cfg(not(target_arch = "wasm32"))]
mod ping_cache;
mod server_ping;
mod items;
mod fishing_host;
mod server_objects;
mod landmark;
mod abnormal_host;
mod appendix_effects;
mod pawn_effects;
mod quest_arrow;
mod lobby;
mod pace;
#[cfg(test)]
mod parity;
mod profile;
mod scene_cache;
mod tiles;
mod trace;
mod walk_world;
mod zone_music;
mod ui;
mod ui2d;
mod ui_keys;
mod ui_key_routes;
#[cfg(test)]
mod ui_port_audit;
mod ui_options;
mod ui_option_config;
mod ui_option_apply;
pub mod ui_audio;
mod ui_audio_host;
pub mod ui_video;
pub mod ui_initialize;
mod ui_initialize_run;
mod ui_status_resize;
mod ui_windows_info;
mod ui_shop;
mod ui_trade;
mod ui_hero_tower;
mod ui_olympiad;
mod ui_broadcast;
mod ui_messenger;
mod ui_petition;
mod ui_manor_setting;
mod ui_multisell;
mod ui_macros;
mod ui_henna;
mod ui_pet;
mod ui_special_maps;
mod ui_siege;
mod ui_ssq;
mod ui_minimized;
mod ui_theme;
mod ui_modern;
mod ui_modern_buffs;
mod ui_modern_windows;
mod ui_recipe;
mod ui_recipe_shop;
mod ui_tutorial;
mod ui_help;
mod ui_train;
mod touch;

use clap::Parser;
use std::path::PathBuf;
use winit::event_loop::{ControlFlow, EventLoop};

#[derive(Parser, Default)]
#[command(name = "killer", about = "Cliente Killer")]
struct Args {
    #[arg(long, value_parser = ["classic", "modern"])]
    hud: Option<String>,
    #[arg(long, value_parser = ["c4", "interlude"], env = "KILLER_CHRONICLE")]
    chronicle: Option<String>,
    #[arg(long, env = "KILLER_BASE_CLIENT")]
    base_client: Option<PathBuf>,
    #[arg(long)]
    tile: Option<String>,
    #[arg(long, env = "KILLER_CLIENT_DIR")]
    client: Option<PathBuf>,
    #[arg(long, env = "KILLER_CLIENT_URL")]
    client_url: Option<String>,
    #[arg(long, env = "KILLER_CLIENT_TOKEN")]
    client_token: Option<String>,
    #[arg(long)]
    hour: Option<f32>,
    #[arg(long)]
    screenshot: Option<PathBuf>,
    #[arg(long, requires = "tile")]
    demo: bool,
    #[arg(long, requires = "demo", default_value_t = 300)]
    demo_crowd: usize,
    #[arg(long, conflicts_with = "tile")]
    lobby: Option<String>,
}

#[cfg(target_os = "ios")]
fn init_logging() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info).filter_module("wgpu_hal", log::LevelFilter::Warn).filter_module("naga", log::LevelFilter::Warn);
    if let Some(home) = std::env::var_os("HOME")
        && let Ok(file) = std::fs::File::create(PathBuf::from(home).join("Documents").join("killer.log"))
    {
        builder.target(env_logger::Target::Pipe(Box::new(file)));
    }
    builder.init();
    std::panic::set_hook(Box::new(|info| log::error!("pânico: {info}")));
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
fn init_logging() {
    env_logger::init();
}

#[cfg(not(target_os = "android"))]
fn main() {
    init_logging();
    run(Args::parse(), EventLoop::new());
}

fn run(args: Args, event_loop: Result<EventLoop<()>, winit::error::EventLoopError>) {
    if let Some(base) = args.base_client.clone() {
        killer_assets::client::set_base_root(base);
    }
    if let Some(chronicle) = args.chronicle.as_deref().and_then(l2_dat::Chronicle::parse) {
        l2_dat::Chronicle::set_override(chronicle);
    }
    let event_loop = match event_loop {
        Ok(l) => l,
        Err(e) => {
            eprintln!("erro: {e}");
            std::process::exit(1);
        }
    };
    event_loop.set_control_flow(ControlFlow::Poll);
    let lobby_demo = match args.lobby.as_deref().map(lobby::demo::parse) {
        Some(Some(demo)) => Some(demo),
        Some(None) => {
            eprintln!("erro: --lobby aceita login, servers, characters ou create");
            std::process::exit(2);
        }
        None => None,
    };
    let mut app =
        app::App::new(args.client, args.tile, args.hour, args.screenshot, args.demo, args.demo_crowd, lobby_demo);
    app.configure_hud(args.hud.as_deref());
    app.configure_download(
        args.client_url.or_else(|| option_env!("KILLER_CLIENT_URL").map(str::to_owned)),
        args.client_token.or_else(|| option_env!("KILLER_CLIENT_TOKEN").map(str::to_owned)),
    );
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("erro: {e}");
        std::process::exit(1);
    }
}
