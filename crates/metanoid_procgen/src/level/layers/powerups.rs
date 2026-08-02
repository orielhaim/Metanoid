use rand::prelude::*;

use crate::biome::parameters::BiomeParams;
use crate::level::data::BrickData;

const PITY_THRESHOLD: usize = 8;

pub fn seed_powerups(bricks: &mut [BrickData], params: &BiomeParams, rng: &mut impl Rng) {
    let base_chance = 0.08 + params.density * 0.08;
    let mut bricks_since_drop: usize = 0;

    for brick in bricks.iter_mut() {
        if !brick.is_destructible() {
            continue;
        }

        bricks_since_drop += 1;
        let mut chance = base_chance;

        if bricks_since_drop >= PITY_THRESHOLD {
            chance = 1.0;
        }

        if rng.random::<f32>() < chance {
            brick.powerup_chance = 0.5 + rng.random::<f32>() * 0.5;
            bricks_since_drop = 0;
        }
    }
}
