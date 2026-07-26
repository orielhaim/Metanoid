use rand::prelude::*;

use super::parameters::BiomeParams;
use crate::seed::hierarchy::BiomeSeed;

#[derive(Debug, Clone)]
pub struct BiomeAttractor {
    pub name: &'static str,
    pub center: BiomeParams,
    pub radius: f32,
}

impl BiomeAttractor {
    pub const fn new(
        name: &'static str,
        center: BiomeParams,
        radius: f32,
    ) -> Self {
        Self { name, center, radius }
    }
}

pub const ATTRACTORS: &[BiomeAttractor] = &[
    BiomeAttractor::new(
        "Neon City",
        BiomeParams {
            temperature: 0.7,
            density: 0.7,
            chaos: 0.3,
            energy: 0.9,
            weirdness: 0.2,
        },
        0.2,
    ),
    BiomeAttractor::new(
        "Deep Ocean",
        BiomeParams {
            temperature: 0.2,
            density: 0.6,
            chaos: 0.2,
            energy: 0.3,
            weirdness: 0.3,
        },
        0.2,
    ),
    BiomeAttractor::new(
        "Volcanic",
        BiomeParams {
            temperature: 1.0,
            density: 0.5,
            chaos: 0.7,
            energy: 0.8,
            weirdness: 0.3,
        },
        0.2,
    ),
    BiomeAttractor::new(
        "Crystal Cavern",
        BiomeParams {
            temperature: 0.1,
            density: 0.8,
            chaos: 0.1,
            energy: 0.5,
            weirdness: 0.4,
        },
        0.2,
    ),
    BiomeAttractor::new(
        "Cosmic Void",
        BiomeParams {
            temperature: 0.5,
            density: 0.3,
            chaos: 0.6,
            energy: 0.7,
            weirdness: 0.9,
        },
        0.25,
    ),
    BiomeAttractor::new(
        "Forest",
        BiomeParams {
            temperature: 0.5,
            density: 0.8,
            chaos: 0.4,
            energy: 0.4,
            weirdness: 0.1,
        },
        0.2,
    ),
    BiomeAttractor::new(
        "Desert",
        BiomeParams {
            temperature: 0.9,
            density: 0.2,
            chaos: 0.2,
            energy: 0.7,
            weirdness: 0.1,
        },
        0.2,
    ),
    BiomeAttractor::new(
        "Arctic",
        BiomeParams {
            temperature: 0.0,
            density: 0.5,
            chaos: 0.1,
            energy: 0.2,
            weirdness: 0.2,
        },
        0.2,
    ),
];

pub struct BiomeGenerator;

impl BiomeGenerator {
    pub fn generate(seed: BiomeSeed) -> BiomeParams {
        let mut rng = seed.rng();

        let raw = BiomeParams::sample(&mut rng);

        let attractor_idx = rng.random_range(0..ATTRACTORS.len());
        let attractor = &ATTRACTORS[attractor_idx];

        let blend: f32 = rng.random_range(0.4..0.8);
        let result = raw.lerp(attractor.center, blend);

        result.clamp01()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::hierarchy::MasterSeed;

    #[test]
    fn generate_in_range() {
        let master = MasterSeed(42);
        for i in 0..100 {
            let biome = BiomeGenerator::generate(master.galaxy(0).biome(i));
            assert!((0.0..=1.0).contains(&biome.temperature), "temp {i}");
            assert!((0.0..=1.0).contains(&biome.density), "density {i}");
            assert!((0.0..=1.0).contains(&biome.chaos), "chaos {i}");
            assert!((0.0..=1.0).contains(&biome.energy), "energy {i}");
            assert!((0.0..=1.0).contains(&biome.weirdness), "weird {i}");
        }
    }

    #[test]
    fn generate_deterministic() {
        let master = MasterSeed(99);
        let seed = master.galaxy(2).biome(1);
        let a = BiomeGenerator::generate(seed);
        let b = BiomeGenerator::generate(seed);
        assert_eq!(a, b);
    }

    #[test]
    fn generate_varied() {
        let master = MasterSeed(100);
        let params: Vec<BiomeParams> = (0..20)
            .map(|i| BiomeGenerator::generate(master.galaxy(0).biome(i)))
            .collect();
        let all_same = params.windows(2).all(|w| w[0] == w[1]);
        assert!(!all_same, "biomes should vary");
    }
}
