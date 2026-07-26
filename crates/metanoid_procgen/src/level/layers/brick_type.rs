use rand::prelude::*;

use crate::biome::parameters::BiomeParams;
use crate::level::data::{BrickData, BrickKind};

pub fn assign_brick_types(
    bricks: &mut [BrickData],
    _cols: usize,
    _rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) {
    let noise_seed = rng.random::<u32>();

    for brick in bricks.iter_mut() {
        let cluster_val = simple_noise_2d(
            noise_seed,
            brick.col as f32 * 0.3,
            brick.row as f32 * 0.3,
        );

        let roll = rng.random::<f32>();

        if cluster_val > 0.8 && params.temperature > 0.5 && roll < 0.15 {
            brick.kind = BrickKind::Explosive;
        } else if cluster_val > 0.7 && roll < 0.05 + params.density * 0.05 {
            brick.kind = BrickKind::Invincible;
        } else if cluster_val > 0.5 && roll < 0.1 + params.density * 0.15 {
            brick.kind = BrickKind::MultiHit;
            brick.health = 2 + (params.density * 2.0) as u32;
            brick.max_health = brick.health;
        } else {
            brick.kind = BrickKind::Normal;
        }
    }

    let total = bricks.len().max(1) as f32;
    let invincible_count = bricks.iter().filter(|b| b.kind == BrickKind::Invincible).count();
    let invincible_ratio = invincible_count as f32 / total;
    if invincible_ratio > 0.2 {
        let excess = ((invincible_ratio - 0.2) * total) as usize;
        let mut converted = 0;
        for brick in bricks.iter_mut() {
            if converted >= excess {
                break;
            }
            if brick.kind == BrickKind::Invincible {
                brick.kind = BrickKind::Normal;
                brick.health = 1;
                brick.max_health = 1;
                converted += 1;
            }
        }
    }
}

fn simple_noise_2d(seed: u32, x: f32, y: f32) -> f32 {
    let ix = (x * 100.0) as i32;
    let iy = (y * 100.0) as i32;
    let hash = hash_pair(
        hash_pair(seed as u64, ix as u64),
        iy as u64,
    );
    (hash & 0xFFFF) as f32 / 0xFFFF as f32
}

fn hash_pair(a: u64, b: u64) -> u64 {
    let h = a.wrapping_mul(0x517cc1b727220a95).wrapping_add(b);
    h ^ (h >> 31)
}
