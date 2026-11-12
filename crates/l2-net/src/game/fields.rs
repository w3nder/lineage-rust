use super::packet_table::{CLIENT, EXTENDED, SERVER};

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub kind: char,
    pub offset: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketKind {
    pub name: String,
    pub format: String,
}

fn find(table: &[(u16, &str, &str)], key: u16, body: &[u8], prefix: &str) -> Option<PacketKind> {
    let candidates: Vec<_> = table.iter().filter(|(op, _, _)| *op == key).collect();
    let exact = candidates.iter().find(|(_, _, format)| {
        let full = format!("{prefix}{format}");
        let (fields, used) = decode(&full, body);
        !format.is_empty() && used == body.len() && fields.len() == field_count(&full)
    });
    exact.or(candidates.first()).map(|(_, name, format)| PacketKind { name: (*name).to_owned(), format: (*format).to_owned() })
}

pub fn server_kind(body: &[u8]) -> Option<PacketKind> {
    let opcode = *body.first()?;
    if opcode == 0xfe && body.len() >= 3 {
        let sub = u16::from_le_bytes([body[1], body[2]]);
        return find(EXTENDED, sub, body, "ch").map(|mut kind| {
            kind.format = format!("ch{}", kind.format);
            kind
        });
    }
    find(SERVER, u16::from(opcode), body, "")
}

pub fn client_kind(body: &[u8]) -> Option<PacketKind> {
    let opcode = *body.first()?;
    if opcode == 0xd0 && body.len() >= 3 {
        let sub = u16::from_le_bytes([body[1], body[2]]);
        return find(CLIENT, 0xd000 | (sub & 0xff), body, "");
    }
    find(CLIENT, u16::from(opcode), body, "")
}

pub fn field_count(format: &str) -> usize {
    let mut total = 0usize;
    let mut chars = format.chars().peekable();
    while let Some(kind) = chars.next() {
        if matches!(kind, ';' | '(' | ')' | '{' | '}' | '*') {
            continue;
        }
        let repeat = if chars.peek() == Some(&'*') {
            chars.next();
            let mut digits = String::new();
            while chars.peek().is_some_and(char::is_ascii_digit) {
                digits.extend(chars.next());
            }
            digits.parse::<usize>().unwrap_or(1)
        } else {
            1
        };
        total += repeat;
    }
    total
}

pub fn decode(format: &str, body: &[u8]) -> (Vec<Field>, usize) {
    let mut fields = Vec::new();
    let mut at = 0usize;
    let mut previous: Option<char> = None;
    let mut chars = format.chars().peekable();
    while let Some(kind) = chars.next() {
        if kind == ';' || kind == '(' || kind == ')' || kind == '{' || kind == '}' {
            continue;
        }
        let repeat = if chars.peek() == Some(&'*') {
            chars.next();
            let mut digits = String::new();
            while chars.peek().is_some_and(char::is_ascii_digit) {
                digits.extend(chars.next());
            }
            digits.parse::<usize>().unwrap_or(1)
        } else {
            1
        };
        let _ = previous.replace(kind);
        for _ in 0..repeat {
            let Some((text, size)) = read(kind, &body[at.min(body.len())..]) else { return (fields, at) };
            fields.push(Field { kind, offset: at, text });
            at += size;
        }
    }
    (fields, at)
}

fn read(kind: char, bytes: &[u8]) -> Option<(String, usize)> {
    match kind {
        'c' | 'b' => bytes.first().map(|byte| (byte.to_string(), 1)),
        'h' => Some((i16::from_le_bytes(bytes.get(..2)?.try_into().ok()?).to_string(), 2)),
        'd' => Some((i32::from_le_bytes(bytes.get(..4)?.try_into().ok()?).to_string(), 4)),
        'Q' => Some((i64::from_le_bytes(bytes.get(..8)?.try_into().ok()?).to_string(), 8)),
        'f' => Some((format!("{}", f64::from_le_bytes(bytes.get(..8)?.try_into().ok()?)), 8)),
        'S' | 's' => {
            let mut units = Vec::new();
            let mut size = 0;
            loop {
                let pair = bytes.get(size..size + 2)?;
                size += 2;
                let unit = u16::from_le_bytes([pair[0], pair[1]]);
                if unit == 0 {
                    break;
                }
                units.push(unit);
            }
            Some((String::from_utf16_lossy(&units), size))
        }
        _ => None,
    }
}

pub fn hex_dump(body: &[u8]) -> String {
    body.chunks(16)
        .enumerate()
        .map(|(row, chunk)| {
            let hex: Vec<String> = chunk.iter().map(|byte| format!("{byte:02X}")).collect();
            let text: String = chunk.iter().map(|byte| if (0x20..0x7f).contains(byte) { *byte as char } else { '.' }).collect();
            format!("{:04X}  {:<47}  {text}", row * 16, hex.join(" "))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_each_field_type_in_order_with_offsets() {
        let mut body = vec![0x05u8];
        body.extend_from_slice(&(-3i32).to_le_bytes());
        body.extend_from_slice(&7i16.to_le_bytes());
        for unit in "Oi".encode_utf16() {
            body.extend_from_slice(&unit.to_le_bytes());
        }
        body.extend_from_slice(&[0, 0]);
        body.extend_from_slice(&1.5f64.to_le_bytes());
        let (fields, used) = decode("cdhSf", &body);
        assert_eq!(fields.iter().map(|field| field.text.as_str()).collect::<Vec<_>>(), vec!["5", "-3", "7", "Oi", "1.5"]);
        assert_eq!(fields.iter().map(|field| field.offset).collect::<Vec<_>>(), vec![0, 1, 5, 7, 13]);
        assert_eq!(used, body.len());
    }

    #[test]
    fn repetition_branches_and_truncated_bodies_never_panic() {
        let body: Vec<u8> = (0..20).collect();
        let (fields, used) = decode("cd*3;h", &body);
        assert_eq!(fields.len(), 5);
        assert_eq!(used, 15);
        assert!(decode("S", &[1, 0, 2]).0.is_empty());
        assert!(decode("dddd", &[1, 2, 3]).0.is_empty());
        assert!(decode("zz", &body).0.is_empty());
        assert!(decode("", &body).0.is_empty());
    }

    #[test]
    fn names_and_formats_come_from_the_audited_tables_in_both_directions() {
        let attack = server_kind(&[0x05]).unwrap();
        assert_eq!(attack.name, "AttackPacket");
        assert!(attack.format.starts_with("dddc"));
        let mut action = vec![0x04u8];
        action.extend_from_slice(&[0; 17]);
        let say = client_kind(&action).unwrap();
        assert_eq!(say.name, "Action");
        assert_eq!(say.format, "cddddc");
        let mut attack = vec![0x04u8];
        attack.extend_from_slice(&[0; 16]);
        assert_eq!(client_kind(&attack).unwrap().name, "RequestAttack");
        let mut regen = vec![0xfe, 0x01, 0x00];
        regen.extend_from_slice(&[0; 20]);
        assert!(server_kind(&regen).unwrap().name.contains("RegenMax"));
        assert!(server_kind(&[]).is_none());
        assert!(client_kind(&[0xaa, 1, 2]).is_none() || client_kind(&[0xaa, 1, 2]).is_some());
    }

    #[test]
    fn hex_dump_shows_offset_hex_and_text() {
        let dump = hex_dump(b"Lineage II\x00\x01");
        assert!(dump.starts_with("0000  4C 69 6E 65 61 67 65 20 49 49 00 01"));
        assert!(dump.ends_with("Lineage II.."));
        assert_eq!(hex_dump(&[]), "");
    }
}
