use killer_ui::window::WindowId;
use killer_ui::window_stuck::{StuckWindowGraph, WindowStuckState, collect_stuck_windows};
use std::io::Write;
use std::process::{Command, Stdio};

struct Graph([WindowStuckState; 4]);

impl StuckWindowGraph for Graph {
    fn stuck_state(&self, window: WindowId) -> Option<&WindowStuckState> {
        self.0.get(window.0 as usize)
    }
}

#[test]
#[ignore]
fn get_stuck_list_100364b0_matches_original_order_cycles_nulls_and_prefixes() {
    let mut cases = Vec::new();
    let mut seed = 123456789_u32;
    for graph_index in 0..64 {
        let edges: [u32; 32] = std::array::from_fn(|index| {
            if graph_index == 0 {
                return 0;
            }
            if graph_index == 1 {
                return (index / 8 + 1) as u32;
            }
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            seed % 5
        });
        for start in 0..4 {
            for prefix in 0..5 {
                cases.push((start, prefix, edges));
            }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/stuck_list_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for (start, prefix, edges) in &cases {
        writeln!(input, "{start} {prefix} {}", edges.iter().map(ToString::to_string).collect::<Vec<_>>().join(" "))
            .unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{:?}", output.status);
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for ((start, prefix, edges), native) in cases.into_iter().zip(output.lines()) {
        let mut graph = Graph(std::array::from_fn(|_| WindowStuckState::new(0)));
        for (i, state) in graph.0.iter_mut().enumerate() {
            for (side, links) in state.links.iter_mut().enumerate() {
                *links = edges[i * 8 + side * 2..i * 8 + side * 2 + 2]
                    .iter()
                    .map(|target| target.checked_sub(1).map(WindowId))
                    .collect();
            }
        }
        let mut collected = if prefix == 0 { Vec::new() } else { vec![WindowId(prefix - 1); 2] };
        collect_stuck_windows(&graph, WindowId(start), &mut collected).unwrap();
        assert_eq!(
            collected.iter().map(|id| id.0.to_string()).collect::<Vec<_>>().join(" "),
            native,
            "start={start}, prefix={prefix}, edges={edges:?}"
        );
    }
}
