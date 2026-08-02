use avian2d::prelude::*;
use bevy::prelude::*;
use metanoid_core::components::ball::{
    Ball, BallEffect, BallEffectKind, BrickThru, Fireball, SplitBall,
};
use metanoid_core::components::powerup::PowerUpKind;
use metanoid_core::constants::*;
use metanoid_core::events::{BallSpeedChangeEvent, BallSplitEvent, PowerUpCollectedEvent};

use crate::systems::level_spawner::LevelEntity;

const EFFECT_DURATION: f32 = 15.0;

pub fn apply_ball_effect(
    trigger: On<PowerUpCollectedEvent>,
    mut commands: Commands,
    balls: Query<Entity, With<Ball>>,
    mut ball_query: Query<&mut Ball>,
    mut colliders: Query<&mut Collider, With<Ball>>,
    transforms: Query<&Transform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mesh_handles: Query<&Mesh2d, With<Ball>>,
) {
    let kind = trigger.kind;

    if kind == PowerUpKind::SplitBall {
        commands.trigger(BallSplitEvent);
    }
    if matches!(kind, PowerUpKind::FastBall | PowerUpKind::SlowBall) {
        commands.trigger(BallSpeedChangeEvent);
    }

    let ball_effects = match kind {
        PowerUpKind::Fireball => vec![BallEffectKind::Fireball],
        PowerUpKind::MegaBall => vec![BallEffectKind::MegaBall],
        PowerUpKind::SplitBall => vec![],
        PowerUpKind::FastBall => vec![BallEffectKind::FastBall],
        PowerUpKind::SlowBall => vec![BallEffectKind::SlowBall],
        PowerUpKind::ShrinkPaddle => vec![BallEffectKind::ShrinkBall],
        _ => vec![],
    };

    for effect_kind in ball_effects {
        for ball_entity in &balls {
            commands
                .entity(ball_entity)
                .insert(BallEffect::new(effect_kind, EFFECT_DURATION));

            match effect_kind {
                BallEffectKind::Fireball => {
                    commands.entity(ball_entity).insert(Fireball);
                }
                BallEffectKind::BrickThru => {
                    commands.entity(ball_entity).insert(BrickThru);
                }
                BallEffectKind::MegaBall => {
                    if let Ok(mut ball) = ball_query.get_mut(ball_entity) {
                        ball.radius = BALL_RADIUS * 2.0;
                    }
                    if let Ok(mut collider) = colliders.get_mut(ball_entity) {
                        *collider = Collider::circle(BALL_RADIUS * 2.0);
                    }
                    if let Ok(mesh2d) = mesh_handles.get(ball_entity) {
                        let _ = meshes.insert(&mesh2d.0, Circle::new(BALL_RADIUS * 2.0).into());
                    }
                }
                BallEffectKind::ShrinkBall => {
                    if let Ok(mut ball) = ball_query.get_mut(ball_entity) {
                        ball.radius = BALL_RADIUS * 0.5;
                    }
                    if let Ok(mut collider) = colliders.get_mut(ball_entity) {
                        *collider = Collider::circle(BALL_RADIUS * 0.5);
                    }
                    if let Ok(mesh2d) = mesh_handles.get(ball_entity) {
                        let _ = meshes.insert(&mesh2d.0, Circle::new(BALL_RADIUS * 0.5).into());
                    }
                }
                BallEffectKind::FastBall => {
                    if let Ok(mut ball) = ball_query.get_mut(ball_entity) {
                        ball.speed = BALL_SPEED * 1.5;
                    }
                }
                BallEffectKind::SlowBall => {
                    if let Ok(mut ball) = ball_query.get_mut(ball_entity) {
                        ball.speed = BALL_SPEED * 0.6;
                    }
                }
                _ => {}
            }
        }
    }

    if kind == PowerUpKind::SplitBall {
        for ball_entity in &balls {
            let Ok(transform) = transforms.get(ball_entity) else {
                continue;
            };
            let pos = transform.translation;

            let split_mesh = meshes.add(Circle::new(BALL_RADIUS));
            commands.spawn((
                Ball {
                    speed: BALL_SPEED,
                    radius: BALL_RADIUS,
                    stuck: false,
                    spin: 0.0,
                },
                SplitBall,
                LevelEntity,
                RigidBody::Dynamic,
                Collider::circle(BALL_RADIUS),
                Transform::from_xyz(pos.x, pos.y, 0.0),
                LinearVelocity(Vec2::new(200.0, BALL_LAUNCH_SPEED)),
                Restitution::new(1.0),
                super::super::physics_layers::layers_ball(),
                CollisionEventsEnabled,
                Mesh2d(split_mesh),
                MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
            ));
        }
    }
}

pub fn tick_ball_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut balls: Query<(Entity, &mut Ball, &mut BallEffect)>,
    mut colliders: Query<&mut Collider, With<Ball>>,
    mesh_handles: Query<&Mesh2d, With<Ball>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut ball, mut effect) in &mut balls {
        effect.timer.tick(time.delta());

        if effect.timer.is_finished() {
            match effect.kind {
                BallEffectKind::MegaBall | BallEffectKind::ShrinkBall => {
                    ball.radius = BALL_RADIUS;
                    if let Ok(mut collider) = colliders.get_mut(entity) {
                        *collider = Collider::circle(BALL_RADIUS);
                    }
                    if let Ok(mesh2d) = mesh_handles.get(entity) {
                        let _ = meshes.insert(&mesh2d.0, Circle::new(BALL_RADIUS).into());
                    }
                }
                BallEffectKind::FastBall | BallEffectKind::SlowBall => {
                    ball.speed = BALL_SPEED;
                }
                _ => {}
            }

            commands.entity(entity).remove::<BallEffect>();
            commands.entity(entity).remove::<Fireball>();
            commands.entity(entity).remove::<BrickThru>();
        }
    }
}
