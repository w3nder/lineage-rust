use killer_assets::client::Client;
use killer_assets::scene::terrain::Grid;
use std::path::PathBuf;
use ue2_level::{SECTOR_MAP_LEN, SECTOR_MAP_SIDE};

fn test_client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

fn tiles() -> Vec<String> {
    (15..=26).flat_map(|x| (10..=26).map(move |y| format!("{x}_{y}"))).collect()
}

#[test]
#[ignore]
fn every_tile_terrain_has_all_its_sectors_maps_and_native_data() {
    let Some(mut client) = test_client() else { return };
    let (mut tiles_checked, mut sectors, mut worst_xy, mut worst_z) = (0, 0usize, 0.0f32, 0.0f32);
    for tile in tiles() {
        let Ok(loaded) = client.load_level(&tile) else { continue };
        for terrain in &loaded.level.terrains {
            assert_eq!(terrain.sectors.len(), 256, "{tile}: setores");
            assert_eq!((terrain.native.sectors_x, terrain.native.sectors_y), (16, 16), "{tile}");
            assert_eq!((terrain.native.heightmap_x, terrain.native.heightmap_y), (256, 256), "{tile}");
            assert_eq!(terrain.native.vertex_colors.len(), 65536, "{tile}: VertexColors");
            assert_eq!(terrain.visible_quads.len(), 2048, "{tile}: QuadVisibilityBitmap");
            assert!(terrain.edge_turn.is_empty() || terrain.edge_turn.len() == 2048, "{tile}: EdgeTurnBitmap {}", terrain.edge_turn.len());
            for sector in &terrain.sectors {
                assert_eq!(sector.intensities.len(), 8, "{tile} setor {:?}", sector.offset);
                assert!(sector.intensities.iter().all(|map| map.len() == SECTOR_MAP_LEN), "{tile} setor {:?}", sector.offset);
                assert_eq!(sector.pass_mask.len(), 256, "{tile} setor {:?}", sector.offset);
                let expected = [if sector.offset[0] == 240 { 15 } else { 16 }, if sector.offset[1] == 240 { 15 } else { 16 }];
                assert_eq!(sector.quads, expected, "{tile} setor {:?}", sector.offset);
                sectors += 1;
            }
            let to_world = terrain.native.to_world.unwrap_or_else(|| panic!("{tile}: sem ToWorld ({:?})", terrain.native));
            let Some(heightmap) = client.catalog().heightmap(&terrain.heightmap) else {
                eprintln!("{tile}: heightmap {} não encontrado", terrain.heightmap);
                continue;
            };
            let grid = Grid::from_samples(heightmap.width, heightmap.height, &heightmap.samples, terrain.location, terrain.scale);
            for y in 0..grid.height {
                for x in 0..grid.width {
                    let ours = grid.position(x, y);
                    let h = f32::from(heightmap.samples[y * grid.width + x]);
                    let theirs = to_world.transform_point([x as f32, y as f32, h]);
                    worst_xy = worst_xy.max((ours[0] - theirs[0]).abs()).max((ours[1] - theirs[1]).abs());
                    worst_z = worst_z.max((ours[2] - theirs[2]).abs());
                }
            }
        }
        tiles_checked += 1;
    }
    eprintln!("{tiles_checked} tiles, {sectors} setores; erro máximo vs ToWorld: xy {worst_xy}, z {worst_z}");
    assert_eq!(tiles_checked, 143);
    assert_eq!(sectors, 143 * 256);
    assert!(worst_xy <= 1e-3 && worst_z <= 1e-2, "xy {worst_xy} z {worst_z}");
}

#[test]
#[ignore]
fn giran_sector_5_intensity_means_match_the_census() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let terrain = &loaded.level.terrains[0];
    let sector = &terrain.sectors[5];
    let means: Vec<u32> = (0..8)
        .map(|slice| {
            let mut sum = 0u32;
            let mut count = 0u32;
            for y in 0..=sector.quads[1] as usize {
                for x in 0..=sector.quads[0] as usize {
                    sum += u32::from(sector.intensity(slice, x, y).unwrap());
                    count += 1;
                }
            }
            (sum + count / 2) / count
        })
        .collect();
    eprintln!("setor 5: médias {means:?}, layer_all {:?}", &sector.layer_all[..8]);
    assert_eq!(means, vec![153, 131, 80, 153, 198, 190, 131, 52]);
    assert_eq!(sector.intensity(0, SECTOR_MAP_SIDE - 1, 0), sector.intensity(0, SECTOR_MAP_SIDE - 1, 0));
}

#[test]
#[ignore]
fn giran_terrain_draws_exactly_the_quads_of_each_pass_mask() {
    use killer_assets::scene::terrain::quad_visible;
    use killer_assets::scene::{Pass, build_scene, DrawKind};
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let terrain = &loaded.level.terrains[0];
    let scene = build_scene(&mut client, &loaded.level);
    let mut drawn = [0usize; 16];
    for draw in scene.draws.iter().filter(|draw| draw.kind == DrawKind::Terrain) {
        let pass = match draw.pass {
            Pass::Opaque => 0,
            Pass::Layer(p) => usize::from(p),
            Pass::Translucent | Pass::Projector => continue,
        };
        drawn[pass] += draw.indices.len() / 6;
    }
    let mut expected = [0usize; 16];
    let mut layers_without_pass = 0;
    let visible = killer_assets::scene::terrain::seamless_visibility(&terrain.visible_quads, 256, 256);
    for sector in &terrain.sectors {
        for y in 0..16 {
            for x in 0..16 {
                if !quad_visible(&visible, 256, sector.offset[0] as usize + x, sector.offset[1] as usize + y) {
                    continue;
                }
                expected[0] += 1;
                let mask = sector.pass_mask[y * 16 + x];
                for (pass, count) in expected.iter_mut().enumerate().skip(1) {
                    if mask & (1 << pass) != 0 {
                        *count += 1;
                    }
                }
            }
        }
        layers_without_pass += sector.layer_all.iter().filter(|&&cache| cache == 0).count();
    }
    let invisible = (0..256 * 256).filter(|i| !quad_visible(&visible, 256, i % 256, i / 256)).count();
    eprintln!("quads por passe: desenhados {:?}, máscaras {:?}; {layers_without_pass} (setor, camada) com alpha toda 0; {invisible} quads invisíveis", &drawn[..6], &expected[..6]);
    assert_eq!(drawn, expected);
    assert_eq!(drawn[0], 65536 - invisible);
    assert!(scene.lightmaps.iter().any(|group| group.width == 257 && group.slices.len() == 8));
}

#[test]
#[ignore]
fn layer_uv_phase_report_over_the_143_tiles() {
    use killer_assets::scene::terrain::layer_uv;
    let Some(mut client) = test_client() else { return };
    let (mut layers, mut shifted, mut panned, mut texture_rotated, mut layer_rotated, mut axis, mut no_alpha0) = (0, 0, 0, 0, 0, 0, 0);
    let (mut alpha_maps, mut alpha_differs, mut lights, mut world_lights, mut timed) = (0, 0, 0, 0, 0);
    for tile in tiles() {
        let Ok(loaded) = client.load_level(&tile) else { continue };
        lights += loaded.level.lights.len();
        world_lights += loaded.level.lights.iter().filter(|l| matches!(l.emitter_light_type, 1 | 2)).count();
        timed += loaded.level.lights.iter().filter(|l| l.time_light).count();
        for terrain in &loaded.level.terrains {
            let Some(heightmap) = client.catalog().heightmap(&terrain.heightmap) else { continue };
            let grid = Grid::from_samples(heightmap.width, heightmap.height, &heightmap.samples, terrain.location, terrain.scale);
            if terrain.layers.first().is_some_and(|layer| layer.alpha_map.is_none()) {
                no_alpha0 += 1;
            }
            for layer in &terrain.layers {
                layers += 1;
                panned += usize::from(layer.pan != [0.0; 2]);
                texture_rotated += usize::from(layer.texture_rotation != 0);
                layer_rotated += usize::from(layer.layer_rotation != [0; 3]);
                if layer.texture_map_axis != 0 {
                    axis += 1;
                    eprintln!("{tile}: camada {} TextureMapAxis {} ({}, UScale {:?})", layers, layer.texture_map_axis, layer.material, layer.scale);
                }
                let old = [0.0 / layer.scale[0].max(f32::EPSILON), 0.0];
                let new = layer_uv(grid.position(0, 0), layer, terrain.scale);
                let differs = (0..2).any(|k| ((new[k] - old[k]).fract().abs() - 0.0).min(1.0 - (new[k] - old[k]).fract().abs()) > 1.0 / 64.0);
                shifted += usize::from(differs);
                if let Some(path) = layer.alpha_map.as_deref()
                    && let Some(raw) = client.catalog().texture_raw(path)
                {
                    alpha_maps += 1;
                    let image = killer_assets::textures::alpha_map_image(raw.clone()).unwrap();
                    let plain = client.catalog().texture(path).unwrap();
                    if image.rgba.as_chunks::<4>().0.iter().zip(plain.rgba.as_chunks::<4>().0.iter()).any(|(a, p)| a[3] != p[0]) {
                        alpha_differs += 1;
                        eprintln!("{tile}: {path} ({:?}) alpha != red", raw.format);
                    }
                }
            }
        }
    }
    eprintln!("{layers} camadas: {shifted} mudam de fase no vértice (0,0) com a fórmula de mundo; {panned} com pan, {texture_rotated} com TextureRotation, {layer_rotated} com LayerRotation, {axis} com TextureMapAxis != 0; {no_alpha0} terrenos sem AlphaMap na camada 0");
    eprintln!("{alpha_maps} alpha maps, {alpha_differs} com A != R; {lights} luzes, {world_lights} com EmitterLightType 1/2, {timed} com bTimeLight");
    assert!(layers >= 887, "{layers}");
}

#[test]
#[ignore]
fn giran_deco_instances_follow_the_density_maps() {
    use killer_assets::scene::deco;
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let terrain = &loaded.level.terrains[0];
    let heightmap = client.catalog().heightmap(&terrain.heightmap).unwrap();
    let grid = Grid::from_samples(heightmap.width, heightmap.height, &heightmap.samples, terrain.location, terrain.scale);
    assert!(!terrain.deco_layers.is_empty());
    let mut checked = 0;
    for (index, layer) in terrain.deco_layers.iter().enumerate() {
        let raw = client.catalog().texture_raw(&layer.density_map).unwrap();
        let image = killer_assets::textures::alpha_map_image(raw).unwrap();
        let density = deco::DensityMap { width: image.width as usize, height: image.height as usize, alpha: image.rgba.as_chunks::<4>().0.iter().map(|px| px[3]).collect() };
        let mut total = 0usize;
        let mut expected = 0.0f64;
        for (sector_index, sector) in terrain.sectors.iter().enumerate() {
            let mut rng = deco::Srand::new(layer.seed.wrapping_add(sector_index as i32));
            total += deco::instances(&grid, terrain, sector, layer, &density, None, &mut rng).len();
            let quads = [if sector.offset[0] == 240 { 15 } else { 16 }, if sector.offset[1] == 240 { 15 } else { 16 }];
            for y in 0..quads[1] {
                for x in 0..quads[0] {
                    let gx = sector.offset[0] as usize + x;
                    let gy = sector.offset[1] as usize + y;
                    if !layer.on_invisible_terrain && !killer_assets::scene::terrain::quad_visible(&terrain.visible_quads, 256, gx, gy) {
                        continue;
                    }
                    let dens = f64::from(density.at(gx, gy, [256, 256])) / 255.0;
                    let multiplier = f64::from(layer.density_multiplier[0] + layer.density_multiplier[1]) / 2.0;
                    expected += f64::from(layer.max_per_quad) * (dens * multiplier).min(1.0);
                }
            }
        }
        eprintln!("camada {index} {}: {total} instâncias, esperado {expected:.0} (MaxPerQuad {}, densidade {:?}, fade {:?})", layer.static_mesh, layer.max_per_quad, layer.density_multiplier, layer.fadeout_radius);
        if expected > 100.0 {
            assert!((total as f64 - expected).abs() <= expected * 0.10, "{total} vs {expected}");
            checked += 1;
        }
    }
    assert!(checked > 0);
    let options = killer_assets::options::ClientOptions { render_deco: true, ..killer_assets::options::ClientOptions::default() };
    let scene = killer_assets::scene::build_scene_with(&mut client, &loaded.level, &options, killer_assets::scene::Neighbours::default());
    let deco_draws = scene.draws.iter().filter(|draw| draw.fade.is_some()).count();
    let deco_triangles: usize = scene.draws.iter().filter(|draw| draw.fade.is_some()).map(|draw| draw.world_index_count() / 3).sum();
    eprintln!("{deco_draws} draws de decoração, {deco_triangles} triângulos");
    assert!(deco_draws > 0);
    let off = killer_assets::scene::build_scene_with(&mut client, &loaded.level, &killer_assets::options::ClientOptions::default(), killer_assets::scene::Neighbours::default());
    assert_eq!(off.draws.iter().filter(|draw| draw.fade.is_some()).count(), 0);
}

#[test]
#[ignore]
fn giran_seams_take_the_first_column_and_row_of_the_loaded_neighbours() {
    use killer_assets::scene::terrain::{seamed_colors, seamed_grid, TerrainSeam};
    let Some(mut client) = test_client() else { return };
    let own = client.load_level("22_22").unwrap();
    let right = client.load_level("23_22").unwrap();
    let top = client.load_level("22_23").unwrap();
    let corner = client.load_level("23_23").unwrap();
    let terrain = &own.level.terrains[0];
    let seam = TerrainSeam { right: right.level.terrains.first(), top: top.level.terrains.first(), corner: corner.level.terrains.first() };
    let grid = seamed_grid(&mut client, terrain, &seam).unwrap();
    let alone = seamed_grid(&mut client, terrain, &TerrainSeam::NONE).unwrap();
    let right_grid = seamed_grid(&mut client, seam.right.unwrap(), &TerrainSeam::NONE).unwrap();
    let top_grid = seamed_grid(&mut client, seam.top.unwrap(), &TerrainSeam::NONE).unwrap();
    assert_eq!((grid.width, grid.height), (257, 257));
    let mut worst = 0.0f32;
    for y in 0..256 {
        let stitched = grid.position(256, y);
        let theirs = right_grid.position(0, y);
        assert_eq!(stitched, theirs, "linha {y}");
        worst = worst.max((stitched[2] - alone.position(256, y)[2]).abs());
    }
    for x in 0..256 {
        assert_eq!(grid.position(x, 256), top_grid.position(x, 0), "coluna {x}");
    }
    let corner_grid = seamed_grid(&mut client, seam.corner.unwrap(), &TerrainSeam::NONE).unwrap();
    assert_eq!(grid.position(256, 256), corner_grid.position(0, 0));
    assert_eq!(alone.position(256, 10)[0], right_grid.position(0, 10)[0]);
    let colors = seamed_colors(&terrain.native.vertex_colors, 256, 256, Some(&seam.right.unwrap().native.vertex_colors), Some(&seam.top.unwrap().native.vertex_colors), None);
    assert_eq!(colors.len(), 257 * 257);
    assert_eq!(colors[5 * 257 + 256], seam.right.unwrap().native.vertex_colors[5 * 256]);
    assert_eq!(colors[256 * 257 + 5], seam.top.unwrap().native.vertex_colors[5]);
    eprintln!("degrau máximo entre a coluna 255 copiada e a coluna 0 de 23_22: {worst:.1} u");
    let scene = killer_assets::scene::build_scene_with(&mut client, &own.level, &killer_assets::options::ClientOptions::default(), killer_assets::scene::Neighbours { right: Some(&right.level), top: Some(&top.level), corner: Some(&corner.level) });
    let seam_vertices = scene
        .draws
        .iter()
        .filter(|draw| draw.kind == killer_assets::scene::DrawKind::Terrain)
        .flat_map(|draw| draw.world_vertices())
        .filter(|v| (v.position[0] - right_grid.position(0, 0)[0]).abs() < 0.5)
        .count();
    assert!(seam_vertices >= 257, "{seam_vertices}");
}
