use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrickType {
    #[default]
    Normal,
    MultiHit,
    Invincible,
    Explosive,
    Moving,
    Regenerating,
}

#[derive(Component)]
pub struct Brick {
    pub brick_type: BrickType,
    pub health: u32,
    pub max_health: u32,
    pub move_origin_x: f32,
    pub move_range: f32,
    pub move_speed: f32,
    pub regen_timer: f32,
}

impl Default for Brick {
    fn default() -> Self {
        Self {
            brick_type: BrickType::Normal,
            health: 1,
            max_health: 1,
            move_origin_x: 0.0,
            move_range: 0.0,
            move_speed: 0.0,
            regen_timer: 0.0,
        }
    }
}
