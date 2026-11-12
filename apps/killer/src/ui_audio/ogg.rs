use std::io::{self, Read, Seek, SeekFrom};

pub struct ClientOggReader<R> {
    inner: R,
    pages: Vec<u64>,
    position: u64,
}

pub(super) fn checksum(bytes: &[u8]) -> u32 {
    let mut crc = 0_u32;
    for (index, byte) in bytes.iter().copied().enumerate() {
        crc ^= u32::from(if (22..26).contains(&index) { 0 } else { byte }) << 24;
        for _ in 0..8 {
            crc = (crc << 1) ^ if crc & 0x8000_0000 != 0 { 0x04c1_1db7 } else { 0 };
        }
    }
    crc
}

fn next_capture<R: Read + Seek>(inner: &mut R, start: u64, length: u64) -> io::Result<u64> {
    inner.seek(SeekFrom::Start(start))?;
    let mut position = start;
    let mut buffer = [0; 4096];
    while position < length {
        let count = ((length - position).min(buffer.len() as u64)) as usize;
        inner.read_exact(&mut buffer[..count])?;
        if let Some(offset) = buffer[..count].iter().position(|byte| *byte == b'O') {
            return Ok(position + offset as u64);
        }
        position += count as u64;
    }
    Ok(length)
}

pub(super) fn normalization_offsets<R: Read + Seek>(inner: &mut R) -> io::Result<Vec<u64>> {
    let length = inner.seek(SeekFrom::End(0))?;
    inner.seek(SeekFrom::Start(0))?;
    let mut magic = [0; 4];
    let mut pages = Vec::new();
    if length < 4 { return Ok(pages); }
    inner.read_exact(&mut magic)?;
    if &magic != b"L2SD" && &magic != b"OggS" { return Ok(pages); }
    let mut position = 0_u64;
    let mut page = Vec::new();
    while length - position >= 27 {
        inner.seek(SeekFrom::Start(position))?;
        page.resize(27, 0);
        inner.read_exact(&mut page)?;
        if &page[..4] != b"L2SD" && &page[..4] != b"OggS" {
            position = next_capture(inner, position + 1, length)?;
            continue;
        }
        pages.push(position);
        page[..4].copy_from_slice(b"OggS");
        let header_length = 27 + usize::from(page[26]);
        if header_length as u64 > length - position { break; }
        page.resize(header_length, 0);
        inner.read_exact(&mut page[27..])?;
        let page_length = header_length + page[27..].iter().map(|byte| usize::from(*byte)).sum::<usize>();
        if page_length as u64 > length - position { break; }
        page.resize(page_length, 0);
        inner.read_exact(&mut page[header_length..])?;
        let expected = u32::from_le_bytes([page[22], page[23], page[24], page[25]]);
        position = if checksum(&page) == expected { position + page_length as u64 }
            else { next_capture(inner, position + 1, length)? };
    }
    Ok(pages)
}

impl<R: Read + Seek> ClientOggReader<R> {
    pub fn new(mut inner: R) -> io::Result<Self> {
        let pages = normalization_offsets(&mut inner)?;
        inner.seek(SeekFrom::Start(0))?;
        Ok(Self { inner, pages, position: 0 })
    }
}

impl<R: Read> Read for ClientOggReader<R> {
    fn read(&mut self, target: &mut [u8]) -> io::Result<usize> {
        let count = self.inner.read(target)?;
        let start = self.position;
        let end = start + count as u64;
        let index = self.pages.partition_point(|page| *page + 4 <= start);
        for page in &self.pages[index..] {
            if *page >= end {
                break;
            }
            for position in (*page).max(start)..(*page + 4).min(end) {
                target[(position - start) as usize] = b"OggS"[(position - *page) as usize];
            }
        }
        self.position = end;
        Ok(count)
    }
}

impl<R: Seek> Seek for ClientOggReader<R> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.position = self.inner.seek(position)?;
        Ok(self.position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn page(payload: &[u8]) -> Vec<u8> {
        assert!(payload.len() < 256);
        let mut page = vec![0; 28];
        page[..4].copy_from_slice(b"OggS");
        page[26] = 1;
        page[27] = payload.len() as u8;
        page.extend_from_slice(payload);
        let crc = checksum(&page);
        page[22..26].copy_from_slice(&crc.to_le_bytes());
        page[..4].copy_from_slice(b"L2SD");
        page
    }

    #[test]
    fn normalization_survives_every_read_size_and_seek_into_magic() {
        let mut input = page(&[]);
        input.extend(page(&[]));
        let mut expected = input.clone();
        expected[..4].copy_from_slice(b"OggS");
        expected[28..32].copy_from_slice(b"OggS");
        for size in 1..=input.len() {
            let mut reader = ClientOggReader::new(Cursor::new(input.clone())).unwrap();
            let mut actual = Vec::new();
            let mut buffer = vec![0; size];
            loop {
                let count = reader.read(&mut buffer).unwrap();
                if count == 0 {
                    break;
                }
                actual.extend_from_slice(&buffer[..count]);
            }
            assert_eq!(actual, expected);
            for offset in 0..input.len() {
                reader.seek(SeekFrom::Start(offset as u64)).unwrap();
                actual.clear();
                reader.read_to_end(&mut actual).unwrap();
                assert_eq!(actual, expected[offset..]);
            }
        }
    }

    #[test]
    fn incomplete_input_is_not_a_fatal_sync_error_and_wave_bytes_are_preserved() {
        assert!(ClientOggReader::new(Cursor::new(b"L2SD".to_vec())).is_ok());
        let bytes = b"RIFF1234WAVEdata";
        let mut reader = ClientOggReader::new(Cursor::new(bytes.to_vec())).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, bytes);
    }

    #[test]
    fn pageseek_10001c60_preserves_every_short_tail_and_incomplete_page() {
        for tail in 0..27 {
            let mut input = page(b"OggS inside payload");
            input.extend(vec![0x55; tail]);
            let mut expected = input.clone();
            expected[..4].copy_from_slice(b"OggS");
            let mut actual = Vec::new();
            ClientOggReader::new(Cursor::new(input)).unwrap().read_to_end(&mut actual).unwrap();
            assert_eq!(actual, expected);
        }
        let input = page(b"L2SD payload");
        for length in 0..input.len() {
            let mut expected = input[..length].to_vec();
            if length >= 27 { expected[..4].copy_from_slice(b"OggS"); }
            let mut actual = Vec::new();
            ClientOggReader::new(Cursor::new(input[..length].to_vec())).unwrap().read_to_end(&mut actual).unwrap();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn pageseek_10001c60_resynchronizes_after_crc_failure_at_o_not_l() {
        let mut input = page(&[0]);
        input[28] ^= 1;
        input.extend(page(&[]));
        let offset = input.len();
        let mut good = page(&[]);
        good[..4].copy_from_slice(b"OggS");
        input.extend(good);
        let last = input.len();
        input.extend(page(&[]));
        assert_eq!(normalization_offsets(&mut Cursor::new(input)).unwrap(), [0, offset as u64, last as u64]);
    }

    #[test]
    #[ignore]
    fn pageseek_10001c60_matches_original_crc_recovery_and_every_truncation() {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut cases = Vec::new();
        let input = [page(b"OggS L2SD payload"), page(b"second")].concat();
        for length in 0..=input.len() { cases.push(input[..length].to_vec()); }
        for tail in 0..64 {
            let mut bytes = input.clone();
            bytes.extend(vec![0x55; tail]);
            cases.push(bytes);
        }
        for offset in 0..input.len() {
            let mut bytes = input.clone();
            bytes[offset] ^= 0x80;
            cases.push(bytes);
        }
        let mut damaged = page(b"bad");
        damaged[28] ^= 1;
        damaged.extend(page(b"skip L"));
        let mut good = page(b"recover O");
        good[..4].copy_from_slice(b"OggS");
        damaged.extend(good);
        damaged.extend(page(b"then L"));
        cases.push(damaged);
        let hex = |bytes: &[u8]| bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let python = std::env::var_os("KILLER_ORACLE_PYTHON").unwrap_or_else(|| "python3".into());
        let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/ogg_sync_reference.py");
        let mut child = Command::new(python).arg(script).arg(root)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        let mut stdin = child.stdin.take().unwrap();
        let encoded_cases: Vec<_> = cases.iter().map(|bytes| hex(bytes)).collect();
        let writer = std::thread::spawn(move || {
            for encoded in encoded_cases { writeln!(stdin, "{encoded}").unwrap(); }
        });
        let output = child.wait_with_output().unwrap();
        writer.join().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let output = String::from_utf8(output.stdout).unwrap();
        let lines: Vec<_> = output.lines().collect();
        assert_eq!(lines.len(), cases.len());
        for (case, (bytes, expected)) in cases.into_iter().zip(lines).enumerate() {
            let mut actual = Vec::new();
            ClientOggReader::new(Cursor::new(bytes)).unwrap().read_to_end(&mut actual).unwrap();
            assert_eq!(hex(&actual), expected, "caso{case}");
        }
    }
}
