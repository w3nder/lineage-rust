use super::model::Model;

fn less(a: f64, b: f64) -> bool {
    a.partial_cmp(&b) == Some(std::cmp::Ordering::Less)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointHit {
    pub location: [f32; 3],
    pub normal: [f32; 3],
    pub item: i32,
}

struct Probe {
    location: [f32; 3],
    extent: [f32; 3],
    flags: u32,
    best: f32,
    hit: PointHit,
}

impl Probe {
    fn clip(&mut self, plane: [f32; 4], item: i32) -> bool {
        let [nx,ny,nz,w] = plane.map(f64::from);
        let [ex,ey,ez] = self.extent.map(f64::from);
        let [lx,ly,lz] = self.location.map(f64::from);
        let a = f64::from((ex*nx) as f32).abs();
        let b = f64::from((ey*ny) as f32).abs();
        let c = f64::from((ez*nz) as f32).abs();
        let reach = (c+b)+a;
        let dist = ((ly*ny+lz*nz)+nx*lx)-w;
        if !less(0.0, dist) || !less(dist, f64::from(self.best)) { return less(dist, reach); }
        if !less(dist, reach) { return false; }
        self.best = dist as f32;
        let scale = f64::from(1.02_f32);
        let lifted = (nz*scale) as f32;
        let depth = f64::from((reach-dist) as f32);
        let x = (f64::from(((nx*scale)*depth) as f32)+lx) as f32;
        let y = ((ny*scale)*depth+ly) as f32;
        let z = (f64::from(lifted)*depth+lz) as f32;
        self.hit = PointHit { location: [x,y,z], normal: [plane[0],plane[1],plane[2]], item };
        true
    }
}

impl Model<'_> {
    pub fn point_check(&self, location: [f32; 3], extent: [f32; 3], flags: u32) -> (bool, PointHit) {
        let hit = PointHit { location, normal: [0.0;3], item: 0 };
        if self.tree.nodes.is_empty() { return (self.data.root_outside, hit); }
        if extent == [0.0;3] { return self.classify(location, hit); }
        let mut probe = Probe { location, extent, flags, best: 100000.0, hit };
        let fits = self.filter(&mut probe, 0, 0, self.data.root_outside);
        (fits, probe.hit)
    }

    fn classify(&self, location: [f32; 3], mut hit: PointHit) -> (bool, PointHit) {
        let [lx,ly,lz] = location.map(f64::from);
        let mut outside = self.data.root_outside;
        let mut index = 0_i32;
        let mut last = 0_usize;
        let mut side = 0_i32;
        while let Ok(current) = usize::try_from(index) {
            let Some(node) = self.tree.nodes.get(current) else { break; };
            last = current;
            let [px,py,pz,pw] = self.plane(node.plane).map(f64::from);
            let solid = node.vertices != 0 && node.flags & 0x21 == 0;
            if ((ly*py+lz*pz)+lx*px)-pw <= 0.0 {
                side = 0;
                outside = outside && !solid;
                index = node.back;
            } else {
                side = 1;
                outside = outside || solid;
                index = node.front;
            }
        }
        hit.item = side+last as i32*2;
        (outside, hit)
    }

    fn filter(&self, probe: &mut Probe, mut parent: usize, mut index: i32, mut outside: bool) -> bool {
        let mut fits = true;
        let mask = (probe.flags as u8)|0x21;
        let [lx,ly,lz] = probe.location.map(f64::from);
        let [ex,ey,ez] = probe.extent.map(|v| f64::from(v)*f64::from(1.1_f32));
        while let Ok(current) = usize::try_from(index) {
            let Some(node) = self.tree.nodes.get(current) else { return fits; };
            parent = current;
            let [px,py,pz,pw] = self.plane(node.plane).map(f64::from);
            let a = f64::from((px*ex) as f32).abs();
            let b = f64::from((py*ey) as f32).abs();
            let c = f64::from((pz*ez) as f32).abs();
            let reach = f64::from(((c+b)+a) as f32);
            let dist = f64::from((((pz*lz+py*ly)+px*lx)-pw) as f32);
            let solid = node.vertices != 0 && node.flags & mask == 0;
            if -reach < dist && !self.filter(probe, current, node.front, outside || solid) { fits = false; }
            let behind = outside && !solid;
            if reach < dist { return fits; }
            outside = behind;
            index = node.back;
        }
        if outside { return fits; }
        let Some(node) = self.tree.nodes.get(parent) else { return fits; };
        if node.collision_bound == -1 { return fits; }
        let Some(leaf) = self.leaf(node) else { return fits; };
        if !self.each_plane(&leaf, &mut |item, plane| probe.clip(plane, item)) { return fits; }
        false
    }
}
