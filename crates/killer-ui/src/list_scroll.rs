use crate::draw::DrawList;
use crate::scroll::{DOWN_BTN, DOWN_ON_BTN, SliderBar, UP_BTN, UP_ON_BTN};
use crate::window::Rect;

pub const COLUMN_X: i32 = 5;
pub const BUTTON: i32 = 15;
pub const TRACK_Y: i32 = 20;
pub const TRACK_INSET: i32 = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListScrollHit {
    Up,
    Down,
    Track,
}

pub fn up_rect() -> Rect {
    Rect::new(COLUMN_X, COLUMN_X, BUTTON, BUTTON)
}

pub fn down_rect(list_height: i32) -> Rect {
    Rect::new(COLUMN_X, list_height - BUTTON, BUTTON, BUTTON)
}

pub fn track_rect(list_height: i32) -> Rect {
    Rect::new(COLUMN_X, TRACK_Y, BUTTON, (list_height - TRACK_INSET).max(0))
}

pub fn hit(list_height: i32, x: i32, y: i32) -> Option<ListScrollHit> {
    if up_rect().contains(x, y) {
        Some(ListScrollHit::Up)
    } else if down_rect(list_height).contains(x, y) {
        Some(ListScrollHit::Down)
    } else if track_rect(list_height).contains(x, y) {
        Some(ListScrollHit::Track)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ListScrollState {
    pub total: usize,
    pub visible: usize,
    pub offset: usize,
    pub pressed: Option<ListScrollHit>,
}

pub fn paint(list: &mut DrawList, x: i32, y: i32, list_height: i32, state: ListScrollState) {
    let ListScrollState { total, visible, offset, pressed } = state;
    let up = up_rect();
    let down = down_rect(list_height);
    let up_texture = if pressed == Some(ListScrollHit::Up) { UP_ON_BTN } else { UP_BTN };
    let down_texture = if pressed == Some(ListScrollHit::Down) { DOWN_ON_BTN } else { DOWN_BTN };
    list.draw_texture(x + up.x, y + up.y, BUTTON, BUTTON, 0, 0, BUTTON, BUTTON, up_texture, 1.0);
    list.draw_texture(x + down.x, y + down.y, BUTTON, BUTTON, 0, 0, BUTTON, BUTTON, down_texture, 1.0);
    let track = track_rect(list_height);
    slider_for(list_height, total, visible, offset).paint(list, x + track.x, y + track.y);
}

fn slider_for(list_height: i32, total: usize, visible: usize, offset: usize) -> SliderBar {
    let mut slider = SliderBar::new(track_rect(list_height).h);
    slider.set_range(total as i32, visible as i32);
    let hidden = total.saturating_sub(visible);
    slider.set_pos(hidden.saturating_sub(offset.min(hidden)) as i32);
    slider
}

pub fn press_thumb(list_height: i32, total: usize, visible: usize, offset: usize, track_y: i32) -> Option<i32> {
    let mut slider = slider_for(list_height, total, visible, offset);
    slider.mouse_down(track_y);
    slider.dragging.then_some(slider.grab)
}

pub fn drag(list_height: i32, total: usize, visible: usize, offset: usize, grab: i32, track_y: i32) -> Option<usize> {
    let mut slider = slider_for(list_height, total, visible, offset);
    slider.dragging = true;
    slider.grab = grab;
    let position = slider.mouse_move(track_y)?;
    let hidden = total.saturating_sub(visible);
    Some(hidden.saturating_sub((position.max(0) as usize).min(hidden)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_column_has_the_buttons_at_the_ends_and_the_track_between_them() {
        assert_eq!(hit(141, 10, 10), Some(ListScrollHit::Up));
        assert_eq!(hit(141, 10, 130), Some(ListScrollHit::Down));
        assert_eq!(hit(141, 10, 60), Some(ListScrollHit::Track));
        assert_eq!(hit(141, 40, 60), None);
    }

    #[test]
    fn dragging_the_thumb_moves_the_offset_and_a_press_outside_it_does_not_grab() {
        let (height, total, visible) = (141, 40, 8);
        assert_eq!(press_thumb(height, total, visible, 0, 2), None);
        let thumb = {
            let slider = slider_for(height, total, visible, 0);
            (slider.thumb_y, slider.thumb)
        };
        let grab = press_thumb(height, total, visible, 0, thumb.0 + 3).unwrap();
        assert_eq!(grab, 3);
        let higher = drag(height, total, visible, 0, grab, thumb.0 - 30).unwrap();
        assert!(higher > 0);
        assert_eq!(drag(height, total, visible, 0, grab, 0), Some(32));
        assert_eq!(drag(height, total, visible, 32, grab, 0), None);
    }

    #[test]
    fn the_thumb_sits_at_the_bottom_when_the_list_shows_the_newest_lines_and_rises_with_the_offset() {
        let thumb_y = |offset: usize| {
            let mut list = DrawList::default();
            paint(&mut list, 0, 0, 141, ListScrollState { total: 40, visible: 8, offset, pressed: None });
            list.quads.iter().find(|quad| quad.texture == crate::scroll::SLIDER_TOP).map(|quad| quad.y)
        };
        assert!(thumb_y(0) > thumb_y(10));
        assert_eq!(thumb_y(32), Some(TRACK_Y as f32));
        let mut list = DrawList::default();
        paint(&mut list, 0, 0, 141, ListScrollState { total: 5, visible: 8, offset: 0, pressed: None });
        assert!(!list.quads.iter().any(|quad| quad.texture == crate::scroll::SLIDER_TOP));
    }
}
