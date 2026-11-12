use glam::Vec3;
use super::model::Hit;
use crate::scene::terrain_normals::safe_normal;
use ue2_level::Coords;

pub struct Vertices {
    pub size: [usize;2],
    pub points: Vec<[f32;3]>,
}

impl Vertices {
    pub fn new(terrain: &ue2_level::Terrain, size: [usize;2], samples: &[u16]) -> Result<Self,String> {
        let count = size[0]*size[1];
        if terrain.native.vertices.len() >= count {
            return Ok(Self { size,points:terrain.native.vertices[..count].to_vec() });
        }
        let coords = terrain.native.to_world.ok_or_else(|| format!("ToWorld ausente: {}",terrain.name))?;
        if samples.len() < count { return Err(format!("heightmap incompleto: {}",terrain.name)); }
        let points = (0..count).map(|index| transform_point_by(coords,[(index%size[0]) as f32,(index/size[0]) as f32,f32::from(samples[index])])).collect();
        Ok(Self { size,points })
    }

    pub fn position(&self, x: usize, y: usize) -> [f32;3] {
        self.points[y*self.size[0]+x]
    }
}

pub fn transform_point_by(coords: Coords, point: [f32;3]) -> [f32;3] {
    let d: [f64;3] = std::array::from_fn(|axis| f64::from(point[axis])-f64::from(coords.origin[axis]));
    let row = |axis: [f32;3]| ((d[0]*f64::from(axis[0])+d[1]*f64::from(axis[1]))+d[2]*f64::from(axis[2])) as f32;
    [row(coords.x_axis),row(coords.y_axis),row(coords.z_axis)]
}

pub struct QuadTrace {
    pub start: [f32;3],
    pub end: [f32;3],
    pub extent: [f32;3],
    pub to_heightmap: Coords,
    pub alternate: Option<Coords>,
    pub size: [usize;2],
}

impl QuadTrace {
    pub fn visit(&self, mut check: impl FnMut(usize,usize) -> bool) -> bool {
        if self.size[0] < 2 || self.size[1] < 2 { return false; }
        let mut start = transform(self.to_heightmap,self.start,true);
        let mut end = transform(self.to_heightmap,self.end,true);
        let radius = transform(self.to_heightmap,self.extent,false).x.abs();
        let direction = safe_normal(end-start);
        if let Some(coords) = self.alternate {
            start = transform(coords,self.start,true);
            end = transform(coords,self.end,true);
        }
        start -= direction*radius;
        end += direction*radius;
        let bounds = [self.size[0] as i32-2,self.size[1] as i32-2];
        if (0..2).any(|axis| (start[axis] < 0.0 && end[axis] < 0.0)
            || (start[axis] > bounds[axis] as f32 && end[axis] > bounds[axis] as f32)) { return false; }
        let cell = |value: f64,axis: usize| (value as i32).clamp(0,bounds[axis]);
        let first = [cell(start.x as f64,0),cell(start.y as f64,1)];
        let last = [cell(end.x as f64,0),cell(end.y as f64,1)];
        if first == last { return check(first[0] as usize,first[1] as usize); }
        let axis = usize::from(first[0] == last[0]);
        let other = 1-axis;
        let slope = (end[other]-start[other])/(end[axis]-start[axis]);
        let intercept = (end[other] as f64-slope as f64*end[axis] as f64) as f32;
        let step = if first[axis] <= last[axis] { 1 } else { -1 };
        let mut first_hit = None;
        let mut at = first[axis];
        while at*step <= last[axis]*step {
            let lo = (at as f64*slope as f64+intercept as f64) as f32;
            let hi = ((at+1) as f64*slope as f64+intercept as f64) as f32;
            let min = cell(lo.min(hi) as f64-radius as f64,other);
            let max = cell((lo.max(hi)+radius) as f64,other);
            for cross in min..=max {
                let coords = if axis == 0 { [at,cross] } else { [cross,at] };
                if check(coords[0] as usize,coords[1] as usize) { first_hit.get_or_insert(at); }
            }
            if first_hit.is_some_and(|first: i32| (first-at).abs() as f32 > radius) { break; }
            at += step;
        }
        first_hit.is_some()
    }
}

fn transform(coords: Coords,point: [f32;3],translate: bool) -> Vec3 {
    let point = Vec3::from(point)-if translate { Vec3::from(coords.origin) } else { Vec3::ZERO };
    Vec3::new(dot(point,Vec3::from(coords.x_axis)) as f32,dot(point,Vec3::from(coords.y_axis)) as f32,
        dot(point,Vec3::from(coords.z_axis)) as f32)
}

pub struct Quad {
    pub vertices: [[f32;3];4],
    pub turned: bool,
    pub inverted: bool,
    pub visible: bool,
    pub deleted: bool,
}

impl Quad {
    pub fn trace(&self, start: [f32;3], end: [f32;3], extent: [f32;3], skip_visibility: bool, hit: &mut Option<Hit>) -> bool {
        if self.deleted || (!self.visible && !skip_visibility) { return false; }
        let sign = if self.inverted { -1.0 } else { 1.0 };
        let shift = sign*f64::from(extent[2]);
        let v = self.vertices.map(|p| [p[0],p[1],(f64::from(p[2])+shift) as f32]);
        let d: [f32;3] = std::array::from_fn(|k| (f64::from(end[k])-f64::from(start[k])) as f32);
        let [dx,dy,dz] = d.map(f64::from);
        let faces = if self.turned { [[0,1,3],[3,1,2]] } else { [[0,1,2],[0,2,3]] };
        let mut accepted = false;
        for (face,[a,b,c]) in faces.into_iter().enumerate() {
            let (pb,pc) = if self.inverted { (b,c) } else { (c,b) };
            let n = face_normal(v[a],v[pb],v[pc]);
            let [nx,ny,nz] = n.map(f64::from);
            if ((ny*dy+nz*dz)+dx*nx).partial_cmp(&f64::from(-0.0001_f32)) != Some(std::cmp::Ordering::Less) { continue; }
            let p = v[[0,2][face]].map(f64::from);
            let w = ((nz*p[2]+p[0]*nx)+p[1]*ny) as f32;
            let point = line_plane(end,start,n,w);
            if let Some(old) = hit {
                let near = |q: [f32;3]| size_squared(std::array::from_fn(|k| (f64::from(q[k])-f64::from(start[k])) as f32)) as f32;
                if near(point).partial_cmp(&near(old.position)) != Some(std::cmp::Ordering::Less) { continue; }
            }
            let di: [f64;3] = std::array::from_fn(|k| f64::from(point[k])-f64::from(start[k]));
            let time = ((di[1]*dy+di[0]*dx)+di[2]*dz)/((dx*dx+dy*dy)+dz*dz);
            if !(0.0..=1.0).contains(&time) { continue; }
            let signed = n.map(|value| (f64::from(value)*sign) as f32).map(f64::from);
            let face_vertices = [a,b,c];
            let inside = (0..3).all(|edge| {
                let from = v[face_vertices[edge]];
                let to = v[face_vertices[(edge+1)%3]];
                let e: [f64;3] = std::array::from_fn(|k| f64::from(to[k])-f64::from(from[k]));
                let cz = f64::from((signed[1]*e[0]-e[1]*signed[0]) as f32);
                let cy = f64::from((e[2]*signed[0]-e[0]*signed[2]) as f32);
                let cx = e[1]*signed[2]-e[2]*signed[1];
                let side = |q: [f32;3]| (f64::from(q[1])*cy+f64::from(q[2])*cz)+cx*f64::from(q[0]);
                side(point)-side(from) <= f64::from(extent[0])
            });
            if !inside { continue; }
            let size = size_squared(d).sqrt() as f32;
            let time = (f64::from(time as f32)-0.5/f64::from(size)) as f32;
            let time = if time < 0.0 { 0.0 } else if time < 1.0 { time } else { 1.0 };
            let position = std::array::from_fn(|k| (f64::from((f64::from(d[k])*f64::from(time)) as f32)+f64::from(start[k])) as f32);
            *hit = Some(Hit { time, normal:n, position });
            accepted = true;
        }
        accepted
    }
}

fn size_squared(v: [f32;3]) -> f64 {
    let [x,y,z] = v.map(f64::from);
    (x*x+y*y)+z*z
}

fn safe_normal_exact(v: [f32;3]) -> [f32;3] {
    let square = size_squared(v);
    if square < 1e-8 { return [0.0;3]; }
    let inverse = 1.0/square.sqrt();
    let rounded = f64::from(inverse as f32);
    [(rounded*f64::from(v[0])) as f32,(rounded*f64::from(v[1])) as f32,(inverse*f64::from(v[2])) as f32]
}

fn face_normal(a: [f32;3], b: [f32;3], c: [f32;3]) -> [f32;3] {
    let u: [f64;3] = std::array::from_fn(|k| f64::from(b[k])-f64::from(a[k]));
    let vx = f64::from((f64::from(c[0])-f64::from(a[0])) as f32);
    let vy = f64::from((f64::from(c[1])-f64::from(a[1])) as f32);
    let vz = f64::from(c[2])-f64::from(a[2]);
    safe_normal_exact([(vy*u[2]-vz*u[1]) as f32,(vz*u[0]-vx*u[2]) as f32,(vx*u[1]-vy*u[0]) as f32])
}

fn line_plane(first: [f32;3], second: [f32;3], normal: [f32;3], w: f32) -> [f32;3] {
    let [nx,ny,nz] = normal.map(f64::from);
    let [fx,fy,fz] = first.map(f64::from);
    let dx = f64::from(second[0])-fx;
    let dy = f64::from((f64::from(second[1])-fy) as f32);
    let dz = f64::from((f64::from(second[2])-fz) as f32);
    let time = (f64::from(w)-((fx*nx+nz*fz)+ny*fy))/((dz*nz+dy*ny)+dx*nx);
    let rx = f64::from(dx as f32);
    [(f64::from((rx*time) as f32)+fx) as f32,(f64::from((dy*time) as f32)+fy) as f32,(dz*time+fz) as f32]
}

fn dot(a: Vec3,b: Vec3) -> f64 {
    a.x as f64*b.x as f64 + a.z as f64*b.z as f64 + a.y as f64*b.y as f64
}
