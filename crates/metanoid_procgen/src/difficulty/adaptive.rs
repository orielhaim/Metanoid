#[derive(Debug, Clone)]
pub struct PlayerMetrics {
    pub avg_combo: f32,
    pub reaction_time_ms: f32,
    pub lives_lost_recent: u32,
    pub bricks_per_second: f32,
}

impl Default for PlayerMetrics {
    fn default() -> Self {
        Self {
            avg_combo: 3.0,
            reaction_time_ms: 300.0,
            lives_lost_recent: 0,
            bricks_per_second: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdaptiveModifier {
    pub speed_factor: f32,
    pub starting_lives_bonus: i32,
    pub powerup_interval_factor: f32,
}

impl Default for AdaptiveModifier {
    fn default() -> Self {
        Self {
            speed_factor: 1.0,
            starting_lives_bonus: 0,
            powerup_interval_factor: 1.0,
        }
    }
}

pub fn compute_adaptive(metrics: &PlayerMetrics) -> AdaptiveModifier {
    let skill = estimate_skill(metrics);
    let center = 0.5;
    let deviation = skill - center;

    AdaptiveModifier {
        speed_factor: (1.0 + deviation * 0.3).clamp(0.7, 1.3),
        starting_lives_bonus: if deviation < -0.2 { 1 } else { 0 },
        powerup_interval_factor: (1.0 + deviation * 0.4).clamp(0.6, 1.5),
    }
}

fn estimate_skill(m: &PlayerMetrics) -> f32 {
    let combo_score = (m.avg_combo / 10.0).clamp(0.0, 1.0);
    let reaction_score = (1.0 - (m.reaction_time_ms - 150.0) / 350.0).clamp(0.0, 1.0);
    let survival_score = (1.0 - m.lives_lost_recent as f32 / 5.0).clamp(0.0, 1.0);
    let speed_score = (m.bricks_per_second / 3.0).clamp(0.0, 1.0);

    combo_score * 0.3 + reaction_score * 0.3 + survival_score * 0.2 + speed_score * 0.2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn average_player_neutral() {
        let m = PlayerMetrics::default();
        let a = compute_adaptive(&m);
        assert!((a.speed_factor - 1.0).abs() < 0.15);
        assert!((a.powerup_interval_factor - 1.0).abs() < 0.25);
    }

    #[test]
    fn skilled_player_harder() {
        let m = PlayerMetrics {
            avg_combo: 8.0,
            reaction_time_ms: 150.0,
            lives_lost_recent: 0,
            bricks_per_second: 2.5,
        };
        let a = compute_adaptive(&m);
        assert!(a.speed_factor > 1.0);
        assert!(a.powerup_interval_factor > 1.0);
        assert_eq!(a.starting_lives_bonus, 0);
    }

    #[test]
    fn struggling_player_easier() {
        let m = PlayerMetrics {
            avg_combo: 1.0,
            reaction_time_ms: 500.0,
            lives_lost_recent: 3,
            bricks_per_second: 0.3,
        };
        let a = compute_adaptive(&m);
        assert!(a.speed_factor < 1.0);
        assert!(a.starting_lives_bonus >= 1);
        assert!(a.powerup_interval_factor < 1.0);
    }
}
