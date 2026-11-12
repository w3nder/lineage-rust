//! `MeshAnimation` objects as Lineage II High Five stores them: named sequences of per-bone keys.

use ue2_core::Reader;
use ue2_package::{Export, ObjectRef, Package};

use super::{array, count, vector};
use crate::Error;
use crate::properties::object_properties_reader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimBone {
    pub name: String,
    /// Index of the parent bone; the root is its own parent.
    pub parent: usize,
}

/// One bone's keys. A track with a single rotation or position holds it for the whole sequence.
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    /// Index into `MeshAnimation::bones`.
    pub bone: usize,
    /// Quaternions (x, y, z, w).
    pub rotations: Vec<[f32; 4]>,
    pub positions: Vec<[f32; 3]>,
    /// Key times in frames.
    pub times: Vec<f32>,
}

/// One `AnimNotify` of a sequence: when it fires (in frames) and the notify object's class.
#[derive(Debug, Clone, PartialEq)]
pub struct Notify {
    pub time: f32,
    pub class: String,
    pub export: Option<usize>,
    pub classes: Vec<NotifyClass>,
    pub properties: Vec<NotifyProperty>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NotifyClass {
    pub name: String,
    pub path: String,
    pub export: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotifyValue {
    Byte(u8),
    Int(i32),
    Bool(bool),
    Float(f32),
    Name(String),
    Object(String),
    Vector([f32; 3]),
    Rotator([i32; 3]),
    Color([u8; 4]),
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NotifyProperty {
    pub name: String,
    pub index: u32,
    pub value: NotifyValue,
}

impl Notify {
    pub fn new(time: f32, class: impl Into<String>) -> Self {
        let class = class.into();
        let classes = vec![NotifyClass { name: class.clone(), path: class.clone(), export: None }];
        Self { time, class, export: None, classes, properties: Vec::new() }
    }

    pub fn property(&self, name: &str) -> Option<&NotifyValue> {
        self.properties.iter().find(|property| property.index == 0 && property.name.eq_ignore_ascii_case(name)).map(|property| &property.value)
    }
}

fn class_chain(package: &Package, start: ObjectRef) -> Vec<NotifyClass> {
    let mut chain = Vec::new();
    let mut current = start;
    while chain.len() < 32 {
        match current {
            ObjectRef::Export(index) => {
                let Some(export) = package.exports().get(index) else { break };
                chain.push(NotifyClass { name: package.object_name(current).to_owned(), path: package.object_path(current), export: Some(index) });
                current = export.super_class;
            }
            ObjectRef::Import(_) => {
                chain.push(NotifyClass { name: package.object_name(current).to_owned(), path: package.object_path(current), export: None });
                break;
            }
            ObjectRef::Null => break,
        }
    }
    chain
}

pub fn read_notify_properties<'a>(package: &'a Package, file: impl Into<ue2_core::ByteWindow<'a>>, export: &Export) -> Vec<NotifyProperty> {
    let Ok((properties, _)) = object_properties_reader(package, file, export) else { return Vec::new() };
    notify_values(package, &properties)
}

pub fn read_class_notify_defaults<'a>(package: &'a Package, file: impl Into<ue2_core::ByteWindow<'a>>, export: &Export) -> Vec<NotifyProperty> {
    let Ok(properties) = crate::properties::class_default_properties(package, file, export) else { return Vec::new() };
    notify_values(package, &properties)
}

fn notify_values(package: &Package, properties: &[crate::properties::Property<'_>]) -> Vec<NotifyProperty> {
    properties
        .iter()
        .map(|property| {
            let value = if let Some(value) = property.float() {
                NotifyValue::Float(value)
            } else if let Some(value) = property.int() {
                NotifyValue::Int(value)
            } else if let Some(value) = property.bool() {
                NotifyValue::Bool(value)
            } else if let Some(value) = property.byte() {
                NotifyValue::Byte(value)
            } else if let Some(value) = property.name_value(package) {
                NotifyValue::Name(value.to_owned())
            } else if let Some(object) = property.object(package) {
                NotifyValue::Object(package.object_path(object))
            } else if let Some(value) = property.vector() {
                NotifyValue::Vector(value)
            } else if let Some(value) = property.rotator() {
                NotifyValue::Rotator(value)
            } else if let Some(value) = property.color() {
                NotifyValue::Color(value)
            } else {
                NotifyValue::Other
            };
            NotifyProperty { name: property.name.to_owned(), index: property.index, value }
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub name: String,
    pub frames: u32,
    /// Frames per second.
    pub rate: f32,
    pub attack_eff: f32,
    pub attack_end_eff: f32,
    pub notifies: Vec<Notify>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeshAnimation {
    pub bones: Vec<AnimBone>,
    pub sequences: Vec<Sequence>,
}

pub fn read_mesh_animation<'a>(package: &'a Package, file: impl Into<ue2_core::ByteWindow<'a>>, export: &Export) -> Result<MeshAnimation, Error> {
    super::skeletal::check_layout(package)?;
    let (_, mut reader) = object_properties_reader(package, file, export)?;
    let reader = &mut reader;
    reader.i32()?; // version
    let bones = array(reader, |reader| {
        let name = package.name_at(reader.compact()?)?.to_owned();
        reader.u32()?; // flags
        let parent = usize::try_from(reader.i32()?).map_err(|_| Error::BadMesh)?;
        Ok(AnimBone { name, parent })
    })?;

    // Motions are stored with the file offsets where the list and each motion end.
    let end = offset(reader)?;
    let motions = count(reader)?;
    if motions > reader.remaining().len() {
        return Err(Error::BadMesh);
    }
    let mut tracks = Vec::with_capacity(motions);
    for _ in 0..motions {
        let motion_end = offset(reader)?;
        tracks.push(motion(reader, bones.len())?);
        if reader.pos() != motion_end {
            return Err(Error::BadMesh);
        }
    }
    if reader.pos() != end {
        return Err(Error::BadMesh);
    }

    let sequences = array(reader, |reader| sequence(package, reader, package.licensee))?;
    if !reader.remaining().is_empty() {
        return Err(Error::TrailingBytes(reader.remaining().len()));
    }
    if sequences.len() != tracks.len() {
        return Err(Error::BadMesh);
    }
    let sequences = sequences
        .into_iter()
        .zip(tracks)
        .map(|((name, frames, rate, attack_eff, attack_end_eff, notifies), tracks)| Sequence { name, frames, rate, attack_eff, attack_end_eff, notifies, tracks })
        .collect();
    Ok(MeshAnimation { bones, sequences })
}

fn offset(reader: &mut Reader<'_>) -> Result<usize, Error> {
    usize::try_from(reader.i32()?).map_err(|_| Error::BadMesh)
}

fn motion(reader: &mut Reader<'_>, bones: usize) -> Result<Vec<Track>, Error> {
    reader.bytes(12 + 4 + 4 + 4)?; // root speed, track time, start bone and flags
    let mut bone_indices = array(reader, |reader| usize::try_from(reader.i32()?).map_err(|_| Error::BadMesh))?;
    let keys = array(reader, analog_track)?;
    analog_track(reader)?; // root track
    // Without bone indices, tracks follow the bones in order.
    if bone_indices.is_empty() {
        bone_indices = (0..keys.len()).collect();
    }
    if bone_indices.len() != keys.len() || bone_indices.iter().any(|&bone| bone >= bones) {
        return Err(Error::BadMesh);
    }
    Ok(bone_indices
        .into_iter()
        .zip(keys)
        .map(|(bone, (rotations, positions, times))| Track { bone, rotations, positions, times })
        .collect())
}

type Keys = (Vec<[f32; 4]>, Vec<[f32; 3]>, Vec<f32>);

fn analog_track(reader: &mut Reader<'_>) -> Result<Keys, Error> {
    reader.u32()?; // flags
    let rotations = array(reader, |reader| Ok([reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?]))?;
    let positions = array(reader, vector)?;
    let times = array(reader, |reader| Ok(reader.f32()?))?;
    Ok((rotations, positions, times))
}

const LICENSEE_PAIRS_ONLY: u16 = 26;

type Parsed = (String, u32, f32, f32, f32, Vec<Notify>);

fn sequence(package: &Package, reader: &mut Reader<'_>, licensee: u16) -> Result<Parsed, Error> {
    reader.i32()?;
    let name = package.name_at(reader.compact()?)?.to_owned();
    array(reader, |reader| Ok(reader.compact()?))?; // groups
    reader.i32()?; // start frame
    let frames = u32::try_from(reader.i32()?).map_err(|_| Error::BadMesh)?;
    let notifies = array(reader, |reader| {
        let time = reader.f32()?;
        reader.compact()?; // function
        let (class, export, classes) = match package.object_at(reader.compact()?)? {
            ObjectRef::Export(index) => match package.exports().get(index) {
                Some(object) => (package.class_name(object).to_owned(), Some(index), class_chain(package, object.class)),
                None => (String::new(), Some(index), Vec::new()),
            },
            object => (package.object_name(object).to_owned(), None, Vec::new()),
        };
        Ok(Notify { time, class, export, classes, properties: Vec::new() })
    })?;
    let rate = reader.f32()?;
    let attack_eff = reader.f32()?;
    let attack_end_eff = reader.f32()?;
    reader.bytes(4)?;
    reader.compact()?;
    reader.bytes(4 * 2)?;
    // `FUN_105af8c0`: licensee 26 keeps one list of pairs; later versions add a flag, groups of pairs and a second list.
    if licensee == LICENSEE_PAIRS_ONLY {
        pairs(reader)?;
    } else if licensee > LICENSEE_PAIRS_ONLY {
        reader.u8()?;
        pairs(reader)?;
        array(reader, |reader| {
            reader.i32()?;
            pairs(reader)
        })?;
        reader.bytes(4 * 2)?;
        pairs(reader)?;
    }
    Ok((name, frames, rate, attack_eff, attack_end_eff, notifies))
}

fn pairs(reader: &mut Reader<'_>) -> Result<(), Error> {
    super::skip_array(reader, 8)
}
