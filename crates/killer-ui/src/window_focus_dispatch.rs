#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusHandler {
    Base,
    Dialog,
    Chat,
    Calc,
    Messenger,
}

impl FocusHandler {
    pub fn from_address(address: u32) -> Option<Self> {
        match address {
            0x10035af0 => Some(Self::Base),
            0x10007e10 => Some(Self::Dialog),
            0x10053c10 => Some(Self::Chat),
            0x1011ec90 => Some(Self::Calc),
            0x10159870 => Some(Self::Messenger),
            _ => None,
        }
    }
}

pub fn focus_mode_from_address(address: u32) -> Option<bool> {
    match address {
        0x100023c0 => Some(false),
        0x100f78b0 => Some(true),
        _ => None,
    }
}

pub trait FocusDispatchBackend {
    type Error;
    fn focus_base(&mut self) -> Result<(), Self::Error>;
    fn dialog_input_mode(&mut self) -> u32;
    fn focus_child(&mut self, slot: u32) -> Result<(), Self::Error>;
    fn kill_own_focus_virtual(&mut self) -> Result<(), Self::Error>;
}

pub fn focus_with_handler<B: FocusDispatchBackend>(backend: &mut B, handler: FocusHandler) -> Result<(), B::Error> {
    match handler {
        FocusHandler::Base | FocusHandler::Messenger => backend.focus_base(),
        FocusHandler::Dialog => {
            backend.focus_base()?;
            if backend.dialog_input_mode() != 0 { backend.focus_child(0x240)?; }
            Ok(())
        }
        FocusHandler::Chat => {
            backend.focus_child(0x134)?;
            backend.kill_own_focus_virtual()
        }
        FocusHandler::Calc => backend.focus_child(0xec),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Backend { mode: u32, next_mode: u32, failure: Option<u32>, calls: Vec<u32> }
    impl FocusDispatchBackend for Backend {
        type Error = u32;
        fn focus_base(&mut self) -> Result<(), u32> {
            self.calls.push(0);
            self.mode = self.next_mode;
            if self.failure == Some(0) { Err(0) } else { Ok(()) }
        }
        fn dialog_input_mode(&mut self) -> u32 { self.calls.push(1); self.mode }
        fn focus_child(&mut self, slot: u32) -> Result<(), u32> {
            self.calls.push(slot);
            if self.failure == Some(slot) { Err(slot) } else { Ok(()) }
        }
        fn kill_own_focus_virtual(&mut self) -> Result<(), u32> { self.calls.push(2); Ok(()) }
    }

    #[test]
    fn dialog_uses_mode_after_base_while_chat_focuses_input_before_virtual_kill() {
        for mode in [0, 1, u32::MAX] {
            let mut b = Backend { mode: !mode, next_mode: mode, failure: None, calls: Vec::new() };
            focus_with_handler(&mut b, FocusHandler::Dialog).unwrap();
            assert_eq!(b.calls, if mode == 0 { vec![0, 1] } else { vec![0, 1, 0x240] });
        }
        let mut b = Backend { mode: 0, next_mode: 0, failure: None, calls: Vec::new() };
        focus_with_handler(&mut b, FocusHandler::Chat).unwrap();
        assert_eq!(b.calls, [0x134, 2]);
        let mut b = Backend { mode: 0, next_mode: 0, failure: None, calls: Vec::new() };
        focus_with_handler(&mut b, FocusHandler::Calc).unwrap();
        assert_eq!(b.calls, [0xec]);
    }

    #[test]
    fn failure_does_not_run_later_effects_and_unknown_dispatch_is_not_base() {
        for (handler, failure, expected) in [
            (FocusHandler::Dialog, 0, vec![0]),
            (FocusHandler::Dialog, 0x240, vec![0, 1, 0x240]),
            (FocusHandler::Chat, 0x134, vec![0x134]),
        ] {
            let mut b = Backend { mode: 1, next_mode: 1, failure: Some(failure), calls: Vec::new() };
            assert_eq!(focus_with_handler(&mut b, handler), Err(failure));
            assert_eq!(b.calls, expected);
        }
        assert_eq!(FocusHandler::from_address(0), None);
        assert_eq!(focus_mode_from_address(0), None);
        assert_eq!(focus_mode_from_address(0x100023c0), Some(false));
        assert_eq!(focus_mode_from_address(0x100f78b0), Some(true));
    }
}
