use killer_platform::fs;
use std::io::{Read,Seek,SeekFrom};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn check_platform(files: js_sys::Array) -> Result<String,JsValue> {
    fn require(condition: bool,message: &str) -> Result<(),JsValue> {
        if condition {Ok(())} else {Err(JsValue::from_str(message))}
    }
    fn io(error: std::io::Error) -> JsValue {JsValue::from_str(&error.to_string())}
    require(fs::mount(&files)?==3,"quantidade de arquivos")?;
    require(fs::is_dir("/CLIENT/System"),"diretório virtual")?;
    require(!fs::is_dir("/client/missing"),"diretório inexistente")?;
    let entries=fs::read_dir("/client").map_err(io)?.collect::<Result<Vec<_>,_>>().map_err(io)?;
    require(entries.len()==1 && entries[0].path()==std::path::Path::new("/client/system"),"índice dos diretórios")?;
    let mut file=fs::File::open("/CLIENT/SYSTEM/fixture.bin").map_err(io)?;
    require(file.metadata().map_err(io)?.len()==65537,"tamanho")?;
    let mut all=Vec::new();
    file.read_to_end(&mut all).map_err(io)?;
    require(all.len()==65537 && all.iter().enumerate().all(|(i,b)|*b==(i%251) as u8),"leitura integral")?;
    file.seek(SeekFrom::Start(8191)).map_err(io)?;
    let mut middle=[0;513];
    file.read_exact(&mut middle).map_err(io)?;
    require(middle.iter().enumerate().all(|(i,b)|*b==((i+8191)%251) as u8),"seek e leitura parcial")?;
    file.seek(SeekFrom::End(-1)).map_err(io)?;
    let mut tail=[0;8];
    require(file.read(&mut tail).map_err(io)?==1 && tail[0]==(65536%251) as u8,"leitura final")?;
    require(file.read(&mut tail).map_err(io)?==0,"EOF")?;
    require(file.seek(SeekFrom::Start(90000)).map_err(io)?==90000 && file.read(&mut tail).map_err(io)?==0,"seek após EOF")?;
    require(file.seek(SeekFrom::Current(-90001)).is_err(),"seek negativo")?;
    require(fs::File::open("/client/system/../fixture.bin").is_err(),"traversal")?;
    require(fs::write("/client/system/fixture.bin",[0]).is_err(),"somente leitura")?;
    require(fs::read("/client/system/empty.bin").map_err(io)?.is_empty(),"arquivo vazio")?;
    let duplicate=js_sys::Array::new();
    duplicate.push(&files.get(0));
    duplicate.push(&files.get(0));
    require(fs::mount(&duplicate).is_err(),"colisão de caminhos")?;
    require(fs::File::open("/client/system/fixture.bin").is_ok(),"mount rejeitado preserva índice")?;
    let mut client=killer_assets::client::Client::open(std::path::Path::new("/client")).map_err(io)?;
    let asset_error=|error: killer_assets::client::AssetError|JsValue::from_str(&error.to_string());
    let original=client.read_system_text("option.ini").map_err(asset_error)?;
    require(original.contains("MyName=True"),"opção original")?;
    client.write_system_text("OPTION.INI","[Game]\nMyName=False\n").map_err(asset_error)?;
    require(client.read_system_text("option.ini").map_err(asset_error)?.contains("MyName=False"),"sobreposição de opções")?;
    require(fs::read("/client/system/option.ini").map_err(io)?==original.as_bytes(),"original preservado")?;
    require(client.write_system_text("engine.dll","invalid").is_err(),"escrita fora das preferências")?;
    let saved=killer_platform::settings();
    killer_platform::restore_settings(Default::default()).map_err(JsValue::from_str)?;
    require(client.read_system_text("option.ini").map_err(asset_error)?==original,"retorno ao original")?;
    killer_platform::restore_settings(saved).map_err(JsValue::from_str)?;
    require(client.read_system_text("option.ini").map_err(asset_error)?.contains("MyName=False"),"restauração das preferências")?;
    let now=killer_platform::time::Instant::now();
    require(now.elapsed().as_secs()<10,"relógio do Worker")?;
    Ok("Verificações passaram: diretórios, leitura integral/parcial, seek, EOF, arquivo vazio, caminhos, somente leitura, mount atômico, preferências restauradas, originais preservados e relógio.".into())
}

#[wasm_bindgen]
pub async fn check_options() -> Result<String,JsValue> {
    crate::runtime::options::check().await.map_err(|error|JsValue::from_str(&error))
}
