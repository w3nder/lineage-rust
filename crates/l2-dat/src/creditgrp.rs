use crate::{DatReader, Error};

const FOOTER: &str = "SafePackage";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credit {
    pub id: u32,
    pub html: String,
    pub image: String,
    pub time: u32,
    pub align: u32,
}

pub fn read(body: &[u8]) -> Result<Vec<Credit>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    let credits = (0..count).map(|_| record(&mut reader)).collect::<Result<Vec<_>, _>>()?;
    let at = body.len() - reader.remaining();
    if reader.ascf()? != FOOTER || reader.remaining() != 0 {
        return Err(Error::InvalidString(at));
    }
    Ok(credits)
}

fn record(r: &mut DatReader<'_>) -> Result<Credit, Error> {
    let id = r.u32()?;
    let html = r.ascf()?;
    let image = r.ascf()?;
    let time = r.u32()?;
    let align = r.u32()?;
    Ok(Credit { id, html, image, time, align })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fstring(s: &str) -> Vec<u8> {
        let mut out = vec![s.len() as u8 + 1];
        out.extend(s.bytes());
        out.push(0);
        out
    }

    #[test]
    fn reads_records_and_the_safe_package_footer() {
        let mut body = 1u32.to_le_bytes().to_vec();
        body.extend(0u32.to_le_bytes());
        body.extend(fstring("credit_ceo.htm"));
        body.extend(fstring("L2UI_CH3.credit_ch3_01"));
        body.extend(10u32.to_le_bytes());
        body.extend(5u32.to_le_bytes());
        let mut file = body.clone();
        file.extend(fstring(FOOTER));
        let credits = read(&file).unwrap();
        assert_eq!(
            credits,
            [Credit {
                id: 0,
                html: "credit_ceo.htm".into(),
                image: "L2UI_CH3.credit_ch3_01".into(),
                time: 10,
                align: 5
            }]
        );
        assert!(read(&body).is_err());
        assert!(read(&file[..file.len() - 1]).is_err());
        file.push(0);
        assert!(read(&file).is_err());
    }
}
