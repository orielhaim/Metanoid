#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelAddress {
    pub galaxy: u64,
    pub biome: u64,
    pub level: u64,
}

impl LevelAddress {
    pub fn new(galaxy: u64, biome: u64, level: u64) -> Self {
        Self {
            galaxy,
            biome,
            level,
        }
    }

    pub fn is_boss(&self, levels_per_biome: u64) -> bool {
        self.level == levels_per_biome - 1
    }
}

pub const LEVELS_PER_BIOME: u64 = 12;

pub struct Progression;

impl Progression {
    pub fn address_at(total_level: u64, biomes_per_galaxy: &[usize]) -> LevelAddress {
        let mut remaining = total_level;
        let mut galaxy = 0u64;

        loop {
            let biome_count = biomes_per_galaxy
                .get(galaxy as usize % biomes_per_galaxy.len())
                .copied()
                .unwrap_or(4) as u64;
            let galaxy_levels = biome_count * LEVELS_PER_BIOME;

            if remaining < galaxy_levels {
                let biome = remaining / LEVELS_PER_BIOME;
                let level = remaining % LEVELS_PER_BIOME;
                return LevelAddress::new(galaxy, biome, level);
            }

            remaining -= galaxy_levels;
            galaxy += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_level() {
        let addr = Progression::address_at(0, &[4]);
        assert_eq!(addr, LevelAddress::new(0, 0, 0));
    }

    #[test]
    fn wraps_biomes() {
        let addr = Progression::address_at(12, &[4]);
        assert_eq!(addr.biome, 1);
        assert_eq!(addr.level, 0);
    }

    #[test]
    fn wraps_galaxies() {
        let biomes = [4usize];
        let galaxy_size = 4 * LEVELS_PER_BIOME;
        let addr = Progression::address_at(galaxy_size, &biomes);
        assert_eq!(addr.galaxy, 1);
        assert_eq!(addr.biome, 0);
    }

    #[test]
    fn boss_detection() {
        let addr = LevelAddress::new(0, 0, LEVELS_PER_BIOME - 1);
        assert!(addr.is_boss(LEVELS_PER_BIOME));
        assert!(!LevelAddress::new(0, 0, 0).is_boss(LEVELS_PER_BIOME));
    }

    #[test]
    fn sequence_100_levels() {
        let biomes = [4usize, 5, 3, 6];
        for i in 0..100 {
            let addr = Progression::address_at(i, &biomes);
            assert!(addr.biome < 6, "level {i} biome={}", addr.biome);
            assert!(addr.level < LEVELS_PER_BIOME);
        }
    }
}
