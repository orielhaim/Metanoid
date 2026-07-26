use bevy::prelude::*;
use metanoid_core::resources::combo::ComboCounter;

use super::lighting::{BiomeLighting, BlackoutState};
use super::powerup::collector::TimeSlowState;
use super::powerup::spawner::PowerUpState;

/// Reset all game effect/state resources to defaults when restarting.
pub fn reset_game_effects(
    mut combo: ResMut<ComboCounter>,
    mut time_slow: ResMut<TimeSlowState>,
    mut blackout: ResMut<BlackoutState>,
    mut biome_lighting: ResMut<BiomeLighting>,
    mut powerup_state: ResMut<PowerUpState>,
) {
    *combo = ComboCounter::default();
    *time_slow = TimeSlowState::default();
    *blackout = BlackoutState::default();
    *biome_lighting = BiomeLighting::default();
    *powerup_state = PowerUpState::default();
}
