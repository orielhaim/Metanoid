use bevy::prelude::*;
use avian2d::prelude::*;
use metanoid_core::components::ball::Ball;
use metanoid_core::components::paddle::{Paddle, PaddleEffect, PaddleEffectKind, LaserPaddle, ShieldBarrier};
use metanoid_core::components::powerup::PowerUpKind;
use metanoid_core::constants::*;
use metanoid_core::events::{LaserFireEvent, PowerUpCollectedEvent, ShieldActivateEvent};

use crate::systems::level_spawner::LevelEntity;

const EFFECT_DURATION: f32 = 15.0;

fn resize_paddle_visuals(
    entity: Entity,
    new_size: Vec2,
    colliders: &mut Query<&mut Collider, With<Paddle>>,
    mesh_handles: &Query<&Mesh2d, With<Paddle>>,
    meshes: &mut Assets<Mesh>,
) {
    if let Ok(mut collider) = colliders.get_mut(entity) {
        *collider = Collider::rectangle(new_size.x, new_size.y);
    }
    if let Ok(mesh2d) = mesh_handles.get(entity) {
        let _ = meshes.insert(&mesh2d.0, Rectangle::new(new_size.x, new_size.y).into());
    }
}

pub fn apply_paddle_effect(
    trigger: On<PowerUpCollectedEvent>,
    mut commands: Commands,
    paddles: Query<Entity, With<Paddle>>,
    mut paddle_query: Query<&mut Paddle>,
    mut colliders: Query<&mut Collider, With<Paddle>>,
    mesh_handles: Query<&Mesh2d, With<Paddle>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let kind = trigger.kind;

    match kind {
        PowerUpKind::ExpandPaddle => {
            for paddle_entity in &paddles {
                let new_size = Vec2::new(PADDLE_WIDTH * 1.5, PADDLE_HEIGHT);
                commands.entity(paddle_entity)
                    .insert(PaddleEffect::new(PaddleEffectKind::Expand, EFFECT_DURATION));
                if let Ok(mut paddle) = paddle_query.get_mut(paddle_entity) {
                    paddle.size = new_size;
                }
                resize_paddle_visuals(paddle_entity, new_size, &mut colliders, &mesh_handles, &mut meshes);
            }
        }
        PowerUpKind::ShrinkPaddle => {
            for paddle_entity in &paddles {
                let new_size = Vec2::new(PADDLE_WIDTH * 0.6, PADDLE_HEIGHT);
                commands.entity(paddle_entity)
                    .insert(PaddleEffect::new(PaddleEffectKind::Shrink, EFFECT_DURATION));
                if let Ok(mut paddle) = paddle_query.get_mut(paddle_entity) {
                    paddle.size = new_size;
                }
                resize_paddle_visuals(paddle_entity, new_size, &mut colliders, &mesh_handles, &mut meshes);
            }
        }
        PowerUpKind::LaserPaddle => {
            for paddle_entity in &paddles {
                commands.entity(paddle_entity)
                    .insert(PaddleEffect::new(PaddleEffectKind::Laser, EFFECT_DURATION))
                    .insert(LaserPaddle {
                        cooldown: Timer::from_seconds(0.3, TimerMode::Repeating),
                    });
            }
        }
        PowerUpKind::GrabPaddle => {
            for paddle_entity in &paddles {
                commands.entity(paddle_entity)
                    .insert(PaddleEffect::new(PaddleEffectKind::Grab, EFFECT_DURATION));
            }
        }
        PowerUpKind::Shield => {
            for paddle_entity in &paddles {
                commands.entity(paddle_entity)
                    .insert(PaddleEffect::new(PaddleEffectKind::Shield, EFFECT_DURATION));
            }
            let barrier_mesh = meshes.add(Rectangle::new(ARENA_WIDTH - WALL_THICKNESS * 2.0, 8.0));
            let barrier_mat = materials.add(Color::srgb(0.0, 0.8, 1.0));
            commands.spawn((
                ShieldBarrier,
                LevelEntity,
                RigidBody::Static,
                Collider::rectangle(ARENA_WIDTH - WALL_THICKNESS * 2.0, 8.0),
                Transform::from_xyz(0.0, PADDLE_Y - 30.0, 0.0),
                CollisionLayers::DEFAULT,
                CollisionEventsEnabled,
                Mesh2d(barrier_mesh),
                MeshMaterial2d(barrier_mat),
            ));
            commands.trigger(ShieldActivateEvent);
        }
        _ => {}
    }
}

pub fn tick_paddle_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut paddles: Query<(Entity, &mut Paddle, &mut PaddleEffect)>,
    mut colliders: Query<&mut Collider, With<Paddle>>,
    mesh_handles: Query<&Mesh2d, With<Paddle>>,
    mut meshes: ResMut<Assets<Mesh>>,
    shields: Query<Entity, With<ShieldBarrier>>,
) {
    for (entity, mut paddle, mut effect) in &mut paddles {
        effect.timer.tick(time.delta());

        if effect.timer.is_finished() {
            match effect.kind {
                PaddleEffectKind::Expand | PaddleEffectKind::Shrink => {
                    paddle.size = Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT);
                    resize_paddle_visuals(entity, paddle.size, &mut colliders, &mesh_handles, &mut meshes);
                }
                PaddleEffectKind::Laser => {
                    commands.entity(entity).remove::<LaserPaddle>();
                }
                PaddleEffectKind::Shield => {
                    for shield_entity in &shields {
                        commands.entity(shield_entity).despawn();
                    }
                }
                _ => {}
            }
            commands.entity(entity).remove::<PaddleEffect>();
        }
    }
}

pub fn fire_lasers(
    mut commands: Commands,
    time: Res<Time>,
    mut lasers: Query<(&Transform, &mut LaserPaddle)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (transform, mut laser) in &mut lasers {
        laser.cooldown.tick(time.delta());

        if laser.cooldown.just_finished() {
            let pos = transform.translation;
            let laser_mesh = meshes.add(Rectangle::new(4.0, 16.0));
            let laser_mat = materials.add(Color::srgb(1.0, 0.2, 0.2));
            commands.trigger(LaserFireEvent);

            commands.spawn((
                LaserProjectile,
                LevelEntity,
                RigidBody::Kinematic,
                Collider::rectangle(4.0, 16.0),
                Sensor,
                CollisionLayers::DEFAULT,
                CollisionEventsEnabled,
                Transform::from_xyz(pos.x - 20.0, pos.y + 20.0, 0.0),
                LinearVelocity(Vec2::new(0.0, 600.0)),
                Mesh2d(laser_mesh.clone()),
                MeshMaterial2d(laser_mat.clone()),
            ));
            commands.spawn((
                LaserProjectile,
                LevelEntity,
                RigidBody::Kinematic,
                Collider::rectangle(4.0, 16.0),
                Sensor,
                CollisionLayers::DEFAULT,
                CollisionEventsEnabled,
                Transform::from_xyz(pos.x + 20.0, pos.y + 20.0, 0.0),
                LinearVelocity(Vec2::new(0.0, 600.0)),
                Mesh2d(laser_mesh),
                MeshMaterial2d(laser_mat),
            ));
        }
    }
}

#[derive(Component)]
pub struct LaserProjectile;

pub fn despawn_offscreen_lasers(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<LaserProjectile>>,
) {
    for (entity, transform) in &query {
        if transform.translation.y > ARENA_HEIGHT / 2.0 + 50.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn laser_hit_bricks(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    lasers: Query<&LaserProjectile>,
    mut bricks: Query<(&mut metanoid_core::components::brick::Brick, &Transform)>,
) {
    for event in collision_reader.read() {
        let (laser_entity, brick_entity) =
            if lasers.get(event.collider1).is_ok() && bricks.get(event.collider2).is_ok() {
                (event.collider1, event.collider2)
            } else if lasers.get(event.collider2).is_ok() && bricks.get(event.collider1).is_ok()
            {
                (event.collider2, event.collider1)
            } else {
                continue;
            };

        let Ok((mut brick, transform)) = bricks.get_mut(brick_entity) else {
            continue;
        };

        let brick_pos = transform.translation.truncate();
        let brick_type = brick.brick_type;

        brick.health = brick.health.saturating_sub(1);
        commands.entity(laser_entity).despawn();

        if brick.health == 0 {
            commands.trigger(metanoid_core::events::BrickDestroyedEvent {
                brick: brick_entity,
                position: brick_pos,
                brick_type,
            });
            commands.entity(brick_entity).despawn();
        }
    }
}

pub fn ball_shield_collision(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    balls: Query<&Ball>,
    shields: Query<&ShieldBarrier>,
) {
    for event in collision_reader.read() {
        let (_ball_entity, shield_entity) =
            if balls.get(event.collider1).is_ok() && shields.get(event.collider2).is_ok() {
                (event.collider1, event.collider2)
            } else if balls.get(event.collider2).is_ok() && shields.get(event.collider1).is_ok()
            {
                (event.collider2, event.collider1)
            } else {
                continue;
            };

        commands.trigger(metanoid_core::events::ShieldHitEvent);
        commands.entity(shield_entity).despawn();
    }
}
