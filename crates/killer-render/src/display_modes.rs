#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub bits: i32,
    pub refresh: i32,
}

pub fn resolutions(modes: &[DisplayMode], capacity: usize) -> Vec<(i32, i32, i32)> {
    let mut result = Vec::new();
    for bits in [32, 16] {
        for mode in modes.iter().filter(|mode| mode.bits == bits) {
            if mode.width < 800 || mode.height < 600 {
                continue;
            }
            let value = (mode.width as i32, mode.height as i32, bits);
            if !result.contains(&value) {
                result.push(value);
                if result.len() >= capacity {
                    break;
                }
            }
        }
    }
    result
}

pub fn refresh_rates(modes: &[DisplayMode], requested: (i32, i32, i32), capacity: usize) -> Vec<i32> {
    let mut result = Vec::new();
    for bits in [32, 16] {
        for mode in modes.iter().filter(|mode| mode.bits == bits) {
            if (mode.width as i32, mode.height as i32, bits) == requested {
                result.push(mode.refresh);
                if result.len() >= capacity {
                    break;
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mode(width: u32, height: u32, bits: i32, refresh: i32) -> DisplayMode {
        DisplayMode { width, height, bits, refresh }
    }

    #[test]
    fn get_res_10011bb0_preserves_format_order_and_deduplicates_only_triples() {
        let modes = [
            mode(800, 600, 16, 60), mode(1920, 1080, 32, 75),
            mode(799, 900, 32, 60), mode(1600, 599, 32, 60),
            mode(1920, 1080, 32, 60), mode(800, 600, 32, 60),
            mode(1024, 768, 24, 60), mode(800, 600, 16, 75),
            mode(1024, 768, 16, 60),
        ];
        assert_eq!(resolutions(&modes, 100), [(1920, 1080, 32), (800, 600, 32), (800, 600, 16), (1024, 768, 16)]);
        assert_eq!(resolutions(&[], 100), []);
        assert_eq!(resolutions(&[mode(u32::MAX, u32::MAX, 32, 60)], 100), [(-1, -1, 32)]);
    }

    #[test]
    fn get_res_10011bb0_tests_capacity_after_append_in_each_format_loop() {
        let modes: Vec<_> = [32, 16].into_iter().flat_map(|bits| (0..120).map(move |i| mode(800 + i, 600, bits, 60))).collect();
        for capacity in [0, 1, 2, 99, 100, 119] {
            let result = resolutions(&modes, capacity);
            assert_eq!(result.len(), capacity.max(1) + 1);
            assert_eq!(result.last(), Some(&(800, 600, 16)));
        }
        assert_eq!(resolutions(&modes, 121).len(), 121);
        assert_eq!(resolutions(&modes, 500).len(), 240);
    }

    #[test]
    fn get_refresh_rate_10011db0_retains_duplicates_and_has_no_minimum_size_filter() {
        let modes = [
            mode(640, 480, 32, 75), mode(640, 480, 16, 85),
            mode(640, 480, 32, 60), mode(640, 480, 32, 75),
            mode(640, 480, 32, 0), mode(640, 480, 24, 120),
            mode(800, 600, 32, 144), mode(640, 480, 16, 85),
        ];
        assert_eq!(refresh_rates(&modes, (640, 480, 32), 100), [75, 60, 75, 0]);
        assert_eq!(refresh_rates(&modes, (640, 480, 16), 100), [85, 85]);
        assert_eq!(refresh_rates(&modes, (640, 480, 24), 100), []);
        assert_eq!(refresh_rates(&modes, (1024, 768, 32), 100), []);
        for capacity in [0, 1, 2, 3, 4, 100] {
            assert_eq!(refresh_rates(&modes, (640, 480, 32), capacity), [75, 60, 75, 0][..capacity.clamp(1, 4)]);
        }
    }
}
