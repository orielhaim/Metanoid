use bevy::prelude::*;
use avian2d::prelude::*;
use metanoid_core::components::ball::Ball;
use metanoid_core::components::paddle::Paddle;
use metanoid_core::constants::{BALL_LAUNCH_SPEED, BALL_RADIUS, BALL_SPEED, PADDLE_HEIGHT, PADDLE_Y};
use metanoid_core::events::LifeLostEvent;

#[derive(Component)]
pub struct DevBall;

pub fn ball_launch(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut balls: Query<(Entity, &mut Ball, &Transform)>,
    paddle: Query<&Transform, (With<Paddle>, Without<Ball>)>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    for (entity, mut ball, ball_transform) in &mut balls {
        if ball.stuck {
            let Ok(paddle_transform) = paddle.single() else {
                continue;
            };
            let offset = ball_transform.translation.x - paddle_transform.translation.x;
            let horizontal = offset * 2.0;

            commands.entity(entity).insert(LinearVelocity(Vec2::new(
                horizontal,
                BALL_LAUNCH_SPEED,
            )));
            ball.stuck = false;
        }
    }
}

pub fn ball_speed_clamp(mut query: Query<(&Ball, &mut LinearVelocity)>) {
    for (ball, mut velocity) in &mut query {
        let speed = velocity.0.length();
        if speed < 1.0 {
            continue;
        }
        // Only enforce minimum speed to prevent stalling — don't cap maximum
        let min_speed = ball.speed * 0.5;
        if speed < min_speed {
            velocity.0 = velocity.0.normalize() * min_speed;
        }
    }
}

/// Prevents the ball from bouncing perfectly horizontal between walls.
/// If the ball's vertical component is too small, nudge it slightly.
pub fn anti_stuck_ball(mut query: Query<&mut LinearVelocity, With<Ball>>) {
    for mut velocity in &mut query {
        let vel = velocity.0;
        let speed = vel.length();
        if speed < 1.0 {
            continue;
        }
        let abs_vy = vel.y.abs();
        let min_vy = speed * 0.12;
        if abs_vy < min_vy {
            let sign = if vel.y >= 0.0 { 1.0 } else { -1.0 };
            let vx_sq = vel.x.abs().powi(2);
            let new_vy = (speed * speed - vx_sq).max(min_vy * min_vy).sqrt() * sign;
            velocity.0 = Vec2::new(vel.x, new_vy);
        }
    }
}

pub fn ball_escape(
    mut commands: Commands,
    query: Query<(Entity, &Ball, &Transform)>,
    dev_balls: Query<&DevBall>,
) {
    for (entity, _ball, transform) in &query {
        if transform.translation.y < -400.0 {
            commands.entity(entity).despawn();
            if dev_balls.get(entity).is_err() {
                commands.trigger(LifeLostEvent);
            }
        }
    }
}

pub fn ball_follow_paddle(
    balls: Query<(&Ball, &mut Transform), Without<Paddle>>,
    paddle: Query<&Transform, (With<Paddle>, Without<Ball>)>,
) {
    let Ok(paddle_transform) = paddle.single() else { return };

    for (ball, mut ball_transform) in balls {
        if ball.stuck {
            ball_transform.translation.x = paddle_transform.translation.x;
            ball_transform.translation.y = paddle_transform.translation.y
                + PADDLE_HEIGHT / 2.0 + BALL_RADIUS + 2.0;
        }
    }
}

/// Dev cheat: Ctrl+O spawns 10 balls
pub fn dev_spawn_balls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    paddle: Query<&Transform, With<Paddle>>,
) {
    if !keyboard.pressed(KeyCode::ControlLeft) || !keyboard.just_pressed(KeyCode::KeyO) {
        return;
    }

    let Ok(paddle_t) = paddle.single() else { return };
    let base_x = paddle_t.translation.x;

    for i in 0..10 {
        let angle = (i as f32 / 10.0) * std::f32::consts::TAU;
        let vx = angle.cos() * BALL_SPEED;
        let vy = angle.sin().abs() * BALL_SPEED + 100.0;

        commands.spawn((
            DevBall,
            Ball {
                speed: BALL_SPEED,
                radius: BALL_RADIUS,
                stuck: false,
            },
            super::level_spawner::LevelEntity,
            RigidBody::Dynamic,
            Collider::circle(BALL_RADIUS),
            Transform::from_xyz(base_x, PADDLE_Y + PADDLE_HEIGHT, 0.0),
            LinearVelocity(Vec2::new(vx, vy.max(200.0))),
            Restitution::new(1.0),
            CollisionLayers::DEFAULT,
            CollisionEventsEnabled,
            Mesh2d(meshes.add(Circle::new(BALL_RADIUS))),
            MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 0.8))),
        ));
    }

    info!("Dev cheat: spawned 10 balls!");
}
