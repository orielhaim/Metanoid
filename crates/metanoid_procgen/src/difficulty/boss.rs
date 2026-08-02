//! Boss levels: denser, tougher, more specials — always harder than pre-boss stages.

use rand::prelude::*;

use crate::biome::parameters::BiomeParams;
use crate::difficulty::curve::DifficultyParams;
use crate::level::composer::compose_level_sized;
use crate::level::data::{BrickKind, BrickMetrics, LevelDefinition, SpecialType};

/// Build a boss encounter that is strictly tougher than a normal late-biome level.
pub fn generate_boss_level(
    biome_params: &BiomeParams,
    base_diff: &DifficultyParams,
    metrics: BrickMetrics,
    rng: &mut impl Rng,
) -> LevelDefinition {
    let boss_params = extremify(biome_params);
    // Force dense, chaotic layouts
    let mut dense = boss_params;
    dense.density = dense.density.max(0.88);
    dense.chaos = dense.chaos.max(0.75);
    dense.energy = dense.energy.max(0.7);

    let mut level = compose_level_sized(metrics.cols, metrics.rows, &dense, rng);
    level.metrics = metrics;

    harden_boss_bricks(&mut level, base_diff, rng);
    stamp_boss_signature(&mut level, rng);

    level
}

/// Difficulty multipliers for boss fights (applied to ActiveLevelDifficulty).
pub fn boss_difficulty_override(base: &DifficultyParams) -> DifficultyParams {
    DifficultyParams {
        ball_speed_mult: (base.ball_speed_mult * 1.4).max(1.35),
        avg_brick_health: base.avg_brick_health + 2.2,
        brick_count_mult: base.brick_count_mult * 1.25,
        powerup_freq_mult: (base.powerup_freq_mult * 0.75).max(0.35),
        negative_powerup_ratio: (base.negative_powerup_ratio + 0.15).min(0.55),
        special_density_mult: base.special_density_mult * 1.6,
        moving_brick_count: base.moving_brick_count + 5,
    }
}

fn extremify(params: &BiomeParams) -> BiomeParams {
    BiomeParams {
        temperature: push_to_extreme(params.temperature),
        density: (params.density * 1.45).min(1.0),
        chaos: (params.chaos * 1.6).min(1.0),
        energy: (params.energy * 1.35).min(1.0),
        weirdness: (params.weirdness * 1.5).min(1.0),
    }
}

fn push_to_extreme(val: f32) -> f32 {
    if val < 0.5 {
        (val * 0.4).max(0.0)
    } else {
        (1.0 - (1.0 - val) * 0.35).min(1.0)
    }
}

fn harden_boss_bricks(level: &mut LevelDefinition, diff: &DifficultyParams, rng: &mut impl Rng) {
    for brick in level.bricks.iter_mut() {
        if !brick.is_destructible() {
            continue;
        }
        // Heavy multi-hit bias
        let roll = rng.random::<f32>();
        if roll < 0.45 {
            brick.kind = BrickKind::MultiHit;
            brick.health = (3 + (diff.avg_brick_health * 0.5) as u32).min(6);
            brick.max_health = brick.health;
        } else if roll < 0.58 {
            brick.kind = BrickKind::Explosive;
            brick.health = 1;
            brick.max_health = 1;
        } else if brick.max_health <= 1 && rng.random::<f32>() < 0.55 {
            brick.health = 2 + rng.random_range(0..2);
            brick.max_health = brick.health;
            brick.kind = BrickKind::MultiHit;
        } else if brick.max_health > 1 {
            brick.health = (brick.health + 2).min(6);
            brick.max_health = brick.health;
        }

        // Boss regenerators
        if brick.special == SpecialType::None && rng.random::<f32>() < 0.12 {
            brick.special = SpecialType::Regenerating;
            brick.health = brick.health.max(2);
            brick.max_health = brick.health;
        }
    }

    // Ensure a solid mover escort on open cells
    let mut open: Vec<usize> = level
        .bricks
        .iter()
        .enumerate()
        .filter(|(_, b)| {
            b.special == SpecialType::None
                && b.is_destructible()
                && has_open_side(&level.bricks, b.col, b.row)
        })
        .map(|(i, _)| i)
        .collect();
    open.shuffle(rng);
    for i in open.into_iter().take(diff.moving_brick_count.max(4)) {
        level.bricks[i].special = SpecialType::Moving;
    }
}

/// Carve a dense “fortress” ring of invincible + multi-hit core for drama.
fn stamp_boss_signature(level: &mut LevelDefinition, rng: &mut impl Rng) {
    let cols = level.cols;
    let rows = level.rows;
    if cols < 6 || rows < 5 {
        return;
    }

    let cx = cols / 2;
    let cy = rows / 2;
    let variant = rng.random_range(0..3);

    match variant {
        0 => {
            // Hollow square core
            for b in level.bricks.iter_mut() {
                let on_ring = (b.col == cx - 2 || b.col == cx + 2)
                    && b.row >= cy.saturating_sub(2)
                    && b.row <= cy + 2
                    || (b.row == cy.saturating_sub(2) || b.row == cy + 2)
                        && b.col >= cx.saturating_sub(2)
                        && b.col <= cx + 2;
                if on_ring && b.is_destructible() && rng.random::<f32>() < 0.55 {
                    b.kind = BrickKind::MultiHit;
                    b.health = b.health.max(4);
                    b.max_health = b.health;
                }
            }
        }
        1 => {
            // Cross of high HP
            for b in level.bricks.iter_mut() {
                if (b.col == cx || b.row == cy) && b.is_destructible() {
                    b.kind = BrickKind::MultiHit;
                    b.health = b.health.max(3);
                    b.max_health = b.health;
                }
            }
        }
        _ => {
            // Diamond shell
            for b in level.bricks.iter_mut() {
                let d = b.col.abs_diff(cx) + b.row.abs_diff(cy);
                if d == 3 && b.is_destructible() {
                    b.kind = BrickKind::MultiHit;
                    b.health = b.health.max(3);
                    b.max_health = b.health;
                    if rng.random::<f32>() < 0.25 {
                        b.special = SpecialType::Regenerating;
                    }
                }
            }
        }
    }

    // Cap invincible so the level remains clearable
    let inv = level.bricks.iter().filter(|b| !b.is_destructible()).count();
    let max_inv = (level.bricks.len() / 5).max(2);
    if inv > max_inv {
        let mut converted = 0;
        let excess = inv - max_inv;
        for b in level.bricks.iter_mut() {
            if converted >= excess {
                break;
            }
            if !b.is_destructible() {
                b.kind = BrickKind::MultiHit;
                b.health = 4;
                b.max_health = 4;
                converted += 1;
            }
        }
    }
}

fn has_open_side(bricks: &[crate::level::data::BrickData], col: usize, row: usize) -> bool {
    let left = !bricks.iter().any(|b| b.row == row && b.col + 1 == col);
    let right = !bricks.iter().any(|b| b.row == row && b.col == col + 1);
    left || right
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biome::generator::BiomeGenerator;
    use crate::difficulty::curve::compute_difficulty;
    use crate::seed::hierarchy::MasterSeed;
    use crate::universe::progression::LEVELS_PER_BIOME;

    #[test]
    fn boss_produces_bricks() {
        let master = MasterSeed(42);
        let biome_seed = master.galaxy(0).biome(0);
        let params = BiomeGenerator::generate(biome_seed);
        let diff = compute_difficulty(&params, 11, LEVELS_PER_BIOME, 0.4);
        let mut rng = biome_seed.rng();
        let metrics = BrickMetrics {
            cols: 16,
            rows: 11,
            brick_w: 55.0,
            brick_h: 22.0,
            gap: 3.0,
        };
        let level = generate_boss_level(&params, &diff, metrics, &mut rng);
        assert!(level.destructible_count() > 20);
    }

    #[test]
    fn boss_override_harder_than_base() {
        let base = DifficultyParams {
            ball_speed_mult: 1.1,
            avg_brick_health: 2.0,
            brick_count_mult: 1.0,
            powerup_freq_mult: 0.8,
            negative_powerup_ratio: 0.2,
            special_density_mult: 1.2,
            moving_brick_count: 2,
        };
        let boss = boss_difficulty_override(&base);
        assert!(boss.ball_speed_mult > base.ball_speed_mult);
        assert!(boss.avg_brick_health > base.avg_brick_health);
        assert!(boss.moving_brick_count > base.moving_brick_count);
    }

    #[test]
    fn extremify_pushes_values() {
        let params = BiomeParams {
            temperature: 0.7,
            density: 0.5,
            chaos: 0.4,
            energy: 0.6,
            weirdness: 0.3,
        };
        let boss = extremify(&params);
        assert!(boss.density > params.density);
        assert!(boss.chaos > params.chaos);
    }
}
