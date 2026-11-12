use glam::Vec3;
use l2_net::game::{ColosseumFenceInfo, ServerPrimitive as PrimitivePacket};
use std::collections::BTreeMap;

pub const PIECE_CLASS: &str = "Engine.SpawnableStaticMeshActor";
pub const COLUMN_MESH: &str = "Aden_Colosseum.aden_colosseum_fence01";
pub const WALL_MESH: &str = "Aden_Colosseum.aden_colosseum_fence02";
pub const FENCE_GRID: i32 = 100;
pub const FENCE_MARGIN: f32 = 7.5;
pub const WALL_OFFSET: f32 = 50.0;
pub const TRACE_STEP: f32 = 30.0;
pub const TRACE_TRIES: usize = 30;
pub const TRACE_FLAGS: u32 = 0x86;
pub const COLUMN_EXTENT: Vec3 = Vec3::new(15.0, 15.0, 10.5);
pub const WALL_EXTENT: Vec3 = Vec3::new(15.0, 15.0, 8.0);
pub const WALL_YAW: i32 = 0x4000;
pub const VIEW_SECONDS: f32 = 1.0;
const POINT_KIND: u8 = 1;
const LINE_KIND: u8 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct PrimitivePoint {
    pub name: String,
    pub point: Vec3,
    pub color: [u8; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveLine {
    pub name: String,
    pub start: Vec3,
    pub end: Vec3,
    pub color: [u8; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerPrimitive {
    pub name: Option<String>,
    pub location: Vec3,
    pub collision_radius: f32,
    pub collision_height: f32,
    pub points: Vec<PrimitivePoint>,
    pub lines: Vec<PrimitiveLine>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FenceKind {
    Column,
    Wall,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FencePiece {
    pub key: u64,
    pub fence_id: i32,
    pub kind: FenceKind,
    pub spawn: Vec3,
    pub rotation: [i32; 3],
    pub despawn_elapsed: Option<f32>,
}

impl FencePiece {
    pub fn collides(&self) -> bool {
        self.despawn_elapsed.is_none()
    }

    pub fn mesh(&self) -> &'static str {
        match self.kind {
            FenceKind::Column => COLUMN_MESH,
            FenceKind::Wall => WALL_MESH,
        }
    }

    pub fn extent(&self) -> Vec3 {
        match self.kind {
            FenceKind::Column => COLUMN_EXTENT,
            FenceKind::Wall => WALL_EXTENT,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColosseumFence {
    pub id: i32,
    pub state: i32,
    pub location: Vec3,
    pub width: i32,
    pub length: i32,
    pub columns: Option<[u64; 4]>,
    pub walls: Option<(u64, usize)>,
}


#[derive(Debug, Clone, Default, PartialEq)]
pub struct ServerObjects {
    pub primitives: Vec<ServerPrimitive>,
    pub fences: BTreeMap<i32, ColosseumFence>,
    pub pieces: Vec<FencePiece>,
    next_key: u64,
}

pub fn fcolor(plane: [f32; 4]) -> [u8; 4] {
    plane.map(|value| {
        let floor = f64::from(value * 255.0).floor() as i64 as i32;
        floor.clamp(0, 255) as u8
    })
}

pub fn fname_key(name: &str) -> Option<String> {
    (!name.is_empty() && !name.eq_ignore_ascii_case("None")).then(|| name.to_ascii_lowercase())
}

pub fn round_to_grid(value: i32) -> i32 {
    value / FENCE_GRID * FENCE_GRID
}

pub fn column_spawns(location: Vec3, width: i32, length: i32) -> [Vec3; 4] {
    let (width, length) = (round_to_grid(width), round_to_grid(length));
    let half_width = (width / 2) as f32;
    let half_length = (length / 2) as f32;
    let left = location.x - half_width - FENCE_MARGIN;
    let right = half_width + location.x + FENCE_MARGIN;
    let top = half_length + location.y + FENCE_MARGIN;
    let bottom = location.y - half_length - FENCE_MARGIN;
    [
        Vec3::new(left, top, location.z),
        Vec3::new(right, top, location.z),
        Vec3::new(left, bottom, location.z),
        Vec3::new(right, bottom, location.z),
    ]
}

pub fn wall_spawns(location: Vec3, width: i32, length: i32) -> Vec<(Vec3, [i32; 3])> {
    let (width, length) = (round_to_grid(width), round_to_grid(length));
    let half_width = (width / 2) as f32;
    let half_length = (length / 2) as f32;
    let mut spawns = Vec::new();
    for column in 0..width / FENCE_GRID {
        let x = location.x - half_width + (column * FENCE_GRID) as f32 + WALL_OFFSET;
        spawns.push((Vec3::new(x, half_length + location.y + FENCE_MARGIN, location.z), [0; 3]));
        spawns.push((Vec3::new(x, location.y - half_length - FENCE_MARGIN, location.z), [0; 3]));
    }
    for row in 0..length / FENCE_GRID {
        let y = location.y - half_length + (row * FENCE_GRID) as f32 + WALL_OFFSET;
        spawns.push((Vec3::new(location.x - half_width - FENCE_MARGIN, y, location.z), [0, WALL_YAW, 0]));
        spawns.push((Vec3::new(half_width + location.x + FENCE_MARGIN, y, location.z), [0, WALL_YAW, 0]));
    }
    spawns
}

pub fn trace_segments(spawn: Vec3) -> impl Iterator<Item = (Vec3, Vec3)> {
    let (mut start, mut end) = (spawn, spawn);
    (0..TRACE_TRIES).map(move |_| {
        start.z += TRACE_STEP;
        end.z -= TRACE_STEP;
        (start, end)
    })
}

pub fn settle(spawn: Vec3, mut trace: impl FnMut(Vec3, Vec3) -> Option<Vec3>) -> Vec3 {
    let mut location = Vec3::ZERO;
    for (start, end) in trace_segments(spawn) {
        if let Some(hit) = trace(start, end) {
            location = hit;
        }
        if location.z != 0.0 {
            break;
        }
    }
    location
}

impl ServerObjects {
    pub fn clear(&mut self) {
        *self = Self { next_key: self.next_key, ..Self::default() };
    }

    pub fn apply_primitive(&mut self, packet: &PrimitivePacket) {
        if packet.values[5] == 0 {
            return;
        }
        let [x, y, z, radius, height, _] = packet.values;
        let mut primitive = ServerPrimitive {
            name: fname_key(&packet.name).map(|_| packet.name.clone()),
            location: Vec3::new(x as f32, y as f32, z as f32),
            collision_radius: radius as f32,
            collision_height: height as f32,
            points: Vec::new(),
            lines: Vec::new(),
        };
        for entry in &packet.entries {
            let values: Vec<f32> = entry.values.iter().map(|&value| value as f32).collect();
            let name = entry.text.clone().unwrap_or_default();
            match (entry.kind, values.as_slice()) {
                (POINT_KIND, &[r, g, b, a, x, y, z]) => {
                    primitive.points.push(PrimitivePoint { name, point: Vec3::new(x, y, z), color: fcolor([r, g, b, a]) });
                }
                (LINE_KIND, &[r, g, b, a, x1, y1, z1, x2, y2, z2]) => primitive.lines.push(PrimitiveLine {
                    name,
                    start: Vec3::new(x1, y1, z1),
                    end: Vec3::new(x2, y2, z2),
                    color: fcolor([r, g, b, a]),
                }),
                _ => {}
            }
        }
        let key = primitive.name.as_deref().and_then(fname_key);
        let existing = key.as_ref().and_then(|key| {
            self.primitives.iter().position(|old| old.name.as_deref().and_then(fname_key).as_ref() == Some(key))
        });
        match existing {
            Some(index) => {
                let name = self.primitives[index].name.take();
                self.primitives[index] = ServerPrimitive { name, ..primitive };
            }
            None => self.primitives.push(primitive),
        }
    }

    pub fn apply_fence(&mut self, info: &ColosseumFenceInfo) {
        let [id, state, x, y, z, width, length] = info.values;
        let fence = self.fences.entry(id).or_insert(ColosseumFence {
            id,
            state: 0,
            location: Vec3::new(x as f32, y as f32, z as f32),
            width,
            length,
            columns: None,
            walls: None,
        });
        let fence_id = fence.id;
        let mut fence = *fence;
        match state {
            0 => {
                self.destroy_columns(&mut fence);
                self.destroy_walls(&mut fence);
            }
            1 => {
                self.make_columns(&mut fence);
                self.destroy_walls(&mut fence);
            }
            2 => {
                self.make_columns(&mut fence);
                self.make_walls(&mut fence);
            }
            _ => {}
        }
        fence.state = state;
        self.fences.insert(fence_id, fence);
    }

    pub fn delete_object(&mut self, object_id: i32) {
        let Some(mut fence) = self.fences.remove(&object_id) else { return };
        self.destroy_columns(&mut fence);
        self.destroy_walls(&mut fence);
    }

    pub fn tick(&mut self, dt: f32) {
        self.pieces.retain_mut(|piece| match &mut piece.despawn_elapsed {
            Some(elapsed) => {
                *elapsed += dt;
                *elapsed < VIEW_SECONDS
            }
            None => true,
        });
    }

    fn key(&mut self) -> u64 {
        self.next_key = self.next_key.wrapping_add(1);
        self.next_key
    }

    fn make_columns(&mut self, fence: &mut ColosseumFence) {
        if fence.columns.is_some() {
            return;
        }
        fence.width = round_to_grid(fence.width);
        fence.length = round_to_grid(fence.length);
        let spawns = column_spawns(fence.location, fence.width, fence.length);
        let keys = spawns.map(|spawn| {
            let key = self.key();
            self.pieces.push(FencePiece { key, fence_id: fence.id, kind: FenceKind::Column, spawn, rotation: [0; 3], despawn_elapsed: None });
            key
        });
        fence.columns = Some(keys);
    }

    fn make_walls(&mut self, fence: &mut ColosseumFence) {
        if fence.walls.is_some_and(|(_, count)| count > 0) {
            return;
        }
        fence.width = round_to_grid(fence.width);
        fence.length = round_to_grid(fence.length);
        let spawns = wall_spawns(fence.location, fence.width, fence.length);
        let first = self.next_key.wrapping_add(1);
        for (spawn, rotation) in &spawns {
            let key = self.key();
            self.pieces.push(FencePiece { key, fence_id: fence.id, kind: FenceKind::Wall, spawn: *spawn, rotation: *rotation, despawn_elapsed: None });
        }
        fence.walls = Some((first, spawns.len()));
    }

    fn destroy_columns(&mut self, fence: &mut ColosseumFence) {
        if let Some(keys) = fence.columns.take() {
            self.despawn(|piece| keys.contains(&piece.key));
        }
    }

    fn destroy_walls(&mut self, fence: &mut ColosseumFence) {
        if let Some((first, count)) = fence.walls.take() {
            self.despawn(|piece| piece.kind == FenceKind::Wall && piece.key.wrapping_sub(first) < count as u64);
        }
    }

    fn despawn(&mut self, matches: impl Fn(&FencePiece) -> bool) {
        for piece in self.pieces.iter_mut().filter(|piece| piece.despawn_elapsed.is_none() && matches(piece)) {
            piece.despawn_elapsed = Some(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2_net::game::ServerPrimitiveEntry;

    fn packet(name: &str, values: [i32; 6], entries: Vec<ServerPrimitiveEntry>) -> PrimitivePacket {
        PrimitivePacket { name: name.into(), values, entries }
    }

    fn point(text: &str, values: [i32; 7]) -> ServerPrimitiveEntry {
        ServerPrimitiveEntry { kind: 1, text: Some(text.into()), values: values.to_vec() }
    }

    fn line(text: &str, values: [i32; 10]) -> ServerPrimitiveEntry {
        ServerPrimitiveEntry { kind: 2, text: Some(text.into()), values: values.to_vec() }
    }

    #[test]
    fn fcolor_scales_by_255_floors_and_clamps_like_the_engine() {
        assert_eq!(fcolor([0.0, 1.0, 0.5, 0.999]), [0, 255, 127, 254]);
        assert_eq!(fcolor([255.0, -1.0, 2.0, 1.0]), [255, 0, 255, 255]);
        assert_eq!(fcolor([8_421_505.0, 8_421_504.0, -8_421_505.0, 3.0e9]), [0, 255, 0, 255]);
    }

    #[test]
    fn primitive_points_and_lines_keep_order_and_saturate_integer_colors() {
        let mut objects = ServerObjects::default();
        objects.apply_primitive(&packet("Debug", [10, 20, 30, 5, 6, 3], vec![
            point("P", [255, 0, 1, 255, 1, 2, 3]),
            ServerPrimitiveEntry { kind: 9, text: None, values: Vec::new() },
            line("L", [0, 255, 0, 0, 4, 5, 6, 7, 8, 9]),
        ]));
        let primitive = &objects.primitives[0];
        assert_eq!(primitive.location, Vec3::new(10.0, 20.0, 30.0));
        assert_eq!((primitive.collision_radius, primitive.collision_height), (5.0, 6.0));
        assert_eq!(primitive.points, [PrimitivePoint { name: "P".into(), point: Vec3::new(1.0, 2.0, 3.0), color: [255, 0, 255, 255] }]);
        assert_eq!(primitive.lines, [PrimitiveLine {
            name: "L".into(),
            start: Vec3::new(4.0, 5.0, 6.0),
            end: Vec3::new(7.0, 8.0, 9.0),
            color: [0, 255, 0, 0],
        }]);
    }

    #[test]
    fn a_second_primitive_with_the_same_fname_replaces_the_first_and_none_spawns_anew() {
        let mut objects = ServerObjects::default();
        objects.apply_primitive(&packet("Zone", [0, 0, 0, 0, 0, 1], vec![point("a", [1; 7])]));
        objects.apply_primitive(&packet("zone", [5, 0, 0, 0, 0, 1], vec![line("b", [1; 10])]));
        assert_eq!(objects.primitives.len(), 1);
        assert_eq!(objects.primitives[0].name.as_deref(), Some("Zone"));
        assert!(objects.primitives[0].points.is_empty());
        assert_eq!(objects.primitives[0].lines.len(), 1);
        assert_eq!(objects.primitives[0].location.x, 5.0);
        objects.apply_primitive(&packet("", [0, 0, 0, 0, 0, 1], vec![point("c", [1; 7])]));
        objects.apply_primitive(&packet("none", [0, 0, 0, 0, 0, 1], vec![point("d", [1; 7])]));
        assert_eq!(objects.primitives.len(), 3);
        objects.apply_primitive(&packet("Zone", [9, 9, 9, 0, 0, 0], Vec::new()));
        assert_eq!(objects.primitives[0].location.x, 5.0);
    }

    #[test]
    fn columns_sit_outside_the_rounded_rectangle_by_the_margin() {
        let corners = column_spawns(Vec3::new(1000.0, 2000.0, -50.0), 599, 401);
        assert_eq!(corners, [
            Vec3::new(1000.0 - 250.0 - 7.5, 2000.0 + 200.0 + 7.5, -50.0),
            Vec3::new(1000.0 + 250.0 + 7.5, 2000.0 + 200.0 + 7.5, -50.0),
            Vec3::new(1000.0 - 250.0 - 7.5, 2000.0 - 200.0 - 7.5, -50.0),
            Vec3::new(1000.0 + 250.0 + 7.5, 2000.0 - 200.0 - 7.5, -50.0),
        ]);
        assert_eq!(round_to_grid(-150), -100);
    }

    #[test]
    fn walls_fill_each_side_every_hundred_units_and_turn_on_the_short_sides() {
        let walls = wall_spawns(Vec3::new(0.0, 0.0, 10.0), 300, 200);
        assert_eq!(walls.len(), 2 * 3 + 2 * 2);
        assert_eq!(walls[0], (Vec3::new(-100.0, 107.5, 10.0), [0; 3]));
        assert_eq!(walls[1], (Vec3::new(-100.0, -107.5, 10.0), [0; 3]));
        assert_eq!(walls[4], (Vec3::new(100.0, 107.5, 10.0), [0; 3]));
        assert_eq!(walls[6], (Vec3::new(-157.5, -50.0, 10.0), [0, WALL_YAW, 0]));
        assert_eq!(walls[7], (Vec3::new(157.5, -50.0, 10.0), [0, WALL_YAW, 0]));
        assert_eq!(walls[9], (Vec3::new(157.5, 50.0, 10.0), [0, WALL_YAW, 0]));
    }

    #[test]
    fn settle_widens_the_vertical_trace_until_a_contact_with_nonzero_height() {
        let spawn = Vec3::new(1.0, 2.0, 100.0);
        let mut seen = Vec::new();
        let hit = settle(spawn, |start, end| {
            seen.push((start.z, end.z));
            (seen.len() == 3).then_some(Vec3::new(1.0, 2.0, 25.5))
        });
        assert_eq!(hit, Vec3::new(1.0, 2.0, 25.5));
        assert_eq!(seen, [(130.0, 70.0), (160.0, 40.0), (190.0, 10.0)]);
        let mut tries = 0;
        assert_eq!(settle(spawn, |_, _| { tries += 1; Some(Vec3::new(4.0, 5.0, 0.0)) }), Vec3::new(4.0, 5.0, 0.0));
        assert_eq!(tries, TRACE_TRIES);
        assert_eq!(settle(spawn, |_, _| None), Vec3::ZERO);
    }

    #[test]
    fn fence_states_build_and_fade_out_columns_and_walls() {
        let mut objects = ServerObjects::default();
        objects.apply_fence(&ColosseumFenceInfo { values: [7, 2, 0, 0, 0, 250, 150] });
        let fence = objects.fences[&7];
        assert_eq!((fence.width, fence.length, fence.state), (200, 100, 2));
        assert_eq!(objects.pieces.iter().filter(|piece| piece.kind == FenceKind::Column).count(), 4);
        assert_eq!(objects.pieces.iter().filter(|piece| piece.kind == FenceKind::Wall).count(), 2 * 2 + 2);
        objects.apply_fence(&ColosseumFenceInfo { values: [7, 1, 900, 900, 900, 900, 900] });
        assert_eq!(objects.fences[&7].location, Vec3::ZERO);
        assert!(objects.pieces.iter().filter(|piece| piece.kind == FenceKind::Wall).all(|piece| piece.despawn_elapsed == Some(0.0)));
        assert!(objects.pieces.iter().filter(|piece| piece.kind == FenceKind::Column).all(|piece| piece.despawn_elapsed.is_none()));
        assert!(objects.pieces.iter().filter(|piece| piece.kind == FenceKind::Wall).all(|piece| !piece.collides()));
        assert!(objects.pieces.iter().filter(|piece| piece.kind == FenceKind::Column).all(FencePiece::collides));
        objects.tick(0.5);
        assert_eq!(objects.pieces.len(), 10);
        objects.tick(0.5);
        assert_eq!(objects.pieces.len(), 4);
        objects.apply_fence(&ColosseumFenceInfo { values: [7, 1, 0, 0, 0, 0, 0] });
        assert_eq!(objects.pieces.len(), 4);
        objects.apply_fence(&ColosseumFenceInfo { values: [7, 5, 0, 0, 0, 0, 0] });
        assert_eq!(objects.fences[&7].state, 5);
        assert_eq!(objects.pieces.len(), 4);
        objects.delete_object(7);
        assert!(objects.fences.is_empty());
        assert!(objects.pieces.iter().all(|piece| piece.despawn_elapsed.is_some()));
        objects.tick(1.0);
        assert!(objects.pieces.is_empty());
    }
}
