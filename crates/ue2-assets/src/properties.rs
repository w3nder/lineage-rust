//! The tagged property list that starts every serialized object.

use ue2_core::Reader;
use ue2_package::{Export, ObjectRef, Package};

use crate::Error;

const BYTE: u8 = 1;
const INT: u8 = 2;
const BOOL: u8 = 3;
const FLOAT: u8 = 4;
const OBJECT: u8 = 5;
const NAME: u8 = 6;
const CLASS: u8 = 8;
const ARRAY: u8 = 9;
const STRUCT: u8 = 10;

/// Objects that carry a script state frame before their properties.
const HAS_STACK: u32 = 0x0200_0000;

/// One tagged property. The value stays raw; accessors read it when its type matches.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Property<'a> {
    pub name: &'a str,
    /// Element index for fixed-size array properties; 0 otherwise.
    pub index: u32,
    kind: u8,
    struct_name: Option<&'a str>,
    value: &'a [u8],
    /// Booleans keep their value in the tag, not in `value`.
    flag: bool,
}

impl<'a> Property<'a> {
    pub fn byte(&self) -> Option<u8> {
        (self.kind == BYTE).then(|| self.value.first().copied()).flatten()
    }

    pub fn int(&self) -> Option<i32> {
        (self.kind == INT).then(|| Reader::at(self.value, 0).i32().ok()).flatten()
    }

    pub fn bool(&self) -> Option<bool> {
        (self.kind == BOOL).then_some(self.flag)
    }

    pub fn float(&self) -> Option<f32> {
        (self.kind == FLOAT).then(|| Reader::at(self.value, 0).f32().ok()).flatten()
    }

    /// An object or class reference.
    pub fn object(&self, package: &Package) -> Option<ObjectRef> {
        matches!(self.kind, OBJECT | CLASS).then(|| object(package, &mut Reader::at(self.value, 0))).flatten()
    }

    pub fn name_value(&self, package: &'a Package) -> Option<&'a str> {
        (self.kind == NAME).then(|| package.name_at(Reader::at(self.value, 0).compact().ok()?).ok()).flatten()
    }

    pub fn names(&self, package: &'a Package) -> Option<Vec<&'a str>> {
        if self.kind != ARRAY { return None; }
        let mut reader = Reader::at(self.value, 0);
        let count = usize::try_from(reader.compact().ok()?).ok()?;
        let names = (0..count).map(|_| package.name_at(reader.compact().ok()?).ok()).collect::<Option<Vec<_>>>()?;
        reader.remaining().is_empty().then_some(names)
    }

    pub fn vector(&self) -> Option<[f32; 3]> {
        let mut reader = self.struct_value("Vector")?;
        Some([reader.f32().ok()?, reader.f32().ok()?, reader.f32().ok()?])
    }

    pub fn plane(&self, package: &'a Package) -> Option<[f32; 4]> {
        if self.kind != STRUCT || self.struct_name != Some("Plane") {
            return None;
        }
        let fields = self.fields(package)?;
        Some(["X", "Y", "Z", "W"].map(|name| find(&fields, name).and_then(Self::float).unwrap_or(0.0)))
    }

    pub fn floats(&self) -> Option<Vec<f32>> {
        if self.kind != ARRAY {
            return None;
        }
        let mut reader = Reader::at(self.value, 0);
        let count = usize::try_from(reader.compact().ok()?).ok()?;
        let values = (0..count).map(|_| reader.f32().ok()).collect::<Option<Vec<_>>>()?;
        reader.remaining().is_empty().then_some(values)
    }

    /// A `Color` struct as RGBA; Unreal stores it blue first.
    pub fn color(&self) -> Option<[u8; 4]> {
        match (self.kind, self.struct_name, self.value) {
            (STRUCT, Some("Color"), &[b, g, r, a]) => Some([r, g, b, a]),
            _ => None,
        }
    }

    /// Pitch, yaw and roll in Unreal units (65536 to a full turn).
    pub fn rotator(&self) -> Option<[i32; 3]> {
        let mut reader = self.struct_value("Rotator")?;
        Some([reader.i32().ok()?, reader.i32().ok()?, reader.i32().ok()?])
    }

    /// The fields of a struct stored as a property list of its own (not an atomic `Vector` or `Color`).
    pub fn fields(&self, package: &'a Package) -> Option<Vec<Property<'a>>> {
        (self.kind == STRUCT).then(|| read_properties(&mut Reader::at(self.value, 0), package).ok()).flatten()
    }

    /// A dynamic array of structs, each a property list of its own.
    pub fn structs(&self, package: &'a Package) -> Option<Vec<Vec<Property<'a>>>> {
        if self.kind != ARRAY {
            return None;
        }
        let mut reader = Reader::at(self.value, 0);
        let count = usize::try_from(reader.compact().ok()?).ok()?;
        let structs = (0..count).map(|_| read_properties(&mut reader, package).ok()).collect::<Option<Vec<_>>>()?;
        reader.remaining().is_empty().then_some(structs)
    }

    /// A dynamic array of 32-bit integers; `None` when the bytes are not exactly that.
    pub fn ints(&self) -> Option<Vec<i32>> {
        if self.kind != ARRAY {
            return None;
        }
        let mut reader = Reader::at(self.value, 0);
        let count = usize::try_from(reader.compact().ok()?).ok()?;
        let ints = (0..count).map(|_| reader.i32().ok()).collect::<Option<Vec<_>>>()?;
        reader.remaining().is_empty().then_some(ints)
    }

    /// A dynamic array of `Color` (stored blue first) as RGBA; `None` when the bytes are not exactly that.
    pub fn colors(&self) -> Option<Vec<[u8; 4]>> {
        if self.kind != ARRAY {
            return None;
        }
        let mut reader = Reader::at(self.value, 0);
        let count = usize::try_from(reader.compact().ok()?).ok()?;
        let colors = (0..count).map(|_| reader.array::<4>().ok().map(|[b, g, r, a]| [r, g, b, a])).collect::<Option<Vec<_>>>()?;
        reader.remaining().is_empty().then_some(colors)
    }

    /// A dynamic array of object references; `None` when the bytes are not exactly that.
    pub fn objects(&self, package: &Package) -> Option<Vec<ObjectRef>> {
        if self.kind != ARRAY {
            return None;
        }
        let mut reader = Reader::at(self.value, 0);
        let count = usize::try_from(reader.compact().ok()?).ok()?;
        let objects = (0..count).map(|_| object(package, &mut reader)).collect::<Option<Vec<_>>>()?;
        reader.remaining().is_empty().then_some(objects)
    }

    fn struct_value(&self, name: &str) -> Option<Reader<'a>> {
        (self.kind == STRUCT && self.struct_name == Some(name) && self.value.len() == 12)
            .then(|| Reader::at(self.value, 0))
    }
}

fn object(package: &Package, reader: &mut Reader<'_>) -> Option<ObjectRef> {
    package.object_at(reader.compact().ok()?).ok()
}

/// Properties of `export`, past the state frame objects with script state carry.
pub fn object_properties<'a>(
    package: &'a Package,
    file: impl Into<ue2_core::ByteWindow<'a>>,
    export: &Export,
) -> Result<Vec<Property<'a>>, Error> {
    object_properties_reader(package, file, export).map(|(properties, _)| properties)
}

/// Properties of `export` and the bytes of the object after them, for native data that follows.
pub fn object_data<'a>(
    package: &'a Package,
    file: impl Into<ue2_core::ByteWindow<'a>>,
    export: &Export,
) -> Result<(Vec<Property<'a>>, &'a [u8]), Error> {
    object_properties_reader(package, file, export).map(|(properties, reader)| (properties, reader.remaining()))
}

pub fn class_default_properties<'a>(package: &'a Package, file: impl Into<ue2_core::ByteWindow<'a>>, export: &Export) -> Result<Vec<Property<'a>>, Error> {
    let mut reader = file.into().range(export.serial_offset, export.serial_size).ok_or(Error::ExportOutOfRange)?.reader();
    for _ in 0..6 {
        reader.compact()?;
    }
    reader.i32()?;
    reader.i32()?;
    let script = reader.i32()?;
    crate::script::skip(&mut reader, package, script)?;
    reader.bytes(16)?;
    reader.u16()?;
    reader.u32()?;
    reader.u32()?;
    reader.bytes(16)?;
    for _ in 0..reader.compact()? {
        reader.compact()?;
        reader.u32()?;
        reader.u32()?;
    }
    for _ in 0..reader.compact()? {
        reader.compact()?;
    }
    reader.compact()?;
    reader.compact()?;
    for _ in 0..reader.compact()? {
        reader.compact()?;
    }
    read_properties(&mut reader, package)
}

/// Properties of `export` and a reader just past them, bounded by the end of the object.
pub(crate) fn object_properties_reader<'a>(
    package: &'a Package,
    file: impl Into<ue2_core::ByteWindow<'a>>,
    export: &Export,
) -> Result<(Vec<Property<'a>>, Reader<'a>), Error> {
    let mut reader = file.into().range(export.serial_offset, export.serial_size).ok_or(Error::ExportOutOfRange)?.reader();
    if export.flags & HAS_STACK != 0 {
        let node = reader.compact()?;
        reader.compact()?; // state node
        reader.bytes(8)?; // probe mask
        reader.u32()?; // latent action
        if node != 0 {
            reader.compact()?; // code offset
        }
    }
    let properties = read_properties(&mut reader, package)?;
    Ok((properties, reader))
}

/// Reads the tagged property list at the reader, up to `None`.
pub(crate) fn read_properties<'a>(reader: &mut Reader<'a>, package: &'a Package) -> Result<Vec<Property<'a>>, Error> {
    let mut properties = Vec::new();
    loop {
        let name = package.name_at(reader.compact()?)?;
        if name.eq_ignore_ascii_case("None") {
            return Ok(properties);
        }
        let info = reader.u8()?;
        let kind = info & 0x0F;
        if kind == 0 {
            return Err(Error::UnknownPropertyType(kind));
        }
        let struct_name = if kind == STRUCT { Some(package.name_at(reader.compact()?)?) } else { None };
        let size = match (info >> 4) & 0x07 {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 12,
            4 => 16,
            5 => usize::from(reader.u8()?),
            6 => usize::from(reader.u16()?),
            _ => reader.u32()? as usize,
        };
        // For booleans the high bit is the value, not an array flag.
        let flag = info & 0x80 != 0;
        let index = if flag && kind != BOOL { array_index(reader)? } else { 0 };
        let value = if kind == BOOL { &[][..] } else { reader.bytes(size)? };
        properties.push(Property { name, index, kind, struct_name, value, flag });
    }
}

/// One byte, or two when the top bit is set, or four when the top two bits are set.
fn array_index(reader: &mut Reader<'_>) -> Result<u32, Error> {
    let first = reader.u8()?;
    let extra = match first {
        0..0x80 => 0,
        0x80..0xC0 => 1,
        _ => 3,
    };
    let mut index = u32::from(if extra == 0 { first } else { first & 0x3F });
    for byte in reader.bytes(extra)? {
        index = (index << 8) | u32::from(*byte);
    }
    Ok(index)
}

/// The first property called `name`, ignoring case as Unreal names do.
pub fn find<'p, 'a>(properties: &'p [Property<'a>], name: &str) -> Option<&'p Property<'a>> {
    properties.iter().find(|property| property.name.eq_ignore_ascii_case(name))
}
