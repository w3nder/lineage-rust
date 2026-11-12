mod assets_cmd;
mod census_cmd;
mod login_cmd;
mod render_cmd;
mod web_cmd;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "killer-cli", about = "Ferramentas do cliente Killer")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Login(login_cmd::Args),
    Package(assets_cmd::PackageArgs),
    Texture(assets_cmd::TextureArgs),
    Level(assets_cmd::LevelArgs),
    Census(census_cmd::CensusArgs),
    Object(assets_cmd::ObjectArgs),
    System(assets_cmd::SystemArgs),
    Render(render_cmd::Args),
    WebExport(web_cmd::Args),
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Login(args) => login_cmd::run(args),
        Command::Package(args) => assets_cmd::package(args),
        Command::Texture(args) => assets_cmd::texture(args),
        Command::Level(args) => assets_cmd::level(args),
        Command::Census(args) => census_cmd::run(args),
        Command::Object(args) => assets_cmd::object(args),
        Command::System(args) => assets_cmd::system(args),
        Command::Render(args) => render_cmd::run(args),
        Command::WebExport(args) => web_cmd::run(args),
    };
    if let Err(err) = result {
        eprintln!("erro: {err}");
        std::process::exit(1);
    }
}
