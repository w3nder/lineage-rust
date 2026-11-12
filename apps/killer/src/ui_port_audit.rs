use std::collections::BTreeSet;
use std::path::Path;

use killer_assets::client::Client;
use killer_assets::textures::{BankSource, TextureBank, write_png};
use killer_platform::time::Instant;
use killer_render::{camera::Camera, environment::Environment, headless::Headless};
use killer_ui::draw::DrawList;
use killer_ui::keys::MainTab;
use killer_ui::window::WindowId;
use serde_json::{Value, json};

use crate::ui2d::{GameUi, world_views};
use crate::ui_modern::ModernHud;

const SIZE: (u32, u32) = (1024, 768);

fn populate_new_windows(ui: &mut GameUi, world: &killer_world::World) {
    use killer_world::WorldEvent;
    use l2_net::game::packets::*;
    let events = [
        WorldEvent::PartyMatchList(PartyMatchList { flag: 2, rooms: (1..=15).map(|id| PartyRoom {
            id, title: format!("Grupo {id}"), location: 3, min_level: 20, max_level: 40,
            members: 3, max_members: 9, owner_name: format!("Lider {id}"),
        }).collect() }),
        WorldEvent::CastleSiegeInfo(CastleSiegeInfo {
            castle_id: 1, owner_flags: 1, owner_id: 8, pledge_name: "Defensores".into(), leader_name: "Lider".into(),
            ally_id: 0, ally_name: String::new(), now: 1_140_000_000, siege_time: 0,
            choices: vec![1_140_086_400, 1_140_172_800],
        }),
        WorldEvent::CastleSiegeAttackerList(SiegeClanList {
            castle_id: 1, header: [0; 3], total: 24, clans: (1..=24).map(|id| SiegeClan {
                pledge_id: id, name: format!("Cla {id}"), leader_name: format!("Lider {id}"),
                crest_id: 0, signed_time: 0, kind: 0, ally_id: 0, ally_name: "Alianca".into(),
                ally_leader_name: "Lider Alianca".into(), ally_crest_id: 0,
            }).collect(),
        }),
        WorldEvent::CastleSiegeDefenderList(SiegeClanList {
            castle_id: 1, header: [0; 3], total: 24, clans: (1..=24).map(|id| SiegeClan {
                pledge_id: id, name: format!("Defensor {id}"), leader_name: format!("Lider {id}"),
                crest_id: 0, signed_time: 0, kind: if id % 2 == 0 { 3 } else { 2 }, ally_id: 0,
                ally_name: String::new(), ally_leader_name: String::new(), ally_crest_id: 0,
            }).collect(),
        }),
        WorldEvent::SsqStatus(SsqStatus { page: 1, period: 1, body: SsqPage::Summary(SsqSummary {
            cycle: 8, period_message: 1290, until_message: 1291, player_cabal: 1, player_seal: 2,
            stones: 2000, adena: 4000, dusk_stone_score: 300, dusk_festival_score: 100,
            dusk_total_score: 400, dusk_percent: 40, dawn_stone_score: 450, dawn_festival_score: 150,
            dawn_total_score: 600, dawn_percent: 60,
        }) }),
        WorldEvent::SsqStatus(SsqStatus { page: 2, period: 1, body: SsqPage::Festivals(vec![SsqFestivalGroup {
            flag: 0, festivals: (1..=5).map(|id| SsqFestival {
                id, level_score: 1000, dusk_score: 450, dusk_members: vec!["Jogador A".into(); 9],
                dawn_score: 550, dawn_members: vec!["Jogador B".into(); 9],
            }).collect(),
        }]) }),
        WorldEvent::SsqStatus(SsqStatus { page: 3, period: 1, body: SsqPage::Seals {
            retain_limit: 10, claim_limit: 35,
            seals: (1..=3).map(|id| SsqSeal { id, owner: id % 2 + 1, dawn_percent: 60, dusk_percent: 40 }).collect(),
        } }),
        WorldEvent::SsqStatus(SsqStatus { page: 4, period: 1, body: SsqPage::Predictions {
            winning_cabal: 2, seals: (1..=3).map(|id| SsqSealPrediction { id, owner: 2, message: 1308 }).collect(),
        } }),
    ];
    for event in events { ui.world_event(&event, world); }
    ui.keys.take_commands();
}

fn quad_json(draw: &DrawList) -> Value {
    json!(draw.quads.iter().map(|q| json!({
        "rect": [q.x, q.y, q.w, q.h], "uv": [q.u, q.v, q.uw, q.vh],
        "texture": q.texture.as_str(), "color": q.color,
    })).collect::<Vec<_>>())
}

fn modern_pixels(gpu: &mut Headless, renderer: &mut egui_wgpu::Renderer, ctx: &egui::Context, mut output: egui::FullOutput) -> (Vec<u8>, usize) {
    let jobs = ctx.tessellate(output.shapes, output.pixels_per_point);
    let engine = gpu.renderer_mut();
    let device = engine.device();
    let queue = engine.queue();
    for (id, deltas) in &output.textures_delta.set {
        for delta in deltas { renderer.update_texture(device, queue, *id, delta); }
    }
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ui-port-audit"),
        size: wgpu::Extent3d { width: SIZE.0, height: SIZE.1, depth_or_array_layers: 1 },
        mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&Default::default());
    let screen = egui_wgpu::ScreenDescriptor { size_in_pixels: [SIZE.0, SIZE.1], pixels_per_point: output.pixels_per_point };
    let extra = renderer.update_buffers(device, queue, &mut encoder, &jobs, &screen);
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view, resolve_target: None, depth_slice: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            ..Default::default()
        }).forget_lifetime();
        renderer.render(&mut pass, &jobs, &screen);
    }
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None, size: u64::from(SIZE.0 * SIZE.1 * 4),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(target.as_image_copy(), wgpu::TexelCopyBufferInfo {
        buffer: &buffer, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(SIZE.0 * 4), rows_per_image: None },
    }, wgpu::Extent3d { width: SIZE.0, height: SIZE.1, depth_or_array_layers: 1 });
    queue.submit(extra.into_iter().chain([encoder.finish()]));
    let (tx, rx) = std::sync::mpsc::channel();
    buffer.slice(..).map_async(wgpu::MapMode::Read, move |value| { tx.send(value).unwrap(); });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    let bytes = buffer.slice(..).get_mapped_range().unwrap().to_vec();
    for id in &output.textures_delta.free { renderer.free_texture(id); }
    output.textures_delta.clear();
    (bytes, jobs.len())
}

#[test]
fn generated_routes_are_registered_and_reach_native_paint() {
    let ui = crate::ui2d::fixture_ui((SIZE.0 as i32, SIZE.1 as i32));
    let world = crate::characters::demo_world(glam::Vec3::ZERO, 0, 0);
    let views = world_views(&world, None, Instant::now());
    let mut ids = BTreeSet::new();
    for id in crate::ui_key_routes::PAINT_ROUTES {
        assert!(ids.insert(id.0), "duplicate {id:?}");
        assert!(killer_ui::window_owner::original_registration(*id).is_some(), "no C4 registration {id:?}");
        let window = ui.desktop.window(*id).unwrap();
        assert!(ui.keys.handles(*id));
        if *id == WindowId::OPTION { continue; }
        let mut direct = DrawList::default();
        let mut routed = DrawList::default();
        ui.keys.paint(*id, &mut direct, window.rect.x, window.rect.y, &world, &ui.small, ui.alpha);
        ui.paint_window(&mut routed, window, &world, &[], None, &views);
        let party = matches!(*id, WindowId::BROADCAST_PARTY_0 | WindowId::BROADCAST_PARTY_1 | WindowId::OLYMPIAD_ABNORMAL_1 | WindowId::OLYMPIAD_ABNORMAL_2);
        assert!(party || !direct.quads.is_empty(), "no paint {id:?}");
        assert_eq!(direct, routed, "lost route {id:?}");
    }
}

#[test]
#[ignore]
fn capture_all_registered_windows_in_both_themes() {
    let root = std::env::var_os("KILLER_CLIENT_DIR").unwrap();
    let directory = std::env::var_os("KILLER_UI_AUDIT_DIR").unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/ui-port/app").into_os_string());
    let directory = Path::new(&directory);
    std::fs::create_dir_all(directory).unwrap();
    let mut client = Client::open(Path::new(&root)).unwrap();
    let tables = client.tables().unwrap();
    let mut ui = GameUi::load(&mut client, (SIZE.0 as i32, SIZE.1 as i32)).unwrap();
    ui.set_tab_names(&tables);
    let mut world = crate::characters::demo_world(glam::Vec3::ZERO, 0, 0);
    crate::ui_keys::demo_fill(&mut world);
    let sample = *world.inventory.items.iter().find(|item| item.item_id == 1060).unwrap();
    world.inventory.items.extend((0..60).map(|index| {
        let mut item = sample;
        item.object_id = 10_000 + index;
        item.equipped = false;
        item
    }));
    ui.sync_windows(&world, Some(&tables), Instant::now());
    ui.open_npc_html(&l2_net::game::NpcHtmlMessage {
        npc_object_id: 2, item_id: 0,
        html: "<html><body>Native NPC window<br><a action=\"bypass npc_test\">Test link</a></body></html>".into(),
    });
    ui.show_board(Some((&vec![String::new(); 8], "<html><body>Native community window</body></html>")));
    ui.help.load(&mut ui.desktop, &ui.small, "<html><body>Native help window</body></html>", &|_, _| None);
    ui.keys.command_channel.set_info(&killer_ui::command_channel::ChannelInfo {
        owner: "Leader".into(), members: 18, parties: 2, rows: vec![("Party A".into(), 9), ("Party B".into(), 9)],
    });
    populate_new_windows(&mut ui, &world);
    ui.warning.show(Some("Aviso de teste do cliente: o valor informado esta fora da faixa permitida."), false);
    let multisell_event = killer_world::WorldEvent::MultiSellList(l2_net::game::MultiSellList {
        list_id: 37, page: 1, finished: true, page_size: 120,
        entries: (1..=120).map(|entry_id| l2_net::game::MultiSellEntry {
            entry_id, stackable: 1,
            products: vec![l2_net::game::MultiSellProduct { item_id: 1060, type2: 5, body_part: 0, count: 1, enchant: 0 }],
            ingredients: vec![l2_net::game::MultiSellIngredient { item_id: 57, type2: 4, count: 100, enchant: 0 }],
        }).collect(),
    });
    world.apply_world(&multisell_event);

    let views = world_views(&world, Some(&tables), Instant::now());
    let ids: Vec<_> = ui.desktop.windows.iter().map(|w| w.id).collect();
    let mut gpu = Headless::new(SIZE.0, SIZE.1).unwrap();
    let mut bank = TextureBank::new();
    for (name, data) in ui.texture_bank.textures() { bank.insert_shared(&name, data); }
    let ctx = egui::Context::default();
    let mut hud = ModernHud::default();
    let mut painter = egui_wgpu::Renderer::new(gpu.renderer_mut().device(), wgpu::TextureFormat::Rgba8Unorm, Default::default());
    let camera = Camera { eye: glam::vec3(0.0, -300.0, 0.0), target: glam::Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    let mut results = Vec::new();
    for id in ids {
        let count = if id == WindowId::MAIN { 5 } else if id == WindowId::OPTION || id == WindowId::INVENTORY { 3 }
            else if id == killer_ui::command_channel::WINDOW_ID || id == WindowId::PARTY_MATCH { 2 } else if matches!(id, WindowId::SIEGE | WindowId::TRAIN | WindowId::MULTISELL | WindowId::DELIVERY | WindowId::SELECT_DELIVERY) { 3 } else if id == WindowId::SSQ { 6 } else { 1 };
        for state in 0..count {
            for window in &mut ui.desktop.windows { window.visible = window.id == id; }
            let window = ui.desktop.window_mut(id).unwrap();
            window.rect.x = 16;
            window.rect.y = 16;
            let rect = window.rect;
            for modern in [false, true] {
                ui.keys.minimized.hide_match();
                for window in &mut ui.desktop.windows { window.visible = window.id == id; }
                ui.shortcut.modern = modern.then(Default::default);
                let mut probes = Vec::new();
                if id == WindowId::MULTISELL {
                    ui.multisell_event(&multisell_event, &world, Some(&tables));
                    ui.keys.multisell.view.select(0);
                } else if id == WindowId::DELIVERY {
                    ui.shop_event(&killer_world::WorldEvent::PackageSendableList(l2_net::game::PackageSendableList {
                        target_id: 77, adena: 123456,
                        items: (0..60).map(|index| l2_net::game::WarehouseItem {
                            warehouse_id: 20000 + index,
                            item: l2_net::game::TradeItem { type1: 4, object_id: 10000 + index, item_id: 1060,
                                count: 1, type2: 5, custom_type1: 0, body_part: 0, enchant: 0, custom_type2: 0 },
                        }).collect(),
                    }), Some(&tables));
                    for _ in 0..12 { ui.keys.delivery.view.move_entry(killer_ui::shop::Side::Top, 0, 1); }
                } else if id == WindowId::SELECT_DELIVERY {
                    ui.keys.select_delivery.open((0..8).map(|i| (i, format!("Character{}", i + 1))));
                    ui.keys.select_delivery.message = crate::ui2d::system_text(Some(&tables), killer_ui::delivery::SELECT_MESSAGE, &[]);
                }
                if matches!(id, WindowId::MULTISELL | WindowId::DELIVERY | WindowId::SELECT_DELIVERY) && state == 1 {
                    let button = if id == WindowId::MULTISELL { killer_ui::multisell::CANCEL_RECT }
                        else if id == WindowId::DELIVERY { killer_ui::shop::CANCEL_RECT }
                        else { killer_ui::delivery::CANCEL_RECT };
                    let handled = ui.mouse_down_at(rect.x + button.x + 1, rect.y + button.y + 1, Instant::now(), &world);
                    let pressed = if id == WindowId::MULTISELL { ui.keys.multisell.view.pressed == Some(killer_ui::multisell::MultiSellHit::Cancel) }
                        else if id == WindowId::DELIVERY { ui.keys.delivery.view.cancel_pressed }
                        else { ui.keys.select_delivery.pressed == Some(killer_ui::delivery::SelectHit::Cancel) };
                    probes.push(json!({"name": "partial_generated_button_press", "passed": handled && pressed && ui.desktop.window(id).unwrap().visible}));
                }
                if id == WindowId::MULTISELL && state == 2 {
                    let grid = killer_ui::multisell::LIST_RECT.offset(rect.x, rect.y);
                    let handled = ui.wheel_over(grid.x + 10, grid.y + 10, -1.0, &[]);
                    probes.push(json!({"name": "partial_generated_grid_scroll", "passed": handled && ui.keys.multisell.view.grid.scroll_row == 1}));
                }
                if id == WindowId::DELIVERY && state == 2 {
                    let grid = ui.keys.delivery.view.top_grid.rect.offset(rect.x, rect.y);
                    let handled = ui.wheel_over(grid.x + 10, grid.y + 10, -1.0, &[]);
                    probes.push(json!({"name": "partial_generated_delivery_scroll", "passed": handled && ui.keys.delivery.view.top_grid.scroll_row == 1}));
                }
                if id == WindowId::SELECT_DELIVERY && state == 2 {
                    let combo = killer_ui::delivery::COMBO_RECT.offset(rect.x, rect.y);
                    let handled = ui.mouse_down_at(combo.x + 1, combo.y + 1, Instant::now(), &world);
                    ui.mouse_up_in(&world);
                    probes.push(json!({"name": "partial_generated_combo_open", "passed": handled && ui.keys.select_delivery.combo.open}));
                }

                if id == WindowId::TRAIN {
                    ui.keys.train.view.info = (state > 0).then(|| killer_ui::train::TrainInfoView {
                        name: "Generated control audit".into(), level: 1, ..Default::default()
                    });
                    ui.keys.train.view.pressed = None;
                    if state == 2 {
                        let button = killer_ui::train::CANCEL_RECT;
                        let handled = ui.mouse_down_at(rect.x + button.x + 4, rect.y + button.y + 4, Instant::now(), &world);
                        probes.push(json!({"name": "generated_button_press", "passed": handled && ui.keys.train.view.pressed == Some(killer_ui::train::TrainHit::Cancel)}));
                    }
                }
                let tab = if id == WindowId::MAIN {
                    ui.keys.main.tab = MainTab::ALL[(state + 1) % 5];
                    Some(killer_ui::mainwnd::tab_rect(MainTab::ALL[state]))
                } else if id == WindowId::OPTION {
                    ui.keys.option.press_tab((state + 1) % 3);
                    Some(ui.keys.option.tabs.buttons[state].paint.rect.offset(
                        killer_ui::option::TABS_RECT.x, killer_ui::option::TABS_RECT.y))
                } else if id == killer_ui::command_channel::WINDOW_ID {
                    ui.keys.command_channel.tab = (state + 1) % 2;
                    Some(killer_ui::command_channel::TAB_RECTS[state])
                } else if id == WindowId::SIEGE {
                    ui.keys.siege.view.select_tab(((state + 1) % 3) as i32);
                    Some(killer_ui::window::Rect::new(12 + state as i32 * 75, 125, 74, 23))
                } else if id == WindowId::SSQ {
                    ui.keys.ssq.select_tab((state % 3 + 1) % 3);
                    Some(killer_ui::window::Rect::new(12 + (state % 3) as i32 * 75, 25, 74, 23))
                } else { None };
                if let Some(tab) = tab {
                    let handled = ui.mouse_down_at(rect.x + tab.x + tab.w / 2, rect.y + tab.y + tab.h / 2, Instant::now(), &world);
                    ui.mouse_up_in(&world);
                    let selected = if id == WindowId::MAIN { ui.keys.main.tab.index() == state }
                        else if id == WindowId::OPTION { ui.keys.option.active_page() == Some(state) }
                        else if id == WindowId::SIEGE { ui.keys.siege.view.tab.index() == state }
                        else if id == WindowId::SSQ { ui.keys.ssq.tab() == state % 3 }
                        else { ui.keys.command_channel.tab == state };
                    probes.push(json!({"name": "tab_click", "passed": handled && selected}));
                }
                if id == WindowId::SSQ && state >= 4 {
                    if state == 4 {
                        ui.keys.ssq.receive_festivals(ui.keys.ssq.period, ui.keys.ssq.festivals.clone());
                    } else {
                        ui.keys.ssq.receive_seals(ui.keys.ssq.period, ui.keys.ssq.retain_limit, ui.keys.ssq.claim_limit, ui.keys.ssq.seals.clone());
                    }
                    ui.keys.ssq.scroll_to(0);
                    let x = rect.x + 20;
                    let y = rect.y + 68;
                    let handled = ui.mouse_down_at(x, y, Instant::now(), &world);
                    ui.mouse_up_in(&world);
                    if state == 4 {
                        ui.mouse_down_at(x + 8, y + ui.small.line_height as i32 + 10, Instant::now(), &world);
                        ui.mouse_up_in(&world);
                    }
                    probes.push(json!({"name": "tree_expand", "passed": handled}));
                }
                if id == WindowId::SSQ && state == 3 && !ui.keys.ssq.prediction_mode {
                    let handled = ui.mouse_down_at(rect.x + 160, rect.y + 382, Instant::now(), &world);
                    ui.mouse_up_in(&world);
                    probes.push(json!({"name": "prediction_toggle", "passed": handled && ui.keys.ssq.prediction_mode}));
                }
                if id == WindowId::SIEGE && state > 0 {
                    ui.keys.siege.view.active_part_mut().unwrap().first = 0;
                    let handled = ui.wheel_over(rect.x + 30, rect.y + 190, -1.0, &[]);
                    probes.push(json!({"name": "siege_wheel", "passed": handled && ui.keys.siege.view.active_part().unwrap().first > 0}));
                }
                if id == WindowId::INVENTORY {
                    ui.keys.inventory.set_first_row(0);
                    ui.keys.inventory.update_rows();
                    let grid = killer_ui::inventory::GRID_RECT.offset(rect.x, rect.y);
                    if state == 1 {
                        let mut handled = true;
                        for _ in 0..ui.keys.inventory.total_rows() {
                            handled &= ui.wheel_over(grid.x + 10, grid.y + 10, -1.0, &[]);
                        }
                        let last = (ui.keys.inventory.total_rows() - killer_ui::inventory::GRID_ROWS as i32).max(0) as usize;
                        probes.push(json!({"name": "wheel_to_bottom", "passed": handled && last > 0 && ui.keys.inventory.scroll_row == last}));
                    } else if state == 2 {
                        let handled = ui.mouse_down_at(grid.x + grid.w - killer_ui::scroll::SCROLL_WIDTH + 3,
                            grid.y + grid.h - 3, Instant::now(), &world);
                        ui.mouse_up_in(&world);
                        probes.push(json!({"name": "scroll_down_click", "passed": handled && ui.keys.inventory.scroll_row == 1}));
                    }
                }
                if id == WindowId::PARTY_MATCH && state == 1 {
                    let button = killer_ui::party_match::MINIMIZE_RECT;
                    let handled = ui.mouse_down_at(rect.x + button.x + 4, rect.y + button.y + 4, Instant::now(), &world);
                    ui.mouse_up_in(&world);
                    probes.push(json!({"name": "minimize", "passed": handled && ui.keys.minimized.party_match.visible && !ui.desktop.window(id).unwrap().visible}));
                }
                ui.take_commands();
                let theme = if modern { "modern" } else { "classic" };
                let stem = format!("{}-{state}-{theme}", id.0);
                let mut draw = DrawList { capture_text: true, ..Default::default() };
                if id == WindowId::PARTY_MATCH && state == 1 {
                    ui.keys.minimized.paint(&mut draw, &ui.small, ui.desktop.size.0);
                } else {
                    ui.paint_window(&mut draw, ui.desktop.window(id).unwrap(), &world, &[], None, &views);
                }
                let native_count = draw.quads.len();
                let textures: BTreeSet<_> = draw.quads.iter().map(|q| q.texture.to_string()).collect();
                for name in &textures { bank.texture(client.catalog(), name); }
                let missing: Vec<_> = textures.iter().filter(|name| bank.texture_available(name) == Some(false)).cloned().collect();
                let mut shapes = 0;
                let mut dispatch_matches = true;
                let pixels = if modern {
                    let native = ui.frame(&world, &[], &[], None, &views);
                    if native.quads.is_empty() {
                        hud.capture_windows(&ui, &world, &[], None, &views);
                        hud.prepare_icons(&ctx, &ui, &mut client);
                        ctx.begin_pass(egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(SIZE.0 as f32, SIZE.1 as f32))),
                            ..Default::default()
                        });
                        hud.paint(&ctx, &ui, &world, &[], 1.0);
                        let (pixels, count) = modern_pixels(&mut gpu, &mut painter, &ctx, ctx.end_pass());
                        shapes = count;
                        pixels
                    } else {
                        gpu.renderer_mut().set_ui(&native, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
                        gpu.render(&camera, &Environment::default()).unwrap()
                    }
                } else {
                    let native = ui.frame(&world, &[], &[], None, &views);
                    dispatch_matches = native.quads == draw.quads;
                    gpu.renderer_mut().set_ui(&native, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
                    gpu.render(&camera, &Environment::default()).unwrap()
                };
                let file = format!("{stem}.png");
                write_png(&directory.join(&file), SIZE.0, SIZE.1, &pixels).unwrap();
                if id == WindowId::TRAIN && state == 2 {
                    ui.mouse_up_in(&world);
                    probes.push(json!({"name": "generated_button_release", "passed": ui.keys.train.view.info.is_none()}));
                }
                if matches!(id, WindowId::MULTISELL | WindowId::DELIVERY | WindowId::SELECT_DELIVERY) && state == 1 {
                    ui.mouse_up_in(&world);
                    probes.push(json!({"name": "partial_generated_button_release", "passed": !ui.desktop.window(id).unwrap().visible}));
                }
                let snapshot = format!("{stem}.json");
                std::fs::write(directory.join(&snapshot), serde_json::to_vec_pretty(&json!({
                    "quads": quad_json(&draw), "text": draw.text_runs.iter().map(|r| &r.text).collect::<Vec<_>>(),
                })).unwrap()).unwrap();
                let status = if probes.iter().any(|probe| probe["passed"] == false) { "input_failed" }
                    else if !dispatch_matches { "dispatch_mismatch" }
                    else if native_count == 0 && shapes == 0 { "no_scenario" }
                    else if missing.is_empty() { "rendered" } else { "missing_textures" };
                results.push(json!({
                    "id": id.0, "state": state, "theme": theme, "status": status,
                    "rect": [rect.x, rect.y, rect.w, rect.h], "native_quads": native_count,
                    "modern_shapes": shapes, "missing_textures": missing, "textures": textures,
                    "dispatch_matches": dispatch_matches,
                    "input_probes": probes,
                    "image": file, "snapshot": snapshot, "behavior_certified": false,
                }));
                std::fs::write(directory.join("windows.json"), serde_json::to_vec_pretty(&json!({
                    "schema": 1, "synthetic": true, "complete_parity": false, "windows": results,
                })).unwrap()).unwrap();
                eprintln!("UI_AUDIT {} {state} {theme}: {status}", id.0);
            }
        }
    }
    assert!(!results.is_empty());
}

#[test]
fn new_windows_receive_events_and_minimized_match_restores_in_both_themes() {
    for modern in [false, true] {
        let mut ui = crate::ui2d::fixture_ui((1024, 768));
        let world = crate::characters::demo_world(glam::Vec3::ZERO, 0, 0);
        ui.shortcut.modern = modern.then(Default::default);
        populate_new_windows(&mut ui, &world);
        for id in [WindowId::PARTY_MATCH, WindowId::SIEGE, WindowId::SSQ] {
            assert!(ui.desktop.window(id).unwrap().visible);
        }
        for window in &mut ui.desktop.windows { window.visible = window.id == WindowId::PARTY_MATCH; }
        let rect = ui.desktop.window(WindowId::PARTY_MATCH).unwrap().rect;
        let min = killer_ui::party_match::MINIMIZE_RECT;
        assert!(ui.mouse_down_at(rect.x + min.x + 4, rect.y + min.y + 4, Instant::now(), &world));
        ui.mouse_up_in(&world);
        assert!(!ui.desktop.window(WindowId::PARTY_MATCH).unwrap().visible);
        assert!(ui.keys.minimized.party_match.visible);
        let icon = ui.keys.minimized.party_match.rect;
        assert_eq!((icon.x, icon.y, icon.w, icon.h), (rect.x, rect.y, 32, 32));
        assert!(ui.mouse_down_at(icon.x + 5, icon.y + 5, Instant::now(), &world));
        assert!(ui.mouse_move(icon.x + 45, icon.y + 35));
        ui.mouse_up_in(&world);
        assert!(ui.keys.minimized.party_match.visible);
        assert!(!ui.desktop.window(WindowId::PARTY_MATCH).unwrap().visible);
        let icon = ui.keys.minimized.party_match.rect;
        assert_eq!((icon.x, icon.y), (rect.x + 40, rect.y + 30));
        assert!(ui.mouse_down_at(icon.x + 4, icon.y + 4, Instant::now(), &world));
        ui.mouse_up_in(&world);
        let restored = ui.desktop.window(WindowId::PARTY_MATCH).unwrap();
        assert!(restored.visible);
        assert_eq!(restored.rect, killer_ui::window::Rect::new(icon.x, icon.y, rect.w, rect.h));
        assert!(!ui.keys.minimized.party_match.visible);
        assert_eq!(ui.keys.minimized.party_match.rect, icon);
        assert!(ui.keys.take_commands().iter().any(|command| matches!(command, crate::ui_keys::UiCommand::InterfaceSound("itemsound.window_open"))));
    }
}

#[test]
fn minimized_match_respects_stacking_resize_and_network_show_cancels_capture() {
    let mut ui = crate::ui2d::fixture_ui((1024, 768));
    let world = crate::characters::demo_world(glam::Vec3::ZERO, 0, 0);
    for window in &mut ui.desktop.windows { window.visible = false; }
    ui.keys.minimized.minimize_match(&mut ui.desktop, "Party");
    let icon = ui.keys.minimized.party_match.rect;
    let point = (icon.x + 4, icon.y + 4);
    assert!(ui.keys.minimized.hit(&ui.desktop, point.0, point.1));
    let window = ui.desktop.window_mut(WindowId::SSQ).unwrap();
    window.rect.x = icon.x;
    window.rect.y = icon.y;
    window.visible = true;
    ui.desktop.bring_to_front(WindowId::SSQ);
    assert!(!ui.keys.minimized.hit(&ui.desktop, point.0, point.1));
    assert_eq!(ui.window_under(point.0, point.1).map(|(id, _, _)| id), Some(WindowId::SSQ));
    ui.desktop.window_mut(WindowId::SSQ).unwrap().visible = false;
    let original_owner = ui.desktop.window(WindowId::PARTY_MATCH).unwrap().rect;
    ui.resized((1280, 960));
    let moved_owner = ui.desktop.window(WindowId::PARTY_MATCH).unwrap().rect;
    let icon = ui.keys.minimized.party_match.rect;
    assert_eq!((icon.x, icon.y), (point.0 - 4 + moved_owner.x - original_owner.x, point.1 - 4 + moved_owner.y - original_owner.y));
    assert!(ui.mouse_down_at(icon.x + 4, icon.y + 4, Instant::now(), &world));
    let event = killer_world::WorldEvent::PartyMatchList(l2_net::game::packets::PartyMatchList { flag: 1, rooms: Vec::new() });
    ui.keys.world_event(&event, &mut ui.desktop, &world);
    ui.mouse_up_in(&world);
    assert!(ui.desktop.window(WindowId::PARTY_MATCH).unwrap().visible);
    assert!(!ui.keys.minimized.party_match.visible);
    assert!(!ui.keys.take_commands().iter().any(|command| matches!(command, crate::ui_keys::UiCommand::InterfaceSound(_))));
}

#[test]
fn siege_date_acceptance_reads_current_callback_state_after_refresh() {
    let mut ui = crate::ui2d::fixture_ui((1024, 768));
    let world = killer_world::World::default();
    let dialog = killer_world::Dialog::Siege {
        message_id: 663, dialog_type: 61, parameter: "old date".into(),
        decision: killer_world::state::SiegeDecision::SetTime { castle_id: 7, time: 123 },
    };
    for (selection, expected_time) in [(0, 0), (1, 456), (2, 789)] {
        ui.keys.siege.view.info.castle_id = 9;
        ui.keys.siege.view.info.choices = vec![456, 789];
        ui.keys.siege.view.date.selected = selection;
        ui.open_dialog(&dialog, &world, None, Instant::now());
        let rect = ui.desktop.window(WindowId::DIALOG).unwrap().rect;
        ui.mouse_down_at(rect.x + killer_ui::dialog::FIRST_BUTTON_X + 4, rect.y + killer_ui::dialog::BUTTON_Y + 4, Instant::now(), &world);
        assert_eq!(ui.dialog_released(), Some(crate::ui2d::KeyAction::Consumed));
        assert!(ui.keys.take_commands().contains(&crate::ui_keys::UiCommand::Net(killer_world::NetCommand::SetCastleSiegeTime { castle_id: 9, time: expected_time })));
    }
}
