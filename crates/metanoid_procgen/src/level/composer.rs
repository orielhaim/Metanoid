use rand::prelude::*;

use crate::biome::parameters::BiomeParams;
use crate::level::data::{BrickData, BrickMetrics, LevelDefinition};
use crate::level::layers::{base, brick_type, carving, health, powerups, specials, validator};

const DEFAULT_COLS: usize = 14;
const DEFAULT_ROWS: usize = 8;

pub fn compose_level(params: &BiomeParams, rng: &mut impl Rng) -> LevelDefinition {
    compose_level_sized(DEFAULT_COLS, DEFAULT_ROWS, params, rng)
}

pub fn compose_level_sized(
    cols: usize,
    rows: usize,
    params: &BiomeParams,
    rng: &mut impl Rng,
) -> LevelDefinition {
    // Layer 1: Base structure
    let grid = base::generate_base_structure(cols, rows, params, rng);

    // Convert grid to BrickData
    let mut bricks: Vec<BrickData> = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            if grid.get(col, row) {
                bricks.push(BrickData::normal(col, row));
            }
        }
    }

    // Layer 2: Brick type assignment
    brick_type::assign_brick_types(&mut bricks, cols, rows, params, rng);

    // Layer 3: Health distribution
    health::distribute_health(&mut bricks, rows, params, rng);

    // Layer 4: Special placement
    specials::place_specials(&mut bricks, cols, rows, params, rng);

    // Layer 5: Power-up seeding
    powerups::seed_powerups(&mut bricks, params, rng);

    // Layer 6: Negative space carving
    carving::carve_negative_space(&mut bricks, cols, rows);

    // Layer 7: Validation & fix
    validator::validate_and_fix(&mut bricks, cols, rows);

    // Layer 8: Never ship an empty/unplayable board
    ensure_playable(&mut bricks, cols, rows, rng);

    LevelDefinition {
        cols,
        rows,
        bricks,
        metrics: BrickMetrics {
            cols,
            rows,
            ..BrickMetrics::default()
        },
    }
}

fn ensure_playable(bricks: &mut Vec<BrickData>, cols: usize, rows: usize, rng: &mut impl Rng) {
    let destructible = bricks.iter().filter(|b| b.is_destructible()).count();
    if destructible >= 3 {
        return;
    }
    // Seed a playable wedge of normal bricks near the top-center
    let target = 8.max(cols / 2);
    let mut added = 0;
    let mut attempts = 0;
    while added < target && attempts < cols * rows * 2 {
        attempts += 1;
        let c = rng.random_range(0..cols);
        let r = rng.random_range(0..rows.max(1).min(4).max(1));
        if bricks.iter().any(|b| b.col == c && b.row == r) {
            continue;
        }
        bricks.push(BrickData::normal(c, r));
        added += 1;
    }
    // Convert any remaining invincible-only leftovers
    if bricks.iter().filter(|b| b.is_destructible()).count() < 3 {
        for b in bricks.iter_mut() {
            if !b.is_destructible() {
                b.kind = crate::level::data::BrickKind::Normal;
                b.health = 1;
                b.max_health = 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biome::generator::BiomeGenerator;
    use crate::seed::hierarchy::MasterSeed;

    #[test]
    fn compose_produces_bricks() {
        let master = MasterSeed(42);
        let biome_seed = master.galaxy(0).biome(0);
        let params = BiomeGenerator::generate(biome_seed);
        let mut rng = biome_seed.rng();
        let level = compose_level(&params, &mut rng);
        assert!(!level.bricks.is_empty(), "level should have bricks");
        assert!(
            level.destructible_count() > 0,
            "level should have destructible bricks"
        );
    }

    #[test]
    fn compose_deterministic() {
        let master = MasterSeed(99);
        let biome_seed = master.galaxy(0).biome(0);
        let params = BiomeGenerator::generate(biome_seed);

        let mut r1 = biome_seed.rng();
        let mut r2 = biome_seed.rng();
        let l1 = compose_level(&params, &mut r1);
        let l2 = compose_level(&params, &mut r2);

        assert_eq!(l1.bricks.len(), l2.bricks.len());
        for (a, b) in l1.bricks.iter().zip(l2.bricks.iter()) {
            assert_eq!(a.col, b.col);
            assert_eq!(a.row, b.row);
            assert_eq!(a.kind, b.kind);
            assert_eq!(a.health, b.health);
            assert_eq!(a.special, b.special);
        }
    }

    #[test]
    fn compose_100_levels_no_crash() {
        let master = MasterSeed(1000);
        for i in 0..100 {
            let galaxy = i / 20;
            let biome = i % 20;
            let biome_seed = master.galaxy(galaxy).biome(biome);
            let params = BiomeGenerator::generate(biome_seed);
            let mut rng = biome_seed.rng();
            let level = compose_level(&params, &mut rng);
            assert!(
                level.destructible_count() > 0,
                "level {i} should have destructible bricks (galaxy={galaxy}, biome={biome})"
            );
        }
    }

    #[test]
    fn health_gradient_higher_at_top() {
        let master = MasterSeed(50);
        let biome_seed = master.galaxy(0).biome(0);
        let params = BiomeGenerator::generate(biome_seed);
        let mut rng = biome_seed.rng();
        let level = compose_level(&params, &mut rng);

        let top_hp: f32 = level
            .bricks
            .iter()
            .filter(|b| b.row == 0 && b.is_destructible())
            .map(|b| b.health as f32)
            .sum::<f32>()
            / level.bricks.iter().filter(|b| b.row == 0).count().max(1) as f32;

        let bot_hp: f32 = level
            .bricks
            .iter()
            .filter(|b| b.row == 7 && b.is_destructible())
            .map(|b| b.health as f32)
            .sum::<f32>()
            / level.bricks.iter().filter(|b| b.row == 7).count().max(1) as f32;

        if top_hp > 0.0 && bot_hp > 0.0 {
            assert!(
                top_hp >= bot_hp,
                "top rows should have >= health: top={top_hp}, bot={bot_hp}"
            );
        }
    }

    #[test]
    fn powerups_present() {
        let master = MasterSeed(42);
        let biome_seed = master.galaxy(0).biome(0);
        let params = BiomeGenerator::generate(biome_seed);
        let mut rng = biome_seed.rng();
        let level = compose_level(&params, &mut rng);
        let powerup_count = level
            .bricks
            .iter()
            .filter(|b| b.powerup_chance > 0.0)
            .count();
        assert!(powerup_count > 0, "should have at least one power-up brick");
    }
}
