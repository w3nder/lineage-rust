use killer_ui::window_resize::{PROPAGATING_CLASSES, WindowResizeBackend, resize_window};
use killer_ui::window_stuck::WindowStuckState;
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    case: [u32; 6],
    size: (f32, f32),
    nodes: usize,
    calls: Vec<String>,
    root_state: WindowStuckState,
    child_states: [WindowStuckState; 3],
}

impl WindowResizeBackend for Backend {
    type ChildNode = usize;
    fn clear_relations(&mut self) {
        self.calls.push("clear".into());
        self.root_state.init_stuck_state();
    }
    fn has_frame_flag(&mut self) -> bool {
        self.case[2] & 1 != 0
    }
    fn has_frame(&mut self) -> bool {
        self.case[2] & 2 != 0
    }
    fn frame_height(&mut self) -> f32 {
        self.calls.push("frame_height".into());
        31.75
    }
    fn resize_frame(&mut self, width: i32, height: i32) {
        self.calls.push(format!("frame:{width}:{height}"));
        if self.case[2] & 8 != 0 {
            self.size = (900.0, 700.0);
        }
    }
    fn size(&mut self) -> (f32, f32) {
        self.size
    }
    fn set_size(&mut self, size: (f32, f32)) {
        self.size = size;
        self.calls.push(format!("size:{}:{}", size.0.to_bits(), size.1.to_bits()));
    }
    fn has_anchor(&mut self) -> bool {
        self.case[2] & 4 != 0
    }
    fn update_anchor(&mut self, x: i32, y: i32) {
        assert_eq!((x, y), (0, 0));
        self.calls.push("anchor".into());
    }
    fn inherits(&mut self, class: u32) -> bool {
        self.case[3] == class
    }
    fn first_child(&mut self) -> Option<usize> {
        (self.nodes > 0).then_some(0)
    }
    fn next_child(&mut self, node: usize) -> Option<usize> {
        (node + 1 < self.nodes).then_some(node + 1)
    }
    fn prepare_child(&mut self, node: usize, mode: i32) {
        assert_eq!(mode, 3);
        self.calls.push(format!("prepare:{node}"));
        self.child_states[node].clear_flags(mode as u32);
        if self.case[2] & 16 != 0 && node == 0 {
            self.nodes = 3;
        }
    }
    fn rearrange_child(&mut self, node: usize, old: (f32, f32), new: (f32, f32)) {
        self.calls.push(format!(
            "rearrange:{node}:{}:{}:{}:{}",
            old.0.to_bits(),
            old.1.to_bits(),
            new.0.to_bits(),
            new.1.to_bits()
        ));
    }
}

#[test]
#[ignore]
fn resize_10036de0_matches_original_branches_class_chains_and_effects() {
    let mut cases = Vec::new();
    for class in PROPAGATING_CLASSES.into_iter().chain([0, 0x102eb440]) {
        for subclass in 0..2 {
            for mask in 0..32 {
                for count in [0, 2] {
                    cases.push([100.75_f32.to_bits(), 52.0_f32.to_bits(), mask, class, subclass, count]);
                }
            }
        }
    }
    for bits in [
        0, 1, 0x80000000, 0x80000001, 0x7fc01234, 0x7f800001, 0xffc01234, 0xff800001, 0x7f800000, 0xff800000,
        0x4f000000, 0x5effffff, 0x5f000000, 0xdf000000, 0xdf000001, 0x7f7fffff,
    ] {
        cases.push([bits, bits ^ 0x80000000, 31, PROPAGATING_CLASSES[0], 1, 2]);
    }
    let mut seed = 123456789_u32;
    for _ in 0..512 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        cases.push([seed, seed ^ 0x80000000, 31, PROPAGATING_CLASSES[9], 1, 2]);
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_resize_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for case in &cases {
        writeln!(input, "{}", case.iter().map(ToString::to_string).collect::<Vec<_>>().join(" ")).unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{:?}", output.status);
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for (case, native) in cases.into_iter().zip(output.lines()) {
        let mut root_state = WindowStuckState::new(0xdeadbeef);
        for (links, count) in root_state.links.iter_mut().zip([1, 0, 3, 2]) {
            links.reserve(8);
            links.resize(count, None);
        }
        let mut backend = Backend {
            case,
            size: (800.0, 600.0),
            nodes: case[5] as usize,
            calls: Vec::new(),
            root_state,
            child_states: std::array::from_fn(|i| WindowStuckState::new(0xabcdef03 + i as u32 * 16)),
        };
        resize_window(&mut backend, f32::from_bits(case[0]), f32::from_bits(case[1]));
        assert!(backend.root_state.links.iter().all(|links| links.is_empty() && links.capacity() == 0));
        assert_eq!(backend.root_state.flags(), 0xdeadbeef);
        backend.calls.push(format!(
            "flags:{}",
            backend.child_states.iter().map(|state| state.flags().to_string()).collect::<Vec<_>>().join(":")
        ));
        assert_eq!(backend.calls.join("|"), native, "{case:?}");
    }
}
