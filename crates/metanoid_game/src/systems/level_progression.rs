use bevy::prelude::*;
use metanoid_core::components::brick::Brick;
use metanoid_core::resources::game_state::GameState;
use metanoid_core::states::AppState;
use metanoid_procgen::biome::generator::BiomeGenerator;
use metanoid_procgen::difficulty::boss::generate_boss_level;
use metanoid_procgen::level::composer::compose_level;
use metanoid_procgen::seed::hierarchy::MasterSeed;
use metanoid_procgen::universe::galaxy::GalaxyDefinition;
use metanoid_procgen::universe::progression::LEVELS_PER_BIOME;

use super::level_spawner::{spawn_bricks, LevelEntity, PendingLevel};
use super::loading_screen::LoadingScreen;

pub fn prepare_level(
    mut commands: Commands,
    game_state: Option<Res<GameState>>,
) {
    let Some(state) = game_state else {
        return;
    };

    let master = MasterSeed::new(state.master_seed);
    let galaxy_seed = master.galaxy(state.galaxy);
    let biome_seed = galaxy_seed.biome(state.biome);
    let biome_params = BiomeGenerator::generate(biome_seed);

    let mut rng = biome_seed.rng();
    let level = if state.is_boss(LEVELS_PER_BIOME) {
        generate_boss_level(&biome_params, &mut rng)
    } else {
        compose_level(&biome_params, &mut rng)
    };

    commands.insert_resource(PendingLevel { level, params: biome_params });
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

    // Reset level clearing flag for the new level
    if let Some(ref mut state) = game_state {
        state.level_clearing = false;
    }

    for entity in &existing_bricks {
        commands.entity(entity).despawn();
    }
    for entity in &existing_level {
        commands.entity(entity).despawn();
    }

    let level = std::mem::replace(
        &mut pending.level,
        metanoid_procgen::level::data::LevelDefinition { cols: 0, rows: 0, bricks: vec![] },
    );
    let params = pending.params;
    commands.remove_resource::<PendingLevel>();

    spawn_bricks(&mut commands, &mut meshes, &mut materials, &level, &params);

    for entity in loading_screen.iter() {
        commands.entity(entity).despawn();
    }

    next_state.set(AppState::Playing);
}

pub fn check_level_clear(
    bricks: Query<&Brick>,
    mut game_state: Option<ResMut<GameState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(ref mut state) = game_state else {
        return;
    };

    // Debounce: don't re-check while a level clear is already in progress
    if state.level_clearing {
        return;
    }

    // Need at least one brick to have been spawned
    if bricks.iter().count() == 0 {
        return;
    }

    let destructible_remaining = bricks
        .iter()
        .filter(|b| {
            matches!(
                b.brick_type,
                metanoid_core::components::brick::BrickType::Normal
                    | metanoid_core::components::brick::BrickType::MultiHit
                    | metanoid_core::components::brick::BrickType::Explosive
                    | metanoid_core::components::brick::BrickType::Moving
                    | metanoid_core::components::brick::BrickType::Regenerating
            )
        })
        .count();

    if destructible_remaining > 0 {
        return;
    }

    // Mark as clearing to prevent re-triggering
    state.level_clearing = true;

    info!(
        "Level cleared! Galaxy {} Biome {} Level {} — Score: {}",
        state.galaxy, state.biome, state.level, state.score
    );

    state.level += 1;

    if state.level >= LEVELS_PER_BIOME {
        state.level = 0;
        state.biome += 1;

        if state.biome >= state.biome_count as u64 {
            state.biome = 0;
            state.galaxy += 1;
            let master = MasterSeed::new(state.master_seed);
            let galaxy_def = GalaxyDefinition::generate(master.galaxy(state.galaxy));
            state.biome_count = galaxy_def.biome_count;
            info!("New galaxy {} with {} biomes!", state.galaxy, galaxy_def.biome_count);
        }

        info!("New biome {}!", state.biome);
    }

    // Transition to level complete screen — entities cleaned up on exit from Playing
    next_state.set(AppState::LevelComplete);
}

pub fn handle_life_lost(
    _trigger: On<metanoid_core::events::LifeLostEvent>,
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
        next_state.set(AppState::GameOver);
    }
}
