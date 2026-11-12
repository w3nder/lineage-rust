use glam::{Mat4, Vec2, Vec3};
use killer_assets::scene::{MeshDraw, Pass, Scene};
use killer_world::Entity;

pub const WALKABLE_FLOOR_Z: f32 = 0.7;
pub const ENGINE_PLACE_ABOVE: f32 = 20.0;
pub const ENGINE_PLACE_BELOW: f32 = 10.0;
pub const ENGINE_PLACE_STEP: f32 = 30.0;
pub const ENGINE_PLACE_RETRIES: u32 = 30;
pub const MIN_FLOOR_DIST: f32 = 1.9;
pub const MAX_FLOOR_DIST: f32 = 2.4;
pub const FLOOR_HOVER: f32 = (MIN_FLOOR_DIST + MAX_FLOOR_DIST) * 0.5;
const CELL: f32 = 128.0;
const EDGE_TOLERANCE: f32 = -1e-5;

#[derive(Debug, Default, Clone)]
pub struct GroundGrid {
    origin: Vec2,
    columns: usize,
    rows: usize,
    cell_start: Vec<u32>,
    cell_triangles: Vec<u32>,
    triangles: Vec<[Vec3; 3]>,
}

impl GroundGrid {
    pub fn build(scene: &Scene) -> Self {
        let draws: Vec<&MeshDraw> = scene.draws.iter().filter(|d| !matches!(d.pass, Pass::Translucent | Pass::Projector) && !scene.zone(d.zone).is_sky).collect();
        let triangles = walkable_triangles(&draws);
        let (min, max) = triangles.iter().flatten().fold((Vec2::INFINITY, Vec2::NEG_INFINITY), |(lo, hi), p| (lo.min(p.truncate()), hi.max(p.truncate())));
        if triangles.is_empty() || !min.is_finite() || !max.is_finite() {
            return Self::default();
        }
        let columns = ((max.x - min.x) / CELL).floor() as usize + 1;
        let rows = ((max.y - min.y) / CELL).floor() as usize + 1;
        let cell_of = |p: Vec2| (((p.x - min.x) / CELL).floor() as usize, ((p.y - min.y) / CELL).floor() as usize);
        let spans: Vec<(usize, usize, usize, usize)> = triangles
            .iter()
            .map(|tri| {
                let lo = tri.iter().fold(Vec2::INFINITY, |lo, p| lo.min(p.truncate()));
                let hi = tri.iter().fold(Vec2::NEG_INFINITY, |hi, p| hi.max(p.truncate()));
                let (c0, r0) = cell_of(lo);
                let (c1, r1) = cell_of(hi);
                (c0, r0, c1.min(columns - 1), r1.min(rows - 1))
            })
            .collect();
        let mut cell_start = vec![0u32; columns * rows + 1];
        for &(c0, r0, c1, r1) in &spans {
            for r in r0..=r1 {
                for c in c0..=c1 {
                    cell_start[r * columns + c + 1] += 1;
                }
            }
        }
        for i in 1..cell_start.len() {
            cell_start[i] += cell_start[i - 1];
        }
        let mut fill = cell_start.clone();
        let mut cell_triangles = vec![0u32; cell_start[cell_start.len() - 1] as usize];
        for (index, &(c0, r0, c1, r1)) in spans.iter().enumerate() {
            for r in r0..=r1 {
                for c in c0..=c1 {
                    let slot = &mut fill[r * columns + c];
                    cell_triangles[*slot as usize] = index as u32;
                    *slot += 1;
                }
            }
        }
        Self { origin: min, columns, rows, cell_start, cell_triangles, triangles }
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    pub fn ground_at(&self, x: f32, y: f32, z_from: f32, z_to: f32) -> Option<f32> {
        if self.triangles.is_empty() || z_to > z_from {
            return None;
        }
        let column = (x - self.origin.x) / CELL;
        let row = (y - self.origin.y) / CELL;
        if column < 0.0 || row < 0.0 || column >= self.columns as f32 || row >= self.rows as f32 {
            return None;
        }
        let cell = row as usize * self.columns + column as usize;
        let point = Vec2::new(x, y);
        let mut best: Option<f32> = None;
        for &index in &self.cell_triangles[self.cell_start[cell] as usize..self.cell_start[cell + 1] as usize] {
            let Some(z) = height_on(&self.triangles[index as usize], point) else { continue };
            if z >= z_to && z <= z_from && best.is_none_or(|b| z > b) {
                best = Some(z);
            }
        }
        best
    }

    pub fn placement_floor(&self, x: f32, y: f32, feet: f32, collision_height: f32) -> Option<f32> {
        (0..ENGINE_PLACE_RETRIES).find_map(|retry| {
            let widen = ENGINE_PLACE_STEP * retry as f32;
            self.ground_at(x, y, feet + ENGINE_PLACE_ABOVE + widen, feet - ENGINE_PLACE_BELOW - collision_height - widen)
        })
    }

    pub fn place(&self, entity: &mut Entity) -> bool {
        let feet = entity.position.z - entity.collision_height;
        match self.placement_floor(entity.position.x, entity.position.y, feet, entity.collision_height) {
            Some(floor) => {
                entity.position.z = floor + entity.collision_height + FLOOR_HOVER;
                true
            }
            None => false,
        }
    }
}

fn draw_walkable(draw: &MeshDraw, out: &mut Vec<[Vec3; 3]>) {
    let triangles = draw.indices.as_chunks::<3>().0;
    let facings: Vec<Vec3> = triangles
        .iter()
        .map(|triangle| triangle.iter().map(|&index| Vec3::from(draw.vertices[index as usize].normal)).sum::<Vec3>())
        .collect();
    let mut positions: Vec<Vec3> = Vec::with_capacity(draw.vertices.len());
    for index in 0..draw.instances.len().max(1) {
        positions.clear();
        match draw.instances.get(index) {
            Some(instance) => {
                let world = Mat4::from_cols_array_2d(&instance.world);
                positions.extend(draw.vertices.iter().map(|vertex| world.transform_point3(Vec3::from(vertex.position))));
            }
            None => positions.extend(draw.vertices.iter().map(|vertex| Vec3::from(vertex.position))),
        }
        for (triangle, &facing) in triangles.iter().zip(&facings) {
            let [a, b, c] = triangle.map(|vertex| positions[vertex as usize]);
            if walkable(a, b, c, facing) {
                out.push([a, b, c]);
            }
        }
    }
}

fn walkable_triangles(draws: &[&MeshDraw]) -> Vec<[Vec3; 3]> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let workers = std::thread::available_parallelism().map_or(1, usize::from).min(draws.len().max(1));
        if workers > 1 {
            let weights: Vec<usize> = draws.iter().map(|draw| draw.world_index_count()).collect();
            let share = weights.iter().sum::<usize>().div_ceil(workers).max(1);
            let mut chunks: Vec<&[&MeshDraw]> = Vec::new();
            let (mut start, mut load) = (0, 0);
            for (index, weight) in weights.iter().enumerate() {
                load += weight;
                if load >= share {
                    chunks.push(&draws[start..=index]);
                    start = index + 1;
                    load = 0;
                }
            }
            if start < draws.len() {
                chunks.push(&draws[start..]);
            }
            let parts: Vec<Vec<[Vec3; 3]>> = std::thread::scope(|scope| {
                let handles: Vec<_> = chunks
                    .iter()
                    .map(|chunk| {
                        scope.spawn(move || {
                            let mut out = Vec::new();
                            for draw in *chunk {
                                draw_walkable(draw, &mut out);
                            }
                            out
                        })
                    })
                    .collect();
                handles.into_iter().map(|handle| handle.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic))).collect()
            });
            return parts.concat();
        }
    }
    let mut out = Vec::new();
    for draw in draws {
        draw_walkable(draw, &mut out);
    }
    out
}

pub(crate) fn walkable(a: Vec3, b: Vec3, c: Vec3, facing: Vec3) -> bool {
    let mut normal = (b - a).cross(c - a);
    let length = normal.length();
    if length <= 0.0 {
        return false;
    }
    if normal.dot(facing) < 0.0 {
        normal = -normal;
    }
    normal.z / length >= WALKABLE_FLOOR_Z
}

fn height_on(tri: &[Vec3; 3], p: Vec2) -> Option<f32> {
    let [a, b, c] = *tri;
    let v0 = (b - a).truncate();
    let v1 = (c - a).truncate();
    let v2 = p - a.truncate();
    let det = v0.x * v1.y - v1.x * v0.y;
    if det.abs() <= f32::EPSILON {
        return None;
    }
    let v = (v2.x * v1.y - v1.x * v2.y) / det;
    let w = (v0.x * v2.y - v2.x * v0.y) / det;
    let u = 1.0 - v - w;
    (u >= EDGE_TOLERANCE && v >= EDGE_TOLERANCE && w >= EDGE_TOLERANCE).then_some(a.z * u + b.z * v + c.z * w)
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_assets::scene::{DrawKind, MeshDraw, Vertex, ZoneLighting, texture_material};
    use killer_world::world::WHITE_NAME_COLOR;
    use killer_world::{AnimPlayer, EntityKind};

    fn vertex(position: [f32; 3]) -> Vertex {
        Vertex { position, normal: [0.0, 0.0, 1.0], uv: [0.0; 2], mask_uv: [0.0; 2], color: [1.0; 4] }
    }

    fn ceiling(z: f32, half: f32) -> MeshDraw {
        let mut draw = plane(z, half, Pass::Opaque, 0);
        for v in &mut draw.vertices {
            v.normal = [0.0, 0.0, -1.0];
        }
        draw
    }

    fn draw(vertices: Vec<Vertex>, indices: Vec<u32>, pass: Pass, zone: u8) -> MeshDraw {
        MeshDraw { instances: Vec::new(), vertices, indices, material: texture_material(""), mask: None, kind: DrawKind::StaticMesh, sun_affect: true, pass, zone, lightmap: None, modulate_2x: true, sector: None, fade: None, tint: None, actor: None }
    }

    fn plane(z: f32, half: f32, pass: Pass, zone: u8) -> MeshDraw {
        let v = vec![vertex([-half, -half, z]), vertex([half, -half, z]), vertex([half, half, z]), vertex([-half, half, z])];
        draw(v, vec![0, 1, 2, 0, 2, 3], pass, zone)
    }

    fn wall(x: f32) -> MeshDraw {
        let v = vec![vertex([x, -500.0, -500.0]), vertex([x, 500.0, -500.0]), vertex([x, 500.0, 500.0]), vertex([x, -500.0, 500.0])];
        draw(v, vec![0, 1, 2, 0, 2, 3], Pass::Opaque, 0)
    }

    fn scene() -> Scene {
        Scene {
            draws: vec![plane(0.0, 1000.0, Pass::Opaque, 0), plane(100.0, 300.0, Pass::Opaque, 0), plane(150.0, 300.0, Pass::Translucent, 0), plane(200.0, 300.0, Pass::Opaque, 1), wall(50.0), ceiling(60.0, 300.0)],
            zones: vec![ZoneLighting::default(), ZoneLighting { is_sky: true, ..Default::default() }],
            ..Default::default()
        }
    }

    #[test]
    fn instanced_decorations_preserve_ground_cells_and_floor_heights() {
        let mut compact = scene();
        for draw in &mut compact.draws {
            draw.instances = (0..3).map(|i| killer_assets::scene::DecorationInstance {
                world: glam::Mat4::from_scale_rotation_translation(
                    Vec3::new(-1.0, 0.5, 2.0), glam::Quat::from_rotation_z(i as f32 * 0.7),
                    Vec3::new(i as f32 * 1100.0, i as f32 * -700.0, i as f32 * 20.0),
                ).to_cols_array_2d(), mask_uv: [0.0; 2], color: [1.0; 4],
            }).collect();
        }
        let mut expanded = compact.clone();
        for draw in &mut expanded.draws {
            let vertices = draw.world_vertices().collect();
            let indices = draw.world_indices().collect();
            draw.vertices = vertices;
            draw.indices = indices;
            draw.instances.clear();
        }
        let actual = GroundGrid::build(&compact);
        let expected = GroundGrid::build(&expanded);
        assert_eq!(actual.origin, expected.origin);
        assert_eq!((actual.columns, actual.rows), (expected.columns, expected.rows));
        assert_eq!(actual.triangles, expected.triangles);
        assert_eq!(actual.cell_start, expected.cell_start);
        assert_eq!(actual.cell_triangles, expected.cell_triangles);
        assert!(!actual.triangles.is_empty());
    }

    #[test]
    fn the_highest_floor_below_the_step_reach_wins() {
        let grid = GroundGrid::build(&scene());
        assert_eq!(grid.triangle_count(), 4);
        assert_eq!(grid.ground_at(10.0, 10.0, 110.0, -50.0), Some(100.0));
        assert_eq!(grid.ground_at(10.0, 10.0, 50.0, -50.0), Some(0.0));
        assert_eq!(grid.ground_at(10.0, 10.0, 110.0, 50.0), Some(100.0));
        assert_eq!(grid.ground_at(10.0, 10.0, 30.0, 20.0), None);
        assert_eq!(grid.ground_at(500.0, 500.0, 300.0, -50.0), Some(0.0));
    }

    #[test]
    fn translucent_sky_wall_and_ceiling_triangles_are_not_floor() {
        let grid = GroundGrid::build(&scene());
        assert_eq!(grid.ground_at(10.0, 10.0, 1000.0, 120.0), None);
        assert_eq!(grid.ground_at(50.0, 0.0, 400.0, 110.0), None);
        assert_eq!(grid.ground_at(10.0, 10.0, 70.0, 50.0), None);
    }

    #[test]
    fn points_outside_the_grid_or_on_a_sloped_floor_are_handled() {
        let grid = GroundGrid::build(&scene());
        assert_eq!(grid.ground_at(5000.0, 0.0, 100.0, -100.0), None);
        assert_eq!(grid.ground_at(-1000.0, -1000.0, 100.0, -100.0), Some(0.0));
        let v = vec![vertex([0.0, 0.0, 0.0]), vertex([100.0, 0.0, 0.0]), vertex([100.0, 100.0, 50.0]), vertex([0.0, 100.0, 50.0])];
        let slope = Scene { draws: vec![draw(v, vec![0, 1, 2, 0, 2, 3], Pass::Opaque, 0)], zones: vec![ZoneLighting::default()], ..Default::default() };
        let grid = GroundGrid::build(&slope);
        let z = grid.ground_at(50.0, 40.0, 100.0, -100.0).unwrap();
        assert!((z - 20.0).abs() < 1e-3, "{z}");
        assert_eq!(GroundGrid::build(&Scene::default()).ground_at(0.0, 0.0, 10.0, -10.0), None);
    }

    fn entity(position: Vec3) -> Entity {
        Entity {
            item_origin: None,
            abnormal_effect: 0,
            object_id: 1,
            kind: EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: false },
            name: String::new(),
            title: String::new(),
            position,
            heading: 0,
            dest: None,
            stop_heading: None,
            desired_heading: 0,
            turn_rate: None,
            run_speed: 120,
            walk_speed: 60,
            speed_multiplier: 1.0,
            attack_speed_multiplier: 1.0,
            m_atk_speed: 0,
            skill_aim: [0; 3],
            p_atk_speed: 0,
            speed_bonus: 0.0,
            speed_bonus_left: 0.0,
            blocked: false,
            follow: None,
            running: true,
            sitting: false,
            fake_dead: false,
            dead: false,
            walk_anim_frame_in_move: 12,
            hp: None,
            mp: None,
            cp: None,
            collision_radius: 8.0,
            collision_height: 24.0,
            base_collision: [0.0; 2],
            karma: 0,
            pvp_flag: 0,
            level: 0,
            exp: 0,
            relation: 0, auto_attack: false, surrender_war_id: 0,
            target: None,
            casting: None,
            magic: Default::default(),
            anim: AnimPlayer::default(),
            spawned: 0,
            name_color: WHITE_NAME_COLOR,
            recommendations: 0,
            name_blink: false,
            name_blink_elapsed: 0.0,
            clan_id: 0,
            clan_crest_id: 0,
            ally_id: 0,
            ally_crest_id: 0,
            store_type: 0,
            in_combat: false,
            fishing: 0,
            mount: None,
            vehicle: None,
            chair: None,
            rotation: None,
        }
    }

    #[test]
    fn place_keeps_the_collision_center_a_height_and_the_hover_above_the_floor() {
        let grid = GroundGrid::build(&scene());
        let mut e = entity(Vec3::new(10.0, 10.0, 100.0 + 24.0 - 3.0));
        assert!(grid.place(&mut e));
        assert_eq!(e.position, Vec3::new(10.0, 10.0, 124.0 + FLOOR_HOVER));
        let mut e = entity(Vec3::new(10.0, 10.0, 100.0 + 24.0 + ENGINE_PLACE_ABOVE - 1.0));
        assert!(grid.place(&mut e));
        assert_eq!(e.position.z, 124.0 + FLOOR_HOVER);
        assert_eq!(grid.placement_floor(10.0, 10.0, 100.0 + ENGINE_PLACE_ABOVE - 1.0, 24.0), Some(100.0));
        assert_eq!(grid.placement_floor(5000.0, 0.0, 0.0, 24.0), None);
    }

    #[test]
    fn a_placed_pawn_hovers_between_the_engine_floor_distances_and_places_again_in_place() {
        let grid = GroundGrid::build(&scene());
        let mut e = entity(Vec3::new(10.0, 10.0, 130.0));
        assert!(grid.place(&mut e));
        let bottom = e.position.z - e.collision_height;
        assert!(bottom - 100.0 >= MIN_FLOOR_DIST && bottom - 100.0 <= MAX_FLOOR_DIST, "{bottom}");
        assert_eq!(FLOOR_HOVER, 2.15);
        let before = e.position;
        assert!(grid.place(&mut e));
        assert_eq!(e.position, before);
    }

    #[test]
    fn place_lifts_an_entity_buried_below_the_floor_and_widens_the_search_by_30_per_retry() {
        let grid = GroundGrid::build(&scene());
        let mut e = entity(Vec3::new(10.0, 10.0, 100.0 + 24.0 - 40.0));
        assert!(grid.place(&mut e));
        assert_eq!(e.position, Vec3::new(10.0, 10.0, 124.0 + FLOOR_HOVER));
        let mut e = entity(Vec3::new(500.0, 500.0, 24.0 - 40.0));
        assert!(grid.place(&mut e));
        assert_eq!(e.position, Vec3::new(500.0, 500.0, 24.0 + FLOOR_HOVER));
        let mut e = entity(Vec3::new(10.0, 10.0, 24.0 - 40.0));
        assert!(grid.place(&mut e));
        assert_eq!(e.position.z, 24.0 + FLOOR_HOVER);
        let mut e = entity(Vec3::new(10.0, 10.0, 100.0 + 24.0 - 600.0));
        assert!(grid.place(&mut e));
        assert_eq!(e.position.z, 24.0 + FLOOR_HOVER);
        let mut e = entity(Vec3::new(10.0, 10.0, 100.0 + 24.0 + 700.0));
        assert!(grid.place(&mut e));
        assert_eq!(e.position.z, 124.0 + FLOOR_HOVER);
        let widest = ENGINE_PLACE_ABOVE + ENGINE_PLACE_STEP * (ENGINE_PLACE_RETRIES - 1) as f32;
        let mut e = entity(Vec3::new(10.0, 10.0, 24.0 - widest - 1.0));
        assert!(!grid.place(&mut e));
        assert_eq!(e.position.z, 24.0 - widest - 1.0);
        let mut e = entity(Vec3::new(5000.0, 0.0, 0.0));
        assert!(!grid.place(&mut e));
    }
}

#[cfg(test)]
mod giran {
    use super::*;
    use killer_assets::client::Client;
    use std::path::PathBuf;

    #[test]
    #[ignore]
    fn the_giran_player_start_hangs_84_units_above_its_floor() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let mut client = Client::open(&root).unwrap();
        let loaded = client.load_level("22_22").unwrap();
        let scene = killer_assets::scene::build_scene_with(&mut client, &loaded.level, &killer_assets::options::ClientOptions::default(), killer_assets::scene::Neighbours::default());
        let start = loaded.level.actors.iter().find(|a| a.class.eq_ignore_ascii_case("PlayerStart")).unwrap();
        let [x, y, z] = start.placement.location;
        let grid = GroundGrid::build(&scene);
        assert_eq!(grid.ground_at(x, y, z, z - 600.0), Some(-3417.0));
        assert_eq!(grid.ground_at(x, y, z + 400.0, z - 600.0), Some(-3229.0));
        assert_eq!(crate::trace::TraceMesh::from_scene(&scene).nearest_hit(Vec3::new(x, y, z), Vec3::new(x, y, z - 600.0)), Some(84.0));
    }
}
