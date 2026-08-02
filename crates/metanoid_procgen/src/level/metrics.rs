//! Brick count / size scaling: easy = few large bricks, hard = many small bricks.

use crate::difficulty::curve::DifficultyParams;
use crate::level::data::BrickMetrics;
use crate::universe::progression::LEVELS_PER_BIOME;

// Match arena used by the game (metanoid_core constants).
const ARENA_WIDTH: f32 = 1280.0;
const WALL_THICKNESS: f32 = 20.0;
const PLAY_MARGIN_X: f32 = 48.0;
const PLAY_TOP: f32 = 48.0;
const PLAY_BOTTOM_RESERVED: f32 = 220.0; // paddle + free air

/// Compute grid + brick pixel size from biome progress and difficulty.
///
/// `progress` is 0..1 within the biome (boss uses elevated progress).
pub fn brick_metrics_for(progress: f32, diff: &DifficultyParams, is_boss: bool) -> BrickMetrics {
    let mut p = progress.clamp(0.0, 1.0);
    if is_boss {
        p = (p + 0.35).min(1.15);
    }
    // Density from difficulty also pushes toward finer grids
    let density_push = ((diff.brick_count_mult - 0.7) / 0.8).clamp(0.0, 1.0) * 0.25;
    p = (p + density_push).min(1.2);

    // Cols: 7 (chunky tutorial) -> 18 (bullet hell)
    let cols = (7.0 + p * 11.0).round().clamp(7.0, 18.0) as usize;
    // Rows: 4 -> 12
    let rows = (4.0 + p * 8.0).round().clamp(4.0, 12.0) as usize;

    let gap = (5.5 - p * 2.8).clamp(2.0, 5.5);

    let play_w = ARENA_WIDTH - WALL_THICKNESS * 2.0 - PLAY_MARGIN_X * 2.0;
    let play_h = (720.0 - WALL_THICKNESS - PLAY_TOP - PLAY_BOTTOM_RESERVED).clamp(200.0, 420.0);

    let brick_w = ((play_w - gap * (cols as f32 - 1.0)) / cols as f32).clamp(36.0, 140.0);
    let brick_h = ((play_h - gap * (rows as f32 - 1.0)) / rows as f32).clamp(14.0, 48.0);

    BrickMetrics {
        cols,
        rows,
        brick_w,
        brick_h,
        gap,
    }
}

pub fn level_progress(level: u64) -> f32 {
    if LEVELS_PER_BIOME <= 1 {
        1.0
    } else {
        (level as f32 / (LEVELS_PER_BIOME - 1) as f32).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mid_diff() -> DifficultyParams {
        DifficultyParams::default()
    }

    #[test]
    fn easy_has_larger_bricks_than_hard() {
        let easy = brick_metrics_for(0.0, &mid_diff(), false);
        let hard = brick_metrics_for(1.0, &mid_diff(), false);
        assert!(
            easy.brick_w > hard.brick_w,
            "easy w {} hard w {}",
            easy.brick_w,
            hard.brick_w
        );
        assert!(easy.cols < hard.cols);
        assert!(easy.rows <= hard.rows);
    }

    #[test]
    fn boss_grid_is_fine() {
        let normal = brick_metrics_for(0.9, &mid_diff(), false);
        let boss = brick_metrics_for(0.9, &mid_diff(), true);
        assert!(boss.cols >= normal.cols || boss.rows >= normal.rows);
        assert!(boss.brick_w <= normal.brick_w + 1.0);
    }
}
