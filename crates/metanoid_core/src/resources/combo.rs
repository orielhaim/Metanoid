use bevy::prelude::*;
use std::time::Duration;

#[derive(Resource)]
pub struct ComboCounter {
    pub count: u32,
    pub timer: Timer,
    pub multiplier: f32,
}

impl Default for ComboCounter {
    fn default() -> Self {
        Self {
            count: 0,
            timer: Timer::from_seconds(2.0, TimerMode::Once),
            multiplier: 1.0,
        }
    }
}

impl ComboCounter {
    pub fn hit(&mut self) {
        self.count += 1;
        self.timer.reset();
        self.multiplier = 1.0 + (self.count as f32 * 0.1).min(3.0);
    }

    pub fn tick(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if self.timer.is_finished() {
            self.count = 0;
            self.multiplier = 1.0;
        }
    }
}
