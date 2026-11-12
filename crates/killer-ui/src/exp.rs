pub const EXP_TABLE: [i64; 88] = [
    0, 68, 363, 1168, 2884, 6038, 11287, 19423,
    31378, 48229, 71202, 101677, 141193, 191454, 254330, 331867,
    426288, 540000, 675596, 835862, 1023784, 1242546, 1495543, 1786379,
    2118876, 2497077, 2925250, 3407897, 3949754, 4555796, 5231246, 5981576,
    6812513, 7730044, 8740422, 9850166, 11066072, 12395215, 13844951, 15422929,
    17137087, 18995665, 21007203, 23180550, 25524868, 28049635, 30764654, 33680052,
    36806289, 40154162, 45525133, 51262490, 57383988, 63907911, 70853089, 80700831,
    91162654, 102265881, 114038596, 126509653, 146308200, 167244337, 189364894, 212717908,
    237352644, 271975263, 308443198, 346827154, 387199547, 429634523, 474207979, 532694979,
    606322775, 696381369, 804225364, 931275828, 1151275834, 1511275834, 2099275834, 2099325834,
    2099375834, 2099425834, 2099475834, 2099525834, 2099575834, 2099625834, 2099675834, 2099725834,
];

pub fn level_bounds(level: i32) -> Option<(i64, i64)> {
    let index = usize::try_from(level).ok()?;
    let start = if index == 0 { 0 } else { *EXP_TABLE.get(index - 1)? };
    let next = *EXP_TABLE.get(index)?;
    (next != 0).then_some((start, next))
}

pub fn exp_percent(level: i32, exp: i64) -> f32 {
    let Some((start, next)) = level_bounds(level) else { return 0.0 };
    ((exp - start) as f32 * 100.0) / (next - start) as f32
}

pub fn exp_fill(level: i32, exp: i64, width: i32) -> i32 {
    let Some((start, next)) = level_bounds(level) else { return 0 };
    let fill = (exp - start) * i64::from(width) / (next - start);
    fill.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_starts_at_zero_and_grows() {
        assert_eq!(&EXP_TABLE[..4], &[0, 68, 363, 1168]);
        assert!(EXP_TABLE.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(EXP_TABLE[77], 1511275834);
        assert_eq!(EXP_TABLE[78], 2099275834);
    }

    #[test]
    fn level_bounds_follow_the_status_paint_indexing() {
        assert_eq!(level_bounds(1), Some((0, 68)));
        assert_eq!(level_bounds(2), Some((68, 363)));
        assert_eq!(level_bounds(52), Some((51262490, 57383988)));
        assert_eq!(level_bounds(0), None);
        assert_eq!(level_bounds(88), None);
        assert_eq!(level_bounds(-1), None);
    }

    #[test]
    fn percent_is_the_float_formula_of_the_client() {
        assert_eq!(exp_percent(2, 127), 20.0);
        assert_eq!(format!("{:.2}%", exp_percent(52, 52262490)), "16.34%");
        assert_eq!(format!("{:.2}%", exp_percent(52, 54323239)), "50.00%");
        assert_eq!(format!("{:.2}%", exp_percent(10, 71201)), "100.00%");
        assert_eq!(exp_percent(1, 0), 0.0);
        assert_eq!(exp_percent(99, 5), 0.0);
    }

    #[test]
    fn bar_fill_truncates_like_ftol() {
        assert_eq!(exp_fill(2, 127, 150), 30);
        assert_eq!(exp_fill(52, 52262490, 150), 24);
        assert_eq!(exp_fill(52, 54323239, 150), 75);
        assert_eq!(exp_fill(1, 0, 150), 0);
        assert_eq!(exp_fill(99, 5, 150), 0);
    }
}
