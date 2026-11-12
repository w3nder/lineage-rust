use killer_assets::client::DatTables;
use killer_ui::initial_position::InitialPosition;
use killer_ui::special_maps::{TownMapView, XmasSealView, CLOSE, SIZE};
use killer_ui::window::{Desktop, Rect, Window, WindowId};
use killer_world::WorldEvent;

pub struct SpecialMaps {
    pub town: TownMapView,
    pub seal: XmasSealView,
}

impl SpecialMaps {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self { town: TownMapView::new(&lookup(447).unwrap_or_default()), seal: XmasSealView::new(&lookup(2).unwrap_or_default()) }
    }

    pub fn create_windows(desktop: &mut Desktop, town_saved: Option<(i32, i32)>, seal_saved: Option<(i32, i32)>) {
        let (width, height) = desktop.size;
        for (id, pos, initial, grip_height) in [
            (WindowId::TOWN_MAP, town_saved.unwrap_or((0, height / 2 - 252)), InitialPosition { enabled: true, horizontal_mode: 0, vertical_mode: 5, x: 0, y: 252 }, 21),
            (WindowId::XMAS_SEAL, seal_saved.unwrap_or((width / 2 - 167, height / 2 - 178)), InitialPosition { enabled: true, horizontal_mode: 0, vertical_mode: 3, x: width / 2 - 167, y: height / 2 - 178 }, 20),
        ] {
            let mut window = Window::new(id, Rect::new(pos.0, pos.1, SIZE.0, SIZE.1), Rect::new(0, 0, SIZE.0, grip_height)).with_initial_position(initial);
            window.visible = false;
            desktop.windows.push(window);
        }
    }

    pub fn world_event(&mut self, event: &WorldEvent, tables: Option<&DatTables>, desktop: &mut Desktop) {
        match event {
            WorldEvent::ShowTownMap(map) => {
                self.town.receive(map.texture.clone(), map.x, map.y);
                show(desktop, WindowId::TOWN_MAP, true);
            }
            WorldEvent::ShowXMasSeal(seal) => {
                self.seal.texture = u32::try_from(seal.item_id).ok()
                    .and_then(|id| tables?.etcitemgrp.get(&id))
                    .and_then(|item| item.textures.first()).filter(|texture| !texture.is_empty()).cloned();
                let visible = desktop.window(WindowId::XMAS_SEAL).is_some_and(|w| w.visible);
                show(desktop, WindowId::XMAS_SEAL, !visible);
            }
            _ => {}
        }
    }

    pub fn mouse_down(&mut self, id: WindowId, x: i32, y: i32, desktop: &mut Desktop) {
        if CLOSE.contains(x, y) {
            self.town.mouse_up();
            show(desktop, id, false);
        } else if id == WindowId::TOWN_MAP {
            self.town.mouse_down(x, y);
        }
    }
}

fn show(desktop: &mut Desktop, id: WindowId, value: bool) {
    if let Some(w) = desktop.window_mut(id) { w.visible = value; }
    if value && let Some(index) = desktop.windows.iter().position(|w| w.id == id) {
        let w = desktop.windows.remove(index);
        desktop.windows.push(w);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn special_maps_native_coordinate_boundaries() {
        let output = std::process::Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
            .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/special_maps_reference.py"))
            .output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let mut view = TownMapView::new("");
        let mut count = 0;
        for line in String::from_utf8(output.stdout).unwrap().lines() {
            let case: serde_json::Value = serde_json::from_str(line).unwrap();
            if case["kind"] == "drag" {
                let start = case["start"].as_i64().unwrap() as i32;
                view.mouse_up();
                view.receive("map".into(),start,start);
                for (step,state) in case["sequence"].as_array().unwrap().iter().zip(case["states"].as_array().unwrap()) {
                    let x = step[1].as_i64().unwrap() as i32;
                    let y = step[2].as_i64().unwrap() as i32;
                    match step[0].as_str().unwrap() {
                        "down" => view.mouse_down(x,y),
                        "move" => { view.mouse_move(x,y); }
                        "up" => view.mouse_up(),
                        _ => unreachable!(),
                    }
                    assert_eq!(*state,serde_json::json!([view.scroll.0,view.scroll.1]),"{case} {step}");
                }
                count += 1;
                continue;
            }
            view.receive("map".into(),case["x"].as_i64().unwrap() as i32,case["y"].as_i64().unwrap() as i32);
            assert_eq!(case["marker"],serde_json::json!([view.marker.0,view.marker.1]));
            assert_eq!(case["scroll"],serde_json::json!([view.scroll.0,view.scroll.1]));
            count += 1;
        }
        assert_eq!(count,117);
    }

    #[test]
    #[ignore]
    fn special_maps_all_town_textures_and_etcitem_records() {
        use killer_assets::{client::Client, textures::{BankSource, TextureBank, write_png}};
        use killer_ui::{draw::DrawList, font::{BitmapFont, Glyphs}};
        use killer_render::{camera::Camera, environment::Environment, headless::Headless};
        let root = std::env::var_os("KILLER_CLIENT_DIR").unwrap();
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let font = BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap());
        let mut maps = SpecialMaps::new(|id| tables.sysstring.get(&id).cloned());
        let mut desktop = Desktop::new((1040,750));
        SpecialMaps::create_windows(&mut desktop,None,None);
        let mut draw = DrawList::default();
        for (index, name) in ["dwarf","talking","orc","darkelf","elf"].into_iter().enumerate() {
            let texture = format!("town_map.town_map_{name}_t00");
            assert!(client.catalog().texture(&texture).is_some(),"{texture}");
            maps.world_event(&WorldEvent::ShowTownMap(l2_net::game::ShowTownMap { texture, x:256, y:256 }),Some(&tables),&mut desktop);
            maps.town.paint(&mut draw,10+(index%3) as i32*344,10+(index/3) as i32*364,&font,1.0);
        }
        for (&id,item) in &tables.etcitemgrp {
            maps.world_event(&WorldEvent::ShowXMasSeal(l2_net::game::ShowXMasSeal { item_id:id as i32 }),Some(&tables),&mut desktop);
            assert_eq!(maps.seal.texture.as_ref(),item.textures.first().filter(|s| !s.is_empty()));
        }
        let id = 5555;
        maps.world_event(&WorldEvent::ShowXMasSeal(l2_net::game::ShowXMasSeal { item_id:id }),Some(&tables),&mut desktop);
        maps.seal.paint(&mut draw,698,374,&font,1.0);
        let mut headless = Headless::new(1040,740).unwrap();
        let mut bank = TextureBank::new();
        headless.renderer_mut().set_ui(&draw,&mut BankSource { bank:&mut bank,catalog:client.catalog() });
        assert_eq!(bank.missing().collect::<Vec<_>>(),Vec::<&str>::new());
        let camera = Camera { eye:glam::vec3(0.0,-300.0,0.0),target:glam::Vec3::ZERO,fov_x_deg:60.0,near:1.0,far:5000.0 };
        let rgba = headless.render(&camera,&Environment::default()).unwrap();
        write_png(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-special-maps.png"),1040,740,&rgba).unwrap();
        eprintln!("special maps: cinco mapas, {} registros etcitem, selo {id}: {:?}",tables.etcitemgrp.len(),maps.seal.texture);
    }

    #[test]
    fn special_maps_1006c210_shows_and_1005c2a0_toggles() {
        let mut desktop = Desktop::new((1024,768));
        SpecialMaps::create_windows(&mut desktop, Some((10,20)), Some((30,40)));
        assert_eq!(desktop.window(WindowId::TOWN_MAP).unwrap().rect, Rect::new(10,20,334,354));
        assert_eq!(desktop.window(WindowId::XMAS_SEAL).unwrap().rect, Rect::new(30,40,334,354));
        let mut maps = SpecialMaps::new(|_| None);
        let town = WorldEvent::ShowTownMap(l2_net::game::ShowTownMap { texture: "town_map.giran".into(), x: 256, y: 512 });
        for _ in 0..2 {
            maps.world_event(&town, None, &mut desktop);
            assert!(desktop.window(WindowId::TOWN_MAP).unwrap().visible);
            assert_eq!(maps.town.scroll,(92,184));
        }
        let seal = WorldEvent::ShowXMasSeal(l2_net::game::ShowXMasSeal { item_id: 5555 });
        for visible in [true,false,true] {
            maps.world_event(&seal, None, &mut desktop);
            assert_eq!(desktop.window(WindowId::XMAS_SEAL).unwrap().visible,visible);
            assert!(maps.seal.texture.is_none());
        }
        maps.mouse_down(WindowId::XMAS_SEAL,CLOSE.x,CLOSE.y,&mut desktop);
        assert!(!desktop.window(WindowId::XMAS_SEAL).unwrap().visible);
        assert!(desktop.window(WindowId::TOWN_MAP).unwrap().visible);
    }
}
