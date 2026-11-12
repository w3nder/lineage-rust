use super::model::Hit;
use glam::Vec3;

pub fn trace(center: [f32;3],radius: f32,height: f32,start: [f32;3],end: [f32;3],extent: [f32;3]) -> Option<Hit> {
    let center = Vec3::from(center);
    let start = Vec3::from(start);
    let end = Vec3::from(end);
    let extent = Vec3::from(extent)+Vec3::new(radius,radius,height);
    if (0..3).any(|axis| (start[axis] > center[axis]+extent[axis] && end[axis] > center[axis]+extent[axis])
        || (start[axis] < center[axis]-extent[axis] && end[axis] < center[axis]-extent[axis])) { return None; }
    let delta = end-start;
    let top = center.z+extent.z;
    let bottom = center.z-extent.z;
    let mut enter = 0.0_f32;
    let mut exit = 1.0_f32;
    let mut normal = Vec3::ZERO;
    let at_z = |z: f32| ((z as f64-start.z as f64)/(end.z as f64-start.z as f64)) as f32;
    if start.z > top && end.z < top {
        let time = at_z(top);
        if time > enter { enter = time; normal = Vec3::Z; }
    } else if start.z < top && end.z > top { exit = exit.min(at_z(top)); }
    if start.z < bottom && end.z > bottom {
        let time = at_z(bottom);
        if time > enter { enter = time; normal = Vec3::new(0.0,0.0,-1.0); }
    } else if start.z > bottom && end.z < bottom { exit = exit.min(at_z(bottom)); }
    if enter >= exit { return None; }
    let offset = start-center;
    let a = (delta.x as f64*delta.x as f64+delta.y as f64*delta.y as f64) as f32;
    let b = (delta.x as f64*offset.x as f64+delta.y as f64*offset.y as f64) as f32;
    let b = b+b;
    let c = (offset.x as f64*offset.x as f64+offset.y as f64*offset.y as f64-extent.x as f64*extent.x as f64) as f32;
    let discriminant = (b as f64*b as f64-c as f64*a as f64*4.0) as f32;
    if c < 1.0 && start.z > bottom && start.z < top {
        let inward = delta.x as f64*offset.x as f64+delta.y as f64*offset.y as f64;
        return (inward < -0.1_f32 as f64).then(|| Hit { time:0.0,position:start.to_array(),normal:normalize(Vec3::new(offset.x,offset.y,0.0)).to_array() });
    }
    if discriminant < 0.0 { return None; }
    if a >= f32::from_bits(0x322b_cc76) {
        let root = (discriminant as f64).sqrt() as f32;
        let scale = (0.5/a as f64) as f32 as f64;
        exit = exit.min(((root as f64-b as f64)*scale) as f32);
        let time = (-((root as f64+b as f64)*scale)) as f32;
        if enter < time {
            normal = normalize(Vec3::new((delta.x as f64*time as f64+start.x as f64) as f32-center.x,
                (delta.y as f64*time as f64+start.y as f64) as f32-center.y,0.0));
            enter = time;
        }
        if enter >= exit { return None; }
    } else if c > 0.0 { return None; }
    let time = (enter-0.001).clamp(0.0,1.0);
    let position = [(delta.x as f64*time as f64+start.x as f64) as f32,(delta.y as f64*time as f64+start.y as f64) as f32,delta.z*time+start.z];
    Some(Hit { time,position,normal:normal.to_array() })
}

fn normalize(v: Vec3) -> Vec3 {
    let [x,y,z] = v.to_array().map(f64::from);
    let squared = x*x+y*y+z*z;
    if squared < 1e-8 { return Vec3::ZERO; }
    let scale = 1.0/squared.sqrt();
    Vec3::new((x*scale) as f32,(y*scale) as f32,(z*scale) as f32)
}

pub fn point_check(center: [f32;3], radius: f32, height: f32, location: [f32;3], extent: [f32;3], hit: &mut super::point_check::PointHit) -> bool {
    let less_equal = |a: f64, b: f64| matches!(a.partial_cmp(&b), Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal));
    let vertical = f64::from(center[2])-f64::from(location[2]);
    let reach = f64::from(extent[2])+f64::from(height);
    if !(reach*reach).partial_cmp(&(vertical*vertical)).is_some_and(|order| order.is_gt()) { return false; }
    let across = [f64::from(center[0])-f64::from(location[0]),f64::from(center[1])-f64::from(location[1])];
    let wide = f64::from(extent[0])+f64::from(radius);
    if less_equal(wide*wide, across[1]*across[1]+across[0]*across[0]) { return false; }
    let offset: [f32;3] = std::array::from_fn(|k| (f64::from(location[k])-f64::from(center[k])) as f32);
    let normal = super::static_mesh::safe_normal_core(offset);
    hit.normal = normal;
    if normal[2] < -0.5 || 0.5 < normal[2] {
        hit.location = [location[0],location[1],(f64::from(center[2])-f64::from(extent[2])) as f32];
        return true;
    }
    let side = super::static_mesh::safe_normal_core([normal[0],normal[1],(f64::from(normal[2])*0.0) as f32]);
    let push = side.map(|v| (f64::from(extent[0])*f64::from(v)) as f32);
    hit.location = [(f64::from(center[0])-f64::from(push[0])) as f32,(f64::from(center[1])-f64::from(push[1])) as f32,
        (f64::from((f64::from(center[2])-f64::from(push[2])) as f32)+f64::from(location[2])) as f32];
    true
}
