use crate::{DatReader, Error};

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneName {
    pub id: u32,
    pub color_id: u32,
    pub grid_x: u32,
    pub grid_y: u32,
    pub top_z: f32,
    pub bottom_z: f32,
    pub name: String,
}

impl ZoneName {
    pub fn is_territory(&self) -> bool {
        self.grid_x == 0 && self.grid_y == 0
    }

    pub fn covers(&self, grid: (u32, u32), z: f32) -> bool {
        (self.grid_x, self.grid_y) == grid && z >= self.bottom_z && z <= self.top_z
    }
}

pub fn read(body: &[u8]) -> Result<Vec<ZoneName>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| record(&mut reader)).collect()
}

const FOOTER: &str = "SafePackage";

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMapInfo {
    pub values: [i32; 6],
    pub scale: f32,
    pub texture: String,
}

impl ZoneMapInfo {
    pub fn is_empty(&self) -> bool {
        self.values == [-1; 6] && self.scale == 0.0 && self.texture.is_empty()
    }
}

pub fn read_as(body: &[u8], chronicle: crate::Chronicle) -> Result<Vec<ZoneName>, Error> {
    match chronicle {
        crate::Chronicle::C4 => read(body),
        crate::Chronicle::Interlude => Ok(read_interlude(body)?.into_iter().map(|(zone, _)| zone).collect()),
    }
}

pub fn read_interlude(body: &[u8]) -> Result<Vec<(ZoneName, ZoneMapInfo)>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    let zones = (0..count).map(|_| interlude_record(&mut reader)).collect::<Result<Vec<_>, _>>()?;
    let at = body.len() - reader.remaining();
    if reader.ascf()? != FOOTER || reader.remaining() != 0 {
        return Err(Error::InvalidString(at));
    }
    Ok(zones)
}

fn interlude_record(r: &mut DatReader<'_>) -> Result<(ZoneName, ZoneMapInfo), Error> {
    let zone = record(r)?;
    let info = ZoneMapInfo {
        values: [r.i32()?, r.i32()?, r.i32()?, r.i32()?, r.i32()?, r.i32()?],
        scale: r.f32()?,
        texture: r.ascf()?,
    };
    Ok((zone, info))
}

fn record(r: &mut DatReader<'_>) -> Result<ZoneName, Error> {
    Ok(ZoneName {
        id: r.u32()?,
        color_id: r.u32()?,
        grid_x: r.u32()?,
        grid_y: r.u32()?,
        top_z: r.f32()?,
        bottom_z: r.f32()?,
        name: r.ascf()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zonename_first_records() {
        let zones = read(include_bytes!("../tests/fixtures/zonename-head.bin")).unwrap();
        assert_eq!(zones.len(), 3);
        assert_eq!((zones[0].id, zones[0].color_id, zones[0].grid_x, zones[0].grid_y), (0, 0, 0, 0));
        assert_eq!((zones[0].top_z, zones[0].bottom_z), (30000.0, -30000.0));
        assert_eq!(zones[0].name, "Neutral Waterways");
        assert!(zones[0].is_territory());
        assert_eq!(zones[1].name, "Talking Island");
        assert_eq!(zones[2].name, "Gludio Territory");
        assert!(zones[2].covers((0, 0), 100.0));
        assert!(!zones[2].covers((22, 22), 100.0));
        assert!(!zones[2].covers((0, 0), 40000.0));
    }

    fn ascii(s: &str) -> Vec<u8> {
        if s.is_empty() {
            return vec![0];
        }
        let mut out = vec![s.len() as u8 + 1];
        out.extend(s.bytes());
        out.push(0);
        out
    }

    fn interlude_body() -> Vec<u8> {
        let mut body = 2u32.to_le_bytes().to_vec();
        for (id, name, values, scale, texture) in [
            (0u32, "Neutral", [-1i32; 6], 0.0f32, ""),
            (20, "Town", [592, 1932, -19968, 117504, 1024, 1024], 0.05, "map_town"),
        ] {
            for word in [id, 255, 19, 21] {
                body.extend(word.to_le_bytes());
            }
            body.extend(30000.0f32.to_le_bytes());
            body.extend((-30000.0f32).to_le_bytes());
            body.extend(ascii(name));
            for value in values {
                body.extend(value.to_le_bytes());
            }
            body.extend(scale.to_le_bytes());
            body.extend(ascii(texture));
        }
        body.extend(ascii(FOOTER));
        body
    }

    #[test]
    fn interlude_records_carry_six_ints_a_scale_and_a_map_texture() {
        let body = interlude_body();
        let rows = read_interlude(&body).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0.name, "Neutral");
        assert!(rows[0].1.is_empty());
        assert_eq!(rows[1].0.id, 20);
        assert_eq!((rows[1].0.grid_x, rows[1].0.grid_y), (19, 21));
        assert_eq!(rows[1].1.values, [592, 1932, -19968, 117504, 1024, 1024]);
        assert_eq!(rows[1].1.scale, 0.05);
        assert_eq!(rows[1].1.texture, "map_town");
        let zones = read_as(&body, crate::Chronicle::Interlude).unwrap();
        assert_eq!(zones[1].name, "Town");
        assert!(read_interlude(&body[..body.len() - 1]).is_err());
        let mut longer = body.clone();
        longer.push(0);
        assert!(read_interlude(&longer).is_err());
    }

    #[test]
    fn truncated_body_is_an_error() {
        let body = include_bytes!("../tests/fixtures/zonename-head.bin");
        assert!(read(&body[..body.len() - 1]).is_err());
    }
}
