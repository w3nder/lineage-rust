use super::{Shape, appearance};
use crate::{characters::{CharacterMesh, load_part}, client::Client, textures::TextureBank};
use l2_dat::armorgrp::ItemDrop;

pub struct Piece {
    pub mesh: CharacterMesh,
    pub shape: Shape,
}

pub struct Sound {
    pub name: String,
    pub encoded: std::sync::Arc<[u8]>,
}

pub struct Asset {
    pub sounds: [Option<std::sync::Arc<Sound>>;2],
    pub pieces: Vec<Piece>,
    pub bank: TextureBank,
    pub missing: Vec<String>,
    pub effects: std::sync::Arc<[Option<crate::scene::emitters::EmitterAsset>;2]>,
    pub effect_bank: TextureBank,
}

pub fn load(client: &mut Client, data: &ItemDrop) -> Asset {
    let mut shape = Shape::from_data(data);
    let mut pieces = Vec::new();
    let mut missing = Vec::new();
    let mut bank = TextureBank::new();
    for mut part in appearance(data).parts {
        part.textures.retain(|texture| texture.as_ref().is_some_and(|name| {
            let exists = client.catalog().object_class(name).is_some();
            if !exists { log::warn!("drop {}: material original ausente: {name}",part.mesh); }
            exists
        }));
        let slot = part.slot;
        part.slot = crate::pawn::SLOT_FACE;
        let Some(mut mesh) = load_part(client, &part) else {
            missing.push(part.mesh);
            continue;
        };
        mesh.slot = slot;
        shape.mesh(data.style, &mesh.floats_150);
        for path in mesh.sections.iter().flat_map(|section| section.material.texture_paths()) {
            bank.texture(client.catalog(), path);
        }
        pieces.push(Piece { mesh, shape });
    }
    Asset { sounds:[None,None],pieces, bank, missing, effects:std::sync::Arc::new([None,None]), effect_bank:TextureBank::new() }
}

pub fn load_effects(client: &mut Client) -> [Option<crate::scene::emitters::EmitterAsset>;2] {
    let options = crate::options::ClientOptions::parse("","",&client.read_system_text("env.int").unwrap_or_default());
    [super::drop::GROUND_EFFECT,super::drop::APPEAR_EFFECT].map(|class| {
        client.emitter_class(class).map(|emitter| crate::scene::emitters::effect_asset(client,&emitter,&options))
    })
}

pub fn with_effects(mut asset: Asset, client: &mut Client, effects: std::sync::Arc<[Option<crate::scene::emitters::EmitterAsset>;2]>) -> Asset {
    let mut effect_bank = TextureBank::new();
    for path in effects.iter().flatten().flat_map(|effect| &effect.parts).flat_map(|part| &part.draws).flat_map(|draw| draw.material.texture_paths()) {
        effect_bank.texture(client.catalog(),path);
    }
    asset.effects = effects;
    asset.effect_bank = effect_bank;
    asset
}
