pub mod window;

pub use window::{Direction, LoadRequest, TileId, TileWindow};

pub const TILE_SIZE: f32 = 32768.0;

pub fn tile_name(x: f32, y: f32) -> String {
    let tx = (x / TILE_SIZE).floor() as i32 + 20;
    let ty = (y / TILE_SIZE).floor() as i32 + 18;
    format!("{tx}_{ty}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn giran_is_22_22() {
        assert_eq!(tile_name(83400.0, 148000.0), "22_22");
        assert_eq!(tile_name(65536.0, 131072.0), "22_22");
        assert_eq!(tile_name(65535.0, 131071.0), "21_21");
        assert_eq!(tile_name(-1.0, -1.0), "19_17");
    }
}
