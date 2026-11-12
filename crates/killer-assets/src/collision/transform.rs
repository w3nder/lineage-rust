use glam::Mat4;
use std::sync::OnceLock;
use ue2_level::Placement;

pub fn actor(placement: Placement,scale: [f32;3],pivot: [f32;3]) -> Mat4 {
    static SINE: OnceLock<[f32;16384]> = OnceLock::new();
    let table = SINE.get_or_init(crate::skeletal_lod::sine_table);
    let sin = |angle: i32| table[((angle>>2)&0x3fff) as usize] as f64;
    let [pitch,yaw,roll] = placement.rotation;
    let (sp,cp) = (sin(pitch),sin(pitch.wrapping_add(0x4000)));
    let (sy,cy) = (sin(yaw),sin(yaw.wrapping_add(0x4000)));
    let (sr,cr) = (sin(roll),sin(roll.wrapping_add(0x4000)));
    let [sx,sz_y,sz] = scale.map(f64::from);
    let cr_sy = (cr*sy) as f32 as f64;
    let cy_cr = (cy*cr) as f32 as f64;
    let x = [(sx*cy*cp) as f32,(sx*cp*sy) as f32,(sx*sp) as f32,0.0];
    let y = [((cy*sp*sr-cr_sy)*sz_y) as f32,((sy*sp*sr+cy_cr)*sz_y) as f32,-(sz_y*cp*sr) as f32,0.0];
    let z = [-((cy_cr*sp+sy*sr)*sz) as f32,((cy*sr-cr_sy*sp)*sz) as f32,(sz*cp*cr) as f32,0.0];
    let px = (sx*pivot[0] as f64) as f32 as f64;
    let pz = (sz*pivot[2] as f64) as f32 as f64;
    let py_wide = sz_y*pivot[1] as f64;
    let py = py_wide as f32 as f64;
    let combined = (pz*sr+py_wide*cr) as f32 as f64;
    let [lx,ly,lz] = placement.location.map(f64::from);
    let translation = [(((lx-px*cy*cp)+pz*cy*cr*sp)-py*cy*sp*sr+combined*sy) as f32,
        (ly-(((px*cp*sy+combined*cy)-pz*cr*sy*sp)+py*sy*sp*sr)) as f32,
        (lz-((pz*cp*cr+px*sp)-py*cp*sr)) as f32,1.0];
    Mat4::from_cols_array_2d(&[x,y,z,translation])
}

pub type Rows = [[f32;4];4];

pub fn world_to_local(placement: Placement,draw_scale: f32,draw_scale_3d: [f32;3],pivot: [f32;3]) -> Mat4 {
    let reciprocal = draw_scale_3d.map(|axis| (1.0/f64::from(axis)) as f32);
    let inverse = 1.0/f64::from(draw_scale);
    let rounded = f64::from(inverse as f32);
    let scale = [(rounded*f64::from(reciprocal[0])) as f32,(rounded*f64::from(reciprocal[1])) as f32,(inverse*f64::from(reciprocal[2])) as f32];
    let [lx,ly,lz] = placement.location;
    let rows = multiply(multiply(multiply(translation([-lx,-ly,-lz]),inverse_rotation(placement.rotation)),diagonal(scale)),translation(pivot));
    Mat4::from_cols_array_2d(&rows)
}

pub fn inverse_rotation(rotation: [i32;3]) -> Rows {
    static SINE: OnceLock<[f32;16384]> = OnceLock::new();
    let table = SINE.get_or_init(crate::skeletal_lod::sine_table);
    let pair = |angle: i32| (table[((0x4000_i32.wrapping_sub(angle)>>2)&0x3fff) as usize],table[((angle.wrapping_neg()>>2)&0x3fff) as usize]);
    let [pitch,yaw,roll] = rotation;
    let (cy,sy) = pair(yaw);
    let (cp,sp) = pair(pitch);
    let (cr,sr) = pair(roll);
    let yaw = [[cy,sy,0.0,0.0],[-sy,cy,0.0,0.0],[0.0,0.0,1.0,0.0],[0.0,0.0,0.0,1.0]];
    let pitch = [[cp,0.0,sp,0.0],[0.0,1.0,0.0,0.0],[-sp,0.0,cp,0.0],[0.0,0.0,0.0,1.0]];
    let roll = [[1.0,0.0,0.0,0.0],[0.0,cr,-sr,0.0],[0.0,sr,cr,0.0],[0.0,0.0,0.0,1.0]];
    multiply(multiply(yaw,pitch),roll)
}

pub fn translation(v: [f32;3]) -> Rows {
    [[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[v[0],v[1],v[2],1.0]]
}

pub fn diagonal(v: [f32;3]) -> Rows {
    [[v[0],0.0,0.0,0.0],[0.0,v[1],0.0,0.0],[0.0,0.0,v[2],0.0],[0.0,0.0,0.0,1.0]]
}

pub fn multiply(a: Rows,b: Rows) -> Rows {
    std::array::from_fn(|i| std::array::from_fn(|j| {
        let order = if i == 0 { [1,0,3,2] } else { [3,2,0,1] };
        order.iter().fold(None,|sum: Option<f64>,&k| {
            let term = f64::from(b[k][j])*f64::from(a[i][k]);
            Some(sum.map_or(term,|sum| sum+term))
        }).unwrap_or(0.0) as f32
    }))
}

pub fn rows(matrix: Mat4) -> Rows {
    matrix.to_cols_array_2d()
}

pub fn transform_fplane(m: Rows, p: [f32;4]) -> [f32;4] {
    std::array::from_fn(|j| (((f64::from(p[0])*f64::from(m[0][j])+f64::from(p[1])*f64::from(m[1][j]))
        +f64::from(p[2])*f64::from(m[2][j]))+f64::from(p[3])*f64::from(m[3][j])) as f32)
}

pub fn transform_point(m: Rows, p: [f32;3]) -> [f32;3] {
    let out = transform_fplane(m,[p[0],p[1],p[2],1.0]);
    [out[0],out[1],out[2]]
}

pub fn determinant(m: Rows) -> f64 {
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

pub fn transpose_adjoint(m: Rows) -> Rows {
    let m = m.map(|row| row.map(f64::from));
    let cofactor = |a: f64,b: f64,c: f64,d: f64| (a*b-c*d) as f32;
    [[cofactor(m[1][1],m[2][2],m[2][1],m[1][2]),cofactor(m[2][0],m[1][2],m[1][0],m[2][2]),cofactor(m[2][1],m[1][0],m[2][0],m[1][1]),0.0],
     [cofactor(m[2][1],m[0][2],m[0][1],m[2][2]),cofactor(m[0][0],m[2][2],m[2][0],m[0][2]),cofactor(m[2][0],m[0][1],m[2][1],m[0][0]),0.0],
     [cofactor(m[0][1],m[1][2],m[0][2],m[1][1]),cofactor(m[1][0],m[0][2],m[0][0],m[1][2]),cofactor(m[1][1],m[0][0],m[1][0],m[0][1]),0.0],
     [0.0,1.0,0.0,0.0]]
}

pub fn safe_normal(v: [f32;3]) -> [f32;3] {
    let [x,y,z] = v.map(f64::from);
    let square = (x*x+y*y)+z*z;
    if square < 1e-8 { return [0.0;3]; }
    let inverse = 1.0/square.sqrt();
    let rounded = f64::from(inverse as f32);
    [(rounded*x) as f32,(rounded*y) as f32,(inverse*z) as f32]
}

pub fn transform_by_using_adjoint_t(plane: [f32;4], m: Rows, adjoint: Rows) -> [f32;4] {
    let turned = transform_fplane(adjoint,[plane[0],plane[1],plane[2],0.0]);
    let mut normal = safe_normal([turned[0],turned[1],turned[2]]);
    if determinant(m) < 0.0 { normal = normal.map(|value| -value); }
    let w = f64::from(plane[3]);
    let point = transform_fplane(m,[(f64::from(plane[0])*w) as f32,(f64::from(plane[1])*w) as f32,(f64::from(plane[2])*w) as f32,1.0]);
    let [nx,ny,nz] = normal.map(f64::from);
    [normal[0],normal[1],normal[2],((f64::from(point[2])*nz+f64::from(point[1])*ny)+f64::from(point[0])*nx) as f32]
}
