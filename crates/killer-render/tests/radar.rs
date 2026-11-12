use glam::vec3;
use killer_assets::client::Client;
use killer_assets::textures::{BankSource, TextureBank, write_png};
use killer_render::camera::Camera;
use killer_render::environment::Environment;
use killer_render::headless::{CLEAR_RGBA, Headless};
use killer_render::radar::{RadarMesh, load_meshes, radar_steps, viewport};
use killer_ui::radar::RadarView;
use std::path::{Path, PathBuf};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

fn crop(rgba: &[u8], rect: [u32; 4]) -> Vec<u8> {
    let mut out = Vec::with_capacity((rect[2] * rect[3] * 4) as usize);
    for y in rect[1]..rect[1] + rect[3] {
        let start = ((y * WIDTH + rect[0]) * 4) as usize;
        out.extend_from_slice(&rgba[start..start + (rect[2] * 4) as usize]);
    }
    out
}

fn background(pixel: &[u8]) -> bool {
    pixel[..3] == CLEAR_RGBA[..3]
}

#[test]
#[ignore]
fn the_nine_lineage_decos_meshes_render_in_the_corner_viewport() {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from).expect("KILLER_CLIENT_DIR");
    let mut client = Client::open(&root).unwrap();
    let meshes = load_meshes(&mut client);
    assert_eq!(meshes.iter().map(|(mesh, _)| *mesh).collect::<Vec<_>>(), RadarMesh::ALL);
    let mut headless = Headless::new(WIDTH, HEIGHT).unwrap();
    let mut bank = TextureBank::new();
    headless.renderer_mut().set_radar_meshes(&meshes, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
    assert_eq!(bank.missing().count(), 0);
    assert!(RadarMesh::ALL.iter().all(|mesh| headless.renderer_mut().radar_mesh_ready(*mesh)));
    let view = RadarView {
        camera_pitch: -0x1000,
        camera_yaw: 0x2000,
        camera_roll: 0,
        party: vec![[600.0, -300.0], [-9000.0, 0.0]],
        target: Some([-400.0, 500.0]),
        arrow_to: Some([1200.0, 900.0, 0.0]),
        points: vec![[300.0, 700.0]],
        show_time: false,
    };
    let camera = Camera { eye: vec3(0.0, -300.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    let rect = viewport((WIDTH, HEIGHT));
    let mut toggle = false;
    let renderer = headless.renderer_mut();
    let steps = radar_steps(&view, |mesh| renderer.radar_mesh_ready(mesh), &mut toggle);
    renderer.set_radar(steps);
    let rgba = headless.render(&camera, &Environment::default()).unwrap();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let inside = x >= rect[0] && x < rect[0] + rect[2] && y >= rect[1] && y < rect[1] + rect[3];
            let at = ((y * WIDTH + x) * 4) as usize;
            if !inside {
                assert!(background(&rgba[at..at + 4]), "fora do radar em {x},{y}");
            }
        }
    }
    let radar = crop(&rgba, rect);
    let painted = radar.chunks(4).filter(|pixel| !background(pixel)).count();
    assert!(painted > (rect[2] * rect[3]) as usize / 4, "{painted}");
    write_png(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/radar.png"), rect[2], rect[3], &radar).unwrap();
    let renderer = headless.renderer_mut();
    let steps = radar_steps(&RadarView { show_time: true, ..view.clone() }, |mesh| renderer.radar_mesh_ready(mesh), &mut toggle);
    renderer.set_radar(steps);
    let time = crop(&headless.render(&camera, &Environment::default()).unwrap(), rect);
    write_png(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/radar-time.png"), rect[2], rect[3], &time).unwrap();
    assert_ne!(time, radar);
    eprintln!("radar: {painted} de {} pixels pintados; nove malhas carregadas", rect[2] * rect[3]);
}
