use bevy::prelude::*;
use metanoid_core::components::brick::{Brick, BrickType};
use metanoid_core::events::BrickRegenEvent;

/// Regenerates health on regenerating bricks after a delay.
pub fn update_regen_bricks(
    time: Res<Time>,
    mut commands: Commands,
    mut bricks: Query<(Entity, &mut Brick)>,
) {
    for (entity, mut brick) in &mut bricks {
        if brick.brick_type != BrickType::Regenerating {
            continue;
        }
        if brick.health == 0 {
            continue;
        }
        if brick.health < brick.max_health {
            brick.regen_timer -= time.delta_secs();
            if brick.regen_timer <= 0.0 {
                brick.health = brick.max_health;
                brick.regen_timer = 3.5;
                commands.trigger(BrickRegenEvent { brick: entity });
            }
        }
    }
}
