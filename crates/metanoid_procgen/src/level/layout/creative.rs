//! Signature creative layouts for more memorable stages.

use rand::prelude::*;

use crate::biome::parameters::BiomeParams;
use crate::level::grid::BrickGrid;

pub fn diamond(cols: usize, rows: usize, _params: &BiomeParams, rng: &mut impl Rng) -> BrickGrid {
    let mut grid = BrickGrid::new(cols, rows);
    let cx = (cols as f32 - 1.0) / 2.0;
    let cy = (rows as f32 - 1.0) / 2.0;
    let radius = (cols.min(rows) as f32) * 0.42;
    let hollow = rng.random::<f32>() < 0.45;
    for r in 0..rows {
        for c in 0..cols {
            let d = (c as f32 - cx).abs() + (r as f32 - cy).abs();
            let on = if hollow {
                d >= radius - 1.2 && d <= radius + 0.4
            } else {
                d <= radius
            };
            if on {
                grid.set(c, r, true);
            }
        }
    }
    maybe_punch_gaps(&mut grid, rng, 0.08);
    grid
}

pub fn chevron(cols: usize, rows: usize, _params: &BiomeParams, rng: &mut impl Rng) -> BrickGrid {
    let mut grid = BrickGrid::new(cols, rows);
    let dir = if rng.random::<bool>() { 1.0 } else { -1.0 };
    let thickness = 1 + rng.random_range(0..2);
    for r in 0..rows {
        for c in 0..cols {
            let wave = ((c as f32 - cols as f32 / 2.0) * dir).abs() * 0.55;
            let band = (r as f32 - wave).abs();
            if band < thickness as f32 + 0.6 {
                grid.set(c, r, true);
            }
        }
    }
    // Second chevron
    if rng.random::<f32>() < 0.6 {
        for r in 0..rows {
            for c in 0..cols {
                let wave =
                    ((c as f32 - cols as f32 / 2.0) * -dir).abs() * 0.55 + rows as f32 * 0.35;
                if (r as f32 - wave).abs() < 1.2 {
                    grid.set(c, r, true);
                }
            }
        }
    }
    grid
}

pub fn castle(cols: usize, rows: usize, _params: &BiomeParams, rng: &mut impl Rng) -> BrickGrid {
    let mut grid = BrickGrid::new(cols, rows);
    // Walls
    for r in 0..rows {
        for c in 0..cols {
            let border = r == 0 || r == rows - 1 || c == 0 || c == cols - 1;
            let battlement = r == 1 && c % 2 == 0;
            let keep = c >= cols / 2 - 1 && c <= cols / 2 + 1 && r >= rows / 2 - 1 && r <= rows - 2;
            if border || battlement || keep {
                grid.set(c, r, true);
            }
        }
    }
    // Gate gap
    if cols > 4 && rows > 2 {
        let gx = cols / 2;
        grid.set(gx, rows - 1, false);
        grid.set(gx.saturating_sub(1), rows - 1, false);
        if gx + 1 < cols {
            grid.set(gx + 1, rows - 1, false);
        }
    }
    if rng.random::<f32>() < 0.5 {
        maybe_punch_gaps(&mut grid, rng, 0.05);
    }
    grid
}

pub fn hourglass(
    cols: usize,
    rows: usize,
    _params: &BiomeParams,
    _rng: &mut impl Rng,
) -> BrickGrid {
    let mut grid = BrickGrid::new(cols, rows);
    let cx = (cols as f32 - 1.0) / 2.0;
    for r in 0..rows {
        let t = r as f32 / (rows.saturating_sub(1).max(1) as f32);
        let width = 0.15 + (0.5 - (t - 0.5).abs()) * 1.4;
        let half = (cols as f32 * width * 0.5).max(1.0);
        for c in 0..cols {
            if (c as f32 - cx).abs() <= half {
                grid.set(c, r, true);
            }
        }
    }
    grid
}

pub fn spiral(cols: usize, rows: usize, _params: &BiomeParams, rng: &mut impl Rng) -> BrickGrid {
    let mut grid = BrickGrid::new(cols, rows);
    let mut left = 0usize;
    let mut right = cols.saturating_sub(1);
    let mut top = 0usize;
    let mut bottom = rows.saturating_sub(1);
    let mut paint = true;
    let mut steps = 0;
    while left <= right && top <= bottom && steps < cols * rows {
        steps += 1;
        for c in left..=right {
            if paint {
                grid.set(c, top, true);
            }
        }
        if top == bottom {
            break;
        }
        top += 1;
        for r in top..=bottom {
            if paint {
                grid.set(right, r, true);
            }
        }
        if left == right {
            break;
        }
        if right == 0 {
            break;
        }
        right -= 1;
        for c in (left..=right).rev() {
            if paint {
                grid.set(c, bottom, true);
            }
        }
        if bottom == 0 {
            break;
        }
        bottom -= 1;
        for r in (top..=bottom).rev() {
            if paint {
                grid.set(left, r, true);
            }
        }
        left += 1;
        // Skip every other ring for open lanes
        paint = !paint || rng.random::<f32>() < 0.35;
        if left > right || top > bottom {
            break;
        }
    }
    grid
}

pub fn checker_burst(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> BrickGrid {
    let mut grid = BrickGrid::new(cols, rows);
    let phase = rng.random_range(0..2);
    let density = 0.45 + params.density * 0.35;
    for r in 0..rows {
        for c in 0..cols {
            let checker = (c + r + phase) % 2 == 0;
            if checker && rng.random::<f32>() < density {
                grid.set(c, r, true);
            } else if !checker && rng.random::<f32>() < density * 0.35 {
                grid.set(c, r, true);
            }
        }
    }
    grid
}

fn maybe_punch_gaps(grid: &mut BrickGrid, rng: &mut impl Rng, rate: f32) {
    let cols = grid.cols;
    let rows = grid.rows;
    for r in 0..rows {
        for c in 0..cols {
            if grid.get(c, r) && rng.random::<f32>() < rate {
                grid.set(c, r, false);
            }
        }
    }
}
