use bevy::prelude::*;
use metanoid_core::components::brick::{Brick, BrickType};

/// Moves bricks horizontally in a sine wave pattern.
pub fn update_moving_bricks(
    time: Res<Time>,
    mut bricks: Query<(&mut Brick, &mut Transform)>,
) {
    for (brick, mut transform) in &mut bricks {
        if brick.brick_type != BrickType::Moving || brick.move_range <= 0.0 {
            continue;
        }
        let t = time.elapsed_secs() * brick.move_speed;
        let offset = t.sin() * brick.move_range;
        transform.translation.x = brick.move_origin_x + offset;
    }
}

/// Regenerates health on regenerating bricks after a delay.
pub fn update_regen_bricks(
    time: Res<Time>,
    mut bricks: Query<&mut Brick>,
) {
    for mut brick in &mut bricks {
        if brick.brick_type != BrickType::Regenerating {
            continue;
        }
        if brick.health < brick.max_health {
            brick.regen_timer -= time.delta_secs();
            if brick.regen_timer <= 0.0 {
                brick.health = brick.max_health;
                brick.regen_timer = 4.0;
            }
        }
    }
}
