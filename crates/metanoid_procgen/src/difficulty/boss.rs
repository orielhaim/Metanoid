use rand::prelude::*;

use crate::biome::parameters::BiomeParams;
use crate::level::composer::compose_level_sized;
use crate::level::data::LevelDefinition;

pub fn generate_boss_level(
    biome_params: &BiomeParams,
    rng: &mut impl Rng,
) -> LevelDefinition {
    let boss_params = extremify(biome_params);
    let cols = 14;
    let rows = 10;
    compose_level_sized(cols, rows, &boss_params, rng)
}

fn extremify(params: &BiomeParams) -> BiomeParams {
    BiomeParams {
        temperature: push_to_extreme(params.temperature),
        density: (params.density * 1.3).min(1.0),
        chaos: (params.chaos * 1.5).min(1.0),
        energy: (params.energy * 1.2).min(1.0),
        weirdness: (params.weirdness * 1.4).min(1.0),
    }
}

fn push_to_extreme(val: f32) -> f32 {
    if val < 0.5 {
        (val * 0.5).max(0.0)
    } else {
        (1.0 - (1.0 - val) * 0.5).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::hierarchy::MasterSeed;
    use crate::biome::generator::BiomeGenerator;

    #[test]
    fn boss_produces_bricks() {
        let master = MasterSeed(42);
        let biome_seed = master.galaxy(0).biome(0);
        let params = BiomeGenerator::generate(biome_seed);
        let mut rng = biome_seed.rng();
        let level = generate_boss_level(&params, &mut rng);
        assert!(!level.bricks.is_empty());
        assert!(level.destructible_count() > 0);
    }

    #[test]
    fn boss_is_larger() {
        let master = MasterSeed(42);
        let biome_seed = master.galaxy(0).biome(0);
        let params = BiomeGenerator::generate(biome_seed);
        let mut rng = biome_seed.rng();
        let level = generate_boss_level(&params, &mut rng);
        assert_eq!(level.rows, 10);
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
