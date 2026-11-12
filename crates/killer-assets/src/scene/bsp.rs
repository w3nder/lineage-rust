use super::{DrawKind, LightmapGroup, MeshDraw, Vertex, ZoneLighting, draw_pass, resolve_material, sun_affect};
use crate::client::Client;
use std::collections::BTreeMap;
use ue2_level::{BspRender, BspTree, HIDDEN_SURFACE_FLAGS};

const PF_UNLIT: u32 = 0x0040_0000;
const PF_TWO_SIDED_OR_PORTAL: u32 = 0x0400_0100;
const TEXF_DXT1: u8 = 3;
const COLOR_MODIFIER: &str = "colormodifier";

pub fn fan_indices(corners: usize, base: u32) -> Vec<u32> {
    (1..corners.saturating_sub(1) as u32).flat_map(|i| [base, base + i, base + i + 1]).collect()
}

pub fn lightmap_groups(render: &BspRender) -> Vec<LightmapGroup> {
    render
        .groups
        .iter()
        .map(|group| {
            let first = group.textures.first();
            let dxt1 = group.textures.iter().all(|texture| texture.format == TEXF_DXT1);
            LightmapGroup {
                width: first.map_or(0, |texture| texture.width),
                height: first.map_or(0, |texture| texture.height),
                format: crate::textures::TextureFormatKind::Bc1,
                slices: if dxt1 { group.textures.iter().map(|texture| texture.mips.clone()).collect() } else { Vec::new() },
            }
        })
        .collect()
}

pub fn build(client: &mut Client, tree: &BspTree, render: &BspRender, zones: &[ZoneLighting]) -> Vec<MeshDraw> {
    let mut by_section: Vec<Vec<usize>> = vec![Vec::new(); render.sections.len()];
    for (index, draw) in render.node_draws.iter().enumerate() {
        if let Ok(section) = usize::try_from(draw.section)
            && let Some(nodes) = by_section.get_mut(section)
            && draw.num_vertices >= 3
            && draw.surface_flags & HIDDEN_SURFACE_FLAGS == 0
        {
            nodes.push(index);
        }
    }
    let mut draws = Vec::new();
    for (index, section) in render.sections.iter().enumerate() {
        let nodes = &by_section[index];
        if nodes.is_empty() {
            continue;
        }
        let material_path = section.material.clone().unwrap_or_default();
        let mut material = resolve_material(client, &material_path);
        let unlit = section.poly_flags & PF_UNLIT != 0;
        if unlit {
            for pass in &mut material.passes {
                pass.lit = false;
            }
        }
        if section.poly_flags & PF_TWO_SIDED_OR_PORTAL != 0 {
            for pass in &mut material.passes {
                pass.two_sided = true;
            }
        }
        let modulate_2x = material.root_class != COLOR_MODIFIER;
        let lightmap = (!unlit).then(|| usize::try_from(section.light_map_texture).ok()).flatten().filter(|group| *group < render.groups.len());
        let vertices: Vec<Vertex> = section
            .vertices
            .iter()
            .map(|vertex| Vertex { position: vertex.position, normal: vertex.normal, uv: vertex.uv, mask_uv: vertex.lightmap_uv, color: [1.0; 4] })
            .collect();
        let node_zone = |node: usize| tree.nodes.get(node).map_or(0, |node| node.zone_front);
        let node_sun = |node: usize| sun_affect(zones, node_zone(node));
        let mut groups: BTreeMap<(bool, u8), Vec<u32>> = BTreeMap::new();
        for &node in nodes {
            let draw = render.node_draws[node];
            let base = draw.first_vertex as u32;
            let indices = groups.entry((node_sun(node), node_zone(node))).or_default();
            indices.extend(fan_indices(usize::from(draw.num_vertices), base));
        }
        let pass = draw_pass(&material);
        for ((sun, zone), indices) in groups {
            draws.push(MeshDraw {
                instances: Vec::new(), vertices: vertices.clone(),
                indices,
                material: material.clone(),
                mask: None,
                kind: DrawKind::Bsp,
                sun_affect: sun,
                pass,
                zone,
                lightmap,
                modulate_2x,
                sector: None,
                fade: None,
                tint: None,
                actor: None,
            });
        }
    }
    draws
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convex_polygon_fans_from_the_first_corner() {
        let idx = fan_indices(5, 100);
        assert_eq!(idx, vec![100, 101, 102, 100, 102, 103, 100, 103, 104]);
    }

    #[test]
    fn lightmap_groups_keep_every_slice_and_its_mips() {
        let texture = |value: u8| ue2_level::StaticLightMap { width: 512, height: 512, format: 3, mips: vec![vec![value; 4], vec![value; 2]] };
        let render = BspRender { groups: vec![ue2_level::LightMapGroup { textures: vec![texture(1), texture(2)] }], ..BspRender::default() };
        let groups = lightmap_groups(&render);
        assert_eq!(groups.len(), 1);
        assert_eq!((groups[0].width, groups[0].height, groups[0].format), (512, 512, crate::textures::TextureFormatKind::Bc1));
        assert_eq!(groups[0].slices, vec![vec![vec![1; 4], vec![1; 2]], vec![vec![2; 4], vec![2; 2]]]);
    }
}
