#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeHandler {
    Base,
    SystemMessage,
    PrimaryRoot,
    SecondaryRoot,
    BroadcastObserver,
}

impl ResizeHandler {
    pub fn from_address(address: u32) -> Option<Self> {
        match address {
            0x10036de0 => Some(Self::Base),
            0x10052b20 => Some(Self::SystemMessage),
            0x1005c370 => Some(Self::PrimaryRoot),
            0x10091080 => Some(Self::SecondaryRoot),
            0x1012ef80 => Some(Self::BroadcastObserver),
            _ => None,
        }
    }
}

pub trait ResizeDispatchBackend {
    type Window;
    type Error;
    fn resize_base(&mut self, width: f32, height: f32) -> Result<(), Self::Error>;
    fn child(&mut self, slot: u32) -> Option<Self::Window>;
    fn size(&mut self) -> (f32, f32);
    fn resize_child(&mut self, window: Self::Window, width: f32, height: f32) -> Result<(), Self::Error>;
}

pub fn resize_with_handler<B: ResizeDispatchBackend>(backend: &mut B, handler: ResizeHandler, width: f32, height: f32) -> Result<(), B::Error> {
    backend.resize_base(width, height)?;
    match handler {
        ResizeHandler::Base | ResizeHandler::SystemMessage | ResizeHandler::BroadcastObserver => {}
        ResizeHandler::PrimaryRoot => {
            if let Some(child) = backend.child(0x1e0) { backend.resize_child(child, width, height)?; }
        }
        ResizeHandler::SecondaryRoot => {
            let message = backend.child(0xf0);
            let (width, height) = backend.size();
            if let Some(message) = message { backend.resize_child(message, width, 52.0)?; }
            if let Some(child) = backend.child(0xec) { backend.resize_child(child, width, height - 52.0)?; }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Backend {
        size: (f32, f32),
        second: Option<u32>,
        failure: Option<u32>,
        calls: Vec<(u32, f32, f32)>,
    }

    impl ResizeDispatchBackend for Backend {
        type Window = u32;
        type Error = u32;
        fn resize_base(&mut self, width: f32, height: f32) -> Result<(), u32> {
            self.calls.push((0, width, height));
            self.size = (777.0, 99.0);
            self.second = Some(2);
            if self.failure == Some(0) { Err(0) } else { Ok(()) }
        }
        fn child(&mut self, slot: u32) -> Option<u32> {
            match slot { 0xf0 | 0x1e0 => Some(1), 0xec => self.second, _ => None }
        }
        fn size(&mut self) -> (f32, f32) { self.size }
        fn resize_child(&mut self, child: u32, width: f32, height: f32) -> Result<(), u32> {
            self.calls.push((child, width, height));
            self.size = (1.0, 2.0);
            self.second = Some(3);
            if self.failure == Some(child) { Err(child) } else { Ok(()) }
        }
    }

    fn backend(failure: Option<u32>) -> Backend {
        Backend { size: (0.0, 0.0), second: None, failure, calls: Vec::new() }
    }

    #[test]
    fn primary_preserves_input_secondary_captures_post_base_size_and_reloads_second_child() {
        let mut primary = backend(None);
        resize_with_handler(&mut primary, ResizeHandler::PrimaryRoot, -3.0, 42.0).unwrap();
        assert_eq!(primary.calls, [(0, -3.0, 42.0), (1, -3.0, 42.0)]);
        let mut secondary = backend(None);
        resize_with_handler(&mut secondary, ResizeHandler::SecondaryRoot, -3.0, 42.0).unwrap();
        assert_eq!(secondary.calls, [(0, -3.0, 42.0), (1, 777.0, 52.0), (3, 777.0, 47.0)]);
    }

    #[test]
    fn base_wrapper_and_callback_failures_do_not_run_extra_effects() {
        for handler in [ResizeHandler::Base, ResizeHandler::SystemMessage] {
            let mut b = backend(None);
            resize_with_handler(&mut b, handler, 10.0, 20.0).unwrap();
            assert_eq!(b.calls, [(0, 10.0, 20.0)]);
        }
        for failure in [0, 1, 3] {
            let mut b = backend(Some(failure));
            assert_eq!(resize_with_handler(&mut b, ResizeHandler::SecondaryRoot, 10.0, 20.0), Err(failure));
            assert_eq!(b.calls.last().unwrap().0, failure);
        }
        assert_eq!(ResizeHandler::from_address(0), None);
    }
}
