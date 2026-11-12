use crate::{DatReader, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemMessage {
    pub id: u32,
    pub present: u32,
    pub text: String,
    pub group: u32,
    pub color: [u8; 3],
    pub alpha: u8,
    pub sound: String,
    pub second_sound: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemMessageExtra {
    pub numbers: [i32; 5],
    pub screen_text: String,
    pub channel: String,
}

impl SystemMessage {
    pub fn argb(&self) -> u32 {
        u32::from_le_bytes([self.color[0], self.color[1], self.color[2], self.alpha])
    }
}

pub fn read(body: &[u8]) -> Result<Vec<SystemMessage>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| record(&mut reader)).collect()
}

pub fn read_as(body: &[u8], chronicle: crate::Chronicle) -> Result<Vec<SystemMessage>, Error> {
    match chronicle {
        crate::Chronicle::C4 => read(body),
        crate::Chronicle::Interlude => read_interlude(body).map(|rows| rows.into_iter().map(|(message, _)| message).collect()),
    }
}

pub fn read_interlude(body: &[u8]) -> Result<Vec<(SystemMessage, SystemMessageExtra)>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count)
        .map(|_| {
            let message = record(&mut reader)?;
            let extra = SystemMessageExtra {
                numbers: [reader.i32()?, reader.i32()?, reader.i32()?, reader.i32()?, reader.i32()?],
                screen_text: reader.ascf()?,
                channel: reader.ascf()?,
            };
            Ok((message, extra))
        })
        .collect()
}

fn record(r: &mut DatReader<'_>) -> Result<SystemMessage, Error> {
    let id = r.u32()?;
    let present = r.u32()?;
    let text = r.ascf()?;
    let group = r.u32()?;
    let color = r.rgb()?;
    let alpha = r.u8()?;
    let sound = r.ascf()?;
    let second_sound = r.ascf()?;
    Ok(SystemMessage { id, present, text, group, color, alpha, sound, second_sound })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systemmsg_first_records() {
        let messages = read(include_bytes!("../tests/fixtures/systemmsg-head.bin")).unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].id, 0);
        assert_eq!(messages[0].text, "You have been disconnected from the server.");
        assert_eq!(messages[0].group, 2);
        assert_eq!(messages[0].color, [0, 0, 0xee]);
        assert_eq!(messages[0].argb(), 0xFFEE0000);
        assert_eq!(messages[0].sound, "ItemSound3.sys_broadcast");
        assert_eq!(messages[1].id, 1);
        assert!(messages[1].text.starts_with("The server will be disconnected in $s1 seconds"));
        assert_eq!(messages[2].id, 13);
        assert!(messages[2].text.starts_with("Unable to disperse"));
        assert_eq!(messages[2].text.len(), 77);
        assert_eq!(messages[2].group, 0);
        assert_eq!(messages[2].color, [0x79, 0x9b, 0xb0]);
        assert_eq!(messages[2].argb(), 0xFFB09B79);
        assert_eq!(messages[2].sound, "ItemSound3.sys_impossible");
        assert_eq!(messages[2].second_sound, "");
    }

    #[test]
    fn truncated_body_is_an_error() {
        let body = include_bytes!("../tests/fixtures/systemmsg-head.bin");
        assert!(read(&body[..body.len() - 1]).is_err());
    }

    fn ascf(s: &str) -> Vec<u8> {
        if s.is_empty() {
            return vec![0];
        }
        let mut out = vec![(s.len() + 1) as u8];
        out.extend(s.bytes());
        out.push(0);
        out
    }

    fn interlude_row(id: u32, text: &str, numbers: [i32; 5], screen_text: &str, channel: &str) -> Vec<u8> {
        let mut body = id.to_le_bytes().to_vec();
        body.extend(1u32.to_le_bytes());
        body.extend(ascf(text));
        body.extend(2u32.to_le_bytes());
        body.extend([0x79, 0x9b, 0xb0, 0xff]);
        body.extend(ascf(""));
        body.extend(ascf(""));
        for v in numbers {
            body.extend(v.to_le_bytes());
        }
        body.extend(ascf(screen_text));
        body.extend(ascf(channel));
        body
    }

    #[test]
    fn interlude_rows_carry_five_ints_a_screen_text_and_a_channel() {
        let mut body = 2u32.to_le_bytes().to_vec();
        body.extend(interlude_row(0, "Disconnected.", [0; 5], "", "server"));
        body.extend(interlude_row(1, "Coming down.", [8, 0, 3, 1, 0], "Exit Game: $s1 second(s)", "server"));
        let rows = read_interlude(&body).unwrap();
        assert_eq!(rows.len(), 2);
        let first = &rows[0].1;
        assert_eq!((first.numbers, first.screen_text.as_str(), first.channel.as_str()), ([0; 5], "", "server"));
        let second = &rows[1].1;
        assert_eq!((second.numbers, second.screen_text.as_str()), ([8, 0, 3, 1, 0], "Exit Game: $s1 second(s)"));
        assert_eq!((rows[1].0.id, rows[1].0.argb()), (1, 0xFFB09B79));
        assert!(read_interlude(&body[..body.len() - 1]).is_err());
        assert_eq!(read_as(&body, crate::Chronicle::Interlude).unwrap().len(), 2);
    }
}
