use clap::Args as ClapArgs;
use l2_net::game::packets::{
    OP_AUTH_LOGIN_FAIL, OP_CHAR_SELECT_INFO, OP_KEY_PACKET, auth_login, protocol_version,
};
use l2_net::game::{GameCrypt, read_char_select_info};
use l2_net::transport::{GameConnection, authenticate};
use std::error::Error;
use std::time::Duration;

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long, default_value = "server.l2killer.com")]
    host: String,
    #[arg(long, default_value_t = 2106)]
    port: u16,
    #[arg(long, env = "KILLER_ACCOUNT")]
    account: String,
}

pub fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let mut password = match std::env::var("KILLER_PASSWORD") {
        Ok(p) if !p.is_empty() => p,
        _ => rpassword::prompt_password("senha: ")?,
    };
    let timeout = Duration::from_secs(10);
    let auth = authenticate(&args.host, args.port, &args.account, &password, timeout)?;
    password.clear();
    password.shrink_to_fit();
    println!(
        "servidor {} em {} ({} jogadores, {})",
        auth.server.id,
        auth.server.address(),
        auth.server.players,
        if auth.server.online { "online" } else { "offline" }
    );

    let mut game = GameConnection::connect(&auth.server.address(), timeout)?;
    game.send(protocol_version())?;
    let key_packet = game.receive()?;
    if key_packet.first() != Some(&OP_KEY_PACKET) {
        return Err(format!("esperava KeyPacket, veio 0x{:02x}", key_packet.first().copied().unwrap_or(0)).into());
    }
    game.enable_crypt(GameCrypt::from_key_packet(&key_packet)?);
    game.send(auth_login(&args.account, &auth))?;

    loop {
        let body = game.receive()?;
        match body.first().copied() {
            Some(OP_CHAR_SELECT_INFO) => {
                print_characters(&read_char_select_info(&body)?);
                return Ok(());
            }
            Some(OP_AUTH_LOGIN_FAIL) => return Err("game server recusou o AuthLogin".into()),
            Some(op) => println!("pacote 0x{op:02x} com {} bytes ignorado", body.len()),
            None => {}
        }
    }
}

fn print_characters(chars: &[l2_net::game::CharSelectEntry]) {
    if chars.is_empty() {
        println!("nenhum personagem na conta");
        return;
    }
    println!("{:<4} {:<18} {:>5} {:>6} {:>9} {:>9}  posição", "slot", "nome", "nível", "classe", "hp", "mp");
    for (slot, c) in chars.iter().enumerate() {
        println!(
            "{slot:<4} {:<18} {:>5} {:>6} {:>4.0}/{:<4.0} {:>4.0}/{:<4.0}  {} {} {}",
            c.name, c.level, c.class_id, c.hp, c.max_hp, c.mp, c.max_mp, c.x, c.y, c.z
        );
    }
}
