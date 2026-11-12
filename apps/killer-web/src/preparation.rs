use std::io::{self, BufWriter, Read, Write};
use std::path::Path;
use wasm_bindgen::prelude::*;
use killer_assets::cache::SourceStamp;

pub const STAMP: SourceStamp = SourceStamp { size: 0, hash: 0 };
struct Sink { callback: js_sys::Function, hash: u64 }
impl Sink { fn new(callback: js_sys::Function) -> Self { Self { callback, hash: 0xcbf2_9ce4_8422_2325 } } }
impl Write for Sink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.callback.call1(&JsValue::NULL, &js_sys::Uint8Array::from(bytes)).map_err(|_| io::Error::other("escrita do cache indisponível"))?;
        for byte in bytes { self.hash = (self.hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3); }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
fn error(reason: impl std::fmt::Display) -> JsValue { JsValue::from_str(&reason.to_string()) }

#[wasm_bindgen]
pub fn prepare_world(files: &js_sys::Array, settings: &str, tile: &str, scene_sink: js_sys::Function, trace_sink: js_sys::Function, progress: Option<js_sys::Function>) -> Result<String, JsValue> {
    let phase = |name: &str| -> Result<(), JsValue> {
        if let Some(callback) = &progress { callback.call1(&JsValue::NULL, &JsValue::from_str(name))?; }
        Ok(())
    };
    phase("read")?;
    console_error_panic_hook::set_once();
    killer_platform::fs::mount(files)?;
    killer_platform::restore_settings(serde_json::from_str(settings).map_err(error)?).map_err(error)?;
    let mut client = killer_assets::client::Client::open(Path::new("/client")).map_err(error)?;
    let loaded = client.load_level(tile).map_err(error)?;
    let options = killer_assets::options::ClientOptions::load(&mut client).map_err(error)?;
    let options = if tile == crate::lobby::LOBBY_MAP { killer_assets::options::ClientOptions { shared_sky: false, ..options } } else { options };
    phase("scene")?;
    let scene = killer_assets::scene::build_scene_with(&mut client, &loaded.level, &options, Default::default());
    drop(loaded);
    drop(client);
    phase("encode")?;
    let mut writer = BufWriter::with_capacity(1 << 20, lz4_flex::frame::FrameEncoder::new(Sink::new(scene_sink)));
    let mut trace = (tile != crate::lobby::LOBBY_MAP).then(|| crate::trace::TraceBuilder::new(&scene));
    killer_assets::cache::encode_consuming(&mut writer, &STAMP, scene, |draw| {
        if let Some(trace) = &mut trace { trace.push(draw); }
    }).map_err(error)?;
    writer.flush().map_err(error)?;
    let sink = writer.into_inner().map_err(error)?.finish().map_err(error)?;
    let scene_hash = format!("{:016x}", sink.hash);
    phase("trace")?;
    let trace = trace.map(crate::trace::TraceBuilder::build);
    phase("trace_write")?;
    let mut writer = BufWriter::with_capacity(1 << 20, Sink::new(trace_sink));
    if let Some(trace) = &trace { trace.write_cache(&mut writer).map_err(error)?; }
    writer.flush().map_err(error)?;
    let trace_hash = format!("{:016x}", writer.get_ref().hash);
    drop(trace);
    phase("done")?;
    Ok(serde_json::json!({"paths":killer_platform::fs::opened_paths(),"sceneHash":scene_hash,"traceHash":trace_hash}).to_string())
}

#[wasm_bindgen]
pub fn fingerprint_files(files: &js_sys::Array, paths: &str) -> Result<String, JsValue> {
    killer_platform::fs::mount(files)?;
    let paths: Vec<String> = serde_json::from_str(paths).map_err(error)?;
    let mut result = Vec::with_capacity(paths.len());
    let mut buffer = vec![0; 1 << 20];
    for path in paths {
        let mut file = killer_platform::fs::File::open(&path).map_err(error)?;
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        loop {
            let count = file.read(&mut buffer).map_err(error)?;
            if count == 0 { break; }
            for byte in &buffer[..count] { hash = (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3); }
        }
        result.push((path, format!("{hash:016x}")));
    }
    serde_json::to_string(&result).map_err(error)
}

#[wasm_bindgen]
pub fn fingerprint_blob(file: web_sys::Blob) -> Result<String,JsValue> {
    let mut file = killer_platform::fs::File::from_blob(file);
    let mut buffer = vec![0; 1 << 20];
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    loop {
        let count = file.read(&mut buffer).map_err(error)?;
        if count == 0 { break; }
        for byte in &buffer[..count] { hash = (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3); }
    }
    Ok(format!("{hash:016x}"))
}
