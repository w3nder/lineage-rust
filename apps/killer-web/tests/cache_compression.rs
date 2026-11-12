use std::io::{self, Read, Write};
use std::time::Instant;

struct Counter<W> { output: W, bytes: u64 }

impl<W: Write> Write for Counter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let count = self.output.write(bytes)?;
        self.bytes += count as u64;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> { self.output.flush() }
}

struct Compare<R> { input: R, scratch: Vec<u8>, bytes: u64 }

impl<R: Read> Write for Compare<R> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.scratch.resize(bytes.len(), 0);
        self.input.read_exact(&mut self.scratch)?;
        if self.scratch != bytes { return Err(io::Error::other("bytes da cena alterados")); }
        self.bytes += bytes.len() as u64;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

#[test]
#[ignore]
fn original_scene_compression_preserves_every_encoded_byte() {
    let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
    let tile = std::env::var("KILLER_CACHE_TILE").unwrap_or_else(|_| "lobby".into());
    let mut client = killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
    let loaded = client.load_level(&tile).unwrap();
    let scene = killer_assets::scene::build_scene(&mut client, &loaded.level);
    drop(loaded);
    drop(client);
    let stamp = killer_assets::cache::SourceStamp { size: 0, hash: 0 };
    let encode = |output: &mut dyn Write| killer_assets::cache::encode(output, &stamp, &scene, &[], &[]).unwrap();

    let started = Instant::now();
    let mut zlib = flate2::write::ZlibEncoder::new(Counter { output: io::sink(), bytes: 0 }, flate2::Compression::fast());
    {
        let mut buffered = io::BufWriter::with_capacity(1 << 20, &mut zlib);
        encode(&mut buffered);
        buffered.flush().unwrap();
    }
    let zlib_bytes = zlib.finish().unwrap().bytes;
    let zlib_ms = started.elapsed().as_secs_f64() * 1000.0;

    let path = std::env::temp_dir().join(format!("killer-web-codec-{}.lz4", std::process::id()));
    let started = Instant::now();
    let output = io::BufWriter::with_capacity(1 << 20, std::fs::File::create(&path).unwrap());
    let mut lz4 = lz4_flex::frame::FrameEncoder::new(Counter { output, bytes: 0 });
    {
        let mut buffered = io::BufWriter::with_capacity(1 << 20, &mut lz4);
        encode(&mut buffered);
        buffered.flush().unwrap();
    }
    let mut result = lz4.finish().unwrap();
    result.flush().unwrap();
    let lz4_bytes = result.bytes;
    drop(result);
    let lz4_ms = started.elapsed().as_secs_f64() * 1000.0;

    let input = io::BufReader::with_capacity(1 << 20, std::fs::File::open(&path).unwrap());
    let mut compared = Compare { input: lz4_flex::frame::FrameDecoder::new(input), scratch: Vec::new(), bytes: 0 };
    killer_assets::cache::encode_consuming(&mut compared, &stamp, scene, |_| {}).unwrap();
    assert_eq!(compared.input.read(&mut [0]).unwrap(), 0);
    std::fs::remove_file(path).unwrap();
    eprintln!("{tile}: zlib {zlib_ms:.1} ms / {zlib_bytes} bytes; LZ4 {lz4_ms:.1} ms / {lz4_bytes} bytes; {} bytes conferidos", compared.bytes);
}
