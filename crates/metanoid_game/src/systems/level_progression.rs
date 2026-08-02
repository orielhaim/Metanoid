use bevy::prelude::*;
use metanoid_core::components::brick::Brick;
use metanoid_core::resources::game_state::GameState;
use metanoid_core::states::AppState;
use metanoid_procgen::biome::composition::{BiomeComposition, generate_composition};
use metanoid_procgen::level::generate::generate_level_at;
use metanoid_procgen::seed::hierarchy::MasterSeed;
use metanoid_procgen::universe::galaxy::GalaxyDefinition;
use metanoid_procgen::universe::progression::LEVELS_PER_BIOME;
use metanoid_visuals::material::{ProceduralMaterials, bake_all};
use metanoid_visuals::recipe::{BiomeRecipe, LevelVisualContext, recipe_for_context};

use super::level_spawner::{LevelEntity, PendingLevel, spawn_bricks};

/// Per-level difficulty multipliers applied during play (ball speed, etc.).
#[derive(Resource, Debug, Clone, Copy)]
pub struct ActiveLevelDifficulty {
    pub ball_speed_mult: f32,
    #[allow(dead_code)]
    pub level_index: u64,
    #[allow(dead_code)]
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

/// Everything the visuals need for the current level, computed once at load.
#[derive(Resource, Clone)]
pub struct ActiveLevelVisuals {
    pub recipe: BiomeRecipe,
    pub materials: ProceduralMaterials,
    #[allow(dead_code)]
    pub context: LevelVisualContext,
}

pub fn prepare_level(
    mut commands: Commands,
    game_state: Option<Res<GameState>>,
    mut images: ResMut<Assets<Image>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(state) = game_state else {
        return;
    };

    // Generate the level package (definition + difficulty + params).
    let generated = generate_level_at(state.master_seed, state.galaxy, state.biome, state.level);

    info!(
        "Generated level G{} B{} L{} (boss={}) bricks={} ball_mult={:.2}",
        state.galaxy,
        state.biome,
        state.level,
        generated.is_boss,
        generated.definition.bricks.len(),
        generated.difficulty.ball_speed_mult,
    );

    commands.insert_resource(ActiveLevelDifficulty {
        ball_speed_mult: generated.difficulty.ball_speed_mult,
        level_index: state.level,
        is_boss: generated.is_boss,
    });

    // Build the visual context from neighboring biomes + this biome's composition.
    let master = MasterSeed::new(state.master_seed);
    let galaxy_seed = master.galaxy(state.galaxy);
    let galaxy_def = GalaxyDefinition::generate(galaxy_seed);
    let biome_count = galaxy_def.biome_count.max(1) as u64;

    let cur_seed = galaxy_seed.biome(state.biome);
    let prev_seed = if state.biome > 0 {
        Some(galaxy_seed.biome(state.biome - 1))
    } else {
        None
    };
    let next_seed = if state.biome + 1 < biome_count {
        Some(galaxy_seed.biome(state.biome + 1))
    } else {
        None
    };

    let cur: BiomeComposition = generate_composition(cur_seed);
    let prev = prev_seed
        .map(generate_composition)
        .unwrap_or_else(|| cur.clone());
    let next = next_seed
        .map(generate_composition)
        .unwrap_or_else(|| cur.clone());

    let progress = if LEVELS_PER_BIOME <= 1 {
        0.0
    } else {
        (state.level as f32 / (LEVELS_PER_BIOME - 1) as f32).clamp(0.0, 1.0)
    };
    let var_seed = cur.variation_seed(cur_seed);
    let context = LevelVisualContext {
        current: cur,
        prev,
        next,
        progress,
        is_boss: generated.is_boss,
        var_seed,
    };
    let recipe = recipe_for_context(&context);
    let materials = bake_all(&mut images, &mut color_materials, &recipe);

    commands.insert_resource(ActiveLevelVisuals {
        recipe,
        materials,
        context,
    });

    commands.insert_resource(PendingLevel {
        level: generated.definition,
    });
}

/// Assemble the play stage (bricks only) once the level is pending.
pub fn loading_ready(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pending: ResMut<PendingLevel>,
    visuals: Res<ActiveLevelVisuals>,
    existing_bricks: Query<Entity, With<Brick>>,
    existing_level: Query<Entity, With<LevelEntity>>,
) {
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
    commands.remove_resource::<PendingLevel>();

    spawn_bricks(&mut commands, &mut meshes, &level, &visuals.materials);
}

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
