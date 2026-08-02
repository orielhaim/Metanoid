//! Avian2D collision layers so moving bricks pass through other bricks.

use avian2d::prelude::*;

/// Game collision memberships.
///
/// - Ball hits paddle, walls, static bricks, and moving bricks.
/// - Static bricks and moving bricks never interact with each other.
/// - Power-ups only care about the paddle.
#[derive(PhysicsLayer, Default, Clone, Copy, Debug)]
pub enum GameLayer {
    #[default]
    Default,
    Ball,
    Paddle,
    Wall,
    Brick,
    MovingBrick,
    PowerUp,
    Projectile,
}

pub fn layers_ball() -> CollisionLayers {
    CollisionLayers::new(
        [GameLayer::Ball],
        [
            GameLayer::Paddle,
            GameLayer::Wall,
            GameLayer::Brick,
            GameLayer::MovingBrick,
        ],
    )
}

pub fn layers_paddle() -> CollisionLayers {
    CollisionLayers::new([GameLayer::Paddle], [GameLayer::Ball, GameLayer::PowerUp])
}

pub fn layers_wall() -> CollisionLayers {
    CollisionLayers::new([GameLayer::Wall], [GameLayer::Ball])
}

pub fn layers_brick() -> CollisionLayers {
    // Bricks never collide with other bricks (including movers).
    CollisionLayers::new([GameLayer::Brick], [GameLayer::Ball, GameLayer::Projectile])
}

pub fn layers_moving_brick() -> CollisionLayers {
    // Moving bricks pass through neighboring bricks; ball + lasers still hit.
    CollisionLayers::new(
        [GameLayer::MovingBrick],
        [GameLayer::Ball, GameLayer::Projectile],
    )
}

pub fn layers_powerup() -> CollisionLayers {
    CollisionLayers::new([GameLayer::PowerUp], [GameLayer::Paddle])
}

/// Lasers that hit bricks but ignore other lasers/powerups.
pub fn layers_projectile() -> CollisionLayers {
    CollisionLayers::new(
        [GameLayer::Projectile],
        [GameLayer::Brick, GameLayer::MovingBrick],
    )
}

pub fn layers_shield() -> CollisionLayers {
    CollisionLayers::new([GameLayer::Wall], [GameLayer::Ball])
}
