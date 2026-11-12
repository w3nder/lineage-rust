use crate::ground::GroundGrid;
use crate::trace::{Hit, TraceMesh};
use glam::Vec3;
use killer_assets::client::Client;
use killer_assets::options::ClientOptions;
use killer_assets::scene::actor_lighting::LightMap;
use killer_assets::scene::{MeshDraw, Neighbours, Scene, build_scene_with, terrain_draws, zone_lighting};
use killer_assets::textures::TextureBank;
use killer_assets::tiles::{LoadRequest, TileId, TileWindow};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use killer_platform::time::Instant;
use ue2_level::{BspTree, Level};

/// One loaded level besides the main one, with what the app queries on the CPU.
pub struct NeighbourTile {
    pub name: String,
    pub trace: TraceMesh,
    pub ground: GroundGrid,
    pub bsp_tree: BspTree,
    pub zones: usize,
    pub lighting: LightMap,
    pub level: Arc<Level>,
    pub draws: usize,
    pub triangles: usize,
}

pub struct LoadedNeighbour {
    pub tile: NeighbourTile,
    pub scene: Scene,
    pub bank: TextureBank,
    pub collision: killer_assets::collision::resources::Resources,
    pub elapsed: killer_platform::time::Duration,
}

pub struct Reseamed {
    pub name: String,
    pub draws: Vec<MeshDraw>,
    pub bank: TextureBank,
}

pub enum NeighbourResult {
    Loaded(Box<LoadedNeighbour>),
    Reseamed(Reseamed),
}

pub enum Job {
    Load(TileId),
    Reseam(String),
}

/// The main tile plus up to four neighbours, kept by the rules of `AdjustTerrain`/`AddSWLevel`/`CheckPurgeLevel`.
pub struct NeighbourSet {
    pub window: TileWindow,
    pub main_level: Arc<Level>,
    pub tiles: HashMap<String, NeighbourTile>,
    pub jobs: std::collections::VecDeque<Job>,
    pub running: Option<Job>,
    /// Tiles whose load failed: `AdjustTerrain` drops any request that names one.
    pub failed: std::collections::HashSet<TileId>,
}

impl NeighbourSet {
    pub fn new(main: TileId, main_level: Arc<Level>) -> Self {
        Self { window: TileWindow::new(main), main_level, tiles: HashMap::new(), jobs: std::collections::VecDeque::new(), running: None, failed: std::collections::HashSet::new() }
    }

    pub fn is_loaded(&self, tile: TileId) -> bool {
        tile == self.window.main || self.tiles.contains_key(&tile.name())
    }

    pub fn level_of(&self, tile: TileId) -> Option<Arc<Level>> {
        if tile == self.window.main {
            return Some(self.main_level.clone());
        }
        self.tiles.get(&tile.name()).map(|t| t.level.clone())
    }

    /// The tile that contains the point when it is loaded and attached, else the main one (`GetCurrentSWLevel`).
    pub fn tile_at(&self, position: Vec3) -> Option<&NeighbourTile> {
        self.tiles.get(&TileId::of(position.x, position.y).name())
    }

    /// `+X`, `+Y` and diagonal levels of `tile`, for the seam of its terrain.
    pub fn seam_levels(&self, tile: TileId) -> [Option<Arc<Level>>; 3] {
        [self.level_of(tile.offset(1, 0)), self.level_of(tile.offset(0, 1)), self.level_of(tile.offset(1, 1))]
    }

    /// Loaded tiles whose seam takes something from `tile`.
    pub fn stitched_to(&self, tile: TileId) -> Vec<TileId> {
        [tile.offset(-1, 0), tile.offset(0, -1), tile.offset(-1, -1)].into_iter().filter(|t| self.is_loaded(*t)).collect()
    }

    /// Advances the window with the pawn position and queues the loads and re-seams it asks for.
    pub fn follow(&mut self, position: Vec3) -> Vec<String> {
        let purged: Vec<String> = self.window.purge([position.x, position.y]).into_iter().map(TileId::name).collect();
        for name in &purged {
            self.tiles.remove(name);
        }
        if self.running.is_none() && self.jobs.is_empty() {
            match self.window.pending() {
                Some(request) => self.progress(request, position),
                None => {
                    if let Some(request) = self.window.step(position.x, position.y) {
                        self.progress(request, position);
                    }
                }
            }
        }
        purged
    }

    fn progress(&mut self, request: LoadRequest, position: Vec3) {
        if request.tiles().iter().any(|tile| self.failed.contains(tile) || self.would_be_purged(*tile, position)) {
            self.window.cancel();
            return;
        }
        match request.tiles().into_iter().find(|tile| !self.is_loaded(*tile)) {
            Some(tile) => self.jobs.push_back(Job::Load(tile)),
            None => self.window.commit(request.direction),
        }
    }

    /// `CheckPurgeLevel` would drop the tile as soon as it arrived: 35000 u from the pawn on either axis.
    fn would_be_purged(&self, tile: TileId, position: Vec3) -> bool {
        let [cx, cy] = tile.center();
        (position.x - cx).abs() > killer_assets::tiles::window::PURGE_DISTANCE || (position.y - cy).abs() > killer_assets::tiles::window::PURGE_DISTANCE
    }

    /// A new main tile is loading: nothing queued for the old window may run any more.
    pub fn cancel_all(&mut self) {
        self.jobs.clear();
        self.running = None;
        self.window.cancel();
    }

    pub fn attach(&mut self, loaded: NeighbourTile, position: Vec3) -> Vec<String> {
        let Some(tile) = TileId::parse(&loaded.name) else { return Vec::new() };
        let removed: Vec<String> = self.window.attach(tile, [position.x, position.y]).into_iter().map(TileId::name).collect();
        for name in &removed {
            self.tiles.remove(name);
        }
        for stitched in self.stitched_to(tile) {
            self.jobs.push_back(Job::Reseam(stitched.name()));
        }
        self.tiles.insert(loaded.name.clone(), loaded);
        removed
    }

    pub fn fail(&mut self, tile: TileId) {
        self.failed.insert(tile);
        self.window.cancel();
    }

    pub fn drawn(&self, view: Vec3, clipping_actor: f32) -> Vec<String> {
        self.window.drawn([view.x, view.y], clipping_actor).into_iter().map(TileId::name).collect()
    }

    pub fn nearest(&self, from: Vec3, to: Vec3, main: &TraceMesh) -> Option<Hit> {
        let mut best = main.nearest(from, to);
        for tile in self.tiles.values() {
            if let Some(hit) = tile.trace.nearest(from, to)
                && best.is_none_or(|b| hit.distance < b.distance)
            {
                best = Some(hit);
            }
        }
        best
    }

    pub fn ground_at<'a>(&'a self, position: Vec3, main: &'a GroundGrid) -> &'a GroundGrid {
        self.tile_at(position).map_or(main, |tile| &tile.ground)
    }

    pub fn zone_at(&self, position: Vec3) -> Option<u8> {
        self.tile_at(position).map(|tile| {
            let zone = tile.bsp_tree.zone_at(position.to_array());
            if usize::from(zone) < tile.zones { zone } else { 0 }
        })
    }

    pub fn draw_totals(&self) -> (usize, usize) {
        self.tiles.values().fold((0, 0), |acc, tile| (acc.0 + tile.draws, acc.1 + tile.triangles))
    }
}

fn neighbours_of(levels: &[Option<Arc<Level>>; 3]) -> Neighbours<'_> {
    Neighbours { right: levels[0].as_deref(), top: levels[1].as_deref(), corner: levels[2].as_deref() }
}

fn decode_textures(client: &mut Client, bank: &mut TextureBank, draws: &[MeshDraw]) {
    for draw in draws {
        for path in draw.material.texture_paths() {
            bank.texture(client.catalog(), path);
        }
        if let Some(mask) = &draw.mask {
            bank.alpha_map(client.catalog(), mask);
        }
    }
}

pub fn load_neighbour(root: &Path, tile: TileId, options: &ClientOptions, seam: [Option<Arc<Level>>; 3]) -> Result<LoadedNeighbour, String> {
    let started = Instant::now();
    let name = tile.name();
    let mut client = Client::open(root).map_err(|e| e.to_string())?;
    let loaded = client.load_level(&name).map_err(|e| e.to_string())?;
    let level = Arc::new(loaded.level);
    let scene = build_scene_with(&mut client, &level, options, neighbours_of(&seam));
    let mut bank = TextureBank::new();
    decode_textures(&mut client, &mut bank, &scene.draws);
    for part in scene.emitters.iter().flat_map(|a| &a.parts) {
        decode_textures(&mut client, &mut bank, &part.draws);
    }
    for actor in &scene.moving {
        decode_textures(&mut client, &mut bank, &actor.draws);
        for grade in &actor.grades { decode_textures(&mut client, &mut bank, grade); }
    }
    let mut collision = killer_assets::collision::resources::Resources::default();
    if let Err(error) = collision.tile(&mut client, &name, &level) {
        log::warn!("colisão do vizinho {name}: {error}");
    }
    let trace = TraceMesh::from_scene(&scene);
    let ground = GroundGrid::build(&scene);
    let lighting = LightMap::new(&level, &scene.lights);
    let tile = NeighbourTile {
        name: name.clone(),
        trace,
        ground,
        bsp_tree: level.bsp_tree.clone(),
        zones: scene.zones.len(),
        lighting,
        level,
        draws: scene.draws.len(),
        triangles: scene.draws.iter().map(|d| d.world_index_count() / 3).sum(),
    };
    log::info!("vizinho {name}: {} draws em {:.1?}", scene.draws.len(), started.elapsed());
    Ok(LoadedNeighbour { tile, scene, bank, collision, elapsed: started.elapsed() })
}

#[cfg(target_arch = "wasm32")]
pub fn load_neighbour_uploaded(root: &Path, tile: TileId, options: &ClientOptions, seam: [Option<Arc<Level>>; 3], renderer: &mut killer_render::renderer::Renderer) -> Result<NeighbourTile, String> {
    let name = tile.name();
    let mut client = Client::open(root).map_err(|e| e.to_string())?;
    let loaded = client.load_level(&name).map_err(|e| e.to_string())?;
    let level = Arc::new(loaded.level);
    let mut scene = build_scene_with(&mut client, &level, options, neighbours_of(&seam));
    let mut bank = TextureBank::new();
    renderer.add_tile(&name, &scene, &mut killer_assets::textures::BankSource {bank: &mut bank, catalog: client.catalog()});
    drop(bank);
    scene.lightmaps = Vec::new();
    client.catalog().release_files();
    let ground = GroundGrid::build(&scene);
    let lighting = LightMap::new(&level, &scene.lights);
    let zones = scene.zones.len();
    let draws = scene.draws.len();
    let triangles = scene.draws.iter().map(|draw| draw.world_index_count() / 3).sum();
    let trace = TraceMesh::from_owned_scene(scene);
    Ok(NeighbourTile {name, trace, ground, bsp_tree: level.bsp_tree.clone(), zones, lighting, level, draws, triangles})
}

pub fn reseam(root: &Path, name: &str, level: Arc<Level>, seam: [Option<Arc<Level>>; 3]) -> Result<Reseamed, String> {
    let mut client = Client::open(root).map_err(|e| e.to_string())?;
    let zones = zone_lighting(&level);
    let draws = terrain_draws(&mut client, &level, &zones, neighbours_of(&seam));
    let mut bank = TextureBank::new();
    decode_textures(&mut client, &mut bank, &draws);
    Ok(Reseamed { name: name.to_owned(), draws, bank })
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_assets::tiles::{Direction, TILE_SIZE};

    fn empty_level() -> Arc<Level> {
        Arc::new(Level::default())
    }

    fn tile(name: &str) -> NeighbourTile {
        NeighbourTile {
            name: name.to_owned(),
            trace: TraceMesh::from_scene(&Scene::default()),
            ground: GroundGrid::default(),
            bsp_tree: BspTree::default(),
            zones: 0,
            lighting: LightMap::default(),
            level: empty_level(),
            draws: 0,
            triangles: 0,
        }
    }

    #[test]
    fn following_west_queues_the_west_tile_then_commits_once_it_is_attached() {
        let main = TileId { x: 22, y: 22 };
        let mut set = NeighbourSet::new(main, empty_level());
        let [cx, cy] = main.center();
        assert!(set.follow(Vec3::new(cx, cy, 0.0)).is_empty());
        assert!(set.jobs.is_empty());
        set.follow(Vec3::new(cx - 9000.0, cy, 0.0));
        assert!(matches!(set.jobs.pop_front(), Some(Job::Load(t)) if t == TileId { x: 21, y: 22 }));
        set.running = Some(Job::Load(TileId { x: 21, y: 22 }));
        set.follow(Vec3::new(cx - 9000.0, cy, 0.0));
        assert!(set.jobs.is_empty());
        set.running = None;
        assert!(set.attach(tile("21_22"), Vec3::new(cx - 9000.0, cy, 0.0)).is_empty());
        assert!(matches!(set.jobs.front(), Some(Job::Reseam(name)) if name == "20_22") || set.jobs.is_empty());
        set.jobs.clear();
        set.follow(Vec3::new(cx - 9000.0, cy, 0.0));
        assert_eq!(set.window.cx, cx - 8192.0);
        assert_eq!(set.window.sx, 1);
        assert!(set.window.pending().is_none());
    }

    #[test]
    fn attaching_the_east_tile_reseams_the_main_one() {
        let main = TileId { x: 22, y: 22 };
        let mut set = NeighbourSet::new(main, empty_level());
        let [cx, cy] = main.center();
        set.attach(tile("23_22"), Vec3::new(cx, cy, 0.0));
        assert!(matches!(set.jobs.pop_front(), Some(Job::Reseam(name)) if name == "22_22"));
        assert!(set.jobs.is_empty());
        let seam = set.seam_levels(main);
        assert!(seam[0].is_some() && seam[1].is_none() && seam[2].is_none());
        assert_eq!(set.stitched_to(TileId { x: 23, y: 23 }), vec![TileId { x: 23, y: 22 }, main]);
        assert!(set.zone_at(Vec3::new(cx + 32768.0, cy, 0.0)).is_some());
        assert!(set.zone_at(Vec3::new(cx, cy, 0.0)).is_none());
        assert_eq!(set.drawn(Vec3::new(cx, cy, 0.0), 6.0), vec!["22_22".to_owned()]);
        assert_eq!(set.drawn(Vec3::new(cx + 12000.0, cy, 0.0), 6.0), vec!["22_22".to_owned(), "23_22".to_owned()]);
        set.window.commit(Direction::PlusX);
        assert_eq!(set.window.sx, -1);
    }

    #[test]
    #[ignore]
    fn walking_west_out_of_heine_loads_and_attaches_22_24_stitched_to_23_24() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(std::path::PathBuf::from) else { return };
        let mut client = Client::open(&root).unwrap();
        let heine = client.load_level("23_24").unwrap();
        let main = TileId { x: 23, y: 24 };
        let mut set = NeighbourSet::new(main, Arc::new(heine.level));
        let [cx, cy] = main.center();
        let position = Vec3::new(cx - 9000.0, cy, -3500.0);
        set.follow(position);
        let Some(Job::Load(tile)) = set.jobs.pop_front() else { panic!("sem pedido de carga") };
        assert_eq!(tile, TileId { x: 22, y: 24 });
        let seam = set.seam_levels(tile);
        assert!(seam[0].is_some() && seam[1].is_none() && seam[2].is_none());
        let loaded = load_neighbour(&root, tile, &ClientOptions::default(), seam).unwrap();
        assert!(loaded.tile.draws > 1000 && loaded.tile.trace.triangles() > 10000);
        assert!(set.attach(loaded.tile, position).is_empty());
        assert!(set.jobs.is_empty());
        set.follow(position);
        assert_eq!(set.window.cx, cx - 8192.0);
        assert!(set.window.pending().is_none());
        assert_eq!(set.drawn(position, 6.0), vec!["23_24".to_owned(), "22_24".to_owned()]);
        assert_eq!(set.drawn(Vec3::new(cx + 100.0, cy, 0.0), 6.0), vec!["23_24".to_owned()]);
        let west_point = Vec3::new(cx - 20000.0, cy, -3500.0);
        assert!(set.zone_at(west_point).is_some());
        assert!(set.ground_at(west_point, &GroundGrid::default()).triangle_count() > 0);
    }

    #[test]
    fn a_request_for_a_tile_the_purge_rule_would_drop_is_cancelled() {
        let main = TileId { x: 22, y: 22 };
        let mut set = NeighbourSet::new(main, empty_level());
        let [cx, cy] = main.center();
        let far = Vec3::new(cx + 3.0 * TILE_SIZE, cy, 0.0);
        set.follow(far);
        assert!(set.jobs.is_empty());
        assert!(set.window.pending().is_none());
        set.jobs.push_back(Job::Load(TileId { x: 23, y: 22 }));
        set.running = Some(Job::Reseam("22_22".into()));
        set.cancel_all();
        assert!(set.jobs.is_empty() && set.running.is_none());
    }

    #[test]
    fn a_failed_tile_is_never_requested_again() {
        let main = TileId { x: 22, y: 22 };
        let mut set = NeighbourSet::new(main, empty_level());
        let [cx, cy] = main.center();
        set.follow(Vec3::new(cx + 9000.0, cy, 0.0));
        assert!(matches!(set.jobs.pop_front(), Some(Job::Load(t)) if t == TileId { x: 23, y: 22 }));
        set.fail(TileId { x: 23, y: 22 });
        set.follow(Vec3::new(cx + 9000.0, cy, 0.0));
        assert!(set.jobs.is_empty());
        assert!(set.window.pending().is_none());
    }
}
