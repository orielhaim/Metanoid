use bevy::prelude::*;
use metanoid_audio::MetanoidAudioPlugin;
use metanoid_core::events::PowerUpCollectedEvent;
use metanoid_core::rating::{LastRatingResult, LevelRunStats};
use metanoid_core::resources::combo::ComboCounter;
use metanoid_core::resources::game_state::GameState;
use metanoid_core::settings::LevelLaunchMode;
use metanoid_core::states::AppState;
use metanoid_procgen::seed::hierarchy::MasterSeed;
use metanoid_procgen::universe::galaxy::GalaxyDefinition;
use metanoid_ui::MetanoidUiPlugin;
use metanoid_vfx::VfxPlugin;
use metanoid_vfx::particles::setup_particle_effects;
use metanoid_visuals::VisualsPlugin;

mod systems;

use metanoid_core::SaveData;
use systems::arena::{
    setup_arena, setup_camera_effects, setup_persistent_camera, teardown_camera_effects,
};
use systems::background::{setup_background, tag_level_scene};
use systems::ball_physics::{
    anti_stuck_ball, ball_escape, ball_follow_paddle, ball_launch, ball_speed_clamp,
    dev_spawn_balls,
};
use systems::ball_speed_fx::{
    SpeedWhooshCooldown, cleanup_orphaned_aeros, on_paddle_side_hit_fx, spawn_ball_aero,
    speed_overdrive_whoosh, update_ball_aero,
};
use systems::brick_damage::{on_brick_hit_damage, on_brick_regen_clear};
use systems::brick_motion::tick_brick_motion;
use systems::collision::brick::ball_brick_collision;
use systems::collision::paddle::{
    apply_ball_spin_physics, ball_paddle_collision, damp_spin_on_brick_hit,
};
use systems::collision::wall::ball_wall_collision;
use systems::combo::{
    on_brick_destroyed_score, on_brick_hit_combo, on_life_lost_reset_combo,
    on_paddle_hit_reset_combo, update_combo,
};
use systems::diagnostics::{DiagnosticsPlugin, setup_fps_display};
use systems::floating_text::{on_floating_text_event, update_floating_text};
use systems::input::{clamp_paddle_position, paddle_input};
use systems::level_clear::{
    LevelClearTracker, arm_level_clear_tracker, check_level_clear, disarm_level_clear_tracker,
    on_brick_destroyed_track_clear,
};
use systems::level_progression::{handle_life_lost, loading_ready, prepare_level};
use systems::level_spawner::{PendingLevel, auto_respawn_ball};
use systems::lighting::{
    BiomeLighting, BlackoutState, apply_biome_lighting, on_blackout_collected, tick_blackout,
};
use systems::loading_screen::{
    finish_loading, setup_loading_screen, start_curtain, tick_reveal_zoom,
};
use systems::menus::{
    cleanup_play_entities, game_over_button_interaction, game_over_map_interaction,
    pause_button_interaction, pause_menu_button_interaction, setup_game_over, setup_pause,
    teardown_game_over, teardown_pause, toggle_pause,
};
use systems::music_control::{start_level_music, start_menu_music, stop_music_on_game_over};
use systems::post_processing::{AppliedPostFx, pulse_lens_distortion, update_post_processing};
use systems::powerup::board_effects::apply_board_effect;
use systems::powerup::collector::{TimeSlowState, collect_powerup, tick_time_slow};
use systems::powerup::effects::{apply_ball_effect, tick_ball_effects};
use systems::powerup::paddle_effects::{
    apply_paddle_effect, ball_shield_collision, despawn_offscreen_lasers, fire_lasers,
    laser_hit_bricks, tick_paddle_effects,
};
use systems::powerup::spawner::{
    PowerUpState, despawn_offscreen_powerups, fall_powerups, spawn_powerup_on_destroy,
    tick_powerup_glow,
};
use systems::reset::reset_game_effects;
use systems::run_stats::{begin_run_stats_on_play, on_life_lost_track_stats, tick_run_stats};
use systems::save::load_save;
use systems::settings::{load_settings, persist_settings_on_change};
use systems::shake::on_brick_destroyed_shake;
use systems::special_bricks::update_regen_bricks;
use systems::tweens::on_brick_hit_flash;
use systems::ui_backdrop::{setup_ui_backdrop, tag_ui_backdrop, teardown_ui_backdrop};
use systems::vfx::{
    cleanup_orphaned_trails, on_brick_destroyed_debris, on_brick_destroyed_particles,
    spawn_ball_trail_for_new_balls, tick_debris, update_ball_trail_positions,
};

fn resolve_assets_path() -> String {
    let current_dir = std::env::current_dir().unwrap_or_default();
    let target = if current_dir.join("assets").exists() {
        current_dir.join("assets")
    } else {
        current_dir.join("../assets")
    };
    std::fs::canonicalize(&target)
        .map(|p| p.to_string_lossy().trim_start_matches(r"\\?\").to_string())
        .unwrap_or_else(|_| "assets".to_string())
}

fn main() {
    let assets_path = resolve_assets_path();
    let save_data = load_save();
    let settings = load_settings();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Metanoid".into(),
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: assets_path,
                    ..default()
                }),
        )
        .add_plugins(avian2d::PhysicsPlugins::default())
        .add_plugins(VfxPlugin)
        .add_plugins(VisualsPlugin)
        .add_plugins(MetanoidAudioPlugin)
        .add_plugins(DiagnosticsPlugin)
        .add_plugins(MetanoidUiPlugin)
        .add_systems(Startup, setup_persistent_camera)
        .init_state::<AppState>()
        .insert_resource(save_data)
        .insert_resource(settings)
        .init_resource::<PowerUpState>()
        .init_resource::<ComboCounter>()
        .init_resource::<TimeSlowState>()
        .init_resource::<BiomeLighting>()
        .init_resource::<BlackoutState>()
        .init_resource::<LevelRunStats>()
        .init_resource::<LastRatingResult>()
        .init_resource::<LevelLaunchMode>()
        .init_resource::<systems::level_progression::ActiveLevelDifficulty>()
        .init_resource::<LevelClearTracker>()
        .init_resource::<SpeedWhooshCooldown>()
        .init_resource::<AppliedPostFx>()
        // Observers
        .add_observer(handle_life_lost)
        .add_observer(on_life_lost_track_stats)
        .add_observer(on_life_lost_reset_combo)
        .add_observer(on_brick_destroyed_track_clear)
        .add_observer(on_powerup_collected)
        .add_observer(spawn_powerup_on_destroy)
        .add_observer(apply_ball_effect)
        .add_observer(apply_paddle_effect)
        .add_observer(apply_board_effect)
        .add_observer(on_brick_hit_combo)
        .add_observer(on_brick_destroyed_score)
        .add_observer(on_brick_hit_damage)
        .add_observer(on_brick_regen_clear)
        .add_observer(on_paddle_hit_reset_combo)
        .add_observer(on_paddle_side_hit_fx)
        .add_observer(on_brick_destroyed_particles)
        .add_observer(on_brick_destroyed_debris)
        .add_observer(on_blackout_collected)
        .add_observer(on_brick_destroyed_shake)
        .add_observer(on_brick_hit_flash)
        .add_observer(on_floating_text_event)
        // Settings persistence
        .add_systems(Update, (persist_settings_on_change, tag_ui_backdrop))
        // Menu
        .add_systems(
            OnEnter(AppState::Menu),
            (init_game_state, start_menu_music, setup_ui_backdrop),
        )
        .add_systems(OnExit(AppState::Menu), teardown_ui_backdrop)
        // Galaxy map backdrop
        .add_systems(OnEnter(AppState::LevelSelect), setup_ui_backdrop)
        .add_systems(OnExit(AppState::LevelSelect), teardown_ui_backdrop)
        // Loading: prepare the level + recipe first, then build the curtain.
        .add_systems(
            OnEnter(AppState::Loading),
            (prepare_level, setup_loading_screen, setup_particle_effects).chain(),
        )
        .add_systems(
            Update,
            (
                loading_ready
                    .run_if(in_state(AppState::Loading))
                    .run_if(resource_exists::<PendingLevel>),
                setup_background.run_if(in_state(AppState::Loading)),
                setup_arena.run_if(in_state(AppState::Loading)),
                setup_camera_effects.run_if(in_state(AppState::Loading)),
                start_curtain.run_if(in_state(AppState::Loading)),
                tick_reveal_zoom.run_if(in_state(AppState::Loading)),
                finish_loading.run_if(in_state(AppState::Loading)),
            ),
        )
        .add_systems(
            OnEnter(AppState::Playing),
            (
                setup_camera_effects,
                setup_arena,
                setup_background,
                setup_fps_display,
                begin_run_stats_on_play,
                arm_level_clear_tracker,
                start_level_music,
            )
                .chain(),
        )
        .add_systems(
            OnExit(AppState::Playing),
            (teardown_camera_effects, disarm_level_clear_tracker),
        )
        .add_systems(
            Update,
            (
                paddle_input,
                clamp_paddle_position,
                ball_launch,
                ball_speed_clamp,
                apply_ball_spin_physics,
                anti_stuck_ball,
                ball_escape,
                ball_follow_paddle,
                ball_paddle_collision,
                ball_brick_collision,
                ball_wall_collision,
                damp_spin_on_brick_hit,
                collect_powerup,
                fall_powerups,
                despawn_offscreen_powerups,
                toggle_pause,
            )
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            (
                spawn_ball_aero,
                update_ball_aero,
                cleanup_orphaned_aeros,
                tick_powerup_glow,
                speed_overdrive_whoosh,
                dev_spawn_balls,
                update_floating_text,
            )
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            (
                tick_ball_effects,
                tick_paddle_effects,
                tick_time_slow,
                tick_blackout,
                update_combo,
                tick_run_stats,
                tick_brick_motion,
                update_regen_bricks,
                update_post_processing,
                pulse_lens_distortion,
                apply_biome_lighting,
                fire_lasers,
                despawn_offscreen_lasers,
                laser_hit_bricks,
                ball_shield_collision,
            )
                .run_if(in_state(AppState::Playing)),
        )
        // Clear check AFTER combat so health==0 from this frame is visible,
        // and after ApplyDeferred so despawns from earlier systems flush when possible.
        .add_systems(
            Update,
            (
                spawn_ball_trail_for_new_balls,
                update_ball_trail_positions,
                cleanup_orphaned_trails,
                auto_respawn_ball,
                tick_debris,
                tag_level_scene,
            )
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            check_level_clear
                .after(ball_brick_collision)
                .after(laser_hit_bricks)
                .run_if(in_state(AppState::Playing)),
        )
        // Level Complete — cleanup play world
        .add_systems(OnEnter(AppState::LevelComplete), cleanup_play_entities)
        // Paused
        .add_systems(OnEnter(AppState::Paused), setup_pause)
        .add_systems(
            OnExit(AppState::Paused),
            (teardown_pause, reset_game_effects),
        )
        .add_systems(
            Update,
            (
                toggle_pause,
                pause_button_interaction,
                pause_menu_button_interaction,
            )
                .run_if(in_state(AppState::Paused)),
        )
        // Game Over
        .add_systems(
            OnEnter(AppState::GameOver),
            (
                cleanup_play_entities,
                setup_game_over,
                stop_music_on_game_over,
            ),
        )
        .add_systems(
            OnExit(AppState::GameOver),
            (teardown_game_over, reset_game_effects),
        )
        .add_systems(
            Update,
            (game_over_button_interaction, game_over_map_interaction)
                .run_if(in_state(AppState::GameOver)),
        )
        .run();
}

fn init_game_state(
    mut commands: Commands,
    existing_state: Option<Res<GameState>>,
    save: Res<SaveData>,
) {
    if existing_state.is_none() {
        let master = MasterSeed::new(save.master_seed);
        let galaxy_def = GalaxyDefinition::generate(master.galaxy(0));
        let mut state = GameState::new(save.master_seed);
        state.biome_count = galaxy_def.biome_count;
        commands.insert_resource(state);
    }
}

fn on_powerup_collected(trigger: On<PowerUpCollectedEvent>) {
    info!("Power-up collected: {:?}", trigger.kind);
}
