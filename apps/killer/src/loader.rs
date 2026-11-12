use crate::characters::AppearanceKey;
use killer_assets::characters::{Appearance, CharacterMesh, Part, load_part};
use killer_assets::client::{ClassRenderDefaults, Client};
use killer_assets::options::ClientOptions;
use killer_assets::pawn::{HERO_AURA_CLASS, PawnEffect};
use killer_assets::scene::draw_pass;
use killer_assets::scene::emitters::{EmitterAsset, effect_asset};
use killer_assets::textures::TextureBank;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender, TryRecvError};
use std::thread;
use ue2_assets::MeshAnimation;

pub struct PickRequest {
    pub views: Vec<killer_assets::collision::world::View>,
    pub query: killer_assets::pickup::TraceQuery,
    pub targets: Vec<killer_assets::collision::world::Target>,
    pub spawned: Vec<killer_assets::collision::world::Spawned>,
}

pub enum LoadRequest {
    Pick { ticket: u64, request: PickRequest },
    Hover { ticket: u64, request: PickRequest },
    Item { item_id: u32, data: l2_dat::armorgrp::ItemDrop, sound: String },
    PlaceItem { ticket: u64, request: killer_assets::pickup::placement::Request },
    Fence { request: crate::server_objects::Request },
    Character { key: AppearanceKey, appearance: Appearance, draw_scale: f32 },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PawnLoad {
    pub defaults: killer_assets::damage_act::PawnDefaults,
    pub damage_effect: Option<String>,
}

pub fn pawn_load(client: &mut Client, appearance: &Appearance) -> PawnLoad {
    let defaults = appearance.class_name.as_deref().map_or_else(Default::default, |name| match client.pawn_class_defaults(name) {
        Ok(defaults) => defaults,
        Err(error) => {
            log::warn!("padrões de pawn {name}: {error}");
            Default::default()
        }
    });
    let damage_effect = appearance.damage_effect.clone().filter(|name| client.class_is_a(name, "Engine.Emitter").unwrap_or(false));
    PawnLoad { defaults, damage_effect }
}

pub struct EffectLoad {
    pub effects: Vec<(PawnEffect, EmitterAsset)>,
    pub bank: TextureBank,
}

pub enum LoadResult {
    Pick { ticket: u64, selection: Result<Option<killer_assets::collision::world::Selection>,String> },
    Hover { ticket: u64, selection: Result<Option<killer_assets::collision::world::Selection>,String> },
    Item { item_id: u32, asset: Result<Box<killer_assets::pickup::assets::Asset>,String> },
    PlaceItem { ticket: u64, pieces: Result<Vec<(usize,killer_assets::pickup::drop::Plan)>,String> },
    Fence { batch: crate::server_objects::Batch },
    Character {
        key: AppearanceKey,
        render_defaults: Option<ClassRenderDefaults>,
        pawn: Box<PawnLoad>,
        parts: Vec<CharacterMesh>,
        animation: Option<Arc<MeshAnimation>>,
        draw_scale: f32,
        weapon_type: usize,
        bank: Box<TextureBank>,
        effects: Box<EffectLoad>,
    },
    Failed {
        key: AppearanceKey,
        reason: String,
    },
}

pub struct Loader {
    pub requests: Sender<LoadRequest>,
    pub results: Receiver<LoadResult>,
}

impl Loader {
    pub fn spawn(root: PathBuf) -> Self {
        let (requests, inbox) = mpsc::channel::<LoadRequest>();
        let (outbox, results) = mpsc::sync_channel(0);
        thread::spawn(move || serve(&root, &inbox, &outbox));
        Self { requests, results }
    }
}

type Animations = HashMap<String, Weak<MeshAnimation>>;

#[derive(Default)]
pub struct Shared {
    pub sounds: HashMap<String,Weak<killer_assets::pickup::assets::Sound>>,
    pub pickup_effects: Option<Arc<[Option<EmitterAsset>;2]>>,
    pub collision: killer_assets::collision::resources::Resources,
    pub animations: Animations,
    pub missing: HashSet<String>,
    pub options: Option<ClientOptions>,
}

fn serve(root: &Path, inbox: &Receiver<LoadRequest>, outbox: &SyncSender<LoadResult>) {
    let mut client = None;
    let mut shared = Shared::default();
    let mut next = inbox.recv();
    while let Ok(request) = next {
        let result = match &mut client {
            Some(client) => handle(client, &mut shared, request),
            None => match Client::open(root) {
                Ok(opened) => handle(client.insert(opened), &mut shared, request),
                Err(e) => match request {
                    LoadRequest::Pick { ticket, .. } => LoadResult::Pick { ticket,selection:Err(format!("cliente: {e}")) },
                    LoadRequest::Hover { ticket, .. } => LoadResult::Hover { ticket,selection:Err(format!("cliente: {e}")) },
                    LoadRequest::Item { item_id, .. } => LoadResult::Item { item_id, asset:Err(format!("cliente: {e}")) },
                    LoadRequest::PlaceItem { ticket, .. } => LoadResult::PlaceItem { ticket, pieces:Err(format!("cliente: {e}")) },
                    LoadRequest::Fence { request } => LoadResult::Fence { batch: crate::server_objects::Batch {
                        pieces: request.pieces.iter().map(|piece| (piece.key, Err(format!("cliente: {e}")))).collect(), bank: TextureBank::new() } },
                    LoadRequest::Character { key, .. } => LoadResult::Failed { key, reason: format!("cliente: {e}") },
                },
            },
        };
        if outbox.send(result).is_err() {
            return;
        }
        next = match inbox.try_recv() {
            Ok(request) => Ok(request),
            Err(TryRecvError::Empty) => {
                if let Some(client) = &mut client {
                    client.catalog().release_files();
                }
                inbox.recv()
            }
            Err(TryRecvError::Disconnected) => return,
        };
    }
}

pub(crate) fn handle(client: &mut Client, shared: &mut Shared, request: LoadRequest) -> LoadResult {
    match request {
        LoadRequest::Pick { ticket,request } => LoadResult::Pick { ticket,selection:killer_assets::collision::world::trace_targets(client,&mut shared.collision,&request.views,request.query,&request.targets,&request.spawned) },
        LoadRequest::Hover { ticket,request } => LoadResult::Hover { ticket,selection:killer_assets::collision::world::trace_targets(client,&mut shared.collision,&request.views,request.query,&request.targets,&request.spawned) },
        LoadRequest::Item { item_id,data,sound } => {
            let effects = shared.pickup_effects.get_or_insert_with(|| Arc::new(killer_assets::pickup::assets::load_effects(client))).clone();
            let mut asset = killer_assets::pickup::assets::load(client,&data);
            shared.sounds.retain(|_,sound| sound.strong_count() > 0);
            asset.sounds = [sound.as_str(),killer_assets::pickup::drop::THROW_SOUND].map(|name| {
                if name.is_empty() || name.eq_ignore_ascii_case("None") { return None; }
                if let Some(sound) = shared.sounds.get(name).and_then(Weak::upgrade) { return Some(sound); }
                let (_,encoded) = client.sound(name)?;
                let sound = Arc::new(killer_assets::pickup::assets::Sound { name:name.into(),encoded:encoded.into() });
                shared.sounds.insert(name.into(),Arc::downgrade(&sound));
                Some(sound)
            });
            LoadResult::Item { item_id,asset:Ok(Box::new(killer_assets::pickup::assets::with_effects(asset,client,effects))) }
        },
        LoadRequest::Fence { request } => LoadResult::Fence { batch: crate::server_objects::build(client,&mut shared.collision,&request) },
        LoadRequest::PlaceItem { ticket,request } => LoadResult::PlaceItem { ticket,pieces:killer_assets::pickup::placement::place(client,&mut shared.collision,&request) },
        LoadRequest::Character { key, appearance, draw_scale } => {
            load_character(client, shared, key, &appearance, draw_scale)
        }
    }
}

fn animation(client: &mut Client, animations: &mut Animations, path: &str) -> Option<Arc<MeshAnimation>> {
    let name = path.to_lowercase();
    if let Some(shared) = animations.get(&name).and_then(Weak::upgrade) {
        return Some(shared);
    }
    let loaded = Arc::new(client.catalog().mesh_animation(path)?);
    animations.retain(|_, animation| animation.strong_count() > 0);
    animations.insert(name, Arc::downgrade(&loaded));
    Some(loaded)
}

pub fn load_character(
    client: &mut Client,
    shared: &mut Shared,
    key: AppearanceKey,
    appearance: &Appearance,
    draw_scale: f32,
) -> LoadResult {
    let render_defaults = appearance.class_name.as_deref().and_then(|name| {
        match client.class_render_defaults(name) {
            Ok(defaults) => Some(defaults),
            Err(error) => {
                log::warn!("defaults de render {name}: {error}; LOD automático indisponível para esta classe");
                None
            }
        }
    });
    let options = shared.options.get_or_insert_with(|| {
        ClientOptions::parse("", "", &client.read_system_text("env.int").unwrap_or_default())
    });
    let mut appearance = appearance.clone();
    appearance.parts.retain(|part| match part.placement {
        killer_assets::pawn::Placement::Enchanted { hero, .. } => enchant_enabled(options.enchant_mesh_show, appearance.enchant_effect, hero),
        _ => true,
    });
    for part in &mut appearance.parts {
        if let killer_assets::pawn::Placement::Enchanted { hero, .. } = part.placement {
            let texture = if hero { &options.hero_weapon_texture } else { &options.enchant_mesh_texture };
            part.textures = killer_assets::pawn::Part::new(part.mesh.clone(), vec![texture.clone()]).textures;
        }
    }
    let mut effects: Vec<(PawnEffect, EmitterAsset)> = Vec::new();
    if appearance.hero_aura && let Some(emitter) = client.emitter_class(HERO_AURA_CLASS) {
        effects.push((PawnEffect::HeroAura, effect_asset(client, &emitter, options)));
    }
    effects.extend(
        appearance
            .weapon_effects
            .iter()
            .filter(|effect| killer_assets::weapon_effects::enabled(options.enchant_effect_show, appearance.enchant_effect, effect.hero))
            .filter_map(|effect| {
                let mut emitter = client.emitter_class(&effect.class)?;
                let particles = options.enchant_effect_particles.get(&appearance.enchant_effect).copied()?;
                killer_assets::weapon_effects::apply_enchant(&mut emitter, effect, particles);
                Some((PawnEffect::Weapon(effect.clone()), effect_asset(client, &emitter, options)))
            }),
    );
    let colors = options.enchant_mesh_colors.get(&appearance.enchant_effect).copied();
    let mut enchant_materials = HashMap::new();
    for part in &appearance.parts {
        if let killer_assets::pawn::Placement::Enchanted { hero, .. } = part.placement {
            enchant_materials.entry(hero).or_insert_with(|| {
                let path = if hero { &options.hero_weapon_texture } else { &options.enchant_mesh_texture };
                client.catalog().enchanted_material(path, if hero { None } else { colors }).map(|mut material| {
                    for pass in &mut material.passes {
                        pass.lit = false;
                    }
                    material
                })
            });
        }
    }
    let parts = match loaded_parts(&appearance, &mut shared.missing, |part| {
        if let killer_assets::pawn::Placement::Enchanted { hero, .. } = part.placement {
            let material = enchant_materials.get(&hero)?.as_ref()?;
            let mut mesh = load_part(client, part)?;
            for section in &mut mesh.sections {
                section.material = material.clone();
                section.pass = draw_pass(material);
            }
            Some(mesh)
        } else {
            load_part(client, part)
        }
    }) {
        Ok(parts) => parts,
        Err(reason) => return LoadResult::Failed { key, reason },
    };
    let path = parts.iter().find_map(|p| p.animation.clone());
    let animation = path.and_then(|path| animation(client, &mut shared.animations, &path));
    let mut bank = TextureBank::new();
    for path in parts.iter().flat_map(|p| &p.sections).flat_map(|section| section.material.texture_paths()) {
        bank.texture(client.catalog(), path);
    }
    let mut effect_bank = TextureBank::new();
    for path in effects.iter().flat_map(|(_, asset)| &asset.parts).flat_map(|part| &part.draws).flat_map(|draw| draw.material.texture_paths()) {
        effect_bank.texture(client.catalog(), path);
    }
    let weapon_type = appearance.weapon_type as usize;
    let pawn = Box::new(pawn_load(client, &appearance));
    LoadResult::Character { key, render_defaults, pawn, parts, animation, draw_scale, weapon_type, bank: Box::new(bank), effects: Box::new(EffectLoad { effects, bank: effect_bank }) }
}

fn enchant_enabled(threshold: Option<i32>, level: u8, hero_weapon: bool) -> bool {
    threshold.is_some_and(|threshold| threshold >= 0 && (hero_weapon || i32::from(level) >= threshold))
}

fn loaded_parts(
    appearance: &Appearance,
    missing: &mut HashSet<String>,
    mut load: impl FnMut(&Part) -> Option<CharacterMesh>,
) -> Result<Vec<CharacterMesh>, String> {
    let mut parts = Vec::with_capacity(appearance.parts.len());
    let mut skipped = Vec::new();
    for part in &appearance.parts {
        match load(part) {
            Some(mesh) => parts.push(mesh),
            None => {
                if missing.insert(part.mesh.to_lowercase()) {
                    log::warn!("malha {} não encontrada; peça pulada", part.mesh);
                }
                skipped.push(part.mesh.as_str());
            }
        }
    }
    if !parts.iter().any(|p| p.is_master()) {
        return Err(format!("malha mestra (slot 7) não carregou: {}", skipped.join(", ")));
    }
    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn distinct_real_characters_share_live_animations_and_release_them_after_removal() {
        let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").unwrap());
        let mut client = Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        let mut weapons: Vec<_> = tables.weapongrp.iter().filter(|(_, weapon)| !weapon.meshes.is_empty()).map(|(&id, _)| id).collect();
        weapons.sort_unstable();
        assert!(weapons.len() >= tables.chargrp.len());
        let mut shared = Shared::default();
        let mut live = Vec::new();
        let mut meshes = HashSet::new();
        for (row, &weapon) in weapons.iter().enumerate().take(tables.chargrp.len()) {
            let key = AppearanceKey::Player { row, face: 0, hair_style: 0, hair_color: 0, equipped: Equipped { rhand: weapon as i32, ..Equipped::default() }, enchant_effect: 0, hero: false, mounted: 0 };
            let look = crate::characters::look(key, &tables).unwrap();
            meshes.extend(look.appearance.parts.iter().map(|part| part.mesh.clone()));
            let result = load_character(&mut client, &mut shared, key, &look.appearance, look.draw_scale);
            client.catalog().release_files();
            if !look.appearance.parts.iter().any(|part| part.placement == killer_assets::pawn::Placement::Master) {
                assert!(matches!(result, LoadResult::Failed { .. }));
                continue;
            }
            match &result {
                LoadResult::Character { parts, animation, .. } => {
                    assert!(!parts.is_empty());
                    let current = animation.as_ref().unwrap();
                    assert!(shared.animations.values().filter_map(Weak::upgrade).any(|cached| Arc::ptr_eq(&cached, current)));
                }
                LoadResult::Failed { reason, .. } => panic!("row {row}: {reason}"),
                _ => panic!("resultado não relacionado ao personagem"),
            }
            live.push(result);
        }
        eprintln!("{} personagens, {} meshes distintos, {} animações compartilhadas", live.len(), meshes.len(), shared.animations.len());
        assert!(meshes.len() > tables.chargrp.len());
        drop(live);
        assert!(shared.animations.values().all(|animation| animation.upgrade().is_none()));
        assert_eq!(client.catalog().cached_bytes(), 0);
    }

    #[test]
    fn get_enchanted_weapon_mesh_105f30d0_requires_enabled_config() {
        for hero in [false, true] {
            assert!(!enchant_enabled(None, 255, hero));
            assert!(!enchant_enabled(Some(-1), 255, hero));
        }
        assert!(!enchant_enabled(Some(4), 3, false));
        assert!(enchant_enabled(Some(4), 4, false));
        assert!(enchant_enabled(Some(4), 0, true));
        assert!(enchant_enabled(Some(0), 0, false));
    }
    use killer_assets::characters::Placement;
    use l2_dat::Equipped;
    use killer_platform::time::Duration;

    fn mesh(part: &Part) -> CharacterMesh {
        let mut mesh = CharacterMesh::empty();
        mesh.slot = part.slot;
        mesh.placement = part.placement;
        mesh
    }

    #[test]
    fn a_missing_part_is_skipped_and_warned_once() {
        let parts = vec![Part::new("Fighter.Face", Vec::new()), Part::new("Nope.Hat", Vec::new())];
        let appearance = Appearance { parts, ..Appearance::default() };
        let mut missing = HashSet::new();
        let load = |part: &Part| (part.mesh == "Fighter.Face").then(|| mesh(part));
        assert_eq!(loaded_parts(&appearance, &mut missing, load).unwrap().len(), 1);
        assert_eq!(loaded_parts(&appearance, &mut missing, load).unwrap().len(), 1);
        assert_eq!(missing.into_iter().collect::<Vec<_>>(), ["nope.hat"]);

        let none = Appearance { parts: vec![Part::new("Nope.Hat", Vec::new())], ..Appearance::default() };
        let reason = loaded_parts(&none, &mut HashSet::new(), |_| None).unwrap_err();
        assert!(reason.contains("Nope.Hat"), "{reason}");
        let mut hat = Part::new("Nope.Hat", Vec::new());
        hat.slot = 6;
        hat.placement = Placement::Skinned;
        let no_master = Appearance { parts: vec![hat], ..Appearance::default() };
        let load = |part: &Part| (part.mesh == "Nope.Hat").then(|| mesh(part));
        let reason = loaded_parts(&no_master, &mut HashSet::new(), load).unwrap_err();
        assert!(reason.contains("slot 7"), "{reason}");
    }

    fn request(id: i32) -> LoadRequest {
        let part = Part::new(format!("Nope.Mesh{id}"), Vec::new());
        let key = AppearanceKey::Npc { npc_id: id, equipped: Equipped::default() };
        let appearance = Appearance { parts: vec![part], ..Appearance::default() };
        LoadRequest::Character { key, appearance, draw_scale: 1.0 }
    }

    fn client_dir() -> Option<PathBuf> {
        let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from);
        if root.is_none() {
            eprintln!("KILLER_CLIENT_DIR not set; skipping");
        }
        root
    }

    #[test]
    #[ignore]
    fn a_missing_part_is_skipped_and_the_rest_of_the_character_loads() {
        let Some(root) = client_dir() else { return };
        let mut client = Client::open(&root).unwrap();
        let mut shared = Shared::default();
        let key = AppearanceKey::Npc { npc_id: 0, equipped: Equipped::default() };
        let parts = vec![
            Part::new("Fighter.MFighter_m000_f", vec!["MFighter.MFighter_m000_t00_f".to_string()]),
            Part::new("Nope.Mesh", Vec::new()),
            Part::new("Magic.MMagic_m005_m00_bh", Vec::new()),
        ];
        let appearance = Appearance { class_name: Some("LineageMonster.gremlin".into()), parts, ..Appearance::default() };
        match load_character(&mut client, &mut shared, key, &appearance, 1.0) {
            LoadResult::Character { parts, render_defaults, .. } => {
                assert_eq!(parts.len(), 1);
                assert_eq!(render_defaults, Some(client.class_render_defaults("LineageMonster.gremlin").unwrap()));
            }
            LoadResult::Failed { reason, .. } => panic!("{reason}"),
            _ => panic!("resultado não relacionado ao personagem"),
        }
        assert_eq!(shared.missing.len(), 2);
        let none = Appearance { parts: vec![Part::new("Nope.Mesh", Vec::new())], ..Appearance::default() };
        match load_character(&mut client, &mut shared, key, &none, 1.0) {
            LoadResult::Failed { reason, .. } => assert!(reason.contains("Nope.Mesh"), "{reason}"),
            _ => panic!("personagem sem malha carregou"),
        }
        assert_eq!(shared.missing.len(), 2);
    }

    #[test]
    fn loader_keeps_serving_after_going_idle() {
        let root = std::env::temp_dir().join(format!("killer-loader-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let loader = Loader::spawn(root);
        for id in [1, 2] {
            loader.requests.send(request(id)).unwrap();
            match loader.results.recv_timeout(Duration::from_secs(10)).unwrap() {
                LoadResult::Failed { key, reason } => {
                    assert_eq!(key, AppearanceKey::Npc { npc_id: id, equipped: Equipped::default() });
                    assert!(reason.contains("Mesh"), "{reason}");
                }
                _ => panic!("malha inexistente carregou"),
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}
