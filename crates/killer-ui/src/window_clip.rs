use crate::positioning_rect::PositioningRect;
use crate::window_restore::float_size_to_int;

pub fn clip_to_parent(
    window: &mut PositioningRect,
    offsets: &mut (i32, i32),
    output: &mut PositioningRect,
    parent: Option<PositioningRect>,
) -> bool {
    let width = float_size_to_int(window.width);
    let height = float_size_to_int(window.height);
    let Some(parent) = parent else { return false };
    let parent_width = float_size_to_int(parent.width);
    let parent_height = float_size_to_int(parent.height);
    let right = window.x.wrapping_add(width);
    let bottom = window.y.wrapping_add(height);
    let parent_right = parent.x.wrapping_add(parent_width);
    let parent_bottom = parent.y.wrapping_add(parent_height);
    if window.x > parent_right || right < parent.x || window.y > parent_bottom || bottom < parent.y {
        return true;
    }
    let mut x = window.x;
    let mut y = window.y;
    let mut clipped_width = width;
    let mut clipped_height = height;
    if x < parent.x {
        output.x = parent.x.wrapping_sub(x);
        offsets.0 = output.x;
        clipped_width = clipped_width.wrapping_sub(output.x);
        x = parent.x;
    } else {
        offsets.0 = 0;
    }
    if right > parent_right {
        clipped_width = clipped_width.wrapping_add(parent_width.wrapping_sub(width).wrapping_sub(window.x).wrapping_add(parent.x));
    }
    output.width = clipped_width as f32;
    if y < parent.y {
        output.y = parent.y.wrapping_sub(y);
        offsets.1 = output.y;
        clipped_height = clipped_height.wrapping_sub(output.y);
        y = parent.y;
    } else {
        offsets.1 = 0;
    }
    if bottom > parent_bottom {
        clipped_height = clipped_height.wrapping_add(parent_height.wrapping_sub(height).wrapping_sub(window.y).wrapping_add(parent.y));
    }
    output.height = clipped_height as f32;
    if clipped_width == 0 && clipped_height == 0 {
        *output = PositioningRect { x: 0, y: 0, width: window.width, height: window.height };
    } else {
        *window = PositioningRect { x, y, width: output.width, height: output.height };
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touching_corner_keeps_geometry_but_updates_offsets() {
        let mut window = PositioningRect { x: -10, y: -20, width: 10.5, height: 20.5 };
        let original = window;
        let mut output = PositioningRect { x: 7, y: 9, width: 0.0, height: 0.0 };
        let mut offsets = (3, 4);
        let parent = PositioningRect { x: 0, y: 0, width: 100.0, height: 100.0 };
        assert!(!clip_to_parent(&mut window, &mut offsets, &mut output, Some(parent)));
        assert_eq!(window, original);
        assert_eq!(offsets, (10, 20));
        assert_eq!(output, PositioningRect { x: 0, y: 0, width: 10.5, height: 20.5 });
    }

    #[test]
    fn containment_truncates_dimensions_without_overwriting_output_origin() {
        let mut window = PositioningRect { x: 10, y: 20, width: 30.75, height: 40.5 };
        let mut output = PositioningRect { x: 7, y: 9, width: 0.0, height: 0.0 };
        let mut offsets = (3, 4);
        let parent = PositioningRect { x: 0, y: 0, width: 100.0, height: 100.0 };
        assert!(!clip_to_parent(&mut window, &mut offsets, &mut output, Some(parent)));
        assert_eq!(window, PositioningRect { x: 10, y: 20, width: 30.0, height: 40.0 });
        assert_eq!(output, PositioningRect { x: 7, y: 9, width: 30.0, height: 40.0 });
        assert_eq!(offsets, (0, 0));
    }
}
