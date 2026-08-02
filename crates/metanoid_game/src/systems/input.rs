use avian2d::prelude::*;
use bevy::prelude::*;
use metanoid_core::components::ball::Ball;
use metanoid_core::components::paddle::Paddle;
use metanoid_core::constants::{ARENA_WIDTH, WALL_THICKNESS};
use metanoid_core::settings::GameSettings;

pub fn paddle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Paddle, &mut LinearVelocity)>,
) {
    let Ok((paddle, mut velocity)) = query.single_mut() else {
        return;
    };

    let mut direction = 0.0;
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        direction -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        direction += 1.0;
    }

    velocity.0 = Vec2::new(direction * paddle.speed, 0.0);
}

/// Clamp the paddle inside the arena and give it a subtle "charging" pulse
/// while a ball is stuck on it awaiting launch.
pub fn clamp_paddle_position(
    time: Res<Time>,
    settings: Res<GameSettings>,
    balls: Query<&Ball>,
    mut query: Query<(&Paddle, &mut Transform), With<Paddle>>,
) {
    let Ok((paddle, mut transform)) = query.single_mut() else {
        return;
    };

    let half_arena = (ARENA_WIDTH - WALL_THICKNESS) / 2.0;
    let half_paddle = paddle.size.x / 2.0;
    let min_x = -half_arena + half_paddle;
    let max_x = half_arena - half_paddle;
    transform.translation.x = transform.translation.x.clamp(min_x, max_x);

    let charging = balls.iter().any(|b| b.stuck);
    let target = if charging && !settings.reduce_motion {
        1.0 + (time.elapsed_secs() * 6.0).sin() * 0.03
    } else {
        1.0
    };
    transform.scale = transform.scale.lerp(Vec3::splat(target), 0.15);
}
