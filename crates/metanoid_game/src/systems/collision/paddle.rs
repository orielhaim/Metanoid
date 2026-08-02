//! Paddle collision with tennis-style english / sidespin.

use avian2d::prelude::*;
use bevy::prelude::*;
use metanoid_core::components::ball::Ball;
use metanoid_core::components::paddle::Paddle;
use metanoid_core::constants::PADDLE_SPEED;
use metanoid_core::events::PaddleHitEvent;

/// How much paddle motion imparts spin (-1..1 scale). Subtle by design.
const SPIN_FROM_PADDLE_MOTION: f32 = 0.42;
/// How much hit offset on the paddle face imparts spin.
const SPIN_FROM_HIT_OFFSET: f32 = 0.18;
/// How strongly paddle velocity bleeds into ball velocity (px/s scale).
const PADDLE_VEL_TRANSFER: f32 = 0.22;
/// Max absolute spin retained on the ball.
const SPIN_CLAMP: f32 = 1.0;

pub fn ball_paddle_collision(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut balls: Query<(&mut LinearVelocity, &mut Ball, &Transform), With<Ball>>,
    paddles: Query<(&Transform, &LinearVelocity, &Paddle), (With<Paddle>, Without<Ball>)>,
) {
    for event in collision_reader.read() {
        let (ball_entity, paddle_entity) =
            if balls.get(event.collider1).is_ok() && paddles.get(event.collider2).is_ok() {
                (event.collider1, event.collider2)
            } else if balls.get(event.collider2).is_ok() && paddles.get(event.collider1).is_ok() {
                (event.collider2, event.collider1)
            } else {
                continue;
            };

        let Ok((mut ball_vel, mut ball, ball_transform)) = balls.get_mut(ball_entity) else {
            continue;
        };
        let Ok((paddle_transform, paddle_vel, paddle)) = paddles.get(paddle_entity) else {
            continue;
        };

        let ball_pos = ball_transform.translation.truncate();
        let paddle_pos = paddle_transform.translation.truncate();
        let half_w = paddle.size.x / 2.0;

        // Ignore underside hits (ball already below paddle center)
        if ball_pos.y < paddle_pos.y {
            continue;
        }

        // Normalized hit offset on paddle face (-1 left .. +1 right)
        let hit_offset = ((ball_pos.x - paddle_pos.x) / half_w).clamp(-1.0, 1.0);

        // Paddle motion as -1..1 (requires moving into the shot for max english)
        let paddle_motion = (paddle_vel.0.x / PADDLE_SPEED.max(1.0)).clamp(-1.0, 1.0);

        // Outgoing launch angle from hit position (classic arkanoid) + small spin tilt
        let base_angle = hit_offset * std::f32::consts::FRAC_PI_3; // +/- 60 deg
        let spin_tilt = ball.spin * 0.12 + paddle_motion * 0.08;
        let angle = (base_angle + spin_tilt).clamp(
            -std::f32::consts::FRAC_PI_3 * 1.05,
            std::f32::consts::FRAC_PI_3 * 1.05,
        );

        let speed = ball_vel.0.length().max(ball.speed * 0.85).max(380.0);
        let mut new_vel = Vec2::new(angle.sin(), angle.cos().abs().max(0.35)) * speed;

        // Direct transfer of a slice of paddle velocity (tennis "push")
        new_vel.x += paddle_vel.0.x * PADDLE_VEL_TRANSFER;

        // Update spin: blend old spin with new english from motion + face offset
        let spin_impulse =
            paddle_motion * SPIN_FROM_PADDLE_MOTION + hit_offset * SPIN_FROM_HIT_OFFSET;
        ball.spin = (ball.spin * 0.35 + spin_impulse).clamp(-SPIN_CLAMP, SPIN_CLAMP);

        // Tiny lateral bias from resulting spin so it "curves" off the paddle
        new_vel.x += ball.spin * 28.0;

        // Keep upward
        if new_vel.y < 120.0 {
            new_vel.y = 120.0;
        }

        ball_vel.0 = new_vel;
        commands.trigger(PaddleHitEvent);
    }
}

/// Apply subtle magnus-style deflection while the ball is in flight / on brick contacts.
pub fn apply_ball_spin_physics(
    time: Res<Time>,
    mut balls: Query<(&mut LinearVelocity, &mut Ball, &mut Transform), With<Ball>>,
) {
    let dt = time.delta_secs();
    for (mut vel, mut ball, mut transform) in &mut balls {
        if ball.stuck || ball.spin.abs() < 0.001 {
            // Still spin the sprite slightly when stuck? no
            continue;
        }

        let speed = vel.0.length();
        if speed < 1.0 {
            continue;
        }

        // Perpendicular to velocity: positive spin curves "right" relative to motion
        let dir = vel.0 / speed;
        let perp = Vec2::new(-dir.y, dir.x);
        // Very subtle continuous curve (tennis sidespin feel)
        let curve = ball.spin * 55.0 * dt;
        vel.0 += perp * curve;

        // Preserve approximate speed
        let new_speed = vel.0.length();
        if new_speed > 1.0 {
            vel.0 *= speed / new_speed;
        }

        // Visual roll of the ball sprite
        let roll = ball.spin * dt * 10.0 + dir.x * dt * 4.0;
        transform.rotate_z(roll);

        // Natural spin decay
        ball.spin *= (1.0 - 0.35 * dt).clamp(0.0, 1.0);
    }
}

/// When the ball hits a brick, bleed a bit of spin into lateral deflection (already in velocity
/// path via continuous spin; also damp spin on hard contact).
pub fn damp_spin_on_brick_hit(
    mut collision_reader: MessageReader<CollisionStart>,
    mut balls: Query<&mut Ball>,
    bricks: Query<Entity, With<metanoid_core::components::brick::Brick>>,
) {
    for event in collision_reader.read() {
        let ball_entity =
            if balls.get(event.collider1).is_ok() && bricks.get(event.collider2).is_ok() {
                event.collider1
            } else if balls.get(event.collider2).is_ok() && bricks.get(event.collider1).is_ok() {
                event.collider2
            } else {
                continue;
            };

        if let Ok(mut ball) = balls.get_mut(ball_entity) {
            // Hard contact reduces spin but leaves a residual curve for a bounce or two
            ball.spin *= 0.72;
        }
    }
}
