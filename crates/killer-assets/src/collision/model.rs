use glam::{Mat4, Vec3};
use ue2_level::{BspNode, BspRender, BspTree};
use super::BoxCheck;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub time: f32,
    pub normal: [f32; 3],
    pub position: [f32; 3],
}

pub struct Model<'a> {
    pub tree: &'a BspTree,
    pub data: &'a BspRender,
    pub transform: Option<Mat4>,
    pub unscaled: bool,
}

impl Model<'_> {
    pub fn trace(&self, start: [f32; 3], end: [f32; 3], extent: [f32; 3], flags: u32) -> Option<Hit> {
        if extent == [0.0;3] { return self.trace_line(start,end,flags); }
        let mut state = BoxCheck::new(start, end, extent, 2.0);
        let mut hit = false;
        self.walk(0, 0, self.data.root_outside, &mut state, &mut hit);
        if !hit { return None; }
        let delta: [f64;3] = std::array::from_fn(|axis| f64::from((f64::from(end[axis])-f64::from(start[axis])) as f32));
        let distance = (delta[0]*delta[0]+delta[1]*delta[1]+delta[2]*delta[2]).sqrt() as f32;
        let inverse = 1.0/f64::from(distance);
        let upper = f64::from((4.0*inverse) as f32);
        let lower = inverse*f64::from(0.1_f32);
        let tenth = f64::from(0.1_f32);
        let backoff = if tenth < lower { lower } else if tenth > upper { upper } else { tenth };
        let time = (f64::from(state.exit)-backoff).clamp(0.0,1.0) as f32;
        let t = f64::from(time);
        let position = [
            (f64::from((delta[0]*t) as f32)+f64::from(start[0])) as f32,
            (f64::from(start[1])+f64::from(((f64::from(end[1])-f64::from(start[1]))*t) as f32)) as f32,
            (f64::from(start[2])+f64::from(((f64::from(end[2])-f64::from(start[2]))*t) as f32)) as f32,
        ];
        (time < 1.0).then_some(Hit { time, normal: state.normal, position })
    }

    pub fn trace_line(&self,start: [f32;3],end: [f32;3],flags: u32) -> Option<Hit> {
        if self.tree.nodes.is_empty() { return None; }
        let mut state = Line { flags, outside_seen:false, hit:None };
        self.walk_line(0,0,[start,end],self.data.root_outside,&mut state);
        let (position,node) = state.hit?;
        let dxf = (f64::from(end[0])-f64::from(start[0])) as f32;
        let dy = f64::from(end[1])-f64::from(start[1]);
        let dz = f64::from(end[2])-f64::from(start[2]);
        let rounded = [dxf,dy as f32,dz as f32];
        let [hx,hy,hz]: [f64;3] = std::array::from_fn(|k| f64::from(position[k])-f64::from(start[k]));
        let dx = f64::from(dxf);
        let time = (((hx*dx+hy*dy)+hz*dz)/((dx*dx+dy*dy)+dz*dz)) as f32;
        let [rx,ry,rz] = rounded.map(f64::from);
        let size = (((rx*rx+ry*ry)+rz*rz).sqrt()) as f32;
        let time = f64::from(time)-0.5/f64::from(size);
        let time = if time < 0.0 { 0.0 } else if time < 1.0 { time as f32 } else { 1.0 };
        let t = f64::from(time);
        let position = [(f64::from((rx*t) as f32)+f64::from(start[0])) as f32,(f64::from(start[1])+f64::from((ry*t) as f32)) as f32,
            (f64::from(start[2])+f64::from((rz*t) as f32)) as f32];
        let raw = self.tree.nodes.get(node)?.plane;
        let mut normal = [raw[0],raw[1],raw[2]];
        if let Some(matrix) = self.transform {
            let adjoint = super::transform::transpose_adjoint(super::transform::rows(matrix));
            let turned = super::transform::transform_fplane(adjoint,[raw[0],raw[1],raw[2],0.0]);
            normal = [turned[0],turned[1],turned[2]];
            if !self.unscaled { normal = normalize(normal); }
        }
        let [nx,ny,nz] = normal.map(f64::from);
        if ((-ry)*ny+(-rz)*nz)+(-rx)*nx < 0.0 { normal = normal.map(|value| -value); }
        Some(Hit { time,normal,position })
    }

    fn walk_line(&self,mut parent: usize,mut index: i32,[mut start,end]: [[f32;3];2],mut outside: bool,state: &mut Line) -> bool {
        while let Ok(current) = usize::try_from(index) {
            let Some(node) = self.tree.nodes.get(current) else { return true; };
            let plane = self.plane(node.plane);
            let [px,py,pz,pw] = plane.map(f64::from);
            let [sx,sy,sz] = start.map(f64::from);
            let [ex,ey,ez] = end.map(f64::from);
            let (a,b) = if self.transform.is_some() {
                ((((sy*py+sz*pz)+sx*px)-pw) as f32,((ey*py+ez*pz)+ex*px)-pw)
            } else {
                ((((sx*px+sy*py)+sz*pz)-pw) as f32,((ex*px+ey*py)+ez*pz)-pw)
            };
            let solid = |mask: u8| node.vertices != 0 && node.flags & mask == 0;
            let low = f64::from(-0.001_f32);
            let high = f64::from(0.001_f32);
            if f64::from(a) > low && b > low {
                outside |= solid((state.flags as u8 & 0xef)|0x21);
                index = node.front;
            } else if f64::from(a) < high && b < high {
                outside &= !solid((state.flags as u8 & 0xef)|0x21);
                index = node.back;
            } else {
                let scale = f64::from((f64::from(a)/(b-f64::from(a))) as f32);
                let dx = f64::from((sx-ex) as f32);
                let middle = [(f64::from((dx*scale) as f32)+sx) as f32,(sy+f64::from(((sy-ey)*scale) as f32)) as f32,
                    (sz+f64::from(((sz-ez)*scale) as f32)) as f32];
                let front = a > 0.0;
                let csg = solid(state.flags as u8|0x21);
                let near_outside = if front { outside || csg } else { outside && !csg };
                let child = if front { node.front } else { node.back };
                if !self.walk_line(parent,child,[start,middle],near_outside,state) { return false; }
                outside = if front { outside && !csg } else { outside || csg };
                index = if front { node.back } else { node.front };
                parent = current;
                start = middle;
            }
        }
        if outside { state.outside_seen = true; }
        else if !state.outside_seen && state.flags & 0x10 != 0 { return true; }
        else { state.hit = Some((start,parent)); }
        outside
    }

    pub fn trace_raw(&self, start: [f32; 3], end: [f32; 3], extent: [f32; 3]) -> Option<Hit> {
        let mut state = BoxCheck::new(start, end, extent, 2.0);
        let mut hit = false;
        self.walk(0, 0, self.data.root_outside, &mut state, &mut hit);
        hit.then(|| Hit { time: state.exit, normal: state.normal, position: (Vec3::from(start) + (Vec3::from(end)-Vec3::from(start))*state.exit).to_array() })
    }

    pub(super) fn plane(&self, plane: [f32; 4]) -> [f32; 4] {
        let Some(matrix) = self.transform else { return plane; };
        let rows = super::transform::rows(matrix);
        super::transform::transform_by_using_adjoint_t(plane,rows,super::transform::transpose_adjoint(rows))
    }

    fn walk(&self, mut parent: usize, mut index: i32, mut outside: bool, state: &mut BoxCheck, hit: &mut bool) {
        while let Ok(current) = usize::try_from(index) {
            let Some(node) = self.tree.nodes.get(current) else { return; };
            let plane = self.plane(node.plane);
            let [px,py,pz,pw] = plane.map(f64::from);
            let [sx,sy,sz] = state.start.map(f64::from);
            let [ex,ey,ez] = state.end.map(f64::from);
            let start = ((py*sy+pz*sz)+px*sx)-pw;
            let end = ((px*ex+py*ey)+pz*ez)-pw;
            let grown = state.extent.map(|value| f64::from(value)*f64::from(1.1_f32));
            let a = f64::from(((grown[0]*px) as f32).abs());
            let b = f64::from(((py*grown[1]) as f32).abs());
            let c = f64::from(((pz*grown[2]) as f32).abs());
            let extent = (c+b)+a;
            let less = |value: f64,limit: f64| value.partial_cmp(&limit) == Some(std::cmp::Ordering::Less);
            let active = [start <= extent || end <= extent, !less(start,-extent) || !less(end,-extent)];
            let near = usize::from(!less(start,end));
            let children = [node.back, node.front];
            let solid = node.vertices != 0 && node.flags & 0x21 == 0;
            let next_outside = [outside && !solid, outside || solid];
            if active[near] { self.walk(current, children[near], next_outside[near], state, hit); }
            if !active[1-near] { return; }
            parent = current;
            index = children[1-near];
            outside = next_outside[1-near];
        }
        if outside { return; }
        let Some(node) = self.tree.nodes.get(parent) else { return; };
        self.hull(node, state, hit);
    }

    fn hull(&self, node: &BspNode, state: &mut BoxCheck, hit: &mut bool) {
        let Some(leaf) = self.leaf(node) else { return; };
        let mut check = BoxCheck::new(state.start, state.end, state.extent, state.exit);
        if !self.each_plane(&leaf, &mut |_, plane| clip_plane(&mut check, plane)) { return; }
        if check.enter > -1.0 && check.enter < check.exit && check.exit > 0.0 {
            state.exit = check.enter;
            state.normal = check.normal;
            *hit = true;
        }
    }

    pub(super) fn leaf(&self, node: &BspNode) -> Option<Leaf> {
        let offset = usize::try_from(node.collision_bound).ok()?;
        let data = self.data.leaf_hulls.get(offset..)?;
        let mut planes = Vec::new();
        let mut items = Vec::new();
        for &index in data.iter().take(64) {
            if index == -1 { break; }
            let node = self.tree.nodes.get((index as u32 & 0xbfff_ffff) as usize)?;
            let mut plane = self.plane(node.plane);
            if index as u32 & 0x4000_0000 != 0 { plane = plane.map(|v| -v); }
            planes.push(plane);
            items.push((index as u32 & 0xbfff_ffff) as i32);
        }
        let bounds = data.get(planes.len()+1..planes.len()+7)?;
        let bounds: Vec<_> = bounds.iter().map(|v| f32::from_bits(*v as u32)).collect();
        Some(Leaf { planes, items, bounds })
    }

    pub(super) fn each_plane(&self, leaf: &Leaf, clip: &mut dyn FnMut(i32, [f32; 4]) -> bool) -> bool {
        let planes = &leaf.planes;
        let bounds = &leaf.bounds;
        for (&plane, &item) in planes.iter().zip(&leaf.items) { if !clip(item, plane) { return false; } }
        for plane in [
            [0.0,0.0,-1.0,(0.1-bounds[2] as f64) as f32],
            [0.0,0.0,1.0,(bounds[5] as f64+0.1) as f32],
            [-1.0,0.0,0.0,(0.1-bounds[0] as f64) as f32],
            [1.0,0.0,0.0,(bounds[3] as f64-0.1) as f32],
            [0.0,-1.0,0.0,(0.1-bounds[1] as f64) as f32],
            [0.0,1.0,0.0,(bounds[4] as f64-0.1) as f32],
        ] { if !clip(-1, self.plane(plane)) { return false; } }
        for (i, &plane) in planes.iter().enumerate() {
            for &other in &planes[..i] {
                let mask = sign_mask(plane) | sign_mask(other);
                for axis in 0..3 {
                    if (mask >> (axis*2)) & 3 != 3 { continue; }
                    let own = axis_cross(axis,[plane[0],plane[1],plane[2]]);
                    let near = axis_cross(axis,[other[0],other[1],other[2]]);
                    let dot = (near[2]*own[2]+f64::from(near[1] as f32)*own[1])+near[0]*own[0];
                    if dot <= f64::from(0.001_f32) || dot.is_nan() { continue; }
                    let (point, direction) = intersect_planes(plane, other);
                    let mut n = unsafe_normal(axis_cross(axis,direction).map(|v| v as f32));
                    let p = |k: usize| f64::from(n[k])*f64::from(plane[k]);
                    let facing = if axis == 2 { (p(1)+p(2))+p(0) } else { (p(0)+p(1))+p(2) };
                    if facing < 0.0 { n = n.map(|v| -v); }
                    let q = |k: usize| f64::from(point[k])*f64::from(n[k]);
                    let w = if axis == 2 { (q(2)+q(0))+q(1) } else { (q(1)+q(2))+q(0) } as f32;
                    if !clip(-1, [n[0],n[1],n[2],w]) { return false; }
                }
            }
        }
        true
    }
}

pub(super) struct Leaf {
    pub(super) planes: Vec<[f32; 4]>,
    pub(super) items: Vec<i32>,
    pub(super) bounds: Vec<f32>,
}

struct Line {
    flags: u32,
    outside_seen: bool,
    hit: Option<([f32;3],usize)>,
}

pub fn clip_plane(state: &mut BoxCheck, plane: [f32; 4]) -> bool {
    let projected = |axis: usize| f64::from((state.extent[axis]*plane[axis]).abs());
    let extent = ((projected(2)+projected(1))+projected(0)) as f32 as f64;
    let side = |point: [f32;3]| ((f64::from(point[1])*f64::from(plane[1])+f64::from(point[2])*f64::from(plane[2]))
        +f64::from(plane[0])*f64::from(point[0])-f64::from(plane[3])) as f32 as f64;
    let start = side(state.start);
    let end = side(state.end);
    let mut numerator = start - extent;
    if end < start && numerator >= -extent && numerator < 0.0 { numerator = 0.0; }
    let delta = (start - end) as f32 as f64;
    let time = numerator/delta;
    if delta < -(1e-5_f32 as f64) {
        if time < state.exit as f64 { state.exit = time as f32; }
    } else if delta > 1e-5_f32 as f64 {
        if time > state.enter as f64 {
            state.enter = time as f32;
            state.normal = [plane[0],plane[1],plane[2]];
        }
    } else if extent < start && extent < end { return false; }
    state.exit > state.enter
}

fn sign_mask(plane: [f32; 4]) -> u32 {
    (0..3).map(|axis| {
        let bits = match plane[axis].partial_cmp(&0.0) { Some(std::cmp::Ordering::Greater) => 2, Some(std::cmp::Ordering::Equal) => 0, _ => 1 };
        bits << (axis*2)
    }).fold(0,|mask,bits| mask | bits)
}

fn axis_cross(axis: usize, v: [f32; 3]) -> [f64; 3] {
    let [x,y,z] = v.map(f64::from);
    match axis {
        0 => [z*0.0-y*0.0, x*0.0-z, y-x*0.0],
        1 => [z-y*0.0, x*0.0-z*0.0, y*0.0-x],
        _ => [z*0.0-y, x-z*0.0, y*0.0-x*0.0],
    }
}

fn size_squared(v: [f32; 3]) -> f64 {
    let [x,y,z] = v.map(f64::from);
    (x*x+y*y)+z*z
}

fn unsafe_normal(v: [f32; 3]) -> [f32; 3] {
    let inverse = 1.0/size_squared(v).sqrt();
    let rounded = f64::from(inverse as f32);
    [(rounded*f64::from(v[0])) as f32,(rounded*f64::from(v[1])) as f32,(inverse*f64::from(v[2])) as f32]
}

fn intersect_planes(a: [f32; 4], b: [f32; 4]) -> ([f32; 3], [f32; 3]) {
    let [ax,ay,az,aw] = a.map(f64::from);
    let [bx,by,bz,bw] = b.map(f64::from);
    let d = [(ay*bz-az*by) as f32,(az*bx-bz*ax) as f32,(ax*by-ay*bx) as f32];
    let square = size_squared(d);
    if square < f64::from(f32::from_bits(0x3586_37be)) { return ([0.0;3],[0.0;3]); }
    let rounded_square = square as f32;
    let [dx,dy,dz] = d.map(f64::from);
    let first = [f64::from((dy*az-ay*dz) as f32)*bw,(ax*dz-dx*az)*bw,(ay*dx-dy*ax)*bw].map(|v| v as f32);
    let second = [f64::from((dz*by-dy*bz) as f32)*aw,(dx*bz-bx*dz)*aw,(dy*bx-dx*by)*aw].map(|v| v as f32);
    let sum: [f32; 3] = std::array::from_fn(|k| (f64::from(second[k])+f64::from(first[k])) as f32);
    let reciprocal = 1.0/f64::from(rounded_square);
    let rounded = f64::from(reciprocal as f32);
    let point = [(rounded*f64::from(sum[0])) as f32,(rounded*f64::from(sum[1])) as f32,(reciprocal*f64::from(sum[2])) as f32];
    (point, normalize(d))
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let square = size_squared(v);
    if square < 1e-8 || square.is_nan() { return v; }
    let inverse = 1.0/square.sqrt();
    v.map(|c| (inverse*f64::from(c)) as f32)
}
