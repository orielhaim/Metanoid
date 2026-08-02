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
    balls: Query<(&Ball, Option<&Fireball>, &LinearVelocity)>,
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

        let Ok((_ball, fireball, velocity)) = balls.get(ball_entity) else {
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
        let ball_speed = velocity.0.length();

        if brick.brick_type == BrickType::Invincible {
            commands.trigger(BrickHitEvent {
                brick: brick_entity,
                ball_speed,
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
        });

        // Keep health > 0 until destroy_brick so the destroy event fires once
        if brick.health <= 1 {
            destroy_brick(&mut commands, brick_entity, &mut brick, brick_pos);
        } else {
            brick.health -= 1;
        }
    }
}
