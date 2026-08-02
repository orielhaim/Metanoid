use crate::level::data::BrickData;

pub fn validate_and_fix(bricks: &mut Vec<BrickData>, cols: usize, rows: usize) {
    remove_trapped_bricks(bricks, cols, rows);
    remove_isolated_invincible(bricks);
    ensure_minimum_destructible(bricks);
}

fn remove_trapped_bricks(bricks: &mut Vec<BrickData>, cols: usize, rows: usize) {
    let mut invincible_grid = vec![false; cols * rows];
    for b in bricks.iter() {
        if !b.is_destructible() {
            invincible_grid[b.row * cols + b.col] = true;
        }
    }

    let mut destructible_grid = vec![false; cols * rows];
    for b in bricks.iter() {
        if b.is_destructible() {
            destructible_grid[b.row * cols + b.col] = true;
        }
    }

    let mut reachable = vec![false; cols * rows];
    let mut stack = Vec::new();

    for col in 0..cols {
        if !invincible_grid[(rows - 1) * cols + col] {
            let idx = (rows - 1) * cols + col;
            reachable[idx] = true;
            stack.push(idx);
        }
    }

    while let Some(idx) = stack.pop() {
        let row = idx / cols;
        let col = idx % cols;
        for (dr, dc) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
            let nr = row as i32 + dr;
            let nc = col as i32 + dc;
            if nr >= 0 && nr < rows as i32 && nc >= 0 && nc < cols as i32 {
                let nidx = nr as usize * cols + nc as usize;
                if !reachable[nidx] && !invincible_grid[nidx] {
                    reachable[nidx] = true;
                    stack.push(nidx);
                }
            }
        }
    }

    let mut removed_powerups = false;
    bricks.retain(|b| {
        if b.is_destructible() && !reachable[b.row * cols + b.col] {
            if b.powerup_chance > 0.0 {
                removed_powerups = true;
            }
            false
        } else {
            true
        }
    });

    if removed_powerups {
        use rand::prelude::*;
        let mut rng = rand::rng();
        let eligible: Vec<usize> = (0..bricks.len())
            .filter(|&i| bricks[i].is_destructible() && bricks[i].powerup_chance == 0.0)
            .collect();
        if !eligible.is_empty() {
            let idx = eligible[rng.random_range(0..eligible.len())];
            bricks[idx].powerup_chance = 0.7;
        }
    }
}

fn remove_isolated_invincible(bricks: &mut Vec<BrickData>) {
    let invincible_count = bricks.iter().filter(|b| !b.is_destructible()).count();
    let total = bricks.len().max(1);
    if invincible_count > total / 3 {
        let excess = invincible_count - total / 3;
        let mut converted = 0;
        for brick in bricks.iter_mut() {
            if converted >= excess {
                break;
            }
            if !brick.is_destructible() {
                brick.kind = crate::level::data::BrickKind::Normal;
                brick.health = 1;
                brick.max_health = 1;
                converted += 1;
            }
        }
    }
}

fn ensure_minimum_destructible(bricks: &mut Vec<BrickData>) {
    let destructible = bricks.iter().filter(|b| b.is_destructible()).count();
    if destructible < 3 && !bricks.is_empty() {
        for brick in bricks.iter_mut() {
            if !brick.is_destructible() {
                brick.kind = crate::level::data::BrickKind::Normal;
                brick.health = 1;
                brick.max_health = 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::data::{BrickData, BrickKind};

    #[test]
    fn removes_nothing_when_valid() {
        let mut bricks: Vec<BrickData> = (0..5).map(|i| BrickData::normal(i, 0)).collect();
        let original = bricks.len();
        validate_and_fix(&mut bricks, 5, 3);
        assert_eq!(bricks.len(), original);
    }

    #[test]
    fn caps_invincible_ratio() {
        let mut bricks: Vec<BrickData> = (0..10)
            .map(|i| {
                let mut b = BrickData::normal(i % 5, i / 5);
                b.kind = BrickKind::Invincible;
                b
            })
            .collect();
        validate_and_fix(&mut bricks, 5, 3);
        let invincible = bricks.iter().filter(|b| !b.is_destructible()).count();
        assert!(invincible <= bricks.len() / 3);
    }
}
