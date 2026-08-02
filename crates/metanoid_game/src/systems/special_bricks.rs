use bevy::prelude::*;
use metanoid_core::components::brick::{Brick, BrickType};
use metanoid_core::constants::{ARENA_WIDTH, WALL_THICKNESS};
use metanoid_core::events::BrickRegenEvent;

/// Moves bricks horizontally within hard world-space bounds (never into neighbors).
pub fn update_moving_bricks(time: Res<Time>, mut bricks: Query<(&Brick, &mut Transform)>) {
    let arena_half = (ARENA_WIDTH - WALL_THICKNESS) / 2.0;

    // Snapshot positions of all solid bricks for multi-mover separation
    let blockers: Vec<(f32, f32, f32)> = bricks
        .iter()
        .filter(|(b, _)| b.brick_type != BrickType::Moving || b.move_range <= 0.0)
        .map(|(b, t)| (t.translation.x, t.translation.y, b.brick_half_w))
        .collect();

    let mut movers: Vec<(f32, f32, f32, f32)> = bricks
        .iter()
        .filter(|(b, _)| b.brick_type == BrickType::Moving && b.move_range > 0.0)
        .map(|(b, t)| {
            (
                t.translation.x,
                t.translation.y,
                b.brick_half_w,
                b.move_origin_x,
            )
        })
        .collect();

    for (brick, mut transform) in &mut bricks {
        if brick.brick_type != BrickType::Moving || brick.move_range <= 0.0 {
            continue;
        }

        let phase = brick.move_origin_x * 0.07;
        let t = time.elapsed_secs() * brick.move_speed + phase;
        // Map sine from [-1,1] into [move_min_x, move_max_x]
        let u = (t.sin() + 1.0) * 0.5; // 0..1
        let mut x = brick.move_min_x + (brick.move_max_x - brick.move_min_x) * u;

        // Arena clamp using this brick's half-width
        let half = brick.brick_half_w + 2.0;
        x = x.clamp(-arena_half + half, arena_half - half);

        // Soft separation from static blockers on roughly the same row
        let y = transform.translation.y;
        for &(bx, by, bhw) in &blockers {
            if (by - y).abs() > brick.brick_half_w.max(12.0) {
                continue;
            }
            let min_dist = brick.brick_half_w + bhw + 2.0;
            let dx = x - bx;
            if dx.abs() < min_dist {
                if dx >= 0.0 {
                    x = bx + min_dist;
                } else {
                    x = bx - min_dist;
                }
            }
        }

        // Separation from other movers (use previous-frame snapshot)
        for &(mx, my, mhw, origin) in &movers {
            if (origin - brick.move_origin_x).abs() < 0.1 {
                continue; // self
            }
            if (my - y).abs() > brick.brick_half_w.max(12.0) {
                continue;
            }
            let min_dist = brick.brick_half_w + mhw + 2.0;
            let dx = x - mx;
            if dx.abs() < min_dist {
                if dx >= 0.0 {
                    x = mx + min_dist;
                } else {
                    x = mx - min_dist;
                }
            }
        }

        // Re-clamp to legal lane after separation
        x = x.clamp(brick.move_min_x, brick.move_max_x);
        transform.translation.x = x;
    }

    // Silence unused mut warning if any
    let _ = &mut movers;
}

/// Regenerates health on regenerating bricks after a delay.
pub fn update_regen_bricks(time: Res<Time>, mut commands: Commands, mut bricks: Query<&mut Brick>) {
    for mut brick in &mut bricks {
        if brick.brick_type != BrickType::Regenerating {
            continue;
        }
        if brick.health == 0 {
            continue;
        }
        if brick.health < brick.max_health {
            brick.regen_timer -= time.delta_secs();
            if brick.regen_timer <= 0.0 {
                brick.health = brick.max_health;
                brick.regen_timer = 3.5;
                commands.trigger(BrickRegenEvent);
            }
        }
    }
}
