use bevy::prelude::*;
use avian2d::prelude::*;
use metanoid_core::components::ball::Ball;
use metanoid_core::components::brick::{Brick, BrickType};
use metanoid_core::components::paddle::Paddle;
use metanoid_core::constants::*;
use metanoid_procgen::biome::parameters::BiomeParams;
use metanoid_procgen::level::data::{BrickKind, LevelDefinition, SpecialType};
use std::collections::HashMap;

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
    let brick_mesh = meshes.add(Rectangle::new(BRICK_WIDTH, BRICK_HEIGHT));
    let mut mat_cache: HashMap<(BrickKind, u32), Handle<ColorMaterial>> = HashMap::new();

    let start_x = -(level.cols as f32 * (BRICK_WIDTH + BRICK_GAP)) / 2.0
        + (BRICK_WIDTH + BRICK_GAP) / 2.0;

    // Place bricks below the top wall, centered vertically between wall and paddle
    let grid_height = level.rows as f32 * (BRICK_HEIGHT + BRICK_GAP);
    let top_wall_y = (ARENA_HEIGHT - WALL_THICKNESS) / 2.0;
    let start_y = top_wall_y - WALL_THICKNESS - grid_height + (BRICK_HEIGHT + BRICK_GAP) / 2.0;

    for data in &level.bricks {
        let x = start_x + data.col as f32 * (BRICK_WIDTH + BRICK_GAP);
        let y = start_y + (level.rows - 1 - data.row) as f32 * (BRICK_HEIGHT + BRICK_GAP);

        let game_brick_type = match data.special {
            SpecialType::Moving => BrickType::Moving,
            SpecialType::Regenerating => BrickType::Regenerating,
            _ => match data.kind {
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

        let move_range = if data.special == SpecialType::Moving {
            BRICK_WIDTH * 1.5
        } else {
            0.0
        };
        let move_speed = if data.special == SpecialType::Moving {
            0.8 + (data.col as f32 * 0.1)
        } else {
            0.0
        };

        commands.spawn((
            Brick {
                brick_type: game_brick_type,
                health: data.health,
                max_health: data.max_health,
                move_origin_x: x,
                move_range,
                move_speed,
                regen_timer: 4.0,
            },
            LevelEntity,
            RigidBody::Static,
            Collider::rectangle(BRICK_WIDTH, BRICK_HEIGHT),
            Transform::from_xyz(x, y, 0.0),
            CollisionLayers::DEFAULT,
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
            let ratio = health as f32 / 4.0;
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
    let Ok(paddle_t) = paddle.single() else { return };

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
        CollisionLayers::DEFAULT,
        CollisionEventsEnabled,
        Mesh2d(meshes.add(Circle::new(BALL_RADIUS))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
    ));
}
