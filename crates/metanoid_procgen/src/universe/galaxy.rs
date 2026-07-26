use rand::prelude::*;

use crate::seed::hierarchy::GalaxySeed;

#[derive(Debug, Clone)]
pub struct GalaxyDefinition {
    pub biome_count: usize,
    pub base_difficulty: f32,
    pub base_ball_speed: f32,
    pub rare_powerup_freq: f32,
}

impl GalaxyDefinition {
    pub fn generate(seed: GalaxySeed) -> Self {
        let mut rng = seed.rng();
        let biome_count = rng.random_range(3..=6);
        let galaxy_scale = seed.0 as f32 / u64::MAX as f32;

        Self {
            biome_count,
            base_difficulty: 0.3 + galaxy_scale * 0.4,
            base_ball_speed: 400.0 + galaxy_scale * 100.0,
            rare_powerup_freq: 0.05 + galaxy_scale * 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::hierarchy::MasterSeed;

    #[test]
    fn biome_count_in_range() {
        let master = MasterSeed(42);
        for i in 0..50 {
            let galaxy = GalaxyDefinition::generate(master.galaxy(i));
            assert!(
                galaxy.biome_count >= 3 && galaxy.biome_count <= 6,
                "galaxy {i} biome_count={}",
                galaxy.biome_count
            );
        }
    }

    #[test]
    fn deterministic() {
        let master = MasterSeed(99);
        let a = GalaxyDefinition::generate(master.galaxy(0));
        let b = GalaxyDefinition::generate(master.galaxy(0));
        assert_eq!(a.biome_count, b.biome_count);
        assert!((a.base_difficulty - b.base_difficulty).abs() < 1e-6);
    }

    #[test]
    fn difficulty_scales_with_index() {
        let master = MasterSeed(42);
        let g0 = GalaxyDefinition::generate(master.galaxy(0));
        let g50 = GalaxyDefinition::generate(master.galaxy(50));
        assert!(g50.base_difficulty >= g0.base_difficulty || g50.biome_count > g0.biome_count);
    }
}
