use crate::seed::hasher::derive;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MasterSeed(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GalaxySeed(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BiomeSeed(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LevelSeed(pub u64);

impl MasterSeed {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn galaxy(&self, index: u64) -> GalaxySeed {
        GalaxySeed(derive(self.0, index))
    }
}

impl GalaxySeed {
    pub fn biome(&self, index: u64) -> BiomeSeed {
        BiomeSeed(derive(self.0, index))
    }

    pub fn rng(&self) -> ChaCha8Rng {
        let mut seed_bytes = [0u8; 32];
        let le = self.0.to_le_bytes();
        seed_bytes[..8].copy_from_slice(&le);
        ChaCha8Rng::from_seed(seed_bytes)
    }
}

impl BiomeSeed {
    pub fn level(&self, index: u64) -> LevelSeed {
        LevelSeed(derive(self.0, index))
    }

    pub fn rng(&self) -> ChaCha8Rng {
        let mut seed_bytes = [0u8; 32];
        let le = self.0.to_le_bytes();
        seed_bytes[..8].copy_from_slice(&le);
        ChaCha8Rng::from_seed(seed_bytes)
    }
}

impl LevelSeed {
    pub fn structure(&self) -> u64 {
        derive(self.0, 0)
    }

    pub fn bricks(&self) -> u64 {
        derive(self.0, 1)
    }

    pub fn health(&self) -> u64 {
        derive(self.0, 2)
    }

    pub fn specials(&self) -> u64 {
        derive(self.0, 3)
    }

    pub fn powerups(&self) -> u64 {
        derive(self.0, 4)
    }

    pub fn validation(&self) -> u64 {
        derive(self.0, 5)
    }

    pub fn rng(&self) -> ChaCha8Rng {
        let mut seed_bytes = [0u8; 32];
        let le = self.0.to_le_bytes();
        seed_bytes[..8].copy_from_slice(&le);
        ChaCha8Rng::from_seed(seed_bytes)
    }
}

impl MasterSeed {
    pub fn to_share_code(&self) -> String {
        crate::seed::sharing::encode(self.0)
    }

    pub fn from_share_code(code: &str) -> Option<Self> {
        crate::seed::sharing::decode(code).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determinism_galaxy() {
        let master = MasterSeed(12345);
        assert_eq!(master.galaxy(0), master.galaxy(0));
        assert_eq!(master.galaxy(7), master.galaxy(7));
    }

    #[test]
    fn determinism_biome() {
        let master = MasterSeed(99);
        let g = master.galaxy(3);
        assert_eq!(g.biome(0), g.biome(0));
        assert_eq!(g.biome(2), g.biome(2));
    }

    #[test]
    fn determinism_level() {
        let master = MasterSeed(42);
        let b = master.galaxy(0).biome(0).level(5);
        assert_eq!(b.structure(), b.structure());
        assert_eq!(b.bricks(), b.bricks());
        assert_eq!(b.health(), b.health());
        assert_eq!(b.specials(), b.specials());
        assert_eq!(b.powerups(), b.powerups());
        assert_eq!(b.validation(), b.validation());
    }

    #[test]
    fn uniqueness() {
        let m1 = MasterSeed(1);
        let m2 = MasterSeed(2);
        assert_ne!(m1.galaxy(0), m2.galaxy(0));
        assert_ne!(m1.galaxy(0).biome(0), m2.galaxy(0).biome(0));
    }

    #[test]
    fn sub_seeds_differ() {
        let master = MasterSeed(100);
        let level = master.galaxy(0).biome(0).level(0);
        let s = level.structure();
        let b = level.bricks();
        let h = level.health();
        assert_ne!(s, b);
        assert_ne!(s, h);
        assert_ne!(b, h);
    }

    #[test]
    fn share_code_roundtrip() {
        let seeds = [0u64, 1, 42, u64::MAX / 2, u64::MAX];
        for seed in seeds {
            let master = MasterSeed(seed);
            let code = master.to_share_code();
            let decoded = MasterSeed::from_share_code(&code).unwrap();
            assert_eq!(master, decoded, "roundtrip failed for seed {seed}");
        }
    }

    #[test]
    fn rng_determinism() {
        use rand::prelude::*;
        let master = MasterSeed(777);
        let level = master.galaxy(0).biome(0).level(0);
        let mut rng1 = level.rng();
        let mut rng2 = level.rng();
        for _ in 0..100 {
            assert_eq!(rng1.random::<u64>(), rng2.random::<u64>());
        }
    }

    #[test]
    fn rng_different_seeds() {
        use rand::prelude::*;
        let master = MasterSeed(777);
        let l1 = master.galaxy(0).biome(0).level(0);
        let l2 = master.galaxy(0).biome(0).level(1);
        let mut rng1 = l1.rng();
        let mut rng2 = l2.rng();
        let vals: Vec<(u64, u64)> = (0..10)
            .map(|_| (rng1.random::<u64>(), rng2.random::<u64>()))
            .collect();
        assert!(vals.iter().any(|(a, b)| a != b));
    }

    #[test]
    fn galaxy_count_independence() {
        let master = MasterSeed(500);
        let g3 = master.galaxy(3);
        let g3_again = master.galaxy(3);
        assert_eq!(g3, g3_again);
        let g5 = master.galaxy(5);
        assert_ne!(g3, g5);
    }
}
