use glam::{Mat4, Vec3, Vec4};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn benchmark_math(iterations: u32) -> Vec<f32> {
    let transforms:Vec<_>=(0..64).map(|index| {
        Mat4::from_translation(Vec3::new(index as f32,2.0,-3.0)) * Mat4::from_rotation_z(index as f32*0.01)
    }).collect();
    let mut result=Vec4::ZERO;
    for index in 0..iterations.min(2_000_000) {
        let a=std::hint::black_box(transforms[index as usize%64]);
        let b=std::hint::black_box(transforms[(index as usize+7)%64]);
        result+=(a*b)*Vec4::new(1.0,2.0,3.0,1.0);
    }
    result.to_array().to_vec()
}

#[wasm_bindgen]
pub fn math_simd() -> bool {cfg!(target_feature="simd128")}

#[wasm_bindgen]
pub fn check_math() -> Result<u32,JsValue> {
    let mut checked=0;
    for a in 0..64 {
        for b in 0..64 {
            let left=Mat4::from_rotation_x(a as f32*0.07)*Mat4::from_translation(Vec3::new(a as f32,2.0,-3.0));
            let right=Mat4::from_rotation_z(b as f32*0.03)*Mat4::from_scale(Vec3::new(1.5,0.7,2.0));
            let actual=(left*right).to_cols_array_2d();
            let left=left.to_cols_array_2d();
            let right=right.to_cols_array_2d();
            for column in 0..4 {
                for row in 0..4 {
                    let expected=(0..4).map(|k|f64::from(left[k][row])*f64::from(right[column][k])).sum::<f64>();
                    if (f64::from(actual[column][row])-expected).abs()>expected.abs().max(1.0)*1e-5 {
                        return Err(JsValue::from_str("produto de matrizes divergiu da referência escalar f64"));
                    }
                    checked+=1;
                }
            }
        }
    }
    Ok(checked)
}
