use avian2d::prelude::*;
use bevy::prelude::*;
use metanoid_core::components::ball::Ball;
use metanoid_core::components::brick::{Brick, BrickType};
use metanoid_core::components::paddle::Paddle;
use metanoid_core::constants::*;
use metanoid_procgen::biome::parameters::BiomeParams;
use metanoid_procgen::level::data::{BrickKind, LevelDefinition, SpecialType};
use metanoid_procgen::level::generate::free_run_cells;
use std::collections::HashMap;

use super::physics_layers::{layers_ball, layers_brick, layers_moving_brick};

#[derive(Component)]
pub struct LevelEntity;

#[derive(Resource)]
pub struct PendingLevel {
    pub level: LevelDefinition,
    pub params: BiomeParams,
}

pub fn spawn_bricks(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    level: &LevelDefinition,
    params: &BiomeParams,
) {
    let m = level.metrics;
    let bw = m.brick_w;
    let bh = m.brick_h;
    let gap = m.gap;
    let cell = bw + gap;

    let brick_mesh = meshes.add(Rectangle::new(bw, bh));
    let mut mat_cache: HashMap<(BrickKind, u32), Handle<ColorMaterial>> = HashMap::new();

    let start_x = -(level.cols as f32 * cell) / 2.0 + cell / 2.0;
    // Pin the grid to the TOP of the arena (row 0 = uppermost).
    // Previous formula subtracted full grid height and pushed stages toward the paddle.
    let top_wall_y = (ARENA_HEIGHT - WALL_THICKNESS) / 2.0;
    let top_padding = 18.0;
    let start_y = top_wall_y - WALL_THICKNESS - top_padding - bh * 0.5;

    for data in &level.bricks {
        let x = start_x + data.col as f32 * cell;
        // Rows increase downward from the top band
        let y = start_y - data.row as f32 * (bh + gap);

        let mut game_brick_type = match data.special {
            SpecialType::Moving => BrickType::Moving,
            SpecialType::Regenerating => BrickType::Regenerating,
            SpecialType::Teleport => match data.kind {
                BrickKind::Normal => BrickType::Normal,
                BrickKind::MultiHit => BrickType::MultiHit,
                BrickKind::Invincible => BrickType::Invincible,
                BrickKind::Explosive => BrickType::Explosive,
            },
            SpecialType::None => match data.kind {
                BrickKind::Normal => BrickType::Normal,
                BrickKind::MultiHit => BrickType::MultiHit,
                BrickKind::Invincible => BrickType::Invincible,
                BrickKind::Explosive => BrickType::Explosive,
            },
        };

        let key = (data.kind, data.health);
        let material = mat_cache
            .entry(key)
            .or_insert_with(|| materials.add(brick_color(data.kind, data.health, params)))
            .clone();

        let (free_left, free_right) = free_run_cells(&level.bricks, data.col, data.row);
        // Travel at most into free empty cells, minus a safety margin so AABBs never touch.
        let margin = 3.0;
        let max_left = if free_left > 0 {
            free_left as f32 * cell - margin
        } else {
            0.0
        };
        let max_right = if free_right > 0 {
            free_right as f32 * cell - margin
        } else {
            0.0
        };

        let is_moving = data.special == SpecialType::Moving && (max_left > 1.0 || max_right > 1.0);
        let (move_min_x, move_max_x, move_range, move_speed) = if is_moving {
            let min_x = x - max_left;
            let max_x = x + max_right;
            // Symmetric amplitude used for sine; clamped to asymmetric bounds each frame
            let range = max_left.min(max_right).max(max_left.max(max_right) * 0.5);
            (
                min_x,
                max_x,
                range.max(0.0),
                0.55 + (data.col as f32 * 0.07) + (data.row as f32 * 0.04),
            )
        } else {
            if data.special == SpecialType::Moving {
                // Demote visually-blocked movers
                game_brick_type = match data.kind {
                    BrickKind::Normal => BrickType::Normal,
                    BrickKind::MultiHit => BrickType::MultiHit,
                    BrickKind::Invincible => BrickType::Invincible,
                    BrickKind::Explosive => BrickType::Explosive,
                };
            }
            (x, x, 0.0, 0.0)
        };

        let layers = if game_brick_type == BrickType::Moving {
            layers_moving_brick()
        } else {
            layers_brick()
        };

        commands.spawn((
            Brick {
                brick_type: game_brick_type,
                health: data.health,
                max_health: data.max_health,
                move_origin_x: x,
                move_range,
                move_speed,
                regen_timer: if game_brick_type == BrickType::Regenerating {
                    3.2
                } else {
                    4.0
                },
                move_min_x,
                move_max_x,
                brick_half_w: bw * 0.5,
            },
            LevelEntity,
            RigidBody::Kinematic,
            Collider::rectangle(bw, bh),
            Transform::from_xyz(x, y, 0.0),
            layers,
            CollisionEventsEnabled,
            Mesh2d(brick_mesh.clone()),
            MeshMaterial2d(material),
        ));
    }
}

fn brick_color(kind: BrickKind, health: u32, params: &BiomeParams) -> Color {
    let hue_base = params.temperature * 60.0 + 200.0;
    match kind {
        BrickKind::Normal => Color::hsl(hue_base, 0.7, 0.5),
        BrickKind::MultiHit => {
            let ratio = health as f32 / 5.0;
            Color::hsl(50.0 - ratio * 50.0, 0.9, 0.4 + ratio * 0.2)
        }
        BrickKind::Invincible => Color::srgb(0.6, 0.6, 0.65),
        BrickKind::Explosive => Color::hsl(10.0, 0.9, 0.45),
    }
}

pub fn auto_respawn_ball(
    mut commands: Commands,
    balls: Query<&Ball>,
    paddle: Query<&Transform, With<Paddle>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if balls.iter().count() > 0 {
        return;
    }
    let Ok(paddle_t) = paddle.single() else {
        return;
    };

    commands.spawn((
        Ball::default(),
        LevelEntity,
        RigidBody::Dynamic,
        Collider::circle(BALL_RADIUS),
        Transform::from_xyz(
            paddle_t.translation.x,
            PADDLE_Y + PADDLE_HEIGHT / 2.0 + BALL_RADIUS + 2.0,
            0.0,
        ),
        LinearVelocity::default(),
        Restitution::new(1.0),
        layers_ball(),
        CollisionEventsEnabled,
        Mesh2d(meshes.add(Circle::new(BALL_RADIUS))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
    ));
}
