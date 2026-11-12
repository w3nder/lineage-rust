use crate::loader::LoadRequest;
use glam::{Mat4, Vec3};
use killer_assets::{characters::CharacterMesh, client::{Client, DatTables}, collision::world::View,
    pickup::{Shape, assets::Asset, drop::{Motion, Plan}, placement}, scene::actor_lighting::{Bounds, Query}, textures::{BankSource,TextureBank}};
use killer_render::{renderer::{Renderer,EffectId,EffectPlacement}, skinned::{CharacterId, CharacterInstance}};
use killer_world::{Entity, EntityKind, ItemOrigin, World};
use std::{collections::{HashMap, HashSet}, sync::{Arc,mpsc::Sender}};

pub trait Geometry {
    fn neighbours(&self) -> &crate::tiles::NeighbourSet;
    fn zone_at(&self,position: Vec3) -> u8;
    fn actor_lighting(&self,query: Query) -> killer_assets::scene::actor_lighting::Sample;
}

#[cfg(target_arch = "wasm32")]
pub struct GeometryView<'a> {
    pub neighbours: &'a crate::tiles::NeighbourSet,
    pub lighting: &'a killer_assets::scene::actor_lighting::LightMap,
    pub bsp_tree: &'a ue2_level::BspTree,
    pub zones: usize,
}

#[cfg(target_arch = "wasm32")]
impl Geometry for GeometryView<'_> {
    fn neighbours(&self) -> &crate::tiles::NeighbourSet {self.neighbours}
    fn zone_at(&self,position: Vec3) -> u8 {
        self.neighbours.zone_at(position).unwrap_or_else(||{
            let zone=self.bsp_tree.zone_at(position.to_array());
            if usize::from(zone)<self.zones {zone} else {0}
        })
    }
    fn actor_lighting(&self,query: Query) -> killer_assets::scene::actor_lighting::Sample {
        self.neighbours.tile_at(query.position).map_or(self.lighting,|tile|&tile.lighting).sample(query,0,0.0)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Geometry for crate::app::WorldView {
    fn neighbours(&self) -> &crate::tiles::NeighbourSet {&self.neighbours}
    fn zone_at(&self,position: Vec3) -> u8 {self.zone_at(position)}
    fn actor_lighting(&self,query: Query) -> killer_assets::scene::actor_lighting::Sample {self.actor_lighting(query)}
}

struct Mesh {
    id: CharacterId,
    mesh: CharacterMesh,
    shape: Shape,
}

#[derive(Default)]
struct Loaded {
    pending: bool,
    failed: bool,
    meshes: Vec<Mesh>,
    effects: Arc<[Option<killer_assets::scene::emitters::EmitterAsset>;2]>,
    bank: TextureBank,
    sounds: [Option<Arc<killer_assets::pickup::assets::Sound>>;2],
}

#[derive(Clone, Copy, PartialEq)]
struct Signature {
    spawned: u64,
    item_id: u32,
    position: Vec3,
    heading: i32,
    origin: ItemOrigin,
}

impl Signature {
    fn of(entity: &Entity) -> Option<Self> {
        let EntityKind::Item { item_id, .. } = entity.kind else { return None; };
        Some(Self { spawned:entity.spawned,item_id:item_id as u32,position:entity.position,heading:entity.heading,origin:entity.item_origin? })
    }
}

struct Piece {
    slot: usize,
    plan: Plan,
    motion: Option<Motion>,
    effect: Option<EffectId>,
    effect_started: bool,
    hidden_for: f32,
    throw_started: bool,
    sound_started: bool,
}

struct Instance {
    signature: Signature,
    ticket: u64,
    pending: bool,
    failed: bool,
    pieces: Vec<Piece>,
}

pub struct SoundRequest {
    pub actor: u64,
    pub sound: Arc<killer_assets::pickup::assets::Sound>,
    pub position: Vec3,
    pub volume: f32,
}

#[derive(Default)]
pub struct Items {
    sounds: Vec<SoundRequest>,
    assets: HashMap<u32,Loaded>,
    instances: HashMap<i32,Instance>,
    next_ticket: u64,
}

impl Items {
    pub fn targets(&self, world: &World) -> Vec<killer_assets::collision::world::Target> {
        self.targets_with_pieces(world).into_iter().map(|(target,_)| target).collect()
    }

    pub fn targets_with_pieces(&self, world: &World) -> Vec<(killer_assets::collision::world::Target,Option<usize>)> {
        let mut targets: Vec<_> = world.me.iter().chain(world.entities.values())
            .filter(|entity| !matches!(entity.kind,EntityKind::Item { .. }) && entity.collision_radius > 0.0 && entity.collision_height > 0.0)
            .map(|entity| (killer_assets::collision::world::Target { object_id:entity.object_id,spawned:entity.spawned,
                position:entity.position,radius:entity.collision_radius,height:entity.collision_height },None)).collect();
        for (&object_id,instance) in &self.instances {
            if world.entities.get(&object_id).and_then(Signature::of) != Some(instance.signature) { continue; }
            for piece in &instance.pieces {
                targets.push((killer_assets::collision::world::Target { object_id,spawned:instance.signature.spawned,
                    position:piece.plan.placement.position,radius:piece.plan.placement.shape.radius,height:piece.plan.placement.shape.height },Some(piece.slot)));
            }
        }
        targets
    }

    pub fn label(&self, world: &World, tables: &DatTables, object_id: i32, spawned: u64, slot: usize) -> Option<(Vec3,String)> {
        let entity = world.entities.get(&object_id).filter(|entity| entity.spawned == spawned)?;
        let instance = self.instances.get(&object_id).filter(|instance| Signature::of(entity) == Some(instance.signature))?;
        let EntityKind::Item { item_id, count, stackable } = entity.kind else { return None; };
        let piece = instance.pieces.iter().find(|piece| piece.slot == slot)?;
        let mesh = self.assets.get(&instance.signature.item_id)?.meshes.iter().find(|mesh| mesh.mesh.slot as usize == slot)?;
        let name = item_name(tables,item_id)?;
        let location = piece.plan.placement.position;
        let z = (f64::from(mesh.mesh.bounding_box[1][2])*0.5+f64::from(location.z)) as f32;
        Some((Vec3::new(location.x,location.y,z),label_text(name,stackable,count)))
    }

    pub fn take_sounds(&mut self) -> Vec<SoundRequest> { std::mem::take(&mut self.sounds) }

    pub fn audio_positions(&self, world: &World) -> HashMap<u64,Vec3> {
        self.instances.iter().filter(|(id,instance)| world.entities.get(id).and_then(Signature::of) == Some(instance.signature))
            .flat_map(|(_,instance)| instance.pieces.iter().map(|piece| (instance.ticket.wrapping_mul(3).wrapping_add(piece.slot as u64),piece.plan.placement.position))).collect()
    }

    pub fn clear(&mut self) {
        self.sounds.clear();
        self.assets.clear();
        self.instances.clear();
    }

    pub fn sync(&mut self, world: &World, renderer: &mut Renderer) {
        self.instances.retain(|id,instance| {
            let keep = world.entities.get(id).and_then(Signature::of) == Some(instance.signature);
            if !keep { for piece in &instance.pieces { if let Some(id) = piece.effect { renderer.remove_effect(id); } } }
            keep
        });
        for entity in world.entities.values() {
            let Some(signature) = Signature::of(entity) else { continue; };
            self.instances.entry(entity.object_id).or_insert_with(|| {
                self.next_ticket = self.next_ticket.wrapping_add(1);
                Instance { signature,ticket:self.next_ticket,pending:false,failed:false,pieces:Vec::new() }
            });
        }
        let used: HashSet<_> = self.instances.values().map(|instance| instance.signature.item_id).collect();
        self.assets.retain(|id,asset| {
            if used.contains(id) { return true; }
            for mesh in &asset.meshes { renderer.remove_character(mesh.id); }
            false
        });
        for id in used { self.assets.entry(id).or_default(); }
    }

    pub fn request(&mut self, tables: &DatTables, view: &impl Geometry, renderer: &Renderer, sender: &Sender<LoadRequest>) {
        self.request_with(tables,view,renderer,|request|sender.send(request).is_ok());
    }

    pub fn request_with(&mut self,tables: &DatTables,view: &impl Geometry,renderer: &Renderer,mut send: impl FnMut(LoadRequest)->bool) {
        let mut available = 2usize.saturating_sub(self.assets.values().filter(|asset| asset.pending).count()
            + self.instances.values().filter(|instance| instance.pending).count());
        for (&item_id,asset) in &mut self.assets {
            if available == 0 { return; }
            if asset.pending || asset.failed || !asset.meshes.is_empty() { continue; }
            let Some(item) = tables.item_icons.get(item_id) else { asset.failed = true; continue; };
            let sound = match item.kind {
                l2_dat::itemicons::ItemKind::Weapon => tables.weapongrp.get(&item_id).map(|item| item.drop_sound.clone()),
                l2_dat::itemicons::ItemKind::Armor => tables.armorgrp.get(&item_id).map(|item| item.drop_sound.clone()),
                l2_dat::itemicons::ItemKind::Etc => tables.etcitemgrp.get(&item_id).map(|item| item.item_sound.clone()),
            }.unwrap_or_default();
            asset.pending = send(LoadRequest::Item { item_id,data:item.drop.clone(),sound });
            if asset.pending { available -= 1; }
        }
        for instance in self.instances.values_mut() {
            if available == 0 { return; }
            if instance.pending || instance.failed || !instance.pieces.is_empty() { continue; }
            let signature = instance.signature;
            let Some(asset) = self.assets.get(&signature.item_id).filter(|asset| !asset.meshes.is_empty()) else { continue; };
            let Some(item) = tables.item_icons.get(signature.item_id) else { continue; };
            let (kind,source) = match signature.origin {
                ItemOrigin::Spawn => (None,None),
                ItemOrigin::Drop { source_position, .. } => (Some(item.drop.parameter_14),source_position),
            };
            let request = placement::Request { position:signature.position,rotation:[0,signature.heading,0],style:item.drop.style,kind,source,
                pieces:asset.meshes.iter().map(|mesh| (mesh.mesh.slot as usize,mesh.shape)).collect(),views:views(view,Some(renderer),signature.position) };
            instance.pending = send(LoadRequest::PlaceItem { ticket:instance.ticket,request });
            if instance.pending { available -= 1; }
        }
    }

    pub fn retry_pending(&mut self) {
        for asset in self.assets.values_mut() { asset.pending = false; }
        for instance in self.instances.values_mut() { instance.pending = false; }
    }

    pub fn install_asset(&mut self, item_id: u32, result: Result<Box<Asset>,String>, renderer: &mut Renderer, client: &mut Client) {
        let Some(asset) = self.assets.get_mut(&item_id).filter(|asset| asset.pending) else { return; };
        asset.pending = false;
        match result {
            Ok(mut loaded) => {
                for path in &loaded.missing { log::warn!("drop {item_id}: recurso original não carregado: {path}"); }
                let mut source = BankSource { bank:&mut loaded.bank,catalog:client.catalog() };
                asset.meshes = loaded.pieces.into_iter().map(|piece| Mesh {
                    id:renderer.upload_character(&piece.mesh,&mut source),mesh:piece.mesh,shape:piece.shape,
                }).collect();
                asset.failed = asset.meshes.is_empty();
                asset.sounds = loaded.sounds;
                asset.effects = loaded.effects;
                asset.bank = loaded.effect_bank;
            }
            Err(reason) => { asset.failed = true; log::warn!("drop {item_id}: {reason}"); }
        }
        client.catalog().release_files();
    }

    pub fn install_placement(&mut self, ticket: u64, result: Result<Vec<(usize,Plan)>,String>) {
        let Some(instance) = self.instances.values_mut().find(|instance| instance.ticket == ticket && instance.pending) else { return; };
        instance.pending = false;
        match result {
            Ok(pieces) => instance.pieces = pieces.into_iter().map(|(slot,plan)| Piece { slot,motion:plan.trajectory.map(Motion::new),plan,effect:None,effect_started:false,hidden_for:0.0,throw_started:false,sound_started:false }).collect(),
            Err(reason) => { instance.failed = true; log::warn!("drop {}: {reason}",instance.signature.item_id); }
        }
    }

    pub fn instances(&mut self, delta: f32, view: &impl Geometry, renderer: &mut Renderer, mut client: Option<&mut Client>) -> Vec<CharacterInstance> {
        let mut rendered = Vec::new();
        for (&object_id,instance) in &mut self.instances {
            let Some(asset) = self.assets.get_mut(&instance.signature.item_id) else { continue; };
            let last_slot = instance.pieces.last().map(|piece| piece.slot);
            for piece in &mut instance.pieces {
                let actor = instance.ticket.wrapping_mul(3).wrapping_add(piece.slot as u64);
                if !piece.throw_started {
                    piece.throw_started = true;
                    if piece.motion.is_some() && let Some(sound) = &asset.sounds[1] {
                        self.sounds.push(SoundRequest { actor,sound:sound.clone(),position:instance.signature.position,volume:0.5 });
                    }
                }
                piece.hidden_for = (piece.hidden_for-delta).max(0.0);
                if let Some(id) = piece.effect && renderer.effect_finished(id) {
                    renderer.remove_effect(id);
                    piece.effect = None;
                }
                if let Some(motion) = &mut piece.motion {
                    motion.tick(delta);
                    piece.plan.placement.position = motion.position();
                    if piece.plan.spin.tick(delta,motion.ended()) { piece.motion = None; }
                    piece.plan.placement.rotation = piece.plan.spin.rotation;
                }
                if !piece.sound_started && piece.motion.is_none() && Some(piece.slot) == last_slot {
                    piece.sound_started = true;
                    if matches!(instance.signature.origin,ItemOrigin::Drop { .. }) && let Some(sound) = &asset.sounds[0] {
                        self.sounds.push(SoundRequest { actor,sound:sound.clone(),position:piece.plan.placement.position,volume:1.0 });
                    }
                }
                if !piece.effect_started && piece.motion.is_none() && matches!(instance.signature.origin,ItemOrigin::Drop { .. })
                    && let Some(client) = client.as_deref_mut()
                {
                    piece.effect_started = true;
                    let appear = piece.plan.kind == 5;
                    if let Some(mut effect) = asset.effects[usize::from(appear)].clone() {
                        if !appear {
                            for sprite in &mut effect.definition.sprites { sprite.start_location_polar[2] = [piece.plan.placement.shape.radius;2]; }
                        }
                        let id = renderer.add_effect(&effect,&mut BankSource { bank:&mut asset.bank,catalog:client.catalog() });
                        let position = if appear { piece.plan.placement.position } else { piece.plan.effect_position };
                        renderer.place_effect(id,EffectPlacement { location:position,rotation:[0;3],zone:view.zone_at(position),visible:true });
                        piece.effect = Some(id);
                        if appear { piece.hidden_for = 0.85; }
                    }
                }
                if piece.hidden_for > 0.0 { continue; }
                let Some(mesh) = asset.meshes.iter().find(|mesh| mesh.mesh.slot as usize == piece.slot) else { continue; };
                let placement = piece.plan.placement;
                let world = killer_assets::collision::transform::actor(ue2_level::Placement {
                    location:placement.position.to_array(),rotation:placement.rotation,
                },[1.0;3],[0.0;3])*mesh.mesh.local;
                let bounds = Bounds::transformed(mesh.mesh.bounding_box,world);
                let center = world.transform_point3(Vec3::from_slice(&mesh.mesh.bounding_sphere[..3]));
                let radius = mesh.mesh.bounding_sphere[3]*mesh.mesh.scale.abs().max_element();
                let lighting = view.actor_lighting(Query { position:placement.position,center,radius,bounds });
                rendered.push(CharacterInstance { overlay: [0.0; 4], actor_id:object_id,actor_position:placement.position,collision_height:placement.shape.height,
                    shadow_caster:false,light_sphere:[center.x,center.y,center.z,radius],mesh:mesh.id,world,
                    palette:vec![Mat4::IDENTITY;mesh.mesh.bones.len()],zone:view.zone_at(placement.position),lights:lighting.lights,pawn_lights:Vec::new(),
                    ambient:lighting.ambient,sun_affect:lighting.sun_affect });
            }
        }
        rendered
    }
}

fn item_name(tables: &DatTables, item_id: i32) -> Option<&str> {
    let id = u32::try_from(item_id).ok()?;
    if !(tables.weapongrp.contains_key(&id) || tables.armorgrp.contains_key(&id) || tables.etcitemgrp.contains_key(&id)) { return None; }
    tables.itemname.get(&id).map(|item| item.name.as_str()).filter(|name| !name.is_empty())
}

pub fn label_text(name: &str, stackable: bool, count: i32) -> String {
    if stackable { format!("{name}({})",stackable_number(count)) } else { name.to_string() }
}

pub fn stackable_number(count: i32) -> String {
    let digits: Vec<char> = count.to_string().chars().collect();
    let mut out = Vec::with_capacity(digits.len()+digits.len().saturating_sub(1)/3);
    for (written,index) in (0..digits.len()).rev().enumerate() {
        if written%3 == 0 && index+1 < digits.len() { out.push(','); }
        out.push(digits[index]);
    }
    out.iter().rev().collect()
}

pub fn views(view: &impl Geometry, renderer: Option<&Renderer>, position: Vec3) -> Vec<View> {
    let neighbours = view.neighbours();
    let main = neighbours.window.main.name();
    let mut views = vec![View { name:main.clone(),level:neighbours.main_level.clone(),poses:Default::default() }];
    for neighbour in &neighbours.window.neighbours {
        if let Some(tile) = neighbours.tiles.get(&neighbour.tile.name()) {
            views.push(View { name:tile.name.clone(),level:tile.level.clone(),poses:Default::default() });
        }
    }
    for (tile,name,pose) in renderer.into_iter().flat_map(Renderer::collision_poses) {
        let tile = if tile.is_empty() { main.as_str() } else { tile };
        if let Some(view) = views.iter_mut().find(|view| view.name == tile) { view.poses.insert(name.into(),pose); }
    }
    let current = killer_assets::tiles::TileId::of(position.x,position.y).name();
    if let Some(index) = views.iter().position(|view| view.name == current) { views.swap(0,index); }
    views
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stackable_number_inserts_commas_like_make_stackable_num_str() {
        for (count,text) in [(0,"0"),(7,"7"),(999,"999"),(1000,"1,000"),(12345,"12,345"),(1234567,"1,234,567"),(i32::MAX,"2,147,483,647"),(-123,"-,123"),(-1234,"-1,234")] {
            assert_eq!(stackable_number(count),text);
        }
    }

    #[test]
    fn label_text_shows_the_count_only_for_stackable_items() {
        assert_eq!(label_text("Adena",true,1500),"Adena(1,500)");
        assert_eq!(label_text("Short Sword",false,1),"Short Sword");
    }
}
