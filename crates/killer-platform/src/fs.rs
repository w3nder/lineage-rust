use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use wasm_bindgen::JsCast;

const CHUNK: u64 = 1 << 20;
const CACHE_LIMIT: usize = 192 << 20;
const READAHEAD: u64 = 8;
const DIRECT: u64 = 16 << 20;

#[derive(Clone)]
enum Entry {
    Local(web_sys::File),
    Remote(Remote),
}

#[derive(Clone)]
struct Remote {
    url: Rc<str>,
    size: u64,
}

#[derive(Default)]
struct Chunks {
    map: HashMap<(Rc<str>, u64), Rc<Vec<u8>>>,
    order: VecDeque<(Rc<str>, u64)>,
    bytes: usize,
}

impl Chunks {
    fn insert(&mut self, key: (Rc<str>, u64), data: Rc<Vec<u8>>) {
        self.bytes += data.len();
        self.order.push_back(key.clone());
        self.map.insert(key, data);
        while self.bytes > CACHE_LIMIT {
            let Some(old) = self.order.pop_front() else { break };
            if let Some(data) = self.map.remove(&old) { self.bytes -= data.len(); }
        }
    }
}

thread_local! {
    static OPENED: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
    static STALE: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
    static FILES: RefCell<BTreeMap<String, Entry>> = const { RefCell::new(BTreeMap::new()) };
    static CHUNKS: RefCell<Chunks> = RefCell::new(Chunks::default());
    static LAST: RefCell<HashMap<Rc<str>, u64>> = RefCell::new(HashMap::new());
    static STATS: Cell<(u64, u64, f64)> = const { Cell::new((0, 0, 0.0)) };
}

fn key(path: impl AsRef<Path>) -> io::Result<String> {
    let path = path.as_ref().to_string_lossy().replace('\\', "/");
    if path.split('/').any(|part| part == "..") {
        return Err(io::ErrorKind::PermissionDenied.into());
    }
    Ok(path.trim_end_matches('/').to_lowercase())
}

pub fn mount(files: &js_sys::Array) -> Result<usize, wasm_bindgen::JsValue> {
    let mut mounted = BTreeMap::new();
    for entry in files.iter() {
        let entry = entry.dyn_into::<js_sys::Array>()?;
        let relative = entry.get(0).as_string().ok_or_else(|| wasm_bindgen::JsValue::from_str("arquivo sem caminho relativo"))?;
        let value = entry.get(1);
        let file = match value.dyn_into::<web_sys::File>() {
            Ok(file) => Entry::Local(file),
            Err(value) => Entry::Remote(remote(&value)?),
        };
        let (_, relative) = relative.split_once('/').ok_or_else(|| wasm_bindgen::JsValue::from_str("selecione a pasta completa do cliente"))?;
        let path = key(format!("/client/{relative}")).map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))?;
        if mounted.insert(path, file).is_some() {
            return Err(wasm_bindgen::JsValue::from_str("arquivos com o mesmo caminho sem distinção de maiúsculas"));
        }
    }
    OPENED.with(|opened| opened.borrow_mut().clear());
    let count = mounted.len();
    FILES.with(|files| *files.borrow_mut() = mounted);
    Ok(count)
}

fn remote(value: &wasm_bindgen::JsValue) -> Result<Remote, wasm_bindgen::JsValue> {
    let url = js_sys::Reflect::get(value, &"url".into())?.as_string().ok_or_else(|| wasm_bindgen::JsValue::from_str("arquivo remoto sem endereço"))?;
    let size = js_sys::Reflect::get(value, &"size".into())?.as_f64().filter(|size| *size >= 0.0 && size.fract() == 0.0).ok_or_else(|| wasm_bindgen::JsValue::from_str("arquivo remoto sem tamanho"))?;
    Ok(Remote { url: url.into(), size: size as u64 })
}

fn fetch_span(remote: &Remote, start: u64, end: u64) -> io::Result<Vec<u8>> {
    let started = js_sys::Date::now();
    let result = fetch_range(remote, start, end);
    let elapsed = js_sys::Date::now() - started;
    STATS.with(|stats| {
        let (requests, bytes, ms) = stats.get();
        stats.set((requests + 1, bytes + result.as_ref().map_or(0, |data| data.len() as u64), ms + elapsed));
    });
    result
}

fn fetch_range(remote: &Remote, start: u64, end: u64) -> io::Result<Vec<u8>> {
    let request = web_sys::XmlHttpRequest::new().map_err(js_error)?;
    request.open_with_async("GET", &remote.url, false).map_err(js_error)?;
    request.set_response_type(web_sys::XmlHttpRequestResponseType::Arraybuffer);
    request.set_request_header("Range", &format!("bytes={start}-{end}")).map_err(js_error)?;
    request.send().map_err(js_error)?;
    let status = request.status().map_err(js_error)?;
    let body = request.response().map_err(js_error)?;
    let bytes = js_sys::Uint8Array::new(&body).to_vec();
    let expected = (end - start + 1) as usize;
    match status {
        206 if bytes.len() == expected => Ok(bytes),
        200 if bytes.len() as u64 == remote.size => Ok(bytes[start as usize..=end as usize].to_vec()),
        404 | 403 => Err(io::ErrorKind::NotFound.into()),
        _ => Err(io::Error::other(format!("falha ao baixar {} (HTTP {status}, {} bytes)", remote.url, bytes.len()))),
    }
}

fn chunk(remote: &Remote, index: u64) -> io::Result<Rc<Vec<u8>>> {
    let key = (remote.url.clone(), index);
    if let Some(data) = CHUNKS.with(|chunks| chunks.borrow().map.get(&key).cloned()) { return Ok(data); }
    let total = remote.size.div_ceil(CHUNK);
    let sequential = index > 0 && LAST.with(|last| last.borrow().get(&remote.url) == Some(&(index - 1)));
    let last = (index + if sequential { READAHEAD } else { 1 }).min(total) - 1;
    let start = index * CHUNK;
    let span = fetch_span(remote, start, ((last + 1) * CHUNK).min(remote.size) - 1)?;
    let mut first = None;
    for block in index..=last {
        let from = ((block - index) * CHUNK) as usize;
        let data = Rc::new(span[from..(from + CHUNK as usize).min(span.len())].to_vec());
        if block == index { first = Some(data.clone()); }
        CHUNKS.with(|chunks| chunks.borrow_mut().insert((remote.url.clone(), block), data));
    }
    LAST.with(|map| map.borrow_mut().insert(remote.url.clone(), last));
    first.ok_or_else(|| io::ErrorKind::UnexpectedEof.into())
}

fn read_direct(remote: &Remote, position: u64, buffer: &mut [u8]) -> io::Result<()> {
    let mut done = 0;
    while done < buffer.len() {
        let start = position + done as u64;
        let take = (buffer.len() - done).min(DIRECT as usize);
        let bytes = fetch_span(remote, start, start + take as u64 - 1)?;
        buffer[done..done + take].copy_from_slice(&bytes);
        done += take;
    }
    Ok(())
}

pub fn remote_stats() -> (u64, u64, f64) { STATS.with(Cell::get) }

pub fn replace(relative: &str, file: &wasm_bindgen::JsValue) -> bool {
    let Ok(file) = file.clone().dyn_into::<web_sys::File>() else { return false };
    let Ok(path) = key(format!("/client/{relative}")) else { return false };
    FILES.with(|files| {
        let mut files = files.borrow_mut();
        let Some(entry) = files.get_mut(&path) else { return false };
        let replaceable = matches!(&*entry, Entry::Remote(remote) if remote.size == file.size() as u64);
        if replaceable { *entry = Entry::Local(file); }
        replaceable
    })
}

pub fn is_dir(path: impl AsRef<Path>) -> bool {
    let Ok(prefix) = key(path).map(|path| format!("{path}/")) else { return false };
    FILES.with(|files| files.borrow().range(prefix.clone()..).next().is_some_and(|(path, _)| path.starts_with(&prefix)))
}

pub struct DirEntry(PathBuf);

impl DirEntry {
    pub fn path(&self) -> PathBuf { self.0.clone() }
}

pub fn read_dir(path: impl AsRef<Path>) -> io::Result<std::vec::IntoIter<io::Result<DirEntry>>> {
    let prefix = format!("{}/", key(path)?);
    FILES.with(|files| {
        let files = files.borrow();
        let children: BTreeSet<_> = files.range(prefix.clone()..).take_while(|(path, _)| path.starts_with(&prefix))
            .filter_map(|(path, _)| path[prefix.len()..].split('/').next())
            .map(|name| PathBuf::from(format!("{prefix}{name}"))).collect();
        if children.is_empty() { return Err(io::ErrorKind::NotFound.into()); }
        Ok(children.into_iter().map(|path| Ok(DirEntry(path))).collect::<Vec<_>>().into_iter())
    })
}

enum Source {
    Blob(web_sys::Blob),
    Remote(Remote),
}

impl Source {
    fn size(&self) -> u64 {
        match self {
            Source::Blob(blob) => blob.size() as u64,
            Source::Remote(remote) => remote.size,
        }
    }
}

pub struct File {
    file: Source,
    position: u64,
    path: Option<String>,
}

pub struct Metadata { len: u64 }

impl Metadata {
    pub fn len(&self) -> u64 { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
}

impl File {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = key(path)?;
        let file = FILES.with(|files| files.borrow().get(&path).cloned()).ok_or(io::ErrorKind::NotFound)?;
        OPENED.with(|opened| { opened.borrow_mut().insert(path.clone()); });
        let (source, local) = match file {
            Entry::Local(file) => (Source::Blob(file.into()), true),
            Entry::Remote(remote) => (Source::Remote(remote), false),
        };
        Ok(Self { file: source, position: 0, path: local.then_some(path) })
    }

    pub fn from_blob(file: web_sys::Blob) -> Self { Self { file: Source::Blob(file), position: 0, path: None } }

    pub fn metadata(&self) -> io::Result<Metadata> { Ok(Metadata { len: self.file.size() }) }
}

impl Read for File {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let count = buffer.len().min(self.file.size().saturating_sub(self.position) as usize);
        if count == 0 { return Ok(0); }
        match &self.file {
            Source::Blob(file) => {
                let blob = file.slice_with_f64_and_f64(self.position as f64, self.position as f64 + count as f64).map_err(js_error)?;
                let reader = web_sys::FileReaderSync::new().map_err(js_error)?;
                let buffer_data = match reader.read_as_array_buffer(&blob) {
                    Ok(data) => data,
                    Err(error) => {
                        if let Some(path) = &self.path {
                            STALE.with(|stale| { stale.borrow_mut().insert(path.clone()); });
                        }
                        return Err(js_error(error));
                    }
                };
                let bytes = js_sys::Uint8Array::new(buffer_data.as_ref());
                if bytes.length() as usize != count { return Err(io::ErrorKind::UnexpectedEof.into()); }
                bytes.copy_to(&mut buffer[..count]);
            }
            Source::Remote(remote) if count as u64 >= 4 * CHUNK => read_direct(remote, self.position, &mut buffer[..count])?,
            Source::Remote(remote) => {
                let mut done = 0;
                while done < count {
                    let position = self.position + done as u64;
                    let data = chunk(remote, position / CHUNK)?;
                    let offset = (position % CHUNK) as usize;
                    let take = (data.len() - offset).min(count - done);
                    buffer[done..done + take].copy_from_slice(&data[offset..offset + take]);
                    done += take;
                }
            }
        }
        self.position += count as u64;
        Ok(count)
    }
}

impl Seek for File {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let position = match from {
            SeekFrom::Start(position) => Some(position),
            SeekFrom::Current(offset) => self.position.checked_add_signed(offset),
            SeekFrom::End(offset) => self.file.size().checked_add_signed(offset),
        }.ok_or(io::ErrorKind::InvalidInput)?;
        self.position = position;
        Ok(position)
    }
}

pub fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut bytes = vec![0; file.metadata()?.len() as usize];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

pub fn write(_path: impl AsRef<Path>, _bytes: impl AsRef<[u8]>) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::PermissionDenied, "pasta do cliente somente para leitura no navegador"))
}

fn js_error(value: wasm_bindgen::JsValue) -> io::Error {
    io::Error::other(value.as_string().unwrap_or_else(|| "falha ao ler arquivo local".into()))
}

pub fn take_stale_paths() -> Vec<String> {
    STALE.with(|stale| std::mem::take(&mut *stale.borrow_mut()).into_iter().map(|path| path.trim_start_matches("/client/").to_owned()).collect())
}

pub fn refresh(relative: &str, file: &wasm_bindgen::JsValue) -> bool {
    let Ok(file) = file.clone().dyn_into::<web_sys::File>() else { return false };
    let Ok(path) = key(format!("/client/{relative}")) else { return false };
    FILES.with(|files| {
        let mut files = files.borrow_mut();
        let Some(entry) = files.get_mut(&path) else { return false };
        let local = matches!(&*entry, Entry::Local(_));
        if local { *entry = Entry::Local(file); }
        local
    })
}

pub fn opened_paths() -> Vec<String> { OPENED.with(|opened| opened.borrow().iter().cloned().collect()) }
