pub trait WindowMessageBackend {
    type Error;
    fn flags(&mut self) -> u32;
    fn call_virtual(&mut self, slot: u32, arguments: &[u32]) -> Result<u32, Self::Error>;
}

pub fn command_target_message<B: WindowMessageBackend>(backend: &mut B, message: u32, wparam: u32) -> Result<bool, B::Error> {
    let value = match message {
        1 => backend.call_virtual(0x94, &[])?,
        2 => backend.call_virtual(0x98, &[])?,
        0x113 => backend.call_virtual(0x90, &[wparam])?,
        _ => return Ok(false),
    };
    Ok(value != 0)
}

pub fn window_message<B: WindowMessageBackend>(backend: &mut B, message: u32, wparam: u32, lparam: u32) -> Result<bool, B::Error> {
    if command_target_message(backend, message, wparam)? { return Ok(true); }
    if matches!(message, 15 | 0x505) {
        if backend.flags() & 2 != 0 {
            backend.call_virtual(if message == 15 { 0xf8 } else { 0xfc }, &[wparam])?;
        }
        return Ok(true);
    }
    let slot = match message {
        0x102 => 0x100,
        0x100 => 0x104,
        0x101 => 0x108,
        0x200 => 0x10c,
        0x201 => 0x110,
        0x204 => 0x118,
        0x202 => 0x114,
        0x205 => 0x11c,
        3 => 0x120,
        0x20a => 0x124,
        _ => return Ok(false),
    };
    Ok(backend.call_virtual(slot, &[wparam, lparam])? != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Backend { flags: u32, result: Result<u32, u32>, calls: Vec<(u32, Vec<u32>)>, queries: usize }
    impl WindowMessageBackend for Backend {
        type Error = u32;
        fn flags(&mut self) -> u32 { self.queries += 1; self.flags }
        fn call_virtual(&mut self, slot: u32, arguments: &[u32]) -> Result<u32, u32> { self.calls.push((slot, arguments.to_vec())); self.result }
    }

    #[test]
    fn base_dispatch_preserves_arguments_guards_and_normalizes_results() {
        for message in (0..=0x506).chain([u32::MAX, 0xf0000001]) {
            for flags in [0, 2, u32::MAX] {
                for value in [0, 1, 0x80000000, u32::MAX] {
                    let mut b = Backend { flags, result: Ok(value), calls: Vec::new(), queries: 0 };
                    let handled = window_message(&mut b, message, 0x12345678, 0x87654321).unwrap();
                    let mapped = [(1, 0x94, 0), (2, 0x98, 0), (0x113, 0x90, 1), (0x102, 0x100, 2), (0x100, 0x104, 2), (0x101, 0x108, 2), (0x200, 0x10c, 2), (0x201, 0x110, 2), (0x204, 0x118, 2), (0x202, 0x114, 2), (0x205, 0x11c, 2), (3, 0x120, 2), (0x20a, 0x124, 2)];
                    if matches!(message, 15 | 0x505) {
                        assert!(handled); assert_eq!(b.queries, 1);
                        assert_eq!(b.calls, if flags & 2 != 0 { vec![(if message == 15 { 0xf8 } else { 0xfc }, vec![0x12345678])] } else { vec![] });
                    } else if let Some((_, slot, count)) = mapped.iter().find(|(m, _, _)| *m == message) {
                        assert_eq!(handled, value != 0); assert_eq!(b.queries, 0);
                        assert_eq!(b.calls, [(*slot, [0x12345678, 0x87654321][..*count].to_vec())]);
                    } else { assert!(!handled); assert_eq!(b.queries, 0); assert!(b.calls.is_empty()); }
                }
            }
        }
    }

    #[test]
    fn callback_failure_propagates_instead_of_claiming_the_message_was_handled() {
        for message in [1, 2, 0x113, 15, 0x505, 0x100] {
            let mut b = Backend { flags: 2, result: Err(99), calls: Vec::new(), queries: 0 };
            assert_eq!(window_message(&mut b, message, 0, 0), Err(99));
            assert_eq!(b.calls.len(), 1);
        }
    }
}
