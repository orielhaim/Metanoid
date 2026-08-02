use bevy::prelude::*;
use std::time::Duration;

/// Vulnerability-chain combo: grows on brick *hits*, not only breaks.
/// Breaking a brick scores more under the current multiplier.
/// Resets when the ball returns to the paddle, a life is lost, or the chain timer expires.
#[derive(Resource)]
pub struct ComboCounter {
    /// Consecutive vulnerability hits (brick touches while chain is live).
    pub count: u32,
    pub timer: Timer,
    pub multiplier: f32,
    /// True while the ball is "in the air" on a chain (until paddle / miss).
    pub chain_live: bool,
}

impl Default for ComboCounter {
    fn default() -> Self {
        Self {
            count: 0,
            // Grace window between brick contacts (forgives slow multi-hit)
            timer: Timer::from_seconds(2.4, TimerMode::Once),
            multiplier: 1.0,
            chain_live: false,
        }
    }
}

impl ComboCounter {
    /// Brick was hit (damage applied or invincible clang) — builds vulnerability.
    pub fn register_hit(&mut self) {
        self.count = self.count.saturating_add(1);
        self.chain_live = true;
        self.timer.reset();
        self.recompute_multiplier();
    }

    /// Ball returned to paddle safely — chain ends (no penalty beyond reset).
    pub fn on_paddle_return(&mut self) {
        self.count = 0;
        self.multiplier = 1.0;
        self.chain_live = false;
        self.timer.reset();
    }

    /// Life lost / death — hard reset.
    pub fn on_life_lost(&mut self) {
        self.on_paddle_return();
    }

    fn recompute_multiplier(&mut self) {
        // Softer curve: vulnerability ramps; breaks cash it out
        // 1.0 at 0, ~1.5 at 5, ~2.5 at 12, cap 4.0x
        self.multiplier = 1.0 + (self.count as f32 * 0.1).min(3.0);
    }

    pub fn tick(&mut self, delta: Duration) {
        if !self.chain_live || self.count == 0 {
            return;
        }
        self.timer.tick(delta);
        if self.timer.is_finished() {
            self.count = 0;
            self.multiplier = 1.0;
            self.chain_live = false;
        }
    }

    /// Score multiplier when a brick is destroyed (break pays more than a chip).
    pub fn break_multiplier(&self) -> f32 {
        // Breaks get full chain multiplier + small destroy bonus scaling
        (self.multiplier * 1.35).min(5.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hits_build_combo_without_break() {
        let mut c = ComboCounter::default();
        c.register_hit();
        c.register_hit();
        c.register_hit();
        assert_eq!(c.count, 3);
        assert!(c.multiplier > 1.0);
        assert!(c.break_multiplier() > c.multiplier);
    }

    #[test]
    fn paddle_return_resets() {
        let mut c = ComboCounter::default();
        c.register_hit();
        c.register_hit();
        c.on_paddle_return();
        assert_eq!(c.count, 0);
        assert!((c.multiplier - 1.0).abs() < 1e-5);
    }
}
