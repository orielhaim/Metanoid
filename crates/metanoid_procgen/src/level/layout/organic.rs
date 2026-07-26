use bevy_math::Vec2;
use noiz::prelude::*;
use rand::prelude::*;

use crate::biome::parameters::BiomeParams;
use crate::level::grid::BrickGrid;

fn sample_noise(seed: u32, x: f32, y: f32, period: f32) -> f32 {
    let mut noise = Noise::<common_noise::Simplex>::default();
    noise.set_seed(seed);
    noise.set_period(period);
    let val: f32 = noise.sample_for(Vec2::new(x, y));
    (val + 1.0) / 2.0
}

pub fn simplex_threshold(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let mut grid = BrickGrid::new(cols, rows);
    let seed = rng.random::<u32>();
    let period = 4.0 + (1.0 - params.density) * 8.0;
    let threshold = 1.0 - (0.3 + params.density * 0.5);

    for row in 0..rows {
        for col in 0..cols {
            let val = sample_noise(seed, col as f32, row as f32, period);
            grid.set(col, row, val >= threshold);
        }
    }

    grid
}

pub fn cellular_automata(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let init_density = 0.3 + params.density * 0.4;
    let iterations = 2 + (params.chaos * 4.0) as usize;
    let birth_limit = 4;
    let survival_limit = 3;

    let mut grid = BrickGrid::new(cols, rows);
    for row in 0..rows {
        for col in 0..cols {
            grid.set(col, row, rng.random::<f32>() < init_density);
        }
    }

    for _ in 0..iterations {
        let mut next = grid.clone();
        for row in 0..rows {
            for col in 0..cols {
                let mut neighbors = 0i32;
                for dy in [-1i32, 0, 1] {
                    for dx in [-1i32, 0, 1] {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nc = col as i32 + dx;
                        let nr = row as i32 + dy;
                        if nc >= 0
                            && nc < cols as i32
                            && nr >= 0
                            && nr < rows as i32
                            && grid.get(nc as usize, nr as usize)
                        {
                            neighbors += 1;
                        }
                    }
                }
                let alive = grid.get(col, row);
                next.set(
                    col,
                    row,
                    if alive {
                        neighbors >= survival_limit
                    } else {
                        neighbors >= birth_limit
                    },
                );
            }
        }
        grid = next;
    }

    grid
}

pub fn dla(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let mut grid = BrickGrid::new(cols, rows);
    let cx = cols / 2;
    let cy = rows / 2;
    grid.set(cx, cy, true);

    let target = ((cols * rows) as f32 * (0.2 + params.density * 0.4)) as usize;
    let max_walk = cols * rows * 4;
    let mut placed = 1;

    for _ in 0..max_walk {
        if placed >= target {
            break;
        }

        let edge = rng.random_range(0..4usize);
        let (mut px, mut py) = match edge {
            0 => (rng.random_range(0..cols), 0usize),
            1 => (rng.random_range(0..cols), rows - 1),
            2 => (0usize, rng.random_range(0..rows)),
            _ => (cols - 1, rng.random_range(0..rows)),
        };

        let max_steps = 100 + (params.energy * 200.0) as usize;
        for _ in 0..max_steps {
            let mut stuck = false;
            for dy in [-1i32, 0, 1] {
                for dx in [-1i32, 0, 1] {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nc = px as i32 + dx;
                    let nr = py as i32 + dy;
                    if nc >= 0
                        && nc < cols as i32
                        && nr >= 0
                        && nr < rows as i32
                        && grid.get(nc as usize, nr as usize)
                    {
                        stuck = true;
                        break;
                    }
                }
                if stuck {
                    break;
                }
            }

            if stuck {
                grid.set(px, py, true);
                placed += 1;
                break;
            }

            let dir = rng.random_range(0..4usize);
            let (dx, dy) = match dir {
                0 => (0i32, 1i32),
                1 => (1, 0),
                2 => (0, -1),
                _ => (-1, 0),
            };
            let nx = px as i32 + dx;
            let ny = py as i32 + dy;
            if nx >= 0 && nx < cols as i32 && ny >= 0 && ny < rows as i32 {
                px = nx as usize;
                py = ny as usize;
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
    fn simplex_has_bricks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let g = simplex_threshold(14, 8, &test_params(), &mut rng);
        assert!(g.count_filled() > 0);
    }

    #[test]
    fn cellular_has_bricks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let g = cellular_automata(14, 8, &test_params(), &mut rng);
        assert!(g.count_filled() > 0);
    }

    #[test]
    fn dla_has_bricks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let g = dla(14, 8, &test_params(), &mut rng);
        assert!(g.count_filled() > 1);
    }

    #[test]
    fn simplex_deterministic() {
        let p = test_params();
        let mut r1 = ChaCha8Rng::seed_from_u64(55);
        let mut r2 = ChaCha8Rng::seed_from_u64(55);
        assert_eq!(
            simplex_threshold(14, 8, &p, &mut r1),
            simplex_threshold(14, 8, &p, &mut r2)
        );
    }

    #[test]
    fn cellular_deterministic() {
        let p = test_params();
        let mut r1 = ChaCha8Rng::seed_from_u64(55);
        let mut r2 = ChaCha8Rng::seed_from_u64(55);
        assert_eq!(
            cellular_automata(14, 8, &p, &mut r1),
            cellular_automata(14, 8, &p, &mut r2)
        );
    }
}
