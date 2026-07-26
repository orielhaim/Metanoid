use bevy::prelude::*;

#[derive(Component)]
pub struct Ball {
    pub speed: f32,
    pub radius: f32,
    pub stuck: bool,
}

impl Default for Ball {
    fn default() -> Self {
        Self {
            speed: crate::constants::BALL_SPEED,
            radius: crate::constants::BALL_RADIUS,
            stuck: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BallEffectKind {
    #[default]
    Fireball,
    BrickThru,
    MegaBall,
    ShrinkBall,
    FastBall,
    SlowBall,
    MagnetBall,
    PhantomBall,
}

#[derive(Component)]
pub struct BallEffect {
    pub kind: BallEffectKind,
    pub timer: Timer,
}

impl BallEffect {
    pub fn new(kind: BallEffectKind, duration_secs: f32) -> Self {
        Self {
            kind,
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }
}

#[derive(Component)]
pub struct Fireball;

#[derive(Component)]
pub struct BrickThru;

#[derive(Component)]
pub struct SplitBall;
