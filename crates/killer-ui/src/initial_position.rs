#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitialAxis {
    Start(i32),
    End(i32),
    Center(i32),
    CenterOffset(i32),
}

impl InitialAxis {
    pub fn horizontal(mode: i32, value: i32) -> Option<Self> {
        match mode {
            0 => Some(Self::Start(value)),
            1 => Some(Self::End(value)),
            2 => Some(Self::Center(value)),
            5 => Some(Self::CenterOffset(value)),
            _ => None,
        }
    }

    pub fn vertical(mode: i32, value: i32) -> Option<Self> {
        match mode {
            3 => Some(Self::Start(value)),
            4 => Some(Self::End(value)),
            2 => Some(Self::Center(value)),
            5 => Some(Self::CenterOffset(value)),
            _ => None,
        }
    }

    pub fn coordinate(self, screen: i32) -> i32 {
        match self {
            Self::Start(value) => value,
            Self::End(value) => screen.wrapping_sub(value),
            Self::Center(value) => (screen / 2).wrapping_sub(value / 2),
            Self::CenterOffset(value) => (screen / 2).wrapping_sub(value),
        }
    }
}

pub trait InitialPositionBackend {
    type RootRect;
    fn root_rect(&mut self) -> Self::RootRect;
    fn set_position(&mut self, x: i32, y: i32, root: Self::RootRect);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InitialPosition {
    pub enabled: bool,
    pub horizontal_mode: i32,
    pub vertical_mode: i32,
    pub x: i32,
    pub y: i32,
}

pub const STATUS_INITIAL_POSITION: InitialPosition =
    InitialPosition { enabled: true, horizontal_mode: 0, vertical_mode: 3, x: 0, y: 0 };
pub const DISABLED_INITIAL_POSITION: InitialPosition =
    InitialPosition { enabled: false, horizontal_mode: 0, vertical_mode: 0, x: 0, y: 0 };
pub const TARGET_INITIAL_POSITION: InitialPosition =
    InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 3, x: 172, y: 0 };

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UndefinedInitialCoordinate {
    Horizontal(i32),
    Vertical(i32),
}

impl InitialPosition {
    pub fn restore(&self, screen: (i32, i32), backend: &mut impl InitialPositionBackend) -> Result<(), UndefinedInitialCoordinate> {
        if !self.enabled {
            return Ok(());
        }
        let x = InitialAxis::horizontal(self.horizontal_mode, self.x)
            .ok_or(UndefinedInitialCoordinate::Horizontal(self.horizontal_mode))?.coordinate(screen.0);
        let y = InitialAxis::vertical(self.vertical_mode, self.y)
            .ok_or(UndefinedInitialCoordinate::Vertical(self.vertical_mode))?.coordinate(screen.1);
        let root = backend.root_rect();
        backend.set_position(x, y, root);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Backend {
        reads: usize,
        moves: Vec<(i32, i32, [f32; 4])>,
    }

    impl InitialPositionBackend for Backend {
        type RootRect = [f32; 4];
        fn root_rect(&mut self) -> Self::RootRect {
            self.reads += 1;
            [11.0, 12.0, 1558.0, 933.0]
        }
        fn set_position(&mut self, x: i32, y: i32, root: Self::RootRect) {
            assert_eq!(self.reads, self.moves.len() + 1);
            self.moves.push((x, y, root));
        }
    }

    #[test]
    fn restore_10036750_passes_screen_based_coordinates_and_the_root_rect() {
        let mut backend = Backend::default();
        let position = InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 4, x: 173, y: 187 };
        position.restore((1559, 933), &mut backend).unwrap();
        assert_eq!(backend.moves, [(693, 746, [11.0, 12.0, 1558.0, 933.0])]);
    }

    #[test]
    fn restore_10036750_disabled_has_no_effect_and_undefined_modes_are_not_invented() {
        let mut backend = Backend::default();
        let mut position = InitialPosition { enabled: false, horizontal_mode: 3, vertical_mode: 0, x: 0, y: 0 };
        position.restore((1558, 933), &mut backend).unwrap();
        assert_eq!(backend.reads, 0);
        position.enabled = true;
        assert_eq!(position.restore((1558, 933), &mut backend), Err(UndefinedInitialCoordinate::Horizontal(3)));
        position.horizontal_mode = 0;
        assert_eq!(position.restore((1558, 933), &mut backend), Err(UndefinedInitialCoordinate::Vertical(0)));
        assert_eq!(backend.reads, 0);
        assert!(backend.moves.is_empty());
    }

    #[test]
    fn creation_positions_capture_float_calculation_before_integer_screen_offsets() {
        for (viewport, main_xy, inventory_xy) in [
            ((1558, 933), (0, 214), (1256, 214)),
            ((503, 503), (0, 0), (201, 0)),
            ((301, 501), (0, -1), (-1, -1)),
            ((-1, -1), (0, -252), (-303, -252)),
            ((16_777_217, 16_777_217), (0, 8_388_356), (16_776_914, 8_388_356)),
        ] {
            assert_eq!(crate::mainwnd::default_position(viewport), main_xy);
            assert_eq!(crate::inventory::default_position(viewport), inventory_xy);
            for (descriptor, expected) in [
                (crate::mainwnd::initial_position(viewport), main_xy),
                (crate::inventory::initial_position(viewport), inventory_xy),
            ] {
                let mut backend = Backend::default();
                descriptor.restore(viewport, &mut backend).unwrap();
                assert_eq!((backend.moves[0].0, backend.moves[0].1), expected);
            }
        }
        assert_eq!(crate::mainwnd::initial_position((503, 503)).y, 251);
        assert_eq!(crate::inventory::initial_position((16_777_217, 503)).x, 303);
    }

    #[test]
    fn initial_axes_use_separate_signed_halves_and_wrapping_subtraction() {
        assert_eq!(InitialAxis::Center(3).coordinate(4), 1);
        assert_eq!(InitialAxis::Center(-3).coordinate(-4), -1);
        assert_eq!(InitialAxis::CenterOffset(3).coordinate(4), -1);
        assert_eq!(InitialAxis::End(-1).coordinate(i32::MAX), i32::MIN);
        assert_eq!(InitialAxis::Start(-9).coordinate(1558), -9);
        for mode in [-1, 0, 1, 2, 3, 4, 5, 6, i32::MIN, i32::MAX] {
            assert_eq!(InitialAxis::horizontal(mode, 0).is_some(), [0, 1, 2, 5].contains(&mode));
            assert_eq!(InitialAxis::vertical(mode, 0).is_some(), [2, 3, 4, 5].contains(&mode));
        }
    }
}
