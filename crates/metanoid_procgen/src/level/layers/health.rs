use rand::prelude::*;

use crate::biome::parameters::BiomeParams;
use crate::level::data::BrickData;

pub fn distribute_health(
    bricks: &mut [BrickData],
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) {
    for brick in bricks.iter_mut() {
        if brick.max_health > 1 {
            continue;
        }

        let row_factor = if rows > 1 {
            (rows - 1 - brick.row) as f32 / (rows - 1) as f32
        } else {
            0.5
        };

        let base_hp = 1.0 + row_factor * (1.0 + params.density * 3.0);
        let noise = rng.random::<f32>() * 1.5;
        let hp = (base_hp + noise).round().max(1.0).min(5.0) as u32;

        brick.health = hp;
        brick.max_health = hp;
    }
}
