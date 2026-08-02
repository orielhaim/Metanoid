use bevy::prelude::*;
use metanoid_core::components::powerup::PowerUpKind;
use metanoid_core::events::PowerUpCollectedEvent;

#[derive(Resource)]
pub struct BiomeLighting {
    pub _base_ambient: f32,
    pub _ball_intensity: f32,
}

impl Default for BiomeLighting {
    fn default() -> Self {
        Self {
            _base_ambient: 0.95,
            _ball_intensity: 0.5,
        }
    }
}

#[derive(Resource)]
pub struct BlackoutState {
    pub active: bool,
    pub timer: Timer,
}

impl Default for BlackoutState {
    fn default() -> Self {
        Self {
            active: false,
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        }
    }
}

pub fn apply_biome_lighting(_biome_lighting: Res<BiomeLighting>, _blackout: Res<BlackoutState>) {
    // Reserved for future 2D lighting integration
}

pub fn tick_blackout(time: Res<Time>, mut blackout: ResMut<BlackoutState>) {
    if blackout.active {
        blackout.timer.tick(time.delta());
        if blackout.timer.is_finished() {
            blackout.active = false;
        }
    }
}

pub fn on_blackout_collected(
    trigger: On<PowerUpCollectedEvent>,
    mut blackout: ResMut<BlackoutState>,
) {
    if trigger.kind == PowerUpKind::Blackout {
        blackout.active = true;
        blackout.timer.reset();
        info!("Blackout activated!");
    }
}
