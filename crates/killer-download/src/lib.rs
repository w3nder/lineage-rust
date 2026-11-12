use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const WORKERS: usize = 4;
const ATTEMPTS: usize = 4;
const CHUNK: usize = 256 * 1024;
const MANIFEST: &str = "manifest.json";
const CACHED_MANIFEST: &str = ".manifest.json";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);
const RETRY_PAUSE: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub files: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Starting,
    Running,
    Done,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub phase: Phase,
    pub done_bytes: u64,
    pub total_bytes: u64,
    pub done_files: usize,
    pub total_files: usize,
}

struct Progress {
    done_bytes: AtomicU64,
    total_bytes: AtomicU64,
    done_files: AtomicUsize,
    total_files: AtomicUsize,
    phase: Mutex<Phase>,
}

pub struct Download {
    progress: Arc<Progress>,
}

impl Download {
    pub fn start(base: String, token: Option<String>, root: PathBuf) -> Self {
        Self::start_filtered(base, token, root, Vec::new())
    }

    pub fn start_filtered(base: String, token: Option<String>, root: PathBuf, prefixes: Vec<String>) -> Self {
        let progress = Arc::new(Progress {
            done_bytes: AtomicU64::new(0),
            total_bytes: AtomicU64::new(0),
            done_files: AtomicUsize::new(0),
            total_files: AtomicUsize::new(0),
            phase: Mutex::new(Phase::Starting),
        });
        let shared = progress.clone();
        std::thread::spawn(move || {
            let outcome = run(&base, token.as_deref(), &root, &prefixes, &shared);
            *shared.phase.lock().unwrap_or_else(|e| e.into_inner()) = match outcome {
                Ok(()) => Phase::Done,
                Err(message) => Phase::Failed(message),
            };
        });
        Self { progress }
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            phase: self.progress.phase.lock().unwrap_or_else(|e| e.into_inner()).clone(),
            done_bytes: self.progress.done_bytes.load(Ordering::Relaxed),
            total_bytes: self.progress.total_bytes.load(Ordering::Relaxed),
            done_files: self.progress.done_files.load(Ordering::Relaxed),
            total_files: self.progress.total_files.load(Ordering::Relaxed),
        }
    }
}

pub fn wanted(path: &str, prefixes: &[String]) -> bool {
    prefixes.is_empty() || prefixes.iter().any(|prefix| path.to_lowercase().starts_with(&prefix.to_lowercase()))
}

fn run(base: &str, token: Option<&str>, root: &Path, prefixes: &[String], progress: &Progress) -> Result<(), String> {
    let agent = agent();
    let mut manifest = load_manifest(&agent, base, token, root)?;
    manifest.files.retain(|entry| wanted(&entry.path, prefixes));
    for entry in &manifest.files {
        validate(entry)?;
    }
    fs::create_dir_all(root).map_err(|e| format!("{}: {e}", root.display()))?;
    let total: u64 = manifest.files.iter().map(|e| e.size).sum();
    progress.total_bytes.store(total, Ordering::Relaxed);
    progress.total_files.store(manifest.files.len(), Ordering::Relaxed);
    let pending: Vec<&Entry> = manifest
        .files
        .iter()
        .filter(|entry| {
            let present = fs::metadata(root.join(&entry.path)).is_ok_and(|m| m.len() == entry.size);
            if present {
                progress.done_bytes.fetch_add(entry.size, Ordering::Relaxed);
                progress.done_files.fetch_add(1, Ordering::Relaxed);
            }
            !present
        })
        .collect();
    *progress.phase.lock().unwrap_or_else(|e| e.into_inner()) = Phase::Running;
    let next = AtomicUsize::new(0);
    let abort = AtomicBool::new(false);
    let failure: Mutex<Option<String>> = Mutex::new(None);
    std::thread::scope(|scope| {
        for _ in 0..WORKERS.min(pending.len()) {
            scope.spawn(|| {
                while !abort.load(Ordering::Relaxed) {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(entry) = pending.get(index) else { break };
                    match fetch(&agent, base, token, root, entry, progress, &abort) {
                        Ok(()) => {
                            progress.done_files.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(message) => {
                            abort.store(true, Ordering::Relaxed);
                            failure.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert(message);
                        }
                    }
                }
            });
        }
    });
    match failure.into_inner().unwrap_or_else(|e| e.into_inner()) {
        Some(message) => Err(message),
        None => Ok(()),
    }
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_recv_response(Some(RESPONSE_TIMEOUT))
        .build()
        .into()
}

fn load_manifest(agent: &ureq::Agent, base: &str, token: Option<&str>, root: &Path) -> Result<Manifest, String> {
    let cache = root.join(CACHED_MANIFEST);
    let fetched = request(agent, &url(base, MANIFEST), token, None).and_then(|mut response| {
        let mut body = Vec::new();
        response.body_mut().as_reader().read_to_end(&mut body).map_err(|e| e.to_string())?;
        serde_json::from_slice::<Manifest>(&body).map_err(|e| format!("manifest.json inválido: {e}"))
    });
    match fetched {
        Ok(manifest) => {
            if fs::create_dir_all(root).is_ok()
                && let Ok(text) = serde_json::to_vec(&manifest)
            {
                let _ = fs::write(&cache, text);
            }
            Ok(manifest)
        }
        Err(error) => fs::read(&cache)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Manifest>(&bytes).ok())
            .ok_or(error),
    }
}

fn validate(entry: &Entry) -> Result<(), String> {
    let safe = !entry.path.is_empty()
        && !entry.path.contains('\\')
        && Path::new(&entry.path).components().all(|c| matches!(c, Component::Normal(_)));
    if !safe {
        return Err(format!("caminho inválido no manifesto: {}", entry.path));
    }
    if entry.sha256.len() != 64 || !entry.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("sha256 inválido no manifesto: {}", entry.path));
    }
    Ok(())
}

fn url(base: &str, path: &str) -> String {
    let encoded: Vec<String> = path.split('/').map(encode).collect();
    format!("{}/{}", base.trim_end_matches('/'), encoded.join("/"))
}

fn encode(segment: &str) -> String {
    segment
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn request(
    agent: &ureq::Agent,
    url: &str,
    token: Option<&str>,
    from: Option<u64>,
) -> Result<ureq::http::Response<ureq::Body>, String> {
    let mut builder = agent.get(url);
    if let Some(token) = token {
        builder = builder.header("Authorization", &format!("Bearer {token}"));
    }
    if let Some(from) = from.filter(|from| *from > 0) {
        builder = builder.header("Range", &format!("bytes={from}-"));
    }
    let response = builder.call().map_err(|e| format!("{url}: {e}"))?;
    let status = response.status().as_u16();
    if status == 200 || status == 206 {
        Ok(response)
    } else {
        Err(format!("{url}: HTTP {status}"))
    }
}

fn fetch(
    agent: &ureq::Agent,
    base: &str,
    token: Option<&str>,
    root: &Path,
    entry: &Entry,
    progress: &Progress,
    abort: &AtomicBool,
) -> Result<(), String> {
    let target = root.join(&entry.path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let part = PathBuf::from(format!("{}.part", target.display()));
    let mut last = String::new();
    for attempt in 0..ATTEMPTS {
        if abort.load(Ordering::Relaxed) {
            return Ok(());
        }
        if attempt > 0 {
            std::thread::sleep(RETRY_PAUSE);
        }
        let mut counted = 0u64;
        match attempt_once(agent, base, token, entry, &part, progress, abort, &mut counted) {
            Ok(()) => {
                fs::rename(&part, &target).map_err(|e| format!("{}: {e}", target.display()))?;
                return Ok(());
            }
            Err(message) => {
                progress.done_bytes.fetch_sub(counted, Ordering::Relaxed);
                last = message;
            }
        }
    }
    Err(last)
}

#[allow(clippy::too_many_arguments)]
fn attempt_once(
    agent: &ureq::Agent,
    base: &str,
    token: Option<&str>,
    entry: &Entry,
    part: &Path,
    progress: &Progress,
    abort: &AtomicBool,
    counted: &mut u64,
) -> Result<(), String> {
    let mut hasher = Sha256::new();
    let mut have = fs::metadata(part).map(|m| m.len()).unwrap_or(0);
    if have > entry.size {
        let _ = fs::remove_file(part);
        have = 0;
    }
    if have > 0 {
        let mut existing = File::open(part).map_err(|e| e.to_string())?;
        let mut buffer = vec![0u8; CHUNK];
        loop {
            let read = existing.read(&mut buffer).map_err(|e| e.to_string())?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
    }
    let mut response = if have == entry.size {
        None
    } else {
        Some(request(agent, &url(base, &entry.path), token, Some(have))?)
    };
    if let Some(response) = &response
        && have > 0
        && response.status().as_u16() == 200
    {
        let _ = fs::remove_file(part);
        have = 0;
        hasher = Sha256::new();
    }
    let mut out = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(have == 0)
        .append(have > 0)
        .open(part)
        .map_err(|e| format!("{}: {e}", part.display()))?;
    progress.done_bytes.fetch_add(have, Ordering::Relaxed);
    *counted += have;
    let mut written = have;
    if let Some(response) = response.as_mut() {
        let mut reader = response.body_mut().as_reader();
        let mut buffer = vec![0u8; CHUNK];
        loop {
            if abort.load(Ordering::Relaxed) {
                return Ok(());
            }
            let read = reader.read(&mut buffer).map_err(|e| format!("{}: {e}", entry.path))?;
            if read == 0 {
                break;
            }
            out.write_all(&buffer[..read]).map_err(|e| format!("{}: {e}", part.display()))?;
            hasher.update(&buffer[..read]);
            written += read as u64;
            progress.done_bytes.fetch_add(read as u64, Ordering::Relaxed);
            *counted += read as u64;
        }
    }
    out.flush().map_err(|e| e.to_string())?;
    drop(out);
    let digest: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
    if written != entry.size || !digest.eq_ignore_ascii_case(&entry.sha256) {
        let _ = fs::remove_file(part);
        return Err(format!("{}: conteúdo não confere com o manifesto", entry.path));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;

    fn digest(bytes: &[u8]) -> String {
        Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
    }

    fn serve(files: Vec<(String, Vec<u8>)>) -> (String, Arc<Mutex<Vec<Option<u64>>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let ranges = Arc::new(Mutex::new(Vec::new()));
        let seen = ranges.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request = String::new();
                reader.read_line(&mut request).unwrap();
                let path = request.split_whitespace().nth(1).unwrap_or("/").trim_start_matches('/').to_string();
                let mut from = None;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line.trim().is_empty() {
                        break;
                    }
                    if let Some(range) = line.to_ascii_lowercase().strip_prefix("range: bytes=") {
                        from = range.trim().trim_end_matches('-').parse::<u64>().ok();
                    }
                }
                seen.lock().unwrap().push(from);
                let body = files.iter().find(|(name, _)| *name == path).map(|(_, data)| data.clone());
                let reply = match body {
                    None => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(),
                    Some(data) => {
                        let start = from.unwrap_or(0) as usize;
                        let status = if from.is_some() { "206 Partial Content" } else { "200 OK" };
                        let mut reply = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", data.len() - start).into_bytes();
                        reply.extend_from_slice(&data[start..]);
                        reply
                    }
                };
                let _ = stream.write_all(&reply);
            }
        });
        (base, ranges)
    }

    fn manifest(files: &[(&str, &[u8])]) -> Vec<u8> {
        let entries = files.iter().map(|(path, data)| Entry { path: path.to_string(), size: data.len() as u64, sha256: digest(data) }).collect();
        serde_json::to_vec(&Manifest { files: entries }).unwrap()
    }

    fn wait(download: &Download) -> Snapshot {
        loop {
            let snapshot = download.snapshot();
            if matches!(snapshot.phase, Phase::Done | Phase::Failed(_)) {
                return snapshot;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("killer-download-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn paths_must_stay_inside_the_client_directory() {
        let entry = |path: &str| Entry { path: path.into(), size: 0, sha256: "0".repeat(64) };
        assert!(validate(&entry("system/a.ini")).is_ok());
        for bad in ["", "../a", "/etc/a", "a/../b", "a\\b"] {
            assert!(validate(&entry(bad)).is_err(), "{bad}");
        }
        assert!(validate(&Entry { path: "a".into(), size: 0, sha256: "zz".into() }).is_err());
    }

    #[test]
    fn urls_escape_each_segment_and_keep_the_slashes() {
        assert_eq!(url("https://h/p/", "system/a b+c.ini"), "https://h/p/system/a%20b%2Bc.ini");
    }

    #[test]
    fn it_downloads_everything_verifies_hashes_and_skips_what_is_present() {
        let (small, big) = (b"alpha".to_vec(), vec![7u8; 600_000]);
        let files = [("system/a.ini", small.as_slice()), ("maps/b.bin", big.as_slice())];
        let mut served = vec![(MANIFEST.to_string(), manifest(&files))];
        served.extend(files.iter().map(|(path, data)| (path.to_string(), data.to_vec())));
        let (base, ranges) = serve(served);
        let root = scratch("full");
        let snapshot = wait(&Download::start(base.clone(), None, root.clone()));
        assert_eq!(snapshot.phase, Phase::Done);
        assert_eq!((snapshot.done_files, snapshot.total_files), (2, 2));
        assert_eq!(snapshot.done_bytes, snapshot.total_bytes);
        assert_eq!(fs::read(root.join("maps/b.bin")).unwrap(), big);
        assert!(!root.join("maps/b.bin.part").exists());
        let before = ranges.lock().unwrap().len();
        assert_eq!(wait(&Download::start(base, None, root.clone())).phase, Phase::Done);
        assert_eq!(ranges.lock().unwrap().len(), before + 1);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn it_resumes_a_partial_file_with_a_range_request() {
        let data: Vec<u8> = (0..200_000u32).map(|n| (n % 251) as u8).collect();
        let files = [("maps/c.bin", data.as_slice())];
        let (base, ranges) = serve(vec![(MANIFEST.to_string(), manifest(&files)), ("maps/c.bin".to_string(), data.clone())]);
        let root = scratch("resume");
        fs::create_dir_all(root.join("maps")).unwrap();
        fs::write(root.join("maps/c.bin.part"), &data[..50_000]).unwrap();
        let snapshot = wait(&Download::start(base, None, root.clone()));
        assert_eq!(snapshot.phase, Phase::Done);
        assert_eq!(fs::read(root.join("maps/c.bin")).unwrap(), data);
        assert!(ranges.lock().unwrap().contains(&Some(50_000)));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_file_that_does_not_match_its_hash_fails_and_leaves_no_file() {
        let honest = b"original".to_vec();
        let files = [("system/d.ini", honest.as_slice())];
        let (base, _) = serve(vec![(MANIFEST.to_string(), manifest(&files)), ("system/d.ini".to_string(), b"tampered".to_vec())]);
        let root = scratch("tamper");
        let snapshot = wait(&Download::start(base, None, root.clone()));
        assert!(matches!(snapshot.phase, Phase::Failed(_)));
        assert!(!root.join("system/d.ini").exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn only_the_wanted_prefixes_are_downloaded_case_insensitively() {
        let files: [(&str, &[u8]); 3] = [("system/a.dat", b"aaa"), ("systextures/Icon.utx", b"icons"), ("maps/big.bin", b"huge")];
        let (base, _) = serve(vec![
            (MANIFEST.to_string(), manifest(&files)),
            ("system/a.dat".to_string(), b"aaa".to_vec()),
            ("systextures/Icon.utx".to_string(), b"icons".to_vec()),
            ("maps/big.bin".to_string(), b"huge".to_vec()),
        ]);
        let root = scratch("filtro");
        let download = Download::start_filtered(base, None, root.clone(), vec!["system/".into(), "SYSTEXTURES/icon".into()]);
        wait(&download);
        assert_eq!(download.snapshot().phase, Phase::Done);
        assert!(root.join("system/a.dat").exists() && root.join("systextures/Icon.utx").exists());
        assert!(!root.join("maps/big.bin").exists());
        assert!(wanted("anything", &[]));
        assert!(!wanted("maps/x", &["system/".to_string()]));
    }
}
