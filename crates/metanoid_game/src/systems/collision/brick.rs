use avian2d::prelude::*;
use bevy::prelude::*;
use metanoid_core::components::ball::{Ball, Fireball};
use metanoid_core::components::brick::{Brick, BrickType};
use metanoid_core::events::BrickHitEvent;

use super::super::level_clear::destroy_brick;

pub fn ball_brick_collision(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut bricks: Query<(&mut Brick, &Transform)>,
    balls: Query<(&Ball, Option<&Fireball>, &LinearVelocity, &Transform)>,
) {
    for event in collision_reader.read() {
        let (ball_entity, brick_entity) =
            if balls.get(event.collider1).is_ok() && bricks.get(event.collider2).is_ok() {
                (event.collider1, event.collider2)
            } else if balls.get(event.collider2).is_ok() && bricks.get(event.collider1).is_ok() {
                (event.collider2, event.collider1)
            } else {
                continue;
            };

        let Ok((ball, fireball, velocity, ball_tf)) = balls.get(ball_entity) else {
            continue;
        };
        let Ok((mut brick, transform)) = bricks.get_mut(brick_entity) else {
            continue;
        };

        // Already destroyed this frame
        if brick.health == 0 {
            continue;
        }

        let brick_pos = transform.translation.truncate();
        let impact_pos = ball_tf.translation.truncate();
        let ball_speed = velocity.0.length();
        // Impact severity from how fast the ball was moving relative to its
        // base speed. Fast balls carve deep, wide damage; slow brushes are
        // lighter. The curve is biased up so even a normal-speed hit leaves a
        // clearly visible mark.
        let ratio = ball_speed / ball.speed.max(1.0);
        let t = ((ratio - 0.5) / 0.9).clamp(0.0, 1.0);
        let severity = 0.35 + 0.65 * t * t * (3.0 - 2.0 * t);

        if brick.brick_type == BrickType::Invincible {
            commands.trigger(BrickHitEvent {
                brick: brick_entity,
                ball_speed,
                position: impact_pos,
                severity,
            });
            if fireball.is_some() {
                // destroy_brick requires health > 0 for the destroy event
                destroy_brick(&mut commands, brick_entity, &mut brick, brick_pos);
            }
            continue;
        }

        commands.trigger(BrickHitEvent {
            brick: brick_entity,
            ball_speed,
            position: impact_pos,
            severity,
        });

        // Keep health > 0 until destroy_brick so the destroy event fires once
        if brick.health <= 1 {
            destroy_brick(&mut commands, brick_entity, &mut brick, brick_pos);
        } else {
            brick.health -= 1;
        }
    }
}
