use crate::level::data::BrickData;

pub fn carve_negative_space(bricks: &mut Vec<BrickData>, cols: usize, rows: usize) {
    let reachable = simulate_ball_paths(bricks, cols, rows);

    bricks.retain(|b| {
        if !b.is_destructible() {
            return true;
        }
        reachable[b.row * cols + b.col]
    });
}

fn simulate_ball_paths(bricks: &[BrickData], cols: usize, rows: usize) -> Vec<bool> {
    let mut occupied = vec![false; cols * rows];
    for b in bricks {
        occupied[b.row * cols + b.col] = true;
    }

    let mut reachable = vec![false; cols * rows];
    let num_rays = cols * 2 + 4;

    for i in 0..num_rays {
        let mut fx = (i as f32 / num_rays as f32) * cols as f32;
        let mut fy = rows as f32 + 1.0;

        let vx = (i as f32 / num_rays as f32 - 0.5) * 0.4;
        let vy = -1.0;
        let len = (vx * vx + vy * vy).sqrt();
        let vx = vx / len;
        let mut vy = vy / len;

        for _ in 0..200 {
            fx += vx;
            fy += vy;

            let col = fx.round() as i32;
            let row = fy.round() as i32;

            if col >= 0 && col < cols as i32 && row >= 0 && row < rows as i32 {
                let idx = row as usize * cols + col as usize;
                reachable[idx] = true;
                if occupied[idx] {
                    vy = -vy;
                    fy += vy * 2.0;
                }
            }

            if fy < -1.0 || fy > rows as f32 + 2.0 {
                vy = -vy;
                fy = fy.clamp(-1.0, rows as f32 + 2.0);
            }
            if fx < -1.0 || fx > cols as f32 + 1.0 {
                break;
            }
        }
    }

    for b in bricks.iter() {
        if b.is_explosive() {
            propagate_explosion_reachability(&mut reachable, &occupied, b.col, b.row, cols, rows);
        }
    }

    reachable
}

fn propagate_explosion_reachability(
    reachable: &mut [bool],
    occupied: &[bool],
    col: usize,
    row: usize,
    cols: usize,
    rows: usize,
) {
    for dr in -1i32..=1 {
        for dc in -1i32..=1 {
            if dr == 0 && dc == 0 {
                continue;
            }
            let nc = col as i32 + dc;
            let nr = row as i32 + dr;
            if nc >= 0 && nc < cols as i32 && nr >= 0 && nr < rows as i32 {
                let idx = nr as usize * cols + nc as usize;
                if occupied[idx] {
                    reachable[idx] = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::data::{BrickData, BrickKind};

    #[test]
    fn all_reachable_stay() {
        let bricks: Vec<BrickData> = (0..5).map(|i| BrickData::normal(i, 0)).collect();
        let mut result = bricks.clone();
        carve_negative_space(&mut result, 5, 3);
        assert_eq!(result.len(), bricks.len());
    }

    #[test]
    fn isolated_brick_removed() {
        let mut bricks = vec![BrickData::normal(0, 0), BrickData::normal(4, 0)];
        let invincible = {
            let mut b = BrickData::normal(2, 0);
            b.kind = BrickKind::Invincible;
            b
        };
        bricks.push(invincible);
        carve_negative_space(&mut bricks, 5, 3);
        assert!(bricks.iter().any(|b| b.kind == BrickKind::Invincible));
    }
}
