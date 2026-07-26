use rand::prelude::*;

use crate::biome::parameters::BiomeParams;
use crate::level::grid::BrickGrid;

pub fn wave_function(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let mut grid = BrickGrid::new(cols, rows);
    let freq_x = 1.0 + params.temperature * 3.0;
    let freq_y = 1.0 + params.energy * 2.0;
    let amp = 0.3 + params.density * 0.4;
    let phase = rng.random::<f32>() * std::f32::consts::TAU;
    let noise_amp = params.chaos * 0.3;

    for row in 0..rows {
        for col in 0..cols {
            let x = col as f32 / cols as f32;
            let y = row as f32 / rows as f32;
            let wave = (x * freq_x * std::f32::consts::TAU + phase).sin() * 0.5
                + (y * freq_y * std::f32::consts::TAU).sin() * 0.5;
            let noise = (rng.random::<f32>() - 0.5) * noise_amp;
            let val = (wave + 1.0) / 2.0 + noise;
            grid.set(col, row, val > (1.0 - amp));
        }
    }

    grid
}

pub fn voronoi_regions(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let mut grid = BrickGrid::new(cols, rows);
    let num_seeds = 3 + (params.density * 8.0) as usize;
    let fill_ratio = 0.4 + params.density * 0.4;

    let seeds: Vec<(f32, f32, bool)> = (0..num_seeds)
        .map(|_| {
            (
                rng.random::<f32>() * cols as f32,
                rng.random::<f32>() * rows as f32,
                rng.random::<f32>() < fill_ratio,
            )
        })
        .collect();

    for row in 0..rows {
        for col in 0..cols {
            let mut min_dist = f32::MAX;
            let mut fill = false;
            for &(sx, sy, should_fill) in &seeds {
                let dx = col as f32 - sx;
                let dy = row as f32 - sy;
                let dist = if params.chaos > 0.5 {
                    (dx * dx + dy * dy).sqrt() + rng.random::<f32>() * params.chaos
                } else {
                    (dx * dx + dy * dy).sqrt()
                };
                if dist < min_dist {
                    min_dist = dist;
                    fill = should_fill;
                }
            }
            grid.set(col, row, fill);
        }
    }

    grid
}

pub fn fractal_subdivision(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let mut grid = BrickGrid::filled(cols, rows);
    let depth = 2 + (params.chaos * 3.0) as usize;
    let remove_prob = 0.2 + (1.0 - params.density) * 0.3;

    fn subdivide(
        grid: &mut BrickGrid,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        depth: usize,
        remove_prob: f32,
        rng: &mut impl Rng,
    ) {
        if depth == 0 || w < 2 || h < 2 {
            return;
        }

        if rng.random::<f32>() < remove_prob {
            let rx = x + rng.random_range(0..w.saturating_sub(1).max(1));
            let ry = y + rng.random_range(0..h.saturating_sub(1).max(1));
            let rw = rng.random_range(1..=(w / 2).max(1));
            let rh = rng.random_range(1..=(h / 2).max(1));
            for dy in 0..rh {
                for dx in 0..rw {
                    grid.set(rx + dx, ry + dy, false);
                }
            }
        }

        let hw = w / 2;
        let hh = h / 2;
        if hw > 0 && hh > 0 {
            subdivide(grid, x, y, hw, hh, depth - 1, remove_prob, rng);
            subdivide(grid, x + hw, y, w - hw, hh, depth - 1, remove_prob, rng);
            subdivide(grid, x, y + hh, hw, h - hh, depth - 1, remove_prob, rng);
            subdivide(grid, x + hw, y + hh, w - hw, h - hh, depth - 1, remove_prob, rng);
        }
    }

    subdivide(&mut grid, 0, 0, cols, rows, depth, remove_prob, rng);
    grid
}

pub fn lsystem_growth(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let mut grid = BrickGrid::new(cols, rows);
    let branches = 2 + (params.density * 4.0) as usize;
    let max_steps = 10 + (params.energy * 20.0) as usize;
    let turn_prob = 0.2 + params.chaos * 0.3;

    for _ in 0..branches {
        let mut x = rng.random_range(0..cols);
        let mut y = rng.random_range(0..rows);
        let mut dir: i32 = rng.random_range(0..4);

        for _ in 0..max_steps {
            grid.set(x, y, true);

            if rng.random::<f32>() < turn_prob {
                dir = (dir + if rng.random::<f32>() < 0.5 { 1 } else { 3 }) % 4;
            }

            let (dx, dy) = match dir {
                0 => (0i32, 1i32),
                1 => (1, 0),
                2 => (0, -1),
                _ => (-1, 0),
            };
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && nx < cols as i32 && ny >= 0 && ny < rows as i32 {
                x = nx as usize;
                y = ny as usize;
            } else {
                break;
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
            chaos: 0.4,
            energy: 0.5,
            weirdness: 0.2,
        }
    }

    #[test]
    fn wave_has_bricks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let g = wave_function(14, 8, &test_params(), &mut rng);
        assert!(g.count_filled() > 0);
    }

    #[test]
    fn voronoi_has_bricks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let g = voronoi_regions(14, 8, &test_params(), &mut rng);
        assert!(g.count_filled() > 0);
    }

    #[test]
    fn fractal_not_full() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let g = fractal_subdivision(14, 8, &test_params(), &mut rng);
        assert!(g.count_filled() < 14 * 8);
    }

    #[test]
    fn lsystem_has_bricks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let g = lsystem_growth(14, 8, &test_params(), &mut rng);
        assert!(g.count_filled() > 0);
    }

    #[test]
    fn deterministic() {
        let p = test_params();
        let mut r1 = ChaCha8Rng::seed_from_u64(77);
        let mut r2 = ChaCha8Rng::seed_from_u64(77);
        assert_eq!(wave_function(14, 8, &p, &mut r1), wave_function(14, 8, &p, &mut r2));
    }
}
