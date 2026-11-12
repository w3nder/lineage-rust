use crate::{DatReader, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MusicInfo {
    pub id: u32,
    pub titles: Vec<String>,
}

pub fn read(body: &[u8]) -> Result<Vec<MusicInfo>, Error> {
    read_prefix(body).map(|(rows, _)| rows)
}

pub fn read_prefix(body: &[u8]) -> Result<(Vec<MusicInfo>, usize), Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    let rows = (0..count).map(|_| record(&mut reader)).collect::<Result<Vec<_>, _>>()?;
    Ok((rows, body.len() - reader.remaining()))
}

fn record(r: &mut DatReader<'_>) -> Result<MusicInfo, Error> {
    let id = r.u32()?;
    let titles = r.unicode_array()?;
    Ok(MusicInfo { id, titles })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unicode(s: &str) -> Vec<u8> {
        let mut out = (s.len() as u32 * 2).to_le_bytes().to_vec();
        out.extend(s.encode_utf16().flat_map(u16::to_le_bytes));
        out
    }

    #[test]
    fn record_is_the_id_then_a_counted_list_of_titles() {
        let mut body = 1u32.to_le_bytes().to_vec();
        body.extend(0u32.to_le_bytes());
        body.extend(3u32.to_le_bytes());
        for title in ["INTRO", "OUTRO", "S20_F"] {
            body.extend(unicode(title));
        }
        let (rows, used) = read_prefix(&body).unwrap();
        assert_eq!(used, body.len());
        assert_eq!(rows, [MusicInfo { id: 0, titles: vec!["INTRO".into(), "OUTRO".into(), "S20_F".into()] }]);
        assert!(read(&body[..body.len() - 1]).is_err());
    }
}
