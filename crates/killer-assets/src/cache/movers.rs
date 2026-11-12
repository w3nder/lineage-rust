use super::{R, Reader, Writer, read_draw, write_draw};
use crate::scene::movers::{AttachedActor, MovingAsset};
use std::io::{Read, Write};
use ue2_level::{
    Placement,
    movers::{L2Movement, Mover},
};

fn placement<W: Write>(w: &mut Writer<W>, p: Placement) {
    w.f32s(&p.location);
    for v in p.rotation {
        w.i32(v);
    }
}

fn read_placement<I: Read>(r: &mut Reader<I>) -> R<Placement> {
    Ok(Placement { location: r.f32s()?, rotation: [r.i32()?, r.i32()?, r.i32()?] })
}

pub(super) fn write<W: Write>(w: &mut Writer<W>, a: &MovingAsset) {
    w.str(&a.name);
    w.bool(a.mesh.is_some());
    if let Some(mesh) = &a.mesh {
        w.str(mesh);
    }
    placement(w, a.origin);
    w.len(a.attached.len());
    for child in &a.attached {
        w.str(&child.name);
        placement(w, child.origin);
        w.bool(child.emitter.is_some());
        if let Some(index) = child.emitter {
            w.u32(index as u32);
        }
    }
    w.bool(a.movement.is_some());
    if let Some(m) = &a.movement {
        placement(w, m.origin);
        for v in m.initial_rotation {
            w.i32(v);
        }
        for v in m.max {
            w.i32(v);
        }
        for v in m.rate {
            w.i32(v);
        }
        w.f32s(&m.accel);
        w.bool(m.random_max);
        w.bool(m.random_start);
    }
    w.bool(a.mover.is_some());
    if let Some(m) = &a.mover {
        placement(w, m.base);
        for v in m.key_pos {
            w.f32s(&v);
        }
        for v in m.key_rot {
            for x in v {
                w.i32(x);
            }
        }
        for mesh in &m.meshes {
            w.bool(mesh.is_some());
            if let Some(mesh) = mesh {
                w.str(mesh);
            }
        }
        w.bool(m.server_id.is_some());
        if let Some(id) = m.server_id {
            w.i32(id);
        }
        w.f32(m.move_time);
        w.bool(m.glide);
    }
    for draws in std::iter::once(&a.draws).chain(&a.grades) {
        w.len(draws.len());
        for draw in draws {
            write_draw(w, draw);
        }
    }
}

pub(super) fn read<I: Read>(r: &mut Reader<I>) -> R<MovingAsset> {
    let name = r.str()?;
    let mesh = if r.bool()? { Some(r.str()?) } else { None };
    let origin = read_placement(r)?;
    let attached = r.vec(|r| {
        Ok(AttachedActor {
            name: r.str()?,
            origin: read_placement(r)?,
            emitter: if r.bool()? { Some(r.u32()? as usize) } else { None },
        })
    })?;
    let movement = if r.bool()? {
        Some(L2Movement {
            origin: read_placement(r)?,
            initial_rotation: [r.i32()?, r.i32()?, r.i32()?],
            max: [r.i32()?, r.i32()?, r.i32()?],
            rate: [r.i32()?, r.i32()?, r.i32()?],
            accel: r.f32s()?,
            random_max: r.bool()?,
            random_start: r.bool()?,
        })
    } else {
        None
    };
    let mover = if r.bool()? {
        let base = read_placement(r)?;
        let mut key_pos = [[0.0; 3]; 8];
        for value in &mut key_pos {
            *value = r.f32s()?;
        }
        let mut key_rot = [[0; 3]; 8];
        for value in &mut key_rot {
            *value = [r.i32()?, r.i32()?, r.i32()?];
        }
        let mut meshes = std::array::from_fn(|_| None);
        for mesh in &mut meshes {
            if r.bool()? {
                *mesh = Some(r.str()?);
            }
        }
        let server_id = if r.bool()? { Some(r.i32()?) } else { None };
        Some(Mover { base, key_pos, key_rot, meshes, server_id, move_time: r.f32()?, glide: r.bool()? })
    } else {
        None
    };
    let draws = r.vec(read_draw)?;
    let mut grades = std::array::from_fn(|_| Vec::new());
    for grade in &mut grades {
        *grade = r.vec(read_draw)?;
    }
    Ok(MovingAsset { name, mesh, origin, movement, mover, draws, grades, attached })
}
