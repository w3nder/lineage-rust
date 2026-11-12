use ue2_core::Reader;
use ue2_package::{Export, ObjectRef, Package};

use super::{array, skip_array, vector};
use crate::{Error, properties::object_properties_reader};

#[derive(Debug, Clone, PartialEq)]
pub struct VertexMesh {
    pub bounding_box: [[f32; 3]; 2],
    pub bounding_sphere: [f32; 4],
    pub vertex_count: usize,
    pub positions: Vec<u32>,
    pub normals: Vec<u32>,
    pub textures: Vec<ObjectRef>,
    pub scale: [f32; 3],
    pub origin: [f32; 3],
    pub rotation: [i32; 3],
    pub face_levels: Vec<u16>,
    pub faces: Vec<[u16; 4]>,
    pub collapse_wedges: Vec<u16>,
    pub wedges: Vec<VertexWedge>,
    pub materials: Vec<(u32, i32)>,
    pub sequences: Vec<VertexSequence>,
    pub frame_boxes: Vec<([[f32; 3]; 2], u8)>,
    pub frame_spheres: Vec<[f32; 4]>,
    pub words_1a8: Vec<[u32; 8]>,
    pub word_1b4: u32,
    pub words_148: Vec<u32>,
    pub words_160: Vec<u32>,
    pub word_190: i32,
    pub word_194: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VertexWedge {
    pub vertex: u16,
    pub uv: [f32; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct VertexSequence {
    pub name: String,
    pub groups: Vec<String>,
    pub start: usize,
    pub frames: usize,
    pub rate: f32,
    pub notifies: Vec<(f32, String, ObjectRef)>,
}

pub fn read_vertex_mesh<'a>(package: &'a Package, file: impl Into<ue2_core::ByteWindow<'a>>, export: &Export) -> Result<VertexMesh, Error> {
    if !matches!((package.version, package.licensee), (123, 27 | 28)) {
        return Err(Error::UnsupportedLayout { version: package.version, licensee: package.licensee });
    }
    let (_, mut reader) = object_properties_reader(package, file, export)?;
    let r = &mut reader;
    let bounding_box = [vector(r)?, vector(r)?];
    r.u8()?;
    let bounding_sphere = sphere(r)?;
    let version = r.i32()?;
    if !(2..=5).contains(&version) {
        return Err(Error::BadMesh);
    }
    let vertex_count = nonnegative(r)?;
    let positions = array(r, |r| Ok(r.u32()?))?;
    let textures = array(r, |r| Ok(package.object_at(r.compact()?)?))?;
    let scale = vector(r)?;
    let origin = vector(r)?;
    let rotation = [r.i32()?, r.i32()?, r.i32()?];
    let face_levels = array(r, |r| Ok(r.u16()?))?;
    let faces = array(r, |r| Ok([r.u16()?, r.u16()?, r.u16()?, r.u16()?]))?;
    let collapse_wedges = array(r, |r| Ok(r.u16()?))?;
    let wedges = array(r, |r| Ok(VertexWedge { vertex: r.u16()?, uv: [r.f32()?, r.f32()?] }))?;
    let materials = array(r, |r| Ok((r.u32()?, r.i32()?)))?;
    r.bytes(24)?;
    if version >= 3 {
        r.i32()?;
        r.compact()?;
        r.bytes(52)?;
    }
    if version >= 4 { r.f32()?; }
    if version >= 5 { r.i32()?; }
    let words_1a8 = array(r, |r| Ok([r.u32()?, r.u32()?, r.u32()?, r.u32()?, r.u32()?, r.u32()?, r.u32()?, r.u32()?]))?;
    let word_1b4 = r.u32()?;
    let words_148 = array(r, |r| Ok(r.u32()?))?;
    let words_160 = array(r, |r| Ok(r.u32()?))?;
    let sequences = array(r, |r| sequence(package, r))?;
    let normals = array(r, |r| Ok(r.u32()?))?;
    let word_190 = r.i32()?;
    let word_194 = r.i32()?;
    let frame_boxes = array(r, |r| Ok(([vector(r)?, vector(r)?], r.u8()?)))?;
    let frame_spheres = array(r, sphere)?;
    if !r.remaining().is_empty() { return Err(Error::TrailingBytes(r.remaining().len())); }
    if vertex_count == 0 || vertex_count > positions.len() || !positions.len().is_multiple_of(vertex_count)
        || positions.len() != normals.len()
        || face_levels.len() != faces.len() || collapse_wedges.len() != wedges.len()
        || wedges.iter().any(|w| usize::from(w.vertex) >= vertex_count)
        || faces.iter().any(|f| f[..3].iter().any(|&w| usize::from(w) >= wedges.len()) || usize::from(f[3]) >= materials.len())
        || sequences.iter().any(|s| s.frames == 0 || s.start.checked_add(s.frames).is_none_or(|end| end > positions.len() / vertex_count))
    { return Err(Error::BadMesh); }
    Ok(VertexMesh { bounding_box, bounding_sphere, vertex_count, positions, normals, textures, scale, origin, rotation, face_levels, faces, collapse_wedges, wedges, materials, sequences, frame_boxes, frame_spheres, words_1a8, word_1b4, words_148, words_160, word_190, word_194 })
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct VertexPoint {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

pub struct VertexPose<'a> {
    mesh: &'a VertexMesh,
    points: Vec<VertexPoint>,
    initialized: bool,
    tween_frame: f32,
    tween_sequence: Option<usize>,
    tween_progress: f32,
    active: usize,
}

impl<'a> VertexPose<'a> {
    pub fn new(mesh: &'a VertexMesh) -> Self {
        Self { mesh, points: vec![VertexPoint::default(); mesh.vertex_count], initialized: false, tween_frame: 0.0, tween_sequence: None, tween_progress: 1.0, active: 0 }
    }

    #[allow(clippy::manual_range_contains, reason = "preserva comparações nativas quando a divisão do tween produz NaN")]
    pub fn sample(&mut self, sequence: Option<usize>, frame: f32, limit: usize) -> Result<&[VertexPoint], Error> {
        if !frame.is_finite() { return Err(Error::BadMesh); }
        let seq = sequence.map(|i| self.mesh.sequences.get(i).ok_or(Error::BadMesh)).transpose()?;
        let count = limit.min(self.mesh.vertex_count);
        if frame >= 0.0 || !self.initialized {
            let (first, second, fraction) = if let Some(seq) = seq {
                let value = seq.frames as f32 * frame.max(0.0);
                if !value.is_finite() || value >= usize::MAX as f32 { return Err(Error::BadMesh); }
                let whole = value.floor() as usize;
                (seq.start + whole % seq.frames, seq.start + (whole + 1) % seq.frames, value - value.floor())
            } else { (0, 0, 0.0) };
            for (i, point) in self.points[..count].iter_mut().enumerate() {
                let a = packed_point(self.mesh, first, i)?;
                *point = if fraction > 0.0 { interpolate(a, packed_point(self.mesh, second, i)?, fraction) } else { a };
            }
            self.active = count;
            self.initialized = true;
        } else {
            self.active = self.active.min(count);
            let first = seq.map_or(0, |s| s.start);
            let start = seq.map_or(0.0, |s| -1.0 / s.frames as f32);
            let mut fraction = 1.0 - frame / self.tween_frame;
            if self.tween_sequence != sequence { self.tween_progress = 0.0; }
            if self.tween_sequence != sequence || fraction < 0.0 || fraction > 1.0 {
                self.tween_frame = start;
                fraction = 0.0;
                self.tween_sequence = sequence;
            }
            self.tween_progress += (1.0 - self.tween_progress) * fraction;
            if self.tween_progress > 0.97 {
                fraction = 0.0;
                for i in self.active..count { self.points[i] = packed_point(self.mesh, first, i)?; }
                self.active = count;
            }
            if fraction > 0.0 {
                for i in 0..self.active { self.points[i] = interpolate(self.points[i], packed_point(self.mesh, first, i)?, fraction); }
            }
            self.tween_frame = frame;
        }
        Ok(&self.points[..self.active])
    }
}

fn packed_point(mesh: &VertexMesh, frame: usize, vertex: usize) -> Result<VertexPoint, Error> {
    let index = frame.checked_mul(mesh.vertex_count).and_then(|n| n.checked_add(vertex)).ok_or(Error::BadMesh)?;
    let position = *mesh.positions.get(index).ok_or(Error::BadMesh)? as i32;
    let normal = *mesh.normals.get(index).ok_or(Error::BadMesh)?;
    Ok(VertexPoint {
        position: [(position << 21 >> 21) as f32, (position << 10 >> 21) as f32, (position >> 22) as f32],
        normal: [(normal & 1023) as f32 - 512.0, (normal >> 10 & 1023) as f32 - 512.0, (normal >> 20 & 1023) as f32 - 512.0],
    })
}

fn interpolate(a: VertexPoint, b: VertexPoint, fraction: f32) -> VertexPoint {
    VertexPoint {
        position: std::array::from_fn(|i| (b.position[i] - a.position[i]) * fraction + a.position[i]),
        normal: std::array::from_fn(|i| (b.normal[i] - a.normal[i]) * fraction + a.normal[i]),
    }
}

fn sphere(r: &mut Reader<'_>) -> Result<[f32; 4], Error> {
    Ok([r.f32()?, r.f32()?, r.f32()?, r.f32()?])
}

fn nonnegative(r: &mut Reader<'_>) -> Result<usize, Error> {
    usize::try_from(r.i32()?).map_err(|_| Error::BadMesh)
}

fn sequence(package: &Package, r: &mut Reader<'_>) -> Result<VertexSequence, Error> {
    r.i32()?;
    let name = package.name_at(r.compact()?)?.to_owned();
    let groups = array(r, |r| Ok(package.name_at(r.compact()?)?.to_owned()))?;
    let start = nonnegative(r)?;
    let frames = nonnegative(r)?;
    let notifies = array(r, |r| Ok((r.f32()?, package.name_at(r.compact()?)?.to_owned(), package.object_at(r.compact()?)?)))?;
    let rate = r.f32()?;
    r.bytes(12)?;
    r.compact()?;
    r.bytes(8)?;
    r.u8()?;
    skip_array(r, 8)?;
    array(r, |r| { r.i32()?; skip_array(r, 8) })?;
    r.bytes(8)?;
    skip_array(r, 8)?;
    Ok(VertexSequence { name, groups, start, frames, rate, notifies })
}
