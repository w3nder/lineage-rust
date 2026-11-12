use crate::Error;
use crate::reader::DatReader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassInfo {
    pub id: u32,
    pub text: String,
}

pub fn read(body: &[u8]) -> Result<Vec<ClassInfo>, Error> {
    let mut r = DatReader::new(body);
    let count = r.u32()? as usize;
    let mut rows = Vec::with_capacity(count.min(256));
    for _ in 0..count {
        let id = r.u32()?;
        let text = r.ascf()?.trim_end_matches('\0').replace("\r\n", "\n");
        rows.push(ClassInfo { id, text });
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_count_then_id_and_text_per_row() {
        let mut body = 2u32.to_le_bytes().to_vec();
        body.extend_from_slice(&0u32.to_le_bytes());
        body.extend_from_slice(b"\x07Select\0");
        body.extend_from_slice(&1u32.to_le_bytes());
        body.extend_from_slice(b"\x08A\r\nB\r\nC\0");
        let rows = read(&body).unwrap();
        assert_eq!(rows[0], ClassInfo { id: 0, text: "Select".into() });
        assert_eq!(rows[1].text, "A\nB\nC");
    }
}
