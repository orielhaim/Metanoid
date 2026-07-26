use rand::prelude::*;

use crate::biome::parameters::BiomeParams;
use crate::level::grid::BrickGrid;

pub fn symmetric_mirror(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let mut grid = BrickGrid::new(cols, rows);
    let half_cols = (cols + 1) / 2;
    let half_rows = (rows + 1) / 2;

    let density = 0.3 + params.density * 0.5;
    let noise_scale = 0.2 + params.chaos * 0.4;

    for row in 0..half_rows {
        for col in 0..half_cols {
            let center_dist = ((col as f32 / half_cols as f32) - 0.5).abs()
                + ((row as f32 / half_rows as f32) - 0.5).abs();
            let prob = density - center_dist * noise_scale + rng.random::<f32>() * 0.2;
            if rng.random::<f32>() < prob {
                grid.set(col, row, true);
            }
        }
    }

    if params.chaos < 0.33 {
        grid.mirror_both();
    } else if params.chaos < 0.66 {
        grid.mirror_horizontal();
        grid.mirror_vertical();
    } else {
        grid.mirror_horizontal();
    }

    grid
}

pub fn concentric_rings(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let mut grid = BrickGrid::new(cols, rows);
    let cx = cols as f32 / 2.0;
    let cy = rows as f32 / 2.0;
    let max_r = (cx * cx + cy * cy).sqrt();
    let ring_width = 0.8 + (1.0 - params.density) * 1.2;
    let noise_amp = params.chaos * 0.3;

    for row in 0..rows {
        for col in 0..cols {
            let dx = (col as f32 + 0.5 - cx) / cx;
            let dy = (row as f32 + 0.5 - cy) / cy;
            let dist = (dx * dx + dy * dy).sqrt() * max_r / 1.5;
            let noise = rng.random::<f32>() * noise_amp;
            let ring = (dist + noise) % ring_width;
            grid.set(col, row, ring < ring_width * 0.6);
        }
    }

    grid
}

pub fn grid_cutouts(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let mut grid = BrickGrid::filled(cols, rows);
    let cutout_prob = 0.1 + (1.0 - params.density) * 0.3;
    let cluster_size = 1 + (params.chaos * 4.0) as usize;

    let num_cutouts = ((cols * rows) as f32 * cutout_prob * 0.3) as usize;
    for _ in 0..num_cutouts {
        let start_col = rng.random_range(0..cols);
        let start_row = rng.random_range(0..rows);
        for dy in 0..cluster_size {
            for dx in 0..cluster_size {
                let c = start_col.wrapping_add(dx);
                let r = start_row.wrapping_add(dy);
                grid.set(c % cols, r % rows, false);
            }
        }
    }

    grid
}

pub fn diagonal_stripes(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let mut grid = BrickGrid::new(cols, rows);
    let stripe_width = 1.0 + (1.0 - params.density) * 2.0;
    let angle = params.chaos * 0.5;
    let noise_amp = params.chaos * 0.3;

    for row in 0..rows {
        for col in 0..cols {
            let x = col as f32 + row as f32 * angle;
            let noise = rng.random::<f32>() * noise_amp;
            let stripe = (x + noise) % stripe_width;
            grid.set(col, row, stripe < stripe_width * 0.5);
        }
    }

    grid
}

pub fn tessellation(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let mut grid = BrickGrid::new(cols, rows);
    let tile_w = 2 + (params.density * 3.0) as usize;
    let tile_h = 2 + (params.density * 2.0) as usize;
    let fill_density = 0.4 + params.density * 0.4;

    let mut tile = vec![false; tile_w * tile_h];
    for cell in tile.iter_mut() {
        *cell = rng.random::<f32>() < fill_density;
    }

    for row in 0..rows {
        for col in 0..cols {
            let tc = col % tile_w;
            let tr = row % tile_h;
            let val = tile[tr * tile_w + tc];
            if params.chaos > 0.5 && rng.random::<f32>() < params.chaos * 0.2 {
                grid.set(col, row, !val);
            } else {
                grid.set(col, row, val);
            }
        }
    }

    grid
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn test_params() -> BiomeParams {
        BiomeParams {
            temperature: 0.5,
            density: 0.6,
            chaos: 0.3,
            energy: 0.5,
            weirdness: 0.2,
        }
    }

    #[test]
    fn symmetric_has_bricks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let g = symmetric_mirror(14, 8, &test_params(), &mut rng);
        assert!(g.count_filled() > 0);
    }

    #[test]
    fn concentric_has_bricks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let g = concentric_rings(14, 8, &test_params(), &mut rng);
        assert!(g.count_filled() > 0);
    }

    #[test]
    fn grid_cutouts_not_full() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let g = grid_cutouts(14, 8, &test_params(), &mut rng);
        assert!(g.count_filled() < 14 * 8);
    }

    #[test]
    fn diagonal_has_bricks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let g = diagonal_stripes(14, 8, &test_params(), &mut rng);
        assert!(g.count_filled() > 0);
    }

    #[test]
    fn tessellation_has_bricks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let g = tessellation(14, 8, &test_params(), &mut rng);
        assert!(g.count_filled() > 0);
    }

    #[test]
    fn deterministic_with_seed() {
        let p = test_params();
        let mut r1 = ChaCha8Rng::seed_from_u64(99);
        let mut r2 = ChaCha8Rng::seed_from_u64(99);
        assert_eq!(
            symmetric_mirror(14, 8, &p, &mut r1),
            symmetric_mirror(14, 8, &p, &mut r2)
        );
    }
}
