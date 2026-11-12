use super::TILE_SIZE;
use crate::options::range_units;

pub const WINDOW_STEP: f32 = 8192.0;
pub const MAX_NEIGHBOURS: usize = 4;
pub const PURGE_DISTANCE: f32 = 35000.0;
pub const DRAW_MARGIN: f32 = 16384.0;
const TILE_X_BASE: i32 = 20;
const TILE_Y_BASE: i32 = 18;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TileId {
    pub x: i32,
    pub y: i32,
}

impl TileId {
    pub fn of(x: f32, y: f32) -> Self {
        Self { x: (x / TILE_SIZE).floor() as i32 + TILE_X_BASE, y: (y / TILE_SIZE).floor() as i32 + TILE_Y_BASE }
    }

    pub fn name(self) -> String {
        format!("{}_{}", self.x, self.y)
    }

    pub fn parse(name: &str) -> Option<Self> {
        let (x, y) = name.split_once('_')?;
        Some(Self { x: x.parse().ok()?, y: y.parse().ok()? })
    }

    /// World center of the tile's terrain, as `Location` of every tile is placed.
    pub fn center(self) -> [f32; 2] {
        [(self.x - TILE_X_BASE) as f32 * TILE_SIZE + TILE_SIZE / 2.0, (self.y - TILE_Y_BASE) as f32 * TILE_SIZE + TILE_SIZE / 2.0]
    }

    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self { x: self.x + dx, y: self.y + dy }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    MinusX,
    PlusX,
    MinusY,
    PlusY,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoadRequest {
    pub first: TileId,
    pub second: TileId,
    pub direction: Direction,
}

impl LoadRequest {
    pub fn tiles(&self) -> Vec<TileId> {
        if self.first == self.second { vec![self.first] } else { vec![self.first, self.second] }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Neighbour {
    pub tile: TileId,
    pub center: [f32; 2],
}

/// The sliding-window bookkeeping of `UGameEngine::AdjustTerrain` and `ULevel::AddSWLevel`.
#[derive(Clone, Debug, PartialEq)]
pub struct TileWindow {
    pub cx: f32,
    pub cy: f32,
    pub sx: i32,
    pub sy: i32,
    pub main: TileId,
    pub neighbours: Vec<Neighbour>,
    pending: Option<LoadRequest>,
}

fn cycle_down(s: i32) -> i32 {
    (s + 2) % 4 - 1
}

fn cycle_up(s: i32) -> i32 {
    (s - 3) % 4 + 2
}

impl TileWindow {
    pub fn new(main: TileId) -> Self {
        let [cx, cy] = main.center();
        Self { cx, cy, sx: 0, sy: 0, main, neighbours: Vec::new(), pending: None }
    }

    pub fn pending(&self) -> Option<LoadRequest> {
        self.pending
    }

    fn window_tile(&self) -> TileId {
        TileId::of(self.cx, self.cy)
    }

    pub fn step(&mut self, x: f32, y: f32) -> Option<LoadRequest> {
        if self.pending.is_some() {
            return None;
        }
        let t = self.window_tile();
        let side_y = if self.sy > 0 { -1 } else if self.sy < 0 { 1 } else { 0 };
        let side_x = if self.sx > 0 { -1 } else if self.sx < 0 { 1 } else { 0 };
        let request = if x < self.cx - WINDOW_STEP {
            let first = t.offset(if self.sx >= 0 { -1 } else { 0 }, 0);
            LoadRequest { first, second: first.offset(0, side_y), direction: Direction::MinusX }
        } else if x > self.cx + WINDOW_STEP {
            let first = t.offset(if self.sx <= 0 { 1 } else { 0 }, 0);
            LoadRequest { first, second: first.offset(0, side_y), direction: Direction::PlusX }
        } else if y < self.cy - WINDOW_STEP {
            let first = t.offset(0, if self.sy >= 0 { -1 } else { 0 });
            LoadRequest { first, second: first.offset(side_x, 0), direction: Direction::MinusY }
        } else if y > self.cy + WINDOW_STEP {
            let first = t.offset(0, if self.sy <= 0 { 1 } else { 0 });
            LoadRequest { first, second: first.offset(side_x, 0), direction: Direction::PlusY }
        } else {
            return None;
        };
        self.pending = Some(request);
        Some(request)
    }

    pub fn commit(&mut self, direction: Direction) {
        match direction {
            Direction::MinusX => {
                self.sx = cycle_down(self.sx);
                self.cx -= WINDOW_STEP;
            }
            Direction::PlusX => {
                self.sx = cycle_up(self.sx);
                self.cx += WINDOW_STEP;
            }
            Direction::MinusY => {
                self.sy = cycle_down(self.sy);
                self.cy -= WINDOW_STEP;
            }
            Direction::PlusY => {
                self.sy = cycle_up(self.sy);
                self.cy += WINDOW_STEP;
            }
        }
        self.pending = None;
    }

    pub fn cancel(&mut self) {
        self.pending = None;
    }

    pub fn attach(&mut self, tile: TileId, position: [f32; 2]) -> Vec<TileId> {
        if tile == self.main || self.neighbours.iter().any(|n| n.tile == tile) {
            return Vec::new();
        }
        let distance = |center: [f32; 2]| (position[0] - center[0]).powi(2) + (position[1] - center[1]).powi(2);
        self.neighbours.sort_by(|a, b| distance(a.center).total_cmp(&distance(b.center)));
        let new = Neighbour { tile, center: tile.center() };
        let at = self.neighbours.iter().position(|n| distance(n.center) > distance(new.center)).unwrap_or(self.neighbours.len());
        self.neighbours.insert(at, new);
        let mut removed = Vec::new();
        while self.neighbours.len() > MAX_NEIGHBOURS {
            let far = self.neighbours.pop().unwrap_or(Neighbour { tile, center: [0.0; 2] });
            removed.push(far.tile);
        }
        removed
    }

    pub fn detach(&mut self, tile: TileId) {
        self.neighbours.retain(|n| n.tile != tile);
    }

    pub fn purge(&mut self, position: [f32; 2]) -> Vec<TileId> {
        let far: Vec<TileId> = self
            .neighbours
            .iter()
            .filter(|n| (position[0] - n.center[0]).abs() > PURGE_DISTANCE || (position[1] - n.center[1]).abs() > PURGE_DISTANCE)
            .map(|n| n.tile)
            .collect();
        for tile in &far {
            self.detach(*tile);
        }
        far
    }

    pub fn current(&self, x: f32, y: f32) -> TileId {
        let here = TileId::of(x, y);
        if self.neighbours.iter().any(|n| n.tile == here) { here } else { self.main }
    }

    pub fn drawn(&self, view: [f32; 2], clipping_actor: f32) -> Vec<TileId> {
        let limit = range_units(clipping_actor) + DRAW_MARGIN;
        let current = self.current(view[0], view[1]);
        let mut tiles = vec![self.main];
        for n in &self.neighbours {
            if n.tile == current || ((view[0] - n.center[0]).abs() <= limit && (view[1] - n.center[1]).abs() <= limit) {
                tiles.push(n.tile);
            }
        }
        tiles
    }

    pub fn loaded(&self) -> impl Iterator<Item = TileId> + '_ {
        std::iter::once(self.main).chain(self.neighbours.iter().map(|n| n.tile))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn giran() -> TileId {
        TileId { x: 22, y: 22 }
    }

    #[test]
    fn tile_ids_match_tile_name_and_centers() {
        assert_eq!(TileId::of(83400.0, 148000.0), giran());
        assert_eq!(giran().center(), [81920.0, 147456.0]);
        assert_eq!(giran().name(), "22_22");
        assert_eq!(TileId::parse("22_22"), Some(giran()));
        assert_eq!(super::super::tile_name(giran().center()[0], giran().center()[1]), "22_22");
    }

    #[test]
    fn walking_west_requests_tiles_exactly_past_each_8192_step() {
        let mut window = TileWindow::new(giran());
        let [cx, cy] = giran().center();
        let mut requests = Vec::new();
        let mut x = cx;
        while x > cx - 3.0 * TILE_SIZE {
            if let Some(request) = window.step(x, cy) {
                requests.push((x, request.first, request.second, window.sx));
                window.commit(request.direction);
            }
            x -= 512.0;
        }
        let first_at = requests[0].0;
        assert!(first_at < cx - WINDOW_STEP && first_at + 512.0 >= cx - WINDOW_STEP, "{first_at}");
        let sequence: Vec<(i32, i32)> = requests.iter().map(|(_, first, _, _)| (first.x, first.y)).collect();
        assert_eq!(&sequence[..8], &[(21, 22), (21, 22), (21, 22), (21, 22), (20, 22), (20, 22), (20, 22), (20, 22)]);
        let shifts: Vec<i32> = requests.iter().map(|r| r.3).collect();
        assert_eq!(&shifts[..8], &[0, 1, 2, -1, 0, 1, 2, -1]);
        assert!(requests.iter().all(|(_, first, second, _)| first == second));
        let centers: Vec<f32> = (1..=requests.len()).map(|i| cx - i as f32 * WINDOW_STEP).collect();
        assert_eq!(window.cx, *centers.last().unwrap());
    }

    #[test]
    fn walking_east_then_north_adds_the_diagonal_from_the_other_axis_shift() {
        let mut window = TileWindow::new(giran());
        let [cx, cy] = giran().center();
        let east = window.step(cx + WINDOW_STEP + 1.0, cy).unwrap();
        assert_eq!((east.first, east.second, east.direction), (TileId { x: 23, y: 22 }, TileId { x: 23, y: 22 }, Direction::PlusX));
        assert!(window.step(cx + WINDOW_STEP + 1.0, cy).is_none());
        window.commit(Direction::PlusX);
        assert_eq!((window.sx, window.cx), (-1, cx + WINDOW_STEP));
        let north = window.step(window.cx, cy + WINDOW_STEP + 1.0).unwrap();
        assert_eq!(north.direction, Direction::PlusY);
        assert_eq!(north.first, TileId { x: 22, y: 23 });
        assert_eq!(north.second, TileId { x: 23, y: 23 });
        window.commit(Direction::PlusY);
        assert_eq!((window.sy, window.cy), (-1, cy + WINDOW_STEP));
        window.commit(Direction::PlusY);
        assert_eq!(window.sy, 2);
        window.commit(Direction::MinusY);
        assert_eq!(window.sy, -1);
        assert_eq!([cycle_down(-1), cycle_down(0), cycle_down(1), cycle_down(2)], [0, 1, 2, -1]);
        assert_eq!([cycle_up(-1), cycle_up(0), cycle_up(1), cycle_up(2)], [2, -1, 0, 1]);
    }

    #[test]
    fn x_takes_priority_over_y_in_a_request() {
        let mut window = TileWindow::new(giran());
        let [cx, cy] = giran().center();
        let request = window.step(cx - WINDOW_STEP - 1.0, cy + WINDOW_STEP + 1.0).unwrap();
        assert_eq!(request.direction, Direction::MinusX);
        assert_eq!(request.tiles(), vec![TileId { x: 21, y: 22 }]);
    }

    #[test]
    fn attach_keeps_the_four_nearest_neighbours_sorted_by_distance() {
        let mut window = TileWindow::new(giran());
        let [cx, cy] = giran().center();
        let position = [cx - 10000.0, cy + 5000.0];
        assert!(window.attach(TileId { x: 23, y: 22 }, position).is_empty());
        assert!(window.attach(TileId { x: 21, y: 22 }, position).is_empty());
        assert!(window.attach(TileId { x: 22, y: 23 }, position).is_empty());
        assert!(window.attach(TileId { x: 22, y: 21 }, position).is_empty());
        assert!(window.attach(giran(), position).is_empty());
        assert_eq!(window.neighbours.len(), 4);
        let removed = window.attach(TileId { x: 21, y: 23 }, position);
        assert_eq!(removed, vec![TileId { x: 23, y: 22 }]);
        assert_eq!(window.neighbours[0].tile, TileId { x: 21, y: 22 });
        assert_eq!(window.neighbours.len(), MAX_NEIGHBOURS);
    }

    #[test]
    fn purge_drops_neighbours_farther_than_35000_on_either_axis() {
        let mut window = TileWindow::new(giran());
        let [cx, cy] = giran().center();
        window.attach(TileId { x: 21, y: 22 }, [cx, cy]);
        window.attach(TileId { x: 22, y: 23 }, [cx, cy]);
        assert!(window.purge([cx + 2000.0, cy]).is_empty());
        assert_eq!(window.purge([cx + 3000.0, cy]), vec![TileId { x: 21, y: 22 }]);
        assert_eq!(window.neighbours.len(), 1);
        assert_eq!(window.purge([cx, cy - 3000.0]), vec![TileId { x: 22, y: 23 }]);
    }

    #[test]
    fn drawn_discards_neighbours_past_28672_units_except_the_current_one() {
        let mut window = TileWindow::new(giran());
        let [cx, cy] = giran().center();
        window.attach(TileId { x: 21, y: 22 }, [cx, cy]);
        window.attach(TileId { x: 22, y: 23 }, [cx, cy]);
        assert_eq!(window.drawn([cx, cy], 6.0), vec![giran()]);
        let near_west_edge = [cx - TILE_SIZE / 2.0 + 4096.0, cy];
        assert_eq!(window.drawn(near_west_edge, 6.0), vec![giran(), TileId { x: 21, y: 22 }]);
        let inside_west = [cx - TILE_SIZE + 100.0, cy];
        assert_eq!(window.current(inside_west[0], inside_west[1]), TileId { x: 21, y: 22 });
        assert_eq!(window.drawn(inside_west, 0.5), vec![giran(), TileId { x: 21, y: 22 }]);
    }
}
