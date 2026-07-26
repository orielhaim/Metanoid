use rand::prelude::*;

use crate::biome::parameters::BiomeParams;
use crate::level::grid::BrickGrid;
use super::geometric;
use super::hybrid;
use super::organic;

pub enum PatternCategory {
    Geometric,
    Hybrid,
    Organic,
}

pub fn select_category(chaos: f32, rng: &mut impl Rng) -> PatternCategory {
    let roll = rng.random::<f32>();
    if chaos < 0.33 {
        if roll < 0.7 {
            PatternCategory::Geometric
        } else {
            PatternCategory::Hybrid
        }
    } else if chaos < 0.66 {
        if roll < 0.4 {
            PatternCategory::Geometric
        } else if roll < 0.8 {
            PatternCategory::Hybrid
        } else {
            PatternCategory::Organic
        }
    } else if roll < 0.2 {
        PatternCategory::Geometric
    } else if roll < 0.5 {
        PatternCategory::Hybrid
    } else {
        PatternCategory::Organic
    }
}

pub fn generate_layout(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    match select_category(params.chaos, rng) {
        PatternCategory::Geometric => generate_geometric(cols, rows, params, rng),
        PatternCategory::Hybrid => generate_hybrid(cols, rows, params, rng),
        PatternCategory::Organic => generate_organic(cols, rows, params, rng),
    }
}

fn generate_geometric(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let variant = rng.random_range(0..5);
    match variant {
        0 => geometric::symmetric_mirror(cols, rows, params, rng),
        1 => geometric::concentric_rings(cols, rows, params, rng),
        2 => geometric::grid_cutouts(cols, rows, params, rng),
        3 => geometric::diagonal_stripes(cols, rows, params, rng),
        _ => geometric::tessellation(cols, rows, params, rng),
    }
}

fn generate_hybrid(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let variant = rng.random_range(0..4);
    match variant {
        0 => hybrid::wave_function(cols, rows, params, rng),
        1 => hybrid::voronoi_regions(cols, rows, params, rng),
        2 => hybrid::fractal_subdivision(cols, rows, params, rng),
        _ => hybrid::lsystem_growth(cols, rows, params, rng),
    }
}

fn generate_organic(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let variant = rng.random_range(0..3);
    match variant {
        0 => organic::simplex_threshold(cols, rows, params, rng),
        1 => organic::cellular_automata(cols, rows, params, rng),
        _ => organic::dla(cols, rows, params, rng),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use crate::seed::hierarchy::MasterSeed;
    use crate::biome::generator::BiomeGenerator;

    #[test]
    fn generate_layout_has_bricks() {
        let master = MasterSeed(42);
        for i in 0..30 {
            let biome_seed = master.galaxy(0).biome(i);
            let params = BiomeGenerator::generate(biome_seed);
            let mut rng = biome_seed.rng();
            let grid = generate_layout(14, 8, &params, &mut rng);
            assert!(grid.count_filled() > 0, "empty layout at biome {i}");
        }
    }

    #[test]
    fn generate_layout_deterministic() {
        let master = MasterSeed(99);
        let biome_seed = master.galaxy(0).biome(0);
        let params = BiomeGenerator::generate(biome_seed);

        let mut r1 = biome_seed.rng();
        let mut r2 = biome_seed.rng();
        assert_eq!(generate_layout(14, 8, &params, &mut r1), generate_layout(14, 8, &params, &mut r2));
    }

    #[test]
    fn variety_across_biomes() {
        let master = MasterSeed(200);
        let grids: Vec<usize> = (0..20)
            .map(|i| {
                let biome_seed = master.galaxy(0).biome(i);
                let params = BiomeGenerator::generate(biome_seed);
                let mut rng = biome_seed.rng();
                generate_layout(14, 8, &params, &mut rng).count_filled()
            })
            .collect();
        let all_same = grids.windows(2).all(|w| w[0] == w[1]);
        assert!(!all_same, "layouts should vary across biomes");
    }

    #[test]
    fn low_chaos_prefers_geometric() {
        let mut geometric_count = 0;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for _ in 0..100 {
            if matches!(select_category(0.1, &mut rng), PatternCategory::Geometric) {
                geometric_count += 1;
            }
        }
        assert!(geometric_count > 50, "low chaos should prefer geometric: {geometric_count}");
    }

    #[test]
    fn high_chaos_prefers_organic() {
        let mut organic_count = 0;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for _ in 0..100 {
            if matches!(select_category(0.9, &mut rng), PatternCategory::Organic) {
                organic_count += 1;
            }
        }
        assert!(organic_count > 30, "high chaos should prefer organic: {organic_count}");
    }
}
