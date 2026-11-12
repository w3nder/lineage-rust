use ue2_core::{ByteWindow, Reader};
use ue2_package::ObjectRef;
use super::{array, count, vector};
use crate::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct CollisionTriangle {
    pub planes: [[f32; 4]; 4],
    pub indices: [i32; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollisionNode {
    pub indices: [i32; 4],
    pub bounds: [[f32; 3]; 2],
    pub valid: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshCollision {
    pub simple_line: bool,
    pub simple_box: bool,
    pub simple_karma: bool,
    pub model: ObjectRef,
    pub triangles: std::ops::Range<usize>,
    pub nodes: std::ops::Range<usize>,
}

impl MeshCollision {
    pub fn read_triangles<'a>(&self, file: impl Into<ByteWindow<'a>>) -> Result<Vec<CollisionTriangle>, Error> {
        read_array(file.into(), &self.triangles, |reader| {
            let mut planes = [[0.0; 4]; 4];
            for plane in &mut planes {
                for value in plane { *value = reader.f32()?; }
            }
            Ok(CollisionTriangle { planes, indices: indices(reader)? })
        })
    }

    pub fn read_nodes<'a>(&self, file: impl Into<ByteWindow<'a>>) -> Result<Vec<CollisionNode>, Error> {
        read_array(file.into(), &self.nodes, |reader| {
            Ok(CollisionNode {
                indices: indices(reader)?,
                bounds: [vector(reader)?, vector(reader)?],
                valid: reader.u8()?,
            })
        })
    }
}

pub(super) fn immediate_range(reader: &mut Reader<'_>, triangles: bool) -> Result<std::ops::Range<usize>, Error> {
    let start = reader.pos();
    let len = count(reader)?;
    if len > reader.remaining().len() { return Err(Error::BadMesh); }
    for _ in 0..len {
        if triangles { reader.bytes(64)?; }
        indices(reader)?;
        if !triangles { reader.bytes(25)?; }
    }
    Ok(start..reader.pos())
}

pub(super) fn lazy_range(reader: &mut Reader<'_>) -> Result<std::ops::Range<usize>, Error> {
    let end = usize::try_from(reader.i32()?).map_err(|_| Error::BadMesh)?;
    let start = reader.pos();
    if end <= start { return Err(Error::BadMesh); }
    reader.seek(end)?;
    Ok(start..end)
}

fn indices(reader: &mut Reader<'_>) -> Result<[i32; 4], Error> {
    Ok([reader.compact()?, reader.compact()?, reader.compact()?, reader.compact()?])
}

fn read_array<'a, T>(file: ByteWindow<'a>, range: &std::ops::Range<usize>, item: impl FnMut(&mut Reader<'a>) -> Result<T, Error>) -> Result<Vec<T>, Error> {
    let len = range.end.checked_sub(range.start).ok_or(Error::BadMesh)?;
    let mut reader = file.range(range.start, len).ok_or(Error::BadMesh)?.reader();
    let result = array(&mut reader, item)?;
    if !reader.remaining().is_empty() { return Err(Error::BadMesh); }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lazy_bounds_reject_truncation_backtracking_and_trailing_data() {
        for end in [0_i32, 4, 6, -1] {
            let mut bytes = end.to_le_bytes().to_vec();
            bytes.push(0);
            assert!(lazy_range(&mut Reader::at(&bytes, 0)).is_err());
        }
        let bytes = [5, 0, 0, 0, 0];
        let range = lazy_range(&mut Reader::at(&bytes, 0)).unwrap();
        let mut collision = MeshCollision { simple_line:false,simple_box:true,simple_karma:false,model: ObjectRef::Null, triangles: range.clone(), nodes: range };
        assert!(collision.read_triangles(&bytes).unwrap().is_empty());
        assert!(collision.read_nodes(&bytes).unwrap().is_empty());
        collision.nodes = 3..5;
        assert!(collision.read_nodes(&bytes).is_err());
        collision.nodes = 5..6;
        assert!(collision.read_nodes(&bytes).is_err());
    }
}
