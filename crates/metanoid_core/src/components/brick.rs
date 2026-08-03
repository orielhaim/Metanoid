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
    pub brick_half_w: f32,
    pub brick_half_h: f32,
    pub regen_timer: f32,
}

impl Default for Brick {
    fn default() -> Self {
        Self {
            brick_type: BrickType::Normal,
            health: 1,
            max_health: 1,
            brick_half_w: 40.0,
            brick_half_h: 15.0,
            regen_timer: 0.0,
        }
    }
}

impl Brick {
    pub fn blocks_level_clear(&self) -> bool {
        self.brick_type != BrickType::Invincible && self.health > 0
    }

    pub fn is_clearable_type(&self) -> bool {
        self.brick_type != BrickType::Invincible
    }
}

pub fn count_blocking_bricks<'a, I>(bricks: I) -> usize
where
    I: IntoIterator<Item = &'a Brick>,
{
    bricks
        .into_iter()
        .filter(|b| b.blocks_level_clear())
        .count()
}

pub fn should_clear_level(level_armed: bool, blocking_remaining: usize) -> bool {
    level_armed && blocking_remaining == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brick(t: BrickType, health: u32) -> Brick {
        Brick {
            brick_type: t,
            health,
            max_health: health.max(1),
            ..Default::default()
        }
    }

    #[test]
    fn invincible_does_not_block() {
        assert!(!brick(BrickType::Invincible, 1).blocks_level_clear());
    }

    #[test]
    fn zero_health_does_not_block() {
        assert!(!brick(BrickType::Normal, 0).blocks_level_clear());
        assert!(!brick(BrickType::Moving, 0).blocks_level_clear());
        assert!(!brick(BrickType::Regenerating, 0).blocks_level_clear());
    }

    #[test]
    fn living_clearables_block() {
        assert!(brick(BrickType::Normal, 1).blocks_level_clear());
        assert!(brick(BrickType::MultiHit, 3).blocks_level_clear());
        assert!(brick(BrickType::Explosive, 1).blocks_level_clear());
        assert!(brick(BrickType::Moving, 1).blocks_level_clear());
        assert!(brick(BrickType::Regenerating, 2).blocks_level_clear());
    }

    #[test]
    fn count_and_should_clear() {
        let mix = vec![
            brick(BrickType::Normal, 1),
            brick(BrickType::Invincible, 1),
            brick(BrickType::Normal, 0), // deferred despawn
        ];
        assert_eq!(count_blocking_bricks(&mix), 1);
        assert!(!should_clear_level(true, 1));
        assert!(should_clear_level(true, 0));
        assert!(!should_clear_level(false, 0)); // not armed yet
    }

    #[test]
    fn only_invincible_left_is_clear() {
        let left = vec![
            brick(BrickType::Invincible, 1),
            brick(BrickType::Invincible, 1),
        ];
        assert_eq!(count_blocking_bricks(&left), 0);
        assert!(should_clear_level(true, 0));
    }

    #[test]
    fn empty_world_is_clear_when_armed() {
        let none: Vec<Brick> = vec![];
        assert_eq!(count_blocking_bricks(&none), 0);
        assert!(should_clear_level(true, 0));
    }
}
