//! Unreal Engine 2 package tables (names, imports, exports) as used by Lineage 2.
//!
//! Input is an already-decrypted package (see `l2-crypto`). Every index is
//! validated while parsing, so the accessors below never panic.

use std::fmt;
use std::io::{Read, Seek, SeekFrom};

use ue2_core::{ReadError, Reader};

pub const TAG: u32 = 0x9E2A_83C1;

#[derive(Debug)]
pub enum Error {
    BadTag(u32),
    Read(ReadError),
    NameOutOfRange(i32),
    ObjectOutOfRange(i32),
    SerialOutOfRange { export: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadTag(tag) => write!(f, "not a UE2 package (tag {tag:#010x})"),
            Self::Read(e) => e.fmt(f),
            Self::NameOutOfRange(i) => write!(f, "name index {i} out of range"),
            Self::ObjectOutOfRange(i) => write!(f, "object reference {i} out of range"),
            Self::SerialOutOfRange { export } => write!(f, "export {export} data lies outside the package"),
        }
    }
}

impl std::error::Error for Error {}

impl From<ReadError> for Error {
    fn from(error: ReadError) -> Self {
        Self::Read(error)
    }
}

/// Package-local object reference: 0 is none, negative an import, positive an export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectRef {
    Null,
    Import(usize),
    Export(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name {
    pub text: String,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub class_package: usize,
    pub class_name: usize,
    pub outer: ObjectRef,
    pub name: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    /// `Null` means the export is itself a class.
    pub class: ObjectRef,
    pub super_class: ObjectRef,
    pub outer: ObjectRef,
    pub name: usize,
    pub flags: u32,
    pub serial_offset: usize,
    pub serial_size: usize,
}

/// Tables are private so every stored index stays valid; like slice indexing, the
/// accessors panic only on an `ObjectRef` or name index that did not come from this package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub version: u16,
    pub licensee: u16,
    pub flags: u32,
    names: Vec<Name>,
    imports: Vec<Import>,
    exports: Vec<Export>,
}

trait IndexRead {
    fn seek(&mut self, position: usize) -> Result<(), ReadError>;
    fn u16(&mut self) -> Result<u16, ReadError>;
    fn u32(&mut self) -> Result<u32, ReadError>;
    fn i32(&mut self) -> Result<i32, ReadError>;
    fn compact(&mut self) -> Result<i32, ReadError>;
    fn string(&mut self) -> Result<String, ReadError>;
}

impl IndexRead for Reader<'_> {
    fn seek(&mut self, position: usize) -> Result<(), ReadError> { Reader::seek(self, position) }
    fn u16(&mut self) -> Result<u16, ReadError> { Reader::u16(self) }
    fn u32(&mut self) -> Result<u32, ReadError> { Reader::u32(self) }
    fn i32(&mut self) -> Result<i32, ReadError> { Reader::i32(self) }
    fn compact(&mut self) -> Result<i32, ReadError> { Reader::compact(self) }
    fn string(&mut self) -> Result<String, ReadError> { Reader::string(self) }
}

struct StreamIndex<R> {
    input: R,
    position: usize,
    size: usize,
}

impl<R: Read + Seek> StreamIndex<R> {
    fn value<T>(&mut self, decode: impl Fn(&mut Reader<'_>) -> Result<T, ReadError>) -> Result<T, ReadError> {
        let mut bytes = Vec::new();
        loop {
            let mut reader = ue2_core::ByteWindow { data: &bytes, base: self.position }.reader();
            match decode(&mut reader) {
                Ok(value) => { self.position = reader.pos(); return Ok(value); }
                Err(ReadError::UnexpectedEof(_)) => {
                    if bytes.len() >= self.size.saturating_sub(self.position) {
                        return Err(ReadError::UnexpectedEof(self.position + bytes.len()));
                    }
                    let mut byte = [0];
                    self.input.read_exact(&mut byte).map_err(|_| ReadError::UnexpectedEof(self.position + bytes.len()))?;
                    bytes.push(byte[0]);
                }
                Err(error) => return Err(error),
            }
        }
    }
}

impl<R: Read + Seek> IndexRead for StreamIndex<R> {
    fn seek(&mut self, position: usize) -> Result<(), ReadError> {
        if position > self.size { return Err(ReadError::UnexpectedEof(position)); }
        self.input.seek(SeekFrom::Start(position as u64)).map_err(|_| ReadError::UnexpectedEof(position))?;
        self.position = position;
        Ok(())
    }
    fn u16(&mut self) -> Result<u16, ReadError> { self.value(|r| r.u16()) }
    fn u32(&mut self) -> Result<u32, ReadError> { self.value(|r| r.u32()) }
    fn i32(&mut self) -> Result<i32, ReadError> { self.value(|r| r.i32()) }
    fn compact(&mut self) -> Result<i32, ReadError> { self.value(|r| r.compact()) }
    fn string(&mut self) -> Result<String, ReadError> { self.value(|r| r.string()) }
}

impl Package {
    pub fn parse(data: &[u8]) -> Result<Self, Error> {
        Self::read_index(&mut Reader::at(data, 0), data.len())
    }

    pub fn read(input: impl Read + Seek, size: usize) -> Result<Self, Error> {
        Self::read_index(&mut StreamIndex { input, position: 0, size }, size)
    }

    fn read_index(r: &mut impl IndexRead, size: usize) -> Result<Self, Error> {
        r.seek(0)?;
        let tag = r.u32()?;
        if tag != TAG {
            return Err(Error::BadTag(tag));
        }
        let version = r.u16()?;
        let licensee = r.u16()?;
        let flags = r.u32()?;
        let name_count = r.u32()?;
        let name_offset = r.u32()? as usize;
        let export_count = r.u32()?;
        let export_offset = r.u32()? as usize;
        let import_count = r.u32()?;
        let import_offset = r.u32()? as usize;

        r.seek(name_offset)?;
        let names = (0..name_count)
            .map(|_| Ok(Name { text: r.string()?, flags: r.u32()? }))
            .collect::<Result<Vec<_>, Error>>()?;

        let name = |raw: i32| name_index(raw, names.len());
        let object = |raw: i32| object_ref(raw, import_count as usize, export_count as usize);

        r.seek(import_offset)?;
        let imports = (0..import_count)
            .map(|_| {
                Ok(Import {
                    class_package: name(r.compact()?)?,
                    class_name: name(r.compact()?)?,
                    outer: object(r.i32()?)?,
                    name: name(r.compact()?)?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        r.seek(export_offset)?;
        let exports = (0..export_count as usize)
            .map(|index| {
                let class = object(r.compact()?)?;
                let super_class = object(r.compact()?)?;
                let outer = object(r.i32()?)?;
                let name = name(r.compact()?)?;
                let flags = r.u32()?;
                let raw_size = r.compact()?;
                let raw_offset = if raw_size != 0 { r.compact()? } else { 0 };
                // Custom packers leave the odd entry with negative values; the engine
                // could never load those, so they are exports without data.
                let (serial_size, serial_offset) = match (usize::try_from(raw_size), usize::try_from(raw_offset)) {
                    (Ok(size), Ok(offset)) => (size, offset),
                    _ => (0, 0),
                };
                if serial_offset.checked_add(serial_size).is_none_or(|end| end > size) {
                    return Err(Error::SerialOutOfRange { export: index });
                }
                Ok(Export { class, super_class, outer, name, flags, serial_offset, serial_size })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(Self { version, licensee, flags, names, imports, exports })
    }

    /// Resolves a name index read from export data.
    pub fn name_at(&self, raw: i32) -> Result<&str, Error> {
        Ok(self.name(name_index(raw, self.names.len())?))
    }

    /// Resolves an object reference read from export data.
    pub fn object_at(&self, raw: i32) -> Result<ObjectRef, Error> {
        object_ref(raw, self.imports.len(), self.exports.len())
    }

    pub fn names(&self) -> &[Name] {
        &self.names
    }

    pub fn imports(&self) -> &[Import] {
        &self.imports
    }

    pub fn exports(&self) -> &[Export] {
        &self.exports
    }

    #[expect(clippy::indexing_slicing, reason = "indices come from `parse`, which validates them")]
    pub fn name(&self, index: usize) -> &str {
        &self.names[index].text
    }

    #[expect(clippy::indexing_slicing, reason = "indices come from `parse`, which validates them")]
    pub fn object_name(&self, object: ObjectRef) -> &str {
        match object {
            ObjectRef::Null => "None",
            ObjectRef::Import(i) => self.name(self.imports[i].name),
            ObjectRef::Export(i) => self.name(self.exports[i].name),
        }
    }

    #[expect(clippy::indexing_slicing, reason = "indices come from `parse`, which validates them")]
    pub fn outer(&self, object: ObjectRef) -> ObjectRef {
        match object {
            ObjectRef::Null => ObjectRef::Null,
            ObjectRef::Import(i) => self.imports[i].outer,
            ObjectRef::Export(i) => self.exports[i].outer,
        }
    }

    pub fn class_name(&self, export: &Export) -> &str {
        match export.class {
            ObjectRef::Null => "Class",
            class => self.object_name(class),
        }
    }

    /// Dotted path through the outer chain, e.g. `Group.Object`.
    pub fn object_path(&self, object: ObjectRef) -> String {
        let mut parts = Vec::new();
        let mut current = object;
        // Bounded so a corrupt outer cycle cannot loop forever.
        while current != ObjectRef::Null && parts.len() < 64 {
            parts.push(self.object_name(current));
            current = self.outer(current);
        }
        parts.reverse();
        parts.join(".")
    }
}
fn name_index(raw: i32, count: usize) -> Result<usize, Error> {
    usize::try_from(raw).ok().filter(|&index| index < count).ok_or(Error::NameOutOfRange(raw))
}

fn object_ref(raw: i32, imports: usize, exports: usize) -> Result<ObjectRef, Error> {
    if raw == 0 {
        return Ok(ObjectRef::Null);
    }
    let index = raw.unsigned_abs() as usize - 1;
    let (count, reference): (usize, fn(usize) -> ObjectRef) =
        if raw < 0 { (imports, ObjectRef::Import) } else { (exports, ObjectRef::Export) };
    if index >= count {
        return Err(Error::ObjectOutOfRange(raw));
    }
    Ok(reference(index))
}
