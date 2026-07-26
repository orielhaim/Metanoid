use bevy::prelude::*;
use metanoid_core::events::BrickDestroyedEvent;
use metanoid_core::resources::combo::ComboCounter;
use metanoid_core::resources::game_state::GameState;

pub fn update_combo(
    time: Res<Time>,
    mut combo: ResMut<ComboCounter>,
) {
    combo.tick(time.delta());
}

pub fn on_brick_destroyed_increment_combo(
    _trigger: On<BrickDestroyedEvent>,
    mut combo: ResMut<ComboCounter>,
    mut game_state: Option<ResMut<GameState>>,
) {
    combo.hit();

    let base_score = 10u64;
    let points = (base_score as f32 * combo.multiplier) as u64;

    if let Some(ref mut state) = game_state {
        state.score += points;
    }

    if combo.count > 0 && combo.count % 5 == 0 {
        info!("Combo x{}! Multiplier: {:.1}x", combo.count, combo.multiplier);
    }
}
