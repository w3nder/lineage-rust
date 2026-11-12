use crate::Error;
use ue2_core::Reader;
use ue2_package::Package;

enum Task {
    Expr,
    Raw(usize),
    Object,
    Name,
    Parameters,
    Debug,
    Bytes,
    Wide,
    Case,
    Labels,
}

pub(crate) fn skip(reader: &mut Reader<'_>, package: &Package, size: i32) -> Result<(), Error> {
    let limit = usize::try_from(size).map_err(|_| Error::BadScript)?;
    let mut logical = 0usize;
    let mut tasks = Vec::new();
    while logical < limit || !tasks.is_empty() {
        let task = tasks.pop().unwrap_or(Task::Expr);
        let mut consumed = 0;
        match task {
            Task::Raw(count) => {
                reader.bytes(count)?;
                consumed = count;
            }
            Task::Object => {
                package.object_at(reader.compact()?)?;
                consumed = 4;
            }
            Task::Name => {
                package.name_at(reader.compact()?)?;
                consumed = 4;
            }
            Task::Bytes => {
                consumed = 1;
                if reader.u8()? != 0 {
                    tasks.push(Task::Bytes);
                }
            }
            Task::Wide => {
                consumed = 2;
                if reader.u16()? >> 8 != 0 {
                    tasks.push(Task::Wide);
                }
            }
            Task::Case => {
                consumed = 2;
                if reader.u16()? != u16::MAX {
                    tasks.push(Task::Expr);
                }
            }
            Task::Labels => {
                if !logical.is_multiple_of(4) {
                    return Err(Error::BadScript);
                }
                let name = package.name_at(reader.compact()?)?;
                reader.u32()?;
                consumed = 8;
                if !name.eq_ignore_ascii_case("None") {
                    tasks.push(Task::Labels);
                }
            }
            Task::Parameters => {
                if reader.remaining().first() != Some(&0x16) {
                    tasks.push(Task::Parameters);
                }
                tasks.push(Task::Expr);
            }
            Task::Debug => {
                if logical < limit && reader.remaining().first() == Some(&0x42) {
                    let bytes = reader.remaining().get(1..5).ok_or(Error::BadScript)?;
                    if bytes == 100i32.to_le_bytes() {
                        tasks.push(Task::Expr);
                    }
                }
            }
            Task::Expr => {
                let token = reader.u8()?;
                consumed = 1;
                let operands = match token {
                    0x00..=0x02 | 0x20 | 0x29 => vec![Task::Object],
                    0x04 | 0x0d | 0x0e | 0x37 => vec![Task::Expr],
                    0x05 | 0x39 => vec![Task::Raw(1), Task::Expr],
                    0x06 => vec![Task::Raw(2)],
                    0x07 | 0x09 | 0x18 => vec![Task::Raw(2), Task::Expr],
                    0x08 | 0x0b | 0x16 | 0x17 | 0x25..=0x28 | 0x2a | 0x2d | 0x30 | 0x31 => vec![],
                    0x0a => vec![Task::Case],
                    0x0c => vec![Task::Labels],
                    0x0f | 0x10 | 0x14 | 0x1a | 0x45 => vec![Task::Expr, Task::Expr],
                    0x11 => vec![Task::Expr, Task::Expr, Task::Expr, Task::Expr],
                    0x12 | 0x19 => vec![Task::Expr, Task::Raw(2), Task::Raw(1), Task::Expr],
                    0x13 | 0x2e | 0x36 => vec![Task::Object, Task::Expr],
                    0x1b | 0x38 => vec![Task::Name, Task::Parameters, Task::Debug],
                    0x1c => vec![Task::Object, Task::Parameters, Task::Debug],
                    0x1d | 0x1e => vec![Task::Raw(4)],
                    0x1f => vec![Task::Bytes],
                    0x21 | 0x44 => vec![Task::Name],
                    0x22 | 0x23 => vec![Task::Raw(12)],
                    0x24 | 0x2c => vec![Task::Raw(1)],
                    0x2f => vec![Task::Expr, Task::Raw(2)],
                    0x32 | 0x33 => vec![Task::Object, Task::Expr, Task::Expr],
                    0x34 => vec![Task::Wide],
                    0x40 | 0x41 => vec![Task::Expr, Task::Expr, Task::Expr],
                    0x42 => vec![Task::Raw(12), Task::Bytes],
                    0x43 => vec![Task::Object, Task::Name],
                    0x60..=0x6f => vec![Task::Raw(1), Task::Parameters, Task::Debug],
                    0x70..=0xff => vec![Task::Parameters, Task::Debug],
                    _ => return Err(Error::UnknownScriptToken(token)),
                };
                tasks.extend(operands.into_iter().rev());
            }
        }
        logical = logical.checked_add(consumed).ok_or(Error::BadScript)?;
        if logical > limit {
            return Err(Error::BadScript);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package() -> Package {
        let mut bytes = Vec::new();
        bytes.extend(0x9e2a83c1u32.to_le_bytes());
        bytes.extend(123u16.to_le_bytes());
        bytes.extend(27u16.to_le_bytes());
        for word in [0u32, 2, 36, 0, 0, 0, 0] {
            bytes.extend(word.to_le_bytes());
        }
        for name in ["None", "Label"] {
            bytes.push((name.len() + 1) as u8);
            bytes.extend(name.as_bytes());
            bytes.push(0);
            bytes.extend(0u32.to_le_bytes());
        }
        Package::parse(&bytes).unwrap()
    }

    fn cases() -> Vec<(Vec<u8>, i32)> {
        let mut cases = Vec::new();
        for token in [0x00, 0x01, 0x02, 0x20, 0x29, 0x21, 0x44] {
            cases.push((vec![token, 0], 5));
        }
        for token in [0x04, 0x0d, 0x0e, 0x37] {
            cases.push((vec![token, 0x25], 2));
        }
        for token in [0x05, 0x39] {
            cases.push((vec![token, 0xab, 0x25], 3));
        }
        cases.push((vec![0x06, 0, 1], 3));
        for token in [0x07, 0x09, 0x18] {
            cases.push((vec![token, 0, 1, 0x25], 4));
        }
        for token in [0x08, 0x0b, 0x16, 0x17, 0x25, 0x26, 0x27, 0x28, 0x2a, 0x2d, 0x30, 0x31] {
            cases.push((vec![token], 1));
        }
        cases.extend([(vec![0x0a, 0xff, 0xff], 3), (vec![0x0a, 0, 0, 0x25], 4)]);
        cases.push((vec![0x0b, 0x0b, 0x0b, 0x0c, 1, 1, 2, 3, 4, 0, 0, 0, 0, 0], 20));
        for token in [0x0f, 0x10, 0x14, 0x1a, 0x45] {
            cases.push((vec![token, 0x25, 0x26], 3));
        }
        cases.push((vec![0x11, 0x25, 0x26, 0x27, 0x28], 5));
        for token in [0x12, 0x19] {
            cases.push((vec![token, 0x25, 0, 0, 0, 0x26], 6));
        }
        for token in [0x13, 0x2e, 0x36] {
            cases.push((vec![token, 0, 0x25], 6));
        }
        for token in [0x1b, 0x38, 0x1c] {
            cases.push((vec![token, 0, 0x25, 0x16], 7));
        }
        for token in [0x1d, 0x1e] {
            cases.push((vec![token, 1, 2, 3, 4], 5));
        }
        cases.push((vec![0x1f, b'a', b'b', 0], 4));
        for token in [0x22, 0x23] {
            let mut bytes = vec![token];
            bytes.extend([0; 12]);
            cases.push((bytes, 13));
        }
        for token in [0x24, 0x2c] {
            cases.push((vec![token, 0xff], 2));
        }
        cases.push((vec![0x2f, 0x25, 1, 2], 4));
        for token in [0x32, 0x33] {
            cases.push((vec![token, 0, 0x25, 0x26], 7));
        }
        cases.push((vec![0x34, 0, 1, 65, 0], 5));
        for token in [0x40, 0x41] {
            cases.push((vec![token, 0x25, 0x26, 0x27], 4));
        }
        let mut debug = vec![0x42];
        debug.extend(100i32.to_le_bytes());
        debug.extend([0; 8]);
        debug.extend([b'x', 0]);
        cases.push((debug.clone(), 15));
        cases.push((vec![0x43, 0, 1], 9));
        for token in 0x60..=0xff {
            let mut bytes = vec![token];
            if token < 0x70 {
                bytes.push(0xab);
            }
            bytes.extend([0x25, 0x16]);
            let size = bytes.len() as i32;
            cases.push((bytes.clone(), size));
            bytes.extend(&debug);
            cases.push((bytes, size + 15));
        }
        cases
    }

    #[test]
    fn serialize_expr_10132200_preserves_logical_size_and_defaults_boundary() {
        let package = package();
        for (mut bytes, size) in cases() {
            let length = bytes.len();
            bytes.extend([0xde, 0xad, 0xbe, 0xef]);
            let mut reader = Reader::at(&bytes, 0);
            skip(&mut reader, &package, size).unwrap_or_else(|error| panic!("{bytes:x?}: {error}"));
            assert_eq!(reader.pos(), length, "{bytes:x?}");
        }
    }

    #[test]
    fn serialize_expr_10132200_rejects_truncation_bad_tokens_and_bad_alignment() {
        let package = package();
        for (bytes, size) in cases() {
            for end in 0..bytes.len() {
                assert!(skip(&mut Reader::at(&bytes[..end], 0), &package, size).is_err(), "{bytes:x?}, cut {end}");
            }
        }
        for token in [0x03, 0x15, 0x2b, 0x35, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x46, 0x5f] {
            assert!(matches!(skip(&mut Reader::at(&[token], 0), &package, 1), Err(Error::UnknownScriptToken(_))));
        }
        assert!(matches!(skip(&mut Reader::at(&[], 0), &package, -1), Err(Error::BadScript)));
        assert!(matches!(skip(&mut Reader::at(&[0x0c, 0, 0, 0, 0, 0], 0), &package, 9), Err(Error::BadScript)));
    }

    #[test]
    fn serialize_expr_10132200_handles_deep_expressions_without_host_recursion() {
        let package = package();
        let mut bytes = vec![0x04; 100_000];
        bytes.push(0x25);
        skip(&mut Reader::at(&bytes, 0), &package, bytes.len() as i32).unwrap();
    }
}
