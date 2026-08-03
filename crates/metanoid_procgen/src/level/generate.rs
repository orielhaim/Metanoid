//! Single entry point: unique, difficulty-scaled, creative levels.

use rand::prelude::*;

use crate::biome::generator::BiomeGenerator;
use crate::biome::parameters::BiomeParams;
use crate::difficulty::boss::{boss_difficulty_override, generate_boss_level};
use crate::difficulty::curve::{DifficultyParams, compute_difficulty};
use crate::level::composer::compose_level_sized;
use crate::level::data::{BrickData, LevelDefinition, SpecialType};
use crate::level::metrics::{brick_metrics_for, level_progress};
use crate::level::motion::{BrickMotion, generate_motion};
use crate::seed::hierarchy::{LevelSeed, MasterSeed};
use crate::universe::galaxy::GalaxyDefinition;
use crate::universe::progression::LEVELS_PER_BIOME;

/// Full generated level package used by the game.
#[derive(Debug, Clone)]
pub struct GeneratedLevel {
    pub definition: LevelDefinition,
    pub biome_params: BiomeParams,
    pub difficulty: DifficultyParams,
    pub level_seed: LevelSeed,
    pub is_boss: bool,
}

/// Generate a deterministic, unique level for (galaxy, biome, level).
pub fn generate_level_at(master_seed: u64, galaxy: u64, biome: u64, level: u64) -> GeneratedLevel {
    let master = MasterSeed::new(master_seed);
    let galaxy_seed = master.galaxy(galaxy);
    let biome_seed = galaxy_seed.biome(biome);
    let level_seed = biome_seed.level(level);

    let biome_params = BiomeGenerator::generate(biome_seed);
    let galaxy_def = GalaxyDefinition::generate(galaxy_seed);
    let is_boss = level + 1 >= LEVELS_PER_BIOME;

    let mut difficulty = compute_difficulty(
        &biome_params,
        level.min(LEVELS_PER_BIOME.saturating_sub(1)),
        LEVELS_PER_BIOME,
        galaxy_def.base_difficulty,
    );
    if is_boss {
        difficulty = boss_difficulty_override(&difficulty);
    }

    // Galaxy progression: deeper galaxies push difficulty further
    let galaxy_boost = 1.0 + (galaxy as f32 * 0.06).min(0.5);
    difficulty.ball_speed_mult *= galaxy_boost;
    difficulty.avg_brick_health *= 1.0 + (galaxy as f32 * 0.04).min(0.35);

    let progress = level_progress(level);
    let metrics = brick_metrics_for(progress, &difficulty, is_boss);
    let mut rng = level_seed.rng();

    let mut definition = if is_boss {
        generate_boss_level(&biome_params, &difficulty, metrics, &mut rng)
    } else {
        let scaled = scale_params_for_difficulty(&biome_params, &difficulty, progress);
        let mut level_def = compose_level_sized(metrics.cols, metrics.rows, &scaled, &mut rng);
        level_def.metrics = metrics;
        level_def.cols = metrics.cols;
        level_def.rows = metrics.rows;
        level_def
    };

    apply_difficulty_post(&mut definition, &difficulty, is_boss, &mut rng);
    // Movers only where there is open runway
    sanitize_movers(&mut definition);
    // Give every remaining mover a full motion spec (shape + speed profile).
    assign_motions(&mut definition, progress, &biome_params, &mut rng);

    GeneratedLevel {
        definition,
        biome_params,
        difficulty,
        level_seed,
        is_boss,
    }
}

/// Assign a motion spec to every moving brick, sized to the free runway around
/// it. Bricks without enough room are demoted to static.
fn assign_motions(
    level: &mut LevelDefinition,
    progress: f32,
    params: &BiomeParams,
    rng: &mut impl Rng,
) {
    let bricks = level.bricks.clone();
    for brick in level.bricks.iter_mut() {
        if brick.special != SpecialType::Moving {
            continue;
        }
        let (left, right) = free_run_cells(&bricks, brick.col, brick.row);
        if left + right < 1 {
            brick.special = SpecialType::None;
            brick.motion = None;
            continue;
        }
        let (up, down) = free_vertical_cells(&bricks, brick.col, brick.row);
        let motion: BrickMotion = generate_motion(rng, progress, params.chaos, up, down);
        brick.motion = Some(motion);
    }
}

fn scale_params_for_difficulty(
    base: &BiomeParams,
    diff: &DifficultyParams,
    progress: f32,
) -> BiomeParams {
    let t = progress.clamp(0.0, 1.0);
    BiomeParams {
        temperature: base.temperature,
        density: (base.density * 0.4 + diff.brick_count_mult * 0.4 + t * 0.25).clamp(0.2, 1.0),
        chaos: (base.chaos * 0.35 + diff.special_density_mult * 0.25 + t * 0.35).clamp(0.15, 1.0),
        energy: (base.energy * 0.5 + t * 0.5).clamp(0.15, 1.0),
        weirdness: (base.weirdness * 0.55 + t * 0.35 + base.chaos * 0.1).clamp(0.05, 1.0),
    }
}

fn apply_difficulty_post(
    level: &mut LevelDefinition,
    diff: &DifficultyParams,
    is_boss: bool,
    rng: &mut impl Rng,
) {
    let hp_boost_chance = if is_boss {
        0.75
    } else {
        ((diff.avg_brick_health - 1.0) / 4.0).clamp(0.0, 0.7)
    };

    for brick in level.bricks.iter_mut() {
        if !brick.is_destructible() {
            continue;
        }
        if brick.max_health <= 1 && rng.random::<f32>() < hp_boost_chance {
            let extra = 1 + (diff.avg_brick_health * 0.45) as u32 + if is_boss { 1 } else { 0 };
            brick.health = (brick.health + extra).min(if is_boss { 6 } else { 5 });
            brick.max_health = brick.health;
            if brick.health > 1 {
                brick.kind = crate::level::data::BrickKind::MultiHit;
            }
        } else if brick.max_health > 1 {
            let scale = 1.0 + (diff.avg_brick_health - 1.0) * (if is_boss { 0.4 } else { 0.28 });
            brick.health = ((brick.health as f32) * scale)
                .round()
                .clamp(2.0, if is_boss { 6.0 } else { 5.0 }) as u32;
            brick.max_health = brick.health;
        }
    }

    let target_moving = if is_boss {
        diff.moving_brick_count.max(5)
    } else {
        diff.moving_brick_count
            .max(((level.bricks.len() as f32) * 0.025 * diff.special_density_mult).round() as usize)
    };
    let current_moving = level
        .bricks
        .iter()
        .filter(|b| b.special == SpecialType::Moving)
        .count();
    if current_moving < target_moving {
        promote_moving_bricks(level, target_moving - current_moving, rng);
    }
}

fn promote_moving_bricks(level: &mut LevelDefinition, need: usize, rng: &mut impl Rng) {
    let mut candidates: Vec<usize> = level
        .bricks
        .iter()
        .enumerate()
        .filter(|(_, b)| {
            b.special == SpecialType::None
                && b.is_destructible()
                && free_run_cells(&level.bricks, b.col, b.row).0
                    + free_run_cells(&level.bricks, b.col, b.row).1
                    >= 1
        })
        .map(|(i, _)| i)
        .collect();
    candidates.shuffle(rng);
    for idx in candidates.into_iter().take(need) {
        level.bricks[idx].special = SpecialType::Moving;
    }
}

fn sanitize_movers(level: &mut LevelDefinition) {
    let bricks = level.bricks.clone();
    for brick in level.bricks.iter_mut() {
        if brick.special != SpecialType::Moving {
            continue;
        }
        let (left, right) = free_run_cells(&bricks, brick.col, brick.row);
        if left + right == 0 {
            brick.special = SpecialType::None;
        }
    }
}

/// Count consecutive empty cells to the left / right on the same row.
pub fn free_run_cells(bricks: &[BrickData], col: usize, row: usize) -> (usize, usize) {
    let occupied = |c: usize, r: usize| bricks.iter().any(|b| b.col == c && b.row == r);

    let mut left = 0usize;
    let mut c = col;
    while c > 0 {
        c -= 1;
        if occupied(c, row) {
            break;
        }
        left += 1;
    }

    let mut right = 0usize;
    c = col;
    let max_c = bricks.iter().map(|b| b.col).max().unwrap_or(col);
    // Also allow scanning past max brick col if grid is sparse — use a generous cap
    let cap = max_c + 4;
    loop {
        c += 1;
        if c > cap {
            break;
        }
        if occupied(c, row) {
            break;
        }
        right += 1;
        if right > 20 {
            break;
        }
    }
    (left, right)
}

/// Occupied neighbor flags (immediate).
pub fn horizontal_clearance(bricks: &[BrickData], col: usize, row: usize) -> (bool, bool) {
    let (l, r) = free_run_cells(bricks, col, row);
    (l > 0, r > 0)
}

/// Count consecutive empty cells above / below on the same column.
pub fn free_vertical_cells(bricks: &[BrickData], col: usize, row: usize) -> (usize, usize) {
    let occupied = |c: usize, r: usize| bricks.iter().any(|b| b.col == c && b.row == r);

    let mut up = 0usize;
    let mut r = row;
    while r > 0 {
        r -= 1;
        if occupied(col, r) {
            break;
        }
        up += 1;
    }

    let mut down = 0usize;
    r = row;
    let max_r = bricks.iter().map(|b| b.row).max().unwrap_or(row);
    let cap = max_r + 4;
    loop {
        r += 1;
        if r > cap {
            break;
        }
        if occupied(col, r) {
            break;
        }
        down += 1;
        if down > 20 {
            break;
        }
    }
    (up, down)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_in_same_biome_differ() {
        let master = 42u64;
        let a = generate_level_at(master, 0, 0, 0);
        let b = generate_level_at(master, 0, 0, 1);
        let c = generate_level_at(master, 0, 0, 5);
        let sig = |g: &GeneratedLevel| {
            g.definition
                .bricks
                .iter()
                .map(|br| (br.col, br.row, br.kind, br.health, br.special))
                .collect::<Vec<_>>()
        };
        assert_ne!(sig(&a), sig(&b));
        assert_ne!(sig(&a), sig(&c));
    }

    #[test]
    fn same_address_is_deterministic() {
        let a = generate_level_at(99, 1, 2, 3);
        let b = generate_level_at(99, 1, 2, 3);
        assert_eq!(a.definition.bricks.len(), b.definition.bricks.len());
        assert_eq!(a.definition.metrics.cols, b.definition.metrics.cols);
    }

    #[test]
    fn easy_bricks_larger_than_late() {
        let early = generate_level_at(7, 0, 0, 0);
        let late = generate_level_at(7, 0, 0, 10);
        assert!(
            early.definition.metrics.brick_w >= late.definition.metrics.brick_w - 0.5,
            "early {} late {}",
            early.definition.metrics.brick_w,
            late.definition.metrics.brick_w
        );
        assert!(
            early.definition.metrics.cols <= late.definition.metrics.cols
                || early.definition.metrics.rows <= late.definition.metrics.rows
        );
    }

    #[test]
    fn boss_harder_than_pre_boss() {
        let pre = generate_level_at(11, 0, 0, 10);
        let boss = generate_level_at(11, 0, 0, 11);
        assert!(boss.is_boss);
        assert!(
            boss.difficulty.ball_speed_mult > pre.difficulty.ball_speed_mult,
            "boss speed {} pre {}",
            boss.difficulty.ball_speed_mult,
            pre.difficulty.ball_speed_mult
        );
        assert!(boss.difficulty.avg_brick_health > pre.difficulty.avg_brick_health);
        let boss_hp: f32 = boss
            .definition
            .bricks
            .iter()
            .filter(|b| b.is_destructible())
            .map(|b| b.health as f32)
            .sum::<f32>()
            / boss.definition.destructible_count().max(1) as f32;
        let pre_hp: f32 = pre
            .definition
            .bricks
            .iter()
            .filter(|b| b.is_destructible())
            .map(|b| b.health as f32)
            .sum::<f32>()
            / pre.definition.destructible_count().max(1) as f32;
        assert!(
            boss_hp + 0.05 >= pre_hp
                || boss.definition.destructible_count() >= pre.definition.destructible_count(),
            "boss should be tougher: boss_hp={boss_hp} pre_hp={pre_hp}"
        );
    }

    #[test]
    fn free_run_detects_neighbors() {
        let bricks = vec![BrickData::normal(2, 0), BrickData::normal(5, 0)];
        // col 2: free right until col 5 => 2 empty cells (3,4)
        let (l, r) = free_run_cells(&bricks, 2, 0);
        assert_eq!(l, 2); // 0,1 empty if we allow - actually col 1 and 0 empty
        assert_eq!(r, 2); // 3,4
        let (l5, r5) = free_run_cells(&bricks, 5, 0);
        assert_eq!(l5, 2);
        assert!(r5 >= 1);
    }

    #[test]
    fn twelve_levels_all_playable() {
        for level in 0..LEVELS_PER_BIOME {
            let g = generate_level_at(42, 0, 0, level);
            assert!(g.definition.destructible_count() > 0, "level {level} empty");
        }
    }

    #[test]
    fn movers_have_motion() {
        let master = 42u64;
        let mut any_mover = false;
        for level in 0..LEVELS_PER_BIOME {
            let g = generate_level_at(master, 0, 0, level);
            for b in &g.definition.bricks {
                if b.special == SpecialType::Moving {
                    any_mover = true;
                    assert!(
                        b.motion.is_some(),
                        "moving brick missing motion at level {level} c{} r{}",
                        b.col,
                        b.row
                    );
                } else {
                    assert!(
                        b.motion.is_none(),
                        "static brick has motion at level {level}"
                    );
                }
            }
        }
        assert!(any_mover, "expected at least one mover across 12 levels");
    }

    #[test]
    fn deep_levels_have_faster_movers() {
        let early = generate_level_at(7, 0, 0, 2);
        let late = generate_level_at(7, 0, 0, 10);
        let avg_speed = |g: &GeneratedLevel| -> f32 {
            let mut sum = 0.0f32;
            let mut n = 0usize;
            for b in &g.definition.bricks {
                if let Some(m) = b.motion {
                    sum += m.base_speed;
                    n += 1;
                }
            }
            if n == 0 { 0.0 } else { sum / n as f32 }
        };
        assert!(
            avg_speed(&late) > avg_speed(&early),
            "late {} should be faster than early {}",
            avg_speed(&late),
            avg_speed(&early)
        );
    }
}
