use avian2d::prelude::*;
use bevy::prelude::*;
use metanoid_core::components::ball::Ball;
use metanoid_core::components::brick::{Brick, BrickType};
use metanoid_core::components::paddle::Paddle;
use metanoid_core::constants::*;
use metanoid_procgen::level::data::{BrickKind, LevelDefinition, SpecialType};
use metanoid_procgen::level::generate::free_run_cells;
use metanoid_visuals::material::ProceduralMaterials;
use metanoid_visuals::recipe::TextureKind;

use super::brick_damage::{BrickDamage, damage_kind_for, is_molten};
use super::brick_motion::BrickMover;
use super::level_progression::ActiveLevelVisuals;
use super::physics_layers::{layers_ball, layers_brick, layers_moving_brick};

#[derive(Component)]
pub struct LevelEntity;

#[derive(Resource)]
pub struct PendingLevel {
    pub level: LevelDefinition,
}

pub fn spawn_bricks(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    level: &LevelDefinition,
    visuals: &ProceduralMaterials,
) {
    let m = level.metrics;
    let bw = m.brick_w;
    let bh = m.brick_h;
    let gap = m.gap;
    let cell = bw + gap;

    let brick_mesh = meshes.add(Rectangle::new(bw, bh));
    let arena_half = (ARENA_WIDTH - WALL_THICKNESS) / 2.0;

    let start_x = -(level.cols as f32 * cell) / 2.0 + cell / 2.0;
    // Pin the grid to the TOP of the arena (row 0 = uppermost).
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

        let health_pct = data.health as f32 / data.max_health.max(1) as f32;
        let kind = damage_kind_for(game_brick_type);
        let material = visuals.brick(kind, health_pct);

        // Horizontal runway (world px) from the free cells either side.
        let (free_left, free_right) = free_run_cells(&level.bricks, data.col, data.row);
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

        let is_moving = data.special == SpecialType::Moving
            && data.motion.is_some()
            && (max_left > 1.0 || max_right > 1.0);

        if data.special == SpecialType::Moving && !is_moving {
            // Demote visually-blocked movers that lost their motion spec.
            game_brick_type = match data.kind {
                BrickKind::Normal => BrickType::Normal,
                BrickKind::MultiHit => BrickType::MultiHit,
                BrickKind::Invincible => BrickType::Invincible,
                BrickKind::Explosive => BrickType::Explosive,
            };
        }

        let layers = if game_brick_type == BrickType::Moving {
            layers_moving_brick()
        } else {
            layers_brick()
        };

        // Damage state (base texture for dynamic cracks).
        let spec = visuals.brick_spec(kind);
        let damage = BrickDamage {
            cracks: Vec::new(),
            base_image: visuals.base_image(kind),
            glow: spec.map(|s| s.glow).unwrap_or(LinearRgba::WHITE),
            molten: is_molten(kind, spec.map(|s| s.texture).unwrap_or(TextureKind::Stone)),
        };

        // Build the mover for moving bricks.
        let mover = if is_moving {
            let motion = data.motion.expect("mover has motion spec");
            let half_w = bw * 0.5;
            let half_h = bh * 0.5;

            // Lane = runway clamped to the arena walls.
            let lane_left = (x - max_left).max(-arena_half + half_w + 2.0);
            let lane_right = (x + max_right).min(arena_half - half_w - 2.0);
            let amp_x = ((lane_right - lane_left) * 0.5).max(2.0);
            let lane_center_x = (lane_left + lane_right) * 0.5;

            // Vertical amplitude bounded by the grid free space and arena top.
            let amp_y = (motion.amp_y_cells * cell * 0.9)
                .min((arena_half - 6.0 - y.abs()).max(0.0))
                .max(0.0);

            Some(BrickMover {
                motion,
                origin: Vec2::new(x, y),
                lane_center_x,
                amp_x,
                amp_y,
                half_w,
                half_h,
                u: motion.phase,
            })
        } else {
            None
        };

        let mut spawn = commands.spawn((
            Brick {
                brick_type: game_brick_type,
                health: data.health,
                max_health: data.max_health,
                brick_half_w: bw * 0.5,
                brick_half_h: bh * 0.5,
                regen_timer: if game_brick_type == BrickType::Regenerating {
                    3.2
                } else {
                    4.0
                },
            },
            damage,
            LevelEntity,
            RigidBody::Kinematic,
            Collider::rectangle(bw, bh),
            Transform::from_xyz(x, y, 0.0),
            layers,
            CollisionEventsEnabled,
            Mesh2d(brick_mesh.clone()),
            MeshMaterial2d(material),
        ));

        if let Some(mover) = mover {
            // Start the mover at its initial path position (inside its runway,
            // so it never spawns overlapping a static brick).
            let pos = mover.path_pos(mover.phase());
            spawn.insert(mover);
            spawn.insert(Transform::from_xyz(pos.x, pos.y, 0.0));
        }
    }
}

pub fn auto_respawn_ball(
    mut commands: Commands,
    balls: Query<&Ball>,
    paddle: Query<&Transform, With<Paddle>>,
    visuals: Option<Res<ActiveLevelVisuals>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if balls.iter().count() > 0 {
        return;
    }
    let Ok(paddle_t) = paddle.single() else {
        return;
    };

    let ball_mat = visuals
        .map(|v| v.materials.ball.clone())
        .unwrap_or_else(|| materials.add(Color::srgb(1.0, 1.0, 1.0)));

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
        MeshMaterial2d(ball_mat),
    ));
}
