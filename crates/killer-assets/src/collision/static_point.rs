use std::cmp::Ordering;
use super::{point_check::PointHit, static_mesh::{Mesh,Rows,determinant,grow,plane_of}};

fn less<T: PartialOrd>(a: T, b: T) -> bool {
    a.partial_cmp(&b) == Some(Ordering::Less)
}

struct Probe {
    center: [f32;3],
    extent: [f32;3],
    normal: [f32;3],
    best: f32,
}

fn place(rows: &Rows, point: [f32;3]) -> [f32;3] {
    let [x,y,z] = point.map(f64::from);
    std::array::from_fn(|j| (((f64::from(rows[3][j])*1.0+z*f64::from(rows[2][j]))+y*f64::from(rows[1][j]))+x*f64::from(rows[0][j])) as f32)
}

fn box_plane(plane: [f32;4], probe: &mut Probe) -> bool {
    let [nx,ny,nz,w] = plane.map(f64::from);
    let [ex,ey,ez] = probe.extent.map(f64::from);
    let [cx,cy,cz] = probe.center.map(f64::from);
    let a = f64::from((ex*nx) as f32).abs();
    let b = f64::from((ey*ny) as f32).abs();
    let c = f64::from((ez*nz) as f32).abs();
    let reach = (c+b)+a;
    let dist = ((cy*ny+cz*nz)+nx*cx)-w;
    let depth = reach-dist;
    if less(depth, f64::from(probe.best)) {
        probe.normal = [plane[0],plane[1],plane[2]];
        probe.best = depth as f32;
    }
    less(0.0, depth)
}

fn box_overlaps_box(min: [f32;3], max: [f32;3], probe: &mut Probe) -> bool {
    [
        [-1.0,0.0,0.0,-min[0]], [1.0,0.0,0.0,max[0]],
        [0.0,-1.0,0.0,-min[1]], [0.0,1.0,0.0,max[1]],
        [0.0,0.0,-1.0,-min[2]], [0.0,0.0,1.0,max[2]],
    ].into_iter().all(|plane| box_plane(plane, probe))
}

fn unsafe_normal(v: [f32;3]) -> [f32;3] {
    let [x,y,z] = v.map(f64::from);
    let inverse = 1.0/((x*x+y*y)+z*z).sqrt();
    let rounded = f64::from(inverse as f32);
    [(rounded*x) as f32,(rounded*y) as f32,(inverse*z) as f32]
}

fn edge_axis(point: [f32;3], edge: [f32;3], reference: [f32;3], axis: [f32;3], probe: &mut Probe) -> bool {
    let [e0,e1,e2] = edge.map(f64::from);
    let [a0,a1,a2] = axis.map(f64::from);
    let cross = [(a1*e2-a2*e1) as f32,(a2*e0-a0*e2) as f32,(a0*e1-a1*e0) as f32];
    let mut n = unsafe_normal(cross);
    let [r0,r1,r2] = reference.map(f64::from);
    if less((f64::from(n[2])*r2+f64::from(n[1])*r1)+f64::from(n[0])*r0, 0.0) { n = n.map(|v| (-f64::from(v)) as f32); }
    let [p0,p1,p2] = point.map(f64::from);
    let w = ((p2*f64::from(n[2])+p1*f64::from(n[1]))+p0*f64::from(n[0])) as f32;
    box_plane([n[0],n[1],n[2],w], probe)
}

impl Mesh<'_> {
    pub fn point_check(&self, location: [f32;3], extent: [f32;3], hit: &mut PointHit) -> bool {
        let to_local = self.world_to_local.to_cols_array_2d();
        let to_world = self.transform.to_cols_array_2d();
        let flip = (determinant(to_world) as f32) < 0.0;
        let center = place(&to_local, location);
        let local_extent = {
            let mut bounds = None;
            for x in [-extent[0],extent[0]] { for y in [-extent[1],extent[1]] { for z in [-extent[2],extent[2]] {
                grow(&mut bounds, place(&to_local,[x,y,z]));
            } } }
            let [min,max] = bounds.unwrap_or([[0.0;3];2]);
            std::array::from_fn::<f32,3,_>(|k| ((f64::from(max[k])-f64::from(min[k]))*0.5) as f32)
        };
        let mut walk = Filter { mesh: self, to_world, flip, location, extent, center, local_extent, best: 100000.0, hit };
        let found = walk.node(0);
        if found { hit.normal = normalize(hit.normal); }
        found
    }
}

fn normalize(v: [f32;3]) -> [f32;3] {
    let [x,y,z] = v.map(f64::from);
    let square = (x*x+y*y)+z*z;
    if less(square, f64::from(1e-8_f32)) { return v; }
    let inverse = 1.0/square.sqrt();
    [(inverse*x) as f32,(inverse*y) as f32,(inverse*z) as f32]
}

struct Filter<'a, 'b, 'c> {
    mesh: &'a Mesh<'b>,
    to_world: Rows,
    flip: bool,
    location: [f32;3],
    extent: [f32;3],
    center: [f32;3],
    local_extent: [f32;3],
    best: f32,
    hit: &'c mut PointHit,
}

impl Filter<'_, '_, '_> {
    fn node(&mut self, mut index: i32) -> bool {
        let mut found = false;
        while index != -1 {
            let Some(node) = usize::try_from(index).ok().and_then(|i| self.mesh.nodes.get(i)) else { break; };
            let Some(triangle) = usize::try_from(node.indices[0]).ok().and_then(|i| self.mesh.triangles.get(i)) else { break; };
            let min: [f32;3] = std::array::from_fn(|k| (f64::from(node.bounds[0][k])-f64::from(self.local_extent[k])) as f32);
            let max: [f32;3] = std::array::from_fn(|k| (f64::from(node.bounds[1][k])+f64::from(self.local_extent[k])) as f32);
            let c = self.center;
            let outside = |k: usize| less(c[k], min[k]) || !matches!(c[k].partial_cmp(&max[k]), Some(Ordering::Less | Ordering::Equal));
            if outside(0) || outside(1) || outside(2) { break; }
            let [nx,ny,nz,w] = triangle.planes[0].map(f64::from);
            let [lx,ly,lz] = self.center.map(f64::from);
            let dist = (((nx*lx+lz*nz)+ly*ny)-w) as f32;
            let [ex,ey,ez] = self.local_extent.map(f64::from);
            let reach = (f64::from((ez*nz) as f32).abs()+f64::from((ey*ny) as f32).abs()+f64::from((nx*ex) as f32).abs()) as f32;
            if !less(-reach, dist) { index = node.indices[2]; continue; }
            if !less(dist, reach) { index = node.indices[3]; continue; }
            let front = usize::from(less(0.0, dist));
            if self.node(node.indices[2+front]) || self.node(node.indices[1]) { found = true; }
            let mut probe = Probe { center: self.location, extent: self.extent, normal: self.hit.normal, best: self.best };
            if self.triangle(triangle, &mut probe) {
                let [nx,ny,nz] = probe.normal.map(f64::from);
                let best = f64::from(probe.best);
                self.hit.location = [(nx*best+f64::from(self.location[0])) as f32,(ny*best+f64::from(self.location[1])) as f32,
                    (f64::from((nz*best) as f32)+f64::from(self.location[2])) as f32];
                self.hit.normal = probe.normal;
                self.hit.item = node.indices[0];
                found = true;
                self.best = probe.best;
            }
            index = node.indices[3-front];
        }
        found
    }

    fn triangle(&self, triangle: &ue2_assets::CollisionTriangle, probe: &mut Probe) -> bool {
        let mut verts = [0, 1, 2].map(|k| usize::try_from(triangle.indices[k]).ok().and_then(|i| self.mesh.vertices.get(i)).copied());
        if verts.iter().any(Option::is_none) { return false; }
        let world: Vec<[f32;3]> = verts.iter_mut().map(|v| place(&self.to_world, v.unwrap_or_default())).collect();
        let mut plane = plane_of(world[2], world[1], world[0]);
        if self.flip { plane = plane.map(|v| -v); }
        let order: [usize;3] = if self.flip { [2,1,0] } else { [0,1,2] };
        let v: [[f32;3];3] = order.map(|i| world[i]);
        let mut min = v[0];
        let mut max = v[0];
        let mut bounds = None;
        for p in v { grow(&mut bounds, p); }
        if let Some([lo,hi]) = bounds { min = lo; max = hi; }
        if !box_overlaps_box(min, max, probe) || !box_plane(plane, probe) || !box_plane(plane.map(|x| -x), probe) { return false; }
        let n = [plane[0],plane[1],plane[2]].map(f64::from);
        for i in 0..3 {
            let a = v[i];
            let b = v[(i+1)%3];
            let edge: [f32;3] = std::array::from_fn(|k| (f64::from(b[k])-f64::from(a[k])) as f32);
            let [e0,e1,e2] = edge.map(f64::from);
            let reference = [(n[1]*e2-n[2]*e1) as f32,(n[2]*e0-n[0]*e2) as f32,(n[0]*e1-n[1]*e0) as f32];
            if (edge[1] != 0.0 || edge[2] != 0.0) && !edge_axis(a, edge, reference, [1.0,0.0,0.0], probe) { return false; }
            if (edge[0] != 0.0 || edge[2] != 0.0) && !edge_axis(a, edge, reference, [0.0,1.0,0.0], probe) { return false; }
            if (edge[0] != 0.0 || edge[1] != 0.0) && !edge_axis(a, edge, reference, [0.0,0.0,1.0], probe) { return false; }
        }
        true
    }
}
