use glam::Vec3;
use rustc_hash::FxHashMap;

#[derive(Default)]
pub struct SunVisibility {
    entries: FxHashMap<i32, Entry>,
}

struct Entry {
    time: f32,
    on: bool,
    previous: bool,
    seen: f32,
}

impl SunVisibility {
    pub fn update(&mut self, now: f32, viewer: Vec3, direction: Vec3, positions: impl IntoIterator<Item = (i32, Vec3)>, mut clear: impl FnMut(Vec3, Vec3) -> bool) {
        for (key, position) in positions {
            let entry = self.entries.entry(key).or_insert(Entry { time: -0.35, on: false, previous: false, seen: now });
            entry.seen = now;
            if now - entry.time >= 0.35 {
                entry.previous = entry.on;
                entry.on = position.truncate().distance(viewer.truncate()) > 512.0 || clear(position, position - direction * 65536.0);
                entry.time = now - if entry.on { 0.35 } else { 0.0 };
            }
        }
        self.entries.retain(|_, entry| now - entry.seen <= 0.35);
    }

    pub fn visible(&self, actor_id: i32) -> bool {
        self.entries.get(&actor_id).is_none_or(|entry| entry.on || entry.previous)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fun_10648fe0_retains_previous_sun_for_035_seconds() {
        let mut state = SunVisibility::default();
        state.update(1.0, Vec3::ZERO, -Vec3::Z, [(1, Vec3::ZERO)], |a, b| { assert_eq!(b - a, Vec3::Z * 65536.0); true });
        assert!(state.visible(1));
        state.update(1.1, Vec3::ZERO, -Vec3::Z, [(1, Vec3::ZERO)], |_, _| false);
        assert!(state.visible(1));
        state.update(1.46, Vec3::ZERO, -Vec3::Z, [(1, Vec3::ZERO)], |_, _| false);
        assert!(!state.visible(1));
    }
}
