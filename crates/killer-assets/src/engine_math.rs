use glam::Vec3;
use std::sync::OnceLock;

const ANGLES: usize = 16384;
const HALF_STEP: f64 = f64::from_bits(0x3f2921fb54442d18);

fn table() -> &'static [f32; ANGLES] {
    static TABLE: OnceLock<Box<[f32; ANGLES]>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = Box::new([0.0; ANGLES]);
        for (index, value) in table.iter_mut().enumerate() {
            *value = ((index + index) as f64 * HALF_STEP).sin() as f32;
        }
        table
    })
}

pub fn sin_tab(units: i32) -> f32 {
    table()[((units >> 2) & 0x3fff) as usize]
}

pub fn cos_tab(units: i32) -> f32 {
    table()[(((units + 0x4000) >> 2) & 0x3fff) as usize]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coords {
    pub origin: [f32; 3],
    pub x: [f32; 3],
    pub y: [f32; 3],
    pub z: [f32; 3],
}

impl Coords {
    pub const UNIT: Coords = Coords { origin: [0.0; 3], x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], z: [0.0, 0.0, 1.0] };

    pub fn to_array(self) -> [f32; 12] {
        [self.origin[0], self.origin[1], self.origin[2], self.x[0], self.x[1], self.x[2], self.y[0], self.y[1], self.y[2], self.z[0], self.z[1], self.z[2]]
    }
}

fn dot(vector: [f64; 3], axis: [f32; 3]) -> f32 {
    (vector[0] * f64::from(axis[0]) + vector[1] * f64::from(axis[1]) + vector[2] * f64::from(axis[2])) as f32
}

pub fn transform_vector_by(vector: [f32; 3], coords: &Coords) -> [f32; 3] {
    let vector = vector.map(f64::from);
    [dot(vector, coords.x), dot(vector, coords.y), dot(vector, coords.z)]
}

pub fn transform_point_by(point: [f32; 3], coords: &Coords) -> [f32; 3] {
    let moved = [f64::from(point[0]) - f64::from(coords.origin[0]), f64::from(point[1]) - f64::from(coords.origin[1]), f64::from(point[2]) - f64::from(coords.origin[2])];
    [dot(moved, coords.x), dot(moved, coords.y), dot(moved, coords.z)]
}

pub fn coords_mul_assign(coords: &mut Coords, other: &Coords) {
    let origin = transform_point_by(coords.origin, other);
    let x = transform_vector_by(coords.x, other);
    let y = transform_vector_by(coords.y, other);
    let z = transform_vector_by(coords.z, other);
    *coords = Coords { origin, x, y, z };
}

pub fn coords_mul_rotator(coords: &Coords, rotator: [i32; 3]) -> Coords {
    let mut out = *coords;
    let (sin, cos) = (sin_tab(rotator[1]), cos_tab(rotator[1]));
    coords_mul_assign(&mut out, &Coords { origin: [0.0; 3], x: [cos, sin, 0.0], y: [-sin, cos, 0.0], z: [0.0, 0.0, 1.0] });
    let (sin, cos) = (sin_tab(rotator[0]), cos_tab(rotator[0]));
    coords_mul_assign(&mut out, &Coords { origin: [0.0; 3], x: [cos, 0.0, sin], y: [0.0, 1.0, 0.0], z: [-sin, 0.0, cos] });
    let (sin, cos) = (sin_tab(rotator[2]), cos_tab(rotator[2]));
    coords_mul_assign(&mut out, &Coords { origin: [0.0; 3], x: [1.0, 0.0, 0.0], y: [0.0, cos, -sin], z: [0.0, sin, cos] });
    out
}

pub fn coords_mul(coords: &Coords, other: &Coords) -> Coords {
    let mut out = *coords;
    coords_mul_assign(&mut out, other);
    out
}

pub fn coords_div_rotator(coords: &Coords, rotator: [i32; 3]) -> Coords {
    let mut out = *coords;
    let (sin, cos) = (sin_tab(rotator[2]), cos_tab(rotator[2]));
    coords_mul_assign(&mut out, &Coords { origin: [0.0; 3], x: [1.0, 0.0, 0.0], y: [0.0, cos, sin], z: [0.0, -sin, cos] });
    let (sin, cos) = (sin_tab(rotator[0]), cos_tab(rotator[0]));
    coords_mul_assign(&mut out, &Coords { origin: [0.0; 3], x: [cos, 0.0, -sin], y: [0.0, 1.0, 0.0], z: [sin, 0.0, cos] });
    let (sin, cos) = (sin_tab(rotator[1]), cos_tab(rotator[1]));
    coords_mul_assign(&mut out, &Coords { origin: [0.0; 3], x: [cos, -sin, 0.0], y: [sin, cos, 0.0], z: [0.0, 0.0, 1.0] });
    out
}

const UNITS_PER_RADIAN: f64 = f64::from_bits(0x40c45f306dc9c883);

fn dot3(a: [f32; 3], b: [f32; 3]) -> f64 {
    f64::from(a[0]) * f64::from(b[0]) + f64::from(a[1]) * f64::from(b[1]) + f64::from(a[2]) * f64::from(b[2])
}

pub fn ortho_rotation(coords: &Coords) -> [i32; 3] {
    let x = coords.x;
    let yaw = ((f64::from(x[1]).atan2(f64::from(x[0]))) * UNITS_PER_RADIAN) as i32;
    let flat = (f64::from(x[0]) * f64::from(x[0]) + f64::from(x[1]) * f64::from(x[1])).sqrt();
    let pitch = ((f64::from(x[2]).atan2(flat)) * UNITS_PER_RADIAN) as i32;
    let inverse = coords_div_rotator(&Coords::UNIT, [pitch, yaw, 0]);
    let roll = (dot3(inverse.y, coords.z).atan2(dot3(inverse.y, coords.y)) * UNITS_PER_RADIAN) as i32;
    [pitch, yaw, roll]
}

pub fn rotator_forward(rotator: [i32; 3]) -> Vec3 {
    let (cos_pitch, sin_pitch) = (cos_tab(rotator[0]), sin_tab(rotator[0]));
    Vec3::new(cos_pitch * cos_tab(rotator[1]), cos_pitch * sin_tab(rotator[1]), sin_pitch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coords_and_transforms_equal_core_dll_on_random_rotators_and_vectors() {
        use crate::scene::transform::vector_rotation;
        let cases: serde_json::Value = serde_json::from_str(include_str!("../tests/data/engine_math_oracle.json")).expect("fixture");
        let floats = |value: &serde_json::Value| -> Vec<f32> { value.as_array().expect("lista").iter().map(|v| v.as_f64().expect("número") as f32).collect() };
        for (index, case) in cases.as_array().expect("casos").iter().enumerate() {
            let rotator: Vec<i32> = case["rotator"].as_array().expect("rotator").iter().map(|v| v.as_i64().expect("inteiro") as i32).collect();
            let vector = floats(&case["vector"]);
            let coords = coords_mul_rotator(&Coords::UNIT, [rotator[0], rotator[1], rotator[2]]);
            let expected = floats(&case["coords"]);
            for (slot, (got, want)) in coords.to_array().iter().zip(&expected).enumerate() {
                assert_eq!(got.to_bits(), want.to_bits(), "caso {index}: coords[{slot}] {got} != {want}");
            }
            let transformed = transform_vector_by([vector[0], vector[1], vector[2]], &coords);
            let want = floats(&case["transformed"]);
            for axis in 0..3 {
                assert_eq!(transformed[axis].to_bits(), want[axis].to_bits(), "caso {index}: vetor[{axis}] {} != {}", transformed[axis], want[axis]);
            }
            let rotation: Vec<i32> = case["rotation"].as_array().expect("rotação").iter().map(|v| v.as_i64().expect("inteiro") as i32).collect();
            assert_eq!(vector_rotation(glam::Vec3::new(vector[0], vector[1], vector[2])).to_vec(), rotation, "caso {index}: Rotation");
        }
    }

    #[test]
    fn coords_division_product_and_ortho_rotation_equal_core_dll() {
        let cases: serde_json::Value = serde_json::from_str(include_str!("../tests/data/coords_ops_oracle.json")).expect("fixture");
        let floats = |value: &serde_json::Value| -> Vec<f32> { value.as_array().expect("lista").iter().map(|v| v.as_f64().expect("número") as f32).collect() };
        let ints = |value: &serde_json::Value| -> [i32; 3] {
            let list: Vec<i32> = value.as_array().expect("lista").iter().map(|v| v.as_i64().expect("inteiro") as i32).collect();
            [list[0], list[1], list[2]]
        };
        let coords = |values: &[f32]| Coords { origin: [values[0], values[1], values[2]], x: [values[3], values[4], values[5]], y: [values[6], values[7], values[8]], z: [values[9], values[10], values[11]] };
        let mut worst = 0u32;
        for (index, case) in cases.as_array().expect("casos").iter().enumerate() {
            let a = coords(&floats(&case["a"]));
            let b = coords(&floats(&case["b"]));
            assert_eq!(coords_mul_rotator(&Coords::UNIT, ints(&case["first"])), a, "caso {index}: a");
            let third = ints(&case["third"]);
            let compare = |got: Coords, want: &serde_json::Value, label: &str| {
                for (slot, (got, want)) in got.to_array().iter().zip(floats(want)).enumerate() {
                    assert_eq!(got.to_bits(), want.to_bits(), "caso {index}: {label}[{slot}] {got} != {want}");
                }
            };
            compare(coords_div_rotator(&a, third), &case["divided"], "divisão");
            compare(coords_div_rotator(&Coords::UNIT, third), &case["unit_divided"], "divisão da unidade");
            let product = coords_mul(&a, &b);
            compare(product, &case["multiplied"], "produto");
            let want = ints(&case["ortho"]);
            let got = ortho_rotation(&coords(&floats(&case["multiplied"])));
            for axis in 0..3 {
                worst = worst.max((got[axis] - want[axis]).unsigned_abs());
            }
            assert_eq!(got, want, "caso {index}: OrthoRotation");
        }
        assert_eq!(worst, 0);
    }

    #[test]
    fn trig_table_equals_the_one_core_dll_builds_in_fglobalmath() {
        let bytes = include_bytes!("../tests/data/gmath_trig.bin");
        assert_eq!(bytes.len(), ANGLES * 4);
        for (index, chunk) in bytes.as_chunks::<4>().0.iter().enumerate() {
            let expected = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            assert_eq!(table()[index].to_bits(), expected.to_bits(), "entrada {index}");
        }
    }
}
