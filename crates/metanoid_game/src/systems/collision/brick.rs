use bevy::prelude::*;
use avian2d::prelude::*;
use metanoid_core::components::ball::{Ball, Fireball};
use metanoid_core::components::brick::{Brick, BrickType};
use metanoid_core::events::{BrickDestroyedEvent, BrickHitEvent};

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
            } else if balls.get(event.collider2).is_ok() && bricks.get(event.collider1).is_ok()
            {
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

        let brick_pos = transform.translation.truncate();
        let brick_type = brick.brick_type;
        let ball_speed = velocity.0.length();

        if brick.brick_type == BrickType::Invincible {
            if fireball.is_some() {
                commands.trigger(BrickHitEvent { brick: brick_entity, ball_speed });
                commands.trigger(BrickDestroyedEvent {
                    brick: brick_entity,
                    position: brick_pos,
                    brick_type,
                });
                commands.entity(brick_entity).despawn();
            } else {
                commands.trigger(BrickHitEvent { brick: brick_entity, ball_speed });
            }
            continue;
        }

        brick.health = brick.health.saturating_sub(1);
        commands.trigger(BrickHitEvent { brick: brick_entity, ball_speed });

        if brick.health == 0 {
            commands.trigger(BrickDestroyedEvent {
                brick: brick_entity,
                position: brick_pos,
                brick_type,
            });
            commands.entity(brick_entity).despawn();
        }
    }
}
