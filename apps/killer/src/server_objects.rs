use crate::items::{Geometry, views};
use crate::loader::LoadRequest;
use crate::ui2d::PrimitiveMark;
use glam::Vec3;
use killer_assets::client::Client;
use killer_assets::collision::{resources::Resources, world::{Spawned, View, trace_spawned}};
use killer_assets::options::ClippingRange;
use killer_assets::pickup::TraceQuery;
use killer_assets::scene::MeshDraw;
use killer_assets::scene::props::{SpawnedMesh, build_spawned};
use killer_assets::textures::{BankSource, TextureBank};
use killer_render::lines::Line;
use killer_render::renderer::Renderer;
use killer_ui::color::Argb;
use killer_world::World;
use killer_world::server_objects::{FencePiece, PIECE_CLASS, TRACE_FLAGS, settle};
use std::collections::HashMap;

pub const SPAWNED_ACTOR_BASE: u32 = 0x8000_0000;

pub fn lines(world: &World) -> Vec<Line> {
    world.server_objects.primitives.iter().flat_map(|primitive| &primitive.lines)
        .map(|line| Line { start: line.start, end: line.end, color: line.color }).collect()
}

pub fn argb(color: [u8; 4]) -> Argb {
    let [r, g, b, a] = color.map(u32::from);
    Argb(a << 24 | r << 16 | g << 8 | b)
}

pub fn marks(world: &World, mut project: impl FnMut(Vec3) -> Option<[f32; 2]>) -> Vec<PrimitiveMark> {
    let mut marks = Vec::new();
    for primitive in &world.server_objects.primitives {
        for point in &primitive.points {
            if let Some(screen) = project(point.point) {
                marks.push(PrimitiveMark { screen, text: point.name.clone(), color: argb(point.color), point: true });
            }
        }
        for line in &primitive.lines {
            if line.name.is_empty() { continue; }
            if let Some(screen) = project((line.start + line.end) / 2.0) {
                marks.push(PrimitiveMark { screen, text: line.name.clone(), color: argb(line.color), point: false });
            }
        }
    }
    marks
}

pub struct PieceRequest {
    pub key: u64,
    pub mesh: &'static str,
    pub spawn: Vec3,
    pub rotation: [i32; 3],
    pub extent: Vec3,
    pub views: Vec<View>,
}

pub struct Request {
    pub main_tile: String,
    pub clipping: ClippingRange,
    pub pieces: Vec<PieceRequest>,
    pub colliders: Vec<Spawned>,
}

pub struct Built {
    pub tile: String,
    pub location: Vec3,
    pub draws: Vec<MeshDraw>,
}

pub struct Batch {
    pub pieces: Vec<(u64, Result<Built, String>)>,
    pub bank: TextureBank,
}

pub fn actor_id(key: u64) -> u32 {
    SPAWNED_ACTOR_BASE | (key as u32 & !SPAWNED_ACTOR_BASE)
}

pub fn collider(mesh: &str, location: Vec3, rotation: [i32; 3]) -> Spawned {
    Spawned { class: PIECE_CLASS.into(), mesh: mesh.into(), placement: ue2_level::Placement { location: location.to_array(), rotation } }
}

pub fn build(client: &mut Client, resources: &mut Resources, request: &Request) -> Batch {
    let mut bank = TextureBank::new();
    let mut placed = request.colliders.clone();
    let pieces = request.pieces.iter().map(|piece| {
        let built = build_piece(client, resources, request, piece, &placed);
        if let Ok(built) = &built {
            placed.push(collider(piece.mesh, built.location, piece.rotation));
            for path in built.draws.iter().flat_map(|draw| draw.material.texture_paths()) { bank.texture(client.catalog(), path); }
        }
        (piece.key, built)
    }).collect();
    Batch { pieces, bank }
}

fn build_piece(client: &mut Client, resources: &mut Resources, request: &Request, piece: &PieceRequest, placed: &[Spawned]) -> Result<Built, String> {
    let mut error = None;
    let location = settle(piece.spawn, |start, end| {
        if error.is_some() { return None; }
        match trace_spawned(client, resources, &piece.views, TraceQuery { start, end, extent: piece.extent, flags: TRACE_FLAGS }, &[], placed) {
            Ok(hit) => hit.map(|selection| selection.hit.position),
            Err(reason) => { error = Some(reason); None }
        }
    });
    if let Some(error) = error { return Err(error); }
    let tile = killer_assets::tiles::TileId::of(location.x, location.y).name();
    let view = piece.views.iter().find(|view| view.name == tile).or(piece.views.first()).ok_or_else(|| "nenhum nível carregado".to_string())?;
    let spawned = SpawnedMesh { mesh: piece.mesh.into(), placement: ue2_level::Placement { location: location.to_array(), rotation: piece.rotation }, actor_id: actor_id(piece.key) };
    let draws = build_spawned(client, &view.level, &request.clipping, std::slice::from_ref(&spawned)).into_iter().flatten().filter(|draw| !draw.indices.is_empty()).collect::<Vec<_>>();
    if draws.is_empty() { return Err(format!("malha original não carregada: {}", piece.mesh)); }
    resources.bounds(client, piece.mesh)?;
    resources.mesh(client, piece.mesh)?;
    let tile = if view.name == request.main_tile { String::new() } else { view.name.clone() };
    Ok(Built { tile, location, draws })
}

#[derive(Default)]
struct Piece {
    pending: bool,
    failed: bool,
    installed: bool,
    despawning: bool,
    location: Option<Vec3>,
}

#[derive(Default)]
pub struct Fences {
    pieces: HashMap<u64, Piece>,
}

impl Fences {
    pub fn clear(&mut self) {
        self.pieces.clear();
    }

    pub fn sync(&mut self, world: &World, renderer: &mut Renderer) {
        self.pieces.retain(|key, piece| {
            let Some(current) = world.server_objects.pieces.iter().find(|current| current.key == *key) else {
                renderer.remove_spawned(*key);
                return false;
            };
            if current.despawn_elapsed.is_some() && !piece.despawning {
                piece.despawning = true;
                renderer.despawn(*key);
            }
            if piece.installed && !renderer.has_spawned(*key) { piece.installed = false; }
            true
        });
        for current in world.server_objects.pieces.iter().filter(|current| current.despawn_elapsed.is_none()) {
            self.pieces.entry(current.key).or_default();
        }
    }

    pub fn request_with(&mut self, world: &World, view: &impl Geometry, renderer: &Renderer, clipping: ClippingRange, send: impl FnOnce(LoadRequest) -> bool) {
        let wanted: Vec<_> = world.server_objects.pieces.iter().filter(|current| current.despawn_elapsed.is_none()
            && self.pieces.get(&current.key).is_some_and(|piece| !piece.pending && !piece.failed && !piece.installed)).collect();
        if wanted.is_empty() { return; }
        let pieces = wanted.iter().map(|current| PieceRequest { key: current.key, mesh: current.mesh(), spawn: current.spawn, rotation: current.rotation,
            extent: current.extent(), views: views(view, Some(renderer), current.spawn) }).collect();
        let keys: Vec<u64> = wanted.iter().map(|current| current.key).collect();
        let colliders = self.colliders_except(world, &keys);
        let request = Request { main_tile: view.neighbours().window.main.name(), clipping, pieces, colliders };
        if send(LoadRequest::Fence { request }) {
            for key in keys { if let Some(piece) = self.pieces.get_mut(&key) { piece.pending = true; } }
        }
    }

    pub fn colliders(&self, world: &World) -> Vec<Spawned> {
        self.colliders_except(world, &[])
    }

    fn colliders_except(&self, world: &World, skip: &[u64]) -> Vec<Spawned> {
        world.server_objects.pieces.iter().filter(|current| current.collides() && !skip.contains(&current.key))
            .filter_map(|current| self.placed(current)).collect()
    }

    fn placed(&self, current: &FencePiece) -> Option<Spawned> {
        let location = self.pieces.get(&current.key)?.location?;
        Some(collider(current.mesh(), location, current.rotation))
    }

    pub fn retry_pending(&mut self) {
        for piece in self.pieces.values_mut() { piece.pending = false; }
    }

    pub fn install(&mut self, mut batch: Batch, renderer: &mut Renderer, client: &mut Client) {
        for (key, built) in batch.pieces {
            let Some(piece) = self.pieces.get_mut(&key).filter(|piece| piece.pending) else { continue };
            piece.pending = false;
            match built {
                Ok(built) => {
                    piece.location = Some(built.location);
                    piece.installed = renderer.add_spawned(key, &built.tile, &built.draws, &mut BankSource { bank: &mut batch.bank, catalog: client.catalog() });
                    piece.failed = !piece.installed;
                    if piece.installed && piece.despawning { renderer.despawn(key); }
                    if piece.failed { log::warn!("cerca do coliseu: peça {key} em {:?} sem tile carregado ({:?})", built.location, built.tile); }
                }
                Err(reason) => {
                    piece.failed = true;
                    log::warn!("cerca do coliseu: peça {key}: {reason}");
                }
            }
        }
        client.catalog().release_files();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2_net::game::{ServerPrimitive, ServerPrimitiveEntry};
    use killer_world::WorldEvent;

    #[test]
    fn marks_and_lines_follow_the_primitive_branch_of_fdynamicactor_render() {
        let mut world = World::default();
        world.apply_world(&WorldEvent::ServerPrimitive(ServerPrimitive { name: "Debug".into(), values: [0, 0, 0, 0, 0, 3], entries: vec![
            ServerPrimitiveEntry { kind: 1, text: Some("Ponto".into()), values: vec![1, 0, 0, 1, 10, 20, 30] },
            ServerPrimitiveEntry { kind: 2, text: Some("Linha".into()), values: vec![0, 0, 1, 0, 0, 0, 0, 10, 20, 30] },
            ServerPrimitiveEntry { kind: 2, text: Some(String::new()), values: vec![0, 1, 0, 1, 0, 0, 0, 1, 1, 1] },
        ] }));
        let lines = lines(&world);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], Line { start: Vec3::ZERO, end: Vec3::new(10.0, 20.0, 30.0), color: [0, 0, 255, 0] });
        let mut projected = Vec::new();
        let marks = marks(&world, |point| { projected.push(point); Some([point.x, point.y]) });
        assert_eq!(projected, [Vec3::new(10.0, 20.0, 30.0), Vec3::new(5.0, 10.0, 15.0)]);
        assert_eq!(marks[0], PrimitiveMark { screen: [10.0, 20.0], text: "Ponto".into(), color: Argb(0xFFFF_0000), point: true });
        assert_eq!(marks[1], PrimitiveMark { screen: [5.0, 10.0], text: "Linha".into(), color: Argb(0x0000_00FF), point: false });
        assert_eq!(actor_id(5), 0x8000_0005);
        assert_eq!(actor_id(0x8000_0007), 0x8000_0007);
    }

    #[test]
    fn colliders_list_settled_live_pieces_and_drop_them_when_set_collision_clears() {
        use killer_world::server_objects::{FenceKind, WALL_MESH, WALL_YAW};
        let mut world = World::default();
        world.apply_world(&WorldEvent::ColosseumFenceInfo(l2_net::game::ColosseumFenceInfo { values: [3, 2, 0, 0, 0, 100, 100] }));
        let pieces = world.server_objects.pieces.clone();
        assert_eq!(pieces.len(), 4 + 4);
        let mut fences = Fences::default();
        for piece in &pieces {
            let location = (piece.key % 2 == 0).then(|| piece.spawn + Vec3::Z * piece.key as f32);
            fences.pieces.insert(piece.key, Piece { location, ..Piece::default() });
        }
        let settled: Vec<_> = pieces.iter().filter(|piece| piece.key % 2 == 0).collect();
        let colliders = fences.colliders(&world);
        assert_eq!(colliders.len(), settled.len());
        for (collider, piece) in colliders.iter().zip(&settled) {
            assert_eq!(collider.class, PIECE_CLASS);
            assert_eq!(collider.mesh, piece.mesh());
            assert_eq!(collider.placement.location, (piece.spawn + Vec3::Z * piece.key as f32).to_array());
            assert_eq!(collider.placement.rotation, piece.rotation);
        }
        assert!(colliders.iter().any(|collider| collider.mesh == WALL_MESH && collider.placement.rotation == [0, WALL_YAW, 0]));
        let skip: Vec<u64> = settled.iter().map(|piece| piece.key).take(1).collect();
        assert_eq!(fences.colliders_except(&world, &skip).len(), settled.len() - 1);
        world.apply_world(&WorldEvent::ColosseumFenceInfo(l2_net::game::ColosseumFenceInfo { values: [3, 1, 0, 0, 0, 100, 100] }));
        let left = fences.colliders(&world);
        assert_eq!(left.len(), settled.iter().filter(|piece| piece.kind == FenceKind::Column).count());
        assert!(left.iter().all(|collider| collider.mesh != WALL_MESH));
        world.apply_world(&WorldEvent::ColosseumFenceInfo(l2_net::game::ColosseumFenceInfo { values: [3, 0, 0, 0, 0, 100, 100] }));
        assert!(fences.colliders(&world).is_empty());
    }
}
