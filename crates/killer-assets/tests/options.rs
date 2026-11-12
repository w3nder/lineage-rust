use killer_assets::client::Client;
use killer_assets::options::{ClientOptions, range_units};
use std::path::PathBuf;

#[test]
#[ignore]
fn the_installed_client_options_match_the_evidence() {
    let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
    let mut client = Client::open(&root).unwrap();
    let options = ClientOptions::load(&mut client).unwrap();
    assert_eq!(options.clipping.actor, 6.0);
    assert_eq!(options.clipping.static_mesh, 4.0);
    assert_eq!(options.clipping.static_mesh_lod, 6.0);
    assert_eq!(options.clipping.pawn, 1.5);
    assert_eq!(options.clipping.terrain, 8.0);
    assert_eq!(options.clipping.projector, 0.2);
    assert!(!options.keep_min_frame_rate && !options.render_deco && options.pawn_shadow && !options.water_effect);
    assert_eq!(options.post_proc, 0);
    assert!(options.seamless && options.shared_sky && options.purge_level && !options.thread_filter);
    assert_eq!(options.fog_range(), (8192.0, 20480.0));
    assert_eq!(range_units(options.clipping.terrain), 16384.0);
    assert_eq!(options.slope_sun_angle, 30.0);
    assert_eq!(options.start_time, 12.0);
}
