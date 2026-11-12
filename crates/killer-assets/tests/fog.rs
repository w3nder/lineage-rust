use killer_assets::client::Client;
use killer_assets::scene::zone_lighting;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn test_client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

#[test]
#[ignore]
fn player_start_zones_and_fog_infos_over_the_143_tiles() {
    let Some(mut client) = test_client() else { return };
    let (mut tiles, mut fog_infos, mut tiles_with_fog, mut clear_to_fog) = (0, 0, 0, 0);
    let mut kinds: BTreeMap<&str, usize> = BTreeMap::new();
    let mut interiors = Vec::new();
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            let Ok(loaded) = client.load_level(&tile) else { continue };
            tiles += 1;
            fog_infos += loaded.level.fog_infos.len();
            tiles_with_fog += usize::from(!loaded.level.fog_infos.is_empty());
            for fog in &loaded.level.fog_infos {
                assert!(fog.colors.len() >= 2, "{tile}: L2FogInfo sem Colors");
                assert!(fog.affect_range[1] > 0.0 && fog.fog_range[4][1] > 0.0, "{tile}: {fog:?}");
            }
            clear_to_fog += loaded.level.zone_infos.values().filter(|z| z.clear_to_fog_color).count();
            let zones = zone_lighting(&loaded.level);
            let Some(start) = loaded.level.actors.iter().find(|a| a.class.eq_ignore_ascii_case("PlayerStart")) else {
                *kinds.entry("sem PlayerStart").or_default() += 1;
                continue;
            };
            let zone = loaded.level.bsp_tree.zone_at(start.placement.location);
            let lighting = zones.get(usize::from(zone)).copied().unwrap_or_default();
            let kind = if lighting.terrain_zone {
                "terreno"
            } else if let Some(fog) = lighting.fog {
                interiors.push(format!("{tile}: zona {zone} {:.0}..{:.0} {:?} blend {}", fog.start, fog.end, fog.color.map(|c| (c * 255.0) as u8), fog.blend_time));
                "interior com bDistanceFog"
            } else {
                "sem névoa"
            };
            *kinds.entry(kind).or_default() += 1;
        }
    }
    eprintln!("{tiles} tiles: zona do PlayerStart {kinds:?}; {fog_infos} L2FogInfo em {tiles_with_fog} tiles; {clear_to_fog} zonas com bClearToFogColor");
    for line in interiors.iter().take(12) {
        eprintln!("  {line}");
    }
    assert_eq!(tiles, 143);
    assert_eq!(fog_infos, 536);
    assert_eq!(tiles_with_fog, 27);
}

#[test]
#[ignore]
fn fog_info_color_tables_against_the_original_interval_search() {
    let Some(mut client) = test_client() else { return };
    let (mut total, mut uncovered, mut unsorted, mut first_times) = (0, 0, 0, BTreeMap::<String, usize>::new());
    for x in 15..=26 {
        for y in 10..=26 {
            let Ok(loaded) = client.load_level(&format!("{x}_{y}")) else { continue };
            for fog in &loaded.level.fog_infos {
                total += 1;
                let times: Vec<f32> = fog.colors.iter().map(|c| c.time).collect();
                *first_times.entry(format!("{:?}", times.first())).or_default() += 1;
                unsorted += usize::from(times.windows(2).any(|pair| pair[1] < pair[0]));
                let gap = (0..2400).map(|step| step as f32 / 100.0).filter(|&hour| !original_interval_exists(&times, hour)).count();
                uncovered += usize::from(gap > 0);
            }
        }
    }
    eprintln!("{total} L2FogInfo, {uncovered} com horas sem intervalo no original, {unsorted} com tempos fora de ordem; primeiro Time: {first_times:?}");
}

fn original_interval_exists(times: &[f32], hour: f32) -> bool {
    let n = times.len();
    (0..n).any(|i| {
        let (a, mut b) = (times[i], times[(i + 1) % n]);
        while a > b {
            b += 24.0;
        }
        hour >= a && hour <= b
    })
}
