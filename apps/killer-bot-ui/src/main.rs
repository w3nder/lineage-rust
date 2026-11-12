mod controller;
mod create_char;
mod data;
mod dto;
mod html;
mod packet_tools;
mod profiles;
mod scripts;
mod secrets;
mod windows;

use controller::{Command, Controller};
use data::DataLoader;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};

struct Bridge {
    sender: Mutex<Sender<Command>>,
    last: Arc<Mutex<Option<dto::Frame>>>,
}

#[tauri::command]
fn snapshot(bridge: State<'_, Bridge>) -> Option<dto::Frame> {
    bridge.last.lock().ok().and_then(|frame| frame.clone())
}

#[tauri::command]
fn command(bridge: State<'_, Bridge>, command: Command) {
    if let Ok(sender) = bridge.sender.lock() {
        let _ = sender.send(command);
    }
}

struct Catalog(Option<Arc<Mutex<DataLoader>>>);

#[tauri::command]
fn static_data(catalog: State<'_, Catalog>) -> dto::StaticDto {
    let Some(loader) = &catalog.0 else { return dto::static_dto(None) };
    let Ok(mut loader) = loader.lock() else { return dto::static_dto(None) };
    dto::static_dto(loader.ready().map(|data| &*data))
}

#[tauri::command]
fn quit(app: tauri::AppHandle) {
    app.exit(0);
}

fn png_response(bytes: Option<Arc<Vec<u8>>>) -> tauri::http::Response<Vec<u8>> {
    match bytes {
        Some(bytes) => tauri::http::Response::builder().header("Content-Type", "image/png").header("Cache-Control", "max-age=86400").body(bytes.as_ref().clone()).unwrap_or_default(),
        None => tauri::http::Response::builder().status(404).body(Vec::new()).unwrap_or_default(),
    }
}

fn icon_for(loader: &Arc<Mutex<DataLoader>>, path: &str) -> Option<Arc<Vec<u8>>> {
    let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    let mut loader = loader.lock().ok()?;
    let data = loader.ready()?;
    let name = match parts.as_slice() {
        ["item", id] => data.item_icon_name(id.parse().ok()?)?,
        ["skill", id] => data.skill_icon_name(id.parse().ok()?, 1)?,
        ["skill", id, level] => data.skill_icon_name(id.parse().ok()?, level.parse().ok()?)?,
        ["texture", name] => (*name).to_owned(),
        _ => return None,
    };
    data.texture_png(&name)
}

fn main() {
    let (sender, receiver) = std::sync::mpsc::channel();
    let last = Arc::new(Mutex::new(None));
    let shared = last.clone();
    let env = |name: &str| std::env::var(name).ok();
    let config = secrets::config_dir(env);
    let loader = config.as_ref().map(|config| Arc::new(Mutex::new(DataLoader::start(data::data_dir(config, env), data::client_url(env)))));
    let icons = loader.clone();
    let result = tauri::Builder::default()
        .manage(Bridge { sender: Mutex::new(sender), last })
        .manage(Catalog(loader.clone()))
        .register_uri_scheme_protocol("gameicon", move |_context, request| match &icons {
            Some(loader) => png_response(icon_for(loader, request.uri().path())),
            None => png_response(None),
        })
        .setup(move |app| {
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let store = secrets::ChainStore(vec![Box::new(secrets::KeyringStore), Box::new(secrets::EnvStore(|name: &str| std::env::var(name).ok()))]);
                let controller = Controller::new(controller::network_connector(), Box::new(store), secrets::profile_dir(|name| std::env::var(name).ok()), loader);
                controller::run(receiver, controller, |frame| {
                    if let Ok(mut slot) = shared.lock() {
                        *slot = Some(frame.clone());
                    }
                    let _ = handle.emit("snapshot", frame);
                });
            });
            let _ = app.get_webview_window("main");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![snapshot, command, quit, static_data])
        .run(tauri::generate_context!());
    if let Err(error) = result {
        eprintln!("falha ao iniciar a janela do bot: {error}");
        std::process::exit(1);
    }
}
