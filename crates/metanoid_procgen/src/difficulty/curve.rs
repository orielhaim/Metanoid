use crate::biome::parameters::BiomeParams;

#[derive(Debug, Clone)]
pub struct DifficultyParams {
    pub ball_speed_mult: f32,
    pub avg_brick_health: f32,
    pub brick_count_mult: f32,
    pub powerup_freq_mult: f32,
    pub negative_powerup_ratio: f32,
    pub special_density_mult: f32,
    pub moving_brick_count: usize,
}

impl Default for DifficultyParams {
    fn default() -> Self {
        Self {
            ball_speed_mult: 1.0,
            avg_brick_health: 1.0,
            brick_count_mult: 1.0,
            powerup_freq_mult: 1.0,
            negative_powerup_ratio: 0.15,
            special_density_mult: 1.0,
            moving_brick_count: 0,
        }
    }
}

pub fn compute_difficulty(
    biome_params: &BiomeParams,
    level_in_biome: u64,
    levels_per_biome: u64,
    galaxy_base_difficulty: f32,
) -> DifficultyParams {
    let progress = if levels_per_biome > 1 {
        level_in_biome as f32 / (levels_per_biome - 1) as f32
    } else {
        1.0
    };

    let curve = ease_in_out(progress);
    let difficulty = galaxy_base_difficulty + curve * (1.0 - galaxy_base_difficulty);

    DifficultyParams {
        ball_speed_mult: 0.8 + difficulty * 0.6,
        avg_brick_health: 1.0 + difficulty * 2.0 + biome_params.density * 1.0,
        brick_count_mult: 0.6 + biome_params.density * 0.6 + curve * 0.3,
        powerup_freq_mult: 1.2 - difficulty * 0.4,
        negative_powerup_ratio: 0.1 + difficulty * 0.2,
        special_density_mult: 0.5 + difficulty * 1.5,
        moving_brick_count: (difficulty * 3.0 * biome_params.chaos) as usize,
    }
}

fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mid_params() -> BiomeParams {
        BiomeParams {
            temperature: 0.5,
            density: 0.5,
            chaos: 0.5,
            energy: 0.5,
            weirdness: 0.5,
        }
    }

    #[test]
    fn difficulty_increases_over_biome() {
        let p = mid_params();
        let early = compute_difficulty(&p, 0, 12, 0.3);
        let late = compute_difficulty(&p, 10, 12, 0.3);
        assert!(late.ball_speed_mult > early.ball_speed_mult);
        assert!(late.avg_brick_health > early.avg_brick_health);
    }

    #[test]
    fn powerup_freq_decreases() {
        let p = mid_params();
        let early = compute_difficulty(&p, 0, 12, 0.3);
        let late = compute_difficulty(&p, 11, 12, 0.3);
        assert!(early.powerup_freq_mult > late.powerup_freq_mult);
    }

    #[test]
    fn galaxy_base_affects_difficulty() {
        let p = mid_params();
        let easy = compute_difficulty(&p, 5, 12, 0.2);
        let hard = compute_difficulty(&p, 5, 12, 0.8);
        assert!(hard.ball_speed_mult > easy.ball_speed_mult);
    }
}
