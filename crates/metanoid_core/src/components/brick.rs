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
    /// Half-amplitude of sine travel (legacy; bounds take priority when set).
    pub move_range: f32,
    pub move_speed: f32,
    pub regen_timer: f32,
    /// Hard world-space clamp so movers never enter neighbor bricks.
    pub move_min_x: f32,
    pub move_max_x: f32,
    pub brick_half_w: f32,
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
            move_min_x: 0.0,
            move_max_x: 0.0,
            brick_half_w: 40.0,
        }
    }
}

impl Brick {
    /// Whether this brick must be cleared for the level to complete.
    ///
    /// Invincible bricks are ignored. Zero-health bricks are treated as already
    /// dead (important while despawn is still deferred for the frame).
    pub fn blocks_level_clear(&self) -> bool {
        self.brick_type != BrickType::Invincible && self.health > 0
    }

    /// True for any brick type that can be destroyed by normal play / fireball.
    pub fn is_clearable_type(&self) -> bool {
        self.brick_type != BrickType::Invincible
    }
}

/// Pure helper used by gameplay systems and unit tests.
pub fn count_blocking_bricks<'a, I>(bricks: I) -> usize
where
    I: IntoIterator<Item = &'a Brick>,
{
    bricks
        .into_iter()
        .filter(|b| b.blocks_level_clear())
        .count()
}

/// Level is clear when there are no blocking bricks left.
/// `level_armed` must be true (bricks finished spawning).
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
