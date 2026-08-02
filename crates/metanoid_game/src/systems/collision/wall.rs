//! Wall bounce detection — fires WallHitEvent for light SFX.

use avian2d::prelude::*;
use bevy::prelude::*;
use metanoid_core::components::ball::Ball;
use metanoid_core::events::WallHitEvent;

use super::super::arena::Wall;

pub fn ball_wall_collision(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    balls: Query<Entity, With<Ball>>,
    walls: Query<Entity, With<Wall>>,
) {
    for event in collision_reader.read() {
        let wall_entity =
            if balls.get(event.collider1).is_ok() && walls.get(event.collider2).is_ok() {
                event.collider2
            } else if balls.get(event.collider2).is_ok() && walls.get(event.collider1).is_ok() {
                event.collider1
            } else {
                continue;
            };

        commands.trigger(WallHitEvent { wall: wall_entity });
    }
}
