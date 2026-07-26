use bevy::prelude::*;

#[derive(Component)]
pub struct Paddle {
    pub speed: f32,
    pub size: Vec2,
}

impl Default for Paddle {
    fn default() -> Self {
        Self {
            speed: crate::constants::PADDLE_SPEED,
            size: Vec2::new(
                crate::constants::PADDLE_WIDTH,
                crate::constants::PADDLE_HEIGHT,
            ),
        }
    }
}

#[derive(Component)]
pub struct PaddleEffect {
    pub kind: PaddleEffectKind,
    pub timer: Timer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaddleEffectKind {
    Expand,
    Shrink,
    Laser,
    Grab,
    Shield,
}

impl PaddleEffect {
    pub fn new(kind: PaddleEffectKind, duration_secs: f32) -> Self {
        Self {
            kind,
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }
}

#[derive(Component)]
pub struct GrabbedBall {
    pub offset_x: f32,
}

#[derive(Component)]
pub struct ShieldBarrier;

#[derive(Component)]
pub struct LaserPaddle {
    pub cooldown: Timer,
}
