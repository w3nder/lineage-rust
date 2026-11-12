pub const FOCUS_CLASS: u32 = 0x102c7560;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissingFocusRecord;

pub trait WindowFocusBackend {
    type Record: Copy + PartialEq;
    fn focused_record(&mut self) -> Option<Self::Record>;
    fn record(&mut self) -> Option<Self::Record>;
    fn kill_record_focus(&mut self, record: Self::Record);
    fn set_focused_record(&mut self, record: Option<Self::Record>);
    fn record_focus_mode(&mut self, record: Self::Record) -> bool;
    fn focus_descendant(&mut self) -> bool;
    fn flags(&mut self) -> u32;
    fn record_inherits(&mut self, record: Self::Record, class: u32) -> bool;
    fn has_direct_focus_class(&mut self) -> bool;
    fn focus_parent_of_class(&mut self) -> bool;
    fn set_own_focus(&mut self, focused: bool);
    fn parent_record(&mut self, record: Self::Record) -> Option<Self::Record>;
    fn promote_child(&mut self, parent: Self::Record, child: Self::Record);
}

pub fn kill_focus(backend: &mut impl WindowFocusBackend) {
    if backend.focused_record() == backend.record() {
        backend.set_focused_record(None);
    }
    backend.set_own_focus(false);
}

pub fn set_focus(backend: &mut impl WindowFocusBackend) -> Result<(), MissingFocusRecord> {
    let old = backend.focused_record();
    if old == backend.record() { return Ok(()); }
    if let Some(old) = old {
        backend.kill_record_focus(old);
        backend.set_focused_record(None);
    }
    let record = backend.record().ok_or(MissingFocusRecord)?;
    let focused_descendant = !backend.record_focus_mode(record) && backend.focus_descendant();
    if backend.flags() & 0x200000 != 0 {
        if focused_descendant { return Ok(()); }
    } else {
        if focused_descendant { return Ok(()); }
        let record = backend.record().ok_or(MissingFocusRecord)?;
        if !backend.record_inherits(record, FOCUS_CLASS)
            && !backend.has_direct_focus_class()
            && backend.focus_parent_of_class()
        { return Ok(()); }
    }
    let record = backend.record();
    backend.set_own_focus(true);
    backend.set_focused_record(record);
    let record = backend.record().ok_or(MissingFocusRecord)?;
    if let Some(parent) = backend.parent_record(record) {
        backend.promote_child(parent, record);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Backend {
        mask: u32,
        own: Option<u32>,
        global: Option<u32>,
        focused: bool,
        events: Vec<u32>,
    }

    impl WindowFocusBackend for Backend {
        type Record = u32;
        fn focused_record(&mut self) -> Option<u32> { self.global }
        fn record(&mut self) -> Option<u32> { self.own }
        fn kill_record_focus(&mut self, _: u32) { self.events.push(1); self.global = Some(99); }
        fn set_focused_record(&mut self, record: Option<u32>) { self.global = record; self.events.push(if record.is_some() { 9 } else { 2 }); }
        fn record_focus_mode(&mut self, _: u32) -> bool { self.events.push(3); self.mask & 1 != 0 }
        fn focus_descendant(&mut self) -> bool { self.events.push(4); self.mask & 2 != 0 }
        fn flags(&mut self) -> u32 { if self.mask & 4 != 0 { 0x200000 } else { 0 } }
        fn record_inherits(&mut self, _: u32, class: u32) -> bool { assert_eq!(class, FOCUS_CLASS); self.events.push(5); self.mask & 8 != 0 }
        fn has_direct_focus_class(&mut self) -> bool { self.events.push(6); self.mask & 16 != 0 }
        fn focus_parent_of_class(&mut self) -> bool { self.events.push(7); self.mask & 32 != 0 }
        fn set_own_focus(&mut self, focused: bool) { self.focused = focused; self.events.push(8); }
        fn parent_record(&mut self, _: u32) -> Option<u32> { (self.mask & 64 != 0).then_some(11) }
        fn promote_child(&mut self, parent: u32, child: u32) { assert_eq!((parent, child), (11, 7)); self.events.push(10); }
    }

    #[test]
    fn set_focus_preserves_every_guard_and_effect_order() {
        for mask in 0..128 {
            let mut b = Backend { mask, own: Some(7), global: Some(5), focused: false, events: Vec::new() };
            set_focus(&mut b).unwrap();
            let mut expected = vec![1, 2, 3];
            let mut accepted = true;
            if mask & 1 == 0 {
                expected.push(4);
                accepted = mask & 2 == 0;
            }
            if accepted && mask & 4 == 0 {
                expected.push(5);
                if mask & 8 == 0 {
                    expected.push(6);
                    if mask & 16 == 0 { expected.push(7); accepted = mask & 32 == 0; }
                }
            }
            if accepted { expected.extend([8, 9]); if mask & 64 != 0 { expected.push(10); } }
            assert_eq!(b.events, expected, "mask={mask}");
            assert_eq!(b.focused, accepted);
            assert_eq!(b.global, accepted.then_some(7));
        }
    }

    #[test]
    fn equal_records_return_early_and_kill_preserves_other_global_focus() {
        for record in [None, Some(7)] {
            let mut b = Backend { mask: 0, own: record, global: record, focused: true, events: Vec::new() };
            set_focus(&mut b).unwrap();
            assert!(b.events.is_empty());
            kill_focus(&mut b);
            assert_eq!(b.global, None);
            assert!(!b.focused);
            b.global = Some(9);
            b.focused = true;
            kill_focus(&mut b);
            assert_eq!(b.global, Some(9));
            assert!(!b.focused);
        }
    }

    #[test]
    #[ignore]
    fn focus_flows_match_native_guards_class_chain_and_effect_order() {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut cases = Vec::new();
        for mask in 0..128 {
            for old in [0, 5, 7] {
                for depth in 0..3 { cases.push([0, mask, old, 1, depth]); }
            }
        }
        cases.push([0, 0, 0, 0, 0]);
        for old in [0, 5, 7] {
            for present in 0..2 { cases.push([1, 0, old, present, 0]); }
        }
        let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
            .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_focus_reference.py"))
            .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
            .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        let mut input = child.stdin.take().unwrap();
        for v in &cases { writeln!(input, "{}", v.iter().map(u32::to_string).collect::<Vec<_>>().join(" ")).unwrap(); }
        drop(input);
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let output = String::from_utf8(output.stdout).unwrap();
        assert_eq!(output.lines().count(), cases.len());
        for (v, line) in cases.iter().zip(output.lines()) {
            let mut b = Backend { mask: v[1], own: (v[3] != 0).then_some(7), global: (v[2] != 0).then_some(v[2]), focused: v[0] != 0, events: Vec::new() };
            if v[0] == 0 { set_focus(&mut b).unwrap(); } else { kill_focus(&mut b); }
            let expected: Vec<_> = [u32::from(b.focused), b.global.unwrap_or(0)].into_iter().chain(b.events).collect();
            let actual: Vec<u32> = line.split_whitespace().map(|value| value.parse().unwrap()).collect();
            assert_eq!(actual, expected, "case={v:?}");
        }
    }
}
