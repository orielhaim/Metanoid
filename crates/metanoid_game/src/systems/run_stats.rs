//! Collect LevelRunStats during play and finalize rating on clear.

use bevy::prelude::*;
use metanoid_core::components::brick::Brick;
use metanoid_core::rating::{LastRatingResult, LevelRunStats, RatingTargets, compute_rating};
use metanoid_core::resources::combo::ComboCounter;
use metanoid_core::resources::game_state::GameState;
use metanoid_core::save_data::{SaveData, apply_level_clear};
use metanoid_procgen::universe::progression::LEVELS_PER_BIOME;

use super::save::{now_unix, save_game};

pub fn tick_run_stats(
    time: Res<Time>,
    mut stats: ResMut<LevelRunStats>,
    game_state: Option<Res<GameState>>,
    combo: Res<ComboCounter>,
) {
    stats.elapsed_secs += time.delta_secs();
    stats.max_combo = stats.max_combo.max(combo.count);
    if let Some(state) = game_state {
        stats.lives_remaining = state.lives;
        stats.run_score_delta = state.score.saturating_sub(stats.score_at_level_start);
    }
}

pub fn begin_run_stats_on_play(
    mut stats: ResMut<LevelRunStats>,
    game_state: Option<Res<GameState>>,
    bricks: Query<&Brick>,
    combo: ResMut<ComboCounter>,
) {
    let breakable = bricks.iter().filter(|b| b.blocks_level_clear()).count() as u32;

    let (lives, score, is_boss, difficulty) = game_state
        .as_ref()
        .map(|s| {
            (
                s.lives,
                s.score,
                s.is_boss(LEVELS_PER_BIOME),
                0.3 + (s.galaxy as f32 * 0.02).min(0.5),
            )
        })
        .unwrap_or((3, 0, false, 0.3));

    stats.begin_level(breakable.max(1), lives, score, is_boss, difficulty);

    // Reset combo for the level
    let mut combo = combo;
    *combo = Default::default();
}

pub fn on_life_lost_track_stats(
    _trigger: On<metanoid_core::events::LifeLostEvent>,
    mut stats: ResMut<LevelRunStats>,
) {
    stats.deaths = stats.deaths.saturating_add(1);
}

/// Finalize rating for the level that was just cleared (before progression advance).
pub fn finalize_rating_for_clear(
    stats: &LevelRunStats,
    galaxy: u64,
    biome: u64,
    level: u64,
    _run_score: u64,
    is_boss: bool,
) -> LastRatingResult {
    let targets = RatingTargets::from_level(
        stats.breakable_bricks,
        stats.biome_difficulty,
        stats.starting_lives,
    );
    let breakdown = compute_rating(stats, &targets);
    let level_score = stats.run_score_delta;

    LastRatingResult {
        galaxy,
        biome,
        level,
        rating: breakdown.total,
        grade: breakdown.grade,
        breakdown: Some(breakdown),
        level_score,
        is_pb: false,
        is_boss,
    }
}

pub fn apply_clear_to_save(
    save: &mut SaveData,
    last: &mut LastRatingResult,
    stats: &LevelRunStats,
    run_score: u64,
) {
    let is_pb = apply_level_clear(
        save,
        last.galaxy,
        last.biome,
        last.level,
        last.rating,
        last.level_score,
        stats.max_combo,
        stats.elapsed_secs,
        run_score,
        stats.bricks_destroyed as u64,
        now_unix(),
    );
    last.is_pb = is_pb;
    save_game(save);
}
