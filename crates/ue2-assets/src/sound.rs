use crate::{Error, object_data};
use ue2_core::Reader;
use ue2_package::{Export, Package};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sound<'a> {
    pub format: &'a str,
    pub encoded: &'a [u8],
}

pub fn read_sound<'a>(package: &'a Package, file: impl Into<ue2_core::ByteWindow<'a>>, export: &Export) -> Result<Sound<'a>, Error> {
    let (_, body) = object_data(package, file, export)?;
    let start = export.serial_offset + export.serial_size - body.len();
    let mut reader = Reader::at(body, 0);
    let format = package.name_at(reader.compact()?)?;
    let end = if package.version > 61 { Some(reader.u32()? as usize) } else { None };
    let count = reader.compact()?;
    let count = usize::try_from(count).map_err(|_| Error::BadSound)?;
    let encoded = reader.bytes(count)?;
    if end.is_some_and(|end| end != start + reader.pos()) {
        return Err(Error::BadSound);
    }
    if !reader.remaining().is_empty() {
        return Err(Error::TrailingBytes(reader.remaining().len()));
    }
    Ok(Sound { format, encoded })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn usound_10503b50_reads_every_sound_in_both_interface_packages() {
        let root = std::path::PathBuf::from(std::env::var("KILLER_CLIENT_DIR").unwrap());
        let mut total = 0;
        for name in ["ItemSound", "InterfaceSound"] {
            let path = root.join("sounds").join(format!("{name}.uax"));
            let bytes = l2_crypto::decrypt_owned(std::fs::read(&path).unwrap(), &path).unwrap();
            let package = Package::parse(&bytes).unwrap();
            let mut count = 0;
            for export in package.exports().iter().filter(|e| package.class_name(e) == "Sound") {
                let sound = read_sound(&package, &bytes, export).unwrap();
                assert!(sound.format.eq_ignore_ascii_case("WAV"), "{}", sound.format);
                assert_eq!(&sound.encoded[..4], b"RIFF");
                assert_eq!(&sound.encoded[8..12], b"WAVE");
                assert_eq!(
                    u32::from_le_bytes(sound.encoded[4..8].try_into().unwrap()) as usize + 8,
                    sound.encoded.len()
                );
                count += 1;
            }
            assert!(count > 0);
            total += count;
            eprintln!("{name}: {count} exports Sound lidos inteiros");
        }
        assert!(total > 1);
    }
}
