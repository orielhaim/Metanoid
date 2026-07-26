use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PowerUpKind {
    #[default]
    Fireball,
    MegaBall,
    SplitBall,
    FastBall,
    SlowBall,
    LaserPaddle,
    GrabPaddle,
    ExpandPaddle,
    ShrinkPaddle,
    Shield,
    ExtraLife,
    DoublePoints,
    LevelWarp,
    KillPaddle,
    TimeSlow,
    FallingBricks,
    Zap,
    Explode,
    ExpandExploding,
    Lightning,
    Shockwave,
    ShuffleBricks,
    Blackout,
}

#[derive(Component)]
pub struct PowerUp {
    pub kind: PowerUpKind,
    pub fall_speed: f32,
}

impl Default for PowerUp {
    fn default() -> Self {
        Self {
            kind: PowerUpKind::Fireball,
            fall_speed: crate::constants::POWERUP_FALL_SPEED,
        }
    }
}
