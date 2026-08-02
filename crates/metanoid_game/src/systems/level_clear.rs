//! Robust level-clear detection.
//!
//! Bugs fixed:
//! 1. Empty brick query early-return prevented clear when the *last* brick was
//!    destroyed and no invincibles remained.
//! 2. Deferred despawn left zero-health (or still-living) bricks countable for
//!    a frame — we ignore health==0 as already dead.
//! 3. Clear is only armed after bricks finish spawning (not on empty Loading).

use bevy::prelude::*;
use metanoid_core::components::brick::{
    Brick, BrickType, count_blocking_bricks, should_clear_level,
};
use metanoid_core::events::{BrickDestroyedEvent, LevelClearEvent};
use metanoid_core::rating::LevelRunStats;
use metanoid_core::resources::game_state::GameState;
use metanoid_core::save_data::SaveData;
use metanoid_core::states::AppState;
use metanoid_procgen::seed::hierarchy::MasterSeed;
use metanoid_procgen::universe::galaxy::GalaxyDefinition;
use metanoid_procgen::universe::progression::LEVELS_PER_BIOME;

use super::run_stats::{apply_clear_to_save, finalize_rating_for_clear};

/// Tracks whether this play session is eligible for level-clear checks.
#[derive(Resource, Debug, Clone)]
pub struct LevelClearTracker {
    /// Bricks have been spawned and clear checks may run.
    pub armed: bool,
    /// Blocking bricks present when the level armed (for diagnostics).
    pub initial_blocking: u32,
    /// Frames since arming (grace so spawn systems settle).
    pub frames_since_arm: u32,
    /// Optional soft countdown of destroys (diagnostics / future UI).
    pub destroys_this_level: u32,
}

impl Default for LevelClearTracker {
    fn default() -> Self {
        Self {
            armed: false,
            initial_blocking: 0,
            frames_since_arm: 0,
            destroys_this_level: 0,
        }
    }
}

impl LevelClearTracker {
    pub fn disarm(&mut self) {
        *self = Self::default();
    }
}

/// Call on enter Playing after bricks exist.
pub fn arm_level_clear_tracker(mut tracker: ResMut<LevelClearTracker>, bricks: Query<&Brick>) {
    let blocking = count_blocking_bricks(bricks.iter()) as u32;
    *tracker = LevelClearTracker {
        armed: true,
        initial_blocking: blocking,
        frames_since_arm: 0,
        destroys_this_level: 0,
    };
    info!("Level clear armed: {blocking} blocking brick(s) (invincible ignored)");
}

pub fn disarm_level_clear_tracker(mut tracker: ResMut<LevelClearTracker>) {
    tracker.disarm();
}

/// Count destroy events for diagnostics (and belt-and-suspenders clear).
pub fn on_brick_destroyed_track_clear(
    _trigger: On<BrickDestroyedEvent>,
    mut tracker: ResMut<LevelClearTracker>,
) {
    if tracker.armed {
        tracker.destroys_this_level = tracker.destroys_this_level.saturating_add(1);
    }
}

/// Primary clear check — runs after combat systems each frame.
pub fn check_level_clear(
    mut tracker: ResMut<LevelClearTracker>,
    bricks: Query<&Brick>,
    mut game_state: Option<ResMut<GameState>>,
    mut next_state: ResMut<NextState<AppState>>,
    stats: Res<LevelRunStats>,
    mut save: ResMut<SaveData>,
    mut commands: Commands,
    combo: Res<metanoid_core::resources::combo::ComboCounter>,
    remaining_bricks: Query<Entity, With<Brick>>,
) {
    let Some(ref mut state) = game_state else {
        return;
    };

    if state.level_clearing {
        return;
    }

    if !tracker.armed {
        return;
    }

    tracker.frames_since_arm = tracker.frames_since_arm.saturating_add(1);
    // One-frame grace so spawn + arm settle; also avoids clearing mid-loading edge cases.
    if tracker.frames_since_arm < 2 {
        return;
    }

    // If generation produced zero clearable bricks, do not soft-lock forever —
    // treat as clear so the player can progress (and log loudly).
    if tracker.initial_blocking == 0 {
        warn!("Level armed with 0 blocking bricks — auto-clearing to avoid soft-lock");
        finalize_and_transition(
            state,
            &mut tracker,
            &stats,
            &combo,
            &mut save,
            &mut commands,
            &mut next_state,
            &remaining_bricks,
        );
        return;
    }

    let blocking = count_blocking_bricks(bricks.iter());

    if !should_clear_level(tracker.armed, blocking) {
        return;
    }

    info!(
        "Level clear detected (blocking=0, destroys={}, initial={})",
        tracker.destroys_this_level, tracker.initial_blocking
    );

    finalize_and_transition(
        state,
        &mut tracker,
        &stats,
        &combo,
        &mut save,
        &mut commands,
        &mut next_state,
        &remaining_bricks,
    );
}

fn finalize_and_transition(
    state: &mut GameState,
    tracker: &mut LevelClearTracker,
    stats: &LevelRunStats,
    combo: &metanoid_core::resources::combo::ComboCounter,
    save: &mut SaveData,
    commands: &mut Commands,
    next_state: &mut NextState<AppState>,
    remaining_bricks: &Query<Entity, With<Brick>>,
) {
    state.level_clearing = true;
    tracker.armed = false;

    // Level-clear SFX (metanoid_audio observer on LevelClearEvent)
    commands.trigger(LevelClearEvent);

    // Snapshot cleared address BEFORE advancing progression
    let cleared_galaxy = state.galaxy;
    let cleared_biome = state.biome;
    let cleared_level = state.level;
    let is_boss = state.is_boss(LEVELS_PER_BIOME);

    let mut stats_snap = stats.clone();
    stats_snap.lives_remaining = state.lives;
    stats_snap.max_combo = stats_snap.max_combo.max(combo.count);
    stats_snap.run_score_delta = state.score.saturating_sub(stats_snap.score_at_level_start);

    let mut last = finalize_rating_for_clear(
        &stats_snap,
        cleared_galaxy,
        cleared_biome,
        cleared_level,
        state.score,
        is_boss,
    );

    let clear_bonus = 100u64 + (last.rating as u64) * 2;
    let boss_bonus = if is_boss {
        500u64 + cleared_galaxy * 50
    } else {
        0
    };
    state.score += clear_bonus + boss_bonus;
    last.level_score = stats_snap
        .run_score_delta
        .saturating_add(clear_bonus + boss_bonus);
    stats_snap.run_score_delta = last.level_score;

    let rating_val = last.rating;
    apply_clear_to_save(save, &mut last, &stats_snap, state.score);
    commands.insert_resource(last);

    // Despawn leftover bricks (e.g. invincible) so they don't linger into complete UI.
    // try_despawn: some may already be queued for destroy this frame.
    for entity in remaining_bricks.iter() {
        commands.entity(entity).try_despawn();
    }

    info!(
        "Level cleared! G{} B{} L{} - Rating {} - Score {}",
        cleared_galaxy, cleared_biome, cleared_level, rating_val, state.score
    );

    // Advance campaign pointer for NEXT level
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
            info!(
                "New galaxy {} with {} biomes!",
                state.galaxy, galaxy_def.biome_count
            );
        }
        info!("New biome {}!", state.biome);
    }

    next_state.set(AppState::LevelComplete);
}

/// Shared destroy helper: mark dead (health=0) then despawn so clear checks
/// treat the brick as gone even before deferred despawn applies.
///
/// Idempotent: a second call the same frame only try_despawns (no double events / warnings).
pub fn destroy_brick(commands: &mut Commands, entity: Entity, brick: &mut Brick, position: Vec2) {
    let was_alive = brick.health > 0;
    let brick_type = brick.brick_type;
    brick.health = 0;
    if was_alive {
        commands.trigger(metanoid_core::events::BrickDestroyedEvent {
            brick: entity,
            position,
            brick_type,
        });
    }
    // Silenced despawn — explosions / lasers / clear may queue the same entity twice.
    commands.entity(entity).try_despawn();
}

/// Convert leftover invincible-only soft-locks: if the only remaining bricks
/// are invincible and fireball-less play can't progress, zap already handles
/// conversion; clear still succeeds when blocking==0.
#[allow(dead_code)]
fn _invincible_only(bricks: &Query<&Brick>) -> bool {
    let mut any = false;
    for b in bricks.iter() {
        any = true;
        if b.blocks_level_clear() {
            return false;
        }
        if b.brick_type != BrickType::Invincible {
            return false;
        }
    }
    any
}
