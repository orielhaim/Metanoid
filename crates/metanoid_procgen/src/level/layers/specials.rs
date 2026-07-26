use rand::prelude::*;

use crate::biome::parameters::BiomeParams;
use crate::level::data::{BrickData, SpecialType};

pub fn place_specials(
    bricks: &mut [BrickData],
    cols: usize,
    _rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) {
    let special_budget = ((bricks.len() as f32) * params.chaos * 0.15) as usize;
    if special_budget == 0 {
        return;
    }

    let mut candidates: Vec<usize> = (0..bricks.len())
        .filter(|&i| bricks[i].special == SpecialType::None && bricks[i].is_destructible())
        .collect();

    for _ in 0..special_budget {
        if candidates.is_empty() {
            break;
        }

        let ci = rng.random_range(0..candidates.len());
        let bi = candidates[ci];
        candidates.swap_remove(ci);

        let at_edge = bricks[bi].col == 0 || bricks[bi].col == cols - 1;
        let at_top = bricks[bi].row <= 1;

        let special = if at_edge && rng.random::<f32>() < 0.4 {
            SpecialType::Moving
        } else if at_top && rng.random::<f32>() < 0.3 {
            SpecialType::Regenerating
        } else {
            match rng.random_range(0..3) {
                0 => SpecialType::Moving,
                1 => SpecialType::Regenerating,
                _ => SpecialType::Teleport,
            }
        };

        bricks[bi].special = special;

        if special == SpecialType::Teleport {
            let brick_col = bricks[bi].col;
            let brick_row = bricks[bi].row;
            let partner = candidates.iter().position(|&j| {
                let dx = bricks[j].col as i32 - brick_col as i32;
                let dy = bricks[j].row as i32 - brick_row as i32;
                (dx.abs() + dy.abs()) >= 3
            });
            if let Some(pi) = partner {
                let pj = candidates[pi];
                bricks[pj].special = SpecialType::Teleport;
                candidates.swap_remove(pi);
            } else {
                bricks[bi].special = SpecialType::None;
            }
        }
    }
}
