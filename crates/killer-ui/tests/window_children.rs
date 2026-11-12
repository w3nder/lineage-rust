use killer_ui::window_children::{ChildRecord, WindowChildren};
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore]
fn child_list_matches_original_insertion_and_first_identity_removal() {
    let mut seed = 0x3291137_u32;
    let mut cases = Vec::new();
    for _ in 0..128 {
        let mut ops = Vec::new();
        for index in 0..64 {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            ops.push([u32::from(index < 8 || seed & 0x10000 != 0), (seed >> 20) % 8, seed]);
        }
        for record in 0..8 {
            for _ in 0..64 {
                ops.push([0, record, 0]);
            }
        }
        cases.push(ops);
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_children_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for ops in &cases {
        writeln!(input, "{}", ops.iter().flatten().map(u32::to_string).collect::<Vec<_>>().join(" ")).unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for (ops, line) in cases.iter().zip(output.lines()) {
        let mut list = WindowChildren::default();
        let (mut freed, mut queries) = (0, 0);
        let mut actual = Vec::new();
        for &[insert, record, flags] in ops {
            if insert != 0 {
                list.insert(ChildRecord(record), || {
                    queries += 1;
                    flags
                });
            } else if list.remove(ChildRecord(record)).is_some() {
                freed += 1;
            }
            let mut node = list.first();
            let mut previous = None;
            while let Some(id) = node {
                assert_eq!(list.previous(id), previous);
                actual.push(i64::from(list.record(id).unwrap().0));
                previous = Some(id);
                node = list.next(id);
            }
            assert_eq!(list.last(), previous);
            actual.extend([-1, freed, queries]);
        }
        let expected: Vec<i64> = line.split_whitespace().map(|v| v.parse().unwrap()).collect();
        assert_eq!(actual, expected);
    }
}
