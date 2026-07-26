use bevy::prelude::*;

#[derive(Event)]
pub struct WallHitEvent {
    pub wall: Entity,
}

#[derive(Event)]
pub struct PaddleHitEvent;

#[derive(Event)]
pub struct LifeLostEvent;

#[derive(Event)]
pub struct BrickHitEvent {
    pub brick: Entity,
    pub ball_speed: f32,
}

#[derive(Event)]
pub struct BrickDestroyedEvent {
    pub brick: Entity,
    pub position: Vec2,
    pub brick_type: crate::components::brick::BrickType,
}

#[derive(Event)]
pub struct PowerUpCollectedEvent {
    pub powerup: Entity,
    pub kind: crate::components::powerup::PowerUpKind,
}

#[derive(Event)]
pub struct PowerUpSpawnedEvent;

#[derive(Event)]
pub struct ComboMilestoneEvent {
    pub count: u32,
}

#[derive(Event)]
pub struct LevelClearEvent;

#[derive(Event)]
pub struct GameOverEvent;

#[derive(Event)]
pub struct LaserFireEvent;

#[derive(Event)]
pub struct ShieldActivateEvent;

#[derive(Event)]
pub struct ShieldHitEvent;

#[derive(Event)]
pub struct BallSplitEvent;

#[derive(Event)]
pub struct BallSpeedChangeEvent;

#[derive(Event)]
pub struct BulletTimeEvent {
    pub entering: bool,
}

#[derive(Event)]
pub struct ShockwaveEvent;

#[derive(Event)]
pub struct LightningEvent;

#[derive(Event)]
pub struct TeleportEvent;

#[derive(Event)]
pub struct BrickRegenEvent;
