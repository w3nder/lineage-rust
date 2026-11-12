use killer_assets::client::Client;
use killer_assets::timeenv::{AmbientKind, TimeEnv};
use std::path::PathBuf;
use ue2_assets::{Mip, TextureFormat, decode_rgba};

const ATLAS: usize = 512;
const SLICES: usize = 8;
const MIP0: usize = 131072;
const MIP1: usize = 32768;

fn test_client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

fn tiles() -> Vec<String> {
    (15..=26).flat_map(|x| (10..=26).map(move |y| format!("{x}_{y}"))).collect()
}

#[test]
#[ignore]
fn every_tile_model_reads_to_its_last_byte_with_consistent_sections_and_lightmaps() {
    let Some(mut client) = test_client() else { return };
    let mut checked = 0;
    let mut worst_uv = 0.0f32;
    let (mut uv_vertices, mut uv_off) = (0usize, 0usize);
    let (mut groups_of_eight, mut groups_of_one) = (0usize, 0usize);
    for tile in tiles() {
        let Ok(loaded) = client.load_level(&tile) else { continue };
        let level = &loaded.level;
        let render = &level.bsp_render;
        assert!(render.complete, "{tile}: leitor não chegou ao fim do Model");
        let nodes = level.bsp_tree.nodes.len();
        for (index, node) in level.bsp_tree.nodes.iter().enumerate() {
            for child in [node.front,node.back,node.coplanar] {
                assert!(child == -1 || (child >= 0 && (child as usize) < nodes), "{tile}: nó {index}, filho {child}");
            }
            let Ok(offset) = usize::try_from(node.collision_bound) else { continue; };
            let hull = render.leaf_hulls.get(offset..).unwrap_or_else(|| panic!("{tile}: nó {index}, hull {offset}"));
            let count = hull.iter().position(|&v| v == -1).expect("terminador de hull");
            assert!(count <= 64, "{tile}: nó {index}, {count} planos");
            for &plane in &hull[..count] {
                assert!((plane as u32 & 0xbfff_ffff) < nodes as u32, "{tile}: nó {index}, plano {plane}");
            }
            let bounds = hull.get(count+1..count+7).expect("FBox do hull");
            assert!(bounds.iter().all(|&v| f32::from_bits(v as u32).is_finite()), "{tile}: nó {index}");
        }
        assert_eq!(render.node_draws.len(), nodes, "{tile}");
        let section_nodes: u32 = render.sections.iter().map(|section| section.num_nodes).sum();
        assert_eq!(section_nodes as usize, nodes, "{tile}: Σ NumNodes");
        for (index, draw) in render.node_draws.iter().enumerate() {
            let section = usize::try_from(draw.section).unwrap_or_else(|_| panic!("{tile} nó {index}: iSection {}", draw.section));
            let section = render.sections.get(section).unwrap_or_else(|| panic!("{tile} nó {index}: iSection {}", draw.section));
            let first = usize::try_from(draw.first_vertex).unwrap();
            assert!(first + usize::from(draw.num_vertices) <= section.vertices.len(), "{tile} nó {index}");
            assert!(draw.light_map < render.light_maps.len() as i32, "{tile} nó {index}: iLightMap {}", draw.light_map);
            if let Ok(light_map) = usize::try_from(draw.light_map) {
                let light_map = &render.light_maps[light_map];
                let m = light_map.world_to_lightmap;
                for vertex in &section.vertices[first..first + usize::from(draw.num_vertices)] {
                    let p = vertex.position;
                    let u = p[0] * m[0][0] + p[1] * m[1][0] + p[2] * m[2][0] + m[3][0];
                    let v = p[0] * m[0][1] + p[1] * m[1][1] + p[2] * m[2][1] + m[3][1];
                    let du = (vertex.lightmap_uv[0] * ATLAS as f32 - light_map.offset[0] as f32 - u).abs();
                    let dv = (vertex.lightmap_uv[1] * ATLAS as f32 - light_map.offset[1] as f32 - v).abs();
                    worst_uv = worst_uv.max(du).max(dv);
                    uv_vertices += 1;
                    if du >= 1e-3 || dv >= 1e-3 {
                        uv_off += 1;
                    }
                }
            }
        }
        for section in &render.sections {
            assert!(section.light_map_texture < render.groups.len() as i32, "{tile}: iLightMapTexture {}", section.light_map_texture);
        }
        for (index, group) in render.groups.iter().enumerate() {
            assert!(matches!(group.textures.len(), 1 | SLICES), "{tile} grupo {index}: {} texturas", group.textures.len());
            if group.textures.len() == SLICES { groups_of_eight += 1 } else { groups_of_one += 1 }
            for texture in &group.textures {
                assert_eq!((texture.width as usize, texture.height as usize, texture.format), (ATLAS, ATLAS, 3), "{tile} grupo {index}");
                assert_eq!(texture.mips.iter().map(Vec::len).collect::<Vec<_>>(), vec![MIP0, MIP1], "{tile} grupo {index}");
            }
        }
        checked += 1;
    }
    eprintln!("{checked} tiles; {groups_of_eight} grupos de 8 fatias, {groups_of_one} de 1; {uv_vertices} vértices com lightmap, {uv_off} fora de 1e-3 texel, erro máximo {worst_uv:e}");
    assert_eq!(checked, 143);
    assert!(uv_off * 50 < uv_vertices, "{uv_off} de {uv_vertices}");
}

#[test]
#[ignore]
fn giran_lightmap_floor_per_slice_is_half_the_bsp_ambient_of_the_slice_middle() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let timeenv = TimeEnv::load(&mut client, 0).unwrap();
    let render = &loaded.level.bsp_render;
    let group = &render.groups[0];
    let sun_zone = |zone: i32| {
        loaded.level.zones.get(usize::try_from(zone).unwrap()).and_then(|zone| zone.actor.as_deref()).and_then(|actor| loaded.level.zone_infos.get(actor)).is_some_and(|info| info.sun_affect)
    };
    let light_maps: Vec<_> = render.light_maps.iter().filter(|lm| sun_zone(lm.zone) && lm.size[0] >= 8 && lm.size[1] >= 8).collect();
    assert!(light_maps.len() > 100, "{}", light_maps.len());
    for (slice, texture) in group.textures.iter().enumerate() {
        let mip = Mip { width: texture.width, height: texture.height, data: &texture.mips[0] };
        let rgba = decode_rgba(TextureFormat::Dxt1, &mip, None).unwrap();
        let mut histogram = [[0usize; 256]; 3];
        for lm in &light_maps {
            for y in lm.offset[1] + 1..lm.offset[1] + lm.size[1] - 1 {
                for x in lm.offset[0] + 1..lm.offset[0] + lm.size[0] - 1 {
                    let at = ((y as usize) * ATLAS + x as usize) * 4;
                    for channel in 0..3 {
                        histogram[channel][usize::from(rgba[at + channel])] += 1;
                    }
                }
            }
        }
        let floor: Vec<u8> = histogram
            .iter()
            .map(|counts| {
                let total: usize = counts.iter().sum();
                let mut seen = 0;
                counts.iter().position(|&count| {
                    seen += count;
                    seen * 100 >= total
                }).unwrap() as u8
            })
            .collect();
        let middle = slice as f32 * 3.0 + 1.5;
        let ambient = timeenv.ambient(AmbientKind::Bsp, middle).map(|c| (c * 255.0 * 0.5).round() as i32);
        eprintln!("fatia {slice}: piso (percentil 1) {floor:?}, 0.5·BSPAmbient({middle}) = {ambient:?}");
        for (channel, tolerance) in [8, 4, 8].into_iter().enumerate() {
            assert!((i32::from(floor[channel]) - ambient[channel]).abs() <= tolerance, "fatia {slice}: piso {floor:?} vs {ambient:?}");
        }
    }
}
