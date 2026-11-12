use glam::Mat4;
use super::BoxCheck;
use ue2_assets::{CollisionNode, CollisionTriangle};
use std::collections::HashSet;
use super::model::Hit;

pub struct Mesh<'a> {
    pub vertices: &'a [[f32; 3]],
    pub triangles: &'a [CollisionTriangle],
    pub nodes: &'a [CollisionNode],
    pub transform: Mat4,
    pub world_to_local: Mat4,
}

pub(super) type Rows = [[f32;4];4];

impl Mesh<'_> {
    pub fn trace(&self, start: [f32; 3], end: [f32; 3], extent: [f32; 3]) -> Option<Hit> {
        if extent == [0.0;3] { return self.trace_line(start,end); }
        let world_to_local = self.world_to_local.to_cols_array_2d();
        let local_to_world = self.transform.to_cols_array_2d();
        let local_start = transform_plane(world_to_local,start);
        let local_end = transform_plane(world_to_local,end);
        let delta: [f32;3] = std::array::from_fn(|k| {
            let value = (f64::from(local_end[k])-f64::from(local_start[k])) as f32;
            if value == 0.0 { (f64::from(value)+f64::from(1e-7_f32)) as f32 } else { value }
        });
        let mut walk = Walk {
            mesh:self,local_to_world,flip:(determinant(local_to_world) as f32) < 0.0,start,end,extent,local_start,local_end,
            local_extent:box_extent(world_to_local,extent),delta,inverse:delta.map(|value| (1.0/f64::from(value)) as f32),
            visited:HashSet::new(),time:1.0,normal:[0.0;3],
        };
        if !walk.node(0) { return None; }
        Some(finish(start,end,walk.time,walk.normal))
    }

    fn trace_line(&self, start: [f32; 3], end: [f32; 3]) -> Option<Hit> {
        let world_to_local = self.world_to_local.to_cols_array_2d();
        let local_start = transform_plane(world_to_local,start);
        let local_end = transform_plane(world_to_local,end);
        let delta: [f32;3] = std::array::from_fn(|k| {
            let value = (f64::from(local_end[k])-f64::from(local_start[k])) as f32;
            if value == 0.0 { (f64::from(value)+f64::from(1e-7_f32)) as f32 } else { value }
        });
        let inverse = delta.map(|value| (1.0/f64::from(value)) as f32);
        let (time,plane) = self.line_node(0,local_start,local_end,delta,inverse)?;
        let rows = self.transform.to_cols_array_2d();
        let normal = std::array::from_fn(|j| (((f64::from(plane[0])*f64::from(rows[0][j])+f64::from(plane[1])*f64::from(rows[1][j]))
            +f64::from(plane[2])*f64::from(rows[2][j]))+0.0*f64::from(rows[3][j])) as f32);
        Some(finish(start,end,time,normal))
    }

    fn line_node(&self, mut index: i32, start: [f32;3], end: [f32;3], delta: [f32;3], inverse: [f32;3]) -> Option<(f32,[f32;4])> {
        while index != -1 {
            let node = self.nodes.get(usize::try_from(index).ok()?)?;
            let face = self.triangles.get(usize::try_from(node.indices[0]).ok()?)?;
            if !line_box_exact(node.bounds,start,delta,inverse) { return None; }
            let plane = face.planes[0];
            let [px,py,pz,pw] = plane.map(f64::from);
            let side = |point: [f32;3]| (((f64::from(point[2])*pz+f64::from(point[1])*py)+px*f64::from(point[0]))-pw) as f32;
            let near_side = side(start);
            let far_side = side(end);
            let low = -0.001_f32;
            let high = 0.001_f32;
            if near_side > low && far_side > low { index = node.indices[3]; continue; }
            if near_side < high && far_side < high { index = node.indices[2]; continue; }
            let near = usize::from(near_side > low);
            if let Some(hit) = self.line_node(node.indices[2+near],start,end,delta,inverse) { return Some(hit); }
            if let Some(hit) = self.line_node(node.indices[1],start,end,delta,inverse) { return Some(hit); }
            let time = (-(f64::from(near_side)/(f64::from(far_side)-f64::from(near_side)))) as f32;
            let t = f64::from(time);
            let point = [(f64::from((t*f64::from(delta[0])) as f32)+f64::from(start[0])) as f32,
                (t*f64::from(delta[1])+f64::from(start[1])) as f32,(t*f64::from(delta[2])+f64::from(start[2])) as f32];
            let inside = face.planes[1..].iter().all(|edge| {
                let [ex,ey,ez,ew] = edge.map(f64::from);
                (((f64::from(point[1])*ey+f64::from(point[2])*ez)+f64::from(point[0])*ex)-ew) < 0.0
            });
            if inside { return Some((time,plane)); }
            index = node.indices[3-near];
        }
        None
    }
}

fn finish(start: [f32;3], end: [f32;3], time: f32, normal: [f32;3]) -> Hit {
    let rounded: [f32;3] = std::array::from_fn(|k| (f64::from(end[k])-f64::from(start[k])) as f32);
    let size = f64::from(rounded[0])*f64::from(rounded[0])+f64::from(rounded[1])*f64::from(rounded[1]);
    let size = (size+f64::from(rounded[2])*f64::from(rounded[2])).sqrt() as f32;
    let lower = (f64::from(0.1_f32)/f64::from(size)) as f32;
    let upper = (1.0/f64::from(size)) as f32;
    let backoff = if 0.1_f32 < lower { lower } else if 0.1_f32 < upper { 0.1_f32 } else { upper };
    let time = f64::from(time)-f64::from(backoff);
    let time = if time >= 1.0 { 1.0 } else if time >= 0.0 { time as f32 } else { 0.0 };
    let [dx,dy,dz] = std::array::from_fn(|k| f64::from(end[k])-f64::from(start[k]));
    let position = [(f64::from((f64::from(dx as f32)*f64::from(time)) as f32)+f64::from(start[0])) as f32,
        (f64::from((dy*f64::from(time)) as f32)+f64::from(start[1])) as f32,
        (f64::from((dz*f64::from(time)) as f32)+f64::from(start[2])) as f32];
    Hit { time,normal:normalize(normal),position }
}

struct Walk<'a, 'b> {
    mesh: &'a Mesh<'b>,
    local_to_world: Rows,
    flip: bool,
    start: [f32;3],
    end: [f32;3],
    extent: [f32;3],
    local_start: [f32;3],
    local_end: [f32;3],
    local_extent: [f32;3],
    delta: [f32;3],
    inverse: [f32;3],
    visited: HashSet<usize>,
    time: f32,
    normal: [f32;3],
}

impl Walk<'_, '_> {
    fn node(&mut self, mut index: i32) -> bool {
        let mut hit = false;
        while index != -1 {
            let Some(node) = usize::try_from(index).ok().and_then(|index| self.mesh.nodes.get(index)) else { break; };
            let Some(face_index) = usize::try_from(node.indices[0]).ok() else { break; };
            let Some(face) = self.mesh.triangles.get(face_index) else { break; };
            let bounds = [std::array::from_fn(|k| (f64::from(node.bounds[0][k])-f64::from(self.local_extent[k])) as f32),
                std::array::from_fn(|k| (f64::from(node.bounds[1][k])+f64::from(self.local_extent[k])) as f32)];
            if !line_box_exact(bounds,self.local_start,self.delta,self.inverse) { break; }
            let plane = face.planes[0];
            let [px,py,pz,pw] = plane.map(f64::from);
            let side = |point: [f32;3]| (((f64::from(point[2])*pz+f64::from(point[1])*py)+px*f64::from(point[0]))-pw) as f32;
            let start = side(self.local_start);
            let end = side(self.local_end);
            let projected = |value: f32| f64::from(value.abs());
            let extent = ((projected(self.local_extent[2]*plane[2])+projected(self.local_extent[1]*plane[1]))
                +projected(plane[0]*self.local_extent[0])) as f32;
            let negative = -extent;
            if start <= negative && end <= negative { index = node.indices[2]; continue; }
            if start >= extent && end >= extent { index = node.indices[3]; continue; }
            let near = if start < end || start.is_nan() || end.is_nan() { 0 } else { 1 };
            if self.node(node.indices[2+near]) {
                hit = true;
                let t = f64::from(self.time);
                let x = f64::from((t*f64::from(self.delta[0])) as f32)+f64::from(self.local_start[0]);
                let y = f64::from((t*f64::from(self.delta[1])) as f32)+f64::from(self.local_start[1]);
                let z = t*f64::from(self.delta[2])+f64::from(self.local_start[2]);
                let at = ((x*px+y*py)+z*pz)-pw;
                if (near == 0 && at < f64::from(negative)) || (near == 1 && at > f64::from(extent)) { break; }
            }
            if self.node(node.indices[1]) { hit = true; }
            if self.visited.insert(face_index) {
                let vertices = face.indices[..3].iter().map(|&index| usize::try_from(index).ok()
                    .and_then(|index| self.mesh.vertices.get(index)).copied()).collect::<Option<Vec<_>>>();
                if let Some(vertices) = vertices {
                    let mut check = BoxCheck::new(self.start,self.end,self.extent,self.time);
                    if triangle_exact(&mut check,[vertices[0],vertices[1],vertices[2]],self.local_to_world,self.flip) && check.contact {
                        self.time = check.enter;
                        self.normal = check.normal;
                        hit = true;
                    }
                }
            }
            let mut active = [true,true];
            if hit {
                let t = f64::from(self.time);
                let x = t*f64::from(self.delta[0])+f64::from(self.local_start[0]);
                let y = f64::from((t*f64::from(self.delta[1])) as f32)+f64::from(self.local_start[1]);
                let z = f64::from((t*f64::from(self.delta[2])) as f32)+f64::from(self.local_start[2]);
                let at = ((y*py+z*pz)+x*px)-pw;
                active = [f64::from(start) <= f64::from(extent) || at <= f64::from(extent),
                    start.partial_cmp(&negative) != Some(std::cmp::Ordering::Less) || at.partial_cmp(&f64::from(negative)) != Some(std::cmp::Ordering::Less)];
            }
            if !active[1-near] { break; }
            index = node.indices[3-near];
        }
        hit
    }
}

fn transform_plane(rows: Rows, point: [f32;3]) -> [f32;3] {
    std::array::from_fn(|j| (((f64::from(point[0])*f64::from(rows[0][j])+f64::from(point[1])*f64::from(rows[1][j]))
        +f64::from(point[2])*f64::from(rows[2][j]))+f64::from(rows[3][j])) as f32)
}

pub(super) fn grow(bounds: &mut Option<[[f32;3];2]>, point: [f32;3]) {
    match bounds {
        None => *bounds = Some([point,point]),
        Some([min,max]) => for k in 0..3 {
            if min[k] > point[k] { min[k] = point[k]; }
            if max[k] < point[k] { max[k] = point[k]; }
        },
    }
}

fn box_extent(rows: Rows, extent: [f32;3]) -> [f32;3] {
    let mut bounds = None;
    for i in [-extent[0],extent[0]] {
        for j in [-extent[1],extent[1]] {
            for k in [-extent[2],extent[2]] { grow(&mut bounds,transform_plane(rows,[i,j,k])); }
        }
    }
    let [min,max] = bounds.unwrap_or([[0.0;3];2]);
    std::array::from_fn(|k| ((f64::from(max[k])-f64::from(min[k]))*0.5) as f32)
}

pub(super) fn determinant(m: Rows) -> f64 {
    let m = m.map(|row| row.map(f64::from));
    let a = m[2][2]*m[3][3]-m[3][2]*m[2][3];
    let b = m[3][3]*m[1][2]-m[3][2]*m[1][3];
    let c = m[2][3]*m[1][2]-m[2][2]*m[1][3];
    let d = m[3][3]*m[0][2]-m[3][2]*m[0][3];
    let e = m[2][3]*m[0][2]-m[2][2]*m[0][3];
    let f = f64::from((m[1][3]*m[0][2]-m[1][2]*m[0][3]) as f32);
    let t0 = ((a*m[1][1]-b*m[2][1])+c*m[3][1])*m[0][0];
    let t1 = ((a*m[0][1]-d*m[2][1])+e*m[3][1])*m[1][0];
    let t2 = ((b*m[0][1]-d*m[1][1])+f*m[3][1])*m[2][0];
    let t3 = ((c*m[0][1]-e*m[1][1])+f*m[2][1])*m[3][0];
    ((t0-t1)+t2)-t3
}

pub fn line_box_exact(bounds: [[f32;3];2], start: [f32;3], delta: [f32;3], inverse: [f32;3]) -> bool {
    let mut inside = true;
    let mut times = [0.0_f64;3];
    for k in 0..3 {
        let boundary = if start[k] < bounds[0][k] {
            if delta[k] <= 0.0 { return false; }
            bounds[0][k]
        } else if start[k] > bounds[1][k] {
            if delta[k].partial_cmp(&0.0) != Some(std::cmp::Ordering::Less) { return false; }
            bounds[1][k]
        } else { continue; };
        inside = false;
        let time = (f64::from(boundary)-f64::from(start[k]))*f64::from(inverse[k]);
        times[k] = if k < 2 { f64::from(time as f32) } else { time };
    }
    if inside { return true; }
    let mut time = times[2];
    if matches!(times[1].partial_cmp(&time),Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)) { time = times[1]; }
    if matches!(times[0].partial_cmp(&time),Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)) { time = times[0]; }
    if !(0.0..=1.0).contains(&time) { return false; }
    let point = [(time*f64::from(delta[0])+f64::from(start[0])) as f32,(time*f64::from(delta[1])+f64::from(start[1])) as f32,
        (f64::from((time*f64::from(delta[2])) as f32)+f64::from(start[2])) as f32];
    (0..3).all(|k| f64::from(bounds[0][k])-f64::from(0.1_f32) < f64::from(point[k]) && f64::from(point[k]) < f64::from(bounds[1][k])+f64::from(0.1_f32))
}

fn size_squared(v: [f32;3]) -> f64 {
    let [x,y,z] = v.map(f64::from);
    (x*x+y*y)+z*z
}

pub(super) fn safe_normal_core(v: [f32;3]) -> [f32;3] {
    let square = size_squared(v);
    if square < 1e-8 { return [0.0;3]; }
    let inverse = 1.0/square.sqrt();
    let rounded = f64::from(inverse as f32);
    [(rounded*f64::from(v[0])) as f32,(rounded*f64::from(v[1])) as f32,(inverse*f64::from(v[2])) as f32]
}

fn normalize(v: [f32;3]) -> [f32;3] {
    let square = size_squared(v);
    if square < 1e-8 || square.is_nan() { return v; }
    let inverse = 1.0/square.sqrt();
    v.map(|c| (inverse*f64::from(c)) as f32)
}

pub(super) fn plane_of(a: [f32;3], b: [f32;3], c: [f32;3]) -> [f32;4] {
    let u: [f64;3] = std::array::from_fn(|k| f64::from(b[k])-f64::from(a[k]));
    let v: [f64;3] = std::array::from_fn(|k| f64::from(c[k])-f64::from(a[k]));
    let uf = u.map(|value| f64::from(value as f32));
    let vf = v.map(|value| f64::from(value as f32));
    let first = safe_normal_core([(uf[1]*v[2]-u[2]*v[1]) as f32,(u[2]*v[0]-v[2]*uf[0]) as f32,(v[1]*uf[0]-uf[1]*v[0]) as f32]);
    let second = safe_normal_core([(uf[1]*vf[2]-uf[2]*vf[1]) as f32,(uf[2]*vf[0]-vf[2]*uf[0]) as f32,(vf[1]*uf[0]-uf[1]*vf[0]) as f32]);
    let w = ((f64::from(a[1])*f64::from(second[1])+f64::from(a[2])*f64::from(second[2]))+f64::from(a[0])*f64::from(second[0])) as f32;
    [first[0],first[1],first[2],w]
}

fn triangle_exact(state: &mut BoxCheck, vertices: [[f32;3];3], local_to_world: Rows, flip: bool) -> bool {
    let mut world = vertices.map(|point| transform_plane(local_to_world,point));
    let mut plane = plane_of(world[2],world[1],world[0]);
    if flip {
        plane = plane.map(|value| -value);
        world.swap(0,2);
    }
    let mut bounds = None;
    for point in world { grow(&mut bounds,point); }
    let Some(bounds) = bounds else { return false; };
    if !state.clip_box(bounds) || !state.clip_plane(plane) || !state.clip_plane(plane.map(|value| -value)) { return false; }
    let [nx,ny,nz] = [plane[0],plane[1],plane[2]].map(f64::from);
    for index in 0..3 {
        let point = world[index];
        let next = world[(index+1)%3];
        let edge: [f32;3] = std::array::from_fn(|k| (f64::from(next[k])-f64::from(point[k])) as f32);
        let [ex,ey,ez] = edge.map(f64::from);
        let orientation = [(ny*ez-nz*ey) as f32,(nz*ex-nx*ez) as f32,(nx*ey-ny*ex) as f32];
        for axis in 0..3 {
            if edge[(axis+1)%3] == 0.0 && edge[(axis+2)%3] == 0.0 { continue; }
            let unit = std::array::from_fn(|i| if i == axis { 1.0 } else { 0.0 });
            if !state.clip_edge(point,edge,orientation,unit) { return false; }
        }
    }
    true
}
