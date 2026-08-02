use bevy::prelude::*;
use metanoid_core::components::brick::Brick;
use metanoid_core::resources::game_state::GameState;
use metanoid_core::states::AppState;
use metanoid_procgen::level::generate::generate_level_at;

use super::level_spawner::{LevelEntity, PendingLevel, spawn_bricks};
use super::loading_screen::LoadingScreen;

/// Per-level difficulty multipliers applied during play (ball speed, etc.).
#[derive(Resource, Debug, Clone, Copy)]
pub struct ActiveLevelDifficulty {
    pub ball_speed_mult: f32,
    pub level_index: u64,
    pub is_boss: bool,
}

impl Default for ActiveLevelDifficulty {
    fn default() -> Self {
        Self {
            ball_speed_mult: 1.0,
            level_index: 0,
            is_boss: false,
        }
    }
}

pub fn prepare_level(mut commands: Commands, game_state: Option<Res<GameState>>) {
    let Some(state) = game_state else {
        return;
    };

    // Unique per (master, galaxy, biome, level) — not biome-only RNG.
    let generated = generate_level_at(state.master_seed, state.galaxy, state.biome, state.level);

    info!(
        "Generated level G{} B{} L{} (boss={}) bricks={} seed={:?} ball_mult={:.2}",
        state.galaxy,
        state.biome,
        state.level,
        generated.is_boss,
        generated.definition.bricks.len(),
        generated.level_seed,
        generated.difficulty.ball_speed_mult,
    );

    commands.insert_resource(ActiveLevelDifficulty {
        ball_speed_mult: generated.difficulty.ball_speed_mult,
        level_index: state.level,
        is_boss: generated.is_boss,
    });

    commands.insert_resource(PendingLevel {
        level: generated.definition,
        params: generated.biome_params,
    });
}

pub fn loading_ready(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    pending: Option<ResMut<PendingLevel>>,
    existing_bricks: Query<Entity, With<Brick>>,
    existing_level: Query<Entity, With<LevelEntity>>,
    loading_screen: Query<Entity, With<LoadingScreen>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_state: Option<ResMut<GameState>>,
) {
    let Some(mut pending) = pending else {
        return;
    };

    if let Some(ref mut state) = game_state {
        state.level_clearing = false;
    }

    for entity in &existing_bricks {
        commands.entity(entity).try_despawn();
    }
    for entity in &existing_level {
        commands.entity(entity).try_despawn();
    }

    let level = std::mem::replace(
        &mut pending.level,
        metanoid_procgen::level::data::LevelDefinition {
            cols: 0,
            rows: 0,
            bricks: vec![],
            metrics: Default::default(),
        },
    );
    let params = pending.params;
    commands.remove_resource::<PendingLevel>();

    spawn_bricks(&mut commands, &mut meshes, &mut materials, &level, &params);

    for entity in loading_screen.iter() {
        commands.entity(entity).try_despawn();
    }

    next_state.set(AppState::Playing);
}

// Level-clear logic lives in `level_clear.rs` (robust empty-query + deferred despawn handling).

pub fn handle_life_lost(
    _trigger: On<metanoid_core::events::LifeLostEvent>,
    mut commands: Commands,
    mut game_state: Option<ResMut<GameState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(ref mut state) = game_state else {
        return;
    };

    state.lives -= 1;
    info!("Lives remaining: {}", state.lives);

    if state.lives <= 0 {
        info!("Game Over! Final score: {}", state.score);
        commands.trigger(metanoid_core::events::GameOverEvent);
        next_state.set(AppState::GameOver);
    }
}
