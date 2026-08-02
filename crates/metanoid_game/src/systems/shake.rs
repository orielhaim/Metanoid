use bevy::prelude::*;
use bevy_trauma_shake::prelude::*;
use metanoid_core::components::brick::BrickType;
use metanoid_core::events::BrickDestroyedEvent;
use metanoid_core::resources::combo::ComboCounter;
use metanoid_core::settings::GameSettings;

pub fn on_brick_destroyed_shake(
    trigger: On<BrickDestroyedEvent>,
    mut commands: Commands,
    combo: Res<ComboCounter>,
    settings: Res<GameSettings>,
) {
    let base_trauma = match trigger.brick_type {
        BrickType::Explosive => 0.5,
        BrickType::MultiHit => 0.08,
        _ => 0.05,
    };

    let combo_bonus = (combo.count as f32 * 0.005).min(0.15);
    let trauma = (base_trauma + combo_bonus).min(0.6) * settings.trauma_scale();

    if trauma > 0.02 {
        commands.add_trauma(trauma);
    }
}
