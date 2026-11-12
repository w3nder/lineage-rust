pub mod doppler;
pub mod audio;
#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(target_arch = "wasm32")]
mod runtime;
#[cfg(target_arch = "wasm32")]
mod crest_file;
#[cfg(target_arch = "wasm32")]
mod network;
pub mod protocol;
pub mod entry_events;
pub mod table_cache;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/camera.rs"]
pub mod camera;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/trace.rs"]
pub mod trace;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ground.rs"]
pub mod ground;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/walk_world.rs"]
pub mod walk_world;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/input.rs"]
pub mod input;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui2d.rs"]
pub mod ui2d;
#[path = "../../killer/src/ui_status_resize.rs"]
pub mod ui_status_resize;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_siege.rs"]
pub mod ui_siege;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_ssq.rs"]
pub mod ui_ssq;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_minimized.rs"]
pub mod ui_minimized;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_keys.rs"]
pub mod ui_keys;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_key_routes.rs"]
pub mod ui_key_routes;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_options.rs"]
pub mod ui_options;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_olympiad.rs"]
pub mod ui_olympiad;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_windows_info.rs"]
pub mod ui_windows_info;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/appendix_effects.rs"]
pub mod appendix_effects;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/fishing_host.rs"]
pub mod fishing_host;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/zone_music.rs"]
pub mod zone_music;

#[path = "../../killer/src/ui_manor_setting.rs"]
pub mod ui_manor_setting;

#[path = "../../killer/src/ui_hero_tower.rs"]
pub mod ui_hero_tower;

#[path = "../../killer/src/ui_broadcast.rs"]
pub mod ui_broadcast;

#[path = "../../killer/src/ui_messenger.rs"]
pub mod ui_messenger;

#[path = "../../killer/src/ui_petition.rs"]
pub mod ui_petition;

#[path = "../../killer/src/ui_shop.rs"]
pub mod ui_shop;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_trade.rs"]
pub mod ui_trade;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_multisell.rs"]
pub mod ui_multisell;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_train.rs"]
pub mod ui_train;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_henna.rs"]
pub mod ui_henna;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_recipe.rs"]
pub mod ui_recipe;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_recipe_shop.rs"]
pub mod ui_recipe_shop;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_pet.rs"]
pub mod ui_pet;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_special_maps.rs"]
pub mod ui_special_maps;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_macros.rs"]
pub mod ui_macros;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_tutorial.rs"]
pub mod ui_tutorial;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_help.rs"]
pub mod ui_help;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_theme.rs"]
pub mod ui_theme;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_modern.rs"]
pub mod ui_modern;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_modern_windows.rs"]
pub mod ui_modern_windows;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_modern_buffs.rs"]
pub mod ui_modern_buffs;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/characters.rs"]
pub mod characters;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/loader.rs"]
pub mod loader;

#[path = "../../killer/src/notifies.rs"]
pub mod notifies;

#[path = "../../killer/src/world_sound.rs"]
pub mod world_sound;
#[path = "../../killer/src/skill_sound.rs"]
pub mod skill_sound;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/lobby/mod.rs"]
pub mod lobby;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/lobby_c4.rs"]
pub mod lobby_c4;
#[cfg(target_arch = "wasm32")]
mod web_lobby;

#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/pace.rs"]
pub mod pace;

#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_initialize.rs"]
pub mod ui_initialize;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_initialize_run.rs"]
pub mod ui_initialize_run;

#[cfg(all(target_arch = "wasm32", feature = "platform-check"))]
mod checks;

#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/tiles.rs"]
pub mod tiles;

#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/items.rs"]
pub mod items;
#[path = "../../killer/src/server_objects.rs"]
pub mod server_objects;

#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_video.rs"]
pub mod ui_video;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ui_option_apply.rs"]
pub mod ui_option_apply;
#[path = "../../killer/src/ui_option_config.rs"]
pub mod ui_option_config;
#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/ping_cache.rs"]
pub mod ping_cache;

#[cfg(all(target_arch = "wasm32", feature = "platform-check"))]
mod performance;

#[path = "../../killer/src/landmark.rs"]
pub mod landmark;

#[path = "../../killer/src/quest_arrow.rs"]
pub mod quest_arrow;

#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/pawn_effects.rs"]
pub mod pawn_effects;
#[path = "../../killer/src/abnormal_host.rs"]
pub mod abnormal_host;

#[cfg(target_arch = "wasm32")]
mod preparation;

#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/cursor.rs"]
pub mod cursor;

#[cfg(target_arch = "wasm32")]
#[path = "../../killer/src/touch.rs"]
pub mod touch;

#[cfg(target_arch = "wasm32")]
mod lobby_ui;

pub mod lobby_input;
