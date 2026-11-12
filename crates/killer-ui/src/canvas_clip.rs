#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasClip {
    pub flags: u32,
    pub origin: [f32; 2],
    pub size: [f32; 2],
    pub cursor: [f32; 2],
    pub saved_cursor: [f32; 2],
    pub saved_origin: [f32; 2],
    pub saved_size: [f32; 2],
}

impl CanvasClip {
    pub fn set_clip(&mut self, rect: [i32; 4]) {
        let [x, y, width, height] = rect.map(|value| value as f32);
        self.flags |= 8;
        self.origin = [x, y];
        self.size = [width, height];
        self.cursor = [0.0; 2];
        self.saved_cursor = [0.0; 2];
        self.saved_origin = self.origin;
        self.saved_size = self.size;
    }

    pub fn release_clip(&mut self) {
        if self.flags & 8 == 0 { return; }
        self.flags &= !8;
        self.cursor = self.saved_cursor;
        self.origin = self.saved_origin;
        self.size = self.saved_size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_replaces_saved_state_and_release_only_copies_when_active() {
        let mut state = CanvasClip { flags: 0x80000000, origin: [1.0; 2], size: [2.0; 2], cursor: [3.0; 2], saved_cursor: [4.0; 2], saved_origin: [5.0; 2], saved_size: [6.0; 2] };
        let before = state;
        state.release_clip(); assert_eq!(state, before);
        state.set_clip([i32::MAX, i32::MIN, -16777217, 16777217]);
        assert_eq!(state.flags, 0x80000008);
        assert_eq!(state.saved_origin, [i32::MAX as f32, i32::MIN as f32]);
        assert_eq!(state.saved_size, [-16777216.0, 16777216.0]);
        assert_eq!(state.saved_cursor, [0.0; 2]);
        state.set_clip([10, 20, 30, 40]);
        state.origin = [-1.0; 2]; state.cursor = [-2.0; 2]; state.size = [-3.0; 2];
        state.release_clip();
        assert_eq!(state.flags, 0x80000000);
        assert_eq!(state.origin, [10.0, 20.0]); assert_eq!(state.size, [30.0, 40.0]); assert_eq!(state.cursor, [0.0; 2]);
    }
}
