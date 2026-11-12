use killer_assets::client::Client;
use killer_assets::scene::lightmap_regen::{ATLAS_SIZE, Regeneration};
use killer_assets::timeenv::TimeEnv;
use killer_render::environment::sun_direction;
use std::path::PathBuf;

fn client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

fn rgba_of_static(render: &ue2_level::BspRender, group: usize, slice: usize) -> Vec<u8> {
    let texture = &render.groups[group].textures[slice];
    let mip = ue2_assets::Mip { width: texture.width, height: texture.height, data: &texture.mips[0] };
    ue2_assets::decode_rgba(ue2_assets::TextureFormat::Dxt1, &mip, None).unwrap()
}

fn sun(hour: f32) -> [f32; 3] {
    sun_direction(hour).to_array()
}

fn covered(render: &ue2_level::BspRender, group: usize) -> Vec<bool> {
    let mut covered = vec![false; ATLAS_SIZE * ATLAS_SIZE];
    for light_map in render.light_maps.iter().filter(|light_map| usize::try_from(light_map.texture) == Ok(group)) {
        for y in 0..light_map.size[1] as usize {
            for x in 0..light_map.size[0] as usize {
                covered[(light_map.offset[1] as usize + y) * ATLAS_SIZE + light_map.offset[0] as usize + x] = true;
            }
        }
    }
    covered
}

#[test]
#[ignore]
fn giran_regenerated_day_slices_track_the_baked_dxt1_slices() {
    let Some(mut client) = client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let env = TimeEnv::load(&mut client, 0).unwrap();
    let level = &loaded.level;
    let render = &level.bsp_render;
    let sun_fn = sun;
    let mut regeneration = Regeneration::new(level, &env, &sun_fn);
    let mask = covered(render, 0);
    for slice in 4..7 {
        let atlas = regeneration.atlas(0, slice, 8, &|_| false);
        let baked = rgba_of_static(render, 0, slice);
        let mut sum = [0u64; 3];
        let mut count = 0u64;
        for (index, &pixel) in atlas.iter().enumerate().filter(|(index, _)| mask[*index]) {
            let ours = [(pixel >> 16) as i32 & 0xff, (pixel >> 8) as i32 & 0xff, pixel as i32 & 0xff];
            for channel in 0..3 {
                sum[channel] += (ours[channel] - i32::from(baked[index * 4 + channel])).unsigned_abs() as u64;
            }
            count += 1;
        }
        let mean = sum.map(|total| total as f64 / count as f64);
        eprintln!("fatia {slice}: texels {count} erro medio {mean:?}");
        assert!(mean.iter().all(|error| *error < 6.0), "fatia {slice}: {mean:?}");
    }
}

#[test]
#[ignore]
fn giran_light_actor_removed_from_a_sun_zone_lightmap_changes_the_regenerated_slice() {
    let Some(mut client) = client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let env = TimeEnv::load(&mut client, 0).unwrap();
    let level = &loaded.level;
    let sun_fn = sun;
    let mut regeneration = Regeneration::new(level, &env, &sun_fn);
    let light_map = &level.bsp_render.light_maps[3];
    let removed = light_map.bitmaps.iter().filter_map(|bitmap| bitmap.actor.clone()).find(|name| name.starts_with("Light")).unwrap();
    let before = regeneration.atlas(0, 4, 8, &|_| false);
    let mut other = Regeneration::new(level, &env, &sun_fn);
    let after = other.atlas(0, 4, 8, &|name| name == removed);
    let changed = before.iter().zip(&after).filter(|(a, b)| a != b).count();
    eprintln!("{removed}: {changed} texels mudaram");
    assert!(changed > 0);
    let group = other.group(0, &|name| name == removed).unwrap();
    assert_eq!((group.width, group.height, group.slices.len()), (512, 512, 8));
    assert!(group.slices.iter().all(|slice| slice.len() == 1 && slice[0].len() == 512 * 512 * 4));
}
