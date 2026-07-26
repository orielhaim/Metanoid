use bevy::prelude::*;
use avian2d::prelude::*;
use metanoid_core::components::ball::Ball;
use metanoid_core::components::paddle::Paddle;
use metanoid_core::events::PaddleHitEvent;

pub fn ball_paddle_collision(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut balls: Query<(&mut LinearVelocity, &Transform), With<Ball>>,
    paddles: Query<(&Transform, &LinearVelocity), (With<Paddle>, Without<Ball>)>,
) {
    for event in collision_reader.read() {
        let (ball_entity, paddle_entity) =
            if balls.get(event.collider1).is_ok() && paddles.get(event.collider2).is_ok() {
                (event.collider1, event.collider2)
            } else if balls.get(event.collider2).is_ok() && paddles.get(event.collider1).is_ok()
            {
                (event.collider2, event.collider1)
            } else {
                continue;
            };

        let Ok((mut ball_vel, ball_transform)) = balls.get_mut(ball_entity) else {
            continue;
        };
        let Ok((paddle_transform, paddle_vel)) = paddles.get(paddle_entity) else {
            continue;
        };

        let ball_pos = ball_transform.translation.truncate();
        let paddle_pos = paddle_transform.translation.truncate();
        let paddle_size = Vec2::new(120.0, 20.0);

        if ball_pos.y < paddle_pos.y {
            continue;
        }

        let relative = (ball_pos.x - paddle_pos.x) / (paddle_size.x / 2.0);
        let clamped = relative.clamp(-1.0, 1.0);
        let angle = clamped * std::f32::consts::FRAC_PI_3;
        let speed = ball_vel.0.length().max(400.0);

        let new_velocity = Vec2::new(angle.sin(), angle.cos()) * speed;
        let paddle_influence = paddle_vel.0.x * 0.2;

        ball_vel.0 = Vec2::new(new_velocity.x + paddle_influence, new_velocity.y);
        commands.trigger(PaddleHitEvent);
    }
}
