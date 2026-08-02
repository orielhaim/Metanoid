use avian2d::prelude::*;
use bevy::camera::{Hdr, ScalingMode};
use bevy::post_process::bloom::Bloom;
use bevy::post_process::effect_stack::{ChromaticAberration, LensDistortion, Vignette};
use bevy::prelude::*;
use bevy_trauma_shake::prelude::*;
use metanoid_core::components::paddle::Paddle;
use metanoid_core::constants::*;

use super::physics_layers::{layers_paddle, layers_wall};

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct ArenaEntity;

/// Marker for arena walls (for bounce SFX / wall hit events).
#[derive(Component)]
pub struct Wall;

#[derive(Resource)]
pub struct ArenaSpawned;

/// Spawn a single persistent camera on Startup. Never despawned.
pub fn setup_persistent_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Hdr,
        GameCamera,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: ARENA_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

/// Add post-processing components to the existing camera when entering gameplay.
pub fn setup_camera_effects(mut commands: Commands, cameras: Query<Entity, With<GameCamera>>) {
    for entity in &cameras {
        commands.entity(entity).insert((
            Shake::default(),
            Bloom {
                intensity: 0.15,
                ..default()
            },
            ChromaticAberration {
                intensity: 0.0,
                ..default()
            },
            Vignette {
                intensity: 0.3,
                radius: 0.8,
                smoothness: 5.0,
                roundness: 1.0,
                ..default()
            },
            LensDistortion {
                intensity: 0.0,
                ..default()
            },
        ));
    }
}

/// Remove post-processing when leaving gameplay.
pub fn teardown_camera_effects(mut commands: Commands, cameras: Query<Entity, With<GameCamera>>) {
    for entity in &cameras {
        commands
            .entity(entity)
            .remove::<Shake>()
            .remove::<Bloom>()
            .remove::<ChromaticAberration>()
            .remove::<Vignette>()
            .remove::<LensDistortion>();
    }
}

pub fn setup_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    arena_spawned: Option<Res<ArenaSpawned>>,
) {
    if arena_spawned.is_some() {
        return;
    }
    commands.insert_resource(ArenaSpawned);
    commands.insert_resource(Gravity(Vec2::ZERO));

    let wall_mesh = meshes.add(Rectangle::new(ARENA_WIDTH, WALL_THICKNESS));
    let wall_mat = materials.add(Color::srgb(0.3, 0.3, 0.3));
    let half_w = (ARENA_WIDTH - WALL_THICKNESS) / 2.0;
    let half_h = (ARENA_HEIGHT - WALL_THICKNESS) / 2.0;

    for (x, y, w, h) in [
        (0.0, half_h, ARENA_WIDTH, WALL_THICKNESS),
        (-half_w, 0.0, WALL_THICKNESS, ARENA_HEIGHT),
        (half_w, 0.0, WALL_THICKNESS, ARENA_HEIGHT),
    ] {
        commands.spawn((
            ArenaEntity,
            Wall,
            RigidBody::Static,
            Collider::rectangle(w, h),
            Transform::from_xyz(x, y, 0.0),
            layers_wall(),
            CollisionEventsEnabled,
            Mesh2d(wall_mesh.clone()),
            MeshMaterial2d(wall_mat.clone()),
        ));
    }

    let paddle_mesh = meshes.add(Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT));
    let paddle_mat = materials.add(Color::srgb(0.9, 0.9, 0.95));
    commands.spawn((
        ArenaEntity,
        Paddle::default(),
        RigidBody::Kinematic,
        Collider::rectangle(PADDLE_WIDTH, PADDLE_HEIGHT),
        Transform::from_xyz(0.0, PADDLE_Y, 0.0),
        LinearVelocity::default(),
        layers_paddle(),
        CollisionEventsEnabled,
        Mesh2d(paddle_mesh),
        MeshMaterial2d(paddle_mat),
    ));
}
