use killer_assets::abnormal_effects::AbnormalEffectHost;
use killer_assets::abnormal_state::{
    AbnormalStat, AbnormalStatHost, GROW_GROWING, GROW_HELD, GROW_IDLE, GROW_SHRINKING, GROW_TARGET, GrowDimensions,
    NpcTextureLists, STATE_BIG_HEAD, STATE_GROW, STATE_MATERIAL_SEQUENCE, STATE_BLEED, STATE_FLAME, STATE_HOLD, STATE_MUTED, STATE_OVERLAY,
    STATE_POISON, STATE_ROOT, STATE_SLEEP, STATE_STUN, check_state, effect_class, init, reconcile, update,
};
use killer_assets::pawn_light::PawnLight;
use killer_assets::texture_modifier::TextureModifyInfo;

struct Host {
    events: Vec<String>,
    skeletal: bool,
    drawn: bool,
    has_alias: bool,
    class_ok: bool,
    spawn_ok: bool,
    attach_ok: bool,
    info: [TextureModifyInfo; 2],
    next_emitter: u32,
    associated: bool,
    size: (f32, f32),
    npc: bool,
    dims: GrowDimensions,
    textures: (Vec<u32>, Vec<u32>),
    lights: Vec<PawnLight>,
}

impl Host {
    fn ready() -> Host {
        Host { skeletal: true, drawn: true, has_alias: true, class_ok: true, spawn_ok: true, attach_ok: true, next_emitter: 100, size: (9.0, 23.0), events: Vec::new(), info: [BLANK; 2], associated: false, npc: false, dims: GrowDimensions::default(), textures: (Vec::new(), Vec::new()), lights: Vec::new() }
    }

    fn log(&mut self, text: impl Into<String>) {
        self.events.push(text.into());
    }

    fn take(&mut self) -> Vec<String> {
        std::mem::take(&mut self.events)
    }
}

impl AbnormalEffectHost for Host {
    type Actor = usize;
    type Mesh = usize;
    type Emitter = u32;
    type Name = u32;
    type Material = u32;
    fn destroy_emitter(&mut self, _: usize, emitter: u32) {
        self.log(format!("destroy {emitter}"));
    }
    fn report_missing_emitter_owner(&mut self) {}
    fn is_pawn(&self, _: usize) -> bool {
        true
    }
    fn active_associated_pawn(&self, _: usize) -> Option<usize> {
        self.associated.then_some(1)
    }
    fn skeletal_instance(&self, actor: usize) -> Option<usize> {
        Some(actor)
    }
    fn texture_info(&mut self, actor: usize) -> &mut TextureModifyInfo {
        &mut self.info[actor]
    }
    fn clear_texture_modifier(&mut self, _: usize) {}
    fn head_bone(&mut self, _: usize) -> u32 {
        0
    }
    fn clear_bone_scaler(&mut self, _: usize, _: u32) {}
    fn clear_scale_fields(&mut self, _: usize) {}
    fn root_scaler_index(&self, _: usize) -> Option<usize> {
        None
    }
    fn remove_scaler(&mut self, _: usize, _: usize) {}
    fn restore_field_4bc(&mut self, _: usize, _: u32) {}
    fn has_game_data(&self) -> bool {
        false
    }
    fn has_mesh_instance(&self, _: usize) -> bool {
        self.skeletal
    }
    fn has_npc_texture_table(&self, _: usize) -> bool {
        false
    }
    fn normal_npc_textures(&mut self, _: usize) -> Vec<u32> {
        Vec::new()
    }
    fn find_material_sequence(&mut self, name: u32) -> Option<u32> {
        self.log(format!("find {name}"));
        name.is_multiple_of(2).then_some(name)
    }
    fn load_material_sequence(&mut self, name: u32) -> Option<u32> {
        self.log(format!("load {name}"));
        name.is_multiple_of(3).then_some(name)
    }
    fn trigger_material_sequence(&mut self, material: u32) {
        self.log(format!("trigger {material}"));
    }
    fn set_npc_texture(&mut self, actor: usize, name: u32, index: usize) {
        self.log(format!("texes {actor} {name} {index}"));
    }
}

impl AbnormalStatHost for Host {
    type Class = u32;
    fn has_skeletal_mesh(&self, _: usize) -> bool {
        self.skeletal
    }
    fn pawn_drawn(&self, _: usize) -> bool {
        self.drawn
    }
    fn load_effect_class(&mut self, name: &str) -> Option<u32> {
        self.log(format!("class {name}"));
        self.class_ok.then_some(7)
    }
    fn spawn_emitter(&mut self, _: u32, _: usize, location: [f32; 3], rotation: [i32; 3]) -> Option<u32> {
        self.log(format!("spawn {location:?} {rotation:?}"));
        if !self.spawn_ok {
            return None;
        }
        self.next_emitter += 1;
        Some(self.next_emitter)
    }
    fn location_rotation(&self, _: usize) -> ([f32; 3], [i32; 3]) {
        ([10.0, 20.0, 30.0], [0, 16384, 0])
    }
    fn collision(&self, _: usize) -> (f32, f32) {
        self.size
    }
    fn draw_scale(&self, _: usize) -> f32 {
        1.5
    }
    fn alias_bone(&self, _: usize, alias: &str) -> Option<u32> {
        (self.has_alias && alias == "e_bone").then_some(55)
    }
    fn alias_origin(&self, _: usize, _: &str) -> [f32; 3] {
        [1.0, 2.0, 3.0]
    }
    fn mesh_origin_z(&self, _: usize) -> f32 {
        4.0
    }
    fn attach_to_bone(&mut self, _: usize, emitter: u32, bone: u32, relative: [f32; 3]) -> bool {
        self.log(format!("bone {emitter} {bone} {relative:?}"));
        self.attach_ok
    }
    fn attach_to_root(&mut self, _: usize, emitter: u32) {
        self.log(format!("root {emitter}"));
    }
    fn set_pre_pivot_z(&mut self, emitter: u32, value: f32) {
        self.log(format!("pivot {emitter} {value}"));
    }
    fn set_size_scale(&mut self, emitter: u32, scale: f32) {
        self.log(format!("scale {emitter} {scale}"));
    }
    fn set_overlay(&mut self, owner: usize, rgb: [u8; 3], mode: u8) {
        self.log(format!("overlay {owner} {rgb:?} {mode}"));
    }
    fn stop_action(&mut self, owner: usize) {
        self.log(format!("stop {owner}"));
    }
    fn head_scale(&self, _: usize) -> f32 {
        2.0
    }
    fn set_bone_scale(&mut self, owner: usize, scale: f32, head: bool) {
        self.log(format!("bone_scale {owner} {scale} {head}"));
    }
    fn big_head_max(&mut self, _: usize) -> f32 {
        2.3
    }
    fn play_damage_sound(&mut self, owner: usize) {
        self.log(format!("damage_sound {owner}"));
    }
    fn pawn_lights(&mut self, owner: usize) -> Option<&mut Vec<PawnLight>> {
        self.log(format!("light {owner}"));
        Some(&mut self.lights)
    }
    fn play_sound(&mut self, owner: usize, name: &str, volume: f32, radius: f32) {
        self.log(format!("sound {owner} {name} {volume} {radius}"));
    }
    fn is_npc(&self, _: usize) -> bool {
        self.npc
    }
    fn npc_texture_lists(&mut self, actor: usize) -> Option<NpcTextureLists<u32>> {
        self.log(format!("table {actor}"));
        Some(NpcTextureLists { normal: self.textures.0.clone(), abnormal: self.textures.1.clone() })
    }
    fn grow_dimensions(&self, _: usize) -> GrowDimensions {
        self.dims
    }
    fn set_collision_size(&mut self, _: usize, size: [f32; 2]) {
        self.dims.collision = size;
    }
    fn set_location_z(&mut self, _: usize, z: f32) {
        self.dims.z = z;
    }
    fn set_weapon_scale(&mut self, _: usize, scale: f32) {
        self.dims.weapon_scale = scale;
    }
}

const BLANK: TextureModifyInfo = TextureModifyInfo { flags: 0, color: 0, value_08: 0, value_0c: 0 };

type Stat = AbnormalStat<usize, u32>;

fn run(host: &mut Host, stats: &mut Vec<Stat>, mask: u32, dt: f32) -> Vec<Stat> {
    reconcile(host, 0, stats, mask, dt)
}

#[test]
fn the_bit_to_class_table_matches_the_init_strings_of_the_engine() {
    let expect = [
        (0x1, "a_u001_a"), (0x4, "a_u001_a"), (0x8, "a_u001_a"), (0x10, "a_u001_a"), (0x2, "a_u000_a"), (0x20, "a_u008_a"),
        (0x40, "a_u002_a"), (0x80, "a_u004_a"), (0x100, "a_u005_a"), (0x200, "a_u006_a"), (0x4000, "a_u007_a"),
    ];
    for (bit, name) in expect {
        assert_eq!(effect_class(bit), Some(format!("LineageEffect.{name}").as_str()), "{bit:#x}");
    }
    for bit in [0x400, 0x800, 0x1000, 0x2000, 0x8000, 0x10000, 0x20000] {
        assert_eq!(effect_class(bit), None, "{bit:#x}");
    }
}

#[test]
fn creation_follows_the_sign_of_the_mask_and_the_bit_range() {
    let mut host = Host::ready();
    let mut stats = Vec::new();
    run(&mut host, &mut stats, 0, 0.1);
    assert!(stats.is_empty());
    run(&mut host, &mut stats, 0x8000_0000 | STATE_BLEED, 0.1);
    assert!(stats.is_empty(), "o bit 31 desliga toda a criação (comparação com sinal)");
    run(&mut host, &mut stats, 0x20000, 0.1);
    assert!(stats.is_empty());
    run(&mut host, &mut stats, 0x1000, 0.1);
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].mask, 0x1000);
}

#[test]
fn bits_are_created_in_increasing_order_and_each_new_stat_goes_to_the_front() {
    let mut host = Host::ready();
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_BLEED | STATE_POISON | STATE_FLAME, 0.1);
    let masks: Vec<u32> = stats.iter().map(|s| s.mask).collect();
    assert_eq!(masks, [STATE_FLAME, STATE_POISON, STATE_BLEED]);
}

#[test]
fn a_stat_without_a_skeletal_mesh_keeps_mask_zero_and_is_dropped_next_frame() {
    let mut host = Host { skeletal: false, ..Host::ready() };
    let mut stats = Vec::new();
    let removed = run(&mut host, &mut stats, STATE_BLEED, 0.1);
    assert_eq!(stats.len(), 0);
    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0].mask, 0);
}

#[test]
fn a_stat_whose_bit_disappeared_is_removed_unless_it_is_growing() {
    let mut host = Host::ready();
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_STUN, 0.1);
    assert_eq!(stats.len(), 1);
    let removed = run(&mut host, &mut stats, 0, 0.1);
    assert!(stats.is_empty());
    assert_eq!(removed.len(), 1);
    let mut growing: Stat = init(&mut host, 0, 0x10000, false);
    growing.state = 3;
    growing.scale = GROW_TARGET;
    let mut stats = vec![growing];
    let removed = run(&mut host, &mut stats, 0, 0.1);
    assert!(removed.is_empty());
    assert_eq!(stats[0].state, 2);
    assert!(!check_state(&host, &stats[0], 0));
}

#[test]
fn the_periodic_effect_is_recreated_when_elapsed_passes_three_seconds_keeping_the_remainder() {
    let mut host = Host::ready();
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_BLEED, 1.0);
    assert_eq!(stats[0].elapsed, 1.0);
    let first = host.take();
    assert_eq!(first[0], "class LineageEffect.a_u001_a");
    assert!(first[1].starts_with("spawn [10.0, 20.0, 30.0] [0, 16384, 0]"));
    assert_eq!(first[2], "root 101");
    assert_eq!(first[3], "light 0");
    assert_eq!(first[4], "damage_sound 0");
    assert_eq!(first.len(), 5, "{first:?}");
    for expected in [2.0, 3.0, 4.0] {
        run(&mut host, &mut stats, STATE_BLEED, 1.0);
        assert_eq!(stats[0].elapsed, expected);
    }
    assert!(host.take().is_empty());
    run(&mut host, &mut stats, STATE_BLEED, 1.0);
    assert_eq!(stats[0].elapsed, 2.0);
    let renewed = host.take();
    assert_eq!(renewed[0], "destroy 101");
    assert_eq!(renewed[1], "class LineageEffect.a_u001_a");
}

#[test]
fn the_size_scale_is_only_applied_when_it_is_above_one() {
    let mut host = Host::ready();
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_BLEED, 1.0);
    assert!(!host.events.iter().any(|e| e.starts_with("scale")), "sqrt(23 * 9) / sqrt(207) é exatamente 1");
    let mut host = Host { size: (12.0, 30.0), ..Host::ready() };
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_BLEED, 1.0);
    let expected = ((30.0f64 * 12.0).sqrt() / 207.0f64.sqrt()) as f32;
    assert!(host.events.contains(&format!("scale 101 {expected}")), "{:?}", host.events);
}

#[test]
fn a_pawn_that_is_not_drawn_loses_its_periodic_stat_every_frame() {
    let mut host = Host { drawn: false, ..Host::ready() };
    let mut stats = Vec::new();
    let removed = run(&mut host, &mut stats, STATE_BLEED, 1.0);
    assert!(stats.is_empty());
    assert_eq!(removed.len(), 1);
}

#[test]
fn poison_attaches_to_the_e_bone_alias_and_falls_back_to_the_root_bone() {
    let mut host = Host::ready();
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_POISON, 1.0);
    let events = host.take();
    assert!(events.contains(&"bone 101 55 [1.0, 2.0, 3.0]".to_string()), "{events:?}");
    assert!(!events.iter().any(|e| e.starts_with("scale")));
    let mut host = Host { attach_ok: false, ..Host::ready() };
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_POISON, 1.0);
    assert!(host.take().contains(&"root 101".to_string()));
    let mut host = Host { has_alias: false, ..Host::ready() };
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_POISON, 1.0);
    assert!(host.take().iter().all(|e| !e.starts_with("spawn")));
}

#[test]
fn the_flame_sits_at_the_pawn_with_the_pre_pivot_and_the_radius_scale() {
    let mut host = Host::ready();
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_FLAME, 1.0);
    let events = host.take();
    assert!(events.contains(&"pivot 101 -4".to_string()), "{events:?}");
    assert!(events.contains(&"scale 101 1".to_string()), "{events:?}");
    assert!(!events.iter().any(|e| e.starts_with("bone") || e.starts_with("root")));
}

#[test]
fn stun_and_sleep_create_at_init_attach_and_stop_the_current_action() {
    for bit in [STATE_STUN, STATE_SLEEP] {
        let mut host = Host::ready();
        let mut stats = Vec::new();
        run(&mut host, &mut stats, bit, 1.0);
        let events = host.take();
        assert!(events[0].starts_with("class LineageEffect.a_u00"), "{events:?}");
        assert!(events.contains(&"bone 101 55 [1.0, 2.0, 3.0]".to_string()));
        assert_eq!(events.last().unwrap(), "stop 0");
        assert!(stats[0].stopped);
        assert_eq!(stats[0].emitter, Some(101));
    }
    let mut host = Host { has_alias: false, ..Host::ready() };
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_STUN, 1.0);
    assert_eq!(host.take(), ["stop 0"]);
}

#[test]
fn muted_attaches_without_stopping_and_root_spawns_at_the_feet() {
    let mut host = Host::ready();
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_MUTED, 1.0);
    let events = host.take();
    assert!(!events.iter().any(|e| e.starts_with("stop")));
    assert!(!stats[0].stopped);
    let mut host = Host::ready();
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_ROOT, 1.0);
    let events = host.take();
    assert!(events[1].starts_with("spawn [10.0, 20.0, 7.0]"), "{events:?}");
    assert!(events.contains(&"pivot 101 -6".to_string()));
    assert!(events.contains(&"scale 101 1".to_string()));
    assert!(stats[0].stopped);
}

#[test]
fn hold_stops_the_action_and_overlay_colors_the_pawn_and_its_associated_one() {
    let mut host = Host::ready();
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_HOLD, 1.0);
    assert_eq!(host.take(), ["stop 0"]);
    let mut host = Host { associated: true, ..Host::ready() };
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_OVERLAY, 1.0);
    assert_eq!(host.take(), ["overlay 0 [72, 105, 140] 1", "stop 0", "overlay 1 [72, 105, 140] 1"]);
}

#[test]
fn big_head_is_detected_from_the_stat_list_and_scales_the_alias_offset() {
    let mut host = Host::ready();
    let mut stats: Vec<Stat> = vec![init(&mut host, 0, STATE_BIG_HEAD, false)];
    assert!(stats[0].mask == STATE_BIG_HEAD);
    host.take();
    run(&mut host, &mut stats, STATE_BIG_HEAD | STATE_POISON, 1.0);
    let events = host.take();
    assert!(events.contains(&"bone 101 55 [2.0, 4.0, 6.0]".to_string()), "{events:?}");
}

#[test]
fn big_head_104946f0_resets_the_head_and_grows_until_the_bighead_int_max() {
    let mut host = Host::ready();
    let stat = init(&mut host, 0, STATE_BIG_HEAD, false);
    assert_eq!((stat.scale, stat.duration), (1.0, 2.3));
    assert_eq!(host.take(), ["bone_scale 0 1 true"]);
    let mut stats = vec![stat];
    run(&mut host, &mut stats, STATE_BIG_HEAD, 0.5);
    assert_eq!(stats[0].scale, 1.0);
    assert_eq!(stats[0].elapsed, 1.0);
    run(&mut host, &mut stats, STATE_BIG_HEAD, 0.5);
    assert_eq!(stats[0].scale, 2.5);
    assert_eq!(stats[0].elapsed, 2.0);
    assert_eq!(host.take(), ["bone_scale 0 1 true", "bone_scale 0 2.5 true"]);
    run(&mut host, &mut stats, STATE_BIG_HEAD, 0.5);
    assert_eq!(stats[0].scale, 2.5);
    assert!(host.take().is_empty());
}

#[test]
fn fear_and_poison_play_their_extra_shot_sounds_after_the_damage_sound() {
    let mut host = Host::ready();
    let mut stats = Vec::new();
    run(&mut host, &mut stats, 0x20, 1.0);
    let events = host.take();
    assert_eq!(&events[events.len() - 2..], ["damage_sound 0", "sound 0 skillsound3.soil_shot 1 5"]);
    let mut host = Host::ready();
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_POISON, 1.0);
    let events = host.take();
    assert_eq!(&events[events.len() - 2..], ["damage_sound 0", "sound 0 skillsound.curse_poison_shot 1 5"]);
}

fn grower() -> Host {
    let mut host = Host::ready();
    host.npc = true;
    host.dims = GrowDimensions { collision: [30.0, 60.0], base_collision: [25.0, 50.0], z: 100.0, weapon_scale: 0.0 };
    host
}

#[test]
fn grow_init_10494817_swaps_the_collision_for_the_second_pair_and_starts_growing() {
    let mut host = grower();
    host.dims.weapon_scale = 1.5;
    let stat = init(&mut host, 0, STATE_GROW, false);
    assert_eq!((stat.mask, stat.state, stat.scale, stat.duration, stat.elapsed), (STATE_GROW, GROW_GROWING, 1.0, GROW_TARGET, 0.0));
    assert_eq!(stat.saved_collision, [30.0, 60.0]);
    assert_eq!(stat.saved_weapon_scale, 1.5);
    assert_eq!(host.dims.collision, [25.0, 50.0]);
    assert_eq!(host.dims.z, 100.0);
    let mut players = Host::ready();
    players.dims = host.dims;
    let player = init(&mut players, 0, STATE_GROW, false);
    assert_eq!((player.mask, player.state, player.duration), (STATE_GROW, GROW_IDLE, 0.0));
}

#[test]
fn grow_update_10495f2c_accelerates_clamps_and_moves_the_cylinder_and_z() {
    let mut host = grower();
    let mut stat = init(&mut host, 0, STATE_GROW, false);
    assert!(update(&mut host, &mut stat, 0.5, false));
    assert_eq!((stat.scale, stat.elapsed), (1.0, 0.5));
    assert_eq!(host.dims.collision, [25.0, 50.0]);
    host.take();
    assert!(update(&mut host, &mut stat, 0.2, false));
    assert_eq!(stat.scale, 1.1);
    assert_eq!(host.take(), ["bone_scale 0 1.1 false"]);
    let factor = (f64::from(1.1f32) - 1.0) / (f64::from(GROW_TARGET) - 1.0);
    assert_eq!(host.dims.collision[0], ((30.0 - 25.0) * factor + 25.0) as f32);
    let height = (60.0 - 50.0) * factor + 50.0;
    assert_eq!(host.dims.collision[1], height as f32);
    assert_eq!(host.dims.z, ((height - 50.0) + 100.0) as f32);
    assert!(update(&mut host, &mut stat, 1.0, false));
    assert_eq!((stat.scale, stat.state), (GROW_TARGET, GROW_HELD));
    assert_eq!(host.dims.collision, [30.0, 60.0]);
    assert!((host.dims.z - 110.0).abs() < 1e-3);
    let elapsed = stat.elapsed;
    assert!(elapsed > 0.0);
    assert!(update(&mut host, &mut stat, 1.0, false));
    assert_eq!((stat.elapsed, stat.saved_collision), (0.0, [25.0, 50.0]));
}

#[test]
fn grow_shrink_10495e38_restores_the_base_cylinder_without_moving_z_and_returns_to_idle() {
    let mut host = grower();
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_GROW, 0.5);
    for _ in 0..8 {
        run(&mut host, &mut stats, STATE_GROW, 0.5);
    }
    assert_eq!(stats[0].state, GROW_HELD);
    let held_z = host.dims.z;
    assert!((held_z - 110.0).abs() < 1e-3);
    let removed = run(&mut host, &mut stats, 0, 0.25);
    assert!(removed.is_empty());
    assert_eq!(stats[0].state, GROW_SHRINKING);
    assert_eq!((stats[0].scale, stats[0].elapsed), (GROW_TARGET, 0.25));
    assert_eq!(host.dims.collision, [25.0, 50.0]);
    assert_eq!(host.dims.z, held_z);
    run(&mut host, &mut stats, 0, 0.5);
    assert_eq!((stats[0].scale, stats[0].state), (1.0, GROW_IDLE));
    let removed = run(&mut host, &mut stats, 0, 0.25);
    assert_eq!(removed.len(), 1);
    assert!(stats.is_empty());
}

#[test]
fn grow_weapon_scale_4bc_follows_the_scale_only_when_set() {
    let mut host = grower();
    host.dims.weapon_scale = 2.0;
    let mut stat = init(&mut host, 0, STATE_GROW, false);
    update(&mut host, &mut stat, 0.5, false);
    update(&mut host, &mut stat, 0.2, false);
    assert_eq!(host.dims.weapon_scale, (2.0f64 * f64::from(1.1f32)) as f32);
    let mut zero = grower();
    let mut stat = init(&mut zero, 0, STATE_GROW, false);
    update(&mut zero, &mut stat, 0.5, false);
    update(&mut zero, &mut stat, 0.2, false);
    assert_eq!(zero.dims.weapon_scale, 0.0);
    assert_eq!(stat.into_effect(String::new()).saved_field_4bc, 0);
}

#[test]
fn material_sequence_init_10494875_walks_the_abnormal_names_inside_the_normal_count() {
    let mut host = Host::ready();
    host.npc = true;
    host.textures = (vec![11, 12, 13], vec![20, 21, 22, 23]);
    let stat = init(&mut host, 0, STATE_MATERIAL_SEQUENCE, false);
    assert_eq!(stat.mask, STATE_MATERIAL_SEQUENCE);
    assert_eq!(
        host.take(),
        ["table 0", "find 20", "trigger 20", "texes 0 20 0", "find 21", "load 21", "trigger 21", "texes 0 21 1", "find 22", "trigger 22", "texes 0 22 2"]
    );
    host.textures = (vec![11, 12, 13], vec![20]);
    init(&mut host, 0, STATE_MATERIAL_SEQUENCE, false);
    assert_eq!(host.take(), ["table 0", "find 20", "trigger 20", "texes 0 20 0"]);
    host.npc = false;
    init(&mut host, 0, STATE_MATERIAL_SEQUENCE, false);
    assert!(host.take().is_empty());
}

#[test]
fn each_renewal_adds_the_pulsing_pawn_light_before_the_damage_sound() {
    let mut host = Host::ready();
    let mut stats = Vec::new();
    run(&mut host, &mut stats, STATE_POISON, 0.1);
    let events = host.take();
    let light = events.iter().position(|event| event == "light 0").expect("luz");
    let sound = events.iter().position(|event| event == "damage_sound 0").expect("som");
    assert!(light < sound);
    assert_eq!(host.lights.len(), 1);
    let poison = host.lights[0];
    assert_eq!(poison.location, [18.0, 0.0, 46.0]);
    assert_eq!(poison.plane, [f32::from_bits(0x3e80_8081), 0.0, f32::from_bits(0x3ec0_c0c1), 1.0]);
    assert_eq!((poison.light_type, poison.effect, poison.attach, poison.period, poison.phase), (2, 12, 1, 0x19, 0xc0));
    assert_eq!(poison.radius, 1050.0);
    assert_eq!(poison.life, f32::from_bits(0x3f33_3333));
    run(&mut host, &mut stats, STATE_POISON | STATE_FLAME, 0.1);
    assert_eq!(host.lights[0].plane, [f32::from_bits(0x3ec0_c0c1), 0.0, 0.0, 1.0]);
}
