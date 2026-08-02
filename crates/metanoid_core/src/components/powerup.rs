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

impl PowerUpKind {
    /// Short ASCII label for floating popups.
    pub fn display_name(self) -> &'static str {
        match self {
            PowerUpKind::Fireball => "FIREBALL",
            PowerUpKind::MegaBall => "MEGA BALL",
            PowerUpKind::SplitBall => "SPLIT BALL",
            PowerUpKind::FastBall => "FAST BALL",
            PowerUpKind::SlowBall => "SLOW BALL",
            PowerUpKind::LaserPaddle => "LASERS",
            PowerUpKind::GrabPaddle => "GRAB",
            PowerUpKind::ExpandPaddle => "EXPAND",
            PowerUpKind::ShrinkPaddle => "SHRINK",
            PowerUpKind::Shield => "SHIELD",
            PowerUpKind::ExtraLife => "EXTRA LIFE",
            PowerUpKind::DoublePoints => "DOUBLE POINTS",
            PowerUpKind::LevelWarp => "LEVEL WARP",
            PowerUpKind::KillPaddle => "KILL PADDLE",
            PowerUpKind::TimeSlow => "BULLET TIME",
            PowerUpKind::FallingBricks => "FALLING BRICKS",
            PowerUpKind::Zap => "ZAP",
            PowerUpKind::Explode => "EXPLODE",
            PowerUpKind::ExpandExploding => "MORE BOMBS",
            PowerUpKind::Lightning => "LIGHTNING",
            PowerUpKind::Shockwave => "SHOCKWAVE",
            PowerUpKind::ShuffleBricks => "SHUFFLE",
            PowerUpKind::Blackout => "BLACKOUT",
        }
    }

    pub fn is_negative(self) -> bool {
        matches!(
            self,
            PowerUpKind::KillPaddle
                | PowerUpKind::ShrinkPaddle
                | PowerUpKind::FallingBricks
                | PowerUpKind::Blackout
        )
    }
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
