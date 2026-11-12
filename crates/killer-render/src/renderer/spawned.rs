use super::{ActorState, Renderer, TextureSource};
use killer_assets::scene::MeshDraw;
use killer_assets::scene::actor_view::ViewType;

impl Renderer {
    pub fn add_spawned(&mut self, key: u64, tile: &str, draws: &[MeshDraw], textures: &mut dyn TextureSource) -> bool {
        self.remove_spawned(key);
        let Some(index) = self.tiles.iter().position(|loaded| loaded.name == tile) else { return false };
        self.reserve_materials(draws.iter().map(|draw| draw.material.passes.len() as u32).sum());
        let mut added = false;
        for draw in draws {
            let Some(mut gpu) = self.gpu_draw(draw, textures, index) else { continue };
            gpu.spawned = Some(key);
            if let Some(view) = gpu.view
                && let Some(state) = self.tiles[index].views.get_mut(view)
            {
                *state = ActorState::default();
                state.view.set_view_type(ViewType::Spawning);
            }
            self.draws.push(gpu);
            added = true;
        }
        added
    }

    pub fn has_spawned(&self, key: u64) -> bool {
        self.draws.iter().any(|draw| draw.spawned == Some(key))
    }

    pub fn despawn(&mut self, key: u64) {
        let views: Vec<(usize, usize)> =
            self.draws.iter().filter(|draw| draw.spawned == Some(key)).filter_map(|draw| Some((draw.tile, draw.view?))).collect();
        for (tile, view) in views {
            if let Some(state) = self.tiles.get_mut(tile).and_then(|tile| tile.views.get_mut(view)) {
                state.view.set_view_type(ViewType::Despawning);
            }
        }
    }

    pub fn remove_spawned(&mut self, key: u64) {
        let mut kept = Vec::with_capacity(self.draws.len());
        for draw in self.draws.drain(..) {
            if draw.spawned == Some(key) {
                for pass in draw.material.passes {
                    self.textures.release_material(pass.slot);
                    self.materials.free.push(pass.slot);
                }
            } else {
                kept.push(draw);
            }
        }
        self.draws = kept;
    }
}
