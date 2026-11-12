const PHYSICAL: [[i32; 6]; 3] = [[0, 2, 3, 3, 4, 5], [0, 2, 4, 4, 5, 6], [0, 4, 6, 6, 8, 10]];
const MAGICAL: [i32; 6] = [0, 2, 3, 3, 3, 4];
const DEFENSE: [i32; 6] = [0, 1, 1, 1, 1, 1];

fn weapon_factor(enchant: i32) -> i32 {
    if enchant > 3 { enchant.wrapping_mul(2).wrapping_sub(3) } else { enchant }
}

fn weapon_table(item_type: i32, body_part: i32) -> Option<usize> {
    match item_type {
        1 | 2 => Some(usize::from(body_part == 0x4000)),
        3 | 4 | 7 => Some(0),
        5 | 8 => Some(1),
        6 => Some(2),
        _ => None,
    }
}

pub fn physical_attack(item_type: i32, body_part: i32, grade: usize, enchant: i32, base: i32) -> Option<i32> {
    let Some(table) = weapon_table(item_type, body_part) else { return Some(base) };
    let coefficient = *PHYSICAL[table].get(grade)?;
    Some((i64::from(base) + i64::from(coefficient) * i64::from(weapon_factor(enchant))) as i32)
}

pub fn magical_attack(item_type: i32, body_part: i32, grade: usize, enchant: i32, base: i32) -> Option<i32> {
    if weapon_table(item_type, body_part).is_none() {
        return Some(base);
    }
    let coefficient = *MAGICAL.get(grade)?;
    Some((i64::from(base) + i64::from(coefficient) * i64::from(weapon_factor(enchant))) as i32)
}

pub fn defense(grade: usize, enchant: i32, base: i32) -> Option<i32> {
    let coefficient = *DEFENSE.get(grade)?;
    let factor = if enchant > 3 { enchant.wrapping_mul(3).wrapping_sub(6) } else { enchant };
    Some((i64::from(base) + i64::from(coefficient) * i64::from(factor)) as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn enchanted_attributes_match_original_x86_instructions() {
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let python = std::env::var_os("KILLER_ORACLE_PYTHON").unwrap_or_else(|| "python3".into());
        let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/item_stats_oracle.py");
        let output = std::process::Command::new(python).arg(script).arg(root.join("system/nwindow.dll")).output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let text = String::from_utf8(output.stdout).unwrap();
        let mut count = 0;
        for line in text.lines() {
            let values: Vec<i32> = line.split_whitespace().map(|value| value.parse().unwrap()).collect();
            assert_eq!(values.len(), 8);
            let (kind, body, grade, enchant, base) = (values[0], values[1], values[2] as usize, values[3], values[4]);
            assert_eq!(physical_attack(kind, body, grade, enchant, base), Some(values[5]), "{line}");
            assert_eq!(magical_attack(kind, body, grade, enchant, base), Some(values[6]), "{line}");
            assert_eq!(defense(grade, enchant, base), Some(values[7]), "{line}");
            count += 1;
        }
        assert_eq!(count, 10 * 4 * 6 * 7 * 4);
    }

    #[test]
    fn enchanted_patk_10074260_uses_native_type_body_and_grade_tables() {
        for kind in 1..=8 {
            for body in [0, 0x80, 0x100, 0x4000] {
                for grade in 0..6 {
                    let coefficients = match kind {
                        6 => [0, 4, 6, 6, 8, 10],
                        5 | 8 => [0, 2, 4, 4, 5, 6],
                        1 | 2 if body == 0x4000 => [0, 2, 4, 4, 5, 6],
                        _ => [0, 2, 3, 3, 4, 5],
                    };
                    for (enchant, factor) in [(-1, -1), (0, 0), (3, 3), (4, 5), (10, 17)] {
                        assert_eq!(physical_attack(kind, body, grade, enchant, 100), Some(100 + coefficients[grade] * factor));
                        assert_eq!(magical_attack(kind, body, grade, enchant, 100), Some(100 + [0, 2, 3, 3, 3, 4][grade] * factor));
                    }
                }
            }
        }
        assert_eq!(physical_attack(0, 0, usize::MAX, 4, 100), Some(100));
        assert_eq!(physical_attack(1, 0, 6, 4, 100), None);
        assert_eq!(magical_attack(1, 0, 6, 4, 100), None);
    }

    #[test]
    fn enchanted_pdef_100743a0_preserves_threshold_and_integer_overflow() {
        assert_eq!(defense(0, 10, 100), Some(100));
        assert_eq!(defense(1, 3, 100), Some(103));
        assert_eq!(defense(5, 4, 100), Some(106));
        assert_eq!(defense(6, 4, 100), None);
        assert_eq!(defense(1, i32::MAX, 0), Some(2147483639));
        assert_eq!(physical_attack(1, 0, 1, i32::MAX, 0), Some(-10));
    }
}
